# Contributor-facing YAML catalogue

Validated 2026-11-11. Data authoring uses one YAML file per printing, with shared
game/set context. Most card files contain only `name`. Compiled artifacts remain
JSON. No upstream catalogue records were imported; TCGdex's directory organisation
was inspected for inspiration.

Card and set facts and all 2,565 semantic record keys are preserved. Source-evidence
files, per-record references, review batches and citation URLs were removed from
the data repository at the operator's request. They are not compiled into dataset
artifacts. Git history and manifest commits track changes. Backend drafts now
accept absent source evidence, and indexing does not create synthetic evidence.

Validation:

- Rust formatting and Clippy with warnings denied pass.
- 36 nextest tests pass; the opt-in performance benchmark remains skipped.
- Workspace doc tests and the production Rust build pass.
- Oxfmt, Oxlint with JSX accessibility and React Doctor, TypeScript, four unit
  tests and the production website build pass.
- All 12 existing browser tests pass. The new evidence-free moderation regression
  also passes after correcting its expected submission status to the API's 200
  contract. The focused rerun removes conflicting terminal colour variables and
  emits no warnings.
- Data `scripts/check.sh` validates actual YAML without a database and runs six
  compiler regression tests.
- Repeated builds produce identical manifests and artifact bytes.
- Integration tests cover source-free indexing, atomicity, idempotency, conflicts,
  retirement, and exact artifact downloads with the new wire format.
- The full real catalogue indexes 2,565 records and repeats with zero changes;
  its index contains zero source and provenance rows.

Regressions cover inherited identity/language/region/date, leading-zero collector
numbers, source-free round trips, shared identities for reprints, unknown fields,
duplicate YAML keys, unsupported tags, aliases/anchors/merges, multiple documents,
invalid attributes, duplicate external identifier patterns, stray files and symlinks.

The data CI job explicitly runs `scripts/check.sh` before reproducibility checks
and upload. Main remains the empty bootstrap until the initial PRs are approved
and merged. No live database or developer deployment was replaced.
