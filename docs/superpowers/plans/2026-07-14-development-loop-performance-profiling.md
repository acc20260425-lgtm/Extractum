# Development Loop Performance Profiling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Measure the remaining Vitest, incremental Cargo, and Rust test-execution costs without changing committed behavior, then select one evidence-backed next action.

**Architecture:** Keep all raw logs, JSON reports, CSV tables, and Cargo timing derivatives in one unique system-temporary scratch directory. Run the three measurement families sequentially from a clean commit, use reversible hash-checked patches only for the two explicitly allowed probes, and commit one final verification document after every source/configuration byte has been restored.

**Tech Stack:** Windows PowerShell, Vitest 4.1.5 JSON/default reporters, Node.js ESM, Cargo stable timing HTML, Rust libtest, Git.

## Global Constraints

- Implement the final approved revision of `docs/superpowers/specs/2026-07-14-development-loop-performance-profiling-design.md`, committed at or after `f3e72c22`.
- Use `npm.cmd`, not plain `npm`, for npm scripts on Windows; direct `node.exe scripts/run-vitest.mjs` is allowed for reporter options not exposed as package scripts.
- Do not change committed application, test, Vitest, Cargo, build, release, Tauri, MCP, or workflow behavior.
- Do not split Vitest projects or commit `isolate: false`.
- Do not install `cargo-nextest` or any profiler, change the linker, clean Cargo artifacts, or use a noncanonical target directory.
- Use only `src-tauri/target`; do not create `codex-*` targets.
- Run Vitest, Cargo, and Rust-test measurements sequentially with no active Cargo, rustc, rust-analyzer, Tauri-dev, Vitest, or Extractum process.
- Store raw measurement artifacts only under the absolute system-temporary scratch directory recorded by this plan.
- Pass absolute system-temporary paths to Vitest `--outputFile`; reject any path inside the repository.
- Restore every temporary source/configuration patch byte-for-byte and verify its SHA-256 hash before continuing.
- Do not turn workstation timings into CI thresholds or portable performance guarantees.
- A failed test run is evidence, not a retry target: record it, exclude it from aggregates, and stop the dependent comparison.
- The execution slice creates and commits only `docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md`.
- Because code and configuration are restored byte-for-byte, do not rerun the full `npm.cmd run verify` solely for this documentation-only slice.
- Do not push after the final commit unless the user separately requests it.

---

### Task 1: Establish the Measurement Workspace and Preconditions

**Files:**
- Read: `docs/superpowers/specs/2026-07-14-development-loop-performance-profiling-design.md`
- Read: `vite.config.js`
- Read: `src-tauri/src/prompt_packs/runtime_config.rs`
- Produce outside repository: `%TEMP%/extractum-performance-profiling-*/preflight.json`

**Interfaces:**
- Produces: `%TEMP%/extractum-performance-profiling-current.txt`, containing the absolute scratch-directory path consumed by Tasks 2-5.
- Produces: `preflight.json`, containing the starting commit, tool/machine state, and source/configuration hashes.
- Preserves: a clean worktree and canonical caches.

- [ ] **Step 1: Verify the clean approved baseline**

Run:

```powershell
$status = @(git status --short --untracked-files=all 2>$null)
$branch = git branch --show-current
git merge-base --is-ancestor f3e72c22 HEAD
$approved = $LASTEXITCODE -eq 0
"STATUS_COUNT=$($status.Count)"
"BRANCH=$branch"
"APPROVED_SPEC_ANCESTOR=$approved"
git log -5 --oneline
if ($status.Count -ne 0 -or -not $approved) {
    $status
    exit 1
}
```

Expected: clean tree, current branch printed, and `APPROVED_SPEC_ANCESTOR=True`.

- [ ] **Step 2: Reject active build/test/application processes**

Run:

```powershell
$blocking = @(Get-Process cargo, rustc, rust-analyzer, extractum -ErrorAction SilentlyContinue)
$nodeBlocking = @(
    Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Name -eq 'node.exe' -and
            $_.CommandLine -match '(vitest|scripts[\\/]tauri\.mjs|tauri(?:\.js)?\s+dev)'
        }
)
$blocking | Select-Object Id, ProcessName, Path
$nodeBlocking | Select-Object ProcessId, Name, CommandLine
if ($blocking.Count -ne 0 -or $nodeBlocking.Count -ne 0) { exit 1 }
```

Expected: both lists are empty. Do not stop listed processes automatically; close or disable their owner and rerun this step.

- [ ] **Step 3: Create one unique external scratch directory**

Run:

```powershell
$scratch = Join-Path $env:TEMP ("extractum-performance-profiling-" + [guid]::NewGuid().ToString('N'))
$pointer = Join-Path $env:TEMP 'extractum-performance-profiling-current.txt'
New-Item -ItemType Directory -Path $scratch -ErrorAction Stop | Out-Null
Set-Content -LiteralPath $pointer -Value $scratch -Encoding UTF8
$resolvedScratch = (Resolve-Path -LiteralPath $scratch).Path
$repo = (Resolve-Path -LiteralPath '.').Path.TrimEnd('\')
"SCRATCH=$resolvedScratch"
if ($resolvedScratch.StartsWith($repo, [StringComparison]::OrdinalIgnoreCase)) { exit 1 }
```

Expected: a new absolute directory below the system temporary directory and outside the repository.

- [ ] **Step 4: Capture tool, machine, security, power, and hash evidence**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$os = Get-CimInstance Win32_OperatingSystem
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$power = @(& powercfg.exe /getactivescheme 2>&1) -join "`n"
$defender = if (Get-Command Get-MpComputerStatus -ErrorAction SilentlyContinue) {
    try { (Get-MpComputerStatus).RealTimeProtectionEnabled } catch { 'unavailable' }
} else { 'unavailable' }
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
    rustc = (& rustc.exe --version).Trim()
    cargo = (& cargo.exe --version).Trim()
    vite_config_sha256 = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash
    runtime_config_sha256 = (Get-FileHash -Algorithm SHA256 'src-tauri/src/prompt_packs/runtime_config.rs').Hash
}
$preflight | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Encoding UTF8
$preflight | Format-List
```

Expected: all versions and hashes are nonempty; Defender may be the literal `unavailable`. Do not change the recorded power or Defender state.

- [ ] **Step 5: Verify the installed Vitest import mechanism and nextest absence**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$help = @(& node.exe node_modules/vitest/vitest.mjs --help --expand-help 2>&1)
$helpCode = $LASTEXITCODE
$help | Set-Content -LiteralPath (Join-Path $scratch 'vitest-expanded-help.txt') -Encoding UTF8
$importFlagPresent = ($help -join "`n") -match '--experimental\.importDurations\.print <boolean\|on-warn>'
$nextest = @(& cargo.exe nextest --version 2>&1)
$nextestCode = $LASTEXITCODE
"VITEST_HELP_EXIT=$helpCode"
"IMPORT_FLAG_PRESENT=$importFlagPresent"
"NEXTEST_INSTALLED=$($nextestCode -eq 0)"
if ($helpCode -ne 0) { exit $helpCode }
if (-not $importFlagPresent) {
    Write-Host 'CLI import flag unavailable; Task 2 must use the reversible config fallback.'
}
if ($nextestCode -eq 0) {
    Write-Host 'nextest is installed but remains outside this approved execution.'
}
```

Expected on the current machine: import flag present; nextest unavailable. Neither outcome changes installed tools.

---

### Task 2: Profile Vitest Files, Imports, and the Conditional Isolation Candidate

**Files:**
- Read: all Vitest test files returned by the existing wrapper
- Conditionally modify and restore: `vite.config.js`
- Produce outside repository: `vitest/run-*.json`, logs, metadata, CSV summaries, and optional A/B reports

**Interfaces:**
- Consumes: scratch pointer and `vite_config_sha256` from Task 1.
- Produces: `vitest-file-medians.csv`, `vitest-summary.json`, `vitest-import.log`, and `vitest-ab-summary.json` or `vitest-ab-skipped.txt`.
- Preserves: complete inventory, current environments, isolation defaults, and the original `vite.config.js` hash.

- [ ] **Step 1: Run three complete JSON baselines and one import-instrumented baseline member**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
New-Item -ItemType Directory -Path $vitestDir -Force | Out-Null
$repo = (Resolve-Path -LiteralPath '.').Path.TrimEnd('\')
$help = Get-Content -LiteralPath (Join-Path $scratch 'vitest-expanded-help.txt') -Raw
$cliImport = $help -match '--experimental\.importDurations\.print <boolean\|on-warn>'

for ($run = 1; $run -le 3; $run++) {
    $report = [IO.Path]::GetFullPath((Join-Path $vitestDir "run-$run.json"))
    if ($report.StartsWith($repo, [StringComparison]::OrdinalIgnoreCase)) { exit 1 }
    if ($run -eq 1 -and $cliImport) {
        $args = @(
            'scripts/run-vitest.mjs', 'run',
            '--reporter=json', '--reporter=default',
            "--outputFile.json=$report",
            '--experimental.importDurations.print=true'
        )
    } else {
        $args = @('scripts/run-vitest.mjs', 'run', '--reporter=json', "--outputFile=$report")
    }
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $output = @(& node.exe @args 2>&1)
    $code = $LASTEXITCODE
    $watch.Stop()
    $output | Set-Content -LiteralPath (Join-Path $vitestDir "run-$run.log") -Encoding UTF8
    $json = if (Test-Path -LiteralPath $report) { Get-Content -LiteralPath $report -Raw | ConvertFrom-Json } else { $null }
    $meta = [ordered]@{
        run = $run
        exit = $code
        wall_seconds = [math]::Round($watch.Elapsed.TotalSeconds, 3)
        success = if ($json) { [bool]$json.success } else { $false }
        files = if ($json) { [int]$json.numTotalTestSuites } else { 0 }
        tests = if ($json) { [int]$json.numTotalTests } else { 0 }
        report = $report
    }
    $meta | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $vitestDir "run-$run-meta.json") -Encoding UTF8
    $meta | Format-List
    if ($code -ne 0 -or -not $json -or -not $json.success -or $json.numTotalTestSuites -le 0 -or $json.numTotalTests -le 0) { exit 1 }
}
```

Expected: three successful nonempty JSON reports with identical file/test inventories. Run 1 also contains `Import Duration Breakdown` when the CLI mechanism works.

- [ ] **Step 2: Enforce inventory equality and decide whether the import fallback is required**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
$metas = @(1..3 | ForEach-Object { Get-Content -LiteralPath (Join-Path $vitestDir "run-$_-meta.json") -Raw | ConvertFrom-Json })
$inventories = @($metas | ForEach-Object { "$($_.files)/$($_.tests)" } | Sort-Object -Unique)
$importLog = Join-Path $vitestDir 'run-1.log'
$importCaptured = Test-Path -LiteralPath $importLog -and (Get-Content -LiteralPath $importLog -Raw) -match 'Import Duration Breakdown'
"INVENTORIES=$($inventories -join ',')"
"IMPORT_BREAKDOWN_CAPTURED=$importCaptured"
if ($inventories.Count -ne 1) { exit 1 }
if (-not $importCaptured) {
    Set-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-required.txt') -Value 'required' -Encoding UTF8
}
```

Expected: one inventory value. The fallback marker exists only if the CLI path did not produce a breakdown.

- [ ] **Step 3: If required, run the reversible import-duration config fallback**

If `vitest/import-fallback-required.txt` does not exist, run the following and skip the rest of this step:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
Set-Content -LiteralPath (Join-Path $scratch 'vitest/import-mechanism.txt') -Value 'CLI mechanism succeeded' -Encoding UTF8
```

If it exists, use `apply_patch` to replace only this block in `vite.config.js`:

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
      limit: 10,
    },
  },
};
```

Then run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
$report = [IO.Path]::GetFullPath((Join-Path $vitestDir 'import-fallback.json'))
$watch = [Diagnostics.Stopwatch]::StartNew()
$output = @(& node.exe scripts/run-vitest.mjs run --reporter=json --reporter=default "--outputFile.json=$report" 2>&1)
$code = $LASTEXITCODE
$watch.Stop()
$output | Set-Content -LiteralPath (Join-Path $vitestDir 'import-fallback.log') -Encoding UTF8
$json = if (Test-Path -LiteralPath $report) { Get-Content -LiteralPath $report -Raw | ConvertFrom-Json } else { $null }
$captured = ($output -join "`n") -match 'Import Duration Breakdown'
$result = [ordered]@{
    exit = $code
    wall_seconds = [math]::Round($watch.Elapsed.TotalSeconds, 3)
    report_exists = [bool]$json
    success = if ($json) { [bool]$json.success } else { $false }
    import_captured = $captured
}
$result | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $vitestDir 'import-fallback-result.json') -Encoding UTF8
$result | Format-List
```

Regardless of the recorded exit/result, immediately use the reverse `apply_patch` to restore the original four-line block before evaluating the fallback, then run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$hash = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash
$result = Get-Content -LiteralPath (Join-Path $scratch 'vitest/import-fallback-result.json') -Raw | ConvertFrom-Json
"VITE_CONFIG_HASH_RESTORED=$($hash -eq $preflight.vite_config_sha256)"
if ($hash -ne $preflight.vite_config_sha256) { exit 1 }
$log = Get-Content -LiteralPath (Join-Path $scratch 'vitest/import-fallback.log') -Raw
$mechanism = if ($result.exit -ne 0 -or -not $result.report_exists -or -not $result.success) {
    "unavailable: config fallback run failed with exit $($result.exit)"
} elseif ($log -match 'Import Duration Breakdown') {
    'config fallback succeeded'
} else {
    'unavailable after CLI and config fallback'
}
Set-Content -LiteralPath (Join-Path $scratch 'vitest/import-mechanism.txt') -Value $mechanism -Encoding UTF8
$mechanism
```

Expected: the original hash is restored before any result is interpreted. If both mechanisms lack a breakdown or the fallback run fails, import profiling is explicitly unavailable rather than inferred from aggregate timing; the already successful three-run baseline remains valid.

- [ ] **Step 4: Derive per-file medians, percentiles, environments, and the slow tail**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
$repo = (Resolve-Path -LiteralPath '.').Path.TrimEnd('\')
function Get-Median([double[]]$values) {
    $sorted = @($values | Sort-Object)
    if ($sorted.Count % 2 -eq 1) { return [double]$sorted[[int]($sorted.Count / 2)] }
    return ([double]$sorted[$sorted.Count / 2 - 1] + [double]$sorted[$sorted.Count / 2]) / 2
}
function Get-NearestRank([double[]]$values, [double]$p) {
    $sorted = @($values | Sort-Object)
    $index = [math]::Max(0, [math]::Ceiling($p * $sorted.Count) - 1)
    return [double]$sorted[$index]
}
$rows = @()
foreach ($run in 1..3) {
    $report = Get-Content -LiteralPath (Join-Path $vitestDir "run-$run.json") -Raw | ConvertFrom-Json
    foreach ($file in $report.testResults) {
        $full = [IO.Path]::GetFullPath([string]$file.name)
        if (-not $full.StartsWith($repo, [StringComparison]::OrdinalIgnoreCase)) { exit 1 }
        $relative = $full.Substring($repo.Length).TrimStart('\','/').Replace('\','/')
        $durationMs = [double]$file.endTime - [double]$file.startTime
        $rows += [pscustomobject]@{ run=$run; path=$relative; duration_ms=$durationMs }
    }
}
$medians = @(
    $rows | Group-Object path | ForEach-Object {
        $source = Get-Content -LiteralPath $_.Name -Raw -ErrorAction Stop
        $environment = if ($source -match '@vitest-environment\s+jsdom') { 'jsdom' } else { 'node/default' }
        [pscustomobject]@{
            path = $_.Name.Replace('\','/')
            environment = $environment
            median_ms = [math]::Round((Get-Median ([double[]]$_.Group.duration_ms)), 3)
            run_1_ms = [double]($_.Group | Where-Object run -eq 1).duration_ms
            run_2_ms = [double]($_.Group | Where-Object run -eq 2).duration_ms
            run_3_ms = [double]($_.Group | Where-Object run -eq 3).duration_ms
        }
    } | Sort-Object median_ms -Descending
)
$values = [double[]]$medians.median_ms
$stats = [ordered]@{
    file_count = $medians.Count
    node_default_files = @($medians | Where-Object environment -eq 'node/default').Count
    jsdom_files = @($medians | Where-Object environment -eq 'jsdom').Count
    p50_ms = [math]::Round((Get-NearestRank $values 0.50), 3)
    p90_ms = [math]::Round((Get-NearestRank $values 0.90), 3)
    p95_ms = [math]::Round((Get-NearestRank $values 0.95), 3)
    top_10 = @($medians | Select-Object -First 10)
}
$medians | Export-Csv -LiteralPath (Join-Path $vitestDir 'vitest-file-medians.csv') -NoTypeInformation -Encoding UTF8
$stats | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $vitestDir 'vitest-summary.json') -Encoding UTF8
$stats | ConvertTo-Json -Depth 5
```

Expected: the derived file count equals the full-suite file inventory; node/default and jsdom counts sum to it; top 10 and p50/p90/p95 are printed.

- [ ] **Step 5: Build and statically review the mechanical no-isolation candidate**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$vitestDir = Join-Path $scratch 'vitest'
$summary = Get-Content -LiteralPath (Join-Path $vitestDir 'vitest-summary.json') -Raw | ConvertFrom-Json
$rows = Import-Csv -LiteralPath (Join-Path $vitestDir 'vitest-file-medians.csv')
$candidates = @($rows | Where-Object { $_.environment -eq 'node/default' -and [double]$_.median_ms -ge [double]$summary.p90_ms })
$candidates.path | Set-Content -LiteralPath (Join-Path $vitestDir 'vitest-ab-candidates.txt') -Encoding UTF8
"CANDIDATE_COUNT=$($candidates.Count)"
$patterns = 'process\.chdir|process\.env(?:\.[A-Za-z_$][\w$]*|\[[^\]]+\])\s*(?:=|\+=|-=|\*=|/=|\?\?=|\|\|=|&&=)|useFakeTimers|setSystemTime|stubGlobal|vi\.mock|jest\.mock|setInterval|setTimeout|Mutex|globalThis\.'
if ($candidates.Count -gt 0) {
    $candidatePaths = [string[]]$candidates.path
    & rg.exe -n -e $patterns -- @candidatePaths 2>&1 |
        Set-Content -LiteralPath (Join-Path $vitestDir 'vitest-ab-static-scan.txt') -Encoding UTF8
}
Get-Content -LiteralPath (Join-Path $vitestDir 'vitest-ab-candidates.txt')
Get-Content -LiteralPath (Join-Path $vitestDir 'vitest-ab-static-scan.txt') -ErrorAction SilentlyContinue
```

Expected: candidates are only node/default files at or above the measured p90. Inspect their imports and the scan results. Remove a file from the copied `vitest-ab-files.txt` scratch list if any process-global owner, mock, timer, environment mutation, module-cache assumption, or shared external resource is not demonstrably restored. If no coherent safe subset remains, create `vitest-ab-skipped.txt` with the exact reason and skip Steps 6-7.

- [ ] **Step 6: Qualify a safe candidate subset against the 10-second floor**

If Step 5 produced a safe subset, copy its final relative paths to `vitest/vitest-ab-files.txt`, one path per line, then run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'vitest'
$files = @(Get-Content -LiteralPath (Join-Path $dir 'vitest-ab-files.txt') | Where-Object { $_ })
if ($files.Count -eq 0) { exit 1 }
$times = @()
$expectedTests = $null
for ($run=1; $run -le 3; $run++) {
    $report = [IO.Path]::GetFullPath((Join-Path $dir "ab-qualify-$run.json"))
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $output = @(& node.exe scripts/run-vitest.mjs run --reporter=json "--outputFile=$report" @files 2>&1)
    $code = $LASTEXITCODE
    $watch.Stop()
    $json = if (Test-Path -LiteralPath $report) { Get-Content -LiteralPath $report -Raw | ConvertFrom-Json } else { $null }
    if ($json -and $null -eq $expectedTests) { $expectedTests = [int]$json.numTotalTests }
    $times += $watch.Elapsed.TotalSeconds
    [ordered]@{run=$run;exit=$code;wall_seconds=[math]::Round($watch.Elapsed.TotalSeconds,3);files=$json.numTotalTestSuites;tests=$json.numTotalTests;success=$json.success} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $dir "ab-qualify-$run-meta.json") -Encoding UTF8
    if ($code -ne 0 -or -not $json.success -or $json.numTotalTestSuites -ne $files.Count -or $json.numTotalTests -ne $expectedTests) { exit 1 }
}
$sorted = @($times | Sort-Object)
$median = [double]$sorted[1]
"QUALIFICATION_MEDIAN_SECONDS=$([math]::Round($median,3))"
if ($median -lt 10) {
    Set-Content -LiteralPath (Join-Path $dir 'vitest-ab-skipped.txt') -Value "Normal-isolation subset median $([math]::Round($median,3)) s is below the 10 s noise floor." -Encoding UTF8
}
```

Expected: three identical successful subset inventories. Below 10 seconds, skip Step 7.

- [ ] **Step 7: If qualified, run the alternating 3+3 isolation A/B**

Run only when `vitest-ab-skipped.txt` is absent:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'vitest'
$files = @(Get-Content -LiteralPath (Join-Path $dir 'vitest-ab-files.txt') | Where-Object { $_ })
$sequence = @('isolate','no-isolate','isolate','no-isolate','isolate','no-isolate')
$rows = @()
$expectedTests = $null
for ($index=0; $index -lt $sequence.Count; $index++) {
    $mode = $sequence[$index]
    $run = $index + 1
    $report = [IO.Path]::GetFullPath((Join-Path $dir "ab-$run-$mode.json"))
    $args = @('scripts/run-vitest.mjs','run','--reporter=json',"--outputFile=$report")
    if ($mode -eq 'no-isolate') { $args += '--no-isolate' }
    $args += $files
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $output = @(& node.exe @args 2>&1)
    $code = $LASTEXITCODE
    $watch.Stop()
    $json = if (Test-Path -LiteralPath $report) { Get-Content -LiteralPath $report -Raw | ConvertFrom-Json } else { $null }
    if ($json -and $null -eq $expectedTests) { $expectedTests = [int]$json.numTotalTests }
    if ($code -ne 0 -or -not $json.success -or $json.numTotalTestSuites -ne $files.Count -or $json.numTotalTests -ne $expectedTests) { exit 1 }
    $rows += [pscustomobject]@{run=$run;mode=$mode;wall_seconds=$watch.Elapsed.TotalSeconds;files=$json.numTotalTestSuites;tests=$json.numTotalTests}
}
function Median([double[]]$v) { $s=@($v|Sort-Object); return [double]$s[1] }
$isolate = Median ([double[]]@($rows | Where-Object mode -eq 'isolate').wall_seconds)
$noIsolate = Median ([double[]]@($rows | Where-Object mode -eq 'no-isolate').wall_seconds)
$improvement = if ($isolate -gt 0) { (($isolate - $noIsolate) / $isolate) * 100 } else { 0 }
$summary = [ordered]@{
    rows = $rows
    isolate_median_seconds = [math]::Round($isolate,3)
    no_isolate_median_seconds = [math]::Round($noIsolate,3)
    improvement_percent = [math]::Round($improvement,2)
    meets_15_percent = $improvement -ge 15
}
$summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $dir 'vitest-ab-summary.json') -Encoding UTF8
$summary | ConvertTo-Json -Depth 5
```

Expected: six successful identical inventories. `meets_15_percent` is evidence, not an automatic configuration change.

---

### Task 3: Separate Cold, No-Op, and Small-Edit Cargo Costs

**Files:**
- Read: `src-tauri/target/cargo-timings/cargo-timing-20260714T180231751Z-a54253738dfaee23.html`
- Temporarily modify and restore: `src-tauri/src/prompt_packs/runtime_config.rs`
- Produce outside repository: `cargo/*.json`, `cargo/*.csv`, logs, and one new ignored timing HTML

**Interfaces:**
- Consumes: scratch pointer and `runtime_config_sha256` from Task 1.
- Produces: cold timing summary, three no-op controls, one small-edit timing summary, and restored source hash evidence.
- Preserves: canonical target and byte-identical Rust source.

- [ ] **Step 1: Parse the existing cold Cargo timing report**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'cargo'
New-Item -ItemType Directory -Path $dir -Force | Out-Null
$path = 'src-tauri/target/cargo-timings/cargo-timing-20260714T180231751Z-a54253738dfaee23.html'
if (-not (Test-Path -LiteralPath $path)) { exit 1 }
$html = Get-Content -LiteralPath $path -Raw
$durationMatch = [regex]::Match($html, '(?m)^DURATION = ([0-9.]+);')
$unitsMatch = [regex]::Match($html, '(?s)const UNIT_DATA = (\[.*?\]);\s*const CONCURRENCY_DATA')
$concurrencyMatch = [regex]::Match($html, '(?s)const CONCURRENCY_DATA = (\[.*?\]);')
if (-not $durationMatch.Success -or -not $unitsMatch.Success -or -not $concurrencyMatch.Success) { exit 1 }
$units = @($unitsMatch.Groups[1].Value | ConvertFrom-Json)
$concurrency = @($concurrencyMatch.Groups[1].Value | ConvertFrom-Json)
$top = @($units | Sort-Object duration -Descending | Select-Object -First 10 name,version,target,mode,duration,start)
$top | Export-Csv -LiteralPath (Join-Path $dir 'cold-top-units.csv') -NoTypeInformation -Encoding UTF8
$summary = [ordered]@{
    report = $path
    shape = 'cold profile-triggered build after approved cleanup'
    duration_seconds = [double]$durationMatch.Groups[1].Value
    unit_count = $units.Count
    max_active = ($concurrency | Measure-Object active -Maximum).Maximum
    max_waiting = ($concurrency | Measure-Object waiting -Maximum).Maximum
    waiting_samples = @($concurrency | Where-Object { $_.waiting -gt 0 }).Count
    top_units = $top
}
$summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $dir 'cold-summary.json') -Encoding UTF8
$summary | ConvertTo-Json -Depth 5
```

Expected: the report parses, the duration is approximately the previously recorded 235 seconds, and top units include the cold-build leaders. Do not compare this duration directly with Task 3 Steps 2-4.

- [ ] **Step 2: Measure three no-op Cargo checks**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'cargo'
$rows = @()
for ($run=1; $run -le 3; $run++) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $output = @(& cargo.exe check --manifest-path src-tauri/Cargo.toml 2>&1)
    $code = $LASTEXITCODE
    $watch.Stop()
    $output | Set-Content -LiteralPath (Join-Path $dir "noop-$run.log") -Encoding UTF8
    $rows += [pscustomobject]@{run=$run;exit=$code;wall_seconds=[math]::Round($watch.Elapsed.TotalSeconds,3)}
    if ($code -ne 0) { exit $code }
}
$rows | Export-Csv -LiteralPath (Join-Path $dir 'noop-runs.csv') -NoTypeInformation -Encoding UTF8
$sorted = @($rows.wall_seconds | ForEach-Object { [double]$_ } | Sort-Object)
[ordered]@{ median_wall_seconds = [double]$sorted[1]; runs = $rows } |
    ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $dir 'noop-summary.json') -Encoding UTF8
$rows | Format-Table
```

Expected: all three checks pass and remain close to the existing warm no-op range; the observed values are recorded without a hard timing assertion.

- [ ] **Step 3: Apply the inert small-edit probe**

Use `apply_patch` to insert exactly this comment after the `crate::error` import in `src-tauri/src/prompt_packs/runtime_config.rs`:

```rust
// Performance profiling probe: intentionally inert and temporary.
```

Then run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$current = (Get-FileHash -Algorithm SHA256 'src-tauri/src/prompt_packs/runtime_config.rs').Hash
"SOURCE_CHANGED=$($current -ne $preflight.runtime_config_sha256)"
git diff -- src-tauri/src/prompt_packs/runtime_config.rs
if ($current -eq $preflight.runtime_config_sha256) { exit 1 }
```

Expected: the only diff is the single inert comment and the hash differs.

- [ ] **Step 4: Capture and parse the incremental Cargo timing report**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'cargo'
$before = @(Get-ChildItem -LiteralPath 'src-tauri/target/cargo-timings' -Filter 'cargo-timing-*.html' | Select-Object -ExpandProperty FullName)
$watch = [Diagnostics.Stopwatch]::StartNew()
$output = @(& cargo.exe check --manifest-path src-tauri/Cargo.toml --timings 2>&1)
$code = $LASTEXITCODE
$watch.Stop()
$output | Set-Content -LiteralPath (Join-Path $dir 'incremental.log') -Encoding UTF8
$after = @(Get-ChildItem -LiteralPath 'src-tauri/target/cargo-timings' -Filter 'cargo-timing-*.html' | Select-Object -ExpandProperty FullName)
$newReports = @($after | Where-Object { $_ -notin $before } | Sort-Object)
$result = [ordered]@{
    exit = $code
    command_wall_seconds = [math]::Round($watch.Elapsed.TotalSeconds,3)
    new_reports = $newReports
}
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $dir 'incremental-command-result.json') -Encoding UTF8
$result | Format-List
```

Expected: the command result and any new report path are recorded without exiting before restoration. Continue immediately to Step 5 even when the command failed.

- [ ] **Step 5: Reverse the probe before any Rust tests**

Use the reverse `apply_patch` to remove exactly the inert comment, then run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$current = (Get-FileHash -Algorithm SHA256 'src-tauri/src/prompt_packs/runtime_config.rs').Hash
$status = @(git status --short --untracked-files=all 2>$null)
"RUNTIME_CONFIG_HASH_RESTORED=$($current -eq $preflight.runtime_config_sha256)"
"STATUS_COUNT=$($status.Count)"
if ($current -ne $preflight.runtime_config_sha256 -or $status.Count -ne 0) {
    $status
    exit 1
}
$result = Get-Content -LiteralPath (Join-Path $scratch 'cargo/incremental-command-result.json') -Raw | ConvertFrom-Json
if ($result.exit -ne 0 -or @($result.new_reports).Count -ne 1) { exit 1 }
$newReport = [string]@($result.new_reports)[0]
$html = Get-Content -LiteralPath $newReport -Raw
$durationMatch = [regex]::Match($html, '(?m)^DURATION = ([0-9.]+);')
$unitsMatch = [regex]::Match($html, '(?s)const UNIT_DATA = (\[.*?\]);\s*const CONCURRENCY_DATA')
$concurrencyMatch = [regex]::Match($html, '(?s)const CONCURRENCY_DATA = (\[.*?\]);')
if (-not $durationMatch.Success -or -not $unitsMatch.Success -or -not $concurrencyMatch.Success) { exit 1 }
$units = @($unitsMatch.Groups[1].Value | ConvertFrom-Json)
$concurrency = @($concurrencyMatch.Groups[1].Value | ConvertFrom-Json)
$extractumUnits = @($units | Where-Object name -eq 'extractum' | Sort-Object duration -Descending)
if ($extractumUnits.Count -eq 0) { exit 1 }
$top = @($units | Sort-Object duration -Descending | Select-Object -First 10 name,version,target,mode,duration,start)
$summary = [ordered]@{
    report = $newReport.Substring((Resolve-Path '.').Path.Length).TrimStart('\').Replace('\','/')
    shape = 'small inert root-crate source edit'
    command_wall_seconds = [double]$result.command_wall_seconds
    cargo_duration_seconds = [double]$durationMatch.Groups[1].Value
    extractum_units = $extractumUnits
    max_active = ($concurrency | Measure-Object active -Maximum).Maximum
    max_waiting = ($concurrency | Measure-Object waiting -Maximum).Maximum
    waiting_samples = @($concurrency | Where-Object { $_.waiting -gt 0 }).Count
    top_units = $top
}
$summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $scratch 'cargo/incremental-summary.json') -Encoding UTF8
$summary | ConvertTo-Json -Depth 6
```

Expected: original hash and clean worktree are established before the measurement result is evaluated; then one successful ignored timestamped report parses with at least one `extractum` unit. Stop here on any mismatch.

---

### Task 4: Profile the Rust Test-Execution Floor

**Files:**
- Read: Rust test inventory under `src-tauri/src`
- Produce outside repository: `rust-tests/*.txt`, CSV inventories, run metadata, group measurements, and static scan results

**Interfaces:**
- Consumes: clean restored source from Task 3.
- Produces: three warm full-harness timings, one sequential timing, exact top-level inventory, valid group timings, optional second-level timings, and hypothesis scan evidence.
- Preserves: canonical target and unchanged test behavior.

- [ ] **Step 1: Capture the exact library-test inventory**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'rust-tests'
New-Item -ItemType Directory -Path $dir -Force | Out-Null
$output = @(& cargo.exe test --manifest-path src-tauri/Cargo.toml --lib -- --list --format terse 2>&1)
$code = $LASTEXITCODE
$output | Set-Content -LiteralPath (Join-Path $dir 'test-list-raw.txt') -Encoding UTF8
if ($code -ne 0) { exit $code }
$tests = @($output | ForEach-Object { if ($_ -match '^(.+): test$') { $matches[1] } })
if ($tests.Count -le 0 -or $tests.Count -ne @($tests | Sort-Object -Unique).Count) { exit 1 }
$tests | Set-Content -LiteralPath (Join-Path $dir 'test-names.txt') -Encoding UTF8
$inventory = @(
    $tests | Group-Object { ($_ -split '::',2)[0] } | ForEach-Object {
        [pscustomobject]@{module=$_.Name;expected_tests=$_.Count}
    } | Sort-Object module
)
$inventory | Export-Csv -LiteralPath (Join-Path $dir 'top-level-inventory.csv') -NoTypeInformation -Encoding UTF8
"LIB_TEST_COUNT=$($tests.Count)"
$inventory | Format-Table
```

Expected: a nonzero unique inventory (currently 1,125 library tests) grouped exactly once by top-level module.

- [ ] **Step 2: Measure three warm full runs and one sequential run**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'rust-tests'
$expected = @(Get-Content -LiteralPath (Join-Path $dir 'test-names.txt')).Count
$rows = @()
for ($run=1; $run -le 3; $run++) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $output = @(& cargo.exe test --manifest-path src-tauri/Cargo.toml --lib 2>&1)
    $code = $LASTEXITCODE
    $watch.Stop()
    $text = $output -join "`n"
    $match = [regex]::Match($text, 'test result: ok\. ([0-9]+) passed;.*finished in ([0-9.]+)s')
    $output | Set-Content -LiteralPath (Join-Path $dir "full-$run.log") -Encoding UTF8
    if ($code -ne 0 -or -not $match.Success -or [int]$match.Groups[1].Value -ne $expected) { exit 1 }
    $rows += [pscustomobject]@{kind='parallel';run=$run;wall_seconds=[math]::Round($watch.Elapsed.TotalSeconds,3);harness_seconds=[double]$match.Groups[2].Value;tests=[int]$match.Groups[1].Value}
}
$watch = [Diagnostics.Stopwatch]::StartNew()
$output = @(& cargo.exe test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1 2>&1)
$code = $LASTEXITCODE
$watch.Stop()
$text = $output -join "`n"
$match = [regex]::Match($text, 'test result: ok\. ([0-9]+) passed;.*finished in ([0-9.]+)s')
$output | Set-Content -LiteralPath (Join-Path $dir 'sequential.log') -Encoding UTF8
if ($code -ne 0 -or -not $match.Success -or [int]$match.Groups[1].Value -ne $expected) { exit 1 }
$rows += [pscustomobject]@{kind='sequential';run=1;wall_seconds=[math]::Round($watch.Elapsed.TotalSeconds,3);harness_seconds=[double]$match.Groups[2].Value;tests=[int]$match.Groups[1].Value}
$rows | Export-Csv -LiteralPath (Join-Path $dir 'full-and-sequential.csv') -NoTypeInformation -Encoding UTF8
$rows | Format-Table
```

Expected: all four runs pass exactly the complete library inventory. Do not compare compilation lines; compare recorded wall and harness durations.

- [ ] **Step 3: Measure exact top-level groups with collision fallback and bounded second-level partitioning**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'rust-tests'
$tests = @(Get-Content -LiteralPath (Join-Path $dir 'test-names.txt'))
$inventory = @(Import-Csv -LiteralPath (Join-Path $dir 'top-level-inventory.csv'))
$rows = @()
$fallbackRows = @()
foreach ($item in $inventory) {
    $module = [string]$item.module
    $expected = [int]$item.expected_tests
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $output = @(& cargo.exe test --manifest-path src-tauri/Cargo.toml --lib "${module}::" 2>&1)
    $code = $LASTEXITCODE
    $watch.Stop()
    $text = $output -join "`n"
    $match = [regex]::Match($text, 'test result: ok\. ([0-9]+) passed;.*finished in ([0-9.]+)s')
    $actual = if ($match.Success) { [int]$match.Groups[1].Value } else { -1 }
    $log = Join-Path $dir ("group-" + $module + '.log')
    $output | Set-Content -LiteralPath $log -Encoding UTF8
    if ($code -ne 0 -or -not $match.Success) { exit 1 }
    if ($actual -eq $expected) {
        $rows += [pscustomobject]@{module=$module;expected=$expected;actual=$actual;wall_seconds=[math]::Round($watch.Elapsed.TotalSeconds,3);harness_seconds=[double]$match.Groups[2].Value;measurement='single filtered process'}
    } else {
        $intended = @($tests | Where-Object { $_.StartsWith("${module}::", [StringComparison]::Ordinal) })
        foreach ($testName in $intended) {
            $exactOutput = @(& cargo.exe test --manifest-path src-tauri/Cargo.toml --lib $testName -- --exact 2>&1)
            $exactCode = $LASTEXITCODE
            $exactText = $exactOutput -join "`n"
            $exactMatch = [regex]::Match($exactText, 'test result: ok\. ([0-9]+) passed;.*finished in ([0-9.]+)s')
            if ($exactCode -ne 0 -or -not $exactMatch.Success -or [int]$exactMatch.Groups[1].Value -ne 1) { exit 1 }
            $fallbackRows += [pscustomobject]@{module=$module;test=$testName;harness_seconds=[double]$exactMatch.Groups[2].Value}
        }
        $rows += [pscustomobject]@{module=$module;expected=$expected;actual=$intended.Count;wall_seconds='';harness_seconds='';measurement='exact-list inventory only; process timings not comparable'}
    }
}
$rows | Export-Csv -LiteralPath (Join-Path $dir 'top-level-groups.csv') -NoTypeInformation -Encoding UTF8
$fallbackRows | Export-Csv -LiteralPath (Join-Path $dir 'exact-fallback-tests.csv') -NoTypeInformation -Encoding UTF8
if (($rows | Measure-Object expected -Sum).Sum -ne $tests.Count -or ($rows | Measure-Object actual -Sum).Sum -ne $tests.Count) { exit 1 }

$parallel = @(Import-Csv -LiteralPath (Join-Path $dir 'full-and-sequential.csv') | Where-Object kind -eq 'parallel')
$parallelMedian = [double](@($parallel.harness_seconds | ForEach-Object {[double]$_} | Sort-Object)[1])
$valid = @($rows | Where-Object { $_.measurement -eq 'single filtered process' } | Sort-Object {[double]$_.harness_seconds} -Descending)
$dominant = if ($valid.Count -gt 0 -and [double]$valid[0].harness_seconds -ge 0.25 * $parallelMedian) { $valid[0] } else { $null }
if ($dominant) {
    $prefix = "$($dominant.module)::"
    $secondInventory = @(
        $tests | Where-Object { $_.StartsWith($prefix, [StringComparison]::Ordinal) } |
            Group-Object { $parts=$_ -split '::'; if($parts.Count -ge 2){"$($parts[0])::$($parts[1])"}else{$parts[0]} } |
            ForEach-Object { [pscustomobject]@{module=$_.Name;expected_tests=$_.Count} }
    )
    $secondRows = @()
    foreach ($item in $secondInventory) {
        $watch = [Diagnostics.Stopwatch]::StartNew()
        $output = @(& cargo.exe test --manifest-path src-tauri/Cargo.toml --lib "$($item.module)::" 2>&1)
        $code = $LASTEXITCODE
        $watch.Stop()
        $match = [regex]::Match(($output -join "`n"), 'test result: ok\. ([0-9]+) passed;.*finished in ([0-9.]+)s')
        if ($code -ne 0 -or -not $match.Success) { exit 1 }
        $secondActual = [int]$match.Groups[1].Value
        if ($secondActual -eq [int]$item.expected_tests) {
            $secondRows += [pscustomobject]@{module=$item.module;expected=[int]$item.expected_tests;actual=$secondActual;wall_seconds=[math]::Round($watch.Elapsed.TotalSeconds,3);harness_seconds=[double]$match.Groups[2].Value;measurement='single filtered process'}
        } else {
            $secondPrefix = "$($item.module)::"
            $intended = @($tests | Where-Object { $_.StartsWith($secondPrefix, [StringComparison]::Ordinal) })
            foreach ($testName in $intended) {
                $exactOutput = @(& cargo.exe test --manifest-path src-tauri/Cargo.toml --lib $testName -- --exact 2>&1)
                $exactCode = $LASTEXITCODE
                $exactMatch = [regex]::Match(($exactOutput -join "`n"), 'test result: ok\. ([0-9]+) passed;.*finished in ([0-9.]+)s')
                if ($exactCode -ne 0 -or -not $exactMatch.Success -or [int]$exactMatch.Groups[1].Value -ne 1) { exit 1 }
                $fallbackRows += [pscustomobject]@{module=$item.module;test=$testName;harness_seconds=[double]$exactMatch.Groups[2].Value}
            }
            $secondRows += [pscustomobject]@{module=$item.module;expected=[int]$item.expected_tests;actual=$intended.Count;wall_seconds='';harness_seconds='';measurement='exact-list inventory only; process timings not comparable'}
        }
    }
    if (($secondRows | Measure-Object expected -Sum).Sum -ne ($secondInventory | Measure-Object expected_tests -Sum).Sum -or ($secondRows | Measure-Object actual -Sum).Sum -ne ($secondInventory | Measure-Object expected_tests -Sum).Sum) { exit 1 }
    $secondRows | Export-Csv -LiteralPath (Join-Path $dir 'second-level-groups.csv') -NoTypeInformation -Encoding UTF8
}
$fallbackRows | Export-Csv -LiteralPath (Join-Path $dir 'exact-fallback-tests.csv') -NoTypeInformation -Encoding UTF8
$rows | Format-Table
```

Expected: the top-level expected and actual sums equal the complete inventory. Substring collisions use exact-list validation and have no comparable group wall time. The operational definition of a dominant group is at least 25% of the median parallel full-harness duration; only that case creates the second-level table.

- [ ] **Step 4: Capture static hypotheses for the slowest valid groups**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$dir = Join-Path $scratch 'rust-tests'
$groups = @(Import-Csv -LiteralPath (Join-Path $dir 'top-level-groups.csv') | Where-Object { $_.measurement -eq 'single filtered process' } | Sort-Object {[double]$_.harness_seconds} -Descending | Select-Object -First 3)
$patterns = 'SqlitePool|sqlite::memory|sqlite:|NamedTempFile|tempdir|std::thread::sleep|tokio::time::sleep|sleep\(|timeout\(|test-threads|Mutex|RwLock|Semaphore|Command::|process::Command|TcpListener|TcpStream'
$scanPaths = @()
foreach ($group in $groups) {
    $dirPath = Join-Path 'src-tauri/src' $group.module
    $filePath = Join-Path 'src-tauri/src' ($group.module + '.rs')
    if (Test-Path -LiteralPath $dirPath) { $scanPaths += $dirPath }
    if (Test-Path -LiteralPath $filePath) { $scanPaths += $filePath }
}
if ($scanPaths.Count -eq 0) { exit 1 }
$uniqueScanPaths = [string[]]@($scanPaths | Sort-Object -Unique)
$scan = @(& rg.exe -n -e $patterns -- @uniqueScanPaths 2>&1)
$scanCode = $LASTEXITCODE
$scanPath = Join-Path $dir 'static-candidate-scan.txt'
if ($scan.Count -eq 0) {
    Set-Content -LiteralPath $scanPath -Value '' -Encoding UTF8
} else {
    $scan | Set-Content -LiteralPath $scanPath -Encoding UTF8
}
if ($scanCode -notin @(0,1)) { exit $scanCode }
$groups | Format-Table
Get-Content -LiteralPath (Join-Path $dir 'static-candidate-scan.txt')
```

Expected: scan evidence is tied only to the three slowest valid top-level groups. Findings remain hypotheses; do not modify tests or fixtures.

---

### Task 5: Select One Outcome and Commit the Evidence Document

**Files:**
- Create: `docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md`

**Interfaces:**
- Consumes: all Task 1-4 scratch artifacts and the decision rules from the approved spec.
- Produces: one self-contained evidence record and exactly one selected outcome.
- Preserves: byte-identical sources/configuration and a clean tree after the evidence commit.

- [ ] **Step 1: Recheck restoration, canonical target policy, and measurement completeness**

Run:

```powershell
$scratch = (Get-Content -LiteralPath (Join-Path $env:TEMP 'extractum-performance-profiling-current.txt') -Raw).Trim()
$preflight = Get-Content -LiteralPath (Join-Path $scratch 'preflight.json') -Raw | ConvertFrom-Json
$viteHash = (Get-FileHash -Algorithm SHA256 'vite.config.js').Hash
$rustHash = (Get-FileHash -Algorithm SHA256 'src-tauri/src/prompt_packs/runtime_config.rs').Hash
$status = @(git status --short --untracked-files=all 2>$null)
$codexTargets = @(Get-ChildItem -LiteralPath 'src-tauri/target' -Directory -Filter 'codex-*' -ErrorAction SilentlyContinue)
$required = @(
    'preflight.json',
    'vitest/vitest-file-medians.csv',
    'vitest/vitest-summary.json',
    'cargo/cold-summary.json',
    'cargo/noop-runs.csv',
    'cargo/noop-summary.json',
    'cargo/incremental-summary.json',
    'rust-tests/test-names.txt',
    'rust-tests/full-and-sequential.csv',
    'rust-tests/top-level-groups.csv',
    'rust-tests/static-candidate-scan.txt'
)
$missing = @($required | Where-Object { -not (Test-Path -LiteralPath (Join-Path $scratch $_)) })
"VITE_HASH_RESTORED=$($viteHash -eq $preflight.vite_config_sha256)"
"RUST_HASH_RESTORED=$($rustHash -eq $preflight.runtime_config_sha256)"
"STATUS_COUNT=$($status.Count)"
"CODEX_TARGET_COUNT=$($codexTargets.Count)"
"MISSING_ARTIFACTS=$($missing.Count)"
if ($viteHash -ne $preflight.vite_config_sha256 -or $rustHash -ne $preflight.runtime_config_sha256 -or $status.Count -ne 0 -or $codexTargets.Count -ne 0 -or $missing.Count -ne 0) {
    $status
    $missing
    exit 1
}
```

Expected: both hashes restored, clean tree, no `codex-*` target, and all mandatory artifacts present.

- [ ] **Step 2: Apply the decision rules to the measured evidence**

Read the scratch summaries and choose exactly one primary outcome:

1. Choose **Vitest follow-up** when either `vitest-ab-summary.json` has `meets_15_percent=true`, or the captured import breakdown identifies one concrete shared expensive import across multiple files that remain in the three-run slow tail. An import-only outcome recommends a validation slice and does not claim saved seconds.
2. Otherwise choose **Rust-test follow-up** when a valid top- or second-level group dominates the harness evidence and `static-candidate-scan.txt` contains a concrete SQLite, real-wait/timeout, serialization, or shared-resource hypothesis belonging to that group.
3. Otherwise choose **Incremental Cargo follow-up** when `incremental-summary.json` shows a material root-crate cost absent from the three no-op controls. Do not use `cold-summary.json` alone for this choice.
4. Otherwise choose **Stop optimizing** and retain the existing focused daily commands.

When multiple outcomes qualify, choose the one with the clearest causal hypothesis and cheapest safe validation; list others as secondary candidates without giving them primary status.

Expected: one primary outcome and an explicit explanation of why the other three were not selected.

- [ ] **Step 3: Write the evidence document with literal observations**

Create `docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md` using `apply_patch`. Include these exact sections, populated with literal values from the scratch artifacts:

```markdown
# Development Loop Performance Profiling Verification

## Scope and Starting State
## Environment
## Vitest Baseline Distribution
## Vitest Import Evidence
## Conditional Isolation Experiment
## Cargo Cold Report
## Cargo No-Op Control
## Cargo Small-Edit Report
## Rust Full and Sequential Harness
## Rust Module Groups
## Rust Static Hypotheses
## Limitations
## Selected Outcome
## Rejected Outcomes
```

Required content:

- starting commit, tool/machine versions, active Windows power scheme, and
  Defender real-time-protection observation;
- exact successful run durations and inventories, not only medians;
- Vitest p50/p90/p95, top ten file medians, and node/default versus jsdom counts;
- the exact import mechanism used, its top import findings, or `unavailable` with both failed mechanisms;
- A/B qualification, six run results and percentage, or the exact skip reason;
- cold/no-op/small-edit Cargo values in separate tables, including report paths, dominant units, active/waiting observations, and `extractum` duration;
- all three parallel Rust runs, the sequential run, complete inventory, top-level groups, optional second-level groups, and any exact-filter fallback;
- static findings labeled as hypotheses rather than causes;
- one primary outcome, secondary candidates if any, and reasons the other outcomes were rejected;
- explicit statements that no behavior changed, no tool was installed, hashes were restored, and machine timings are not portable thresholds.

Expected: a self-contained report; no raw logs, JSON blobs, CSV files, or Cargo HTML are copied into docs.

- [ ] **Step 4: Review the only intended repository diff**

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
if ($paths.Count -ne 1 -or $paths[0] -ne 'docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md') { exit 1 }
git diff -- docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md
```

Expected: only the new verification document, with no whitespace errors and no source/configuration diff.

- [ ] **Step 5: Commit the evidence**

Run:

```powershell
git add -- docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md
git diff --cached --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git commit -m "docs: profile development loop performance"
```

Expected: one documentation commit. Raw scratch and canonical target artifacts remain unstaged.

- [ ] **Step 6: Verify final state without pushing**

Run:

```powershell
$status = @(git status --short --untracked-files=all 2>$null)
"STATUS_COUNT=$($status.Count)"
git log -3 --oneline
git show --check --stat --oneline HEAD
if ($status.Count -ne 0) { $status; exit 1 }
```

Expected: clean tree and a whitespace-clean evidence commit. Do not push.
