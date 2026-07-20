# LLM Crate Boundary Design

**Status:** Owner-approved; implementation not started
**Date:** 2026-07-20

**Roadmap authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

**Verification-loop authority:**
[`2026-07-17-focused-rust-loop-design.md`](2026-07-17-focused-rust-loop-design.md)

This specification defines the just-in-time Phase 5 boundary for
`extractum-llm`. It supersedes only the short Phase 5 architecture,
dependency, measurement, and execution placeholder in the crate roadmap. It
does not change the retained Phase 4 result, reopen Phase 3, or alter the
planned layering of the later prompt-pack, analysis, and Telegram crates.

## Purpose

Phase 5 extracts the portable LLM execution engine into `extractum-llm` so
provider, streaming, request, and scheduling work can be checked without
recompiling the application package. Application-specific persistence,
credentials, Tauri IPC, events, and diagnostics remain in `extractum`.

The extraction must preserve current behavior and consumer import paths. It is
an ownership change, not an LLM feature redesign, storage abstraction,
provider rewrite, or IPC migration.

## Decision

The selected boundary is a portable execution crate behind the existing
private application facade:

1. `extractum-llm` owns provider clients, provider/model policy, request and
   completion DTOs, SSE parsing, execution, timeout/retry behavior, and the
   scheduler/cancellation state machine;
2. `extractum` owns profile persistence and lifecycle, secure credential
   storage, Tauri commands and event emission, database access, and diagnostic
   assembly;
3. the private `crate::llm` module remains an explicit compatibility facade so
   current application consumers keep their Rust import paths during this
   slice;
4. no profile-store or secret-store port is introduced.

The app resolves one persisted profile and its credential into a
`ResolvedLlmProfile`, then passes that value to the crate. The crate neither
loads nor persists profiles and cannot access `AppHandle`, SQLx, keyring, or
application state.

## Alternatives Considered

### Full LLM domain behind storage ports

This option would also move profile orchestration and introduce async settings
and credential-store interfaces implemented by the app. It would maximize
crate ownership, but it would add a second abstraction layer around stable
application infrastructure and enlarge the security-sensitive public API.
The current evidence does not justify that cost.

### Provider-neutral kernel only

This option would move only DTOs, validation, and the scheduler while keeping
Gemini and OpenAI-compatible HTTP clients in the app behind a provider port.
It has the smallest visibility change, but leaves most LLM churn in
`extractum` and weakens the focused-loop benefit required by future
`extractum-prompt-packs` and `extractum-analysis`.

### Selected: portable execution crate

Moving the concrete portable clients and scheduler while retaining storage and
IPC app-side follows the observed ownership seam, avoids the Telegram/auth
cycle risk, and provides the useful dependency for Phases 6 and 7 without a
speculative port.

## Fresh Evidence Snapshot

The snapshot was taken on 2026-07-20 at `464c5709` with a clean worktree.

- `src-tauri/src/llm` contains 8 files and 4,717 physical lines: 2,660 lines
  before the test modules and 2,057 lines in test regions.
- The source contains 51 statically inventoried Rust tests: 25 `#[test]` and
  26 `#[tokio::test]`. The implementation plan must confirm the executable
  Cargo inventory before freezing the baseline; a filtered run reporting zero
  tests is not evidence.
- Since 2026-06-01, 19 commits touched `src-tauri/src/llm`. Under the same
  Rust-domain method used by the Phase 4 JIT snapshot, 12 of 19 (63.2%) touched
  no other categorized Rust domain.
- Raw joint-touch counts in that window are `prompt_packs` 4, `analysis` 3,
  `youtube` 3, `telegram` 2, and one each for `accounts`, `secret_store`,
  `telegram_session_store`, `diagnostics`, `gemini_browser`, `process_tree`,
  and `projects`. Broad formatting and secure-storage sweeps are not ownership
  evidence by themselves.
- The roadmap's historical `analysis + llm = 12` and `llm + telegram = 6`
  figures use a wider window and different bucketing. They remain historical
  context; the fresh figures above govern this design.
- There is no direct production import from the current Telegram module into
  `crate::llm`. The live Telegram-related seam is account deletion checking
  scheduler-owned run IDs, while the historical joint work is primarily
  credential and diagnostics infrastructure.

Current production consumers are the application root, analysis chat and
report execution, prompt-pack request/execution adapters, diagnostics, account
deletion, and account commands. The future prompt-pack and analysis crates are
the intended downstream consumers of `extractum-llm`.

## Target Dependency Structure

```text
extractum
  |-- Tauri commands and llm://response events
  |-- SQLx profile persistence and app_settings
  |-- SecretStore/keyring credential ownership
  |-- diagnostics and application wiring
  `-- extractum-llm
        |-- provider clients and model policy
        |-- streaming and execution
        |-- request/completion DTOs
        |-- scheduler and cancellation
        `-- extractum-core::error

future extractum-prompt-packs ----> extractum-llm
future extractum-analysis --------> extractum-llm
```

There is no dependency from `extractum-llm` back to `extractum`. The crate
must not depend on `extractum-prompt-packs`, `extractum-analysis`,
`extractum-telegram`, `extractum-gemini-browser`, or any future application
adapter crate.

The expected direct production dependency roots are:

- `extractum-core`, for the existing `AppError`, `AppErrorKind`, and
  `AppResult` contract;
- `reqwest`;
- `secrecy`;
- `serde` and `serde_json`;
- `tokio` and `tokio-util`.

`reqwest` and `secrecy` are each currently declared exactly once, in the
application `[dependencies]`, and are absent from `[workspace.dependencies]`.
Because the application retains direct uses while `extractum-llm` becomes a
second consumer, Phase 5 must add both canonical declarations to
`[workspace.dependencies]` and convert both the app and new crate to workspace
inheritance. This is a required manifest transition, not cleanup of an
existing duplicate. The pre-manifest inventory must choose and contract-pin
the exact `reqwest` feature placement; no package-local version declaration
may remain afterward.

The implementation plan must derive exact Cargo features from the prepared
moved code before creating the manifest. It must omit unused expected roots
and justify any additional root. `url` is not currently a direct LLM-module
dependency because URL handling uses `reqwest::Url`.

Forbidden crate dependencies include Tauri and its plugins, SQLx, keyring,
Apalis, Apalis-SQLite, `windows-sys`, Grammers, `extractum-gemini-browser`, and
the application package. Manifest changes must include the corresponding
`src-tauri/Cargo.lock` update before any `--locked` validation.

## Ownership Boundary

### Crate-owned portable behavior

`extractum-llm` owns:

- `ProviderKind`, its stable snake-case representation, accepted aliases,
  display name, and safe base-URL normalization;
- provider model DTOs and exact model input/output-limit lookup rules;
- `LlmMessage`, `LlmChatRequest`, `LlmUsage`, `LlmCompletion`, and the
  non-serializable resolved execution profile;
- request validation and effective-model selection;
- Gemini request/response mapping, authentication headers, streaming, model
  listing, retry classification, and error formatting;
- OpenAI-compatible request/response mapping, bearer authentication,
  streaming, model listing, retry classification, and error formatting;
- SSE event-boundary detection and data parsing;
- the 90-second LLM execution timeout and the model-lookup timeouts;
- request scheduling, per-provider/profile concurrency, interactive priority,
  queue positions, snapshots, owner-run tracking, and cancellation;
- stable diagnostic keys derived from scheduler request kinds and states.

### Application-owned integration

`extractum` retains:

- all nine `#[tauri::command]` functions and their registration;
- `AppHandle`, `State`, task spawning for IPC, and `llm://response` emission;
- `LlmProfile`, `LlmProfilesState`, `LlmStreamEvent`, and the app-only stream
  event builder;
- the complete `profiles.rs` persistence/lifecycle implementation, including
  profile-ID validation, credential-scope checks, and all setting-key rules;
- SQLx queries against `app_settings` and database-pool acquisition;
- `SecretStoreState`, key construction, keyring access, and secret deletion;
- provider-profile diagnostic aggregation and mapping into app diagnostic
  DTOs;
- conversion between LLM completions and prompt-pack-specific completion
  values;
- analysis and prompt-pack provenance persistence.

The app may call crate-owned provider parsing and base-URL normalization from
its profile logic, but ownership of the profile lifecycle and side-effect
ordering remains app-side.

## Current-File Disposition

| Current file | Phase 5 disposition |
| --- | --- |
| `gemini.rs` | Move to `extractum-llm`; preserve request mapping, streaming, retry, model-listing, and tests. |
| `openai_compat.rs` | Move to `extractum-llm`; preserve request mapping, streaming, retry, model-listing, and tests. |
| `runner.rs` | Move to `extractum-llm`; preserve validation, provider dispatch, and timeout behavior. |
| `scheduler.rs` | Move to `extractum-llm`; preserve queue, priority, cancellation, snapshot, and owner-run behavior. |
| `streaming.rs` | Move to `extractum-llm`; preserve byte-level SSE parsing behavior. |
| `types.rs` | Split: request/execution/provider DTOs move; profile-state and Tauri stream-event DTOs remain app-side. |
| `profiles.rs` | Stay app-side in full; import only the crate-owned provider/profile execution types it needs. |
| `mod.rs` | Remain the private app facade; move provider/model policy, re-export the curated crate API, and retain commands/events/diagnostics. |

The preparation checkpoint may split mixed files and introduce the safe
resolved-profile construction API while everything still compiles in
`extractum`. After that checkpoint, the physical cross-crate move must be
mechanical. This phase does not authorize unrelated provider, scheduler,
profile, analysis, prompt-pack, or frontend changes.

## Public Rust API

The crate root uses named modules and explicit re-exports. Public glob exports,
public test helpers, and public provider implementation details are forbidden.

The curated surface contains:

- DTOs: `LlmMessage`, `LlmChatRequest`, `LlmUsage`, `LlmCompletion`,
  `LlmProviderModel`, `LlmProviderAccess`, `ResolvedLlmProfile`, and
  `ProviderKind`;
- provider policy: `ProviderKind::parse`, `ProviderKind::as_str`, and
  `normalize_base_url`;
- execution: `validate_request`, `resolve_effective_model`,
  `run_llm_collect_with_profile`, `run_llm_stream_with_profile`,
  `list_provider_models`, `resolve_model_input_token_limit`, and
  `resolve_model_output_token_limit`;
- scheduling: `LlmSchedulerState`, `LlmRequestMetadata`, `LlmRequestKind`,
  `LlmRequestPriority`, `LlmRequestSnapshot`, `LlmRequestSnapshotState`,
  `LlmRequestControl`, and `LlmRequestError`;
- scheduler operations required by current consumers:
  `LlmSchedulerState::new`, `LlmSchedulerState::run_request`,
  `LlmRequestControl::run_cancellable`,
  `LlmSchedulerState::cancel_request`,
  `LlmSchedulerState::cancel_run_requests`,
  `LlmSchedulerState::request_snapshots`, and
  `LlmSchedulerState::active_owner_run_ids`;
- `llm_request_kind_diagnostic_key` and
  `llm_request_state_diagnostic_key`.

Every `pub(crate)`, `pub(super)`, or `pub(in crate::llm)` item widened for the
crate boundary must be listed explicitly in the implementation plan and
covered by a consumer or source contract. Visibility must not be widened merely
to make compilation convenient.

`LlmProviderAccess` and `ResolvedLlmProfile` remain non-serializable and carry
the API key as `SecretString`. Neither type has a public secret field or secret
getter. `LlmProviderAccess::new(provider, api_key, base_url)` is the exact
credential seam for `list_provider_models(&LlmProviderAccess)`. The app-owned
profile adapter constructs it directly from configured or saved values, so it
never extracts a key back out of `ResolvedLlmProfile`.

`ResolvedLlmProfile::new(profile_id, default_model, provider_access)` consumes
that access value for execution. Its public getters are exactly `profile_id`,
`provider`, `default_model`, and `base_url`; provider implementations inside
the crate can read the secret privately. The preparation checkpoint changes
current field access to this API and proves behavior before the move.

The private app facade preserves the `crate::llm::ResolvedLlmProfile` import,
but it cannot preserve construction through a cross-crate struct literal. The
preparation checkpoint must therefore migrate both current external
construction sites while the type still belongs to `extractum`:

- `analysis/report/tests/harness.rs::sample_resolved_profile`;
- the `ResolvedLlmProfile` fixture in
  `prompt_packs/completion_transport.rs::api_model_context_retains_profile_and_override`.

After that checkpoint, no `ResolvedLlmProfile { ... }` literal may remain
outside the owning module. Current evidence finds no `.api_key` read outside
`src-tauri/src/llm`, so removing public field access does not require an
external secret migration.

`LlmRequestMetadata` remains a plain construction DTO because analysis,
prompt-packs, account-deletion tests, and the app command all build it. Its
fields may be public. `LlmCompletion` remains readable by downstream adapters.

## Data Flow

### Stored profile to execution

1. The app reads profile metadata from `app_settings` and obtains the scoped
   credential from `SecretStoreState`.
2. The app preserves current provider/origin validation and constructs one
   `ResolvedLlmProfile`.
3. Analysis and prompt-pack flows keep that resolved value for the lifetime of
   the run, preserving resolve-once behavior and persisted provenance.
4. The crate validates the request, resolves the effective model, schedules
   execution, and calls the selected provider.
5. The crate returns deltas through the existing callback and returns a typed
   completion or `AppError`.
6. The app creates the existing Tauri event or domain-specific completion and
   persists any app-owned result.

### Model listing

The app resolves configured or saved credentials and the effective base URL.
Its profile adapter creates an `LlmProviderAccess` without exposing a getter on
the resolved execution profile. It calls
`list_provider_models(&LlmProviderAccess)`, which owns provider selection,
provider-specific model-listing timeouts, and provider error behavior. The app
returns the unchanged `LlmProviderModel` list through the existing command.

### Scheduling and cancellation

The singleton `LlmSchedulerState` is crate-owned but remains managed by the
Tauri app. App and future domain consumers construct request metadata and call
the crate. Cancellation remains distinct from provider failure through
`LlmRequestError::Cancelled` and `LlmRequestError::Failed(AppError)`.
Diagnostics read typed scheduler snapshots; account deletion reads active
owner-run IDs. Neither consumer reads scheduler internals.

## Compatibility Contract

The extraction must preserve the following observable behavior exactly.

### Tauri and frontend surface

The command names remain:

- `get_llm_profiles`;
- `get_llm_request_snapshots`;
- `save_llm_profile`;
- `clear_llm_profile_api_key`;
- `delete_llm_profile`;
- `set_active_llm_profile`;
- `list_llm_provider_models`;
- `ask_llm_stream`;
- `cancel_llm_request`.

Their frontend-visible camel-case payload and result contract is frozen as:

| Command | Payload keys | Success value |
| --- | --- | --- |
| `get_llm_request_snapshots` | none | `LlmRequestSnapshot[]` |
| `get_llm_profiles` | none | `LlmProfilesState` |
| `save_llm_profile` | `profileId?`, `provider`, `defaultModel`, `apiKey?`, `baseUrl?`, `setActive?` | `LlmProfilesState` |
| `clear_llm_profile_api_key` | `profileId` | `LlmProfilesState` |
| `set_active_llm_profile` | `profileId` | `LlmProfilesState` |
| `delete_llm_profile` | `profileId` | `LlmProfilesState` |
| `list_llm_provider_models` | `provider`, `profileId?`, `apiKey?`, `baseUrl?` | `LlmProviderModel[]` |
| `ask_llm_stream` | `requestId`, `messages`, `modelOverride?`, `profileId?` | `null`/void after validation, profile resolution, and background-task spawn |
| `cancel_llm_request` | `requestId` | `null`/void |

Injected `AppHandle` and Tauri state parameters are not frontend payload keys.
The implementation plan must characterize these payload keys, optional/null
rules, result DTOs, and rejection serialization; preserving only the Rust
function names is insufficient.

The event channel remains `llm://response`. Event kinds remain `queued`,
`started`, `delta`, `completed`, `failed`, and `cancelled`; field names and
optional/null behavior remain snake-case-compatible with `src/lib/types/llm.ts`.
The cancellation event continues to carry `"Request cancelled."`.

`ask_llm_stream` continues to return after spawning background work rather than
waiting for scheduler registration or a terminal provider result. A normally
registered request emits one or more `queued` events as its position is
reported or recomputed. If it starts, it then emits exactly one `started`, zero
or more `delta`, and exactly one terminal `completed`, `failed`, or `cancelled`
event. Cancellation while still queued emits no `started` event. A duplicate
request ID that fails scheduler registration emits a terminal `failed` event
without `queued` or `started`. No logical emission follows a terminal event,
and event-delivery errors continue to be ignored. Validation or profile
resolution that rejects before task spawning returns a command error and emits
no lifecycle event.

### Errors, timeouts, and retries

Phase 5 deliberately differs from Phase 4's `GeminiBrowserError` strategy.
The Gemini Browser crate owns a job lifecycle with domain-specific timeout,
cancellation, protocol, browser, and terminal-outcome distinctions that must
be mapped by its app adapter; introducing its domain error also replaced an
unsafe string-classification boundary. LLM provider and validation failures,
by contrast, already use the shared core `AppError` taxonomy throughout
analysis and prompt-pack consumers, while `LlmRequestError` separately
preserves cancellation versus `Failed(AppError)`. Replacing that established
identity with a second domain error would add widespread translation without
adding a missing semantic distinction.

Future JIT crate designs apply the semantic rule rather than forcing one error
style for consistency: reuse core `AppError` when the shared
validation/auth/network/conflict/internal taxonomy is the actual public
contract; introduce a crate-specific domain error when the crate owns
additional recoverable states or lifecycle outcomes that require an explicit
adapter mapping. Phases 6 and 7 may therefore consume both patterns without
normalizing either one merely for uniformity.

Provider, validation, scheduler, and streaming failures continue to use
`extractum_core::error::AppError { kind, message }`. A rejected Tauri command
serializes that object with the existing snake-case kind. By contrast,
`llm://response.error` remains `Option<String>`/a nullable string: the `failed`
event contains `AppError::to_string()`, not a serialized `{ kind, message }`
object. The implementation plan must add or preserve characterization tests
for both distinct forms and their exact outgoing messages, not merely
classification predicates.

The 90-second execution timeout, provider model-listing timeouts, five-second
limit lookup, bounded transient retry policies, and existing provider error
messages remain unchanged. No string-prefix error classification is introduced
by this extraction.

### Profiles and persistence

The following remain unchanged:

- active-profile key `llm.active_provider_profile`;
- per-profile keys `llm.profile.<id>.provider`,
  `llm.profile.<id>.default_model`, and `llm.profile.<id>.base_url`;
- secure credential key `llm.profile.<id>.api_key`;
- accepted profile IDs, provider aliases, default provider/model/base URL, and
  remote-plaintext URL rejection;
- binding an existing key to the same provider and normalized origin unless a
  replacement key is supplied or the old key is cleared;
- secret-first deletion failure behavior and the current setting-write order;
- the rule that IPC exposes `api_key_configured`, never the key;
- analysis and prompt-pack stored provider-profile/model provenance.

### Scheduler serialization and behavior

Request-kind values remain `provider_test`, `analysis_chat`,
`analysis_report_map`, `analysis_report_reduce`, and `prompt_pack_stage`.
Snapshot states remain `queued` and `running`. Priority ordering, per-key
concurrency, stable queue positions, capacity release after failure, typed
cancellation, and active owner-run reporting remain unchanged.

## Test Inventory and Ownership

The baseline inventory contains exactly 51 named tests. The approved
disposition is 36 tests in `extractum-llm` and 15 tests in `extractum`.
Every baseline name must appear exactly once after extraction; renamed copies,
disabled copies, and duplicate implementations do not satisfy this rule.

### Crate-owned tests (36)

From `gemini.rs`:

- `gemini_request_mapping_keeps_system_history_and_roles`
- `gemini_request_mapping_keeps_existing_messages_without_output_limit`
- `gemini_stream_chunk_text_and_usage_are_parsed`
- `gemini_model_mapping_uses_short_model_id`
- `gemini_request_rejects_unsupported_roles_with_typed_validation_error`
- `gemini_model_listing_requires_typed_auth_error`
- `gemini_server_error_message_includes_transient_recovery_hint`

From `openai_compat.rs`:

- `openai_compat_request_keeps_standard_roles`
- `openai_compat_stream_chunk_mapping_reads_delta_and_usage`
- `openai_compat_model_mapping_uses_model_id`
- `openai_compat_model_mapping_reads_omniroute_limits_and_capabilities`
- `openai_compat_request_rejects_unsupported_roles_with_typed_validation_error`
- `openai_compat_retry_status_policy_is_bounded_to_transient_failures`
- `openai_compat_stream_retries_transient_http_before_streaming`
- `openai_compat_model_listing_requires_typed_auth_error`

From `runner.rs`:

- `validate_request_returns_typed_validation_error`
- `resolve_effective_model_returns_typed_validation_error`
- `run_llm_collect_returns_typed_validation_error`

From `scheduler.rs`:

- `requests_with_different_profiles_run_without_blocking_each_other`
- `interactive_requests_jump_ahead_of_background_queue`
- `queued_requests_can_be_cancelled_before_start`
- `cancelling_owned_run_requests_aborts_running_work`
- `request_snapshots_report_running_and_queued_requests`
- `active_owner_run_ids_reports_running_and_queued_owned_requests`
- `queue_positions_are_recomputed_after_cancelling_a_queued_request`
- `failed_requests_release_capacity_for_next_queued_request`
- `failed_requests_preserve_typed_error_kind`

From `streaming.rs`:

- `sse_data_is_parsed_from_stream_chunks`
- `sse_data_decode_failures_are_typed_internal_errors`

From current `mod.rs` provider/model/scheduler-key policy:

- `provider_parse_returns_typed_validation_error`
- `provider_parse_accepts_openai_compatible_aliases`
- `model_input_token_limit_lookup_matches_provider_model_ids_and_names`
- `model_output_token_limit_lookup_matches_provider_model_ids_and_names`
- `normalize_base_url_returns_typed_validation_error`
- `normalize_base_url_allows_https_and_loopback_http_only`
- `llm_request_diagnostic_keys_are_stable_snake_case`

### Application-owned tests (15)

From `profiles.rs`:

- `profile_settings_roundtrip_stores_api_key_in_secret_store`
- `active_profile_resolution_loads_key_from_secret_store`
- `legacy_remote_http_profile_is_rejected_before_request_configuration`
- `changing_key_scope_without_replacement_is_rejected`
- `keyed_legacy_profile_materializes_effective_base_url_while_unkeyed_stays_blank`
- `credential_scope_uses_provider_origin_and_effective_port_but_not_path`
- `materialization_write_failure_fails_closed_during_state_load`
- `profile_state_lists_multiple_saved_profiles`
- `validate_profile_id_rejects_invalid_characters`
- `set_active_profile_returns_typed_not_found_error`
- `empty_save_preserves_existing_secret`
- `clear_profile_api_key_deletes_secret`
- `delete_profile_removes_settings_and_secret_and_resets_active`
- `delete_profile_fails_if_secret_store_fails_leaving_db_settings_intact`

From current `mod.rs` app diagnostics:

- `provider_diagnostics_exclude_profile_ids_and_base_urls`

The plan may add new characterization and cross-crate contract tests. Those
new tests do not change the exact disposition of the 51-name baseline.

## Source and Manifest Contracts

A dedicated Vitest source-boundary contract must verify at minimum:

- the exact workspace member and single app path dependency;
- the new package's `Cargo.lock` entry and the app lock dependency;
- direct dependency roots and exact feature ownership;
- absence of Tauri, SQLx, keyring, Apalis, Grammers, process, app, analysis,
  prompt-pack, Telegram, and Gemini Browser imports from the crate;
- explicit crate-root exports and the approved visibility map;
- the secret fields have no public accessor and `LlmProviderAccess` plus
  `ResolvedLlmProfile` are not serializable;
- all nine commands and `llm://response` remain app-owned;
- `profiles.rs`, profile SQL keys, and secret-store calls remain app-owned;
- every frozen test name appears exactly once in the approved owner;
- moved implementations are absent from the app except for the explicit
  facade and adapters;
- there are no `#[cfg(any())]`, commented, or otherwise compile-disabled
  ownership copies detectable by the frozen source patterns.

Because a source scanner cannot prove semantic absence of an arbitrarily
renamed copy, the mechanical-move diff and an explicit manual ownership review
must separately prove that no renamed implementation or test copy remains.

The plan must inventory and update the workspace-member array in
`rust-workspace-core-contract.test.ts`, the workspace-member string in
`gemini-browser-crate-boundary-contract.test.ts`, and that file's exact
`[workspace.dependencies]` name/text allowlist. The latter must explicitly add
the new canonical `reqwest` and `secrecy` roots while the app assertions change
from their current single package-local version declarations to workspace
inheritance. The plan must also update the scheduler/model ownership paths in
`docs/value-registry.md` and inventory any source contract that reads a current
`src-tauri/src/llm` path before the physical move, so `npm.cmd run verify` is
not the first stale-path detector.

## Implementation Shape

The implementation plan must use these checkpoints:

1. freeze the clean baseline, exact 51-test inventory, consumer map, IPC/error
   characterization, dependency roots, and visibility map;
2. prepare mixed-file seams and the safe `ResolvedLlmProfile`
   plus `LlmProviderAccess` APIs while all production code still belongs to
   `extractum`; migrate
   `analysis/report/tests/harness.rs::sample_resolved_profile` and
   `prompt_packs/completion_transport.rs::api_model_context_retains_profile_and_override`
   away from struct literals, prove no external literal remains, then run exact
   RED/GREEN additions, baseline characterization tests, and the app package
   checkpoint;
3. add a RED source-boundary contract that expects the new crate and exact
   ownership;
4. run and restore the complete baseline advisory timing series against the
   prepared portable source in `extractum`, including that state's SHA-256 and
   clean-worktree proofs;
5. create the workspace member, manifests, and lock update, then move the
   prepared portable sources and tests without copying behavior;
6. restore the private app `crate::llm` facade and retained integration code;
7. run crate-focused checks/tests, then the immediate dependent `extractum`
   checkpoint for the public cross-crate interface;
8. run and restore the complete candidate advisory timing series against the
   same logical source in `extractum-llm`, including that state's SHA-256 and
   clean-worktree proofs;
9. run the complete source-contract and workspace gates, release/startup
   evidence, then record a verification document and roadmap result.

Each checkpoint must leave the worktree explainable. A preparation commit may
reshape internal APIs but must not create the crate. The physical extraction
commit must not include unrelated behavior changes.

## Rust Verification Loops

The implementation plan changes Rust and must include a `## Rust Verification
Loops` section compliant with the standing repository policy.

Affected packages are initially `extractum`; after creation they are
`extractum-llm` and its immediate consumer `extractum`.

The plan must name non-empty exact RED/GREEN additions for:

- resolved-profile/provider-access construction and secret non-exposure;
- Tauri event/error serialization gaps identified by the frozen compatibility
  inventory;
- the source-boundary contract.

Existing provider parsing, base-URL, request validation, provider retry,
scheduler, and profile-persistence tests are characterization tests. The plan
must run named non-empty selections before and after the move, but must not
manufacture a RED failure for already-correct behavior.

After a small crate change, use:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib <full-test-name> -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
```

The package checkpoint is:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
```

Every public cross-crate interface change also requires the immediate consumer
checkpoint:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

End-of-slice completion requires:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

Focused passes are accelerators, not completion evidence. All packages continue
to share canonical `src-tauri/target`.

## Advisory Compile-Time Measurement

Phase 5 follows the current focused-loop policy and must not recreate the
retired shell-cap machinery.

The matched logical probe is one inert edit in a portable LLM source that
belongs to `extractum` before extraction and to `extractum-llm` afterward.
Baseline and candidate each use sequential Cargo commands against the shared
canonical target:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
```

Only the command appropriate to the current state is run. Each state uses one
discarded warm-up and three recorded samples. Record raw values, median of
three, absolute delta, and percentage delta.

The probe must be restored byte-for-byte in a `finally` path. After each
state's complete series, record one SHA-256 source proof and one clean-worktree
proof before changing ownership or starting the other state. The procedure
must not build a new quiet-window, Job Object, process scanner,
Defender/power capture, stability retry, shell A/B, cumulative ledger, or
per-sample artifact system.

Timing is advisory: timing alone never rejects, reverts, retains, or weakens
the correctness requirements of the slice. A command failure or unproven
restoration ends the measurement, preserves any observed raw values, and
produces no median or performance conclusion; there is no protocol-mandated
retry.

Record the duration already emitted by the successful mandatory end-of-slice
workspace check. Phase 4's ordinary result was 1,620 ms, below 15,000 ms, so
Phase 5 cannot by itself satisfy the two-adjacent-slice trigger. Two adjacent
completed crate-extraction slices whose ordinary workspace-check results are
each at or above 15,000 ms trigger a separate owner-approved performance
investigation. Do not rerun the check or add samples for that rule, and do not
fail or revert either slice on timing alone.

## Release and Runtime Evidence

Full MSI bundling remains outside the gate because of the documented
pre-existing WiX `light.exe` failure. Release evidence uses:

```powershell
npm.cmd run tauri -- build --no-bundle
```

The plan must also run a bounded startup smoke that proves the built application
starts and remains alive long enough for initialization. Startup-runner
infrastructure failure is not candidate failure; confirmed early exit of the
application is a completion failure. Any helper process started by the smoke
must be stopped and reaped before completion.

No live provider request is required by default because it would depend on a
user credential and external service. If the implementation changes behavior
beyond the characterized request/response adapters, that expansion requires a
separate owner-approved smoke design.

## Failure and Rollback

- A baseline failure stops the slice before crate creation.
- A preparation failure is fixed while ownership remains in `extractum`; it
  must not be hidden inside the mechanical move.
- A crate-focused or immediate-consumer failure is a candidate correctness
  failure and blocks retention.
- Any end-of-slice workspace, frontend, release, or confirmed application
  startup failure leaves the slice incomplete.
- A measurement failure or advisory regression is recorded but does not fail
  or revert a correct candidate.
- If the extraction is not retained, restore the last clean pre-extraction
  checkpoint and prove the workspace member, path dependency, crate files, and
  lock entry are absent. Do not leave duplicate or disabled source ownership.
- Never weaken gates, change IPC strings, widen dependency allowances, or
  reinterpret a failed check after observing candidate results.

## Acceptance Criteria

Phase 5 is complete only when all of the following are true:

1. `extractum-llm` exists as one workspace member and the app has one path
   dependency on it.
2. The dependency graph follows this design and contains no forbidden edge.
3. Portable provider, runner, streaming, DTO, and scheduler behavior has one
   owner in the crate.
4. Profiles, credentials, Tauri IPC/events, SQLx, and diagnostic aggregation
   remain app-owned.
5. The private app facade preserves existing Rust import paths, and both
   external resolved-profile struct literals were migrated during preparation.
6. The secrets inside `LlmProviderAccess` and `ResolvedLlmProfile` are neither
   serializable nor publicly readable.
7. All nine command payload/result contracts, six-event lifecycle, distinct
   command/event error shapes and messages, profile keys, provenance,
   timeout/retry, and scheduler behaviors remain compatible.
8. Every one of the 51 frozen baseline tests appears exactly once: 36 in the
   crate and 15 in the app.
9. The source-boundary contract and every affected existing allowlist pass.
10. Focused crate and immediate-consumer checkpoints pass with non-empty tests.
11. Rustfmt, workspace check, workspace tests, and `npm.cmd run verify` pass.
12. Release no-bundle and bounded startup evidence pass.
13. Advisory timing is recorded or honestly classified as incomplete after
    exact probe restoration; timing does not decide retention.
14. A verification document records final ownership, dependencies, test
    inventory, gate results, timing disposition, workspace-check duration, and
    roadmap outcome.

## Non-Goals

Phase 5 does not:

- redesign profile storage or introduce repository/service traits;
- move SQLx, `app_settings`, SecretStore, keyring, Tauri, or diagnostics into
  the crate;
- add providers, change models, alter retry counts, or change timeouts;
- change frontend LLM types, commands, events, or user flows;
- move analysis, prompt-pack, Telegram, account, or Gemini Browser behavior;
- recreate `extractum-process` or add process infrastructure;
- add a live credentialed provider test as a default gate;
- use timing as an automatic retention decision.

## Resulting Plan

After this specification is reviewed, Phase 5 requires a separate detailed
implementation plan. The plan must preserve the preparation/mechanical-move
separation, enumerate every visibility change and all 51 baseline test names,
include the required `## Rust Verification Loops` section, and use the small
advisory timing protocol above.
