use tauri::AppHandle;

use crate::error::AppResult;
use crate::sources::SourceIdentityRepairState;

use super::jobs::{self, SourceJobListFilter, SourceJobRecord, SourceJobState, YoutubeSyncOptions};

#[tauri::command]
pub(crate) async fn sync_youtube_source(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, SourceJobState>,
    source_id: i64,
    options: YoutubeSyncOptions,
) -> AppResult<SourceJobRecord> {
    jobs::start_youtube_source_job(
        handle,
        repair_state.inner(),
        state.inner(),
        source_id,
        options,
    )
    .await
}

#[tauri::command]
pub(crate) async fn sync_youtube_playlist_video(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, SourceJobState>,
    playlist_source_id: i64,
    video_source_id: i64,
    options: YoutubeSyncOptions,
) -> AppResult<SourceJobRecord> {
    jobs::start_youtube_playlist_video_job(
        handle,
        repair_state.inner(),
        state.inner(),
        playlist_source_id,
        video_source_id,
        options,
    )
    .await
}

#[tauri::command]
pub(crate) async fn retry_failed_youtube_playlist_videos(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, SourceJobState>,
    source_id: i64,
    _options: YoutubeSyncOptions,
) -> AppResult<SourceJobRecord> {
    jobs::start_failed_youtube_playlist_video_retry_job(
        handle,
        repair_state.inner(),
        state.inner(),
        source_id,
    )
    .await
}

#[tauri::command]
pub(crate) async fn cancel_source_job(
    handle: AppHandle,
    state: tauri::State<'_, SourceJobState>,
    job_id: String,
) -> AppResult<()> {
    jobs::request_source_job_cancel(&handle, state.inner(), &job_id).await
}

#[tauri::command]
pub(crate) async fn list_source_jobs(
    state: tauri::State<'_, SourceJobState>,
    filter: SourceJobListFilter,
) -> AppResult<Vec<SourceJobRecord>> {
    jobs::list_source_job_records(state.inner(), filter).await
}

#[cfg(debug_assertions)]
#[tauri::command]
pub(crate) async fn seed_source_job_cancellation_smoke_fixture(
    handle: AppHandle,
    state: tauri::State<'_, SourceJobState>,
) -> AppResult<SourceJobRecord> {
    jobs::seed_source_job_cancellation_smoke_fixture(handle, state.inner()).await
}

#[cfg(debug_assertions)]
#[tauri::command]
pub(crate) async fn clear_source_job_cancellation_smoke_fixture(
    state: tauri::State<'_, SourceJobState>,
) -> AppResult<usize> {
    jobs::clear_source_job_cancellation_smoke_fixture(state.inner()).await
}
