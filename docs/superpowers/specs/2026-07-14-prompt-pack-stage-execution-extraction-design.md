# Prompt Pack Stage Execution Extraction Design

**Status:** Proposed for implementation planning

**Date:** 2026-07-14

## Goal

Reduce the remaining stage-specific execution responsibilities in
`src-tauri/src/prompt_packs/runtime.rs` by moving the five request-preparation
and transport-invocation functions into one private sibling module. Preserve
all request, budget, provider, cancellation, event, persistence, error, and
public API behavior.

## Current Problem

After the run-store, stage-request-policy, run-control, and completion-transport
extractions, `prompt_packs/runtime.rs` is still approximately 1,928 lines. A
contiguous block of roughly 230 lines bridges two responsibilities that now
already have dedicated modules:

- `stage_request_policy` calculates stage budgets and builds LLM requests;
- `completion_transport` resolves provider model context and executes the
  prepared completion request.

The five bridge functions are cohesive but remain embedded in the command and
run-lifecycle module. This makes `runtime.rs` retain detailed knowledge of
every stage's budget, request shape, phase, message, and discriminator even
though the top-level orchestrator only needs to select a stage and await its
result.

This is an ownership refactor only. It does not redesign stage dispatch,
provider selection, prompt policy, completion transport, or lifecycle
orchestration.

## Considered Approaches

### 1. Extract only the five stage executors and Gem helpers (selected)

Move the five stage-specific functions and their two small Gem presentation
helpers to a private sibling module. Keep the
`YoutubeSummaryStageExecutionRequest` match in `runtime.rs`.

This creates a narrow boundary between orchestration and stage execution with
the smallest behavioral surface and a mechanically reviewable diff.

### 2. Extract the stage dispatcher as well

Move the request enum match together with the five executors and expose one
dispatch function to `runtime.rs`.

This would make `runtime.rs` smaller, but it would also move orchestration
control flow and closure-captured runtime state into the new module. The
additional abstraction is not required to establish ownership of stage
preparation, so it is rejected for this slice.

### 3. Extract runtime configuration and provider construction

Move `RunRuntimeProvider`, `RunRuntimeConfig`, `load_run_runtime_config`, and
construction of `RunCompletionRuntime` together with stage execution.

This would combine persisted runtime configuration, provider resolution, and
stage execution in one change. It is rejected because those responsibilities
have different callers, dependencies, and failure modes.

## Selected Architecture

Add one private sibling module:

```text
src-tauri/src/prompt_packs/       # relevant fragment
|-- completion_transport.rs
|-- mod.rs
|-- runtime.rs                    # commands, dispatch, and run lifecycle
|-- stage_execution.rs            # stage preparation and transport invocation
|-- stage_request_policy.rs
`-- ...
```

Register it in `prompt_packs/mod.rs` as:

```rust
mod stage_execution;
```

The module remains private. Expose only these functions within
`prompt_packs`, using `pub(super)` so sibling `runtime.rs` can call them:

- `run_transcript_analysis_stage_request`;
- `run_synthesis_stage_request`;
- `run_json_repair_stage_request`;
- `run_gem_analysis_part_stage_request`;
- `run_gem_analysis_part_repair_request`.

Move these private helpers with their only consumers:

- `gem_part_phase`;
- `gem_part_started_message`.

The helper functions remain private to `stage_execution.rs`.

`runtime.rs` imports the five stage executors and retains the existing
`match YoutubeSummaryStageExecutionRequest` dispatcher. No unified dispatcher,
trait, state object, or public API is introduced.

## Data Flow

The execution flow remains:

1. `runtime.rs` loads persisted run configuration and constructs
   `RunCompletionRuntime`.
2. `runtime.rs` creates the stage-executor closure and matches the existing
   `YoutubeSummaryStageExecutionRequest` variants.
3. The selected function in `stage_execution.rs` asks
   `RunCompletionRuntime::model_context()` for provider-specific model values.
4. It calls the same `stage_request_policy` functions to calculate budgets and
   build the same `LlmChatRequest`.
5. It populates every field of the same `StageCompletionRequest` explicitly.
6. It calls `RunCompletionRuntime::execute(...)` and returns the result without
   wrapping or transforming it.

The dispatcher continues to pass `AppHandle`, `SqlitePool`, the cloned
`RunCompletionRuntime`, the optional run cancellation token, and the concrete
stage request exactly as it does now.

## Extraction Boundary

Move to `stage_execution.rs`:

- the five `run_*_stage_request` functions listed above;
- `gem_part_phase`;
- `gem_part_started_message`;
- only the imports needed by those functions.

Keep in `runtime.rs`:

- all Tauri command adapters;
- `spawn_youtube_summary_execution` and
  `execute_youtube_summary_run`;
- the `YoutubeSummaryStageExecutionRequest` dispatcher;
- `RunRuntimeProvider`, `RunRuntimeConfig`, and runtime-config loading;
- construction of `RunCompletionRuntime`;
- Browser readiness and preflight handling;
- start, cancellation, update, delete, cleanup, and fixture behavior;
- terminal-event construction and emission.

Keep in `stage_request_policy.rs` all budget calculations, prompt assets,
request builders, suffix builders, and control-preset policy.

Keep in `completion_transport.rs` provider model-context resolution,
transport selection, API/Browser execution, transport events, cancellation,
result conversion, and Browser provenance persistence.

## Dependencies

`stage_execution.rs` may depend on:

- `sqlx::SqlitePool`;
- `tauri::AppHandle`;
- `tokio_util::sync::CancellationToken`;
- `super::completion_transport::{RunCompletionRuntime,
  StageCompletionRequest}`;
- the relevant builders and budget helpers from
  `super::stage_request_policy`;
- stage request and result/error types from `super::youtube_summary` and
  `super::json_repair`.

It must not depend on:

- `super::runtime`;
- Tauri command attributes or command adapters;
- database-pool lookup through `crate::db::get_pool`;
- runtime-config parsing or loading;
- profile resolution or Browser readiness/preflight logic;
- run lifecycle state, terminal-event emission, cleanup, or smoke fixtures.

Import cleanup is part of implementation. Imports used only by the moved block
leave `runtime.rs`; imports still required by commands, the dispatcher,
runtime construction, or tests remain. Compiler and rustfmt output are the
authority rather than a hand-maintained complete import list.

## Behavioral Compatibility

Move the seven function bodies statement-for-statement. Permitted edits are
limited to visibility, module paths, imports, indentation, and rustfmt output.
Do not change:

- the order of model-context lookup, budget calculation, request building, or
  transport execution;
- any stage name, phase, started message, request discriminator, or repair
  attempt value;
- source-snapshot handling;
- output-token calculations or control-preset branching;
- cloning or ownership semantics of request fields;
- cancellation token propagation;
- error or result types.

The stage dispatcher remains byte-for-byte equivalent apart from calling
imported sibling-module functions instead of functions defined later in the
same file.

No registered status, state, kind, mode, phase, provider, subtype, scope,
severity, event channel, persisted value, or wire value changes. Therefore
`docs/value-registry.md` does not change.

## Error Handling

The extraction introduces no new error, retry, timeout, fallback, log entry,
or context wrapper. Policy errors continue to return through the existing
`AppResult` conversion into `YoutubeSummaryStageExecutionError`. Transport
errors and cancellation outcomes pass through unchanged.

The new module does not catch errors or mutate run status. Top-level failure
marking and terminal-event behavior remain in `runtime.rs`.

## Tests

Existing Rust tests stay in their current modules. The extraction does not
move fixtures or add a provider mock: existing Prompt Pack execution tests
already exercise the five stage paths through the real dispatcher and
transport boundary.

Add a raw-source ownership contract:

```text
src/lib/prompt-pack-stage-execution-contract.test.ts
```

It verifies:

- `prompt_packs/mod.rs` registers private `stage_execution`;
- `stage_execution.rs` owns all five stage functions and both Gem helpers;
- those definitions no longer appear in `runtime.rs`;
- the five stage functions have `pub(super)` visibility while the two Gem
  helpers remain private;
- `stage_execution.rs` contains exactly five calls to `model_context()` and
  five calls to `execute()`;
- `runtime.rs` retains the five dispatcher calls;
- `stage_execution.rs` does not contain the stage request enum match, Tauri
  commands, runtime-config loading, preflight/readiness functions, cleanup, or
  smoke fixtures;
- `stage_execution.rs` does not depend on `super::runtime`.

Update the existing completion-transport source contract so its ownership
assertion reads `stage_execution.rs`, where the five `model_context()` and
`execute()` calls now live, instead of `runtime.rs`.

Update the existing stage-request-policy contract so it no longer requires
`gem_part_phase` and `gem_part_started_message` to remain in `runtime.rs`. It
continues to assert that those lifecycle-message helpers do not move into the
policy module; the new stage-execution contract becomes authoritative for
their positive ownership.

All source readers continue to normalize CRLF before matching.

## Verification

Run, at minimum:

1. the new stage-execution source contract in RED before extraction and GREEN
   afterward;
2. the updated completion-transport and stage-request-policy contracts;
3. focused Prompt Pack stage/execution Rust tests;
4. all Prompt Pack Rust tests;
5. the complete Vitest suite;
6. the complete Rust suite;
7. `npm.cmd run check:rustfmt`;
8. `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` with zero
   warnings.

## Scope

Implementation changes are limited to exactly these six files:

- `src-tauri/src/prompt_packs/mod.rs`;
- `src-tauri/src/prompt_packs/runtime.rs`;
- new `src-tauri/src/prompt_packs/stage_execution.rs`;
- new `src/lib/prompt-pack-stage-execution-contract.test.ts`;
- `src/lib/prompt-pack-completion-transport-contract.test.ts`;
- `src/lib/prompt-pack-stage-request-policy-contract.test.ts`.

Do not modify migrations, dependencies, assets, frontend behavior, Tauri
command registration, `docs/project.md`, or `docs/value-registry.md`.

## Acceptance Criteria

1. The five stage-specific preparation/execution functions live in private
   `stage_execution.rs` with sibling-only visibility.
2. The two Gem phase/message helpers live privately beside their only
   consumers.
3. `runtime.rs` retains top-level orchestration and the existing stage request
   dispatcher.
4. Request construction, budgets, model context, transport execution,
   cancellation, messages, identifiers, errors, and results are unchanged.
5. The new module has no reverse dependency on `runtime.rs` and owns no
   command, configuration, preflight, lifecycle, cleanup, or fixture logic.
6. The three source contracts agree on the new ownership boundary and support
   Windows line endings.
7. Focused and full tests, rustfmt, and all-target checks pass with zero
   warnings.
8. The implementation diff contains only the six files permitted by scope.

## Deferred Follow-ups

This slice does not extract the stage dispatcher, runtime-config loading,
provider construction, top-level run execution, Browser readiness/preflight,
terminal events, interrupted-run cleanup, Tauri commands, or dev fixtures.
Each requires a separate design if pursued later.
