use super::super::{clear_analysis_redesign_fixtures_in_pool, AnalysisRedesignFixtureSummary};
use super::harness::{count, fixture_pool};
use sqlx::{Pool, Sqlite};
async fn insert_minimal_clear_fixture(pool: &Pool<Sqlite>) {
    sqlx::query(
        "INSERT INTO accounts (id, label, api_id, api_hash, created_at)
         VALUES (10, '__analysis_redesign_fixture__ Account', 100001, '', 10)",
    )
    .execute(pool)
    .await
    .expect("insert fixture account");
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, account_id, external_id,
            title, last_synced_at, is_active, is_member, created_at
         )
         VALUES (
            20, 'youtube', 'video', NULL,
            '__analysis_redesign_fixture__:clear-source',
            '__analysis_redesign_fixture__ Clear Source',
            10, 1, 0, 10
         )",
    )
    .execute(pool)
    .await
    .expect("insert fixture source");
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at, ingested_at,
            content_kind, has_media
         )
         VALUES (
            30, 20, '__analysis_redesign_fixture__:clear-item',
            'youtube_transcript', 'Fixture', 10, 10, 'text_only', 0
         )",
    )
    .execute(pool)
    .await
    .expect("insert fixture item");
    sqlx::query(
        "INSERT INTO youtube_transcript_segments (
            item_id, source_id, segment_index, start_ms, end_ms, text
         )
         VALUES (30, 20, 0, 1000, 2000, 'Fixture clear segment')",
    )
    .execute(pool)
    .await
    .expect("insert fixture transcript segment");
    sqlx::query(
        "INSERT INTO youtube_playlist_items (
            playlist_source_id, video_source_id, video_id, position, availability_status,
            is_removed_from_playlist, created_at, updated_at
         )
         VALUES (20, NULL, '__analysis_redesign_fixture__:video', 1, 'available', 0, 10, 10)",
    )
    .execute(pool)
    .await
    .expect("insert fixture playlist item");
    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, last_seen_at, updated_at
         )
         VALUES (20, 1, 1, '__analysis_redesign_fixture__ Topic', 10, 10)",
    )
    .execute(pool)
    .await
    .expect("insert fixture topic");
    sqlx::query(
        "INSERT INTO analysis_prompt_templates (
            id, name, template_kind, body, version, is_builtin, created_at, updated_at
         )
         VALUES (
            40, '__analysis_redesign_fixture__ Template', 'report', 'Body', 1, 0, 10, 10
         )",
    )
    .execute(pool)
    .await
    .expect("insert fixture template");
    sqlx::query(
        "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
         VALUES (50, '__analysis_redesign_fixture__ Group', 'youtube', 10, 10)",
    )
    .execute(pool)
    .await
    .expect("insert fixture group");
    sqlx::query(
        "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
         VALUES (50, 20, 10)",
    )
    .execute(pool)
    .await
    .expect("insert fixture group member");
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_id, period_from, period_to, output_language,
            prompt_template_id, prompt_template_version, provider_profile, provider, model,
            youtube_corpus_mode, status, result_markdown, scope_label_snapshot, created_at,
            completed_at
         )
         VALUES (
            60, 'report', 'single_source', 20, 1, 2, 'English', 40, 1,
            '__analysis_redesign_fixture__', 'gemini', 'model', 'transcript_description',
            'completed', 'Fixture result', '__analysis_redesign_fixture__ Run', 10, 11
         )",
    )
    .execute(pool)
    .await
    .expect("insert fixture run");
    sqlx::query(
        "INSERT INTO analysis_run_messages (
            run_id, item_id, source_id, external_id, author, published_at, ref, content_zstd
         )
         VALUES (
            60, 30, 20, '__analysis_redesign_fixture__:clear-item',
            'Fixture', 10, 's20-i30', x'28B52FFD0000010000'
         )",
    )
    .execute(pool)
    .await
    .expect("insert fixture run message");
    sqlx::query(
        "INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
         VALUES (60, 'user', 'Fixture chat', 10)",
    )
    .execute(pool)
    .await
    .expect("insert fixture chat message");
    for key in [
        "llm.profile.__analysis_redesign_fixture__.provider",
        "llm.profile.__analysis_redesign_fixture__.default_model",
        "llm.profile.__analysis_redesign_fixture__.base_url",
    ] {
        sqlx::query("INSERT INTO app_settings (key, value) VALUES (?, 'fixture')")
            .bind(key)
            .execute(pool)
            .await
            .expect("insert fixture profile setting");
    }
}

#[tokio::test]
async fn clear_removes_only_fixture_rows_and_is_idempotent() {
    let pool = fixture_pool().await;
    sqlx::query(
        "INSERT INTO accounts (label, api_id, api_hash, created_at)
         VALUES ('Personal', 12345, '', 1)",
    )
    .execute(&pool)
    .await
    .expect("insert non-fixture account");
    sqlx::query(
        "INSERT INTO sources (
            source_type, source_subtype, account_id, external_id,
            title, is_active, is_member, created_at
         )
         VALUES ('telegram', 'channel', 1, 'real-source', 'Real Source', 1, 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert non-fixture source");

    insert_minimal_clear_fixture(&pool).await;

    let cleared = clear_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("clear fixtures");
    let second_clear = clear_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("clear fixtures again");

    assert_eq!(cleared.accounts, 1);
    assert_eq!(cleared.sources, 1);
    assert_eq!(cleared.source_groups, 1);
    assert_eq!(cleared.prompt_templates, 1);
    assert_eq!(cleared.runs, 1);
    assert_eq!(cleared.snapshot_messages, 1);
    assert_eq!(cleared.chat_messages, 1);
    assert_eq!(cleared.youtube_transcript_segments, 1);
    assert_eq!(cleared.youtube_playlist_items, 1);
    assert_eq!(second_clear, AnalysisRedesignFixtureSummary::default());
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM accounts WHERE label = 'Personal'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sources WHERE title = 'Real Source'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn clear_preserves_non_fixture_groups_and_members() {
    let pool = fixture_pool().await;

    let real_account_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (label, api_id, api_hash, created_at)
         VALUES ('Personal', 1, '', 1)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("insert non-fixture account");

    let real_source_id: i64 = sqlx::query_scalar(
        "INSERT INTO sources (
            source_type, source_subtype, account_id, external_id, title,
            is_active, is_member, created_at
         )
         VALUES ('telegram', 'channel', ?, 'real-source', 'Real Source', 1, 1, 1)
         RETURNING id",
    )
    .bind(real_account_id)
    .fetch_one(&pool)
    .await
    .expect("insert non-fixture source");

    let real_group_id: i64 = sqlx::query_scalar(
        "INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)
         VALUES ('Real Group', 'telegram', 1, 1)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("insert non-fixture group");

    sqlx::query(
        "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
         VALUES (?, ?, 1)",
    )
    .bind(real_group_id)
    .bind(real_source_id)
    .execute(&pool)
    .await
    .expect("insert non-fixture group member");

    insert_minimal_clear_fixture(&pool).await;

    clear_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("clear fixtures");

    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_source_groups WHERE name = 'Real Group'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_source_group_members member
             JOIN analysis_source_groups group_row ON group_row.id = member.group_id
             JOIN sources source_row ON source_row.id = member.source_id
             WHERE group_row.name = 'Real Group' AND source_row.title = 'Real Source'",
        )
        .await,
        1
    );
}

#[tokio::test]
async fn clear_deletes_child_rows_through_fixture_parent_ids() {
    let pool = fixture_pool().await;
    insert_minimal_clear_fixture(&pool).await;

    sqlx::query(
        "INSERT INTO analysis_runs (
            run_type, scope_type, source_id, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, youtube_corpus_mode,
            status, result_markdown, scope_label_snapshot, created_at, completed_at
         )
         VALUES (
            'report', 'single_source', NULL, 1, 2, 'English', 1, 'default', 'gemini',
            'model', 'transcript_description', 'completed',
            '__analysis_redesign_fixture__ text in user content',
            'User Run', 3, 4
         )",
    )
    .execute(&pool)
    .await
    .expect("insert non-fixture run with marker text");

    clear_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("clear fixtures");

    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_runs WHERE scope_label_snapshot = 'User Run'"
        )
        .await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_run_messages").await,
        0
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_chat_messages").await,
        0
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM youtube_transcript_segments").await,
        0
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM youtube_playlist_items").await,
        0
    );
}
