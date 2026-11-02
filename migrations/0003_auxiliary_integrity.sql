CREATE FUNCTION enforce_printing_reference() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS(SELECT 1 FROM catalogue_records WHERE id=NEW.printing_id AND kind='printing' AND deleted_at IS NULL) THEN
        RAISE EXCEPTION 'A printing variant or image must reference an active printing' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER variant_printing BEFORE INSERT OR UPDATE ON printing_variants FOR EACH ROW EXECUTE FUNCTION enforce_printing_reference();
CREATE TRIGGER image_printing BEFORE INSERT OR UPDATE ON images FOR EACH ROW EXECUTE FUNCTION enforce_printing_reference();
