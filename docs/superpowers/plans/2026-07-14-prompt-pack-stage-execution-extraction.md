# Prompt Pack Stage Execution Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the five Prompt Pack stage preparation/execution functions and their two Gem helpers from `runtime.rs` into one private `stage_execution.rs` module without changing requests, budgets, provider behavior, cancellation, messages, errors, or public APIs.

**Architecture:** Add private `prompt_packs::stage_execution` between the existing stage-request policy and completion-transport modules. Keep the stage request dispatcher and run lifecycle in `runtime.rs`; the new module prepares each stage request and invokes the existing `RunCompletionRuntime` interface.

**Tech Stack:** Rust 2021, Tauri 2, Tokio, SQLx/SQLite, SvelteKit/TypeScript, Vitest raw-source contracts.

## Global Constraints

- Implement the approved design in `docs/superpowers/specs/2026-07-14-prompt-pack-stage-execution-extraction-design.md` at or after commit `1c64154c`.
- Modify exactly six implementation files: `src-tauri/src/prompt_packs/mod.rs`, `src-tauri/src/prompt_packs/runtime.rs`, new `src-tauri/src/prompt_packs/stage_execution.rs`, new `src/lib/prompt-pack-stage-execution-contract.test.ts`, `src/lib/prompt-pack-completion-transport-contract.test.ts`, and `src/lib/prompt-pack-stage-request-policy-contract.test.ts`.
- Register exactly private `mod stage_execution;`; do not expose or re-export the module itself.
- Move exactly five stage functions with `pub(super)` visibility and keep `gem_part_phase` and `gem_part_started_message` private to the new module.
- Keep the `YoutubeSummaryStageExecutionRequest` match, runtime-config loading, provider construction, commands, preflight/readiness, terminal events, cleanup, and fixtures in `runtime.rs`.
- Preserve the seven moved bodies statement-for-statement except for visibility, module paths, imports, indentation, and rustfmt output.
- Do not change stage names, phases, started messages, discriminators, repair-attempt values, source-snapshot handling, budgets, request fields, cancellation propagation, error mapping, or results.
- Do not add dependencies, traits, dispatch abstractions, mocks, migrations, logging, retries, timeouts, fallbacks, or error wrapping.
- Keep existing Rust test bodies and fixtures in their current modules.
- Do not modify `docs/project.md` or `docs/value-registry.md`; no registered or wire value changes.
- Preserve unrelated user changes and require a clean worktree before starting.

---

### Task 1: Extract Prompt Pack Stage Execution

**Files:**
- Create: `src-tauri/src/prompt_packs/stage_execution.rs`
- Create: `src/lib/prompt-pack-stage-execution-contract.test.ts`
- Modify: `src-tauri/src/prompt_packs/mod.rs` near private module declarations
- Modify: `src-tauri/src/prompt_packs/runtime.rs` production imports and the contiguous stage-function block
- Modify: `src/lib/prompt-pack-completion-transport-contract.test.ts` imports and stage-call ownership assertion
- Modify: `src/lib/prompt-pack-stage-request-policy-contract.test.ts` lifecycle-helper ownership assertion

**Interfaces:**
- Consumes: `RunCompletionRuntime`, `StageCompletionRequest`, stage request-policy builders/budgets, `AppHandle`, `SqlitePool`, `CancellationToken`, and existing Prompt Pack stage request/result/error types.
- Produces: private module `prompt_packs::stage_execution`; five `pub(super) async fn run_*` entry points with their existing signatures; private `gem_part_phase` and `gem_part_started_message` helpers.
- Preserves: the dispatcher closure in `runtime.rs`, all runtime configuration/provider construction, all public commands, and all behavior of the five stage paths.

- [ ] **Step 1: Verify clean-tree, approved-spec, formatting, and ownership baselines**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git merge-base --is-ancestor 1c64154c HEAD
$approvedSpecPresent = $LASTEXITCODE -eq 0
$runtime = Get-Content -Raw 'src-tauri/src/prompt_packs/runtime.rs'
$stageDefinitions = [regex]::Matches(
    $runtime,
    '(?m)^async fn run_(?:transcript_analysis|synthesis|json_repair|gem_analysis_part(?:_repair)?)_stage_request|^async fn run_gem_analysis_part_repair_request'
).Count
$gemHelperDefinitions = [regex]::Matches(
    $runtime,
    '(?m)^fn gem_part_(?:phase|started_message)\s*\('
).Count
$modelContextCalls = [regex]::Matches($runtime, 'completion_runtime\.model_context\(\)\.await\?').Count
$executeCalls = [regex]::Matches($runtime, 'completion_runtime\s*\.execute\s*\(').Count
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_PRESENT=$approvedSpecPresent"
"STAGE_DEFINITION_COUNT=$stageDefinitions"
"GEM_HELPER_DEFINITION_COUNT=$gemHelperDefinitions"
"MODEL_CONTEXT_CALL_COUNT=$modelContextCalls"
"EXECUTE_CALL_COUNT=$executeCalls"
if (
    $status.Count -ne 0 -or
    -not $approvedSpecPresent -or
    $stageDefinitions -ne 5 -or
    $gemHelperDefinitions -ne 2 -or
    $modelContextCalls -ne 5 -or
    $executeCalls -ne 5
) { exit 1 }
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: clean tree, approved spec present, exactly five stage definitions, two Gem helper definitions, five model-context calls, five execute calls, and rustfmt exit 0.

- [ ] **Step 2: Add the failing stage-execution contract and update neighboring contracts**

Create `src/lib/prompt-pack-stage-execution-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import promptPacksModuleSource from "../../src-tauri/src/prompt_packs/mod.rs?raw";
import runtimeSource from "../../src-tauri/src/prompt_packs/runtime.rs?raw";
import stageExecutionSource from "../../src-tauri/src/prompt_packs/stage_execution.rs?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");
const matches = (source: string, pattern: RegExp) => source.match(pattern) ?? [];

const stageFunctions = [
  "run_transcript_analysis_stage_request",
  "run_synthesis_stage_request",
  "run_json_repair_stage_request",
  "run_gem_analysis_part_stage_request",
  "run_gem_analysis_part_repair_request",
] as const;

const gemHelpers = ["gem_part_phase", "gem_part_started_message"] as const;

describe("Prompt Pack stage execution ownership", () => {
  it("registers a private stage_execution sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod stage_execution;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod stage_execution;/);
  });

  it.each(stageFunctions)("moves %s out of runtime with sibling visibility", (name) => {
    const stageExecution = normalized(stageExecutionSource);
    const runtime = normalized(runtimeSource);
    const definition = new RegExp(`^pub\\(super\\) async fn ${name}\\s*\\(`, "m");
    const runtimeDefinition = new RegExp(`^(?:pub\\(super\\) )?async fn ${name}\\s*\\(`, "m");

    expect(stageExecution).toMatch(definition);
    expect(runtime).not.toMatch(runtimeDefinition);
    expect(matches(runtime, new RegExp(`\\b${name}\\s*\\(`, "g"))).toHaveLength(1);
  });

  it.each(gemHelpers)("keeps %s private beside its consumers", (name) => {
    const stageExecution = normalized(stageExecutionSource);
    const runtime = normalized(runtimeSource);

    expect(stageExecution).toMatch(new RegExp(`^fn ${name}\\s*\\(`, "m"));
    expect(stageExecution).not.toMatch(
      new RegExp(`^pub(?:\\([^)]*\\))?\\s+fn ${name}\\s*\\(`, "m"),
    );
    expect(runtime).not.toMatch(new RegExp(`^fn ${name}\\s*\\(`, "m"));
  });

  it("owns exactly five policy-to-transport bridges", () => {
    const stageExecution = normalized(stageExecutionSource);
    const runtime = normalized(runtimeSource);

    expect(
      matches(stageExecution, /completion_runtime\.model_context\(\)\.await\?/g),
    ).toHaveLength(5);
    expect(
      matches(stageExecution, /completion_runtime\s*\.execute\s*\(/g),
    ).toHaveLength(5);
    expect(runtime).not.toContain("completion_runtime.model_context().await?");
    expect(runtime).not.toMatch(/completion_runtime\s*\.execute\s*\(/);
  });

  it("keeps dispatch and lifecycle responsibilities in runtime", () => {
    const stageExecution = normalized(stageExecutionSource);
    const runtime = normalized(runtimeSource);

    expect(
      matches(runtime, /YoutubeSummaryStageExecutionRequest::/g),
    ).toHaveLength(5);
    const forbidden = [
      "YoutubeSummaryStageExecutionRequest::",
      "#[tauri::command]",
      "load_run_runtime_config",
      "preflight_youtube_summary_run",
      "browser_runtime_start_failures_for_request",
      "emit_youtube_summary_terminal_event",
      "cleanup_interrupted_prompt_pack_runs",
      "seed_prompt_pack_cancellation_smoke_fixture",
      "clear_prompt_pack_cancellation_smoke_fixture",
      "super::runtime",
    ];

    for (const marker of forbidden) {
      expect(stageExecution).not.toContain(marker);
    }
  });
});
```

In `src/lib/prompt-pack-completion-transport-contract.test.ts`, add this raw import beside the existing imports:

```ts
import stageExecutionSource from "../../src-tauri/src/prompt_packs/stage_execution.rs?raw";
```

Replace the existing test named `removes all five stage-level provider matches` with:

```ts
it("keeps all five stage bridges behind the transport interface", () => {
  const runtime = normalized(runtimeSource);
  const stageExecution = normalized(stageExecutionSource);

  expect(matches(runtime, /match\s+&?completion_runtime\b/g)).toHaveLength(0);
  expect(
    matches(stageExecution, /completion_runtime\.model_context\(\)\.await\?/g),
  ).toHaveLength(5);
  expect(
    matches(stageExecution, /completion_runtime\s*\.execute\s*\(/g),
  ).toHaveLength(5);
});
```

In `src/lib/prompt-pack-stage-request-policy-contract.test.ts`, replace the existing test named `keeps execution lifecycle messages in runtime` with:

```ts
it("keeps execution lifecycle messages out of request policy", () => {
  const policy = normalized(stageRequestPolicySource);

  expect(policy).not.toMatch(/^fn gem_part_phase\s*\(/m);
  expect(policy).not.toMatch(/^fn gem_part_started_message\s*\(/m);
});
```

Do not remove `runtimeSource` from the policy contract; its other ownership tests still use it.

Expected: all three contracts normalize CRLF; the new contract owns the positive stage/helper assertions, while the two existing contracts point to the new boundary without duplicating it.

- [ ] **Step 3: Run the three source contracts to verify RED**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-stage-execution-contract.test.ts `
    src/lib/prompt-pack-completion-transport-contract.test.ts `
    src/lib/prompt-pack-stage-request-policy-contract.test.ts
```

Expected: FAIL during Vite module resolution because `src-tauri/src/prompt_packs/stage_execution.rs` does not exist. This is the intended RED, not a Vitest infrastructure failure.

- [ ] **Step 4: Register the private module and create the complete stage-execution module**

In `src-tauri/src/prompt_packs/mod.rs`, add this declaration in alphabetical order immediately after `pub mod seed;` and before `pub mod stage_io;`:

```rust
mod stage_execution;
```

Create `src-tauri/src/prompt_packs/stage_execution.rs` with exactly:

```rust
use sqlx::SqlitePool;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::completion_transport::{RunCompletionRuntime, StageCompletionRequest};
use super::json_repair::JsonRepairStageExecutionRequest;
use super::stage_request_policy::{
    build_gem_analysis_part_llm_request, build_gem_analysis_part_repair_llm_request,
    build_json_repair_llm_request, build_synthesis_llm_request,
    build_transcript_analysis_llm_request, gem_analysis_part_max_output_tokens,
    gem_part_repair_request_suffix, gem_part_request_suffix,
    synthesis_stage_max_output_token_budget, transcript_analysis_control_preset,
    transcript_analysis_max_output_tokens, transcript_analysis_stage_max_output_token_budget,
    transcript_analysis_stage_max_output_token_budget_for_control_preset,
};
use super::youtube_summary::{
    GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
    LlmCompletion as PromptPackLlmCompletion, SynthesisStageExecutionRequest,
    TranscriptAnalysisStageExecutionRequest, YoutubeSummaryStageExecutionError,
};

pub(super) async fn run_transcript_analysis_stage_request(
    handle: AppHandle,
    pool: SqlitePool,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: TranscriptAnalysisStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let control_preset = transcript_analysis_control_preset(&stage_request.prompt_input_json);
    let stage_output_budget =
        transcript_analysis_stage_max_output_token_budget_for_control_preset(&control_preset)?;
    let max_output_tokens = transcript_analysis_max_output_tokens(
        stage_output_budget,
        model_context.model_output_limit,
    );
    let llm_request = build_transcript_analysis_llm_request(
        &stage_request,
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );

    completion_runtime
        .execute(
            handle,
            pool,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: Some(stage_request.source_snapshot_id),
                stage_name: "youtube_summary/transcript_analysis".to_string(),
                phase: "transcript_analysis",
                started_message: "Analyzing transcript",
                repair_attempt_number: None,
                request_discriminator: None,
                run_cancellation_token,
            },
        )
        .await
}

pub(super) async fn run_synthesis_stage_request(
    handle: AppHandle,
    pool: SqlitePool,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: SynthesisStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let stage_output_budget = synthesis_stage_max_output_token_budget()?;
    let max_output_tokens = transcript_analysis_max_output_tokens(
        stage_output_budget,
        model_context.model_output_limit,
    );
    let llm_request = build_synthesis_llm_request(
        stage_request.run_id,
        stage_request.stage_run_id,
        stage_request.prompt_input_json.clone(),
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );

    completion_runtime
        .execute(
            handle,
            pool,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: None,
                stage_name: "youtube_summary/synthesis".to_string(),
                phase: "synthesis",
                started_message: "Synthesizing videos",
                repair_attempt_number: None,
                request_discriminator: None,
                run_cancellation_token,
            },
        )
        .await
}

pub(super) async fn run_json_repair_stage_request(
    handle: AppHandle,
    pool: SqlitePool,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: JsonRepairStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let stage_output_budget = if stage_request.stage_name == "youtube_summary/synthesis" {
        synthesis_stage_max_output_token_budget()?
    } else if stage_request.stage_name == "youtube_summary/transcript_analysis" {
        let control_preset = transcript_analysis_control_preset(&stage_request.prompt_input_json);
        transcript_analysis_stage_max_output_token_budget_for_control_preset(&control_preset)?
    } else {
        transcript_analysis_stage_max_output_token_budget()?
    };
    let max_output_tokens = transcript_analysis_max_output_tokens(
        stage_output_budget,
        model_context.model_output_limit,
    );
    let llm_request = build_json_repair_llm_request(
        &stage_request,
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );

    completion_runtime
        .execute(
            handle,
            pool,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: None,
                stage_name: stage_request.stage_name.clone(),
                phase: "repair",
                started_message: "Repairing provider JSON",
                repair_attempt_number: Some(stage_request.attempt_number),
                request_discriminator: None,
                run_cancellation_token,
            },
        )
        .await
}

pub(super) async fn run_gem_analysis_part_stage_request(
    handle: AppHandle,
    pool: SqlitePool,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: GemAnalysisPartStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let max_output_tokens =
        gem_analysis_part_max_output_tokens(stage_request.part, model_context.model_output_limit);
    let llm_request = build_gem_analysis_part_llm_request(
        &stage_request,
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );
    let phase = gem_part_phase(stage_request.part);
    let started_message = gem_part_started_message(stage_request.part);
    let request_discriminator = Some(gem_part_request_suffix(stage_request.part));

    completion_runtime
        .execute(
            handle,
            pool,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: Some(stage_request.source_snapshot_id),
                stage_name: "youtube_summary/transcript_analysis".to_string(),
                phase,
                started_message,
                repair_attempt_number: None,
                request_discriminator,
                run_cancellation_token,
            },
        )
        .await
}

pub(super) async fn run_gem_analysis_part_repair_request(
    handle: AppHandle,
    pool: SqlitePool,
    completion_runtime: RunCompletionRuntime,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: GemAnalysisPartRepairRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let model_context = completion_runtime.model_context().await?;
    let max_output_tokens =
        gem_analysis_part_max_output_tokens(stage_request.part, model_context.model_output_limit);
    let llm_request = build_gem_analysis_part_repair_llm_request(
        &stage_request,
        model_context.profile_id,
        model_context.model_override,
        max_output_tokens,
    );
    let request_discriminator = Some(gem_part_repair_request_suffix(
        stage_request.part,
        stage_request.attempt_number,
    ));

    completion_runtime
        .execute(
            handle,
            pool,
            StageCompletionRequest {
                llm_request,
                run_id: stage_request.run_id,
                stage_run_id: stage_request.stage_run_id,
                source_snapshot_id: Some(stage_request.source_snapshot_id),
                stage_name: "youtube_summary/transcript_analysis".to_string(),
                phase: "gem_part_repair",
                started_message: "Gem analysis: repairing part JSON",
                repair_attempt_number: None,
                request_discriminator,
                run_cancellation_token,
            },
        )
        .await
}

fn gem_part_phase(part: GemAnalysisPart) -> &'static str {
    match part {
        GemAnalysisPart::Passport => "gem_passport",
        GemAnalysisPart::Comments => "gem_comments",
        GemAnalysisPart::DeepRecap => "gem_deep_recap",
    }
}

fn gem_part_started_message(part: GemAnalysisPart) -> &'static str {
    match part {
        GemAnalysisPart::Passport => "Gem analysis: building analytical passport",
        GemAnalysisPart::Comments => "Gem analysis: analyzing comments",
        GemAnalysisPart::DeepRecap => "Gem analysis: writing deep recap",
    }
}
```

Expected: the new module is a statement-for-statement relocation of the existing seven bodies, with only the five stage entry points widened to `pub(super)`.

- [ ] **Step 5: Rewire runtime and remove the old contiguous block**

In `src-tauri/src/prompt_packs/runtime.rs`, replace:

```rust
use tokio_util::sync::CancellationToken;

use super::completion_transport::{RunCompletionRuntime, StageCompletionRequest};
use super::json_repair::JsonRepairStageExecutionRequest;
```

with:

```rust
use super::completion_transport::RunCompletionRuntime;
```

Add this import beside the other sibling-module imports:

```rust
use super::stage_execution::{
    run_gem_analysis_part_repair_request, run_gem_analysis_part_stage_request,
    run_json_repair_stage_request, run_synthesis_stage_request,
    run_transcript_analysis_stage_request,
};
```

Replace the production `stage_request_policy` import with the exact remaining runtime requirements:

```rust
use super::stage_request_policy::{
    gem_input_cap, transcript_analysis_stage_max_prompt_token_budget,
};
```

Replace the production `youtube_summary` import with:

```rust
use super::youtube_summary::{
    execute_youtube_summary_run_with_stage_executor_with_options,
    load_youtube_summary_run_by_client_request_id_in_pool, model_budget_for_runtime,
    preflight_youtube_summary_in_pool, start_youtube_summary_run_in_pool,
    start_youtube_summary_run_with_preflight_failures_in_pool, GemAnalysisInputBudget,
    YoutubeSummaryExecutionOptions, YoutubeSummaryRunExecutionOutcome,
    YoutubeSummaryStageExecutionRequest,
};
```

Delete the complete contiguous block beginning with:

```rust
async fn run_transcript_analysis_stage_request(
```

and ending with the closing brace of:

```rust
fn gem_part_started_message(part: GemAnalysisPart) -> &'static str
```

Do not change the dispatcher inside `execute_youtube_summary_run`; its five branches continue to call the same names through the new sibling import.

In `runtime::tests`, replace the existing `use crate::prompt_packs::youtube_summary::{...};` block with:

```rust
use crate::prompt_packs::youtube_summary::{
    GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
    TranscriptAnalysisStageExecutionRequest, YoutubeSummaryStageExecutionError,
};
```

Remove `YoutubeSummaryStageExecutionError` from the local `use super::{...};` block because runtime production code no longer imports it.

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: rustfmt changes only the three scoped Rust files. The dispatcher still has five `YoutubeSummaryStageExecutionRequest::` branches, while runtime has no moved definitions or direct `model_context()`/`execute()` calls.

- [ ] **Step 6: Run all three source contracts for GREEN**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-stage-execution-contract.test.ts `
    src/lib/prompt-pack-completion-transport-contract.test.ts `
    src/lib/prompt-pack-stage-request-policy-contract.test.ts
```

Expected: Vitest runs all tests in the three files and exits 0. The new module owns five sibling-visible stage functions, two private Gem helpers, five model-context calls, and five execute calls; runtime retains exactly five dispatcher branches.

- [ ] **Step 7: Run focused Prompt Pack Rust tests**

Run:

```powershell
$filters = @(
    'prompt_packs::runtime::tests',
    'prompt_packs::youtube_summary::execution_tests'
)
foreach ($filter in $filters) {
    cargo test --manifest-path src-tauri/Cargo.toml --lib $filter
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: runtime tests, YouTube Summary execution tests, and all Prompt Pack Rust tests pass. Each filtered command must report at least one executed test; a zero-match run is not GREEN.

- [ ] **Step 8: Run complete frontend and Rust suites**

Run:

```powershell
npm.cmd run test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: complete Vitest and Rust suites exit 0 with no failed tests.

- [ ] **Step 9: Verify formatting and all Rust targets with zero warnings**

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

Expected: rustfmt exits 0, `CARGO_EXIT=0`, and `WARNING_COUNT=0`. `cmd.exe` keeps redirected native stderr as ordinary text under Windows PowerShell 5.1.

- [ ] **Step 10: Review exact six-file scope and commit**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$changed = @(git status --porcelain=v1 --untracked-files=all | ForEach-Object {
    $_.Substring(3).Replace('\', '/')
})
$allowed = @(
    'src-tauri/src/prompt_packs/mod.rs',
    'src-tauri/src/prompt_packs/runtime.rs',
    'src-tauri/src/prompt_packs/stage_execution.rs',
    'src/lib/prompt-pack-stage-execution-contract.test.ts',
    'src/lib/prompt-pack-completion-transport-contract.test.ts',
    'src/lib/prompt-pack-stage-request-policy-contract.test.ts'
)
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
"CHANGED=$($changed -join ',')"
"UNEXPECTED=$($unexpected -join ',')"
if ($changed.Count -ne 6 -or $unexpected.Count -ne 0) { exit 1 }
git add -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src-tauri/src/prompt_packs/stage_execution.rs `
    src/lib/prompt-pack-stage-execution-contract.test.ts `
    src/lib/prompt-pack-completion-transport-contract.test.ts `
    src/lib/prompt-pack-stage-request-policy-contract.test.ts
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git diff --cached --stat
git diff --cached -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src-tauri/src/prompt_packs/stage_execution.rs `
    src/lib/prompt-pack-stage-execution-contract.test.ts `
    src/lib/prompt-pack-completion-transport-contract.test.ts `
    src/lib/prompt-pack-stage-request-policy-contract.test.ts
git commit -m "refactor: extract prompt pack stage execution"
git status --short --branch
```

Expected: the implementation commit contains exactly the private module registration, seven-function extraction, runtime import/dispatcher wiring, new ownership contract, and two neighboring contract updates. The worktree is clean after commit.
