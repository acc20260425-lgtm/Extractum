# Extractum Prompt Packs Crate Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the portable Prompt Packs and YouTube Summary domain into `extractum-prompt-packs` while preserving every current command, IPC payload, persisted value, execution order, asset byte, and baseline test identity.

**Architecture:** Prepare all cross-crate seams while the code still belongs to `extractum`, ending each preparation checkpoint in a green commit. Then add one intentionally RED source-boundary contract, perform a mechanical ownership move, and retain only a private Tauri facade plus source, Browser, event, credential, and spawn adapters in the application.

**Tech Stack:** Rust 2021, Tauri 2, Tokio 1, SQLx 0.8/SQLite, `extractum-core`, `extractum-llm`, `extractum-gemini-browser`, Serde, Vitest/TypeScript, PowerShell on Windows.

## Global Constraints

- Authority: `docs/superpowers/specs/2026-07-20-prompt-packs-crate-boundary-design.md` at owner-approved status. Do not infer permission for Phase 7 or reopen the canceled process-crate work.
- Start from a clean commit at or after evidence commit `7d748cf94c8de0a0f263900549d530a4b52260a7`; never roll back to that commit. Record the actual `HEAD`, prove the evidence commit is its ancestor, and re-run Task 1 inventories before editing whenever `HEAD` is newer.
- Checkpoints 1–4 each end at a separately identifiable green commit. The application remains the production owner until the mechanical extraction commit.
- Checkpoint 5 is intentionally RED and is committed separately. It may fail only because the future member/path/move is absent.
- The move changes ownership, module-relative imports, manifest inheritance, and the centralized asset/test-migration prefixes. It must not change algorithms, SQL text, serialized values, messages, schemas, provider policy, cancellation behavior, or preflight count.
- Keep all 14 production Tauri commands, both `#[cfg(dev)]` commands, their names, arguments, registrations, and return JSON byte-compatible.
- Keep `src-tauri/src/lib.rs` consumer paths unchanged through the private `prompt_packs` facade.
- Keep migrations and all eight bundled assets single-owned at their current paths. Do not create, rewrite, delete, or rename a migration.
- The crate owns Prompt Pack SQL only. Foreign reads of `sources`, `youtube_video_sources`, `youtube_playlist_items`, `youtube_transcript_segments`, and `items` stay in `extractum`.
- Do not introduce a generic repository/unit-of-work/service locator, universal completion executor, shared SQLite/source/analysis/test-support crate, or dependency on `extractum-analysis`.
- Do not add SQLx, Tokio, or Prompt Pack DTOs to `extractum-core`; do not publish a test helper or add an upward dev dependency.
- Preserve the current fresh-read sequence; do not cache or merge source-reader calls.
- API preflight uses `Some(32_000)` and Browser preflight uses `None`; resolved model limits apply only to later Gem execution budgeting.
- Profile and secret resolution occurs only inside the spawned app task. Never log, persist, serialize, or expose the API key.
- Apply `PromptPackRunState` transitions before synchronously invoking `PromptPackEventSink`; the Tauri sink ignores emit failures exactly as today.
- Keep `extractum_core::error::AppError`; do not introduce a Prompt Packs domain error enum.
- Use canonical `src-tauri/target`; do not add a target directory, harness, runner, Job Object, quiet-window scan, retry policy, or performance ledger.
- Advisory timing cannot veto retention. One warm-up plus three samples per state, no retries; interrupted/failed series means `incomplete / no performance conclusion` after byte restoration.
- Use `npm.cmd`, not `npm`, on Windows. A filtered Cargo command that does not print `running 1 test` is a failed verification.
- Stage only files named by the current task. Preserve unrelated user changes and never use `git reset`, destructive checkout, or history rewriting.

### Phase 6 Roadmap Status State Machine

The parenthetical suffix of the Phase 6 roadmap heading is a closed lifecycle vocabulary. `src/lib/crate-extraction-shell-cap-contract.test.ts` must parse the suffix and accept exactly one of these complete values:

```text
design approved; implementation not started
preparation Checkpoint 1 retained
preparation Checkpoint 2 retained
preparation Checkpoint 3 retained
preparation Checkpoint 4 retained
done: retained
not retained
```

Task 1 installs this standing assertion and changes the roadmap to `preparation Checkpoint 1 retained` in the same green commit. Each later green preparation boundary changes only the checkpoint number in its own commit. `done: retained` is written only after every completion gate; `not retained` is written only by a durable negative disposition when no preparation checkpoint remains the truthful retained state. Every other Phase 6 scope, ownership, timing, and approval assertion in the shell-cap contract remains strict.

The Phase 5 sentence is historical evidence, not the live Phase 6 state. Task 1 changes it once to `At Phase 5 completion, Phase 6 \`extractum-prompt-packs\` remained next`; the shell-cap contract pins that past-tense sentence in every Phase 6 state. It must never branch on crate-manifest existence or continue claiming in the present tense that Phase 6 "remains next".

---

## Rust Verification Loops

Affected packages are the future `extractum-prompt-packs`, its immediate consumer `extractum`, and source contracts protecting the lower `extractum-core`, `extractum-llm`, and `extractum-gemini-browser` crates. Do not repeatedly compile a lower crate unless its own source/API changes.

Before Task 1, read the later `Frozen File Map`, `Frozen 225-Test Ownership`, `Frozen Port and Service API`, visibility allowlist, and manifest/schema/fixture/asset contracts. Those sections are prerequisite task inputs and override any accidental shorthand in an individual step.

Define this helper once in each PowerShell execution shell. It makes a zero-test Cargo success an explicit failure:

```powershell
function Invoke-ExactRust([string]$Package, [string]$Name) {
  $exactOutput = @(cargo test --manifest-path src-tauri/Cargo.toml -p $Package --lib $Name -- --exact 2>&1)
  $exactExit = $LASTEXITCODE
  $exactOutput | Out-Host
  if ($exactExit -ne 0) { throw "$Name failed" }
  if (($exactOutput -join "`n") -notmatch 'running 1 test') {
    throw "$Name did not execute exactly one test"
  }
}
```

Before the move, RED/GREEN tests use `-p extractum`; after the move they use their approved owner. Every public cross-crate interface checkpoint runs both packages:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

End-of-slice gates are fixed:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

### Task 1: Checkpoint 1 — Freeze and Characterize the Existing Boundary

**Files:**

- Create: `src/lib/prompt-pack-application-contract.test.ts`
- Create: `src/lib/prompt-pack-contract-paths.ts`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `src/lib/api/prompt-packs.test.ts`
- Modify: `src-tauri/src/prompt_packs/dto.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/completion_transport.rs`
- Modify: `src-tauri/src/prompt_packs/seed.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`
- Modify: the six `src/lib/prompt-pack-*-contract.test.ts` files named in Task 8

**Interfaces:**

- Consumes: current app-owned Prompt Packs behavior at the recorded start commit.
- Produces: executable 225-test evidence, exact IPC/ordering characterizations, and a raw-source helper that resolves exactly one current or final owner without weakening assertions.

- [ ] **Step 1: Prove identity and a clean start**

```powershell
$phase6Evidence = '7d748cf94c8de0a0f263900549d530a4b52260a7'
$phase6Head = (git rev-parse HEAD).Trim()
git merge-base --is-ancestor $phase6Evidence $phase6Head
if ($LASTEXITCODE -ne 0) {
  throw "HEAD must be at or after Phase 6 evidence commit $phase6Evidence; do not roll back"
}
if ($phase6Head -ne $phase6Evidence) {
  Write-Host "HEAD is newer than the approved evidence; refreshing every Task 1 inventory from: $phase6Head"
}
$phase6Dirty = @(git status --porcelain=v1 --untracked-files=all)
if ($phase6Dirty.Count -ne 0) { throw 'Phase 6 must start from a clean worktree' }
git status --short
```

Expected: no status lines and successful ancestry proof. A newer clean `HEAD` is allowed only after recording it in the verification draft and repeating all inventories in this task.

- [ ] **Step 2: Freeze the executable Cargo inventory**

```powershell
$listed = @(cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs:: -- --list --format terse 2>&1)
if ($LASTEXITCODE -ne 0) { $listed | Out-Host; throw 'Prompt-pack inventory failed' }
$baselineTests = @($listed | ForEach-Object { $_.ToString() } |
  Where-Object { $_ -match '^prompt_packs::.*: test$' } |
  ForEach-Object { $_ -replace ': test$', '' })
if ($baselineTests.Count -ne 225) { throw "Expected 225, found $($baselineTests.Count)" }
if (($baselineTests | Where-Object { $_ -match 'now_string_uses_current_utc_time$' }).Count -ne 2) {
  throw 'Expected both module-qualified now_string identities'
}
$baselineTests | Sort-Object | Set-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-baseline-tests.txt"
```

Expected: 225 executable module-qualified identities; the temporary sorted list is evidence only and is not committed.

- [ ] **Step 3: Add the dual-state raw-source helper and update existing raw readers**

Implement `prompt-pack-contract-paths.ts` with fail-closed ownership:

```ts
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dirname, "../..");

export function readPromptPackDomainSource(
  relativePath: string,
  preparedAppRelativePath = relativePath,
): string {
  const cratePath = resolve(repositoryRoot, "src-tauri/crates/extractum-prompt-packs/src", relativePath);
  const preparedPath = resolve(repositoryRoot, "src-tauri/src/prompt_packs", preparedAppRelativePath);
  const rootTranslation = relativePath === "lib.rs" && preparedAppRelativePath === "mod.rs";
  if (existsSync(cratePath)) {
    if (!rootTranslation && existsSync(preparedPath)) {
      throw new Error(`duplicate Prompt Packs domain owner for ${relativePath}`);
    }
    return readFileSync(cratePath, "utf8");
  }
  if (!existsSync(preparedPath)) {
    throw new Error(`missing Prompt Packs domain owner for ${relativePath}`);
  }
  return readFileSync(preparedPath, "utf8");
}

export function readPromptPackAppFacade(): string {
  return readFileSync(resolve(repositoryRoot, "src-tauri/src/prompt_packs/mod.rs"), "utf8");
}
```

Replace only the static `?raw` imports in these six tests with the helper while keeping every current assertion:

```text
prompt-pack-completion-transport-contract.test.ts
prompt-pack-run-control-contract.test.ts
prompt-pack-run-store-contract.test.ts
prompt-pack-runtime-config-contract.test.ts
prompt-pack-stage-execution-contract.test.ts
prompt-pack-stage-request-policy-contract.test.ts
```

For the future crate root, read `lib.rs` with prepared fallback `mod.rs`; use `readPromptPackAppFacade()` for assertions about the app facade itself.

- [ ] **Step 4: Add exact application-boundary source tests**

Create three tests with these exact titles:

```text
keeps all production and dev command attributes registrations and argument spellings
keeps start idempotency readiness preflight queued-event spawn and profile-resolution order
keeps startup seed and interrupted-run cleanup wiring
```

The first test parses the current/final wrapper sources and uses this literal argument map; it requires one `#[tauri::command]` definition and one `tauri::generate_handler!` registration for every name:

```ts
const commandArguments = {
  get_prompt_pack_library: ["handle"],
  preflight_youtube_summary_run: ["handle", "project_id", "source_ids", "profile_id", "model_override", "runtime_provider", "browser_provider_config", "output_language", "control_preset", "evidence_mode", "include_comments"],
  start_youtube_summary_run: ["handle", "state", "client_request_id", "project_id", "source_ids", "profile_id", "model_override", "runtime_provider", "browser_provider_config", "output_language", "control_preset", "evidence_mode", "include_comments"],
  cancel_prompt_pack_run: ["handle", "state", "scheduler", "run_id"],
  update_prompt_pack_run: ["handle", "run_id", "run_label"],
  delete_prompt_pack_run: ["handle", "state", "run_id"],
  list_prompt_pack_runs: ["handle", "project_id", "limit"],
  list_active_prompt_pack_runs: ["handle", "state"],
  list_prompt_pack_run_stages: ["handle", "run_id"],
  get_prompt_pack_result: ["handle", "run_id"],
  list_prompt_pack_stage_artifacts: ["handle", "stage_run_id"],
  get_prompt_pack_stage_artifact: ["handle", "stage_run_id", "artifact_kind", "attempt_number", "artifact_index"],
  get_prompt_pack_validation_findings: ["handle", "run_id"],
  list_prompt_pack_audit_events: ["handle", "run_id"],
} as const;
const devCommandArguments = {
  seed_prompt_pack_cancellation_smoke_fixture: ["handle", "state"],
  clear_prompt_pack_cancellation_smoke_fixture: ["handle", "state"],
} as const;
```

Extract parameter identifiers from each Rust function body and compare exact arrays; `src/lib/api/prompt-packs.test.ts` separately asserts the current camelCase invoke payload keys. The second source test compares token indexes in the fixed order `empty-ID guard < first existing lookup < Browser readiness < second lookup/preflight < track_if_absent < queued state/event < spawn`, and asserts `resolve_profile_for_backend` occurs inside the spawned-task block. The third requires both `seed_builtin_prompt_packs(handle.clone())` and `cleanup_interrupted_prompt_pack_runs(handle.clone())` in application setup, in that order.

- [ ] **Step 5: Write the missing Rust characterizations first**

Add these exact tests and assert literal JSON objects/strings, not only successful serialization:

```rust
prompt_packs::dto::tests::start_outcomes_serialize_exact_ipc_contract
prompt_packs::dto::tests::prompt_pack_run_events_serialize_exact_ipc_contract
prompt_packs::dto::tests::prompt_pack_errors_serialize_exact_json_contract
prompt_packs::youtube_summary::preflight_tests::api_runtime_preflight_uses_fixed_32000_input_limit
prompt_packs::seed::tests::bundled_assets_hashes_and_source_path_match_canonical_bytes
```

The event test covers queued, started, repair/Gem-stage, completed, failed, and cancelled values. For each event assert all camelCase keys, explicit `null` optionals, exact `message`/`error` text, and the channel `prompt-pack-run-event`. The error test covers validation, not-found, conflict, and internal `AppError` JSON. The asset test asserts all eight full hashes in the frozen asset table and the persisted source path.

- [ ] **Step 6: Add orchestration characterizations without changing behavior**

In existing test modules, add fakes/counters that prove:

```text
empty client_request_id -> no Browser/source call
existing request -> no Browser/source call
new Browser request -> readiness before source reads
new runnable request -> outer preflight + skeleton preflight + post-insert fresh reads
queued state applied and event published before spawn directive
profile resolution hook runs only inside the spawned execution hook
candidate comment body is read for estimate, then selected body is read again
Browser cancellation precedes terminal persistence/event
Browser provenance is persisted before completion validation
```

Do not refactor production code in this step; expose only `#[cfg(test)]` hooks where a source-order assertion otherwise cannot observe the boundary.

- [ ] **Step 7: Install the lifecycle-status contract and record Checkpoint 1**

In `src/lib/crate-extraction-shell-cap-contract.test.ts`, replace only the fixed Phase 6 status assertion with a fail-closed extraction of the roadmap heading suffix and the exact closed set from `Phase 6 Roadmap Status State Machine`. Require a match before indexing it; do not use a loose regex such as `Checkpoint \d` and do not remove or union any other assertion. The core check is:

```typescript
const phase6Status = phase6Roadmap.match(
  /### Phase 6 — `extractum-prompt-packs` \(([^)]+)\)/,
)?.[1];

expect(phase6Status).toBeDefined();
expect([
  "design approved; implementation not started",
  "preparation Checkpoint 1 retained",
  "preparation Checkpoint 2 retained",
  "preparation Checkpoint 3 retained",
  "preparation Checkpoint 4 retained",
  "done: retained",
  "not retained",
]).toContain(phase6Status);
```

Also make the three Phase 5 assertions permanently historical and exact: require `At Phase 5 completion, Phase 6 \`extractum-prompt-packs\` remained next`, `fresh JIT boundary design was owner-approved`, and `implementation was not authorized`. In the roadmap, change that paragraph to the same past tense and change only the Phase 6 heading suffix to `preparation Checkpoint 1 retained`. Keep the owner-instruction prerequisite and all Phase 6 boundary/timing prose unchanged.

Run the standing contract immediately:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
```

Expected: the contract passes with the exact Checkpoint 1 state and no present-tense `Phase 6 \`extractum-prompt-packs\` remains next` assertion survives.

- [ ] **Step 8: Run exact focused and source tests**

```powershell
Invoke-ExactRust extractum 'prompt_packs::dto::tests::start_outcomes_serialize_exact_ipc_contract'
Invoke-ExactRust extractum 'prompt_packs::dto::tests::prompt_pack_run_events_serialize_exact_ipc_contract'
Invoke-ExactRust extractum 'prompt_packs::dto::tests::prompt_pack_errors_serialize_exact_json_contract'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::preflight_tests::api_runtime_preflight_uses_fixed_32000_input_limit'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::preflight_tests::browser_runtime_preflight_does_not_apply_api_input_limit'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::start_returns_existing_run_for_duplicate_client_request_id'
Invoke-ExactRust extractum 'prompt_packs::gemini_browser_stage::tests::timeout_latest_ok_result_is_not_prompt_completion'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::cleanup_interrupted_prompt_pack_runs_marks_stale_active_rows_interrupted'
Invoke-ExactRust extractum 'prompt_packs::seed::tests::bundled_assets_hashes_and_source_path_match_canonical_bytes'
npm.cmd run test -- src/lib/prompt-pack-application-contract.test.ts src/lib/api/prompt-packs.test.ts src/lib/prompt-pack-completion-transport-contract.test.ts src/lib/prompt-pack-run-control-contract.test.ts src/lib/prompt-pack-run-store-contract.test.ts src/lib/prompt-pack-runtime-config-contract.test.ts src/lib/prompt-pack-stage-execution-contract.test.ts src/lib/prompt-pack-stage-request-policy-contract.test.ts
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: each exact Rust invocation prints `running 1 test`; Vitest and the app check pass.

- [ ] **Step 9: Review and commit Checkpoint 1 GREEN**

```powershell
git diff --check
git status --short
git add docs/superpowers/specs/2026-07-17-crate-roadmap.md src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/prompt-pack-application-contract.test.ts src/lib/prompt-pack-contract-paths.ts src/lib/api/prompt-packs.test.ts src/lib/prompt-pack-completion-transport-contract.test.ts src/lib/prompt-pack-run-control-contract.test.ts src/lib/prompt-pack-run-store-contract.test.ts src/lib/prompt-pack-runtime-config-contract.test.ts src/lib/prompt-pack-stage-execution-contract.test.ts src/lib/prompt-pack-stage-request-policy-contract.test.ts src-tauri/src/prompt_packs/dto.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/completion_transport.rs src-tauri/src/prompt_packs/seed.rs src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs
git commit -m "test: characterize prompt-pack boundary and start phase 6"
```

Expected: clean Checkpoint 1 commit; no unrelated paths staged.

### Task 2: Checkpoint 2 — Make Cross-Crate Values Safely Constructible

**Files:**

- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src-tauri/src/prompt_packs/dto.rs`
- Modify: `src-tauri/src/prompt_packs/library.rs`
- Modify: `src-tauri/src/prompt_packs/run_control.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/stage_io.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/mod.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`

**Interfaces:**

- Consumes: Checkpoint 1 serialization and ordering characterizations.
- Produces: private-field DTOs with complete constructors, narrow accessors, and the exact public lifecycle surface in the visibility allowlist.

- [ ] **Step 1: Add a failing constructor/serialization test**

Add exact identity:

```rust
#[test]
fn crate_boundary_constructors_and_accessors_preserve_serialized_shapes() {
    let request = StartYoutubeSummaryRunRequest::new(
        "client-1".into(), Some(7), vec![11], Some("profile-1".into()),
        Some("model-1".into()), PromptPackRuntimeProvider::Api, None,
        "English".into(), "detailed_report".into(), "strict".into(), true,
    );
    assert_eq!(request.client_request_id(), "client-1");
    assert_eq!(request.runtime_provider(), PromptPackRuntimeProvider::Api);
    assert_eq!(request.profile_id(), Some("profile-1"));
    assert_eq!(request.model_override(), Some("model-1"));
}
```

Run and require a real missing-method compile failure:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs::dto::tests::crate_boundary_constructors_and_accessors_preserve_serialized_shapes -- --exact
```

Expected: non-zero because `new`/accessors do not exist, never `0 tests`.

- [ ] **Step 2: Add exact constructors and narrow accessors**

Implement these signatures without changing Serde derives or rename rules:

```rust
impl PreflightYoutubeSummaryRunRequest {
    pub fn new(project_id: Option<i64>, source_ids: Vec<i64>, profile_id: Option<String>,
        model_override: Option<String>, runtime_provider: PromptPackRuntimeProvider,
        browser_provider_config: Option<GeminiBrowserProviderConfig>, output_language: String,
        control_preset: String, evidence_mode: String, include_comments: bool) -> Self;
}

impl StartYoutubeSummaryRunRequest {
    pub fn new(client_request_id: String, project_id: Option<i64>, source_ids: Vec<i64>,
        profile_id: Option<String>, model_override: Option<String>,
        runtime_provider: PromptPackRuntimeProvider,
        browser_provider_config: Option<GeminiBrowserProviderConfig>, output_language: String,
        control_preset: String, evidence_mode: String, include_comments: bool) -> Self;
    pub fn client_request_id(&self) -> &str;
    pub fn runtime_provider(&self) -> PromptPackRuntimeProvider;
    pub fn profile_id(&self) -> Option<&str>;
    pub fn model_override(&self) -> Option<&str>;
}

impl ListPromptPackRunsRequest {
    pub fn new(project_id: Option<i64>, limit: Option<i64>) -> Self;
}
```

Make input fields private. Keep response fields private unless current app code reads them; replace such reads with a named accessor. Update all app-side struct literals now, including tests and `stage_io.rs`, so the later move contains no consumer repair.

- [ ] **Step 3: Narrow library DTOs and freeze run-state methods**

Keep the five library DTOs serializable with unchanged camelCase output, but make fields private and construct them inside `library.rs`. After extraction, only `PromptPackRunState::new` is public because the app constructs the Tauri-managed state; `track_if_absent`, `request_cancel`, `child_token`, `finish`, `active_run_ids`, and `apply_event` remain crate-visible and are invoked only by exported services. Leave internal maps/tokens private.

- [ ] **Step 4: Run GREEN checks**

```powershell
Invoke-ExactRust extractum 'prompt_packs::dto::tests::crate_boundary_constructors_and_accessors_preserve_serialized_shapes'
Invoke-ExactRust extractum 'prompt_packs::dto::tests::start_outcomes_serialize_exact_ipc_contract'
Invoke-ExactRust extractum 'prompt_packs::library::tests::get_prompt_pack_library_returns_active_youtube_summary_pack'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::prompt_pack_run_state_tracks_active_and_cancel_requested_runs'
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: all exact tests run once and pass; app all-target check passes.

- [ ] **Step 5: Commit Checkpoint 2 GREEN**

After the code gates pass, change only the Phase 6 roadmap heading suffix from `preparation Checkpoint 1 retained` to `preparation Checkpoint 2 retained`, then prove the standing lifecycle contract remains GREEN:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
git diff --check
git add docs/superpowers/specs/2026-07-17-crate-roadmap.md src-tauri/src/prompt_packs/dto.rs src-tauri/src/prompt_packs/library.rs src-tauri/src/prompt_packs/run_control.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/stage_io.rs src-tauri/src/prompt_packs/youtube_summary/mod.rs src-tauri/src/prompt_packs/youtube_summary/test_support.rs src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs
git commit -m "refactor: prepare prompt-pack public construction"
```

Expected: the Checkpoint 2 commit is independently GREEN and its roadmap status is truthful if execution pauses here.

### Task 3: Introduce the Source Reader and App SQL Adapter

**Files:**

- Create: `src-tauri/src/prompt_packs/source_port.rs`
- Create: `src-tauri/src/prompt_packs/source_adapter.rs`
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/preflight.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`
- Delete after all callers migrate: `src-tauri/src/prompt_packs/youtube_summary/sources.rs`

**Interfaces:**

- Consumes: frozen source ABI and private-field construction from Task 2.
- Produces: six object-safe reads, a fresh-query app adapter, and scripted/counting fakes; no foreign SQL remains in crate-destined files.

- [ ] **Step 1: Write six failing adapter tests and four domain-sequence tests**

Add exact identities:

```text
prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables
prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows
prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows
prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id
prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback
prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback
prompt_packs::youtube_summary::snapshots_tests::runnable_start_uses_complete_fresh_source_read_sequence
prompt_packs::youtube_summary::snapshots_tests::selected_comment_body_is_reloaded_after_candidate_estimation
prompt_packs::youtube_summary::snapshots_tests::transcript_material_policy_uses_owned_segment_reader_values
prompt_packs::youtube_summary::snapshots_tests::comment_material_ref_policy_preserves_order_and_token_cap
```

The scripted fake records an ordered enum log. The runnable-start assertion must include outer preflight reads, repeated skeleton preflight reads, post-insert source/video/playlist/transcript reads, candidate reads, and a fresh body read for every selected external ID.

- [ ] **Step 2: Implement the frozen source ABI**

Create all types, constructors, getters, alias, and six trait methods exactly as frozen above. Do not expose SQLx rows, borrowed data, table names, eligibility policy, token caps, or material-ref rules.

- [ ] **Step 3: Move every foreign read into `AppPromptPackSourceReader`**

Implement the trait for:

```rust
#[derive(Clone)]
pub struct AppPromptPackSourceReader { pool: SqlitePool }

impl AppPromptPackSourceReader {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
}
```

Each method issues a fresh SQL query. Preserve these exact SQL semantics:

```text
load_source: sources by id; nullable subtype/title
load_video: youtube_video_sources plus sources metadata; missing row remains None
load_playlist_items: active rows ordered by position ASC, id ASC; preserve unlinked rows
load_transcript_segments: ordered by segment_index ASC, id ASC
select_comment_candidates: youtube_comment items ordered by published_at IS NULL ASC,
  published_at ASC, external_id ASC, id ASC; apply requested LIMIT; decompression failure -> empty body
load_comment_body: a separate SELECT by source_id + external_id; missing/null/decompression failure -> empty string
```

- [ ] **Step 4: Convert domain code to the port without changing call order**

Thread `&dyn PromptPackSourceReader` through preflight, skeleton, snapshots, and Gem material preparation. Reuse no request-wide loaded graph. Keep `SqlitePool` beside the port for prompt-pack-owned writes. Move title fallback, playlist-child status, origin inclusion, token estimates, comment material IDs, and caps into domain code.

- [ ] **Step 5: Run the source seam GREEN**

```powershell
Invoke-ExactRust extractum 'prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables'
Invoke-ExactRust extractum 'prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows'
Invoke-ExactRust extractum 'prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows'
Invoke-ExactRust extractum 'prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id'
Invoke-ExactRust extractum 'prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback'
Invoke-ExactRust extractum 'prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::runnable_start_uses_complete_fresh_source_read_sequence'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::selected_comment_body_is_reloaded_after_candidate_estimation'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::transcript_material_policy_uses_owned_segment_reader_values'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::comment_material_ref_policy_preserves_order_and_token_cap'
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

- [ ] **Step 6: Commit the independently green source seam**

```powershell
git diff --check
git add src-tauri/src/prompt_packs/source_port.rs src-tauri/src/prompt_packs/source_adapter.rs src-tauri/src/prompt_packs/mod.rs src-tauri/src/prompt_packs/youtube_summary/preflight.rs src-tauri/src/prompt_packs/youtube_summary/snapshots.rs src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs src-tauri/src/prompt_packs/youtube_summary/test_support.rs src-tauri/src/prompt_packs/youtube_summary/sources.rs
git commit -m "refactor: isolate prompt-pack source reads"
```

### Task 4: Introduce Browser and Event Ports

**Files:**

- Create: `src-tauri/src/prompt_packs/browser_port.rs`
- Create: `src-tauri/src/prompt_packs/browser_adapter.rs`
- Create: `src-tauri/src/prompt_packs/events.rs`
- Create: `src-tauri/src/prompt_packs/event_adapter.rs`
- Modify: `src-tauri/src/prompt_packs/dto.rs`
- Modify: `src-tauri/src/prompt_packs/completion_transport.rs`
- Modify: `src-tauri/src/prompt_packs/run_control.rs`
- Modify: `src-tauri/src/prompt_packs/stage_execution.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/mod.rs`

**Interfaces:**

- Consumes: complete lower-crate Browser types and frozen legacy event JSON.
- Produces: object-safe three-operation Browser execution and synchronous typed event publication with state-before-sink ordering.

- [ ] **Step 1: Write failing port/mapping/order tests**

Add exact tests:

```text
prompt_packs::browser_adapter::tests::browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result
prompt_packs::event_adapter::tests::typed_events_map_to_exact_legacy_ipc_payloads
prompt_packs::run_control::tests::apply_event_updates_state_before_synchronous_sink_observes_it
prompt_packs::completion_transport::tests::api_stage_uses_background_scheduler_prompt_pack_metadata_and_typed_cancellation
```

The Browser fake must compare the complete `GeminiBrowserRunResult`, including status, completion reason, provider mode, latest message, artifacts/run-log identity, and output. The event test compares serialized legacy JSON byte-for-byte with Task 1 fixtures.

- [ ] **Step 2: Implement Browser ABI and Tauri adapter**

Implement the frozen trait/request types. `TauriGeminiBrowserPort` owns cloned `AppHandle`/Browser state access and delegates without policy to the three current concrete functions. Keep artifact mode exactly `"reduced"`.

- [ ] **Step 3: Implement event ABI, state application, and legacy mapper**

Move the serializable `PromptPackRunEvent` and `PROMPT_PACK_RUN_EVENT` from `dto.rs` to app-owned `event_adapter.rs`. Implement `PromptPackEventSink` for `TauriPromptPackEventSink`. Change every crate-destined publication site to:

```rust
state.apply_event(&event).await;
events.emit(event);
```

The sink maps all 13 fields exactly and ignores only `AppHandle::emit` failure. No crate-destined module imports `Emitter` or the IPC DTO.

- [ ] **Step 4: Convert Browser completion and cancellation to the port**

Remove `AppHandle` and concrete Browser calls from `completion_transport.rs` and `stage_execution.rs`. Pass `Arc<dyn PromptPackBrowserExecutor>` and preserve prompt formatting, deterministic run/source IDs, queue/active cancellation, latest-OK rejection, provenance persistence, and typed cancellation distinctions.

- [ ] **Step 5: Run GREEN checks and commit**

```powershell
Invoke-ExactRust extractum 'prompt_packs::browser_adapter::tests::browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result'
Invoke-ExactRust extractum 'prompt_packs::event_adapter::tests::typed_events_map_to_exact_legacy_ipc_payloads'
Invoke-ExactRust extractum 'prompt_packs::run_control::tests::apply_event_updates_state_before_synchronous_sink_observes_it'
Invoke-ExactRust extractum 'prompt_packs::completion_transport::tests::api_stage_uses_background_scheduler_prompt_pack_metadata_and_typed_cancellation'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::prompt_pack_browser_stage_cancelled_while_active_stops_sidecar'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::persist_browser_stage_provenance_records_result_identity'
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
git diff --check
git add src-tauri/src/prompt_packs/browser_port.rs src-tauri/src/prompt_packs/browser_adapter.rs src-tauri/src/prompt_packs/events.rs src-tauri/src/prompt_packs/event_adapter.rs src-tauri/src/prompt_packs/dto.rs src-tauri/src/prompt_packs/completion_transport.rs src-tauri/src/prompt_packs/run_control.rs src-tauri/src/prompt_packs/stage_execution.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/mod.rs
git commit -m "refactor: isolate prompt-pack browser and events"
```

### Task 5: Complete Checkpoint 3 with the Opaque Execution Handoff

**Files:**

- Create: `src-tauri/src/prompt_packs/runtime_commands.rs`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/runtime_config.rs`
- Modify: `src-tauri/src/prompt_packs/completion_transport.rs`
- Modify: `src-tauri/src/prompt_packs/stage_execution.rs`
- Modify: `src-tauri/src/prompt_packs/mod.rs`

**Interfaces:**

- Consumes: source, Browser, event ports; `ResolvedLlmProfile`; `LlmSchedulerState`.
- Produces: exact `StartServiceOutcome`, borrowed-ticket preparation, provider-specific prepared values, and a thin app task-spawn/profile adapter.

- [ ] **Step 1: Write failing start/ticket/profile tests**

Add exact identities:

```text
prompt_packs::runtime::tests::start_service_rejects_empty_id_before_browser_or_source_ports
prompt_packs::runtime::tests::start_service_returns_existing_before_browser_or_source_ports
prompt_packs::runtime::tests::start_service_issues_ticket_after_queued_event_and_new_tracking
prompt_packs::runtime::tests::start_service_returns_ticket_for_untracked_existing_queued_run
prompt_packs::runtime::tests::prepare_execution_borrows_the_same_ticket_for_terminal_failure
prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task
prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket
```

Use non-clone compile/runtime shape checks for the ticket; do not expose a constructor merely to test it.

- [ ] **Step 2: Implement the frozen service handoff**

Implement the exact types/functions in the frozen API section. `start_youtube_summary_run_service` owns this sequence:

```text
empty-ID guard
first idempotency lookup
Browser readiness only for a new Browser request
second idempotency lookup
outer fixed-budget preflight
skeleton's repeated fixed-budget preflight and insertion
run reload
run_status == "queued" && track_if_absent(run_id)
state.apply_event(queued) then sink.emit(queued)
opaque ticket only when tracking was newly acquired
```

- [ ] **Step 3: Make `runtime_commands.rs` the only spawn/profile owner**

Use this control shape; adapters are constructed inside the wrapper and the response is returned unchanged:

```rust
fn spawn_youtube_summary_execution(handle: AppHandle, ticket: RunExecutionTicket) {
    tauri::async_runtime::spawn(async move {
        let pool = match get_pool(&handle).await {
            Ok(pool) => pool,
            Err(error) => {
                eprintln!("Prompt Pack run {} could not acquire the application pool: {error}", ticket.run_id());
                return;
            }
        };
        let state = handle.state::<PromptPackRunState>();
        let events: Arc<dyn PromptPackEventSink> = Arc::new(TauriPromptPackEventSink::new(handle.clone()));
        let prepared = match prepare_run_execution(&pool, &ticket).await {
            Ok(value) => value,
            Err(error) => {
                let _ = fail_run_execution(&pool, state.inner(), events, &ticket, &error).await;
                return;
            }
        };
        let result = match prepared {
            PreparedRunExecution::Api(api) => match resolve_profile_for_backend(&handle, api.profile_id()).await {
                Ok(profile) => execute_prepared_api_run(
                    &pool, state.inner(), handle.state::<LlmSchedulerState>().inner(), events.clone(), api, profile,
                ).await,
                Err(error) => Err(error),
            },
            PreparedRunExecution::GeminiBrowser(browser_run) => execute_prepared_browser_run(
                &pool, state.inner(), Arc::new(TauriGeminiBrowserPort::new(handle.clone())), events.clone(), browser_run,
            ).await,
        };
        if let Err(error) = result {
            let _ = fail_run_execution(&pool, state.inner(), events, &ticket, &error).await;
        }
    });
}
```

Use the repository's existing error logging style around pool/failure-service errors, but never print profile secrets. `execute_prepared_api_run` performs effective-model resolution and emits Started only after it succeeds.

- [ ] **Step 4: Move all Tauri commands into the app adapter and leave portable runtime green**

`runtime_commands.rs` retains the original command attributes/signatures and uses constructors from Task 2. `runtime.rs` retains services, SQL, state, execution orchestration, and its 40 baseline tests, with no `tauri`, `AppHandle`, `State`, `Emitter`, `Manager`, `get_pool`, `resolve_profile_for_backend`, or task-spawn token.

- [ ] **Step 5: Run Checkpoint 3 GREEN**

```powershell
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::start_service_rejects_empty_id_before_browser_or_source_ports'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::start_service_returns_existing_before_browser_or_source_ports'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::start_service_issues_ticket_after_queued_event_and_new_tracking'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::start_service_returns_ticket_for_untracked_existing_queued_run'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::prepare_execution_borrows_the_same_ticket_for_terminal_failure'
Invoke-ExactRust extractum 'prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task'
Invoke-ExactRust extractum 'prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::terminal_event_removes_run_from_active_state'
npm.cmd run test -- src/lib/prompt-pack-application-contract.test.ts
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

- [ ] **Step 6: Commit Checkpoint 3 GREEN**

After the code gates pass, change only the Phase 6 roadmap heading suffix from `preparation Checkpoint 2 retained` to `preparation Checkpoint 3 retained`, then prove the standing lifecycle contract remains GREEN:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
git diff --check
git add docs/superpowers/specs/2026-07-17-crate-roadmap.md src-tauri/src/prompt_packs/runtime_commands.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/runtime_config.rs src-tauri/src/prompt_packs/completion_transport.rs src-tauri/src/prompt_packs/stage_execution.rs src-tauri/src/prompt_packs/mod.rs
git commit -m "refactor: prepare prompt-pack execution handoff"
```

This commit is the Checkpoint 3 boundary. Stopping here is a valid green pause.

## Frozen File Map

### Final application-owned files

| Path | Responsibility |
| --- | --- |
| `src-tauri/src/prompt_packs/mod.rs` | Private compatibility facade; explicit re-exports only. |
| `src-tauri/src/prompt_packs/runtime_commands.rs` | Tauri lifecycle commands, `get_pool`, exact one-task spawn, profile/credential resolution. |
| `src-tauri/src/prompt_packs/library_command.rs` | `get_prompt_pack_library` Tauri/get-pool wrapper. |
| `src-tauri/src/prompt_packs/result_commands.rs` | Five result/artifact/finding/audit Tauri/get-pool wrappers. |
| `src-tauri/src/prompt_packs/seed_command.rs` | Startup seeding/get-pool wrapper. |
| `src-tauri/src/prompt_packs/source_adapter.rs` | `AppPromptPackSourceReader`; all foreign-table SQL reads and decompression fallback. |
| `src-tauri/src/prompt_packs/browser_adapter.rs` | `TauriGeminiBrowserPort`; concrete Browser status, submission, cancellation. |
| `src-tauri/src/prompt_packs/event_adapter.rs` | Legacy serializable event DTO, channel, exact mapper, Tauri emitter. |
| `src-tauri/src/prompt_packs/youtube_summary/mod.rs` | Test-only module preserving the two app-owned logical paths. |
| `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs` | Exactly the two app-owned baseline tests. |
| `src-tauri/src/prompt_packs/youtube_summary/test_support.rs` | Private fixtures used only by those two app tests. |

Delete the prepared app-owned `dto.rs`, `runtime.rs`, `completion_transport.rs`, and all other crate-destined modules only through the reviewed move. Do not retain duplicate implementations.

### Final crate-owned root files

Create `src-tauri/crates/extractum-prompt-packs/Cargo.toml` and move the prepared portable modules to `src-tauri/crates/extractum-prompt-packs/src/`:

```text
assets.rs
browser_port.rs
completion_transport.rs
dto.rs
events.rs
gemini_browser_stage.rs
json_repair.rs
lib.rs
library.rs
models.rs
projections.rs
result_builder.rs
result_service.rs
run_control.rs
run_store.rs
runtime.rs
runtime_config.rs
seed.rs
source_port.rs
stage_execution.rs
stage_io.rs
stage_output_normalization.rs
stage_request_policy.rs
store.rs
test_schema.rs
validation.rs
```

Move these files to `src-tauri/crates/extractum-prompt-packs/src/youtube_summary/`:

```text
entities.rs
entities_tests.rs
execution.rs
execution_result.rs
execution_tests.rs
facade_tests.rs
gem_analysis.rs
mod.rs
outputs.rs
outputs_tests.rs
preflight.rs
preflight_tests.rs
progress.rs
result_validation.rs
snapshots.rs
snapshots_tests.rs          # nine crate-owned tests only
store.rs
synthesis_execution.rs
synthesis_input.rs
synthesis_input_tests.rs
tail_stages.rs
test_support.rs             # crate-private domain fixtures only
transcript_execution.rs
types.rs
```

`src-tauri/src/prompt_packs/youtube_summary/sources.rs` is not moved or copied. Its portable values/trait become `source_port.rs`, while its SQL implementation becomes `source_adapter.rs`.

## Frozen 225-Test Ownership

The full logical identity is `(Appendix A logical file/module, leaf test name)` from the approved spec. The implementation and standing contract must parse that Appendix rather than maintain a second 225-name literal. The final owner is the exact set complement below:

| Appendix A group | Baseline count | Final owner |
| --- | ---: | --- |
| `completion_transport.rs` | 2 | crate |
| `dto.rs` | 2 | crate |
| `gemini_browser_stage.rs` | 3 | crate |
| `library.rs` | 1 | crate |
| `projections.rs` | 5 | crate |
| `result_builder.rs` | 11 | crate |
| `runtime.rs` | 40 | crate |
| `seed.rs` | 5 | crate |
| `stage_io.rs` | 3 | crate |
| `stage_output_normalization.rs` | 1 | crate |
| `store.rs` | 2 | crate |
| `validation.rs` | 25 | crate |
| `youtube_summary/entities_tests.rs` | 11 | crate |
| `youtube_summary/execution_tests.rs` | 19 | crate |
| `youtube_summary/facade_tests.rs` | 1 | crate |
| `youtube_summary/gem_analysis.rs` | 11 | crate |
| `youtube_summary/outputs_tests.rs` | 15 | crate |
| `youtube_summary/preflight_tests.rs` | 5 | crate |
| `youtube_summary/result_validation.rs` | 47 | crate |
| `youtube_summary/snapshots_tests.rs` | 9 crate + 2 app | split exactly below |
| `youtube_summary/synthesis_input_tests.rs` | 5 | crate |

The only app-owned Appendix A identities are:

```text
prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer
prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled
```

Every other Appendix A pair is crate-owned: 223 crate + 2 app = 225. New characterization tests are outside this frozen set. Treat the two `now_string_uses_current_utc_time` leaves as distinct because their logical modules differ.

## Frozen Port and Service API

### Source port

Put this ABI in `source_port.rs`. All fields are private. Every request/record receives a complete public `new(...)`; request getters are public for `AppPromptPackSourceReader`, while record getters are crate-visible unless an app test consumes them.

```rust
use std::{future::Future, pin::Pin};
use extractum_core::error::AppResult;

pub type PromptPackPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

pub trait PromptPackSourceReader: Send + Sync + 'static {
    fn load_source(&self, source_id: i64)
        -> PromptPackPortFuture<'_, Option<PromptPackSourceRecord>>;
    fn load_video(&self, request: YoutubeVideoReadRequest)
        -> PromptPackPortFuture<'_, Option<PromptPackYoutubeVideoRecord>>;
    fn load_playlist_items(&self, playlist_source_id: i64)
        -> PromptPackPortFuture<'_, Vec<PromptPackPlaylistItemRecord>>;
    fn load_transcript_segments(&self, source_id: i64)
        -> PromptPackPortFuture<'_, Vec<PromptPackTranscriptSegment>>;
    fn select_comment_candidates(&self, request: CommentCandidateReadRequest)
        -> PromptPackPortFuture<'_, Vec<PromptPackCommentCandidate>>;
    fn load_comment_body(&self, request: CommentBodyReadRequest)
        -> PromptPackPortFuture<'_, String>;
}
```

Freeze these value shapes:

```rust
PromptPackSourceRecord {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
    title: Option<String>,
}
YoutubeVideoReadRequest { source_id: i64 }
PromptPackYoutubeVideoRecord {
    source_id: i64,
    video_id: String,
    canonical_url: String,
    title: Option<String>,
    channel_title: Option<String>,
    published_at: Option<String>,
    description: Option<String>,
}
PromptPackPlaylistItemRecord {
    video_source_id: Option<i64>,
    video_id: String,
    title: Option<String>,
}
PromptPackTranscriptSegment { start_ms: i64, end_ms: i64, text: String }
CommentCandidateReadRequest { source_id: i64, limit: i64 }
PromptPackCommentCandidate { external_id: Option<String>, body: String }
CommentBodyReadRequest { source_id: i64, external_id: Option<String> }
```

Derive `Clone, Debug, Eq, PartialEq` for the values. Preserve title fallback, playlist-child classification, comment caps, token estimates, material refs, and title coalescing inside the crate rather than in the adapter.

### Browser port

```rust
use extractum_gemini_browser::{
    GeminiBrowserProviderConfig, GeminiBrowserProviderStatus, GeminiBrowserRunResult,
};

pub type PromptPackBrowserFuture<'a, T> =
    Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

pub trait PromptPackBrowserExecutor: Send + Sync + 'static {
    fn read_status(&self, request: PromptPackBrowserStatusRequest)
        -> PromptPackBrowserFuture<'_, GeminiBrowserProviderStatus>;
    fn submit(&self, request: PromptPackBrowserRunRequest)
        -> PromptPackBrowserFuture<'_, GeminiBrowserRunResult>;
    fn cancel(&self, request: PromptPackBrowserCancelRequest)
        -> PromptPackBrowserFuture<'_, ()>;
}
```

The three request types have private fields, complete public constructors, and adapter getters:

```rust
PromptPackBrowserStatusRequest {
    provider_config: Option<GeminiBrowserProviderConfig>,
}
PromptPackBrowserRunRequest {
    run_id: String,
    prompt: String,
    source: String,
    artifact_mode: String, // current value is always "reduced"
    provider_config: Option<GeminiBrowserProviderConfig>,
}
PromptPackBrowserCancelRequest { run_id: String }
```

Map methods exactly to app-owned `provider_status`, `send_single_prompt`, and `cancel_gemini_browser_job`. Never narrow `GeminiBrowserRunResult` to text at the port.

### Event and execution handoff

`PromptPackEvent` has the current 13 event fields and types but no Serde derive. The app maps it to its retained serializable `PromptPackRunEvent` and constant `PROMPT_PACK_RUN_EVENT = "prompt-pack-run-event"`.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPackEvent {
    pub run_id: i64,
    pub request_id: String,
    pub kind: String,
    pub run_status: String,
    pub phase: String,
    pub stage_run_id: Option<i64>,
    pub stage_name: Option<String>,
    pub source_snapshot_id: Option<i64>,
    pub queue_position: Option<i64>,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

pub trait PromptPackEventSink: Send + Sync + 'static {
    fn emit(&self, event: PromptPackEvent);
}
```

`PromptPackRunState::apply_event(&self, event: &PromptPackEvent)` is async, crate-visible, and borrows the event. Every publisher awaits it before calling the synchronous sink.

Freeze the opaque, non-Serde, non-`Clone` handoff:

```rust
pub struct StartServiceOutcome {
    pub response: StartYoutubeSummaryRunOutcomeDto,
    pub execution_ticket: Option<RunExecutionTicket>,
}

pub struct RunExecutionTicket { run_id: i64 }
impl RunExecutionTicket { pub fn run_id(&self) -> i64 { self.run_id } }

pub enum PreparedRunExecution {
    Api(PreparedApiRunExecution),
    GeminiBrowser(PreparedBrowserRunExecution),
}
pub struct PreparedApiRunExecution {
    run_id: i64,
    profile_id: Option<String>,
    model_override: Option<String>,
}
pub struct PreparedBrowserRunExecution {
    run_id: i64,
    browser_provider_config: Option<GeminiBrowserProviderConfig>,
}
```

`PreparedApiRunExecution` exposes `profile_id(&self) -> Option<&str>` and `model_override(&self) -> Option<&str>`. Prepared values are consumed by execution. Resolve the spec's borrow/consume ambiguity in favor of borrowing the ticket: the app must retain the same unforgeable ticket for preparation, profile-resolution, effective-model, or execution failure handling.

```rust
pub async fn prepare_run_execution(
    pool: &SqlitePool,
    ticket: &RunExecutionTicket,
) -> AppResult<PreparedRunExecution>;

pub async fn execute_prepared_api_run(
    pool: &SqlitePool,
    state: &PromptPackRunState,
    scheduler: &LlmSchedulerState,
    events: Arc<dyn PromptPackEventSink>,
    prepared: PreparedApiRunExecution,
    profile: ResolvedLlmProfile,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>;

pub async fn execute_prepared_browser_run(
    pool: &SqlitePool,
    state: &PromptPackRunState,
    browser: Arc<dyn PromptPackBrowserExecutor>,
    events: Arc<dyn PromptPackEventSink>,
    prepared: PreparedBrowserRunExecution,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>;

pub async fn fail_run_execution(
    pool: &SqlitePool,
    state: &PromptPackRunState,
    events: Arc<dyn PromptPackEventSink>,
    ticket: &RunExecutionTicket,
    error: &AppError,
) -> AppResult<()>;
```

Freeze the remaining app-facing service signatures as well:

```rust
pub async fn get_prompt_pack_library_in_pool(pool: &SqlitePool)
    -> AppResult<PromptPackLibraryDto>;
pub async fn seed_builtin_prompt_packs_in_pool(pool: &SqlitePool) -> AppResult<()>;

pub async fn preflight_youtube_summary_run(
    source: &dyn PromptPackSourceReader,
    request: PreflightYoutubeSummaryRunRequest,
) -> AppResult<YoutubeSummaryPreflightResponse>;
pub async fn start_youtube_summary_run_service(
    pool: &SqlitePool,
    state: &PromptPackRunState,
    source: &dyn PromptPackSourceReader,
    browser: &dyn PromptPackBrowserExecutor,
    events: &dyn PromptPackEventSink,
    request: StartYoutubeSummaryRunRequest,
) -> AppResult<StartServiceOutcome>;
pub async fn cancel_prompt_pack_run_in_pool(
    pool: &SqlitePool,
    state: &PromptPackRunState,
    scheduler: &LlmSchedulerState,
    events: &dyn PromptPackEventSink,
    run_id: i64,
) -> AppResult<()>;
pub async fn update_prompt_pack_run_in_pool(
    pool: &SqlitePool, run_id: i64, run_label: Option<String>,
) -> AppResult<PromptPackRunSummaryDto>;
pub async fn delete_prompt_pack_run_in_pool(
    pool: &SqlitePool, state: &PromptPackRunState, run_id: i64,
) -> AppResult<()>;
pub async fn list_prompt_pack_runs_in_pool(
    pool: &SqlitePool, request: ListPromptPackRunsRequest,
) -> AppResult<Vec<PromptPackRunSummaryDto>>;
pub async fn list_active_prompt_pack_runs_in_pool(
    pool: &SqlitePool, state: &PromptPackRunState,
) -> AppResult<Vec<PromptPackRunSummaryDto>>;
pub async fn list_prompt_pack_run_stages_in_pool(
    pool: &SqlitePool, run_id: i64,
) -> AppResult<Vec<PromptPackStageRunDto>>;
pub async fn cleanup_interrupted_prompt_pack_runs_in_pool(
    pool: &SqlitePool, state: &PromptPackRunState,
) -> AppResult<()>;

pub async fn get_prompt_pack_result_in_pool(
    pool: &SqlitePool, run_id: i64,
) -> AppResult<PromptPackResultDto>;
pub async fn list_prompt_pack_stage_artifacts_in_pool(
    pool: &SqlitePool, stage_run_id: i64,
) -> AppResult<Vec<PromptPackStageArtifactSummaryDto>>;
pub async fn get_prompt_pack_stage_artifact_in_pool(
    pool: &SqlitePool, stage_run_id: i64, artifact_kind: String,
    attempt_number: i64, artifact_index: i64,
) -> AppResult<PromptPackStageArtifactDto>;
pub async fn get_prompt_pack_validation_findings_in_pool(
    pool: &SqlitePool, run_id: i64,
) -> AppResult<Vec<PromptPackValidationFindingDto>>;
pub async fn list_prompt_pack_audit_events_in_pool(
    pool: &SqlitePool, run_id: i64,
) -> AppResult<Vec<PromptPackAuditEventDto>>;

#[cfg(dev)]
pub async fn seed_prompt_pack_cancellation_smoke_fixture_in_pool(
    pool: &SqlitePool, state: &PromptPackRunState,
) -> AppResult<PromptPackRunSummaryDto>;
#[cfg(dev)]
pub async fn clear_prompt_pack_cancellation_smoke_fixture_in_pool(
    pool: &SqlitePool, state: &PromptPackRunState,
) -> AppResult<i64>;
```

The app resolves `ResolvedLlmProfile` only after `prepare_run_execution` returns the API variant inside the spawned task. It calls `fail_run_execution` with the original ticket on any preparation/profile/effective-model/execution error. A Started event is impossible before successful profile/effective-model resolution.

### Curated crate root

`lib.rs` declares all modules privately and explicitly re-exports only:

- DTOs: `PromptPackRuntimeProvider`, `PreflightYoutubeSummaryRunRequest`, `StartYoutubeSummaryRunRequest`, `YoutubeSummaryPreflightResponse`, `YoutubeSummaryPreflightVideo`, `YoutubeSummaryPreflightSkippedVideo`, `YoutubeSummaryPreflightFailure`, `ListPromptPackRunsRequest`, `PromptPackRunSummaryDto`, `PromptPackStageRunDto`, `StartYoutubeSummaryRunOutcomeDto`, `PromptPackResultDto`, `PromptPackStageArtifactSummaryDto`, `PromptPackStageArtifactDto`, `PromptPackValidationFindingDto`, `PromptPackAuditEventDto`, `PromptPackLibraryDto`, `PromptPackDto`, `PromptPackVersionDto`, `PromptPackStageTemplateDto`, and `PromptPackSchemaAssetDto`.
- Source ABI: `PromptPackPortFuture`, `PromptPackSourceReader`, and the eight source request/record types above.
- Browser ABI: `PromptPackBrowserFuture`, `PromptPackBrowserExecutor`, and the three Browser request types.
- Runtime ABI: `PromptPackEvent`, `PromptPackEventSink`, `PromptPackRunState`, `StartServiceOutcome`, `RunExecutionTicket`, `PreparedRunExecution`, `PreparedApiRunExecution`, `PreparedBrowserRunExecution`, and `YoutubeSummaryRunExecutionOutcome`.
- Service functions: `get_prompt_pack_library_in_pool`, `seed_builtin_prompt_packs_in_pool`, `preflight_youtube_summary_run`, `start_youtube_summary_run_service`, `cancel_prompt_pack_run_in_pool`, `update_prompt_pack_run_in_pool`, `delete_prompt_pack_run_in_pool`, `list_prompt_pack_runs_in_pool`, `list_active_prompt_pack_runs_in_pool`, `list_prompt_pack_run_stages_in_pool`, `get_prompt_pack_result_in_pool`, `list_prompt_pack_stage_artifacts_in_pool`, `get_prompt_pack_stage_artifact_in_pool`, `get_prompt_pack_validation_findings_in_pool`, `list_prompt_pack_audit_events_in_pool`, `cleanup_interrupted_prompt_pack_runs_in_pool`, `prepare_run_execution`, `execute_prepared_api_run`, `execute_prepared_browser_run`, and `fail_run_execution`.
- Under `#[cfg(dev)]` only: `seed_prompt_pack_cancellation_smoke_fixture_in_pool` and `clear_prompt_pack_cancellation_smoke_fixture_in_pool`.

No module is public. No glob export, row type, schema type, asset constant, validation/stage internal, or test helper is public.

## Exact Visibility-Widening Allowlist

Only these prepared app-private items may become public at the crate edge:

| Current/prepared owner | Public item after move | Consumer |
| --- | --- | --- |
| `dto.rs` | the 16 command-facing DTOs listed above plus their constructors/accessors | app wrappers and public service signatures |
| `library.rs` | five library DTOs and `get_prompt_pack_library_in_pool` | `library_command.rs` |
| `run_control.rs` | `PromptPackRunState` and `PromptPackRunState::new`; other lifecycle methods remain crate-visible | Tauri-managed app state and public service signatures |
| `run_store.rs` | update/delete/list/list-active/stage-list pool services | app runtime wrappers |
| `result_service.rs` | five pool result/artifact/finding/audit services | `result_commands.rs` |
| `runtime.rs` | start/preflight/cancel/cleanup/dev-fixture services and execution handoff types/functions | `runtime_commands.rs` |
| `seed.rs` | `seed_builtin_prompt_packs_in_pool` | `seed_command.rs` |
| `source_port.rs` | source trait, alias, six request/record families | source adapter and public service signatures |
| `browser_port.rs` | Browser trait, alias, three requests | Browser adapter and public service signatures |
| `events.rs` | event value and sink trait | event adapter and public service signatures |
| `youtube_summary/types.rs` | `YoutubeSummaryRunExecutionOutcome` | runtime execution result |

All other current `pub(crate)`, `pub(super)`, and private items remain crate-private after the move. Do not widen any item for a test; crate tests use private module access.

## Exact Manifest, Lock, Schema, Fixture, and Asset Contracts

The workspace and new manifest must be exactly:

```toml
[workspace]
members = [".", "crates/extractum-core", "crates/extractum-gemini-browser", "crates/extractum-llm", "crates/extractum-prompt-packs"]
resolver = "2"

[workspace.dependencies]
sha2 = "0.10"
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
```

```toml
[package]
name = "extractum-prompt-packs"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
extractum-core = { path = "../extractum-core" }
extractum-gemini-browser = { path = "../extractum-gemini-browser" }
extractum-llm = { path = "../extractum-llm" }
jsonschema = { version = "0.46.5", default-features = false }
serde.workspace = true
serde_json.workspace = true
sha2.workspace = true
sqlx.workspace = true
tokio = { workspace = true, features = ["macros", "sync"] }
tokio-util.workspace = true

[dev-dependencies]
tempfile.workspace = true
time.workspace = true
tokio = { workspace = true, features = ["io-util", "net", "rt", "time"] }
```

The exact direct-root allowlists above are closed. In particular, the new crate must not directly depend on Tauri or a Tauri plugin, `keyring`, `secrecy`, `reqwest`, `parking_lot`, `url`, `zstd`, Apalis, Grammers, `windows-sys`, a process crate, `extractum`, or `extractum-analysis`. It also must not add Tokio `test-util`. Compression comes from `extractum-core`, provider HTTP/secrets from `extractum-llm`, and Browser internals from `extractum-gemini-browser` plus the app port.

The app gains `extractum-prompt-packs = { path = "crates/extractum-prompt-packs" }`, changes `sha2` and `sqlx` to `.workspace = true`, loses `jsonschema`, and retains direct `sha2`, `sqlx`, and `tempfile`. The new lock package has exactly these dependencies and no `source` or `checksum`:

```text
extractum-core
extractum-gemini-browser
extractum-llm
jsonschema
serde
serde_json
sha2
sqlx
tempfile
time
tokio
tokio-util
```

The app lock stanza gains exactly one `extractum-prompt-packs` edge and loses `jsonschema`; lower crates gain no reverse edge. Registry versions remain `jsonschema 0.46.5`, `sha2 0.10.9`, `sqlx 0.8.6`, `tempfile 3.27.0`, `time 0.3.47`, `tokio 1.52.1`, and `tokio-util 0.7.18`.

Production SQL in the new crate is limited to these 32 domain tables:

```text
prompt_packs
prompt_pack_versions
prompt_pack_stage_templates
prompt_pack_schema_assets
prompt_pack_runs
prompt_pack_run_scopes
prompt_pack_run_source_snapshots
prompt_pack_run_source_origins
prompt_pack_run_material_snapshots
prompt_pack_stage_runs
prompt_pack_stage_artifacts
prompt_pack_results
prompt_pack_result_source_refs
prompt_pack_result_claims
prompt_pack_result_evidence
prompt_pack_result_ref_edges
prompt_pack_result_unknowns
prompt_pack_result_verification_tasks
prompt_pack_result_warnings
prompt_pack_result_limitations
prompt_pack_result_quality_flags
prompt_pack_result_audit_refs
prompt_pack_youtube_videos
prompt_pack_youtube_segments
prompt_pack_youtube_key_points
prompt_pack_youtube_quotes
prompt_pack_youtube_action_items
prompt_pack_youtube_open_questions
prompt_pack_youtube_synthesis_items
prompt_pack_result_validation_findings
prompt_pack_audit_events
prompt_pack_result_quarantine_artifacts
```

`test_schema.rs` contains a private `#[cfg(test)]` ordered array with exactly these repository paths and canonical includes:

```rust
const PROMPT_PACK_TEST_MIGRATIONS: [(&str, &str); 12] = [
    ("src-tauri/migrations/0001_current_schema_baseline.sql", include_str!("../../../migrations/0001_current_schema_baseline.sql")),
    ("src-tauri/migrations/0002_migrated_history_opt_in_schema.sql", include_str!("../../../migrations/0002_migrated_history_opt_in_schema.sql")),
    ("src-tauri/migrations/0003_analysis_telegram_history_scope.sql", include_str!("../../../migrations/0003_analysis_telegram_history_scope.sql")),
    ("src-tauri/migrations/0004_source_delete_cascade_indexes.sql", include_str!("../../../migrations/0004_source_delete_cascade_indexes.sql")),
    ("src-tauri/migrations/0005_projects_mvp.sql", include_str!("../../../migrations/0005_projects_mvp.sql")),
    ("src-tauri/migrations/0006_prompt_pack_mvp.sql", include_str!("../../../migrations/0006_prompt_pack_mvp.sql")),
    ("src-tauri/migrations/0007_prompt_pack_run_idempotency.sql", include_str!("../../../migrations/0007_prompt_pack_run_idempotency.sql")),
    ("src-tauri/migrations/0008_prompt_pack_run_labels.sql", include_str!("../../../migrations/0008_prompt_pack_run_labels.sql")),
    ("src-tauri/migrations/0009_prompt_pack_intermediate_entities_artifacts.sql", include_str!("../../../migrations/0009_prompt_pack_intermediate_entities_artifacts.sql")),
    ("src-tauri/migrations/0010_prompt_pack_runtime_provider.sql", include_str!("../../../migrations/0010_prompt_pack_runtime_provider.sql")),
    ("src-tauri/migrations/0011_prompt_pack_stage_browser_provenance.sql", include_str!("../../../migrations/0011_prompt_pack_stage_browser_provenance.sql")),
    ("src-tauri/migrations/0012_projects_redesign.sql", include_str!("../../../migrations/0012_projects_redesign.sql")),
];
```

Apply them with `sqlx::raw_sql` in one transaction. Do not create `_sqlx_migrations`, include Apalis SQL, import app Rust, or export the helper. Before the move, the include prefix is `../../migrations/`; the mechanical move alone changes it to `../../../migrations/`.

Centralize the eight assets in private `assets.rs`; after the move, each include uses `concat!(env!("CARGO_MANIFEST_DIR"), "/../../prompt-packs/youtube_summary/1.0.0/...")`. Preserve `bundled_source_path = "src-tauri/prompt-packs/youtube_summary/1.0.0"` and these SHA-384 values:

| Relative asset | SHA-384 |
| --- | --- |
| `pack.json` | `21d0e7803f25474bb761cbe5c9fe6e45ef363cf5d9c7f030f7c84ee02ef9b7d8dd3664dfed782a3e8c607b7a0f37cf06` |
| `runtime/synthesis.json` | `36b1c4653bc4befdcd168b482929f3b34980c58d9179cb0e0e3db9ac4d3760f9e66dc834ad6a799df6df62618b28d367` |
| `runtime/transcript_analysis.json` | `a9ba63c8ff582429866042aad354693cf9a583f5fc05f319189f44266d9eec6871b0ceb40758719a4b0d95dc8f25ee8f` |
| `schemas/canonical-result.json` | `067ac18d452b6ec6ca2000899d3e7d8df87ace30e4676c7f88080a59cc4731887032943c7ea961ac39b69ab17e9697fd` |
| `schemas/stage-io-youtube-summary-synthesis-output.json` | `ff518213fba16805dfbde2c6c55f8d3ca204ca7f772fb2348cfc375e83070289bfd29623ea4af1b78044504e92a22dac` |
| `schemas/stage-io-youtube-summary-transcript-analysis-input.json` | `bb75aad9fd645912f723ad470a715f7b43c3af964ee4ea74cd84bebb635a1d3bc5bb0ac5460c9608e15eabee07b74419` |
| `schemas/stage-io-youtube-summary-transcript-analysis-output.json` | `9d3d32cf7b7bfd00fdc5ae6d74dac8ad06f488b05e31e52866553aeaa1cd836c1d6599d5dd21c2228abf51e4bcc5f693` |
| `stages/transcript_analysis.json` | `1b4f18dc3b1baf4b01389a6187d54b96ed689dc044aefd6338a2a176779f433359b0bdc77364fec1ef2ccb58a9088793` |

---

### Task 6: Isolate Pool Services, Assets, and Core Utilities

**Files:**

- Create: `src-tauri/src/prompt_packs/assets.rs`
- Create: `src-tauri/src/prompt_packs/library_command.rs`
- Create: `src-tauri/src/prompt_packs/seed_command.rs`
- Create: `src-tauri/src/prompt_packs/result_service.rs`
- Modify: `src-tauri/src/prompt_packs/library.rs`
- Modify: `src-tauri/src/prompt_packs/result_commands.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/runtime_commands.rs`
- Modify: `src-tauri/src/prompt_packs/seed.rs`
- Modify: `src-tauri/src/prompt_packs/stage_request_policy.rs`
- Modify: `src-tauri/src/prompt_packs/validation.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/execution.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`
- Modify: `src-tauri/src/prompt_packs/mod.rs`

**Interfaces:**

- Consumes: exact app wrappers and frozen service allowlist.
- Produces: pool-taking portable services, one asset owner, and crate-destined production code using `extractum-core` for error/compression/time.

- [ ] **Step 1: Add failing pool-service and asset-owner tests**

Add source/Rust tests that require all five result commands, library, seed, run list/update/delete/stage list, cancellation, cleanup, and dev fixture operations to have pool/service functions. Add one source assertion that exactly one Rust file contains each of the eight `include_str!` asset paths.

Use exact Rust sentinels:

```text
prompt_packs::library::tests::get_prompt_pack_library_returns_active_youtube_summary_pack
prompt_packs::runtime::tests::list_prompt_pack_runs_returns_recent_runs_for_project
prompt_packs::runtime::tests::update_prompt_pack_run_updates_user_label_only
prompt_packs::runtime::tests::delete_prompt_pack_run_rejects_active_runs
prompt_packs::seed::tests::seed_youtube_summary_pack_is_idempotent
```

- [ ] **Step 2: Centralize all compile-time assets**

Create private constants in `assets.rs` using the app-preparation prefix:

```rust
pub(crate) const PACK_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/youtube_summary/1.0.0/pack.json"));
pub(crate) const TRANSCRIPT_RUNTIME_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/youtube_summary/1.0.0/runtime/transcript_analysis.json"));
pub(crate) const SYNTHESIS_RUNTIME_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json"));
pub(crate) const CANONICAL_RESULT_SCHEMA_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/youtube_summary/1.0.0/schemas/canonical-result.json"));
pub(crate) const TRANSCRIPT_INPUT_SCHEMA_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-input.json"));
pub(crate) const TRANSCRIPT_OUTPUT_SCHEMA_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-output.json"));
pub(crate) const SYNTHESIS_OUTPUT_SCHEMA_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-synthesis-output.json"));
pub(crate) const TRANSCRIPT_STAGE_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prompt-packs/youtube_summary/1.0.0/stages/transcript_analysis.json"));
pub(crate) const BUNDLED_SOURCE_PATH: &str = "src-tauri/prompt-packs/youtube_summary/1.0.0";
```

Rust does not allow concatenating a const into `include_str!`; therefore every macro uses the explicit `concat!(env!(...), literal)` shown above. Replace every direct include in `seed.rs`, `stage_request_policy.rs`, `validation.rs`, and `youtube_summary/result_validation.rs` with these constants.

- [ ] **Step 3: Split pool behavior from Tauri wrappers**

Use this exact wrapper pattern for library, seed, and all five result commands:

```rust
#[tauri::command]
pub async fn get_prompt_pack_library(handle: AppHandle) -> AppResult<PromptPackLibraryDto> {
    let pool = get_pool(&handle).await?;
    get_prompt_pack_library_in_pool(&pool).await
}
```

`library.rs`, `seed.rs`, `result_service.rs`, `run_store.rs`, and portable `runtime.rs` own SQL/service behavior. App wrapper files contain only Tauri attributes, state/pool acquisition, adapter construction, delegation, and existing logging.

- [ ] **Step 4: Route portable primitives through `extractum-core`**

In every crate-destined production module replace app imports with:

```rust
use extractum_core::compression::{compress_text, decompress_text};
use extractum_core::error::{AppError, AppResult};
use extractum_core::time::now_rfc3339_utc;
```

Replace `seed.rs` wall-clock code and the direct `time` use in `youtube_summary/execution.rs` with the core time API. Do not add SQLx, Tokio, or Prompt Pack types to `extractum-core`.

- [ ] **Step 5: Prove wrappers and portable modules are green**

```powershell
Invoke-ExactRust extractum 'prompt_packs::library::tests::get_prompt_pack_library_returns_active_youtube_summary_pack'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::list_prompt_pack_runs_returns_recent_runs_for_project'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::update_prompt_pack_run_updates_user_label_only'
Invoke-ExactRust extractum 'prompt_packs::runtime::tests::delete_prompt_pack_run_rejects_active_runs'
Invoke-ExactRust extractum 'prompt_packs::seed::tests::seed_youtube_summary_pack_is_idempotent'
Invoke-ExactRust extractum 'prompt_packs::seed::tests::bundled_assets_hashes_and_source_path_match_canonical_bytes'
npm.cmd run test -- src/lib/prompt-pack-application-contract.test.ts
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

- [ ] **Step 6: Commit the green pool/asset preparation**

```powershell
git diff --check
git add src-tauri/src/prompt_packs/assets.rs src-tauri/src/prompt_packs/library_command.rs src-tauri/src/prompt_packs/seed_command.rs src-tauri/src/prompt_packs/result_service.rs src-tauri/src/prompt_packs/library.rs src-tauri/src/prompt_packs/result_commands.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/runtime_commands.rs src-tauri/src/prompt_packs/seed.rs src-tauri/src/prompt_packs/stage_request_policy.rs src-tauri/src/prompt_packs/validation.rs src-tauri/src/prompt_packs/youtube_summary/execution.rs src-tauri/src/prompt_packs/youtube_summary/result_validation.rs src-tauri/src/prompt_packs/mod.rs
git commit -m "refactor: isolate prompt-pack pool services and assets"
```

### Task 7: Complete Checkpoint 4 with the Private Test Schema and 223/2 Split

**Files:**

- Create: `src-tauri/src/prompt_packs/test_schema.rs`
- Create: `src-tauri/src/prompt_packs/youtube_summary/domain_snapshots_tests.rs`
- Create: `src-tauri/src/prompt_packs/youtube_summary/app_test_support.rs`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/mod.rs`
- Modify test-schema callers in `library.rs`, `projections.rs`, `result_builder.rs`, `runtime.rs`, `seed.rs`, `stage_io.rs`, `store.rs`, `validation.rs`, `youtube_summary/result_validation.rs`, and `youtube_summary/test_support.rs`

**Interfaces:**

- Consumes: canonical app migration SQL files and the frozen Appendix A partition.
- Produces: a private reduced schema bootstrap, a mechanically movable nine-test file, two app tests with app-only support, and Checkpoint 4 GREEN.

- [ ] **Step 1: Write fixture-shape tests before the fixture**

Add exact identities:

```text
prompt_packs::test_schema::tests::canonical_fixture_applies_declared_consumed_schema
prompt_packs::test_schema::tests::canonical_fixture_preserves_consumed_indexes_and_foreign_keys
```

The first test opens an in-memory SQLite pool, applies the 12 entries, and asserts every table/column consumed by moved tests. The second uses SQLite pragmas to pin required indexes and foreign keys; it does not assert Apalis tables or a migration history table.

- [ ] **Step 2: Implement the private canonical fixture**

Use the exact 12-entry array from the frozen contract, but while it remains at `src-tauri/src/prompt_packs/test_schema.rs` use `include_str!("../../migrations/<file>")`. Apply it in one transaction:

```rust
#[cfg(test)]
pub(crate) async fn prompt_pack_test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    let mut transaction = pool.begin().await.unwrap();
    for (_, sql) in PROMPT_PACK_TEST_MIGRATIONS {
        sqlx::raw_sql(sql).execute(&mut *transaction).await.unwrap();
    }
    transaction.commit().await.unwrap();
    pool
}
```

Keep the module and helper private under `#[cfg(test)]`. Do not import `crate::migrations` or `apply_all_migrations_for_test_pool` in any crate-destined test.

- [ ] **Step 3: Prepare the snapshots test split without changing logical identities**

Move the nine crate-owned test bodies from `snapshots_tests.rs` into `domain_snapshots_tests.rs`. Keep the two approved app-owned bodies in `snapshots_tests.rs`, import their extracted fixtures through `super::app_test_support`, declare `#[cfg(test)] mod app_test_support;` in the prepared YouTube module, and include the domain file inside the existing `snapshots_tests` module:

```rust
include!("domain_snapshots_tests.rs");
```

This preserves all eleven current `prompt_packs::youtube_summary::snapshots_tests::*` executable identities before the move. Put only app migration/source fixtures used by the two retained tests in `app_test_support.rs`; keep portable domain fixtures in `test_support.rs`.

- [ ] **Step 4: Convert all crate-destined tests to the private fixture**

Replace transitive app migration helpers in exactly these callers:

```text
library.rs
projections.rs
result_builder.rs
runtime.rs
seed.rs
stage_io.rs
store.rs
validation.rs
youtube_summary/result_validation.rs
youtube_summary/test_support.rs
```

Keep migration-registration and the two source-SQL tests on the app's full migration helper.

- [ ] **Step 5: Run fixture, migration, ownership, and full app-package gates**

```powershell
Invoke-ExactRust extractum 'prompt_packs::test_schema::tests::canonical_fixture_applies_declared_consumed_schema'
Invoke-ExactRust extractum 'prompt_packs::test_schema::tests::canonical_fixture_preserves_consumed_indexes_and_foreign_keys'
Invoke-ExactRust extractum 'migrations::tests::prompt_pack_mvp_migration_creates_library_and_run_tables'
Invoke-ExactRust extractum 'migrations::tests::prompt_pack_mvp_migration_declares_required_integrity_constraints'
Invoke-ExactRust extractum 'migrations::tests::build_migrations_includes_prompt_pack_runtime_provider_version_ten'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled'
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs::
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: the filtered run is non-empty and the full package run executes all 225 Appendix A identities plus the new characterizations. Do not reuse Task 1's `Count -eq 225` assertion here because the added characterizations legitimately increase the Cargo list; Task 8 performs the exact standing Appendix A comparison before ownership moves.

- [ ] **Step 6: Commit Checkpoint 4 GREEN**

After the code gates pass, change only the Phase 6 roadmap heading suffix from `preparation Checkpoint 3 retained` to `preparation Checkpoint 4 retained`, then prove the standing lifecycle contract remains GREEN:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
git diff --check
git add docs/superpowers/specs/2026-07-17-crate-roadmap.md src-tauri/src/prompt_packs/test_schema.rs src-tauri/src/prompt_packs/youtube_summary/domain_snapshots_tests.rs src-tauri/src/prompt_packs/youtube_summary/app_test_support.rs src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs src-tauri/src/prompt_packs/youtube_summary/test_support.rs src-tauri/src/prompt_packs/youtube_summary/mod.rs src-tauri/src/prompt_packs/library.rs src-tauri/src/prompt_packs/projections.rs src-tauri/src/prompt_packs/result_builder.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/seed.rs src-tauri/src/prompt_packs/stage_io.rs src-tauri/src/prompt_packs/store.rs src-tauri/src/prompt_packs/validation.rs src-tauri/src/prompt_packs/youtube_summary/result_validation.rs
git commit -m "test: prepare prompt-pack crate fixtures"
```

This is the Checkpoint 4 boundary. Stopping here retains four independently green preparation checkpoints.

### Task 8: Checkpoint 5 — Add the Intentional RED Boundary Contract

**Files:**

- Create: `src/lib/prompt-pack-crate-boundary-contract.test.ts`
- Modify: `src/lib/rust-workspace-core-contract.test.ts`
- Modify: `src/lib/gemini-browser-crate-boundary-contract.test.ts`
- Modify: `src/lib/llm-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `src/lib/focused-rust-loop-contract.test.ts`
- Modify: `src/lib/development-loop-performance-contract.test.ts`
- Modify: `src/lib/prompt-pack-completion-transport-contract.test.ts`
- Modify: `src/lib/prompt-pack-run-control-contract.test.ts`
- Modify: `src/lib/prompt-pack-run-store-contract.test.ts`
- Modify: `src/lib/prompt-pack-runtime-config-contract.test.ts`
- Modify: `src/lib/prompt-pack-stage-execution-contract.test.ts`
- Modify: `src/lib/prompt-pack-stage-request-policy-contract.test.ts`

**Interfaces:**

- Consumes: frozen file/API/dependency/schema/asset/test maps and the dual-state path helper.
- Produces: one fail-closed contract that is RED only for the absent future member/path/move.

- [ ] **Step 1: Implement exact contract cases**

Use these exact titles:

```text
declares one app edge and the exact locked dependency surface
keeps a curated crate API and private explicit app facade
moves every frozen baseline identity to its approved 223/2 owner exactly once
rejects disabled renamed or copied legacy prompt-pack tests
keeps production SQL and app-only integrations in their approved owners
pins the source browser event and execution-ticket handoffs
centralizes all eight canonical bundled assets
keeps the crate-private schema fixture in exact ordered parity with the registered non-Apalis migration prefix
```

The tests assert the complete frozen maps, not counts or prefixes alone. Scan production crate `.rs` files excluding `#[cfg(test)]` regions for forbidden Tauri/app imports and foreign table tokens. Reject `pub mod`, `pub use *`, public test helpers, reverse lower-crate edges, source/checksum on the path package, and registry-version churn.

- [ ] **Step 2: Parse Appendix A and enforce executable 223/2 ownership**

Parse every `#### \`file.rs\` (N)` heading and following bullet until the next heading in the approved spec. Resolve each logical pair against Rust source declarations and final module paths. Assert 225 unique logical pairs, 223 crate owners, and the two exact app owners. Fail on `#[cfg(any())]`, known false cfg sentinels, commented `#[test]` blocks, `legacy_disabled_*`, renamed substitutes, duplicates, or a leaf-only identity model.

- [ ] **Step 3: Implement the standing migration parity parser**

The parser must fail closed in this order:

```text
1. Slice build_migrations() from `let mut migrations = vec![` through its closing `];`
   immediately before `migrations.extend(apalis_sqlite_migrations())`.
2. Extract the ordered migration function calls; require exactly 12.
3. Resolve each `fn <name>() -> Migration` to its `sql: <CONST>` token.
4. Resolve each constant to one `include_str!("../migrations/...")`.
5. Parse the fixture's explicit ordered `(repository path, include_str!(...))` pairs and `[...; 12]` length.
6. Resolve include paths relative to their Rust files and normalize to repository-relative POSIX paths.
7. Require exact ordered equality, 12 unique existing files, repository path == include target, and no Apalis entry.
```

Any unmatched function, constant, include, array shape, or registry shape is a test failure.

- [ ] **Step 4: Update the existing contracts without weakening them**

Update the workspace/member/owner/path portions of the workspace, focused-loop, performance, and shell-cap contracts with an exact two-state branch keyed only by existence of `src-tauri/crates/extractum-prompt-packs/Cargo.toml`: before extraction they require the old exact member/owner list; after extraction they require the new exact member/owner list. This is not a permissive union. Lower-boundary forbidden-edge lists include `extractum-prompt-packs` immediately in both states.

Keep the Phase 6 lifecycle assertion installed by Task 1 independent from that manifest branch: it parses the exact roadmap status and accepts only the seven declared values. Do not infer lifecycle state from `Cargo.toml`; after the mechanical move the manifest exists while the last retained roadmap state truthfully remains `preparation Checkpoint 4 retained` until completion evidence changes it to `done: retained`. Preserve every non-status Phase 6 assertion unchanged.

Keep the six Prompt Pack raw-path contracts dual-state through `prompt-pack-contract-paths.ts`; they must be GREEN before and after the move. Inventory `rg -n "src-tauri[/\\\\]src[/\\\\]prompt_packs|prompt_packs/" src/lib -g "*.test.ts"` and add any newly discovered path consumer to this task.

- [ ] **Step 5: Prove the dedicated contract is RED for the intended reason only**

```powershell
$boundaryRed = @(npm.cmd run test -- src/lib/prompt-pack-crate-boundary-contract.test.ts 2>&1)
$boundaryExit = $LASTEXITCODE
$boundaryRed | Out-Host
if ($boundaryExit -eq 0) { throw 'Expected boundary RED' }
if (($boundaryRed -join "`n") -match 'No test files found|0 tests') { throw 'Contract did not execute' }
if (($boundaryRed -join "`n") -notmatch 'extractum-prompt-packs') { throw 'Unexpected RED reason' }
```

Then prove the app and all existing contracts remain green:

```powershell
npm.cmd run test -- src/lib/prompt-pack-application-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/focused-rust-loop-contract.test.ts src/lib/development-loop-performance-contract.test.ts src/lib/prompt-pack-completion-transport-contract.test.ts src/lib/prompt-pack-run-control-contract.test.ts src/lib/prompt-pack-run-store-contract.test.ts src/lib/prompt-pack-runtime-config-contract.test.ts src/lib/prompt-pack-stage-execution-contract.test.ts src/lib/prompt-pack-stage-request-policy-contract.test.ts
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

- [ ] **Step 6: Commit the intentional RED separately**

```powershell
git diff --check
git add src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/focused-rust-loop-contract.test.ts src/lib/development-loop-performance-contract.test.ts src/lib/prompt-pack-completion-transport-contract.test.ts src/lib/prompt-pack-run-control-contract.test.ts src/lib/prompt-pack-run-store-contract.test.ts src/lib/prompt-pack-runtime-config-contract.test.ts src/lib/prompt-pack-stage-execution-contract.test.ts src/lib/prompt-pack-stage-request-policy-contract.test.ts
git commit -m "test: define prompt-pack crate boundary"
```

Do not leave any other failing test in this commit. If work stops before extraction, revert only this commit with ordinary `git revert`.

### Task 9: Capture the Minimal Advisory Baseline

**Files:**

- Temporarily modify and restore: `src-tauri/src/prompt_packs/stage_output_normalization.rs`
- Temporary evidence only: `%TEMP%\extractum-phase6-prompt-pack-timing.csv`
- Temporary hash only: `%TEMP%\extractum-phase6-prompt-pack-baseline.sha256`

**Interfaces:**

- Consumes: clean committed Checkpoint 5 state and the app-owned portable normalization module.
- Produces: one complete three-sample baseline median or the literal disposition `incomplete / no performance conclusion`; no repository change.

- [ ] **Step 1: Prove clean source identity and initialize scratch evidence**

```powershell
$baselineProbe = 'src-tauri/src/prompt_packs/stage_output_normalization.rs'
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) { throw 'Timing requires a clean worktree' }
(Get-FileHash -Algorithm SHA256 -LiteralPath $baselineProbe).Hash.ToLowerInvariant() |
  Set-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-baseline.sha256"
'state,sample,milliseconds' |
  Set-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-timing.csv"
```

- [ ] **Step 2: Add marker A with `apply_patch` and discard one warm-up**

```diff
*** Begin Patch
*** Update File: src-tauri/src/prompt_packs/stage_output_normalization.rs
@@
+// extractum-prompt-packs timing probe a
*** End Patch
```

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Baseline warm-up failed' }
```

- [ ] **Step 3: Toggle to marker B and record baseline sample 1**

```diff
*** Begin Patch
*** Update File: src-tauri/src/prompt_packs/stage_output_normalization.rs
@@
-// extractum-prompt-packs timing probe a
+// extractum-prompt-packs timing probe b
*** End Patch
```

```powershell
$timingWatch = [System.Diagnostics.Stopwatch]::StartNew()
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
$timingWatch.Stop()
if ($LASTEXITCODE -ne 0) { throw 'Baseline sample 1 failed' }
"baseline,1,$($timingWatch.ElapsedMilliseconds)" | Add-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-timing.csv"
```

- [ ] **Step 4: Toggle B → A and record baseline sample 2**

Apply the inverse one-line patch, then run:

```powershell
$timingWatch = [System.Diagnostics.Stopwatch]::StartNew()
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
$timingWatch.Stop()
if ($LASTEXITCODE -ne 0) { throw 'Baseline sample 2 failed' }
"baseline,2,$($timingWatch.ElapsedMilliseconds)" | Add-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-timing.csv"
```

- [ ] **Step 5: Toggle A → B and record baseline sample 3**

Apply the Step 3 one-line patch, then run:

```powershell
$timingWatch = [System.Diagnostics.Stopwatch]::StartNew()
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
$timingWatch.Stop()
if ($LASTEXITCODE -ne 0) { throw 'Baseline sample 3 failed' }
"baseline,3,$($timingWatch.ElapsedMilliseconds)" | Add-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-timing.csv"
```

- [ ] **Step 6: Remove the marker and prove byte restoration**

```diff
*** Begin Patch
*** Update File: src-tauri/src/prompt_packs/stage_output_normalization.rs
@@
-// extractum-prompt-packs timing probe b
*** End Patch
```

```powershell
$expectedHash = (Get-Content -Raw "$env:TEMP\extractum-phase6-prompt-pack-baseline.sha256").Trim()
$actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/prompt_packs/stage_output_normalization.rs').Hash.ToLowerInvariant()
if ($actualHash -ne $expectedHash) { throw 'Baseline probe bytes were not restored' }
git diff --exit-code -- src-tauri/src/prompt_packs/stage_output_normalization.rs
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) { throw 'Baseline series left repository changes' }
$baselineValues = @(Import-Csv "$env:TEMP\extractum-phase6-prompt-pack-timing.csv" |
  Where-Object state -eq 'baseline' | ForEach-Object { [int64]$_.milliseconds } | Sort-Object)
if ($baselineValues.Count -ne 3) { throw 'Baseline is incomplete' }
"baseline median: $($baselineValues[1]) ms"
```

If any warm-up/sample/edit fails, remove whichever marker is present with `apply_patch`, prove the original hash and clean status, replace the baseline rows with `baseline,incomplete,no performance conclusion`, and do not retry.

### Task 10: Perform the Mechanical Crate Extraction

**Files:**

- Create: `src-tauri/crates/extractum-prompt-packs/Cargo.toml`
- Create: `src-tauri/crates/extractum-prompt-packs/src/lib.rs`
- Move: every crate-owned file in the frozen file map
- Retain/modify: every application-owned file in the frozen file map
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `package.json`
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Modify: all contract files from Task 8 as required by their prepared owner paths

**Interfaces:**

- Consumes: frozen prepared seams and one intentional RED contract.
- Produces: one workspace member, one app edge, a private curated crate root, a private app facade, and GREEN focused/package/source contracts.

- [ ] **Step 1: Reconfirm the frozen pre-move state**

```powershell
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) { throw 'Mechanical move requires a clean worktree' }
npm.cmd run test -- src/lib/prompt-pack-crate-boundary-contract.test.ts
if ($LASTEXITCODE -eq 0) { throw 'Boundary contract must still be RED before the move' }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Prepared app is not green' }
```

- [ ] **Step 2: Create the manifest and private crate root**

Create the exact manifest from the frozen contract. In `lib.rs`, declare every moved module with `mod`, gate only `test_schema` with `#[cfg(test)]`, and write one explicit `pub use` list containing exactly the curated root allowlist. Do not declare `pub mod` or use a glob.

- [ ] **Step 3: Move whole root modules with Git-aware moves**

```powershell
$crateRoot = 'src-tauri/crates/extractum-prompt-packs/src'
New-Item -ItemType Directory -Force -Path $crateRoot | Out-Null
$rootModules = @(
  'assets.rs','browser_port.rs','completion_transport.rs','dto.rs','events.rs',
  'gemini_browser_stage.rs','json_repair.rs','library.rs','models.rs','projections.rs',
  'result_builder.rs','result_service.rs','run_control.rs','run_store.rs','runtime.rs',
  'runtime_config.rs','seed.rs','source_port.rs','stage_execution.rs','stage_io.rs',
  'stage_output_normalization.rs','stage_request_policy.rs','store.rs','test_schema.rs','validation.rs'
)
foreach ($module in $rootModules) {
  git mv "src-tauri/src/prompt_packs/$module" "$crateRoot/$module"
  if ($LASTEXITCODE -ne 0) { throw "Failed to move $module" }
}
```

The directory target is exact and workspace-local. Stop immediately if any named source is absent; do not broaden the move with a wildcard.

- [ ] **Step 4: Move YouTube production/tests and complete the prepared split**

```powershell
$youtubeTarget = 'src-tauri/crates/extractum-prompt-packs/src/youtube_summary'
New-Item -ItemType Directory -Force -Path $youtubeTarget | Out-Null
$youtubeWhole = @(
  'entities.rs','entities_tests.rs','execution.rs','execution_result.rs','execution_tests.rs',
  'facade_tests.rs','gem_analysis.rs','mod.rs','outputs.rs','outputs_tests.rs','preflight.rs',
  'preflight_tests.rs','progress.rs','result_validation.rs','snapshots.rs','store.rs',
  'synthesis_execution.rs','synthesis_input.rs','synthesis_input_tests.rs','tail_stages.rs',
  'test_support.rs','transcript_execution.rs','types.rs'
)
foreach ($module in $youtubeWhole) {
  git mv "src-tauri/src/prompt_packs/youtube_summary/$module" "$youtubeTarget/$module"
  if ($LASTEXITCODE -ne 0) { throw "Failed to move youtube_summary/$module" }
}
git mv src-tauri/src/prompt_packs/youtube_summary/domain_snapshots_tests.rs "$youtubeTarget/snapshots_tests.rs"
git mv src-tauri/src/prompt_packs/youtube_summary/app_test_support.rs src-tauri/src/prompt_packs/youtube_summary/test_support.rs
```

Use `apply_patch` to remove `include!("domain_snapshots_tests.rs")` from the retained app `snapshots_tests.rs` and change its fixture import from `super::app_test_support` to `super::test_support`. Create the new app test-only `youtube_summary/mod.rs` with exactly:

```rust
#[cfg(test)]
mod snapshots_tests;
#[cfg(test)]
mod test_support;
```

- [ ] **Step 5: Rewire imports, asset/fixture prefixes, and the app facade**

Use `extractum_core`, `extractum_llm`, and `extractum_gemini_browser` paths inside the moved crate. Change only the centralized asset prefix from `/prompt-packs/...` to `/../../prompt-packs/...` and test fixture includes from `../../migrations/...` to `../../../migrations/...`.

The retained app `mod.rs` declares the seven adapter/wrapper modules plus the test-only YouTube module and re-exports the same functions/state consumed by `src-tauri/src/lib.rs`:

```rust
mod browser_adapter;
mod event_adapter;
mod library_command;
mod result_commands;
mod runtime_commands;
mod seed_command;
mod source_adapter;
#[cfg(test)]
mod youtube_summary;

pub use extractum_prompt_packs::PromptPackRunState;
pub use library_command::get_prompt_pack_library;
pub use result_commands::{get_prompt_pack_result, get_prompt_pack_stage_artifact,
    get_prompt_pack_validation_findings, list_prompt_pack_audit_events,
    list_prompt_pack_stage_artifacts};
pub use runtime_commands::{cancel_prompt_pack_run, cleanup_interrupted_prompt_pack_runs,
    delete_prompt_pack_run, list_active_prompt_pack_runs, list_prompt_pack_run_stages,
    list_prompt_pack_runs, preflight_youtube_summary_run, start_youtube_summary_run,
    update_prompt_pack_run};
#[cfg(dev)]
pub use runtime_commands::{clear_prompt_pack_cancellation_smoke_fixture,
    seed_prompt_pack_cancellation_smoke_fixture};
pub use seed_command::seed_builtin_prompt_packs;
```

Do not edit `src-tauri/src/lib.rs` unless the application contract exposes a pre-existing path mistake; such a finding stops the mechanical move for a spec correction.

- [ ] **Step 6: Apply exact manifest inheritance and update the lockfile**

Add the member/app edge, move `sha2` and `sqlx` declarations to `[workspace.dependencies]`, set both app uses to `{ workspace = true }`, and transfer `jsonschema` to the new crate. Change `package.json`:

```json
"test:rust:prompt-pack-runs": "cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib prompt_pack_run"
```

Update `Cargo.lock` through Cargo, then validate rather than hand-editing registry packages:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Locked workspace metadata failed' }
```

- [ ] **Step 7: Turn the source-boundary contract GREEN**

```powershell
npm.cmd run test -- src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/prompt-pack-application-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/focused-rust-loop-contract.test.ts src/lib/development-loop-performance-contract.test.ts src/lib/prompt-pack-completion-transport-contract.test.ts src/lib/prompt-pack-run-control-contract.test.ts src/lib/prompt-pack-run-store-contract.test.ts src/lib/prompt-pack-runtime-config-contract.test.ts src/lib/prompt-pack-stage-execution-contract.test.ts src/lib/prompt-pack-stage-request-policy-contract.test.ts
```

Expected: every listed contract passes, including exact migration parity, one app edge, 223/2 ownership, no duplicate legacy tests, exact lock roots, and no forbidden production dependency/token.

- [ ] **Step 8: Run focused crate and immediate-consumer gates**

```powershell
Invoke-ExactRust extractum-prompt-packs 'dto::tests::start_outcomes_serialize_exact_ipc_contract'
Invoke-ExactRust extractum-prompt-packs 'youtube_summary::snapshots_tests::runnable_start_uses_complete_fresh_source_read_sequence'
Invoke-ExactRust extractum-prompt-packs 'runtime::tests::prompt_pack_browser_stage_cancelled_while_active_stops_sidecar'
Invoke-ExactRust extractum-prompt-packs 'runtime::tests::terminal_event_removes_run_from_active_state'
Invoke-ExactRust extractum-prompt-packs 'completion_transport::tests::api_stage_uses_background_scheduler_prompt_pack_metadata_and_typed_cancellation'
Invoke-ExactRust extractum-prompt-packs 'runtime::tests::persist_browser_stage_provenance_records_result_identity'
Invoke-ExactRust extractum-prompt-packs 'projections::tests::low_level_result_persistence_rolls_back_when_projection_insert_fails'
Invoke-ExactRust extractum-prompt-packs 'validation::tests::synthesis_output_validator_rejects_backend_owned_ids'
Invoke-ExactRust extractum-prompt-packs 'youtube_summary::result_validation::tests::validation_wrapper_rolls_back_result_findings_when_persistence_fails_after_validation'
Invoke-ExactRust extractum-prompt-packs 'youtube_summary::execution_tests::execute_queued_run_with_stage_executor_finishes_complete'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled'
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

- [ ] **Step 9: Review moved-not-copied evidence and commit the extraction**

```powershell
git diff --check
git diff --summary
rg -n "tauri::|AppHandle|State<'|Emitter|Manager|get_pool|crate::migrations|crate::sources" src-tauri/crates/extractum-prompt-packs/src -g "*.rs"
rg -n "FROM (sources|youtube_video_sources|youtube_playlist_items|youtube_transcript_segments|items)|JOIN (sources|youtube_video_sources|youtube_playlist_items|youtube_transcript_segments|items)" src-tauri/crates/extractum-prompt-packs/src -g "*.rs"
git status --short
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/crates/extractum-prompt-packs src-tauri/src/prompt_packs package.json src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/prompt-pack-application-contract.test.ts src/lib/prompt-pack-contract-paths.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/focused-rust-loop-contract.test.ts src/lib/development-loop-performance-contract.test.ts src/lib/prompt-pack-completion-transport-contract.test.ts src/lib/prompt-pack-run-control-contract.test.ts src/lib/prompt-pack-run-store-contract.test.ts src/lib/prompt-pack-runtime-config-contract.test.ts src/lib/prompt-pack-stage-execution-contract.test.ts src/lib/prompt-pack-stage-request-policy-contract.test.ts
git commit -m "refactor: extract prompt-pack domain crate"
```

Expected: both `rg` scans return no production violation; reviewed rename/split summary accounts for every frozen file; staging contains only the named extraction/contracts/manifests.

### Task 11: Capture Candidate Timing and Run Completion Gates

**Files:**

- Temporarily modify and restore: `src-tauri/crates/extractum-prompt-packs/src/stage_output_normalization.rs`
- Append temporary evidence: `%TEMP%\extractum-phase6-prompt-pack-timing.csv`
- Modify only if a mechanical wiring defect is found: files in the Task 10 extraction commit

**Interfaces:**

- Consumes: clean committed extraction and the complete baseline scratch result.
- Produces: advisory candidate result, exact 223/2 evidence, and all Rust/repository completion results.

- [ ] **Step 1: Prove the candidate probe starts clean and record its hash**

```powershell
$candidateProbe = 'src-tauri/crates/extractum-prompt-packs/src/stage_output_normalization.rs'
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) { throw 'Candidate timing requires a clean worktree' }
$baselineRows = @(Import-Csv "$env:TEMP\extractum-phase6-prompt-pack-timing.csv" |
  Where-Object { $_.state -eq 'baseline' -and $_.sample -match '^\d+$' -and $_.milliseconds -match '^\d+$' })
if ($baselineRows.Count -ne 3) {
  Write-Host 'Baseline incomplete; carry forward incomplete / no performance conclusion and skip candidate Steps 2-4'
}
(Get-FileHash -Algorithm SHA256 -LiteralPath $candidateProbe).Hash.ToLowerInvariant() |
  Set-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-candidate.sha256"
```

If the baseline row count is not three, do not add a candidate marker; continue at Step 5.

- [ ] **Step 2: Add marker A and discard one candidate warm-up**

```diff
*** Begin Patch
*** Update File: src-tauri/crates/extractum-prompt-packs/src/stage_output_normalization.rs
@@
+// extractum-prompt-packs timing probe a
*** End Patch
```

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Candidate warm-up failed' }
```

- [ ] **Step 3: Record three candidate samples with B/A/B toggles**

For sample 1, change marker `a` to `b` with `apply_patch`, then run:

```powershell
$timingWatch = [System.Diagnostics.Stopwatch]::StartNew()
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
$timingWatch.Stop()
if ($LASTEXITCODE -ne 0) { throw 'Candidate sample 1 failed' }
"candidate,1,$($timingWatch.ElapsedMilliseconds)" | Add-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-timing.csv"
```

For sample 2, change marker `b` to `a` with `apply_patch`, then run the same command with failure text `Candidate sample 2 failed` and append `candidate,2,<milliseconds>`. For sample 3, change marker `a` to `b`, run it with failure text `Candidate sample 3 failed`, and append `candidate,3,<milliseconds>`.

- [ ] **Step 4: Remove the candidate marker and record the advisory result**

Remove the exact marker line with `apply_patch`, then run:

```powershell
$expectedHash = (Get-Content -Raw "$env:TEMP\extractum-phase6-prompt-pack-candidate.sha256").Trim()
$actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/crates/extractum-prompt-packs/src/stage_output_normalization.rs').Hash.ToLowerInvariant()
if ($actualHash -ne $expectedHash) { throw 'Candidate probe bytes were not restored' }
git diff --exit-code -- src-tauri/crates/extractum-prompt-packs/src/stage_output_normalization.rs
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) { throw 'Candidate series left repository changes' }
$timingRows = Import-Csv "$env:TEMP\extractum-phase6-prompt-pack-timing.csv"
$baselineValues = @($timingRows | Where-Object { $_.state -eq 'baseline' -and $_.sample -match '^\d+$' -and $_.milliseconds -match '^\d+$' } | ForEach-Object { [int64]$_.milliseconds } | Sort-Object)
$candidateValues = @($timingRows | Where-Object { $_.state -eq 'candidate' -and $_.sample -match '^\d+$' -and $_.milliseconds -match '^\d+$' } | ForEach-Object { [int64]$_.milliseconds } | Sort-Object)
if ($baselineValues.Count -eq 3 -and $candidateValues.Count -eq 3) {
  $baselineMedian = $baselineValues[1]
  $candidateMedian = $candidateValues[1]
  "baseline=$baselineMedian candidate=$candidateMedian delta=$($candidateMedian-$baselineMedian) ms"
} else {
  'incomplete / no performance conclusion'
}
```

Record raw values and medians in the final verification document. Do not compare them to a retention threshold. If any candidate step fails, restore/hash-check/clean-check exactly as above, append `candidate,incomplete,no performance conclusion` to the scratch CSV, record that exact disposition, and do not retry.

- [ ] **Step 5: Prove the exact baseline partition and package ownership**

```powershell
npm.cmd run test -- src/lib/prompt-pack-crate-boundary-contract.test.ts
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer'
Invoke-ExactRust extractum 'prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled'
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: the contract reports 223/2 for Appendix A, every crate and app package test passes, and no frozen identity is disabled, copied, or renamed.

- [ ] **Step 6: Run rustfmt and time the single mandatory workspace check**

```powershell
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { throw 'rustfmt gate failed' }
$workspaceWatch = [System.Diagnostics.Stopwatch]::StartNew()
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
$workspaceWatch.Stop()
if ($LASTEXITCODE -ne 0) { throw 'workspace check failed' }
"workspace_check,1,$($workspaceWatch.ElapsedMilliseconds)" | Add-Content -LiteralPath "$env:TEMP\extractum-phase6-prompt-pack-timing.csv"
```

This is the only ordinary workspace-check timing result. Do not rerun it for measurement; a later correctness rerun after a real fix is a gate rerun and must be recorded separately as such.

- [ ] **Step 7: Run the remaining full gates**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

Expected: both commands pass. If a failure identifies a behavioral change, stop and follow the rollback ladder. A purely mechanical import/visibility defect may be corrected within the extraction slice, then all affected focused/package/full gates are rerun and the fix is committed as `fix: complete prompt-pack crate wiring`.

### Task 12: Prove Release Startup and Record the Retained Result

**Files:**

- Create: `docs/superpowers/verification/2026-07-20-extractum-prompt-packs-extraction.md`
- Modify: `docs/superpowers/specs/2026-07-20-prompt-packs-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `docs/value-registry.md`
- Modify: `docs/project.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`

**Interfaces:**

- Consumes: all correctness, timing, release, startup, manifest, and ownership evidence.
- Produces: durable Phase 6 verification, truthful roadmap status, updated ownership docs, and a clean documentation commit.

- [ ] **Step 1: Build the release executable without MSI/WiX**

```powershell
npm.cmd run tauri -- build --no-bundle
if ($LASTEXITCODE -ne 0) { throw 'Release no-bundle build failed' }
$releaseExe = (Resolve-Path 'src-tauri/target/release/extractum.exe').Path
if (-not (Test-Path -LiteralPath $releaseExe -PathType Leaf)) { throw 'Release executable is missing' }
```

Expected: `extractum.exe` exists. Do not run an installer build.

- [ ] **Step 2: Run the bounded exact-PID startup smoke**

```powershell
$existingExtractum = @(Get-Process -Name extractum -ErrorAction SilentlyContinue)
if ($existingExtractum.Count -ne 0) { throw '[infrastructure] pre-existing extractum process blocks startup smoke' }
$smokeProcess = $null
try {
  try {
    $smokeProcess = Start-Process -FilePath $releaseExe -PassThru -WindowStyle Hidden
  } catch {
    throw "[infrastructure] failed to launch release executable: $($_.Exception.Message)"
  }
  Start-Sleep -Seconds 5
  $smokeProcess.Refresh()
  if ($smokeProcess.HasExited) {
    throw "[completion] application exited early with code $($smokeProcess.ExitCode)"
  }
  Write-Host "startup smoke survived 5 seconds; pid=$($smokeProcess.Id)"
} finally {
  if ($null -ne $smokeProcess) {
    $smokeProcess.Refresh()
    if (-not $smokeProcess.HasExited) {
      Stop-Process -Id $smokeProcess.Id -Force
      Wait-Process -Id $smokeProcess.Id -Timeout 10 -ErrorAction Stop
    }
  }
}
$residue = @(Get-Process -Name extractum -ErrorAction SilentlyContinue)
if ($residue.Count -ne 0) { throw '[infrastructure] startup smoke left process residue' }
```

Only a confirmed early application exit is a completion failure. Launch, inspection, or cleanup failure is infrastructure and stops for investigation without reclassifying the candidate. Do not issue a live provider request or mutate account data.

- [ ] **Step 3: Write the durable verification document**

Record, with exact hashes/commands/raw values:

```text
start and candidate commit identities
Checkpoint 1–4 green commit identities
Checkpoint 5 RED commit and intended failure text
mechanical extraction commit identity
46-file prepared inventory and final app/crate ownership
Appendix A result: 223 crate / 2 app / 225 total
new characterization test counts separately from the baseline
manifest and Cargo.lock package/edge/version proof
32-table production allowlist and five forbidden foreign-table scan result
12-entry migration registry/fixture ordered parity result
eight asset paths, full SHA-384 values, and unchanged bundled_source_path
focused crate/app/package/workspace/repository command outcomes
baseline and candidate raw milliseconds/medians, or exact incomplete disposition
single ordinary workspace-check duration
release no-bundle result and exact-PID five-second startup result
manual moved-not-copied/rename review result
```

Do not infer a causal performance conclusion from the advisory numbers.

- [ ] **Step 4: Update status and ownership documentation**

Set the Phase 6 spec status to implemented/retained and link the verification document. Change the roadmap heading from `preparation Checkpoint 4 retained` to the exact terminal status `done: retained`; do not use that status before every completion gate above passes. Update the roadmap with the retained crate, actual ordinary workspace duration, advisory timing disposition, and Phase 7 only as a future fresh JIT design. State mechanically that Phase 5 was `10,410 ms`, below `15,000 ms`, so even a Phase 6 ordinary result at or above `15,000 ms` is only the first possible member of an adjacent pair and does not trigger an investigation by itself. Update `docs/project.md`, `docs/value-registry.md`, and the shell-cap contract for ownership/path changes only; preserve the lifecycle vocabulary and do not add or rename registry values.

- [ ] **Step 5: Validate documentation/contracts and commit**

```powershell
npm.cmd run test -- src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/prompt-pack-application-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/focused-rust-loop-contract.test.ts src/lib/development-loop-performance-contract.test.ts
git diff --check
git status --short
git add docs/superpowers/verification/2026-07-20-extractum-prompt-packs-extraction.md docs/superpowers/specs/2026-07-20-prompt-packs-crate-boundary-design.md docs/superpowers/specs/2026-07-17-crate-roadmap.md docs/value-registry.md docs/project.md src/lib/crate-extraction-shell-cap-contract.test.ts
git commit -m "docs: record retained prompt-pack extraction"
git status --short
```

Expected: focused contracts pass and final status is clean.

## Pause and Rollback Ladder

1. Before a checkpoint reaches its green boundary commit, correct or abandon only that in-progress work. Do not rewrite earlier checkpoint history.
2. Every Checkpoint 1, 2, 3, or 4 green boundary commit already contains its exact `preparation Checkpoint N retained` roadmap status and a GREEN shell-cap contract. If the owner pauses there, write a short verification disposition that confirms that existing status; do not create a different lifecycle transition merely because execution paused.
3. If a retained preparation checkpoint is not independently useful, revert it and later dependent preparation commits in reverse order using ordinary `git revert`; preserve earlier green commits. The roadmap then returns mechanically to the last retained checkpoint state. Use `not retained` only in a durable owner closure when no preparation checkpoint remains the truthful retained state; if reverting Checkpoint 1 also removes the standing lifecycle assertion, the closure commit must restore that exact seven-value assertion before recording `not retained`.
4. If work stops after Checkpoint 5 but before extraction, revert only `test: define prompt-pack crate boundary` and keep Checkpoints 1–4.
5. If the extraction or a completion gate fails and cannot be corrected as a purely mechanical wiring defect, stage only the exact candidate/evidence paths, commit them as `refactor: preserve failed prompt-pack extraction`, and write the verification disposition. Then use ordinary `git revert <failed-candidate-commit>` followed by `git revert <checkpoint-5-red-commit>`; decide each earlier green checkpoint independently. If the candidate already has the Task 10 extraction commit plus a later fix commit, revert the fix first, the extraction second, and the RED contract third.
6. Timing interruption is not a correctness failure: remove the marker, prove SHA-256 restoration and clean status, record `incomplete / no performance conclusion`, and continue only after restoration.
7. No rollback path authorizes `git reset`, forced branch deletion, destructive checkout, manual evidence deletion, or overwriting unrelated work.

## Final Manual Review

- Compare every prepared source/test item against the frozen file map; account for each rename and both deliberate split files.
- Confirm the app contains only the seven integration/wrapper modules plus its two-test module.
- Confirm the crate exposes no public module/glob/test helper and every widened symbol appears in the visibility allowlist.
- Confirm the source adapter, Browser adapter, event adapter, and spawned credential resolver are the only upward integrations.
- Confirm the exact repeated source-read trace, fixed preflight budgets, queued-event-before-spawn order, and profile-resolution-after-spawn order remain characterized.
- Confirm all 225 Appendix A identities occur once in the correct owner, and new tests are not counted as baseline replacements.
- Confirm no migration, frontend contract, asset byte, persisted string, event message, error JSON, or `src-tauri/src/lib.rs` consumer path changed.
