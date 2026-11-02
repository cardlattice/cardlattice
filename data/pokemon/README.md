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

## Numbered card checklists (2026-11-02)

`cards-reviewed-2026-11-02.json` adds independently transcribed names and collector
numbers from eight publisher-hosted English PDF checklists. The PDF source URLs,
retrieval timestamps and SHA-256 digests are in `checklist-plan.json` and every
imported record's provenance. No third-party catalogue records, prices, images,
marketing prose or rules text are imported. Search/editorial pages were used only
to discover some outbound links to the publisher's PDFs, never as card-data inputs.

| Set | Numbered entries transcribed |
| --- | ---: |
| Mega Evolution | 188 |
| Phantasmal Flames | 130 |
| Ascended Heroes | 295 |
| Perfect Order | 124 |
| Chaos Rising | 122 |
| Pitch Black | 120 |
| 30th Celebration | 158 |
| Delta Reign | 141 |

These 1,278 entries cover each selected checklist's consecutive main numbered
section, including its numbered alternate illustrations. They **do not establish
complete set coverage**: promotional products, languages, regional differences,
finishes and other variants remain unverified. The 30th Celebration checklist also
lists Classic Collection and separate Basic Energy sections with overlapping or
original numbers; those sections are intentionally excluded pending identity review.
Delta Reign remains an announced future release as of the retrieval date.

Rarity, category, HP, attacks, rules, evolution and artwork remain unknown. Energy
symbols in names were visually transcribed into accessible words (for example,
“Growing Grass Energy”); Nidoran♀ spacing was corrected. Exact extraction-to-review
changes are recorded in `checklist-transcription-corrections.json`. Wrapped names,
number continuity, source headings and anomalous text were reviewed. This is an
operator-requested automated review, not a claim of expert human verification.

Every entry creates a distinct printing and a **provisional** card identity.
`attributes.identityScope` contains its internally generated printing UUID. Names
alone cannot establish rules equivalence, so same-name entries are never merged by
this importer. Later moderator-reviewed comparisons can link equivalent printings.
Unknown typed Pokémon categories are nullable; known categories still validate.

The source PDFs contain copyrighted layout, logos and other content. They are not
committed, served or included in datasets. Public availability is not a redistribution
license. General Pokémon terms are linked for transparency, not treated as blanket
permission to copy content; the main site's access challenge was not bypassed.
Only independently structured factual metadata is dedicated under CC0. This batch
does not authorize a recursive crawler or artwork collection.

### Reproduce and review

Fetch the explicitly listed public PDF URLs using an ordinary HTTP client, allowing
at least five seconds between requests. Save each privately as
`<contentSha256>.pdf`. Stop on challenges, redirects, changed checksums or access
restrictions; do not try to bypass them. Python 3 and Poppler are optional offline
review tools, not application dependencies:

```sh
python3 scripts/pokemon/extract_checklists.py \
  data/pokemon/checklist-plan.json .local/pokemon-cards .local/card-candidates.json
```

The tool verifies PDF digests and consecutive numbers and produces `reviewedBy:
null`. Review the PDFs visually, including multiline names and non-text energy
symbols, and review source usage policies before setting a truthful reviewer.
`checklist-transcription-corrections.json` documents this batch's visual corrections;
it is not automatically applied to future PDFs. Poppler reports “Invalid Font Weight”
for the 30th Celebration source's font metadata; its names and main numbered section
were separately checked. No software quality-check warnings are waived.

```sh
cargo run -p cardlattice-api -- import-pokemon-cards data/pokemon/cards-reviewed-2026-11-02.json
cargo run -p cardlattice-api -- import-pokemon-cards data/pokemon/cards-reviewed-2026-11-02.json --apply
```

Import the reviewed sets first. Back up the database before applying a new batch.
Dry runs roll back all data, sources, audit rows and events (PostgreSQL sequences can
still advance). The importer is deliberately insert-only: identical printings are
skipped, while conflicting identities, changed names, tombstones and manual duplicate
printings abort the whole transaction for correction review. Existing community
metadata is never overwritten. There is no network access during import.

Each inserted record keeps its own revision and source provenance. All invalidation
paths for the import are deduplicated into one event **in the same transaction**;
parent set/card URLs and collection URLs are included. Existing purge batching,
backoff and manual replay apply. Arbitrary filtered/search pages retain bounded TTLs.
New records appear immediately after commit; immutable dataset releases still occur
at most once per UTC day. Importing cards does not replace today's snapshot.
