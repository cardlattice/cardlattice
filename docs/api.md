# API reference

The source of truth is generated [OpenAPI 3.1](openapi.json), also served at
`/openapi.json`. The website's `/docs` page reads that document and provides an
interactive same-origin GET explorer, parameter fields, schemas, status/headers,
and curl/Rust/JavaScript/Python examples. Orval generates the TypeScript client and
TanStack Query hooks from the same document.

| Public GET/HEAD | Response |
| --- | --- |
| `/v1/games` | Paginated supported games |
| `/v1/games/{game}` | Game record under `data` |
| `/v1/games/{game}/series` | Paginated series |
| `/v1/games/{game}/series/{id}` | Series detail |
| `/v1/games/{game}/sets` | Paginated sets |
| `/v1/games/{game}/sets/{id}` | Set and first 20 related printings |
| `/v1/games/{game}/cards` | Paginated card identities |
| `/v1/games/{game}/cards/{id}` | Card identity and first 20 printings |
| `/v1/games/{game}/printings` | Paginated printings, including `setId`/`cardId` filters |
| `/v1/games/{game}/printings/{id}` | Printing, variants, permitted images and provenance |
| `/v1/search` | Paginated cross-game search; optional `game` slug filter |
| `/v1/stats` | Current record counts, missing metadata, image coverage, unknown completeness |
| `/v1/datasets` | Latest 50 immutable manifests; old artifact URLs remain valid |
| `/v1/status` | Latest 120 persisted probes and publication health |
| `/v1/changelog` | Latest 50 published revisions |
| `/datasets/{filename}` | Immutable NDJSON download |
| `/healthz`, `/readyz` | Process liveness and database readiness |

All public reads are anonymous. `game` is a slug such as `pokemon`; record IDs
are Cardlattice UUIDs. JSON uses camelCase. Errors are always
`{"error":{"code":"...","message":"..."}}`, including extractor/body-limit errors.

## Collection parameters

- `limit`: 1–100, default 20. Lists always have `data`, `nextCursor`, and `limit`.
- `cursor`: opaque continuation from the previous response. Keep filters unchanged.
- `sort`: `name` ascending or `newest` descending; UUID breaks ties.
- `q`: at most 120 Unicode characters and 8 whitespace-delimited terms.
- `language`: language tag; translations fall back to canonical names. Printing
  lists also filter to the requested printing language.
- `setId`, `cardId`, `rarity`: printing filters.
- `game`: a game slug for global search.

Query order and default values normalise through 308 redirects. Unknown/repeated
parameters are rejected. Query strings are limited to 4096 bytes. Cursors are
validated against their endpoint/filter scope. The first UI selects from up to
100 related records when creating a printing; larger catalogues should use the
API until searchable relationship selectors are implemented.

## Conditional requests

Record/list responses include ETag, Last-Modified, Cache-Control and CDN-Cache-Control.
An `If-None-Match` match returns 304, supports weak request validators and lists,
and takes precedence over If-Modified-Since. HEAD has the same representation
validators as GET and no response body. Errors and private data are no-store.
Downloads use digest ETags and one-year immutable caching.

## Contributions and moderation

`POST /v1/contributions` requires the contributor bearer capability, a UUID
`Idempotency-Key`, and a `Submission`. Accepted operations are `create`, `update`,
and `delete`; updates/removals require `targetId` and `baseUpdatedAt`. The draft
contains typed core fields, bounded attributes and a source. `termsAccepted` must
be true; the honeypot `website` field must be absent or empty. Identical replays
return the original ID; a reused key with a different request returns 409.

`GET /v1/contributions/{id}` returns the capability's submission and decisions;
moderators can inspect any submission. With shared role capabilities, all users of
that capability share ownership. Use individually issued capabilities in a future
account implementation before inviting untrusted contributors at scale.

`POST /v1/contributions/{id}/scans/{front|back}` takes raw PNG/JPEG bytes, up to
8 MiB, at most 6000 pixels per dimension and 24 megapixels. No SVG or remote URL
fetching. Accepted uploads retain private review-only rights. A moderator retrieves
the original from `/v1/moderation/scans/{id}` as an attachment.

Moderator endpoints: `GET /v1/moderation/contributions?cursor=UUID`,
`POST /v1/moderation/contributions/{id}/review` with `decision` and `reason`,
`GET /v1/moderation/outbox`, and `POST /v1/moderation/outbox/{id}/replay`.
Approvals are transactional and optimistic updates reject stale bases. Rejections
preserve history. Public clients cannot bypass review by writing directly to records.

## Examples

```sh
curl -i 'http://localhost:8080/v1/games/pokemon/cards?limit=10'
curl -i -H 'If-None-Match: "replace-with-returned-etag"' http://localhost:8080/v1/games
```

```js
const response = await fetch('http://localhost:8080/v1/games/pokemon/cards?limit=10');
if (!response.ok) throw new Error(`HTTP ${response.status}`);
const { data, nextCursor } = await response.json();
```

```python
import json
from urllib.request import urlopen
with urlopen('http://localhost:8080/v1/games/pokemon/cards?limit=10') as response:
    page = json.load(response)
    print(page['data'], page['nextCursor'])
```

```rust
// Cargo dependencies: reqwest with json/rustls-tls, serde_json, tokio with full.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page: serde_json::Value = reqwest::get(
        "http://localhost:8080/v1/games/pokemon/cards?limit=10"
    ).await?.error_for_status()?.json().await?;
    println!("{}", page["data"]);
    Ok(())
}
```
