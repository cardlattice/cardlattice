# Readable source citations — 2026-11-02

Official checklist and set imports previously displayed their entire internal audit
description on the record page. Those known importer-generated notes now display
as named links to the original Pokémon references. Checksums, retrieval timestamps,
review limitations and full provenance remain unchanged in the API and database;
a separate JSON link makes them accessible. Other source descriptions retain their
full text so contributor credits and attribution are preserved.

Validation: `pnpm check` passed (Oxfmt, zero-warning Oxlint with JSX-a11y and React
Doctor, TypeScript, four unit tests and Vite build). `docker compose build web`
passed and the rebuilt website was deployed. No backend or data changes.

Live card/set source sections passed 12 accessibility/layout/browser-diagnostic
checks across light/dark themes and 390/768/1440 px viewports. Browser assertions
confirmed that the audit paragraphs and field counts are absent from the known
import citations, while source links remain. An intercepted response with an
additional community credit confirmed that the credit remains visible. The API
still returns the full original checksum-bearing description. Tailscale HTTPS
responded with HTTP 200 and certificate validation enabled.

- Before: [card, dark phone](screenshots/source-citations/before-card-dark-phone.png)
- After: [card, dark phone](screenshots/source-citations/after-card-dark-phone.png)
- Before: [card, light desktop](screenshots/source-citations/before-card-light-desktop.png)
- After: [card, light desktop](screenshots/source-citations/after-card-light-desktop.png)

The baseline is commit `27aecec`. Changes are on `luna/readable-source-citations`.
No Git remote is configured, so no PR could be created. Backend tests were not
rerun for this presentation-only change.
