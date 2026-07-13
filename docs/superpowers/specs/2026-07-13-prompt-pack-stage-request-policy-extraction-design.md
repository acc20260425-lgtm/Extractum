# Prompt Pack Stage Request Policy Extraction Design

**Status:** Proposed for implementation planning
**Date:** 2026-07-13

## Goal

Reduce the size and mixed responsibilities of
`src-tauri/src/prompt_packs/runtime.rs` by extracting pure LLM request
construction and token-budget policy into one focused private module. Preserve
all request IDs, prompt text, token limits, errors, public interfaces, and
runtime behavior.

## Current Problem

After the run-store extraction, `prompt_packs/runtime.rs` remains approximately
3,050 lines and combines two different responsibilities:

- orchestration: profile resolution, provider execution, cancellation,
  persistence, progress events, and lifecycle messages;
- policy: exact prompts, request envelopes, bundled stage-budget parsing, and
  model-limit clamping.

The policy functions are deterministic and already have focused tests, but
their large prompt constants and helper block obscure the execution flow.
Moving them behind a sibling-module boundary makes `runtime.rs` easier to read
without changing the pipeline.

## Selected Architecture

Add a private sibling module. This diagram is a fragment of the existing
`prompt_packs` tree:

```text
src-tauri/src/prompt_packs/
|-- mod.rs
|-- run_store.rs
|-- runtime.rs                  # Tauri and execution orchestration
|-- stage_request_policy.rs     # prompt construction and token budgets
`-- ...
```

Register it in `prompt_packs/mod.rs` as private
`mod stage_request_policy;`. Do not re-export the module or any of its
contents.

`TRANSCRIPT_ANALYSIS_STAGE_JSON` and `SYNTHESIS_STAGE_JSON` are
`include_str!` constants with paths resolved relative to their containing Rust
source file. `stage_request_policy.rs` remains in the same directory as
`runtime.rs`, so copy the relative paths unchanged. Moving the policy module
into a subdirectory is outside this design because it would require adjusting
and separately verifying those compile-time paths.

`runtime.rs` continues to determine when a stage runs and which provider,
profile, model, cancellation token, and persistence path it uses. It imports
only the policy functions required to build an `LlmChatRequest` or calculate
an input/output limit.

## Extraction Boundary

Move these constants and private data structures to
`stage_request_policy.rs`:

- `TRANSCRIPT_ANALYSIS_STAGE_JSON`;
- `SYNTHESIS_STAGE_JSON`;
- `DETAILED_REPORT_CONTROL_PRESET`;
- `STANDARD_VIDEO_SUMMARY_PROMPT`;
- `DETAILED_VIDEO_SUMMARY_PROMPT`;
- `StageRuntimeConfigAsset`;
- `StageRuntimeConfiguration`;
- `StageBudgetLimits`.

Move these functions and give them `pub(super)` visibility within the parent
`prompt_packs` module. `runtime` is their only current production consumer:

- `transcript_analysis_control_preset`;
- `build_transcript_analysis_llm_request`;
- `build_synthesis_llm_request`;
- `gem_part_request_suffix`;
- `gem_part_repair_request_suffix`;
- `gem_analysis_part_max_output_tokens`;
- `build_gem_analysis_part_llm_request`;
- `build_gem_analysis_part_repair_llm_request`;
- `build_json_repair_llm_request`;
- `transcript_analysis_stage_max_output_token_budget`;
- `transcript_analysis_stage_max_prompt_token_budget`;
- `transcript_analysis_stage_max_output_token_budget_for_control_preset`;
- `synthesis_stage_max_output_token_budget`;
- `transcript_analysis_max_output_tokens`;
- `gem_input_cap`.

The existing runtime test module imports
`DETAILED_REPORT_CONTROL_PRESET`, so that constant also uses `pub(super)`.
All other constants, configuration structs, and implementation helpers remain
private to `stage_request_policy.rs`, including:

- `transcript_analysis_summary_prompt`;
- `gem_part_output_budget`;
- `stage_max_prompt_token_budget`;
- `stage_max_output_token_budget`.

## Explicit Runtime Ownership

Keep these helpers in `runtime.rs`:

- `gem_part_phase`;
- `gem_part_started_message`.

They describe execution progress and user-visible lifecycle messages rather
than LLM request policy. The runtime continues to own all provider calls,
browser-run identity assembly, cancellation, retries, stage persistence,
terminal events, cleanup, and Tauri command adapters.

The request suffix helpers move because they are part of the stable request ID
format used by both request builders and browser-run discriminators. Runtime
imports them instead of duplicating the format.

## Dependencies and Data Flow

`stage_request_policy.rs` depends only on:

- `serde::Deserialize` and `serde_json` for bundled configuration and
  `controlPreset` parsing;
- `crate::error::{AppError, AppResult}`;
- `crate::llm::{LlmChatRequest, LlmMessage}`;
- `super::json_repair::JsonRepairStageExecutionRequest`;
- existing YouTube Summary request and Gem part types from
  `super::youtube_summary`.

The production flow remains:

```text
runtime resolves profile/model and stage context
    -> stage_request_policy calculates the effective budget
    -> stage_request_policy constructs LlmChatRequest
    -> runtime executes API or browser provider
    -> runtime persists provenance and results
```

The policy module must not depend on `AppHandle`, Tauri state, SQLx, provider
execution functions, cancellation tokens, or persistence modules.

## Prompt and Budget Compatibility

This is a structural refactor only. Preserve literally:

- every system and user prompt string, including whitespace, escaping,
  Unicode text, JSON examples, and Markdown instructions;
- request ID formats for transcript, synthesis, Gem, Gem repair, and JSON
  repair;
- `controlPreset` and `control_preset` fallback behavior;
- the `detailed_report` minimum output budget of 8,192;
- Gem per-part budgets of 4,096 or 8,192;
- stage asset parsing and positive-value validation;
- model output-limit clamping and input-limit conversion;
- all existing `AppError::internal` messages.

Do not reword or re-encode prompt constants while moving them. Do not replace
the bundled JSON parsing with new constants or cached state. Any prompt or
budget change requires a separate behavioral design.

## Error Handling

No new error layer is introduced. Bundled asset parse failures and missing or
non-positive budget fields continue to return the same `AppError::internal`
messages. Builders remain infallible where they are currently infallible.
Runtime continues to own provider, cancellation, and persistence errors.

## Tests

Keep the existing prompt and budget tests in `runtime::tests` for this slice.
Only their imports change to reference `stage_request_policy`. Moving the tests
would require separating shared runtime fixtures and would widen a mechanical
refactor without improving coverage.

The existing tests continue to cover:

- transcript request shape and frozen input;
- detailed-report prompt selection;
- forbidden backend-owned refs;
- synthesis allowed/forbidden refs;
- stage budget parsing and model clamping;
- input budget clamping.

Add two focused Rust regression tests before the move because the Gem builders
currently have no direct content assertions:

- the normal Gem request keeps its exact request-ID suffix, part value, frozen
  input, system role, and user role;
- the Gem repair request keeps its attempt-number suffix, parser error,
  original input, invalid provider output, system role, and user role.

Keep these new tests in `runtime::tests` with the other request-policy tests;
write them first against the current private functions through `use super::...`.
When the functions move, keep the test bodies and assertions unchanged and
change only their imports to `stage_request_policy`.

Add a focused raw-source contract that verifies:

- `prompt_packs/mod.rs` registers private `stage_request_policy`;
- the enumerated policy functions are defined in the policy module with
  `pub(super)` and are no longer defined in `runtime.rs`;
- prompt constants and budget configuration structs live in the policy module;
- `gem_part_phase` and `gem_part_started_message` remain in `runtime.rs`;
- the policy module does not import Tauri, SQLx, cancellation, or runtime.

The contract checks ownership markers, not exact prompt bodies or formatting.
Behavioral Rust tests remain the authority for request contents and budgets.

## Verification

Run, at minimum:

1. the new source contract;
2. focused existing prompt and budget Rust tests;
3. all `prompt_packs::runtime::tests`;
4. all `prompt_packs` Rust tests;
5. the complete Vitest suite;
6. the complete Rust suite;
7. `npm.cmd run check:rustfmt`;
8. `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` with zero
   warnings.

## Scope

Implementation changes are limited to:

- `src-tauri/src/prompt_packs/mod.rs`;
- `src-tauri/src/prompt_packs/runtime.rs`;
- new `src-tauri/src/prompt_packs/stage_request_policy.rs`;
- one focused raw-source contract under `src/lib/`.

Do not modify DTOs, migrations, prompt-pack JSON assets, frontend behavior,
dependencies, Tauri command registration, `docs/project.md`, or
`docs/value-registry.md`. No registered status, state, kind, mode, phase,
provider, or persisted value changes.

## Acceptance Criteria

1. The enumerated prompt, request-builder, and budget-policy items live in
   `stage_request_policy.rs` with the specified visibility.
2. `runtime.rs` retains execution orchestration and lifecycle messages but no
   longer defines the extracted policy block.
3. Prompt text, request IDs, budgets, errors, and public interfaces are
   unchanged.
4. Existing tests remain in `runtime::tests` with assertions unchanged.
5. Source ownership, focused behavior, full Vitest, full Rust, rustfmt, and
   all-target checks pass with zero warnings.
6. The implementation diff contains only the four files permitted by scope.

## Deferred Follow-ups

This slice does not extract provider execution, browser orchestration,
runtime-state management, lifecycle messages, or the runtime test module.
Those remain independent future designs.
