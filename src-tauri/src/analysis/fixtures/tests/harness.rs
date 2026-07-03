use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
pub(super) async fn fixture_pool() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    crate::migrations::apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable foreign keys");
    pool
}

pub(super) async fn count(pool: &Pool<Sqlite>, sql: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| panic!("count query failed: {sql}: {error}"))
}

#[tokio::test]
async fn fixture_test_pool_has_required_tables() {
    let pool = fixture_pool().await;

    for table in [
        "accounts",
        "sources",
        "items",
        "telegram_forum_topics",
        "youtube_transcript_segments",
        "youtube_playlist_items",
        "analysis_prompt_templates",
        "analysis_source_groups",
        "analysis_source_group_members",
        "analysis_runs",
        "analysis_run_messages",
        "analysis_chat_messages",
        "app_settings",
    ] {
        let exists = count(
            &pool,
            &format!(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '{table}'"
            ),
        )
        .await;
        assert_eq!(exists, 1, "missing table {table}");
    }
}
