use serde::Serialize;
use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager, State};

use super::store::set_run_status;
use super::AnalysisState;
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::time::now_secs;

mod seed;

use self::seed::seed_analysis_redesign_fixtures_in_pool;

const FIXTURE_MARKER: &str = "__analysis_redesign_fixture__";
const FIXTURE_EXTERNAL_PREFIX: &str = "__analysis_redesign_fixture__:";
const FIXTURE_PROFILE_ID: &str = "__analysis_redesign_fixture__";
const FIXTURE_NOW: i64 = 1_778_400_000;
const FIXTURE_PERIOD_FROM: i64 = 1_777_968_000;
const FIXTURE_PERIOD_TO: i64 = 1_778_313_600;

const TELEGRAM_CHANNEL_LABEL: &str = "__analysis_redesign_fixture__ Telegram Channel";
const TELEGRAM_SUPERGROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Supergroup";
const YOUTUBE_VIDEO_LABEL: &str = "__analysis_redesign_fixture__ YouTube Video";
const YOUTUBE_PLAYLIST_LABEL: &str = "__analysis_redesign_fixture__ YouTube Playlist";
const YOUTUBE_FIXTURE_VIDEO_ID: &str = "analysis_fixture_video";
const YOUTUBE_FIXTURE_PLAYLIST_ID: &str = "PLanalysisfixture";
const TELEGRAM_FIXTURE_CHANNEL_PEER_ID: i64 = 10_000_001;
const TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID: i64 = 10_000_002;
const TELEGRAM_GROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Source Group";
const COMPLETED_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Completed Snapshot Run";
const MISSING_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Missing Snapshot Run";
const CAPTURE_FAILED_SNAPSHOT_RUN_LABEL: &str =
    "__analysis_redesign_fixture__ Capture Failed Snapshot Run";
const CAPTURE_FAILED_SNAPSHOT_ERROR: &str =
    "Snapshot capture failed: fixture write boundary unavailable";
const CANCELLED_RUN_MESSAGE: &str = "Analysis run cancelled.";
const RUNNING_RUN_LABEL: &str = "__analysis_redesign_fixture__ Running Run";
const FAILED_RUN_LABEL: &str = "__analysis_redesign_fixture__ Failed Run";
const CANCELLED_RUN_LABEL: &str = "__analysis_redesign_fixture__ Cancelled Run";
const GROUP_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Group Snapshot Run";
const LLM_PROFILE_LABEL: &str = "__analysis_redesign_fixture__ LLM Profile";
const FIXTURE_SNAPSHOT_CAPTURED_AT: &str = "2026-05-18T10:00:00Z";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisRedesignFixtureSummary {
    pub accounts: i64,
    pub llm_profiles: i64,
    pub sources: i64,
    pub source_groups: i64,
    pub prompt_templates: i64,
    pub runs: i64,
    pub snapshot_messages: i64,
    pub chat_messages: i64,
    pub youtube_transcript_segments: i64,
    pub youtube_playlist_items: i64,
}

#[tauri::command]
pub async fn seed_analysis_redesign_fixtures(
    handle: AppHandle,
    state: State<'_, AnalysisState>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let pool = get_pool(&handle).await?;
    let previous_run_ids = fixture_run_ids(&pool).await?;
    remove_fixture_active_runs(state.inner(), &previous_run_ids).await;
    let summary = seed_analysis_redesign_fixtures_in_pool(&pool).await?;
    let active_run_ids = register_fixture_active_runs(&pool, state.inner()).await?;
    spawn_fixture_cancellation_waiters(handle, active_run_ids).await;
    Ok(summary)
}

#[tauri::command]
pub async fn clear_analysis_redesign_fixtures(
    handle: AppHandle,
    state: State<'_, AnalysisState>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let pool = get_pool(&handle).await?;
    let run_ids = fixture_run_ids(&pool).await?;
    remove_fixture_active_runs(state.inner(), &run_ids).await;
    clear_analysis_redesign_fixtures_in_pool(&pool).await
}

#[tauri::command]
pub async fn clear_analysis_redesign_fixture_active_runs(
    handle: AppHandle,
    state: State<'_, AnalysisState>,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let run_ids = fixture_run_ids(&pool).await?;
    remove_fixture_active_runs(state.inner(), &run_ids).await;
    Ok(())
}

async fn fixture_run_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    let marker_pattern = format!("{FIXTURE_MARKER}%");
    sqlx::query_scalar(
        "SELECT id FROM analysis_runs
         WHERE scope_label_snapshot LIKE ? OR provider_profile = ?",
    )
    .bind(marker_pattern)
    .bind(FIXTURE_PROFILE_ID)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn register_fixture_active_runs(
    pool: &Pool<Sqlite>,
    state: &AnalysisState,
) -> AppResult<Vec<i64>> {
    let run_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM analysis_runs
         WHERE scope_label_snapshot = ? AND status = 'running'",
    )
    .bind(RUNNING_RUN_LABEL)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    for run_id in &run_ids {
        state.insert_active_report_run(*run_id).await;
    }

    Ok(run_ids)
}

async fn remove_fixture_active_runs(state: &AnalysisState, run_ids: &[i64]) {
    for run_id in run_ids {
        state.request_report_run_cancel(*run_id).await;
        state.remove_active_report_run(*run_id).await;
    }
}

async fn finish_cancelled_fixture_run(
    pool: &Pool<Sqlite>,
    state: &AnalysisState,
    run_id: i64,
) -> AppResult<()> {
    set_run_status(
        pool,
        run_id,
        crate::analysis::ANALYSIS_STATUS_CANCELLED,
        None,
        None,
        Some(CANCELLED_RUN_MESSAGE),
        Some(now_secs()),
    )
    .await?;
    state.remove_active_report_run(run_id).await;
    Ok(())
}

async fn spawn_fixture_cancellation_waiters(handle: AppHandle, run_ids: Vec<i64>) {
    for run_id in run_ids {
        let state = handle.state::<AnalysisState>();
        let Some(token) = state.report_run_child_token(run_id).await else {
            continue;
        };
        let task_handle = handle.clone();
        tauri::async_runtime::spawn(async move {
            token.cancelled().await;
            let Ok(pool) = get_pool(&task_handle).await else {
                return;
            };
            let state = task_handle.state::<AnalysisState>();
            let _ = finish_cancelled_fixture_run(&pool, state.inner(), run_id).await;
        });
    }
}

async fn clear_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let marker_pattern = format!("{FIXTURE_MARKER}%");
    let external_pattern = format!("{FIXTURE_EXTERNAL_PREFIX}%");
    let profile_settings_pattern = format!("llm.profile.{FIXTURE_PROFILE_ID}.%");

    let mut summary = AnalysisRedesignFixtureSummary::default();

    summary.chat_messages = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_chat_messages
             WHERE run_id IN (
                SELECT id FROM analysis_runs
                WHERE scope_label_snapshot LIKE ? OR provider_profile = ?
             )",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.snapshot_messages = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_run_messages
             WHERE run_id IN (
                SELECT id FROM analysis_runs
                WHERE scope_label_snapshot LIKE ? OR provider_profile = ?
             )",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.runs = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_runs
             WHERE scope_label_snapshot LIKE ? OR provider_profile = ?",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    let fixture_profile_setting_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM app_settings WHERE key LIKE ?")
            .bind(&profile_settings_pattern)
            .fetch_one(&mut *tx)
            .await
            .map_err(AppError::database)?;
    sqlx::query("DELETE FROM app_settings WHERE key LIKE ?")
        .bind(&profile_settings_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    summary.llm_profiles = if fixture_profile_setting_count > 0 {
        1
    } else {
        0
    };

    summary.prompt_templates = rows_to_i64(
        sqlx::query("DELETE FROM analysis_prompt_templates WHERE name LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    sqlx::query(
        "DELETE FROM analysis_source_group_members
         WHERE group_id IN (SELECT id FROM analysis_source_groups WHERE name LIKE ?)
            OR source_id IN (
                SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
            )",
    )
    .bind(&marker_pattern)
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    summary.source_groups = rows_to_i64(
        sqlx::query("DELETE FROM analysis_source_groups WHERE name LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    summary.youtube_playlist_items = rows_to_i64(
        sqlx::query(
            "DELETE FROM youtube_playlist_items
             WHERE playlist_source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR video_source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR video_id LIKE ?",
        )
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&external_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.youtube_transcript_segments = rows_to_i64(
        sqlx::query(
            "DELETE FROM youtube_transcript_segments
             WHERE source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR item_id IN (
                    SELECT items.id
                    FROM items
                    JOIN sources ON sources.id = items.source_id
                    WHERE sources.title LIKE ? OR sources.external_id LIKE ?
                )",
        )
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    sqlx::query(
        "DELETE FROM telegram_forum_topics
         WHERE source_id IN (
            SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
         )",
    )
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        "DELETE FROM items
         WHERE source_id IN (
            SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
         )",
    )
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    summary.sources = rows_to_i64(
        sqlx::query("DELETE FROM sources WHERE title LIKE ? OR external_id LIKE ?")
            .bind(&marker_pattern)
            .bind(&external_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    summary.accounts = rows_to_i64(
        sqlx::query("DELETE FROM accounts WHERE label LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    tx.commit().await.map_err(AppError::database)?;
    Ok(summary)
}

fn rows_to_i64(rows: u64) -> i64 {
    rows as i64
}

#[cfg(test)]
mod tests;
