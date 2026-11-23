# Catalogue status dashboard

React/Vite dashboard consuming the verified public catalogue from
`https://cardlattice-status.pages.dev/data/`. The data repository remains responsible
for YAML validation, Rust compilation, immutable releases and snapshot publishing.
This app verifies the downloaded checksum, source revision and printing count before
showing coverage. TanStack Query handles caching, cancellation and retries.

From the repository root:

```sh
pnpm install --frozen-lockfile
pnpm dev:status
pnpm --filter @cardlattice/status test
pnpm build:status
```

Vite serves port 8788. Production output is `apps/status/dist`. No Rust compiler,
local data checkout, Cloudflare account or credentials are needed to run the app.

`src/components` contains one React component per file. `src/hooks` contains hooks,
and `src/lib` contains queries, coverage calculations and the snapshot loader.
All application files have a per-file 100% Vitest statement, branch, function and
line coverage threshold. Oxlint includes React Doctor and TanStack Query rules;
Oxfmt handles formatting. Braces are required on control-flow blocks; type assertions
and non-null assertions are rejected by lint. Incoming JSON is validated before use.

Each component keeps its own StyleX definitions beside its JSX. Shared palette tokens automatically follow the
system light/dark preference. There is no theme picker or stored theme preference.
SVG artwork is rendered by individual React components. The global stylesheet only
contains the font declaration and base resets in a lower-priority CSS layer.
The combobox and every select use Base UI primitives
with keyboard navigation. Flags come from `flag-icons`; Inter comes from
`@fontsource-variable/inter`. These assets are bundled locally, not fetched from a CDN.
Clicking or focusing the search input opens suggestions. Hovering or focusing a
progress segment shows its set count and percentage; Escape dismisses the tooltip.

## Deployment

The Pages workflow uploads the dashboard artifact for PRs. Main can deploy it to
its own `cardlattice-status-app` Pages project once the repository variable
`STATUS_PAGES_PROJECT` is set to that project name. It uses the existing
`CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN` credentials.

Move the `status.cardlattice.com` custom domain to the new project only after the
new deployment is verified. Keep the existing `cardlattice-status` data project:
its `pages.dev/data/` endpoint is the canonical snapshot source for this app and
continues to be deployed by `cardlattice-data`. The app must never overwrite that
project or republish a stale snapshot. No scheduled app rebuild is necessary when
data changes because coverage is loaded at runtime.

## Migration provenance

Ports the dashboard changes from cardlattice/cardlattice-data#30:

- `36147f9`: dashboard redesign, including language coverage tables and filtering.
- `acc80af`: layout stability improvements, retained through bundled typography,
  fixed icon dimensions and controlled pagination.
- `9dd55f6`: prevent text selection on status badges.

Original design: Sean Lodhammar. The Windows-only YAML validator test adjustments
in that PR belong to `cardlattice-data`; they are not dashboard code and remain there.
