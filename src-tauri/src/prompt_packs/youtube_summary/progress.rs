use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

pub(crate) async fn mark_run_running(pool: &SqlitePool, run_id: i64, total: i64) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'running',
             started_at = COALESCE(started_at, ?),
             latest_message = 'Running',
             progress_current = COALESCE(progress_current, 0),
             progress_total = ?,
             updated_at = ?
         WHERE id = ? AND run_status = 'queued'",
    )
    .bind(now_string())
    .bind(total)
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn is_run_cancelled(pool: &SqlitePool, run_id: i64) -> AppResult<bool> {
    sqlx::query_scalar::<_, String>("SELECT run_status FROM prompt_pack_runs WHERE id = ?")
        .bind(run_id)
        .fetch_one(pool)
        .await
        .map(|status| status == "cancelled")
        .map_err(AppError::database)
}

pub(crate) async fn update_run_progress(
    pool: &SqlitePool,
    run_id: i64,
    successes: i64,
    total: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET progress_current = ?,
             progress_total = ?,
             latest_message = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(successes)
    .bind(total)
    .bind(format!("Processed {successes} of {total} video(s)"))
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_run_cancelled(
    pool: &SqlitePool,
    run_id: i64,
    progress_current: i64,
    progress_total: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'cancelled',
             latest_message = 'Cancelled',
             progress_current = ?,
             progress_total = ?,
             completed_at = COALESCE(completed_at, ?),
             updated_at = ?
         WHERE id = ?",
    )
    .bind(progress_current)
    .bind(progress_total)
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn now_string() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
