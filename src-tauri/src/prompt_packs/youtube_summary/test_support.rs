use extractum_core::compression::compress_text;

use crate::migrations::apply_all_migrations_for_test_pool;

pub(super) async fn migrated_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");
    pool
}

pub(super) async fn insert_youtube_video(pool: &sqlx::SqlitePool, source_id: i64, video_id: &str) {
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, external_id, title,
            is_active, is_member, created_at
         )
         VALUES (?, 'youtube', 'video', ?, ?, 1, 0, 1)",
    )
    .bind(source_id)
    .bind(video_id)
    .bind(format!("Video {video_id}"))
    .execute(pool)
    .await
    .expect("insert source");

    sqlx::query(
        "INSERT INTO youtube_video_sources (
            source_id, video_id, canonical_url, title, description,
            video_form, availability_status
         )
         VALUES (?, ?, ?, ?, 'Description', 'regular', 'available')",
    )
    .bind(source_id)
    .bind(video_id)
    .bind(format!("https://www.youtube.com/watch?v={video_id}"))
    .bind(format!("Video {video_id}"))
    .execute(pool)
    .await
    .expect("insert video metadata");
}

pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, external_id, title,
            is_active, is_member, created_at
         )
         VALUES (?, 'youtube', 'playlist', 'playlist-1', 'Playlist', 1, 0, 1)",
    )
    .bind(playlist_source_id)
    .execute(pool)
    .await
    .expect("insert playlist source");

    sqlx::query(
        "INSERT INTO youtube_playlist_sources (
            source_id, playlist_id, canonical_url, title, availability_status
         )
         VALUES (?, 'playlist-1', 'https://www.youtube.com/playlist?list=playlist-1', 'Playlist', 'available')",
    )
    .bind(playlist_source_id)
    .execute(pool)
    .await
    .expect("insert playlist metadata");
}

pub(super) async fn insert_playlist_item(
    pool: &sqlx::SqlitePool,
    playlist_source_id: i64,
    video_source_id: Option<i64>,
    video_id: &str,
    position: i64,
) {
    sqlx::query(
        "INSERT INTO youtube_playlist_items (
            playlist_source_id, video_source_id, video_id, position,
            title_snapshot, availability_status, is_removed_from_playlist
         )
         VALUES (?, ?, ?, ?, ?, 'available', 0)",
    )
    .bind(playlist_source_id)
    .bind(video_source_id)
    .bind(video_id)
    .bind(position)
    .bind(format!("Video {video_id}"))
    .execute(pool)
    .await
    .expect("insert playlist item");
}

pub(super) async fn insert_transcript(pool: &sqlx::SqlitePool, source_id: i64, text: &str) {
    let item_id: i64 = sqlx::query_scalar(
        "INSERT INTO items (
            source_id, external_id, published_at, ingested_at, item_kind
         )
         VALUES (?, ?, 1, 1, 'youtube_transcript')
         RETURNING id",
    )
    .bind(source_id)
    .bind(format!("item-{source_id}"))
    .fetch_one(pool)
    .await
    .expect("insert transcript item");

    sqlx::query(
        "INSERT INTO youtube_transcript_segments (
            item_id, source_id, segment_index, start_ms, end_ms, text
         )
         VALUES (?, ?, 0, 0, 1000, ?)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(text)
    .execute(pool)
    .await
    .expect("insert transcript segment");
}

pub(super) async fn insert_comment(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    external_id: &str,
    published_at: i64,
    text: &str,
) {
    sqlx::query(
        "INSERT INTO items (
            source_id, external_id, author, published_at, ingested_at,
            content_zstd, content_kind, has_media, item_kind
         )
         VALUES (?, ?, 'Alice', ?, 1, ?, 'text_only', 0, 'youtube_comment')",
    )
    .bind(source_id)
    .bind(external_id)
    .bind(published_at)
    .bind(compress_text(text).expect("compress comment"))
    .execute(pool)
    .await
    .expect("insert comment");
}

pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    let pool = migrated_pool().await;
    insert_youtube_video(&pool, 901, "v-ready").await;
    insert_transcript(&pool, 901, "Ready transcript").await;
    pool
}

pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    let pool = test_pool_with_ready_video().await;
    insert_comment(&pool, 901, "comment-newer", 20, "newer").await;
    insert_comment(&pool, 901, "comment-oldest", 10, "oldest").await;
    insert_comment(&pool, 901, "comment-middle", 15, "middle").await;
    pool
}
