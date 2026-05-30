# Saved Runs Backend Narrowing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the existing `/analysis` Runs companion filters narrow saved-run history in the backend before ordering and limiting.

**Architecture:** Move saved-run list filtering into a single Rust query-builder path owned by `analysis::store`, keep the Tauri command as a thin argument adapter, and pass the existing `runsFilter` state from the route through `analysis-run-workflow`. The frontend keeps local filtering as a final merge guard for active runs, while saved-run reloads gain debounce and stale-response protection.

**Tech Stack:** Tauri commands, Rust, `sqlx::QueryBuilder`, Svelte 5 runes, Vitest raw-source and unit tests, existing `npm.cmd run verify` pipeline.

---

## Reference

- Spec: `docs/superpowers/specs/2026-05-30-saved-runs-backend-narrowing-design.md`
- Current backend command: `src-tauri/src/analysis/mod.rs`
- Current saved-run workflow: `src/lib/analysis-run-workflow.ts`
- Current Runs tab filter state: `src/lib/analysis-run-companion-state.ts`

## File Structure

- `src-tauri/src/analysis/store.rs`
  - Owns the new `AnalysisRunListFilters` struct.
  - Owns the query builder and saved-run summary list tests.
  - Keeps SQL predicate construction away from the command wrapper.
- `src-tauri/src/analysis/mod.rs`
  - Accepts new Tauri command arguments.
  - Clamps limit, rejects mutually exclusive scope inputs, and delegates to store.
- `src/lib/types/analysis.ts`
  - Extends `ListAnalysisRunsInput` with optional backend filter fields.
- `src/lib/api/analysis-runs.test.ts`
  - Locks the invoke payload contract.
- `src/lib/analysis-run-workflow.ts`
  - Adds `runsFilter` to workflow state.
  - Projects `runsFilter` into backend list arguments.
  - Prevents stale saved-run responses from overwriting newer results.
- `src/lib/analysis-run-workflow.test.ts`
  - Covers backend filter projection and stale saved-run request behavior.
- `src/routes/analysis/+page.svelte`
  - Schedules saved-run reloads when `runsFilter` changes.
  - Debounces reloads and clears pending timers during route teardown.
- `src/lib/analysis-route-effects.test.ts`
  - Locks the route effect contract around `historyScopeParams`, `runsFilter`, and scheduled reloads.
- `docs/backlog.md`
  - Removes the shipped historical saved-run narrowing item and leaves remaining cleanup/discoverability follow-up.
- `docs/superpowers/specs/README.md`
  - Removes this active spec after implementation is verified.
- `docs/superpowers/archive/specs/2026-05-30-saved-runs-backend-narrowing-design.md`
  - Archives the shipped spec.
- `docs/superpowers/plans/README.md`
  - Removes this active plan after implementation is verified.

---

### Task 1: Backend Saved-Run Query Builder

**Files:**
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/mod.rs`

- [ ] **Step 1: Add failing backend tests in `src-tauri/src/analysis/store.rs`**

Add the new query tests inside the existing `#[cfg(test)] mod tests` in `src-tauri/src/analysis/store.rs`. Extend the `use super::{ ... }` list with `list_analysis_run_summaries` and `AnalysisRunListFilters`, then add this helper and tests near the other store tests:

```rust
    #[derive(Clone)]
    struct RunListFixture {
        id: i64,
        source_id: Option<i64>,
        source_group_id: Option<i64>,
        scope_label_snapshot: &'static str,
        prompt_template_id: Option<i64>,
        provider_profile: &'static str,
        provider: &'static str,
        model: &'static str,
        status: &'static str,
        error: Option<&'static str>,
        created_at: i64,
    }

    impl RunListFixture {
        fn completed(id: i64, created_at: i64, label: &'static str) -> Self {
            Self {
                id,
                source_id: Some(1),
                source_group_id: None,
                scope_label_snapshot: label,
                prompt_template_id: Some(1),
                provider_profile: "default",
                provider: "gemini",
                model: "gemini-2.5-flash",
                status: "completed",
                error: None,
                created_at,
            }
        }
    }

    async fn run_list_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create sources");

        sqlx::query(
            r#"
            CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create groups");

        sqlx::query(
            r#"
            CREATE TABLE analysis_prompt_templates (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                template_kind TEXT NOT NULL,
                body TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                is_builtin BOOLEAN NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create templates");

        sqlx::query(
            r#"
            CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                run_type TEXT NOT NULL DEFAULT 'report',
                scope_type TEXT NOT NULL DEFAULT 'single_source',
                source_id INTEGER,
                source_group_id INTEGER,
                period_from INTEGER NOT NULL DEFAULT 0,
                period_to INTEGER NOT NULL DEFAULT 0,
                output_language TEXT NOT NULL DEFAULT 'English',
                prompt_template_id INTEGER,
                prompt_template_version INTEGER NOT NULL DEFAULT 1,
                provider_profile TEXT NOT NULL DEFAULT 'default',
                provider TEXT NOT NULL DEFAULT 'gemini',
                model TEXT NOT NULL DEFAULT 'gemini-2.5-flash',
                youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
                telegram_history_scope TEXT NOT NULL DEFAULT 'current',
                status TEXT NOT NULL DEFAULT 'completed',
                result_markdown TEXT,
                trace_data_zstd BLOB,
                scope_label_snapshot TEXT,
                snapshot_captured_at TEXT,
                snapshot_error TEXT,
                error TEXT,
                created_at INTEGER NOT NULL,
                completed_at INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create runs");

        sqlx::query(
            r#"
            CREATE TABLE analysis_run_messages (
                run_id INTEGER NOT NULL,
                ref TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create run messages");

        sqlx::query("INSERT INTO sources (id, title) VALUES (1, 'Alpha Source'), (2, 'Beta Source')")
            .execute(&pool)
            .await
            .expect("insert sources");
        sqlx::query("INSERT INTO analysis_source_groups (id, name) VALUES (10, 'Research Group')")
            .execute(&pool)
            .await
            .expect("insert group");
        sqlx::query(
            "INSERT INTO analysis_prompt_templates (id, name, template_kind, body, version, is_builtin, created_at, updated_at) VALUES (1, 'Weekly Digest', 'report', 'body', 1, 0, 1, 1), (2, 'Incident Review', 'report', 'body', 1, 0, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert templates");

        pool
    }

    async fn insert_run_list_fixture(pool: &sqlx::SqlitePool, fixture: RunListFixture) {
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id,
                run_type,
                scope_type,
                source_id,
                source_group_id,
                period_from,
                period_to,
                output_language,
                prompt_template_id,
                prompt_template_version,
                provider_profile,
                provider,
                model,
                youtube_corpus_mode,
                telegram_history_scope,
                status,
                result_markdown,
                trace_data_zstd,
                scope_label_snapshot,
                snapshot_captured_at,
                snapshot_error,
                error,
                created_at,
                completed_at
            )
            VALUES (?, 'report', ?, ?, ?, 0, 0, 'English', ?, 1, ?, ?, ?, 'transcript_description', 'current', ?, 'Report', NULL, ?, NULL, NULL, ?, ?, ?)
            "#,
        )
        .bind(fixture.id)
        .bind(if fixture.source_group_id.is_some() { "source_group" } else { "single_source" })
        .bind(fixture.source_id)
        .bind(fixture.source_group_id)
        .bind(fixture.prompt_template_id)
        .bind(fixture.provider_profile)
        .bind(fixture.provider)
        .bind(fixture.model)
        .bind(fixture.status)
        .bind(fixture.scope_label_snapshot)
        .bind(fixture.error)
        .bind(fixture.created_at)
        .bind(if fixture.status == "completed" { Some(fixture.created_at + 10) } else { None })
        .execute(pool)
        .await
        .expect("insert run fixture");
    }

    #[tokio::test]
    async fn list_analysis_run_summaries_applies_query_before_limit() {
        let pool = run_list_pool().await;
        insert_run_list_fixture(&pool, RunListFixture::completed(1, 300, "Newest irrelevant")).await;
        insert_run_list_fixture(&pool, RunListFixture::completed(2, 200, "Older target nebula")).await;
        insert_run_list_fixture(&pool, RunListFixture::completed(3, 100, "Oldest target nebula")).await;

        let runs = list_analysis_run_summaries(
            &pool,
            AnalysisRunListFilters {
                source_id: None,
                source_group_id: None,
                limit: 1,
                query: Some("nebula".to_string()),
                ..AnalysisRunListFilters::default()
            },
        )
        .await
        .expect("list runs");

        assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![2]);
    }

    #[tokio::test]
    async fn list_analysis_run_summaries_combines_scope_and_field_filters() {
        let pool = run_list_pool().await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 1,
                source_id: Some(1),
                provider: "gemini",
                model: "gemini-2.5-pro",
                created_at: 300,
                ..RunListFixture::completed(1, 300, "Source match")
            },
        )
        .await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 2,
                source_id: Some(2),
                provider: "openai",
                model: "gpt-5",
                created_at: 200,
                ..RunListFixture::completed(2, 200, "Other source")
            },
        )
        .await;

        let runs = list_analysis_run_summaries(
            &pool,
            AnalysisRunListFilters {
                source_id: Some(1),
                source_group_id: None,
                limit: 50,
                provider: Some("GEM".to_string()),
                model: Some("pro".to_string()),
                ..AnalysisRunListFilters::default()
            },
        )
        .await
        .expect("list runs");

        assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
    }

    #[tokio::test]
    async fn list_analysis_run_summaries_filters_source_groups_and_template_names() {
        let pool = run_list_pool().await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 1,
                source_id: None,
                source_group_id: Some(10),
                scope_label_snapshot: "Research Group",
                prompt_template_id: Some(2),
                created_at: 300,
                ..RunListFixture::completed(1, 300, "Research Group")
            },
        )
        .await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 2,
                source_id: Some(1),
                source_group_id: None,
                prompt_template_id: Some(1),
                created_at: 200,
                ..RunListFixture::completed(2, 200, "Single source")
            },
        )
        .await;

        let runs = list_analysis_run_summaries(
            &pool,
            AnalysisRunListFilters {
                source_id: None,
                source_group_id: Some(10),
                limit: 50,
                template: Some("incident".to_string()),
                ..AnalysisRunListFilters::default()
            },
        )
        .await
        .expect("list runs");

        assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
        assert_eq!(runs[0].source_group_name.as_deref(), Some("Research Group"));
    }

    #[tokio::test]
    async fn list_analysis_run_summaries_rejects_both_scope_ids() {
        let pool = run_list_pool().await;

        let error = list_analysis_run_summaries(
            &pool,
            AnalysisRunListFilters {
                source_id: Some(1),
                source_group_id: Some(10),
                limit: 50,
                ..AnalysisRunListFilters::default()
            },
        )
        .await
        .expect_err("both scope ids should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
    }

    #[tokio::test]
    async fn list_analysis_run_summaries_filters_status_and_dates() {
        let pool = run_list_pool().await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 1,
                status: "completed",
                created_at: 1_704_153_600,
                ..RunListFixture::completed(1, 1_704_153_600, "Jan 2")
            },
        )
        .await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 2,
                status: "failed",
                created_at: 1_704_240_000,
                ..RunListFixture::completed(2, 1_704_240_000, "Jan 3")
            },
        )
        .await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 3,
                status: "running",
                created_at: 1_704_326_400,
                ..RunListFixture::completed(3, 1_704_326_400, "Jan 4")
            },
        )
        .await;

        let completed = list_analysis_run_summaries(
            &pool,
            AnalysisRunListFilters {
                limit: 50,
                status: Some("completed".to_string()),
                date_from: Some("2024-01-02".to_string()),
                date_to: Some("2024-01-02".to_string()),
                ..AnalysisRunListFilters::default()
            },
        )
        .await
        .expect("list completed");
        assert_eq!(completed.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);

        let active = list_analysis_run_summaries(
            &pool,
            AnalysisRunListFilters {
                limit: 50,
                status: Some("queued_running".to_string()),
                date_from: Some("invalid".to_string()),
                date_to: Some("20240104".to_string()),
                ..AnalysisRunListFilters::default()
            },
        )
        .await
        .expect("list active");
        assert_eq!(active.iter().map(|run| run.id).collect::<Vec<_>>(), vec![3]);
    }

    #[tokio::test]
    async fn list_analysis_run_summaries_escapes_literal_like_characters() {
        let pool = run_list_pool().await;
        insert_run_list_fixture(&pool, RunListFixture::completed(1, 300, "100%_literal")).await;
        insert_run_list_fixture(&pool, RunListFixture::completed(2, 200, "100 percent literal")).await;

        let runs = list_analysis_run_summaries(
            &pool,
            AnalysisRunListFilters {
                limit: 50,
                query: Some("100%_literal".to_string()),
                ..AnalysisRunListFilters::default()
            },
        )
        .await
        .expect("list literal percent underscore");

        assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
    }

    #[tokio::test]
    async fn list_analysis_run_summaries_matches_all_query_terms_across_any_field() {
        let pool = run_list_pool().await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 1,
                source_id: Some(1),
                source_group_id: None,
                provider_profile: "research-profile",
                error: Some("quota exhausted"),
                created_at: 300,
                ..RunListFixture::completed(1, 300, "Plain label")
            },
        )
        .await;
        insert_run_list_fixture(
            &pool,
            RunListFixture {
                id: 2,
                source_id: Some(2),
                provider_profile: "research-profile",
                error: Some("different failure"),
                created_at: 200,
                ..RunListFixture::completed(2, 200, "Plain label")
            },
        )
        .await;

        let runs = list_analysis_run_summaries(
            &pool,
            AnalysisRunListFilters {
                limit: 50,
                query: Some("alpha quota".to_string()),
                ..AnalysisRunListFilters::default()
            },
        )
        .await
        .expect("list terms");

        assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
    }
```

- [ ] **Step 2: Run backend tests to verify they fail**

Run:

```powershell
cargo test -p extractum-tauri analysis::store::tests::list_analysis_run_summaries -- --nocapture
```

Expected: FAIL because `AnalysisRunListFilters` and `list_analysis_run_summaries` do not exist.

- [ ] **Step 3: Add the query builder implementation in `src-tauri/src/analysis/store.rs`**

Change the top imports:

```rust
use sqlx::{Pool, QueryBuilder, Sqlite};
```

Add this import near the existing crate imports:

```rust
use crate::time::ymd_to_unix_midnight;
```

Add this code near the existing store query functions, before `fetch_prompt_template`:

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct AnalysisRunListFilters {
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) limit: i64,
    pub(crate) query: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) template: Option<String>,
    pub(crate) date_from: Option<String>,
    pub(crate) date_to: Option<String>,
}

const ANALYSIS_RUN_LIST_SELECT: &str = r#"
    SELECT
        runs.id,
        runs.run_type,
        runs.scope_type,
        runs.source_id,
        sources.title AS source_title,
        runs.source_group_id,
        groups.name AS source_group_name,
        runs.period_from,
        runs.period_to,
        runs.output_language,
        runs.prompt_template_id,
        templates.name AS prompt_template_name,
        runs.prompt_template_version,
        runs.provider_profile,
        runs.provider,
        runs.model,
        runs.youtube_corpus_mode,
        COALESCE(runs.telegram_history_scope, 'current') AS telegram_history_scope,
        runs.status,
        runs.result_markdown,
        runs.trace_data_zstd,
        runs.scope_label_snapshot,
        runs.snapshot_captured_at,
        runs.snapshot_error,
        COALESCE(snapshot_counts.snapshot_message_count, 0) AS snapshot_message_count,
        runs.error,
        runs.created_at,
        runs.completed_at
    FROM analysis_runs runs
    LEFT JOIN sources ON sources.id = runs.source_id
    LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
    LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
    LEFT JOIN (
        SELECT run_id, COUNT(*) AS snapshot_message_count
        FROM analysis_run_messages
        GROUP BY run_id
    ) snapshot_counts ON snapshot_counts.run_id = runs.id
    WHERE 1 = 1
"#;

const RUN_QUERY_FIELDS: [&str; 8] = [
    "lower(coalesce(runs.scope_label_snapshot, ''))",
    "lower(coalesce(sources.title, ''))",
    "lower(coalesce(groups.name, ''))",
    "lower(coalesce(templates.name, ''))",
    "lower(coalesce(runs.provider_profile, ''))",
    "lower(coalesce(runs.provider, ''))",
    "lower(coalesce(runs.model, ''))",
    "lower(coalesce(runs.error, ''))",
];

fn trimmed_filter(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn escaped_like_contains(value: &str) -> String {
    format!(
        "%{}%",
        value
            .trim()
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

fn parse_yyyy_mm_dd_midnight(value: &str) -> Option<i64> {
    let value = value.trim();
    let bytes = value.as_bytes();
    if bytes.len() != 10 ||
        bytes[4] != b'-' ||
        bytes[7] != b'-' ||
        !bytes.iter().enumerate().all(|(index, byte)| {
            index == 4 || index == 7 || byte.is_ascii_digit()
        })
    {
        return None;
    }

    ymd_to_unix_midnight(value)
}

fn parse_yyyy_mm_dd_day_end(value: &str) -> Option<i64> {
    parse_yyyy_mm_dd_midnight(value).map(|start| start + 86_399)
}

fn push_like_predicate(query: &mut QueryBuilder<'_, Sqlite>, expression: &str, value: &str) {
    query.push(" AND ");
    query.push(expression);
    query.push(" LIKE ");
    query.push_bind(escaped_like_contains(value));
    query.push(" ESCAPE '\\'");
}

fn push_search_term_predicate(query: &mut QueryBuilder<'_, Sqlite>, term: &str) {
    query.push(" AND (");
    for (index, field) in RUN_QUERY_FIELDS.iter().enumerate() {
        if index > 0 {
            query.push(" OR ");
        }
        query.push(*field);
        query.push(" LIKE ");
        query.push_bind(escaped_like_contains(term));
        query.push(" ESCAPE '\\'");
    }
    query.push(")");
}

pub(crate) async fn list_analysis_run_summaries(
    pool: &Pool<Sqlite>,
    filters: AnalysisRunListFilters,
) -> AppResult<Vec<AnalysisRunSummary>> {
    if filters.source_id.is_some() && filters.source_group_id.is_some() {
        return Err(AppError::validation(
            "Pass either source_id or source_group_id, not both",
        ));
    }

    let mut query = QueryBuilder::<Sqlite>::new(ANALYSIS_RUN_LIST_SELECT);

    if let Some(source_id) = filters.source_id {
        query.push(" AND runs.source_id = ");
        query.push_bind(source_id);
    }

    if let Some(source_group_id) = filters.source_group_id {
        query.push(" AND runs.source_group_id = ");
        query.push_bind(source_group_id);
    }

    match trimmed_filter(filters.status).as_deref() {
        Some("queued_running") => {
            query.push(" AND runs.status IN (");
            let mut separated = query.separated(", ");
            separated.push_bind(ANALYSIS_STATUS_QUEUED);
            separated.push_bind(ANALYSIS_STATUS_RUNNING);
            separated.push_unseparated(")");
        }
        Some("all") | None => {}
        Some(status) => {
            query.push(" AND runs.status = ");
            query.push_bind(status.to_string());
        }
    }

    if let Some(date_from) = trimmed_filter(filters.date_from)
        .as_deref()
        .and_then(parse_yyyy_mm_dd_midnight)
    {
        query.push(" AND runs.created_at >= ");
        query.push_bind(date_from);
    }

    if let Some(date_to) = trimmed_filter(filters.date_to)
        .as_deref()
        .and_then(parse_yyyy_mm_dd_day_end)
    {
        query.push(" AND runs.created_at <= ");
        query.push_bind(date_to);
    }

    if let Some(provider) = trimmed_filter(filters.provider) {
        push_like_predicate(&mut query, "lower(coalesce(runs.provider, ''))", &provider);
    }

    if let Some(model) = trimmed_filter(filters.model) {
        push_like_predicate(&mut query, "lower(coalesce(runs.model, ''))", &model);
    }

    if let Some(template) = trimmed_filter(filters.template) {
        push_like_predicate(&mut query, "lower(coalesce(templates.name, ''))", &template);
    }

    if let Some(search) = trimmed_filter(filters.query) {
        for term in search.split_whitespace() {
            push_search_term_predicate(&mut query, term);
        }
    }

    query.push(" ORDER BY runs.created_at DESC LIMIT ");
    query.push_bind(filters.limit.clamp(1, 100));

    let rows = query
        .build_query_as::<AnalysisRunRow>()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    Ok(rows.into_iter().map(map_run_summary).collect())
}
```

- [ ] **Step 4: Replace the duplicated SQL branches in `src-tauri/src/analysis/mod.rs`**

Update the store import list at the top of `src-tauri/src/analysis/mod.rs` so it includes the new store API:

```rust
use self::store::{
    delete_saved_run, ensure_builtin_report_template, fetch_prompt_template,
    list_analysis_run_summaries, mark_run_capture_failed, AnalysisRunInsert,
    AnalysisRunListFilters,
};
```

Replace the current `list_analysis_runs` function with:

```rust
#[tauri::command]
pub async fn list_analysis_runs(
    handle: AppHandle,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    limit: Option<i64>,
    query: Option<String>,
    status: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    template: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> AppResult<Vec<AnalysisRunSummary>> {
    let pool = get_pool(&handle).await?;
    let limit = limit.unwrap_or(20).clamp(1, 100);

    list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            source_id,
            source_group_id,
            limit,
            query,
            status,
            provider,
            model,
            template,
            date_from,
            date_to,
        },
    )
    .await
}
```

This keeps the mutual-exclusion validation in `store.rs`; do not add a second SQL path in `mod.rs`.

- [ ] **Step 5: Run backend tests**

Run:

```powershell
cargo test -p extractum-tauri analysis::store::tests::list_analysis_run_summaries -- --nocapture
cargo test -p extractum-tauri analysis::mod::tests::list_analysis_runs -- --nocapture
```

Expected: PASS. If `analysis::mod::tests::list_analysis_runs` has no matching test names, run:

```powershell
cargo test -p extractum-tauri analysis::mod::tests -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Commit backend work**

```powershell
git add src-tauri/src/analysis/store.rs src-tauri/src/analysis/mod.rs
git commit -m "feat: narrow saved analysis runs in backend"
```

---

### Task 2: TypeScript API Contract

**Files:**
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-runs.test.ts`

- [ ] **Step 1: Extend `ListAnalysisRunsInput` in `src/lib/types/analysis.ts`**

Replace the current interface with:

```ts
export interface ListAnalysisRunsInput {
  sourceId: number | null;
  sourceGroupId: number | null;
  limit: number;
  query?: string;
  status?: "all" | "completed" | "failed" | "cancelled" | "queued_running";
  provider?: string;
  model?: string;
  template?: string;
  dateFrom?: string;
  dateTo?: string;
}
```

- [ ] **Step 2: Update the API wrapper payload test**

In `src/lib/api/analysis-runs.test.ts`, replace the first `listAnalysisRuns` call in `"wraps analysis run list commands with typed arguments"` with:

```ts
    await expect(listAnalysisRuns({
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
      query: "older run",
      status: "completed",
      provider: "gemini",
      model: "flash",
      template: "digest",
      dateFrom: "2026-05-01",
      dateTo: "2026-05-30",
    })).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_runs", {
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
      query: "older run",
      status: "completed",
      provider: "gemini",
      model: "flash",
      template: "digest",
      dateFrom: "2026-05-01",
      dateTo: "2026-05-30",
    });
```

- [ ] **Step 3: Run the focused API test**

Run:

```powershell
npm.cmd run test -- src/lib/api/analysis-runs.test.ts
```

Expected: PASS.

- [ ] **Step 4: Commit API contract work**

```powershell
git add src/lib/types/analysis.ts src/lib/api/analysis-runs.test.ts
git commit -m "feat: expose saved run list filters in API"
```

---

### Task 3: Workflow Filter Projection And Stale Guard

**Files:**
- Modify: `src/lib/analysis-run-workflow.ts`
- Modify: `src/lib/analysis-run-workflow.test.ts`

- [ ] **Step 1: Update workflow tests for `runsFilter` projection**

In `src/lib/analysis-run-workflow.test.ts`, add this import:

```ts
import { runsFilterDefaults, type CompanionRunsFilterState } from "./analysis-run-companion-state";
```

Add `runsFilter` to the harness state type:

```ts
type AnalysisRunWorkflowHarnessState = AnalysisRunWorkflowState & {
  runsFilter: CompanionRunsFilterState;
  runs: AnalysisRunSummary[];
  activeRuns: AnalysisRunSummary[];
  loadingRuns: boolean;
  loadingActiveRuns: boolean;
  loadingRunDetail: boolean;
  inspectorMode: "active" | "history" | "trace" | "chunks";
  startingReport: boolean;
  deletingRunIds: Record<number, boolean>;
  status: string;
};
```

Add the default state value:

```ts
    runsFilter: runsFilterDefaults(),
```

Update the `"loads saved runs for the provided scope without reading workflow state"` test so it calls `loadRunsForScope` with an explicit filter and expects backend filter fields:

```ts
    const filter: CompanionRunsFilterState = {
      ...runsFilterDefaults(),
      query: "older target",
      status: "completed",
      dateFrom: "2026-05-01",
      dateTo: "2026-05-30",
      provider: "gemini",
      model: "flash",
      template: "digest",
    };

    await workflow.loadRunsForScope(params, filter);

    expect(deps.listRuns).toHaveBeenCalledWith({
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
      query: "older target",
      status: "completed",
      dateFrom: "2026-05-01",
      dateTo: "2026-05-30",
      provider: "gemini",
      model: "flash",
      template: "digest",
    });
```

Update null-scope tests to pass a filter without reading workflow state:

```ts
    await workflow.loadRunsForScope(null, runsFilterDefaults());
```

Update wrapper expectations in tests that call `workflow.loadRuns()` so expected list payloads include the default filter projection:

```ts
    expect(deps.listRuns).toHaveBeenCalledWith({
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
      query: "",
      status: "all",
      dateFrom: "",
      dateTo: "",
      provider: "",
      model: "",
      template: "",
    });
```

Add this stale-response test after the saved-run loading failure test:

```ts
  it("ignores stale saved run responses when a newer load finishes first", async () => {
    const params: AnalysisHistoryScopeParams = { sourceId: 7, sourceGroupId: null };
    const { state, deps, workflow } = createHarness({ historyScopeParams: params });
    let resolveFirst: (runs: AnalysisRunSummary[]) => void = () => {};
    let resolveSecond: (runs: AnalysisRunSummary[]) => void = () => {};
    deps.listRuns
      .mockReturnValueOnce(new Promise<AnalysisRunSummary[]>((resolve) => {
        resolveFirst = resolve;
      }))
      .mockReturnValueOnce(new Promise<AnalysisRunSummary[]>((resolve) => {
        resolveSecond = resolve;
      }));

    const first = workflow.loadRunsForScope(params, {
      ...runsFilterDefaults(),
      query: "first",
    });
    const second = workflow.loadRunsForScope(params, {
      ...runsFilterDefaults(),
      query: "second",
    });

    resolveSecond([runSummary({ id: 2, scope_label: "Second" })]);
    await second;
    expect(state.runs.map((run) => run.id)).toEqual([2]);
    expect(state.loadingRuns).toBe(false);

    resolveFirst([runSummary({ id: 1, scope_label: "First" })]);
    await first;
    expect(state.runs.map((run) => run.id)).toEqual([2]);
    expect(state.loadingRuns).toBe(false);
  });
```

- [ ] **Step 2: Run workflow tests to verify failures**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-workflow.test.ts
```

Expected: FAIL because `AnalysisRunWorkflowState` has no `runsFilter`, `loadRunsForScope` accepts one argument, and stale saved-run responses are not guarded.

- [ ] **Step 3: Implement workflow projection and stale guard**

In `src/lib/analysis-run-workflow.ts`, add this import:

```ts
import type { CompanionRunsFilterState } from "$lib/analysis-run-companion-state";
```

Add `runsFilter` to `AnalysisRunWorkflowState`:

```ts
export interface AnalysisRunWorkflowState {
  historyScopeParams: AnalysisHistoryScopeParams | null;
  runsFilter: CompanionRunsFilterState;
  activeRunId: number | null;
  currentRun: AnalysisRunDetail | null;
  activeChatRequestId: string | null;
  activeChatRunId: number | null;
  runs: AnalysisRunSummary[];
  activeRuns: AnalysisRunSummary[];
  deletingRunIds: Record<number, boolean>;
}
```

Add a projection helper above `createAnalysisRunWorkflow`:

```ts
export function analysisRunsBackendFilters(filter: CompanionRunsFilterState): Pick<
  ListAnalysisRunsInput,
  "query" | "status" | "dateFrom" | "dateTo" | "provider" | "model" | "template"
> {
  return {
    query: filter.query,
    status: filter.status,
    dateFrom: filter.dateFrom,
    dateTo: filter.dateTo,
    provider: filter.provider,
    model: filter.model,
    template: filter.template,
  };
}
```

Add a saved-run load token inside `createAnalysisRunWorkflow`:

```ts
  let openRunRequestToken = 0;
  let savedRunsRequestToken = 0;
```

Replace `loadRunsForScope` and `loadRuns` with:

```ts
  async function loadRunsForScope(
    params: AnalysisHistoryScopeParams | null,
    filter: CompanionRunsFilterState,
  ) {
    const requestToken = ++savedRunsRequestToken;

    if (params === null) {
      deps.patch({ runs: [], loadingRuns: false });
      return;
    }

    deps.patch({ loadingRuns: true });
    try {
      const summaries = await deps.listRuns({
        sourceId: params.sourceId,
        sourceGroupId: params.sourceGroupId,
        limit: 50,
        ...analysisRunsBackendFilters(filter),
      });
      if (requestToken !== savedRunsRequestToken) {
        return;
      }
      deps.patch({ runs: summaries.filter((run) => !isActiveRunStatus(run.status)) });
    } catch (error) {
      if (requestToken !== savedRunsRequestToken) {
        return;
      }
      deps.patch({ status: deps.formatError("loading analysis runs", error) });
    } finally {
      if (requestToken === savedRunsRequestToken) {
        deps.patch({ loadingRuns: false });
      }
    }
  }

  async function loadRuns() {
    const state = deps.getState();
    await loadRunsForScope(state.historyScopeParams, state.runsFilter);
  }
```

- [ ] **Step 4: Run workflow tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit workflow work**

```powershell
git add src/lib/analysis-run-workflow.ts src/lib/analysis-run-workflow.test.ts
git commit -m "feat: load saved runs with companion filters"
```

---

### Task 4: Route Reload Debounce

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/analysis-route-effects.test.ts`

- [ ] **Step 1: Update the route effect contract test**

In `src/lib/analysis-route-effects.test.ts`, replace the `"keeps saved run history loading out of effect dependency tracking"` test with:

```ts
  it("schedules saved run history loading from explicit scope params and runs filters", () => {
    const effect = historyScopeEffect();

    expect(effect, "history-scope effect should read explicit history-scope params").toContain(
      "const params = historyScopeParams;",
    );
    expect(effect, "history-scope effect should read the canonical companion runs filter").toContain(
      "const filter = runsFilter;",
    );
    expect(effect, "history-scope effect should schedule the explicit-scope loader").toContain(
      "scheduleSavedRunsLoad(params, filter);",
    );
    expect(effect, "history-scope effect must not call the broad wrapper directly").not.toContain(
      "loadRuns();",
    );
    expect(effect, "history-scope effect should not need untrack after explicit params").not.toContain(
      "untrack(",
    );
  });
```

Add this test after it:

```ts
  it("debounces saved run reloads and clears pending timers on teardown", () => {
    expect(analysisPageSource).toContain("let savedRunsLoadTimer: ReturnType<typeof setTimeout> | null = null;");
    expect(analysisPageSource).toContain("const savedRunsLoadDelayMs = 250;");
    expect(analysisPageSource).toContain("function scheduleSavedRunsLoad(");
    expect(analysisPageSource).toContain("clearSavedRunsLoadTimer();");
    expect(analysisPageSource).toContain("void runWorkflow.loadRunsForScope(params, filter);");
  });
```

- [ ] **Step 2: Run the route effect test to verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-route-effects.test.ts
```

Expected: FAIL because the route still calls `runWorkflow.loadRunsForScope(params)` directly.

- [ ] **Step 3: Add route debounce helpers in `src/routes/analysis/+page.svelte`**

In the `analysis-scope-state` import, add the type used by the helper signature:

```svelte
    type AnalysisHistoryScopeParams,
```

Near the existing route-level timer state, add:

```svelte
  let savedRunsLoadTimer: ReturnType<typeof setTimeout> | null = null;
  const savedRunsLoadDelayMs = 250;
```

Add these helper functions near the other run-loading functions:

```svelte
  function clearSavedRunsLoadTimer() {
    if (savedRunsLoadTimer) {
      clearTimeout(savedRunsLoadTimer);
      savedRunsLoadTimer = null;
    }
  }

  function scheduleSavedRunsLoad(
    params: AnalysisHistoryScopeParams | null,
    filter: CompanionRunsFilterState,
  ) {
    clearSavedRunsLoadTimer();

    if (params === null) {
      void runWorkflow.loadRunsForScope(null, filter);
      return;
    }

    savedRunsLoadTimer = setTimeout(() => {
      savedRunsLoadTimer = null;
      void runWorkflow.loadRunsForScope(params, filter);
    }, savedRunsLoadDelayMs);
  }
```

Replace the current history-scope load effect with:

```svelte
  $effect(() => {
    const params = historyScopeParams;
    const filter = runsFilter;

    scheduleSavedRunsLoad(params, filter);
  });
```

In the existing route teardown returned from `onMount`, add:

```svelte
      clearSavedRunsLoadTimer();
```

Use the existing teardown function; do not create a second `onMount` solely for this cleanup.

- [ ] **Step 4: Ensure workflow state includes `runsFilter`**

In the `createAnalysisRunWorkflow` deps object in `src/routes/analysis/+page.svelte`, make sure `getState()` returns `runsFilter`. The returned object should include:

```svelte
      runsFilter,
```

- [ ] **Step 5: Run route and workflow tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-route-effects.test.ts src/lib/analysis-run-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit route debounce work**

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-route-effects.test.ts src/lib/analysis-run-workflow.test.ts src/lib/analysis-run-workflow.ts
git commit -m "feat: debounce saved run filter reloads"
```

---

### Task 5: Existing Client Filter Safety And Raw Contracts

**Files:**
- Modify: `src/lib/analysis-run-companion-state.test.ts`
- Inspect: `src/lib/analysis-run-companion-state.ts`
- Inspect: `src/lib/components/analysis/run-companion-runs-tab.svelte`

- [ ] **Step 1: Add a regression test that local filtering remains a final guard**

In `src/lib/analysis-run-companion-state.test.ts`, add this test near existing `filterCompanionRuns` tests:

```ts
  it("keeps local saved-run filtering as a final consistency guard", () => {
    const filter = {
      ...runsFilterDefaults(),
      query: "needle",
      status: "completed",
      provider: "gemini",
      model: "flash",
      template: "digest",
    };
    const entries = filterCompanionRuns(
      [],
      [
        runSummary({
          id: 1,
          scope_label: "Needle report",
          status: "completed",
          provider: "gemini",
          model: "gemini-2.5-flash",
          prompt_template_name: "Daily digest",
        }),
        runSummary({
          id: 2,
          scope_label: "Needle report",
          status: "failed",
          provider: "gemini",
          model: "gemini-2.5-flash",
          prompt_template_name: "Daily digest",
        }),
      ],
      filter,
    );

    expect(entries.map((entry) => entry.run.id)).toEqual([1]);
  });
```

Use the existing `runSummary` helper name from that file. If the file uses a different local helper name, keep the existing helper and only add the assertion above.

- [ ] **Step 2: Run the companion-state tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-companion-state.test.ts
```

Expected: PASS. This test should pass without changing production code because the local guard already exists.

- [ ] **Step 3: Inspect Runs tab controls for unchanged visible behavior**

Run:

```powershell
rg -n "query|queued_running|dateFrom|dateTo|provider|model|template|Clear filters" src/lib/components/analysis/run-companion-runs-tab.svelte
```

Expected: output shows the existing text input, status controls, date/provider/model/template filters, and clear-filters button. Do not redesign this component in this slice.

- [ ] **Step 4: Commit client guard test**

```powershell
git add src/lib/analysis-run-companion-state.test.ts
git commit -m "test: preserve saved run client filter guard"
```

---

### Task 6: Docs And Superpowers State

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/specs/README.md`
- Move: `docs/superpowers/specs/2026-05-30-saved-runs-backend-narrowing-design.md` to `docs/superpowers/archive/specs/2026-05-30-saved-runs-backend-narrowing-design.md`
- Modify: `docs/superpowers/plans/README.md`
- Move: `docs/superpowers/plans/2026-05-30-saved-runs-backend-narrowing-implementation.md` to `docs/superpowers/archive/plans/2026-05-30-saved-runs-backend-narrowing-implementation.md`

- [ ] **Step 1: Update backlog item after implementation**

In `docs/backlog.md`, replace the Saved Runs section checklist with:

```markdown
- [ ] consider UI affordances for missing legacy/capture failed saved-run states
```

Keep the section title `### 3.2 Saved Runs Discoverability And Cleanup` because cleanup affordances remain open. Keep the acceptance text.

- [ ] **Step 2: Archive the shipped spec**

Move the spec:

```powershell
Move-Item -LiteralPath docs\superpowers\specs\2026-05-30-saved-runs-backend-narrowing-design.md -Destination docs\superpowers\archive\specs\2026-05-30-saved-runs-backend-narrowing-design.md
```

Update `docs/superpowers/specs/README.md` so the active specs list is:

```markdown
Active specs:

- None currently.
```

If another active spec was added after this plan was written, preserve that active entry and remove only the saved-runs backend narrowing entry.

- [ ] **Step 3: Remove this completed active plan from the active plans index**

Update `docs/superpowers/plans/README.md` so the active plans list does not contain this plan. If no active plans remain, the section should read:

```markdown
Active plans:

- None currently.
```

After the final verification commit, move this plan to `docs/superpowers/archive/plans/2026-05-30-saved-runs-backend-narrowing-implementation.md`. This slice keeps the plan as historical context because it documents backend filter semantics and stale-response behavior.

- [ ] **Step 4: Commit docs state**

```powershell
git add docs/backlog.md docs/superpowers/specs/README.md docs/superpowers/archive/specs/2026-05-30-saved-runs-backend-narrowing-design.md docs/superpowers/plans/README.md docs/superpowers/plans/2026-05-30-saved-runs-backend-narrowing-implementation.md docs/superpowers/archive/plans/2026-05-30-saved-runs-backend-narrowing-implementation.md
git commit -m "docs: archive saved runs narrowing work"
```


---

### Task 7: Verification

**Files:**
- Inspect: full working tree

- [ ] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/api/analysis-runs.test.ts src/lib/analysis-run-workflow.test.ts src/lib/analysis-route-effects.test.ts src/lib/analysis-run-companion-state.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run focused backend tests**

Run:

```powershell
cargo test -p extractum-tauri analysis::store::tests::list_analysis_run_summaries -- --nocapture
```

Expected: PASS.

- [ ] **Step 3: Run full verification**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS, including Vitest, Svelte check, Rust tests, and diff check.

- [ ] **Step 4: Run opt-in Analysis smoke if GUI automation is available**

Run:

```powershell
npm.cmd run smoke:analysis
```

Expected: PASS with all bridge probes, Source Browser smoke steps, and Workspace Parity smoke steps. In this Windows sandbox, GUI smoke may require escalation; if the command cannot access the GUI bridge in the current environment, record `not run in this environment` in the final verification note and do not treat that as a product failure.

- [ ] **Step 5: Inspect final diff**

Run:

```powershell
git status --short
git diff --stat HEAD
```

Expected: only saved-run backend narrowing code, tests, and docs are changed since the last commit. If all tasks committed cleanly, `git status --short` should be empty.

---

## Self-Review

Spec coverage:

- Backend filters before limit: Task 1 query-before-limit test and query builder.
- Scope mutual exclusion: Task 1 validation test and store validation.
- Date format and invalid-date semantics: Task 1 date tests and strict `YYYY-MM-DD` parser.
- Literal search escaping: Task 1 literal `%`/`_` test and `escaped_like_contains`.
- Query terms `AND` across terms and `OR` across fields: Task 1 multi-term test and `RUN_QUERY_FIELDS`.
- Provider/model/template semantics: Task 1 field tests and template-name predicate.
- Frontend API contract: Task 2.
- Workflow filter projection and stale response guard: Task 3.
- Debounced reload from canonical `runsFilter`: Task 4.
- Local client guard remains: Task 5.
- Verification and smoke expectations: Task 7.

Placeholder scan:

- The plan contains concrete file paths, commands, snippets, and expected results.
- No implementation step relies on unspecified helper behavior.

Type consistency:

- `AnalysisRunListFilters` fields are snake_case in Rust and match command parameters.
- `ListAnalysisRunsInput` fields are camelCase in TypeScript and match Tauri invoke payloads.
- `analysisRunsBackendFilters` returns the optional filter subset consumed by `listRuns`.
