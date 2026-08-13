#[cfg(test)]
mod snapshots_tests;
#[cfg(test)]
mod test_support;

#[cfg(test)]
pub(crate) mod source_adapter_test_support {
    pub(crate) async fn migrated_pool() -> sqlx::SqlitePool {
        super::test_support::migrated_pool().await
    }

    pub(crate) async fn insert_youtube_video(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        video_id: &str,
    ) {
        super::test_support::insert_youtube_video(pool, source_id, video_id).await;
    }

    pub(crate) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
        super::test_support::insert_playlist(pool, playlist_source_id).await;
    }

    pub(crate) async fn insert_playlist_item(
        pool: &sqlx::SqlitePool,
        playlist_source_id: i64,
        video_source_id: Option<i64>,
        video_id: &str,
        position: i64,
    ) {
        super::test_support::insert_playlist_item(
            pool,
            playlist_source_id,
            video_source_id,
            video_id,
            position,
        )
        .await;
    }

    pub(crate) async fn insert_transcript(pool: &sqlx::SqlitePool, source_id: i64, text: &str) {
        super::test_support::insert_transcript(pool, source_id, text).await;
    }

    pub(crate) async fn insert_comment(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        external_id: &str,
        published_at: i64,
        text: &str,
    ) {
        super::test_support::insert_comment(pool, source_id, external_id, published_at, text).await;
    }
}
