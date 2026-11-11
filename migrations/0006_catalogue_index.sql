-- CI-built data is authoritative for these records only. Community drafts and
-- private review material are never included in, or deleted by, the data artifact.
CREATE TABLE catalogue_index_records (
    record_id text PRIMARY KEY REFERENCES catalogue_records(id),
    indexed_record jsonb NOT NULL,
    source_commit text NOT NULL CHECK (source_commit ~ '^[0-9a-fA-F]{40}$')
);
