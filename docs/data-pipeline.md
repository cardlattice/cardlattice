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

## Build in CI

Data PRs validate and compile with the exact application commit pinned by
`builder-ref.txt`. The offline compiler uses the same domain validation as the
API. It rejects synthetic data, invalid relationships, duplicate identities,
invalid source licences and oversized inputs. It sorts records and produces
byte-identical output for the same semantic input and source commit.

The artifact contains `manifest.json` and `catalogue-<sha256>.json`. Manifest
counts are measured; its source commit, size and checksum identify the input and
output. All catalogue keys and provenance are retained. Daily GitHub releases
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
records, source provenance, audit revisions and one deduplicated invalidation
event. Identical records create no revisions. Missing previously managed records
are retired children-first. Other locally created records are not deleted.
Changes made locally after the last index cause a conflict rather than silently
being overwritten. Reconcile those changes in a data PR before retrying.

The first artifact can adopt an identical existing semantic catalogue. It rejects
conflicting existing records. This is not a UUID migration: old databases are not
supported. The first data repository contains only reviewed real facts; old demo
records and snapshots are not published into it.

The initial bundle covers the common core and external identifiers. Supplementary
localisations, finishes and reviewed image references will need an explicit bundle
schema extension before those tables can be rebuilt from Git. They are currently
unpopulated in the reviewed catalogue. Private submissions, scans, moderation
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
