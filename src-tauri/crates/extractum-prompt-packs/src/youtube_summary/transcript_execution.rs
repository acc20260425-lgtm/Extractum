use sqlx::SqlitePool;

use crate::stage_io::insert_stage_artifact_in_pool;
use extractum_core::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub(crate) struct TranscriptStageRow {
    pub(crate) stage_run_id: i64,
    pub(crate) source_snapshot_id: i64,
    pub(crate) source_ref_id: String,
}

pub(crate) async fn load_pending_transcript_stage_rows(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<TranscriptStageRow>> {
    sqlx::query_as::<_, (i64, i64, String)>(
        "SELECT stages.id, snapshots.id, snapshots.source_ref_id
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.run_id = ?
           AND stages.stage_name = 'youtube_summary/transcript_analysis'
           AND stages.stage_status = 'pending'
         ORDER BY stages.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(stage_run_id, source_snapshot_id, source_ref_id)| TranscriptStageRow {
                    stage_run_id,
                    source_snapshot_id,
                    source_ref_id,
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

pub(crate) async fn mark_transcript_stage_failed(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    error: &str,
) -> AppResult<()> {
    mark_transcript_stage_failed_for_attempt(pool, run_id, stage_run_id, 1, error).await
}

pub(crate) async fn mark_transcript_stage_failed_for_attempt(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    attempt_number: i64,
    error: &str,
) -> AppResult<()> {
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "error",
        attempt_number,
        99,
        &serde_json::json!({ "error": error }).to_string(),
    )
    .await?;
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'failed',
             error_message = ?,
             latest_message = ?,
             completed_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(error)
    .bind(error)
    .bind(extractum_core::time::now_rfc3339_utc())
    .bind(extractum_core::time::now_rfc3339_utc())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_transcript_stage_cancelled(
    pool: &SqlitePool,
    stage_run_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'cancelled',
             latest_message = 'Cancelled',
             completed_at = COALESCE(completed_at, ?),
             updated_at = ?
         WHERE id = ? AND stage_status IN ('pending', 'running')",
    )
    .bind(extractum_core::time::now_rfc3339_utc())
    .bind(extractum_core::time::now_rfc3339_utc())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
