# Prompt Pack Run Store Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the Prompt Pack run catalog read/manage persistence API from `runtime.rs` into a focused private `run_store.rs` module without changing runtime behavior or public interfaces.

**Architecture:** Add private `prompt_packs::run_store` containing five `pub(super)` pool-level operations plus private label normalization and row-to-DTO mapping. Keep Tauri commands, lifecycle SQL, cancellation, events, smoke fixtures, and the shared runtime timestamp helper in `runtime.rs`; the extracted update operation calls `crate::time::now_rfc3339_utc()` directly.

**Tech Stack:** Rust 2021, Tauri 2, Tokio, SQLx 0.8.6, SQLite, TypeScript, Vitest raw-source contracts.

## Global Constraints

- Implement the approved design in `docs/superpowers/specs/2026-07-13-prompt-pack-run-store-extraction-design.md`.
- Modify only `src-tauri/src/prompt_packs/mod.rs`, `src-tauri/src/prompt_packs/runtime.rs`, new `src-tauri/src/prompt_packs/run_store.rs`, and new `src/lib/prompt-pack-run-store-contract.test.ts`.
- Register the module exactly as private `mod run_store;`; do not re-export it.
- Use `pub(super)` only for the five functions called by `runtime.rs`; keep `normalize_prompt_pack_run_label`, `RunSummaryRow`, and its conversion private.
- Preserve Tauri command names, parameters, exports, DTOs, SQL text, query ordering, limits, status checks, error messages, and label-normalization behavior.
- Keep all lifecycle/execution SQL in `runtime.rs`, including cancellation, stage provenance/status, failure marking, interrupted-run cleanup, and dev-fixture SQL.
- Do not add dependencies, migrations, transactions, retries, caching, logging, timestamp helpers, persisted values, or frontend behavior.
- Keep the existing storage behavior tests in `runtime::tests`; change only the imports needed to reach `run_store`.
- Do not modify `docs/project.md` or `docs/value-registry.md` because no behavior or registered value changes.
- Preserve unrelated user changes and require a clean worktree before starting.

---

### Task 1: Extract the Run Catalog Store

**Files:**
- Create: `src-tauri/src/prompt_packs/run_store.rs`
- Create: `src/lib/prompt-pack-run-store-contract.test.ts`
- Modify: `src-tauri/src/prompt_packs/mod.rs:1-15`
- Modify: `src-tauri/src/prompt_packs/runtime.rs:1-35, 1819-1855, 1993-2224, 2231-2250`
- Test: `src-tauri/src/prompt_packs/runtime.rs:2751-2958`

**Interfaces:**
- Consumes: `crate::error::{AppError, AppResult}`, `crate::time::now_rfc3339_utc`, `sqlx::SqlitePool`, and DTOs from `super::dto`.
- Produces: private module `prompt_packs::run_store` with these exact sibling-visible functions:
  - `pub(super) async fn list_prompt_pack_runs_in_pool(&SqlitePool, ListPromptPackRunsRequest) -> AppResult<Vec<PromptPackRunSummaryDto>>`
  - `pub(super) async fn update_prompt_pack_run_in_pool(&SqlitePool, i64, Option<String>) -> AppResult<PromptPackRunSummaryDto>`
  - `pub(super) async fn delete_prompt_pack_run_in_pool(&SqlitePool, i64) -> AppResult<()>`
  - `pub(super) async fn list_prompt_pack_run_stages_in_pool(&SqlitePool, i64) -> AppResult<Vec<PromptPackStageRunDto>>`
  - `pub(super) async fn load_run_summary_optional(&SqlitePool, i64) -> AppResult<Option<PromptPackRunSummaryDto>>`
- Preserves: all existing public Tauri commands and exports from `prompt_packs/mod.rs`.

- [ ] **Step 1: Verify clean-tree and approved-spec preconditions**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git merge-base --is-ancestor 3d3a24de HEAD
$specPresent = $LASTEXITCODE -eq 0
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_PRESENT=$specPresent"
if ($status.Count -ne 0 -or -not $specPresent) { exit 1 }
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: `STATUS_COUNT=0`, `APPROVED_SPEC_PRESENT=True`, and the existing
repository-wide Rust formatting baseline passes. This guarantees the later
`cargo fmt` cannot introduce unrelated formatting changes into a clean tree.

- [ ] **Step 2: Add the failing source-ownership contract**

Create `src/lib/prompt-pack-run-store-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import promptPacksModuleSource from "../../src-tauri/src/prompt_packs/mod.rs?raw";
import runStoreSource from "../../src-tauri/src/prompt_packs/run_store.rs?raw";
import runtimeSource from "../../src-tauri/src/prompt_packs/runtime.rs?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

const extractedFunctions = [
  "list_prompt_pack_runs_in_pool",
  "update_prompt_pack_run_in_pool",
  "delete_prompt_pack_run_in_pool",
  "list_prompt_pack_run_stages_in_pool",
  "load_run_summary_optional",
] as const;

describe("Prompt Pack run store ownership", () => {
  it("registers a private run_store sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod run_store;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod run_store;/);
  });

  it.each(extractedFunctions)("moves %s out of runtime", (functionName) => {
    const store = normalized(runStoreSource);
    const runtime = normalized(runtimeSource);
    const definition = new RegExp(
      `pub\\(super\\)\\s+async\\s+fn\\s+${functionName}\\s*\\(`,
    );
    const runtimeDefinition = new RegExp(
      `(?:pub\\(crate\\)\\s+|pub\\(super\\)\\s+)?async\\s+fn\\s+${functionName}\\s*\\(`,
    );

    expect(store).toMatch(definition);
    expect(runtime).not.toMatch(runtimeDefinition);
  });

  it("keeps row mapping private and avoids a reverse runtime dependency", () => {
    const store = normalized(runStoreSource);

    expect(store).toContain("struct RunSummaryRow {");
    expect(store).toContain("impl From<RunSummaryRow> for PromptPackRunSummaryDto");
    expect(store).toContain(".bind(crate::time::now_rfc3339_utc())");
    expect(store).not.toContain("super::runtime");
    expect(store).not.toMatch(/(?:pub|pub\([^)]*\))\s+struct RunSummaryRow/);
  });

  it("leaves lifecycle SQL in runtime", () => {
    const runtime = normalized(runtimeSource);

    expect(runtime).toContain("async fn mark_prompt_pack_run_failed(");
    expect(runtime).toContain("pub(crate) async fn cleanup_interrupted_prompt_pack_runs_in_pool(");
    expect(runtime).toContain("async fn seed_prompt_pack_cancellation_smoke_fixture_in_pool(");
  });
});
```

Expected: the test normalizes Windows CRLF, checks only the five enumerated functions, and explicitly permits lifecycle SQL to remain in `runtime.rs`.

- [ ] **Step 3: Run the contract to verify RED**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-run-store-contract.test.ts
```

Expected: FAIL during Vite module resolution because
`src-tauri/src/prompt_packs/run_store.rs` does not exist. This is the intended
RED, not an infrastructure failure.

- [ ] **Step 4: Register the private module**

In `src-tauri/src/prompt_packs/mod.rs`, add this line in alphabetical order,
between `pub mod result_commands;` and `pub mod runtime;`:

```rust
mod run_store;
```

Do not add `pub`, `pub(crate)`, or a re-export.

- [ ] **Step 5: Create the complete run store**

Create `src-tauri/src/prompt_packs/run_store.rs` with exactly:

```rust
use sqlx::SqlitePool;

use super::dto::{
    ListPromptPackRunsRequest, PromptPackRunSummaryDto, PromptPackStageRunDto,
};
use crate::error::{AppError, AppResult};

pub(super) async fn list_prompt_pack_runs_in_pool(
    pool: &SqlitePool,
    request: ListPromptPackRunsRequest,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let limit = request.limit.unwrap_or(20).clamp(1, 100);
    let rows = if let Some(project_id) = request.project_id {
        sqlx::query_as::<_, RunSummaryRow>(
            "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                    run_status, result_status, created_at, started_at, completed_at,
                    latest_message, progress_current, progress_total, queue_position
             FROM prompt_pack_runs
             WHERE project_id = ?
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?
    } else {
        sqlx::query_as::<_, RunSummaryRow>(
            "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                    run_status, result_status, created_at, started_at, completed_at,
                    latest_message, progress_current, progress_total, queue_position
             FROM prompt_pack_runs
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?
    };
    Ok(rows.into_iter().map(Into::into).collect())
}

pub(super) async fn update_prompt_pack_run_in_pool(
    pool: &SqlitePool,
    run_id: i64,
    run_label: Option<String>,
) -> AppResult<PromptPackRunSummaryDto> {
    let normalized_label = normalize_prompt_pack_run_label(run_label);
    let result = sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_label = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&normalized_label)
    .bind(crate::time::now_rfc3339_utc())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Prompt Pack run {run_id} not found"
        )));
    }

    load_run_summary_optional(pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Prompt Pack run {run_id} not found")))
}

pub(super) async fn delete_prompt_pack_run_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<()> {
    let status =
        sqlx::query_scalar::<_, String>("SELECT run_status FROM prompt_pack_runs WHERE id = ?")
            .bind(run_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?
            .ok_or_else(|| AppError::not_found(format!("Prompt Pack run {run_id} not found")))?;

    if status == "queued" || status == "running" {
        return Err(AppError::conflict(
            "Queued or running Prompt Pack runs cannot be deleted",
        ));
    }

    sqlx::query("DELETE FROM prompt_pack_runs WHERE id = ?")
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

fn normalize_prompt_pack_run_label(label: Option<String>) -> Option<String> {
    label
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) async fn list_prompt_pack_run_stages_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<PromptPackStageRunDto>> {
    sqlx::query_as::<
        _,
        (
            i64,
            i64,
            Option<i64>,
            String,
            i64,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT id, run_id, source_snapshot_id, stage_name, stage_order,
                stage_status, latest_message, browser_run_id, browser_run_status,
                browser_completion_reason, browser_provider_mode, browser_run_message
         FROM prompt_pack_stage_runs
         WHERE run_id = ?
         ORDER BY stage_order ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(
                    stage_run_id,
                    run_id,
                    source_snapshot_id,
                    stage_name,
                    stage_order,
                    stage_status,
                    latest_message,
                    browser_run_id,
                    browser_run_status,
                    browser_completion_reason,
                    browser_provider_mode,
                    browser_run_message,
                )| PromptPackStageRunDto {
                    stage_run_id,
                    run_id,
                    source_snapshot_id,
                    stage_name,
                    stage_order,
                    stage_status,
                    latest_message,
                    browser_run_id,
                    browser_run_status,
                    browser_completion_reason,
                    browser_provider_mode,
                    browser_run_message,
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

pub(super) async fn load_run_summary_optional(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Option<PromptPackRunSummaryDto>> {
    sqlx::query_as::<_, RunSummaryRow>(
        "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                run_status, result_status, created_at, started_at, completed_at,
                latest_message, progress_current, progress_total, queue_position
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(Into::into))
    .map_err(AppError::database)
}

#[derive(sqlx::FromRow)]
struct RunSummaryRow {
    id: i64,
    project_id: Option<i64>,
    run_label: Option<String>,
    runtime_provider: String,
    pack_id: String,
    pack_version: String,
    run_status: String,
    result_status: String,
    created_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    latest_message: Option<String>,
    progress_current: Option<i64>,
    progress_total: Option<i64>,
    queue_position: Option<i64>,
}

impl From<RunSummaryRow> for PromptPackRunSummaryDto {
    fn from(row: RunSummaryRow) -> Self {
        Self {
            run_id: row.id,
            project_id: row.project_id,
            run_label: row.run_label,
            runtime_provider: row.runtime_provider,
            pack_id: row.pack_id,
            pack_version: row.pack_version,
            run_status: row.run_status,
            result_status: row.result_status,
            created_at: row.created_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
            latest_message: row.latest_message,
            progress_current: row.progress_current,
            progress_total: row.progress_total,
            queue_position: row.queue_position,
        }
    }
}
```

Expected: the SQL and error strings are copied unchanged from `runtime.rs`.
The only syntactic behavior-preserving adjustment is replacing
`.bind(now_string())` with `.bind(crate::time::now_rfc3339_utc())`.

- [ ] **Step 6: Wire runtime to the new store and remove old definitions**

Near the existing DTO imports in `runtime.rs`, add:

```rust
use super::run_store::{
    delete_prompt_pack_run_in_pool, list_prompt_pack_run_stages_in_pool,
    list_prompt_pack_runs_in_pool, load_run_summary_optional, update_prompt_pack_run_in_pool,
};
```

Delete from `runtime.rs` only the complete definitions of:

```text
list_prompt_pack_runs_in_pool
update_prompt_pack_run_in_pool
delete_prompt_pack_run_in_pool
normalize_prompt_pack_run_label
list_prompt_pack_run_stages_in_pool
load_run_summary_optional
RunSummaryRow
impl From<RunSummaryRow> for PromptPackRunSummaryDto
```

Keep `emit_prompt_pack_run_event` and `now_string` in `runtime.rs`. Do not move
or edit any lifecycle, stage, cleanup, cancellation, or fixture query.

- [ ] **Step 7: Update only the runtime test imports**

In `runtime.rs`, remove these names from the existing `use super::{ ... };`
inside `mod tests`:

```rust
delete_prompt_pack_run_in_pool
list_prompt_pack_run_stages_in_pool
list_prompt_pack_runs_in_pool
update_prompt_pack_run_in_pool
```

Then add this import immediately after the `use super::{ ... };` block:

```rust
use super::super::run_store::{
    delete_prompt_pack_run_in_pool, list_prompt_pack_run_stages_in_pool,
    list_prompt_pack_runs_in_pool, update_prompt_pack_run_in_pool,
};
```

Do not move the tests or their fixtures. `load_run_summary_optional` is not
directly imported by the test module.

- [ ] **Step 8: Run formatting and the source contract for GREEN**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
npm.cmd run test -- src/lib/prompt-pack-run-store-contract.test.ts
```

Expected: Vitest runs 8 tests (one module-registration test, five parameterized
function-ownership cases, one row/timestamp test, and one lifecycle ownership
test), all pass. Formatting changes only the three Rust files in scope.

- [ ] **Step 9: Run focused storage behavior tests**

Run:

```powershell
$tests = @(
    'prompt_packs::runtime::tests::list_prompt_pack_runs_returns_recent_runs_for_project',
    'prompt_packs::runtime::tests::list_prompt_pack_run_stages_returns_browser_provenance',
    'prompt_packs::runtime::tests::update_prompt_pack_run_updates_user_label_only',
    'prompt_packs::runtime::tests::delete_prompt_pack_run_rejects_active_runs'
)
foreach ($test in $tests) {
    cargo test --manifest-path src-tauri/Cargo.toml --lib $test -- --exact
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Expected: each command runs exactly one test and passes. These tests preserve
list ordering/filtering, stage DTO mapping, whitespace label normalization,
unchanged run status during label updates, the active-run deletion guard, and
completed-run deletion.

- [ ] **Step 10: Run the complete runtime and Prompt Pack Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::runtime::tests
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
```

Expected: both commands exit 0 with no failed tests. The broader Prompt Pack
filter covers storage callers outside the focused runtime tests.

- [ ] **Step 11: Run the complete Vitest suite**

Run:

```powershell
npm.cmd run test
```

Expected: exit 0 with no failed frontend or raw-source contract tests.

- [ ] **Step 12: Run the complete Rust suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: exit 0 with no failed unit, integration, or doc tests.

- [ ] **Step 13: Verify formatting and all Rust targets with zero warnings**

Run:

```powershell
npm.cmd run check:rustfmt
$output = & cmd.exe /d /c "cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1"
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object {
    $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`'
}
"CARGO_EXIT=$cargoExit"
"WARNING_COUNT=$($warnings.Count)"
$warnings
if ($warnings.Count -ne 0) { exit 1 }
exit $cargoExit
```

Expected: rustfmt exits 0; `CARGO_EXIT=0`; `WARNING_COUNT=0`. Running native
Cargo through `cmd.exe` makes redirected stderr ordinary text even under
Windows PowerShell 5.1 and avoids `ErrorRecord` pipeline behavior.

- [ ] **Step 14: Review exact scope and commit**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$changed = @(git status --porcelain=v1 --untracked-files=all | ForEach-Object {
    $_.Substring(3).Replace('\', '/')
})
$allowed = @(
    'src-tauri/src/prompt_packs/mod.rs',
    'src-tauri/src/prompt_packs/run_store.rs',
    'src-tauri/src/prompt_packs/runtime.rs',
    'src/lib/prompt-pack-run-store-contract.test.ts'
)
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
"CHANGED=$($changed -join ',')"
"UNEXPECTED=$($unexpected -join ',')"
if ($changed.Count -ne 4 -or $unexpected.Count -ne 0) { exit 1 }
git add -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/run_store.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src/lib/prompt-pack-run-store-contract.test.ts
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git diff --cached -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/run_store.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src/lib/prompt-pack-run-store-contract.test.ts
git commit -m "refactor: extract prompt pack run store"
git status --short --branch
```

Expected: the implementation commit contains exactly the private module,
unchanged storage logic, runtime imports/removals, and the focused contract.
The worktree is clean after commit.
