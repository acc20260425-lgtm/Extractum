# Telegram Phase 8C Crate Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the retained 19-file Telegram implementation into the seventh Rust workspace package, preserve all 736 logical library tests, and retain one atomic implementation commit plus one terminal evidence commit.

**Architecture:** Move the frozen 8B tree behind a behavior-free private app facade, transfer its direct low-level dependencies to `extractum-telegram`, and expose exactly two consumer-test fixtures through a non-default `app-test-support` feature. Observe the cross-package fixture failure as one uncommitted compile-time RED, then make the same atomic working change GREEN before retaining `EXTRACTION_COMMIT`; run live MCP and startup evidence only after that commit.

**Tech Stack:** Rust 2021, Cargo resolver v2, Tauri 2, Tokio, Grammers, TypeScript, Vitest, PowerShell, Git.

**Status:** Ready for owner review; implementation not authorized.

## Global Constraints

- The frozen 8B source authority is commit `c4c446b1169733d8623f84bbda5e028c2e7fa365`; later documentation commits do not replace it.
- Implementation starts only from a clean, committed plan revision. Record that clean HEAD as `BASE_COMMIT`; it is evidence and a rollback anchor, not an implementation checkpoint.
- Phase 8C has exactly one implementation commit, named `EXTRACTION_COMMIT`. Tasks 2 through 6 remain one uncommitted change; do not retain a preparation, RED, manifest, contract, or test-only commit.
- Move the exact 19 relative source paths listed below. Seventeen destination blobs must equal their frozen source blobs. `lib.rs` and `takeout/mod.rs` may contain only the approved eight line substitutions.
- Do not regenerate or edit `src/lib/telegram-8b-staging-sha256.json`, `src/lib/telegram-8b-test-identities.json`, `src/lib/telegram-8b-symbol-map.json`, or `src/lib/telegram-grammers-feature-baseline.json`.
- The producer owns only the empty non-default `app-test-support` feature. Its two definitions and two crate-root re-exports use only `#[cfg(feature = "app-test-support")]`; no producer test consumes either fixture.
- The app's normal dependency stays feature-free. Only its dev-dependency enables `app-test-support`; the private app facade re-exports the fixtures only under `#[cfg(test)]`.
- Preserve `#[path = "telegram_impl/lib.rs"] mod telegram_impl;` and every existing `crate::telegram_impl::` consumer path byte-for-byte. Do not edit any app Rust consumer outside the new facade.
- Keep `[workspace.dependencies]` byte-identical. Do not change a Grammers revision, default-feature policy, explicit feature, or resolved feature closure.
- Remove exactly these six direct app roots: `grammers-client`, `grammers-session`, `grammers-mtsender`, `grammers-tl-types`, `chacha20poly1305`, and `rand_core`. Keep `base64` and `secrecy` in the app because app-owned code still uses them.
- Do not run `cargo fmt` over the moved tree. `npm.cmd run check:rustfmt` and the final verifier are checks; a formatting failure stops the slice because formatting the frozen files would violate content authority.
- Do not run `src/lib/telegram-crate-boundary-contract.test.ts` or another topology contract during the transient RED. Update the standing contract once for the final layout and run it first in final GREEN.
- Use canonical shared `src-tauri/target`. Do not set a slice-specific target directory.
- The terminal order is fixed: final uncommitted `npm.cmd run verify` -> release build with explicit host target -> `EXTRACTION_COMMIT` -> one live MCP smoke -> self-managed startup smoke -> docs-only terminal evidence commit.
- The 8C precedence section leaves the parent's retained full-gate/timing clauses in force. Immediately before the fixed terminal suffix, run standalone rustfmt, locked metadata, one ordinary workspace check, and one workspace test; record only the standalone workspace-check duration. The subsequent final verifier's internal workspace check is correctness-only and its duration is ignored.
- A failed post-commit MCP or startup smoke leaves `EXTRACTION_COMMIT` intact and Phase 8C pending. It does not authorize rewriting the extraction.
- Stop and amend the approved design if implementation requires another moved-source edit, dependency capability, public name, fixture, consumer rewrite, wire change, transaction-owner change, or reverse dependency.

## File and Ownership Map

### Exact 19-file move

Move each source from `src-tauri/src/telegram_impl/<relative-path>` to `src-tauri/crates/extractum-telegram/src/<relative-path>`:

```text
dto.rs
error.rs
lib.rs
live/avatar.rs
live/messages.rs
live/mod.rs
live/peer.rs
live/topics.rs
media.rs
runtime.rs
session.rs
takeout/export_dc.rs
takeout/forum_topics.rs
takeout/mod.rs
takeout/operations.rs
takeout/pagination.rs
takeout/raw_parse.rs
takeout/transport.rs
takeout/types.rs
```

The vacated `src-tauri/src/telegram_impl/lib.rs` becomes new app-owned facade content. The other 18 old paths disappear.

### Create

- `src-tauri/crates/extractum-telegram/Cargo.toml` — exact package, feature, production dependency, and test-runtime contract.
- `src-tauri/crates/extractum-telegram/src/**` — the 19 moved files.
- `docs/superpowers/verification/2026-08-01-extractum-telegram-8c-extraction.md` — terminal evidence written only after MCP and startup pass.

### Modify in `EXTRACTION_COMMIT`

- `src-tauri/Cargo.toml` — seventh member, normal/dev package edges, six direct-root removals.
- `src-tauri/Cargo.lock` — Cargo-generated path-package ownership change only.
- `src-tauri/src/telegram_impl/lib.rs` — explicit behavior-free compatibility facade.
- `src-tauri/crates/extractum-telegram/src/lib.rs` — four approved fixture-boundary line substitutions.
- `src-tauri/crates/extractum-telegram/src/takeout/mod.rs` — four approved fixture-boundary line substitutions.
- `scripts/telegram-grammers-feature-baseline.mjs` — resolve the same four Grammers packages from their new direct owner while rendering the frozen JSON byte-for-byte.
- `src/lib/telegram-crate-boundary-contract.test.ts` — physical-state lifecycle, exact final manifest/facade/API, metadata graph, feature closure, fixture allowlist, and final identity ownership.
- `src/lib/crate-extraction-shell-cap-contract.test.ts` — admit the approved bounded 8C authority and both pending/terminal status states before terminal docs change.
- `src/lib/rust-workspace-core-contract.test.ts` — include the seventh member when its manifest exists.
- `src/lib/gemini-browser-crate-boundary-contract.test.ts` — include the seventh member without weakening Gemini ownership assertions.
- `src/lib/llm-crate-boundary-contract.test.ts` — include the seventh member while retaining Telegram as a forbidden LLM dependency.
- `src/lib/prompt-pack-crate-boundary-contract.test.ts` — include the seventh member while retaining Telegram as a foreign package.
- `src/lib/analysis-crate-boundary-contract.test.ts` — include the seventh member without changing analysis ownership.
- `src/lib/analysis-application-contract.test.ts` — treat final Telegram crate leaves as the retained Telegram boundary when selecting the already-frozen app SQL fingerprints.

### Modify only in the terminal docs commit

- `docs/superpowers/specs/2026-08-01-telegram-8c-extraction-design.md` — terminal implemented/retained status and verification link.
- `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md` — terminal Phase 8 disposition and verification link; retain historical 8A/8B text.
- `docs/superpowers/specs/2026-07-17-crate-roadmap.md` — `done: retained`, final Phase 8 summary, verification link, and the one ordinary workspace-check duration.
- `docs/superpowers/verification/2026-08-01-extractum-telegram-8c-extraction.md` — complete evidence.
- `docs/project.md` — current seven-package architecture and the Telegram crate/facade ownership split.
- `docs/value-registry.md` — register the already-closed `8c-extracted` lifecycle selector and `done: retained` status input when they become the active retained state.

### Explicitly unchanged

- All app Rust consumers outside `src-tauri/src/telegram_impl/lib.rs`.
- `src-tauri/src/lib.rs`, including `#[path = "telegram_impl/lib.rs"] mod telegram_impl;`.
- `[workspace.dependencies]` in `src-tauri/Cargo.toml`.
- Frontend source, Tauri commands/events/permissions, migrations, SQLite ownership, secret storage, and all four immutable 8B artifacts.
- `docs/superpowers/verification/README.md`; retained precedent reserves that index for active/reusable notes, not one-off completion records.

## Rust Verification Loops

Affected packages are `extractum-telegram` and its immediate consumer `extractum`.

- Compile-time RED: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib --no-run` must fail at the facade with primary `E0432` naming both fixtures before a test executable exists.
- Narrow consumer GREEN: run each of the four exact fixture-consuming app tests and require `running 1 test`.
- Canonical producer feature-off evidence: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`.
- Producer-isolated focused check: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`; factually feature-off, but not the canonical feature-off proof.
- Producer-isolated checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
- Normal consumer check: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --lib`.
- Feature-on consumer focused check: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.
- Feature-on consumer checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.
- End-of-slice gates on the complete, staged, still-uncommitted extraction tree: retained standalone rustfmt, locked metadata, workspace check/test, then one final `npm.cmd run verify`. Record timing only from the standalone workspace check; the verifier's internal repeat is correctness-only.

`cargo check/test -p extractum --all-targets` are feature-on evidence under resolver v2. Neither may be described as feature-off proof. A filtered GREEN run that reports zero tests is not evidence.

## PowerShell Invocation Contract

PowerShell tool calls are stateless. Task 1 Step 1 creates a run-specific `$EvidenceRoot`, `helpers.ps1`, `state.ps1`, `state.clixml`, and one repository-bound bootstrap file under the system temporary directory. No workflow state file is created in the repository.

Every `powershell` fence after Task 1 Step 1 is a command body. Execute this exact prelude in the same fresh PowerShell call immediately before the shown body:

```powershell
$Phase8CRepoRoot = [IO.Path]::GetFullPath((git rev-parse --show-toplevel).Trim())
if ($LASTEXITCODE -ne 0) { throw 'Could not resolve Phase 8C repository root' }
$Phase8CRepoToken =
    $Phase8CRepoRoot.ToLowerInvariant() -replace '[^a-z0-9.-]', '_'
$Phase8CBootstrapPath = Join-Path `
    ([IO.Path]::GetTempPath()) `
    "extractum-telegram-8c-$Phase8CRepoToken.bootstrap.ps1"
. $Phase8CBootstrapPath
```

The bootstrap rejects a different repository root, dot-sources the shared helpers, imports the latest CLIXML state into the fresh call, and reconstructs PID lists as `List[int]`. If the bootstrap already exists when starting Task 1 Step 1, do not overwrite it: resume that run or inspect and manually retire its temporary state first. Each named persistence checkpoint below must complete before starting a later fence or dispatching a fresh subagent.

---

### Task 1: Freeze the Clean Baseline and Exact Test Sets

**Files:**
- Read: `docs/superpowers/specs/2026-08-01-telegram-8c-extraction-design.md`
- Read: `src/lib/telegram-8b-staging-sha256.json`
- Read: `src/lib/telegram-8b-test-identities.json`
- Read: `src/lib/telegram-8b-symbol-map.json`
- Read: `src/lib/telegram-grammers-feature-baseline.json`
- Read: `src-tauri/Cargo.toml`
- Read: `src-tauri/Cargo.lock`
- Create outside the repository: one run-specific temporary evidence directory

**Interfaces:**
- Consumes: frozen 8B commit `c4c446b1169733d8623f84bbda5e028c2e7fa365` and the six approved SHA-256 values.
- Produces: `$BASE_COMMIT`, `$EvidenceRoot`, `$BASE_TESTS`, `$EXPECTED_APP_TESTS`, `$EXPECTED_CRATE_TESTS`, exact 19-path blob equality, and baseline metadata/test-list captures.

- [ ] **Step 1: Initialize recoverable out-of-repository state and shared helpers**

This is the only PowerShell fence that does not use the common prelude. It fails if another Phase 8C bootstrap already exists for this repository.

````powershell
$ErrorActionPreference = 'Stop'
$Phase8CRepoRoot = [IO.Path]::GetFullPath((git rev-parse --show-toplevel).Trim())
if ($LASTEXITCODE -ne 0) { throw 'Could not resolve Phase 8C repository root' }
$Phase8CRepoToken =
    $Phase8CRepoRoot.ToLowerInvariant() -replace '[^a-z0-9.-]', '_'
$Phase8CBootstrapPath = Join-Path `
    ([IO.Path]::GetTempPath()) `
    "extractum-telegram-8c-$Phase8CRepoToken.bootstrap.ps1"
if (Test-Path -LiteralPath $Phase8CBootstrapPath) {
    throw "Existing Phase 8C bootstrap must be resumed or manually retired: $Phase8CBootstrapPath"
}

$dirty = @(git status --porcelain=v1 --untracked-files=all)
if ($LASTEXITCODE -ne 0) { throw 'git status failed' }
if ($dirty.Count -ne 0) { $dirty; throw 'Phase 8C requires a clean working tree' }
$BASE_COMMIT = (git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $BASE_COMMIT -notmatch '^[a-f0-9]{40}$') {
    throw 'Could not resolve clean BASE_COMMIT'
}
$Frozen8B = 'c4c446b1169733d8623f84bbda5e028c2e7fa365'
git merge-base --is-ancestor $Frozen8B $BASE_COMMIT
if ($LASTEXITCODE -ne 0) { throw 'Frozen 8B authority is not an ancestor of BASE_COMMIT' }

$EvidenceRoot = Join-Path `
    ([IO.Path]::GetTempPath()) `
    ("extractum-telegram-8c-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $EvidenceRoot | Out-Null

$helpersSource = @'
function Invoke-CapturedNative {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )
    $previousErrorActionPreference = $ErrorActionPreference
    $previousLastExitCode = $global:LASTEXITCODE
    $output = @()
    $exitCodeRef = [ref][int]::MinValue
    try {
        $ErrorActionPreference = 'Continue'
        $global:LASTEXITCODE = [int]::MinValue
        $output = @(& {
            & $Command
            $exitCodeRef.Value = $LASTEXITCODE
        } 2>&1)
    }
    finally {
        $global:LASTEXITCODE = $previousLastExitCode
        $ErrorActionPreference = $previousErrorActionPreference
    }
    if ($exitCodeRef.Value -eq [int]::MinValue) {
        throw "$Label did not execute a native command"
    }
    [pscustomobject]@{
        ExitCode = [int]$exitCodeRef.Value
        Text = ($output | Out-String)
    }
}

function Invoke-CheckedNative {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )
    $capture = Invoke-CapturedNative -Label $Label -Command $Command
    Write-Host -NoNewline $capture.Text
    if ($capture.ExitCode -ne 0) {
        throw "$Label failed with exit code $($capture.ExitCode)"
    }
    return $capture
}

function Get-RustTestIds {
    param([Parameter(Mandatory = $true)][string]$Package)
    $capture = Invoke-CapturedNative -Label "Cargo test inventory $Package" -Command {
        cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib -- --list
    }
    if ($capture.ExitCode -ne 0) {
        $capture.Text
        throw "Cargo test inventory failed for $Package"
    }
    $ids = @(
        $capture.Text -split "\r?\n" |
            ForEach-Object { if ($_ -match '^([^ ]+): test$') { $Matches[1] } }
    )
    if ($ids.Count -eq 0) { throw "Cargo test inventory is empty for $Package" }
    if (@($ids | Select-Object -Unique).Count -ne $ids.Count) {
        throw "Cargo test inventory contains duplicate IDs for $Package"
    }
    @($ids | Sort-Object)
}

function Assert-ExactStringSet {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string[]]$Expected,
        [Parameter(Mandatory = $true)][string[]]$Actual
    )
    $expectedSorted = @($Expected | Sort-Object)
    $actualSorted = @($Actual | Sort-Object)
    if (@($expectedSorted | Select-Object -Unique).Count -ne $expectedSorted.Count) {
        throw "$Label expected set contains duplicates"
    }
    if (@($actualSorted | Select-Object -Unique).Count -ne $actualSorted.Count) {
        throw "$Label actual set contains duplicates"
    }
    if (($expectedSorted -join "`n") -ne ($actualSorted -join "`n")) {
        Compare-Object $expectedSorted $actualSorted
        throw "$Label exact set drifted"
    }
}

function Invoke-ExactRustTest {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$TestName
    )
    $capture = Invoke-CapturedNative -Label "exact Rust test $Package::$TestName" -Command {
        cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib $TestName -- --exact
    }
    Write-Host -NoNewline $capture.Text
    if ($capture.ExitCode -ne 0) { throw "Exact Rust test failed: $Package::$TestName" }
    if ($capture.Text -notmatch '(?m)^[ \t]*running 1 test[ \t]*\r?$') {
        throw "Exact Rust test was empty or ambiguous: $Package::$TestName"
    }
    return $capture
}

function Read-EvidenceText {
    param([Parameter(Mandatory = $true)][string]$Name)
    $evidencePath = Join-Path $EvidenceRoot $Name
    if (-not (Test-Path -LiteralPath $evidencePath -PathType Leaf)) {
        throw "Missing retained evidence capture: $Name"
    }
    return (Get-Content -Raw -LiteralPath $evidencePath).TrimEnd()
}

function Get-TomlSectionFromGitText {
    param([string]$Text, [string]$Heading)
    $normalized = $Text -replace "\r\n?", "`n"
    $match = [regex]::Match(
        $normalized,
        "(?ms)^\[$([regex]::Escape($Heading))\]`n.*?(?=^\[|\z)"
    )
    if (-not $match.Success) { throw "Missing TOML section [$Heading]" }
    return $match.Value
}

function Assert-ExactDiffBody {
    param(
        [string]$Label,
        [string]$Diff,
        [string[]]$ExpectedRemoved,
        [string[]]$ExpectedAdded,
        [string[]]$ExpectedHunkPrefixes
    )
    $lines = $Diff -split "\r?\n"
    $removed = @($lines | Where-Object { $_.StartsWith('-') -and -not $_.StartsWith('---') } | ForEach-Object { $_.Substring(1) })
    $added = @($lines | Where-Object { $_.StartsWith('+') -and -not $_.StartsWith('+++') } | ForEach-Object { $_.Substring(1) })
    $hunks = @($lines | Where-Object { $_.StartsWith('@@ ') })
    if (($removed -join "`n") -cne ($ExpectedRemoved -join "`n")) {
        $removed
        throw "$Label removed-line sequence drifted"
    }
    if (($added -join "`n") -cne ($ExpectedAdded -join "`n")) {
        $added
        throw "$Label added-line sequence drifted"
    }
    if ($hunks.Count -ne $ExpectedHunkPrefixes.Count) {
        throw "$Label hunk count drifted"
    }
    for ($index = 0; $index -lt $hunks.Count; $index++) {
        if (-not $hunks[$index].StartsWith($ExpectedHunkPrefixes[$index])) {
            throw "$Label hunk position drifted: $($hunks[$index])"
        }
    }
}

function Get-ProcessTreeIds {
    param([Parameter(Mandatory = $true)][int]$RootProcessId)
    $rows = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId)
    $queue = [Collections.Generic.Queue[int]]::new()
    $queue.Enqueue($RootProcessId)
    $result = [Collections.Generic.List[int]]::new()
    while ($queue.Count -ne 0) {
        $parentId = $queue.Dequeue()
        if (-not $result.Contains($parentId)) { $result.Add($parentId) }
        foreach ($child in @($rows | Where-Object { $_.ParentProcessId -eq $parentId })) {
            if (-not $result.Contains([int]$child.ProcessId)) {
                $queue.Enqueue([int]$child.ProcessId)
            }
        }
    }
    return @($result)
}

function Stop-OwnedDevTrees {
    $seenOwnedPids = [Collections.Generic.HashSet[int]]::new()
    $ownedDevPids = @(
        foreach ($ownedRootPid in @($ownedRootPids)) {
            foreach ($treePid in @(Get-ProcessTreeIds -RootProcessId $ownedRootPid)) {
                if ($seenOwnedPids.Add($treePid)) { $treePid }
            }
        }
    )
    [array]::Reverse($ownedDevPids)
    foreach ($ownedPid in $ownedDevPids) {
        if ($ownedPid -eq $PID) { throw 'Owned dev tree unexpectedly contains the evidence shell' }
        $ownedProcess = Get-Process -Id $ownedPid -ErrorAction SilentlyContinue
        if ($null -ne $ownedProcess) { Stop-Process -Id $ownedPid -Force }
    }
    $devCleanupDeadline = [DateTime]::UtcNow.AddSeconds(15)
    while (
        (@($ownedDevPids | Where-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue }).Count -ne 0) -and
        ([DateTime]::UtcNow -lt $devCleanupDeadline)
    ) {
        Start-Sleep -Milliseconds 200
    }
    $ownedResidue = @($ownedDevPids | Where-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
    if ($ownedResidue.Count -ne 0) {
        throw "Owned MCP dev process residue remains: $($ownedResidue -join ', ')"
    }
    $ownedRootPids.Clear()
}

function Save-Phase8CState {
    param([Parameter(Mandatory = $true)][hashtable]$Values)
    $statePath = Join-Path $EvidenceRoot 'state.clixml'
    $state = if (Test-Path -LiteralPath $statePath) {
        [hashtable](Import-Clixml -LiteralPath $statePath)
    } else {
        @{}
    }
    foreach ($name in $Values.Keys) { $state[$name] = $Values[$name] }
    $temporaryStatePath = "$statePath.tmp"
    $state | Export-Clixml -LiteralPath $temporaryStatePath
    Move-Item -LiteralPath $temporaryStatePath -Destination $statePath -Force
}
'@

$stateLoaderSource = @'
$statePath = Join-Path $EvidenceRoot 'state.clixml'
if (-not (Test-Path -LiteralPath $statePath -PathType Leaf)) {
    throw "Missing Phase 8C state: $statePath"
}
$phase8CState = [hashtable](Import-Clixml -LiteralPath $statePath)
foreach ($entry in $phase8CState.GetEnumerator()) {
    if ($entry.Key -in @('launchedRootPids', 'ownedRootPids')) {
        $pidList = [Collections.Generic.List[int]]::new()
        foreach ($processId in @($entry.Value)) { [void]$pidList.Add([int]$processId) }
        Set-Variable -Name $entry.Key -Value $pidList
    } else {
        Set-Variable -Name $entry.Key -Value $entry.Value
    }
}
'@

$utf8NoBom = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText(
    (Join-Path $EvidenceRoot 'helpers.ps1'),
    (($helpersSource -replace "\r\n?", "`n").TrimEnd("`n") + "`n"),
    $utf8NoBom
)
[IO.File]::WriteAllText(
    (Join-Path $EvidenceRoot 'state.ps1'),
    (($stateLoaderSource -replace "\r\n?", "`n").TrimEnd("`n") + "`n"),
    $utf8NoBom
)
@{
    RepositoryRoot = $Phase8CRepoRoot
    BASE_COMMIT = $BASE_COMMIT
    Frozen8B = $Frozen8B
    EvidenceRoot = $EvidenceRoot
    mcpCleanupComplete = $true
} | Export-Clixml -LiteralPath (Join-Path $EvidenceRoot 'state.clixml')

$escapedRepoRoot = $Phase8CRepoRoot.Replace("'", "''")
$escapedEvidenceRoot = $EvidenceRoot.Replace("'", "''")
$bootstrapSource = @"
`$ErrorActionPreference = 'Stop'
`$phase8CCurrentRoot = [IO.Path]::GetFullPath((git rev-parse --show-toplevel).Trim())
if (`$LASTEXITCODE -ne 0) { throw 'Could not resolve Phase 8C repository root' }
if (-not [String]::Equals(`$phase8CCurrentRoot, '$escapedRepoRoot', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Phase 8C bootstrap belongs to a different repository root'
}
`$EvidenceRoot = '$escapedEvidenceRoot'
. (Join-Path `$EvidenceRoot 'helpers.ps1')
. (Join-Path `$EvidenceRoot 'state.ps1')
"@
[IO.File]::WriteAllText(
    $Phase8CBootstrapPath,
    (($bootstrapSource -replace "\r\n?", "`n").TrimEnd("`n") + "`n"),
    $utf8NoBom
)
. $Phase8CBootstrapPath
"BASE_COMMIT=$BASE_COMMIT EvidenceRoot=$EvidenceRoot Bootstrap=$Phase8CBootstrapPath"
````

Expected: one clean 40-character `BASE_COMMIT`; a random evidence directory plus helper/state files outside the repository; one repository-bound bootstrap; no repository change.

- [ ] **Step 2: Prove a fresh PowerShell call rehydrates the baseline state**

Run this body with the common prelude:

```powershell
foreach ($helper in @(
    'Invoke-CapturedNative',
    'Invoke-CheckedNative',
    'Get-RustTestIds',
    'Assert-ExactStringSet',
    'Invoke-ExactRustTest',
    'Read-EvidenceText',
    'Get-TomlSectionFromGitText',
    'Assert-ExactDiffBody',
    'Get-ProcessTreeIds',
    'Stop-OwnedDevTrees',
    'Save-Phase8CState'
)) {
    if ($null -eq (Get-Command $helper -CommandType Function -ErrorAction SilentlyContinue)) {
        throw "Fresh Phase 8C shell omitted helper: $helper"
    }
}
if ($BASE_COMMIT -notmatch '^[a-f0-9]{40}$') { throw 'Fresh shell omitted BASE_COMMIT' }
if ($Frozen8B -ne 'c4c446b1169733d8623f84bbda5e028c2e7fa365') {
    throw 'Fresh shell omitted frozen 8B authority'
}
if (-not (Test-Path -LiteralPath $EvidenceRoot -PathType Container)) {
    throw 'Fresh shell omitted EvidenceRoot'
}
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) {
    throw 'State initialization changed the repository'
}
```

Expected: a genuinely fresh PowerShell process recovers all shared helpers and baseline variables from temporary durable state.

- [ ] **Step 3: Run all four retained authority checks while their 8B physical owner still exists**

```powershell
$authorityChecks = [ordered]@{
    'scripts/telegram-staging-sha256.mjs' = '--check'
    'scripts/telegram-grammers-feature-baseline.mjs' = '--check'
    'scripts/telegram-8b-test-identities.mjs' = '--check'
    'scripts/telegram-8b-symbol-map.mjs' = '--check'
}
foreach ($authorityCheck in $authorityChecks.GetEnumerator()) {
    Invoke-CheckedNative "retained authority $($authorityCheck.Key)" {
        node $authorityCheck.Key $authorityCheck.Value
    } | Out-Null
}
```

Expected: all four checks exit 0. This is the only Phase 8C execution of `telegram-staging-sha256.mjs --check`; after the move its retained 8B app-owned layout is intentionally no longer current. Never run any of these scripts with `--write` in Phase 8C.

- [ ] **Step 4: Verify all six frozen SHA-256 authorities before the first source edit**

```powershell
$expectedHashes = [ordered]@{
    'src/lib/telegram-grammers-feature-baseline.json' = '774e2b979d5cdc8185a85488c965548cf09cdf1ef0ab4b9ecad58246283cf5b3'
    'src/lib/telegram-8b-test-identities.json' = '507f09f4fab76bee4360185eca3fbef17fb1563e784f7654bebb430cf7f08a95'
    'src/lib/telegram-8b-symbol-map.json' = 'f978e80cd58303fd9cd6402ba17deef1df817d22fdbf821830e9e0a5968c13b3'
    'src/lib/telegram-8b-staging-sha256.json' = '12e99b10aaaccc471ae4c950b4a3ea0331ae68db45618823ea2aa58bae29d1a9'
    'src-tauri/Cargo.toml' = 'ee323d7b613573918d4ad3777b238bc7e107d049588ddcfa0959dacfd1e2cf69'
    'src-tauri/Cargo.lock' = '720e38ea632d7b932b2a23d1481528845ec9304376035b1c851c546ea402e43c'
}
foreach ($entry in $expectedHashes.GetEnumerator()) {
    $actual = (Get-FileHash -LiteralPath $entry.Key -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $entry.Value) {
        throw "Frozen SHA-256 drift for $($entry.Key): $actual"
    }
    "$($entry.Key) $actual"
}
Save-Phase8CState -Values @{ expectedHashes = $expectedHashes }
```

Expected: six exact matches. Any mismatch stops planning execution; do not normalize it inside 8C.

- [ ] **Step 5: Prove every `BASE_COMMIT` staged-source blob equals frozen 8B and record the exact path list**

```powershell
$stagingAuthority = Get-Content -Raw -LiteralPath `
    'src/lib/telegram-8b-staging-sha256.json' | ConvertFrom-Json
$relativePaths = @($stagingAuthority.files.path)
if ($stagingAuthority.schemaVersion -ne 1) { throw 'Unexpected staging schema' }
if ($stagingAuthority.root -ne 'src-tauri/src/telegram_impl') { throw 'Unexpected staging root' }
if ($relativePaths.Count -ne 19) { throw "Expected 19 staged paths, found $($relativePaths.Count)" }
if (($relativePaths -join "`n") -ne (($relativePaths | Sort-Object) -join "`n")) {
    throw 'Staging paths are not sorted'
}

$implementationFixedPaths = @(
    'scripts/telegram-grammers-feature-baseline.mjs',
    'src-tauri/Cargo.toml',
    'src-tauri/Cargo.lock',
    'src-tauri/crates/extractum-telegram/Cargo.toml',
    'src/lib/telegram-crate-boundary-contract.test.ts',
    'src/lib/crate-extraction-shell-cap-contract.test.ts',
    'src/lib/rust-workspace-core-contract.test.ts',
    'src/lib/gemini-browser-crate-boundary-contract.test.ts',
    'src/lib/llm-crate-boundary-contract.test.ts',
    'src/lib/prompt-pack-crate-boundary-contract.test.ts',
    'src/lib/analysis-crate-boundary-contract.test.ts',
    'src/lib/analysis-application-contract.test.ts'
)
$oldTreePaths = @($relativePaths | ForEach-Object { "src-tauri/src/telegram_impl/$_" })
$newTreePaths = @($relativePaths | ForEach-Object { "src-tauri/crates/extractum-telegram/src/$_" })
$expectedImplementationPaths = @(
    ($implementationFixedPaths + $oldTreePaths + $newTreePaths) |
        Sort-Object -Unique
)

$baseBlobRows = foreach ($relativePath in $relativePaths) {
    $frozenSpec = "${Frozen8B}:src-tauri/src/telegram_impl/$relativePath"
    $baseSpec = "${BASE_COMMIT}:src-tauri/src/telegram_impl/$relativePath"
    $frozenBlob = (git rev-parse $frozenSpec).Trim()
    if ($LASTEXITCODE -ne 0) { throw "Missing frozen blob $frozenSpec" }
    $baseBlob = (git rev-parse $baseSpec).Trim()
    if ($LASTEXITCODE -ne 0) { throw "Missing BASE blob $baseSpec" }
    if ($frozenBlob -ne $baseBlob) {
        throw "BASE source drift for ${relativePath}: $baseBlob != $frozenBlob"
    }
    "$relativePath $baseBlob"
}
$baseBlobRows | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'base-source-blobs.txt')
$baseBlobRows
Save-Phase8CState -Values @{
    relativePaths = @($relativePaths)
    expectedImplementationPaths = @($expectedImplementationPaths)
}
```

Expected: exactly 19 equal blob rows.

- [ ] **Step 6: Capture the exact 736-test baseline and derive the final 665/71 expected sets**

```powershell
$BASE_TESTS = @(Get-RustTestIds -Package 'extractum')
if ($BASE_TESTS.Count -ne 736) {
    throw "Expected 736 BASE extractum library tests, found $($BASE_TESTS.Count)"
}
$BASE_TESTS | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'base-extractum-tests.txt')

$identityAuthority = Get-Content -Raw -LiteralPath `
    'src/lib/telegram-8b-test-identities.json' | ConvertFrom-Json
$futureOwner = @(
    @($identityAuthority.preNewStaged) + @($identityAuthority.phase8BNewStaged) |
        Sort-Object
)
if ($futureOwner.Count -ne 71) { throw "Expected 71 future-owner tests, found $($futureOwner.Count)" }
if (@($futureOwner | Select-Object -Unique).Count -ne 71) { throw 'Future-owner authority contains duplicates' }
if (@($futureOwner | Where-Object { -not $_.StartsWith('telegram_impl::') }).Count -ne 0) {
    throw 'Future-owner identity lacks telegram_impl prefix'
}

$futureOwnerSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
$futureOwner | ForEach-Object { [void]$futureOwnerSet.Add($_) }
$EXPECTED_APP_TESTS = @($BASE_TESTS | Where-Object { -not $futureOwnerSet.Contains($_) } | Sort-Object)
$EXPECTED_CRATE_TESTS = @($futureOwner | ForEach-Object { $_ -replace '^telegram_impl::', '' } | Sort-Object)
if ($EXPECTED_APP_TESTS.Count -ne 665) { throw "Expected 665 final app tests, found $($EXPECTED_APP_TESTS.Count)" }
if ($EXPECTED_CRATE_TESTS.Count -ne 71) { throw "Expected 71 final crate tests, found $($EXPECTED_CRATE_TESTS.Count)" }

$EXPECTED_APP_TESTS | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'expected-final-app-tests.txt')
$EXPECTED_CRATE_TESTS | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'expected-final-crate-tests.txt')
Save-Phase8CState -Values @{
    BASE_TESTS = @($BASE_TESTS)
    EXPECTED_APP_TESTS = @($EXPECTED_APP_TESTS)
    EXPECTED_CRATE_TESTS = @($EXPECTED_CRATE_TESTS)
}
```

Expected: exact counts 736 baseline, 665 expected app, and 71 expected crate.

- [ ] **Step 7: Capture clean locked metadata and immutable hashes for later comparison**

```powershell
$baselineMetadata = Invoke-CheckedNative 'BASE locked Cargo metadata' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
}
$baselineMetadata.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'base-cargo-metadata.json')
$expectedHashes.GetEnumerator() |
    ForEach-Object { "$($_.Key) $($_.Value)" } |
    Set-Content -LiteralPath (Join-Path $EvidenceRoot 'frozen-authority-sha256.txt')
```

Expected: metadata exit 0 and no repository diff.

### Task 2: Create the Transient Extracted Layout and Observe the Exact `E0432` RED

**Files:**
- Create: `src-tauri/crates/extractum-telegram/Cargo.toml`
- Move: the exact 19 paths from `src-tauri/src/telegram_impl/**` to `src-tauri/crates/extractum-telegram/src/**`
- Replace: `src-tauri/src/telegram_impl/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock` through Cargo only

**Interfaces:**
- Consumes: `$BASE_COMMIT`, `$Frozen8B`, `$EvidenceRoot`, and the exact 19-path authority from Task 1.
- Produces: the final physical package edge without test-support declarations, plus a recorded primary facade `E0432` naming both unresolved fixtures.

- [ ] **Step 1: Recheck the source/target paths, then move the complete directory once**

```powershell
$repositoryRoot = (Resolve-Path -LiteralPath '.').Path
$sourceRoot = (Resolve-Path -LiteralPath 'src-tauri/src/telegram_impl').Path
$expectedSourceRoot = [IO.Path]::GetFullPath((Join-Path $repositoryRoot 'src-tauri/src/telegram_impl'))
if (-not [String]::Equals($sourceRoot, $expectedSourceRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Unexpected Telegram source root: $sourceRoot"
}
$crateRoot = [IO.Path]::GetFullPath((Join-Path $repositoryRoot 'src-tauri/crates/extractum-telegram'))
$workspacePrefix = [IO.Path]::GetFullPath((Join-Path $repositoryRoot 'src-tauri/crates')) + [IO.Path]::DirectorySeparatorChar
if (-not $crateRoot.StartsWith($workspacePrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Telegram crate target escaped src-tauri/crates: $crateRoot"
}
if (Test-Path -LiteralPath $crateRoot) { throw 'extractum-telegram target already exists' }
New-Item -ItemType Directory -Path $crateRoot | Out-Null

git mv -- src-tauri/src/telegram_impl src-tauri/crates/extractum-telegram/src
if ($LASTEXITCODE -ne 0) { throw 'git mv of the exact Telegram tree failed' }
New-Item -ItemType Directory -Path 'src-tauri/src/telegram_impl' | Out-Null
```

Expected: all 19 files are under the new `src` root; no individual file is copied or reconstructed. The vacated app directory exists and is empty so Step 3 can add the new facade as genuinely new app-owned content.

- [ ] **Step 2: Create the exact transient producer manifest, deliberately without `[features]`**

Use `apply_patch` to create `src-tauri/crates/extractum-telegram/Cargo.toml` with exactly:

```toml
[package]
name = "extractum-telegram"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
base64.workspace = true
chacha20poly1305.workspace = true
extractum-core = { path = "../extractum-core" }
grammers-client.workspace = true
grammers-mtsender.workspace = true
grammers-session.workspace = true
grammers-tl-types.workspace = true
rand_core.workspace = true
secrecy.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio = { workspace = true, features = ["rt", "sync", "time"] }

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "test-util"] }
```

Expected: this manifest is already frozen from the moved source imports; only the approved feature table is intentionally absent for RED. Tokio's current `#[tokio::test]` default is `current_thread`: the production `rt` feature supplies that runtime, `macros` supplies the attribute, and `test-util` supplies `start_paused`; no moved test selects `flavor = "multi_thread"`, so `rt-multi-thread` is deliberately absent.

- [ ] **Step 3: Create the exact private app facade, including the still-unresolvable test imports**

Use `apply_patch` to create `src-tauri/src/telegram_impl/lib.rs` with exactly:

```rust
#[allow(unused_imports)]
pub(crate) use extractum_telegram::{
    decode_session_json, encode_session_json, session_json_requires_existing_key,
    DialogListing, ForumTopicSnapshot, LiveMessage, LiveMessageBatch, MessageRange,
    PeerDescriptor, SessionEncryptionKey, TakeoutAttempt, TakeoutCount, TakeoutFallback,
    TakeoutFallbackKind, TakeoutMessage, TakeoutPage, TakeoutPeer, TakeoutTransport,
    TelegramApiHash, TelegramClientHandle, TelegramItemContext, TelegramLoginAttempt,
    TelegramMediaPayload, TelegramMessageDraft, TelegramMessageIdentity, TelegramRuntime,
    TelegramRuntimeStatus, TelegramSession, ITEM_KIND_TELEGRAM_MESSAGE,
};

#[cfg(test)]
pub(crate) use extractum_telegram::{
    takeout_attempt_fixture,
    takeout_fallback_fixture,
};
```

Expected: one explicit production allowlist plus exactly two `cfg(test)` fixture names; no glob, module, wrapper, function, type, or behavior.

- [ ] **Step 4: Apply the exact transient root-manifest delta**

Use `apply_patch` on `src-tauri/Cargo.toml`:

```diff
-members = [".", "crates/extractum-core", "crates/extractum-gemini-browser", "crates/extractum-llm", "crates/extractum-prompt-packs", "crates/extractum-analysis"]
+members = [".", "crates/extractum-core", "crates/extractum-gemini-browser", "crates/extractum-llm", "crates/extractum-prompt-packs", "crates/extractum-analysis", "crates/extractum-telegram"]
```

Remove exactly these `[dependencies]` lines:

```toml
grammers-client = { workspace = true }
grammers-session = { workspace = true }
grammers-mtsender = { workspace = true }
grammers-tl-types = { workspace = true }
chacha20poly1305 = { workspace = true }
rand_core = { workspace = true }
```

Add exactly this feature-free normal edge to `[dependencies]`, adjacent to the other `extractum-*` path packages:

```toml
extractum-telegram = { path = "crates/extractum-telegram" }
```

Do not add an app dev-dependency in this step. Do not change `[workspace.dependencies]`.

- [ ] **Step 5: Let Cargo generate the path-package lockfile change without compiling tests**

```powershell
$transientMetadata = Invoke-CheckedNative 'transient Cargo metadata and lock refresh' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1
}
$transientMetadata.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'transient-cargo-metadata.json')
```

Expected: exit 0; `Cargo.lock` gains the local package/edge but no registry version, checksum, Git revision, or workspace-dependency declaration changes.

- [ ] **Step 6: Run the one accepted compile-time RED and validate its primary diagnostic exactly**

```powershell
$redCapture = Invoke-CapturedNative -Label 'Phase 8C fixture-boundary RED' -Command {
    cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib --no-run
}
Write-Host -NoNewline $redCapture.Text
$redCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'fixture-boundary-red.txt')
if ($redCapture.ExitCode -eq 0) { throw 'Expected fixture-boundary compile failure' }

$codedDiagnostics = @([regex]::Matches(
    $redCapture.Text,
    '(?ms)^error\[(E\d+)\]:.*?(?=^error(?:\[E\d+\])?:|\z)'
))
if ($codedDiagnostics.Count -lt 1 -or $codedDiagnostics.Count -gt 2) {
    throw "Expected one primary and at most one secondary coded diagnostic, found $($codedDiagnostics.Count)"
}
$wrongCodes = @($codedDiagnostics | Where-Object { $_.Groups[1].Value -ne 'E0432' })
if ($wrongCodes.Count -ne 0) { throw 'RED emitted a compiler error other than E0432' }
$bareErrors = @(
    $redCapture.Text -split "\r?\n" |
        Where-Object {
            $_ -match '^error:'
            -and $_ -notmatch '^error: (?:could not compile|aborting due to)'
        }
)
if ($bareErrors.Count -ne 0) {
    $bareErrors
    throw 'RED emitted a non-diagnostic Cargo/compiler error'
}

$primaryDiagnostics = @($codedDiagnostics | Where-Object {
    $_.Value -match 'src-tauri[\\/]src[\\/]telegram_impl[\\/]lib\.rs'
})
if ($primaryDiagnostics.Count -ne 1) {
    throw "Expected exactly one primary facade E0432, found $($primaryDiagnostics.Count)"
}
$primaryDiagnostic = $primaryDiagnostics[0].Value
if (-not $primaryDiagnostic.Contains('extractum_telegram::takeout_attempt_fixture')) {
    throw 'Primary E0432 omitted takeout_attempt_fixture'
}
if (-not $primaryDiagnostic.Contains('extractum_telegram::takeout_fallback_fixture')) {
    throw 'Primary E0432 omitted takeout_fallback_fixture'
}
$secondaryDiagnostics = @($codedDiagnostics | Where-Object {
    $_.Index -ne $primaryDiagnostics[0].Index
})
if ($secondaryDiagnostics.Count -eq 1 -and $secondaryDiagnostics[0].Value -notmatch 'takeout_import[\\/]mod\.rs') {
    throw 'The optional secondary E0432 is not at the takeout_import import'
}
if ($redCapture.Text -match '(?m)^\s*Executable unittests') {
    throw 'RED unexpectedly produced a test executable'
}
if ($redCapture.Text -match '(?m)^\s*Running unittests|^test result:') {
    throw 'RED unexpectedly started tests'
}
```

Expected: nonzero exit and the primary facade `E0432` naming both fixtures. Preserve the complete output. If the compiler also emits a downstream `takeout_import` `E0432`, record it as secondary evidence; do not require it and do not accept it instead of the facade diagnostic.

- [ ] **Step 7: Inspect the transient state without running a boundary contract or committing it**

```powershell
git status --short
git diff -- src-tauri/Cargo.toml src-tauri/Cargo.lock `
    src-tauri/src/telegram_impl src-tauri/crates/extractum-telegram
```

Expected: one uncommitted extraction change. Do not run Vitest, `npm.cmd run test:changed`, or `npm.cmd run verify` in this state.

### Task 3: Apply the Atomic Fixture Correction and Make Both Rust Packages GREEN

**Files:**
- Modify: `src-tauri/crates/extractum-telegram/Cargo.toml`
- Modify: `src-tauri/crates/extractum-telegram/src/lib.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/takeout/mod.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock` through Cargo only

**Interfaces:**
- Consumes: the recorded Task 2 RED and the exact two-file correction from the approved design.
- Produces: one final uncommitted Rust/Cargo state in which the producer is feature-off in isolation, the consumer is feature-on for development targets, and all four app fixture consumers pass.

- [ ] **Step 1: Apply only the eight approved moved-source substitutions**

Use `apply_patch` on `src-tauri/crates/extractum-telegram/src/lib.rs`:

```diff
-#[cfg(test)]
-pub(crate) use takeout::attempt_fixture as takeout_attempt_fixture;
-#[cfg(test)]
-pub(crate) use takeout::fallback_fixture as takeout_fallback_fixture;
+#[cfg(feature = "app-test-support")]
+pub use takeout::attempt_fixture as takeout_attempt_fixture;
+#[cfg(feature = "app-test-support")]
+pub use takeout::fallback_fixture as takeout_fallback_fixture;
```

Use `apply_patch` on `src-tauri/crates/extractum-telegram/src/takeout/mod.rs`:

```diff
-#[cfg(test)]
-pub(crate) fn fallback_fixture(
+#[cfg(feature = "app-test-support")]
+pub fn fallback_fixture(
@@
-#[cfg(test)]
-pub(crate) fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {
+#[cfg(feature = "app-test-support")]
+pub fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {
```

Expected: two attribute substitutions and two visibility substitutions in each file. Do not run a formatter or touch either function body/signature beyond the shown visibility token.

- [ ] **Step 2: Add the producer feature and consumer dev edge together, then refresh the lockfile**

Insert this table between `[package]` and `[dependencies]` in the producer manifest:

```toml
[features]
app-test-support = []
```

The complete final producer manifest must be exactly:

```toml
[package]
name = "extractum-telegram"
version.workspace = true
edition.workspace = true
publish = false

[features]
app-test-support = []

[dependencies]
base64.workspace = true
chacha20poly1305.workspace = true
extractum-core = { path = "../extractum-core" }
grammers-client.workspace = true
grammers-mtsender.workspace = true
grammers-session.workspace = true
grammers-tl-types.workspace = true
rand_core.workspace = true
secrecy.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio = { workspace = true, features = ["rt", "sync", "time"] }

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "test-util"] }
```

Add exactly this second declaration to the app's existing `[dev-dependencies]` table:

```toml
extractum-telegram = { path = "crates/extractum-telegram", features = ["app-test-support"] }
```

Keep the normal declaration exactly `extractum-telegram = { path = "crates/extractum-telegram" }`. Then let Cargo update the existing lockfile:

```powershell
$greenMetadata = Invoke-CheckedNative 'final Cargo metadata and lock refresh' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1
}
$greenMetadata.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'green-cargo-metadata.json')
```

Expected: metadata exit 0. The producer declares one non-default empty feature; the app declares the same canonical path once normally and once as a dev-dependency with exactly that feature.

- [ ] **Step 3: Run all four exact app-owned fixture consumers and reject empty selections**

```powershell
$fixtureConsumerTests = @(
    'takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future',
    'takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation',
    'takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize',
    'takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues'
)
foreach ($testName in $fixtureConsumerTests) {
    $capture = Invoke-ExactRustTest -Package 'extractum' -TestName $testName
    $capture.Text | Set-Content -LiteralPath (
        Join-Path $EvidenceRoot ("fixture-green-" + ($testName -replace '[^A-Za-z0-9_.-]', '_') + '.txt')
    )
}
Save-Phase8CState -Values @{ fixtureConsumerTests = @($fixtureConsumerTests) }
```

Expected: four commands, each reports `running 1 test`, one pass, and zero failures.

- [ ] **Step 4: Run and record the complete focused Rust verification loop in ownership order**

```powershell
$rustLoop = [ordered]@{
    'producer-feature-off-lib-check' = {
        cargo check --color never --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features
    }
    'producer-isolated-all-targets-check' = {
        cargo check --color never --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
    }
    'producer-isolated-all-targets-test' = {
        cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets
    }
    'consumer-normal-lib-check' = {
        cargo check --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib
    }
    'consumer-feature-on-all-targets-check' = {
        cargo check --color never --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
    }
    'consumer-feature-on-all-targets-test' = {
        cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
    }
}
foreach ($entry in $rustLoop.GetEnumerator()) {
    $capture = Invoke-CheckedNative $entry.Key $entry.Value
    $capture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot "$($entry.Key).txt")
}
```

Expected: all six commands exit 0. Describe only the first command as canonical feature-off proof. The two producer-isolated `--all-targets` commands also execute feature-off in this package selection, but are package checkpoints rather than the canonical proof. The consumer `--all-targets` commands are feature-on through resolver v2.

- [ ] **Step 5: Prove the producer tests do not rely on either fixture**

```powershell
$crateSourceRoot = (Resolve-Path -LiteralPath 'src-tauri/crates/extractum-telegram/src').Path
$fixtureReferences = @(
    Get-ChildItem -LiteralPath $crateSourceRoot -Recurse -Filter '*.rs' -File |
        Select-String -Pattern 'takeout_(?:attempt|fallback)_fixture|\b(?:attempt|fallback)_fixture\b' |
        ForEach-Object {
            $relative = $_.Path.Substring($crateSourceRoot.Length + 1).Replace('\', '/')
            "$relative|$($_.Line.Trim())"
        }
)
$expectedFixtureReferences = @(
    'lib.rs|pub use takeout::attempt_fixture as takeout_attempt_fixture;',
    'lib.rs|pub use takeout::fallback_fixture as takeout_fallback_fixture;',
    'takeout/mod.rs|pub fn fallback_fixture(',
    'takeout/mod.rs|pub fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {'
)
Assert-ExactStringSet -Label 'producer fixture declaration/reference allowlist' `
    -Expected $expectedFixtureReferences -Actual $fixtureReferences
```

Expected: exactly the two private definitions and two crate-root re-exports, with no fifth match in a producer test body. The final TypeScript contract makes this a standing assertion.

### Task 4: Update the Standing Final-Layout Contracts Without Rewriting 8B Authority

**Files:**
- Modify: `scripts/telegram-grammers-feature-baseline.mjs`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `src/lib/rust-workspace-core-contract.test.ts`
- Modify: `src/lib/gemini-browser-crate-boundary-contract.test.ts`
- Modify: `src/lib/llm-crate-boundary-contract.test.ts`
- Modify: `src/lib/prompt-pack-crate-boundary-contract.test.ts`
- Modify: `src/lib/analysis-crate-boundary-contract.test.ts`
- Modify: `src/lib/analysis-application-contract.test.ts`
- Read unchanged: `src/lib/telegram-contract-paths.ts`
- Read unchanged: all four immutable Telegram JSON authorities

**Interfaces:**
- Consumes: the final physical layout, the existing `8c-extracted` path mapping, the immutable 8B path/test/symbol/feature authorities, and Cargo locked metadata.
- Produces: one fail-closed final-boundary assertion that remains GREEN before terminal status changes and preserves all older package-boundary assertions.

- [ ] **Step 1: Move the generated Grammers feature proof to the new direct owner without changing its output**

Apply this complete owner-transfer diff; do not change revision/source/feature-universe logic, negative source/feature-definition tests, or serialization:

```diff
diff --git a/scripts/telegram-grammers-feature-baseline.mjs b/scripts/telegram-grammers-feature-baseline.mjs
--- a/scripts/telegram-grammers-feature-baseline.mjs
+++ b/scripts/telegram-grammers-feature-baseline.mjs
@@
 const revision = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa";
 const exactSource =
   `git+https://codeberg.org/Lonami/grammers?rev=${revision}#${revision}`;
+const directOwnerName = "extractum-telegram";
 const packageNames = [
@@
-  const extractumPackages = metadataPackages.filter(
-    (candidate) => candidate?.name === "extractum",
+  const directOwnerPackages = metadataPackages.filter(
+    (candidate) => candidate?.name === directOwnerName,
   );
-  if (extractumPackages.length !== 1) {
-    fail(`expected one extractum package, found ${extractumPackages.length}`);
+  if (directOwnerPackages.length !== 1) {
+    fail(
+      `expected one ${directOwnerName} package, found ${directOwnerPackages.length}`,
+    );
   }
-  const extractumNodes = metadataNodes.filter(
-    (candidate) => candidate?.id === extractumPackages[0].id,
+  const directOwnerNodes = metadataNodes.filter(
+    (candidate) => candidate?.id === directOwnerPackages[0].id,
   );
-  if (extractumNodes.length !== 1) {
+  if (directOwnerNodes.length !== 1) {
     fail(
-      `expected one resolved extractum node, found ${extractumNodes.length}`,
+      `expected one resolved ${directOwnerName} node, found ${directOwnerNodes.length}`,
     );
   }
-  const extractumNode = extractumNodes[0];
-  if (!Array.isArray(extractumNode.deps)) {
-    fail("missing resolved extractum node");
+  const directOwnerNode = directOwnerNodes[0];
+  if (!Array.isArray(directOwnerNode.deps)) {
+    fail(`missing resolved ${directOwnerName} node`);
   }

-  const directPackageIds = extractumNode.deps
+  const directPackageIds = directOwnerNode.deps
@@
   ) {
-    fail("extractum must have four distinct direct Grammers package IDs");
+    fail(
+      `${directOwnerName} must have four distinct direct Grammers package IDs`,
+    );
   }

diff --git a/src/lib/crate-extraction-shell-cap-contract.test.ts b/src/lib/crate-extraction-shell-cap-contract.test.ts
--- a/src/lib/crate-extraction-shell-cap-contract.test.ts
+++ b/src/lib/crate-extraction-shell-cap-contract.test.ts
@@
       {
-        id: "extractum-fixture",
-        name: "extractum",
+        id: "extractum-telegram-fixture",
+        name: "extractum-telegram",
         features: {},
       },
@@
         {
-          id: "extractum-fixture",
+          id: "extractum-telegram-fixture",
           deps: packageRecords.map(({ id }) => ({ pkg: id })),
           features: [],
         },
@@
-  it("feature authority rejects duplicate extractum resolve nodes", () => {
+  it("feature authority rejects duplicate extractum-telegram resolve nodes", () => {
     const metadata = featureMetadataFixture();
-    const extractumNode = metadata.resolve.nodes[0];
+    const directOwnerNode = metadata.resolve.nodes[0];
     metadata.resolve.nodes.push({
-      ...extractumNode,
-      deps: [...extractumNode.deps],
-      features: [...extractumNode.features],
+      ...directOwnerNode,
+      deps: [...directOwnerNode.deps],
+      features: [...directOwnerNode.features],
     });
     expect(() => generateFeatureBaseline(metadata)).toThrow(
-      /expected one resolved extractum node/,
+      /expected one resolved extractum-telegram node, found 2/,
     );
   });
```

```powershell
node scripts/telegram-grammers-feature-baseline.mjs --check
$featureHash = (Get-FileHash -LiteralPath `
    'src/lib/telegram-grammers-feature-baseline.json' -Algorithm SHA256).Hash.ToLowerInvariant()
if ($featureHash -ne $expectedHashes['src/lib/telegram-grammers-feature-baseline.json']) {
    throw "Grammers feature artifact changed after owner transfer: $featureHash"
}
```

Expected: check exit 0 and SHA-256 remains `774e2b979d5cdc8185a85488c965548cf09cdf1ef0ab4b9ecad58246283cf5b3`.

- [ ] **Step 2: Add one exact physical-state detector to the Telegram contract and make it the final lifecycle authority**

In `src/lib/telegram-crate-boundary-contract.test.ts`:

1. Read the approved bounded design `docs/superpowers/specs/2026-08-01-telegram-8c-extraction-design.md`, the immutable staging manifest, the new producer manifest, facade, and locked Cargo metadata.
2. Define the exact 19 relative paths from `telegram-8b-staging-sha256.json`; do not copy a second inventory.
3. Add a pure `phase8CPhysicalLifecycle(snapshot)` helper. It returns `undefined` only when every 8C signal is absent, returns `"8c-extracted"` only when all signals below are exact, and throws `Partial Phase 8C layout` for every mixed state:

```text
producer Cargo.toml exists
all 19 crate destinations exist and no extra Rust path exists under the crate src root
the 18 non-lib old staged paths are absent
the old staged lib path contains the exact app facade, not a moved implementation copy
workspace members are exactly the seven approved members
the app normal dependency is exact and feature-free
the app dev-dependency is exact and enables only app-test-support
the producer manifest is exact
```

4. Select `physicalLifecycle ?? retainedPreparationLifecycle`; never require roadmap/design terminal status for final GREEN.
5. Add table-driven unit cases for no signals, every single partial signal, one missing signal from an otherwise complete snapshot, extra destination, wrong facade, wrong member, wrong normal edge, wrong dev edge, and the exact complete state.

Use this closed detector shape (the manifest/facade constants contain the exact text frozen in Tasks 2–3):

```ts
type Phase8CStagedLibState =
  | "absent"
  | "retained-implementation"
  | "exact-facade"
  | "other";

type Phase8BStagingHashManifest = {
  schemaVersion: number;
  algorithm: string;
  root: string;
  files: Array<{
    path: string;
    sha256: string;
  }>;
};

type Phase8CPhysicalSnapshot = Readonly<{
  producerManifest: string | undefined;
  destinationRustPaths: readonly string[];
  oldNonFacadeRustPaths: readonly string[];
  stagedLibState: Phase8CStagedLibState;
  workspaceMembers: readonly string[];
  normalTelegramDeclarations: readonly string[];
  devTelegramDeclarations: readonly string[];
}>;

const phase8CExactProducerManifest = `[package]
name = "extractum-telegram"
version.workspace = true
edition.workspace = true
publish = false

[features]
app-test-support = []

[dependencies]
base64.workspace = true
chacha20poly1305.workspace = true
extractum-core = { path = "../extractum-core" }
grammers-client.workspace = true
grammers-mtsender.workspace = true
grammers-session.workspace = true
grammers-tl-types.workspace = true
rand_core.workspace = true
secrecy.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio = { workspace = true, features = ["rt", "sync", "time"] }

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "test-util"] }
`;
const phase8CExactFacade = `#[allow(unused_imports)]
pub(crate) use extractum_telegram::{
    decode_session_json, encode_session_json, session_json_requires_existing_key,
    DialogListing, ForumTopicSnapshot, LiveMessage, LiveMessageBatch, MessageRange,
    PeerDescriptor, SessionEncryptionKey, TakeoutAttempt, TakeoutCount, TakeoutFallback,
    TakeoutFallbackKind, TakeoutMessage, TakeoutPage, TakeoutPeer, TakeoutTransport,
    TelegramApiHash, TelegramClientHandle, TelegramItemContext, TelegramLoginAttempt,
    TelegramMediaPayload, TelegramMessageDraft, TelegramMessageIdentity, TelegramRuntime,
    TelegramRuntimeStatus, TelegramSession, ITEM_KIND_TELEGRAM_MESSAGE,
};

#[cfg(test)]
pub(crate) use extractum_telegram::{
    takeout_attempt_fixture,
    takeout_fallback_fixture,
};
`;
const phase8CStagingAuthority =
  readGeneratedJson<Phase8BStagingHashManifest>(
    "src/lib/telegram-8b-staging-sha256.json",
  );
const phase8CRelativeRustPaths = phase8CStagingAuthority.files.map(
  ({ path: relativePath }) => relativePath,
);
const phase8CPathAuthorityIsSorted = phase8CRelativeRustPaths.every(
  (relativePath, index) =>
    index === 0
    || phase8CRelativeRustPaths[index - 1].localeCompare(relativePath) < 0,
);
if (
  phase8CStagingAuthority.schemaVersion !== 1
  || phase8CStagingAuthority.algorithm !== "sha256"
  || phase8CStagingAuthority.root !== "src-tauri/src/telegram_impl"
  || phase8CRelativeRustPaths.length !== 19
  || new Set(phase8CRelativeRustPaths).size !== 19
  || !phase8CPathAuthorityIsSorted
  || phase8CStagingAuthority.files.some(({ path: relativePath, sha256 }) =>
    path.posix.isAbsolute(relativePath)
    || relativePath.includes("\\")
    || !relativePath.endsWith(".rs")
    || relativePath.split("/").some(
      (segment) => !segment || segment === "." || segment === "..",
    )
    || !/^[0-9a-f]{64}$/.test(sha256)
  )
) {
  throw new Error("Malformed Phase 8C staging authority");
}

const phase8CRetainedLibRecords = phase8CStagingAuthority.files.filter(
  ({ path: relativePath }) => relativePath === "lib.rs",
);
if (phase8CRetainedLibRecords.length !== 1) {
  throw new Error("Malformed Phase 8C retained lib authority");
}
const phase8CRetainedLibSha256 = phase8CRetainedLibRecords[0].sha256;
const phase8CStagedLibPath = `${phase8CStagingAuthority.root}/lib.rs`;
const phase8CDestinationRoot =
  "src-tauri/crates/extractum-telegram/src";
const phase8CDestinationRustPaths = phase8CRelativeRustPaths.map(
  (relativePath) => `${phase8CDestinationRoot}/${relativePath}`,
);
const phase8COldNonFacadeRustPaths = phase8CRelativeRustPaths
  .filter((relativePath) => relativePath !== "lib.rs")
  .map(
    (relativePath) => `${phase8CStagingAuthority.root}/${relativePath}`,
  );

function sameStrings(
  left: readonly string[],
  right: readonly string[],
): boolean {
  const leftSorted = [...left].sort();
  const rightSorted = [...right].sort();
  if (
    new Set(leftSorted).size !== leftSorted.length
    || new Set(rightSorted).size !== rightSorted.length
  ) {
    return false;
  }
  return leftSorted.length === rightSorted.length
    && leftSorted.every((value, index) => value === rightSorted[index]);
}

const phase8CWorkspaceMembers = [
  ".",
  "crates/extractum-core",
  "crates/extractum-gemini-browser",
  "crates/extractum-llm",
  "crates/extractum-prompt-packs",
  "crates/extractum-analysis",
  "crates/extractum-telegram",
] as const;
const phase8BWorkspaceMembers = phase8CWorkspaceMembers.filter(
  (member) => member !== "crates/extractum-telegram",
);
const phase8CNormalDeclaration =
  'extractum-telegram = { path = "crates/extractum-telegram" }';
const phase8CDevDeclaration =
  'extractum-telegram = { path = "crates/extractum-telegram", features = ["app-test-support"] }';

function phase8CPhysicalLifecycle(
  snapshot: Phase8CPhysicalSnapshot,
  retainedPreparationLifecycle: telegramContractPaths.TelegramLifecycle,
): telegramContractPaths.TelegramLifecycle | undefined {
  const positiveSignal = snapshot.producerManifest !== undefined
    || snapshot.destinationRustPaths.length !== 0
    || snapshot.stagedLibState === "exact-facade"
    || snapshot.workspaceMembers.includes("crates/extractum-telegram")
    || snapshot.normalTelegramDeclarations.length !== 0
    || snapshot.devTelegramDeclarations.length !== 0;
  if (!positiveSignal) {
    if (retainedPreparationLifecycle === "8c-extracted") {
      throw new Error(
        "Partial Phase 8C layout: terminal status without extracted physical state",
      );
    }
    if (retainedPreparationLifecycle === "8b-preparation") {
      const retainedFailures = [
        snapshot.producerManifest === undefined
          ? undefined : "unexpected producer manifest",
        snapshot.destinationRustPaths.length === 0
          ? undefined : "unexpected destination tree",
        sameStrings(
          snapshot.oldNonFacadeRustPaths,
          phase8COldNonFacadeRustPaths,
        )
          ? undefined : "retained non-facade tree",
        snapshot.stagedLibState === "retained-implementation"
          ? undefined : "retained staged lib",
        sameStrings(snapshot.workspaceMembers, phase8BWorkspaceMembers)
          ? undefined : "retained workspace members",
        snapshot.normalTelegramDeclarations.length === 0
          ? undefined : "unexpected normal dependency",
        snapshot.devTelegramDeclarations.length === 0
          ? undefined : "unexpected dev dependency",
      ].filter((failure): failure is string => failure !== undefined);
      if (retainedFailures.length !== 0) {
        throw new Error(
          `Partial Phase 8C layout: ${retainedFailures.join(", ")}`,
        );
      }
    }
    return undefined;
  }

  const failures = [
    snapshot.producerManifest === phase8CExactProducerManifest
      ? undefined : "producer manifest",
    sameStrings(snapshot.destinationRustPaths, phase8CDestinationRustPaths)
      ? undefined : "destination tree",
    snapshot.oldNonFacadeRustPaths.length === 0
      ? undefined : "old non-facade tree",
    snapshot.stagedLibState === "exact-facade" ? undefined : "facade",
    sameStrings(snapshot.workspaceMembers, phase8CWorkspaceMembers)
      ? undefined : "workspace members",
    sameStrings(snapshot.normalTelegramDeclarations, [phase8CNormalDeclaration])
      ? undefined : "normal dependency",
    sameStrings(snapshot.devTelegramDeclarations, [phase8CDevDeclaration])
      ? undefined : "dev dependency",
  ].filter((failure): failure is string => failure !== undefined);
  if (failures.length !== 0) {
    throw new Error(`Partial Phase 8C layout: ${failures.join(", ")}`);
  }
  return "8c-extracted";
}
```

Move the existing `Phase8BStagingHashManifest` declaration to this block and delete its later duplicate. Place this entire constants/helper block before the current `const statusLifecycle`; function declarations such as `tomlSection`, `rustPathsUnder`, and `readGeneratedJson` may remain below because they are hoisted, but none of the new `const` declarations may be initialized after `lifecycle`.

Add the real snapshot wiring immediately after `phase8CPhysicalLifecycle`:

```ts
function phase8COptionalContractSource(
  relativePath: string,
): string | undefined {
  return existsSync(path.join(repoRoot, relativePath))
    ? telegramContractPaths.readTelegramContractFile(relativePath)
    : undefined;
}

function phase8CRealStagedLibState(): Phase8CStagedLibState {
  if (!existsSync(path.join(repoRoot, phase8CStagedLibPath))) {
    return "absent";
  }

  const absolutePath = telegramContractPaths.resolveTelegramContractPath(
    phase8CStagedLibPath,
  );
  const rawSource = readFileSync(absolutePath);
  const normalizedSource = normalize(rawSource.toString("utf8"));
  if (normalizedSource === phase8CExactFacade) {
    return "exact-facade";
  }

  const sha256 = createHash("sha256").update(rawSource).digest("hex");
  return sha256 === phase8CRetainedLibSha256
    ? "retained-implementation"
    : "other";
}

function phase8CWorkspaceMembersFromCargo(source: string): string[] {
  const workspace = tomlSection(source, "workspace");
  const declarations = [
    ...workspace.matchAll(/^members\s*=\s*(\[[^\n]*\])\s*$/gm),
  ];
  if (declarations.length !== 1) {
    throw new Error(
      "Partial Phase 8C layout: malformed workspace members declaration",
    );
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(declarations[0][1]);
  } catch {
    throw new Error(
      "Partial Phase 8C layout: malformed workspace members array",
    );
  }
  if (
    !Array.isArray(parsed)
    || parsed.some((member) => typeof member !== "string")
  ) {
    throw new Error(
      "Partial Phase 8C layout: non-string workspace member",
    );
  }
  return parsed as string[];
}

function phase8CTelegramDependencyDeclarations(
  source: string,
  heading: "dependencies" | "dev-dependencies",
): string[] {
  return tomlSection(source, heading)
    .split("\n")
    .filter((line) => line.includes("extractum-telegram"));
}

function phase8CRealPhysicalSnapshot(): Phase8CPhysicalSnapshot {
  return {
    producerManifest: phase8COptionalContractSource(
      "src-tauri/crates/extractum-telegram/Cargo.toml",
    ),
    destinationRustPaths: rustPathsUnder(phase8CDestinationRoot),
    oldNonFacadeRustPaths:
      rustPathsUnder(phase8CStagingAuthority.root).filter(
        (relativePath) => relativePath !== phase8CStagedLibPath,
      ),
    stagedLibState: phase8CRealStagedLibState(),
    workspaceMembers: phase8CWorkspaceMembersFromCargo(rootCargo),
    normalTelegramDeclarations: phase8CTelegramDependencyDeclarations(
      rootCargo,
      "dependencies",
    ),
    devTelegramDeclarations: phase8CTelegramDependencyDeclarations(
      rootCargo,
      "dev-dependencies",
    ),
  };
}
```

Replace the existing `statusLifecycle` through `lifecycle` initialization with this complete block; it makes physical state authoritative even after the roadmap becomes terminal:

```ts
const statusLifecycle =
  telegramContractPaths.telegramLifecycleFromStatus(phase8Status);
const checkpoint3LeavesExist = [
  "src-tauri/src/telegram/dto.rs",
  "src-tauri/src/telegram/media.rs",
].every((relativePath) =>
  existsSync(path.join(repoRoot, relativePath))
);
const checkpointThreeLifecycle =
  statusLifecycle === "8a-checkpoint-2" && checkpoint3LeavesExist
    ? "8a-checkpoint-3"
    : statusLifecycle;
const checkpoint4LeafExists = existsSync(
  path.join(repoRoot, "src-tauri/src/telegram/session.rs"),
);
const checkpointFourLifecycle =
  checkpointThreeLifecycle === "8a-checkpoint-3"
    && checkpoint4LeafExists
    ? "8a-checkpoint-4"
    : checkpointThreeLifecycle;
const stagedFoundationIsCurrent =
  phase8BStagedFoundationPaths.every((relativePath) =>
    existsSync(path.join(repoRoot, relativePath))
  )
  && phase8BOldFoundationPaths.every((relativePath) =>
    !existsSync(path.join(repoRoot, relativePath))
  );
const retainedPreparationLifecycle =
  checkpointFourLifecycle === "8b-checkpoint-2"
    && stagedFoundationIsCurrent
    ? "8b-checkpoint-3"
    : checkpointFourLifecycle;

const phase8CPhysicalSnapshot = phase8CRealPhysicalSnapshot();
const physicalLifecycle = phase8CPhysicalLifecycle(
  phase8CPhysicalSnapshot,
  retainedPreparationLifecycle,
);
const lifecycle = physicalLifecycle ?? retainedPreparationLifecycle;
```

The exact facade constant is:

```rust
#[allow(unused_imports)]
pub(crate) use extractum_telegram::{
    decode_session_json, encode_session_json, session_json_requires_existing_key,
    DialogListing, ForumTopicSnapshot, LiveMessage, LiveMessageBatch, MessageRange,
    PeerDescriptor, SessionEncryptionKey, TakeoutAttempt, TakeoutCount, TakeoutFallback,
    TakeoutFallbackKind, TakeoutMessage, TakeoutPage, TakeoutPeer, TakeoutTransport,
    TelegramApiHash, TelegramClientHandle, TelegramItemContext, TelegramLoginAttempt,
    TelegramMediaPayload, TelegramMessageDraft, TelegramMessageIdentity, TelegramRuntime,
    TelegramRuntimeStatus, TelegramSession, ITEM_KIND_TELEGRAM_MESSAGE,
};

#[cfg(test)]
pub(crate) use extractum_telegram::{
    takeout_attempt_fixture,
    takeout_fallback_fixture,
};
```

Compare normalized LF source plus one terminal newline; reject glob exports, additional items, modules, declarations, or behavior.

- [ ] **Step 3: Reuse the retained 8B scanners by mapping final paths back to their staged logical identity**

Extend current-path resolution so every immutable staged path maps to `src-tauri/crates/extractum-telegram/src/<relative>` under lifecycle `8c-extracted`, not only the seven earlier lifecycle examples. Before feeding the two corrected final files into retained 8B source/symbol scanners, apply an exact in-memory reverse correction:

```text
crate lib.rs: two feature attributes -> cfg(test); two public fixture uses -> pub(crate) use
crate takeout/mod.rs: two feature attributes -> cfg(test); two public fixture definitions -> pub(crate) fn
```

Each reverse replacement must match exactly twice for the attribute and exactly once for each named declaration; otherwise throw. Never write the normalized source to disk and never weaken the frozen 8B source/symbol assertions.

Use one exact-count primitive and keep the correction local to the two approved paths:

```ts
function replaceExactCount(
  source: string,
  before: string,
  after: string,
  expectedCount: number,
): string {
  const actualCount = source.split(before).length - 1;
  if (actualCount !== expectedCount) {
    throw new Error(`Phase 8C reverse correction count drifted: ${before}`);
  }
  return source.split(before).join(after);
}

function phase8BComparableExtractedSource(
  stagedRelativePath: string,
  source: string,
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): string {
  if (value !== "8c-extracted") return source;
  if (stagedRelativePath === "lib.rs") {
    return replaceExactCount(
      replaceExactCount(
        replaceExactCount(
          source,
          '#[cfg(feature = "app-test-support")]',
          "#[cfg(test)]",
          2,
        ),
        "pub use takeout::attempt_fixture as takeout_attempt_fixture;",
        "pub(crate) use takeout::attempt_fixture as takeout_attempt_fixture;",
        1,
      ),
      "pub use takeout::fallback_fixture as takeout_fallback_fixture;",
      "pub(crate) use takeout::fallback_fixture as takeout_fallback_fixture;",
      1,
    );
  }
  if (stagedRelativePath === "takeout/mod.rs") {
    return replaceExactCount(
      replaceExactCount(
        replaceExactCount(
          source,
          '#[cfg(feature = "app-test-support")]',
          "#[cfg(test)]",
          2,
        ),
        "pub fn fallback_fixture(",
        "pub(crate) fn fallback_fixture(",
        1,
      ),
      "pub fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {",
      "pub(crate) fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {",
      1,
    );
  }
  return source;
}
```

Keep two explicit collections: physical final keys for ownership/identity, and staged logical keys for immutable 8B scanners. The new compatibility facade participates only in app-facade assertions. Add these helpers beside the current-path helpers:

```ts
const phase8CStagedLifecycleSources =
  phase8CRelativeRustPaths.map((relativePath) => ({
    baselinePath: `${phase8CStagingAuthority.root}/${relativePath}`,
    stagedPath: `${phase8CStagingAuthority.root}/${relativePath}`,
    finalOwner: "extractum-telegram" as const,
  }));

function phase8BComparableCheckpoint(
  value: telegramContractPaths.TelegramLifecycle,
): number | undefined {
  return value === "8c-extracted"
    ? 8
    : telegramContractPaths.phase8BCheckpointNumber(value);
}

function phase8BComparableLifecycle(
  value: telegramContractPaths.TelegramLifecycle,
): telegramContractPaths.TelegramLifecycle {
  return value === "8c-extracted" ? "8b-checkpoint-8" : value;
}

function phase8BPhysicalStagedPath(
  stagedPath: string,
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): string {
  if (value !== "8c-extracted") return stagedPath;
  const prefix = `${phase8CStagingAuthority.root}/`;
  if (!stagedPath.startsWith(prefix)) {
    throw new Error(
      `Phase 8B comparable path lies outside staging root: ${stagedPath}`,
    );
  }
  return `${phase8CDestinationRoot}/${stagedPath.slice(prefix.length)}`;
}

function phase8BComparableCurrentPath(
  currentPath: string,
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): string {
  if (value !== "8c-extracted") return currentPath;
  const destinationPrefix = `${phase8CDestinationRoot}/`;
  if (!currentPath.startsWith(destinationPrefix)) return currentPath;
  return `${phase8CStagingAuthority.root}/${
    currentPath.slice(destinationPrefix.length)
  }`;
}

function resolvePhase8BComparableSourcePath(
  relativePath: string,
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): string {
  return phase8BComparableCurrentPath(
    resolveCurrentTelegramSourcePath(relativePath, value),
    value,
  );
}
```

Replace `resolveCurrentTelegramSourcePath`, `readCurrentTelegramContractFile`, and `currentAppRustPaths` with these full bodies:

```ts
function resolveCurrentTelegramSourcePath(
  relativePath: string,
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): string {
  if (
    value === "8c-extracted"
    && relativePath.startsWith(`${phase8CDestinationRoot}/`)
  ) {
    return relativePath;
  }

  const source = [
    ...phase8BFoundationLifecycleSources,
    ...phase8BRelocatedTakeoutLifecycleSources,
    ...phase8CStagedLifecycleSources,
  ].find(
    ({ baselinePath, stagedPath }) =>
      relativePath === baselinePath || relativePath === stagedPath,
  );
  return source === undefined
    ? relativePath
    : telegramContractPaths.resolveTelegramLifecyclePath(source, value);
}

function readCurrentTelegramContractFile(
  relativePath: string,
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): string {
  const resolvedPath = resolveCurrentTelegramSourcePath(relativePath, value);
  const source = sourceOverrides.get(resolvedPath)
    ?? sourceOverrides.get(relativePath)
    ?? telegramContractPaths.readTelegramContractFile(resolvedPath);

  if (
    value !== "8c-extracted"
    || !resolvedPath.startsWith(`${phase8CDestinationRoot}/`)
  ) {
    return source;
  }
  return phase8BComparableExtractedSource(
    resolvedPath.slice(`${phase8CDestinationRoot}/`.length),
    source,
    value,
  );
}

function currentAppRustPaths(
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): string[] {
  const appPaths = rustPathsUnder("src-tauri/src");
  if (value === "8c-extracted") {
    return [
      ...appPaths.filter(
        (relativePath) => relativePath !== phase8CStagedLibPath,
      ),
      ...rustPathsUnder(phase8CDestinationRoot),
    ].sort();
  }

  const selectedFoundationPaths = new Set(
    phase8BFoundationLifecycleSources.map((source) =>
      telegramContractPaths.resolveTelegramLifecyclePath(source, value)
    ),
  );
  const allFoundationPaths = new Set<string>([
    ...phase8BOldFoundationPaths,
    ...phase8BStagedFoundationPaths,
  ]);
  return appPaths.filter(
    (relativePath) =>
      !allFoundationPaths.has(relativePath)
      || selectedFoundationPaths.has(relativePath),
  );
}
```

Replace `checkpointThreeAppRustSources` and add the logical scanner collection:

```ts
function checkpointThreeAppRustSources(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): ReadonlyMap<string, string> {
  return new Map(
    currentAppRustPaths(value).map((relativePath) => [
      relativePath,
      readCurrentTelegramContractFile(
        relativePath,
        sourceOverrides,
        value,
      ),
    ]),
  );
}

function phase8BComparableCurrentSources(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): ReadonlyMap<string, string> {
  const comparable = new Map<string, string>();
  for (
    const [physicalPath, source]
    of checkpointThreeAppRustSources(sourceOverrides, value)
  ) {
    const comparablePath = phase8BComparableCurrentPath(
      physicalPath,
      value,
    );
    if (comparable.has(comparablePath)) {
      throw new Error(
        `Duplicate Phase 8B comparable source: ${comparablePath}`,
      );
    }
    comparable.set(comparablePath, source);
  }
  return comparable;
}
```

Apply the substitutions below in exactly these five tests, in this order:

1. `rejects an unlisted current production definition in a fully moved module` — DTO path;
2. `rejects an unlisted current associated definition in a fully moved module` — DTO path;
3. `rejects a current Phase 8B method spoofed by a same-named field` — runtime path;
4. `rejects a current Phase 8B method spoofed by a same-named associated non-function` — runtime path;
5. `rejects a current Phase 8B field spoofed by a same-named method` — pagination path.

Their complete existing mutation callbacks and error regexes remain byte-identical:

```diff
-    const currentSources = new Map(
-      rustPathsUnder("src-tauri/src").map((relativePath) => [
-        relativePath,
-        telegramContractPaths.readTelegramContractFile(relativePath),
-      ]),
-    );
+    const currentSources = phase8BComparableCurrentSources();
@@
-      resolveCurrentTelegramSourcePath(
+      resolvePhase8BComparableSourcePath(
@@
-      assertCurrentPhase8BSymbolAuthority(mutated, artifact)
+      assertCurrentPhase8BSymbolAuthority(
+        mutated,
+        artifact,
+        phase8BComparableLifecycle(lifecycle),
+      )
```

No callback body, source literal, or expected error regex changes.

Replace the live current-source block in `reconciles every Phase 8B checkpoint source fixture with plan authority`:

```ts
const comparableLifecycle = phase8BComparableLifecycle(lifecycle);
const currentCheckpoint =
  telegramContractPaths.phase8BCheckpointNumber(comparableLifecycle);
const currentSources = phase8BComparableCurrentSources();

if (currentCheckpoint !== undefined) {
  assertPhase8BCheckpointSourceContract(
    comparableLifecycle,
    currentSources,
    artifact,
  );
}
if (currentCheckpoint !== undefined && currentCheckpoint >= 3) {
  const identityArtifact = readGeneratedJson<Phase8BIdentityAuthority>(
    "src/lib/telegram-8b-test-identities.json",
  );
  assertPhase8BCP3FoundationIdentityDeclarations(
    currentSources,
    identityArtifact,
  );
}
assertCurrentPhase8BSymbolAuthority(
  currentSources,
  artifact,
  comparableLifecycle,
);
```

In exactly these four helpers — `assertCheckpointThreeOwnershipContract`, `assertCheckpointThreeApiContract`, `assertCheckpointFourSessionContract`, and `assertCheckpointFiveRuntimeContract` — replace:

```ts
const phase8BCheckpoint =
  telegramContractPaths.phase8BCheckpointNumber(lifecycle);
```

with:

```ts
const phase8BCheckpoint = phase8BComparableCheckpoint(lifecycle);
```

The persistence mutation test has a different inline form. In `rejects mutated persistence signatures, facade allowlist, and import directions`, replace its complete checkpoint block with:

```ts
const phase8BCheckpoint = phase8BComparableCheckpoint(lifecycle);
const foundationIsStaged =
  phase8BCheckpoint !== undefined && phase8BCheckpoint >= 3;
```

In the session contract, replace the root selection with:

```ts
const telegramPath = foundationIsStaged
  ? resolveCurrentTelegramSourcePath(
    "src-tauri/src/telegram_impl/lib.rs",
  )
  : "src-tauri/src/telegram.rs";
```

In that same `assertCheckpointFourSessionContract`, add the physical live-module path immediately after `adapterPath`:

```ts
const liveModulePath = phase8BPhysicalStagedPath(
  "src-tauri/src/telegram_impl/live/mod.rs",
);
```

Then replace only the first line of the expected raw-adapter record:

```diff
-            "src-tauri/src/telegram_impl/live/mod.rs|pub(super) async fn fetch_message_batch(",
+            `${liveModulePath}|pub(super) async fn fetch_message_batch(`,
```

This keeps the retained 8B expectation unchanged while pointing the terminal comparison at the producer's physical `live/mod.rs`.

In the runtime contract, replace it with:

```ts
const runtimeFacadePath = foundationIsStaged
  ? resolveCurrentTelegramSourcePath(
    "src-tauri/src/telegram_impl/lib.rs",
  )
  : telegramPath;
```

In both CP7 short-circuit tests (`keeps semantic facade sites stable...` and `rejects moving a facade reference...`), replace the opening branch with:

```ts
if (
  lifecycle === "8c-extracted"
  || (phase8BCheckpoint !== undefined && phase8BCheckpoint >= 7)
) {
  expect(
    phase8BRelocatedTakeoutLifecycleSources.map(
      ({ baselinePath }) =>
        resolveCurrentTelegramSourcePath(baselinePath),
    ),
  ).toEqual(
    phase8BRelocatedTakeoutLifecycleSources.map(
      ({ stagedPath }) => phase8BPhysicalStagedPath(stagedPath),
    ),
  );
  return;
}
```

Replace the source setup in `checkpoint-gates all 18 plan-added identities outside the immutable 140`:

```ts
const rustSources = checkpointThreeAppRustSources();
assertAddedIdentityDeclarations(
  addedIdentities,
  lifecycle,
  rustSources,
);
```

Allow the physical final lifecycle in the following accounting assertion without adding a terminal row to `retainedPreparationStates`:

```ts
expect([
  ...retainedPreparationStates.map(({ lifecycle: state }) => state),
  "8c-extracted",
]).toContain(lifecycle);
```

Replace the final future-owner leak source selection with raw physical sources:

```ts
const futureOwnerLeaves = currentAppRustPaths().filter(
  (relativePath) =>
    relativePath.startsWith("src-tauri/src/telegram/")
    || relativePath.startsWith("src-tauri/src/telegram_impl/")
    || relativePath.startsWith(`${phase8CDestinationRoot}/`),
).map(
  (relativePath) =>
    telegramContractPaths.readTelegramContractFile(relativePath),
).join("\n");
```

Make the two generated structural-authority tests lifecycle-aware. In the first test, retain the generator existence assertion but run `--check` only while the staged tree is physical, and replace the future-root assertion:

```ts
if (lifecycle !== "8c-extracted") {
  const result = runNodeGenerator(generatorPath, ["--check"]);
  expect(result.status, `${result.stdout}${result.stderr}`).toBe(0);
}
expect(existsSync(path.join(repoRoot, futureRoot))).toBe(
  lifecycle === "8c-extracted",
);
```

After reading the artifact in `rejects missing extra reordered or byte-drifted staged files`, replace the remaining body with:

```ts
const expectedRoot = "src-tauri/src/telegram_impl";
const expectedPaths = phase8BPortableTreePaths().map(
  (relativePath) => relativePath.slice(`${expectedRoot}/`.length),
);
expect(artifact.files.map(({ path: relativePath }) => relativePath))
  .toEqual(expectedPaths);

const physicalRoot = lifecycle === "8c-extracted"
  ? phase8CDestinationRoot
  : expectedRoot;
const actualPaths = recursiveRepositoryFiles(physicalRoot).sort();
expect(actualPaths).toEqual(expectedPaths);
if (lifecycle === "8c-extracted") return;

const actualRecords = actualPaths.map((relativePath) => {
  return {
    path: relativePath,
    sha256: createHash("sha256")
      .update(
        readFileSync(
          path.join(repoRoot, physicalRoot, relativePath),
        ),
      )
      .digest("hex"),
  };
});

expect(artifact.files).toEqual(actualRecords);
expect(
  artifact.files.every(
    ({ sha256 }) => /^[0-9a-f]{64}$/.test(sha256),
  ),
).toBe(true);
```

The staging generator is intentionally not invoked against the extracted layout: Task 1 already checked it against retained physical 8B, and Tasks 5/6 provide the one-time normative 17-blob/two-diff extraction proof. The terminal standing test keeps the exact 19-path inventory without freezing future legitimate producer source edits to 8B hashes. Retained source/API/symbol scanners still use the in-memory reverse correction because rustc does not replace their exact visibility and restricted-bridge allowlists.

The final identity assertion continues to derive its 71 expected crate identities from the immutable 8B identity artifact by removing `telegram_impl::`; do not embed the 736-list or add a second generated ownership artifact.

Replace the complete existing test `normalizes the pinned Telegram dependencies without workspace graph drift` with this lifecycle-aware version. This replacement is required in addition to the new focused 8C `describe` block below; otherwise the retained six-member assertion makes the extracted layout impossible to turn GREEN.

```ts
it("normalizes the pinned Telegram dependencies without workspace graph drift", () => {
  const workspace = tomlSection(rootCargo, "workspace");
  const workspaceDependencies = tomlSection(
    rootCargo,
    "workspace.dependencies",
  );
  const appDependencies = tomlSection(rootCargo, "dependencies");
  const members = workspace
    .match(/^members\s*=\s*\[([^\]]+)\]$/m)?.[1]
    .split(",")
    .map((member) => member.trim().replace(/^"|"$/g, ""));
  const telegramCrateExtracted = lifecycle === "8c-extracted";

  expect(members).toEqual([
    ".",
    "crates/extractum-core",
    "crates/extractum-gemini-browser",
    "crates/extractum-llm",
    "crates/extractum-prompt-packs",
    "crates/extractum-analysis",
    ...(telegramCrateExtracted
      ? ["crates/extractum-telegram"]
      : []),
  ]);
  expect(
    rootCargo
      .split("\n")
      .filter((line) => line.startsWith("extractum-telegram = ")),
  ).toEqual(
    telegramCrateExtracted
      ? [
          'extractum-telegram = { path = "crates/extractum-telegram" }',
          'extractum-telegram = { path = "crates/extractum-telegram", features = ["app-test-support"] }',
        ]
      : [],
  );

  const movedDirectRoots = new Set([
    "chacha20poly1305",
    "grammers-client",
    "grammers-mtsender",
    "grammers-session",
    "grammers-tl-types",
    "rand_core",
  ]);
  for (const specification of workspaceDependencyNormalization) {
    expect(
      workspaceDependencies
        .split("\n")
        .filter((line) => line === specification),
      specification,
    ).toEqual([specification]);
    const dependency = specification.slice(0, specification.indexOf(" ="));
    const inheritedLine = `${dependency} = { workspace = true }`;
    expect(
      appDependencies
        .split("\n")
        .filter((line) => line === inheritedLine),
      `${dependency} workspace inheritance`,
    ).toEqual(
      telegramCrateExtracted && movedDirectRoots.has(dependency)
        ? []
        : [inheritedLine],
    );
  }

  expect(grammersFeatureBaseline.revision).toBe(
    "1f901ce6e973fdcf0e74267f3d8efad5c729daaa",
  );
  expect(
    grammersFeatureBaseline.packages.map(({ name, required }) => ({
      name,
      required,
    })),
  ).toEqual([
    { name: "grammers-client", required: [] },
    { name: "grammers-mtsender", required: [] },
    { name: "grammers-session", required: ["serde"] },
    {
      name: "grammers-tl-types",
      required: [
        "default",
        "deserializable-functions",
        "impl-debug",
        "impl-from-enum",
        "impl-from-type",
        "tl-api",
        "tl-mtproto",
      ],
    },
  ]);
});
```

Replace the complete existing test `records only the truthful retained Phase 8 lifecycle states` with:

```ts
it("records only the truthful retained Phase 8 lifecycle states", () => {
  const current = {
    roadmapStatus: phase8Status,
    designStatus: designPhase8Status,
    lifecycle: statusLifecycle,
  };

  if (statusLifecycle === "8c-extracted") {
    expect(current).toEqual({
      roadmapStatus: "done: retained",
      designStatus:
        "Implemented and retained; [verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md)",
      lifecycle: "8c-extracted",
    });
  } else {
    expect(retainedPreparationStates).toContainEqual(current);
  }
});
```

Rename `recognizes every retained Phase 8B lifecycle and rejects unknown values` to `recognizes every retained Phase 8 lifecycle and rejects unknown values`. Immediately after its loop that checks the sixteen historical Phase 8B rows, insert this exact conditional terminal-row contract:

```ts
const terminalRegistryRows = [
  "| `8c-extracted` | agent-workflow lifecycle | Selects the retained Phase 8C `extractum-telegram` package layout. | `telegram-contract-paths.ts` | terminal | yes | none | tests and agent workflow |",
  "| `done: retained` | agent-workflow status input | Maps the terminal Phase 8 roadmap disposition to `8c-extracted`. | `telegram-contract-paths.ts` | terminal | yes | none | tests and agent workflow |",
];
for (const row of terminalRegistryRows) {
  const occurrences = valueRegistry
    .split(/\r?\n/)
    .filter((line) => line === row)
    .length;
  expect(occurrences).toBe(statusLifecycle === "8c-extracted" ? 1 : 0);
}
```

- [ ] **Step 4: Add the exact final package, fixture, metadata, and graph-cut assertions**

Add one focused `describe("Phase 8C extracted Telegram boundary", ...)` block that proves:

- producer manifest exactness, including only `app-test-support = []`, the 12 production dependency roots, and Tokio dev features `macros`/`test-util`;
- app manifest exactness: exactly one feature-free normal edge plus one dev edge enabling only `app-test-support`, exactly six removed direct roots, retained app `base64`/`secrecy`, and byte-identical `[workspace.dependencies]` declarations;
- the app's `[features]` table equals the exact two-line normalized-LF literal copied from `BASE_COMMIT`, the two app dependency declarations are exact, and `app-test-support` occurs in workspace manifests only in the app `[dev-dependencies]` table and producer `[features]` table. Do not add a second activation model for package scripts, Cargo aliases/configs, Tauri overlays, target-specific synthetic edges, or `--all-features`; the approved design relies on the two Cargo command invariants below;
- exactly four feature-gated declarations in moved source: two private-module `pub fn` definitions and two crate-root `pub use` exports, representing exactly the external names `takeout_attempt_fixture` and `takeout_fallback_fixture`;
- both producer definitions and exports use the single exact attribute `#[cfg(feature = "app-test-support")]`; reject unconditional forms, `cfg(test)`, `cfg(any(...))`, a third fixture, public modules, a producer test reference, or a widened production item;
- the app facade contains only its curated production allowlist plus the exact `cfg(test)` block above;
- locked metadata contains seven workspace members and one `extractum-telegram` package; the app has one resolved edge to it whose `dep_kinds` are exactly normal and dev, and both declarations resolve to the same canonical path package;
- the app has no direct resolved edge to any of the four Grammers packages, `chacha20poly1305`, or `rand_core`; the producer has exactly those six roots plus `base64`, `extractum-core`, `secrecy`, `serde`, `serde_json`, and `tokio`;
- masked Rust-source searches prove each intentional duplicate root is consumed at least once on both sides: app and producer for `base64`, and app and producer for `secrecy`. Do not freeze unrelated app file locations or add removal mutations; metadata presence alone is not ownership evidence;
- all four Grammers package source/revision/required/forbidden/universe values equal the immutable feature artifact;
- after removing the app-to-`extractum-telegram` edge from the resolved graph, no Grammers node remains reachable from the app; no reverse producer-to-app edge exists;
- all 71 future-owner test declarations occur exactly once in the producer under their prefix-removed names and none remains under the old staged implementation tree.

Keep the parent-required Grammers scan as secondary evidence and the `extractum_telegram` inventory as an independent facade-only standing contract. Both use the closed app-package perimeter below, not only `src-tauri/src/**`: `src-tauri/build.rs` plus every Rust file under the conventional app-owned `src`, `tests`, `examples`, and `benches` roots. Mask comments and string/character literals with the retained lexical helper. The combined alias mutation contains both `use grammers_client::tl as raw_tl` and `raw_tl::enums::Peer`, so the origin cannot be separated from the alias use. These source scans supplement, never replace, locked metadata and graph-cut proof.

```ts
type Phase8CNamedSource = Readonly<{
  relativePath: string;
  source: string;
}>;

const phase8CAppFeatureSection = [
  'csp-verification = ["tauri/devtools"]',
  'prompt-pack-dev-fixtures = ["extractum-prompt-packs/dev-fixtures"]',
].join("\n");
const phase8CAppTelegramDeclarations = [
  'extractum-telegram = { path = "crates/extractum-telegram" }',
  'extractum-telegram = { path = "crates/extractum-telegram", features = ["app-test-support"] }',
] as const;
const phase8CWorkspaceManifestPaths = phase8CWorkspaceMembers.map((member) =>
  member === "."
    ? "src-tauri/Cargo.toml"
    : `src-tauri/${member}/Cargo.toml`
);
const phase8CAppRustDirectoryRoots = [
  "src-tauri/src",
  "src-tauri/tests",
  "src-tauri/examples",
  "src-tauri/benches",
] as const;

function phase8CAppOwnedRustPaths(): string[] {
  const paths = [
    ...(existsSync(path.join(repoRoot, "src-tauri/build.rs"))
      ? ["src-tauri/build.rs"]
      : []),
    ...phase8CAppRustDirectoryRoots.flatMap((relativeRoot) =>
      rustPathsUnder(relativeRoot)
    ),
  ].sort();
  if (new Set(paths).size !== paths.length) {
    throw new Error("Duplicate app-owned Rust path");
  }
  return paths;
}

function phase8CNamedRustSources(
  relativePaths: readonly string[],
): Phase8CNamedSource[] {
  return relativePaths.map((relativePath) => ({
    relativePath,
    source: telegramContractPaths.readTelegramContractFile(relativePath),
  }));
}

function phase8CMaskedMatchPaths(
  sources: readonly Phase8CNamedSource[],
  expression: RegExp,
): string[] {
  const flags = expression.flags.replace(/[gy]/g, "");
  return sources.flatMap(({ relativePath, source }) => {
    const probe = new RegExp(expression.source, flags);
    return probe.test(maskRustLexicalNonCode(source)) ? [relativePath] : [];
  }).sort();
}

function phase8CTomlTables(
  source: string,
): Array<Readonly<{ heading: string; body: string }>> {
  const markers = [
    ...source.matchAll(/^(\[\[?)([^\]\n]+)(\]\]?)[ \t]*(?:#[^\n]*)?$/gm),
  ];
  for (const marker of markers) {
    if (marker[1].length !== marker[3].length) {
      throw new Error(`Malformed TOML table heading: ${marker[0]}`);
    }
  }
  return markers.map((marker, index) => {
    const bodyStart = (marker.index ?? 0) + marker[0].length;
    const bodyEnd = markers[index + 1]?.index ?? source.length;
    return {
      heading: marker[2],
      body: source.slice(bodyStart, bodyEnd).trim(),
    };
  });
}

function phase8CWorkspaceManifestSources(): ReadonlyMap<string, string> {
  return new Map(phase8CWorkspaceManifestPaths.map((relativePath) => [
    relativePath,
    telegramContractPaths.readTelegramContractFile(relativePath),
  ]));
}

function phase8CWorkspaceFeatureMentions(
  workspaceManifests: ReadonlyMap<string, string>,
): string[] {
  return [...workspaceManifests.entries()].flatMap(
    ([relativePath, source]) => phase8CTomlTables(source).flatMap(
      ({ heading, body }) => Array.from(
        { length: body.split("app-test-support").length - 1 },
        () => `${relativePath}|${heading}`,
      ),
    ),
  ).sort();
}
```

Add these complete tests to the Phase 8C `describe` block:

```ts
it("freezes the exact producer manifest and resolver-v2 consumer declarations", () => {
  const producerManifest = telegramContractPaths.readTelegramContractFile(
    "src-tauri/crates/extractum-telegram/Cargo.toml",
  );
  expect(producerManifest).toBe(phase8CExactProducerManifest);
  assertPhase8CMetadataBoundary(phase8CLockedMetadata());

  const producerManifestDrift = exactMutation(
    producerManifest,
    'features = ["macros", "test-util"]',
    'features = ["macros", "rt-multi-thread", "test-util"]',
    "Phase 8C producer Tokio dev-feature widening",
  );
  expect(producerManifestDrift).not.toBe(phase8CExactProducerManifest);

  const metadataKindDrift = phase8CCloneMetadata(phase8CLockedMetadata()) as {
    packages: Phase8CMetadataPackage[];
    workspace_members: string[];
    resolve: { nodes: Array<{
      id: string;
      features: string[];
      deps: Phase8CMetadataNodeDep[];
    }> };
  };
  const metadataApp = phase8CWorkspacePackage(metadataKindDrift, "extractum");
  const metadataProducer = phase8CWorkspacePackage(
    metadataKindDrift,
    "extractum-telegram",
  );
  const metadataAppNode = metadataKindDrift.resolve.nodes.find(
    ({ id }) => id === metadataApp.id,
  );
  const metadataTelegramEdge = metadataAppNode?.deps.find(
    ({ pkg }) => pkg === metadataProducer.id,
  );
  if (!metadataTelegramEdge) throw new Error("Missing Telegram metadata edge");
  metadataTelegramEdge.dep_kinds.push({ kind: "build", target: null });
  expect(() => assertPhase8CMetadataBoundary(metadataKindDrift)).toThrow();

  const workspaceManifests = phase8CWorkspaceManifestSources();
  expect(workspaceManifests.get("src-tauri/Cargo.toml")).toBe(rootCargo);
  expect(tomlSection(rootCargo, "features")).toBe(phase8CAppFeatureSection);
  expect(
    rootCargo
      .split("\n")
      .filter((line) => /^extractum-telegram\s*=/.test(line)),
  ).toEqual([...phase8CAppTelegramDeclarations]);
  expect(phase8CWorkspaceFeatureMentions(workspaceManifests)).toEqual([
    "src-tauri/Cargo.toml|dev-dependencies",
    "src-tauri/crates/extractum-telegram/Cargo.toml|features",
  ]);
});

it("scans the complete app Rust perimeter and permits only the Telegram facade", () => {
  const appRustPaths = phase8CAppOwnedRustPaths();
  expect(appRustPaths).toContain("src-tauri/build.rs");
  const appSources = phase8CNamedRustSources(appRustPaths);
  const forbiddenGrammers =
    /\b(?:grammers_client|grammers_session|grammers_mtsender|grammers_tl_types)\b|\btl\s*::|\b(?:MemorySession|LoginToken|PeerRef|RemoteCall|InvocationError)\b/;
  expect(phase8CMaskedMatchPaths(appSources, forbiddenGrammers)).toEqual([]);

  const grammersMutations = [
    "use grammers_client::Client;",
    "fn leak() { let _ = grammers_client::Client::connect; }",
    "use grammers_client::tl as raw_tl; fn leak(peer: raw_tl::enums::Peer) {}",
    "pub use grammers_session::MemorySession;",
    "fn leak(peer: tl::enums::Peer) {}",
    "fn leak(value: MemorySession) {}",
    "fn leak(value: LoginToken) {}",
    "fn leak(value: PeerRef) {}",
    "fn leak(value: RemoteCall) {}",
    "fn leak(value: InvocationError) {}",
  ];
  for (const [index, source] of grammersMutations.entries()) {
    const relativePath = `src-tauri/tests/grammers-leak-${index}.rs`;
    expect(phase8CMaskedMatchPaths(
      [...appSources, { relativePath, source }],
      forbiddenGrammers,
    )).toEqual([relativePath]);
  }
  expect(phase8CMaskedMatchPaths([
    ...appSources.filter(({ relativePath }) => relativePath !== "src-tauri/build.rs"),
    { relativePath: "src-tauri/build.rs", source: "use grammers_client::Client;" },
  ], forbiddenGrammers)).toEqual(["src-tauri/build.rs"]);

  const telegramPackagePaths = phase8CMaskedMatchPaths(
    appSources,
    /\bextractum_telegram\b/,
  );
  expect(telegramPackagePaths).toEqual([
    "src-tauri/src/telegram_impl/lib.rs",
  ]);
  const facade = appSources.find(({ relativePath }) =>
    relativePath === "src-tauri/src/telegram_impl/lib.rs"
  );
  if (!facade) throw new Error("Missing app Telegram facade source");
  expect([
    ...maskRustLexicalNonCode(facade.source).matchAll(/\bextractum_telegram\b/g),
  ]).toHaveLength(2);

  const facadeBypassMutations = [
    "use extractum_telegram::TelegramSession;",
    "use extractum_telegram as raw_telegram;",
    "fn leak() { let _ = extractum_telegram::TelegramRuntimeStatus::default(); }",
    "pub use extractum_telegram::TelegramSession;",
  ];
  for (const [index, source] of facadeBypassMutations.entries()) {
    const relativePath = index === 0
      ? "src-tauri/build.rs"
      : `src-tauri/tests/facade-bypass-${index}.rs`;
    expect(phase8CMaskedMatchPaths(
      [...appSources.filter((entry) => entry.relativePath !== relativePath), {
        relativePath,
        source,
      }],
      /\bextractum_telegram\b/,
    )).toEqual([
      "src-tauri/src/telegram_impl/lib.rs",
      relativePath,
    ].sort());
  }
});

it("moves direct dependency ownership and preserves the Grammers feature closure", () => {
  const metadata = phase8CLockedMetadata();
  assertPhase8CMetadataBoundary(metadata);
  assertPhase8CGrammersClosure(metadata);

  const featureDrift = phase8CCloneMetadata(metadata) as {
    packages: Phase8CMetadataPackage[];
    workspace_members: string[];
    resolve: { nodes: Array<{
      id: string;
      features: string[];
      deps: Phase8CMetadataNodeDep[];
    }> };
  };
  const tlTypes = featureDrift.packages.find(
    ({ name }) => name === "grammers-tl-types",
  );
  const tlTypesNode = tlTypes === undefined
    ? undefined
    : featureDrift.resolve.nodes.find(({ id }) => id === tlTypes.id);
  if (!tlTypesNode) throw new Error("Missing Grammers feature mutation node");
  tlTypesNode.features.push("impl-serde");
  expect(() => assertPhase8CGrammersClosure(featureDrift)).toThrow();

  const appSources = phase8CNamedRustSources(phase8CAppOwnedRustPaths());
  const producerSources = phase8CNamedRustSources(
    rustPathsUnder("src-tauri/crates/extractum-telegram/src"),
  );
  const dependencyPaths = (
    sources: readonly Phase8CNamedSource[],
    dependency: "base64" | "secrecy",
  ) => phase8CMaskedMatchPaths(
    sources,
    new RegExp(`\\b${dependency}\\b`),
  );
  expect(dependencyPaths(appSources, "base64").length).toBeGreaterThan(0);
  expect(dependencyPaths(producerSources, "base64").length)
    .toBeGreaterThan(0);
  expect(dependencyPaths(appSources, "secrecy").length).toBeGreaterThan(0);
  expect(dependencyPaths(producerSources, "secrecy").length)
    .toBeGreaterThan(0);
});
```

Use this complete physical-state mutation test:

```ts
it("derives the complete extracted layout before terminal status and rejects every partial layout", () => {
  const pending: Phase8CPhysicalSnapshot = {
    producerManifest: undefined,
    destinationRustPaths: [],
    oldNonFacadeRustPaths: phase8COldNonFacadeRustPaths,
    stagedLibState: "retained-implementation",
    workspaceMembers: phase8BWorkspaceMembers,
    normalTelegramDeclarations: [],
    devTelegramDeclarations: [],
  };
  expect(phase8CPhysicalLifecycle(pending, "8b-preparation")).toBeUndefined();

  const complete: Phase8CPhysicalSnapshot = {
    producerManifest: phase8CExactProducerManifest,
    destinationRustPaths: phase8CDestinationRustPaths,
    oldNonFacadeRustPaths: [],
    stagedLibState: "exact-facade",
    workspaceMembers: phase8CWorkspaceMembers,
    normalTelegramDeclarations: [phase8CNormalDeclaration],
    devTelegramDeclarations: [phase8CDevDeclaration],
  };
  expect(phase8CPhysicalLifecycle(complete, "8b-preparation"))
    .toBe("8c-extracted");

  const singleSignalCases: Array<[string, Phase8CPhysicalSnapshot]> = [
    ["manifest", { ...pending, producerManifest: phase8CExactProducerManifest }],
    ["destination", { ...pending, destinationRustPaths: [phase8CDestinationRustPaths[0]] }],
    ["facade", { ...pending, stagedLibState: "exact-facade" }],
    ["member", { ...pending, workspaceMembers: phase8CWorkspaceMembers }],
    ["normal edge", { ...pending, normalTelegramDeclarations: [phase8CNormalDeclaration] }],
    ["dev edge", { ...pending, devTelegramDeclarations: [phase8CDevDeclaration] }],
  ];
  for (const [label, snapshot] of singleSignalCases) {
    expect(
      () => phase8CPhysicalLifecycle(snapshot, "8b-preparation"),
      label,
    ).toThrow(/Partial Phase 8C layout/);
  }

  const invalidCompleteCases: Array<[string, Phase8CPhysicalSnapshot]> = [
    ["missing manifest", { ...complete, producerManifest: undefined }],
    ["missing destination", { ...complete, destinationRustPaths: phase8CDestinationRustPaths.slice(1) }],
    ["extra destination", { ...complete, destinationRustPaths: [...phase8CDestinationRustPaths, "src-tauri/crates/extractum-telegram/src/extra.rs"] }],
    ["old leaf retained", { ...complete, oldNonFacadeRustPaths: [phase8COldNonFacadeRustPaths[0]] }],
    ["wrong facade", { ...complete, stagedLibState: "other" }],
    ["missing member", { ...complete, workspaceMembers: phase8CWorkspaceMembers.slice(0, -1) }],
    ["wrong normal edge", { ...complete, normalTelegramDeclarations: [phase8CDevDeclaration] }],
    ["wrong dev edge", { ...complete, devTelegramDeclarations: [phase8CNormalDeclaration] }],
  ];
  for (const [label, snapshot] of invalidCompleteCases) {
    expect(
      () => phase8CPhysicalLifecycle(snapshot, "8b-preparation"),
      label,
    ).toThrow(/Partial Phase 8C layout/);
  }

  for (const stagedLibState of ["other", "absent"] as const) {
    expect(() => phase8CPhysicalLifecycle(
      { ...pending, stagedLibState },
      "8b-preparation",
    )).toThrow(/Partial Phase 8C layout/);
  }
  expect(() => phase8CPhysicalLifecycle(
    pending,
    "8c-extracted",
  )).toThrow(/terminal status without extracted physical state/);
  expect(() => phase8CPhysicalLifecycle({
    ...pending,
    oldNonFacadeRustPaths: phase8COldNonFacadeRustPaths.slice(1),
  }, "8b-preparation")).toThrow(/Partial Phase 8C layout/);
  expect(() => phase8CPhysicalLifecycle({
    ...pending,
    oldNonFacadeRustPaths: [
      ...phase8COldNonFacadeRustPaths,
      "src-tauri/src/telegram_impl/extra.rs",
    ],
  }, "8b-preparation")).toThrow(/Partial Phase 8C layout/);
});
```

Add these complete metadata, fixture, graph-cut, and identity helpers. They read Cargo's one locked resolver result; they do not model feature unification independently or require a separate explicit feature-on producer command.

```ts
type Phase8CMetadataDependency = Readonly<{
  name: string;
  source: string | null;
  req: string;
  kind: "dev" | "build" | null;
  rename: string | null;
  path: string | null;
  registry: string | null;
  uses_default_features: boolean;
  features: string[];
  target: string | null;
}>;
type Phase8CMetadataPackage = Readonly<{
  id: string;
  name: string;
  source: string | null;
  manifest_path: string;
  features: Record<string, string[]>;
  dependencies: Phase8CMetadataDependency[];
}>;
type Phase8CMetadataNodeDep = Readonly<{
  name: string;
  pkg: string;
  dep_kinds: Array<Readonly<{
    kind: "dev" | "build" | null;
    target: string | null;
  }>>;
}>;
type Phase8CMetadataNode = Readonly<{
  id: string;
  features: string[];
  deps: Phase8CMetadataNodeDep[];
}>;
type Phase8CCargoMetadata = Readonly<{
  packages: Phase8CMetadataPackage[];
  workspace_members: string[];
  resolve: Readonly<{ nodes: Phase8CMetadataNode[] }>;
}>;
type Phase8CIdentityAuthority = Readonly<{
  preNewStaged: string[];
  phase8BNewStaged: string[];
}>;

let phase8CLockedMetadataCache: Phase8CCargoMetadata | undefined;

function phase8CLockedMetadata(): Phase8CCargoMetadata {
  if (phase8CLockedMetadataCache) return phase8CLockedMetadataCache;
  const result = spawnSync(
    "cargo",
    [
      "metadata",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--locked",
      "--format-version",
      "1",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      shell: false,
      maxBuffer: 256 * 1024 * 1024,
    },
  );
  if (result.error || result.status !== 0 || !result.stdout.trim()) {
    throw new Error(
      `Phase 8C locked Cargo metadata failed: ${result.stderr || result.error}`,
    );
  }
  phase8CLockedMetadataCache = JSON.parse(result.stdout) as Phase8CCargoMetadata;
  return phase8CLockedMetadataCache;
}

function phase8CCloneMetadata(
  metadata: Phase8CCargoMetadata,
): Phase8CCargoMetadata {
  return JSON.parse(JSON.stringify(metadata)) as Phase8CCargoMetadata;
}

function phase8CPackageById(
  metadata: Phase8CCargoMetadata,
  packageId: string,
): Phase8CMetadataPackage {
  const matches = metadata.packages.filter(({ id }) => id === packageId);
  if (matches.length !== 1) {
    throw new Error(`Expected one metadata package for ${packageId}`);
  }
  return matches[0];
}

function phase8CWorkspacePackage(
  metadata: Phase8CCargoMetadata,
  name: "extractum" | "extractum-telegram",
): Phase8CMetadataPackage {
  const members = new Set(metadata.workspace_members);
  const matches = metadata.packages.filter(
    (candidate) => candidate.name === name && members.has(candidate.id),
  );
  if (matches.length !== 1) {
    throw new Error(`Expected one workspace package named ${name}`);
  }
  return matches[0];
}

function phase8CNode(
  metadata: Phase8CCargoMetadata,
  packageId: string,
): Phase8CMetadataNode {
  const matches = metadata.resolve.nodes.filter(({ id }) => id === packageId);
  if (matches.length !== 1) {
    throw new Error(`Expected one resolved node for ${packageId}`);
  }
  return matches[0];
}

function phase8CNormalizedDepKinds(
  dependency: Phase8CMetadataNodeDep,
): Array<Readonly<{ kind: "normal" | "dev"; target: null }>> {
  const order = new Map([["normal", 0], ["dev", 1]]);
  return dependency.dep_kinds.map(({ kind, target }) => ({
    kind: kind ?? "normal",
    target: target ?? null,
  })).sort((left, right) =>
    (order.get(left.kind) ?? 99) - (order.get(right.kind) ?? 99)
  ) as Array<Readonly<{ kind: "normal" | "dev"; target: null }>>;
}

function assertPhase8CMetadataBoundary(
  metadata: Phase8CCargoMetadata,
): void {
  const memberNames = metadata.workspace_members.map((packageId) =>
    phase8CPackageById(metadata, packageId).name
  );
  exactInventory(memberNames, [
    "extractum",
    "extractum-core",
    "extractum-gemini-browser",
    "extractum-llm",
    "extractum-prompt-packs",
    "extractum-analysis",
    "extractum-telegram",
  ], "Phase 8C Cargo metadata workspace members");

  const appPackage = phase8CWorkspacePackage(metadata, "extractum");
  const producerPackage = phase8CWorkspacePackage(
    metadata,
    "extractum-telegram",
  );
  expect(producerPackage.source).toBeNull();
  expect(path.resolve(producerPackage.manifest_path)).toBe(
    path.join(repoRoot, "src-tauri/crates/extractum-telegram/Cargo.toml"),
  );
  expect(Object.keys(producerPackage.features).sort()).toEqual([
    "app-test-support",
  ]);

  const declaredTelegramEdges = appPackage.dependencies.filter(
    ({ name }) => name === "extractum-telegram",
  ).map((dependency) => ({
    kind: dependency.kind ?? "normal",
    target: dependency.target,
    rename: dependency.rename,
    source: dependency.source,
    path: dependency.path === null ? null : path.resolve(dependency.path),
    features: [...dependency.features].sort(),
  })).sort((left) => left.kind === "normal" ? -1 : 1);
  expect(declaredTelegramEdges).toEqual([
    {
      kind: "normal",
      target: null,
      rename: null,
      source: null,
      path: path.join(repoRoot, "src-tauri/crates/extractum-telegram"),
      features: [],
    },
    {
      kind: "dev",
      target: null,
      rename: null,
      source: null,
      path: path.join(repoRoot, "src-tauri/crates/extractum-telegram"),
      features: ["app-test-support"],
    },
  ]);

  const appNode = phase8CNode(metadata, appPackage.id);
  const telegramNodeDependencies = appNode.deps.filter(
    ({ pkg }) => pkg === producerPackage.id,
  );
  expect(telegramNodeDependencies).toHaveLength(1);
  expect(telegramNodeDependencies[0].name).toBe("extractum_telegram");
  expect(phase8CNormalizedDepKinds(telegramNodeDependencies[0])).toEqual([
    { kind: "normal", target: null },
    { kind: "dev", target: null },
  ]);

  const packageNameById = new Map(
    metadata.packages.map(({ id, name }) => [id, name]),
  );
  const producerRoots = phase8CNode(metadata, producerPackage.id).deps.map(
    ({ pkg }) => packageNameById.get(pkg) ?? `missing:${pkg}`,
  ).sort();
  expect(producerRoots).toEqual([
    "base64",
    "chacha20poly1305",
    "extractum-core",
    "grammers-client",
    "grammers-mtsender",
    "grammers-session",
    "grammers-tl-types",
    "rand_core",
    "secrecy",
    "serde",
    "serde_json",
    "tokio",
  ]);
  const forbiddenAppRoots = new Set([
    "grammers-client",
    "grammers-mtsender",
    "grammers-session",
    "grammers-tl-types",
    "chacha20poly1305",
    "rand_core",
  ]);
  expect(appNode.deps.flatMap(({ pkg }) => {
    const name = packageNameById.get(pkg);
    return name && forbiddenAppRoots.has(name) ? [name] : [];
  })).toEqual([]);
  expect(
    phase8CNode(metadata, producerPackage.id).deps.some(
      ({ pkg }) => pkg === appPackage.id,
    ),
  ).toBe(false);

  expect(
    createHash("sha256")
      .update(tomlSection(rootCargo, "workspace.dependencies"))
      .digest("hex"),
  ).toBe("9a494a667ddc7d8c440de23e3163084f98fb76ea167ea3e62438ef5e62736450");
}

function assertPhase8CGrammersClosure(
  metadata: Phase8CCargoMetadata,
): void {
  const authority = grammersFeatureBaseline as unknown as {
    revision: string;
    packages: Array<{
      name: string;
      required: string[];
      forbidden: string[];
      universe: string[];
    }>;
  };
  const producer = phase8CWorkspacePackage(metadata, "extractum-telegram");
  const directPackageIds = new Set(
    phase8CNode(metadata, producer.id).deps.map(({ pkg }) => pkg),
  );
  exactInventory(
    [...directPackageIds].flatMap((packageId) => {
      const name = phase8CPackageById(metadata, packageId).name;
      return name.startsWith("grammers-") ? [name] : [];
    }),
    authority.packages.map(({ name }) => name),
    "Phase 8C direct Grammers roots",
  );

  for (const expected of authority.packages) {
    const packages = metadata.packages.filter(({ name }) => name === expected.name);
    expect(packages, expected.name).toHaveLength(1);
    const selected = packages[0];
    expect(selected.source).toBe(
      `git+https://codeberg.org/Lonami/grammers?rev=${authority.revision}#${authority.revision}`,
    );
    expect(Object.keys(selected.features).sort()).toEqual(
      [...expected.universe].sort(),
    );
    const enabled = [...phase8CNode(metadata, selected.id).features].sort();
    expect(enabled).toEqual([...expected.required].sort());
    expect(enabled.filter((feature) => expected.forbidden.includes(feature)))
      .toEqual([]);
  }
}

function assertPhase8CGraphCut(metadata: Phase8CCargoMetadata): void {
  const app = phase8CWorkspacePackage(metadata, "extractum");
  const producer = phase8CWorkspacePackage(metadata, "extractum-telegram");
  const nodes = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
  const reachable = new Set<string>();
  const queue = [app.id];
  while (queue.length !== 0) {
    const current = queue.shift();
    if (!current || reachable.has(current)) continue;
    reachable.add(current);
    const node = nodes.get(current);
    if (!node) throw new Error(`Missing graph node: ${current}`);
    for (const dependency of node.deps) {
      if (current === app.id && dependency.pkg === producer.id) continue;
      queue.push(dependency.pkg);
    }
  }
  const grammersLeaks = metadata.packages.flatMap(({ id, name }) =>
    reachable.has(id) && name.startsWith("grammers-") ? [name] : []
  ).sort();
  expect(grammersLeaks).toEqual([]);
  expect(
    phase8CNode(metadata, producer.id).deps.some(({ pkg }) => pkg === app.id),
  ).toBe(false);
}

function assertPhase8CFixtureBoundary(
  producerSources: readonly Phase8CNamedSource[],
): void {
  const sourceByPath = new Map(
    producerSources.map(({ relativePath, source }) => [relativePath, source]),
  );
  const libPath = "src-tauri/crates/extractum-telegram/src/lib.rs";
  const takeoutPath =
    "src-tauri/crates/extractum-telegram/src/takeout/mod.rs";
  const lib = sourceByPath.get(libPath);
  const takeout = sourceByPath.get(takeoutPath);
  if (!lib || !takeout) throw new Error("Missing fixture-boundary source");
  const snippets = [
    [libPath, lib, '#[cfg(feature = "app-test-support")]\npub use takeout::attempt_fixture as takeout_attempt_fixture;'],
    [libPath, lib, '#[cfg(feature = "app-test-support")]\npub use takeout::fallback_fixture as takeout_fallback_fixture;'],
    [takeoutPath, takeout, '#[cfg(feature = "app-test-support")]\npub fn fallback_fixture('],
    [takeoutPath, takeout, '#[cfg(feature = "app-test-support")]\npub fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {'],
  ] as const;
  for (const [, source, snippet] of snippets) {
    expect(source.split(snippet).length - 1, snippet).toBe(1);
  }

  const residualSources = producerSources.map((entry) => ({
    ...entry,
    source: snippets.reduce(
      (source, [relativePath, , snippet]) =>
        relativePath === entry.relativePath
          ? source.replace(snippet, "")
          : source,
      entry.source,
    ),
  }));
  const residualFixtureIdentifiers = residualSources.flatMap(
    ({ relativePath, source }) =>
      [...maskRustLexicalNonCode(source).matchAll(
        /\b[A-Za-z_][A-Za-z0-9_]*fixture\b/g,
      )].map((match) => `${relativePath}|${match[0]}`),
  );
  exactInventory(
    residualFixtureIdentifiers,
    [],
    "Phase 8C producer fixture references outside the four declarations",
  );
}

function phase8CProducerTestIds(
  producerSources: readonly Phase8CNamedSource[],
): string[] {
  const cratePrefix = "src-tauri/crates/extractum-telegram/src/";
  return producerSources.flatMap(({ relativePath, source }) => {
    if (!relativePath.startsWith(cratePrefix)) {
      throw new Error(`Producer test source escaped crate: ${relativePath}`);
    }
    const cratePath = relativePath.slice(cratePrefix.length);
    const modulePath = cratePath === "lib.rs"
      ? ""
      : cratePath.endsWith("/mod.rs")
      ? cratePath.slice(0, -"/mod.rs".length).replaceAll("/", "::")
      : cratePath.slice(0, -".rs".length).replaceAll("/", "::");
    const searchable = maskRustLexicalNonCode(source);
    return [
      ...searchable.matchAll(
        /#\[(?:tokio::)?test(?:\s*\([^\]]*\))?\]\s*(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/g,
      ),
    ].map((match) =>
      `${modulePath ? `${modulePath}::` : ""}tests::${match[1]}`
    );
  });
}
```

Add these three remaining required tests after the helper block:

```ts
it("allows exactly two feature-gated fixture names and no producer test consumer", () => {
  const producerSources = phase8CNamedRustSources(
    rustPathsUnder("src-tauri/crates/extractum-telegram/src"),
  );
  assertPhase8CFixtureBoundary(producerSources);

  const mutate = (relativePath: string, before: string, after: string) =>
    producerSources.map((entry) => entry.relativePath === relativePath
      ? {
          ...entry,
          source: exactMutation(
            entry.source,
            before,
            after,
            `Phase 8C fixture mutation ${relativePath}`,
          ),
        }
      : entry);
  expect(() => assertPhase8CFixtureBoundary(mutate(
    "src-tauri/crates/extractum-telegram/src/lib.rs",
    '#[cfg(feature = "app-test-support")]\npub use takeout::attempt_fixture',
    "#[cfg(test)]\npub use takeout::attempt_fixture",
  ))).toThrow();
  expect(() => assertPhase8CFixtureBoundary(mutate(
    "src-tauri/crates/extractum-telegram/src/takeout/mod.rs",
    "pub fn fallback_fixture(",
    "pub(crate) fn fallback_fixture(",
  ))).toThrow();
  expect(() => assertPhase8CFixtureBoundary([
    ...producerSources,
    {
      relativePath: "src-tauri/crates/extractum-telegram/src/third.rs",
      source: '#[cfg(feature = "app-test-support")]\npub fn third_fixture() {}\n',
    },
  ])).toThrow();
  expect(() => assertPhase8CFixtureBoundary(
    producerSources.map((entry) => entry.relativePath.endsWith("/dto.rs")
      ? { ...entry, source: `${entry.source}\nfn producer_test_leak() { let _ = takeout_attempt_fixture; }\n` }
      : entry),
  )).toThrow();
});

it("cuts every app-to-Grammers path when the producer edge is removed", () => {
  const metadata = phase8CLockedMetadata();
  assertPhase8CGraphCut(metadata);

  const mutated = phase8CCloneMetadata(metadata) as {
    packages: Phase8CMetadataPackage[];
    workspace_members: string[];
    resolve: { nodes: Array<{
      id: string;
      features: string[];
      deps: Phase8CMetadataNodeDep[];
    }> };
  };
  const app = phase8CWorkspacePackage(mutated, "extractum");
  const grammers = mutated.packages.find(
    ({ name }) => name === "grammers-client",
  );
  const appNode = mutated.resolve.nodes.find(({ id }) => id === app.id);
  if (!grammers || !appNode) throw new Error("Missing graph mutation node");
  appNode.deps.push({
    name: "grammers_client",
    pkg: grammers.id,
    dep_kinds: [{ kind: null, target: null }],
  });
  expect(() => assertPhase8CGraphCut(mutated)).toThrow();
});

it("moves all 71 future-owner identities exactly once", () => {
  const authority = readGeneratedJson<Phase8CIdentityAuthority>(
    "src/lib/telegram-8b-test-identities.json",
  );
  const expected = [
    ...authority.preNewStaged,
    ...authority.phase8BNewStaged,
  ].map((identity) => identity.replace(/^telegram_impl::/, "")).sort();
  expect(expected).toHaveLength(71);
  const producerSources = phase8CNamedRustSources(
    rustPathsUnder("src-tauri/crates/extractum-telegram/src"),
  );
  exactInventory(
    phase8CProducerTestIds(producerSources),
    expected,
    "Phase 8C producer test identity declarations",
  );

  const duplicated = producerSources.map((entry) =>
    entry.relativePath.endsWith("/dto.rs")
      ? {
          ...entry,
          source: `${entry.source}\n#[test]\nfn telegram_item_kind_constant_matches_persisted_wire_value() {}\n`,
        }
      : entry
  );
  expect(() => exactInventory(
    phase8CProducerTestIds(duplicated),
    expected,
    "mutated Phase 8C producer test identities",
  )).toThrow();
});
```

The retained negative mutations cover physical lifecycle, dependency kinds, fixture declarations/references, Grammers feature closure, graph cut, facade-only source perimeter, and 71-test identity uniqueness. Exact manifest feature mentions and duplicate-dependency use remain direct standing assertions without a second activation model or frozen file inventory.

- [ ] **Step 5: Teach all cross-crate contracts the seventh-member final state without weakening their own boundaries**

Apply the following complete replacements; all omitted assertions remain unchanged.

In `rust-workspace-core-contract.test.ts`, add the detector after `analysisCrateExtracted`, replace the workspace/edge assertion body shown below, and append the two core exclusions beside the existing analysis exclusions:

```ts
const telegramCrateExtracted = existsSync(
  path.join(repoRoot, "src-tauri/crates/extractum-telegram/Cargo.toml"),
);

const expectedMembers = [
  ".",
  "crates/extractum-core",
  "crates/extractum-gemini-browser",
  "crates/extractum-llm",
  ...(promptPackCrateExtracted ? ["crates/extractum-prompt-packs"] : []),
  ...(analysisCrateExtracted ? ["crates/extractum-analysis"] : []),
  ...(telegramCrateExtracted ? ["crates/extractum-telegram"] : []),
];
expect(members).toEqual(expectedMembers);
expect(members).toHaveLength(telegramCrateExtracted ? 7 : 6);
expect(
  tomlSection(rootCargo, "dependencies").match(
    /^extractum-analysis = \{ path = "crates\/extractum-analysis" \}$/gm,
  ) ?? [],
).toHaveLength(analysisCrateExtracted ? 1 : 0);
expect(
  rootCargo
    .split("\n")
    .filter((line) => line.includes("extractum-analysis")),
).toHaveLength(analysisCrateExtracted ? 2 : 0);
expect(
  tomlSection(rootCargo, "dependencies").match(
    /^extractum-telegram = \{ path = "crates\/extractum-telegram" \}$/gm,
  ) ?? [],
).toHaveLength(telegramCrateExtracted ? 1 : 0);
expect(
  tomlSection(rootCargo, "dev-dependencies").match(
    /^extractum-telegram\s*=\s*\{\s*path\s*=\s*"crates\/extractum-telegram"\s*,\s*features\s*=\s*\["app-test-support"\]\s*,?\s*\}$/gm,
  ) ?? [],
).toHaveLength(telegramCrateExtracted ? 1 : 0);
expect(
  lockPackage(cargoLock, "extractum").match(
    /^ "extractum-telegram(?: [^"]+)?",?$/gm,
  ) ?? [],
).toHaveLength(telegramCrateExtracted ? 1 : 0);

expect(coreCargo).not.toContain("extractum-telegram");
expect(
  lockPackage(cargoLock, "extractum-core").match(
    /^ "extractum-telegram(?: [^"]+)?",?$/gm,
  ) ?? [],
).toHaveLength(0);
```

In `gemini-browser-crate-boundary-contract.test.ts`, add this detector immediately after `analysisCrateExtracted`:

```ts
const telegramCrateExtracted = existsSync(
  path.join(repoRoot, "src-tauri/crates/extractum-telegram/Cargo.toml"),
);
```

Then replace the workspace assertions (including deletion of `expect(rootCargo).not.toContain("extractum-telegram")`) and append the foreign-package assertions:

```ts
expect(members).toEqual([
  ".",
  "crates/extractum-core",
  "crates/extractum-gemini-browser",
  "crates/extractum-llm",
  ...(promptPackCrateExtracted ? ["crates/extractum-prompt-packs"] : []),
  ...(analysisCrateExtracted ? ["crates/extractum-analysis"] : []),
  ...(telegramCrateExtracted ? ["crates/extractum-telegram"] : []),
]);
expect(members).toHaveLength(telegramCrateExtracted ? 7 : 6);
expect(crateCargo).not.toContain("extractum-telegram");
expect(
  lockDependencies(lockPackage(cargoLock, "extractum-gemini-browser")),
).not.toContain("extractum-telegram");
```

In `llm-crate-boundary-contract.test.ts`, add this detector immediately after `analysisCrateExtracted`:

```ts
const telegramCrateExtracted = existsSync(
  path.join(repoRoot, "src-tauri/crates/extractum-telegram/Cargo.toml"),
);
```

Then replace the workspace assertion. Retain the existing `for (const name of forbiddenCrateDependencyNames)` body byte-for-byte; its current list already forbids `extractum-telegram` and all four Grammers packages.

```ts
expect(members).toEqual([
  ".",
  "crates/extractum-core",
  "crates/extractum-gemini-browser",
  "crates/extractum-llm",
  ...(promptPackCrateExtracted ? ["crates/extractum-prompt-packs"] : []),
  ...(analysisCrateExtracted ? ["crates/extractum-analysis"] : []),
  ...(telegramCrateExtracted ? ["crates/extractum-telegram"] : []),
]);
expect(members).toHaveLength(telegramCrateExtracted ? 7 : 6);
for (const name of forbiddenCrateDependencyNames) {
  expect([
    ...dependencyNames(tomlSection(crateCargo, "dependencies")),
    ...dependencyNames(tomlSection(crateCargo, "dev-dependencies")),
  ]).not.toContain(name);
}
```

In `prompt-pack-crate-boundary-contract.test.ts`, use its actual root identifier, add this detector after `analysisCrateExtracted`, replace the workspace assertion, append the foreign assertions after the existing analysis exclusions, and replace the lower-package loop:

```ts
const telegramCrateExtracted = existsSync(
  path.join(
    repositoryRoot,
    "src-tauri/crates/extractum-telegram/Cargo.toml",
  ),
);

expect(members).toEqual([
  ".",
  "crates/extractum-core",
  "crates/extractum-gemini-browser",
  "crates/extractum-llm",
  "crates/extractum-prompt-packs",
  ...(analysisCrateExtracted ? ["crates/extractum-analysis"] : []),
  ...(telegramCrateExtracted ? ["crates/extractum-telegram"] : []),
]);
expect(members).toHaveLength(telegramCrateExtracted ? 7 : 6);
expect(crateCargo).not.toContain("extractum-telegram");
expect(lockDependencies(cratePackages[0])).not.toContain(
  "extractum-telegram",
);
for (const lower of [
  "extractum-core",
  "extractum-gemini-browser",
  "extractum-llm",
  ...(telegramCrateExtracted ? ["extractum-telegram"] : []),
]) {
  const packages = lockPackages(cargoLock, lower);
  expect(packages, lower).toHaveLength(1);
  expect(lockDependencies(packages[0]), lower).not.toContain(
    "extractum-prompt-packs",
  );
  expect(
    read(`src-tauri/crates/${lower}/Cargo.toml`),
    lower,
  ).not.toContain("extractum-prompt-packs");
}
```

In `analysis-crate-boundary-contract.test.ts`, add the detector after `const extracted`, replace the workspace block, append the foreign loop after `analysisLock` is created, and add the retained predicates without renaming the existing staged predicates used by graph-breadth checks:

```ts
const telegramCrateExtracted = existsSync(
  path.join(repoRoot, "src-tauri/crates/extractum-telegram/Cargo.toml"),
);

const expectedMembers = [
  ".",
  "crates/extractum-core",
  "crates/extractum-gemini-browser",
  "crates/extractum-llm",
  "crates/extractum-prompt-packs",
  ...(extracted ? ["crates/extractum-analysis"] : []),
  ...(telegramCrateExtracted ? ["crates/extractum-telegram"] : []),
];
expect(workspaceMembers(rootCargo)).toEqual(expectedMembers);
expect(expectedMembers).toHaveLength(telegramCrateExtracted ? 7 : 6);

for (const foreignDependency of [
  "extractum-telegram",
  "grammers-client",
  "grammers-mtsender",
  "grammers-session",
  "grammers-tl-types",
]) {
  expect(
    read("src-tauri/crates/extractum-analysis/Cargo.toml"),
  ).not.toContain(foreignDependency);
  expect(lockDependencies(analysisLock)).not.toContain(foreignDependency);
}

const telegramPeerAvatarBoundaryIsRetained =
  telegramPeerAvatarBoundaryIsStaged
  || existsSync(
    path.join(
      repoRoot,
      "src-tauri/crates/extractum-telegram/src/live/peer.rs",
    ),
  );
const telegramTakeoutBoundaryIsRetained =
  telegramTakeoutBoundaryIsStaged
  || existsSync(
    path.join(
      repoRoot,
      "src-tauri/crates/extractum-telegram/src/takeout/mod.rs",
    ),
  );
const sourcesStoreSourceFingerprint = telegramTakeoutBoundaryIsRetained
  ? "340890b2099e76c9da00e7c52d1746c13a25876b3d9a9580025c5f6601e3f7bb"
  : telegramPeerAvatarBoundaryIsRetained
  ? "01f4773cdc95e94f83c14b21efd0ae62976c3326895d0938925f6ab36c5d6167"
  : "28a8eb092ae42884e8b06bd6681bc36c8378cb1facea61fd8076b6ed284e3895";
```

Place each retained predicate immediately after its corresponding staged predicate, and replace only the existing fingerprint selector; keep graph breadth and required-path logic on `...IsStaged`.

In `analysis-application-contract.test.ts`, replace the three predicates/reachability branch and the two selectors below; do not change any fingerprint literal:

```ts
const telegramFacadeIsRetained = existsSync(
  path.join(appSourceRoot, "telegram_impl/lib.rs"),
);
const telegramPeerAvatarBoundaryIsRetained =
  existsSync(path.join(appSourceRoot, "telegram_impl/live/peer.rs"))
  || existsSync(
    path.join(
      repoRoot,
      "src-tauri/crates/extractum-telegram/src/live/peer.rs",
    ),
  );
const telegramTakeoutBoundaryIsRetained =
  existsSync(path.join(appSourceRoot, "telegram_impl/takeout/mod.rs"))
  || existsSync(
    path.join(
      repoRoot,
      "src-tauri/crates/extractum-telegram/src/takeout/mod.rs",
    ),
  );
const productionAppPaths = appReachability.production
  .map(({ relative }) => relative);
if (telegramFacadeIsRetained) {
  expect(
    productionAppPaths,
    "the retained Telegram facade must be production-reachable when present",
  ).toContain("telegram_impl/lib.rs");
} else {
  expect(productionAppPaths).not.toContain("telegram_impl/lib.rs");
}

const ingestProvenanceSourceFingerprint = telegramFacadeIsRetained
  ? "3f64c972ebc82996e65a396054ec8d16e73dc5fa911b92b9b10b79ed100b4a29"
  : "a327cabba5f1ab4f3af5c2f405ccb56a4500dc2bc736d8dc33a220f128323bde";
const sourcesStoreSourceFingerprint = telegramTakeoutBoundaryIsRetained
  ? "f808fff21440a539658b600440dfaa8168007fba11756326fa557a1fc6b0e70b"
  : telegramPeerAvatarBoundaryIsRetained
  ? "c752a11642888a3c45df0c53587588e3a7603b584de1ca19e2a3c8edc025b3e9"
  : "e510e682120b6566460fddf405f6c45d31ee7e96b399948fe91b75a51fa4a92d";
```

Every contract continues to accept its historical pre-Telegram state only when the Telegram crate manifest is absent and fails a partial workspace list.

- [ ] **Step 6: Admit the bounded 8C authority and both pre-terminal/final documentation states in shell-cap**

Add these imports immediately after the parent Telegram design import:

```ts
import telegram8CDesignRaw from "../../docs/superpowers/specs/2026-08-01-telegram-8c-extraction-design.md?raw";
import telegram8CPlanRaw from "../../docs/superpowers/plans/2026-08-01-extractum-telegram-8c-extraction.md?raw";
```

Add these normalized authorities immediately after `telegramBoundaryDesign`:

```ts
const telegram8CDesign = compact(telegram8CDesignRaw);
const telegram8CPlan = compact(telegram8CPlanRaw);
const telegram8CVerificationPath = path.join(
  repoRoot,
  "docs/superpowers/verification/2026-08-01-extractum-telegram-8c-extraction.md",
);
const telegram8CVerification = existsSync(telegram8CVerificationPath)
  ? compact(readFileSync(telegram8CVerificationPath, "utf8"))
  : undefined;
const phase8TerminalVerificationLink =
  "[verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md)";
```

Add this closed assertion helper before the Phase 8 test:

```ts
function assertPhase8CBoundedAuthority(): void {
  expect(phase8Status).toBeDefined();
  expect([
    "8B preparation retained; 8C pending",
    "done: retained",
  ]).toContain(phase8Status);

  const phase8Terminal = phase8Status === "done: retained";
  if (phase8Terminal) {
    const terminalStatus =
      `**Status:** Implemented and retained; ${phase8TerminalVerificationLink}`;
    expect(telegramBoundaryDesign).toContain(terminalStatus);
    expect(telegram8CDesign).toContain(terminalStatus);
    expect(telegram8CVerification).toBeDefined();
    expect(telegram8CVerification).toContain(
      "**Result: implemented and retained.**",
    );
  } else {
    expect(telegramBoundaryDesign).toContain(
      "**Status:** Approved; 8B preparation retained; 8C pending",
    );
    expect(telegram8CDesign).toContain(
      "**Status:** Approved for implementation planning; implementation not authorized",
    );
    expect(telegram8CVerification).toBeUndefined();
  }

  expect(compact(crateRoadmap)).not.toContain(
    "implementation has not started",
  );
  expect(telegramBoundaryDesign).toContain(
    "[`2026-08-01-telegram-8c-extraction-design.md`](2026-08-01-telegram-8c-extraction-design.md)",
  );
  expect(telegram8CDesign).toContain(
    "[`2026-07-26-telegram-crate-boundary-design.md`](2026-07-26-telegram-crate-boundary-design.md)",
  );
  expect(telegram8CDesign).toContain(
    "Use one forward-only, atomic Phase 8C implementation checkpoint.",
  );
  expect(telegram8CDesign).toContain(
    "The single retained implementation checkpoint is `EXTRACTION_COMMIT`.",
  );
  expect(telegram8CPlan).toContain(
    "Phase 8C has exactly one implementation commit, named `EXTRACTION_COMMIT`.",
  );
  expect(telegram8CDesign).toContain(
    "The feature is declared only by `extractum-telegram` and is non-default. The normal app dependency remains feature-free; only the app dev-dependency enables it.",
  );
  expect(telegram8CDesign).toContain(
    "The contract derives the extracted state from exact package, path, and facade preconditions and must pass before terminal roadmap/design statuses are updated. Partial layouts fail closed; terminal status is not a prerequisite for final GREEN.",
  );
  expect(telegram8CDesign).toContain(
    "The required order is final uncommitted `npm.cmd run verify` -> release command above -> `EXTRACTION_COMMIT` -> live MCP -> self-managed startup smoke -> docs-only terminal commit.",
  );
  expect(telegram8CPlan).toContain(
    "The terminal order is fixed: final uncommitted `npm.cmd run verify` -> release build with explicit host target -> `EXTRACTION_COMMIT` -> one live MCP smoke -> self-managed startup smoke -> docs-only terminal evidence commit.",
  );
  expect(telegram8CDesign).toContain(
    "The counts must still be 665 app / 71 crate / 736 union.",
  );
  expect(phase8Roadmap).toContain(
    "2026-07-26-telegram-crate-boundary-design.md",
  );
}
```

Rename the existing test and insert the helper call at its top:

```diff
-  it("records the retained Phase 8 Telegram preparation disposition", () => {
+  it("records the retained Phase 8 authority and bounded 8C lifecycle", () => {
+    assertPhase8CBoundedAuthority();
```

Keep the historical status assertions, but make their three terminal-sensitive portions exact with these edits. The final hunk applies only to the shorter second `phase8Status` array (the longer first array already contains `done: retained`):

```diff
         "**Status:** Approved; 8B preparation retained; 8C pending",
+        "**Status:** Implemented and retained;",
       ].some((status) => telegramBoundaryDesign.includes(status)),
@@
-    expect(
+    if (phase8Status !== "done: retained") {
+      expect(
       [
@@
-    ).toBe(true);
+      ).toBe(true);
+    }
@@
       "8B preparation retained; 8C pending",
+      "done: retained",
     ]).toContain(phase8Status);
```

Replace only the seven obsolete parent assertions later in that same test with this exact diff:

```diff
-    expect(phase8Roadmap).toContain(
-      "8C preserves staged files byte-for-byte",
-    );
+    expect(telegram8CDesign).toContain(
+      "For 17 relative paths other than `lib.rs` and `takeout/mod.rs`, completion evidence requires:",
+    );
@@
-    expect(phase8Roadmap).toContain(
-      "8C makes no visibility change",
-    );
+    expect(telegram8CDesign).toContain(
+      "The remaining two destination files may differ from their frozen source only by this exact patch:",
+    );
@@
-    expect(telegramBoundaryDesign).toContain(
-      "No app dev dependency, reverse dependency, or fixture crate is introduced",
-    );
+    expect(telegram8CDesign).toContain(
+      "The normal app dependency remains feature-free; only the app dev-dependency enables it.",
+    );
@@
-    expect(telegramBoundaryDesign).toContain(
-      "The move preserves every staged source file byte-for-byte",
-    );
+    expect(telegram8CDesign).toContain(
+      "For 17 relative paths other than `lib.rs` and `takeout/mod.rs`, completion evidence requires:",
+    );
@@
-    expect(telegramBoundaryDesign).toContain(
-      "requires exact path and byte-hash equality",
-    );
+    expect(telegram8CDesign).toContain(
+      "Excluding diff headers, the two diffs together may contain only the eight line substitutions in the table:",
+    );
@@
-    expect(telegramStagingContract).toContain(
-      "Inside the moved tree, no module path, import, visibility, or source text changes in 8C",
-    );
+    expect(telegram8CDesign).toContain(
+      "function bodies, signatures, names, order, imports, and all other bytes remain unchanged.",
+    );
@@
-    expect(telegramVisibilityAllowlist).toContain(
-      "No existing free function is widened in place",
-    );
-    expect(telegramVisibilityAllowlist).toContain(
-      "8C changes no visibility",
-    );
+    expect(telegram8CDesign).toContain(
+      "The production API is exactly the curated parent-design API already prepared by 8B. Phase 8C widens no production item and changes no signature.",
+    );
+    expect(telegram8CDesign).toContain(
+      "The only additional externally reachable names are the two functions under `app-test-support`.",
+    );
```

- [ ] **Step 7: Run the complete coupled final-layout contract set once**

```powershell
$contractCapture = Invoke-CheckedNative 'Phase 8C coupled final-layout contracts' {
    npm.cmd run test -- `
        src/lib/telegram-crate-boundary-contract.test.ts `
        src/lib/crate-extraction-shell-cap-contract.test.ts `
        src/lib/rust-workspace-core-contract.test.ts `
        src/lib/gemini-browser-crate-boundary-contract.test.ts `
        src/lib/llm-crate-boundary-contract.test.ts `
        src/lib/prompt-pack-crate-boundary-contract.test.ts `
        src/lib/analysis-crate-boundary-contract.test.ts `
        src/lib/analysis-application-contract.test.ts
}
$contractCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'final-layout-contracts.txt')
```

Expected: exit 0 on the final physical layout while Phase 8 documentation still says pending. This is the first boundary-contract run after Task 2 RED.

### Task 5: Prove Exact Source, Test, Dependency, and Staged-Index Identity

**Files:**
- Read: the complete final implementation diff
- Read: `src/lib/telegram-8b-staging-sha256.json`
- Read: `src/lib/telegram-8b-test-identities.json`
- Read: `src/lib/telegram-grammers-feature-baseline.json`
- Stage: only the implementation allowlist from Tasks 2–4

**Interfaces:**
- Consumes: `$BASE_COMMIT`, `$Frozen8B`, `$relativePaths`, `$EXPECTED_APP_TESTS`, `$EXPECTED_CRATE_TESTS`, and the final GREEN tree.
- Produces: a closed staged checkpoint tree, exact 17 blob equalities, two complete approved diffs, exact 665/71/736 sets, and locked dependency evidence.

- [ ] **Step 1: Recheck immutable authorities and capture final locked metadata**

```powershell
$immutableAuthorityPaths = @(
    'src/lib/telegram-grammers-feature-baseline.json',
    'src/lib/telegram-8b-test-identities.json',
    'src/lib/telegram-8b-symbol-map.json',
    'src/lib/telegram-8b-staging-sha256.json'
)
foreach ($authorityPath in $immutableAuthorityPaths) {
    $actual = (Get-FileHash -LiteralPath $authorityPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expectedHashes[$authorityPath]) {
        throw "Immutable authority drift for ${authorityPath}: $actual"
    }
}

$finalMetadataCapture = Invoke-CheckedNative 'final locked Cargo metadata' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
}
$finalMetadataCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'final-cargo-metadata.json')
Invoke-CheckedNative 'final Grammers feature authority' {
    node scripts/telegram-grammers-feature-baseline.mjs --check
} | Out-Null
```

Expected: four hashes unchanged, locked metadata exit 0, and the generated feature artifact remains byte-identical. Do not run the staging generator against the final layout.

- [ ] **Step 2: Capture and compare the exact final library-test sets**

```powershell
$FINAL_APP_TESTS = @(Get-RustTestIds -Package 'extractum')
$FINAL_CRATE_TESTS = @(Get-RustTestIds -Package 'extractum-telegram')
Assert-ExactStringSet -Label 'final app tests' `
    -Expected $EXPECTED_APP_TESTS -Actual $FINAL_APP_TESTS
Assert-ExactStringSet -Label 'final Telegram crate tests' `
    -Expected $EXPECTED_CRATE_TESTS -Actual $FINAL_CRATE_TESTS

$FINAL_LOGICAL_TESTS = @(
    $FINAL_APP_TESTS
    $FINAL_CRATE_TESTS | ForEach-Object { "telegram_impl::$_" }
)
Assert-ExactStringSet -Label 'final logical workspace test union' `
    -Expected $BASE_TESTS -Actual $FINAL_LOGICAL_TESTS
if ($FINAL_APP_TESTS.Count -ne 665) { throw "Final app test count is $($FINAL_APP_TESTS.Count), expected 665" }
if ($FINAL_CRATE_TESTS.Count -ne 71) { throw "Final crate test count is $($FINAL_CRATE_TESTS.Count), expected 71" }
if ($FINAL_LOGICAL_TESTS.Count -ne 736) { throw "Final logical union is $($FINAL_LOGICAL_TESTS.Count), expected 736" }

$FINAL_APP_TESTS | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'final-app-tests.txt')
$FINAL_CRATE_TESTS | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'final-telegram-crate-tests.txt')
$FINAL_LOGICAL_TESTS | Sort-Object |
    Set-Content -LiteralPath (Join-Path $EvidenceRoot 'final-logical-tests.txt')
```

Expected: exact set equality, not count-only evidence: 665 app, 71 producer, and the same 736 logical identities captured at `BASE_COMMIT`.

- [ ] **Step 3: Stage only the closed implementation allowlist and reject every extra path**

```powershell
git add -- `
    scripts/telegram-grammers-feature-baseline.mjs `
    src-tauri/Cargo.toml `
    src-tauri/Cargo.lock `
    src-tauri/src/telegram_impl `
    src-tauri/crates/extractum-telegram `
    src/lib/telegram-crate-boundary-contract.test.ts `
    src/lib/crate-extraction-shell-cap-contract.test.ts `
    src/lib/rust-workspace-core-contract.test.ts `
    src/lib/gemini-browser-crate-boundary-contract.test.ts `
    src/lib/llm-crate-boundary-contract.test.ts `
    src/lib/prompt-pack-crate-boundary-contract.test.ts `
    src/lib/analysis-crate-boundary-contract.test.ts `
    src/lib/analysis-application-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Failed to stage the Phase 8C implementation allowlist' }

$actualImplementationPaths = @(
    git diff --cached --no-renames --name-only --diff-filter=ACDMRTUXB |
        Sort-Object -Unique
)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate staged implementation paths' }
Assert-ExactStringSet -Label 'staged implementation path allowlist' `
    -Expected $expectedImplementationPaths -Actual $actualImplementationPaths

git diff --quiet
if ($LASTEXITCODE -eq 1) { throw 'Tracked unstaged changes exist after closed staging' }
if ($LASTEXITCODE -gt 1) { throw 'git diff --quiet failed' }
$unexpectedUntracked = @(git ls-files --others --exclude-standard)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect untracked files' }
if ($unexpectedUntracked.Count -ne 0) { $unexpectedUntracked; throw 'Unexpected untracked files exist' }

$CHECKPOINT_TREE = (git write-tree).Trim()
if ($LASTEXITCODE -ne 0 -or $CHECKPOINT_TREE -notmatch '^[a-f0-9]{40}$') {
    throw 'Could not record final staged checkpoint tree'
}
Save-Phase8CState -Values @{ CHECKPOINT_TREE = $CHECKPOINT_TREE }
```

Expected: the staged diff is exactly the closed path set; there are no unstaged tracked or untracked files. Rename display is not used as authority because `--no-renames` exposes both old and new paths.

- [ ] **Step 4: Prove every sourced lock package remains byte-identical in Git blobs**

```powershell
function Get-SourcedLockBlocks {
    param([string]$LockText)
    $normalized = $LockText -replace "\r\n?", "`n"
    @(
        [regex]::Matches($normalized, '(?ms)^\[\[package\]\]\n.*?(?=^\[\[package\]\]|\z)') |
            ForEach-Object { $_.Value.TrimEnd() } |
            Where-Object { $_ -match '(?m)^source = ' } |
            Sort-Object
    )
}
$baseLockBlob = (git show "${BASE_COMMIT}:src-tauri/Cargo.lock" | Out-String)
if ($LASTEXITCODE -ne 0) { throw 'Could not read BASE Cargo.lock blob' }
$indexLockBlob = (git show ':src-tauri/Cargo.lock' | Out-String)
if ($LASTEXITCODE -ne 0) { throw 'Could not read indexed Cargo.lock blob' }
Assert-ExactStringSet -Label 'sourced Cargo.lock package blocks' `
    -Expected (Get-SourcedLockBlocks $baseLockBlob) `
    -Actual (Get-SourcedLockBlocks $indexLockBlob)
```

Expected: all registry/Git package blocks are unchanged. Only source-less workspace package dependency blocks may differ or be added. The standing final-layout TypeScript contract already proves the exact `[workspace.dependencies]` section; closed staging and the later commit-tree equality bind that result to the retained tree.

- [ ] **Step 5: Compare all 17 unchanged destination blobs against frozen 8B from the index**

```powershell
$correctedPaths = @('lib.rs', 'takeout/mod.rs')
$unchangedPaths = @($relativePaths | Where-Object { $_ -notin $correctedPaths })
if ($unchangedPaths.Count -ne 17) { throw "Expected 17 unchanged paths, found $($unchangedPaths.Count)" }

$indexedBlobRows = foreach ($relativePath in $unchangedPaths) {
    $frozenBlob = (git rev-parse "${Frozen8B}:src-tauri/src/telegram_impl/$relativePath").Trim()
    if ($LASTEXITCODE -ne 0) { throw "Missing frozen source blob for $relativePath" }
    $indexedBlob = (git rev-parse ":src-tauri/crates/extractum-telegram/src/$relativePath").Trim()
    if ($LASTEXITCODE -ne 0) { throw "Missing indexed destination blob for $relativePath" }
    if ($frozenBlob -ne $indexedBlob) {
        throw "Indexed destination blob drift for ${relativePath}: $indexedBlob != $frozenBlob"
    }
    "$relativePath $frozenBlob $indexedBlob"
}
$indexedBlobRows | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'indexed-unchanged-blobs.txt')
Save-Phase8CState -Values @{
    correctedPaths = @($correctedPaths)
    unchangedPaths = @($unchangedPaths)
}
```

Expected: exactly 17 rows with equal source/destination blob IDs.

- [ ] **Step 6: Record both complete corrected-file diffs and accept only the eight substitutions**

```powershell
$expectedCorrectedDiffs = [ordered]@{
    'lib.rs' = @{
        Removed = @(
            '#[cfg(test)]',
            'pub(crate) use takeout::attempt_fixture as takeout_attempt_fixture;',
            '#[cfg(test)]',
            'pub(crate) use takeout::fallback_fixture as takeout_fallback_fixture;'
        )
        Added = @(
            '#[cfg(feature = "app-test-support")]',
            'pub use takeout::attempt_fixture as takeout_attempt_fixture;',
            '#[cfg(feature = "app-test-support")]',
            'pub use takeout::fallback_fixture as takeout_fallback_fixture;'
        )
        Hunks = @('@@ -24,4 +24,4 @@')
    }
    'takeout/mod.rs' = @{
        Removed = @(
            '#[cfg(test)]',
            'pub(crate) fn fallback_fixture(',
            '#[cfg(test)]',
            'pub(crate) fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {'
        )
        Added = @(
            '#[cfg(feature = "app-test-support")]',
            'pub fn fallback_fixture(',
            '#[cfg(feature = "app-test-support")]',
            'pub fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {'
        )
        Hunks = @('@@ -74,2 +74,2 @@', '@@ -87,2 +87,2 @@')
    }
}

$indexedCorrectedRows = foreach ($relativePath in $correctedPaths) {
    $sourceBlob = (git rev-parse "${Frozen8B}:src-tauri/src/telegram_impl/$relativePath").Trim()
    $destinationBlob = (git rev-parse ":src-tauri/crates/extractum-telegram/src/$relativePath").Trim()
    if ($LASTEXITCODE -ne 0) { throw "Could not resolve corrected blob pair for $relativePath" }
    $diffCapture = Invoke-CheckedNative "indexed corrected diff $relativePath" {
        git diff --no-ext-diff --unified=0 $sourceBlob $destinationBlob
    }
    $diffCapture.Text | Set-Content -LiteralPath (
        Join-Path $EvidenceRoot ("indexed-corrected-" + ($relativePath -replace '/', '-') + '.diff')
    )
    $expected = $expectedCorrectedDiffs[$relativePath]
    Assert-ExactDiffBody -Label $relativePath -Diff $diffCapture.Text `
        -ExpectedRemoved $expected.Removed -ExpectedAdded $expected.Added `
        -ExpectedHunkPrefixes $expected.Hunks
    "$relativePath $sourceBlob $destinationBlob"
}
$indexedCorrectedRows | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'indexed-corrected-blobs.txt')
Save-Phase8CState -Values @{
    expectedCorrectedDiffs = $expectedCorrectedDiffs
}
```

Expected: the complete two diffs contain exactly eight removed and eight added lines implementing the eight substitutions; all surrounding bytes are identical.

- [ ] **Step 7: Inspect the staged implementation summary and freeze its tree identity**

```powershell
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Staged implementation diff failed whitespace validation' }
git diff --cached --stat
git diff --cached -- src-tauri/Cargo.toml src-tauri/Cargo.lock `
    src-tauri/crates/extractum-telegram/Cargo.toml `
    scripts/telegram-grammers-feature-baseline.mjs
$treeNow = (git write-tree).Trim()
if ($treeNow -ne $CHECKPOINT_TREE) { throw 'Staged checkpoint tree changed during evidence checks' }
```

Expected: no unexpected dependency, artifact, source, or contract change. Keep the index staged for the complete terminal pre-retention sequence.

### Task 6: Run the Retained Pre-Retention Sequence and Create the Sole Implementation Commit

**Files:**
- Read/execute: the exact staged checkpoint tree
- Create ignored build output only: `src-tauri/target/<host>/release/extractum.exe`
- Commit: the exact Task 5 index

**Interfaces:**
- Consumes: `$BASE_COMMIT`, `$CHECKPOINT_TREE`, `$EvidenceRoot`, and the closed staged implementation index.
- Produces: passing full verification, one ordinary workspace-check timing, a host-qualified release executable/hash, and clean `EXTRACTION_COMMIT`.

- [ ] **Step 1: Install the recoverable pre-commit rollback procedure before running expensive gates**

If any Task 2–6 pre-commit assertion fails, first prove every dirty path belongs to `$expectedImplementationPaths`, then stash exactly those workflow-owned paths:

```powershell
$dirtyPaths = @(
    git status --porcelain=v1 --untracked-files=all |
        ForEach-Object { $_.Substring(3) -replace '^.* -> ', '' } |
        Sort-Object -Unique
)
$unexpectedDirty = @($dirtyPaths | Where-Object { $_ -notin $expectedImplementationPaths })
if ($unexpectedDirty.Count -ne 0) {
    $unexpectedDirty
    throw 'Rollback stopped because unrelated dirty paths exist'
}
if ($dirtyPaths.Count -eq 0) { throw 'Rollback requested with no workflow change to preserve' }
$rollbackMessage = "phase-8c-recoverable-$BASE_COMMIT-$(Get-Date -Format yyyyMMddHHmmss)"
git stash push --include-untracked -m $rollbackMessage
if ($LASTEXITCODE -ne 0) { throw 'Recoverable Phase 8C stash failed' }
$rollbackStash = (git rev-parse refs/stash).Trim()
if ($LASTEXITCODE -ne 0 -or $rollbackStash -notmatch '^[a-f0-9]{40}$') {
    throw 'Rollback stash ref was not retained'
}
if ((git rev-parse HEAD).Trim() -ne $BASE_COMMIT) { throw 'Rollback changed HEAD' }
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) {
    throw 'Rollback did not restore the clean BASE tree'
}
$topStash = git stash list -1
if ($topStash -notmatch [regex]::Escape($rollbackMessage)) {
    throw 'Top rollback stash does not carry the expected recovery label'
}
"rollback_stash=$rollbackStash message=$rollbackMessage"
```

Expected: the working tree returns clean at `BASE_COMMIT` and the complete change remains recoverable in one named stash. Never use `git reset --hard`, reconstruct files individually, or discard/drop the stash without explicit owner approval. If the failed command can be corrected without abandoning the slice, keep the working state and rerun the invalidated gates instead of invoking rollback.

- [ ] **Step 2: Run the retained standalone full gates, record the ordinary check, then run the final verifier**

```powershell
$rustfmtCapture = Invoke-CheckedNative 'retained Phase 8C rustfmt gate' {
    npm.cmd run check:rustfmt
}
$rustfmtCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'check-rustfmt.txt')

$gateMetadataCapture = Invoke-CheckedNative 'retained Phase 8C locked metadata gate' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
}
$gateMetadataCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'gate-cargo-metadata.json')

$workspaceCheckCapture = Invoke-CheckedNative 'retained Phase 8C ordinary workspace check' {
    cargo check --color never --manifest-path src-tauri/Cargo.toml --workspace --all-targets
}
$workspaceCheckCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'workspace-check.txt')
$workspaceFinished = @(
    $workspaceCheckCapture.Text -split "\r?\n" |
        Where-Object { $_ -match 'Finished .+ target\(s\) in ([0-9]+(?:\.[0-9]+)?)s' }
)
if ($workspaceFinished.Count -ne 1) {
    throw "Expected one ordinary workspace-check timing line, found $($workspaceFinished.Count)"
}
$null = $workspaceFinished[0] -match 'in ([0-9]+(?:\.[0-9]+)?)s'
$WORKSPACE_CHECK_DURATION_MS = [int][Math]::Round(
    [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture) * 1000
)
$workspaceFinished[0] | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'workspace-check-timing.txt')

$workspaceTestCapture = Invoke-CheckedNative 'retained Phase 8C workspace test gate' {
    cargo test --color never --manifest-path src-tauri/Cargo.toml --workspace --all-targets
}
$workspaceTestCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'workspace-test.txt')

$verifyCapture = Invoke-CheckedNative 'final uncommitted Phase 8C verifier' {
    npm.cmd run verify
}
$verifyCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'npm-verify.txt')

git diff --quiet
if ($LASTEXITCODE -ne 0) { throw 'A retained full gate changed a tracked checkpoint file' }
if ((git write-tree).Trim() -ne $CHECKPOINT_TREE) { throw 'A retained full gate changed the staged checkpoint tree' }
$unexpectedUntracked = @(git ls-files --others --exclude-standard)
if ($unexpectedUntracked.Count -ne 0) {
    $unexpectedUntracked
    throw 'A retained full gate created unexpected untracked files'
}
Save-Phase8CState -Values @{
    WORKSPACE_CHECK_DURATION_MS = $WORKSPACE_CHECK_DURATION_MS
}
```

Expected: standalone rustfmt/metadata/workspace check/workspace test all exit 0, `All verification checks passed.`, and exactly one retained timing value from the standalone workspace check. The verifier's internal workspace check is a duplicate correctness gate mandated by the existing workflow; ignore its duration.

- [ ] **Step 3: Derive the sole host target, run the exact release build, and bind its executable to the same tree**

```powershell
if (
    -not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR) -or
    -not [string]::IsNullOrWhiteSpace($env:CARGO_BUILD_TARGET)
) {
    throw 'CARGO_TARGET_DIR and CARGO_BUILD_TARGET must be unset for canonical release evidence'
}

$releaseMetadataCapture = Invoke-CheckedNative 'locked release target metadata' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 --no-deps
}
$metadataJsonStart = $releaseMetadataCapture.Text.IndexOf('{')
if ($metadataJsonStart -lt 0) { throw 'Cargo metadata emitted no JSON object' }
$releaseMetadata = $releaseMetadataCapture.Text.Substring($metadataJsonStart) | ConvertFrom-Json
$repositoryRoot = (Resolve-Path -LiteralPath '.').Path
$expectedTarget = [IO.Path]::GetFullPath((Join-Path $repositoryRoot 'src-tauri/target'))
$metadataTarget = [IO.Path]::GetFullPath([string]$releaseMetadata.target_directory)
if (-not [String]::Equals($metadataTarget, $expectedTarget, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Cargo target_directory is not canonical src-tauri/target: $metadataTarget"
}

$rustcCapture = Invoke-CheckedNative 'release host target' { rustc -vV }
$hostMatches = @([regex]::Matches($rustcCapture.Text, '(?m)^host:\s+([^\s]+)\s*$'))
if ($hostMatches.Count -ne 1) { throw 'rustc -vV did not emit exactly one host target' }
$hostTarget = $hostMatches[0].Groups[1].Value

$releaseCapture = Invoke-CheckedNative 'Phase 8C release build' {
    npm.cmd run tauri -- build --no-bundle --target $hostTarget
}
$releaseCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'release-build.txt')
$releaseExe = (Resolve-Path -LiteralPath (
    Join-Path $metadataTarget "$hostTarget/release/extractum.exe"
)).Path
$releaseHash = (Get-FileHash -LiteralPath $releaseExe -Algorithm SHA256).Hash.ToLowerInvariant()
"host_target=$hostTarget`nrelease_path=$releaseExe`nrelease_sha256=$releaseHash" |
    Set-Content -LiteralPath (Join-Path $EvidenceRoot 'release-identity.txt')

git diff --quiet
if ($LASTEXITCODE -ne 0) { throw 'Release build changed a tracked checkpoint file' }
if ((git write-tree).Trim() -ne $CHECKPOINT_TREE) { throw 'Release build changed the staged checkpoint tree' }
$unexpectedUntracked = @(git ls-files --others --exclude-standard)
if ($unexpectedUntracked.Count -ne 0) {
    $unexpectedUntracked
    throw 'Release build created unexpected untracked files'
}
Save-Phase8CState -Values @{
    repositoryRoot = $repositoryRoot
    metadataTarget = $metadataTarget
    hostTarget = $hostTarget
    releaseExe = $releaseExe
    releaseHash = $releaseHash
}
```

Expected: release exit 0 and only `src-tauri/target/$hostTarget/release/extractum.exe` is accepted. Any tracked checkpoint-file change after the verifier or release invalidates both gates: restage, record a new `$CHECKPOINT_TREE`, rerun `npm.cmd run verify`, and rerun the release build.

- [ ] **Step 4: Commit the unchanged index once and record `EXTRACTION_COMMIT`**

```powershell
if ((git write-tree).Trim() -ne $CHECKPOINT_TREE) { throw 'Index drifted before extraction commit' }
$unexpectedUntracked = @(git ls-files --others --exclude-standard)
if ($unexpectedUntracked.Count -ne 0) {
    $unexpectedUntracked
    throw 'Pre-retention tree contains unexpected untracked files'
}
git commit -m "refactor: extract Telegram integration crate"
if ($LASTEXITCODE -ne 0) { throw 'Phase 8C implementation commit failed; leave the staged tree intact' }
$EXTRACTION_COMMIT = (git rev-parse HEAD).Trim()
if ($EXTRACTION_COMMIT -notmatch '^[a-f0-9]{40}$') { throw 'Could not resolve EXTRACTION_COMMIT' }
$retainedTree = (git rev-parse "${EXTRACTION_COMMIT}^{tree}").Trim()
if ($retainedTree -ne $CHECKPOINT_TREE) {
    throw "EXTRACTION_COMMIT tree $retainedTree differs from verified index tree $CHECKPOINT_TREE"
}
$extractionParent = (git rev-parse "${EXTRACTION_COMMIT}^").Trim()
if ($extractionParent -ne $BASE_COMMIT) {
    throw "EXTRACTION_COMMIT parent $extractionParent is not BASE_COMMIT $BASE_COMMIT"
}
if ((git show -s --format=%s $EXTRACTION_COMMIT).Trim() -ne 'refactor: extract Telegram integration crate') {
    throw 'Unexpected extraction commit subject'
}
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) {
    throw 'EXTRACTION_COMMIT did not leave a clean working tree'
}
Save-Phase8CState -Values @{
    EXTRACTION_COMMIT = $EXTRACTION_COMMIT
}
```

Expected: one clean implementation commit directly atop `BASE_COMMIT`. No docs status is terminal yet.

- [ ] **Step 5: Repeat all 17 blob equalities and both approved diffs against `EXTRACTION_COMMIT`**

```powershell
$retainedBlobRows = foreach ($relativePath in $unchangedPaths) {
    $frozenBlob = (git rev-parse "${Frozen8B}:src-tauri/src/telegram_impl/$relativePath").Trim()
    if ($LASTEXITCODE -ne 0) { throw "Missing frozen source blob for $relativePath" }
    $retainedBlob = (git rev-parse "${EXTRACTION_COMMIT}:src-tauri/crates/extractum-telegram/src/$relativePath").Trim()
    if ($LASTEXITCODE -ne 0) { throw "Missing retained destination blob for $relativePath" }
    if ($frozenBlob -ne $retainedBlob) {
        throw "Retained destination blob drift for ${relativePath}: $retainedBlob != $frozenBlob"
    }
    "$relativePath $frozenBlob $retainedBlob"
}
if ($retainedBlobRows.Count -ne 17) {
    throw "Expected 17 retained unchanged blob rows, found $($retainedBlobRows.Count)"
}
$retainedBlobRows | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'retained-unchanged-blobs.txt')

$retainedCorrectedRows = foreach ($relativePath in $correctedPaths) {
    $sourceBlob = (git rev-parse "${Frozen8B}:src-tauri/src/telegram_impl/$relativePath").Trim()
    if ($LASTEXITCODE -ne 0) { throw "Missing frozen corrected source blob for $relativePath" }
    $retainedBlob = (git rev-parse "${EXTRACTION_COMMIT}:src-tauri/crates/extractum-telegram/src/$relativePath").Trim()
    if ($LASTEXITCODE -ne 0) { throw "Missing retained corrected blob for $relativePath" }
    $diffCapture = Invoke-CheckedNative "retained corrected diff $relativePath" {
        git diff --no-ext-diff --unified=0 $sourceBlob $retainedBlob
    }
    $evidenceName = switch ($relativePath) {
        'lib.rs' { 'retained-corrected-lib.rs.diff' }
        'takeout/mod.rs' { 'retained-corrected-takeout-mod.rs.diff' }
        default { throw "Unexpected corrected path: $relativePath" }
    }
    $diffCapture.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot $evidenceName)
    $expected = $expectedCorrectedDiffs[$relativePath]
    Assert-ExactDiffBody -Label "retained $relativePath" -Diff $diffCapture.Text `
        -ExpectedRemoved $expected.Removed -ExpectedAdded $expected.Added `
        -ExpectedHunkPrefixes $expected.Hunks
    "$relativePath $sourceBlob $retainedBlob"
}
if ($retainedCorrectedRows.Count -ne 2) {
    throw "Expected two retained corrected blob rows, found $($retainedCorrectedRows.Count)"
}
$retainedCorrectedRows | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'retained-corrected-blobs.txt')
```

Expected: the authoritative retained commit proves the same 17 equal blobs and the same two exact corrected diffs as the precommit index.

### Task 7: Run the One-Time Post-Commit MCP and Startup Evidence

**Files:**
- Read/execute: clean `EXTRACTION_COMMIT`
- Read: `src-tauri/target/<host>/release/extractum.exe`
- Generate ignored only: `.playwright-mcp/` if the live driver creates it

**Interfaces:**
- Consumes: clean `$EXTRACTION_COMMIT`, `$hostTarget`, `$releaseExe`, and `$releaseHash` from Task 6.
- Produces: one real-webview `[]` result and one exact-PID five-second release startup/cleanup record. Neither check mutates product data.

- [ ] **Step 1: Prove the retained commit and development ports/processes are clean before MCP startup**

```powershell
if ((git rev-parse HEAD).Trim() -ne $EXTRACTION_COMMIT) { throw 'HEAD moved after extraction retention' }
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) { throw 'MCP evidence requires a clean tree' }
if (@(Get-Process -Name 'extractum' -ErrorAction SilentlyContinue).Count -ne 0) {
    throw 'A pre-existing extractum process blocks MCP evidence'
}
$configuredDevPort = 1420
if (@(Get-NetTCPConnection -State Listen -LocalPort $configuredDevPort -ErrorAction SilentlyContinue).Count -ne 0) {
    throw "Configured dev port $configuredDevPort is already listening"
}
Save-Phase8CState -Values @{
    configuredDevPort = $configuredDevPort
}
```

Expected: clean retained commit, no app process, and the configured dev port free. Still use the actual URL printed by Vite as runtime evidence; do not infer a fallback port.

- [ ] **Step 2: Start the canonical MCP-enabled development app and invoke the real webview command once**

Start the canonical command in one hidden, PID-owned host and capture its logs outside the repository:

```powershell
$launchedRootPids = [Collections.Generic.List[int]]::new()
$ownedRootPids = [Collections.Generic.List[int]]::new()

function Start-OwnedHiddenPowerShell {
    param(
        [Parameter(Mandatory = $true)][string]$Command,
        [Parameter(Mandatory = $true)][string]$StdoutPath,
        [Parameter(Mandatory = $true)][string]$StderrPath
    )
    $encodedCommand = [Convert]::ToBase64String(
        [Text.Encoding]::Unicode.GetBytes($Command)
    )
    $hostProcess = Start-Process `
        -FilePath 'powershell.exe' `
        -ArgumentList @('-NoLogo', '-NoProfile', '-EncodedCommand', $encodedCommand) `
        -RedirectStandardOutput $StdoutPath `
        -RedirectStandardError $StderrPath `
        -PassThru `
        -WindowStyle Hidden
    [void]$launchedRootPids.Add([int]$hostProcess.Id)
    [void]$ownedRootPids.Add([int]$hostProcess.Id)
    return $hostProcess
}

function Read-CombinedLog {
    param([string]$StdoutPath, [string]$StderrPath)
    return @(
        Get-Content -Raw -LiteralPath $StdoutPath -ErrorAction SilentlyContinue
        Get-Content -Raw -LiteralPath $StderrPath -ErrorAction SilentlyContinue
    ) -join "`n"
}

function Wait-LoggedLocalUrl {
    param(
        [Parameter(Mandatory = $true)]$HostProcess,
        [Parameter(Mandatory = $true)][string]$StdoutPath,
        [Parameter(Mandatory = $true)][string]$StderrPath,
        [Parameter(Mandatory = $true)][DateTime]$Deadline
    )
    while ([DateTime]::UtcNow -lt $Deadline) {
        $HostProcess.Refresh()
        $log = Read-CombinedLog -StdoutPath $StdoutPath -StderrPath $StderrPath
        $match = [regex]::Match($log, 'https?://(?:localhost|127\.0\.0\.1):\d+/?')
        if ($match.Success) { return $match.Value }
        if ($HostProcess.HasExited) { return $null }
        Start-Sleep -Milliseconds 250
    }
    return $null
}

$escapedRepositoryRoot = $repositoryRoot.Replace("'", "''")
$devStdout = Join-Path $EvidenceRoot 'tauri-dev.stdout.txt'
$devStderr = Join-Path $EvidenceRoot 'tauri-dev.stderr.txt'
$devCommand = "Set-Location -LiteralPath '$escapedRepositoryRoot'; npm.cmd run tauri dev"
$devHost = $null
$viteFallbackHost = $null
$fallbackTauriHost = $null

try {
    $devHost = Start-OwnedHiddenPowerShell -Command $devCommand `
        -StdoutPath $devStdout -StderrPath $devStderr
    $actualViteUrl = Wait-LoggedLocalUrl -HostProcess $devHost `
        -StdoutPath $devStdout -StderrPath $devStderr `
        -Deadline ([DateTime]::UtcNow.AddMinutes(3))

    if ($null -eq $actualViteUrl) {
        $devHost.Refresh()
        if (-not $devHost.HasExited) {
            throw 'Vite did not print a local URL within three minutes'
        }
        (Read-CombinedLog -StdoutPath $devStdout -StderrPath $devStderr) |
            Set-Content -LiteralPath (Join-Path $EvidenceRoot 'tauri-dev-canonical-failure.txt')
        Stop-OwnedDevTrees
        if (@(Get-NetTCPConnection -State Listen -LocalPort $configuredDevPort -ErrorAction SilentlyContinue).Count -ne 0) {
            throw 'Canonical dev failure left the configured Vite port occupied'
        }

        $viteStdout = Join-Path $EvidenceRoot 'vite-fallback.stdout.txt'
        $viteStderr = Join-Path $EvidenceRoot 'vite-fallback.stderr.txt'
        $viteCommand = "Set-Location -LiteralPath '$escapedRepositoryRoot'; node.exe node_modules/vite/bin/vite.js --host 127.0.0.1"
        $viteFallbackHost = Start-OwnedHiddenPowerShell -Command $viteCommand `
            -StdoutPath $viteStdout -StderrPath $viteStderr
        $actualViteUrl = Wait-LoggedLocalUrl -HostProcess $viteFallbackHost `
            -StdoutPath $viteStdout -StderrPath $viteStderr `
            -Deadline ([DateTime]::UtcNow.AddMinutes(3))
        if ($null -eq $actualViteUrl) {
            throw 'Escalated hidden Vite fallback did not become ready'
        }

        $fallbackTauriStdout = Join-Path $EvidenceRoot 'tauri-dev-fallback.stdout.txt'
        $fallbackTauriStderr = Join-Path $EvidenceRoot 'tauri-dev-fallback.stderr.txt'
        $fallbackOverlay = '{"build":{"beforeDevCommand":null,"features":["prompt-pack-dev-fixtures"]},"app":{"withGlobalTauri":true}}'
        $escapedFallbackOverlay = $fallbackOverlay.Replace("'", "''")
        $fallbackTauriCommand = "Set-Location -LiteralPath '$escapedRepositoryRoot'; npm.cmd run tauri -- dev --config '$escapedFallbackOverlay'"
        $fallbackTauriHost = Start-OwnedHiddenPowerShell -Command $fallbackTauriCommand `
            -StdoutPath $fallbackTauriStdout -StderrPath $fallbackTauriStderr

        $appDeadline = [DateTime]::UtcNow.AddMinutes(3)
        while (
            @(Get-Process -Name 'extractum' -ErrorAction SilentlyContinue).Count -eq 0 -and
            [DateTime]::UtcNow -lt $appDeadline
        ) {
            $fallbackTauriHost.Refresh()
            if ($fallbackTauriHost.HasExited) {
                throw 'Fallback Tauri dev host exited before the app process started'
            }
            Start-Sleep -Milliseconds 250
        }
        if (@(Get-Process -Name 'extractum' -ErrorAction SilentlyContinue).Count -ne 1) {
            throw 'Fallback Tauri dev did not create exactly one app process'
        }
    }

    $actualDevPort = ([uri]$actualViteUrl).Port
    Save-Phase8CState -Values @{
        actualDevPort = $actualDevPort
        actualViteUrl = $actualViteUrl
        launchedRootPids = @($launchedRootPids)
        ownedRootPids = @($ownedRootPids)
        mcpCleanupComplete = $false
    }
}
catch {
    $startupFailure = $_
    Stop-OwnedDevTrees
    throw $startupFailure
}
```

Do not use direct `npx tauri dev`. The fallback Vite `Start-Process` call is the one command to rerun with sandbox escalation/approval when the canonical host's child Vite exits immediately; its Tauri restart still goes through `npm.cmd run tauri` and reproduces the MCP overlay exactly while setting `beforeDevCommand` to `null`. `$launchedRootPids` retains every started host PID, including an already-exited canonical host, while `$ownedRootPids` contains only trees still owed cleanup. Once the Tauri window is ready, treat the driver/MCP sequence and Step 3 cleanup as one `try/finally`: on any driver start, backend, window, invocation, or assertion failure, retain the error, stop the driver if it started, run `Stop-OwnedDevTrees`, perform the Step 3 residue assertions, then rethrow. Use the live MCP tools in this exact order:

```text
driver_session start
ipc_get_backend_state
manage_window list
webview_execute_js script="(async () => await window.__TAURI__.core.invoke('tg_get_account_statuses', { accountIds: [] }))()"
```

Require:

```text
backend connected to org.ai.extractum
one live Extractum window
webview result exactly []
```

The pinned bridge's plugin executor is not valid evidence for application commands. Do not create/delete an account, send a code, sign in, touch credentials, add a source, or start Takeout.

Record sanitized backend/window identity, actual Vite URL, and the literal `[]` in `$EvidenceRoot/mcp-smoke.txt`. Never record API hashes, phone numbers, sessions, login tokens, or account contents.

- [ ] **Step 3: Stop the live driver and only the dev processes it owns, then prove cleanup**

In the `finally` path, invoke `driver_session stop` if and only if `driver_session start` succeeded; preserve any earlier MCP error while still attempting process cleanup. Then snapshot and terminate only the exact hidden host tree started in Step 2, children before parent, and verify:

```text
driver_session stop
```

```powershell
Stop-OwnedDevTrees
if (@(Get-Process -Name 'extractum' -ErrorAction SilentlyContinue).Count -ne 0) {
    throw 'MCP dev app process residue remains'
}
if (@(Get-NetTCPConnection -State Listen -LocalPort $actualDevPort -ErrorAction SilentlyContinue).Count -ne 0) {
    throw 'MCP dev Vite port remains occupied'
}
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) {
    throw 'MCP smoke changed tracked or non-ignored repository state'
}
"launched_root_pids=$($launchedRootPids -join ',')" |
    Set-Content -LiteralPath (Join-Path $EvidenceRoot 'mcp-owned-root-pids.txt')
$mcpCleanupComplete = $true
Save-Phase8CState -Values @{
    ownedRootPids = @()
    mcpCleanupComplete = $mcpCleanupComplete
}
```

The cleanup uses the actual Vite port and the captured process tree; it never scans and kills unrelated Node/Cargo processes. `.playwright-mcp/` is generated and ignored; never stage it.

- [ ] **Step 4: Rebind the prebuilt release executable to its precommit hash before startup**

```powershell
if (-not $mcpCleanupComplete) {
    throw 'Release startup is blocked until MCP-owned processes are cleaned up'
}
$releaseExeAfterMcp = (Resolve-Path -LiteralPath (
    Join-Path 'src-tauri/target' "$hostTarget/release/extractum.exe"
)).Path
if (-not [String]::Equals($releaseExeAfterMcp, $releaseExe, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Host-qualified release path changed after MCP smoke'
}
$releaseHashAfterMcp = (Get-FileHash -LiteralPath $releaseExeAfterMcp -Algorithm SHA256).Hash.ToLowerInvariant()
if ($releaseHashAfterMcp -ne $releaseHash) {
    throw "Prebuilt release hash changed after MCP smoke: $releaseHashAfterMcp != $releaseHash"
}
```

Expected: the startup smoke will execute the exact binary built on the uncommitted checkpoint tree and retained in `EXTRACTION_COMMIT`.

- [ ] **Step 5: Run the exact hidden PID/path-bounded five-second startup smoke**

```powershell
$preExisting = @(Get-Process -Name 'extractum' -ErrorAction SilentlyContinue)
if ($preExisting.Count -ne 0) {
    throw 'Pre-existing extractum process prevents bounded startup evidence'
}

$started = Start-Process -FilePath $releaseExe -PassThru -WindowStyle Hidden
$startedId = $started.Id
$actualPath = $null
try {
    $survivalDeadline = [DateTime]::UtcNow.AddSeconds(5)
    while ([DateTime]::UtcNow -lt $survivalDeadline) {
        $started.Refresh()
        if ($started.HasExited) {
            throw 'Release executable exited before the five-second survival window'
        }
        Start-Sleep -Milliseconds 200
    }

    $live = Get-Process -Id $startedId -ErrorAction Stop
    $actualPath = (Resolve-Path -LiteralPath $live.Path).Path
    if (-not [String]::Equals($actualPath, $releaseExe, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Startup PID executable path does not match the built release executable'
    }
}
finally {
    $owned = Get-Process -Id $startedId -ErrorAction SilentlyContinue
    if ($null -ne $owned) { Stop-Process -Id $startedId -Force }
    $cleanupDeadline = [DateTime]::UtcNow.AddSeconds(10)
    while (
        $null -ne (Get-Process -Id $startedId -ErrorAction SilentlyContinue) -and
        [DateTime]::UtcNow -lt $cleanupDeadline
    ) {
        Start-Sleep -Milliseconds 200
    }
    if ($null -ne (Get-Process -Id $startedId -ErrorAction SilentlyContinue)) {
        throw 'Owned release PID did not terminate within ten seconds'
    }
}

if (@(Get-Process -Name 'extractum' -ErrorAction SilentlyContinue).Count -ne 0) {
    throw 'Unexpected extractum process residue after bounded startup smoke'
}
@(
    "pid=$startedId",
    "path=$actualPath",
    'survival_seconds=5',
    'owned_pid_terminated=true',
    'extractum_residue=0'
) | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'startup-smoke.txt')
```

Expected: the exact PID remains alive for five seconds, resolves to the host-qualified release path, is stopped by exact PID, and leaves no `extractum` residue.

If MCP or startup fails, leave `EXTRACTION_COMMIT` intact, keep Phase 8 pending, write no terminal evidence/status commit, and report the environment-sensitive failure. Do not amend or erase the extraction solely because one of these post-commit checks failed.

### Task 8: Record Durable Evidence and Close Phase 8 in a Docs-Only Commit

**Files:**
- Create: `docs/superpowers/verification/2026-08-01-extractum-telegram-8c-extraction.md`
- Modify: `docs/superpowers/specs/2026-08-01-telegram-8c-extraction-design.md`
- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `docs/project.md`
- Modify: `docs/value-registry.md`

**Interfaces:**
- Consumes: every literal capture in `$EvidenceRoot`, clean `$EXTRACTION_COMMIT`, and successful post-commit MCP/startup results.
- Produces: a self-contained verification record, terminal `done: retained` disposition, current project architecture, and one docs-only commit.

- [ ] **Step 1: Create the verification document with literal retained evidence**

First render the complete document mechanically from the already retained captures. The `@@...@@` strings below are transient renderer tokens, not documentation placeholders: the script requires every token exactly once, substitutes every one, rejects any residue, and writes only a preview inside `$EvidenceRoot`. Do not create or edit a repository file from PowerShell.

````powershell
function Join-EvidenceBlocks {
    param([Parameter(Mandatory = $true)][string[]]$Names)
    $blocks = foreach ($name in $Names) {
        "#### $name"
        ''
        '```text'
        Read-EvidenceText $name
        '```'
        ''
    }
    return ($blocks -join "`n").TrimEnd()
}

$redText = Read-EvidenceText 'fixture-boundary-red.txt'
foreach ($fixtureName in @('takeout_attempt_fixture', 'takeout_fallback_fixture')) {
    if ($redText -notmatch [regex]::Escape($fixtureName)) {
        throw "Transient RED omitted $fixtureName"
    }
}
$secondaryRed = if (
    $redText -match 'src-tauri[/\\]src[/\\]takeout_import[/\\]mod\.rs'
) { 'present and retained in the capture' } else { 'absent from this rustc emission' }

$finalMetadata = Read-EvidenceText 'final-cargo-metadata.json' |
    ConvertFrom-Json
$workspacePackagesById = @{}
foreach ($package in $finalMetadata.packages) {
    $workspacePackagesById[[string]$package.id] = $package
}
$workspaceMemberNames = @(
    $finalMetadata.workspace_members |
        ForEach-Object { [string]$workspacePackagesById[[string]$_].name }
)
Assert-ExactStringSet -Label 'terminal metadata workspace members' `
    -Expected @(
        'extractum',
        'extractum-analysis',
        'extractum-core',
        'extractum-gemini-browser',
        'extractum-llm',
        'extractum-prompt-packs',
        'extractum-telegram'
    ) -Actual $workspaceMemberNames

$appPackage = @($finalMetadata.packages | Where-Object name -eq 'extractum')
$producerPackage = @($finalMetadata.packages | Where-Object name -eq 'extractum-telegram')
if ($appPackage.Count -ne 1 -or $producerPackage.Count -ne 1) {
    throw 'Locked metadata does not contain one app and one Telegram producer package'
}
$appNode = @($finalMetadata.resolve.nodes | Where-Object id -eq $appPackage[0].id)
$producerNode = @($finalMetadata.resolve.nodes | Where-Object id -eq $producerPackage[0].id)
if ($appNode.Count -ne 1 -or $producerNode.Count -ne 1) {
    throw 'Locked metadata does not contain one app and one Telegram producer node'
}
$producerEdges = @($appNode[0].deps | Where-Object pkg -eq $producerPackage[0].id)
if ($producerEdges.Count -ne 1) { throw 'App does not have exactly one resolved producer edge' }
$producerEdgeKinds = @(
    $producerEdges[0].dep_kinds |
        ForEach-Object {
            if ([string]::IsNullOrWhiteSpace([string]$_.kind)) {
                'normal'
            } else {
                [string]$_.kind
            }
        } |
        Sort-Object -Unique
)
Assert-ExactStringSet -Label 'terminal app-to-producer edge kinds' `
    -Expected @('dev', 'normal') -Actual $producerEdgeKinds

$producerDirectRoots = @(
    $producerPackage[0].dependencies |
        Where-Object {
            [string]::IsNullOrWhiteSpace([string]$_.kind) -or $_.kind -eq 'normal'
        } |
        ForEach-Object { [string]$_.name } |
        Sort-Object -Unique
)
$expectedProducerDirectRoots = @(
    'base64',
    'chacha20poly1305',
    'extractum-core',
    'grammers-client',
    'grammers-mtsender',
    'grammers-session',
    'grammers-tl-types',
    'rand_core',
    'secrecy',
    'serde',
    'serde_json',
    'tokio'
)
Assert-ExactStringSet -Label 'terminal producer direct roots' `
    -Expected $expectedProducerDirectRoots -Actual $producerDirectRoots
$removedAppRoots = @(
    'chacha20poly1305',
    'grammers-client',
    'grammers-mtsender',
    'grammers-session',
    'grammers-tl-types',
    'rand_core'
)
$appDirectNames = @($appPackage[0].dependencies | ForEach-Object { [string]$_.name })
if (@($removedAppRoots | Where-Object { $_ -in $appDirectNames }).Count -ne 0) {
    throw 'A moved Telegram dependency root remains direct on the app package'
}

$baseCargoText = (git show "${BASE_COMMIT}:src-tauri/Cargo.toml" | Out-String)
if ($LASTEXITCODE -ne 0) { throw 'Could not reread BASE Cargo.toml' }
$retainedCargoText = (git show "${EXTRACTION_COMMIT}:src-tauri/Cargo.toml" | Out-String)
if ($LASTEXITCODE -ne 0) { throw 'Could not reread retained Cargo.toml' }
$baseWorkspaceDependencies =
    Get-TomlSectionFromGitText $baseCargoText 'workspace.dependencies'
$retainedWorkspaceDependencies =
    Get-TomlSectionFromGitText $retainedCargoText 'workspace.dependencies'
if ($baseWorkspaceDependencies -cne $retainedWorkspaceDependencies) {
    throw 'Retained workspace dependency declarations differ from BASE'
}

$cargoSummary = [ordered]@{
    workspace_members = $workspaceMemberNames
    app_to_extractum_telegram_dep_kinds = $producerEdgeKinds
    removed_app_direct_roots = $removedAppRoots
    producer_direct_roots = $producerDirectRoots
    producer_to_app_edge = @($producerNode[0].deps | Where-Object pkg -eq $appPackage[0].id).Count
    workspace_dependencies_equal_base = $true
} | ConvertTo-Json -Depth 5

$fixtureCaptureNames = @(
    $fixtureConsumerTests | ForEach-Object {
        'fixture-green-' + ($_ -replace '[^A-Za-z0-9_.-]', '_') + '.txt'
    }
)
$focusedCaptureNames = @(
    'producer-feature-off-lib-check.txt',
    'producer-isolated-all-targets-check.txt',
    'producer-isolated-all-targets-test.txt',
    'consumer-normal-lib-check.txt',
    'consumer-feature-on-all-targets-check.txt',
    'consumer-feature-on-all-targets-test.txt'
)
$fullGateCaptureNames = @(
    'check-rustfmt.txt',
    'workspace-check.txt',
    'workspace-test.txt',
    'npm-verify.txt'
)
$workspaceTimingLine = Read-EvidenceText 'workspace-check-timing.txt'
if ($workspaceTimingLine -notmatch 'Finished .+ in ([0-9]+(?:\.[0-9]+)?)s') {
    throw 'Retained workspace timing line is malformed'
}
$durationMilliseconds = [int][Math]::Round(
    [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture) * 1000
)
if ($durationMilliseconds -ne $WORKSPACE_CHECK_DURATION_MS) {
    throw 'Retained timing line no longer matches WORKSPACE_CHECK_DURATION_MS'
}

$verificationTemplate = @'
# Extractum Telegram Phase 8C Extraction Verification

**Result: implemented and retained.**

## Retained commits

- `BASE_COMMIT`: `@@BASE_COMMIT@@`
- `EXTRACTION_COMMIT`: `@@EXTRACTION_COMMIT@@`
- extraction parent equals `BASE_COMMIT`: `true`
- extraction subject: `refactor: extract Telegram integration crate`
- extraction tree equals the fully verified staged tree: `true`

## Frozen 8B authority and BASE_COMMIT

- frozen 8B commit: `@@FROZEN_8B@@`
- frozen 8B is an ancestor of `BASE_COMMIT`: `true`

```text
@@FROZEN_HASHES@@
```

The retained Grammers feature artifact stayed byte-identical:

```json
@@GRAMMERS_ARTIFACT@@
```

## Transient fixture-boundary RED

The primary facade re-export failed with `E0432` and named both fixture imports.
The secondary consumer cascade was @@SECONDARY_RED@@. The command was
`cargo check`; no test executable was produced and no test ran.

```text
@@TRANSIENT_RED@@
```

## File disposition and content identity

The retained extraction moved exactly 19 prepared source paths. Seventeen are
Git-blob identical; only `lib.rs` and `takeout/mod.rs` have the two approved
complete zero-context diffs below.

### 17 unchanged Git blob pairs

```text
@@UNCHANGED_BLOBS@@
```

### lib.rs approved complete diff

```diff
@@LIB_DIFF@@
```

### takeout/mod.rs approved complete diff

```diff
@@TAKEOUT_DIFF@@
```

The two corrected source/destination blob pairs are:

```text
@@CORRECTED_BLOBS@@
```

## Cargo package and dependency ownership

```json
@@CARGO_SUMMARY@@
```

The exact `[workspace.dependencies]` block is byte-identical to `BASE_COMMIT`:

```toml
@@WORKSPACE_DEPENDENCIES@@
```

The locked metadata and graph-cut contract prove seven members, exactly one
normal/dev app edge to the canonical producer package, removal of the six raw
Telegram roots from the app, the exact twelve producer roots, and no reverse
producer-to-app edge.

## Resolver-v2 fixture boundary

The first capture is the canonical feature-off proof. Producer-isolated
`--all-targets` captures are package checkpoints; consumer `--all-targets`
captures activate `app-test-support` through the app dev-dependency.

@@FOCUSED_CAPTURES@@

All four app-owned fixture consumers selected and passed exactly one test:

@@FIXTURE_CAPTURES@@

## Exact Rust test identity sets

The comparison is exact set equality: `736 BASE = 665 app + 71 producer`, and
prefixing each producer identity with `telegram_impl::` recreates the complete
736-entry BASE set.

### 736 pre-edit BASE identities

```text
@@BASE_TESTS@@
```

### 665 extractum identities

```text
@@FINAL_APP_TESTS@@
```

### 71 extractum-telegram identities

```text
@@FINAL_CRATE_TESTS@@
```

### 736 logical union reconciliation

```text
@@FINAL_LOGICAL_TESTS@@
```

## Focused Rust and contract verification

The coupled eight-file boundary run passed on the final physical layout while
the Phase 8 documentation was still pending:

```text
@@FINAL_CONTRACTS@@
```

## Full uncommitted verification

@@FULL_GATE_CAPTURES@@

The single admitted ordinary workspace-check timing was:

```text
@@WORKSPACE_TIMING@@
workspace_check_duration_ms=@@WORKSPACE_DURATION_MS@@
```

## Release build

- exact command: `npm.cmd run tauri -- build --no-bundle --target @@HOST_TARGET@@`
- canonical target directory: `@@CANONICAL_TARGET_DIR@@`
- command exit: `0`

```text
@@RELEASE_IDENTITY@@
```

```text
@@RELEASE_CAPTURE@@
```

## Live MCP smoke

The retained capture contains only sanitized backend/window identity, the
actual Vite URL, and the exact empty account-status result:

```text
@@MCP_SMOKE@@
```

## Bounded startup smoke

```text
@@STARTUP_SMOKE@@
```

The exact release PID survived five seconds, resolved to the host-qualified
release executable, was terminated by PID, and left zero `extractum` residue.

## Security and non-goals

Phase 8C introduced no credentialed Telegram mutation, wire/API behavior
change, SQLite migration, secret read/write, or production fixture capability.
No account, phone number, API hash, session, login token, source payload, or
private Telegram data is retained in this evidence. `app-test-support` is a
development-isolation mechanism, not a security boundary.

## Final disposition

The one implementation commit is retained, the extraction boundary and exact
test identity are proven, release/MCP/startup evidence passed, and Phase 8 can
advance to `done: retained` in a separate docs-only commit.
'@

$verificationReplacements = [ordered]@{
    '@@BASE_COMMIT@@' = $BASE_COMMIT
    '@@EXTRACTION_COMMIT@@' = $EXTRACTION_COMMIT
    '@@FROZEN_8B@@' = $Frozen8B
    '@@FROZEN_HASHES@@' = Read-EvidenceText 'frozen-authority-sha256.txt'
    '@@GRAMMERS_ARTIFACT@@' = (
        Get-Content -Raw -LiteralPath 'src/lib/telegram-grammers-feature-baseline.json'
    ).TrimEnd()
    '@@SECONDARY_RED@@' = $secondaryRed
    '@@TRANSIENT_RED@@' = $redText
    '@@UNCHANGED_BLOBS@@' = Read-EvidenceText 'retained-unchanged-blobs.txt'
    '@@LIB_DIFF@@' = Read-EvidenceText 'retained-corrected-lib.rs.diff'
    '@@TAKEOUT_DIFF@@' = Read-EvidenceText 'retained-corrected-takeout-mod.rs.diff'
    '@@CORRECTED_BLOBS@@' = Read-EvidenceText 'retained-corrected-blobs.txt'
    '@@CARGO_SUMMARY@@' = $cargoSummary
    '@@WORKSPACE_DEPENDENCIES@@' = $baseWorkspaceDependencies.TrimEnd()
    '@@FOCUSED_CAPTURES@@' = Join-EvidenceBlocks $focusedCaptureNames
    '@@FIXTURE_CAPTURES@@' = Join-EvidenceBlocks $fixtureCaptureNames
    '@@BASE_TESTS@@' = Read-EvidenceText 'base-extractum-tests.txt'
    '@@FINAL_APP_TESTS@@' = Read-EvidenceText 'final-app-tests.txt'
    '@@FINAL_CRATE_TESTS@@' = Read-EvidenceText 'final-telegram-crate-tests.txt'
    '@@FINAL_LOGICAL_TESTS@@' = Read-EvidenceText 'final-logical-tests.txt'
    '@@FINAL_CONTRACTS@@' = Read-EvidenceText 'final-layout-contracts.txt'
    '@@FULL_GATE_CAPTURES@@' = Join-EvidenceBlocks $fullGateCaptureNames
    '@@WORKSPACE_TIMING@@' = $workspaceTimingLine
    '@@WORKSPACE_DURATION_MS@@' = [string]$WORKSPACE_CHECK_DURATION_MS
    '@@HOST_TARGET@@' = $hostTarget
    '@@CANONICAL_TARGET_DIR@@' = $metadataTarget
    '@@RELEASE_IDENTITY@@' = Read-EvidenceText 'release-identity.txt'
    '@@RELEASE_CAPTURE@@' = Read-EvidenceText 'release-build.txt'
    '@@MCP_SMOKE@@' = Read-EvidenceText 'mcp-smoke.txt'
    '@@STARTUP_SMOKE@@' = Read-EvidenceText 'startup-smoke.txt'
}
$templateTokens = @(
    [regex]::Matches($verificationTemplate, '@@[A-Z0-9_]+@@') |
        ForEach-Object Value |
        Sort-Object -Unique
)
Assert-ExactStringSet -Label 'verification renderer tokens' `
    -Expected @($verificationReplacements.Keys) -Actual $templateTokens
foreach ($replacement in $verificationReplacements.GetEnumerator()) {
    $verificationTemplate = $verificationTemplate.Replace(
        [string]$replacement.Key,
        [string]$replacement.Value
    )
}
if ($verificationTemplate -match '@@[A-Z0-9_]+@@') {
    throw 'Verification renderer left an unresolved token'
}
$verificationPreview = Join-Path $EvidenceRoot 'terminal-verification-rendered.md'
$renderedVerificationText = (
    ($verificationTemplate -replace "\r\n?", "`n").TrimEnd("`n") + "`n"
)
[IO.File]::WriteAllText(
    $verificationPreview,
    $renderedVerificationText,
    [Text.UTF8Encoding]::new($false)
)
Save-Phase8CState -Values @{
    verificationPreview = $verificationPreview
}
````

Inspect the rendered preview for secrets. Then use `apply_patch` to create `docs/superpowers/verification/2026-08-01-extractum-telegram-8c-extraction.md` with the preview's exact normalized-LF contents. Assert byte equality after the patch:

```powershell
$retainedVerification = (
    Get-Content -Raw -LiteralPath `
        'docs/superpowers/verification/2026-08-01-extractum-telegram-8c-extraction.md'
) -replace "\r\n?", "`n"
$renderedVerification = (
    Get-Content -Raw -LiteralPath $verificationPreview
) -replace "\r\n?", "`n"
if ($retainedVerification -cne $renderedVerification) {
    throw 'Retained verification document differs from the exact rendered evidence'
}
```

Do not include secrets or retain the temporary evidence directory in Git.

- [ ] **Step 2: Apply the exact terminal design and roadmap dispositions**

Set the one status line in both design specs to exactly:

```markdown
**Status:** Implemented and retained; [verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md)
```

In `docs/superpowers/specs/2026-07-17-crate-roadmap.md`, change the heading to:

```markdown
### Phase 8 — `extractum-telegram` (done: retained)
```

Build the exact retained summary from already verified runtime values:

```powershell
$roadmapSummaryTemplate = @'
Phase 8C is implemented and retained in the single implementation commit
`@@EXTRACTION_COMMIT@@`, whose parent is the clean `BASE_COMMIT`
`@@BASE_COMMIT@@`. The extraction moved the exact 19-file prepared Telegram
tree into `extractum-telegram`: 17 destination blobs are identical to frozen
8B and `lib.rs` plus `takeout/mod.rs` contain only their two approved complete
diffs. The exact library-test split is 665 app / 71 producer / 736 normalized
logical union, equal to the 736-test BASE set. The producer's canonical
feature-off check and the app's resolver-v2 feature-on tests passed; the app
has only the feature-free normal edge plus the feature-enabled dev edge. Locked
metadata proves the seven-member workspace, removal of the six raw Telegram
roots from the app, and their producer ownership.

The final uncommitted verifier and host-target release build passed for
`@@HOST_TARGET@@`; the retained executable is `@@RELEASE_PATH@@` with SHA-256
`@@RELEASE_SHA256@@`. The live MCP empty-account-status smoke returned `[]`,
and the same executable passed the bounded five-second startup/cleanup smoke
with zero residue. The retained evidence is
[Phase 8C extraction verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md).
The single admitted ordinary workspace check completed in
`@@WORKSPACE_DURATION_MS@@ ms`.
'@
$releaseIdentity = @{}
foreach ($line in (Read-EvidenceText 'release-identity.txt') -split "`n") {
    $parts = $line -split '=', 2
    if ($parts.Count -eq 2) {
        $releaseIdentity[$parts[0].Trim()] = $parts[1].Trim()
    }
}
foreach ($requiredReleaseKey in @('host_target', 'release_path', 'release_sha256')) {
    if ([string]::IsNullOrWhiteSpace([string]$releaseIdentity[$requiredReleaseKey])) {
        throw "Release identity omitted $requiredReleaseKey"
    }
}
$roadmapReplacements = [ordered]@{
    '@@EXTRACTION_COMMIT@@' = $EXTRACTION_COMMIT
    '@@BASE_COMMIT@@' = $BASE_COMMIT
    '@@HOST_TARGET@@' = $releaseIdentity.host_target
    '@@RELEASE_PATH@@' = $releaseIdentity.release_path
    '@@RELEASE_SHA256@@' = $releaseIdentity.release_sha256
    '@@WORKSPACE_DURATION_MS@@' = [string]$WORKSPACE_CHECK_DURATION_MS
}
foreach ($replacement in $roadmapReplacements.GetEnumerator()) {
    $roadmapSummaryTemplate = $roadmapSummaryTemplate.Replace(
        [string]$replacement.Key,
        [string]$replacement.Value
    )
}
if ($roadmapSummaryTemplate -match '@@[A-Z0-9_]+@@') {
    throw 'Roadmap summary renderer left an unresolved token'
}
$roadmapSummaryPreview = Join-Path $EvidenceRoot 'terminal-roadmap-summary-rendered.md'
[IO.File]::WriteAllText(
    $roadmapSummaryPreview,
    (($roadmapSummaryTemplate -replace "\r\n?", "`n").TrimEnd("`n") + "`n"),
    [Text.UTF8Encoding]::new($false)
)
Save-Phase8CState -Values @{
    roadmapSummaryPreview = $roadmapSummaryPreview
}
```

Use `apply_patch` to insert the rendered summary immediately after the terminal Phase 8 heading. Make these three additional exact replacements while retaining every historical 8A/8B checkpoint paragraph and the timing-policy paragraph.

Replace the three-line paragraph immediately before the heading:

```markdown
Phase 8 has an owner-approved Phase 8 boundary; 8A preparation and 8B
preparation are retained, while 8C remains pending. Further execution follows
the three separately reviewed plans and requires explicit owner instructions.
```

with:

```markdown
Phase 8 has an owner-approved and retained boundary. Preparatory slices 8A and
8B remain historical authority; 8C completed the extraction, with terminal
evidence linked from the retained Phase 8 summary below.
```

Replace the current linked-design summary near the top of the roadmap:

```markdown
- [`2026-07-26-telegram-crate-boundary-design.md`](2026-07-26-telegram-crate-boundary-design.md)
  — owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 5
  authority are retained. It replaces the stale
  five-file cluster with three separately retainable sub-slices that remove
  the full current Grammers perimeter without moving app-owned SQL, secrets,
  events, or Takeout job orchestration.
```

with:

```markdown
- [`2026-07-26-telegram-crate-boundary-design.md`](2026-07-26-telegram-crate-boundary-design.md)
  — implemented and retained Phase 8 boundary. The 8A and 8B preparations
  remain historical authority; 8C retained the seven-member extraction without
  moving app-owned SQL, secrets, events, or Takeout job orchestration. See the
  [terminal verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md).
```

Replace the current summary immediately before the Phase 7 heading:

```markdown
Phase 7 is implemented and retained under the
[JIT boundary design](2026-07-22-analysis-crate-boundary-design.md). The
[verification document](../verification/2026-07-22-extractum-analysis-extraction.md)
records the retained result. Phase 8A preparation and Phase 8B Checkpoint 5
authority are retained.
```

with:

```markdown
Phase 7 is implemented and retained under the
[JIT boundary design](2026-07-22-analysis-crate-boundary-design.md). The
[verification document](../verification/2026-07-22-extractum-analysis-extraction.md)
records the retained result. Phase 8 is implemented and retained under the
[Telegram boundary design](2026-07-26-telegram-crate-boundary-design.md); its
[verification document](../verification/2026-08-01-extractum-telegram-8c-extraction.md)
records the terminal extraction result.
```

Replace the paragraph beginning `By the end of 8B, staged modules use only` and ending `8C makes no visibility change.` with:

```markdown
By the end of 8B, staged modules used only relative `self::`/`super::` paths,
while app-owned callers used the exact `crate::telegram_impl::` prefix. The
retained 8C extraction preserves 17 prepared files as exact Git blobs. Its two
approved corrections are confined to the producer crate root's feature-gated
fixture exports and the Takeout fixture definitions; function bodies,
production signatures, and the curated production API remain unchanged. The
vacated app `telegram_impl/lib.rs` is a private explicit compatibility facade,
so app-owned production consumer paths remain stable. Manifest and lockfile
changes transfer dependency ownership without widening a production item.
```

Replace the final authorization paragraph:

```markdown
The approved design does not authorize implementation. 8A, 8B, and 8C each
require a separate plan and explicit owner instruction.
```

with:

```markdown
Phase 8 is implemented and retained. The approved boundary, the separate 8A,
8B, and 8C plans, and the terminal verification record remain its authority.
```

After patching, require the exact rendered summary to occur once:

```powershell
$roadmapText = (
    Get-Content -Raw -LiteralPath `
        'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
) -replace "\r\n?", "`n"
$renderedRoadmapSummary = (
    Get-Content -Raw -LiteralPath $roadmapSummaryPreview
) -replace "\r\n?", "`n"
if ([regex]::Matches(
    $roadmapText,
    [regex]::Escape($renderedRoadmapSummary.TrimEnd())
).Count -ne 1) {
    throw 'Roadmap does not contain the exact rendered retained summary once'
}
```

- [ ] **Step 3: Update current architecture and the active terminal workflow-value entries**

Add this current-state paragraph beside the Prompt Pack and Analysis ownership paragraphs in `docs/project.md`:

```markdown
`src-tauri/crates/extractum-telegram/src/` owns retained low-level Telegram
runtime, session, media, live-peer/message/topic, and Takeout transport/domain
logic. `src-tauri/src/telegram_impl/lib.rs` is the behavior-free private app
facade; app-owned Tauri commands, persistence/transactions, source orchestration,
and the four fixture-consuming app-owned unit tests remain under `src-tauri/src/`.
The app enables the non-default `app-test-support` feature only through its
dev-dependency. Any external Cargo consumer can explicitly enable this public
feature; it is a development-isolation mechanism, not a security boundary.
```

Replace the complete `## Reading order for implementation work` numbered list with:

```markdown
1. `src-tauri/crates/extractum-telegram/src/` for retained low-level Telegram runtime, session, live-source, media, and Takeout logic
2. `src-tauri/src/telegram_impl/lib.rs` for the private application compatibility facade
3. `src-tauri/src/telegram.rs` for app-owned Telegram commands and orchestration
4. `src-tauri/src/takeout_import/mod.rs`
5. `src-tauri/src/takeout_import/raw_parse.rs`
6. `src-tauri/src/sources/mod.rs`
7. `src-tauri/src/source_ingest.rs`
8. `src-tauri/src/youtube/`
9. `src-tauri/crates/extractum-analysis/src/` for the portable analysis domain and owned persistence
10. `src-tauri/src/analysis/` for application adapters and command facade
11. `src-tauri/src/llm/`
12. `src-tauri/src/diagnostics/`
13. `src-tauri/crates/extractum-prompt-packs/src/`
14. `src-tauri/src/prompt_packs/` for application adapters and command facade
15. `src/routes/projects/`
16. `src/lib/components/research-projects/`
17. `research/youtube_pipeline/` for research-only YouTube summary pipeline work
18. `src/routes/analysis/+page.svelte`
19. `src/lib/components/analysis/`
20. `src/routes/settings/+page.svelte`
21. `src/routes/diagnostics/+page.svelte`
22. `src/lib/diagnostics-view-model.ts`
23. `src/routes/sources/+page.svelte`
24. `src-tauri/src/error.rs`
25. `src-tauri/src/migrations.rs`
```

Append these two rows to `## Telegram crate-preparation agent lifecycle` in `docs/value-registry.md`:

```markdown
| `8c-extracted` | agent-workflow lifecycle | Selects the retained Phase 8C `extractum-telegram` package layout. | `telegram-contract-paths.ts` | terminal | yes | none | tests and agent workflow |
| `done: retained` | agent-workflow status input | Maps the terminal Phase 8 roadmap disposition to `8c-extracted`. | `telegram-contract-paths.ts` | terminal | yes | none | tests and agent workflow |
```

These values already belong to the helper's closed vocabulary; this docs change records their newly active terminal mapping, not a broad backfill of unchanged historical or alternative helper values. No source vocabulary changes in 8C. The Telegram contract must accept these two rows' absence while roadmap status is pending and require their exact presence once status is terminal.

- [ ] **Step 4: Validate the terminal documentation state without rerunning environment-sensitive gates**

```powershell
$terminalContracts = Invoke-CheckedNative 'terminal Phase 8 contracts' {
    npm.cmd run test -- `
        src/lib/telegram-crate-boundary-contract.test.ts `
        src/lib/crate-extraction-shell-cap-contract.test.ts
}
$terminalContracts.Text | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'terminal-contracts.txt')
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Terminal documentation diff failed whitespace validation' }
```

Expected: the two status/evidence-sensitive contracts pass and the docs diff is clean. The complete coupled contracts, Grammers check, Rust gates, and full verifier already ran against the unchanged `EXTRACTION_COMMIT` tree; do not insert duplicate correctness, release, MCP, or startup gates between the successful startup smoke and the docs-only commit. If any Rust source, script, TypeScript contract, manifest, or lockfile changes at this point, stop: the docs-only boundary is broken and the previous verify/release/MCP/startup chain is no longer sufficient.

- [ ] **Step 5: Stage exactly six documentation files and create the terminal commit**

```powershell
$terminalDocPaths = @(
    'docs/superpowers/verification/2026-08-01-extractum-telegram-8c-extraction.md',
    'docs/superpowers/specs/2026-08-01-telegram-8c-extraction-design.md',
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md',
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md',
    'docs/project.md',
    'docs/value-registry.md'
)
git add -- $terminalDocPaths
if ($LASTEXITCODE -ne 0) { throw 'Failed to stage terminal Phase 8 documentation' }
$stagedTerminalPaths = @(git diff --cached --name-only | Sort-Object -Unique)
Assert-ExactStringSet -Label 'terminal docs-only path set' `
    -Expected $terminalDocPaths -Actual $stagedTerminalPaths
git diff --quiet
if ($LASTEXITCODE -ne 0) { throw 'Unstaged tracked changes remain before terminal commit' }
$untrackedAfterDocs = @(git ls-files --others --exclude-standard)
if ($untrackedAfterDocs.Count -ne 0) { $untrackedAfterDocs; throw 'Unexpected untracked files remain' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Terminal docs diff failed whitespace validation' }
$TERMINAL_DOCS_TREE = (git write-tree).Trim()
if ($TERMINAL_DOCS_TREE -notmatch '^[a-f0-9]{40}$') { throw 'Could not record terminal docs tree' }

git commit -m "docs: record Telegram crate extraction"
if ($LASTEXITCODE -ne 0) { throw 'Terminal Phase 8 documentation commit failed' }
$DOCS_COMMIT = (git rev-parse HEAD).Trim()
if ((git rev-parse "${DOCS_COMMIT}^{tree}").Trim() -ne $TERMINAL_DOCS_TREE) {
    throw 'Terminal docs commit tree differs from the reviewed docs index tree'
}
if ((git rev-parse "${DOCS_COMMIT}^").Trim() -ne $EXTRACTION_COMMIT) {
    throw 'Terminal docs commit is not directly atop EXTRACTION_COMMIT'
}
if ((git show -s --format=%s $DOCS_COMMIT).Trim() -ne 'docs: record Telegram crate extraction') {
    throw 'Unexpected terminal docs commit subject'
}
if (@(git status --porcelain=v1 --untracked-files=all).Count -ne 0) {
    throw 'Phase 8 terminal commit did not leave a clean working tree'
}
```

Expected: exactly two Phase 8C commits after the committed plan: one implementation commit and one docs-only terminal commit; the branch is clean and Phase 8 is `done: retained`.

## Completion Handoff

Report the literal `BASE_COMMIT`, `EXTRACTION_COMMIT`, and `DOCS_COMMIT`; the 17/2 content proof; 665/71/736 set result; focused/full/release/MCP/startup outcomes; and the verification-document path. Mention any recoverable rollback stash if one was created. Do not claim completion before the clean terminal commit exists.
