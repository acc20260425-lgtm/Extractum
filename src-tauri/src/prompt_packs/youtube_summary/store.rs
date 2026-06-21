use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::prompt_packs::dto::PromptPackRunSummaryDto;

pub(crate) async fn ensure_pack_version(pool: &SqlitePool) -> AppResult<i64> {
    if let Some(id) = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM prompt_pack_versions
         WHERE pack_id = 'youtube_summary' AND pack_version = '1.0.0'",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    {
        return Ok(id);
    }

    crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool(pool).await?;
    crate::prompt_packs::store::require_prompt_pack_version_id(pool, "youtube_summary", "1.0.0")
        .await
}

pub(crate) async fn load_run_by_client_request_id(
    pool: &SqlitePool,
    client_request_id: &str,
) -> AppResult<Option<PromptPackRunSummaryDto>> {
    let run_id =
        sqlx::query_scalar::<_, i64>("SELECT id FROM prompt_pack_runs WHERE client_request_id = ?")
            .bind(client_request_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?;
    match run_id {
        Some(run_id) => Ok(Some(load_run_summary(pool, run_id).await?)),
        None => Ok(None),
    }
}

pub(crate) async fn load_run_summary(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<PromptPackRunSummaryDto> {
    sqlx::query_as::<
        _,
        (
            i64,
            Option<i64>,
            Option<String>,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
        ),
    >(
        "SELECT id, project_id, run_label, pack_id, pack_version, run_status, result_status,
                created_at, started_at, completed_at, latest_message,
                progress_current, progress_total, queue_position
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map(
        |(
            run_id,
            project_id,
            run_label,
            pack_id,
            pack_version,
            run_status,
            result_status,
            created_at,
            started_at,
            completed_at,
            latest_message,
            progress_current,
            progress_total,
            queue_position,
        )| PromptPackRunSummaryDto {
            run_id,
            project_id,
            run_label,
            runtime_provider: "api".to_string(),
            pack_id,
            pack_version,
            run_status,
            result_status,
            created_at,
            started_at,
            completed_at,
            latest_message,
            progress_current,
            progress_total,
            queue_position,
        },
    )
    .map_err(AppError::database)
}
