use std::collections::HashSet;

use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::{
    AnalysisRunDetail, AnalysisRunMessage, AnalysisRunMessageCursor, AnalysisRunMessagesPage,
    CorpusMessage, StoredRunSnapshotRow,
};
use super::store::fetch_source_group;
#[cfg(test)]
use super::{ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP};
use crate::compression::{decompress_bytes, decompress_text};
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisRunPreflightLimits {
    pub max_messages_per_run: usize,
    pub max_chunks_per_run: usize,
    pub max_estimated_input_chars_per_run: usize,
    /// Reserved for future retry-aware budgeting. Currently equals
    /// `max_chunks_per_run` because each chunk creates exactly one
    /// background request.
    pub max_background_requests_per_run: usize,
}

impl Default for AnalysisRunPreflightLimits {
    fn default() -> Self {
        Self {
            max_messages_per_run: 10_000,
            max_chunks_per_run: 80,
            max_estimated_input_chars_per_run: 1_500_000,
            max_background_requests_per_run: 80,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisRunPreflight {
    pub source_ids: Vec<i64>,
    pub message_count: usize,
    pub estimated_input_chars: usize,
    pub estimated_chunks: usize,
    pub limits: AnalysisRunPreflightLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum YoutubeCorpusMode {
    TranscriptOnly,
    TranscriptDescription,
    TranscriptDescriptionComments,
}

impl YoutubeCorpusMode {
    pub(crate) fn from_wire(value: Option<&str>) -> Result<Self, String> {
        match value.unwrap_or("transcript_description") {
            "transcript_only" => Ok(Self::TranscriptOnly),
            "transcript_description" => Ok(Self::TranscriptDescription),
            "transcript_description_comments" => Ok(Self::TranscriptDescriptionComments),
            other => Err(format!("Unsupported youtube_corpus_mode '{other}'")),
        }
    }

    pub(crate) fn as_wire(self) -> &'static str {
        match self {
            Self::TranscriptOnly => "transcript_only",
            Self::TranscriptDescription => "transcript_description",
            Self::TranscriptDescriptionComments => "transcript_description_comments",
        }
    }

    pub(crate) fn includes_description(self) -> bool {
        matches!(
            self,
            Self::TranscriptDescription | Self::TranscriptDescriptionComments
        )
    }

    pub(crate) fn includes_comments(self) -> bool {
        matches!(self, Self::TranscriptDescriptionComments)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CorpusLoadRequest {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
}

pub(crate) struct ResolvedAnalysisSources {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    #[allow(dead_code)]
    pub(crate) skipped_unlinked_playlist_items: usize,
}

pub(crate) fn estimate_message_input_chars(
    content: &str,
    r#ref: &str,
    author: Option<&str>,
) -> usize {
    content.len() + r#ref.len() + author.unwrap_or("").len() + 64
}

#[allow(dead_code)]
pub(crate) fn live_corpus_ref(source_id: i64, item_id: i64) -> String {
    crate::analysis_documents::live_item_ref(source_id, item_id)
}

pub(crate) fn estimate_preflight_chunk_count(message_sizes: &[usize], max_chars: usize) -> usize {
    let mut chunks = 0usize;
    let mut current_chars = 0usize;

    for size in message_sizes {
        if current_chars > 0 && current_chars + size > max_chars {
            chunks += 1;
            current_chars = 0;
        }
        current_chars += size;
    }

    if current_chars > 0 {
        chunks += 1;
    }

    chunks
}

#[derive(sqlx::FromRow)]
struct AnalysisSourceScopeRow {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
}

async fn load_source_scope_row(
    pool: &Pool<Sqlite>,
    source_id: i64,
) -> AppResult<AnalysisSourceScopeRow> {
    sqlx::query_as(
        r#"
        SELECT id, source_type, source_subtype
        FROM sources
        WHERE id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found(format!("Source {source_id} not found")))
}

async fn linked_playlist_video_source_ids(
    pool: &Pool<Sqlite>,
    playlist_source_id: i64,
) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        r#"
        SELECT video_source_id
        FROM youtube_playlist_items
        WHERE playlist_source_id = ?
          AND video_source_id IS NOT NULL
          AND is_removed_from_playlist = 0
        ORDER BY COALESCE(position, 9223372036854775807), video_id
        "#,
    )
    .bind(playlist_source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn count_skipped_unlinked_playlist_items(
    pool: &Pool<Sqlite>,
    playlist_source_id: i64,
) -> AppResult<usize> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM youtube_playlist_items
        WHERE playlist_source_id = ?
          AND video_source_id IS NULL
          AND is_removed_from_playlist = 0
        "#,
    )
    .bind(playlist_source_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(count.max(0) as usize)
}

pub(crate) async fn resolve_analysis_sources(
    pool: &Pool<Sqlite>,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
) -> AppResult<ResolvedAnalysisSources> {
    if source_id.is_some() == source_group_id.is_some() {
        return Err(AppError::validation(
            "Select either a source or a source group",
        ));
    }

    let source_type;
    let mut source_ids = Vec::new();
    let mut seen_source_ids = HashSet::new();
    let mut skipped_unlinked_playlist_items = 0usize;

    if let Some(source_id) = source_id {
        let source = load_source_scope_row(pool, source_id).await?;
        source_type = source.source_type.clone();
        if source.source_type == "youtube" && source.source_subtype.as_deref() == Some("playlist") {
            skipped_unlinked_playlist_items +=
                count_skipped_unlinked_playlist_items(pool, source.id).await?;
            for video_source_id in linked_playlist_video_source_ids(pool, source.id).await? {
                if seen_source_ids.insert(video_source_id) {
                    source_ids.push(video_source_id);
                }
            }
        } else if seen_source_ids.insert(source.id) {
            source_ids.push(source.id);
        }
    } else {
        let group_id = source_group_id.expect("validated source_group_id");
        let group = fetch_source_group(pool, group_id)
            .await
            .map_err(AppError::database)?
            .ok_or_else(|| {
                AppError::not_found(format!("Analysis source group {group_id} not found"))
            })?;
        source_type = group.source_type.clone();

        for member in group.members {
            let source = load_source_scope_row(pool, member.source_id).await?;
            if source.source_type == "youtube"
                && source.source_subtype.as_deref() == Some("playlist")
            {
                skipped_unlinked_playlist_items +=
                    count_skipped_unlinked_playlist_items(pool, source.id).await?;
                for video_source_id in linked_playlist_video_source_ids(pool, source.id).await? {
                    if seen_source_ids.insert(video_source_id) {
                        source_ids.push(video_source_id);
                    }
                }
            } else if seen_source_ids.insert(source.id) {
                source_ids.push(source.id);
            }
        }
    }

    if source_type == "youtube" && source_ids.is_empty() {
        return Err(AppError::validation(
            "No linked YouTube videos are available for analysis in this scope",
        ));
    }

    Ok(ResolvedAnalysisSources {
        source_type,
        source_ids,
        skipped_unlinked_playlist_items,
    })
}

#[cfg(test)]
pub(crate) async fn resolve_run_source_ids(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<i64>, String> {
    let snapshot_source_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT DISTINCT source_id
        FROM analysis_run_messages
        WHERE run_id = ?
        ORDER BY source_id ASC
        "#,
    )
    .bind(run.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if !snapshot_source_ids.is_empty() {
        return Ok(snapshot_source_ids);
    }

    if run.scope_type == ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE {
        let source_id = run
            .source_id
            .ok_or_else(|| format!("Analysis run {} is missing source_id", run.id))?;
        return Ok(vec![source_id]);
    }

    if run.scope_type == ANALYSIS_SCOPE_TYPE_SOURCE_GROUP {
        let group_id = run
            .source_group_id
            .ok_or_else(|| format!("Analysis run {} is missing source_group_id", run.id))?;
        let group = fetch_source_group(pool, group_id)
            .await?
            .ok_or_else(|| format!("Analysis source group {group_id} not found"))?;
        return Ok(group
            .members
            .into_iter()
            .map(|member| member.source_id)
            .collect());
    }

    Err(format!("Unsupported analysis scope '{}'", run.scope_type))
}

pub(crate) async fn load_corpus_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, String> {
    if request.source_ids.is_empty() {
        return Ok(Vec::new());
    }

    load_analysis_document_messages(pool, request).await
}

#[derive(sqlx::FromRow)]
struct AnalysisDocumentRow {
    item_id: Option<i64>,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    ref_: String,
    content_zstd: Vec<u8>,
    document_kind: String,
    source_type: String,
    source_subtype: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
}

async fn load_analysis_document_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, String> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            item_id,
            source_id,
            external_id,
            author,
            published_at,
            ref AS ref_,
            content_zstd,
            document_kind,
            source_type,
            source_subtype,
            metadata_zstd
        FROM analysis_documents
        WHERE published_at >=
        "#,
    );
    query.push_bind(request.period_from);
    query.push(" AND published_at <= ");
    query.push_bind(request.period_to);
    query.push(" AND source_id IN (");
    {
        let mut separated = query.separated(", ");
        for source_id in &request.source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");
    match request.source_type.as_str() {
        "telegram" => {
            query.push(" AND source_type = 'telegram' AND document_kind = 'telegram_message'");
        }
        "youtube" => {
            query.push(" AND source_type = 'youtube' AND document_kind IN (");
            query.push("'youtube_transcript'");
            if request.youtube_corpus_mode.includes_description() {
                query.push(", 'youtube_description'");
            }
            if request.youtube_corpus_mode.includes_comments() {
                query.push(", 'youtube_comment'");
            }
            query.push(")");
        }
        other => return Err(format!("Unsupported analysis corpus source_type '{other}'")),
    }
    query.push(" ORDER BY published_at ASC, source_id ASC, document_order ASC, id ASC");

    let rows: Vec<AnalysisDocumentRow> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?;

    rows.into_iter()
        .map(|row| {
            Ok(CorpusMessage {
                item_id: row.item_id.unwrap_or(0),
                source_id: row.source_id,
                external_id: row.external_id,
                published_at: row.published_at,
                author: row.author,
                content: decompress_text(&row.content_zstd)?,
                r#ref: row.ref_,
                item_kind: Some(row.document_kind),
                source_type: Some(row.source_type),
                source_subtype: row.source_subtype,
                metadata_zstd: row.metadata_zstd,
            })
        })
        .collect()
}

pub(crate) async fn preflight_analysis_run(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
    chunk_target_chars: usize,
    limits: AnalysisRunPreflightLimits,
) -> Result<AnalysisRunPreflight, String> {
    if request.source_ids.is_empty() {
        return Ok(AnalysisRunPreflight {
            source_ids: Vec::new(),
            message_count: 0,
            estimated_input_chars: 0,
            estimated_chunks: 0,
            limits,
        });
    }

    let corpus = load_corpus_messages(pool, request).await?;

    let mut message_sizes = Vec::with_capacity(corpus.len());
    let mut estimated_input_chars = 0usize;
    for message in &corpus {
        let size = estimate_message_input_chars(
            &message.content,
            &message.r#ref,
            message.author.as_deref(),
        );
        estimated_input_chars += size;
        message_sizes.push(size);
    }

    let estimated_chunks = estimate_preflight_chunk_count(&message_sizes, chunk_target_chars);

    Ok(AnalysisRunPreflight {
        source_ids: request.source_ids.clone(),
        message_count: message_sizes.len(),
        estimated_input_chars,
        estimated_chunks,
        limits,
    })
}

pub(crate) fn preflight_limit_error(preflight: &AnalysisRunPreflight) -> Option<String> {
    let exceeds_messages = preflight.message_count > preflight.limits.max_messages_per_run;
    let exceeds_chunks = preflight.estimated_chunks > preflight.limits.max_chunks_per_run;
    let exceeds_chars =
        preflight.estimated_input_chars > preflight.limits.max_estimated_input_chars_per_run;

    if !(exceeds_messages || exceeds_chunks || exceeds_chars) {
        return None;
    }

    Some(format!(
        "Analysis scope is too large: {} documents, {} estimated chunks, \
         {} estimated input characters. \
         Narrow the period or choose a smaller source scope.",
        preflight.message_count, preflight.estimated_chunks, preflight.estimated_input_chars
    ))
}

pub(crate) async fn load_run_snapshot_messages(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> Result<Vec<CorpusMessage>, String> {
    let rows: Vec<StoredRunSnapshotRow> = sqlx::query_as(
        r#"
        SELECT
            item_id,
            source_id,
            external_id,
            author,
            published_at,
            ref,
            content_zstd,
            item_kind,
            source_type,
            source_subtype,
            metadata_zstd
        FROM analysis_run_messages
        WHERE run_id = ?
        ORDER BY published_at ASC, ref ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|row| {
            Ok(CorpusMessage {
                item_id: row.item_id,
                source_id: row.source_id,
                external_id: row.external_id,
                published_at: row.published_at,
                author: row.author,
                content: decompress_text(&row.content_zstd)?,
                r#ref: row.r#ref,
                item_kind: row.item_kind,
                source_type: row.source_type,
                source_subtype: row.source_subtype,
                metadata_zstd: row.metadata_zstd,
            })
        })
        .collect()
}

pub(crate) struct ListRunSnapshotMessagesRequest {
    pub(crate) run_id: i64,
    pub(crate) after: Option<AnalysisRunMessageCursor>,
    pub(crate) limit: usize,
    pub(crate) source_id: Option<i64>,
    pub(crate) around_ref: Option<String>,
}

fn decode_optional_metadata_json(
    metadata_zstd: Option<&[u8]>,
) -> Result<Option<serde_json::Value>, String> {
    let Some(bytes) = metadata_zstd else {
        return Ok(None);
    };

    let decompressed = decompress_bytes(bytes)?;
    serde_json::from_slice(&decompressed)
        .map(Some)
        .map_err(|e| format!("Failed to decode run message metadata JSON: {e}"))
}

fn run_message_from_snapshot_row(row: StoredRunSnapshotRow) -> Result<AnalysisRunMessage, String> {
    Ok(AnalysisRunMessage {
        item_id: row.item_id,
        source_id: row.source_id,
        external_id: row.external_id,
        author: row.author,
        published_at: row.published_at,
        r#ref: row.r#ref,
        content: decompress_text(&row.content_zstd)?,
        item_kind: row.item_kind,
        source_type: row.source_type,
        source_subtype: row.source_subtype,
        metadata_json: decode_optional_metadata_json(row.metadata_zstd.as_deref())?,
    })
}

pub(crate) async fn list_run_snapshot_messages_page(
    pool: &Pool<Sqlite>,
    request: ListRunSnapshotMessagesRequest,
) -> Result<AnalysisRunMessagesPage, String> {
    let limit = request.limit.clamp(1, 500);
    let fetch_limit = (limit + 1) as i64;

    let rows: Vec<StoredRunSnapshotRow> = if let Some(after) = request.after {
        sqlx::query_as(
            r#"
            SELECT
                item_id,
                source_id,
                external_id,
                author,
                published_at,
                ref,
                content_zstd,
                item_kind,
                source_type,
                source_subtype,
                metadata_zstd
            FROM analysis_run_messages
            WHERE run_id = ?
              AND (? IS NULL OR source_id = ?)
              AND (
                published_at > ?
                OR (published_at = ? AND ref > ?)
              )
            ORDER BY published_at ASC, ref ASC
            LIMIT ?
            "#,
        )
        .bind(request.run_id)
        .bind(request.source_id)
        .bind(request.source_id)
        .bind(after.published_at)
        .bind(after.published_at)
        .bind(after.r#ref)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?
    } else if let Some(around_ref) = request.around_ref.as_deref() {
        let around_cursor = sqlx::query_as::<_, (i64, String)>(
            r#"
            SELECT published_at, ref
            FROM analysis_run_messages
            WHERE run_id = ?
              AND (? IS NULL OR source_id = ?)
              AND ref = ?
            LIMIT 1
            "#,
        )
        .bind(request.run_id)
        .bind(request.source_id)
        .bind(request.source_id)
        .bind(around_ref)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .map(|(published_at, r#ref)| AnalysisRunMessageCursor {
            published_at,
            r#ref,
        });

        if let Some(around) = around_cursor {
            sqlx::query_as(
                r#"
                SELECT
                    item_id,
                    source_id,
                    external_id,
                    author,
                    published_at,
                    ref,
                    content_zstd,
                    item_kind,
                    source_type,
                    source_subtype,
                    metadata_zstd
                FROM analysis_run_messages
                WHERE run_id = ?
                  AND (? IS NULL OR source_id = ?)
                  AND (
                    published_at > ?
                    OR (published_at = ? AND ref >= ?)
                  )
                ORDER BY published_at ASC, ref ASC
                LIMIT ?
                "#,
            )
            .bind(request.run_id)
            .bind(request.source_id)
            .bind(request.source_id)
            .bind(around.published_at)
            .bind(around.published_at)
            .bind(around.r#ref)
            .bind(fetch_limit)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?
        } else {
            sqlx::query_as(
                r#"
                SELECT
                    item_id,
                    source_id,
                    external_id,
                    author,
                    published_at,
                    ref,
                    content_zstd,
                    item_kind,
                    source_type,
                    source_subtype,
                    metadata_zstd
                FROM analysis_run_messages
                WHERE run_id = ?
                  AND (? IS NULL OR source_id = ?)
                ORDER BY published_at ASC, ref ASC
                LIMIT ?
                "#,
            )
            .bind(request.run_id)
            .bind(request.source_id)
            .bind(request.source_id)
            .bind(fetch_limit)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?
        }
    } else {
        sqlx::query_as(
            r#"
            SELECT
                item_id,
                source_id,
                external_id,
                author,
                published_at,
                ref,
                content_zstd,
                item_kind,
                source_type,
                source_subtype,
                metadata_zstd
            FROM analysis_run_messages
            WHERE run_id = ?
              AND (? IS NULL OR source_id = ?)
            ORDER BY published_at ASC, ref ASC
            LIMIT ?
            "#,
        )
        .bind(request.run_id)
        .bind(request.source_id)
        .bind(request.source_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?
    };

    let has_more = rows.len() > limit;
    let page_rows = rows.into_iter().take(limit).collect::<Vec<_>>();
    let mut messages = Vec::with_capacity(page_rows.len());
    for row in page_rows {
        messages.push(run_message_from_snapshot_row(row)?);
    }

    let next_cursor = if has_more {
        messages.last().map(|message| AnalysisRunMessageCursor {
            published_at: message.published_at,
            r#ref: message.r#ref.clone(),
        })
    } else {
        None
    };

    Ok(AnalysisRunMessagesPage {
        messages,
        next_cursor,
        has_more,
    })
}

pub(crate) async fn load_run_corpus_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<CorpusMessage>, String> {
    let snapshot = load_run_snapshot_messages(pool, run.id).await?;
    if !snapshot.is_empty() {
        return Ok(snapshot);
    }

    let resolved = resolve_analysis_sources(pool, run.source_id, run.source_group_id)
        .await
        .map_err(|e| e.message)?;
    let request = CorpusLoadRequest {
        source_type: resolved.source_type,
        source_ids: resolved.source_ids,
        period_from: run.period_from,
        period_to: run.period_to,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
    };
    load_corpus_messages(pool, &request).await
}

pub(crate) async fn load_trace_resolution_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<CorpusMessage>, String> {
    let snapshot = load_run_snapshot_messages(pool, run.id).await?;
    if !snapshot.is_empty() {
        return Ok(snapshot);
    }

    if run.status == "completed" {
        return Ok(Vec::new());
    }

    load_run_corpus_messages(pool, run).await
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;

    use super::{
        estimate_message_input_chars, estimate_preflight_chunk_count,
        list_run_snapshot_messages_page, live_corpus_ref, load_corpus_messages,
        load_run_corpus_messages, load_run_snapshot_messages, load_trace_resolution_messages,
        preflight_analysis_run, preflight_limit_error, resolve_analysis_sources,
        resolve_run_source_ids, AnalysisRunPreflight, AnalysisRunPreflightLimits,
        CorpusLoadRequest, ListRunSnapshotMessagesRequest, YoutubeCorpusMode,
    };
    use crate::analysis::models::{AnalysisRunDetail, AnalysisRunMessageCursor, CorpusMessage};
    use crate::analysis::store::persist_run_snapshot;
    use crate::compression::{compress_json_bytes, compress_text};
    use crate::youtube::dto::{YoutubeAvailabilityStatus, YoutubeVideoForm, YoutubeVideoMetadata};

    fn sample_corpus() -> Vec<CorpusMessage> {
        vec![
            CorpusMessage {
                item_id: 11,
                source_id: 2,
                external_id: "100".to_string(),
                published_at: 1_710_000_000,
                author: Some("Alice".to_string()),
                content: "First frozen message".to_string(),
                r#ref: "s2-m100".to_string(),
                item_kind: Some("youtube_transcript".to_string()),
                source_type: Some("youtube".to_string()),
                source_subtype: Some("video".to_string()),
                metadata_zstd: Some(
                    compress_json_bytes(
                        br#"{"video_id":"video2","item_kind":"youtube_transcript"}"#,
                    )
                    .expect("compress metadata"),
                ),
            },
            CorpusMessage {
                item_id: 12,
                source_id: 4,
                external_id: "101".to_string(),
                published_at: 1_710_000_100,
                author: None,
                content: "Second frozen message".to_string(),
                r#ref: "s4-m101".to_string(),
                item_kind: Some("telegram_message".to_string()),
                source_type: Some("telegram".to_string()),
                source_subtype: Some("channel".to_string()),
                metadata_zstd: None,
            },
        ]
    }

    async fn snapshot_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_type TEXT NOT NULL DEFAULT 'telegram',
                source_subtype TEXT,
                external_id TEXT NOT NULL DEFAULT '',
                title TEXT,
                metadata_zstd BLOB
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create sources");
        sqlx::query(
            r#"
            INSERT INTO sources (id, source_type, source_subtype, external_id, title)
            VALUES (2, 'telegram', 'channel', 'telegram-2', 'Telegram 2'),
                   (4, 'telegram', 'channel', 'telegram-4', 'Telegram 4')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert default telegram sources");

        sqlx::query(
            r#"
            CREATE TABLE items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                item_kind TEXT NOT NULL DEFAULT 'telegram_message',
                author TEXT,
                published_at INTEGER NOT NULL,
                ingested_at INTEGER NOT NULL DEFAULT 0,
                content_kind TEXT NOT NULL DEFAULT 'text_only',
                has_media INTEGER NOT NULL DEFAULT 0,
                content_zstd BLOB,
                raw_data_zstd BLOB,
                media_kind TEXT,
                media_metadata_zstd BLOB
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create items");

        sqlx::query(
            r#"
            CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL DEFAULT 'telegram',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create groups");

        sqlx::query(
            r#"
            CREATE TABLE analysis_source_group_members (
                group_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create group members");

        sqlx::query(
            r#"
            CREATE TABLE youtube_playlist_items (
                playlist_source_id INTEGER NOT NULL,
                video_id TEXT NOT NULL,
                video_source_id INTEGER,
                position INTEGER,
                availability_status TEXT NOT NULL,
                is_removed_from_playlist BOOLEAN NOT NULL DEFAULT 0,
                metadata_zstd BLOB
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create youtube playlist items");

        sqlx::query(
            r#"
            CREATE TABLE youtube_transcript_segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                segment_index INTEGER NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER,
                text TEXT NOT NULL,
                chapter_index INTEGER,
                caption_language TEXT,
                caption_track_kind TEXT,
                is_auto_generated INTEGER NOT NULL DEFAULT 0,
                metadata_zstd BLOB,
                UNIQUE(item_id, segment_index)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create youtube transcript segments");
        crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;

        sqlx::query(
            r#"
            CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_type TEXT NOT NULL,
                scope_type TEXT NOT NULL,
                source_id INTEGER,
                source_group_id INTEGER,
                period_from INTEGER NOT NULL,
                period_to INTEGER NOT NULL,
                output_language TEXT NOT NULL,
                prompt_template_id INTEGER,
                prompt_template_version INTEGER NOT NULL,
                provider_profile TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
                status TEXT NOT NULL,
                result_markdown TEXT,
                trace_data_zstd BLOB,
                scope_label_snapshot TEXT,
                snapshot_captured_at TEXT,
                snapshot_error TEXT,
                error TEXT,
                created_at INTEGER NOT NULL,
                completed_at INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create runs");

        sqlx::query(
            r#"
            CREATE TABLE analysis_run_messages (
                run_id INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                author TEXT,
                published_at INTEGER NOT NULL,
                ref TEXT NOT NULL,
                content_zstd BLOB NOT NULL,
                item_kind TEXT,
                source_type TEXT,
                source_subtype TEXT,
                metadata_zstd BLOB,
                PRIMARY KEY (run_id, ref)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create run messages");

        pool
    }

    fn corpus_request(
        source_type: &str,
        source_ids: Vec<i64>,
        youtube_corpus_mode: YoutubeCorpusMode,
    ) -> CorpusLoadRequest {
        CorpusLoadRequest {
            source_type: source_type.to_string(),
            source_ids,
            period_from: 1_700_000_000,
            period_to: 1_800_000_000,
            youtube_corpus_mode,
        }
    }

    async fn rebuild_documents_for_sources(pool: &sqlx::SqlitePool, source_ids: &[i64]) {
        crate::sources::test_support::create_analysis_documents_table(pool).await;
        for source_id in source_ids {
            crate::analysis_documents::rebuild_analysis_documents_for_source(pool, *source_id)
                .await
                .unwrap_or_else(|error| panic!("rebuild source {source_id}: {error}"));
        }
    }

    fn youtube_metadata_zstd(video_id: &str, title: &str, description: Option<&str>) -> Vec<u8> {
        let metadata = YoutubeVideoMetadata {
            video_id: video_id.to_string(),
            canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
            title: Some(title.to_string()),
            channel_title: Some("Channel".to_string()),
            channel_id: Some("UCdemo".to_string()),
            channel_handle: Some("@channel".to_string()),
            channel_url: Some("https://www.youtube.com/@channel".to_string()),
            author_display: Some("Channel".to_string()),
            published_at: Some("2026-05-01".to_string()),
            duration_seconds: Some(120),
            description: description.map(ToString::to_string),
            thumbnail_url: None,
            tags: Vec::new(),
            chapters: Vec::new(),
            view_count: None,
            like_count: None,
            comment_count: None,
            category: None,
            video_form: YoutubeVideoForm::Regular,
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: serde_json::json!({ "id": video_id }),
        };
        let json = serde_json::to_vec(&metadata).expect("serialize youtube metadata");
        compress_json_bytes(&json).expect("compress youtube metadata")
    }

    async fn insert_youtube_video_source(pool: &SqlitePool, source_id: i64) {
        insert_youtube_video_source_with_typed_metadata(
            pool,
            source_id,
            &format!("video{source_id}"),
            &format!("Video {source_id}"),
            None,
            Some("2026-05-01"),
        )
        .await;
    }

    async fn insert_youtube_video_source_with_typed_metadata(
        pool: &SqlitePool,
        source_id: i64,
        video_id: &str,
        title: &str,
        description: Option<&str>,
        published_at: Option<&str>,
    ) {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd)
             VALUES (?, 'youtube', 'video', ?, ?, ?)",
        )
        .bind(source_id)
        .bind(video_id)
        .bind(title)
        .bind(youtube_metadata_zstd(
            video_id,
            title,
            description,
        ))
        .execute(pool)
        .await
        .expect("insert youtube video source");
        insert_typed_youtube_video_source(
            pool,
            source_id,
            video_id,
            title,
            description,
            published_at,
        )
        .await;
    }

    async fn insert_typed_youtube_video_source(
        pool: &SqlitePool,
        source_id: i64,
        video_id: &str,
        title: &str,
        description: Option<&str>,
        published_at: Option<&str>,
    ) {
        sqlx::query(
            r#"
            INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, channel_title,
                channel_handle, published_at, description, video_form, availability_status
            )
            VALUES (?, ?, ?, ?, 'Channel', '@channel', ?, ?, 'regular', 'available')
            "#,
        )
        .bind(source_id)
        .bind(video_id)
        .bind(format!("https://www.youtube.com/watch?v={video_id}"))
        .bind(title)
        .bind(published_at)
        .bind(description)
        .execute(pool)
        .await
        .expect("insert typed youtube video source");
    }

    async fn insert_youtube_transcript_segment(
        pool: &SqlitePool,
        item_id: i64,
        source_id: i64,
        start_ms: i64,
        text: &str,
    ) {
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text,
                caption_language, caption_track_kind, is_auto_generated
             )
             VALUES (?, ?, 0, ?, ?, ?, 'en', 'manual', 0)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(start_ms)
        .bind(start_ms + 1_000)
        .bind(text)
        .execute(pool)
        .await
        .expect("insert youtube transcript segment");
    }

    #[tokio::test]
    async fn youtube_description_rows_use_typed_metadata_with_corrupt_source_blob() {
        let pool = snapshot_pool().await;
        insert_youtube_video_source_with_typed_metadata(
            &pool,
            401,
            "video401",
            "Typed title",
            Some("Typed description"),
            Some("2026-05-17"),
        )
        .await;
        sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = 401")
            .execute(&pool)
            .await
            .expect("corrupt source blob");
        rebuild_documents_for_sources(&pool, &[401]).await;

        let request = CorpusLoadRequest {
            source_type: "youtube".to_string(),
            source_ids: vec![401],
            period_from: 1,
            period_to: i64::MAX,
            youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        };
        let messages = load_corpus_messages(&pool, &request)
            .await
            .expect("load descriptions");

        assert_eq!(messages.len(), 1);
        assert!(messages[0].content.contains("Typed description"));
        assert!(messages[0]
            .content
            .contains("URL: https://www.youtube.com/watch?v=video401"));
    }

    #[tokio::test]
    async fn youtube_description_missing_typed_metadata_skips_without_decoding_source_blob() {
        let pool = snapshot_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd) VALUES (402, 'youtube', 'video', 'video402', 'Generic title', x'00')",
        )
        .execute(&pool)
        .await
        .expect("insert source");
        rebuild_documents_for_sources(&pool, &[402]).await;

        let request = CorpusLoadRequest {
            source_type: "youtube".to_string(),
            source_ids: vec![402],
            period_from: 1,
            period_to: i64::MAX,
            youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        };
        let messages = load_corpus_messages(&pool, &request)
            .await
            .expect("load descriptions");

        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn youtube_transcript_segment_evidence_uses_typed_source_context() {
        let pool = snapshot_pool().await;
        insert_youtube_video_source_with_typed_metadata(
            &pool,
            403,
            "video403",
            "Typed title",
            None,
            Some("2026-05-17"),
        )
        .await;
        sqlx::query("UPDATE sources SET title = 'Generic title' WHERE id = 403")
            .execute(&pool)
            .await
            .expect("set generic source title");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at)
             VALUES (9001, 403, 'transcript:video403:en:manual', 'youtube_transcript', 'Channel', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert transcript item");
        insert_youtube_transcript_segment(&pool, 9001, 403, 12_000, "segment text").await;
        sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = 403")
            .execute(&pool)
            .await
            .expect("corrupt source blob");
        rebuild_documents_for_sources(&pool, &[403]).await;

        let request = CorpusLoadRequest {
            source_type: "youtube".to_string(),
            source_ids: vec![403],
            period_from: 1,
            period_to: i64::MAX,
            youtube_corpus_mode: YoutubeCorpusMode::TranscriptOnly,
        };
        let messages = load_corpus_messages(&pool, &request)
            .await
            .expect("load transcript segments");

        let metadata_json = decode_message_metadata_for_test(&messages[0]);
        assert_eq!(metadata_json["video_id"], "video403");
        assert_eq!(
            metadata_json["canonical_url"],
            "https://www.youtube.com/watch?v=video403"
        );
        assert_eq!(metadata_json["title"], "Typed title");
        assert_eq!(metadata_json["segment_start_ms"], 12_000);
    }

    fn decode_message_metadata_for_test(message: &CorpusMessage) -> serde_json::Value {
        let bytes = message.metadata_zstd.as_deref().expect("message metadata");
        let decoded = crate::compression::decompress_bytes(bytes).expect("decompress metadata");
        serde_json::from_slice(&decoded).expect("parse metadata")
    }

    fn sample_run() -> AnalysisRunDetail {
        AnalysisRunDetail {
            id: 1,
            run_type: "report".to_string(),
            scope_type: "source_group".to_string(),
            source_id: None,
            source_title: None,
            source_group_id: Some(9),
            source_group_name: Some("Live group".to_string()),
            scope_label: "Frozen group".to_string(),
            period_from: 1_700_000_000,
            period_to: 1_800_000_000,
            output_language: "English".to_string(),
            prompt_template_id: Some(1),
            prompt_template_name: Some("Default".to_string()),
            prompt_template_version: 1,
            provider_profile: "default".to_string(),
            provider: "gemini".to_string(),
            model: "gemini-2.5-flash".to_string(),
            youtube_corpus_mode: "transcript_description".to_string(),
            status: "completed".to_string(),
            result_markdown: Some("Saved report".to_string()),
            error: None,
            has_trace_data: true,
            snapshot_state: Some(crate::analysis::models::AnalysisSnapshotState::Captured),
            snapshot_captured_at: Some("2026-05-18T10:00:00Z".to_string()),
            snapshot_error: None,
            created_at: 1_710_000_500,
            completed_at: Some(1_710_000_600),
            scope_label_snapshot: Some("Frozen group".to_string()),
            snapshot_message_count: 1,
        }
    }

    #[test]
    fn estimated_message_chars_match_report_chunk_accounting() {
        let message = CorpusMessage {
            item_id: 11,
            source_id: 2,
            external_id: "100".to_string(),
            published_at: 1_710_000_000,
            author: Some("Alice".to_string()),
            content: "First live document".to_string(),
            r#ref: "s2-i11".to_string(),
            item_kind: Some("telegram_message".to_string()),
            source_type: Some("telegram".to_string()),
            source_subtype: Some("channel".to_string()),
            metadata_zstd: None,
        };

        assert_eq!(
            estimate_message_input_chars(
                &message.content,
                &message.r#ref,
                message.author.as_deref()
            ),
            message.content.len() + message.r#ref.len() + "Alice".len() + 64
        );
    }

    #[test]
    fn estimated_chunk_count_matches_chunk_boundary_behavior() {
        assert_eq!(estimate_preflight_chunk_count(&[], 16_000), 0);
        assert_eq!(estimate_preflight_chunk_count(&[8_000, 7_000], 16_000), 1);
        assert_eq!(estimate_preflight_chunk_count(&[8_000, 9_000], 16_000), 2);
        assert_eq!(estimate_preflight_chunk_count(&[20_000], 16_000), 1);
    }

    #[test]
    fn default_preflight_limits_are_conservative() {
        let limits = AnalysisRunPreflightLimits::default();

        assert_eq!(limits.max_messages_per_run, 10_000);
        assert_eq!(limits.max_chunks_per_run, 80);
        assert_eq!(limits.max_estimated_input_chars_per_run, 1_500_000);
        assert_eq!(limits.max_background_requests_per_run, 80);
    }

    #[test]
    fn preflight_limit_error_reports_all_scale_dimensions() {
        let preflight = AnalysisRunPreflight {
            source_ids: vec![1],
            message_count: 73_102,
            estimated_input_chars: 6_200_000,
            estimated_chunks: 381,
            limits: AnalysisRunPreflightLimits::default(),
        };

        let error = preflight_limit_error(&preflight).expect("limit error");

        assert!(error.contains("73102 documents"));
        assert!(error.contains("381 estimated chunks"));
        assert!(error.contains("6200000 estimated input characters"));
        assert!(error.contains("Narrow the period or choose a smaller source scope"));
    }

    #[test]
    fn preflight_limit_error_allows_runs_within_limits() {
        let preflight = AnalysisRunPreflight {
            source_ids: vec![1],
            message_count: 1_000,
            estimated_input_chars: 100_000,
            estimated_chunks: 10,
            limits: AnalysisRunPreflightLimits::default(),
        };

        assert_eq!(preflight_limit_error(&preflight), None);
    }

    #[tokio::test]
    async fn run_snapshot_roundtrips_frozen_corpus() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id,
                run_type,
                scope_type,
                source_group_id,
                period_from,
                period_to,
                output_language,
                prompt_template_version,
                provider_profile,
                provider,
                model,
                status,
                created_at
            )
            VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");

        let corpus = sample_corpus();
        persist_run_snapshot(&pool, 1, "Frozen group", &corpus)
            .await
            .expect("persist snapshot");

        let loaded = load_run_snapshot_messages(&pool, 1)
            .await
            .expect("load snapshot");

        assert_eq!(loaded.len(), corpus.len());
        assert_eq!(loaded[0].r#ref, "s2-m100");
        assert_eq!(loaded[1].content, "Second frozen message");
    }

    #[tokio::test]
    async fn list_run_snapshot_messages_page_reads_saved_snapshot_only() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id, run_type, scope_type, source_group_id, period_from, period_to,
                output_language, prompt_template_version, provider_profile, provider,
                model, status, created_at
            )
            VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");

        persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
            .await
            .expect("persist snapshot");

        let page = list_run_snapshot_messages_page(
            &pool,
            ListRunSnapshotMessagesRequest {
                run_id: 1,
                after: None,
                limit: 1,
                source_id: None,
                around_ref: None,
            },
        )
        .await
        .expect("load first page");

        assert_eq!(page.messages.len(), 1);
        assert_eq!(page.messages[0].content, "First frozen message");
        assert_eq!(page.messages[0].source_type.as_deref(), Some("youtube"));
        assert_eq!(
            page.messages[0]
                .metadata_json
                .as_ref()
                .and_then(|value| value.get("video_id"))
                .and_then(|value| value.as_str()),
            Some("video2")
        );
        assert!(page.has_more);

        let second_page = list_run_snapshot_messages_page(
            &pool,
            ListRunSnapshotMessagesRequest {
                run_id: 1,
                after: page.next_cursor,
                limit: 1,
                source_id: None,
                around_ref: None,
            },
        )
        .await
        .expect("load second page");

        assert_eq!(second_page.messages.len(), 1);
        assert_eq!(second_page.messages[0].content, "Second frozen message");
        assert!(!second_page.has_more);
        assert_eq!(second_page.next_cursor, None);

        let filtered_page = list_run_snapshot_messages_page(
            &pool,
            ListRunSnapshotMessagesRequest {
                run_id: 1,
                after: None,
                limit: 25,
                source_id: Some(4),
                around_ref: None,
            },
        )
        .await
        .expect("load source-filtered page");

        assert_eq!(filtered_page.messages.len(), 1);
        assert_eq!(filtered_page.messages[0].source_id, 4);
        assert_eq!(filtered_page.messages[0].content, "Second frozen message");
    }

    #[tokio::test]
    async fn list_run_snapshot_messages_page_starts_at_around_ref() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id, run_type, scope_type, source_group_id, period_from, period_to,
                output_language, prompt_template_version, provider_profile, provider,
                model, status, created_at
            )
            VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");

        persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
            .await
            .expect("persist snapshot");

        let page = list_run_snapshot_messages_page(
            &pool,
            ListRunSnapshotMessagesRequest {
                run_id: 1,
                after: None,
                limit: 10,
                source_id: None,
                around_ref: Some("s4-m101".to_string()),
            },
        )
        .await
        .expect("load around ref");

        assert_eq!(
            page.messages
                .iter()
                .map(|message| message.r#ref.as_str())
                .collect::<Vec<_>>(),
            vec!["s4-m101"]
        );
    }

    #[tokio::test]
    async fn list_run_snapshot_messages_page_does_not_fall_back_to_live_source() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id, run_type, scope_type, source_id, period_from, period_to,
                output_language, prompt_template_version, provider_profile, provider,
                model, status, created_at
            )
            VALUES (1, 'report', 'single_source', 2, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");

        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(11_i64)
        .bind(2_i64)
        .bind("100")
        .bind("telegram_message")
        .bind("Alice")
        .bind(1_710_000_000_i64)
        .bind(compress_text("Live source message").expect("compress live message"))
        .execute(&pool)
        .await
        .expect("insert live item");

        let page = list_run_snapshot_messages_page(
            &pool,
            ListRunSnapshotMessagesRequest {
                run_id: 1,
                after: None,
                limit: 25,
                source_id: None,
                around_ref: None,
            },
        )
        .await
        .expect("load snapshot-only page");

        assert_eq!(page.messages, Vec::new());
        assert_eq!(page.next_cursor, None);
        assert!(!page.has_more);
    }

    #[tokio::test]
    async fn trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot() {
        let pool = snapshot_pool().await;
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(11_i64)
        .bind(2_i64)
        .bind("100")
        .bind("telegram_message")
        .bind("Alice")
        .bind(1_710_000_000_i64)
        .bind(compress_text("Live source text").expect("compress live text"))
        .execute(&pool)
        .await
        .expect("insert live item");

        let messages = load_trace_resolution_messages(&pool, &sample_run())
            .await
            .expect("load trace resolution messages");

        assert!(messages.is_empty());
    }

    #[test]
    fn run_message_cursor_uses_ref_and_published_at() {
        let cursor = AnalysisRunMessageCursor {
            published_at: 1_710_000_000,
            r#ref: "s7-i1".to_string(),
        };

        assert_eq!(cursor.published_at, 1_710_000_000);
        assert_eq!(cursor.r#ref, "s7-i1");
    }

    #[tokio::test]
    async fn resolve_run_source_ids_prefers_snapshot_over_live_group_membership() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_source_groups (id, name, created_at, updated_at)
            VALUES (9, 'Live group', 1, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert group");
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES (9, 77, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert live member");
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id,
                run_type,
                scope_type,
                source_group_id,
                period_from,
                period_to,
                output_language,
                prompt_template_version,
                provider_profile,
                provider,
                model,
                status,
                created_at
            )
            VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");

        persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
            .await
            .expect("persist snapshot");

        let source_ids = resolve_run_source_ids(&pool, &sample_run())
            .await
            .expect("resolve source ids");

        assert_eq!(source_ids, vec![2, 4]);
    }

    #[tokio::test]
    async fn load_run_corpus_messages_uses_snapshot_when_available() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id,
                run_type,
                scope_type,
                source_group_id,
                period_from,
                period_to,
                output_language,
                prompt_template_version,
                provider_profile,
                provider,
                model,
                status,
                created_at
            )
            VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");
        persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
            .await
            .expect("persist snapshot");

        let corpus = load_run_corpus_messages(&pool, &sample_run())
            .await
            .expect("load run corpus");

        assert_eq!(corpus.len(), 2);
        assert_eq!(corpus[0].external_id, "100");
        assert_eq!(corpus[0].item_kind.as_deref(), Some("youtube_transcript"));
        assert_eq!(corpus[0].source_type.as_deref(), Some("youtube"));
        assert_eq!(corpus[0].source_subtype.as_deref(), Some("video"));
        assert!(corpus[0].metadata_zstd.is_some());
        assert_eq!(corpus[1].r#ref, "s4-m101");
    }

    #[tokio::test]
    async fn live_corpus_refs_use_local_item_ids() {
        let pool = snapshot_pool().await;
        let first_content = compress_text("First live document").expect("compress first");
        let second_content = compress_text("Second live document").expect("compress second");
        sqlx::query(
            r#"
            INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(11_i64)
        .bind(2_i64)
        .bind("100")
        .bind("Alice")
        .bind(1_710_000_000_i64)
        .bind(first_content)
        .execute(&pool)
        .await
        .expect("insert first item");
        sqlx::query(
            r#"
            INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(12_i64)
        .bind(4_i64)
        .bind("101")
        .bind(Option::<String>::None)
        .bind(1_710_000_100_i64)
        .bind(second_content)
        .execute(&pool)
        .await
        .expect("insert second item");
        rebuild_documents_for_sources(&pool, &[2, 4]).await;

        let request = corpus_request(
            "telegram",
            vec![2, 4],
            YoutubeCorpusMode::TranscriptDescription,
        );
        let corpus = load_corpus_messages(&pool, &request)
            .await
            .expect("load live corpus");

        assert_eq!(corpus.len(), 2);
        assert_eq!(corpus[0].r#ref, "s2-i11");
        assert_eq!(corpus[1].r#ref, "s4-i12");
    }

    #[tokio::test]
    async fn preflight_counts_eligible_text_messages_for_sources() {
        let pool = snapshot_pool().await;
        let first_content = compress_text("First live document").expect("compress first");
        let second_content = compress_text("Second live document").expect("compress second");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(11_i64)
        .bind(2_i64)
        .bind("100")
        .bind("Alice")
        .bind(1_710_000_000_i64)
        .bind(first_content)
        .execute(&pool)
        .await
        .expect("insert first item");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(12_i64)
        .bind(4_i64)
        .bind("101")
        .bind(Option::<String>::None)
        .bind(1_710_000_100_i64)
        .bind(second_content)
        .execute(&pool)
        .await
        .expect("insert second item");
        rebuild_documents_for_sources(&pool, &[2, 4]).await;

        let preflight = preflight_analysis_run(
            &pool,
            &corpus_request(
                "telegram",
                vec![2, 4],
                YoutubeCorpusMode::TranscriptDescription,
            ),
            16_000,
            AnalysisRunPreflightLimits::default(),
        )
        .await
        .expect("preflight");

        assert_eq!(preflight.source_ids, vec![2, 4]);
        assert_eq!(preflight.message_count, 2);
        assert_eq!(preflight.estimated_chunks, 1);
        assert!(preflight.estimated_input_chars > 0);
    }

    #[tokio::test]
    async fn preflight_ref_format_matches_corpus_loader_ref_format() {
        let pool = snapshot_pool().await;
        let content = compress_text("Test message").expect("compress");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(11_i64)
        .bind(2_i64)
        .bind("100")
        .bind(Option::<String>::None)
        .bind(1_710_000_000_i64)
        .bind(content)
        .execute(&pool)
        .await
        .expect("insert item");
        rebuild_documents_for_sources(&pool, &[2]).await;

        let corpus = load_corpus_messages(
            &pool,
            &corpus_request(
                "telegram",
                vec![2],
                YoutubeCorpusMode::TranscriptDescription,
            ),
        )
        .await
        .expect("load corpus");

        assert_eq!(
            corpus[0].r#ref,
            live_corpus_ref(corpus[0].source_id, corpus[0].item_id)
        );
    }

    #[tokio::test]
    async fn preflight_ignores_media_only_items_without_text_content() {
        let pool = snapshot_pool().await;
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, NULL)",
        )
        .bind(11_i64)
        .bind(2_i64)
        .bind("100")
        .bind("Alice")
        .bind(1_710_000_000_i64)
        .execute(&pool)
        .await
        .expect("insert media-only item");
        rebuild_documents_for_sources(&pool, &[2]).await;

        let preflight = preflight_analysis_run(
            &pool,
            &corpus_request(
                "telegram",
                vec![2],
                YoutubeCorpusMode::TranscriptDescription,
            ),
            16_000,
            AnalysisRunPreflightLimits::default(),
        )
        .await
        .expect("preflight");

        assert_eq!(preflight.message_count, 0);
        assert_eq!(preflight.estimated_chunks, 0);
        assert_eq!(preflight.estimated_input_chars, 0);
    }

    #[tokio::test]
    async fn load_corpus_messages_orders_transcript_segments_by_document_order_not_ref() {
        let pool = snapshot_pool().await;
        insert_youtube_video_source(&pool, 20).await;
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
             VALUES (?, ?, ?, 'youtube_transcript', 'Channel', ?, ?)",
        )
        .bind(21_i64)
        .bind(20_i64)
        .bind("transcript:v1:en:manual")
        .bind(1_710_000_000_i64)
        .bind(compress_text("full transcript").expect("compress"))
        .execute(&pool)
        .await
        .expect("insert transcript item");
        insert_youtube_transcript_segment(&pool, 21, 20, 900, "early").await;
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text,
                caption_language, caption_track_kind, is_auto_generated
             ) VALUES (21, 20, 1, 10000, 11000, 'late', 'en', 'manual', 0)",
        )
        .execute(&pool)
        .await
        .expect("insert late segment");
        rebuild_documents_for_sources(&pool, &[20]).await;

        let corpus = load_corpus_messages(
            &pool,
            &corpus_request("youtube", vec![20], YoutubeCorpusMode::TranscriptOnly),
        )
        .await
        .expect("load corpus");

        assert_eq!(
            corpus
                .iter()
                .map(|message| message.r#ref.as_str())
                .collect::<Vec<_>>(),
            vec!["s20-i21@900ms", "s20-i21@10000ms"]
        );
    }

    #[test]
    fn youtube_corpus_mode_parses_wire_values_and_defaults() {
        assert_eq!(
            YoutubeCorpusMode::from_wire(None).expect("default mode"),
            YoutubeCorpusMode::TranscriptDescription
        );
        assert_eq!(
            YoutubeCorpusMode::from_wire(Some("transcript_only")).expect("transcript only"),
            YoutubeCorpusMode::TranscriptOnly
        );
        assert_eq!(
            YoutubeCorpusMode::from_wire(Some("transcript_description_comments"))
                .expect("comments mode"),
            YoutubeCorpusMode::TranscriptDescriptionComments
        );
        assert!(YoutubeCorpusMode::from_wire(Some("all_text")).is_err());
        assert_eq!(
            YoutubeCorpusMode::TranscriptOnly.as_wire(),
            "transcript_only"
        );
        assert_eq!(
            YoutubeCorpusMode::TranscriptDescription.as_wire(),
            "transcript_description"
        );
        assert_eq!(
            YoutubeCorpusMode::TranscriptDescriptionComments.as_wire(),
            "transcript_description_comments"
        );
    }

    #[tokio::test]
    async fn load_corpus_messages_filters_telegram_to_telegram_message() {
        let pool = snapshot_pool().await;
        insert_youtube_video_source(&pool, 20).await;
        let telegram_text = compress_text("Telegram message").expect("compress telegram");
        let youtube_text = compress_text("YouTube comment").expect("compress youtube");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(11_i64)
        .bind(2_i64)
        .bind("100")
        .bind("telegram_message")
        .bind("Alice")
        .bind(1_710_000_000_i64)
        .bind(telegram_text)
        .bind(12_i64)
        .bind(20_i64)
        .bind("comment:c1")
        .bind("youtube_comment")
        .bind("Bob")
        .bind(1_710_000_001_i64)
        .bind(youtube_text)
        .execute(&pool)
        .await
        .expect("insert mixed items");
        rebuild_documents_for_sources(&pool, &[2, 20]).await;

        let corpus = load_corpus_messages(
            &pool,
            &corpus_request(
                "telegram",
                vec![2, 20],
                YoutubeCorpusMode::TranscriptDescription,
            ),
        )
        .await
        .expect("load telegram corpus");

        assert_eq!(corpus.len(), 1);
        assert_eq!(corpus[0].external_id, "100");
        assert_eq!(corpus[0].content, "Telegram message");
    }

    #[tokio::test]
    async fn load_corpus_messages_filters_youtube_transcript_only_to_transcripts() {
        let pool = snapshot_pool().await;
        insert_youtube_video_source(&pool, 20).await;
        let transcript = compress_text("Transcript text").expect("compress transcript");
        let comment = compress_text("Comment text").expect("compress comment");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(21_i64)
        .bind(20_i64)
        .bind("transcript:v1:en:manual")
        .bind("youtube_transcript")
        .bind("Channel")
        .bind(1_710_000_000_i64)
        .bind(transcript)
        .bind(22_i64)
        .bind(20_i64)
        .bind("comment:c1")
        .bind("youtube_comment")
        .bind("Commenter")
        .bind(1_710_000_001_i64)
        .bind(comment)
        .execute(&pool)
        .await
        .expect("insert youtube items");
        insert_youtube_transcript_segment(&pool, 21, 20, 754_000, "Transcript text").await;
        rebuild_documents_for_sources(&pool, &[20]).await;

        let corpus = load_corpus_messages(
            &pool,
            &corpus_request("youtube", vec![20], YoutubeCorpusMode::TranscriptOnly),
        )
        .await
        .expect("load youtube transcript-only corpus");

        assert_eq!(corpus.len(), 1);
        assert_eq!(corpus[0].external_id, "transcript:v1:en:manual");
        assert_eq!(corpus[0].r#ref, "s20-i21@754000ms");
    }

    #[tokio::test]
    async fn load_corpus_messages_includes_youtube_comment_only_in_comments_mode() {
        let pool = snapshot_pool().await;
        insert_youtube_video_source(&pool, 20).await;
        let transcript = compress_text("Transcript text").expect("compress transcript");
        let comment = compress_text("Comment text").expect("compress comment");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(21_i64)
        .bind(20_i64)
        .bind("transcript:v1:en:manual")
        .bind("youtube_transcript")
        .bind("Channel")
        .bind(1_710_000_000_i64)
        .bind(transcript)
        .bind(22_i64)
        .bind(20_i64)
        .bind("comment:c1")
        .bind("youtube_comment")
        .bind("Commenter")
        .bind(1_710_000_001_i64)
        .bind(comment)
        .execute(&pool)
        .await
        .expect("insert youtube items");
        insert_youtube_transcript_segment(&pool, 21, 20, 754_000, "Transcript text").await;
        rebuild_documents_for_sources(&pool, &[20]).await;

        let without_comments = load_corpus_messages(
            &pool,
            &corpus_request(
                "youtube",
                vec![20],
                YoutubeCorpusMode::TranscriptDescription,
            ),
        )
        .await
        .expect("load youtube transcript+description corpus");
        let with_comments = load_corpus_messages(
            &pool,
            &corpus_request(
                "youtube",
                vec![20],
                YoutubeCorpusMode::TranscriptDescriptionComments,
            ),
        )
        .await
        .expect("load youtube comments corpus");

        assert_eq!(without_comments.len(), 1);
        assert_eq!(with_comments.len(), 2);
        assert!(with_comments
            .iter()
            .any(|message| message.external_id == "comment:c1"));
    }

    #[tokio::test]
    async fn playlist_expansion_excludes_unlinked_and_removed_rows() {
        let pool = snapshot_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (10, 'youtube', 'playlist', 'PLdemo', 'Playlist'),
                    (20, 'youtube', 'video', 'video1', 'Video 1'),
                    (21, 'youtube', 'video', 'video2', 'Video 2')",
        )
        .execute(&pool)
        .await
        .expect("insert sources");
        sqlx::query(
            "INSERT INTO youtube_playlist_items (
                playlist_source_id, video_id, video_source_id, position, availability_status, is_removed_from_playlist
             )
             VALUES (10, 'video1', 20, 1, 'available', 0),
                    (10, 'missing', NULL, 2, 'unavailable_unknown', 0),
                    (10, 'removed', 21, 3, 'removed_from_playlist', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert playlist rows");

        let resolved = resolve_analysis_sources(&pool, Some(10), None)
            .await
            .expect("resolve playlist scope");

        assert_eq!(resolved.source_type, "youtube");
        assert_eq!(resolved.source_ids, vec![20]);
        assert_eq!(resolved.skipped_unlinked_playlist_items, 1);
    }

    #[tokio::test]
    async fn description_mode_creates_synthetic_description_message() {
        let pool = snapshot_pool().await;
        insert_youtube_video_source_with_typed_metadata(
            &pool,
            20,
            "video1",
            "Video 1",
            Some("Description body"),
            Some("2026-05-01"),
        )
        .await;
        rebuild_documents_for_sources(&pool, &[20]).await;

        let corpus = load_corpus_messages(
            &pool,
            &corpus_request(
                "youtube",
                vec![20],
                YoutubeCorpusMode::TranscriptDescription,
            ),
        )
        .await
        .expect("load youtube corpus");

        assert_eq!(corpus.len(), 1);
        assert_eq!(corpus[0].item_id, 0);
        assert_eq!(corpus[0].external_id, "description:video1");
        assert_eq!(corpus[0].r#ref, "s20-i0");
        assert!(corpus[0].content.contains("YouTube video description"));
        assert!(corpus[0].content.contains("Description body"));
    }

    #[tokio::test]
    async fn preflight_count_matches_loader_for_youtube_corpus_modes() {
        let pool = snapshot_pool().await;
        insert_youtube_video_source_with_typed_metadata(
            &pool,
            20,
            "video1",
            "Video 1",
            Some("Description body"),
            Some("2026-05-01"),
        )
        .await;
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
             VALUES (?, ?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(21_i64)
        .bind(20_i64)
        .bind("transcript:v1:en:manual")
        .bind("youtube_transcript")
        .bind("Channel")
        .bind(1_710_000_000_i64)
        .bind(compress_text("Transcript text").expect("compress transcript"))
        .bind(22_i64)
        .bind(20_i64)
        .bind("comment:c1")
        .bind("youtube_comment")
        .bind("Commenter")
        .bind(1_710_000_001_i64)
        .bind(compress_text("Comment text").expect("compress comment"))
        .execute(&pool)
        .await
        .expect("insert youtube items");
        insert_youtube_transcript_segment(&pool, 21, 20, 754_000, "Transcript text").await;
        rebuild_documents_for_sources(&pool, &[20]).await;

        for mode in [
            YoutubeCorpusMode::TranscriptOnly,
            YoutubeCorpusMode::TranscriptDescription,
            YoutubeCorpusMode::TranscriptDescriptionComments,
        ] {
            let request = corpus_request("youtube", vec![20], mode);
            let corpus = load_corpus_messages(&pool, &request)
                .await
                .expect("load corpus");
            let preflight = preflight_analysis_run(
                &pool,
                &request,
                16_000,
                AnalysisRunPreflightLimits::default(),
            )
            .await
            .expect("preflight");

            assert_eq!(preflight.message_count, corpus.len(), "mode {mode:?}");
        }
    }
}
