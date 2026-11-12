-- Explicit clearing, set metadata and observed variant relationships.
CREATE OR REPLACE FUNCTION sync_catalogue_links() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM catalogue_links WHERE record_id=NEW.id;
    IF NEW.deleted_at IS NULL THEN
        IF NEW.kind IN ('printing','set') THEN
            INSERT INTO catalogue_links(record_id,relation,target_id)
            SELECT NEW.id, 'products', value FROM jsonb_array_elements_text(COALESCE(NULLIF(NEW.attributes->'products','null'),'[]'))
            UNION ALL SELECT NEW.id, 'collections', value FROM jsonb_array_elements_text(COALESCE(NULLIF(NEW.attributes->'collections','null'),'[]'));
            INSERT INTO catalogue_links(record_id,relation,target_id)
            SELECT DISTINCT NEW.id, 'products', product
            FROM jsonb_each(COALESCE(NULLIF(NEW.attributes->'variants','null'),'{}')) variant,
                 jsonb_array_elements_text(COALESCE(variant.value->'products','[]')) product
            ON CONFLICT DO NOTHING;
            IF NEW.attributes->>'artwork' IS NOT NULL THEN
                INSERT INTO catalogue_links VALUES(NEW.id,NEW.attributes->>'artwork','artwork');
            END IF;
        ELSIF NEW.kind IN ('product','collection') THEN
            INSERT INTO catalogue_links(record_id,relation,target_id)
            SELECT NEW.id,'relatedSets',value FROM jsonb_array_elements_text(COALESCE(NEW.attributes->'relatedSets','[]'));
        END IF;
    END IF;
    RETURN NEW;
END $$;

CREATE OR REPLACE FUNCTION enforce_catalogue_links() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM catalogue_links l
        JOIN catalogue_records r ON r.id=l.record_id
        JOIN catalogue_records t ON t.id=l.target_id
        WHERE (l.record_id=NEW.id OR l.target_id=NEW.id)
        AND (r.deleted_at IS NOT NULL OR t.deleted_at IS NOT NULL
          OR r.game_id IS DISTINCT FROM t.game_id
          OR t.kind <> CASE l.relation WHEN 'products' THEN 'product'
                WHEN 'collections' THEN 'collection' WHEN 'artwork' THEN 'artwork' ELSE 'set' END
          OR (l.relation='relatedSets' AND r.kind NOT IN ('product','collection'))
          OR (l.relation<>'relatedSets' AND r.kind NOT IN ('printing','set')))
    ) THEN
        RAISE EXCEPTION 'Relationships require active targets of the correct kind and game' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;

-- Rebuild existing edges through the trigger.
UPDATE catalogue_records SET attributes=attributes WHERE kind IN ('printing','set');

