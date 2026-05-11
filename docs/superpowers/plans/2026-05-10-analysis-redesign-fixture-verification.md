# Analysis Redesign Fixture Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add debug-only seed and clear commands that create representative `/analysis` data for fixture-backed browser verification.

**Architecture:** All fixture behavior lives in a debug-only Rust backend module under `analysis`, uses the existing `sqlite:extractum.db` pool, and exposes only two Tauri commands in debug builds. The fixture dataset is deterministic in labels and counts, uses existing zstd helpers for compressed fields, clears by fixture markers plus parent ids, and leaves production behavior untouched.

**Tech Stack:** Tauri 2, Rust 2021, SQLx SQLite, zstd compression helpers, existing Extractum analysis/source/YouTube schemas, manual browser verification through the Tauri global.

---

## Scope Boundary

This plan implements verification infrastructure only. It does not add a visible product UI, does not add Playwright, does not create real Telegram or LLM secrets, and does not update completed Part 7 browser scenario results before the fixture-backed scenarios have actually been exercised.

The approved design spec is:

`docs/superpowers/specs/2026-05-10-analysis-redesign-fixture-verification-design.md`

The browser result document to update after manual verification is:

`docs/superpowers/verification/2026-05-10-analysis-redesign.md`

## File Structure

- Create: `src-tauri/src/analysis/fixtures.rs`
  - Owns fixture constants, summary DTO, seed and clear commands, SQL helpers, deterministic dataset builders, and backend tests.
- Modify: `src-tauri/src/analysis/mod.rs`
  - Adds `#[cfg(debug_assertions)] mod fixtures;` and debug-only re-exports for the two commands.
- Modify: `src-tauri/src/lib.rs`
  - Imports the two debug-only commands and registers them in `tauri::generate_handler!` only under `#[cfg(debug_assertions)]`.
- Update: `docs/superpowers/verification/2026-05-10-analysis-redesign.md`
  - After browser execution, records fixture-backed `PASS`, `FAIL`, or remaining `BLOCKED` rows.
- Optional during execution: update `reference/session-context-2026-05-10-analysis-redesign.md`
  - Only after implementation and verification, refreshes session restoration context.

## Dataset Contract

Use these exact constants in `fixtures.rs`:

```rust
const FIXTURE_MARKER: &str = "__analysis_redesign_fixture__";
const FIXTURE_EXTERNAL_PREFIX: &str = "__analysis_redesign_fixture__:";
const FIXTURE_PROFILE_ID: &str = "__analysis_redesign_fixture__";
const FIXTURE_NOW: i64 = 1_778_400_000;
const FIXTURE_PERIOD_FROM: i64 = 1_777_968_000;
const FIXTURE_PERIOD_TO: i64 = 1_778_313_600;
```

Seed these visible labels:

```rust
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
```

The fixture command summary type must be serializable and use this exact shape:

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
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
```

## Task 1: Add Fixture Module Skeleton And Schema Test Harness

**Files:**
- Create: `src-tauri/src/analysis/fixtures.rs`

- [ ] **Step 1: Create the failing summary and schema tests**

Create `src-tauri/src/analysis/fixtures.rs` with this initial test-first content:

```rust
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
                &format!("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '{table}'"),
            )
            .await;
            assert_eq!(exists, 1, "missing table {table}");
        }
    }

}
```

- [ ] **Step 2: Run the skeleton tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::summary_serializes_with_camel_case_keys analysis::fixtures::tests::fixture_test_pool_has_required_tables
```

Expected: PASS.

- [ ] **Step 3: Commit the skeleton tests**

Run after the skeleton tests pass:

```powershell
git add src-tauri/src/analysis/fixtures.rs
git commit -m "test: add analysis fixture contract skeleton"
```

## Task 2: Implement Safe Marker Discovery And Clear

**Files:**
- Modify: `src-tauri/src/analysis/fixtures.rs`

- [ ] **Step 1: Add clear safety tests**

Append these tests inside the existing `#[cfg(test)] mod tests`:

```rust
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
         VALUES (60, 30, 20, '__analysis_redesign_fixture__:clear-item', 'Fixture', 10, 's20-i30', x'28B52FFD0000010000')",
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
        count(&pool, "SELECT COUNT(*) FROM accounts WHERE label = 'Personal'").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM sources WHERE title = 'Real Source'").await,
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
        count(&pool, "SELECT COUNT(*) FROM analysis_runs WHERE scope_label_snapshot = 'User Run'").await,
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
```

- [ ] **Step 2: Run the clear safety tests and observe failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::clear_
```

Expected: FAIL because clear still returns a default summary and does not delete seeded rows.

- [ ] **Step 3: Replace the clear helper with parent-id-based deletion**

Replace `clear_analysis_redesign_fixtures_in_pool` with this implementation and add the helper functions above the test module:

```rust
async fn fixture_run_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        "SELECT id FROM analysis_runs WHERE scope_label_snapshot LIKE ? ORDER BY id",
    )
    .bind(format!("{FIXTURE_MARKER}%"))
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn fixture_source_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        "SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ? ORDER BY id",
    )
    .bind(format!("{FIXTURE_MARKER}%"))
    .bind(format!("{FIXTURE_EXTERNAL_PREFIX}%"))
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn fixture_source_group_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        "SELECT id FROM analysis_source_groups WHERE name LIKE ? ORDER BY id",
    )
    .bind(format!("{FIXTURE_MARKER}%"))
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn fixture_prompt_template_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        "SELECT id FROM analysis_prompt_templates WHERE name LIKE ? ORDER BY id",
    )
    .bind(format!("{FIXTURE_MARKER}%"))
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn fixture_account_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    sqlx::query_scalar("SELECT id FROM accounts WHERE label LIKE ? ORDER BY id")
        .bind(format!("{FIXTURE_MARKER}%"))
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
}

fn fixture_profile_settings() -> Vec<String> {
    vec![
        format!("llm.profile.{FIXTURE_PROFILE_ID}.provider"),
        format!("llm.profile.{FIXTURE_PROFILE_ID}.default_model"),
        format!("llm.profile.{FIXTURE_PROFILE_ID}.base_url"),
    ]
}

async fn count_by_ids<'e, E>(
    executor: E,
    table: &str,
    column: &str,
    ids: &[i64],
) -> AppResult<i64>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    if ids.is_empty() {
        return Ok(0);
    }
    let placeholders = std::iter::repeat("?")
        .take(ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE {column} IN ({placeholders})");
    let mut query = sqlx::query_scalar::<_, i64>(&sql);
    for id in ids {
        query = query.bind(*id);
    }
    query.fetch_one(executor).await.map_err(AppError::database)
}

async fn delete_by_ids<'e, E>(
    executor: E,
    table: &str,
    column: &str,
    ids: &[i64],
) -> AppResult<i64>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    if ids.is_empty() {
        return Ok(0);
    }
    let placeholders = std::iter::repeat("?")
        .take(ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("DELETE FROM {table} WHERE {column} IN ({placeholders})");
    let mut query = sqlx::query(&sql);
    for id in ids {
        query = query.bind(*id);
    }
    query
        .execute(executor)
        .await
        .map(|result| result.rows_affected() as i64)
        .map_err(AppError::database)
}

async fn clear_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let run_ids = fixture_run_ids(pool).await?;
    let source_ids = fixture_source_ids(pool).await?;
    let source_group_ids = fixture_source_group_ids(pool).await?;
    let prompt_template_ids = fixture_prompt_template_ids(pool).await?;
    let account_ids = fixture_account_ids(pool).await?;
    let profile_keys = fixture_profile_settings();
    let profile_setting_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT key) FROM app_settings WHERE key IN (?, ?, ?)",
    )
    .bind(&profile_keys[0])
    .bind(&profile_keys[1])
    .bind(&profile_keys[2])
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    let profile_count = profile_setting_count.min(1);

    let summary = AnalysisRedesignFixtureSummary {
        accounts: account_ids.len() as i64,
        llm_profiles: profile_count,
        sources: source_ids.len() as i64,
        source_groups: source_group_ids.len() as i64,
        prompt_templates: prompt_template_ids.len() as i64,
        runs: run_ids.len() as i64,
        snapshot_messages: count_by_ids(pool, "analysis_run_messages", "run_id", &run_ids).await?,
        chat_messages: count_by_ids(pool, "analysis_chat_messages", "run_id", &run_ids).await?,
        youtube_transcript_segments: count_by_ids(
            pool,
            "youtube_transcript_segments",
            "source_id",
            &source_ids,
        )
        .await?,
        youtube_playlist_items: count_by_ids(
            pool,
            "youtube_playlist_items",
            "playlist_source_id",
            &source_ids,
        )
        .await?,
    };

    let mut tx = pool.begin().await.map_err(AppError::database)?;

    delete_by_ids(&mut *tx, "analysis_chat_messages", "run_id", &run_ids).await?;
    delete_by_ids(&mut *tx, "analysis_run_messages", "run_id", &run_ids).await?;
    delete_by_ids(&mut *tx, "analysis_runs", "id", &run_ids).await?;
    sqlx::query("DELETE FROM app_settings WHERE key IN (?, ?, ?)")
        .bind(&profile_keys[0])
        .bind(&profile_keys[1])
        .bind(&profile_keys[2])
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    delete_by_ids(&mut *tx, "analysis_prompt_templates", "id", &prompt_template_ids).await?;
    delete_by_ids(&mut *tx, "analysis_source_group_members", "group_id", &source_group_ids).await?;
    delete_by_ids(&mut *tx, "analysis_source_groups", "id", &source_group_ids).await?;
    delete_by_ids(&mut *tx, "youtube_playlist_items", "playlist_source_id", &source_ids).await?;
    delete_by_ids(&mut *tx, "youtube_transcript_segments", "source_id", &source_ids).await?;
    delete_by_ids(&mut *tx, "telegram_forum_topics", "source_id", &source_ids).await?;
    delete_by_ids(&mut *tx, "items", "source_id", &source_ids).await?;
    delete_by_ids(&mut *tx, "sources", "id", &source_ids).await?;
    delete_by_ids(&mut *tx, "accounts", "id", &account_ids).await?;

    tx.commit().await.map_err(AppError::database)?;
    Ok(summary)
}
```

- [ ] **Step 4: Run the clear safety tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::clear_
```

Expected: PASS after the helper signatures compile.

- [ ] **Step 5: Commit safe clear**

Run:

```powershell
git add src-tauri/src/analysis/fixtures.rs
git commit -m "feat: add safe analysis fixture cleanup"
```

## Task 3: Seed Accounts, Prompt Template, LLM Profile, Sources, And Group

**Files:**
- Modify: `src-tauri/src/analysis/fixtures.rs`

- [ ] **Step 1: Add structural seed tests**

Append these tests:

```rust
#[tokio::test]
async fn seed_creates_safe_account_prompt_profile_sources_and_group() {
    let pool = fixture_pool().await;

    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let account: (String, i64, String, Option<String>) = sqlx::query_as(
        "SELECT label, api_id, api_hash, phone FROM accounts WHERE label = ?",
    )
    .bind(TELEGRAM_CHANNEL_LABEL.replace("Telegram Channel", "Telegram Account"))
    .fetch_one(&pool)
    .await
    .expect("load fixture account");
    assert!(account.0.starts_with(FIXTURE_MARKER));
    assert_eq!(account.1, 100_001);
    assert_eq!(account.2, "");
    assert_eq!(account.3, None);

    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%'").await,
        4
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM sources WHERE source_type = 'telegram' AND account_id IS NOT NULL").await,
        2
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_source_groups WHERE name = '__analysis_redesign_fixture__ Telegram Group'").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_source_group_members").await,
        2
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_prompt_templates WHERE name LIKE '__analysis_redesign_fixture__%' AND template_kind = 'report'").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM app_settings WHERE key LIKE 'llm.profile.__analysis_redesign_fixture__.%'").await,
        3
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM app_settings WHERE key LIKE 'llm.profile.__analysis_redesign_fixture__.api_key'").await,
        0
    );
}

```

- [ ] **Step 2: Run the structural tests and observe failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_safe_account_prompt_profile_sources_and_group
```

Expected: FAIL because the seed still inserts no rows.

- [ ] **Step 3: Add compressed JSON helpers and structural seed functions**

Add these helpers above `seed_analysis_redesign_fixtures_in_pool`:

```rust
fn json_zstd(value: serde_json::Value) -> AppResult<Vec<u8>> {
    let json = serde_json::to_vec(&value).map_err(|error| AppError::internal(error.to_string()))?;
    crate::compression::compress_json_bytes(&json).map_err(AppError::internal)
}

async fn insert_fixture_account(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<i64> {
    sqlx::query_scalar(
        "INSERT INTO accounts (label, api_id, api_hash, phone, created_at)
         VALUES (?, 100001, '', NULL, ?)
         RETURNING id",
    )
    .bind(format!("{FIXTURE_MARKER} Telegram Account"))
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_fixture_prompt_template(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<i64> {
    sqlx::query_scalar(
        "INSERT INTO analysis_prompt_templates (
            name, template_kind, body, version, is_builtin, created_at, updated_at
         )
         VALUES (?, 'report', ?, 1, 0, ?, ?)
         RETURNING id",
    )
    .bind(format!("{FIXTURE_MARKER} Report Template"))
    .bind(
        "Write a concise fixture report. Cite saved evidence refs and keep fixture labels visible.",
    )
    .bind(FIXTURE_NOW)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_fixture_llm_profile(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<()> {
    for (key, value) in [
        (
            format!("llm.profile.{FIXTURE_PROFILE_ID}.provider"),
            "gemini".to_string(),
        ),
        (
            format!("llm.profile.{FIXTURE_PROFILE_ID}.default_model"),
            "gemini-2.5-flash".to_string(),
        ),
        (
            format!("llm.profile.{FIXTURE_PROFILE_ID}.base_url"),
            "".to_string(),
        ),
    ] {
        sqlx::query(
            "INSERT INTO app_settings (key, value)
             VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }
    Ok(())
}

async fn insert_telegram_source(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    account_id: i64,
    label: &str,
    kind: &str,
    external_suffix: &str,
    last_sync_state: i64,
) -> AppResult<i64> {
    sqlx::query_scalar(
        "INSERT INTO sources (
            source_type, source_subtype, telegram_source_kind, account_id, external_id, title,
            metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
         )
         VALUES ('telegram', ?, ?, ?, ?, ?, ?, ?, ?, 1, 1, ?)
         RETURNING id",
    )
    .bind(kind)
    .bind(kind)
    .bind(account_id)
    .bind(format!("{FIXTURE_EXTERNAL_PREFIX}{external_suffix}"))
    .bind(label)
    .bind(json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "peer_identity": {
            "strategy": "dialog",
            "username": external_suffix,
            "access_hash": 424242
        }
    }))?)
    .bind(last_sync_state)
    .bind(FIXTURE_NOW - 600)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_youtube_video_source(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<i64> {
    let video_id = format!("{FIXTURE_EXTERNAL_PREFIX}youtube-video");
    sqlx::query_scalar(
        "INSERT INTO sources (
            source_type, source_subtype, telegram_source_kind, account_id, external_id, title,
            metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
         )
         VALUES ('youtube', 'video', '', NULL, ?, ?, ?, NULL, ?, 1, 0, ?)
         RETURNING id",
    )
    .bind(&video_id)
    .bind(YOUTUBE_VIDEO_LABEL)
    .bind(json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "video_id": video_id,
        "canonical_url": "https://www.youtube.com/watch?v=analysis_fixture_video",
        "title": YOUTUBE_VIDEO_LABEL,
        "channel_title": "Fixture Channel",
        "channel_id": "UCfixture",
        "channel_handle": "@analysisfixture",
        "channel_url": "https://www.youtube.com/@analysisfixture",
        "author_display": "Fixture Channel",
        "published_at": "2026-05-01",
        "duration_seconds": 920,
        "description": "Fixture video description for analysis report verification.",
        "thumbnail_url": null,
        "tags": ["fixture", "analysis"],
        "chapters": [],
        "view_count": 1250,
        "like_count": 86,
        "comment_count": 2,
        "category": "Education",
        "video_form": "regular",
        "availability_status": "available",
        "raw_metadata_json": { "fixture": true }
    }))?)
    .bind(FIXTURE_NOW - 540)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_youtube_playlist_source(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<i64> {
    let playlist_id = format!("{FIXTURE_EXTERNAL_PREFIX}youtube-playlist");
    sqlx::query_scalar(
        "INSERT INTO sources (
            source_type, source_subtype, telegram_source_kind, account_id, external_id, title,
            metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
         )
         VALUES ('youtube', 'playlist', '', NULL, ?, ?, ?, NULL, ?, 1, 0, ?)
         RETURNING id",
    )
    .bind(&playlist_id)
    .bind(YOUTUBE_PLAYLIST_LABEL)
    .bind(json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "playlist_id": playlist_id,
        "canonical_url": "https://www.youtube.com/playlist?list=analysis_fixture_playlist",
        "title": YOUTUBE_PLAYLIST_LABEL,
        "channel_title": "Fixture Channel",
        "channel_id": "UCfixture",
        "channel_handle": "@analysisfixture",
        "channel_url": "https://www.youtube.com/@analysisfixture",
        "thumbnail_url": null,
        "video_count": 2,
        "items": [],
        "availability_status": "available",
        "raw_metadata_json": { "fixture": true }
    }))?)
    .bind(FIXTURE_NOW - 500)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_fixture_source_group(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    telegram_channel_id: i64,
    telegram_supergroup_id: i64,
) -> AppResult<i64> {
    let group_id: i64 = sqlx::query_scalar(
        "INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)
         VALUES (?, 'telegram', ?, ?)
         RETURNING id",
    )
    .bind(TELEGRAM_GROUP_LABEL)
    .bind(FIXTURE_NOW)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    for source_id in [telegram_channel_id, telegram_supergroup_id] {
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (?, ?, ?)",
        )
        .bind(group_id)
        .bind(source_id)
        .bind(FIXTURE_NOW)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }
    Ok(group_id)
}
```

- [ ] **Step 4: Update seed to insert the structural dataset**

Replace `seed_analysis_redesign_fixtures_in_pool` with:

```rust
async fn seed_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let _ = clear_analysis_redesign_fixtures_in_pool(pool).await?;
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    let account_id = insert_fixture_account(&mut tx).await?;
    let prompt_template_id = insert_fixture_prompt_template(&mut tx).await?;
    insert_fixture_llm_profile(&mut tx).await?;
    let telegram_channel_id = insert_telegram_source(
        &mut tx,
        account_id,
        TELEGRAM_CHANNEL_LABEL,
        "channel",
        "telegram-channel",
        9001,
    )
    .await?;
    let telegram_supergroup_id = insert_telegram_source(
        &mut tx,
        account_id,
        TELEGRAM_SUPERGROUP_LABEL,
        "supergroup",
        "telegram-supergroup",
        9101,
    )
    .await?;
    let youtube_video_id = insert_youtube_video_source(&mut tx).await?;
    let youtube_playlist_id = insert_youtube_playlist_source(&mut tx).await?;
    let source_group_id =
        insert_fixture_source_group(&mut tx, telegram_channel_id, telegram_supergroup_id).await?;

    let _ = (
        prompt_template_id,
        source_group_id,
        youtube_video_id,
        youtube_playlist_id,
    );

    tx.commit().await.map_err(AppError::database)?;

    Ok(AnalysisRedesignFixtureSummary {
        accounts: 1,
        llm_profiles: 1,
        sources: 4,
        source_groups: 1,
        prompt_templates: 1,
        runs: 0,
        snapshot_messages: 0,
        chat_messages: 0,
        youtube_transcript_segments: 0,
        youtube_playlist_items: 0,
    })
}
```

- [ ] **Step 5: Run structural tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_safe_account_prompt_profile_sources_and_group
```

Expected: PASS.

- [ ] **Step 6: Commit structural seed**

Run:

```powershell
git add src-tauri/src/analysis/fixtures.rs
git commit -m "feat: seed analysis fixture structure"
```

## Task 4: Seed Source Items, YouTube Transcript, Playlist Rows, And Telegram Topics

**Files:**
- Modify: `src-tauri/src/analysis/fixtures.rs`

- [ ] **Step 1: Add source content tests**

Append these tests:

```rust
#[tokio::test]
async fn seed_creates_post_sync_reader_content() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM items WHERE item_kind = 'telegram_message'").await,
        4
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM telegram_forum_topics").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM items WHERE has_media = 1 AND media_metadata_zstd IS NOT NULL").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM items WHERE reply_to_top_id IS NOT NULL OR reply_to_msg_id IS NOT NULL").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM items WHERE reaction_count IS NOT NULL").await,
        2
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM items WHERE item_kind = 'youtube_transcript'").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM items WHERE item_kind = 'youtube_comment'").await,
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
        count(&pool, "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%' AND last_synced_at IS NOT NULL").await,
        4
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

    assert!(
        crate::compression::decompress_text(&content)
            .expect("decompress content")
            .contains("fixture channel update")
    );
    assert!(
        String::from_utf8(crate::compression::decompress_bytes(&media).expect("decompress media"))
            .expect("media utf8")
            .contains("image/jpeg")
    );
    assert!(
        String::from_utf8(crate::compression::decompress_bytes(&raw).expect("decompress raw"))
            .expect("raw utf8")
            .contains("analysis_redesign_fixture")
    );
}
```

- [ ] **Step 2: Run source content tests and observe failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_post_sync_reader_content analysis::fixtures::tests::compressed_fixture_fields_are_readable
```

Expected: FAIL because source items and provider child rows have not been seeded.

- [ ] **Step 3: Add source item helpers**

Add these helpers:

```rust
async fn insert_item(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    source_id: i64,
    external_suffix: &str,
    item_kind: &str,
    author: &str,
    published_at: i64,
    content: &str,
    content_kind: &str,
    media_kind: Option<&str>,
    media_metadata: Option<serde_json::Value>,
    reply_to_msg_id: Option<i64>,
    reply_to_top_id: Option<i64>,
    reaction_count: Option<i64>,
) -> AppResult<i64> {
    let raw = json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "external_suffix": external_suffix,
        "item_kind": item_kind
    }))?;
    let media_metadata_zstd = media_metadata.map(json_zstd).transpose()?;
    sqlx::query_scalar(
        "INSERT INTO items (
            source_id, external_id, item_kind, author, published_at, ingested_at, content_zstd,
            raw_data_zstd, content_kind, has_media, media_kind, media_metadata_zstd,
            reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
            reaction_count
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, ?, ?)
         RETURNING id",
    )
    .bind(source_id)
    .bind(format!("{FIXTURE_EXTERNAL_PREFIX}{external_suffix}"))
    .bind(item_kind)
    .bind(author)
    .bind(published_at)
    .bind(FIXTURE_NOW - 480)
    .bind(crate::compression::compress_text(content).map_err(AppError::internal)?)
    .bind(raw)
    .bind(content_kind)
    .bind(media_metadata_zstd.is_some())
    .bind(media_kind)
    .bind(media_metadata_zstd)
    .bind(reply_to_msg_id)
    .bind(reply_to_top_id)
    .bind(reaction_count)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_telegram_content(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    telegram_channel_id: i64,
    telegram_supergroup_id: i64,
) -> AppResult<()> {
    insert_item(
        tx,
        telegram_channel_id,
        "tg-channel-1",
        "telegram_message",
        "Fixture Editor",
        FIXTURE_PERIOD_FROM + 1_200,
        "fixture channel update: result-first analysis now has source evidence",
        "text_only",
        None,
        None,
        None,
        None,
        Some(4),
    )
    .await?;
    insert_item(
        tx,
        telegram_channel_id,
        "tg-channel-2",
        "telegram_message",
        "Fixture Analyst",
        FIXTURE_PERIOD_FROM + 2_400,
        "fixture channel update: browser verification should show this timeline row",
        "text_only",
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, icon_color, icon_emoji_id,
            is_closed, is_pinned, is_hidden, is_deleted, sort_order, last_seen_at, updated_at
         )
         VALUES (?, 501, 7001, ?, 7322096, NULL, 0, 1, 0, 0, 1, ?, ?)",
    )
    .bind(telegram_supergroup_id)
    .bind(format!("{FIXTURE_MARKER} Topic"))
    .bind(FIXTURE_NOW - 420)
    .bind(FIXTURE_NOW - 420)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;

    insert_item(
        tx,
        telegram_supergroup_id,
        "tg-supergroup-topic",
        "telegram_message",
        "Fixture Moderator",
        FIXTURE_PERIOD_FROM + 3_600,
        "fixture supergroup topic: grouped source reader should preserve topic metadata",
        "text_only",
        None,
        None,
        None,
        Some(7001),
        Some(7),
    )
    .await?;
    insert_item(
        tx,
        telegram_supergroup_id,
        "tg-supergroup-media",
        "telegram_message",
        "Fixture Member",
        FIXTURE_PERIOD_FROM + 4_800,
        "fixture supergroup media placeholder: no binary preview is available",
        "text_with_media",
        Some("photo"),
        Some(serde_json::json!({
            "analysis_redesign_fixture": true,
            "summary": "Fixture image placeholder",
            "file_name": "fixture-proof.jpg",
            "mime_type": "image/jpeg",
            "size_bytes": 204800,
            "width": 1280,
            "height": 720
        })),
        Some(7001),
        Some(7001),
        Some(2),
    )
    .await?;
    Ok(())
}

async fn insert_youtube_content(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    youtube_video_id: i64,
    youtube_playlist_id: i64,
) -> AppResult<()> {
    let transcript_item_id = insert_item(
        tx,
        youtube_video_id,
        "youtube-transcript",
        "youtube_transcript",
        "Fixture Channel",
        FIXTURE_PERIOD_FROM + 6_000,
        "Fixture transcript full text for result-first report evidence.",
        "text_only",
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    for (index, start_ms, text) in [
        (0_i64, 0_i64, "Fixture opening segment introduces the redesign."),
        (1_i64, 754_000_i64, "Fixture timestamp segment supports Show in source."),
        (2_i64, 790_000_i64, "Fixture closing segment mentions evidence tabs."),
    ] {
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text,
                chapter_index, caption_language, caption_track_kind, is_auto_generated,
                metadata_zstd
             )
             VALUES (?, ?, ?, ?, ?, ?, NULL, 'en', 'manual', 0, ?)",
        )
        .bind(transcript_item_id)
        .bind(youtube_video_id)
        .bind(index)
        .bind(start_ms)
        .bind(start_ms + 25_000)
        .bind(text)
        .bind(json_zstd(serde_json::json!({ "analysis_redesign_fixture": true }))?)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    insert_item(
        tx,
        youtube_video_id,
        "youtube-comment",
        "youtube_comment",
        "Fixture Commenter",
        FIXTURE_PERIOD_FROM + 7_200,
        "Fixture comment validates transcript_description_comments mode.",
        "text_only",
        None,
        None,
        None,
        None,
        Some(5),
    )
    .await?;

    sqlx::query(
        "INSERT INTO youtube_playlist_items (
            playlist_source_id, video_source_id, video_id, position, title_snapshot, url,
            thumbnail_url, availability_status, is_removed_from_playlist, last_seen_at,
            metadata_zstd, created_at, updated_at
         )
         VALUES
            (?, ?, ?, 1, ?, 'https://www.youtube.com/watch?v=analysis_fixture_video', NULL,
             'available', 0, ?, ?, ?, ?),
            (?, NULL, ?, 2, ?, 'https://www.youtube.com/watch?v=analysis_fixture_missing', NULL,
             'private_or_auth_required', 0, ?, ?, ?, ?)",
    )
    .bind(youtube_playlist_id)
    .bind(youtube_video_id)
    .bind(format!("{FIXTURE_EXTERNAL_PREFIX}youtube-video"))
    .bind(YOUTUBE_VIDEO_LABEL)
    .bind(FIXTURE_NOW - 360)
    .bind(json_zstd(serde_json::json!({ "analysis_redesign_fixture": true, "linked": true }))?)
    .bind(FIXTURE_NOW)
    .bind(FIXTURE_NOW)
    .bind(youtube_playlist_id)
    .bind(format!("{FIXTURE_EXTERNAL_PREFIX}youtube-missing"))
    .bind(format!("{FIXTURE_MARKER} Unavailable Playlist Item"))
    .bind(FIXTURE_NOW - 360)
    .bind(json_zstd(serde_json::json!({ "analysis_redesign_fixture": true, "linked": false }))?)
    .bind(FIXTURE_NOW)
    .bind(FIXTURE_NOW)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [ ] **Step 4: Call source content helpers from seed**

In `seed_analysis_redesign_fixtures_in_pool`, after `insert_fixture_source_group`, add:

```rust
insert_telegram_content(&mut tx, telegram_channel_id, telegram_supergroup_id).await?;
insert_youtube_content(&mut tx, youtube_video_id, youtube_playlist_id).await?;
```

Update the returned summary values:

```rust
youtube_transcript_segments: 3,
youtube_playlist_items: 2,
```

- [ ] **Step 5: Run source content tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_post_sync_reader_content analysis::fixtures::tests::compressed_fixture_fields_are_readable
```

Expected: PASS.

- [ ] **Step 6: Commit source content seed**

Run:

```powershell
git add src-tauri/src/analysis/fixtures.rs
git commit -m "feat: seed analysis fixture source content"
```

## Task 5: Seed Analysis Runs, Snapshots, Trace Data, And Chat History

**Files:**
- Modify: `src-tauri/src/analysis/fixtures.rs`

- [ ] **Step 1: Add run and trace tests**

Append these tests:

```rust
#[tokio::test]
async fn seed_creates_fixture_runs_with_statuses_templates_and_snapshots() {
    let pool = fixture_pool().await;
    let summary = seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    assert_eq!(summary.runs, 6);
    assert_eq!(summary.snapshot_messages, 4);
    assert_eq!(summary.chat_messages, 2);
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_runs WHERE prompt_template_id IS NOT NULL").await,
        6
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(DISTINCT status) FROM analysis_runs WHERE scope_label_snapshot LIKE '__analysis_redesign_fixture__%'").await,
        4
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_runs WHERE status = 'completed'").await,
        3
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_runs WHERE status = 'running'").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_runs WHERE status = 'failed'").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_runs WHERE status = 'cancelled'").await,
        1
    );
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
        &crate::compression::decompress_bytes(&youtube_trace).expect("decompress youtube trace"),
    )
    .expect("parse youtube trace");
    let telegram_json: serde_json::Value = serde_json::from_slice(
        &crate::compression::decompress_bytes(&telegram_trace).expect("decompress telegram trace"),
    )
    .expect("parse telegram trace");

    assert!(
        youtube_json["refs"]
            .as_array()
            .expect("youtube refs")
            .iter()
            .any(|value| value["ref"].as_str().unwrap_or_default().contains("@754000ms"))
    );
    assert!(
        telegram_json["refs"]
            .as_array()
            .expect("telegram refs")
            .iter()
            .any(|value| value["source_type"] == "telegram")
    );
}

#[tokio::test]
async fn missing_snapshot_run_has_trace_but_no_saved_messages() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let run_id: i64 = sqlx::query_scalar(
        "SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?",
    )
    .bind(MISSING_SNAPSHOT_RUN_LABEL)
    .fetch_one(&pool)
    .await
    .expect("load missing snapshot run");

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
            &format!("SELECT COUNT(*) FROM analysis_runs WHERE id = {run_id} AND trace_data_zstd IS NOT NULL")
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
        count(&pool, "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%'").await,
        4
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_runs WHERE scope_label_snapshot LIKE '__analysis_redesign_fixture__%'").await,
        6
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_run_messages").await,
        4
    );
}
```

- [ ] **Step 2: Run run and trace tests and observe failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_fixture_runs_with_statuses_templates_and_snapshots analysis::fixtures::tests::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot analysis::fixtures::tests::missing_snapshot_run_has_trace_but_no_saved_messages analysis::fixtures::tests::seed_twice_keeps_one_deterministic_fixture_set
```

Expected: FAIL because analysis runs have not been seeded.

- [ ] **Step 3: Add analysis run helpers**

Add these helpers:

```rust
struct FixtureIds {
    prompt_template_id: i64,
    telegram_channel_id: i64,
    telegram_supergroup_id: i64,
    youtube_video_id: i64,
    source_group_id: i64,
}

async fn insert_run(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    label: &str,
    scope_type: &str,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    prompt_template_id: i64,
    status: &str,
    result_markdown: Option<&str>,
    trace_data_zstd: Option<Vec<u8>>,
    error: Option<&str>,
    completed_at: Option<i64>,
) -> AppResult<i64> {
    sqlx::query_scalar(
        "INSERT INTO analysis_runs (
            run_type, scope_type, source_id, source_group_id, period_from, period_to,
            output_language, prompt_template_id, prompt_template_version, provider_profile,
            provider, model, youtube_corpus_mode, status, result_markdown, trace_data_zstd,
            scope_label_snapshot, error, created_at, completed_at
         )
         VALUES (
            'report', ?, ?, ?, ?, ?, 'English', ?, 1, ?, 'gemini',
            'gemini-2.5-flash', 'transcript_description_comments', ?, ?, ?, ?, ?, ?, ?
         )
         RETURNING id",
    )
    .bind(scope_type)
    .bind(source_id)
    .bind(source_group_id)
    .bind(FIXTURE_PERIOD_FROM)
    .bind(FIXTURE_PERIOD_TO)
    .bind(prompt_template_id)
    .bind(FIXTURE_PROFILE_ID)
    .bind(status)
    .bind(result_markdown)
    .bind(trace_data_zstd)
    .bind(label)
    .bind(error)
    .bind(FIXTURE_NOW)
    .bind(completed_at)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_snapshot_message(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    run_id: i64,
    item_id: i64,
    source_id: i64,
    external_id: &str,
    author: &str,
    published_at: i64,
    reference: &str,
    content: &str,
    item_kind: &str,
    source_type: &str,
    source_subtype: &str,
    metadata: Option<serde_json::Value>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO analysis_run_messages (
            run_id, item_id, source_id, external_id, author, published_at, ref, content_zstd,
            item_kind, source_type, source_subtype, metadata_zstd
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(item_id)
    .bind(source_id)
    .bind(external_id)
    .bind(author)
    .bind(published_at)
    .bind(reference)
    .bind(crate::compression::compress_text(content).map_err(AppError::internal)?)
    .bind(item_kind)
    .bind(source_type)
    .bind(source_subtype)
    .bind(metadata.map(json_zstd).transpose()?)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn trace_zstd(refs: serde_json::Value) -> AppResult<Vec<u8>> {
    json_zstd(serde_json::json!({ "refs": refs }))
}

async fn first_item_id(tx: &mut sqlx::Transaction<'_, Sqlite>, external_suffix: &str) -> AppResult<i64> {
    sqlx::query_scalar("SELECT id FROM items WHERE external_id = ?")
        .bind(format!("{FIXTURE_EXTERNAL_PREFIX}{external_suffix}"))
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::database)
}

async fn insert_analysis_runs(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    ids: FixtureIds,
) -> AppResult<()> {
    let youtube_item_id = first_item_id(tx, "youtube-transcript").await?;
    let telegram_item_id = first_item_id(tx, "tg-channel-1").await?;
    let youtube_ref = format!("s{}-i{}@754000ms", ids.youtube_video_id, youtube_item_id);
    let telegram_ref = format!("s{}-i{}", ids.telegram_channel_id, telegram_item_id);

    let completed_youtube_run_id = insert_run(
        tx,
        COMPLETED_SNAPSHOT_RUN_LABEL,
        "single_source",
        Some(ids.youtube_video_id),
        None,
        ids.prompt_template_id,
        "completed",
        Some(&format!(
            "# {COMPLETED_SNAPSHOT_RUN_LABEL}\n\nYouTube evidence is available at [{youtube_ref}]."
        )),
        Some(trace_zstd(serde_json::json!([{
            "ref": youtube_ref,
            "item_id": youtube_item_id,
            "source_id": ids.youtube_video_id,
            "external_id": format!("{FIXTURE_EXTERNAL_PREFIX}youtube-transcript"),
            "published_at": FIXTURE_PERIOD_FROM + 6_000,
            "excerpt": "Fixture timestamp segment supports Show in source.",
            "youtube_url": "https://www.youtube.com/watch?v=analysis_fixture_video&t=754",
            "youtube_timestamp_seconds": 754,
            "youtube_display_label": format!("{YOUTUBE_VIDEO_LABEL} at 12:34"),
            "is_synthetic": false
        }] ))?),
        None,
        Some(FIXTURE_NOW + 20),
    )
    .await?;
    insert_snapshot_message(
        tx,
        completed_youtube_run_id,
        youtube_item_id,
        ids.youtube_video_id,
        &format!("{FIXTURE_EXTERNAL_PREFIX}youtube-transcript"),
        "Fixture Channel",
        FIXTURE_PERIOD_FROM + 6_000,
        &youtube_ref,
        "Fixture timestamp segment supports Show in source.",
        "youtube_transcript",
        "youtube",
        "video",
        Some(serde_json::json!({
            "analysis_redesign_fixture": true,
            "canonical_url": "https://www.youtube.com/watch?v=analysis_fixture_video",
            "title": YOUTUBE_VIDEO_LABEL,
            "segment_start_ms": 754000,
            "segment_end_ms": 779000
        })),
    )
    .await?;

    let missing_ref = format!("s{}-i999999", ids.telegram_channel_id);
    insert_run(
        tx,
        MISSING_SNAPSHOT_RUN_LABEL,
        "single_source",
        Some(ids.telegram_channel_id),
        None,
        ids.prompt_template_id,
        "completed",
        Some(&format!(
            "# {MISSING_SNAPSHOT_RUN_LABEL}\n\nThis report cites missing saved evidence [{missing_ref}]."
        )),
        Some(trace_zstd(serde_json::json!([{
            "ref": missing_ref,
            "item_id": 999999,
            "source_id": ids.telegram_channel_id,
            "external_id": "missing-fixture-item",
            "published_at": FIXTURE_PERIOD_FROM + 100,
            "excerpt": "Missing fixture evidence",
            "youtube_url": null,
            "youtube_timestamp_seconds": null,
            "youtube_display_label": null,
            "is_synthetic": false
        }] ))?),
        None,
        Some(FIXTURE_NOW + 30),
    )
    .await?;

    for (label, status, error, completed_at) in [
        (RUNNING_RUN_LABEL, "running", None, None),
        (
            FAILED_RUN_LABEL,
            "failed",
            Some("Fixture failure: provider request failed without changing user data"),
            Some(FIXTURE_NOW + 40),
        ),
        (
            CANCELLED_RUN_LABEL,
            "cancelled",
            Some("Fixture cancellation: run was cancelled before snapshot capture"),
            Some(FIXTURE_NOW + 50),
        ),
    ] {
        insert_run(
            tx,
            label,
            "single_source",
            Some(ids.telegram_channel_id),
            None,
            ids.prompt_template_id,
            status,
            None,
            None,
            error,
            completed_at,
        )
        .await?;
    }

    let group_run_id = insert_run(
        tx,
        GROUP_SNAPSHOT_RUN_LABEL,
        "source_group",
        None,
        Some(ids.source_group_id),
        ids.prompt_template_id,
        "completed",
        Some(&format!(
            "# {GROUP_SNAPSHOT_RUN_LABEL}\n\nTelegram evidence is available at [{telegram_ref}]."
        )),
        Some(trace_zstd(serde_json::json!([{
            "ref": telegram_ref,
            "item_id": telegram_item_id,
            "source_id": ids.telegram_channel_id,
            "external_id": format!("{FIXTURE_EXTERNAL_PREFIX}tg-channel-1"),
            "published_at": FIXTURE_PERIOD_FROM + 1_200,
            "excerpt": "fixture channel update: result-first analysis now has source evidence",
            "youtube_url": null,
            "youtube_timestamp_seconds": null,
            "youtube_display_label": null,
            "source_type": "telegram",
            "is_synthetic": false
        }] ))?),
        None,
        Some(FIXTURE_NOW + 60),
    )
    .await?;
    insert_snapshot_message(
        tx,
        group_run_id,
        telegram_item_id,
        ids.telegram_channel_id,
        &format!("{FIXTURE_EXTERNAL_PREFIX}tg-channel-1"),
        "Fixture Editor",
        FIXTURE_PERIOD_FROM + 1_200,
        &telegram_ref,
        "fixture channel update: result-first analysis now has source evidence",
        "telegram_message",
        "telegram",
        "channel",
        Some(serde_json::json!({ "analysis_redesign_fixture": true })),
    )
    .await?;

    let supergroup_topic_id = first_item_id(tx, "tg-supergroup-topic").await?;
    let supergroup_media_id = first_item_id(tx, "tg-supergroup-media").await?;
    insert_snapshot_message(
        tx,
        group_run_id,
        supergroup_topic_id,
        ids.telegram_supergroup_id,
        &format!("{FIXTURE_EXTERNAL_PREFIX}tg-supergroup-topic"),
        "Fixture Moderator",
        FIXTURE_PERIOD_FROM + 3_600,
        &format!("s{}-i{}", ids.telegram_supergroup_id, supergroup_topic_id),
        "fixture supergroup topic: grouped source reader should preserve topic metadata",
        "telegram_message",
        "telegram",
        "supergroup",
        Some(serde_json::json!({ "analysis_redesign_fixture": true })),
    )
    .await?;
    insert_snapshot_message(
        tx,
        group_run_id,
        supergroup_media_id,
        ids.telegram_supergroup_id,
        &format!("{FIXTURE_EXTERNAL_PREFIX}tg-supergroup-media"),
        "Fixture Member",
        FIXTURE_PERIOD_FROM + 4_800,
        &format!("s{}-i{}", ids.telegram_supergroup_id, supergroup_media_id),
        "fixture supergroup media placeholder: no binary preview is available",
        "telegram_message",
        "telegram",
        "supergroup",
        Some(serde_json::json!({ "analysis_redesign_fixture": true })),
    )
    .await?;

    for (role, content, created_at) in [
        ("user", "Summarize the strongest fixture evidence.", FIXTURE_NOW + 70),
        (
            "assistant",
            "The fixture evidence highlights saved snapshots, YouTube timestamps, and Telegram source context.",
            FIXTURE_NOW + 71,
        ),
    ] {
        sqlx::query(
            "INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(group_run_id)
        .bind(role)
        .bind(content)
        .bind(created_at)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    Ok(())
}
```

- [ ] **Step 4: Call analysis run helper from seed**

After the source content helpers in `seed_analysis_redesign_fixtures_in_pool`, add:

```rust
insert_analysis_runs(
    &mut tx,
    FixtureIds {
        prompt_template_id,
        telegram_channel_id,
        telegram_supergroup_id,
        youtube_video_id,
        source_group_id,
    },
)
.await?;
```

Update returned summary:

```rust
runs: 6,
snapshot_messages: 4,
chat_messages: 2,
```

- [ ] **Step 5: Run run and trace tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_fixture_runs_with_statuses_templates_and_snapshots analysis::fixtures::tests::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot analysis::fixtures::tests::missing_snapshot_run_has_trace_but_no_saved_messages
```

Expected: PASS.

- [ ] **Step 6: Run the full fixture module tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests
```

Expected: PASS.

- [ ] **Step 7: Commit analysis run seed**

Run:

```powershell
git add src-tauri/src/analysis/fixtures.rs
git commit -m "feat: seed analysis fixture runs"
```

## Task 6: Export And Register Debug-Only Tauri Commands

**Files:**
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Export fixture commands from `analysis/mod.rs`**

In `src-tauri/src/analysis/mod.rs`, add this module declaration near the other `mod` declarations:

```rust
#[cfg(debug_assertions)]
mod fixtures;
```

Add this re-export near the existing `pub use` block:

```rust
#[cfg(debug_assertions)]
pub use self::fixtures::{
    clear_analysis_redesign_fixtures, seed_analysis_redesign_fixtures,
};
```

- [ ] **Step 2: Import fixture commands in `lib.rs`**

In `src-tauri/src/lib.rs`, add the two commands to the existing `use analysis::{ ... }` list with cfg attributes:

```rust
#[cfg(debug_assertions)]
use analysis::{clear_analysis_redesign_fixtures, seed_analysis_redesign_fixtures};
```

Keep the current broad `use analysis::{ ... }` list for existing commands unchanged.

- [ ] **Step 3: Register fixture commands under debug assertions**

In the existing `tauri::generate_handler![ ... ]` invocation in `src-tauri/src/lib.rs`, append these entries after `cancel_analysis_run` and before the YouTube command entries:

```rust
            #[cfg(debug_assertions)]
            seed_analysis_redesign_fixtures,
            #[cfg(debug_assertions)]
            clear_analysis_redesign_fixtures,
```

The Tauri macro supports outer attributes on command definitions, so these command names are included in debug builds and absent from release builds.

- [ ] **Step 4: Verify debug build command registration compiles**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::summary_serializes_with_camel_case_keys
```

Expected: PASS, proving the module export compiles in the normal debug test build.

- [ ] **Step 5: Verify release build does not register fixture commands**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --release
```

Expected: PASS. The release build must not require `seed_analysis_redesign_fixtures` or `clear_analysis_redesign_fixtures` to be imported or registered.

- [ ] **Step 6: Commit command registration**

Run:

```powershell
git add src-tauri/src/analysis/mod.rs src-tauri/src/lib.rs
git commit -m "feat: register analysis fixture commands in debug"
```

## Task 7: Run Backend Verification For Fixture Infrastructure

**Files:**
- Verify: `src-tauri/src/analysis/fixtures.rs`
- Verify: `src-tauri/src/analysis/mod.rs`
- Verify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Run focused fixture tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests
```

Expected: PASS.

- [ ] **Step 2: Run focused adjacent analysis tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::trace::tests analysis::corpus::tests::load_run_corpus_messages_uses_snapshot_when_available analysis::corpus::tests::playlist_expansion_excludes_unlinked_and_removed_rows youtube::detail::tests::video_detail_reports_synced_transcript_comments_and_playlist_memberships youtube::detail::tests::playlist_detail_reports_ordered_items_and_summary_counts
```

Expected: PASS.

- [ ] **Step 3: Run the full backend suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 4: Run release compile verification**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --release
```

Expected: PASS.

- [ ] **Step 5: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [ ] **Step 6: Commit verification fixes**

If the previous commands required small compile or formatting repairs, commit only those repairs:

```powershell
git add src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/mod.rs src-tauri/src/lib.rs
git commit -m "fix: harden analysis fixture verification"
```

If no repairs were made, skip this commit.

## Task 8: Run Fixture-Backed Browser Verification

**Files:**
- Update: `docs/superpowers/verification/2026-05-10-analysis-redesign.md`

- [ ] **Step 1: Start the Tauri development app**

Run the project’s Tauri development command:

```powershell
npm.cmd run tauri -- dev
```

Expected: the Tauri dev app opens and the MCP bridge remains available in debug mode.

- [ ] **Step 2: Seed fixtures from the browser console**

In the running debug app, open devtools console for the webview and run:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

Expected: the seed call returns:

```json
{
  "accounts": 1,
  "llmProfiles": 1,
  "sources": 4,
  "sourceGroups": 1,
  "promptTemplates": 1,
  "runs": 6,
  "snapshotMessages": 4,
  "chatMessages": 2,
  "youtubeTranscriptSegments": 3,
  "youtubePlaylistItems": 2
}
```

- [ ] **Step 3: Open `/analysis` and exercise fixture scenarios**

Open `/analysis` in the debug app. Exercise these rows from `docs/superpowers/verification/2026-05-10-analysis-redesign.md` and record actual results:

```text
Selecting source clears opened run and shows live source + Runs
Completed saved run opens Report + Evidence and aligns rail if live scope exists
Completed saved run with missing snapshot does not resolve evidence/chat against live source
Running run opens Report and Source shows pending snapshot
Failed/cancelled run shows snapshot if available, otherwise explicit live source option
Trace ref click activates Evidence
Show in source prefers run snapshot and highlights message/segment
Chat tab activates only on explicit tab selection or question submit
Runs search/status/scope filters work and exclude source ingest jobs
Telegram timeline shows groups, metadata, and media placeholders only
YouTube video reader shows transcript timestamps and copy/open actions
YouTube playlist reader shows playlist item list before transcript reading
Source group reader groups by source with counts
Workspace persistence restores source/group and UI context without opening a run
```

Use fixture labels to select exact rows. Mark a scenario `PASS` only after it is exercised in the browser. Mark a scenario `FAIL` with the observed broken behavior. Keep `BLOCKED` only for behavior outside this fixture scope, such as live source job progress.

- [ ] **Step 4: Clear fixtures after browser verification**

Run in the webview console:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
```

Expected: the returned deleted-row summary matches the seeded row counts.

- [ ] **Step 5: Update verification document**

In `docs/superpowers/verification/2026-05-10-analysis-redesign.md`, replace the fixture-covered `BLOCKED` rows with the actual browser results from Step 3. Add a short residual risk entry for any scenario that remains outside DB fixture scope.

- [ ] **Step 6: Commit browser verification record**

Run:

```powershell
git add docs/superpowers/verification/2026-05-10-analysis-redesign.md
git commit -m "docs: record fixture-backed analysis verification"
```

## Task 9: Final Gate

**Files:**
- Verify all changed files.

- [ ] **Step 1: Run final backend checks**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --release
```

Expected: all commands PASS.

- [ ] **Step 2: Run frontend regression checks from the redesign**

Run:

```powershell
npm.cmd test -- src/lib/analysis-redesign-workflow-scenarios.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts
npm.cmd run check
```

Expected: both commands PASS.

- [ ] **Step 3: Search for accidental fixture leakage**

Run:

```powershell
rg -n "seed_analysis_redesign_fixtures|clear_analysis_redesign_fixtures|__analysis_redesign_fixture__" src src-tauri/src docs/superpowers/verification/2026-05-10-analysis-redesign.md
```

Expected: matches are limited to `src-tauri/src/analysis/fixtures.rs`, debug-only exports/registration in `src-tauri/src/analysis/mod.rs` and `src-tauri/src/lib.rs`, and the verification document.

- [ ] **Step 4: Check whitespace and status**

Run:

```powershell
git diff --check
git status --short
```

Expected: `git diff --check` exits 0. `git status --short` is clean after all implementation and verification commits.

- [ ] **Step 5: Stop for user review**

Report:

```text
Fixture-backed browser verification infrastructure is implemented, debug-only, backend-tested, and the /analysis verification record has been updated from the seeded dataset.
```

Do not begin additional runtime job fixtures, product UI, or Playwright automation without a new explicit request.

## Self-Review

- Spec coverage: the plan covers debug-only seed and clear commands, existing SQLite path, marker-based cleanup, parent-id child cleanup, deterministic labels, Telegram fixture account, prompt template, non-secret LLM profile metadata, post-sync sources, source group, source items, YouTube transcript and playlist rows, analysis statuses, snapshots, missing-snapshot degradation, trace refs, chat history, and manual browser verification.
- Non-goals: the plan does not add product UI, release command registration, fake secrets, live ingest jobs, Playwright, or completed-run evidence fallback to live sources.
- Test coverage: the plan adds backend tests for idempotent seed, safe clear, non-fixture preservation, child cleanup, secret-free account/profile state, reader content, trace refs, statuses, snapshots, missing snapshots, and compressed field readability.
- Type consistency: command names are `seed_analysis_redesign_fixtures` and `clear_analysis_redesign_fixtures`; summary fields serialize to camelCase while Rust fields stay snake_case.
