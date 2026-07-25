#[cfg(test)]
const ANALYSIS_TEST_MIGRATIONS: [(&str, &str); 12] = [
    (
        "src-tauri/migrations/0001_current_schema_baseline.sql",
        include_str!("../../migrations/0001_current_schema_baseline.sql"),
    ),
    (
        "src-tauri/migrations/0002_migrated_history_opt_in_schema.sql",
        include_str!("../../migrations/0002_migrated_history_opt_in_schema.sql"),
    ),
    (
        "src-tauri/migrations/0003_analysis_telegram_history_scope.sql",
        include_str!("../../migrations/0003_analysis_telegram_history_scope.sql"),
    ),
    (
        "src-tauri/migrations/0004_source_delete_cascade_indexes.sql",
        include_str!("../../migrations/0004_source_delete_cascade_indexes.sql"),
    ),
    (
        "src-tauri/migrations/0005_projects_mvp.sql",
        include_str!("../../migrations/0005_projects_mvp.sql"),
    ),
    (
        "src-tauri/migrations/0006_prompt_pack_mvp.sql",
        include_str!("../../migrations/0006_prompt_pack_mvp.sql"),
    ),
    (
        "src-tauri/migrations/0007_prompt_pack_run_idempotency.sql",
        include_str!("../../migrations/0007_prompt_pack_run_idempotency.sql"),
    ),
    (
        "src-tauri/migrations/0008_prompt_pack_run_labels.sql",
        include_str!("../../migrations/0008_prompt_pack_run_labels.sql"),
    ),
    (
        "src-tauri/migrations/0009_prompt_pack_intermediate_entities_artifacts.sql",
        include_str!("../../migrations/0009_prompt_pack_intermediate_entities_artifacts.sql"),
    ),
    (
        "src-tauri/migrations/0010_prompt_pack_runtime_provider.sql",
        include_str!("../../migrations/0010_prompt_pack_runtime_provider.sql"),
    ),
    (
        "src-tauri/migrations/0011_prompt_pack_stage_browser_provenance.sql",
        include_str!("../../migrations/0011_prompt_pack_stage_browser_provenance.sql"),
    ),
    (
        "src-tauri/migrations/0012_projects_redesign.sql",
        include_str!("../../migrations/0012_projects_redesign.sql"),
    ),
];

#[cfg(test)]
pub(crate) async fn analysis_test_pool() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect analysis test pool");
    let mut transaction = pool.begin().await.expect("begin analysis test schema");
    for (_, sql) in ANALYSIS_TEST_MIGRATIONS {
        sqlx::raw_sql(sql)
            .execute(&mut *transaction)
            .await
            .expect("apply analysis test migration");
    }
    transaction
        .commit()
        .await
        .expect("commit analysis test schema");
    pool
}

#[cfg(test)]
mod tests {
    use super::analysis_test_pool;

    const CONSUMED_TABLE_COLUMNS: &[(&str, &[&str])] = &[
        (
            "analysis_runs",
            &[
                "id",
                "run_type",
                "scope_type",
                "source_id",
                "period_from",
                "period_to",
                "output_language",
                "prompt_template_id",
                "prompt_template_version",
                "provider_profile",
                "provider",
                "model",
                "status",
                "result_markdown",
                "trace_data_zstd",
                "error",
                "created_at",
                "completed_at",
                "source_group_id",
                "scope_label_snapshot",
                "youtube_corpus_mode",
                "snapshot_captured_at",
                "snapshot_error",
                "telegram_history_scope",
                "project_id",
            ],
        ),
        (
            "analysis_run_messages",
            &[
                "run_id",
                "item_id",
                "source_id",
                "external_id",
                "author",
                "published_at",
                "ref",
                "content_zstd",
                "item_kind",
                "source_type",
                "source_subtype",
                "metadata_zstd",
            ],
        ),
        (
            "analysis_chat_messages",
            &["id", "run_id", "role", "content", "created_at"],
        ),
        (
            "analysis_prompt_templates",
            &[
                "id",
                "name",
                "template_kind",
                "body",
                "version",
                "is_builtin",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "analysis_source_groups",
            &["id", "name", "created_at", "updated_at", "source_type"],
        ),
        (
            "analysis_source_group_members",
            &["group_id", "source_id", "created_at"],
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

    async fn index_columns(pool: &sqlx::SqlitePool, index: &str) -> Vec<String> {
        sqlx::query_scalar::<_, String>(&format!(
            "SELECT name FROM pragma_index_info('{index}') ORDER BY seqno"
        ))
        .fetch_all(pool)
        .await
        .unwrap_or_else(|error| panic!("read columns for index {index}: {error}"))
    }

    async fn foreign_keys(
        pool: &sqlx::SqlitePool,
        table: &str,
    ) -> Vec<(String, String, String, String)> {
        sqlx::query_as::<_, (String, String, String, String)>(&format!(
            r#"SELECT "from", "table", "to", "on_delete"
               FROM pragma_foreign_key_list('{table}')"#
        ))
        .fetch_all(pool)
        .await
        .unwrap_or_else(|error| panic!("read foreign keys for {table}: {error}"))
    }

    async fn row_count(pool: &sqlx::SqlitePool, table: &str) -> i64 {
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(pool)
            .await
            .unwrap_or_else(|error| panic!("count rows in {table}: {error}"))
    }

    #[tokio::test]
    async fn canonical_fixture_applies_analysis_consumed_schema() {
        let pool = analysis_test_pool().await;

        for (table, expected_columns) in CONSUMED_TABLE_COLUMNS {
            assert_eq!(
                table_columns(&pool, table).await,
                expected_columns
                    .iter()
                    .map(|column| (*column).to_string())
                    .collect::<Vec<_>>(),
                "RED: CP5 consumed canonical schema: {table} columns"
            );
        }
    }

    #[tokio::test]
    async fn canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys() {
        let pool = analysis_test_pool().await;

        let foreign_keys_enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .expect("read foreign_keys pragma");
        assert_eq!(
            foreign_keys_enabled, 1,
            "RED: CP5 canonical indexes and foreign keys"
        );

        for (index, expected_columns) in [
            (
                "idx_analysis_chat_messages_run_created",
                &["run_id", "created_at", "id"][..],
            ),
            (
                "idx_analysis_prompt_templates_kind_name",
                &["template_kind", "name"][..],
            ),
            (
                "idx_analysis_run_messages_run_published",
                &["run_id", "published_at", "ref"][..],
            ),
            (
                "idx_analysis_run_messages_run_source",
                &["run_id", "source_id"][..],
            ),
            (
                "idx_analysis_runs_source_created",
                &["source_id", "created_at"][..],
            ),
            (
                "idx_analysis_runs_source_group_created",
                &["source_group_id", "created_at"][..],
            ),
            (
                "idx_analysis_runs_status_created",
                &["status", "created_at"][..],
            ),
            (
                "idx_analysis_source_group_members_source_id",
                &["source_id"][..],
            ),
            (
                "idx_analysis_source_groups_source_type",
                &["source_type"][..],
            ),
            ("idx_analysis_source_groups_updated_at", &["updated_at"][..]),
            (
                "idx_analysis_runs_project_id_created_at",
                &["project_id", "created_at"][..],
            ),
        ] {
            assert_eq!(
                index_columns(&pool, index).await,
                expected_columns
                    .iter()
                    .map(|column| (*column).to_string())
                    .collect::<Vec<_>>(),
                "{index} columns"
            );
        }

        for (table, expected_keys) in [
            (
                "analysis_chat_messages",
                &[("run_id", "analysis_runs", "id", "CASCADE")][..],
            ),
            (
                "analysis_run_messages",
                &[("run_id", "analysis_runs", "id", "CASCADE")][..],
            ),
            (
                "analysis_source_group_members",
                &[
                    ("group_id", "analysis_source_groups", "id", "CASCADE"),
                    ("source_id", "sources", "id", "CASCADE"),
                ][..],
            ),
            (
                "analysis_runs",
                &[("project_id", "projects", "id", "CASCADE")][..],
            ),
        ] {
            let actual = foreign_keys(&pool, table).await;
            for (from, target_table, to, on_delete) in expected_keys {
                let expected = (
                    (*from).to_string(),
                    (*target_table).to_string(),
                    (*to).to_string(),
                    (*on_delete).to_string(),
                );
                assert!(
                    actual.contains(&expected),
                    "{table} must preserve {expected:?}; actual: {actual:?}"
                );
            }
        }

        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, created_at
             ) VALUES (1, 'youtube', 'video', 'fixture-video-1', 'Fixture 1', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert first cascade source");
        sqlx::query(
            "INSERT INTO projects (id, name, created_at, updated_at)
             VALUES (1, 'Fixture project', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert cascade project");
        sqlx::query(
            "INSERT INTO analysis_source_groups (
                id, name, source_type, created_at, updated_at
             ) VALUES (1, 'Fixture group', 'youtube', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert cascade source group");
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert cascade source group member");
        sqlx::query(
            "INSERT INTO analysis_runs (
                id, run_type, scope_type, project_id, period_from, period_to,
                output_language, prompt_template_version, provider_profile,
                provider, model, status, created_at
             ) VALUES (
                1, 'report', 'project', 1, 1, 2, 'Russian', 1,
                'default', 'openai', 'fixture-model', 'completed', 1
             )",
        )
        .execute(&pool)
        .await
        .expect("insert cascade analysis run");
        sqlx::query(
            "INSERT INTO analysis_run_messages (
                run_id, item_id, source_id, external_id, published_at, ref, content_zstd
             ) VALUES (1, 1, 1, 'fixture-item', 1, 's1-i1', x'00')",
        )
        .execute(&pool)
        .await
        .expect("insert cascade run message");
        sqlx::query(
            "INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
             VALUES (1, 'user', 'Fixture question', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert cascade chat message");

        sqlx::query("DELETE FROM projects WHERE id = 1")
            .execute(&pool)
            .await
            .expect("delete cascade project");
        assert_eq!(row_count(&pool, "analysis_runs").await, 0);
        assert_eq!(row_count(&pool, "analysis_run_messages").await, 0);
        assert_eq!(row_count(&pool, "analysis_chat_messages").await, 0);

        sqlx::query("DELETE FROM sources WHERE id = 1")
            .execute(&pool)
            .await
            .expect("delete cascade source");
        assert_eq!(row_count(&pool, "analysis_source_group_members").await, 0);

        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, created_at
             ) VALUES (2, 'youtube', 'video', 'fixture-video-2', 'Fixture 2', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert second cascade source");
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (1, 2, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert second cascade source group member");
        sqlx::query("DELETE FROM analysis_source_groups WHERE id = 1")
            .execute(&pool)
            .await
            .expect("delete cascade source group");
        assert_eq!(row_count(&pool, "analysis_source_group_members").await, 0);
    }
}
