ALTER TABLE analysis_runs
ADD COLUMN telegram_history_scope TEXT
CHECK (
    telegram_history_scope IS NULL
    OR telegram_history_scope IN ('current', 'current_plus_migrated')
);
