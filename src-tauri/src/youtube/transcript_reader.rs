use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Sqlite};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeTranscriptSegmentCursor {
    pub start_ms: i64,
    pub segment_id: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListYoutubeTranscriptSegmentsRequest {
    pub source_id: i64,
    pub after: Option<YoutubeTranscriptSegmentCursor>,
    pub limit: i64,
    pub search_query: Option<String>,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct YoutubeTranscriptSegmentDto {
    pub id: i64,
    pub source_id: i64,
    pub item_id: i64,
    pub segment_index: i64,
    pub start_ms: i64,
    pub end_ms: Option<i64>,
    pub text: String,
    pub caption_language: Option<String>,
    pub caption_track_kind: Option<String>,
    pub is_auto_generated: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct YoutubeTranscriptSegmentsPage {
    pub segments: Vec<YoutubeTranscriptSegmentDto>,
    pub next_cursor: Option<YoutubeTranscriptSegmentCursor>,
    pub has_more: bool,
}

pub(crate) async fn list_youtube_transcript_segments_from_pool(
    pool: &sqlx::SqlitePool,
    request: ListYoutubeTranscriptSegmentsRequest,
) -> AppResult<YoutubeTranscriptSegmentsPage> {
    let limit = request.limit.clamp(1, 200);
    let fetch_limit = limit + 1;
    let search = request
        .search_query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            format!(
                "%{}%",
                value
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            )
        });

    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            id,
            source_id,
            item_id,
            segment_index,
            start_ms,
            end_ms,
            text,
            caption_language,
            caption_track_kind,
            is_auto_generated
        FROM youtube_transcript_segments
        WHERE source_id =
        "#,
    );
    query.push_bind(request.source_id);

    if let Some(after) = request.after {
        query.push(" AND (start_ms, id) > (");
        query.push_bind(after.start_ms);
        query.push(", ");
        query.push_bind(after.segment_id);
        query.push(")");
    }

    if let Some(search) = search {
        query.push(" AND text LIKE ");
        query.push_bind(search);
        query.push(" ESCAPE '\\'");
    }

    query.push(" ORDER BY start_ms ASC, id ASC LIMIT ");
    query.push_bind(fetch_limit);

    let mut segments: Vec<YoutubeTranscriptSegmentDto> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    let has_more = segments.len() > limit as usize;
    if has_more {
        segments.truncate(limit as usize);
    }

    let next_cursor = if has_more {
        segments
            .last()
            .map(|segment| YoutubeTranscriptSegmentCursor {
                start_ms: segment.start_ms,
                segment_id: segment.id,
            })
    } else {
        None
    };

    Ok(YoutubeTranscriptSegmentsPage {
        segments,
        next_cursor,
        has_more,
    })
}

#[tauri::command]
pub async fn list_youtube_transcript_segments(
    handle: AppHandle,
    request: ListYoutubeTranscriptSegmentsRequest,
) -> AppResult<YoutubeTranscriptSegmentsPage> {
    let pool = get_pool(&handle).await?;
    list_youtube_transcript_segments_from_pool(&pool, request).await
}

#[cfg(test)]
mod tests {
    use super::{list_youtube_transcript_segments_from_pool, ListYoutubeTranscriptSegmentsRequest};

    async fn transcript_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("create memory pool");
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
                metadata_zstd BLOB
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create segments table");
        pool
    }

    async fn insert_segment(pool: &sqlx::SqlitePool, source_id: i64, start_ms: i64, text: &str) {
        sqlx::query(
            r#"
            INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text,
                caption_language, caption_track_kind, is_auto_generated
            ) VALUES (?, ?, ?, ?, ?, ?, 'en', 'manual', 0)
            "#,
        )
        .bind(10_i64)
        .bind(source_id)
        .bind(start_ms / 1000)
        .bind(start_ms)
        .bind(start_ms + 2000)
        .bind(text)
        .execute(pool)
        .await
        .expect("insert segment");
    }

    #[tokio::test]
    async fn list_youtube_transcript_segments_pages_by_time_and_id() {
        let pool = transcript_pool().await;
        insert_segment(&pool, 20, 1000, "first").await;
        insert_segment(&pool, 20, 2000, "second").await;
        insert_segment(&pool, 20, 3000, "third").await;
        insert_segment(&pool, 21, 1000, "other source").await;

        let first = list_youtube_transcript_segments_from_pool(
            &pool,
            ListYoutubeTranscriptSegmentsRequest {
                source_id: 20,
                after: None,
                limit: 2,
                search_query: None,
            },
        )
        .await
        .expect("load first page");

        assert_eq!(first.segments.len(), 2);
        assert!(first.has_more);
        assert_eq!(
            first.next_cursor.as_ref().map(|cursor| cursor.start_ms),
            Some(2000)
        );

        let second = list_youtube_transcript_segments_from_pool(
            &pool,
            ListYoutubeTranscriptSegmentsRequest {
                source_id: 20,
                after: first.next_cursor,
                limit: 2,
                search_query: None,
            },
        )
        .await
        .expect("load second page");

        assert_eq!(second.segments.len(), 1);
        assert_eq!(second.segments[0].text, "third");
        assert!(!second.has_more);
    }

    #[tokio::test]
    async fn list_youtube_transcript_segments_filters_by_search() {
        let pool = transcript_pool().await;
        insert_segment(&pool, 20, 1000, "alpha topic").await;
        insert_segment(&pool, 20, 2000, "beta topic").await;

        let page = list_youtube_transcript_segments_from_pool(
            &pool,
            ListYoutubeTranscriptSegmentsRequest {
                source_id: 20,
                after: None,
                limit: 20,
                search_query: Some("beta".to_string()),
            },
        )
        .await
        .expect("search transcript");

        assert_eq!(page.segments.len(), 1);
        assert_eq!(page.segments[0].text, "beta topic");
    }

    #[tokio::test]
    async fn search_escapes_existing_backslashes_before_like_wildcards() {
        let pool = transcript_pool().await;
        insert_segment(&pool, 20, 1000, r"literal \_ marker").await;
        insert_segment(&pool, 20, 2000, r"literal \x marker").await;

        let page = list_youtube_transcript_segments_from_pool(
            &pool,
            ListYoutubeTranscriptSegmentsRequest {
                source_id: 20,
                after: None,
                limit: 20,
                search_query: Some(r"\_".to_string()),
            },
        )
        .await
        .expect("search transcript");

        assert_eq!(page.segments.len(), 1);
        assert_eq!(page.segments[0].text, r"literal \_ marker");
    }
}
