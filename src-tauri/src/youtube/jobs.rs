use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::load_source;

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
        _options: YoutubeSyncOptions,
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
        inner.jobs.insert(job_id, record.clone());
        Ok(record)
    }

    pub(crate) async fn list_jobs(
        &self,
        filter: SourceJobListFilter,
    ) -> Vec<SourceJobRecord> {
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
    if let Some(record) = state
        .update_job(&job_id, |job| {
            job.status = SourceJobStatus::Running;
            job.message = Some("Source job running.".to_string());
        })
        .await
    {
        emit_source_job_event(&handle, &record);
    }

    if state.is_cancel_requested(&job_id).await {
        finish_cancelled_job(&handle, &state, &job_id).await;
        return;
    }

    if let Some(record) = state
        .finish_job(&job_id, |job| {
            job.status = SourceJobStatus::Succeeded;
            job.message = Some("Source job completed.".to_string());
        })
        .await
    {
        emit_source_job_event(&handle, &record);
    }
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
            .create_job(7, SourceJobType::YoutubeVideoMetadataSync, None, options.clone())
            .await
            .expect("create first job");
        let duplicate = state
            .create_job(7, SourceJobType::YoutubeVideoMetadataSync, None, options.clone())
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
            .create_job(1, SourceJobType::YoutubeVideoMetadataSync, None, options.clone())
            .await
            .expect("create first job");
        state
            .finish_job(&first.job_id, |job| {
                job.status = SourceJobStatus::Succeeded;
            })
            .await
            .expect("finish first job");
        let second = state
            .create_job(2, SourceJobType::YoutubeVideoMetadataSync, None, options.clone())
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
