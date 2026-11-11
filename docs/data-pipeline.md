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

Set `defaults` supply shared printing details (finishes, stamps, regulation mark,
printed denominator and illustrators). Explicit card fields replace defaults;
`stamps: []` clears an inherited stamp. The compiler materialises these fields in
printing `attributes`, which the existing API and indexer preserve. No new SQL
migration is needed. This does not populate the separate printing-variant table
or create finish/stamp cross-products. Product membership, variant-specific images
and supplementary localisations still need a separate data model. Private submissions, scans, moderation
credentials and audit identities are never exported into the data repository.

The deployment trigger and R2 publishing credentials are operator configuration;
repository CI does not deploy to the running local catalogue. The existing
contribution UI remains available for drafts. Turning an approved draft into a
GitHub PR is not automated yet.

Indexing also stores the exact verified CI artifact in `DATASET_DIRECTORY` and
registers it for `/v1/datasets` and downloads, at most once per UTC day. It does not
regenerate the bytes. Registration can be retried safely after an interruption;
record indexing is already committed and remains idempotent. The old daily export
library remains covered by tests, but the running API no longer schedules it.
