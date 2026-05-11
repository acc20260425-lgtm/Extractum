use serde::Serialize;
use sqlx::{Pool, Sqlite};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};

const FIXTURE_MARKER: &str = "__analysis_redesign_fixture__";
const FIXTURE_EXTERNAL_PREFIX: &str = "__analysis_redesign_fixture__:";
const FIXTURE_PROFILE_ID: &str = "__analysis_redesign_fixture__";
const FIXTURE_NOW: i64 = 1_778_400_000;
const FIXTURE_PERIOD_FROM: i64 = 1_777_968_000;
const FIXTURE_PERIOD_TO: i64 = 1_778_313_600;

const TELEGRAM_CHANNEL_LABEL: &str = "__analysis_redesign_fixture__ Telegram Channel";
const TELEGRAM_SUPERGROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Supergroup";
const YOUTUBE_VIDEO_LABEL: &str = "__analysis_redesign_fixture__ YouTube Video";
const YOUTUBE_PLAYLIST_LABEL: &str = "__analysis_redesign_fixture__ YouTube Playlist";
const TELEGRAM_GROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Group";
const COMPLETED_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Completed Snapshot Run";
const MISSING_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Missing Snapshot Run";
const RUNNING_RUN_LABEL: &str = "__analysis_redesign_fixture__ Running Run";
const FAILED_RUN_LABEL: &str = "__analysis_redesign_fixture__ Failed Run";
const CANCELLED_RUN_LABEL: &str = "__analysis_redesign_fixture__ Cancelled Run";
const GROUP_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Group Snapshot Run";
const LLM_PROFILE_LABEL: &str = "__analysis_redesign_fixture__ LLM Profile";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisRedesignFixtureSummary {
    pub accounts: i64,
    pub llm_profiles: i64,
    pub sources: i64,
    pub source_groups: i64,
    pub prompt_templates: i64,
    pub runs: i64,
    pub snapshot_messages: i64,
    pub chat_messages: i64,
    pub youtube_transcript_segments: i64,
    pub youtube_playlist_items: i64,
}

#[tauri::command]
pub async fn seed_analysis_redesign_fixtures(
    handle: AppHandle,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let pool = get_pool(&handle).await?;
    seed_analysis_redesign_fixtures_in_pool(&pool).await
}

#[tauri::command]
pub async fn clear_analysis_redesign_fixtures(
    handle: AppHandle,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let pool = get_pool(&handle).await?;
    clear_analysis_redesign_fixtures_in_pool(&pool).await
}

async fn seed_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let _ = clear_analysis_redesign_fixtures_in_pool(pool).await?;
    Ok(AnalysisRedesignFixtureSummary::default())
}

async fn clear_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let marker_pattern = format!("{FIXTURE_MARKER}%");
    let external_pattern = format!("{FIXTURE_EXTERNAL_PREFIX}%");
    let profile_settings_pattern = format!("llm.profile.{FIXTURE_PROFILE_ID}.%");

    let mut summary = AnalysisRedesignFixtureSummary::default();

    summary.chat_messages = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_chat_messages
             WHERE run_id IN (
                SELECT id FROM analysis_runs
                WHERE scope_label_snapshot LIKE ? OR provider_profile = ?
             )",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.snapshot_messages = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_run_messages
             WHERE run_id IN (
                SELECT id FROM analysis_runs
                WHERE scope_label_snapshot LIKE ? OR provider_profile = ?
             )",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.runs = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_runs
             WHERE scope_label_snapshot LIKE ? OR provider_profile = ?",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    let fixture_profile_setting_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM app_settings WHERE key LIKE ?",
    )
    .bind(&profile_settings_pattern)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::database)?;
    sqlx::query("DELETE FROM app_settings WHERE key LIKE ?")
        .bind(&profile_settings_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    summary.llm_profiles = if fixture_profile_setting_count > 0 { 1 } else { 0 };

    summary.prompt_templates = rows_to_i64(
        sqlx::query("DELETE FROM analysis_prompt_templates WHERE name LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    sqlx::query(
        "DELETE FROM analysis_source_group_members
         WHERE group_id IN (SELECT id FROM analysis_source_groups WHERE name LIKE ?)
            OR source_id IN (
                SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
            )",
    )
    .bind(&marker_pattern)
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    summary.source_groups = rows_to_i64(
        sqlx::query("DELETE FROM analysis_source_groups WHERE name LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    summary.youtube_playlist_items = rows_to_i64(
        sqlx::query(
            "DELETE FROM youtube_playlist_items
             WHERE playlist_source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR video_source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR video_id LIKE ?",
        )
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&external_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.youtube_transcript_segments = rows_to_i64(
        sqlx::query(
            "DELETE FROM youtube_transcript_segments
             WHERE source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR item_id IN (
                    SELECT items.id
                    FROM items
                    JOIN sources ON sources.id = items.source_id
                    WHERE sources.title LIKE ? OR sources.external_id LIKE ?
                )",
        )
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    sqlx::query(
        "DELETE FROM telegram_forum_topics
         WHERE source_id IN (
            SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
         )",
    )
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        "DELETE FROM items
         WHERE source_id IN (
            SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
         )",
    )
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    summary.sources = rows_to_i64(
        sqlx::query("DELETE FROM sources WHERE title LIKE ? OR external_id LIKE ?")
            .bind(&marker_pattern)
            .bind(&external_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    summary.accounts = rows_to_i64(
        sqlx::query("DELETE FROM accounts WHERE label LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    tx.commit().await.map_err(AppError::database)?;
    Ok(summary)
}

fn rows_to_i64(rows: u64) -> i64 {
    rows as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fixture_pool() -> Pool<Sqlite> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .expect("enable foreign keys");
        for migration in crate::migrations::build_migrations() {
            sqlx::raw_sql(migration.sql)
                .execute(&pool)
                .await
                .unwrap_or_else(|error| panic!("apply migration {}: {error}", migration.version));
        }
        pool
    }

    async fn count(pool: &Pool<Sqlite>, sql: &str) -> i64 {
        sqlx::query_scalar::<_, i64>(sql)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|error| panic!("count query failed: {sql}: {error}"))
    }

    #[tokio::test]
    async fn summary_serializes_with_camel_case_keys() {
        let summary = AnalysisRedesignFixtureSummary {
            accounts: 1,
            llm_profiles: 1,
            sources: 4,
            source_groups: 1,
            prompt_templates: 1,
            runs: 6,
            snapshot_messages: 4,
            chat_messages: 2,
            youtube_transcript_segments: 3,
            youtube_playlist_items: 2,
        };

        let value = serde_json::to_value(summary).expect("serialize summary");

        assert_eq!(value["llmProfiles"], 1);
        assert_eq!(value["sourceGroups"], 1);
        assert_eq!(value["promptTemplates"], 1);
        assert_eq!(value["snapshotMessages"], 4);
        assert_eq!(value["youtubeTranscriptSegments"], 3);
        assert_eq!(value["youtubePlaylistItems"], 2);
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
                id, source_type, source_subtype, telegram_source_kind, account_id, external_id,
                title, last_synced_at, is_active, is_member, created_at
             )
             VALUES (
                20, 'youtube', 'video', '', NULL,
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
                source_type, source_subtype, telegram_source_kind, account_id, external_id,
                title, is_active, is_member, created_at
             )
             VALUES ('telegram', 'channel', 'channel', 1, 'real-source', 'Real Source', 1, 1, 1)",
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
}
