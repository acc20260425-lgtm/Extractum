# Rust Infrastructure Wave 0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a green, reproducible Windows Rust 1.95 baseline with pinned tooling/MSRV, Clippy-clean code, locked verification, supply-chain policy, executable repository rules, and CI/release executors before any edition or dependency upgrade.

**Architecture:** Keep the repository root as the operator context and `src-tauri/Cargo.toml` as the explicit Cargo workspace manifest. Reuse the existing Vitest repository-index/rule system for static policy, add one generated build-graph baseline for real Windows duplicate versions, and make GitHub Actions execute the same fast/full/release gates documented locally. Wave 0 is multi-commit infrastructure bootstrap; Waves 1–5 receive fresh plans after their dependency inventories are re-measured.

**Tech Stack:** Rust 1.95.0, Cargo/rustup, Clippy, rustfmt, cargo-deny 0.20.2, Node.js/TypeScript, Vitest 4, GitHub Actions, Tauri 2, Playwright 1.61, PowerShell on `windows-latest`.

## Global Constraints

- Pin Rust exactly to `1.95.0`; declare workspace `rust-version = "1.95"`; keep Edition `2021` in Wave 0.
- Run all Cargo commands from the repository root with `--manifest-path src-tauri/Cargo.toml`; resolution-consuming gates use `--locked`.
- Do not add `cargo-audit`; deterministic PR policy is cargo-deny `bans licenses sources`, while `advisories` runs on schedule and release.
- Every cargo-deny command passes `--config deny.toml` explicitly.
- The Windows duplicate baseline measures the active feature graph from compact normalized `cargo tree`, not Cargo metadata or `cargo tree -d` rendering.
- Keep `csp-verification`, `prompt-pack-dev-fixtures`, and `app-test-support` out of the normal Clippy gate; feature-off production checks remain separate Cargo checks.
- Preserve Gemini Browser sidecar JSON wire format and all existing product behavior.
- Keep SQLite backend-owned and migrations additive; Wave 0 creates no migration.
- Use `npm.cmd` for repository npm scripts on Windows.
- Each task ends with its own reviewable commit; do not squash Wave 0 into one atomic dependency-style commit.
- Before the first full verifier in a fresh checkout, run `npm.cmd ci` and `npm.cmd run bootstrap:testing`.

---

## File and Responsibility Map

**Create**

- `rust-toolchain.toml` — root rustup pin, components, and Windows target.
- `deny.toml` — Windows-only cargo-deny license, bans, sources, and advisory policy.
- `scripts/testing/rust-dependency-policy.json` — canonical toolchain/MSRV, exact-pin, prerelease, and Tauri-family ownership artifact.
- `scripts/testing/rust-duplicate-baseline.json` — schema/target, active graph totals, and sorted duplicate-name cardinalities.
- `scripts/rust-duplicate-baseline.mjs` — compact Cargo-tree parser and generator.
- `scripts/rust-duplicate-baseline.test.ts` — parser/generator unit tests.
- `scripts/testing/rust-supply-chain-exceptions.json` — owner/reason/review-date records for cargo-deny exceptions and approved duplicate growth.
- `.github/tools/cargo-deny.json` — pinned cargo-deny Windows release URL and SHA-256.
- `.github/actions/setup-cargo-deny/action.yml` — shared download/checksum/PATH bootstrap.
- `.github/workflows/rust-fast.yml` — push and PR fast Rust gate.
- `.github/workflows/rust-full.yml` — PR/manual full repository gate.
- `.github/workflows/rust-release.yml` — scheduled advisories and manual/tag Windows bundle gate.
- `scripts/windows-release-artifacts.mjs` — release artifact inventory/validation.
- `scripts/windows-release-artifacts.test.ts` — artifact validation unit tests.
- `scripts/testing/github-rust-workflows.test.ts` — contracts for triggers, shared tool bootstrap, and exact gate commands.
- `docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md` — final factual evidence.

**Modify**

- `src-tauri/Cargo.toml` — workspace MSRV, root inheritance, and `publish = false`.
- `src-tauri/crates/*/Cargo.toml` — `rust-version.workspace = true` for all six extracted crates.
- `scripts/verify.mjs` — remove redundant workspace check; lock workspace tests.
- `scripts/verify.test.ts` — assert exact Cargo test arguments and absence of Cargo check.
- `src-tauri/crates/extractum-gemini-browser/src/execution.rs` — box private completed selection.
- `src-tauri/crates/extractum-gemini-browser/src/types.rs` — box sidecar run-result payload while preserving JSON.
- `src-tauri/src/gemini_browser/sidecar.rs` — unwrap the boxed cross-crate response.
- `src-tauri/crates/extractum-llm/src/scheduler.rs` — box `LlmRequestError::Failed`; implement meaningful `Default`.
- `src-tauri/crates/extractum-prompt-packs/src/completion_transport.rs` — adapt owned boxed LLM failures.
- `src-tauri/crates/extractum-analysis/src/chat.rs` — checkpoint borrowed boxed LLM failure formatting.
- `src-tauri/crates/extractum-analysis/src/report/phases.rs` — checkpoint borrowed boxed LLM failure formatting.
- `src-tauri/src/llm/mod.rs` — checkpoint the root event consumer of boxed LLM failures.
- `src-tauri/crates/extractum-telegram/src/runtime.rs` — implement meaningful `Default`.
- `scripts/testing/repository-index.mjs` — cache raw compact Cargo-tree output.
- `scripts/testing/repository-rules.mjs` — add three Rust policy rules and duplicate evaluator.
- `scripts/testing/repository-rules.test.ts` — positive/mutation fixtures and registry inventory.
- `scripts/testing/test-conventions.test.ts` — require the new policy and generator files in the repository test inventory.
- `package.json` — named fast Rust and supply-chain scripts used locally and by CI.
- `AGENTS.md`, `docs/project.md`, `README.md` — canonical pinned/locked/CI/release workflow.

## Interfaces Between Tasks

- `generateRustDuplicateBaseline(treeText: string): RustDuplicateBaseline` produces the canonical JSON shape consumed by repository rules.
- `RepositoryIndex#getCargoTree(): string` supplies cached compact Windows Cargo-tree output; fixtures inject it without running Cargo.
- `rule:rust-toolchain-policy`, `rule:rust-dependency-policy`, and `rule:rust-duplicate-baseline` join `registeredRuleIds`.
- `.github/actions/setup-cargo-deny` reads `.github/tools/cargo-deny.json` and leaves `cargo-deny.exe` on `PATH` for all workflows.
- `validateWindowsReleaseArtifacts(paths: string[]): WindowsReleaseInventory` validates release outputs before workflow artifact upload.
- `npm.cmd run check:rust:fast` is the shared fast gate; `npm.cmd run verify` owns the single locked workspace test command.

---

### Task 1: Pin Rust and Declare the Workspace MSRV

**Files:**
- Create: `rust-toolchain.toml`
- Modify: `src-tauri/Cargo.toml`
- Modify: all six `src-tauri/crates/*/Cargo.toml`
- Test: `scripts/testing/repository-rules.test.ts` (rule implementation lands in Task 6)

**Interfaces:**
- Produces: root toolchain `1.95.0`, workspace `rust-version = "1.95"`, seven inherited package declarations, seven unpublished packages.
- Consumes: no prior Wave 0 interface.

- [ ] **Step 1: Record the current synchronized baseline**

Run:

```powershell
rustc -Vv
cargo -V
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 | Out-Null
```

Expected: Rust/Cargo `1.95.0`; locked metadata exits `0` before manifest edits.

- [ ] **Step 2: Add the root toolchain pin**

Create `rust-toolchain.toml` exactly:

```toml
[toolchain]
channel = "1.95.0"
components = ["rustfmt", "clippy"]
targets = ["x86_64-pc-windows-msvc"]
profile = "minimal"
```

- [ ] **Step 3: Add MSRV and private-package declarations**

In `src-tauri/Cargo.toml`, make the workspace/root package blocks contain:

```toml
[workspace.package]
version = "0.2.0"
edition = "2021"
rust-version = "1.95"

[package]
name = "extractum"
version = "0.2.0"
description = "A Tauri App"
authors = ["you"]
edition.workspace = true
rust-version.workspace = true
publish = false
```

In every extracted crate manifest, add directly below `edition.workspace = true`:

```toml
rust-version.workspace = true
```

- [ ] **Step 4: Verify all seven normalized packages**

Run:

```powershell
$metadata = cargo metadata --manifest-path src-tauri/Cargo.toml --locked --no-deps --format-version 1 | ConvertFrom-Json
$metadata.packages | Sort-Object name | Select-Object name,edition,rust_version
```

Expected: seven packages, all `edition = 2021`, all `rust_version = 1.95`.

- [ ] **Step 5: Run focused manifest gates**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-core --lib --no-default-features --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
npm.cmd run bootstrap:testing
npm.cmd run verify
git diff --check
```

Expected: all pass; `src-tauri/Cargo.lock` remains unchanged.

- [ ] **Step 6: Commit the reproducible toolchain baseline**

```powershell
git add rust-toolchain.toml src-tauri/Cargo.toml src-tauri/crates/*/Cargo.toml
git commit -m "build: pin Rust toolchain and MSRV"
```

---

### Task 2: Remove Redundant Cargo Work From the Full Verifier

**Files:**
- Modify: `scripts/verify.mjs`
- Modify: `scripts/verify.test.ts`
- Modify: `package.json`

**Interfaces:**
- Produces: one locked workspace test step in `verify`; reusable `check:rust:fast` script.
- Consumes: Task 1 root toolchain/manifest.

- [ ] **Step 1: Write the failing verifier contract test**

Replace the first test's expected trailing steps so it asserts one Cargo command and exact arguments:

```ts
const steps = createVerifySteps({ npmExecPath: "npm-cli.js", platform: "win32" });
expect(steps.map((step) => step.npmScript ?? step.command)).toEqual([
  "check:gemini-browser-sidecar-binary",
  "test:unit",
  "test:component",
  "test:architecture",
  "test:integration:os",
  "test:e2e",
  "test:app:e2e",
  "check",
  "check:rustfmt",
  "cargo",
  "git",
]);
const cargoSteps = steps.filter((step) => step.command === "cargo");
expect(cargoSteps).toEqual([{
  title: "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked",
  command: "cargo",
  args: ["test", "--manifest-path", "src-tauri/Cargo.toml", "--workspace", "--all-targets", "--locked"],
}]);
```

- [ ] **Step 2: Run the RED test**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts
```

Expected: FAIL because two unlocked Cargo steps still exist.

- [ ] **Step 3: Implement the minimal verifier change**

In `createVerifySteps`, delete the workspace Cargo check object and replace the test object with:

```js
{
  title: "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked",
  command: "cargo",
  args: ["test", "--manifest-path", "src-tauri/Cargo.toml", "--workspace", "--all-targets", "--locked"],
},
```

Add these `package.json` scripts (the deny script becomes executable after Task 4):

```json
"check:rust:clippy": "cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked -- -D warnings",
"check:rust:production": "cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features --locked && cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib --no-default-features --locked",
"check:rust:supply-chain": "cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check bans licenses sources",
"check:rust:advisories": "cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check advisories",
"check:rust:fast": "npm run check:rustfmt && npm run check:rust:clippy && npm run check:rust:production && npm run check:rust:supply-chain"
```

- [ ] **Step 4: Run GREEN tests**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts
npm.cmd run check:rustfmt
```

Expected: PASS. Do not run `check:rust:fast` before cargo-deny exists.

- [ ] **Step 5: Commit the verifier simplification**

```powershell
git add scripts/verify.mjs scripts/verify.test.ts package.json
git commit -m "build: lock and streamline Rust verification"
```

---

### Task 3: Establish a Green Clippy Baseline

**Files:**
- Modify: `src-tauri/crates/extractum-gemini-browser/src/execution.rs`
- Modify: `src-tauri/crates/extractum-gemini-browser/src/types.rs`
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Modify: `src-tauri/crates/extractum-llm/src/scheduler.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/runtime.rs`

**Interfaces:**
- Produces: unchanged sidecar JSON, boxed large Rust variants, meaningful `Default` for two empty runtime states.
- Consumes: Task 2 accepted Clippy command.

- [ ] **Step 1: Capture the accepted five-finding RED baseline**

Ensure no unrelated Cargo process owns `src-tauri/target/debug/.cargo-lock`, then run:

```powershell
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked --message-format=short -- -D warnings
```

Expected: exactly three `large_enum_variant` and two `new_without_default` failures. If the set differs, update the verification record and re-scope before editing.

- [ ] **Step 2: Add the sidecar wire-format RED test**

In `types.rs` tests, add a helper result and an exact response round-trip:

```rust
#[test]
fn sidecar_run_result_response_keeps_wire_shape() {
    let response = GeminiBrowserSidecarResponse::RunResult {
        result: Box::new(GeminiBrowserRunResult {
            run_id: "run-1".to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 42,
            debug_summary: None,
        }),
    };
    let json = serde_json::to_value(&response).expect("serialize sidecar response");
    assert_eq!(json["type"], "run_result");
    assert_eq!(json["result"]["run_id"], "run-1");
    assert!(json.get("result").is_some());
    let decoded: GeminiBrowserSidecarResponse =
        serde_json::from_value(json).expect("deserialize sidecar response");
    assert_eq!(decoded, response);
}
```

Initially write it against the intended boxed API so it fails to compile before the implementation.

- [ ] **Step 3: Box the two Gemini Browser variants**

Change the private selection:

```rust
enum ExecutionSelection {
    Completed(Box<GeminiBrowserResult<GeminiBrowserRunResult>>),
    Cancelled,
    TimedOut,
}
```

Construct with `ExecutionSelection::Completed(Box::new(result))` and match with:

```rust
ExecutionSelection::Completed(result) => match *result {
    Ok(result) => {
        terminalize(runtime, state, observer, &input, result.clone()).await?;
        Ok(DeliveryOutcome::Completed { result })
    }
    Err(error) => {
        let result = failed_result(&input.job.run_id, error.to_string());
        terminalize(runtime, state, observer, &input, result.clone()).await?;
        Ok(DeliveryOutcome::Failed { result })
    }
},
```

Change the public response variant:

```rust
RunResult { result: Box<GeminiBrowserRunResult> },
```

In app-side `sidecar.rs`, unwrap at the immediate consumer:

```rust
GeminiBrowserSidecarResponse::RunResult { result } => Ok(*result),
```

- [ ] **Step 4: Run Gemini Browser focused tests and consumer checkpoint**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
```

Expected: non-empty PASS; app consumer compiles.

- [ ] **Step 5: Box the public LLM error payload and add `Default`**

Change:

```rust
pub enum LlmRequestError {
    Cancelled,
    Failed(Box<AppError>),
}
```

Update both scheduler constructors:

```rust
result = future => result.map_err(|error| LlmRequestError::Failed(Box::new(error.into()))),
```

```rust
.map_err(|error| LlmRequestError::Failed(Box::new(error)))?;
```

Update scheduler tests that construct the variant:

```rust
Err(LlmRequestError::Failed(Box::new(AppError::from("boom"))))
```

Borrow-only consumers in `extractum-analysis/src/chat.rs`,
`extractum-analysis/src/report/phases.rs`, and app `src/llm/mod.rs` continue to
use `error.to_string()` or deref coercion. In
`extractum-prompt-packs/src/completion_transport.rs`, update both owned
conversions:

```rust
Err(LlmRequestError::Failed(error)) => {
    Err(YoutubeSummaryStageExecutionError::Failed(*error))
}
```

and update the browser-error constructor:

```rust
.map_err(|error| LlmRequestError::Failed(Box::new(error)))
```

Add:

```rust
impl Default for LlmSchedulerState {
    fn default() -> Self {
        Self::new()
    }
}
```

Add a focused synchronous test in the scheduler test module:

```rust
#[test]
fn scheduler_default_matches_new_empty_state() {
    fn assert_default<T: Default>() {}
    assert_default::<LlmSchedulerState>();
    let _scheduler = LlmSchedulerState::default();
}
```

- [ ] **Step 6: Run LLM owner and immediate dependent checkpoints**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib scheduler::tests::scheduler_default_matches_new_empty_state --locked -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
```

Expected: all pass; no public consumer retains the unboxed constructor pattern.

- [ ] **Step 7: Add meaningful `Default` for Telegram runtime**

Add immediately after `impl TelegramRuntime` or before it:

```rust
impl Default for TelegramRuntime {
    fn default() -> Self {
        Self::new()
    }
}
```

Add in its test module:

```rust
#[test]
fn telegram_runtime_default_constructs_empty_runtime() {
    fn assert_default<T: Default>() {}
    assert_default::<TelegramRuntime>();
    let _runtime = TelegramRuntime::default();
}
```

- [ ] **Step 8: Run Telegram feature-off, owner, and app consumer gates**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib runtime::tests::telegram_runtime_default_constructs_empty_runtime --locked -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
```

Expected: all pass; the feature-off check proves production surface, app tests prove the dev-feature seam.

- [ ] **Step 9: Close the accepted Clippy gate**

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked -- -D warnings
npm.cmd run verify
```

Expected: PASS with no global or crate-level lint allow.

- [ ] **Step 10: Commit the Clippy baseline**

```powershell
git add src-tauri/crates/extractum-gemini-browser/src/execution.rs src-tauri/crates/extractum-gemini-browser/src/types.rs src-tauri/src/gemini_browser/sidecar.rs src-tauri/crates/extractum-llm/src/scheduler.rs src-tauri/crates/extractum-prompt-packs/src/completion_transport.rs src-tauri/crates/extractum-analysis/src/chat.rs src-tauri/crates/extractum-analysis/src/report/phases.rs src-tauri/src/llm/mod.rs src-tauri/crates/extractum-telegram/src/runtime.rs
git commit -m "refactor: establish Rust Clippy baseline"
```

---

### Task 4: Bootstrap Cargo-Deny and License Policy

**Files:**
- Create: `deny.toml`
- Create: `.github/tools/cargo-deny.json`
- Create: `.github/actions/setup-cargo-deny/action.yml`
- Create: `scripts/testing/rust-supply-chain-exceptions.json`
- Modify: `package.json`

**Interfaces:**
- Produces: pinned `cargo-deny.exe` bootstrap and green deterministic/advisory commands.
- Consumes: Task 1 `publish = false` for all workspace packages; Task 2 scripts.

- [ ] **Step 1: Add the pinned tool manifest**

Create `.github/tools/cargo-deny.json`:

```json
{
  "schemaVersion": 1,
  "version": "0.20.2",
  "target": "x86_64-pc-windows-msvc",
  "url": "https://github.com/EmbarkStudios/cargo-deny/releases/download/0.20.2/cargo-deny-0.20.2-x86_64-pc-windows-msvc.tar.gz",
  "sha256": "975a22143262fd27476d19ee00c7af67978426e40e1dee94eed6bbade1cf87dc"
}
```

- [ ] **Step 2: Add the shared composite setup action**

Create `.github/actions/setup-cargo-deny/action.yml`:

```yaml
name: Setup pinned cargo-deny
description: Download and verify the repository-pinned Windows cargo-deny binary
runs:
  using: composite
  steps:
    - name: Download and verify cargo-deny
      shell: pwsh
      run: |
        $manifest = Get-Content -Raw '.github/tools/cargo-deny.json' | ConvertFrom-Json
        $toolDir = Join-Path $env:RUNNER_TEMP 'extractum-cargo-deny'
        New-Item -ItemType Directory -Force -Path $toolDir | Out-Null
        $archive = Join-Path $toolDir 'cargo-deny.tar.gz'
        Invoke-WebRequest -Uri $manifest.url -OutFile $archive
        $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $manifest.sha256) { throw "cargo-deny SHA-256 mismatch: $actual" }
        tar.exe -xzf $archive -C $toolDir
        $binary = Get-ChildItem -LiteralPath $toolDir -Recurse -Filter 'cargo-deny.exe' | Select-Object -First 1
        if (-not $binary) { throw 'cargo-deny.exe missing from verified archive' }
        $binary.DirectoryName | Out-File -FilePath $env:GITHUB_PATH -Encoding utf8 -Append
        & $binary.FullName --version
```

- [ ] **Step 3: Add a curated initial `deny.toml`**

Create:

```toml
[graph]
targets = ["x86_64-pc-windows-msvc"]

[advisories]
unmaintained = "workspace"
unsound = "all"
yanked = "warn"

[bans]
multiple-versions = "warn"
wildcards = "deny"

[licenses]
confidence-threshold = 0.93
unused-allowed-license = "warn"
allow = [
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-3-Clause",
  "CDLA-Permissive-2.0",
  "ISC",
  "MIT",
  "MIT-0",
  "MPL-2.0",
  "Unicode-3.0",
  "Zlib",
]

[licenses.private]
ignore = true

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

Do not add `[[licenses.clarify]]` unless cargo-deny 0.20.2 proves a specific packaged expression cannot be validated. If required for `ring`, `unicode-ident`, or `rustls-webpki`, hash the exact cached license file and bind the clarification to the exact crate version; record the inspected archive URL in the exception artifact.

- [ ] **Step 4: Add the exception artifact schema**

Create `scripts/testing/rust-supply-chain-exceptions.json`:

```json
{
  "schemaVersion": 1,
  "licenseExceptions": [],
  "advisoryExceptions": [],
  "duplicateGrowthExceptions": []
}
```

Each later entry must contain `package`, `owner`, `reason`, `reviewAfter`, and the relevant advisory/license/duplicate identity.

- [ ] **Step 5: Exercise the pinned binary locally using the same manifest**

Download into a temporary directory (do not install globally), validate the
recorded checksum, extract, and prepend its directory to the current process
`PATH`:

```powershell
$manifest = Get-Content -Raw '.github/tools/cargo-deny.json' | ConvertFrom-Json
$toolDir = Join-Path ([System.IO.Path]::GetTempPath()) ("extractum-cargo-deny-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $toolDir | Out-Null
try {
  $archive = Join-Path $toolDir 'cargo-deny.tar.gz'
  Invoke-WebRequest -Uri $manifest.url -OutFile $archive
  $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actual -ne $manifest.sha256) { throw "cargo-deny SHA-256 mismatch: $actual" }
  tar.exe -xzf $archive -C $toolDir
  $binary = Get-ChildItem -LiteralPath $toolDir -Recurse -Filter 'cargo-deny.exe' | Select-Object -First 1
  if (-not $binary) { throw 'cargo-deny.exe missing from verified archive' }
  $env:PATH = "$($binary.DirectoryName);$env:PATH"
  cargo deny --version
  cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check bans licenses sources
  cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check advisories
} finally {
  Remove-Item -LiteralPath $toolDir -Recurse -Force -ErrorAction SilentlyContinue
}
```

The two policy commands executed inside the `try` block are:

```powershell
cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check bans licenses sources
cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check advisories
```

Expected: both commands execute cargo-deny 0.20.2. Resolve license failures by narrowing exact exceptions/clarifications, never by setting a blanket allow.

- [ ] **Step 6: Run the complete fast gate**

```powershell
npm.cmd run check:rust:fast
```

Expected: fmt, Clippy, both feature-off checks, and deterministic cargo-deny pass.

- [ ] **Step 7: Commit supply-chain bootstrap**

```powershell
git add deny.toml .github/tools/cargo-deny.json .github/actions/setup-cargo-deny/action.yml scripts/testing/rust-supply-chain-exceptions.json package.json
git commit -m "build: add Rust supply-chain policy"
```

---

### Task 5: Generate and Enforce the Active Windows Duplicate Baseline

**Files:**
- Create: `scripts/rust-duplicate-baseline.mjs`
- Create: `scripts/rust-duplicate-baseline.test.ts`
- Create: `scripts/testing/rust-duplicate-baseline.json`
- Modify: `scripts/testing/repository-index.mjs`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `package.json`

**Interfaces:**
- Produces: `generateRustDuplicateBaseline(treeText)`, `RepositoryIndex#getCargoTree()`, `rule:rust-duplicate-baseline`.
- Consumes: Task 4 exception artifact for approved future growth.

- [ ] **Step 1: Write parser RED tests**

Create tests that prove deduplication, `(*)` tolerance, per-name cardinality, and stable sort:

```ts
import { describe, expect, it } from "vitest";
import { generateRustDuplicateBaseline } from "./rust-duplicate-baseline.mjs";

describe("Rust duplicate baseline", () => {
  it("counts unique package versions and localizes duplicate cardinality", () => {
    const baseline = generateRustDuplicateBaseline([
      "extractum v0.2.0 (G:\\Develop\\Extractum\\src-tauri)",
      "getrandom v0.2.17",
      "getrandom v0.4.3 (*)",
      "getrandom v0.2.17 (*)",
      "serde v1.0.229",
    ].join("\n"));
    expect(baseline).toEqual({
      schemaVersion: 1,
      target: "x86_64-pc-windows-msvc",
      packageVersionCount: 4,
      packageNameCount: 3,
      duplicateNameCount: 1,
      duplicateVersionInstanceCount: 2,
      duplicateCardinality: { getrandom: 2 },
    });
  });
});
```

- [ ] **Step 2: Run RED**

```powershell
npm.cmd run test:unit -- scripts/rust-duplicate-baseline.test.ts
```

Expected: FAIL because generator does not exist.

- [ ] **Step 3: Implement parser and CLI generation**

Export:

```js
export function generateRustDuplicateBaseline(treeText) {
  const identities = new Map();
  for (const line of String(treeText).split(/\r?\n/)) {
    const match = /^(\S+) v(\S+)/.exec(line.trim());
    if (!match) continue;
    const [, name, version] = match;
    identities.set(`${name}@${version}`, { name, version });
  }
  const versionsByName = new Map();
  for (const { name, version } of identities.values()) {
    const versions = versionsByName.get(name) ?? new Set();
    versions.add(version);
    versionsByName.set(name, versions);
  }
  const duplicateCardinality = Object.fromEntries(
    [...versionsByName]
      .filter(([, versions]) => versions.size > 1)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, versions]) => [name, versions.size]),
  );
  return {
    schemaVersion: 1,
    target: "x86_64-pc-windows-msvc",
    packageVersionCount: identities.size,
    packageNameCount: versionsByName.size,
    duplicateNameCount: Object.keys(duplicateCardinality).length,
    duplicateVersionInstanceCount: Object.values(duplicateCardinality).reduce((sum, count) => sum + count, 0),
    duplicateCardinality,
  };
}
```

The CLI invokes exactly:

```js
execFileSync("cargo", [
  "tree", "--manifest-path", "src-tauri/Cargo.toml", "--locked",
  "--target", "x86_64-pc-windows-msvc", "--workspace",
  "--prefix", "none", "--format", "{p}",
], { cwd: repoRoot, encoding: "utf8", maxBuffer: 16 * 1024 * 1024, windowsHide: true });
```

Support `--write scripts/testing/rust-duplicate-baseline.json`; default mode prints canonical JSON without writing.

- [ ] **Step 4: Generate and inspect the real baseline**

```powershell
node scripts/rust-duplicate-baseline.mjs --write scripts/testing/rust-duplicate-baseline.json
Get-Content -Raw scripts/testing/rust-duplicate-baseline.json
```

Expected initial snapshot: `448`, `400`, `32`, `80`, and sorted per-name counts. If current graph differs, record the re-measurement; do not force stale design numbers.

- [ ] **Step 5: Extend the repository index with injected Cargo-tree loading**

Add `loadCargoTree` beside `loadCargoMetadata`, cache success/error once, and expose:

```js
getCargoTree() {
  if (cargoTreeLoaded) {
    if (cargoTreeError) throw cargoTreeError;
    return cargoTree;
  }
  cargoTreeLoaded = true;
  try {
    cargoTree = String(loadCargoTree());
    return cargoTree;
  } catch (error) {
    cargoTreeError = errorFor("src-tauri/Cargo.toml", error);
    throw cargoTreeError;
  }
},
```

The default loader runs the exact compact locked command above.

- [ ] **Step 6: Add the duplicate evaluator and mutation fixtures**

In `repository-rules.mjs`, compare generated and canonical state with these rules:

```js
if (actual.duplicateNameCount > baseline.duplicateNameCount) violations.push("Rust duplicate-name count grew");
if (actual.duplicateVersionInstanceCount > baseline.duplicateVersionInstanceCount) violations.push("Rust duplicate version-instance count grew");
for (const [name, count] of Object.entries(actual.duplicateCardinality)) {
  if (count > (baseline.duplicateCardinality[name] ?? 1)) {
    violations.push(`${name}: duplicate version cardinality grew to ${count}`);
  }
}
```

Register `rule:rust-duplicate-baseline`. Fixtures must cover a positive snapshot, a new duplicate name, a third version of an existing duplicate, and a version replacement at unchanged cardinality.

- [ ] **Step 7: Run generator/rule GREEN tests and real snapshot**

```powershell
npm.cmd run test:unit -- scripts/rust-duplicate-baseline.test.ts scripts/testing/repository-rules.test.ts
npm.cmd run test:unit
```

Expected: PASS; current repository rule accepts current canonical baseline.

- [ ] **Step 8: Commit duplicate enforcement**

```powershell
git add scripts/rust-duplicate-baseline.mjs scripts/rust-duplicate-baseline.test.ts scripts/testing/rust-duplicate-baseline.json scripts/testing/repository-index.mjs scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts package.json
git commit -m "test: enforce Rust duplicate baseline"
```

---

### Task 6: Enforce Toolchain, Tauri-Family, Pin, and Prerelease Policy

**Files:**
- Create: `scripts/testing/rust-dependency-policy.json`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `scripts/testing/test-conventions.test.ts`

**Interfaces:**
- Produces: `rule:rust-toolchain-policy`, `rule:rust-dependency-policy`.
- Consumes: Task 1 manifests/toolchain; Task 5 rule framework.

- [ ] **Step 1: Add canonical policy data**

Create:

```json
{
  "schemaVersion": 1,
  "toolchain": {
    "channel": "1.95.0",
    "rustVersion": "1.95",
    "edition": "2021",
    "target": "x86_64-pc-windows-msvc",
    "components": ["clippy", "rustfmt"],
    "workspacePackages": [
      "extractum",
      "extractum-analysis",
      "extractum-core",
      "extractum-gemini-browser",
      "extractum-llm",
      "extractum-prompt-packs",
      "extractum-telegram"
    ]
  },
  "exactPins": {
    "apalis": "=1.0.0-rc.8",
    "apalis-sqlite": "=1.0.0-rc.8",
    "grammers-client": "=0.10.0",
    "grammers-mtsender": "=0.10.0",
    "grammers-session": "=0.10.0",
    "grammers-tl-types": "=0.10.0"
  },
  "approvedPrereleases": {
    "apalis": "1.0.0-rc.8",
    "apalis-sqlite": "1.0.0-rc.8"
  },
  "tauriFamily": {
    "cargoMajor": 2,
    "npmMajor": 2,
    "mcpBridgeMinor": 11,
    "pairs": [
      ["tauri", "@tauri-apps/api"],
      ["tauri-build", "@tauri-apps/cli"],
      ["tauri-plugin-dialog", "@tauri-apps/plugin-dialog"],
      ["tauri-plugin-opener", "@tauri-apps/plugin-opener"],
      ["tauri-plugin-sql", "@tauri-apps/plugin-sql"]
    ]
  }
}
```

- [ ] **Step 2: Write positive and violating fixtures first**

Add fixture mutations for:

- missing `rust-version.workspace = true` in one package;
- toolchain channel changed to `1.96.0` while policy remains 1.95;
- root package not private;
- `@tauri-apps/cli` major differs from Cargo Tauri major;
- MCP bridge manifest widened/moved outside minor 0.11;
- one Grammers exact pin widened;
- unapproved prerelease introduced.

Run:

```powershell
npm.cmd run test:unit -- scripts/testing/repository-rules.test.ts
```

Expected: RED until evaluators exist.

- [ ] **Step 3: Implement the two evaluators**

Use `index.getText("rust-toolchain.toml")`, `index.getCargoMetadata()`, `index.getJson("package.json")`, and `index.getJson("scripts/testing/rust-dependency-policy.json")`. The toolchain evaluator checks exact channel/components/target, seven package `rust_version`, edition 2021, and seven `publish = []/false` metadata states. The dependency evaluator checks direct manifest requirements for exact pins, approved prerelease inventory, Tauri/npm major compatibility, and MCP bridge minor 11.

Implement shared parsers with explicit failure messages:

```js
function quotedTomlValue(source, key) {
  const match = new RegExp(`^${key.replaceAll("-", "\\-")}\\s*=\\s*"([^"]+)"`, "m").exec(source);
  return match?.[1];
}

function quotedTomlArray(source, key) {
  const match = new RegExp(`^${key}\\s*=\\s*\\[([^\\]]*)\\]`, "m").exec(source);
  return match ? [...match[1].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]).sort() : undefined;
}

function dependencyByName(metadata, packageName, dependencyName) {
  const selected = workspacePackage(metadata, packageName);
  return selected?.dependencies?.find(({ name, kind }) => name === dependencyName && kind === null);
}

function requirementMajor(requirement) {
  const match = /(?:^|[^0-9])(\d+)(?:\.|$)/.exec(String(requirement));
  return match ? Number(match[1]) : undefined;
}
```

The toolchain evaluator must emit violations for every mismatch rather than
returning after the first:

```js
const toolchain = index.getText("rust-toolchain.toml");
if (quotedTomlValue(toolchain, "channel") !== policy.toolchain.channel) {
  violations.push("rust-toolchain.toml: channel must match Rust dependency policy");
}
if (JSON.stringify(quotedTomlArray(toolchain, "components")) !== JSON.stringify([...policy.toolchain.components].sort())) {
  violations.push("rust-toolchain.toml: component inventory drifted");
}
if (JSON.stringify(quotedTomlArray(toolchain, "targets")) !== JSON.stringify([policy.toolchain.target])) {
  violations.push("rust-toolchain.toml: target inventory drifted");
}
for (const name of policy.toolchain.workspacePackages) {
  const pkg = workspacePackage(metadata, name);
  if (!pkg) violations.push(`${name}: missing workspace package`);
  else {
    if (pkg.rust_version !== policy.toolchain.rustVersion) violations.push(`${name}: rust-version drifted`);
    if (pkg.edition !== policy.toolchain.edition) violations.push(`${name}: edition drifted during Wave 0`);
    if (JSON.stringify(pkg.publish) !== "[]") violations.push(`${name}: package must be unpublished`);
  }
}
```

For dependency policy, collect normal and build direct dependencies from all workspace
packages, compare every `exactPins` requirement exactly, reject any `req`
containing a prerelease segment unless its package/version pair is present in
`approvedPrereleases`, require every `tauriFamily.pairs` Cargo/npm requirement
to share configured major 2, and require
`tauri-plugin-mcp-bridge` requirement major/minor `0.11`.

Register exactly:

```js
["rule:rust-toolchain-policy", evaluateRustToolchainPolicy],
["rule:rust-dependency-policy", evaluateRustDependencyPolicy],
```

- [ ] **Step 4: Update the registry inventory test**

Expected `registeredRuleIds` becomes sorted and includes the three new Rust IDs:

```ts
"rule:rust-dependency-policy",
"rule:rust-duplicate-baseline",
"rule:rust-toolchain-policy",
```

Keep the existing three rules unchanged.

- [ ] **Step 5: Run rule and convention GREEN tests**

```powershell
npm.cmd run test:unit -- scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
npm.cmd run test:unit
```

Expected: PASS with positive and mutation coverage for every registered evaluator.

- [ ] **Step 6: Commit executable Rust policy**

```powershell
git add scripts/testing/rust-dependency-policy.json scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
git commit -m "test: enforce Rust infrastructure policy"
```

---

### Task 7: Add Fast, Full, Advisory, and Release CI Executors

**Files:**
- Create: `.github/workflows/rust-fast.yml`
- Create: `.github/workflows/rust-full.yml`
- Create: `.github/workflows/rust-release.yml`
- Create: `scripts/windows-release-artifacts.mjs`
- Create: `scripts/windows-release-artifacts.test.ts`
- Create: `scripts/testing/github-rust-workflows.test.ts`

**Interfaces:**
- Produces: push/PR fast gate, PR/manual full gate, scheduled advisories, tag/manual bundle gate.
- Consumes: Task 4 setup action and Task 2/4 npm scripts.

- [ ] **Step 1: Write release-artifact RED tests**

Define and test:

```ts
expect(validateWindowsReleaseArtifacts([
  "src-tauri/target/x86_64-pc-windows-msvc/release/extractum.exe",
  "src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/extractum_0.2.0_x64_en-US.msi",
  "src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/extractum_0.2.0_x64-setup.exe",
])).toEqual({ executable: expect.any(String), msi: expect.any(String), nsis: expect.any(String) });
```

Add failures for missing executable, MSI, and NSIS.

- [ ] **Step 2: Implement artifact validation**

Export a pure function and CLI. Normalize slashes and require exactly one existing executable plus at least one `.msi` under `/bundle/msi/` and one `-setup.exe` under `/bundle/nsis/`. CLI recursively lists `src-tauri/target/x86_64-pc-windows-msvc/release` and prints canonical JSON.

- [ ] **Step 3: Create the fast workflow**

Use `actions/checkout@v6`, repository toolchain (`rustup show active-toolchain`), `Swatinem/rust-cache@v2` with `workspaces: "./src-tauri -> target"`, local cargo-deny setup action, then:

```yaml
name: Rust Fast
on:
  push:
  pull_request:
jobs:
  rust-fast:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v6
      - run: rustup show active-toolchain
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "./src-tauri -> target"
      - uses: ./.github/actions/setup-cargo-deny
      - run: npm.cmd run check:rust:fast
```

- [ ] **Step 4: Create the full PR/manual workflow**

Trigger `pull_request` and `workflow_dispatch`. Use checkout, `actions/setup-node@v6` with cache `npm`, Rust cache, shared cargo-deny setup, then:

```yaml
- run: npm.cmd ci
- run: npm.cmd run bootstrap:testing
- run: node_modules\.bin\playwright.cmd install chromium
- run: npm.cmd run check:rust:fast
- run: npm.cmd run verify
```

Upload `test-results/`, `playwright-report/`, and `artifacts/playwright-results.json` on failure with `actions/upload-artifact@v4` and `if: ${{ !cancelled() }}`.

- [ ] **Step 5: Create scheduled advisory and release workflow**

Trigger:

```yaml
on:
  schedule:
    - cron: "17 4 * * 1"
  workflow_dispatch:
    inputs:
      bundle:
        description: Build and smoke the Windows bundle
        required: true
        type: boolean
        default: true
  push:
    tags: ["v*"]
```

`advisories` always checks out, sets up cargo-deny, and runs `npm.cmd run check:rust:advisories`. `windows-bundle` runs only for a tag or manual `bundle=true`, performs the full setup/gate, then:

```powershell
npm.cmd run tauri -- build --target x86_64-pc-windows-msvc
node scripts/windows-release-artifacts.mjs
npm.cmd run smoke:gemini-browser-sidecar:binary
```

Launch `src-tauri/target/x86_64-pc-windows-msvc/release/extractum.exe` with `Start-Process -PassThru -WindowStyle Hidden`, observe five seconds, fail on confirmed early exit, then stop/reap that exact PID in `finally`. Do not make live Telegram/LLM requests. Upload MSI/NSIS artifacts with `actions/upload-artifact@v4`.

- [ ] **Step 6: Test local scripts and inspect workflow syntax**

```powershell
npm.cmd run test:unit -- scripts/windows-release-artifacts.test.ts scripts/testing/github-rust-workflows.test.ts
npm.cmd run test:unit
git diff --check
```

`github-rust-workflows.test.ts` reads all three workflow files and asserts the
required triggers, `windows-latest`, local cargo-deny setup action, exact npm
gate names, release target, schedule, and artifact upload. GitHub parses and
validates the actual workflow syntax on the first PR; record that accepted run
URL in Task 8.

- [ ] **Step 7: Commit CI executors**

```powershell
git add .github/workflows/rust-fast.yml .github/workflows/rust-full.yml .github/workflows/rust-release.yml scripts/windows-release-artifacts.mjs scripts/windows-release-artifacts.test.ts scripts/testing/github-rust-workflows.test.ts
git commit -m "ci: add Rust verification and release gates"
```

---

### Task 8: Update Canonical Documentation and Capture Wave 0 Evidence

**Files:**
- Modify: `AGENTS.md`
- Modify: `docs/project.md`
- Modify: `README.md`
- Create: `docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md`

**Interfaces:**
- Produces: operator-facing workflow identical to executable scripts/CI; factual completion evidence.
- Consumes: all prior Wave 0 tasks.

- [ ] **Step 1: Update agent and project workflow rules**

Document these exact distinctions:

- root `rust-toolchain.toml` and MSRV inheritance;
- fast local gate `npm.cmd run check:rust:fast`;
- full gate bootstrap and `npm.cmd run verify`;
- `verify` owns one locked workspace test and no longer runs workspace check first;
- separate producer feature-off checks;
- cargo-deny deterministic versus advisory cadence;
- toolchain bump isolation as a dedicated future wave;
- dependency updates use explicit manifest and precise direct packages;
- release job requirement for Tauri, `windows-sys`, Keyring, or SQLx waves.

Update the numbered `docs/project.md` verifier list so it matches `createVerifySteps` exactly.

- [ ] **Step 2: Run focused documentation/policy tests**

```powershell
npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 3: Run the complete local acceptance contract**

```powershell
npm.cmd run bootstrap:testing
npm.cmd run check:rust:fast
npm.cmd run verify
npm.cmd run check:rust:advisories
```

Expected: all pass. If advisory state changes independently, record the exact RustSec finding and follow the approved exception/security-task path; do not silently suppress it.

- [ ] **Step 4: Dispatch and verify CI executors**

Push the branch/PR, then require:

- green `Rust Fast` run;
- green `Rust Full` run;
- manual green `Rust Release` run with bundle enabled;
- uploaded MSI and NSIS artifacts;
- successful five-second application smoke and sidecar binary smoke.

Do not claim CI complete from local commands alone.

- [ ] **Step 5: Write the verification record with actual evidence**

Populate the verification document with no placeholders:

- commit range for Wave 0;
- `rustc -Vv`, `cargo -V`, cargo-deny version and SHA-256;
- five Clippy classifications and exact focused test names/results;
- license allowlist and any exact clarifications/exceptions;
- actual duplicate baseline totals and cardinality changes;
- fast/full/advisory command outputs;
- GitHub Actions run URLs;
- release executable/MSI/NSIS paths and hashes;
- sidecar/application smoke results;
- confirmation that Edition remains 2021 and dependency versions were not intentionally upgraded.

- [ ] **Step 6: Run final consistency checks**

```powershell
rg -n "TBD|TODO|implement later|fill in" docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md
git diff --check
git status --short
```

Expected: no placeholders; only intended documentation/evidence files remain.

- [ ] **Step 7: Commit documentation and evidence**

```powershell
git add AGENTS.md docs/project.md README.md docs/superpowers/verification/2026-08-12-rust-infrastructure-wave-0.md
git commit -m "docs: verify Rust infrastructure baseline"
```

---

## Rust Verification Loops

### Affected Packages

- `extractum-gemini-browser`: `ExecutionSelection`, public sidecar response wire type.
- `extractum-llm`: public `LlmRequestError`, public scheduler default construction.
- `extractum-telegram`: public runtime default construction and feature-off surface.
- `extractum`: immediate consumer of Gemini Browser, LLM, and Telegram APIs.
- `extractum-analysis` and `extractum-prompt-packs`: immediate LLM consumers.
- All seven packages: toolchain/MSRV and final locked workspace test.

### Narrow RED/GREEN Tests

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --lib scheduler::tests::scheduler_default_matches_new_empty_state --locked -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib runtime::tests::telegram_runtime_default_constructs_empty_runtime --locked -- --exact
```

Before trusting an exact filter, list tests if necessary and reject any run reporting zero tests.

### Focused Checks

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib --no-default-features --locked
```

### Package Checkpoints and Immediate Dependents

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked
```

### End-of-Slice Workspace Gates

Every Rust-source task ends with:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked -- -D warnings
npm.cmd run verify
```

At Wave 0 completion also run the two feature-off checks and cargo-deny commands through `npm.cmd run check:rust:fast` and `npm.cmd run check:rust:advisories`. A filtered Cargo run, changed-test selector, or focused package checkpoint is an accelerator, never completion evidence.

---

## Post-Wave Planning Checkpoints

Do not precompute implementation versions for Waves 1–5 in this plan. After Wave 0 is green and merged, re-run the locked/dry-run inventory and create separate plans in order:

1. **Wave 1:** compatible low-risk directs (`anyhow`, `serde`/`serde_core`, `serde_json`, `time`) and the separately reviewable Tauri family; refresh actual versions first.
2. **Wave 2:** actual pending risk-owned directs (`reqwest`, `tokio-util` in the 2026-08-12 snapshot) followed by one residual transitive-refresh commit; do not create empty Tokio/SQLx/Keyring slices.
3. **Wave 3:** incompatible `jsonschema` and MCP bridge ownership/version decisions as independent designs/slices.
4. **Wave 4:** Edition 2024 migration after dependency stabilization.
5. **Wave 5:** maintenance automation and optional commit-range enforcement after the first future compiler bump demonstrates need.

Each later plan begins from the current post-previous-wave manifests/index, records additions/updates/removals and direct owners, and applies the same duplicate-growth exception procedure and release executor requirements.
