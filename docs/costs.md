# Operating cost model

No vendor plan or price is assumed here. Prices and free allowances change; obtain
current quotes before committing. Core operation needs a modest host, PostgreSQL,
disk, backups, and a domain. Cloudflare and object storage are optional.

Let `R` be monthly requests, `H` the **measured** edge hit fraction, `B` average
response bytes, and `W` monthly accepted revisions. Then:

```text
origin requests ≈ R × (1 − H)
origin response transfer ≈ R × (1 − H) × B
total public transfer ≈ R × B
snapshot storage ≈ retained snapshot count × average snapshot bytes
private scan storage ≈ retained scans × average scan bytes
monthly cost = host + database + backup storage + object storage
             + billable transfer + domain amortisation
```

These are planning equations, not measured performance. Query-heavy clients reduce
H because the conservative cache rules bypass query variation. Encourage immutable
bulk datasets for exhaustive traversal. Use a small connection pool, bounded
queries, bounded uploads and rate budgets to contain worst-case origin work.

The first publisher may create a full snapshot after every revision processed;
multiple events can converge on one latest revision. As data grows, coalesce export
work and introduce a deliberate retention schedule. Never delete artifacts whose
URLs are promised to remain available without changing that policy explicitly.

Private scans in PostgreSQL simplify local operation but increase database and
backup size. An optional private object-store adapter and retention enforcement
should precede large-scale scanning. Rights-approved public images and metadata
snapshots can later use R2; do not upload private originals to a public bucket.

Budget alerts should cover disk usage, database growth, origin transfer, pending/
failed invalidations, publication lag, and the age of the last successful backup.
Free API access is a project policy; infrastructure still has operating costs.
