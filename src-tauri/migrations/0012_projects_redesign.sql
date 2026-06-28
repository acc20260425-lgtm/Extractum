ALTER TABLE projects ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
ALTER TABLE projects ADD COLUMN archived_at INTEGER;

CREATE INDEX IF NOT EXISTS idx_projects_pinned_archived
    ON projects(
        CASE WHEN archived_at IS NULL THEN 0 ELSE 1 END ASC,
        pinned DESC,
        updated_at DESC,
        id DESC
    );
