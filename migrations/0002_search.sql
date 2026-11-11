CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX catalogue_substring_search ON catalogue_records USING gin(normalized_name gin_trgm_ops) WHERE deleted_at IS NULL;
CREATE INDEX localisation_substring_search ON localisations USING gin(normalized_name gin_trgm_ops);
CREATE INDEX catalogue_name_c ON catalogue_records(kind,game_id,normalized_name COLLATE "C",id) WHERE deleted_at IS NULL;
