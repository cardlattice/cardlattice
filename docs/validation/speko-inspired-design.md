# Design refresh — 2026-11-11

Adapted Speko's centered typography, blue accents, dotted textures and framed product
panels to Cardlattice. No third-party logos, artwork or source code were reused.
The home page now pairs live set links with a compact API excerpt. Shared theme
colors, controls and panels extend across catalogue, documentation and contribution
pages. Both color modes remain supported; no fabricated catalogue metrics were added.

`pnpm check` passed: Oxfmt, Oxlint including JSX-a11y and React Doctor with zero
warnings, TypeScript, four unit tests and production build. Live axe and overflow
checks passed for home, sets, docs and contribution pages at 390, 768 and 1440 px in
both themes (24 screens). Browser warnings/errors: zero. The full fixture-dependent
write/moderation E2E suite was not rerun for this visual change. Database contents
were preserved.

Before and after use the same live catalogue and viewports:

- [Before desktop](screenshots/redesign/before-desktop.png)
- [After desktop](screenshots/redesign/after-desktop.png)
- [Before dark phone](screenshots/redesign/before-phone-dark.png)
- [After dark phone](screenshots/redesign/after-phone-dark.png)

The Vite dev server runs as the user service `cardlattice-web-dev` on port 13088,
with API requests proxied to 18087. The existing Tailscale HTTPS route on port 5206
now points to the dev server. Both the HTML and API endpoints returned HTTP 200
with TLS certificate validation. Other Tailscale routes were preserved. The stale
migration container image was rebuilt; migrations completed using the existing DB.

Branch: `luna/speko-inspired-design`. No Git remote is configured, so no PR was
created. The proposed Git-backed data workflow has not been implemented in this
presentation change.
