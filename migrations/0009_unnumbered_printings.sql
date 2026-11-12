-- Some official printings (for example basic Energy cards) have no collector number.
-- Keep a semantic filename key without presenting it as a printed number.
ALTER TABLE catalogue_records DROP CONSTRAINT catalogue_records_check1;
ALTER TABLE catalogue_records ADD CONSTRAINT catalogue_printing_identity CHECK (
    (kind = 'printing' AND set_id IS NOT NULL AND card_id IS NOT NULL AND language IS NOT NULL
        AND (
            (collector_number IS NOT NULL AND NOT (attributes ? 'unnumberedKey'))
            OR (collector_number IS NULL AND COALESCE(
                jsonb_typeof(attributes->'unnumberedKey') = 'string'
                AND attributes->>'unnumberedKey' ~ '^[A-Za-z0-9_.-]{1,100}$'
                AND attributes->>'unnumberedKey' NOT IN ('.','..'), false))
        ))
    OR (kind <> 'printing' AND set_id IS NULL AND card_id IS NULL AND collector_number IS NULL
        AND NOT (attributes ? 'unnumberedKey'))
);
