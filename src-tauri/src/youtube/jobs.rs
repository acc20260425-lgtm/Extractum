use std::collections::{HashMap, HashSet};
use std::future::Future;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::job_helpers::{ActiveJobGuards, CancellationState};
use crate::secret_store::SecretStoreState;
use crate::sources::{
    load_source, require_source_identity_ready, upsert_youtube_comment_item,
    upsert_youtube_playlist_source, upsert_youtube_transcript_item, upsert_youtube_video_source,
    SourceIdentityRepairState, SourceSyncTarget,
};
use crate::time::{now_secs, ymd_to_unix_midnight};

use super::captions::{
    fetch_transcript_for_video, replace_transcript_segments, transcript_external_id,
};
use super::comments::{fetch_comments_for_video, DEFAULT_MAX_COMMENTS_PER_VIDEO};
use super::dto::{YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata};
use super::metadata::{fetch_playlist_metadata, fetch_video_metadata};
use super::playlist::upsert_playlist_items;
use super::settings::load_youtube_auth_cookies_from_state;
use super::source_metadata::{
    load_playlist_source_metadata_map, load_video_source_metadata_map, YoutubeVideoSourceMetadata,
};

pub(crate) const SOURCE_JOB_EVENT: &str = "sources://source-job";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct YoutubeSyncOptions {
    pub(crate) metadata: bool,
    pub(crate) transcripts: bool,
    pub(crate) comments: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    CancelRequested,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub(crate) enum SourceJobType {
    YoutubeVideoMetadataSync,
    YoutubeVideoTranscriptSync,
    YoutubeVideoCommentsSync,
    YoutubeVideoFullSync,
    YoutubePlaylistMetadataSync,
    YoutubePlaylistFullSync,
    YoutubePlaylistVideoSync,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SourceJobRecord {
    pub(crate) job_id: String,
    pub(crate) source_id: i64,
    pub(crate) related_source_id: Option<i64>,
    pub(crate) job_type: SourceJobType,
    pub(crate) status: SourceJobStatus,
    pub(crate) message: Option<String>,
    pub(crate) progress_current: Option<i64>,
    pub(crate) progress_total: Option<i64>,
    pub(crate) started_at: i64,
    pub(crate) finished_at: Option<i64>,
    pub(crate) warnings: Vec<String>,
    pub(crate) error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceJobDiagnosticCount {
    pub(crate) job_type: String,
    pub(crate) status: String,
    pub(crate) warning_state: String,
    pub(crate) error_kind: String,
    pub(crate) count: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceJobListFilter {
    pub source_id: Option<i64>,
    pub status: Option<SourceJobStatus>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SourceJobKey {
    source_id: i64,
    job_type: SourceJobType,
    related_source_id: Option<i64>,
}

#[derive(Default)]
struct SourceJobStateInner {
    next_job_id: u64,
    jobs: HashMap<String, SourceJobRecord>,
    active_jobs: ActiveJobGuards<SourceJobKey>,
    options_by_job_id: HashMap<String, YoutubeSyncOptions>,
    cancel_requested: CancellationState,
}

pub(crate) struct SourceJobState {
    inner: Mutex<SourceJobStateInner>,
}

impl SourceJobState {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(SourceJobStateInner::default()),
        }
    }

    pub(crate) async fn create_job(
        &self,
        source_id: i64,
        job_type: SourceJobType,
        related_source_id: Option<i64>,
        options: YoutubeSyncOptions,
    ) -> AppResult<SourceJobRecord> {
        let key = SourceJobKey {
            source_id,
            job_type: job_type.clone(),
            related_source_id,
        };
        let mut inner = self.inner.lock().await;
        if let Some(job_id) = inner.active_jobs.active_job_id(&key) {
            return Err(AppError::conflict(format!(
                "Source job scope already has active job {job_id}"
            )));
        }

        inner.next_job_id += 1;
        let job_id = format!("source-job-{}", inner.next_job_id);
        let record = SourceJobRecord {
            job_id: job_id.clone(),
            source_id,
            related_source_id,
            job_type,
            status: SourceJobStatus::Queued,
            message: Some("Source job queued.".to_string()),
            progress_current: None,
            progress_total: None,
            started_at: now_secs(),
            finished_at: None,
            warnings: Vec::new(),
            error: None,
        };

        inner.active_jobs.track(key, job_id.clone());
        inner.options_by_job_id.insert(job_id.clone(), options);
        inner.jobs.insert(job_id, record.clone());
        Ok(record)
    }

    pub(crate) async fn job_options(&self, job_id: &str) -> Option<YoutubeSyncOptions> {
        self.inner
            .lock()
            .await
            .options_by_job_id
            .get(job_id)
            .cloned()
    }

    pub(crate) async fn list_jobs(&self, filter: SourceJobListFilter) -> Vec<SourceJobRecord> {
        let limit = filter.limit.unwrap_or(100).min(500);
        let mut jobs = self
            .inner
            .lock()
            .await
            .jobs
            .values()
            .filter(|job| {
                filter
                    .source_id
                    .is_none_or(|source_id| job.source_id == source_id)
            })
            .filter(|job| {
                filter
                    .status
                    .as_ref()
                    .is_none_or(|status| job.status == *status)
            })
            .cloned()
            .collect::<Vec<_>>();

        jobs.sort_by(|a, b| {
            b.started_at
                .cmp(&a.started_at)
                .then_with(|| b.job_id.cmp(&a.job_id))
        });
        jobs.truncate(limit);
        jobs
    }

    pub(crate) async fn active_jobs_for_sources(&self, source_ids: &[i64]) -> Vec<SourceJobRecord> {
        let source_ids = source_ids.iter().copied().collect::<HashSet<_>>();
        let mut jobs = self
            .inner
            .lock()
            .await
            .jobs
            .values()
            .filter(|job| {
                source_ids.contains(&job.source_id)
                    || job
                        .related_source_id
                        .is_some_and(|source_id| source_ids.contains(&source_id))
            })
            .filter(|job| !is_terminal_status(&job.status))
            .cloned()
            .collect::<Vec<_>>();
        jobs.sort_by(|a, b| {
            a.started_at
                .cmp(&b.started_at)
                .then_with(|| a.job_id.cmp(&b.job_id))
        });
        jobs
    }

    pub(crate) async fn catalog_jobs_for_sources(
        &self,
        source_ids: &[i64],
    ) -> Vec<SourceJobRecord> {
        let source_ids = source_ids.iter().copied().collect::<HashSet<_>>();
        let mut jobs = self
            .inner
            .lock()
            .await
            .jobs
            .values()
            .filter(|job| {
                source_ids.contains(&job.source_id)
                    || job
                        .related_source_id
                        .is_some_and(|source_id| source_ids.contains(&source_id))
            })
            .filter(|job| {
                matches!(
                    &job.status,
                    SourceJobStatus::Queued | SourceJobStatus::Running | SourceJobStatus::Failed
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        jobs.sort_by(|a, b| {
            b.started_at
                .cmp(&a.started_at)
                .then_with(|| b.job_id.cmp(&a.job_id))
        });
        jobs
    }

    pub(crate) async fn diagnostic_counts(&self) -> Vec<SourceJobDiagnosticCount> {
        let inner = self.inner.lock().await;
        let mut counts = std::collections::BTreeMap::<(String, String, String, String), i64>::new();
        for job in inner.jobs.values() {
            let key = (
                source_job_type_diagnostic_key(&job.job_type).to_string(),
                source_job_status_diagnostic_key(&job.status).to_string(),
                if job.warnings.is_empty() {
                    "none".to_string()
                } else {
                    "present".to_string()
                },
                job.error
                    .as_deref()
                    .map(classify_diagnostic_error_kind)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "none".to_string()),
            );
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .map(|((job_type, status, warning_state, error_kind), count)| {
                SourceJobDiagnosticCount {
                    job_type,
                    status,
                    warning_state,
                    error_kind,
                    count,
                }
            })
            .collect()
    }

    pub(crate) async fn request_cancel(&self, job_id: &str) -> Option<SourceJobRecord> {
        let mut inner = self.inner.lock().await;
        if is_terminal_status(&inner.jobs.get(job_id)?.status) {
            return None;
        }

        inner.cancel_requested.request(job_id);
        let job = inner.jobs.get_mut(job_id)?;
        job.status = SourceJobStatus::CancelRequested;
        job.message = Some("Cancel requested.".to_string());
        Some(job.clone())
    }

    pub(crate) async fn is_cancel_requested(&self, job_id: &str) -> bool {
        self.inner
            .lock()
            .await
            .cancel_requested
            .is_requested(job_id)
    }

    pub(crate) async fn cancellation_token(&self, job_id: &str) -> Option<CancellationToken> {
        let mut inner = self.inner.lock().await;
        let job = inner.jobs.get(job_id)?;
        if is_terminal_status(&job.status) {
            return None;
        }
        Some(inner.cancel_requested.child_token(job_id))
    }

    pub(crate) async fn update_job<F>(&self, job_id: &str, update: F) -> Option<SourceJobRecord>
    where
        F: FnOnce(&mut SourceJobRecord),
    {
        let mut inner = self.inner.lock().await;
        let job = inner.jobs.get_mut(job_id)?;
        update(job);
        Some(job.clone())
    }

    pub(crate) async fn finish_job<F>(&self, job_id: &str, update: F) -> Option<SourceJobRecord>
    where
        F: FnOnce(&mut SourceJobRecord),
    {
        let mut inner = self.inner.lock().await;
        let cancel_requested = inner.cancel_requested.is_requested(job_id);
        {
            let job = inner.jobs.get_mut(job_id)?;
            update(job);
            if cancel_requested {
                job.status = SourceJobStatus::Cancelled;
                job.message = Some("Source job cancelled.".to_string());
                job.error = None;
            }
            job.finished_at = Some(now_secs());
        }
        inner.active_jobs.release_by_job_id(job_id);
        inner.options_by_job_id.remove(job_id);
        inner.cancel_requested.clear(job_id);
        inner.jobs.get(job_id).cloned()
    }
}

pub(crate) async fn start_youtube_source_job(
    handle: AppHandle,
    repair_state: &SourceIdentityRepairState,
    state: &SourceJobState,
    source_id: i64,
    options: YoutubeSyncOptions,
) -> AppResult<SourceJobRecord> {
    require_source_identity_ready(repair_state).await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    ensure_youtube_source(&source)?;
    let job_type = source_job_type_for_source_options(source.source_subtype.as_deref(), &options);
    let record = state.create_job(source_id, job_type, None, options).await?;
    emit_source_job_event(&handle, &record);

    let job_id = record.job_id.clone();
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_source_job(task_handle, job_id).await;
    });

    Ok(record)
}

pub(crate) async fn start_youtube_playlist_video_job(
    handle: AppHandle,
    repair_state: &SourceIdentityRepairState,
    state: &SourceJobState,
    playlist_source_id: i64,
    video_source_id: i64,
    options: YoutubeSyncOptions,
) -> AppResult<SourceJobRecord> {
    require_source_identity_ready(repair_state).await?;
    let record = state
        .create_job(
            playlist_source_id,
            SourceJobType::YoutubePlaylistVideoSync,
            Some(video_source_id),
            options,
        )
        .await?;
    emit_source_job_event(&handle, &record);

    let job_id = record.job_id.clone();
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_source_job(task_handle, job_id).await;
    });

    Ok(record)
}

pub(crate) async fn start_failed_youtube_playlist_video_retry_job(
    handle: AppHandle,
    repair_state: &SourceIdentityRepairState,
    state: &SourceJobState,
    source_id: i64,
) -> AppResult<SourceJobRecord> {
    require_source_identity_ready(repair_state).await?;
    let retry_options = YoutubeSyncOptions {
        metadata: false,
        transcripts: true,
        comments: false,
    };
    let record = state
        .create_job(
            source_id,
            SourceJobType::YoutubePlaylistFullSync,
            None,
            retry_options,
        )
        .await?;
    emit_source_job_event(&handle, &record);

    let job_id = record.job_id.clone();
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_retry_playlist_job(task_handle, job_id).await;
    });

    Ok(record)
}

pub(crate) async fn request_source_job_cancel(
    handle: &AppHandle,
    state: &SourceJobState,
    job_id: &str,
) -> AppResult<()> {
    if let Some(record) = state.request_cancel(job_id).await {
        emit_source_job_event(handle, &record);
    }
    Ok(())
}

pub(crate) async fn list_source_job_records(
    state: &SourceJobState,
    filter: SourceJobListFilter,
) -> AppResult<Vec<SourceJobRecord>> {
    Ok(state.list_jobs(filter).await)
}

pub(crate) async fn retryable_playlist_video_rows(
    pool: &sqlx::SqlitePool,
    playlist_source_id: i64,
) -> AppResult<Vec<RetryablePlaylistVideoRow>> {
    sqlx::query_as(
        r#"
        SELECT video_id, video_source_id
        FROM youtube_playlist_items
        WHERE playlist_source_id = ?
          AND is_removed_from_playlist = 0
          AND availability_status IN (
              'live_ended_transcript_pending',
              'no_captions',
              'unavailable_unknown'
          )
        ORDER BY COALESCE(position, id), id
        "#,
    )
    .bind(playlist_source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct RetryablePlaylistVideoRow {
    #[allow(dead_code)]
    pub(crate) video_id: String,
    pub(crate) video_source_id: Option<i64>,
}

async fn run_source_job(handle: AppHandle, job_id: String) {
    let state = handle.state::<SourceJobState>();
    let Some(record) = state
        .update_job(&job_id, |job| {
            job.status = SourceJobStatus::Running;
            job.message = Some("Source job running.".to_string());
        })
        .await
    else {
        return;
    };
    emit_source_job_event(&handle, &record);

    if state.is_cancel_requested(&job_id).await {
        finish_cancelled_job(&handle, &state, &job_id).await;
        return;
    }

    let options = state
        .job_options(&job_id)
        .await
        .unwrap_or(YoutubeSyncOptions {
            metadata: false,
            transcripts: false,
            comments: false,
        });
    let result = run_source_job_steps(
        &handle,
        &state,
        &job_id,
        record.source_id,
        record.related_source_id,
        &options,
    )
    .await;

    match result {
        Ok(warnings) => {
            if let Some(record) = state
                .finish_job(&job_id, |job| {
                    job.status = SourceJobStatus::Succeeded;
                    job.message = Some("Source job completed.".to_string());
                    job.warnings = warnings;
                })
                .await
            {
                emit_source_job_event(&handle, &record);
            }
        }
        Err(error) if state.is_cancel_requested(&job_id).await => {
            finish_cancelled_job(&handle, &state, &job_id).await;
            let _ = error;
        }
        Err(error) => {
            if let Some(record) = state
                .finish_job(&job_id, |job| {
                    job.status = SourceJobStatus::Failed;
                    job.message = None;
                    job.error = Some(error.to_string());
                })
                .await
            {
                emit_source_job_event(&handle, &record);
            }
        }
    }
}

async fn run_source_job_steps(
    handle: &AppHandle,
    state: &SourceJobState,
    job_id: &str,
    source_id: i64,
    related_source_id: Option<i64>,
    options: &YoutubeSyncOptions,
) -> AppResult<Vec<String>> {
    let repair_state = handle.state::<SourceIdentityRepairState>();
    require_source_identity_ready(repair_state.inner()).await?;
    let mut warnings = Vec::new();
    let sync_source_id = related_source_id.unwrap_or(source_id);
    let cancellation_token = state.cancellation_token(job_id).await;
    if options.metadata {
        update_and_emit_source_job(handle, state, job_id, |job| {
            job.message = Some("Refreshing YouTube metadata.".to_string());
        })
        .await;
        run_source_job_step_with_cancel(
            cancellation_token.clone(),
            sync_youtube_metadata(handle, sync_source_id),
        )
        .await?;
    }

    if state.is_cancel_requested(job_id).await {
        return Err(AppError::validation("Source job cancelled"));
    }

    if options.transcripts {
        update_and_emit_source_job(handle, state, job_id, |job| {
            job.message = Some("Syncing YouTube transcript.".to_string());
        })
        .await;
        run_source_job_step_with_cancel(
            cancellation_token.clone(),
            sync_youtube_transcript(handle, sync_source_id),
        )
        .await?;
    }
    if options.comments {
        update_and_emit_source_job(handle, state, job_id, |job| {
            job.message = Some("Syncing YouTube comments.".to_string());
        })
        .await;
        warnings.extend(
            run_source_job_step_with_cancel(
                cancellation_token.clone(),
                sync_youtube_comments(handle, sync_source_id),
            )
            .await?,
        );
    }

    Ok(warnings)
}

async fn run_source_job_step_with_cancel<Fut, T>(
    cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> AppResult<T>
where
    Fut: Future<Output = AppResult<T>>,
{
    let Some(cancellation_token) = cancellation_token else {
        return future.await;
    };

    if cancellation_token.is_cancelled() {
        return Err(AppError::validation("Source job cancelled"));
    }

    tokio::select! {
        result = future => result,
        _ = cancellation_token.cancelled() => Err(AppError::validation("Source job cancelled")),
    }
}

async fn load_video_metadata_or_refresh<F, Fut>(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    refresh: F,
) -> AppResult<YoutubeVideoSourceMetadata>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<()>>,
{
    if let Some(metadata) = load_video_source_metadata_map(pool, &[source_id])
        .await?
        .remove(&source_id)
    {
        return Ok(metadata);
    }

    refresh().await?;

    load_video_source_metadata_map(pool, &[source_id])
        .await?
        .remove(&source_id)
        .ok_or_else(|| {
            AppError::validation(format!(
                "Source {source_id} has missing or invalid typed YouTube video metadata"
            ))
        })
}

async fn load_playlist_metadata_for_refresh(
    pool: &sqlx::SqlitePool,
    source_id: i64,
) -> AppResult<Option<crate::youtube::source_metadata::YoutubePlaylistSourceMetadata>> {
    Ok(load_playlist_source_metadata_map(pool, &[source_id])
        .await?
        .remove(&source_id))
}

async fn sync_youtube_metadata(handle: &AppHandle, source_id: i64) -> AppResult<()> {
    let pool = get_pool(handle).await?;
    let source = load_source(&pool, source_id).await?;
    ensure_youtube_source(&source)?;
    let secrets = handle.state::<SecretStoreState>();
    let cookies = load_youtube_auth_cookies_from_state(&pool, &secrets).await?;

    enum MetadataSyncPayload {
        Video(YoutubeVideoMetadata),
        Playlist(YoutubePlaylistMetadata),
    }

    let payload = match source.source_subtype.as_deref() {
        Some("playlist") => {
            let typed = load_playlist_metadata_for_refresh(&pool, source_id).await?;
            let canonical_url = typed
                .as_ref()
                .map(|metadata| metadata.canonical_url.clone())
                .unwrap_or_else(|| playlist_canonical_url(&source));
            MetadataSyncPayload::Playlist(fetch_playlist_metadata(&canonical_url, cookies).await?)
        }
        _ => {
            let typed = load_video_source_metadata_map(&pool, &[source_id])
                .await?
                .remove(&source_id);
            let canonical_url = typed
                .as_ref()
                .map(|metadata| metadata.canonical_url.clone())
                .unwrap_or_else(|| video_canonical_url(&source));
            let video_form = typed
                .as_ref()
                .and_then(|metadata| metadata.video_form_for_provider())
                .unwrap_or(YoutubeVideoForm::Regular);
            MetadataSyncPayload::Video(
                fetch_video_metadata(&canonical_url, video_form, cookies).await?,
            )
        }
    };

    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let refreshed_source_id = match payload {
        MetadataSyncPayload::Playlist(metadata) => {
            let refreshed_source_id = upsert_youtube_playlist_source(&mut tx, &metadata).await?;
            upsert_playlist_items(&mut tx, refreshed_source_id, &metadata).await?;
            mark_source_synced(&mut tx, refreshed_source_id).await?;
            refreshed_source_id
        }
        MetadataSyncPayload::Video(metadata) => {
            let refreshed_source_id = upsert_youtube_video_source(&mut tx, &metadata).await?;
            mark_source_synced(&mut tx, refreshed_source_id).await?;
            refreshed_source_id
        }
    };
    tx.commit().await.map_err(AppError::database)?;
    crate::archive_read_model::mark_source_stale(&pool, refreshed_source_id).await
}

async fn sync_youtube_transcript(handle: &AppHandle, source_id: i64) -> AppResult<()> {
    let pool = get_pool(handle).await?;
    let source = load_source(&pool, source_id).await?;
    ensure_youtube_source(&source)?;
    if source.source_subtype.as_deref() != Some("video") {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a YouTube video source"
        )));
    }

    let metadata = load_video_metadata_or_refresh(&pool, source_id, || {
        sync_youtube_metadata(handle, source_id)
    })
    .await?;
    let metadata_for_provider = metadata.to_provider_metadata();
    let preferred_language = load_preferred_caption_language(&pool).await?;
    let secrets = handle.state::<SecretStoreState>();
    let cookies = load_youtube_auth_cookies_from_state(&pool, &secrets).await?;
    let transcript = fetch_transcript_for_video(
        &metadata_for_provider,
        Some(preferred_language.as_str()),
        metadata.caption_language_override.as_deref(),
        cookies,
    )
    .await?;
    if transcript.segments.is_empty() {
        return Err(AppError::validation(
            "YouTube transcript has no text segments",
        ));
    }

    let content = transcript
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let external_id = transcript_external_id(
        &transcript.video_id,
        transcript.language.as_deref(),
        &transcript.track_kind,
    );
    let author = metadata
        .author_display
        .as_deref()
        .or(metadata.channel_title.as_deref());
    let published_at = metadata
        .published_at
        .as_deref()
        .and_then(ymd_to_unix_midnight)
        .unwrap_or_else(now_secs);

    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let item_id = upsert_youtube_transcript_item(
        &mut tx,
        source_id,
        &external_id,
        author,
        published_at,
        &content,
        &transcript,
    )
    .await?;
    replace_transcript_segments(&mut tx, item_id, source_id, &transcript).await?;
    mark_source_synced(&mut tx, source_id).await?;
    tx.commit().await.map_err(AppError::database)?;
    crate::archive_read_model::mark_source_stale(&pool, source_id).await
}

async fn sync_youtube_comments(handle: &AppHandle, source_id: i64) -> AppResult<Vec<String>> {
    let pool = get_pool(handle).await?;
    let source = load_source(&pool, source_id).await?;
    ensure_youtube_source(&source)?;
    if source.source_subtype.as_deref() != Some("video") {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a YouTube video source"
        )));
    }

    let metadata = load_video_metadata_or_refresh(&pool, source_id, || {
        sync_youtube_metadata(handle, source_id)
    })
    .await?;
    let metadata_for_provider = metadata.to_provider_metadata();
    let secrets = handle.state::<SecretStoreState>();
    let cookies = load_youtube_auth_cookies_from_state(&pool, &secrets).await?;
    let comments = fetch_comments_for_video(
        &metadata_for_provider,
        DEFAULT_MAX_COMMENTS_PER_VIDEO,
        now_secs(),
        cookies,
    )
    .await?;

    let mut tx = pool.begin().await.map_err(AppError::database)?;
    for comment in &comments.comments {
        upsert_youtube_comment_item(&mut tx, source_id, comment).await?;
    }
    mark_source_synced(&mut tx, source_id).await?;
    tx.commit().await.map_err(AppError::database)?;
    crate::archive_read_model::mark_source_stale(&pool, source_id).await?;
    Ok(comments.warnings)
}

async fn mark_source_synced(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<()> {
    sqlx::query("UPDATE sources SET last_synced_at = strftime('%s','now') WHERE id = ?")
        .bind(source_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

async fn run_retry_playlist_job(handle: AppHandle, job_id: String) {
    let state = handle.state::<SourceJobState>();
    let Some(record) = state
        .update_job(&job_id, |job| {
            job.status = SourceJobStatus::Running;
            job.message = Some("Finding retryable playlist videos.".to_string());
        })
        .await
    else {
        return;
    };
    emit_source_job_event(&handle, &record);

    let result = async {
        let repair_state = handle.state::<SourceIdentityRepairState>();
        require_source_identity_ready(repair_state.inner()).await?;
        let pool = get_pool(&handle).await?;
        let rows = retryable_playlist_video_rows(&pool, record.source_id).await?;
        update_and_emit_source_job(&handle, &state, &job_id, |job| {
            job.progress_current = Some(0);
            job.progress_total = Some(rows.len() as i64);
            job.message = Some("Retrying playlist videos.".to_string());
        })
        .await;

        for (index, _row) in rows.iter().enumerate() {
            if state.is_cancel_requested(&job_id).await {
                return Err(AppError::validation("Source job cancelled"));
            }
            if let Some(video_source_id) = _row.video_source_id {
                sync_youtube_transcript(&handle, video_source_id).await?;
            }
            update_and_emit_source_job(&handle, &state, &job_id, |job| {
                job.progress_current = Some(index as i64 + 1);
            })
            .await;
        }
        Ok::<(), AppError>(())
    }
    .await;

    match result {
        Ok(()) => {
            if let Some(record) = state
                .finish_job(&job_id, |job| {
                    job.status = SourceJobStatus::Succeeded;
                    job.message = Some("Playlist retry job completed.".to_string());
                })
                .await
            {
                emit_source_job_event(&handle, &record);
            }
        }
        Err(error) if state.is_cancel_requested(&job_id).await => {
            finish_cancelled_job(&handle, &state, &job_id).await;
            if !matches!(error.kind, crate::error::AppErrorKind::Validation) {
                let _ = error;
            }
        }
        Err(error) => {
            if let Some(record) = state
                .finish_job(&job_id, |job| {
                    job.status = SourceJobStatus::Failed;
                    job.message = None;
                    job.error = Some(error.to_string());
                })
                .await
            {
                emit_source_job_event(&handle, &record);
            }
        }
    }
}

async fn update_and_emit_source_job<F>(
    handle: &AppHandle,
    state: &SourceJobState,
    job_id: &str,
    update: F,
) where
    F: FnOnce(&mut SourceJobRecord),
{
    if let Some(record) = state.update_job(job_id, update).await {
        emit_source_job_event(handle, &record);
    }
}

async fn finish_cancelled_job(handle: &AppHandle, state: &SourceJobState, job_id: &str) {
    if let Some(record) = state
        .finish_job(job_id, |job| {
            job.status = SourceJobStatus::Cancelled;
            job.message = Some("Source job cancelled.".to_string());
        })
        .await
    {
        emit_source_job_event(handle, &record);
    }
}

fn source_job_type_for_source_options(
    source_subtype: Option<&str>,
    options: &YoutubeSyncOptions,
) -> SourceJobType {
    match source_subtype {
        Some("playlist") => {
            if options.metadata && !options.transcripts && !options.comments {
                SourceJobType::YoutubePlaylistMetadataSync
            } else {
                SourceJobType::YoutubePlaylistFullSync
            }
        }
        _ => match (options.metadata, options.transcripts, options.comments) {
            (true, false, false) => SourceJobType::YoutubeVideoMetadataSync,
            (false, true, false) => SourceJobType::YoutubeVideoTranscriptSync,
            (false, false, true) => SourceJobType::YoutubeVideoCommentsSync,
            _ => SourceJobType::YoutubeVideoFullSync,
        },
    }
}

fn ensure_youtube_source(source: &crate::sources::SourceSyncTarget) -> AppResult<()> {
    if source.source_type != "youtube" {
        return Err(AppError::validation(format!(
            "Source {} is not a YouTube source",
            source.id
        )));
    }
    Ok(())
}

fn video_canonical_url(source: &SourceSyncTarget) -> String {
    format!("https://www.youtube.com/watch?v={}", source.external_id)
}

fn playlist_canonical_url(source: &SourceSyncTarget) -> String {
    format!(
        "https://www.youtube.com/playlist?list={}",
        source.external_id
    )
}

async fn load_preferred_caption_language(pool: &sqlx::SqlitePool) -> AppResult<String> {
    let value: Option<String> = sqlx::query_scalar("SELECT value FROM app_settings WHERE key = ?")
        .bind("youtube.captions.preferred_language")
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;
    Ok(value.unwrap_or_else(|| "original".to_string()))
}

fn emit_source_job_event(handle: &AppHandle, record: &SourceJobRecord) {
    let _ = handle.emit(SOURCE_JOB_EVENT, record);
}

fn is_terminal_status(status: &SourceJobStatus) -> bool {
    matches!(
        status,
        SourceJobStatus::Succeeded | SourceJobStatus::Failed | SourceJobStatus::Cancelled
    )
}

fn source_job_type_diagnostic_key(job_type: &SourceJobType) -> &'static str {
    match job_type {
        SourceJobType::YoutubeVideoMetadataSync => "youtube_video_metadata_sync",
        SourceJobType::YoutubeVideoTranscriptSync => "youtube_video_transcript_sync",
        SourceJobType::YoutubeVideoCommentsSync => "youtube_video_comments_sync",
        SourceJobType::YoutubeVideoFullSync => "youtube_video_full_sync",
        SourceJobType::YoutubePlaylistMetadataSync => "youtube_playlist_metadata_sync",
        SourceJobType::YoutubePlaylistFullSync => "youtube_playlist_full_sync",
        SourceJobType::YoutubePlaylistVideoSync => "youtube_playlist_video_sync",
    }
}

fn source_job_status_diagnostic_key(status: &SourceJobStatus) -> &'static str {
    match status {
        SourceJobStatus::Queued => "queued",
        SourceJobStatus::Running => "running",
        SourceJobStatus::Succeeded => "succeeded",
        SourceJobStatus::Failed => "failed",
        SourceJobStatus::CancelRequested => "cancel_requested",
        SourceJobStatus::Cancelled => "cancelled",
    }
}

fn classify_diagnostic_error_kind(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if lower.trim().is_empty() {
        "none"
    } else if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("dns")
        || lower.contains("socket")
        || lower.contains("transport")
    {
        "network"
    } else if lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("api key")
        || lower.contains("not authenticated")
    {
        "auth"
    } else if lower.contains("invalid")
        || lower.contains("unsupported")
        || lower.contains("required")
        || lower.contains("cannot be empty")
    {
        "validation"
    } else {
        "internal"
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        retryable_playlist_video_rows, run_source_job_step_with_cancel, SourceJobListFilter,
        SourceJobState, SourceJobStatus, SourceJobType, YoutubeSyncOptions,
    };
    use crate::error::{AppError, AppErrorKind, AppResult};
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn jobs_reload_missing_typed_video_metadata_after_refresh_callback() {
        let pool = crate::sources::test_support::memory_pool_with_sources().await;
        crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (701, 'youtube', 'video', 'jobvideo', 'Job video', x'00', 1, 0, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert source");

        let refreshed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let refreshed_for_callback = refreshed.clone();
        let metadata = super::load_video_metadata_or_refresh(&pool, 701, || {
            let pool = pool.clone();
            async move {
                refreshed_for_callback.store(true, std::sync::atomic::Ordering::SeqCst);
                insert_typed_video_metadata_for_job_test(&pool, 701, "jobvideo").await;
                Ok(())
            }
        })
        .await
        .expect("load refreshed metadata");

        assert!(refreshed.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(metadata.video_id, "jobvideo");
        assert_eq!(
            metadata.canonical_url,
            "https://www.youtube.com/watch?v=jobvideo"
        );
    }

    #[tokio::test]
    async fn jobs_missing_typed_video_metadata_errors_after_failed_refresh() {
        let pool = crate::sources::test_support::memory_pool_with_sources().await;
        crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (702, 'youtube', 'video', 'jobmissing', 'Job missing', x'00', 1, 0, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert source");

        let error = super::load_video_metadata_or_refresh(&pool, 702, || async { Ok(()) })
            .await
            .expect_err("missing typed metadata rejected");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert!(error.to_string().contains("typed YouTube video metadata"));
        assert!(!error.to_string().contains("metadata_zstd"));
    }

    #[test]
    fn source_jobs_no_longer_decode_source_metadata_blobs() {
        let source = std::fs::read_to_string("src/youtube/jobs.rs").expect("read jobs.rs");
        let decode_symbol = ["decode", "youtube", "metadata"].join("_");
        let decompress_symbol = ["decompress", "bytes"].join("_");
        assert!(!source.contains(&decode_symbol));
        assert!(!source.contains(&decompress_symbol));
    }

    #[test]
    fn source_job_workflow_file_has_no_tauri_command_adapters() {
        let source = std::fs::read_to_string("src/youtube/jobs.rs").expect("read jobs.rs");
        let command_attribute = ["#[tauri", "::command]"].join("");

        assert!(
            !source.contains(&command_attribute),
            "YouTube job command adapters should live outside src/youtube/jobs.rs"
        );
    }

    #[tokio::test]
    async fn job_state_rejects_duplicate_active_scope_but_allows_different_job_types() {
        let state = SourceJobState::new();
        let options = YoutubeSyncOptions {
            metadata: true,
            transcripts: false,
            comments: false,
        };

        let first = state
            .create_job(
                7,
                SourceJobType::YoutubeVideoMetadataSync,
                None,
                options.clone(),
            )
            .await
            .expect("create first job");
        let duplicate = state
            .create_job(
                7,
                SourceJobType::YoutubeVideoMetadataSync,
                None,
                options.clone(),
            )
            .await
            .expect_err("duplicate job scope should fail");
        let transcript = state
            .create_job(7, SourceJobType::YoutubeVideoTranscriptSync, None, options)
            .await
            .expect("different job type can coexist");

        assert_eq!(first.job_id, "source-job-1");
        assert_eq!(duplicate.kind, AppErrorKind::Conflict);
        assert_eq!(transcript.job_id, "source-job-2");
    }

    #[tokio::test]
    async fn job_state_list_filters_before_limit_and_sorts_newest_first() {
        let state = SourceJobState::new();
        let options = YoutubeSyncOptions {
            metadata: true,
            transcripts: false,
            comments: false,
        };

        let first = state
            .create_job(
                1,
                SourceJobType::YoutubeVideoMetadataSync,
                None,
                options.clone(),
            )
            .await
            .expect("create first job");
        state
            .finish_job(&first.job_id, |job| {
                job.status = SourceJobStatus::Succeeded;
            })
            .await
            .expect("finish first job");
        let second = state
            .create_job(
                2,
                SourceJobType::YoutubeVideoMetadataSync,
                None,
                options.clone(),
            )
            .await
            .expect("create second job");
        state
            .finish_job(&second.job_id, |job| {
                job.status = SourceJobStatus::Failed;
                job.error = Some("failed".to_string());
            })
            .await
            .expect("finish second job");
        let third = state
            .create_job(1, SourceJobType::YoutubeVideoTranscriptSync, None, options)
            .await
            .expect("create third job");

        let jobs = state
            .list_jobs(SourceJobListFilter {
                source_id: Some(1),
                status: None,
                limit: Some(1),
            })
            .await;

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_id, third.job_id);
    }

    #[tokio::test]
    async fn job_state_finishes_cancel_requested_jobs_as_cancelled() {
        let state = SourceJobState::new();
        let options = YoutubeSyncOptions {
            metadata: false,
            transcripts: false,
            comments: true,
        };
        let job = state
            .create_job(7, SourceJobType::YoutubeVideoCommentsSync, None, options)
            .await
            .expect("create comments job");

        state
            .request_cancel(&job.job_id)
            .await
            .expect("request cancel");
        let finished = state
            .finish_job(&job.job_id, |job| {
                job.status = SourceJobStatus::Succeeded;
                job.message = Some("Source job completed.".to_string());
            })
            .await
            .expect("finish job");

        assert_eq!(finished.status, SourceJobStatus::Cancelled);
        assert_eq!(finished.message.as_deref(), Some("Source job cancelled."));
    }

    #[tokio::test]
    async fn job_state_cancels_child_tokens() {
        let state = SourceJobState::new();
        let options = YoutubeSyncOptions {
            metadata: false,
            transcripts: false,
            comments: true,
        };
        let job = state
            .create_job(7, SourceJobType::YoutubeVideoCommentsSync, None, options)
            .await
            .expect("create comments job");
        let token = state
            .cancellation_token(&job.job_id)
            .await
            .expect("cancellation token");

        assert!(!token.is_cancelled());

        state
            .request_cancel(&job.job_id)
            .await
            .expect("request cancel");
        tokio::time::timeout(std::time::Duration::from_secs(1), token.cancelled())
            .await
            .expect("token cancelled");

        state
            .finish_job(&job.job_id, |job| {
                job.status = SourceJobStatus::Succeeded;
            })
            .await
            .expect("finish job");
        assert!(state.cancellation_token(&job.job_id).await.is_none());
    }

    #[tokio::test]
    async fn source_job_step_cancel_wrapper_allows_completed_future() {
        let result = run_source_job_step_with_cancel(None, async { Ok::<_, AppError>("done") })
            .await
            .expect("step result");

        assert_eq!(result, "done");
    }

    #[tokio::test]
    async fn source_job_step_cancel_wrapper_interrupts_pending_future() {
        let token = CancellationToken::new();
        token.cancel();

        let result: AppResult<()> =
            run_source_job_step_with_cancel(Some(token), std::future::pending()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn active_jobs_for_sources_filters_non_terminal_direct_and_related_sources() {
        let state = SourceJobState::new();
        let options = YoutubeSyncOptions {
            metadata: true,
            transcripts: false,
            comments: false,
        };
        let direct = state
            .create_job(
                7,
                SourceJobType::YoutubeVideoFullSync,
                None,
                options.clone(),
            )
            .await
            .expect("direct job");
        let related = state
            .create_job(
                99,
                SourceJobType::YoutubePlaylistVideoSync,
                Some(8),
                options.clone(),
            )
            .await
            .expect("related job");
        let terminal = state
            .create_job(
                8,
                SourceJobType::YoutubeVideoMetadataSync,
                None,
                options.clone(),
            )
            .await
            .expect("terminal job");
        let _unowned = state
            .create_job(42, SourceJobType::YoutubeVideoTranscriptSync, None, options)
            .await
            .expect("unowned job");
        state
            .finish_job(&terminal.job_id, |job| {
                job.status = SourceJobStatus::Succeeded;
            })
            .await
            .expect("finish terminal job");
        state
            .request_cancel(&direct.job_id)
            .await
            .expect("cancel requested remains non-terminal");

        let active = state.active_jobs_for_sources(&[7, 8]).await;
        let active_ids = active
            .iter()
            .map(|job| job.job_id.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            active_ids,
            BTreeSet::from([direct.job_id.as_str(), related.job_id.as_str()])
        );
    }

    #[tokio::test]
    async fn catalog_jobs_for_sources_includes_latest_failed_jobs() {
        let state = SourceJobState::new();
        let options = YoutubeSyncOptions {
            metadata: true,
            transcripts: false,
            comments: false,
        };

        let succeeded = state
            .create_job(
                7,
                SourceJobType::YoutubeVideoMetadataSync,
                None,
                options.clone(),
            )
            .await
            .expect("create succeeded job");
        state
            .finish_job(&succeeded.job_id, |job| {
                job.status = SourceJobStatus::Succeeded;
                job.started_at = 10;
            })
            .await
            .expect("finish succeeded job");

        let failed = state
            .create_job(
                7,
                SourceJobType::YoutubeVideoTranscriptSync,
                None,
                options.clone(),
            )
            .await
            .expect("create failed job");
        state
            .finish_job(&failed.job_id, |job| {
                job.status = SourceJobStatus::Failed;
                job.started_at = 20;
                job.error = Some("Transcript quota exceeded".to_string());
            })
            .await
            .expect("finish failed job");

        let related = state
            .create_job(
                99,
                SourceJobType::YoutubePlaylistVideoSync,
                Some(8),
                options,
            )
            .await
            .expect("create related job");
        state
            .update_job(&related.job_id, |job| {
                job.status = SourceJobStatus::Running;
                job.started_at = 30;
                job.message = Some("Syncing playlist video.".to_string());
            })
            .await
            .expect("update related job");

        let jobs = state.catalog_jobs_for_sources(&[7, 8]).await;

        assert_eq!(
            jobs.iter()
                .map(|job| job.job_id.as_str())
                .collect::<Vec<_>>(),
            vec![related.job_id.as_str(), failed.job_id.as_str()]
        );
        assert_eq!(jobs[0].related_source_id, Some(8));
        assert_eq!(jobs[1].status, SourceJobStatus::Failed);
    }

    #[tokio::test]
    async fn diagnostic_counts_group_source_jobs_without_ids_or_raw_errors() {
        let state = SourceJobState::new();
        let job = state
            .create_job(
                10,
                SourceJobType::YoutubeVideoFullSync,
                None,
                YoutubeSyncOptions {
                    metadata: true,
                    transcripts: true,
                    comments: true,
                },
            )
            .await
            .expect("create job");
        state
            .finish_job(&job.job_id, |record| {
                record.status = SourceJobStatus::Failed;
                record.error =
                    Some("timeout with https://youtube.example/watch?v=private".to_string());
                record.warnings = vec!["raw warning with private title".to_string()];
            })
            .await
            .expect("finish job");

        let counts = state.diagnostic_counts().await;

        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].job_type, "youtube_video_full_sync");
        assert_eq!(counts[0].status, "failed");
        assert_eq!(counts[0].warning_state, "present");
        assert_eq!(counts[0].error_kind, "network");
        assert_eq!(counts[0].count, 1);
        let json = serde_json::to_string(&counts).expect("serialize counts");
        assert!(!json.contains("source-job-"));
        assert!(!json.contains("source_id"));
        assert!(!json.contains("related_source_id"));
        assert!(!json.contains("youtube.example"));
        assert!(!json.contains("private title"));
    }

    #[test]
    fn source_job_type_uses_comments_specific_type_for_comments_only_video_sync() {
        let comments_only = YoutubeSyncOptions {
            metadata: false,
            transcripts: false,
            comments: true,
        };
        let full = YoutubeSyncOptions {
            metadata: true,
            transcripts: true,
            comments: true,
        };

        assert_eq!(
            super::source_job_type_for_source_options(Some("video"), &comments_only),
            SourceJobType::YoutubeVideoCommentsSync
        );
        assert_eq!(
            super::source_job_type_for_source_options(Some("video"), &full),
            SourceJobType::YoutubeVideoFullSync
        );
    }

    #[tokio::test]
    async fn retryable_playlist_video_rows_excludes_auth_deleted_and_removed_entries() {
        let pool = crate::sources::test_support::memory_pool_with_sources().await;
        sqlx::query(
            r#"
            CREATE TABLE youtube_playlist_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                playlist_source_id INTEGER NOT NULL,
                video_source_id INTEGER,
                video_id TEXT NOT NULL,
                position INTEGER,
                title_snapshot TEXT,
                url TEXT,
                thumbnail_url TEXT,
                availability_status TEXT NOT NULL,
                is_removed_from_playlist INTEGER NOT NULL DEFAULT 0,
                last_seen_at INTEGER,
                metadata_zstd BLOB,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(playlist_source_id, video_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create playlist table");

        for (index, (video_id, status, removed)) in [
            ("retry-live", "live_ended_transcript_pending", 0),
            ("retry-none", "no_captions", 0),
            ("retry-unknown", "unavailable_unknown", 0),
            ("auth", "private_or_auth_required", 0),
            ("members", "members_only", 0),
            ("age", "age_restricted", 0),
            ("geo", "geo_blocked", 0),
            ("deleted", "deleted", 0),
            ("removed-status", "removed_from_playlist", 0),
            ("removed-flag", "no_captions", 1),
        ]
        .into_iter()
        .enumerate()
        {
            sqlx::query(
                r#"
                INSERT INTO youtube_playlist_items (
                    playlist_source_id,
                    video_source_id,
                    video_id,
                    position,
                    availability_status,
                    is_removed_from_playlist,
                    created_at,
                    updated_at
                )
                VALUES (42, ?, ?, ?, ?, ?, 1, 1)
                "#,
            )
            .bind(Some(index as i64 + 100))
            .bind(video_id)
            .bind(index as i64 + 1)
            .bind(status)
            .bind(removed)
            .execute(&pool)
            .await
            .expect("insert playlist row");
        }

        let rows = retryable_playlist_video_rows(&pool, 42)
            .await
            .expect("load retryable rows");

        assert_eq!(
            rows.iter()
                .map(|row| row.video_id.as_str())
                .collect::<Vec<_>>(),
            vec!["retry-live", "retry-none", "retry-unknown"]
        );
        assert!(rows.iter().all(|row| row.video_source_id.is_some()));
    }

    async fn insert_typed_video_metadata_for_job_test(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        video_id: &str,
    ) {
        let metadata = crate::youtube::dto::YoutubeVideoMetadata {
            video_id: video_id.to_string(),
            canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
            title: Some("Job typed video".to_string()),
            channel_title: Some("Job channel".to_string()),
            channel_id: None,
            channel_handle: None,
            channel_url: None,
            author_display: Some("Job channel".to_string()),
            published_at: Some("2026-05-17".to_string()),
            duration_seconds: Some(30),
            description: None,
            thumbnail_url: None,
            tags: Vec::new(),
            chapters: Vec::new(),
            view_count: None,
            like_count: None,
            comment_count: None,
            category: None,
            video_form: crate::youtube::dto::YoutubeVideoForm::Regular,
            availability_status: crate::youtube::dto::YoutubeAvailabilityStatus::Available,
            raw_metadata_json: serde_json::json!({
                "id": video_id,
                "caption_language_override": "en"
            }),
        };
        crate::youtube::source_metadata::insert_video_source_metadata_for_pool_test(
            pool, source_id, &metadata,
        )
        .await;
    }
}
