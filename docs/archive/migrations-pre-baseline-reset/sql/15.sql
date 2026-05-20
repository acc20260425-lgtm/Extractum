ALTER TABLE sources ADD COLUMN source_subtype TEXT;

UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype IS NULL;
