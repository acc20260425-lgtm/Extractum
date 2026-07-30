# Extractum Telegram Phase 8B Preparation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the separately green Phase 8B app-side Telegram boundary: build the exact portable `src-tauri/src/telegram_impl/**` tree, replace every raw live-source and Takeout consumer with owned operations, normalize the existing workspace dependencies without adding a package edge, and retain a contract-GREEN `extractum` application for the later mechanical 8C extraction.

**Architecture:** `extractum` remains the only owner package in this plan. The staged implementation owns the opaque Telegram runtime/session, Grammers transport, peer/message/topic conversion, and concrete Takeout operations. The application keeps Tauri commands/events, SQL and transactions, source/item/topic persistence, avatar paths/files/data URLs, Takeout jobs/cancellation/provenance, and terminal workflow decisions. The only module edge is private `#[path = "telegram_impl/lib.rs"] mod telegram_impl;`; staged modules use relative `self::`/`super::` paths, while application consumers use the exact `crate::telegram_impl::` prefix.

**Tech Stack:** Rust 2021, Tauri, Tokio, SQLx/SQLite, Grammers at pinned revision `1f901ce6e973fdcf0e74267f3d8efad5c729daaa`, `extractum-core`, `secrecy` 0.8, XChaCha20-Poly1305, Vitest/TypeScript source contracts, Node.js baseline generators, and PowerShell on Windows.

**Authority:** [`2026-07-26-telegram-crate-boundary-design.md`](../specs/2026-07-26-telegram-crate-boundary-design.md), [`2026-07-17-crate-roadmap.md`](../specs/2026-07-17-crate-roadmap.md), [`2026-07-26-extractum-telegram-8a-preparation.md`](2026-07-26-extractum-telegram-8a-preparation.md), [`2026-07-26-extractum-telegram-8a-preparation verification`](../verification/2026-07-26-extractum-telegram-8a-preparation.md), and [`2026-07-17-focused-rust-loop-design.md`](../specs/2026-07-17-focused-rust-loop-design.md).

**Current retained identity:** clean `main` at `dd14161386a946dfd070e25dae06bbb99ac62cfb`, synchronized with `origin/main`, with `719/719` unique `extractum` library tests and `128/128` Telegram boundary-contract tests passing. Execution may start at a later clean descendant that tracks this exact approved plan; it must never reset back to this SHA.

## Bounded API Clarification Requiring Plan Approval

The approved design deliberately requires the 8B plan to freeze every new
operation signature before RED, but it names no live-message batch value and
does not name all pure Takeout attempt/fallback values. Fresh current-flow and
pinned-Grammers analysis proves that these details cannot be omitted:

1. `sources/sync.rs` durably persists one iterator message at a time. Mapping
   an entire history, or failing a whole converted batch after earlier entries
   should have persisted, would change partial progress.
2. At Grammers revision
   `1f901ce6e973fdcf0e74267f3d8efad5c729daaa`, one
   `messages.getHistory` response is atomic and the high-level iterator page
   limit is 100. A loop over `MessageIter::next`, however, may perform a second
   invoke after a short `Slice`, so the seam must issue exactly one raw invoke
   per owned batch.
3. Peer resolution strategy remains app-owned. Once the app chooses an
   approved `PeerDescriptor`, staged operations reconstruct the private
   Grammers peer locally without repeating network resolution.
4. Takeout must record the export-DC attempt before each concrete remote call,
   expose fallback metadata after success/error/cancellation, and let the app
   persist an only-my-messages fallback before the explicit search
   continuation.
5. The dialog picker currently interleaves each dialog-page item with that
   peer's avatar request under one app-selected 4,000 ms budget. Returning all
   descriptors before fetching avatars would change remote call order and the
   set of photos attempted within the budget.
6. The pinned Grammers high-level history iterator updates its in-memory peer
   cache after each response. A raw single-invoke batch must reproduce that
   side effect before it returns owned messages.
7. That peer-cache side effect is conditional in pinned Grammers:
   `ClientConfiguration.auto_cache_peers` must be true before the inner
   `Peer::auth().is_some()` predicate is evaluated. Extractum currently gets
   `true` through `Client::new` and
   `grammers_client::client::ClientConfiguration::default()`, but relying on
   an implicit default would make the staged raw path diverge if that default
   changed.
8. Pinned Grammers `MessageIter::fill_buffer` panics if a
   `messages.getHistory` request with `hash = 0` returns
   `Messages::NotModified`. Existing Extractum Takeout page/count parsing
   instead rejects that response as a typed network error. The owned live
   boundary must make this panic-to-error divergence explicit rather than
   hide it behind the phrase "internal invariant."

Explicit approval of this plan also approves only the following clarification:

- add private-field, owned `LiveMessageBatch` and `LiveMessage` seam values;
- add one private-field opaque `DialogListing` whose `next()` preserves the
  existing dialog-next/avatar interleaving and returns one owned descriptor at
  a time under the app-selected budget;
- reconstruct the private Grammers peer locally from the already-approved
  `PeerDescriptor` fields (`source_subtype`, `external_id`, and `access_hash`)
  after the app-owned stored-hash → username → dialog resolution plan chooses
  the descriptor;
- add the private-field Takeout values `TakeoutPeer`, `TakeoutTransport`,
  `TakeoutAttempt`, `TakeoutFallback`, `TakeoutFallbackKind`, and
  `TakeoutCount` alongside the design-named `MessageRange`, `TakeoutPage`, and
  `TakeoutMessage`;
- allow stateful attempt/fallback draining on `TakeoutTransport`, including
  metadata retained when the concrete call returns `Err` or its future is
  cancelled;
- replace implicit `Client::new` construction with an explicitly materialized
  default `ClientConfiguration`, fail closed with the exact internal error
  `Grammers client configuration must enable auto_cache_peers` when
  `auto_cache_peers` is false, and pass the validated value to
  `Client::with_configuration`;
- keep `initialize_grammers_client` configuration-free at its call boundary:
  it accepts only `api_id` and `session`, and materializes/validates the
  default configuration inside its own body;
- intentionally replace pinned Grammers' `Messages::NotModified` panic on the
  live `hash = 0` history path with
  `AppError::network("Telegram returned messagesNotModified for live history batch")`,
  matching the existing typed Takeout rejection convention without accepting
  `NotModified.count`;
- allow Phase 8B and 8C plans to import, by exact committed section hash, the
  immutable literal 140-row map from the retained 8A plan instead of copying
  37,436 bytes and creating a second mutable authority.

This clarification adds no raw public type, dependency, owner, command, status,
or schema. It **does** add two concrete stateful opaque capabilities,
`DialogListing` and `TakeoutTransport`, to the approved boundary-handle list;
neither is a generic invocation abstraction and each exposes only the frozen
Telegram operations below.
Task 0 must amend the design's handle list, Takeout ownership clauses, staging
map, public allowlist, and rationale in a docs-only commit before any 8B
contract or production change. If the owner does not approve this explicit
architecture amendment, execution stops before Task 0 and the plan returns for
redesign.

## Global Constraints

- [ ] Do not execute this plan until the owner reviews it and explicitly authorizes Phase 8B. Approval of the boundary design or creation of this plan is not execution authority.
- [ ] Start only from a clean commit that tracks this exact plan. `git ls-files --error-unmatch docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md` must succeed.
- [ ] Record the actual start SHA and prove `dd14161386a946dfd070e25dae06bbb99ac62cfb` is an ancestor. Never reset, check out, or destructively restore the repository to that SHA.
- [ ] Inspect `git status --short`, the complete scoped diff, and the staged diff before every commit. Stage only the task allowlist; preserve unrelated user changes.
- [ ] Every Checkpoint 1–8 is independently GREEN, separately committed, and
  truthfully represented in the roadmap/design status in that same commit.
  Implementation and pre-status gates remain on the preceding retained status.
  The exact uncommitted `Checkpoint N retained` pair may be written only as the
  candidate lifecycle selector immediately before the final checkpoint gates;
  it is not retained until those gates pass and the same commit records it. A
  failed candidate gate restores the preceding pair before any further fix.
- [ ] Use the canonical shared `src-tauri/target`. Do not create a worktree, alternate target, timing harness, process scanner, quiet-window protocol, retry loop, or temporary Cargo profile.
- [ ] Phase 8B keeps exactly six workspace members. It creates no `extractum-telegram` package, member, manifest, path dependency, package metadata node, or resolved edge.
- [ ] All four Grammers roots remain direct dependencies of `extractum` throughout 8B. Dependency-removal benefit is not claimed before 8C.
- [ ] Do not edit migrations, schemas, frontend runtime/UI code, command signatures/registration, IPC/event payloads, persisted status values, secret identifiers, session format/path/AAD, transaction ownership, outer cancellation-select boundaries, message order/limits/cutoffs, fallback rules, or durable progress boundaries.
- [ ] Do not introduce SQL, Tauri, keyring, filesystem, app-module, `crate::error`, `crate::compression`, or `crate::time` imports into `src-tauri/src/telegram_impl/**`.
- [ ] Staged-to-staged references use only `self::` and `super::`. No staged
  file contains `crate::`, an application alias, or an application re-export.
  Raw Grammers types remain private implementation details, parameters/results
  of the literal final restricted bridge allowlist, or members of the exact
  CP3→CP6 transitional raw bridge frozen below; none is externally public or
  root-re-exported.
- [ ] Except for the exact CP3→CP6 package-private media compatibility facade
  and the exact CP3→CP6 application-facing members of the transitional raw
  bridge frozen below, every application reference to staged API uses an
  explicit `crate::telegram_impl::...` path. Do not add
  `use crate::telegram_impl as ...`, another facade, a glob, or a duplicate
  DTO.
- [ ] Public fallible operations return `extractum_core::error::AppResult<T>` directly. The root does not re-export `AppError`/`AppResult` and defines no `TelegramError`.
- [ ] The public API contains no `Client`, `MemorySession`, `LoginToken`, `PeerRef`, `RemoteCall`, `InvocationError`, raw `tl::*`, SQLx, Tauri, keyring, secret getter, or app type.
- [ ] At CP7/CP8, restricted `pub(super)` bridges are allowed only for the
  literal 67-entry final internal bridge allowlist frozen below. They are not
  root-re-exported. Before CP7, the only additional staged
  restricted/package bridges are the exact CP3→CP6 transitional raw bridge and
  exact CP3→CP4 three-constant peer-kind root bridge named in the next bullet.
  TypeScript does not authorize or reject those temporary definitions or uses;
  the mandatory LLM review does. Every externally reachable public item
  outside the frozen API remains forbidden.
- [ ] The only lifecycle-gated exceptions to the terminal API/visibility rules
  are: (a) the exact four-item CP3→CP6 media compatibility set, including its
  staged-root export and package-private `src-tauri/src/media.rs` facade for the
  exact frozen consumers; and (b) the exact CP3→CP6 transitional raw bridge
  `TelegramClientHandle::{raw_client,raw_session}`,
  `TelegramSession::raw_memory_session`,
  `telegram::{get_client,get_authorized_client}`, and
  `{ResolvedSyncPeer::peer,legacy_peer_ref_from_descriptor}`. The staged raw
  accessors retain only their existing package/super visibility and exact
  callsites; and (c) the exact CP3→CP4 package-private staged-root re-export of
  `{TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER}`
  for the sole `sources::sync::fallback_message_identity` consumer. Task 5
  removes (c) with that mapper. All exception sets are removed/demoted by their
  stated lifecycle and authorize no public module, glob, fifth media item, or
  additional raw/constant bridge.
- [ ] Remove `TelegramClientHandle::{raw_client,raw_session}` and app helpers `get_client`/`get_authorized_client` only after their last consumers use owned operations; none may exist at Checkpoint 7 or 8.
- [ ] App persistence remains incremental. It processes every `LiveMessage` and every `TakeoutMessage` in order, records max IDs before skip/parse decisions exactly as today, and returns a per-entry error only after prior entries have been durably handled.
- [ ] Every live batch performs exactly one raw `messages.getHistory` invoke with `1 <= limit <= 100`; do not build it by draining `MessageIter::next`. It preserves newest-to-oldest order, exact offset-id/offset-date advancement, the pinned Grammers terminal rule, and per-message conversion after app cutoffs.
- [ ] The raw live batch reproduces the pinned Grammers peer-map/session-cache update before returning and performs no second remote call.
- [ ] Grammers client construction explicitly materializes `grammers_client::client::ClientConfiguration::default()`, rejects `auto_cache_peers == false` with `AppError::internal("Grammers client configuration must enable auto_cache_peers")` before `SenderPool::new` or `tokio::spawn`, and calls `Client::with_configuration`. The raw live batch may reproduce the enabled cache side effect only under this fail-closed construction invariant.
- [ ] `initialize_grammers_client` retains the exact configuration-free signature frozen in Task 3. It must not accept `ClientConfiguration`, `auto_cache_peers`, a configuration factory, or any equivalent injected policy. Making the otherwise unreachable false-default branch testable by parameterizing this function moves the invariant to its caller, is forbidden, and requires an approved plan/design amendment.
- [ ] The one declared error-behavior exception is `Messages::NotModified` for `messages.getHistory { hash: 0 }`: pinned Grammers panics, while Extractum deliberately returns `AppErrorKind::Network` with exact message `Telegram returned messagesNotModified for live history batch`. This aligns live history with the existing Takeout page/count typed-rejection convention; it neither unwinds nor accepts `NotModified.count`. No other panic/error-kind/message/JSON behavior may change.
- [ ] The app records an export-DC attempt before each concrete remote call and drains fallback metadata after success, error, or cancellation. On a terminal remote error, failure to record export-DC fallback metadata is best-effort and the original remote error wins. Only-my-messages persistence is mandatory and completes before the explicit search continuation.
- [ ] Avatar download remains staged; avatar cache keys/paths/writes/cleanup and base64/data-URL presentation remain app-owned. The app creates `DialogListing` with the exact 4,000 ms list budget, converts each returned descriptor before requesting the next one, and staged `next()` preserves dialog-next/avatar interleaving plus the exact 750 ms per-photo timeout.
- [ ] The immutable 140 primaries and three declared companions remain the only baseline decompositions. Any fourth decomposition or identity rename stops for a plan/design amendment.
- [ ] Every exact Rust RED is a runtime RED: a compiling test executes exactly once, fails for the named semantic reason, and contains no compiler error. A compile failure or zero-test filter is never RED evidence.
- [ ] No live credentialed Telegram mutation is a gate. Deterministic tests prove login, source, session, and Takeout behavior.
- [ ] If implementation requires another public type, dependency, raw escape hatch, transaction owner, wire change, cross-domain port, or generic invocation/repository/event abstraction, stop before code and amend the approved design and plan.

### Phase 8B Status State Machine

Task 1 installs this closed vocabulary atomically in `telegram-contract-paths.ts`, the Telegram boundary contract, and the shell-cap contract:

1. roadmap `8A preparation retained`; design `Approved; 8A preparation retained; 8B not started` — starting state;
2. roadmap `8B preparation Checkpoint 1 retained` through
   `8B preparation Checkpoint 8 retained`; design
   `Approved; 8B preparation Checkpoint N retained` — the exact uncommitted
   pair selects the candidate lifecycle for the final named gates and becomes
   retained only when those gates pass and the checkpoint commit records it;
3. roadmap `8B preparation retained; 8C pending`; design `Approved; 8B preparation retained; 8C pending` — only after Checkpoint 8 release/startup evidence and the durable verification document;
4. `done: retained` and `not retained` remain future/rollback states and are not produced by this plan.

`TelegramLifecycle` gains exact `8b-checkpoint-1` through `8b-checkpoint-8` values; the existing terminal 8B layout becomes `8b-preparation`. Per-path resolution is checkpoint-aware:

| Staged owner path | First physical owner checkpoint |
| --- | ---: |
| `telegram_impl/{lib,dto,media,runtime,session}.rs` | 3 |
| `telegram_impl/live/{mod,avatar,peer}.rs` | 4 |
| `telegram_impl/live/{messages,topics}.rs` | 5 |
| `telegram_impl/error.rs` | 4 |
| `telegram_impl/takeout/**` | 7 |

Checkpoint 6 adds the two raw-parse companion assertions at their current `takeout_import::raw_parse::tests` temporary IDs; Checkpoint 7 moves them to the staged IDs. No lifecycle infers a path from existence alone.

## Rust Verification Loops

The only affected package is `extractum`. `extractum-core` is an unchanged dependency-contract subject. `extractum-telegram` must remain absent for the entire plan.

Start each execution session with fail-closed helpers:

```powershell
$ErrorActionPreference = 'Stop'

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
        # Windows PowerShell 5.1 surfaces redirected native stderr as
        # NativeCommandError/RemoteException. Keep that stream capturable while
        # preserving fail-fast behavior for PowerShell outside this tiny scope.
        $ErrorActionPreference = 'Continue'
        # A scriptblock that runs no native command must not inherit a sticky
        # LASTEXITCODE from an earlier command.
        $global:LASTEXITCODE = [int]::MinValue
        # Read LASTEXITCODE inside the redirecting wrapper. Reading it after
        # `& $Command 2>&1` in the parent scope can falsely retain zero.
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
    $capture.Text
    if ($capture.ExitCode -ne 0) {
        throw "$Label failed with exit code $($capture.ExitCode)"
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
    $exitCode = $capture.ExitCode
    $text = $capture.Text
    $text
    if ($exitCode -ne 0) { throw "Exact Rust test failed: $Package::$TestName" }
    if ($text -notmatch '(?m)^[ \t]*running 1 test[ \t]*\r?$') {
        throw "Exact Rust test was empty or ambiguous: $Package::$TestName"
    }
}

function Assert-ExactRustRuntimeRed {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$TestName,
        [Parameter(Mandatory = $true)][string]$SourcePath,
        [Parameter(Mandatory = $true)][string]$ExpectedSentinel
    )
    $leaf = ($TestName -split '::')[-1]
    if (-not (Select-String -LiteralPath $SourcePath -SimpleMatch $leaf -Quiet)) {
        throw "RED source does not contain exact test leaf: $TestName"
    }
    if (-not (Select-String -LiteralPath $SourcePath -SimpleMatch $ExpectedSentinel -Quiet)) {
        throw "RED source does not contain unique sentinel: $ExpectedSentinel"
    }
    if ($TestName.Contains($ExpectedSentinel)) {
        throw "RED sentinel must not occur in the test identity"
    }
    $capture = Invoke-CapturedNative -Label "exact Rust RED $Package::$TestName" -Command {
        cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib $TestName -- --exact
    }
    $exitCode = $capture.ExitCode
    $text = $capture.Text
    $text
    if ($exitCode -eq 0) { throw "Expected exact Rust runtime RED: $Package::$TestName" }
    if ($text -notmatch '(?m)^[ \t]*running 1 test[ \t]*\r?$') {
        throw "Exact Rust RED did not execute exactly one test: $Package::$TestName"
    }
    if ($text -match 'could not compile|error\[E[0-9]+\]') {
        throw "Exact Rust RED was a compile failure: $Package::$TestName"
    }
    if (-not $text.Contains($ExpectedSentinel)) {
        throw "Unexpected Rust runtime RED reason: $Package::$TestName"
    }
}

function Invoke-NonEmptyRustSuite {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$TestFilter
    )
    $capture = Invoke-CapturedNative -Label $Label -Command {
        cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib $TestFilter -- --nocapture
    }
    $exitCode = $capture.ExitCode
    $text = $capture.Text
    $text
    if ($exitCode -ne 0) { throw "$Label failed with exit code $exitCode" }
    $positiveRuns = @(
        [regex]::Matches($text, '(?m)^[ \t]*running ([1-9][0-9]*) tests?[ \t]*\r?$') |
            ForEach-Object { [int]$_.Groups[1].Value }
    )
    if ($positiveRuns.Count -eq 0) {
        throw "$Label executed zero tests or produced no recognized libtest count"
    }
}

function Assert-ExactRustIdentitySet {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$Prefix,
        [Parameter(Mandatory = $true)][string[]]$Expected
    )
    if ($Expected.Count -eq 0) { throw "Expected identity set is empty for $Prefix" }
    $expectedSorted = @($Expected | Sort-Object)
    if (@($expectedSorted | Select-Object -Unique).Count -ne $Expected.Count) {
        throw "Expected identity set contains duplicates for $Prefix"
    }
    if (@($expectedSorted | Where-Object { -not $_.StartsWith($Prefix) }).Count -ne 0) {
        throw "Expected identity lies outside prefix $Prefix"
    }
    $capture = Invoke-CapturedNative -Label "Cargo test list $Package::$Prefix" -Command {
        cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib -- --list
    }
    if ($capture.ExitCode -ne 0) {
        $capture.Text
        throw "Cargo test list failed for $Package::$Prefix"
    }
    $listed = @(
        $capture.Text -split "\r?\n" |
            ForEach-Object { if ($_ -match '^([^ ]+): test$') { $Matches[1] } }
    )
    $actual = @($listed | Where-Object { $_.StartsWith($Prefix) } | Sort-Object)
    if (@($actual | Select-Object -Unique).Count -ne $actual.Count) {
        throw "Actual Cargo identity set contains duplicates for $Prefix"
    }
    if (($actual -join "`n") -ne ($expectedSorted -join "`n")) {
        Compare-Object $expectedSorted $actual
        throw "Exact Rust identity set drifted for $Package::$Prefix"
    }
}

function Assert-NoMatches {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$Pattern,
        [Parameter(Mandatory = $true)][string[]]$Paths
    )
    $capture = Invoke-CapturedNative -Label $Label -Command {
        rg -n --glob '*.rs' -- $Pattern @Paths
    }
    $exitCode = $capture.ExitCode
    if ($capture.Text.Trim().Length -ne 0) { $capture.Text }
    if ($exitCode -eq 0) { throw "$Label found forbidden matches" }
    if ($exitCode -ne 1) { throw "$Label scan failed with exit code $exitCode" }
}

function Invoke-RetainedCheckpointGates {
    param([Parameter(Mandatory = $true)][string]$Label)
    Invoke-CheckedNative "$Label rustfmt" { npm.cmd run check:rustfmt }
    Invoke-CheckedNative "$Label metadata" {
        cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
    }
    $workspaceCheck = Measure-Command {
        Invoke-CheckedNative "$Label workspace check" {
            cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
        }
    }
    "checkpoint=$Label workspace_check_ms=$([math]::Round($workspaceCheck.TotalMilliseconds))"
    Invoke-CheckedNative "$Label workspace tests" {
        cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
    }
    Invoke-CheckedNative "$Label repository verify" { npm.cmd run verify }
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
    @($ids)
}

function Assert-RustPackageTestTotal {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][int]$ExpectedTotal
    )
    $ids = @(Get-RustTestIds -Package $Package)
    if ($ids.Count -ne $ExpectedTotal) {
        throw "Expected $ExpectedTotal unique $Package lib tests, found $($ids.Count)"
    }
}

function Assert-Phase8BFinalTestInventory {
    param(
        [Parameter(Mandatory = $true)][string]$AuthorityPath,
        [Parameter(Mandatory = $true)][int]$ExpectedPackageTotal
    )
    $authority = Get-Content -LiteralPath $AuthorityPath -Raw |
        ConvertFrom-Json
    $baseline = @($authority.baselineDerived)
    $preNewApp = @($authority.preNewApp)
    $preNewStaged = @($authority.preNewStaged)
    $newApp = @($authority.phase8BNewApp)
    $newStaged = @($authority.phase8BNewStaged)
    if (
        $baseline.Count -ne 143 -or
        $preNewApp.Count -ne 103 -or
        $preNewStaged.Count -ne 57 -or
        $newApp.Count -ne 1 -or
        $newStaged.Count -ne 14
    ) {
        throw "Phase 8B authority partition counts drifted"
    }
    if (
        @($preNewStaged + $newStaged |
            Where-Object { -not $_.StartsWith('telegram_impl::') }).Count -ne 0 -or
        @($preNewApp + $newApp |
            Where-Object { $_.StartsWith('telegram_impl::') }).Count -ne 0
    ) {
        throw "Phase 8B authority owner partition drifted"
    }
    $finalApp = @($preNewApp + $newApp)
    $finalStaged = @($preNewStaged + $newStaged)
    $tracked = @($finalApp + $finalStaged)
    $sets = @(
        [pscustomobject]@{ Name = 'baseline'; Values = $baseline }
        [pscustomobject]@{ Name = 'final app'; Values = $finalApp }
        [pscustomobject]@{ Name = 'final staged'; Values = $finalStaged }
        [pscustomobject]@{ Name = 'tracked union'; Values = $tracked }
    )
    foreach ($set in $sets) {
        if (@($set.Values | Select-Object -Unique).Count -ne $set.Values.Count) {
            throw "Phase 8B $($set.Name) authority contains a duplicate or overlap"
        }
    }
    if ($finalApp.Count -ne 104 -or $finalStaged.Count -ne 71 -or $tracked.Count -ne 175) {
        throw "Phase 8B final identity totals drifted"
    }
    if (@($baseline | Where-Object { $_ -notin $tracked }).Count -ne 0) {
        throw "Baseline-derived identity is absent from final tracked authority"
    }
    $actual = @(Get-RustTestIds -Package extractum)
    if ($actual.Count -ne $ExpectedPackageTotal) {
        throw "Expected $ExpectedPackageTotal unique extractum lib tests, found $($actual.Count)"
    }
    if (@($tracked | Where-Object { $_ -notin $actual }).Count -ne 0) {
        Compare-Object $tracked $actual
        throw "A tracked Phase 8B identity is absent from Cargo inventory"
    }
    Assert-ExactRustIdentitySet `
        -Package extractum `
        -Prefix 'telegram_impl::' `
        -Expected $finalStaged
}

$ExactStoreTests = @(
    'sources::store::tests::avatar_cache_key_skips_non_telegram_metadata'
    'sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents'
    'sources::store::tests::delete_source_is_blocked_when_source_is_used_by_project'
    'sources::store::tests::delete_source_waits_for_temporary_database_write_lock'
    'sources::store::tests::dialog_picked_channel_writes_dialog_typed_identity_with_access_hash'
    'sources::store::tests::dialog_picked_group_writes_dialog_dependent_typed_identity_without_access_hash'
    'sources::store::tests::dialog_picked_supergroup_writes_dialog_typed_identity_with_access_hash'
    'sources::store::tests::list_sources_exposes_migrated_history_counts_without_old_chat_identity'
    'sources::store::tests::list_sources_exposes_sanitized_migrated_history_status_without_chat_id'
    'sources::store::tests::load_source_returns_not_found_for_missing_source'
    'sources::store::tests::source_record_parts_allow_non_telegram_source'
    'sources::store::tests::source_record_parts_emit_only_source_subtype'
    'sources::store::tests::telegram_identity_allows_same_peer_on_different_accounts'
    'sources::store::tests::telegram_identity_rejects_same_account_peer_conflict_at_typed_boundary'
    'sources::store::tests::telegram_source_upsert_inserts_null_metadata'
    'sources::store::tests::telegram_source_upsert_preserves_existing_legacy_metadata_blob'
    'sources::store::tests::telegram_source_upsert_rolls_back_source_when_typed_identity_fails'
    'sources::store::tests::telegram_source_upsert_writes_required_identity_and_available_optional_fields'
    'sources::store::tests::upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind'
    'sources::store::tests::upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata'
    'sources::store::tests::upsert_youtube_video_source_conflict_clears_existing_legacy_blob'
    'sources::store::tests::upsert_youtube_video_source_handles_legacy_not_null_telegram_kind'
    'sources::store::tests::upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row'
    'sources::store::tests::upsert_youtube_video_source_writes_typed_row_and_null_source_metadata'
)

$Phase8BRedCases = @(
    [pscustomobject]@{
        TestName = 'telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check'
        SourcePath = 'src-tauri/src/telegram_impl/runtime.rs'
        Sentinel = 'PHASE8B_RED_RUNTIME_CLIENT_LOOKUP'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure'
        SourcePath = 'src-tauri/src/telegram_impl/live/avatar.rs'
        Sentinel = 'PHASE8B_RED_AVATAR_OWNED_TIMEOUT'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget'
        SourcePath = 'src-tauri/src/telegram_impl/live/peer.rs'
        Sentinel = 'PHASE8B_RED_DIALOG_AVATAR_INTERLEAVING'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes'
        SourcePath = 'src-tauri/src/telegram_impl/live/peer.rs'
        Sentinel = 'PHASE8B_RED_PEER_RESOLUTION_PRIMITIVES'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule'
        SourcePath = 'src-tauri/src/telegram_impl/live/messages.rs'
        Sentinel = 'PHASE8B_RED_MESSAGE_BATCH_AND_PEER_CACHE'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload'
        SourcePath = 'src-tauri/src/telegram_impl/live/messages.rs'
        Sentinel = 'PHASE8B_RED_LIVE_MESSAGE_OWNED_DRAFT'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor'
        SourcePath = 'src-tauri/src/telegram_impl/live/topics.rs'
        Sentinel = 'PHASE8B_RED_LIVE_TOPIC_PAGES'
    }
    [pscustomobject]@{
        TestName = 'sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error'
        SourcePath = 'src-tauri/src/sources/sync.rs'
        Sentinel = 'PHASE8B_RED_APP_LIVE_BATCH_COORDINATOR'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error'
        SourcePath = 'src-tauri/src/telegram_impl/takeout/transport.rs'
        Sentinel = 'PHASE8B_RED_TAKEOUT_TRANSPORT_METADATA'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges'
        SourcePath = 'src-tauri/src/telegram_impl/takeout/operations.rs'
        Sentinel = 'PHASE8B_RED_TAKEOUT_START_RANGES'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity'
        SourcePath = 'src-tauri/src/telegram_impl/takeout/operations.rs'
        Sentinel = 'PHASE8B_RED_TAKEOUT_MIGRATION_IDENTITY'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome'
        SourcePath = 'src-tauri/src/telegram_impl/takeout/operations.rs'
        Sentinel = 'PHASE8B_RED_TAKEOUT_COUNT_FALLBACK'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages'
        SourcePath = 'src-tauri/src/telegram_impl/takeout/operations.rs'
        Sentinel = 'PHASE8B_RED_TAKEOUT_PAGE_SEARCH'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping'
        SourcePath = 'src-tauri/src/telegram_impl/takeout/operations.rs'
        Sentinel = 'PHASE8B_RED_TAKEOUT_FINISH'
    }
    [pscustomobject]@{
        TestName = 'telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots'
        SourcePath = 'src-tauri/src/telegram_impl/takeout/forum_topics.rs'
        Sentinel = 'PHASE8B_RED_TAKEOUT_FORUM_TOPICS'
    }
)

function Invoke-Phase8BRedCases {
    param([Parameter(Mandatory = $true)][string[]]$TestNames)
    foreach ($testName in $TestNames) {
        $matches = @($Phase8BRedCases | Where-Object TestName -eq $testName)
        if ($matches.Count -ne 1) {
            throw "Expected one literal Phase 8B RED case for $testName"
        }
        $case = $matches[0]
        Assert-ExactRustRuntimeRed `
            -Package extractum `
            -TestName $case.TestName `
            -SourcePath $case.SourcePath `
            -ExpectedSentinel $case.Sentinel
    }
}
```

Every finite native command in this plan runs through
`Invoke-CheckedNative`, `Invoke-CapturedNative`, or one of the Rust helpers
that uses them. A later successful command never masks an earlier nonzero
exit. The only intentional long-running exception is the observed
`npm.cmd run tauri dev` process in Task 8C; its startup/exit is controlled by
the live MCP procedure and exact process cleanup rather than a finite wrapper.

Narrow RED/GREEN and package checkpoints are always:

```powershell
Invoke-ExactRustTest extractum 'telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check'
Invoke-CheckedNative 'extractum focused check' {
    cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
Invoke-CheckedNative 'extractum package checkpoint' {
    cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
```

Every retained checkpoint invokes `Invoke-RetainedCheckpointGates` once. Its mandatory workspace check is the checkpoint's only timing sample; the check inside `npm.cmd run verify` is correctness-only and its duration is ignored.

## Normative Immutable Test Authority

The immutable map is included here by exact content-addressed reference:

```text
source_path = docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
start_heading = ## Literal Immutable 140-Test Identity Map
end_marker = LF + ### Exact New-Test Identity Map
normalized_lf_bytes = 37436
sha256 = ceab6cef728d396bf2136207f2130974dee2cc0be3c5184eabd8c8de5e58b3ca
```

The retained 8A addition table is independently content-addressed:

```text
source_path = docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
start_heading = ### Exact New-Test Identity Map
end_marker = LF + --- + LF
normalized_lf_bytes = 3717
sha256 = a8dce5a0a00ac8cdcf83ef7eab2304f482e7c3967ec26ab8c8270d6fde42f539
```

Task 1 makes the contract normalize CRLF to LF, slice exactly between those headings, verify byte count and SHA-256 before parsing, and continue to require:

```text
140 immutable primaries = 99 app + 41 future-owner
3 companions = item-kind + 2 deferred raw-parse companions
143 eventual baseline-derived identities
18 Phase-8A additions = 1 companion + 17 additional verification
8A retained = 141 baseline-derived / 158 tracked
8B before new seam tests = 143 baseline-derived / 160 tracked
```

No task edits the imported table or its 8A exact-addition table. A hash mismatch stops execution and requires a reviewed authority amendment.

### Exact Phase 8B New-Test Table

These 15 seam tests are additional verification, not new baseline companions.
Fourteen belong to the staged future owner; one is the app-owned live batch
coordinator test required to preserve incremental durability and the raw
message limit:

| Checkpoint | Exact identity | Subject |
| ---: | --- | --- |
| 3 | `telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check` | non-authorized opaque lookup |
| 4 | `telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure` | 750 ms owned-byte avatar behavior |
| 4 | `telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget` | dialog/avatar interleaving, 4 s cutoff, order, and owned descriptors |
| 4 | `telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes` | app-owned plan primitives and exact outcomes |
| 5 | `telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule` | one raw invoke, cache update under the validated `auto_cache_peers` invariant, 1..=100 limit, offsets, ordering, terminal rule, typed `NotModified` rejection |
| 5 | `telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload` | per-entry conversion and empty skip |
| 5 | `telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor` | owned topics/deletions/pagination |
| 5 | `sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error` | app coordinator durability, raw-message limit, and no post-error fetch |
| 7 | `telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error` | attempt snapshot and fallback queue after success/error |
| 7 | `telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges` | self-check/init/split selection |
| 7 | `telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity` | migration detect/revalidate |
| 7 | `telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome` | classified fallback queue and owned only-my count |
| 7 | `telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages` | concrete page/search operations |
| 7 | `telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping` | concrete finish behavior |
| 7 | `telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots` | post-Takeout remote topic result |

The two deferred companions are separate and retain their design-mandated IDs:

```text
telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids
telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id
```

Starting from 719 current library identities, Phase 8B adds exactly 17 tests:
two companions plus 15 additional seam tests. The final executable library set
is therefore exactly 736 unique IDs. The Phase 8 tracked subset is exactly 175:
104 app identities and 71 staged future-owner identities. Any other count
requires a plan amendment before implementation continues.

Checkpoint accounting is exact:

| Retained checkpoint | Unique `extractum` library IDs | Present baseline-derived | Present Phase-8 tracked |
| ---: | ---: | ---: | ---: |
| 1 | 719 | 141 | 158 |
| 2 | 719 | 141 | 158 |
| 3 | 720 | 141 | 159 |
| 4 | 723 | 141 | 162 |
| 5 | 727 | 141 | 166 |
| 6 | 729 | 143 | 168 |
| 7 | 736 | 143 | 175 |
| 8 | 736 | 143 | 175 |

## Exact Portable Tree

Checkpoint 7 must produce exactly these 19 Rust files and no other file below the root:

```text
src-tauri/src/telegram_impl/
  lib.rs
  dto.rs
  error.rs
  runtime.rs
  session.rs
  media.rs
  live/
    mod.rs
    avatar.rs
    peer.rs
    messages.rs
    topics.rs
  takeout/
    mod.rs
    types.rs
    transport.rs
    export_dc.rs
    operations.rs
    pagination.rs
    raw_parse.rs
    forum_topics.rs
```

Checkpoint 8 writes `src/lib/telegram-8b-staging-sha256.json` with schema:

```json
{
  "schemaVersion": 1,
  "algorithm": "sha256",
  "root": "src-tauri/src/telegram_impl",
  "files": [
    { "path": "dto.rs", "sha256": "generated lowercase SHA-256" }
  ]
}
```

The real artifact contains 19 records sorted by forward-slash relative path. `scripts/telegram-staging-sha256.mjs --write` is the only generator; `--check` recomputes from bytes and fails on a missing/extra path, order drift, root drift, or hash mismatch. The 8C plan must compare the same relative-path/hash records against `src-tauri/crates/extractum-telegram/src`.

## Frozen Final Public API

Task 0 copies this exact signature authority into the design. Task 1 makes the
source contract reject any other terminal root re-export, `pub` item, public
field, or public method and lifecycle-gates the exact CP3→CP6 media exception
described below.

```rust
// telegram_impl/dto.rs
pub const ITEM_KIND_TELEGRAM_MESSAGE: &str = "telegram_message";

pub struct TelegramMessageIdentity {
    pub history_peer_kind: String,
    pub history_peer_id: i64,
    pub telegram_message_id: i64,
    pub migration_domain: Option<String>,
    pub is_migrated_history: bool,
}
impl TelegramMessageIdentity {
    pub fn validate(&self) -> extractum_core::error::AppResult<()>;
}

pub struct TelegramItemContext {
    pub reply_to_msg_id: Option<i64>,
    pub reply_to_peer_kind: Option<String>,
    pub reply_to_peer_id: Option<String>,
    pub reply_to_top_id: Option<i64>,
    pub reaction_count: Option<i64>,
}

pub struct TelegramMessageDraft {
    pub telegram_identity: Option<TelegramMessageIdentity>,
    pub telegram_context: TelegramItemContext,
    pub content: Option<String>,
    pub content_kind: &'static str,
    pub author: Option<String>,
    pub published_at: i64,
    pub raw_data: Vec<u8>,
    pub item_kind: String,
    pub media: Option<TelegramMediaPayload>,
}

pub struct PeerDescriptor {
    pub external_id: String,
    pub title: String,
    pub source_subtype: String,
    pub is_member: bool,
    pub username: Option<String>,
    pub access_hash: Option<i64>,
    pub avatar_bytes: Option<Vec<u8>>,
}

pub struct ForumTopicSnapshot {
    pub topic_id: i64,
    pub top_message_id: i64,
    pub title: String,
    pub icon_color: i64,
    pub icon_emoji_id: Option<i64>,
    pub is_closed: bool,
    pub is_pinned: bool,
    pub is_hidden: bool,
    pub sort_order: i64,
}

// telegram_impl/media.rs
pub struct TelegramMediaPayload {
    pub kind: String,
    pub metadata: extractum_core::media_metadata::ItemMediaMetadata,
}

// telegram_impl/session.rs
pub struct SessionEncryptionKey(/* private secret container */);
impl SessionEncryptionKey {
    pub fn try_from_encoded(
        encoded: secrecy::SecretString,
    ) -> extractum_core::error::AppResult<Self>;
    pub fn generate() -> (Self, secrecy::SecretString);
}

pub struct TelegramSession { /* private Grammers session */ }
impl TelegramSession {
    pub fn empty() -> Self;
    pub(super) fn clone_memory_session(
        &self,
    ) -> std::sync::Arc<grammers_session::storages::MemorySession>;
}

pub fn session_json_requires_existing_key(
    json: &str,
) -> extractum_core::error::AppResult<bool>;
pub fn decode_session_json(
    json: &str,
    account_id: i64,
    key: Option<&SessionEncryptionKey>,
) -> extractum_core::error::AppResult<TelegramSession>;
pub async fn encode_session_json(
    session: &TelegramSession,
    account_id: i64,
    key: &SessionEncryptionKey,
) -> extractum_core::error::AppResult<String>;

// telegram_impl/runtime.rs
pub struct TelegramApiHash(/* private secret container */);
impl TelegramApiHash {
    pub fn new(value: secrecy::SecretString) -> Self;
}

pub enum TelegramRuntimeStatus {
    Ready,
    ReauthRequired,
}

pub struct TelegramClientHandle { /* private client/session capability */ }
pub struct TelegramLoginAttempt { /* private login token and phone */ }
pub struct TelegramRuntime { /* private account map/test callbacks */ }

impl TelegramRuntime {
    pub fn new() -> Self;
    pub async fn initialize_account(
        &self,
        account_id: i64,
        api_id: i32,
        api_hash: TelegramApiHash,
        session: TelegramSession,
    ) -> extractum_core::error::AppResult<TelegramRuntimeStatus>;
    pub async fn is_authenticated(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<bool>;
    pub async fn request_login_code(
        &self,
        account_id: i64,
        phone: String,
    ) -> extractum_core::error::AppResult<()>;
    pub async fn sign_in(
        &self,
        account_id: i64,
        code: String,
    ) -> extractum_core::error::AppResult<TelegramSession>;
    pub async fn client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle>;
    pub async fn authorized_client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle>;
    pub async fn clear_account(&self, account_id: i64, sign_out: bool);
}

// telegram_impl/live/peer.rs
pub struct DialogListing { /* private dialog iterator/start/budget/client */ }

// telegram_impl/live/messages.rs
pub struct LiveMessageBatch { /* private messages/offset/terminal fields */ }
pub struct LiveMessage { /* private raw message plus owned peer lookup */ }

impl LiveMessageBatch {
    pub fn take_messages(&mut self) -> Vec<LiveMessage>;
    pub fn is_terminal(&self) -> bool;
    pub fn next_offset_id(&self) -> i32;
    pub fn next_offset_date(&self) -> i32;
}
impl LiveMessage {
    pub fn message_id(&self) -> i64;
    pub fn published_at(&self) -> i64;
    pub fn into_draft(
        self,
        source_title: Option<&str>,
    ) -> extractum_core::error::AppResult<Option<TelegramMessageDraft>>;
}

// telegram_impl/runtime.rs; delegates through live::* restricted facades
impl TelegramClientHandle {
    pub fn dialog_listing(
        &self,
        avatar_budget_ms: u64,
    ) -> DialogListing;
    pub async fn resolve_dialog_peer(
        &self,
        peer_id: i64,
        expected_subtype: Option<&str>,
    ) -> extractum_core::error::AppResult<PeerDescriptor>;
    pub async fn resolve_username(
        &self,
        username: &str,
        expected_subtype: Option<&str>,
    ) -> extractum_core::error::AppResult<Option<PeerDescriptor>>;
    pub async fn peer_avatar_bytes(
        &self,
        peer: &PeerDescriptor,
    ) -> Option<Vec<u8>>;
    pub async fn fetch_message_batch(
        &self,
        peer: &PeerDescriptor,
        offset_id: i32,
        offset_date: i32,
        limit: usize,
    ) -> extractum_core::error::AppResult<LiveMessageBatch>;
    pub async fn fetch_forum_topics(
        &self,
        peer: &PeerDescriptor,
    ) -> extractum_core::error::AppResult<
        Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>,
    >;
}

impl DialogListing {
    pub async fn next(
        &mut self,
    ) -> extractum_core::error::AppResult<Option<PeerDescriptor>>;
}

// telegram_impl/takeout/types.rs
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TakeoutFallbackKind {
    ExportDc,
    OnlyMyMessages,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TakeoutAttempt { /* private home_dc_id/export_dc_id fields */ }
impl TakeoutAttempt {
    pub fn home_dc_id(&self) -> i32;
    pub fn export_dc_id(&self) -> i32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TakeoutFallback { /* private kind/warning/provenance fields */ }
impl TakeoutFallback {
    pub fn kind(&self) -> TakeoutFallbackKind;
    pub fn warning(&self) -> &str;
    pub fn provenance_message(&self) -> Option<&str>;
}

// telegram_impl/takeout/transport.rs
pub struct TakeoutTransport { /* private client/DC/fallback state */ }
impl TakeoutTransport {
    pub fn export_dc_attempt(&self) -> TakeoutAttempt;
    pub fn drain_fallbacks(&mut self) -> Vec<TakeoutFallback>;
}

// telegram_impl/takeout/types.rs
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TakeoutPeer { /* private subtype/kind/id/access-hash fields */ }
impl TakeoutPeer {
    pub fn from_descriptor(
        descriptor: &PeerDescriptor,
    ) -> extractum_core::error::AppResult<Self>;
    pub fn peer_kind(&self) -> &'static str;
    pub fn peer_id(&self) -> i64;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageRange { /* private min/max fields */ }
impl MessageRange {
    pub fn min_id(&self) -> i32;
    pub fn max_id(&self) -> i32;
}

pub struct TakeoutCount { /* private count/only-my fields */ }
impl TakeoutCount {
    pub fn count(&self) -> i64;
    pub fn only_my_messages(&self) -> bool;
}

pub struct TakeoutPage { /* private cursor/messages/continuation fields */ }
impl TakeoutPage {
    pub fn take_messages(&mut self) -> Vec<TakeoutMessage>;
    pub fn has_next(&self) -> bool;
    pub fn only_my_messages(&self) -> bool;
    pub fn take_pagination_fallback_warning(&mut self) -> Option<String>;
}

pub struct TakeoutMessage { /* private raw TL message */ }
impl TakeoutMessage {
    pub fn message_id(&self) -> i64;
    pub fn into_draft(
        self,
        source_title: Option<&str>,
    ) -> extractum_core::error::AppResult<Option<TelegramMessageDraft>>;
}

// telegram_impl/runtime.rs; delegates through takeout::* restricted facades
impl TelegramClientHandle {
    pub async fn takeout_self_check(
        &self,
    ) -> extractum_core::error::AppResult<()>;
    pub async fn prepare_takeout(
        &self,
    ) -> extractum_core::error::AppResult<TakeoutTransport>;
    pub async fn takeout_forum_topics(
        &self,
        peer: &PeerDescriptor,
    ) -> extractum_core::error::AppResult<
        Option<(Vec<ForumTopicSnapshot>, Vec<i64>)>,
    >;
}

impl TakeoutTransport {
    pub async fn init(
        &mut self,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<i64>;
    pub async fn message_ranges(
        &mut self,
        takeout_id: i64,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<(i64, Vec<MessageRange>)>;
    pub async fn validate_peer(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<()>;
    pub async fn detect_supergroup_migration(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<Option<i64>>;
    pub async fn revalidate_migrated_peer(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
    ) -> extractum_core::error::AppResult<Option<(i64, TakeoutPeer)>>;
    pub async fn history_count(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        source_subtype: &str,
    ) -> extractum_core::error::AppResult<TakeoutCount>;
    pub async fn search_my_history_count(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
    ) -> extractum_core::error::AppResult<TakeoutCount>;
    pub async fn history_page(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        count: &TakeoutCount,
        previous: Option<&TakeoutPage>,
    ) -> extractum_core::error::AppResult<TakeoutPage>;
    pub async fn search_my_history_page(
        &mut self,
        takeout_id: i64,
        peer: &TakeoutPeer,
        range: &MessageRange,
        count: &TakeoutCount,
        previous: Option<&TakeoutPage>,
    ) -> extractum_core::error::AppResult<TakeoutPage>;
    pub async fn finish(
        &mut self,
        takeout_id: i64,
        success: bool,
    ) -> extractum_core::error::AppResult<()>;
}
```

`TelegramApiHash`, `TelegramRuntimeStatus`, `TelegramClientHandle`,
`DialogListing`,
`TelegramLoginAttempt`, `TelegramRuntime`, `SessionEncryptionKey`,
`TelegramSession`, `session_json_requires_existing_key`,
`decode_session_json`, and `encode_session_json` retain their Phase 8A
external visibility/signatures. There are two explicit internal/API-surface
changes: add public `TelegramRuntime::client`, and add restricted
`TelegramSession::clone_memory_session(&self) -> Arc<MemorySession>` while the
borrowed restricted
`TelegramSession::raw_memory_session(&self) -> &Arc<MemorySession>` coexists
only through CP6 and is deleted at CP7. This is a coexist-then-replace
transition, not an in-place signature change. `initialized_client` stays
private and `authorized_client` stays public.
Private derives and test-only constructors are allowed; no additional
externally reachable production `pub` item exists at CP7/CP8. The exact
CP3→CP6 media compatibility exports and the exact package-private CP3→CP4
peer-kind constant exports named below are the only lifecycle-gated staged-root
exceptions; each is removed/demoted at its stated checkpoint.

The exact restricted internal bridge allowlist is below. A name on the
`live::{...}` or `takeout::{...}` line is a `pub(super)` parent facade called
only by `runtime.rs`, except that
`takeout/forum_topics.rs` is the one additional caller of
`live::fetch_forum_topics`. A leaf name is `pub(super)` to its immediate
parent. Public inherent methods listed in the API block are not duplicated
here.

```text
live::{dialog_listing,resolve_dialog_peer,resolve_username,peer_avatar_bytes,fetch_message_batch,fetch_forum_topics}
live::avatar::{peer_photo_bytes_with_timeout,peer_photo_bytes}
live::peer::{DialogListing::new,resolve_dialog_peer,resolve_username,peer_avatar_bytes}
live::messages::fetch_message_batch
live::topics::fetch_forum_topics
media::{derive_content_kind,derive_document_media_kind_from_parts,extract_item_payload}
error::{is_non_forum_topic_refresh_error,is_channel_private_error,should_fallback_export_dc_error}
session::TelegramSession::{clone_memory_session,cache_peer_infos}
takeout::{takeout_self_check,prepare_takeout,takeout_forum_topics}
takeout::export_dc::{prepare_export_dc_alias,export_dc_invoke,finish_takeout_session}
takeout::forum_topics::takeout_forum_topics
takeout::operations::takeout_self_check
takeout::pagination::TakeoutPaginationProfile
takeout::pagination::TakeoutPageRequest
takeout::pagination::TakeoutPageRequest::offset_id
takeout::pagination::TakeoutPageRequest::add_offset
takeout::pagination::TakeoutPageRequest::limit
takeout::pagination::TakeoutPaginationCursor
takeout::pagination::TakeoutPaginationCursor::new
takeout::pagination::TakeoutCursorAdvance
takeout::pagination::TakeoutCursorAdvance::cursor
takeout::pagination::TakeoutCursorAdvance::advanced
takeout::pagination::TakeoutCursorAdvance::reached_range_start
takeout::pagination::TakeoutPaginationFallbackReason
takeout::pagination::select_history_splits
takeout::pagination::takeout_page_request
takeout::pagination::next_takeout_cursor
takeout::pagination::should_restart_with_descending_fallback
takeout::pagination::takeout_pagination_fallback_warning
takeout::pagination::parse_takeout_page
takeout::raw_parse::{parse_raw_message,peer_ref_identity,messages_response_count}
takeout::transport::TakeoutTransport::{new,queue_fallback,client,session,home_dc_id,export_dc_id}
takeout::types::TakeoutAttempt::new
takeout::types::TakeoutFallback::new
takeout::types::TakeoutPeer::{new,source_subtype,access_hash}
takeout::types::MessageRange::new
takeout::types::TakeoutCount::new
takeout::types::TakeoutPage::{from_parts,pagination_state}
takeout::types::TakeoutMessage::from_raw
```

The pagination subsection is intentionally one fully qualified symbol per
line; no nested brace syntax is legal. Task 1's
`scripts/telegram-8b-symbol-map.mjs` parses this entire fenced allowlist,
expands only the remaining single-level brace groups, and materializes the
sorted canonical 67-entry result as `restrictedFinalSymbols` in the generated
symbol artifact. The TypeScript contract consumes that generated array instead
of maintaining or parsing a second pagination list. Flattening also
deliberately closes two omissions in the old nested notation by listing the
`TakeoutPageRequest` and `TakeoutCursorAdvance` types themselves. This is not
formatting-only drift: the sibling-visible `takeout_page_request` and
`next_takeout_cursor` functions return those types, respectively, so the types
must themselves be `pub(super)`. The separate `pagination_state` bridge retains
its frozen `(TakeoutPaginationProfile, TakeoutPaginationCursor, usize)` return
tuple.

Every listed bridge is spelled `pub(super)` at its leaf or is reached through
one `pub(super)` parent facade; no `pub(crate)`, `pub(in ...)`, root export, or
other restricted production visibility is allowed. This keeps staged files
free of `crate::` and lets sibling modules collaborate without expanding the
future crate API. This paragraph governs the CP7/CP8 final inventory. The exact
CP3→CP6 raw bridge is a separately generated transitional inventory, may retain
only the existing package/super visibility frozen above, and is excluded from
`restrictedFinalSymbols`.

The final media bridge that replaces the transitional `DocumentSignals`
cross-module construction is exact:

```rust
pub(super) fn derive_document_media_kind_from_parts(
    mime_type: Option<&str>,
    has_video: bool,
    has_audio: bool,
    is_voice: bool,
    is_animated: bool,
) -> &'static str;
```

`DocumentSignals` and all five fields are private again at CP7. The bridge
reuses the existing classifier and introduces no second classification rule.

The pagination types on the allowlist are `pub(super)` only to the `takeout`
parent; they are neither root exports nor public API. `TakeoutPage` stores that
restricted state and exposes it to sibling operations only as:

```rust
pub(super) fn pagination_state(
    &self,
) -> (
    TakeoutPaginationProfile,
    TakeoutPaginationCursor,
    usize,
);
```

This is the sole internal cursor-state bridge and does not authorize a public
cursor/stream.

The live batch implementation is additionally frozen as follows:

- `fetch_message_batch` validates `1 <= limit <= 100`, constructs exactly one
  raw `tl::functions::messages::GetHistory` request with the supplied
  `offset_id` and `offset_date`, and performs exactly one `Client::invoke`;
- it maps the response's messages/users/chats into private owned lookup state
  sufficient to preserve current author, reply, media, raw-data, and fallback
  identity behavior without a second remote call;
- before returning, it reproduces pinned Grammers `build_peer_map` behavior and
  updates the runtime's in-memory `TelegramSession` peer cache only after
  runtime construction has validated that
  `ClientConfiguration.auto_cache_peers` is true, and then for exactly those
  response peers for which pinned Grammers `Peer::auth().is_some()` is true.
  This preserves both levels of the pinned condition. The inner predicate
  includes ordinary chats with default auth and excludes min users/channels
  whose auth is unavailable; it is not an access-hash-only predicate;
- newest-to-oldest response order is retained;
- `Messages::Messages` is terminal; `Messages::Slice` and
  `Messages::ChannelMessages` are terminal exactly when empty or when the first
  returned message ID is less than or equal to the request limit, matching the
  pinned Grammers `fill_buffer` rule;
- for this `GetHistory` request whose `hash` is exactly zero,
  `Messages::NotModified` is the one intentional divergence from pinned
  `fill_buffer`: pinned Grammers panics, whereas the owned live operation
  returns `AppErrorKind::Network` with exact message
  `Telegram returned messagesNotModified for live history batch`. It does not
  use `NotModified.count`. This matches the existing Extractum Takeout
  convention, whose operation-specific messages remain exactly
  `Telegram returned messagesNotModified for Takeout history page` and
  `Telegram returned messagesNotModified for Takeout history count probe`;
- a non-terminal batch advances both offsets from its last raw message;
- the app checks prior-ID and date cutoffs before `into_draft`, then updates
  `max_message_id`, converts, and persists/skips in that order. It never asks
  for the next batch after a cutoff or conversion/persistence error.

The Takeout transition protocol is also part of the API authority:

1. before each standalone concrete `TakeoutTransport` call, the app performs
   the existing cancellation pre-check and durably deduplicates both
   `transport.export_dc_attempt().home_dc_id()` and
   `transport.export_dc_attempt().export_dc_id()`. For a classified
   history-to-search pair, the one outer pre-check covers both calls; the app
   records the current attempt again immediately before search but performs no
   second cancellation pre-check or select;
2. one existing outer `run_takeout_step_with_cancel` future contains the
   concrete history call and, when needed, fallback persistence plus the
   explicit only-my search continuation; no second cancellation select or
   pre-check is inserted between them;
3. the app captures the outer future result and every completed recorder result
   without `?`; after success, error, or cancellation it drains all fallback
   metadata and applies the exact `FallbackRecordState` protocol in Task 7B
   before propagating the captured result. A completed recorder failure is
   never called again;
4. `validate_peer` may return `Ok(())` with an `OnlyMyMessages` fallback;
5. `history_count` or `history_page` returns the classified channel-private
   error and queues `OnlyMyMessages`; after the app durably records that
   fallback, it immediately calls the corresponding
   `search_my_history_count` or `search_my_history_page` concrete operation;
6. no progress event, durable page update, second select/pre-check, or unrelated
   job-state decision is inserted between that fallback record and search
   continuation; dropping the one outer future preserves the existing
   cancellation point;
7. a validation-level `OnlyMyMessages` warning is metadata only: every range
   still tries `history_count` first. Only a classified history error makes the
   returned `TakeoutCount::only_my_messages()` route that range's pages through
   search;
8. shifted-export-DC → home-DC remains internal to one concrete call. Its
   `ExportDc` fallback is drained after either success or error;
9. pagination fallback remains page-local, may recur for separate ranges, and
   is exposed only through `TakeoutPage::take_pagination_fallback_warning`.

The root re-export allowlist is exactly:

```text
ITEM_KIND_TELEGRAM_MESSAGE
TelegramMessageIdentity
TelegramItemContext
TelegramMessageDraft
PeerDescriptor
ForumTopicSnapshot
TelegramMediaPayload
TelegramApiHash
TelegramRuntimeStatus
TelegramClientHandle
DialogListing
TelegramLoginAttempt
TelegramRuntime
SessionEncryptionKey
TelegramSession
session_json_requires_existing_key
decode_session_json
encode_session_json
LiveMessageBatch
LiveMessage
TakeoutAttempt
TakeoutFallbackKind
TakeoutFallback
TakeoutPeer
TakeoutTransport
MessageRange
TakeoutCount
TakeoutPage
TakeoutMessage
```

There are no public modules or glob exports. `DocumentSignals`, all
media/content helpers, raw peer/TL/request/response/pagination types, protocol
classifiers, session envelopes, test callbacks, and constructors not named
above remain private or appear only on the exact restricted bridge allowlist.

At CP3 through CP6 only, `telegram_impl/lib.rs` additionally re-exports this
exact compatibility set for the not-yet-moved app consumers:

```text
DocumentSignals (including its five current fields)
derive_content_kind
derive_document_media_kind
extract_item_payload
```

`src-tauri/src/media.rs` re-exports that same set package-privately during the
transition. Separately, at CP3 and CP4 only, `telegram_impl/lib.rs`
package-privately re-exports
`{TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER}`
for the sole `sources::sync::fallback_message_identity` consumer. Task 5 moves
that mapper into `telegram_impl/live/messages.rs` and removes all three root
exports. No other transitional staged-root item is legal. Task 5 also removes
the `extract_item_payload` live-sync consumer; Task 7 moves raw parsing,
deletes the remaining app compatibility re-exports, removes the four media
root exports, and demotes the leaf helpers/fields to their final private or
exact `pub(super)` visibility before CP7 is retained.

## Symbol-Level Source Map

The two file/responsibility tables immediately below are a readable layout
summary only. They do not authorize a catch-all move, an unnamed helper, or
checkpoint drift. The literal production-symbol disposition table that
follows them is normative and is materialized by
`scripts/telegram-8b-symbol-map.mjs`.

| Final staged owner | Current production source and exact responsibility |
| --- | --- |
| `telegram_impl/lib.rs` | New private module shell and curated exports; replaces the future-owner module declarations/re-exports at the top of `telegram.rs`. |
| `telegram_impl/dto.rs` | Move all production/tests from `telegram/dto.rs`; move `ResolvedTelegramSource` into public `PeerDescriptor`; move `ForumTopicSnapshot` from `sources/topics.rs`. |
| `telegram_impl/error.rs` | Private shell at CP4; move `is_non_forum_topic_refresh_error` at CP5, then `is_channel_private_error` and export-DC local-transport classification at CP7; terminal `AppError` construction only. |
| `telegram_impl/runtime.rs` | Move all production/tests from `telegram/runtime.rs`; add `TelegramRuntime::client`; own every public `TelegramClientHandle` method and delegate through the exact restricted live/Takeout parent facades; remove raw client/session adapters at Checkpoint 7. |
| `telegram_impl/session.rs` | Move all production/tests from `telegram/session.rs`, including the opaque session/key, envelope/legacy codec, AAD, base64, encryption logic, and permanent restricted `clone_memory_session`/peer-cache bridges. |
| `telegram_impl/media.rs` | Move all production/tests from `telegram/media.rs`, including the Grammers media adapter and private content/media classifiers. |
| `telegram_impl/live/mod.rs` | Private live module shell. |
| `telegram_impl/live/avatar.rs` | Move `TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS`, `peer_photo_bytes_with_timeout`, and raw avatar download from `sources/avatar.rs`; expose owned bytes only. |
| `telegram_impl/live/peer.rs` | Own `DialogListing` and its public `next`; move username/dialog transport lookup, raw peer mapping/reconstruction, subtype/member/access-hash helpers, and budgeted dialog/avatar listing from `sources/{peer_resolution,identity,store}.rs`. |
| `telegram_impl/live/messages.rs` | Move raw history fetch and message/media/author/reply/raw-payload/fallback-identity adaptation from `sources/{sync,items}.rs`; own the single-invoke batch seam. |
| `telegram_impl/live/topics.rs` | Move `fetch_all_forum_topics`, cursor/date helpers, raw mapping, and the private topic fetch shared with Takeout. |
| `telegram_impl/takeout/mod.rs` | Private Takeout module shell and curated internal exports. |
| `telegram_impl/takeout/types.rs` | Own `TakeoutAttempt`, `TakeoutFallback*`, `TakeoutPeer`, `MessageRange`, `TakeoutCount`, `TakeoutPage`, and `TakeoutMessage`; `TakeoutPage` stores only the exact sibling-restricted cursor/profile state and exposes the frozen tuple bridge. |
| `telegram_impl/takeout/transport.rs` | Own client/session/DC state, attempt snapshots, and fallback queueing; expose only the exact restricted accessors used by concrete siblings. |
| `telegram_impl/takeout/export_dc.rs` | Move the raw, provenance-free half of `export_dc_invoke_with_provenance`, `ExportDcAlias`, DC selection, init request construction, shifted/home invocation/fallback mechanics, and all seven mapped tests. |
| `telegram_impl/takeout/operations.rs` | Move self-check, init/finish, validation, migration, split/count/history/search operations, and only-my classification. |
| `telegram_impl/takeout/pagination.rs` | Move all production/tests from `takeout_import/pagination.rs`; own split selection, the exact `pub(super)` TDesktop/descending state types, request/advance fields, restart/warning, and response classification. |
| `telegram_impl/takeout/raw_parse.rs` | Move all production/tests from `takeout_import/raw_parse.rs`, `peer_ref_identity`, raw response count/classification, and both companion tests. |
| `telegram_impl/takeout/forum_topics.rs` | Own the post-Takeout remote topic operation and delegate by relative path to the private live topic fetcher. |

Application residual ownership is exact:

| App file | Residual responsibility |
| --- | --- |
| `src-tauri/src/lib.rs` | App composition/commands and `#[path = "telegram_impl/lib.rs"] mod telegram_impl;`. |
| `src-tauri/src/telegram.rs` | Wire-status mapping, account/credential SQL and secret lookup, managed state, restore, events, and six `tg_*` commands; delete app client lookup helpers. |
| `src-tauri/src/telegram_session_store.rs` | App-data path, keyring, temp/atomic file lifecycle, legacy migration orchestration, and adapter tests. |
| `src-tauri/src/media.rs` | Provider-neutral core re-exports only at CP7/CP8; its exact four-item Telegram compatibility facade is transitional through CP6. |
| `src-tauri/src/sources/store.rs` | Tauri commands, the exact 4,000 ms avatar-budget input, avatar presentation/cache/persistence/SQL, and UI DTO mapping. |
| `src-tauri/src/sources/items.rs` | SQL, transactions, insert/upsert/list/read models, `PreparedSourceItem`, outcomes, and app tests. |
| `src-tauri/src/sources/peer_resolution.rs` | Resolution plans, aggregate failure wording, manual parser, typed DB identity, metadata/cache coordination, and final pure `ResolvedSyncPeer`; its exact raw field plus `legacy_peer_ref_from_descriptor` are transitional through CP6 and deleted at CP7. Dialog not-found/subtype errors are staged. |
| `src-tauri/src/sources/identity.rs` | DB rows, policy enums, normalization, SQL loads, readiness; no raw peer conversion. |
| `src-tauri/src/sources/avatar.rs` | Data URL, 4 s budget, cache key/path/read/write/cleanup. |
| `src-tauri/src/sources/sync.rs` | Provider/lock/settings policy, bounded batch loop, message-by-message persistence, counters/finalization, and command wrapper. |
| `src-tauri/src/sources/topics.rs` | UI/read models, refresh coordination, subtype/SQL checks, upsert/read commands. |
| `src-tauri/src/takeout_import/mod.rs` | Commands, jobs/start records, cancellation, provenance, warnings, range/page selection loop, persistence, progress/events, terminal finalization. |
| `src-tauri/src/takeout_import/forum_topics.rs` | Completion policy, warning/provenance recording, and all three app tests. |
| `src-tauri/src/takeout_import/migrated_history.rs` | Capability SQL, migration policy/errors, and identity construction. |
| `src-tauri/src/ingest_provenance.rs` | Entirely app-owned; only imports change. |
| `src-tauri/src/sources/types.rs` | App-owned; at CP4 add only `SourceSyncTarget::is_member`, populated by the existing `sources.is_member` column through `sources::store::load_source`, so stored descriptors preserve membership without network work. |
| `src-tauri/src/sources/mod.rs` | App module shell and non-Telegram exports; redirects exact staged public values only where existing app modules require them. |

### Literal Machine-Checked Production-Symbol Disposition

This table is bounded to current production symbols that move, split, change
shape, or are deleted, plus every new boundary/replacement symbol. A brace
group is an exact comma-separated identifier list, not a wildcard; the
generator expands it into one JSON row per current identifier. Singleton
targets apply to every current identifier, equal-cardinality groups align
positionally, and one current identifier may name multiple `finalTargets`.
Any other cardinality is rejected. `=` means the final identifier is exactly
the current identifier. `retained` means the symbol
remains app-owned after the named rewrite. No `*`, `all helpers`, `as needed`,
or unnamed fragment is legal in the generated artifact.

| Current path | Current exact symbol(s) | Final path | Final exact symbol(s) | Semantic owner | First checkpoint | Removal checkpoint | Disposition |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| `telegram/dto.rs` | `{ITEM_KIND_TELEGRAM_MESSAGE,TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER,TelegramMessageIdentity,TelegramMessageIdentity::validate,TelegramItemContext,TelegramMessageDraft}` | `telegram_impl/dto.rs` | `=` | staged | 3 | 3 | move |
| `telegram/media.rs` | `{CONTENT_KIND_TEXT_ONLY,CONTENT_KIND_TEXT_WITH_MEDIA,CONTENT_KIND_MEDIA_ONLY,TelegramMediaPayload,DocumentSignals,trimmed_non_empty,derive_content_kind,collect_document_signals,derive_document_media_kind,contact_summary,extract_document_media_payload,extract_media_payload,extract_item_payload}` | `telegram_impl/media.rs` | `=` | staged | 3 | 3 | move |
| `<new>` | `derive_document_media_kind_from_parts` | `telegram_impl/media.rs` | `derive_document_media_kind_from_parts` | staged-internal | 7 | retained | new-restricted-bridge |
| `telegram/runtime.rs` | `{TelegramApiHash,TelegramApiHash::new,TelegramRuntimeStatus,TelegramClientInner,TelegramClientHandle,TelegramLoginAttemptToken,TelegramLoginAttempt,TelegramRuntimeAccount,detach_replaced,TelegramRuntime,TelegramRuntime::new,TelegramRuntime::initialize_account,TelegramRuntime::is_authenticated,TelegramRuntime::request_login_code,TelegramRuntime::sign_in,TelegramRuntime::authorized_client,TelegramRuntime::initialized_client,TelegramRuntime::clear_account,TelegramRuntime::handle_is_authorized,initialize_grammers_client}` | `telegram_impl/runtime.rs` | `=` | staged | 3 | 3 | move |
| `<new>` | `TelegramRuntime::client` | `telegram_impl/runtime.rs` | `TelegramRuntime::client` | staged | 3 | retained | new |
| `telegram/runtime.rs` | `{TelegramClientHandle::raw_client,TelegramClientHandle::raw_session}` | `telegram_impl/runtime.rs` | `=` | transitional | 3 | 7 | move-then-delete |
| `telegram/session.rs` | `{SESSION_KEY_BYTES,ENVELOPE_VERSION,ENVELOPE_ALGORITHM,SavedSession,EncryptedSessionEnvelope,SessionEncryptionKey,SessionEncryptionKey::try_from_encoded,SessionEncryptionKey::generate,encode_base64,decode_base64,associated_data,memory_session_to_saved,saved_to_telegram_session,encrypt_saved_session,decrypt_saved_session,TelegramSession,TelegramSession::empty,session_json_requires_existing_key,decode_session_json,encode_session_json}` | `telegram_impl/session.rs` | `=` | staged | 3 | 3 | move |
| `telegram/session.rs` | `TelegramSession::raw_memory_session` | `telegram_impl/session.rs` | `=` | transitional | 3 | 7 | move-then-delete |
| `<new>` | `TelegramSession::clone_memory_session` | `telegram_impl/session.rs` | `TelegramSession::clone_memory_session` | staged-internal | 3 | retained | new-restricted-bridge |
| `telegram.rs` | `{mod::dto,mod::media,mod::runtime,mod::session}` | `telegram_impl/lib.rs` | `{mod::dto,mod::media,mod::runtime,mod::session}` | staged | 3 | 3 | replace-module-root |
| `media.rs` | `TelegramMediaPayload` | `telegram_impl/lib.rs` | `TelegramMediaPayload` | staged | 3 | 3 | replace-compat-reexport |
| `media.rs` | `{derive_content_kind,derive_document_media_kind,extract_item_payload,DocumentSignals}` | `telegram_impl/lib.rs` | `=` | transitional | 3 | 7 | redirect-compat-reexport |
| `media.rs` | `{derive_content_kind,derive_document_media_kind,extract_item_payload,DocumentSignals}` | `media.rs` | `=` | transitional | 3 | 7 | retain-compat-reexport |
| `telegram.rs` | `{get_client,get_authorized_client}` | `telegram.rs` | `=` | transitional | existing | 7 | delete |
| `sources/peer_resolution.rs` | `ResolvedTelegramSource` | `telegram_impl/dto.rs` | `PeerDescriptor` | staged | 4 | 4 | replace |
| `sources/peer_resolution.rs` | `resolve_telegram_source_by_username` | `telegram_impl/live/peer.rs` | `resolve_username` | staged | 4 | 4 | split-stage |
| `sources/peer_resolution.rs` | `resolve_telegram_source_by_username` | `sources/peer_resolution.rs` | `resolve_telegram_source` | app | 4 | retained | split-app-dispatch |
| `sources/peer_resolution.rs` | `resolve_telegram_source_from_dialogs` | `telegram_impl/live/peer.rs` | `resolve_dialog_peer` | staged | 4 | 4 | split-stage |
| `sources/peer_resolution.rs` | `resolve_telegram_source_from_dialogs` | `sources/peer_resolution.rs` | `resolve_telegram_source` | app | 4 | retained | split-app-dispatch |
| `sources/peer_resolution.rs` | `{dialog_lookup_not_found_message,dialog_lookup_not_found_error,telegram_source_subtype_matches,validate_expected_telegram_source_subtype,resolved_telegram_source_from_peer,telegram_group_kind,telegram_group_is_member,peer_access_hash}` | `telegram_impl/live/peer.rs` | `{dialog_lookup_not_found_message,dialog_lookup_not_found_error,telegram_source_subtype_matches,validate_expected_telegram_source_subtype,peer_descriptor_from_peer,telegram_group_kind,telegram_group_is_member,peer_access_hash}` | staged | 4 | 4 | move-or-rename |
| `sources/peer_resolution.rs` | `telegram_source_info_from_peer` | `telegram_impl/live/peer.rs` | `peer_descriptor_from_peer` | staged | 4 | 4 | split-stage |
| `sources/peer_resolution.rs` | `telegram_source_info_from_peer` | `sources/store.rs` | `peer_descriptor_to_source_info` | app | 4 | retained | split-app-projection |
| `sources/peer_resolution.rs` | `{source_peer_ref_from_identity,peer_ref_for_source_subtype,peer_ref_for_typed_identity}` | `telegram_impl/live/peer.rs` | `peer_ref_from_descriptor` | staged | 4 | 4 | replace |
| `sources/identity.rs` | `TelegramSourceIdentity::peer_ref` | `telegram_impl/live/peer.rs` | `peer_ref_from_descriptor` | staged | 4 | 4 | replace |
| `sources/peer_resolution.rs` | `{source_peer_ref_from_identity,peer_ref_for_source_subtype,peer_ref_for_typed_identity}` | `sources/peer_resolution.rs` | `legacy_peer_ref_from_descriptor` | transitional | 4 | 4 | replace-with-transitional-copy |
| `sources/identity.rs` | `TelegramSourceIdentity::peer_ref` | `sources/peer_resolution.rs` | `legacy_peer_ref_from_descriptor` | transitional | 4 | 4 | replace-with-transitional-copy |
| `<new>` | `legacy_peer_ref_from_descriptor` | `sources/peer_resolution.rs` | `legacy_peer_ref_from_descriptor` | transitional | 4 | 7 | new-then-delete |
| `sources/peer_resolution.rs` | `{typed_peer_resolution_plan,resolve_source_peer_from_typed_identity,resolve_and_refresh_peer,refresh_source_avatar_cache}` | `sources/peer_resolution.rs` | `=` | app | 4 | retained | rewrite-owned-seam |
| `<new>` | `peer_descriptor_from_stored_identity` | `sources/peer_resolution.rs` | `peer_descriptor_from_stored_identity` | app | 4 | retained | new |
| `sources/peer_resolution.rs` | `ResolvedSyncPeer::peer` | `sources/peer_resolution.rs` | `ResolvedSyncPeer::peer` | transitional | existing | 7 | retain-then-delete |
| `<new>` | `ResolvedSyncPeer::descriptor` | `sources/peer_resolution.rs` | `ResolvedSyncPeer::descriptor` | app | 4 | retained | new |
| `sources/types.rs` | `SourceSyncTarget` | `sources/types.rs` | `SourceSyncTarget` | app | 4 | retained | add-is-member-field |
| `<new>` | `SourceSyncTarget::is_member` | `sources/types.rs` | `SourceSyncTarget::is_member` | app | 4 | retained | new |
| `sources/store.rs` | `load_source` | `sources/store.rs` | `load_source` | app | 4 | retained | rewrite-owned-query |
| `sources/store.rs` | `add_telegram_source` | `sources/store.rs` | `add_telegram_source` | app | 4 | retained | rewrite-owned-seam |
| `sources/avatar.rs` | `{TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS,peer_photo_bytes_with_timeout,peer_photo_bytes}` | `telegram_impl/live/avatar.rs` | `=` | staged | 4 | 4 | move |
| `sources/avatar.rs` | `peer_photo_data_url_with_timeout` | `sources/store.rs` | `peer_descriptor_to_source_info` | app | 4 | 4 | replace |
| `sources/store.rs` | `list_telegram_sources::dialog_and_avatar_segment` | `telegram_impl/live/mod.rs` | `dialog_listing` | staged-internal | 4 | 4 | split-stage-facade |
| `sources/store.rs` | `list_telegram_sources::dialog_and_avatar_segment` | `telegram_impl/live/peer.rs` | `DialogListing::next` | staged | 4 | 4 | split-stage-iterator |
| `<new>` | `DialogListing` | `telegram_impl/live/peer.rs` | `DialogListing` | staged | 4 | retained | new |
| `<new>` | `DialogListing::next` | `telegram_impl/live/peer.rs` | `DialogListing::next` | staged | 4 | retained | new |
| `<new>` | `DialogListing::new` | `telegram_impl/live/peer.rs` | `DialogListing::new` | staged-internal | 4 | retained | new-restricted-bridge |
| `<new>` | `peer_avatar_bytes` | `telegram_impl/live/peer.rs` | `peer_avatar_bytes` | staged-internal | 4 | retained | new-restricted-leaf |
| `<new>` | `{dialog_listing,resolve_dialog_peer,resolve_username,peer_avatar_bytes}` | `telegram_impl/live/mod.rs` | `=` | staged-internal | 4 | retained | new-restricted-facade |
| `<new>` | `{TelegramClientHandle::dialog_listing,TelegramClientHandle::resolve_dialog_peer,TelegramClientHandle::resolve_username,TelegramClientHandle::peer_avatar_bytes}` | `telegram_impl/runtime.rs` | `=` | staged | 4 | retained | new |
| `sources/store.rs` | `list_telegram_sources` | `sources/store.rs` | `list_telegram_sources` | app | 4 | retained | split-app-command |
| `sources/items.rs` | `{message_author,extract_telegram_context,reply_peer_context,build_raw_payload}` | `telegram_impl/live/messages.rs` | `{raw_message_author,extract_telegram_context,reply_peer_context,build_raw_payload}` | staged | 5 | 5 | move-or-rename |
| `sources/sync.rs` | `fallback_message_identity` | `telegram_impl/live/messages.rs` | `fallback_message_identity` | staged | 5 | 5 | move |
| `sources/sync.rs` | `persist_items::raw_history_segment` | `telegram_impl/live/messages.rs` | `fetch_message_batch` | staged-internal | 5 | 5 | split-stage |
| `sources/sync.rs` | `{persist_items,sync_telegram_source}` | `sources/sync.rs` | `=` | app | 5 | retained | rewrite-owned-seam |
| `<new>` | `run_telegram_batch_loop` | `sources/sync.rs` | `run_telegram_batch_loop` | app | 5 | retained | new-private-test-seam |
| `<new>` | `TelegramSession::cache_peer_infos` | `telegram_impl/session.rs` | `TelegramSession::cache_peer_infos` | staged-internal | 5 | retained | new-restricted-bridge |
| `<new>` | `{LiveMessageBatch,LiveMessage}` | `telegram_impl/live/messages.rs` | `=` | staged | 5 | retained | new |
| `<new>` | `{LiveMessageBatch::take_messages,LiveMessageBatch::is_terminal,LiveMessageBatch::next_offset_id,LiveMessageBatch::next_offset_date,LiveMessage::message_id,LiveMessage::published_at,LiveMessage::into_draft}` | `telegram_impl/live/messages.rs` | `=` | staged | 5 | retained | new |
| `<new>` | `{fetch_message_batch,fetch_forum_topics}` | `telegram_impl/live/mod.rs` | `=` | staged-internal | 5 | retained | new-restricted-facade |
| `<new>` | `{TelegramClientHandle::fetch_message_batch,TelegramClientHandle::fetch_forum_topics}` | `telegram_impl/runtime.rs` | `=` | staged | 5 | retained | new |
| `sources/topics.rs` | `ForumTopicSnapshot` | `telegram_impl/dto.rs` | `ForumTopicSnapshot` | staged | 5 | 5 | move |
| `sources/topics.rs` | `{fetch_all_forum_topics,forum_topic_page_cursor,forum_topic_message_date}` | `telegram_impl/live/topics.rs` | `{fetch_forum_topics,forum_topic_page_cursor,forum_topic_message_date}` | staged | 5 | 5 | move-or-rename |
| `sources/topics.rs` | `is_non_forum_topic_refresh_error` | `telegram_impl/error.rs` | `is_non_forum_topic_refresh_error` | staged | 5 | 5 | move |
| `sources/topics.rs` | `refresh_forum_topics` | `sources/topics.rs` | `refresh_forum_topics` | app | 5 | retained | rewrite-owned-seam |
| `takeout_import/mod.rs` | `export_dc_invoke_with_provenance::raw_segment` | `telegram_impl/takeout/export_dc.rs` | `export_dc_invoke` | staged-internal | 7 | 7 | split-stage-raw-invoke |
| `takeout_import/mod.rs` | `export_dc_invoke_with_provenance::durable_segment` | `takeout_import/mod.rs` | `run_takeout_transport_step_with_provenance` | app | 7 | retained | split-app-coordinator |
| `takeout_import/mod.rs` | `{record_export_dc_attempt_if_needed,record_export_dc_fallback_if_needed,record_only_my_messages_fallback_if_needed}` | `takeout_import/mod.rs` | `=` | app | 7 | retained | rewrite-owned-values |
| `takeout_import/mod.rs` | `record_channel_private_fallback_if_supported` | `takeout_import/mod.rs` | `record_only_my_messages_fallback_if_needed` | app | 7 | 7 | replace |
| `takeout_import/mod.rs` | `peer_ref_identity` | `telegram_impl/takeout/raw_parse.rs` | `peer_ref_identity` | staged | 7 | 7 | move |
| `takeout_import/mod.rs` | `{validate_takeout_peer,detect_supergroup_migration,revalidate_migrated_from_chat_id}` | `telegram_impl/takeout/operations.rs` | `{TakeoutTransport::validate_peer,TakeoutTransport::detect_supergroup_migration,TakeoutTransport::revalidate_migrated_peer}` | staged | 7 | 7 | replace |
| `takeout_import/mod.rs` | `takeout_get_history` | `telegram_impl/takeout/operations.rs` | `TakeoutTransport::history_count` | staged | 7 | 7 | split-stage-count |
| `takeout_import/mod.rs` | `takeout_get_history` | `telegram_impl/takeout/operations.rs` | `TakeoutTransport::history_page` | staged | 7 | 7 | split-stage-page |
| `takeout_import/mod.rs` | `takeout_search_my_messages` | `telegram_impl/takeout/operations.rs` | `TakeoutTransport::search_my_history_count` | staged | 7 | 7 | split-stage-search-count |
| `takeout_import/mod.rs` | `takeout_search_my_messages` | `telegram_impl/takeout/operations.rs` | `TakeoutTransport::search_my_history_page` | staged | 7 | 7 | split-stage-search-page |
| `takeout_import/mod.rs` | `{supports_only_my_messages_fallback,is_channel_private_error,messages_response_count}` | `{telegram_impl/takeout/operations.rs,telegram_impl/error.rs,telegram_impl/takeout/raw_parse.rs}` | `{supports_only_my_messages_fallback,is_channel_private_error,messages_response_count}` | staged | 7 | 7 | move |
| `takeout_import/mod.rs` | `{takeout_history_count_probe,takeout_history_page_response}` | `takeout_import/mod.rs` | `=` | app | 7 | retained | rewrite-owned-seam |
| `takeout_import/mod.rs` | `TakeoutHistoryProbe` | `telegram_impl/takeout/types.rs` | `TakeoutCount` | staged | 7 | 7 | replace |
| `takeout_import/mod.rs` | `CountedMessageRange` | `takeout_import/mod.rs` | `CountedMessageRange` | app | 7 | retained | replace-fields-with-owned-values |
| `takeout_import/mod.rs` | `{run_export_dc_spike_for_handle,run_takeout_source_import,run_takeout_migrated_history_import,run_started_takeout_source_import,run_started_takeout_source_import_inner,import_takeout_history_ranges,import_takeout_history_pages}` | `takeout_import/mod.rs` | `=` | app | 7 | retained | rewrite-owned-seam |
| `takeout_import/export_dc.rs` | `{EXPORT_DC_SHIFT,TAKEOUT_FILE_MAX_SIZE,ExportDcAlias,prepare_export_dc_alias,export_dc_id_for_home_dc,takeout_init_request_for_source_subtype,export_dc_invoke,export_dc_invoke_with,finish_takeout_session}` | `telegram_impl/takeout/export_dc.rs` | `=` | staged | 7 | 7 | move |
| `takeout_import/export_dc.rs` | `should_fallback_export_dc_error` | `telegram_impl/error.rs` | `should_fallback_export_dc_error` | staged | 7 | 7 | move |
| `takeout_import/export_dc.rs` | `ExportDcAttemptState` | `telegram_impl/takeout/transport.rs` | `ExportDcAttemptTracker` | staged | 7 | 7 | split-stage-tracker |
| `takeout_import/export_dc.rs` | `ExportDcAttemptState` | `telegram_impl/takeout/types.rs` | `TakeoutAttempt` | staged | 7 | 7 | split-stage-attempt-value |
| `takeout_import/pagination.rs` | `{TAKEOUT_HISTORY_PAGE_LIMIT,TakeoutPaginationProfile,TakeoutPageRequest,TakeoutPaginationCursor,TakeoutPaginationCursor::new,TakeoutCursorAdvance,TakeoutPaginationFallbackReason,select_history_splits,fallback_message_range,takeout_page_request,next_takeout_cursor,should_restart_with_descending_fallback,takeout_pagination_fallback_warning}` | `telegram_impl/takeout/pagination.rs` | `=` | staged | 7 | 7 | move |
| `takeout_import/pagination.rs` | `{TakeoutPageRequest::offset_id,TakeoutPageRequest::add_offset,TakeoutPageRequest::limit,TakeoutCursorAdvance::cursor,TakeoutCursorAdvance::advanced,TakeoutCursorAdvance::reached_range_start}` | `telegram_impl/takeout/pagination.rs` | `=` | staged-internal | 7 | 7 | move-restricted-fields |
| `takeout_import/pagination.rs` | `{ParsedTakeoutPage,parse_takeout_page}` | `{telegram_impl/takeout/types.rs,telegram_impl/takeout/pagination.rs}` | `{TakeoutPage,parse_takeout_page}` | staged | 7 | 7 | absorb-and-rewrite |
| `takeout_import/pagination.rs` | `{message_range_min_id,message_range_max_id}` | `telegram_impl/takeout/types.rs` | `{MessageRange::min_id,MessageRange::max_id}` | staged | 7 | 7 | replace |
| `takeout_import/raw_parse.rs` | `parse_raw_message` | `telegram_impl/takeout/raw_parse.rs` | `parse_raw_message` | staged-internal | 7 | 7 | move |
| `takeout_import/raw_parse.rs` | `{extract_raw_media_payload,extract_photo_media_payload,extract_document_media_payload,extract_raw_telegram_context,raw_message_identity,reaction_count,raw_message_author,peer_context,peer_id_string,raw_summary_media,apply_larger_photo_size,contact_summary,trimmed_non_empty}` | `telegram_impl/takeout/raw_parse.rs` | `=` | staged | 7 | 7 | move |
| `takeout_import/forum_topics.rs` | `refresh_forum_topics_after_completed_takeout::remote_segment` | `telegram_impl/takeout/forum_topics.rs` | `takeout_forum_topics` | staged-internal | 7 | 7 | split-stage |
| `takeout_import/forum_topics.rs` | `refresh_forum_topics_after_completed_takeout` | `takeout_import/forum_topics.rs` | `refresh_forum_topics_after_completed_takeout` | app | 7 | retained | split-app-policy |
| `<new>` | `{TakeoutAttempt,TakeoutFallbackKind,TakeoutFallback,TakeoutPeer,MessageRange,TakeoutCount,TakeoutPage,TakeoutMessage}` | `telegram_impl/takeout/types.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutAttempt::home_dc_id,TakeoutAttempt::export_dc_id,TakeoutFallback::kind,TakeoutFallback::warning,TakeoutFallback::provenance_message,TakeoutPeer::from_descriptor,TakeoutPeer::peer_kind,TakeoutPeer::peer_id,MessageRange::min_id,MessageRange::max_id,TakeoutCount::count,TakeoutCount::only_my_messages,TakeoutPage::take_messages,TakeoutPage::has_next,TakeoutPage::only_my_messages,TakeoutPage::take_pagination_fallback_warning,TakeoutMessage::message_id,TakeoutMessage::into_draft}` | `telegram_impl/takeout/types.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutAttempt::new,TakeoutFallback::new,TakeoutPeer::new,TakeoutPeer::source_subtype,TakeoutPeer::access_hash,MessageRange::new,TakeoutCount::new,TakeoutPage::from_parts,TakeoutPage::pagination_state,TakeoutMessage::from_raw}` | `telegram_impl/takeout/types.rs` | `=` | staged-internal | 7 | retained | new-restricted-bridge |
| `<new>` | `TakeoutTransport` | `telegram_impl/takeout/transport.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutTransport::export_dc_attempt,TakeoutTransport::drain_fallbacks}` | `telegram_impl/takeout/transport.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutTransport::new,TakeoutTransport::queue_fallback,TakeoutTransport::client,TakeoutTransport::session,TakeoutTransport::home_dc_id,TakeoutTransport::export_dc_id}` | `telegram_impl/takeout/transport.rs` | `=` | staged-internal | 7 | retained | new-restricted-bridge |
| `<new>` | `takeout_self_check` | `telegram_impl/takeout/operations.rs` | `takeout_self_check` | staged-internal | 7 | retained | new-restricted-leaf |
| `<new>` | `{takeout_self_check,prepare_takeout,takeout_forum_topics}` | `telegram_impl/takeout/mod.rs` | `=` | staged-internal | 7 | retained | new-restricted-facade |
| `<new>` | `{TelegramClientHandle::takeout_self_check,TelegramClientHandle::prepare_takeout}` | `telegram_impl/runtime.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `TelegramClientHandle::takeout_forum_topics` | `telegram_impl/runtime.rs` | `=` | staged | 7 | retained | new |
| `<new>` | `{TakeoutTransport::init,TakeoutTransport::message_ranges,TakeoutTransport::validate_peer,TakeoutTransport::detect_supergroup_migration,TakeoutTransport::revalidate_migrated_peer,TakeoutTransport::history_count,TakeoutTransport::search_my_history_count,TakeoutTransport::history_page,TakeoutTransport::search_my_history_page,TakeoutTransport::finish}` | `telegram_impl/takeout/operations.rs` | `=` | staged | 7 | retained | new |

The `raw_memory_session` and `clone_memory_session` rows are an intentional
coexist-then-replace pair: CP3 through CP6 require both exact methods, while
CP7 and later require the restricted owned-`Arc` bridge exactly once and zero
occurrences of the borrowed bridge name in production or tests.

The generator also freezes three transition inventories that are not inferred
from path existence:

```text
CP3 raw handle callsites:
  sources::store::{list_telegram_sources,add_telegram_source}
  sources::sync::sync_telegram_source
  takeout_import::{run_export_dc_spike_for_handle,run_takeout_migrated_history_import,run_takeout_source_import}
CP4 raw handle callsites:
  sources::sync::sync_telegram_source
  takeout_import::{run_export_dc_spike_for_handle,run_takeout_migrated_history_import,run_takeout_source_import}
CP4 ResolvedSyncPeer::peer consumers:
  sources::sync::sync_telegram_source
  takeout_import::{run_takeout_migrated_history_import,run_takeout_source_import}
CP5 raw handle callsites:
  takeout_import::{run_export_dc_spike_for_handle,run_takeout_migrated_history_import,run_takeout_source_import}
CP5 ResolvedSyncPeer::peer consumers:
  takeout_import::{run_takeout_migrated_history_import,run_takeout_source_import}
CP7 raw bridge symbols/callsites: empty
```

Synthetic `::segment` keys are source fragments, not Rust identifiers. Their
JSON rows carry these exact pre-move anchors, each required in its enclosing
function at CP1 and forbidden there after the removal checkpoint:

```text
list_telegram_sources::dialog_and_avatar_segment:
  client.iter_dialogs()
  peer_photo_data_url_with_timeout(&client, dialog.peer())
persist_items::raw_history_segment:
  client.iter_messages(peer)
  messages.next()
export_dc_invoke_with_provenance::raw_segment:
  export_dc_invoke(client, alias, request, warnings, fallback_used).await
export_dc_invoke_with_provenance::durable_segment:
  record_export_dc_attempt_if_needed
  record_export_dc_fallback_if_needed
refresh_forum_topics_after_completed_takeout::remote_segment:
  refresh_forum_topics(pool, client, peer, source).await
```

The app-retained production symbols not named above are out of the move scope
and may receive import-only edits. A body change to any such symbol must be
listed as `rewrite-owned-seam` before implementation.

## Generated Grammers Feature Baseline

Checkpoint 1 creates
`src/lib/telegram-grammers-feature-baseline.json` and
`scripts/telegram-grammers-feature-baseline.mjs`. The script accepts exactly
`--write` or `--check`, invokes locked Cargo metadata, rejects a
malformed/missing package-graph result, and writes stable UTF-8 JSON with this
schema:

```json
{
  "schemaVersion": 1,
  "revision": "1f901ce6e973fdcf0e74267f3d8efad5c729daaa",
  "packages": [
    {
      "name": "grammers-client",
      "required": [],
      "forbidden": ["default"],
      "universe": ["default"]
    }
  ]
}
```

The real arrays are sorted and contain every feature key exposed by each
pinned package. The generator uses the resolved `extractum` node only to find
the four exact direct-dependency package IDs. It reads required features from
each Grammers dependency's own `resolve.nodes[].features` entry and reads that
package's feature universe from the matching `packages[].features` keys.
Forbidden is exactly `universe - required`. The frozen observations are:

| Package | Required | Forbidden |
| --- | --- | --- |
| `grammers-client` | none | `default`, `fs`, `html`, `html5ever`, `markdown`, `parse_invite_link`, `proxy`, `pulldown-cmark`, `url` |
| `grammers-session` | `serde` | `default`, `sqlite-storage` |
| `grammers-mtsender` | none | `hickory-resolver`, `proxy`, `tokio-socks`, `url` |
| `grammers-tl-types` | `default`, `deserializable-functions`, `impl-debug`, `impl-from-enum`, `impl-from-type`, `tl-api`, `tl-mtproto` | `impl-serde` |

`--check` fails on revision, package, order, required, forbidden, universe, or
format drift. The Telegram boundary contract executes `--check`, so
`npm.cmd run verify` remains the standing gate.

## Task 0: Synchronize the Narrow API Authority

**Files:**

- Modify:
  `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Verify:
  `docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md`

- [ ] Confirm the owner has explicitly approved this plan and its bounded API
  clarification. Record the approval reference in the execution notes; do not
  infer approval from the plan's existence.
- [ ] Prove the worktree is clean, the plan is tracked, the start SHA descends
  from `dd14161386a946dfd070e25dae06bbb99ac62cfb`, and no
  `src-tauri/src/telegram_impl` or `extractum-telegram` package exists.
- [ ] Amend the design's public-item allowlist with exactly
  `DialogListing`, `LiveMessageBatch`, `LiveMessage`, `TakeoutAttempt`,
  `TakeoutFallbackKind`, `TakeoutFallback`, `TakeoutPeer`,
  `TakeoutTransport`, `MessageRange`, `TakeoutCount`, `TakeoutPage`, and
  `TakeoutMessage`. Keep every field private.
- [ ] Amend the design's boundary-handle list, Takeout ownership clauses,
  staging map, and rationale to name the concrete stateful `DialogListing` and
  `TakeoutTransport`. State explicitly that this is the sole architecture
  amendment and neither type is a generic invocation, repository, event, or
  callback abstraction.
- [ ] Copy the exact public signatures and live/Takeout transition protocol
  from this plan into the design. Explicitly reject `PeerLocator`, a public
  raw/generic cursor or stream other than the exact opaque `DialogListing`,
  and a generic invocation/provenance callback.
- [ ] Copy the exact restricted-visibility allowlist, the symbol-disposition
  authority, the CP3→CP7 transitional raw bridge, budgeted dialog/avatar
  interleaving, peer-cache side effect, and one-outer-select Takeout protocol
  into the design. Remove every superseded broad/catch-all ownership clause.
- [ ] Authorize the content-addressed 8A map import and all generated authority
  artifact paths. Keep the design/roadmap status at the starting state; 8B has
  not started.
- [ ] Copy the checkpoint candidate-status rule into the authority: dirty
  implementation remains on the preceding retained pair; the exact
  uncommitted next-checkpoint pair exists only to select the final lifecycle
  gates, becomes retained only in the GREEN checkpoint commit, and is restored
  before any fix after a failed gate.
- [ ] Run:

```powershell
Invoke-CheckedNative 'Task 0 Telegram contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Task 0 shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'Task 0 diff check' { git diff --check }
```

Expected: existing 8A production remains GREEN. If the docs amendment makes
the retained contract RED, stop and revise the conditional authority parser;
do not commit a false retained state.

- [ ] Inspect the docs-only diff, stage only the two authority docs, and commit:

```powershell
Invoke-CheckedNative 'stage Task 0' {
    git add docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md docs/superpowers/specs/2026-07-17-crate-roadmap.md
}
Invoke-CheckedNative 'commit Task 0' {
    git commit -m "docs: freeze Phase 8B Telegram boundary API"
}
```

## Task 1: Install Checkpoint Authority, Identity Accounting, and Feature Baseline

**Files:**

- Modify: `src/lib/telegram-contract-paths.ts`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Create: `scripts/telegram-grammers-feature-baseline.mjs`
- Create: `src/lib/telegram-grammers-feature-baseline.json`
- Create: `scripts/telegram-8b-test-identities.mjs`
- Create: `src/lib/telegram-8b-test-identities.json`
- Create: `scripts/telegram-8b-symbol-map.mjs`
- Create: `src/lib/telegram-8b-symbol-map.json`
- Modify: `docs/value-registry.md`
- Modify:
  `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`

- [ ] Add compiling structural RED cases named:

```text
recognizes every retained Phase 8B lifecycle and rejects unknown values
imports the immutable Phase 8A identity map by exact section hash
parses the exact Phase 8B new-test table without duplicates
materializes the exact Phase 8B test partitions from content-addressed authority
materializes every production-symbol disposition and transitional bridge
materializes the exact restricted-visibility allowlist without duplicates
checks the generated Grammers feature baseline
keeps Checkpoint 1 on the application-owned pre-staging layout
```

- [ ] Run the focused Telegram and shell-cap contracts. Confirm the new cases
  execute and fail for missing 8B lifecycle/hash/generator support, not for a
  TypeScript compile error.
- [ ] Implement exact `8b-checkpoint-1` through `8b-checkpoint-8` and terminal
  `8b-preparation` lifecycles. Reject unknown statuses and path inference from
  file existence.
- [ ] Register both the eight `8b-checkpoint-N` lifecycle tokens and their
  eight exact `8B preparation Checkpoint N retained` status inputs in
  `docs/value-registry.md`, with owner `telegram-contract-paths.ts`, no
  persistence/API/UI impact, transitional lifecycle, and test/agent-workflow
  usage. Do not present them as product or database statuses.
- [ ] Normalize the retained 8A plan to LF, slice from
  `## Literal Immutable 140-Test Identity Map` up to but excluding the exact
  `LF + ### Exact New-Test Identity Map` marker, and require both 37,436 bytes and SHA-256
  `ceab6cef728d396bf2136207f2130974dee2cc0be3c5184eabd8c8de5e58b3ca`
  before parsing any row.
- [ ] Slice the retained 8A exact-addition authority from
  `### Exact New-Test Identity Map` up to but excluding the first exact
  `LF + --- + LF` section separator; require 3,717 LF-normalized bytes and SHA-256
  `a8dce5a0a00ac8cdcf83ef7eab2304f482e7c3967ec26ab8c8270d6fde42f539`.
  Fail closed on malformed headings, empty tables, duplicate IDs, duplicate
  paths, or unknown owners.
- [ ] Parse this plan's 15-row new-test table and require exact count,
  checkpoint allocation, uniqueness, exactly 14 staged-prefix IDs, and the one
  exact app ID. Assert current 719, eventual 736, baseline-derived 143,
  pre-new tracked 160, final tracked 175, app 104, and staged 71 as separate
  metrics.
- [ ] Implement `scripts/telegram-8b-test-identities.mjs` with exact `--write`
  and `--check` modes. It verifies both imported 8A section hashes and this
  plan's new-test table, then emits sorted `baselineDerived` (143),
  `preNewApp` (103), `preNewStaged` (57), `phase8BNewApp` (1), and
  `phase8BNewStaged` (14) arrays. Reject duplicate/overlapping IDs, wrong
  owners/prefixes/checkpoints, wrong totals, malformed tables, order drift, or
  unsupported args. `baselineDerived` is the immutable 140 final identities,
  the one retained 8A companion, and the two exact deferred companions;
  `preNew*` is the complete retained 158 plus those two deferred companions,
  partitioned by exact final `telegram_impl::` prefix. Prove a second
  `--write` is byte-identical.
- [ ] Implement `scripts/telegram-8b-symbol-map.mjs` with exact `--write` and
  `--check` modes from both the literal symbol-disposition table and the exact
  restricted-bridge fence. Its JSON rows contain `currentPath`,
  `currentSymbol`, exact `finalTargets` path/symbol pairs, `semanticOwner`,
  `firstCheckpoint`, `removalCheckpoint`, and `disposition`; split symbols use
  separate named fragment rows with their literal `currentAnchors` or multiple
  exact final targets. The same artifact contains an exact 67-entry, sorted,
  duplicate-free `restrictedFinalSymbols` array. Parse singleton allowlist lines and
  single-level brace groups only; reject nested braces, so the pagination
  subsection must remain fully qualified and flat. Every restricted symbol
  must occur in the disposition table's final targets and must not occur in
  the root public allowlist. Reject an unlisted current production symbol in
  scope, duplicate current path/symbol/disposition tuples, duplicate
  restricted symbols, wildcard/catch-all symbols, malformed groups, missing
  final symbols after their checkpoint, or a transitional bridge surviving
  its removal checkpoint. Prove a second `--write` is byte-identical and
  `--check` rejects either table/fence drift.
- [ ] Implement the feature generator and create the artifact with `--write`;
  immediately prove a second `--write` is byte-idempotent and `--check` is
  GREEN.
- [ ] Add contract assertions for the exact public/restricted allowlists,
  exact 19-file terminal tree, checkpoint path table, direct-Grammers consumer
  inventory, forbidden raw/app types, exact test-partition artifact, and exact
  symbol-disposition artifact. The CP7/CP8 source contract must compare the
  complete normalized production `pub(super)` inventory to generated
  `restrictedFinalSymbols`; it must not hard-code a second pagination list.
  Mutation cases delete `TakeoutPaginationCursor::new`, delete either returned
  type entry (`TakeoutPageRequest` or `TakeoutCursorAdvance`), and widen one
  `TakeoutPageRequest` field; every case must fail the focused contract.
  Starting at Checkpoint 3, also require
  `initialize_grammers_client` to materialize
  `grammers_client::client::ClientConfiguration::default()`, fail closed on
  `!configuration.auto_cache_peers` with the frozen internal-error message,
  do so before `SenderPool::new`/`tokio::spawn`, and construct through
  `Client::with_configuration`; reject a missing or inverted guard, the wrong
  configuration path, post-spawn validation, or a return to implicit
  `Client::new`. Parse and enforce the exact two-parameter function signature;
  mutation cases that add a `ClientConfiguration`, boolean, factory, or
  generic configuration input must fail even if the body still calls
  `ClientConfiguration::default()`. Require the default construction and guard
  inside this function body, not in a caller/helper. Require both exact session
  accessors at CP3 through CP6, then at CP7/CP8 require exactly one restricted
  `clone_memory_session` definition and zero `raw_memory_session` identifiers
  across production and tests. At Checkpoint 1 these future conditions are
  authority-only; current source is still the retained 8A layout.
- [ ] Update statuses atomically to:

```text
roadmap: 8B preparation Checkpoint 1 retained
design: Approved; 8B preparation Checkpoint 1 retained
```

- [ ] Run the focused contracts:

```powershell
Invoke-CheckedNative 'Task 1 Telegram contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Task 1 shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'Task 1 media contract' {
    npm.cmd run test -- src/lib/media-metadata-core-contract.test.ts
}
Invoke-CheckedNative 'Task 1 feature authority' {
    node scripts/telegram-grammers-feature-baseline.mjs --check
}
Invoke-CheckedNative 'Task 1 identity authority' {
    node scripts/telegram-8b-test-identities.mjs --check
}
Invoke-CheckedNative 'Task 1 symbol authority' {
    node scripts/telegram-8b-symbol-map.mjs --check
}
```

- [ ] Run the package checkpoint, assert 719 unique library test IDs, then run:

```powershell
Assert-RustPackageTestTotal -Package extractum -ExpectedTotal 719
Invoke-RetainedCheckpointGates '8B-CP1'
```

- [ ] Inspect/stage only the Task 1 allowlist and commit:

```powershell
Invoke-CheckedNative 'commit Task 1' {
    git commit -m "test: install Phase 8B Telegram preparation authority"
}
```

## Forward-Only Amendment After Retained Checkpoint 1

**Effective scope:** this amendment governs only unexecuted work after the
retained Checkpoint 1 commit. It does not rewrite Task 1, regenerate its
schema-v1 artifact, relabel its RED/GREEN evidence, or change the retained
roadmap/design status pair.

Before Task 2, commit one focused contract simplification with no production
Rust, manifest, lockfile, artifact-schema, or status change:

- start from the known unstaged review-fix overlay already present in
  `src/lib/telegram-crate-boundary-contract.test.ts` after retained CP1. At
  authority commit `56b9d399`, that overlay is `+1531/-132` with an empty
  index. It is unretained candidate work in the target file, not a clean-HEAD
  precondition and not automatically unrelated user work;
- keep `src/lib/telegram-8b-symbol-map.json` at `schemaVersion: 1`;
  `transitionInventories` remains a readable LLM-review checklist and is not
  executable Rust source authority;
- remove from `src/lib/telegram-crate-boundary-contract.test.ts` every helper,
  assertion, and mutation whose result decides whether a Rust definition or
  use of
  the transitional four-item media compatibility facade,
  `raw_client`, `raw_session`, `raw_memory_session`, `get_client`,
  `get_authorized_client`, `ResolvedSyncPeer::peer`, or
  `legacy_peer_ref_from_descriptor` is authorized;
- do not tokenize or mask Rust for this purpose, isolate Rust functions,
  resolve Rust names/types/imports/bindings/scopes/shadowing, infer producer
  results or call graphs, trace forwarding/data flow, count same-named
  occurrences, or introduce generated use-site anchors/edges/fences;
- explicitly remove the dirty-only semantic overlay represented by
  `phase8BTypedReceiverNames`, `phase8BProducerBoundReceiverNames`,
  `phase8BFunctionUsesTransitionBridge`, and
  `phase8BIsCanonicalBridgeTypePath`, plus every helper/assertion/mutation that
  exists only to feed them. Also remove the retained CP1 fixture/assertion
  branches that authorize or reject the named escape-hatch definitions or
  uses, including the `phase8BSessionFixture(includeRawAccessor)` lifecycle
  branch;
- preserve unrelated document/artifact serialization, lifecycle vocabulary,
  manifest/dependency, public-surface, identity, symbol-disposition, and
  behavior contracts; the escape-hatch rows remain in the generated
  symbol-disposition/review inventory, but TypeScript does not reconcile their
  definitions or use-sites against Rust source. This amendment does not turn
  off the rest of the Phase 8 boundary. In particular, preserve the terminal
  direct-Grammers dependency-path inventory currently named
  `phase8BTerminalRawConsumerAuthority`: it is dependency-ownership metadata,
  not escape-hatch use-site authority. Rename it if needed to make that
  distinction explicit.

Before editing the target TypeScript file:

```powershell
Invoke-CheckedNative 'post-CP1 target index is empty' {
    git diff --cached --quiet -- src/lib/telegram-crate-boundary-contract.test.ts
}
git diff -- src/lib/telegram-crate-boundary-contract.test.ts
git diff HEAD -- src/lib/telegram-crate-boundary-contract.test.ts
```

Review and classify every existing hunk. Reconcile the dirty-only analyzer
overlay and the retained CP1 contract in one final HEAD-relative file: keep the
schema-v1 serialization/type metadata and every unrelated boundary contract;
remove only the prohibited Rust definition/use analysis. Do not use
`git restore`, `git checkout`, or interactive/partial staging. If any hunk
cannot be classified as an explicitly retained contract or the prohibited
analyzer overlay, stop for owner disposition before editing or staging.

For transitional escape-hatch use-sites, automatic evidence is limited to
Rust compilation plus the named Rust focused, package, and workspace behavior
tests. The focused TypeScript contract still runs to prove its remaining
non-use-site responsibilities, but a TypeScript source contract is not
evidence that an escape-hatch use is allowed and may not accept or reject such
a Rust use.

### Mandatory Reusable CP2–CP8 LLM Retention Gate

Every Task 2–8 checkpoint must perform this same gate after its final scoped
Rust diff is ready and before writing candidate status, running final retained
checkpoint gates, retaining, staging, or committing the checkpoint:

1. A fresh independent LLM reviewer that did not implement the checkpoint and
   has no prior implementation-conversation context receives a bounded review
   packet: the checkpoint and retained predecessor, the complete scoped Rust
   diff including production and test Rust, enough unchanged surrounding Rust
   source, this plan/design authority, and committed
   `src/lib/telegram-8b-symbol-map.json`. The reviewer reads the artifact's
   complete `transitionInventories` field and symbol-disposition rows together
   with all complete exception sets active at that checkpoint, including the
   forward-only CP3→CP4 peer-kind bridge; the artifact is an input checklist,
   not executable source authority. CP2 uses the explicit no-Rust-change rule;
   CP3 uses `cp3RawHandleCallsites`; CP4 uses both CP4 arrays; CP5 uses both CP5
   arrays; CP6 carries the CP5 arrays forward unless the reviewed diff removes
   a listed use; and CP7/CP8 use/carry forward the empty
   `cp7RawBridgeSymbolsAndCallsites` array.
2. The review accounts for every affected producer and full forwarding chain,
   including
   `run_takeout_source_import → run_started_takeout_source_import →
   run_started_takeout_source_import_inner`.
3. The reviewer records in the active task thread the checkpoint and retained
   predecessor; the exact exception definitions and physical uses that remain;
   each logical owner and full forwarding chain; every use that must disappear
   at this checkpoint; aliases, re-exports, wrappers, function items, and
   unrelated same-named fields; and every finding.
4. The record ends with exactly one verdict:
   `Escape-hatch review verdict: CLEAN` or
   `Escape-hatch review verdict: BLOCKED`. Only `CLEAN` permits candidate
   status, final gates, retention, staging, or commit. Any unlisted or
   ambiguous use is `BLOCKED`; it may be removed and followed by a clean
   re-review without an amendment. An approved plan/design amendment is
   required only if the implementation intends to retain that use.
5. Every review after `BLOCKED` discards the prior verdict and receives the
   complete current scoped Rust diff plus required unchanged context, not only
   a fix delta. This full re-review is mandatory even when the resolution was
   added context, reviewer clarification, or an approved plan/design amendment
   rather than a Rust edit. Any Rust or escape-hatch-authority change after
   `CLEAN` likewise invalidates that verdict and requires the same fresh full
   review.
6. After `CLEAN` and before candidate status, copy the complete final review
   record into
   `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`.
   CP2 creates this cumulative durable ledger; CP3–CP8 append one exact
   checkpoint section and commit it with that checkpoint. A `BLOCKED` attempt
   remains diagnostic task-thread evidence and is not copied as retained
   evidence. TypeScript does not parse, validate, or gate on this prose.
7. At CP2, the record must explicitly confirm that the complete scoped Rust
   diff contains no escape-hatch definition, physical-use, ownership, or
   forwarding-chain change. The CP8 section contains only the complete CP8
   review record and its one final CLEAN marker.

The cumulative verification document uses the exact section headings
`## CP2 Escape-Hatch LLM Review` through
`## CP8 Escape-Hatch LLM Review`. Every section records the retained
predecessor SHA, reviewer independence statement, reviewed Rust path list,
artifact/inventory inputs, remaining definitions and physical uses, logical
owners and forwarding chains, required removals, aliases/re-exports/wrappers/
function items/same-named fields, findings, and exactly one final
`Escape-hatch review verdict: CLEAN` line.

Task 8 adds an aggregate CP2–CP8 outcome table outside those seven exact review
sections. The table references each section and checkpoint SHA and records the
outcome as plain `CLEAN`; it does not repeat the exact verdict-marker line.

Literal source search may help the independent reviewer navigate the packet,
but no search pattern, occurrence count, or exit status authorizes or blocks
an escape-hatch use or serves as checkpoint-retention evidence. This includes
the terminal CP7/CP8 zero-use claims.

Run:

```powershell
Invoke-CheckedNative 'post-CP1 Telegram contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'post-CP1 shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'post-CP1 symbol authority' {
    node scripts/telegram-8b-symbol-map.mjs --check
}
Invoke-CheckedNative 'post-CP1 TypeScript diff check' {
    git diff --check -- src/lib/telegram-crate-boundary-contract.test.ts
}
```

The plan and design are already committed authority; do not stage them in this
one-time simplification commit. Once every target-file hunk is classified and
the complete working file has the intended HEAD-relative state, stage the
whole focused file non-interactively and inspect both sides:

```powershell
Invoke-CheckedNative 'stage post-CP1 contract simplification' {
    git add -- src/lib/telegram-crate-boundary-contract.test.ts
}
$PostCp1CachedPaths = @(git diff --cached --name-only)
if ($LASTEXITCODE -ne 0) {
    throw 'Unable to inspect post-CP1 cached path allowlist'
}
if (
    $PostCp1CachedPaths.Count -ne 1 `
    -or $PostCp1CachedPaths[0] `
        -ne 'src/lib/telegram-crate-boundary-contract.test.ts'
) {
    throw "Unexpected post-CP1 cached paths: $($PostCp1CachedPaths -join ', ')"
}
git diff --cached -- src/lib/telegram-crate-boundary-contract.test.ts
git diff -- src/lib/telegram-crate-boundary-contract.test.ts
Invoke-CheckedNative 'post-CP1 target has no unstaged diff' {
    git diff --quiet -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'post-CP1 cached diff check' {
    git diff --cached --check
}
Invoke-CheckedNative 'commit post-CP1 contract simplification' {
    git commit -m "test: simplify Telegram escape-hatch review authority"
}
```

The cached path allowlist must contain exactly the target TypeScript file, and
its unstaged target diff must be empty before commit. This is a non-checkpoint
authority-maintenance commit: it keeps the exact CP1 status pair, is recorded
with authority commits
`1c0961516c338073ba9578edb18a61c7b1285897` and
`56b9d3995f10f0070f4e2d0f94fb048181218ae3` in the final verification ledger,
along with the reviewed `docs: harden Telegram LLM review gate` correction
commit, and must be an ancestor of CP2 and the final Phase 8B disposition.

## Task 2: Normalize Existing Workspace Dependencies Without Graph Drift

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Verify only unless Cargo proves real resolution drift:
  `src-tauri/Cargo.lock`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `src/lib/gemini-browser-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify:
  `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Create:
  `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`

Promote exactly these entries to `[workspace.dependencies]`:

```toml
base64 = "0.22"
chacha20poly1305 = { version = "0.10", features = ["std"] }
grammers-client = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", default-features = false }
grammers-session = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", default-features = false, features = ["serde"] }
grammers-mtsender = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa" }
grammers-tl-types = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", features = ["deserializable-functions"] }
rand_core = { version = "0.6", features = ["getrandom"] }
```

- [ ] Record start hashes. The retained authority is:

```text
src-tauri/Cargo.toml  81A773E6FFB5E4BC1AF7C25D2B3F723424E060A515289B2CB28B7C45360A31FF
src-tauri/Cargo.lock  720E38EA632D7B932B2A23D1481528845EC9304376035B1C851C546EA402E43C
```

If execution starts from a later approved descendant, explain any pre-existing
hash change before continuing.

- [ ] Add structural RED assertions that require the seven exact workspace
  dependency entries, exact `extractum` `{ workspace = true }` inheritance,
  six members, no `extractum-telegram`, and unchanged effective Grammers
  revision/features. Run Telegram and Gemini contracts and observe those new
  cases fail against the pre-normalized manifest.
- [ ] Edit only `src-tauri/Cargo.toml`: promote the seven entries and replace
  the app's direct specifications with workspace inheritance. Do not add,
  remove, or change any effective dependency edge.
- [ ] Run locked metadata. `Cargo.lock` is expected byte-identical. If Cargo
  reports a required lock update, inspect the full package/feature diff and
  stop for plan review unless it proves a semantically identical normalization;
  never casually regenerate the lock.
- [ ] Run the feature baseline `--check`; it must remain byte-identical.
- [ ] Update the Gemini exact workspace-dependency contract and extend the
  shell-cap contract's retained-current-status acceptance through Checkpoint 2
  atomically. Run the shell-cap contract explicitly before candidate status,
  plus analysis, LLM, and prompt-pack contracts because they fingerprint the
  manifest/reachable graph:

```powershell
Invoke-CheckedNative 'Task 2 Telegram contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Task 2 Gemini contract' {
    npm.cmd run test -- src/lib/gemini-browser-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Task 2 shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'Task 2 analysis contracts' {
    npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Task 2 LLM and prompt contracts' {
    npm.cmd run test -- src/lib/llm-crate-boundary-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Task 2 feature authority' {
    node scripts/telegram-grammers-feature-baseline.mjs --check
}
Invoke-CheckedNative 'Task 2 locked metadata' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
}
```

- [ ] Obtain `Escape-hatch review verdict: CLEAN` from the mandatory reusable
  CP2–CP8 LLM retention gate, including CP2's explicit no-scoped-Rust
  escape-hatch-change finding. Create the cumulative verification document and
  copy the complete CLEAN record into its CP2 section. Only then update statuses
  to Checkpoint 2, run the `extractum` package checkpoint, assert 719 unique
  tests, and run:

```powershell
Assert-RustPackageTestTotal -Package extractum -ExpectedTotal 719
Invoke-RetainedCheckpointGates '8B-CP2'
```

- [ ] Inspect the manifest/contract/docs diff, prove the lock is unchanged (or
  document the separately reviewed metadata-only change), stage only the Task
  2 allowlist, and commit:

```powershell
Invoke-CheckedNative 'commit Task 2' {
    git commit -m "build: normalize Telegram workspace dependencies"
}
```

## Forward-Only Checkpoint 3 Authority Correction

**Effective scope:** this correction governs only unexecuted Task 3 and later
work after retained Checkpoint 2. It does not alter either retained checkpoint,
their artifacts, tests, status history, or verification evidence.

- Task 3 has an exact 25-path staging allowlist. It includes
  `src/lib/crate-extraction-shell-cap-contract.test.ts`, because candidate
  Checkpoint 3 status cannot pass the retained status contract without the
  corresponding CP3 vocabulary and prose.
- CP3 and CP4 additionally permit one exact package-private staged-root bridge:
  `TELEGRAM_PEER_KIND_CHANNEL`, `TELEGRAM_PEER_KIND_CHAT`, and
  `TELEGRAM_PEER_KIND_USER`, re-exported from `telegram_impl::dto` and consumed
  only by `sources::sync::fallback_message_identity`. Task 5 removes the bridge
  when that mapper moves into `telegram_impl/live/messages.rs`. This is a
  forward-only third transitional exception, not an externally public API and
  not an addition to the retained schema-v1 artifact.
- The mandatory checkpoint LLM review owns authorization of that bridge and
  every other temporary escape-hatch definition and physical use. TypeScript
  may continue checking permanent API shape, source ownership/paths,
  identities, dependency metadata, lifecycle vocabulary, and status prose, but
  it must not decide whether any temporary raw/media/peer-kind escape-hatch
  definition or use is allowed. Automatic evidence for those definitions and
  uses remains Rust compilation plus the named behavior tests.
- The CP3 review record names retained Checkpoint 2 commit
  `951b88b004ba4493a73bd2ccf93a2e8aa31dae6d` as the retained predecessor and
  separately records the later authority-correction commit as the actual
  implementation-diff base. The correction does not relabel the retained
  checkpoint.
- The CP3 Telegram contract must resolve the real staged source tree, keep
  historical fixtures isolated from current-file reads, route the 20 moved
  identities plus the one new runtime identity to `telegram_impl::`, and
  reject retained old leaf files/declarations. It must not reintroduce the
  semantic Rust analyzer removed by the post-CP1 amendment.

### Forward-Only Task 3 Analysis-Gate Compatibility Correction

Before retrying candidate Checkpoint 3 status, make one focused correction
commit containing only this plan and:

- `src/lib/analysis-application-contract.test.ts`
- `src/lib/analysis-crate-boundary-contract.test.ts`

The private `#[path = "telegram_impl/lib.rs"] mod telegram_impl;` root adds one
reachable production Rust file and exercises Rust's path-override module
semantics: child `mod` declarations resolve relative to the directory
containing the override target, not a synthetic directory named after its file
stem. Update the shared analysis reachability fixture to cover a nested child
of a path-overridden module. Make the frozen all-app production breadth exact
for both lifecycle states: 125 before the staged root exists and 126 once
`telegram_impl/lib.rs` exists, with that path required in the latter state.
Likewise retain the old exact unresolved-query context hash before Checkpoint 3
and select the new exact hash only after the staged root exists for the two
unchanged `ingest_provenance.rs` functions whose file context changes when the
permanent `TelegramMessageIdentity` import moves from `crate::telegram` to
`crate::telegram_impl`; keep their function/query identities exact. Do not
change analysis ownership, SQL, portable-crate, cfg-only, or application
behavior authority.

This is a verification-contract compatibility correction, not a Task 3
implementation path. It changes no Rust, status, generated artifact, or
escape-hatch authority. Commit it separately before candidate status, so the
literal 25-path Task 3 allowlist remains unchanged:

```powershell
Invoke-CheckedNative 'Task 3 analysis compatibility contracts' {
    npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'commit Task 3 analysis compatibility correction' {
    git commit -m "test: align analysis contracts with Telegram staged root"
}
```

## Task 3: Relocate the Prepared Foundation Into the Portable Root

**Files:**

- Create: `src-tauri/src/telegram_impl/lib.rs`
- Create: `src-tauri/src/telegram_impl/dto.rs`
- Create: `src-tauri/src/telegram_impl/media.rs`
- Create: `src-tauri/src/telegram_impl/runtime.rs`
- Create: `src-tauri/src/telegram_impl/session.rs`
- Delete after successful relocation: `src-tauri/src/telegram/dto.rs`
- Delete after successful relocation: `src-tauri/src/telegram/media.rs`
- Delete after successful relocation: `src-tauri/src/telegram/runtime.rs`
- Delete after successful relocation: `src-tauri/src/telegram/session.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/telegram.rs`
- Modify: `src-tauri/src/telegram_session_store.rs`
- Modify: `src-tauri/src/media.rs`
- Modify: `src-tauri/src/ingest_provenance.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/migrated_history.rs`
- Modify: `src-tauri/src/takeout_import/raw_parse.rs`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`

- [ ] Copy the four retained 8A leaves byte-for-byte into their staged owners
  with `apply_patch`; change only paths/visibility required by the new root.
  Preserve all 8A tests and relative `super::` leaf imports.
- [ ] Install the private app module exactly as:

```rust
#[path = "telegram_impl/lib.rs"]
mod telegram_impl;
```

Across the five-file staged tree, `telegram_impl/lib.rs` declares exactly four
private leaf modules: `dto`, `media`, `runtime`, and `session`. It curates
their permitted exports and contains no public module.

- [ ] Add a minimal compiling `TelegramRuntime::client` candidate that
  incorrectly delegates to `authorized_client`, plus the exact test:

```text
telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check
```

The fixture initializes an account whose authorization probe is false. Run:

```powershell
Invoke-Phase8BRedCases -TestNames @(
    'telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check'
)
```

Expected RED: one runtime test fails because non-authorizing lookup incorrectly
returns the unauthenticated error and emits only the literal
`PHASE8B_RED_RUNTIME_CLIENT_LOOKUP` assertion sentinel. Compiler failure is not
accepted.

- [ ] Implement `client` as the renamed public non-authorizing lookup over the
  existing map-lock behavior. Keep `initialized_client` private for internal
  reuse and keep `authorized_client` behavior/signature unchanged.
- [ ] In `initialize_grammers_client`, replace the implicit
  `grammers_client::Client::new(pool.handle)` with this fail-closed
  construction invariant. Keep its exact signature; do not add a configuration
  parameter merely to make the false-default branch reachable in a unit test:

```rust
async fn initialize_grammers_client(
    api_id: i32,
    session: &TelegramSession,
) -> extractum_core::error::AppResult<(TelegramClientInner, JoinHandle<()>, bool)> {
    let configuration =
        grammers_client::client::ClientConfiguration::default();
    if !configuration.auto_cache_peers {
        return Err(AppError::internal(
            "Grammers client configuration must enable auto_cache_peers",
        ));
    }
    let pool = SenderPool::new(session.clone_memory_session(), api_id);
    let runner = tokio::spawn(async move {
        let _ = pool.runner.run().await;
    });
    let client =
        grammers_client::Client::with_configuration(pool.handle, configuration);
    let is_authorized = client
        .is_authorized()
        .await
        .map_err(AppError::telegram_network)?;
    Ok((TelegramClientInner::Grammers(client), runner, is_authorized))
}
```

  The pinned default is true, so this does not change the retained runtime
  path. It turns the outer Grammers cache gate into an executable precondition:
  a future default/configuration drift fails during client initialization
  before any detached runner exists, instead of letting the raw live path cache
  peers under different rules. The false branch is intentionally unreachable
  under the pinned default and is protected structurally; do not weaken the
  ownership invariant to manufacture runtime coverage.
- [ ] At CP3, add the permanent restricted bridge exactly as
  `TelegramSession::clone_memory_session(&self) -> Arc<MemorySession>` with
  body `Arc::clone(&self.inner)`. Keep the existing borrowed
  `raw_memory_session(&self) -> &Arc<MemorySession>` only for the enumerated
  transitional consumers through CP6; do not silently change that method's
  signature in place. Without adding or renaming test identities, update the
  existing `encrypted_session_load_round_trips`,
  `initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner`,
  `failed_sign_in_retains_pending_attempt`, and
  `successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt`
  assertions to hold cloned `Arc` values and prove with `Arc::ptr_eq` that the
  bridge points to the same `MemorySession`.
- [ ] Redirect every future-owner type/constant/codec consumer to an explicit
  `crate::telegram_impl::...` path. Do not redirect app-owned
  `TelegramState`, Tauri commands, or account/diagnostics helpers out of
  `crate::telegram`. For CP3 and CP4, install only the exact package-private
  staged-root bridge
  `{TELEGRAM_PEER_KIND_CHANNEL,TELEGRAM_PEER_KIND_CHAT,TELEGRAM_PEER_KIND_USER}`
  for the sole `sources::sync::fallback_message_identity` consumer. It is
  reviewed as a temporary escape hatch by the mandatory LLM gate; TypeScript
  does not authorize its definition or use.
- [ ] Install only the exact CP3→CP6 media compatibility set:
  `DocumentSignals` with its current fields, `derive_content_kind`,
  `derive_document_media_kind`, and `extract_item_payload`. The staged root
  exports them and `src-tauri/src/media.rs` package-privately re-exports them
  for `sources/sync.rs` and `takeout_import/raw_parse.rs`; the lifecycle
  inventory remains the exact reviewer checklist. The checkpoint LLM review
  freezes those consumers and forbids any new one; TypeScript does not infer
  the facade consumers from Rust.
- [ ] Remove the `#[cfg(test)]` Telegram content-kind re-exports from app
  `media.rs`. Rewrite all six existing `sources/items.rs` test references,
  including fixture initializers and assertions, to use the unchanged
  `"text_only"` / `"text_with_media"` literals; keep every test identity and
  all production code unchanged.
- [ ] Keep the temporary package-private `raw_client`/`raw_session` adapters
  only for the still-unmoved live/Takeout consumers. The checkpoint LLM review
  must list each temporary callsite exactly; no new raw callsite may appear.
  The TypeScript contract does not infer or enforce that list from Rust.
- [ ] Delete the four old leaf files and their old module declarations only
  after the staged versions compile. There must be no duplicate DTO, item-kind
  literal, media payload, runtime, session codec, or test identity.
- [ ] Update the Telegram contract for CP3 before requesting the checkpoint
  LLM verdict: admit retained/candidate CP3 vocabulary; resolve current
  DTO/media/runtime/session source from `telegram_impl`; keep historical 8A
  fixtures independent of deleted current paths; require the four old leaves
  and their old declarations to be absent; recognize permanent
  `TelegramRuntime::client`, `TelegramSession::clone_memory_session`, and the
  exact moved/new identity ownership. Remove any remaining raw-use inventory
  branch that would classify a temporary escape-hatch definition or use.
  Do not add media-facade, raw-accessor, peer-kind-bridge, `SenderPool`
  argument, callsite, alias, forwarding, or occurrence authority to
  TypeScript.
- [ ] Extend the shell-cap contract's exact status/prose acceptance through
  Checkpoint 3. This is lifecycle/status authority only and performs no Rust
  semantic analysis.
- [ ] Run the exact new GREEN and all moved foundation suites:

```powershell
Invoke-ExactRustTest extractum 'telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check'
Invoke-NonEmptyRustSuite -Label 'staged DTO tests' -Package extractum -TestFilter 'telegram_impl::dto::tests::'
Invoke-NonEmptyRustSuite -Label 'staged media tests' -Package extractum -TestFilter 'telegram_impl::media::tests::'
Invoke-NonEmptyRustSuite -Label 'staged runtime tests' -Package extractum -TestFilter 'telegram_impl::runtime::tests::'
Invoke-NonEmptyRustSuite -Label 'staged session tests' -Package extractum -TestFilter 'telegram_impl::session::tests::'
Invoke-CheckedNative 'Task 3 extractum check' {
    cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
Invoke-CheckedNative 'Task 3 package checkpoint' {
    cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
```

- [ ] Assert exactly 720 unique `extractum` library IDs and the exact 21-ID
  staged foundation set: 20 mapped DTO/media/runtime/session identities plus
  the one new CP3 runtime identity. No old
  `telegram::{dto,media,runtime,session}` test prefix remains:

```powershell
Invoke-CheckedNative 'Task 3 identity authority' {
    node scripts/telegram-8b-test-identities.mjs --check
}
$Cp3IdentityAuthority =
    Get-Content -LiteralPath 'src/lib/telegram-8b-test-identities.json' -Raw |
    ConvertFrom-Json
$ExactCp3StagedTests = @(
    @($Cp3IdentityAuthority.preNewStaged) +
        @($Cp3IdentityAuthority.phase8BNewStaged) |
        Where-Object {
            $_ -match '^telegram_impl::(dto|media|runtime|session)::tests::'
        }
)
if ($ExactCp3StagedTests.Count -ne 21) {
    throw "Expected exact 21-ID CP3 staged foundation authority"
}
Assert-ExactRustIdentitySet `
    -Package extractum `
    -Prefix 'telegram_impl::' `
    -Expected $ExactCp3StagedTests
$OldFoundationTests = @(
    Get-RustTestIds -Package extractum |
        Where-Object {
            $_ -match '^telegram::(dto|media|runtime|session)::tests::'
        }
)
if ($OldFoundationTests.Count -ne 0) {
    $OldFoundationTests
    throw "Old Telegram foundation test identities survived CP3"
}
```

- [ ] Obtain `Escape-hatch review verdict: CLEAN` from the mandatory reusable
  CP2–CP8 LLM retention gate and append the complete CLEAN record to the
  cumulative verification document. Only then update candidate statuses to
  Checkpoint 3, run Telegram, shell-cap, media, analysis, LLM, and prompt-pack
  source contracts, then run:

```powershell
Invoke-CheckedNative 'Task 3 Telegram contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Task 3 shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Assert-RustPackageTestTotal -Package extractum -ExpectedTotal 720
Invoke-RetainedCheckpointGates '8B-CP3'
```

If a gate fails, the checkpoint is not retained: restore every candidate CP3
status/prose hunk in both status documents to the retained Checkpoint 2 state
with `apply_patch`, fix the cause, and rerun the complete Checkpoint 3 gates.

- [ ] Inspect the complete move/import diff and staged diff. Compare the sorted
  cached path list to the literal 25-path Task 3 allowlist above, stage no
  other path, and commit:

```powershell
Invoke-CheckedNative 'commit Task 3' {
    git commit -m "refactor: stage Telegram runtime foundation"
}
```

## Task 4: Move Peer and Avatar Transport Behind Owned Values

**Files:**

- Create: `src-tauri/src/telegram_impl/error.rs`
- Create: `src-tauri/src/telegram_impl/live/mod.rs`
- Create: `src-tauri/src/telegram_impl/live/avatar.rs`
- Create: `src-tauri/src/telegram_impl/live/peer.rs`
- Modify: `src-tauri/src/telegram_impl/{lib,dto,runtime}.rs`
- Modify:
  `src-tauri/src/sources/{avatar,identity,peer_resolution,store,types,mod}.rs`
- Modify: exact transitional consumers
  `src-tauri/src/sources/sync.rs` and
  `src-tauri/src/takeout_import/mod.rs`
- Modify: the Telegram contract and two status docs
- Modify:
  `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`

- [ ] Introduce compiling private test callbacks/fakes and deliberately
  semantic RED scaffolds for the three exact Checkpoint 4 tests:

```text
telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure
telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget
telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes
```

Run all three literal cases:

```powershell
Invoke-Phase8BRedCases -TestNames @(
    'telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure'
    'telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget'
    'telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes'
)
```

Expected reasons are respectively timeout/transport leakage, broken
dialog-next/avatar interleaving/user skipping or budget cutoff, and wrong
username/dialog/subtype/avatar result. Never use live Telegram credentials.

- [ ] Move `ResolvedTelegramSource` to the exact public `PeerDescriptor` DTO
  and move only the raw peer conversion/reconstruction symbols enumerated in
  the literal disposition table into `live/peer.rs`. The private peer
  reconstruction rules are:

```text
channel/supergroup + access_hash -> channel PeerRef with PeerAuth hash
group -> ambient chat only after the app-owned resolution plan succeeds
unsupported subtype/kind or invalid numeric ID -> existing validation error
```

- [ ] Freeze the two exact app adapters and one exact transitional raw builder:
  `peer_descriptor_from_stored_identity(source, identity)` supplies title,
  membership, subtype, external ID, username, and stored access hash from the
  already-loaded app rows; `peer_descriptor_to_source_info(descriptor)`
  performs only UI DTO/data-URL projection. Add `SourceSyncTarget::is_member`
  and select `sources.is_member` in `load_source`; the stored adapter reads that
  field rather than guessing membership. Neither adapter imports Grammers or
  performs network work. `legacy_peer_ref_from_descriptor(descriptor)` is the
  sole app-private Grammers builder used only to fill transitional
  `ResolvedSyncPeer::peer` through CP6; delete it with that field at CP7.
- [ ] Rewrite `typed_peer_resolution_plan` to determine stored-identity
  eligibility from typed identity fields without constructing a raw peer:
  channel/supergroup requires a stored access hash, while a small group remains
  dialog-dependent. Copy reconstruction into staged
  `peer_ref_from_descriptor` for staged internals and replace the four current
  app builders with the one transitional legacy builder above.
- [ ] Implement the final peer methods:

```text
dialog_listing(avatar_budget_ms)
DialogListing::next()
resolve_dialog_peer(peer_id, expected_subtype)
resolve_username(username, expected_subtype)
peer_avatar_bytes
```

The app retains the stored-channel-hash → username → dialog strategy,
manual-reference parser/dispatch, cache coordination, and metadata decisions.
`resolve_dialog_peer` receives the already parsed numeric ID and owns only
ordered transport lookup plus the existing not-found/subtype result.
`resolve_username` owns only username transport lookup plus subtype validation.
Both methods construct the descriptor from the already resolved raw `Peer`,
call the existing 750 ms avatar helper for that same `Peer`, and fill
`avatar_bytes` before returning. They perform no second username/dialog lookup.
Equal bare IDs with different peer kinds retain the current
wrong-subtype-versus-not-found behavior. A small group never takes the
stored-channel shortcut.

- [ ] Move only remote avatar download into `live/avatar.rs`. Preserve the
  exact 750 ms per-photo timeout, best-effort `None` on timeout/transport
  failure, and owned `Vec<u8>`. `list_telegram_sources` passes the exact
  app-owned `4_000` ms constant to `dialog_listing`; the opaque listing starts
  one budget clock before dialog iteration. `DialogListing` and its `next`
  implementation live together in `live/peer.rs`; the public handle method in
  `runtime.rs` delegates through `live::dialog_listing`. Each `next()` loops:
  advance the raw dialog iterator, map the peer, continue internally for an
  unsupported `Peer::User`, and for a supported peer attempt that same peer's
  750 ms avatar while budget remains before returning `Some`. It returns
  `None` only at true iterator exhaustion. The app converts returned bytes to a
  data URL before calling `next()` again, so presentation time continues to
  count toward the same 4,000 ms clock exactly as today. After expiry, `next()`
  continues dialog pagination in order but skips all later avatar calls and
  returns `avatar_bytes: None`. Cache path/key/read/write/delete and data URL
  stay app-owned.
- [ ] Make the dialog seam fixture exactly supported channel → unsupported
  user → supported group. Assert two descriptors in order, true EOF only after
  the group, and no avatar call for the user. In the resolution seam, assert
  username and dialog results contain avatar bytes from exactly one 750 ms call
  on the same resolved peer. Strengthen the existing exact store test
  `telegram_source_upsert_writes_required_identity_and_available_optional_fields`
  to retain its avatar-cache-key assertion and additionally load/assert
  `SourceSyncTarget::is_member`; do not add or rename a test identity.
- [ ] Extend the lifecycle-gated source contract to require the app-owned
  4,000 ms constant, the exact dialog-next/avatar call order, the elapsed
  guard before each avatar, and no avatar attempt after budget expiry.
- [ ] Create `error.rs` as the private classifier shell. Move the non-forum
  topic classifier at Checkpoint 5 with its consumer; leave
  channel-private/export-DC classifiers at their current owners until
  Checkpoint 7. It must export no externally reachable public item.
- [ ] Remove raw Grammers types/operations from `sources/avatar.rs`,
  `sources/identity.rs`, and the dialog-listing portion of
  `sources/store.rs`. Keep `sources/peer_resolution.rs::ResolvedSyncPeer::peer`
  as the one explicitly lifecycle-gated app-private raw field and add its pure
  `descriptor: PeerDescriptor` alongside it.
- [ ] Have the checkpoint LLM review freeze the exact
  `ResolvedSyncPeer::peer` consumers: one in
  `sources::sync::sync_telegram_source` and two Takeout resolution sites at
  CP4; exactly two Takeout resolution owners after CP5; zero at CP7.
- [ ] Keep exactly
  `TelegramClientHandle::{raw_client,raw_session}`,
  `TelegramSession::raw_memory_session`,
  `telegram::{get_client,get_authorized_client}`, and
  `{ResolvedSyncPeer::peer,legacy_peer_ref_from_descriptor}` as the CP3→CP7
  transitional bridge. `TelegramSession::clone_memory_session` is the permanent
  restricted runtime-initialization bridge and is not part of that deletion.
  The LLM review freezes the transitional definitions/callsites, rejects any
  new raw callsite, and requires deletion at CP7. TypeScript does not decide
  whether a Rust callsite belongs to this set.
- [ ] Run all three exact GREEN tests plus mapped peer/store suites:

```powershell
Invoke-ExactRustTest extractum 'telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure'
Invoke-ExactRustTest extractum 'telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget'
Invoke-ExactRustTest extractum 'telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes'
Invoke-NonEmptyRustSuite -Label 'staged peer tests' -Package extractum -TestFilter 'telegram_impl::live::peer::tests::'
Invoke-NonEmptyRustSuite -Label 'source peer coordinator tests' -Package extractum -TestFilter 'sources::peer_resolution::tests::'
Assert-ExactRustIdentitySet -Package extractum -Prefix 'sources::store::tests::' -Expected $ExactStoreTests
Invoke-CheckedNative 'Task 4 extractum check' {
    cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
Invoke-CheckedNative 'Task 4 package checkpoint' {
    cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
```

- [ ] Assert exactly 723 unique library IDs. Obtain
  `Escape-hatch review verdict: CLEAN` from the mandatory reusable CP2–CP8 LLM
  retention gate and append the complete CLEAN record to the cumulative
  verification document. Only then update candidate statuses to Checkpoint 4
  and require the contract to resolve exactly
  `telegram_impl/{error,live/mod,live/avatar,live/peer}.rs` at this lifecycle.
- [ ] Run:

```powershell
Assert-RustPackageTestTotal -Package extractum -ExpectedTotal 723
Invoke-RetainedCheckpointGates '8B-CP4'
```

On failure, restore the prior status text before fixing. Inspect/stage only
Task 4 and commit:

```powershell
Invoke-CheckedNative 'commit Task 4' {
    git commit -m "refactor: stage Telegram peer and avatar boundary"
}
```

## Task 5: Replace Live History and Topic Consumers With Owned Batches

**Files:**

- Create: `src-tauri/src/telegram_impl/live/messages.rs`
- Create: `src-tauri/src/telegram_impl/live/topics.rs`
- Modify:
  `src-tauri/src/telegram_impl/{lib,dto,error,media,runtime,session}.rs`
- Modify: `src-tauri/src/telegram_impl/live/mod.rs`
- Modify:
  `src-tauri/src/sources/{items,peer_resolution,sync,topics,mod}.rs`
- Modify: `src-tauri/src/takeout_import/forum_topics.rs` to use the shared
  owned `fetch_forum_topics` operation during the CP5→CP7 transition
- Modify: the Telegram contract and two status docs
- Modify:
  `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`

- [ ] Add compiling semantic RED scaffolds for:

```text
telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule
telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload
telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor
sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error
```

Run all four literal RED cases:

```powershell
Invoke-Phase8BRedCases -TestNames @(
    'telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule'
    'telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload'
    'telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor'
    'sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error'
)
```

The raw message fixture must count invokes and begin with an empty
`TelegramSession`. After the one raw invoke, the first test queries the backing
`MemorySession::peer` state through the pinned `grammers_session::Session`
trait for each exact fixture peer ID: every auth-eligible peer must be present,
while the min user/channel peers without auth must remain absent. This proves
the real cache side effect without adding an instrumented `Session` seam. The
first test is explicitly downstream of the Checkpoint 3
`auto_cache_peers == true` construction invariant and fails unless one batch
performs exactly one invoke, updates the in-memory peer cache with the exact
pinned `Peer::auth().is_some()` inner predicate (including an ordinary chat
with default auth and excluding min user/channel fixtures without auth),
validates 1..=100, preserves response order, advances both offsets, and
implements the pinned terminal rule. Its final subcase supplies
`Messages::NotModified { count: 0 }`, requires an `Err` whose kind is exactly
`AppErrorKind::Network` and whose message equals
`Telegram returned messagesNotModified for live history batch`, and therefore
fails if the implementation panics, accepts the count, or chooses another
error mapping. The lifecycle source contract separately fails if the validated
outer gate is removed or bypassed. The second fails on per-entry
conversion/skip/fallback-identity drift. The third fails on topic/deletion
order or non-advancing terminal cursor drift. The app test fails unless prior
entries remain durable, every returned raw message consumes the RecentMessages
limit even when conversion skips it, and a conversion/persistence error
prevents the next fetch.

- [ ] Implement `fetch_message_batch` directly over one raw
  `messages.getHistory`; do not use `MessageIter::next`. Recreate the pinned
  `build_peer_map` result and update the runtime's in-memory
  `TelegramSession` peer cache before return, without a second invoke. This
  update is permitted only because every real client passed to this path was
  created under the checked `auto_cache_peers == true` invariant in Task 3;
  do not add an independent unconditional-cache policy. Keep `LiveMessage`
  raw/owned internally so `into_draft` runs only after app cutoffs.
- [ ] Match `Messages::NotModified` before peer-map/cache work and return
  exactly
  `AppError::network("Telegram returned messagesNotModified for live history batch")`.
  This is the declared replacement for pinned Grammers' panic, not a new
  generic error abstraction and not permission to alter any other error.
- [ ] Remove the `sources/sync.rs` use of the transitional media facade after
  conversion moves. At CP5/CP6 the only remaining app consumer of the exact
  compatibility set is `takeout_import/raw_parse.rs`.
- [ ] Rewrite `sources/sync.rs::persist_items` as this exact bounded loop:

```text
remaining = RecentMessages value, otherwise unbounded
offset_id = 0; offset_date = 0
fetch min(remaining, 100) or 100
for each message in returned order:
  decrement remaining for this returned raw message, before any cutoff/skip
  if id <= previous_last_sync: stop successfully
  if published_at < date cutoff: stop successfully
  update in-memory max_message_id
  into_draft(source title)
  persist or count skipped
if terminal or remaining reached zero: stop
else advance both offsets and fetch next batch
```

Do not finalize the source, move the durable watermark, or delay an insertion
across a different error boundary.

- [ ] Factor only this app-owned coordinator into a private fakeable helper and
  implement the exact app test above with three subcases: a second entry fails
  after the first insert, an empty-payload message still consumes one requested
  slot across batches, and any conversion/persistence error leaves the fake
  fetch count unchanged after the failing batch. Add no cross-domain public
  trait or generic staged callback.

- [ ] Move raw author/reply/context/raw-data/fallback-identity conversion from
  `sources/items.rs`/`sources/sync.rs` into `live/messages.rs`. Keep all SQL,
  transactions, `PreparedSourceItem`, insert outcomes, read models, and app
  tests in `sources/items.rs`. Remove the exact three-constant CP3→CP4
  staged-root peer-kind bridge in the same checkpoint; after the mapper moves,
  no app consumer or replacement bridge remains.
- [ ] Move the full remote forum-topic loop and snapshot mapping to
  `live/topics.rs`; return `Ok(None)` only for the two existing non-forum RPC
  classifiers and `Ok(Some((topics, deleted_ids)))` otherwise. Keep subtype
  gate, SQL upsert/deletion, membership rebuild, warnings, and read models in
  `sources/topics.rs`.
- [ ] Move `is_non_forum_topic_refresh_error` to `telegram_impl/error.rs` in
  this checkpoint. Both the normal refresh and the temporary Takeout
  completion path call public `TelegramClientHandle::fetch_forum_topics`; no
  raw topic loop or raw bridge survives CP5. Checkpoint 7 adds only the
  Takeout-owned delegation/test in `takeout/forum_topics.rs`.
- [ ] After live consumers are converted, raw-handle accessors may remain only
  under the three exact raw-handle logical owners/callsites
  `run_export_dc_spike_for_handle`,
  `run_takeout_migrated_history_import`, and
  `run_takeout_source_import`. Separately,
  `ResolvedSyncPeer::peer` may remain under only the two Takeout resolution
  owners `run_takeout_migrated_history_import` and
  `run_takeout_source_import`. The LLM review confirms both exact inventories;
  separately assert no Grammers import remains in
  `sources/{items,sync,topics}.rs`.
- [ ] Run the four exact GREEN tests and focused suites:

```powershell
Invoke-ExactRustTest extractum 'telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule'
Invoke-ExactRustTest extractum 'telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload'
Invoke-ExactRustTest extractum 'telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor'
Invoke-ExactRustTest extractum 'sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error'
Invoke-NonEmptyRustSuite -Label 'live message tests' -Package extractum -TestFilter 'telegram_impl::live::messages::tests::'
Invoke-NonEmptyRustSuite -Label 'live topic tests' -Package extractum -TestFilter 'telegram_impl::live::topics::tests::'
Invoke-NonEmptyRustSuite -Label 'sync app tests' -Package extractum -TestFilter 'sources::sync::tests::'
Invoke-NonEmptyRustSuite -Label 'topic app tests' -Package extractum -TestFilter 'sources::topics::tests::'
Invoke-CheckedNative 'Task 5 extractum check' {
    cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
Invoke-CheckedNative 'Task 5 package checkpoint' {
    cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
```

- [ ] Assert exactly 727 unique library IDs and the exact staged mapped-prefix
  counts through live messages/peer/media/runtime/session. Obtain
  `Escape-hatch review verdict: CLEAN` from the mandatory reusable CP2–CP8 LLM
  retention gate and append the complete CLEAN record to the cumulative
  verification document. Only then update candidate statuses to Checkpoint 5,
  run all boundary contracts, then run:

```powershell
Assert-RustPackageTestTotal -Package extractum -ExpectedTotal 727
Invoke-RetainedCheckpointGates '8B-CP5'
```

- [ ] On GREEN, inspect/stage only Task 5 and commit:

```powershell
Invoke-CheckedNative 'commit Task 5' {
    git commit -m "refactor: stage Telegram live message and topic operations"
}
```

## Task 6: Decompose the Two Raw-TL/SQL Baselines

**Files:**

- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/raw_parse.rs`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify:
  `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify:
  `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`

This is a characterization decomposition, not new production behavior. The two
companion tests are expected GREEN immediately because the raw parser already
preserves the identity; a compile failure or intentionally broken production
stub is not useful RED evidence here.

- [ ] Rewrite
  `takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers`
  to use two preconstructed `TelegramMessageDraft`s:

```text
draft A: history peer channel 12345, Telegram message 42
draft B: history peer chat 777, Telegram message 42
```

Keep the application assertion that both rows insert and:

```sql
SELECT COUNT(*) FROM items WHERE external_id = '42'
```

returns exactly `2`. Remove raw TL fixture construction from the app test.

- [ ] Add the temporary companion in the current raw parser:

```text
takeout_import::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids
```

It constructs the two raw fixtures and proves equal message ID plus distinct
history peer kind/ID. It performs no SQL.

- [ ] Rewrite
  `takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once`
  to use two preconstructed drafts with channel `12345`, message `42`, and
  distinct content/raw payload. Retain:

```text
first outcome = inserted
second outcome = duplicate/skipped
final telegram_topic_resolution_state = ("ready", 1)
```

- [ ] Add the temporary companion:

```text
takeout_import::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id
```

It proves payload differences do not alter the identical channel/message
identity. Move `takeout_raw_message_for_identity_test` into raw-parser test
support. No app test may retain `grammers_client::tl`.

- [ ] Run both app primaries and both companions exactly:

```powershell
Invoke-ExactRustTest extractum 'takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers'
Invoke-ExactRustTest extractum 'takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once'
Invoke-ExactRustTest extractum 'takeout_import::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids'
Invoke-ExactRustTest extractum 'takeout_import::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id'
Assert-NoMatches `
  -Label 'raw TL leaked into decomposed app tests' `
  -Pattern 'grammers_client::tl|takeout_raw_message_for_identity_test' `
  -Paths @('src-tauri/src/takeout_import/mod.rs')
Invoke-CheckedNative 'Task 6 extractum check' {
    cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
Invoke-CheckedNative 'Task 6 package checkpoint' {
    cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
```

- [ ] Assert exactly 729 unique library IDs, 143 present baseline-derived IDs,
  and 168 present tracked IDs. The contract must map the companions to their
  temporary Checkpoint 6 IDs and reserve their staged Checkpoint 7 IDs.
- [ ] Obtain `Escape-hatch review verdict: CLEAN` from the mandatory reusable
  CP2–CP8 LLM retention gate and append the complete CLEAN record to the
  cumulative verification document. Only then update candidate statuses to
  Checkpoint 6, run the Telegram contract, and run:

```powershell
Assert-RustPackageTestTotal -Package extractum -ExpectedTotal 729
Invoke-RetainedCheckpointGates '8B-CP6'
```

- [ ] Inspect the test-only decomposition diff, stage only Task 6, and commit:

```powershell
Invoke-CheckedNative 'commit Task 6' {
    git commit -m "test: split Telegram raw identity from app persistence"
}
```

## Task 7: Complete the Concrete Takeout Boundary and Seal the Portable Tree

**Files:**

- Create: `src-tauri/src/telegram_impl/takeout/mod.rs`
- Create: `src-tauri/src/telegram_impl/takeout/types.rs`
- Create: `src-tauri/src/telegram_impl/takeout/transport.rs`
- Create: `src-tauri/src/telegram_impl/takeout/export_dc.rs`
- Create: `src-tauri/src/telegram_impl/takeout/operations.rs`
- Create: `src-tauri/src/telegram_impl/takeout/pagination.rs`
- Create: `src-tauri/src/telegram_impl/takeout/raw_parse.rs`
- Create: `src-tauri/src/telegram_impl/takeout/forum_topics.rs`
- Modify:
  `src-tauri/src/telegram_impl/{lib,error,media,runtime,session}.rs`
- Modify: `src-tauri/src/{media,telegram}.rs`
- Modify:
  `src-tauri/src/takeout_import/{mod,forum_topics,migrated_history}.rs`
- Modify:
  `src-tauri/src/sources/{peer_resolution,topics}.rs` only where final pure
  peer/topic coordination requires it
- Delete after successful relocation:
  `src-tauri/src/takeout_import/{export_dc,pagination,raw_parse}.rs`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify:
  `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify:
  `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`

### Task 7A: Owned Transport and Attempt/Fallback Ordering

- [ ] Add a deterministic fake transport and a compiling semantic RED for:

```text
telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error
```

The test must prove the attempt snapshot exists before the fake remote future
is polled, contains both the unchanged `home_dc_id` and shifted
`export_dc_id`, and that `ExportDc` fallback metadata can be drained after
both `Ok` and `Err`. This staged test proves only transport-owned snapshot,
queue, and original remote result behavior; app recorder precedence is proved
by the existing app tests below.

```powershell
Invoke-Phase8BRedCases -TestNames @(
    'telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error'
)
```

- [ ] Move export-DC alias/selection/invocation and its seven mapped tests into
  `takeout/{transport,export_dc}.rs`. Replace the generic public invocation
  shape with private raw helpers only; no `RemoteCall` appears in a public
  signature.
- [ ] Implement `TakeoutAttempt`, `TakeoutFallbackKind`,
  `TakeoutFallback`, and `TakeoutTransport` exactly as frozen. The fallback
  queue is drained, not borrowed, and deduplication remains app-owned using the
  durable batch state. `TakeoutAttempt::home_dc_id()` preserves the existing
  `TakeoutExportDcSpikeResult.home_dc_id`; the app must never duplicate the
  private export-DC shift constant.
- [ ] Keep all fields private and make the sibling-module implementation
  compile only through the exact restricted methods in the allowlist:
  constructors for attempt/fallback/peer/range/count/page/message, private
  peer/page accessors, and
  `TakeoutTransport::{new,queue_fallback,client,session,home_dc_id,export_dc_id}`.
  `runtime.rs` reaches Takeout only through the three parent facade functions;
  add no field visibility, `pub(crate)`, or second raw-session adapter.

### Task 7B: Concrete Operations, Migration, and Only-My Continuations

- [ ] Add compiling semantic RED scaffolds for:

```text
telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges
telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity
telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome
telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages
telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping
```

Each fake records exact call order and returns raw fixtures. Run every literal
case:

```powershell
Invoke-Phase8BRedCases -TestNames @(
    'telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges'
    'telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity'
    'telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome'
    'telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages'
    'telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping'
)
```

Accepted reasons are wrong source-subtype flags/range selection, wrong migrated
chat identity, missing OnlyMy fallback metadata, wrong page/search ownership
or pagination state, and wrong finish mapping. The history-count test also has
a `Messages::NotModified { count: 0 }` subcase that requires exact
`AppErrorKind::Network`, exact message
`Telegram returned messagesNotModified for Takeout history count probe`, no
queued `OnlyMyMessages` fallback, and no search continuation. Compiler REDs
are forbidden.

- [ ] Move self-check, prepare/init, validation, migration, split selection,
  count, history, search-my, and finish logic into the frozen concrete
  operations. `TakeoutPeer::from_descriptor` is the only public source-peer
  constructor; migrated revalidation returns the old-chat `TakeoutPeer` plus
  ID without exposing a raw peer.
- [ ] Preserve the app transition around every concrete call:

```rust
// Application pseudocode; one outer cancellation select is preserved.
enum FallbackRecordState {
    Empty,
    Pending(Vec<TakeoutFallback>),
    InFlight(Vec<TakeoutFallback>),
    Recorded,
    Failed {
        fallbacks: Vec<TakeoutFallback>,
        error: AppError,
    },
}

enum StepOutcome<T> {
    Remote(Result<T, AppError>),
    MetadataStopped(Result<T, AppError>),
    AttemptStopped {
        remote: Result<T, AppError>,
        error: AppError,
    },
}

check_existing_cancellation_state()?;
record_attempt_if_needed(transport.export_dc_attempt()).await?;
let mut record_state = FallbackRecordState::Empty;
let selected_outcome: AppResult<StepOutcome<_>> =
    run_takeout_step_with_cancel(
        cancellation_token.clone(),
        async {
        let history_result = transport.history_count(...).await;
        queue_drained_fallbacks(
            &mut record_state,
            transport.drain_fallbacks(),
        );

        if is_classified_only_my(&history_result) {
            // Pending -> InFlight before await; capture completion without `?`.
            record_pending_once(&mut record_state).await;
            if matches!(record_state, FallbackRecordState::Failed { .. }) {
                return Ok(StepOutcome::MetadataStopped(history_result));
            }

            // The outer pre-check covers this pair: record the attempt, but do
            // not perform a second cancellation pre-check or select.
            if let Err(error) =
                record_attempt_if_needed(transport.export_dc_attempt()).await
            {
                return Ok(StepOutcome::AttemptStopped {
                    remote: history_result,
                    error,
                });
            }
            let search_result = transport.search_my_history_count(...).await;
            queue_drained_fallbacks(
                &mut record_state,
                transport.drain_fallbacks(),
            );
            return Ok(StepOutcome::Remote(search_result));
        }
        Ok(StepOutcome::Remote(history_result))
        },
    ).await;
queue_drained_fallbacks(&mut record_state, transport.drain_fallbacks());
record_after_select_once(&mut record_state).await;
resolve_metadata_precedence(record_state, selected_outcome)
```

For a classified channel-private result from `history_count`/`history_page`,
capture the `Err`, record the drained `OnlyMyMessages` warning and provenance,
then invoke `search_my_history_count`/`search_my_history_page` inside that same
outer selected future. `record_state` is declared outside the future so a
cancellation during metadata persistence leaves its owned values in
`InFlight`; after the selected future returns, drain once more and apply
precedence. Do not use `?` on a remote result or recorder completion before
metadata is captured. Do not emit progress, perform a second select/pre-check,
or persist a page before the search continuation.

`queue_drained_fallbacks` has only these legal transitions: `Empty` or
`Recorded` plus nonempty values becomes `Pending`; `Pending` extends in order;
empty input is a no-op; `InFlight` and `Failed` reject new values as an
invariant error. `record_pending_once` moves the owned vector
`Pending → InFlight` before awaiting persistence, then to `Recorded` on success
or to `Failed { fallbacks, error }` on a completed error. It never uses `?`.
`record_after_select_once` records `Pending` normally, makes exactly one
cancellation-recovery attempt for retained `InFlight`, and does nothing for
`Empty`, `Recorded`, or `Failed`. A completed `Failed` recorder is therefore
never called a second time; recovery of a dropped in-flight attempt is not a
loop or retry after completion and relies on the existing durable dedupe.

`run_takeout_step_with_cancel` keeps its retained
`Future<Output = AppResult<T>> → AppResult<T>` signature; no callsite or wrapper
signature changes. `resolve_metadata_precedence` is app-private concrete
choreography, not a generic repository/transport port. Its exhaustive result
table is:

```text
OnlyMyMessages recorder failure (any remote/outer result)
  -> recorder error; search was not issued
attempt recorder failure before search
  -> attempt error; search was not issued
ExportDc-only recorder failure + remote Err or outer cancellation
  -> original remote/cancellation error
ExportDc-only recorder failure + remote Ok
  -> recorder error
no recorder/attempt failure
  -> search/history result, or outer cancellation error
```

No `Failed` recorder is called again.

- [ ] Preserve exact OnlyMy texts:

```text
warning:
Channel history is private; falling back to messages.search(from_id=self).

provenance:
Channel history is private; importing only messages visible through from_id=self fallback.
```

Validation preflight remains `Ok(())` plus drained fallback. Groups never use
OnlyMy fallback. A validation fallback is metadata only: every range still
calls `history_count` first, and only the returned `TakeoutCount` selects page
routing. Export shifted→home remains inside one concrete call.

- [ ] Strengthen and run the three existing app-owned identities without
  renaming them:

```powershell
Invoke-ExactRustTest extractum 'takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize'
Invoke-ExactRustTest extractum 'takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation'
Invoke-ExactRustTest extractum 'takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future'
```

The first proves original-remote-error precedence when export fallback
recording fails, adds the remote-success/recorder-failure subcase that returns
the recorder error, and proves the completed recorder is called exactly once
in both subcases. The
second proves mandatory record-before-search, exactly one outer cancellation
pre-check, attempt recording immediately before search, no search after an
OnlyMy recorder failure, and that a validation-only warning does not bypass the
first history count. The third cancels while persistence is `InFlight` and
proves exactly one recovery record attempt, one outer select, and no search or
progress afterward.

### Task 7C: Pagination, Raw Parsing, Forum Topics, and App Loop

- [ ] Move `pagination.rs` and `raw_parse.rs` production/tests to their exact
  staged owners. Move the two temporary companions to their final staged IDs
  byte-for-byte; delete their temporary IDs.
- [ ] Without adding or renaming a test identity, strengthen the moved
  `messages_not_modified_response_is_rejected_for_takeout_page` test to
  require exact `AppErrorKind::Network` and exact message
  `Telegram returned messagesNotModified for Takeout history page` rather than
  substring matching. Together with the live-batch and count-probe subcases,
  this freezes all three operation-specific typed rejections.
- [ ] While moving raw parsing, replace cross-module construction of
  `DocumentSignals` with the exact
  `derive_document_media_kind_from_parts(mime_type, has_video, has_audio,
  is_voice, is_animated)` bridge. Delete the four CP3→CP6 root/app media
  compatibility re-exports, make `DocumentSignals` and its fields private, and
  retain the existing classifier tests plus raw-parser results unchanged.
- [ ] `TakeoutPage` privately owns pagination profile/cursor/page index,
  terminal/advance state, OnlyMy state, and messages. On TDesktop restart it
  returns zero messages, `has_next() == true`, and one page-local fallback
  warning; the app emits the warning/checks cancellation before the next
  concrete call. Store the exact sibling-restricted pagination types and use
  only the frozen `(profile, cursor, page_index)` tuple bridge; no app or root
  caller sees them. Do not globally dedupe pagination fallback across ranges.
- [ ] For every `TakeoutMessage`, the app:

```text
checks range lower bound
durably updates max_message_id before conversion
calls into_draft(source title)
persists or counts skipped
updates progress/events after the page
checks cancellation
uses page.has_next() to select the next concrete call
```

SQL, transactions, observations, migrated-history identity override, events,
and finalization remain application-owned.

- [ ] Add the final semantic RED/GREEN:

```text
telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots
```

```powershell
Invoke-Phase8BRedCases -TestNames @(
    'telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots'
)
```

Implement it by the explicitly shared `live::fetch_forum_topics` parent facade,
which delegates to `live/topics.rs`; do not copy the raw topic pagination loop.
Keep completion policy and warning/provenance tests in app
`takeout_import/forum_topics.rs`.

### Task 7D: Remove Raw Escape Hatches and Prove the Final Tree

- [ ] Delete `TelegramClientHandle::{raw_client,raw_session}`,
  `TelegramSession::raw_memory_session`,
  `telegram::{get_client,get_authorized_client}`, and
  `{ResolvedSyncPeer::peer,legacy_peer_ref_from_descriptor}` after their final
  two Takeout consumers use owned operations. Before deletion, every remaining
  production and test consumer switches from the transitional borrowed
  accessor to permanent restricted
  `TelegramSession::clone_memory_session`, including same-module session
  codec/tests, runtime initialization/tests, and sibling modules. CP7 and later
  contain zero `raw_memory_session` identifiers anywhere under
  `src-tauri/src`, while the exact restricted `clone_memory_session`
  definition remains; this zero-use/deletion claim is established by the
  checkpoint LLM review plus compilation/behavior gates, not a TypeScript Rust
  parser. Package-private
  `TelegramState` methods may delegate to
  `TelegramRuntime::{client,authorized_client}`; they expose only the opaque
  handle. Public handle methods that need raw internals are implemented in
  `runtime.rs` and delegate by relative path through the exact `pub(super)`
  allowlist, so no raw accessor is visible to an app module.
- [ ] Produce exactly the 19-file portable tree. Assert no extra/missing path
  and the exact 15 staged raw-consumer paths:

```text
telegram_impl/error.rs
telegram_impl/live/avatar.rs
telegram_impl/live/messages.rs
telegram_impl/live/peer.rs
telegram_impl/live/topics.rs
telegram_impl/media.rs
telegram_impl/runtime.rs
telegram_impl/session.rs
telegram_impl/takeout/export_dc.rs
telegram_impl/takeout/forum_topics.rs
telegram_impl/takeout/operations.rs
telegram_impl/takeout/pagination.rs
telegram_impl/takeout/raw_parse.rs
telegram_impl/takeout/transport.rs
telegram_impl/takeout/types.rs
```

- [ ] Run fail-closed scans:

```powershell
Assert-NoMatches `
  -Label 'staged crate-qualified import' `
  -Pattern '\bcrate::' `
  -Paths @('src-tauri/src/telegram_impl')
Assert-NoMatches `
  -Label 'staged app/Tauri/SQL/keyring leakage' `
  -Pattern 'sqlx|tauri|keyring|crate::error|crate::compression|crate::time' `
  -Paths @('src-tauri/src/telegram_impl')
Assert-NoMatches `
  -Label 'raw app leakage' `
  -Pattern 'grammers_|RemoteCall|InvocationError|PeerRef|tl::' `
  -Paths @(
    'src-tauri/src/telegram.rs',
    'src-tauri/src/sources',
    'src-tauri/src/takeout_import',
    'src-tauri/src/telegram_session_store.rs'
  )
```

`rg` exit 1 is clean; any other nonzero is infrastructure failure. The
checkpoint LLM review separately confirms that every named transitional escape
hatch and every alias/forwarding use is absent; do not add those names back to
an automatic occurrence scan. Separately assert that every
application use of a staged public symbol begins with exact
`crate::telegram_impl::` and no alias/glob exists. Parse
`src/lib/telegram-8b-symbol-map.json`; require every CP7 final symbol, every
app-retained replacement, the review-only CP7 empty transitional inventory,
and no unlisted production `pub(super)` item. Compare the complete normalized
`pub(super)` inventory to the artifact's `restrictedFinalSymbols`; do not
parse Rust escape-hatch use-sites or hard-code a second pagination allowlist
in TypeScript.

- [ ] Run all seven new Task 7 tests, both final companions, and the complete
  mapped Takeout suites:

```powershell
Invoke-ExactRustTest extractum 'telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error'
Invoke-ExactRustTest extractum 'telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges'
Invoke-ExactRustTest extractum 'telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity'
Invoke-ExactRustTest extractum 'telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome'
Invoke-ExactRustTest extractum 'telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages'
Invoke-ExactRustTest extractum 'telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping'
Invoke-ExactRustTest extractum 'telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots'
Invoke-ExactRustTest extractum 'telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids'
Invoke-ExactRustTest extractum 'telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id'
Invoke-NonEmptyRustSuite -Label 'Takeout export DC tests' -Package extractum -TestFilter 'telegram_impl::takeout::export_dc::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout operation tests' -Package extractum -TestFilter 'telegram_impl::takeout::operations::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout pagination tests' -Package extractum -TestFilter 'telegram_impl::takeout::pagination::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout raw parse tests' -Package extractum -TestFilter 'telegram_impl::takeout::raw_parse::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout app tests' -Package extractum -TestFilter 'takeout_import::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout forum app tests' -Package extractum -TestFilter 'takeout_import::forum_topics::tests::'
Invoke-NonEmptyRustSuite -Label 'migrated history app tests' -Package extractum -TestFilter 'takeout_import::migrated_history::tests::'
```

- [ ] Materialize and verify the exact identity partitions, then run:

```powershell
Invoke-CheckedNative 'Phase 8B identity authority' {
    node scripts/telegram-8b-test-identities.mjs --check
}
Assert-Phase8BFinalTestInventory `
    -AuthorityPath 'src/lib/telegram-8b-test-identities.json' `
    -ExpectedPackageTotal 736
Assert-ExactRustIdentitySet `
    -Package extractum `
    -Prefix 'sources::store::tests::' `
    -Expected $ExactStoreTests
```

This requires exactly 143 baseline-derived, 57 pre-new staged, 14 new staged,
103 pre-new app, one new app, 175 tracked, 104 app, 71 staged, and 736 unique
package IDs. The staged prefix is compared against the complete 71-ID union;
the 57/14 partition is separately count-, overlap-, and membership-checked.
- [ ] Obtain `Escape-hatch review verdict: CLEAN` from the mandatory reusable
  CP2–CP8 LLM retention gate and append the complete CLEAN record to the
  cumulative verification document. Only then update candidate statuses to
  Checkpoint 7. Run Telegram, shell-cap, media, Gemini, analysis, LLM, and
  prompt-pack contracts plus feature baseline check; then run:

```powershell
Invoke-RetainedCheckpointGates '8B-CP7'
```

- [ ] Inspect all moves and every app residual carefully. Stage only Task 7
  and commit:

```powershell
Invoke-CheckedNative 'commit Task 7' {
    git commit -m "refactor: complete staged Telegram Takeout boundary"
}
```

## Task 8: Generate Portable Evidence, Prove Runtime Registration, and Retain 8B

**Files:**

- Create: `scripts/telegram-staging-sha256.mjs`
- Create: `src/lib/telegram-8b-staging-sha256.json`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify:
  `docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md`
- Modify:
  `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`

### Task 8A: Content-Address the Exact Portable Tree

- [ ] Add compiling structural RED cases:

```text
requires the exact generated Phase 8B staging hash manifest
rejects missing extra reordered or byte-drifted staged files
accepts only the terminal Phase 8B retained status pair
```

Run the focused contracts and confirm failure is due to the missing generator
artifact/terminal evidence, not TypeScript compilation.

- [ ] Implement `scripts/telegram-staging-sha256.mjs` with exact `--write` and
  `--check` modes. It resolves the repository root safely, requires the exact
  19 forward-slash relative paths, reads raw bytes, writes lowercase SHA-256
  records in sorted order, and rejects symlink/realpath escape, missing/extra
  files, duplicate records, wrong root/schema/algorithm, or unsupported args.
- [ ] Run `--write`, save the first artifact hash, run `--write` again, and
  prove both the artifact bytes and its own hash are unchanged. Then run:

```powershell
$stagingArtifact =
    'src/lib/telegram-8b-staging-sha256.json'
Invoke-CheckedNative 'first staging hash write' {
    node scripts/telegram-staging-sha256.mjs --write
}
$firstStagingBytes =
    [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $stagingArtifact))
$firstStagingHash =
    (Get-FileHash -LiteralPath $stagingArtifact -Algorithm SHA256).Hash
Invoke-CheckedNative 'second staging hash write' {
    node scripts/telegram-staging-sha256.mjs --write
}
$secondStagingBytes =
    [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $stagingArtifact))
$secondStagingHash =
    (Get-FileHash -LiteralPath $stagingArtifact -Algorithm SHA256).Hash
if (
    $firstStagingHash -ne $secondStagingHash -or
    [Convert]::ToBase64String($firstStagingBytes) -cne
        [Convert]::ToBase64String($secondStagingBytes)
) {
    throw 'Staging hash --write is not byte-for-byte idempotent'
}
Invoke-CheckedNative 'staging hash check' {
    node scripts/telegram-staging-sha256.mjs --check
}
Invoke-CheckedNative 'Grammers feature check' {
    node scripts/telegram-grammers-feature-baseline.mjs --check
}
Invoke-CheckedNative 'test identity authority check' {
    node scripts/telegram-8b-test-identities.mjs --check
}
Invoke-CheckedNative 'symbol disposition check' {
    node scripts/telegram-8b-symbol-map.mjs --check
}
```

- [ ] Make the Telegram contract compare every staged byte/hash record and
  freeze the 8C destination root
  `src-tauri/crates/extractum-telegram/src` without creating it.

### Task 8B: Final Deterministic Gates

- [ ] Run `Assert-Phase8BFinalTestInventory` against the generated authority.
  Require exactly 736 unique IDs, 143 baseline-derived identities, 175 tracked
  identities, 104 app identities, 71 staged identities, all 15 new seam
  identities, both final companion identities, and the independent exact
  24-store set.
- [ ] Execute the independent store set rather than relying on its count:

```powershell
Assert-Phase8BFinalTestInventory `
    -AuthorityPath 'src/lib/telegram-8b-test-identities.json' `
    -ExpectedPackageTotal 736
Assert-ExactRustIdentitySet `
    -Package extractum `
    -Prefix 'sources::store::tests::' `
    -Expected $ExactStoreTests
```
- [ ] Run the complete focused contract set:

```powershell
Invoke-CheckedNative 'final Telegram contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'final shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'final media contract' {
    npm.cmd run test -- src/lib/media-metadata-core-contract.test.ts
}
Invoke-CheckedNative 'final Gemini contract' {
    npm.cmd run test -- src/lib/gemini-browser-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'final analysis contracts' {
    npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'final LLM and prompt contracts' {
    npm.cmd run test -- src/lib/llm-crate-boundary-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts
}
```

- [ ] Prove locked metadata has exactly six workspace members, no
  `extractum-telegram`, all four Grammers roots remain direct on `extractum`,
  and the feature artifact remains exact.
- [ ] Obtain `Escape-hatch review verdict: CLEAN` from the mandatory reusable
  CP2–CP8 LLM retention gate and append the complete CLEAN record to the
  cumulative verification document. Only then set candidate statuses to
  Checkpoint 8 and run:

```powershell
Invoke-RetainedCheckpointGates '8B-CP8'
```

Do not retain or commit Checkpoint 8 if any deterministic gate fails.

### Task 8C: Live MCP Command-Registration Smoke

This smoke is non-mutating and does not require Telegram credentials.

- [ ] Prove no stale Vite/Tauri app port or `extractum` process remains.
- [ ] Start the MCP-enabled development app with:

```powershell
npm.cmd run tauri dev
```

Do not use direct `npx tauri dev`. If the sandboxed Vite host exits, use the
repository's documented hidden-host procedure and the actual Vite URL printed
by Vite.

- [ ] With the live Tauri tools, perform in this order:

```text
driver_session start
ipc_get_backend_state
manage_window list
ipc_execute_command command=tg_get_account_statuses args={"accountIds":[]}
```

Require a connected backend/window and a successful serializable command
result. Do not create an account, request a code, sign in, add a source, start
Takeout, or mutate credentials.
- [ ] Stop the driver session and the exact dev processes it started. Prove
ports are free. `.playwright-mcp/` is generated/ignored and must not be staged.

### Task 8D: Release and Bounded Self-Managed Startup

- [ ] Build the release executable without bundling:

```powershell
if (
    -not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR) -or
    -not [string]::IsNullOrWhiteSpace($env:CARGO_BUILD_TARGET)
) {
    throw 'CARGO_TARGET_DIR and CARGO_BUILD_TARGET must be unset for canonical release evidence'
}

$metadataCapture = Invoke-CapturedNative `
    -Label 'locked release target metadata' `
    -Command {
        cargo metadata `
            --manifest-path src-tauri/Cargo.toml `
            --locked `
            --format-version 1 `
            --no-deps
    }
if ($metadataCapture.ExitCode -ne 0) {
    $metadataCapture.Text
    throw 'Locked Cargo metadata failed before the release build'
}
$metadataJsonStart = $metadataCapture.Text.IndexOf('{')
if ($metadataJsonStart -lt 0) {
    throw 'Cargo metadata emitted no JSON object'
}
$metadata = $metadataCapture.Text.Substring($metadataJsonStart) |
    ConvertFrom-Json
$repositoryRoot = (Resolve-Path -LiteralPath '.').Path
$expectedTarget = [IO.Path]::GetFullPath(
    (Join-Path $repositoryRoot 'src-tauri/target')
)
$metadataTarget = [IO.Path]::GetFullPath(
    [string]$metadata.target_directory
)
if (-not [string]::Equals(
    $metadataTarget,
    $expectedTarget,
    [StringComparison]::OrdinalIgnoreCase
)) {
    throw "Cargo metadata target_directory is not canonical src-tauri/target: $metadataTarget"
}

$rustcCapture = Invoke-CapturedNative `
    -Label 'release host target' `
    -Command { rustc -vV }
if ($rustcCapture.ExitCode -ne 0) {
    $rustcCapture.Text
    throw 'rustc host-target discovery failed'
}
$hostMatches = @(
    [regex]::Matches($rustcCapture.Text, '(?m)^host:\s+([^\s]+)\s*$')
)
if ($hostMatches.Count -ne 1) {
    throw 'rustc -vV did not emit exactly one host target'
}
$hostTarget = $hostMatches[0].Groups[1].Value

Invoke-CheckedNative 'Phase 8B release build' {
    # Explicit --target overrides any Cargo [build] target configuration.
    npm.cmd run tauri -- build --no-bundle --target $hostTarget
}
$resolvedExpectedTarget = (Resolve-Path -LiteralPath $expectedTarget).Path
$resolvedMetadataTarget = (Resolve-Path -LiteralPath $metadataTarget).Path
if (-not [string]::Equals(
    $resolvedMetadataTarget,
    $resolvedExpectedTarget,
    [StringComparison]::OrdinalIgnoreCase
)) {
    throw 'Resolved release target_directory escaped canonical src-tauri/target'
}
$releaseExe = (Resolve-Path -LiteralPath (
    Join-Path $resolvedMetadataTarget "$hostTarget/release/extractum.exe"
)).Path
$releaseHash = (Get-FileHash -LiteralPath $releaseExe -Algorithm SHA256).Hash.ToLowerInvariant()
"release_path=$releaseExe release_sha256=$releaseHash"
```

Do not build if either Cargo target override environment variable is present or
metadata resolves a different target directory. The explicit host `--target`
overrides any effective Cargo `[build] target`; resolve only the corresponding
`target/<host>/release/extractum.exe`. Do not resolve or start the executable
unless the wrapped build returned zero. The verification document records the
metadata target, host target, build exit, resolved path, and hash so a stale
pre-existing canonical or cross-target executable cannot become completion
evidence.

- [ ] Run this exact PID/path-bounded smoke only after the live MCP smoke and
port cleanup:

```powershell
$preExisting = @(Get-Process -Name 'extractum' -ErrorAction SilentlyContinue)
if ($preExisting.Count -ne 0) {
    throw "Pre-existing extractum process prevents bounded startup evidence"
}

$started = Start-Process `
    -FilePath $releaseExe `
    -PassThru `
    -WindowStyle Hidden
$startedId = $started.Id
$cleanupDeadline = $null

try {
    $survivalDeadline = [DateTime]::UtcNow.AddSeconds(5)
    while ([DateTime]::UtcNow -lt $survivalDeadline) {
        $started.Refresh()
        if ($started.HasExited) {
            throw "Release executable exited before the five-second survival window"
        }
        Start-Sleep -Milliseconds 200
    }

    $live = Get-Process -Id $startedId -ErrorAction Stop
    $actualPath = (Resolve-Path -LiteralPath $live.Path).Path
    if (-not [String]::Equals(
        $actualPath,
        $releaseExe,
        [StringComparison]::OrdinalIgnoreCase
    )) {
        throw "Startup PID executable path does not match the built release executable"
    }
}
finally {
    $owned = Get-Process -Id $startedId -ErrorAction SilentlyContinue
    if ($null -ne $owned) {
        Stop-Process -Id $startedId -Force
    }
    $cleanupDeadline = [DateTime]::UtcNow.AddSeconds(10)
    while (
        $null -ne (Get-Process -Id $startedId -ErrorAction SilentlyContinue) -and
        [DateTime]::UtcNow -lt $cleanupDeadline
    ) {
        Start-Sleep -Milliseconds 200
    }
    if ($null -ne (Get-Process -Id $startedId -ErrorAction SilentlyContinue)) {
        throw "Owned release PID did not terminate within ten seconds"
    }
}

if (@(Get-Process -Name 'extractum' -ErrorAction SilentlyContinue).Count -ne 0) {
    throw "Unexpected extractum process residue after bounded startup smoke"
}
```

An early exit, path mismatch, cleanup timeout, or residue is a failed
completion gate. Stop only the exact PID started by the script.

### Task 8E: Durable Verification and Final Disposition

- [ ] Write the verification document with:

```text
start SHA and ancestor proof
Checkpoint 1–8 commit SHAs
post-CP1 non-checkpoint authority-maintenance SHAs in chronological order:
  1c0961516c338073ba9578edb18a61c7b1285897
    (`docs: define generated Telegram bridge-use authority`)
  56b9d3995f10f0070f4e2d0f94fb048181218ae3
    (`docs: move Telegram escape-hatch authority to LLM review`)
  reviewed `docs: harden Telegram LLM review gate` correction commit
  post-CP1 TypeScript simplification commit
authority/map hashes and counts
manifest/lock hashes and six-member metadata proof
feature-baseline artifact hash
19-file staging artifact hash and all file records
exact test counts/identity sets
focused/full command exit results
one workspace-check duration per checkpoint
complete cumulative CP2–CP8 LLM review sections, each with one exact CLEAN
  marker, plus an external aggregate outcome table with section/SHA references
live MCP command result (sanitized, no credentials)
release build path/hash
startup PID/path/survival/cleanup evidence
known non-gating lack of credentialed Telegram mutation
```

Never include API hashes, phone numbers, session bytes, login tokens, usernames
from real accounts, or raw credential-bearing diagnostics.

- [ ] With status still `Checkpoint 8 retained`, rerun the focused Telegram and
  shell-cap contracts plus the generated authority checks and
  `npm.cmd run verify`. Inspect/stage the Task 8
  artifact/scripts/contracts/evidence/status diff and commit the separately
  GREEN checkpoint:

```powershell
Invoke-CheckedNative 'CP8 precommit Telegram contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'CP8 precommit shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'CP8 precommit Grammers features' {
    node scripts/telegram-grammers-feature-baseline.mjs --check
}
Invoke-CheckedNative 'CP8 precommit test identities' {
    node scripts/telegram-8b-test-identities.mjs --check
}
Invoke-CheckedNative 'CP8 precommit symbol disposition' {
    node scripts/telegram-8b-symbol-map.mjs --check
}
Invoke-CheckedNative 'CP8 precommit staging hash' {
    node scripts/telegram-staging-sha256.mjs --check
}
Assert-Phase8BFinalTestInventory `
    -AuthorityPath 'src/lib/telegram-8b-test-identities.json' `
    -ExpectedPackageTotal 736
Assert-ExactRustIdentitySet `
    -Package extractum `
    -Prefix 'sources::store::tests::' `
    -Expected $ExactStoreTests
Invoke-CheckedNative 'CP8 precommit repository verify' {
    npm.cmd run verify
}
Invoke-CheckedNative 'CP8 precommit diff check' {
    git diff --check
}
Invoke-CheckedNative 'stage Task 8 checkpoint' {
    git add `
        scripts/telegram-staging-sha256.mjs `
        src/lib/telegram-8b-staging-sha256.json `
        src/lib/telegram-crate-boundary-contract.test.ts `
        src/lib/crate-extraction-shell-cap-contract.test.ts `
        docs/superpowers/verification/2026-07-28-extractum-telegram-8b-preparation.md `
        docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md `
        docs/superpowers/specs/2026-07-17-crate-roadmap.md
}
Invoke-CheckedNative 'CP8 staged diff check' {
    git diff --cached --check
}
Invoke-CheckedNative 'commit Task 8 checkpoint' {
    git commit -m "test: retain Phase 8B Telegram checkpoint 8"
}
```

- [ ] Record that commit SHA in the verification document. Then change only
  the final disposition authority/evidence to:

```text
roadmap: 8B preparation retained; 8C pending
design: Approved; 8B preparation retained; 8C pending
```

- [ ] Run:

```powershell
Invoke-CheckedNative 'terminal Telegram contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'terminal shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'terminal staging hash' {
    node scripts/telegram-staging-sha256.mjs --check
}
Invoke-CheckedNative 'terminal Grammers features' {
    node scripts/telegram-grammers-feature-baseline.mjs --check
}
Invoke-CheckedNative 'terminal test identities' {
    node scripts/telegram-8b-test-identities.mjs --check
}
Invoke-CheckedNative 'terminal symbol disposition' {
    node scripts/telegram-8b-symbol-map.mjs --check
}
Assert-Phase8BFinalTestInventory `
    -AuthorityPath 'src/lib/telegram-8b-test-identities.json' `
    -ExpectedPackageTotal 736
Assert-ExactRustIdentitySet `
    -Package extractum `
    -Prefix 'sources::store::tests::' `
    -Expected $ExactStoreTests
Invoke-CheckedNative 'terminal repository verify' { npm.cmd run verify }
Invoke-CheckedNative 'terminal diff check' { git diff --check }
```

- [ ] Inspect/stage only the final docs disposition and commit:

```powershell
Invoke-CheckedNative 'commit terminal Phase 8B status' {
    git commit -m "docs: retain Phase 8B Telegram preparation"
}
```

- [ ] Prove the worktree is clean, the final commit descends from the recorded
  start, every Checkpoint 1–8 commit and every recorded post-CP1
  authority-maintenance commit is an ancestor, and no ignored/generated browser
  artifact was staged.

## Rollback and Pause Ladder

- [ ] Every checkpoint commit is the only retained rollback boundary. If work
  pauses before a commit, report the last retained checkpoint, dirty allowlist,
  exact failing command, and next checkbox; do not advance status.
- [ ] On a failed candidate gate, restore only the two candidate status lines
  to the preceding retained value with `apply_patch`, keep diagnostic evidence
  outside commits unless the plan names it, and fix forward. Do not use
  `git reset --hard`, `git checkout --`, force deletion, or broad cleanup.
- [ ] If manifest/lock/features drift unexpectedly, stop at Checkpoint 1. If
  live-owned operations need a raw/app type, stop at Checkpoint 3 or 4. If
  Takeout needs SQL/cancellation/event ownership in staging, stop at
  Checkpoint 6. Each case requires a reviewed design/plan amendment.
- [ ] If live MCP, release, or startup evidence fails after Checkpoint 7, keep
  Checkpoint 7 as the truthful retained state; do not claim Checkpoint 8 or
  terminal 8B retention.
- [ ] Phase 8C may begin only from a clean terminal 8B commit, explicit owner
  authorization of a reviewed 8C plan, a GREEN staging hash check, and the
  exact six-member/no-new-crate state retained here.

## Execution Handoff

Plan complete and saved at
`docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md`.
The recommended execution mode is `superpowers:subagent-driven-development`,
one task/checkpoint at a time with specification review followed by code
quality review. `superpowers:executing-plans` is the fallback when a single
worker must execute sequentially. Neither mode is authorized until the owner
explicitly approves Phase 8B execution and the bounded API clarification.
