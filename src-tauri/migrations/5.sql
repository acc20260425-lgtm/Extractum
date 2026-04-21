-- Add analysis storage for prompt templates and saved report runs

CREATE TABLE IF NOT EXISTS analysis_prompt_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    template_kind TEXT NOT NULL,
    body TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_builtin BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_analysis_prompt_templates_kind_name
ON analysis_prompt_templates(template_kind, name);

CREATE TABLE IF NOT EXISTS analysis_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_type TEXT NOT NULL,
    scope_type TEXT NOT NULL,
    source_id INTEGER,
    period_from INTEGER NOT NULL,
    period_to INTEGER NOT NULL,
    output_language TEXT NOT NULL,
    prompt_template_id INTEGER,
    prompt_template_version INTEGER NOT NULL,
    provider_profile TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    status TEXT NOT NULL,
    result_markdown TEXT,
    trace_data_zstd BLOB,
    error TEXT,
    created_at INTEGER NOT NULL,
    completed_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_analysis_runs_source_created
ON analysis_runs(source_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_analysis_runs_status_created
ON analysis_runs(status, created_at DESC);
