ALTER TABLE analysis_runs
    ADD COLUMN source_group_id INTEGER;

CREATE INDEX IF NOT EXISTS idx_analysis_runs_source_group_created
    ON analysis_runs(source_group_id, created_at DESC);
