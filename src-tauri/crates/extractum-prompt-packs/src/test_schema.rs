#[cfg(test)]
const PROMPT_PACK_TEST_MIGRATIONS: [(&str, &str); 12] = [
    (
        "src-tauri/migrations/0001_current_schema_baseline.sql",
        include_str!("../../../migrations/0001_current_schema_baseline.sql"),
    ),
    (
        "src-tauri/migrations/0002_migrated_history_opt_in_schema.sql",
        include_str!("../../../migrations/0002_migrated_history_opt_in_schema.sql"),
    ),
    (
        "src-tauri/migrations/0003_analysis_telegram_history_scope.sql",
        include_str!("../../../migrations/0003_analysis_telegram_history_scope.sql"),
    ),
    (
        "src-tauri/migrations/0004_source_delete_cascade_indexes.sql",
        include_str!("../../../migrations/0004_source_delete_cascade_indexes.sql"),
    ),
    (
        "src-tauri/migrations/0005_projects_mvp.sql",
        include_str!("../../../migrations/0005_projects_mvp.sql"),
    ),
    (
        "src-tauri/migrations/0006_prompt_pack_mvp.sql",
        include_str!("../../../migrations/0006_prompt_pack_mvp.sql"),
    ),
    (
        "src-tauri/migrations/0007_prompt_pack_run_idempotency.sql",
        include_str!("../../../migrations/0007_prompt_pack_run_idempotency.sql"),
    ),
    (
        "src-tauri/migrations/0008_prompt_pack_run_labels.sql",
        include_str!("../../../migrations/0008_prompt_pack_run_labels.sql"),
    ),
    (
        "src-tauri/migrations/0009_prompt_pack_intermediate_entities_artifacts.sql",
        include_str!("../../../migrations/0009_prompt_pack_intermediate_entities_artifacts.sql"),
    ),
    (
        "src-tauri/migrations/0010_prompt_pack_runtime_provider.sql",
        include_str!("../../../migrations/0010_prompt_pack_runtime_provider.sql"),
    ),
    (
        "src-tauri/migrations/0011_prompt_pack_stage_browser_provenance.sql",
        include_str!("../../../migrations/0011_prompt_pack_stage_browser_provenance.sql"),
    ),
    (
        "src-tauri/migrations/0012_projects_redesign.sql",
        include_str!("../../../migrations/0012_projects_redesign.sql"),
    ),
];

#[cfg(test)]
pub(crate) async fn prompt_pack_test_pool() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect Prompt Pack test pool");
    let mut transaction = pool.begin().await.expect("begin Prompt Pack test schema");
    for (_, sql) in PROMPT_PACK_TEST_MIGRATIONS {
        sqlx::raw_sql(sql)
            .execute(&mut *transaction)
            .await
            .expect("apply Prompt Pack test migration");
    }
    transaction
        .commit()
        .await
        .expect("commit Prompt Pack test schema");
    pool
}

#[cfg(test)]
mod tests {
    use super::prompt_pack_test_pool;

    const CONSUMED_TABLE_COLUMNS: &[(&str, &[&str])] = &[
        (
            "prompt_packs",
            &[
                "pack_id",
                "display_name",
                "is_builtin",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "prompt_pack_versions",
            &[
                "id",
                "pack_id",
                "pack_version",
                "schema_version",
                "origin_kind",
                "lifecycle_status",
                "content_hash",
                "bundled_source_path",
                "default_control_preset",
                "default_evidence_mode",
                "default_include_comments",
                "seeded_at",
                "last_seeded_at",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "prompt_pack_stage_templates",
            &[
                "id",
                "pack_version_id",
                "pack_id",
                "pack_version",
                "schema_version",
                "stage_name",
                "stage_order",
                "provider_family",
                "input_schema_id",
                "output_schema_id",
                "validator_mode",
                "prompt_template_json_zstd",
                "content_hash",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "prompt_pack_schema_assets",
            &[
                "id",
                "pack_version_id",
                "pack_id",
                "pack_version",
                "schema_version",
                "schema_id",
                "schema_kind",
                "content_hash",
                "content_json_zstd",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "prompt_pack_runs",
            &[
                "id",
                "project_id",
                "pack_version_id",
                "pack_id",
                "pack_version",
                "schema_version",
                "run_status",
                "result_status",
                "request_json_zstd",
                "preflight_json_zstd",
                "provider_profile_id",
                "model",
                "output_language",
                "control_preset",
                "evidence_mode",
                "include_comments",
                "latest_message",
                "queue_position",
                "progress_current",
                "progress_total",
                "created_at",
                "started_at",
                "completed_at",
                "updated_at",
                "client_request_id",
                "run_label",
                "runtime_provider",
                "browser_provider_config_json",
            ],
        ),
        (
            "prompt_pack_run_scopes",
            &[
                "id",
                "run_id",
                "source_id",
                "source_type",
                "source_subtype",
                "scope_kind",
                "title",
                "metadata_json_zstd",
                "created_at",
            ],
        ),
        (
            "prompt_pack_run_source_snapshots",
            &[
                "id",
                "run_id",
                "source_id",
                "source_ref_id",
                "video_id",
                "title",
                "channel_title",
                "published_at",
                "url",
                "metadata_json_zstd",
                "created_at",
            ],
        ),
        (
            "prompt_pack_run_source_origins",
            &[
                "id",
                "run_id",
                "origin_scope_id",
                "source_snapshot_id",
                "video_source_id",
                "playlist_item_id",
                "video_id",
                "inclusion_status",
                "reason",
                "created_at",
            ],
        ),
        (
            "prompt_pack_run_material_snapshots",
            &[
                "id",
                "run_id",
                "source_snapshot_id",
                "material_ref_id",
                "material_kind",
                "source_table",
                "source_row_id",
                "external_id",
                "sequence_index",
                "text_zstd",
                "token_estimate",
                "metadata_json_zstd",
                "created_at",
            ],
        ),
        (
            "prompt_pack_stage_runs",
            &[
                "id",
                "run_id",
                "source_snapshot_id",
                "stage_name",
                "stage_order",
                "stage_status",
                "attempt_count",
                "latest_message",
                "error_message",
                "started_at",
                "completed_at",
                "created_at",
                "updated_at",
                "browser_run_id",
                "browser_run_status",
                "browser_completion_reason",
                "browser_provider_mode",
                "browser_run_message",
            ],
        ),
        (
            "prompt_pack_stage_artifacts",
            &[
                "id",
                "run_id",
                "stage_run_id",
                "artifact_kind",
                "attempt_number",
                "artifact_index",
                "content_type",
                "content_hash",
                "content_zstd",
                "input_tokens",
                "output_tokens",
                "redaction_state",
                "created_at",
            ],
        ),
        (
            "prompt_pack_results",
            &[
                "id",
                "run_id",
                "result_id",
                "result_status",
                "schema_version",
                "canonical_hash",
                "canonical_json_zstd",
                "projection_updated_at",
                "storage_warning",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "prompt_pack_result_source_refs",
            &[
                "id",
                "result_row_id",
                "run_id",
                "source_ref_id",
                "source_snapshot_id",
                "title",
            ],
        ),
        (
            "prompt_pack_result_claims",
            &[
                "id",
                "result_row_id",
                "run_id",
                "claim_id",
                "source_ref_id",
                "text",
                "confidence",
            ],
        ),
        (
            "prompt_pack_result_evidence",
            &[
                "id",
                "result_row_id",
                "run_id",
                "evidence_id",
                "claim_id",
                "material_ref_id",
                "text",
            ],
        ),
        (
            "prompt_pack_result_ref_edges",
            &[
                "id",
                "result_row_id",
                "run_id",
                "from_ref",
                "to_ref",
                "edge_kind",
            ],
        ),
        (
            "prompt_pack_result_unknowns",
            &["id", "result_row_id", "run_id", "unknown_id", "text"],
        ),
        (
            "prompt_pack_result_verification_tasks",
            &["id", "result_row_id", "run_id", "task_id", "text"],
        ),
        (
            "prompt_pack_result_warnings",
            &[
                "id",
                "result_row_id",
                "run_id",
                "warning_id",
                "code",
                "message",
            ],
        ),
        (
            "prompt_pack_result_limitations",
            &["id", "result_row_id", "run_id", "limitation_id", "message"],
        ),
        (
            "prompt_pack_result_quality_flags",
            &[
                "id",
                "result_row_id",
                "run_id",
                "flag_id",
                "severity",
                "message",
            ],
        ),
        (
            "prompt_pack_result_audit_refs",
            &[
                "id",
                "result_row_id",
                "run_id",
                "audit_ref_id",
                "event_kind",
            ],
        ),
        (
            "prompt_pack_youtube_videos",
            &[
                "id",
                "result_row_id",
                "run_id",
                "video_id",
                "source_ref_id",
                "title",
                "summary_text",
            ],
        ),
        (
            "prompt_pack_youtube_segments",
            &[
                "id",
                "result_row_id",
                "run_id",
                "video_id",
                "segment_id",
                "start_seconds",
                "end_seconds",
                "text",
            ],
        ),
        (
            "prompt_pack_youtube_key_points",
            &[
                "id",
                "result_row_id",
                "run_id",
                "video_id",
                "key_point_id",
                "text",
            ],
        ),
        (
            "prompt_pack_youtube_quotes",
            &[
                "id",
                "result_row_id",
                "run_id",
                "video_id",
                "quote_id",
                "text",
                "speaker",
            ],
        ),
        (
            "prompt_pack_youtube_action_items",
            &[
                "id",
                "result_row_id",
                "run_id",
                "video_id",
                "action_item_id",
                "text",
            ],
        ),
        (
            "prompt_pack_youtube_open_questions",
            &[
                "id",
                "result_row_id",
                "run_id",
                "video_id",
                "open_question_id",
                "text",
            ],
        ),
        (
            "prompt_pack_youtube_synthesis_items",
            &["id", "result_row_id", "run_id", "synthesis_id", "text"],
        ),
        (
            "prompt_pack_result_validation_findings",
            &[
                "id",
                "run_id",
                "stage_run_id",
                "severity",
                "code",
                "message",
                "object_path",
                "created_at",
            ],
        ),
        (
            "prompt_pack_audit_events",
            &[
                "id",
                "run_id",
                "event_kind",
                "message",
                "payload_json_zstd",
                "created_at",
            ],
        ),
        (
            "prompt_pack_result_quarantine_artifacts",
            &[
                "id",
                "run_id",
                "stage_run_id",
                "object_path",
                "reason",
                "content_json_zstd",
                "created_at",
            ],
        ),
    ];

    async fn table_columns(pool: &sqlx::SqlitePool, table: &str) -> Vec<String> {
        sqlx::query_scalar::<_, String>(&format!(
            "SELECT name FROM pragma_table_info('{table}') ORDER BY cid"
        ))
        .fetch_all(pool)
        .await
        .unwrap_or_else(|error| panic!("read columns for {table}: {error}"))
    }

    async fn foreign_key_pairs(pool: &sqlx::SqlitePool, table: &str) -> Vec<(String, String)> {
        sqlx::query_as::<_, (String, String)>(&format!(
            "SELECT \"from\", \"table\" FROM pragma_foreign_key_list('{table}')"
        ))
        .fetch_all(pool)
        .await
        .unwrap_or_else(|error| panic!("read foreign keys for {table}: {error}"))
    }

    #[tokio::test]
    async fn canonical_fixture_applies_declared_consumed_schema() {
        let pool = prompt_pack_test_pool().await;

        for (table, expected_columns) in CONSUMED_TABLE_COLUMNS {
            assert_eq!(
                table_columns(&pool, table).await,
                expected_columns
                    .iter()
                    .map(|column| (*column).to_string())
                    .collect::<Vec<_>>(),
                "{table} columns"
            );
        }
    }

    #[tokio::test]
    async fn canonical_fixture_preserves_consumed_indexes_and_foreign_keys() {
        let pool = prompt_pack_test_pool().await;

        for index in [
            "idx_prompt_pack_versions_one_active",
            "idx_prompt_pack_runs_project_created",
            "idx_prompt_pack_runs_client_request_id_unique",
            "idx_prompt_pack_stage_runs_run_order",
            "idx_prompt_pack_results_run",
        ] {
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|error| panic!("read index {index}: {error}"));
            assert_eq!(count, 1, "missing index {index}");
        }

        for (table, expected_pairs) in [
            ("prompt_pack_versions", &[("pack_id", "prompt_packs")][..]),
            (
                "prompt_pack_runs",
                &[
                    ("pack_version_id", "prompt_pack_versions"),
                    ("project_id", "projects"),
                ][..],
            ),
            (
                "prompt_pack_run_scopes",
                &[("run_id", "prompt_pack_runs"), ("source_id", "sources")][..],
            ),
            (
                "prompt_pack_run_source_snapshots",
                &[("run_id", "prompt_pack_runs"), ("source_id", "sources")][..],
            ),
            (
                "prompt_pack_stage_runs",
                &[
                    ("run_id", "prompt_pack_runs"),
                    ("source_snapshot_id", "prompt_pack_run_source_snapshots"),
                ][..],
            ),
            (
                "prompt_pack_stage_artifacts",
                &[
                    ("run_id", "prompt_pack_runs"),
                    ("stage_run_id", "prompt_pack_stage_runs"),
                ][..],
            ),
            ("prompt_pack_results", &[("run_id", "prompt_pack_runs")][..]),
            (
                "prompt_pack_result_source_refs",
                &[("result_row_id", "prompt_pack_results")][..],
            ),
            (
                "prompt_pack_result_validation_findings",
                &[
                    ("run_id", "prompt_pack_runs"),
                    ("stage_run_id", "prompt_pack_stage_runs"),
                ][..],
            ),
            (
                "prompt_pack_result_quarantine_artifacts",
                &[
                    ("run_id", "prompt_pack_runs"),
                    ("stage_run_id", "prompt_pack_stage_runs"),
                ][..],
            ),
        ] {
            let actual = foreign_key_pairs(&pool, table).await;
            for (from, target) in expected_pairs {
                assert!(
                    actual
                        .iter()
                        .any(|pair| pair == &((*from).to_string(), (*target).to_string())),
                    "{table} must preserve {from} -> {target}; actual: {actual:?}"
                );
            }
        }
    }
}
