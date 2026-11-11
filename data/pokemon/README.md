# Reviewed Pokémon data moved

The authoritative catalogue and original source evidence now live in
[cardlattice-data](https://github.com/cardlattice/cardlattice-data).
Correct the readable set files there through a reviewed PR. Its CI compiles
catalogues; the application consumes the resulting artifacts.

See [the data pipeline](../../docs/data-pipeline.md) for validation and indexing.
The bounded `scan-pokemon-sets`, `import-pokemon-sets` and `import-pokemon-cards`
commands remain available for preparing or testing reviewed official-source data.
They do not ingest third-party catalogue dumps. Do not use them to bypass the data
repository's review process in a deployed catalogue.
