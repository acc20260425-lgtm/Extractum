# NotebookLM Render Crate Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task by task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Determine cheaply whether an extracted Rust dependency improves the
full incremental workspace-check loop and, only after a positive preflight,
extract the seven pure NotebookLM rendering modules without changing behavior.

**Architecture:** Stage 0 compares the existing application boundary with the
already-extracted `extractum-core` boundary under the canonical full-workspace
check. A failed preflight ends the plan with documentation only. A passing
preflight unlocks a mechanical move into `extractum-notebooklm-render`, with
explicit compatibility facades in the application and a second measured
retention decision.

**Tech Stack:** Rust 2021, Cargo workspaces, Tauri 2, serde, serde_json, time,
PowerShell 5.1 on Windows, Node.js ESM, Vitest source-contract tests.

## Global Constraints

- Follow
  `docs/superpowers/specs/2026-07-17-notebooklm-render-crate-boundary-design.md`.
- The approved spec and this plan must be committed, and the worktree must be
  clean, before Stage 0 begins.
- Stage 0 is a hard gate. Do not edit manifests, create the render crate, move
  Rust source, or add the boundary contract unless both preflight thresholds
  pass.
- The Stage 0 surrogate passes only when its median full-workspace check is at
  least 25% and at least 2.0 seconds faster than the application-domain median.
- The surrogate is conservative: `extractum-core` has its own test targets and
  is shared more broadly than the proposed render crate. Record this bias, but
  do not relax either threshold after observing results.
- Use `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`
  for every full-workspace probe.
- Use the existing canonical `src-tauri/target`; do not set
  `CARGO_TARGET_DIR`, use `--target-dir`, or run `cargo clean`.
- Do not measure while Cargo, rustc, rust-analyzer, Tauri, Vite, Vitest, or
  Extractum processes are active.
- Store raw timing logs, probe metadata, source snapshots, and generated
  summaries under an absolute `$env:TEMP` directory, never in the repository.
- Every probe restores its source file byte-for-byte and verifies its original
  SHA-256 before another probe starts.
- Missing metadata, failure before Cargo starts, or an unconfirmed runner
  result invalidates the entire measurement session. Restart from both
  warm-ups; never replace one sample in place.
- A complete metadata record with a nonzero Cargo exit is a confirmed probe
  failure. Stop and investigate; do not treat it as a timing sample or a
  performance no-go.
- Focused package timings are diagnostic only and cannot override a failed
  full-workspace Stage 0.
- If Stage 1 is reached, the candidate passes only with a domain improvement
  of at least 25% and 2.0 seconds, shell regression no greater than both 5% and
  0.5 seconds, preserved test inventory, and all correctness gates green.
- Move only `model`, `filename`, `links`, `media`, `renderer`, `glossary`, and
  `chunker`. Keep `query`, `message_mapping`, command/event orchestration,
  SQLx, filesystem work, and `AppError` in the application.
- The new crate may depend directly only on `extractum-core`, `serde`,
  `serde_json`, and `time`.
- Preserve DTO derives, fields, field order, serde attributes, defaults,
  serialized names, Markdown, filenames, validation behavior, events, and
  error behavior.
- Use explicit public modules, explicit re-exports, and explicit application
  facades. Glob imports and glob re-exports are forbidden.
- Use `npm.cmd`, not plain `npm`, on Windows.
- Preserve unrelated user changes. Inspect the worktree before each commit and
  stage only paths owned by the current task.

---

### Task 1: Establish the Stage 0 Measurement Session

**Files:**
- Read: `src-tauri/src/notebooklm_export/renderer.rs`
- Read: `src-tauri/crates/extractum-core/src/media_metadata.rs`
- Read: `src-tauri/Cargo.toml`
- Create outside repository: a directory named
  `extractum-notebooklm-render-` followed by the exact starting commit hash
  under `$env:TEMP`

**Interfaces:**
- Produces an absolute scratch locator at
  `$env:TEMP/extractum-notebooklm-render-current.txt`.
- Produces `environment.txt`, two source snapshots, and a reusable probe runner
  in the scratch directory.

- [ ] **Step 1: Require a clean, committed starting state**

Run:

```powershell
$ErrorActionPreference = 'Stop'
$spec = 'docs/superpowers/specs/2026-07-17-notebooklm-render-crate-boundary-design.md'
$plan = 'docs/superpowers/plans/2026-07-17-notebooklm-render-crate-boundary.md'
$status = @(git status --short --untracked-files=all)
$specTracked = @(git ls-files --error-unmatch $spec 2>$null).Count -eq 1
$planTracked = @(git ls-files --error-unmatch $plan 2>$null).Count -eq 1
"STATUS_COUNT=$($status.Count)"
"SPEC_TRACKED=$specTracked"
"PLAN_TRACKED=$planTracked"
if ($status.Count -ne 0 -or -not $specTracked -or -not $planTracked) { exit 1 }
```

Expected: clean worktree and both documents tracked at `HEAD`. Stop before
measurement if any condition fails.

- [ ] **Step 2: Record the environment and reject active build processes**

Run:

```powershell
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path '.').Path
$head = (git rev-parse HEAD).Trim()
$scratch = Join-Path $env:TEMP "extractum-notebooklm-render-$head"
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$scratch | Set-Content -LiteralPath (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt')

$active = @(Get-CimInstance Win32_Process | Where-Object {
  $_.Name -in @('cargo.exe','rustc.exe','rust-analyzer.exe','extractum.exe') -or
  ($_.Name -eq 'node.exe' -and $_.CommandLine -match '(?i)(vitest|vite(?:\.js)?|tauri)')
})
$metadataText = & cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --no-deps
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$target = [IO.Path]::GetFullPath(($metadataText | ConvertFrom-Json).target_directory)
$expectedTarget = [IO.Path]::GetFullPath((Join-Path $repo 'src-tauri/target'))
$power = (& powercfg /getactivescheme | Out-String).Trim()
$defender = try {
  (Get-MpComputerStatus -ErrorAction Stop).RealTimeProtectionEnabled
} catch {
  'unavailable'
}
@(
  "HEAD=$head"
  "DATE=$([DateTimeOffset]::Now.ToString('o'))"
  "OS=$([Environment]::OSVersion.VersionString)"
  "CPU=$env:PROCESSOR_IDENTIFIER"
  "LOGICAL_CORES=$([Environment]::ProcessorCount)"
  "RUSTC=$(& rustc -Vv | Out-String)"
  "CARGO=$(& cargo -V)"
  "POWER_PROFILE=$power"
  "DEFENDER_REALTIME=$defender"
  "TARGET=$target"
  "EXPECTED_TARGET=$expectedTarget"
  "ACTIVE_PROCESS_COUNT=$($active.Count)"
) | Set-Content -LiteralPath (Join-Path $scratch 'environment.txt')
if ($active.Count -ne 0 -or $target -ne $expectedTarget) { exit 1 }
```

Expected: zero selected processes and target directory exactly
`src-tauri/target`. Close the editor if rust-analyzer cannot remain idle.

- [ ] **Step 3: Snapshot both probe files byte-for-byte**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$probes = @{
  app = 'src-tauri/src/notebooklm_export/renderer.rs'
  core = 'src-tauri/crates/extractum-core/src/media_metadata.rs'
}
foreach ($entry in $probes.GetEnumerator()) {
  $resolved = (Resolve-Path $entry.Value).Path
  $bytes = [IO.File]::ReadAllBytes($resolved)
  [IO.File]::WriteAllBytes((Join-Path $scratch "$($entry.Key)-original.bin"), $bytes)
  @{
    variant = $entry.Key
    path = $entry.Value
    sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
    length = $bytes.Length
  } | ConvertTo-Json | Set-Content (Join-Path $scratch "$($entry.Key)-source.json")
}
```

Expected: two snapshots and two metadata files outside the repository.

- [ ] **Step 4: Create the reusable full-workspace probe runner outside the repository**

Create `$scratch/invoke-probe.ps1` with this exact content:

```powershell
param(
  [Parameter(Mandatory = $true)][string]$SourceKey,
  [Parameter(Mandatory = $true)][string]$Label,
  [switch]$FocusedCore
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path '.').Path
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$sourceMeta = Get-Content (Join-Path $scratch "$SourceKey-source.json") -Raw | ConvertFrom-Json
$sourcePath = [IO.Path]::GetFullPath((Join-Path $repo $sourceMeta.path))
$originalPath = Join-Path $scratch "$SourceKey-original.bin"
$runDir = Join-Path $scratch 'runs'
New-Item -ItemType Directory -Force -Path $runDir | Out-Null
$metaPath = Join-Path $runDir "$Label-meta.json"
$stdoutPath = Join-Path $runDir "$Label.stdout.log"
$stderrPath = Join-Path $runDir "$Label.stderr.log"

function Get-Sha256([string]$Path) {
  (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
}

$meta = [ordered]@{
  label = $Label
  source_key = $SourceKey
  focused_core = [bool]$FocusedCore
  cargo_started = $false
  exit = $null
  elapsed_ms = $null
  restored = $false
  original_sha256 = $sourceMeta.sha256
  changed_sha256 = $null
}
$meta | ConvertTo-Json | Set-Content -LiteralPath $metaPath

if ((Get-Sha256 $sourcePath) -ne $sourceMeta.sha256) {
  throw "Probe source does not match its original hash: $sourcePath"
}

$noopArgs = @('check','--manifest-path','src-tauri/Cargo.toml','--workspace','--all-targets')
& cargo @noopArgs
if ($LASTEXITCODE -ne 0) { throw 'Pre-probe no-op check failed' }

$original = [IO.File]::ReadAllBytes($originalPath)
$suffix = [Text.Encoding]::UTF8.GetBytes("`n// cargo-measurement-probe: $Label`n")
$changed = New-Object byte[] ($original.Length + $suffix.Length)
[Array]::Copy($original, 0, $changed, 0, $original.Length)
[Array]::Copy($suffix, 0, $changed, $original.Length, $suffix.Length)

try {
  [IO.File]::WriteAllBytes($sourcePath, $changed)
  $meta.changed_sha256 = Get-Sha256 $sourcePath
  $cargoArgs = if ($FocusedCore) {
    @('check','--manifest-path','src-tauri/Cargo.toml','-p','extractum-core','--all-targets')
  } else {
    $noopArgs
  }
  $meta.cargo_started = $true
  $meta | ConvertTo-Json | Set-Content -LiteralPath $metaPath
  $watch = [Diagnostics.Stopwatch]::StartNew()
  $process = Start-Process -FilePath 'cargo.exe' -ArgumentList $cargoArgs `
    -WorkingDirectory $repo -NoNewWindow -Wait -PassThru `
    -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
  $watch.Stop()
  $meta.exit = $process.ExitCode
  $meta.elapsed_ms = [int64]$watch.ElapsedMilliseconds
} finally {
  [IO.File]::WriteAllBytes($sourcePath, $original)
  $meta.restored = (Get-Sha256 $sourcePath) -eq $sourceMeta.sha256
  $meta | ConvertTo-Json | Set-Content -LiteralPath $metaPath
}

if (-not $meta.restored) { throw 'Probe source restoration failed' }
if ($meta.exit -ne 0) { exit $meta.exit }
```

Expected: the runner exists only under `$scratch`; no repository file changed.

---

### Task 2: Execute and Decide Stage 0

**Files:**
- Read/temporarily edit and restore:
  `src-tauri/src/notebooklm_export/renderer.rs`
- Read/temporarily edit and restore:
  `src-tauri/crates/extractum-core/src/media_metadata.rs`
- Create: `docs/superpowers/verification/2026-07-17-notebooklm-render-crate-boundary.md`

**Interfaces:**
- Consumes `$scratch/invoke-probe.ps1` from Task 1.
- Produces `stage0-summary.json` with medians, deltas, gate booleans, and
  `decision` equal to `go` or `no_go`.
- A `no_go` outcome terminates this plan after the verification commit.

- [ ] **Step 1: Run the two discarded warm-ups**

Run sequentially:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-probe.ps1'
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey app -Label 'warmup-app'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey core -Label 'warmup-core'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: both pass and both files return to their recorded SHA-256. Warm-up
durations are never included in medians. Missing/incomplete metadata is an
infrastructure failure and invalidates the session.

- [ ] **Step 2: Run five alternating recorded pairs**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-probe.ps1'
for ($index = 1; $index -le 5; $index++) {
  foreach ($variant in @('app','core')) {
    $label = "recorded-$index-$variant"
    & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey $variant -Label $label
    if ($LASTEXITCODE -ne 0) {
      $metaPath = Join-Path $scratch "runs/$label-meta.json"
      if (-not (Test-Path $metaPath)) { throw 'Infrastructure failure: metadata absent' }
      $meta = Get-Content $metaPath -Raw | ConvertFrom-Json
      if (-not $meta.cargo_started -or $null -eq $meta.exit) {
        throw 'Infrastructure failure: invalidate session and restart from warm-ups'
      }
      throw "Confirmed Cargo probe failure for $label; stop and investigate"
    }
  }
}
```

Expected: ten valid recorded samples. Do not rerun only a failed member of a
pair.

- [ ] **Step 3: Measure the focused core loop for diagnostic context**

Run one discarded warm-up and five recorded samples using the core probe:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-probe.ps1'
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey core -Label 'focused-warmup' -FocusedCore
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
for ($index = 1; $index -le 5; $index++) {
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey core `
    -Label "focused-$index" -FocusedCore
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Expected: five diagnostic samples. They cannot alter the full-workspace gate.

- [ ] **Step 4: Compute medians and the predeclared decision**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
function Get-Median([long[]]$Values) {
  $sorted = @($Values | Sort-Object)
  if ($sorted.Count -eq 0) { throw 'Median requires at least one value' }
  if ($sorted.Count % 2 -eq 1) { return [double]$sorted[[int]($sorted.Count / 2)] }
  return ($sorted[$sorted.Count / 2 - 1] + $sorted[$sorted.Count / 2]) / 2.0
}
$all = @(Get-ChildItem (Join-Path $scratch 'runs') -Filter '*-meta.json' |
  ForEach-Object { Get-Content $_.FullName -Raw | ConvertFrom-Json })
$app = @($all | Where-Object { $_.label -match '^recorded-\d+-app$' })
$core = @($all | Where-Object { $_.label -match '^recorded-\d+-core$' })
$focused = @($all | Where-Object { $_.label -match '^focused-\d+$' })
foreach ($set in @($app,$core,$focused)) {
  if ($set.Count -ne 5 -or @($set | Where-Object {
    -not $_.cargo_started -or $_.exit -ne 0 -or -not $_.restored -or $null -eq $_.elapsed_ms
  }).Count -ne 0) { throw 'Invalid or incomplete Stage 0 sample set' }
}
$appMedian = Get-Median @($app.elapsed_ms)
$coreMedian = Get-Median @($core.elapsed_ms)
$focusedMedian = Get-Median @($focused.elapsed_ms)
$absoluteImprovementMs = $appMedian - $coreMedian
$percentImprovement = if ($appMedian -eq 0) { 0 } else {
  100.0 * $absoluteImprovementMs / $appMedian
}
$summary = [ordered]@{
  app_samples_ms = @($app.elapsed_ms)
  core_samples_ms = @($core.elapsed_ms)
  focused_core_samples_ms = @($focused.elapsed_ms)
  app_median_ms = $appMedian
  core_median_ms = $coreMedian
  focused_core_median_ms = $focusedMedian
  absolute_improvement_ms = $absoluteImprovementMs
  percent_improvement = $percentImprovement
  passes_percent = $percentImprovement -ge 25.0
  passes_absolute = $absoluteImprovementMs -ge 2000.0
}
$summary.decision = if ($summary.passes_percent -and $summary.passes_absolute) { 'go' } else { 'no_go' }
$summary | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $scratch 'stage0-summary.json')
$summary | Format-List
```

Expected: exactly five values per series and a decision computed without
changing thresholds.

- [ ] **Step 5: Prove byte restoration and a clean repository**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
foreach ($variant in @('app','core')) {
  $meta = Get-Content (Join-Path $scratch "$variant-source.json") -Raw | ConvertFrom-Json
  $actual = (Get-FileHash $meta.path -Algorithm SHA256).Hash
  if ($actual -ne $meta.sha256) { throw "$variant probe was not restored" }
}
$status = @(git status --short --untracked-files=all)
if ($status.Count -ne 0) { $status; exit 1 }
```

Expected: both original hashes match and the worktree is clean.

- [ ] **Step 6: Write the Stage 0 verification record**

Create
`docs/superpowers/verification/2026-07-17-notebooklm-render-crate-boundary.md`.
Write the title `NotebookLM Render Crate Boundary Verification`, date, exact
starting commit from `environment.txt`, and exact `go`/`no_go` decision from
`stage0-summary.json`. Add an Environment section containing the recorded
toolchain, CPU, power profile, Defender state, and canonical target. Add a
Stage 0 Samples table with all five values and median for application renderer,
extracted core surrogate, and focused core diagnostic. Add a Decision section
with the observed percent and absolute improvements next to the fixed `25%`
and `2000 ms` requirements. Add an Integrity section confirming both restored
hashes, exclusion of warm-ups, absence of replacement samples, and the absolute
scratch path. State that the core surrogate conservatively includes core's own
test targets and represents a more broadly shared dependency.

If the decision is `no_go`, also state explicitly that no manifest, Rust
source, workspace member, or boundary contract was changed and that Tasks 3-8
were not executed. If it is `go`, label Stage 1 as pending.

- [ ] **Step 7: Commit the Stage 0 evidence and obey the decision**

Run:

```powershell
$path = 'docs/superpowers/verification/2026-07-17-notebooklm-render-crate-boundary.md'
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git add -- $path
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git commit -m "docs: record notebooklm render preflight"
```

Expected: one documentation-only commit. Read `decision` from
`stage0-summary.json`. On `no_go`, mark this plan complete and stop. Execute
Task 3 only on `go`.

---

### Task 3: Capture Stage 1 Baselines and the Complete Test Inventory

**Condition:** Execute only when Task 2 recorded `decision = go`.

**Files:**
- Read/temporarily edit and restore:
  `src-tauri/src/notebooklm_export/renderer.rs`
- Read/temporarily edit and restore:
  `src-tauri/src/notebooklm_export/mod.rs`
- Read: all Cargo workspace sources for test inventory

**Produces:** Stage 1 baseline domain/shell sample sets, one Cargo timing
report reference, no-op samples, complete test-name inventory, and the explicit
22-entry rename map in scratch.

- [ ] **Step 1: Capture the complete baseline test inventory**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$log = Join-Path $scratch 'stage1-baseline-test-inventory.log'
& cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- --list 2>&1 |
  Tee-Object -FilePath $log
$exit = $LASTEXITCODE
$names = @(Get-Content $log | ForEach-Object { $_.ToString() } |
  Where-Object { $_ -match ': test$' } |
  ForEach-Object { ($_ -replace ': test$', '').Trim() })
$names | Sort-Object | Set-Content (Join-Path $scratch 'stage1-baseline-test-names.txt')
if ($exit -ne 0 -or $names.Count -eq 0 -or @($names | Sort-Object -Unique).Count -ne $names.Count) {
  exit 1
}
```

Expected: successful, nonzero, unique complete inventory.

- [ ] **Step 2: Save the exact pure-test rename map**

Create `$scratch/pure-test-rename-map.json` with these 22 entries. `model` has
zero tests and therefore no map entry.

```json
{
  "notebooklm_export::filename::tests::sanitizes_unsafe_filename_parts": "filename::tests::sanitizes_unsafe_filename_parts",
  "notebooklm_export::filename::tests::rejects_reserved_components": "filename::tests::rejects_reserved_components",
  "notebooklm_export::filename::tests::child_paths_stay_under_base": "filename::tests::child_paths_stay_under_base",
  "notebooklm_export::filename::tests::accepts_safe_relative_child_paths": "filename::tests::accepts_safe_relative_child_paths",
  "notebooklm_export::filename::tests::rejects_unsafe_relative_child_paths": "filename::tests::rejects_unsafe_relative_child_paths",
  "notebooklm_export::links::tests::detects_and_trims_http_urls": "links::tests::detects_and_trims_http_urls",
  "notebooklm_export::media::tests::renders_useful_media_placeholder_parts": "media::tests::renders_useful_media_placeholder_parts",
  "notebooklm_export::media::tests::renders_numeric_only_media_metadata": "media::tests::renders_numeric_only_media_metadata",
  "notebooklm_export::renderer::tests::formats_metadata_as_rfc3339": "renderer::tests::formats_metadata_as_rfc3339",
  "notebooklm_export::renderer::tests::renders_message_metadata_and_text": "renderer::tests::renders_message_metadata_and_text",
  "notebooklm_export::renderer::tests::renders_reply_thread_and_reaction_metadata": "renderer::tests::renders_reply_thread_and_reaction_metadata",
  "notebooklm_export::renderer::tests::renders_migrated_history_scope_metadata": "renderer::tests::renders_migrated_history_scope_metadata",
  "notebooklm_export::renderer::tests::renders_json_compatible_yaml_string_scalars": "renderer::tests::renders_json_compatible_yaml_string_scalars",
  "notebooklm_export::renderer::tests::renders_topic_aware_document_header": "renderer::tests::renders_topic_aware_document_header",
  "notebooklm_export::glossary::tests::aggregates_participants_by_author": "glossary::tests::aggregates_participants_by_author",
  "notebooklm_export::chunker::tests::filters_short_text_without_other_signal": "chunker::tests::filters_short_text_without_other_signal",
  "notebooklm_export::chunker::tests::keeps_yearly_group_when_within_limits": "chunker::tests::keeps_yearly_group_when_within_limits",
  "notebooklm_export::chunker::tests::falls_back_to_month_when_year_exceeds_limits": "chunker::tests::falls_back_to_month_when_year_exceeds_limits",
  "notebooklm_export::chunker::tests::splits_by_word_and_byte_limits": "chunker::tests::splits_by_word_and_byte_limits",
  "notebooklm_export::chunker::tests::accounts_for_document_overhead_when_splitting": "chunker::tests::accounts_for_document_overhead_when_splitting",
  "notebooklm_export::chunker::tests::groups_chunks_by_topic_slug": "chunker::tests::groups_chunks_by_topic_slug",
  "notebooklm_export::chunker::tests::falls_back_to_topic_id_when_topic_title_slug_is_invalid": "chunker::tests::falls_back_to_topic_id_when_topic_title_slug_is_invalid"
}
```

Expected: exactly 22 unique old keys and 22 unique new values; every old key
exists in the baseline inventory.

- [ ] **Step 3: Capture fresh Stage 1 domain and shell baselines**

The `app` source key already points to the baseline renderer. Add a `shell`
source snapshot with:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$path = 'src-tauri/src/notebooklm_export/mod.rs'
$bytes = [IO.File]::ReadAllBytes((Resolve-Path $path).Path)
[IO.File]::WriteAllBytes((Join-Path $scratch 'shell-original.bin'), $bytes)
@{
  variant = 'shell'
  path = $path
  sha256 = (Get-FileHash $path -Algorithm SHA256).Hash
  length = $bytes.Length
} | ConvertTo-Json | Set-Content (Join-Path $scratch 'shell-source.json')
```

Run discarded warm-ups and five alternating pairs:

```powershell
$runner = Join-Path $scratch 'invoke-probe.ps1'
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey app -Label 'stage1-baseline-warmup-domain'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey shell -Label 'stage1-baseline-warmup-shell'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
for ($index = 1; $index -le 5; $index++) {
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey app -Label "stage1-baseline-domain-$index"
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey shell -Label "stage1-baseline-shell-$index"
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Validate and summarize with:

```powershell
function Get-Median([long[]]$Values) {
  $sorted = @($Values | Sort-Object)
  if ($sorted.Count -eq 0) { throw 'Median requires at least one value' }
  if ($sorted.Count % 2 -eq 1) { return [double]$sorted[[int]($sorted.Count / 2)] }
  return ($sorted[$sorted.Count / 2 - 1] + $sorted[$sorted.Count / 2]) / 2.0
}
$metas = @(Get-ChildItem (Join-Path $scratch 'runs') -Filter 'stage1-baseline-*-meta.json' |
  ForEach-Object { Get-Content $_.FullName -Raw | ConvertFrom-Json })
$domain = @($metas | Where-Object { $_.label -match '^stage1-baseline-domain-\d+$' })
$shell = @($metas | Where-Object { $_.label -match '^stage1-baseline-shell-\d+$' })
foreach ($set in @($domain,$shell)) {
  if ($set.Count -ne 5 -or @($set | Where-Object {
    -not $_.cargo_started -or $_.exit -ne 0 -or -not $_.restored -or $null -eq $_.elapsed_ms
  }).Count -ne 0) { throw 'Invalid Stage 1 baseline sample set' }
}
@{
  domain_samples_ms = @($domain.elapsed_ms)
  domain_median_ms = Get-Median @($domain.elapsed_ms)
  shell_samples_ms = @($shell.elapsed_ms)
  shell_median_ms = Get-Median @($shell.elapsed_ms)
} | ConvertTo-Json -Depth 3 | Set-Content (Join-Path $scratch 'stage1-baseline-summary.json')
```

Any infrastructure or confirmed Cargo failure follows the Global Constraints;
do not replace a single sample.

Expected: five valid samples per variant and original hashes restored.

- [ ] **Step 4: Capture diagnostic no-op values and Cargo timings**

Run five sequential no-op full-workspace checks, recording logs and wall time.
Then apply one measured domain comment, run:

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --workspace --all-targets --timings
```

Record the newest `src-tauri/target/cargo-timings/cargo-timing*.html` path and
SHA-256 in scratch. Apply the comment from a saved byte array and wrap the
build in `try/finally`; the `finally` block must restore the saved bytes and
verify the starting SHA-256 before one restoring check. These values are
diagnostic, not retention gates.

---

### Task 4: Add the RED Render-Boundary Contract

**Condition:** Execute only after a positive Stage 0 and valid Stage 1 baseline.

**Files:**
- Create: `src/lib/notebooklm-render-crate-contract.test.ts`
- Modify if stale: `src/lib/rust-workspace-core-contract.test.ts`
- Read: `src-tauri/Cargo.toml`
- Read: `src-tauri/src/notebooklm_export/mod.rs`

**Contract:** Protect the workspace member, four dependency roots, curated API,
absence of forbidden dependencies, explicit app facades, removal of old files,
and movement rather than copying of all 22 pure tests.

- [ ] **Step 1: Create the failing source contract**

Create `src/lib/notebooklm-render-crate-contract.test.ts` using normalized
`node:fs` reads. The test must:

1. require workspace member `"crates/extractum-notebooklm-render"`;
2. parse the new manifest's `[dependencies]` keys and expect the sorted array
   `['extractum-core', 'serde', 'serde_json', 'time']` exactly;
3. require seven explicit `pub mod` declarations and reject `pub use ...::*`;
4. scan all seven new modules and reject `tauri`, `sqlx`, `grammers`,
   `crate::sources`, `crate::notebooklm_export`, DB/readiness imports, and glob
   imports;
5. require the seven old app `.rs` files not to exist;
6. require seven inline compatibility modules in app `mod.rs`, each with an
   explicit `pub(crate) use` from its matching
   `extractum_notebooklm_render::{chunker,filename,glossary,links,media,model,renderer}`
   module and no glob;
7. hold the exact 22 names from Task 3 in an array, print any offenders, and
   assert every old app source lacks them while the matching new source
   contains them.

Use `readOptionalSource()` for files that do not yet exist so RED is a normal
assertion failure rather than an import-time exception.

- [ ] **Step 2: Run the focused contract and full Vitest suite to verify RED**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/notebooklm-render-crate-contract.test.ts
```

Expected: FAIL and identify the missing crate plus all current old module
paths. A zero-test run fails this step.

Then run:

```powershell
npm.cmd run test
```

Expected: FAIL only for the new contract and any existing exact workspace
allowlist that must be updated in Task 5. Record every stale contract before
editing implementation files.

---

### Task 5: Create the Render Crate and Compatibility Boundary

**Condition:** Execute only after Task 4 produced the expected RED.

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/notebooklm_export/mod.rs`
- Create: `src-tauri/crates/extractum-notebooklm-render/Cargo.toml`
- Create: `src-tauri/crates/extractum-notebooklm-render/src/lib.rs`
- Move and adapt:
  `src-tauri/src/notebooklm_export/{model,filename,links,media,renderer,glossary,chunker}.rs`
- Delete after successful move: the seven old app source files above
- Modify: `src/lib/notebooklm-render-crate-contract.test.ts`
- Modify if stale: `src/lib/rust-workspace-core-contract.test.ts`

**Interfaces:**
- Produces package `extractum-notebooklm-render` and crate
  `extractum_notebooklm_render`.
- Preserves current app paths through private inline modules in
  `notebooklm_export/mod.rs`.
- Keeps exactly 22 pure tests with the rename map from Task 3.

- [ ] **Step 1: Add the workspace and dependency declarations**

In `src-tauri/Cargo.toml`:

1. add `"crates/extractum-notebooklm-render"` to `[workspace].members`;
2. add `extractum-core = { path = "crates/extractum-core" }` under
   `[workspace.dependencies]` and change the root package declaration to
   `extractum-core = { workspace = true }`;
3. add
   `extractum-notebooklm-render = { path = "crates/extractum-notebooklm-render" }`
   under `[workspace.dependencies]`;
4. add `extractum-notebooklm-render = { workspace = true }` under the root
   package `[dependencies]`;
5. leave all external versions, profiles, target settings, and external
   dependency declarations unchanged.

Step 2 is required before the new crate can inherit
`extractum-core.workspace = true`.

Create the new manifest exactly as:

```toml
[package]
name = "extractum-notebooklm-render"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
extractum-core.workspace = true
serde.workspace = true
serde_json.workspace = true
time.workspace = true
```

- [ ] **Step 2: Move the seven modules without rewriting behavior**

Move each current file into
`src-tauri/crates/extractum-notebooklm-render/src/`. Preserve function bodies,
tests, strings, derives, serde annotations, and declaration order. Apply only
these import adaptations:

- `model.rs`: `crate::media::ItemMediaMetadata` becomes
  `extractum_core::media_metadata::ItemMediaMetadata`;
- `media.rs`: `crate::media::{media_label, ItemMediaMetadata}` becomes the
  equivalent explicit `extractum_core::media_metadata::{...}` import;
- `renderer.rs` imports model items from `crate::model`;
- `glossary.rs` imports model items from `crate::model` and renderer helpers
  from `crate::renderer`;
- `chunker.rs` imports filename/model items from `crate::filename` and
  `crate::model`;
- test-only `crate::media::ItemMediaMetadata` imports become the explicit core
  import;
- in the `renderer`, `glossary`, and `chunker` test modules, define
  `const CURRENT_HISTORY_SCOPE: &str = "current_supergroup_history";` and
  replace only their test-fixture references to
  `crate::sources::NOTEBOOKLM_HISTORY_SCOPE_CURRENT_SUPERGROUP` with that local
  constant.

The local test constant is required because the new crate must not depend on
the app `sources` module. It preserves the exact fixture value from
`src-tauri/src/sources/types.rs` and does not change production behavior.

- [ ] **Step 3: Expand only the cross-crate visibility**

Change the current `pub(crate)` visibility to `pub` for:

- all three `DEFAULT_*` constants;
- `NotebookLmExportScope`, both variants, and `event_scope_id`;
- `NotebookLmExportConfig`, all its fields, and `event_scope_id`;
- `NotebookLmExportSource` and all its fields;
- `NotebookLmExportMessage` and all its fields;
- the types `ParticipantSummary`, `RenderedMessageBlock`,
  `ExportTopicDescriptor`, and `ChunkFile`;
- `RenderedMessageBlock::message`, which the application clones while
  assembling exported-message summaries;
- every `ChunkFile` field;
- `DocumentRenderContext` and all its fields;
- the existing public candidate functions:
  `sanitize_path_component`, `is_rejected_component`, `ensure_child_path`,
  `ensure_child_relative_path`, `detect_urls`, `render_media_placeholders`,
  `aggregate_participants`, `render_glossary`, `glossary_word_count`,
  `format_unix_rfc3339`, `approx_word_count`, `render_message_block`,
  `render_document`, `render_document_overhead`, `should_export_message`, and
  `build_chunks`.

Keep all `ParticipantSummary` and `ExportTopicDescriptor` fields at
`pub(crate)`. Keep `RenderedMessageBlock::{markdown, approximate_word_count,
byte_size}` at `pub(crate)`. Their construction and remaining inspection stay
inside the render crate; do not broaden them speculatively.

- [ ] **Step 4: Create the curated crate root**

Create `src-tauri/crates/extractum-notebooklm-render/src/lib.rs` with seven
explicit public modules and explicit root re-exports:

```rust
pub mod chunker;
pub mod filename;
pub mod glossary;
pub mod links;
pub mod media;
pub mod model;
pub mod renderer;

pub use chunker::{build_chunks, should_export_message};
pub use filename::{
    ensure_child_path, ensure_child_relative_path, is_rejected_component,
    sanitize_path_component,
};
pub use glossary::{aggregate_participants, glossary_word_count, render_glossary};
pub use links::detect_urls;
pub use media::render_media_placeholders;
pub use model::{
    ChunkFile, ExportTopicDescriptor, NotebookLmExportConfig, NotebookLmExportFile,
    NotebookLmExportMessage, NotebookLmExportRequest, NotebookLmExportResult,
    NotebookLmExportScope, NotebookLmExportSource, ParticipantSummary,
    RenderedMessageBlock, DEFAULT_MAX_BYTES_PER_FILE, DEFAULT_MAX_WORDS_PER_FILE,
    DEFAULT_MIN_MESSAGE_LENGTH,
};
pub use renderer::{
    approx_word_count, format_unix_rfc3339, render_document, render_document_overhead,
    render_message_block, DocumentRenderContext,
};
```

- [ ] **Step 5: Replace the seven app declarations with explicit facades**

In `src-tauri/src/notebooklm_export/mod.rs`, replace only the seven file-backed
declarations. Keep `mod message_mapping;` and `mod query;` unchanged.

```rust
mod chunker {
    pub(crate) use extractum_notebooklm_render::chunker::{build_chunks, should_export_message};
}
mod filename {
    pub(crate) use extractum_notebooklm_render::filename::{
        ensure_child_path, ensure_child_relative_path, sanitize_path_component,
    };
}
mod glossary {
    pub(crate) use extractum_notebooklm_render::glossary::{
        aggregate_participants, glossary_word_count, render_glossary,
    };
}
mod links {
    pub(crate) use extractum_notebooklm_render::links::detect_urls;
}
mod media {
    pub(crate) use extractum_notebooklm_render::media::render_media_placeholders;
}
mod model {
    pub(crate) use extractum_notebooklm_render::model::{
        ChunkFile, NotebookLmExportConfig, NotebookLmExportFile, NotebookLmExportMessage,
        NotebookLmExportRequest, NotebookLmExportResult, NotebookLmExportScope,
        NotebookLmExportSource, ParticipantSummary, DEFAULT_MAX_BYTES_PER_FILE,
        DEFAULT_MAX_WORDS_PER_FILE, DEFAULT_MIN_MESSAGE_LENGTH,
    };
}
mod renderer {
    pub(crate) use extractum_notebooklm_render::renderer::{
        approx_word_count, render_document, render_document_overhead, render_message_block,
        DocumentRenderContext,
    };
}
```

- [ ] **Step 6: Format and run focused compile/test checks**

Run:

```powershell
npm.cmd run check:rustfmt
```

Expected: FAIL until formatting is applied if rustfmt changes the moved files.
Then run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-notebooklm-render --all-targets
```

Expected: all pass; the focused test output executes exactly 22 tests. Zero
tests is a failure.

- [ ] **Step 7: Make the source contract GREEN**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/notebooklm-render-crate-contract.test.ts
npm.cmd run test
```

Expected: the focused contract passes and the full frontend/Vitest inventory
passes. Update only the exact workspace-member/dependency allowlist in
`rust-workspace-core-contract.test.ts` if Task 4 proved it stale.

- [ ] **Step 8: Commit the candidate implementation**

Review `git diff --stat`, `git diff --check`, and the complete diff. Stage only
the manifests/lockfile, seven moved modules, new crate root, app facade, and
contract files listed in this task. Commit:

```powershell
git commit -m "refactor: extract notebooklm render crate"
```

Record this commit hash as `candidate_commit` in scratch. Do not yet call the
candidate retained. Also record the committed shell shape that the post probe
must protect:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$candidateCommit = (git rev-parse HEAD).Trim()
$shellPath = 'src-tauri/src/notebooklm_export/mod.rs'
@{
  candidate_commit = $candidateCommit
  path = $shellPath
  sha256 = (Get-FileHash $shellPath -Algorithm SHA256).Hash
  blob = (git rev-parse "$candidateCommit`:$shellPath").Trim()
} | ConvertTo-Json | Set-Content (Join-Path $scratch 'candidate-shell.json')
```

Expected: the working-tree SHA-256 describes the committed candidate file and
the Git blob resolves at `candidate_commit`.

---

### Task 6: Prove Test-Inventory Preservation

**Condition:** Execute only for the committed Stage 1 candidate.

**Files:**
- Read: `$scratch/stage1-baseline-test-names.txt`
- Read: `$scratch/pure-test-rename-map.json`
- Read: current complete Cargo test inventory

- [ ] **Step 1: Capture the post-extraction complete inventory**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$log = Join-Path $scratch 'stage1-post-test-inventory.log'
& cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- --list 2>&1 |
  Tee-Object -FilePath $log
$exit = $LASTEXITCODE
$names = @(Get-Content $log | ForEach-Object { $_.ToString() } |
  Where-Object { $_ -match ': test$' } |
  ForEach-Object { ($_ -replace ': test$', '').Trim() })
$names | Sort-Object | Set-Content (Join-Path $scratch 'stage1-post-test-names.txt')
if ($exit -ne 0 -or $names.Count -eq 0 -or @($names | Sort-Object -Unique).Count -ne $names.Count) {
  exit 1
}
```

Expected: successful, nonzero, unique complete inventory.

- [ ] **Step 2: Compare every baseline name through the declared map**

Run:

```powershell
$baseline = @(Get-Content (Join-Path $scratch 'stage1-baseline-test-names.txt'))
$post = @(Get-Content (Join-Path $scratch 'stage1-post-test-names.txt'))
$mapObject = Get-Content (Join-Path $scratch 'pure-test-rename-map.json') -Raw | ConvertFrom-Json
$map = @{}
$mapObject.PSObject.Properties | ForEach-Object { $map[$_.Name] = [string]$_.Value }
$expected = @($baseline | ForEach-Object { if ($map.ContainsKey($_)) { $map[$_] } else { $_ } })
$missing = @($expected | Where-Object { $_ -notin $post })
$unexpected = @($post | Where-Object { $_ -notin $expected })
$oldStillPresent = @($map.Keys | Where-Object { $_ -in $post })
$mappedMissing = @($map.Values | Where-Object { $_ -notin $post })
$oldFiles = @('model','filename','links','media','renderer','glossary','chunker') |
  ForEach-Object { "src-tauri/src/notebooklm_export/$_.rs" } |
  Where-Object { Test-Path $_ }
$comparison = [ordered]@{
  baseline_count = $baseline.Count
  post_count = $post.Count
  rename_count = $map.Count
  missing = $missing
  unexpected = $unexpected
  old_names_still_present = $oldStillPresent
  mapped_names_missing = $mappedMissing
  old_files_still_present = @($oldFiles)
  passed = ($baseline.Count -eq $post.Count -and $map.Count -eq 22 -and
    $missing.Count -eq 0 -and $unexpected.Count -eq 0 -and
    $oldStillPresent.Count -eq 0 -and $mappedMissing.Count -eq 0 -and
    @($oldFiles).Count -eq 0)
}
$comparison | ConvertTo-Json -Depth 4 |
  Set-Content (Join-Path $scratch 'stage1-inventory-comparison.json')
if (-not $comparison.passed) { $comparison | Format-List; exit 1 }
```

Expected: equal counts, exactly 22 declared renames, no missing/unexpected
names, no old mapped names, and no old candidate files.

- [ ] **Step 3: Run focused app NotebookLM tests with a nonzero guard**

Run:

```powershell
$log = Join-Path $scratch 'focused-app-notebooklm-tests.log'
& cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib notebooklm_export 2>&1 |
  Tee-Object -FilePath $log
$exit = $LASTEXITCODE
$executed = @(Get-Content $log | Select-String -Pattern 'test result: ok\. ([1-9][0-9]*) passed').Count
if ($exit -ne 0 -or $executed -eq 0) { exit 1 }
```

Expected: successful command and at least one nonzero test-result line.

---

### Task 7: Measure and Decide the Stage 1 Candidate

**Condition:** Execute only after correctness and inventory checks pass.

**Files:**
- Temporarily edit and restore:
  `src-tauri/crates/extractum-notebooklm-render/src/renderer.rs`
- Temporarily edit and restore:
  `src-tauri/src/notebooklm_export/mod.rs`
- Update: `docs/superpowers/verification/2026-07-17-notebooklm-render-crate-boundary.md`

**Produces:** Five post-candidate samples per probe, diagnostic no-op/timing
evidence, and immutable `stage1-decision.json`.

- [ ] **Step 1: Snapshot the post-extraction logical probe files**

Run:

```powershell
$scratch = (Get-Content (Join-Path $env:TEMP 'extractum-notebooklm-render-current.txt') -Raw).Trim()
$candidateShell = Get-Content (Join-Path $scratch 'candidate-shell.json') -Raw | ConvertFrom-Json
$candidateHead = (git rev-parse HEAD).Trim()
$candidateBlob = (git rev-parse "$candidateHead`:$($candidateShell.path)").Trim()
if ($candidateHead -ne $candidateShell.candidate_commit -or
    $candidateBlob -ne $candidateShell.blob -or
    (Get-FileHash $candidateShell.path -Algorithm SHA256).Hash -ne $candidateShell.sha256) {
  throw 'Application shell changed after the candidate commit'
}
$postSources = @{
  'post-domain' = 'src-tauri/crates/extractum-notebooklm-render/src/renderer.rs'
  'post-shell' = 'src-tauri/src/notebooklm_export/mod.rs'
}
foreach ($entry in $postSources.GetEnumerator()) {
  $bytes = [IO.File]::ReadAllBytes((Resolve-Path $entry.Value).Path)
  [IO.File]::WriteAllBytes((Join-Path $scratch "$($entry.Key)-original.bin"), $bytes)
  @{
    variant = $entry.Key
    path = $entry.Value
    sha256 = (Get-FileHash $entry.Value -Algorithm SHA256).Hash
    length = $bytes.Length
  } | ConvertTo-Json | Set-Content (Join-Path $scratch "$($entry.Key)-source.json")
}
```

Expected: the shell matches its committed candidate shape, which intentionally
differs from the pre-extraction baseline because it now contains compatibility
facades; both post snapshots exist outside the repository.

- [ ] **Step 2: Run post-extraction domain and shell series**

Run:

```powershell
$runner = Join-Path $scratch 'invoke-probe.ps1'
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey post-domain -Label 'stage1-post-warmup-domain'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey post-shell -Label 'stage1-post-warmup-shell'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
for ($index = 1; $index -le 5; $index++) {
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey post-domain -Label "stage1-post-domain-$index"
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $runner -SourceKey post-shell -Label "stage1-post-shell-$index"
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

Expected: ten valid samples and both files restored byte-for-byte.

- [ ] **Step 3: Capture post diagnostic no-op and Cargo timing evidence**

Run five sequential no-op commands, each timed with `Stopwatch` and saved under
`$scratch/stage1-post-noop-{1..5}.json`:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

Then use `post-domain-original.bin` plus the exact byte-safe probe suffix from
the runner, execute:

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --workspace --all-targets --timings
```

Restore the renderer in `finally`, verify its SHA-256, and record the newest
`src-tauri/target/cargo-timings/cargo-timing*.html` path and SHA-256. Do not use
no-op or timing-report values as retention gates.

- [ ] **Step 4: Compute the predeclared retention decision**

Run:

```powershell
function Get-Median([long[]]$Values) {
  $sorted = @($Values | Sort-Object)
  if ($sorted.Count -eq 0) { throw 'Median requires at least one value' }
  if ($sorted.Count % 2 -eq 1) { return [double]$sorted[[int]($sorted.Count / 2)] }
  return ($sorted[$sorted.Count / 2 - 1] + $sorted[$sorted.Count / 2]) / 2.0
}
$baseline = Get-Content (Join-Path $scratch 'stage1-baseline-summary.json') -Raw | ConvertFrom-Json
$inventory = Get-Content (Join-Path $scratch 'stage1-inventory-comparison.json') -Raw | ConvertFrom-Json
$metas = @(Get-ChildItem (Join-Path $scratch 'runs') -Filter 'stage1-post-*-meta.json' |
  ForEach-Object { Get-Content $_.FullName -Raw | ConvertFrom-Json })
$domain = @($metas | Where-Object { $_.label -match '^stage1-post-domain-\d+$' })
$shell = @($metas | Where-Object { $_.label -match '^stage1-post-shell-\d+$' })
foreach ($set in @($domain,$shell)) {
  if ($set.Count -ne 5 -or @($set | Where-Object {
    -not $_.cargo_started -or $_.exit -ne 0 -or -not $_.restored -or $null -eq $_.elapsed_ms
  }).Count -ne 0) { throw 'Invalid Stage 1 post sample set' }
}
$domainPost = Get-Median @($domain.elapsed_ms)
$shellPost = Get-Median @($shell.elapsed_ms)
$domainAbsolute = [double]$baseline.domain_median_ms - $domainPost
$domainPercent = 100.0 * $domainAbsolute / [double]$baseline.domain_median_ms
$shellAbsolute = $shellPost - [double]$baseline.shell_median_ms
$shellPercent = 100.0 * $shellAbsolute / [double]$baseline.shell_median_ms
$decision = [ordered]@{
  baseline_domain_median_ms = [double]$baseline.domain_median_ms
  post_domain_median_ms = $domainPost
  baseline_shell_median_ms = [double]$baseline.shell_median_ms
  post_shell_median_ms = $shellPost
  domain_percent_improvement = $domainPercent
  domain_absolute_improvement_ms = $domainAbsolute
  shell_percent_regression = $shellPercent
  shell_absolute_regression_ms = $shellAbsolute
  passes_domain_percent = $domainPercent -ge 25.0
  passes_domain_absolute = $domainAbsolute -ge 2000.0
  passes_shell_percent = $shellPercent -le 5.0
  passes_shell_absolute = $shellAbsolute -le 500.0
  inventory_passed = [bool]$inventory.passed
}
$decision.decision = if ($decision.passes_domain_percent -and
  $decision.passes_domain_absolute -and $decision.passes_shell_percent -and
  $decision.passes_shell_absolute -and $decision.inventory_passed) { 'retain' } else { 'revert' }
$decision | ConvertTo-Json -Depth 3 | Set-Content (Join-Path $scratch 'stage1-decision.json')
$decision | Format-List
```

Expected: `retain` only when all five predeclared gates pass.

- [ ] **Step 5: Update verification evidence before changing candidate state**

Append Stage 1 tables, medians, formulas, inventory comparison, diagnostic
evidence, candidate commit, and the unmodified retention decision to the
verification document. State that release validation is pending only for a
retained candidate.

- [ ] **Step 6: Retain or revert the candidate**

If `retain`, leave the candidate commit in place and continue to Task 8.

If `revert`, create an additive revert commit for the candidate implementation
commit (never reset), verify manifests and all moved source paths match the
pre-candidate tree byte-for-byte, update the verification record to state that
no production/workspace change remains, commit that documentation update, and
stop. Do not run release build/smoke for a rejected candidate.

---

### Task 8: Run Full Gates and Finalize a Retained Candidate

**Condition:** Execute only when `stage1-decision.json` says `retain`.

**Files:**
- Update: `docs/superpowers/verification/2026-07-17-notebooklm-render-crate-boundary.md`

- [ ] **Step 1: Run all automated gates**

Run sequentially:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

Expected: every command exits 0. Record command, exit, duration, and relevant
inventory counts. Do not claim a pass from older output.

- [ ] **Step 2: Build the release executable without bundling**

Run:

```powershell
npm.cmd run tauri -- build --no-bundle
```

Expected: exit 0 and `src-tauri/target/release/extractum.exe` exists. WiX/MSI
bundling is outside this slice and is not a performance or correctness gate.

- [ ] **Step 3: Perform startup and normal-shutdown smoke**

Launch the release executable from an interactive Windows session. Confirm the
main window opens and the process exits normally when the window is closed.
Record startup/shutdown evidence. Report navigation as unverified unless a
human or desktop automation actually navigates the app.

- [ ] **Step 4: Finalize the verification document**

Record all final gates, release smoke, known navigation limitation, retained
commit, exact changed-file scope, and final clean-worktree result. Explicitly
state that behavior and wire values were unchanged and that the compile-time
claim applies only to the measured full-workspace command on the recorded
machine.

- [ ] **Step 5: Commit final verification evidence**

Run `git diff --check`, inspect the complete diff, stage only the verification
document, and commit:

```powershell
git commit -m "docs: verify notebooklm render crate boundary"
```

Expected: clean worktree and a retained candidate whose preflight, correctness,
inventory, performance, release build, and smoke evidence are all recorded.
