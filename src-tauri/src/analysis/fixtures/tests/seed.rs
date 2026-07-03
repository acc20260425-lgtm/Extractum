use super::super::{
    seed_analysis_redesign_fixtures_in_pool, FIXTURE_MARKER, TELEGRAM_CHANNEL_LABEL,
    YOUTUBE_PLAYLIST_LABEL, YOUTUBE_VIDEO_LABEL,
};
use super::harness::{count, fixture_pool};
#[tokio::test]
async fn seed_creates_safe_account_prompt_profile_sources_and_group() {
    let pool = fixture_pool().await;

    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let account: (String, i64, String, Option<String>) =
        sqlx::query_as("SELECT label, api_id, api_hash, phone FROM accounts WHERE label = ?")
            .bind(TELEGRAM_CHANNEL_LABEL.replace("Telegram Channel", "Telegram Account"))
            .fetch_one(&pool)
            .await
            .expect("load fixture account");
    assert!(account.0.starts_with(FIXTURE_MARKER));
    assert_eq!(account.1, 100_001);
    assert_eq!(account.2, "");
    assert_eq!(account.3, None);

    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%'"
        )
        .await,
        4
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sources WHERE source_type = 'telegram' AND account_id IS NOT NULL"
        )
        .await,
        2
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_source_groups WHERE name = '__analysis_redesign_fixture__ Telegram Source Group'"
        )
        .await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_source_group_members").await,
        2
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_prompt_templates WHERE name LIKE '__analysis_redesign_fixture__%' AND template_kind = 'report'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM app_settings WHERE key LIKE 'llm.profile.__analysis_redesign_fixture__.%'"
        )
        .await,
        3
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM app_settings WHERE key LIKE 'llm.profile.__analysis_redesign_fixture__.api_key'"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn seed_creates_post_sync_reader_content() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM items WHERE item_kind = 'telegram_message'"
        )
        .await,
        4
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM telegram_forum_topics").await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM items WHERE has_media = 1 AND media_metadata_zstd IS NOT NULL"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM items WHERE reply_to_top_id IS NOT NULL OR reply_to_msg_id IS NOT NULL"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM items WHERE reaction_count IS NOT NULL"
        )
        .await,
        2
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM items WHERE item_kind = 'youtube_transcript'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM items WHERE item_kind = 'youtube_comment'"
        )
        .await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM youtube_transcript_segments").await,
        3
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM youtube_playlist_items").await,
        2
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%' AND last_synced_at IS NOT NULL"
        )
        .await,
        4
    );
}

#[tokio::test]
async fn seed_creates_valid_typed_youtube_detail_metadata() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let video_source_id: i64 = sqlx::query_scalar("SELECT id FROM sources WHERE title = ?")
        .bind(YOUTUBE_VIDEO_LABEL)
        .fetch_one(&pool)
        .await
        .expect("load fixture video source");
    let video_detail =
        crate::youtube::detail::get_youtube_video_detail_from_pool(&pool, video_source_id)
            .await
            .expect("load fixture video detail");

    assert_eq!(
        video_detail.source_metadata.video_id,
        "analysis_fixture_video"
    );
    assert_eq!(
        video_detail.source_metadata.raw_metadata_json,
        Some(serde_json::json!({ "fixture": true }))
    );

    let playlist_source_id: i64 = sqlx::query_scalar("SELECT id FROM sources WHERE title = ?")
        .bind(YOUTUBE_PLAYLIST_LABEL)
        .fetch_one(&pool)
        .await
        .expect("load fixture playlist source");
    let playlist_detail =
        crate::youtube::detail::get_youtube_playlist_detail_from_pool(&pool, playlist_source_id)
            .await
            .expect("load fixture playlist detail");

    assert_eq!(playlist_detail.items.len(), 2);
    assert_eq!(playlist_detail.items[0].video_id, "analysis_fixture_video");
}

#[tokio::test]
async fn seed_creates_sources_that_pass_identity_repair() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let report = crate::sources::identity_repair::repair_source_identity(
        &pool,
        crate::sources::identity_repair::SourceIdentityRepairMode::Apply,
    )
    .await
    .expect("repair seeded fixture identities");

    assert!(report.fatal_errors.is_empty());
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM telegram_sources").await,
        2
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM telegram_sources WHERE source_subtype = 'channel' AND peer_kind = 'channel'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM telegram_sources WHERE source_subtype = 'supergroup' AND peer_kind = 'channel'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn compressed_fixture_fields_are_readable() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let content: Vec<u8> = sqlx::query_scalar(
        "SELECT content_zstd FROM items WHERE external_id LIKE '__analysis_redesign_fixture__:tg-channel-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("load content");
    let media: Vec<u8> = sqlx::query_scalar(
        "SELECT media_metadata_zstd FROM items WHERE media_metadata_zstd IS NOT NULL LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load media metadata");
    let raw: Vec<u8> = sqlx::query_scalar(
        "SELECT raw_data_zstd FROM items WHERE raw_data_zstd IS NOT NULL LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load raw data");

    assert!(crate::compression::decompress_text(&content)
        .expect("decompress content")
        .contains("fixture channel update"));
    assert!(String::from_utf8(
        crate::compression::decompress_bytes(&media).expect("decompress media")
    )
    .expect("media utf8")
    .contains("image/jpeg"));
    assert!(
        String::from_utf8(crate::compression::decompress_bytes(&raw).expect("decompress raw"))
            .expect("raw utf8")
            .contains("analysis_redesign_fixture")
    );
}

#[tokio::test]
async fn seed_creates_fixture_runs_with_statuses_templates_and_snapshots() {
    let pool = fixture_pool().await;
    let summary = seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    assert_eq!(summary.runs, 7);
    assert_eq!(summary.snapshot_messages, 4);
    assert_eq!(summary.chat_messages, 2);
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_runs WHERE prompt_template_id IS NOT NULL"
        )
        .await,
        7
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(DISTINCT status) FROM analysis_runs WHERE scope_label_snapshot LIKE '__analysis_redesign_fixture__%'"
        )
        .await,
        4
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_runs WHERE status = 'completed'"
        )
        .await,
        3
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_runs WHERE status = 'running'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_runs WHERE status = 'failed'"
        )
        .await,
        2
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_runs WHERE status = 'cancelled'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn seed_twice_keeps_one_deterministic_fixture_set() {
    let pool = fixture_pool().await;

    let first = seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures once");
    let second = seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures twice");

    assert_eq!(first, second);
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%'"
        )
        .await,
        4
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_runs WHERE scope_label_snapshot LIKE '__analysis_redesign_fixture__%'"
        )
        .await,
        7
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_run_messages").await,
        4
    );
}
