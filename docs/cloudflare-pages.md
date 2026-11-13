# Cloudflare Pages deployment

The public frontend is a Vite SPA using TanStack Query to download a verified
catalogue snapshot and TanStack DB to keep its records in a browser collection.
Browsing, search, filters, pagination and related-printing lookup run client-side.
No Rust service, database connection or secret is shipped to the browser.

- `cardlattice.com`: Pages project `cardlattice-app`, built from `apps/web`.
- `status.cardlattice.com`: Pages project `cardlattice-status`, built in
  `cardlattice/cardlattice-data` from the validated catalogue. This replaces the
  GitHub Pages coverage dashboard and serves `/data/manifest.json` plus the
  checksum-named JSON snapshot.
- `api.cardlattice.com`: temporary Pages project `cardlattice-api-coming-soon`,
  serving `infra/api-coming-soon`. Replace its domain/DNS assignment when the
  Rust server is deployed. No API endpoints are live yet.

## Continuous deployment

`.github/workflows/pages.yml` checks and builds the app on PRs and main pushes.
The production job downloads that run's artifact and deploys the app and API
placeholder only on main (including manual dispatch on main). PR code never
receives production credentials. Production deployments are serialized.

The data repository's Catalogue workflow similarly deploys its verified status
site and static snapshot from main. New data does not require an app rebuild.
The browser loads one snapshot per visit; reload the app to pick up a new release.

Both repositories require:

- Repository variable `CLOUDFLARE_ACCOUNT_ID`.
- Repository secret `CLOUDFLARE_API_TOKEN` with Cloudflare Pages Edit on that account.
- A `production` GitHub environment (optional environment protection rules).

Create the three Pages projects as Direct Upload projects with production branch
`main`. Attach each custom domain in Cloudflare Pages and point its proxied CNAME
at the project's returned `pages.dev` hostname. The domain must be on the same
Cloudflare account for apex hosting. DNS Edit and Zone Read are needed only when
provisioning domains, not for routine CI deployments.

Keep the old GitHub Pages deployment available until the new status domain and
data downloads have been verified. The replacement workflow no longer deploys
GitHub Pages; old URLs can be retired or redirected after cutover.

## Local development and validation

```sh
pnpm install --frozen-lockfile
pnpm check
pnpm test:e2e
pnpm dev
```

The app defaults to `https://status.cardlattice.com/data/`. For a separately hosted
snapshot, set `VITE_CATALOGUE_URL` to its directory URL (with a trailing slash)
before starting Vite or building. It must serve a valid manifest and bundle,
allow browser CORS requests, and use HTTPS outside localhost. The published
snapshot contains no artwork or secrets. Hash, size, revision, schema and record
counts are checked before accepting it. Downloads fail visibly rather than
falling back to invented data.

Browser tests intercept the two static JSON requests with original test records;
they do not need the Rust API. Existing Rust integration tests still exercise the
server independently. Cloudflare Pages serves SPA deep links using `_redirects`.

When the Rust API is ready, remove the placeholder deployment step before changing
`api.cardlattice.com` to the server. Keep catalogue browsing independent of it.
