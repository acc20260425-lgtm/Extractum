# Prompt Pack Stage Request Policy Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract deterministic Prompt Pack LLM request construction and token-budget policy from `runtime.rs` into a focused private `stage_request_policy.rs` module without changing prompts, request IDs, limits, errors, public interfaces, or runtime behavior.

**Architecture:** Add a private sibling module under `prompt_packs` that owns the bundled stage configuration, prompt constants, request builders, request-ID suffixes, and token-budget calculations. Keep provider execution, Tauri integration, cancellation, persistence, progress/lifecycle messages, and all existing behavioral tests in `runtime.rs`; the runtime imports the policy through sibling-visible `pub(super)` functions.

**Tech Stack:** Rust 2021, Tauri 2, Tokio, Serde/serde_json, TypeScript, Vitest raw-source contracts.

## Global Constraints

- Implement the approved design in `docs/superpowers/specs/2026-07-13-prompt-pack-stage-request-policy-extraction-design.md`.
- Modify only `src-tauri/src/prompt_packs/mod.rs`, `src-tauri/src/prompt_packs/runtime.rs`, new `src-tauri/src/prompt_packs/stage_request_policy.rs`, and new `src/lib/prompt-pack-stage-request-policy-contract.test.ts`.
- Register exactly private `mod stage_request_policy;`; do not re-export the module or its contents.
- Preserve literally every prompt string, whitespace sequence, escape, Unicode character, JSON example, Markdown instruction, request-ID format, budget, fallback, clamp, and `AppError::internal` message.
- Keep the two `include_str!` paths byte-for-byte unchanged; the new Rust file must remain in the same `src-tauri/src/prompt_packs/` directory so their relative resolution is unchanged.
- Give only the 15 enumerated policy functions and `DETAILED_REPORT_CONTROL_PRESET` `pub(super)` visibility; keep the remaining constants, configuration structs, and implementation helpers private.
- Keep `gem_part_phase` and `gem_part_started_message` in `runtime.rs`.
- Keep all provider calls, browser orchestration, runtime state, cancellation, persistence, lifecycle/progress messages, Tauri commands, and existing Rust test bodies in `runtime.rs`.
- Keep existing prompt/budget tests in `runtime::tests`; after the move, change only their imports. Add the two approved Gem characterization tests before extraction and keep their bodies unchanged during extraction.
- Do not add dependencies, migrations, caching, lazy initialization, new error layers, DTO changes, frontend behavior, or registered/persisted values.
- Do not modify `docs/project.md` or `docs/value-registry.md` because behavior and registered values do not change.
- Preserve unrelated user changes and require a clean worktree before starting.

---

### Task 1: Extract Stage Request Policy

**Files:**
- Create: `src-tauri/src/prompt_packs/stage_request_policy.rs`
- Create: `src/lib/prompt-pack-stage-request-policy-contract.test.ts`
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Test: `src-tauri/src/prompt_packs/runtime.rs` (`mod tests`)

**Interfaces:**
- Consumes: `serde::Deserialize`, `serde_json`, `crate::error::{AppError, AppResult}`, `crate::llm::{LlmChatRequest, LlmMessage}`, `super::json_repair::JsonRepairStageExecutionRequest`, and `super::youtube_summary::{GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest, TranscriptAnalysisStageExecutionRequest}`.
- Produces: private module `prompt_packs::stage_request_policy` with sibling-visible `DETAILED_REPORT_CONTROL_PRESET` and the exact functions `transcript_analysis_control_preset`, `build_transcript_analysis_llm_request`, `build_synthesis_llm_request`, `gem_part_request_suffix`, `gem_part_repair_request_suffix`, `gem_analysis_part_max_output_tokens`, `build_gem_analysis_part_llm_request`, `build_gem_analysis_part_repair_llm_request`, `build_json_repair_llm_request`, `transcript_analysis_stage_max_output_token_budget`, `transcript_analysis_stage_max_prompt_token_budget`, `transcript_analysis_stage_max_output_token_budget_for_control_preset`, `synthesis_stage_max_output_token_budget`, `transcript_analysis_max_output_tokens`, and `gem_input_cap`, each with `pub(super)` visibility and its existing signature.
- Preserves: every public Tauri command/export and the runtime-owned helpers `gem_part_phase` and `gem_part_started_message`.

- [ ] **Step 1: Verify clean-tree, approved-spec, and formatting preconditions**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git merge-base --is-ancestor 1c61ba63 HEAD
$approvedSpecPresent = $LASTEXITCODE -eq 0
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_PRESENT=$approvedSpecPresent"
if ($status.Count -ne 0 -or -not $approvedSpecPresent) { exit 1 }
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: `STATUS_COUNT=0`, `APPROVED_SPEC_PRESENT=True`, and rustfmt exits 0. The clean formatting baseline guarantees that the later formatter run cannot change unrelated Rust files.

- [ ] **Step 2: Add the two missing Gem request characterization tests**

In `runtime.rs`, extend the test-only YouTube Summary import to exactly:

```rust
use crate::prompt_packs::youtube_summary::{
    GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
    TranscriptAnalysisStageExecutionRequest,
};
```

Add these tests immediately before `transcript_analysis_llm_request_embeds_frozen_stage_input`:

```rust
#[test]
fn gem_analysis_part_llm_request_preserves_part_and_frozen_input() {
    let request = build_gem_analysis_part_llm_request(
        &GemAnalysisPartStageExecutionRequest {
            run_id: 42,
            stage_run_id: 1001,
            source_snapshot_id: 501,
            source_ref_id: "source_ref_1".to_string(),
            part: GemAnalysisPart::Passport,
            prompt_input_json: "{\"frozen_input\":\"passport-source-material\"}".to_string(),
        },
        Some("profile-1".to_string()),
        Some("model-1".to_string()),
        Some(8_192),
    );

    assert_eq!(
        request.request_id,
        "prompt-pack-run-42-stage-1001-gem-passport"
    );
    assert_eq!(request.profile_id.as_deref(), Some("profile-1"));
    assert_eq!(request.model_override.as_deref(), Some("model-1"));
    assert_eq!(request.max_output_tokens, Some(8_192));
    assert_eq!(request.messages[0].role, "system");
    assert_eq!(request.messages[1].role, "user");
    assert!(request.messages[1].content.contains("\"part\": \"passport\""));
    assert!(request.messages[1]
        .content
        .contains("{\"frozen_input\":\"passport-source-material\"}"));
}

#[test]
fn gem_analysis_part_repair_llm_request_preserves_attempt_and_repair_context() {
    let request = build_gem_analysis_part_repair_llm_request(
        &GemAnalysisPartRepairRequest {
            run_id: 42,
            stage_run_id: 1002,
            source_snapshot_id: 501,
            source_ref_id: "source_ref_1".to_string(),
            part: GemAnalysisPart::Comments,
            attempt_number: 2,
            prompt_input_json: "{\"frozen_input\":\"comments-source-material\"}".to_string(),
            raw_output: "{invalid-provider-output".to_string(),
            error_message: "parser-sentinel: missing closing brace".to_string(),
        },
        Some("profile-1".to_string()),
        Some("model-1".to_string()),
        Some(4_096),
    );

    assert_eq!(
        request.request_id,
        "prompt-pack-run-42-stage-1002-gem-comments-repair-2"
    );
    assert_eq!(request.profile_id.as_deref(), Some("profile-1"));
    assert_eq!(request.model_override.as_deref(), Some("model-1"));
    assert_eq!(request.max_output_tokens, Some(4_096));
    assert_eq!(request.messages[0].role, "system");
    assert_eq!(request.messages[1].role, "user");
    assert!(request.messages[1]
        .content
        .contains("Repair the invalid Gem analysis part output for part `comments`"));
    assert!(request.messages[1]
        .content
        .contains("parser-sentinel: missing closing brace"));
    assert!(request.messages[1]
        .content
        .contains("{\"frozen_input\":\"comments-source-material\"}"));
    assert!(request.messages[1]
        .content
        .contains("{invalid-provider-output"));
}
```

Expected: the tests directly freeze normal and repair request IDs, selected Gem part, request metadata, roles, frozen input, parser error, and invalid provider output before any ownership change.

- [ ] **Step 3: Run the characterization tests against the current implementation**

Run:

```powershell
$tests = @(
    'prompt_packs::runtime::tests::gem_analysis_part_llm_request_preserves_part_and_frozen_input',
    'prompt_packs::runtime::tests::gem_analysis_part_repair_llm_request_preserves_attempt_and_repair_context'
)
foreach ($test in $tests) {
    cargo test --manifest-path src-tauri/Cargo.toml --lib $test -- --exact
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Expected: each command runs exactly one test and passes. These are characterization GREEN tests: they prove current behavior before the mechanical move rather than introducing new behavior.

- [ ] **Step 4: Add the failing source-ownership contract**

Create `src/lib/prompt-pack-stage-request-policy-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import promptPacksModuleSource from "../../src-tauri/src/prompt_packs/mod.rs?raw";
import runtimeSource from "../../src-tauri/src/prompt_packs/runtime.rs?raw";
import stageRequestPolicySource from "../../src-tauri/src/prompt_packs/stage_request_policy.rs?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

const extractedFunctions = [
  "transcript_analysis_control_preset",
  "build_transcript_analysis_llm_request",
  "build_synthesis_llm_request",
  "gem_part_request_suffix",
  "gem_part_repair_request_suffix",
  "gem_analysis_part_max_output_tokens",
  "build_gem_analysis_part_llm_request",
  "build_gem_analysis_part_repair_llm_request",
  "build_json_repair_llm_request",
  "transcript_analysis_stage_max_output_token_budget",
  "transcript_analysis_stage_max_prompt_token_budget",
  "transcript_analysis_stage_max_output_token_budget_for_control_preset",
  "synthesis_stage_max_output_token_budget",
  "transcript_analysis_max_output_tokens",
  "gem_input_cap",
] as const;

const movedConstants = [
  "TRANSCRIPT_ANALYSIS_STAGE_JSON",
  "SYNTHESIS_STAGE_JSON",
  "DETAILED_REPORT_CONTROL_PRESET",
  "STANDARD_VIDEO_SUMMARY_PROMPT",
  "DETAILED_VIDEO_SUMMARY_PROMPT",
] as const;

const movedStructs = [
  "StageRuntimeConfigAsset",
  "StageRuntimeConfiguration",
  "StageBudgetLimits",
] as const;

describe("Prompt Pack stage request policy ownership", () => {
  it("registers a private stage_request_policy sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod stage_request_policy;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod stage_request_policy;/);
  });

  it.each(extractedFunctions)("moves %s out of runtime", (functionName) => {
    const policy = normalized(stageRequestPolicySource);
    const runtime = normalized(runtimeSource);
    const policyDefinition = new RegExp(
      `^pub\\(super\\)\\s+fn\\s+${functionName}\\s*\\(`,
      "m",
    );
    const runtimeDefinition = new RegExp(
      `^(?:pub\\(super\\)\\s+)?fn\\s+${functionName}\\s*\\(`,
      "m",
    );

    expect(policy).toMatch(policyDefinition);
    expect(runtime).not.toMatch(runtimeDefinition);
  });

  it("moves prompt assets and budget configuration without changing include paths", () => {
    const policy = normalized(stageRequestPolicySource);
    const runtime = normalized(runtimeSource);

    for (const constantName of movedConstants) {
      expect(policy).toMatch(new RegExp(`\\b${constantName}\\b`));
      expect(runtime).not.toMatch(new RegExp(`\\b(?:const|static)\\s+${constantName}\\b`));
    }
    for (const structName of movedStructs) {
      expect(policy).toMatch(new RegExp(`^struct\\s+${structName}\\s*\\{`, "m"));
      expect(runtime).not.toMatch(new RegExp(`^struct\\s+${structName}\\s*\\{`, "m"));
    }
    expect(policy).toMatch(
      /^pub\(super\) const DETAILED_REPORT_CONTROL_PRESET: &str = "detailed_report";$/m,
    );
    expect(policy).toContain(
      'include_str!("../../prompt-packs/youtube_summary/1.0.0/runtime/transcript_analysis.json")',
    );
    expect(policy).toContain(
      'include_str!("../../prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json")',
    );
  });

  it("keeps execution lifecycle messages in runtime", () => {
    const policy = normalized(stageRequestPolicySource);
    const runtime = normalized(runtimeSource);

    expect(runtime).toMatch(/^fn gem_part_phase\s*\(/m);
    expect(runtime).toMatch(/^fn gem_part_started_message\s*\(/m);
    expect(policy).not.toMatch(/^fn gem_part_phase\s*\(/m);
    expect(policy).not.toMatch(/^fn gem_part_started_message\s*\(/m);
  });

  it("keeps the policy module independent from runtime infrastructure", () => {
    const policy = normalized(stageRequestPolicySource);

    expect(policy).not.toMatch(/\btauri\b/);
    expect(policy).not.toMatch(/\bsqlx\b/);
    expect(policy).not.toMatch(/\bCancellationToken\b/);
    expect(policy).not.toMatch(/\bsuper::runtime\b/);
    expect(policy).not.toMatch(/\bAppHandle\b/);
  });
});
```

Expected: CRLF normalization makes the contract stable on Windows; the contract checks the exact approved ownership boundary and intentionally does not compare large prompt bodies.

- [ ] **Step 5: Run the source contract to verify RED**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-stage-request-policy-contract.test.ts
```

Expected: FAIL during Vite module resolution because `src-tauri/src/prompt_packs/stage_request_policy.rs` does not exist. This is the intended RED, not a Vitest infrastructure failure.

- [ ] **Step 6: Register the private sibling module**

In `src-tauri/src/prompt_packs/mod.rs`, insert this line in alphabetical order between `pub mod stage_output_normalization;` and `pub mod store;`:

```rust
mod stage_request_policy;
```

Do not add `pub`, `pub(crate)`, or a re-export.

- [ ] **Step 7: Create the policy module by moving the approved definitions unchanged**

Create `src-tauri/src/prompt_packs/stage_request_policy.rs` with this exact import block:

```rust
use serde::Deserialize;

use super::json_repair::JsonRepairStageExecutionRequest;
use super::youtube_summary::{
    GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
    TranscriptAnalysisStageExecutionRequest,
};
use crate::error::{AppError, AppResult};
use crate::llm::{LlmChatRequest, LlmMessage};
```

After the imports, mechanically move the complete existing definitions from `runtime.rs` in their current relative order. Copy every body byte-for-byte; change only the visibility markers listed here:

```text
private const TRANSCRIPT_ANALYSIS_STAGE_JSON
private const SYNTHESIS_STAGE_JSON
private struct StageRuntimeConfigAsset
private struct StageRuntimeConfiguration
private struct StageBudgetLimits
pub(super) const DETAILED_REPORT_CONTROL_PRESET
private const STANDARD_VIDEO_SUMMARY_PROMPT
private const DETAILED_VIDEO_SUMMARY_PROMPT
pub(super) fn transcript_analysis_control_preset
private fn transcript_analysis_summary_prompt
pub(super) fn build_transcript_analysis_llm_request
pub(super) fn build_synthesis_llm_request
pub(super) fn gem_part_request_suffix
pub(super) fn gem_part_repair_request_suffix
private fn gem_part_output_budget
pub(super) fn gem_analysis_part_max_output_tokens
pub(super) fn build_gem_analysis_part_llm_request
pub(super) fn build_gem_analysis_part_repair_llm_request
pub(super) fn build_json_repair_llm_request
pub(super) fn transcript_analysis_stage_max_output_token_budget
pub(super) fn transcript_analysis_stage_max_prompt_token_budget
pub(super) fn transcript_analysis_stage_max_output_token_budget_for_control_preset
pub(super) fn synthesis_stage_max_output_token_budget
private fn stage_max_prompt_token_budget
private fn stage_max_output_token_budget
pub(super) fn transcript_analysis_max_output_tokens
pub(super) fn gem_input_cap
```

Use these exact unchanged `include_str!` declarations at the top of the moved definition block:

```rust
const TRANSCRIPT_ANALYSIS_STAGE_JSON: &str =
    include_str!("../../prompt-packs/youtube_summary/1.0.0/runtime/transcript_analysis.json");
const SYNTHESIS_STAGE_JSON: &str =
    include_str!("../../prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json");
```

Do not move `gem_part_phase` or `gem_part_started_message`. Do not retype, normalize, rewrap, or re-encode either large prompt constant; use a literal source move so the implementation diff shows deletion from `runtime.rs` and addition to the new file without content edits.

- [ ] **Step 8: Wire runtime to policy and remove only moved definitions**

In `runtime.rs`, remove:

```rust
use serde::Deserialize;
```

From the existing `crate::llm` import, remove only `LlmMessage`; keep `LlmChatRequest` and every execution-related type. Add this sibling import after `use super::run_store::{ ... };`:

```rust
use super::stage_request_policy::{
    build_gem_analysis_part_llm_request, build_gem_analysis_part_repair_llm_request,
    build_json_repair_llm_request, build_synthesis_llm_request,
    build_transcript_analysis_llm_request, gem_analysis_part_max_output_tokens, gem_input_cap,
    gem_part_repair_request_suffix, gem_part_request_suffix, synthesis_stage_max_output_token_budget,
    transcript_analysis_control_preset, transcript_analysis_max_output_tokens,
    transcript_analysis_stage_max_output_token_budget,
    transcript_analysis_stage_max_output_token_budget_for_control_preset,
    transcript_analysis_stage_max_prompt_token_budget,
};
```

Delete from `runtime.rs` exactly the constants, three structs, and functions moved in Step 7. Leave these two existing definitions in their current runtime location and unchanged:

```rust
fn gem_part_phase(part: GemAnalysisPart) -> &'static str
fn gem_part_started_message(part: GemAnalysisPart) -> &'static str
```

Keep the existing runtime imports of `JsonRepairStageExecutionRequest`, `GemAnalysisPart`, `GemAnalysisPartRepairRequest`, `GemAnalysisPartStageExecutionRequest`, and `TranscriptAnalysisStageExecutionRequest`: runtime orchestration still uses those types in its own signatures and call sites.

- [ ] **Step 9: Update test imports without changing test bodies**

Inside `runtime::tests`, remove these policy names from the existing `use super::{ ... };` block:

```text
build_synthesis_llm_request
build_transcript_analysis_llm_request
gem_input_cap
synthesis_stage_max_output_token_budget
transcript_analysis_max_output_tokens
transcript_analysis_stage_max_output_token_budget
transcript_analysis_stage_max_output_token_budget_for_control_preset
transcript_analysis_stage_max_prompt_token_budget
DETAILED_REPORT_CONTROL_PRESET
```

Also ensure the two new Gem builders are not imported through `super`. Add this exact import immediately after the existing `use super::super::run_store::{ ... };` block:

```rust
use super::super::stage_request_policy::{
    build_gem_analysis_part_llm_request, build_gem_analysis_part_repair_llm_request,
    build_synthesis_llm_request, build_transcript_analysis_llm_request, gem_input_cap,
    synthesis_stage_max_output_token_budget, transcript_analysis_max_output_tokens,
    transcript_analysis_stage_max_output_token_budget,
    transcript_analysis_stage_max_output_token_budget_for_control_preset,
    transcript_analysis_stage_max_prompt_token_budget, DETAILED_REPORT_CONTROL_PRESET,
};
```

Do not move or edit any existing test body. Do not edit either new Gem test body added in Step 2; only its function imports change ownership.

- [ ] **Step 10: Format and run the source contract for GREEN**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
npm.cmd run test -- src/lib/prompt-pack-stage-request-policy-contract.test.ts
```

Expected: Vitest runs 19 cases (one registration case, 15 parameterized function cases, one assets/structs case, one lifecycle-ownership case, and one dependency-boundary case), all pass. `git status --short` lists only the four allowed implementation files.

- [ ] **Step 11: Re-run both Gem characterization tests after the move**

Run:

```powershell
$tests = @(
    'prompt_packs::runtime::tests::gem_analysis_part_llm_request_preserves_part_and_frozen_input',
    'prompt_packs::runtime::tests::gem_analysis_part_repair_llm_request_preserves_attempt_and_repair_context'
)
foreach ($test in $tests) {
    cargo test --manifest-path src-tauri/Cargo.toml --lib $test -- --exact
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Expected: each command runs exactly one test and passes with the same test bodies used before extraction.

- [ ] **Step 12: Run the existing focused prompt and budget behavior tests**

Run:

```powershell
$tests = @(
    'prompt_packs::runtime::tests::transcript_analysis_llm_request_embeds_frozen_stage_input',
    'prompt_packs::runtime::tests::transcript_analysis_llm_request_uses_detailed_report_prompt_for_control_preset',
    'prompt_packs::runtime::tests::transcript_analysis_llm_request_describes_candidate_indexes_and_forbids_backend_refs',
    'prompt_packs::runtime::tests::synthesis_llm_request_describes_allowed_refs_and_forbids_direct_intermediate_refs',
    'prompt_packs::runtime::tests::transcript_analysis_output_budget_is_clamped_to_model_limit',
    'prompt_packs::runtime::tests::transcript_analysis_output_budget_comes_from_stage_runtime_config',
    'prompt_packs::runtime::tests::transcript_analysis_stage_max_prompt_token_budget_reads_runtime_config',
    'prompt_packs::runtime::tests::gem_input_budget_uses_lower_known_model_limit',
    'prompt_packs::runtime::tests::detailed_report_control_preset_uses_larger_transcript_analysis_output_budget',
    'prompt_packs::runtime::tests::synthesis_output_budget_comes_from_stage_runtime_config'
)
foreach ($test in $tests) {
    cargo test --manifest-path src-tauri/Cargo.toml --lib $test -- --exact
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Expected: every command runs exactly one test and passes. If a filter matches zero tests, treat that as a plan/test-name mismatch and correct the filter before proceeding rather than accepting an empty run.

- [ ] **Step 13: Run the complete runtime and Prompt Pack Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::runtime::tests
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: both filtered suites exit 0 with no failed tests.

- [ ] **Step 14: Run the complete Vitest and Rust suites**

Run:

```powershell
npm.cmd run test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: both complete suites exit 0 with no failed frontend, contract, Rust unit, integration, or doc tests.

- [ ] **Step 15: Verify formatting and all Rust targets with zero warnings**

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

- [ ] **Step 16: Review exact scope and commit**

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
    'src-tauri/src/prompt_packs/stage_request_policy.rs',
    'src/lib/prompt-pack-stage-request-policy-contract.test.ts'
)
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
"CHANGED=$($changed -join ',')"
"UNEXPECTED=$($unexpected -join ',')"
if ($changed.Count -ne 4 -or $unexpected.Count -ne 0) { exit 1 }
git add -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src-tauri/src/prompt_packs/stage_request_policy.rs `
    src/lib/prompt-pack-stage-request-policy-contract.test.ts
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git diff --cached --stat
git diff --cached -- src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src-tauri/src/prompt_packs/stage_request_policy.rs `
    src/lib/prompt-pack-stage-request-policy-contract.test.ts
git commit -m "refactor: extract prompt pack stage request policy"
git status --short --branch
```

Expected: the implementation commit contains exactly the private module registration, literal policy move, runtime/test import changes, two Gem characterization tests, and focused source contract. The worktree is clean after commit.
