CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL COLLATE NOCASE,
    description TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_name_unique
ON projects(name COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS project_sources (
    project_id INTEGER NOT NULL
        REFERENCES projects(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL
        REFERENCES sources(id) ON DELETE RESTRICT,
    added_at INTEGER NOT NULL,
    PRIMARY KEY (project_id, source_id)
);

CREATE INDEX IF NOT EXISTS idx_project_sources_source_id
ON project_sources(source_id);

CREATE INDEX IF NOT EXISTS idx_project_sources_project_id_added_at
ON project_sources(project_id, added_at DESC, source_id DESC);

ALTER TABLE analysis_runs ADD COLUMN project_id INTEGER
    REFERENCES projects(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_analysis_runs_project_id_created_at
ON analysis_runs(project_id, created_at DESC);
