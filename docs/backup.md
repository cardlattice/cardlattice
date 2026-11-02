# Backup and recovery

Back up PostgreSQL and the dataset volume. The database includes private scans,
provenance, contributions, review decisions, revisions, outbox state and manifests;
protect backups as private data. Keep credentials outside backup bundles or encrypt
them separately. Never place a database dump in a public dataset directory.

## Local example

```sh
mkdir -p .local/backups
docker compose exec -T db pg_dump -U cardlattice -d cardlattice -Fc > .local/backups/catalogue.dump
```

PostgreSQL's local container connection works as its configured database role.
For hosted PostgreSQL use your secret-managed connection and provider's supported
backup tooling. Verify exit status and test `pg_restore --list` on the result.

Snapshot files must remain available for their manifests. Copy the mounted dataset
volume to durable backup storage. A database-consistent backup plus an artifact
backup taken after it is sufficient when artifacts are append-only. Freeze writes
and stop the publisher during a coordinated first-release recovery rehearsal.

## Recovery rehearsal

1. Restore into an isolated PostgreSQL instance; never overwrite the live database
   without an approved recovery operation.
2. Inspect the dump and restore using `pg_restore --exit-on-error`. Apply only the
   migrations appropriate to the restored application version.
3. Restore snapshot artifacts into the configured dataset directory and ownership.
   For every manifest, verify the file exists, its SHA-256, byte size and line count.
4. Start the API privately. Check `/readyz`, record counts, provenance, review
   history, a private scan with authorised access, and anonymous scan denial.
5. Inspect outbox state. Crash-interrupted claims roll back; failed events can be
   manually replayed. Purge all affected public URLs (or the zone cache during an
   approved disaster recovery) before returning traffic to the restored revision.
6. Run the smoke and integration checks against the restored service, then record
   actual restore time and data-loss window. This repository does not claim an RPO
   or RTO before those measurements exist.

The publisher currently reuses a manifest when that revision exists. A missing
artifact must be recovered from backup; it is not silently rebuilt under an existing
immutable filename. Periodically reconcile manifests and files. Do not regenerate
old revisions from the current catalogue and label them as historical snapshots.

Production should use encrypted off-host backups, tested retention, and PostgreSQL
WAL archiving/PITR when required. Retention of review scans and personal data must
be documented separately from metadata audit retention.
