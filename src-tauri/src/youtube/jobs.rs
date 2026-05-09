use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::compression::decompress_bytes;
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{
    load_source, upsert_youtube_comment_item, upsert_youtube_playlist_source,
    upsert_youtube_transcript_item, upsert_youtube_video_source, SourceSyncTarget,
};

use super::captions::{
    fetch_transcript_for_video, replace_transcript_segments, transcript_external_id,
};
use super::comments::{fetch_comments_for_video, DEFAULT_MAX_COMMENTS_PER_VIDEO};
use super::dto::{YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata};
use super::metadata::{fetch_playlist_metadata, fetch_video_metadata};
use super::playlist::upsert_playlist_items;

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
    active_by_key: HashMap<SourceJobKey, String>,
    key_by_job_id: HashMap<String, SourceJobKey>,
    options_by_job_id: HashMap<String, YoutubeSyncOptions>,
    cancel_requested: HashSet<String>,
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
        if let Some(job_id) = inner.active_by_key.get(&key) {
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

        inner.active_by_key.insert(key.clone(), job_id.clone());
        inner.key_by_job_id.insert(job_id.clone(), key);
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
                    .map_or(true, |source_id| job.source_id == source_id)
            })
            .filter(|job| {
                filter
                    .status
                    .as_ref()
                    .map_or(true, |status| job.status == *status)
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

    pub(crate) async fn request_cancel(&self, job_id: &str) -> Option<SourceJobRecord> {
        let mut inner = self.inner.lock().await;
        if is_terminal_status(&inner.jobs.get(job_id)?.status) {
            return None;
        }

        inner.cancel_requested.insert(job_id.to_string());
        let job = inner.jobs.get_mut(job_id)?;
        job.status = SourceJobStatus::CancelRequested;
        job.message = Some("Cancel requested.".to_string());
        Some(job.clone())
    }

    pub(crate) async fn is_cancel_requested(&self, job_id: &str) -> bool {
        self.inner.lock().await.cancel_requested.contains(job_id)
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
        {
            let job = inner.jobs.get_mut(job_id)?;
            update(job);
            job.finished_at = Some(now_secs());
        }
        if let Some(key) = inner.key_by_job_id.remove(job_id) {
            inner.active_by_key.remove(&key);
        }
        inner.options_by_job_id.remove(job_id);
        inner.cancel_requested.remove(job_id);
        inner.jobs.get(job_id).cloned()
    }
}

#[tauri::command]
pub(crate) async fn sync_youtube_source(
    handle: AppHandle,
    state: tauri::State<'_, SourceJobState>,
    source_id: i64,
    options: YoutubeSyncOptions,
) -> AppResult<SourceJobRecord> {
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

#[tauri::command]
pub(crate) async fn sync_youtube_playlist_video(
    handle: AppHandle,
    state: tauri::State<'_, SourceJobState>,
    playlist_source_id: i64,
    video_source_id: i64,
    options: YoutubeSyncOptions,
) -> AppResult<SourceJobRecord> {
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

#[tauri::command]
pub(crate) async fn retry_failed_youtube_playlist_videos(
    handle: AppHandle,
    state: tauri::State<'_, SourceJobState>,
    source_id: i64,
    _options: YoutubeSyncOptions,
) -> AppResult<SourceJobRecord> {
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

#[tauri::command]
pub(crate) async fn cancel_source_job(
    handle: AppHandle,
    state: tauri::State<'_, SourceJobState>,
    job_id: String,
) -> AppResult<()> {
    if let Some(record) = state.request_cancel(&job_id).await {
        emit_source_job_event(&handle, &record);
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn list_source_jobs(
    state: tauri::State<'_, SourceJobState>,
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
    let mut warnings = Vec::new();
    let sync_source_id = related_source_id.unwrap_or(source_id);
    if options.metadata {
        update_and_emit_source_job(handle, state, job_id, |job| {
            job.message = Some("Refreshing YouTube metadata.".to_string());
        })
        .await;
        sync_youtube_metadata(handle, sync_source_id).await?;
    }

    if state.is_cancel_requested(job_id).await {
        return Err(AppError::validation("Source job cancelled"));
    }

    if options.transcripts {
        update_and_emit_source_job(handle, state, job_id, |job| {
            job.message = Some("Syncing YouTube transcript.".to_string());
        })
        .await;
        sync_youtube_transcript(handle, sync_source_id).await?;
    }
    if options.comments {
        update_and_emit_source_job(handle, state, job_id, |job| {
            job.message = Some("Syncing YouTube comments.".to_string());
        })
        .await;
        warnings.extend(sync_youtube_comments(handle, sync_source_id).await?);
    }

    Ok(warnings)
}

async fn sync_youtube_metadata(handle: &AppHandle, source_id: i64) -> AppResult<()> {
    let pool = get_pool(handle).await?;
    let source = load_source(&pool, source_id).await?;
    ensure_youtube_source(&source)?;

    enum MetadataSyncPayload {
        Video(YoutubeVideoMetadata),
        Playlist(YoutubePlaylistMetadata),
    }

    let payload = match source.source_subtype.as_deref() {
        Some("playlist") => MetadataSyncPayload::Playlist(
            fetch_playlist_metadata(&playlist_canonical_url(&source)).await?,
        ),
        _ => {
            let existing = decode_video_metadata(&source);
            let canonical_url = existing
                .as_ref()
                .map(|metadata| metadata.canonical_url.clone())
                .unwrap_or_else(|| video_canonical_url(&source));
            let video_form = existing
                .as_ref()
                .map(|metadata| metadata.video_form.clone())
                .unwrap_or(YoutubeVideoForm::Regular);
            MetadataSyncPayload::Video(fetch_video_metadata(&canonical_url, video_form).await?)
        }
    };

    let mut tx = pool.begin().await.map_err(AppError::database)?;
    match payload {
        MetadataSyncPayload::Playlist(metadata) => {
            let refreshed_source_id = upsert_youtube_playlist_source(&mut tx, &metadata).await?;
            upsert_playlist_items(&mut tx, refreshed_source_id, &metadata).await?;
            mark_source_synced(&mut tx, refreshed_source_id).await?;
        }
        MetadataSyncPayload::Video(metadata) => {
            let refreshed_source_id = upsert_youtube_video_source(&mut tx, &metadata).await?;
            mark_source_synced(&mut tx, refreshed_source_id).await?;
        }
    }
    tx.commit().await.map_err(AppError::database)
}

async fn sync_youtube_transcript(handle: &AppHandle, source_id: i64) -> AppResult<()> {
    let pool = get_pool(handle).await?;
    let mut source = load_source(&pool, source_id).await?;
    ensure_youtube_source(&source)?;
    if source.source_subtype.as_deref() != Some("video") {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a YouTube video source"
        )));
    }

    if decode_video_metadata(&source).is_none() {
        sync_youtube_metadata(handle, source_id).await?;
        source = load_source(&pool, source_id).await?;
    }

    let metadata = decode_video_metadata(&source).ok_or_else(|| {
        AppError::validation(format!("Source {source_id} has no YouTube video metadata"))
    })?;
    let preferred_language = load_preferred_caption_language(&pool).await?;
    let transcript = fetch_transcript_for_video(
        &metadata,
        Some(preferred_language.as_str()),
        caption_language_override(&metadata).as_deref(),
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
    tx.commit().await.map_err(AppError::database)
}

async fn sync_youtube_comments(handle: &AppHandle, source_id: i64) -> AppResult<Vec<String>> {
    let pool = get_pool(handle).await?;
    let mut source = load_source(&pool, source_id).await?;
    ensure_youtube_source(&source)?;
    if source.source_subtype.as_deref() != Some("video") {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a YouTube video source"
        )));
    }

    if decode_video_metadata(&source).is_none() {
        sync_youtube_metadata(handle, source_id).await?;
        source = load_source(&pool, source_id).await?;
    }

    let metadata = decode_video_metadata(&source).ok_or_else(|| {
        AppError::validation(format!("Source {source_id} has no YouTube video metadata"))
    })?;
    let comments =
        fetch_comments_for_video(&metadata, DEFAULT_MAX_COMMENTS_PER_VIDEO, now_secs()).await?;

    let mut tx = pool.begin().await.map_err(AppError::database)?;
    for comment in &comments.comments {
        upsert_youtube_comment_item(&mut tx, source_id, comment).await?;
    }
    mark_source_synced(&mut tx, source_id).await?;
    tx.commit().await.map_err(AppError::database)?;
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
    decode_playlist_metadata(source)
        .map(|metadata| metadata.canonical_url)
        .unwrap_or_else(|| {
            format!(
                "https://www.youtube.com/playlist?list={}",
                source.external_id
            )
        })
}

fn decode_video_metadata(source: &SourceSyncTarget) -> Option<YoutubeVideoMetadata> {
    decode_youtube_metadata(source.metadata_zstd.as_deref())
}

fn decode_playlist_metadata(source: &SourceSyncTarget) -> Option<YoutubePlaylistMetadata> {
    decode_youtube_metadata(source.metadata_zstd.as_deref())
}

fn decode_youtube_metadata<T>(metadata_zstd: Option<&[u8]>) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    let metadata = metadata_zstd?;
    let json = decompress_bytes(metadata).ok()?;
    serde_json::from_slice(&json).ok()
}

async fn load_preferred_caption_language(pool: &sqlx::SqlitePool) -> AppResult<String> {
    let value: Option<String> = sqlx::query_scalar("SELECT value FROM app_settings WHERE key = ?")
        .bind("youtube.captions.preferred_language")
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;
    Ok(value.unwrap_or_else(|| "original".to_string()))
}

fn caption_language_override(metadata: &YoutubeVideoMetadata) -> Option<String> {
    metadata
        .raw_metadata_json
        .get("caption_language_override")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn ymd_to_unix_midnight(value: &str) -> Option<i64> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
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

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::{
        retryable_playlist_video_rows, SourceJobListFilter, SourceJobState, SourceJobStatus,
        SourceJobType, YoutubeSyncOptions,
    };
    use crate::error::AppErrorKind;

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
}
