# Rust Workspace Core Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task by task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish `src-tauri` as a Cargo workspace, move `error`, `time`, and
`compression` into a minimal `extractum-core` crate, preserve application
behavior and internal import paths, and record trustworthy before/after daily
loop measurements.

**Architecture:** Keep `src-tauri/Cargo.toml` as both the Tauri application
package and workspace root. Add `crates/extractum-core`, expose its three
modules through a curated `lib.rs`, and expose their required items through
three explicit private wrapper modules in the application crate so existing
`crate::...` consumers remain unchanged.
Make all canonical complete Cargo gates workspace-aware and prove that the 18
moved unit tests remain in the executed inventory.

**Tech Stack:** Rust 2021, Cargo workspaces, Tauri 2, serde, time, zstd,
PowerShell 5.1 on Windows, Node.js ESM, Vitest source-contract tests.

## Global Constraints

- Implement only the first slice of
  `docs/superpowers/specs/2026-07-15-rust-workspace-crate-extraction-design.md`.
- The approved spec and this plan must be committed, and the worktree must be
  clean, before baseline measurements begin.
- Do not implement the deferred `sql_helpers`, `tx`, `media_metadata`,
  `youtube::dto`, `notebooklm_export`, or any other domain extraction.
- Do not change application behavior, Tauri commands, managed state, database
  behavior, migrations, process lifecycle, capabilities, plugins, or UI.
- Keep `src-tauri/Cargo.toml` as the workspace root; do not create a repository
  root `Cargo.toml`.
- Keep the existing `[profile.dev]` and `[profile.dev.package."*"]` sections
  only in the workspace-root manifest and preserve their values.
- Keep all ordinary Cargo work in the canonical `src-tauri/target`; do not set
  `CARGO_TARGET_DIR`, use `--target-dir`, or run `cargo clean`.
- Use `npm.cmd`, not plain `npm`, for npm scripts on Windows.
- Preserve the existing application paths `crate::error`, `crate::time`, and
  `crate::compression` through explicit app-crate wrapper modules. Do not
  rewrite their consumers in this slice.
- The only required visibility expansions are the existing two
  `pub(crate)` error mappers, three time functions, and four compression
  functions. Keep `classify_message` private and keep test modules test-only.
- Do not add glob exports or a public-API snapshot dependency.
- Do not rewrite archived plans, archived verification evidence, or unrelated
  historical command examples.
- Preserve unrelated user changes. Inspect the worktree before every commit
  and stage only files owned by this plan.

## Measurement Constants

- Domain probe logical file:
  `src-tauri/src/notebooklm_export/chunker.rs`.
- Application-shell probe logical file: `src-tauri/src/lib.rs`.
- Probe operation: alternate one inert comment between two predetermined
  spellings, run one `cargo check`, and reverse it byte-for-byte after every
  sample. Each timed sample must begin from a checked, warm state and must
  cause Cargo to invoke the root crate compiler; five consecutive checks after
  one edit are not five incremental samples.
- Fast metrics: five warmed samples and median.
- `cargo test --no-run`: three samples.
- Production compile/package preparation uses
  `npm.cmd run tauri -- build --no-bundle`: two samples; a third is allowed
  only if the first two differ enough to affect the conclusion. Baseline MSI
  bundling was already broken before workspace changes (`light.exe` failed
  once and hung once), so WiX duration cannot provide a valid before/after
  signal for this slice. The no-bundle command produces the same release
  executable required by the production startup smoke.
- Every PowerShell block that reads or writes measurement evidence must reload
  `$scratch` from the absolute locator file
  `$env:TEMP/extractum-workspace-core-current.txt`; variables do not persist
  between tool calls or terminal sessions.
- Core extraction is enabling infrastructure. Record its timing result, but do
  not apply the later domain 25%/two-second success gate to this slice.
- A no-op regression is material only when it exceeds both 5% and 0.5 seconds.
- A shell-probe regression is material only when it exceeds both 5% and
  1 second.

---

### Task 1: Capture the Clean Pre-Workspace Baseline

**Files:**
- Read: `src-tauri/Cargo.toml`
- Read: `src-tauri/src/notebooklm_export/chunker.rs`
- Read: `src-tauri/src/lib.rs`
- Create later from temporary evidence:
  `docs/superpowers/verification/2026-07-15-rust-workspace-core-extraction.md`

**Produces:** A system-temporary evidence directory containing the starting
commit, environment, Cargo metadata, test inventory, two probe series, no-op
series, test compilation series, production build series, and Cargo timing
report references. No baseline artifact is written into the repository before
the source changes.

- [ ] **Step 1: Require the approved, clean starting state**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
$spec = 'docs/superpowers/specs/2026-07-15-rust-workspace-crate-extraction-design.md'
$plan = 'docs/superpowers/plans/2026-07-15-rust-workspace-core-extraction.md'
$specTracked = @(git ls-files --error-unmatch $spec 2>$null).Count -eq 1
$planTracked = @(git ls-files --error-unmatch $plan 2>$null).Count -eq 1
$head = (git rev-parse HEAD).Trim()
$manifestHash = (Get-FileHash 'src-tauri/Cargo.toml' -Algorithm SHA256).Hash
$domainHash = (Get-FileHash 'src-tauri/src/notebooklm_export/chunker.rs' -Algorithm SHA256).Hash
$shellHash = (Get-FileHash 'src-tauri/src/lib.rs' -Algorithm SHA256).Hash
"STATUS_COUNT=$($status.Count)"
"SPEC_TRACKED=$specTracked"
"PLAN_TRACKED=$planTracked"
"HEAD=$head"
"MANIFEST_SHA256=$manifestHash"
"DOMAIN_PROBE_SHA256=$domainHash"
"SHELL_PROBE_SHA256=$shellHash"
if ($status.Count -ne 0 -or -not $specTracked -or -not $planTracked) { exit 1 }
```

Expected: clean worktree; the approved spec and plan are tracked at `HEAD`;
all three hashes are recorded. If the files are still untracked, stop and ask
the user whether to commit the documentation before measuring.

- [ ] **Step 2: Record the machine, toolchain, profile, process, and target state**

Create an absolute scratch directory below `$env:TEMP`, outside the repository,
named with the starting commit. Record:

```powershell
$repo = (Resolve-Path '.').Path
$head = (git rev-parse HEAD).Trim()
$scratch = Join-Path $env:TEMP "extractum-workspace-core-$head"
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$scratchLocator = Join-Path $env:TEMP 'extractum-workspace-core-current.txt'
$scratch | Set-Content -LiteralPath $scratchLocator
$processes = @(Get-CimInstance Win32_Process | Where-Object {
  $_.Name -in @('cargo.exe','rustc.exe','rust-analyzer.exe','extractum.exe') -or
  ($_.Name -eq 'node.exe' -and $_.CommandLine -match '(?i)(vitest|vite(?:\.js)?|tauri)')
})
$metadata = cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --no-deps
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$metadataObject = $metadata | ConvertFrom-Json
$target = [System.IO.Path]::GetFullPath($metadataObject.target_directory)
$expectedTarget = [System.IO.Path]::GetFullPath((Join-Path $repo 'src-tauri/target'))
@(
  "HEAD=$head"
  "DATE=$([DateTimeOffset]::Now.ToString('o'))"
  "OS=$([System.Environment]::OSVersion.VersionString)"
  "PROCESSOR=$env:PROCESSOR_IDENTIFIER"
  "LOGICAL_CORES=$([Environment]::ProcessorCount)"
  "RUSTC=$(& rustc --version)"
  "CARGO=$(& cargo --version)"
  "TARGET=$target"
  "EXPECTED_TARGET=$expectedTarget"
  "ACTIVE_PROCESS_COUNT=$($processes.Count)"
) | Set-Content -LiteralPath (Join-Path $scratch 'environment.txt')
if ($processes.Count -ne 0 -or $target -ne $expectedTarget) { exit 1 }
```

Expected: no active Cargo/rustc/rust-analyzer/Node/Extractum process and Cargo
metadata resolves the target to `src-tauri/target`. If Node is active for an
unrelated editor tool, it is not selected by the command-line filter and does
not require an exception. Any selected process blocks measurement.

- [ ] **Step 3: Capture the complete pre-workspace test inventory**

Run and tee the full list into the scratch directory:

```powershell
$scratchLocator = Join-Path $env:TEMP 'extractum-workspace-core-current.txt'
if (-not (Test-Path -LiteralPath $scratchLocator)) { throw 'Scratch locator missing' }
$scratch = (Get-Content -LiteralPath $scratchLocator -Raw).Trim()
if (-not (Test-Path -LiteralPath $scratch)) { throw 'Scratch directory missing' }
$inventoryLog = Join-Path $scratch 'baseline-test-inventory.log'
& cargo test --manifest-path src-tauri/Cargo.toml --all-targets -- --list 2>&1 |
  Tee-Object -FilePath $inventoryLog
$inventoryExit = $LASTEXITCODE
$testNames = @(
  Get-Content -LiteralPath $inventoryLog |
    ForEach-Object { $_.ToString() } |
    Where-Object { $_ -match ': test$' } |
    ForEach-Object { ($_ -replace ': test$', '').Trim() }
)
$testNames | Sort-Object -Unique |
  Set-Content -LiteralPath (Join-Path $scratch 'baseline-test-names.txt')
@{
  exit = $inventoryExit
  count = $testNames.Count
  unique_count = @($testNames | Sort-Object -Unique).Count
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'baseline-test-inventory.json')
if ($inventoryExit -ne 0 -or $testNames.Count -eq 0 -or
    @($testNames | Sort-Object -Unique).Count -ne $testNames.Count) { exit 1 }
```

Expected: nonzero, unique test inventory and successful compilation. Keep the
exact count as observed; do not hard-code an older repository count in the
plan.

- [ ] **Step 4: Warm the canonical target and capture five no-op checks**

Reload `$scratch` from the required locator file.

Run one unrecorded warm-up `cargo check --manifest-path src-tauri/Cargo.toml
--all-targets`. It must pass. Then run five no-op checks sequentially, recording
wall milliseconds, exit code, and log for each run under `$scratch/noop`.

Each command is:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: five successful samples. Record the median; do not overlap commands.

- [ ] **Step 5: Capture five real incremental domain-probe samples**

Reload `$scratch` from the required locator file.

Use only `src-tauri/src/notebooklm_export/chunker.rs`. Record its starting
SHA-256. Alternate an inert comment immediately before its test module between
these predetermined forms:

```rust
// cargo-measurement-probe: a
```

```rust
// cargo-measurement-probe: b
```

For each of five samples:

1. Apply exactly one A/B transition with a focused patch.
2. Time `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`.
3. Require exit code zero and save the log and milliseconds.
4. Do not count a check that reports no rebuilt `extractum` unit.

After sample five, remove the probe with a focused reverse patch, run one
untimed restoring check, and confirm the original file hash.

Expected: five successful incremental compilations and byte-for-byte
restoration. Any unrelated source change invalidates this series.

- [ ] **Step 6: Capture five real incremental application-shell samples**

Reload `$scratch` from the required locator file.

Repeat Step 5 against `src-tauri/src/lib.rs`, placing the same A/B comment
immediately before `pub fn run()`. Restore the original bytes and verify the
starting hash after the fifth sample.

Expected: five successful shell samples measured independently from the domain
series.

- [ ] **Step 7: Capture test-compilation, test-execution, and production-build baselines**

Reload `$scratch` from the required locator file. Run three sequential
test-compilation samples of:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --no-run
```

Then run three sequential test-execution samples of:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Record command wall time and the test harness duration separately when libtest
reports it. Do not combine compilation and execution numbers.

Then run two sequential samples of:

```powershell
npm.cmd run tauri -- build --no-bundle
```

Record wall time, exit status, and logs. Run a third production build only if
the first two differ enough that the later comparison could change its
conclusion.

The two earlier full-bundle attempts are not timing samples: the first was a
cold release compile followed by a WiX error, and the second timed out in WiX.
Retain their logs only as environment evidence proving that MSI packaging was
already unavailable on the baseline commit. Capture two fresh successful
no-bundle samples before Task 2.

Expected: all builds pass. Stop on any failure; do not classify a failed build
as a timing sample.

- [ ] **Step 8: Capture a pre-workspace Cargo timing report**

Reload `$scratch` from the required locator file.

Apply one domain-probe comment, run:

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --all-targets --timings
```

Copy the newly generated timestamped timing HTML path and its SHA-256 into the
scratch evidence, then remove the comment and confirm the original source
hash. Do not delete or relocate the canonical timing report.

Expected: the report contains the root `extractum` unit and the repository is
clean again.

---

### Task 2: Add a RED Workspace and Verification Contract

**Files:**
- Create: `src/lib/rust-workspace-core-contract.test.ts`
- Read: `src/lib/development-loop-performance-contract.test.ts`
- Read: `src-tauri/Cargo.toml`
- Read: `src-tauri/src/lib.rs`
- Read: `package.json`
- Read: `scripts/verify.mjs`

**Contract:** Protect the workspace member, curated core surface, private app
facade, workspace-aware complete commands, canonical target policy, and absence
of glob exports. The contract reads files with normalized CRLF/LF and reports
all missing conditions in normal Vitest assertions.

- [ ] **Step 1: Create the failing source contract**

Create `src/lib/rust-workspace-core-contract.test.ts` using `node:fs` and
`node:path`. It must assert:

- `src-tauri/Cargo.toml` contains `[workspace]`, resolver `2`, both `"."` and
  `"crates/extractum-core"` members, `[workspace.dependencies]`, and the
  existing profile values;
- `src-tauri/crates/extractum-core/Cargo.toml` exists, names package
  `extractum-core`, and inherits `serde`, `time`, and `zstd` from the workspace;
- core `lib.rs` exposes exactly `error`, `time`, and `compression` with explicit
  `pub mod` declarations and contains no glob re-export;
- root `lib.rs` contains the three explicit private wrapper modules shown in
  Task 3 and no file-backed `mod error;`, `mod time;`, or `mod compression;`
  declarations;
- `package.json` owns the exact complete command
  `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets`,
  the focused prompt-pack command explicitly selects `-p extractum`, and
  `check:rustfmt` includes `--all`;
- `scripts/verify.mjs` passes `--workspace` and `--all-targets` to both Cargo
  check and Cargo test;
- active guidance names `src-tauri/target` and the workspace-aware check/test
  policy through stable marker sections, without matching archived documents.

Use an existence helper before reading the not-yet-created core files so RED
is an assertion failure rather than a module-resolution failure.

- [ ] **Step 2: Run the contract to verify RED**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/rust-workspace-core-contract.test.ts
```

Expected: FAIL and identify the absent workspace/core plus root-only Cargo
commands. A zero-test run is a failure of this step.

---

### Task 3: Establish the Workspace and Minimal Core

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/crates/extractum-core/Cargo.toml`
- Create: `src-tauri/crates/extractum-core/src/lib.rs`
- Create from existing source: `src-tauri/crates/extractum-core/src/error.rs`
- Create from existing source: `src-tauri/crates/extractum-core/src/time.rs`
- Create from existing source: `src-tauri/crates/extractum-core/src/compression.rs`
- Delete after successful copy: `src-tauri/src/error.rs`
- Delete after successful copy: `src-tauri/src/time.rs`
- Delete after successful copy: `src-tauri/src/compression.rs`

**Interfaces:**
- Produces package `extractum-core` and Rust crate `extractum_core`.
- Produces public modules `extractum_core::{error,time,compression}`.
- Preserves app-internal paths through three private wrapper modules with
  explicit item re-exports. Keeping `mod time { ... }` also preserves the
  current name-resolution shape for files that import the external `time`
  crate directly.
- Preserves all existing tests in their original module-local files.

- [ ] **Step 1: Add the package/workspace root without moving profiles**

In `src-tauri/Cargo.toml`:

1. Add `[workspace]` with members `"."` and `"crates/extractum-core"` and
   `resolver = "2"`.
2. Add `[workspace.package]` with `version = "0.2.0"` and `edition = "2021"`.
3. Add `[workspace.dependencies]` for the existing exact `serde`, `time`, and
   `zstd` declarations.
4. Change the root package's `serde`, `time`, and `zstd` dependency
   declarations to `workspace = true` because the application still uses all
   three directly. In particular, `analysis/trace.rs` calls
   `zstd::encode_all` and `zstd::decode_all` outside the moved compression
   module.
5. Add `extractum-core = { path = "crates/extractum-core" }`.
6. Leave both profile sections and every unrelated dependency unchanged.

Do not convert `analysis/trace.rs` to the compression facade in this mechanical
slice. Record that possible consolidation as backlog evidence only.

- [ ] **Step 2: Create the minimal core manifest and curated root**

Create `src-tauri/crates/extractum-core/Cargo.toml`:

```toml
[package]
name = "extractum-core"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
serde.workspace = true
time.workspace = true
zstd.workspace = true
```

Create `src-tauri/crates/extractum-core/src/lib.rs`:

```rust
pub mod compression;
pub mod error;
pub mod time;
```

Do not add glob re-exports or dependencies on Tauri, plugins, sqlx, grammers,
or any application domain.

- [ ] **Step 3: Move the three modules and expand only required visibility**

Copy the current files and tests byte-for-byte into the core source directory,
then make only these production visibility changes:

- `error::database_error`: `pub(crate)` to `pub`;
- `error::internal_error`: `pub(crate)` to `pub`;
- all three current `time` functions: `pub(crate)` to `pub`;
- all four current `compression` functions: `pub(crate)` to `pub`.

Keep `classify_message` private. Keep `#[cfg(test)]` modules and private test
helpers unchanged. Delete the three original root source files only after the
new files contain the full implementations and tests.

- [ ] **Step 4: Replace root module declarations with the compatibility facade**

In `src-tauri/src/lib.rs`, remove the three separate declarations:

```rust
mod compression;
mod error;
mod time;
```

Replace the file-backed declarations with three explicit private wrappers:

```rust
mod compression {
    pub(crate) use extractum_core::compression::{
        compress_json_bytes, compress_text, decompress_bytes, decompress_text,
    };
}

mod error {
    pub(crate) use extractum_core::error::{
        database_error, internal_error, AppError, AppErrorKind, AppResult,
    };
}

mod time {
    pub(crate) use extractum_core::time::{
        now_rfc3339_utc, now_secs, ymd_to_unix_midnight,
    };
}
```

Use explicit item lists rather than glob re-exports. This avoids introducing a
root `use` binding named `time`, preserves the current `mod time` resolution
shape, and keeps the application facade auditable.

Do not change any existing `crate::error`, `crate::time`, or
`crate::compression` consumer.

- [ ] **Step 5: Refresh Cargo metadata and lock data**

Run:

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --no-deps
```

Expected: exit 0; workspace members are exactly `extractum` and
`extractum-core`; target directory resolves to `src-tauri/target`; no ignored
profile warning. Allow Cargo to update only the workspace/package portions of
`src-tauri/Cargo.lock`; dependency versions must not upgrade.

- [ ] **Step 6: Run focused core checks before broader workflow edits**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-core --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-core --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: all pass; core executes 18 unit tests; the application compiles
without consumer import edits.

---

### Task 4: Make Complete Workflows Workspace-Aware

**Files:**
- Modify: `package.json`
- Modify: `scripts/verify.mjs`
- Modify: `src/lib/development-loop-performance-contract.test.ts`
- Modify: `src/lib/rust-workspace-core-contract.test.ts`
- Modify: `AGENTS.md`
- Modify: `docs/project.md`

**Interfaces:**
- Canonical full Rust test command:
  `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets`.
- Canonical full Rust check command:
  `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`.
- Focused prompt-pack tests explicitly select `-p extractum`.
- Rustfmt checks the complete workspace with `--all`.

- [ ] **Step 1: Update package scripts**

Set:

```json
"test:rust": "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets",
"test:rust:prompt-pack-runs": "cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_pack_run",
"check:rustfmt": "cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check"
```

Do not change unrelated scripts.

- [ ] **Step 2: Update the authoritative verification pipeline**

In `scripts/verify.mjs`, add `--workspace` and `--all-targets` to both Cargo
steps and update their displayed titles. Preserve step order, error handling,
formatting, and all frontend checks.

- [ ] **Step 3: Update existing and new source contracts**

Update `src/lib/development-loop-performance-contract.test.ts` to expect the
new exact package-script strings. Complete the new workspace contract so it
checks final files rather than only their absence. Keep CRLF/LF normalization.

- [ ] **Step 4: Update active developer guidance**

In `AGENTS.md` and `docs/project.md`:

- preserve the existing `<!-- daily-development-loop -->` marker;
- state that canonical full Rust checks/tests use `--workspace --all-targets`;
- retain focused `-p extractum` guidance for root-only filters;
- state that every member shares `src-tauri/target`;
- do not rewrite archived plans, archived evidence, or specialized historical
  troubleshooting transcripts.

- [ ] **Step 5: Audit every active Cargo check/test invocation**

Run a repository search excluding `node_modules`, `src-tauri/target`, archived
Superpowers material, approved historical plans/specs, and verification
evidence. Classify each remaining invocation as:

- canonical complete gate: must use `--workspace --all-targets`;
- focused root-package command: must explicitly use `-p extractum` or be clearly
  documented as focused;
- historical/specialized evidence: leave unchanged and record why.

Expected: `package.json`, `scripts/verify.mjs`, `AGENTS.md`, and
`docs/project.md` are corrected; no active command claims complete verification
while selecting only the root package.

- [ ] **Step 6: Run the contracts to verify GREEN**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/rust-workspace-core-contract.test.ts src/lib/development-loop-performance-contract.test.ts
```

Expected: both files pass with a nonzero test count.

---

### Task 5: Prove Workspace Test Inventory, Correctness, and Tauri Startup

**Files:**
- Read: baseline inventory under the system-temporary evidence directory
- Read: all files changed by Tasks 2-4

- [ ] **Step 1: Enumerate the complete post-workspace inventory**

Run:

```powershell
$scratchLocator = Join-Path $env:TEMP 'extractum-workspace-core-current.txt'
if (-not (Test-Path -LiteralPath $scratchLocator)) { throw 'Scratch locator missing' }
$scratch = (Get-Content -LiteralPath $scratchLocator -Raw).Trim()
if (-not (Test-Path -LiteralPath $scratch)) { throw 'Scratch directory missing' }
$postLog = Join-Path $scratch 'post-test-inventory.log'
& cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- --list 2>&1 |
  Tee-Object -FilePath $postLog
$postExit = $LASTEXITCODE
$postNames = @(
  Get-Content -LiteralPath $postLog |
    ForEach-Object { $_.ToString() } |
    Where-Object { $_ -match ': test$' } |
    ForEach-Object { ($_ -replace ': test$', '').Trim() }
)
$baselineNames = @(Get-Content -LiteralPath (Join-Path $scratch 'baseline-test-names.txt'))
$missing = @($baselineNames | Where-Object { $_ -notin $postNames })
$movedCorePatterns = @(
  'classify_message_treats_dialog_lookup_misses_as_not_found',
  'now_rfc3339_utc_returns_current_utc_timestamp',
  'text_roundtrip_through_zstd'
)
$missingCoreSentinels = @($movedCorePatterns | Where-Object {
  $needle = $_
  -not ($postNames | Where-Object { $_ -like "*$needle" })
})
@{
  exit = $postExit
  baseline_count = $baselineNames.Count
  post_count = $postNames.Count
  missing_count = $missing.Count
  missing_core_sentinel_count = $missingCoreSentinels.Count
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'post-test-inventory.json')
if ($postExit -ne 0 -or $postNames.Count -lt $baselineNames.Count -or
    $missing.Count -ne 0 -or $missingCoreSentinels.Count -ne 0) { exit 1 }
```

Expected: post count is unchanged or larger; no baseline test name is missing;
all three core modules are represented. If package movement changes a displayed
prefix while preserving the test identity, compare normalized suffixes and
document the normalization rather than weakening the count gate.

- [ ] **Step 2: Run focused and complete Rust correctness checks**

Run:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-core --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

Expected: all pass, no warnings from owned code, and the complete command runs
both packages.

- [ ] **Step 3: Verify dependency and profile boundaries mechanically**

Inspect `cargo metadata` and manifests. Fail if:

- core depends on Tauri, any Tauri plugin, sqlx, grammers, or an application
  path dependency other than workspace-owned shared dependencies;
- a member manifest contains `[profile.`;
- Cargo reports an ignored profile;
- the resolved target directory differs from `src-tauri/target`;
- `Cargo.lock` shows an unexpected external version change.

Expected: core's external dependency closure begins from only serde, time, and
zstd; profiles remain root-owned.

- [ ] **Step 4: Run the complete repository gate**

Run:

```powershell
npm.cmd run verify
```

Expected: frontend tests/checks, workspace rustfmt/check/tests, and diff check
all pass. Confirm from output that both Cargo steps display `--workspace
--all-targets`.

- [ ] **Step 5: Run the normal Tauri development startup smoke**

Start the canonical MCP-enabled development workflow:

```powershell
npm.cmd run tauri dev
```

Expected: Vite and the Tauri application start normally from the workspace;
the main window renders and one ordinary navigation action works. Record any
console/build error. Close the application normally and confirm the Tauri,
Vite, and Extractum processes have exited before continuing. Do not leave this
session running during the release smoke.

- [ ] **Step 6: Launch and smoke the built production application**

Use the release artifact produced by the latest successful
`npm.cmd run tauri -- build --no-bundle`. Resolve the actual executable from
the build output; on the current Windows target the expected path is
`src-tauri/target/release/extractum.exe`. Launch it visibly, confirm the main
window renders and basic navigation works, then close it normally and confirm
the process exits.

Expected: production startup succeeds under the packaged configuration. Build
success without launching the executable does not satisfy this step.

- [ ] **Step 7: Review implementation scope before the measurement checkpoint**

Run `git diff --check` and inspect `git status --short --untracked-files=all`.
The implementation diff is limited to:

```text
AGENTS.md
docs/project.md
package.json
scripts/verify.mjs
src/lib/development-loop-performance-contract.test.ts
src/lib/rust-workspace-core-contract.test.ts
src-tauri/Cargo.lock
src-tauri/Cargo.toml
src-tauri/crates/extractum-core/Cargo.toml
src-tauri/crates/extractum-core/src/lib.rs
src-tauri/crates/extractum-core/src/error.rs
src-tauri/crates/extractum-core/src/time.rs
src-tauri/crates/extractum-core/src/compression.rs
src-tauri/src/compression.rs
src-tauri/src/error.rs
src-tauri/src/lib.rs
src-tauri/src/time.rs
```

The three old root module files appear only as deletions. Confirm that module
implementations/tests moved without behavioral edits, only the nine planned
functions expanded from `pub(crate)` to `pub`, no root consumer paths were
mass-rewritten, and `Cargo.lock` contains no external version upgrade.

- [ ] **Step 8: Commit the verified implementation checkpoint**

Stage only the paths listed in Step 7, including the three deletions, and run:

```powershell
git commit -m "refactor: establish extractum core workspace"
```

Expected: commit succeeds and `git status --short` is empty. Record the commit
hash as the implementation commit. Do not push.

---

### Task 6: Re-measure the Same Two Probes

**Files:**
- Temporarily modify and restore:
  `src-tauri/src/notebooklm_export/chunker.rs`
- Temporarily modify and restore: `src-tauri/src/lib.rs`
- Read/write only system-temporary measurement evidence

- [ ] **Step 1: Confirm the implementation state is stable before timing**

Require:

- no blocking process selected by the Task 1 command-line process filter;
- both probe files match their expected implementation hashes;
- `cargo metadata` resolves `src-tauri/target`;
- the implementation commit from Task 5 is checked out and the worktree is
  clean;
- no pending formatter changes.

Do not mix compilation caused by unfinished edits with measurement samples.

- [ ] **Step 2: Repeat five no-op checks**

Use the new canonical command:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

Run one unrecorded warm-up followed by five recorded sequential samples.

- [ ] **Step 3: Repeat the five domain-probe samples**

Use the same logical `notebooklm_export/chunker.rs`, same A/B comments, same
placement, and the workspace-aware check command. The file has not moved in
this first core slice. Restore it byte-for-byte afterward.

- [ ] **Step 4: Repeat the five application-shell samples**

Use the same `src-tauri/src/lib.rs` A/B probe and workspace-aware check command.
Restore it byte-for-byte afterward.

- [ ] **Step 5: Repeat test-compilation, test-execution, and production-build samples**

Reload `$scratch` from the required locator file. Run three test-compilation
samples of:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets --no-run
```

Then run three test-execution samples of:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

Record command wall time and harness duration separately.

Run two samples of:

```powershell
npm.cmd run tauri -- build --no-bundle
```

Use the same third-run rule as baseline. Every sample must pass.

- [ ] **Step 6: Capture the post-workspace timing report**

Apply the same domain probe and run:

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --workspace --all-targets --timings
```

Record the new report path/hash and compare the root `extractum` and new
`extractum-core` units. Restore the source hash.

- [ ] **Step 7: Compute the predeclared comparisons**

Produce medians and deltas for:

- no-op check;
- domain incremental probe;
- application-shell incremental probe;
- test compilation;
- test execution;
- production build.

Classify no-op as a material regression only when both `>5%` and `>0.5s`.
Classify shell-probe regression only when both `>5%` and `>1s`. Record the
domain-probe result without applying the later 25%/two-second domain gate.

Expected: measurements are complete and honestly reported even if the minimal
core slice is neutral or slower.

---

### Task 7: Record and Commit Verification Evidence

**Files:**
- Create:
  `docs/superpowers/verification/2026-07-15-rust-workspace-core-extraction.md`

- [ ] **Step 1: Write the verification record**

Include:

- starting and implementation commit hashes;
- machine/toolchain/profile/target environment;
- exact two probe files and comment operation;
- raw sample tables and medians;
- before/after test counts and moved-core test evidence;
- workspace members and dependency boundary;
- Cargo timing report paths and interpretation;
- focused, workspace, repository, and Tauri build results;
- development and production startup smoke results;
- the pre-existing baseline WiX failure/hang, the exclusion of MSI packaging
  from this slice's before/after gate, and the separate follow-up status;
- no-op and shell regression-gate outcomes;
- an explicit statement that the later domain stop/go gate does not apply to
  this enabling slice;
- deferred `sql_helpers`/`tx`, `media_metadata`, YouTube DTO, and test-support
  follow-ups without implementing them.
- the optional later consolidation of direct `analysis/trace.rs` zstd calls
  behind the shared compression API, without claiming it belongs to this
  mechanical slice.

- [ ] **Step 2: Run final hygiene and evidence-scope checks**

Run:

```powershell
npm.cmd run check:rustfmt
npm.cmd run verify
git diff --check
git status --short --untracked-files=all
```

Expected changed paths are limited to:

```text
docs/superpowers/verification/2026-07-15-rust-workspace-core-extraction.md
```

No probe comment, temporary log, alternate target, build artifact, implementation
source edit, or unrelated change may remain.

- [ ] **Step 3: Review the verification diff**

Confirm:

- every recorded command and count agrees with the temporary raw evidence;
- the document distinguishes baseline and implementation commits;
- test inventory and moved-core evidence agree;
- no performance claim applies the later domain gate to this core slice;
- limitations and any noisy samples are reported rather than omitted.

- [ ] **Step 4: Commit the verification record**

Stage only the verification document and commit:

```powershell
git add docs/superpowers/verification/2026-07-15-rust-workspace-core-extraction.md
git commit -m "docs: record rust workspace core verification"
```

Expected: commit succeeds and `git status --short` is empty. Do not push unless
the user asks.
