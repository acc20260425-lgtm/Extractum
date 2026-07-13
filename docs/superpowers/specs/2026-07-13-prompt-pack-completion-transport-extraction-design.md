# Prompt Pack Completion Transport Extraction Design

**Status:** Proposed for implementation planning
**Date:** 2026-07-13

## Goal

Reduce the remaining provider-transport responsibilities in
`src-tauri/src/prompt_packs/runtime.rs` by extracting API and Gemini Browser
stage-completion execution behind one private transport interface. Preserve
all request construction, scheduling, cancellation, event, persistence,
error, and public API behavior.

## Current Problem

After the run-store, stage-request-policy, and run-control extractions,
`prompt_packs/runtime.rs` still owns three different concerns in the same
section:

- stage-specific request preparation and token-budget policy;
- selection between the API and Gemini Browser providers;
- the low-level mechanics of executing either provider transport.

Each of the five stage functions currently performs provider dispatch twice:
first to derive the profile/model context used by the prompt builder, and then
again to select `run_api_llm_request` or `run_browser_llm_request`. The
transport block also owns browser prompt conversion, browser job identity,
cancellation, result conversion, and provenance persistence. This repetition
makes it harder to review whether all stages preserve the same provider
semantics.

This is an ownership problem, not a request to change provider behavior. The
existing browser readiness/preflight gate, runtime-config loading, prompt
builders, and stage-specific budgets remain separate concerns.

## Considered Approaches

### 1. Extract only the existing free functions

Move the API and Browser helper functions to a sibling module while retaining
both provider `match` expressions in each stage function.

This produces the smallest mechanical diff, but leaves duplicated dispatch
and does not create one reviewable transport boundary. It is rejected because
the central problem would remain in `runtime.rs`.

### 2. Private transport module with one enum interface (selected)

Move the provider enum and transport mechanics to a private sibling module.
Give the enum a model-context method and one execution method. Stage functions
continue to own semantic request preparation but no longer know how either
transport runs.

This removes repeated dispatch while preserving the existing two-provider
model and avoiding a public abstraction.

### 3. Trait-based provider hierarchy

Introduce provider traits and separate API/Browser implementations.

This would offer more extensibility, but the application currently has only
two closed provider variants. Dynamic dispatch, trait lifetimes, and test
doubles would expand the change without serving the behavior-preserving goal.
It is rejected for this slice.

## Selected Architecture

Add this private sibling module:

```text
src-tauri/src/prompt_packs/
|-- completion_transport.rs   # provider selection and completion execution
|-- dto.rs                    # shared DTOs and run-event channel constant
|-- mod.rs
|-- run_control.rs
|-- run_store.rs
|-- runtime.rs                # commands and stage orchestration
|-- stage_request_policy.rs
`-- ...
```

Register it in `prompt_packs/mod.rs` as exactly:

```rust
mod completion_transport;
```

The module stays private. Its internal interface consists of:

- `RunCompletionRuntime::{Api, GeminiBrowser}`;
- `CompletionModelContext`;
- `StageCompletionRequest`;
- `RunCompletionRuntime::model_context()`;
- `RunCompletionRuntime::execute(...)`.

Give only the visibility needed by the sibling `runtime` module, using
`pub(super)` rather than widening these types into public API.

The enum, both request/context structures, their fields consumed or populated
by `runtime.rs`, and the two methods are `pub(super)`. The methods have these
semantic shapes:

```rust
pub(super) async fn model_context(&self) -> AppResult<CompletionModelContext>;

pub(super) async fn execute(
    self,
    handle: AppHandle,
    pool: SqlitePool,
    request: StageCompletionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError>;
```

`model_context()` is asynchronous because the existing backend output-limit
resolution is asynchronous. Do not add constructors or accessors merely to
hide fields inside this private module boundary.

`RunCompletionRuntime` continues to contain the resolved API profile and model
override for `Api`, or the optional Browser Provider configuration for
`GeminiBrowser`. Construction remains in `runtime.rs` after persisted runtime
configuration is loaded. Provider parsing and `load_run_runtime_config` do not
move.

`CompletionModelContext` carries the three values required before prompt
construction:

- `profile_id: Option<String>`;
- `model_override: Option<String>`;
- `model_output_limit: Option<i64>`.

For the API variant, `model_context()` resolves the effective model and its
backend output limit exactly as the current first provider match does. For the
Browser variant, all three fields are `None`.

`StageCompletionRequest` carries the already-built `LlmChatRequest` plus every
semantic value currently passed to the two transport functions:

- run and stage-run IDs;
- optional source-snapshot ID;
- stage name;
- phase;
- started message;
- optional repair-attempt number;
- optional request discriminator;
- run cancellation token.

The structure has no defaults. Every stage function must populate every field
explicitly so omissions remain visible in review. Store
`request_discriminator` as `Option<String>`; this avoids borrowing a temporary
suffix across an async boundary while preserving the exact discriminator
content.

`execute()` also receives the existing `AppHandle` and `SqlitePool` required
by the transports. It dispatches internally to the API or Browser runner and
returns the existing
`Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError>`.

## Event Constant Ownership

Move the Rust constant:

```rust
pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";
```

from `runtime.rs` to `dto.rs`. Both the completion transport and runtime event
emission depend on this channel name, so placing it with the shared event DTO
avoids a reverse dependency from the transport module to orchestration.

Preserve the existing Rust path with a public re-export from `runtime.rs`:

```rust
pub use super::dto::PROMPT_PACK_RUN_EVENT;
```

The string value, frontend listener, emitted payloads, ordering, and public
path do not change. This move does not introduce a new registered value and
does not require a `docs/value-registry.md` update.

## Extraction Boundary

Move to `completion_transport.rs`:

- `RunCompletionRuntime`;
- API model-context resolution currently repeated by the five stage
  functions;
- `llm_chat_request_to_browser_prompt`;
- `browser_run_id_for_stage`;
- `browser_run_source_for_stage`;
- `browser_stage_completion_from_result`;
- `run_api_llm_request`;
- `run_browser_llm_request`;
- `run_browser_stage_result_with_cancellation`;
- `persist_browser_stage_provenance`;
- `non_empty_string`;
- the new `CompletionModelContext` and `StageCompletionRequest`;
- the unified `RunCompletionRuntime::model_context()` and `execute()` methods.

Keep in `runtime.rs`:

- `RunRuntimeProvider`, `RunRuntimeConfig`, and
  `load_run_runtime_config`;
- construction of `RunCompletionRuntime`;
- all five stage-specific request functions;
- stage budgets, control presets, and prompt builders;
- Tauri commands and top-level run execution;
- Browser readiness and preflight checks;
- terminal-event construction and emission;
- interrupted-run cleanup and dev fixtures.

The five stage-specific functions are:

- `run_transcript_analysis_stage_request`;
- `run_synthesis_stage_request`;
- `run_json_repair_stage_request`;
- `run_gem_analysis_part_stage_request`;
- `run_gem_analysis_part_repair_request`.

Each follows one common flow after the extraction:

1. call `completion_runtime.model_context()`;
2. calculate the existing stage-specific output budget;
3. build the same `LlmChatRequest` with the returned model context;
4. construct an explicit `StageCompletionRequest`;
5. call `completion_runtime.execute(...)`.

No stage function retains its own `Api`/`GeminiBrowser` dispatch.

## Behavioral Compatibility

### API transport

Preserve exactly:

- use of the existing `LlmSchedulerState` and `LlmRequestMetadata`;
- request kind, priority, owner-run ID, profile ID, provider, and request ID;
- queued and started event payloads and their order;
- repair-specific queue text;
- cooperative cancellation through
  `run_with_prompt_pack_run_cancellation` and the scheduler control;
- latency measurement;
- response-text and token-usage mapping;
- mapping `LlmRequestError::Cancelled` to
  `YoutubeSummaryStageExecutionError::Cancelled`;
- mapping `LlmRequestError::Failed` without extra wrapping.

### Gemini Browser transport

Preserve exactly:

- the early cancellation check before prompt conversion and execution;
- browser job ID and source formatting, including repair attempts and Gem
  discriminators;
- conversion of supported `system` and `user` messages and rejection of any
  other role;
- empty-prompt validation;
- queued and started event payloads and their order;
- the existing Browser Provider request and configuration;
- cancellation by browser job ID and the post-result cancellation check;
- result-to-completion conversion and absent token counts;
- provenance extraction and SQL persistence before returning the completion;
- existing cancellation and failure mappings.

Do not move the Browser readiness gate into the transport. `execute()` assumes
the caller reached it through the already-established orchestration path.

For the provenance `updated_at` value, call
`crate::time::now_rfc3339_utc()` directly from the extracted module. This is
the same implementation used by runtime's private `now_string()` helper and
avoids a reverse dependency on `runtime.rs`. Do not move or duplicate
`now_string()`.

## Error Handling

This slice introduces no new error type, logging, fallback, retry, timeout, or
context wrapper. The transport interface returns the existing stage execution
error unchanged.

Validation failures from Browser prompt conversion remain `AppError`
validation errors and continue through the same existing conversion path.
Database failures from provenance persistence remain database errors. A
provenance write still completes before a successful Browser completion is
returned.

## Tests

Keep the existing behavioral tests in `runtime::tests`. Their bodies and
assertions remain in place; update only imports so they reach moved helpers
through `completion_transport`. This avoids combining the ownership refactor
with test-fixture relocation.

Existing tests remain the authority for:

- Browser cancellation before execution and while a request is active;
- browser prompt formatting and validation;
- browser run ID and source formatting;
- Browser result conversion;
- provenance persistence;
- stage behavior and Browser readiness/preflight behavior.

Add a small focused Rust assertion for model context:

- `GeminiBrowser` returns no profile ID, model override, or output limit;
- `Api` retains its resolved profile ID and model override and calculates the
  same model output limit as before.

The real stage and Prompt Pack tests, rather than a mock provider hierarchy,
continue to cover provider dispatch.

Add one raw-source contract at:

```text
src/lib/prompt-pack-completion-transport-contract.test.ts
```

The contract verifies:

- `prompt_packs/mod.rs` registers private `completion_transport`;
- the new module owns `RunCompletionRuntime`, `CompletionModelContext`,
  `StageCompletionRequest`, `model_context`, `execute`, both provider runners,
  and the listed Browser helpers;
- those definitions no longer remain in `runtime.rs`;
- all five stage functions call `model_context()` and `execute()` and contain
  no direct `RunCompletionRuntime::Api` or `GeminiBrowser` dispatch;
- `dto.rs` contains the single definition of the Rust
  `PROMPT_PACK_RUN_EVENT` constant;
- `runtime.rs` publicly re-exports that constant;
- `completion_transport.rs` contains no Tauri commands, start/preflight or
  readiness functions, lifecycle cleanup, interrupted-run cleanup, or dev
  fixtures.

The source contract checks ownership and compatibility boundaries. Rust tests
remain authoritative for behavior.

## Verification

Run, at minimum:

1. the new source contract in RED before extraction and GREEN afterward;
2. focused completion-transport and moved-helper Rust tests;
3. all `prompt_packs::runtime::tests`;
4. all Prompt Pack Rust tests;
5. the complete Vitest suite;
6. the complete Rust suite;
7. `npm.cmd run check:rustfmt`;
8. `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` with zero
   warnings.

## Scope

Implementation changes are limited to exactly these five files:

- `src-tauri/src/prompt_packs/mod.rs`;
- `src-tauri/src/prompt_packs/dto.rs`;
- `src-tauri/src/prompt_packs/runtime.rs`;
- new `src-tauri/src/prompt_packs/completion_transport.rs`;
- new `src/lib/prompt-pack-completion-transport-contract.test.ts`.

Do not modify migrations, dependencies, Prompt Pack assets, frontend behavior,
command registration, `docs/project.md`, or `docs/value-registry.md`. No
registered status, state, kind, mode, phase, provider, subtype, scope,
severity, persisted value, event channel, or wire value changes.

## Acceptance Criteria

1. Provider selection and low-level API/Browser completion execution live in
   private `completion_transport.rs` behind `model_context()` and `execute()`.
2. The five stage-specific functions retain budget and prompt construction but
   contain no provider dispatch.
3. API scheduling, events, cancellation, timing, usage, and error behavior are
   unchanged.
4. Browser prompt conversion, identity, events, cancellation, result mapping,
   provenance order, and error behavior are unchanged.
5. Runtime configuration, Browser readiness/preflight, commands, lifecycle
   cleanup, and fixtures remain in `runtime.rs`.
6. The event-channel string and existing Rust path remain unchanged, with one
   constant definition in `dto.rs` and a runtime re-export.
7. Existing behavioral test bodies remain in `runtime::tests`; only imports
   change, plus the focused model-context assertion and source contract.
8. Focused and full tests, rustfmt, and all-target checks pass with zero
   warnings.
9. The implementation diff contains only the five files permitted by scope.

## Deferred Follow-ups

This slice does not extract runtime-config loading, Browser readiness or
preflight, stage-specific prompt/budget policy, terminal-event construction,
interrupted-run cleanup, Tauri commands, or dev fixtures. It does not add a
provider trait hierarchy or a mock transport layer. Any further decomposition
requires a separate design.
