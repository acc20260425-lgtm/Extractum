use std::collections::HashSet;

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;

use super::dto::{
    PromptPackRunEvent, PromptPackRunSummaryDto, PromptPackStageRunDto,
    StartYoutubeSummaryRunOutcomeDto,
};
use super::youtube_summary::{
    preflight_youtube_summary_in_pool, start_youtube_summary_run_in_pool, ModelBudget,
};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};

pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";

#[derive(Default)]
pub struct PromptPackRunState {
    active: Mutex<HashSet<i64>>,
    cancel_requested: Mutex<HashSet<i64>>,
}

impl PromptPackRunState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn track(&self, run_id: i64) -> AppResult<()> {
        self.active.lock().await.insert(run_id);
        Ok(())
    }

    pub async fn request_cancel(&self, run_id: i64) -> AppResult<()> {
        self.cancel_requested.lock().await.insert(run_id);
        Ok(())
    }

    pub async fn is_cancel_requested(&self, run_id: i64) -> bool {
        self.cancel_requested.lock().await.contains(&run_id)
    }

    pub async fn finish(&self, run_id: i64) {
        self.active.lock().await.remove(&run_id);
        self.cancel_requested.lock().await.remove(&run_id);
    }

    pub async fn active_run_ids(&self) -> Vec<i64> {
        let mut ids = self.active.lock().await.iter().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    pub async fn apply_event(&self, event: PromptPackRunEvent) {
        if matches!(
            event.kind.as_str(),
            "completed" | "partial" | "failed" | "cancelled" | "interrupted"
        ) {
            self.finish(event.run_id).await;
        }
    }
}

#[tauri::command]
pub async fn preflight_youtube_summary_run(
    handle: AppHandle,
    project_id: Option<i64>,
    source_ids: Vec<i64>,
    profile_id: Option<String>,
    model_override: Option<String>,
    output_language: String,
    control_preset: String,
    evidence_mode: String,
    include_comments: bool,
) -> AppResult<super::dto::YoutubeSummaryPreflightResponse> {
    let pool = get_pool(&handle).await?;
    preflight_youtube_summary_in_pool(
        &pool,
        super::dto::PreflightYoutubeSummaryRunRequest {
            project_id,
            source_ids,
            profile_id,
            model_override,
            output_language,
            control_preset,
            evidence_mode,
            include_comments,
        },
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await
}

#[tauri::command]
pub async fn start_youtube_summary_run(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
    client_request_id: String,
    project_id: Option<i64>,
    source_ids: Vec<i64>,
    profile_id: Option<String>,
    model_override: Option<String>,
    output_language: String,
    control_preset: String,
    evidence_mode: String,
    include_comments: bool,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    let pool = get_pool(&handle).await?;
    let outcome = start_youtube_summary_run_in_pool(
        &pool,
        super::dto::StartYoutubeSummaryRunRequest {
            client_request_id,
            project_id,
            source_ids,
            profile_id,
            model_override,
            output_language,
            control_preset,
            evidence_mode,
            include_comments,
        },
    )
    .await?;
    if let StartYoutubeSummaryRunOutcomeDto::Started { run } = &outcome {
        state.track(run.run_id).await?;
        emit_prompt_pack_run_event(
            &handle,
            &state,
            PromptPackRunEvent {
                run_id: run.run_id,
                request_id: format!("run-{}", run.run_id),
                kind: "queued".to_string(),
                run_status: run.run_status.clone(),
                phase: "snapshot".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: run.queue_position,
                progress_current: run.progress_current,
                progress_total: run.progress_total,
                message: run.latest_message.clone(),
                error: None,
            },
        )
        .await;
    }
    Ok(outcome)
}

#[tauri::command]
pub async fn cancel_prompt_pack_run(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    state.request_cancel(run_id).await?;
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'cancelled', completed_at = COALESCE(completed_at, ?), updated_at = ?
         WHERE id = ? AND run_status IN ('queued', 'running')",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .execute(&pool)
    .await
    .map_err(AppError::database)?;
    emit_prompt_pack_run_event(
        &handle,
        &state,
        PromptPackRunEvent {
            run_id,
            request_id: format!("cancel-{run_id}"),
            kind: "cancelled".to_string(),
            run_status: "cancelled".to_string(),
            phase: "terminal".to_string(),
            stage_run_id: None,
            stage_name: None,
            source_snapshot_id: None,
            queue_position: None,
            progress_current: None,
            progress_total: None,
            message: Some("Cancelled".to_string()),
            error: None,
        },
    )
    .await;
    Ok(())
}

#[tauri::command]
pub async fn list_prompt_pack_runs(
    handle: AppHandle,
    project_id: Option<i64>,
    limit: Option<i64>,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let pool = get_pool(&handle).await?;
    list_prompt_pack_runs_in_pool(
        &pool,
        super::dto::ListPromptPackRunsRequest { project_id, limit },
    )
    .await
}

#[tauri::command]
pub async fn list_active_prompt_pack_runs(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let pool = get_pool(&handle).await?;
    let ids = state.active_run_ids().await;
    let mut runs = Vec::new();
    for run_id in ids {
        if let Some(run) = load_run_summary_optional(&pool, run_id).await? {
            runs.push(run);
        }
    }
    Ok(runs)
}

#[tauri::command]
pub async fn list_prompt_pack_run_stages(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Vec<PromptPackStageRunDto>> {
    let pool = get_pool(&handle).await?;
    list_prompt_pack_run_stages_in_pool(&pool, run_id).await
}

pub(crate) async fn cleanup_interrupted_prompt_pack_runs_in_pool(
    pool: &SqlitePool,
    state: &PromptPackRunState,
) -> AppResult<()> {
    let now = now_string();
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'interrupted', completed_at = COALESCE(completed_at, ?), updated_at = ?,
             latest_message = 'Interrupted during app shutdown'
         WHERE run_status IN ('queued', 'running')",
    )
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    for run_id in state.active_run_ids().await {
        state.finish(run_id).await;
    }
    Ok(())
}

pub async fn cleanup_interrupted_prompt_pack_runs(handle: AppHandle) {
    match get_pool(&handle).await {
        Ok(pool) => {
            let state = handle.state::<PromptPackRunState>();
            if let Err(error) = cleanup_interrupted_prompt_pack_runs_in_pool(&pool, &state).await {
                eprintln!("Prompt Pack cleanup failed: {error}");
            }
        }
        Err(error) => eprintln!("Prompt Pack cleanup skipped: {error}"),
    }
}

pub(crate) async fn list_prompt_pack_runs_in_pool(
    pool: &SqlitePool,
    request: super::dto::ListPromptPackRunsRequest,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let limit = request.limit.unwrap_or(20).clamp(1, 100);
    let rows = if let Some(project_id) = request.project_id {
        sqlx::query_as::<_, RunSummaryRow>(
            "SELECT id, project_id, pack_id, pack_version, run_status, result_status,
                    created_at, started_at, completed_at, latest_message,
                    progress_current, progress_total, queue_position
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
            "SELECT id, project_id, pack_id, pack_version, run_status, result_status,
                    created_at, started_at, completed_at, latest_message,
                    progress_current, progress_total, queue_position
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

async fn list_prompt_pack_run_stages_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<PromptPackStageRunDto>> {
    sqlx::query_as::<_, (i64, i64, Option<i64>, String, i64, String, Option<String>)>(
        "SELECT id, run_id, source_snapshot_id, stage_name, stage_order,
                stage_status, latest_message
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
                )| PromptPackStageRunDto {
                    stage_run_id,
                    run_id,
                    source_snapshot_id,
                    stage_name,
                    stage_order,
                    stage_status,
                    latest_message,
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

async fn load_run_summary_optional(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Option<PromptPackRunSummaryDto>> {
    sqlx::query_as::<_, RunSummaryRow>(
        "SELECT id, project_id, pack_id, pack_version, run_status, result_status,
                created_at, started_at, completed_at, latest_message,
                progress_current, progress_total, queue_position
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(Into::into))
    .map_err(AppError::database)
}

async fn emit_prompt_pack_run_event(
    handle: &AppHandle,
    state: &PromptPackRunState,
    event: PromptPackRunEvent,
) {
    state.apply_event(event.clone()).await;
    let _ = handle.emit(PROMPT_PACK_RUN_EVENT, event);
}

#[derive(sqlx::FromRow)]
struct RunSummaryRow {
    id: i64,
    project_id: Option<i64>,
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

fn now_string() -> String {
    "2026-06-14T00:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        cleanup_interrupted_prompt_pack_runs_in_pool, list_prompt_pack_runs_in_pool,
        PromptPackRunState,
    };
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::dto::{ListPromptPackRunsRequest, PromptPackRunEvent};
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;

    #[tokio::test]
    async fn prompt_pack_run_state_tracks_active_and_cancel_requested_runs() {
        let state = PromptPackRunState::new();

        state.track(42).await.expect("track");
        assert!(state.active_run_ids().await.contains(&42));

        state.request_cancel(42).await.expect("cancel");
        assert!(state.is_cancel_requested(42).await);

        state.finish(42).await;
        assert!(!state.active_run_ids().await.contains(&42));
    }

    #[tokio::test]
    async fn terminal_event_removes_run_from_active_state() {
        let state = PromptPackRunState::new();

        state.track(42).await.expect("track");
        state
            .apply_event(PromptPackRunEvent {
                run_id: 42,
                request_id: "req-42".to_string(),
                kind: "completed".to_string(),
                run_status: "complete".to_string(),
                phase: "terminal".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: None,
                progress_current: Some(1),
                progress_total: Some(1),
                message: Some("Completed".to_string()),
                error: None,
            })
            .await;

        assert!(!state.active_run_ids().await.contains(&42));
    }

    #[tokio::test]
    async fn cleanup_interrupted_prompt_pack_runs_marks_stale_active_rows_interrupted() {
        let pool = test_pool_with_prompt_pack_runs([
            (41, None, "queued", "2026-06-14T10:00:00Z"),
            (42, None, "running", "2026-06-14T11:00:00Z"),
            (43, None, "complete", "2026-06-14T12:00:00Z"),
        ])
        .await;
        let state = PromptPackRunState::new();

        cleanup_interrupted_prompt_pack_runs_in_pool(&pool, &state)
            .await
            .expect("cleanup");

        let statuses = list_run_statuses(&pool).await;
        assert_eq!(statuses.get(&41).map(String::as_str), Some("interrupted"));
        assert_eq!(statuses.get(&42).map(String::as_str), Some("interrupted"));
        assert_eq!(statuses.get(&43).map(String::as_str), Some("complete"));
    }

    #[tokio::test]
    async fn list_prompt_pack_runs_returns_recent_runs_for_project() {
        let pool = test_pool_with_prompt_pack_runs([
            (41, Some(7), "complete", "2026-06-14T10:00:00Z"),
            (42, Some(7), "running", "2026-06-14T11:00:00Z"),
            (43, Some(8), "complete", "2026-06-14T12:00:00Z"),
        ])
        .await;

        let runs = list_prompt_pack_runs_in_pool(
            &pool,
            ListPromptPackRunsRequest {
                project_id: Some(7),
                limit: Some(20),
            },
        )
        .await
        .expect("recent runs");

        assert_eq!(
            runs.iter().map(|run| run.run_id).collect::<Vec<_>>(),
            vec![42, 41]
        );
        assert!(runs.iter().all(|run| run.project_id == Some(7)));
    }

    async fn test_pool_with_prompt_pack_runs<const N: usize>(
        rows: [(i64, Option<i64>, &str, &str); N],
    ) -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        seed_builtin_prompt_packs_in_pool(&pool).await.expect("seed");
        for (run_id, project_id, status, created_at) in rows {
            if let Some(project_id) = project_id {
                sqlx::query(
                    "INSERT OR IGNORE INTO projects (id, name, created_at, updated_at)
                     VALUES (?, ?, 1, 1)",
                )
                .bind(project_id)
                .bind(format!("Project {project_id}"))
                .execute(&pool)
                .await
                .expect("insert project");
            }
            sqlx::query(
                "INSERT INTO prompt_pack_runs (
                    id, project_id, pack_version_id, pack_id, pack_version,
                    schema_version, run_status, result_status, output_language,
                    control_preset, evidence_mode, include_comments,
                    latest_message, created_at, updated_at
                 )
                 VALUES (?, ?, 1, 'youtube_summary', '1.0.0', '1.0',
                    ?, 'none', 'en', 'standard', 'standard', 0,
                    'Test run', ?, ?)",
            )
            .bind(run_id)
            .bind(project_id)
            .bind(status)
            .bind(created_at)
            .bind(created_at)
            .execute(&pool)
            .await
            .expect("insert run");
        }
        pool
    }

    async fn list_run_statuses(pool: &sqlx::SqlitePool) -> std::collections::HashMap<i64, String> {
        sqlx::query_as::<_, (i64, String)>(
            "SELECT id, run_status FROM prompt_pack_runs ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .expect("statuses")
        .into_iter()
        .collect()
    }
}
