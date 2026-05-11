use serde::Serialize;
use sqlx::{Pool, Sqlite};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::AppResult;

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
    _pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    Ok(AnalysisRedesignFixtureSummary::default())
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
}
