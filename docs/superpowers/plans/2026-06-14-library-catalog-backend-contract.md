# Library Catalog Backend Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `list_library_catalog` as the backend-owned Library catalog contract and migrate Library/Projects source rows away from frontend-only source-job status merging.

**Architecture:** Keep the existing `list_library_sources` command as the raw compatibility read model. Add a catalog envelope in `src-tauri/src/library_sources` that composes existing source rows with `SourceJobState`, source-level capabilities, disabled reasons, and backend filter counts. Update TypeScript API/types and frontend workflows so `/projects/library` uses one catalog call for source rows and `/projects` uses catalog records for Library source status while keeping `listSourceJobs` only for the bottom queue.

**Tech Stack:** Rust/Tauri commands, SQLx/SQLite, in-memory `SourceJobState`, Svelte 5, TypeScript, Vitest, cargo tests, existing extractum-ui wrappers.

---

## Approved Design Inputs

- Spec: `docs/superpowers/specs/2026-06-14-library-catalog-backend-contract-design.md`
- Backlog section: `docs/backlog.md`, `3.6 Library Source Operations`
- Existing backend read model: `src-tauri/src/library_sources/mod.rs`
- Existing backend source models: `src-tauri/src/library_sources/models.rs`
- Existing source jobs state: `src-tauri/src/youtube/jobs.rs`
- Existing frontend Library types/API: `src/lib/types/library-sources.ts`, `src/lib/api/library-sources.ts`
- Existing Library catalog model/workflow: `src/lib/ui/library-catalog-model.ts`, `src/lib/ui/library-catalog-workflow.ts`
- Existing Projects workflow/model: `src/lib/ui/research-projects-workflow.ts`, `src/lib/ui/research-projects-model.ts`

## Scope Check

This plan implements only the first Library Source Operations backend simplification slice.

In scope:

- new `list_library_catalog` command;
- latest relevant source job in catalog response;
- backend source status, status detail, capabilities, disabled reasons;
- backend provider/subtype filter counts;
- `/projects/library` source rows loaded through one catalog API;
- `/projects` Library source rows based on catalog records;
- old `list_library_sources` kept registered.

Out of scope:

- YouTube duplicate add outcome contract;
- playlist item addability;
- playlist video materialization;
- source-first project connection commands;
- durable source edit/archive overrides;
- source delete UI;
- Project Export.

## File Structure

Backend:

- Modify `src-tauri/src/youtube/jobs.rs`
  - Add a `catalog_jobs_for_sources` helper that includes queued, running, and failed jobs.
- Modify `src-tauri/src/library_sources/models.rs`
  - Add catalog response, catalog record, status, capabilities, disabled reasons, and filter count models.
- Modify `src-tauri/src/library_sources/mod.rs`
  - Add `list_library_catalog`, `query_library_catalog`, catalog status/capability helpers, and filter count generation.
- Modify `src-tauri/src/lib.rs`
  - Register the new command without removing `list_library_sources`.

Frontend API/types:

- Modify `src/lib/types/library-sources.ts`
  - Add catalog response and catalog record types.
- Modify `src/lib/api/library-sources.ts`
  - Add `listLibraryCatalog()`.
- Modify `src/lib/api/library-sources.test.ts`
  - Verify the new command name.

Frontend Library catalog:

- Modify `src/lib/ui/library-catalog-model.ts`
  - Map backend catalog records directly into table rows.
  - Build filter tree from backend filter counts.
- Modify `src/lib/ui/library-catalog-model.test.ts`
  - Replace source-job merging expectations with backend-status expectations.
- Modify `src/lib/ui/library-catalog-workflow.ts`
  - Load `listCatalog()` only.
- Modify `src/lib/ui/library-catalog-workflow.test.ts`
  - Verify no source job dependency remains.
- Modify `src/routes/projects/library/+page.svelte`
  - Use `listLibraryCatalog`, not `listLibrarySources` plus `listSourceJobs`.
- Modify `src/lib/components/research-projects/LibraryScreen.svelte`
  - Build filter rows from backend filter counts.

Frontend Projects:

- Modify `src/lib/ui/research-projects-model.ts`
  - Build Library source rows from `LibraryCatalogRecord[]`.
- Modify `src/lib/ui/research-projects-model.test.ts`
  - Verify catalog disabled reasons and `Already in project` override.
- Modify `src/lib/ui/research-projects-workflow.ts`
  - Load catalog records for Library source rows.
  - Keep `listSourceJobs()` only for bottom queue state.
- Modify `src/lib/ui/research-projects-workflow.test.ts`
  - Verify source row status comes from catalog, not jobs.
- Modify `src/routes/projects/+page.svelte`
  - Use `listLibraryCatalog`.
- Modify `src/lib/research-projects-route-contract.test.ts`
  - Update route contract.

Svelte checks:

- Run `mcp__svelte_server__.svelte_autofixer` on modified Svelte files before final verification.

---

## Task 0: Baseline

**Files:**
- No source edits.

- [x] **Step 1: Confirm clean worktree**

Run:

```powershell
git status --short --branch
```

Expected: output starts with `## main` and contains no modified files.

- [x] **Step 2: Run focused baseline tests**

Run:

```powershell
npm.cmd test -- --run src/lib/api/library-sources.test.ts src/lib/ui/library-catalog-model.test.ts src/lib/ui/library-catalog-workflow.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/library-prototype-contract.test.ts src/lib/research-projects-route-contract.test.ts
```

Expected: PASS before changes. If this fails, stop and record the pre-existing failure.

- [x] **Step 3: Run backend baseline tests**

Run:

```powershell
cargo test library_sources --manifest-path src-tauri/Cargo.toml
cargo test active_jobs_for_sources --manifest-path src-tauri/Cargo.toml
```

Expected: PASS before changes.

---

## Task 1: Backend Catalog Contract

**Files:**
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/library_sources/models.rs`
- Modify: `src-tauri/src/library_sources/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Write failing source-job helper test**

In `src-tauri/src/youtube/jobs.rs`, inside the existing `#[cfg(test)] mod tests`, add:

```rust
#[tokio::test]
async fn catalog_jobs_for_sources_includes_latest_failed_jobs() {
    let state = SourceJobState::new();
    let options = YoutubeSyncOptions {
        metadata: true,
        transcripts: false,
        comments: false,
    };

    let succeeded = state
        .create_job(
            7,
            SourceJobType::YoutubeVideoMetadataSync,
            None,
            options.clone(),
        )
        .await
        .expect("create succeeded job");
    state
        .finish_job(&succeeded.job_id, |job| {
            job.status = SourceJobStatus::Succeeded;
            job.started_at = 10;
        })
        .await
        .expect("finish succeeded job");

    let failed = state
        .create_job(
            7,
            SourceJobType::YoutubeVideoTranscriptSync,
            None,
            options.clone(),
        )
        .await
        .expect("create failed job");
    state
        .finish_job(&failed.job_id, |job| {
            job.status = SourceJobStatus::Failed;
            job.started_at = 20;
            job.error = Some("Transcript quota exceeded".to_string());
        })
        .await
        .expect("finish failed job");

    let related = state
        .create_job(
            99,
            SourceJobType::YoutubePlaylistVideoSync,
            Some(8),
            options,
        )
        .await
        .expect("create related job");
    state
        .update_job(&related.job_id, |job| {
            job.status = SourceJobStatus::Running;
            job.started_at = 30;
            job.message = Some("Syncing playlist video.".to_string());
        })
        .await
        .expect("update related job");

    let jobs = state.catalog_jobs_for_sources(&[7, 8]).await;

    assert_eq!(
        jobs.iter().map(|job| job.job_id.as_str()).collect::<Vec<_>>(),
        vec![related.job_id.as_str(), failed.job_id.as_str()]
    );
    assert_eq!(jobs[0].related_source_id, Some(8));
    assert_eq!(jobs[1].status, SourceJobStatus::Failed);
}
```

- [x] **Step 2: Run source-job test and verify failure**

Run:

```powershell
cargo test catalog_jobs_for_sources_includes_latest_failed_jobs --manifest-path src-tauri/Cargo.toml
```

Expected: FAIL with a missing `catalog_jobs_for_sources` method.

- [x] **Step 3: Implement source-job catalog helper**

In `impl SourceJobState` in `src-tauri/src/youtube/jobs.rs`, add:

```rust
pub(crate) async fn catalog_jobs_for_sources(&self, source_ids: &[i64]) -> Vec<SourceJobRecord> {
    let source_ids = source_ids.iter().copied().collect::<HashSet<_>>();
    let mut jobs = self
        .inner
        .lock()
        .await
        .jobs
        .values()
        .filter(|job| {
            source_ids.contains(&job.source_id)
                || job
                    .related_source_id
                    .is_some_and(|source_id| source_ids.contains(&source_id))
        })
        .filter(|job| {
            matches!(
                &job.status,
                SourceJobStatus::Queued | SourceJobStatus::Running | SourceJobStatus::Failed
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    jobs.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then_with(|| b.job_id.cmp(&a.job_id))
    });
    jobs
}
```

The file already imports `HashSet`; do not add a duplicate import.

- [x] **Step 4: Run source-job test and verify pass**

Run:

```powershell
cargo test catalog_jobs_for_sources_includes_latest_failed_jobs --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [x] **Step 5: Write failing Library catalog backend tests**

In `src-tauri/src/library_sources/mod.rs`, extend the test module imports:

```rust
use crate::youtube::jobs::{SourceJobState, SourceJobStatus, SourceJobType, YoutubeSyncOptions};
```

Add this test below the existing Library source tests:

```rust
#[tokio::test]
async fn list_library_catalog_returns_status_capabilities_and_filter_counts() {
    let pool = memory_pool().await;
    insert_source(
        &pool,
        1,
        "youtube",
        Some("video"),
        None,
        "vid-1",
        "Video fallback",
        100,
        Some(200),
    )
    .await;
    insert_source(
        &pool,
        2,
        "youtube",
        Some("playlist"),
        None,
        "pl-1",
        "Playlist fallback",
        101,
        None,
    )
    .await;
    insert_source(
        &pool,
        3,
        "telegram",
        Some("supergroup"),
        Some(77),
        "-1007",
        "Drone Radar",
        102,
        Some(202),
    )
    .await;
    insert_source(
        &pool,
        4,
        "youtube",
        Some("channel"),
        None,
        "chan-1",
        "Unsupported channel",
        103,
        None,
    )
    .await;

    sqlx::query("INSERT INTO items (id, source_id, content_zstd) VALUES (1, 1, X'01'), (2, 2, X'02'), (3, 3, X'03')")
        .execute(&pool)
        .await
        .expect("insert items");
    sqlx::query("INSERT INTO projects (id, name, created_at, updated_at) VALUES (10, 'Project A', 1, 1), (11, 'Project B', 1, 1)")
        .execute(&pool)
        .await
        .expect("insert projects");
    sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (10, 1, 1), (11, 1, 1)")
        .execute(&pool)
        .await
        .expect("insert project sources");
    sqlx::query(
        r#"
        INSERT INTO youtube_video_sources (
            source_id, video_id, canonical_url, title, channel_title,
            duration_seconds, video_form, availability_status
        )
        VALUES (1, 'vid-1', 'https://youtu.be/vid-1', 'Video title', 'Channel A', 321, 'short', 'available')
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

    let source_jobs = SourceJobState::new();
    let options = YoutubeSyncOptions {
        metadata: true,
        transcripts: false,
        comments: false,
    };
    let failed = source_jobs
        .create_job(
            1,
            SourceJobType::YoutubeVideoTranscriptSync,
            None,
            options.clone(),
        )
        .await
        .expect("create failed source job");
    source_jobs
        .finish_job(&failed.job_id, |job| {
            job.status = SourceJobStatus::Failed;
            job.started_at = 20;
            job.error = Some("Transcript quota exceeded".to_string());
        })
        .await
        .expect("finish failed source job");
    let running = source_jobs
        .create_job(
            2,
            SourceJobType::YoutubePlaylistFullSync,
            None,
            options,
        )
        .await
        .expect("create running source job");
    source_jobs
        .update_job(&running.job_id, |job| {
            job.status = SourceJobStatus::Running;
            job.started_at = 30;
            job.message = Some("Syncing playlist.".to_string());
        })
        .await
        .expect("update running source job");

    let catalog = query_library_catalog(&pool, &source_jobs)
        .await
        .expect("query library catalog");

    let video = catalog
        .sources
        .iter()
        .find(|record| record.source.source_id == 1)
        .expect("video catalog record");
    assert_eq!(video.status, LibraryCatalogStatus::Error);
    assert_eq!(video.status_detail.as_deref(), Some("Transcript quota exceeded"));
    assert_eq!(video.latest_job.as_ref().map(|job| job.job_id.as_str()), Some(failed.job_id.as_str()));
    assert!(!video.capabilities.can_delete);
    assert_eq!(
        video.disabled_reasons.delete.as_deref(),
        Some("Source 1 is used by 2 project(s). Remove it from projects first.")
    );
    assert!(!video.capabilities.can_edit);
    assert_eq!(
        video.disabled_reasons.edit.as_deref(),
        Some("Source editing is not available yet.")
    );

    let playlist = catalog
        .sources
        .iter()
        .find(|record| record.source.source_id == 2)
        .expect("playlist catalog record");
    assert_eq!(playlist.status, LibraryCatalogStatus::Syncing);
    assert_eq!(playlist.status_detail.as_deref(), Some("Syncing playlist."));
    assert!(!playlist.capabilities.can_refresh_source);
    assert_eq!(
        playlist.disabled_reasons.refresh_source.as_deref(),
        Some("Source is syncing.")
    );

    let telegram = catalog
        .sources
        .iter()
        .find(|record| record.source.source_id == 3)
        .expect("telegram catalog record");
    assert_eq!(telegram.status, LibraryCatalogStatus::Active);
    assert!(telegram.latest_job.is_none());
    assert!(telegram.capabilities.can_connect_to_project);

    let channel = catalog
        .sources
        .iter()
        .find(|record| record.source.source_id == 4)
        .expect("youtube channel catalog record");
    assert_eq!(channel.status, LibraryCatalogStatus::Unavailable);
    assert_eq!(
        channel.status_detail.as_deref(),
        Some(YOUTUBE_CHANNEL_DISABLED_REASON)
    );
    assert!(!channel.capabilities.can_refresh_source);
    assert!(!channel.capabilities.can_connect_to_project);
    assert_eq!(
        channel.disabled_reasons.connect_to_project.as_deref(),
        Some(YOUTUBE_CHANNEL_DISABLED_REASON)
    );

    let youtube_video_count = catalog
        .filter_counts
        .iter()
        .find(|count| count.provider == "youtube" && count.source_subtype.as_deref() == Some("video"))
        .expect("youtube video count");
    assert_eq!(youtube_video_count.count, 1);
    assert!(!youtube_video_count.disabled);

    let youtube_channel_count = catalog
        .filter_counts
        .iter()
        .find(|count| count.provider == "youtube" && count.source_subtype.as_deref() == Some("channel"))
        .expect("youtube channel count");
    assert_eq!(youtube_channel_count.count, 1);
    assert!(youtube_channel_count.disabled);
    assert_eq!(
        youtube_channel_count.disabled_reason.as_deref(),
        Some(YOUTUBE_CHANNEL_DISABLED_REASON)
    );
}
```

- [x] **Step 6: Run Library catalog backend test and verify failure**

Run:

```powershell
cargo test list_library_catalog_returns_status_capabilities_and_filter_counts --manifest-path src-tauri/Cargo.toml
```

Expected: FAIL with missing catalog types and `query_library_catalog`.

- [x] **Step 7: Add catalog models**

In `src-tauri/src/library_sources/models.rs`, add this import with the existing imports at the top:

```rust
use crate::youtube::jobs::SourceJobRecord;
```

Then add these models after `LibrarySourceRecord`:

```rust

#[derive(Clone, Debug, Serialize)]
pub(crate) struct LibraryCatalogResponse {
    pub sources: Vec<LibraryCatalogRecord>,
    pub filter_counts: Vec<LibraryCatalogFilterCount>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct LibraryCatalogRecord {
    pub source: LibrarySourceRecord,
    pub latest_job: Option<SourceJobRecord>,
    pub status: LibraryCatalogStatus,
    pub status_detail: Option<String>,
    pub capabilities: LibraryCatalogCapabilities,
    pub disabled_reasons: LibraryCatalogDisabledReasons,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LibraryCatalogStatus {
    Active,
    Syncing,
    Error,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LibraryCatalogCapabilities {
    pub can_refresh_source: bool,
    pub can_delete: bool,
    pub can_edit: bool,
    pub can_connect_to_project: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LibraryCatalogDisabledReasons {
    pub refresh_source: Option<String>,
    pub delete: Option<String>,
    pub edit: Option<String>,
    pub connect_to_project: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LibraryCatalogFilterCount {
    pub provider: String,
    pub source_subtype: Option<String>,
    pub count: i64,
    pub disabled: bool,
    pub disabled_reason: Option<String>,
}
```

- [x] **Step 8: Implement catalog query and helpers**

In `src-tauri/src/library_sources/mod.rs`, keep the existing public source
record export, keep the existing `LibrarySourceRow` import, and add
crate-visible catalog exports:

```rust
use models::LibrarySourceRow;
pub(crate) use models::{
    LibraryCatalogCapabilities, LibraryCatalogDisabledReasons, LibraryCatalogFilterCount,
    LibraryCatalogRecord, LibraryCatalogResponse, LibraryCatalogStatus,
};
pub use models::{LibrarySourceRecord, LibraryTelegramSourceDetails, LibraryYoutubeSourceDetails};
```

Add imports:

```rust
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::youtube::jobs::{SourceJobRecord, SourceJobState, SourceJobStatus};
```

Add constants near `LIBRARY_SOURCES_SQL`:

```rust
pub(crate) const YOUTUBE_CHANNEL_DISABLED_REASON: &str =
    "YouTube channel sources are not supported by the current backend.";
const SOURCE_EDIT_DISABLED_REASON: &str = "Source editing is not available yet.";
const SOURCE_SYNCING_DISABLED_REASON: &str = "Source is syncing.";
```

Add the Tauri command:

```rust
#[tauri::command]
pub(crate) async fn list_library_catalog(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    source_job_state: tauri::State<'_, SourceJobState>,
) -> AppResult<LibraryCatalogResponse> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    query_library_catalog(&pool, source_job_state.inner()).await
}
```

Add testable query helpers below `query_library_sources`:

```rust
pub(crate) async fn query_library_catalog(
    pool: &sqlx::SqlitePool,
    source_job_state: &SourceJobState,
) -> AppResult<LibraryCatalogResponse> {
    let sources = query_library_sources(pool).await?;
    let source_ids = sources
        .iter()
        .map(|source| source.source_id)
        .collect::<Vec<_>>();
    let jobs = source_job_state.catalog_jobs_for_sources(&source_ids).await;
    let latest_jobs = latest_catalog_jobs_by_source(&source_ids, jobs);
    let filter_counts = build_catalog_filter_counts(&sources);
    let records = sources
        .into_iter()
        .map(|source| {
            let latest_job = latest_jobs.get(&source.source_id).cloned();
            build_catalog_record(source, latest_job)
        })
        .collect::<Vec<_>>();

    Ok(LibraryCatalogResponse {
        sources: records,
        filter_counts,
    })
}

fn latest_catalog_jobs_by_source(
    source_ids: &[i64],
    jobs: Vec<SourceJobRecord>,
) -> HashMap<i64, SourceJobRecord> {
    let source_ids = source_ids.iter().copied().collect::<HashSet<_>>();
    let mut latest = HashMap::<i64, SourceJobRecord>::new();

    for job in jobs {
        let mut matched_source_ids = Vec::new();
        if source_ids.contains(&job.source_id) {
            matched_source_ids.push(job.source_id);
        }
        if let Some(related_source_id) = job.related_source_id {
            if source_ids.contains(&related_source_id)
                && !matched_source_ids.contains(&related_source_id)
            {
                matched_source_ids.push(related_source_id);
            }
        }

        for source_id in matched_source_ids {
            let replace = latest.get(&source_id).is_none_or(|current| {
                job.started_at > current.started_at
                    || (job.started_at == current.started_at && job.job_id > current.job_id)
            });
            if replace {
                latest.insert(source_id, job.clone());
            }
        }
    }

    latest
}

fn build_catalog_record(
    source: LibrarySourceRecord,
    latest_job: Option<SourceJobRecord>,
) -> LibraryCatalogRecord {
    let unsupported_reason = unsupported_source_reason(&source);
    let (job_status, job_status_detail) = catalog_status(&latest_job);
    let status = if unsupported_reason.is_some() {
        LibraryCatalogStatus::Unavailable
    } else {
        job_status
    };
    let status_detail = unsupported_reason.clone().or(job_status_detail);
    let syncing = matches!(&status, LibraryCatalogStatus::Syncing);
    let delete_reason = (source.project_count > 0).then(|| {
        format!(
            "Source {} is used by {} project(s). Remove it from projects first.",
            source.source_id, source.project_count
        )
    });

    LibraryCatalogRecord {
        source,
        latest_job,
        status,
        status_detail,
        capabilities: LibraryCatalogCapabilities {
            can_refresh_source: unsupported_reason.is_none() && !syncing,
            can_delete: delete_reason.is_none(),
            can_edit: false,
            can_connect_to_project: unsupported_reason.is_none(),
        },
        disabled_reasons: LibraryCatalogDisabledReasons {
            refresh_source: unsupported_reason
                .clone()
                .or_else(|| syncing.then(|| SOURCE_SYNCING_DISABLED_REASON.to_string())),
            delete: delete_reason,
            edit: Some(SOURCE_EDIT_DISABLED_REASON.to_string()),
            connect_to_project: unsupported_reason,
        },
    }
}

fn unsupported_source_reason(source: &LibrarySourceRecord) -> Option<String> {
    match (source.provider.as_str(), source.source_subtype.as_deref()) {
        ("youtube", Some("channel")) => Some(YOUTUBE_CHANNEL_DISABLED_REASON.to_string()),
        _ => None,
    }
}

fn catalog_status(job: &Option<SourceJobRecord>) -> (LibraryCatalogStatus, Option<String>) {
    let Some(job) = job else {
        return (LibraryCatalogStatus::Active, None);
    };

    match &job.status {
        SourceJobStatus::Queued | SourceJobStatus::Running => (
            LibraryCatalogStatus::Syncing,
            Some(job.message.clone().unwrap_or_else(|| "Syncing".to_string())),
        ),
        SourceJobStatus::Failed => (
            LibraryCatalogStatus::Error,
            Some(job.error.clone().unwrap_or_else(|| "Last sync failed".to_string())),
        ),
        SourceJobStatus::Succeeded
        | SourceJobStatus::CancelRequested
        | SourceJobStatus::Cancelled => (LibraryCatalogStatus::Active, None),
    }
}

fn build_catalog_filter_counts(sources: &[LibrarySourceRecord]) -> Vec<LibraryCatalogFilterCount> {
    let mut counts = BTreeMap::<(String, Option<String>), i64>::new();
    for source in sources {
        *counts
            .entry((source.provider.clone(), source.source_subtype.clone()))
            .or_insert(0) += 1;
    }

    let stable_rows = [
        ("youtube", Some("video"), false, None),
        ("youtube", Some("playlist"), false, None),
        (
            "youtube",
            Some("channel"),
            true,
            Some(YOUTUBE_CHANNEL_DISABLED_REASON),
        ),
        ("telegram", Some("channel"), false, None),
        ("telegram", Some("supergroup"), false, None),
        ("telegram", Some("group"), false, None),
    ];

    let mut rows = stable_rows
        .iter()
        .map(|(provider, subtype, disabled, disabled_reason)| {
            let subtype = subtype.map(str::to_string);
            LibraryCatalogFilterCount {
                provider: (*provider).to_string(),
                source_subtype: subtype.clone(),
                count: counts
                    .get(&((*provider).to_string(), subtype))
                    .copied()
                    .unwrap_or(0),
                disabled: *disabled,
                disabled_reason: disabled_reason.map(|reason| (*reason).to_string()),
            }
        })
        .collect::<Vec<_>>();

    for ((provider, subtype), count) in counts {
        if stable_rows
            .iter()
            .any(|(stable_provider, stable_subtype, _, _)| {
                *stable_provider == provider.as_str()
                    && stable_subtype.map(str::to_string) == subtype
            })
        {
            continue;
        }
        rows.push(LibraryCatalogFilterCount {
            provider,
            source_subtype: subtype,
            count,
            disabled: false,
            disabled_reason: None,
        });
    }

    rows
}
```

- [x] **Step 9: Register the command**

In `src-tauri/src/lib.rs`, replace the import:

```rust
use library_sources::list_library_sources;
```

with:

```rust
use library_sources::{list_library_catalog, list_library_sources};
```

Add `list_library_catalog` next to `list_library_sources` inside `tauri::generate_handler!`:

```rust
            list_library_sources,
            list_library_catalog,
```

- [x] **Step 10: Run backend tests**

Run:

```powershell
cargo test catalog_jobs_for_sources_includes_latest_failed_jobs --manifest-path src-tauri/Cargo.toml
cargo test list_library_catalog_returns_status_capabilities_and_filter_counts --manifest-path src-tauri/Cargo.toml
cargo test library_sources --manifest-path src-tauri/Cargo.toml
```

Expected: all PASS.

- [x] **Step 11: Commit backend catalog contract**

Run:

```powershell
git add src-tauri\src\youtube\jobs.rs src-tauri\src\library_sources\models.rs src-tauri\src\library_sources\mod.rs src-tauri\src\lib.rs
git commit -m "feat: add library catalog backend contract"
```

---

## Task 2: Frontend API, Types, And Catalog Model

**Files:**
- Modify: `src/lib/types/library-sources.ts`
- Modify: `src/lib/api/library-sources.ts`
- Modify: `src/lib/api/library-sources.test.ts`
- Modify: `src/lib/ui/library-catalog-model.ts`
- Modify: `src/lib/ui/library-catalog-model.test.ts`

- [x] **Step 1: Write failing API wrapper test**

In `src/lib/api/library-sources.test.ts`, update the import:

```ts
import { listLibraryCatalog, listLibrarySources } from "./library-sources";
```

Add this test:

```ts
  it("lists backend-owned library catalog records", async () => {
    const response = {
      sources: [
        {
          source: {
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
            youtube: null,
            telegram: null,
          },
          latest_job: null,
          status: "active",
          status_detail: null,
          capabilities: {
            can_refresh_source: true,
            can_delete: false,
            can_edit: false,
            can_connect_to_project: true,
          },
          disabled_reasons: {
            refresh_source: null,
            delete: "Source 1 is used by 2 project(s). Remove it from projects first.",
            edit: "Source editing is not available yet.",
            connect_to_project: null,
          },
        },
      ],
      filter_counts: [
        {
          provider: "youtube",
          source_subtype: "video",
          count: 1,
          disabled: false,
          disabled_reason: null,
        },
      ],
    };
    invokeMock.mockResolvedValueOnce(response);

    await expect(listLibraryCatalog()).resolves.toEqual(response);

    expect(invokeMock).toHaveBeenLastCalledWith("list_library_catalog");
  });
```

- [x] **Step 2: Run API test and verify failure**

Run:

```powershell
npm.cmd test -- --run src/lib/api/library-sources.test.ts
```

Expected: FAIL because `listLibraryCatalog` is not exported.

- [x] **Step 3: Add frontend catalog types**

In `src/lib/types/library-sources.ts`, add:

```ts
import type { SourceJobRecord } from "$lib/types/sources";

export type LibraryCatalogStatus = "active" | "syncing" | "error" | "unavailable";

export interface LibraryCatalogCapabilities {
  can_refresh_source: boolean;
  can_delete: boolean;
  can_edit: boolean;
  can_connect_to_project: boolean;
}

export interface LibraryCatalogDisabledReasons {
  refresh_source: string | null;
  delete: string | null;
  edit: string | null;
  connect_to_project: string | null;
}

export interface LibraryCatalogRecord {
  source: LibrarySourceRecord;
  latest_job: SourceJobRecord | null;
  status: LibraryCatalogStatus;
  status_detail: string | null;
  capabilities: LibraryCatalogCapabilities;
  disabled_reasons: LibraryCatalogDisabledReasons;
}

export interface LibraryCatalogFilterCount {
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  count: number;
  disabled: boolean;
  disabled_reason: string | null;
}

export interface LibraryCatalogResponse {
  sources: LibraryCatalogRecord[];
  filter_counts: LibraryCatalogFilterCount[];
}
```

Place the `import type` at the top of the file before exported type declarations.

- [x] **Step 4: Add API wrapper**

In `src/lib/api/library-sources.ts`, update the type import:

```ts
import type { LibraryCatalogResponse, LibrarySourceRecord } from "$lib/types/library-sources";
```

Add:

```ts
export function listLibraryCatalog() {
  return invoke<LibraryCatalogResponse>("list_library_catalog");
}
```

Keep `listLibrarySources()` unchanged.

- [x] **Step 5: Run API test and verify pass**

Run:

```powershell
npm.cmd test -- --run src/lib/api/library-sources.test.ts
```

Expected: PASS.

- [x] **Step 6: Write failing catalog model tests**

In `src/lib/ui/library-catalog-model.test.ts`, remove the `SourceJobRecord`
import and remove the `job()` helper. Then replace the job-derived status test
with:

```ts
  it("maps backend catalog status and status detail into catalog rows", () => {
    const rows = buildLibraryCatalogSourcesView([
      catalogRecord({
        source: record({
          source_id: 3,
          provider: "youtube",
          source_subtype: "video",
          title: "Running",
        }),
        status: "syncing",
        status_detail: "Syncing playlist.",
      }),
      catalogRecord({
        source: record({
          source_id: 4,
          provider: "youtube",
          source_subtype: "video",
          title: "Failed",
        }),
        status: "error",
        status_detail: "Quota",
      }),
    ]);

    expect(rows.find((row) => row.sourceId === 3)?.status).toBe("syncing");
    expect(rows.find((row) => row.sourceId === 3)?.statusDetail).toBe("Syncing playlist.");
    expect(rows.find((row) => row.sourceId === 4)?.status).toBe("error");
    expect(rows.find((row) => row.sourceId === 4)?.statusDetail).toBe("Quota");
  });
```

Add this helper near the existing `record` helper:

```ts
function catalogRecord(overrides: Partial<LibraryCatalogRecord> = {}): LibraryCatalogRecord {
  return {
    source: record(),
    latest_job: null,
    status: "active",
    status_detail: null,
    capabilities: {
      can_refresh_source: true,
      can_delete: true,
      can_edit: false,
      can_connect_to_project: true,
    },
    disabled_reasons: {
      refresh_source: null,
      delete: null,
      edit: "Source editing is not available yet.",
      connect_to_project: null,
    },
    ...overrides,
  };
}
```

Update the import from `$lib/types/library-sources` to include:

```ts
import type {
  LibraryCatalogFilterCount,
  LibraryCatalogRecord,
  LibrarySourceProvider,
  LibrarySourceRecord,
  LibrarySourceSubtype,
} from "$lib/types/library-sources";
```

Add this filter count helper:

```ts
function filterCount(
  provider: LibrarySourceProvider,
  sourceSubtype: LibrarySourceSubtype,
  count: number,
  disabled = false,
  disabledReason: string | null = null,
): LibraryCatalogFilterCount {
  return {
    provider,
    source_subtype: sourceSubtype,
    count,
    disabled,
    disabled_reason: disabledReason,
  };
}
```

Replace the filter tree test input with backend filter counts:

```ts
    expect(
      buildLibraryCatalogFilterTree([
        filterCount("youtube", "video", 1),
        filterCount("youtube", "playlist", 1),
        filterCount("youtube", "channel", 0, true, "Backend disabled"),
        filterCount("telegram", "channel", 1),
        filterCount("telegram", "supergroup", 1),
        filterCount("telegram", "group", 1),
      ]),
    ).toEqual([
```

Update the expected YouTube channel row to assert `disabledReason: "Backend disabled"`.

For every call to `buildLibraryCatalogSourcesView([...records], [])`, change it to `buildLibraryCatalogSourcesView([...records].map((source) => catalogRecord({ source })))`.

- [x] **Step 7: Run model test and verify failure**

Run:

```powershell
npm.cmd test -- --run src/lib/ui/library-catalog-model.test.ts
```

Expected: FAIL because `buildLibraryCatalogSourcesView` and `buildLibraryCatalogFilterTree` still use old signatures.

- [x] **Step 8: Update catalog model**

In `src/lib/ui/library-catalog-model.ts`, update imports:

```ts
import type {
  LibraryCatalogFilterCount,
  LibraryCatalogRecord,
  LibrarySourceProvider,
  LibrarySourceSubtype,
  LibraryTelegramSourceDetails,
  LibraryYoutubeSourceDetails,
} from "$lib/types/library-sources";
```

Remove the `SourceJobRecord` import and remove the helper functions `latestJobBySource` and `statusFromJob`.

Change `buildLibraryCatalogSourcesView` to:

```ts
export function buildLibraryCatalogSourcesView(
  records: LibraryCatalogRecord[],
): LibraryCatalogSourceView[] {
  return records.map((record) => ({
    id: sourceRowId(record.source.source_id),
    sourceId: record.source.source_id,
    provider: record.source.provider,
    sourceSubtype: record.source.source_subtype,
    title: record.source.title ?? `Source #${record.source.source_id}`,
    subtitle: record.source.subtitle,
    typeLabel: typeLabel(record.source.provider, record.source.source_subtype),
    status: record.status,
    statusDetail: record.status_detail,
    projectCount: record.source.project_count,
    itemCount: record.source.item_count,
    itemCountLabel: countLabel(record.source.item_count),
    addedAtLabel: dateLabel(record.source.created_at) ?? "Unknown",
    lastSyncedLabel: dateLabel(record.source.last_synced_at) ?? "Never",
    canonicalUrl: record.source.canonical_url,
    externalId: record.source.external_id,
    youtube: record.source.youtube,
    telegram: record.source.telegram,
  }));
}
```

Change `buildLibraryCatalogFilterTree` to:

```ts
function countProviderFromBackend(
  counts: LibraryCatalogFilterCount[],
  provider: LibrarySourceProvider,
) {
  return counts
    .filter((count) => count.provider === provider)
    .reduce((total, count) => total + count.count, 0);
}

function backendSubtypeRow(
  counts: LibraryCatalogFilterCount[],
  provider: LibrarySourceProvider,
  subtype: Exclude<LibrarySourceSubtype, null>,
  label: string,
): LibraryCatalogFilterTreeRow {
  const backendCount = counts.find(
    (count) => count.provider === provider && count.source_subtype === subtype,
  );
  return {
    id: `provider:${provider}/subtype:${subtype}` as LibraryCatalogFilterId,
    label,
    provider,
    subtype,
    count: backendCount?.count ?? 0,
    disabled: backendCount?.disabled ?? false,
    disabledReason: backendCount?.disabled_reason ?? undefined,
  };
}

export function buildLibraryCatalogFilterTree(
  counts: LibraryCatalogFilterCount[],
): LibraryCatalogFilterTreeRow[] {
  const total = counts.reduce((sum, count) => sum + count.count, 0);
  return [
    {
      id: LIBRARY_CATALOG_ALL_FILTER_ID,
      label: "All sources",
      provider: "all",
      count: total,
    },
    {
      id: "provider:youtube",
      label: "YouTube",
      provider: "youtube",
      count: countProviderFromBackend(counts, "youtube"),
      data: [
        backendSubtypeRow(counts, "youtube", "video", "Videos"),
        backendSubtypeRow(counts, "youtube", "playlist", "Playlists"),
        backendSubtypeRow(counts, "youtube", "channel", "Channels"),
      ],
    },
    {
      id: "provider:telegram",
      label: "Telegram",
      provider: "telegram",
      count: countProviderFromBackend(counts, "telegram"),
      data: [
        backendSubtypeRow(counts, "telegram", "channel", "Channels"),
        backendSubtypeRow(counts, "telegram", "supergroup", "Supergroups"),
        backendSubtypeRow(counts, "telegram", "group", "Groups"),
      ],
    },
  ];
}
```

Remove `YOUTUBE_CHANNEL_DISABLED_REASON`, `countProvider`, `countSubtype`, and `subtypeRow` when they are no longer used.

- [x] **Step 9: Run model tests**

Run:

```powershell
npm.cmd test -- --run src/lib/ui/library-catalog-model.test.ts
```

Expected: PASS.

- [x] **Step 10: Commit frontend API and catalog model**

Run:

```powershell
git add src\lib\types\library-sources.ts src\lib\api\library-sources.ts src\lib\api\library-sources.test.ts src\lib\ui\library-catalog-model.ts src\lib\ui\library-catalog-model.test.ts
git commit -m "feat: add library catalog frontend contract"
```

---

## Task 3: Standalone Library Workflow And Route

**Files:**
- Modify: `src/lib/ui/library-catalog-workflow.ts`
- Modify: `src/lib/ui/library-catalog-workflow.test.ts`
- Modify: `src/routes/projects/library/+page.svelte`
- Modify: `src/lib/components/research-projects/LibraryScreen.svelte`
- Modify: `src/lib/library-prototype-contract.test.ts`

- [ ] **Step 1: Write failing workflow and route tests**

In `src/lib/ui/library-catalog-workflow.test.ts`, remove the `SourceJobRecord` import and the `job()` helper.

Change `LibraryCatalogWorkflowState` creation to:

```ts
  const state: LibraryCatalogWorkflowState = {
    catalogRecords: [],
    filterCounts: [],
    sources: [],
    loading: false,
    status: "",
    ...initial,
  };
```

Change deps creation to:

```ts
  const deps = {
    getState: () => state,
    patch: vi.fn((patch: Partial<LibraryCatalogWorkflowState>) => Object.assign(state, patch)),
    listCatalog: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };
```

Change the load success test to:

```ts
  it("loads backend catalog records into catalog rows", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listCatalog.mockResolvedValueOnce({
      sources: [
        catalogRecord({
          source: record(),
          status: "syncing",
          status_detail: "Syncing",
        }),
      ],
      filter_counts: [
        {
          provider: "youtube",
          source_subtype: "video",
          count: 1,
          disabled: false,
          disabled_reason: null,
        },
      ],
    });

    await workflow.loadLibrary();

    expect(deps.listCatalog).toHaveBeenCalledTimes(1);
    expect(state.catalogRecords).toHaveLength(1);
    expect(state.filterCounts).toHaveLength(1);
    expect(state.sources[0]).toEqual(
      expect.objectContaining({
        sourceId: 1,
        title: "Video title",
        status: "syncing",
      }),
    );
    expect(state.loading).toBe(false);
    expect(state.status).toBe("");
  });
```

Add this helper near the existing `record` helper:

```ts
function catalogRecord(overrides: Partial<LibraryCatalogRecord> = {}): LibraryCatalogRecord {
  return {
    source: record(),
    latest_job: null,
    status: "active",
    status_detail: null,
    capabilities: {
      can_refresh_source: true,
      can_delete: true,
      can_edit: false,
      can_connect_to_project: true,
    },
    disabled_reasons: {
      refresh_source: null,
      delete: null,
      edit: "Source editing is not available yet.",
      connect_to_project: null,
    },
    ...overrides,
  };
}
```

Update the import from `$lib/types/library-sources` to include `LibraryCatalogRecord`.

In `src/lib/library-prototype-contract.test.ts`, update the route test:

```ts
    expect(routeSource).toContain("listLibraryCatalog");
    expect(routeSource).not.toContain("listLibrarySources");
    expect(routeSource).not.toContain("listSourceJobs");
```

Update the screen coordination test:

```ts
    expect(screenSource).toContain("buildLibraryCatalogFilterTree(workflowState.filterCounts)");
```

- [ ] **Step 2: Run workflow tests and verify failure**

Run:

```powershell
npm.cmd test -- --run src/lib/ui/library-catalog-workflow.test.ts src/lib/library-prototype-contract.test.ts
```

Expected: FAIL because workflow and route still use `listSourceJobs`.

- [ ] **Step 3: Update Library catalog workflow**

Replace `src/lib/ui/library-catalog-workflow.ts` with this shape:

```ts
import type {
  LibraryCatalogFilterCount,
  LibraryCatalogRecord,
  LibraryCatalogResponse,
} from "$lib/types/library-sources";
import {
  buildLibraryCatalogSourcesView,
  type LibraryCatalogSourceView,
} from "./library-catalog-model";

export interface LibraryCatalogWorkflowState {
  catalogRecords: LibraryCatalogRecord[];
  filterCounts: LibraryCatalogFilterCount[];
  sources: LibraryCatalogSourceView[];
  loading: boolean;
  status: string;
}

export interface LibraryCatalogWorkflowDeps {
  getState(): LibraryCatalogWorkflowState;
  patch(patch: Partial<LibraryCatalogWorkflowState>): void;
  listCatalog(): Promise<LibraryCatalogResponse>;
  formatError(action: string, error: unknown): string;
}

export function createLibraryCatalogWorkflow(deps: LibraryCatalogWorkflowDeps) {
  function refreshDerivedState() {
    const state = deps.getState();
    deps.patch({
      sources: buildLibraryCatalogSourcesView(state.catalogRecords),
    });
  }

  async function loadLibrary() {
    deps.patch({ loading: true, status: "" });
    try {
      const catalog = await deps.listCatalog();
      deps.patch({
        catalogRecords: catalog.sources,
        filterCounts: catalog.filter_counts,
      });
      refreshDerivedState();
    } catch (error) {
      deps.patch({ status: deps.formatError("loading library catalog", error) });
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

- [ ] **Step 4: Update `/projects/library` route**

In `src/routes/projects/library/+page.svelte`, replace the script block with:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import LibraryScreen from "$lib/components/research-projects/LibraryScreen.svelte";
  import { listLibraryCatalog } from "$lib/api/library-sources";
  import {
    createLibraryCatalogWorkflow,
    type LibraryCatalogWorkflowState,
  } from "$lib/ui/library-catalog-workflow";

  const state = $state<LibraryCatalogWorkflowState>({
    catalogRecords: [],
    filterCounts: [],
    sources: [],
    loading: false,
    status: "",
  });

  const workflow = createLibraryCatalogWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listCatalog: listLibraryCatalog,
    formatError: (action, error) => `Error ${action}: ${String(error)}`,
  });

  onMount(() => {
    void workflow.loadLibrary();
  });
</script>
```

Keep the existing markup below the script.

- [ ] **Step 5: Update Library screen filter source**

In `src/lib/components/research-projects/LibraryScreen.svelte`, change:

```svelte
  let filterRows = $derived(buildLibraryCatalogFilterTree(workflowState.sources));
```

to:

```svelte
  let filterRows = $derived(buildLibraryCatalogFilterTree(workflowState.filterCounts));
```

- [ ] **Step 6: Run Svelte autofixer for modified Svelte files**

Run `mcp__svelte_server__.svelte_autofixer` for:

- `src/routes/projects/library/+page.svelte`
- `src/lib/components/research-projects/LibraryScreen.svelte`

Use desired Svelte version `5`.

Expected: no blocking issues. Apply any concrete fixes the autofixer reports.

- [ ] **Step 7: Run focused tests**

Run:

```powershell
npm.cmd test -- --run src/lib/ui/library-catalog-workflow.test.ts src/lib/ui/library-catalog-model.test.ts src/lib/library-prototype-contract.test.ts
npm.cmd run check
```

Expected: all PASS and `svelte-check` reports 0 errors.

- [ ] **Step 8: Commit standalone Library migration**

Run:

```powershell
git add src\lib\ui\library-catalog-workflow.ts src\lib\ui\library-catalog-workflow.test.ts src\routes\projects\library\+page.svelte src\lib\components\research-projects\LibraryScreen.svelte src\lib\library-prototype-contract.test.ts
git commit -m "feat: load library screen from catalog contract"
```

---

## Task 4: Projects Workflow Uses Catalog Source Rows

**Files:**
- Modify: `src/lib/ui/research-projects-model.ts`
- Modify: `src/lib/ui/research-projects-model.test.ts`
- Modify: `src/lib/ui/research-projects-workflow.ts`
- Modify: `src/lib/ui/research-projects-workflow.test.ts`
- Modify: `src/routes/projects/+page.svelte`
- Modify: `src/lib/research-projects-route-contract.test.ts`

- [ ] **Step 1: Write failing Projects model tests**

In `src/lib/ui/research-projects-model.test.ts`, change the Library type import:

```ts
import type { LibraryCatalogRecord } from "$lib/types/library-sources";
```

Replace `const library: LibrarySourceRecord[] = [` with:

```ts
const library: LibraryCatalogRecord[] = [
  {
    source: {
      source_id: 10,
      provider: "youtube",
      source_subtype: "video",
      account_id: null,
      external_id: "v1",
      title: "Video",
      subtitle: "Channel",
      canonical_url: "https://youtu.be/v1",
      created_at: 100,
      last_synced_at: 110,
      item_count: 3,
      project_count: 1,
      youtube: {
        video_form: "video",
        duration_seconds: 120,
        playlist_video_count: null,
        channel_title: "Channel",
        availability_status: "available",
      },
      telegram: null,
    },
    latest_job: null,
    status: "error",
    status_detail: "Last sync failed",
    capabilities: {
      can_refresh_source: true,
      can_delete: false,
      can_edit: false,
      can_connect_to_project: true,
    },
    disabled_reasons: {
      refresh_source: null,
      delete: "Source 10 is used by 1 project(s). Remove it from projects first.",
      edit: "Source editing is not available yet.",
      connect_to_project: null,
    },
  },
];
```

Add this test:

```ts
  it("uses catalog disabled reasons as project Library source base state", () => {
    const rows = buildLibrarySourcesView(
      [
        {
          ...library[0],
          capabilities: {
            ...library[0].capabilities,
            can_connect_to_project: false,
          },
          disabled_reasons: {
            ...library[0].disabled_reasons,
            connect_to_project: "Source type cannot be connected.",
          },
        },
      ],
      [],
      "project:1",
    );

    expect(rows[0]).toMatchObject({
      status: "error",
      disabledReason: "Source type cannot be connected.",
      connectable: false,
    });
  });
```

Update the existing already-connected test to call `buildLibrarySourcesView(library, projectSources, "project:1")` with no source-job argument and keep expecting `Already in project`.

- [ ] **Step 2: Write failing Projects workflow and route tests**

In `src/lib/ui/research-projects-workflow.test.ts`, replace `LibrarySourceRecord` import with `LibraryCatalogRecord` and update helper:

```ts
function libraryCatalogRecord(overrides: Partial<LibraryCatalogRecord> = {}): LibraryCatalogRecord {
  return {
    source: librarySource(),
    latest_job: null,
    status: "active",
    status_detail: null,
    capabilities: {
      can_refresh_source: true,
      can_delete: true,
      can_edit: false,
      can_connect_to_project: true,
    },
    disabled_reasons: {
      refresh_source: null,
      delete: null,
      edit: "Source editing is not available yet.",
      connect_to_project: null,
    },
    ...overrides,
  };
}
```

Keep `librarySource()` as a helper returning the nested source object.

In `createInitialState()`, replace `libraryRecords: []` with:

```ts
    libraryCatalogRecords: [],
```

In deps, replace `listLibrarySources: vi.fn()` with:

```ts
    listLibraryCatalog: vi.fn(),
```

In tests, replace:

```ts
deps.listLibrarySources.mockResolvedValue([librarySource()]);
```

with:

```ts
deps.listLibraryCatalog.mockResolvedValue({
  sources: [libraryCatalogRecord()],
  filter_counts: [],
});
```

Update the source-jobs derived state test assertion:

```ts
    expect(state.sourceJobs).toHaveLength(1);
    expect(state.librarySources[0].status).toBe("active");
```

Add:

```ts
    expect(deps.listSourceJobs).toHaveBeenCalledTimes(1);
```

This proves `listSourceJobs` remains for bottom queue state, but not for source row status.

In `src/lib/research-projects-route-contract.test.ts`, update the first test:

```ts
    expect(pageSource).toContain("listLibraryCatalog");
    expect(pageSource).not.toContain("listLibrarySources");
```

- [ ] **Step 3: Run Projects tests and verify failure**

Run:

```powershell
npm.cmd test -- --run src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/research-projects-route-contract.test.ts
```

Expected: FAIL because Projects still uses raw Library sources.

- [ ] **Step 4: Update Projects model**

In `src/lib/ui/research-projects-model.ts`, replace the Library import:

```ts
import type { LibraryCatalogRecord, LibrarySourceProvider } from "$lib/types/library-sources";
```

Remove the `SourceJobRecord` import if no longer used.

Remove `activeJobBySource` and `jobBlockedState`.

Change `buildLibrarySourcesView` signature and body:

```ts
export function buildLibrarySourcesView(
  catalogRecords: LibraryCatalogRecord[],
  projectSources: ProjectSourceRecord[],
  selectedProjectId: string | null,
): LibrarySourceView[] {
  const projectId = projectIdFromViewId(selectedProjectId);
  const connectedIds = new Set(
    projectSources
      .filter((source) => projectId !== null && source.project_id === projectId)
      .map((source) => source.source_id),
  );

  return catalogRecords.map((record) => {
    const source = record.source;
    const alreadyConnected = connectedIds.has(source.source_id);
    const catalogDisabledReason = record.disabled_reasons.connect_to_project;
    const disabledReason = alreadyConnected ? "Already in project" : catalogDisabledReason;
    const connectable = disabledReason === null && record.capabilities.can_connect_to_project;

    return {
      id: sourceRowId(source.source_id),
      sourceId: source.source_id,
      provider: source.provider,
      title: source.title ?? `Source #${source.source_id}`,
      subtitle: source.subtitle,
      projectCount: source.project_count,
      lastCollectedLabel: dateLabel(source.last_synced_at),
      localCopyLabel: materialLabel(source.item_count),
      status: record.status,
      disabledReason,
      alreadyConnected,
      connectable,
    };
  });
}
```

- [ ] **Step 5: Update Projects workflow**

In `src/lib/ui/research-projects-workflow.ts`, replace the Library type import:

```ts
import type { LibraryCatalogRecord, LibraryCatalogResponse } from "$lib/types/library-sources";
```

In `ResearchProjectsWorkflowState`, replace:

```ts
  libraryRecords: LibrarySourceRecord[];
```

with:

```ts
  libraryCatalogRecords: LibraryCatalogRecord[];
```

In deps, replace:

```ts
  listLibrarySources(): Promise<LibrarySourceRecord[]>;
```

with:

```ts
  listLibraryCatalog(): Promise<LibraryCatalogResponse>;
```

In `refreshDerivedState`, replace:

```ts
      state.libraryRecords,
      state.projectSources,
      selectedProjectId,
      state.sourceJobs,
```

with:

```ts
      state.libraryCatalogRecords,
      state.projectSources,
      selectedProjectId,
```

In `loadWorkspace`, replace the Promise binding:

```ts
      const [projectsRaw, libraryCatalog, sourceJobs, promptTemplates] = await Promise.all([
        deps.listProjects(),
        deps.listLibraryCatalog(),
        deps.listSourceJobs(),
        deps.listPromptTemplates(),
      ]);
```

and patch:

```ts
        libraryCatalogRecords: libraryCatalog.sources,
```

Remove the old `libraryRecords` patch.

- [ ] **Step 6: Update Projects route**

In `src/routes/projects/+page.svelte`, replace:

```svelte
  import { listLibrarySources } from "$lib/api/library-sources";
```

with:

```svelte
  import { listLibraryCatalog } from "$lib/api/library-sources";
```

In initial state, replace:

```ts
    libraryRecords: [],
```

with:

```ts
    libraryCatalogRecords: [],
```

In workflow deps, replace:

```ts
    listLibrarySources,
```

with:

```ts
    listLibraryCatalog,
```

Keep:

```ts
    listSourceJobs: () => listSourceJobs({ limit: 50 }),
```

This is still needed by `BottomQueue`.

- [ ] **Step 7: Run Svelte autofixer for modified route**

Run `mcp__svelte_server__.svelte_autofixer` for `src/routes/projects/+page.svelte` with desired Svelte version `5`.

Expected: no blocking issues. Apply any concrete fixes the autofixer reports.

- [ ] **Step 8: Run focused Projects tests**

Run:

```powershell
npm.cmd test -- --run src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/research-projects-route-contract.test.ts
npm.cmd run check
```

Expected: all PASS and `svelte-check` reports 0 errors.

- [ ] **Step 9: Commit Projects catalog migration**

Run:

```powershell
git add src\lib\ui\research-projects-model.ts src\lib\ui\research-projects-model.test.ts src\lib\ui\research-projects-workflow.ts src\lib\ui\research-projects-workflow.test.ts src\routes\projects\+page.svelte src\lib\research-projects-route-contract.test.ts
git commit -m "feat: use catalog records in projects library rows"
```

---

## Task 5: Final Verification And Tauri MCP Smoke

**Files:**
- No planned source edits unless verification exposes a defect.

- [ ] **Step 1: Run backend verification**

Run:

```powershell
cargo test library_sources --manifest-path src-tauri/Cargo.toml
cargo test catalog_jobs_for_sources --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all PASS.

- [ ] **Step 2: Run frontend verification**

Run:

```powershell
npm.cmd test -- --run src/lib/api/library-sources.test.ts src/lib/ui/library-catalog-model.test.ts src/lib/ui/library-catalog-workflow.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/library-prototype-contract.test.ts src/lib/research-projects-route-contract.test.ts
npm.cmd run check
```

Expected: all PASS and `svelte-check` reports 0 errors.

- [ ] **Step 3: Run full project verification**

Run:

```powershell
npm.cmd run verify
```

Expected: full verification PASS.

- [ ] **Step 4: Start Tauri app for UI smoke**

If no app is already running, start it with the existing hidden dev-process pattern:

```powershell
Start-Process -FilePath "npm.cmd" -ArgumentList @("run","tauri","dev") -WorkingDirectory "G:\Develop\Extractum" -WindowStyle Hidden -RedirectStandardOutput "G:\Develop\Extractum\tmp\tauri-dev.out.log" -RedirectStandardError "G:\Develop\Extractum\tmp\tauri-dev.err.log"
```

Then connect:

```text
mcp__tauri__.driver_session action=start host=localhost port=9223
```

Expected: bridge session starts and reports connected app metadata.

- [ ] **Step 5: Smoke `/projects/library` with Tauri MCP bridge**

Use Tauri MCP DOM snapshots and interactions.

Verify:

- `/projects/library` route loads.
- Library table rows render.
- Filter rail renders backend-backed counts.
- YouTube channel filter row is disabled with backend disabled reason.
- Selecting a source updates Inspector metadata.
- Route-level Refresh reloads catalog.
- No visible error status appears after load.

- [ ] **Step 6: Smoke `/projects` with Tauri MCP bridge**

Use Tauri MCP DOM snapshots and interactions.

Verify:

- `/projects` route loads.
- Add from Library still opens.
- Already connected source rows still show `Already in project`.
- Bottom queue still renders active source jobs when present.
- Library row status comes from catalog response; it does not require route-level `sourceJobs` in the model path.

- [ ] **Step 7: Stop Tauri session and dev process if started by this task**

Run:

```text
mcp__tauri__.driver_session action=stop
```

If this task started the app, stop only the spawned dev process and confirm no `:1420` listener remains:

```powershell
netstat -ano | Select-String ':1420'
```

Expected: no listener remains after stopping the process.

- [ ] **Step 8: Commit verification note only if a note is created**

If a verification note is created, use:

```powershell
git add docs\superpowers\verification\2026-06-14-library-catalog-backend-contract.md
git commit -m "test: verify library catalog contract"
```

If no verification note is created, do not create an empty commit.

---

## Plan Self-Review

- Spec coverage:
  - `list_library_catalog` backend command: Task 1.
  - latest queued/running/failed relevant job: Task 1.
  - source status and status detail from backend: Task 1 and Task 2.
  - capabilities and disabled reasons: Task 1 and Task 4.
  - provider/subtype filter counts from backend: Task 1, Task 2, and Task 3.
  - `/projects/library` one catalog API: Task 3.
  - `/projects` catalog-backed Library rows while keeping bottom queue jobs: Task 4.
  - old `list_library_sources` compatibility: Task 1 and Task 2 keep the old wrapper and command.
  - no schema migration: no migration task is included.
- Placeholder scan:
  - No placeholder markers are intentionally left.
  - Each code-changing step includes the concrete snippet or replacement needed for the implementation.
- Type consistency:
  - Rust uses `can_refresh_source` and `refresh_source`.
  - TypeScript uses the same snake_case catalog contract fields as Rust serialization.
  - Library workflow uses `catalogRecords` and `filterCounts`.
  - Projects workflow uses `libraryCatalogRecords` and keeps `sourceJobs` only for queue state.
