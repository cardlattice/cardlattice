import { z } from "zod";
export const recordSchema = z.object({
  id: z.string(),
  metadata: z.object({
    kind: z.string(),
    name: z.string().optional(),
    setId: z.string().optional(),
    gameId: z.string().optional(),
    seriesId: z.string().optional(),
    language: z.string().optional(),
    region: z.string().optional(),
    rarity: z.string().optional(),
    attributes: z.record(z.string(), z.unknown()).optional(),
  }),
});
export type RecordData = z.infer<typeof recordSchema>;
export interface Coverage {
  key: string;
  name: string;
  game: string;
  gameId: string;
  series: string;
  language: string;
  region: string;
  setId: string;
  cards: number;
  finishes: number;
  illustrators: number;
  rarities: number;
}
const nonempty = (value: unknown) => typeof value === "string" && value.trim().length > 0;
const list = (value: unknown) => Array.isArray(value) && value.some(nonempty);
export function hasFinish(attributes: Record<string, unknown>) {
  if (list(attributes.finishes)) {
    return true;
  }
  const variants = attributes.variants;
  return (
    !!variants &&
    typeof variants === "object" &&
    Object.values(variants).some(
      (variant: unknown) =>
        !!variant && typeof variant === "object" && "finish" in variant && nonempty(variant.finish),
    )
  );
}
export function summarize(records: RecordData[]) {
  const ids = new Set<string>();
  for (const record of records) {
    if (ids.has(record.id)) {
      throw new Error(`Duplicate record: ${record.id}`);
    }
    ids.add(record.id);
  }
  const sets = new Map(
    records.filter((r) => r.metadata.kind === "set").map((r) => [r.id, r.metadata]),
  );
  const names = new Map(records.map((r) => [r.id, r.metadata.name ?? r.id]));
  const rows = new Map<string, Coverage>();
  function row(setId: string, language: string, region: string) {
    const key = JSON.stringify([setId, language, region]);
    const existing = rows.get(key);
    if (existing) {
      return existing;
    }
    const set = sets.get(setId);
    if (!set) {
      throw new Error(`Unknown printing set: ${setId}`);
    }
    const created = {
      key,
      setId,
      language,
      region,
      name: set.name ?? setId,
      game: names.get(set.gameId ?? "") ?? "Unknown game",
      gameId: set.gameId ?? "unknown",
      series: names.get(set.seriesId ?? "") ?? "Unassigned series",
      cards: 0,
      finishes: 0,
      illustrators: 0,
      rarities: 0,
    };
    rows.set(key, created);
    return created;
  }
  for (const { metadata: m } of records) {
    if (m.kind !== "printing") {
      continue;
    }
    const target = row(m.setId ?? "", m.language ?? "und", m.region ?? "global");
    const attrs = m.attributes ?? {};
    target.cards++;
    target.finishes += Number(hasFinish(attrs));
    target.illustrators += Number(list(attrs.illustrators));
    target.rarities += Number(nonempty(m.rarity));
  }
  const represented = new Set([...rows.values()].map((r) => r.setId));
  for (const [id, set] of sets) {
    if (!represented.has(id)) {
      row(id, set.language ?? "und", set.region ?? "global");
    }
  }
  return [...rows.values()].sort(
    (a, b) => a.name.localeCompare(b.name, "en") || a.key.localeCompare(b.key, "en"),
  );
}
export function percent(count: number, total: number) {
  return total ? Math.round((count / total) * 100) : 0;
}
