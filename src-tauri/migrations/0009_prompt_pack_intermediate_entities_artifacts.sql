ALTER TABLE prompt_pack_stage_artifacts
RENAME TO prompt_pack_stage_artifacts_old;

CREATE TABLE prompt_pack_stage_artifacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    stage_run_id INTEGER NOT NULL,
    artifact_kind TEXT NOT NULL CHECK (
        artifact_kind IN ('prompt_input', 'raw_output', 'parsed_output', 'metrics', 'error', 'repair_input', 'intermediate_entities')
    ),
    attempt_number INTEGER NOT NULL,
    artifact_index INTEGER NOT NULL,
    content_type TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    content_zstd BLOB NOT NULL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    redaction_state TEXT NOT NULL DEFAULT 'none',
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (stage_run_id, run_id) REFERENCES prompt_pack_stage_runs(id, run_id) ON DELETE CASCADE,
    UNIQUE(id, run_id),
    UNIQUE(stage_run_id, artifact_kind, attempt_number, artifact_index)
);

INSERT INTO prompt_pack_stage_artifacts (
    id,
    run_id,
    stage_run_id,
    artifact_kind,
    attempt_number,
    artifact_index,
    content_type,
    content_hash,
    content_zstd,
    input_tokens,
    output_tokens,
    redaction_state,
    created_at
)
SELECT
    id,
    run_id,
    stage_run_id,
    artifact_kind,
    attempt_number,
    artifact_index,
    content_type,
    content_hash,
    content_zstd,
    input_tokens,
    output_tokens,
    redaction_state,
    created_at
FROM prompt_pack_stage_artifacts_old;

DROP TABLE prompt_pack_stage_artifacts_old;
