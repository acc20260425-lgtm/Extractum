use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::compression::{compress_text, decompress_text};
use crate::error::{AppError, AppResult};

#[allow(dead_code)]
pub(crate) async fn persist_final_result_transaction(
    pool: &SqlitePool,
    run_id: i64,
    canonical_result: serde_json::Value,
    terminal_status: &str,
) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    persist_final_result_in_transaction(&mut tx, run_id, &canonical_result, terminal_status)
        .await?;
    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn persist_final_result_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: i64,
    canonical_result: &serde_json::Value,
    terminal_status: &str,
) -> AppResult<()> {
    let canonical_json = canonical_result.to_string();
    let now = crate::time::now_rfc3339_utc();
    let result_row_id: i64 = sqlx::query_scalar(
        "INSERT INTO prompt_pack_results (
            run_id, result_id, result_status, schema_version, canonical_hash,
            canonical_json_zstd, projection_updated_at, created_at, updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, NULL, ?, ?)
         ON CONFLICT(run_id) DO UPDATE SET
            result_status = excluded.result_status,
            canonical_hash = excluded.canonical_hash,
            canonical_json_zstd = excluded.canonical_json_zstd,
            updated_at = excluded.updated_at
         RETURNING id",
    )
    .bind(run_id)
    .bind(canonical_result["result_id"].as_str().unwrap_or("result"))
    .bind(terminal_status)
    .bind(canonical_result["schema_version"].as_str().unwrap_or("1.0"))
    .bind(format!("sha384-{}", sha384_hex(canonical_json.as_bytes())))
    .bind(compress_text(&canonical_json).map_err(AppError::internal)?)
    .bind(&now)
    .bind(&now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    rebuild_projection_rows_in_transaction(tx, result_row_id, run_id, canonical_result).await?;
    sqlx::query(
        "UPDATE prompt_pack_results SET projection_updated_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(&now)
    .bind(result_row_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = ?, result_status = ?, completed_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(terminal_status)
    .bind(terminal_status)
    .bind(&now)
    .bind(&now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    sqlx::query(
        "INSERT INTO prompt_pack_audit_events (run_id, event_kind, message, created_at)
         VALUES (?, 'terminal_result_persisted', 'Prompt Pack result persisted', ?)",
    )
    .bind(run_id)
    .bind(&now)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn repair_prompt_pack_result_projections(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<()> {
    let (result_row_id, canonical_json_zstd): (i64, Vec<u8>) =
        sqlx::query_as("SELECT id, canonical_json_zstd FROM prompt_pack_results WHERE run_id = ?")
            .bind(run_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
    let canonical_json = decompress_text(&canonical_json_zstd).map_err(AppError::internal)?;
    let canonical: serde_json::Value = serde_json::from_str(&canonical_json)
        .map_err(|error| AppError::internal(format!("parse canonical result: {error}")))?;
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    rebuild_projection_rows_in_transaction(&mut tx, result_row_id, run_id, &canonical).await?;
    sqlx::query("UPDATE prompt_pack_results SET projection_updated_at = ? WHERE id = ?")
        .bind(crate::time::now_rfc3339_utc())
        .bind(result_row_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}

async fn rebuild_projection_rows_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    result_row_id: i64,
    run_id: i64,
    canonical: &serde_json::Value,
) -> AppResult<()> {
    for table in [
        "prompt_pack_result_source_refs",
        "prompt_pack_result_claims",
        "prompt_pack_result_evidence",
        "prompt_pack_result_ref_edges",
        "prompt_pack_result_unknowns",
        "prompt_pack_result_verification_tasks",
        "prompt_pack_result_warnings",
        "prompt_pack_result_limitations",
        "prompt_pack_result_quality_flags",
        "prompt_pack_result_audit_refs",
        "prompt_pack_youtube_videos",
        "prompt_pack_youtube_segments",
        "prompt_pack_youtube_key_points",
        "prompt_pack_youtube_quotes",
        "prompt_pack_youtube_action_items",
        "prompt_pack_youtube_open_questions",
        "prompt_pack_youtube_synthesis_items",
    ] {
        sqlx::query(&format!("DELETE FROM {table} WHERE result_row_id = ?"))
            .bind(result_row_id)
            .execute(&mut **tx)
            .await
            .map_err(AppError::database)?;
    }

    for source_ref in canonical
        .get("source_refs")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        sqlx::query(
            "INSERT INTO prompt_pack_result_source_refs (
                result_row_id, run_id, source_ref_id, source_snapshot_id, title
             )
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(result_row_id)
        .bind(run_id)
        .bind(source_ref["source_ref_id"].as_str().unwrap_or(""))
        .bind(source_ref["source_snapshot_id"].as_i64().unwrap_or(0))
        .bind(source_ref["title"].as_str())
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    for claim in canonical
        .get("claims")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        sqlx::query(
            "INSERT INTO prompt_pack_result_claims (
                result_row_id, run_id, claim_id, source_ref_id, text
             )
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(result_row_id)
        .bind(run_id)
        .bind(claim["claim_id"].as_str().unwrap_or(""))
        .bind(claim["source_ref_id"].as_str())
        .bind(claim["text"].as_str().unwrap_or(""))
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    for evidence in canonical
        .get("evidence")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        sqlx::query(
            "INSERT INTO prompt_pack_result_evidence (
                result_row_id, run_id, evidence_id, claim_id, material_ref_id, text
             )
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(result_row_id)
        .bind(run_id)
        .bind(evidence["evidence_id"].as_str().unwrap_or(""))
        .bind(evidence["claim_id"].as_str())
        .bind(evidence["material_ref_id"].as_str())
        .bind(evidence["text"].as_str().unwrap_or(""))
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    for video in canonical["outputs"]["pack_data"]["youtube_summary"]["videos"]
        .as_array()
        .into_iter()
        .flatten()
    {
        sqlx::query(
            "INSERT INTO prompt_pack_youtube_videos (
                result_row_id, run_id, video_id, source_ref_id, title, summary_text
             )
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(result_row_id)
        .bind(run_id)
        .bind(video["video_id"].as_str().unwrap_or(""))
        .bind(video["source_ref_id"].as_str().unwrap_or(""))
        .bind(video["title"].as_str())
        .bind(video["summary_text"].as_str())
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    if let Some(synthesis) =
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"].as_object()
    {
        for item in synthesis
            .get("cross_video_themes")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            insert_youtube_synthesis_projection_item(
                tx,
                result_row_id,
                run_id,
                item["theme_id"].as_str().unwrap_or(""),
                item["theme_text"].as_str().unwrap_or(""),
            )
            .await?;
        }

        for item in synthesis
            .get("common_claims")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            insert_youtube_synthesis_projection_item(
                tx,
                result_row_id,
                run_id,
                item["common_claim_id"].as_str().unwrap_or(""),
                item["summary_text"].as_str().unwrap_or(""),
            )
            .await?;
        }

        for item in synthesis
            .get("contradictions_across_videos")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            insert_youtube_synthesis_projection_item(
                tx,
                result_row_id,
                run_id,
                item["contradiction_id"].as_str().unwrap_or(""),
                item["description"].as_str().unwrap_or(""),
            )
            .await?;
        }
    }

    Ok(())
}

async fn insert_youtube_synthesis_projection_item(
    tx: &mut Transaction<'_, Sqlite>,
    result_row_id: i64,
    run_id: i64,
    synthesis_id: &str,
    text: &str,
) -> AppResult<()> {
    if synthesis_id.is_empty() || text.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO prompt_pack_youtube_synthesis_items (
            result_row_id, run_id, synthesis_id, text
         )
         VALUES (?, ?, ?, ?)",
    )
    .bind(result_row_id)
    .bind(run_id)
    .bind(synthesis_id)
    .bind(text)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn sha384_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha384};
    Sha384::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{persist_final_result_transaction, repair_prompt_pack_result_projections};
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;

    #[tokio::test]
    async fn persist_final_result_sets_terminal_status_after_projection_rows_exist() {
        let pool = test_pool_with_canonical_result_ready().await;

        persist_final_result_transaction(&pool, 42, test_canonical_result(), "complete")
            .await
            .expect("persist result");

        let run_status: String =
            sqlx::query_scalar("SELECT run_status FROM prompt_pack_runs WHERE id = 42")
                .fetch_one(&pool)
                .await
                .expect("run status");
        let projected_videos: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_youtube_videos WHERE run_id = 42")
                .fetch_one(&pool)
                .await
                .expect("projected videos");
        let result_status: String =
            sqlx::query_scalar("SELECT result_status FROM prompt_pack_results WHERE run_id = 42")
                .fetch_one(&pool)
                .await
                .expect("result status");

        assert_eq!(run_status, "complete");
        assert_eq!(result_status, "complete");
        assert!(projected_videos > 0);
    }

    #[tokio::test]
    async fn repair_rebuilds_missing_projection_rows_from_canonical_json() {
        let pool = test_pool_with_canonical_result_ready().await;
        persist_final_result_transaction(&pool, 42, test_canonical_result(), "complete")
            .await
            .expect("persist result");
        sqlx::query("DELETE FROM prompt_pack_result_claims WHERE run_id = 42")
            .execute(&pool)
            .await
            .expect("delete claims");

        repair_prompt_pack_result_projections(&pool, 42)
            .await
            .expect("repair projections");

        let projected_claims: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_result_claims WHERE run_id = 42")
                .fetch_one(&pool)
                .await
                .expect("projected claims");

        assert!(projected_claims > 0);
    }

    #[tokio::test]
    async fn persist_final_result_projects_youtube_synthesis_items() {
        let pool = test_pool_with_canonical_result_ready().await;

        persist_final_result_transaction(
            &pool,
            42,
            test_canonical_result_with_synthesis(),
            "complete",
        )
        .await
        .expect("persist result");

        let items: Vec<(String, String)> = sqlx::query_as(
            "SELECT synthesis_id, text
             FROM prompt_pack_youtube_synthesis_items
             WHERE run_id = 42
             ORDER BY synthesis_id ASC",
        )
        .fetch_all(&pool)
        .await
        .expect("synthesis projection rows");

        assert_eq!(
            items,
            vec![
                (
                    "common_claim_1".to_string(),
                    "Both videos mention pilots.".to_string()
                ),
                ("theme_1".to_string(), "Shared theme".to_string()),
            ]
        );
    }

    #[tokio::test]
    async fn repair_rebuilds_missing_youtube_synthesis_projection_rows() {
        let pool = test_pool_with_canonical_result_ready().await;
        persist_final_result_transaction(
            &pool,
            42,
            test_canonical_result_with_synthesis(),
            "complete",
        )
        .await
        .expect("persist result");
        sqlx::query("DELETE FROM prompt_pack_youtube_synthesis_items WHERE run_id = 42")
            .execute(&pool)
            .await
            .expect("delete synthesis projections");

        repair_prompt_pack_result_projections(&pool, 42)
            .await
            .expect("repair projections");

        let projected_items: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_youtube_synthesis_items WHERE run_id = 42",
        )
        .fetch_one(&pool)
        .await
        .expect("projected synthesis items");

        assert_eq!(projected_items, 2);
    }

    #[tokio::test]
    async fn persist_final_result_uses_current_time_for_run_completion() {
        use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

        let pool = test_pool_with_canonical_result_ready().await;
        let before = OffsetDateTime::now_utc() - Duration::seconds(5);
        persist_final_result_transaction(&pool, 42, test_canonical_result(), "complete")
            .await
            .expect("persist result");
        let after = OffsetDateTime::now_utc() + Duration::seconds(5);

        let completed_at: String =
            sqlx::query_scalar("SELECT completed_at FROM prompt_pack_runs WHERE id = 42")
                .fetch_one(&pool)
                .await
                .expect("completed_at");
        let parsed = OffsetDateTime::parse(&completed_at, &Rfc3339).expect("parse completed_at");

        assert_ne!(completed_at, "2026-06-14T00:00:00Z");
        assert!(
            parsed >= before && parsed <= after,
            "expected {completed_at} to be between {before} and {after}"
        );
    }

    #[tokio::test]
    async fn low_level_result_persistence_rolls_back_when_projection_insert_fails() {
        let pool = test_pool_with_canonical_result_ready().await;
        // This deliberately bypasses result validation to test the lower-level
        // projection persistence transaction boundary.
        let mut canonical = test_canonical_result();
        canonical["source_refs"] = serde_json::json!([
            { "source_ref_id": "source_ref_1", "source_snapshot_id": 501, "title": "Video" },
            { "source_ref_id": "source_ref_1", "source_snapshot_id": 502, "title": "Duplicate" }
        ]);

        let error = persist_final_result_transaction(&pool, 42, canonical, "complete")
            .await
            .expect_err("projection unique constraint should fail");

        assert!(
            error.message.contains("Database error"),
            "unexpected error: {error:?}"
        );
        let result_rows: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_results WHERE run_id = 42")
                .fetch_one(&pool)
                .await
                .expect("result count");
        let source_projection_rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_result_source_refs WHERE run_id = 42",
        )
        .fetch_one(&pool)
        .await
        .expect("source projection count");

        assert_eq!(result_rows, 0);
        assert_eq!(source_projection_rows, 0);
    }

    async fn test_pool_with_canonical_result_ready() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed");
        sqlx::query(
            "INSERT INTO prompt_pack_runs (
                id, pack_version_id, pack_id, pack_version, schema_version,
                run_status, result_status, output_language, control_preset,
                evidence_mode, include_comments, created_at, updated_at
             )
             VALUES (42, 1, 'youtube_summary', '1.0.0', '1.0',
                'running', 'none', 'en', 'standard', 'standard', 0,
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert run");
        pool
    }

    fn test_canonical_result() -> serde_json::Value {
        serde_json::json!({
            "schema_version": "1.0",
            "result_id": "result_42",
            "run_id": 42,
            "pack_id": "youtube_summary",
            "pack_version": "1.0.0",
            "stage": "youtube_summary/transcript_analysis",
            "created_at": "2026-06-14T00:00:00Z",
            "output_language": "en",
            "metadata": {},
            "run_context": {},
            "outputs": {
                "pack_data": {
                    "youtube_summary": {
                        "videos": [{
                            "video_id": "video_1",
                            "source_ref_id": "source_ref_1",
                            "title": "Video",
                            "summary_text": "Summary"
                        }],
                        "synthesis": null
                    }
                }
            },
            "source_refs": [{
                "source_ref_id": "source_ref_1",
                "source_snapshot_id": 501,
                "title": "Video"
            }],
            "claims": [{
                "claim_id": "claim_1",
                "source_ref_id": "source_ref_1",
                "text": "Claim"
            }],
            "evidence": [{
                "evidence_id": "evidence_1",
                "claim_id": "claim_1",
                "text": "Evidence"
            }],
            "warnings": [],
            "limitations": [],
            "quality_flags": [],
            "audit_refs": []
        })
    }

    fn test_canonical_result_with_synthesis() -> serde_json::Value {
        let mut canonical = test_canonical_result();
        canonical["outputs"]["pack_data"]["youtube_summary"]["videos"] = serde_json::json!([
            {
                "video_id": "video_1",
                "source_ref_id": "source_ref_1",
                "title": "Video 1",
                "summary_text": "Summary 1"
            },
            {
                "video_id": "video_2",
                "source_ref_id": "source_ref_2",
                "title": "Video 2",
                "summary_text": "Summary 2"
            }
        ]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"] = serde_json::json!({
            "cross_video_themes": [
                {
                    "theme_id": "theme_1",
                    "theme_text": "Shared theme",
                    "video_refs": ["video_1", "video_2"],
                    "claim_refs": [],
                    "evidence_refs": []
                }
            ],
            "common_claims": [
                {
                    "common_claim_id": "common_claim_1",
                    "summary_text": "Both videos mention pilots.",
                    "video_refs": ["video_1", "video_2"],
                    "claim_refs": [],
                    "evidence_refs": []
                }
            ],
            "contradictions_across_videos": [],
            "claim_refs": [],
            "relation_refs": [],
            "evidence_refs": [],
            "source_refs": ["source_ref_1", "source_ref_2"]
        });
        canonical
    }
}
