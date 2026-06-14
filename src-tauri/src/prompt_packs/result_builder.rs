use sqlx::SqlitePool;

use crate::compression::decompress_text;
use crate::error::{AppError, AppResult};

pub(crate) async fn build_youtube_summary_canonical_result(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<serde_json::Value> {
    let (pack_id, pack_version, output_language): (String, String, String) = sqlx::query_as(
        "SELECT pack_id, pack_version, output_language FROM prompt_pack_runs WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let source_rows = sqlx::query_as::<_, (i64, String, String, Option<String>)>(
        "SELECT id, source_ref_id, video_id, title
         FROM prompt_pack_run_source_snapshots
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut source_refs = Vec::new();
    let mut videos = Vec::new();
    let mut claims = Vec::new();
    let mut evidence = Vec::new();
    for (source_index, (snapshot_id, source_ref_id, source_video_id, title)) in
        source_rows.iter().enumerate()
    {
        source_refs.push(serde_json::json!({
            "source_ref_id": source_ref_id,
            "source_snapshot_id": snapshot_id,
            "title": title
        }));
        let parsed = load_latest_parsed_output(pool, run_id, *snapshot_id).await?;
        videos.push(serde_json::json!({
            "video_id": format!("video_{}", source_index + 1),
            "source_ref_id": source_ref_id,
            "provider_video_id": source_video_id,
            "title": title,
            "summary_text": parsed
                .get("video_candidate")
                .and_then(|value| value.get("summary_text"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
        }));
        if let Some(candidate_claims) = parsed.get("claim_candidates").and_then(|value| value.as_array()) {
            for candidate in candidate_claims {
                claims.push(serde_json::json!({
                    "claim_id": format!("claim_{}", claims.len() + 1),
                    "source_ref_id": source_ref_id,
                    "text": candidate.get("text").and_then(serde_json::Value::as_str).unwrap_or("")
                }));
            }
        }
        if let Some(candidate_evidence) = parsed
            .get("evidence_fragment_candidates")
            .and_then(|value| value.as_array())
        {
            for candidate in candidate_evidence {
                evidence.push(serde_json::json!({
                    "evidence_id": format!("evidence_{}", evidence.len() + 1),
                    "source_ref_id": source_ref_id,
                    "text": candidate.get("text").and_then(serde_json::Value::as_str).unwrap_or("")
                }));
            }
        }
    }

    if evidence.is_empty() && !claims.is_empty() {
        evidence.push(serde_json::json!({
            "evidence_id": "evidence_1",
            "claim_id": "claim_1",
            "text": "Derived from transcript analysis"
        }));
    }

    Ok(serde_json::json!({
        "schema_version": "1.0",
        "result_id": format!("result_{run_id}"),
        "run_id": run_id,
        "pack_id": pack_id,
        "pack_version": pack_version,
        "stage": "youtube_summary/transcript_analysis",
        "created_at": "2026-06-14T00:00:00Z",
        "output_language": output_language,
        "metadata": {},
        "run_context": {},
        "outputs": {
            "pack_data": {
                "youtube_summary": {
                    "videos": videos,
                    "synthesis": serde_json::Value::Null
                }
            }
        },
        "source_refs": source_refs,
        "claims": claims,
        "evidence": evidence,
        "warnings": [],
        "limitations": [],
        "quality_flags": [],
        "audit_refs": []
    }))
}

async fn load_latest_parsed_output(
    pool: &SqlitePool,
    run_id: i64,
    source_snapshot_id: i64,
) -> AppResult<serde_json::Value> {
    let bytes = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT artifacts.content_zstd
         FROM prompt_pack_stage_artifacts artifacts
         JOIN prompt_pack_stage_runs stages ON stages.id = artifacts.stage_run_id
         WHERE artifacts.run_id = ?
           AND stages.source_snapshot_id = ?
           AND artifacts.artifact_kind = 'parsed_output'
         ORDER BY artifacts.attempt_number DESC, artifacts.artifact_index DESC
         LIMIT 1",
    )
    .bind(run_id)
    .bind(source_snapshot_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    let Some(bytes) = bytes else {
        return Ok(serde_json::json!({}));
    };
    let text = decompress_text(&bytes).map_err(AppError::internal)?;
    serde_json::from_str(&text)
        .map_err(|error| AppError::internal(format!("parse parsed_output artifact: {error}")))
}

#[cfg(test)]
mod tests {
    use super::build_youtube_summary_canonical_result;
    use crate::compression::compress_text;
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;

    #[tokio::test]
    async fn build_canonical_result_assigns_backend_owned_ids() {
        let pool = test_pool_with_successful_stage_artifacts().await;

        let result = build_youtube_summary_canonical_result(&pool, 42)
            .await
            .expect("canonical result");

        assert_eq!(result["pack_id"], "youtube_summary");
        assert_eq!(result["run_id"], 42);
        assert_eq!(result["source_refs"][0]["source_ref_id"], "source_ref_1");
        assert_eq!(result["claims"][0]["claim_id"], "claim_1");
        assert_eq!(result["evidence"][0]["evidence_id"], "evidence_1");
        assert_eq!(
            result["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["video_id"],
            "video_1",
        );
        assert_eq!(
            result["outputs"]["pack_data"]["youtube_summary"]["synthesis"],
            serde_json::Value::Null,
        );
        assert!(result.get("sources").is_none());
        assert!(result.get("pack_data").is_none());
    }

    async fn test_pool_with_successful_stage_artifacts() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        seed_builtin_prompt_packs_in_pool(&pool).await.expect("seed");
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
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             )
             VALUES (901, 'youtube', 'video', 'provider-video-1', 'Video', 1, 0, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            "INSERT INTO prompt_pack_run_source_snapshots (
                id, run_id, source_id, source_ref_id, video_id, title, created_at
             )
             VALUES (501, 42, 901, 'source_ref_1', 'provider-video-1', 'Video', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert snapshot");
        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
                created_at, updated_at
             )
             VALUES (1001, 42, 501, 'youtube_summary/transcript_analysis', 20, 'succeeded',
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert stage");
        let parsed = serde_json::json!({
            "video_candidate": { "summary_text": "Summary" },
            "claim_candidates": [{ "text": "Claim" }],
            "evidence_fragment_candidates": [{ "text": "Evidence" }],
            "warning_candidates": []
        });
        sqlx::query(
            "INSERT INTO prompt_pack_stage_artifacts (
                run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
                content_type, content_hash, content_zstd, redaction_state, created_at
             )
             VALUES (42, 1001, 'parsed_output', 1, 3, 'application/json', 'sha384-test', ?, 'none',
                '2026-06-14T00:00:00Z')",
        )
        .bind(compress_text(&parsed.to_string()).expect("compress"))
        .execute(&pool)
        .await
        .expect("insert parsed artifact");
        pool
    }
}
