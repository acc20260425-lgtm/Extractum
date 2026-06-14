CREATE TABLE IF NOT EXISTS prompt_packs (
    pack_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    is_builtin INTEGER NOT NULL DEFAULT 1 CHECK (is_builtin IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS prompt_pack_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pack_id TEXT NOT NULL,
    pack_version TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    origin_kind TEXT NOT NULL CHECK (origin_kind IN ('bundled', 'user')),
    lifecycle_status TEXT NOT NULL CHECK (lifecycle_status IN ('draft', 'active', 'archived')),
    content_hash TEXT NOT NULL,
    bundled_source_path TEXT,
    default_control_preset TEXT NOT NULL DEFAULT 'standard',
    default_evidence_mode TEXT NOT NULL DEFAULT 'standard',
    default_include_comments INTEGER NOT NULL DEFAULT 0 CHECK (default_include_comments IN (0, 1)),
    seeded_at INTEGER,
    last_seeded_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (pack_id) REFERENCES prompt_packs(pack_id) ON DELETE CASCADE,
    UNIQUE(pack_id, pack_version),
    UNIQUE(id, pack_id, pack_version, schema_version)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_prompt_pack_versions_one_active
ON prompt_pack_versions(pack_id)
WHERE lifecycle_status = 'active';

CREATE TABLE IF NOT EXISTS prompt_pack_stage_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pack_version_id INTEGER NOT NULL,
    pack_id TEXT NOT NULL,
    pack_version TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    stage_name TEXT NOT NULL,
    stage_order INTEGER NOT NULL,
    provider_family TEXT NOT NULL,
    input_schema_id TEXT NOT NULL,
    output_schema_id TEXT NOT NULL,
    validator_mode TEXT NOT NULL,
    prompt_template_json_zstd BLOB NOT NULL,
    content_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (pack_version_id, pack_id, pack_version, schema_version)
        REFERENCES prompt_pack_versions(id, pack_id, pack_version, schema_version)
        ON DELETE CASCADE,
    UNIQUE(pack_version_id, stage_name)
);

CREATE TABLE IF NOT EXISTS prompt_pack_schema_assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pack_version_id INTEGER NOT NULL,
    pack_id TEXT NOT NULL,
    pack_version TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    schema_id TEXT NOT NULL,
    schema_kind TEXT NOT NULL CHECK (
        schema_kind IN (
            'canonical_result',
            'stage_input',
            'stage_output',
            'pack_data_schema'
        )
    ),
    content_hash TEXT NOT NULL,
    content_json_zstd BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (pack_version_id, pack_id, pack_version, schema_version)
        REFERENCES prompt_pack_versions(id, pack_id, pack_version, schema_version)
        ON DELETE CASCADE,
    UNIQUE(pack_version_id, schema_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER,
    pack_version_id INTEGER NOT NULL,
    pack_id TEXT NOT NULL,
    pack_version TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    run_status TEXT NOT NULL CHECK (
        run_status IN ('queued', 'running', 'complete', 'partial', 'failed', 'cancelled', 'interrupted')
    ),
    result_status TEXT NOT NULL DEFAULT 'none' CHECK (
        result_status IN ('none', 'complete', 'partial', 'failed')
    ),
    request_json_zstd BLOB,
    preflight_json_zstd BLOB,
    provider_profile_id TEXT,
    model TEXT,
    output_language TEXT NOT NULL,
    control_preset TEXT NOT NULL,
    evidence_mode TEXT NOT NULL,
    include_comments INTEGER NOT NULL DEFAULT 0 CHECK (include_comments IN (0, 1)),
    latest_message TEXT,
    queue_position INTEGER,
    progress_current INTEGER,
    progress_total INTEGER,
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL,
    FOREIGN KEY (pack_version_id, pack_id, pack_version, schema_version)
        REFERENCES prompt_pack_versions(id, pack_id, pack_version, schema_version)
);

CREATE TABLE IF NOT EXISTS prompt_pack_run_scopes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    source_type TEXT NOT NULL,
    source_subtype TEXT NOT NULL,
    scope_kind TEXT NOT NULL CHECK (scope_kind IN ('explicit_video', 'playlist')),
    title TEXT,
    metadata_json_zstd BLOB,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE,
    UNIQUE(id, run_id),
    UNIQUE(run_id, source_id, scope_kind)
);

CREATE TABLE IF NOT EXISTS prompt_pack_run_source_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    source_ref_id TEXT NOT NULL,
    video_id TEXT NOT NULL,
    title TEXT,
    channel_title TEXT,
    published_at TEXT,
    url TEXT,
    metadata_json_zstd BLOB,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE,
    UNIQUE(id, run_id),
    UNIQUE(run_id, source_id),
    UNIQUE(run_id, source_ref_id),
    UNIQUE(run_id, video_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_run_source_origins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    origin_scope_id INTEGER NOT NULL,
    source_snapshot_id INTEGER,
    video_source_id INTEGER,
    playlist_item_id INTEGER,
    video_id TEXT,
    inclusion_status TEXT NOT NULL CHECK (inclusion_status IN ('included', 'skipped', 'blocking')),
    reason TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (origin_scope_id, run_id) REFERENCES prompt_pack_run_scopes(id, run_id) ON DELETE CASCADE,
    FOREIGN KEY (source_snapshot_id, run_id) REFERENCES prompt_pack_run_source_snapshots(id, run_id) ON DELETE CASCADE,
    CHECK (inclusion_status <> 'included' OR source_snapshot_id IS NOT NULL),
    UNIQUE(id, run_id),
    UNIQUE(run_id, origin_scope_id, video_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_run_material_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    source_snapshot_id INTEGER NOT NULL,
    material_ref_id TEXT NOT NULL,
    material_kind TEXT NOT NULL CHECK (material_kind IN ('transcript', 'description', 'comment')),
    source_table TEXT,
    source_row_id INTEGER,
    external_id TEXT,
    sequence_index INTEGER NOT NULL DEFAULT 0,
    text_zstd BLOB NOT NULL,
    token_estimate INTEGER NOT NULL DEFAULT 0,
    metadata_json_zstd BLOB,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (source_snapshot_id, run_id) REFERENCES prompt_pack_run_source_snapshots(id, run_id) ON DELETE CASCADE,
    UNIQUE(id, run_id),
    UNIQUE(run_id, material_ref_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_stage_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    source_snapshot_id INTEGER,
    stage_name TEXT NOT NULL,
    stage_order INTEGER NOT NULL,
    stage_status TEXT NOT NULL CHECK (
        stage_status IN ('pending', 'running', 'succeeded', 'failed', 'skipped', 'cancelled', 'not_implemented')
    ),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    latest_message TEXT,
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (source_snapshot_id, run_id) REFERENCES prompt_pack_run_source_snapshots(id, run_id) ON DELETE CASCADE,
    UNIQUE(id, run_id),
    UNIQUE(run_id, stage_name, source_snapshot_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_stage_artifacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    stage_run_id INTEGER NOT NULL,
    artifact_kind TEXT NOT NULL CHECK (
        artifact_kind IN ('prompt_input', 'raw_output', 'parsed_output', 'metrics', 'error', 'repair_input')
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

CREATE TABLE IF NOT EXISTS prompt_pack_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    result_id TEXT NOT NULL,
    result_status TEXT NOT NULL CHECK (result_status IN ('complete', 'partial', 'failed')),
    schema_version TEXT NOT NULL,
    canonical_hash TEXT NOT NULL,
    canonical_json_zstd BLOB NOT NULL,
    projection_updated_at TEXT,
    storage_warning TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    UNIQUE(id, run_id),
    UNIQUE(run_id),
    UNIQUE(result_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_source_refs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    source_ref_id TEXT NOT NULL,
    source_snapshot_id INTEGER NOT NULL,
    title TEXT,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, source_ref_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_claims (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    claim_id TEXT NOT NULL,
    source_ref_id TEXT,
    text TEXT NOT NULL,
    confidence REAL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, claim_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_evidence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    evidence_id TEXT NOT NULL,
    claim_id TEXT,
    material_ref_id TEXT,
    text TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, evidence_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_ref_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    from_ref TEXT NOT NULL,
    to_ref TEXT NOT NULL,
    edge_kind TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_unknowns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    unknown_id TEXT NOT NULL,
    text TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_verification_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    task_id TEXT NOT NULL,
    text TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_warnings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    warning_id TEXT,
    code TEXT,
    message TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_limitations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    limitation_id TEXT,
    message TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_quality_flags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    flag_id TEXT,
    severity TEXT,
    message TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_audit_refs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    audit_ref_id TEXT NOT NULL,
    event_kind TEXT,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_youtube_videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    video_id TEXT NOT NULL,
    source_ref_id TEXT NOT NULL,
    title TEXT,
    summary_text TEXT,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, video_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_youtube_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    video_id TEXT NOT NULL,
    segment_id TEXT NOT NULL,
    start_seconds REAL,
    end_seconds REAL,
    text TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, segment_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_youtube_key_points (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    video_id TEXT NOT NULL,
    key_point_id TEXT NOT NULL,
    text TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, key_point_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_youtube_quotes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    video_id TEXT NOT NULL,
    quote_id TEXT NOT NULL,
    text TEXT NOT NULL,
    speaker TEXT,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, quote_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_youtube_action_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    video_id TEXT NOT NULL,
    action_item_id TEXT NOT NULL,
    text TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, action_item_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_youtube_open_questions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    video_id TEXT NOT NULL,
    open_question_id TEXT NOT NULL,
    text TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, open_question_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_youtube_synthesis_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_row_id INTEGER NOT NULL,
    run_id INTEGER NOT NULL,
    synthesis_id TEXT NOT NULL,
    text TEXT NOT NULL,
    FOREIGN KEY (result_row_id, run_id) REFERENCES prompt_pack_results(id, run_id) ON DELETE CASCADE,
    UNIQUE(result_row_id, synthesis_id)
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_validation_findings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    stage_run_id INTEGER,
    severity TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'error')),
    code TEXT NOT NULL,
    message TEXT NOT NULL,
    object_path TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (stage_run_id, run_id) REFERENCES prompt_pack_stage_runs(id, run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_audit_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    event_kind TEXT NOT NULL,
    message TEXT,
    payload_json_zstd BLOB,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS prompt_pack_result_quarantine_artifacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    stage_run_id INTEGER,
    object_path TEXT NOT NULL,
    reason TEXT NOT NULL,
    content_json_zstd BLOB NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES prompt_pack_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (stage_run_id, run_id) REFERENCES prompt_pack_stage_runs(id, run_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_prompt_pack_runs_project_created
ON prompt_pack_runs(project_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_prompt_pack_stage_runs_run_order
ON prompt_pack_stage_runs(run_id, stage_order, id);

CREATE INDEX IF NOT EXISTS idx_prompt_pack_results_run
ON prompt_pack_results(run_id);
