# Extractum Telegram Phase 8A Preparation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the separately green Phase 8A contract, DTO/media, session/secret, and opaque-runtime preparation inside the `extractum` package without creating `extractum-telegram`, changing the package graph, or changing observable Telegram/Takeout behavior.

**Architecture:** The application remains the only package and continues to own Tauri commands/events, SQL, generic secure storage, filesystem paths and writes, source persistence, Takeout jobs, and all four direct Grammers roots. Phase 8A splits future-owner `dto`, `media`, `session`, and `runtime` leaf modules under the existing private `crate::telegram` facade, keeps temporary raw capabilities crate-private, and leaves their physical move into the complete portable `telegram_impl` staging tree plus all live-source/Takeout consumer cleanup to separately planned Phase 8B.

**Tech Stack:** Rust 2021, Tauri, Tokio, SQLx/SQLite, Grammers at pinned revision `1f901ce6e973fdcf0e74267f3d8efad5c729daaa`, `extractum-core`, `secrecy` 0.8, XChaCha20-Poly1305, Vitest/TypeScript source contracts, PowerShell on Windows.

**Authority:** [`2026-07-26-telegram-crate-boundary-design.md`](../specs/2026-07-26-telegram-crate-boundary-design.md), [`2026-07-17-crate-roadmap.md`](../specs/2026-07-17-crate-roadmap.md), and [`2026-07-17-focused-rust-loop-design.md`](../specs/2026-07-17-focused-rust-loop-design.md).

**Bounded evidence correction requiring plan approval:** planning found `src-tauri/src/sources/store.rs` as one app-owned dependent consumer of the raw-client seam. It is outside the immutable 19-path ownership/move map and adds no test identity, but the approved design's “complete move-and-touch” wording must name it. Explicit owner approval of this plan also approves only that evidence correction. Task 0 synchronizes it in a docs-only commit before any Phase 8A production or test code changes.

## Global Constraints

- [ ] Start at or after clean approval commit `9f56a4584b0c2abb331c1b2ab7f198ccb89db042`. Record the actual starting `HEAD`; never reset or check out the repository back to that commit.
- [ ] Do not execute this plan until the owner reviews and explicitly authorizes this implementation plan. Approval of the boundary design and creation of this plan are not execution authority.
- [ ] Execution starts only from a clean commit that already tracks this exact owner-approved plan. `git ls-files --error-unmatch docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md` must succeed; an untracked or dirty plan stops execution.
- [ ] Before every commit, run `git status --short`, inspect the complete scoped diff, and stage only the files named by that task. Preserve unrelated user changes.
- [ ] Use the canonical shared `src-tauri/target`. Do not create a worktree, alternate target, timing harness, process scanner, quiet-window rule, retry loop, or build-profile change.
- [ ] Phase 8A must leave `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` byte-identical. It creates no package, workspace member, path dependency, dependency promotion, feature-baseline file, or `src-tauri/crates/extractum-telegram` directory.
- [ ] All four direct Grammers roots remain application dependencies. No source or contract may claim the Phase 8 dependency-removal outcome before 8C.
- [ ] Do not edit a migration, schema registry, value registry, frontend runtime/UI source, command registration, IPC signature, event name/payload, status string, error kind/message/JSON, secret identifier, session path, envelope byte/string contract, transaction boundary, cancellation point, message order, limit, or fallback rule. Test-only contract files under `src/lib/*.test.ts` and their test helpers are explicitly allowed only where a task names them.
- [ ] Do not introduce a repository, service locator, SQL port, event bus, generic Telegram operation trait, public raw handle, public `TelegramError`, app mirror DTO, conversion-only DTO, second media-payload owner, or second Telegram item-kind literal owner.
- [ ] Public fallible Telegram seam operations return `extractum_core::error::AppResult<T>` directly. They never expose `grammers_*`, raw `tl`, SQLx, Tauri, keyring, `RemoteCall`, or `InvocationError`.
- [ ] `TelegramApiHash` and `SessionEncryptionKey` use `secrecy` containers, have no `Debug` implementation that can reveal a secret, no public secret-bearing field, and no borrowed plaintext getter.
- [ ] The first retained code checkpoint changes exactly the six identity-seam `crate::error` paths named below. It must not normalize the other 44 known `crate::{error,compression,time}` facade references. The preceding Task 0 authority synchronization is docs-only.
- [ ] Test accounting uses five distinct metrics and never calls all of them “final”: 140 immutable baseline primaries; 141 baseline-derived identities present after 8A (140 primaries plus the item-kind companion); 158 tracked identities after 8A (those 141 plus 17 additional verification tests); 143 eventual baseline-derived identities after all three declared companions exist; and 160 eventual tracked identities if all 17 additional verification tests remain. Drift in any metric requires a plan amendment and renewed review; it is not repaired during execution.
- [ ] Every exact RED is a runtime RED. Add the minimum compiling signature and dummy behavior first; a compile failure or a zero-test filter is not evidence.
- [ ] Every preparation checkpoint is independently useful, package-GREEN, separately committed, and may be retained on pause. Never batch two checkpoints into one commit.
- [ ] Only Task 6 runs the full repository completion gates and records one already-required workspace-check duration. Timing is advisory and cannot fail or revert the slice.
- [ ] Release build, startup smoke, and live credentialed Telegram requests are not Phase 8A gates; the approved design first requires release/startup evidence in 8B.
- [ ] If implementation discovers another public type, operation, dependency, mixed test subject, transaction owner, wire change, or app-only capability crossing the seam, stop and amend the approved design and this plan before changing code.

### Phase 8A Roadmap and Design Status State Machine

The Phase 8 roadmap heading, the design status line, and the exact expectation/allowlist in `src/lib/crate-extraction-shell-cap-contract.test.ts` move in the same commit:

1. Roadmap `design approved; implementation not started`; design `Approved; implementation not started` — starting state.
2. Roadmap `8A preparation Checkpoint 1 retained` through `8A preparation Checkpoint 5 retained`; design `Approved; 8A preparation Checkpoint N retained` — only after the named checkpoint is GREEN and the truthful status update is included in that same checkpoint commit.
3. Roadmap `8A preparation retained`; design `Approved; 8A preparation retained; 8B not started` — only after Task 6 evidence and all completion gates pass.
4. `8B preparation retained; 8C pending`, `done: retained`, and `not retained` remain allowed future/rollback states but are not produced by this plan.

Task 1 installs this closed vocabulary before first changing the status. At a pause, leave the exact last committed checkpoint state. Dirty work never advances status.

## Rust Verification Loops

The only affected package in Phase 8A is `extractum`. `extractum-core` is an unchanged dependency-contract subject. `extractum-telegram` must not exist.

Bootstrap these fail-closed helpers at the start of every execution session:

```powershell
$ErrorActionPreference = 'Stop'

function Invoke-CheckedNative {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )
    & $Command
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) { throw "$Label failed with exit code $exitCode" }
}

function Invoke-ExactRustTest {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$TestName
    )
    $output = & cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib $TestName -- --exact 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | Out-String)
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
        [Parameter(Mandatory = $true)][string]$ExpectedPattern
    )
    $leaf = ($TestName -split '::')[-1]
    if (-not (Select-String -LiteralPath $SourcePath -SimpleMatch $leaf -Quiet)) {
        throw "RED source does not contain exact test leaf: $TestName"
    }
    $output = & cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib $TestName -- --exact 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | Out-String)
    $text
    if ($exitCode -eq 0) { throw "Expected exact Rust runtime RED: $Package::$TestName" }
    if ($text -notmatch '(?m)^[ \t]*running 1 test[ \t]*\r?$') {
        throw "Exact Rust RED did not execute exactly one test: $Package::$TestName"
    }
    if ($text -match 'could not compile|error\[E[0-9]+\]') {
        throw "Exact Rust RED was a compile failure: $Package::$TestName"
    }
    if ($text -notmatch $ExpectedPattern) {
        throw "Unexpected Rust runtime RED reason: $Package::$TestName"
    }
}

function Invoke-NonEmptyRustSuite {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$TestFilter
    )
    $output = & cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib $TestFilter -- --nocapture 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | Out-String)
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

function Invoke-NonEmptyRustTestList {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$Package,
        [string]$TestFilter = ''
    )
    $arguments = @('test', '--color', 'never', '--manifest-path', 'src-tauri/Cargo.toml', '-p', $Package, '--lib')
    if ($TestFilter) { $arguments += $TestFilter }
    $arguments += @('--', '--list')
    $output = & cargo @arguments 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | Out-String)
    $text
    if ($exitCode -ne 0) { throw "$Label failed with exit code $exitCode" }
    $listedCounts = @(
        [regex]::Matches($text, '(?m)^[ \t]*([0-9]+) tests?,[ \t]+[0-9]+ benchmarks?[ \t]*\r?$') |
            ForEach-Object { [int]$_.Groups[1].Value }
    )
    if ($listedCounts.Count -eq 0 -or ($listedCounts | Measure-Object -Sum).Sum -le 0) {
        throw "$Label listed zero tests or produced no recognized libtest count"
    }
}

function Assert-ExactRustIdentitySet {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$Prefix,
        [Parameter(Mandatory = $true)][string[]]$Expected
    )
    $expectedSorted = @($Expected | Sort-Object -Unique)
    if ($expectedSorted.Count -ne $Expected.Count) {
        throw "Expected identity set contains duplicates for $Prefix"
    }
    if (@($expectedSorted | Where-Object { -not $_.StartsWith($Prefix) }).Count -ne 0) {
        throw "Expected identity lies outside prefix $Prefix"
    }
    $output = & cargo test --color never --manifest-path src-tauri/Cargo.toml -p $Package --lib -- --list 2>&1
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) { $output; throw "Cargo test list failed for $Package::$Prefix" }
    $listed = @($output | ForEach-Object {
        if ($_ -match '^([^ ]+): test$') { $Matches[1] }
    })
    $actualSorted = @($listed | Where-Object { $_.StartsWith($Prefix) } | Sort-Object -Unique)
    if (($actualSorted -join "`n") -ne ($expectedSorted -join "`n")) {
        Compare-Object $expectedSorted $actualSorted
        throw "Exact Rust identity set drifted for $Package::$Prefix"
    }
}

$sourcesStoreBroadRegressionTestIds = @(
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

function Assert-NoMatches {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )
    $output = & $Command 2>&1
    $exitCode = $LASTEXITCODE
    $output | Out-Host
    if ($exitCode -eq 0) { throw "$Label found forbidden matches" }
    if ($exitCode -ne 1) { throw "$Label scan failed with exit code $exitCode" }
}

function Get-ChangedPaths {
    $unstaged = @(& git diff --name-only --relative)
    if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate unstaged paths' }
    $cached = @(& git diff --cached --name-only --relative)
    if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate cached paths' }
    $untracked = @(& git ls-files --others --exclude-standard)
    if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate untracked paths' }
    @($unstaged + $cached + $untracked | Where-Object { $_ } | Sort-Object -Unique)
}

function Assert-CleanWorktree {
    param([Parameter(Mandatory = $true)][string]$Label)
    $changed = @(Get-ChangedPaths)
    if ($changed.Count -ne 0) {
        throw "$Label requires a clean worktree; found: $($changed -join ', ')"
    }
}

function Assert-ScopedChanges {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string[]]$Allowed,
        [switch]$RequireChanges
    )
    $changed = @(Get-ChangedPaths)
    $unexpected = @($changed | Where-Object { $_ -notin $Allowed })
    if ($unexpected.Count -ne 0) {
        throw "$Label has out-of-scope paths: $($unexpected -join ', ')"
    }
    if ($RequireChanges -and $changed.Count -eq 0) {
        throw "$Label produced no changed paths"
    }
    $changed
}

function Get-CargoIdentityHashes {
    [pscustomobject]@{
        Manifest = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.toml').Hash
        Lock = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.lock').Hash
    }
}

function Commit-ScopedCheckpoint {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string[]]$Allowed,
        [Parameter(Mandatory = $true)][object]$StartingCargoHashes,
        [Parameter(Mandatory = $true)][string]$Message
    )
    $sortedAllowed = @($Allowed | Sort-Object -Unique)
    $currentCargoHashes = Get-CargoIdentityHashes
    if ($currentCargoHashes.Manifest -ne $StartingCargoHashes.Manifest) {
        throw "$Label changed src-tauri/Cargo.toml"
    }
    if ($currentCargoHashes.Lock -ne $StartingCargoHashes.Lock) {
        throw "$Label changed src-tauri/Cargo.lock"
    }
    Assert-ScopedChanges -Label "$Label scope" -Allowed $sortedAllowed -RequireChanges
    Invoke-CheckedNative "$Label unstaged diff check" { git diff --check }
    Invoke-CheckedNative "$Label stage exact paths" { git add -- $sortedAllowed }
    $cached = @(& git diff --cached --name-only --relative | Sort-Object -Unique)
    if ($LASTEXITCODE -ne 0) { throw "$Label could not enumerate staged paths" }
    if (($cached -join "`n") -ne ($sortedAllowed -join "`n")) {
        throw "$Label staged unexpected paths: $($cached -join ', ')"
    }
    Invoke-CheckedNative "$Label cached diff check" { git diff --cached --check }
    Invoke-CheckedNative "$Label commit" { git commit -m $Message }
    Assert-CleanWorktree "$Label post-commit"
}

function Commit-ScopedFix {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string[]]$OwningAllowed,
        [Parameter(Mandatory = $true)][object]$StartingCargoHashes,
        [Parameter(Mandatory = $true)][string]$Message
    )
    $sortedOwningAllowed = @($OwningAllowed | Sort-Object -Unique)
    $changed = @(Get-ChangedPaths | Sort-Object -Unique)
    if ($changed.Count -eq 0) { throw "$Label produced no changed paths" }
    $unexpected = @($changed | Where-Object { $_ -notin $sortedOwningAllowed })
    if ($unexpected.Count -ne 0) {
        throw "$Label has out-of-owner paths: $($unexpected -join ', ')"
    }
    if ($changed.Count -ge $sortedOwningAllowed.Count) {
        throw "$Label is not a strict subset of its owning checkpoint scope"
    }
    $currentCargoHashes = Get-CargoIdentityHashes
    if ($currentCargoHashes.Manifest -ne $StartingCargoHashes.Manifest) {
        throw "$Label changed src-tauri/Cargo.toml"
    }
    if ($currentCargoHashes.Lock -ne $StartingCargoHashes.Lock) {
        throw "$Label changed src-tauri/Cargo.lock"
    }
    Invoke-CheckedNative "$Label unstaged diff check" { git diff --check } | Out-Host
    Invoke-CheckedNative "$Label stage exact changed paths" { git add -- $changed } | Out-Host
    $cached = @(& git diff --cached --name-only --relative | Sort-Object -Unique)
    if ($LASTEXITCODE -ne 0) { throw "$Label could not enumerate staged paths" }
    if (($cached -join "`n") -ne ($changed -join "`n")) {
        throw "$Label staged unexpected paths: $($cached -join ', ')"
    }
    Invoke-CheckedNative "$Label cached diff check" { git diff --cached --check } | Out-Host
    Invoke-CheckedNative "$Label commit" { git commit -m $Message } | Out-Host
    Assert-CleanWorktree "$Label post-commit"
    $sha = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw "$Label could not record its SHA" }
    [pscustomobject]@{ Sha = $sha; Paths = $changed }
}
```

Every `Assert-ExactRustRuntimeRed` invocation assumes the minimum compiling types, signatures, dummy behavior, and test double for that case already exist. Process one runtime RED/GREEN case at a time; step numbering never authorizes testing an undefined API.

After each small Rust change:

```powershell
Invoke-ExactRustTest -Package extractum -TestName 'sources::types::tests::telegram_message_identity_validation_rejects_invalid_values'
Invoke-CheckedNative 'focused app check' {
    cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
```

At each retained checkpoint:

```powershell
Invoke-CheckedNative 'format checkpoint' { cargo fmt --manifest-path src-tauri/Cargo.toml --all }
Invoke-CheckedNative 'app checkpoint check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'app checkpoint tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

Task 6 completion gates are exactly:

```powershell
Invoke-CheckedNative 'rustfmt gate' { npm.cmd run check:rustfmt }
Invoke-CheckedNative 'locked metadata gate' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
}
$workspaceCheck = Measure-Command {
    Invoke-CheckedNative 'workspace check' {
        cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
    }
}
"Phase 8A ordinary workspace check: $([math]::Round($workspaceCheck.TotalMilliseconds)) ms"
Invoke-CheckedNative 'workspace tests' {
    cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
}
Invoke-CheckedNative 'repository verify' { npm.cmd run verify }
```

Record only this already-required workspace-check duration. Do not repeat it for measurement.

## Frozen Observable Compatibility

Task 2 converts this table into standing source/serialization assertions before any seam changes.

| Subject | Frozen Phase 8A contract |
| --- | --- |
| Tauri commands | Exact declarations and `generate_handler!` registration for `list_accounts`, `get_account`, `create_account`, `set_account_phone`, `clear_account_phone`, `delete_account`, `tg_init`, `tg_is_authenticated`, `tg_get_account_statuses`, `tg_send_code`, `tg_sign_in`, and `tg_logout` |
| Telegram events | `telegram://restore-failure`, payload `{ message }`; `telegram://account-status`, payload `{ account_id, status, message }`; current best-effort emit behavior and emit ordering |
| Takeout event | `sources://takeout-import`; current payload fields, serde casing, emission ordering, and best-effort failure behavior |
| Telegram statuses | `not_initialized`, `restoring`, `ready`, `reauth_required`, `restore_failed` |
| Takeout statuses | `queued`, `running`, `cancel_requested`, `failed`, `cancelled`, `completed` |
| Login results/errors | `tg_send_code -> "Code sent"`; `tg_sign_in -> true`; `tg_logout -> true`; exact current auth/network error kinds, messages, and `{"kind","message"}` JSON |
| Secret identifiers | service `org.ai.extractum`; `telegram.account.<account_id>.api_hash`; `telegram.account.<account_id>.session_key` |
| Session path | app-data `telegram_<account_id>.session.json`; temporary path produced by `with_extension("session.json.tmp")`; write then rename; deletion order unchanged |
| Session crypto | version `1`; algorithm `XChaCha20-Poly1305`; 32-byte key; 24-byte nonce; URL-safe base64 without padding; AAD `org.ai.extractum.telegram.session.v1.account.<account_id>` |
| Session compatibility | exact canonical envelope field names/order produced by current serde struct; encrypted and legacy inputs; wrong-account, missing-key, invalid-key, malformed-envelope, encrypt/decrypt, secret-write, file-write, rename, and deletion behavior |
| Source/Takeout flow | current message ordering, page/range limits, fallback identity, partial progress, fetch/persist boundary, cancellation checks, warning/provenance order, and terminal finalization |
| Errors | direct `extractum_core::error::AppResult`; exact current kind/message/JSON; no public `TelegramError`; protocol retry/fallback classifiers remain private |

The source contract hashes or parses exact current command signatures rather than merely counting names. Characterization tests pin byte/string output and ordering; a source scan alone is insufficient.

## Frozen Phase 8A Public and Internal API

Phase 8A creates these future-owner leaf modules under the existing application facade and no `telegram_impl` path:

```text
src-tauri/src/telegram/
  dto.rs
  media.rs
  runtime.rs
  session.rs
```

`src-tauri/src/telegram.rs` declares them privately and exposes an explicit curated app-only allowlist:

```rust
mod dto;
mod media;
mod runtime;
mod session;

pub(crate) use dto::{
    TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity,
    ITEM_KIND_TELEGRAM_MESSAGE, TELEGRAM_PEER_KIND_CHANNEL,
    TELEGRAM_PEER_KIND_CHAT, TELEGRAM_PEER_KIND_USER,
};
pub(crate) use media::{
    derive_content_kind, derive_document_media_kind, extract_item_payload, DocumentSignals,
    TelegramMediaPayload, CONTENT_KIND_TEXT_ONLY, CONTENT_KIND_TEXT_WITH_MEDIA,
};
pub(crate) use runtime::{
    TelegramApiHash, TelegramClientHandle, TelegramRuntime, TelegramRuntimeStatus,
};
pub(crate) use session::{
    decode_session_json, encode_session_json, session_json_requires_existing_key,
    SessionEncryptionKey, TelegramSession,
};
```

`TelegramLoginAttempt` remains `pub` with private fields in `runtime.rs` for the approved eventual crate API, but 8A does not re-export it through the app facade because no app consumer may hold it. The `crate::media` compatibility facade re-exports from the list above only the symbols its current consumers require and retains its existing provider-neutral `extractum-core` re-exports. It defines no duplicate value or wrapper.

Existing app consumers use the exact `crate::telegram::` facade or the existing private `crate::media` compatibility facade. Phase 8B mechanically moves these four leaf files into `src-tauri/src/telegram_impl/**`, installs the approved `crate::telegram_impl::` consumer prefix, and completes the rest of the staging tree. Phase 8A must not create `src-tauri/src/telegram_impl`, `error.rs`, `live/**`, or `takeout/**`.

The existing facade contains an explicit curated `pub(crate) use` list for app consumers and no glob. Types are declared `pub` only where the approved final cross-crate API requires it. The exact Phase 8A API is:

Future-owner leaf-to-leaf imports use only `super::` relative paths from birth; they never import through `crate::telegram`. App-owned consumers outside `src-tauri/src/telegram/**` use the facade. This keeps the leaf content portable without prematurely creating the 8B staging root.

```rust
pub const ITEM_KIND_TELEGRAM_MESSAGE: &str = "telegram_message";
pub(crate) const TELEGRAM_PEER_KIND_CHANNEL: &str = "channel";
pub(crate) const TELEGRAM_PEER_KIND_CHAT: &str = "chat";
pub(crate) const TELEGRAM_PEER_KIND_USER: &str = "user";

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TelegramItemContext {
    pub reply_to_msg_id: Option<i64>,
    pub reply_to_peer_kind: Option<String>,
    pub reply_to_peer_id: Option<String>,
    pub reply_to_top_id: Option<i64>,
    pub reaction_count: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TelegramMediaPayload {
    pub kind: String,
    pub metadata: extractum_core::media_metadata::ItemMediaMetadata,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone)]
pub struct TelegramApiHash(secrecy::SecretString);

impl TelegramApiHash {
    pub fn new(value: secrecy::SecretString) -> Self;
}

#[derive(Clone)]
pub struct SessionEncryptionKey(
    std::sync::Arc<secrecy::SecretVec<u8>>,
);

impl SessionEncryptionKey {
    pub fn try_from_encoded(
        encoded: secrecy::SecretString,
    ) -> extractum_core::error::AppResult<Self>;

    pub fn generate() -> (Self, secrecy::SecretString);
}

#[derive(Clone)]
pub struct TelegramSession {
    inner: std::sync::Arc<grammers_session::storages::MemorySession>,
}

impl TelegramSession {
    pub fn empty() -> Self;

    pub(super) fn raw_memory_session(
        &self,
    ) -> &std::sync::Arc<grammers_session::storages::MemorySession>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramRuntimeStatus {
    Ready,
    ReauthRequired,
}

#[derive(Clone)]
enum TelegramClientInner {
    Grammers(grammers_client::Client),
    #[cfg(test)]
    Test { account_id: i64 },
}

#[derive(Clone)]
pub struct TelegramClientHandle {
    client: TelegramClientInner,
    session: TelegramSession,
}

impl TelegramClientHandle {
    pub(crate) fn raw_client(&self) -> &grammers_client::Client;

    pub(crate) fn raw_session(
        &self,
    ) -> &std::sync::Arc<grammers_session::storages::MemorySession>;
}

enum TelegramLoginAttemptToken {
    Grammers(grammers_client::client::LoginToken),
    #[cfg(test)]
    Test(u64),
}

pub struct TelegramLoginAttempt {
    token: TelegramLoginAttemptToken,
    phone: String,
}

struct TelegramRuntimeAccount {
    handle: TelegramClientHandle,
    api_hash: TelegramApiHash,
    login_attempt: Option<TelegramLoginAttempt>,
    runner: tokio::task::JoinHandle<()>,
}

pub struct TelegramRuntime {
    accounts: tokio::sync::Mutex<
        std::collections::HashMap<i64, TelegramRuntimeAccount>,
    >,
    #[cfg(test)]
    test_callbacks: Option<std::sync::Arc<TelegramRuntimeTestCallbacks>>,
}

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

    pub async fn authorized_client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle>;

    pub(super) async fn initialized_client(
        &self,
        account_id: i64,
    ) -> extractum_core::error::AppResult<TelegramClientHandle>;

    pub async fn clear_account(
        &self,
        account_id: i64,
        sign_out: bool,
    );
}
```

The private `Arc` in `SessionEncryptionKey` is required by the pinned
`secrecy` 0.8 API: `SecretVec<u8>` itself is not `Clone` because that release
does not implement `CloneableSecret` for `u8`. Sharing the secret container
preserves the frozen public `Clone` capability without adding a plaintext
exposure or secret copy; the key bytes remain zeroized when the last clone is
dropped.

The two private enums above are the exact safe test seam: Grammers owns `LoginToken`'s fields and exposes no safe constructor, so tests use `Test(u64)` without fabricating or transmuting a dependency type. `TelegramClientHandle::raw_client` returns the Grammers variant and treats the test variant as unreachable; runtime unit tests never call a raw adapter. Neither enum is public or re-exported, and production builds contain only the Grammers variants.

The exact private test-only callback interface in `runtime.rs` is:

```rust
#[cfg(test)]
type TelegramRuntimeTestFuture<T> = std::pin::Pin<
    Box<
        dyn std::future::Future<
                Output = extractum_core::error::AppResult<T>,
            > + Send
            + 'static,
    >,
>;

#[cfg(test)]
#[derive(Clone)]
struct TelegramRuntimeTestCallbacks {
    is_authorized: std::sync::Arc<
        dyn Fn(i64) -> TelegramRuntimeTestFuture<bool> + Send + Sync,
    >,
    request_login_code: std::sync::Arc<
        dyn Fn(i64, String) -> TelegramRuntimeTestFuture<u64> + Send + Sync,
    >,
    sign_in: std::sync::Arc<
        dyn Fn(i64, u64, String, String) -> TelegramRuntimeTestFuture<()>
            + Send
            + Sync,
    >,
    sign_out: std::sync::Arc<
        dyn Fn(i64) -> TelegramRuntimeTestFuture<()> + Send + Sync,
    >,
    on_runner_drop: std::sync::Arc<dyn Fn(i64) + Send + Sync>,
}

#[cfg(test)]
struct TelegramRuntimeTestRunnerDropProbe {
    account_id: i64,
    on_drop: std::sync::Arc<dyn Fn(i64) + Send + Sync>,
}

#[cfg(test)]
impl Drop for TelegramRuntimeTestRunnerDropProbe {
    fn drop(&mut self) {
        (self.on_drop)(self.account_id);
    }
}

#[cfg(test)]
impl TelegramRuntimeTestCallbacks {
    fn spawn_pending_runner(&self, account_id: i64) -> tokio::task::JoinHandle<()> {
        let probe = TelegramRuntimeTestRunnerDropProbe {
            account_id,
            on_drop: std::sync::Arc::clone(&self.on_runner_drop),
        };
        tokio::spawn(async move {
            let _probe = probe;
            std::future::pending::<()>().await;
        })
    }
}

#[cfg(test)]
impl TelegramRuntime {
    fn with_test_callbacks(callbacks: TelegramRuntimeTestCallbacks) -> Self;
}
```

Production methods call Grammers directly. Only `cfg(test)` dispatches a `Test` handle/token to these callbacks. Each fake initialization creates its required runner only through `spawn_pending_runner`; its drop probe fires because the spawned future is actually dropped by `JoinHandle::abort`, not because production/test code manually reports an abort. Dropping a `JoinHandle` without abort detaches the still-pending task and does not fire the probe during the assertion window. Tests coordinate pending operations with `tokio::sync::oneshot` or `Semaphore`, never sleeps, and assert callback/order ledgers before releasing each operation.

Phase 8A preserves the current concurrency and linearization behavior exactly:

- the single `accounts` mutex serializes guarded operations globally, including different account IDs;
- the app loads/decodes the session before entering the runtime; the runtime starts the runner and awaits authorization without the accounts lock; `accounts.insert` is its publication point, the last insertion wins, and replacing an entry drops—without aborting—the replaced runner; each call writes its returned status after insertion, so the last status write is not atomically tied to the map winner;
- authorization failure before insertion drops—without aborting—the newly spawned runner handle; the app facade then removes and aborts whichever entry is in the map when failure handling runs, including an entry inserted by a concurrent initialization;
- `request_login_code` holds the accounts lock from lookup through the network await and successful attempt replacement; failure retains the previous attempt;
- `sign_in` holds the accounts lock through the network await; failure retains the attempt, while success clears it and clones the session before unlocking; session persistence and the `ready` status happen after unlock;
- `clear_account` removes the entry and, while still holding the accounts lock, awaits best-effort sign-out and then aborts its runner; file deletion and `not_initialized` status happen after unlock; a missing entry is a no-op at the runtime layer;
- `authorized_client` clones the handle under the lock, releases it, and only then performs the authorization check.

Consequently, queued code requests run in lock order and the later successful request replaces the attempt; clear waits for an in-flight code request or sign-in; code queued after clear observes `Account not initialized`; and an initialization prepared concurrently with a code request may insert afterward and discard that attempt. Runtime-map linearization does not make the later file persistence or status/event work atomic with the map.

The app-owned root facade preserves—rather than retains by name—the two current lookup semantics through these exact temporary signatures:

```rust
pub(crate) async fn get_client(
    state: &TelegramState,
    account_id: i64,
) -> extractum_core::error::AppResult<TelegramClientHandle>;

pub(crate) async fn get_authorized_client(
    state: &TelegramState,
    account_id: i64,
) -> extractum_core::error::AppResult<TelegramClientHandle>;
```

Current `get_client(&HashMap<i64, AccountClient>, account_id) -> &Client` is replaced, not merely moved: the Phase 8A form delegates to `TelegramRuntime::initialized_client`, owns its single lock internally, returns an owned opaque handle, and intentionally narrows `pub` to `pub(crate)` because the containing `telegram` module is already application-private and has no external crate consumer. It performs no `is_authorized` call and preserves exact missing-account text `Account {account_id} not initialized`. Current `get_authorized_runtime -> AuthorizedTelegramRuntime` is replaced by `get_authorized_client -> TelegramClientHandle`; it delegates to the public runtime operation and preserves both the same missing-account text and `Account {account_id} is not authenticated`. `AuthorizedTelegramRuntime` and `get_authorized_runtime` must be absent after Checkpoint 5. These app-only free functions are not final crate API and disappear in 8B after their consumers become owned operations.

The session codec and app adapter use this exact split:

```rust
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
```

`session_json_requires_existing_key` parses the format before any secure-store lookup: `true` means encrypted and `false` means legacy. `decode_session_json` reparses inside the codec; a legacy input accepts `None`, while encrypted input with no key returns the exact current missing-key Auth error. This two-call seam preserves malformed-input-before-key-lookup ordering without exposing envelope structs. Returning the encoded secret only from `generate` is a write-only persistence hand-off; neither wrapper has a borrowed/raw getter or implements `ExposeSecret`.

The owning private module may import `secrecy::ExposeSecret` and expose a contained value only inside the narrow encryption/decryption or Grammers-construction statement that consumes it. The exposed borrow is never returned, stored outside the opaque owner, formatted, logged, serialized as plaintext, or made reachable through a field/getter/trait implementation. Source contracts distinguish this required private internal use from forbidden public exposure.

The only app-visible raw adapters are `TelegramClientHandle::raw_client` and `TelegramClientHandle::raw_session` with the exact signatures above. `raw_session` is not dead code in 8A: `takeout_import/mod.rs` is its sole path consumer, with exactly three workflow sites—export-DC spike, migrated history, and current history—where the cloned session remains an input to export-DC alias preparation/transport. The baseline `#[allow(dead_code)]` on `AuthorizedTelegramRuntime.session` is a redundant legacy annotation despite those three reads: the lifecycle-gated contract permits that exact field annotation only through Checkpoint 4, then Checkpoint 5 removes it with `AuthorizedTelegramRuntime`. From Checkpoint 5 onward, neither new handle adapter may carry `#[allow(dead_code)]`. `TelegramSession::raw_memory_session` is `pub(super)` leaf-internal and exists only so the sibling runtime can initialize Grammers. There is no raw constructor, consuming conversion, mutable accessor, public raw accessor, or raw type re-export. Phase 8B must remove both `pub(crate)` handle adapters and both app-only lookup functions after their consumers become owned operations.

`TelegramMessageIdentity::validate()` preserves exact branch order, `AppErrorKind::Validation`, messages, and JSON:

1. `Unsupported Telegram history peer kind '{}'`;
2. `Telegram history peer id must be positive`;
3. `Telegram message id must be positive`.

The unsupported value remains verbatim inside the existing single quotes. The first failing branch wins.

The remainder of the approved existing-symbol visibility allowlist is frozen but deferred to 8B: `ResolvedTelegramSource -> PeerDescriptor` with exactly public `external_id`, `title`, `source_subtype`, `is_member`, `username`, `access_hash`, and `avatar_bytes`; and `ForumTopicSnapshot` with exactly public `topic_id`, `top_message_id`, `title`, `icon_color`, `icon_emoji_id`, `is_closed`, `is_pinned`, `is_hidden`, and `sort_order`. Phase 8A does not widen or introduce either type. `DocumentSignals`, content/media helpers/constants, envelope structs, protocol classifiers, raw Takeout/TL types, and every test helper remain private.

## Frozen 19-File Symbol Disposition

This is the symbol-level authority for Phase 8A and the input to the later 8B plan. “App” means the symbol remains physically and semantically application-owned; “stage” means the named final-owner symbol is introduced/moved into the exact Phase 8A staging file.

| Current path | Stage in 8A | Remain app-owned in 8A |
| --- | --- | --- |
| `telegram.rs` | `AccountClient` internals become `TelegramRuntime`; raw `Client`/session clone and `AuthorizedTelegramRuntime` are eliminated in favor of private-field `TelegramClientHandle`; `LoginToken` becomes private `TelegramLoginAttempt`; runtime init/auth/code/sign-in/clear/client lookup implementations move to `telegram/runtime.rs` | `AccountCredentials`, `AccountRuntimeStatus`, `RestoreFailureEvent`, `TelegramState` facade/status map, credential SQL/secret resolution, `telegram_api_id`, `set_account_status`, restore loop, all six `tg_*` commands, event emission; temporary `get_authorized_client` replaces `get_authorized_runtime`, while temporary `get_client` changes from caller-locked `&HashMap -> &Client` to internally locked `&TelegramState -> TelegramClientHandle` and narrows `pub` to `pub(crate)`; both temporary lookups disappear in 8B |
| `telegram_session_store.rs` | `SavedSession`, `EncryptedSessionEnvelope`, AAD/base64/encrypt/decrypt, MemorySession conversion, and codec-subject tests move to `telegram/session.rs`; public opaque `TelegramSession` and secret key wrapper are introduced there | `session_path`, `session_exists`, secret-store reads/writes/deletes, temp path, filesystem write/rename/delete, legacy migration coordination, and adapter-subject tests |
| `media.rs` | `ExtractedMediaPayload -> TelegramMediaPayload`; `ExtractedItemPayload` fields merge into `TelegramMessageDraft`; `DocumentSignals`, content constants/classifiers, Grammers media conversion, and two classification tests move to `telegram/media.rs` | private compatibility re-exports only; provider-neutral metadata remains owned by `extractum-core` |
| `sources/types.rs` | `TelegramMessageIdentity`, `TELEGRAM_PEER_KIND_{CHANNEL,CHAT,USER}`, and `ITEM_KIND_TELEGRAM_MESSAGE` move to `telegram/dto.rs`; the identity test moves and the Telegram wire assertion becomes a companion test | every other constant/type/test; the baseline item-kind test retains only YouTube assertions |
| `sources/items.rs` | `SourceItemInsert -> TelegramMessageDraft`; `TelegramItemContext` moves to `dto.rs`; `reply_peer_context` is assigned to later `live/messages.rs`; `ExtractedItemPayload` disappears | all SQL/transaction/read-model code, `PreparedSourceItem`, insert outcomes/context, list/upsert functions, and app tests; dead `insert_source_item` and draft `external_id` are removed |
| `sources/peer_resolution.rs` | raw peer conversion and remote lookup symbols are assigned to later `live/peer.rs`: `resolve_telegram_source_by_username`, `resolve_telegram_source_from_dialogs`, `resolved_telegram_source_from_peer`, `telegram_group_kind`, `telegram_group_is_member`, `peer_access_hash`, raw peer-ref constructors, and `ResolvedTelegramSource -> PeerDescriptor` | planning, typed DB identity loading, metadata decode, cache coordination, `resolve_and_refresh_peer`, and app-owned tests remain until the 8B seam is implemented |
| `sources/identity.rs` | Grammers conversion in `TelegramSourceIdentity::peer_ref` is assigned to later `live/peer.rs` | DB rows, `SourceIdentity`, `TelegramPeerKind`, `TelegramResolutionStrategy`, `TelegramSourceIdentity`, normalization, SQL loads, readiness policy, and app tests |
| `sources/sync.rs` | live fetch/message adaptation and `fallback_message_identity` are assigned to later `live/messages.rs` | provider selection, locks/settings, persistence, `finalize_sync`, command wrapper, and app tests |
| `sources/topics.rs` | `ForumTopicSnapshot`, remote page retrieval/mapping, and message-date/cursor protocol logic are assigned to later `live/topics.rs`; the terminal non-forum classifier/test is assigned to later private `error.rs` | SQL upsert/read model, refresh coordination, command wrapper, and app tests |
| `sources/avatar.rs` | `peer_photo_bytes` and timeout belong to later `live/avatar.rs` | data URL, cache key/path, file write and cleanup |
| `takeout_import/mod.rs` | raw export-DC transport, peer/TL conversion, remote validation/migration/count/history/search/finish operations, and attempt/fallback outcomes are assigned to later `takeout/{transport,operations,types}.rs`; the channel-private classifier/test is assigned to later private `error.rs` | commands, job/start records, cancellation, locks, persistence, warnings/provenance, page loop, progress/events, and terminal finalization |
| `takeout_import/export_dc.rs` | all raw export-DC selection/invocation/classification and its seven tests are assigned to later `takeout/export_dc.rs` | no Grammers-bearing production owner; app provenance recording remains in `takeout_import/mod.rs` |
| `takeout_import/pagination.rs` | all range/page/cursor structs/functions and nine tests are assigned to later `takeout/pagination.rs` | none |
| `takeout_import/raw_parse.rs` | all raw TL-to-draft parsing helpers and five tests are assigned to later `takeout/raw_parse.rs` | none |
| `takeout_import/forum_topics.rs` | remote Telegram topic operation is assigned to later `takeout/forum_topics.rs` | completion policy, warnings, provenance, and all three current tests |
| `takeout_import/migrated_history.rs` | no production symbol in 8A; it consumes public identity later | capability SQL, policy/errors, identity construction, and all six tests |
| `ingest_provenance.rs` | none; it consumes public identity/item-kind later | all constants, DTOs, SQL, provider-key formatting, policy, and seven tests |
| `sources/mod.rs` | no owner; redirects exact moved public values through the 8A `crate::telegram` facade, then through `crate::telegram_impl` in 8B | module shell and all non-Telegram re-exports |
| `lib.rs` | no Phase 8A production symbol; its exact command registration is characterized | composition root and complete command registration |

No symbol assigned to “later 8B” is physically moved by this plan unless it is one of the explicitly named DTO/media/session/runtime symbols above. Their 8A physical paths are `telegram/*.rs`; the identity map's `staged_path` column deliberately records their later 8B/8C portable owner path.

## Literal Immutable 140-Test Identity Map

This table is executable plan data. Task 1's TypeScript contract parses it directly and requires exactly 140 unique baseline rows, 99 app primaries, 41 future-crate primaries, and exactly three future-crate companions. Those rows define the 143 eventual baseline-derived identities, not the complete executable test total at the end of 8A. `staged_path` is mapping metadata for the final 8B app-side portable path even for a value first prepared under `telegram/*.rs` in 8A; during baseline and 8A it is explicitly not a filesystem-existence assertion. App primaries keep their current path unless the row states another final identity.

| baseline_package | baseline_full_id | staged_path | final_owner | final_full_id | companion_final_ids |
| --- | --- | --- | --- | --- | --- |
| `extractum` | `ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags` | `src-tauri/src/ingest_provenance.rs` | `extractum` | `ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags` | — |
| `extractum` | `ingest_provenance::tests::create_takeout_batch_inserts_generic_and_detail_rows_atomically` | `src-tauri/src/ingest_provenance.rs` | `extractum` | `ingest_provenance::tests::create_takeout_batch_inserts_generic_and_detail_rows_atomically` | — |
| `extractum` | `ingest_provenance::tests::migrated_history_deferred_scope_finalizes_partial_and_records_warning_once` | `src-tauri/src/ingest_provenance.rs` | `extractum` | `ingest_provenance::tests::migrated_history_deferred_scope_finalizes_partial_and_records_warning_once` | — |
| `extractum` | `ingest_provenance::tests::migrated_small_group_imported_allows_duplicate_only_success` | `src-tauri/src/ingest_provenance.rs` | `extractum` | `ingest_provenance::tests::migrated_small_group_imported_allows_duplicate_only_success` | — |
| `extractum` | `ingest_provenance::tests::migrated_small_group_scope_can_be_marked_running_and_completed` | `src-tauri/src/ingest_provenance.rs` | `extractum` | `ingest_provenance::tests::migrated_small_group_scope_can_be_marked_running_and_completed` | — |
| `extractum` | `ingest_provenance::tests::mixed_partial_scope_finalizes_as_partial` | `src-tauri/src/ingest_provenance.rs` | `extractum` | `ingest_provenance::tests::mixed_partial_scope_finalizes_as_partial` | — |
| `extractum` | `ingest_provenance::tests::terminal_update_recalculates_counters_and_sanitizes_error` | `src-tauri/src/ingest_provenance.rs` | `extractum` | `ingest_provenance::tests::terminal_update_recalculates_counters_and_sanitizes_error` | — |
| `extractum` | `media::tests::derive_content_kind_tracks_text_and_media_presence` | `src-tauri/src/telegram_impl/media.rs` | `extractum-telegram` | `media::tests::derive_content_kind_tracks_text_and_media_presence` | — |
| `extractum` | `media::tests::derive_document_media_kind_prefers_specific_signals` | `src-tauri/src/telegram_impl/media.rs` | `extractum-telegram` | `media::tests::derive_document_media_kind_prefers_specific_signals` | — |
| `extractum` | `sources::identity::tests::canonical_external_id_rejects_malformed_values` | `src-tauri/src/sources/identity.rs` | `extractum` | `sources::identity::tests::canonical_external_id_rejects_malformed_values` | — |
| `extractum` | `sources::identity::tests::load_telegram_identity_returns_typed_row` | `src-tauri/src/sources/identity.rs` | `extractum` | `sources::identity::tests::load_telegram_identity_returns_typed_row` | — |
| `extractum` | `sources::identity::tests::load_telegram_runtime_source_pairs_source_with_typed_identity` | `src-tauri/src/sources/identity.rs` | `extractum` | `sources::identity::tests::load_telegram_runtime_source_pairs_source_with_typed_identity` | — |
| `extractum` | `sources::identity::tests::peer_kind_matches_telegram_subtype` | `src-tauri/src/sources/identity.rs` | `extractum` | `sources::identity::tests::peer_kind_matches_telegram_subtype` | — |
| `extractum` | `sources::identity::tests::username_normalization_removes_url_and_at_syntax` | `src-tauri/src/sources/identity.rs` | `extractum` | `sources::identity::tests::username_normalization_removes_url_and_at_syntax` | — |
| `extractum` | `sources::items::tests::forum_topic_filter_deserializes_camel_case_topic_id` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::forum_topic_filter_deserializes_camel_case_topic_id` | — |
| `extractum` | `sources::items::tests::insert_source_item_writes_payload_and_skips_duplicates` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates` | — |
| `extractum` | `sources::items::tests::insert_telegram_source_item_allows_same_message_id_in_different_history_domains` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::insert_telegram_source_item_allows_same_message_id_in_different_history_domains` | — |
| `extractum` | `sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item` | — |
| `extractum` | `sources::items::tests::insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload` | — |
| `extractum` | `sources::items::tests::list_source_items_enriches_youtube_comment_rows_from_raw_payload` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::list_source_items_enriches_youtube_comment_rows_from_raw_payload` | — |
| `extractum` | `sources::items::tests::list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed` | — |
| `extractum` | `sources::items::tests::media_metadata_roundtrip_through_zstd` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::media_metadata_roundtrip_through_zstd` | — |
| `extractum` | `sources::items::tests::migrated_insert_idempotency_uses_old_chat_native_identity` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::migrated_insert_idempotency_uses_old_chat_native_identity` | — |
| `extractum` | `sources::items::tests::migrated_small_group_insert_skips_current_history_derived_writes` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::migrated_small_group_insert_skips_current_history_derived_writes` | — |
| `extractum` | `sources::items::tests::reply_peer_context_uses_telegram_peer_kinds` | `src-tauri/src/telegram_impl/live/messages.rs` | `extractum-telegram` | `live::messages::tests::reply_peer_context_uses_telegram_peer_kinds` | — |
| `extractum` | `sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item` | — |
| `extractum` | `sources::items::tests::single_telegram_insert_maintains_ready_archive_model` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::single_telegram_insert_maintains_ready_archive_model` | — |
| `extractum` | `sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build` | — |
| `extractum` | `sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate` | — |
| `extractum` | `sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows` | — |
| `extractum` | `sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction` | — |
| `extractum` | `sources::items::tests::text_roundtrip_through_zstd` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::text_roundtrip_through_zstd` | — |
| `extractum` | `sources::items::tests::upsert_youtube_comment_item_updates_existing_text_and_reaction_count` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::upsert_youtube_comment_item_updates_existing_text_and_reaction_count` | — |
| `extractum` | `sources::items::tests::upsert_youtube_transcript_item_updates_existing_text_and_returns_id` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::upsert_youtube_transcript_item_updates_existing_text_and_returns_id` | — |
| `extractum` | `sources::items::tests::youtube_comment_upsert_targets_non_telegram_partial_unique_index` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::youtube_comment_upsert_targets_non_telegram_partial_unique_index` | — |
| `extractum` | `sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content` | — |
| `extractum` | `sources::items::tests::youtube_transcript_upsert_targets_non_telegram_partial_unique_index` | `src-tauri/src/sources/items.rs` | `extractum` | `sources::items::tests::youtube_transcript_upsert_targets_non_telegram_partial_unique_index` | — |
| `extractum` | `sources::peer_resolution::tests::add_source_resolution_strategy_distinguishes_username_and_dialog_flows` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::add_source_resolution_strategy_distinguishes_username_and_dialog_flows` | — |
| `extractum` | `sources::peer_resolution::tests::dialog_lookup_misses_are_not_found` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::dialog_lookup_misses_are_not_found` | — |
| `extractum` | `sources::peer_resolution::tests::dialog_lookup_not_found_message_explains_numeric_manual_limit` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::dialog_lookup_not_found_message_explains_numeric_manual_limit` | — |
| `extractum` | `sources::peer_resolution::tests::peer_ref_from_identity_ignores_small_groups_without_supported_identity` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::peer_ref_from_identity_ignores_small_groups_without_supported_identity` | — |
| `extractum` | `sources::peer_resolution::tests::peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation` | — |
| `extractum` | `sources::peer_resolution::tests::peer_ref_from_identity_uses_channel_access_hash` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::peer_ref_from_identity_uses_channel_access_hash` | — |
| `extractum` | `sources::peer_resolution::tests::peer_ref_from_identity_uses_supergroup_access_hash` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::peer_ref_from_identity_uses_supergroup_access_hash` | — |
| `extractum` | `sources::peer_resolution::tests::source_metadata_decode_failures_are_internal` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::source_metadata_decode_failures_are_internal` | — |
| `extractum` | `sources::peer_resolution::tests::source_metadata_decodes_old_dialog_payloads_into_peer_identity` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::source_metadata_decodes_old_dialog_payloads_into_peer_identity` | — |
| `extractum` | `sources::peer_resolution::tests::source_metadata_decodes_old_username_only_payloads` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::source_metadata_decodes_old_username_only_payloads` | — |
| `extractum` | `sources::peer_resolution::tests::source_metadata_decodes_typed_peer_identity_payloads` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::source_metadata_decodes_typed_peer_identity_payloads` | — |
| `extractum` | `sources::peer_resolution::tests::source_peer_input_rejects_malformed_external_id_as_validation` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::source_peer_input_rejects_malformed_external_id_as_validation` | — |
| `extractum` | `sources::peer_resolution::tests::source_peer_input_rejects_unsupported_source_type_as_validation` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::source_peer_input_rejects_unsupported_source_type_as_validation` | — |
| `extractum` | `sources::peer_resolution::tests::source_peer_resolution_failure_explains_small_group_dialog_dependency` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::source_peer_resolution_failure_explains_small_group_dialog_dependency` | — |
| `extractum` | `sources::peer_resolution::tests::source_peer_resolution_plan_prefers_explicit_strategy_order` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::source_peer_resolution_plan_prefers_explicit_strategy_order` | — |
| `extractum` | `sources::peer_resolution::tests::typed_identity_builds_channel_peer_ref_when_access_hash_exists` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::typed_identity_builds_channel_peer_ref_when_access_hash_exists` | — |
| `extractum` | `sources::peer_resolution::tests::typed_identity_plan_allows_username_resolution_without_access_hash` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::typed_identity_plan_allows_username_resolution_without_access_hash` | — |
| `extractum` | `sources::peer_resolution::tests::typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan` | — |
| `extractum` | `sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists` | — |
| `extractum` | `sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists` | — |
| `extractum` | `sources::peer_resolution::tests::typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists` | — |
| `extractum` | `sources::peer_resolution::tests::typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing` | `src-tauri/src/sources/peer_resolution.rs` | `extractum` | `sources::peer_resolution::tests::typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing` | — |
| `extractum` | `sources::peer_resolution::tests::typed_identity_rejects_subtype_peer_kind_mismatch` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::typed_identity_rejects_subtype_peer_kind_mismatch` | — |
| `extractum` | `sources::peer_resolution::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype` | `src-tauri/src/telegram_impl/live/peer.rs` | `extractum-telegram` | `live::peer::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype` | — |
| `extractum` | `sources::sync::tests::determine_sync_policy_only_applies_initial_settings_on_first_sync` | `src-tauri/src/sources/sync.rs` | `extractum` | `sources::sync::tests::determine_sync_policy_only_applies_initial_settings_on_first_sync` | — |
| `extractum` | `sources::sync::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary` | `src-tauri/src/telegram_impl/live/messages.rs` | `extractum-telegram` | `live::messages::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary` | — |
| `extractum` | `sources::sync::tests::finalize_sync_preserves_existing_legacy_metadata_blob` | `src-tauri/src/sources/sync.rs` | `extractum` | `sources::sync::tests::finalize_sync_preserves_existing_legacy_metadata_blob` | — |
| `extractum` | `sources::sync::tests::finalize_sync_updates_source_state_and_typed_avatar_cache` | `src-tauri/src/sources/sync.rs` | `extractum` | `sources::sync::tests::finalize_sync_updates_source_state_and_typed_avatar_cache` | — |
| `extractum` | `sources::sync::tests::sync_provider_accepts_telegram_sources` | `src-tauri/src/sources/sync.rs` | `extractum` | `sources::sync::tests::sync_provider_accepts_telegram_sources` | — |
| `extractum` | `sources::sync::tests::sync_provider_rejects_manual_youtube_video_sources` | `src-tauri/src/sources/sync.rs` | `extractum` | `sources::sync::tests::sync_provider_rejects_manual_youtube_video_sources` | — |
| `extractum` | `sources::topics::tests::forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists` | `src-tauri/src/sources/topics.rs` | `extractum` | `sources::topics::tests::forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists` | — |
| `extractum` | `sources::topics::tests::forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind` | `src-tauri/src/sources/topics.rs` | `extractum` | `sources::topics::tests::forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind` | — |
| `extractum` | `sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket` | `src-tauri/src/sources/topics.rs` | `extractum` | `sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket` | — |
| `extractum` | `sources::topics::tests::non_forum_topic_refresh_errors_are_detected` | `src-tauri/src/telegram_impl/error.rs` | `extractum-telegram` | `error::tests::non_forum_topic_refresh_errors_are_detected` | — |
| `extractum` | `sources::topics::tests::topic_refresh_rebuilds_materialized_memberships` | `src-tauri/src/sources/topics.rs` | `extractum` | `sources::topics::tests::topic_refresh_rebuilds_materialized_memberships` | — |
| `extractum` | `sources::topics::tests::upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted` | `src-tauri/src/sources/topics.rs` | `extractum` | `sources::topics::tests::upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted` | — |
| `extractum` | `sources::types::tests::item_kind_constants_match_persisted_wire_values` | `src-tauri/src/sources/types.rs` | `extractum` | `sources::types::tests::item_kind_constants_match_persisted_wire_values` | `dto::tests::telegram_item_kind_constant_matches_persisted_wire_value` |
| `extractum` | `sources::types::tests::source_type_serializes_supported_provider_values` | `src-tauri/src/sources/types.rs` | `extractum` | `sources::types::tests::source_type_serializes_supported_provider_values` | — |
| `extractum` | `sources::types::tests::telegram_message_identity_validation_rejects_invalid_values` | `src-tauri/src/telegram_impl/dto.rs` | `extractum-telegram` | `dto::tests::telegram_message_identity_validation_rejects_invalid_values` | — |
| `extractum` | `sources::types::tests::telegram_source_subtype_parses_from_canonical_source_subtype` | `src-tauri/src/sources/types.rs` | `extractum` | `sources::types::tests::telegram_source_subtype_parses_from_canonical_source_subtype` | — |
| `extractum` | `sources::types::tests::telegram_source_subtype_parses_supported_values` | `src-tauri/src/sources/types.rs` | `extractum` | `sources::types::tests::telegram_source_subtype_parses_supported_values` | — |
| `extractum` | `sources::types::tests::telegram_source_subtype_rejects_unknown_values_as_validation` | `src-tauri/src/sources/types.rs` | `extractum` | `sources::types::tests::telegram_source_subtype_rejects_unknown_values_as_validation` | — |
| `extractum` | `sources::types::tests::telegram_source_subtype_rejects_unsupported_source_subtype` | `src-tauri/src/sources/types.rs` | `extractum` | `sources::types::tests::telegram_source_subtype_rejects_unsupported_source_subtype` | — |
| `extractum` | `sources::types::tests::telegram_source_subtype_serializes_as_existing_wire_value` | `src-tauri/src/sources/types.rs` | `extractum` | `sources::types::tests::telegram_source_subtype_serializes_as_existing_wire_value` | — |
| `extractum` | `takeout_import::export_dc::tests::export_dc_attempt_state_detects_first_fallback_transition` | `src-tauri/src/telegram_impl/takeout/export_dc.rs` | `extractum-telegram` | `takeout::export_dc::tests::export_dc_attempt_state_detects_first_fallback_transition` | — |
| `extractum` | `takeout_import::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors` | `src-tauri/src/telegram_impl/takeout/export_dc.rs` | `extractum-telegram` | `takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors` | — |
| `extractum` | `takeout_import::export_dc::tests::export_dc_id_applies_tdesktop_shift` | `src-tauri/src/telegram_impl/takeout/export_dc.rs` | `extractum-telegram` | `takeout::export_dc::tests::export_dc_id_applies_tdesktop_shift` | — |
| `extractum` | `takeout_import::export_dc::tests::export_dc_invoke_does_not_fallback_for_rpc_errors` | `src-tauri/src/telegram_impl/takeout/export_dc.rs` | `extractum-telegram` | `takeout::export_dc::tests::export_dc_invoke_does_not_fallback_for_rpc_errors` | — |
| `extractum` | `takeout_import::export_dc::tests::export_dc_invoke_falls_back_to_home_dc_on_local_error` | `src-tauri/src/telegram_impl/takeout/export_dc.rs` | `extractum-telegram` | `takeout::export_dc::tests::export_dc_invoke_falls_back_to_home_dc_on_local_error` | — |
| `extractum` | `takeout_import::export_dc::tests::export_dc_invoke_uses_home_dc_directly_after_fallback` | `src-tauri/src/telegram_impl/takeout/export_dc.rs` | `extractum-telegram` | `takeout::export_dc::tests::export_dc_invoke_uses_home_dc_directly_after_fallback` | — |
| `extractum` | `takeout_import::export_dc::tests::takeout_init_request_uses_source_subtype_flags_and_file_limit` | `src-tauri/src/telegram_impl/takeout/export_dc.rs` | `extractum-telegram` | `takeout::export_dc::tests::takeout_init_request_uses_source_subtype_flags_and_file_limit` | — |
| `extractum` | `takeout_import::forum_topics::tests::completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups` | `src-tauri/src/takeout_import/forum_topics.rs` | `extractum` | `takeout_import::forum_topics::tests::completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups` | — |
| `extractum` | `takeout_import::forum_topics::tests::takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize` | `src-tauri/src/takeout_import/forum_topics.rs` | `extractum` | `takeout_import::forum_topics::tests::takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize` | — |
| `extractum` | `takeout_import::forum_topics::tests::takeout_forum_topic_refresh_success_records_no_warning` | `src-tauri/src/takeout_import/forum_topics.rs` | `extractum` | `takeout_import::forum_topics::tests::takeout_forum_topic_refresh_success_records_no_warning` | — |
| `extractum` | `takeout_import::migrated_history::tests::capability_available_is_source_level_and_restart_safe` | `src-tauri/src/takeout_import/migrated_history.rs` | `extractum` | `takeout_import::migrated_history::tests::capability_available_is_source_level_and_restart_safe` | — |
| `extractum` | `takeout_import::migrated_history::tests::capability_unavailable_keeps_reason_internal_and_clears_chat_hint` | `src-tauri/src/takeout_import/migrated_history.rs` | `extractum` | `takeout_import::migrated_history::tests::capability_unavailable_keeps_reason_internal_and_clears_chat_hint` | — |
| `extractum` | `takeout_import::migrated_history::tests::migrated_history_errors_are_typed_for_frontend_behavior` | `src-tauri/src/takeout_import/migrated_history.rs` | `extractum` | `takeout_import::migrated_history::tests::migrated_history_errors_are_typed_for_frontend_behavior` | — |
| `extractum` | `takeout_import::migrated_history::tests::migrated_small_group_identity_uses_native_old_chat_scope` | `src-tauri/src/takeout_import/migrated_history.rs` | `extractum` | `takeout_import::migrated_history::tests::migrated_small_group_identity_uses_native_old_chat_scope` | — |
| `extractum` | `takeout_import::migrated_history::tests::validation_accepts_matching_revalidated_chat_id` | `src-tauri/src/takeout_import/migrated_history.rs` | `extractum` | `takeout_import::migrated_history::tests::validation_accepts_matching_revalidated_chat_id` | — |
| `extractum` | `takeout_import::migrated_history::tests::validation_rejects_missing_or_changed_revalidated_chat_id` | `src-tauri/src/takeout_import/migrated_history.rs` | `extractum` | `takeout_import::migrated_history::tests::validation_rejects_missing_or_changed_revalidated_chat_id` | — |
| `extractum` | `takeout_import::pagination::tests::descending_fallback_keeps_raw_order_and_moves_to_min_message_id` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::descending_fallback_keeps_raw_order_and_moves_to_min_message_id` | — |
| `extractum` | `takeout_import::pagination::tests::messages_not_modified_response_is_rejected_for_takeout_page` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::messages_not_modified_response_is_rejected_for_takeout_page` | — |
| `extractum` | `takeout_import::pagination::tests::messages_response_without_slice_is_terminal_page` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::messages_response_without_slice_is_terminal_page` | — |
| `extractum` | `takeout_import::pagination::tests::split_selection_falls_back_when_telegram_returns_no_ranges` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::split_selection_falls_back_when_telegram_returns_no_ranges` | — |
| `extractum` | `takeout_import::pagination::tests::split_selection_uses_all_ranges_for_small_group` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::split_selection_uses_all_ranges_for_small_group` | — |
| `extractum` | `takeout_import::pagination::tests::split_selection_uses_last_range_for_channel_and_supergroup` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::split_selection_uses_last_range_for_channel_and_supergroup` | — |
| `extractum` | `takeout_import::pagination::tests::tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback` | — |
| `extractum` | `takeout_import::pagination::tests::tdesktop_non_advancing_cursor_restarts_descending_fallback` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::tdesktop_non_advancing_cursor_restarts_descending_fallback` | — |
| `extractum` | `takeout_import::pagination::tests::tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id` | `src-tauri/src/telegram_impl/takeout/pagination.rs` | `extractum-telegram` | `takeout::pagination::tests::tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id` | — |
| `extractum` | `takeout_import::raw_parse::tests::parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids` | `src-tauri/src/telegram_impl/takeout/raw_parse.rs` | `extractum-telegram` | `takeout::raw_parse::tests::parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids` | — |
| `extractum` | `takeout_import::raw_parse::tests::parses_document_media_kind_filename_and_dimensions` | `src-tauri/src/telegram_impl/takeout/raw_parse.rs` | `extractum-telegram` | `takeout::raw_parse::tests::parses_document_media_kind_filename_and_dimensions` | — |
| `extractum` | `takeout_import::raw_parse::tests::parses_photo_message_metadata` | `src-tauri/src/telegram_impl/takeout/raw_parse.rs` | `extractum-telegram` | `takeout::raw_parse::tests::parses_photo_message_metadata` | — |
| `extractum` | `takeout_import::raw_parse::tests::parses_text_message_with_reply_and_reactions` | `src-tauri/src/telegram_impl/takeout/raw_parse.rs` | `extractum-telegram` | `takeout::raw_parse::tests::parses_text_message_with_reply_and_reactions` | — |
| `extractum` | `takeout_import::raw_parse::tests::skips_empty_raw_messages` | `src-tauri/src/telegram_impl/takeout/raw_parse.rs` | `extractum-telegram` | `takeout::raw_parse::tests::skips_empty_raw_messages` | — |
| `extractum` | `takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation` | — |
| `extractum` | `takeout_import::tests::channel_private_detection_reads_rpc_name_from_error_message` | `src-tauri/src/telegram_impl/error.rs` | `extractum-telegram` | `error::tests::channel_private_detection_reads_rpc_name_from_error_message` | — |
| `extractum` | `takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues` | — |
| `extractum` | `takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize` | — |
| `extractum` | `takeout_import::tests::historical_batch_completion_does_not_advance_source_watermark` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::historical_batch_completion_does_not_advance_source_watermark` | — |
| `extractum` | `takeout_import::tests::locked_start_allows_only_one_batch_for_same_source` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::locked_start_allows_only_one_batch_for_same_source` | — |
| `extractum` | `takeout_import::tests::locked_start_conflict_creates_no_provenance_rows` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::locked_start_conflict_creates_no_provenance_rows` | — |
| `extractum` | `takeout_import::tests::migrated_history_detected_warning_is_sanitized` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::migrated_history_detected_warning_is_sanitized` | — |
| `extractum` | `takeout_import::tests::migrated_history_start_records_use_same_source_takeout_lock` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::migrated_history_start_records_use_same_source_takeout_lock` | — |
| `extractum` | `takeout_import::tests::migrated_history_start_requires_available_capability` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::migrated_history_start_requires_available_capability` | — |
| `extractum` | `takeout_import::tests::only_my_messages_fallback_is_limited_to_channels` | `src-tauri/src/telegram_impl/takeout/operations.rs` | `extractum-telegram` | `takeout::operations::tests::only_my_messages_fallback_is_limited_to_channels` | — |
| `extractum` | `takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once` | `takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id` |
| `extractum` | `takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers` | `takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids` |
| `extractum` | `takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future` | — |
| `extractum` | `takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future` | — |
| `extractum` | `takeout_import::tests::takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists` | — |
| `extractum` | `takeout_import::tests::takeout_subtype_load_uses_typed_identity_not_legacy_kind` | `src-tauri/src/takeout_import/mod.rs` | `extractum` | `takeout_import::tests::takeout_subtype_load_uses_typed_identity_not_legacy_kind` | — |
| `extractum` | `telegram::tests::diagnostic_status_counts_do_not_return_account_ids_or_messages` | `src-tauri/src/telegram.rs` | `extractum` | `telegram::tests::diagnostic_status_counts_do_not_return_account_ids_or_messages` | — |
| `extractum` | `telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column` | `src-tauri/src/telegram.rs` | `extractum` | `telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column` | — |
| `extractum` | `telegram::tests::legacy_api_hash_remains_when_secret_write_fails` | `src-tauri/src/telegram.rs` | `extractum` | `telegram::tests::legacy_api_hash_remains_when_secret_write_fails` | — |
| `extractum` | `telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error` | `src-tauri/src/telegram.rs` | `extractum` | `telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error` | — |
| `extractum` | `telegram::tests::telegram_api_id_out_of_range_returns_typed_validation_error` | `src-tauri/src/telegram.rs` | `extractum` | `telegram::tests::telegram_api_id_out_of_range_returns_typed_validation_error` | — |
| `extractum` | `telegram_session_store::tests::delete_session_from_path_removes_file_and_key` | `src-tauri/src/telegram_session_store.rs` | `extractum` | `telegram_session_store::tests::delete_session_from_path_removes_file_and_key` | — |
| `extractum` | `telegram_session_store::tests::encrypted_session_load_fails_for_wrong_account_id` | `src-tauri/src/telegram_impl/session.rs` | `extractum-telegram` | `session::tests::encrypted_session_load_fails_for_wrong_account_id` | — |
| `extractum` | `telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing` | `src-tauri/src/telegram_session_store.rs` | `extractum` | `telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing` | — |
| `extractum` | `telegram_session_store::tests::encrypted_session_load_round_trips` | `src-tauri/src/telegram_impl/session.rs` | `extractum-telegram` | `session::tests::encrypted_session_load_round_trips` | — |
| `extractum` | `telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file` | `src-tauri/src/telegram_session_store.rs` | `extractum` | `telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file` | — |
| `extractum` | `telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails` | `src-tauri/src/telegram_session_store.rs` | `extractum` | `telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails` | — |
| `extractum` | `telegram_session_store::tests::saving_session_writes_encrypted_envelope_not_plaintext` | `src-tauri/src/telegram_impl/session.rs` | `extractum-telegram` | `session::tests::saving_session_writes_encrypted_envelope_not_plaintext` | — |

### Exact New-Test Identity Map

These are exactly 18 plan-added rows, separate from the immutable 140 baseline. The item-kind test is the one mandatory companion counted in both the 141 post-8A and 143 eventual baseline-derived metrics; the other 17 rows are additional verification. Therefore 8A ends with 158 tracked identities, while the eventual tracked total is 160 only after the other two companions exist.

| Checkpoint | Phase 8A exact identity in `extractum` | Future owner / final identity |
| --- | --- | --- |
| 2 | `telegram::tests::telegram_status_and_event_payload_contract_is_exact` | app / unchanged |
| 2 | `telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact` | app / unchanged |
| 2 | `takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact` | app / unchanged |
| 3 | `telegram::dto::tests::telegram_item_kind_constant_matches_persisted_wire_value` | crate / `dto::tests::telegram_item_kind_constant_matches_persisted_wire_value` |
| 3 | `telegram::dto::tests::telegram_message_draft_has_single_persistence_shape` | crate / `dto::tests::telegram_message_draft_has_single_persistence_shape` |
| 4 | `telegram::session::tests::session_encryption_key_rejects_invalid_length` | crate / `session::tests::session_encryption_key_rejects_invalid_length` |
| 4 | `telegram::session::tests::legacy_json_returns_rewrite_decision` | crate / `session::tests::legacy_json_returns_rewrite_decision` |
| 4 | `telegram::session::tests::missing_encrypted_key_preserves_auth_error` | crate / `session::tests::missing_encrypted_key_preserves_auth_error` |
| 4 | `telegram::session::tests::generated_session_key_returns_write_only_encoded_secret` | crate / `session::tests::generated_session_key_returns_write_only_encoded_secret` |
| 5 | `telegram::runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner` | crate / `runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner` |
| 5 | `telegram::runtime::tests::missing_account_authentication_is_false` | crate / `runtime::tests::missing_account_authentication_is_false` |
| 5 | `telegram::runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt` | crate / `runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt` |
| 5 | `telegram::runtime::tests::sign_in_without_code_request_preserves_auth_error` | crate / `runtime::tests::sign_in_without_code_request_preserves_auth_error` |
| 5 | `telegram::runtime::tests::failed_sign_in_retains_pending_attempt` | crate / `runtime::tests::failed_sign_in_retains_pending_attempt` |
| 5 | `telegram::runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt` | crate / `runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt` |
| 5 | `telegram::runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure` | crate / `runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure` |
| 5 | `telegram::runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors` | crate / `runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors` |
| 5 | `telegram::tests::runtime_status_maps_to_existing_wire_strings` | app / unchanged |

The exact test-only runtime seam is frozen in `Frozen Phase 8A Public and Internal API`. It is `#[cfg(test)]`, private, and accepts deterministic fake authorization/login/sign-out/runner behavior. It is not a production transport trait or public runtime abstraction.

---

### Task 0: Synchronize the Bounded Evidence Correction Before Code

**Files:**

- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Verify only: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Verify only: the exact 19 ownership/move paths plus `src-tauri/src/sources/store.rs`
- Verify only: `src-tauri/Cargo.toml`
- Verify only: `src-tauri/Cargo.lock`

**Interfaces:**

- **Consumes:** the owner-approved plan at the actual clean `HEAD`, approval ancestor `9f56a4584b0c2abb331c1b2ab7f198ccb89db042`, the current four direct Grammers declarations, the exact 19 symbol-discovered paths, the separate wiring-only `src-tauri/src/lib.rs`, and their classified 20-path union: 19 ownership/move paths plus dependent-only `sources/store.rs`.
- **Produces:** one docs-only authority-synchronization commit whose only paths are the design and roadmap, whose status strings are unchanged, and whose durable correction says “19 ownership/move paths plus one dependent-only consumer with 24 regressions.”
- **Hands to Task 1:** `$authoritySyncCommit`, the pre-sync starting SHA, exact terminal path inventories, and unchanged manifest/lock hashes.

This is an authority-synchronization commit, not a Phase 8A preparation checkpoint. It changes no roadmap/design status, production source, test source, manifest, lockfile, or executable behavior. Do not begin Task 1 unless the owner explicitly approved this plan and therefore the bounded correction stated above.

- [ ] **Step 1: Record the pre-execution identity and refresh direct Grammers dependency declarations.**

```powershell
Assert-CleanWorktree 'Phase 8A authority synchronization start'
Invoke-CheckedNative 'approved implementation plan is tracked' {
    git ls-files --error-unmatch docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
}
$authorityStartingCommit = (git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Could not record authority synchronization start' }
Invoke-CheckedNative 'verify approved design ancestor' {
    git merge-base --is-ancestor 9f56a4584b0c2abb331c1b2ab7f198ccb89db042 HEAD
}
$authorityManifestHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.toml').Hash
$authorityLockHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.lock').Hash

$metadataOutput = & cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 --no-deps
if ($LASTEXITCODE -ne 0) { $metadataOutput; throw 'Locked authority metadata failed' }
$metadata = ($metadataOutput | Out-String) | ConvertFrom-Json
$app = @($metadata.packages | Where-Object { $_.name -eq 'extractum' })
if ($app.Count -ne 1) { throw "Expected one extractum package, found $($app.Count)" }

$expectedGrammers = [ordered]@{
    'grammers-client' = @{
        source = 'git+https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa'
        uses_default_features = $false
        features = @()
    }
    'grammers-mtsender' = @{
        source = 'git+https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa'
        uses_default_features = $true
        features = @()
    }
    'grammers-session' = @{
        source = 'git+https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa'
        uses_default_features = $false
        features = @('serde')
    }
    'grammers-tl-types' = @{
        source = 'git+https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa'
        uses_default_features = $true
        features = @('deserializable-functions')
    }
}
$actualGrammers = @($app[0].dependencies | Where-Object { $_.name -like 'grammers-*' })
if ($actualGrammers.Count -ne 4) { throw "Expected four direct Grammers roots, found $($actualGrammers.Count)" }
foreach ($name in $expectedGrammers.Keys) {
    $actual = @($actualGrammers | Where-Object { $_.name -eq $name })
    if ($actual.Count -ne 1) { throw "Expected one direct dependency declaration for $name" }
    $expected = $expectedGrammers[$name]
    if ($actual[0].source -ne $expected.source) { throw "Unexpected source/revision for $name" }
    if ($actual[0].uses_default_features -ne $expected.uses_default_features) {
        throw "Unexpected default-feature declaration for $name"
    }
    if (
        $actual[0].req -ne '*' -or
        $null -ne $actual[0].kind -or
        $null -ne $actual[0].rename -or
        $actual[0].optional -ne $false -or
        $null -ne $actual[0].target -or
        $null -ne $actual[0].registry
    ) {
        throw "Unexpected dependency kind/target/optional/rename/requirement for $name"
    }
    $actualFeatures = @($actual[0].features | Sort-Object)
    $expectedFeatures = @($expected.features | Sort-Object)
    if (($actualFeatures -join "`n") -ne ($expectedFeatures -join "`n")) {
        throw "Unexpected explicit features for $name"
    }
}

$telegramPackages = @($metadata.packages | Where-Object { $_.name -eq 'extractum-telegram' })
if ($telegramPackages.Count -ne 0) { throw 'extractum-telegram package already exists' }
if (@($metadata.workspace_members | Where-Object { $_ -match 'extractum-telegram' }).Count -ne 0) {
    throw 'extractum-telegram workspace member already exists'
}
$telegramEdges = @($app[0].dependencies | Where-Object { $_.name -eq 'extractum-telegram' })
if ($telegramEdges.Count -ne 0) { throw 'extractum already has an extractum-telegram dependency edge' }
if (Test-Path -LiteralPath 'src-tauri/crates/extractum-telegram') {
    throw 'src-tauri/crates/extractum-telegram already exists'
}
```

This refresh freezes only current manifest declarations. The resolved required/forbidden feature baseline remains an 8B deliverable exactly as the approved design requires.

- [ ] **Step 2: Refresh direct Grammers paths and the moved-type fan-in/fan-out to a fixed point.**

```powershell
$expectedDirectGrammersPaths = @(
    'src-tauri/src/media.rs'
    'src-tauri/src/sources/avatar.rs'
    'src-tauri/src/sources/identity.rs'
    'src-tauri/src/sources/items.rs'
    'src-tauri/src/sources/peer_resolution.rs'
    'src-tauri/src/sources/sync.rs'
    'src-tauri/src/sources/topics.rs'
    'src-tauri/src/takeout_import/export_dc.rs'
    'src-tauri/src/takeout_import/forum_topics.rs'
    'src-tauri/src/takeout_import/mod.rs'
    'src-tauri/src/takeout_import/pagination.rs'
    'src-tauri/src/takeout_import/raw_parse.rs'
    'src-tauri/src/telegram.rs'
    'src-tauri/src/telegram_session_store.rs'
) | Sort-Object -Unique

$directOutput = & rg -l 'grammers_(client|session|mtsender|tl_types)' src-tauri/src --glob '*.rs'
if ($LASTEXITCODE -ne 0) { $directOutput; throw 'Direct Grammers path refresh failed or selected no files' }
$actualDirectGrammersPaths = @(
    $directOutput | ForEach-Object { $_.Replace('\', '/') } | Sort-Object -Unique
)
if (($actualDirectGrammersPaths -join "`n") -ne ($expectedDirectGrammersPaths -join "`n")) {
    Compare-Object $expectedDirectGrammersPaths $actualDirectGrammersPaths
    throw 'Direct Grammers production path set drifted'
}

$expectedDiscoveredFixedPoint = @(
    $expectedDirectGrammersPaths
    'src-tauri/src/ingest_provenance.rs'
    'src-tauri/src/sources/mod.rs'
    'src-tauri/src/sources/store.rs'
    'src-tauri/src/sources/types.rs'
    'src-tauri/src/takeout_import/migrated_history.rs'
) | Sort-Object -Unique

$fanPattern = '(?:\b(get_client|get_authorized_runtime|AuthorizedTelegramRuntime|AccountClient|raw_client|raw_session|MemorySession|LoginToken|TelegramMessageIdentity|TelegramItemContext|SourceItemInsert|ExtractedItemPayload|ExtractedMediaPayload|ITEM_KIND_TELEGRAM_MESSAGE)\b|\.accounts\s*\.lock\s*\(\s*\)\s*\.await\b)'
$fanOutput = & rg -l $fanPattern src-tauri/src --glob '*.rs'
if ($LASTEXITCODE -ne 0) { $fanOutput; throw 'Moved-type fan-in/fan-out refresh failed or selected no files' }
$actualDiscoveredFixedPoint = @(
    $actualDirectGrammersPaths
    ($fanOutput | ForEach-Object { $_.Replace('\', '/') })
) | Sort-Object -Unique
if (($actualDiscoveredFixedPoint -join "`n") -ne ($expectedDiscoveredFixedPoint -join "`n")) {
    Compare-Object $expectedDiscoveredFixedPoint $actualDiscoveredFixedPoint
    throw 'Phase 8 symbol-discovered fixed-point path set drifted'
}
if ($actualDiscoveredFixedPoint.Count -ne 19) {
    throw "Expected 19 symbol-discovered paths, found $($actualDiscoveredFixedPoint.Count)"
}

$wiringPaths = @(
    'src-tauri/src/lib.rs'
) | Sort-Object -Unique
foreach ($wiringPath in $wiringPaths) {
    if (-not (Test-Path -LiteralPath $wiringPath -PathType Leaf)) {
        throw "Required explicit wiring path is missing: $wiringPath"
    }
}
$unexpectedWiringDiscovery = @(
    $actualDiscoveredFixedPoint | Where-Object { $wiringPaths -contains $_ }
)
if ($unexpectedWiringDiscovery.Count -ne 0) {
    $unexpectedWiringDiscovery
    throw 'Explicit wiring path unexpectedly entered symbol discovery; amend its classification'
}

$expectedCompleteSurface = @(
    $expectedDiscoveredFixedPoint
    $wiringPaths
) | Sort-Object -Unique
$actualCompleteSurface = @(
    $actualDiscoveredFixedPoint
    $wiringPaths
) | Sort-Object -Unique
if (
    ($actualCompleteSurface -join "`n") -ne ($expectedCompleteSurface -join "`n") -or
    $actualCompleteSurface.Count -ne 20
) {
    Compare-Object $expectedCompleteSurface $actualCompleteSurface
    throw 'Phase 8 complete 20-path touch surface drifted'
}

$ownershipMovePaths = @($actualCompleteSurface | Where-Object { $_ -ne 'src-tauri/src/sources/store.rs' })
if ($ownershipMovePaths.Count -ne 19) { throw "Expected 19 ownership/move paths, found $($ownershipMovePaths.Count)" }
$dependentOnlyPaths = @($actualDiscoveredFixedPoint | Where-Object { $_ -eq 'src-tauri/src/sources/store.rs' })
if ($dependentOnlyPaths.Count -ne 1) { throw 'Expected sources/store.rs as the sole dependent-only path' }
```

`$directOutput` is the raw-type import/qualification sentinel: any file that directly names `Client`, `PeerRef`, `Media`, `Message`, or another raw Grammers type through a `grammers_*` path enters that set. `$fanPattern` separately catches application wrapper/accessor/type names plus direct `.accounts.lock().await` access, including inferred raw values whose concrete type is absent from the consumer text. `$actualDiscoveredFixedPoint` contains only the 19 paths discovered by those scans: 18 ownership/move paths plus dependent-only `sources/store.rs`. `src-tauri/src/lib.rs` is separately named, existence-checked, and required not to overlap discovery in `$wiringPaths` because it is explicit application wiring, not a moved/raw-symbol match. `$actualCompleteSurface` is their exact 20-path union, yielding 19 ownership/move paths plus one dependent-only path. Do not add bare `Client|Media|Message` tokens to the same repository-wide regex: they currently collide with Gemini, YouTube, NotebookLM, and ordinary message/media vocabulary without detecting fully inferred propagation. The standing contract instead rejects any new raw `pub`/`pub(crate) use`, raw type alias, raw-returning facade function/field, or raw accessor outside exact baseline `AccountClient`, `AuthorizedTelegramRuntime`, `get_client`, `get_authorized_runtime` and the later status-gated adapter allowlist.

Manually inspect every `rg` match grouped by moved/raw symbol. For every `as` alias, re-export, wrapper return, or field that changes the searched identifier, add that new identifier to `$fanPattern` and repeat the scan. Continue until an iteration adds neither a path nor a seed; record the terminal exact 19-owner plus one-dependent disposition. A new path, raw-client/session consumer, or symbol owner is not folded into the plan: stop for a design/plan amendment. Task 1 turns this terminal inventory into a standing fail-closed source contract so later checkpoints cannot silently add an alias or consumer.

- [ ] **Step 3: Synchronize the approved bounded correction and commit it alone.**

Update the design evidence and implementation requirements to say:

- the immutable ownership/move surface remains 19 paths / 140 tests;
- `src-tauri/src/sources/store.rs` is the sole known dependent-only raw-client consumer outside that map;
- its 24 app tests stay outside the ownership map and run as broad store regressions in 8A Checkpoint 5; none directly invokes `list_telegram_sources` or `add_telegram_source`, so the Checkpoint 5 runtime lookup test plus lifecycle-gated source contract—not this suite—prove the caller-lock rewrite;
- the complete Phase 8 implementation touch surface is therefore the 19 ownership/move paths plus this one dependent-only path.

Mirror the same bounded statement in the Phase 8 roadmap. Do not change the current approval/status strings. Prove the spec and roadmap agree, the manifest/lock hashes still equal Step 1, and run:

```powershell
Invoke-CheckedNative 'unchanged Phase 8 status contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
$currentManifestHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.toml').Hash
$currentLockHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.lock').Hash
if ($currentManifestHash -ne $authorityManifestHash) { throw 'Task 0 changed Cargo.toml' }
if ($currentLockHash -ne $authorityLockHash) { throw 'Task 0 changed Cargo.lock' }
$task0Allowed = @(
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md'
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
)
Assert-ScopedChanges -Label 'Task 0 docs-only scope' -Allowed $task0Allowed -RequireChanges
Invoke-CheckedNative 'Task 0 diff check' { git diff --check }
```

Inspect and stage only the two documentation files, verify the cached path set equals `$task0Allowed`, then:

```powershell
Invoke-CheckedNative 'stage bounded Phase 8 evidence correction' {
    git add -- docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md docs/superpowers/specs/2026-07-17-crate-roadmap.md
}
$cachedTask0 = @(& git diff --cached --name-only --relative | Sort-Object -Unique)
if (($cachedTask0 -join "`n") -ne (($task0Allowed | Sort-Object) -join "`n")) {
    throw "Unexpected Task 0 staged paths: $($cachedTask0 -join ', ')"
}
Invoke-CheckedNative 'Task 0 cached diff check' { git diff --cached --check }
Invoke-CheckedNative 'commit bounded Phase 8 evidence correction' {
    git commit -m "docs: correct Phase 8 Telegram dependent surface"
}
$authoritySyncCommit = (git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Could not record authority-synchronization SHA' }
"authority_sync_sha=$authoritySyncCommit"
```

Record the authority-synchronization SHA. Reload the committed design, roadmap, and this plan before Task 1. Do not combine this commit with any code or contract implementation.

---

### Task 1: Checkpoint 1 — Freeze the Boundary, Test Map, and Core Error Seam

**Files:**

- Create: `src/lib/telegram-contract-paths.ts`
- Create: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Verify only: the exact 19 ownership/move paths in the symbol table plus dependent-only `src-tauri/src/sources/store.rs`
- Verify only: `src-tauri/Cargo.toml`
- Verify only: `src-tauri/Cargo.lock`

**Interfaces:**

- **Consumes:** the clean Task 0 commit/SHA, its exact 19+1 path disposition, the executable 140-primary map, the separate exact 24-test `sources::store` set, and unchanged Cargo identity.
- **Produces:** `telegram-contract-paths.ts`, the standing Telegram boundary contract, the closed Phase 8 status vocabulary, and exactly six `sources/types.rs` validation paths normalized to `extractum_core::error`.
- **Hands to Task 2:** a committed fail-closed source/test lifecycle contract, `$startingCommit`, `$manifestHash`, `$lockHash`, and Checkpoint 1 SHA; no production Telegram value or behavior changes.

- [ ] **Step 1: Record the actual clean starting identity and immutable manifest hashes.**

```powershell
Assert-CleanWorktree 'Phase 8A start'
$startingCommit = (git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Could not record starting commit' }
$startingSubject = (git show -s --format=%s HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $startingSubject -ne 'docs: correct Phase 8 Telegram dependent surface') {
    throw 'Task 1 must start directly from the committed Task 0 authority synchronization'
}
$expectedTask0Paths = @(
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md'
) | Sort-Object
$actualTask0Paths = @(& git diff-tree --no-commit-id --name-only -r HEAD | Sort-Object -Unique)
if (($actualTask0Paths -join "`n") -ne ($expectedTask0Paths -join "`n")) {
    throw "Task 0 commit has unexpected paths: $($actualTask0Paths -join ', ')"
}
Invoke-CheckedNative 'verify approved ancestor' {
    git merge-base --is-ancestor 9f56a4584b0c2abb331c1b2ab7f198ccb89db042 HEAD
}
$manifestHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.toml').Hash
$lockHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.lock').Hash
"starting_head=$startingCommit manifest_sha256=$manifestHash lock_sha256=$lockHash"
Invoke-CheckedNative 'locked starting metadata' {
    cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
}
```

Expected: clean worktree; approval commit is an ancestor; metadata succeeds; no `src-tauri/crates/extractum-telegram` path/member/edge.

- [ ] **Step 2: Reproduce the exact 140-test baseline from executable output.**

Run `cargo test -- --list`, select only these exact prefixes, and require the listed counts:

```powershell
$phase8Prefixes = [ordered]@{
    'telegram::tests::' = 5
    'telegram_session_store::tests::' = 7
    'media::tests::' = 2
    'sources::avatar::tests::' = 0
    'sources::identity::tests::' = 5
    'sources::items::tests::' = 23
    'sources::peer_resolution::tests::' = 24
    'sources::sync::tests::' = 6
    'sources::topics::tests::' = 6
    'takeout_import::tests::' = 17
    'takeout_import::export_dc::tests::' = 7
    'takeout_import::forum_topics::tests::' = 3
    'takeout_import::pagination::tests::' = 9
    'takeout_import::raw_parse::tests::' = 5
    'sources::types::tests::' = 8
    'ingest_provenance::tests::' = 7
    'takeout_import::migrated_history::tests::' = 6
}
$output = & cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list 2>&1
if ($LASTEXITCODE -ne 0) { $output; throw 'Phase 8A baseline test list failed' }
$listed = @($output | ForEach-Object {
    if ($_ -match '^([^ ]+): test$') { $Matches[1] }
})
$selected = @()
foreach ($entry in $phase8Prefixes.GetEnumerator()) {
    $matches = @($listed | Where-Object { $_.StartsWith($entry.Key) })
    if ($matches.Count -ne $entry.Value) {
        throw "Prefix $($entry.Key) expected $($entry.Value), found $($matches.Count)"
    }
    $selected += $matches
}
$selected = @($selected | Sort-Object -Unique)
if ($selected.Count -ne 140) { throw "Expected 140 unique baseline identities, found $($selected.Count)" }
```

Compare `$selected` exactly to the plan table's `baseline_full_id` column. A missing, extra, renamed, or duplicate identity stops execution and requires a plan amendment.

Independently freeze the dependent-only store suite without adding it to the 140-row ownership map:

```powershell
Assert-ExactRustIdentitySet `
  -Package extractum `
  -Prefix 'sources::store::tests::' `
  -Expected $sourcesStoreBroadRegressionTestIds
```

The TypeScript boundary contract parses the single committed `$sourcesStoreBroadRegressionTestIds` literal from this plan and requires exactly 24 unique identities, all under `sources::store::tests::`, declared exactly once in `sources/store.rs`, and absent from the immutable 140-row ownership map. It does not spawn Cargo. `Assert-ExactRustIdentitySet` is the separate executable Cargo-list proof at CP1, CP5, and Task 6.

- [ ] **Step 3: Implement the standing literal-map and lifecycle contract.**

`telegram-contract-paths.ts` reads only explicit repository-relative files, rejects traversal/missing paths, and understands four closed layouts:

1. current baseline modules;
2. 8A future-owner leaves under `src-tauri/src/telegram/*.rs`;
3. 8B staging under `src-tauri/src/telegram_impl/**`;
4. 8C crate sources plus retained app consumers.

In baseline and every 8A checkpoint, the parser never passes a row's `staged_path` directly to a filesystem-existence assertion. For the four prepared leaves it uses this closed lifecycle map:

```text
src-tauri/src/telegram_impl/dto.rs     -> src-tauri/src/telegram/dto.rs
src-tauri/src/telegram_impl/media.rs   -> src-tauri/src/telegram/media.rs
src-tauri/src/telegram_impl/session.rs -> src-tauri/src/telegram/session.rs
src-tauri/src/telegram_impl/runtime.rs -> src-tauri/src/telegram/runtime.rs
```

Before each leaf's Checkpoint 3–5 introduction it resolves that row to the current baseline source path; other future-owner rows remain on their current baseline paths throughout 8A. Literal `staged_path` existence becomes mandatory only in the corresponding approved 8B lifecycle state.

`telegram-crate-boundary-contract.test.ts` parses the literal table from this committed plan; never copy its rows into TypeScript. It fails on malformed rows, duplicate baseline/final identities, missing paths, owner values outside `extractum|extractum-telegram`, or companions outside the exact three declared IDs. It asserts:

```text
baseline rows:                 140
baseline package extractum:   140
app primaries:                 99
future-crate primaries:        41
companions:                    3
eventual baseline-derived:    143 unique identities
plan-added rows:               18
additional verification:      17
post-8A baseline-derived:     141
post-8A tracked total:        158
eventual tracked total:       160
```

It also parses the approved specification and proves:

- all exact 43 helper-dependent identities map to `extractum`;
- the exact three credential-SQL identities map to `extractum`;
- the residual 73 direct-perimeter identities are individually present;
- the 21 type-closure identities are exactly `sources::types` 8 + `ingest_provenance` 7 + `takeout_import::migrated_history` 6;
- that closure has 20 app primaries, one future-crate identity-validation primary, and the one item-kind companion;
- the two raw-TL/SQL app primaries carry only their two declared raw-parse companions;
- `insert_source_item_writes_payload_and_skips_duplicates` has the exact renamed retained final identity;
- `media_metadata_roundtrip_through_zstd` remains app-owned.

Before CP3, the baseline-derived set remains exactly 140 while Checkpoint 2 has added exactly three separately tracked verification identities. After CP3, the contract maps DTO/media leaves to `telegram::{dto,media}::tests`, preserves all app identities, and admits only the item-kind companion, making 141 baseline-derived identities plus the Checkpoint 2 and Checkpoint 3 additional verification rows. After CP4 it additionally maps the three codec-subject tests to `telegram::session::tests`. Future 8B/8C mappings are parsed now but remain inactive until those exact paths/manifests exist.

Plan-added identities are checkpoint-gated by the closed roadmap state: each is required from its declared checkpoint onward and forbidden before that checkpoint unless an amended plan moves it deliberately. They never alter the immutable 140-row counts.

- [ ] **Step 4: Freeze the synchronized 19-file `crate::<root>` inventory and dependent-only consumer.**

Before Step 5, prove and record that the approved 19-path set contains exactly 166 references:

```text
error=43; compression=5; time=2; sources=46; ingest_provenance=19;
archive_read_model=7; topic_memberships=7; db=6; youtube=6;
secret_store=5; media=4; source_ingest=3; takeout_import=3; telegram=3;
analysis_documents=2; tx=2; forum_topics=1; readiness=1;
telegram_session_store=1
```

The standing contract keeps this immutable baseline table as evidence. It also encodes Task 0's terminal direct-Grammers and moved/raw-symbol seed sets, rejects an unexpected alias/re-export/wrapper consumer, and separately asserts exactly 19 symbol-discovered paths, wiring-only set `{src-tauri/src/lib.rs}`, zero overlap, and their exact 20-path union. It then classifies that union as 19 ownership/move paths plus sole dependent-only `sources/store.rs`; the wiring path is never presented as scanner output. In the Checkpoint 1 layout after Step 5 it requires the live tree to contain exactly 160 references with `error=37` and every other root count unchanged, plus the exact six direct-core replacements; it must not keep asserting the obsolete live count of 166. When later checkpoints physically split named owners, the contract follows the lifecycle path map, keeps the six direct-core paths and 44 app-facade sentinels, and no longer applies the CP1 aggregate to a different file layout.

Require the Task 0 design/roadmap addendum to be present and contractually encode `src-tauri/src/sources/store.rs` as the sole app-only dependent consumer of `get_client`, not a moved owner. Its exact 24-identity set is the separate `$sourcesStoreBroadRegressionTestIds` contract above and is not added to the immutable ownership/move map; Task 5 proves membership again and runs it only as a broad store regression suite. The contract explicitly records that none of those 24 identities invokes `list_telegram_sources` or `add_telegram_source`; direct evidence for their facade rewrite is the existing status-gated runtime lookup identity plus the exact command-body source assertions in Task 5. Discovery of any other raw-client/session consumer stops the plan for amendment.

- [ ] **Step 5: Strengthen identity characterization, then normalize exactly six paths.**

First extend the existing
`sources::types::tests::telegram_message_identity_validation_rejects_invalid_values`
identity so it pins all three exact messages, Validation kind, exact serialized `{"kind","message"}` JSON, and multi-invalid precedence in the declared order. Run it GREEN against the current code.

Then replace exactly:

```text
production:
1 return type crate::error::AppResult
3 crate::error::AppError::validation constructors

test:
2 crate::error::AppErrorKind::Validation assertions
```

with direct `extractum_core::error` paths. The new message/JSON/precedence assertions use direct core paths from birth. Run the same exact identity GREEN again:

```powershell
Invoke-ExactRustTest extractum 'sources::types::tests::telegram_message_identity_validation_rejects_invalid_values'
```

The source contract requires exactly six changed identity-seam occurrences and sentinel-preserves the other 44 known facade references, including app-owned `TelegramSourceKind`/`SourceItemsCursor`, item/peer/sync compression, and Takeout time calls.

- [ ] **Step 6: Install the closed status vocabulary and make Checkpoint 1 truthful.**

Add all five intermediate checkpoint values and the final 8A value to the shell-cap allowlist without weakening any existing Phase 8 assertion. Update:

```text
roadmap: 8A preparation Checkpoint 1 retained
design:  Approved; 8A preparation Checkpoint 1 retained
```

Run:

```powershell
Invoke-CheckedNative 'Telegram boundary contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Phase 8 status contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'Checkpoint 1 format' { cargo fmt --manifest-path src-tauri/Cargo.toml --all }
Invoke-CheckedNative 'Checkpoint 1 app check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 1 app tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

- [ ] **Step 7: Commit the first green checkpoint.**

Verify scope and immutable inputs mechanically:

```powershell
$task1Allowed = @(
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md'
    'src-tauri/src/sources/types.rs'
    'src/lib/crate-extraction-shell-cap-contract.test.ts'
    'src/lib/telegram-contract-paths.ts'
    'src/lib/telegram-crate-boundary-contract.test.ts'
) | Sort-Object
$currentManifestHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.toml').Hash
$currentLockHash = (Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/Cargo.lock').Hash
if ($currentManifestHash -ne $manifestHash) { throw 'Checkpoint 1 changed Cargo.toml' }
if ($currentLockHash -ne $lockHash) { throw 'Checkpoint 1 changed Cargo.lock' }
Assert-ScopedChanges -Label 'Checkpoint 1 scope' -Allowed $task1Allowed -RequireChanges
Invoke-CheckedNative 'Checkpoint 1 diff check' { git diff --check }
Invoke-CheckedNative 'stage Checkpoint 1' {
    git add -- docs/superpowers/specs/2026-07-17-crate-roadmap.md docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md src-tauri/src/sources/types.rs src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/telegram-contract-paths.ts src/lib/telegram-crate-boundary-contract.test.ts
}
$cachedTask1 = @(& git diff --cached --name-only --relative | Sort-Object -Unique)
if (($cachedTask1 -join "`n") -ne ($task1Allowed -join "`n")) {
    throw "Unexpected Checkpoint 1 staged paths: $($cachedTask1 -join ', ')"
}
Invoke-CheckedNative 'Checkpoint 1 cached diff check' { git diff --cached --check }
Invoke-CheckedNative 'commit Checkpoint 1' {
    git commit -m "refactor: freeze Phase 8A Telegram boundary"
}
```

Record the commit SHA for Task 6 evidence.

---

### Task 2: Checkpoint 2 — Characterize Observable Telegram and Takeout Behavior

**Files:**

- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `src-tauri/src/telegram.rs`
- Modify: `src-tauri/src/telegram_session_store.rs`
- Verify only: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/state.rs`
- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Verify only: `src-tauri/src/accounts.rs`
- Verify only: `src-tauri/src/lib.rs`
- Verify only: `src-tauri/src/sources/sync.rs`
- Verify only: `src-tauri/src/takeout_import/{export_dc,pagination,raw_parse,forum_topics}.rs`

**Interfaces:**

- **Consumes:** the clean Checkpoint 1 contract/map and the observable strings, payloads, paths, ordering, errors, and cancellation points in `Frozen Observable Compatibility`.
- **Produces:** exactly three additional executable characterization identities—Telegram event/status, Takeout state/event, and session path/error—and source assertions for all twelve command declarations/registration plus flow ordering.
- **Hands to Task 3:** a package-GREEN Checkpoint 2 in which the current wire/error/path/order behavior is executable evidence; it adds no domain API and changes no production outcome.

- [ ] **Step 1: Pin exact command declarations and registration.**

Start from the clean Checkpoint 1 commit and freeze task-local Cargo identity:

```powershell
Assert-CleanWorktree 'Checkpoint 2 start'
$task2CargoHashes = Get-CargoIdentityHashes
```

Extend the TypeScript contract to extract the complete declaration text—not only names—for the twelve commands in the Frozen Observable Compatibility table. Require each exactly once under `#[tauri::command]` and exactly once in the `tauri::generate_handler!` registration. Pin parameter names/types, return types, Tauri state types, and current camelCase IPC behavior.

The contract also proves that account SQL, credential resolution, cross-domain delete blockers, command registration, and all command functions remain outside `telegram/*.rs`.

- [ ] **Step 2: Add exact event/status/result characterization.**

Add the two exact Rust tests with literal assertions for every value named in the paragraph after this block. Give each test a temporary deliberately wrong expected literal so the exact test fails at runtime, replace only that literal with the current production value, and then run:

```powershell
Assert-ExactRustRuntimeRed extractum `
  'telegram::tests::telegram_status_and_event_payload_contract_is_exact' `
  'src-tauri/src/telegram.rs' `
  'RED: CP2 Telegram status and event payload'
Invoke-ExactRustTest extractum 'telegram::tests::telegram_status_and_event_payload_contract_is_exact'

Assert-ExactRustRuntimeRed extractum `
  'takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact' `
  'src-tauri/src/takeout_import/state.rs' `
  'RED: CP2 Takeout event status cancellation'
Invoke-ExactRustTest extractum 'takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact'
```

Pin all event/status strings in the compatibility table, payload field names/serde casing/optional behavior, `"Code sent"`, boolean command results, best-effort emit behavior, and ordering relative to state changes. The Takeout state test pins cancellation state transitions and terminal status selection; the TypeScript source contract pins the unchanged checks before/after remote steps plus warning/provenance/event order without introducing a fake provider abstraction.

- [ ] **Step 3: Add exact path/session/error characterization.**

Extract the existing expression into one private non-`cfg` helper `fn session_temp_path(path: &std::path::Path) -> std::path::PathBuf` whose body is exactly `path.with_extension("session.json.tmp")`; production save code and the test both call that same helper, so characterization cannot fork from production. Add literal assertions for the service/key IDs, filename, envelope/AAD, and exact `AppError` kind/message/JSON values named below; seed the runtime RED with one wrong expected temp path, correct it to the existing value, then run:

```powershell
Assert-ExactRustRuntimeRed extractum `
  'telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact' `
  'src-tauri/src/telegram_session_store.rs' `
  'RED: CP2 session path and error contract'
Invoke-ExactRustTest extractum 'telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact'
```

Pin service/key IDs, filename, temp extension, write-before-rename and delete-file-before-delete-key ordering, envelope fields/version/algorithm/base64/AAD, every exact current codec error string/kind/JSON, missing-key Auth text, and legacy compensation behavior. This step characterizes; it does not repair the current rename primitive.

- [ ] **Step 4: Freeze message/page/fallback and fetch/persist boundaries.**

The source contract and existing deterministic tests together pin:

- live sync limit/date/range selection, message order, fallback identity, and durable persist/finalize order;
- Takeout range/page cursor order, TDesktop fallback, `only_my_messages`, channel-private fallback, raw identity, incremental persistence, cancellation, warning/provenance, and finalization;
- the two raw-TL/SQL mixed baselines before their later 8B decomposition.

Run exact/non-empty witnesses:

```powershell
Invoke-ExactRustTest extractum 'sources::sync::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary'
Invoke-ExactRustTest extractum 'takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future'
Invoke-ExactRustTest extractum 'takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future'
Invoke-ExactRustTest extractum 'takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation'
Invoke-ExactRustTest extractum 'takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues'
Invoke-ExactRustTest extractum 'takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers'
Invoke-ExactRustTest extractum 'takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once'
Invoke-NonEmptyRustSuite -Label 'Takeout pagination characterization' -Package extractum -TestFilter 'takeout_import::pagination::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout raw parse characterization' -Package extractum -TestFilter 'takeout_import::raw_parse::tests::'
```

- [ ] **Step 5: Run the checkpoint, advance status, and commit.**

Run the Telegram boundary/status contracts, format, `-p extractum` check, and all-target tests. Only after GREEN update both status authorities to Checkpoint 2. Then verify the new state before staging:

```powershell
Invoke-CheckedNative 'Checkpoint 2 Telegram boundary contract after status update' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Checkpoint 2 Phase 8 status contract after status update' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
```

Verify manifest/lock byte identity, inspect the complete Task 2 diff, and commit:

```powershell
$task2Allowed = @(
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md'
    'src-tauri/src/takeout_import/state.rs'
    'src-tauri/src/telegram.rs'
    'src-tauri/src/telegram_session_store.rs'
    'src/lib/crate-extraction-shell-cap-contract.test.ts'
    'src/lib/telegram-crate-boundary-contract.test.ts'
)
$task2Commit = @{
    Label = 'Checkpoint 2'
    Allowed = $task2Allowed
    StartingCargoHashes = $task2CargoHashes
    Message = 'test: characterize Phase 8A Telegram behavior'
}
Commit-ScopedCheckpoint @task2Commit
```

Record the SHA.

---

### Task 3: Checkpoint 3 — Establish DTO, Media, Identity, and Persistence Ownership

**Files:**

- Create: `src-tauri/src/telegram/dto.rs`
- Create: `src-tauri/src/telegram/media.rs`
- Modify: `src-tauri/src/telegram.rs`
- Modify: `src-tauri/src/media.rs`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/raw_parse.rs`
- Modify: `src-tauri/src/takeout_import/migrated_history.rs`
- Modify: `src-tauri/src/ingest_provenance.rs`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Verify only: `src/lib/media-metadata-core-contract.test.ts`

**Interfaces:**

- **Consumes:** the clean Checkpoint 2 behavior contract and the exact DTO/media signatures, visibilities, facade allowlist, persistence signatures, and ownership table frozen above.
- **Produces:** private `telegram::{dto,media}` leaves; the sole `TelegramMessageDraft`, `TelegramMediaPayload`, identity/item-kind owners; temporary peer-kind facade re-exports; and app persistence functions accepting the draft directly.
- **Hands to Task 4:** the exact curated `crate::telegram` DTO/media surface, 140 baseline primaries plus the one item-kind companion, removed legacy insert/payload structs, and a package-GREEN Checkpoint 3.

- [ ] **Step 1: Add the minimum compiling DTO/media skeleton before runtime RED.**

Start from the clean Checkpoint 2 commit and freeze task-local Cargo identity:

```powershell
Assert-CleanWorktree 'Checkpoint 3 start'
$task3CargoHashes = Get-CargoIdentityHashes
```

Declare the exact types/visibility from Frozen Phase 8A API under private `telegram::{dto,media}` leaves and curated facade re-exports. Start with constructors/default values sufficient to compile existing code; do not create a mirror DTO or conversion-only layer.

- [ ] **Step 2: RED/GREEN the single draft and item-kind contracts.**

```powershell
Assert-ExactRustRuntimeRed extractum `
  'telegram::dto::tests::telegram_message_draft_has_single_persistence_shape' `
  'src-tauri/src/telegram/dto.rs' `
  'RED: CP3 single Telegram draft shape'
Invoke-ExactRustTest extractum 'telegram::dto::tests::telegram_message_draft_has_single_persistence_shape'

Assert-ExactRustRuntimeRed extractum `
  'telegram::dto::tests::telegram_item_kind_constant_matches_persisted_wire_value' `
  'src-tauri/src/telegram/dto.rs' `
  'RED: CP3 Telegram item kind owner'
Invoke-ExactRustTest extractum 'telegram::dto::tests::telegram_item_kind_constant_matches_persisted_wire_value'
```

The app baseline `sources::types::tests::item_kind_constants_match_persisted_wire_values` remains executable and asserts only YouTube transcript/comment values.

- [ ] **Step 3: Move identity behavior without changing it.**

Move `TelegramMessageIdentity`, its five fields, three crate-private peer-kind constants, `validate`, and its one baseline test into `telegram/dto.rs`. Temporarily re-export `TELEGRAM_PEER_KIND_{CHANNEL,CHAT,USER}` through the curated `crate::telegram` facade and change `sources/sync.rs` so `fallback_message_identity` imports those three constants from `crate::telegram::{...}`; it must not retain a second literal owner or import them from `sources::types`. The moved test's Phase 8A exact identity is:

```powershell
Invoke-ExactRustTest extractum 'telegram::dto::tests::telegram_message_identity_validation_rejects_invalid_values'
```

The six direct `extractum_core::error` paths and all Task 1 message/kind/JSON/precedence assertions must be byte-equivalent apart from module/import movement. Update `ingest_provenance.rs` and `takeout_import/migrated_history.rs` only to consume the curated facade values. The TypeScript boundary contract requires exactly these three temporary peer-kind re-exports in 8A and requires their removal in 8B when `fallback_message_identity` moves to `live/messages.rs`; they are not part of the eventual public crate API.

- [ ] **Step 4: Move the single media owner and its two baseline tests.**

Move/rename `ExtractedMediaPayload -> TelegramMediaPayload`, absorb `ExtractedItemPayload` fields into the draft, and move `DocumentSignals`, content constants/classifiers, Grammers adapters, and exact tests to `telegram/media.rs`. Keep `src-tauri/src/media.rs` as the private provider-neutral core compatibility facade plus only explicitly needed Telegram re-exports.

Do not replace `ExtractedItemPayload` with another named content DTO. Until 8B moves complete message adaptation, the private Grammers helper returns only this unnamed construction tuple:

```rust
pub(crate) fn extract_item_payload(
    message: &grammers_client::message::Message,
) -> Option<(
    Option<String>,
    &'static str,
    Option<TelegramMediaPayload>,
)>;
```

The live caller destructures it immediately into the one `TelegramMessageDraft`. The tuple is private, never re-exported, and is removed/reworked inside the later final-owner `live/messages.rs`.

```powershell
Invoke-ExactRustTest extractum 'telegram::media::tests::derive_content_kind_tracks_text_and_media_presence'
Invoke-ExactRustTest extractum 'telegram::media::tests::derive_document_media_kind_prefers_specific_signals'
Invoke-CheckedNative 'media core standing contract' {
    npm.cmd run test -- src/lib/media-metadata-core-contract.test.ts
}
```

- [ ] **Step 5: Make app persistence accept `TelegramMessageDraft` directly.**

Use these exact app-owned signatures:

```rust
fn prepare_source_item(
    draft: &crate::telegram::TelegramMessageDraft,
) -> extractum_core::error::AppResult<Option<PreparedSourceItem>>;

pub(crate) async fn insert_telegram_source_item(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    draft: crate::telegram::TelegramMessageDraft,
) -> extractum_core::error::AppResult<bool>;

pub(crate) async fn insert_telegram_source_item_outcome(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    draft: crate::telegram::TelegramMessageDraft,
) -> extractum_core::error::AppResult<TelegramItemInsertOutcome>;

pub(crate) async fn insert_telegram_source_item_with_observation(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    source_id: i64,
    draft: crate::telegram::TelegramMessageDraft,
) -> extractum_core::error::AppResult<TelegramItemInsertOutcome>;
```

Apply the same identity-in-draft rule to the existing `_in_context` and `_on_connection` private variants. Validate/extract `draft.telegram_identity` at the existing boundary; preserve exact missing-identity behavior and every transaction/order/outcome assertion. Update live/raw constructors to populate it. Migrated history changes the draft identity field before persistence rather than passing a second identity parameter.

- [ ] **Step 6: RED the forbidden old seam, remove it, and retain its assertions.**

Add the TypeScript source assertion first and prove it RED on the exact forbidden symbols:

```text
struct SourceItemInsert
struct ExtractedItemPayload
fn insert_source_item
#[allow(dead_code)] attached to that function/field
TelegramMessageDraft.external_id
```

Then remove them and make the source assertion GREEN. Rename and rewrite the baseline test to:

```powershell
Invoke-ExactRustTest extractum 'sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates'
```

It must exercise the retained production Telegram insert path and preserve the prior payload/duplicate assertions. Do not keep the old generic function under another name.

- [ ] **Step 7: Reconcile the complete map and run checkpoint regressions.**

Require current lifecycle counts:

```text
baseline primaries still represented: 140
current companion identities:         1
current mapped union:                 141
unclassified/deleted baseline rows:   0
```

Run:

```powershell
Invoke-NonEmptyRustSuite -Label 'DTO tests' -Package extractum -TestFilter 'telegram::dto::tests::'
Invoke-NonEmptyRustSuite -Label 'media tests' -Package extractum -TestFilter 'telegram::media::tests::'
Invoke-NonEmptyRustSuite -Label 'Telegram item persistence tests' -Package extractum -TestFilter 'sources::items::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout raw parse tests' -Package extractum -TestFilter 'takeout_import::raw_parse::tests::'
Invoke-CheckedNative 'Telegram boundary contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts src/lib/media-metadata-core-contract.test.ts
}
```

- [ ] **Step 8: Run the checkpoint, advance status, and commit.**

Format, run `-p extractum` check/tests, update both status authorities to Checkpoint 3 only after GREEN, then verify the new state before staging:

```powershell
Invoke-CheckedNative 'Checkpoint 3 Telegram boundary contract after status update' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Checkpoint 3 Phase 8 status contract after status update' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
```

Prove manifest/lock byte identity, inspect the complete Task 3 diff, and commit:

```powershell
$task3Allowed = @(
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md'
    'src-tauri/src/ingest_provenance.rs'
    'src-tauri/src/media.rs'
    'src-tauri/src/sources/items.rs'
    'src-tauri/src/sources/mod.rs'
    'src-tauri/src/sources/sync.rs'
    'src-tauri/src/sources/types.rs'
    'src-tauri/src/takeout_import/migrated_history.rs'
    'src-tauri/src/takeout_import/mod.rs'
    'src-tauri/src/takeout_import/raw_parse.rs'
    'src-tauri/src/telegram.rs'
    'src-tauri/src/telegram/dto.rs'
    'src-tauri/src/telegram/media.rs'
    'src/lib/crate-extraction-shell-cap-contract.test.ts'
    'src/lib/telegram-crate-boundary-contract.test.ts'
)
$task3Commit = @{
    Label = 'Checkpoint 3'
    Allowed = $task3Allowed
    StartingCargoHashes = $task3CargoHashes
    Message = 'refactor: prepare Telegram DTO and media ownership'
}
Commit-ScopedCheckpoint @task3Commit
```

Record the SHA.

---

### Task 4: Checkpoint 4 — Separate Session Codec and Secret-Key Ownership

**Files:**

- Create: `src-tauri/src/telegram/session.rs`
- Modify: `src-tauri/src/telegram.rs`
- Modify: `src-tauri/src/telegram_session_store.rs`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`

**Interfaces:**

- **Consumes:** the clean Checkpoint 3 facade and the exact `TelegramSession`, `SessionEncryptionKey`, codec signatures, envelope/AAD constants, and adapter ordering frozen above.
- **Produces:** private `telegram::session`; an opaque session; a secret key without a plaintext getter; pure classify/decode/encode operations; and the existing app path/keyring/file adapter reduced to orchestration.
- **Hands to Task 5:** the curated session/key facade, its sole `pub(super) raw_memory_session` sibling accessor, moved codec primaries, exact app-owned failure compensation, and a package-GREEN Checkpoint 4.

- [ ] **Step 1: Add minimum compiling opaque session/key signatures.**

Start from the clean Checkpoint 3 commit and freeze task-local Cargo identity:

```powershell
Assert-CleanWorktree 'Checkpoint 4 start'
$task4CargoHashes = Get-CargoIdentityHashes
```

Add `TelegramSession`, its sole `pub(super) raw_memory_session` leaf-internal accessor, `SessionEncryptionKey`, `try_from_encoded`, `generate`, `session_json_requires_existing_key`, `decode_session_json`, and `encode_session_json` exactly as frozen above. No app-visible raw session adapter exists in CP4; the TypeScript contract permits only this one sibling-runtime accessor and rejects it at `pub(crate)` or `pub`.

- [ ] **Step 2: RED/GREEN key construction and write-only generation.**

```powershell
Assert-ExactRustRuntimeRed extractum `
  'telegram::session::tests::session_encryption_key_rejects_invalid_length' `
  'src-tauri/src/telegram/session.rs' `
  'RED: CP4 invalid session key length'
Invoke-ExactRustTest extractum 'telegram::session::tests::session_encryption_key_rejects_invalid_length'

Assert-ExactRustRuntimeRed extractum `
  'telegram::session::tests::generated_session_key_returns_write_only_encoded_secret' `
  'src-tauri/src/telegram/session.rs' `
  'RED: CP4 generated session key'
Invoke-ExactRustTest extractum 'telegram::session::tests::generated_session_key_returns_write_only_encoded_secret'
```

Invalid length preserves exact Internal message `Invalid Telegram session key length`. Source contracts reject secret-bearing `Debug`, public fields, a wrapper `ExposeSecret` implementation, `as_bytes`, or another plaintext getter. They permit only the private module's narrowly scoped internal `expose_secret()` calls at the declared crypto sinks.

- [ ] **Step 3: Move codec-subject baseline tests and make the codec GREEN.**

Move the three baseline primaries to their Phase 8A identities:

```powershell
Invoke-ExactRustTest extractum 'telegram::session::tests::saving_session_writes_encrypted_envelope_not_plaintext'
Invoke-ExactRustTest extractum 'telegram::session::tests::encrypted_session_load_round_trips'
Invoke-ExactRustTest extractum 'telegram::session::tests::encrypted_session_load_fails_for_wrong_account_id'
```

Rewrite only their fixture setup from file/keyring helpers to the pure codec. Preserve their subjects and final leaf names. The canonical envelope, AAD, base64, roundtrip, and exact wrong-account decryption error remain unchanged.

- [ ] **Step 4: RED/GREEN legacy and missing-key decisions.**

```powershell
Assert-ExactRustRuntimeRed extractum `
  'telegram::session::tests::legacy_json_returns_rewrite_decision' `
  'src-tauri/src/telegram/session.rs' `
  'RED: CP4 legacy rewrite decision'
Invoke-ExactRustTest extractum 'telegram::session::tests::legacy_json_returns_rewrite_decision'

Assert-ExactRustRuntimeRed extractum `
  'telegram::session::tests::missing_encrypted_key_preserves_auth_error' `
  'src-tauri/src/telegram/session.rs' `
  'RED: CP4 missing encrypted session key'
Invoke-ExactRustTest extractum 'telegram::session::tests::missing_encrypted_key_preserves_auth_error'
```

Format classification returns `true` for encrypted and `false` for legacy. Unsupported/malformed input and every error string/kind/JSON remain Task 2 exact values.

- [ ] **Step 5: Reduce `telegram_session_store.rs` to the app adapter.**

Keep path lookup, existence, SecretStore calls, temp write/rename, delete ordering, and legacy compensation in the app. It classifies before secure-store access, calls the pure codec, and preserves:

```text
encrypted: read file -> classify -> read existing key -> decode
legacy:    read file -> classify -> decode legacy -> read existing key;
           if absent generate/store key -> encode canonical envelope ->
           temp write -> rename
save:      read/generate/store key -> encode -> temp write -> rename
delete:    remove/ignore missing file -> delete key
```

Run the four app-owned baseline primaries:

```powershell
Invoke-ExactRustTest extractum 'telegram_session_store::tests::delete_session_from_path_removes_file_and_key'
Invoke-ExactRustTest extractum 'telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing'
Invoke-ExactRustTest extractum 'telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file'
Invoke-ExactRustTest extractum 'telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails'
Invoke-ExactRustTest extractum 'telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact'
```

- [ ] **Step 6: Reconcile identities and enforce the secret/public-API contract.**

The current baseline representation remains 140; item-kind companion remains one; the three moved codec primaries now resolve under `telegram::session::tests`. Assert no raw session/secret type is public and no generic SecretStore/keyring/path/fs symbol entered `telegram/session.rs`.

- [ ] **Step 7: Run the checkpoint, advance status, and commit.**

Run non-empty session/app suites, boundary/status contracts, format, `-p extractum` check/tests. Only after GREEN update both status authorities to Checkpoint 4. Then verify the new state before staging:

```powershell
Invoke-CheckedNative 'Checkpoint 4 Telegram boundary contract after status update' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Checkpoint 4 Phase 8 status contract after status update' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
```

Prove manifest/lock identity, inspect the complete Task 4 diff, and commit:

```powershell
$task4Allowed = @(
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md'
    'src-tauri/src/telegram.rs'
    'src-tauri/src/telegram/session.rs'
    'src-tauri/src/telegram_session_store.rs'
    'src/lib/crate-extraction-shell-cap-contract.test.ts'
    'src/lib/telegram-crate-boundary-contract.test.ts'
)
$task4Commit = @{
    Label = 'Checkpoint 4'
    Allowed = $task4Allowed
    StartingCargoHashes = $task4CargoHashes
    Message = 'refactor: isolate Telegram session codec'
}
Commit-ScopedCheckpoint @task4Commit
```

Record the SHA.

---

### Task 5: Checkpoint 5 — Introduce the Opaque Runtime and Login Seam

**Files:**

- Create: `src-tauri/src/telegram/runtime.rs`
- Modify: `src-tauri/src/telegram.rs`
- Verify only: `src-tauri/src/telegram_session_store.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Verify only: `src-tauri/src/accounts.rs`
- Verify only: `src-tauri/src/diagnostics/runtime.rs`
- Modify: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Verify only: `src-tauri/src/lib.rs`

**Interfaces:**

- **Consumes:** the clean Checkpoint 4 opaque session/key API, the exact runtime/public/private signatures and concurrency contract frozen above, and the Task 2 command/event/error evidence.
- **Produces:** private `telegram::runtime`; opaque runtime/client/login types; the exact `cfg(test)` callback seam; two temporary app lookup functions; two `pub(crate)` raw handle adapters; and rewired dependent consumers.
- **Hands to Task 6:** a package-GREEN Checkpoint 5 with the same global-mutex/network-await ordering, raw-access allowlist, 158 post-8A tracked identities, unchanged app commands/events, and no crate/package-graph change.

- [ ] **Step 1: Add the minimum compiling runtime types and private test seam.**

Start from the clean Checkpoint 4 commit and freeze task-local Cargo identity:

```powershell
Assert-CleanWorktree 'Checkpoint 5 start'
$task5CargoHashes = Get-CargoIdentityHashes
```

Add `TelegramApiHash`, `TelegramRuntimeStatus`, `TelegramClientHandle`, `TelegramLoginAttempt`, and `TelegramRuntime` signatures exactly as frozen; consume the already-GREEN opaque `TelegramSession` from CP4 without redefining it. `TelegramLoginAttempt` remains a public opaque allowlisted type with private fields even though the runtime, not the app, stores it. It is never returned merely to justify its visibility.

Implement `TelegramClientInner`, `TelegramLoginAttemptToken`, `TelegramRuntimeTestFuture`, `TelegramRuntimeTestCallbacks`, `TelegramRuntimeTestRunnerDropProbe`, `spawn_pending_runner`, `TelegramRuntime::with_test_callbacks`, and the `cfg(test)` runtime field exactly as frozen above. The fake constructor initializes an empty account map and stores the callbacks. The production constructor initializes the same empty map and contains no callback field after cfg-elision. Production code directly owns Grammers; do not add a public or production-generic transport trait.

- [ ] **Step 2: RED/GREEN authorization and missing-account behavior.**

```powershell
Assert-ExactRustRuntimeRed extractum `
  'telegram::runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner' `
  'src-tauri/src/telegram/runtime.rs' `
  'RED: CP5 initialization ordering'
Invoke-ExactRustTest extractum 'telegram::runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner'

Assert-ExactRustRuntimeRed extractum `
  'telegram::runtime::tests::missing_account_authentication_is_false' `
  'src-tauri/src/telegram/runtime.rs' `
  'RED: CP5 missing account authentication'
Invoke-ExactRustTest extractum 'telegram::runtime::tests::missing_account_authentication_is_false'
```

For the first test, use callback barriers to prepare two initializations concurrently with distinct `TelegramSession` values, release them in the opposite order, and prove that the last `accounts.insert` wins. Call `runtime.initialized_client(account_id).await` directly with no externally held map guard, compare the returned handle's session with `Arc::ptr_eq` inside the private module, then call it for a missing ID and assert exact Auth text `Account {account_id} not initialized`. This is the direct executable lookup proof; the Task 5 source contract separately proves that both store commands relinquish their old caller-held locks. Also assert `Ready` and `ReauthRequired` mapping and prove the replaced runner was detached because its drop probe has not fired in the assertion window. The TypeScript source contract separately pins the existing app-facade initialization-error branch: it clears whichever map entry exists when the error is handled before recording/emitting `restore_failed`; this plan characterizes that race and does not improve it.

- [ ] **Step 3: RED/GREEN login-attempt state transitions one at a time.**

Run each runtime RED, implement only that case, then exact GREEN before adding the next:

```powershell
Assert-ExactRustRuntimeRed extractum 'telegram::runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt' 'src-tauri/src/telegram/runtime.rs' 'RED: CP5 serialized login attempts'
Invoke-ExactRustTest extractum 'telegram::runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt'

Assert-ExactRustRuntimeRed extractum 'telegram::runtime::tests::sign_in_without_code_request_preserves_auth_error' 'src-tauri/src/telegram/runtime.rs' 'RED: CP5 missing login attempt'
Invoke-ExactRustTest extractum 'telegram::runtime::tests::sign_in_without_code_request_preserves_auth_error'

Assert-ExactRustRuntimeRed extractum 'telegram::runtime::tests::failed_sign_in_retains_pending_attempt' 'src-tauri/src/telegram/runtime.rs' 'RED: CP5 retain failed login attempt'
Invoke-ExactRustTest extractum 'telegram::runtime::tests::failed_sign_in_retains_pending_attempt'

Assert-ExactRustRuntimeRed extractum 'telegram::runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt' 'src-tauri/src/telegram/runtime.rs' 'RED: CP5 serialized successful sign in'
Invoke-ExactRustTest extractum 'telegram::runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt'
```

The request test uses three calls: `A1` enters its callback and blocks; `B1` targets a different account and cannot enter while `A1` owns the global lock; after `A1` and `B1` complete, `A2` targets the first account, and a subsequent sign-in for A proves that the retained marker/phone came from `A2`, not `A1`. The successful-sign-in test has two deterministic subcases under the same identity: first, sign in without a concurrent clear, verify the returned session, and call sign-in again to observe exact `Call tg_send_code first`; second, use a fresh runtime, block sign-in, queue clear, prove that neither sign-out nor runner abort begins early, release sign-in, and then prove clear removed the account. Preserve exact `Account not initialized`, network mapping, failed-request and failed-sign-in retention, and successful-sign-in clearing.

- [ ] **Step 4: RED/GREEN cleanup and authorized capability behavior.**

```powershell
Assert-ExactRustRuntimeRed extractum 'telegram::runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure' 'src-tauri/src/telegram/runtime.rs' 'RED: CP5 runtime clear ordering'
Invoke-ExactRustTest extractum 'telegram::runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure'

Assert-ExactRustRuntimeRed extractum 'telegram::runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors' 'src-tauri/src/telegram/runtime.rs' 'RED: CP5 authorized client'
Invoke-ExactRustTest extractum 'telegram::runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors'
```

Block a code request, queue clear, and prove clear neither signs out nor drops the runner future before the request finishes. After release, prove clear removes the new attempt, ignores sign-out failure, and causes the actual pending-runner drop probe to fire only after sign-out completion. In the same exact test, queue a request after clear releases the lock and assert exact `Account not initialized`; a missing clear remains a no-op. Authorized lookup preserves exact missing-account and unauthenticated error strings/kinds and performs its callback after releasing the map lock.

- [ ] **Step 5: Wire the private app facade without changing commands/events.**

`TelegramState` owns `TelegramRuntime` plus the existing app status map. App commands resolve SQL/secrets/files, construct secret/session wrappers, call runtime operations, map only `Ready|ReauthRequired` to exact strings, persist returned sessions, update status, and emit in the current order.

Split the current SQL row from resolved credentials so legacy `accounts.api_hash` is read as an app-local `String`, immediately moved into `secrecy::SecretString`, and thereafter carried only as `TelegramApiHash`. Secure-store reads remain `SecretString`; do not convert them back to ordinary `String`, clone plaintext, log it, or add a getter. The three credential-SQL baseline identities must remain GREEN and app-owned.

Install the root-facade `get_client(&TelegramState, account_id)` and `get_authorized_client(&TelegramState, account_id)` signatures exactly. The former is a deliberate signature/ownership rewrite and `pub -> pub(crate)` visibility narrowing, delegates to `initialized_client` without an authorization probe, and owns no second lock outside the runtime. The latter replaces `get_authorized_runtime` and delegates to `authorized_client`. This distinction is observable error timing and is not collapsed. The boundary contract requires zero `AuthorizedTelegramRuntime` and zero `get_authorized_runtime` declarations/imports/calls from Checkpoint 5 onward.

Add:

```powershell
Assert-ExactRustRuntimeRed extractum `
  'telegram::tests::runtime_status_maps_to_existing_wire_strings' `
  'src-tauri/src/telegram.rs' `
  'RED: CP5 app runtime status mapping'
Invoke-ExactRustTest extractum 'telegram::tests::runtime_status_maps_to_existing_wire_strings'
```

Keep `not_initialized`, `restoring`, and `restore_failed` application-owned. Keep all twelve commands and all events app-owned.

- [ ] **Step 6: Adapt current raw consumers through temporary crate-private access only.**

Apply this complete temporary consumer map and no other raw access:

```text
sources/store.rs list/add commands:
    get_client -> TelegramClientHandle::raw_client
sources/sync.rs:
    get_authorized_client -> TelegramClientHandle::raw_client
takeout_import/mod.rs spike/current/migrated paths:
    get_authorized_client -> TelegramClientHandle::{raw_client,raw_session}
telegram/runtime.rs initialization only:
    TelegramSession::raw_memory_session (pub(super), not app-visible)
```

Replace these two exact current caller-locked blocks:

```rust
// current list_telegram_sources — remove in Checkpoint 5
let client = {
    let accounts = state.accounts.lock().await;
    crate::telegram::get_client(&accounts, account_id)
        .await?
        .clone()
};

// current add_telegram_source — remove in Checkpoint 5
let client = {
    let accounts = state.accounts.lock().await;
    crate::telegram::get_client(&accounts, request.account_id)
        .await?
        .clone()
};
```

with these two exact internally locked forms:

```rust
// list_telegram_sources
let client_handle =
    crate::telegram::get_client(state.inner(), account_id).await?;
let client = client_handle.raw_client().clone();

// add_telegram_source
let client_handle =
    crate::telegram::get_client(state.inner(), request.account_id).await?;
let client = client_handle.raw_client().clone();
```

Delete both old blocks completely. The lifecycle-gated TypeScript contract requires exactly these two `get_client(state.inner(), ...)` calls in the named command bodies at Checkpoint 5 and rejects `state.accounts`, `.accounts.lock`, or `get_client(&accounts` in either body. Before Checkpoint 5 it requires the two exact current caller-locked forms above instead, so the contract cannot pass early on the future layout.

For Takeout, rename the private spike helper to `run_export_dc_spike_for_handle` and accept `TelegramClientHandle`, not `AuthorizedTelegramRuntime` or a raw tuple. The export-DC spike, migrated-history import, and current-history import each clone `handle.raw_client()` and `Arc::clone(handle.raw_session())` at the same current ownership point; the spike's cloned session remains the argument to `prepare_export_dc_alias`. `takeout_import/mod.rs` is the sole `raw_session` path and contains exactly three workflow invocations. At Checkpoint 5 the lifecycle contract pins `sources/sync.rs` to exactly one `get_authorized_client(state.inner(), account_id)`, one `raw_client`, and zero `raw_session` calls; it pins Takeout to exactly three authorized lookups, three `raw_client` calls, and three `raw_session` calls distributed only across spike, migrated-history, and current-history workflows. Both store commands consume only `raw_client`; the boundary contract rejects `raw_session` everywhere else. Its lifecycle rule permits the exact baseline legacy-field `#[allow(dead_code)]` only through Checkpoint 4, requires that annotation and `AuthorizedTelegramRuntime` to disappear together at Checkpoint 5, and rejects `#[allow(dead_code)]` on either new adapter.

Clone the current raw client/session at the same ownership points as today. Do not broaden lock scope, but preserve the existing asymmetric rule exactly: initialized/authorized lookup releases the accounts lock before its network authorization await, while request-code, sign-in, and clear/sign-out retain the single global accounts lock across their network awaits. Preserve behavior and run:

```powershell
Invoke-ExactRustTest extractum 'telegram::runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner'
Assert-ExactRustIdentitySet -Package extractum -Prefix 'sources::store::tests::' -Expected $sourcesStoreBroadRegressionTestIds
Invoke-NonEmptyRustSuite -Label 'source store broad regressions' -Package extractum -TestFilter 'sources::store::tests::'
Invoke-NonEmptyRustSuite -Label 'source sync broad regressions' -Package extractum -TestFilter 'sources::sync::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout broad regressions' -Package extractum -TestFilter 'takeout_import::tests::'
```

Do not run the lifecycle-gated Telegram boundary contract before the Checkpoint 5 status update: while the roadmap still truthfully retains Checkpoint 4, that contract must reject the nine Checkpoint 5 identities and future consumer layout introduced by this task. The required first GREEN run for the new layout is the existing post-status invocation in Step 8 after both status authorities advance together.

The boundary contract requires exactly the two `pub(crate)` handle adapters, the one `pub(super)` session accessor, the two app-only lookup functions, and the consumer map above. It rejects any other raw accessor, constructor/conversion, consumer, visibility, raw type re-export, or caller-held lock around either lookup. The exact 24 store, six source-sync, and 17 Takeout identities are broad regression suites, not direct facade/command/lookup/deadlock coverage: none invokes `list_telegram_sources`, `add_telegram_source`, `sync_telegram_source`, `run_takeout_export_dc_spike`, `run_takeout_migrated_history_import`, or `run_takeout_source_import`. The four sync/Takeout substitutions do not change lock discipline because current `get_authorized_runtime` already accepts `&TelegramState` and locks internally; compilation plus the lifecycle-gated source contract—not the filtered-suite labels—prove the type/accessor rewrites. The adapters' and lookup functions' mandatory removal belongs to 8B. Preserve the existing `clear_account_runtime` and `TelegramState::diagnostic_status_counts` compatibility facade so `accounts.rs` and diagnostics remain byte-identical; their SQL/event/aggregation ownership is unchanged.

- [ ] **Step 7: Enforce public API, secrecy, and ownership absence rules.**

The source contract requires:

- exact public items/signatures from this plan and no extra `pub`;
- no public Grammers/TL/raw handle, SQLx, Tauri, keyring, SecretStore, path/fs, or app error facade;
- no public secret-bearing field, secret-bearing `Debug`, plaintext getter, wrapper `ExposeSecret` implementation, or plaintext log/serialization path; narrowly scoped private internal `ExposeSecret::expose_secret` calls are allowed only at the declared encryption/decryption and Grammers-construction sinks;
- no `telegram_impl` path, crate directory, workspace member, path edge, manifest/lock change;
- every runtime/session/live/Takeout implementation remains in package `extractum`;
- all four Grammers roots remain app-owned.

- [ ] **Step 8: Run the checkpoint, advance status, and commit.**

Run every new runtime identity exactly, all existing five `telegram::tests`, non-empty dependent suites, boundary/status contracts, format, `-p extractum` check/tests. Only after GREEN update both status authorities to Checkpoint 5. Then verify the new state before staging:

```powershell
Invoke-CheckedNative 'Checkpoint 5 Telegram boundary contract after status update' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'Checkpoint 5 Phase 8 status contract after status update' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
```

Prove manifest/lock byte identity, inspect the complete Task 5 diff, and commit:

```powershell
$task5Allowed = @(
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md'
    'src-tauri/src/sources/store.rs'
    'src-tauri/src/sources/sync.rs'
    'src-tauri/src/takeout_import/mod.rs'
    'src-tauri/src/telegram.rs'
    'src-tauri/src/telegram/runtime.rs'
    'src/lib/crate-extraction-shell-cap-contract.test.ts'
    'src/lib/telegram-crate-boundary-contract.test.ts'
)
$task5Commit = @{
    Label = 'Checkpoint 5'
    Allowed = $task5Allowed
    StartingCargoHashes = $task5CargoHashes
    Message = 'refactor: prepare opaque Telegram runtime'
}
Commit-ScopedCheckpoint @task5Commit
```

Record the SHA.

---

### Task 6: Run Completion Gates and Record the Retained 8A Disposition

**Files:**

- Create: `docs/superpowers/verification/2026-07-26-extractum-telegram-8a-preparation.md`
- Modify: `docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Verify only: every file changed by Tasks 1–5
- Verify only: `src-tauri/Cargo.toml`
- Verify only: `src-tauri/Cargo.lock`

**Interfaces:**

- **Consumes:** clean Task 0 and Checkpoint 1–5 commits, their exact SHAs/scopes, the five identity metrics, unchanged Cargo hashes, and an initially empty fix ledger.
- **Produces:** the retained verification document, final roadmap/design status, exact shell-cap expectation, all completion-gate evidence, advisory workspace-check duration, and a final docs-only evidence commit.
- **Handoff:** report the final commit SHA outside its own verification document and stop with “8B not authorized”; any correction commit is first added to the fix ledger and re-enters Task 6 at Step 1.

The Task 6 fix ledger is a closed execution record with these exact columns:

```text
sequence | fix_sha | owning_checkpoint | failing_gate | failure_summary |
exact_changed_paths | owning_allowed_set | focused_rerun | checkpoint_rerun
```

Initialize it as `none`. Each fix row must name one Checkpoint 1–5 owner; `exact_changed_paths` must be a non-empty strict subset of that checkpoint's committed `Allowed` array; both rerun columns contain the literal command plus exit/result evidence, not “passed.” A failure spanning two checkpoint scopes is not a fix row: stop for a plan amendment. The verification document records `Fix ledger: none` or every row in sequence and includes every fix SHA in its changed-path and rollback manifest.

The single dated repository-contract correction below is the only exception to the one-owner/strict-subset rule. It cannot be generalized to another failure, path, or checkpoint.

- [ ] **Step 1: Reconcile source, executable identities, and checkpoint SHAs.**

Start from the clean Checkpoint 5 commit when the fix ledger is `none`, or from the clean latest ledger fix commit otherwise, and freeze Task 6 Cargo identity:

```powershell
Assert-CleanWorktree 'Task 6 start'
$task6CargoHashes = Get-CargoIdentityHashes
```

Run the committed map contract, list the current app tests fail-closed, and prove:

```powershell
Assert-ExactRustIdentitySet -Package extractum -Prefix 'sources::store::tests::' -Expected $sourcesStoreBroadRegressionTestIds
```

```text
140 baseline primaries represented once
99 eventual app / 41 eventual crate primary assignments unchanged
one current mandatory companion present (item kind)
141 post-8A baseline-derived identities
17 additional verification identities
158 post-8A tracked identities
two raw-parse companions correctly deferred to 8B
143 eventual baseline-derived identities
160 eventual tracked identities
all plan-added identities present once
no unclassified/deleted baseline
24 dependent-only `sources::store` identities present exactly
authority-synchronization SHA recorded
Checkpoint 1–5 SHAs recorded
```

The verification document records the pre-Task-0 starting HEAD, authority-synchronization SHA, each checkpoint SHA, every fix-ledger row or `Fix ledger: none`, the latest fix SHA (or Checkpoint 5 when there is no fix) as the pre-final evidence HEAD, actual changed path inventory, the synchronized `sources/store.rs` dependent-consumer treatment, and any refreshed non-normative line counts. The final evidence commit SHA cannot be self-recorded inside that commit and belongs only in the post-commit handoff.

- [ ] **Step 2: Prove manifest/lock/package-graph identity.**

Compare both SHA-256 hashes to Task 1 and run locked metadata. Assert:

```text
no extractum-telegram package/member/path dependency
no src-tauri/crates/extractum-telegram path
all four direct Grammers declarations still belong to extractum
no dependency/feature/revision/lock change
```

Record hashes and metadata result in verification.

- [ ] **Step 3: Draft the durable evidence and release/startup disposition before final gates.**

Create the verification document with:

- authority links, pre-Task-0 starting HEAD, authority-synchronization SHA, Checkpoint 1–5 SHAs, the exact fix ledger or `none`, and the latest fix SHA (or Checkpoint 5 when none) as the pre-final evidence HEAD;
- checkpoint rollback order and actual changed path inventory;
- all five identity metrics: 140 baseline, 141 post-8A baseline-derived, 158 post-8A tracked, 143 eventual baseline-derived, and 160 eventual tracked; plus the unchanged 99/41 owner split and exact 18-row/17-additional breakdown;
- exact 43 helper-dependent, three credential-SQL, residual 73, and 21-closure results;
- six-path core normalization proof and 44 sentinel references;
- command/event/status/error/session/secret compatibility results;
- DTO/media/session/runtime ownership and public-API/secrecy scans;
- direct initialized-client lookup evidence; lifecycle-gated proof of two store `get_client` sites, one source-sync authorized/raw-client site, and three Takeout authorized/raw-client/raw-session workflows; explicit classification of the 24/6/17 filtered suites as broad rather than direct facade/command/lookup/deadlock coverage; and old authorized-runtime symbol/legacy dead-code annotation absence;
- manifest/lock hashes and locked metadata;
- explicit no-crate/no-edge/all-Grammers-app-owned statement;
- explicit “Phase 8 incomplete; 8B not authorized” statement;
- this exact release/startup disposition:

```text
Release --no-bundle build: not applicable to 8A by approved design.
Startup smoke: not applicable to 8A by approved design.
Live credentialed Telegram request: intentionally not a gate.
```

Use one explicit dirty-work marker `Completion gates: PENDING in Task 6` in the verification document for the not-yet-run final gate section. It is allowed only during Task 6 and is forbidden from the retained verification file. Do not launch the app or add a process-control harness.

- [ ] **Step 4: Advance the final 8A status before running final-state gates.**

After Steps 1–3 are internally consistent, update:

```text
roadmap: 8A preparation retained
design:  Approved; 8A preparation retained; 8B not started
```

Update the exact shell-cap expectation without removing intermediate/future allowed states. Do not stage or commit yet.

- [ ] **Step 5: Run all completion gates, capture one advisory duration, and finalize evidence.**

Run the exact Task 6 completion block in `## Rust Verification Loops` against the complete Task 6 draft/status tree. Record only its explicitly measured workspace-check duration. A successful value is diagnostic Phase 8A evidence only and does not enter the completed-phase adjacent `>= 15,000 ms` rule. The workspace check repeated internally by `npm.cmd run verify` is a correctness invocation and is not another admitted timing result.

If any correctness gate fails:

An error confined to the still-uncommitted Task 6 evidence/status text, with no change to an approved value or retained checkpoint, is corrected inside the exact Task 6 allowlist and rerun in place; it is not a checkpoint fix commit or ledger row. If the failure exposes a defect in committed Checkpoint 1–5 work, use this procedure:

1. classify one owning Checkpoint 1–5 and capture the failing command/output;
2. copy the dirty Task 6 draft diff and new verification draft to a unique `%TEMP%` recovery directory, then restore only the three exact Task 6 tracked paths to the pre-Task-6 commit and move the new verification file into that recovery directory; require a clean worktree before fixing;
3. edit only a non-empty strict subset of the owning checkpoint's `Allowed` paths and prove the actual path set against that subset;
4. run the owner-compatible current-state verification subset: the exact failing command; every focused exact/non-empty Rust test and non-status contract named by that owning task; `npm.cmd run check:rustfmt`; the Telegram boundary contract and current CP5 shell-cap status contract; `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`; and `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`. Do not replay the historical status mutation, historical status expectation, or checkpoint commit step;
5. re-declare the exact `$task1Allowed` through `$task5Allowed` literals from their checkpoint blocks, set `$fixOwner` to the classified integer, and commit only the observed subset:

```powershell
$checkpointAllowedByNumber = @{
    1 = $task1Allowed
    2 = $task2Allowed
    3 = $task3Allowed
    4 = $task4Allowed
    5 = $task5Allowed
}
if ($fixOwner -notin 1..5) { throw "Invalid fix owner: $fixOwner" }
$fixResult = Commit-ScopedFix `
    -Label "Checkpoint $fixOwner fix" `
    -OwningAllowed $checkpointAllowedByNumber[$fixOwner] `
    -StartingCargoHashes $task6CargoHashes `
    -Message "fix: correct Phase 8A Checkpoint $fixOwner gate"
```

6. append the exact ledger row from `$fixResult.Sha`, `$fixResult.Paths`, the captured failure, and both literal rerun evidences; then restart Task 6 from Step 1 and regenerate final evidence from committed state. The recovery copy is diagnostic scratch, not authority and not reapplied blindly.

Never mix Task 6 status/evidence edits into a fix commit. A failed gate that cannot be assigned to one owner, requires a path outside its `Allowed` set, or changes an approved contract stops execution for a plan amendment. Timing never authorizes a fix or rollback.

#### Observed 2026-07-27: cross-checkpoint repository-contract correction

- [ ] **Authorize only the observed correction.**

The literal `npm.cmd run verify` failed from clean fix HEAD
`69032633fc2346d56e0854475be045cfb3e8a35b` with three retained-contract
defects:

1. `analysis-application-contract.test.ts` still found exactly 38 production
   SQL-consumer records, but two `ingest_provenance.rs` whole-source hashes
   and one `sources/store.rs` whole-source hash were stale;
2. `analysis-crate-boundary-contract.test.ts` expected all-app production
   breadth `121`, observed `125`, and carried the same three stale
   whole-source hashes;
3. the Checkpoint 5 mutation case
   `sync authorized lookup through function-value alias` exceeded Vitest's
   default `5000ms` only under the full-suite load.

The root cause is bounded: Checkpoint 3 added two reachable production files
and changed the import-only `ingest_provenance.rs` source hash, Checkpoint 4
added one reachable file, Checkpoint 5 added one reachable file and changed
an unrelated command body in `sources/store.rs`, and Checkpoint 5 omitted a
local timeout for its heavy mutation table. The SQL record count remains
exactly `38`; no product behavior, SQL text, SQL ownership, migration,
command, or wire contract regressed.

This correction is authorized only after the failed Task 6 draft has been
copied/moved into the unique recovery directory by the procedure above and
the repository has been restored to a clean tree. Commit this plan amendment
alone before editing a contract:

```powershell
$task6RepositoryCorrectionAmendmentAllowed = @(
    'docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md'
)
$task6RepositoryCorrectionCargoHashes = Get-CargoIdentityHashes
$amendmentPaths = @(
    Assert-ScopedChanges `
      -Label 'Task 6 repository-correction amendment' `
      -Allowed $task6RepositoryCorrectionAmendmentAllowed `
      -RequireChanges |
      Sort-Object -Unique
)
if (($amendmentPaths -join "`n") -ne
    (($task6RepositoryCorrectionAmendmentAllowed | Sort-Object) -join "`n")) {
    throw "Repository-correction amendment must change exactly its one plan file"
}
Commit-ScopedCheckpoint `
  -Label 'Task 6 repository-correction amendment' `
  -Allowed $task6RepositoryCorrectionAmendmentAllowed `
  -StartingCargoHashes $task6RepositoryCorrectionCargoHashes `
  -Message 'docs: authorize Phase 8A repository contract correction'
$task6RepositoryCorrectionAmendmentSha = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or !$task6RepositoryCorrectionAmendmentSha) {
    throw 'Could not record repository-correction amendment SHA'
}
```

After this amendment commit, the correction allowlist is exactly these three
test contracts and no other file:

```powershell
$task6RepositoryCorrectionAllowed = @(
    'src/lib/analysis-application-contract.test.ts'
    'src/lib/analysis-crate-boundary-contract.test.ts'
    'src/lib/telegram-crate-boundary-contract.test.ts'
)
Assert-CleanWorktree 'Task 6 repository-contract correction start'
```

No production, documentation, roadmap/design/status, Cargo manifest, or lock
file may change in the correction commit. The Cargo hashes must remain:

```text
src-tauri/Cargo.toml  81A773E6FFB5E4BC1AF7C25D2B3F723424E060A515289B2CB28B7C45360A31FF
src-tauri/Cargo.lock  720E38EA632D7B932B2A23D1481528845EC9304376035B1C851C546EA402E43C
```

- [ ] **Capture the two focused snapshot REDs.**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-application-contract.test.ts -t 'keeps analysis SQL ownership and borrowed coordinator capabilities fail closed'
npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t 'keeps production SQL in the exact six-table owner'
```

Require the first command to exit nonzero with `1 failed | 50 skipped (51)`,
the same `38` expected/actual records, and only the two ingest plus one store
whole-source hash differences below. Require the second to exit nonzero with
`1 failed | 15 skipped (16)` and exact breadth evidence
`expected 125 to be 121`.

- [ ] **Update only the exact analysis snapshots and expose the parser RED.**

Make these literal replacements:

```text
src/lib/analysis-application-contract.test.ts
  ingest source hash, exactly 2 occurrences:
    d938b75a13bb7f4e16e66a570144c3cc5749739433130f87611b96dab7baa0b7
    -> a327cabba5f1ab4f3af5c2f405ccb56a4500dc2bc736d8dc33a220f128323bde
  store source hash, exactly 1 occurrence:
    048b8f5d768f5c8b075505c6f73da4ce5e805c841b57b0bc714ab31a1e1e2a54
    -> e510e682120b6566460fddf405f6c45d31ee7e96b399948fe91b75a51fa4a92d

src/lib/analysis-crate-boundary-contract.test.ts
  all-app production graph breadth, exactly 1 occurrence:
    toBe(121) -> toBe(125)
  ingest source hash, exactly 2 occurrences:
    bfd9f8555486d10a1f4976e59a49e7c87ad01ca0b64662b97891a94bd958f94d
    -> 5a7f28da697b149cf3ba2f9ff01074aa662ea5f6a56c983e8ee24f83816372a2
  store source hash, exactly 1 occurrence:
    830aa5a38060767eeb3c8875abcff435d9c5dfe389be2a9fb9fe886355fb8470
    -> 28a8eb092ae42884e8b06bd6681bc36c8378cb1facea61fd8076b6ed284e3895
```

Rerun the exact analysis-crate focused command. It must remain runtime RED,
now with exact evidence
`telegram/runtime.rs: unmatched Rust opening brace`; a different failure
does not authorize the parser change.

- [ ] **Add the cfg-disabled block-match-arm regression probe, then make it GREEN.**

In the existing `keeps production SQL in the exact six-table owner` case,
place this probe before the production-source traversal:

```typescript
const cfgMatchArmProbe = productionRust(`
  match client {
    TelegramClientInner::Grammers(client) => live(client),
    #[cfg(test)]
    TelegramClientInner::Test { .. } => {
      unreachable!("test-only")
    }
  }
`);
expect(cfgMatchArmProbe).not.toContain("TelegramClientInner::Test");
expect(normalized(cfgMatchArmProbe)).toContain(
  "TelegramClientInner::Grammers(client) => live(client),",
);
expect(() => rustBraceRegions(cfgMatchArmProbe)).not.toThrow();
```

After inserting the probe and before changing `disabledItemEnd`, rerun the
exact analysis-crate focused command and require a nonzero runtime RED from
the cfg-disabled no-trailing-comma match arm. After the helper change, rerun
it and require `1 passed | 15 skipped (16)`.

The valid Rust arm in `src-tauri/src/telegram/runtime.rs` is evidence and must
not change. In `productionRust.disabledItemEnd`, add a
`topLevelMatchArm` boolean. Set it only after encountering `=>` while the
delimiter stack is empty. When the next top-level `{` is the match-arm body,
return the body's `closingDelimiter(...) + 1`, consuming a following
top-level comma when present; for an expression arm, permit its top-level
comma to terminate the disabled arm. The minimal control flow is:

```typescript
let topLevelMatchArm = false;
// inside the scan, before opening-delimiter handling:
if (
  stack.length === 0
  && token === "="
  && syntax[index + 1] === ">"
) {
  topLevelMatchArm = true;
  index += 1;
  continue;
}
// replace the existing top-level braced-item return:
if (
  token === "{"
  && stack.length === 0
  && (bracedItem || topLevelMatchArm)
) {
  const bodyEnd = closingDelimiter(syntax, index, "{", "}") + 1;
  if (!topLevelMatchArm) return bodyEnd;
  let armEnd = bodyEnd;
  while (armEnd < syntax.length && /\s/.test(syntax[armEnd])) armEnd += 1;
  return syntax[armEnd] === "," ? armEnd + 1 : bodyEnd;
}
// widen only the existing top-level comma condition:
if (
  stack.length === 0
  && token === ","
  && (commaTerminated || topLevelMatchArm)
) return index + 1;
```

Do not change cfg field, enum-variant, item, `cfg(any(...))`, or `cfg_attr`
handling. The existing probes for those forms plus the new no-trailing-comma
match-arm probe must all remain GREEN.

- [ ] **Apply the one local Checkpoint 5 timeout.**

The focused command

```powershell
npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts -t 'sync authorized lookup through functi'
```

The unique `functi` prefix is intentional: Vitest 4.1.5 renders the
parameterized `$name` as `sync authorized lookup through functi…`, while the
full phrase selects zero tests. Require exactly
`1 passed | 127 skipped (128)` (observed about `2.2s`); a zero-test filter is
not evidence. This focused case is semantic GREEN, so do not manufacture a
focused RED. The accepted runtime RED is the exact `5000ms` timeout already
captured from the literal full `npm.cmd run verify`. Add only `10_000` as the
third argument of the existing Checkpoint 5 runtime-mutation `it.each` block:

```typescript
-  });
+  }, 10_000);
```

Do not set a file, suite, Vitest, or global timeout.

- [ ] **Run the correction verification sequence with exact counts.**

Run the focused and full analysis contracts:

```powershell
Invoke-CheckedNative 'corrected analysis application focus' {
    npm.cmd run test -- src/lib/analysis-application-contract.test.ts -t 'keeps analysis SQL ownership and borrowed coordinator capabilities fail closed'
}
Invoke-CheckedNative 'corrected analysis crate focus' {
    npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t 'keeps production SQL in the exact six-table owner'
}
Invoke-CheckedNative 'corrected full analysis contracts' {
    npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-crate-boundary-contract.test.ts
}
```

Require `1 passed | 50 skipped (51)`, `1 passed | 15 skipped (16)`, and then
two files with exactly `67 passed`.

Run the full Telegram and current Checkpoint 5 status contracts:

```powershell
Invoke-CheckedNative 'corrected full Telegram boundary' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'current Checkpoint 5 shell-cap contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
```

Require exactly `128 passed` and `8 passed`, respectively. Then run all nine
Task 5 exact Rust identities:

```powershell
$task5RepositoryCorrectionExactIds = @(
    'telegram::runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner'
    'telegram::runtime::tests::missing_account_authentication_is_false'
    'telegram::runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt'
    'telegram::runtime::tests::sign_in_without_code_request_preserves_auth_error'
    'telegram::runtime::tests::failed_sign_in_retains_pending_attempt'
    'telegram::runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt'
    'telegram::runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure'
    'telegram::runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors'
    'telegram::tests::runtime_status_maps_to_existing_wire_strings'
)
foreach ($identity in $task5RepositoryCorrectionExactIds) {
    Invoke-ExactRustTest extractum $identity
}
Assert-ExactRustIdentitySet `
  -Package extractum `
  -Prefix 'sources::store::tests::' `
  -Expected $sourcesStoreBroadRegressionTestIds
```

Require each exact identity to report `1 passed` and `718 filtered out`, and
the store identity assertion to report the exact `24/24` set. Run these four
broad suites and require the exact `7/24/6/17` counts:

```powershell
Invoke-NonEmptyRustSuite -Label 'Telegram app broad regressions' -Package extractum -TestFilter 'telegram::tests::'
Invoke-NonEmptyRustSuite -Label 'source store broad regressions' -Package extractum -TestFilter 'sources::store::tests::'
Invoke-NonEmptyRustSuite -Label 'source sync broad regressions' -Package extractum -TestFilter 'sources::sync::tests::'
Invoke-NonEmptyRustSuite -Label 'Takeout broad regressions' -Package extractum -TestFilter 'takeout_import::tests::'
```

Finish the pre-commit gates:

```powershell
Invoke-CheckedNative 'repository-correction rustfmt' { npm.cmd run check:rustfmt }
Invoke-CheckedNative 'repository-correction Svelte check' { npm.cmd run check }
Invoke-CheckedNative 'repository-correction app check' {
    cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
Invoke-CheckedNative 'repository-correction app tests' {
    cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
}
Invoke-CheckedNative 'repository-correction full Vitest under load' {
    npm.cmd run test
}
```

Require rustfmt clean, Svelte `0 errors and 0 warnings`, Cargo check exit
`0`, app tests `719 passed` plus the zero-test binary, and the full Vitest
load `177 passed` files with exactly `1608 passed` tests. The last command is
the load-bearing GREEN for the local `10_000` timeout.

- [ ] **Commit exactly the three contracts and append special ledger row 2.**

Before staging, require exactly the full `3/3` allowlist, unchanged Cargo
hashes, and both diff checks:

```powershell
$repositoryCorrectionPaths = @(
    Assert-ScopedChanges `
      -Label 'Task 6 repository-contract correction' `
      -Allowed $task6RepositoryCorrectionAllowed `
      -RequireChanges |
      Sort-Object -Unique
)
if (($repositoryCorrectionPaths -join "`n") -ne
    (($task6RepositoryCorrectionAllowed | Sort-Object) -join "`n")) {
    throw "Repository-contract correction must change exactly 3/3 paths"
}
Commit-ScopedCheckpoint `
  -Label 'Task 6 repository-contract correction' `
  -Allowed $task6RepositoryCorrectionAllowed `
  -StartingCargoHashes $task6RepositoryCorrectionCargoHashes `
  -Message 'test: reconcile Phase 8A repository contracts'
$task6RepositoryCorrectionSha = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or !$task6RepositoryCorrectionSha) {
    throw 'Could not record repository-contract correction SHA'
}
```

Append ledger row `2` with:

```text
owning_checkpoint:
  3-5 (plan-amended repository-contract correction)
owning_allowed_set and exact_changed_paths:
  src/lib/analysis-application-contract.test.ts
  src/lib/analysis-crate-boundary-contract.test.ts
  src/lib/telegram-crate-boundary-contract.test.ts
failing_gate:
  npm.cmd run verify => nonzero; 38-record 2+1 hash drift; expected/actual
  breadth 121/125; CP5 function-value-alias timeout at 5000ms
failure_summary:
  authorized by $task6RepositoryCorrectionAmendmentSha; CP3 +2/import-only
  ingest hash, CP4 +1, CP5 +1/unrelated store command hash and local-timeout
  gap; no product/SQL behavior regression
focused_rerun:
  npm.cmd run test -- src/lib/analysis-application-contract.test.ts -t
  'keeps analysis SQL ownership and borrowed coordinator capabilities fail
  closed' => exit 0; 1 passed | 50 skipped (51)
  npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t
  'keeps production SQL in the exact six-table owner' => exit 0;
  1 passed | 15 skipped (16)
  npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts -t
  'sync authorized lookup through functi' => exit 0;
  1 passed | 127 skipped (128)
checkpoint_rerun:
  npm.cmd run test -- src/lib/analysis-application-contract.test.ts
  src/lib/analysis-crate-boundary-contract.test.ts => exit 0; 2 files;
  67 passed
  npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts =>
  exit 0; 128 passed
  npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts =>
  exit 0; 8 passed
  cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
  => exit 0; 719 passed plus zero-test binary
  npm.cmd run test => exit 0; 177 files; 1608 passed
```

For this observed trigger only, row 2's exact three-path
`owning_allowed_set` may span Checkpoints 3–5 and
`exact_changed_paths == owning_allowed_set`; this explicitly overrides the
ordinary single-owner/strict-subset rule and nothing else.

Add both `$task6RepositoryCorrectionAmendmentSha` and
`$task6RepositoryCorrectionSha` as separate chronological entries in the
verification document's commit ledger, changed-path/range manifest, and
rollback order. Row 2's `fix_sha` is
`$task6RepositoryCorrectionSha`; its failure summary names the amendment
SHA. Restart Task 6 at Step 1 from the clean correction SHA and regenerate
the draft from committed state rather than reapplying recovery scratch.
Discard the failed attempt's timing: the restarted final-tree completion
block supplies the only recorded duration.

Replace the verification document's sole `PENDING` marker with exact package/workspace/repository gate results and the advisory duration. Confirm that file contains no placeholder. After this evidence edit, do not change any retained file unless a failed final-tree gate requires restarting Task 6.

- [ ] **Step 6: Verify the exact final tree and commit verification.**

Run the state-gated contracts and then the full repository verifier once more after the last evidence edit:

```powershell
Invoke-CheckedNative 'final Telegram boundary contract' {
    npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts
}
Invoke-CheckedNative 'final Phase 8 status contract' {
    npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
}
Invoke-CheckedNative 'final exact-tree repository verify' {
    npm.cmd run verify
}
```

The final verifier's internal workspace check is a correctness recheck, not a measurement sample; do not record another duration. Require the verification document to contain no `PENDING`, `TODO`, or placeholder text. Allow only Task 6 documentation/status files, inspect/stage the exact final diff without further edits, and:

```powershell
$task6Allowed = @(
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
    'docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md'
    'docs/superpowers/verification/2026-07-26-extractum-telegram-8a-preparation.md'
    'src/lib/crate-extraction-shell-cap-contract.test.ts'
)
$task6Commit = @{
    Label = 'Task 6 retained evidence'
    Allowed = $task6Allowed
    StartingCargoHashes = $task6CargoHashes
    Message = 'docs: record retained Phase 8A Telegram preparation'
}
Commit-ScopedCheckpoint @task6Commit
```

Record the final SHA in the handoff only. Do not amend the committed verification document merely to insert its own SHA. Do not write or start an 8B plan without a new explicit owner instruction.

## Pause and Rollback Ladder

- Checkpoints 1–5 are ordinary independently GREEN commits. A pause retains the last completed checkpoint and records its exact roadmap/design status.
- Task 0 is a separately committed durable evidence correction, not a code checkpoint. Retain it through ordinary checkpoint rollback unless a new owner-approved evidence amendment supersedes it.
- The committed verification rollback manifest includes Task 0, Checkpoints 1–5, every ordinary fix SHA with its owner, the repository-correction amendment SHA, repository-correction row 2 SHA, and an explicit `final evidence SHA: handoff-only (not self-recorded)` slot. The post-commit handoff fills that slot outside the document and gives the executable order.
- The canonical reverse order for the observed history is: final evidence SHA → repository-correction row 2 SHA → repository-correction amendment SHA → earlier ordinary fix rows in reverse sequence (currently Fix 1 `69032633fc2346d56e0854475be045cfb3e8a35b`) → Checkpoint 5 → Checkpoint 4 → Checkpoint 3 → Checkpoint 2 → Checkpoint 1. When rolling back below the repository correction, revert row 2 first but retain its separately committed plan amendment as durable authority unless a new owner-approved authority amendment explicitly supersedes it; in that ordinary case the amendment's position in the reverse list is a retained/no-op step, not an implicit revert.
- Any rollback below CP5 then continues through earlier ordinary fix rows in reverse sequence and Checkpoints 5 down through N+1. A fix still needed by retained Checkpoint N or earlier is reapplied/reimplemented against that retained tree under a new SHA and must pass the same owner-compatible current-state subset before the rollback disposition is committed; never retain a CP5-shaped fix above reverted checkpoints.
- A full 8A rollback reverts final evidence, repository-correction row 2, every earlier ordinary fix in reverse sequence, and Checkpoints 5→1. It retains both the repository-correction plan amendment and Task 0 as durable authority unless either is separately superseded; if an owner-approved superseding authority explicitly requires reverting the plan amendment, revert it at its canonical position between row 2 and the earlier ordinary fixes.
- If dirty failed work contains useful evidence, first validate an exact path allowlist and commit only that evidence; never use `git reset --hard`, destructive checkout, or deletion of evidence.
- After rollback, run the last retained checkpoint's exact tests/contracts, `cargo check -p extractum --all-targets`, `cargo test -p extractum --all-targets`, and locked metadata.
- Prove no crate/member/path edge exists, all Grammers roots remain app-owned, manifest/lock match the retained checkpoint, and the worktree is clean.
- Write a separate durable verification disposition. Use `not retained` only when no 8A checkpoint remains; otherwise name the exact last retained checkpoint.
- Never roll back because of advisory timing.

## Final Manual Review

- [ ] Every plan checkbox is resolved or the pause disposition names the exact last retained checkpoint.
- [ ] The literal map still parses as 140 rows / 99 app / 41 crate / three companions with no duplicate baseline or final identity.
- [ ] The three mandatory decompositions and dead-insert test remap are exact.
- [ ] No baseline identity disappeared behind a zero-test filter or deleted source path.
- [ ] The complete retained range contains no migration, frontend runtime/UI, manifest, lockfile, dependency, feature, revision, persisted/wire value, command signature/registration, event name/payload/order, runtime status value, or transaction change. Implementation/correction commits change only the explicitly named test-contract frontend files and the planned roadmap/design authority-status progression; the sole separately committed authority exception is `docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md` from the dated repository-correction amendment, which authorizes no production path, other documentation path, or status change.
- [ ] Public API equals the allowlist; no public raw Grammers/TL/SQLx/Tauri/keyring/secret getter/error conversion exists.
- [ ] DTO/media/session/runtime leaves live under `src-tauri/src/telegram`, not `telegram_impl` or a new crate.
- [ ] App filesystem/secret/SQL/event/Takeout ownership is intact; temporary raw adapters are crate-private and enumerated for 8B removal.
- [ ] Both `sources/store.rs` commands call `get_client(state.inner(), ...)` without an outer accounts lock; the direct runtime lookup identity and lifecycle-gated source contract are GREEN, while the 24 store identities are recorded only as broad regressions.
- [ ] The source-sync and Takeout lifecycle assertions prove the exact 1/3 authorized-lookup and raw-adapter callsites; their six/17 filtered identities are recorded only as broad regressions.
- [ ] `AuthorizedTelegramRuntime`/`get_authorized_runtime` are absent; only the three enumerated Takeout workflows consume `raw_session`, with no dead-code allowance.
- [ ] Verification contains fresh command output, exact checkpoint SHAs, hashes, advisory timing, and truthful incomplete-Phase-8 disposition.
- [ ] Worktree is clean after the final retained commit.
