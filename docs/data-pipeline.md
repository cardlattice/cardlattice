# Catalogue repository and CI

`cardlattice/cardlattice-data` is authoritative for reviewed catalogue records.
Application source remains in `cardlattice/cardlattice`. Original development
fixtures remain in the application for disposable tests. Images belong in separate
object storage; the data dedication does not license artwork.

The data repository uses semantic text keys: game slug, set identifier, language,
region and collector number. For example `pokemon-mega-evolution-en-US-001` is a
printing and `card-pokemon-mega-evolution-en-US-001` is its provisional identity.
Set names currently supply the readable set identifier; shorter verified set codes
can be adopted through reviewed data changes. Never merge cards solely by name.
Variants require distinct keys. There are no UUID aliases or compatibility routes.
The schema is intentionally breaking for this unreleased project: use a fresh
PostgreSQL database and rebuild it from the reviewed artifact, not old migrations.

## Editing data

Contributors edit one YAML file per printing:
`catalogue/pokemon/cards/mega-evolution/en/US/001.yaml`. Most files contain only
`name`; the path supplies identity and language/region context. Separate set
files supply set metadata, and optional `release.yaml` supplies release overrides
and external identifier patterns. Provisional card identities are generated
automatically. Reviewed reprints can reference an explicit shared identity.

No source-evidence files or per-record citations are required or included in the
data tree or compiled artifact. Git history and the manifest commit track changes.
The API accepts drafts without evidence; optional existing provenance support is
independent of the data-repository workflow. No executable contributor code is
loaded. YAML comments are allowed, but duplicate keys, aliases, anchors, merges,
unsupported tags, unknown fields and symlinks are rejected. Local validation is
`cardlattice-api validate-catalogue <catalogue-directory>` without a database.

## Build in CI

Data PR schema validation checks only changed card/set YAML files. Main-branch
validation checks the whole tree. PRs still compile the whole preview artifact,
including reference validation, so deletion cannot leave a broken catalogue.

Data PRs compile with the exact application commit pinned by
`builder-ref.txt`. The offline compiler uses the same domain validation as the
API. It rejects synthetic data, invalid relationships, duplicate identities,
invalid typed attributes and oversized inputs. It sorts records and produces
byte-identical output for the same semantic input and source commit.

The artifact contains `manifest.json` and `catalogue-<sha256>.json`. Manifest
counts are measured; its source commit, size and checksum identify the input and
output. Catalogue keys are retained. Daily GitHub releases
publish from reviewed `main`; PR artifacts are previews only. A deployment may
mirror the exact bytes to R2 without rebuilding the catalogue.

## Index a trusted artifact

Download both files from an approved successful data release. Never blindly
consume an arbitrary PR artifact or an untrusted URL. A checksum is an integrity
check, not a signature or approval. No database credentials belong in data CI.

```sh
cargo run -p cardlattice-api -- migrate
cargo run -p cardlattice-api -- index-catalogue /path/to/manifest.json
cargo run -p cardlattice-api -- index-catalogue /path/to/manifest.json --apply
```

The default is a full transactional dry run. `--apply` atomically writes changed
records, audit revisions and one deduplicated invalidation
event. Identical records create no revisions. Missing previously managed records
are retired children-first. Other locally created records are not deleted.
Changes made locally after the last index cause a conflict rather than silently
being overwritten. Reconcile those changes in a data PR before retrying.

The first artifact can adopt an identical existing semantic catalogue. It rejects
conflicting existing records. This is not a UUID migration: old databases are not
supported. The first data repository contains only reviewed real facts; old demo
records and snapshots are not published into it.

The bundle covers core records, external identifiers, typed Pokémon gameplay,
and printing finishes/stamps. Card files accept ordered attacks (including free
attacks and repeated energy costs), abilities, evolution, weakness/resistance,
retreat cost, rules, subtypes and optional Trainer affiliations. Damage strings
preserve modifiers such as `20×`. Missing values remain unknown; explicit empty
lists mean none.

Set-wide printing details are top-level YAML fields. The precedence is card >
release > set. Missing fields inherit, null clears a scalar, and [] clears a list.
The compiler preserves presence in printing JSON attributes; it does not fill
unknown lists with empty arrays. `printedDenominator` is the printed collector
number denominator, never a measured catalogue count. Regional dates belong in
release.yaml. Unsupported authoring aliases (defaults, attributes,
printingAttributes and printedTotal) are rejected. No compatibility aliases exist.

The deployment trigger and R2 publishing credentials are operator configuration;
repository CI does not deploy to the running local catalogue. The existing
contribution UI remains available for drafts. Turning an approved draft into a
GitHub PR is not automated yet.

Indexing also stores the exact verified CI artifact in `DATASET_DIRECTORY` and
registers it for `/v1/datasets` and downloads, at most once per UTC day. It does not
regenerate the bytes. Registration can be retried safely after an interruption;
record indexing is already committed and remains idempotent. The old daily export
library remains covered by tests, but the running API no longer schedules it.

Pokémon category, energy type, stage, subtype, rarity, finish, ability kind and
regulation mark use case-sensitive vocabularies. The Rust enums in
`crates/catalogue/src/pokemon/vocabulary.rs` generate parsing and OpenAPI enum
schemas; `pokemon-vocabulary` exports the exact spellings. The data build compares
its schema enums with this export from its pinned compiler without scanning any
card files. A vocabulary extension requires updating both repositories and the
compiler pin. There are no permissive aliases or `Other` fallbacks. Optional
unknown fields stay omitted; supported vocabularies do not claim historical or
regional completeness.

Pokémon rarity is validated through its enum at the game-specific draft boundary;
the multi-game record's rarity field remains a string so other games can use their
own taxonomy. Contribution submissions, YAML imports (including shared identities)
and compiled bundle validation all use the same domain checks. Product stamp IDs,
trainer affiliations, names and prose remain open-ended text.

## Printing relationships

Product, collection and artwork files are first-class records compiled with stable
`<game>-product-<key>`, `<game>-collection-<key>` and `<game>-artwork-<key>` IDs.
A printing's `products`, `collections` and `artwork` references are independent of
its canonical set, collector number and shared rules identity. Product/collection
`relatedSets` supplies context without implicitly including every set member.
Artwork contains descriptive metadata, not image assets or publication rights.

Set `printedIdentifiers` lists language/region-scoped printed codes and/or symbol
descriptions. Codes are searchable; names and canonical IDs remain independent.
Symbol images and membership of particular finish/stamp combinations are outside
this change. Attacks remain inline: neither name nor era guarantees equal rules.

Migration 0007 adds product/collection/artwork record kinds and `catalogue_links`.
Database triggers maintain indexed links from typed JSON attributes. Deferred
integrity checks reject missing, retired, wrong-kind or cross-game targets while
allowing an authoritative transaction to update links and retire records together.
The CI compiler validates the complete graph before producing a bundle. Schema
checks in data PRs still inspect only changed YAML files, including new definitions.

The public API exposes `/v1/games/{game}/products`, `/collections`, `/artworks`
and their `/{id}` details using the existing pagination and caching contracts.
Printing lists accept `productId`, `collectionId` and `artworkId` filters, which
are included in cursor scope. Details include up to 20 related printings and a
paginated URL. Link changes purge old and new target details; arbitrary filtered
collections and search remain covered by their bounded TTLs. The web browser can
open these records and submit corrections through the existing moderation flow.

Run `cargo test -p cardlattice-api --test relationships` with a disposable
PostgreSQL database to verify the graph, indexing, queries, retirement and outbox.

## Schema audit format

All authored record kinds have JSON Schemas in the data repository, sharing a
common vocabulary. PR schema checks read changed files; compilation checks the
complete relationship graph. Language tags use the canonical subset language,
optional script and optional region (en, en-US, zh-Hant, es-419). Regions are
uppercase two-letter codes; the directory token global compiles to no region.

Reviewed identity files contain localisations keyed by language, each with name
and typed pokemon fields. Printings resolve to an explicitly supplied matching
localisation; there is no fallback translation. Compiled card records link via
identityGroup and include the language in duplicate detection. Generated identity
metadata is not accepted in authored YAML.

Optional named variants describe actual finish/edition/stamp/product combinations.
Each map entry requires finish, optionally edition (first-edition or unlimited),
stamps and products. Normal cards still use a finishes array. Set/release variant
inheritance replaces the whole map; null clears it. If finishes is also present,
it must agree with the variants. Product filters include nested variant products.
Migration 0008 handles explicit null artwork and indexes set and variant links.

Historical Trainer HP does not require a retroactive Item subtype. Printed damage
can be ?, and weakness/resistance may contain only a type where no modifier is
printed. Category/subtype compatibility is validated alongside canonical enums.

Deploy the application/compiler PR before merging the coordinated data PR, which
pins its compiler commit in builder-ref.txt. Old authored aliases are deliberately
unsupported. Rebuild/reindex the data artifact; do not edit immutable old snapshots.
