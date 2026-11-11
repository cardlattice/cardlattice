# Reviewed Pokémon data moved

The authoritative catalogue facts now live in
[cardlattice-data](https://github.com/cardlattice/cardlattice-data).
Correct individual card YAML files or separate set files through a PR. Its CI compiles
catalogues; the application consumes the resulting artifacts.

See [the data pipeline](../../docs/data-pipeline.md) for validation and indexing.
The bounded `scan-pokemon-sets`, `import-pokemon-sets` and `import-pokemon-cards`
commands remain available for preparing or testing reviewed official-source data.
They do not ingest third-party catalogue dumps. Do not use them to bypass the data
repository's review process in a deployed catalogue.
