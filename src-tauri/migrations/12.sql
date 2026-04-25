DROP INDEX IF EXISTS idx_sources_ext;

CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_ext
    ON sources(account_id, source_type, telegram_source_kind, external_id);
