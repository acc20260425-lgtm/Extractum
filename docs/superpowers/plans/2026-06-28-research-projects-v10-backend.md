# Research Projects v10 Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the approved Research Projects v10 backend contract so the frontend can consume project summaries, source rows, data ranges, and pin/archive state without client-side aggregation hacks.

**Architecture:** Keep the existing thin Tauri/Rust backend shape, but split `projects.rs` into a focused `projects/` module before adding read-model logic. Store pin/archive state in SQLite, reuse library catalog status calculation for project sources, derive project summaries in SQL-backed Rust read models, and mirror analysis corpus filtering for the toolbar data range.

**Tech Stack:** Rust, Tauri commands, SQLx SQLite, SvelteKit TypeScript invoke wrappers, Vitest, `cargo test`, `cargo check`, `npm.cmd run check`.

## Global Constraints

- Fat frontend, thin backend: low-level integration and SQLite aggregation stay in Rust; user-flow orchestration stays in Svelte.
- Prefer small explicit Tauri commands over broad generic commands.
- Use `npm.cmd`, not `npm`, in all Windows validation commands.
- SQLite migrations are additive; do not delete, rename, or rewrite existing migration files.
- Register new migrations in `src-tauri/src/migrations.rs`; adding a `.sql` file alone is not enough.
- `dataRange` must mirror the actual analysis corpus: `analysis_documents` with the same source/document-kind filters, plus Telegram migrated-history `items` only when enabled.
- `get_project_data_range` filters by resolved `source_ids` from `resolve_analysis_sources`, not by `project_sources.source_id`.
- `get_project_data_range` returns `{ from: None, to: None }` for valid but non-runnable project scopes, including mixed-provider projects and unmaterialized YouTube playlists. It must still reject `include_migrated_history=true` for any project with non-Telegram sources before returning a null range.
- `ProjectSummary.material_count` and `ProjectSourceRecord.item_count` count collected local-copy materials with the same rule: direct sources use their own `source_id`; YouTube playlist sources expand to linked video `source_id`s. A project with one playlist source and two linked video items must show `material_count = 2`, and that playlist source row must show `item_count = 2`.
- `include_migrated_history=true` is valid only for Telegram and must match `start_project_analysis` validation.
- Source status wire values are exactly `active`, `syncing`, `error`, `unavailable`; do not introduce `idle`.
- `ProjectStatus` wire values are exactly `ready`, `running`, `needs_attention`, `empty` in snake_case.
- `.playwright-mcp/` is generated and must not be staged.

---

## File Structure

- Move: `src-tauri/src/projects.rs` -> `src-tauri/src/projects/mod.rs`
- Create: `src-tauri/src/projects/read_model.rs`
- Create: `src-tauri/src/projects/data_range.rs`
- Create: `src-tauri/migrations/0012_projects_redesign.sql`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/library_sources/models.rs`
- Modify: `src-tauri/src/library_sources/mod.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/types/projects.ts`
- Modify: `src/lib/api/projects.ts`
- Modify: `src/lib/api/projects.test.ts`
- Modify: `docs/value-registry.md`

## Interfaces

The implementation produces these stable Rust/TS contracts:

```rust
#[derive(Clone, Copy, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LibraryCatalogStatus {
    Active,
    Syncing,
    Error,
    Unavailable,
}

#[derive(Clone, Copy, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Ready,
    Running,
    NeedsAttention,
    Empty,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub source_count: i64,
    pub material_count: i64,
    pub status: ProjectStatus,
    pub last_run_at: Option<i64>,
    pub pinned: bool,
    pub archived: bool,
    pub updated_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ProjectDataRange {
    pub from: Option<i64>,
    pub to: Option<i64>,
}
```

```ts
export type ProjectStatus = "ready" | "running" | "needs_attention" | "empty";

export interface ProjectSummary {
  id: number;
  name: string;
  description: string | null;
  source_count: number;
  material_count: number;
  status: ProjectStatus;
  last_run_at: number | null;
  pinned: boolean;
  archived: boolean;
  updated_at: number;
}

export interface ProjectDataRange {
  from: number | null;
  to: number | null;
}
```

### Task 1: Projects Module Split And Migration 0012

**Files:**
- Move: `src-tauri/src/projects.rs` -> `src-tauri/src/projects/mod.rs`
- Create: `src-tauri/migrations/0012_projects_redesign.sql`
- Modify: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/migrations.rs`

**Interfaces:**
- Consumes: existing `mod projects;` declaration in `src-tauri/src/lib.rs`
- Produces: a `projects` module directory ready for `read_model.rs` and `data_range.rs`, plus schema columns `projects.pinned` and `projects.archived_at`

- [ ] **Step 1: Move the current projects module without behavior changes**

Run:

```powershell
git mv src-tauri/src/projects.rs src-tauri/src/projects/mod.rs
```

No Rust import changes are needed for `src-tauri/src/lib.rs`; `mod projects;` resolves to `projects/mod.rs`.

- [ ] **Step 2: Run the existing projects smoke tests after the move**

Run:

```powershell
cargo test projects::tests::create_project_trims_and_rejects_duplicate_names_case_insensitively projects::tests::add_project_sources_is_idempotent_and_lists_ui_ready_rows
```

Expected: both selected tests pass. If module paths changed unexpectedly, fix the path before continuing.

- [ ] **Step 3: Add the migration SQL**

Create `src-tauri/migrations/0012_projects_redesign.sql`:

```sql
ALTER TABLE projects ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
ALTER TABLE projects ADD COLUMN archived_at INTEGER;

CREATE INDEX IF NOT EXISTS idx_projects_pinned_archived
    ON projects(pinned DESC, archived_at, updated_at DESC);
```

- [ ] **Step 4: Register migration constants and function**

In `src-tauri/src/migrations.rs`, add after the `PROMPT_PACK_STAGE_BROWSER_PROVENANCE_*` constants:

```rust
const PROJECTS_REDESIGN_VERSION: i64 = 12;
const PROJECTS_REDESIGN_DESCRIPTION: &str = "projects redesign schema";
const PROJECTS_REDESIGN_SQL: &str = include_str!("../migrations/0012_projects_redesign.sql");
```

Add after `prompt_pack_stage_browser_provenance_migration()`:

```rust
fn projects_redesign_migration() -> Migration {
    Migration {
        version: PROJECTS_REDESIGN_VERSION,
        description: PROJECTS_REDESIGN_DESCRIPTION,
        sql: PROJECTS_REDESIGN_SQL,
        kind: MigrationKind::Up,
    }
}
```

Add `projects_redesign_migration()` to `build_migrations()` immediately after `prompt_pack_stage_browser_provenance_migration()` and before `migrations.extend(apalis_sqlite_migrations())`.

- [ ] **Step 5: Update migration version expectations**

In `src-tauri/src/migrations.rs`, update the test constant:

```rust
const EXPECTED_BUILD_MIGRATION_VERSIONS: [i64; 20] = [
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    20220530084123,
    20250313213411,
    20251013233016,
    20251017150712,
    20251017162501,
    20251018162501,
    20251018164941,
    20260506101935,
];
```

- [ ] **Step 6: Add migration coverage for columns, index, and defaults**

Add this test inside `src-tauri/src/migrations.rs` tests:

```rust
#[tokio::test]
async fn fresh_schema_includes_projects_redesign_columns_index_and_defaults() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    for column in ["pinned", "archived_at"] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = ?",
        )
        .bind(column)
        .fetch_one(&pool)
        .await
        .expect("check projects column");
        assert_eq!(exists, 1, "missing projects.{column}");
    }

    let index_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_projects_pinned_archived'",
    )
    .fetch_one(&pool)
    .await
    .expect("check projects redesign index");
    assert_eq!(index_exists, 1);

    sqlx::query(
        "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (91, 'Default flags', NULL, 10, 10)",
    )
    .execute(&pool)
    .await
    .expect("insert project");

    let row: (i64, Option<i64>) =
        sqlx::query_as("SELECT pinned, archived_at FROM projects WHERE id = 91")
            .fetch_one(&pool)
            .await
            .expect("load project flags");
    assert_eq!(row, (0, None));
}
```

- [ ] **Step 7: Verify migration task**

Run:

```powershell
cargo test migrations::tests::fresh_schema_includes_projects_redesign_columns_index_and_defaults migrations::tests::build_migrations_starts_at_current_schema_baseline
cargo test projects::tests::create_project_trims_and_rejects_duplicate_names_case_insensitively
```

Expected: all selected tests pass.

- [ ] **Step 8: Commit Task 1**

```powershell
git add src-tauri/src/projects/mod.rs src-tauri/migrations/0012_projects_redesign.sql src-tauri/src/migrations.rs
git commit -m "feat: add projects redesign migration"
```

### Task 2: Pin And Archive Mutations

**Files:**
- Modify: `src-tauri/src/projects/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/projects/mod.rs`

**Interfaces:**
- Consumes: `projects.pinned INTEGER NOT NULL DEFAULT 0`, `projects.archived_at INTEGER`
- Produces:
  - `set_project_pinned(handle, project_id, pinned) -> AppResult<()>`
  - `set_project_archived(handle, project_id, archived) -> AppResult<()>`

- [ ] **Step 1: Add failing tests for the two mutations**

Add inside `src-tauri/src/projects/mod.rs` tests:

```rust
#[tokio::test]
async fn set_project_pinned_toggles_flag_and_updates_timestamp() {
    let pool = pool().await;
    let project = create_project_in_pool(&pool, "Pinned", None)
        .await
        .expect("create project");
    let before = project.updated_at;

    set_project_pinned_in_pool(&pool, project.id, true)
        .await
        .expect("pin project");
    let row: (i64, i64) = sqlx::query_as("SELECT pinned, updated_at FROM projects WHERE id = ?")
        .bind(project.id)
        .fetch_one(&pool)
        .await
        .expect("load pinned project");
    assert_eq!(row.0, 1);
    assert!(row.1 >= before);

    set_project_pinned_in_pool(&pool, project.id, false)
        .await
        .expect("unpin project");
    let pinned: i64 = sqlx::query_scalar("SELECT pinned FROM projects WHERE id = ?")
        .bind(project.id)
        .fetch_one(&pool)
        .await
        .expect("load unpinned project");
    assert_eq!(pinned, 0);
}

#[tokio::test]
async fn set_project_archived_toggles_timestamp_and_rejects_missing_project() {
    let pool = pool().await;
    let project = create_project_in_pool(&pool, "Archive", None)
        .await
        .expect("create project");

    set_project_archived_in_pool(&pool, project.id, true)
        .await
        .expect("archive project");
    let archived_at: Option<i64> =
        sqlx::query_scalar("SELECT archived_at FROM projects WHERE id = ?")
            .bind(project.id)
            .fetch_one(&pool)
            .await
            .expect("load archived project");
    assert!(archived_at.is_some());

    set_project_archived_in_pool(&pool, project.id, false)
        .await
        .expect("restore project");
    let archived_at: Option<i64> =
        sqlx::query_scalar("SELECT archived_at FROM projects WHERE id = ?")
            .bind(project.id)
            .fetch_one(&pool)
            .await
            .expect("load restored project");
    assert_eq!(archived_at, None);

    let missing = set_project_archived_in_pool(&pool, 404_404, true)
        .await
        .expect_err("missing project rejected");
    assert_eq!(missing.kind, crate::error::AppErrorKind::NotFound);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test projects::tests::set_project_pinned_toggles_flag_and_updates_timestamp projects::tests::set_project_archived_toggles_timestamp_and_rejects_missing_project
```

Expected: compile failure because `set_project_pinned_in_pool` and `set_project_archived_in_pool` are not defined.

- [ ] **Step 3: Add in-pool mutation helpers**

Add to `src-tauri/src/projects/mod.rs` near other project mutation helpers:

```rust
pub(crate) async fn set_project_pinned_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    pinned: bool,
) -> AppResult<()> {
    let now = crate::time::now_secs();
    let result = sqlx::query(
        r#"
        UPDATE projects
        SET pinned = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(if pinned { 1_i64 } else { 0_i64 })
    .bind(now)
    .bind(project_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Project {project_id} not found")));
    }
    Ok(())
}

pub(crate) async fn set_project_archived_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    archived: bool,
) -> AppResult<()> {
    let now = crate::time::now_secs();
    let archived_at = archived.then_some(now);
    let result = sqlx::query(
        r#"
        UPDATE projects
        SET archived_at = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(archived_at)
    .bind(now)
    .bind(project_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Project {project_id} not found")));
    }
    Ok(())
}
```

- [ ] **Step 4: Add Tauri commands**

Add to `src-tauri/src/projects/mod.rs` near the existing command wrappers:

```rust
#[tauri::command]
pub async fn set_project_pinned(
    handle: AppHandle,
    project_id: i64,
    pinned: bool,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    set_project_pinned_in_pool(&pool, project_id, pinned).await
}

#[tauri::command]
pub async fn set_project_archived(
    handle: AppHandle,
    project_id: i64,
    archived: bool,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    set_project_archived_in_pool(&pool, project_id, archived).await
}
```

In `src-tauri/src/lib.rs`, add both commands to the `use projects::{...}` list and to `tauri::generate_handler![...]` after `delete_project`.

- [ ] **Step 5: Verify mutation task**

Run:

```powershell
cargo test projects::tests::set_project_pinned_toggles_flag_and_updates_timestamp projects::tests::set_project_archived_toggles_timestamp_and_rejects_missing_project
cargo check
```

Expected: selected tests and `cargo check` pass.

- [ ] **Step 6: Commit Task 2**

```powershell
git add src-tauri/src/projects/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add project pin and archive commands"
```

### Task 3: Reuse Library Catalog Status In Project Sources

**Files:**
- Modify: `src-tauri/src/library_sources/models.rs`
- Modify: `src-tauri/src/library_sources/mod.rs`
- Modify: `src-tauri/src/projects/mod.rs`
- Test: `src-tauri/src/projects/mod.rs`, `src-tauri/src/library_sources/mod.rs`

**Interfaces:**
- Consumes:
  - `SourceJobState::catalog_jobs_for_sources(&[i64]) -> Vec<SourceJobRecord>`
  - `latest_catalog_jobs_by_source(source_ids, jobs)`
  - `catalog_status_for_input(input, latest_job)`
- Produces: extended `ProjectSourceRecord` with `last_synced_at`, `sync_status`, and `handle`

- [ ] **Step 1: Make catalog status type public**

In `src-tauri/src/library_sources/models.rs`, change:

```rust
pub(crate) enum LibraryCatalogStatus {
```

to:

```rust
pub enum LibraryCatalogStatus {
```

Keep `#[serde(rename_all = "snake_case")]`.

In `src-tauri/src/library_sources/mod.rs`, extend the existing model re-export:

```rust
pub use models::{
    LibraryCatalogStatus, LibrarySourceRecord, LibraryTelegramSourceDetails,
    LibraryYoutubeSourceDetails,
};
```

- [ ] **Step 2: Extract a minimal catalog status helper**

In `src-tauri/src/library_sources/mod.rs`, change:

```rust
fn latest_catalog_jobs_by_source(
```

to:

```rust
pub(crate) fn latest_catalog_jobs_by_source(
```

Do not expose `catalog_status_for_source` as a public helper for partial project rows. It accepts a full `LibrarySourceRecord`, and project source rows should not construct fake records with `youtube: None` / `telegram: None`. Instead, add a stable minimal input beside it:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CatalogStatusInput<'a> {
    pub provider: &'a str,
    pub source_subtype: Option<&'a str>,
}

pub(crate) fn catalog_status_for_input(
    input: CatalogStatusInput<'_>,
    latest_job: Option<&SourceJobRecord>,
) -> (LibraryCatalogStatus, Option<String>) {
    if input.provider == "youtube" && input.source_subtype == Some("channel") {
        return (
            LibraryCatalogStatus::Unavailable,
            Some(YOUTUBE_CHANNEL_DISABLED_REASON.to_string()),
        );
    }

    if let Some(job) = latest_job {
        return match job.status {
            SourceJobStatus::Queued | SourceJobStatus::Running => (
                LibraryCatalogStatus::Syncing,
                job.message
                    .clone()
                    .or_else(|| Some(SOURCE_SYNCING_DISABLED_REASON.to_string())),
            ),
            SourceJobStatus::Failed => (
                LibraryCatalogStatus::Error,
                job.error.clone().or_else(|| job.message.clone()),
            ),
            SourceJobStatus::Succeeded
            | SourceJobStatus::CancelRequested
            | SourceJobStatus::Cancelled => (LibraryCatalogStatus::Active, None),
        };
    }

    (LibraryCatalogStatus::Active, None)
}
```

Then make the private full-record helper delegate to the minimal helper:

```rust
fn catalog_status_for_source(
    source: &LibrarySourceRecord,
    latest_job: Option<&SourceJobRecord>,
) -> (LibraryCatalogStatus, Option<String>) {
    catalog_status_for_input(
        CatalogStatusInput {
            provider: source.provider.as_str(),
            source_subtype: source.source_subtype.as_deref(),
        },
        latest_job,
    )
}
```

Also add a regression test in `src-tauri/src/library_sources/mod.rs` so the new helper preserves the existing catalog contract for failed jobs without detail:

```rust
#[test]
fn catalog_status_for_input_keeps_failed_job_without_detail_empty() {
    let job = SourceJobRecord {
        job_id: "job-1".to_string(),
        source_id: 1,
        related_source_id: None,
        job_type: SourceJobType::YoutubeVideoTranscriptSync,
        status: SourceJobStatus::Failed,
        message: None,
        progress_current: None,
        progress_total: None,
        started_at: 10,
        finished_at: Some(11),
        warnings: Vec::new(),
        error: None,
    };

    let (status, detail) = catalog_status_for_input(
        CatalogStatusInput {
            provider: "youtube",
            source_subtype: Some("video"),
        },
        Some(&job),
    );

    assert_eq!(status, LibraryCatalogStatus::Error);
    assert_eq!(detail, None);
}
```

- [ ] **Step 3: Add failing project source status test**

In `src-tauri/src/projects/mod.rs`, extend the test imports:

```rust
use crate::youtube::jobs::{SourceJobState, SourceJobStatus, SourceJobType, YoutubeSyncOptions};
```

Add:

```rust
#[tokio::test]
async fn list_project_sources_includes_catalog_status_last_sync_and_handle() {
    let pool = pool().await;
    seed_source(&pool, 10, "youtube", "video").await;
    sqlx::query("UPDATE sources SET last_synced_at = 1234, external_id = 'video-10' WHERE id = 10")
        .execute(&pool)
        .await
        .expect("update source metadata");
    let project = create_project_in_pool(&pool, "Status rows", None)
        .await
        .expect("create project");
    add_project_sources_in_pool(&pool, project.id, vec![10])
        .await
        .expect("add source");

    let source_jobs = SourceJobState::new();
    let options = YoutubeSyncOptions {
        metadata: true,
        transcripts: true,
        comments: false,
    };
    let failed = source_jobs
        .create_job(10, SourceJobType::YoutubeVideoTranscriptSync, None, options)
        .await
        .expect("create job");
    source_jobs
        .finish_job(&failed.job_id, |job| {
            job.status = SourceJobStatus::Failed;
            job.started_at = 77;
            job.error = Some("Transcript quota exceeded".to_string());
        })
        .await
        .expect("finish job");

    let sources = list_project_sources_in_pool(&pool, &source_jobs, project.id)
        .await
        .expect("list project sources");

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].last_synced_at, Some(1234));
    assert_eq!(sources[0].sync_status, crate::library_sources::LibraryCatalogStatus::Error);
    assert_eq!(sources[0].handle.as_deref(), Some("video-10"));
}

#[tokio::test]
async fn list_project_sources_counts_playlist_linked_video_materials() {
    let pool = pool().await;
    seed_source(&pool, 20, "youtube", "playlist").await;
    seed_source(&pool, 21, "youtube", "video").await;
    let project = create_project_in_pool(&pool, "Playlist rows", None)
        .await
        .expect("create project");
    add_project_sources_in_pool(&pool, project.id, vec![20])
        .await
        .expect("add playlist source");
    sqlx::query(
        "INSERT INTO youtube_playlist_items (playlist_source_id, video_source_id, video_id, position, availability_status, is_removed_from_playlist) VALUES (20, 21, 'video-21', 1, 'available', 0)",
    )
    .execute(&pool)
    .await
    .expect("link playlist video");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_zstd, item_kind) VALUES (210, 21, 'video-item-1', 'Author', 1000, 1001, x'01', 'youtube_transcript'), (211, 21, 'video-item-2', 'Author', 1002, 1003, x'01', 'youtube_description')",
    )
    .execute(&pool)
    .await
    .expect("seed video items");

    let sources = list_project_sources_in_pool(&pool, &SourceJobState::new(), project.id)
        .await
        .expect("list project sources");

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].source_id, 20);
    assert_eq!(sources[0].item_count, 2);
}
```

- [ ] **Step 4: Run test to verify it fails**

Run:

```powershell
cargo test projects::tests::list_project_sources_includes_catalog_status_last_sync_and_handle projects::tests::list_project_sources_counts_playlist_linked_video_materials
```

Expected: compile failure because the `ProjectSourceRecord` fields and new `list_project_sources_in_pool` signature do not exist yet.

- [ ] **Step 5: Extend `ProjectSourceRecord`**

In `src-tauri/src/projects/mod.rs`, add the import:

```rust
use crate::library_sources::LibraryCatalogStatus;
use crate::youtube::jobs::SourceJobState;
```

Change `ProjectSourceRecord` to include:

```rust
pub last_synced_at: Option<i64>,
pub sync_status: LibraryCatalogStatus,
pub handle: Option<String>,
```

Remove `sqlx::FromRow` from this struct, because the status is computed from SQL rows plus in-memory jobs:

```rust
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ProjectSourceRecord {
    pub project_id: i64,
    pub source_id: i64,
    pub provider: String,
    pub source_subtype: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub item_count: i64,
    pub added_at: i64,
    pub last_synced_at: Option<i64>,
    pub sync_status: LibraryCatalogStatus,
    pub handle: Option<String>,
}
```

- [ ] **Step 6: Add a private SQL row and handle mapper**

Add near `ProjectSourceRecord`:

```rust
#[derive(sqlx::FromRow)]
struct ProjectSourceRow {
    project_id: i64,
    source_id: i64,
    provider: String,
    source_subtype: Option<String>,
    title: Option<String>,
    subtitle: Option<String>,
    item_count: i64,
    added_at: i64,
    last_synced_at: Option<i64>,
    external_id: Option<String>,
}

fn project_source_handle(external_id: Option<String>) -> Option<String> {
    external_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
```

- [ ] **Step 7: Update `list_project_sources_in_pool` to consume `SourceJobState`**

Change the signature:

```rust
pub(crate) async fn list_project_sources_in_pool(
    pool: &sqlx::SqlitePool,
    source_jobs: &SourceJobState,
    project_id: i64,
) -> AppResult<Vec<ProjectSourceRecord>> {
```

Use this query body:

```rust
ensure_project_exists(pool, project_id).await?;
let rows: Vec<ProjectSourceRow> = sqlx::query_as(
    r#"
    WITH source_material_sources AS (
        SELECT
            ps.project_id,
            ps.source_id AS row_source_id,
            CASE
                WHEN s.source_type = 'youtube'
                 AND s.source_subtype = 'playlist'
                THEN ypi.video_source_id
                ELSE ps.source_id
            END AS material_source_id
        FROM project_sources ps
        JOIN sources s ON s.id = ps.source_id
        LEFT JOIN youtube_playlist_items ypi
            ON ypi.playlist_source_id = ps.source_id
           AND ypi.video_source_id IS NOT NULL
           AND ypi.is_removed_from_playlist = 0
        WHERE ps.project_id = ?
          AND (
              s.source_type <> 'youtube'
           OR s.source_subtype <> 'playlist'
           OR ypi.video_source_id IS NOT NULL
          )
    ),
    item_counts AS (
        SELECT
            sms.project_id,
            sms.row_source_id AS source_id,
            COUNT(DISTINCT items.id) AS item_count
        FROM source_material_sources sms
        JOIN items ON items.source_id = sms.material_source_id
        WHERE items.content_zstd IS NOT NULL
        GROUP BY sms.project_id, sms.row_source_id
    )
    SELECT
        ps.project_id,
        s.id AS source_id,
        s.source_type AS provider,
        s.source_subtype,
        s.title,
        CASE
            WHEN s.account_id IS NOT NULL THEN 'Account #' || s.account_id
            ELSE NULL
        END AS subtitle,
        COALESCE(item_counts.item_count, 0) AS item_count,
        ps.added_at,
        s.last_synced_at,
        s.external_id
    FROM project_sources ps
    JOIN sources s ON s.id = ps.source_id
    LEFT JOIN item_counts
        ON item_counts.project_id = ps.project_id
       AND item_counts.source_id = ps.source_id
    WHERE ps.project_id = ?
    ORDER BY ps.added_at DESC, s.id DESC
    "#,
)
.bind(project_id)
.bind(project_id)
.fetch_all(pool)
.await
.map_err(AppError::database)?;

let source_ids = rows.iter().map(|row| row.source_id).collect::<Vec<_>>();
let jobs = source_jobs.catalog_jobs_for_sources(&source_ids).await;
let latest_jobs = crate::library_sources::latest_catalog_jobs_by_source(&source_ids, jobs);

Ok(rows
    .into_iter()
    .map(|row| {
        let (sync_status, _) = crate::library_sources::catalog_status_for_input(
            crate::library_sources::CatalogStatusInput {
                provider: row.provider.as_str(),
                source_subtype: row.source_subtype.as_deref(),
            },
            latest_jobs.get(&row.source_id),
        );
        ProjectSourceRecord {
            project_id: row.project_id,
            source_id: row.source_id,
            provider: row.provider,
            source_subtype: row.source_subtype,
            title: row.title,
            subtitle: row.subtitle,
            item_count: row.item_count,
            added_at: row.added_at,
            last_synced_at: row.last_synced_at,
            sync_status,
            handle: project_source_handle(row.external_id),
        }
    })
    .collect())
```

- [ ] **Step 8: Update Tauri command wrapper**

Change `list_project_sources`:

```rust
#[tauri::command]
pub async fn list_project_sources(
    handle: AppHandle,
    source_jobs: tauri::State<'_, SourceJobState>,
    project_id: i64,
) -> AppResult<Vec<ProjectSourceRecord>> {
    let pool = get_pool(&handle).await?;
    list_project_sources_in_pool(&pool, source_jobs.inner(), project_id).await
}
```

Update existing tests that call `list_project_sources_in_pool` by passing `&SourceJobState::new()`.

- [ ] **Step 9: Verify source status task**

Run:

```powershell
cargo test projects::tests::add_project_sources_is_idempotent_and_lists_ui_ready_rows projects::tests::list_project_sources_includes_catalog_status_last_sync_and_handle projects::tests::list_project_sources_counts_playlist_linked_video_materials
cargo test library_sources::tests
cargo check
```

Expected: selected tests and `cargo check` pass.

- [ ] **Step 10: Commit Task 3**

```powershell
git add src-tauri/src/library_sources/models.rs src-tauri/src/library_sources/mod.rs src-tauri/src/projects/mod.rs
git commit -m "feat: expose project source sync status"
```

### Task 4: Research Projects Read Model

**Files:**
- Modify: `src-tauri/src/projects/mod.rs`
- Create: `src-tauri/src/projects/read_model.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/projects/read_model.rs`

**Interfaces:**
- Consumes: `projects.pinned`, `projects.archived_at`, `project_sources`, `items`, `analysis_runs`
- Produces:
  - `list_research_projects_in_pool(pool) -> AppResult<Vec<ProjectSummary>>`
  - `list_research_projects(handle) -> AppResult<Vec<ProjectSummary>>`

- [ ] **Step 1: Add module declaration and re-exports**

At the top of `src-tauri/src/projects/mod.rs`:

```rust
mod read_model;

pub(crate) use read_model::list_research_projects_in_pool;
pub use read_model::{list_research_projects, ProjectStatus, ProjectSummary};
```

- [ ] **Step 2: Create failing read-model tests**

Create `src-tauri/src/projects/read_model.rs` with the test module first:

```rust
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppErrorKind, AppResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::apply_all_migrations_for_test_pool;

    async fn pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        pool
    }

    async fn seed_project(pool: &sqlx::SqlitePool, id: i64, name: &str, updated_at: i64) {
        sqlx::query(
            "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, NULL, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(updated_at)
        .bind(updated_at)
        .execute(pool)
        .await
        .expect("seed project");
    }

    async fn seed_source(pool: &sqlx::SqlitePool, id: i64, provider: &str, subtype: &str) {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (?, ?, ?, ?, ?, 1, 0, 100)",
        )
        .bind(id)
        .bind(provider)
        .bind(subtype)
        .bind(format!("{provider}-{id}"))
        .bind(format!("Source {id}"))
        .execute(pool)
        .await
        .expect("seed source");
    }

    async fn attach_source(pool: &sqlx::SqlitePool, project_id: i64, source_id: i64) {
        sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (?, ?, 100)")
            .bind(project_id)
            .bind(source_id)
            .execute(pool)
            .await
            .expect("attach source");
    }

    async fn seed_item(pool: &sqlx::SqlitePool, id: i64, source_id: i64) {
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_zstd, item_kind) VALUES (?, ?, ?, 'Author', 1000, 1001, x'01', 'telegram_message')",
        )
        .bind(id)
        .bind(source_id)
        .bind(format!("item-{id}"))
        .execute(pool)
        .await
        .expect("seed item");
    }

    async fn seed_run(pool: &sqlx::SqlitePool, id: i64, project_id: i64, status: &str, created_at: i64) {
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id, run_type, scope_type, project_id, period_from, period_to,
                output_language, prompt_template_id, prompt_template_version,
                provider_profile, provider, model, status, created_at
            )
            VALUES (?, 'report', 'project', ?, 1, 2, 'en', 1, 1, 'default', 'openai', 'gpt', ?, ?)
            "#,
        )
        .bind(id)
        .bind(project_id)
        .bind(status)
        .bind(created_at)
        .execute(pool)
        .await
        .expect("seed run");
    }

    #[tokio::test]
    async fn list_research_projects_derives_counts_status_and_last_run_without_fanout() {
        let pool = pool().await;
        seed_project(&pool, 1, "Ready", 10).await;
        seed_project(&pool, 2, "Failed", 20).await;
        seed_source(&pool, 10, "telegram", "channel").await;
        seed_source(&pool, 11, "telegram", "channel").await;
        attach_source(&pool, 1, 10).await;
        attach_source(&pool, 1, 11).await;
        attach_source(&pool, 2, 10).await;
        seed_item(&pool, 100, 10).await;
        seed_item(&pool, 101, 10).await;
        seed_item(&pool, 102, 11).await;
        seed_run(&pool, 500, 1, "completed", 1000).await;
        seed_run(&pool, 501, 2, "completed", 1000).await;
        seed_run(&pool, 502, 2, "failed", 1000).await;

        let rows = list_research_projects_in_pool(&pool)
            .await
            .expect("list research projects");

        let ready = rows.iter().find(|row| row.id == 1).expect("ready project");
        assert_eq!(ready.source_count, 2);
        assert_eq!(ready.material_count, 3);
        assert_eq!(ready.status, ProjectStatus::Ready);
        assert_eq!(ready.last_run_at, Some(1000));

        let failed = rows.iter().find(|row| row.id == 2).expect("failed project");
        assert_eq!(failed.source_count, 1);
        assert_eq!(failed.material_count, 2);
        assert_eq!(failed.status, ProjectStatus::NeedsAttention);
    }

    #[tokio::test]
    async fn list_research_projects_counts_playlist_linked_video_materials() {
        let pool = pool().await;
        seed_project(&pool, 4, "Playlist", 40).await;
        seed_source(&pool, 40, "youtube", "playlist").await;
        seed_source(&pool, 41, "youtube", "video").await;
        attach_source(&pool, 4, 40).await;
        sqlx::query(
            "INSERT INTO youtube_playlist_items (playlist_source_id, video_source_id, video_id, position, availability_status, is_removed_from_playlist) VALUES (40, 41, 'video-41', 1, 'available', 0)",
        )
        .execute(&pool)
        .await
        .expect("link playlist video");
        seed_item(&pool, 410, 41).await;
        seed_item(&pool, 411, 41).await;

        let rows = list_research_projects_in_pool(&pool)
            .await
            .expect("list research projects");

        let playlist = rows.iter().find(|row| row.id == 4).expect("playlist project");
        assert_eq!(playlist.source_count, 1);
        assert_eq!(playlist.material_count, 2);
    }

    #[tokio::test]
    async fn list_research_projects_prioritizes_running_over_empty_and_sorts_pinned_active_first() {
        let pool = pool().await;
        seed_project(&pool, 1, "Archived", 30).await;
        seed_project(&pool, 2, "Pinned", 20).await;
        seed_project(&pool, 3, "Running empty", 10).await;
        sqlx::query("UPDATE projects SET archived_at = 300 WHERE id = 1")
            .execute(&pool)
            .await
            .expect("archive project");
        sqlx::query("UPDATE projects SET pinned = 1 WHERE id = 2")
            .execute(&pool)
            .await
            .expect("pin project");
        seed_run(&pool, 700, 3, "running", 2000).await;

        let rows = list_research_projects_in_pool(&pool)
            .await
            .expect("list research projects");

        assert_eq!(
            rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![2, 3, 1]
        );
        let running = rows.iter().find(|row| row.id == 3).expect("running project");
        assert_eq!(running.status, ProjectStatus::Running);
        assert_eq!(running.source_count, 0);
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```powershell
cargo test projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials projects::read_model::tests::list_research_projects_prioritizes_running_over_empty_and_sorts_pinned_active_first
```

Expected: compile failure because `ProjectSummary`, `ProjectStatus`, and `list_research_projects_in_pool` are not implemented.

- [ ] **Step 4: Add read-model structs and row mapper**

Add above the tests in `src-tauri/src/projects/read_model.rs`:

```rust
#[derive(Clone, Copy, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Ready,
    Running,
    NeedsAttention,
    Empty,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub source_count: i64,
    pub material_count: i64,
    pub status: ProjectStatus,
    pub last_run_at: Option<i64>,
    pub pinned: bool,
    pub archived: bool,
    pub updated_at: i64,
}

#[derive(sqlx::FromRow)]
struct ProjectSummaryRow {
    id: i64,
    name: String,
    description: Option<String>,
    source_count: i64,
    material_count: i64,
    latest_run_status: Option<String>,
    last_run_at: Option<i64>,
    has_active_run: i64,
    pinned: i64,
    archived_at: Option<i64>,
    updated_at: i64,
}

fn project_status(row: &ProjectSummaryRow) -> ProjectStatus {
    if row.has_active_run > 0 {
        ProjectStatus::Running
    } else if row.source_count == 0 {
        ProjectStatus::Empty
    } else if row.latest_run_status.as_deref() == Some("failed") {
        ProjectStatus::NeedsAttention
    } else {
        ProjectStatus::Ready
    }
}

fn map_project_summary(row: ProjectSummaryRow) -> ProjectSummary {
    let status = project_status(&row);
    ProjectSummary {
        id: row.id,
        name: row.name,
        description: row.description,
        source_count: row.source_count,
        material_count: row.material_count,
        status,
        last_run_at: row.last_run_at,
        pinned: row.pinned != 0,
        archived: row.archived_at.is_some(),
        updated_at: row.updated_at,
    }
}
```

- [ ] **Step 5: Implement SQL read model without fanout**

Add:

```rust
pub(crate) async fn list_research_projects_in_pool(
    pool: &sqlx::SqlitePool,
) -> AppResult<Vec<ProjectSummary>> {
    let rows: Vec<ProjectSummaryRow> = sqlx::query_as(
        r#"
        WITH resolved_material_sources AS (
            SELECT
                ps.project_id,
                CASE
                    WHEN s.source_type = 'youtube'
                     AND s.source_subtype = 'playlist'
                    THEN ypi.video_source_id
                    ELSE ps.source_id
                END AS material_source_id
            FROM project_sources ps
            JOIN sources s ON s.id = ps.source_id
            LEFT JOIN youtube_playlist_items ypi
                ON ypi.playlist_source_id = ps.source_id
               AND ypi.video_source_id IS NOT NULL
               AND ypi.is_removed_from_playlist = 0
            WHERE s.source_type <> 'youtube'
               OR s.source_subtype <> 'playlist'
               OR ypi.video_source_id IS NOT NULL
        ),
        source_counts AS (
            SELECT project_id, COUNT(*) AS source_count
            FROM project_sources
            GROUP BY project_id
        ),
        material_counts AS (
            SELECT
                rms.project_id,
                COUNT(DISTINCT items.id) AS material_count
            FROM resolved_material_sources rms
            JOIN items ON items.source_id = rms.material_source_id
            WHERE items.content_zstd IS NOT NULL
            GROUP BY rms.project_id
        )
        SELECT
            p.id,
            p.name,
            p.description,
            COALESCE(source_counts.source_count, 0) AS source_count,
            COALESCE(material_counts.material_count, 0) AS material_count,
            (
                SELECT ar.status
                FROM analysis_runs ar
                WHERE ar.project_id = p.id
                ORDER BY ar.created_at DESC, ar.id DESC
                LIMIT 1
            ) AS latest_run_status,
            (
                SELECT ar.created_at
                FROM analysis_runs ar
                WHERE ar.project_id = p.id
                ORDER BY ar.created_at DESC, ar.id DESC
                LIMIT 1
            ) AS last_run_at,
            CASE WHEN EXISTS (
                SELECT 1
                FROM analysis_runs ar
                WHERE ar.project_id = p.id
                  AND ar.status IN ('queued', 'running')
            ) THEN 1 ELSE 0 END AS has_active_run,
            p.pinned,
            p.archived_at,
            p.updated_at
        FROM projects p
        LEFT JOIN source_counts ON source_counts.project_id = p.id
        LEFT JOIN material_counts ON material_counts.project_id = p.id
        ORDER BY
            CASE WHEN p.archived_at IS NULL THEN 0 ELSE 1 END ASC,
            p.pinned DESC,
            p.updated_at DESC,
            p.id DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(rows.into_iter().map(map_project_summary).collect())
}

#[tauri::command]
pub async fn list_research_projects(handle: AppHandle) -> AppResult<Vec<ProjectSummary>> {
    let pool = get_pool(&handle).await?;
    list_research_projects_in_pool(&pool).await
}
```

- [ ] **Step 6: Register command**

In `src-tauri/src/lib.rs`, add `list_research_projects` to the `use projects::{...}` list and to `tauri::generate_handler![...]` next to `list_projects`.

- [ ] **Step 7: Verify read-model task**

Run:

```powershell
cargo test projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials projects::read_model::tests::list_research_projects_prioritizes_running_over_empty_and_sorts_pinned_active_first
cargo check
```

Expected: selected tests and `cargo check` pass.

- [ ] **Step 8: Commit Task 4**

```powershell
git add src-tauri/src/projects/mod.rs src-tauri/src/projects/read_model.rs src-tauri/src/lib.rs
git commit -m "feat: add research projects read model"
```

### Task 5: Lazy Project Data Range

**Files:**
- Modify: `src-tauri/src/projects/mod.rs`
- Create: `src-tauri/src/projects/data_range.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/projects/data_range.rs`

**Interfaces:**
- Consumes:
  - `resolve_analysis_sources(pool, None, None, Some(project_id))`
  - `YoutubeCorpusMode::from_wire`
  - `resolve_analysis_telegram_history_scope(include_migrated_history, source_type)`
- Produces:
  - `get_project_data_range_in_pool(pool, project_id, youtube_corpus_mode, include_migrated_history)`
  - `get_project_data_range(handle, project_id, youtube_corpus_mode, include_migrated_history)`

- [ ] **Step 1: Raise Telegram history validation helper visibility**

In `src-tauri/src/analysis/report.rs`, change:

```rust
fn resolve_analysis_telegram_history_scope(
```

to:

```rust
pub(crate) fn resolve_analysis_telegram_history_scope(
```

- [ ] **Step 2: Extract shared document-kind filter helper**

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
pub(crate) fn push_analysis_document_kind_filter(
    query: &mut QueryBuilder<'_, Sqlite>,
    source_type: &str,
    youtube_corpus_mode: YoutubeCorpusMode,
    table_alias: &str,
) -> AppResult<()> {
    match source_type {
        "telegram" => {
            query.push(" AND ");
            query.push(table_alias);
            query.push(".source_type = 'telegram' AND ");
            query.push(table_alias);
            query.push(".document_kind = 'telegram_message'");
            Ok(())
        }
        "youtube" => {
            query.push(" AND ");
            query.push(table_alias);
            query.push(".source_type = 'youtube' AND ");
            query.push(table_alias);
            query.push(".document_kind IN (");
            query.push("'youtube_transcript'");
            if youtube_corpus_mode.includes_description() {
                query.push(", 'youtube_description'");
            }
            if youtube_corpus_mode.includes_comments() {
                query.push(", 'youtube_comment'");
            }
            query.push(")");
            Ok(())
        }
        other => Err(AppError::validation(format!(
            "Unsupported analysis corpus source_type '{other}'"
        ))),
    }
}
```

Then replace the matching block in `load_analysis_document_messages` with:

First update the existing query in `load_analysis_document_messages` to alias `analysis_documents`:

```rust
        FROM analysis_documents d
        WHERE d.published_at >=
```

Then update the following clauses in that function to use the same alias:

```rust
query.push(" AND d.published_at <= ");
query.push_bind(request.period_to);
query.push(" AND d.source_id IN (");
```

Qualify every `analysis_documents` column in that query with `d.` while editing this block: `d.item_id`, `d.source_id`, `d.external_id`, `d.author`, `d.published_at`, `d.ref`, `d.content_zstd`, `d.document_kind`, `d.source_type`, `d.source_subtype`, `d.metadata_zstd`, `d.document_order`, and `d.id`.

```rust
push_analysis_document_kind_filter(
    &mut query,
    request.source_type.as_str(),
    request.youtube_corpus_mode,
    "d",
)?;
```

- [ ] **Step 3: Re-export shared corpus APIs from `analysis`**

`src-tauri/src/analysis/mod.rs` declares `mod corpus;`, so `projects::data_range` cannot import `crate::analysis::corpus::*`. Keep the module private and re-export only the APIs needed by the project data range command.

Add near the existing `pub use self::...` block in `src-tauri/src/analysis/mod.rs`:

```rust
pub(crate) use self::corpus::{
    push_analysis_document_kind_filter, resolve_analysis_sources, YoutubeCorpusMode,
};
```

`projects/data_range.rs` must import these from `crate::analysis`, not `crate::analysis::corpus`.

- [ ] **Step 4: Add module declaration and re-export**

At the top of `src-tauri/src/projects/mod.rs`:

```rust
mod data_range;

pub(crate) use data_range::get_project_data_range_in_pool;
pub use data_range::{get_project_data_range, ProjectDataRange};
```

- [ ] **Step 5: Create failing data range tests**

Create `src-tauri/src/projects/data_range.rs` with tests:

```rust
use sqlx::{QueryBuilder, Sqlite};
use tauri::AppHandle;

use crate::analysis::{
    push_analysis_document_kind_filter, resolve_analysis_sources, YoutubeCorpusMode,
};
use crate::analysis::report::resolve_analysis_telegram_history_scope;
use crate::db::get_pool;
use crate::error::{AppError, AppErrorKind, AppResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::apply_all_migrations_for_test_pool;

    async fn pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        pool
    }

    async fn seed_project(pool: &sqlx::SqlitePool, project_id: i64) {
        sqlx::query(
            "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, NULL, 1, 1)",
        )
        .bind(project_id)
        .bind(format!("Project {project_id}"))
        .execute(pool)
        .await
        .expect("seed project");
    }

    async fn seed_source(pool: &sqlx::SqlitePool, id: i64, provider: &str, subtype: &str) {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (?, ?, ?, ?, ?, 1, 0, 1)",
        )
        .bind(id)
        .bind(provider)
        .bind(subtype)
        .bind(format!("{provider}-{id}"))
        .bind(format!("Source {id}"))
        .execute(pool)
        .await
        .expect("seed source");
    }

    async fn attach(pool: &sqlx::SqlitePool, project_id: i64, source_id: i64) {
        sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (?, ?, 1)")
            .bind(project_id)
            .bind(source_id)
            .execute(pool)
            .await
            .expect("attach source");
    }

    async fn seed_document(
        pool: &sqlx::SqlitePool,
        id: i64,
        source_id: i64,
        source_type: &str,
        source_subtype: &str,
        document_kind: &str,
        published_at: i64,
    ) {
        let item_id = match document_kind {
            "telegram_message" | "youtube_transcript" | "youtube_comment" => Some(id + 10_000),
            "youtube_description" => None,
            other => panic!("unsupported test document kind {other}"),
        };
        let document_key = match item_id {
            Some(item_id) => format!("item:{item_id}"),
            None => "youtube:description".to_string(),
        };
        if let Some(item_id) = item_id {
            sqlx::query(
                "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_zstd, item_kind) VALUES (?, ?, ?, 'Author', ?, ?, x'01', ?)",
            )
            .bind(item_id)
            .bind(source_id)
            .bind(format!("item-{id}"))
            .bind(published_at)
            .bind(published_at + 1)
            .bind(document_kind)
            .execute(pool)
            .await
            .expect("seed backing item");
        }

        sqlx::query(
            r#"
            INSERT INTO analysis_documents (
                id, source_id, item_id, document_key, document_kind, source_type, source_subtype,
                external_id, author, published_at, document_order, ref, content_zstd, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'Author', ?, 0, ?, x'01', ?, ?)
            "#,
        )
        .bind(id)
        .bind(source_id)
        .bind(item_id)
        .bind(document_key)
        .bind(document_kind)
        .bind(source_type)
        .bind(source_subtype)
        .bind(format!("external-{id}"))
        .bind(published_at)
        .bind(format!("ref-{id}"))
        .bind(published_at)
        .bind(published_at)
        .execute(pool)
        .await
        .expect("seed document");
    }

    #[tokio::test]
    async fn project_data_range_returns_nulls_for_empty_project() {
        let pool = pool().await;
        seed_project(&pool, 4).await;

        let range = get_project_data_range_in_pool(&pool, 4, None, false)
            .await
            .expect("empty project range");

        assert_eq!(range, ProjectDataRange { from: None, to: None });
    }

    #[tokio::test]
    async fn project_data_range_uses_youtube_mode_document_kinds() {
        let pool = pool().await;
        seed_project(&pool, 1).await;
        seed_source(&pool, 10, "youtube", "video").await;
        attach(&pool, 1, 10).await;
        seed_document(&pool, 1, 10, "youtube", "video", "youtube_transcript", 100).await;
        seed_document(&pool, 2, 10, "youtube", "video", "youtube_description", 50).await;
        seed_document(&pool, 3, 10, "youtube", "video", "youtube_comment", 200).await;

        let transcript_only = get_project_data_range_in_pool(
            &pool,
            1,
            Some("transcript_only".to_string()),
            false,
        )
        .await
        .expect("range transcript only");
        assert_eq!(transcript_only, ProjectDataRange { from: Some(100), to: Some(100) });

        let all_text = get_project_data_range_in_pool(
            &pool,
            1,
            Some("transcript_description_comments".to_string()),
            false,
        )
        .await
        .expect("range all text");
        assert_eq!(all_text, ProjectDataRange { from: Some(50), to: Some(200) });
    }

    #[tokio::test]
    async fn project_data_range_expands_playlist_to_linked_video_sources() {
        let pool = pool().await;
        seed_project(&pool, 2).await;
        seed_source(&pool, 20, "youtube", "playlist").await;
        seed_source(&pool, 21, "youtube", "video").await;
        attach(&pool, 2, 20).await;
        sqlx::query(
            "INSERT INTO youtube_playlist_items (playlist_source_id, video_source_id, video_id, position, availability_status, is_removed_from_playlist) VALUES (20, 21, 'video-21', 1, 'available', 0)",
        )
        .execute(&pool)
        .await
        .expect("link playlist item");
        seed_document(&pool, 4, 21, "youtube", "video", "youtube_transcript", 777).await;

        let range = get_project_data_range_in_pool(&pool, 2, None, false)
            .await
            .expect("playlist range");

        assert_eq!(range, ProjectDataRange { from: Some(777), to: Some(777) });
    }

    #[tokio::test]
    async fn project_data_range_returns_nulls_for_unmaterialized_playlist_project() {
        let pool = pool().await;
        seed_project(&pool, 5).await;
        seed_source(&pool, 50, "youtube", "playlist").await;
        attach(&pool, 5, 50).await;

        let range = get_project_data_range_in_pool(&pool, 5, None, false)
            .await
            .expect("unmaterialized playlist range");

        assert_eq!(range, ProjectDataRange { from: None, to: None });
    }

    #[tokio::test]
    async fn project_data_range_returns_nulls_for_mixed_provider_project() {
        let pool = pool().await;
        seed_project(&pool, 7).await;
        seed_source(&pool, 70, "youtube", "video").await;
        seed_source(&pool, 71, "telegram", "supergroup").await;
        attach(&pool, 7, 70).await;
        attach(&pool, 7, 71).await;

        let range = get_project_data_range_in_pool(&pool, 7, None, false)
            .await
            .expect("mixed provider project range");

        assert_eq!(range, ProjectDataRange { from: None, to: None });
    }

    #[tokio::test]
    async fn project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project() {
        let pool = pool().await;
        seed_project(&pool, 6).await;
        seed_source(&pool, 60, "youtube", "playlist").await;
        attach(&pool, 6, 60).await;

        let error = get_project_data_range_in_pool(&pool, 6, None, true)
            .await
            .expect_err("unmaterialized playlist migrated history rejected");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("Migrated historical scope"));
    }

    #[tokio::test]
    async fn project_data_range_rejects_migrated_history_for_non_telegram() {
        let pool = pool().await;
        seed_project(&pool, 3).await;
        seed_source(&pool, 30, "youtube", "video").await;
        attach(&pool, 3, 30).await;

        let error = get_project_data_range_in_pool(&pool, 3, None, true)
            .await
            .expect_err("non-telegram migrated history rejected");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("Migrated historical scope"));
    }
}
```

- [ ] **Step 6: Run tests to verify they fail**

Run:

```powershell
cargo test projects::data_range::tests::project_data_range_returns_nulls_for_empty_project projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project projects::data_range::tests::project_data_range_returns_nulls_for_mixed_provider_project projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram
```

Expected: compile failure because `ProjectDataRange` and `get_project_data_range_in_pool` are not implemented.

- [ ] **Step 7: Implement data range structs and query helper**

Add above the tests in `src-tauri/src/projects/data_range.rs`:

```rust
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ProjectDataRange {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

fn push_source_ids(query: &mut QueryBuilder<'_, Sqlite>, source_ids: &[i64]) {
    let mut separated = query.separated(", ");
    for source_id in source_ids {
        separated.push_bind(source_id);
    }
}

pub(crate) async fn get_project_data_range_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    youtube_corpus_mode: Option<String>,
    include_migrated_history: bool,
) -> AppResult<ProjectDataRange> {
    crate::projects::get_project_in_pool(pool, project_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Project {project_id} not found")))?;

    let youtube_corpus_mode = YoutubeCorpusMode::from_wire(youtube_corpus_mode.as_deref())
        .map_err(AppError::validation)?;

    let source_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_sources WHERE project_id = ?")
            .bind(project_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
    if source_count == 0 {
        return Ok(ProjectDataRange { from: None, to: None });
    }

    if include_migrated_history {
        let non_telegram_source_type: Option<String> = sqlx::query_scalar(
            r#"
            SELECT s.source_type
            FROM project_sources ps
            JOIN sources s ON s.id = ps.source_id
            WHERE ps.project_id = ?
              AND s.source_type <> 'telegram'
            ORDER BY s.id ASC
            LIMIT 1
            "#,
        )
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;
        if let Some(source_type) = non_telegram_source_type {
            resolve_analysis_telegram_history_scope(true, &source_type)?;
        }
    }

    let resolved = match resolve_analysis_sources(pool, None, None, Some(project_id)).await {
        Ok(resolved) => resolved,
        Err(error)
            if error.kind == AppErrorKind::Validation
                && matches!(
                    error.message.as_str(),
                    "No linked YouTube videos are available for analysis in this scope"
                        | "mixed_provider_project_runs_not_supported"
                ) =>
        {
            return Ok(ProjectDataRange { from: None, to: None });
        }
        Err(error) => return Err(error),
    };
    let (_, include_migrated_history) = resolve_analysis_telegram_history_scope(
        include_migrated_history,
        &resolved.source_type,
    )?;

    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT MIN(published_at) AS from_ts, MAX(published_at) AS to_ts
        FROM (
            SELECT d.published_at AS published_at
            FROM analysis_documents d
            WHERE d.source_id IN (
        "#,
    );
    push_source_ids(&mut query, &resolved.source_ids);
    query.push(")");
    push_analysis_document_kind_filter(
        &mut query,
        resolved.source_type.as_str(),
        youtube_corpus_mode,
        "d",
    )?;

    if include_migrated_history {
        query.push(
            r#"
            UNION ALL
            SELECT items.published_at AS published_at
            FROM items
            JOIN sources ON sources.id = items.source_id
            JOIN telegram_messages tm ON tm.item_id = items.id
            WHERE items.source_id IN (
            "#,
        );
        push_source_ids(&mut query, &resolved.source_ids);
        query.push(
            r#")
              AND sources.source_type = 'telegram'
              AND items.item_kind = 'telegram_message'
              AND tm.is_migrated_history = 1
              AND tm.migration_domain = 'migrated_from_chat'
              AND items.content_zstd IS NOT NULL
              AND items.content_kind IN ('text_only', 'text_with_media')
            "#,
        );
    }
    query.push(")");

    let row: (Option<i64>, Option<i64>) = query
        .build_query_as()
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    Ok(ProjectDataRange {
        from: row.0,
        to: row.1,
    })
}

#[tauri::command]
pub async fn get_project_data_range(
    handle: AppHandle,
    project_id: i64,
    youtube_corpus_mode: Option<String>,
    include_migrated_history: bool,
) -> AppResult<ProjectDataRange> {
    let pool = get_pool(&handle).await?;
    get_project_data_range_in_pool(&pool, project_id, youtube_corpus_mode, include_migrated_history)
        .await
}
```

- [ ] **Step 8: Register command**

In `src-tauri/src/lib.rs`, add `get_project_data_range` to the `use projects::{...}` list and to `tauri::generate_handler![...]` near `start_project_analysis`.

- [ ] **Step 9: Verify data range task**

Run:

```powershell
cargo test projects::data_range::tests::project_data_range_returns_nulls_for_empty_project projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram
cargo test analysis::corpus::tests::youtube_corpus_mode_from_wire_parses_expected_values
cargo check
```

Expected: selected tests and `cargo check` pass.

- [ ] **Step 10: Commit Task 5**

```powershell
git add src-tauri/src/projects/mod.rs src-tauri/src/projects/data_range.rs src-tauri/src/analysis/mod.rs src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/report.rs src-tauri/src/lib.rs
git commit -m "feat: add project data range command"
```

### Task 6: TypeScript Contracts, Invoke Wrappers, And Value Registry

**Files:**
- Modify: `src/lib/types/projects.ts`
- Modify: `src/lib/api/projects.ts`
- Modify: `src/lib/api/projects.test.ts`
- Modify: `docs/value-registry.md`

**Interfaces:**
- Consumes: Tauri commands from Tasks 2, 4, and 5
- Produces:
  - `listResearchProjects()`
  - `getProjectDataRange(input)`
  - `setProjectPinned(input)`
  - `setProjectArchived(input)`

- [ ] **Step 1: Extend TypeScript project types**

In `src/lib/types/projects.ts`, update imports:

```ts
import type { AnalysisRunSummary, YoutubeCorpusMode } from "$lib/types/analysis";
import type {
  LibraryCatalogStatus,
  LibrarySourceProvider,
  LibrarySourceSubtype,
} from "$lib/types/library-sources";
```

Add:

```ts
export type ProjectStatus = "ready" | "running" | "needs_attention" | "empty";

export interface ProjectSummary {
  id: number;
  name: string;
  description: string | null;
  source_count: number;
  material_count: number;
  status: ProjectStatus;
  last_run_at: number | null;
  pinned: boolean;
  archived: boolean;
  updated_at: number;
}

export interface ProjectDataRange {
  from: number | null;
  to: number | null;
}

export interface ProjectDataRangeInput {
  projectId: number;
  youtubeCorpusMode: YoutubeCorpusMode | null;
  includeMigratedHistory: boolean;
}

export interface ProjectPinnedInput {
  projectId: number;
  pinned: boolean;
}

export interface ProjectArchivedInput {
  projectId: number;
  archived: boolean;
}
```

Extend `ProjectSourceRecord`:

```ts
export interface ProjectSourceRecord {
  project_id: number;
  source_id: number;
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  title: string | null;
  subtitle: string | null;
  item_count: number;
  added_at: number;
  last_synced_at: number | null;
  sync_status: LibraryCatalogStatus;
  handle: string | null;
}
```

- [ ] **Step 2: Add invoke wrappers**

In `src/lib/api/projects.ts`, import the new input/output types and add:

```ts
export function listResearchProjects() {
  return invoke<ProjectSummary[]>("list_research_projects");
}

export function getProjectDataRange(input: ProjectDataRangeInput) {
  return invoke<ProjectDataRange>("get_project_data_range", { ...input });
}

export function setProjectPinned(input: ProjectPinnedInput) {
  return invoke<void>("set_project_pinned", { ...input });
}

export function setProjectArchived(input: ProjectArchivedInput) {
  return invoke<void>("set_project_archived", { ...input });
}
```

- [ ] **Step 3: Add API mapping tests**

In `src/lib/api/projects.test.ts`, include the new functions in the import list and add:

```ts
it("maps research projects v10 commands", async () => {
  invokeMock.mockResolvedValueOnce([]);
  await listResearchProjects();
  expect(invokeMock).toHaveBeenLastCalledWith("list_research_projects");

  invokeMock.mockResolvedValueOnce({ from: 10, to: 20 });
  await getProjectDataRange({
    projectId: 2,
    youtubeCorpusMode: "transcript_description",
    includeMigratedHistory: false,
  });
  expect(invokeMock).toHaveBeenLastCalledWith("get_project_data_range", {
    projectId: 2,
    youtubeCorpusMode: "transcript_description",
    includeMigratedHistory: false,
  });

  invokeMock.mockResolvedValueOnce(undefined);
  await setProjectPinned({ projectId: 2, pinned: true });
  expect(invokeMock).toHaveBeenLastCalledWith("set_project_pinned", {
    projectId: 2,
    pinned: true,
  });

  invokeMock.mockResolvedValueOnce(undefined);
  await setProjectArchived({ projectId: 2, archived: true });
  expect(invokeMock).toHaveBeenLastCalledWith("set_project_archived", {
    projectId: 2,
    archived: true,
  });
});
```

- [ ] **Step 4: Update value registry**

In `docs/value-registry.md`, update the `Research project status` row to record backend ownership:

```markdown
| Research project status | `ProjectStatus` | `ready`, `running`, `needs_attention`, `empty` | `src-tauri/src/projects/read_model.rs`, `src/lib/types/projects.ts` | Backend-derived API status for `ProjectSummary`; wire values are snake_case. |
```

Keep the `Library catalog status` row unchanged; the values are reused, not changed.

- [ ] **Step 5: Verify TypeScript and registry task**

Run:

```powershell
npm.cmd test -- src/lib/api/projects.test.ts
npm.cmd run check
```

Expected: the API test and Svelte/TypeScript check pass.

- [ ] **Step 6: Commit Task 6**

```powershell
git add src/lib/types/projects.ts src/lib/api/projects.ts src/lib/api/projects.test.ts docs/value-registry.md
git commit -m "feat: add research projects v10 frontend contract"
```

### Task 7: Full Verification And Integration Review

**Files:**
- Review: all files changed by Tasks 1-6
- Test: full focused backend/frontend validation

**Interfaces:**
- Consumes: all prior task commits
- Produces: verified branch ready for UI integration work

- [ ] **Step 1: Inspect dirty worktree**

Run:

```powershell
git status --short
```

Expected: no unexpected unrelated files. If `.playwright-mcp/` appears, leave it unstaged.

- [ ] **Step 2: Run backend focused tests**

Run:

```powershell
cargo test migrations::tests::fresh_schema_includes_projects_redesign_columns_index_and_defaults
cargo test projects::tests
cargo test projects::read_model::tests
cargo test projects::data_range::tests
cargo test library_sources::tests
```

Expected: all selected test sets pass.

- [ ] **Step 3: Run backend compile check**

Run:

```powershell
cargo check
```

Expected: `Finished` without errors.

- [ ] **Step 4: Run frontend checks**

Run:

```powershell
npm.cmd test -- src/lib/api/projects.test.ts
npm.cmd run check
```

Expected: selected Vitest test passes and `npm.cmd run check` exits successfully.

- [ ] **Step 5: Review command registration**

Run:

```powershell
rg -n "list_research_projects|get_project_data_range|set_project_pinned|set_project_archived" src-tauri/src/lib.rs src-tauri/src/projects src/lib/api src/lib/types
```

Expected:
- `src-tauri/src/lib.rs` imports and registers all four commands.
- `src-tauri/src/analysis/mod.rs` re-exports the corpus APIs used by `projects/data_range.rs`.
- `src-tauri/src/projects/read_model.rs` defines `list_research_projects`.
- `src-tauri/src/projects/data_range.rs` defines `get_project_data_range`.
- `src-tauri/src/projects/mod.rs` defines pin/archive commands.
- `src/lib/api/projects.ts` exposes invoke wrappers.
- `src/lib/types/projects.ts` exposes TS contracts.

- [ ] **Step 6: Commit verification-only fixes if any**

If Step 2-5 forced mechanical fixes, commit only planned implementation files:

```powershell
git add src-tauri/src/projects/mod.rs src-tauri/src/projects/read_model.rs src-tauri/src/projects/data_range.rs src-tauri/src/migrations.rs src-tauri/migrations/0012_projects_redesign.sql src-tauri/src/library_sources/models.rs src-tauri/src/library_sources/mod.rs src-tauri/src/analysis/mod.rs src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/report.rs src-tauri/src/lib.rs src/lib/types/projects.ts src/lib/api/projects.ts src/lib/api/projects.test.ts docs/value-registry.md
git commit -m "fix: stabilize research projects v10 backend contract"
```

If there were no fixes, do not create an empty commit.

- [ ] **Step 7: Final status**

Run:

```powershell
git status --short
git log --oneline -6
```

Expected: working tree is clean except ignored/generated files, and the latest commits correspond to Tasks 1-6 plus an optional stabilization commit.

## Self-Review

**Spec coverage:** This plan covers migration and registration, pin/archive mutations, source-row `last_synced_at`/status/handle via minimal catalog status input, backend-derived project summary status, playlist-aware `material_count` and `ProjectSourceRecord.item_count`, lazy `dataRange` with empty-project and unmaterialized-playlist null ranges, resolved source IDs, and Telegram migrated-history validation, command registration, TS contracts, value registry, and focused tests.

**Red-flag scan:** The plan avoids unresolved markers and gives exact file paths, command names, test names, SQL, Rust signatures, TypeScript interfaces, and expected command outcomes.

**Type consistency:** Rust `ProjectStatus` maps to TS `ProjectStatus`; Rust `ProjectDataRange { from, to }` maps to TS `ProjectDataRange`; project source `sync_status` reuses `LibraryCatalogStatus` in Rust and TypeScript; invoke wrapper argument names use camelCase for Tauri calls matching existing `projects.ts` style.
