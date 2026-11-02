# Contributing

Start with `docker compose up --build -d`. The complete contributor/moderator
walkthrough is in the README. All example data must be original and visibly
synthetic. Never add card artwork, catalogue dumps, or third-party API responses
to tests, screenshots, documentation examples, or seeds.

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

Capabilities are deliberately simple in this initial release. Do not expose a
shared moderator capability publicly or embed it in a built website. A deployed
project needs a published maintainer contact and clear role-issuance policy.
