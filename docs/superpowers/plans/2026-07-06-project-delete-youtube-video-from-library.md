# Project YouTube Video Delete From Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe project Sources action that deletes one selected YouTube video from the current project and from Library in one atomic operation.

**Architecture:** The backend owns the authoritative delete transaction, source-kind validation, cross-project blocking check, ingest/delete lock, and SQLite foreign-key enforcement. The frontend adds a narrow command wrapper, workflow/model helpers, and two UI integrations: the current `SourcesTab` toolbar and the `/projects/next` `SourcesBulkBar`. Existing `Remove` remains project-membership-only.

**Tech Stack:** Tauri 2, Rust, SQLx SQLite, Svelte 5 runes, TypeScript, Vitest, Testing Library Svelte.

## Global Constraints

- Execution starts by creating a feature branch, for example `feat/project-delete-youtube-video-library`.
- Use `apply_patch` for manual edits.
- Do not stage or commit `.claude/settings.local.json`.
- Use `npm.cmd` for frontend validation commands on Windows.
- Run focused tests for changed helper, contract, and command logic.
- Run `npm.cmd run check` after Svelte or TypeScript changes.
- Run `cargo check --manifest-path src-tauri/Cargo.toml` after Rust/Tauri changes.
- The new action deletes only one selected source, only when `provider === "youtube"` and `source_subtype === "video"`.
- The existing `Remove` action keeps deleting only `project_sources` membership.
- No partial deletion is allowed when another project uses the source.
- The backend returns at most three blocking projects and `remaining_blocking_project_count`.
- Omit the `and N more` suffix when `remaining_blocking_project_count === 0`.
- `PRAGMA foreign_keys = ON` must be enabled and verified before `BEGIN IMMEDIATE`.
- `youtube_playlist_items.video_source_id` must detach with `ON DELETE SET NULL`.
- Dependent source materials are removed through SQLite FK cascade, not through a second manual cleanup path.
- New API `status` values must be documented in `docs/value-registry.md`.

---

## File Map

- `src-tauri/src/tx.rs`: add reusable SQLite FK setup and immediate-transaction helper.
- `src-tauri/src/sources/store.rs`: factor standalone source row deletion so project-scoped deletion can reuse it on one prepared connection.
- `src-tauri/src/projects/mod.rs`: add project-scoped delete DTOs, in-pool implementation, Tauri command, and backend tests.
- `src-tauri/src/lib.rs`: register the new Tauri command.
- `src/lib/types/projects.ts`: add frontend request/response types for the new command.
- `src/lib/api/projects.ts`: add `deleteProjectYoutubeVideoSourceFromLibrary`.
- `src/lib/api/projects.test.ts`: cover the new Tauri invoke wrapper.
- `src/lib/ui/research-projects-model.ts`: add disabled-state and blocked-status helpers.
- `src/lib/ui/research-projects-model.test.ts`: cover helper behavior.
- `src/lib/ui/research-projects-workflow.ts`: add workflow dependency and delete helper for the current projects screen.
- `src/lib/ui/research-projects-workflow.test.ts`: cover confirm cancel, success refresh, and blocked status.
- `src/lib/components/research-projects/SourcesTab.svelte`: add the `Delete from Library` toolbar button.
- `src/lib/components/research-projects/ProjectWorkspace.svelte`: pass the new callback to `SourcesTab`.
- `src/lib/components/research-projects/ProjectsShell.svelte`: expose the new callback to the workspace.
- `src/routes/projects/+page.svelte`: wire the new API wrapper into the current route workflow.
- `src/routes/projects/list/+page.svelte`: wire the new API wrapper into the list route workflow.
- `src/lib/research-projects-route-contract.test.ts`: cover route-level wiring and preserve membership-only Remove.
- `src/lib/components/research-projects/SourcesBulkBar.svelte`: add the `/projects/next` button and confirmation dialog.
- `src/lib/components/research-projects/SourcesBulkBar.test.ts`: cover visibility, disabled behavior, cancel, and confirm.
- `src/routes/projects/next/+page.svelte`: wire the new API wrapper and refresh path.
- `docs/value-registry.md`: document the new command outcome values.

---

### Task 1: SQLite FK Helper And Standalone Source Delete Factoring

**Files:**
- Modify: `src-tauri/src/tx.rs`
- Modify: `src-tauri/src/sources/store.rs`

**Interfaces:**
- Produces: `tx::enable_foreign_keys(conn: &mut SqlitePoolConnection) -> AppResult<()>`
- Produces: `tx::begin_immediate_with_foreign_keys(pool: &Pool<Sqlite>) -> AppResult<SqlitePoolConnection>`
- Produces: `sources::store::delete_source_row_on_connection(conn: &mut SqlitePoolConnection, source_id: i64) -> AppResult<u64>`
- Consumed by: Task 2 project-scoped delete transaction.

- [ ] **Step 1: Add failing FK helper tests in `src-tauri/src/tx.rs`**

Add imports in the test module:

```rust
use super::{begin_immediate, begin_immediate_with_foreign_keys, commit, finish_manual_transaction, rollback};
```

Insert these tests in `mod tests`:

```rust
#[tokio::test]
async fn begin_immediate_with_foreign_keys_enforces_cascade() {
    let pool = sqlx::SqlitePool::connect(":memory:")
        .await
        .expect("connect in-memory db");
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&pool)
        .await
        .expect("disable foreign keys for baseline");
    sqlx::query("CREATE TABLE parents (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .expect("create parent table");
    sqlx::query(
        "CREATE TABLE children (id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES parents(id) ON DELETE CASCADE)",
    )
    .execute(&pool)
    .await
    .expect("create child table");
    sqlx::query("INSERT INTO parents (id) VALUES (1)")
        .execute(&pool)
        .await
        .expect("insert parent");
    sqlx::query("INSERT INTO children (id, parent_id) VALUES (10, 1)")
        .execute(&pool)
        .await
        .expect("insert child");

    let mut conn = begin_immediate_with_foreign_keys(&pool)
        .await
        .expect("begin immediate with foreign keys");
    let enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut *conn)
        .await
        .expect("read foreign key pragma");
    assert_eq!(enabled, 1);

    sqlx::query("DELETE FROM parents WHERE id = 1")
        .execute(&mut *conn)
        .await
        .expect("delete parent");
    commit(&mut conn).await.expect("commit");

    let child_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM children")
        .fetch_one(&pool)
        .await
        .expect("count children");
    assert_eq!(child_count, 0);
}

#[tokio::test]
async fn sqlite_ignores_foreign_keys_pragma_inside_open_transaction() {
    let pool = sqlx::SqlitePool::connect(":memory:")
        .await
        .expect("connect in-memory db");
    let mut conn = pool.acquire().await.expect("acquire connection");
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await
        .expect("disable foreign keys");
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .expect("begin immediate");
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await
        .expect("attempt pragma inside transaction");

    let enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut *conn)
        .await
        .expect("read foreign key pragma");
    assert_eq!(enabled, 0);

    sqlx::query("ROLLBACK")
        .execute(&mut *conn)
        .await
        .expect("rollback");
}
```

- [ ] **Step 2: Run the FK helper tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml tx::tests::begin_immediate_with_foreign_keys_enforces_cascade
```

Expected: FAIL to compile because `begin_immediate_with_foreign_keys` is missing.

- [ ] **Step 3: Implement FK setup in `src-tauri/src/tx.rs`**

Replace the top imports and add the helper:

```rust
use crate::error::{database_error, AppError, AppResult};
use sqlx::{Pool, Sqlite};

pub(crate) type SqlitePoolConnection = sqlx::pool::PoolConnection<Sqlite>;

pub(crate) async fn enable_foreign_keys(conn: &mut SqlitePoolConnection) -> AppResult<()> {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut **conn)
        .await
        .map_err(database_error)?;

    let enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut **conn)
        .await
        .map_err(database_error)?;
    if enabled != 1 {
        return Err(AppError::internal(
            "SQLite foreign key enforcement could not be enabled",
        ));
    }
    Ok(())
}

pub(crate) async fn begin_immediate_with_foreign_keys(
    pool: &Pool<Sqlite>,
) -> AppResult<SqlitePoolConnection> {
    let mut conn = pool.acquire().await.map_err(database_error)?;
    enable_foreign_keys(&mut conn).await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(database_error)?;
    Ok(conn)
}
```

Keep the existing `begin_immediate`, `commit`, `rollback`, and `finish_manual_transaction` functions unchanged.

- [ ] **Step 4: Add failing standalone cascade test in `src-tauri/src/sources/store.rs`**

Add this test to the existing `mod tests`:

```rust
#[tokio::test]
async fn delete_source_from_pool_enables_foreign_keys_and_cascades_dependents() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    crate::migrations::apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (70, 'youtube', 'video', 'video-70', 'Video 70', 1, 0, 100)",
    )
    .execute(&pool)
    .await
    .expect("seed source");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_zstd, item_kind) VALUES (700, 70, 'transcript-70', 'Author', 100, 101, x'01', 'youtube_transcript')",
    )
    .execute(&pool)
    .await
    .expect("seed item");
    sqlx::query(
        "INSERT INTO youtube_transcript_segments (item_id, source_id, start_ms, end_ms, text) VALUES (700, 70, 0, 1000, 'hello')",
    )
    .execute(&pool)
    .await
    .expect("seed transcript segment");
    sqlx::query(
        "INSERT INTO analysis_documents (id, source_id, document_kind, external_id, title, content_zstd, created_at, updated_at) VALUES (701, 70, 'youtube_transcript', 'doc-70', 'Doc 70', x'01', 100, 100)",
    )
    .execute(&pool)
    .await
    .expect("seed analysis document");

    let rows = delete_source_from_pool(&pool, 70)
        .await
        .expect("delete source");
    assert_eq!(rows, 1);

    for (label, query) in [
        ("sources", "SELECT COUNT(*) FROM sources WHERE id = 70"),
        ("items", "SELECT COUNT(*) FROM items WHERE source_id = 70"),
        (
            "youtube_transcript_segments",
            "SELECT COUNT(*) FROM youtube_transcript_segments WHERE source_id = 70",
        ),
        (
            "analysis_documents",
            "SELECT COUNT(*) FROM analysis_documents WHERE source_id = 70",
        ),
    ] {
        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|error| panic!("count {label}: {error}"));
        assert_eq!(count, 0, "{label} rows should be removed");
    }
}
```

- [ ] **Step 5: Run the standalone cascade test and verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents
```

Expected: FAIL because `delete_source_from_pool` has not enabled FK enforcement on its connection.

- [ ] **Step 6: Factor source row deletion in `src-tauri/src/sources/store.rs`**

Add this import near the existing crate imports:

```rust
use crate::tx::{enable_foreign_keys, SqlitePoolConnection};
```

Replace `delete_source_from_pool` with this version and add the low-level helper below it:

```rust
async fn delete_source_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<u64> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    enable_foreign_keys(&mut conn).await?;
    sqlx::query(&format!(
        "PRAGMA busy_timeout = {SOURCE_DELETE_BUSY_TIMEOUT_MS}"
    ))
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let project_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_sources WHERE source_id = ?")
            .bind(source_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(AppError::database)?;

    if project_count > 0 {
        return Err(AppError::validation(format!(
            "Source {source_id} is used by {project_count} project(s). Remove it from projects first."
        )));
    }

    delete_source_row_on_connection(&mut conn, source_id).await
}

pub(crate) async fn delete_source_row_on_connection(
    conn: &mut SqlitePoolConnection,
    source_id: i64,
) -> AppResult<u64> {
    sqlx::query("DELETE FROM sources WHERE id = ?")
        .bind(source_id)
        .execute(&mut **conn)
        .await
        .map(|result| result.rows_affected())
        .map_err(AppError::database)
}
```

- [ ] **Step 7: Run Task 1 tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml tx::tests::
cargo test --manifest-path src-tauri/Cargo.toml sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents
```

Expected: PASS.

- [ ] **Step 8: Commit Task 1**

Run:

```powershell
git add src-tauri/src/tx.rs src-tauri/src/sources/store.rs
git commit -m "fix: enforce sqlite foreign keys for source deletion"
```

---

### Task 2: Project-Scoped Backend Delete Command

**Files:**
- Modify: `src-tauri/src/projects/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `docs/value-registry.md`

**Interfaces:**
- Consumes: `begin_immediate_with_foreign_keys`
- Consumes: `delete_source_row_on_connection`
- Produces: Tauri command `delete_project_youtube_video_source_from_library(project_id: i64, source_id: i64)`
- Produces: Rust DTOs `DeleteProjectYoutubeVideoSourceOutcome`, `DeleteProjectYoutubeVideoSourceStatus`, `BlockingProjectReference`

- [ ] **Step 1: Add failing backend tests in `src-tauri/src/projects/mod.rs`**

In `mod tests`, add this helper:

```rust
async fn count_rows(pool: &sqlx::SqlitePool, sql: &str) -> i64 {
    sqlx::query_scalar(sql)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| panic!("count query failed: {sql}: {error}"))
}

fn quote_sqlite_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}
```

Add these tests:

```rust
#[tokio::test]
async fn project_scoped_delete_schema_source_foreign_keys_are_delete_safe() {
    use sqlx::Row;

    let pool = pool().await;
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .expect("list sqlite tables");

    let mut unsafe_refs = Vec::new();
    for table in tables {
        let pragma = format!("PRAGMA foreign_key_list({})", quote_sqlite_identifier(&table));
        let rows = sqlx::query(&pragma)
            .fetch_all(&pool)
            .await
            .unwrap_or_else(|error| panic!("foreign_key_list for {table}: {error}"));
        for row in rows {
            let target_table: String = row.try_get("table").expect("target table");
            if target_table != "sources" {
                continue;
            }
            let from_column: String = row.try_get("from").expect("from column");
            let on_delete: String = row.try_get("on_delete").expect("on_delete action");
            let delete_safe = matches!(on_delete.as_str(), "CASCADE" | "SET NULL")
                || (table == "project_sources" && on_delete == "RESTRICT");
            if !delete_safe {
                unsafe_refs.push(format!("{table}.{from_column} -> sources ON DELETE {on_delete}"));
            }
        }
    }

    assert!(
        unsafe_refs.is_empty(),
        "Every FK to sources(id) must be CASCADE/SET NULL, except project_sources.source_id RESTRICT handled by the project delete transaction: {unsafe_refs:?}"
    );
}

#[tokio::test]
async fn project_scoped_delete_removes_youtube_video_and_cascaded_materials() {
    let pool = pool().await;
    seed_source(&pool, 30, "youtube", "playlist").await;
    seed_source(&pool, 31, "youtube", "video").await;
    let project = create_project_in_pool(&pool, "Video project", None)
        .await
        .expect("create project");
    add_project_sources_in_pool(&pool, project.id, vec![31])
        .await
        .expect("link video");
    sqlx::query(
        "INSERT INTO youtube_playlist_items (playlist_source_id, video_source_id, video_id, position, availability_status, is_removed_from_playlist) VALUES (30, 31, 'video-31', 1, 'available', 0)",
    )
    .execute(&pool)
    .await
    .expect("link playlist item");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_zstd, item_kind) VALUES (310, 31, 'transcript-31', 'Author', 100, 101, x'01', 'youtube_transcript')",
    )
    .execute(&pool)
    .await
    .expect("seed transcript item");
    sqlx::query(
        "INSERT INTO youtube_transcript_segments (item_id, source_id, start_ms, end_ms, text) VALUES (310, 31, 0, 1000, 'hello')",
    )
    .execute(&pool)
    .await
    .expect("seed transcript segment");
    sqlx::query(
        "INSERT INTO analysis_documents (id, source_id, document_kind, external_id, title, content_zstd, created_at, updated_at) VALUES (311, 31, 'youtube_transcript', 'doc-31', 'Doc 31', x'01', 100, 100)",
    )
    .execute(&pool)
    .await
    .expect("seed analysis document");

    let outcome = delete_project_youtube_video_source_from_library_in_pool(&pool, project.id, 31)
        .await
        .expect("delete project video source");

    assert_eq!(outcome.status, DeleteProjectYoutubeVideoSourceStatus::Deleted);
    assert!(outcome.blocking_projects.is_empty());
    assert_eq!(outcome.remaining_blocking_project_count, 0);
    assert_eq!(count_rows(&pool, "SELECT COUNT(*) FROM sources WHERE id = 31").await, 0);
    assert_eq!(count_rows(&pool, "SELECT COUNT(*) FROM project_sources WHERE source_id = 31").await, 0);
    assert_eq!(count_rows(&pool, "SELECT COUNT(*) FROM items WHERE source_id = 31").await, 0);
    assert_eq!(count_rows(&pool, "SELECT COUNT(*) FROM youtube_transcript_segments WHERE source_id = 31").await, 0);
    assert_eq!(count_rows(&pool, "SELECT COUNT(*) FROM analysis_documents WHERE source_id = 31").await, 0);

    let detached: Option<i64> =
        sqlx::query_scalar("SELECT video_source_id FROM youtube_playlist_items WHERE video_id = 'video-31'")
            .fetch_one(&pool)
            .await
            .expect("load playlist item");
    assert_eq!(detached, None);
}

#[tokio::test]
async fn project_scoped_delete_blocks_other_active_and_archived_projects_without_mutation() {
    let pool = pool().await;
    seed_source(&pool, 40, "youtube", "video").await;
    let current = create_project_in_pool(&pool, "Current", None)
        .await
        .expect("create current project");
    let active = create_project_in_pool(&pool, "Active blocker", None)
        .await
        .expect("create active project");
    let archived = create_project_in_pool(&pool, "Archived blocker", None)
        .await
        .expect("create archived project");
    set_project_archived_in_pool(&pool, archived.id, true)
        .await
        .expect("archive blocker");
    add_project_sources_in_pool(&pool, current.id, vec![40])
        .await
        .expect("link current");
    add_project_sources_in_pool(&pool, active.id, vec![40])
        .await
        .expect("link active");
    add_project_sources_in_pool(&pool, archived.id, vec![40])
        .await
        .expect("link archived");

    let outcome = delete_project_youtube_video_source_from_library_in_pool(&pool, current.id, 40)
        .await
        .expect("blocked outcome");

    assert_eq!(
        outcome.status,
        DeleteProjectYoutubeVideoSourceStatus::BlockedByOtherProjects
    );
    assert_eq!(
        outcome
            .blocking_projects
            .iter()
            .map(|project| project.title.as_str())
            .collect::<Vec<_>>(),
        vec!["Active blocker", "Archived blocker"]
    );
    assert_eq!(outcome.remaining_blocking_project_count, 0);
    assert_eq!(count_rows(&pool, "SELECT COUNT(*) FROM sources WHERE id = 40").await, 1);
    assert_eq!(count_rows(&pool, "SELECT COUNT(*) FROM project_sources WHERE source_id = 40").await, 3);
}

#[tokio::test]
async fn project_scoped_delete_caps_blocking_projects_and_reports_remaining_count() {
    let pool = pool().await;
    seed_source(&pool, 50, "youtube", "video").await;
    let current = create_project_in_pool(&pool, "Current", None)
        .await
        .expect("create current project");
    add_project_sources_in_pool(&pool, current.id, vec![50])
        .await
        .expect("link current");
    for name in ["Alpha", "Beta", "Gamma", "Omega", "Zeta"] {
        let project = create_project_in_pool(&pool, name, None)
            .await
            .expect("create blocker");
        add_project_sources_in_pool(&pool, project.id, vec![50])
            .await
            .expect("link blocker");
    }

    let outcome = delete_project_youtube_video_source_from_library_in_pool(&pool, current.id, 50)
        .await
        .expect("blocked outcome");

    assert_eq!(
        outcome
            .blocking_projects
            .iter()
            .map(|project| project.title.as_str())
            .collect::<Vec<_>>(),
        vec!["Alpha", "Beta", "Gamma"]
    );
    assert_eq!(outcome.remaining_blocking_project_count, 2);
}

#[tokio::test]
async fn project_scoped_delete_rejects_invalid_sources_and_missing_links() {
    let pool = pool().await;
    seed_account(&pool, 1).await;
    seed_source(&pool, 60, "youtube", "playlist").await;
    seed_source(&pool, 61, "telegram", "supergroup").await;
    seed_source(&pool, 62, "youtube", "video").await;
    let project = create_project_in_pool(&pool, "Validation", None)
        .await
        .expect("create project");
    add_project_sources_in_pool(&pool, project.id, vec![60, 61])
        .await
        .expect("link invalid types");

    for (source_id, label) in [(60, "playlist"), (61, "telegram")] {
        let error = delete_project_youtube_video_source_from_library_in_pool(&pool, project.id, source_id)
            .await
            .expect_err("invalid source should be rejected");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(
            error.message.contains("Only YouTube video"),
            "{label} should report a YouTube-video validation message"
        );
    }

    let missing_link = delete_project_youtube_video_source_from_library_in_pool(&pool, project.id, 62)
        .await
        .expect_err("missing link rejected");
    assert_eq!(missing_link.kind, crate::error::AppErrorKind::Validation);

    let missing_project = delete_project_youtube_video_source_from_library_in_pool(&pool, 999_999, 62)
        .await
        .expect_err("missing project rejected");
    assert_eq!(missing_project.kind, crate::error::AppErrorKind::NotFound);
}
```

- [ ] **Step 2: Run the backend command tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::tests::project_scoped_delete
```

Expected: FAIL to compile because `delete_project_youtube_video_source_from_library_in_pool` and DTOs are missing. The schema guard test is named with the same `project_scoped_delete` prefix so it runs under this filter.

- [ ] **Step 3: Add backend DTOs in `src-tauri/src/projects/mod.rs`**

Add after `AddProjectSourcesOutcome`:

```rust
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeleteProjectYoutubeVideoSourceStatus {
    Deleted,
    BlockedByOtherProjects,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct BlockingProjectReference {
    pub project_id: i64,
    pub title: String,
    pub archived: bool,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct DeleteProjectYoutubeVideoSourceOutcome {
    pub status: DeleteProjectYoutubeVideoSourceStatus,
    pub blocking_projects: Vec<BlockingProjectReference>,
    pub remaining_blocking_project_count: i64,
}
```

Add imports near the top of the file:

```rust
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::sources::identity_repair::{require_source_identity_ready, SourceIdentityRepairState};
use crate::sources::store::delete_source_row_on_connection;
use crate::tx::{begin_immediate_with_foreign_keys, commit, rollback};
```

- [ ] **Step 4: Add the in-pool delete implementation in `src-tauri/src/projects/mod.rs`**

Add this helper before the command section:

```rust
#[derive(sqlx::FromRow)]
struct BlockingProjectRow {
    project_id: i64,
    title: String,
    archived: i64,
}

pub(crate) async fn delete_project_youtube_video_source_from_library_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    source_id: i64,
) -> AppResult<DeleteProjectYoutubeVideoSourceOutcome> {
    let mut conn = begin_immediate_with_foreign_keys(pool).await?;

    let result: AppResult<DeleteProjectYoutubeVideoSourceOutcome> = async {
        let source: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT source_type, source_subtype FROM sources WHERE id = ?",
        )
        .bind(source_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(AppError::database)?;

        let Some((provider, subtype)) = source else {
            return Err(AppError::not_found(format!("Source {source_id} not found")));
        };
        if provider != "youtube" || subtype.as_deref() != Some("video") {
            return Err(AppError::validation(
                "Only YouTube video sources can be deleted from Library here",
            ));
        }

        let project_exists: i64 =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)")
                .bind(project_id)
                .fetch_one(&mut *conn)
                .await
                .map_err(AppError::database)?;
        if project_exists == 0 {
            return Err(AppError::not_found(format!(
                "Project {project_id} not found"
            )));
        }

        let linked: i64 = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM project_sources WHERE project_id = ? AND source_id = ?)",
        )
        .bind(project_id)
        .bind(source_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;
        if linked == 0 {
            return Err(AppError::validation(format!(
                "Source {source_id} is not linked to project {project_id}"
            )));
        }

        let blocking_rows: Vec<BlockingProjectRow> = sqlx::query_as(
            r#"
            SELECT p.id AS project_id,
                   p.name AS title,
                   CASE WHEN p.archived_at IS NULL THEN 0 ELSE 1 END AS archived
            FROM project_sources ps
            JOIN projects p ON p.id = ps.project_id
            WHERE ps.source_id = ? AND ps.project_id <> ?
            ORDER BY p.name COLLATE NOCASE ASC, p.id ASC
            "#,
        )
        .bind(source_id)
        .bind(project_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::database)?;

        if !blocking_rows.is_empty() {
            let total = blocking_rows.len();
            let blocking_projects = blocking_rows
                .into_iter()
                .take(3)
                .map(|row| BlockingProjectReference {
                    project_id: row.project_id,
                    title: row.title,
                    archived: row.archived != 0,
                })
                .collect::<Vec<_>>();
            return Ok(DeleteProjectYoutubeVideoSourceOutcome {
                status: DeleteProjectYoutubeVideoSourceStatus::BlockedByOtherProjects,
                blocking_projects,
                remaining_blocking_project_count: total.saturating_sub(3) as i64,
            });
        }

        sqlx::query("DELETE FROM project_sources WHERE project_id = ? AND source_id = ?")
            .bind(project_id)
            .bind(source_id)
            .execute(&mut *conn)
            .await
            .map_err(AppError::database)?;
        sqlx::query("UPDATE projects SET updated_at = ? WHERE id = ?")
            .bind(crate::time::now_secs())
            .bind(project_id)
            .execute(&mut *conn)
            .await
            .map_err(AppError::database)?;
        let deleted = delete_source_row_on_connection(&mut conn, source_id).await?;
        if deleted == 0 {
            return Err(AppError::not_found(format!("Source {source_id} not found")));
        }

        Ok(DeleteProjectYoutubeVideoSourceOutcome {
            status: DeleteProjectYoutubeVideoSourceStatus::Deleted,
            blocking_projects: Vec::new(),
            remaining_blocking_project_count: 0,
        })
    }
    .await;

    match result {
        Ok(outcome) => {
            commit(&mut conn).await?;
            Ok(outcome)
        }
        Err(error) => {
            let _ = rollback(&mut conn).await;
            Err(error)
        }
    }
}
```

- [ ] **Step 5: Add the Tauri command in `src-tauri/src/projects/mod.rs`**

Add near the other `#[tauri::command]` functions:

```rust
#[tauri::command]
pub async fn delete_project_youtube_video_source_from_library(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    ingest_locks: tauri::State<'_, SourceIngestLocks>,
    project_id: i64,
    source_id: i64,
) -> AppResult<DeleteProjectYoutubeVideoSourceOutcome> {
    require_source_identity_ready(repair_state.inner()).await?;
    let _ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::Delete)
        .await?;
    let pool = get_pool(&handle).await?;
    delete_project_youtube_video_source_from_library_in_pool(&pool, project_id, source_id).await
}
```

- [ ] **Step 6: Register the Tauri command in `src-tauri/src/lib.rs`**

Add the command to the `use crate::projects::{...}` import list and to `tauri::generate_handler![...]`:

```rust
delete_project_youtube_video_source_from_library,
```

- [ ] **Step 7: Update `docs/value-registry.md`**

In the `Library, projects, and source import UI` table, add this row:

```markdown
| Project YouTube video Library delete outcome | `DeleteProjectYoutubeVideoSourceOutcome.status` | `deleted`, `blocked_by_other_projects` | `src-tauri/src/projects/mod.rs`, `src/lib/types/projects.ts` | Tauri API response for project-scoped Library deletion. `deleted` is terminal success; `blocked_by_other_projects` is an expected non-error result when other projects still reference the source. |
```

- [ ] **Step 8: Run Task 2 tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::tests::project_scoped_delete
cargo test --manifest-path src-tauri/Cargo.toml sources::store::tests::delete_source
```

Expected: PASS.

- [ ] **Step 9: Commit Task 2**

Run:

```powershell
git add src-tauri/src/projects/mod.rs src-tauri/src/lib.rs docs/value-registry.md
git commit -m "feat: add project-scoped youtube library delete command"
```

---

### Task 3: Frontend API, Model, And Workflow Helpers

**Files:**
- Modify: `src/lib/types/projects.ts`
- Modify: `src/lib/api/projects.ts`
- Modify: `src/lib/api/projects.test.ts`
- Modify: `src/lib/ui/research-projects-model.ts`
- Modify: `src/lib/ui/research-projects-model.test.ts`
- Modify: `src/lib/ui/research-projects-workflow.ts`
- Modify: `src/lib/ui/research-projects-workflow.test.ts`

**Interfaces:**
- Produces: `deleteProjectYoutubeVideoSourceFromLibrary(input: DeleteProjectYoutubeVideoSourceInput): Promise<DeleteProjectYoutubeVideoSourceOutcome>`
- Produces: `selectedProjectSourceLibraryDeleteDisabledReason(rows): string | null`
- Produces: `projectSourceLibraryDeleteStatus(outcome): string`
- Produces: workflow method `deleteProjectYoutubeVideoSourceFromLibrary(sourceId: number): Promise<void>`

- [ ] **Step 1: Add failing API wrapper test in `src/lib/api/projects.test.ts`**

Add import:

```ts
deleteProjectYoutubeVideoSourceFromLibrary,
```

Add test:

```ts
it("invokes delete_project_youtube_video_source_from_library with project and source ids", async () => {
  mockedInvoke.mockResolvedValueOnce({
    status: "deleted",
    blocking_projects: [],
    remaining_blocking_project_count: 0,
  });

  const outcome = await deleteProjectYoutubeVideoSourceFromLibrary({
    projectId: 7,
    sourceId: 31,
  });

  expect(mockedInvoke).toHaveBeenCalledWith("delete_project_youtube_video_source_from_library", {
    projectId: 7,
    sourceId: 31,
  });
  expect(outcome.status).toBe("deleted");
});
```

- [ ] **Step 2: Add failing model tests in `src/lib/ui/research-projects-model.test.ts`**

Add tests:

```ts
describe("selectedProjectSourceLibraryDeleteDisabledReason", () => {
  const youtubeVideo = { provider: "youtube", subtype: "video" };
  const youtubePlaylist = { provider: "youtube", subtype: "playlist" };
  const telegram = { provider: "telegram", subtype: "supergroup" };

  it("requires exactly one selected source", () => {
    expect(selectedProjectSourceLibraryDeleteDisabledReason([])).toBe("Select one YouTube video source");
    expect(selectedProjectSourceLibraryDeleteDisabledReason([youtubeVideo, youtubeVideo])).toBe(
      "Select one YouTube video source",
    );
  });

  it("allows one YouTube video and rejects other source types", () => {
    expect(selectedProjectSourceLibraryDeleteDisabledReason([youtubeVideo])).toBeNull();
    expect(selectedProjectSourceLibraryDeleteDisabledReason([youtubePlaylist])).toBe(
      "Only YouTube videos can be deleted from Library here",
    );
    expect(selectedProjectSourceLibraryDeleteDisabledReason([telegram])).toBe(
      "Only YouTube videos can be deleted from Library here",
    );
  });
});

describe("projectSourceLibraryDeleteStatus", () => {
  it("formats successful deletion", () => {
    expect(
      projectSourceLibraryDeleteStatus({
        status: "deleted",
        blocking_projects: [],
        remaining_blocking_project_count: 0,
      }),
    ).toBe("Source deleted from project and Library.");
  });

  it("formats blocked projects with an extra count only when present", () => {
    expect(
      projectSourceLibraryDeleteStatus({
        status: "blocked_by_other_projects",
        blocking_projects: [
          { project_id: 1, title: "Alpha", archived: false },
          { project_id: 2, title: "Beta", archived: true },
          { project_id: 3, title: "Gamma", archived: false },
        ],
        remaining_blocking_project_count: 2,
      }),
    ).toBe(
      "Cannot delete from Library: source is used by other projects: Alpha, Beta, Gamma, and 2 more.",
    );
    expect(
      projectSourceLibraryDeleteStatus({
        status: "blocked_by_other_projects",
        blocking_projects: [{ project_id: 1, title: "Alpha", archived: false }],
        remaining_blocking_project_count: 0,
      }),
    ).toBe("Cannot delete from Library: source is used by other projects: Alpha.");
  });
});
```

- [ ] **Step 3: Run the new frontend helper tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/api/projects.test.ts src/lib/ui/research-projects-model.test.ts
```

Expected: FAIL because the functions and types are missing.

- [ ] **Step 4: Add TypeScript API types in `src/lib/types/projects.ts`**

Add after `ProjectSourcesInput`:

```ts
export interface DeleteProjectYoutubeVideoSourceInput {
  projectId: number;
  sourceId: number;
}

export type DeleteProjectYoutubeVideoSourceStatus = "deleted" | "blocked_by_other_projects";

export interface BlockingProjectReference {
  project_id: number;
  title: string;
  archived: boolean;
}

export interface DeleteProjectYoutubeVideoSourceOutcome {
  status: DeleteProjectYoutubeVideoSourceStatus;
  blocking_projects: BlockingProjectReference[];
  remaining_blocking_project_count: number;
}
```

- [ ] **Step 5: Add API wrapper in `src/lib/api/projects.ts`**

Extend imports:

```ts
DeleteProjectYoutubeVideoSourceInput,
DeleteProjectYoutubeVideoSourceOutcome,
```

Add after `removeProjectSources`:

```ts
export function deleteProjectYoutubeVideoSourceFromLibrary(
  input: DeleteProjectYoutubeVideoSourceInput,
) {
  return invoke<DeleteProjectYoutubeVideoSourceOutcome>(
    "delete_project_youtube_video_source_from_library",
    { ...input },
  );
}
```

- [ ] **Step 6: Add model helpers in `src/lib/ui/research-projects-model.ts`**

Add import:

```ts
import type { DeleteProjectYoutubeVideoSourceOutcome } from "$lib/types/projects";
```

Add near `selectedProjectSourcesSyncDisabledReason`:

```ts
export const PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM =
  "Delete this YouTube video from the project and Library? The app will cancel the deletion if another project still uses it. This will remove its transcript, comments, and stored materials.";

export function selectedProjectSourceLibraryDeleteDisabledReason(
  rows: Pick<ProjectSourceLinkView, "provider" | "subtype">[],
) {
  if (rows.length !== 1) return "Select one YouTube video source";
  const [row] = rows;
  if (row.provider !== "youtube" || row.subtype !== "video") {
    return "Only YouTube videos can be deleted from Library here";
  }
  return null;
}

export function projectSourceLibraryDeleteStatus(
  outcome: DeleteProjectYoutubeVideoSourceOutcome,
) {
  if (outcome.status === "deleted") return "Source deleted from project and Library.";
  const names = outcome.blocking_projects.map((project) => project.title).join(", ");
  const suffix =
    outcome.remaining_blocking_project_count > 0
      ? `, and ${outcome.remaining_blocking_project_count} more`
      : "";
  return `Cannot delete from Library: source is used by other projects: ${names}${suffix}.`;
}
```

- [ ] **Step 7: Add workflow tests in `src/lib/ui/research-projects-workflow.test.ts`**

Extend the model import at the top of the test file:

```ts
import {
  PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM,
  buildProjectSourceLinksView,
} from "./research-projects-model";
```

Add mocked dependencies to `createDeps`:

```ts
deleteProjectYoutubeVideoSourceFromLibrary: vi.fn(),
confirm: vi.fn(() => true),
```

Add this helper near `createDeps`:

```ts
function createStateWithSelectedYoutubeVideoSource() {
  const state = createInitialState();
  state.selectedProjectId = "project:1";
  state.projectsRaw = [project()];
  state.projectSources = [projectSource()];
  state.projectSourceLinks = buildProjectSourceLinksView("project:1", state.projectSources);
  return state;
}
```

Add tests:

```ts
it("does not call delete command when confirmation is cancelled", async () => {
  const state = createStateWithSelectedYoutubeVideoSource();
  const deps = createDeps(state);
  deps.confirm.mockReturnValueOnce(false);
  const workflow = createResearchProjectsWorkflow(deps);

  await workflow.deleteProjectYoutubeVideoSourceFromLibrary(10);

  expect(deps.confirm).toHaveBeenCalledWith(PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM);
  expect(deps.deleteProjectYoutubeVideoSourceFromLibrary).not.toHaveBeenCalled();
});

it("deletes one project youtube video source from library and refreshes workspace", async () => {
  const state = createStateWithSelectedYoutubeVideoSource();
  const deps = createDeps(state);
  deps.deleteProjectYoutubeVideoSourceFromLibrary.mockResolvedValueOnce({
    status: "deleted",
    blocking_projects: [],
    remaining_blocking_project_count: 0,
  });
  deps.listProjects.mockResolvedValue([project()]);
  deps.listProjectSources.mockResolvedValue([projectSource()]);
  deps.listLibraryCatalog.mockResolvedValue({ sources: [], filter_counts: [] });
  deps.listProjectRuns.mockResolvedValue([]);
  deps.listPromptTemplates.mockResolvedValue([]);
  deps.listSourceJobs.mockResolvedValue([]);
  const workflow = createResearchProjectsWorkflow(deps);

  await workflow.deleteProjectYoutubeVideoSourceFromLibrary(10);

  expect(deps.confirm).toHaveBeenCalledWith(PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM);
  expect(deps.deleteProjectYoutubeVideoSourceFromLibrary).toHaveBeenCalledWith({
    projectId: 1,
    sourceId: 10,
  });
  expect(state.status).toBe("Source deleted from project and Library.");
  expect(deps.listLibraryCatalog).toHaveBeenCalledTimes(1);
  expect(deps.listProjectSources).toHaveBeenCalled();
});

it("keeps current project membership when backend reports blocking projects", async () => {
  const state = createStateWithSelectedYoutubeVideoSource();
  const deps = createDeps(state);
  deps.deleteProjectYoutubeVideoSourceFromLibrary.mockResolvedValueOnce({
    status: "blocked_by_other_projects",
    blocking_projects: [
      { project_id: 2, title: "Alpha", archived: false },
      { project_id: 3, title: "Beta", archived: true },
      { project_id: 4, title: "Gamma", archived: false },
    ],
    remaining_blocking_project_count: 1,
  });
  const workflow = createResearchProjectsWorkflow(deps);

  await workflow.deleteProjectYoutubeVideoSourceFromLibrary(10);

  expect(state.status).toBe(
    "Cannot delete from Library: source is used by other projects: Alpha, Beta, Gamma, and 1 more.",
  );
  expect(deps.listLibraryCatalog).toHaveBeenCalledTimes(0);
});
```

- [ ] **Step 8: Implement workflow helper in `src/lib/ui/research-projects-workflow.ts`**

Extend imports:

```ts
PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM,
projectSourceLibraryDeleteStatus,
selectedProjectSourceLibraryDeleteDisabledReason,
```

Extend project type imports:

```ts
DeleteProjectYoutubeVideoSourceInput,
DeleteProjectYoutubeVideoSourceOutcome,
```

Extend `ResearchProjectsWorkflowDeps`:

```ts
deleteProjectYoutubeVideoSourceFromLibrary(
  input: DeleteProjectYoutubeVideoSourceInput,
): Promise<DeleteProjectYoutubeVideoSourceOutcome>;
confirm(message: string): boolean;
```

Add method before `setStatus`:

```ts
async function deleteProjectYoutubeVideoSourceFromLibrary(sourceId: number) {
  const state = deps.getState();
  const projectId = projectIdFromViewId(state.selectedProjectId);
  if (!projectId) {
    deps.patch({ status: "Select a project" });
    return;
  }
  const row = state.projectSourceLinks.find((source) => source.sourceNumericId === sourceId);
  const disabledReason = selectedProjectSourceLibraryDeleteDisabledReason(row ? [row] : []);
  if (disabledReason) {
    deps.patch({ status: disabledReason });
    return;
  }
  if (!deps.confirm(PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM)) {
    return;
  }

  deps.patch({ saving: true });
  try {
    const outcome = await deps.deleteProjectYoutubeVideoSourceFromLibrary({ projectId, sourceId });
    deps.patch({ status: projectSourceLibraryDeleteStatus(outcome) });
    if (outcome.status === "deleted") {
      await loadWorkspace();
    }
  } catch (error) {
    deps.patch({ status: deps.formatError("deleting project source from Library", error) });
  } finally {
    deps.patch({ saving: false });
  }
}
```

Return the method:

```ts
deleteProjectYoutubeVideoSourceFromLibrary,
```

- [ ] **Step 9: Run Task 3 tests**

Run:

```powershell
npm.cmd run test -- src/lib/api/projects.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 10: Commit Task 3**

Run:

```powershell
git add src/lib/types/projects.ts src/lib/api/projects.ts src/lib/api/projects.test.ts src/lib/ui/research-projects-model.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.ts src/lib/ui/research-projects-workflow.test.ts
git commit -m "feat: add project source library delete frontend helpers"
```

---

### Task 4: Current Projects Screen Toolbar Integration

**Files:**
- Modify: `src/lib/components/research-projects/SourcesTab.svelte`
- Modify: `src/lib/components/research-projects/ProjectWorkspace.svelte`
- Modify: `src/lib/components/research-projects/ProjectsShell.svelte`
- Modify: `src/routes/projects/+page.svelte`
- Modify: `src/routes/projects/list/+page.svelte`
- Modify: `src/lib/research-projects-route-contract.test.ts`

**Interfaces:**
- Consumes: workflow method `deleteProjectYoutubeVideoSourceFromLibrary(sourceId: number)`
- Consumes: model helper `selectedProjectSourceLibraryDeleteDisabledReason`

- [ ] **Step 1: Add failing route contract tests**

In `src/lib/research-projects-route-contract.test.ts`, add tests that assert both current routes import and pass the new command:

```ts
it("wires project source Library delete through the main projects route", () => {
  expect(projectsRouteSource).toContain("deleteProjectYoutubeVideoSourceFromLibrary");
  expect(projectsRouteSource).toContain(
    "onDeleteProjectSourceFromLibrary={workflow.deleteProjectYoutubeVideoSourceFromLibrary}",
  );
});

it("wires project source Library delete through the list projects route", () => {
  expect(projectsListRouteSource).toContain("deleteProjectYoutubeVideoSourceFromLibrary");
  expect(projectsListRouteSource).toContain(
    "onDeleteProjectSourceFromLibrary={workflow.deleteProjectYoutubeVideoSourceFromLibrary}",
  );
});
```

Add a component contract assertion for `SourcesTab.svelte`:

```ts
it("keeps Remove membership-only and adds a separate Delete from Library action", () => {
  expect(sourcesTabSource).toContain("Delete from Library");
  expect(sourcesTabSource).toContain("onDeleteProjectSourceFromLibrary");
  expect(sourcesTabSource).toContain("onRemoveSource");
});
```

- [ ] **Step 2: Run the route contract tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts
```

Expected: FAIL because the route and component wiring is missing.

- [ ] **Step 3: Add `SourcesTab.svelte` prop and derived disabled state**

In `SourcesTab.svelte`, import the helper:

```ts
selectedProjectSourceLibraryDeleteDisabledReason,
```

Add prop:

```ts
onDeleteProjectSourceFromLibrary = async (_sourceId: number) => {},
```

Add prop type:

```ts
onDeleteProjectSourceFromLibrary?: (sourceId: number) => Promise<void> | void;
```

Add derived state near `syncDisabledReason`:

```ts
let libraryDeleteDisabledReason = $derived(
  selectedProjectSourceLibraryDeleteDisabledReason(selectedRows),
);
```

Add handler:

```ts
async function handleDeleteSelectedSourceFromLibrary() {
  if (libraryDeleteDisabledReason || selectedRows.length !== 1) return;
  const [source] = selectedRows;
  await onDeleteProjectSourceFromLibrary(source.sourceNumericId);
  onSelectedSourceIdsChange([]);
}
```

- [ ] **Step 4: Add the `Delete from Library` button in the selected-source toolbar**

Place this `ExtractumButton` near the existing `Remove` button:

```svelte
<ExtractumButton
  variant="destructive"
  disabled={saving || libraryDeleteDisabledReason !== null}
  title={libraryDeleteDisabledReason ?? ""}
  aria-label="Delete selected YouTube video from Library"
  onclick={() => void handleDeleteSelectedSourceFromLibrary()}
>
  <Trash2 size={12} aria-hidden="true" />
  Delete from Library
</ExtractumButton>
```

Use the existing toolbar button element style in `SourcesTab.svelte`; the required props are `disabled`, `title`, `onclick`, icon, and the visible label `Delete from Library`.

- [ ] **Step 5: Thread the callback through workspace components**

In `ProjectWorkspace.svelte`, add prop:

```ts
onDeleteProjectSourceFromLibrary = async (_sourceId: number) => {},
```

Pass it to `SourcesTab`:

```svelte
onDeleteProjectSourceFromLibrary={onDeleteProjectSourceFromLibrary}
```

In `ProjectsShell.svelte`, add prop:

```ts
onDeleteProjectSourceFromLibrary = async (_sourceId: number) => {},
```

Pass it to `ProjectWorkspace`:

```svelte
onDeleteProjectSourceFromLibrary={onDeleteProjectSourceFromLibrary}
```

- [ ] **Step 6: Wire route dependencies**

In both `src/routes/projects/+page.svelte` and `src/routes/projects/list/+page.svelte`, import:

```ts
deleteProjectYoutubeVideoSourceFromLibrary,
```

Add it to `createResearchProjectsWorkflow` dependencies:

```ts
deleteProjectYoutubeVideoSourceFromLibrary,
confirm: (message) => window.confirm(message),
```

Pass the callback to `ProjectsShell`:

```svelte
onDeleteProjectSourceFromLibrary={workflow.deleteProjectYoutubeVideoSourceFromLibrary}
```

- [ ] **Step 7: Run Task 4 tests**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts
```

Expected: PASS.

- [ ] **Step 8: Commit Task 4**

Run:

```powershell
git add src/lib/components/research-projects/SourcesTab.svelte src/lib/components/research-projects/ProjectWorkspace.svelte src/lib/components/research-projects/ProjectsShell.svelte src/routes/projects/+page.svelte src/routes/projects/list/+page.svelte src/lib/research-projects-route-contract.test.ts
git commit -m "feat: add library delete action to project sources toolbar"
```

---

### Task 5: `/projects/next` Bulk Bar Integration

**Files:**
- Modify: `src/lib/components/research-projects/SourcesBulkBar.svelte`
- Modify: `src/lib/components/research-projects/SourcesBulkBar.test.ts`
- Modify: `src/routes/projects/next/+page.svelte`
- Modify: `src/lib/research-projects-route-contract.test.ts`

**Interfaces:**
- Consumes: `selectedProjectSourceLibraryDeleteDisabledReason`
- Consumes: `projectSourceLibraryDeleteStatus`
- Consumes: API wrapper `deleteProjectYoutubeVideoSourceFromLibrary`

- [ ] **Step 1: Add failing `SourcesBulkBar` tests**

In `SourcesBulkBar.test.ts`, add:

```ts
it("shows Delete from Library as a separate action and respects disabled reason", () => {
  render(SourcesBulkBar, {
    props: {
      count: 2,
      libraryDeleteDisabled: true,
      libraryDeleteTitle: "Select one YouTube video source",
    },
  });

  const button = screen.getByRole("button", { name: "Delete from Library" }) as HTMLButtonElement;
  expect(button.disabled).toBe(true);
  expect(button.getAttribute("title")).toBe("Select one YouTube video source");
});

it("confirms before deleting from Library and deletes only on confirm", async () => {
  const onDeleteFromLibrary = vi.fn();
  render(SourcesBulkBar, {
    props: {
      count: 1,
      libraryDeleteDisabled: false,
      onDeleteFromLibrary,
    },
  });

  await fireEvent.click(screen.getByRole("button", { name: "Delete from Library" }));
  expect(onDeleteFromLibrary).not.toHaveBeenCalled();

  await fireEvent.click(screen.getByRole("button", { name: "Delete from Library permanently" }));
  expect(onDeleteFromLibrary).toHaveBeenCalledOnce();
});

it("does not delete from Library when the confirmation is cancelled", async () => {
  const onDeleteFromLibrary = vi.fn();
  render(SourcesBulkBar, {
    props: {
      count: 1,
      libraryDeleteDisabled: false,
      onDeleteFromLibrary,
    },
  });

  await fireEvent.click(screen.getByRole("button", { name: "Delete from Library" }));
  await fireEvent.click(screen.getByRole("button", { name: "Cancel Library deletion" }));
  expect(onDeleteFromLibrary).not.toHaveBeenCalled();
});
```

- [ ] **Step 2: Run `SourcesBulkBar` tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/components/research-projects/SourcesBulkBar.test.ts
```

Expected: FAIL because the new props and dialog are missing.

- [ ] **Step 3: Add `SourcesBulkBar.svelte` props and dialog state**

Add import:

```ts
import { PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM } from "$lib/ui/research-projects-model";
```

Extend props:

```ts
libraryDeleteDisabled = true,
libraryDeleteTitle = "",
onDeleteFromLibrary = () => {},
```

Extend prop types:

```ts
libraryDeleteDisabled?: boolean;
libraryDeleteTitle?: string;
onDeleteFromLibrary?: () => void;
```

Add state:

```ts
let libraryDeleteConfirmOpen = $state(false);

function confirmDeleteFromLibrary() {
  libraryDeleteConfirmOpen = false;
  onDeleteFromLibrary();
}
```

- [ ] **Step 4: Add the `/projects/next` bulk-bar button and dialog**

Add button before the existing destructive `Remove` button:

```svelte
<ExtractumButton
  variant="destructive"
  disabled={libraryDeleteDisabled}
  title={libraryDeleteDisabled ? libraryDeleteTitle : ""}
  onclick={() => (libraryDeleteConfirmOpen = true)}
>
  Delete from Library
</ExtractumButton>
```

Add separate dialog after the existing remove dialog:

```svelte
<ExtractumDialog bind:open={libraryDeleteConfirmOpen} title="Delete from Library">
  <div class="sources-bulk-bar__confirm">
    <p>
      {PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM}
    </p>
    <footer>
      <ExtractumButton
        type="button"
        variant="outline"
        onclick={() => (libraryDeleteConfirmOpen = false)}
      >
        Cancel Library deletion
      </ExtractumButton>
      <ExtractumButton type="button" variant="destructive" onclick={confirmDeleteFromLibrary}>
        Delete from Library permanently
      </ExtractumButton>
    </footer>
  </div>
</ExtractumDialog>
```

- [ ] **Step 5: Add `/projects/next` derived state and delete handler**

In `src/routes/projects/next/+page.svelte`, import:

```ts
deleteProjectYoutubeVideoSourceFromLibrary,
```

from `$lib/api/projects`, and import helpers:

```ts
projectSourceLibraryDeleteStatus,
selectedProjectSourceLibraryDeleteDisabledReason,
```

from `$lib/ui/research-projects-model`.

Add derived selected rows:

```ts
let selectedProjectSourceRows = $derived(
  sources.filter((source) => selectedSourceIds.includes(String(source.source_id))),
);
let bulkLibraryDeleteDisabledReason = $derived(
  selectedProjectSourceLibraryDeleteDisabledReason(
    selectedProjectSourceRows.map((source) => ({
      provider: source.provider,
      subtype: source.source_subtype,
    })),
  ),
);
```

Add handler near `deleteSelectedSources`:

```ts
async function deleteSelectedSourceFromLibrary() {
  if (
    selectedProjectId === null ||
    bulkLibraryDeleteDisabledReason !== null ||
    selectedProjectSourceRows.length !== 1
  ) {
    return;
  }
  const sourceId = selectedProjectSourceRows[0].source_id;
  railState = { ...railState, saving: true, status: "" };
  try {
    const outcome = await deleteProjectYoutubeVideoSourceFromLibrary({
      projectId: selectedProjectId,
      sourceId,
    });
    railState = { ...railState, status: projectSourceLibraryDeleteStatus(outcome) };
    if (outcome.status === "deleted") {
      selectedSourceIds = [];
      activeSourceId = activeSourceId === String(sourceId) ? null : activeSourceId;
      const catalog = await listLibraryCatalog();
      libraryCatalogRecords = catalog.sources;
      sources = await listProjectSources(selectedProjectId);
      await workflow.reload();
    }
  } catch (error) {
    railState = {
      ...railState,
      status: formatAppError("deleting project source from Library", error),
    };
  } finally {
    railState = { ...railState, saving: false };
  }
}
```

- [ ] **Step 6: Pass new props to `SourcesBulkBar` through `/projects/next` shell config**

Extend `bulkBar={...}`:

```ts
libraryDeleteDisabled: railState.saving || bulkLibraryDeleteDisabledReason !== null,
libraryDeleteTitle: bulkLibraryDeleteDisabledReason ?? "",
onDeleteFromLibrary: deleteSelectedSourceFromLibrary,
```

- [ ] **Step 7: Add `/projects/next` route contract assertion**

In `src/lib/research-projects-route-contract.test.ts`, add:

```ts
it("wires Delete from Library in the next projects bulk bar", () => {
  expect(projectsNextRouteSource).toContain("deleteProjectYoutubeVideoSourceFromLibrary");
  expect(projectsNextRouteSource).toContain("onDeleteFromLibrary: deleteSelectedSourceFromLibrary");
  expect(projectsNextRouteSource).toContain("bulkLibraryDeleteDisabledReason");
});
```

- [ ] **Step 8: Run Task 5 tests**

Run:

```powershell
npm.cmd run test -- src/lib/components/research-projects/SourcesBulkBar.test.ts src/lib/research-projects-route-contract.test.ts
```

Expected: PASS.

- [ ] **Step 9: Commit Task 5**

Run:

```powershell
git add src/lib/components/research-projects/SourcesBulkBar.svelte src/lib/components/research-projects/SourcesBulkBar.test.ts src/routes/projects/next/+page.svelte src/lib/research-projects-route-contract.test.ts
git commit -m "feat: add library delete action to next project sources"
```

---

### Task 6: Final Verification And Review

**Files:**
- No new source files required.
- May modify files touched in Tasks 1-5 if verification finds defects.

**Interfaces:**
- Consumes all previous task outputs.
- Produces a verified branch ready for user review.

- [ ] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/api/projects.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/components/research-projects/SourcesBulkBar.test.ts src/lib/research-projects-route-contract.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run frontend type and Svelte checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 3: Run focused Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml tx::tests::
cargo test --manifest-path src-tauri/Cargo.toml sources::store::tests::delete_source
cargo test --manifest-path src-tauri/Cargo.toml projects::tests::project_scoped_delete
```

Expected: PASS.

- [ ] **Step 4: Run Rust compile check**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 5: Inspect worktree before final commit**

Run:

```powershell
git status --short
```

Expected: only intended files from Tasks 1-5 are modified, plus no staged `.claude/settings.local.json`.

- [ ] **Step 6: Commit verification fixes if any files changed**

If verification required source fixes, run:

```powershell
git add <fixed-files>
git commit -m "fix: polish project source library deletion"
```

If no files changed after Task 5, record this step as complete without a commit.

- [ ] **Step 7: Self-review the final branch**

Check these items manually:

- `Delete from Library` is separate from `Remove`.
- Both current projects routes and `/projects/next` expose the action.
- Disabled state requires exactly one YouTube video source.
- Confirmation appears before backend mutation.
- Backend blocks active and archived other projects without deleting current membership.
- Backend returns at most three blocking projects.
- `and N more` appears only when N is greater than zero.
- Standalone `delete_source` still blocks project-linked sources.
- Cascade cleanup is proven by tests, not implemented as duplicate manual deletes.
- Value registry includes `deleted` and `blocked_by_other_projects`.

- [ ] **Step 8: Report verification evidence**

Final response must include exact commands run and whether each passed. Mention any command that could not be run.

---

## Plan Self-Review

**Spec coverage:** Covered the new project-scoped Tauri command, structured outcome, FK setup before `BEGIN IMMEDIATE`, delete lock reuse, validation errors, active/archived blocking projects, payload cap, playlist `SET NULL`, cascade cleanup plus schema-wide FK guard, standalone delete semantics, API wrapper, workflow helper, model helper, both UI locations, shared confirmation copy, success/blocked statuses, and final validation commands.

**Placeholder scan:** The plan uses concrete function names, copy, file paths, commands, and code snippets. It avoids deferred implementation markers.

**Type consistency:** Rust DTO names map to TypeScript types and the API wrapper. Frontend helper names match workflow and UI calls: `selectedProjectSourceLibraryDeleteDisabledReason`, `projectSourceLibraryDeleteStatus`, and `deleteProjectYoutubeVideoSourceFromLibrary`.
