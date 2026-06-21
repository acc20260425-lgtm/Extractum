# Gemini Browser Prompt-Pack Runtime Provider Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let YouTube Summary prompt-pack runs choose `api` or `gemini_browser` as the completion runtime while keeping `ProviderKind` API-only.

**Architecture:** Add runtime selection to the prompt-pack run contract and persisted run row, then dispatch existing stage `LlmChatRequest`s through either the current API scheduler or a reusable Gemini Browser send-single helper. Browser results are converted through the existing hardened `gemini_browser_stage::browser_result_to_completion_text` path and then flow through the same prompt-pack validation and persistence as API completions.

**Tech Stack:** Rust/Tauri, sqlx SQLite migrations, Svelte 5, TypeScript, Vitest, existing Gemini Browser sidecar command protocol.

---

## File Structure

- Create `src-tauri/migrations/0010_prompt_pack_runtime_provider.sql`: schema fields for runtime selection and browser config snapshots.
- Modify `src-tauri/src/prompt_packs/dto.rs`: runtime provider enum, request fields, run summary field.
- Modify `src-tauri/src/prompt_packs/youtube_summary/mod.rs`: runtime-aware start preflight budget.
- Modify `src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs`: browser-runtime preflight coverage.
- Modify `src-tauri/src/prompt_packs/youtube_summary/snapshots.rs`: persist runtime fields and audit snapshot.
- Modify `src-tauri/src/prompt_packs/youtube_summary/store.rs`: expose runtime provider in YouTube Summary run summaries.
- Modify `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`: default test requests use `api`.
- Modify `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`: storage/idempotency coverage.
- Modify `src-tauri/src/gemini_browser/commands.rs`: extract reusable `send_single_prompt`.
- Modify `src-tauri/src/gemini_browser/mod.rs`: re-export `send_single_prompt` for prompt-pack runtime use.
- Modify `src-tauri/src/prompt_packs/runtime.rs`: load runtime config, format browser prompts, dispatch stages to API or browser backend.
- Modify `src/lib/types/prompt-packs.ts`: TypeScript runtime provider contract.
- Modify `src/lib/api/prompt-packs.test.ts`: API wrapper coverage for browser runtime payload.
- Modify `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte`: runtime selector and Browser Provider status.
- Modify `src/lib/youtube-summary-launch-contract.test.ts`: source-contract coverage for runtime selector wiring.

Do not add `gemini_browser` to `src-tauri/src/llm/mod.rs::ProviderKind`.

---

### Task 1: Runtime Provider Contract and Migration

**Files:**
- Create: `src-tauri/migrations/0010_prompt_pack_runtime_provider.sql`
- Modify: `src-tauri/src/prompt_packs/dto.rs`
- Modify: `src/lib/types/prompt-packs.ts`

- [ ] **Step 1: Write the failing Rust DTO tests**

Append this test module to `src-tauri/src/prompt_packs/dto.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_request_defaults_to_api_runtime_provider() {
        let request: PreflightYoutubeSummaryRunRequest = serde_json::from_value(serde_json::json!({
            "projectId": null,
            "sourceIds": [901],
            "profileId": null,
            "modelOverride": null,
            "outputLanguage": "en",
            "controlPreset": "standard",
            "evidenceMode": "standard",
            "includeComments": false
        }))
        .expect("deserialize preflight request");

        assert_eq!(request.runtime_provider, PromptPackRuntimeProvider::Api);
        assert!(request.browser_provider_config.is_none());
    }

    #[test]
    fn start_request_accepts_gemini_browser_runtime_provider() {
        let request: StartYoutubeSummaryRunRequest = serde_json::from_value(serde_json::json!({
            "clientRequestId": "req-browser-runtime-1",
            "projectId": null,
            "sourceIds": [901],
            "profileId": null,
            "modelOverride": null,
            "outputLanguage": "en",
            "controlPreset": "standard",
            "evidenceMode": "standard",
            "includeComments": false,
            "runtimeProvider": "gemini_browser",
            "browserProviderConfig": {
                "mode": "cdp_attach",
                "cdpEndpoint": "http://127.0.0.1:9222"
            }
        }))
        .expect("deserialize start request");

        assert_eq!(
            request.runtime_provider,
            PromptPackRuntimeProvider::GeminiBrowser
        );
        let config = request.browser_provider_config.expect("browser config");
        assert_eq!(config.mode, crate::gemini_browser::GeminiBrowserProviderMode::CdpAttach);
        assert_eq!(config.cdp_endpoint.as_deref(), Some("http://127.0.0.1:9222"));
    }
}
```

- [ ] **Step 2: Run the DTO tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::dto::tests
```

Expected: FAIL because `PromptPackRuntimeProvider`, `runtime_provider`, and `browser_provider_config` do not exist.

- [ ] **Step 3: Add the migration**

Create `src-tauri/migrations/0010_prompt_pack_runtime_provider.sql`:

```sql
ALTER TABLE prompt_pack_runs
ADD COLUMN runtime_provider TEXT NOT NULL DEFAULT 'api'
CHECK (runtime_provider IN ('api', 'gemini_browser'));

ALTER TABLE prompt_pack_runs
ADD COLUMN browser_provider_config_json TEXT
CHECK (
    browser_provider_config_json IS NULL
    OR length(trim(browser_provider_config_json)) > 0
);
```

- [ ] **Step 4: Add Rust DTO fields and enum**

At the top of `src-tauri/src/prompt_packs/dto.rs`, add:

```rust
use crate::gemini_browser::GeminiBrowserProviderConfig;
```

Then add this enum above `PreflightYoutubeSummaryRunRequest`:

```rust
#[derive(
    Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum PromptPackRuntimeProvider {
    Api,
    GeminiBrowser,
}

impl Default for PromptPackRuntimeProvider {
    fn default() -> Self {
        Self::Api
    }
}

impl PromptPackRuntimeProvider {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::GeminiBrowser => "gemini_browser",
        }
    }
}
```

Add these fields to both `PreflightYoutubeSummaryRunRequest` and `StartYoutubeSummaryRunRequest` after `model_override`:

```rust
    #[serde(default)]
    pub runtime_provider: PromptPackRuntimeProvider,
    #[serde(default)]
    pub browser_provider_config: Option<GeminiBrowserProviderConfig>,
```

Add this field to `PromptPackRunSummaryDto` after `run_label`:

```rust
    pub runtime_provider: String,
```

- [ ] **Step 5: Add TypeScript types**

In `src/lib/types/prompt-packs.ts`, add this import at the top:

```ts
import type { GeminiBrowserProviderConfig } from "./gemini-browser";
```

Add this type near the run status types:

```ts
export type PromptPackRuntimeProvider = "api" | "gemini_browser";
```

Add these fields to both `PreflightYoutubeSummaryRunInput` and `StartYoutubeSummaryRunInput` after `modelOverride`:

```ts
  runtimeProvider?: PromptPackRuntimeProvider;
  browserProviderConfig?: GeminiBrowserProviderConfig | null;
```

Add this optional field to `PromptPackRunSummary` after `runLabel`:

```ts
  runtimeProvider?: PromptPackRuntimeProvider;
```

- [ ] **Step 6: Run contract tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::dto::tests
npm.cmd run test -- src/lib/api/prompt-packs.test.ts
```

Expected: Rust DTO tests PASS. Frontend tests should still PASS because new input fields are optional.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri/migrations/0010_prompt_pack_runtime_provider.sql src-tauri/src/prompt_packs/dto.rs src/lib/types/prompt-packs.ts
git commit -m "feat: add prompt pack runtime provider contract"
```

---

### Task 2: Persist Runtime Choice and Browser Config

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/mod.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/store.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`

- [ ] **Step 1: Write failing storage tests**

In `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`, add imports:

```rust
use crate::compression::decompress_text;
use crate::gemini_browser::{GeminiBrowserProviderConfig, GeminiBrowserProviderMode};
use crate::prompt_packs::dto::PromptPackRuntimeProvider;
```

Append these tests:

```rust
#[tokio::test]
async fn start_persists_gemini_browser_runtime_and_config_snapshot() {
    let pool = test_pool_with_ready_video().await;
    let mut request = start_request("req-browser-runtime-start", vec![901]);
    request.runtime_provider = PromptPackRuntimeProvider::GeminiBrowser;
    request.profile_id = None;
    request.model_override = None;
    request.browser_provider_config = Some(GeminiBrowserProviderConfig {
        mode: GeminiBrowserProviderMode::CdpAttach,
        cdp_endpoint: Some("http://127.0.0.1:9222".to_string()),
    });

    let run = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start browser runtime")
        .expect_started("browser runtime run");

    assert_eq!(run.runtime_provider, "gemini_browser");

    let (runtime_provider, browser_config_json, request_json_zstd): (String, Option<String>, Vec<u8>) =
        sqlx::query_as(
            "SELECT runtime_provider, browser_provider_config_json, request_json_zstd
             FROM prompt_pack_runs
             WHERE id = ?",
        )
        .bind(run.run_id)
        .fetch_one(&pool)
        .await
        .expect("runtime row");

    assert_eq!(runtime_provider, "gemini_browser");
    let browser_config_json = browser_config_json.expect("browser config json");
    assert!(browser_config_json.contains("\"mode\":\"cdp_attach\""));
    assert!(browser_config_json.contains("127.0.0.1:9222"));

    let request_json = decompress_text(&request_json_zstd).expect("decompress request");
    assert!(request_json.contains("\"runtimeProvider\":\"gemini_browser\""));
    assert!(request_json.contains("\"browserProviderConfig\""));
}

#[tokio::test]
async fn duplicate_client_request_id_preserves_existing_runtime_provider() {
    let pool = test_pool_with_ready_video().await;
    let mut browser_request = start_request("req-runtime-idempotent", vec![901]);
    browser_request.runtime_provider = PromptPackRuntimeProvider::GeminiBrowser;
    browser_request.profile_id = None;
    browser_request.model_override = None;

    let first = start_youtube_summary_run_in_pool(&pool, browser_request)
        .await
        .expect("first start")
        .expect_started("first start");

    let api_request = start_request("req-runtime-idempotent", vec![901]);
    let second = start_youtube_summary_run_in_pool(&pool, api_request)
        .await
        .expect("second start")
        .expect_started("second start");

    assert_eq!(first.run_id, second.run_id);
    assert_eq!(second.runtime_provider, "gemini_browser");
}
```

In `src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs`, append:

```rust
#[tokio::test]
async fn browser_runtime_preflight_does_not_apply_api_input_limit() {
    let pool = test_pool_with_ready_video().await;
    insert_transcript(&pool, 901, &"x".repeat(160_000)).await;
    let mut request = request_for_video(901);
    request.runtime_provider = crate::prompt_packs::dto::PromptPackRuntimeProvider::GeminiBrowser;
    request.model_override = None;

    let response = preflight_youtube_summary_in_pool(
        &pool,
        request,
        ModelBudget {
            input_token_limit: None,
        },
    )
    .await
    .expect("browser preflight");

    assert_eq!(response.included_videos.len(), 1);
    assert_eq!(response.selected_model_input_limit, None);
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::youtube_summary::snapshots_tests
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::youtube_summary::preflight_tests::browser_runtime_preflight_does_not_apply_api_input_limit
```

Expected: FAIL because runtime fields are not copied, stored, or exposed in run summaries.

- [ ] **Step 3: Update test support defaults**

In `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`, import:

```rust
use crate::prompt_packs::dto::PromptPackRuntimeProvider;
```

Add these fields to the request literals in `request_for_video()` and `start_request()` after `model_override`:

```rust
        runtime_provider: PromptPackRuntimeProvider::Api,
        browser_provider_config: None,
```

- [ ] **Step 4: Add runtime-aware budget helper**

In `src-tauri/src/prompt_packs/youtube_summary/mod.rs`, extend the DTO import:

```rust
use super::dto::{
    PreflightYoutubeSummaryRunRequest, PromptPackRuntimeProvider,
    StartYoutubeSummaryRunOutcomeDto, StartYoutubeSummaryRunRequest,
};
```

Add this helper above `start_youtube_summary_run_in_pool`:

```rust
pub(crate) fn model_budget_for_runtime(
    runtime_provider: PromptPackRuntimeProvider,
) -> ModelBudget {
    match runtime_provider {
        PromptPackRuntimeProvider::Api => ModelBudget {
            input_token_limit: Some(32_000),
        },
        PromptPackRuntimeProvider::GeminiBrowser => ModelBudget {
            input_token_limit: None,
        },
    }
}
```

When building `preflight_request`, copy the new fields:

```rust
        runtime_provider: request.runtime_provider,
        browser_provider_config: request.browser_provider_config.clone(),
```

Replace the hard-coded `ModelBudget` in `start_youtube_summary_run_in_pool` with:

```rust
        model_budget_for_runtime(request.runtime_provider),
```

- [ ] **Step 5: Update skeleton persistence**

In `src-tauri/src/prompt_packs/youtube_summary/snapshots.rs`, add `model_budget_for_runtime` to the `super` import:

```rust
use super::{estimate_tokens, model_budget_for_runtime, now_string, ModelBudget, SYNTHESIS_STAGE_NAME};
```

When building the internal `PreflightYoutubeSummaryRunRequest`, copy:

```rust
            runtime_provider: request.runtime_provider,
            browser_provider_config: request.browser_provider_config.clone(),
```

Replace the hard-coded `ModelBudget` with:

```rust
        model_budget_for_runtime(request.runtime_provider),
```

Before `request_json`, add:

```rust
    let browser_provider_config_json = request
        .browser_provider_config
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| AppError::internal(format!("serialize browser provider config: {error}")))?;
```

Replace `request_json` with:

```rust
    let request_json = serde_json::to_string(&serde_json::json!({
        "clientRequestId": request.client_request_id,
        "projectId": request.project_id,
        "sourceIds": request.source_ids,
        "profileId": request.profile_id,
        "modelOverride": request.model_override,
        "runtimeProvider": request.runtime_provider.as_str(),
        "browserProviderConfig": request.browser_provider_config,
        "outputLanguage": request.output_language,
        "controlPreset": request.control_preset,
        "evidenceMode": request.evidence_mode,
        "includeComments": request.include_comments
    }))
    .map_err(|error| AppError::internal(format!("serialize request: {error}")))?;
```

Update the insert column list:

```sql
            provider_profile_id, model, runtime_provider, browser_provider_config_json,
            output_language, control_preset, evidence_mode,
```

Update the values list by adding two SQL bind markers after `model`:

```sql
            'queued', 'none', ?, ?, ?, ?, ?, ?, ?, ?, ?, 'Queued',
```

Bind the two new values after `model_override`:

```rust
    .bind(request.runtime_provider.as_str())
    .bind(&browser_provider_config_json)
```

- [ ] **Step 6: Update runtime command preflight arguments**

In `src-tauri/src/prompt_packs/runtime.rs`, add `PromptPackRuntimeProvider` to the DTO import:

```rust
use super::dto::{
    PromptPackRunEvent, PromptPackRunSummaryDto, PromptPackRuntimeProvider,
    PromptPackStageRunDto, StartYoutubeSummaryRunOutcomeDto,
};
```

Add `model_budget_for_runtime` to the youtube summary import:

```rust
    execute_youtube_summary_run_with_stage_executor, model_budget_for_runtime,
    preflight_youtube_summary_in_pool, start_youtube_summary_run_in_pool,
```

Add command parameters to `preflight_youtube_summary_run` after `model_override`:

```rust
    runtime_provider: Option<PromptPackRuntimeProvider>,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
```

At the start of that function, add:

```rust
    let runtime_provider = runtime_provider.unwrap_or_default();
```

Pass the new DTO fields:

```rust
            runtime_provider,
            browser_provider_config,
```

Replace the hard-coded budget with:

```rust
        model_budget_for_runtime(runtime_provider),
```

Add command parameters to `start_youtube_summary_run` after `model_override`:

```rust
    runtime_provider: Option<PromptPackRuntimeProvider>,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
```

At the start of that function, add:

```rust
    let runtime_provider = runtime_provider.unwrap_or_default();
```

Pass the new DTO fields:

```rust
            runtime_provider,
            browser_provider_config,
```

- [ ] **Step 7: Expose runtime provider in run summaries**

In `src-tauri/src/prompt_packs/youtube_summary/store.rs`, extend both `load_run_summary()` tuple and SQL with `runtime_provider` after `run_label`, then set `runtime_provider` on `PromptPackRunSummaryDto`.

In `src-tauri/src/prompt_packs/runtime.rs`, update:

- `RunSummaryRow` with `runtime_provider: String`
- `load_run_summary_optional()` SELECT with `runtime_provider` after `run_label`
- `list_prompt_pack_runs_in_pool()` query tuples and mapping with `runtime_provider`
- `test_pool_with_prompt_pack_runs()` inserts with `runtime_provider = 'api'`

Use this field assignment in all `PromptPackRunSummaryDto` constructors:

```rust
            runtime_provider,
```

- [ ] **Step 8: Run storage tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::youtube_summary::snapshots_tests
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::youtube_summary::preflight_tests::browser_runtime_preflight_does_not_apply_api_input_limit
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::runtime::tests::list_prompt_pack_runs_returns_recent_runs_for_project
```

Expected: PASS.

- [ ] **Step 9: Commit**

```powershell
git add src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/youtube_summary/mod.rs src-tauri/src/prompt_packs/youtube_summary/snapshots.rs src-tauri/src/prompt_packs/youtube_summary/test_support.rs src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs src-tauri/src/prompt_packs/youtube_summary/store.rs
git commit -m "feat: persist prompt pack runtime provider"
```

---

### Task 3: Reusable Gemini Browser Send-Single Helper

**Files:**
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`

- [ ] **Step 1: Write the refactor target**

In `src-tauri/src/gemini_browser/commands.rs`, add this helper above `gemini_bridge_send_single`:

```rust
pub(crate) async fn send_single_prompt(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    run_id: String,
    prompt: String,
    source: Option<String>,
    artifact_mode: Option<String>,
    browser_config: Option<GeminiBrowserProviderConfig>,
) -> AppResult<GeminiBrowserRunResult> {
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::validation("prompt cannot be empty"));
    }
    let request = GeminiBrowserRunRequest {
        run_id,
        prompt,
        source: source.unwrap_or_else(|| "settings_test".to_string()),
        artifact_mode: artifact_mode.unwrap_or_else(|| "reduced".to_string()),
    };

    let runs_root = runs_dir(handle)?;
    create_queued_run(
        &runs_root,
        &request.run_id,
        &request.source,
        &request.prompt,
    )?;
    let queue_position = state.enqueue(request.clone()).await;
    emit_run_event(
        handle,
        GeminiBrowserRunEvent {
            run_id: request.run_id.clone(),
            status: GeminiBrowserRunStatus::Queued,
            message: Some("Queued".to_string()),
            queue_position: Some(queue_position),
        },
    );

    let next = state
        .pop_next()
        .await
        .ok_or_else(|| AppError::internal("Gemini browser queue unexpectedly empty"))?;
    let _token = state.start_run(next.run_id.clone()).await;
    mark_running(&runs_root, &next.run_id)?;
    emit_run_event(
        handle,
        GeminiBrowserRunEvent {
            run_id: next.run_id.clone(),
            status: GeminiBrowserRunStatus::Running,
            message: Some("Running".to_string()),
            queue_position: None,
        },
    );

    let artifact_dir = path_string(&run_dir(handle, &next.run_id)?);
    let result = match sidecar::send_single(
        handle,
        state,
        next.clone(),
        path_string(&profile_dir(handle)?),
        artifact_dir,
        browser_config,
    )
    .await
    {
        Ok(result) => result,
        Err(_) => sidecar::sidecar_unavailable_result(next.clone()),
    };
    finish_run(&runs_root, &next.run_id, result.clone())?;
    state.finish_run(&next.run_id).await;
    emit_run_event(
        handle,
        GeminiBrowserRunEvent {
            run_id: next.run_id,
            status: result.status.clone(),
            message: result.message.clone(),
            queue_position: None,
        },
    );
    Ok(result)
}
```

- [ ] **Step 2: Delegate the Tauri command**

Replace the body of `gemini_bridge_send_single()` with:

```rust
    send_single_prompt(
        &handle,
        &state,
        run_id,
        prompt,
        source,
        artifact_mode,
        browser_config,
    )
    .await
```

- [ ] **Step 3: Re-export the reusable helper**

In `src-tauri/src/gemini_browser/mod.rs`, add a crate-visible re-export directly below the existing public command re-export block:

```rust
pub use commands::{
    gemini_bridge_list_runs, gemini_bridge_open_browser, gemini_bridge_open_run_folder,
    gemini_bridge_resume, gemini_bridge_send_single, gemini_bridge_start_cdp_chrome,
    gemini_bridge_status, gemini_bridge_stop,
};
pub(crate) use commands::send_single_prompt;
```

Keep `send_single_prompt` `pub(crate)` in `commands.rs`; it is an internal runtime helper, not a Tauri command.

- [ ] **Step 4: Run Gemini Browser tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
```

Expected: PASS. This task is a refactor only; command behavior stays unchanged.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/mod.rs
git commit -m "refactor: reuse Gemini browser send single command"
```

---

### Task 4: Browser Prompt Formatting and Result Conversion Helpers

**Files:**
- Modify: `src-tauri/src/prompt_packs/runtime.rs`

- [ ] **Step 1: Add failing helper tests**

In `src-tauri/src/prompt_packs/runtime.rs` test module imports, add:

```rust
        browser_run_id_for_stage, browser_run_source_for_stage, browser_stage_completion_from_result,
        llm_chat_request_to_browser_prompt,
```

Add `LlmChatRequest` and `LlmMessage` to the existing `use crate::llm` test import:

```rust
    use crate::llm::{LlmChatRequest, LlmMessage, LlmRequestError};
```

Add these tests:

```rust
    #[test]
    fn browser_prompt_formatter_preserves_role_order_and_content() {
        let request = LlmChatRequest {
            request_id: "req-browser-format".to_string(),
            profile_id: None,
            model_override: None,
            max_output_tokens: None,
            messages: vec![
                LlmMessage {
                    role: "system".to_string(),
                    content: "Return strict JSON.".to_string(),
                },
                LlmMessage {
                    role: "user".to_string(),
                    content: "Analyze this transcript.".to_string(),
                },
                LlmMessage {
                    role: "user".to_string(),
                    content: "Use source_ref_1 only.".to_string(),
                },
            ],
        };

        let prompt = llm_chat_request_to_browser_prompt(&request).expect("format prompt");

        assert_eq!(
            prompt,
            "System:\nReturn strict JSON.\n\nUser:\nAnalyze this transcript.\n\nUser:\nUse source_ref_1 only."
        );
    }

    #[test]
    fn browser_prompt_formatter_rejects_unsupported_roles() {
        let request = LlmChatRequest {
            request_id: "req-browser-format".to_string(),
            profile_id: None,
            model_override: None,
            max_output_tokens: None,
            messages: vec![LlmMessage {
                role: "assistant".to_string(),
                content: "previous answer".to_string(),
            }],
        };

        let error = llm_chat_request_to_browser_prompt(&request).expect_err("unsupported role");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("assistant"));
    }

    #[test]
    fn browser_run_identity_includes_repair_attempt_when_present() {
        assert_eq!(
            browser_run_id_for_stage(42, 1001, None),
            "prompt-pack-42-stage-1001"
        );
        assert_eq!(
            browser_run_id_for_stage(42, 1001, Some(2)),
            "prompt-pack-42-stage-1001-repair-2"
        );
        assert_eq!(
            browser_run_source_for_stage(42, 1001, "youtube_summary/transcript_analysis"),
            "prompt_pack:youtube_summary:youtube_summary/transcript_analysis:run:42:stage:1001"
        );
    }

    #[test]
    fn browser_stage_result_maps_to_prompt_pack_completion_without_tokens() {
        let result = crate::gemini_browser::GeminiBrowserRunResult {
            run_id: "prompt-pack-42-stage-1001".to_string(),
            status: crate::gemini_browser::GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 321,
            debug_summary: None,
        };

        let completion = browser_stage_completion_from_result(result).expect("completion");

        assert_eq!(completion.text, "answer");
        assert_eq!(completion.input_tokens, None);
        assert_eq!(completion.output_tokens, None);
        assert_eq!(completion.latency_ms, 321);
    }
```

- [ ] **Step 2: Run helper tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::runtime::tests::browser_
```

Expected: FAIL because helper functions do not exist.

- [ ] **Step 3: Add helper functions**

In `src-tauri/src/prompt_packs/runtime.rs`, add this block after `load_run_llm_config()`:

```rust
fn llm_chat_request_to_browser_prompt(request: &LlmChatRequest) -> AppResult<String> {
    let mut sections = Vec::with_capacity(request.messages.len());
    for message in &request.messages {
        let label = match message.role.as_str() {
            "system" => "System",
            "user" => "User",
            other => {
                return Err(AppError::validation(format!(
                    "Unsupported Browser Provider prompt message role: {other}"
                )));
            }
        };
        sections.push(format!("{label}:\n{}", message.content));
    }
    let prompt = sections.join("\n\n");
    if prompt.trim().is_empty() {
        return Err(AppError::validation("Browser Provider prompt cannot be empty"));
    }
    Ok(prompt)
}

fn browser_run_id_for_stage(
    run_id: i64,
    stage_run_id: i64,
    repair_attempt_number: Option<i64>,
) -> String {
    match repair_attempt_number {
        Some(attempt_number) => {
            format!("prompt-pack-{run_id}-stage-{stage_run_id}-repair-{attempt_number}")
        }
        None => format!("prompt-pack-{run_id}-stage-{stage_run_id}"),
    }
}

fn browser_run_source_for_stage(run_id: i64, stage_run_id: i64, stage_name: &str) -> String {
    format!("prompt_pack:youtube_summary:{stage_name}:run:{run_id}:stage:{stage_run_id}")
}

fn browser_stage_completion_from_result(
    result: crate::gemini_browser::GeminiBrowserRunResult,
) -> AppResult<PromptPackLlmCompletion> {
    let latency_ms = result.elapsed_ms as i64;
    let text = super::gemini_browser_stage::browser_result_to_completion_text(result)?;
    Ok(PromptPackLlmCompletion {
        text,
        input_tokens: None,
        output_tokens: None,
        latency_ms,
    })
}
```

- [ ] **Step 4: Run helper tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::runtime::tests::browser_
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::gemini_browser_stage
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/prompt_packs/runtime.rs
git commit -m "feat: format prompt pack browser completions"
```

---

### Task 5: Route Prompt-Pack Stages by Runtime Provider

**Files:**
- Modify: `src-tauri/src/prompt_packs/runtime.rs`

- [ ] **Step 1: Add failing runtime config tests**

In `src-tauri/src/prompt_packs/runtime.rs` test module imports, add:

```rust
        load_run_runtime_config, RunCompletionRuntime, RunRuntimeProvider,
```

Add this test:

```rust
    #[tokio::test]
    async fn load_run_runtime_config_reads_api_and_browser_rows() {
        let pool = test_pool_with_prompt_pack_runs([]).await;
        sqlx::query(
            "INSERT INTO prompt_pack_runs (
                id, project_id, pack_version_id, pack_id, pack_version,
                schema_version, run_status, result_status, provider_profile_id, model,
                runtime_provider, browser_provider_config_json, output_language,
                control_preset, evidence_mode, include_comments, latest_message,
                created_at, updated_at
             )
             VALUES
                (101, NULL, 1, 'youtube_summary', '1.0.0', '1.0',
                 'queued', 'none', 'profile-1', 'model-1', 'api', NULL,
                 'en', 'standard', 'standard', 0, 'Queued', '2026-06-21T00:00:00Z', '2026-06-21T00:00:00Z'),
                (102, NULL, 1, 'youtube_summary', '1.0.0', '1.0',
                 'queued', 'none', NULL, NULL, 'gemini_browser',
                 '{\"mode\":\"cdp_attach\",\"cdp_endpoint\":\"http://127.0.0.1:9222\"}',
                 'en', 'standard', 'standard', 0, 'Queued', '2026-06-21T00:00:00Z', '2026-06-21T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert runtime rows");

        let api = load_run_runtime_config(&pool, 101).await.expect("api config");
        assert_eq!(api.runtime_provider, RunRuntimeProvider::Api);
        assert_eq!(api.profile_id.as_deref(), Some("profile-1"));
        assert_eq!(api.model_override.as_deref(), Some("model-1"));

        let browser = load_run_runtime_config(&pool, 102)
            .await
            .expect("browser config");
        assert_eq!(browser.runtime_provider, RunRuntimeProvider::GeminiBrowser);
        let browser_config = browser.browser_provider_config.expect("browser config");
        assert_eq!(browser_config.cdp_endpoint.as_deref(), Some("http://127.0.0.1:9222"));
    }
```

- [ ] **Step 2: Run config test and verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::runtime::tests::load_run_runtime_config_reads_api_and_browser_rows
```

Expected: FAIL because `load_run_runtime_config` and runtime types do not exist.

- [ ] **Step 3: Replace `RunLlmConfig` with runtime config**

In `src-tauri/src/prompt_packs/runtime.rs`, replace `RunLlmConfig` and `load_run_llm_config()` with:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunRuntimeProvider {
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
struct RunRuntimeConfig {
    runtime_provider: RunRuntimeProvider,
    profile_id: Option<String>,
    model_override: Option<String>,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
}

async fn load_run_runtime_config(pool: &SqlitePool, run_id: i64) -> AppResult<RunRuntimeConfig> {
    sqlx::query_as::<_, (Option<String>, Option<String>, String, Option<String>)>(
        "SELECT provider_profile_id, model, runtime_provider, browser_provider_config_json
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
    .and_then(|(profile_id, model_override, runtime_provider, browser_config_json)| {
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
    })
}
```

In `execute_youtube_summary_run()`, replace `load_run_llm_config` with `load_run_runtime_config`.

- [ ] **Step 4: Add runtime enum used by stage execution**

Add this enum after `RunRuntimeConfig`:

```rust
#[derive(Clone, Debug)]
enum RunCompletionRuntime {
    Api {
        profile: ResolvedLlmProfile,
        model_override: Option<String>,
    },
    GeminiBrowser {
        browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    },
}
```

In `execute_youtube_summary_run()`, replace unconditional profile resolution with:

```rust
    let completion_runtime = match config.runtime_provider {
        RunRuntimeProvider::Api => RunCompletionRuntime::Api {
            profile: resolve_profile_for_backend(&handle, config.profile_id.as_deref()).await?,
            model_override: config.model_override.clone(),
        },
        RunRuntimeProvider::GeminiBrowser => RunCompletionRuntime::GeminiBrowser {
            browser_provider_config: config.browser_provider_config.clone(),
        },
    };
```

Inside the stage executor closure, clone `completion_runtime` instead of profile/model fields and pass it to each stage function.

- [ ] **Step 5: Add Browser Provider execution helper**

Add this function after `browser_stage_completion_from_result()`:

```rust
async fn run_browser_llm_request(
    handle: AppHandle,
    run_id: i64,
    stage_run_id: i64,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    run_cancellation_token: Option<CancellationToken>,
    stage_name: String,
    source_snapshot_id: Option<i64>,
    phase: &'static str,
    started_message: &'static str,
    repair_attempt_number: Option<i64>,
    llm_request: LlmChatRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    if run_cancellation_token
        .as_ref()
        .is_some_and(CancellationToken::is_cancelled)
    {
        return Err(YoutubeSummaryStageExecutionError::Cancelled);
    }

    let prompt = llm_chat_request_to_browser_prompt(&llm_request)?;
    let browser_run_id = browser_run_id_for_stage(run_id, stage_run_id, repair_attempt_number);
    let source = browser_run_source_for_stage(run_id, stage_run_id, &stage_name);

    let queued_handle = handle.clone();
    let started_handle = handle.clone();
    let request_id = llm_request.request_id.clone();
    let started_request_id = request_id.clone();
    let queued_stage_name = stage_name.to_string();
    let started_stage_name = queued_stage_name.clone();
    let queued_phase = phase.to_string();
    let started_phase = queued_phase.clone();
    let run_cancellation_for_stop = run_cancellation_token.clone();

    let _ = queued_handle.emit(
        PROMPT_PACK_RUN_EVENT,
        PromptPackRunEvent {
            run_id,
            request_id: request_id.clone(),
            kind: "queued".to_string(),
            run_status: "running".to_string(),
            phase: queued_phase,
            stage_run_id: Some(stage_run_id),
            stage_name: Some(queued_stage_name),
            source_snapshot_id,
            queue_position: None,
            progress_current: None,
            progress_total: None,
            message: Some("Browser Provider request queued".to_string()),
            error: None,
        },
    );

    let browser_state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    let browser_future = async {
        let _ = started_handle.emit(
            PROMPT_PACK_RUN_EVENT,
            PromptPackRunEvent {
                run_id,
                request_id: started_request_id,
                kind: "started".to_string(),
                run_status: "running".to_string(),
                phase: started_phase,
                stage_run_id: Some(stage_run_id),
                stage_name: Some(started_stage_name),
                source_snapshot_id,
                queue_position: None,
                progress_current: None,
                progress_total: None,
                message: Some(started_message.to_string()),
                error: None,
            },
        );
        crate::gemini_browser::send_single_prompt(
            &handle,
            &browser_state,
            browser_run_id,
            prompt,
            Some(source),
            Some("reduced".to_string()),
            browser_provider_config,
        )
        .await
        .map_err(LlmRequestError::Failed)
    };

    let result = match run_with_prompt_pack_run_cancellation(run_cancellation_token, browser_future).await {
        Ok(result) => result,
        Err(LlmRequestError::Cancelled) => {
            if run_cancellation_for_stop
                .as_ref()
                .is_some_and(CancellationToken::is_cancelled)
            {
                browser_state.request_stop().await;
            }
            return Err(YoutubeSummaryStageExecutionError::Cancelled);
        }
        Err(LlmRequestError::Failed(error)) => {
            return Err(YoutubeSummaryStageExecutionError::Failed(error));
        }
    };

    if run_cancellation_for_stop
        .as_ref()
        .is_some_and(CancellationToken::is_cancelled)
    {
        return Err(YoutubeSummaryStageExecutionError::Cancelled);
    }

    browser_stage_completion_from_result(result).map_err(YoutubeSummaryStageExecutionError::from)
}
```

- [ ] **Step 6: Update stage functions to branch on runtime**

Change each stage function signature so it accepts `completion_runtime: RunCompletionRuntime` instead of `profile: ResolvedLlmProfile` and `model_override: Option<String>`.

Inside each function, compute `(profile_id, model_override, max_output_tokens)` with this pattern:

```rust
    let (profile_id, model_override, model_output_limit) = match &completion_runtime {
        RunCompletionRuntime::Api {
            profile,
            model_override,
        } => {
            let effective_model = resolve_effective_model(profile, model_override.as_deref())?;
            let model_output_limit =
                resolve_model_output_token_limit_for_backend(profile, &effective_model).await;
            (
                Some(profile.profile_id.clone()),
                model_override.clone(),
                model_output_limit,
            )
        }
        RunCompletionRuntime::GeminiBrowser { .. } => (None, None, None),
    };
```

Keep the existing stage budget functions and build the same `LlmChatRequest` with `profile_id`, `model_override`, and `max_output_tokens`.

After building `llm_request`, branch:

```rust
    match completion_runtime {
        RunCompletionRuntime::Api { profile, .. } => {
            run_api_llm_request(
                handle,
                profile,
                run_cancellation_token,
                llm_request,
                stage_request.run_id,
                stage_request.stage_run_id,
                Some(stage_request.source_snapshot_id),
                "youtube_summary/transcript_analysis".to_string(),
                "transcript_analysis",
                "Analyzing transcript",
            )
            .await
        }
        RunCompletionRuntime::GeminiBrowser {
            browser_provider_config,
        } => {
            run_browser_llm_request(
                handle,
                stage_request.run_id,
                stage_request.stage_run_id,
                browser_provider_config,
                run_cancellation_token,
                "youtube_summary/transcript_analysis".to_string(),
                Some(stage_request.source_snapshot_id),
                "transcript_analysis",
                "Analyzing transcript",
                None,
                llm_request,
            )
            .await
        }
    }
```

For synthesis, pass `run_id`, `stage_run_id`, `source_snapshot_id: None`, stage name `"youtube_summary/synthesis".to_string()`, phase `"synthesis"`, and message `"Synthesizing summaries"`.

For JSON repair, pass `run_id`, `stage_run_id`, `source_snapshot_id: None`, `stage_request.stage_name.clone()`, phase `"repair"`, message `"Repairing stage output"`, and `repair_attempt_number: Some(stage_request.attempt_number)`.

Extract the existing scheduler body into `run_api_llm_request()` so this step does not duplicate the scheduler closure three times. Its signature should be:

```rust
async fn run_api_llm_request(
    handle: AppHandle,
    profile: ResolvedLlmProfile,
    run_cancellation_token: Option<CancellationToken>,
    llm_request: LlmChatRequest,
    run_id: i64,
    stage_run_id: i64,
    source_snapshot_id: Option<i64>,
    stage_name: String,
    phase: &'static str,
    started_message: &'static str,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError>
```

Move the current scheduler `run_request` code into this helper and preserve:

- `LlmRequestKind::PromptPackStage`
- `LlmRequestPriority::Background`
- `owner_run_id: Some(run_id)`
- queue event with scheduler `queue_position`
- started event with no `queue_position`
- usage-to-token conversion
- `LlmRequestError::Cancelled` mapping to `YoutubeSummaryStageExecutionError::Cancelled`

- [ ] **Step 7: Run runtime tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs::runtime
```

Expected: PASS.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/prompt_packs/runtime.rs
git commit -m "feat: route prompt pack stages to browser runtime"
```

---

### Task 6: Frontend API and Dialog Runtime Selector

**Files:**
- Modify: `src/lib/api/prompt-packs.test.ts`
- Modify: `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte`
- Modify: `src/lib/youtube-summary-launch-contract.test.ts`

- [ ] **Step 1: Add failing API wrapper test**

In `src/lib/api/prompt-packs.test.ts`, add `preflightYoutubeSummaryRun` to the import list.

Add this test after `"starts youtube summary run"`:

```ts
  it("forwards browser runtime provider fields for preflight and start", async () => {
    const browserProviderConfig = {
      mode: "cdp_attach" as const,
      cdpEndpoint: "http://127.0.0.1:9222",
    };
    invokeMock.mockResolvedValueOnce({
      packId: "youtube_summary",
      packVersion: "1.0.0",
      includedVideos: [],
      skippedVideos: [],
      blockingFailures: [],
      estimatedInputTokens: 0,
      selectedModelInputLimit: null,
    });

    await preflightYoutubeSummaryRun({
      projectId: null,
      sourceIds: [901],
      profileId: null,
      modelOverride: null,
      runtimeProvider: "gemini_browser",
      browserProviderConfig,
      outputLanguage: "en",
      controlPreset: "standard",
      evidenceMode: "standard",
      includeComments: false,
    });

    expect(invokeMock).toHaveBeenLastCalledWith("preflight_youtube_summary_run", {
      projectId: null,
      sourceIds: [901],
      profileId: null,
      modelOverride: null,
      runtimeProvider: "gemini_browser",
      browserProviderConfig,
      outputLanguage: "en",
      controlPreset: "standard",
      evidenceMode: "standard",
      includeComments: false,
    });

    invokeMock.mockResolvedValueOnce({
      kind: "started",
      run: {
        runId: 42,
        runStatus: "queued",
        latestMessage: "Queued",
        runtimeProvider: "gemini_browser",
      },
    });

    await startYoutubeSummaryRun({
      clientRequestId: "req-browser-runtime-ui",
      projectId: null,
      sourceIds: [901],
      profileId: null,
      modelOverride: null,
      runtimeProvider: "gemini_browser",
      browserProviderConfig,
      outputLanguage: "en",
      controlPreset: "standard",
      evidenceMode: "standard",
      includeComments: false,
    });

    expect(invokeMock).toHaveBeenLastCalledWith("start_youtube_summary_run", {
      clientRequestId: "req-browser-runtime-ui",
      projectId: null,
      sourceIds: [901],
      profileId: null,
      modelOverride: null,
      runtimeProvider: "gemini_browser",
      browserProviderConfig,
      outputLanguage: "en",
      controlPreset: "standard",
      evidenceMode: "standard",
      includeComments: false,
    });
  });
```

- [ ] **Step 2: Add failing source-contract test**

In `src/lib/youtube-summary-launch-contract.test.ts`, append:

```ts
  it("wires Gemini Browser runtime selector into preflight and start requests", () => {
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");

    expect(dialog).toContain("runtimeProvider = $state");
    expect(dialog).toContain("Gemini Browser");
    expect(dialog).toContain("geminiBridgeStatus");
    expect(dialog).toContain("deriveGeminiBrowserSetupChecks");
    expect(dialog).toContain("runtimeProvider,");
    expect(dialog).toContain("browserProviderConfig:");
  });
```

- [ ] **Step 3: Run frontend tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/api/prompt-packs.test.ts src/lib/youtube-summary-launch-contract.test.ts
```

Expected: source-contract test FAILS because dialog does not contain Browser Provider runtime wiring.

- [ ] **Step 4: Add dialog state and imports**

In `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte`, add imports:

```ts
  import {
    geminiBridgeListRuns,
    geminiBridgeStatus,
  } from "$lib/api/gemini-browser";
  import {
    deriveGeminiBrowserSetupChecks,
    setupCheckStateLabel,
  } from "$lib/gemini-browser-setup-status";
  import type {
    GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode,
    GeminiBrowserProviderStatus,
    GeminiBrowserRun,
  } from "$lib/types/gemini-browser";
  import type { PromptPackRuntimeProvider } from "$lib/types/prompt-packs";
```

Add state after `modelOverride`:

```ts
  let runtimeProvider = $state<PromptPackRuntimeProvider>("api");
  let browserProviderMode = $state<GeminiBrowserProviderMode>("managed");
  let cdpEndpoint = $state("http://127.0.0.1:9222");
  let browserStatus = $state<GeminiBrowserProviderStatus | null>(null);
  let browserRuns = $state<GeminiBrowserRun[]>([]);
  let browserStatusLoadError = $state<string | null>(null);
```

Add derived setup checks:

```ts
  const browserProviderConfig = $derived<GeminiBrowserProviderConfig | null>(
    runtimeProvider === "gemini_browser"
      ? browserConfig()
      : null,
  );
  const browserSetupChecks = $derived(
    deriveGeminiBrowserSetupChecks({
      status: browserStatus,
      providerMode: browserProviderMode,
      cdpEndpoint,
      runs: browserRuns,
      selectedRun: browserRuns[0] ?? null,
      busy: loading,
      statusLoadError: browserStatusLoadError,
    }),
  );
  const browserRuntimeBlockingCheck = $derived(
    runtimeProvider === "gemini_browser"
      ? browserSetupChecks.find((check) => check.state === "failed" || check.state === "action_needed") ?? null
      : null,
  );
```

Add helpers:

```ts
  function browserConfig(): GeminiBrowserProviderConfig {
    if (browserProviderMode === "managed") return { mode: "managed" };
    return {
      mode: "cdp_attach",
      cdpEndpoint: cdpEndpoint.trim() || null,
    };
  }

  async function refreshBrowserStatus() {
    if (runtimeProvider !== "gemini_browser") return;
    try {
      browserStatusLoadError = null;
      const [status, runs] = await Promise.all([
        geminiBridgeStatus(browserConfig()),
        geminiBridgeListRuns(5),
      ]);
      browserStatus = status;
      browserRuns = runs.runs;
    } catch (cause) {
      browserStatusLoadError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function handleRuntimeChange(event: Event) {
    runtimeProvider = (event.currentTarget as HTMLSelectElement).value as PromptPackRuntimeProvider;
    if (runtimeProvider === "gemini_browser") void refreshBrowserStatus();
    void runPreflight();
  }
```

In the `$effect` that resets open state, add:

```ts
      runtimeProvider = "api";
      browserProviderMode = "managed";
      cdpEndpoint = "http://127.0.0.1:9222";
      browserStatus = null;
      browserRuns = [];
      browserStatusLoadError = null;
```

- [ ] **Step 5: Send runtime fields in preflight and start**

In both `preflightYoutubeSummaryRun` and `startYoutubeSummaryRun` calls, change profile/model fields to:

```ts
        profileId: runtimeProvider === "api" ? profileId || null : null,
        modelOverride: runtimeProvider === "api" ? modelOverride || null : null,
        runtimeProvider,
        browserProviderConfig,
```

Change `startRun()` guard to:

```ts
    if (!source || !outputLanguage || !canStartYoutubeSummary(preflight) || browserRuntimeBlockingCheck) return;
```

- [ ] **Step 6: Add runtime selector markup**

Inside `.inputs-grid`, insert this label before `LLM profile`:

```svelte
      <label>
        <span>Runtime</span>
        <select bind:value={runtimeProvider} aria-label="Runtime provider" onchange={handleRuntimeChange}>
          <option value="api">API profile</option>
          <option value="gemini_browser">Gemini Browser</option>
        </select>
      </label>
```

Wrap the LLM profile and model override controls:

```svelte
      {#if runtimeProvider === "api"}
        <label>
          <span>LLM profile</span>
          <select bind:value={profileId} aria-label="LLM Profile" onchange={() => void runPreflight()}>
            {#each llmProfiles as profile (profile.profile_id)}
              <option value={profile.profile_id}>
                {profile.profile_id} ({profile.default_model})
              </option>
            {/each}
          </select>
        </label>
        <label class="full-width">
          <span>Model override</span>
          <ExtractumTextInput bind:value={modelOverride} onchange={() => void runPreflight()} />
        </label>
      {:else}
        <label>
          <span>Browser mode</span>
          <select bind:value={browserProviderMode} aria-label="Browser Provider mode" onchange={() => { void refreshBrowserStatus(); void runPreflight(); }}>
            <option value="managed">Managed</option>
            <option value="cdp_attach">Attach Chrome</option>
          </select>
        </label>
        {#if browserProviderMode === "cdp_attach"}
          <label class="full-width">
            <span>CDP endpoint</span>
            <ExtractumTextInput bind:value={cdpEndpoint} onchange={() => { void refreshBrowserStatus(); void runPreflight(); }} />
          </label>
        {/if}
      {/if}
```

Add Browser Provider setup status before the preflight section:

```svelte
    {#if runtimeProvider === "gemini_browser"}
      <section class="preflight-section" aria-label="Browser Provider setup">
        <h3>Browser Provider</h3>
        {#each browserSetupChecks.slice(0, 3) as check (check.id)}
          <ExtractumStatusMessage tone={check.state === "failed" || check.state === "action_needed" ? "error" : "info"}>
            {check.label}: {setupCheckStateLabel(check.state)} - {check.message}
          </ExtractumStatusMessage>
        {/each}
        <ExtractumButton type="button" variant="outline" onclick={() => void refreshBrowserStatus()} disabled={loading}>
          Refresh Browser Provider
        </ExtractumButton>
      </section>
    {/if}
```

Update the submit button disabled condition:

```svelte
      <ExtractumButton type="submit" disabled={!source || loading || !canStartYoutubeSummary(preflight) || Boolean(browserRuntimeBlockingCheck)}>Start</ExtractumButton>
```

- [ ] **Step 7: Run frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/api/prompt-packs.test.ts src/lib/youtube-summary-launch-contract.test.ts
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 8: Commit**

```powershell
git add src/lib/types/prompt-packs.ts src/lib/api/prompt-packs.test.ts src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte src/lib/youtube-summary-launch-contract.test.ts
git commit -m "feat: select prompt pack runtime in youtube summary dialog"
```

---

### Task 7: Final Verification and Documentation Sync

**Files:**
- Modify: `docs/browser-providers-llm-troubleshooting.md`
- Modify: `docs/architecture-deep-dive.md`

- [ ] **Step 1: Update troubleshooting docs**

In `docs/browser-providers-llm-troubleshooting.md`, add a short section near the prompt-pack/browser provider notes:

```markdown
## Prompt-Pack Runtime Provider

YouTube Summary runs can use `runtimeProvider: "api"` or
`runtimeProvider: "gemini_browser"`.

- `api` keeps using the LLM profile scheduler and stores `provider_profile_id`
  plus `model`.
- `gemini_browser` stores `runtime_provider = 'gemini_browser'` and an optional
  `browser_provider_config_json` snapshot on `prompt_pack_runs`.
- Browser-backed stages create Gemini Browser run history entries with IDs such
  as `prompt-pack-42-stage-1001` and
  `prompt-pack-42-stage-1001-repair-2`.
- Browser-backed answers must pass
  `prompt_packs::gemini_browser_stage::browser_result_to_completion_text`;
  `timeout_latest` partial-risk answers fail closed.
```

- [ ] **Step 2: Update architecture docs**

In `docs/architecture-deep-dive.md`, add one paragraph to the Browser Provider or Prompt Pack architecture section:

```markdown
Prompt packs do not model Gemini Browser as an `llm::ProviderKind`. Instead,
the run row stores a prompt-pack runtime provider. The API runtime resolves an
LLM profile and schedules requests through `LlmSchedulerState`; the
`gemini_browser` runtime converts the same stage chat request into a single
browser prompt and calls the Gemini Browser send-single helper.
```

- [ ] **Step 3: Run full verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib prompt_packs
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
npm.cmd run test
npm.cmd run check
git diff --check
```

Expected:

- Rust prompt-pack tests PASS.
- Rust Gemini Browser tests PASS.
- Vitest suite PASS.
- Svelte check reports `0 errors and 0 warnings`.
- `git diff --check` exits `0`.

- [ ] **Step 4: Commit**

```powershell
git add docs/browser-providers-llm-troubleshooting.md docs/architecture-deep-dive.md
git commit -m "docs: document prompt pack browser runtime"
```

---

## Review Checklist

Before calling the implementation complete:

- `src-tauri/src/llm/mod.rs::ProviderKind` still contains only `Gemini` and `OpenAiCompatible`.
- Existing API-backed YouTube Summary starts still send `runtimeProvider: "api"` or omit it and default to API.
- Browser-backed starts persist `runtime_provider = 'gemini_browser'`.
- Browser-backed preflight returns `selectedModelInputLimit: null`.
- Browser-backed runtime does not resolve an LLM profile.
- Browser-backed stage events do not show API scheduler queue positions.
- Browser-backed JSON repair run IDs include repair attempt number.
- Browser result conversion still rejects `timeout_latest` partial-risk answers.
