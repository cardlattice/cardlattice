import { summarize, type RecordData } from "./coverage-model.ts";
import type { CoverageData } from "./coverage-query.ts";

export const catalogueDataUrl = "https://cardlattice-status.pages.dev/data/";
export async function loadCoverage(signal: AbortSignal): Promise<CoverageData> {
  const manifestResponse = await fetch(`${catalogueDataUrl}manifest.json`, { signal });
  if (!manifestResponse.ok)
    throw new Error(`Could not load catalogue manifest (HTTP ${manifestResponse.status})`);
  const manifest = await manifestResponse.json();
  if (
    !/^catalogue-[a-f0-9]{64}\.json$/.test(manifest.filename) ||
    !/^[a-f0-9]{40}$/.test(manifest.sourceCommit)
  )
    throw new Error("Invalid catalogue manifest");
  const response = await fetch(new URL(manifest.filename, catalogueDataUrl), { signal });
  if (!response.ok) throw new Error(`Could not load catalogue snapshot (HTTP ${response.status})`);
  const raw = await response.arrayBuffer();
  const digest = await crypto.subtle.digest("SHA-256", raw);
  const checksum = Array.from(new Uint8Array(digest), (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");
  if (checksum !== manifest.sha256) throw new Error("Catalogue snapshot checksum mismatch");
  const snapshot: { sourceCommit: string; records: RecordData[] } = JSON.parse(
    new TextDecoder().decode(raw),
  );
  if (snapshot.sourceCommit !== manifest.sourceCommit)
    throw new Error("Catalogue source revision mismatch");
  const rows = summarize(snapshot.records);
  if (rows.reduce((total, row) => total + row.cards, 0) !== manifest.counts.printing)
    throw new Error("Catalogue printing count mismatch");
  return { rows, sourceCommit: snapshot.sourceCommit, snapshotSha256: checksum };
}
