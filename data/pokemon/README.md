# Official Pokémon set facts

The first reviewed batch contains eight set names and announced US release dates
from the English-language product pages on The Pokémon Company International's
official North America press site. It was collected on 2026-11-02. Delta Reign's
2026-11-06 date is an announced future release, not evidence it is already available.
This batch is deliberately incomplete: it contains no card records, card counts,
set codes, series assignments, promotional subsets, prices, or artwork. Missing
fields remain unknown. The named source pages are the only data inputs.

## Source and rights boundary

- Source host: `press.pokemon.com`, specifically `/en/products/` pages.
- [Media usage guidelines](https://press.pokemon.com/en/Assets-Use-Terms) permit
  informational/editorial use, restrict commercial reuse and sublicensing of the
  publisher's content, and prohibit implying endorsement. This catalogue records
  basic facts in its own schema; it does not republish product prose or assets.
- CC0 covers Cardlattice's original factual metadata contribution only. It does not
  relicense Pokémon's website, marketing text, branding, artwork, or media assets.
- Each record retains its official URL, retrieval time, page SHA-256, source method,
  and rights explanation. The import audit also retains the reviewed input and the
  exact catalogue revisions it produced. The initial reviewer is explicitly an
  operator-requested automated source-field review, not a claimed human review.
- The main `www.pokemon.com` catalogue returned an access challenge during this
  run. It was not bypassed. Its general terms are not treated as a grant for bulk
  catalogue scraping. No TCGdex or other third-party catalogue was consulted for data.

## Discover a small batch

Maintain `scan-plan.json` as a manually selected list of actual set product pages.
Do not treat every TCG product as a set: collections, decks and accessories can
also have product pages. The scanner deliberately does not recursively follow links.

```sh
cargo run -p cardlattice-api -- scan-pokemon-sets \
  data/pokemon/scan-plan.json .local/pokemon-candidates-new.json
```

Discovery needs network access but no database or credentials. It accepts at most
24 unique URLs, only on the exact official product path, with no query parameters
or redirects. It requests `robots.txt` first and checks the policy reviewed on
2026-11-02: a five-second wildcard crawl delay and no product-page prohibition.
If that policy changes, the scanner stops for a source-policy review. Requests
are sequential with a five-second delay, a 20-second timeout and 2 MiB body limit.
Missing or ambiguous title/date fields, partial dates, challenges and HTTP failures
abort the scan. It never follows image links or downloads assets. There is no
background crawler or automatic publication schedule.

The generated file has `reviewedBy: null`. Review the candidate names, full dates,
product-versus-set classification, source URLs and current usage terms. Remove
uncertain candidates, then set `reviewedBy` to a truthful reviewer identity. Raw HTML
is not committed or exported; only the fact fields and page digests are retained.

## Preview and publish

```sh
cargo run -p cardlattice-api -- import-pokemon-sets path/to/reviewed.json --retire-demo
cargo run -p cardlattice-api -- import-pokemon-sets path/to/reviewed.json --apply --retire-demo
```

Without `--apply`, all mutations are validated in a transaction and rolled back.
Application requires a reviewed batch. Internally generated UUIDs are canonical;
official page URLs are secondary external identifiers for idempotent updates.
Existing independently recorded attributes and series links are preserved. A
conflicting or retired source identity aborts the whole transaction for review.

`--retire-demo` promotes an existing synthetic Pokémon adapter record to a real
game and retires all remaining active records marked `synthetic`, children first.
It also rejects pending synthetic contributions with a recorded decision. It does
not physically erase provenance, prior approvals, private scans, revisions or old
dataset files. The daily-release migration withdraws historical demo manifests
from public listings and downloads while retaining private audit evidence. If any non-demo
record depends on a demo parent, the whole operation rolls back for manual repair.

Back up PostgreSQL before retiring existing records (see `docs/operations.md`).
Every change, revision and invalidation event commits atomically. The invalidation
worker immediately purges affected API URLs; a separate daily release check exports
non-synthetic records. Downloads may lag until the next UTC day. Cloudflare remains
optional. Reimporting identical facts produces no new
records, revisions or source duplicates. `seed` also skips tombstoned fixture IDs,
so a later demo seed does not resurrect retired fixtures.

Do not run the fixture-based browser suite against a real catalogue. CI and
integration tests use disposable databases with original synthetic fixtures.
