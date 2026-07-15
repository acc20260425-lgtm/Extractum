# Lucide Direct Import Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate and, only if the approved retention gates pass, keep direct Lucide icon imports in `ProjectRailPanel.svelte` and `SourcesTab.svelte`.

**Architecture:** Build one reversible two-file patch, compare unchanged A and direct-import B with warm-ups and alternating complete Vitest runs, and use a comprehensive import-duration tree to prove whether the two target paths still traverse the Lucide barrel. Keep raw artifacts outside the repository; add a narrow `?raw` source contract and commit the candidate only after the A/B decision.

**Tech Stack:** Windows PowerShell, Git, Svelte 5, `@lucide/svelte` 1.17.0 direct exports, Vitest 4.1.5 JSON and import-duration reporters, Vite `?raw` imports.

## Global Constraints

- Implement `docs/superpowers/specs/2026-07-15-lucide-direct-import-validation-design.md` at or after approved commit `09f3d452`.
- Change application code only in `src/lib/components/research-projects/ProjectRailPanel.svelte` and `src/lib/components/research-projects/SourcesTab.svelte`.
- Do not change component markup, props, events, styling, icon identifiers, package files, Vite/Vitest configuration permanently, `extractum-ui`, or `docs/value-registry.md`.
- Do not migrate any other Lucide import or create a local icon facade.
- Use `npm.cmd` for npm scripts on Windows; direct `node.exe scripts/run-vitest.mjs` is allowed for reporter options.
- Use one absolute `%TEMP%` scratch directory outside the repository for raw JSON, logs, patches, and summaries.
- Run all benchmark commands sequentially with no active Vitest, Vite, Tauri-dev, or Extractum process.
- Record but do not change the active Windows power scheme or Defender real-time-protection state.
- Baseline A and candidate B performance runs must execute identical committed test inventories; create the contract only after the A/B decision.
- The sensitive metrics are the two target test-file medians and their import trees. Complete-suite wall time is only a non-regression gate.
- Use three recorded runs per side in A/B/A/B/A/B order after one discarded warm-up per side.
- Permit exactly one additional A/B/A/B/A/B sequence only when every other gate passes and the initial B median is more than 5% and no more than 8% slower than A.
- Store the candidate patch outside the repository and verify A/B SHA-256 hashes after every transition.
- A failed or empty warm-up invalidates the measurement session; investigate and restart from warm-ups rather than discarding the failure.
- Do not push after the final commit unless the user separately requests it.

---

### Task 1: Establish the Clean Measurement Workspace

**Files:**
- Read: `docs/superpowers/specs/2026-07-15-lucide-direct-import-validation-design.md`
- Read: `vite.config.js`
- Read: `src/lib/components/research-projects/ProjectRailPanel.svelte`
- Read: `src/lib/components/research-projects/SourcesTab.svelte`
- Produce outside repository: `%TEMP%/extractum-lucide-import-*/preflight.json`

**Interfaces:**
- Produces: `%TEMP%/extractum-lucide-import-current.txt`, the scratch-path pointer consumed by Tasks 2-7.
- Produces: `preflight.json` with the baseline commit, environment, and byte hashes.
- Preserves: a clean worktree.

- [ ] **Step 1: Verify the approved clean baseline**

Run:

```powershell
$status = @(git status --short --untracked-files=all 2>$null)
git merge-base --is-ancestor 09f3d452 HEAD
$approved = $LASTEXITCODE -eq 0
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_ANCESTOR=$approved"
git log -5 --oneline
if ($status.Count -ne 0 -or -not $approved) {
    $status
    exit 1
}
```

Expected: `STATUS_COUNT=0` and `APPROVED_SPEC_ANCESTOR=True`.

- [ ] **Step 2: Reject competing application and test processes**

Run:

```powershell
$nodeBlocking = @(
    Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Name -eq 'node.exe' -and
            $_.CommandLine -match '((?:^|[\\/\s"-])vitest(?:\.mjs)?(?:\s|$)|(?:^|[\\/\s"])(?:bin[\\/])?vite(?:\.js)?(?:\s|$)|scripts[\\/]tauri\.mjs|tauri(?:\.js)?\s+dev)'
        }
)
$appBlocking = @(Get-Process extractum -ErrorAction SilentlyContinue)
$nodeBlocking | Select-Object ProcessId, Name, CommandLine
$appBlocking | Select-Object Id, ProcessName, Path
if ($nodeBlocking.Count -ne 0 -or $appBlocking.Count -ne 0) { exit 1 }
```

Expected: both lists are empty. Do not stop processes automatically; close their owner and rerun.

- [ ] **Step 3: Create the external scratch directory**

Run:

```powershell
$scratch = Join-Path $env:TEMP ("extractum-lucide-import-" + [guid]::NewGuid().ToString('N'))
$pointer = Join-Path $env:TEMP 'extractum-lucide-import-current.txt'
New-Item -ItemType Directory -Path $scratch -ErrorAction Stop | Out-Null
Set-Content -LiteralPath $pointer -Value $scratch -Encoding UTF8
$resolvedScratch = (Resolve-Path -LiteralPath $scratch).Path
$repo = (Resolve-Path -LiteralPath '.').Path.TrimEnd('\')
"SCRATCH=$resolvedScratch"
if ($resolvedScratch.StartsWith($repo, [StringComparison]::OrdinalIgnoreCase)) { exit 1 }
```

Expected: one new absolute directory under `%TEMP%`, outside the repository.

- [ ] **Step 4: Capture environment, source hashes, and package-export preconditions**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$projectPath = 'src/lib/components/research-projects/ProjectRailPanel.svelte'
$sourcesPath = 'src/lib/components/research-projects/SourcesTab.svelte'
$projectSource = Get-Content -LiteralPath $projectPath -Raw
$sourcesSource = Get-Content -LiteralPath $sourcesPath -Raw
$icons = 'list','plus','refresh-cw','search','x','library','download','trash-2'
$missingIcons = @($icons | Where-Object { -not (Test-Path -LiteralPath "node_modules/@lucide/svelte/dist/icons/$_.js") })
$os = Get-CimInstance Win32_OperatingSystem
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$power = @(& powercfg.exe /getactivescheme 2>&1) -join "`n"
$defender = if (Get-Command Get-MpComputerStatus -ErrorAction SilentlyContinue) {
    try { (Get-MpComputerStatus).RealTimeProtectionEnabled } catch { 'unavailable' }
} else { 'unavailable' }
$help = @(& node.exe node_modules/vitest/vitest.mjs --help --expand-help 2>&1)
$helpCode = $LASTEXITCODE
$preflight = [ordered]@{
    commit = (git rev-parse HEAD).Trim()
    branch = (git branch --show-current).Trim()
    os = $os.Caption
    os_version = $os.Version
    logical_cores = $cpu.NumberOfLogicalProcessors
    memory_gib = [math]::Round($os.TotalVisibleMemorySize / 1MB, 2)
    power_scheme = $power.Trim()
    defender_realtime = $defender
    node = (& node.exe --version).Trim()
    npm = (& npm.cmd --version).Trim()
    vitest = (& node.exe node_modules/vitest/vitest.mjs --version).Trim()
    project_path = $projectPath
    sources_path = $sourcesPath
    project_a_sha256 = (Get-FileHash -Algorithm SHA256 $projectPath).Hash
    sources_a_sha256 = (Get-FileHash -Algorithm SHA256 $sourcesPath).Hash
    vite_config_sha256 = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash
    import_flag_present = (($help -join "`n") -match '--experimental\.importDurations\.print\b')
    import_limit_present = (($help -join "`n") -match '--experimental\.importDurations\.limit\b')
}
$help | Set-Content -LiteralPath (Join-Path $scratch 'vitest-expanded-help.txt') -Encoding UTF8
$preflight | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Encoding UTF8
$preflight | Format-List
"MISSING_DIRECT_ICON_MODULES=$($missingIcons.Count)"
if ($helpCode -ne 0 -or $missingIcons.Count -ne 0) { $missingIcons; exit 1 }
if ($projectSource -notmatch 'from\s+["'']@lucide/svelte["'']' -or $sourcesSource -notmatch 'from\s+["'']@lucide/svelte["'']') { exit 1 }
```

Expected: both components contain the baseline root import, all eight direct modules exist, and both import-duration option tokens are printed as available on Vitest 4.1.5. If a token is absent, keep the observation; Task 4 uses the reversible config fallback.

---

### Task 2: Build and Prove the Reversible Two-File Candidate

**Files:**
- Modify temporarily: `src/lib/components/research-projects/ProjectRailPanel.svelte:2`
- Modify temporarily: `src/lib/components/research-projects/SourcesTab.svelte:2`
- Produce outside repository: `candidate.patch`, `candidate.json`

**Interfaces:**
- Consumes: A hashes and baseline commit from `preflight.json`.
- Produces: a Git-applicable candidate patch and A/B hashes used by Tasks 3-6.
- Leaves: variant A and a clean worktree.

- [ ] **Step 1: Replace only the ProjectRailPanel Lucide import**

Use `apply_patch` to replace:

```svelte
  import { List, Plus, RefreshCw, Search, X } from "@lucide/svelte";
```

with:

```svelte
  import List from "@lucide/svelte/icons/list";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
```

Expected: no other line in `ProjectRailPanel.svelte` changes.

- [ ] **Step 2: Replace only the SourcesTab Lucide import**

Use `apply_patch` to replace:

```svelte
  import { Library, RefreshCw, Download, Trash2, X, Plus } from "@lucide/svelte";
```

with:

```svelte
  import Library from "@lucide/svelte/icons/library";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Download from "@lucide/svelte/icons/download";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import Plus from "@lucide/svelte/icons/plus";
```

Expected: no other line in `SourcesTab.svelte` changes.

- [ ] **Step 3: Capture B hashes and the exact candidate patch**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$projectPath = [string]$preflight.project_path
$sourcesPath = [string]$preflight.sources_path
$paths = @(git diff --name-only -- $projectPath $sourcesPath)
$allDirty = @(git status --short --untracked-files=all 2>$null)
$candidatePatch = Join-Path $scratch 'candidate.patch'
git diff --binary --output=$candidatePatch -- $projectPath $sourcesPath
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$candidate = [ordered]@{
    baseline_commit = [string]$preflight.commit
    project_path = $projectPath
    sources_path = $sourcesPath
    project_a_sha256 = [string]$preflight.project_a_sha256
    sources_a_sha256 = [string]$preflight.sources_a_sha256
    project_b_sha256 = (Get-FileHash -Algorithm SHA256 $projectPath).Hash
    sources_b_sha256 = (Get-FileHash -Algorithm SHA256 $sourcesPath).Hash
    patch = $candidatePatch
}
$candidate | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Encoding UTF8
"DIFF_PATHS=$($paths -join ',')"
"DIRTY_COUNT=$($allDirty.Count)"
"PATCH_BYTES=$((Get-Item -LiteralPath $candidatePatch).Length)"
if ($paths.Count -ne 2 -or $allDirty.Count -ne 2 -or -not (Test-Path -LiteralPath $candidatePatch) -or (Get-Item -LiteralPath $candidatePatch).Length -le 0) { exit 1 }
git diff --check -- $projectPath $sourcesPath
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: exactly the two owned components are modified and `candidate.patch` is nonempty.

- [ ] **Step 4: Restore A, apply B, and reverse to A with hash checks**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$paths = @([string]$candidate.project_path, [string]$candidate.sources_path)
git restore --source=$candidate.baseline_commit -- @paths
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$aOk = (Get-FileHash -Algorithm SHA256 $paths[0]).Hash -eq $candidate.project_a_sha256 -and
       (Get-FileHash -Algorithm SHA256 $paths[1]).Hash -eq $candidate.sources_a_sha256
git apply --check -- $candidate.patch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git apply -- $candidate.patch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$bOk = (Get-FileHash -Algorithm SHA256 $paths[0]).Hash -eq $candidate.project_b_sha256 -and
       (Get-FileHash -Algorithm SHA256 $paths[1]).Hash -eq $candidate.sources_b_sha256
git apply -R --check -- $candidate.patch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git apply -R -- $candidate.patch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$aRestored = (Get-FileHash -Algorithm SHA256 $paths[0]).Hash -eq $candidate.project_a_sha256 -and
             (Get-FileHash -Algorithm SHA256 $paths[1]).Hash -eq $candidate.sources_a_sha256
$status = @(git status --short --untracked-files=all 2>$null)
"A_INITIAL=$aOk"
"B_APPLIED=$bOk"
"A_RESTORED=$aRestored"
"STATUS_COUNT=$($status.Count)"
if (-not $aOk -or -not $bOk -or -not $aRestored -or $status.Count -ne 0) { $status; exit 1 }
```

Expected: all three booleans are `True`, the worktree returns to A, and it is clean.

---

### Task 3: Run Warm-Ups and the Initial Alternating A/B Sequence

**Files:**
- Toggle temporarily: the two owned Svelte files via `candidate.patch`
- Produce outside repository: `invoke-complete-variant.ps1`, `vitest/warmup-*.json`, `vitest/recorded-*.json`, logs, and metadata

**Interfaces:**
- Consumes: `candidate.json`.
- Produces: two successful discarded warm-ups and three recorded full-suite results per variant.
- Leaves: variant B with only the two owned component diffs.

- [ ] **Step 1: Create the external complete-run helper**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$runnerPath = Join-Path $scratch 'invoke-complete-variant.ps1'
$runnerSource = @'
param(
    [Parameter(Mandatory=$true)][string]$Label,
    [Parameter(Mandatory=$true)][ValidateSet('A','B')][string]$Variant,
    [switch]$Recorded
)
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
New-Item -ItemType Directory -Path $vitestDir -Force | Out-Null
$repo = (Resolve-Path -LiteralPath '.').Path.TrimEnd('\')
$paths = @([string]$candidate.project_path, [string]$candidate.sources_path)
function Get-Variant {
    $projectHash = (Get-FileHash -Algorithm SHA256 $paths[0]).Hash
    $sourcesHash = (Get-FileHash -Algorithm SHA256 $paths[1]).Hash
    if ($projectHash -eq $candidate.project_a_sha256 -and $sourcesHash -eq $candidate.sources_a_sha256) { return 'A' }
    if ($projectHash -eq $candidate.project_b_sha256 -and $sourcesHash -eq $candidate.sources_b_sha256) { return 'B' }
    throw 'Owned component hashes match neither A nor B.'
}
function Set-Variant([ValidateSet('A','B')][string]$target) {
    if ((Get-Variant) -eq $target) { return }
    if ($target -eq 'B') { & git.exe apply -- $candidate.patch } else { & git.exe apply -R -- $candidate.patch }
    if ($LASTEXITCODE -ne 0 -or (Get-Variant) -ne $target) { throw "Failed to switch to $target" }
}
Set-Variant $Variant
$report = [IO.Path]::GetFullPath((Join-Path $vitestDir "$Label.json"))
if ($report.StartsWith($repo, [StringComparison]::OrdinalIgnoreCase)) { throw 'Report path is inside repository.' }
$watch = [Diagnostics.Stopwatch]::StartNew()
$output = @(& node.exe scripts/run-vitest.mjs run --reporter=json --reporter=default "--outputFile.json=$report" 2>&1)
$code = $LASTEXITCODE
$watch.Stop()
$output | Set-Content -LiteralPath (Join-Path $vitestDir "$Label.log") -Encoding UTF8
$json = if (Test-Path -LiteralPath $report) { Get-Content -LiteralPath $report -Raw | ConvertFrom-Json } else { $null }
$meta = [ordered]@{
    label = $Label
    variant = $Variant
    recorded = [bool]$Recorded
    exit = $code
    wall_seconds = [math]::Round($watch.Elapsed.TotalSeconds, 3)
    success = if ($json) { [bool]$json.success } else { $false }
    files = if ($json) { @($json.testResults).Count } else { 0 }
    suites = if ($json) { [int]$json.numTotalTestSuites } else { 0 }
    tests = if ($json) { [int]$json.numTotalTests } else { 0 }
    report = $report
}
$meta | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $vitestDir "$Label-meta.json") -Encoding UTF8
$meta | Format-List
if ($code -ne 0 -or -not $json -or -not $json.success -or $meta.files -le 0 -or $meta.tests -le 0) { exit 1 }
'@
Set-Content -LiteralPath $runnerPath -Value $runnerSource -Encoding UTF8
"RUNNER=$runnerPath"
if (-not (Test-Path -LiteralPath $runnerPath) -or (Get-Item -LiteralPath $runnerPath).Length -le 0) { exit 1 }
```

Expected: a nonempty `invoke-complete-variant.ps1` exists only in the external scratch directory.

- [ ] **Step 2: Execute the warm-ups and recorded A/B sequence**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-complete-variant.ps1'
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Label 'warmup-A' -Variant 'A'
if ($LASTEXITCODE -ne 0) { throw 'Warm-up A failed; investigate and restart from both warm-ups.' }
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Label 'warmup-B' -Variant 'B'
if ($LASTEXITCODE -ne 0) { throw 'Warm-up B failed; investigate and restart from both warm-ups.' }
$sequence = @('A','B','A','B','A','B')
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$paths = @([string]$candidate.project_path, [string]$candidate.sources_path)
$earlyRejected = $false
for ($index = 0; $index -lt $sequence.Count; $index++) {
    $number = $index + 1
    $label = "recorded-{0:D2}-{1}" -f $number, $sequence[$index]
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Label $label -Variant $sequence[$index] -Recorded
    if ($LASTEXITCODE -ne 0) {
        git restore --source=$candidate.baseline_commit -- @paths
        if ($LASTEXITCODE -ne 0) { throw "Recorded run $label failed and A restoration also failed." }
        $aRestored = (Get-FileHash -Algorithm SHA256 $paths[0]).Hash -eq $candidate.project_a_sha256 -and
                     (Get-FileHash -Algorithm SHA256 $paths[1]).Hash -eq $candidate.sources_a_sha256
        if (-not $aRestored) { throw "Recorded run $label failed and A hashes were not restored." }
        if ($sequence[$index] -eq 'A') {
            [ordered]@{reason='baseline_recorded_failure';failed_label=$label;scratch=$scratch} |
                ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'session-invalid.json') -Encoding UTF8
            throw "Baseline A recorded run failed: $label. Preserve this scratch directory, investigate, create a fresh Task 1 scratch directory, and restart from both warm-ups. Do not combine sessions."
        }
        $decision = [ordered]@{
            decision = 'rejected'
            reason = 'candidate_recorded_failure'
            failed_label = $label
            retry_used = $false
            repeat_failed = $false
            runs_per_variant = 0
            final_a_wall_median_seconds = $null
            final_b_wall_median_seconds = $null
            final_wall_delta_percent = $null
            timing_gate = $false
            inventory_gate = $false
            source_gate = $false
            import_gate = $false
            correctness_gate = $false
        }
        $decision | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'decision.json') -Encoding UTF8
        Set-Content -LiteralPath (Join-Path $scratch 'early-rejection.txt') -Value $label -Encoding UTF8
        $earlyRejected = $true
        break
    }
}
if ($earlyRejected) {
    $status = @(git status --short --untracked-files=all 2>$null)
    "EARLY_REJECTION=candidate_recorded_failure"
    if ($status.Count -ne 0) { $status; exit 1 }
} else {
    $dirty = @(git diff --name-only)
    $unexpected = @($dirty | Where-Object { $_ -notin $paths })
    if ($dirty.Count -ne 2 -or $unexpected.Count -ne 0) { $dirty; exit 1 }
    if ((Get-FileHash -Algorithm SHA256 $paths[0]).Hash -ne $candidate.project_b_sha256 -or
        (Get-FileHash -Algorithm SHA256 $paths[1]).Hash -ne $candidate.sources_b_sha256) { exit 1 }
}
```

Expected on the normal path: both warm-ups and all six recorded runs pass nonempty inventories, each log includes readable default-reporter diagnostics, final variant is B, and only the two owned files are dirty. An A recorded failure invalidates the entire session and requires a fresh scratch plus restart from warm-ups. A B recorded failure conservatively rejects the candidate, restores exact A, records `decision.json`, skips Task 3 Step 3 and Tasks 4-6, and continues with the early-rejection evidence path in Task 7.

- [ ] **Step 3: Verify recorded inventory equality before import profiling**

Run only when `early-rejection.txt` does not exist:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
$metas = @(Get-ChildItem -LiteralPath $vitestDir -Filter 'recorded-*-meta.json' | Sort-Object Name | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json })
$inventories = @($metas | ForEach-Object { "$($_.files)/$($_.suites)/$($_.tests)" } | Sort-Object -Unique)
$aCount = @($metas | Where-Object variant -eq 'A').Count
$bCount = @($metas | Where-Object variant -eq 'B').Count
"RECORDED_RUNS=$($metas.Count)"
"A_RUNS=$aCount"
"B_RUNS=$bCount"
"INVENTORIES=$($inventories -join ',')"
if ($metas.Count -ne 6 -or $aCount -ne 3 -or $bCount -ne 3 -or $inventories.Count -ne 1) { exit 1 }
```

Expected: six recorded runs, three per side, and exactly one shared inventory.

---

### Task 4: Capture Comprehensive A and B Import Trees

If Task 3 created `early-rejection.txt`, skip this entire task and continue at Task 7.

**Files:**
- Toggle temporarily: the two owned Svelte files
- Conditionally modify and restore: `vite.config.js`
- Produce outside repository: `invoke-import-variant.ps1`, `vitest/import-*.json`, logs, subtrees, and `import-mechanism.txt`

**Interfaces:**
- Consumes: candidate hashes, Vitest help, and the original Vite config hash.
- Produces: canonical `import-A.log` and `import-B.log` containing both target roots with up to 2,000 collected imports.
- Leaves: variant B and byte-identical `vite.config.js`.

- [ ] **Step 1: Try the comprehensive CLI mechanism on A and B**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
$runnerPath = Join-Path $scratch 'invoke-import-variant.ps1'
$runnerSource = @'
param(
    [Parameter(Mandatory=$true)][ValidateSet('A','B')][string]$Variant,
    [Parameter(Mandatory=$true)][string]$Prefix,
    [switch]$Cli
)
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
$paths = @([string]$candidate.project_path, [string]$candidate.sources_path)
function Get-Variant {
    $p = (Get-FileHash -Algorithm SHA256 $paths[0]).Hash
    $s = (Get-FileHash -Algorithm SHA256 $paths[1]).Hash
    if ($p -eq $candidate.project_a_sha256 -and $s -eq $candidate.sources_a_sha256) { return 'A' }
    if ($p -eq $candidate.project_b_sha256 -and $s -eq $candidate.sources_b_sha256) { return 'B' }
    throw 'Unknown component variant.'
}
function Set-Variant([ValidateSet('A','B')][string]$target) {
    if ((Get-Variant) -eq $target) { return }
    if ($target -eq 'B') { & git.exe apply -- $candidate.patch } else { & git.exe apply -R -- $candidate.patch }
    if ($LASTEXITCODE -ne 0 -or (Get-Variant) -ne $target) { throw "Failed to switch to $target" }
}
Set-Variant $Variant
$report = [IO.Path]::GetFullPath((Join-Path $vitestDir "$Prefix-$Variant.json"))
$vitestArgs = @('scripts/run-vitest.mjs','run','--reporter=json','--reporter=default',"--outputFile.json=$report")
if ($Cli) { $vitestArgs += @('--experimental.importDurations.print','--experimental.importDurations.limit=2000') }
$watch = [Diagnostics.Stopwatch]::StartNew()
$output = @(& node.exe @vitestArgs 2>&1)
$code = $LASTEXITCODE
$watch.Stop()
$log = Join-Path $vitestDir "$Prefix-$Variant.log"
$output | Set-Content -LiteralPath $log -Encoding UTF8
$json = if (Test-Path -LiteralPath $report) { Get-Content -LiteralPath $report -Raw | ConvertFrom-Json } else { $null }
$meta = [ordered]@{
    variant = $Variant
    mechanism = $Prefix
    exit = $code
    wall_seconds = [math]::Round($watch.Elapsed.TotalSeconds,3)
    success = if ($json) { [bool]$json.success } else { $false }
    files = if ($json) { @($json.testResults).Count } else { 0 }
    tests = if ($json) { [int]$json.numTotalTests } else { 0 }
}
$meta | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $vitestDir "$Prefix-$Variant-meta.json") -Encoding UTF8
$meta | Format-List
if ($code -ne 0 -or -not $json -or -not $json.success -or $meta.files -le 0 -or $meta.tests -le 0) { exit 1 }
'@
Set-Content -LiteralPath $runnerPath -Value $runnerSource -Encoding UTF8
$cliAvailable = [bool]$preflight.import_flag_present -and [bool]$preflight.import_limit_present
$cliRunsOk = $false
if ($cliAvailable) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runnerPath -Variant 'A' -Prefix 'import-cli' -Cli
    $aCode = $LASTEXITCODE
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runnerPath -Variant 'B' -Prefix 'import-cli' -Cli
    $bCode = $LASTEXITCODE
    $cliRunsOk = $aCode -eq 0 -and $bCode -eq 0
}
$cliComplete = $false
if ($cliAvailable -and $cliRunsOk) {
    $a = Get-Content -LiteralPath (Join-Path $vitestDir 'import-cli-A.log') -Raw
    $b = Get-Content -LiteralPath (Join-Path $vitestDir 'import-cli-B.log') -Raw
    $cliComplete = $a -match 'Import Duration Breakdown' -and $b -match 'Import Duration Breakdown' -and
                   $a -match 'ProjectRailPanel\.test\.ts' -and $a -match 'SourcesTab\.test\.ts' -and
                   $b -match 'ProjectRailPanel\.test\.ts' -and $b -match 'SourcesTab\.test\.ts'
}
"CLI_IMPORT_COMPLETE=$cliComplete"
if ($cliComplete) {
    Copy-Item -LiteralPath (Join-Path $vitestDir 'import-cli-A.log') -Destination (Join-Path $vitestDir 'import-A.log')
    Copy-Item -LiteralPath (Join-Path $vitestDir 'import-cli-B.log') -Destination (Join-Path $vitestDir 'import-B.log')
    Set-Content -LiteralPath (Join-Path $vitestDir 'import-mechanism.txt') -Value 'CLI print with limit=2000' -Encoding UTF8
} else {
    Set-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-required.txt') -Value 'required' -Encoding UTF8
}
```

Expected: the external import runner is fully defined. CLI rejection or incomplete output creates the fallback marker rather than aborting; CLI logs are canonical only when both runs pass and contain the breakdown plus both target roots.

- [ ] **Step 2: If required, add the reversible comprehensive config fallback**

If `vitest/import-fallback-required.txt` does not exist, skip to Step 4.

Otherwise use `apply_patch` to replace:

```js
export const VITEST_TEST_CONFIG = {
  pool: "threads",
};
```

with:

```js
export const VITEST_TEST_CONFIG = {
  pool: "threads",
  experimental: {
    importDurations: {
      print: true,
      limit: 2000,
    },
  },
};
```

Expected: only this temporary Vitest config block changes in `vite.config.js`.

- [ ] **Step 3: Run fallback A/B and restore Vite config before interpreting output**

Run only when `vitest/import-fallback-required.txt` exists:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-import-variant.ps1'
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Variant 'A' -Prefix 'import-fallback'
$aCode = $LASTEXITCODE
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Variant 'B' -Prefix 'import-fallback'
$bCode = $LASTEXITCODE
[ordered]@{a_exit=$aCode;b_exit=$bCode} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'vitest/import-fallback-codes.json') -Encoding UTF8
```

Immediately use `apply_patch` to restore the original block:

```js
export const VITEST_TEST_CONFIG = {
  pool: "threads",
};
```

Then run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
$codes = Get-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-codes.json') -Raw | ConvertFrom-Json
$viteHash = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash
$a = Get-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-A.log') -Raw
$b = Get-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-B.log') -Raw
$complete = $a -match 'Import Duration Breakdown' -and $b -match 'Import Duration Breakdown' -and
            $a -match 'ProjectRailPanel\.test\.ts' -and $a -match 'SourcesTab\.test\.ts' -and
            $b -match 'ProjectRailPanel\.test\.ts' -and $b -match 'SourcesTab\.test\.ts'
"VITE_HASH_RESTORED=$($viteHash -eq $preflight.vite_config_sha256)"
"FALLBACK_IMPORT_COMPLETE=$complete"
if ($viteHash -ne $preflight.vite_config_sha256 -or $codes.a_exit -ne 0 -or $codes.b_exit -ne 0 -or -not $complete) { exit 1 }
Copy-Item -LiteralPath (Join-Path $vitestDir 'import-fallback-A.log') -Destination (Join-Path $vitestDir 'import-A.log')
Copy-Item -LiteralPath (Join-Path $vitestDir 'import-fallback-B.log') -Destination (Join-Path $vitestDir 'import-B.log')
Set-Content -LiteralPath (Join-Path $vitestDir 'import-mechanism.txt') -Value 'config fallback with limit=2000' -Encoding UTF8
```

Expected: both complete fallback logs exist, the Vite hash is restored before the result is used, and variant B remains active.

- [ ] **Step 4: Extract target-root subtrees and enforce qualitative attribution**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
function Get-ImportSubtree([string[]]$lines, [string]$rootName) {
    $breakdownStart = -1
    for ($i=0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match 'Import Duration Breakdown') { $breakdownStart = $i; break }
    }
    if ($breakdownStart -lt 0) { throw 'Import Duration Breakdown section not found.' }
    $start = -1
    for ($i=$breakdownStart + 1; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match [regex]::Escape($rootName)) { $start = $i; break }
    }
    if ($start -lt 0) { throw "Import root not found: $rootName" }
    $result = [System.Collections.Generic.List[string]]::new()
    $result.Add($lines[$start])
    $branchPattern = '^\s*' + [regex]::Escape([string][char]0x21B3)
    for ($i=$start + 1; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -notmatch $branchPattern) { break }
        $result.Add($lines[$i])
    }
    if ($result.Count -le 1) { throw "Import subtree for $rootName has no branch rows; reporter format is not attributable." }
    return [string[]]$result.ToArray()
}
$roots = 'ProjectRailPanel.test.ts','SourcesTab.test.ts'
$summary = @()
foreach ($variant in 'A','B') {
    $lines = @(Get-Content -LiteralPath (Join-Path $vitestDir "import-$variant.log"))
    foreach ($root in $roots) {
        $subtree = @(Get-ImportSubtree $lines $root)
        $safeRoot = $root -replace '\.test\.ts$',''
        $subtree | Set-Content -LiteralPath (Join-Path $vitestDir "import-$variant-$safeRoot-subtree.txt") -Encoding UTF8
        $summary += [pscustomobject]@{
            variant = $variant
            root = $root
            contains_icons_index = ((($subtree -join "`n").Replace('\','/')) -match 'icons/index\.js')
        }
    }
}
$summary | Export-Csv -LiteralPath (Join-Path $vitestDir 'import-attribution.csv') -NoTypeInformation -Encoding UTF8
$summary | Format-Table
$aRows = @($summary | Where-Object variant -eq 'A')
$bRows = @($summary | Where-Object variant -eq 'B')
if (@($aRows | Where-Object { -not $_.contains_icons_index }).Count -ne 0) { exit 1 }
if (@($bRows | Where-Object contains_icons_index).Count -ne 0) { exit 1 }
```

Expected: both A target subtrees contain `icons/index.js`; neither B target subtree does. Global presence of `icons/index.js` outside these two subtrees is expected and ignored.

---

### Task 5: Compute the Retention Decision and Optional Single Retry

If Task 3 already created `decision.json` with reason `candidate_recorded_failure`, skip this entire task and Task 6, then continue at Task 7.

**Files:**
- Toggle temporarily: the two owned Svelte files
- Produce outside repository: `vitest/ab-initial-summary.json`, optional repeat runs, and `decision.json`

**Interfaces:**
- Consumes: initial six recorded reports, import attribution, candidate source, and approved thresholds.
- Produces: one final `retained` or `rejected` decision with no discretionary reruns.
- Leaves: B when retained; byte-identical A and clean worktree when rejected.

- [ ] **Step 1: Compute initial medians, target-file medians, and non-timing gates**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
$projectHash = (Get-FileHash -Algorithm SHA256 $candidate.project_path).Hash
$sourcesHash = (Get-FileHash -Algorithm SHA256 $candidate.sources_path).Hash
if ($projectHash -ne $candidate.project_b_sha256 -or $sourcesHash -ne $candidate.sources_b_sha256) { exit 1 }
$focusedOutput = @(& node.exe scripts/run-vitest.mjs run src/lib/components/research-projects/ProjectRailPanel.test.ts src/lib/components/research-projects/SourcesTab.test.ts 2>&1)
$focusedCode = $LASTEXITCODE
$focusedOutput | Set-Content -LiteralPath (Join-Path $vitestDir 'predecision-focused.log') -Encoding UTF8
$checkOutput = @()
$checkCode = 1
if ($focusedCode -eq 0) {
    $checkOutput = @(& npm.cmd run check 2>&1)
    $checkCode = $LASTEXITCODE
    $checkOutput | Set-Content -LiteralPath (Join-Path $vitestDir 'predecision-check.log') -Encoding UTF8
}
$correctnessGate = $focusedCode -eq 0 -and $checkCode -eq 0
function Get-Median([double[]]$values) {
    $sorted = @($values | Sort-Object)
    if ($sorted.Count % 2 -eq 1) { return [double]$sorted[[int][math]::Floor($sorted.Count / 2)] }
    return ([double]$sorted[$sorted.Count / 2 - 1] + [double]$sorted[$sorted.Count / 2]) / 2
}
function Get-FileDuration([object]$report, [string]$suffix) {
    $found = @($report.testResults | Where-Object { ([string]$_.name).Replace('\','/').EndsWith($suffix, [StringComparison]::OrdinalIgnoreCase) })
    if ($found.Count -ne 1) { throw "Expected one test file ending in $suffix, got $($found.Count)." }
    return [double]$found[0].endTime - [double]$found[0].startTime
}
$rows = @()
foreach ($metaFile in Get-ChildItem -LiteralPath $vitestDir -Filter 'recorded-*-meta.json' | Sort-Object Name) {
    $meta = Get-Content -LiteralPath $metaFile.FullName -Raw | ConvertFrom-Json
    $report = Get-Content -LiteralPath $meta.report -Raw | ConvertFrom-Json
    $rows += [pscustomobject]@{
        label = $meta.label
        variant = $meta.variant
        wall_seconds = [double]$meta.wall_seconds
        files = [int]$meta.files
        tests = [int]$meta.tests
        project_ms = Get-FileDuration $report 'src/lib/components/research-projects/ProjectRailPanel.test.ts'
        sources_ms = Get-FileDuration $report 'src/lib/components/research-projects/SourcesTab.test.ts'
    }
}
$inventories = @($rows | ForEach-Object { "$($_.files)/$($_.tests)" } | Sort-Object -Unique)
$a = @($rows | Where-Object variant -eq 'A')
$b = @($rows | Where-Object variant -eq 'B')
$aMedian = Get-Median ([double[]]$a.wall_seconds)
$bMedian = Get-Median ([double[]]$b.wall_seconds)
$delta = (($bMedian - $aMedian) / $aMedian) * 100
$projectSource = Get-Content -LiteralPath $candidate.project_path -Raw
$sourcesSource = Get-Content -LiteralPath $candidate.sources_path -Raw
$sourceGate = $projectSource -notmatch 'from\s+["'']@lucide/svelte["'']' -and
              $sourcesSource -notmatch 'from\s+["'']@lucide/svelte["'']'
$attribution = @(Import-Csv -LiteralPath (Join-Path $vitestDir 'import-attribution.csv'))
$importGate = @($attribution | Where-Object { $_.variant -eq 'B' -and $_.contains_icons_index -eq 'True' }).Count -eq 0
$summary = [ordered]@{
    initial_runs_per_variant = 3
    inventory = if ($inventories.Count -eq 1) { $inventories[0] } else { $inventories -join ',' }
    inventory_equal = $inventories.Count -eq 1
    a_wall_median_seconds = [math]::Round($aMedian,3)
    b_wall_median_seconds = [math]::Round($bMedian,3)
    initial_wall_delta_percent = [math]::Round($delta,3)
    a_project_median_ms = [math]::Round((Get-Median ([double[]]$a.project_ms)),3)
    b_project_median_ms = [math]::Round((Get-Median ([double[]]$b.project_ms)),3)
    a_sources_median_ms = [math]::Round((Get-Median ([double[]]$a.sources_ms)),3)
    b_sources_median_ms = [math]::Round((Get-Median ([double[]]$b.sources_ms)),3)
    source_gate = $sourceGate
    import_gate = $importGate
    correctness_gate = $correctnessGate
    focused_exit = $focusedCode
    check_exit = $checkCode
    retry_required = $delta -gt 5 -and $delta -le 8 -and $inventories.Count -eq 1 -and $sourceGate -and $importGate -and $correctnessGate
}
$rows | Export-Csv -LiteralPath (Join-Path $vitestDir 'ab-runs.csv') -NoTypeInformation -Encoding UTF8
$summary | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $vitestDir 'ab-initial-summary.json') -Encoding UTF8
$summary | Format-List
if ($a.Count -ne 3 -or $b.Count -ne 3) { exit 1 }
```

Expected: three rows per variant and literal initial medians. Focused tests and `npm.cmd run check` are recorded before any marginal retry. `retry_required` is true only in the predeclared `(5%; 8%]` window when inventory, source, import, and correctness gates all pass; any failed gate proceeds to deterministic rejection without a retry.

- [ ] **Step 2: Run exactly one additional sequence only when predeclared**

If `ab-initial-summary.json` has `retry_required=false`, do not run this step.

If true, run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-complete-variant.ps1'
$vitestDir = Join-Path $scratch 'vitest'
$sequence = @('A','B','A','B','A','B')
$repeatFailed = $false
for ($index = 0; $index -lt $sequence.Count; $index++) {
    $number = $index + 1
    $label = "repeat-{0:D2}-{1}" -f $number, $sequence[$index]
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Label $label -Variant $sequence[$index] -Recorded
    if ($LASTEXITCODE -ne 0) {
        Set-Content -LiteralPath (Join-Path $vitestDir 'repeat-failed.txt') -Value $label -Encoding UTF8
        $repeatFailed = $true
        break
    }
}
"REPEAT_FAILED=$repeatFailed"
```

Expected: normally, exactly six additional successful nonempty runs, no extra warm-up, and final variant B. A failure writes `repeat-failed.txt`, stops the sequence, and forces rejection in Step 3; do not retry again.

- [ ] **Step 3: Make the deterministic final decision and restore the correct state**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
$initial = Get-Content -LiteralPath (Join-Path $vitestDir 'ab-initial-summary.json') -Raw | ConvertFrom-Json
$repeatFailed = Test-Path -LiteralPath (Join-Path $vitestDir 'repeat-failed.txt')
function Get-Median([double[]]$values) {
    $sorted = @($values | Sort-Object)
    if ($sorted.Count % 2 -eq 1) { return [double]$sorted[[int][math]::Floor($sorted.Count / 2)] }
    return ([double]$sorted[$sorted.Count / 2 - 1] + [double]$sorted[$sorted.Count / 2]) / 2
}
$metaFiles = @(Get-ChildItem -LiteralPath $vitestDir -Filter 'recorded-*-meta.json')
if ($initial.retry_required -and -not $repeatFailed) { $metaFiles += @(Get-ChildItem -LiteralPath $vitestDir -Filter 'repeat-*-meta.json') }
$metas = @($metaFiles | Sort-Object Name | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json })
$a = @($metas | Where-Object variant -eq 'A')
$b = @($metas | Where-Object variant -eq 'B')
$inventories = @($metas | ForEach-Object { "$($_.files)/$($_.tests)" } | Sort-Object -Unique)
$inventoryGate = $inventories.Count -eq 1
$allRunsPassed = @($metas | Where-Object { $_.exit -ne 0 -or -not $_.success -or $_.files -le 0 -or $_.tests -le 0 }).Count -eq 0
$aMedian = Get-Median ([double[]]$a.wall_seconds)
$bMedian = Get-Median ([double[]]$b.wall_seconds)
$delta = (($bMedian - $aMedian) / $aMedian) * 100
$expectedPerSide = if ($initial.retry_required -and -not $repeatFailed) { 6 } else { 3 }
$timingGate = -not $repeatFailed -and $delta -le 5
$retained = $a.Count -eq $expectedPerSide -and $b.Count -eq $expectedPerSide -and
            $inventoryGate -and $allRunsPassed -and [bool]$initial.source_gate -and
            [bool]$initial.import_gate -and [bool]$initial.correctness_gate -and $timingGate
$decision = [ordered]@{
    decision = if ($retained) { 'retained' } else { 'rejected' }
    retry_used = [bool]$initial.retry_required
    repeat_failed = $repeatFailed
    runs_per_variant = $expectedPerSide
    final_a_wall_median_seconds = [math]::Round($aMedian,3)
    final_b_wall_median_seconds = [math]::Round($bMedian,3)
    final_wall_delta_percent = [math]::Round($delta,3)
    combined_inventory = if ($inventoryGate) { $inventories[0] } else { $inventories -join ',' }
    all_performance_runs_passed = $allRunsPassed
    timing_gate = $timingGate
    inventory_gate = $inventoryGate
    source_gate = [bool]$initial.source_gate
    import_gate = [bool]$initial.import_gate
    correctness_gate = [bool]$initial.correctness_gate
}
$decision | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'decision.json') -Encoding UTF8
$decision | Format-List
$paths = @([string]$candidate.project_path, [string]$candidate.sources_path)
if ($retained) {
    $p = (Get-FileHash -Algorithm SHA256 $paths[0]).Hash
    $s = (Get-FileHash -Algorithm SHA256 $paths[1]).Hash
    if ($p -ne $candidate.project_b_sha256 -or $s -ne $candidate.sources_b_sha256) { exit 1 }
} else {
    git restore --source=$candidate.baseline_commit -- @paths
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $p = (Get-FileHash -Algorithm SHA256 $paths[0]).Hash
    $s = (Get-FileHash -Algorithm SHA256 $paths[1]).Hash
    if ($p -ne $candidate.project_a_sha256 -or $s -ne $candidate.sources_a_sha256) { exit 1 }
}
```

Expected: one `decision.json`. The final `inventory_gate` is recomputed across all three or all six runs per side, including every successful repeat run. Retained leaves exact B hashes; rejected restores exact A hashes. No further timing retry is permitted.

---

### Task 6: Protect and Commit a Retained Candidate with TDD

**Files:**
- Create only when retained: `src/lib/lucide-direct-import-contract.test.ts`
- Modify when retained: `src/lib/components/research-projects/ProjectRailPanel.svelte`
- Modify when retained: `src/lib/components/research-projects/SourcesTab.svelte`

**Interfaces:**
- Consumes: `decision.json` and `candidate.patch`.
- Produces when retained: a narrow `?raw` contract and one code commit.
- Skips completely when rejected.

- [ ] **Step 1: Branch on the recorded decision**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$decision = Get-Content -LiteralPath (Join-Path $scratch 'decision.json') -Raw | ConvertFrom-Json
"DECISION=$($decision.decision)"
if ($decision.decision -eq 'rejected') {
    Write-Host 'Skip Task 6 and continue with rejected evidence in Task 7.'
}
```

Expected: continue Steps 2-7 only for `retained`.

- [ ] **Step 2: Restore A temporarily for the RED contract step**

Run only when retained:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$paths = @([string]$candidate.project_path, [string]$candidate.sources_path)
git apply -R --check -- $candidate.patch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git apply -R -- $candidate.patch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
if ((Get-FileHash -Algorithm SHA256 $paths[0]).Hash -ne $candidate.project_a_sha256 -or
    (Get-FileHash -Algorithm SHA256 $paths[1]).Hash -ne $candidate.sources_a_sha256) { exit 1 }
```

Expected: both components are exact A while the A/B decision remains recorded externally.

- [ ] **Step 3: Write the focused contract**

Create `src/lib/lucide-direct-import-contract.test.ts` with `apply_patch`:

```ts
import { describe, expect, it } from "vitest";
import projectRailPanelSource from "./components/research-projects/ProjectRailPanel.svelte?raw";
import sourcesTabSource from "./components/research-projects/SourcesTab.svelte?raw";

const ROOT_LUCIDE_IMPORT = /from\s+["']@lucide\/svelte["']/;
const LUCIDE_PACKAGE_IMPORT = /from\s+["'](@lucide\/svelte[^"']*)["']/g;

function normalized(source: string): string {
  return source.replace(/\r\n/g, "\n");
}

function lucidePackageImports(source: string): string[] {
  return [...normalized(source).matchAll(LUCIDE_PACKAGE_IMPORT)].map((match) => {
    const specifier = match[1];
    if (!specifier) {
      throw new Error("Lucide import regex matched without a module specifier");
    }
    return specifier;
  });
}

describe("research-project Lucide import boundaries", () => {
  it.each([
    ["ProjectRailPanel", projectRailPanelSource],
    ["SourcesTab", sourcesTabSource],
  ])("keeps %s off the Lucide root barrel", (_name, rawSource) => {
    const source = normalized(rawSource);
    const imports = lucidePackageImports(source);

    expect(source).not.toMatch(ROOT_LUCIDE_IMPORT);
    expect(imports.every((specifier) => specifier.startsWith("@lucide/svelte/icons/"))).toBe(true);
  });
});
```

Expected: the contract protects only the two target files, imports both with `?raw`, and does not enumerate icon names.

- [ ] **Step 4: Run the contract to verify RED on A**

Run:

```powershell
node.exe scripts/run-vitest.mjs run src/lib/lucide-direct-import-contract.test.ts
$code = $LASTEXITCODE
"RED_EXIT=$code"
if ($code -eq 0) { exit 1 }
```

Expected: FAIL because both A sources still import from `@lucide/svelte`. A zero exit is a false RED and fails the step.

- [ ] **Step 5: Reapply B and verify GREEN**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
git apply --check -- $candidate.patch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git apply -- $candidate.patch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
node.exe scripts/run-vitest.mjs run src/lib/lucide-direct-import-contract.test.ts
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: exact B is restored and the two contract cases pass.

- [ ] **Step 6: Run focused behavior and static checks**

Run:

```powershell
node.exe scripts/run-vitest.mjs run src/lib/lucide-direct-import-contract.test.ts src/lib/components/research-projects/ProjectRailPanel.test.ts src/lib/components/research-projects/SourcesTab.test.ts
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
npm.cmd run check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: the contract and both existing component suites pass; Svelte/type checking passes.

- [ ] **Step 7: Review and commit only the retained code surface**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$paths = @()
$paths += git diff --name-only
$paths += git ls-files --others --exclude-standard 2>$null
$paths = @($paths | Where-Object { $_ } | Sort-Object -Unique)
$expected = @(
    'src/lib/components/research-projects/ProjectRailPanel.svelte',
    'src/lib/components/research-projects/SourcesTab.svelte',
    'src/lib/lucide-direct-import-contract.test.ts'
) | Sort-Object
$paths
if (($paths -join "`n") -ne ($expected -join "`n")) { exit 1 }
git diff -- src/lib/components/research-projects/ProjectRailPanel.svelte src/lib/components/research-projects/SourcesTab.svelte
Get-Content -LiteralPath 'src/lib/lucide-direct-import-contract.test.ts'
git add -- src/lib/components/research-projects/ProjectRailPanel.svelte src/lib/components/research-projects/SourcesTab.svelte src/lib/lucide-direct-import-contract.test.ts
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git commit -m "perf: use direct lucide icon imports"
```

Expected: one commit containing exactly the two import-block changes and the focused contract.

---

### Task 7: Run the Final Gate and Commit Verification Evidence

**Files:**
- Create: `docs/superpowers/verification/2026-07-15-lucide-direct-import-validation.md`

**Interfaces:**
- Consumes: all scratch metadata and the final retained/rejected decision.
- Produces: one self-contained evidence document and one documentation commit.
- Preserves: clean final worktree; no raw scratch artifacts staged.

- [ ] **Step 1: Run the final repository gate only for retained code**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$decision = Get-Content -LiteralPath (Join-Path $scratch 'decision.json') -Raw | ConvertFrom-Json
if ($decision.decision -eq 'retained') {
    npm.cmd run verify
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} else {
    Write-Host 'Candidate rejected and byte-identical A restored; full verify is not repeated for a documentation-only result.'
}
```

Expected: retained code passes the complete repository gate. A rejected candidate skips this redundant gate after successful A/B suites and exact restoration.

- [ ] **Step 2: Verify restoration/scope and artifact completeness**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-lucide-import-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$decision = Get-Content -LiteralPath (Join-Path $scratch 'decision.json') -Raw | ConvertFrom-Json
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$required = @(
    'preflight.json',
    'candidate.json',
    'candidate.patch',
    'decision.json'
)
$earlyRejection = $decision.reason -eq 'candidate_recorded_failure'
if ($earlyRejection) {
    $required += @(
        'early-rejection.txt',
        ("vitest/{0}-meta.json" -f $decision.failed_label),
        ("vitest/{0}.log" -f $decision.failed_label)
    )
} else {
    $required += @(
        'vitest/ab-runs.csv',
        'vitest/ab-initial-summary.json',
        'vitest/import-mechanism.txt',
        'vitest/import-attribution.csv',
        'vitest/import-A-ProjectRailPanel-subtree.txt',
        'vitest/import-A-SourcesTab-subtree.txt',
        'vitest/import-B-ProjectRailPanel-subtree.txt',
        'vitest/import-B-SourcesTab-subtree.txt'
    )
}
$missing = @($required | Where-Object { -not (Test-Path -LiteralPath (Join-Path $scratch $_)) })
$warmupMetas = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'vitest') -Filter 'warmup-*-meta.json' -ErrorAction SilentlyContinue)
$recordedMetas = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'vitest') -Filter 'recorded-*-meta.json' -ErrorAction SilentlyContinue)
$initialRunArtifactsValid = if ($earlyRejection) {
    $warmupMetas.Count -eq 2 -and $recordedMetas.Count -ge 1 -and $recordedMetas.Count -le 6
} else {
    $warmupMetas.Count -eq 2 -and $recordedMetas.Count -eq 6
}
$repeatMetas = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'vitest') -Filter 'repeat-*-meta.json' -ErrorAction SilentlyContinue)
$repeatArtifactsValid = if (-not $decision.retry_used) {
    $repeatMetas.Count -eq 0
} elseif ($decision.repeat_failed) {
    (Test-Path -LiteralPath (Join-Path $scratch 'vitest/repeat-failed.txt')) -and $repeatMetas.Count -ge 1 -and $repeatMetas.Count -le 6
} else {
    $repeatMetas.Count -eq 6
}
$viteRestored = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash -eq $preflight.vite_config_sha256
"DECISION=$($decision.decision)"
"MISSING_ARTIFACTS=$($missing.Count)"
"INITIAL_RUN_ARTIFACTS_VALID=$initialRunArtifactsValid"
"REPEAT_ARTIFACTS_VALID=$repeatArtifactsValid"
"VITE_HASH_RESTORED=$viteRestored"
if ($missing.Count -ne 0 -or -not $initialRunArtifactsValid -or -not $repeatArtifactsValid -or -not $viteRestored) { $missing; exit 1 }
if ($decision.decision -eq 'rejected') {
    $status = @(git status --short --untracked-files=all 2>$null)
    $aRestored = (Get-FileHash -Algorithm SHA256 $candidate.project_path).Hash -eq $candidate.project_a_sha256 -and
                 (Get-FileHash -Algorithm SHA256 $candidate.sources_path).Hash -eq $candidate.sources_a_sha256
    "A_RESTORED=$aRestored"
    "STATUS_COUNT=$($status.Count)"
    if (-not $aRestored -or $status.Count -ne 0) { $status; exit 1 }
}
```

Expected: all path-specific artifacts exist, every normal retry has exactly six `repeat-*-meta.json` files (or a bounded partial set plus `repeat-failed.txt`), Vite is byte-identical to preflight, and a rejected result has a clean exact-A worktree. An early B-run rejection requires its failed readable log/meta but does not falsely require import or aggregate artifacts that were never reached.

- [ ] **Step 3: Write the verification document from literal artifacts**

Create `docs/superpowers/verification/2026-07-15-lucide-direct-import-validation.md` with `apply_patch`. Use these exact sections:

```markdown
# Lucide Direct Import Validation Verification

## Scope and Starting State
## Environment
## Candidate Patch
## Warm-Ups
## Recorded A/B Runs
## Target Test-File Medians
## Import Mechanism
## Target Import Trees
## Retention Criteria
## Retry Decision
## Final Decision
## Correctness Verification
## Limitations
```

For the normal completed protocol, populate them with literal values from `preflight.json`, every warm-up/recorded/repeat metadata file, `ab-initial-summary.json`, `decision.json`, `import-mechanism.txt`, `import-attribution.csv`, and the four target subtree files. Include:

- the starting commit, tool versions, power scheme, and Defender observation;
- both discarded warm-up results and the exact recorded order;
- every A/B wall time, inventory, medians, and percentage delta;
- both target-file medians per side as the primary quantitative result;
- whether the predeclared retry ran and, if so, first and pooled medians;
- the qualitative A/B import-tree result without comparing single-run import durations;
- all six retention gates with pass/fail evidence;
- retained/rejected outcome and exact restoration state;
- focused/check/full-gate results that actually ran;
- the limitation that full-suite timing noise can still reject a correct candidate and that about 20 root-import consumers remain outside this slice.

For an early `candidate_recorded_failure`, use the same headings but state explicitly that aggregate medians, import profiling, retry, focused checks, and the full gate were not reached. Include both warm-ups, every recorded run attempted through the failed B label, the readable failed log diagnosis, the conservative rejection reason, and exact A restoration. Do not invent missing retention results.

Expected: a self-contained summary; raw logs and JSON remain outside the repository.

- [ ] **Step 4: Review and commit only the evidence document**

Run:

```powershell
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$paths = @()
$paths += git diff --name-only
$paths += git diff --cached --name-only
$paths += git ls-files --others --exclude-standard 2>$null
$paths = @($paths | Where-Object { $_ } | Sort-Object -Unique)
$paths
if ($paths.Count -ne 1 -or $paths[0] -ne 'docs/superpowers/verification/2026-07-15-lucide-direct-import-validation.md') { exit 1 }
git add -- docs/superpowers/verification/2026-07-15-lucide-direct-import-validation.md
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git commit -m "docs: verify lucide direct imports"
```

Expected: one documentation commit; no scratch artifacts are staged.

- [ ] **Step 5: Verify the final state without pushing**

Run:

```powershell
$status = @(git status --short --untracked-files=all 2>$null)
"STATUS_COUNT=$($status.Count)"
git log -5 --oneline
git show --check --stat --oneline HEAD
if ($status.Count -ne 0) { $status; exit 1 }
```

Expected: clean worktree and whitespace-clean final evidence commit. Do not push.
