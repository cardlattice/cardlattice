# Contributing

Start with `docker compose up --build -d`. The complete contributor/moderator
walkthrough is in the README. Demo seeding is opt-in; use a disposable database for
fixture-based browser tests. All example data must be original and visibly
synthetic. Never add card artwork, catalogue dumps, or third-party API responses
to tests, screenshots, documentation examples, or seeds.

Real official-source imports follow the bounded discovery and review process in
[`data/pokemon/README.md`](../data/pokemon/README.md). Only reviewed factual set
metadata is imported; the scan never downloads publisher images or third-party
catalogue data. The command defaults to a rollback-only preview.

## Metadata

Inspect your own card or an explicitly permitted official reference. Record the
source and distinguish card identity from printing information. Provide a language
tag and collector number on every printing. Unknown fields should remain unknown;
do not invent missing metadata. Explain corrections and submit them from the record
detail page so the base revision is retained.

Duplicate detection uses a canonical fingerprint plus pending and published unique
indexes. It compares normalised names plus game attributes for card identities, and set/release/collector/language/region for printings; it is not visual card recognition. Same-name cards with different attributes remain separate identities. Moderators must resolve ambiguous identity and incomplete metadata.
Submission replay is safe with the same Idempotency-Key and identical body.

## Scans and OCR adapters

Upload originals only after creating a draft. Front and back are separate files;
each contribution accepts one of each. Files are decoded, bounded, hashed and stored
privately. Approval never moves them to the public image table. The manual pipeline
returns empty candidate fields with `requiresHumanReview: true`.

An optional OCR adapter implements `ingestion::ScanExtractor` and returns candidate
fields, engine identity, and a human-review requirement. It must not publish records
or contact a paid service by default. Future adapters should record field confidence,
set/collector/language/rarity candidates, and source bytes' digest. No scanner or
OCR service is needed to run the application. This release does not perform OCR
automatically or create a draft from an unassociated scan: create the manual draft
first, attach evidence, then review.

## Code

Use a `luna/*` branch, Conventional Commits, and the PR template. Rust forbids unsafe
code by default. Prefer small domain changes and parameterised SQL. Any catalogue
write must preserve the revision/provenance/outbox transaction. Add a new forward
migration instead of editing a migration deployed elsewhere.

Run `cargo fmt --check`, Clippy with denied warnings, PostgreSQL integration tests,
`pnpm check`, and relevant Playwright tests. Regenerate OpenAPI and the Orval client
when contracts change. Generated files must not be hand-edited. UI work requires
keyboard checks, accessible labels/states, both themes, responsive layouts, and
before/after evidence in a PR where a predecessor exists.

`pnpm check` runs Oxfmt, Oxlint, TypeScript, unit tests, and the production build.
Use `pnpm format` to fix formatting, or `pnpm format:check` to check it without
writing files. Both Oxc tools cover the whole frontend package, including tests,
Vite/Playwright configuration and the public theme script. Generator-owned code
and build/test artifacts are excluded; regenerate the API client instead of
editing it manually.

Oxlint enables every built-in `jsx-a11y` rule and React Doctor's recommended and
TanStack Query rules as errors. React Doctor runs through `jsPlugins`, not a separate
CLI. Its framework setting is `vite`, so server-action advice is not applied to this
client-only application. There are no disabled-rule overrides or inline lint
suppressions. Warnings and unused suppression comments fail lint. Vite also fails
the production build if its logger reports a warning. These checks run in CI.

The Zod 4.6.7 pnpm patch changes two explanatory comments that Rollup mistakes for
purity annotations. It does not change executable code or suppress diagnostics.
Keep `patches/` available during Docker dependency installation. Remove the patch
when an upgraded Zod release builds cleanly without it. Jest DOM is pinned to the
maintainer-recommended 6.9.1 release; 6.10.0 was deprecated for accidentally shipping
breaking changes in a minor release.

Capabilities are deliberately simple in this initial release. Do not expose a
shared moderator capability publicly or embed it in a built website. A deployed
project needs a published maintainer contact and clear role-issuance policy.
