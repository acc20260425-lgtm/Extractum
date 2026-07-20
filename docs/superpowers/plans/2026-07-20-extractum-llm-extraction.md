# Extractum LLM Crate Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the portable LLM execution engine into `extractum-llm` while preserving the existing profile, credential, Tauri IPC/event, provider, scheduler, error, and Rust-consumer behavior.

**Architecture:** `extractum-llm` owns provider/model policy, portable request and completion DTOs, concrete Gemini and OpenAI-compatible clients, SSE parsing, execution, timeouts/retries, and scheduler/cancellation. `extractum` remains the only owner of SQLx profile persistence, `app_settings`, `SecretStoreState`/keyring credentials, all nine Tauri commands, `llm://response`, and diagnostic aggregation. A private `crate::llm` facade preserves current application import paths; a safe `LlmProviderAccess`/`ResolvedLlmProfile` seam crosses the crate boundary without exposing or serializing API keys.

**Tech Stack:** Rust 2021, Cargo workspace, `extractum-core`, Reqwest 0.12, Secrecy 0.8, Serde/serde_json, Tokio/tokio-util, Tauri 2 in the app adapter, Vitest source-boundary contracts, PowerShell on Windows.

## Global Constraints

- Authority: [approved LLM boundary specification](../specs/2026-07-20-llm-crate-boundary-design.md), [crate roadmap](../specs/2026-07-17-crate-roadmap.md), and [focused Rust loop](../specs/2026-07-17-focused-rust-loop-design.md).
- Execute one Phase 5 slice on one implementation branch/worktree. Do not reopen Phase 3 or rebuild any retired shell-cap, quiet-window, Job Object, scanner, retry, or cumulative-ledger machinery.
- Use the checkout's canonical `src-tauri/target`; run Cargo commands sequentially and never set `CARGO_TARGET_DIR`.
- Preserve all nine Tauri command names/signatures, `llm://response`, the six event kinds, optional/null fields, current command-error object shape, event-error string shape, profile keys, credential scoping, side-effect order, provider aliases, retry counts, and timeout values.
- Preserve `ask_llm_stream` lifecycle semantics: return after spawn; queued before started; zero or more deltas; exactly one terminal event; queued cancellation has no started event; duplicate registration may fail without queued/started; emit failures remain ignored.
- Do not move Tauri, SQLx, `app_settings`, SecretStore/keyring, profile lifecycle, app diagnostics, analysis, prompt-pack, Telegram, account, Gemini Browser, or provenance persistence into the new crate.
- Do not introduce a profile-store/secret-store port, a new domain error, a provider redesign, a live credentialed provider gate, or an unrelated frontend/database change.
- `LlmProviderAccess` and `ResolvedLlmProfile` derive only `Clone`. They are not serializable or debuggable, and neither exposes a public API-key field, getter, test helper, or `ExposeSecret` path.
- `extractum-llm` reuses `extractum_core::error::{AppError, AppErrorKind, AppResult}`. Cancellation remains `LlmRequestError::Cancelled`; provider/validation failure remains `LlmRequestError::Failed(AppError)`.
- The frozen baseline is exactly 51 unique test names with the 36/15 disposition in Appendix A. Added characterization/contract tests do not alter that baseline count.
- Resolve dependency roots/features and every consumer change while code is still app-owned. Once Task 3 is committed, Task 6 moves prepared code without behavioral edits.
- `reqwest` and `secrecy` become new canonical workspace dependency roots. Both the app and new crate inherit them; no package-local version remains. Commit `src-tauri/Cargo.lock` before any `--locked` validation.
- Timing is advisory: one discarded warm-up plus three recorded samples per state on the same LF source and inert marker. A timing failure yields `incomplete / no conclusion`; it never rejects, reverts, or retains a correct slice.
- Keep commits scoped. Before each commit inspect `git status --short` and `git diff --cached --stat`; stage only files listed by that task. Never include `.playwright-mcp/`.
- Treat every native command as fail-fast: before running the next native command, inspect `$LASTEXITCODE` and throw on any unexpected nonzero value. An explicitly expected RED or an `rg` no-match sentinel must validate its exact expected exit code before continuing.

## Failure and Rollback

- A baseline inventory, characterization, preparation, boundary-contract parse, package, workspace, release-build, or startup failure stops retention. Fix failures only in the task that owns the behavior; do not hide them in the mechanical move.
- An advisory timing failure is not a correctness failure. Restore the probe byte-for-byte, record `incomplete / no conclusion`, and continue only after the clean-tree proof succeeds.
- If Task 6 fails before its extraction commit exists and the owner chooses not to retain Phase 5, first prove every dirty path belongs to Task 6, stage that allowlisted dirty state, and create `chore: preserve failed LLM extraction attempt` with `--no-verify`; revert that evidence commit immediately. This returns the tree to the clean Task 4/5 checkpoint while preserving the failed candidate in history. Then revert the unique `test: define LLM crate boundary` commit so the intentionally RED contract does not remain active.
- If the extraction commit already exists, resolve the unique commits by exact subjects `refactor: extract portable LLM engine` and `test: define LLM crate boundary`, then revert the extraction commit first and the RED-contract commit second. Never use `git reset`, force deletion, or path checkout to erase evidence. Before either rollback route, stop if an exact subject is absent/non-unique or a dirty path falls outside Task 6's file map.
- After a non-retained rollback, prove that the workspace has no `crates/extractum-llm` member, app path edge, crate directory, or `extractum-llm` lock package; rerun the prepared app package check/test and the existing workspace/Gemini Browser/shell-cap contracts. Write a verification disposition, update Phase 5 in the roadmap as not retained, and do not claim the crate exists.

For the uncommitted-Task-6 route, run this only after the owner selects
non-retention:

```powershell
$unstaged = @(git diff --name-only)
if ($LASTEXITCODE -ne 0) { throw 'Failed to inventory unstaged rollback paths' }
$staged = @(git diff --cached --name-only)
if ($LASTEXITCODE -ne 0) { throw 'Failed to inventory staged rollback paths' }
$untracked = @(git ls-files --others --exclude-standard)
if ($LASTEXITCODE -ne 0) { throw 'Failed to inventory untracked rollback paths' }
$dirtyPaths = @($unstaged + $staged + $untracked | Sort-Object -Unique)
$allowedPattern = '^(?:src-tauri/Cargo\.(?:toml|lock)|src-tauri/crates/extractum-llm(?:/.*)?|src-tauri/src/llm/.*|src/lib/(?:llm-crate-boundary-contract|rust-workspace-core-contract|gemini-browser-crate-boundary-contract)\.test\.ts|docs/value-registry\.md)$'
$unexpected = @($dirtyPaths | Where-Object { ($_ -replace '\\', '/') -notmatch $allowedPattern })
if ($unexpected.Count -ne 0) { throw "Unexpected rollback path(s):`n$($unexpected -join "`n")" }
if ($dirtyPaths.Count -ne 0) {
  git add -A -- .
  if ($LASTEXITCODE -ne 0) { throw 'Failed to stage failed-candidate evidence' }
  git commit --no-verify -m "chore: preserve failed LLM extraction attempt"
  if ($LASTEXITCODE -ne 0) { throw 'Failed to commit failed-candidate evidence' }
  $evidenceCommit = @(git rev-parse HEAD)
  if ($LASTEXITCODE -ne 0 -or $evidenceCommit.Count -ne 1) { throw 'Failed to resolve evidence commit' }
  git revert --no-edit $evidenceCommit[0]
  if ($LASTEXITCODE -ne 0) { throw 'Failed to revert failed-candidate evidence commit' }
}
$history = @(git log --format='%H%x09%s')
if ($LASTEXITCODE -ne 0) { throw 'Failed to inspect rollback history' }
$redMatches = @($history | Where-Object { ($_ -split "`t", 2)[1] -eq 'test: define LLM crate boundary' })
if ($redMatches.Count -ne 1) { throw "Expected one RED-contract commit, found $($redMatches.Count)" }
$redCommit = ($redMatches[0] -split "`t", 2)[0]
git revert --no-edit $redCommit
if ($LASTEXITCODE -ne 0) { throw 'Failed to revert RED boundary contract' }
```

If the extraction commit exists, run the self-contained committed route:

```powershell
$rollbackStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Failed to inspect committed-candidate rollback status' }
if ($rollbackStatus.Count -ne 0) { throw "Committed-candidate rollback requires a clean worktree:`n$($rollbackStatus -join "`n")" }
$history = @(git log --format='%H%x09%s')
if ($LASTEXITCODE -ne 0) { throw 'Failed to inspect committed-candidate history' }
$extractionMatches = @($history | Where-Object { ($_ -split "`t", 2)[1] -eq 'refactor: extract portable LLM engine' })
$redMatches = @($history | Where-Object { ($_ -split "`t", 2)[1] -eq 'test: define LLM crate boundary' })
if ($extractionMatches.Count -ne 1) { throw "Expected one extraction commit, found $($extractionMatches.Count)" }
if ($redMatches.Count -ne 1) { throw "Expected one RED-contract commit, found $($redMatches.Count)" }
$extractionCommit = ($extractionMatches[0] -split "`t", 2)[0]
$redCommit = ($redMatches[0] -split "`t", 2)[0]
git revert --no-edit $extractionCommit
if ($LASTEXITCODE -ne 0) { throw 'Failed to revert retained extraction candidate' }
git revert --no-edit $redCommit
if ($LASTEXITCODE -ne 0) { throw 'Failed to revert RED boundary contract' }
```

After either route, run the exact non-retention proof:

```powershell
if (Test-Path -LiteralPath 'src-tauri/crates/extractum-llm') { throw 'Rollback left the extractum-llm directory' }
$residue = rg -n "crates/extractum-llm|extractum-llm\s*=|name = \"extractum-llm\"" src-tauri/Cargo.toml src-tauri/Cargo.lock
if ($LASTEXITCODE -eq 0) { throw "Rollback left manifest/lock residue:`n$residue" }
if ($LASTEXITCODE -ne 1) { throw 'Rollback residue scan failed' }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Rolled-back app check failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Rolled-back app test failed' }
npm.cmd run test -- src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Rolled-back existing contract group failed' }
$rollbackStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Failed to inspect final rollback status' }
if ($rollbackStatus.Count -ne 0) { throw "Rollback left a dirty worktree:`n$($rollbackStatus -join "`n")" }
```

## Final File Map

```text
src-tauri/crates/extractum-llm/
|-- Cargo.toml
`-- src/
    |-- lib.rs
    |-- provider.rs
    |-- types.rs
    |-- gemini.rs
    |-- openai_compat.rs
    |-- runner.rs
    |-- scheduler.rs
    `-- streaming.rs

src-tauri/src/llm/
|-- mod.rs
|-- profiles.rs
`-- app_types.rs

src/lib/llm-crate-boundary-contract.test.ts
docs/superpowers/verification/2026-07-20-extractum-llm-extraction.md
```

`app_types.rs` owns `LlmProfile`, `LlmProfilesState`, and `LlmStreamEvent`. The seven portable files are prepared in place first, then moved without copied implementations or tests.

## Exact Cross-Crate Interfaces

The preparation checkpoint implements these shapes before `extractum-llm` exists. Names and visibility are fixed.

```rust
#[derive(Clone)]
pub struct LlmProviderAccess {
    provider: ProviderKind,
    api_key: SecretString,
    base_url: String,
}

impl LlmProviderAccess {
    pub fn new(
        provider: ProviderKind,
        api_key: SecretString,
        base_url: String,
    ) -> Self {
        Self {
            provider,
            api_key,
            base_url,
        }
    }

    pub(super) fn provider(&self) -> ProviderKind {
        self.provider
    }

    pub(super) fn api_key(&self) -> &SecretString {
        &self.api_key
    }

    pub(super) fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[derive(Clone)]
pub struct ResolvedLlmProfile {
    profile_id: String,
    default_model: String,
    provider_access: LlmProviderAccess,
}

impl ResolvedLlmProfile {
    pub fn new(
        profile_id: String,
        default_model: String,
        provider_access: LlmProviderAccess,
    ) -> Self {
        Self {
            profile_id,
            default_model,
            provider_access,
        }
    }

    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    pub fn provider(&self) -> ProviderKind {
        self.provider_access.provider()
    }

    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    pub fn base_url(&self) -> &str {
        self.provider_access.base_url()
    }

    pub(super) fn provider_access(&self) -> &LlmProviderAccess {
        &self.provider_access
    }
}
```

`pub(super)` above exposes credentials only to sibling implementation modules through their common crate parent. It never exposes them through `extractum_llm` or the app facade.

Provider policy exports:

```rust
pub async fn list_provider_models(
    access: &LlmProviderAccess,
) -> AppResult<Vec<LlmProviderModel>>;

pub async fn resolve_model_input_token_limit(
    profile: &ResolvedLlmProfile,
    model: &str,
) -> Option<usize>;

pub async fn resolve_model_output_token_limit(
    profile: &ResolvedLlmProfile,
    model: &str,
) -> Option<i64>;

pub fn normalize_base_url(
    provider: ProviderKind,
    base_url: Option<&str>,
) -> AppResult<String>;
```

The provider module keeps this helper private so five-second model-limit lookup does not call the public 30-second wrapper:

```rust
async fn list_provider_models_without_timeout(
    access: &LlmProviderAccess,
) -> AppResult<Vec<LlmProviderModel>>;
```

The app-owned profile seam is exact:

```rust
struct ResolvedProfileMaterial {
    profile_id: String,
    provider: ProviderKind,
    default_model: String,
    api_key: SecretString,
    base_url: String,
}

async fn resolve_profile_material_from_pool(
    pool: &Pool<Sqlite>,
    secret_store: &SecretStoreState,
    requested_profile_id: Option<&str>,
) -> AppResult<ResolvedProfileMaterial>;

pub(super) async fn resolve_provider_access_from_pool(
    pool: &Pool<Sqlite>,
    secret_store: &SecretStoreState,
    provider: ProviderKind,
    requested_profile_id: Option<&str>,
    configured_api_key: Option<SecretString>,
    configured_base_url: Option<String>,
) -> AppResult<LlmProviderAccess>;
```

`list_llm_provider_models` must retain the current fast path: if both a non-empty configured key and configured base URL are supplied, normalize the URL and construct `LlmProviderAccess` without calling `get_pool()`. If either is absent, acquire the pool once and call the adapter. Resolve the two fields independently: key = configured key, else matching saved key, else empty; URL = configured normalized URL, else matching saved URL, else the requested provider's normalized default. A mismatching saved profile contributes neither field but never discards a configured field.

The final crate root is byte-for-byte:

```rust
mod gemini;
mod openai_compat;
mod provider;
mod runner;
mod scheduler;
mod streaming;
mod types;

pub use provider::{
    list_provider_models, normalize_base_url, resolve_model_input_token_limit,
    resolve_model_output_token_limit, ProviderKind,
};
pub use runner::{
    resolve_effective_model, run_llm_collect_with_profile, run_llm_stream_with_profile,
    validate_request,
};
pub use scheduler::{
    llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key, LlmRequestControl,
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority,
    LlmRequestSnapshot, LlmRequestSnapshotState, LlmSchedulerState,
};
pub use types::{
    LlmChatRequest, LlmCompletion, LlmMessage, LlmProviderAccess, LlmProviderModel,
    LlmUsage, ResolvedLlmProfile,
};
```

No `pub mod`, glob export, provider-specific HTTP function, SSE helper, internal scheduler entry, or test helper is public.

Visibility widened only for the cross-crate edge:

- `ProviderKind`, `ProviderKind::parse`, `ProviderKind::as_str`, and `normalize_base_url`;
- `validate_request`, `resolve_effective_model`, `run_llm_collect_with_profile`, and `run_llm_stream_with_profile`;
- `LlmCompletion` and its readable fields;
- `LlmRequestMetadata` and its construction fields;
- `LlmRequestControl`, `LlmRequestError`, `request_snapshots`, and `active_owner_run_ids`; narrow the currently public `LlmRequestControl::is_cancelled` to private so the only public `LlmRequestControl` method is `run_cancellable`;
- the new model-list/model-limit functions and the two diagnostic-key functions.

`ProviderKind::display_name`, provider configs/functions, secret access, SSE helpers, scheduler internals, and constructors for scheduler controls remain internal.

The final private app facade keeps existing names, including the two backend aliases:

```rust
mod app_types;
mod profiles;

pub use app_types::{LlmProfile, LlmProfilesState, LlmStreamEvent};
pub use extractum_llm::{LlmChatRequest, LlmMessage, LlmProviderModel, LlmUsage};
pub(crate) use extractum_llm::{
    llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key,
    normalize_base_url, resolve_effective_model,
    resolve_model_input_token_limit as resolve_model_input_token_limit_for_backend,
    resolve_model_output_token_limit as resolve_model_output_token_limit_for_backend,
    run_llm_collect_with_profile, run_llm_stream_with_profile, validate_request,
    LlmCompletion, LlmProviderAccess, LlmRequestError, LlmRequestKind,
    LlmRequestMetadata, LlmRequestPriority, LlmRequestSnapshot,
    LlmRequestSnapshotState, LlmSchedulerState, ProviderKind, ResolvedLlmProfile,
};
```

Add only a local `use extractum_llm::list_provider_models;` for the app command; do not widen it through the facade without a consumer.

## Manifest Contract

The final workspace roots are:

```toml
[workspace.dependencies]
parking_lot = "0.12"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
secrecy = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
time = { version = "0.3", features = ["formatting", "parsing", "macros"] }
tokio = "1"
tokio-util = "0.7"
url = "2"
zstd = "0.13"
```

The app replaces package-local `reqwest`/`secrecy` declarations and adds one edge:

```toml
reqwest = { workspace = true }
secrecy = { workspace = true }
extractum-llm = { path = "crates/extractum-llm" }
```

The new manifest is:

```toml
[package]
name = "extractum-llm"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
extractum-core = { path = "../extractum-core" }
reqwest.workspace = true
secrecy.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio = { workspace = true, features = ["macros", "sync", "time"] }
tokio-util.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["io-util", "net", "rt", "test-util"] }
```

Expected `extractum-llm` lock dependencies, sorted, are `extractum-core`, `reqwest`, `secrecy`, `serde`, `serde_json`, `tokio`, and `tokio-util`.

## Rust Verification Loops

Affected packages are `extractum` before the move, then `extractum-llm` and immediate consumer `extractum` after the public edge exists.

Exact RED/GREEN safe-profile test before the move:

```powershell
$redOutput = @(cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::types::tests::resolved_profile_construction_preserves_execution_access_and_public_metadata -- --exact 2>&1)
$redExit = $LASTEXITCODE
$redOutput | Out-Host
if ($redExit -eq 0) { throw 'Expected safe-profile constructor/getter RED, but test passed' }
if ($redOutput -match 'running 0 tests') { throw 'Safe-profile RED selected 0 tests' }
if ($redOutput -notmatch 'LlmProviderAccess|ResolvedLlmProfile') { throw 'Safe-profile RED failed for an unexpected reason' }
```

The initial RED must be an unresolved `LlmProviderAccess`/constructor/getter or a failed value assertion, never `0 tests`.

Exact app characterization tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::llm_stream_events_serialize_exact_lifecycle_contract -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url -- --exact
```

Named pre-move characterization selections, after portable modules are
prepared but still app-owned:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::provider::tests::provider_parse_returns_typed_validation_error -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::provider::tests::normalize_base_url_allows_https_and_loopback_http_only -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::runner::tests::validate_request_returns_typed_validation_error -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::openai_compat::tests::openai_compat_stream_retries_transient_http_before_streaming -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::scheduler::tests::failed_requests_preserve_typed_error_kind -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store -- --exact
```

Preparation checkpoint:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Post-move exact crate tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib types::tests::resolved_profile_construction_preserves_execution_access_and_public_metadata -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib provider::tests::provider_parse_returns_typed_validation_error -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib provider::tests::normalize_base_url_allows_https_and_loopback_http_only -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib runner::tests::validate_request_returns_typed_validation_error -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib scheduler::tests::failed_requests_preserve_typed_error_kind -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib openai_compat::tests::openai_compat_stream_retries_transient_http_before_streaming -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib streaming::tests::sse_data_decode_failures_are_typed_internal_errors -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store -- --exact
```

Package and immediate-consumer checkpoints:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

End-of-slice completion gates:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

Focused commands are accelerators only. The slice is incomplete until all four completion gates pass.

---

### Task 1: Freeze the Baseline and Characterize IPC/Error Output

**Files:**

- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src/lib/api/llm.test.ts`
- Read: `src-tauri/src/llm/{gemini.rs,openai_compat.rs,profiles.rs,runner.rs,scheduler.rs,streaming.rs,types.rs}`
- Read: `src/lib/api/llm.ts`
- Read: `src/lib/types/llm.ts`
- Read: `src-tauri/Cargo.toml`

**Interfaces:**

- Consumes: the 51-name baseline, current app-only stream builder, `AppError` serialization, command wrappers, and current Cargo roots.
- Produces: exact outgoing event/error fixtures and a durable pre-move evidence checkpoint; no new crate or production behavior.

- [ ] **Step 1: Prove the branch is clean and freeze the executable 51-name inventory.**

```powershell
$initialStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Initial git status failed' }
if ($initialStatus.Count -ne 0) { throw "Task 1 requires a clean worktree:`n$($initialStatus -join "`n")" }
$listed = cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list
if ($LASTEXITCODE -ne 0) { throw 'Baseline test listing failed' }
$actual = @(
  $listed |
    Select-String '^llm::.*: test$' |
    ForEach-Object { (($_.Line -replace ': test$', '').Split('::'))[-1] }
)
$plan = Get-Content -Raw docs/superpowers/plans/2026-07-20-extractum-llm-extraction.md
$appendix = $plan.Substring($plan.LastIndexOf('## Appendix A: Frozen 51-Test Ownership Map'))
$expected = @(
  [regex]::Matches($appendix, '(?m)^- `([^`]+)`$') |
    ForEach-Object { $_.Groups[1].Value }
)
if ($actual.Count -ne 51) { throw "Expected 51 executable LLM tests, found $($actual.Count)" }
if ($expected.Count -ne 51) { throw "Expected 51 plan names, found $($expected.Count)" }
$missing = @($expected | Where-Object { $_ -notin $actual })
$extra = @($actual | Where-Object { $_ -notin $expected })
$duplicates = @($actual | Group-Object | Where-Object Count -ne 1)
if ($missing.Count -or $extra.Count -or $duplicates.Count) {
  throw "Inventory mismatch: missing=$($missing -join ','); extra=$($extra -join ','); duplicate=$($duplicates.Name -join ',')"
}
```

Expected: empty status; 51 executable names; exact set equality with Appendix A; no duplicate. A baseline mismatch stops Phase 5 before source changes.

- [ ] **Step 2: Record the pre-seam consumer, literal, field-read, dependency, and source-contract inventory.**

```powershell
rg -n "crate::llm::|llm::\{" src-tauri/src --glob '*.rs'
if ($LASTEXITCODE -ne 0) { throw 'LLM consumer inventory failed or found no expected consumers' }
rg -n "ResolvedLlmProfile\s*\{|\.api_key\b|\.profile_id\b|\.provider\b|\.default_model\b|\.base_url\b" src-tauri/src/llm src-tauri/src/analysis src-tauri/src/prompt_packs --glob '*.rs'
if ($LASTEXITCODE -ne 0) { throw 'Resolved-profile inventory failed or found no expected sites' }
rg -n "^(reqwest|secrecy)\s*=|^(reqwest|secrecy)\.workspace\s*=" src-tauri/Cargo.toml src-tauri/crates --glob 'Cargo.toml'
if ($LASTEXITCODE -ne 0) { throw 'Dependency-root inventory failed' }
rg -n "src-tauri/src/llm|workspace\.dependencies|crates/extractum-gemini-browser" src/lib docs/value-registry.md --glob '*.ts' --glob '*.md'
if ($LASTEXITCODE -ne 0) { throw 'Source-contract inventory failed' }
rg -n "extractum_core|crate::error|AppError|AppResult|media_metadata" src-tauri/src/llm --glob '*.rs'
if ($LASTEXITCODE -ne 0) { throw 'Core/error inventory failed' }
```

Record in the task notes:

- the two external literals in `analysis/report/tests/harness.rs` and `prompt_packs/completion_transport.rs`;
- all external metadata reads enumerated in Task 2;
- no secret read outside `src-tauri/src/llm`;
- `reqwest` and `secrecy` each declared once in app dependencies and absent from workspace roots;
- `types.rs` does not use any `extractum-core` API; the portable error users need only `extractum_core::error`.

- [ ] **Step 3: Add exact six-event serialization characterization first and prove RED.**

Add `cancelled_stream_event`, `failed_stream_event`, `LlmUsage`, and
`StreamEvent` to the existing grouped `use super` list in the test module, and change the error import to
`use crate::error::{AppError, AppErrorKind};`. Then add this test without
changing `StreamEvent`:

```rust
#[test]
fn llm_stream_events_serialize_exact_lifecycle_contract() {
    let base = || {
        (
            "request-1".to_string(),
            "gemini".to_string(),
            "gemini-2.5-flash".to_string(),
        )
    };
    let (request_id, provider, model) = base();
    let queued = StreamEvent::new(request_id, "queued", provider, model)
        .queue_position(2)
        .build();
    let (request_id, provider, model) = base();
    let started = StreamEvent::new(request_id, "started", provider, model).build();
    let (request_id, provider, model) = base();
    let delta = StreamEvent::new(request_id, "delta", provider, model)
        .delta("hello".to_string())
        .build();
    let (request_id, provider, model) = base();
    let completed = StreamEvent::new(request_id, "completed", provider, model)
        .text("hello".to_string())
        .usage(Some(LlmUsage {
            input_tokens: Some(3),
            output_tokens: Some(2),
            total_tokens: Some(5),
        }))
        .build();
    let failure = AppError::network("LLM request failed: transport");
    let (request_id, provider, model) = base();
    let failed = failed_stream_event(request_id, provider, model, &failure);
    let (request_id, provider, model) = base();
    let cancelled = cancelled_stream_event(request_id, provider, model);

    assert_eq!(serde_json::to_string(&queued).unwrap(), r#"{"request_id":"request-1","kind":"queued","queue_position":2,"delta":null,"text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":null}"#);
    assert_eq!(serde_json::to_string(&started).unwrap(), r#"{"request_id":"request-1","kind":"started","queue_position":null,"delta":null,"text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":null}"#);
    assert_eq!(serde_json::to_string(&delta).unwrap(), r#"{"request_id":"request-1","kind":"delta","queue_position":null,"delta":"hello","text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":null}"#);
    assert_eq!(serde_json::to_string(&completed).unwrap(), r#"{"request_id":"request-1","kind":"completed","queue_position":null,"delta":null,"text":"hello","provider":"gemini","model":"gemini-2.5-flash","usage":{"input_tokens":3,"output_tokens":2,"total_tokens":5},"error":null}"#);
    assert_eq!(serde_json::to_string(&failed).unwrap(), r#"{"request_id":"request-1","kind":"failed","queue_position":null,"delta":null,"text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":"LLM request failed: transport"}"#);
    assert_eq!(serde_json::to_string(&cancelled).unwrap(), r#"{"request_id":"request-1","kind":"cancelled","queue_position":null,"delta":null,"text":null,"provider":"gemini","model":"gemini-2.5-flash","usage":null,"error":"Request cancelled."}"#);
}
```

```powershell
$redOutput = @(cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::llm_stream_events_serialize_exact_lifecycle_contract -- --exact 2>&1)
$redExit = $LASTEXITCODE
$redOutput | Out-Host
if ($redExit -eq 0) { throw 'Expected unresolved stream-event helper RED, but test passed' }
if ($redOutput -match 'running 0 tests') { throw 'RED selected 0 tests' }
if ($redOutput -notmatch 'failed_stream_event|cancelled_stream_event') { throw 'RED failed for an unexpected reason' }
```

Expected RED: unresolved `failed_stream_event` and
`cancelled_stream_event`. A syntax error or `0 tests` is not the intended RED.

- [ ] **Step 4: Add the distinct error-shape test, then implement the production event helpers.**

```rust
#[test]
fn llm_command_errors_and_failed_events_keep_distinct_json_shapes() {
    let error = AppError::network("LLM request failed: transport");
    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        r#"{"kind":"network","message":"LLM request failed: transport"}"#,
    );

    let failed = failed_stream_event(
        "request-1".to_string(),
        "gemini".to_string(),
        "gemini-2.5-flash".to_string(),
        &error,
    );
    assert_eq!(
        serde_json::to_value(failed).unwrap()["error"],
        serde_json::json!("LLM request failed: transport"),
    );
}
```

This pins the outgoing message as well as the error classification. Do not replace it with a predicate-only assertion.

Implement the two app-private helpers beside `StreamEvent`:

```rust
fn failed_stream_event(
    request_id: String,
    provider: String,
    model: String,
    error: &AppError,
) -> LlmStreamEvent {
    StreamEvent::new(request_id, "failed", provider, model)
        .error(error.to_string())
        .build()
}

fn cancelled_stream_event(
    request_id: String,
    provider: String,
    model: String,
) -> LlmStreamEvent {
    StreamEvent::new(request_id, "cancelled", provider, model)
        .error("Request cancelled.".to_string())
        .build()
}
```

Replace the two terminal arms in the production `ask_llm_stream` task with
calls to these helpers. The failed arm passes `&error`; the cancelled arm has
no error input. Do not change the surrounding match, emission, scheduler, or
spawn structure. The tests now exercise the same builders as production.

- [ ] **Step 5: Extend the existing frontend wrapper characterization for the two unasserted profile commands.**

Import `deleteLlmProfile` and `setActiveLlmProfile` in `src/lib/api/llm.test.ts`, then add:

```typescript
it("wraps active-profile and profile-deletion commands", async () => {
  invokeMock.mockResolvedValue({ active_profile: "default", profiles: [] });

  await setActiveLlmProfile("work");
  expect(invokeMock).toHaveBeenLastCalledWith("set_active_llm_profile", {
    profileId: "work",
  });

  await deleteLlmProfile("work");
  expect(invokeMock).toHaveBeenLastCalledWith("delete_llm_profile", {
    profileId: "work",
  });
});
```

The source-boundary contract in Task 4 freezes all nine Rust signatures, including the diagnostics-only snapshots command.

- [ ] **Step 6: Run the exact new characterization tests and the existing frontend wrapper test.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::llm_stream_events_serialize_exact_lifecycle_contract -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Lifecycle serialization characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Error-shape characterization failed' }
npm.cmd run test -- src/lib/api/llm.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Focused LLM frontend wrapper test failed' }
```

Expected: one Rust test in each Cargo run and the focused Vitest file passes.

- [ ] **Step 7: Run the unchanged baseline and commit the characterization checkpoint.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Task 1 app package checkpoint failed' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 1 diff check failed' }
$taskStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 1 status inspection failed' }
git add src-tauri/src/llm/mod.rs src/lib/api/llm.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Task 1 staging failed' }
git diff --cached --stat
if ($LASTEXITCODE -ne 0) { throw 'Task 1 staged-stat inspection failed' }
git commit -m "test: characterize LLM IPC and errors"
if ($LASTEXITCODE -ne 0) { throw 'Task 1 commit failed' }
$finalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 1 final status failed' }
if ($finalStatus.Count -ne 0) { throw "Task 1 commit left a dirty worktree:`n$($finalStatus -join "`n")" }
```

Expected: app package passes; the 51 frozen names still exist exactly once, plus two new Rust characterization tests.

---
### Task 2: Introduce the Safe Profile and Model-Listing Seams In-App

**Files:**

- Modify: `src-tauri/src/llm/types.rs`
- Create: `src-tauri/src/llm/provider.rs`
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/llm/profiles.rs`
- Modify: `src-tauri/src/llm/gemini.rs`
- Modify: `src-tauri/src/llm/openai_compat.rs`
- Modify: `src-tauri/src/llm/runner.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/report/phases.rs`
- Modify: `src-tauri/src/analysis/report/tests/harness.rs`
- Modify: `src-tauri/src/analysis/report/tests/scope.rs`
- Modify: `src-tauri/src/prompt_packs/completion_transport.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`

**Interfaces:**

- Consumes: current secret-bearing struct fields, provider/model policy in `mod.rs`, and the current saved/configured model-listing precedence.
- Produces: the exact safe constructors/getters, app-owned profile material adapter, portable provider module, and getter-based consumers while all code still compiles in `extractum`.

- [ ] **Step 1: Add the safe-profile test first and prove RED.**

Append this test module to `src-tauri/src/llm/types.rs` before defining `LlmProviderAccess` or the constructors:

```rust
#[cfg(test)]
mod tests {
    use secrecy::{ExposeSecret, SecretString};

    use super::{LlmProviderAccess, ResolvedLlmProfile};
    use crate::llm::ProviderKind;

    #[test]
    fn resolved_profile_construction_preserves_execution_access_and_public_metadata() {
        let profile = ResolvedLlmProfile::new(
            "research".to_string(),
            "gemini-2.5-flash".to_string(),
            LlmProviderAccess::new(
                ProviderKind::Gemini,
                SecretString::new("secret-key".to_string()),
                String::new(),
            ),
        );

        assert_eq!(profile.profile_id(), "research");
        assert_eq!(profile.provider(), ProviderKind::Gemini);
        assert_eq!(profile.default_model(), "gemini-2.5-flash");
        assert_eq!(profile.base_url(), "");
        assert_eq!(
            profile.provider_access().api_key().expose_secret(),
            "secret-key",
        );
    }
}
```

```powershell
$redOutput = @(cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::types::tests::resolved_profile_construction_preserves_execution_access_and_public_metadata -- --exact 2>&1)
$redExit = $LASTEXITCODE
$redOutput | Out-Host
if ($redExit -eq 0) { throw 'Expected safe-profile constructor/getter RED, but test passed' }
if ($redOutput -match 'running 0 tests') { throw 'Safe-profile RED selected 0 tests' }
if ($redOutput -notmatch 'LlmProviderAccess|ResolvedLlmProfile') { throw 'Safe-profile RED failed for an unexpected reason' }
```

Expected RED: unresolved `LlmProviderAccess`/constructors/getters. If Cargo reports `0 tests`, fix the test path rather than continuing.

- [ ] **Step 2: Implement the exact secret-safe types from `Exact Cross-Crate Interfaces`.**

In `types.rs`:

- add `LlmProviderAccess` with private `provider`, `api_key`, and `base_url`;
- replace the five-field `ResolvedLlmProfile` with `profile_id`, `default_model`, and `provider_access`;
- implement the two public constructors, four public non-secret profile getters, and only the `pub(super)` internal accessors listed above;
- make `LlmCompletion` and its fields `pub` now, because current consumers already read them through the private facade;
- derive only `Clone` for both secret-bearing types.

Do not implement `Debug`, `Serialize`, `Deserialize`, `AsRef<SecretString>`, `ExposeSecret`, or a secret-returning method on the public surface.

- [ ] **Step 3: Migrate all resolved-profile constructions before changing ownership.**

Use this construction form in the two external fixtures and the internal provider/runner/profile sites:

```rust
ResolvedLlmProfile::new(
    profile_id,
    default_model,
    LlmProviderAccess::new(provider, api_key, base_url),
)
```

Required literal sites are:

- `src-tauri/src/llm/profiles.rs::resolve_profile_from_pool`;
- two fixtures in `src-tauri/src/llm/runner.rs`;
- the streaming retry fixture in `src-tauri/src/llm/openai_compat.rs`;
- `src-tauri/src/analysis/report/tests/harness.rs::sample_resolved_profile`;
- `src-tauri/src/prompt_packs/completion_transport.rs::api_model_context_retains_profile_and_override`.

Then prove no owner-bypassing literal remains:

```powershell
$candidates = @(rg -n "ResolvedLlmProfile\s*\{" src-tauri/src --glob '*.rs')
if ($LASTEXITCODE -notin 0, 1) { throw 'ResolvedLlmProfile scan failed' }
$literals = @($candidates | Where-Object {
  $_ -notmatch '\b(?:struct|impl)\s+ResolvedLlmProfile\b' -and
  $_ -notmatch '->\s*ResolvedLlmProfile\s*\{'
})
if ($literals.Count -ne 0) { throw "ResolvedLlmProfile literal remains:`n$($literals -join "`n")" }
```

- [ ] **Step 4: Migrate every external metadata read to the four getters.**

Use only these mechanical replacements:

```rust
profile.profile_id.clone()       // -> profile.profile_id().to_string()
&profile.profile_id              // -> profile.profile_id()
profile.provider.as_str()        // -> profile.provider().as_str()
profile.default_model.as_str()   // -> profile.default_model()
profile.base_url.clone()         // -> profile.base_url().to_string()
```

Update all reads in:

- `src-tauri/src/llm/{mod.rs,gemini.rs,openai_compat.rs,runner.rs}`;
- `src-tauri/src/analysis/{chat.rs,report.rs}`;
- `src-tauri/src/analysis/report/{phases.rs,tests/scope.rs}`;
- `src-tauri/src/prompt_packs/completion_transport.rs`.

Provider implementations obtain `let access = profile.provider_access();` and call only the internal accessors. They must preserve every existing auth/error message.

- [ ] **Step 5: Remove secret-field assertions from the three app-owned profile tests without weakening them.**

In `active_profile_resolution_loads_key_from_secret_store`, assert the four
allowed metadata getters and call the new private
`resolve_profile_material_from_pool`; assert its `api_key` is `"alt-key"`.
This preserves proof that resolution, not merely storage, loaded the key.

In `empty_save_preserves_existing_secret` and
`clear_profile_api_key_deletes_secret`, read
`llm_profile_api_key_secret(profile_id)` directly through the existing
app-owned `SecretStoreState` and assert `Some("initial-key")` or `None` as the
current behavior requires.

Do not add a public or facade-level API-key getter merely to preserve the old assertions.

- [ ] **Step 6: Move provider/model policy from `mod.rs` into new app-owned `provider.rs`.**

Move, do not copy:

- `ProviderKind` plus `parse`, `as_str`, and `display_name`;
- `normalize_base_url` and the default provider/base-URL constants it needs;
- both model-limit match helpers and their two baseline tests;
- the 30-second model-listing timeout policy and five-second limit-lookup policy;
- Gemini/OpenAI-compatible provider dispatch;
- provider parse/base-URL tests.

Keep `DEFAULT_PROFILE_ID` and `DEFAULT_MODEL` app-side. Remove the app-only
`DEFAULT_PROVIDER` constant and change the profile fallback to
`ProviderKind::Gemini.as_str().to_string()` so the provider string has one
owner without exporting another constant.

Implement the three public functions and one private helper from `Exact Cross-Crate Interfaces`. Keep provider-specific configs/functions internal. In the app facade temporarily re-export the prepared API with current consumer aliases:

```rust
pub(crate) use provider::{
    normalize_base_url, resolve_model_input_token_limit as resolve_model_input_token_limit_for_backend,
    resolve_model_output_token_limit as resolve_model_output_token_limit_for_backend,
    ProviderKind,
};
use provider::list_provider_models;
```

Update the three model-limit consumer call sites in `analysis/report.rs`, `prompt_packs/runtime.rs`, and `prompt_packs/completion_transport.rs` only if necessary for the aliases above; their facade import names must remain unchanged at the end of the task.

- [ ] **Step 7: Split profile resolution into app-owned material and safe public values.**

Implement `ResolvedProfileMaterial`, `resolve_profile_material_from_pool`, and `resolve_provider_access_from_pool` exactly as declared above. `resolve_profile_from_pool` becomes a thin conversion:

```rust
let material = resolve_profile_material_from_pool(
    pool,
    secret_store,
    requested_profile_id,
).await?;
Ok(ResolvedLlmProfile::new(
    material.profile_id,
    material.default_model,
    LlmProviderAccess::new(material.provider, material.api_key, material.base_url),
))
```

Refactor `list_llm_provider_models` to build one `LlmProviderAccess` and call `list_provider_models(&access)`. Preserve these branches exactly:

1. both configured values present: no DB pool lookup;
2. one or both absent: resolve the saved profile once;
3. key selection is independent: configured key, else matching saved key, else empty;
4. URL selection is independent: configured normalized URL, else matching saved URL, else normalized provider default;
5. a mismatching saved provider contributes neither field but never discards a configured key or URL.

- [ ] **Step 8: Add a black-box model-listing seam test and prove GREEN without a secret getter.**

Add `provider_access_resolution_uses_saved_key_with_configured_base_url` to `profiles.rs`. Use a one-request `TcpListener` bound to `127.0.0.1:0`, save an `openai_compatible` profile with key `saved-key` and a same-origin `/old` URL, pass the same origin with `/v1` as configured URL, resolve access, and call `list_provider_models(&access)`.

Assert the request line starts with `GET /v1/models `; lowercase a copy of the
header text and assert it contains `authorization: bearer saved-key`. Respond
with `HTTP/1.1 200 OK`, lowercase `content-type: application/json`, the exact
`content-length`, `connection: close`, and body `{"data":[]}`. Assert the
returned model list is empty and the server task completes within two seconds.
This proves configured-URL/saved-key precedence through actual request
behavior, without reading the secret.

Add `provider_access_resolution_uses_configured_key_with_saved_base_url`
against the same server pattern: save `saved-key` with the server `/v1` URL,
pass `configured-key` and no configured URL, then assert the request uses the
saved `/v1/models` URL but header `authorization: bearer configured-key`.
Together the two tests pin independent precedence in both partial-input
directions.

Extend the test-module imports exactly:

```rust
use super::{
    clear_profile_api_key, credential_scope, delete_profile_from_pool,
    load_profiles_state_from_pool, resolve_profile_from_pool,
    resolve_provider_access_from_pool, save_profile_to_pool,
    set_active_profile_in_pool, validate_profile_id,
};
use crate::error::AppErrorKind;
use crate::llm::{list_provider_models, ProviderKind};
use crate::secret_store::tests::InMemorySecretStore;
use crate::secret_store::{llm_profile_api_key_secret, SecretStoreState};
use secrecy::{ExposeSecret, SecretString};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::{timeout, Duration},
};
```

Add this helper beside `memory_pool` and `memory_secret_store`:

```rust
async fn start_model_list_server(
    expected_path: &'static str,
    expected_bearer: &'static str,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind model-list server");
    let origin = format!(
        "http://{}",
        listener.local_addr().expect("model-list server address")
    );

    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept model-list request");
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            let read = socket.read(&mut chunk).await.expect("read model-list request");
            assert!(read > 0, "model-list request ended before headers");
            request.extend_from_slice(&chunk[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let request = String::from_utf8(request).expect("model-list request is UTF-8");
        assert!(
            request.starts_with(format!("GET {expected_path} ").as_str()),
            "unexpected model-list request path"
        );
        assert!(
            request.to_ascii_lowercase().contains(
                format!("authorization: bearer {}", expected_bearer.to_ascii_lowercase()).as_str()
            ),
            "model-list request omitted the expected bearer credential"
        );

        let body = r#"{"data":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body,
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write model-list response");
    });

    (origin, server)
}
```

Add the two tests exactly:

```rust
#[tokio::test]
async fn provider_access_resolution_uses_saved_key_with_configured_base_url() {
    let pool = memory_pool().await;
    let (_store, secret_store) = memory_secret_store();
    let (origin, server) = start_model_list_server("/v1/models", "saved-key").await;

    save_profile_to_pool(
        &pool,
        &secret_store,
        "default",
        "openai_compatible",
        "saved-model",
        Some("saved-key"),
        &format!("{origin}/old"),
        true,
    )
    .await
    .expect("save OpenAI-compatible profile");

    let access = resolve_provider_access_from_pool(
        &pool,
        &secret_store,
        ProviderKind::OpenAiCompatible,
        Some("default"),
        None,
        Some(format!("{origin}/v1")),
    )
    .await
    .expect("resolve saved key with configured base URL");

    let models = timeout(Duration::from_secs(2), list_provider_models(&access))
        .await
        .expect("model listing timed out")
        .expect("list provider models");
    assert!(models.is_empty());
    timeout(Duration::from_secs(2), server)
        .await
        .expect("model-list server timed out")
        .expect("model-list server failed");
}

#[tokio::test]
async fn provider_access_resolution_uses_configured_key_with_saved_base_url() {
    let pool = memory_pool().await;
    let (_store, secret_store) = memory_secret_store();
    let (origin, server) = start_model_list_server("/v1/models", "configured-key").await;

    save_profile_to_pool(
        &pool,
        &secret_store,
        "default",
        "openai_compatible",
        "saved-model",
        Some("saved-key"),
        &format!("{origin}/v1"),
        true,
    )
    .await
    .expect("save OpenAI-compatible profile");

    let access = resolve_provider_access_from_pool(
        &pool,
        &secret_store,
        ProviderKind::OpenAiCompatible,
        Some("default"),
        Some(SecretString::new("configured-key".to_string())),
        None,
    )
    .await
    .expect("resolve configured key with saved base URL");

    let models = timeout(Duration::from_secs(2), list_provider_models(&access))
        .await
        .expect("model listing timed out")
        .expect("list provider models");
    assert!(models.is_empty());
    timeout(Duration::from_secs(2), server)
        .await
        .expect("model-list server timed out")
        .expect("model-list server failed");
}
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Saved-key/configured-URL seam test failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Configured-key/saved-URL seam test failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::types::tests::resolved_profile_construction_preserves_execution_access_and_public_metadata -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Safe-profile seam test failed' }
```

Expected: one non-empty GREEN test in each command.

- [ ] **Step 9: Prove the API shape and run the app checkpoint.**

```powershell
$candidates = @(rg -n "ResolvedLlmProfile\s*\{" src-tauri/src --glob '*.rs')
if ($LASTEXITCODE -notin 0, 1) { throw 'ResolvedLlmProfile scan failed' }
$literals = @($candidates | Where-Object {
  $_ -notmatch '\b(?:struct|impl)\s+ResolvedLlmProfile\b' -and
  $_ -notmatch '->\s*ResolvedLlmProfile\s*\{'
})
if ($literals.Count -ne 0) { throw "Literal remains:`n$($literals -join "`n")" }
$resolvedProfileConsumers = @(
  'src-tauri/src/analysis/chat.rs',
  'src-tauri/src/analysis/report.rs',
  'src-tauri/src/analysis/report/phases.rs',
  'src-tauri/src/analysis/report/tests/harness.rs',
  'src-tauri/src/analysis/report/tests/scope.rs',
  'src-tauri/src/prompt_packs/completion_transport.rs'
)
$externalSecrets = rg -n "\.api_key\b|api_key\(\)" $resolvedProfileConsumers
if ($LASTEXITCODE -eq 0) { throw "External secret access remains:`n$externalSecrets" }
if ($LASTEXITCODE -ne 1) { throw 'External secret scan failed' }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Task 2 app check failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Task 2 app test failed' }
```

Expected: no literals or external secret access; app check/test pass.

- [ ] **Step 10: Commit the safe seam separately from the physical extraction.**

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 2 diff check failed' }
$taskStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 2 status inspection failed' }
git add src-tauri/src/llm src-tauri/src/analysis/chat.rs src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/phases.rs src-tauri/src/analysis/report/tests/harness.rs src-tauri/src/analysis/report/tests/scope.rs src-tauri/src/prompt_packs/completion_transport.rs src-tauri/src/prompt_packs/runtime.rs
if ($LASTEXITCODE -ne 0) { throw 'Task 2 staging failed' }
git diff --cached --stat
if ($LASTEXITCODE -ne 0) { throw 'Task 2 staged-stat inspection failed' }
git commit -m "refactor: prepare portable LLM boundary"
if ($LASTEXITCODE -ne 0) { throw 'Task 2 commit failed' }
$finalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 2 final status failed' }
if ($finalStatus.Count -ne 0) { throw "Task 2 commit left a dirty worktree:`n$($finalStatus -join "`n")" }
```

Expected: all production code still belongs to `extractum`; there is no workspace member/path edge yet.

---

### Task 3: Prepare the Seven Portable Files for a Mechanical Move

**Files:**

- Create: `src-tauri/src/llm/app_types.rs`
- Modify: `src-tauri/src/llm/types.rs`
- Modify: `src-tauri/src/llm/provider.rs`
- Modify: `src-tauri/src/llm/gemini.rs`
- Modify: `src-tauri/src/llm/openai_compat.rs`
- Modify: `src-tauri/src/llm/runner.rs`
- Modify: `src-tauri/src/llm/scheduler.rs`
- Modify: `src-tauri/src/llm/streaming.rs`
- Modify: `src-tauri/src/llm/mod.rs`

**Interfaces:**

- Consumes: the safe app-owned seam from Task 2.
- Produces: seven internally portable source files with final imports, tests, and visibility; `extractum` remains the only package owner.

- [ ] **Step 1: Split mixed DTO ownership without changing serialized fields.**

Move exactly these unchanged definitions from `types.rs` to new `app_types.rs`:

- `LlmStreamEvent`;
- `LlmProfile`;
- `LlmProfilesState`.

Keep their derive lists, field order, names, types, and visibility byte-equivalent. `types.rs` retains `LlmMessage`, `LlmChatRequest`, `LlmUsage`, `LlmProviderModel`, `LlmProviderAccess`, `ResolvedLlmProfile`, `LlmCompletion`, and its new safe-type test.

Add `use super::LlmUsage;` in `app_types.rs`; that path resolves through the
same private facade both before and after the move.

In `mod.rs`, add `mod app_types;` and re-export app types from there. Do not leave aliases or duplicate definitions in portable `types.rs`.

- [ ] **Step 2: Point every portable error import directly at core.**

Change only the portable modules:

```rust
use extractum_core::error::{AppError, AppResult};
```

Required files are `provider.rs`, `gemini.rs`, `openai_compat.rs`, `runner.rs`, `scheduler.rs`, and `streaming.rs`. Replace test imports of `crate::error::AppErrorKind` with `extractum_core::error::AppErrorKind` and scheduler test imports with `extractum_core::error::{AppError, AppErrorKind}`.

Keep `profiles.rs` and app `mod.rs` on `crate::error`; they are application adapters.

- [ ] **Step 3: Make portable test imports ownership-relative.**

Inside portable nested test modules, replace app-root paths such as:

```rust
use crate::llm::{LlmChatRequest, LlmMessage, ProviderKind, ResolvedLlmProfile};
```

with the exact ownership-relative imports:

```rust
// gemini.rs tests
use super::super::{LlmChatRequest, LlmMessage};
// openai_compat.rs tests
use super::super::{LlmChatRequest, LlmMessage, ProviderKind, ResolvedLlmProfile};
// runner.rs tests
use super::super::{LlmChatRequest, ProviderKind, ResolvedLlmProfile};
```

In `types.rs` tests use `super::super::ProviderKind`. No portable test may import `crate::llm` or `crate::error` after this step.

- [ ] **Step 4: Move scheduler diagnostic-key ownership and its frozen test.**

Move `llm_request_kind_diagnostic_key`, `llm_request_state_diagnostic_key`, and `llm_request_diagnostic_keys_are_stable_snake_case` from app `mod.rs` into `scheduler.rs`. Make both functions `pub`. The app diagnostics test remains in `mod.rs` and consumes the functions through the temporary facade re-export.

Do not alter the five request-kind strings or two snapshot-state strings.

- [ ] **Step 5: Apply only the approved visibility widenings.**

Make public exactly the items listed under `Exact Cross-Crate Interfaces`:

- provider policy and the three provider operations;
- four runner operations;
- `LlmCompletion` and its fields;
- `LlmRequestMetadata` and its existing construction fields;
- `LlmRequestControl` and `LlmRequestError`;
- `LlmSchedulerState::request_snapshots` and `active_owner_run_ids`;
- the two diagnostic-key functions.

Change `LlmRequestControl::is_cancelled` from `pub fn` to private `fn`. Keep
`LlmRequestControl::new`, `cancel`, and `is_cancelled`, scheduler-key/entry/inner
types, provider configs/functions, SSE functions, HTTP mapping structs, and
retry helpers private at their current narrowest viable visibility. The only
public method in `impl LlmRequestControl` is `run_cancellable`.

- [ ] **Step 6: Freeze the exact pre-manifest dependency/feature inventory.**

```powershell
rg -n "^(use|pub use) (reqwest|secrecy|serde|serde_json|tokio|tokio_util|extractum_core)|(?:reqwest|secrecy|serde_json|tokio|tokio_util)::" src-tauri/src/llm/types.rs src-tauri/src/llm/provider.rs src-tauri/src/llm/gemini.rs src-tauri/src/llm/openai_compat.rs src-tauri/src/llm/runner.rs src-tauri/src/llm/scheduler.rs src-tauri/src/llm/streaming.rs
if ($LASTEXITCODE -ne 0) { throw 'Portable dependency inventory failed' }
rg -n "tokio::(io|net|test|time|sync)|#\[tokio::test" src-tauri/src/llm --glob '*.rs'
if ($LASTEXITCODE -ne 0) { throw 'Tokio feature inventory failed' }
rg -n "url::|reqwest::Url" src-tauri/src/llm --glob '*.rs'
if ($LASTEXITCODE -notin 0, 1) { throw 'URL-root inventory failed' }
```

Confirm and record:

- production roots exactly `extractum-core`, `reqwest`, `secrecy`, `serde`, `serde_json`, `tokio`, `tokio-util`;
- production Tokio features `macros`, `sync`, `time`;
- test-only Tokio features `io-util`, `net`, `rt`, `test-util`;
- no direct `url` root;
- no `media_metadata` or other `extractum-core` module besides `error`.

If prepared code proves a different feature/root, stop and amend the approved design before manifest creation; do not improvise a dependency in Task 6.

- [ ] **Step 7: Prove portable files are app-independent before the move.**

```powershell
$portable = @(
  'src-tauri/src/llm/types.rs',
  'src-tauri/src/llm/provider.rs',
  'src-tauri/src/llm/gemini.rs',
  'src-tauri/src/llm/openai_compat.rs',
  'src-tauri/src/llm/runner.rs',
  'src-tauri/src/llm/scheduler.rs',
  'src-tauri/src/llm/streaming.rs'
)
$forbidden = rg -n "crate::(?:llm|db|secret_store|diagnostics|analysis|prompt_packs|telegram|gemini_browser)|tauri|sqlx|keyring|apalis|grammers|windows_sys" $portable
if ($LASTEXITCODE -eq 0) { throw "Portable dependency leak:`n$forbidden" }
if ($LASTEXITCODE -ne 1) { throw 'Portable dependency scan failed' }
```

Also inspect `git diff --find-renames -- src-tauri/src/llm` and prove each moved definition/test has one owner, not a copied compatibility implementation.

- [ ] **Step 8: Re-run the exact frozen inventory and app checkpoint.**

```powershell
$listed = cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list
if ($LASTEXITCODE -ne 0) { throw 'Prepared test listing failed' }
$names = @(
  $listed |
    Select-String '^llm::.*: test$' |
    ForEach-Object { (($_.Line -replace ': test$', '').Split('::'))[-1] }
)
$plan = Get-Content -Raw docs/superpowers/plans/2026-07-20-extractum-llm-extraction.md
$appendix = $plan.Substring($plan.LastIndexOf('## Appendix A: Frozen 51-Test Ownership Map'))
$expected = @([regex]::Matches($appendix, '(?m)^- `([^`]+)`$') | ForEach-Object { $_.Groups[1].Value })
$missing = @($expected | Where-Object { $_ -notin $names })
$duplicates = @($names | Where-Object { $_ -in $expected } | Group-Object | Where-Object Count -ne 1)
if ($names.Count -ne 56 -or $missing.Count -or $duplicates.Count) {
  throw "Prepared inventory mismatch: total=$($names.Count), missing=$($missing -join ','), duplicate=$($duplicates.Name -join ',')"
}
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::provider::tests::provider_parse_returns_typed_validation_error -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Prepared provider-parse characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::provider::tests::normalize_base_url_allows_https_and_loopback_http_only -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Prepared base-URL characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::runner::tests::validate_request_returns_typed_validation_error -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Prepared request-validation characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::openai_compat::tests::openai_compat_stream_retries_transient_http_before_streaming -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Prepared retry characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::scheduler::tests::failed_requests_preserve_typed_error_kind -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Prepared scheduler characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Prepared profile-persistence characterization failed' }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Prepared app check failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Prepared app test failed' }
$history = @(git log --format='%H%x09%s')
if ($LASTEXITCODE -ne 0) { throw 'Characterization-history inspection failed' }
$characterizationMatches = @($history | Where-Object { ($_ -split "`t", 2)[1] -eq 'test: characterize LLM IPC and errors' })
if ($characterizationMatches.Count -ne 1) { throw "Expected one characterization commit, found $($characterizationMatches.Count)" }
$characterizationCommit = ($characterizationMatches[0] -split "`t", 2)[0]
git diff "$characterizationCommit^" -- src-tauri/src/llm/mod.rs
if ($LASTEXITCODE -ne 0) { throw 'Lifecycle preparation diff failed' }
```

Expected: all 51 frozen names exactly once plus exactly the five named Rust tests added by Tasks 1-2; app check/test pass.

Before committing, resolve the unique `test: characterize LLM IPC and errors`
commit and inspect the `src-tauri/src/llm/mod.rs` diff from its parent through
the prepared tree. Record that Tasks 1-3 changed only event-helper calls,
metadata getters, provider-policy/diagnostic-key ownership, and imports. The
spawn boundary, pre-spawn validation/resolution, queue-before-start ordering,
delta callback, exactly-one-terminal match, ignored emit results, immediate
outer `Ok(())`, queued-cancellation behavior, and duplicate-registration
behavior must remain structurally unchanged. Use the existing scheduler
cancellation/registration tests plus the Task 1 byte fixtures as the
behavioral support; any other control-flow change stops the mechanical slice.

- [ ] **Step 9: Commit the final app-owned preparation checkpoint.**

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 3 diff check failed' }
$taskStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 3 status inspection failed' }
git add src-tauri/src/llm
if ($LASTEXITCODE -ne 0) { throw 'Task 3 staging failed' }
git diff --cached --stat
if ($LASTEXITCODE -ne 0) { throw 'Task 3 staged-stat inspection failed' }
git commit -m "refactor: make LLM engine portable"
if ($LASTEXITCODE -ne 0) { throw 'Task 3 commit failed' }
$finalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 3 final status failed' }
if ($finalStatus.Count -ne 0) { throw "Task 3 commit left a dirty worktree:`n$($finalStatus -join "`n")" }
```

Expected: clean commit; no crate directory, workspace member, path dependency, or lock package yet.

---

### Task 4: Add the Deliberately RED LLM Boundary Contract

**Files:**

- Create: `src/lib/llm-crate-boundary-contract.test.ts`
- Read: `src/lib/gemini-browser-crate-boundary-contract.test.ts`
- Read: `src/lib/rust-workspace-core-contract.test.ts`
- Read: `src-tauri/Cargo.toml`
- Read: `src-tauri/Cargo.lock`
- Read: `src-tauri/src/llm/{mod.rs,profiles.rs,app_types.rs}`
- Read: `src-tauri/src/lib.rs`

**Interfaces:**

- Consumes: prepared source layout, exact API/manifest contracts above, and Appendix A.
- Produces: one Vitest file that is RED solely because the approved crate/member/path/lock ownership does not exist yet.

- [ ] **Step 1: Create the contract using the existing section/lock/source helpers.**

Copy the generic `read`, recursive `rustSources`, `tomlSection`, `dependencyNames`, `lockPackage`, `lockDependencies`, and Rust test-name parser helpers from `gemini-browser-crate-boundary-contract.test.ts`; do not import helpers from a production module.

Define these exact paths and expected dependency sets:

```typescript
const crateDir = "src-tauri/crates/extractum-llm";
const appDir = "src-tauri/src/llm";
const expectedCrateDependencies = [
  "extractum-core",
  "reqwest",
  "secrecy",
  "serde",
  "serde_json",
  "tokio",
  "tokio-util",
].sort();
const forbiddenCrateDependencyNames = [
  "apalis",
  "apalis-sqlite",
  "extractum",
  "extractum-analysis",
  "extractum-gemini-browser",
  "extractum-prompt-packs",
  "extractum-telegram",
  "grammers-client",
  "grammers-mtsender",
  "grammers-session",
  "grammers-tl-types",
  "keyring",
  "sqlx",
  "tauri",
  "windows-sys",
];
```

Embed the exact crate-root string from `Exact Cross-Crate Interfaces`, and copy the 36/15 bare-name arrays verbatim from Appendix A.

- [ ] **Step 2: Assert exact workspace, manifest, lock, and feature ownership.**

The first contract test must assert:

- workspace members equal `['.', 'crates/extractum-core', 'crates/extractum-gemini-browser', 'crates/extractum-llm']` in that order;
- the app `[dependencies]` contains one `extractum-llm = { path = "crates/extractum-llm" }` edge and no second textual occurrence;
- crate production dependency names equal `expectedCrateDependencies`; dev dependency names equal `['tokio']`;
- none of `forbiddenCrateDependencyNames` appears in either crate dependency section;
- the workspace `reqwest` line is exactly the version/default-features/three-feature declaration in `Manifest Contract`;
- workspace `secrecy = "0.8"`; the app uses `{ workspace = true }`, the crate uses `.workspace = true`, and neither has a package-local version/features;
- crate production Tokio features are exactly `macros`, `sync`, `time`; dev features exactly `io-util`, `net`, `rt`, `test-util`;
- the `extractum-llm` lock package exists with the seven sorted dependencies above;
- the `extractum` lock package includes exactly one `extractum-llm` dependency.

- [ ] **Step 3: Assert the curated root, visibility, and secret boundary.**

The next contract test must assert:

- `src-tauri/crates/extractum-llm/src/lib.rs` equals the exact root text in this plan after CRLF normalization;
- it has no `pub mod`, glob export, `#[cfg(test)]`, or public test helper;
- `types.rs` contains exactly `#[derive(Clone)]` immediately above each secret-bearing struct;
- neither secret-bearing struct derives `Serialize`, `Deserialize`, or `Debug`;
- neither secret-bearing struct has a manual `impl Serialize`, `impl Deserialize`, `impl serde::Serialize`, or `impl serde::Deserialize`;
- no `pub api_key`, `pub fn api_key`, `pub(crate) fn api_key`, public `SecretString` return, or `ExposeSecret` re-export exists;
- public `LlmProviderAccess` methods are exactly `new`;
- public `ResolvedLlmProfile` methods are exactly `new`, `profile_id`, `provider`, `default_model`, and `base_url`;
- public `LlmRequestControl` methods are exactly `run_cancellable`;
- public `ProviderKind` inherent methods are exactly `parse` and `as_str`;
- public `LlmSchedulerState` inherent methods are exactly `new`, `run_request`, `cancel_request`, `cancel_run_requests`, `request_snapshots`, and `active_owner_run_ids`;
- the approved widened functions/types occur in the crate root and have a real app consumer or facade re-export;
- `ProviderKind::display_name`, `list_provider_models_without_timeout`, provider-specific request/list functions, SSE helpers, `LlmRequestControl::new`, and scheduler internals are not root exports.

- [ ] **Step 4: Assert forbidden imports and single physical ownership.**

Join all crate Rust sources and reject these source patterns:

```typescript
const forbiddenSourcePatterns = [
  /\btauri(?:_[a-z0-9_]+)?\b/,
  /\bsqlx\b/,
  /\bkeyring\b/,
  /\bapalis(?:_sqlite)?\b/,
  /\bgrammers(?:_[a-z_]+)?\b/,
  /\bwindows_sys\b/,
  /\b(?:Child|Command|Stdio|ProcessTreeGuard)\b/,
  /(?:std|tokio)::process/,
  /\bprocess_tree\b/,
  /\bextractum_process\b/,
  /crate::(?:db|secret_store|diagnostics|analysis|prompt_packs|telegram|gemini_browser)/,
  /extractum_(?:analysis|prompt_packs|telegram|gemini_browser)/,
];
```

Assert app paths `gemini.rs`, `openai_compat.rs`, `provider.rs`, `runner.rs`, `scheduler.rs`, `streaming.rs`, and `types.rs` are absent. Assert `app_types.rs`, `profiles.rs`, and `mod.rs` exist. Across all Rust sources reject `#[cfg(any())]`; across app sources reject copied provider structs, SSE helpers, scheduler inner types, and all seven moved module declarations.

The test-name parser must prove each frozen crate-owned name occurs exactly once in crate sources and zero times in app sources, and each frozen app-owned name occurs exactly once in app sources and zero times in crate sources. Also assert the union has length 51 and no duplicates. New test names are ignored by this frozen-set ownership assertion.

- [ ] **Step 5: Assert app-owned commands, event, profiles, diagnostics, and payload surface.**

Read app `mod.rs`, `profiles.rs`, `src/lib/types/llm.ts`, and `src/lib/api/llm.ts`. Assert:

- exactly nine `#[tauri::command]` functions with the names in the approved spec;
- those same nine names occur exactly once in the `use llm::{...}` import and exactly once in `tauri::generate_handler![...]` in `src-tauri/src/lib.rs`:

```typescript
const commandNames = [
  "get_llm_profiles",
  "get_llm_request_snapshots",
  "save_llm_profile",
  "clear_llm_profile_api_key",
  "delete_llm_profile",
  "set_active_llm_profile",
  "list_llm_provider_models",
  "ask_llm_stream",
  "cancel_llm_request",
] as const;
```

- command parameter names/types encode exactly the approved payload keys, with `Option` only on `profile_id`, `api_key`, `base_url`, `set_active`, and `model_override` where specified;
- return types remain `AppResult<Vec<LlmRequestSnapshot>>`, `AppResult<LlmProfilesState>`, `AppResult<Vec<LlmProviderModel>>`, or `AppResult<()>` as specified;
- `LLM_RESPONSE_EVENT` remains `"llm://response"` in Rust and TypeScript;
- TypeScript event union remains exactly `queued | started | delta | completed | failed | cancelled` and all optional values remain nullable fields;
- all four setting-key forms and calls to `llm_profile_api_key_secret` remain in `profiles.rs`; the helper definition remains app-owned in `src-tauri/src/secret_store.rs`; all are absent from the crate;
- `sqlx`, `SecretStoreState`, pool access, diagnostics aggregation structs/functions, and the `StreamEvent` builder remain app-owned;
- `profiles.rs` is not imported or textually copied by the crate.

Pin the frontend bridge to these exact interfaces and wrappers, including
camelCase names and optional-versus-null semantics:

```typescript
export interface SaveLlmProfileInput {
  profileId: LlmProfile["profile_id"];
  provider: LlmProfile["provider"];
  defaultModel: LlmProfile["default_model"];
  apiKey: string | null;
  baseUrl: LlmProfile["base_url"] | null;
  setActive: boolean;
}

export interface ListLlmProviderModelsInput {
  provider: string;
  profileId?: string | null;
  apiKey?: string | null;
  baseUrl?: string | null;
}

export interface AskLlmStreamInput {
  requestId: string;
  profileId: string | null;
  messages: LlmMessage[];
  modelOverride: string | null;
}

invoke<LlmProfilesState>("get_llm_profiles");
invoke<LlmProfilesState>("save_llm_profile", { ...input });
invoke<LlmProviderModel[]>("list_llm_provider_models", { ...input });
invoke<void>("ask_llm_stream", { ...input });
invoke<LlmProfilesState>("clear_llm_profile_api_key", { profileId });
invoke<LlmProfilesState>("delete_llm_profile", { profileId });
invoke<LlmProfilesState>("set_active_llm_profile", { profileId });
invoke<void>("cancel_llm_request", { requestId });
```

Also extract the production `ask_llm_stream` function body from app `mod.rs`
and pin its lifecycle structure: `tokio::spawn` occurs before the outer final
`Ok(())`; the spawned task calls `run_request`; its queue callback builds the
`queued` event; its execution callback emits `started` before calling
`run_cancellable(run_llm_stream_with_profile(...))`; the delta callback emits
`delta`; and the outer result match has exactly the completed, failed, and
cancelled terminal arms. Assert there is no `.await` on the spawn handle. This
is a source-structure contract for the existing lifecycle, not a replacement
runtime implementation.

Pin the behavioral constants and messages in their final owner files:

- `runner.rs`: `LLM_STREAM_TIMEOUT_SECS = 90` and the exact message `LLM request timed out after {LLM_STREAM_TIMEOUT_SECS} seconds` in both collect and stream paths;
- `provider.rs`: Gemini model listing `30`, OpenAI-compatible model listing `30`, and model-limit lookup `5` seconds;
- `gemini.rs`: `GEMINI_STREAM_MAX_ATTEMPTS = 3` and `GEMINI_RETRY_DELAY_MS = 600`;
- `openai_compat.rs`: `OPENAI_COMPAT_STREAM_MAX_ATTEMPTS = 3` and `OPENAI_COMPAT_RETRY_DELAY_MS = 600`.

These are exact source assertions in addition to the frozen behavior tests;
changing them requires a new approved behavior design, not a relaxed contract.

- [ ] **Step 6: Run the contract and prove the expected RED reason.**

```powershell
$redOutput = @(npm.cmd run test -- src/lib/llm-crate-boundary-contract.test.ts 2>&1)
$redExit = $LASTEXITCODE
$redOutput | Out-Host
if ($redExit -eq 0) { throw 'Expected missing LLM crate boundary RED, but test passed' }
if ($redOutput -match 'No test files found|0 tests') { throw 'LLM boundary RED did not execute the contract' }
if ($redOutput -notmatch 'extractum-llm') { throw 'LLM boundary contract failed for an unexpected reason' }
```

Expected RED: missing `src-tauri/crates/extractum-llm`, fourth workspace member, app path dependency, and lock package. The test must parse and execute; syntax/helper failures are not the intended RED.

- [ ] **Step 7: Commit the clean RED contract checkpoint.**

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 4 diff check failed' }
git add src/lib/llm-crate-boundary-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Task 4 staging failed' }
git diff --cached --stat
if ($LASTEXITCODE -ne 0) { throw 'Task 4 staged-stat inspection failed' }
git commit -m "test: define LLM crate boundary"
if ($LASTEXITCODE -ne 0) { throw 'Task 4 commit failed' }
$finalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 4 final status failed' }
if ($finalStatus.Count -ne 0) { throw "Task 4 commit left a dirty worktree:`n$($finalStatus -join "`n")" }
```

Expected: clean worktree. This branch intentionally has one RED source-boundary test until Task 6; Cargo remains green.

---

### Task 5: Capture the Minimal Advisory Baseline

**Files:**

- Temporarily modify and restore: `src-tauri/src/llm/streaming.rs`
- Record later in: `docs/superpowers/verification/2026-07-20-extractum-llm-extraction.md`

**Interfaces:**

- Consumes: clean prepared app-owned source and canonical Cargo target.
- Produces: three baseline raw values, median, source SHA-256, and restoration proof; no repository change.

- [ ] **Step 1: Run the complete baseline session in one PowerShell process.**

Run this block exactly from the repository root. It validates the clean LF
precondition, uses only `apply_patch` for marker edits, discards one warm-up,
records three samples in `a -> b`, `b -> a`, `a -> b` order, and restores the
source inside a real `finally` block:

```powershell
$ErrorActionPreference = 'Stop'
$probe = 'src-tauri/src/llm/streaming.rs'
$package = 'extractum'
$label = 'BASELINE'
$sourceCheckpointSubject = 'refactor: make LLM engine portable'
$sourceCheckpointPath = 'src-tauri/src/llm/streaming.rs'
$marker = $null
$samples = New-Object 'System.Collections.Generic.List[Int64]'
$measurementFailure = $null
$restorationProblems = New-Object 'System.Collections.Generic.List[string]'
$afterHash = $null
$statusAfter = @()
$codexCommand = Get-Command codex.exe -CommandType Application -ErrorAction Stop
$codexExe = $codexCommand.Source

function Invoke-ProbePatch {
  param([Parameter(Mandatory = $true)][string]$PatchText)
  & $codexExe --codex-run-as-apply-patch $PatchText
  if ($LASTEXITCODE -ne 0) { throw 'apply_patch failed for timing probe' }
}

function Add-ProbeMarker {
  param([Parameter(Mandatory = $true)][string]$Path)
  Invoke-ProbePatch @"
*** Begin Patch
*** Update File: $Path
@@
 use extractum_core::error::{AppError, AppResult};
+// extractum-llm timing probe a
*** End Patch
"@
}

function Set-ProbeMarker {
  param(
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter(Mandatory = $true)][string]$From,
    [Parameter(Mandatory = $true)][string]$To
  )
  Invoke-ProbePatch @"
*** Begin Patch
*** Update File: $Path
@@
-// extractum-llm timing probe $From
+// extractum-llm timing probe $To
*** End Patch
"@
}

function Remove-ProbeMarker {
  param(
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter(Mandatory = $true)][string]$Current
  )
  Invoke-ProbePatch @"
*** Begin Patch
*** Update File: $Path
@@
-// extractum-llm timing probe $Current
*** End Patch
"@
}

function Invoke-TimedCheck {
  param(
    [Parameter(Mandatory = $true)][string]$Package,
    [Parameter(Mandatory = $true)][string]$SampleLabel
  )
  $elapsed = Measure-Command {
    & cargo check --manifest-path src-tauri/Cargo.toml -p $Package --all-targets
    if ($LASTEXITCODE -ne 0) { throw "$SampleLabel Cargo check failed" }
  }
  [Int64][Math]::Round($elapsed.TotalMilliseconds)
}

$statusBefore = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Baseline clean-tree check failed' }
if ($statusBefore.Count -ne 0) { throw "Baseline requires a clean worktree:`n$($statusBefore -join "`n")" }
$history = @(git log --format='%H%x09%s')
if ($LASTEXITCODE -ne 0) { throw 'Baseline history inspection failed' }
$checkpointMatches = @($history | Where-Object { ($_ -split "`t", 2)[1] -eq $sourceCheckpointSubject })
if ($checkpointMatches.Count -ne 1) { throw "Expected one '$sourceCheckpointSubject' ancestor, found $($checkpointMatches.Count)" }
$sourceCheckpoint = ($checkpointMatches[0] -split "`t", 2)[0]
git merge-base --is-ancestor $sourceCheckpoint HEAD
if ($LASTEXITCODE -ne 0) { throw 'Prepared-source checkpoint is not an ancestor of HEAD' }
$eol = @(git ls-files --eol -- $probe)
if ($LASTEXITCODE -ne 0) { throw 'Baseline EOL inspection failed' }
if ($eol.Count -ne 1 -or $eol[0] -notmatch '\bw/lf\b') { throw "Baseline probe is not w/lf: $($eol -join ' | ')" }
if (Select-String -LiteralPath $probe -SimpleMatch 'extractum-llm timing probe' -Quiet) {
  throw 'Baseline probe marker already exists'
}
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $probe).Path)
if ([regex]::Matches($source, [regex]::Escape('use extractum_core::error::{AppError, AppResult};')).Count -ne 1) {
  throw 'Expected the exact core-error import once'
}
$beforeHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $probe).Hash
$currentBlob = @(git rev-parse "HEAD:$probe")
if ($LASTEXITCODE -ne 0 -or $currentBlob.Count -ne 1) { throw 'Baseline probe blob resolution failed' }
$referenceBlob = @(git rev-parse "${sourceCheckpoint}:$sourceCheckpointPath")
if ($LASTEXITCODE -ne 0 -or $referenceBlob.Count -ne 1) { throw 'Prepared probe blob resolution failed' }
if ($currentBlob[0] -ne $referenceBlob[0]) {
  throw "Measured probe is not byte-identical: $($referenceBlob[0]) -> $($currentBlob[0])"
}

try {
  Add-ProbeMarker -Path $probe
  $marker = 'a'

  & cargo check --manifest-path src-tauri/Cargo.toml -p $package --all-targets
  if ($LASTEXITCODE -ne 0) { throw 'Baseline warm-up failed' }

  Set-ProbeMarker -Path $probe -From 'a' -To 'b'
  $marker = 'b'
  $samples.Add((Invoke-TimedCheck -Package $package -SampleLabel 'Baseline sample 1')) | Out-Null

  Set-ProbeMarker -Path $probe -From 'b' -To 'a'
  $marker = 'a'
  $samples.Add((Invoke-TimedCheck -Package $package -SampleLabel 'Baseline sample 2')) | Out-Null

  Set-ProbeMarker -Path $probe -From 'a' -To 'b'
  $marker = 'b'
  $samples.Add((Invoke-TimedCheck -Package $package -SampleLabel 'Baseline sample 3')) | Out-Null
} catch {
  $measurementFailure = $_.Exception.Message
} finally {
  foreach ($candidateMarker in @('a', 'b')) {
    try {
      if (Select-String -LiteralPath $probe -SimpleMatch "extractum-llm timing probe $candidateMarker" -Quiet) {
        Remove-ProbeMarker -Path $probe -Current $candidateMarker
      }
    } catch {
      [void]$restorationProblems.Add($_.Exception.Message)
    }
  }
  try {
    $afterHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $probe).Hash
    $statusAfter = @(git status --short)
    if ($LASTEXITCODE -ne 0) { throw 'Baseline final clean-tree check failed' }
  } catch {
    [void]$restorationProblems.Add($_.Exception.Message)
  }
}

if ($restorationProblems.Count -ne 0) { throw "Baseline restoration infrastructure failure: $($restorationProblems -join '; ')" }
if ($afterHash -ne $beforeHash) { throw "Baseline probe restoration failed: $beforeHash -> $afterHash" }
if ($statusAfter.Count -ne 0) { throw "Baseline series left a dirty worktree:`n$($statusAfter -join "`n")" }
"BASELINE_SHA256=$afterHash"
"BASELINE_BLOB=$($currentBlob[0])"
"BASELINE_RAW_MS=$($samples -join ',')"
if ($measurementFailure -or $samples.Count -ne 3) {
  "BASELINE_RESULT=incomplete / no conclusion; $measurementFailure"
} else {
  $sorted = @($samples | Sort-Object)
  "BASELINE_MEDIAN_MS=$($sorted[1])"
}
```

- [ ] **Step 2: Record the output without retrying the series.**

Record `BASELINE_SHA256`, all observed raw values, and the median only when
three values exist. If the result is `incomplete / no conclusion`, retain the
observed values and continue the correctness slice only after the block has
proved byte restoration and a clean worktree. Do not rerun the series.

There is no timing commit.

---

### Task 6: Create `extractum-llm` and Move the Prepared Sources

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Create: `src-tauri/crates/extractum-llm/Cargo.toml`
- Create: `src-tauri/crates/extractum-llm/src/lib.rs`
- Move: `src-tauri/src/llm/{types.rs,provider.rs,gemini.rs,openai_compat.rs,runner.rs,scheduler.rs,streaming.rs}` -> `src-tauri/crates/extractum-llm/src/`
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/llm/profiles.rs`
- Modify: `src/lib/llm-crate-boundary-contract.test.ts`
- Modify: `src/lib/rust-workspace-core-contract.test.ts`
- Modify: `src/lib/gemini-browser-crate-boundary-contract.test.ts`
- Modify: `docs/value-registry.md`

**Interfaces:**

- Consumes: the clean prepared checkpoint, RED boundary contract, exact manifests/root/facade above.
- Produces: one crate owner, one app dependency edge, explicit private facade, GREEN contracts, and locked dependency graph.

- [ ] **Step 1: Add the member, canonical dependency roots, app inheritance, and path edge.**

Keep the existing one-line workspace layout and make it exactly:

```toml
members = [".", "crates/extractum-core", "crates/extractum-gemini-browser", "crates/extractum-llm"]
```

Insert workspace `reqwest` and `secrecy` in the exact sorted block from `Manifest Contract`. Replace the app's package-local declarations with:

```toml
secrecy = { workspace = true }
reqwest = { workspace = true }
```

Keep each in its existing logical dependency position; add:

```toml
extractum-llm = { path = "crates/extractum-llm" }
```

beside the other local path dependencies. Do not change features on unrelated packages.

- [ ] **Step 2: Create the exact new manifest and crate root.**

Use `apply_patch` to add `src-tauri/crates/extractum-llm/Cargo.toml` from `Manifest Contract` and `src/lib.rs` from `Exact Cross-Crate Interfaces`. Do not add a README, build script, package-local profile, target section, feature flag, glob export, or test-only export.

- [ ] **Step 3: Move the seven prepared files exactly once.**

First prove targets are absent, then use Git-aware moves:

```powershell
$names = @('types.rs','provider.rs','gemini.rs','openai_compat.rs','runner.rs','scheduler.rs','streaming.rs')
foreach ($name in $names) {
  $target = Join-Path 'src-tauri/crates/extractum-llm/src' $name
  if (Test-Path -LiteralPath $target) { throw "Move target already exists: $target" }
}
foreach ($name in $names) {
  git mv -- "src-tauri/src/llm/$name" "src-tauri/crates/extractum-llm/src/$name"
  if ($LASTEXITCODE -ne 0) { throw "git mv failed for $name" }
}
```

Do not edit provider behavior, test assertions, retry/timeouts, messages, or DTO fields during these moves.

- [ ] **Step 4: Replace prepared local-module wiring with the private app facade.**

Remove the moved module declarations from app `mod.rs`; keep `mod app_types; mod profiles;`. Add the exact app re-export block from this plan and local `use extractum_llm::list_provider_models;`.

Update `profiles.rs` imports to consume `LlmProviderAccess`, `ResolvedLlmProfile`, `ProviderKind`, and `normalize_base_url` through `super`, preserving the private facade path. The model-listing seam test must continue to call the facade-visible operation rather than importing a secret API.

No consumer outside `src-tauri/src/llm` may change its `crate::llm` import in this step.

- [ ] **Step 5: Refresh and validate the lock file before locked checks.**

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --no-deps | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata/lock refresh failed' }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Unlocked crate check/lock refresh failed' }
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --no-deps --locked | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Locked Cargo metadata failed' }
git diff -- src-tauri/Cargo.lock
```

Inspect the diff. It must add `extractum-llm` with exactly the seven expected dependencies and add `extractum-llm` to the app package dependency list; registry package versions/checksums should not churn merely from inheritance.

- [ ] **Step 6: Update all existing exact allowlists in the same change.**

In `rust-workspace-core-contract.test.ts`, append `crates/extractum-llm` to the exact member array.

In `gemini-browser-crate-boundary-contract.test.ts`:

- update the exact one-line member string with the fourth member;
- add `reqwest` and `secrecy` to the exact sorted workspace dependency-name array;
- change the exact workspace dependency text to the 11-line block in `Manifest Contract`;
- add `reqwest` and `secrecy` to the app inheritance assertions;
- keep every Gemini Browser package dependency, root, ownership, and test assertion unchanged.

Do not edit historical fixtures under `scripts/process-shell-diagnostic/`.

- [ ] **Step 7: Update value ownership paths, not registry values.**

In `docs/value-registry.md`, replace the representative LLM ownership paths with:

```markdown
- `src-tauri/crates/extractum-llm/src/scheduler.rs`
- `src-tauri/crates/extractum-llm/src/provider.rs`
- `src-tauri/src/llm/mod.rs`
```

Keep all five request-kind, two scheduler-state, and two provider-kind values and meanings unchanged. This is an ownership-path update, not a taxonomy change.

- [ ] **Step 8: Turn the new boundary contract GREEN and run all four boundary files together.**

Finish only mechanical path/regex adjustments required by the final files, then run:

```powershell
npm.cmd run test -- src/lib/llm-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Boundary contract group failed' }
```

Expected: all tests in all four files pass. An empty selection, stale allowlist, relaxed exact root, or removed frozen name is a failure.

- [ ] **Step 9: Run locked package checks and exact ownership sentinels.**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets --locked
if ($LASTEXITCODE -ne 0) { throw 'Locked extractum-llm check failed' }
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum-llm --lib types::tests::resolved_profile_construction_preserves_execution_access_and_public_metadata -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Locked safe-profile test failed' }
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum-llm --lib scheduler::tests::failed_requests_preserve_typed_error_kind -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Locked scheduler error-kind test failed' }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
if ($LASTEXITCODE -ne 0) { throw 'Locked app check failed' }
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum --lib llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Locked app error-shape test failed' }
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum --lib llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Locked profile seam test failed' }
cargo test --manifest-path src-tauri/Cargo.toml --locked -p extractum --lib llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Locked inverse profile seam test failed' }
```

Expected: every exact command runs one test; both packages check successfully.

- [ ] **Step 10: Prove the 36/15 frozen disposition and no disabled/copy owner.**

```powershell
$crateListed = cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib -- --list
if ($LASTEXITCODE -ne 0) { throw 'Crate test listing failed' }
$appListed = cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list
if ($LASTEXITCODE -ne 0) { throw 'App test listing failed' }
$crateLlmTests = @($crateListed | Select-String '^(?:gemini|openai_compat|provider|runner|scheduler|streaming|types)::.*: test$')
$appLlmTests = @($appListed | Select-String '^llm::.*: test$')
if ($crateLlmTests.Count -ne 37) { throw "Expected 37 crate tests, found $($crateLlmTests.Count)" }
if ($appLlmTests.Count -ne 19) { throw "Expected 19 app LLM tests, found $($appLlmTests.Count)" }
npm.cmd run test -- src/lib/llm-crate-boundary-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'LLM boundary contract failed during ownership proof' }
$disabled = rg -n "#\[cfg\(any\(\)\)\]|#\[cfg\(FALSE\)\]" src-tauri/src/llm src-tauri/crates/extractum-llm/src --glob '*.rs'
if ($LASTEXITCODE -eq 0) { throw "Disabled copy found:`n$disabled" }
if ($LASTEXITCODE -ne 1) { throw 'Disabled-copy scan failed' }
```

The source contract supplies exact bare-name ownership; the Cargo counts prove the tests are executable. Manually inspect `git diff --find-renames --summary` and record that the seven implementations were moved, not copied or renamed to evade the scanner. Also compare app `mod.rs` between the Task 3 preparation commit and the staged extraction tree. Record that only module/import/facade paths changed: the `ask_llm_stream` spawn, `run_request`, queue/start/delta ordering, three terminal arms, ignored emit results, and outer immediate `Ok(())` control flow are unchanged.

- [ ] **Step 11: Run package/immediate-consumer checkpoints and commit extraction.**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Task 6 extractum-llm check failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Task 6 extractum-llm test failed' }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Task 6 app check failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Task 6 app test failed' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 6 diff check failed' }
$taskStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 6 status inspection failed' }
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/crates/extractum-llm src-tauri/src/llm src/lib/llm-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts docs/value-registry.md
if ($LASTEXITCODE -ne 0) { throw 'Task 6 staging failed' }
git diff --cached --stat
if ($LASTEXITCODE -ne 0) { throw 'Task 6 staged-stat inspection failed' }
git diff --cached --find-renames --summary
if ($LASTEXITCODE -ne 0) { throw 'Task 6 rename inspection failed' }
git commit -m "refactor: extract portable LLM engine"
if ($LASTEXITCODE -ne 0) { throw 'Task 6 commit failed' }
$finalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 6 final status failed' }
if ($finalStatus.Count -ne 0) { throw "Task 6 commit left a dirty worktree:`n$($finalStatus -join "`n")" }
```

Expected: clean committed extraction; no roadmap/spec status claim yet.

---

### Task 7: Capture Candidate Timing and Verify Both Packages

**Files:**

- Temporarily modify and restore: `src-tauri/crates/extractum-llm/src/streaming.rs`
- Record later in: `docs/superpowers/verification/2026-07-20-extractum-llm-extraction.md`

**Interfaces:**

- Consumes: clean committed extraction and the same prepared logical source used by Task 5.
- Produces: three candidate raw values, median/delta when complete, byte-restoration proof, and final package checkpoints; no timing commit.

- [ ] **Step 1: Run the complete candidate session in one PowerShell process.**

Run the block below exactly. It uses the same marker/warm-up/sample/cleanup
algorithm as baseline and adds an automatic blob comparison with the unique
prepared-source checkpoint:

```powershell
$ErrorActionPreference = 'Stop'
$probe = 'src-tauri/crates/extractum-llm/src/streaming.rs'
$package = 'extractum-llm'
$label = 'CANDIDATE'
$sourceCheckpointSubject = 'refactor: make LLM engine portable'
$sourceCheckpointPath = 'src-tauri/src/llm/streaming.rs'
$marker = $null
$samples = New-Object 'System.Collections.Generic.List[Int64]'
$measurementFailure = $null
$restorationProblems = New-Object 'System.Collections.Generic.List[string]'
$afterHash = $null
$statusAfter = @()
$codexCommand = Get-Command codex.exe -CommandType Application -ErrorAction Stop
$codexExe = $codexCommand.Source

function Invoke-ProbePatch {
  param([Parameter(Mandatory = $true)][string]$PatchText)
  & $codexExe --codex-run-as-apply-patch $PatchText
  if ($LASTEXITCODE -ne 0) { throw 'apply_patch failed for timing probe' }
}

function Add-ProbeMarker {
  param([Parameter(Mandatory = $true)][string]$Path)
  Invoke-ProbePatch @"
*** Begin Patch
*** Update File: $Path
@@
 use extractum_core::error::{AppError, AppResult};
+// extractum-llm timing probe a
*** End Patch
"@
}

function Set-ProbeMarker {
  param(
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter(Mandatory = $true)][string]$From,
    [Parameter(Mandatory = $true)][string]$To
  )
  Invoke-ProbePatch @"
*** Begin Patch
*** Update File: $Path
@@
-// extractum-llm timing probe $From
+// extractum-llm timing probe $To
*** End Patch
"@
}

function Remove-ProbeMarker {
  param(
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter(Mandatory = $true)][string]$Current
  )
  Invoke-ProbePatch @"
*** Begin Patch
*** Update File: $Path
@@
-// extractum-llm timing probe $Current
*** End Patch
"@
}

function Invoke-TimedCheck {
  param(
    [Parameter(Mandatory = $true)][string]$Package,
    [Parameter(Mandatory = $true)][string]$SampleLabel
  )
  $elapsed = Measure-Command {
    & cargo check --manifest-path src-tauri/Cargo.toml -p $Package --all-targets
    if ($LASTEXITCODE -ne 0) { throw "$SampleLabel Cargo check failed" }
  }
  [Int64][Math]::Round($elapsed.TotalMilliseconds)
}

$statusBefore = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Candidate clean-tree check failed' }
if ($statusBefore.Count -ne 0) { throw "Candidate requires a clean worktree:`n$($statusBefore -join "`n")" }
$history = @(git log --format='%H%x09%s')
if ($LASTEXITCODE -ne 0) { throw 'Candidate history inspection failed' }
$checkpointMatches = @($history | Where-Object { ($_ -split "`t", 2)[1] -eq $sourceCheckpointSubject })
if ($checkpointMatches.Count -ne 1) { throw "Expected one '$sourceCheckpointSubject' ancestor, found $($checkpointMatches.Count)" }
$sourceCheckpoint = ($checkpointMatches[0] -split "`t", 2)[0]
git merge-base --is-ancestor $sourceCheckpoint HEAD
if ($LASTEXITCODE -ne 0) { throw 'Prepared-source checkpoint is not an ancestor of HEAD' }
$eol = @(git ls-files --eol -- $probe)
if ($LASTEXITCODE -ne 0) { throw 'Candidate EOL inspection failed' }
if ($eol.Count -ne 1 -or $eol[0] -notmatch '\bw/lf\b') { throw "Candidate probe is not w/lf: $($eol -join ' | ')" }
if (Select-String -LiteralPath $probe -SimpleMatch 'extractum-llm timing probe' -Quiet) {
  throw 'Candidate probe marker already exists'
}
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $probe).Path)
if ([regex]::Matches($source, [regex]::Escape('use extractum_core::error::{AppError, AppResult};')).Count -ne 1) {
  throw 'Expected the exact core-error import once'
}
$beforeHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $probe).Hash
$currentBlob = @(git rev-parse "HEAD:$probe")
if ($LASTEXITCODE -ne 0 -or $currentBlob.Count -ne 1) { throw 'Candidate probe blob resolution failed' }
$referenceBlob = @(git rev-parse "${sourceCheckpoint}:$sourceCheckpointPath")
if ($LASTEXITCODE -ne 0 -or $referenceBlob.Count -ne 1) { throw 'Prepared probe blob resolution failed' }
if ($currentBlob[0] -ne $referenceBlob[0]) {
  throw "Measured probe is not byte-identical: $($referenceBlob[0]) -> $($currentBlob[0])"
}

try {
  Add-ProbeMarker -Path $probe
  $marker = 'a'

  & cargo check --manifest-path src-tauri/Cargo.toml -p $package --all-targets
  if ($LASTEXITCODE -ne 0) { throw 'Candidate warm-up failed' }

  Set-ProbeMarker -Path $probe -From 'a' -To 'b'
  $marker = 'b'
  $samples.Add((Invoke-TimedCheck -Package $package -SampleLabel 'Candidate sample 1')) | Out-Null

  Set-ProbeMarker -Path $probe -From 'b' -To 'a'
  $marker = 'a'
  $samples.Add((Invoke-TimedCheck -Package $package -SampleLabel 'Candidate sample 2')) | Out-Null

  Set-ProbeMarker -Path $probe -From 'a' -To 'b'
  $marker = 'b'
  $samples.Add((Invoke-TimedCheck -Package $package -SampleLabel 'Candidate sample 3')) | Out-Null
} catch {
  $measurementFailure = $_.Exception.Message
} finally {
  foreach ($candidateMarker in @('a', 'b')) {
    try {
      if (Select-String -LiteralPath $probe -SimpleMatch "extractum-llm timing probe $candidateMarker" -Quiet) {
        Remove-ProbeMarker -Path $probe -Current $candidateMarker
      }
    } catch {
      [void]$restorationProblems.Add($_.Exception.Message)
    }
  }
  try {
    $afterHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $probe).Hash
    $statusAfter = @(git status --short)
    if ($LASTEXITCODE -ne 0) { throw 'Candidate final clean-tree check failed' }
  } catch {
    [void]$restorationProblems.Add($_.Exception.Message)
  }
}

if ($restorationProblems.Count -ne 0) { throw "Candidate restoration infrastructure failure: $($restorationProblems -join '; ')" }
if ($afterHash -ne $beforeHash) { throw "Candidate probe restoration failed: $beforeHash -> $afterHash" }
if ($statusAfter.Count -ne 0) { throw "Candidate series left a dirty worktree:`n$($statusAfter -join "`n")" }
"CANDIDATE_SHA256=$afterHash"
"CANDIDATE_BLOB=$($currentBlob[0])"
"CANDIDATE_RAW_MS=$($samples -join ',')"
if ($measurementFailure -or $samples.Count -ne 3) {
  "CANDIDATE_RESULT=incomplete / no conclusion; $measurementFailure"
} else {
  $sorted = @($samples | Sort-Object)
  "CANDIDATE_MEDIAN_MS=$($sorted[1])"
}
```

- [ ] **Step 2: Prove source identity and calculate only a complete comparison.**

Compare `CANDIDATE_SHA256` with the `BASELINE_SHA256` recorded in Task 5.
They must match because Task 6 was a mechanical move. A mismatch is a
correctness investigation, not a timing result.

For complete baseline and candidate triples, record:

```text
baseline raw ms: [b1, b2, b3]
baseline median ms: B
candidate raw ms: [c1, c2, c3]
candidate median ms: C
absolute delta ms: C - B
percentage delta: ((C - B) / B) * 100
```

Round only displayed percentage, not raw/median values. If either series is incomplete, write `no median / no performance conclusion`. Do not retry, reject, revert, or alter gates based on timing.

- [ ] **Step 3: Run non-empty exact crate sentinels.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib types::tests::resolved_profile_construction_preserves_execution_access_and_public_metadata -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Candidate safe-profile sentinel failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib provider::tests::provider_parse_returns_typed_validation_error -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Candidate provider-parse characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib provider::tests::normalize_base_url_allows_https_and_loopback_http_only -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Candidate base-URL characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib runner::tests::validate_request_returns_typed_validation_error -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Candidate request-validation characterization failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib scheduler::tests::failed_requests_preserve_typed_error_kind -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Candidate scheduler sentinel failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib openai_compat::tests::openai_compat_stream_retries_transient_http_before_streaming -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Candidate retry sentinel failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib streaming::tests::sse_data_decode_failures_are_typed_internal_errors -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Candidate SSE sentinel failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store -- --exact
if ($LASTEXITCODE -ne 0) { throw 'Candidate app profile-persistence characterization failed' }
```

- [ ] **Step 4: Run both package checkpoints and the boundary group.**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Candidate extractum-llm check failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Candidate extractum-llm test failed' }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Candidate app check failed' }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Candidate app test failed' }
npm.cmd run test -- src/lib/llm-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Candidate boundary contract group failed' }
$finalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Candidate final status failed' }
if ($finalStatus.Count -ne 0) { throw "Candidate verification left a dirty worktree:`n$($finalStatus -join "`n")" }
```

Expected: all pass and worktree is clean. There is no commit for this task.

---

### Task 8: Run Completion, Release, and Bounded Startup Gates

**Files:**

- Build: `src-tauri/target/release/extractum.exe`
- Record later in: `docs/superpowers/verification/2026-07-20-extractum-llm-extraction.md`

**Interfaces:**

- Consumes: clean correct candidate.
- Produces: mandatory workspace duration, full verification, no-bundle release build, and bounded startup evidence.

- [ ] **Step 1: Run rustfmt and capture the one ordinary mandatory workspace-check duration.**

```powershell
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { throw 'Rustfmt gate failed' }
$workspaceCheckOutput = @()
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets 2>&1 | Tee-Object -Variable workspaceCheckOutput
if ($LASTEXITCODE -ne 0) { throw 'Mandatory workspace check failed' }
$finishedLines = @($workspaceCheckOutput | ForEach-Object { $_.ToString() } | Select-String '^\s*Finished .+ in ([0-9.]+)s$')
if ($finishedLines.Count -ne 1) { throw "Expected one Cargo Finished duration, found $($finishedLines.Count)" }
$workspaceCheckCargoLine = $finishedLines[0].Line.Trim()
$null = $workspaceCheckCargoLine -match ' in ([0-9.]+)s$'
$workspaceCheckMs = [int64][Math]::Round([double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture) * 1000)
"WORKSPACE_CHECK_LINE=$workspaceCheckCargoLine"
"WORKSPACE_CHECK_MS=$workspaceCheckMs"
```

This one value is Phase 5's roadmap timing signal. Do not rerun the workspace check to improve the number. Phase 4 was 1,620 ms, below 15,000 ms, so Phase 5 alone cannot trigger the two-adjacent-slice investigation rule regardless of its result.

- [ ] **Step 2: Run the remaining full completion gates.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Workspace test gate failed' }
npm.cmd run verify
if ($LASTEXITCODE -ne 0) { throw 'Full verification gate failed' }
```

Expected: both pass. Checks nested inside `npm.cmd run verify` do not create additional roadmap timing samples.

- [ ] **Step 3: Build release without MSI bundling.**

```powershell
npm.cmd run tauri -- build --no-bundle
if ($LASTEXITCODE -ne 0) { throw 'No-bundle release build failed' }
$appLiteral = 'src-tauri/target/release/extractum.exe'
if (-not (Test-Path -LiteralPath $appLiteral -PathType Leaf)) { throw "Release executable missing: $appLiteral" }
$appPath = (Resolve-Path -LiteralPath $appLiteral -ErrorAction Stop).Path
```

Full MSI/WiX remains excluded by the documented pre-existing `light.exe` issue.

- [ ] **Step 4: Run the minimal five-second startup smoke with exact-PID cleanup.**

```powershell
$existing = @(Get-Process -Name extractum -ErrorAction SilentlyContinue)
if ($existing.Count -ne 0) { throw "Infrastructure: existing Extractum PID(s): $($existing.Id -join ', ')" }
$appLiteral = 'src-tauri/target/release/extractum.exe'
if (-not (Test-Path -LiteralPath $appLiteral -PathType Leaf)) { throw "Infrastructure: release executable missing: $appLiteral" }
$appPath = (Resolve-Path -LiteralPath $appLiteral -ErrorAction Stop).Path
$app = $null
$appPid = $null
$observationFailure = $null
$cleanupFailure = $null
$cleanupErrors = New-Object 'System.Collections.Generic.List[string]'
try {
  try {
    $app = Start-Process -FilePath $appPath -PassThru -WindowStyle Hidden -ErrorAction Stop
    $appPid = $app.Id
  } catch {
    throw "Infrastructure: failed to start release executable: $($_.Exception.Message)"
  }
  Start-Sleep -Seconds 5
  try { $app.Refresh() } catch { throw "Infrastructure: failed to inspect PID ${appPid}: $($_.Exception.Message)" }
  if ($app.HasExited) { throw "Completion failure: Extractum exited early with code $($app.ExitCode)" }
  "STARTUP_ALIVE_PID=$appPid"
} catch {
  $observationFailure = $_.Exception.Message
} finally {
  if ($null -ne $appPid) {
    $owned = Get-Process -Id $appPid -ErrorAction SilentlyContinue
    if ($null -ne $owned) {
      try { Stop-Process -Id $appPid -Force -ErrorAction Stop } catch { $cleanupErrors.Add("stop failed: $($_.Exception.Message)") }
      try {
        if (-not $app.WaitForExit(10000)) { $cleanupErrors.Add("PID $appPid was not reaped") }
      } catch {
        $cleanupErrors.Add("wait failed: $($_.Exception.Message)")
      }
    }
    if ($null -ne $appPid -and (Get-Process -Id $appPid -ErrorAction SilentlyContinue)) {
      $cleanupErrors.Add("PID $appPid survived cleanup")
    }
  }
  if ($cleanupErrors.Count -ne 0) {
    $cleanupFailure = "Infrastructure cleanup: $($cleanupErrors -join '; ')"
  }
}
if ($observationFailure -and $cleanupFailure) { throw "$observationFailure; $cleanupFailure" }
if ($observationFailure) { throw $observationFailure }
if ($cleanupFailure) { throw $cleanupFailure }
"STARTUP_CLEANUP=PASS"
```

Classification is fixed: pre-existing process, launch/inspection/helper failure, and kill/reap failure are infrastructure; a confirmed readable early exit of `extractum.exe` is a completion failure. Record both observation and cleanup errors if both happen. Do not make a live provider request.

- [ ] **Step 5: Confirm no helper or repository residue.**

```powershell
$processResidue = @(Get-Process -Name extractum -ErrorAction SilentlyContinue)
if ($processResidue.Count -ne 0) { throw "Startup smoke left Extractum PID(s): $($processResidue.Id -join ', ')" }
$finalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Post-startup git status failed' }
if ($finalStatus.Count -ne 0) { throw "Startup smoke left a dirty worktree:`n$($finalStatus -join "`n")" }
```

Expected: no Extractum process from the smoke and clean worktree.

---

### Task 9: Record Verification and Retain Phase 5

**Files:**

- Create: `docs/superpowers/verification/2026-07-20-extractum-llm-extraction.md`
- Modify: `docs/superpowers/specs/2026-07-20-llm-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`

**Interfaces:**

- Consumes: committed extraction, complete correctness/release/startup evidence, and advisory timing disposition.
- Produces: durable Phase 5 result, enforced roadmap state, and Phase 6 as the next JIT design target.

- [ ] **Step 1: Write the verification document from observed output only.**

Use these exact top-level sections:

```markdown
# Extractum LLM Crate Extraction Verification

## Scope and Commits
## Final Ownership and Dependencies
## Safe Profile and Credential Boundary
## Frozen Test Inventory
## IPC, Event, Error, Profile, and Scheduler Compatibility
## Boundary Contract and Mechanical-Move Review
## Package and Workspace Gates
## Advisory Focused Timing
## Ordinary Workspace Timing Signal
## Release and Startup Evidence
## Infrastructure Failures and Exclusions
## Result and Next Phase
```

Record:

- every implementation commit hash/message and the approved spec/plan links;
- final app/crate file owners, seven production dependency roots, Tokio feature placement, and lock evidence;
- safe constructors/getters, both migrated external literals, no external secret getter/serialization, and both black-box partial-input precedence tests;
- all 51 baseline names summarized as exact 36/15 ownership, plus five new Rust tests separately;
- exact event/error characterization outcomes and unchanged nine-command/six-event/profile/scheduler contracts;
- RED reason and GREEN result for the source contract, existing allowlist updates, and manual moved-not-copied review;
- exact commands/results for both package checkpoints and four completion gates;
- baseline/candidate raw samples, medians/delta only when complete, both SHA-256 proofs, and advisory/no-veto disposition;
- exact Cargo `Finished` line and `$workspaceCheckMs` from Task 8, noting Phase 4's 1,620 ms breaks adjacency;
- no-bundle result, executable path, alive PID observation, cleanup result, and no live-provider test;
- any infrastructure failure honestly, with no excluded correctness failure;
- final result `implemented and retained` only if every non-timing gate passed; otherwise leave Phase 5 incomplete.

- [ ] **Step 2: Update spec and roadmap status only after successful evidence.**

Change the LLM spec status to:

```markdown
**Status:** Implemented and retained; [verification](../verification/2026-07-20-extractum-llm-extraction.md)
```

Update the Phase 5 roadmap heading to `done: retained`, link the verification, and replace forecast language with observed final ownership, exact dependencies, 36/15 inventory, advisory timing disposition, and ordinary workspace-check milliseconds. State whether the ordinary result is below or above 15,000 ms, while also stating it cannot form an adjacent pair with Phase 4's 1,620 ms result.

End Phase 5 with: Phase 6 `extractum-prompt-packs` is next and still requires a fresh owner-approved JIT boundary design; this result does not authorize implementation directly.

- [ ] **Step 3: Extend the timing/roadmap contract to Phase 5.**

In `crate-extraction-shell-cap-contract.test.ts`:

- import the Phase 5 design and verification as raw Markdown;
- add a `phase5Roadmap` section from `### Phase 5 —` to `### Phase 6 —`;
- assert the Phase 5 spec status/link, roadmap retained heading/link, seven final dependency roots, 36/15 ownership, small advisory protocol, exact workspace result, and Phase 6-next language;
- assert the verification records three raw values per complete state or explicitly says timing was incomplete with no conclusion;
- assert timing did not decide retention and no shell A/B, quiet-window, scanner, Job Object, retry, or cumulative ledger was introduced;
- keep all Phase 3 and Phase 4 assertions unchanged.

- [ ] **Step 4: Run the documentation/contract checks, then the final repository gate.**

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Final documentation/contract group failed' }
npm.cmd run verify
if ($LASTEXITCODE -ne 0) { throw 'Final repository verification failed' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Final diff check failed' }
```

Expected: all pass. `npm.cmd run verify` after documentation changes is the final repository gate; do not substitute the earlier run.

- [ ] **Step 5: Commit the durable retained result and prove clean handoff.**

```powershell
$taskStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 9 status inspection failed' }
git add docs/superpowers/verification/2026-07-20-extractum-llm-extraction.md docs/superpowers/specs/2026-07-20-llm-crate-boundary-design.md docs/superpowers/specs/2026-07-17-crate-roadmap.md src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Task 9 staging failed' }
git diff --cached --stat
if ($LASTEXITCODE -ne 0) { throw 'Task 9 staged-stat inspection failed' }
git commit -m "docs: record LLM crate extraction"
if ($LASTEXITCODE -ne 0) { throw 'Task 9 commit failed' }
$finalStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Task 9 final status failed' }
if ($finalStatus.Count -ne 0) { throw "Task 9 commit left a dirty worktree:`n$($finalStatus -join "`n")" }
git log -6 --oneline
if ($LASTEXITCODE -ne 0) { throw 'Task 9 handoff log failed' }
```

Expected: clean worktree and a verification-linked retained Phase 5. If any correctness, contract, workspace, release, or confirmed startup gate is not green, do not make this retained-status commit.

---

## Appendix A: Frozen 51-Test Ownership Map

### `extractum-llm` (36)

Gemini provider:

- `gemini_request_mapping_keeps_system_history_and_roles`
- `gemini_request_mapping_keeps_existing_messages_without_output_limit`
- `gemini_stream_chunk_text_and_usage_are_parsed`
- `gemini_model_mapping_uses_short_model_id`
- `gemini_request_rejects_unsupported_roles_with_typed_validation_error`
- `gemini_model_listing_requires_typed_auth_error`
- `gemini_server_error_message_includes_transient_recovery_hint`

OpenAI-compatible provider:

- `openai_compat_request_keeps_standard_roles`
- `openai_compat_stream_chunk_mapping_reads_delta_and_usage`
- `openai_compat_model_mapping_uses_model_id`
- `openai_compat_model_mapping_reads_omniroute_limits_and_capabilities`
- `openai_compat_request_rejects_unsupported_roles_with_typed_validation_error`
- `openai_compat_retry_status_policy_is_bounded_to_transient_failures`
- `openai_compat_stream_retries_transient_http_before_streaming`
- `openai_compat_model_listing_requires_typed_auth_error`

Runner:

- `validate_request_returns_typed_validation_error`
- `resolve_effective_model_returns_typed_validation_error`
- `run_llm_collect_returns_typed_validation_error`

Scheduler:

- `requests_with_different_profiles_run_without_blocking_each_other`
- `interactive_requests_jump_ahead_of_background_queue`
- `queued_requests_can_be_cancelled_before_start`
- `cancelling_owned_run_requests_aborts_running_work`
- `request_snapshots_report_running_and_queued_requests`
- `active_owner_run_ids_reports_running_and_queued_owned_requests`
- `queue_positions_are_recomputed_after_cancelling_a_queued_request`
- `failed_requests_release_capacity_for_next_queued_request`
- `failed_requests_preserve_typed_error_kind`

Streaming:

- `sse_data_is_parsed_from_stream_chunks`
- `sse_data_decode_failures_are_typed_internal_errors`

Provider/model/diagnostic policy:

- `provider_parse_returns_typed_validation_error`
- `provider_parse_accepts_openai_compatible_aliases`
- `model_input_token_limit_lookup_matches_provider_model_ids_and_names`
- `model_output_token_limit_lookup_matches_provider_model_ids_and_names`
- `normalize_base_url_returns_typed_validation_error`
- `normalize_base_url_allows_https_and_loopback_http_only`
- `llm_request_diagnostic_keys_are_stable_snake_case`

### `extractum` (15)

Profiles:

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

App diagnostics:

- `provider_diagnostics_exclude_profile_ids_and_base_urls`
