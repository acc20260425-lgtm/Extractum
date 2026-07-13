# Prompt Pack Run Control Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the in-memory Prompt Pack active-run registry and cooperative cancellation helper from `runtime.rs` into a focused private `run_control.rs` module without changing behavior or public paths.

**Architecture:** Add private `prompt_packs::run_control` containing public `PromptPackRunState` and sibling-visible `run_with_prompt_pack_run_cancellation`. Preserve `prompt_packs::runtime::PromptPackRunState` through a runtime re-export and preserve the existing root `prompt_packs::PromptPackRunState` export unchanged; keep commands, event emission, SQL cleanup, provider/browser orchestration, and dev fixtures in `runtime.rs`.

**Tech Stack:** Rust 2021, Tauri 2, Tokio, tokio-util CancellationToken, TypeScript, Vitest raw-source contracts.

## Global Constraints

- Implement the approved design in `docs/superpowers/specs/2026-07-13-prompt-pack-run-control-extraction-design.md`.
- Modify only `src-tauri/src/prompt_packs/mod.rs`, `src-tauri/src/prompt_packs/runtime.rs`, new `src-tauri/src/prompt_packs/run_control.rs`, and new `src/lib/prompt-pack-run-control-contract.test.ts`.
- Register exactly private `mod run_control;`; do not expose or re-export the module itself.
- Preserve both public paths `prompt_packs::runtime::PromptPackRunState` and `prompt_packs::PromptPackRunState`.
- Move `PromptPackRunState`, its complete implementation, and `run_with_prompt_pack_run_cancellation` without changing bodies, signatures, lock structure, token relationships, terminal-kind handling, or error behavior.
- Keep `PromptPackRunState` public; give only `run_with_prompt_pack_run_cancellation` `pub(super)` visibility; keep `ensure_cancellation_token` private.
- Keep Tauri commands, provider/browser execution, browser cancellation commands, event construction/emission, interrupted-run SQL cleanup, dev smoke fixtures, `PROMPT_PACK_RUN_EVENT`, the fixture label, `emit_prompt_pack_run_event`, and `now_string` in `runtime.rs`.
- Do not combine the two mutexes, introduce atomics, timeouts, cleanup-on-drop, new logging, new errors, or new cancellation semantics.
- Keep all existing behavioral tests in `runtime::tests`; do not move or edit their bodies or assertions.
- Do not add dependencies, migrations, DTO changes, persistence changes, frontend behavior, or registered/persisted values.
- Do not modify `docs/project.md` or `docs/value-registry.md` because behavior and registered values do not change.
- Preserve unrelated user changes and require a clean worktree before starting.

---

### Task 1: Extract Prompt Pack Run Control

**Files:**
- Create: `src-tauri/src/prompt_packs/run_control.rs`
- Create: `src/lib/prompt-pack-run-control-contract.test.ts`
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Test: `src-tauri/src/prompt_packs/runtime.rs` (`mod tests` remains in place)

**Interfaces:**
- Consumes: `std::collections::{HashMap, HashSet}`, `std::future::Future`, `tokio::sync::Mutex`, `tokio_util::sync::CancellationToken`, `super::dto::PromptPackRunEvent`, `crate::error::AppResult`, and `crate::llm::LlmRequestError`.
- Produces: private module `prompt_packs::run_control` with `pub struct PromptPackRunState` and `pub(super) async fn run_with_prompt_pack_run_cancellation<Fut, T>(Option<CancellationToken>, Fut) -> Result<T, LlmRequestError>` where `Fut: Future<Output = Result<T, LlmRequestError>>`.
- Preserves: `prompt_packs::runtime::PromptPackRunState`, root `prompt_packs::PromptPackRunState`, all existing public methods on the state, Tauri managed-state registration, and every existing runtime test body.

- [ ] **Step 1: Verify clean-tree, approved-spec, and formatting preconditions**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git merge-base --is-ancestor f7a843f8 HEAD
$approvedSpecPresent = $LASTEXITCODE -eq 0
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_PRESENT=$approvedSpecPresent"
if ($status.Count -ne 0 -or -not $approvedSpecPresent) { exit 1 }
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: `STATUS_COUNT=0`, `APPROVED_SPEC_PRESENT=True`, and rustfmt exits 0. The clean formatting baseline guarantees that the later formatter cannot introduce unrelated Rust changes.

- [ ] **Step 2: Add the failing source-ownership and compatibility contract**

Create `src/lib/prompt-pack-run-control-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import promptPacksModuleSource from "../../src-tauri/src/prompt_packs/mod.rs?raw";
import runControlSource from "../../src-tauri/src/prompt_packs/run_control.rs?raw";
import runtimeSource from "../../src-tauri/src/prompt_packs/runtime.rs?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

describe("Prompt Pack run control ownership", () => {
  it("registers a private run_control sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod run_control;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod run_control;/);
  });

  it("moves the state and cancellation helper out of runtime", () => {
    const control = normalized(runControlSource);
    const runtime = normalized(runtimeSource);

    expect(control).toMatch(/^pub struct PromptPackRunState\s*\{/m);
    expect(control).toMatch(
      /^pub\(super\) async fn run_with_prompt_pack_run_cancellation<Fut, T>\s*\(/m,
    );
    expect(runtime).not.toMatch(/^pub struct PromptPackRunState\s*\{/m);
    expect(runtime).not.toMatch(
      /^(?:pub(?:\([^)]*\))?\s+)?async fn run_with_prompt_pack_run_cancellation<Fut, T>\s*\(/m,
    );
  });

  it("preserves both public PromptPackRunState paths", () => {
    const moduleSource = normalized(promptPacksModuleSource);
    const runtime = normalized(runtimeSource);

    expect(runtime).toMatch(
      /^pub use super::run_control::PromptPackRunState;$/m,
    );
    expect(moduleSource).toMatch(
      /pub use runtime::\{[\s\S]*?\bPromptPackRunState,\s*\n\};/,
    );
  });

  it("keeps the exact terminal event cleanup set", () => {
    const control = normalized(runControlSource);

    expect(control).toMatch(
      /"completed"\s*\|\s*"partial"\s*\|\s*"failed"\s*\|\s*"cancelled"\s*\|\s*"interrupted"/,
    );
  });

  it("keeps run control independent from runtime infrastructure", () => {
    const control = normalized(runControlSource);

    expect(control).not.toMatch(/\btauri\b/);
    expect(control).not.toMatch(/\bsqlx\b/);
    expect(control).not.toMatch(/\bAppHandle\b/);
    expect(control).not.toMatch(/\bEmitter\b/);
    expect(control).not.toMatch(/\bget_pool\b/);
    expect(control).not.toMatch(/\brun_store\b/);
    expect(control).not.toMatch(/\bstage_request_policy\b/);
  });
});
```

Expected: the contract normalizes CRLF, checks the exact ownership and public compatibility paths, freezes the existing terminal cleanup set, and forbids runtime infrastructure dependencies.

- [ ] **Step 3: Run the source contract to verify RED**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-run-control-contract.test.ts
```

Expected: FAIL during Vite module resolution because `src-tauri/src/prompt_packs/run_control.rs` does not exist. This is the intended RED, not a Vitest infrastructure failure.

- [ ] **Step 4: Register the private sibling module**

In `src-tauri/src/prompt_packs/mod.rs`, insert this line in alphabetical order immediately before the existing `mod run_store;`:

```rust
mod run_control;
```

Do not add `pub`, `pub(crate)`, or a module re-export. Keep the existing `pub use runtime::{ ... PromptPackRunState, ... };` block unchanged.

- [ ] **Step 5: Create the complete run-control module**

Create `src-tauri/src/prompt_packs/run_control.rs` with exactly:

```rust
use std::collections::{HashMap, HashSet};
use std::future::Future;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::dto::PromptPackRunEvent;
use crate::error::AppResult;
use crate::llm::LlmRequestError;

#[derive(Default)]
pub struct PromptPackRunState {
    active: Mutex<HashSet<i64>>,
    cancellation_tokens: Mutex<HashMap<i64, CancellationToken>>,
}

impl PromptPackRunState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn track(&self, run_id: i64) -> AppResult<()> {
        self.active.lock().await.insert(run_id);
        self.ensure_cancellation_token(run_id).await;
        Ok(())
    }

    pub async fn track_if_absent(&self, run_id: i64) -> AppResult<bool> {
        let inserted = self.active.lock().await.insert(run_id);
        self.ensure_cancellation_token(run_id).await;
        Ok(inserted)
    }

    pub async fn request_cancel(&self, run_id: i64) -> AppResult<()> {
        self.ensure_cancellation_token(run_id).await.cancel();
        Ok(())
    }

    pub async fn child_token(&self, run_id: i64) -> Option<CancellationToken> {
        self.cancellation_tokens
            .lock()
            .await
            .get(&run_id)
            .map(CancellationToken::child_token)
    }

    pub async fn finish(&self, run_id: i64) {
        self.active.lock().await.remove(&run_id);
        self.cancellation_tokens.lock().await.remove(&run_id);
    }

    pub async fn active_run_ids(&self) -> Vec<i64> {
        let mut ids = self.active.lock().await.iter().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    pub async fn apply_event(&self, event: PromptPackRunEvent) {
        if matches!(
            event.kind.as_str(),
            "completed" | "partial" | "failed" | "cancelled" | "interrupted"
        ) {
            self.finish(event.run_id).await;
        }
    }

    async fn ensure_cancellation_token(&self, run_id: i64) -> CancellationToken {
        self.cancellation_tokens
            .lock()
            .await
            .entry(run_id)
            .or_insert_with(CancellationToken::new)
            .clone()
    }
}

pub(super) async fn run_with_prompt_pack_run_cancellation<Fut, T>(
    run_cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>,
{
    let Some(run_cancellation_token) = run_cancellation_token else {
        return future.await;
    };

    if run_cancellation_token.is_cancelled() {
        return Err(LlmRequestError::Cancelled);
    }

    tokio::select! {
        result = future => result,
        _ = run_cancellation_token.cancelled() => Err(LlmRequestError::Cancelled),
    }
}
```

Expected: the state and helper bodies are copied unchanged from `runtime.rs`; only the helper receives the approved `pub(super)` visibility.

- [ ] **Step 6: Preserve runtime paths and remove the old definitions**

In `runtime.rs`, delete these imports because they are fully owned by `run_control.rs` after the move:

```rust
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;
```

Keep these imports in `runtime.rs` because other runtime-owned generic browser and stage functions still use them:

```rust
use std::future::Future;
use tokio_util::sync::CancellationToken;
```

Immediately before the existing `use super::run_store::{ ... };` block, add:

```rust
use super::run_control::run_with_prompt_pack_run_cancellation;
pub use super::run_control::PromptPackRunState;
```

Delete from `runtime.rs` exactly:

```text
#[derive(Default)] pub struct PromptPackRunState
the complete impl PromptPackRunState block
async fn run_with_prompt_pack_run_cancellation<Fut, T>
```

Do not edit any state call site, command signature, event-emission path,
cleanup function, fixture function, test body, or test assertion. In
particular, leave these runtime-owned functions unchanged:

```text
emit_prompt_pack_run_event
cleanup_interrupted_prompt_pack_runs_in_pool
cleanup_interrupted_prompt_pack_runs
seed_prompt_pack_cancellation_smoke_fixture
clear_prompt_pack_cancellation_smoke_fixture
seed_prompt_pack_cancellation_smoke_fixture_in_pool
clear_prompt_pack_cancellation_smoke_fixture_in_pool
prompt_pack_cancellation_smoke_fixture_run_ids
```

The existing test-module `use super::{ ... PromptPackRunState,
run_with_prompt_pack_run_cancellation, ... };` remains valid through the two
runtime imports above and therefore does not need to change.

- [ ] **Step 7: Format and run the source contract for GREEN**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
npm.cmd run test -- src/lib/prompt-pack-run-control-contract.test.ts
```

Expected: Vitest runs 5 tests, all pass. Formatting changes only the three Rust files permitted by scope.

- [ ] **Step 8: Run the focused state and cancellation behavior tests**

Run:

```powershell
$tests = @(
    'prompt_packs::runtime::tests::prompt_pack_run_state_tracks_active_and_cancel_requested_runs',
    'prompt_packs::runtime::tests::prompt_pack_run_state_cancels_child_tokens',
    'prompt_packs::runtime::tests::terminal_event_removes_run_from_active_state',
    'prompt_packs::runtime::tests::prompt_pack_run_cancellation_allows_completed_stage_future',
    'prompt_packs::runtime::tests::prompt_pack_run_cancellation_interrupts_stage_future',
    'prompt_packs::runtime::tests::prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job',
    'prompt_packs::runtime::tests::prompt_pack_browser_stage_cancelled_while_active_stops_sidecar'
)
foreach ($test in $tests) {
    cargo test --manifest-path src-tauri/Cargo.toml --lib $test -- --exact
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Expected: every command runs exactly one test and passes. If a filter matches zero tests, correct the filter before proceeding; an empty run is not GREEN.

- [ ] **Step 9: Run the complete runtime and Prompt Pack Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::runtime::tests
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: both filtered suites exit 0 with no failed tests.

- [ ] **Step 10: Run the complete Vitest and Rust suites**

Run:

```powershell
npm.cmd run test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: both complete suites exit 0 with no failed frontend, contract, Rust unit, integration, or doc tests.

- [ ] **Step 11: Verify formatting and all Rust targets with zero warnings**

Run:

```powershell
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
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

Expected: rustfmt exits 0, `CARGO_EXIT=0`, and `WARNING_COUNT=0`. Running native Cargo through `cmd.exe` keeps redirected stderr as ordinary text under Windows PowerShell 5.1.

- [ ] **Step 12: Review exact scope and commit**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$changed = @(git status --porcelain=v1 --untracked-files=all | ForEach-Object {
    $_.Substring(3).Replace('\', '/')
})
$allowed = @(
    'src-tauri/src/prompt_packs/mod.rs',
    'src-tauri/src/prompt_packs/run_control.rs',
    'src-tauri/src/prompt_packs/runtime.rs',
    'src/lib/prompt-pack-run-control-contract.test.ts'
)
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
"CHANGED=$($changed -join ',')"
"UNEXPECTED=$($unexpected -join ',')"
if ($changed.Count -ne 4 -or $unexpected.Count -ne 0) { exit 1 }
git add -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/run_control.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src/lib/prompt-pack-run-control-contract.test.ts
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git diff --cached --stat
git diff --cached -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/run_control.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src/lib/prompt-pack-run-control-contract.test.ts
git commit -m "refactor: extract prompt pack run control"
git status --short --branch
```

Expected: the implementation commit contains exactly the private module registration, unchanged state/control bodies, runtime imports/re-export and removals, plus the focused ownership contract. The worktree is clean after commit.
