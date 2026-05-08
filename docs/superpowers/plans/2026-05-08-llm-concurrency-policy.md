# LLM Concurrency Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add backend analysis preflight caps so large local corpora cannot create unbounded report runs, while documenting the existing LLM scheduler concurrency policy.

**Architecture:** Keep request execution policy in `llm::scheduler`. Add analysis-run preflight in `analysis::corpus` and call it from `analysis::report` before duplicate-run handling and run insertion. Emit a preflight summary when a run starts, and reject oversized runs with typed validation errors.

**Tech Stack:** Rust, Tauri commands, SQLx SQLite, existing analysis report pipeline, existing LLM scheduler tests, Vitest/Svelte check for frontend regression.

---

## File Structure

- Modify `src-tauri/src/analysis/corpus.rs`
  - Add preflight structs, constants, estimated-size helpers, query logic, and unit tests.
- Modify `src-tauri/src/analysis/report.rs`
  - Call preflight before `insert_analysis_run`.
  - Pass preflight into `ReportRunInput`.
  - Emit preflight summary before loading full corpus.
  - Add tests for rejection before run insertion if practical in current harness.
- Modify docs:
  - `docs/backlog.md`
  - `docs/project.md`
  - `docs/design-document.md`
  - `docs/architecture-deep-dive.md`

---

### Task 1: Add Pure Preflight Estimation Helpers

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`

- [ ] **Step 1: Add failing tests for estimation helpers**

Append tests in `#[cfg(test)] mod tests` in `src-tauri/src/analysis/corpus.rs`:

```rust
use super::{
    estimate_message_input_chars, estimate_preflight_chunk_count, AnalysisRunPreflightLimits,
};

#[test]
fn estimated_message_chars_match_report_chunk_accounting() {
    let message = CorpusMessage {
        item_id: 11,
        source_id: 2,
        external_id: "100".to_string(),
        published_at: 1_710_000_000,
        author: Some("Alice".to_string()),
        content: "First live document".to_string(),
        r#ref: "s2-i11".to_string(),
    };

    assert_eq!(
        estimate_message_input_chars(&message.content, &message.r#ref, message.author.as_deref()),
        message.content.len() + message.r#ref.len() + "Alice".len() + 64
    );
}

#[test]
fn estimated_chunk_count_matches_chunk_boundary_behavior() {
    assert_eq!(estimate_preflight_chunk_count(&[], 16_000), 0);
    assert_eq!(estimate_preflight_chunk_count(&[8_000, 7_000], 16_000), 1);
    assert_eq!(estimate_preflight_chunk_count(&[8_000, 9_000], 16_000), 2);
    assert_eq!(estimate_preflight_chunk_count(&[20_000], 16_000), 1);
}

#[test]
fn default_preflight_limits_are_conservative() {
    let limits = AnalysisRunPreflightLimits::default();

    assert_eq!(limits.max_messages_per_run, 10_000);
    assert_eq!(limits.max_chunks_per_run, 80);
    assert_eq!(limits.max_estimated_input_chars_per_run, 1_500_000);
    assert_eq!(limits.max_background_requests_per_run, 80);
}
```

- [ ] **Step 2: Run the focused tests and verify RED**

Run:

```powershell
cargo test analysis::corpus::tests::estimated_message_chars_match_report_chunk_accounting analysis::corpus::tests::estimated_chunk_count_matches_chunk_boundary_behavior analysis::corpus::tests::default_preflight_limits_are_conservative
```

Expected before implementation: compile failure for missing `estimate_message_input_chars`, `estimate_preflight_chunk_count`, or `AnalysisRunPreflightLimits`.

- [ ] **Step 3: Add helper structs and functions**

Add near the top of `src-tauri/src/analysis/corpus.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisRunPreflightLimits {
    pub max_messages_per_run: usize,
    pub max_chunks_per_run: usize,
    pub max_estimated_input_chars_per_run: usize,
    /// Reserved for future retry-aware budgeting. Currently equals
    /// `max_chunks_per_run` because each chunk creates exactly one
    /// background request. Not checked in `preflight_limit_error` until
    /// the values diverge.
    pub max_background_requests_per_run: usize,
}

impl Default for AnalysisRunPreflightLimits {
    fn default() -> Self {
        Self {
            max_messages_per_run: 10_000,
            max_chunks_per_run: 80,
            max_estimated_input_chars_per_run: 1_500_000,
            max_background_requests_per_run: 80,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisRunPreflight {
    pub source_ids: Vec<i64>,
    pub message_count: usize,
    pub estimated_input_chars: usize,
    pub estimated_chunks: usize,
    pub limits: AnalysisRunPreflightLimits,
}

pub(crate) fn estimate_message_input_chars(
    content: &str,
    r#ref: &str,
    author: Option<&str>,
) -> usize {
    content.len() + r#ref.len() + author.unwrap_or("").len() + 64
}

pub(crate) fn live_corpus_ref(source_id: i64, item_id: i64) -> String {
    format!("s{source_id}-i{item_id}")
}

pub(crate) fn estimate_preflight_chunk_count(message_sizes: &[usize], max_chars: usize) -> usize {
    let mut chunks = 0usize;
    let mut current_chars = 0usize;

    for size in message_sizes {
        if current_chars > 0 && current_chars + size > max_chars {
            chunks += 1;
            current_chars = 0;
        }
        current_chars += size;
    }

    if current_chars > 0 {
        chunks += 1;
    }

    chunks
}
```

- [ ] **Step 4: Verify helper tests pass**

Run:

```powershell
cargo test analysis::corpus::tests::estimated_message_chars_match_report_chunk_accounting analysis::corpus::tests::estimated_chunk_count_matches_chunk_boundary_behavior analysis::corpus::tests::default_preflight_limits_are_conservative
```

Expected: selected tests pass.

- [ ] **Step 5: Commit**

Run:

```powershell
git add src-tauri/src/analysis/corpus.rs
git commit -m "feat(analysis): add report preflight policy types"
```

---

### Task 2: Add Database-Backed Preflight

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`

- [ ] **Step 1: Add failing tests for preflight query behavior and ref consistency**

Add tests in `src-tauri/src/analysis/corpus.rs` using the existing `snapshot_pool()` helper:

```rust
#[tokio::test]
async fn preflight_counts_eligible_text_messages_for_sources() {
    let pool = snapshot_pool().await;
    let first_content = compress_text("First live document").expect("compress first");
    let second_content = compress_text("Second live document").expect("compress second");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .bind(first_content)
    .execute(&pool)
    .await
    .expect("insert first item");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(12_i64)
    .bind(4_i64)
    .bind("101")
    .bind(Option::<String>::None)
    .bind(1_710_000_100_i64)
    .bind(second_content)
    .execute(&pool)
    .await
    .expect("insert second item");

    let preflight = preflight_analysis_run(
        &pool,
        &[2, 4],
        1_700_000_000_i64,
        1_800_000_000_i64,
        16_000,
        AnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");

    assert_eq!(preflight.source_ids, vec![2, 4]);
    assert_eq!(preflight.message_count, 2);
    assert_eq!(preflight.estimated_chunks, 1);
    assert!(preflight.estimated_input_chars > 0);
}

#[tokio::test]
async fn preflight_ref_format_matches_corpus_loader_ref_format() {
    let pool = snapshot_pool().await;
    let content = compress_text("Test message").expect("compress");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind(Option::<String>::None)
    .bind(1_710_000_000_i64)
    .bind(content)
    .execute(&pool)
    .await
    .expect("insert item");

    let corpus = load_corpus_messages(&pool, &[2], 1_700_000_000_i64, 1_800_000_000_i64)
        .await
        .expect("load corpus");

    assert_eq!(corpus[0].r#ref, live_corpus_ref(corpus[0].source_id, corpus[0].item_id));
}

#[tokio::test]
async fn preflight_ignores_media_only_items_without_text_content() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, NULL)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .execute(&pool)
    .await
    .expect("insert media-only item");

    let preflight = preflight_analysis_run(
        &pool,
        &[2],
        1_700_000_000_i64,
        1_800_000_000_i64,
        16_000,
        AnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");

    assert_eq!(preflight.message_count, 0);
    assert_eq!(preflight.estimated_chunks, 0);
    assert_eq!(preflight.estimated_input_chars, 0);
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```powershell
cargo test analysis::corpus::tests::preflight_counts_eligible_text_messages_for_sources analysis::corpus::tests::preflight_ref_format_matches_corpus_loader_ref_format analysis::corpus::tests::preflight_ignores_media_only_items_without_text_content
```

Expected before implementation: compile failure for missing `preflight_analysis_run` or `live_corpus_ref`.

- [ ] **Step 3: Implement preflight query**

First update `load_corpus_messages` to use the shared ref helper:

```rust
r#ref: live_corpus_ref(row.source_id, row.id),
```

Then add to `src-tauri/src/analysis/corpus.rs` near `load_corpus_messages`:

```rust
pub(crate) async fn preflight_analysis_run(
    pool: &Pool<Sqlite>,
    source_ids: &[i64],
    period_from: i64,
    period_to: i64,
    chunk_target_chars: usize,
    limits: AnalysisRunPreflightLimits,
) -> Result<AnalysisRunPreflight, String> {
    if source_ids.is_empty() {
        return Ok(AnalysisRunPreflight {
            source_ids: Vec::new(),
            message_count: 0,
            estimated_input_chars: 0,
            estimated_chunks: 0,
            limits,
        });
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT id, source_id, author, content_zstd FROM items WHERE content_zstd IS NOT NULL AND published_at >= ",
    );
    query.push_bind(period_from);
    query.push(" AND published_at <= ");
    query.push_bind(period_to);
    query.push(" AND source_id IN (");

    {
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id);
        }
    }

    query.push(") ORDER BY published_at ASC, id ASC");

    #[derive(sqlx::FromRow)]
    struct PreflightRow {
        id: i64,
        source_id: i64,
        author: Option<String>,
        content_zstd: Option<Vec<u8>>,
    }

    let rows: Vec<PreflightRow> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut message_sizes = Vec::with_capacity(rows.len());
    let mut estimated_input_chars = 0usize;
    for row in rows {
        let content = decompress_text(
            row.content_zstd
                .as_deref()
                .ok_or_else(|| format!("Item {} is missing content", row.id))?,
        )?;
        let r#ref = live_corpus_ref(row.source_id, row.id);
        let size = estimate_message_input_chars(&content, &r#ref, row.author.as_deref());
        estimated_input_chars += size;
        message_sizes.push(size);
    }

    let estimated_chunks = estimate_preflight_chunk_count(&message_sizes, chunk_target_chars);

    Ok(AnalysisRunPreflight {
        source_ids: source_ids.to_vec(),
        message_count: message_sizes.len(),
        estimated_input_chars,
        estimated_chunks,
        limits,
    })
}
```

- [ ] **Step 4: Verify preflight tests pass**

Run:

```powershell
cargo test analysis::corpus::tests::preflight_counts_eligible_text_messages_for_sources analysis::corpus::tests::preflight_ref_format_matches_corpus_loader_ref_format analysis::corpus::tests::preflight_ignores_media_only_items_without_text_content
```

Expected: selected tests pass.

- [ ] **Step 5: Commit**

Run:

```powershell
git add src-tauri/src/analysis/corpus.rs
git commit -m "feat(analysis): preflight report corpus size"
```

---

### Task 3: Enforce Hard Caps Before Run Insertion

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/report.rs`

- [ ] **Step 1: Add pure violation tests**

In `src-tauri/src/analysis/corpus.rs`, add tests:

```rust
use super::{preflight_limit_error, AnalysisRunPreflight};

#[test]
fn preflight_limit_error_reports_all_scale_dimensions() {
    let preflight = AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 73_102,
        estimated_input_chars: 6_200_000,
        estimated_chunks: 381,
        limits: AnalysisRunPreflightLimits::default(),
    };

    let error = preflight_limit_error(&preflight).expect("limit error");

    assert!(error.contains("73102 documents"));
    assert!(error.contains("381 estimated chunks"));
    assert!(error.contains("6200000 estimated input characters"));
    assert!(error.contains("Narrow the period or choose a smaller source scope"));
}

#[test]
fn preflight_limit_error_allows_runs_within_limits() {
    let preflight = AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 1_000,
        estimated_input_chars: 100_000,
        estimated_chunks: 10,
        limits: AnalysisRunPreflightLimits::default(),
    };

    assert_eq!(preflight_limit_error(&preflight), None);
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```powershell
cargo test analysis::corpus::tests::preflight_limit_error_reports_all_scale_dimensions analysis::corpus::tests::preflight_limit_error_allows_runs_within_limits
```

Expected before implementation: compile failure for missing `preflight_limit_error`.

- [ ] **Step 3: Implement violation helper**

Add to `src-tauri/src/analysis/corpus.rs`:

```rust
pub(crate) fn preflight_limit_error(preflight: &AnalysisRunPreflight) -> Option<String> {
    let exceeds_messages = preflight.message_count > preflight.limits.max_messages_per_run;
    let exceeds_chunks = preflight.estimated_chunks > preflight.limits.max_chunks_per_run;
    let exceeds_chars =
        preflight.estimated_input_chars > preflight.limits.max_estimated_input_chars_per_run;

    if !(exceeds_messages || exceeds_chunks || exceeds_chars) {
        return None;
    }

    Some(format!(
        "Analysis scope is too large: {} documents, {} estimated chunks, \
         {} estimated input characters. \
         Narrow the period or choose a smaller source scope.",
        preflight.message_count, preflight.estimated_chunks, preflight.estimated_input_chars
    ))
}
```

- [ ] **Step 4: Wire preflight into `start_analysis_report`**

In `src-tauri/src/analysis/report.rs`, change the import:

```rust
use super::corpus::{load_corpus_messages, preflight_analysis_run, preflight_limit_error, AnalysisRunPreflight, AnalysisRunPreflightLimits};
```

Extend `ReportRunInput`:

```rust
struct ReportRunInput {
    run_id: i64,
    scope_label: String,
    source_ids: Vec<i64>,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template: AnalysisPromptTemplate,
    model_override: Option<String>,
    profile_id: Option<String>,
    preflight: AnalysisRunPreflight,
}
```

In `start_analysis_report`, after `source_ids` is resolved and before `find_active_duplicate_run`, add:

```rust
let preflight = preflight_analysis_run(
    &pool,
    &source_ids,
    period_from,
    period_to,
    ANALYSIS_CHUNK_TARGET_CHARS,
    AnalysisRunPreflightLimits::default(),
)
.await
.map_err(AppError::database)?;

if preflight.message_count == 0 {
    return Err(AppError::validation(
        "No synced source documents were found for the selected analysis scope and period",
    ));
}

if let Some(error) = preflight_limit_error(&preflight) {
    return Err(AppError::validation(error));
}
```

Pass `preflight` into `ReportRunInput`.

- [ ] **Step 5: Emit preflight summary before loading full corpus**

In `run_report_pipeline`, replace the existing `load_items` started message with:

```rust
RunEvent::new(run_id, "started", "load_items")
    .message(format!(
        "Preflight passed: {} documents, {} estimated chunks, {} estimated input characters.",
        input.preflight.message_count,
        input.preflight.estimated_chunks,
        input.preflight.estimated_input_chars
    ))
    .emit(&handle);
```

Keep the subsequent full corpus load. Keep the existing empty-corpus guard as defense in depth.

- [ ] **Step 6: Verify focused backend tests**

Run:

```powershell
cargo test analysis::corpus::
cargo test analysis::report::
```

Expected: all selected tests pass.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/report.rs
git commit -m "feat(analysis): enforce report preflight limits"
```

---

### Task 4: Add Integration-Level Report Start Coverage

**Files:**
- Modify: `src-tauri/src/analysis/report.rs`

- [ ] **Step 1: Add test-only helper for preflight decision**

If direct Tauri command integration is too heavy, add a small pure helper in `report.rs`:

```rust
fn validate_report_preflight(preflight: &AnalysisRunPreflight) -> AppResult<()> {
    if preflight.message_count == 0 {
        return Err(AppError::validation(
            "No synced source documents were found for the selected analysis scope and period",
        ));
    }

    if let Some(error) = preflight_limit_error(preflight) {
        return Err(AppError::validation(error));
    }

    Ok(())
}
```

Update `start_analysis_report` to call `validate_report_preflight(&preflight)?;`.

- [ ] **Step 2: Add tests for `validate_report_preflight`**

In `report.rs` tests, import:

```rust
use super::validate_report_preflight;
use crate::analysis::corpus::{AnalysisRunPreflight, AnalysisRunPreflightLimits};
use crate::error::AppErrorKind;
```

Add:

```rust
#[test]
fn validate_report_preflight_rejects_empty_corpus() {
    let error = validate_report_preflight(&AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 0,
        estimated_input_chars: 0,
        estimated_chunks: 0,
        limits: AnalysisRunPreflightLimits::default(),
    })
    .expect_err("empty corpus should fail");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(
        error.message,
        "No synced source documents were found for the selected analysis scope and period"
    );
}

#[test]
fn validate_report_preflight_rejects_oversized_runs() {
    let error = validate_report_preflight(&AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 10_001,
        estimated_input_chars: 100_000,
        estimated_chunks: 10,
        limits: AnalysisRunPreflightLimits::default(),
    })
    .expect_err("oversized corpus should fail");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert!(error.message.contains("Analysis scope is too large"));
}

#[test]
fn validate_report_preflight_allows_runs_within_limits() {
    validate_report_preflight(&AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 100,
        estimated_input_chars: 50_000,
        estimated_chunks: 4,
        limits: AnalysisRunPreflightLimits::default(),
    })
    .expect("preflight should pass");
}
```

- [ ] **Step 3: Run focused report tests**

Run:

```powershell
cargo test analysis::report::tests::validate_report_preflight_
```

Expected: tests pass.

- [ ] **Step 4: Commit**

Run:

```powershell
git add src-tauri/src/analysis/report.rs
git commit -m "test(analysis): cover report preflight validation"
```

---

### Task 5: Update Documentation

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/project.md`
- Modify: `docs/design-document.md`
- Modify: `docs/architecture-deep-dive.md`

- [ ] **Step 1: Update backlog Phase 4.3**

In `docs/backlog.md`, update Phase 4.3 so it states:

```markdown
Scope:

- [x] decide whether per-provider and per-profile concurrency limits need explicit configuration beyond the current shared default
- [x] cap analysis report runs before chunk workers are spawned

Acceptance criteria:

- [x] concurrency limits are intentional and documented
- [x] request scheduling remains predictable under mixed interactive and background load
- [x] oversized analysis scopes fail fast with a validation error before creating an analysis run

Current notes:

- LLM scheduler concurrency is intentionally `2` running requests per `(provider, profile)`.
- Interactive requests jump ahead of background requests in the same scheduler key.
- Analysis report runs are capped by backend preflight limits: `10_000` messages, `80` chunks, `1_500_000` estimated input characters, and `80` background requests per run.
```

- [ ] **Step 2: Update current-state docs**

Add concise notes to `docs/project.md`, `docs/design-document.md`, and `docs/architecture-deep-dive.md`:

```markdown
LLM scheduling allows two running requests per `(provider, profile)` and prioritizes interactive requests over background work. Analysis report runs run a backend preflight before run creation and are capped at `10_000` messages, `80` estimated chunks, `1_500_000` estimated input characters, and `80` background requests.
```

- [ ] **Step 3: Scan for stale concurrency wording**

Run:

```powershell
rg -n "LLM concurrency|concurrency limits|shared default|limit policy|preflight|large Telegram archives" README.md docs src-tauri/src
```

Expected: no current-state doc still says the concurrency policy is undecided.

- [ ] **Step 4: Commit**

Run:

```powershell
git add docs/backlog.md docs/project.md docs/design-document.md docs/architecture-deep-dive.md
git commit -m "docs(llm): document report preflight limits"
```

---

### Task 6: Final Verification

**Files:**
- Verify all changed files.

- [ ] **Step 1: Run focused Rust tests**

Run:

```powershell
cargo test analysis::corpus::
cargo test analysis::report::
cargo test llm::scheduler::
```

Expected: all pass.

- [ ] **Step 2: Run full backend tests**

Run:

```powershell
cargo test
```

Expected: all pass.

- [ ] **Step 3: Run frontend checks**

Run:

```powershell
npm.cmd test
npm.cmd run check
```

Expected: Vitest passes and Svelte check reports `0 errors`.

- [ ] **Step 4: Run formatting and diff checks**

Run:

```powershell
cargo fmt --check
git diff --check
```

Expected: both pass.

- [ ] **Step 5: Inspect final state**

Run:

```powershell
git status --short
git log --oneline -8
```

Expected: working tree is clean after commits.
