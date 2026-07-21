use sqlx::SqlitePool;

use super::dto::{ListPromptPackRunsRequest, PromptPackRunSummaryDto, PromptPackStageRunDto};
use extractum_core::error::{AppError, AppResult};

pub(super) async fn list_prompt_pack_runs_in_pool(
    pool: &SqlitePool,
    request: ListPromptPackRunsRequest,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let limit = request.limit.unwrap_or(20).clamp(1, 100);
    let rows = if let Some(project_id) = request.project_id {
        sqlx::query_as::<_, RunSummaryRow>(
            "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                    run_status, result_status, created_at, started_at, completed_at,
                    latest_message, progress_current, progress_total, queue_position
             FROM prompt_pack_runs
             WHERE project_id = ?
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?
    } else {
        sqlx::query_as::<_, RunSummaryRow>(
            "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                    run_status, result_status, created_at, started_at, completed_at,
                    latest_message, progress_current, progress_total, queue_position
             FROM prompt_pack_runs
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?
    };
    Ok(rows.into_iter().map(Into::into).collect())
}

pub(super) async fn update_prompt_pack_run_in_pool(
    pool: &SqlitePool,
    run_id: i64,
    run_label: Option<String>,
) -> AppResult<PromptPackRunSummaryDto> {
    let normalized_label = normalize_prompt_pack_run_label(run_label);
    let result = sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_label = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&normalized_label)
    .bind(extractum_core::time::now_rfc3339_utc())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Prompt Pack run {run_id} not found"
        )));
    }

    load_run_summary_optional(pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Prompt Pack run {run_id} not found")))
}

pub(super) async fn delete_prompt_pack_run_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<()> {
    let status =
        sqlx::query_scalar::<_, String>("SELECT run_status FROM prompt_pack_runs WHERE id = ?")
            .bind(run_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?
            .ok_or_else(|| AppError::not_found(format!("Prompt Pack run {run_id} not found")))?;

    if status == "queued" || status == "running" {
        return Err(AppError::conflict(
            "Queued or running Prompt Pack runs cannot be deleted",
        ));
    }

    sqlx::query("DELETE FROM prompt_pack_runs WHERE id = ?")
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

fn normalize_prompt_pack_run_label(label: Option<String>) -> Option<String> {
    label
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) async fn list_prompt_pack_run_stages_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<PromptPackStageRunDto>> {
    sqlx::query_as::<
        _,
        (
            i64,
            i64,
            Option<i64>,
            String,
            i64,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT id, run_id, source_snapshot_id, stage_name, stage_order,
                stage_status, latest_message, browser_run_id, browser_run_status,
                browser_completion_reason, browser_provider_mode, browser_run_message
         FROM prompt_pack_stage_runs
         WHERE run_id = ?
         ORDER BY stage_order ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(
                    stage_run_id,
                    run_id,
                    source_snapshot_id,
                    stage_name,
                    stage_order,
                    stage_status,
                    latest_message,
                    browser_run_id,
                    browser_run_status,
                    browser_completion_reason,
                    browser_provider_mode,
                    browser_run_message,
                )| PromptPackStageRunDto {
                    stage_run_id,
                    run_id,
                    source_snapshot_id,
                    stage_name,
                    stage_order,
                    stage_status,
                    latest_message,
                    browser_run_id,
                    browser_run_status,
                    browser_completion_reason,
                    browser_provider_mode,
                    browser_run_message,
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

pub(super) async fn load_run_summary_optional(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Option<PromptPackRunSummaryDto>> {
    sqlx::query_as::<_, RunSummaryRow>(
        "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                run_status, result_status, created_at, started_at, completed_at,
                latest_message, progress_current, progress_total, queue_position
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(Into::into))
    .map_err(AppError::database)
}

#[derive(sqlx::FromRow)]
struct RunSummaryRow {
    id: i64,
    project_id: Option<i64>,
    run_label: Option<String>,
    runtime_provider: String,
    pack_id: String,
    pack_version: String,
    run_status: String,
    result_status: String,
    created_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    latest_message: Option<String>,
    progress_current: Option<i64>,
    progress_total: Option<i64>,
    queue_position: Option<i64>,
}

impl From<RunSummaryRow> for PromptPackRunSummaryDto {
    fn from(row: RunSummaryRow) -> Self {
        Self {
            run_id: row.id,
            project_id: row.project_id,
            run_label: row.run_label,
            runtime_provider: row.runtime_provider,
            pack_id: row.pack_id,
            pack_version: row.pack_version,
            run_status: row.run_status,
            result_status: row.result_status,
            created_at: row.created_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
            latest_message: row.latest_message,
            progress_current: row.progress_current,
            progress_total: row.progress_total,
            queue_position: row.queue_position,
        }
    }
}
