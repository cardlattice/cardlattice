# Delivery status and roadmap

The first release implements a working end-to-end local application. It is not a
claim that every production requirement or every game rule is complete.

| Phase | Implemented | Remaining |
| --- | --- | --- |
| 1 | Cargo/pnpm workspace, PostgreSQL migrations, Axum, React/Vite/Tailwind, Compose, CI | Public remote/PR and operator-selected production host |
| 2 | Anonymous read routes, generated OpenAPI, generated TS client/hooks, reviewed create/update/delete for game/series/set/card/printing | Per-person contributor identities; richer editing for auxiliary records |
| 3 | Bounded keyset pagination, filters, Unicode normalisation, full-text/trigram search, translation fallback, Pokémon core attributes | Dedicated localisation/variant/identifier contribution forms, broader game adapters, large relationship selectors |
| 4 | Conditional GET/HEAD, explicit Cache Rules, atomic outbox, retry/replay, immutable streaming snapshots and manifests | Live-zone verification, R2 adapter, shared-storage multi-replica operation, job coalescing, full supplementary-data export |
| 5 | Home, catalogue, details, search, interactive docs, datasets, coverage, persisted status, changelog, policy/governance pages; light/dark responsive UI | Extended cross-browser and assistive-technology audit; project-specific contact/remote links |
| 6 | Capability-protected drafts, duplicate detection, review queue and decisions, private front/back scans, manual OCR adapter seam | Real optional OCR adapter, field-confidence review UI, scan-first draft creation, image-rights publication UI, per-person revocation |
| 7 | Real PostgreSQL tests, contract/ETag/permissions tests, browser workflow/a11y tests, opt-in measured benchmarks, audits, operations/backup/cost/deployment docs | Independent security review, real-zone purge exercise, disaster-recovery rehearsal, production capacity tests |

## Known first-release boundaries

No catalogue completeness, uptime, CDN hit ratio or throughput is advertised without
measurement. Demo fixtures are opt-in. A bounded official Pokémon press-site scanner
and reviewed importer provide an initial eight real sets, with factual names and
announced US release dates. There are no real card or printing records yet, and no
claim of complete set coverage. The manual scan fallback is
operational; OCR itself is not implemented. Snapshot exports contain public core
records; auxiliary localisation, variant, source and image-rights records remain in
PostgreSQL and detail responses. The source acceptance policy limits exported
metadata to CC0-compatible original contributions.

Correction links retain their game scope. Related-record selectors list up to 100 entries. Role tokens
are a minimal abuse-resistant deployment mechanism, not a multi-user account system.
Scan originals are stored privately in PostgreSQL, with no automated retention job.

SQLx uses runtime parameterised queries instead of compile-time/offline metadata.
No auth library or object-store daemon is added solely for scaffolding. Turborepo
is unnecessary for one frontend package; Cargo and pnpm provide the workspace tasks.
No remote is created without an operator-selected destination, so PR attachment and
remote CI execution are separate from the local validation record.
