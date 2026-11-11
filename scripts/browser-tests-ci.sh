#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

# The image and SDK must match. Derive the version from the frozen-lockfile install.
version="$(node -p "require('./apps/web/node_modules/@playwright/test/package.json').version")"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]

# The API runs on the Linux CI host. Host networking lets both Vite and Chromium
# reach it. This official image already contains browsers, fonts and native deps;
# do not run apt or `playwright install --with-deps` on the runner.
docker run --rm --init --ipc=host --network=host \
  --volume "$root:/work" --workdir /work \
  --mount type=volume,destination=/work/node_modules,volume-nocopy \
  --mount type=volume,destination=/work/apps/web/node_modules,volume-nocopy \
  --env CI=true --env CONTRIBUTOR_TOKEN --env MODERATOR_TOKEN \
  --env API_PROXY_TARGET --env WEB_TEST_ORIGIN \
  "mcr.microsoft.com/playwright:v${version}-noble" \
  /bin/bash -euc 'npm install --global --allow-scripts=pnpm pnpm@12.4.1 && pnpm install --frozen-lockfile && pnpm test:e2e'
