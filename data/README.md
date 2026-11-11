# Catalogue data

Reviewed real catalogue data lives in [cardlattice-data](https://github.com/cardlattice/cardlattice-data).
See [the CI pipeline](../docs/data-pipeline.md) for building and indexing it.

## Original development fixtures

`development.json` is original, synthetic test data authored for this project.
The real game name Pokémon identifies the supported validation adapter only.
Every series, set, card, collector number and printing in this file is invented
and explicitly marked `synthetic: true`. These are not real Pokémon products.
No artwork, provider IDs, catalogue dumps or API responses are included.

The factual metadata in these fixtures is dedicated to the public domain under
CC0-1.0. Fixture IDs are stable internal semantic identifiers. `cardlattice-api seed` is explicit
and idempotent; migrations never insert development data. Compose only exposes the
seed service through the opt-in `demo` profile. Keep these fixtures in disposable
test databases; they are not the default running catalogue.
