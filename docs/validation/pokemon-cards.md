# Official checklist import validation — 2026-11-02

Applied the reviewed factual batch to the running local Compose catalogue after a
successful dry run and private PostgreSQL custom-format backup. The import created
1,278 provisional card identities and 1,278 distinct printings across all eight
existing real sets. An immediate second applied run inserted zero records and
reported 1,278 unchanged printings. No synthetic records or images were published.

Every numbered entry was checked through the public API against the reviewed batch,
following every cursor with a 50-record page limit and checking set membership,
unique IDs, collector number and Unicode name. Per-set counts were 188, 130, 295,
124, 122, 120, 158 and 141 in reviewed-batch order. Eight printing details passed
source-reference and conditional-GET/304 checks. Unicode search returned Nidoran♀.
The Tailscale HTTPS endpoint returned the same live counts with certificate checking.

The consolidated invalidation event contained 2,572 distinct paths and completed
on its first attempt. Cloudflare is not configured here: this verifies transactional
local invalidation processing, not a live edge purge. Existing retry/replay tests
passed. Today's single dataset manifest and its content digest stayed unchanged;
the new cards will appear in the next eligible UTC daily release.

## Checks

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo nextest run --workspace --all-features`: 28 passed; opt-in benchmark run
  separately below.
- `cargo test --workspace --doc --all-features`: passed (no doctest examples).
- `cargo test -p cardlattice-api --test performance -- --ignored --nocapture`:
  passed on a disposable PostgreSQL database with 20,007 original synthetic records,
  50 warmups and 500 measured samples; indexed query plan verified. These are local
  workload measurements, not production latency or CDN performance claims.
- `pnpm check`: Oxfmt, zero-warning Oxlint including JSX-a11y and React Doctor,
  TypeScript, four unit tests and production build passed.
- `pnpm test:e2e`: 12 passed against a separate original-fixture catalogue,
  including contribution/moderation/publication and parent-filter navigation.
- `bash scripts/audit.sh`: dependency advisories, licenses and sources passed;
  no new advisory exceptions or dependencies were added.
- `docker compose build api web`: passed; rebuilt services deployed locally.
- OpenAPI 3.1 and the generated TypeScript client regenerated from Rust schemas.
- Offline extraction reproduced all 1,278 entries from checksum-verified publisher
  PDFs plus the 12 explicitly documented visual transcription corrections.

Live set detail and filtered printing pages were checked at 390, 768 and 1440 px
in both themes: 12 after-state axe/layout checks and 12 before screenshots, with
zero browser warnings/errors. Pagination retained the set filter. Provisional card
detail also passed axe/layout checks at phone and desktop widths. Automated axe
checks support, but do not establish, full WCAG 2.2 AA conformance.

## Before and after

- Before: [empty set, dark phone](screenshots/pokemon-cards/before-set-dark-phone.png)
- After: [populated set, dark phone](screenshots/pokemon-cards/after-set-dark-phone.png)
- Before: [empty printings, light desktop](screenshots/pokemon-cards/before-printings-light-desktop.png)
- After: [set-filtered pagination, light desktop](screenshots/pokemon-cards/after-printings-light-desktop.png)

## Limits and delivery

Main numbered English checklist sections only. The 30th Celebration Classic
Collection and separate Basic Energy section require further identity review.
Rarity, rules, category, HP and printing variants remain unknown; artwork is absent.
Card identities remain provisional pending comparison of their characteristics.
Source PDF font metadata warnings and review boundaries are documented in
`data/pokemon/README.md`; no application lint/build warnings were waived.

Changes are on `luna/official-pokemon-cards`. No Git remote is configured, so a PR
cannot be opened from this checkout. No third-party catalogue database was used.

The live phone check found that a UUID in the attributes JSON could expand the
detail grid. Grid tracks now allow shrinking and metadata JSON wraps within its
panel. A dedicated phone regression test exercises this provisional-identity case.

- Before wrapping fix: [card detail, dark phone](screenshots/pokemon-cards/before-card-dark-phone.png)
- After wrapping fix: [card detail, dark phone](screenshots/pokemon-cards/after-card-dark-phone.png)

Gitleaks scanned the staged source snapshot with redaction enabled: no leaks found.
