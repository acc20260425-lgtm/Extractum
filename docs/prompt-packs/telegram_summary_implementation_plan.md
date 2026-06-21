# Telegram Summary Pack Implementation Plan

> **For agentic workers:** execute this plan task-by-task. Use `superpowers:executing-plans` for checkpointed execution; use `superpowers:test-driven-development` before implementation code; do not use sub-agents in sessions where they are explicitly forbidden. After every task, update completed checkboxes in this file and commit the task.

**Goal:** implement `telegram_summary` as an executable Prompt Pack for Telegram channels, chats, supergroups, forum topics, and mixed Telegram source sets.

**Architecture:** add `telegram_summary` as a sibling executable pack to `youtube_summary`. Reuse shared Prompt Pack storage, run state, eventing, stage artifacts, result storage, and validation persistence. Add Telegram-specific code only where message identity, reply chains, forum topics, forwarded messages, and pack-specific validation require it.

**Primary user outcomes:**

- Generate a digest for a Telegram source or selected Telegram source set.
- Preserve message-level traceability through `telegram_summary.message_refs`.
- Support channel posts, chat messages, reply chains, forum topics, migrated history, and forwarded-message metadata.
- Produce short summary, timeline, key messages, topics, claims, threads, forwarded items, limitations, and optional message quality signals.
- Block or flag results that cite non-existent message refs.

**Tech stack:** Rust/Tauri, SQLx/SQLite, serde/serde_json, existing Prompt Pack runtime/store APIs, existing Telegram `sources` and `notebooklm_export` query helpers, Svelte/TypeScript API wrappers, JSON fixtures under `docs/prompt-packs`.

**Implementation style:**

- Write failing tests before implementation in each task.
- Keep each task independently committable.
- Do not change `youtube_summary` behavior except where shared seed/runtime helpers need pack-generic names.
- Do not add Python to the product path.
- Treat `docs/prompt-packs/telegram_summary_pack_spec.md` and `docs/prompt-packs/TELEGRAM_SUMMARY_PACK_DECISIONS.md` as the contract source.

## Current Repository Anchors

Use these existing files as integration points:

- `src-tauri/src/prompt_packs/seed.rs` currently seeds the bundled `youtube_summary` pack.
- `src-tauri/src/prompt_packs/library.rs` exposes active pack metadata to the UI.
- `src-tauri/src/prompt_packs/runtime.rs` owns Tauri commands, run state, cancellation, event emission, and `youtube_summary` execution spawning.
- `src-tauri/src/prompt_packs/dto.rs` owns Tauri DTOs and TypeScript-facing response shapes.
- `src-tauri/src/prompt_packs/youtube_summary/` is the executable-pack reference implementation.
- `src-tauri/src/sources/items.rs`, `src-tauri/src/sources/items/query.rs`, `src-tauri/src/sources/topics.rs`, and `src-tauri/src/notebooklm_export/query.rs` already understand Telegram message identity, reply metadata, and forum topics.
- `src/lib/api/prompt-packs.ts` and `src/lib/types/prompt-packs.ts` expose frontend Prompt Pack APIs.
- `docs/prompt-packs/schemas/v1/packs/telegram_summary/pack_data.schema.json` is the pack-specific JSON Schema contract.
- `docs/prompt-packs/fixtures/v1/fixture_manifest.json` lists validator/parser/prompt fixtures.

## Pack Assets To Add

Add bundled pack assets under:

```text
src-tauri/prompt-packs/telegram_summary/1.0.0/
  pack.json
  stages/pack_data_generation.json
  runtime/pack_data_generation.json
  schemas/stage-io-telegram-summary-pack-data-generation-input.json
  schemas/stage-io-telegram-summary-pack-data-generation-output.json
  schemas/canonical-result.json
```

Pack metadata:

```json
{
  "pack_id": "telegram_summary",
  "pack_version": "1.0.0",
  "schema_version": "1.0",
  "display_name": "Telegram Summary",
  "origin_kind": "bundled",
  "lifecycle_status": "active",
  "default_control_preset": "standard",
  "default_evidence_mode": "standard",
  "default_include_comments": false
}
```

Stage metadata:

```json
{
  "stage_name": "telegram_summary/pack_data_generation",
  "stage_order": 1,
  "provider_family": "llm",
  "input_schema_id": "stage-io/telegram_summary_pack_data_generation_input",
  "output_schema_id": "stage-io/telegram_summary_pack_data_generation_output",
  "validator_mode": "canonical_result",
  "prompt_template": {}
}
```

The stage prompt template should be based on `docs/prompt-packs/prompts/v1/telegram_summary_pack_data_generation.md`.

## Task 1: Seed Bundled Telegram Summary Assets

**Purpose:** make `telegram_summary` appear as an active built-in pack in the Prompt Pack library without adding runtime execution yet.

### Tests First

- [ ] Add a failing test in `src-tauri/src/prompt_packs/seed.rs` named `seed_builtin_prompt_packs_includes_telegram_summary`.
- [ ] Assert `prompt_pack_versions` contains exactly one active `telegram_summary@1.0.0` row after two seed calls.
- [ ] Assert the row has `origin_kind = 'bundled'`, `lifecycle_status = 'active'`, `default_control_preset = 'standard'`, and `default_evidence_mode = 'standard'`.
- [ ] Add a failing test in `src-tauri/src/prompt_packs/library.rs` named `get_prompt_pack_library_returns_active_telegram_summary_pack`.
- [ ] Assert the library entry has display name `Telegram Summary`.
- [ ] Assert the active version exposes one stage named `telegram_summary/pack_data_generation`.
- [ ] Assert schema assets include:
  - `stage-io/telegram_summary_pack_data_generation_input`
  - `stage-io/telegram_summary_pack_data_generation_output`
  - `canonical-result/telegram_summary`

Run the red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::seed::tests::seed_builtin_prompt_packs_includes_telegram_summary
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::library::tests::get_prompt_pack_library_returns_active_telegram_summary_pack
```

### Implementation

- [ ] Add the six bundled files listed in **Pack Assets To Add**.
- [ ] Refactor `src-tauri/src/prompt_packs/seed.rs` from single-pack constants into a small `BuiltinPromptPackBundle` list.
- [ ] Keep `youtube_summary` seeded exactly as before.
- [ ] Include Telegram schema content from the already-authored docs schema, not by duplicating a divergent shape.
- [ ] Compute content hash per pack bundle from pack JSON, stage JSON, runtime JSON if used by seed hash, and all schema assets.
- [ ] Store `bundled_source_path = 'src-tauri/prompt-packs/telegram_summary/1.0.0'`.

Suggested internal shape:

```rust
struct BuiltinPromptPackBundle {
    pack_json: &'static str,
    bundled_source_path: &'static str,
    stages: &'static [&'static str],
    schemas: &'static [BuiltinSchemaAsset],
}
```

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::seed::tests
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::library::tests
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/prompt-packs/telegram_summary src-tauri/src/prompt_packs/seed.rs src-tauri/src/prompt_packs/library.rs
git commit -m "feat(prompt-packs): seed telegram summary pack"
```

## Task 2: Add Telegram Summary DTOs And API Wrappers

**Purpose:** expose preflight/start contracts to Rust and TypeScript without starting execution logic yet.

### Tests First

- [ ] Add TypeScript tests in `src/lib/api/prompt-packs.test.ts` for:
  - `preflightTelegramSummaryRun(input)` invokes `preflight_telegram_summary_run`.
  - `startTelegramSummaryRun(input)` invokes `start_telegram_summary_run`.
  - request arguments preserve `sourceIds`, `projectId`, `profileId`, `modelOverride`, `runtimeProvider`, `browserProviderConfig`, `outputLanguage`, `controlPreset`, `evidenceMode`, `timeWindow`, and `includeMessageQualitySignals`.
- [ ] Add Rust serialization tests in `src-tauri/src/prompt_packs/dto.rs` for:
  - `PreflightTelegramSummaryRunRequest`
  - `StartTelegramSummaryRunRequest`
  - `TelegramSummaryPreflightResponse`
  - `StartTelegramSummaryRunOutcomeDto`

Run red tests:

```powershell
npm.cmd run test -- --run src/lib/api/prompt-packs.test.ts
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::dto::tests::telegram_summary
```

### Implementation

- [ ] Add DTOs in `src-tauri/src/prompt_packs/dto.rs`:
  - `PreflightTelegramSummaryRunRequest`
  - `StartTelegramSummaryRunRequest`
  - `TelegramSummaryPreflightResponse`
  - `TelegramSummaryPreflightSource`
  - `TelegramSummaryPreflightSkippedSource`
  - `TelegramSummaryPreflightFailure`
  - `StartTelegramSummaryRunOutcomeDto`
- [ ] Use the existing `PromptPackRuntimeProvider` and `GeminiBrowserProviderConfig`.
- [ ] Add TypeScript interfaces in `src/lib/types/prompt-packs.ts` with camelCase fields.
- [ ] Add wrappers in `src/lib/api/prompt-packs.ts`:

```ts
export function preflightTelegramSummaryRun(input: PreflightTelegramSummaryRunInput) {
  return invoke<TelegramSummaryPreflightResponse>("preflight_telegram_summary_run", { ...input });
}

export function startTelegramSummaryRun(input: StartTelegramSummaryRunInput) {
  return invoke<StartTelegramSummaryRunOutcome>("start_telegram_summary_run", { ...input });
}
```

- [ ] Do not wire UI controls in this task.

### Verification

- [ ] Run:

```powershell
npm.cmd run test -- --run src/lib/api/prompt-packs.test.ts
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::dto::tests
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src/lib/api/prompt-packs.ts src/lib/api/prompt-packs.test.ts src/lib/types/prompt-packs.ts src-tauri/src/prompt_packs/dto.rs
git commit -m "feat(prompt-packs): add telegram summary dto contracts"
```

## Task 3: Create Telegram Summary Module Skeleton And Commands

**Purpose:** make Tauri commands compile and return real preflight blocking responses before execution is implemented.

### Tests First

- [ ] Add tests in a new `src-tauri/src/prompt_packs/telegram_summary/mod.rs` or `preflight.rs`:
  - `preflight_blocks_empty_source_selection`
  - `preflight_blocks_non_telegram_source`
  - `preflight_reports_no_messages_for_empty_telegram_source`
- [ ] Add runtime command registration checks if the project has existing source-level route tests for commands.

Run red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary
```

### Implementation

- [ ] Create `src-tauri/src/prompt_packs/telegram_summary/mod.rs`.
- [ ] Add child modules:
  - `types.rs`
  - `preflight.rs`
  - `message_registry.rs`
  - `stage_input.rs`
  - `result_validation.rs`
  - `execution.rs`
  - `test_support.rs`
- [ ] Export `preflight_telegram_summary_in_pool` and `start_telegram_summary_run_in_pool`.
- [ ] Add `pub mod telegram_summary;` in `src-tauri/src/prompt_packs/mod.rs`.
- [ ] Add Tauri commands in `src-tauri/src/prompt_packs/runtime.rs`:
  - `preflight_telegram_summary_run`
  - `start_telegram_summary_run`
- [ ] Register the commands in `src-tauri/src/lib.rs`.
- [ ] For now, `start_telegram_summary_run_in_pool` may return `blocked` with preflight failures when execution is not enabled, but the command should compile and have deterministic errors.

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::runtime
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/src/prompt_packs src-tauri/src/lib.rs
git commit -m "feat(prompt-packs): add telegram summary command skeleton"
```

## Task 4: Build Telegram Message Registry

**Purpose:** convert selected Telegram corpus rows into stable pack-local `message_refs`.

### Tests First

Add tests in `src-tauri/src/prompt_packs/telegram_summary/message_registry.rs`:

- [ ] `registry_assigns_summary_source_namespace_per_source`
- [ ] `registry_preserves_same_message_id_across_different_sources`
- [ ] `registry_preserves_reply_chain_refs`
- [ ] `registry_preserves_forum_topic_refs`
- [ ] `registry_preserves_forwarded_message_metadata_when_available`
- [ ] `registry_preserves_migrated_history_namespace`

Use existing test helpers:

- `crate::migrations::apply_all_migrations_for_test_pool`
- `crate::sources::test_support::create_telegram_messages_table`
- `crate::sources::test_support::create_item_topic_memberships_table`
- fixture patterns from `src-tauri/src/notebooklm_export/query.rs` tests.

Run red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::message_registry
```

### Implementation

- [ ] Query Telegram items through existing source tables, preferring the same read path as `src-tauri/src/notebooklm_export/query.rs`.
- [ ] Include these fields in internal rows when available:
  - `source_id`
  - source title/subtype
  - item id
  - `telegram_message_id`
  - `history_peer_kind`
  - `history_peer_id`
  - `published_at`
  - text content or decompressed content
  - `reply_to_msg_id`
  - `reply_to_top_id`
  - `forum_topic_id`
  - `forum_topic_title`
  - `forum_topic_top_message_id`
  - `reaction_count`
  - media kind/caption metadata when available
  - forwarded metadata when available
- [ ] Generate deterministic `summary_source_id` values such as `tg_source_1`, `tg_source_2`.
- [ ] Generate deterministic `message_ref_id` values such as `msg_ref_1`, ordered by source namespace, history scope, peer identity, published time, message id, and item id.
- [ ] Resolve `reply_to_message_ref` and `reply_to_top_message_ref` by native Telegram identity within the same source namespace.
- [ ] Preserve unresolved reply ids as limitations or missing-reference diagnostics in preflight, not as invalid refs.
- [ ] Return a registry object with:
  - `sources`
  - `messages`
  - `message_ref_by_native_identity`
  - `estimated_input_tokens`
  - `limitations`

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::message_registry
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/src/prompt_packs/telegram_summary
git commit -m "feat(prompt-packs): build telegram message registry"
```

## Task 5: Implement Telegram Preflight

**Purpose:** give the UI and runtime a reliable readiness check before starting a run.

### Tests First

Add tests in `src-tauri/src/prompt_packs/telegram_summary/preflight.rs`:

- [ ] `preflight_accepts_telegram_channel_source_with_messages`
- [ ] `preflight_accepts_telegram_chat_source_with_threads`
- [ ] `preflight_accepts_mixed_telegram_sources`
- [ ] `preflight_blocks_sources_outside_selected_model_budget`
- [ ] `preflight_skips_empty_sources_without_blocking_nonempty_sources`
- [ ] `preflight_reports_selected_time_window`

Run red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::preflight
```

### Implementation

- [ ] Implement `preflight_telegram_summary_in_pool`.
- [ ] Validate all requested `source_ids` exist.
- [ ] Block non-Telegram sources with reason `not_telegram_source`.
- [ ] Allow mixed Telegram channel/chat/supergroup source sets.
- [ ] Build the message registry using Task 4.
- [ ] Estimate input tokens from included message text plus metadata overhead.
- [ ] Compare the estimate with `model_budget_for_runtime(runtime_provider)`.
- [ ] Populate `includedSources`, `skippedSources`, `blockingFailures`, `estimatedInputTokens`, and `selectedModelInputLimit`.
- [ ] Preserve time-window parameters in the response.
- [ ] Keep `views` optional and never require them for readiness.

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::preflight
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/src/prompt_packs/telegram_summary
git commit -m "feat(prompt-packs): implement telegram summary preflight"
```

## Task 6: Build Stage Input And Prompt Payload

**Purpose:** create the exact JSON payload that the LLM sees for `telegram_summary/pack_data_generation`.

### Tests First

Add tests in `src-tauri/src/prompt_packs/telegram_summary/stage_input.rs`:

- [ ] `stage_input_contains_allowed_message_refs`
- [ ] `stage_input_contains_reply_chain_context`
- [ ] `stage_input_contains_forum_topic_context`
- [ ] `stage_input_contains_forwarded_metadata_without_treating_it_as_confirmation`
- [ ] `stage_input_omits_raw_private_paths_and_internal_ids`
- [ ] `stage_input_respects_quick_standard_deep_preset`
- [ ] `stage_input_respects_narrative_standard_strict_evidence_mode`

Run red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::stage_input
```

### Implementation

- [ ] Add a `TelegramSummaryStageInput` DTO matching `stage-io-telegram-summary-pack-data-generation-input.json`.
- [ ] Include:
  - run settings
  - source summaries
  - `allowed_message_ref_ids`
  - compact message registry rows
  - reply/thread adjacency
  - forum topic records
  - forwarded-message records
  - optional quality-signal instructions
  - time-window metadata
- [ ] Store only the minimum text needed for the LLM to perform the summary.
- [ ] Preserve message order and source namespace.
- [ ] Put the prompt rules from `docs/prompt-packs/prompts/v1/telegram_summary_pack_data_generation.md` into the bundled stage template.

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::stage_input
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/prompt-packs/telegram_summary src-tauri/src/prompt_packs/telegram_summary
git commit -m "feat(prompt-packs): build telegram summary stage input"
```

## Task 7: Implement Pack-Specific Result Validation

**Purpose:** enforce the `telegram_summary` contract after the LLM returns JSON.

### Tests First

Add tests in `src-tauri/src/prompt_packs/telegram_summary/result_validation.rs`:

- [ ] `valid_minimal_fixture_has_no_telegram_summary_errors`
- [ ] `valid_chat_threads_fixture_has_no_telegram_summary_errors`
- [ ] `valid_forum_topics_fixture_has_no_telegram_summary_errors`
- [ ] `valid_forwarded_message_fixture_has_no_telegram_summary_errors`
- [ ] `dangling_message_ref_fixture_returns_vr_ts_006`
- [ ] `same_message_id_in_different_sources_is_allowed_when_namespaced`
- [ ] `strict_mode_requires_claim_evidence`
- [ ] `complete_standard_result_requires_digest_timeline_and_key_messages`
- [ ] `forwarded_item_does_not_count_as_independent_confirmation`

Use fixture files listed in `docs/prompt-packs/fixtures/v1/fixture_manifest.json`.

Run red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::result_validation
```

### Implementation

- [ ] Implement `validate_telegram_summary_result`.
- [ ] Enforce these rules from `docs/prompt-packs/validation_rules.md`:
  - `VR-TS-001` source shape enum
  - `VR-TS-002` non-empty source list for complete result
  - `VR-TS-003` unique `summary_source_id`
  - `VR-TS-004` `message_ref_id` uniqueness
  - `VR-TS-005` `summary_source_id + message_id` uniqueness when `message_id` is known
  - `VR-TS-006` all message refs point to known `message_refs`
  - `VR-TS-007` reply refs point to known `message_refs`
  - `VR-TS-008` key messages have evidence/message refs in standard/strict modes
  - `VR-TS-009` claims have evidence/message refs in standard/strict modes
  - `VR-TS-010` strict mode claim evidence is not prose-only
  - `VR-TS-011` forum topic refs resolve when topic metadata is present
  - `VR-TS-012` forwarded-message origin is labelled as forwarded context, not confirmation
  - `VR-TS-013` message quality scores have reasons when present
- [ ] Persist findings using existing Prompt Pack validation finding storage.
- [ ] Reuse shared JSON path/object path conventions from `youtube_summary` validation.

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::result_validation
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/src/prompt_packs/telegram_summary
git commit -m "feat(prompt-packs): validate telegram summary results"
```

## Task 8: Execute Telegram Summary Runs

**Purpose:** make `start_telegram_summary_run` create, execute, validate, and persist a real Prompt Pack run.

### Tests First

Add tests in `src-tauri/src/prompt_packs/telegram_summary/execution.rs`:

- [ ] `start_creates_queued_run_for_valid_sources`
- [ ] `start_is_idempotent_by_client_request_id`
- [ ] `execution_writes_stage_input_artifact`
- [ ] `execution_writes_stage_output_artifact`
- [ ] `execution_persists_canonical_result`
- [ ] `execution_persists_validation_findings`
- [ ] `execution_marks_run_complete_when_result_valid`
- [ ] `execution_marks_run_partial_when_validation_has_errors`
- [ ] `execution_emits_progress_events_for_stage_and_validation`

Use a fake stage executor instead of calling an actual LLM in unit tests.

Run red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::execution
```

### Implementation

- [ ] Implement `start_telegram_summary_run_in_pool`.
- [ ] Create a `prompt_pack_runs` row with `pack_id = 'telegram_summary'`.
- [ ] Use `client_request_id` for idempotency.
- [ ] Implement `execute_telegram_summary_run_with_stage_executor`.
- [ ] Reuse the existing LLM completion path from `runtime.rs` and `youtube_summary` where possible.
- [ ] Support both runtime providers:
  - `api`
  - `gemini_browser`
- [ ] Create one stage run for `telegram_summary/pack_data_generation`.
- [ ] Store stage input artifact before LLM execution.
- [ ] Store raw LLM output artifact after execution.
- [ ] Parse repaired JSON using existing stage output normalization/repair infrastructure where available.
- [ ] Validate the canonical result with Task 7.
- [ ] Persist canonical result and validation findings.
- [ ] Mark run:
  - `complete` when valid and result status is complete
  - `partial` when recoverable validation errors exist
  - `failed` when execution cannot produce parseable output
- [ ] Emit run events through the same event path as `youtube_summary`.
- [ ] Add cancellation checks around long operations.

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::execution
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::runtime
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/src/prompt_packs
git commit -m "feat(prompt-packs): execute telegram summary runs"
```

## Task 9: Wire Runtime Spawn And Cancellation

**Purpose:** make the Tauri runtime spawn background Telegram Summary execution like YouTube Summary.

### Tests First

- [ ] Add tests in `src-tauri/src/prompt_packs/runtime.rs`:
  - `start_telegram_summary_run_tracks_active_run`
  - `start_telegram_summary_run_emits_queued_event`
  - `cancel_prompt_pack_run_can_cancel_telegram_summary_run`
  - `active_run_state_finishes_after_telegram_terminal_event`

Run red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::runtime::tests::start_telegram_summary
```

### Implementation

- [ ] Add `spawn_telegram_summary_execution`.
- [ ] Add Telegram execution branch using `execute_telegram_summary_run_with_stage_executor`.
- [ ] Keep `PromptPackRunState` pack-agnostic.
- [ ] Ensure terminal events call `finish(run_id)`.
- [ ] Ensure cancellation token is passed to Telegram execution.
- [ ] Keep YouTube execution behavior unchanged.

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::runtime
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/telegram_summary
git commit -m "feat(prompt-packs): run telegram summary in background"
```

## Task 10: Frontend API And Minimal UI Entry

**Purpose:** let the user start `telegram_summary` without manually invoking Tauri commands.

### Tests First

- [ ] Extend `src/lib/api/prompt-packs.test.ts` for the Telegram wrappers if not already covered in Task 2.
- [ ] Add source-contract or model tests for whichever UI component currently starts Prompt Pack runs.
- [ ] Assert Telegram sources can choose `Telegram Summary`.
- [ ] Assert non-Telegram sources do not show `Telegram Summary` as the primary action.
- [ ] Assert preflight failures are displayed before start.

Run red tests:

```powershell
npm.cmd run test -- --run src/lib/api/prompt-packs.test.ts
npm.cmd run test -- --run src/lib/**/*.test.ts
```

### Implementation

- [ ] Add a generic Prompt Pack selection path if the UI is currently YouTube-only.
- [ ] Prefer minimal UI changes:
  - expose `Telegram Summary` from the existing prompt-pack library
  - call `preflightTelegramSummaryRun`
  - call `startTelegramSummaryRun`
  - reuse existing run history and result viewer components
- [ ] Add control fields:
  - output language
  - control preset
  - evidence mode
  - optional time window
  - include message quality signals
  - runtime provider
  - profile/model override
- [ ] Do not build a custom Telegram report viewer in this task. The canonical result viewer and run history are enough for MVP.

### Verification

- [ ] Run:

```powershell
npm.cmd run test -- --run src/lib/api/prompt-packs.test.ts
npm.cmd run check
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src/lib
git commit -m "feat(prompt-packs): wire telegram summary frontend entry"
```

## Task 11: End-To-End Smoke Fixture

**Purpose:** verify the pack can run without relying on a real Telegram account or live LLM.

### Tests First

- [ ] Add a backend smoke test fixture with a small Telegram channel/chat corpus:
  - two sources
  - one reply chain
  - one forum topic
  - one forwarded message
  - one repeated claim
- [ ] Add a fake stage executor that returns a valid canonical result referencing those message refs.
- [ ] Add a second fake output that references a missing message ref and assert run becomes `partial` with `VR-TS-006`.

Run red tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary::tests::smoke
```

### Implementation

- [ ] Add fixture builders in `src-tauri/src/prompt_packs/telegram_summary/test_support.rs`.
- [ ] Seed source rows through existing source helper paths instead of raw SQL where possible.
- [ ] Keep fixture content short and deterministic.
- [ ] Store expected outputs under Rust test constants or `src-tauri/src/prompt_packs/telegram_summary/testdata/` if multiline JSON becomes large.

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary
```

- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs/telegram_summary_implementation_plan.md src-tauri/src/prompt_packs/telegram_summary
git commit -m "test(prompt-packs): add telegram summary smoke coverage"
```

## Task 12: Documentation And Manual Validation

**Purpose:** leave the implemented pack understandable and recoverable.

### Documentation Updates

- [ ] Update `docs/prompt-packs/README.md` with `telegram_summary` execution status.
- [ ] Update `docs/prompt-packs/telegram_summary_pack_spec.md` if implementation reveals contract adjustments.
- [ ] Update `docs/prompt-packs/TELEGRAM_SUMMARY_PACK_DECISIONS.md` only for new decisions, not routine implementation notes.
- [ ] Add a short operator section:
  - which source types work
  - how to run preflight
  - how to inspect run history
  - how to interpret `partial`
  - what `VR-TS-006` means

### Manual Validation

Run the app with a small imported Telegram corpus:

```powershell
npm.cmd run tauri dev
```

Validate:

- [ ] `Telegram Summary` appears in the Prompt Pack library.
- [ ] Preflight succeeds on a Telegram channel source with messages.
- [ ] Preflight succeeds on a Telegram chat/supergroup source with reply chains.
- [ ] Preflight blocks non-Telegram sources.
- [ ] Start run produces events in run history.
- [ ] Result contains:
  - short summary
  - timeline
  - key messages
  - topics
  - claims
  - threads when reply chains exist
  - forwarded items when forwarded messages exist
  - limitations when data is incomplete
- [ ] Key messages reference valid `message_ref_id` values.
- [ ] A deliberately invalid fixture yields `partial` and `VR-TS-006`.

### Verification

- [ ] Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::telegram_summary
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::seed::tests
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs::library::tests
npm.cmd run test -- --run src/lib/api/prompt-packs.test.ts
npm.cmd run check
```

- [ ] Run the existing fixture/schema validation command if the repository has one. If there is no single fixture validator script, run the targeted Rust validation tests from Task 7 and document that in the final implementation notes.
- [ ] Mark this task complete in this file.
- [ ] Commit:

```powershell
git add docs/prompt-packs
git commit -m "docs(prompt-packs): document telegram summary implementation"
```

## Final Verification Before Merge

Run all checks below before presenting the branch as complete:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml prompt_packs
cargo test --manifest-path src-tauri/Cargo.toml sources
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export
npm.cmd run test -- --run src/lib/api/prompt-packs.test.ts
npm.cmd run check
```

If any command is too broad or slow for the current workstation, run the nearest targeted subset and record the skipped command plus reason in the final notes.

## Acceptance Criteria

- [ ] `telegram_summary@1.0.0` is seeded as an active bundled Prompt Pack.
- [ ] Prompt Pack library returns `Telegram Summary` with one executable stage.
- [ ] Frontend API exposes preflight and start wrappers.
- [ ] Preflight accepts Telegram channel/chat/supergroup sources and blocks non-Telegram sources.
- [ ] Stage input preserves message refs, reply chains, forum topics, forwarded metadata, and source namespaces.
- [ ] Execution works with fake stage executor tests and real runtime plumbing.
- [ ] Canonical result includes `outputs.pack_data.telegram_summary`.
- [ ] Validator catches dangling message refs with `VR-TS-006`.
- [ ] Valid canonical Telegram fixtures remain valid.
- [ ] Run history can show terminal result, validation findings, and artifacts.
- [ ] Existing `youtube_summary` tests still pass.

## Non-Goals For This Implementation Slice

- No live Telegram syncing changes.
- No new media OCR/STT.
- No author reputation or social graph scoring.
- No separate custom Telegram report viewer.
- No external fact checking beyond the selected corpus.
- No production alerting or realtime monitoring.

## Known Risks And Mitigations

- **Large chats can exceed context limits.** Mitigate with preflight token estimates and deterministic trimming before stage execution.
- **Telegram `message_id` is not globally unique.** Mitigate with `summary_source_id`, native peer identity, and history scope.
- **Reply targets can be missing.** Preserve unresolved metadata as limitations; do not emit invalid `message_ref_id`.
- **Forwarded content can look like independent confirmation.** Validator and prompt rules must label forwarded items separately.
- **Forum topic assignment can be incomplete.** Prefer existing `item_topic_memberships`, then reply-top metadata, then semantic grouping.
- **Prompt output may be structurally valid but weak.** Keep message quality signals optional and preserve validation findings for traceability.
