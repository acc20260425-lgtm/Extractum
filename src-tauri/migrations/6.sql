CREATE TABLE IF NOT EXISTS analysis_source_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS analysis_source_group_members (
    group_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (group_id, source_id),
    FOREIGN KEY (group_id) REFERENCES analysis_source_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_analysis_source_groups_updated_at
    ON analysis_source_groups(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_analysis_source_group_members_source_id
    ON analysis_source_group_members(source_id);
