# Apalis Jobs Inspector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a top-level read-only `/jobs` inspector for all Apalis jobs with manual refresh, server-side filters, counts, and safe payload previews.

**Architecture:** Add a focused Rust `apalis_jobs` read-model module that reads the existing Apalis SQLite `Jobs` table through the app database pool and exposes one Tauri command, `apalis_jobs_list`. Add a narrow TS API/types layer and a Svelte split inspector route that never mutates jobs and reloads through the backend whenever filters change. Wire the route as its own sidebar item in both legacy and projects navigation modes.

**Tech Stack:** Tauri 2, Rust 2021, sqlx SQLite, apalis `=1.0.0-rc.8`, apalis-sqlite `=1.0.0-rc.8`, serde, time, Svelte 5, Vitest, lucide-svelte.

---

## File Structure

- Create `src-tauri/src/apalis_jobs.rs`
  - Owns request/response DTOs, schema discovery, read-only SQL queries, sorting, timestamp normalization, payload redaction, payload truncation, and backend tests.
- Modify `src-tauri/src/lib.rs`
  - Registers the `apalis_jobs` module and `apalis_jobs_list` Tauri command.
- Create `src/lib/types/apalis-jobs.ts`
  - Defines stable frontend DTO types in camelCase matching Rust `#[serde(rename_all = "camelCase")]`.
- Create `src/lib/api/apalis-jobs.ts`
  - Wraps `invoke("apalis_jobs_list", { request })`.
- Create `src/lib/api/apalis-jobs.test.ts`
  - Verifies the API wrapper command name and request payload.
- Create `src/lib/apalis-jobs-route-contract.test.ts`
  - Source-level route/navigation/UI contract tests for command isolation, manual refresh, server-side filtering, read-only UI, and navigation.
- Create `src/lib/components/jobs/ApalisJobsPanel.svelte`
  - Implements split inspector UI, filter controls, manual refresh, table, detail panel, loading/empty/error states, and selection handling.
- Create `src/routes/jobs/+page.svelte`
  - Adds the top-level route and delegates to `ApalisJobsPanel`.
- Modify `src/routes/+layout.svelte`
  - Adds `Jobs` to both nav modes and shows `Jobs` in the topbar route label.

---

## Task 1: Backend Command Shell And Local Schema Probe

**Files:**
- Create: `src-tauri/src/apalis_jobs.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing backend tests for local Apalis schema and empty missing-table response**

Create `src-tauri/src/apalis_jobs.rs` with DTOs, an intentional empty implementation, and these tests at the bottom of the file:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use tauri::AppHandle;

const DEFAULT_LIMIT: u32 = 100;
const MAX_LIMIT: u32 = 500;
const JOBS_TABLE: &str = "Jobs";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobsListRequest {
    pub limit: Option<u32>,
    pub status: Option<String>,
    pub job_type: Option<String>,
    pub search: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobsListResponse {
    pub jobs: Vec<ApalisJobRow>,
    pub total_matching: u32,
    pub status_counts: Vec<ApalisJobStatusCount>,
    pub job_type_counts: Vec<ApalisJobTypeCount>,
    pub refreshed_at: String,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobRow {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub attempts: u32,
    pub max_attempts: Option<u32>,
    pub run_at: Option<String>,
    pub lock_at: Option<String>,
    pub lock_by: Option<String>,
    pub done_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub priority: Option<u32>,
    pub idempotency_key: Option<String>,
    pub job_preview: Option<String>,
    pub job_truncated: bool,
    pub job_json: Option<Value>,
    pub last_result: Option<Value>,
    pub last_result_truncated: bool,
    pub metadata: Option<Value>,
    pub metadata_truncated: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobStatusCount {
    pub status: String,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobTypeCount {
    pub job_type: String,
    pub count: u32,
}

#[tauri::command]
pub(crate) async fn apalis_jobs_list(
    handle: AppHandle,
    request: Option<ApalisJobsListRequest>,
) -> crate::error::AppResult<ApalisJobsListResponse> {
    let pool = crate::db::get_pool(&handle).await?;
    apalis_jobs_list_from_pool(&pool, request.unwrap_or_default()).await
}

async fn apalis_jobs_list_from_pool(
    _pool: &SqlitePool,
    request: ApalisJobsListRequest,
) -> crate::error::AppResult<ApalisJobsListResponse> {
    Ok(empty_response(normalized_limit(request.limit)))
}

fn normalized_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

fn empty_response(limit: u32) -> ApalisJobsListResponse {
    ApalisJobsListResponse {
        jobs: Vec::new(),
        total_matching: 0,
        status_counts: Vec::new(),
        job_type_counts: Vec::new(),
        refreshed_at: crate::time::now_rfc3339_utc(),
        limit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apalis::prelude::TaskSink;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn memory_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("open memory sqlite")
    }

    fn test_job(run_id: &str) -> crate::gemini_browser::jobs::GeminiBrowserJob {
        crate::gemini_browser::jobs::GeminiBrowserJob {
            run_id: run_id.to_string(),
            prompt: format!("Prompt for {run_id}"),
            source: "apalis-jobs-test".to_string(),
            artifact_mode: crate::gemini_browser::jobs::GeminiBrowserArtifactMode::Reduced,
            browser_config: None,
        }
    }

    async fn seed_apalis_job(pool: &SqlitePool, run_id: &str) {
        crate::gemini_browser::jobs::setup_gemini_browser_apalis_storage(pool)
            .await
            .expect("setup apalis sqlite storage");
        let mut storage = crate::gemini_browser::jobs::open_gemini_browser_job_storage(pool)
            .await
            .expect("open gemini browser storage");
        storage
            .push(test_job(run_id))
            .await
            .expect("push apalis task");
    }

    async fn table_columns(pool: &SqlitePool, table: &str) -> Vec<String> {
        sqlx::query(&format!("PRAGMA table_info('{table}')"))
            .fetch_all(pool)
            .await
            .expect("read table info")
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect()
    }

    #[tokio::test]
    async fn apalis_jobs_schema_probe_documents_local_jobs_table_shape() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "schema-probe").await;

        let columns = table_columns(&pool, JOBS_TABLE).await;
        for expected in [
            "id",
            "job_type",
            "status",
            "attempts",
            "max_attempts",
            "run_at",
            "last_result",
            "lock_at",
            "lock_by",
            "done_at",
            "priority",
            "job",
            "metadata",
            "idempotency_key",
        ] {
            assert!(
                columns.iter().any(|column| column == expected),
                "missing Apalis Jobs column {expected}; actual columns: {columns:?}"
            );
        }

        let row = sqlx::query("SELECT job_type, status, idempotency_key FROM Jobs LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("read seeded job");
        assert_eq!(row.get::<String, _>("job_type"), "gemini-browser");
        assert_eq!(row.get::<String, _>("status"), "Pending");
        assert_eq!(row.get::<String, _>("idempotency_key"), "schema-probe");
    }

    #[tokio::test]
    async fn apalis_jobs_list_returns_empty_when_jobs_table_missing() {
        let pool = memory_pool().await;

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("missing Jobs table is not fatal");

        assert!(response.jobs.is_empty());
        assert_eq!(response.total_matching, 0);
        assert!(response.status_counts.is_empty());
        assert!(response.job_type_counts.is_empty());
        assert_eq!(response.limit, DEFAULT_LIMIT);
        assert!(response.refreshed_at.ends_with('Z'));
    }
}
```

- [ ] **Step 2: Run tests to verify the read model test fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
```

Expected: `apalis_jobs_schema_probe_documents_local_jobs_table_shape` passes and no production read rows exist yet. If `storage.push(...)` does not compile, switch that one line to the existing helper:

```rust
crate::gemini_browser::jobs::enqueue_gemini_browser_job_to_storage(&mut storage, test_job(run_id))
    .await
    .expect("enqueue apalis task");
```

- [ ] **Step 3: Register the command in `src-tauri/src/lib.rs`**

Add the module and import near the other backend modules:

```rust
mod apalis_jobs;
use apalis_jobs::apalis_jobs_list;
```

Add the command to `tauri::generate_handler![...]` immediately after `get_diagnostic_summary`:

```rust
apalis_jobs_list,
```

- [ ] **Step 4: Run the backend tests again**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
```

Expected: both tests pass.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/apalis_jobs.rs src-tauri/src/lib.rs
git commit -m "feat: add apalis jobs inspector command shell"
```

---

## Task 2: Backend Read Model, Filters, Counts, Sorting, And Timestamps

**Files:**
- Modify: `src-tauri/src/apalis_jobs.rs`

- [ ] **Step 1: Add failing read-model tests**

Append these tests inside the existing `#[cfg(test)] mod tests`:

```rust
    async fn update_job_row(
        pool: &SqlitePool,
        idempotency_key: &str,
        status: &str,
        run_at: Option<&str>,
        lock_at: Option<&str>,
        done_at: Option<&str>,
    ) {
        sqlx::query(
            "UPDATE Jobs
             SET status = ?, run_at = ?, lock_at = ?, done_at = ?
             WHERE idempotency_key = ?",
        )
        .bind(status)
        .bind(run_at)
        .bind(lock_at)
        .bind(done_at)
        .bind(idempotency_key)
        .execute(pool)
        .await
        .expect("update Jobs row");
    }

    #[tokio::test]
    async fn apalis_jobs_list_returns_rows_from_jobs_table() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "row-1").await;

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list apalis jobs");

        assert_eq!(response.jobs.len(), 1);
        let job = &response.jobs[0];
        assert!(!job.id.is_empty());
        assert_eq!(job.job_type, "gemini-browser");
        assert_eq!(job.status, "Pending");
        assert_eq!(job.attempts, 0);
        assert_eq!(job.idempotency_key.as_deref(), Some("row-1"));
        assert!(job.run_at.is_some());
        assert!(job.job_json.is_some());
        assert_eq!(response.total_matching, 1);
    }

    #[tokio::test]
    async fn apalis_jobs_list_filters_by_status_job_type_and_search() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "search-one").await;
        seed_apalis_job(&pool, "search-two").await;
        update_job_row(&pool, "search-two", "Failed", None, None, Some("2026-06-23T10:10:00Z")).await;

        let response = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: None,
                status: Some("Failed".to_string()),
                job_type: Some("gemini-browser".to_string()),
                search: Some("two".to_string()),
            },
        )
        .await
        .expect("list filtered apalis jobs");

        assert_eq!(response.jobs.len(), 1);
        assert_eq!(response.jobs[0].idempotency_key.as_deref(), Some("search-two"));
        assert_eq!(response.jobs[0].status, "Failed");
        assert_eq!(response.total_matching, 1);
    }

    #[tokio::test]
    async fn apalis_jobs_list_clamps_limit() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "limit-1").await;

        let low = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: Some(0),
                ..Default::default()
            },
        )
        .await
        .expect("list with low limit");
        let high = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: Some(999),
                ..Default::default()
            },
        )
        .await
        .expect("list with high limit");

        assert_eq!(low.limit, 1);
        assert_eq!(high.limit, MAX_LIMIT);
    }

    #[tokio::test]
    async fn apalis_jobs_list_does_not_mutate_jobs() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "no-mutate").await;
        let before: Vec<(String, String, i64)> =
            sqlx::query_as("SELECT id, status, attempts FROM Jobs ORDER BY id")
                .fetch_all(&pool)
                .await
                .expect("read before");

        let _response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list jobs");

        let after: Vec<(String, String, i64)> =
            sqlx::query_as("SELECT id, status, attempts FROM Jobs ORDER BY id")
                .fetch_all(&pool)
                .await
                .expect("read after");
        assert_eq!(after, before);
    }

    #[tokio::test]
    async fn apalis_jobs_list_sorts_by_latest_activity_timestamp() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "old-done").await;
        seed_apalis_job(&pool, "new-lock").await;
        seed_apalis_job(&pool, "new-done").await;
        update_job_row(&pool, "old-done", "Done", Some("2026-06-23T08:00:00Z"), None, Some("2026-06-23T09:00:00Z")).await;
        update_job_row(&pool, "new-lock", "Running", Some("2026-06-23T07:00:00Z"), Some("2026-06-23T11:00:00Z"), None).await;
        update_job_row(&pool, "new-done", "Done", Some("2026-06-23T06:00:00Z"), Some("2026-06-23T08:30:00Z"), Some("2026-06-23T12:00:00Z")).await;

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list sorted jobs");

        let keys = response
            .jobs
            .iter()
            .map(|job| job.idempotency_key.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(keys, vec![Some("new-done"), Some("new-lock"), Some("old-done")]);
        assert_eq!(
            response.jobs[0].last_activity_at.as_deref(),
            Some("2026-06-23T12:00:00Z")
        );
    }

    #[tokio::test]
    async fn apalis_jobs_list_returns_rfc3339_utc_timestamps() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "timestamps").await;
        update_job_row(
            &pool,
            "timestamps",
            "Done",
            Some("2026-06-23 08:00:00"),
            Some("1719129600"),
            Some("2026-06-23T12:00:00+03:00"),
        )
        .await;

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list timestamp jobs");
        let job = &response.jobs[0];

        assert_eq!(job.run_at.as_deref(), Some("2026-06-23T08:00:00Z"));
        assert_eq!(job.lock_at.as_deref(), Some("2024-06-23T08:00:00Z"));
        assert_eq!(job.done_at.as_deref(), Some("2026-06-23T09:00:00Z"));
    }

    #[tokio::test]
    async fn apalis_jobs_counts_ignore_their_own_active_filter() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "count-pending").await;
        seed_apalis_job(&pool, "count-failed").await;
        update_job_row(&pool, "count-failed", "Failed", None, None, Some("2026-06-23T10:00:00Z")).await;

        let response = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: None,
                status: Some("Failed".to_string()),
                job_type: Some("gemini-browser".to_string()),
                search: Some("count".to_string()),
            },
        )
        .await
        .expect("list count jobs");

        assert_eq!(response.total_matching, 1);
        assert_eq!(
            response.status_counts,
            vec![
                ApalisJobStatusCount {
                    status: "Failed".to_string(),
                    count: 1,
                },
                ApalisJobStatusCount {
                    status: "Pending".to_string(),
                    count: 1,
                },
            ]
        );
        assert_eq!(
            response.job_type_counts,
            vec![ApalisJobTypeCount {
                job_type: "gemini-browser".to_string(),
                count: 1,
            }]
        );
    }
```

- [ ] **Step 2: Run tests to verify failures**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
```

Expected: new tests fail because `apalis_jobs_list_from_pool` still returns an empty response.

- [ ] **Step 3: Implement schema discovery, filters, counts, sorting, and timestamp normalization**

Replace `apalis_jobs_list_from_pool` and add these helpers above the test module:

```rust
#[derive(Clone, Debug)]
struct ApalisJobsFilters {
    limit: u32,
    status: Option<String>,
    job_type: Option<String>,
    search: Option<String>,
}

#[derive(Clone, Debug)]
struct JobsTableSchema {
    columns: std::collections::HashSet<String>,
}

impl JobsTableSchema {
    fn has(&self, column: &str) -> bool {
        self.columns.contains(column)
    }
}

fn normalize_filter(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn filters_from_request(request: ApalisJobsListRequest) -> ApalisJobsFilters {
    ApalisJobsFilters {
        limit: normalized_limit(request.limit),
        status: normalize_filter(request.status),
        job_type: normalize_filter(request.job_type),
        search: normalize_filter(request.search),
    }
}

async fn jobs_table_schema(pool: &SqlitePool) -> crate::error::AppResult<Option<JobsTableSchema>> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ? LIMIT 1",
    )
    .bind(JOBS_TABLE)
    .fetch_optional(pool)
    .await
    .map_err(crate::error::AppError::database)?;

    if exists.is_none() {
        return Ok(None);
    }

    let rows = sqlx::query("PRAGMA table_info('Jobs')")
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::database)?;
    let columns = rows
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect::<std::collections::HashSet<_>>();

    Ok(Some(JobsTableSchema { columns }))
}

async fn apalis_jobs_list_from_pool(
    pool: &SqlitePool,
    request: ApalisJobsListRequest,
) -> crate::error::AppResult<ApalisJobsListResponse> {
    let filters = filters_from_request(request);
    let Some(schema) = jobs_table_schema(pool).await? else {
        return Ok(empty_response(filters.limit));
    };

    let all_rows = fetch_jobs_rows(pool, &schema).await?;
    let mut matching = all_rows
        .iter()
        .filter(|row| matches_filters(row, &filters, true, true))
        .cloned()
        .collect::<Vec<_>>();
    matching.sort_by(|left, right| compare_jobs(right, left));

    let total_matching = matching.len() as u32;
    let jobs = matching
        .into_iter()
        .take(filters.limit as usize)
        .map(|row| row.into_dto())
        .collect::<Vec<_>>();

    Ok(ApalisJobsListResponse {
        jobs,
        total_matching,
        status_counts: status_counts(&all_rows, &filters),
        job_type_counts: job_type_counts(&all_rows, &filters),
        refreshed_at: crate::time::now_rfc3339_utc(),
        limit: filters.limit,
    })
}

#[derive(Clone, Debug)]
struct InternalJobRow {
    id: String,
    job_type: String,
    status: String,
    attempts: u32,
    max_attempts: Option<u32>,
    run_at: Option<String>,
    lock_at: Option<String>,
    lock_by: Option<String>,
    done_at: Option<String>,
    last_activity_at: Option<String>,
    priority: Option<u32>,
    idempotency_key: Option<String>,
    job_raw: Option<Vec<u8>>,
    last_result_raw: Option<Vec<u8>>,
    metadata_raw: Option<Vec<u8>>,
}

impl InternalJobRow {
    fn into_dto(self) -> ApalisJobRow {
        ApalisJobRow {
            id: self.id,
            job_type: self.job_type,
            status: self.status,
            attempts: self.attempts,
            max_attempts: self.max_attempts,
            run_at: self.run_at,
            lock_at: self.lock_at,
            lock_by: self.lock_by,
            done_at: self.done_at,
            last_activity_at: self.last_activity_at,
            priority: self.priority,
            idempotency_key: self.idempotency_key,
            job_preview: None,
            job_truncated: false,
            job_json: None,
            last_result: None,
            last_result_truncated: false,
            metadata: None,
            metadata_truncated: false,
        }
    }
}

async fn fetch_jobs_rows(
    pool: &SqlitePool,
    schema: &JobsTableSchema,
) -> crate::error::AppResult<Vec<InternalJobRow>> {
    let rows = sqlx::query("SELECT * FROM Jobs")
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::database)?;

    Ok(rows
        .into_iter()
        .map(|row| internal_row_from_sql(row, schema))
        .collect())
}

fn internal_row_from_sql(row: sqlx::sqlite::SqliteRow, schema: &JobsTableSchema) -> InternalJobRow {
    let run_at = normalize_timestamp(optional_text(&row, schema, "run_at"));
    let lock_at = normalize_timestamp(optional_text(&row, schema, "lock_at"));
    let done_at = normalize_timestamp(optional_text(&row, schema, "done_at"));
    let last_activity_at = latest_timestamp([done_at.as_deref(), lock_at.as_deref(), run_at.as_deref()]);

    InternalJobRow {
        id: text_or_default(&row, schema, "id", ""),
        job_type: text_or_default(&row, schema, "job_type", ""),
        status: text_or_default(&row, schema, "status", "unknown"),
        attempts: optional_i64(&row, schema, "attempts").unwrap_or_default().max(0) as u32,
        max_attempts: optional_i64(&row, schema, "max_attempts").map(|value| value.max(0) as u32),
        run_at,
        lock_at,
        lock_by: optional_text(&row, schema, "lock_by"),
        done_at,
        last_activity_at,
        priority: optional_i64(&row, schema, "priority").map(|value| value.max(0) as u32),
        idempotency_key: optional_text(&row, schema, "idempotency_key"),
        job_raw: optional_blob_or_text(&row, schema, "job"),
        last_result_raw: optional_blob_or_text(&row, schema, "last_result"),
        metadata_raw: optional_blob_or_text(&row, schema, "metadata"),
    }
}

fn optional_text(row: &sqlx::sqlite::SqliteRow, schema: &JobsTableSchema, column: &str) -> Option<String> {
    if !schema.has(column) {
        return None;
    }
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn text_or_default(row: &sqlx::sqlite::SqliteRow, schema: &JobsTableSchema, column: &str, default: &str) -> String {
    optional_text(row, schema, column).unwrap_or_else(|| default.to_string())
}

fn optional_i64(row: &sqlx::sqlite::SqliteRow, schema: &JobsTableSchema, column: &str) -> Option<i64> {
    if !schema.has(column) {
        return None;
    }
    row.try_get::<Option<i64>, _>(column).ok().flatten()
}

fn optional_blob_or_text(row: &sqlx::sqlite::SqliteRow, schema: &JobsTableSchema, column: &str) -> Option<Vec<u8>> {
    if !schema.has(column) {
        return None;
    }
    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(column) {
        return value;
    }
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .map(String::into_bytes)
}

fn normalize_timestamp(value: Option<String>) -> Option<String> {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime, PrimitiveDateTime};

    let value = value?.trim().to_string();
    if value.is_empty() {
        return None;
    }

    if let Ok(epoch) = value.parse::<i64>() {
        return OffsetDateTime::from_unix_timestamp(epoch)
            .ok()
            .and_then(|timestamp| timestamp.format(&Rfc3339).ok());
    }

    if let Ok(timestamp) = OffsetDateTime::parse(&value, &Rfc3339) {
        return timestamp.to_offset(time::UtcOffset::UTC).format(&Rfc3339).ok();
    }

    if let Ok(timestamp) = PrimitiveDateTime::parse(
        &value,
        time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    ) {
        return timestamp.assume_utc().format(&Rfc3339).ok();
    }

    None
}

fn latest_timestamp(values: [Option<&str>; 3]) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .max_by_key(|value| *value)
        .map(ToOwned::to_owned)
}

fn compare_jobs(left: &InternalJobRow, right: &InternalJobRow) -> std::cmp::Ordering {
    left.last_activity_at
        .cmp(&right.last_activity_at)
        .then_with(|| left.id.cmp(&right.id))
}

fn matches_filters(
    row: &InternalJobRow,
    filters: &ApalisJobsFilters,
    include_status: bool,
    include_job_type: bool,
) -> bool {
    if include_status && filters.status.as_deref().is_some_and(|status| row.status != status) {
        return false;
    }
    if include_job_type && filters.job_type.as_deref().is_some_and(|job_type| row.job_type != job_type) {
        return false;
    }
    if let Some(search) = filters.search.as_deref() {
        let search = search.to_ascii_lowercase();
        let id_matches = row.id.to_ascii_lowercase().contains(&search);
        let key_matches = row
            .idempotency_key
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&search);
        if !id_matches && !key_matches {
            return false;
        }
    }
    true
}

fn status_counts(rows: &[InternalJobRow], filters: &ApalisJobsFilters) -> Vec<ApalisJobStatusCount> {
    let mut counts = std::collections::BTreeMap::<String, u32>::new();
    for row in rows {
        if matches_filters(row, filters, false, true) {
            *counts.entry(row.status.clone()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(status, count)| ApalisJobStatusCount { status, count })
        .collect()
}

fn job_type_counts(rows: &[InternalJobRow], filters: &ApalisJobsFilters) -> Vec<ApalisJobTypeCount> {
    let mut counts = std::collections::BTreeMap::<String, u32>::new();
    for row in rows {
        if matches_filters(row, filters, true, false) {
            *counts.entry(row.job_type.clone()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(job_type, count)| ApalisJobTypeCount { job_type, count })
        .collect()
}
```

- [ ] **Step 4: Run backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
```

Expected: read-model tests pass except payload-specific tests, which are added in the next task.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/apalis_jobs.rs
git commit -m "feat: read apalis jobs table"
```

---

## Task 3: Backend Payload Redaction, Truncation, And Stable Optional Shape

**Files:**
- Modify: `src-tauri/src/apalis_jobs.rs`

- [ ] **Step 1: Add failing payload and stable-shape tests**

Append these tests inside the existing test module:

```rust
    #[tokio::test]
    async fn apalis_jobs_payloads_are_redacted_and_truncated() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "payload-secret").await;
        let large_text = "x".repeat(70 * 1024);
        sqlx::query(
            "UPDATE Jobs
             SET job = ?, last_result = ?, metadata = ?
             WHERE idempotency_key = ?",
        )
        .bind(r#"{"apiKey":"sk-secret","nested":{"refresh-token":"rt-secret"},"prompt":"safe prompt"}"#)
        .bind(format!(r#"{{"message":"{large_text}","authorization":"Bearer secret"}}"#))
        .bind(r#"{"API Key":"another-secret","normal":"visible"}"#)
        .bind("payload-secret")
        .execute(&pool)
        .await
        .expect("update payload columns");

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list payload jobs");
        let job = &response.jobs[0];

        assert_eq!(job.job_json.as_ref().unwrap()["apiKey"], "[redacted]");
        assert_eq!(job.job_json.as_ref().unwrap()["nested"]["refresh-token"], "[redacted]");
        assert_eq!(job.metadata.as_ref().unwrap()["API Key"], "[redacted]");
        assert_eq!(job.metadata.as_ref().unwrap()["normal"], "visible");
        assert!(job.last_result_truncated);
        assert_eq!(job.last_result.as_ref().unwrap()["truncated"], true);
        assert!(
            job.last_result
                .as_ref()
                .unwrap()["preview"]
                .as_str()
                .unwrap()
                .chars()
                .count()
                <= 2000
        );
        let serialized = serde_json::to_string(job).expect("serialize response row");
        assert!(!serialized.contains("sk-secret"));
        assert!(!serialized.contains("rt-secret"));
        assert!(!serialized.contains("another-secret"));
        assert!(!serialized.contains("Bearer secret"));
    }

    #[tokio::test]
    async fn apalis_jobs_decode_failure_returns_redacted_preview_without_json() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "payload-invalid").await;
        sqlx::query("UPDATE Jobs SET job = ? WHERE idempotency_key = ?")
            .bind("apiKey=sk-secret; plain text payload")
            .bind("payload-invalid")
            .execute(&pool)
            .await
            .expect("update invalid payload");

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list payload jobs");
        let job = &response.jobs[0];

        assert_eq!(job.job_json, None);
        assert!(!job.job_truncated);
        assert_eq!(job.job_preview.as_deref(), Some("apiKey=[redacted]"));
    }

    #[tokio::test]
    async fn apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent() {
        let pool = memory_pool().await;
        sqlx::query(
            "CREATE TABLE Jobs (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                attempts INTEGER NOT NULL,
                job TEXT
             )",
        )
        .execute(&pool)
        .await
        .expect("create reduced Jobs table");
        sqlx::query("INSERT INTO Jobs (id, status, attempts, job) VALUES ('job-1', 'Queued', 2, '{\"safe\":true}')")
            .execute(&pool)
            .await
            .expect("insert reduced job");

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list reduced jobs");
        let serialized = serde_json::to_value(&response.jobs[0]).expect("serialize row");

        for key in [
            "id",
            "jobType",
            "status",
            "attempts",
            "maxAttempts",
            "runAt",
            "lockAt",
            "lockBy",
            "doneAt",
            "lastActivityAt",
            "priority",
            "idempotencyKey",
            "jobPreview",
            "jobTruncated",
            "jobJson",
            "lastResult",
            "lastResultTruncated",
            "metadata",
            "metadataTruncated",
        ] {
            assert!(serialized.get(key).is_some(), "missing serialized key {key}");
        }

        assert_eq!(response.jobs[0].job_type, "");
        assert_eq!(response.jobs[0].status, "Queued");
        assert_eq!(response.jobs[0].last_result, None);
        assert_eq!(response.jobs[0].metadata, None);
    }
```

- [ ] **Step 2: Run tests to verify failures**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
```

Expected: payload tests fail because raw JSON columns are not decoded, redacted, or truncated yet.

- [ ] **Step 3: Implement exact payload handling**

Add constants and helpers above `InternalJobRow`:

```rust
const MAX_PAYLOAD_JSON_BYTES: usize = 64 * 1024;
const TRUNCATED_PREVIEW_CHARS: usize = 2000;
const DECODE_FAILURE_PREVIEW_CHARS: usize = 500;
const REDACTED: &str = "[redacted]";
const SENSITIVE_KEY_FRAGMENTS: &[&str] = &[
    "apikey",
    "authorization",
    "bearer",
    "cookie",
    "credentials",
    "password",
    "secret",
    "session",
    "token",
    "apihash",
    "refreshtoken",
];

#[derive(Clone, Debug, PartialEq)]
struct PayloadView {
    json: Option<Value>,
    preview: Option<String>,
    truncated: bool,
}

fn payload_view(raw: Option<Vec<u8>>, decode_failure_preview: bool) -> PayloadView {
    let Some(raw) = raw else {
        return PayloadView {
            json: None,
            preview: None,
            truncated: false,
        };
    };

    match serde_json::from_slice::<Value>(&raw) {
        Ok(value) => bounded_json_payload(redact_json(value)),
        Err(_) => {
            let text = String::from_utf8_lossy(&raw);
            PayloadView {
                json: None,
                preview: if decode_failure_preview {
                    Some(redacted_text_preview(&text, DECODE_FAILURE_PREVIEW_CHARS))
                } else {
                    None
                },
                truncated: false,
            }
        }
    }
}

fn bounded_json_payload(value: Value) -> PayloadView {
    let serialized = serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string());
    if serialized.len() <= MAX_PAYLOAD_JSON_BYTES {
        return PayloadView {
            json: Some(value),
            preview: None,
            truncated: false,
        };
    }

    PayloadView {
        json: Some(serde_json::json!({
            "truncated": true,
            "preview": redacted_text_preview(&serialized, TRUNCATED_PREVIEW_CHARS),
        })),
        preview: None,
        truncated: true,
    }
}

fn redact_json(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    if is_sensitive_key(&key) {
                        (key, Value::String(REDACTED.to_string()))
                    } else {
                        (key, redact_json(value))
                    }
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(redact_json).collect()),
        other => other,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    SENSITIVE_KEY_FRAGMENTS
        .iter()
        .any(|fragment| normalized.contains(fragment))
}

fn redacted_text_preview(text: &str, max_chars: usize) -> String {
    let redacted = redact_text_fragments(text);
    redacted.chars().take(max_chars).collect()
}

fn redact_text_fragments(text: &str) -> String {
    text.split(';')
        .filter_map(|part| {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                return None;
            }
            let mut pieces = trimmed.splitn(2, ['=', ':']);
            let key = pieces.next().unwrap_or_default().trim();
            let value = pieces.next();
            Some(if value.is_some() && is_sensitive_key(key) {
                format!("{key}={REDACTED}")
            } else {
                trimmed.to_string()
            })
        })
        .collect::<Vec<_>>()
        .join("; ")
}
```

Then replace `InternalJobRow::into_dto` with:

```rust
impl InternalJobRow {
    fn into_dto(self) -> ApalisJobRow {
        let job = payload_view(self.job_raw, true);
        let last_result = payload_view(self.last_result_raw, false);
        let metadata = payload_view(self.metadata_raw, false);

        ApalisJobRow {
            id: self.id,
            job_type: self.job_type,
            status: self.status,
            attempts: self.attempts,
            max_attempts: self.max_attempts,
            run_at: self.run_at,
            lock_at: self.lock_at,
            lock_by: self.lock_by,
            done_at: self.done_at,
            last_activity_at: self.last_activity_at,
            priority: self.priority,
            idempotency_key: self.idempotency_key,
            job_preview: job.preview,
            job_truncated: job.truncated,
            job_json: job.json,
            last_result: last_result.json,
            last_result_truncated: last_result.truncated,
            metadata: metadata.json,
            metadata_truncated: metadata.truncated,
        }
    }
}
```

- [ ] **Step 4: Run backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
```

Expected: all `apalis_jobs` backend tests pass.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/apalis_jobs.rs
git commit -m "feat: sanitize apalis job payloads"
```

---

## Task 4: Frontend Types And Tauri API Wrapper

**Files:**
- Create: `src/lib/types/apalis-jobs.ts`
- Create: `src/lib/api/apalis-jobs.ts`
- Create: `src/lib/api/apalis-jobs.test.ts`

- [ ] **Step 1: Write failing API wrapper tests**

Create `src/lib/api/apalis-jobs.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import apalisJobsApiSource from "./apalis-jobs.ts?raw";
import { loadApalisJobs } from "./apalis-jobs";
import type { ApalisJobsListResponse } from "$lib/types/apalis-jobs";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

function responseFixture(): ApalisJobsListResponse {
  return {
    jobs: [
      {
        id: "job-1",
        jobType: "gemini-browser",
        status: "Pending",
        attempts: 0,
        maxAttempts: 1,
        runAt: "2026-06-23T10:00:00Z",
        lockAt: null,
        lockBy: null,
        doneAt: null,
        lastActivityAt: "2026-06-23T10:00:00Z",
        priority: 0,
        idempotencyKey: "run-1",
        jobPreview: null,
        jobTruncated: false,
        jobJson: { run_id: "run-1" },
        lastResult: null,
        lastResultTruncated: false,
        metadata: null,
        metadataTruncated: false,
      },
    ],
    totalMatching: 1,
    statusCounts: [{ status: "Pending", count: 1 }],
    jobTypeCounts: [{ jobType: "gemini-browser", count: 1 }],
    refreshedAt: "2026-06-23T10:00:01Z",
    limit: 100,
  };
}

describe("apalis jobs api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads Apalis jobs through the dedicated Tauri command", async () => {
    const fixture = responseFixture();
    invokeMock.mockResolvedValueOnce(fixture);

    await expect(
      loadApalisJobs({
        limit: 50,
        status: "Pending",
        jobType: "gemini-browser",
        search: "run",
      }),
    ).resolves.toBe(fixture);

    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith("apalis_jobs_list", {
      request: {
        limit: 50,
        status: "Pending",
        jobType: "gemini-browser",
        search: "run",
      },
    });
  });

  it("keeps the wrapper narrow and free of logging or client-side mapping", () => {
    expect(apalisJobsApiSource).not.toContain("console.error");
    expect(apalisJobsApiSource).not.toContain("JSON.stringify");
    expect(apalisJobsApiSource).not.toContain(".then(");
    expect(apalisJobsApiSource).not.toContain("filter(");
  });
});
```

- [ ] **Step 2: Run API tests to verify failure**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/api/apalis-jobs.test.ts
```

Expected: fails because the new files do not exist.

- [ ] **Step 3: Create stable frontend types**

Create `src/lib/types/apalis-jobs.ts`:

```ts
export type ApalisJsonValue =
  | null
  | boolean
  | number
  | string
  | ApalisJsonValue[]
  | { [key: string]: ApalisJsonValue };

export interface ApalisJobsListRequest {
  limit?: number | null;
  status?: string | null;
  jobType?: string | null;
  search?: string | null;
}

export interface ApalisJobsListResponse {
  jobs: ApalisJobRow[];
  totalMatching: number;
  statusCounts: ApalisJobStatusCount[];
  jobTypeCounts: ApalisJobTypeCount[];
  refreshedAt: string;
  limit: number;
}

export interface ApalisJobRow {
  id: string;
  jobType: string;
  status: string;
  attempts: number;
  maxAttempts: number | null;
  runAt: string | null;
  lockAt: string | null;
  lockBy: string | null;
  doneAt: string | null;
  lastActivityAt: string | null;
  priority: number | null;
  idempotencyKey: string | null;
  jobPreview: string | null;
  jobTruncated: boolean;
  jobJson: ApalisJsonValue | null;
  lastResult: ApalisJsonValue | null;
  lastResultTruncated: boolean;
  metadata: ApalisJsonValue | null;
  metadataTruncated: boolean;
}

export interface ApalisJobStatusCount {
  status: string;
  count: number;
}

export interface ApalisJobTypeCount {
  jobType: string;
  count: number;
}
```

- [ ] **Step 4: Create the API wrapper**

Create `src/lib/api/apalis-jobs.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { ApalisJobsListRequest, ApalisJobsListResponse } from "$lib/types/apalis-jobs";

export function loadApalisJobs(request: ApalisJobsListRequest = {}) {
  return invoke<ApalisJobsListResponse>("apalis_jobs_list", { request });
}
```

- [ ] **Step 5: Run API tests**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/api/apalis-jobs.test.ts
```

Expected: `apalis jobs api wrapper` tests pass.

- [ ] **Step 6: Commit**

```powershell
git add src/lib/types/apalis-jobs.ts src/lib/api/apalis-jobs.ts src/lib/api/apalis-jobs.test.ts
git commit -m "feat: add apalis jobs frontend api"
```

---

## Task 5: Jobs Split Inspector Route And Navigation

**Files:**
- Create: `src/lib/apalis-jobs-route-contract.test.ts`
- Create: `src/lib/components/jobs/ApalisJobsPanel.svelte`
- Create: `src/routes/jobs/+page.svelte`
- Modify: `src/routes/+layout.svelte`

- [ ] **Step 1: Write failing route and navigation contract tests**

Create `src/lib/apalis-jobs-route-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import layoutSource from "../routes/+layout.svelte?raw";
import jobsPageSource from "../routes/jobs/+page.svelte?raw";
import jobsPanelSource from "./components/jobs/ApalisJobsPanel.svelte?raw";

describe("apalis jobs inspector frontend source contracts", () => {
  it("adds Jobs as a separate top-level navigation item in both modes", () => {
    expect(layoutSource).toContain("ListChecks");
    expect(layoutSource.match(/label: "Jobs"/g)?.length).toBe(2);
    expect(layoutSource.match(/caption: "Apalis queue"/g)?.length).toBe(2);
    expect(layoutSource.match(/pathname.startsWith\("\/jobs"\)/g)?.length).toBe(3);
    expect(layoutSource).toContain("Jobs");
  });

  it("keeps Tauri invocation inside the Apalis jobs API wrapper", () => {
    expect(jobsPageSource).toContain("ApalisJobsPanel");
    expect(jobsPanelSource).toContain('import { loadApalisJobs } from "$lib/api/apalis-jobs";');
    expect(jobsPageSource).not.toContain("invoke(");
    expect(jobsPanelSource).not.toContain("invoke(");
  });

  it("implements manual refresh without auto polling or mutations", () => {
    expect(jobsPanelSource).toMatch(/onMount\s*\(\s*\(\)\s*=>/);
    expect(jobsPanelSource).toContain("refreshJobs(true)");
    expect(jobsPanelSource).toContain("refreshJobs(false)");
    expect(jobsPanelSource).not.toContain("setInterval");
    expect(jobsPanelSource).not.toContain("retry");
    expect(jobsPanelSource).not.toContain("cancel");
    expect(jobsPanelSource).not.toContain("kill");
    expect(jobsPanelSource).not.toContain("delete");
    expect(jobsPanelSource).not.toContain("copy");
  });

  it("reloads through the backend when filters change", () => {
    expect(jobsPanelSource).toContain("function handleFilterChange");
    expect(jobsPanelSource).toContain("void refreshJobs(false)");
    expect(jobsPanelSource).not.toContain(".filter((job");
    expect(jobsPanelSource).not.toContain(".filter(job");
  });

  it("renders split inspector pieces and safe payload labels", () => {
    for (const token of [
      "Status",
      "Job type",
      "Search",
      "Limit",
      "Refresh",
      "Job payload",
      "Last result",
      "Metadata",
      "truncated",
      "redacted",
      "No Apalis jobs match these filters.",
      "Select a job",
    ]) {
      expect(jobsPanelSource).toContain(token);
    }
  });
});
```

- [ ] **Step 2: Run route contract test to verify failure**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/apalis-jobs-route-contract.test.ts
```

Expected: fails because the route, panel, and navigation do not exist.

- [ ] **Step 3: Create the route**

Create `src/routes/jobs/+page.svelte`:

```svelte
<script lang="ts">
  import ApalisJobsPanel from "$lib/components/jobs/ApalisJobsPanel.svelte";
</script>

<ApalisJobsPanel />
```

- [ ] **Step 4: Create the split inspector component**

Create `src/lib/components/jobs/ApalisJobsPanel.svelte`:

```svelte
<script lang="ts">
  import { RefreshCw } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { loadApalisJobs } from "$lib/api/apalis-jobs";
  import { formatDataGridDateTimeValue } from "$lib/components/extractum-ui/data-grid-date-format";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";
  import type {
    ApalisJobRow,
    ApalisJobsListRequest,
    ApalisJobsListResponse,
    ApalisJsonValue,
  } from "$lib/types/apalis-jobs";

  const statusOptions = ["", "Pending", "Queued", "Running", "Done", "Failed", "Killed"];
  const limitOptions = [50, 100, 200, 500];

  let response = $state<ApalisJobsListResponse | null>(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let error = $state<string | null>(null);
  let statusFilter = $state("");
  let jobTypeFilter = $state("");
  let search = $state("");
  let limit = $state(100);
  let selectedJobId = $state<string | null>(null);

  let selectedJob = $derived(
    response?.jobs.find((job) => job.id === selectedJobId) ?? response?.jobs[0] ?? null,
  );

  onMount(() => {
    void refreshJobs(true);
  });

  function request(): ApalisJobsListRequest {
    return {
      limit,
      status: statusFilter || null,
      jobType: jobTypeFilter || null,
      search: search.trim() || null,
    };
  }

  async function refreshJobs(initial: boolean) {
    if (initial) {
      loading = true;
    } else {
      refreshing = true;
    }
    error = null;

    try {
      const next = await loadApalisJobs(request());
      response = next;
      if (selectedJobId && !next.jobs.some((job) => job.id === selectedJobId)) {
        selectedJobId = next.jobs[0]?.id ?? null;
      } else if (!selectedJobId) {
        selectedJobId = next.jobs[0]?.id ?? null;
      }
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
      if (initial) response = null;
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  function handleFilterChange() {
    void refreshJobs(false);
  }

  function formatTime(value: string | null) {
    return String(formatDataGridDateTimeValue(value, "datetime", "en-US", "UTC") ?? "Never");
  }

  function statusTone(status: string) {
    if (status === "Done") return "success";
    if (status === "Failed" || status === "Killed") return "error";
    if (status === "Running") return "info";
    return "default";
  }

  function countForStatus(status: string) {
    return response?.statusCounts.find((row) => row.status === status)?.count ?? 0;
  }

  function jsonPreview(value: ApalisJsonValue | null, fallback: string | null) {
    if (value !== null) return JSON.stringify(value, null, 2);
    return fallback ?? "No data";
  }
</script>

<section class="page-shell jobs-page">
  <div class="page-hero jobs-hero">
    <div>
      <p class="eyebrow">Apalis queue</p>
      <h1>Jobs</h1>
      <p>Read-only inspector for local Apalis jobs.</p>
    </div>
    <Button variant="secondary" onclick={() => refreshJobs(false)} disabled={loading || refreshing}>
      <RefreshCw size={15} aria-hidden="true" />
      Refresh
    </Button>
  </div>

  <div class="jobs-layout">
    <SurfaceCard className="jobs-list-panel">
      <div class="jobs-toolbar">
        <label>
          <span>Status</span>
          <select bind:value={statusFilter} onchange={handleFilterChange}>
            {#each statusOptions as status}
              <option value={status}>{status || "All statuses"}</option>
            {/each}
          </select>
        </label>
        <label>
          <span>Job type</span>
          <select bind:value={jobTypeFilter} onchange={handleFilterChange}>
            <option value="">All job types</option>
            {#each response?.jobTypeCounts ?? [] as row}
              <option value={row.jobType}>{row.jobType} ({row.count})</option>
            {/each}
          </select>
        </label>
        <label>
          <span>Search</span>
          <input bind:value={search} oninput={handleFilterChange} placeholder="id or idempotency key" />
        </label>
        <label>
          <span>Limit</span>
          <select bind:value={limit} onchange={handleFilterChange}>
            {#each limitOptions as option}
              <option value={option}>{option}</option>
            {/each}
          </select>
        </label>
      </div>

      {#if error}
        <StatusMessage tone="error">{error}</StatusMessage>
      {/if}

      {#if loading}
        <StatusMessage tone="info">Loading Apalis jobs...</StatusMessage>
      {:else if !response || response.jobs.length === 0}
        <StatusMessage tone="muted">No Apalis jobs match these filters.</StatusMessage>
      {:else}
        <div class="jobs-summary" aria-label="Apalis job status counts">
          {#each response.statusCounts as row}
            <Badge variant={statusTone(row.status)}>{row.status} {countForStatus(row.status)}</Badge>
          {/each}
          <span>{response.totalMatching} matching</span>
          <span>Refreshed {formatTime(response.refreshedAt)}</span>
        </div>

        <div class="jobs-table" role="table" aria-label="Apalis jobs">
          <div class="jobs-row jobs-head" role="row">
            <span role="columnheader">Status</span>
            <span role="columnheader">Job type</span>
            <span role="columnheader">Key</span>
            <span role="columnheader">Attempts</span>
            <span role="columnheader">Activity</span>
          </div>
          {#each response.jobs as job (job.id)}
            <button
              class="jobs-row"
              class:selected={selectedJob?.id === job.id}
              role="row"
              type="button"
              onclick={() => (selectedJobId = job.id)}
            >
              <span role="cell"><Badge variant={statusTone(job.status)}>{job.status}</Badge></span>
              <span role="cell">{job.jobType || "unknown"}</span>
              <span role="cell">{job.idempotencyKey ?? job.id}</span>
              <span role="cell">{job.attempts}/{job.maxAttempts ?? "-"}</span>
              <span role="cell">{formatTime(job.lastActivityAt)}</span>
            </button>
          {/each}
        </div>
      {/if}
    </SurfaceCard>

    <SurfaceCard className="jobs-detail-panel">
      {#if selectedJob}
        <div class="detail-header">
          <div>
            <p class="eyebrow">Select a job</p>
            <h2>{selectedJob.idempotencyKey ?? selectedJob.id}</h2>
          </div>
          <Badge variant={statusTone(selectedJob.status)}>{selectedJob.status}</Badge>
        </div>

        <dl class="job-fields">
          <div><dt>ID</dt><dd>{selectedJob.id}</dd></div>
          <div><dt>Job type</dt><dd>{selectedJob.jobType || "unknown"}</dd></div>
          <div><dt>Run at</dt><dd>{formatTime(selectedJob.runAt)}</dd></div>
          <div><dt>Lock at</dt><dd>{formatTime(selectedJob.lockAt)}</dd></div>
          <div><dt>Done at</dt><dd>{formatTime(selectedJob.doneAt)}</dd></div>
          <div><dt>Lock by</dt><dd>{selectedJob.lockBy ?? "-"}</dd></div>
        </dl>

        {@render payloadSection("Job payload", selectedJob.jobJson, selectedJob.jobPreview, selectedJob.jobTruncated)}
        {@render payloadSection("Last result", selectedJob.lastResult, null, selectedJob.lastResultTruncated)}
        {@render payloadSection("Metadata", selectedJob.metadata, null, selectedJob.metadataTruncated)}
      {:else}
        <StatusMessage tone="muted">Select a job to inspect its redacted payload.</StatusMessage>
      {/if}
    </SurfaceCard>
  </div>
</section>

{#snippet payloadSection(title: string, value: ApalisJsonValue | null, preview: string | null, truncated: boolean)}
  <section class="payload-section">
    <div class="payload-title">
      <h3>{title}</h3>
      {#if truncated}
        <Badge variant="warning">truncated</Badge>
      {/if}
      <span>redacted</span>
    </div>
    <pre>{jsonPreview(value, preview)}</pre>
  </section>
{/snippet}

<style>
  .jobs-page {
    gap: 1rem;
  }

  .jobs-hero {
    align-items: flex-end;
    justify-content: space-between;
  }

  .jobs-layout {
    display: grid;
    grid-template-columns: minmax(0, 1.35fr) minmax(320px, 0.65fr);
    gap: 1rem;
    align-items: start;
  }

  .jobs-toolbar {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .jobs-toolbar label {
    display: grid;
    gap: 0.35rem;
    font-size: 0.78rem;
    color: var(--muted);
  }

  .jobs-toolbar select,
  .jobs-toolbar input {
    min-width: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
    color: var(--text);
    padding: 0.5rem 0.6rem;
  }

  .jobs-summary {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    margin-bottom: 0.75rem;
    color: var(--muted);
    font-size: 0.82rem;
  }

  .jobs-table {
    display: grid;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .jobs-row {
    display: grid;
    grid-template-columns: 120px minmax(140px, 1fr) minmax(150px, 1.2fr) 88px 150px;
    gap: 0.75rem;
    align-items: center;
    width: 100%;
    border: 0;
    border-bottom: 1px solid var(--border);
    background: transparent;
    color: inherit;
    padding: 0.65rem 0.75rem;
    text-align: left;
    font: inherit;
  }

  .jobs-row:last-child {
    border-bottom: 0;
  }

  .jobs-row:not(.jobs-head):hover,
  .jobs-row.selected {
    background: color-mix(in srgb, var(--primary) 9%, transparent);
  }

  .jobs-head {
    background: var(--panel-strong);
    color: var(--muted);
    font-size: 0.78rem;
    font-weight: 700;
  }

  .jobs-detail-panel {
    position: sticky;
    top: 1rem;
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .detail-header h2 {
    margin: 0;
    font-size: 1rem;
    word-break: break-word;
  }

  .job-fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.75rem;
    margin: 0 0 1rem;
  }

  .job-fields div {
    min-width: 0;
  }

  .job-fields dt {
    color: var(--muted);
    font-size: 0.75rem;
  }

  .job-fields dd {
    margin: 0.15rem 0 0;
    word-break: break-word;
  }

  .payload-section {
    display: grid;
    gap: 0.45rem;
    margin-top: 1rem;
  }

  .payload-title {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .payload-title h3 {
    margin: 0;
    font-size: 0.9rem;
  }

  .payload-title span {
    color: var(--muted);
    font-size: 0.78rem;
  }

  pre {
    max-height: 260px;
    overflow: auto;
    margin: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel-strong);
    padding: 0.75rem;
    white-space: pre-wrap;
    word-break: break-word;
    font-size: 0.78rem;
  }

  @media (max-width: 980px) {
    .jobs-layout {
      grid-template-columns: 1fr;
    }

    .jobs-detail-panel {
      position: static;
    }

    .jobs-toolbar {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .jobs-row {
      grid-template-columns: 100px minmax(110px, 1fr) minmax(130px, 1fr);
    }

    .jobs-row span:nth-child(4),
    .jobs-row span:nth-child(5) {
      display: none;
    }
  }

  @media (max-width: 640px) {
    .jobs-toolbar,
    .job-fields {
      grid-template-columns: 1fr;
    }

    .jobs-row {
      grid-template-columns: 92px minmax(0, 1fr);
    }

    .jobs-row span:nth-child(3) {
      display: none;
    }
  }
</style>
```

- [ ] **Step 5: Wire navigation and topbar route label**

In `src/routes/+layout.svelte`, add `ListChecks` to the lucide import:

```ts
import { Activity, FolderKanban, LayoutDashboard, Library, ListChecks, Menu, Moon, Settings, ShieldCheck, Sun, UserRound } from "@lucide/svelte";
```

Add this object to `legacyNavItems` after `Accounts`:

```ts
    {
      href: "/jobs",
      label: "Jobs",
      caption: "Apalis queue",
      icon: ListChecks,
      active: (pathname: string) => pathname.startsWith("/jobs"),
    },
```

Add the same object to `projectsNavItems` after `Runs`:

```ts
    {
      href: "/jobs",
      label: "Jobs",
      caption: "Apalis queue",
      icon: ListChecks,
      active: (pathname: string) => pathname.startsWith("/jobs"),
    },
```

Add a topbar branch before Diagnostics:

```svelte
              {:else if page.url.pathname.startsWith("/jobs")}
                Jobs
```

- [ ] **Step 6: Run route contract tests**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/apalis-jobs-route-contract.test.ts src/lib/api/apalis-jobs.test.ts
```

Expected: all Apalis jobs frontend tests pass.

- [ ] **Step 7: Commit**

```powershell
git add src/lib/apalis-jobs-route-contract.test.ts src/lib/components/jobs/ApalisJobsPanel.svelte src/routes/jobs/+page.svelte src/routes/+layout.svelte
git commit -m "feat: add apalis jobs inspector route"
```

---

## Task 6: Full Verification And Manual Dev Server Check

**Files:**
- Verify all files touched by prior tasks.

- [ ] **Step 1: Run focused backend verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
```

Expected: all `apalis_jobs` tests pass.

- [ ] **Step 2: Run focused frontend verification**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/api/apalis-jobs.test.ts src/lib/apalis-jobs-route-contract.test.ts
```

Expected: all Apalis jobs frontend tests pass.

- [ ] **Step 3: Run Svelte type checking**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check` exits with code 0.

- [ ] **Step 4: Start dev server for browser verification**

Run:

```powershell
npm.cmd exec vite -- --host 127.0.0.1 --port 5184 --strictPort
```

Expected: Vite serves `http://127.0.0.1:5184/`. Keep this process running while checking the browser.

- [ ] **Step 5: Verify `/jobs` in the browser**

Open `http://127.0.0.1:5184/jobs` in the in-app browser or Playwright. Verify:

- Sidebar shows `Jobs` as a separate top-level item.
- The page title is `Jobs`.
- `Refresh`, `Status`, `Job type`, `Search`, and `Limit` controls are visible.
- Empty state says `No Apalis jobs match these filters.` when the local web-only dev server has no Tauri backend.
- There are no mutation controls for retry, cancel, kill, delete, cleanup, export, or copy.

- [ ] **Step 6: Inspect working tree and whitespace**

Run:

```powershell
git diff --check
git status --short
```

Expected: `git diff --check` prints no errors. `git status --short` shows only intentional files if a final commit remains.

- [ ] **Step 7: Commit final verification adjustments if any were needed**

```powershell
git add src-tauri/src/apalis_jobs.rs src-tauri/src/lib.rs src/lib/types/apalis-jobs.ts src/lib/api/apalis-jobs.ts src/lib/api/apalis-jobs.test.ts src/lib/apalis-jobs-route-contract.test.ts src/lib/components/jobs/ApalisJobsPanel.svelte src/routes/jobs/+page.svelte src/routes/+layout.svelte
git commit -m "test: verify apalis jobs inspector"
```

If no files changed after Task 5, skip this commit and record the verification commands in the final handoff.

---

## Self-Review

- Spec coverage: The plan covers the top-level Jobs navigation, split inspector UI, read-only/manual-refresh behavior, all-job read model, server-side filters, total/count semantics, actual local Apalis schema probe, stable DTO shape, correct latest timestamp sorting, RFC3339 UTC normalization, missing-table behavior, redaction, truncation, and dev-server verification.
- Placeholder scan: The plan contains concrete file paths, command lines, DTO shapes, test code, implementation snippets, and expected outcomes for each task.
- Type consistency: Rust DTOs use `#[serde(rename_all = "camelCase")]`; TS types use camelCase keys (`jobType`, `statusCounts`, `lastActivityAt`) and API wrapper sends `{ request }`, matching the Tauri command signature.
