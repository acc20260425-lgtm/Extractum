# Prompt Pack Completion Transport Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract Prompt Pack API and Gemini Browser stage-completion execution from `runtime.rs` into one private `completion_transport.rs` module without changing provider behavior, events, cancellation, persistence, errors, or public paths.

**Architecture:** Add private `prompt_packs::completion_transport` containing the closed `RunCompletionRuntime` enum, provider model context, an explicit stage request, and one `execute` dispatch method. Keep runtime-config loading and all five stage-specific budget/prompt builders in `runtime.rs`; move the shared event constant to `dto.rs` and preserve its runtime path through a public re-export.

**Tech Stack:** Rust 2021, Tauri 2, Tokio, SQLx/SQLite, SvelteKit/TypeScript, Vitest raw-source contracts.

## Global Constraints

- Implement the approved design in `docs/superpowers/specs/2026-07-13-prompt-pack-completion-transport-extraction-design.md` at or after commit `bcda8bd8`.
- Modify exactly five implementation files: `src-tauri/src/prompt_packs/mod.rs`, `src-tauri/src/prompt_packs/dto.rs`, `src-tauri/src/prompt_packs/runtime.rs`, new `src-tauri/src/prompt_packs/completion_transport.rs`, and new `src/lib/prompt-pack-completion-transport-contract.test.ts`.
- Register exactly private `mod completion_transport;`; do not expose or re-export the module itself.
- Keep `RunRuntimeProvider`, `RunRuntimeConfig`, `load_run_runtime_config`, construction of `RunCompletionRuntime`, all five stage functions, budgets, prompt builders, commands, preflight/readiness, terminal-event emission, interrupted-run cleanup, and dev fixtures in `runtime.rs`.
- Give the transport enum, context/request structures, fields shared with runtime, `model_context`, and `execute` only `pub(super)` visibility.
- Keep `StageCompletionRequest` explicit with no `Default`; store `request_discriminator` as owned `Option<String>`.
- Preserve both transport bodies statement-for-statement except for imports/module paths, indentation, request-field destructuring, the event-constant path, and direct use of `crate::time::now_rfc3339_utc()`.
- Preserve direct queued/started `handle.emit` calls; do not route them through `emit_prompt_pack_run_event` or `PromptPackRunState::apply_event`.
- Preserve scheduler metadata, event payloads/order/text, cancellation order, browser identity/source, prompt conversion, result mapping, provenance-before-return order, errors, and all wire/persisted values.
- Move the single `PROMPT_PACK_RUN_EVENT` definition to `dto.rs`; preserve `prompt_packs::runtime::PROMPT_PACK_RUN_EVENT` with `pub use`, accepting the additional `prompt_packs::dto::PROMPT_PACK_RUN_EVENT` path.
- Keep existing behavioral test bodies and assertions in `runtime::tests`; change only imports. Add only the focused model-context tests inside `completion_transport.rs`.
- Do not add dependencies, traits, mock transports, migrations, registered values, logging, retries, timeouts, fallbacks, or new error wrapping.
- Do not modify `docs/project.md` or `docs/value-registry.md`; no behavior or registered value changes.
- Preserve unrelated user changes and require a clean worktree before starting.

---

### Task 1: Extract the Prompt Pack Completion Transport

**Files:**
- Create: `src-tauri/src/prompt_packs/completion_transport.rs`
- Create: `src/lib/prompt-pack-completion-transport-contract.test.ts`
- Modify: `src-tauri/src/prompt_packs/mod.rs` near the private sibling-module declarations
- Modify: `src-tauri/src/prompt_packs/dto.rs` near the module imports and DTO declarations
- Modify: `src-tauri/src/prompt_packs/runtime.rs` imports, transport block, five stage functions, and `mod tests` imports

**Interfaces:**
- Consumes: `ResolvedLlmProfile`, `LlmChatRequest`, `LlmSchedulerState`, `SqlitePool`, `AppHandle`, `CancellationToken`, `run_with_prompt_pack_run_cancellation`, Browser Provider execution, Prompt Pack completion/error types, and `PROMPT_PACK_RUN_EVENT` from `dto`.
- Produces: private module `prompt_packs::completion_transport`; `pub(super) enum RunCompletionRuntime`; `pub(super) struct CompletionModelContext`; `pub(super) struct StageCompletionRequest`; async `RunCompletionRuntime::model_context(&self)` and `RunCompletionRuntime::execute(self, AppHandle, SqlitePool, StageCompletionRequest)`.
- Preserves: five stage-specific request functions in `runtime.rs`, existing runtime construction of the enum, existing runtime tests, the runtime event-constant path, and all API/Browser behavior.

- [ ] **Step 1: Verify clean-tree, approved-spec, formatting, and dispatch baselines**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git merge-base --is-ancestor bcda8bd8 HEAD
$approvedSpecPresent = $LASTEXITCODE -eq 0
$runtime = Get-Content -Raw 'src-tauri/src/prompt_packs/runtime.rs'
$dispatchMatches = [regex]::Matches($runtime, 'match\s+&?completion_runtime\b').Count
$eventDefinitions = [regex]::Matches($runtime, 'pub const PROMPT_PACK_RUN_EVENT').Count
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_PRESENT=$approvedSpecPresent"
"DISPATCH_MATCH_COUNT=$dispatchMatches"
"RUNTIME_EVENT_DEFINITION_COUNT=$eventDefinitions"
if (
    $status.Count -ne 0 -or
    -not $approvedSpecPresent -or
    $dispatchMatches -ne 10 -or
    $eventDefinitions -ne 1
) { exit 1 }
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: clean tree, `APPROVED_SPEC_PRESENT=True`, `DISPATCH_MATCH_COUNT=10`, `RUNTIME_EVENT_DEFINITION_COUNT=1`, and rustfmt exits 0. The ten matches are the two current provider matches in each of the five stage functions.

- [ ] **Step 2: Add the failing source-ownership and compatibility contract**

Create `src/lib/prompt-pack-completion-transport-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import completionTransportSource from "../../src-tauri/src/prompt_packs/completion_transport.rs?raw";
import dtoSource from "../../src-tauri/src/prompt_packs/dto.rs?raw";
import promptPacksModuleSource from "../../src-tauri/src/prompt_packs/mod.rs?raw";
import runtimeSource from "../../src-tauri/src/prompt_packs/runtime.rs?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");
const matches = (source: string, pattern: RegExp) => source.match(pattern) ?? [];

describe("Prompt Pack completion transport ownership", () => {
  it("registers a private completion_transport sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod completion_transport;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod completion_transport;/);
  });

  it("moves the provider transport interface out of runtime", () => {
    const transport = normalized(completionTransportSource);
    const runtime = normalized(runtimeSource);

    expect(transport).toMatch(/^pub\(super\) enum RunCompletionRuntime\s*\{/m);
    expect(transport).toMatch(/^pub\(super\) struct CompletionModelContext\s*\{/m);
    expect(transport).toMatch(/^pub\(super\) struct StageCompletionRequest\s*\{/m);
    expect(transport).toMatch(/pub\(super\) async fn model_context\s*\(/);
    expect(transport).toMatch(/pub\(super\) async fn execute\s*\(/);
    expect(transport).toMatch(/async fn run_api_llm_request\s*\(/);
    expect(transport).toMatch(/async fn run_browser_llm_request\s*\(/);
    const movedHelpers = [
      "llm_chat_request_to_browser_prompt",
      "browser_run_id_for_stage",
      "browser_run_source_for_stage",
      "browser_stage_completion_from_result",
      "run_browser_stage_result_with_cancellation",
      "persist_browser_stage_provenance",
      "non_empty_string",
    ];
    for (const helper of movedHelpers) {
      expect(transport).toMatch(new RegExp(`(?:async\\s+)?fn\\s+${helper}\\b`));
      expect(runtime).not.toMatch(new RegExp(`(?:async\\s+)?fn\\s+${helper}\\b`));
    }
    expect(transport).toContain("resolve_model_output_token_limit_for_backend");
    expect(runtime).not.toMatch(/^enum RunCompletionRuntime\s*\{/m);
    expect(runtime).not.toMatch(/async fn run_api_llm_request\s*\(/);
    expect(runtime).not.toMatch(/async fn run_browser_llm_request\s*\(/);
  });

  it("removes all five stage-level provider matches", () => {
    const runtime = normalized(runtimeSource);

    expect(matches(runtime, /match\s+&?completion_runtime\b/g)).toHaveLength(0);
    expect(
      matches(runtime, /completion_runtime\.model_context\(\)\.await\?/g),
    ).toHaveLength(5);
    expect(
      matches(runtime, /completion_runtime\s*\.execute\s*\(/g),
    ).toHaveLength(5);
  });

  it("keeps one event constant definition and the runtime compatibility path", () => {
    const dto = normalized(dtoSource);
    const runtime = normalized(runtimeSource);
    const transport = normalized(completionTransportSource);
    const combined = [dto, runtime, transport].join("\n");

    expect(
      matches(combined, /pub const PROMPT_PACK_RUN_EVENT\s*:/g),
    ).toHaveLength(1);
    expect(dto).toContain(
      'pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";',
    );
    expect(runtime).toMatch(
      /^pub use super::dto::PROMPT_PACK_RUN_EVENT;$/m,
    );
  });

  it("preserves direct transport events and the repair queue text", () => {
    const transport = normalized(completionTransportSource);

    expect(transport).toContain("JSON repair queued at position {position}");
    expect(
      matches(transport, /\.emit\(\s*PROMPT_PACK_RUN_EVENT,/g),
    ).toHaveLength(4);
    expect(transport).not.toContain("emit_prompt_pack_run_event");
    expect(transport).not.toContain("apply_event");
  });

  it("keeps orchestration and lifecycle responsibilities out of transport", () => {
    const transport = normalized(completionTransportSource);
    const forbidden = [
      "#[tauri::command]",
      "start_youtube_summary_run",
      "preflight_youtube_summary_run",
      "browser_runtime_start_blocking_failure",
      "cleanup_interrupted_prompt_pack_runs",
      "seed_prompt_pack_cancellation_smoke_fixture",
      "clear_prompt_pack_cancellation_smoke_fixture",
      "load_run_runtime_config",
    ];

    for (const marker of forbidden) {
      expect(transport).not.toContain(marker);
    }
  });
});
```

Expected: the contract normalizes CRLF, checks exact ownership, freezes the `10 -> 0` stage-dispatch transition, preserves one event constant and four direct transport emits, and forbids orchestration leakage.

- [ ] **Step 3: Run the source contract to verify RED**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-completion-transport-contract.test.ts
```

Expected: FAIL during Vite module resolution because `src-tauri/src/prompt_packs/completion_transport.rs` does not exist. This is the intended RED, not a Vitest infrastructure failure.

- [ ] **Step 4: Register the private module and move the event constant**

In `src-tauri/src/prompt_packs/mod.rs`, add the private module in alphabetical order immediately before `pub mod dto;`:

```rust
mod completion_transport;
```

In `src-tauri/src/prompt_packs/dto.rs`, add the single shared definition before the first DTO type:

```rust
pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";
```

In `runtime.rs`, replace the old constant definition with this re-export adjacent to the other `super` imports:

```rust
pub use super::dto::PROMPT_PACK_RUN_EVENT;
```

Do not change the string, frontend listener, event payloads, or the existing runtime path.

- [ ] **Step 5: Create the transport interface and model-context implementation**

Create `src-tauri/src/prompt_packs/completion_transport.rs` with the exact visibility and field surface below, followed by the moved helpers from Step 6:

```rust
use std::future::Future;
use std::time::Instant;

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

use super::dto::{PromptPackRunEvent, PROMPT_PACK_RUN_EVENT};
use super::run_control::run_with_prompt_pack_run_cancellation;
use super::youtube_summary::{
    LlmCompletion as PromptPackLlmCompletion, YoutubeSummaryStageExecutionError,
};
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_effective_model, resolve_model_output_token_limit_for_backend,
    run_llm_collect_with_profile, LlmChatRequest, LlmRequestError, LlmRequestKind,
    LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState, ResolvedLlmProfile,
};

#[derive(Clone)]
pub(super) enum RunCompletionRuntime {
    Api {
        profile: ResolvedLlmProfile,
        model_override: Option<String>,
    },
    GeminiBrowser {
        browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    },
}

pub(super) struct CompletionModelContext {
    pub(super) profile_id: Option<String>,
    pub(super) model_override: Option<String>,
    pub(super) model_output_limit: Option<i64>,
}

pub(super) struct StageCompletionRequest {
    pub(super) llm_request: LlmChatRequest,
    pub(super) run_id: i64,
    pub(super) stage_run_id: i64,
    pub(super) source_snapshot_id: Option<i64>,
    pub(super) stage_name: String,
    pub(super) phase: &'static str,
    pub(super) started_message: &'static str,
    pub(super) repair_attempt_number: Option<i64>,
    pub(super) request_discriminator: Option<String>,
    pub(super) run_cancellation_token: Option<CancellationToken>,
}

impl RunCompletionRuntime {
    pub(super) async fn model_context(&self) -> AppResult<CompletionModelContext> {
        match self {
            Self::Api {
                profile,
                model_override,
            } => {
                let effective_model =
                    resolve_effective_model(profile, model_override.as_deref())?;
                let model_output_limit =
                    resolve_model_output_token_limit_for_backend(profile, &effective_model).await;
                Ok(CompletionModelContext {
                    profile_id: Some(profile.profile_id.clone()),
                    model_override: model_override.clone(),
                    model_output_limit,
                })
            }
            Self::GeminiBrowser { .. } => Ok(CompletionModelContext {
                profile_id: None,
                model_override: None,
                model_output_limit: None,
            }),
        }
    }

    pub(super) async fn execute(
        self,
        handle: AppHandle,
        pool: SqlitePool,
        request: StageCompletionRequest,
    ) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
        match self {
            Self::Api { profile, .. } => run_api_llm_request(handle, profile, request).await,
            Self::GeminiBrowser {
                browser_provider_config,
            } => {
                run_browser_llm_request(
                    handle,
                    pool,
                    browser_provider_config,
                    request,
                )
                .await
            }
        }
    }
}
```

Expected: the private module exposes only the sibling-visible closed enum, context, explicit request, and the two methods required by runtime.

- [ ] **Step 6: Move the API and Browser transport helpers statement-for-statement**

Move these complete definitions from `runtime.rs` into `completion_transport.rs`, preserving their order and bodies:

```text
llm_chat_request_to_browser_prompt
browser_run_id_for_stage
browser_run_source_for_stage
browser_stage_completion_from_result
run_api_llm_request
run_browser_llm_request
run_browser_stage_result_with_cancellation
persist_browser_stage_provenance
non_empty_string
```

For the six helpers used by `runtime::tests`, change only the declaration
visibility from private to `pub(super)`: `llm_chat_request_to_browser_prompt`,
`browser_run_id_for_stage`, `browser_run_source_for_stage`,
`browser_stage_completion_from_result`,
`run_browser_stage_result_with_cancellation`, and
`persist_browser_stage_provenance`.

Keep the two provider runners and `non_empty_string` private. Change only the runner signatures and begin each body with the following mechanical destructuring.

For API:

```rust
async fn run_api_llm_request(
    handle: AppHandle,
    profile: ResolvedLlmProfile,
    request: StageCompletionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let StageCompletionRequest {
        llm_request,
        run_id,
        stage_run_id,
        source_snapshot_id,
        stage_name,
        phase,
        started_message,
        run_cancellation_token,
        ..
    } = request;
    let request_id = llm_request.request_id.clone();
}
```

The shown `let request_id` is the first existing statement. Retain it once and
retain every following statement through the function's closing brace exactly
as currently written.

For Browser:

```rust
async fn run_browser_llm_request(
    handle: AppHandle,
    pool: SqlitePool,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    request: StageCompletionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let StageCompletionRequest {
        llm_request,
        run_id,
        stage_run_id,
        source_snapshot_id,
        stage_name,
        phase,
        started_message,
        repair_attempt_number,
        request_discriminator,
        run_cancellation_token,
    } = request;
    let request_discriminator = request_discriminator.as_deref();
    let browser_run_id = browser_run_id_for_stage(
        run_id,
        stage_run_id,
        repair_attempt_number,
        request_discriminator,
    );
}
```

The shown `let browser_run_id` block is the first existing operation. Retain it
once and retain every following statement through the function's closing brace
exactly as currently written.

The comments above describe the splice point; do not add them to production code. In `persist_browser_stage_provenance`, replace only:

```rust
.bind(now_string())
```

with:

```rust
.bind(crate::time::now_rfc3339_utc())
```

The moved runners must still contain exactly four direct `emit` calls whose
first argument is `PROMPT_PACK_RUN_EVENT`, including the literal:

```rust
format!("JSON repair queued at position {position}")
```

Do not call `emit_prompt_pack_run_event`, reorder cancellation/provenance operations, or alter event fields.

- [ ] **Step 7: Replace both provider matches in all five stage functions**

In `runtime.rs`, import the sibling interface:

```rust
use super::completion_transport::{
    RunCompletionRuntime, StageCompletionRequest,
};
```

In each stage function, replace the first provider match with:

```rust
let model_context = completion_runtime.model_context().await?;
```

Pass these fields to the existing prompt builder instead of the old tuple variables:

```rust
model_context.profile_id,
model_context.model_override,
// existing max-output calculation receives model_context.model_output_limit
```

After building the request, replace the second provider match with one `execute` call. Use these exact semantic field values:

```rust
// run_transcript_analysis_stage_request
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

// run_synthesis_stage_request
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

// run_json_repair_stage_request
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
```

For the two Gem functions, calculate the existing discriminator before constructing the request and use:

```rust
// run_gem_analysis_part_stage_request
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

// run_gem_analysis_part_repair_request
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
```

Expected: runtime still builds the same five LLM requests and budgets, but contains no `match completion_runtime` or `match &completion_runtime`.

- [ ] **Step 8: Update runtime test imports and add focused model-context tests**

In `runtime::tests`, remove the moved helper names from the existing parent
module import block and import them from the private sibling module without
changing any existing test body:

```rust
use super::super::completion_transport::{
    browser_run_id_for_stage, browser_run_source_for_stage,
    browser_stage_completion_from_result, llm_chat_request_to_browser_prompt,
    persist_browser_stage_provenance, run_browser_stage_result_with_cancellation,
};
use super::super::run_control::run_with_prompt_pack_run_cancellation;
```

Remove `run_with_prompt_pack_run_cancellation` from that same existing parent
module import block along with the six transport helpers above.

At the bottom of `completion_transport.rs`, add:

```rust
#[cfg(test)]
mod tests {
    use super::RunCompletionRuntime;
    use crate::llm::{ProviderKind, ResolvedLlmProfile};

    #[tokio::test]
    async fn browser_model_context_has_no_api_fields() {
        let runtime = RunCompletionRuntime::GeminiBrowser {
            browser_provider_config: None,
        };

        let context = runtime.model_context().await.expect("browser context");

        assert_eq!(context.profile_id, None);
        assert_eq!(context.model_override, None);
        assert_eq!(context.model_output_limit, None);
    }

    #[tokio::test]
    async fn api_model_context_retains_profile_and_override() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind unavailable model endpoint");
        let base_url = format!("http://{}", listener.local_addr().expect("model endpoint"));
        drop(listener);
        let runtime = RunCompletionRuntime::Api {
            profile: ResolvedLlmProfile {
                profile_id: "profile-7".to_string(),
                provider: ProviderKind::OpenAiCompatible,
                default_model: "default-model".to_string(),
                api_key: "test-api-key".to_string().into(),
                base_url,
            },
            model_override: Some("override-model".to_string()),
        };

        let context = runtime.model_context().await.expect("api context");

        assert_eq!(context.profile_id.as_deref(), Some("profile-7"));
        assert_eq!(context.model_override.as_deref(), Some("override-model"));
        assert_eq!(context.model_output_limit, None);
    }
}
```

Expected: Browser context is explicitly empty; API context preserves profile/model override and exercises the existing unavailable-model-endpoint fallback to `None` without external network access.

- [ ] **Step 9: Clean production imports and format the five scoped files**

Remove production imports from `runtime.rs` that became transport-only, including:

```rust
use std::future::Future;
use std::time::Instant;
use super::run_control::run_with_prompt_pack_run_cancellation;
```

From the production grouped `crate::llm` import, remove these transport-only
names no longer used by runtime:

```text
resolve_model_output_token_limit_for_backend
run_llm_collect_with_profile
LlmChatRequest
LlmRequestError
LlmRequestKind
LlmRequestMetadata
LlmRequestPriority
ResolvedLlmProfile
```

Keep `resolve_effective_model`, `resolve_model_input_token_limit_for_backend`, `resolve_profile_for_backend`, and `LlmSchedulerState` because runtime preflight/command code still uses them. Keep `SqlitePool`, `AppHandle`, `Emitter`, `Manager`, and `CancellationToken` because runtime still uses them outside the moved block. Let the compiler identify any additional stale import; do not suppress warnings.

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: formatting changes only the four scoped Rust files.

- [ ] **Step 10: Run the source contract for GREEN**

Run:

```powershell
npm.cmd run test -- src/lib/prompt-pack-completion-transport-contract.test.ts
```

Expected: Vitest runs 6 tests and all pass. In particular, runtime has zero provider-match markers, exactly five `model_context()` calls and five `execute()` calls, and transport retains four direct event emits.

- [ ] **Step 11: Run focused model-context and moved-helper behavior tests**

Run:

```powershell
$tests = @(
    'prompt_packs::completion_transport::tests::browser_model_context_has_no_api_fields',
    'prompt_packs::completion_transport::tests::api_model_context_retains_profile_and_override',
    'prompt_packs::runtime::tests::browser_prompt_formatter_preserves_role_order_and_content',
    'prompt_packs::runtime::tests::browser_prompt_formatter_rejects_unsupported_roles',
    'prompt_packs::runtime::tests::browser_run_identity_includes_repair_attempt_when_present',
    'prompt_packs::runtime::tests::browser_run_id_accepts_optional_gem_discriminator',
    'prompt_packs::runtime::tests::prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job',
    'prompt_packs::runtime::tests::prompt_pack_browser_stage_cancelled_while_active_stops_sidecar',
    'prompt_packs::runtime::tests::cancelled_browser_stage_does_not_persist_success_provenance'
)
foreach ($test in $tests) {
    cargo test --manifest-path src-tauri/Cargo.toml --lib $test -- --exact
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Expected: every command runs exactly one test and passes. If a filter matches zero tests, correct the name before proceeding; an empty run is not GREEN.

- [ ] **Step 12: Run complete Prompt Pack and repository test suites**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::runtime::tests
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
npm.cmd run test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: runtime tests, all Prompt Pack Rust tests, complete Vitest, and complete Rust suites exit 0 with no failed tests.

- [ ] **Step 13: Verify formatting and all Rust targets with zero warnings**

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

- [ ] **Step 14: Review exact scope and commit**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$changed = @(git status --porcelain=v1 --untracked-files=all | ForEach-Object {
    $_.Substring(3).Replace('\', '/')
})
$allowed = @(
    'src-tauri/src/prompt_packs/completion_transport.rs',
    'src-tauri/src/prompt_packs/dto.rs',
    'src-tauri/src/prompt_packs/mod.rs',
    'src-tauri/src/prompt_packs/runtime.rs',
    'src/lib/prompt-pack-completion-transport-contract.test.ts'
)
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
"CHANGED=$($changed -join ',')"
"UNEXPECTED=$($unexpected -join ',')"
if ($changed.Count -ne 5 -or $unexpected.Count -ne 0) { exit 1 }
git add -- src-tauri/src/prompt_packs/completion_transport.rs `
    src-tauri/src/prompt_packs/dto.rs `
    src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src/lib/prompt-pack-completion-transport-contract.test.ts
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git diff --cached --stat
git diff --cached -- src-tauri/src/prompt_packs/completion_transport.rs `
    src-tauri/src/prompt_packs/dto.rs `
    src-tauri/src/prompt_packs/mod.rs `
    src-tauri/src/prompt_packs/runtime.rs `
    src/lib/prompt-pack-completion-transport-contract.test.ts
git commit -m "refactor: extract prompt pack completion transport"
git status --short --branch
```

Expected: the implementation commit contains exactly the private module registration, event-constant move/re-export, transport extraction, five stage rewrites, import/test wiring, focused model-context tests, and source contract. The worktree is clean after commit.
