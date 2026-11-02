-- Release dates are public versions; revision remains an internal audit watermark.
ALTER TABLE publication_manifests
    ADD COLUMN release_date date,
    ADD COLUMN withdrawn_at timestamptz,
    ADD COLUMN withdrawal_reason text;
UPDATE publication_manifests SET release_date=(created_at AT TIME ZONE 'UTC')::date;
ALTER TABLE publication_manifests ALTER COLUMN release_date SET NOT NULL;

-- Reconstruct the historical snapshot state, not today's synthetic flags.
UPDATE publication_manifests m
SET withdrawn_at=clock_timestamp(), withdrawal_reason='Contains synthetic development records'
WHERE EXISTS (
    SELECT 1 FROM (
        SELECT DISTINCT ON (record_id) after_data
        FROM catalogue_revisions WHERE revision<=m.revision
        ORDER BY record_id, revision DESC
    ) historical
    WHERE after_data->>'synthetic'='true' AND after_data->>'deleted_at' IS NULL
);
-- Collapse any legacy same-day releases; preserve their manifests for audit.
UPDATE publication_manifests m
SET withdrawn_at=clock_timestamp(), withdrawal_reason='Superseded legacy same-day release'
WHERE withdrawn_at IS NULL AND EXISTS (
    SELECT 1 FROM publication_manifests newer
    WHERE newer.withdrawn_at IS NULL AND newer.release_date=m.release_date
      AND newer.revision>m.revision
);
CREATE UNIQUE INDEX one_public_release_per_day ON publication_manifests(release_date)
    WHERE withdrawn_at IS NULL;

-- Withdrawals also invalidate any configured CDN; use the existing durable retry path.
INSERT INTO cache_invalidation_outbox(id,revision,paths)
SELECT gen_random_uuid(),m.revision,
       jsonb_build_array('/datasets/'||m.filename,'/v1/datasets','/v1/status')
FROM publication_manifests m JOIN catalogue_revisions r ON r.revision=m.revision
WHERE m.withdrawn_at IS NOT NULL;
