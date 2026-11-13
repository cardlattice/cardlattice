import { z } from "zod";

export const manifestSchema = z.object({
  schemaVersion: z.literal(1),
  sourceCommit: z.string().regex(/^[a-f0-9]{40}$/),
  filename: z.string().regex(/^catalogue-[a-f0-9]{64}\.json$/),
  sha256: z.string().regex(/^[a-f0-9]{64}$/),
  byteSize: z
    .number()
    .int()
    .positive()
    .max(25 * 1024 * 1024),
  counts: z.record(z.string(), z.number().int().nonnegative()),
});
export type Manifest = z.infer<typeof manifestSchema>;
const metadataSchema = z.object({
  kind: z.enum(["game", "series", "set", "card", "printing", "product", "collection", "artwork"]),
  name: z.string(),
  gameId: z.string().nullish(),
  seriesId: z.string().nullish(),
  setId: z.string().nullish(),
  cardId: z.string().nullish(),
  slug: z.string().nullish(),
  collectorNumber: z.string().nullish(),
  language: z.string().nullish(),
  region: z.string().nullish(),
  releaseDate: z.string().nullish(),
  rarity: z.string().nullish(),
  attributes: z.record(z.string(), z.unknown()).default({}),
  synthetic: z.literal(false).optional(),
});
const bundleSchema = z.object({
  schemaVersion: z.literal(1),
  sourceCommit: z.string(),
  records: z.array(z.object({ id: z.string().min(1), metadata: metadataSchema })),
});
export type CatalogueRecord = z.infer<typeof metadataSchema> & { id: string };
export type Snapshot = { manifest: Manifest; records: CatalogueRecord[] };
export const catalogueBase = (
  import.meta.env.VITE_CATALOGUE_URL || "https://status.cardlattice.com/data/"
).replace(/\/?$/, "/");
export const manifestUrl = `${catalogueBase}manifest.json`;

export async function loadSnapshot(signal?: AbortSignal): Promise<Snapshot> {
  const response = await fetch(manifestUrl, { signal, cache: "no-cache" });
  if (!response.ok) throw new Error(`Catalogue manifest unavailable (${response.status}).`);
  const manifest = manifestSchema.parse(await response.json());
  if (manifest.filename !== `catalogue-${manifest.sha256}.json`)
    throw new Error("Invalid catalogue filename.");
  const data = await fetch(`${catalogueBase}${manifest.filename}`, { signal });
  if (!data.ok) throw new Error(`Catalogue download unavailable (${data.status}).`);
  const bytes = await data.arrayBuffer();
  if (bytes.byteLength !== manifest.byteSize)
    throw new Error("Catalogue size does not match the manifest.");
  const hash = await crypto.subtle.digest("SHA-256", bytes);
  const checksum = Array.from(new Uint8Array(hash), (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");
  if (checksum !== manifest.sha256)
    throw new Error("Catalogue checksum does not match the manifest.");
  const bundle = bundleSchema.parse(JSON.parse(new TextDecoder().decode(bytes)));
  if (bundle.sourceCommit !== manifest.sourceCommit)
    throw new Error("Catalogue revision does not match the manifest.");
  const records = bundle.records.map(({ id, metadata }) => ({ ...metadata, id }));
  if (new Set(records.map((r) => r.id)).size !== records.length)
    throw new Error("Duplicate catalogue identifiers.");
  const counts: Record<string, number> = {};
  for (const record of records) counts[record.kind] = (counts[record.kind] || 0) + 1;
  if (
    Object.keys({ ...counts, ...manifest.counts }).some(
      (kind) => (counts[kind] || 0) !== (manifest.counts[kind] || 0),
    )
  )
    throw new Error("Catalogue counts do not match the manifest.");
  return { manifest, records };
}

function related(record: CatalogueRecord, key: string, id: string) {
  if (key === "setId" || key === "cardId") return record[key] === id;
  const attr = record.attributes;
  if (key === "artworkId") return attr.artwork === id;
  const field = key === "productId" ? "products" : "collections";
  if (Array.isArray(attr[field]) && attr[field].includes(id)) return true;
  return (
    key === "productId" &&
    !!attr.variants &&
    typeof attr.variants === "object" &&
    Object.values(attr.variants).some(
      (value) =>
        !!value &&
        typeof value === "object" &&
        "products" in value &&
        Array.isArray(value.products) &&
        value.products.includes(id),
    )
  );
}
const normalize = (value: string) => value.normalize("NFKC").toLocaleLowerCase("en");
export function selectCatalogue(records: CatalogueRecord[], path: string) {
  const url = new URL(path, "https://cardlattice.com");
  if (url.pathname === "/stats") {
    const count = (kind: string) => records.filter((r) => r.kind === kind).length;
    return {
      games: count("game"),
      series: count("series"),
      sets: count("set"),
      cards: count("card"),
      printings: count("printing"),
      missingRarity: records.filter((r) => r.kind === "printing" && !r.rarity).length,
      printingsWithImages: 0,
      syntheticRecords: 0,
      completenessNote:
        "Known records do not establish complete coverage of every card or variant.",
    };
  }
  const parts = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  const kinds: Record<string, string> = {
    games: "game",
    series: "series",
    sets: "set",
    cards: "card",
    printings: "printing",
    products: "product",
    collections: "collection",
    artworks: "artwork",
  };
  const search = parts[0] === "search";
  if (!search && parts[0] !== "games") throw new Error("Unknown catalogue page.");
  const game = parts[1];
  const kind = kinds[parts[2] || "games"];
  const id = parts[3] || (parts.length === 2 ? game : undefined);
  if (!search && !kind) throw new Error("Unknown catalogue record type.");
  if (id) {
    const record = records.find(
      (r) =>
        r.kind === kind &&
        (kind === "game" ? r.id === id || r.slug === id : r.id === id && r.gameId === game),
    );
    if (!record) throw new Error("This record is not in the published catalogue.");
    return {
      data: record,
      printings: records
        .filter(
          (r) =>
            r.kind === "printing" && r.gameId === game && related(r, `${record.kind}Id`, record.id),
        )
        .slice(0, 20),
      images: [],
      sources: [],
      localisations: [],
      variants: [],
    };
  }
  const params = url.searchParams;
  const needle = normalize(params.get("q") || "");
  const matches = records.filter(
    (r) =>
      (search || r.kind === kind) &&
      (!game || r.gameId === game) &&
      (!params.get("language") || r.language === params.get("language")) &&
      ["setId", "cardId", "productId", "collectionId", "artworkId"].every(
        (key) => !params.get(key) || related(r, key, params.get(key)!),
      ) &&
      (!needle ||
        normalize(
          `${r.name} ${r.id} ${r.collectorNumber || ""} ${JSON.stringify(r.attributes)}`,
        ).includes(needle)),
  );
  matches.sort(
    (a, b) =>
      (params.get("sort") === "newest"
        ? (b.releaseDate || "").localeCompare(a.releaseDate || "")
        : 0) ||
      a.name.localeCompare(b.name, "en", { numeric: true }) ||
      a.id.localeCompare(b.id),
  );
  const cursor = params.get("cursor") || "0";
  const requestedLimit = params.get("limit") || "50";
  if (!/^\d+$/.test(cursor) || !/^\d+$/.test(requestedLimit))
    throw new Error("Invalid catalogue page.");
  const offset = Number(cursor),
    limit = Math.min(100, Math.max(1, Number(requestedLimit)));
  if (!Number.isSafeInteger(offset)) throw new Error("Invalid catalogue page.");
  return {
    data: matches.slice(offset, offset + limit),
    limit,
    nextCursor: offset + limit < matches.length ? String(offset + limit) : null,
  };
}
