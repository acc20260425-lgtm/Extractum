use serde::Serialize;
use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager, State};

use super::store::set_run_status;
use super::AnalysisState;
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::time::now_secs;

mod seed;

use self::seed::seed_analysis_redesign_fixtures_in_pool;

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
const YOUTUBE_FIXTURE_VIDEO_ID: &str = "analysis_fixture_video";
const YOUTUBE_FIXTURE_PLAYLIST_ID: &str = "PLanalysisfixture";
const TELEGRAM_FIXTURE_CHANNEL_PEER_ID: i64 = 10_000_001;
const TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID: i64 = 10_000_002;
const TELEGRAM_GROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Source Group";
const COMPLETED_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Completed Snapshot Run";
const MISSING_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Missing Snapshot Run";
const CAPTURE_FAILED_SNAPSHOT_RUN_LABEL: &str =
    "__analysis_redesign_fixture__ Capture Failed Snapshot Run";
const CAPTURE_FAILED_SNAPSHOT_ERROR: &str =
    "Snapshot capture failed: fixture write boundary unavailable";
const CANCELLED_RUN_MESSAGE: &str = "Analysis run cancelled.";
const RUNNING_RUN_LABEL: &str = "__analysis_redesign_fixture__ Running Run";
const FAILED_RUN_LABEL: &str = "__analysis_redesign_fixture__ Failed Run";
const CANCELLED_RUN_LABEL: &str = "__analysis_redesign_fixture__ Cancelled Run";
const GROUP_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Group Snapshot Run";
const LLM_PROFILE_LABEL: &str = "__analysis_redesign_fixture__ LLM Profile";
const FIXTURE_SNAPSHOT_CAPTURED_AT: &str = "2026-05-18T10:00:00Z";

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
    state: State<'_, AnalysisState>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let pool = get_pool(&handle).await?;
    let previous_run_ids = fixture_run_ids(&pool).await?;
    remove_fixture_active_runs(state.inner(), &previous_run_ids).await;
    let summary = seed_analysis_redesign_fixtures_in_pool(&pool).await?;
    let active_run_ids = register_fixture_active_runs(&pool, state.inner()).await?;
    spawn_fixture_cancellation_waiters(handle, active_run_ids).await;
    Ok(summary)
}

#[tauri::command]
pub async fn clear_analysis_redesign_fixtures(
    handle: AppHandle,
    state: State<'_, AnalysisState>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let pool = get_pool(&handle).await?;
    let run_ids = fixture_run_ids(&pool).await?;
    remove_fixture_active_runs(state.inner(), &run_ids).await;
    clear_analysis_redesign_fixtures_in_pool(&pool).await
}

#[tauri::command]
pub async fn clear_analysis_redesign_fixture_active_runs(
    handle: AppHandle,
    state: State<'_, AnalysisState>,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let run_ids = fixture_run_ids(&pool).await?;
    remove_fixture_active_runs(state.inner(), &run_ids).await;
    Ok(())
}

async fn fixture_run_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    let marker_pattern = format!("{FIXTURE_MARKER}%");
    sqlx::query_scalar(
        "SELECT id FROM analysis_runs
         WHERE scope_label_snapshot LIKE ? OR provider_profile = ?",
    )
    .bind(marker_pattern)
    .bind(FIXTURE_PROFILE_ID)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn register_fixture_active_runs(
    pool: &Pool<Sqlite>,
    state: &AnalysisState,
) -> AppResult<Vec<i64>> {
    let run_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM analysis_runs
         WHERE scope_label_snapshot = ? AND status = 'running'",
    )
    .bind(RUNNING_RUN_LABEL)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    for run_id in &run_ids {
        state.insert_active_report_run(*run_id).await;
    }

    Ok(run_ids)
}

async fn remove_fixture_active_runs(state: &AnalysisState, run_ids: &[i64]) {
    for run_id in run_ids {
        state.request_report_run_cancel(*run_id).await;
        state.remove_active_report_run(*run_id).await;
    }
}

async fn finish_cancelled_fixture_run(
    pool: &Pool<Sqlite>,
    state: &AnalysisState,
    run_id: i64,
) -> AppResult<()> {
    set_run_status(
        pool,
        run_id,
        crate::analysis::ANALYSIS_STATUS_CANCELLED,
        None,
        None,
        Some(CANCELLED_RUN_MESSAGE),
        Some(now_secs()),
    )
    .await?;
    state.remove_active_report_run(run_id).await;
    Ok(())
}

async fn spawn_fixture_cancellation_waiters(handle: AppHandle, run_ids: Vec<i64>) {
    for run_id in run_ids {
        let state = handle.state::<AnalysisState>();
        let Some(token) = state.report_run_child_token(run_id).await else {
            continue;
        };
        let task_handle = handle.clone();
        tauri::async_runtime::spawn(async move {
            token.cancelled().await;
            let Ok(pool) = get_pool(&task_handle).await else {
                return;
            };
            let state = task_handle.state::<AnalysisState>();
            let _ = finish_cancelled_fixture_run(&pool, state.inner(), run_id).await;
        });
    }
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

    let fixture_profile_setting_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM app_settings WHERE key LIKE ?")
            .bind(&profile_settings_pattern)
            .fetch_one(&mut *tx)
            .await
            .map_err(AppError::database)?;
    sqlx::query("DELETE FROM app_settings WHERE key LIKE ?")
        .bind(&profile_settings_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    summary.llm_profiles = if fixture_profile_setting_count > 0 {
        1
    } else {
        0
    };

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
    use sqlx::sqlite::SqlitePoolOptions;

    async fn fixture_pool() -> Pool<Sqlite> {
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
            runs: 7,
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
        let playlist_detail = crate::youtube::detail::get_youtube_playlist_detail_from_pool(
            &pool,
            playlist_source_id,
        )
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
        assert!(String::from_utf8(
            crate::compression::decompress_bytes(&raw).expect("decompress raw")
        )
        .expect("raw utf8")
        .contains("analysis_redesign_fixture"));
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
    async fn seeded_snapshot_runs_expose_captured_snapshot_state() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        for label in [COMPLETED_SNAPSHOT_RUN_LABEL, GROUP_SNAPSHOT_RUN_LABEL] {
            let run_id: i64 =
                sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                    .bind(label)
                    .fetch_one(&pool)
                    .await
                    .expect("load fixture run id");
            let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
                .await
                .expect("fetch fixture run")
                .map(crate::analysis::store::map_run_detail)
                .expect("fixture run exists");

            assert_eq!(
                detail.snapshot_state,
                Some(crate::analysis::models::AnalysisSnapshotState::Captured),
                "{label} should expose captured snapshot state"
            );
            assert!(
                detail.snapshot_captured_at.is_some(),
                "{label} should expose snapshot capture marker"
            );
            assert_eq!(detail.snapshot_error, None);
        }
    }

    #[tokio::test]
    async fn fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let youtube_trace: Vec<u8> = sqlx::query_scalar(
            "SELECT trace_data_zstd FROM analysis_runs WHERE scope_label_snapshot = ?",
        )
        .bind(COMPLETED_SNAPSHOT_RUN_LABEL)
        .fetch_one(&pool)
        .await
        .expect("load youtube trace");
        let telegram_trace: Vec<u8> = sqlx::query_scalar(
            "SELECT trace_data_zstd FROM analysis_runs WHERE scope_label_snapshot = ?",
        )
        .bind(GROUP_SNAPSHOT_RUN_LABEL)
        .fetch_one(&pool)
        .await
        .expect("load telegram trace");

        let youtube_json: serde_json::Value = serde_json::from_slice(
            &crate::compression::decompress_bytes(&youtube_trace)
                .expect("decompress youtube trace"),
        )
        .expect("parse youtube trace");
        let telegram_json: serde_json::Value = serde_json::from_slice(
            &crate::compression::decompress_bytes(&telegram_trace)
                .expect("decompress telegram trace"),
        )
        .expect("parse telegram trace");

        assert!(youtube_json["refs"]
            .as_array()
            .expect("youtube refs")
            .iter()
            .any(|value| value["ref"]
                .as_str()
                .unwrap_or_default()
                .contains("@754000ms")));
        assert!(telegram_json["refs"]
            .as_array()
            .expect("telegram refs")
            .iter()
            .any(|value| value["source_type"] == "telegram"));
    }

    #[tokio::test]
    async fn missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(MISSING_SNAPSHOT_RUN_LABEL)
                .fetch_one(&pool)
                .await
                .expect("load missing snapshot run");

        let summaries = crate::analysis::store::list_analysis_run_summaries(
            &pool,
            crate::analysis::store::AnalysisRunListFilters {
                query: Some(MISSING_SNAPSHOT_RUN_LABEL.to_string()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .expect("list fixture runs");
        let summary = summaries
            .iter()
            .find(|run| run.scope_label == MISSING_SNAPSHOT_RUN_LABEL)
            .expect("missing snapshot summary");
        assert_eq!(
            summary.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
        );

        let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
            .await
            .expect("fetch missing snapshot run")
            .map(crate::analysis::store::map_run_detail)
            .expect("missing snapshot run exists");
        assert_eq!(
            detail.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
        );
        assert_eq!(detail.snapshot_error, None);

        assert_eq!(
            count(
                &pool,
                &format!("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = {run_id}")
            )
            .await,
            0
        );
        assert_eq!(
            count(
                &pool,
                &format!(
                    "SELECT COUNT(*) FROM analysis_runs WHERE id = {run_id} AND trace_data_zstd IS NOT NULL"
                )
            )
            .await,
            1
        );
    }

    #[tokio::test]
    async fn capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(CAPTURE_FAILED_SNAPSHOT_RUN_LABEL)
                .fetch_one(&pool)
                .await
                .expect("load capture failed snapshot run");

        let summaries = crate::analysis::store::list_analysis_run_summaries(
            &pool,
            crate::analysis::store::AnalysisRunListFilters {
                query: Some(CAPTURE_FAILED_SNAPSHOT_RUN_LABEL.to_string()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .expect("list fixture runs");
        let summary = summaries
            .iter()
            .find(|run| run.scope_label == CAPTURE_FAILED_SNAPSHOT_RUN_LABEL)
            .expect("capture failed summary");
        assert_eq!(
            summary.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
        );
        assert_eq!(
            summary.snapshot_error.as_deref(),
            Some(CAPTURE_FAILED_SNAPSHOT_ERROR)
        );

        let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
            .await
            .expect("fetch capture failed snapshot run")
            .map(crate::analysis::store::map_run_detail)
            .expect("capture failed snapshot run exists");
        assert_eq!(detail.status, "failed");
        assert!(detail
            .result_markdown
            .as_deref()
            .unwrap_or_default()
            .contains("This capture-failed fixture report remains readable."));
        assert_eq!(
            detail.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
        );
        assert_eq!(
            detail.snapshot_error.as_deref(),
            Some(CAPTURE_FAILED_SNAPSHOT_ERROR)
        );
        assert_eq!(detail.snapshot_captured_at, None);

        assert_eq!(
            count(
                &pool,
                &format!("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = {run_id}")
            )
            .await,
            0
        );
        assert_eq!(
            count(
                &pool,
                &format!(
                    "SELECT COUNT(*) FROM analysis_runs WHERE id = {run_id} AND trace_data_zstd IS NOT NULL"
                )
            )
            .await,
            1
        );
    }

    #[tokio::test]
    async fn fixture_active_state_tracks_seeded_running_run() {
        let pool = fixture_pool().await;
        let state = super::super::AnalysisState::new();
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        register_fixture_active_runs(&pool, &state)
            .await
            .expect("register active fixture runs");

        let active_run_ids = state.active_report_run_ids().await;
        let running_run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(RUNNING_RUN_LABEL)
                .fetch_one(&pool)
                .await
                .expect("load running run");

        assert_eq!(active_run_ids.len(), 1);
        assert!(active_run_ids.contains(&running_run_id));
        let child_token = state
            .report_run_child_token(running_run_id)
            .await
            .expect("child token");

        let fixture_run_ids = fixture_run_ids(&pool).await.expect("load fixture run ids");
        remove_fixture_active_runs(&state, &fixture_run_ids).await;

        assert!(state.active_report_run_ids().await.is_empty());
        tokio::time::timeout(std::time::Duration::from_secs(1), child_token.cancelled())
            .await
            .expect("fixture child token cancelled");
    }

    #[tokio::test]
    async fn fixture_cancel_waiter_marks_running_run_cancelled() {
        let pool = fixture_pool().await;
        let state = super::super::AnalysisState::new();
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");
        register_fixture_active_runs(&pool, &state)
            .await
            .expect("register active fixture runs");
        let running_run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(RUNNING_RUN_LABEL)
                .fetch_one(&pool)
                .await
                .expect("load running run");
        state.request_report_run_cancel(running_run_id).await;

        finish_cancelled_fixture_run(&pool, &state, running_run_id)
            .await
            .expect("finish cancelled fixture");

        let status: String = sqlx::query_scalar("SELECT status FROM analysis_runs WHERE id = ?")
            .bind(running_run_id)
            .fetch_one(&pool)
            .await
            .expect("load status");
        assert_eq!(status, crate::analysis::ANALYSIS_STATUS_CANCELLED);
        assert!(!state
            .active_report_run_ids()
            .await
            .contains(&running_run_id));
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
}
