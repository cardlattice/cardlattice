# Cardlattice

Free, open trading card metadata. Independently maintained public data
infrastructure, with anonymous read access and original, reviewed contributions.
Software is MIT-licensed; original metadata is CC0-1.0. Card artwork is separate.

## Run the whole application

Requirements: Docker with Compose v2. No account, API key, Cloudflare, OCR service,
or external catalogue API is required.

```sh
docker compose up --build -d
docker compose ps
```

- Website: <http://localhost:3000>
- API: <http://localhost:8080/v1/games>
- Interactive API reference: <http://localhost:3000/docs>
- OpenAPI 3.1: <http://localhost:8080/openapi.json>
- Readiness: <http://localhost:8080/readyz>

Compose generates local credentials in a private Docker volume, starts PostgreSQL,
and applies versioned migrations. New databases start empty. Import the reviewed
catalogue artifact from [cardlattice-data](https://github.com/cardlattice/cardlattice-data):

```sh
docker compose run --rm --no-deps -v "$PWD/.local/catalogue:/import:ro" api \
  index-catalogue /import/manifest.json
# Apply after inspecting the dry-run summary:
docker compose run --rm --no-deps -v "$PWD/.local/catalogue:/import:ro" api \
  index-catalogue /import/manifest.json --apply
```

Download both the release manifest and its catalogue JSON into `.local/catalogue`.
The initial catalogue contains eight sets and 1,278 provisional card identities and
printings, with no artwork. See [CI builds and indexing](docs/data-pipeline.md).
Catalogue IDs are readable game/set/language/collector keys; this unreleased schema
requires a fresh database rather than an upgrade from the earlier UUID experiment.

Set `CARDLATTICE_WEB_PORT` and `CARDLATTICE_API_PORT` to change host ports. Production
must use the separate deployment guidance. Published ports bind to loopback.
Synthetic data is opt-in for disposable development databases:
`docker compose --profile demo run --rm seed`. It is never automatically restored
on restart, and the importer preserves retired records as audit tombstones.

## Try the complete contribution workflow

1. In a disposable development catalogue, run the opt-in demo seed above, then
   browse **Catalogue → Pokémon**. Keep test contributions out of a real catalogue.
2. Retrieve local capability tokens:

   ```sh
   docker compose exec api cat /run/cardlattice/contributor
   docker compose exec api cat /run/cardlattice/moderator
   ```

3. Open `/contribute`, enter the contributor token, create a synthetic card, explain
   your original source, mark it synthetic, and accept the terms. Optionally attach
   a front/back PNG or JPEG after submitting. Originals remain private.
4. Open `/moderate`, enter the moderator token, inspect the draft, give a reason,
   and approve. Rejection also records an audit decision.
5. The approved record becomes readable immediately. The transaction also creates
   a revision and invalidation event. The worker purges affected URLs on its
   five-second tick (a successful no-op locally). Datasets are built in the data repository CI and registered when an approved artifact is indexed; demo-only catalogues have no public downloads.
   Check `/datasets`, `/changelog`, `/status`, and the moderator
   outbox. Failed invalidations have bounded exponential retries and manual replay.

All public GET/HEAD requests remain unauthenticated. The initial contribution
mechanism uses separate, operator-issued role capabilities, not public accounts.
Tokens identify a role capability, not an individual; see the limitations below.

```sh
curl -i http://localhost:8080/v1/games
curl -i 'http://localhost:8080/v1/games/pokemon/sets?limit=1'
curl -i 'http://localhost:8080/v1/search?q=Mega%20Evolution'
```

## Workspace

| Path | Responsibility |
| --- | --- |
| `apps/api` | Axum server, API contract, moderation, integration tests |
| `apps/web` | Vite, React, TypeScript, Tailwind v4, TanStack Router/Query/Form |
| `crates/catalogue` | Domain validation, Unicode normalisation, cursors |
| `crates/ingestion` | Reviewed official set imports, demo retirement, private scans, OCR interface |
| `crates/publishing` | Streaming snapshots, manifests, durable invalidation worker |
| `crates/observability` | Prometheus histograms/counters, structured tracing |
| `migrations` | Ordered PostgreSQL schema migrations and indexes |
| `data` | Opt-in synthetic test fixtures; reviewed facts live in cardlattice-data |
| `docs` | Architecture, operations, sourcing, API, roadmap |
| `infra` | Production Dockerfiles, local proxy, Cloudflare examples |

## Native development and validation

Use latest stable Rust, Node 24, pnpm 12.4.1, and PostgreSQL 18 with `pg_trgm`.
`mise.toml` records tool choices. Set variables from `.env.example` in your shell;
the server does not implicitly load `.env`. Secrets can instead be supplied through
`DATABASE_URL_FILE`, `CONTRIBUTOR_TOKEN_FILE`, `MODERATOR_TOKEN_FILE`, and
`METRICS_TOKEN_FILE`.

```sh
cargo run -p cardlattice-api -- migrate
# Choose reviewed real metadata, or `seed` only for a disposable demo database:
cargo run -p cardlattice-api -- index-catalogue .local/catalogue/manifest.json --apply
cargo run -p cardlattice-api -- serve
# In another terminal:
pnpm install --frozen-lockfile
pnpm dev
```

The Vite proxy defaults to `http://localhost:8080`; override `API_PROXY_TARGET` if
needed. The API startup validates configuration. Migrations are explicit deployment
commands, never an implicit action in the HTTP server.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
# DATABASE_URL must point to a local test PostgreSQL role with CREATEDB:
cargo test --workspace
cargo nextest run --workspace --all-features
cargo test --workspace --doc
pnpm check
pnpm --filter @cardlattice/web exec playwright install chromium
# API must be running with matching test tokens; tests create synthetic records:
pnpm test:e2e
cargo audit
cargo deny check
pnpm audit
```

The PostgreSQL test harness creates disposable databases and applies real migrations.
Browser tests require `CONTRIBUTOR_TOKEN` and `MODERATOR_TOKEN`; use a disposable
development catalogue. They test the actual API, not mocked success responses.
They also capture light/dark phone/tablet/desktop screenshots and run axe checks.

`pnpm check` includes Oxfmt formatting, Oxlint with JSX accessibility and React
Doctor rules, TypeScript, unit tests, and the production build. Lint warnings,
unused lint suppressions, formatting differences, and Vite build warnings fail
the check. Run `pnpm format` to apply formatting across the frontend package.

Generate the contract and TypeScript client/hooks after an API change:

```sh
cargo run --quiet -p cardlattice-api -- openapi > docs/openapi.json
pnpm --filter @cardlattice/web generate
```

Opt-in benchmark: `cargo test --release -p cardlattice-api --test performance -- --ignored --nocapture`.
It generates 20,000 additional original synthetic records in a disposable database,
warms up, reports measured p50/p95/p99, and prints the search query plan. It does not
measure Cloudflare or claim production capacity.

## Scope and remaining work

This is a working first release, not a complete production catalogue. The supported
flow is browse → draft → review → publish → invalidate/export. See
[the roadmap](docs/roadmap.md) and [validation report](docs/validation.md) for exact
verification and outstanding work. In particular:

- Manual scan entry works; an OCR service implementation is not bundled.
- Contributor/moderator capabilities are shared per role. Per-person access,
  revocation and public self-service onboarding remain future work.
- Localisation, variants, external IDs and image-rights tables are modelled, but
  dedicated editing/review interfaces for these supplementary records remain.
- R2 is optional future storage integration; current snapshots use a persistent
  filesystem and scans use private PostgreSQL bytea storage.
- Live Cloudflare purging requires an operator's zone and secret. Local semantics
  and injected purge failures are tested; a real zone is not required for development.
- Production deployment and automatic consumption of approved data releases require operator setup.

Read [architecture](docs/architecture.md), [schema](docs/schema.md),
[API](docs/api.md), [contributing](docs/contributing.md),
[sourcing and licensing](docs/licensing.md), [Cloudflare](docs/cloudflare.md),
[cost model](docs/costs.md), and [backup/recovery](docs/backup.md).

To populate the reviewed Pokémon sets with independently transcribed card names and
collector numbers, see the [official checklist import workflow](data/pokemon/README.md#numbered-card-checklists-2026-11-02)
and its [validation record](docs/validation/pokemon-cards.md). It imports factual
metadata only, with provisional card identities and no artwork.
