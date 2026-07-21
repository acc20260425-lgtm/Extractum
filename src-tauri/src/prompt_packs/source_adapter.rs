use sqlx::SqlitePool;

use crate::compression::decompress_text;
use crate::error::AppError;
use extractum_prompt_packs::{
    CommentBodyReadRequest, CommentCandidateReadRequest, PromptPackCommentCandidate,
    PromptPackPlaylistItemRecord, PromptPackPortFuture, PromptPackSourceReader,
    PromptPackSourceRecord, PromptPackTranscriptSegment, PromptPackYoutubeVideoRecord,
    YoutubeVideoReadRequest,
};

#[derive(Clone)]
pub struct AppPromptPackSourceReader {
    pool: SqlitePool,
}

impl AppPromptPackSourceReader {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl PromptPackSourceReader for AppPromptPackSourceReader {
    fn load_source(
        &self,
        source_id: i64,
    ) -> PromptPackPortFuture<'_, Option<PromptPackSourceRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (i64, String, Option<String>, Option<String>)>(
                "SELECT id, source_type, source_subtype, title FROM sources WHERE id = ?",
            )
            .bind(source_id)
            .fetch_optional(&pool)
            .await
            .map(|row| {
                row.map(|(id, source_type, source_subtype, title)| {
                    PromptPackSourceRecord::new(id, source_type, source_subtype, title)
                })
            })
            .map_err(AppError::database)
        })
    }

    fn load_video(
        &self,
        request: YoutubeVideoReadRequest,
    ) -> PromptPackPortFuture<'_, Option<PromptPackYoutubeVideoRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<
                _,
                (
                    i64,
                    String,
                    String,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                ),
            >(
                "SELECT yvs.source_id, yvs.video_id, yvs.canonical_url, yvs.title,
                        yvs.channel_title, yvs.published_at, yvs.description
                 FROM youtube_video_sources yvs
                 JOIN sources ON sources.id = yvs.source_id
                 WHERE yvs.source_id = ?",
            )
            .bind(request.source_id())
            .fetch_optional(&pool)
            .await
            .map(|row| {
                row.map(
                    |(
                        source_id,
                        video_id,
                        canonical_url,
                        title,
                        channel_title,
                        published_at,
                        description,
                    )| {
                        PromptPackYoutubeVideoRecord::new(
                            source_id,
                            video_id,
                            canonical_url,
                            title,
                            channel_title,
                            published_at,
                            description,
                        )
                    },
                )
            })
            .map_err(AppError::database)
        })
    }

    fn load_playlist_items(
        &self,
        playlist_source_id: i64,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackPlaylistItemRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (Option<i64>, String, Option<String>)>(
                "SELECT video_source_id, video_id, title_snapshot
                 FROM youtube_playlist_items
                 WHERE playlist_source_id = ? AND is_removed_from_playlist = 0
                 ORDER BY position ASC, id ASC",
            )
            .bind(playlist_source_id)
            .fetch_all(&pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(video_source_id, video_id, title)| {
                        PromptPackPlaylistItemRecord::new(video_source_id, video_id, title)
                    })
                    .collect()
            })
            .map_err(AppError::database)
        })
    }

    fn load_transcript_segments(
        &self,
        source_id: i64,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackTranscriptSegment>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (i64, i64, String)>(
                "SELECT start_ms, end_ms, text
                 FROM youtube_transcript_segments
                 WHERE source_id = ?
                 ORDER BY segment_index ASC, id ASC",
            )
            .bind(source_id)
            .fetch_all(&pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(start_ms, end_ms, text)| {
                        PromptPackTranscriptSegment::new(start_ms, end_ms, text)
                    })
                    .collect()
            })
            .map_err(AppError::database)
        })
    }

    fn select_comment_candidates(
        &self,
        request: CommentCandidateReadRequest,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackCommentCandidate>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let rows = sqlx::query_as::<_, (Option<String>, Option<Vec<u8>>)>(
                "SELECT external_id, content_zstd
                 FROM items
                 WHERE source_id = ? AND item_kind = 'youtube_comment'
                 ORDER BY published_at IS NULL ASC, published_at ASC, external_id ASC, id ASC
                 LIMIT ?",
            )
            .bind(request.source_id())
            .bind(request.limit())
            .fetch_all(&pool)
            .await
            .map_err(AppError::database)?;

            Ok(rows
                .into_iter()
                .map(|(external_id, content_zstd)| {
                    let body = content_zstd
                        .as_deref()
                        .and_then(|bytes| decompress_text(bytes).ok())
                        .unwrap_or_default();
                    PromptPackCommentCandidate::new(external_id, body)
                })
                .collect())
        })
    }

    fn load_comment_body(
        &self,
        request: CommentBodyReadRequest,
    ) -> PromptPackPortFuture<'_, String> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let Some(external_id) = request.external_id() else {
                return Ok(String::new());
            };
            let bytes = sqlx::query_scalar::<_, Vec<u8>>(
                "SELECT content_zstd FROM items
                 WHERE source_id = ? AND item_kind = 'youtube_comment' AND external_id = ?
                 LIMIT 1",
            )
            .bind(request.source_id())
            .bind(external_id)
            .fetch_optional(&pool)
            .await
            .map_err(AppError::database)?;
            Ok(bytes
                .as_deref()
                .and_then(|bytes| decompress_text(bytes).ok())
                .unwrap_or_default())
        })
    }
}

#[cfg(test)]
#[path = "youtube_summary/test_support.rs"]
mod source_adapter_test_support;

#[cfg(test)]
mod tests {
    use super::source_adapter_test_support::{
        insert_comment, insert_playlist, insert_playlist_item, insert_transcript,
        insert_youtube_video, migrated_pool,
    };
    use super::AppPromptPackSourceReader;
    use crate::compression::compress_text;
    use extractum_prompt_packs::{
        CommentBodyReadRequest, CommentCandidateReadRequest, PromptPackCommentCandidate,
        PromptPackPlaylistItemRecord, PromptPackSourceReader, PromptPackSourceRecord,
        PromptPackTranscriptSegment, PromptPackYoutubeVideoRecord, YoutubeVideoReadRequest,
    };

    #[tokio::test]
    async fn load_source_preserves_caller_order_missing_rows_and_nullables() {
        let pool = migrated_pool().await;
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             ) VALUES
                (11, 'rss', NULL, 'source-11', NULL, 1, 0, 1),
                (12, 'rss', 'feed', 'source-12', 'Feed 12', 1, 0, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert source fixtures");
        let reader = AppPromptPackSourceReader::new(pool);

        let mut rows = Vec::new();
        for source_id in [12, 999, 11] {
            rows.push(reader.load_source(source_id).await.expect("load source"));
        }

        assert_eq!(
            rows,
            vec![
                Some(PromptPackSourceRecord::new(
                    12,
                    "rss".to_string(),
                    Some("feed".to_string()),
                    Some("Feed 12".to_string()),
                )),
                None,
                Some(PromptPackSourceRecord::new(
                    11,
                    "rss".to_string(),
                    None,
                    None,
                )),
            ]
        );
    }

    #[tokio::test]
    async fn load_video_maps_full_nullable_metadata_and_missing_rows() {
        let pool = migrated_pool().await;
        insert_youtube_video(&pool, 901, "video-901").await;
        sqlx::query(
            "UPDATE youtube_video_sources
             SET title = NULL,
                 channel_title = 'Channel 901',
                 published_at = '2026-07-20T10:00:00Z',
                 description = NULL
             WHERE source_id = 901",
        )
        .execute(&pool)
        .await
        .expect("update nullable video metadata");
        let reader = AppPromptPackSourceReader::new(pool);

        let video = reader
            .load_video(YoutubeVideoReadRequest::new(901))
            .await
            .expect("load video");
        let missing = reader
            .load_video(YoutubeVideoReadRequest::new(902))
            .await
            .expect("load missing video");

        assert_eq!(
            video,
            Some(PromptPackYoutubeVideoRecord::new(
                901,
                "video-901".to_string(),
                "https://www.youtube.com/watch?v=video-901".to_string(),
                None,
                Some("Channel 901".to_string()),
                Some("2026-07-20T10:00:00Z".to_string()),
                None,
            ))
        );
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows() {
        let pool = migrated_pool().await;
        insert_playlist(&pool, 701).await;
        insert_youtube_video(&pool, 901, "video-linked").await;
        insert_playlist_item(&pool, 701, Some(901), "video-linked", 2).await;
        insert_playlist_item(&pool, 701, None, "video-unlinked-a", 1).await;
        insert_playlist_item(&pool, 701, None, "video-unlinked-b", 1).await;
        insert_playlist_item(&pool, 701, None, "video-removed", 0).await;
        sqlx::query(
            "UPDATE youtube_playlist_items
             SET is_removed_from_playlist = 1
             WHERE video_id = 'video-removed'",
        )
        .execute(&pool)
        .await
        .expect("mark playlist row removed");
        let reader = AppPromptPackSourceReader::new(pool);

        let items = reader
            .load_playlist_items(701)
            .await
            .expect("load playlist items");

        assert_eq!(
            items,
            vec![
                PromptPackPlaylistItemRecord::new(
                    None,
                    "video-unlinked-a".to_string(),
                    Some("Video video-unlinked-a".to_string()),
                ),
                PromptPackPlaylistItemRecord::new(
                    None,
                    "video-unlinked-b".to_string(),
                    Some("Video video-unlinked-b".to_string()),
                ),
                PromptPackPlaylistItemRecord::new(
                    Some(901),
                    "video-linked".to_string(),
                    Some("Video video-linked".to_string()),
                ),
            ]
        );
    }

    #[tokio::test]
    async fn load_transcript_segments_orders_segment_index_then_row_id() {
        let pool = migrated_pool().await;
        insert_youtube_video(&pool, 901, "video-901").await;
        insert_transcript(&pool, 901, "segment-zero").await;
        let item_id: i64 = sqlx::query_scalar(
            "INSERT INTO items (
                source_id, external_id, published_at, ingested_at, item_kind
             ) VALUES (901, 'transcript-second', 1, 1, 'youtube_transcript')
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("insert second transcript item");
        for (segment_index, start_ms, end_ms, text) in [
            (2_i64, 2_000_i64, 3_000_i64, "segment-two"),
            (1_i64, 1_000_i64, 2_000_i64, "segment-one"),
        ] {
            sqlx::query(
                "INSERT INTO youtube_transcript_segments (
                    item_id, source_id, segment_index, start_ms, end_ms, text
                 ) VALUES (?, 901, ?, ?, ?, ?)",
            )
            .bind(item_id)
            .bind(segment_index)
            .bind(start_ms)
            .bind(end_ms)
            .bind(text)
            .execute(&pool)
            .await
            .expect("insert transcript segment");
        }
        let reader = AppPromptPackSourceReader::new(pool);

        let segments = reader
            .load_transcript_segments(901)
            .await
            .expect("load transcript segments");

        assert_eq!(
            segments,
            vec![
                PromptPackTranscriptSegment::new(0, 1_000, "segment-zero".to_string()),
                PromptPackTranscriptSegment::new(1_000, 2_000, "segment-one".to_string()),
                PromptPackTranscriptSegment::new(2_000, 3_000, "segment-two".to_string()),
            ]
        );
    }

    #[tokio::test]
    async fn select_comment_candidates_applies_limit_order_and_decompression_fallback() {
        let pool = migrated_pool().await;
        insert_youtube_video(&pool, 901, "video-901").await;
        insert_comment(&pool, 901, "comment-later", 30, "later").await;
        insert_comment(&pool, 901, "comment-first", 10, "first body").await;
        sqlx::query(
            "INSERT INTO items (
                source_id, external_id, author, published_at, ingested_at,
                content_zstd, content_kind, has_media, item_kind
             ) VALUES (901, 'comment-corrupt', 'Alice', 20, 1, X'010203',
                'text_only', 0, 'youtube_comment')",
        )
        .execute(&pool)
        .await
        .expect("insert corrupt comment");
        let reader = AppPromptPackSourceReader::new(pool);

        let comments = reader
            .select_comment_candidates(CommentCandidateReadRequest::new(901, 2))
            .await
            .expect("select comments");

        assert_eq!(
            comments,
            vec![
                PromptPackCommentCandidate::new(
                    Some("comment-first".to_string()),
                    "first body".to_string(),
                ),
                PromptPackCommentCandidate::new(Some("comment-corrupt".to_string()), String::new(),),
            ]
        );
    }

    #[tokio::test]
    async fn load_comment_body_performs_a_fresh_read_with_decompression_fallback() {
        let pool = migrated_pool().await;
        insert_youtube_video(&pool, 901, "video-901").await;
        insert_comment(&pool, 901, "comment-fresh", 10, "first body").await;
        let reader = AppPromptPackSourceReader::new(pool.clone());
        let request = || CommentBodyReadRequest::new(901, Some("comment-fresh".to_string()));

        let first = reader
            .load_comment_body(request())
            .await
            .expect("load first body");
        sqlx::query(
            "UPDATE items SET content_zstd = ?
             WHERE source_id = 901 AND external_id = 'comment-fresh'",
        )
        .bind(compress_text("second body").expect("compress replacement"))
        .execute(&pool)
        .await
        .expect("replace comment body");
        let second = reader
            .load_comment_body(request())
            .await
            .expect("load fresh body");
        sqlx::query(
            "UPDATE items SET content_zstd = X'010203'
             WHERE source_id = 901 AND external_id = 'comment-fresh'",
        )
        .execute(&pool)
        .await
        .expect("corrupt comment body");
        let corrupt = reader
            .load_comment_body(request())
            .await
            .expect("load corrupt body");
        let missing = reader
            .load_comment_body(CommentBodyReadRequest::new(
                901,
                Some("comment-missing".to_string()),
            ))
            .await
            .expect("load missing body");
        let absent_id = reader
            .load_comment_body(CommentBodyReadRequest::new(901, None))
            .await
            .expect("load absent id");

        assert_eq!(first, "first body");
        assert_eq!(second, "second body");
        assert_eq!(corrupt, "");
        assert_eq!(missing, "");
        assert_eq!(absent_id, "");
    }
}
