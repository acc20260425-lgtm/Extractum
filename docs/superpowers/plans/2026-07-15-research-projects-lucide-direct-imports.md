# Research Projects Lucide Direct Imports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate and, only when every approved retention gate passes, keep direct Lucide imports in the 20 remaining root-import components under `src/lib/components/research-projects/`.

**Architecture:** Build one reversible 20-file candidate with exact A/B byte snapshots outside the repository, compare A and B using warm-ups plus alternating complete Vitest runs, and use per-test import trees to prove the selected research-project paths no longer traverse the Lucide barrel. Add a directory-wide raw-source contract only after a retained decision, then commit code and verification evidence separately.

**Tech Stack:** Windows PowerShell, Git, Svelte 5, `@lucide/svelte` 1.17.0 direct exports, Vitest 4.1.5 JSON/import-duration reporters, Vite eager `?raw` globs.

## Global Constraints

- Implement `docs/superpowers/specs/2026-07-15-research-projects-lucide-direct-imports-design.md` at or after approved commit `e284b783`.
- The performance candidate may modify only the 20 Svelte files listed in Task 1; do not modify `ProjectRailPanel.svelte`, `SourcesTab.svelte`, another UI area, or any package/configuration file permanently.
- Preserve every existing local icon identifier, use site, markup node, prop, event, style, and accessibility behavior.
- Use canonical modules for the four deprecated aliases: `AlertTriangle -> triangle-alert`, `Edit3 -> pen-line`, `PlayCircle -> circle-play`, and `XCircle -> circle-x`.
- Do not introduce a local icon facade or use `alert-triangle`, `edit-3`, `play-circle`, or `x-circle`.
- Do not change `docs/value-registry.md`; this slice introduces no domain string value.
- Use `npm.cmd` for npm scripts on Windows. Direct `node.exe scripts/run-vitest.mjs` is allowed for reporter options.
- Keep raw JSON, logs, snapshots, patches, scripts, and summaries under one absolute `%TEMP%` scratch directory outside the repository.
- Run benchmark commands sequentially with no active Vitest, Vite, Tauri-dev, or Extractum process. Record, but do not change, the active power scheme and Defender real-time-protection state.
- Performance A and B must execute identical committed test inventories. Create the source contract only after the A/B decision.
- Run one discarded warm-up per side, then three recorded runs per side in `A/B/A/B/A/B` order.
- Permit exactly one additional `A/B/A/B/A/B` sequence only when all non-timing gates pass and initial B is more than 5% and no more than 8% slower than A.
- Missing/unreadable run metadata or a confirmed A failure invalidates the session. A confirmed B failure rejects the candidate. Apply the same rule to the retry.
- Switch variants from exact external byte snapshots and verify all 20 SHA-256 hashes after every transition. A partial switch is infrastructure failure, never a measurable variant.
- Do not push after the final commit unless the user separately requests it.

---

### Task 1: Establish the Clean Baseline and External Snapshot

**Files:**
- Read: `docs/superpowers/specs/2026-07-15-research-projects-lucide-direct-imports-design.md`
- Read: `vite.config.js`
- Read: `src/lib/components/research-projects/*.svelte`
- Produce outside repository: `%TEMP%/extractum-research-lucide-*/A/*.svelte`
- Produce outside repository: `%TEMP%/extractum-research-lucide-*/preflight.json`

**Interfaces:**
- Produces: `%TEMP%/extractum-research-lucide-current.txt`, the scratch pointer used by every later task.
- Produces: `preflight.json` with the exact 20 paths, A hashes, environment, config hash, and import-duration capabilities.
- Preserves: clean baseline A.

- [ ] **Step 1: Verify the approved clean baseline**

Run:

```powershell
$status = @(git status --short --untracked-files=all 2>$null)
git merge-base --is-ancestor e284b783 HEAD
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

Expected: both lists are empty. Close processes through their owner and rerun; do not terminate them automatically.

- [ ] **Step 3: Create the external scratch directory**

Run:

```powershell
$scratch = Join-Path $env:TEMP ("extractum-research-lucide-" + [guid]::NewGuid().ToString('N'))
$pointer = Join-Path $env:TEMP 'extractum-research-lucide-current.txt'
New-Item -ItemType Directory -Path $scratch -ErrorAction Stop | Out-Null
New-Item -ItemType Directory -Path (Join-Path $scratch 'A') -ErrorAction Stop | Out-Null
New-Item -ItemType Directory -Path (Join-Path $scratch 'B') -ErrorAction Stop | Out-Null
New-Item -ItemType Directory -Path (Join-Path $scratch 'vitest') -ErrorAction Stop | Out-Null
Set-Content -LiteralPath $pointer -Value $scratch -Encoding UTF8
$resolvedScratch = (Resolve-Path -LiteralPath $scratch).Path
$repo = (Resolve-Path -LiteralPath '.').Path.TrimEnd('\')
"SCRATCH=$resolvedScratch"
if ($resolvedScratch.StartsWith($repo, [StringComparison]::OrdinalIgnoreCase)) { exit 1 }
```

Expected: one new absolute scratch directory under `%TEMP%`, outside the repository.

- [ ] **Step 4: Verify the exact live source inventory and installed canonical exports**

Run:

```powershell
$componentDir = 'src/lib/components/research-projects'
$expected = @(
    'ConnectFromLibrary.svelte',
    'IconRail.svelte',
    'Inspector.svelte',
    'LibraryFilterRail.svelte',
    'LibraryInspector.svelte',
    'LibraryTelegramDialogImport.svelte',
    'LibraryWorkspace.svelte',
    'LibraryYoutubePlaylistImport.svelte',
    'LibraryYoutubeSmartImport.svelte',
    'ProjectInspector.svelte',
    'ProjectRail.svelte',
    'ProjectRunReportPanel.svelte',
    'ProjectRunsScreen.svelte',
    'ProjectRunsTab.svelte',
    'ProjectsShell.svelte',
    'RunDock.svelte',
    'TopCommandBar.svelte',
    'YoutubeSummaryResultView.svelte',
    'YoutubeSummaryRunDialog.svelte',
    'YoutubeSummaryRunsPanel.svelte'
) | Sort-Object
$actual = @(
    Get-ChildItem -LiteralPath $componentDir -Filter '*.svelte' |
        Where-Object { Select-String -LiteralPath $_.FullName -SimpleMatch 'from "@lucide/svelte"' -Quiet } |
        Select-Object -ExpandProperty Name |
        Sort-Object
)
$icons = @(
    'activity','triangle-alert','book-open','braces','check','chevron-down',
    'chevron-left','chevron-right','download','pen-line','external-link','eye',
    'file-json','file-text','folder','folder-kanban','layers','library','link-2',
    'minus','panel-left-close','panel-left-open','pencil','play','circle-play',
    'plus','refresh-cw','save','search','settings','shield-check','trash-2','x','circle-x'
)
$missingIcons = @($icons | Where-Object { -not (Test-Path -LiteralPath "node_modules/@lucide/svelte/dist/icons/$_.js") })
"ROOT_IMPORT_COUNT=$($actual.Count)"
"MISSING_DIRECT_MODULES=$($missingIcons.Count)"
if (($actual -join "`n") -ne ($expected -join "`n") -or $missingIcons.Count -ne 0) {
    Compare-Object $expected $actual
    $missingIcons
    exit 1
}
```

Expected: exactly the 20 approved files and all 34 canonical direct modules exist. Any inventory drift stops the slice for scope review.

- [ ] **Step 5: Capture A bytes, hashes, environment, and Vitest capabilities**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$componentDir = 'src/lib/components/research-projects'
$paths = @(
    Get-ChildItem -LiteralPath $componentDir -Filter '*.svelte' |
        Where-Object { Select-String -LiteralPath $_.FullName -SimpleMatch 'from "@lucide/svelte"' -Quiet } |
        ForEach-Object { $_.FullName.Substring((Resolve-Path '.').Path.Length + 1).Replace('\','/') } |
        Sort-Object
)
$files = @()
foreach ($path in $paths) {
    $name = Split-Path -Leaf $path
    Copy-Item -LiteralPath $path -Destination (Join-Path $scratch "A/$name") -Force
    $files += [pscustomobject]@{
        path = $path
        name = $name
        a_sha256 = (Get-FileHash -Algorithm SHA256 $path).Hash
    }
}
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
    vite_config_sha256 = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash
    import_flag_present = (($help -join "`n") -match '--experimental\.importDurations\.print\b')
    import_limit_present = (($help -join "`n") -match '--experimental\.importDurations\.limit\b')
    files = $files
}
$help | Set-Content -LiteralPath (Join-Path $scratch 'vitest-expanded-help.txt') -Encoding UTF8
$preflight | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Encoding UTF8
$copied = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'A') -Filter '*.svelte')
"A_SNAPSHOT_COUNT=$($copied.Count)"
"HELP_EXIT=$helpCode"
if ($files.Count -ne 20 -or $copied.Count -ne 20 -or $helpCode -ne 0) { exit 1 }
```

Expected: `preflight.json` and 20 exact A snapshots exist outside the repository.

---

### Task 2: Build and Prove the Reversible 20-File Candidate

**Files:**
- Modify temporarily: the 20 approved `.svelte` files from Task 1
- Produce outside repository: `%TEMP%/extractum-research-lucide-*/B/*.svelte`
- Produce outside repository: `candidate.patch`, `candidate.json`, `set-variant.ps1`

**Interfaces:**
- Consumes: `preflight.json` and A snapshots.
- Produces: exact B snapshots and a variant switcher used by Tasks 3-6.
- Leaves: exact B with only the 20 approved source files modified.

- [ ] **Step 1: Migrate the first five components**

Use `apply_patch` to replace each file's single root Lucide import with the exact block shown:

`ConnectFromLibrary.svelte`:

```svelte
  import Check from "@lucide/svelte/icons/check";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
```

`IconRail.svelte`:

```svelte
  import Activity from "@lucide/svelte/icons/activity";
  import FolderKanban from "@lucide/svelte/icons/folder-kanban";
  import Library from "@lucide/svelte/icons/library";
  import Settings from "@lucide/svelte/icons/settings";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
```

`Inspector.svelte`:

```svelte
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Minus from "@lucide/svelte/icons/minus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
```

`LibraryFilterRail.svelte`:

```svelte
  import PanelLeftClose from "@lucide/svelte/icons/panel-left-close";
  import PanelLeftOpen from "@lucide/svelte/icons/panel-left-open";
```

`LibraryInspector.svelte`:

```svelte
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Link2 from "@lucide/svelte/icons/link-2";
  import PlayCircle from "@lucide/svelte/icons/circle-play";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
```

Expected: only the Lucide import block changes in each file; `PlayCircle` keeps its local name but uses the canonical module.

- [ ] **Step 2: Migrate the next five components**

Use `apply_patch` with these exact replacement blocks:

`LibraryTelegramDialogImport.svelte`:

```svelte
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
```

`LibraryWorkspace.svelte`:

```svelte
  import BookOpen from "@lucide/svelte/icons/book-open";
  import Edit3 from "@lucide/svelte/icons/pen-line";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
```

`LibraryYoutubePlaylistImport.svelte`:

```svelte
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
```

`LibraryYoutubeSmartImport.svelte`:

```svelte
  import Eye from "@lucide/svelte/icons/eye";
  import Plus from "@lucide/svelte/icons/plus";
```

`ProjectInspector.svelte`:

```svelte
  import ChevronLeft from "@lucide/svelte/icons/chevron-left";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Play from "@lucide/svelte/icons/play";
  import PlayCircle from "@lucide/svelte/icons/circle-play";
  import Trash2 from "@lucide/svelte/icons/trash-2";
```

Expected: only import blocks change; `Edit3` and `PlayCircle` use canonical modules.

- [ ] **Step 3: Migrate the next five components**

Use `apply_patch` with these exact replacement blocks:

`ProjectRail.svelte`:

```svelte
  import Search from "@lucide/svelte/icons/search";
```

`ProjectRunReportPanel.svelte`:

```svelte
  import Braces from "@lucide/svelte/icons/braces";
  import FileJson from "@lucide/svelte/icons/file-json";
  import Layers from "@lucide/svelte/icons/layers";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
```

`ProjectRunsScreen.svelte`:

```svelte
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Save from "@lucide/svelte/icons/save";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import XCircle from "@lucide/svelte/icons/circle-x";
```

`ProjectRunsTab.svelte`:

```svelte
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
```

`ProjectsShell.svelte`:

```svelte
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Folder from "@lucide/svelte/icons/folder";
  import FolderKanban from "@lucide/svelte/icons/folder-kanban";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
```

Expected: only import blocks change; `XCircle` uses `circle-x`.

- [ ] **Step 4: Migrate the final five components**

Use `apply_patch` with these exact replacement blocks:

`RunDock.svelte`:

```svelte
  import Download from "@lucide/svelte/icons/download";
```

`TopCommandBar.svelte`:

```svelte
  import Download from "@lucide/svelte/icons/download";
  import Play from "@lucide/svelte/icons/play";
```

`YoutubeSummaryResultView.svelte`:

```svelte
  import AlertTriangle from "@lucide/svelte/icons/triangle-alert";
  import FileText from "@lucide/svelte/icons/file-text";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
```

`YoutubeSummaryRunDialog.svelte`:

```svelte
  import PlayCircle from "@lucide/svelte/icons/circle-play";
```

`YoutubeSummaryRunsPanel.svelte`:

```svelte
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import XCircle from "@lucide/svelte/icons/circle-x";
```

Expected: only import blocks change; all three deprecated local names use canonical modules.

- [ ] **Step 5: Validate scope and capture exact B bytes**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$rootOffenders = @()
$badLucideImports = @()
$deprecatedImports = @()
foreach ($entry in $preflight.files) {
    $source = Get-Content -LiteralPath $entry.path -Raw
    if ($source -match 'from\s+["'']@lucide/svelte["'']') { $rootOffenders += $entry.path }
    foreach ($match in [regex]::Matches($source, 'from\s+["''](@lucide/svelte[^"'']*)["'']')) {
        $specifier = $match.Groups[1].Value
        if (-not $specifier.StartsWith('@lucide/svelte/icons/')) {
            $badLucideImports += "$($entry.path):$specifier"
        }
        if ($specifier -match '^@lucide/svelte/icons/(alert-triangle|edit-3|play-circle|x-circle)$') {
            $deprecatedImports += "$($entry.path):$specifier"
        }
    }
}
$changed = @(git diff --name-only | Sort-Object)
$expected = @($preflight.files.path | Sort-Object)
"ROOT_OFFENDERS=$($rootOffenders.Count)"
"BAD_LUCIDE_IMPORTS=$($badLucideImports.Count)"
"DEPRECATED_DIRECT_PATHS=$($deprecatedImports.Count)"
"CHANGED_COUNT=$($changed.Count)"
if ($rootOffenders.Count -ne 0 -or $badLucideImports.Count -ne 0 -or $deprecatedImports.Count -ne 0 -or
    ($changed -join "`n") -ne ($expected -join "`n")) {
    $rootOffenders; $badLucideImports; $deprecatedImports
    Compare-Object $expected $changed
    exit 1
}
$candidateFiles = @()
foreach ($entry in $preflight.files) {
    Copy-Item -LiteralPath $entry.path -Destination (Join-Path $scratch "B/$($entry.name)") -Force
    $candidateFiles += [pscustomobject]@{
        path = [string]$entry.path
        name = [string]$entry.name
        a_sha256 = [string]$entry.a_sha256
        b_sha256 = (Get-FileHash -Algorithm SHA256 $entry.path).Hash
    }
}
$patchPath = Join-Path $scratch 'candidate.patch'
git diff --binary --output=$patchPath -- @expected
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$candidate = [ordered]@{
    baseline_commit = [string]$preflight.commit
    snapshot_a = (Join-Path $scratch 'A')
    snapshot_b = (Join-Path $scratch 'B')
    patch = $patchPath
    files = $candidateFiles
}
$candidate | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Encoding UTF8
git diff --check -- @expected
if ($LASTEXITCODE -ne 0 -or $candidateFiles.Count -ne 20 -or (Get-Item -LiteralPath $patchPath).Length -le 0) { exit 1 }
```

Expected: exactly 20 changed paths, no root/deprecated imports, 20 exact B snapshots, and one nonempty candidate patch.

- [ ] **Step 6: Create the byte-snapshot switcher and prove A -> B -> A -> B**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$switcher = Join-Path $scratch 'set-variant.ps1'
$source = @'
param([Parameter(Mandatory=$true)][ValidateSet('A','B')][string]$Variant)
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
function Test-Variant([ValidateSet('A','B')][string]$Expected) {
    foreach ($entry in $candidate.files) {
        $wanted = if ($Expected -eq 'A') { [string]$entry.a_sha256 } else { [string]$entry.b_sha256 }
        if (-not (Test-Path -LiteralPath $entry.path)) { return $false }
        if ((Get-FileHash -Algorithm SHA256 $entry.path).Hash -ne $wanted) { return $false }
    }
    return $true
}
function Copy-Variant([ValidateSet('A','B')][string]$Target) {
    $snapshot = if ($Target -eq 'A') { [string]$candidate.snapshot_a } else { [string]$candidate.snapshot_b }
    foreach ($entry in $candidate.files) {
        Copy-Item -LiteralPath (Join-Path $snapshot $entry.name) -Destination $entry.path -Force -ErrorAction Stop
    }
}
try {
    Copy-Variant $Variant
    if (-not (Test-Variant $Variant)) { throw "Variant $Variant hash verification failed." }
} catch {
    try {
        Copy-Variant 'A'
        if (-not (Test-Variant 'A')) { throw 'A restoration hash verification failed.' }
    } catch {
        throw "Variant switch failed and exact A could not be restored: $($_.Exception.Message)"
    }
    throw "Variant switch failed; exact A restored: $($_.Exception.Message)"
}
"VARIANT=$Variant"
'@
Set-Content -LiteralPath $switcher -Value $source -Encoding UTF8
foreach ($variant in @('A','B','A','B')) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $switcher -Variant $variant
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$bOk = @($candidate.files | Where-Object { (Get-FileHash -Algorithm SHA256 $_.path).Hash -ne $_.b_sha256 }).Count -eq 0
$status = @(git status --short --untracked-files=all 2>$null)
"B_REPRODUCED=$bOk"
"DIRTY_COUNT=$($status.Count)"
if (-not $bOk -or $status.Count -ne 20) { $status; exit 1 }
```

Expected: all four transitions pass, final state is exact B, and only the 20 owned files are dirty.

---

### Task 3: Run Warm-Ups and the Initial Alternating A/B Sequence

**Files:**
- Toggle temporarily: the 20 owned Svelte files via `set-variant.ps1`
- Produce outside repository: `invoke-complete-variant.ps1`
- Produce outside repository: `vitest/warmup-*.json`, `vitest/recorded-*.json`, logs, and metadata

**Interfaces:**
- Consumes: `candidate.json` and `set-variant.ps1`.
- Produces: two discarded warm-ups plus three recorded nonempty results per side, or a classified early rejection/invalidation.
- Leaves: B after a normal sequence; A after an early B rejection or invalidated session.

- [ ] **Step 1: Create the external complete-suite runner**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$runnerPath = Join-Path $scratch 'invoke-complete-variant.ps1'
$runnerSource = @'
param(
    [Parameter(Mandatory=$true)][string]$Label,
    [Parameter(Mandatory=$true)][ValidateSet('A','B')][string]$Variant,
    [switch]$Recorded
)
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
$switcher = Join-Path $scratch 'set-variant.ps1'
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $switcher -Variant $Variant
if ($LASTEXITCODE -ne 0) { exit 2 }
$report = [IO.Path]::GetFullPath((Join-Path $vitestDir "$Label.json"))
$repo = (Resolve-Path -LiteralPath '.').Path.TrimEnd('\')
if ($report.StartsWith($repo, [StringComparison]::OrdinalIgnoreCase)) { exit 2 }
$watch = [Diagnostics.Stopwatch]::StartNew()
$output = @(& node.exe scripts/run-vitest.mjs run --reporter=json --reporter=default "--outputFile.json=$report" 2>&1)
$code = $LASTEXITCODE
$watch.Stop()
$log = Join-Path $vitestDir "$Label.log"
$output | Set-Content -LiteralPath $log -Encoding UTF8
$json = $null
if (Test-Path -LiteralPath $report) {
    try { $json = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json } catch { $json = $null }
}
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
    log = $log
}
$meta | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $vitestDir "$Label-meta.json") -Encoding UTF8
$meta | Format-List
if ($code -ne 0 -or -not $json -or -not $json.success -or $meta.files -le 0 -or $meta.tests -le 0) { exit 1 }
'@
Set-Content -LiteralPath $runnerPath -Value $runnerSource -Encoding UTF8
"RUNNER=$runnerPath"
if (-not (Test-Path -LiteralPath $runnerPath) -or (Get-Item -LiteralPath $runnerPath).Length -le 0) { exit 1 }
```

Expected: a nonempty runner exists only in the external scratch directory. Exit `2` means failure before Vitest metadata; exit `1` with metadata means a started but failed/empty run.

- [ ] **Step 2: Execute both discarded warm-ups**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-complete-variant.ps1'
$switcher = Join-Path $scratch 'set-variant.ps1'
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Label 'warmup-A' -Variant 'A'
$aCode = $LASTEXITCODE
if ($aCode -eq 0) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Label 'warmup-B' -Variant 'B'
    $bCode = $LASTEXITCODE
} else { $bCode = -1 }
if ($aCode -ne 0 -or $bCode -ne 0) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $switcher -Variant 'A'
    if ($LASTEXITCODE -ne 0) { throw 'Warm-up failed and exact A restoration also failed.' }
    [ordered]@{reason='warmup_failure';a_exit=$aCode;b_exit=$bCode;scratch=$scratch} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'session-invalid.json') -Encoding UTF8
    throw 'Warm-up failed or was empty. Preserve scratch, investigate, create a fresh Task 1 session, and restart from both warm-ups.'
}
```

Expected: both warm-ups pass with nonzero file/test inventories. Any failure restores A and invalidates the complete session; warm-up timings are never recorded in aggregates.

- [ ] **Step 3: Execute the recorded A/B/A/B/A/B sequence with explicit failure policy**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-complete-variant.ps1'
$switcher = Join-Path $scratch 'set-variant.ps1'
$vitestDir = Join-Path $scratch 'vitest'
$sequence = @('A','B','A','B','A','B')
$earlyRejected = $false
for ($index = 0; $index -lt $sequence.Count; $index++) {
    $variant = $sequence[$index]
    $label = "recorded-{0:D2}-{1}" -f ($index + 1), $variant
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Label $label -Variant $variant -Recorded
    if ($LASTEXITCODE -eq 0) { continue }
    $metaPath = Join-Path $vitestDir "$label-meta.json"
    $failedMeta = $null
    if (Test-Path -LiteralPath $metaPath) {
        try { $failedMeta = Get-Content -LiteralPath $metaPath -Raw | ConvertFrom-Json } catch { $failedMeta = $null }
    }
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $switcher -Variant 'A'
    if ($LASTEXITCODE -ne 0) { throw "$label failed and exact A restoration also failed." }
    if ($null -eq $failedMeta) {
        [ordered]@{reason='runner_infrastructure_failure';failed_label=$label;failed_variant=$variant;scratch=$scratch} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'session-invalid.json') -Encoding UTF8
        throw "$label failed before readable run metadata. Preserve scratch and restart from fresh warm-ups after investigation."
    }
    $confirmedRunFailure = $failedMeta.exit -ne 0 -or -not $failedMeta.success -or
                           $failedMeta.files -le 0 -or $failedMeta.tests -le 0
    if (-not $confirmedRunFailure) {
        [ordered]@{reason='runner_post_meta_failure';failed_label=$label;failed_variant=$variant;scratch=$scratch} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'session-invalid.json') -Encoding UTF8
        throw "$label failed after successful-looking metadata. Treat as infrastructure failure and restart the session."
    }
    if ($variant -eq 'A') {
        [ordered]@{reason='baseline_recorded_failure';failed_label=$label;scratch=$scratch} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'session-invalid.json') -Encoding UTF8
        throw "Baseline A run $label failed. Invalidate the session; do not reject B from this evidence."
    }
    $decision = [ordered]@{
        decision = 'rejected'
        reason = 'candidate_recorded_failure'
        failed_label = $label
        failed_variant = 'B'
        retry_used = $false
        scratch = $scratch
    }
    $decision | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'decision.json') -Encoding UTF8
    Set-Content -LiteralPath (Join-Path $scratch 'early-rejection.txt') -Value $label -Encoding UTF8
    $earlyRejected = $true
    break
}
if (-not $earlyRejected) {
    Set-Content -LiteralPath (Join-Path $scratch 'recorded-complete.txt') -Value 'A/B/A/B/A/B' -Encoding UTF8
}
"EARLY_REJECTED=$earlyRejected"
```

Expected: normally six successful recorded runs and final exact B. A confirmed B failure creates an early rejected `decision.json` and exact A; infrastructure/A failures invalidate the session and stop execution.

---

### Task 4: Capture A and B Import Trees

Skip this task when `early-rejection.txt` exists.

**Files:**
- Toggle temporarily: the 20 owned Svelte files through `set-variant.ps1`
- Conditionally modify and restore: `vite.config.js`
- Produce outside repository: `invoke-import-variant.ps1`, `vitest/import-*.log`, three A/B subtree pairs, and `import-attribution.csv`

**Interfaces:**
- Consumes: exact A/B snapshots and Vitest capability evidence.
- Produces: qualitative import attribution for `ResearchProjectsShell.test.ts`, `Inspector.test.ts`, and `RunDock.test.ts`.
- Leaves: exact B and byte-identical `vite.config.js`.

- [ ] **Step 1: Create the import runner and try the comprehensive CLI mechanism**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
if (Test-Path -LiteralPath (Join-Path $scratch 'early-rejection.txt')) { Write-Host 'Skip Task 4'; return }
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
$runnerPath = Join-Path $scratch 'invoke-import-variant.ps1'
$runnerSource = @'
param(
    [Parameter(Mandatory=$true)][ValidateSet('A','B')][string]$Variant,
    [Parameter(Mandatory=$true)][string]$Prefix,
    [switch]$Cli
)
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $scratch 'set-variant.ps1') -Variant $Variant
if ($LASTEXITCODE -ne 0) { exit 2 }
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
    variant=$Variant; mechanism=$Prefix; exit=$code
    wall_seconds=[math]::Round($watch.Elapsed.TotalSeconds,3)
    success=if ($json) { [bool]$json.success } else { $false }
    files=if ($json) { @($json.testResults).Count } else { 0 }
    tests=if ($json) { [int]$json.numTotalTests } else { 0 }
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
$roots = 'ResearchProjectsShell.test.ts','Inspector.test.ts','RunDock.test.ts'
$cliComplete = $false
if ($cliAvailable -and $cliRunsOk) {
    $a = Get-Content -LiteralPath (Join-Path $vitestDir 'import-cli-A.log') -Raw
    $b = Get-Content -LiteralPath (Join-Path $vitestDir 'import-cli-B.log') -Raw
    $cliComplete = $a -match 'Import Duration Breakdown' -and $b -match 'Import Duration Breakdown'
    foreach ($root in $roots) { $cliComplete = $cliComplete -and $a.Contains($root) -and $b.Contains($root) }
}
"CLI_IMPORT_COMPLETE=$cliComplete"
if ($cliComplete) {
    Copy-Item -LiteralPath (Join-Path $vitestDir 'import-cli-A.log') -Destination (Join-Path $vitestDir 'import-A.log') -Force
    Copy-Item -LiteralPath (Join-Path $vitestDir 'import-cli-B.log') -Destination (Join-Path $vitestDir 'import-B.log') -Force
    Set-Content -LiteralPath (Join-Path $vitestDir 'import-mechanism.txt') -Value 'CLI print with limit=2000' -Encoding UTF8
} else {
    Set-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-required.txt') -Value 'required' -Encoding UTF8
}
```

Expected: CLI logs become canonical only when both complete runs contain the breakdown and all three roots. CLI rejection/incomplete output requests the reversible fallback instead of producing a verdict.

- [ ] **Step 2: Apply the temporary config fallback only when requested**

Skip this step when `vitest/import-fallback-required.txt` does not exist. Otherwise use `apply_patch` to replace:

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

Expected: only the temporary exported Vitest config block changes. Do not run formatting or commit this state.

- [ ] **Step 3: Run fallback A/B and restore Vite before reading the result**

Run only when the fallback marker exists:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-import-variant.ps1'
$vitestDir = Join-Path $scratch 'vitest'
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Variant 'A' -Prefix 'import-fallback'
$aCode = $LASTEXITCODE
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Variant 'B' -Prefix 'import-fallback'
$bCode = $LASTEXITCODE
[ordered]@{a_exit=$aCode;b_exit=$bCode} | ConvertTo-Json |
    Set-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-codes.json') -Encoding UTF8
```

Immediately use `apply_patch` to restore the exact original block:

```js
export const VITEST_TEST_CONFIG = {
  pool: "threads",
};
```

Then run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
$codes = Get-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-codes.json') -Raw | ConvertFrom-Json
$a = Get-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-A.log') -Raw
$b = Get-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-B.log') -Raw
$complete = $a -match 'Import Duration Breakdown' -and $b -match 'Import Duration Breakdown'
foreach ($root in 'ResearchProjectsShell.test.ts','Inspector.test.ts','RunDock.test.ts') {
    $complete = $complete -and $a.Contains($root) -and $b.Contains($root)
}
$viteRestored = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash -eq $preflight.vite_config_sha256
"VITE_HASH_RESTORED=$viteRestored"
"FALLBACK_IMPORT_COMPLETE=$complete"
if (-not $viteRestored -or $codes.a_exit -ne 0 -or $codes.b_exit -ne 0 -or -not $complete) { exit 1 }
Copy-Item -LiteralPath (Join-Path $vitestDir 'import-fallback-A.log') -Destination (Join-Path $vitestDir 'import-A.log') -Force
Copy-Item -LiteralPath (Join-Path $vitestDir 'import-fallback-B.log') -Destination (Join-Path $vitestDir 'import-B.log') -Force
Set-Content -LiteralPath (Join-Path $vitestDir 'import-mechanism.txt') -Value 'config fallback with limit=2000' -Encoding UTF8
```

Expected: both canonical logs exist, Vite is byte-identical to preflight before interpretation, and exact B remains active.

- [ ] **Step 4: Extract the three target subtrees and enforce qualitative attribution**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
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
    if ($result.Count -le 1) { throw "Import subtree for $rootName is empty; reporter format is not attributable." }
    return [string[]]$result.ToArray()
}
$roots = 'ResearchProjectsShell.test.ts','Inspector.test.ts','RunDock.test.ts'
$summary = @()
foreach ($variant in 'A','B') {
    $lines = @(Get-Content -LiteralPath (Join-Path $vitestDir "import-$variant.log"))
    foreach ($root in $roots) {
        $subtree = @(Get-ImportSubtree $lines $root)
        $safeRoot = $root -replace '\.test\.ts$',''
        $subtree | Set-Content -LiteralPath (Join-Path $vitestDir "import-$variant-$safeRoot-subtree.txt") -Encoding UTF8
        $normalized = ($subtree -join "`n").Replace('\','/')
        $summary += [pscustomobject]@{
            variant = $variant
            root = $root
            rows = $subtree.Count
            contains_icons_index = $normalized -match 'icons/index\.js'
        }
    }
}
$summary | Export-Csv -LiteralPath (Join-Path $vitestDir 'import-attribution.csv') -NoTypeInformation -Encoding UTF8
$summary | Format-Table
$aRows = @($summary | Where-Object variant -eq 'A')
$bRows = @($summary | Where-Object variant -eq 'B')
if ($aRows.Count -ne 3 -or $bRows.Count -ne 3) { exit 1 }
if (@($aRows | Where-Object { -not $_.contains_icons_index }).Count -ne 0) { exit 1 }
if (@($bRows | Where-Object contains_icons_index).Count -ne 0) { exit 1 }
```

Expected: every A subtree is nonempty and contains `icons/index.js`; no B subtree contains it. Global Lucide-index presence outside these roots is ignored.

#### Approved Execution Correction: Scoped Attribution When Top 2000 Omits a Root

During inline execution, both complete CLI runs passed, but the B global Top
2000 omitted `Inspector.test.ts` and `RunDock.test.ts` after their import totals
dropped. The original completeness check saw those names earlier in the normal
test-result output and therefore produced a false `CLI_IMPORT_COMPLETE=True`.
The parser correctly rejected the missing breakdown roots.

The user approved this bounded correction on 2026-07-15: retain both complete
instrumented A/B logs as suite-wide evidence, then run one additional scoped
A/B pair containing exactly the three target tests. Use the scoped breakdowns
only for qualitative subtree attribution; do not add their wall times to the
complete-suite performance decision.

Run for each exact byte-snapshot variant:

```powershell
$vitestArgs = @(
    'scripts/run-vitest.mjs', 'run',
    'src/lib/components/research-projects/ResearchProjectsShell.test.ts',
    'src/lib/components/research-projects/Inspector.test.ts',
    'src/lib/components/research-projects/RunDock.test.ts',
    '--experimental.importDurations.print',
    '--experimental.importDurations.limit=2000',
    '--reporter=default'
)
node.exe @vitestArgs
```

Save the outputs as `vitest/import-scoped-A.log` and
`vitest/import-scoped-B.log`, require both runs to pass and contain the
breakdown plus all three roots, and copy them to the canonical `import-A.log`
and `import-B.log` paths consumed by the subtree parser. Preserve
`import-cli-A.log` and `import-cli-B.log` unchanged. Record
`full CLI evidence plus scoped three-root CLI attribution, limit=2000` in
`import-mechanism.txt`.

---

### Task 5: Compute the Retention Decision and Optional Single Retry

Skip this task when Task 3 already created `decision.json` with reason `candidate_recorded_failure`.

**Files:**
- Toggle temporarily: the 20 owned Svelte files through `set-variant.ps1`
- Produce outside repository: `vitest/ab-runs.csv`, `vitest/ab-initial-summary.json`, optional repeat results, and `decision.json`

**Interfaces:**
- Consumes: six initial reports, import attribution, source state, and the approved thresholds.
- Produces: one deterministic retained/rejected decision.
- Leaves: exact B when retained; exact A when rejected.

- [ ] **Step 1: Run focused correctness checks on B and compute the initial summary**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
if (Test-Path -LiteralPath (Join-Path $scratch 'early-rejection.txt')) { Write-Host 'Skip Task 5'; return }
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $scratch 'set-variant.ps1') -Variant 'B'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$focusedReport = Join-Path $vitestDir 'predecision-focused.json'
$focusedOutput = @(& node.exe scripts/run-vitest.mjs run src/lib/components/research-projects --reporter=json --reporter=default "--outputFile.json=$focusedReport" 2>&1)
$focusedCode = $LASTEXITCODE
$focusedOutput | Set-Content -LiteralPath (Join-Path $vitestDir 'predecision-focused.log') -Encoding UTF8
$focusedJson = if (Test-Path -LiteralPath $focusedReport) { Get-Content -LiteralPath $focusedReport -Raw | ConvertFrom-Json } else { $null }
$focusedNonempty = $focusedJson -and @($focusedJson.testResults).Count -gt 0 -and [int]$focusedJson.numTotalTests -gt 0
$checkOutput = @()
$checkCode = 1
if ($focusedCode -eq 0 -and $focusedJson.success -and $focusedNonempty) {
    $checkOutput = @(& npm.cmd run check 2>&1)
    $checkCode = $LASTEXITCODE
    $checkOutput | Set-Content -LiteralPath (Join-Path $vitestDir 'predecision-check.log') -Encoding UTF8
}
$correctnessGate = $focusedCode -eq 0 -and $focusedJson.success -and $focusedNonempty -and $checkCode -eq 0
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
        shell_ms = Get-FileDuration $report 'src/lib/components/research-projects/ResearchProjectsShell.test.ts'
        inspector_ms = Get-FileDuration $report 'src/lib/components/research-projects/Inspector.test.ts'
        dock_ms = Get-FileDuration $report 'src/lib/components/research-projects/RunDock.test.ts'
    }
}
$inventories = @($rows | ForEach-Object { "$($_.files)/$($_.tests)" } | Sort-Object -Unique)
$a = @($rows | Where-Object variant -eq 'A')
$b = @($rows | Where-Object variant -eq 'B')
$aMedian = Get-Median ([double[]]$a.wall_seconds)
$bMedian = Get-Median ([double[]]$b.wall_seconds)
$delta = (($bMedian - $aMedian) / $aMedian) * 100
$rootOffenders = @()
$badImports = @()
$deprecatedImports = @()
foreach ($file in Get-ChildItem -LiteralPath 'src/lib/components/research-projects' -Filter '*.svelte') {
    $source = Get-Content -LiteralPath $file.FullName -Raw
    if ($source -match 'from\s+["'']@lucide/svelte["'']') { $rootOffenders += $file.FullName }
    foreach ($match in [regex]::Matches($source, 'from\s+["''](@lucide/svelte[^"'']*)["'']')) {
        $specifier = $match.Groups[1].Value
        if (-not $specifier.StartsWith('@lucide/svelte/icons/')) {
            $badImports += "$($file.FullName):$specifier"
        }
        if ($specifier -match '^@lucide/svelte/icons/(alert-triangle|edit-3|play-circle|x-circle)$') {
            $deprecatedImports += "$($file.FullName):$specifier"
        }
    }
}
$sourceGate = $rootOffenders.Count -eq 0 -and $badImports.Count -eq 0 -and $deprecatedImports.Count -eq 0
$attribution = @(Import-Csv -LiteralPath (Join-Path $vitestDir 'import-attribution.csv'))
$aImportGate = @($attribution | Where-Object { $_.variant -eq 'A' -and $_.contains_icons_index -ne 'True' }).Count -eq 0
$bImportGate = @($attribution | Where-Object { $_.variant -eq 'B' -and $_.contains_icons_index -eq 'True' }).Count -eq 0
$importGate = $attribution.Count -eq 6 -and $aImportGate -and $bImportGate
$summary = [ordered]@{
    initial_runs_per_variant = 3
    inventory = if ($inventories.Count -eq 1) { $inventories[0] } else { $inventories -join ',' }
    inventory_equal = $inventories.Count -eq 1
    a_wall_median_seconds = [math]::Round($aMedian,3)
    b_wall_median_seconds = [math]::Round($bMedian,3)
    initial_wall_delta_percent = [math]::Round($delta,3)
    a_shell_median_ms = [math]::Round((Get-Median ([double[]]$a.shell_ms)),3)
    b_shell_median_ms = [math]::Round((Get-Median ([double[]]$b.shell_ms)),3)
    a_inspector_median_ms = [math]::Round((Get-Median ([double[]]$a.inspector_ms)),3)
    b_inspector_median_ms = [math]::Round((Get-Median ([double[]]$b.inspector_ms)),3)
    a_dock_median_ms = [math]::Round((Get-Median ([double[]]$a.dock_ms)),3)
    b_dock_median_ms = [math]::Round((Get-Median ([double[]]$b.dock_ms)),3)
    source_gate = $sourceGate
    import_gate = $importGate
    correctness_gate = $correctnessGate
    focused_exit = $focusedCode
    focused_files = if ($focusedJson) { @($focusedJson.testResults).Count } else { 0 }
    focused_tests = if ($focusedJson) { [int]$focusedJson.numTotalTests } else { 0 }
    check_exit = $checkCode
    retry_required = $delta -gt 5 -and $delta -le 8 -and $inventories.Count -eq 1 -and $sourceGate -and $importGate -and $correctnessGate
}
$rows | Export-Csv -LiteralPath (Join-Path $vitestDir 'ab-runs.csv') -NoTypeInformation -Encoding UTF8
$summary | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $vitestDir 'ab-initial-summary.json') -Encoding UTF8
$summary | Format-List
if ($a.Count -ne 3 -or $b.Count -ne 3) { exit 1 }
```

Expected: three rows per variant, equal live inventories, three diagnostic file medians per side, and explicit source/import/correctness gates. A focused zero-test selection makes `correctness_gate=False`.

- [ ] **Step 2: Run the one permitted retry only inside the predeclared marginal window**

Skip when `ab-initial-summary.json` has `retry_required=false`. Otherwise run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$runner = Join-Path $scratch 'invoke-complete-variant.ps1'
$switcher = Join-Path $scratch 'set-variant.ps1'
$vitestDir = Join-Path $scratch 'vitest'
$sequence = @('A','B','A','B','A','B')
$repeatFailed = $false
for ($index = 0; $index -lt $sequence.Count; $index++) {
    $variant = $sequence[$index]
    $label = "repeat-{0:D2}-{1}" -f ($index + 1), $variant
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner -Label $label -Variant $variant -Recorded
    if ($LASTEXITCODE -eq 0) { continue }
    $metaPath = Join-Path $vitestDir "$label-meta.json"
    $failedMeta = $null
    if (Test-Path -LiteralPath $metaPath) {
        try { $failedMeta = Get-Content -LiteralPath $metaPath -Raw | ConvertFrom-Json } catch { $failedMeta = $null }
    }
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $switcher -Variant 'A'
    if ($LASTEXITCODE -ne 0) { throw "$label failed and exact A restoration also failed." }
    if ($null -eq $failedMeta) {
        [ordered]@{reason='runner_infrastructure_failure';failed_label=$label;failed_variant=$variant;scratch=$scratch} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'session-invalid.json') -Encoding UTF8
        throw "Retry $label failed before readable metadata. Invalidate the whole session and restart from fresh warm-ups."
    }
    $confirmedRunFailure = $failedMeta.exit -ne 0 -or -not $failedMeta.success -or
                           $failedMeta.files -le 0 -or $failedMeta.tests -le 0
    if (-not $confirmedRunFailure) {
        [ordered]@{reason='runner_post_meta_failure';failed_label=$label;failed_variant=$variant;scratch=$scratch} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'session-invalid.json') -Encoding UTF8
        throw "Retry $label failed after successful-looking metadata. Invalidate the session."
    }
    if ($variant -eq 'A') {
        [ordered]@{reason='baseline_repeat_failure';failed_label=$label;scratch=$scratch} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'session-invalid.json') -Encoding UTF8
        throw "Baseline A retry $label failed. Invalidate the whole session; do not reject B."
    }
    Set-Content -LiteralPath (Join-Path $vitestDir 'repeat-failed.txt') -Value $label -Encoding UTF8
    $repeatFailed = $true
    break
}
"REPEAT_FAILED=$repeatFailed"
```

Expected: normally six additional successful runs and final B. Infrastructure/A failure invalidates the full session; confirmed B failure records `repeat-failed.txt` and forces rejection without another retry.

- [ ] **Step 3: Make the final decision and restore the correct variant**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$initial = Get-Content -LiteralPath (Join-Path $scratch 'vitest/ab-initial-summary.json') -Raw | ConvertFrom-Json
$vitestDir = Join-Path $scratch 'vitest'
$repeatFailed = Test-Path -LiteralPath (Join-Path $vitestDir 'repeat-failed.txt')
function Get-Median([double[]]$values) {
    $sorted = @($values | Sort-Object)
    if ($sorted.Count % 2 -eq 1) { return [double]$sorted[[int][math]::Floor($sorted.Count / 2)] }
    return ([double]$sorted[$sorted.Count / 2 - 1] + [double]$sorted[$sorted.Count / 2]) / 2
}
$metaFiles = @(Get-ChildItem -LiteralPath $vitestDir -Filter 'recorded-*-meta.json')
if ($initial.retry_required -and -not $repeatFailed) {
    $metaFiles += @(Get-ChildItem -LiteralPath $vitestDir -Filter 'repeat-*-meta.json')
}
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
    reason = 'protocol_completed'
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
$target = if ($retained) { 'B' } else { 'A' }
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $scratch 'set-variant.ps1') -Variant $target
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$wrongHashes = @($candidate.files | Where-Object {
    $wanted = if ($target -eq 'A') { $_.a_sha256 } else { $_.b_sha256 }
    (Get-FileHash -Algorithm SHA256 $_.path).Hash -ne $wanted
})
if ($wrongHashes.Count -ne 0) { exit 1 }
```

Expected: one final `decision.json`; retained leaves exact B, rejected exact A. No discretionary rerun remains.

---

### Task 6: Add the Directory Contract and Commit a Retained Candidate

Skip this entire task when `decision.json` says `rejected`.

**Files:**
- Create when retained: `src/lib/research-projects-lucide-import-contract.test.ts`
- Modify when retained: the 20 approved Svelte files

**Interfaces:**
- Consumes: retained `decision.json`, exact A/B snapshots, and `set-variant.ps1`.
- Produces: a directory-wide eager raw-source contract and one focused code commit.

- [ ] **Step 1: Restore exact A for the RED phase**

Run only when retained:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$decision = Get-Content -LiteralPath (Join-Path $scratch 'decision.json') -Raw | ConvertFrom-Json
if ($decision.decision -ne 'retained') { throw 'Task 6 may run only for a retained candidate.' }
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $scratch 'set-variant.ps1') -Variant 'A'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: all 20 files match their A hashes and the worktree is clean before creating the contract.

- [ ] **Step 2: Write the aggregate directory contract**

Create `src/lib/research-projects-lucide-import-contract.test.ts` with `apply_patch`:

```ts
import { describe, expect, it } from "vitest";

const componentSources = import.meta.glob<string>(
  "./components/research-projects/*.svelte",
  {
    query: "?raw",
    import: "default",
    eager: true,
  },
);

const LUCIDE_IMPORT = /from\s+["'](@lucide\/svelte[^"']*)["']/g;

function normalize(source: string): string {
  return source.replace(/\r\n/g, "\n");
}

function findOffenders(): string[] {
  return Object.entries(componentSources)
    .flatMap(([path, rawSource]) => {
      const source = normalize(rawSource);
      return [...source.matchAll(LUCIDE_IMPORT)]
        .map((match) => match[1])
        .filter((specifier): specifier is string => Boolean(specifier))
        .filter((specifier) => !specifier.startsWith("@lucide/svelte/icons/"))
        .map((specifier) => `${path}: ${specifier}`);
    })
    .sort();
}

describe("research-projects Lucide import boundary", () => {
  it("uses only direct Lucide icon modules", () => {
    const offenders = findOffenders();
    if (offenders.length > 0) {
      console.error(offenders.join("\n"));
    }
    expect(offenders).toEqual([]);
  });
});
```

Expected: the contract discovers all immediate Svelte files through Vite and aggregates every non-direct Lucide import without hard-coding files, icons, or offender count. On RED it writes the complete offender list to stderr before the assertion, so Vitest diff truncation cannot hide paths.

- [ ] **Step 3: Run the contract to prove RED identifies every current offender**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$redLog = Join-Path $scratch 'contract-red.log'
$output = @(& node.exe scripts/run-vitest.mjs run src/lib/research-projects-lucide-import-contract.test.ts 2>&1)
$code = $LASTEXITCODE
$output | Set-Content -LiteralPath $redLog -Encoding UTF8
$text = $output -join "`n"
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$missingPaths = @($preflight.files | Where-Object { -not $text.Contains($_.name) })
"RED_EXIT=$code"
"RED_MISSING_OFFENDER_PATHS=$($missingPaths.Count)"
if ($code -eq 0 -or $missingPaths.Count -ne 0) { $missingPaths.path; exit 1 }
```

Expected: nonzero RED exit and the aggregated diff names every current root-import component. The contract itself contains no hard-coded count.

- [ ] **Step 4: Restore exact B and prove GREEN**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $scratch 'set-variant.ps1') -Variant 'B'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$badHashes = @($candidate.files | Where-Object { (Get-FileHash -Algorithm SHA256 $_.path).Hash -ne $_.b_sha256 })
if ($badHashes.Count -ne 0) { exit 1 }
& node.exe scripts/run-vitest.mjs run src/lib/research-projects-lucide-import-contract.test.ts
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: exact B is restored and the contract passes.

- [ ] **Step 5: Run focused behavior tests and static checks**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$report = Join-Path $scratch 'vitest/final-focused.json'
& node.exe scripts/run-vitest.mjs run src/lib/components/research-projects src/lib/research-projects-lucide-import-contract.test.ts --reporter=json --reporter=default "--outputFile.json=$report"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$json = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
"FOCUSED_FILES=$(@($json.testResults).Count)"
"FOCUSED_TESTS=$($json.numTotalTests)"
if (-not $json.success -or @($json.testResults).Count -le 0 -or $json.numTotalTests -le 0) { exit 1 }
npm.cmd run check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

Expected: a nonempty focused selection plus the contract pass, followed by a clean Svelte/TypeScript check.

- [ ] **Step 6: Review and commit only the retained code surface**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$paths = @()
$paths += git diff --name-only
$paths += git ls-files --others --exclude-standard 2>$null
$paths = @($paths | Where-Object { $_ } | Sort-Object -Unique)
$expected = @($candidate.files.path) + 'src/lib/research-projects-lucide-import-contract.test.ts'
$expected = @($expected | Sort-Object)
git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
"CHANGED_COUNT=$($paths.Count)"
if (($paths -join "`n") -ne ($expected -join "`n")) {
    Compare-Object $expected $paths
    exit 1
}
git diff -- @($candidate.files.path)
Get-Content -LiteralPath 'src/lib/research-projects-lucide-import-contract.test.ts'
git add -- @($candidate.files.path) 'src/lib/research-projects-lucide-import-contract.test.ts'
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git commit -m "perf: migrate research project lucide imports"
```

Expected: one code commit containing exactly 20 import-block changes and the focused contract.

---

### Task 7: Run the Final Gate and Commit Verification Evidence

**Files:**
- Create: `docs/superpowers/verification/2026-07-15-research-projects-lucide-direct-imports.md`

**Interfaces:**
- Consumes: all scratch metadata, retained/rejected state, and actual verification output.
- Produces: one self-contained evidence document and one documentation commit.
- Preserves: a clean final worktree with no scratch artifacts staged.

- [ ] **Step 1: Run the complete repository gate for retained code**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$decision = Get-Content -LiteralPath (Join-Path $scratch 'decision.json') -Raw | ConvertFrom-Json
if ($decision.decision -eq 'retained') {
    npm.cmd run verify
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} else {
    Write-Host 'Candidate rejected and exact A restored; full verify is not repeated for a documentation-only result.'
}
```

Expected: retained code passes the full repository gate. Rejected exact A skips the redundant final gate.

- [ ] **Step 2: Verify exact final state and required artifact set**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-research-lucide-current.txt') -Raw).Trim()
$candidate = Get-Content -LiteralPath (Join-Path $scratch 'candidate.json') -Raw | ConvertFrom-Json
$decision = Get-Content -LiteralPath (Join-Path $scratch 'decision.json') -Raw | ConvertFrom-Json
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$required = @('preflight.json','candidate.json','candidate.patch','decision.json','set-variant.ps1')
$early = $decision.reason -eq 'candidate_recorded_failure'
if ($early) {
    $required += @('early-rejection.txt',("vitest/{0}-meta.json" -f $decision.failed_label),("vitest/{0}.log" -f $decision.failed_label))
} else {
    $required += @(
        'recorded-complete.txt','vitest/ab-runs.csv','vitest/ab-initial-summary.json',
        'vitest/import-mechanism.txt','vitest/import-attribution.csv',
        'vitest/import-A-ResearchProjectsShell-subtree.txt','vitest/import-B-ResearchProjectsShell-subtree.txt',
        'vitest/import-A-Inspector-subtree.txt','vitest/import-B-Inspector-subtree.txt',
        'vitest/import-A-RunDock-subtree.txt','vitest/import-B-RunDock-subtree.txt'
    )
}
$missing = @($required | Where-Object { -not (Test-Path -LiteralPath (Join-Path $scratch $_)) })
$aSnapshots = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'A') -Filter '*.svelte' -ErrorAction SilentlyContinue)
$bSnapshots = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'B') -Filter '*.svelte' -ErrorAction SilentlyContinue)
$viteRestored = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash -eq $preflight.vite_config_sha256
$warmupMetas = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'vitest') -Filter 'warmup-*-meta.json' -ErrorAction SilentlyContinue)
$recordedMetas = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'vitest') -Filter 'recorded-*-meta.json' -ErrorAction SilentlyContinue)
$initialRunsValid = if ($early) {
    $warmupMetas.Count -eq 2 -and $recordedMetas.Count -ge 1 -and $recordedMetas.Count -le 6
} else {
    $warmupMetas.Count -eq 2 -and $recordedMetas.Count -eq 6
}
$repeatMetas = @(Get-ChildItem -LiteralPath (Join-Path $scratch 'vitest') -Filter 'repeat-*-meta.json' -ErrorAction SilentlyContinue)
$repeatValid = if (-not $decision.retry_used) {
    $repeatMetas.Count -eq 0
} elseif ($decision.repeat_failed) {
    (Test-Path -LiteralPath (Join-Path $scratch 'vitest/repeat-failed.txt')) -and $repeatMetas.Count -ge 1 -and $repeatMetas.Count -le 6
} else { $repeatMetas.Count -eq 6 }
"MISSING_ARTIFACTS=$($missing.Count)"
"A_SNAPSHOTS=$($aSnapshots.Count)"
"B_SNAPSHOTS=$($bSnapshots.Count)"
"VITE_HASH_RESTORED=$viteRestored"
"INITIAL_RUN_ARTIFACTS_VALID=$initialRunsValid"
"REPEAT_ARTIFACTS_VALID=$repeatValid"
if ($missing.Count -ne 0 -or $aSnapshots.Count -ne 20 -or $bSnapshots.Count -ne 20 -or
    -not $viteRestored -or -not $initialRunsValid -or -not $repeatValid) {
    $missing
    exit 1
}
if ($decision.decision -eq 'rejected') {
    $badA = @($candidate.files | Where-Object { (Get-FileHash -Algorithm SHA256 $_.path).Hash -ne $_.a_sha256 })
    $status = @(git status --short --untracked-files=all 2>$null)
    "A_HASH_FAILURES=$($badA.Count)"
    "STATUS_COUNT=$($status.Count)"
    if ($badA.Count -ne 0 -or $status.Count -ne 0) { $status; exit 1 }
}
```

Expected: all applicable artifacts exist, both snapshot sets contain 20 files, Vite is restored, retry artifacts match the decision, and rejection leaves clean exact A.

- [ ] **Step 3: Write the evidence document from literal artifacts**

Create `docs/superpowers/verification/2026-07-15-research-projects-lucide-direct-imports.md` with `apply_patch` and these exact headings:

```markdown
# Research Projects Lucide Direct Imports Verification

## Scope and Starting State
## Environment
## Candidate and Snapshot Integrity
## Warm-Ups
## Recorded A/B Runs
## Representative Test-File Medians
## Import Mechanism and Target Trees
## Retention Criteria
## Retry Decision
## Final Decision
## Correctness Verification
## Remaining Scope and Limitations
```

For a completed protocol, populate the sections with literal values from `preflight.json`, all warm-up/recorded/repeat metadata, `ab-initial-summary.json`, `decision.json`, `import-mechanism.txt`, `import-attribution.csv`, and all six subtree files. Include the exact A/B order, inventories, per-run wall times, medians, percentage delta, three diagnostic file medians, retry policy/result, six gate outcomes, focused/check/verify outcomes that actually ran, and the remaining 36 out-of-scope root-import consumers.

For early `candidate_recorded_failure`, use the same headings but state that aggregate medians, import profiling, retry, contract, focused checks, and final verify were not reached. Record the successful warm-ups, every attempted recorded run, readable failure log, conservative rejection, and exact A restoration. Do not invent absent results.

Expected: one self-contained evidence document; no raw scratch file enters the repository.

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
$expected = 'docs/superpowers/verification/2026-07-15-research-projects-lucide-direct-imports.md'
$paths
if ($paths.Count -ne 1 -or $paths[0] -ne $expected) { exit 1 }
git add -- $expected
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git commit -m "docs: verify research project lucide imports"
```

Expected: one evidence-only documentation commit.

- [ ] **Step 5: Verify the final clean state without pushing**

Run:

```powershell
$status = @(git status --short --untracked-files=all 2>$null)
"STATUS_COUNT=$($status.Count)"
git log -6 --oneline
git show --check --stat --oneline HEAD
if ($status.Count -ne 0) { $status; exit 1 }
```

Expected: clean worktree and whitespace-clean final evidence commit. Do not push.
