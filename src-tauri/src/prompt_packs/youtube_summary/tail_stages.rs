use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

pub(crate) async fn mark_pending_mvp_tail_stages_skipped(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'skipped',
             latest_message = 'Handled by combined MVP stage',
             completed_at = ?,
             updated_at = ?
         WHERE run_id = ?
           AND stage_status = 'pending'
           AND stage_name NOT IN (
               'youtube_summary/transcript_analysis',
               'youtube_summary/synthesis'
           )",
    )
    .bind(crate::time::now_rfc3339_utc())
    .bind(crate::time::now_rfc3339_utc())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
