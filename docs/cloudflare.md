# Cloudflare deployment and cache behaviour

Cloudflare is optional. All endpoints, moderation, publication and retries run
locally without it. Use ordinary CDN cache hits; no Worker executes on each request.
The example uses URL purging, not Enterprise cache tags or custom cache keys.

## Origin deployment

1. Build the API and static website images. Provision PostgreSQL and a persistent
   dataset directory; back up both. Use a dedicated migration role with DDL rights
   and a restricted runtime role with only needed table/sequence permissions.
2. Mount distinct contributor, moderator and metrics secrets and a database URL.
   Run `cardlattice-api migrate` once per release. Do **not** run `seed` against the
   production catalogue. Configure the web reverse proxy to the origin API.
3. Set `PUBLIC_API_ORIGIN=https://api.example.org` to the exact hostname clients
   use for cached API URLs. The current purger targets one public origin; if the
   website also serves `/v1`, keep that host's API responses outside CDN caching or
   extend the purger to all origins. Never cache the same API on a second hostname
   without covering its invalidation.
4. Put the API hostname behind a proxied Cloudflare DNS record. Use Full (strict)
   TLS with a valid origin certificate. Restrict direct-origin access as appropriate
   for your host. The local Caddy example is HTTP on loopback, not a production TLS
   configuration.
5. Set both `CLOUDFLARE_ZONE_ID` and `CLOUDFLARE_API_TOKEN` on the API. The token
   needs only cache-purge access for that zone. Do not put it in the web build.

## Explicit Cache Rules

JSON is **not** cached automatically by default. Adapt
[`infra/cloudflare-cache-rules.json`](../infra/cloudflare-cache-rules.json) to your
hostname in the zone's `http_request_cache_settings` ruleset. The JSON is a reviewable
Rulesets API example, not an automatic deployment script. Preserve unrelated rules
when installing it.

1. Explicitly bypass `/v1/`, health, metrics, and dataset traffic by default.
2. Override eligibility only for GET/HEAD on the selected API hostname, with no
   query string, no Authorization header, and no Cookie header, on `/v1/games`
   and its descendants, `/v1/stats`, `/v1/datasets`, and immutable dataset URLs.
3. Respect origin cache headers. Do not override no-store or assign a positive
   TTL to error/redirect responses. Enable origin cache control. Do not cache
   contributions, moderation, status, health or metrics.

Origin headers: records use `max-age=15, s-maxage=86400`; collections/statistics/
manifest listings use `max-age=15, s-maxage=60`; artifacts use five-minute browser and one-day edge TTLs. Contents remain immutable,
but bounded caching permits withdrawal. Already downloaded/cached copies cannot
be recalled; purge withdrawn URLs at the CDN and allow browser TTLs to expire.
Errors and private responses use no-store. Cache headers and ETags also work without
Cloudflare. Long shared TTLs assume a working purge worker; monitor its outbox.

## Query keys and bounded variation

The origin canonicalises equivalent parameter order, default limit/sort, Unicode
search normalisation, and empty search values via a 308 Location. Unknown and
duplicate parameters fail. Query-free requests are the edge-cacheable common path.
The supplied rules **bypass all query-bearing requests**, including search/cursors,
rather than filling the CDN with arbitrary combinations. The default key can retain
the complete URL; no paid key customisation or query sorting feature is required.

Language selection is an explicit query parameter, not Accept-Language or a cookie.
No public response depends on login identity or request cookies. Do not exclude a
semantic parameter from a cache key while continuing to cache the response.

## Invalidation and verification

Approvals atomically create outbox events. Once origin data is committed, the worker
purges exact query-free URLs in batches of at most 30. Daily dataset releases run
independently and do not delay API invalidation. Withdrawal migrations enqueue the
withdrawn download URLs and manifest listing through the same durable outbox. It checks both HTTP
success and Cloudflare's JSON success flag. Failures remain pending with backoff,
then enter a visible failed state after ten attempts. A moderator can replay failed
events in the web UI or with `POST /v1/moderation/outbox/{id}/replay`.

Default parent/collection URLs are purged; filtered lists/search are bounded by
60-second response TTLs and bypass the supplied edge rule. Failed URL purges can
leave record data stale up to the one-day edge TTL. In-flight cache fills after purge
are also bounded by TTL. If strict freshness is needed, reduce record TTLs.

After deployment, make two requests to an eligible URL and inspect `CF-Cache-Status`
and `Age`. Confirm the second can be HIT, a query-bearing request is BYPASS/DYNAMIC,
and all private responses remain uncached. Approve a synthetic staging correction,
verify worker completion and the API ETag change. Verify the next eligible daily
release separately using non-synthetic staging metadata and its snapshot digest. Do not report a CDN
hit rate without actual edge measurements. Test purge credentials and negative
cases on a staging zone before relying on long TTLs.

References: Cloudflare's [cache overview](https://developers.cloudflare.com/cache/get-started/),
[Cache Rules settings](https://developers.cloudflare.com/cache/how-to/cache-rules/settings/),
and [purge by URL](https://developers.cloudflare.com/cache/how-to/purge-cache/purge-by-single-file/).
