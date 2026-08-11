# Grammers 0.10 Upgrade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade all four Extractum grammers dependencies to Codeberg revision `5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` (`0.10.0`) without changing product behavior or the public `extractum-telegram` API.

**Architecture:** Keep the migration inside the existing session, live, avatar, and Takeout adapters. Propagate the new fallible session operations through internal `AppResult` boundaries, keep avatar loading best-effort, make export-DC fallback exhaustive, and let compilation identify layer-227 fixture additions.

**Tech Stack:** Rust 2024, Cargo workspace, `grammers-client`, `grammers-mtsender`, `grammers-session`, `grammers-tl-types`, Tokio tests, Vitest repository rules, Node.js baseline generator.

## Global Constraints

- Update `grammers-client`, `grammers-mtsender`, `grammers-session`, and `grammers-tl-types` together to `5c6d44ff30e02d6c9295bcf1fcb51403ad77c981`.
- Do not add product support for rich messages, guard bots, or other layer-227 capabilities.
- Stop and re-plan if compilation requires changes outside the existing session, live, avatar, or Takeout adapters.
- Session access is fail-closed; avatar loading retains its existing best-effort outer boundary.
- `InvocationError::Session` is an explicit non-fallback export-DC error.
- Preserve the public `extractum-telegram` API and the encrypted session JSON schema.
- Preserve user-owned working-tree changes. Several target files are already modified; do not revert them and do not stage or commit an entire overlapping file without reviewing its pre-existing diff.
- Baseline JSON is generated. Edit only the revision constant, run the generator, accept `universe`/`forbidden` drift as upstream fact, and stop for review if `required` changes.
- Use `npm.cmd`, not `npm`, on Windows.

## File Structure

- `src-tauri/Cargo.toml`: owns the four coordinated Git pins.
- `src-tauri/Cargo.lock`: records the resolved `0.10.0` dependency graph.
- `src-tauri/crates/extractum-telegram/src/session.rs`: converts fallible `MemorySession` reads/writes into internal `AppResult` values.
- `src-tauri/crates/extractum-telegram/src/live/messages.rs`: propagates peer-cache failures and updates session assertions in tests.
- `src-tauri/crates/extractum-telegram/src/live/avatar.rs`: adapts fallible `Peer::photo` while retaining best-effort suppression.
- `src-tauri/crates/extractum-telegram/src/error.rs`: makes export-DC fallback classification exhaustive.
- `src-tauri/crates/extractum-telegram/src/takeout/export_dc.rs`: preserves missing-DC diagnostics, propagates session failures, and tests the new `Session` variant.
- `src-tauri/crates/extractum-telegram/src/live/topics.rs`, `live/messages.rs`, `takeout/operations.rs`, `takeout/pagination.rs`, and `takeout/raw_parse.rs`: update raw layer-227 test fixtures with neutral values.
- `scripts/telegram-grammers-feature-baseline.mjs`: owns the baseline revision constant.
- `src/lib/telegram-grammers-feature-baseline.json`: generated feature graph.
- `scripts/testing/repository-rules.test.ts`: owns the synthetic repository-rule revision fixture.
- `docs/project.md`: records the active version, revision, reason, and sanitized validation evidence.
- `docs/superpowers/verification/2026-08-11-grammers-0-10-upgrade.md`: records automated and live validation evidence produced by this slice.

## Rust Verification Loops

**Affected package:** `extractum-telegram`. The root application consumes it but its public interface does not change, so no immediate dependent-package checkpoint is required before the end-of-slice workspace gate.

**Narrow RED/GREEN tests:**

- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib session::tests::saving_session_writes_encrypted_envelope_not_plaintext -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule -- --exact`

**Focused checks:**

- Compiler RED/GREEN: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib`
- Fixture RED/GREEN: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`

**Package checkpoints:**

- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`
- `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`

**End-of-slice workspace gate:** `npm.cmd run verify`.

---

### Task 1: Establish the 0.10 Compiler RED

**Files:**
- Modify: `src-tauri/Cargo.toml:12-15`
- Modify: `src-tauri/Cargo.lock`

**Interfaces:**
- Consumes: the current coordinated `0.9.0` Git dependency line.
- Produces: one coordinated `0.10.0` dependency graph for all later tasks.

- [ ] **Step 1: Replace all four revisions**

Use the same full revision in every dependency while preserving the current feature declarations:

```toml
grammers-client = { git = "https://codeberg.org/Lonami/grammers", rev = "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981", default-features = false }
grammers-mtsender = { git = "https://codeberg.org/Lonami/grammers", rev = "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981" }
grammers-session = { git = "https://codeberg.org/Lonami/grammers", rev = "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981", default-features = false, features = ["serde"] }
grammers-tl-types = { git = "https://codeberg.org/Lonami/grammers", rev = "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981", features = ["deserializable-functions"] }
```

- [ ] **Step 2: Run the production compiler RED**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib
```

Expected: FAIL on fallible `home_dc_id`, `updates_state`, `dc_option`, `set_dc_option`, and `Peer::photo` calls. The isolated probe found nine production errors across `session.rs`, `live/avatar.rs`, and `takeout/export_dc.rs`. Cargo refreshes `Cargo.lock` to the new Git source while resolving required transitive changes.

- [ ] **Step 3: Review lockfile scope**

Run:

```powershell
git diff -- src-tauri/Cargo.toml src-tauri/Cargo.lock
```

Expected: all `grammers-*` sources use the new revision and versions are `0.10.0`; every other change is required by the new upstream constraints. Do not accept unrelated manual package refreshes.

---

### Task 2: Propagate Fallible Session and Avatar Operations

**Files:**
- Modify: `src-tauri/crates/extractum-telegram/src/session.rs:72-87,202-208,250-254`
- Modify: `src-tauri/crates/extractum-telegram/src/live/messages.rs:344-350`
- Modify: `src-tauri/crates/extractum-telegram/src/live/avatar.rs:31-40`
- Modify: `src-tauri/crates/extractum-telegram/src/takeout/export_dc.rs:22-38`
- Test: existing tests in the same modules

**Interfaces:**
- Consumes: `MemorySession` methods returning `Result<_, MemorySessionError>` and `Peer::photo` returning `Result<Option<ChatPhoto>, InvocationError>`.
- Produces: `memory_session_to_saved(...) -> AppResult<SavedSession>` and `cache_peer_infos(...) -> AppResult<()>`; public interfaces remain unchanged.

- [ ] **Step 1: Adapt the session serialization boundary**

Change the private helper and its public caller to propagate every session read:

```rust
async fn memory_session_to_saved(session: &TelegramSession) -> AppResult<SavedSession> {
    let session = session.clone_memory_session();
    let home_dc = session
        .home_dc_id()
        .map_err(|error| AppError::internal(error.to_string()))?;
    let updates_state = session
        .updates_state()
        .await
        .map_err(|error| AppError::internal(error.to_string()))?;
    let mut dc_options = HashMap::new();
    for dc_id in 1..=5i32 {
        if let Some(dc) = session
            .dc_option(dc_id)
            .map_err(|error| AppError::internal(error.to_string()))?
        {
            dc_options.insert(dc_id, dc);
        }
    }
    Ok(SavedSession {
        home_dc,
        dc_options,
        updates_state,
    })
}
```

In `encode_session_json`, use:

```rust
let saved = memory_session_to_saved(session).await?;
```

Test-only direct calls use `.await.expect("save memory session")`; production code does not unwrap session errors.

- [ ] **Step 2: Make peer caching fail closed**

Change the method and its single caller:

```rust
pub(super) async fn cache_peer_infos(&self, peer_infos: &[PeerInfo]) -> AppResult<()> {
    for peer_info in peer_infos {
        if peer_info.auth().is_some() {
            self.inner
                .cache_peer(peer_info)
                .await
                .map_err(|error| AppError::internal(error.to_string()))?;
        }
    }
    Ok(())
}
```

```rust
session.cache_peer_infos(&peer_infos).await?;
```

- [ ] **Step 3: Preserve avatar best-effort behavior**

Adapt only the inner fallible operation:

```rust
let Some(photo) = peer
    .photo(false)
    .await
    .map_err(|error| AppError::network(error.to_string()))?
else {
    return Ok(None);
};
```

The existing `best_effort_avatar_with_timeout` remains unchanged and continues to suppress both timeout and `AppError`.

- [ ] **Step 4: Preserve the missing-DC diagnostic**

Use separate `Result` and `Option` handling:

```rust
let home_dc_id = session
    .home_dc_id()
    .map_err(|error| AppError::internal(error.to_string()))?;
let export_dc_id = export_dc_id_for_home_dc(home_dc_id);
let mut export_option = session
    .dc_option(home_dc_id)
    .map_err(|error| AppError::internal(error.to_string()))?
    .ok_or_else(|| {
        AppError::internal(format!(
            "Home DC option {home_dc_id} is missing from session"
        ))
    })?;
export_option.id = export_dc_id;
session
    .set_dc_option(&export_option)
    .await
    .map_err(|error| AppError::internal(error.to_string()))?;
```

- [ ] **Step 5: Run the production GREEN checks**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib session::tests::saving_session_writes_encrypted_envelope_not_plaintext -- --exact
```

Expected: PASS with no unused-`Result` warning.

---

### Task 3: Make Export-DC Fallback Exhaustive

**Files:**
- Modify: `src-tauri/crates/extractum-telegram/src/error.rs:14-24`
- Test: `src-tauri/crates/extractum-telegram/src/takeout/export_dc.rs:312-323`

**Interfaces:**
- Consumes: all `grammers_mtsender::InvocationError` variants from `0.10.0`.
- Produces: exhaustive `should_fallback_export_dc_error(&InvocationError) -> bool` classification.

- [ ] **Step 1: Create the compile-time RED**

Replace `matches!` with an exhaustive `match`, initially omitting `InvocationError::Session(_)`:

```rust
match error {
    InvocationError::InvalidDc
    | InvocationError::Io(_)
    | InvocationError::Transport(_)
    | InvocationError::Authentication(_)
    | InvocationError::Dropped => true,
    InvocationError::Rpc(_) | InvocationError::Deserialize(_) => false,
}
```

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib
```

Expected: FAIL with non-exhaustive pattern coverage for `InvocationError::Session(_)`.

- [ ] **Step 2: Add the explicit non-fallback branch and runtime assertion**

Use the complete implementation:

```rust
use grammers_mtsender::InvocationError;

pub(super) fn should_fallback_export_dc_error(error: &InvocationError) -> bool {
    match error {
        InvocationError::InvalidDc
        | InvocationError::Io(_)
        | InvocationError::Transport(_)
        | InvocationError::Authentication(_)
        | InvocationError::Dropped => true,
        InvocationError::Session(_)
        | InvocationError::Rpc(_)
        | InvocationError::Deserialize(_) => false,
    }
}
```

Extend `export_dc_fallback_is_only_for_local_transport_errors`:

```rust
assert!(!should_fallback_export_dc_error(&InvocationError::Session(
    Box::new(std::io::Error::other("session failure")),
)));
```

- [ ] **Step 3: Run the fallback GREEN test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors -- --exact
```

Expected: PASS.

---

### Task 4: Adapt Layer-227 Test Fixtures

**Files:**
- Modify: `src-tauri/crates/extractum-telegram/src/live/messages.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/live/topics.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/session.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/takeout/operations.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/takeout/pagination.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/takeout/raw_parse.rs`

**Interfaces:**
- Consumes: layer-227 generated raw structs and fallible test-facing `Session` methods.
- Produces: semantically unchanged test fixtures that compile against `grammers 0.10.0`.

- [ ] **Step 1: Run the fixture compiler RED**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
```

Expected: FAIL only in tests. The probe found 15 errors: missing neutral raw fields and assertions still treating session `Result` values as direct values.

- [ ] **Step 2: Add neutral values required by the generated structs**

At every compiler-reported raw fixture initializer, add the neutral value reported by the generated type. For the observed layer-227 diff, the additions are:

```rust
rich_message: None,
bot_guard: false,
reply_to_ephemeral: false,
```

Do not read or map these fields into product payloads.

- [ ] **Step 3: Unwrap successful session operations only in tests**

Change test assertions from direct `Result` use to explicit successful setup expectations:

```rust
Session::peer(memory_session.as_ref(), peer_id)
    .await
    .expect("read cached peer")
    .is_some()
```

Use `.expect("read cached peer")` before every existing `.is_some()`/`.is_none()` assertion, and use:

```rust
assert_eq!(
    loaded_memory_session
        .home_dc_id()
        .expect("read loaded home DC"),
    2
);
```

- [ ] **Step 4: Run fixture and behavior GREEN checks**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule -- --exact
```

Expected: PASS with no compile warnings.

---

### Task 5: Regenerate Feature Policy and Update Dependency Records

**Files:**
- Modify: `scripts/telegram-grammers-feature-baseline.mjs:11`
- Regenerate: `src/lib/telegram-grammers-feature-baseline.json`
- Modify: `scripts/testing/repository-rules.test.ts:84`
- Modify: `docs/project.md:201-239`

**Interfaces:**
- Consumes: the resolved Cargo graph from Tasks 1-4.
- Produces: repository enforcement and dependency documentation for the exact `0.10.0` revision.

- [ ] **Step 1: Update the two source revision constants**

Use the full hash in the generator and synthetic test fixture:

```javascript
const revision = "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981";
```

```typescript
revision: "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981",
```

- [ ] **Step 2: Regenerate, do not hand-edit, the JSON baseline**

Run:

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --write
node scripts/telegram-grammers-feature-baseline.mjs --check
```

Expected: both commands exit 0 and the JSON revision is the full target hash.

- [ ] **Step 3: Audit requested-feature policy**

Run:

```powershell
git diff -- src/lib/telegram-grammers-feature-baseline.json
```

Expected: changes to `universe` or `forbidden` reflect upstream feature declarations. The `required` arrays must remain policy-equivalent: client `[]`, mtsender `[]`, session `["serde"]`, and TL types the generated default/deserialization set. If any `required` array changes, stop and investigate the Cargo feature graph before proceeding.

- [ ] **Step 4: Update the dependency policy record**

In `docs/project.md`, replace the active pin with version `0.10.0` and revision `5c6d44ff30e02d6c9295bcf1fcb51403ad77c981`. State that the update introduces fallible session access and TL layer 227, affecting live sync, Takeout, session handling, and source identity adapters. Keep prior `0.9.0` validation evidence as historical evidence; append the new sanitized evidence only after Task 7.

- [ ] **Step 5: Run repository-rule GREEN checks**

Run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.test.ts
node scripts/telegram-grammers-feature-baseline.mjs --check
```

Expected: PASS.

---

### Task 6: Run Rust Package Gates and Workspace Verification

**Files:**
- Inspect: all files modified by Tasks 1-5

**Interfaces:**
- Consumes: the complete migration implementation and metadata.
- Produces: fresh automated completion evidence.

- [ ] **Step 1: Format only the affected Rust package**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -p extractum-telegram
```

Expected: exit 0. Review the diff so formatting does not rewrite unrelated user-owned work unexpectedly.

- [ ] **Step 2: Run the package checkpoint**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
```

Expected: PASS with a non-zero test count.

- [ ] **Step 3: Prove the production surface without default features**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features
```

Expected: PASS.

- [ ] **Step 4: Run the full Rust-slice gate**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS. If it fails in an unrelated dirty area, report the exact failing command and distinguish pre-existing failures from migration failures; do not claim completion.

- [ ] **Step 5: Review the final dependency diff**

Run:

```powershell
git diff --check
git status --short
git diff -- src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/crates/extractum-telegram scripts/telegram-grammers-feature-baseline.mjs src/lib/telegram-grammers-feature-baseline.json scripts/testing/repository-rules.test.ts docs/project.md
```

Expected: no whitespace errors, no source pin left at the old revision in live files, and no product behavior outside the approved adapters.

---

### Task 7: Record Live Telegram and Takeout Evidence

**Files:**
- Create: `docs/superpowers/verification/2026-08-11-grammers-0-10-upgrade.md`
- Modify: `docs/project.md`

**Interfaces:**
- Consumes: a verified application build and existing local Telegram accounts/sources.
- Produces: sanitized runtime evidence required by the dependency policy.

- [ ] **Step 1: Start the MCP-enabled Tauri application**

Run:

```powershell
npm.cmd run tauri dev
```

Use the actual Vite URL printed by the command. Do not use direct `npx tauri dev`.

- [ ] **Step 2: Exercise one live-sync source**

Through the running application, sync an existing non-sensitive Telegram source. Record only source ID/title, inserted/skipped counts, last message ID, terminal state, and warning codes. Confirm source identity remains bound to the expected Telegram peer and account.

- [ ] **Step 3: Exercise one Takeout path**

Run an existing Takeout import or export-DC-backed import far enough to cover session loading, DC alias setup, and at least one message batch. Record only sanitized source ID/title, attempt/fallback state, imported/skipped counts, and warning codes.

- [ ] **Step 4: Write the verification record**

Create the verification document with these exact sections and only observed values:

```markdown
# Grammers 0.10 Upgrade Verification

## Automated Gates

- extractum-telegram package tests: passed
- extractum-telegram feature-off check: passed
- workspace verify: passed

## Live Sync

Record the sanitized source identity, counters, last-message state, and warnings observed in Step 2.

## Takeout

Record the sanitized attempt/fallback state, counters, and warnings observed in Step 3.

## Sensitive Data

No Telegram session, API hash, credential, access hash, or raw private payload is included.
```

If no usable local account/source is available, do not fabricate this file; report the live gate as incomplete.

- [ ] **Step 5: Append the new evidence pointer to `docs/project.md`**

Add one sentence under the current pin linking the verification record and summarizing only its sanitized pass/fail outcome.

---

### Task 8: Prepare the Migration Handoff

**Files:**
- Inspect: complete working-tree diff

**Interfaces:**
- Consumes: automated and live verification evidence.
- Produces: a reviewable migration handoff without absorbing unrelated user changes.

- [ ] **Step 1: Separate migration changes from pre-existing dirty work**

Compare the final status with the pre-migration dirty-file list. For overlapping files (`runtime.rs`, `media.rs`, Takeout files, and any others), describe which hunks belong to the migration. Do not stage the whole file merely because it contains a migration hunk.

- [ ] **Step 2: Commit only with an approved clean staging set**

If the migration hunks can be staged without including user-owned changes, stage the exact reviewed set and run:

```powershell
git diff --cached --check
git diff --cached --stat
git commit -m "build: upgrade grammers to 0.10"
```

If they cannot be separated safely, leave implementation changes uncommitted and report that constraint instead of committing mixed ownership.

- [ ] **Step 3: Report completion evidence**

Report the exact target revision, changed adapters, package-test result, feature-off result, workspace-verify result, live-smoke status, and whether implementation changes were committed.
