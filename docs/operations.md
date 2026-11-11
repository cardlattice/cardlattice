# Operations and security

The API emits structured JSON tracing and exposes `/metrics` only with its separate
metrics bearer capability. Do not expose a metrics token in a URL, web bundle, or
public monitoring dashboard. Restrict origin ports and use TLS in production.

Prometheus examples:

```promql
sum(rate(cardlattice_http_requests_total[5m])) by (route, status)
histogram_quantile(0.50, sum(rate(cardlattice_http_duration_seconds_bucket[5m])) by (le, route))
histogram_quantile(0.95, sum(rate(cardlattice_http_duration_seconds_bucket[5m])) by (le, route))
histogram_quantile(0.99, sum(rate(cardlattice_http_duration_seconds_bucket[5m])) by (le, route))
histogram_quantile(0.95, sum(rate(cardlattice_database_duration_seconds_bucket[5m])) by (le, operation))
cardlattice_invalidation_failed
cardlattice_invalidation_pending
cardlattice_publication_revision_lag
```

HTTP route templates bound metric cardinality. Counters include HTTP errors and
ingestion/upload request failures by route/status; database latency instrumentation
currently covers catalogue collection operations, not every SQL statement. Record
counts come from SQL, not estimates. Missing metadata/image counts are on `/v1/stats`.
The origin cannot observe ordinary CDN HIT requests; collect those measurements at
Cloudflare. The API explorer displays CF-Cache-Status when present, otherwise absent.

The public status page uses stored 30-second probes, up to 120 recent measurements,
publication time and invalidation counts. Probes older than 90 seconds are unknown.
During a complete database outage the status endpoint can return 503 rather than
fake healthy data. External probes are needed for a global availability/SLO claim.

## Dataset releases and withdrawal

The first release is published when real metadata becomes available. Thereafter,
the minute-based check permits at most one new release per UTC day, only when
non-synthetic data changed. Internal revision gaps are normal; revision lag during
a release day is expected and must not be treated as an immediate alert. Check
publication errors in structured logs and verify a pending change is exported
after the next UTC boundary. Daily publication retries independently of cache
invalidation; it cannot block immediate public API corrections.

Migration 0005 reconstructs old snapshot contents from historical revisions,
withdraws demo snapshots, collapses legacy same-day releases, and queues purges.
Withdrawn files may remain on the private dataset volume for audit but cannot be
downloaded through the API. A backup may retain them too. Do not re-serve the
volume directly from Caddy/R2 without applying the same withdrawal policy. Copies
already downloaded or held in old immutable browser caches cannot be recalled.

## Initial security review boundaries

- All user SQL values are bound. Dynamic identifiers/sort fragments are code-owned.
- Public reads use no identity. Writes require bearer capabilities, rate budgets,
  input checks, idempotency, terms and human approval. Moderator and contributor
  tokens must differ and be at least 32 bytes. They are compared as fixed-size hashes.
- Browser credentials stay in component memory. CORS permits public GET/HEAD only;
  writes use a same-origin reverse proxy. Tokens are not cookies, so cross-site
  requests cannot automatically attach them.
- JSON bodies are capped at 64 KiB. Scans are capped at 8 MiB and decoded with
  dimension/allocation limits on a blocking task. Only PNG/JPEG formats are accepted.
  No submitted image is automatically public. There is no remote-image fetch/SSRF.
- Public output is escaped JSON/React text. Reference URLs must use HTTPS. The static
  container sets CSP, nosniff, frame denial and a referrer policy.
- Writes serialise through a transaction lock. Stale approvals conflict, duplicate
  indexes prevent races, and failed mutations roll back the whole audit/outbox write.
- Private responses, status, metrics, error responses and scan attachments are
  no-store. CDN rules explicitly exclude private and query-bearing traffic.

Before a broad public launch, add individually revocable contributor credentials,
trusted-proxy rate limits, connection/upload concurrency limits, automated private
scan retention, a monitored rights contact, external uptime probes, resource/disk
quotas, and an independent security review. This checklist identifies real launch
work; it does not make public API access contingent on accounts or paid services.
