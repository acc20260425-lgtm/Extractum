ALTER TABLE analysis_runs ADD COLUMN scope_label_snapshot TEXT;

CREATE TABLE IF NOT EXISTS analysis_run_messages (
    run_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    author TEXT,
    published_at INTEGER NOT NULL,
    ref TEXT NOT NULL,
    content_zstd BLOB NOT NULL,
    PRIMARY KEY (run_id, ref),
    FOREIGN KEY (run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_analysis_run_messages_run_published
    ON analysis_run_messages(run_id, published_at ASC, ref ASC);

CREATE INDEX IF NOT EXISTS idx_analysis_run_messages_run_source
    ON analysis_run_messages(run_id, source_id);
