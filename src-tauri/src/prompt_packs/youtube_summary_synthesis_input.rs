use sqlx::SqlitePool;

use crate::compression::decompress_text;
use crate::error::{AppError, AppResult};

pub(crate) async fn build_synthesis_stage_input(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<serde_json::Value> {
    let rows = sqlx::query_as::<_, (i64, i64, String, Option<String>, Vec<u8>)>(
        "SELECT stages.id, snapshots.id, snapshots.source_ref_id, snapshots.title, artifacts.content_zstd
         FROM prompt_pack_run_source_snapshots snapshots
         JOIN prompt_pack_stage_runs stages
           ON stages.run_id = snapshots.run_id
          AND stages.source_snapshot_id = snapshots.id
          AND stages.stage_name = 'youtube_summary/transcript_analysis'
          AND stages.stage_status = 'succeeded'
         JOIN prompt_pack_stage_artifacts artifacts
           ON artifacts.stage_run_id = stages.id
          AND artifacts.artifact_kind = 'parsed_output'
          AND artifacts.id = (
              SELECT latest.id
              FROM prompt_pack_stage_artifacts latest
              WHERE latest.stage_run_id = stages.id
                AND latest.artifact_kind = 'parsed_output'
              ORDER BY latest.attempt_number DESC, latest.artifact_index DESC, latest.id DESC
              LIMIT 1
          )
         WHERE snapshots.run_id = ?
         ORDER BY snapshots.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut videos = Vec::new();
    let mut claim_candidates = Vec::new();
    let mut evidence_fragment_candidates = Vec::new();
    let mut warning_candidates = Vec::new();

    for (_stage_run_id, source_snapshot_id, source_ref_id, title, content_zstd) in rows {
        let text = decompress_text(&content_zstd).map_err(AppError::internal)?;
        let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
            AppError::internal(format!("parse transcript parsed_output: {error}"))
        })?;
        videos.push(serde_json::json!({
            "source_snapshot_id": source_snapshot_id,
            "source_ref_id": source_ref_id,
            "title": title,
            "video_candidate": parsed
                .get("video_candidate")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}))
        }));
        wrap_candidates(
            &mut claim_candidates,
            parsed.get("claim_candidates"),
            &source_ref_id,
        );
        wrap_candidates(
            &mut evidence_fragment_candidates,
            parsed.get("evidence_fragment_candidates"),
            &source_ref_id,
        );
        wrap_candidates(
            &mut warning_candidates,
            parsed.get("warning_candidates"),
            &source_ref_id,
        );
    }

    Ok(serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "run_id": run_id,
        "videos": videos,
        "claim_candidates": claim_candidates,
        "evidence_fragment_candidates": evidence_fragment_candidates,
        "warning_candidates": warning_candidates
    }))
}

fn wrap_candidates(
    target: &mut Vec<serde_json::Value>,
    value: Option<&serde_json::Value>,
    source_ref_id: &str,
) {
    if let Some(items) = value.and_then(serde_json::Value::as_array) {
        for item in items {
            target.push(serde_json::json!({
                "source_ref_id": source_ref_id,
                "candidate": item
            }));
        }
    }
}
