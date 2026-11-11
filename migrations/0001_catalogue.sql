-- A common identity table preserves cross-game semantics; typed views are read models.
CREATE TABLE catalogue_records (
    id text PRIMARY KEY CHECK (length(id) BETWEEN 1 AND 200 AND id ~ '^[A-Za-z0-9_.-]+$'),
    kind text NOT NULL CHECK (kind IN ('game','series','set','card','printing')),
    game_id text REFERENCES catalogue_records(id),
    slug text,
    name text NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
    normalized_name text NOT NULL,
    series_id text REFERENCES catalogue_records(id),
    set_id text REFERENCES catalogue_records(id),
    card_id text REFERENCES catalogue_records(id),
    collector_number text,
    language text,
    region text,
    rarity text,
    release_date date,
    attributes jsonb NOT NULL DEFAULT '{}' CHECK (jsonb_typeof(attributes) = 'object'),
    synthetic boolean NOT NULL DEFAULT false,
    duplicate_key text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    deleted_at timestamptz,
    search_document tsvector GENERATED ALWAYS AS (to_tsvector('simple', normalized_name)) STORED,
    CHECK ((kind = 'game' AND game_id IS NULL AND slug IS NOT NULL) OR (kind <> 'game' AND game_id IS NOT NULL AND slug IS NULL)),
    CHECK ((kind = 'printing' AND set_id IS NOT NULL AND card_id IS NOT NULL AND collector_number IS NOT NULL AND language IS NOT NULL) OR (kind <> 'printing' AND set_id IS NULL AND card_id IS NULL AND collector_number IS NULL)),
    CHECK (series_id IS NULL OR kind = 'set')
);
CREATE UNIQUE INDEX catalogue_slug ON catalogue_records(slug) WHERE kind = 'game' AND deleted_at IS NULL;
CREATE UNIQUE INDEX catalogue_duplicate ON catalogue_records(duplicate_key) WHERE deleted_at IS NULL;
CREATE INDEX catalogue_name ON catalogue_records(kind, game_id, normalized_name, id) WHERE deleted_at IS NULL;
CREATE INDEX catalogue_newest ON catalogue_records(kind, game_id, created_at, id) WHERE deleted_at IS NULL;
CREATE INDEX catalogue_search ON catalogue_records USING gin(search_document) WHERE deleted_at IS NULL;
CREATE INDEX printing_set ON catalogue_records(set_id, language, id) WHERE kind = 'printing' AND deleted_at IS NULL;
CREATE INDEX printing_card ON catalogue_records(card_id, id) WHERE kind = 'printing' AND deleted_at IS NULL;
CREATE INDEX printing_rarity ON catalogue_records(game_id, rarity, id) WHERE kind = 'printing' AND deleted_at IS NULL;

CREATE FUNCTION enforce_catalogue_relations() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'UPDATE' AND (NEW.kind IS DISTINCT FROM OLD.kind OR NEW.game_id IS DISTINCT FROM OLD.game_id OR NEW.slug IS DISTINCT FROM OLD.slug) THEN
        RAISE EXCEPTION 'Record kind, game and slug are immutable' USING ERRCODE = '23514';
    END IF;
    IF NEW.deleted_at IS NOT NULL AND EXISTS (SELECT 1 FROM catalogue_records WHERE deleted_at IS NULL AND (game_id = NEW.id OR series_id = NEW.id OR set_id = NEW.id OR card_id = NEW.id)) THEN
        RAISE EXCEPTION 'Cannot remove a record with active dependents' USING ERRCODE = '23514';
    END IF;
    IF NEW.game_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM catalogue_records WHERE id = NEW.game_id AND kind = 'game' AND deleted_at IS NULL) THEN
        RAISE EXCEPTION 'Parent game does not exist' USING ERRCODE = '23514';
    END IF;
    IF NEW.series_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM catalogue_records WHERE id = NEW.series_id AND kind = 'series' AND game_id = NEW.game_id AND deleted_at IS NULL) THEN
        RAISE EXCEPTION 'Series must belong to the same game' USING ERRCODE = '23514';
    END IF;
    IF NEW.kind = 'printing' AND (NOT EXISTS (SELECT 1 FROM catalogue_records WHERE id = NEW.card_id AND kind = 'card' AND game_id = NEW.game_id AND deleted_at IS NULL) OR NOT EXISTS (SELECT 1 FROM catalogue_records WHERE id = NEW.set_id AND kind = 'set' AND game_id = NEW.game_id AND deleted_at IS NULL)) THEN
        RAISE EXCEPTION 'Printing card and set must belong to the same game' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER catalogue_relations BEFORE INSERT OR UPDATE ON catalogue_records FOR EACH ROW EXECUTE FUNCTION enforce_catalogue_relations();

CREATE VIEW games AS SELECT * FROM catalogue_records WHERE kind = 'game' AND deleted_at IS NULL;
CREATE VIEW series AS SELECT * FROM catalogue_records WHERE kind = 'series' AND deleted_at IS NULL;
CREATE VIEW sets AS SELECT * FROM catalogue_records WHERE kind = 'set' AND deleted_at IS NULL;
CREATE VIEW card_identities AS SELECT * FROM catalogue_records WHERE kind = 'card' AND deleted_at IS NULL;
CREATE VIEW printings AS SELECT * FROM catalogue_records WHERE kind = 'printing' AND deleted_at IS NULL;

CREATE TABLE printing_variants (
    id text PRIMARY KEY,
    printing_id text NOT NULL REFERENCES catalogue_records(id),
    finish text NOT NULL,
    stamp text NOT NULL DEFAULT '',
    attributes jsonb NOT NULL DEFAULT '{}',
    UNIQUE(printing_id, finish, stamp)
);
CREATE TABLE localisations (
    record_id text NOT NULL REFERENCES catalogue_records(id),
    language text NOT NULL,
    name text NOT NULL,
    normalized_name text NOT NULL,
    text_content text,
    search_document tsvector GENERATED ALWAYS AS (to_tsvector('simple', normalized_name)) STORED,
    PRIMARY KEY(record_id, language)
);
CREATE INDEX localisation_search ON localisations USING gin(search_document);
CREATE TABLE external_identifiers (
    record_id text NOT NULL REFERENCES catalogue_records(id),
    namespace text NOT NULL,
    value text NOT NULL,
    PRIMARY KEY(record_id, namespace, value),
    UNIQUE(namespace, value)
);
CREATE TABLE sources (
    id text PRIMARY KEY,
    description text NOT NULL,
    reference_url text,
    method text NOT NULL,
    license text NOT NULL,
    recorded_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE provenance (
    record_id text NOT NULL REFERENCES catalogue_records(id),
    source_id text NOT NULL REFERENCES sources(id),
    field_path text NOT NULL DEFAULT '$',
    PRIMARY KEY(record_id, source_id, field_path)
);
CREATE TABLE contributions (
    id text PRIMARY KEY,
    owner_hash text NOT NULL,
    idempotency_key text NOT NULL,
    request_hash text NOT NULL,
    operation text NOT NULL CHECK(operation IN ('create','update','delete')),
    target_id text REFERENCES catalogue_records(id),
    base_updated_at timestamptz,
    payload jsonb NOT NULL,
    duplicate_key text NOT NULL,
    status text NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','approved','rejected')),
    record_id text REFERENCES catalogue_records(id),
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    reviewed_at timestamptz,
    UNIQUE(owner_hash, idempotency_key),
    CHECK ((operation = 'create' AND target_id IS NULL) OR (operation IN ('update','delete') AND target_id IS NOT NULL AND base_updated_at IS NOT NULL))
);
CREATE UNIQUE INDEX pending_duplicate ON contributions(duplicate_key) WHERE status = 'pending';
CREATE INDEX contribution_queue ON contributions(created_at, id) WHERE status = 'pending';
CREATE TABLE review_decisions (
    id text PRIMARY KEY,
    contribution_id text NOT NULL REFERENCES contributions(id),
    reviewer_hash text NOT NULL,
    decision text NOT NULL CHECK(decision IN ('approve','reject')),
    reason text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE scan_uploads (
    id text PRIMARY KEY,
    contribution_id text NOT NULL REFERENCES contributions(id),
    side text NOT NULL CHECK(side IN ('front','back')),
    media_type text NOT NULL,
    sha256 text NOT NULL,
    original bytea NOT NULL CHECK(octet_length(original) <= 8388608),
    candidate_metadata jsonb NOT NULL DEFAULT '{}',
    rights_status text NOT NULL DEFAULT 'private_review_only' CHECK(rights_status IN ('private_review_only','permission_documented','rejected')),
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    UNIQUE(contribution_id, side),
    UNIQUE(contribution_id, sha256)
);
CREATE TABLE image_rights (
    id text PRIMARY KEY,
    source_id text NOT NULL REFERENCES sources(id),
    license text NOT NULL,
    permission_evidence text NOT NULL,
    attribution text,
    approved_by text NOT NULL,
    approved_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE images (
    id text PRIMARY KEY,
    printing_id text NOT NULL REFERENCES catalogue_records(id),
    rights_id text NOT NULL REFERENCES image_rights(id),
    public_url text NOT NULL CHECK(public_url LIKE 'https://%'),
    side text NOT NULL CHECK(side IN ('front','back')),
    sha256 text NOT NULL
);
CREATE TABLE catalogue_revisions (
    revision bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    record_id text NOT NULL REFERENCES catalogue_records(id),
    contribution_id text REFERENCES contributions(id),
    before_data jsonb,
    after_data jsonb,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE cache_invalidation_outbox (
    id text PRIMARY KEY,
    revision bigint NOT NULL REFERENCES catalogue_revisions(revision),
    paths jsonb NOT NULL,
    status text NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','processing','completed','failed')),
    attempts integer NOT NULL DEFAULT 0,
    next_attempt_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    last_error text,
    completed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX outbox_pending ON cache_invalidation_outbox(next_attempt_at) WHERE status = 'pending';
CREATE TABLE publication_manifests (
    id text PRIMARY KEY,
    revision bigint NOT NULL UNIQUE,
    schema_version integer NOT NULL DEFAULT 1,
    sha256 text NOT NULL,
    byte_size bigint NOT NULL,
    record_count bigint NOT NULL,
    filename text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE health_measurements (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    healthy boolean NOT NULL,
    database_latency_ms double precision NOT NULL,
    measured_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX health_recent ON health_measurements(measured_at DESC);
CREATE TABLE rate_limits (
    subject text NOT NULL,
    bucket timestamptz NOT NULL,
    hits integer NOT NULL,
    PRIMARY KEY(subject, bucket)
);
