# Analysis Snapshot Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make saved analysis runs snapshot-first by capturing `analysis_run_messages` before provider calls and exposing explicit snapshot state on saved-run DTOs.

**Architecture:** Add migration 25 marker columns to `analysis_runs`, compute snapshot state from marker columns plus saved snapshot row counts, and replace the current late `persist_run_snapshot` flow with a transactional capture helper that reloads and verifies rows before setting `snapshot_captured_at`. The report pipeline must use the reloaded snapshot for provider input and trace building, while saved-run read paths stop reconstructing missing completed snapshots from live sources.

**Tech Stack:** Rust, SQLx SQLite, Tauri commands, zstd-backed snapshot rows, Svelte/TypeScript API types, Vitest, `cargo test`.

---

## File Structure

- Create `src-tauri/migrations/25.sql`: add `snapshot_captured_at` and `snapshot_error` to `analysis_runs`.
- Modify `src-tauri/src/migrations.rs`: register migration 25 and add migration/fresh-schema tests.
- Modify `src-tauri/src/analysis/models.rs`: add `AnalysisSnapshotState`, DTO fields, and row fields needed for computed state.
- Modify `src-tauri/src/analysis/store.rs`: compute snapshot state, select snapshot marker/count fields, sanitize capture errors, and add transactional snapshot capture helpers.
- Modify `src-tauri/src/analysis/report.rs`: capture and reload the snapshot before provider execution; distinguish capture failures from provider failures.
- Modify `src-tauri/src/analysis/corpus.rs`: remove normal saved-run live fallback and add defensive captured-snapshot read checks.
- Modify `src-tauri/src/analysis/chat.rs`: keep completed follow-up chat snapshot-bound and assert missing-legacy behavior.
- Modify `src-tauri/src/analysis/mod.rs`: expose snapshot marker fields in list/detail queries and keep trace resolution snapshot-bound.
- Modify `src/lib/types/analysis.ts`: add `AnalysisSnapshotState` and marker fields to `AnalysisRunSummary`.
- Modify frontend tests that build `AnalysisRunSummary` or `AnalysisRunDetail`: include the new nullable fields.
- Modify `docs/database-schema.md`, `docs/database-schema-legacy-analysis.md`, and `docs/backlog.md`: document snapshot hardening as shipped and remaining follow-ups.

---

### Task 1: Migration 25 And Snapshot DTO State

**Files:**
- Create: `src-tauri/migrations/25.sql`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Test: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/analysis/store.rs`
- Test: `src-tauri/src/analysis/mod.rs`

- [x] **Step 1: Add failing migration registration and schema tests**

In `src-tauri/src/migrations.rs`, add this test near the existing migration registration tests:

```rust
#[test]
fn includes_analysis_snapshot_hardening_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 25)
        .expect("version 25 migration is registered");

    for fragment in [
        "ALTER TABLE analysis_runs ADD COLUMN snapshot_captured_at TEXT",
        "ALTER TABLE analysis_runs ADD COLUMN snapshot_error TEXT",
    ] {
        assert!(
            migration.sql.contains(fragment),
            "missing migration fragment {fragment}"
        );
    }
}

#[tokio::test]
async fn fresh_schema_includes_analysis_snapshot_markers() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    for column in ["snapshot_captured_at", "snapshot_error"] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('analysis_runs') WHERE name = ?",
        )
        .bind(column)
        .fetch_one(&pool)
        .await
        .expect("check analysis_runs column");
        assert_eq!(exists, 1, "missing analysis_runs.{column}");
    }
}
```

- [x] **Step 2: Run migration tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_analysis_snapshot_hardening_migration
```

Expected: fails because migration 25 is not registered.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_analysis_snapshot_markers
```

Expected: fails because the columns are missing.

- [x] **Step 3: Add and register migration 25**

Create `src-tauri/migrations/25.sql`:

```sql
ALTER TABLE analysis_runs ADD COLUMN snapshot_captured_at TEXT;
ALTER TABLE analysis_runs ADD COLUMN snapshot_error TEXT;
```

In `src-tauri/src/migrations.rs`, append this entry after version 24 in `build_migrations()`:

```rust
Migration {
    version: 25,
    description: "harden analysis run snapshots",
    sql: include_str!("../migrations/25.sql"),
    kind: MigrationKind::Up,
},
```

- [x] **Step 4: Add failing DTO mapping tests**

In `src-tauri/src/analysis/store.rs`, extend `sample_run_row()` with these fields after `scope_label_snapshot`:

```rust
snapshot_captured_at: Some("2026-05-18T10:00:00Z".to_string()),
snapshot_error: None,
snapshot_message_count: 2,
```

Add these tests:

```rust
#[test]
fn map_run_summary_exposes_captured_snapshot_state() {
    let summary = map_run_summary(sample_run_row());

    assert_eq!(
        summary.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::Captured)
    );
    assert_eq!(
        summary.snapshot_captured_at.as_deref(),
        Some("2026-05-18T10:00:00Z")
    );
    assert_eq!(summary.snapshot_error, None);
}

#[test]
fn map_run_detail_exposes_missing_legacy_snapshot_state() {
    let mut row = sample_run_row();
    row.snapshot_captured_at = None;
    row.snapshot_error = None;
    row.snapshot_message_count = 0;
    row.status = crate::analysis::ANALYSIS_STATUS_COMPLETED.to_string();

    let detail = map_run_detail(row);

    assert_eq!(
        detail.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::MissingLegacy)
    );
    assert_eq!(detail.snapshot_captured_at, None);
    assert_eq!(detail.snapshot_error, None);
}

#[test]
fn map_run_summary_exposes_capture_failed_snapshot_state() {
    let mut row = sample_run_row();
    row.snapshot_captured_at = None;
    row.snapshot_error = Some("Snapshot capture failed".to_string());
    row.snapshot_message_count = 0;
    row.status = crate::analysis::ANALYSIS_STATUS_FAILED.to_string();

    let summary = map_run_summary(row);

    assert_eq!(
        summary.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );
    assert_eq!(
        summary.snapshot_error.as_deref(),
        Some("Snapshot capture failed")
    );
}

#[test]
fn map_run_summary_exposes_null_snapshot_state_for_active_runs_before_capture() {
    let mut row = sample_run_row();
    row.snapshot_captured_at = None;
    row.snapshot_error = None;
    row.snapshot_message_count = 0;
    row.status = crate::analysis::ANALYSIS_STATUS_RUNNING.to_string();

    let summary = map_run_summary(row);

    assert_eq!(summary.snapshot_state, None);
}

#[test]
fn failed_terminal_run_without_capture_marker_is_capture_failed() {
    let mut row = sample_run_row();
    row.snapshot_captured_at = None;
    row.snapshot_error = None;
    row.snapshot_message_count = 0;
    row.status = crate::analysis::ANALYSIS_STATUS_CANCELLED.to_string();

    let summary = map_run_summary(row);

    assert_eq!(
        summary.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );
}
```

- [x] **Step 5: Run DTO tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_summary_exposes_captured_snapshot_state
```

Expected: compile failure because `AnalysisSnapshotState` and DTO fields do not exist.

- [x] **Step 6: Implement snapshot state models**

In `src-tauri/src/analysis/models.rs`, add this enum before `AnalysisRunSummary`:

```rust
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisSnapshotState {
    Captured,
    MissingLegacy,
    CaptureFailed,
}
```

Add these public fields to both `AnalysisRunSummary` and `AnalysisRunDetail`, immediately after `has_trace_data`:

```rust
pub snapshot_state: Option<AnalysisSnapshotState>,
pub snapshot_captured_at: Option<String>,
pub snapshot_error: Option<String>,
```

Add this private field to `AnalysisRunDetail` after `scope_label_snapshot`:

```rust
#[serde(skip_serializing)]
pub(crate) snapshot_message_count: i64,
```

Add these fields to `AnalysisRunRow` after `scope_label_snapshot`:

```rust
pub(crate) snapshot_captured_at: Option<String>,
pub(crate) snapshot_error: Option<String>,
pub(crate) snapshot_message_count: i64,
```

- [x] **Step 7: Compute snapshot state in store mapping**

In `src-tauri/src/analysis/store.rs`, import `AnalysisSnapshotState` and status constants:

```rust
use super::models::{
    AnalysisPromptTemplate, AnalysisRunDetail, AnalysisRunRow, AnalysisRunSummary,
    AnalysisSnapshotState, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow,
    CorpusMessage,
};
use super::{
    default_report_template_body, now_secs, ANALYSIS_RUN_TYPE_REPORT,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED,
    ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
    DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_REPORT,
};
```

Add:

```rust
fn compute_snapshot_state(row: &AnalysisRunRow) -> Option<AnalysisSnapshotState> {
    if row.snapshot_captured_at.is_some() && row.snapshot_error.is_none() {
        return Some(AnalysisSnapshotState::Captured);
    }

    if row.snapshot_error.is_some() {
        return Some(AnalysisSnapshotState::CaptureFailed);
    }

    match row.status.as_str() {
        ANALYSIS_STATUS_COMPLETED if row.snapshot_message_count == 0 => {
            Some(AnalysisSnapshotState::MissingLegacy)
        }
        ANALYSIS_STATUS_FAILED | ANALYSIS_STATUS_CANCELLED => {
            Some(AnalysisSnapshotState::CaptureFailed)
        }
        _ => None,
    }
}
```

In `map_run_summary` and `map_run_detail`, compute state before moving row fields:

```rust
let snapshot_state = compute_snapshot_state(&row);
```

Then set:

```rust
snapshot_state,
snapshot_captured_at: row.snapshot_captured_at,
snapshot_error: row.snapshot_error,
```

For `AnalysisRunDetail`, also set:

```rust
snapshot_message_count: row.snapshot_message_count,
```

- [x] **Step 8: Include snapshot fields and row counts in run queries**

Update every `AnalysisRunRow` SELECT in `src-tauri/src/analysis/store.rs::fetch_run_row` and `src-tauri/src/analysis/mod.rs::list_analysis_runs` to select:

```sql
runs.snapshot_captured_at,
runs.snapshot_error,
COALESCE(snapshot_counts.snapshot_message_count, 0) AS snapshot_message_count,
```

Add this join before `WHERE` or `ORDER BY`:

```sql
LEFT JOIN (
    SELECT run_id, COUNT(*) AS snapshot_message_count
    FROM analysis_run_messages
    GROUP BY run_id
) snapshot_counts ON snapshot_counts.run_id = runs.id
```

Keep the existing joins to `sources`, `analysis_source_groups`, and
`analysis_prompt_templates`.

- [x] **Step 9: Update in-memory analysis run tables in Rust tests**

For each test-only `CREATE TABLE analysis_runs` in these files, add:

```sql
snapshot_captured_at TEXT,
snapshot_error TEXT,
```

Files to update:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/mod.rs`
- `src-tauri/src/analysis/fixtures.rs`

For direct `AnalysisRunDetail` and `AnalysisRunRow` struct literals, add:

```rust
snapshot_state: Some(crate::analysis::models::AnalysisSnapshotState::Captured),
snapshot_captured_at: Some("2026-05-18T10:00:00Z".to_string()),
snapshot_error: None,
snapshot_message_count: 1,
```

Use `snapshot_state: None`, `snapshot_captured_at: None`, `snapshot_error: None`, and `snapshot_message_count: 0` for active runs created before capture.

- [x] **Step 10: Run Task 1 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_analysis_snapshot_hardening_migration
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_analysis_snapshot_markers
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: selected tests pass, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 11: Commit migration and DTO state**

Run:

```powershell
git add src-tauri/migrations/25.sql src-tauri/src/migrations.rs src-tauri/src/analysis/models.rs src-tauri/src/analysis/store.rs src-tauri/src/analysis/mod.rs src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/fixtures.rs
git commit -m "feat: expose analysis snapshot state"
```

Expected: commit succeeds.

---

### Task 2: Transactional Snapshot Capture Store Helpers

**Files:**
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Test: `src-tauri/src/analysis/store.rs`
- Test: `src-tauri/src/analysis/corpus.rs`

- [x] **Step 1: Add failing sanitizer tests**

In `src-tauri/src/analysis/store.rs`, add:

```rust
#[test]
fn sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens() {
    let long = "x".repeat(600);
    let raw = format!(
        "failed at C:\\Users\\Dima\\AppData\\Local\\Extractum\\db.sqlite\n\
         see /home/dima/.config/extractum/db.sqlite and file:///tmp/secret.txt \
         https://example.test/path?token=abc#frag \
         bearer sk-live-secret api_key=secret {long}"
    );

    let sanitized = sanitize_snapshot_error("Snapshot capture failed", &raw);

    assert!(sanitized.chars().count() <= 512);
    assert!(!sanitized.contains('\n'));
    assert!(!sanitized.contains("C:\\"));
    assert!(!sanitized.contains("/home/dima"));
    assert!(!sanitized.contains("file://"));
    assert!(!sanitized.contains("?token="));
    assert!(!sanitized.contains("#frag"));
    assert!(!sanitized.to_lowercase().contains("bearer"));
    assert!(!sanitized.contains("sk-live-secret"));
    assert!(!sanitized.contains("api_key=secret"));
}
```

- [x] **Step 2: Add failing capture transaction tests**

In `src-tauri/src/analysis/store.rs`, add a test memory-pool helper with the new marker columns:

```rust
async fn snapshot_store_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query(
        r#"
        CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT,
            status TEXT,
            error TEXT,
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
            item_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            external_id TEXT NOT NULL,
            author TEXT,
            published_at INTEGER NOT NULL,
            ref TEXT NOT NULL,
            content_zstd BLOB NOT NULL,
            item_kind TEXT,
            source_type TEXT,
            source_subtype TEXT,
            metadata_zstd BLOB,
            PRIMARY KEY (run_id, ref)
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create messages");
    sqlx::query("INSERT INTO analysis_runs (id, status) VALUES (1, 'running')")
        .execute(&pool)
        .await
        .expect("insert run");
    pool
}

fn strict_snapshot_message(label: &str) -> CorpusMessage {
    CorpusMessage {
        item_id: 10,
        source_id: 2,
        external_id: label.to_string(),
        published_at: 1_710_000_000,
        author: Some("Alice".to_string()),
        content: format!("content {label}"),
        r#ref: format!("s2-i10-{label}"),
        item_kind: Some("telegram_message".to_string()),
        source_type: Some("telegram".to_string()),
        source_subtype: Some("channel".to_string()),
        metadata_zstd: None,
    }
}
```

Add:

```rust
#[tokio::test]
async fn capture_run_snapshot_marks_captured_after_reload_and_replaces_rows() {
    let pool = snapshot_store_pool().await;

    let first = capture_run_snapshot(&pool, 1, "Frozen scope", &[strict_snapshot_message("a")])
        .await
        .expect("capture first");
    let second = capture_run_snapshot(&pool, 1, "Frozen scope", &[strict_snapshot_message("b")])
        .await
        .expect("capture second");

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].external_id, "b");

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("count messages");
    assert_eq!(count, 1);

    let marker: Option<String> =
        sqlx::query_scalar("SELECT snapshot_captured_at FROM analysis_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("load marker");
    assert!(marker.is_some());

    let snapshot_error: Option<String> =
        sqlx::query_scalar("SELECT snapshot_error FROM analysis_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("load snapshot error");
    assert_eq!(snapshot_error, None);
}

#[tokio::test]
async fn capture_run_snapshot_rejects_missing_required_fields_without_marker() {
    let pool = snapshot_store_pool().await;
    let mut message = strict_snapshot_message("bad");
    message.item_kind = None;

    let error = capture_run_snapshot(&pool, 1, "Frozen scope", &[message])
        .await
        .expect_err("missing item_kind should fail");
    assert!(error.contains("item_kind"));

    let marker: Option<String> =
        sqlx::query_scalar("SELECT snapshot_captured_at FROM analysis_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("load marker");
    assert_eq!(marker, None);

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("count messages");
    assert_eq!(count, 0);
}
```

- [x] **Step 3: Run store tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens
```

Expected: compile failure because `sanitize_snapshot_error` does not exist.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::capture_run_snapshot_
```

Expected: compile failure because `capture_run_snapshot` does not exist.

- [x] **Step 4: Implement shared snapshot error sanitizer**

In `src-tauri/src/analysis/store.rs`, add:

```rust
pub(crate) fn sanitize_snapshot_error(category: &str, raw: &str) -> String {
    let mut text = raw
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect::<String>();

    for marker in ["file://", "C:\\", "c:\\", "/home/", "/Users/", "/tmp/"] {
        while let Some(start) = text.find(marker) {
            let end = text[start..]
                .find(char::is_whitespace)
                .map(|offset| start + offset)
                .unwrap_or_else(|| text.len());
            text.replace_range(start..end, "[redacted]");
        }
    }

    for marker in ["http://", "https://"] {
        let mut search_from = 0usize;
        while let Some(relative_start) = text[search_from..].find(marker) {
            let start = search_from + relative_start;
            let end = text[start..]
                .find(char::is_whitespace)
                .map(|offset| start + offset)
                .unwrap_or_else(|| text.len());
            let url = &text[start..end];
            let clean_end = url.find(['?', '#']).unwrap_or(url.len());
            let replacement = format!("{}[redacted]", &url[..clean_end]);
            text.replace_range(start..end, &replacement);
            search_from = start + replacement.len();
        }
    }

    let lower = text.to_lowercase();
    if lower.contains("bearer ")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("sk-")
        || lower.contains("cookie")
    {
        text = category.to_string();
    }

    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let bounded = compact.chars().take(512).collect::<String>();
    if bounded.trim().is_empty() {
        category.to_string()
    } else {
        bounded
    }
}
```

- [x] **Step 5: Implement transactional snapshot capture**

In `src-tauri/src/analysis/store.rs`, add imports:

```rust
use sqlx::{Pool, Sqlite, SqliteConnection};
```

Add validation:

```rust
fn validate_snapshot_message(message: &CorpusMessage) -> Result<(), String> {
    if message.r#ref.trim().is_empty() {
        return Err("Snapshot message ref is required".to_string());
    }
    if message.content.trim().is_empty() {
        return Err(format!("Snapshot message {} content is required", message.r#ref));
    }
    if message.item_kind.as_deref().unwrap_or("").trim().is_empty() {
        return Err(format!("Snapshot message {} item_kind is required", message.r#ref));
    }
    let source_type = message.source_type.as_deref().unwrap_or("").trim();
    if source_type.is_empty() {
        return Err(format!("Snapshot message {} source_type is required", message.r#ref));
    }
    if matches!(source_type, "telegram" | "youtube")
        && message.source_subtype.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err(format!(
            "Snapshot message {} source_subtype is required for {source_type}",
            message.r#ref
        ));
    }
    Ok(())
}
```

Add a connection-bound loader:

```rust
async fn load_run_snapshot_messages_on_connection(
    conn: &mut SqliteConnection,
    run_id: i64,
) -> Result<Vec<CorpusMessage>, String> {
    let rows: Vec<StoredRunSnapshotRow> = sqlx::query_as(
        r#"
        SELECT
            item_id,
            source_id,
            external_id,
            author,
            published_at,
            ref,
            content_zstd,
            item_kind,
            source_type,
            source_subtype,
            metadata_zstd
        FROM analysis_run_messages
        WHERE run_id = ?
        ORDER BY published_at ASC, ref ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(conn)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|row| {
            Ok(CorpusMessage {
                item_id: row.item_id,
                source_id: row.source_id,
                external_id: row.external_id,
                published_at: row.published_at,
                author: row.author,
                content: crate::compression::decompress_text(&row.content_zstd)?,
                r#ref: row.r#ref,
                item_kind: row.item_kind,
                source_type: row.source_type,
                source_subtype: row.source_subtype,
                metadata_zstd: row.metadata_zstd,
            })
        })
        .collect()
}
```

Replace `persist_run_snapshot` with this new helper and keep a wrapper:

```rust
pub(crate) async fn capture_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> Result<Vec<CorpusMessage>, String> {
    if corpus.is_empty() {
        return Err("Snapshot capture failed: empty corpus".to_string());
    }

    for message in corpus {
        validate_snapshot_message(message)?;
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("UPDATE analysis_runs SET scope_label_snapshot = ?, snapshot_captured_at = NULL, snapshot_error = NULL WHERE id = ?")
        .bind(scope_label)
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM analysis_run_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for message in corpus {
        let content_zstd = compress_text(&message.content)?;
        sqlx::query(
            r#"
            INSERT INTO analysis_run_messages (
                run_id,
                item_id,
                source_id,
                external_id,
                author,
                published_at,
                ref,
                content_zstd,
                item_kind,
                source_type,
                source_subtype,
                metadata_zstd
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(run_id)
        .bind(message.item_id)
        .bind(message.source_id)
        .bind(&message.external_id)
        .bind(&message.author)
        .bind(message.published_at)
        .bind(&message.r#ref)
        .bind(content_zstd)
        .bind(message.item_kind.as_deref())
        .bind(message.source_type.as_deref())
        .bind(message.source_subtype.as_deref())
        .bind(message.metadata_zstd.as_deref())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    let captured = load_run_snapshot_messages_on_connection(&mut tx, run_id).await?;
    if captured.is_empty() {
        return Err("Snapshot capture failed: reloaded snapshot is empty".to_string());
    }

    sqlx::query(
        "UPDATE analysis_runs SET snapshot_captured_at = datetime('now'), snapshot_error = NULL WHERE id = ?",
    )
    .bind(run_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(captured)
}

pub(crate) async fn persist_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> Result<(), String> {
    capture_run_snapshot(pool, run_id, scope_label, corpus)
        .await
        .map(|_| ())
}
```

If the compiler rejects `load_run_snapshot_messages_on_connection(&mut tx, run_id)`, change the helper argument to `&mut sqlx::Transaction<'_, Sqlite>` and use `fetch_all(&mut **tx)` inside the helper.

- [x] **Step 6: Update snapshot fixtures for strict source subtype**

In `src-tauri/src/analysis/corpus.rs::sample_corpus`, change the Telegram message from:

```rust
source_subtype: None,
```

to:

```rust
source_subtype: Some("channel".to_string()),
```

Do the same for `sample_corpus_message()` in `src-tauri/src/analysis/report.rs` and any other new snapshot-capture test fixture that uses `source_type: Some("telegram".to_string())`.

- [x] **Step 7: Run Task 2 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::capture_run_snapshot_
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::run_snapshot_roundtrips_frozen_corpus
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: selected tests pass, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 8: Commit snapshot capture helpers**

Run:

```powershell
git add src-tauri/src/analysis/store.rs src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/report.rs
git commit -m "feat: capture analysis snapshots transactionally"
```

Expected: commit succeeds.

---

### Task 3: Early Snapshot Capture In Report Pipeline

**Files:**
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Test: `src-tauri/src/analysis/report.rs`

- [x] **Step 1: Add failing early-capture helper tests**

In `src-tauri/src/analysis/report.rs`, add a small helper test around an extracted function named `capture_report_corpus`:

```rust
#[tokio::test]
async fn capture_report_corpus_returns_reloaded_snapshot_before_provider_phases() {
    let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
    crate::sources::test_support::create_analysis_documents_table(&pool).await;
    sqlx::query(
        "CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("create runs");
    sqlx::query(
        "CREATE TABLE analysis_run_messages (
            run_id INTEGER NOT NULL,
            item_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            external_id TEXT NOT NULL,
            author TEXT,
            published_at INTEGER NOT NULL,
            ref TEXT NOT NULL,
            content_zstd BLOB NOT NULL,
            item_kind TEXT,
            source_type TEXT,
            source_subtype TEXT,
            metadata_zstd BLOB,
            PRIMARY KEY (run_id, ref)
        )",
    )
    .execute(&pool)
    .await
    .expect("create run messages");
    sqlx::query("INSERT INTO analysis_runs (id) VALUES (1)")
        .execute(&pool)
        .await
        .expect("insert run");
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
         VALUES (2, 'telegram', 'channel', 'tg2', 'Telegram 2', 1, 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert source");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_kind, has_media, content_zstd)
         VALUES (10, 2, '10', 'telegram_message', 'Alice', 100, 100, 'text_only', 0, ?)",
    )
    .bind(crate::compression::compress_text("captured text").expect("compress"))
    .execute(&pool)
    .await
    .expect("insert item");
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 2)
        .await
        .expect("rebuild docs");

    let request = CorpusLoadRequest {
        source_type: "telegram".to_string(),
        source_ids: vec![2],
        period_from: 1,
        period_to: 1_000,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
    };

    let captured = capture_report_corpus(&pool, 1, "Frozen source", &request)
        .await
        .expect("capture report corpus");

    sqlx::query("DELETE FROM analysis_documents WHERE source_id = 2")
        .execute(&pool)
        .await
        .expect("delete live docs after capture");

    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].content, "captured text");
}
```

- [x] **Step 2: Run early-capture test and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
```

Expected: compile failure because `capture_report_corpus` does not exist.

- [x] **Step 3: Implement report capture helper**

In `src-tauri/src/analysis/report.rs`, update imports:

```rust
use super::store::{
    capture_run_snapshot, fetch_prompt_template, fetch_run_row, fetch_source_group,
    find_active_duplicate_run, insert_analysis_run, sanitize_snapshot_error, set_run_status,
    AnalysisRunInsert, DuplicateRunLookup,
};
```

Add:

```rust
const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";

async fn capture_report_corpus(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, ReportRunError> {
    let corpus = load_corpus_messages(pool, request)
        .await
        .map_err(|error| {
            ReportRunError::CaptureFailed(sanitize_snapshot_error(
                "Corpus preload failed",
                &error,
            ))
        })?;

    if corpus.is_empty() {
        return Err(ReportRunError::CaptureFailed(
            SNAPSHOT_CAPTURE_FAILED_MESSAGE.to_string(),
        ));
    }

    capture_run_snapshot(pool, run_id, scope_label, &corpus)
        .await
        .map_err(|error| {
            ReportRunError::CaptureFailed(sanitize_snapshot_error(
                SNAPSHOT_CAPTURE_FAILED_MESSAGE,
                &error,
            ))
        })
}
```

Change `ReportRunError` to:

```rust
enum ReportRunError {
    Failed(String),
    CaptureFailed(String),
    Cancelled(String),
}
```

- [x] **Step 4: Move snapshot capture before chunking/provider work**

In `run_report_pipeline`, replace:

```rust
let corpus = load_corpus_messages(&pool, &input.corpus_request)
    .await
    .map_err(ReportRunError::Failed)?;
if corpus.is_empty() {
    return Err(ReportRunError::Failed(
        "No synced source documents were found for the selected analysis scope and period"
            .to_string(),
    ));
}
```

with:

```rust
let corpus = capture_report_corpus(&pool, run_id, &input.scope_label, &input.corpus_request)
    .await?;
```

Remove the late call to `persist_run_snapshot` before `set_run_status`. Keep `build_trace_data(&reduce_result.completion.text, &corpus)` exactly on the reloaded captured corpus.

- [x] **Step 5: Add capture failure persistence helper**

In `src-tauri/src/analysis/store.rs`, add:

```rust
pub(crate) async fn mark_run_capture_failed(
    pool: &Pool<Sqlite>,
    run_id: i64,
    snapshot_error: &str,
    completed_at: i64,
) -> Result<(), String> {
    let sanitized = sanitize_snapshot_error("Snapshot capture failed", snapshot_error);
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET
            status = ?,
            error = ?,
            snapshot_error = ?,
            completed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(crate::analysis::ANALYSIS_STATUS_FAILED)
    .bind(&sanitized)
    .bind(&sanitized)
    .bind(completed_at)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
```

In `src-tauri/src/analysis/report.rs`, add `mark_run_capture_failed` to the store imports and add:

```rust
async fn fail_capture_run(handle: &AppHandle, run_id: i64, error: String) {
    if let Ok(pool) = get_pool(handle).await {
        let _ = mark_run_capture_failed(&pool, run_id, &error, now_secs()).await;
    }

    RunEvent::new(run_id, "failed", "persist")
        .message("Report run failed before snapshot capture completed.".to_string())
        .error(error)
        .emit(handle);
}
```

In the `tokio::spawn` match, handle capture failures separately:

```rust
Err(ReportRunError::CaptureFailed(error)) => {
    fail_capture_run(&app_handle, run_id, error).await
}
```

- [x] **Step 6: Run Task 3 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: selected tests pass, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 7: Commit early capture pipeline**

Run:

```powershell
git add src-tauri/src/analysis/report.rs src-tauri/src/analysis/store.rs
git commit -m "feat: capture analysis snapshot before provider calls"
```

Expected: commit succeeds.

---

### Task 4: Capture Failure, Provider Failure, Cancellation, And Recovery Semantics

**Files:**
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Test: `src-tauri/src/analysis/report.rs`
- Test: `src-tauri/src/analysis/store.rs`

- [x] **Step 1: Add failing failure semantics tests**

In `src-tauri/src/analysis/store.rs`, add:

```rust
#[tokio::test]
async fn mark_run_capture_failed_sets_snapshot_error() {
    let pool = snapshot_store_pool().await;

    mark_run_capture_failed(
        &pool,
        1,
        "failed at C:\\Users\\Dima\\secret.sqlite?token=abc",
        1_710_000_500,
    )
    .await
    .expect("mark capture failed");

    let row: (String, Option<String>, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT status, error, snapshot_error, completed_at FROM analysis_runs WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load run");

    assert_eq!(row.0, crate::analysis::ANALYSIS_STATUS_FAILED);
    assert_eq!(row.1, row.2);
    assert_eq!(row.3, Some(1_710_000_500));
    assert!(!row.2.unwrap().contains("C:\\"));
}
```

In `src-tauri/src/analysis/report.rs`, extract interrupted cleanup into a testable helper:

```rust
#[tokio::test]
async fn interrupted_cleanup_preserves_captured_snapshot_state_marker() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query(
        "CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY,
            status TEXT NOT NULL,
            error TEXT,
            completed_at INTEGER,
            snapshot_captured_at TEXT,
            snapshot_error TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("create runs");
    sqlx::query(
        "INSERT INTO analysis_runs (id, status, snapshot_captured_at, snapshot_error)
         VALUES (1, 'running', '2026-05-18T10:00:00Z', NULL)",
    )
    .execute(&pool)
    .await
    .expect("insert running captured run");

    mark_interrupted_analysis_runs(&pool)
        .await
        .expect("mark interrupted");

    let row: (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT status, snapshot_captured_at, snapshot_error FROM analysis_runs WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load run");

    assert_eq!(row.0, crate::analysis::ANALYSIS_STATUS_CANCELLED);
    assert_eq!(row.1.as_deref(), Some("2026-05-18T10:00:00Z"));
    assert_eq!(row.2, None);
}
```

- [x] **Step 2: Run failure semantics tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::mark_run_capture_failed_sets_snapshot_error
```

Expected: compile failure until `mark_run_capture_failed` exists.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::interrupted_cleanup_preserves_captured_snapshot_state_marker
```

Expected: compile failure until `mark_interrupted_analysis_runs` exists.

- [x] **Step 3: Implement interrupted cleanup helper**

In `src-tauri/src/analysis/report.rs`, add:

```rust
pub(crate) async fn mark_interrupted_analysis_runs(pool: &Pool<Sqlite>) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET status = ?, error = ?, completed_at = ?
        WHERE status IN (?, ?)
        "#,
    )
    .bind(ANALYSIS_STATUS_CANCELLED)
    .bind(INTERRUPTED_RUN_MESSAGE)
    .bind(now_secs())
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(ANALYSIS_STATUS_RUNNING)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
```

Replace the SQL body in `cleanup_interrupted_analysis_runs` with:

```rust
let _ = mark_interrupted_analysis_runs(&pool).await;
```

Do not set `snapshot_error` in this helper.

- [x] **Step 4: Assert provider/cancel errors do not write snapshot_error**

Add store-level tests:

```rust
#[tokio::test]
async fn provider_failure_status_update_does_not_write_snapshot_error() {
    let pool = snapshot_store_pool().await;
    sqlx::query(
        "UPDATE analysis_runs SET snapshot_captured_at = '2026-05-18T10:00:00Z' WHERE id = 1",
    )
    .execute(&pool)
    .await
    .expect("mark captured");

    set_run_status(
        &pool,
        1,
        crate::analysis::ANALYSIS_STATUS_FAILED,
        None,
        None,
        Some("Provider network failed"),
        Some(1_710_000_500),
    )
    .await
    .expect("mark provider failed");

    let snapshot_error: Option<String> =
        sqlx::query_scalar("SELECT snapshot_error FROM analysis_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("load snapshot_error");
    assert_eq!(snapshot_error, None);
}

#[tokio::test]
async fn cancellation_after_capture_does_not_write_snapshot_error() {
    let pool = snapshot_store_pool().await;
    sqlx::query(
        "UPDATE analysis_runs SET snapshot_captured_at = '2026-05-18T10:00:00Z' WHERE id = 1",
    )
    .execute(&pool)
    .await
    .expect("mark captured");

    set_run_status(
        &pool,
        1,
        crate::analysis::ANALYSIS_STATUS_CANCELLED,
        None,
        None,
        Some("Analysis run cancelled."),
        Some(1_710_000_500),
    )
    .await
    .expect("mark cancelled");

    let snapshot_error: Option<String> =
        sqlx::query_scalar("SELECT snapshot_error FROM analysis_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("load snapshot_error");
    assert_eq!(snapshot_error, None);
}
```

- [x] **Step 5: Run Task 4 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::mark_run_capture_failed_sets_snapshot_error
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::provider_failure_status_update_does_not_write_snapshot_error
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::cancellation_after_capture_does_not_write_snapshot_error
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::interrupted_cleanup_preserves_captured_snapshot_state_marker
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: selected tests pass, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 6: Commit failure semantics**

Run:

```powershell
git add src-tauri/src/analysis/report.rs src-tauri/src/analysis/store.rs
git commit -m "feat: harden analysis snapshot failure states"
```

Expected: commit succeeds.

---

### Task 5: Saved-Run Snapshot Read Path Hardening

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Test: `src-tauri/src/analysis/corpus.rs`
- Test: `src-tauri/src/analysis/chat.rs`
- Test: `src-tauri/src/analysis/mod.rs`

- [x] **Step 1: Add failing no-live-fallback and corrupt snapshot tests**

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
#[tokio::test]
async fn load_run_corpus_messages_does_not_reconstruct_completed_missing_legacy_from_live_rows() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_id, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status, created_at
         )
         VALUES (1, 'report', 'single_source', 2, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)",
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (11, 2, '100', 'telegram_message', 'Alice', ?, ?)",
    )
    .bind(1_710_000_000_i64)
    .bind(compress_text("live drift").expect("compress"))
    .execute(&pool)
    .await
    .expect("insert live item");
    rebuild_documents_for_sources(&pool, &[2]).await;

    let mut run = sample_run();
    run.id = 1;
    run.scope_type = crate::analysis::ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE.to_string();
    run.source_id = Some(2);
    run.source_group_id = None;
    run.snapshot_state = Some(crate::analysis::models::AnalysisSnapshotState::MissingLegacy);
    run.snapshot_captured_at = None;
    run.snapshot_error = None;
    run.snapshot_message_count = 0;

    let corpus = load_run_corpus_messages(&pool, &run)
        .await
        .expect("load snapshot-only corpus");

    assert!(corpus.is_empty());
}

#[tokio::test]
async fn captured_marker_with_missing_rows_returns_corrupt_snapshot_error() {
    let pool = snapshot_pool().await;
    let mut run = sample_run();
    run.snapshot_state = Some(crate::analysis::models::AnalysisSnapshotState::Captured);
    run.snapshot_captured_at = Some("2026-05-18T10:00:00Z".to_string());
    run.snapshot_error = None;
    run.snapshot_message_count = 0;

    let error = load_run_corpus_messages(&pool, &run)
        .await
        .expect_err("captured marker without rows should fail defensively");

    assert!(error.contains("snapshot is unavailable"));
}
```

- [x] **Step 2: Add source group drift test**

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
#[tokio::test]
async fn source_group_membership_drift_after_capture_does_not_change_saved_run_corpus() {
    let pool = snapshot_pool().await;
    sqlx::query("INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at) VALUES (9, 'Group', 'telegram', 1, 1)")
        .execute(&pool)
        .await
        .expect("insert group");
    sqlx::query("INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (9, 2, 1), (9, 4, 1)")
        .execute(&pool)
        .await
        .expect("insert original members");
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_group_id, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status, snapshot_captured_at, created_at
         )
         VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', '2026-05-18T10:00:00Z', ?)",
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");
    persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
        .await
        .expect("persist snapshot");
    sqlx::query("DELETE FROM analysis_source_group_members WHERE group_id = 9 AND source_id = 4")
        .execute(&pool)
        .await
        .expect("remove member after capture");

    let mut run = sample_run();
    run.id = 1;
    run.source_group_id = Some(9);
    run.snapshot_state = Some(crate::analysis::models::AnalysisSnapshotState::Captured);
    run.snapshot_captured_at = Some("2026-05-18T10:00:00Z".to_string());
    run.snapshot_message_count = 2;

    let corpus = load_run_corpus_messages(&pool, &run)
        .await
        .expect("load saved corpus");

    assert_eq!(corpus.len(), 2);
    assert_eq!(
        corpus.iter().map(|message| message.source_id).collect::<Vec<_>>(),
        vec![2, 4]
    );
}
```

This test uses `sample_corpus()` source ids as the frozen snapshot. It proves the saved corpus is whatever was captured, not the current group membership resolver output after a member is removed.

- [x] **Step 3: Run read-path tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::load_run_corpus_messages_does_not_reconstruct_completed_missing_legacy_from_live_rows
```

Expected: fails because `load_run_corpus_messages` reconstructs live rows.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::captured_marker_with_missing_rows_returns_corrupt_snapshot_error
```

Expected: fails because captured missing rows currently fall through to live fallback or empty handling.

- [x] **Step 4: Implement snapshot-only saved-run corpus loading**

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
fn captured_snapshot_missing_error(run_id: i64) -> String {
    format!("Analysis run {run_id} captured snapshot is unavailable")
}

fn ensure_captured_snapshot_rows(
    run: &AnalysisRunDetail,
    snapshot: &[CorpusMessage],
) -> Result<(), String> {
    if run.snapshot_state == Some(crate::analysis::models::AnalysisSnapshotState::Captured)
        && run.snapshot_message_count == 0
        && snapshot.is_empty()
    {
        return Err(captured_snapshot_missing_error(run.id));
    }
    Ok(())
}
```

Replace `load_run_corpus_messages` with:

```rust
pub(crate) async fn load_run_corpus_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<CorpusMessage>, String> {
    let snapshot = load_run_snapshot_messages(pool, run.id).await?;
    ensure_captured_snapshot_rows(run, &snapshot)?;
    Ok(snapshot)
}
```

Replace `load_trace_resolution_messages` with:

```rust
pub(crate) async fn load_trace_resolution_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<CorpusMessage>, String> {
    let snapshot = load_run_snapshot_messages(pool, run.id).await?;
    ensure_captured_snapshot_rows(run, &snapshot)?;
    Ok(snapshot)
}
```

Remove the old live reconstruction block from `load_run_corpus_messages`. Do not call `resolve_analysis_sources` from saved-run corpus loading.

- [x] **Step 5: Add missing-legacy command behavior tests**

In `src-tauri/src/analysis/mod.rs`, extend the tests with a memory table that includes the marker columns and `analysis_run_messages`. Add:

```rust
#[tokio::test]
async fn list_analysis_run_messages_returns_empty_page_for_missing_legacy_run() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_id, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status, result_markdown,
            created_at, completed_at
         )
         VALUES (1, 'report', 'single_source', 2, 1, 2, 'English', 1, 'default', 'gemini', 'model', 'completed', 'Saved report', 1, 2)",
    )
    .execute(&pool)
    .await
    .expect("insert run");

    let detail = super::store::fetch_run_row(&pool, 1)
        .await
        .expect("fetch run")
        .map(super::store::map_run_detail)
        .expect("run exists");

    assert_eq!(
        detail.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::MissingLegacy)
    );
}
```

Keep `list_analysis_run_messages` snapshot-only. It already returns an empty page when no rows exist; the run-level DTO carries `missing_legacy`.

- [x] **Step 6: Keep follow-up chat snapshot-bound**

In `src-tauri/src/analysis/chat.rs`, update existing `completed_chat_context_requires_saved_snapshot_messages` to assert the conflict message:

```rust
assert_eq!(
    error.message,
    "This completed analysis run has no saved snapshot context for follow-up chat"
);
```

No live corpus loader should be called from `ask_analysis_run_question`.

- [x] **Step 7: Run Task 5 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::load_run_corpus_messages_does_not_reconstruct_completed_missing_legacy_from_live_rows
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::captured_marker_with_missing_rows_returns_corrupt_snapshot_error
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::source_group_membership_drift_after_capture_does_not_change_saved_run_corpus
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::completed_chat_context_requires_saved_snapshot_messages
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: selected tests pass, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 8: Commit saved-run read path hardening**

Run:

```powershell
git add src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/chat.rs src-tauri/src/analysis/mod.rs
git commit -m "feat: keep saved analysis reads snapshot bound"
```

Expected: commit succeeds.

---

### Task 6: Frontend Type Compatibility

**Files:**
- Modify: `src/lib/types/analysis.ts`
- Modify: frontend tests that construct `AnalysisRunSummary` or `AnalysisRunDetail`
- Test: `src/lib/analysis-state.test.ts`
- Test: `src/lib/analysis-run-workflow.test.ts`
- Test: `src/lib/analysis-chat-workflow.test.ts`
- Test: `src/lib/analysis-run-companion-state.test.ts`
- Test: `src/lib/analysis-trace-workflow.test.ts`

- [x] **Step 1: Add TypeScript snapshot state type**

In `src/lib/types/analysis.ts`, add before `AnalysisRunSummary`:

```ts
export type AnalysisSnapshotState = "captured" | "missing_legacy" | "capture_failed";
```

Add these fields to `AnalysisRunSummary` after `has_trace_data`:

```ts
snapshot_state: AnalysisSnapshotState | null;
snapshot_captured_at: string | null;
snapshot_error: string | null;
```

`AnalysisRunDetail extends AnalysisRunSummary`, so do not duplicate the fields there.

- [x] **Step 2: Update frontend test builders**

For every test builder returning `AnalysisRunSummary` or `AnalysisRunDetail`, add:

```ts
snapshot_state: "captured",
snapshot_captured_at: "2026-05-18T10:00:00Z",
snapshot_error: null,
```

Use these values for ordinary completed saved-run fixtures. Use `snapshot_state: null`, `snapshot_captured_at: null`, and `snapshot_error: null` for queued/running fixtures. Use `snapshot_state: "missing_legacy"` for fixture runs that intentionally have saved markdown but no snapshot messages.

Files to scan and update:

```powershell
rg -n "function run|function runSummary|function runDetail|AnalysisRunSummary|AnalysisRunDetail" src\\lib src\\routes\\analysis
```

- [x] **Step 3: Run frontend type/tests**

Run:

```powershell
npm test -- src/lib/analysis-state.test.ts src/lib/analysis-run-workflow.test.ts src/lib/analysis-chat-workflow.test.ts src/lib/analysis-run-companion-state.test.ts src/lib/analysis-trace-workflow.test.ts
npm run check
git diff --check
```

Expected: selected Vitest suites pass, Svelte check passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 4: Commit frontend compatibility**

Run:

```powershell
git add src/lib/types/analysis.ts src/lib src/routes/analysis
git commit -m "feat: type analysis snapshot state in frontend"
```

Expected: commit succeeds.

---

### Task 7: Documentation And Full Verification

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/database-schema-legacy-analysis.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/specs/2026-05-18-analysis-snapshot-hardening-design.md`
- Modify: `docs/superpowers/plans/2026-05-18-analysis-snapshot-hardening.md`
- Test: full Rust suite
- Test: frontend checks/tests

- [ ] **Step 1: Update schema docs**

In `docs/database-schema.md`, update the `analysis_runs` section to include:

```markdown
- `snapshot_captured_at`: set after a report run's frozen corpus has been
  persisted to `analysis_run_messages`, reloaded, and verified as usable before
  provider execution.
- `snapshot_error`: bounded sanitized error category for capture-preventing
  failures only. Provider/model/auth/network failures after successful capture
  remain in `error` and do not populate `snapshot_error`.
```

Update the `analysis_run_messages` section:

```markdown
For new report runs, `analysis_run_messages` is captured before provider
execution and is the authoritative saved-run corpus for provider prompts,
trace building, evidence resolution, saved-run source context, and follow-up
chat. Completed historical runs without rows are treated as missing legacy
snapshots; saved-run read paths must not reconstruct them from current live
sources.
```

Update the migration history table with:

```markdown
| 25 | `25.sql` | Add analysis snapshot capture marker and error columns |
```

- [ ] **Step 2: Update legacy analysis and backlog docs**

In `docs/database-schema-legacy-analysis.md`, mark snapshot hardening as shipped:

```markdown
Analysis snapshot hardening is shipped: new report runs capture
`analysis_run_messages` before provider execution and expose explicit snapshot
state on saved-run DTOs. Historical completed rows without saved messages remain
readable as missing legacy snapshots, but evidence, chat, and saved-run source
context do not reconstruct them from live provider/archive state.
```

In `docs/backlog.md`, keep future work limited to UI presentation or export follow-ups:

```markdown
- [ ] consider UI affordances for missing legacy/capture failed saved-run states
- [ ] consider saved-analysis-snapshot export based on `analysis_run_messages`
```

- [ ] **Step 3: Mark plan steps complete as each task lands**

Before the Task 7 commit, ensure every completed step in this plan uses `[x]`.
Do not mark a step complete before running its verification command.

- [ ] **Step 4: Run containment scans**

Run:

```powershell
rg -n "snapshot_error" src-tauri/src/analysis
rg -n "load_run_corpus_messages|load_trace_resolution_messages|resolve_analysis_sources" src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/chat.rs src-tauri/src/analysis/mod.rs
rg -n "TranscriptDescription" src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/report.rs
rg -n "T[O]DO|T[B]D|FIX[M]E|if poss[i]ble|not_captured_d[u]e" docs src-tauri/src src/lib
```

Expected:

- `snapshot_error` is written only by capture-failure paths and schema/DTO code;
- saved-run corpus/trace/chat paths do not reconstruct completed snapshotless runs from live sources;
- YouTube `TranscriptDescription` is used for live/default request construction only, not as a saved-run fallback;
- no new unfinished-work markers are introduced.

- [ ] **Step 5: Run targeted Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_analysis_snapshot_hardening_migration
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_analysis_snapshot_markers
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::tests::
```

Expected: all targeted Rust tests pass.

- [ ] **Step 6: Run full verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
npm test
npm run check
git diff --check
git status --short
```

Expected: Rust suite, Rust formatting, Vitest suite, Svelte check, and diff check pass. `git status --short` shows only intended Task 7 docs/plan changes before commit.

- [ ] **Step 7: Commit docs and final verification notes**

Run:

```powershell
git add docs/database-schema.md docs/database-schema-legacy-analysis.md docs/backlog.md docs/superpowers/specs/2026-05-18-analysis-snapshot-hardening-design.md docs/superpowers/plans/2026-05-18-analysis-snapshot-hardening.md
git commit -m "docs: document analysis snapshot hardening"
```

Expected: commit succeeds.

---

## Self-Review Checklist

- Spec coverage: Task 1 covers migration 25 and DTO state rules; Task 2 covers transactional capture, strict snapshot row contract, idempotent replacement, and sanitizer requirements; Task 3 moves capture before provider execution and uses the reloaded corpus; Task 4 covers capture/provider/cancel/restart failure semantics; Task 5 removes saved-run live reconstruction and adds corrupt/missing legacy behavior; Task 6 covers frontend type compatibility; Task 7 covers docs and full verification.
- Drift coverage: Task 3 covers live document drift after capture; Task 5 covers source-group membership drift after capture.
- Snapshot marker semantics: only `capture_run_snapshot` writes `snapshot_captured_at`, and only capture-failure paths write `snapshot_error`.
- Saved-run containment: `list_analysis_run_messages`, trace resolution, follow-up chat, and saved-run source context stay snapshot-bound; ordinary live source browsing remains live and out of scope.
- TDD: every implementation task starts with failing tests and exact commands for the red and green loops.
