# Extractum Process Crate Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract shared external-process admission, hidden-child launch, and
process-tree containment into `extractum-process` without rewriting any current
application consumer.

**Architecture:** Add one lower-level workspace crate containing the current
`external_process`, `child_process`, and `process_tree` implementations. Keep
three private glob facades in the application crate so YouTube, Gemini Browser,
diagnostics, and shutdown wiring retain their existing `crate::...` paths.
Treat focused-package timing as diagnostic evidence; retain the architectural
slice only when correctness passes and the application-shell regression cap is
met.

**Tech Stack:** Rust 2021, Cargo workspaces, Tokio, parking_lot, anyhow,
Windows Job Objects through windows-sys, Vitest 4 source contracts, PowerShell
5.1 on Windows.

## Global Constraints

- Implement only Phase 3 from
  `docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md`.
- Do not begin Gemini Browser internal preparation or create
  `extractum-gemini-browser` in this plan.
- Move exactly `external_process.rs`, `child_process.rs`, and
  `process_tree.rs`; `job_helpers.rs` remains app-side.
- Phase 3 is architecturally justified. Its focused-package measurement is
  diagnostic, not a retention gate.
- Domain probes use an explicit package (`-p ... --all-targets`); application
  shell probes use the normative full workspace (`--workspace --all-targets`).
  Every timed probe is preceded by an untimed no-op check against the restored
  tree so restoration work cannot leak into the next sample.
- Protocol roots and baseline/post/repeat attempts use timestamp/GUID IDs.
  Infrastructure failures preserve an `invalid*.json` marker and their raw
  logs; retries create a new attempt and update the relevant locator.
- Retention requires all correctness gates and application-shell regression
  no greater than both 5% and 0.5 seconds.
- A primary shell result that fails the retention cap but is no worse than
  both 8% and 0.8 seconds is marginal and consumes the one predeclared repeat.
  Results beyond either marginal cap fail without a repeat.
- Preserve current application consumer paths through private app-side glob
  facades. Do not edit consumer files in the extraction candidate.
- `extractum-process` direct dependency roots are exactly `anyhow`,
  `parking_lot`, `tokio`, and target-specific `windows-sys`. Tokio test-util
  is a dev feature of the same dependency.
- `extractum-process/src/lib.rs` contains only three named public modules and
  no root glob export.
- Enumerate every `pub(crate)` visibility decision. Do not expose test-only
  seams or mechanically widen the entire files.
- Preserve all 20 process test names without duplication or loss.
- Phase 3 acceptance runs only on the canonical Windows/MSVC environment. Do
  not install or invoke a Linux Rust target for this slice.
- Move the existing `#[cfg(not(windows))]` process-tree stub byte-for-byte, but
  do not treat its compilation as Phase 3 acceptance evidence.
- Use canonical `src-tauri/target`; do not set `CARGO_TARGET_DIR`, pass
  `--target-dir`, or run `cargo clean`.
- Use `npm.cmd`, not plain `npm`, for npm scripts on Windows.
- Do not change migrations, Tauri commands, UI code, value-registry entries,
  package scripts, build tooling, or MSI/WiX configuration.
- Full release evidence uses
  `npm.cmd run tauri -- build --no-bundle`; MSI remains excluded due to the
  documented pre-existing WiX failure.
- Inspect the dirty worktree before each commit and stage only files owned by
  this plan.

## File Map

**Create:**

- `src-tauri/crates/extractum-process/Cargo.toml` — minimal package manifest.
- `src-tauri/crates/extractum-process/src/lib.rs` — curated three-module root.
- `src-tauri/crates/extractum-process/src/external_process.rs` — shutdown and
  admission coordinator moved from the app.
- `src-tauri/crates/extractum-process/src/child_process.rs` — hidden-console
  helper moved from the app.
- `src-tauri/crates/extractum-process/src/process_tree.rs` — Windows job-object
  containment and non-Windows stub moved from the app.
- `src/lib/process-crate-boundary-contract.test.ts` — workspace, dependency,
  visibility, facade, moved-test, and unchanged-consumer contract.
- `docs/superpowers/verification/2026-07-17-extractum-process-extraction.md` —
  literal measurement and verification evidence.

**Modify:**

- `src-tauri/Cargo.toml` — workspace member/dependencies and app inheritance.
- `src-tauri/Cargo.lock` — new local package graph.
- `src-tauri/src/lib.rs` — replace three file modules with private facades.
- `src/lib/external-process-lifecycle-contract.test.ts` — read implementations
  from the new crate and assert the private facade.
- `src/lib/hidden-child-process-contract.test.ts` — read the new helper and
  assert the private facade/public cross-crate constant.
- `docs/superpowers/specs/2026-07-17-crate-roadmap.md` — record the literal
  retained or not-retained Phase 3 outcome.

**Delete after byte-preserving moves:**

- `src-tauri/src/external_process.rs`
- `src-tauri/src/child_process.rs`
- `src-tauri/src/process_tree.rs`

## Rust Verification Loops

The boundary contract supplies RED before the crate exists:

```powershell
npm.cmd run test -- src/lib/process-crate-boundary-contract.test.ts
```

The narrow post-move Rust GREEN test is:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-process --lib external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets -- --exact
```

Focused package checks and checkpoint:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-process --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-process --all-targets
```

The immediate dependent-package checkpoint required for the new public
cross-crate interface is:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Measurement scope is deliberately asymmetric:

```powershell
# Domain before/after (with the owning package substituted after extraction)
cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets

# Application shell before/after
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

End-of-slice completion gates are:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

---

### Task 1: Establish Baseline, Windows Environment, Inventory, and Measurements

**Files:**

- Read: `docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md`
- Read: `docs/superpowers/plans/2026-07-17-extractum-process-extraction.md`
- Read: `src-tauri/src/external_process.rs`
- Read: `src-tauri/src/lib.rs`
- Create temporary artifacts only below `$env:TEMP`

**Interfaces:**

- Produces an absolute scratch locator at
  `$env:TEMP/extractum-process-current.txt`.
- Produces `baseline-test-names.txt`, `process-test-names.txt`,
  `consumer-hashes.json`, `environment.txt`, `baseline-summary.json`, and an
  executable byte-restoring probe runner in scratch.
- Provides the runner interface consumed by Task 3:
  `invoke-cargo-probe.ps1 -Path <absolute-or-repo-path> -Mode
  <focused|workspace> [-Package <name>] -Label <unique-label>
  -ExpectedSha256 <hash> -Scratch <absolute-path>`. Focused probes require a
  package; workspace probes reject one.

- [ ] **Step 1: Require a clean, committed, approved starting point**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
$spec = 'docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md'
$plan = 'docs/superpowers/plans/2026-07-17-extractum-process-extraction.md'
$specTracked = @(git ls-files --error-unmatch $spec 2>$null).Count -eq 1
$planTracked = @(git ls-files --error-unmatch $plan 2>$null).Count -eq 1
"STATUS_COUNT=$($status.Count)"
"SPEC_TRACKED=$specTracked"
"PLAN_TRACKED=$planTracked"
"HEAD=$((git rev-parse HEAD).Trim())"
if ($status.Count -ne 0 -or -not $specTracked -or -not $planTracked) { exit 1 }
```

Expected: clean tree and both approved documents tracked. Stop instead of
mixing unrelated work into the slice.

- [ ] **Step 2: Require an idle Rust/Tauri toolchain**

Run:

```powershell
$names = @('cargo', 'rustc', 'rust-analyzer', 'extractum')
$active = @(Get-Process -ErrorAction SilentlyContinue | Where-Object {
  $_.ProcessName -in $names -or $_.ProcessName -like 'cargo-*'
})
$active | Select-Object Id, ProcessName
if ($active.Count -ne 0) { exit 1 }
```

Expected: no matching process. Close the editor if rust-analyzer immediately
respawns Cargo; do not kill unrelated user processes automatically.

- [ ] **Step 3: Confirm the canonical Windows/MSVC environment before editing**

Run:

```powershell
$isWindows = [Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
  [Runtime.InteropServices.OSPlatform]::Windows
)
$hostLine = (& rustc -vV | Select-String '^host:').Line
$rustHost = ($hostLine -replace '^host:\s*', '').Trim()
"IS_WINDOWS=$isWindows HOST=$rustHost"
if (-not $isWindows -or $rustHost -ne 'x86_64-pc-windows-msvc') { exit 1 }
```

Expected: `IS_WINDOWS=True HOST=x86_64-pc-windows-msvc`. A different host is
a blocker for this Windows-only Phase 3 plan; do not substitute a cross-target
check.

- [ ] **Step 4: Create the isolated measurement scratch and environment record**

Run:

```powershell
$head = (git rev-parse HEAD).Trim()
$sessionId = "{0}-{1}" -f ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')),
  ([guid]::NewGuid().ToString('N'))
$scratch = Join-Path $env:TEMP "extractum-process-$head-$sessionId"
New-Item -ItemType Directory -Path $scratch | Out-Null
New-Item -ItemType Directory -Path (Join-Path $scratch 'measurements') | Out-Null
$locator = Join-Path $env:TEMP 'extractum-process-current.txt'
$locatorTemp = "$locator.$sessionId.tmp"
$scratch | Set-Content -LiteralPath $locatorTemp
if (Test-Path -LiteralPath $locator) {
  [IO.File]::Replace($locatorTemp, $locator, $null)
} else {
  Move-Item -LiteralPath $locatorTemp -Destination $locator
}

$defender = try {
  Get-MpComputerStatus |
    Select-Object AMServiceEnabled, AntivirusEnabled, RealTimeProtectionEnabled |
    ConvertTo-Json -Compress
} catch {
  "unavailable: $($_.Exception.Message)"
}
$power = try { (powercfg /getactivescheme | Out-String).Trim() } catch { "unavailable" }
@(
  "head=$head"
  "session_id=$sessionId"
  "started_at=$([DateTimeOffset]::Now.ToString('o'))"
  "cargo=$((& cargo -V) | Out-String).Trim()"
  "rustc=$((& rustc -Vv) | Out-String).Trim()"
  "target_dir=$((Resolve-Path 'src-tauri/target').Path)"
  "power=$power"
  "defender=$defender"
) | Set-Content -LiteralPath (Join-Path $scratch 'environment.txt')
```

Expected: one new absolute scratch directory outside the repository. If
`src-tauri/target` does not yet exist, run one no-op app check first and repeat
this step; do not invent another target directory. Every invocation creates a
new timestamp/GUID path and atomically replaces the locator. Never delete or
reuse an older scratch root. When restarting the whole protocol, first write
an `invalid-session.json` with the failure stage, reason, and timestamp into
the old root, then rerun this step; the old evidence remains available but is
excluded from all summaries.

Use this explicit whole-protocol restart only before a valid baseline summary
exists (later baseline/post/repeat failures use their attempt-level restart):

```powershell
$locator = Join-Path $env:TEMP 'extractum-process-current.txt'
$oldScratch = (Get-Content $locator -Raw).Trim()
if (Test-Path (Join-Path $oldScratch 'baseline-summary.json')) {
  throw 'Use the attempt-level restart; a valid baseline already exists'
}
[ordered]@{
  classification = 'infrastructure_failure'
  stage = 'pre_baseline'
  invalidated_at = [DateTimeOffset]::Now.ToString('o')
  reason = 'pre-baseline setup or runner infrastructure failed'
} | ConvertTo-Json | Set-Content (Join-Path $oldScratch 'invalid-session.json')
```

After this marker is written, rerun Step 4. Its new GUID root replaces the
locator without deleting or overwriting `$oldScratch`.

- [ ] **Step 5: Capture the full baseline test inventory and exact process set**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$stdout = Join-Path $scratch 'baseline-inventory.stdout.log'
$stderr = Join-Path $scratch 'baseline-inventory.stderr.log'
$cargoExe = (Get-Command cargo.exe).Source
$args = @(
  'test', '--manifest-path', 'src-tauri/Cargo.toml', '--workspace',
  '--all-targets', '--', '--list'
)
$process = Start-Process -FilePath $cargoExe -ArgumentList $args -Wait -PassThru `
  -NoNewWindow -RedirectStandardOutput $stdout -RedirectStandardError $stderr
$lines = @(
  @(Get-Content -LiteralPath $stdout -ErrorAction SilentlyContinue)
  @(Get-Content -LiteralPath $stderr -ErrorAction SilentlyContinue)
) | ForEach-Object { $_.ToString() }
$lines | Set-Content -LiteralPath (Join-Path $scratch 'baseline-inventory.log')
$names = @($lines | Where-Object { $_ -match ': test$' } |
  ForEach-Object { ($_ -replace ': test$', '').Trim() })
$unique = @($names | Sort-Object -Unique)
$processNames = @($unique | Where-Object {
  $_ -match '^(external_process|child_process|process_tree)::'
})
$unique | Set-Content -LiteralPath (Join-Path $scratch 'baseline-test-names.txt')
$processNames | Set-Content -LiteralPath (Join-Path $scratch 'process-test-names.txt')
@{
  exit = $process.ExitCode
  count = $names.Count
  unique_count = $unique.Count
  process_count = $processNames.Count
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'baseline-inventory.json')
if ($process.ExitCode -ne 0 -or $names.Count -eq 0 -or
    $unique.Count -ne $names.Count -or $processNames.Count -ne 20) { exit 1 }
```

Expected: a nonzero unique workspace inventory and exactly 20 process tests.
The observed total is evidence, not a permanent hard-coded constant.

- [ ] **Step 6: Verify the exact 20-name baseline**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$expected = @(
  'child_process::tests::create_no_window_matches_win32_process_creation_flags'
  'external_process::tests::admission_wait_consumes_the_shared_graceful_budget'
  'external_process::tests::cleanup_tasks_start_concurrently_and_isolate_error_and_panic'
  'external_process::tests::concurrent_watchdogs_invoke_exit_once'
  'external_process::tests::exhausted_admission_budget_skips_the_cleanup_factory'
  'external_process::tests::injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback'
  'external_process::tests::permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown'
  'external_process::tests::permits_acquired_before_shutdown_are_waited_for'
  'external_process::tests::repeated_start_does_not_replace_code_or_schedule_again'
  'external_process::tests::start_reports_completed_after_watchdog_claims_exit'
  'external_process::tests::start_returns_started_and_schedules_one_watchdog'
  'external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets'
  'external_process::tests::watchdog_exits_with_the_preserved_code_unless_cleanup_completed'
  'process_tree::tests::assigns_a_directly_owned_std_child'
  'process_tree::tests::creates_a_job_object'
  'process_tree::tests::dropping_the_guard_closes_the_job_and_kills_its_children'
  'process_tree::tests::process_tree_guard_can_be_owned_by_async_application_state'
  'process_tree::tests::terminate_failure_remains_reportable_and_retryable'
  'process_tree::tests::terminate_is_idempotent'
  'process_tree::tests::terminates_a_descendant_created_after_assignment'
)
$actual = @(Get-Content -LiteralPath (Join-Path $scratch 'process-test-names.txt'))
$missing = @($expected | Where-Object { $_ -notin $actual })
$extra = @($actual | Where-Object { $_ -notin $expected })
"MISSING=$($missing.Count) EXTRA=$($extra.Count)"
if ($missing.Count -ne 0 -or $extra.Count -ne 0) { exit 1 }
```

Expected: `MISSING=0 EXTRA=0`.

- [ ] **Step 7: Snapshot every consumer that the candidate must not edit**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$consumerPaths = @(
  'src-tauri/src/diagnostics/runtime.rs'
  'src-tauri/src/gemini_browser/cdp_chrome.rs'
  'src-tauri/src/gemini_browser/commands.rs'
  'src-tauri/src/gemini_browser/sidecar.rs'
  'src-tauri/src/youtube/captions.rs'
  'src-tauri/src/youtube/comments.rs'
  'src-tauri/src/youtube/jobs.rs'
  'src-tauri/src/youtube/metadata.rs'
  'src-tauri/src/youtube/preview.rs'
  'src-tauri/src/youtube/process_runtime.rs'
  'src-tauri/src/youtube/runtime.rs'
  'src-tauri/src/youtube/ytdlp.rs'
)
$hashes = @($consumerPaths | ForEach-Object {
  [pscustomobject]@{
    path = $_
    sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash
  }
})
$hashes | ConvertTo-Json -Depth 3 |
  Set-Content -LiteralPath (Join-Path $scratch 'consumer-hashes.json')
if ($hashes.Count -ne 12) { exit 1 }
```

Expected: 12 path/hash records. `src-tauri/src/lib.rs` is intentionally absent
because it becomes the compatibility facade owner.

- [ ] **Step 8: Create the byte-safe Cargo probe runner in scratch**

Write `$scratch/invoke-cargo-probe.ps1` with this complete content:

```powershell
param(
  [Parameter(Mandatory = $true)][string]$Path,
  [Parameter(Mandatory = $true)]
  [ValidateSet('focused', 'workspace')][string]$Mode,
  [string]$Package = '',
  [Parameter(Mandatory = $true)][string]$Label,
  [Parameter(Mandatory = $true)][string]$ExpectedSha256,
  [Parameter(Mandatory = $true)][string]$Scratch
)

$ErrorActionPreference = 'Stop'
$metaPath = Join-Path $Scratch "runs/$Label-meta.json"
$syncStdoutPath = Join-Path $Scratch "runs/$Label.sync.stdout.log"
$syncStderrPath = Join-Path $Scratch "runs/$Label.sync.stderr.log"
$stdoutPath = Join-Path $Scratch "runs/$Label.stdout.log"
$stderrPath = Join-Path $Scratch "runs/$Label.stderr.log"
if (Test-Path -LiteralPath $metaPath) { throw "Duplicate label: $Label" }
if ($Mode -eq 'focused' -and [string]::IsNullOrWhiteSpace($Package)) {
  throw 'Focused mode requires -Package'
}
if ($Mode -eq 'workspace' -and -not [string]::IsNullOrWhiteSpace($Package)) {
  throw 'Workspace mode must not receive -Package'
}

$resolved = (Resolve-Path -LiteralPath $Path).Path
$startingHash = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
if ($startingHash -ne $ExpectedSha256) {
  throw "Starting hash mismatch for $Path"
}
$original = [IO.File]::ReadAllBytes($resolved)
$suffix = [Text.Encoding]::UTF8.GetBytes("`n// cargo-measurement-probe: $Label`n")
$combined = New-Object byte[] ($original.Length + $suffix.Length)
[Array]::Copy($original, 0, $combined, 0, $original.Length)
[Array]::Copy($suffix, 0, $combined, $original.Length, $suffix.Length)

$meta = [ordered]@{
  label = $Label
  path = $Path
  mode = $Mode
  package = $Package
  sync_started = $false
  sync_exit_code = $null
  started = $false
  completed = $false
  restored = $false
  exit_code = $null
  elapsed_ms = $null
  starting_sha256 = $startingHash
  error = $null
}

try {
  $cargoExe = (Get-Command cargo.exe).Source
  $arguments = @('check', '--manifest-path', 'src-tauri/Cargo.toml')
  if ($Mode -eq 'focused') {
    $arguments += @('-p', $Package)
  } else {
    $arguments += '--workspace'
  }
  $arguments += '--all-targets'

  # Synchronize Cargo to the restored tree outside the timed sample. Without
  # this no-op check, alternating probes can charge restoration work to the
  # next logical edit.
  $sync = Start-Process -FilePath $cargoExe -ArgumentList $arguments -Wait `
    -PassThru -NoNewWindow -RedirectStandardOutput $syncStdoutPath `
    -RedirectStandardError $syncStderrPath
  $meta.sync_started = $true
  $meta.sync_exit_code = $sync.ExitCode
  if ($sync.ExitCode -ne 0) {
    throw "Untimed synchronization check failed with $($sync.ExitCode)"
  }

  [IO.File]::WriteAllBytes($resolved, $combined)
  $watch = [Diagnostics.Stopwatch]::StartNew()
  $process = Start-Process -FilePath $cargoExe -ArgumentList $arguments -Wait `
    -PassThru -NoNewWindow -RedirectStandardOutput $stdoutPath `
    -RedirectStandardError $stderrPath
  $watch.Stop()
  $meta.started = $true
  $meta.exit_code = $process.ExitCode
  $meta.elapsed_ms = $watch.ElapsedMilliseconds
} catch {
  $meta.error = $_.Exception.Message
} finally {
  [IO.File]::WriteAllBytes($resolved, $original)
  $meta.restored =
    ((Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash -eq $ExpectedSha256)
  $meta.completed = $true
  $meta | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $metaPath
}

if (-not $meta.completed -or -not $meta.restored -or -not $meta.sync_started) { exit 2 }
if ($meta.sync_exit_code -ne 0) { exit 1 }
if (-not $meta.started) { exit 2 }
if ($meta.exit_code -ne 0) { exit 1 }
exit 0
```

Save the shown block verbatim as UTF-8 using the execution environment's
file-editing mechanism, then verify it without changing repository files:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-cargo-probe.ps1'
if (-not (Test-Path -LiteralPath $runner)) { exit 1 }
$runnerText = Get-Content -LiteralPath $runner -Raw
if ($runnerText -notmatch 'cargo-measurement-probe' -or
    $runnerText -notmatch '\[IO\.File\]::WriteAllBytes' -or
    $runnerText -notmatch "ValidateSet\('focused', 'workspace'\)" -or
    $runnerText -notmatch 'sync_started') { exit 1 }
```

Expected: the runner exists only under scratch. This is the only plan step
whose content is written outside the repository from the shown block.

- [ ] **Step 9: Record baseline probe source hashes**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$sources = [ordered]@{
  domain = 'src-tauri/src/external_process.rs'
  shell = 'src-tauri/src/lib.rs'
}
foreach ($entry in $sources.GetEnumerator()) {
  @{
    path = $entry.Value
    sha256 = (Get-FileHash -LiteralPath $entry.Value -Algorithm SHA256).Hash
  } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch "baseline-$($entry.Key)-source.json")
}
```

Expected: two source records whose hashes match the clean committed tree.

- [ ] **Step 10: Run warm-ups, primary baselines, and the shell repeat reserve**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-cargo-probe.ps1'
$attemptId = "baseline-{0}-{1}" -f ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')),
  ([guid]::NewGuid().ToString('N'))
$attempt = Join-Path $scratch "measurements/$attemptId"
New-Item -ItemType Directory -Path (Join-Path $attempt 'runs') -Force | Out-Null
$attempt | Set-Content (Join-Path $scratch 'baseline-current.txt')
$domain = Get-Content (Join-Path $scratch 'baseline-domain-source.json') -Raw | ConvertFrom-Json
$shell = Get-Content (Join-Path $scratch 'baseline-shell-source.json') -Raw | ConvertFrom-Json

function Invalidate-Attempt($reason) {
  $restored =
    ((Get-FileHash $domain.path -Algorithm SHA256).Hash -eq $domain.sha256) -and
    ((Get-FileHash $shell.path -Algorithm SHA256).Hash -eq $shell.sha256)
  [ordered]@{
    stage = 'baseline'
    attempt = $attemptId
    invalidated_at = [DateTimeOffset]::Now.ToString('o')
    reason = $reason
    source_bytes_restored = $restored
  } | ConvertTo-Json | Set-Content (Join-Path $attempt 'invalid.json')
  if (-not $restored) { throw 'Cannot restart baseline: probe bytes are not restored' }
}

function Invoke-CheckedProbe($source, $mode, $package, $label) {
  $runnerArgs = @(
    '-NoLogo', '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $runner,
    '-Path', $source.path, '-Mode', $mode, '-Label', $label,
    '-ExpectedSha256', $source.sha256, '-Scratch', $attempt
  )
  if ($mode -eq 'focused') { $runnerArgs += @('-Package', $package) }
  & powershell.exe @runnerArgs
  $code = $LASTEXITCODE
  $metaPath = Join-Path $attempt "runs/$label-meta.json"
  if (-not (Test-Path $metaPath)) {
    Invalidate-Attempt "$label metadata missing"
    throw "Infrastructure failure: $label metadata missing"
  }
  $meta = Get-Content $metaPath -Raw | ConvertFrom-Json
  if ($meta.sync_started -and $meta.sync_exit_code -ne 0) {
    throw "Confirmed baseline synchronization Cargo failure: $label"
  }
  if ($code -eq 2 -or -not $meta.started -or -not $meta.restored) {
    Invalidate-Attempt "$label did not complete with restored bytes"
    throw "Infrastructure failure: invalidate this measurement session"
  }
  if ($code -ne 0 -or $meta.exit_code -ne 0) {
    throw "Confirmed baseline Cargo failure: $label"
  }
}

Invoke-CheckedProbe $domain 'focused' 'extractum' 'baseline-domain-warmup'
Invoke-CheckedProbe $shell 'workspace' '' 'baseline-shell-warmup'
Invoke-CheckedProbe $shell 'workspace' '' 'baseline-shell-reserve-warmup'
foreach ($index in 1..5) {
  Invoke-CheckedProbe $domain 'focused' 'extractum' "baseline-domain-$index"
  Invoke-CheckedProbe $shell 'workspace' '' "baseline-shell-$index"
  Invoke-CheckedProbe $shell 'workspace' '' "baseline-shell-reserve-$index"
}
```

Expected: all 18 probes pass and restore both files. The three warm-ups are
discarded. The reserve series is used only if Task 3 classifies the primary
post-shell result as marginal. On infrastructure failure, verify both source
hashes, keep the failed attempt with `invalid.json`, and rerun this entire step;
it creates a new attempt and replaces `baseline-current.txt`. Never retry a
label inside the invalid attempt. A confirmed baseline Cargo failure is not
retryable measurement noise and stops the slice.

- [ ] **Step 11: Compute and persist baseline medians**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$attempt = (Get-Content (Join-Path $scratch 'baseline-current.txt') -Raw).Trim()
if (Test-Path (Join-Path $attempt 'invalid.json')) { throw 'Active baseline attempt is invalid' }
function Get-Series([string]$pattern) {
  @(Get-ChildItem (Join-Path $attempt 'runs') -Filter $pattern |
    ForEach-Object { Get-Content $_.FullName -Raw | ConvertFrom-Json } |
    Where-Object { $_.label -notmatch 'warmup' })
}
function Get-Median($values) {
  $sorted = @($values | Sort-Object)
  if ($sorted.Count -ne 5) { throw "Expected five samples, got $($sorted.Count)" }
  [int64]$sorted[2]
}
$domain = Get-Series 'baseline-domain-*-meta.json'
$shell = Get-Series 'baseline-shell-?-meta.json'
$reserve = Get-Series 'baseline-shell-reserve-*-meta.json'
$summary = [ordered]@{
  attempt = $attempt
  domain_samples_ms = @($domain.elapsed_ms)
  domain_median_ms = Get-Median @($domain.elapsed_ms)
  shell_samples_ms = @($shell.elapsed_ms)
  shell_median_ms = Get-Median @($shell.elapsed_ms)
  shell_reserve_samples_ms = @($reserve.elapsed_ms)
  shell_reserve_median_ms = Get-Median @($reserve.elapsed_ms)
}
$summary | ConvertTo-Json -Depth 4 |
  Set-Content -LiteralPath (Join-Path $scratch 'baseline-summary.json')
$summary | Format-List
```

Expected: three five-sample sets and three medians. Do not edit thresholds in
response to these values.

- [ ] **Step 12: Run all three baseline process test modules**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib external_process::
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib child_process::
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib process_tree::
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected on Windows: 12, 1, and 7 tests pass respectively; a zero-test module
is a failure.

---

### Task 2: Add the RED Boundary Contract and Extract the Process Crate

**Files:**

- Create: `src/lib/process-crate-boundary-contract.test.ts`
- Create: `src-tauri/crates/extractum-process/Cargo.toml`
- Create: `src-tauri/crates/extractum-process/src/lib.rs`
- Move: `src-tauri/src/external_process.rs` to
  `src-tauri/crates/extractum-process/src/external_process.rs`
- Move: `src-tauri/src/child_process.rs` to
  `src-tauri/crates/extractum-process/src/child_process.rs`
- Move: `src-tauri/src/process_tree.rs` to
  `src-tauri/crates/extractum-process/src/process_tree.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/external-process-lifecycle-contract.test.ts`
- Modify: `src/lib/hidden-child-process-contract.test.ts`

**Interfaces:**

- Produces `extractum_process::{external_process, child_process, process_tree}`.
- Preserves app paths `crate::external_process`, `crate::child_process`, and
  `crate::process_tree` through private glob facades.
- Produces the exact public API sets declared in Step 5; all other moved
  helpers remain private.

- [ ] **Step 1: Create the source-boundary contract**

Create `src/lib/process-crate-boundary-contract.test.ts` with this complete
content:

```typescript
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repoRoot = process.cwd();
const readSource = (relativePath: string) =>
  readFileSync(path.join(repoRoot, relativePath), "utf8").replace(/\r\n/g, "\n");
const readOptionalSource = (relativePath: string) =>
  existsSync(path.join(repoRoot, relativePath)) ? readSource(relativePath) : "";

const rootCargo = readSource("src-tauri/Cargo.toml");
const rootLib = readSource("src-tauri/src/lib.rs");
const processCargo = readOptionalSource("src-tauri/crates/extractum-process/Cargo.toml");
const processLib = readOptionalSource("src-tauri/crates/extractum-process/src/lib.rs");
const externalProcess = readOptionalSource(
  "src-tauri/crates/extractum-process/src/external_process.rs",
);
const childProcess = readOptionalSource(
  "src-tauri/crates/extractum-process/src/child_process.rs",
);
const processTree = readOptionalSource(
  "src-tauri/crates/extractum-process/src/process_tree.rs",
);
const oldImplementations = [
  readOptionalSource("src-tauri/src/external_process.rs"),
  readOptionalSource("src-tauri/src/child_process.rs"),
  readOptionalSource("src-tauri/src/process_tree.rs"),
].join("\n");

const publicNames = (source: string) =>
  Array.from(
    new Set(
      Array.from(
        source.matchAll(
          /^\s*pub\s+(?:async\s+)?(?:type|struct|enum|fn|const)\s+([A-Za-z_]\w*)/gm,
        ),
        (match) => match[1],
      ),
    ),
  ).sort();

const processTests = [
  "create_no_window_matches_win32_process_creation_flags",
  "admission_wait_consumes_the_shared_graceful_budget",
  "cleanup_tasks_start_concurrently_and_isolate_error_and_panic",
  "concurrent_watchdogs_invoke_exit_once",
  "exhausted_admission_budget_skips_the_cleanup_factory",
  "injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback",
  "permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown",
  "permits_acquired_before_shutdown_are_waited_for",
  "repeated_start_does_not_replace_code_or_schedule_again",
  "start_reports_completed_after_watchdog_claims_exit",
  "start_returns_started_and_schedules_one_watchdog",
  "timing_exposes_the_graceful_and_watchdog_budgets",
  "watchdog_exits_with_the_preserved_code_unless_cleanup_completed",
  "assigns_a_directly_owned_std_child",
  "creates_a_job_object",
  "dropping_the_guard_closes_the_job_and_kills_its_children",
  "process_tree_guard_can_be_owned_by_async_application_state",
  "terminate_failure_remains_reportable_and_retryable",
  "terminate_is_idempotent",
  "terminates_a_descendant_created_after_assignment",
];

describe("extractum process crate boundary", () => {
  it("defines one minimal workspace process package", () => {
    expect(rootCargo).toMatch(
      /members\s*=\s*\[[\s\S]*"\."[\s\S]*"crates\/extractum-core"[\s\S]*"crates\/extractum-process"[\s\S]*\]/,
    );
    expect(processCargo).toBe(
      [
        "[package]",
        'name = "extractum-process"',
        "version.workspace = true",
        "edition.workspace = true",
        "publish = false",
        "",
        "[dependencies]",
        "anyhow.workspace = true",
        "parking_lot.workspace = true",
        "tokio.workspace = true",
        "",
        "[target.'cfg(windows)'.dependencies]",
        "windows-sys.workspace = true",
        "",
        "[dev-dependencies]",
        'tokio = { workspace = true, features = ["test-util"] }',
        "",
      ].join("\n"),
    );
    expect(processLib).toBe(
      [
        "pub mod child_process;",
        "pub mod external_process;",
        "pub mod process_tree;",
        "",
      ].join("\n"),
    );
    expect(processLib).not.toMatch(/pub\s+use\s+[^;]*\*/);
  });

  it("keeps dependency roots exact and application concerns out", () => {
    for (const dependency of ["anyhow", "parking_lot", "tokio", "windows-sys"]) {
      expect(rootCargo).toMatch(
        new RegExp(`\\[workspace\\.dependencies\\][\\s\\S]*${dependency}`),
      );
    }
    const processSource = [externalProcess, childProcess, processTree].join("\n");
    for (const forbidden of [
      "tauri",
      "sqlx",
      "apalis",
      "extractum_core",
      "gemini_browser",
      "youtube",
      "job_helpers",
    ]) {
      expect(processCargo).not.toContain(forbidden);
      expect(processSource).not.toContain(forbidden);
    }
  });

  it("exposes only the reviewed cross-crate API", () => {
    expect(publicNames(externalProcess)).toEqual(
      [
        "AdmissionPermit",
        "AdmissionRejected",
        "CleanupFactory",
        "ExitCallback",
        "ExternalProcessShutdownState",
        "MonotonicClock",
        "ShutdownCleanup",
        "ShutdownCleanupError",
        "ShutdownRun",
        "ShutdownStart",
        "ShutdownTiming",
        "WatchdogScheduler",
        "WatchdogTask",
        "coordinate",
        "new",
        "os_thread_watchdog_scheduler",
        "start",
        "system_monotonic_clock",
        "try_admit",
        "warn_shutdown_stage",
      ].sort(),
    );
    expect(externalProcess).toMatch(/pub\s+graceful:\s*Duration/);
    expect(externalProcess).toMatch(/pub\s+watchdog:\s*Duration/);
    expect(publicNames(childProcess)).toEqual(
      ["CREATE_NO_WINDOW", "hide_console_window"].sort(),
    );
    expect(publicNames(processTree)).toEqual(
      ["ProcessTreeGuard", "assign_std", "assign_tokio", "new", "terminate"].sort(),
    );
    expect([externalProcess, childProcess, processTree].join("\n")).not.toContain(
      "pub(crate)",
    );
  });

  it("preserves private app-side glob facades and existing consumer paths", () => {
    for (const moduleName of ["external_process", "child_process", "process_tree"]) {
      expect(rootLib).not.toContain(`mod ${moduleName};`);
      expect(rootLib).toMatch(
        new RegExp(
          `mod\\s+${moduleName}\\s*\\{[\\s\\S]*?pub\\(crate\\)\\s+use\\s+extractum_process::${moduleName}::\\*;[\\s\\S]*?\\}`,
        ),
      );
    }
    expect(readSource("src-tauri/src/youtube/process_runtime.rs")).toContain(
      "crate::external_process",
    );
    expect(readSource("src-tauri/src/youtube/process_runtime.rs")).toContain(
      "crate::process_tree",
    );
    expect(readSource("src-tauri/src/diagnostics/runtime.rs")).toContain(
      "crate::child_process",
    );
    expect(readSource("src-tauri/src/gemini_browser/sidecar.rs")).toContain(
      "crate::child_process::hide_console_window",
    );
  });

  it("moves implementations and all twenty tests instead of copying them", () => {
    expect(oldImplementations).toBe("");
    const movedSource = [externalProcess, childProcess, processTree].join("\n");
    for (const testName of processTests) {
      expect(movedSource).toContain(`fn ${testName}(`);
      expect(oldImplementations).not.toContain(`fn ${testName}(`);
    }
  });
});
```

- [ ] **Step 2: Run the new contract and observe the intended RED**

Run:

```powershell
npm.cmd run test -- src/lib/process-crate-boundary-contract.test.ts
```

Expected: the test file loads, then fails because the process workspace member,
manifest, modules, and facades do not exist. A TypeScript/import failure is the
wrong RED and must be fixed before continuing.

- [ ] **Step 3: Add exact shared workspace dependencies and the new package**

In `src-tauri/Cargo.toml`, make the workspace/member and dependency ownership
exactly:

```toml
[workspace]
members = [".", "crates/extractum-core", "crates/extractum-process"]
resolver = "2"

[workspace.dependencies]
anyhow = "1.0"
parking_lot = "0.12"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
time = { version = "0.3", features = ["formatting", "parsing", "macros"] }
tokio = { version = "1", features = ["full"] }
windows-sys = { version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }
zstd = "0.13"
```

Change the existing app dependencies to:

```toml
parking_lot = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
```

Change the existing app dev dependency to:

```toml
[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

Remove the app package's entire
`[target.'cfg(windows)'.dependencies]` block: after the move, no app source
uses `windows-sys` directly. Add this root app dependency alongside
`extractum-core`:

```toml
extractum-process = { path = "crates/extractum-process" }
```

Create `src-tauri/crates/extractum-process/Cargo.toml` exactly as asserted by
the contract:

```toml
[package]
name = "extractum-process"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
anyhow.workspace = true
parking_lot.workspace = true
tokio.workspace = true

[target.'cfg(windows)'.dependencies]
windows-sys.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

Create `src-tauri/crates/extractum-process/src/lib.rs` exactly:

```rust
pub mod child_process;
pub mod external_process;
pub mod process_tree;
```

- [ ] **Step 4: Move the three implementations byte-for-byte**

Move, do not copy, these files:

```text
src-tauri/src/external_process.rs
  -> src-tauri/crates/extractum-process/src/external_process.rs
src-tauri/src/child_process.rs
  -> src-tauri/crates/extractum-process/src/child_process.rs
src-tauri/src/process_tree.rs
  -> src-tauri/crates/extractum-process/src/process_tree.rs
```

Immediately verify the old paths are absent and the new paths exist. Do not
reformat or edit test bodies during the move.

- [ ] **Step 5: Apply the enumerated visibility boundary**

In the moved `external_process.rs`, make exactly these declarations public:

```text
type ExitCallback
type MonotonicClock
type WatchdogTask
type WatchdogScheduler
type ShutdownCleanup
type CleanupFactory
fn warn_shutdown_stage
struct ShutdownTiming, including fields graceful and watchdog
struct AdmissionRejected
enum ShutdownCleanupError
struct ExternalProcessShutdownState
struct AdmissionPermit
enum ShutdownStart
struct ShutdownRun
fn system_monotonic_clock
fn os_thread_watchdog_scheduler
ExternalProcessShutdownState::{new, try_admit, start}
ShutdownRun::coordinate
```

Use ordinary `pub`, not `pub(crate)`. Narrow these former crate-visible items
to private because they are used only inside the moved module and its child
tests:

```text
GRACEFUL_SHUTDOWN_TIMEOUT
SHUTDOWN_WATCHDOG_TIMEOUT
warn_shutdown_coordinator_stage
ShutdownPhase
ExternalProcessShutdownState::{wait_for_startups, run_watchdog, schedule_watchdog}
```

Do not change already-private helpers or fields.

In moved `child_process.rs`, use:

```rust
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub fn hide_console_window(command: &mut Command) -> &mut Command {
```

Preserve the existing cfg attribute and function body.

In both cfg implementations in moved `process_tree.rs`, make
`ProcessTreeGuard` and exactly these methods public:

```rust
pub fn new() -> anyhow::Result<Self>
pub fn assign_tokio(&self, child: &tokio::process::Child) -> anyhow::Result<()>
pub fn assign_std(&self, child: &std::process::Child) -> anyhow::Result<()>
pub fn terminate(&self) -> anyhow::Result<()>
```

Keep `assign_raw` private. After these edits, none of the three moved files may
contain `pub(crate)`.

- [ ] **Step 6: Replace app file modules with the three private facades**

In `src-tauri/src/lib.rs`, replace each one-line module declaration with the
matching facade while keeping the surrounding imports in their current order:

```rust
mod external_process {
    pub(crate) use extractum_process::external_process::*;
}
```

```rust
mod child_process {
    pub(crate) use extractum_process::child_process::*;
}
```

```rust
mod process_tree {
    pub(crate) use extractum_process::process_tree::*;
}
```

Do not edit the existing `use external_process::{...}` or any downstream
consumer file.

- [ ] **Step 7: Migrate the two existing source contracts to the new owner**

In `src/lib/external-process-lifecycle-contract.test.ts`, replace imports:

```typescript
import coordinatorSource from "../../src-tauri/crates/extractum-process/src/external_process.rs?raw";
import processTreeSource from "../../src-tauri/crates/extractum-process/src/process_tree.rs?raw";
```

Replace the old module assertion:

```typescript
expect(lib).toMatch(
  /mod\s+external_process\s*\{[\s\S]*pub\(crate\)\s+use\s+extractum_process::external_process::\*;/,
);
```

Leave every lifecycle/sidecar/CDP semantic assertion unchanged.

In `src/lib/hidden-child-process-contract.test.ts`, replace the helper import:

```typescript
import childProcessSource from "../../src-tauri/crates/extractum-process/src/child_process.rs?raw";
```

Replace its module/constant assertions with:

```typescript
expect(libSource).toMatch(
  /mod\s+child_process\s*\{[\s\S]*pub\(crate\)\s+use\s+extractum_process::child_process::\*;/,
);
expect(childProcessSource).toContain(
  "pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;",
);
```

Leave all consumer assertions unchanged.

- [ ] **Step 8: Format Rust and materialize the lockfile**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-process --all-targets
```

Expected: Cargo updates `src-tauri/Cargo.lock`, the new crate checks, and no
warning reports dead or inaccessible process API.

- [ ] **Step 9: Run the RED contract GREEN and protect legacy contracts**

Run:

```powershell
npm.cmd run test -- src/lib/process-crate-boundary-contract.test.ts src/lib/external-process-lifecycle-contract.test.ts src/lib/hidden-child-process-contract.test.ts
```

Expected: exactly the three requested contract files pass. Any failure in an
unchanged semantic assertion means the move altered behavior or the contract
was over-edited.

- [ ] **Step 10: Run narrow GREEN, package checkpoint, and app checkpoint**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$narrowStdout = Join-Path $scratch 'narrow-process.stdout.log'
$narrowStderr = Join-Path $scratch 'narrow-process.stderr.log'
$narrowProcess = Start-Process -FilePath (Get-Command cargo.exe).Source -Wait -PassThru `
  -NoNewWindow -RedirectStandardOutput $narrowStdout -RedirectStandardError $narrowStderr `
  -ArgumentList @('test', '--manifest-path', 'src-tauri/Cargo.toml', '-p',
    'extractum-process', '--lib',
    'external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets',
    '--', '--exact')
$narrowOutput = @(
  @(Get-Content -LiteralPath $narrowStdout -ErrorAction SilentlyContinue)
  @(Get-Content -LiteralPath $narrowStderr -ErrorAction SilentlyContinue)
) | ForEach-Object { $_.ToString() }
$narrowOutput
if ($narrowProcess.ExitCode -ne 0 -or
    -not (($narrowOutput -join "`n") -match 'test result: ok\. 1 passed')) { exit 1 }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-process --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: one exact narrow test, all 20 process tests, then the immediate
dependent app package check pass. A zero-test result fails the corresponding
step.

- [ ] **Step 11: Enforce the candidate file allowlist**

Run:

```powershell
$expected = @(
  'src-tauri/Cargo.lock'
  'src-tauri/Cargo.toml'
  'src-tauri/crates/extractum-process/Cargo.toml'
  'src-tauri/crates/extractum-process/src/child_process.rs'
  'src-tauri/crates/extractum-process/src/external_process.rs'
  'src-tauri/crates/extractum-process/src/lib.rs'
  'src-tauri/crates/extractum-process/src/process_tree.rs'
  'src-tauri/src/child_process.rs'
  'src-tauri/src/external_process.rs'
  'src-tauri/src/lib.rs'
  'src-tauri/src/process_tree.rs'
  'src/lib/external-process-lifecycle-contract.test.ts'
  'src/lib/hidden-child-process-contract.test.ts'
  'src/lib/process-crate-boundary-contract.test.ts'
)
$changed = @(
  @(git diff --name-only --no-renames)
  @(git ls-files --others --exclude-standard)
) | Sort-Object -Unique
$extra = @($changed | Where-Object { $_ -notin $expected })
$missing = @($expected | Where-Object { $_ -notin $changed })
"CHANGED=$($changed.Count) EXTRA=$($extra.Count) MISSING=$($missing.Count)"
if ($extra.Count -ne 0 -or $missing.Count -ne 0) { exit 1 }
```

Expected: exactly the 14 paths above. Deleted old paths count as intended
changes. Stop if Cargo formatting or any manual edit touched a consumer.

- [ ] **Step 12: Commit the extraction candidate and record its identity**

Run:

```powershell
git diff --check
git add -- `
  src-tauri/Cargo.lock `
  src-tauri/Cargo.toml `
  src-tauri/crates/extractum-process `
  src-tauri/src/child_process.rs `
  src-tauri/src/external_process.rs `
  src-tauri/src/lib.rs `
  src-tauri/src/process_tree.rs `
  src/lib/external-process-lifecycle-contract.test.ts `
  src/lib/hidden-child-process-contract.test.ts `
  src/lib/process-crate-boundary-contract.test.ts
git diff --cached --check
git commit -m "refactor: extract process infrastructure crate"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$candidate = (git rev-parse HEAD).Trim()
$candidate | Set-Content -LiteralPath (Join-Path $scratch 'candidate-commit.txt')
git status --short
```

Expected: one candidate commit and a clean tree. Do not call it retained until
Tasks 3 and 4 pass.

---

### Task 3: Verify Windows Surface, Inventory, Consumer Stability, and Retention

**Files:**

- Read: `$env:TEMP/extractum-process-current.txt`
- Read: scratch baseline summaries and hashes from Task 1
- Read: candidate commit from Task 2
- Create: scratch post-inventory, post-probe, and decision artifacts

**Interfaces:**

- Consumes the committed `extractum-process` candidate and the scratch probe
  runner.
- Produces `post-test-names.txt`, `inventory-comparison.json`,
  `post-summary.json`, and `decision.json` with `retain_candidate` and exact
  gate values.
- Classifies probe exit 2/missing metadata as infrastructure failure and exit
  1 with complete metadata as confirmed Cargo failure.

- [ ] **Step 1: Reconfirm the candidate and clean measurement state**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$candidate = (Get-Content (Join-Path $scratch 'candidate-commit.txt') -Raw).Trim()
$head = (git rev-parse HEAD).Trim()
$status = @(git status --short --untracked-files=all)
"CANDIDATE=$candidate HEAD=$head STATUS_COUNT=$($status.Count)"
if ($candidate -ne $head -or $status.Count -ne 0) { exit 1 }

$names = @('cargo', 'rustc', 'rust-analyzer', 'extractum')
$active = @(Get-Process -ErrorAction SilentlyContinue | Where-Object {
  $_.ProcessName -in $names -or $_.ProcessName -like 'cargo-*'
})
$active | Select-Object Id, ProcessName
if ($active.Count -ne 0) { exit 1 }
```

Expected: clean candidate HEAD and no active toolchain/app process.

- [ ] **Step 2: Prove the Windows process-crate surface on the canonical host**

Run:

```powershell
$hostLine = (& rustc -vV | Select-String '^host:').Line
$rustHost = ($hostLine -replace '^host:\s*', '').Trim()
if ($rustHost -ne 'x86_64-pc-windows-msvc') { throw "Unexpected Rust host: $rustHost" }
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-process --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: the process crate, its test targets, and Windows job-object surface
type-check on `x86_64-pc-windows-msvc`. The moved non-Windows stub remains in
source but is outside this slice's acceptance gate.

- [ ] **Step 3: Capture and compare the complete post-extraction inventory**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$stdout = Join-Path $scratch 'post-inventory.stdout.log'
$stderr = Join-Path $scratch 'post-inventory.stderr.log'
$cargoExe = (Get-Command cargo.exe).Source
$args = @(
  'test', '--manifest-path', 'src-tauri/Cargo.toml', '--workspace',
  '--all-targets', '--', '--list'
)
$process = Start-Process -FilePath $cargoExe -ArgumentList $args -Wait -PassThru `
  -NoNewWindow -RedirectStandardOutput $stdout -RedirectStandardError $stderr
$lines = @(
  @(Get-Content -LiteralPath $stdout -ErrorAction SilentlyContinue)
  @(Get-Content -LiteralPath $stderr -ErrorAction SilentlyContinue)
) | ForEach-Object { $_.ToString() }
$names = @($lines | Where-Object { $_ -match ': test$' } |
  ForEach-Object { ($_ -replace ': test$', '').Trim() })
$unique = @($names | Sort-Object -Unique)
$unique | Set-Content -LiteralPath (Join-Path $scratch 'post-test-names.txt')
$baseline = @(Get-Content -LiteralPath (Join-Path $scratch 'baseline-test-names.txt'))
$processBaseline = @(Get-Content -LiteralPath (Join-Path $scratch 'process-test-names.txt'))
$missing = @($baseline | Where-Object { $_ -notin $unique })
$postProcess = @($unique | Where-Object {
  $_ -match '^(external_process|child_process|process_tree)::'
})
$missingProcess = @($processBaseline | Where-Object { $_ -notin $postProcess })
$extraProcess = @($postProcess | Where-Object { $_ -notin $processBaseline })
$comparison = [ordered]@{
  exit = $process.ExitCode
  baseline_count = $baseline.Count
  post_count = $unique.Count
  missing_count = $missing.Count
  process_baseline_count = $processBaseline.Count
  process_post_count = $postProcess.Count
  missing_process = $missingProcess
  extra_process = $extraProcess
}
$comparison | ConvertTo-Json -Depth 5 |
  Set-Content -LiteralPath (Join-Path $scratch 'inventory-comparison.json')
$comparison | Format-List
if ($process.ExitCode -ne 0 -or $unique.Count -ne $names.Count -or
    $unique.Count -lt $baseline.Count -or
    $missing.Count -ne 0 -or $postProcess.Count -ne 20 -or
    $missingProcess.Count -ne 0 -or $extraProcess.Count -ne 0) { exit 1 }
```

Expected: no baseline test disappears, total count does not decrease, and the
same exact 20 process names now execute from the new package. Because libtest
names do not include package names, no rename map is required for this move.

- [ ] **Step 4: Prove every application consumer remained byte-for-byte stable**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$baseline = @(Get-Content (Join-Path $scratch 'consumer-hashes.json') -Raw | ConvertFrom-Json)
$changed = @($baseline | Where-Object {
  -not (Test-Path -LiteralPath $_.path) -or
  (Get-FileHash -LiteralPath $_.path -Algorithm SHA256).Hash -ne $_.sha256
})
$changed | Format-Table path, sha256
"CHANGED_CONSUMERS=$($changed.Count)"
if ($changed.Count -ne 0) { exit 1 }
```

Expected: `CHANGED_CONSUMERS=0`.

- [ ] **Step 5: Snapshot the committed post-extraction probe sources**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$sources = [ordered]@{
  domain = 'src-tauri/crates/extractum-process/src/external_process.rs'
  shell = 'src-tauri/src/lib.rs'
}
foreach ($entry in $sources.GetEnumerator()) {
  @{
    path = $entry.Value
    sha256 = (Get-FileHash -LiteralPath $entry.Value -Algorithm SHA256).Hash
  } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch "post-$($entry.Key)-source.json")
}
```

Expected: the domain path is the moved logical file; the shell path is the
retained, committed application facade shape.

- [ ] **Step 6: Run primary post-domain and post-shell measurements**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-cargo-probe.ps1'
$attemptId = "post-{0}-{1}" -f ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')),
  ([guid]::NewGuid().ToString('N'))
$attempt = Join-Path $scratch "measurements/$attemptId"
New-Item -ItemType Directory -Path (Join-Path $attempt 'runs') -Force | Out-Null
$attempt | Set-Content (Join-Path $scratch 'post-current.txt')
$domain = Get-Content (Join-Path $scratch 'post-domain-source.json') -Raw | ConvertFrom-Json
$shell = Get-Content (Join-Path $scratch 'post-shell-source.json') -Raw | ConvertFrom-Json

function Invalidate-Attempt($reason) {
  $restored =
    ((Get-FileHash $domain.path -Algorithm SHA256).Hash -eq $domain.sha256) -and
    ((Get-FileHash $shell.path -Algorithm SHA256).Hash -eq $shell.sha256)
  [ordered]@{
    stage = 'post'
    attempt = $attemptId
    invalidated_at = [DateTimeOffset]::Now.ToString('o')
    reason = $reason
    source_bytes_restored = $restored
  } | ConvertTo-Json | Set-Content (Join-Path $attempt 'invalid.json')
  if (-not $restored) { throw 'Cannot restart post measurements: probe bytes are not restored' }
}

function Invoke-CheckedProbe($source, $mode, $package, $label) {
  $runnerArgs = @(
    '-NoLogo', '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $runner,
    '-Path', $source.path, '-Mode', $mode, '-Label', $label,
    '-ExpectedSha256', $source.sha256, '-Scratch', $attempt
  )
  if ($mode -eq 'focused') { $runnerArgs += @('-Package', $package) }
  & powershell.exe @runnerArgs
  $code = $LASTEXITCODE
  $metaPath = Join-Path $attempt "runs/$label-meta.json"
  if (-not (Test-Path $metaPath)) {
    Invalidate-Attempt "$label metadata missing"
    throw "Infrastructure failure: $label metadata missing"
  }
  $meta = Get-Content $metaPath -Raw | ConvertFrom-Json
  if ($meta.sync_started -and $meta.sync_exit_code -ne 0) {
    throw "Confirmed candidate synchronization Cargo failure: $label"
  }
  if ($code -eq 2 -or -not $meta.started -or -not $meta.restored) {
    Invalidate-Attempt "$label did not complete with restored bytes"
    throw "Infrastructure failure: invalidate the post measurement session"
  }
  if ($code -ne 0 -or $meta.exit_code -ne 0) {
    throw "Confirmed candidate Cargo failure: $label"
  }
}

Invoke-CheckedProbe $domain 'focused' 'extractum-process' 'post-domain-warmup'
Invoke-CheckedProbe $shell 'workspace' '' 'post-shell-warmup'
foreach ($index in 1..5) {
  Invoke-CheckedProbe $domain 'focused' 'extractum-process' "post-domain-$index"
  Invoke-CheckedProbe $shell 'workspace' '' "post-shell-$index"
}
```

Expected: 12 probes pass; two warm-ups are excluded. The focused process
series is diagnostic. The shell series determines retention. On infrastructure
failure, verify both source hashes, preserve the failed attempt with
`invalid.json`, and rerun this whole step. The new attempt replaces only
`post-current.txt`; the valid baseline attempt and summary remain unchanged.

- [ ] **Step 7: Compute the primary decision before any repeat**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$baseline = Get-Content (Join-Path $scratch 'baseline-summary.json') -Raw | ConvertFrom-Json
$postAttempt = (Get-Content (Join-Path $scratch 'post-current.txt') -Raw).Trim()
if (Test-Path (Join-Path $postAttempt 'invalid.json')) { throw 'Active post attempt is invalid' }
function Get-Series([string]$pattern) {
  @(Get-ChildItem (Join-Path $postAttempt 'runs') -Filter $pattern |
    ForEach-Object { Get-Content $_.FullName -Raw | ConvertFrom-Json } |
    Where-Object { $_.label -notmatch 'warmup' })
}
function Get-Median($values) {
  $sorted = @($values | Sort-Object)
  if ($sorted.Count -ne 5) { throw "Expected five samples, got $($sorted.Count)" }
  [int64]$sorted[2]
}
$domain = Get-Series 'post-domain-*-meta.json'
$shell = Get-Series 'post-shell-?-meta.json'
$domainMedian = Get-Median @($domain.elapsed_ms)
$shellMedian = Get-Median @($shell.elapsed_ms)
$shellDelta = [int64]$shellMedian - [int64]$baseline.shell_median_ms
$shellPercent = if ($baseline.shell_median_ms -eq 0) {
  [double]::PositiveInfinity
} else {
  100.0 * $shellDelta / [double]$baseline.shell_median_ms
}
$primaryPass = $shellDelta -le 500 -and $shellPercent -le 5.0
$marginal = -not $primaryPass -and $shellDelta -le 800 -and $shellPercent -le 8.0
$summary = [ordered]@{
  attempt = $postAttempt
  domain_samples_ms = @($domain.elapsed_ms)
  domain_median_ms = $domainMedian
  domain_delta_ms = $domainMedian - [int64]$baseline.domain_median_ms
  shell_samples_ms = @($shell.elapsed_ms)
  shell_median_ms = $shellMedian
  shell_delta_ms = $shellDelta
  shell_delta_percent = $shellPercent
  primary_shell_pass = $primaryPass
  marginal_repeat_allowed = $marginal
}
$summary | ConvertTo-Json -Depth 4 |
  Set-Content -LiteralPath (Join-Path $scratch 'post-summary.json')
$summary | Format-List
```

Expected: the decision is computed from the predeclared 5%/500 ms cap. Do not
round before comparisons or change the marginal window.

- [ ] **Step 8: Consume the one repeat only for a marginal primary result**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$post = Get-Content (Join-Path $scratch 'post-summary.json') -Raw | ConvertFrom-Json
$baseline = Get-Content (Join-Path $scratch 'baseline-summary.json') -Raw | ConvertFrom-Json
$shell = Get-Content (Join-Path $scratch 'post-shell-source.json') -Raw | ConvertFrom-Json
$runner = Join-Path $scratch 'invoke-cargo-probe.ps1'
$repeatUsed = $false
$repeatPass = $false
$repeatSamples = @()
$repeatMedian = $null
$repeatDelta = $null
$repeatPercent = $null
$repeatAttempt = $null

if ($post.marginal_repeat_allowed) {
  $repeatUsed = $true
  $attemptId = "repeat-{0}-{1}" -f ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')),
    ([guid]::NewGuid().ToString('N'))
  $repeatAttempt = Join-Path $scratch "measurements/$attemptId"
  New-Item -ItemType Directory -Path (Join-Path $repeatAttempt 'runs') -Force | Out-Null
  $repeatAttempt | Set-Content (Join-Path $scratch 'repeat-current.txt')
  function Invalidate-Repeat($reason) {
    $restored = (Get-FileHash $shell.path -Algorithm SHA256).Hash -eq $shell.sha256
    [ordered]@{
      stage = 'repeat'
      attempt = $attemptId
      invalidated_at = [DateTimeOffset]::Now.ToString('o')
      reason = $reason
      source_bytes_restored = $restored
    } | ConvertTo-Json | Set-Content (Join-Path $repeatAttempt 'invalid.json')
    if (-not $restored) { throw 'Cannot restart repeat: probe bytes are not restored' }
  }
  function Invoke-RepeatProbe($label) {
    & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner `
      -Path $shell.path -Mode workspace -Label $label `
      -ExpectedSha256 $shell.sha256 -Scratch $repeatAttempt
    $code = $LASTEXITCODE
    $metaPath = Join-Path $repeatAttempt "runs/$label-meta.json"
    if (-not (Test-Path $metaPath)) {
      Invalidate-Repeat "$label metadata missing"
      throw "Infrastructure repeat failure"
    }
    $meta = Get-Content $metaPath -Raw | ConvertFrom-Json
    if ($meta.sync_started -and $meta.sync_exit_code -ne 0) {
      throw "Confirmed synchronization Cargo failure in repeat"
    }
    if ($code -eq 2 -or -not $meta.started -or -not $meta.restored) {
      Invalidate-Repeat "$label did not complete with restored bytes"
      throw "Infrastructure repeat failure: invalidate repeat session"
    }
    if ($code -ne 0 -or $meta.exit_code -ne 0) {
      throw "Confirmed Cargo failure in repeat"
    }
  }
  Invoke-RepeatProbe 'post-shell-repeat-warmup'
  foreach ($index in 1..5) { Invoke-RepeatProbe "post-shell-repeat-$index" }
  $metas = @(Get-ChildItem (Join-Path $repeatAttempt 'runs') -Filter 'post-shell-repeat-?-meta.json' |
    ForEach-Object { Get-Content $_.FullName -Raw | ConvertFrom-Json })
  $repeatSamples = @($metas.elapsed_ms)
  $sorted = @($repeatSamples | Sort-Object)
  if ($sorted.Count -ne 5) { throw "Repeat did not produce five samples" }
  $repeatMedian = [int64]$sorted[2]
  $repeatDelta = $repeatMedian - [int64]$baseline.shell_reserve_median_ms
  $repeatPercent = 100.0 * $repeatDelta / [double]$baseline.shell_reserve_median_ms
  $repeatPass = $repeatDelta -le 500 -and $repeatPercent -le 5.0
}

$retain = [bool]$post.primary_shell_pass -or ($repeatUsed -and $repeatPass)
$decision = [ordered]@{
  reason = 'protocol_completed'
  primary_shell_pass = [bool]$post.primary_shell_pass
  repeat_used = $repeatUsed
  repeat_attempt = $repeatAttempt
  repeat_samples_ms = $repeatSamples
  repeat_median_ms = $repeatMedian
  repeat_delta_ms = $repeatDelta
  repeat_delta_percent = $repeatPercent
  repeat_shell_pass = $repeatPass
  retain_candidate = $retain
}
$decision | ConvertTo-Json -Depth 5 |
  Set-Content -LiteralPath (Join-Path $scratch 'decision.json')
$decision | Format-List
```

Expected:

- primary pass: no repeat files and `retain_candidate=true`;
- marginal primary: exactly one five-sample repeat compared with the reserved
  baseline series; an infrastructure failure preserves that repeat attempt as
  invalid and requires rerunning all of Step 8 in a new repeat attempt;
- non-marginal fail: no repeat and `retain_candidate=false`.

- [ ] **Step 9: Verify the candidate is still committed and byte-clean**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$candidate = (Get-Content (Join-Path $scratch 'candidate-commit.txt') -Raw).Trim()
if ((git rev-parse HEAD).Trim() -ne $candidate) { exit 1 }
if (@(git status --short --untracked-files=all).Count -ne 0) { exit 1 }
foreach ($name in @('post-domain', 'post-shell')) {
  $source = Get-Content (Join-Path $scratch "$name-source.json") -Raw | ConvertFrom-Json
  if ((Get-FileHash $source.path -Algorithm SHA256).Hash -ne $source.sha256) {
    throw "$name source was not restored"
  }
}
```

Expected: clean candidate commit and exact restoration of both logical probe
files.

---

### Task 4: Complete Correctness Gates and Record Retention or Rollback

**Files:**

- Create: `docs/superpowers/verification/2026-07-17-extractum-process-extraction.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Read: every scratch artifact produced by Tasks 1 and 3
- Conditional history mutation: non-destructive `git revert` of the candidate
  only when `decision.retain_candidate` is false

**Interfaces:**

- Consumes `decision.json`; no human judgment changes its boolean. A confirmed
  completion failure may mechanically change provisional retention to
  `not_retained` with reason `completion_failure`.
- Produces one committed verification record with status `retained` or
  `not_retained` and literal commands/results.
- A retained path leaves `extractum-process` in the workspace. A rejected path
  restores the pre-candidate code through a revert commit while preserving
  design and evidence.

- [ ] **Step 1: Branch only on the precomputed decision**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$decision = Get-Content (Join-Path $scratch 'decision.json') -Raw | ConvertFrom-Json
$decision | Format-List
if ($decision.reason -ne 'protocol_completed') { exit 1 }
if ($decision.retain_candidate) {
  'PATH=provisionally_retained' | Set-Content (Join-Path $scratch 'current-path.txt')
} else {
  'PATH=not_retained' | Set-Content (Join-Path $scratch 'current-path.txt')
}
Get-Content (Join-Path $scratch 'current-path.txt')
```

Expected: exactly one provisional path selected. Never reinterpret a failing
metric as an architectural exception; Phase 3 already has its explicit
architectural rule, the shell cap remains binding, and a provisional retained
path can still become `not_retained` on a completion failure.

- [ ] **Step 2A: Run every command-based completion gate and route a failure**

Run this step only while `decision.retain_candidate` is true. The first
started command that exits nonzero, or a test gate that selects the wrong
inventory, is a confirmed completion failure. It changes the decision to
`not_retained` and joins Step 2B; it must not leave the candidate in an
unclassified state.

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$decisionPath = Join-Path $scratch 'decision.json'
$decision = Get-Content $decisionPath -Raw | ConvertFrom-Json
if ($decision.retain_candidate) {
  $cargoExe = (Get-Command cargo.exe).Source
  $cmdExe = (Get-Command cmd.exe).Source
  $gates = @(
    [pscustomobject]@{ name = 'process-tests'; file = $cargoExe; args = @(
      'test', '--manifest-path', 'src-tauri/Cargo.toml', '-p',
      'extractum-process', '--all-targets'); pattern = 'test result: ok\. 20 passed' },
    [pscustomobject]@{ name = 'process-check'; file = $cargoExe; args = @(
      'check', '--manifest-path', 'src-tauri/Cargo.toml', '-p',
      'extractum-process', '--all-targets'); pattern = '' },
    [pscustomobject]@{ name = 'app-check'; file = $cargoExe; args = @(
      'check', '--manifest-path', 'src-tauri/Cargo.toml', '-p',
      'extractum', '--all-targets'); pattern = '' },
    [pscustomobject]@{ name = 'youtube-process-tests'; file = $cargoExe; args = @(
      'test', '--manifest-path', 'src-tauri/Cargo.toml', '-p',
      'extractum', '--lib', 'youtube::process_runtime::');
      pattern = 'test result: ok\. [1-9][0-9]* passed' },
    [pscustomobject]@{ name = 'rustfmt'; file = $cmdExe; args = @(
      '/d', '/c', 'npm.cmd', 'run', 'check:rustfmt'); pattern = '' },
    [pscustomobject]@{ name = 'workspace-check'; file = $cargoExe; args = @(
      'check', '--manifest-path', 'src-tauri/Cargo.toml', '--workspace',
      '--all-targets'); pattern = '' },
    [pscustomobject]@{ name = 'workspace-tests'; file = $cargoExe; args = @(
      'test', '--manifest-path', 'src-tauri/Cargo.toml', '--workspace',
      '--all-targets'); pattern = '' },
    [pscustomobject]@{ name = 'verify'; file = $cmdExe; args = @(
      '/d', '/c', 'npm.cmd', 'run', 'verify'); pattern = '' },
    [pscustomobject]@{ name = 'release-no-bundle'; file = $cmdExe; args = @(
      '/d', '/c', 'npm.cmd', 'run', 'tauri', '--', 'build', '--no-bundle');
      pattern = '' }
  )

  $results = @()
  $failure = $null
  foreach ($gate in $gates) {
    $stdout = Join-Path $scratch "completion-$($gate.name).stdout.log"
    $stderr = Join-Path $scratch "completion-$($gate.name).stderr.log"
    try {
      $process = Start-Process -FilePath $gate.file -ArgumentList $gate.args `
        -Wait -PassThru -NoNewWindow -RedirectStandardOutput $stdout `
        -RedirectStandardError $stderr
    } catch {
      [ordered]@{
        gate = $gate.name
        classification = 'infrastructure_failure'
        error = $_.Exception.Message
      } | ConvertTo-Json | Set-Content `
        (Join-Path $scratch 'completion-infrastructure-failure.json')
      throw
    }
    $output = @(
      @(Get-Content $stdout -ErrorAction SilentlyContinue)
      @(Get-Content $stderr -ErrorAction SilentlyContinue)
    ) | ForEach-Object { $_.ToString() }
    $patternOk = [string]::IsNullOrEmpty($gate.pattern) -or
      (($output -join "`n") -match $gate.pattern)
    $result = [pscustomobject]@{
      gate = $gate.name
      exit_code = $process.ExitCode
      inventory_pattern_ok = $patternOk
    }
    $results += $result
    if ($process.ExitCode -ne 0 -or -not $patternOk) {
      $failure = $result
      break
    }
  }
  $results | ConvertTo-Json -Depth 4 |
    Set-Content (Join-Path $scratch 'completion-gates.json')

  if ($null -ne $failure) {
    $failure | ConvertTo-Json -Depth 4 |
      Set-Content (Join-Path $scratch 'completion-failure.json')
    $decision.retain_candidate = $false
    $decision.reason = 'completion_failure'
    $decision | Add-Member -NotePropertyName completion_gate `
      -NotePropertyValue $failure.gate -Force
    $decision | ConvertTo-Json -Depth 6 | Set-Content $decisionPath
    'PATH=not_retained' | Set-Content (Join-Path $scratch 'current-path.txt')
  }
}
```

Expected on success: all nine records pass, including exactly 20 process tests
and a nonzero YouTube selection; the decision stays provisionally retained.
Expected on confirmed gate failure: `completion-failure.json` names the first
failed gate, the decision becomes `completion_failure`, later gates are not
run, and execution continues at Step 2B. An infrastructure start failure is
recorded separately and stops the session without blaming or reverting the
candidate.

- [ ] **Step 3A: On the retained path, smoke the release executable**

Re-read `decision.json` and run this step only while
`decision.retain_candidate` is true:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$decisionPath = Join-Path $scratch 'decision.json'
$decision = Get-Content $decisionPath -Raw | ConvertFrom-Json
$process = $null
$smokeFailure = $null
$cleanupFailure = $null
if ($decision.retain_candidate) {
  try {
    $exe = (Resolve-Path 'src-tauri/target/release/extractum.exe').Path
    $process = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds 5
    $process.Refresh()
    if ($process.HasExited) {
      throw "Release executable exited early with code $($process.ExitCode)"
    }
  } catch {
    $smokeFailure = $_.Exception.Message
  } finally {
    if ($null -ne $process) {
      try {
        $process.Refresh()
        if (-not $process.HasExited) {
          Stop-Process -Id $process.Id -Force -ErrorAction Stop
          if (-not $process.WaitForExit(10000)) {
            throw "Timed out waiting for process $($process.Id) to exit"
          }
        }
        $process.Refresh()
        if (-not $process.HasExited -or
            $null -ne (Get-Process -Id $process.Id -ErrorAction SilentlyContinue)) {
          throw "Process $($process.Id) is still observable after cleanup"
        }
      } catch {
        $cleanupFailure = $_.Exception.Message
      }
    }
  }
  if ($null -ne $cleanupFailure) {
    $cleanupArtifact = Join-Path $scratch (
      'startup-smoke-infrastructure-failure-{0}-{1}.json' -f
      ([DateTimeOffset]::Now.ToString('yyyyMMddTHHmmssfff')),
      ([guid]::NewGuid().ToString('N'))
    )
    [ordered]@{
      gate = 'startup-smoke-cleanup'
      classification = 'infrastructure_failure'
      error = $cleanupFailure
      process_id = if ($null -ne $process) { $process.Id } else { $null }
    } | ConvertTo-Json | Set-Content $cleanupArtifact
    throw "Startup smoke cleanup is unconfirmed: $cleanupFailure"
  }
  if ($null -ne $smokeFailure) {
    [ordered]@{
      gate = 'startup-smoke'
      classification = 'completion_failure'
      error = $smokeFailure
    } | ConvertTo-Json | Set-Content `
      (Join-Path $scratch 'completion-failure.json')
    $decision.retain_candidate = $false
    $decision.reason = 'completion_failure'
    $decision | Add-Member -NotePropertyName completion_gate `
      -NotePropertyValue 'startup-smoke' -Force
    $decision | ConvertTo-Json -Depth 6 | Set-Content $decisionPath
    'PATH=not_retained' | Set-Content (Join-Path $scratch 'current-path.txt')
  } else {
    [ordered]@{ gate = 'startup-smoke'; passed = $true } | ConvertTo-Json |
      Set-Content (Join-Path $scratch 'startup-smoke.json')
    'PATH=retained' | Set-Content (Join-Path $scratch 'current-path.txt')
  }
}
```

Expected on success: the release executable remains alive for five seconds,
is stopped by PID, and the path becomes finally retained. Expected on failure:
the process is still cleaned up, the failed smoke joins the negative
evidence/revert path, and no `extractum` process remains. A cleanup error does
not retain or reject the candidate: it writes
the uniquely named `startup-smoke-infrastructure-failure-*.json`, stops the
protocol, and requires the process to be terminated and the smoke rerun before
any final decision.

- [ ] **Step 2B: On the rejected path, write the negative decision before rollback**

Re-read `decision.json` and run this step whenever
`decision.retain_candidate` is false. Create
`docs/superpowers/verification/2026-07-17-extractum-process-extraction.md`
with the document structure in Step 5, status `not_retained`, and literal
baseline/post/repeat metrics. For `protocol_completed`, state that the shell
cap failed. For `completion_failure`, name the failed gate from
`completion-failure.json`, list prior gates that passed, and state that later
gates were skipped. Do not describe either branch as retained.

- [ ] **Step 3B: On the rejected path, revert the candidate non-destructively**

Run this step only after the negative evidence file exists:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-process-current.txt') -Raw).Trim()
$candidate = (Get-Content (Join-Path $scratch 'candidate-commit.txt') -Raw).Trim()
git revert --no-edit $candidate
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
if (Test-Path 'src-tauri/crates/extractum-process') { exit 1 }
foreach ($path in @(
  'src-tauri/src/external_process.rs',
  'src-tauri/src/child_process.rs',
  'src-tauri/src/process_tree.rs'
)) {
  if (-not (Test-Path $path)) { throw "Rollback did not restore $path" }
}
```

Expected: one revert commit restores the pre-candidate implementation; the
verification document remains uncommitted for Step 6.

- [ ] **Step 4B: On the rejected path, verify the restored workspace**

Run:

```powershell
npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts src/lib/hidden-child-process-contract.test.ts
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib external_process::
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: restored source contracts, workspace check, and 12 coordinator tests
pass. The candidate boundary contract was removed by the revert.

- [ ] **Step 5: Populate the verification record from literal artifacts**

Create or complete
`docs/superpowers/verification/2026-07-17-extractum-process-extraction.md`
with these headings and actual values; do not prefill PASS values:

```markdown
# Extractum Process Extraction Verification

**Date:** 2026-07-17
**Baseline commit:** `<environment.txt head>`
**Candidate commit:** `<candidate-commit.txt>`
**Outcome:** `<retained | not_retained>`

## Environment

- Cargo/Rust versions: `<literal environment.txt values>`
- Measurement session/attempt IDs: `<valid baseline, post, and optional repeat>`
- Invalidated attempts: `<paths and reasons, or none; excluded from medians>`
- Power profile: `<literal value>`
- Defender state: `<literal value>`
- Canonical target: `<absolute path>`
- Platform/host: `Windows / x86_64-pc-windows-msvc`
- Cross-target scope: `not an acceptance gate for Windows-only Phase 3`

## Boundary Evidence

- Process fan-in and architectural justification: `<summary>`
- Consumer hash comparison: `12 unchanged / 12`
- Public API review: `<exact exposed names>`
- Direct dependency roots: `anyhow`, `parking_lot`, `tokio`, `windows-sys`

## Test Inventory

- Baseline total: `<baseline_count>`
- Candidate total: `<post_count>`
- Missing baseline tests: `<missing_count>`
- Process tests before/after: `<20 / 20>`

## Measurements

| Series | Samples (ms) | Median (ms) |
| --- | --- | ---: |
| Baseline app-domain | `<five values>` | `<value>` |
| Candidate focused process | `<five values>` | `<value>` |
| Baseline app-shell | `<five values>` | `<value>` |
| Candidate app-shell | `<five values>` | `<value>` |
| Reserved/repeat shell | `<values or not used>` | `<value or n/a>` |

- Shell delta: `<ms>` / `<percent>`
- Primary cap pass: `<true/false>`
- Repeat used/pass: `<values>`
- Focused process timing role: diagnostic only

## Verification

- Boundary RED observation: `<literal failure reason>`
- Boundary GREEN: `<literal Vitest count>`
- Focused process tests/check: `<literal results>`
- Windows process-crate check: `<literal result>`
- App dependent checkpoint: `<literal result>`
- Completion outcome: `<passed | failed gate name from completion-failure.json>`
- Workspace check/test: `<literal result or skipped on negative path>`
- `npm.cmd run verify`: `<literal result or skipped>`
- Release no-bundle/startup smoke: `<literal result or skipped>`
- MSI/WiX: excluded due to pre-existing baseline failure

## Decision

`<decision.json rendered in prose; include rollback commit when not retained>`
```

Expected: every claim points to a command output or scratch artifact. A
negative record clearly distinguishes skipped gates from passing gates.

Update `docs/superpowers/specs/2026-07-17-crate-roadmap.md` from the literal
decision at the same time:

- retained: mark Phase 3 completed with the candidate commit and verification
  document, and identify Phase 4 planning as the next authorized step;
- not retained: mark Phase 3 not retained with the exact performance or
  completion-failure reason and rollback commit, and keep Phase 4 blocked
  pending a new approved design.

Do not change phase ordering or thresholds while recording the result.

- [ ] **Step 6: Verify scope and commit the evidence**

On either path, the only uncommitted files must be the verification document
and roadmap status update. On the rejected path, the revert commit is already
clean before these two documentation files are staged.

Run:

```powershell
$expected = @(
  'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
  'docs/superpowers/verification/2026-07-17-extractum-process-extraction.md'
)
$changed = @(git status --short --untracked-files=all | ForEach-Object { $_.Substring(3) })
$changed
$extra = @($changed | Where-Object { $_ -notin $expected })
$missing = @($expected | Where-Object { $_ -notin $changed })
if ($extra.Count -ne 0 -or $missing.Count -ne 0) { exit 1 }
git diff --check
git add -- $expected
git diff --cached --check
git commit -m "docs: record extractum process extraction"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git status --short
```

Expected: evidence commit succeeds and the working tree is clean.

- [ ] **Step 7: Report the retained or rolled-back result without starting Phase 4**

Report:

- candidate and evidence commit IDs;
- retained/reverted outcome;
- exact shell delta and whether repeat was used;
- process test inventory result;
- Windows host and process-crate check result;
- full-gate results or explicitly skipped gates;
- release smoke result when retained.

Stop. The Gemini Browser implementation plan is a separate artifact written
only after a retained Phase 3 result.
