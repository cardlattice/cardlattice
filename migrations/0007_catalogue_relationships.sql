-- Catalogue groups and illustrations are data records, not inventory or image assets.
ALTER TABLE catalogue_records DROP CONSTRAINT catalogue_records_kind_check;
ALTER TABLE catalogue_records ADD CONSTRAINT catalogue_records_kind_check
    CHECK (kind IN ('game','series','set','card','printing','product','collection','artwork'));

CREATE TABLE catalogue_links (
    record_id text NOT NULL REFERENCES catalogue_records(id) ON DELETE CASCADE,
    target_id text NOT NULL REFERENCES catalogue_records(id) DEFERRABLE INITIALLY DEFERRED,
    relation text NOT NULL CHECK (relation IN ('products','collections','artwork','relatedSets')),
    PRIMARY KEY(record_id, relation, target_id)
);
CREATE INDEX catalogue_links_target ON catalogue_links(target_id, relation, record_id);

-- Rebuild just the changed record's edges; JSON remains the public representation.
CREATE FUNCTION sync_catalogue_links() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM catalogue_links WHERE record_id=NEW.id;
    IF NEW.deleted_at IS NULL THEN
        IF NEW.kind='printing' THEN
            INSERT INTO catalogue_links(record_id,relation,target_id)
            SELECT NEW.id, 'products', value FROM jsonb_array_elements_text(COALESCE(NEW.attributes->'products','[]'))
            UNION ALL SELECT NEW.id, 'collections', value FROM jsonb_array_elements_text(COALESCE(NEW.attributes->'collections','[]'));
            IF NEW.attributes ? 'artwork' THEN
                INSERT INTO catalogue_links VALUES(NEW.id,NEW.attributes->>'artwork','artwork');
            END IF;
        ELSIF NEW.kind IN ('product','collection') THEN
            INSERT INTO catalogue_links(record_id,relation,target_id)
            SELECT NEW.id,'relatedSets',value FROM jsonb_array_elements_text(COALESCE(NEW.attributes->'relatedSets','[]'));
        END IF;
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER sync_catalogue_links AFTER INSERT OR UPDATE OF attributes,kind,game_id,deleted_at
    ON catalogue_records FOR EACH ROW EXECUTE FUNCTION sync_catalogue_links();

-- Deferred checks allow atomic rewiring and retiring a complete linked bundle.
-- They still reject dangling references from locally maintained records.
CREATE FUNCTION enforce_catalogue_links() RETURNS trigger LANGUAGE plpgsql AS $$
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
          OR (l.relation<>'relatedSets' AND r.kind<>'printing'))
    ) THEN
        RAISE EXCEPTION 'Relationships require active targets of the correct kind and game' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
CREATE CONSTRAINT TRIGGER valid_catalogue_links AFTER INSERT OR UPDATE
    ON catalogue_records DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION enforce_catalogue_links();
CREATE INDEX catalogue_set_marks_search ON catalogue_records USING gin
    (to_tsvector('simple', COALESCE(attributes->'printedIdentifiers','[]')::text))
    WHERE kind='set' AND deleted_at IS NULL;
