# Validation record

This file records actual checks from the initial implementation. It is updated
after the final verification pass; it does not claim deployment or catalogue
completeness. See the roadmap for unimplemented features.

## Verified on 2026-11-02

- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo nextest run --workspace --all-features`: 17 passed; the opt-in benchmark
  is intentionally excluded from the ordinary test run.
- `cargo test --workspace` and `cargo test --workspace --doc`: passed. There are
  no doctest examples requiring execution at present.
- PostgreSQL integration tests apply all migrations to disposable databases and
  cover empty state, filters, Unicode/localisation, both page sort orders, cursor
  scope, duplicate/idempotent submissions, ETags/HEAD/CORS, roles, private scans,
  approval/rejection, stale edits, rollback, update/delete, a second game, outbox
  backoff/replay, snapshot checksums, manifests, and streamed downloads.
- `pnpm check`: lint, TypeScript, four unit tests, and Vite production build passed.
- `pnpm test:e2e`: nine browser tests passed against the real API. Coverage includes
  navigation, direct links/reloads/history, Unicode and empty search, interactive
  API docs, submit/review/publication, six responsive theme captures, keyboard skip
  navigation, overflow checks, and axe WCAG 2/2.1/2.2 AA rules on core pages.
  The final run also passed all nine tests against the Docker-served production
  website with its Content Security Policy enabled. After captures are stored in
  [`validation/screenshots`](validation/screenshots/).
- `docker compose build` and `docker compose up -d --wait`: passed. PostgreSQL,
  API readiness and Caddy serving were checked. The container build used stable
  Rust 1.99.1; the native development toolchain was 1.98.1.
- `scripts/audit.sh`: cargo-audit with the documented lock-only exception,
  cargo-deny advisory/license/source checks, and pnpm audit passed.
- Gitleaks 8.30.1 scanned an exported copy of staged source files: no secrets found.
- OpenAPI and Orval generation completed successfully; the generated client is
  used for website statistics and generated types for catalogue responses.

## Frontend quality follow-up on 2026-11-02

- `pnpm check`: Oxfmt checked 41 files; Oxlint checked 35 JavaScript/TypeScript
  files with 370 active rules and zero diagnostics. TypeScript, four unit tests,
  and the Vite production build passed with no warnings or errors.
- All built-in JSX accessibility rules and the pinned React Doctor recommended
  and TanStack Query rules are errors. No disabled-rule overrides or inline lint
  suppressions remain. Temporary deliberate violations verified that both plugins
  reject bad code; an injected build warning verified the production warning gate.
- `WEB_TEST_ORIGIN=http://127.0.0.1:13087 pnpm test:e2e`, with matching local role
  tokens: ten tests passed against the Docker-served production build. Coverage
  additionally checks set/printing controls, private synthetic scan upload and
  moderator inspection, form/token reset, and phone-sized API explorer reflow.
  An earlier run had one browser resource load interrupted by the host's
  `ERR_NETWORK_CHANGED`; its trace identified the transport failure. The complete
  subsequent run passed without retries or changes to test thresholds.
- `pnpm install --frozen-lockfile`, `pnpm audit`, and `docker compose build web`
  passed. Install and build logs contain no warnings, and the dependency audit
  reports no known vulnerabilities. The Zod patch changes comments only.
- Captured 30 before and 30 after screenshots across home, contribution,
  moderation, API documentation, and status pages in both themes at 390, 768, and
  1440 pixels. Local captures are under `.local/lint-validation/`. Contribution and
  moderation captures are pixel-identical; the phone documentation page now fits
  its viewport. Final capture runs reported no browser warnings, errors, or
  horizontal overflow. Live counts/probe timestamps naturally differ between runs.
- Rebuilt and restarted the local website; the existing Tailscale HTTPS URL remains
  available. No Rust source or API contract changed. No PR was opened because this
  repository has no configured Git remote; screenshots cannot be attached remotely.

## Measured benchmark

`cargo test --release -p cardlattice-api --test performance -- --ignored --nocapture`
passed against a disposable PostgreSQL 18 database containing 20,007 original
synthetic records, with 50 warm-up operations and 500 recorded samples per workload.
The verified plan uses `Bitmap Index Scan on catalogue_search`. The benchmark
vacuum/analyse step flushes the bulk insertion's GIN pending list before measuring
the indexed workload. Raw measurements and environment limits are in
[`validation/benchmark.json`](validation/benchmark.json).

These are local, single-client SQL/response-construction measurements. They exclude
HTTP transport, concurrent production traffic and Cloudflare; they are not advertised
as service latency, throughput or an uptime promise.

## Dependency advisory boundary

SQLx 0.8's optional MySQL backend appears in Cargo.lock and locks `rsa 0.9.10`,
which has [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html)
with no fixed release. Cardlattice builds PostgreSQL only. `cargo tree --workspace
--all-features --target all --invert rsa` has no reachable dependency. A narrowly
documented cargo-audit exception covers that lock-only advisory; `scripts/audit.sh`
fails if RSA becomes reachable. Cargo-deny checks the actual graph with no advisory
exception. This is not a claim that the upstream vulnerability is fixed.

## Project standards deviations

- SQLx dynamic/runtime queries are validated against disposable PostgreSQL rather
  than compile-time query metadata. All user values remain parameterised.
- Shared role capabilities supply the initial abuse-resistant contribution path;
  per-person authentication/account provisioning is deferred. Public reads remain
  anonymous. No mandatory proprietary auth provider is added.
- One frontend package uses pnpm tasks directly; there is no redundant task runner.
- Private byte storage uses PostgreSQL and public snapshots use a persistent local
  volume. An object-store adapter is a future scaling feature.
- React Doctor recommended and TanStack Query rules are loaded through Oxlint.
  Version 0.5.8 is pinned after inspecting its unmodified MIT licence. Newer versions
  add use restrictions, so Renovate is constrained until a licence review permits
  an upgrade; the project's permissive licensing requirement takes precedence.
  The subsequent frontend quality pass removed the initial disabled-rule and
  complexity exceptions. Every recommended/TanStack Query rule is now an error,
  alongside all built-in JSX accessibility rules. The explicit Vite framework
  setting keeps server-action advice out of this client-only application.
  Status messages use native output elements, the probe chart uses a named figure,
  the contribution form is split by responsibility, and related request state
  updates together. Form controls retain label associations checked by axe.
- There was no predecessor UI or Git remote. A genuine before screenshot and a PR
  attachment cannot be supplied from an empty directory. Responsive after captures
  are generated by Playwright; remote CI and PR creation require a remote destination.

No Cloudflare credentials, production deployment, real card artwork, or external
catalogue data were used during validation.
