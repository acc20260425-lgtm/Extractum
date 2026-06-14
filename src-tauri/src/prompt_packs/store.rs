use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

pub(crate) async fn require_prompt_pack_version_id(
    pool: &SqlitePool,
    pack_id: &str,
    pack_version: &str,
) -> AppResult<i64> {
    sqlx::query_scalar(
        "SELECT id FROM prompt_pack_versions WHERE pack_id = ? AND pack_version = ?",
    )
    .bind(pack_id)
    .bind(pack_version)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found(format!("Prompt pack {pack_id}@{pack_version} not found")))
}

#[cfg(test)]
mod tests {
    use crate::migrations::apply_all_migrations_for_test_pool;

    async fn test_pool_with_prompt_pack_schema() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        seed_minimal_pack_version(&pool).await;
        pool
    }

    async fn seed_minimal_pack_version(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT INTO prompt_packs (pack_id, display_name, is_builtin, created_at, updated_at)
             VALUES ('youtube_summary', 'YouTube Summary', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("insert prompt pack");

        sqlx::query(
            "INSERT INTO prompt_pack_versions (
                id, pack_id, pack_version, schema_version, origin_kind, lifecycle_status,
                content_hash, bundled_source_path, default_control_preset,
                default_evidence_mode, default_include_comments, created_at, updated_at
             )
             VALUES (
                10, 'youtube_summary', '1.0.0', '1.0', 'bundled', 'active',
                'sha384-test', 'test', 'standard', 'standard', 0, 1, 1
             )",
        )
        .execute(pool)
        .await
        .expect("insert prompt pack version");
    }

    async fn insert_minimal_prompt_pack_run(
        pool: &sqlx::SqlitePool,
        run_id: i64,
        client_request_id: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO prompt_pack_runs (
                id, project_id, pack_version_id, pack_id, pack_version, schema_version,
                run_status, result_status, output_language, control_preset, evidence_mode,
                include_comments, created_at, updated_at, client_request_id
             )
             VALUES (
                ?, NULL, 10, 'youtube_summary', '1.0.0', '1.0',
                'queued', 'none', 'en', 'standard', 'standard',
                0, '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z', ?
             )",
        )
        .bind(run_id)
        .bind(client_request_id)
        .execute(pool)
        .await
        .map(|_| ())
    }

    #[tokio::test]
    async fn prompt_pack_runs_client_request_id_is_unique_when_present() {
        let pool = test_pool_with_prompt_pack_schema().await;

        insert_minimal_prompt_pack_run(&pool, 41, Some("req-duplicate"))
            .await
            .expect("first run");
        let duplicate = insert_minimal_prompt_pack_run(&pool, 42, Some("req-duplicate"))
            .await
            .expect_err("duplicate request id rejected");

        assert!(duplicate.to_string().contains("client_request_id"));
    }

    #[tokio::test]
    async fn prompt_pack_runs_allow_null_client_request_id_for_pre_existing_rows() {
        let pool = test_pool_with_prompt_pack_schema().await;

        insert_minimal_prompt_pack_run(&pool, 41, None)
            .await
            .expect("first legacy-compatible run");
        insert_minimal_prompt_pack_run(&pool, 42, None)
            .await
            .expect("second legacy-compatible run");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id IS NULL",
        )
        .fetch_one(&pool)
        .await
        .expect("null request ids");

        assert_eq!(count, 2);
    }
}
