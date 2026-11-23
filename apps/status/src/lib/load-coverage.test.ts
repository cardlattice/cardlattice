import { createHash, webcrypto } from "node:crypto";
import { afterEach, expect, it, vi } from "vitest";
import { catalogueDataUrl, loadCoverage } from "./load-coverage.ts";
const revision = "a".repeat(40);
const records = [
  { id: "pokemon", metadata: { kind: "game", name: "Pokémon" } },
  { id: "pokemon.base", metadata: { kind: "set", name: "Base Set", gameId: "pokemon" } },
  {
    id: "card",
    metadata: {
      kind: "printing",
      setId: "pokemon.base",
      language: "nl",
      region: "global",
      rarity: "common",
      attributes: { finishes: ["normal"], illustrators: ["Artist"] },
    },
  },
];
function fixture(overrides = {}) {
  const raw = JSON.stringify({ sourceCommit: revision, records });
  const sha256 = createHash("sha256").update(raw).digest("hex");
  return {
    raw,
    manifest: {
      filename: `catalogue-${sha256}.json`,
      sha256,
      sourceCommit: revision,
      counts: { printing: 1 },
      ...overrides,
    },
  };
}
function mockSnapshot(overrides = {}, body?: string) {
  const { raw, manifest } = fixture(overrides);
  const fetcher = vi
    .fn()
    .mockResolvedValueOnce(new Response(JSON.stringify(manifest)))
    .mockResolvedValueOnce(new Response(body ?? raw));
  vi.stubGlobal("fetch", fetcher);
  vi.stubGlobal("crypto", webcrypto);
  return fetcher;
}
afterEach(() => vi.unstubAllGlobals());
it("verifies and summarizes the published snapshot, including Dutch global printings", async () => {
  const fetcher = mockSnapshot();
  const signal = new AbortController().signal;
  const data = await loadCoverage(signal);
  expect(data.sourceCommit).toBe(revision);
  expect(data.rows[0]).toMatchObject({
    name: "Base Set",
    language: "nl",
    region: "global",
    cards: 1,
    finishes: 1,
  });
  expect(fetcher).toHaveBeenNthCalledWith(1, `${catalogueDataUrl}manifest.json`, { signal });
  expect(fetcher).toHaveBeenNthCalledWith(
    2,
    new URL(fixture().manifest.filename, catalogueDataUrl),
    { signal },
  );
});
it("reports manifest HTTP failures", async () => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(null, { status: 503 })));
  await expect(loadCoverage(new AbortController().signal)).rejects.toThrow("manifest (HTTP 503)");
});
it.each([{ filename: "../../secret" }, { sourceCommit: "invalid" }])(
  "rejects an invalid manifest %j",
  async (override) => {
    const fetcher = mockSnapshot(override);
    await expect(loadCoverage(new AbortController().signal)).rejects.toThrow(
      "Invalid catalogue manifest",
    );
    expect(fetcher).toHaveBeenCalledTimes(1);
  },
);
it("reports snapshot HTTP failures", async () => {
  const fetcher = mockSnapshot();
  fetcher
    .mockReset()
    .mockResolvedValueOnce(new Response(JSON.stringify(fixture().manifest)))
    .mockResolvedValueOnce(new Response(null, { status: 404 }));
  await expect(loadCoverage(new AbortController().signal)).rejects.toThrow("snapshot (HTTP 404)");
});
it("rejects modified snapshots", async () => {
  mockSnapshot({}, "tampered");
  await expect(loadCoverage(new AbortController().signal)).rejects.toThrow("checksum mismatch");
});
it("rejects mismatched revisions", async () => {
  mockSnapshot({ sourceCommit: "b".repeat(40) });
  await expect(loadCoverage(new AbortController().signal)).rejects.toThrow(
    "source revision mismatch",
  );
});
it("rejects mismatched printing totals", async () => {
  mockSnapshot({ counts: { printing: 2 } });
  await expect(loadCoverage(new AbortController().signal)).rejects.toThrow(
    "printing count mismatch",
  );
});

it.each([
  { records: [{ id: "invalid", metadata: { kind: 42 } }] },
  { records: "not an array" },
  { records: [{ id: "invalid", metadata: { kind: "printing", attributes: [] } }] },
])("rejects a checksum-valid snapshot with invalid record data: %j", async (invalid) => {
  const body = JSON.stringify({ sourceCommit: revision, ...invalid });
  const sha256 = createHash("sha256").update(body).digest("hex");
  mockSnapshot({ sha256, filename: `catalogue-${sha256}.json` }, body);
  await expect(loadCoverage(new AbortController().signal)).rejects.toThrow();
});
