import { webcrypto } from "node:crypto";
import { afterEach, describe, expect, it, vi } from "vitest";
import { loadSnapshot, manifestUrl, selectCatalogue, type CatalogueRecord } from "./catalogue-data";
const game: CatalogueRecord = {
  id: "pokemon",
  kind: "game",
  name: "Pokémon",
  slug: "pokemon",
  attributes: {},
};
const records: CatalogueRecord[] = [
  game,
  { id: "set", kind: "set", gameId: "pokemon", name: "Original test set", attributes: {} },
  {
    id: "card",
    kind: "card",
    gameId: "pokemon",
    name: "Café Seed",
    language: "nl",
    attributes: {},
  },
  {
    id: "printing-1",
    kind: "printing",
    gameId: "pokemon",
    cardId: "card",
    setId: "set",
    name: "Café Seed",
    language: "nl",
    attributes: { variants: { special: { products: ["product"] } } },
  },
  {
    id: "printing-2",
    kind: "printing",
    gameId: "pokemon",
    cardId: "card",
    setId: "other-set",
    name: "Café Seed",
    language: "en",
    attributes: {},
  },
];
afterEach(() => vi.unstubAllGlobals());
async function mockSnapshot(tamper = false) {
  const sourceCommit = "a".repeat(40);
  const raw = JSON.stringify({
    schemaVersion: 1,
    sourceCommit,
    records: records.map(({ id, ...metadata }) => ({ id, metadata })),
  });
  const hash = await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(raw));
  const sha256 = Buffer.from(hash).toString("hex");
  const manifest = {
    schemaVersion: 1,
    sourceCommit,
    filename: `catalogue-${sha256}.json`,
    sha256,
    byteSize: new TextEncoder().encode(raw).length,
    counts: { game: 1, set: 1, card: 1, printing: 2 },
  };
  const fetch = vi
    .fn()
    .mockResolvedValueOnce(new Response(JSON.stringify(manifest)))
    .mockResolvedValueOnce(new Response(tamper ? raw.replace("Café Seed", "Café Weed") : raw));
  vi.stubGlobal("fetch", fetch);
  vi.stubGlobal("crypto", webcrypto);
  return fetch;
}
describe("static catalogue", () => {
  it("loads and verifies a snapshot using only static data URLs", async () => {
    const fetch = await mockSnapshot();
    const snapshot = await loadSnapshot();
    expect(snapshot.records).toEqual(records);
    expect(fetch.mock.calls[0][0]).toBe(manifestUrl);
    expect(fetch.mock.calls[1][0]).toMatch(/\/data\/catalogue-[a-f0-9]{64}\.json$/);
    expect(fetch).toHaveBeenCalledTimes(2);
  });
  it("rejects changed snapshot bytes", async () => {
    await mockSnapshot(true);
    await expect(loadSnapshot()).rejects.toThrow(/size|checksum/);
  });
  it("reports an unavailable manifest without inventing fallback data", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response("", { status: 503 })));
    await expect(loadSnapshot()).rejects.toThrow("503");
  });
  it("rejects path traversal in the manifest", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(new Response(JSON.stringify({ filename: "../secret" }))),
    );
    await expect(loadSnapshot()).rejects.toThrow();
  });
  it("searches localised names and filters languages", () => {
    expect(selectCatalogue(records, "/search?q=caf%C3%A9&language=nl")).toMatchObject({
      data: [{ id: "card" }, { id: "printing-1" }],
    });
    expect(selectCatalogue(records, "/games/pokemon/printings?setId=set")).toMatchObject({
      data: [{ id: "printing-1" }],
    });
    expect(selectCatalogue(records, "/games/pokemon/printings?productId=product")).toMatchObject({
      data: [{ id: "printing-1" }],
    });
  });
  it("keeps details and pagination scoped to the requested game", () => {
    expect(selectCatalogue(records, "/games/pokemon/cards/card")).toMatchObject({
      data: { id: "card" },
      printings: [{ id: "printing-1" }, { id: "printing-2" }],
    });
    expect(() => selectCatalogue(records, "/games/other/cards/card")).toThrow(
      "not in the published catalogue",
    );
    expect(selectCatalogue(records, "/games/pokemon/printings?limit=1")).toMatchObject({
      data: [{ id: "printing-1" }],
      nextCursor: "1",
    });
    expect(selectCatalogue(records, "/games/pokemon/printings?limit=1&cursor=1")).toMatchObject({
      data: [{ id: "printing-2" }],
      nextCursor: null,
    });
    expect(() => selectCatalogue(records, "/search?cursor=NaN")).toThrow("Invalid catalogue page");
  });
});
