# Gemini Browser Prompt-Pack Runtime Provider Design

Date: 2026-06-21

## Problem

YouTube Summary prompt-pack runs currently execute every LLM stage through the
API-backed LLM scheduler. The run stores `provider_profile_id` and `model`, the
runtime resolves an LLM profile, and each stage calls
`run_llm_collect_with_profile`.

Gemini Browser Provider now has a working single-prompt execution path, run
history, setup diagnostics, and a hardened answer extraction adapter. The next
step is to let a prompt-pack run use Gemini Browser as its completion runtime.

The key design question is whether this should be modeled as a new generic LLM
provider kind named `gemini_browser`, or as a prompt-pack runtime choice.

## Decision

Use an explicit prompt-pack runtime selector:

- `api`
- `gemini_browser`

Do not add `gemini_browser` to `ProviderKind`.

`ProviderKind` remains the API profile family for things that have an API key,
base URL, model list, and generic chat-completion transport. Gemini Browser is a
browser-side runtime with sidecar setup, Chrome/CDP state, login state,
best-effort cancellation, browser run history, and artifact handling. Treating it
as an LLM profile would make the profile UI and backend contracts carry many
fields that do not apply.

## Goals

- Let YouTube Summary prompt-pack runs execute transcript analysis, synthesis,
  and JSON repair stages through Gemini Browser Provider.
- Preserve the current API-backed path as the default.
- Store the selected runtime on each prompt-pack run for auditability and
  repeatable inspection.
- Reuse existing prompt construction, stage validation, run events, and final
  result persistence.
- Reuse Browser Provider send-single execution, run logging, setup config, and
  hardened result conversion.
- Keep failures conservative: partial-risk browser answers must fail the stage
  rather than silently becoming valid completions.

## Non-Goals

- No new `ProviderKind::GeminiBrowser`.
- No Browser Provider entry in the generic LLM profile CRUD flow.
- No model listing, API key validation, or OpenAI-compatible base URL behavior
  for Gemini Browser.
- No full graceful cancellation of an in-progress browser answer loop in this
  slice. Cancellation is best-effort where the current Browser Provider can
  stop the active sidecar run.
- No generalized runtime plugin framework for every future prompt pack. This
  design adds the runtime choice to the YouTube Summary prompt pack first.
- No change to the research adapter under `research/gemini_browser_adapter`
  except if later implementation tests reveal a protocol mismatch.

## Current Context

Relevant current backend boundaries:

- `src-tauri/src/llm/mod.rs` defines `ProviderKind` with `gemini` and
  `openai_compatible`; legacy `omniroute` parses as `openai_compatible`.
- `src-tauri/src/prompt_packs/runtime.rs` loads `provider_profile_id` and
  `model` from `prompt_pack_runs`, resolves the LLM profile, and dispatches
  prompt-pack stage requests to API-backed LLM calls.
- `src-tauri/src/prompt_packs/youtube_summary/execution.rs` already executes
  stages through an injected stage executor returning
  `youtube_summary::LlmCompletion`.
- `src-tauri/src/gemini_browser/commands.rs` exposes
  `gemini_bridge_send_single`, which validates a prompt, creates a queued
  Browser Provider run, starts the sidecar, records result history, and emits
  browser run events.
- `src-tauri/src/prompt_packs/gemini_browser_stage.rs` already converts
  `GeminiBrowserRunResult` into trusted completion text and rejects risky
  statuses such as timeout-latest partial answers.

These boundaries mean the prompt-pack runtime does not need to enter the generic
LLM provider/profile layer to use Browser Provider.

## Data Model

Add a new migration after `0009_prompt_pack_intermediate_entities_artifacts.sql`
that extends `prompt_pack_runs`:

- `runtime_provider TEXT NOT NULL DEFAULT 'api'`
- `browser_provider_config_json TEXT NULL`

`runtime_provider` uses the wire values `api` and `gemini_browser`.

Existing rows default to `api`. New rows persist the runtime chosen at start
time. `browser_provider_config_json` stores the Browser Provider config snapshot
used for the run when `runtime_provider = 'gemini_browser'`; it stays `NULL` for
API-backed runs.

Idempotency keeps its current behavior: if a repeated `client_request_id`
returns an existing run, the existing row's runtime wins and is not rewritten by
the repeat request.

## API and DTO Contract

Extend the prompt-pack start and preflight inputs in Rust and TypeScript:

- `runtimeProvider: "api" | "gemini_browser"`
- `browserProviderConfig?: GeminiBrowserProviderConfig | null`

Rust input structs should accept missing `runtime_provider` as `api` so older
frontends or saved invocations do not fail deserialization.

For `api` runs:

- `profileId` and `modelOverride` keep their current meaning.
- Preflight keeps resolving the selected profile and model limit.

For `gemini_browser` runs:

- `profileId` and `modelOverride` are not required for execution.
- Preflight still estimates prompt-pack input size, source readiness, skipped
  videos, and blocking source failures.
- `selected_model_input_limit` is `null` because Browser Provider does not
  expose a reliable model metadata API through the existing sidecar contract.
- The run uses `browserProviderConfig` if supplied, otherwise the Browser
  Provider backend defaults.

## Runtime Flow

The YouTube Summary runtime loads a `RunRuntimeConfig` instead of only
`RunLlmConfig`.

`RunRuntimeConfig` contains:

- `runtime_provider`
- API profile/model fields for `api`
- Browser Provider config for `gemini_browser`

Execution then uses one stage executor with two completion backends.

For `api`:

1. Resolve the effective LLM profile and model.
2. Build the same stage `LlmChatRequest` as today.
3. Schedule and execute the request through `LlmSchedulerState`.
4. Convert API response usage and latency into prompt-pack `LlmCompletion`.

For `gemini_browser`:

1. Build the same stage `LlmChatRequest` that the API path would have sent.
2. Convert chat messages into a single browser prompt.
3. Call a reusable Browser Provider send-single helper.
4. Convert `GeminiBrowserRunResult` with
   `gemini_browser_stage::browser_result_to_completion_text`.
5. Return prompt-pack `LlmCompletion` with completion text, no token counts, and
   latency measured around the Browser Provider call.

The prompt-pack stage validation stays unchanged. If Browser Provider returns
text that does not satisfy the stage schema or quality gates, the existing
prompt-pack stage fails or repairs exactly as the API path would.

## Browser Provider Reuse

Extract the body of `gemini_bridge_send_single` into a reusable
`send_single_prompt` backend helper inside
`src-tauri/src/gemini_browser/commands.rs`.

The helper should accept:

- `&AppHandle`
- `&GeminiBrowserState`
- run id
- prompt
- source
- artifact mode
- optional `GeminiBrowserProviderConfig`

The existing Tauri command delegates to this helper. Prompt-pack runtime calls
the same helper via `handle.state::<GeminiBrowserState>()`. The helper is
`pub(crate)` and is not a new Tauri command.

Prompt-pack browser run IDs should be deterministic and traceable, for example:

`prompt-pack-{run_id}-stage-{stage_run_id}`

The Browser Provider run source should include the pack and stage identity, for
example:

`prompt_pack:youtube_summary:{stage}:run:{run_id}:stage:{stage_run_id}`

Prompt-pack runtime should use reduced artifact mode unless a later product
choice introduces a UI control for full Browser Provider artifacts.

## Prompt Formatting

Browser Provider receives one plain prompt string. The formatter converts
`LlmChatRequest.messages` into stable labeled sections:

```text
System:
<system message content>

User:
<user message content>
```

If a request contains multiple messages of the same role, the formatter keeps
their order and repeats the role label. The formatter does not wrap the prompt in
Markdown fences and does not drop any content.

Unsupported roles should fail with a validation error rather than being silently
flattened into an ambiguous prompt.

## Cancellation Semantics

API runtime keeps the current cancellation behavior through
`PromptPackRunState`, scheduler request cancellation, and
`run_with_prompt_pack_run_cancellation`.

Browser runtime observes the same prompt-pack cancellation token before starting
a browser request and after the browser result returns. If cancellation is
requested while a browser request is active, runtime should call the existing
Browser Provider stop path as a best-effort interrupt and return a cancelled
stage outcome when control returns.

This design does not claim hard cancellation of the sidecar's in-browser answer
extraction loop. That remains a separate Browser Provider hardening task.

## Events and User Feedback

Prompt-pack UI continues to listen to prompt-pack run events. Browser Provider
events may still be emitted for the Browser Provider run history, but the
YouTube Summary UI should not need to subscribe to Gemini Browser-specific
events to understand prompt-pack progress.

Prompt-pack runtime should emit stage status messages that make the selected
runtime visible, such as:

- `Browser Provider request queued`
- `Running Browser Provider stage`
- `Browser Provider returned answer`

API path event text remains unchanged unless implementation discovers a shared
message helper that can be updated without changing meaning.

## Frontend UX

`YoutubeSummaryRunDialog.svelte` adds a compact runtime selector near the
current LLM controls:

- `API profile`
- `Gemini Browser`

When `API profile` is selected, the existing profile and model controls remain
active.

When `Gemini Browser` is selected:

- API profile/model controls are hidden or disabled.
- Browser Provider setup status is shown using existing setup diagnostics.
- The dialog sends the current Browser Provider config shape if that config is
  already available to the frontend; it does not add new Chrome/CDP setup fields
  in this slice.
- The start button is blocked only for known blocking setup failures; warning or
  manual-action states are surfaced but not disguised as API profile errors.

The frontend sends `runtimeProvider` in both preflight and start calls.

## Error Handling

Browser Provider execution failures map into prompt-pack stage failures with
actionable messages:

- empty prompt: validation failure before Browser Provider launch
- Browser Provider setup or sidecar failure: stage failure with provider message
- manual action required: stage failure with the Browser Provider message
- timeout-latest or partial-risk result: stage failure through
  `browser_result_to_completion_text`
- empty successful text: stage failure through `browser_result_to_completion_text`
- prompt-pack output validation failure: existing stage validation failure

The terminal prompt-pack run state remains the source of truth for the YouTube
Summary run. Browser Provider run history is supporting evidence and debug
context.

## Testing Strategy

Backend tests:

- runtime provider enum parsing/defaulting accepts missing value as `api`
- migration-backed run creation stores `runtime_provider`
- browser config JSON round-trips through `prompt_pack_runs`
- API runtime path still resolves profile/model and uses the scheduler
- Browser runtime path does not require an LLM profile
- chat-message formatter preserves role order and content
- unsupported browser prompt role fails validation
- Browser Provider `ok` result becomes prompt-pack `LlmCompletion`
- `timeout_latest`, `ready`, failed, manual-action, and empty-text results fail
  the stage
- prompt-pack cancellation before Browser Provider launch returns cancelled
- prompt-pack cancellation during Browser Provider launch calls the best-effort
  Browser Provider stop path

Frontend tests:

- `PreflightYoutubeSummaryRunInput` and `StartYoutubeSummaryRunInput` include
  `runtimeProvider`
- API mode sends the current profile/model payload
- Gemini Browser mode sends browser config and does not require profile/model
- dialog source contract includes the runtime selector and browser setup state

Verification commands for the implementation plan should include:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
npm.cmd run test
npm.cmd run check
```

## Open Risks

- Browser Provider cannot report reliable token usage or model limits today.
  Prompt-pack run history should show missing token counts rather than invented
  estimates.
- Browser cancellation remains best-effort until the sidecar protocol supports
  a stronger in-answer abort.
- Browser Provider queue and prompt-pack scheduler queue are separate systems.
  The implementation must avoid showing API scheduler queue positions for
  Browser Provider stages.
- Gemini web UI changes can still break answer extraction. The prompt-pack path
  must rely on the existing hardened converter and fail closed.

## Acceptance Criteria

- Existing API-backed YouTube Summary runs behave as before.
- New YouTube Summary runs can be started with `runtimeProvider =
  "gemini_browser"`.
- Browser-backed runs persist the selected runtime and browser config snapshot.
- Browser-backed stages are visible in prompt-pack progress and Gemini Browser
  run history.
- Browser-backed successful answers flow through the same prompt-pack
  validation and persistence as API-backed answers.
- Browser-backed partial or risky answers fail closed with clear diagnostics.
- `ProviderKind` remains limited to API profile families.
