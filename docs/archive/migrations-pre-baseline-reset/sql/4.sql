-- Add user-facing timestamp for the last successful sync

ALTER TABLE sources ADD COLUMN last_synced_at INTEGER;
