# Prompt Pack Runtime Config Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract persisted Prompt Pack runtime-provider parsing and configuration loading from `runtime.rs` into one private `runtime_config.rs` module without changing SQL, provider values, decoded configuration, errors, provider resolution, or run behavior.

**Architecture:** Add a private `prompt_packs::runtime_config` sibling that owns `RunRuntimeProvider`, `RunRuntimeConfig`, and `load_run_runtime_config`. Keep LLM profile/model resolution, `RunCompletionRuntime` construction, stage dispatch, commands, and lifecycle orchestration in `runtime.rs`; retain existing database tests in `runtime::tests` and protect the ownership boundary with a CRLF-safe Vitest raw-source contract.

**Tech Stack:** Rust 2021, SQLx/SQLite, Serde JSON, Tauri 2, SvelteKit/TypeScript, Vitest raw-source contracts.

## Global Constraints

- Implement the approved design in `docs/superpowers/specs/2026-07-14-prompt-pack-runtime-config-extraction-design.md` at or after commit `ef17929a`.
- Modify exactly four implementation files across the complete slice: `src-tauri/src/prompt_packs/mod.rs`, `src-tauri/src/prompt_packs/runtime.rs`, new `src-tauri/src/prompt_packs/runtime_config.rs`, and new `src/lib/prompt-pack-runtime-config-contract.test.ts`.
- Register exactly private `mod runtime_config;`; do not expose or re-export the module itself.
- Move `RunRuntimeProvider`, its private `parse`, `RunRuntimeConfig`, and `load_run_runtime_config` without semantic rewriting.
- Give the enum, struct, struct fields, and loader `pub(super)` visibility only where required by sibling `runtime.rs`; keep `RunRuntimeProvider::parse` private.
- Preserve the SQL text, selected columns, bind order, `fetch_one`, provider strings `api` and `gemini_browser`, Browser Provider JSON decoding, error kinds, and error message construction.
- Keep LLM profile resolution, effective-model resolution, model input-limit resolution, `RunCompletionRuntime` construction, stage dispatch, commands, events, cleanup, fixtures, and lifecycle state in `runtime.rs`.
- Do not add dependencies, migrations, DTOs, traits, abstractions, mocks, logging, retries, fallbacks, defaults, or error wrapping.
- Keep all existing and new Rust test bodies and database fixtures in `runtime::tests`; after extraction, change only their imports as required.
- Do not modify `docs/project.md` or `docs/value-registry.md`; no public, persisted, wire, UI, or registered-value behavior changes.
- Preserve unrelated user changes and require a clean worktree before starting.

---

### Task 1: Characterize Runtime-Config Failure Behavior

**Files:**
- Modify: `src-tauri/src/prompt_packs/runtime.rs:1205-1250` in `runtime::tests`, beside `load_run_runtime_config_reads_api_and_browser_rows`

**Interfaces:**
- Consumes: existing private `load_run_runtime_config`, `RunRuntimeProvider`, `test_pool_with_prompt_pack_runs`, SQLx test pool, and `crate::error::AppErrorKind`.
- Produces: `load_run_runtime_config_rejects_unsupported_provider` and `load_run_runtime_config_rejects_malformed_browser_config` characterization tests.
- Preserves: all production code and the existing successful API/Browser-row test.

- [ ] **Step 1: Verify clean-tree, approved-spec, formatting, and ownership baselines**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git merge-base --is-ancestor ef17929a HEAD
$approvedSpecPresent = $LASTEXITCODE -eq 0
$runtime = Get-Content -Raw 'src-tauri/src/prompt_packs/runtime.rs'
$enumCount = [regex]::Matches($runtime, '(?m)^enum RunRuntimeProvider\s*\{').Count
$structCount = [regex]::Matches($runtime, '(?m)^struct RunRuntimeConfig\s*\{').Count
$loaderCount = [regex]::Matches(
    $runtime,
    '(?m)^async fn load_run_runtime_config\s*\('
).Count
$moduleExists = Test-Path 'src-tauri/src/prompt_packs/runtime_config.rs'
$contractExists = Test-Path 'src/lib/prompt-pack-runtime-config-contract.test.ts'
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_PRESENT=$approvedSpecPresent"
"ENUM_COUNT=$enumCount"
"STRUCT_COUNT=$structCount"
"LOADER_COUNT=$loaderCount"
"MODULE_EXISTS=$moduleExists"
"CONTRACT_EXISTS=$contractExists"
if (
    $status.Count -ne 0 -or
    -not $approvedSpecPresent -or
    $enumCount -ne 1 -or
    $structCount -ne 1 -or
    $loaderCount -ne 1 -or
    $moduleExists -or
    $contractExists
) { exit 1 }
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: clean tree; approved spec present; one enum, one struct, and one loader definition remain in `runtime.rs`; neither new file exists; rustfmt exits 0.

- [ ] **Step 2: Add the unsupported-provider characterization test**

Immediately after `load_run_runtime_config_reads_api_and_browser_rows`, add:

```rust
    #[tokio::test]
    async fn load_run_runtime_config_rejects_unsupported_provider() {
        let pool = test_pool_with_prompt_pack_runs([(
            103,
            None,
            "queued",
            "2026-06-21T00:00:00Z",
        )])
        .await;
        sqlx::query(
            "UPDATE prompt_pack_runs
             SET runtime_provider = 'unsupported'
             WHERE id = 103",
        )
        .execute(&pool)
        .await
        .expect("set unsupported runtime provider");

        let error = load_run_runtime_config(&pool, 103)
            .await
            .expect_err("unsupported provider");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert_eq!(
            error.message,
            "Unsupported prompt-pack runtime provider: unsupported"
        );
    }
```

Expected: the new test uses the existing loader and asserts the exact stable validation message.

- [ ] **Step 3: Add the malformed-Browser-config characterization test**

Immediately after the unsupported-provider test, add:

```rust
    #[tokio::test]
    async fn load_run_runtime_config_rejects_malformed_browser_config() {
        let pool = test_pool_with_prompt_pack_runs([(
            104,
            None,
            "queued",
            "2026-06-21T00:00:00Z",
        )])
        .await;
        sqlx::query(
            "UPDATE prompt_pack_runs
             SET runtime_provider = 'gemini_browser',
                 browser_provider_config_json = '{not-json'
             WHERE id = 104",
        )
        .execute(&pool)
        .await
        .expect("set malformed browser config");

        let error = load_run_runtime_config(&pool, 104)
            .await
            .expect_err("malformed browser config");

        assert_eq!(error.kind, crate::error::AppErrorKind::Internal);
        assert!(
            error
                .message
                .starts_with("parse Browser Provider config snapshot:"),
            "unexpected error message: {}",
            error.message
        );
    }
```

Expected: the test fixes the internal-error classification and stable message prefix without coupling to Serde's line/column wording.

- [ ] **Step 4: Run the three runtime-config characterization tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-prompt-pack-runtime-config --lib prompt_packs::runtime::tests::load_run_runtime_config_
```

Expected: PASS with exactly three matching tests: the existing happy path plus the two new error paths.

- [ ] **Step 5: Review and commit the characterization tests**

Run:

```powershell
$changed = @(git status --short --untracked-files=all)
$changed
if ($changed.Count -ne 1 -or $changed[0] -notmatch 'src-tauri/src/prompt_packs/runtime\.rs$') {
    exit 1
}
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git diff -- src-tauri/src/prompt_packs/runtime.rs
git add -- src-tauri/src/prompt_packs/runtime.rs
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git commit -m "test: characterize prompt pack runtime config errors"
```

Expected: the diff contains only the two tests; commit succeeds; worktree is clean afterward.

---

### Task 2: Extract the Private Runtime-Config Module

**Files:**
- Create: `src-tauri/src/prompt_packs/runtime_config.rs`
- Create: `src/lib/prompt-pack-runtime-config-contract.test.ts`
- Modify: `src-tauri/src/prompt_packs/mod.rs` near `pub mod runtime;`
- Modify: `src-tauri/src/prompt_packs/runtime.rs:1-35, 289-315, 417-469, 728-745`

**Interfaces:**
- Consumes: `sqlx::SqlitePool`, `crate::error::{AppError, AppResult}`, `crate::gemini_browser::GeminiBrowserProviderConfig`, and Serde JSON decoding.
- Produces: private `prompt_packs::runtime_config`; `pub(super) enum RunRuntimeProvider`; `pub(super) struct RunRuntimeConfig` with sibling-visible fields; `pub(super) async fn load_run_runtime_config(pool: &SqlitePool, run_id: i64) -> AppResult<RunRuntimeConfig>`.
- Preserves: private `RunRuntimeProvider::parse`; the exact SQL/config decoding; provider resolution and `RunCompletionRuntime` construction in `runtime.rs`; the three Task 1 test bodies.

- [ ] **Step 1: Add the failing CRLF-safe ownership contract**

Create `src/lib/prompt-pack-runtime-config-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import promptPacksModuleSource from "../../src-tauri/src/prompt_packs/mod.rs?raw";
import runtimeSource from "../../src-tauri/src/prompt_packs/runtime.rs?raw";
import runtimeConfigSource from "../../src-tauri/src/prompt_packs/runtime_config.rs?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");
const matches = (source: string, pattern: RegExp) => source.match(pattern) ?? [];
const productionPart = (source: string) =>
  normalized(source).split("\n#[cfg(test)]\nmod tests")[0];

describe("Prompt Pack runtime config ownership", () => {
  it("registers a private runtime_config sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod runtime_config;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod runtime_config;/);
  });

  it("moves provider parsing and loaded config out of runtime", () => {
    const runtime = productionPart(runtimeSource);
    const runtimeConfig = normalized(runtimeConfigSource);

    expect(runtimeConfig).toMatch(/^pub\(super\) enum RunRuntimeProvider\s*\{/m);
    expect(runtimeConfig).toMatch(/^pub\(super\) struct RunRuntimeConfig\s*\{/m);
    expect(runtimeConfig).toMatch(
      /^pub\(super\) async fn load_run_runtime_config\s*\(/m,
    );
    expect(runtimeConfig).toMatch(/^    fn parse\(value: &str\)/m);
    expect(runtimeConfig).not.toMatch(/^    pub(?:\([^)]*\))?\s+fn parse\(/m);
    for (const field of [
      "runtime_provider",
      "profile_id",
      "model_override",
      "browser_provider_config",
    ]) {
      expect(runtimeConfig).toMatch(
        new RegExp(`^\\s+pub\\(super\\) ${field}:`, "m"),
      );
    }

    expect(runtime).not.toMatch(/^enum RunRuntimeProvider\s*\{/m);
    expect(runtime).not.toMatch(/^struct RunRuntimeConfig\s*\{/m);
    expect(runtime).not.toMatch(/^async fn load_run_runtime_config\s*\(/m);
  });

  it("owns the persisted runtime-config query and decoding errors", () => {
    const runtime = productionPart(runtimeSource);
    const runtimeConfig = normalized(runtimeConfigSource);
    const selectMarker =
      "SELECT provider_profile_id, model, runtime_provider, browser_provider_config_json";

    expect(runtimeConfig).toContain(selectMarker);
    expect(runtimeConfig).toContain("serde_json::from_str");
    expect(runtimeConfig).toContain(
      "Unsupported prompt-pack runtime provider: {other}",
    );
    expect(runtimeConfig).toContain(
      "parse Browser Provider config snapshot: {error}",
    );
    expect(runtime).not.toContain(selectMarker);
  });

  it("keeps resolution and completion-runtime construction in runtime", () => {
    const runtime = productionPart(runtimeSource);

    expect(matches(runtime, /\bload_run_runtime_config\s*\(/g)).toHaveLength(1);
    expect(runtime).toContain("RunRuntimeProvider::Api");
    expect(runtime).toContain("RunRuntimeProvider::GeminiBrowser");
    expect(runtime).toContain("resolve_profile_for_backend");
    expect(runtime).toContain("resolve_effective_model");
    expect(runtime).toContain("resolve_model_input_token_limit_for_backend");
    expect(runtime).toContain("RunCompletionRuntime::Api");
    expect(runtime).toContain("RunCompletionRuntime::GeminiBrowser");
  });

  it("keeps orchestration dependencies out of runtime_config", () => {
    const runtimeConfig = normalized(runtimeConfigSource);
    const forbidden = [
      "tauri::",
      "AppHandle",
      "#[tauri::command]",
      "RunCompletionRuntime",
      "resolve_profile_for_backend",
      "resolve_effective_model",
      "resolve_model_input_token_limit_for_backend",
      "YoutubeSummaryStageExecutionRequest",
      "PromptPackRunState",
      "super::runtime",
    ];

    for (const marker of forbidden) {
      expect(runtimeConfig).not.toContain(marker);
    }
  });
});
```

Expected: the contract imports the not-yet-existing module and precisely separates the production portion of `runtime.rs` from its database tests.

- [ ] **Step 2: Run the contract to verify RED**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-runtime-config-contract.test.ts
```

Expected: FAIL with a Vite module-resolution error because `src-tauri/src/prompt_packs/runtime_config.rs` does not exist. This is the intended RED, not an infrastructure failure.

- [ ] **Step 3: Create `runtime_config.rs` with the exact moved behavior**

Create `src-tauri/src/prompt_packs/runtime_config.rs`:

```rust
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RunRuntimeProvider {
    Api,
    GeminiBrowser,
}

impl RunRuntimeProvider {
    fn parse(value: &str) -> AppResult<Self> {
        match value {
            "api" => Ok(Self::Api),
            "gemini_browser" => Ok(Self::GeminiBrowser),
            other => Err(AppError::validation(format!(
                "Unsupported prompt-pack runtime provider: {other}"
            ))),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct RunRuntimeConfig {
    pub(super) runtime_provider: RunRuntimeProvider,
    pub(super) profile_id: Option<String>,
    pub(super) model_override: Option<String>,
    pub(super) browser_provider_config:
        Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
}

pub(super) async fn load_run_runtime_config(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<RunRuntimeConfig> {
    sqlx::query_as::<_, (Option<String>, Option<String>, String, Option<String>)>(
        "SELECT provider_profile_id, model, runtime_provider, browser_provider_config_json
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
    .and_then(
        |(profile_id, model_override, runtime_provider, browser_config_json)| {
            let browser_provider_config = browser_config_json
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|error| {
                    AppError::internal(format!("parse Browser Provider config snapshot: {error}"))
                })?;
            Ok(RunRuntimeConfig {
                runtime_provider: RunRuntimeProvider::parse(&runtime_provider)?,
                profile_id,
                model_override,
                browser_provider_config,
            })
        },
    )
}
```

Expected: the file contains only persistence decoding and its direct dependencies; no Tauri, transport, stage, or lifecycle code is introduced.

- [ ] **Step 4: Register the module and connect production `runtime.rs`**

In `src-tauri/src/prompt_packs/mod.rs`, add the private module immediately after `pub mod runtime;`:

```rust
pub mod runtime;
mod runtime_config;
pub mod seed;
```

In the production imports of `src-tauri/src/prompt_packs/runtime.rs`, add immediately after the `run_store` import:

```rust
use super::runtime_config::{load_run_runtime_config, RunRuntimeProvider};
```

Delete the complete old production block beginning with:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunRuntimeProvider {
```

and ending with the closing brace of `load_run_runtime_config`. Do not change `execute_youtube_summary_run`, its provider match, profile/model resolution, `RunCompletionRuntime` construction, or the following `mark_prompt_pack_run_failed` function.

Expected: production `runtime.rs` calls the sibling loader once and retains the complete provider-resolution/construction match unchanged.

- [ ] **Step 5: Redirect only the runtime-test imports**

In `runtime::tests`, add this import immediately after the `run_store` import:

```rust
    use super::super::runtime_config::{load_run_runtime_config, RunRuntimeProvider};
```

Replace the current `use super::{...};` block with:

```rust
    use super::{
        browser_runtime_start_blocking_failure, cleanup_interrupted_prompt_pack_runs_in_pool,
        clear_prompt_pack_cancellation_smoke_fixture_in_pool, now_string,
        seed_prompt_pack_cancellation_smoke_fixture_in_pool, PromptPackRunState,
    };
```

Do not move or rewrite the three `load_run_runtime_config_*` test bodies or their database fixtures.

Expected: tests reach the sibling module directly; unrelated runtime-test imports and bodies remain unchanged.

- [ ] **Step 6: Run focused GREEN checks**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-runtime-config-contract.test.ts
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-prompt-pack-runtime-config --lib prompt_packs::runtime::tests::load_run_runtime_config_
```

Expected: the source contract passes all five cases and exactly three Rust characterization tests pass.

- [ ] **Step 7: Run the complete Prompt Pack runtime test module**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-prompt-pack-runtime-config --lib prompt_packs::runtime::tests
```

Expected: all `prompt_packs::runtime::tests` pass with no new warnings or ignored failures.

- [ ] **Step 8: Run full Rust and Vitest suites**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-prompt-pack-runtime-config
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
npm.cmd run test
```

Expected: both complete suites pass. Existing completion-transport and stage-execution ownership contracts continue to pass without modification.

- [ ] **Step 9: Run formatting and all-target compilation checks**

Run:

```powershell
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo check --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-prompt-pack-runtime-config --all-targets
```

Expected: rustfmt and all Rust targets pass with no new warnings.

- [ ] **Step 10: Verify exact implementation scope and commit**

Run:

```powershell
$expected = @(
    'src-tauri/src/prompt_packs/mod.rs',
    'src-tauri/src/prompt_packs/runtime.rs',
    'src-tauri/src/prompt_packs/runtime_config.rs',
    'src/lib/prompt-pack-runtime-config-contract.test.ts'
) | Sort-Object
$actual = @(
    git status --short --untracked-files=all |
        ForEach-Object { $_.Substring(3) }
) | Sort-Object
$scopeDiff = @(Compare-Object $expected $actual)
$actual
if ($scopeDiff.Count -ne 0) {
    $scopeDiff
    exit 1
}
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git diff --stat
git diff -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src-tauri/src/prompt_packs/runtime_config.rs `
    src/lib/prompt-pack-runtime-config-contract.test.ts
git add -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src-tauri/src/prompt_packs/runtime_config.rs `
    src/lib/prompt-pack-runtime-config-contract.test.ts
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git commit -m "refactor: extract prompt pack runtime config"
git status --short --branch
```

Expected: exactly the four declared implementation files are changed; the staged diff is whitespace-clean; commit succeeds; worktree is clean afterward.
