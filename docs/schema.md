# Database schema

Migrations are ordered, versioned SQL under `migrations/`. Apply using
`cardlattice-api migrate`; production upgrades must be a dedicated release step.
Migrations do not seed data. UUIDs are internally assigned; upstream IDs can only
appear in the external identifier relation.

| Relation | Meaning and constraints |
| --- | --- |
| `catalogue_records` | UUID identity, constrained kind, game, name, canonical search name, optional parent series/set/card, printing metadata, bounded application attributes, synthetic flag, duplicate fingerprint, timestamps and tombstone |
| `games`, `series`, `sets`, `card_identities`, `printings` | Typed views of active records; not duplicate data stores |
| `printing_variants` | Printing, finish, stamp and game attributes; unique combination |
| `localisations` | Record + BCP 47 tag, name, original text, normalised name and search vector |
| `images` | Public image URL and printing; mandatory reviewed rights FK |
| `image_rights` | Permission evidence, licence, attribution, reviewer and timestamp |
| `external_identifiers` | Namespace/value mappings; never canonical identity |
| `sources` | Original contribution, physical inspection, or permitted reference; description, URL, licence |
| `provenance` | Record, source and JSON field path; each accepted draft field has a source link |
| `contributions` | Capability hash, idempotency UUID and request digest, operation, base timestamp, draft JSON, status |
| `review_decisions` | Append-only application audit of reviewer hash, decision and reason |
| `scan_uploads` | Original private bytes, side, media type, digest, candidate metadata, private rights state |
| `catalogue_revisions` | Monotonic revision, before/after data and contribution reference |
| `cache_invalidation_outbox` | Revision, affected URL paths, attempts, next retry, errors, completion |
| `publication_manifests` | Unique revision and immutable filename, schema version, SHA-256, bytes and count |
| `health_measurements` | Stored probe outcome and observed database latency; seven-day retention |
| `rate_limits` | Durable minute buckets; expired buckets removed hourly |

A database trigger enforces parent types and same-game relationships. Record kind,
game, and game slug cannot change in place. Tombstoning a parent with active
dependents fails. The write path serialises catalogue mutations with an advisory
lock; production application roles should not permit arbitrary direct writes.

Indexes cover game/kind/canonical-name/UUID pages, created time, printing card/set,
rarity, pending review/outbox queues, health timestamps, full-text GIN, and trigram
GIN searches. Public list cursors retain canonical sort key and UUID, and bind to a
hash of the request's path and filters. Cursor data is not a secret or authority.

Supplementary tables have foundational foreign keys; image/variant references also
have database-enforced printing kinds. Their future write interfaces must route changes through the same revision
and outbox transaction. Direct SQL imports bypass that workflow and are not a
supported public contribution mechanism.

SQLx queries are parameterised and decoded into typed structs. This release uses
runtime queries, including QueryBuilder for dynamic filters, rather than checked-in
SQLx offline macro metadata. Real PostgreSQL integration tests exercise the schema;
compile-time query metadata remains a documented project-standards deviation.
