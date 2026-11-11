# Separate repositories and CI catalogue validation

Validated 2026-11-11. Application: `cardlattice/cardlattice`; reviewed metadata:
`cardlattice/cardlattice-data`. Both repositories are public. No artwork or private
scan/review tables were copied into the data repository.

## Identity and artifacts

Catalogue keys now use readable game/set/language/region/collector identifiers.
No UUID aliases or migrations from the unreleased UUID schema are provided.
The existing eight-set metadata batch was converted to semantic keys without
changing its names, collector numbers or source evidence.

The offline compiler produced a 2,565-record artifact: 1 game, 8 sets, 1,278
provisional card identities and 1,278 printings. A fresh disposable PostgreSQL
index inserted 2,565 records. Repeating the same import inserted/updated/retired
zero records and reported 2,565 unchanged records. The running developer database
was not reset or migrated.

## Checks

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: pass.
- `cargo nextest run --workspace --all-features`: 31 passed; the existing opt-in
  20,000-record performance benchmark was not run.
- `cargo test --workspace --doc --all-features`: pass.
- `pnpm check`: pass, including Oxfmt, Oxlint with JSX accessibility/React Doctor,
  TypeScript, four unit tests and the production Vite build.
- `pnpm test:e2e`: 12 passed against a disposable database and API on port 18197;
  includes real contribution/moderation, scoped links and axe checks in light/dark
  modes at phone/tablet/desktop sizes. An initial attempt used an occupied port;
  it was stopped and the suite rerun against the verified isolated API.
- `cargo build --release --package cardlattice-api --locked`: pass.
- `docker compose build`: pass for the API, migration and website images.
- `scripts/audit.sh`: pass with no emitted warnings. SQLx upgraded to 0.9, removing
  the old optional RSA advisory exception. One documented exception remains for
  RUSTSEC-2026-0459 in SQLx's lock-only SQLite dependency: SQLite is disabled, an
  all-target/all-feature reachability guard fails if it becomes used, and
  cargo-deny independently scans the actual build graph. No active dependency
  vulnerability is suppressed.

New tests cover deterministic builds, checksum corruption, invalid references,
synthetic data rejection, duplicate IDs, dry runs, atomic/idempotent indexing,
corrections, conflict detection, retirement and serving the exact CI artifact.

CI workflow configuration is included in the data PR. GitHub execution and release
publication are separate from the local results above. Daily releases begin only
once that workflow is merged to the default branch. No production auto-deploy or
R2 credentials have been configured.
