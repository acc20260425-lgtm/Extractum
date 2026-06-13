# Library Source Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enrich `/projects/library` with a dedicated `list_library_sources` read model, active subtype filters, source metadata columns, and an Inspector that shows YouTube and Telegram details.

**Architecture:** Add a backend-only read model over existing `sources`, `items`, `analysis_source_group_members`, `youtube_video_sources`, and `youtube_playlist_sources` tables. Keep `list_analysis_sources` unchanged for analysis/connect flows. Add a Library-specific frontend API, workflow, and catalog view model so `/projects/library` no longer depends on project-selection rules from Connect from library.

**Tech Stack:** Rust, Tauri 2 commands, SQLx/SQLite, Svelte 5, SvelteKit SPA/Tauri, TypeScript, Vitest, existing `extractum-ui` grid wrappers.

---

## Approved Design Inputs

- Spec: `docs/superpowers/specs/2026-06-13-library-source-metadata-design.md`
- Existing prototype plan: `docs/superpowers/plans/2026-06-13-library-prototype.md`
- Current Library route: `src/routes/projects/library/+page.svelte`
- Current Library components: `src/lib/components/research-projects/Library*.svelte`
- Current connect/project workflow: `src/lib/ui/research-projects-workflow.ts`
- Current prototype model helpers: `src/lib/ui/research-projects-model.ts`

## Scope Check

This plan is one vertical slice: read enriched source metadata and show it on `/projects/library`.

It does not add source CRUD, a durable `library_sources` table, YouTube channel ingestion, full project link details, or stale/freshness policy.

## File Structure

### Backend

- Create: `src-tauri/src/library_sources/models.rs`
  - Serializes the new `LibrarySourceRecord` contract.
- Create: `src-tauri/src/library_sources/mod.rs`
  - Owns the `list_library_sources` Tauri command and the testable `query_library_sources(&SqlitePool)` function.
- Modify: `src-tauri/src/lib.rs`
  - Registers `mod library_sources`, imports `list_library_sources`, and adds it to `tauri::generate_handler!`.

### Frontend API And Types

- Create: `src/lib/types/library-sources.ts`
  - Mirrors the backend record contract.
- Create: `src/lib/api/library-sources.ts`
  - Wraps `invoke<LibrarySourceRecord[]>("list_library_sources")`.
- Create: `src/lib/api/library-sources.test.ts`
  - Verifies the invoke command name.

### Frontend Catalog Model And Workflow

- Create: `src/lib/ui/library-catalog-model.ts`
  - Converts `LibrarySourceRecord[]` plus `SourceJobRecord[]` into Library table/Inspector rows.
  - Builds provider/subtype filter tree rows.
  - Filters rows by selected tree row and search query.
  - Reconciles row selection after filters change.
- Create: `src/lib/ui/library-catalog-model.test.ts`
  - Covers labels, status derivation, subtype tree counts, filtering, nullable detail fields, and selection reconciliation.
- Create: `src/lib/ui/library-catalog-workflow.ts`
  - Loads `listLibrarySources()` and `listSourceJobs()` for Library only.
- Create: `src/lib/ui/library-catalog-workflow.test.ts`
  - Covers load success and load failure.

### Library Components

- Modify: `src/routes/projects/library/+page.svelte`
  - Use `createLibraryCatalogWorkflow`, not `createResearchProjectsWorkflow`.
- Modify: `src/lib/components/research-projects/LibraryScreen.svelte`
  - Use `LibraryCatalogWorkflowState` and new catalog model helpers.
- Modify: `src/lib/components/research-projects/LibraryFilterRail.svelte`
  - Import `LibraryCatalogFilterId` and `LibraryCatalogFilterTreeRow`.
- Modify: `src/lib/components/research-projects/LibraryWorkspace.svelte`
  - Use `LibraryCatalogSourceView`.
  - Update columns to Source, Type, Status, Projects, Items, Added, Last synced.
- Modify: `src/lib/components/research-projects/LibraryInspector.svelte`
  - Show canonical URL, external id, added at, last synced, counts, and provider detail blocks.
  - Treat nullable detail fields consistently.
- Modify: `src/lib/components/research-projects/LibrarySourceCell.svelte`
  - Keep source title/subtitle rendering, with no project-flow fallback text.

### Contracts

- Modify: `src/lib/library-prototype-contract.test.ts`
  - Update route expectations from `listAnalysisSources` to `listLibrarySources`.
  - Assert table metadata columns and Inspector metadata labels.
- Keep: `src/lib/research-projects-import-boundary.test.ts`
  - No direct shadcn/SVAR imports from Library feature files.

---

## Task 0: Baseline

**Files:**
- No file changes.

- [ ] **Step 1: Confirm branch and clean worktree**

Run:

```powershell
git status --short --branch
```

Expected: current branch is `feature/library-prototype` and no modified files are listed.

- [ ] **Step 2: Run focused baseline tests**

Run:

```powershell
npm.cmd run test -- src/lib/library-prototype-contract.test.ts src/lib/research-projects-import-boundary.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts
```

Expected: PASS. If these fail before changes, stop and record the existing failure.

- [ ] **Step 3: Run Rust baseline for nearby source modules**

Run:

```powershell
cargo test library_sources --manifest-path src-tauri/Cargo.toml
```

Expected: no matching tests yet or PASS. This establishes the command that later runs the new backend tests.

---

## Task 1: Backend Library Read Model

**Files:**
- Create: `src-tauri/src/library_sources/models.rs`
- Create: `src-tauri/src/library_sources/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the backend model and query tests**

Create `src-tauri/src/library_sources/models.rs` with the public contract:

```rust
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryYoutubeSourceDetails {
    pub video_form: Option<String>,
    pub duration_seconds: Option<i64>,
    pub playlist_video_count: Option<i64>,
    pub channel_title: Option<String>,
    pub availability_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryTelegramSourceDetails {
    pub account_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibrarySourceRecord {
    pub source_id: i64,
    pub provider: String,
    pub source_subtype: Option<String>,
    pub account_id: Option<i64>,
    pub external_id: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub canonical_url: Option<String>,
    pub created_at: i64,
    pub last_synced_at: Option<i64>,
    pub item_count: i64,
    pub project_count: i64,
    pub youtube: Option<LibraryYoutubeSourceDetails>,
    pub telegram: Option<LibraryTelegramSourceDetails>,
}

#[derive(Debug, FromRow)]
pub(crate) struct LibrarySourceRow {
    pub(crate) source_id: i64,
    pub(crate) provider: String,
    pub(crate) source_subtype: Option<String>,
    pub(crate) account_id: Option<i64>,
    pub(crate) external_id: Option<String>,
    pub(crate) source_title: Option<String>,
    pub(crate) created_at: i64,
    pub(crate) last_synced_at: Option<i64>,
    pub(crate) item_count: i64,
    pub(crate) project_count: i64,
    pub(crate) video_title: Option<String>,
    pub(crate) video_canonical_url: Option<String>,
    pub(crate) video_channel_title: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) video_form: Option<String>,
    pub(crate) video_availability_status: Option<String>,
    pub(crate) playlist_title: Option<String>,
    pub(crate) playlist_canonical_url: Option<String>,
    pub(crate) playlist_channel_title: Option<String>,
    pub(crate) playlist_video_count: Option<i64>,
    pub(crate) playlist_availability_status: Option<String>,
}
```

Create `src-tauri/src/library_sources/mod.rs` with the module skeleton and these tests:

```rust
mod models;

use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{require_source_identity_ready, SourceIdentityRepairState};

pub use models::{LibrarySourceRecord, LibraryTelegramSourceDetails, LibraryYoutubeSourceDetails};
use models::LibrarySourceRow;

#[tauri::command]
pub async fn list_library_sources(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
) -> AppResult<Vec<LibrarySourceRecord>> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    query_library_sources(&pool).await
}

pub(crate) async fn query_library_sources(
    pool: &sqlx::SqlitePool,
) -> AppResult<Vec<LibrarySourceRecord>> {
    let rows: Vec<LibrarySourceRow> = sqlx::query_as(LIBRARY_SOURCES_SQL)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    Ok(rows.into_iter().map(map_library_source_row).collect())
}

const LIBRARY_SOURCES_SQL: &str = r#"
    WITH item_counts AS (
        SELECT source_id, COUNT(content_zstd) AS item_count
        FROM items
        GROUP BY source_id
    ),
    project_counts AS (
        SELECT source_id, COUNT(DISTINCT group_id) AS project_count
        FROM analysis_source_group_members
        GROUP BY source_id
    )
    SELECT
        s.id AS source_id,
        s.source_type AS provider,
        s.source_subtype,
        s.account_id,
        s.external_id,
        s.title AS source_title,
        s.created_at,
        s.last_synced_at,
        COALESCE(item_counts.item_count, 0) AS item_count,
        COALESCE(project_counts.project_count, 0) AS project_count,
        yvs.title AS video_title,
        yvs.canonical_url AS video_canonical_url,
        yvs.channel_title AS video_channel_title,
        yvs.duration_seconds,
        yvs.video_form,
        yvs.availability_status AS video_availability_status,
        yps.title AS playlist_title,
        yps.canonical_url AS playlist_canonical_url,
        yps.channel_title AS playlist_channel_title,
        yps.video_count AS playlist_video_count,
        yps.availability_status AS playlist_availability_status
    FROM sources s
    LEFT JOIN item_counts ON item_counts.source_id = s.id
    LEFT JOIN project_counts ON project_counts.source_id = s.id
    LEFT JOIN youtube_video_sources yvs
        ON yvs.source_id = s.id
        AND s.source_type = 'youtube'
        AND s.source_subtype = 'video'
    LEFT JOIN youtube_playlist_sources yps
        ON yps.source_id = s.id
        AND s.source_type = 'youtube'
        AND s.source_subtype = 'playlist'
    ORDER BY s.created_at DESC, s.id DESC
"#;

fn map_library_source_row(row: LibrarySourceRow) -> LibrarySourceRecord {
    let youtube = match (row.provider.as_str(), row.source_subtype.as_deref()) {
        ("youtube", "video")
            if row.video_title.is_some()
                || row.video_canonical_url.is_some()
                || row.video_channel_title.is_some()
                || row.duration_seconds.is_some()
                || row.video_form.is_some()
                || row.video_availability_status.is_some() =>
        {
            Some(LibraryYoutubeSourceDetails {
                video_form: row.video_form.clone(),
                duration_seconds: row.duration_seconds,
                playlist_video_count: None,
                channel_title: row.video_channel_title.clone(),
                availability_status: row.video_availability_status.clone(),
            })
        }
        ("youtube", "playlist")
            if row.playlist_title.is_some()
                || row.playlist_canonical_url.is_some()
                || row.playlist_channel_title.is_some()
                || row.playlist_video_count.is_some()
                || row.playlist_availability_status.is_some() =>
        {
            Some(LibraryYoutubeSourceDetails {
                video_form: None,
                duration_seconds: None,
                playlist_video_count: row.playlist_video_count,
                channel_title: row.playlist_channel_title.clone(),
                availability_status: row.playlist_availability_status.clone(),
            })
        }
        _ => None,
    };

    let telegram = if row.provider == "telegram" {
        Some(LibraryTelegramSourceDetails {
            account_id: row.account_id,
        })
    } else {
        None
    };

    let title = match row.source_subtype.as_deref() {
        Some("video") => row.video_title.clone().or_else(|| row.source_title.clone()),
        Some("playlist") => row.playlist_title.clone().or_else(|| row.source_title.clone()),
        _ => row.source_title.clone(),
    };
    let subtitle = match row.source_subtype.as_deref() {
        Some("video") => row.video_channel_title.clone(),
        Some("playlist") => row.playlist_channel_title.clone(),
        _ => row.account_id.map(|account_id| format!("Account #{account_id}")),
    };
    let canonical_url = match row.source_subtype.as_deref() {
        Some("video") => row.video_canonical_url.clone(),
        Some("playlist") => row.playlist_canonical_url.clone(),
        _ => None,
    };

    LibrarySourceRecord {
        source_id: row.source_id,
        provider: row.provider,
        source_subtype: row.source_subtype,
        account_id: row.account_id,
        external_id: row.external_id,
        title,
        subtitle,
        canonical_url,
        created_at: row.created_at,
        last_synced_at: row.last_synced_at,
        item_count: row.item_count,
        project_count: row.project_count,
        youtube,
        telegram,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        create_schema(&pool).await;
        pool
    }

    async fn create_schema(pool: &sqlx::SqlitePool) {
        for statement in [
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                source_type TEXT NOT NULL,
                source_subtype TEXT,
                account_id INTEGER,
                external_id TEXT,
                title TEXT,
                last_synced_at INTEGER,
                created_at INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE items (
                id INTEGER PRIMARY KEY,
                source_id INTEGER NOT NULL,
                content_zstd BLOB
            )
            "#,
            r#"
            CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE analysis_source_group_members (
                group_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE youtube_video_sources (
                source_id INTEGER PRIMARY KEY,
                video_id TEXT NOT NULL,
                canonical_url TEXT,
                title TEXT,
                channel_title TEXT,
                duration_seconds INTEGER,
                video_form TEXT,
                availability_status TEXT
            )
            "#,
            r#"
            CREATE TABLE youtube_playlist_sources (
                source_id INTEGER PRIMARY KEY,
                playlist_id TEXT NOT NULL,
                canonical_url TEXT,
                title TEXT,
                channel_title TEXT,
                video_count INTEGER,
                availability_status TEXT
            )
            "#,
        ] {
            sqlx::query(statement)
                .execute(pool)
                .await
                .expect("create library source test schema");
        }
    }

    async fn insert_source(
        pool: &sqlx::SqlitePool,
        id: i64,
        provider: &str,
        subtype: Option<&str>,
        account_id: Option<i64>,
        external_id: &str,
        title: &str,
        created_at: i64,
        last_synced_at: Option<i64>,
    ) {
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id,
                title, created_at, last_synced_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(provider)
        .bind(subtype)
        .bind(account_id)
        .bind(external_id)
        .bind(title)
        .bind(created_at)
        .bind(last_synced_at)
        .execute(pool)
        .await
        .expect("insert source");
    }

    #[tokio::test]
    async fn list_library_sources_returns_youtube_and_telegram_metadata() {
        let pool = memory_pool().await;
        insert_source(&pool, 1, "youtube", Some("video"), None, "vid-1", "Fallback video", 100, Some(200)).await;
        insert_source(&pool, 2, "youtube", Some("playlist"), None, "pl-1", "Fallback playlist", 101, None).await;
        insert_source(&pool, 3, "telegram", Some("supergroup"), Some(77), "-1007", "Drone Radar", 102, Some(202)).await;

        sqlx::query("INSERT INTO items (id, source_id, content_zstd) VALUES (1, 1, X'01'), (2, 1, X'02'), (3, 3, X'03')")
            .execute(&pool)
            .await
            .expect("insert items");
        sqlx::query("INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at) VALUES (10, 'Project A', 'youtube', 1, 1), (11, 'Project B', 'youtube', 1, 1)")
            .execute(&pool)
            .await
            .expect("insert groups");
        sqlx::query("INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (10, 1, 1), (11, 1, 1), (10, 3, 1)")
            .execute(&pool)
            .await
            .expect("insert members");
        sqlx::query(
            r#"
            INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, channel_title,
                duration_seconds, video_form, availability_status
            )
            VALUES (1, 'vid-1', 'https://youtu.be/vid-1', 'Video title', NULL, 321, 'short', 'available')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert video metadata");
        sqlx::query(
            r#"
            INSERT INTO youtube_playlist_sources (
                source_id, playlist_id, canonical_url, title, channel_title,
                video_count, availability_status
            )
            VALUES (2, 'pl-1', 'https://www.youtube.com/playlist?list=pl-1', 'Playlist title', 'Channel B', 44, 'available')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert playlist metadata");

        let rows = query_library_sources(&pool).await.expect("list library sources");

        assert_eq!(rows.iter().map(|row| row.source_id).collect::<Vec<_>>(), vec![3, 2, 1]);

        let video = rows.iter().find(|row| row.source_id == 1).expect("video source");
        assert_eq!(video.source_subtype.as_deref(), Some("video"));
        assert_eq!(video.title.as_deref(), Some("Video title"));
        assert_eq!(video.subtitle, None);
        assert_eq!(video.canonical_url.as_deref(), Some("https://youtu.be/vid-1"));
        assert_eq!(video.item_count, 2);
        assert_eq!(video.project_count, 2);
        assert_eq!(
            video.youtube,
            Some(LibraryYoutubeSourceDetails {
                video_form: Some("short".to_string()),
                duration_seconds: Some(321),
                playlist_video_count: None,
                channel_title: None,
                availability_status: Some("available".to_string()),
            })
        );

        let playlist = rows.iter().find(|row| row.source_id == 2).expect("playlist source");
        assert_eq!(playlist.source_subtype.as_deref(), Some("playlist"));
        assert_eq!(playlist.title.as_deref(), Some("Playlist title"));
        assert_eq!(playlist.subtitle.as_deref(), Some("Channel B"));
        assert_eq!(playlist.item_count, 0);
        assert_eq!(playlist.project_count, 0);
        assert_eq!(playlist.youtube.as_ref().and_then(|details| details.playlist_video_count), Some(44));

        let telegram = rows.iter().find(|row| row.source_id == 3).expect("telegram source");
        assert_eq!(telegram.source_subtype.as_deref(), Some("supergroup"));
        assert_eq!(telegram.subtitle.as_deref(), Some("Account #77"));
        assert_eq!(telegram.telegram, Some(LibraryTelegramSourceDetails { account_id: Some(77) }));
    }

    #[tokio::test]
    async fn list_library_sources_keeps_sources_with_missing_provider_details() {
        let pool = memory_pool().await;
        insert_source(&pool, 5, "youtube", Some("video"), None, "missing-video", "Stored title", 500, None).await;

        let rows = query_library_sources(&pool).await.expect("list library sources");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source_id, 5);
        assert_eq!(rows[0].title.as_deref(), Some("Stored title"));
        assert_eq!(rows[0].canonical_url, None);
        assert_eq!(rows[0].youtube, None);
    }
}
```

- [ ] **Step 2: Run the new backend tests and verify compile/test failure**

Run:

```powershell
cargo test library_sources --manifest-path src-tauri/Cargo.toml
```

Expected: FAIL until `src-tauri/src/lib.rs` declares the new module, or PASS if the module is compiled as soon as it is declared. If it fails only because the module is not wired, continue to Step 3.

- [ ] **Step 3: Wire the backend module into Tauri**

Modify `src-tauri/src/lib.rs`.

Add near other module declarations:

```rust
mod library_sources;
use library_sources::list_library_sources;
```

Add `list_library_sources` to the `tauri::generate_handler!` macro near the analysis source commands:

```rust
            list_analysis_sources,
            list_library_sources,
            list_analysis_prompt_templates,
```

- [ ] **Step 4: Run the backend tests and verify they pass**

Run:

```powershell
cargo test library_sources --manifest-path src-tauri/Cargo.toml
```

Expected: PASS for both `library_sources` tests.

- [ ] **Step 5: Commit backend read model**

Run:

```powershell
git add src-tauri/src/library_sources/models.rs src-tauri/src/library_sources/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add library source read model"
```

---

## Task 2: Frontend Library API And Wire Types

**Files:**
- Create: `src/lib/types/library-sources.ts`
- Create: `src/lib/api/library-sources.ts`
- Create: `src/lib/api/library-sources.test.ts`

- [ ] **Step 1: Write the failing API wrapper test**

Create `src/lib/api/library-sources.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import { listLibrarySources } from "./library-sources";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("library source api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("lists enriched library source records", async () => {
    const records = [
      {
        source_id: 1,
        provider: "youtube",
        source_subtype: "video",
        account_id: null,
        external_id: "vid-1",
        title: "Video title",
        subtitle: "Channel title",
        canonical_url: "https://youtu.be/vid-1",
        created_at: 1_717_000_000,
        last_synced_at: 1_717_000_100,
        item_count: 12,
        project_count: 2,
        youtube: {
          video_form: "short",
          duration_seconds: 45,
          playlist_video_count: null,
          channel_title: "Channel title",
          availability_status: "available",
        },
        telegram: null,
      },
    ];
    invokeMock.mockResolvedValueOnce(records);

    await expect(listLibrarySources()).resolves.toEqual(records);

    expect(invokeMock).toHaveBeenLastCalledWith("list_library_sources");
  });
});
```

- [ ] **Step 2: Run the API test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/api/library-sources.test.ts
```

Expected: FAIL because `src/lib/api/library-sources.ts` does not exist.

- [ ] **Step 3: Add Library source types**

Create `src/lib/types/library-sources.ts`:

```ts
export type LibrarySourceProvider = "telegram" | "youtube" | "rss" | "forum" | "web" | "other";

export type LibrarySourceSubtype =
  | "video"
  | "playlist"
  | "channel"
  | "supergroup"
  | "group"
  | "feed"
  | "thread"
  | "board"
  | "site"
  | null;

export interface LibraryYoutubeSourceDetails {
  video_form: string | null;
  duration_seconds: number | null;
  playlist_video_count: number | null;
  channel_title: string | null;
  availability_status: string | null;
}

export interface LibraryTelegramSourceDetails {
  account_id: number | null;
}

export interface LibrarySourceRecord {
  source_id: number;
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  account_id: number | null;
  external_id: string | null;
  title: string | null;
  subtitle: string | null;
  canonical_url: string | null;
  created_at: number;
  last_synced_at: number | null;
  item_count: number;
  project_count: number;
  youtube: LibraryYoutubeSourceDetails | null;
  telegram: LibraryTelegramSourceDetails | null;
}
```

- [ ] **Step 4: Add the API wrapper**

Create `src/lib/api/library-sources.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { LibrarySourceRecord } from "$lib/types/library-sources";

export function listLibrarySources() {
  return invoke<LibrarySourceRecord[]>("list_library_sources");
}
```

- [ ] **Step 5: Run the API test and verify it passes**

Run:

```powershell
npm.cmd run test -- src/lib/api/library-sources.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit API and types**

Run:

```powershell
git add src/lib/types/library-sources.ts src/lib/api/library-sources.ts src/lib/api/library-sources.test.ts
git commit -m "feat: add library source api wrapper"
```

---

## Task 3: Library Catalog View Model

**Files:**
- Create: `src/lib/ui/library-catalog-model.ts`
- Create: `src/lib/ui/library-catalog-model.test.ts`

- [ ] **Step 1: Write catalog model tests**

Create `src/lib/ui/library-catalog-model.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { SourceJobRecord } from "$lib/types/sources";
import type { LibrarySourceRecord } from "$lib/types/library-sources";
import {
  LIBRARY_CATALOG_ALL_FILTER_ID,
  buildLibraryCatalogFilterTree,
  buildLibraryCatalogSourcesView,
  filterLibraryCatalogSources,
  reconcileLibraryCatalogSourceSelection,
} from "./library-catalog-model";

function record(overrides: Partial<LibrarySourceRecord> = {}): LibrarySourceRecord {
  return {
    source_id: 1,
    provider: "telegram",
    source_subtype: "supergroup",
    account_id: 10,
    external_id: "-1001",
    title: "Radar BPLA",
    subtitle: "Account #10",
    canonical_url: null,
    created_at: 1_716_000_000,
    last_synced_at: 1_717_000_000,
    item_count: 128,
    project_count: 2,
    youtube: null,
    telegram: { account_id: 10 },
    ...overrides,
  };
}

function job(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 3,
    related_source_id: null,
    job_type: "youtube_video_full_sync",
    status: "running",
    message: "Syncing",
    progress_current: 1,
    progress_total: 3,
    started_at: 1_717_000_100,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

describe("library catalog model", () => {
  it("maps source metadata into catalog rows without project-connect state", () => {
    const [row] = buildLibraryCatalogSourcesView([
      record({
        source_id: 3,
        provider: "youtube",
        source_subtype: "video",
        title: "Alpha Drones",
        subtitle: null,
        canonical_url: "https://youtu.be/alpha",
        external_id: "alpha",
        item_count: 7,
        project_count: 1,
        youtube: {
          video_form: "longform",
          duration_seconds: 367,
          playlist_video_count: null,
          channel_title: null,
          availability_status: "available",
        },
        telegram: null,
      }),
    ], []);

    expect(row).toEqual(expect.objectContaining({
      id: "source:3",
      sourceId: 3,
      provider: "youtube",
      sourceSubtype: "video",
      title: "Alpha Drones",
      subtitle: null,
      typeLabel: "YouTube / Video",
      status: "active",
      projectCount: 1,
      itemCount: 7,
      itemCountLabel: "7 items",
      addedAtLabel: expect.any(String),
      lastSyncedLabel: expect.any(String),
      canonicalUrl: "https://youtu.be/alpha",
      externalId: "alpha",
      youtube: {
        video_form: "longform",
        duration_seconds: 367,
        playlist_video_count: null,
        channel_title: null,
        availability_status: "available",
      },
      telegram: null,
    }));
  });

  it("derives syncing and failed status from the latest source job", () => {
    const rows = buildLibraryCatalogSourcesView([
      record({ source_id: 3, provider: "youtube", source_subtype: "video", title: "Running" }),
      record({ source_id: 4, provider: "youtube", source_subtype: "video", title: "Failed" }),
    ], [
      job({ source_id: 3, status: "running", started_at: 20 }),
      job({ source_id: 4, status: "failed", error: "Quota", started_at: 10 }),
    ]);

    expect(rows.find((row) => row.sourceId === 3)?.status).toBe("syncing");
    expect(rows.find((row) => row.sourceId === 4)?.status).toBe("error");
    expect(rows.find((row) => row.sourceId === 4)?.statusDetail).toBe("Quota");
  });

  it("builds active subtype filters for YouTube and Telegram while keeping YouTube channels disabled", () => {
    const rows = buildLibraryCatalogSourcesView([
      record({ source_id: 1, provider: "youtube", source_subtype: "video", title: "Video" }),
      record({ source_id: 2, provider: "youtube", source_subtype: "playlist", title: "Playlist" }),
      record({ source_id: 3, provider: "telegram", source_subtype: "channel", title: "Channel" }),
      record({ source_id: 4, provider: "telegram", source_subtype: "supergroup", title: "Supergroup" }),
      record({ source_id: 5, provider: "telegram", source_subtype: "group", title: "Group" }),
    ], []);

    expect(buildLibraryCatalogFilterTree(rows)).toEqual([
      expect.objectContaining({ id: "all", count: 5 }),
      expect.objectContaining({
        id: "provider:youtube",
        count: 2,
        data: [
          expect.objectContaining({ id: "provider:youtube/subtype:video", count: 1, disabled: false }),
          expect.objectContaining({ id: "provider:youtube/subtype:playlist", count: 1, disabled: false }),
          expect.objectContaining({ id: "provider:youtube/subtype:channel", count: 0, disabled: true }),
        ],
      }),
      expect.objectContaining({
        id: "provider:telegram",
        count: 3,
        data: [
          expect.objectContaining({ id: "provider:telegram/subtype:channel", count: 1, disabled: false }),
          expect.objectContaining({ id: "provider:telegram/subtype:supergroup", count: 1, disabled: false }),
          expect.objectContaining({ id: "provider:telegram/subtype:group", count: 1, disabled: false }),
        ],
      }),
    ]);
  });

  it("filters by selected provider subtype and search query", () => {
    const rows = buildLibraryCatalogSourcesView([
      record({ source_id: 1, provider: "youtube", source_subtype: "video", title: "Alpha Video" }),
      record({ source_id: 2, provider: "youtube", source_subtype: "playlist", title: "Alpha Playlist" }),
      record({ source_id: 3, provider: "telegram", source_subtype: "channel", title: "Alpha Channel" }),
    ], []);

    expect(filterLibraryCatalogSources(rows, { filterId: LIBRARY_CATALOG_ALL_FILTER_ID, query: "alpha" }).map((row) => row.id))
      .toEqual(["source:1", "source:2", "source:3"]);
    expect(filterLibraryCatalogSources(rows, { filterId: "provider:youtube/subtype:video", query: "" }).map((row) => row.id))
      .toEqual(["source:1"]);
    expect(filterLibraryCatalogSources(rows, { filterId: "provider:telegram/subtype:channel", query: "" }).map((row) => row.id))
      .toEqual(["source:3"]);
  });

  it("reconciles selected rows after filtering", () => {
    const rows = buildLibraryCatalogSourcesView([
      record({ source_id: 1, title: "First" }),
      record({ source_id: 2, title: "Second" }),
    ], []);

    expect(reconcileLibraryCatalogSourceSelection(rows, "source:2")).toBe("source:2");
    expect(reconcileLibraryCatalogSourceSelection([rows[0]], "source:2")).toBe("source:1");
    expect(reconcileLibraryCatalogSourceSelection([], "source:2")).toBeNull();
  });
});
```

- [ ] **Step 2: Run catalog model tests and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-catalog-model.test.ts
```

Expected: FAIL because `src/lib/ui/library-catalog-model.ts` does not exist.

- [ ] **Step 3: Implement the catalog model**

Create `src/lib/ui/library-catalog-model.ts`:

```ts
import type {
  LibrarySourceProvider,
  LibrarySourceRecord,
  LibrarySourceSubtype,
  LibraryTelegramSourceDetails,
  LibraryYoutubeSourceDetails,
} from "$lib/types/library-sources";
import type { SourceJobRecord } from "$lib/types/sources";

export type LibraryCatalogSourceStatus = "active" | "syncing" | "error" | "unavailable";

export type LibraryCatalogSourceView = {
  id: string;
  sourceId: number;
  provider: LibrarySourceProvider;
  sourceSubtype: LibrarySourceSubtype;
  title: string;
  subtitle: string | null;
  typeLabel: string;
  status: LibraryCatalogSourceStatus;
  statusDetail: string | null;
  projectCount: number;
  itemCount: number;
  itemCountLabel: string;
  addedAtLabel: string;
  lastSyncedLabel: string;
  canonicalUrl: string | null;
  externalId: string | null;
  youtube: LibraryYoutubeSourceDetails | null;
  telegram: LibraryTelegramSourceDetails | null;
};

export type LibraryCatalogFilterId =
  | "all"
  | `provider:${LibrarySourceProvider}`
  | `provider:${LibrarySourceProvider}/subtype:${Exclude<LibrarySourceSubtype, null>}`;

export type LibraryCatalogFilterTreeRow = {
  id: LibraryCatalogFilterId;
  label: string;
  provider: LibrarySourceProvider | "all";
  subtype?: Exclude<LibrarySourceSubtype, null>;
  count: number;
  disabled?: boolean;
  disabledReason?: string;
  data?: LibraryCatalogFilterTreeRow[];
};

export type LibraryCatalogFilterState = {
  filterId: LibraryCatalogFilterId;
  query: string;
};

export const LIBRARY_CATALOG_ALL_FILTER_ID: LibraryCatalogFilterId = "all";
export const YOUTUBE_CHANNEL_DISABLED_REASON =
  "YouTube channel sources are not supported by the current backend.";

const PROVIDER_LABELS: Record<LibrarySourceProvider, string> = {
  telegram: "Telegram",
  youtube: "YouTube",
  rss: "RSS",
  forum: "Forum",
  web: "Web",
  other: "Other",
};

const SUBTYPE_LABELS: Record<Exclude<LibrarySourceSubtype, null>, string> = {
  video: "Video",
  playlist: "Playlist",
  channel: "Channel",
  supergroup: "Supergroup",
  group: "Group",
  feed: "Feed",
  thread: "Thread",
  board: "Board",
  site: "Site",
};

function sourceRowId(sourceId: number) {
  return `source:${sourceId}`;
}

function countLabel(count: number) {
  if (count === 1) return "1 item";
  return `${count} items`;
}

function dateLabel(unixSeconds: number | null) {
  if (!unixSeconds) return null;
  return new Intl.DateTimeFormat("en-US", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(unixSeconds * 1000));
}

function typeLabel(provider: LibrarySourceProvider, subtype: LibrarySourceSubtype) {
  const providerLabel = PROVIDER_LABELS[provider] ?? provider;
  if (!subtype) return `${providerLabel} source`;
  return `${providerLabel} / ${SUBTYPE_LABELS[subtype] ?? subtype}`;
}

function latestJobBySource(sourceJobs: SourceJobRecord[]) {
  const jobsBySource = new Map<number, SourceJobRecord>();
  for (const job of sourceJobs) {
    if (job.status !== "queued" && job.status !== "running" && job.status !== "failed") continue;
    const current = jobsBySource.get(job.source_id);
    if (!current || job.started_at > current.started_at) jobsBySource.set(job.source_id, job);
  }
  return jobsBySource;
}

function statusFromJob(job: SourceJobRecord | undefined): {
  status: LibraryCatalogSourceStatus;
  statusDetail: string | null;
} {
  if (!job) return { status: "active", statusDetail: null };
  if (job.status === "queued" || job.status === "running") {
    return { status: "syncing", statusDetail: job.message ?? "Syncing" };
  }
  if (job.status === "failed") {
    return { status: "error", statusDetail: job.error ?? "Last sync failed" };
  }
  return { status: "active", statusDetail: null };
}

export function buildLibraryCatalogSourcesView(
  records: LibrarySourceRecord[],
  sourceJobs: SourceJobRecord[] = [],
): LibraryCatalogSourceView[] {
  const jobsBySource = latestJobBySource(sourceJobs);

  return records.map((record) => {
    const jobStatus = statusFromJob(jobsBySource.get(record.source_id));
    return {
      id: sourceRowId(record.source_id),
      sourceId: record.source_id,
      provider: record.provider,
      sourceSubtype: record.source_subtype,
      title: record.title ?? `Source #${record.source_id}`,
      subtitle: record.subtitle,
      typeLabel: typeLabel(record.provider, record.source_subtype),
      status: jobStatus.status,
      statusDetail: jobStatus.statusDetail,
      projectCount: record.project_count,
      itemCount: record.item_count,
      itemCountLabel: countLabel(record.item_count),
      addedAtLabel: dateLabel(record.created_at) ?? "Unknown",
      lastSyncedLabel: dateLabel(record.last_synced_at) ?? "Never",
      canonicalUrl: record.canonical_url,
      externalId: record.external_id,
      youtube: record.youtube,
      telegram: record.telegram,
    };
  });
}

function countProvider(sources: LibraryCatalogSourceView[], provider: LibrarySourceProvider) {
  return sources.filter((source) => source.provider === provider).length;
}

function countSubtype(
  sources: LibraryCatalogSourceView[],
  provider: LibrarySourceProvider,
  subtype: Exclude<LibrarySourceSubtype, null>,
) {
  return sources.filter((source) => source.provider === provider && source.sourceSubtype === subtype).length;
}

function subtypeRow(
  sources: LibraryCatalogSourceView[],
  provider: LibrarySourceProvider,
  subtype: Exclude<LibrarySourceSubtype, null>,
  label: string,
  disabled = false,
  disabledReason: string | null = null,
): LibraryCatalogFilterTreeRow {
  return {
    id: `provider:${provider}/subtype:${subtype}` as LibraryCatalogFilterId,
    label,
    provider,
    subtype,
    count: countSubtype(sources, provider, subtype),
    disabled,
    disabledReason: disabledReason ?? undefined,
  };
}

export function buildLibraryCatalogFilterTree(
  sources: LibraryCatalogSourceView[],
): LibraryCatalogFilterTreeRow[] {
  return [
    {
      id: LIBRARY_CATALOG_ALL_FILTER_ID,
      label: "All sources",
      provider: "all",
      count: sources.length,
    },
    {
      id: "provider:youtube",
      label: "YouTube",
      provider: "youtube",
      count: countProvider(sources, "youtube"),
      data: [
        subtypeRow(sources, "youtube", "video", "Videos"),
        subtypeRow(sources, "youtube", "playlist", "Playlists"),
        subtypeRow(sources, "youtube", "channel", "Channels", true, YOUTUBE_CHANNEL_DISABLED_REASON),
      ],
    },
    {
      id: "provider:telegram",
      label: "Telegram",
      provider: "telegram",
      count: countProvider(sources, "telegram"),
      data: [
        subtypeRow(sources, "telegram", "channel", "Channels"),
        subtypeRow(sources, "telegram", "supergroup", "Supergroups"),
        subtypeRow(sources, "telegram", "group", "Groups"),
      ],
    },
  ];
}

function filterParts(filterId: LibraryCatalogFilterId): {
  provider: LibrarySourceProvider | null;
  subtype: Exclude<LibrarySourceSubtype, null> | null;
} {
  if (filterId === LIBRARY_CATALOG_ALL_FILTER_ID) return { provider: null, subtype: null };
  const [providerPart, subtypePart] = filterId.split("/subtype:");
  const provider = providerPart.replace("provider:", "") as LibrarySourceProvider;
  return {
    provider,
    subtype: subtypePart ? (subtypePart as Exclude<LibrarySourceSubtype, null>) : null,
  };
}

export function filterLibraryCatalogSources(
  sources: LibraryCatalogSourceView[],
  filters: LibraryCatalogFilterState,
) {
  const query = filters.query.trim().toLocaleLowerCase();
  const { provider, subtype } = filterParts(filters.filterId);

  return sources.filter((source) => {
    const matchesProvider = !provider || source.provider === provider;
    const matchesSubtype = !subtype || source.sourceSubtype === subtype;
    const matchesQuery =
      !query ||
      `${source.title} ${source.subtitle ?? ""} ${source.typeLabel} ${source.externalId ?? ""}`
        .toLocaleLowerCase()
        .includes(query);
    return matchesProvider && matchesSubtype && matchesQuery;
  });
}

export function reconcileLibraryCatalogSourceSelection(
  sources: LibraryCatalogSourceView[],
  selectedSourceId: string | null,
) {
  if (selectedSourceId && sources.some((source) => source.id === selectedSourceId)) {
    return selectedSourceId;
  }
  return sources[0]?.id ?? null;
}
```

- [ ] **Step 4: Run catalog model tests and verify they pass**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-catalog-model.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit catalog model**

Run:

```powershell
git add src/lib/ui/library-catalog-model.ts src/lib/ui/library-catalog-model.test.ts
git commit -m "feat: add library catalog view model"
```

---

## Task 4: Library Catalog Workflow

**Files:**
- Create: `src/lib/ui/library-catalog-workflow.ts`
- Create: `src/lib/ui/library-catalog-workflow.test.ts`

- [ ] **Step 1: Write workflow tests**

Create `src/lib/ui/library-catalog-workflow.test.ts`:

```ts
import { describe, expect, it, vi } from "vitest";
import type { LibrarySourceRecord } from "$lib/types/library-sources";
import type { SourceJobRecord } from "$lib/types/sources";
import { createLibraryCatalogWorkflow, type LibraryCatalogWorkflowState } from "./library-catalog-workflow";

function record(overrides: Partial<LibrarySourceRecord> = {}): LibrarySourceRecord {
  return {
    source_id: 1,
    provider: "youtube",
    source_subtype: "video",
    account_id: null,
    external_id: "vid-1",
    title: "Video title",
    subtitle: "Channel title",
    canonical_url: "https://youtu.be/vid-1",
    created_at: 1_716_000_000,
    last_synced_at: 1_717_000_000,
    item_count: 10,
    project_count: 2,
    youtube: {
      video_form: "longform",
      duration_seconds: 120,
      playlist_video_count: null,
      channel_title: "Channel title",
      availability_status: "available",
    },
    telegram: null,
    ...overrides,
  };
}

function job(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 1,
    related_source_id: null,
    job_type: "youtube_video_full_sync",
    status: "running",
    message: "Syncing",
    progress_current: 1,
    progress_total: 2,
    started_at: 1_717_000_100,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

function createHarness(initial: Partial<LibraryCatalogWorkflowState> = {}) {
  const state: LibraryCatalogWorkflowState = {
    sourceRecords: [],
    sourceJobs: [],
    sources: [],
    loading: false,
    status: "",
    ...initial,
  };
  const deps = {
    getState: () => state,
    patch: vi.fn((patch: Partial<LibraryCatalogWorkflowState>) => Object.assign(state, patch)),
    listSources: vi.fn(),
    listSourceJobs: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };
  return { state, deps, workflow: createLibraryCatalogWorkflow(deps) };
}

describe("library catalog workflow", () => {
  it("loads library source records and source jobs into catalog rows", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listSources.mockResolvedValueOnce([record()]);
    deps.listSourceJobs.mockResolvedValueOnce([job()]);

    await workflow.loadLibrary();

    expect(state.sourceRecords).toHaveLength(1);
    expect(state.sourceJobs).toHaveLength(1);
    expect(state.sources[0]).toEqual(expect.objectContaining({
      sourceId: 1,
      title: "Video title",
      status: "syncing",
    }));
    expect(state.loading).toBe(false);
    expect(state.status).toBe("");
  });

  it("keeps previous rows and reports a load error", async () => {
    const { state, deps, workflow } = createHarness({
      sources: [{ id: "source:9", sourceId: 9, provider: "telegram", sourceSubtype: "group", title: "Cached", subtitle: null, typeLabel: "Telegram / Group", status: "active", statusDetail: null, projectCount: 0, itemCount: 0, itemCountLabel: "0 items", addedAtLabel: "Unknown", lastSyncedLabel: "Never", canonicalUrl: null, externalId: null, youtube: null, telegram: { account_id: null } }],
    });
    deps.listSources.mockRejectedValueOnce(new Error("offline"));

    await workflow.loadLibrary();

    expect(state.sources.map((source) => source.id)).toEqual(["source:9"]);
    expect(state.status).toBe("Error loading library sources: Error: offline");
    expect(state.loading).toBe(false);
  });
});
```

- [ ] **Step 2: Run workflow tests and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-catalog-workflow.test.ts
```

Expected: FAIL because `src/lib/ui/library-catalog-workflow.ts` does not exist.

- [ ] **Step 3: Implement the workflow**

Create `src/lib/ui/library-catalog-workflow.ts`:

```ts
import type { LibrarySourceRecord } from "$lib/types/library-sources";
import type { SourceJobRecord } from "$lib/types/sources";
import {
  buildLibraryCatalogSourcesView,
  type LibraryCatalogSourceView,
} from "./library-catalog-model";

export interface LibraryCatalogWorkflowState {
  sourceRecords: LibrarySourceRecord[];
  sourceJobs: SourceJobRecord[];
  sources: LibraryCatalogSourceView[];
  loading: boolean;
  status: string;
}

export interface LibraryCatalogWorkflowDeps {
  getState(): LibraryCatalogWorkflowState;
  patch(patch: Partial<LibraryCatalogWorkflowState>): void;
  listSources(): Promise<LibrarySourceRecord[]>;
  listSourceJobs(): Promise<SourceJobRecord[]>;
  formatError(action: string, error: unknown): string;
}

export function createLibraryCatalogWorkflow(deps: LibraryCatalogWorkflowDeps) {
  function refreshDerivedState() {
    const state = deps.getState();
    deps.patch({
      sources: buildLibraryCatalogSourcesView(state.sourceRecords, state.sourceJobs),
    });
  }

  async function loadLibrary() {
    deps.patch({ loading: true, status: "" });
    try {
      const [sourceRecords, sourceJobs] = await Promise.all([
        deps.listSources(),
        deps.listSourceJobs(),
      ]);
      deps.patch({ sourceRecords, sourceJobs });
      refreshDerivedState();
    } catch (error) {
      deps.patch({ status: deps.formatError("loading library sources", error) });
    } finally {
      deps.patch({ loading: false });
    }
  }

  return {
    refreshDerivedState,
    loadLibrary,
  };
}
```

- [ ] **Step 4: Run workflow tests**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-catalog-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit workflow**

Run:

```powershell
git add src/lib/ui/library-catalog-workflow.ts src/lib/ui/library-catalog-workflow.test.ts
git commit -m "feat: add library catalog workflow"
```

---

## Task 5: Library Screen Uses Catalog Rows

**Files:**
- Modify: `src/routes/projects/library/+page.svelte`
- Modify: `src/lib/components/research-projects/LibraryScreen.svelte`
- Modify: `src/lib/components/research-projects/LibraryFilterRail.svelte`
- Modify: `src/lib/components/research-projects/LibrarySourceCell.svelte`
- Modify: `src/lib/library-prototype-contract.test.ts`

- [ ] **Step 1: Update Library route and screen contract expectations**

Modify `src/lib/library-prototype-contract.test.ts`.

In the first test, replace the old workflow expectations:

```ts
    expect(routeSource).toContain("createResearchProjectsWorkflow");
    expect(routeSource).toContain("listAnalysisSources");
```

with:

```ts
    expect(routeSource).toContain("createLibraryCatalogWorkflow");
    expect(routeSource).toContain("listLibrarySources");
```

Modify the final test in `src/lib/library-prototype-contract.test.ts`.

Replace:

```ts
    expect(screenSource).toContain("buildLibraryFilterTree");
    expect(screenSource).toContain("filterLibrarySourcesForLibrary");
    expect(screenSource).toContain("reconcileLibrarySourceSelection");
```

with:

```ts
    expect(screenSource).toContain("buildLibraryCatalogFilterTree");
    expect(screenSource).toContain("filterLibraryCatalogSources");
    expect(screenSource).toContain("reconcileLibraryCatalogSourceSelection");
```

- [ ] **Step 2: Update `/projects/library` route to use the Library workflow**

Replace the script block in `src/routes/projects/library/+page.svelte` with:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import LibraryScreen from "$lib/components/research-projects/LibraryScreen.svelte";
  import { listLibrarySources } from "$lib/api/library-sources";
  import { listSourceJobs } from "$lib/api/source-jobs";
  import {
    createLibraryCatalogWorkflow,
    type LibraryCatalogWorkflowState,
  } from "$lib/ui/library-catalog-workflow";

  const state = $state<LibraryCatalogWorkflowState>({
    sourceRecords: [],
    sourceJobs: [],
    sources: [],
    loading: false,
    status: "",
  });

  const workflow = createLibraryCatalogWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listSources: listLibrarySources,
    listSourceJobs: () => listSourceJobs({ limit: 50 }),
    formatError: (action, error) => `Error ${action}: ${String(error)}`,
  });

  onMount(() => {
    void workflow.loadLibrary();
  });
</script>
```

Keep the markup as:

```svelte
<section data-ui-route="library-prototype">
  <LibraryScreen {state} onRefresh={workflow.loadLibrary} />
</section>
```

- [ ] **Step 3: Update `LibraryScreen.svelte` imports and state type**

In `src/lib/components/research-projects/LibraryScreen.svelte`, replace the model imports with:

```svelte
  import {
    LIBRARY_CATALOG_ALL_FILTER_ID,
    buildLibraryCatalogFilterTree,
    filterLibraryCatalogSources,
    reconcileLibraryCatalogSourceSelection,
    type LibraryCatalogFilterId,
  } from "$lib/ui/library-catalog-model";
  import type { LibraryCatalogWorkflowState } from "$lib/ui/library-catalog-workflow";
```

Replace the props type:

```svelte
    state: LibraryCatalogWorkflowState;
```

Replace local state initialization:

```svelte
  let selectedFilterId = $state<LibraryCatalogFilterId>(LIBRARY_CATALOG_ALL_FILTER_ID);
```

Replace derived rows:

```svelte
  let filterRows = $derived(buildLibraryCatalogFilterTree(workflowState.sources));
  let visibleSources = $derived(
    filterLibraryCatalogSources(workflowState.sources, { filterId: selectedFilterId, query }),
  );
```

Replace selection reconciliation:

```svelte
    const nextSelectedId = reconcileLibraryCatalogSourceSelection(visibleSources, selectedSourceId);
```

- [ ] **Step 4: Update `LibraryFilterRail.svelte` imports and prop types**

In `src/lib/components/research-projects/LibraryFilterRail.svelte`, replace:

```svelte
  import type { LibraryFilterId, LibraryFilterTreeRow } from "$lib/ui/research-projects-model";
```

with:

```svelte
  import type {
    LibraryCatalogFilterId,
    LibraryCatalogFilterTreeRow,
  } from "$lib/ui/library-catalog-model";
```

Replace prop types:

```svelte
    rows: LibraryCatalogFilterTreeRow[];
    selectedFilterId: LibraryCatalogFilterId;
    onSelectedFilterIdChange: (id: LibraryCatalogFilterId) => void;
```

Replace the callback cast:

```svelte
      if (id) onSelectedFilterIdChange(id as LibraryCatalogFilterId);
```

- [ ] **Step 5: Remove project-flow fallback text from `LibrarySourceCell.svelte`**

Replace the `subtitle` derived expression with:

```svelte
  let subtitle = $derived(
    typeof row.subtitle === "string" && row.subtitle.length > 0
      ? row.subtitle
      : typeof row.typeLabel === "string"
        ? row.typeLabel
        : "",
  );
```

- [ ] **Step 6: Run focused component contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/library-prototype-contract.test.ts src/lib/ui/library-catalog-model.test.ts src/lib/ui/library-catalog-workflow.test.ts
```

Expected: PASS after Task 5 changes.

- [ ] **Step 7: Run Svelte check for type errors**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 8: Commit screen catalog switch**

Run:

```powershell
git add src/routes/projects/library/+page.svelte src/lib/components/research-projects/LibraryScreen.svelte src/lib/components/research-projects/LibraryFilterRail.svelte src/lib/components/research-projects/LibrarySourceCell.svelte src/lib/library-prototype-contract.test.ts
git commit -m "feat: switch library screen to catalog rows"
```

---

## Task 6: Library Table Columns And Inspector Metadata

**Files:**
- Modify: `src/lib/components/research-projects/LibraryWorkspace.svelte`
- Modify: `src/lib/components/research-projects/LibraryInspector.svelte`
- Modify: `src/lib/library-prototype-contract.test.ts`

- [ ] **Step 1: Extend component contract tests**

Modify the workspace test in `src/lib/library-prototype-contract.test.ts` to include metadata columns:

```ts
    expect(workspaceSource).toContain('header: "Source"');
    expect(workspaceSource).toContain('header: "Type"');
    expect(workspaceSource).toContain('header: "Status"');
    expect(workspaceSource).toContain('header: "Projects"');
    expect(workspaceSource).toContain('header: "Items"');
    expect(workspaceSource).toContain('header: "Added"');
    expect(workspaceSource).toContain('header: "Last synced"');
```

Modify the Inspector test to include metadata labels:

```ts
    expect(inspectorSource).toContain("Canonical URL");
    expect(inspectorSource).toContain("External ID");
    expect(inspectorSource).toContain("Added");
    expect(inspectorSource).toContain("Last synced");
    expect(inspectorSource).toContain("YouTube details");
    expect(inspectorSource).toContain("Telegram details");
    expect(inspectorSource).toContain("metadataRows");
```

- [ ] **Step 2: Update `LibraryWorkspace.svelte` to use catalog rows and new columns**

Replace the type import:

```svelte
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
```

Replace prop types:

```svelte
    sources: LibraryCatalogSourceView[];
    selectedSource: LibraryCatalogSourceView | null;
```

Replace `columns` with:

```svelte
  const columns = [
    { id: "title", header: "Source", flexgrow: 1, cell: LibrarySourceCell },
    { id: "typeLabel", header: "Type", width: 150 },
    { id: "status", header: "Status", width: 110 },
    { id: "projectCount", header: "Projects", width: 92 },
    { id: "itemCountLabel", header: "Items", width: 100 },
    { id: "addedAtLabel", header: "Added", width: 136 },
    { id: "lastSyncedLabel", header: "Last synced", width: 136 },
  ];
```

- [ ] **Step 3: Update `LibraryInspector.svelte` type import and nullable helpers**

Replace the type import with:

```svelte
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
```

Replace the prop type:

```svelte
  let { selectedSource }: { selectedSource: LibraryCatalogSourceView | null } = $props();
```

Add helper functions below props:

```svelte
  type MetadataRow = { label: string; value: string | null; href?: string };

  function present(value: string | number | null | undefined) {
    if (value === null || value === undefined || value === "") return null;
    return String(value);
  }

  function secondsLabel(seconds: number | null | undefined) {
    if (seconds === null || seconds === undefined) return null;
    const minutes = Math.floor(seconds / 60);
    const remainder = seconds % 60;
    if (minutes <= 0) return `${remainder}s`;
    return `${minutes}m ${remainder}s`;
  }

  let metadataRows = $derived<MetadataRow[]>(
    selectedSource
      ? [
          { label: "Source ID", value: String(selectedSource.sourceId) },
          { label: "Type", value: selectedSource.typeLabel },
          {
            label: "Canonical URL",
            value: present(selectedSource.canonicalUrl),
            href: selectedSource.canonicalUrl ?? undefined,
          },
          { label: "External ID", value: present(selectedSource.externalId) },
          { label: "Added", value: selectedSource.addedAtLabel },
          { label: "Last synced", value: selectedSource.lastSyncedLabel },
          { label: "Items", value: selectedSource.itemCountLabel },
          { label: "Projects", value: String(selectedSource.projectCount) },
        ]
      : [],
  );

  let youtubeRows = $derived<MetadataRow[]>(
    selectedSource?.youtube
      ? [
          { label: "Channel", value: present(selectedSource.youtube.channel_title) },
          { label: "Video form", value: present(selectedSource.youtube.video_form) },
          { label: "Duration", value: secondsLabel(selectedSource.youtube.duration_seconds) },
          { label: "Playlist videos", value: present(selectedSource.youtube.playlist_video_count) },
          { label: "Availability", value: present(selectedSource.youtube.availability_status) },
        ]
      : [],
  );

  let telegramRows = $derived<MetadataRow[]>(
    selectedSource?.telegram
      ? [
          { label: "Subtype", value: present(selectedSource.sourceSubtype) },
          { label: "Account", value: present(selectedSource.telegram.account_id) },
        ]
      : [],
  );
```

- [ ] **Step 4: Replace Inspector metadata markup**

In the selected-source branch, replace the existing metadata definition list with:

```svelte
    <dl class="meta-list">
      {#each metadataRows as row}
        <div>
          <dt>{row.label}</dt>
          <dd>
            {#if row.href && row.value}
              <a href={row.href} target="_blank" rel="noreferrer">{row.value}</a>
            {:else}
              {row.value ?? "N/A"}
            {/if}
          </dd>
        </div>
      {/each}
    </dl>

    {#if selectedSource.youtube}
      <section class="detail-section" aria-label="YouTube details">
        <h3>YouTube details</h3>
        <dl class="meta-list">
          {#each youtubeRows as row}
            <div>
              <dt>{row.label}</dt>
              <dd>{row.value ?? "N/A"}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}

    {#if selectedSource.telegram}
      <section class="detail-section" aria-label="Telegram details">
        <h3>Telegram details</h3>
        <dl class="meta-list">
          {#each telegramRows as row}
            <div>
              <dt>{row.label}</dt>
              <dd>{row.value ?? "N/A"}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}
```

Replace the connected pill condition:

```svelte
      {#if selectedSource.statusDetail}
        <span class="meta-pill">{selectedSource.statusDetail}</span>
      {/if}
```

Remove the `disabledReason` notice block because catalog rows no longer carry project-connect disabled reasons.

- [ ] **Step 5: Add small Inspector heading/link styles**

Add to the `<style>` block in `LibraryInspector.svelte`:

```css
  h3 {
    margin: 12px 0 8px;
    font-size: 13px;
    font-weight: 700;
  }

  a {
    color: var(--extractum-primary);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }
```

- [ ] **Step 6: Run focused UI tests**

Run:

```powershell
npm.cmd run test -- src/lib/library-prototype-contract.test.ts src/lib/research-projects-import-boundary.test.ts
```

Expected: PASS.

- [ ] **Step 7: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 8: Commit UI metadata display**

Run:

```powershell
git add src/lib/components/research-projects/LibraryWorkspace.svelte src/lib/components/research-projects/LibraryInspector.svelte src/lib/library-prototype-contract.test.ts
git commit -m "feat: show library source metadata"
```

---

## Task 7: Remove Prototype-Only Library Subtype Behavior From Old Model Tests

**Files:**
- Modify: `src/lib/ui/research-projects-model.test.ts`
- Modify: `src/lib/ui/research-projects-model.ts` only if unused Library prototype helpers are no longer imported.

- [ ] **Step 1: Check whether old prototype filter helpers are still used**

Run:

```powershell
rg -n "buildLibraryFilterTree|filterLibrarySourcesForLibrary|reconcileLibrarySourceSelection|YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON|LibraryFilterTreeRow|LibraryFilterId" src
```

Expected: only `src/lib/ui/research-projects-model.ts` and `src/lib/ui/research-projects-model.test.ts` remain.

- [ ] **Step 2: Keep connect helpers and remove unused prototype-only helpers**

If Step 1 confirms the old filter helpers are unused, remove these exports from `src/lib/ui/research-projects-model.ts`:

```ts
export type LibrarySourceSubtype = "video" | "playlist" | "channel";
export type LibraryFilterId =
  | "all"
  | `provider:${LibrarySourceProvider}`
  | `provider:youtube/subtype:${LibrarySourceSubtype}`;

export type LibraryFilterTreeRow = {
  id: LibraryFilterId;
  label: string;
  provider: LibrarySourceProvider | "all";
  subtype?: LibrarySourceSubtype;
  count: number;
  disabled?: boolean;
  disabledReason?: string;
  data?: LibraryFilterTreeRow[];
};

export type LibraryTableFilterState = {
  filterId: LibraryFilterId;
  query: string;
};

export const LIBRARY_ALL_FILTER_ID: LibraryFilterId = "all";
export const YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON =
  "Subtype filtering requires source subtype metadata.";
```

Also remove these prototype-only functions from `src/lib/ui/research-projects-model.ts`:

- `countProvider`;
- `disabledYoutubeSubtypeRow`;
- `buildLibraryFilterTree`;
- `providerFromFilterId`;
- `filterLibrarySourcesForLibrary`;
- `reconcileLibrarySourceSelection`.

Keep these connect/project helpers in `research-projects-model.ts`:

- `buildLibrarySourcesView`;
- `filterLibrarySources`;
- `connectableSelection`;
- `buildSourceGroupUpdateInput`;
- `buildProjectSourceLinksView`.

- [ ] **Step 3: Remove obsolete tests**

In `src/lib/ui/research-projects-model.test.ts`, remove imports for:

```ts
  LIBRARY_ALL_FILTER_ID,
  YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
  buildLibraryFilterTree,
  filterLibrarySourcesForLibrary,
  reconcileLibrarySourceSelection,
```

Remove these tests because `src/lib/ui/library-catalog-model.test.ts` now owns the Library screen filtering contract:

- `"builds the Library filter tree with disabled YouTube subtype rows"`;
- `"filters Library sources by selected tree row and search query"`;
- `"reconciles selected Library source with the visible rows"`.

- [ ] **Step 4: Run model tests**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-model.test.ts src/lib/ui/library-catalog-model.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run TypeScript/Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 6: Commit cleanup**

Run:

```powershell
git add src/lib/ui/research-projects-model.ts src/lib/ui/research-projects-model.test.ts
git commit -m "refactor: separate library catalog model"
```

---

## Task 8: Final Verification

**Files:**
- No planned source changes unless verification exposes a defect.

- [ ] **Step 1: Run backend tests for the new module**

Run:

```powershell
cargo test library_sources --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 2: Run focused frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/api/library-sources.test.ts src/lib/ui/library-catalog-model.test.ts src/lib/ui/library-catalog-workflow.test.ts src/lib/library-prototype-contract.test.ts src/lib/research-projects-import-boundary.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run full frontend test suite**

Run:

```powershell
npm.cmd run test
```

Expected: PASS.

- [ ] **Step 4: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 5: Start or reuse the dev server**

If no dev server is running, start one:

```powershell
npm.cmd run dev -- --host 127.0.0.1
```

Expected: Vite prints a local URL, usually `http://127.0.0.1:5173/` or the Tauri dev URL already used by the project.

- [ ] **Step 6: Browser-verify `/projects/library`**

Open `/projects/library` in the in-app browser or Chrome DevTools browser.

Verify:

- Library route loads without console errors.
- `IconRail` still marks Library active.
- Filter tree shows `All sources`, `YouTube`, `YouTube / Videos`, `YouTube / Playlists`, `YouTube / Channels`, `Telegram`, `Telegram / Channels`, `Telegram / Supergroups`, and `Telegram / Groups`.
- `YouTube / Channels` is disabled.
- Selecting provider and subtype filters updates table rows.
- Table headers are `Source`, `Type`, `Status`, `Projects`, `Items`, `Added`, `Last synced`.
- Selecting a source updates Inspector metadata.
- A provider detail field that is null displays `N/A` or is consistently handled according to the implementation.
- `Edit` and `Delete` remain disabled when no source is selected.
- No horizontal overflow at desktop width and narrow laptop width.

- [ ] **Step 7: Browser-verify `/projects` and Connect from library**

Open `/projects`.

Verify:

- Projects route still loads.
- `ProjectRail` is still present only on `/projects`.
- Connect from library still uses project compatibility state and still calls `list_analysis_sources` through the existing research-projects workflow.

- [ ] **Step 8: Commit any verification fixes**

If verification required code changes, stage the exact files reported by `git status --short` and commit them. For example, if verification changed only the Library screen and Inspector, run:

```powershell
git add src/lib/components/research-projects/LibraryScreen.svelte src/lib/components/research-projects/LibraryInspector.svelte
git commit -m "fix: stabilize library source metadata"
```

If no changes were required, do not create an empty commit.

---

## Completion Criteria

- `list_library_sources` is registered as a Tauri command.
- Backend query tests prove YouTube video, YouTube playlist, Telegram, counts, timestamps, and missing provider metadata behavior.
- `/projects/library` uses `listLibrarySources`, not `listAnalysisSources`.
- Existing project/connect workflow remains on `listAnalysisSources`.
- Library subtype filters are active for YouTube video, YouTube playlist, Telegram channel, Telegram supergroup, and Telegram group.
- YouTube channel filter remains disabled.
- Library table shows the approved metadata columns.
- Inspector shows source metadata and provider details with safe nullable-field rendering.
- Import-boundary tests still prevent direct shadcn/SVAR imports from feature screens.
- `cargo test library_sources --manifest-path src-tauri/Cargo.toml`, focused Vitest tests, full Vitest suite, and `npm.cmd run check` pass.
