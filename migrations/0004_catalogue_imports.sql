-- Operator-run imports retain the reviewed factual batch and every resulting revision.
CREATE TABLE catalogue_imports (
    id text PRIMARY KEY,
    batch_sha256 text NOT NULL CHECK (length(batch_sha256) = 64),
    reviewed_by text NOT NULL,
    source_policy_url text NOT NULL,
    batch jsonb NOT NULL,
    summary jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE catalogue_import_revisions (
    import_id text NOT NULL REFERENCES catalogue_imports(id),
    revision bigint NOT NULL REFERENCES catalogue_revisions(revision),
    PRIMARY KEY (import_id, revision)
);
