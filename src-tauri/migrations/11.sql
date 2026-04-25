ALTER TABLE sources ADD COLUMN telegram_source_kind TEXT NOT NULL DEFAULT 'channel';

UPDATE sources
SET telegram_source_kind = 'channel'
WHERE source_type = 'telegram_channel';

UPDATE sources
SET source_type = 'telegram'
WHERE source_type = 'telegram_channel';

DROP INDEX IF EXISTS idx_sources_ext;

CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_ext
    ON sources(source_type, telegram_source_kind, external_id);
