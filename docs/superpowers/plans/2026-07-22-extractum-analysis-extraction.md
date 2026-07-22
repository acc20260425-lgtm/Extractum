# Extractum Analysis Crate Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the portable analysis domain and all runtime SQL for its six owned tables into `extractum-analysis`, while preserving every command, wire value, transaction, event, scheduling, snapshot, cancellation, and error contract.

**Architecture:** The application remains the Tauri composition root and owns credentials, migrations, source/project/playlist SQL, task spawning, event emission, dev fixtures, and cross-domain transactions. The new crate owns analysis DTOs, state, report/chat execution, trace/snapshot behavior, and SQL over exactly six analysis tables. Preparation is completed as five independently useful green commits; the boundary contract is then committed RED, and the physical extraction is a separate mechanical commit.

**Tech Stack:** Rust 2021, Tauri, Tokio, SQLx/SQLite, `extractum-core`, `extractum-llm`, Vitest/TypeScript source contracts, PowerShell on Windows.

**Authority:** [`2026-07-22-analysis-crate-boundary-design.md`](../specs/2026-07-22-analysis-crate-boundary-design.md), [`2026-07-17-crate-roadmap.md`](../specs/2026-07-17-crate-roadmap.md), and [`2026-07-17-focused-rust-loop-design.md`](../specs/2026-07-17-focused-rust-loop-design.md).

## Global Constraints

- [ ] Start at or after approved commit `38dfaea7d36cba427d9e8f9e4d3401e4be226e64`. Record the actual starting `HEAD`; never roll back to the approval commit.
- [ ] Do not execute this plan until the owner explicitly approves the implementation plan and instructs execution; the approved design alone is not execution authority.
- [ ] Before every commit, run `git status --short`, inspect the diff, and stage only files named by that task. Preserve unrelated user changes.
- [ ] With a clean scoped worktree, run `cargo fmt --manifest-path src-tauri/Cargo.toml --all` before each Rust checkpoint's package check, then inspect the resulting diff; never format unrelated user changes.
- [ ] Use the canonical shared `src-tauri/target`. Do not create an alternate target directory, measurement worktree, linker configuration, or build profile.
- [ ] Do not edit, move, rename, duplicate, or add a production migration. `src-tauri/src/migrations.rs` and `src-tauri/migrations/**` remain app-owned.
- [ ] Do not move `analysis_documents` or any source, project, Telegram, YouTube, NotebookLM, account, diagnostic, or dev-fixture table into the crate.
- [ ] Do not introduce a generic repository, unit of work, service locator, supervisor, runtime abstraction, source crate, SQLite crate, or test-support crate.
- [ ] Preserve the two independent corpus reads. Read A drives synchronous preflight and the later `started/load_items` summary; read B alone is captured, reloaded, chunked, mapped, reduced, and traced.
- [ ] Preserve report profile resolution before run creation and chat profile resolution inside the detached task. Preserve the post-spawn report pool lookup, terminal pool lookup points, and second chat persistence pool lookup.
- [ ] Preserve all 21 analysis commands, three project commands, three dev commands, command registration, camelCase parameters, return shapes, event channels, event ordering, request IDs, `AppError` JSON, and existing persisted/wire values.
- [ ] Do not change `docs/value-registry.md`; Phase 7 introduces no new persisted or wire value.
- [ ] No public submodules, glob exports, public SQL rows, public fields added for cross-crate convenience, secret getter, or public test helper.
- [ ] Production crate SQL may mention only the six-table allowlist defined below. The three named no-live-fallback tests may seed foreign sentinel rows privately; this is not a production exception.
- [ ] Every unexpected need for a new port, dependency, public field, SQL exception, fifth transaction family, or observable behavior change stops the mechanical move. Amend the approved design and a green preparation checkpoint first.
- [ ] Timing is advisory. Never revert or reject a correct boundary because the ordinary workspace check is slower.

### Phase 7 Roadmap Status State Machine

The heading in `docs/superpowers/specs/2026-07-17-crate-roadmap.md` and the exact expectation in `src/lib/crate-extraction-shell-cap-contract.test.ts` move together:

1. `design approved; implementation not started` — starting state.
2. `preparation Checkpoint 1 retained` through `preparation Checkpoint 5 retained` — only after that checkpoint is green and committed.
3. Checkpoint 6 has no retained roadmap state because it is intentionally RED.
4. `done: retained` — only after all correctness, release, startup, inventory, and documentation evidence passes.
5. `not retained` — only with a durable verification disposition after reverting the candidate.

At any pause after Checkpoint 1–5, leave the last truthful retained status. Do not claim a later checkpoint merely because files were edited.

## Rust Verification Loops

Affected packages are `extractum` during Checkpoints 1–5, then both `extractum-analysis` and immediate consumer `extractum` after Checkpoint 7. Lower crates are dependency-contract subjects but are not changed.

Use this helper for every named exact Rust RED/GREEN test so a typo cannot turn `0 tests` into evidence:

```powershell
$ErrorActionPreference = 'Stop'

function Invoke-ExactRustTest {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$TestName
    )

    $output = & cargo test --manifest-path src-tauri/Cargo.toml -p $Package --lib $TestName -- --exact 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | Out-String)
    $text
    if ($exitCode -ne 0) {
        throw "Exact Rust test failed: $Package::$TestName"
    }
    if ($text -notmatch 'running 1 test') {
        throw "Exact Rust test was empty or ambiguous: $Package::$TestName"
    }
}

function Invoke-CheckedNative {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )

    & $Command
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "$Label failed with exit code $exitCode"
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
    $output = & cargo test --manifest-path src-tauri/Cargo.toml -p $Package --lib $TestName -- --exact 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | Out-String)
    $text
    if ($exitCode -eq 0) { throw "Expected exact Rust runtime RED: $Package::$TestName" }
    if ($text -notmatch 'running 1 test') { throw "Exact Rust RED did not execute exactly one test: $Package::$TestName" }
    if ($text -match 'could not compile|error\[E[0-9]+\]') { throw "Exact Rust RED was a compile failure: $Package::$TestName" }
    if ($text -notmatch $ExpectedPattern) { throw "Unexpected Rust runtime RED reason: $Package::$TestName" }
}

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
    if ($changed.Count -ne 0) { throw "$Label requires a clean worktree; found: $($changed -join ', ')" }
}

function Assert-ScopedChanges {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string[]]$Allowed,
        [switch]$RequireChanges
    )
    $changed = @(Get-ChangedPaths)
    $unexpected = @($changed | Where-Object { $_ -notin $Allowed })
    if ($unexpected.Count -ne 0) { throw "$Label has out-of-scope paths: $($unexpected -join ', ')" }
    if ($RequireChanges -and $changed.Count -eq 0) { throw "$Label produced no changed paths" }
    $changed
}

function Add-ScopedChanges {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string[]]$Allowed
    )
    Invoke-CheckedNative "status before $Label" { git status --short }
    $changed = @(Assert-ScopedChanges -Label $Label -Allowed $Allowed -RequireChanges)
    Invoke-CheckedNative "stage $Label" { git add -- $changed }
    $remainingUnstaged = @(& git diff --name-only --relative)
    if ($LASTEXITCODE -ne 0) { throw "Could not verify unstaged paths for $Label" }
    $remainingUntracked = @(& git ls-files --others --exclude-standard)
    if ($LASTEXITCODE -ne 0) { throw "Could not verify untracked paths for $Label" }
    if (@($remainingUnstaged + $remainingUntracked | Where-Object { $_ }).Count -ne 0) {
        throw "$Label was not staged completely"
    }
    $cached = @(& git diff --cached --name-only --relative)
    if ($LASTEXITCODE -ne 0) { throw "Could not verify cached paths for $Label" }
    $unexpectedCached = @($cached | Where-Object { $_ -notin $Allowed })
    if ($unexpectedCached.Count -ne 0) { throw "$Label cached out-of-scope paths: $($unexpectedCached -join ', ')" }
    if ($cached.Count -eq 0) { throw "$Label has no cached changes" }
}

function Get-Task7AllowedPaths {
    $planPath = 'docs/superpowers/plans/2026-07-22-extractum-analysis-extraction.md'
    $planText = Get-Content -LiteralPath $planPath -Raw
    $task7Start = [regex]::Match($planText, '(?m)^### Task 7:').Index
    $task8Start = [regex]::Match($planText, '(?m)^### Task 8:').Index
    if ($task7Start -lt 0 -or $task8Start -le $task7Start) { throw 'Cannot parse Task 7 scope from the committed plan' }
    $task7Text = $planText.Substring($task7Start, $task8Start - $task7Start)
    $paths = @(
        [regex]::Matches($task7Text, "'((?:src-tauri|src/lib)/[^'`r`n]+)'") |
            ForEach-Object { $_.Groups[1].Value } |
            Where-Object { $_ -match '(\.rs|\.ts|\.toml|Cargo\.lock)$' } |
            Sort-Object -Unique
    )
    # 145 is the frozen unique union of whole-move sources/destinations,
    # split-move sources/destinations, retained app paths, and singletons.
    # Change it only together with those exact arrays and the disposition map.
    if ($paths.Count -ne 145) { throw "Task 7 exact allowlist drifted: expected 145 paths, found $($paths.Count)" }
    $paths
}
```

Run this bootstrap once at the start of every execution session. Every `cargo`, `npm.cmd`, and mutating `git` command in this plan is fail-fast: invoke it through `Invoke-CheckedNative`, except where the step captures output and checks the saved exit code explicitly. Never paste a multi-command block after an unguarded native command. Read-only `git`/`rg` inventory commands may be raw only when no later result or commit depends on their exit code.

### Exact New-Test Identity Map

These tests are additional to Appendix A's frozen 143. Before extraction use the `analysis::` prefix in package `extractum`; after extraction remove that prefix and use package `extractum-analysis`, except for tests explicitly retained in the application. Each task below runs every listed identity with `Invoke-ExactRustTest`; a missing or renamed identity is a checkpoint failure.

| Checkpoint | Pre-move exact identity | Final owner / exact identity |
| --- | --- | --- |
| 1 | `analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels` | app / unchanged pre-move identity |
| 1 | `analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit` | app / unchanged pre-move identity |
| 1 | `analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot` | app / unchanged pre-move identity |
| 1 | `analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id` | app / unchanged pre-move identity |
| 1 | `analysis::chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message` | crate / `chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message` |
| 1 | `analysis::report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails` | crate / `report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails` |
| 1 | `analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads` | app / unchanged pre-move identity |
| 2 | `analysis::report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes` | crate / `report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes` |
| 2 | `analysis::store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes` | crate / `store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes` |
| 2 | `analysis::report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities` | crate / `report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities` |
| 2 | `analysis::report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label` | crate / `report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label` |
| 3 | `analysis::report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads` | crate / `report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads` |
| 3 | `analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure` | crate / `report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure` |
| 3 | `analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure` | crate / `report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure` |
| 4 | `analysis::report::tests::runtime::report_execution_publishes_typed_events_in_existing_order` | crate / `report::tests::runtime::report_execution_publishes_typed_events_in_existing_order` |
| 4 | `analysis::chat::tests::chat_execution_persists_turns_before_completed_event` | crate / `chat::tests::chat_execution_persists_turns_before_completed_event` |
| 4 | `analysis::report::tests::runtime::terminal_cleanup_always_removes_active_report_state` | crate / `report::tests::runtime::terminal_cleanup_always_removes_active_report_state` |
| 4 | `analysis::tests_application::report_profile_resolution_failure_prevents_run_creation` | app / unchanged pre-move identity |
| 5 | `analysis::test_schema::tests::canonical_fixture_applies_analysis_consumed_schema` | crate / `test_schema::tests::canonical_fixture_applies_analysis_consumed_schema` |
| 5 | `analysis::test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys` | crate / `test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys` |

`analysis_wire_contract_serializes_commands_events_and_errors_unchanged` is a Vitest title in `analysis-application-contract.test.ts`, not a Rust identity. `youtube_corpus_mode_parses_wire_values_and_defaults` already exists and remains part of Appendix A; it is rerun rather than counted as new.

During preparation, use the helper with a concrete identity from the task, for example:

```powershell
Invoke-ExactRustTest -Package extractum -TestName 'analysis::trace::tests::decode_trace_data_returns_typed_internal_for_invalid_zstd'
Invoke-CheckedNative 'pre-move app check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'pre-move app tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

After the move, use the owning package and check the app immediately after every public interface change:

```powershell
Invoke-ExactRustTest -Package extractum-analysis -TestName 'trace::tests::decode_trace_data_returns_typed_internal_for_invalid_zstd'
Invoke-ExactRustTest -Package extractum -TestName 'analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows'
Invoke-CheckedNative 'crate check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets }
Invoke-CheckedNative 'crate tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets }
Invoke-CheckedNative 'immediate consumer check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'immediate consumer tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

End-of-slice gates are exactly:

```powershell
Invoke-CheckedNative 'rustfmt gate' { npm.cmd run check:rustfmt }
Invoke-CheckedNative 'workspace check' { cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets }
Invoke-CheckedNative 'workspace tests' { cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets }
Invoke-CheckedNative 'repository verify' { npm.cmd run verify }
```

Only the explicit workspace `cargo check` above is timed once for advisory evidence. `npm.cmd run verify` may internally repeat correctness checks; those are not measurement samples.

---

### Task 1: Checkpoint 1 — Freeze and Characterize the Current Boundary

**Files:**

- Create: `src/lib/analysis-contract-paths.ts`
- Create: `src/lib/analysis-application-contract.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `src-tauri/src/analysis/chat.rs`
- Create: `src-tauri/src/analysis/tests_application.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/report/tests/lifecycle.rs`
- Verify only: `src-tauri/src/projects/mod.rs`
- Verify only: `src-tauri/src/projects/read_model.rs`
- Verify only: `src-tauri/src/account_deletion.rs`
- Verify only: `src-tauri/src/diagnostics/database.rs`
- Verify only: `src-tauri/src/notebooklm_export/query.rs`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`

- [ ] **Step 1: Capture identity and verify the frozen baseline.**

```powershell
$startingCommit = (git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Could not record starting commit' }
Invoke-CheckedNative 'verify approved ancestor' { git merge-base --is-ancestor 38dfaea7d36cba427d9e8f9e4d3401e4be226e64 HEAD }
Assert-CleanWorktree 'Phase 7 start'
$analysisFiles = Get-ChildItem -LiteralPath src-tauri/src/analysis -Recurse -File -Filter '*.rs'
$analysisLines = ($analysisFiles | ForEach-Object { @(Get-Content -LiteralPath $_.FullName).Count } | Measure-Object -Sum).Sum
"files=$($analysisFiles.Count) lines=$analysisLines"
Invoke-CheckedNative 'baseline analysis identity list' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::' -- --list }
```

Expected before new tests: clean worktree, approval commit is an ancestor, `54` Rust files, `13,187` physical lines, and exactly `143` `analysis::` test identities. If source has changed since approval, refresh the non-normative line estimate but do not silently alter Appendix A's 95/48 identity partition.

- [ ] **Step 2: Add a fail-closed dual-owner source reader.**

`analysis-contract-paths.ts` must expose `readAppAnalysisSource(relativePath)`, `readCrateAnalysisSource(relativePath)`, and this manifest-keyed selector:

```ts
type AnalysisContractSource = {
  before: string;
  after: { owner: "app" | "crate"; path: string };
};

readAnalysisContractSource(source: AnalysisContractSource): string;
```

It checks exactly these roots:

```text
pre-move: src-tauri/src/analysis
post-move: src-tauri/crates/extractum-analysis/src
```

Before the analysis manifest exists, the selector reads the explicit `before` app path. After it exists, it reads only the declared final owner/path. It rejects a missing selected file, traversal outside the selected root, an unexpected manifest/layout state, and use of a crate path before extraction. It deliberately permits an approved split to have app and crate files at the same relative path; the final boundary contract separately proves that each behavior/assertion is attached to its one approved owner and that production logic was not copied. `analysis-redesign-safety-contract.test.ts` must use this selector for every raw analysis source assertion so deleting the old path cannot make snapshot-only, chat-persistence, label-precedence, or no-live-fallback protections disappear.

- [ ] **Step 3: Freeze Appendix A against executable output.**

In `analysis-application-contract.test.ts`, parse Appendix A directly from the approved specification instead of copying 143 names into TypeScript. Fail closed on missing headings, duplicate full identities, count drift, malformed bullets, or unexpected current/final prefixes. Assert:

```text
current executable identities: 143 exactly once
approved crate identities:      95 exactly once
approved retained-app identities: 48 exactly once
intersection:                   empty
union:                          current 143
```

New characterization tests added by this plan are counted separately and must not be mistaken for frozen baseline identities.

- [ ] **Step 4: Freeze the command and integration inventory.**

The TypeScript contract must parse command declarations and `generate_handler!` registration and assert these exact sets:

```text
analysis release (21):
list_analysis_sources
list_analysis_runs
list_active_analysis_runs
get_analysis_run
list_analysis_run_messages
get_analysis_run_trace
delete_analysis_run
resolve_analysis_trace_refs
list_analysis_prompt_templates
create_analysis_prompt_template
update_analysis_prompt_template
delete_analysis_prompt_template
list_analysis_source_groups
create_analysis_source_group
update_analysis_source_group
delete_analysis_source_group
list_analysis_chat_messages
clear_analysis_chat_messages
ask_analysis_run_question
start_analysis_report
cancel_analysis_run

project release (3):
start_project_analysis
list_project_runs
get_project_data_range

dev (3):
seed_analysis_redesign_fixtures
clear_analysis_redesign_fixtures
clear_analysis_redesign_fixture_active_runs
```

Also pin startup cleanup, project list aggregates, project deletion, account-deletion dependency checks, NotebookLM group export, and diagnostic aggregation as named compatibility paths without adding them to the 27-command count.

- [ ] **Step 5: Write characterization tests before changing seams.**

Add or strengthen exact tests that pin:

- `run_reads_preserve_deleted_blank_and_snapshot_scope_labels` — list, active, get, deleted source/project, blank live label, and stored snapshot precedence;
- `analysis_run_search_escapes_percent_underscore_and_backslash_before_limit` — literal `%`, `_`, and `\`, multi-term matching across owned/foreign fields, filtering before `LIMIT`;
- `chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot` — only null/blank snapshot falls back to live source/project labels;
- `chat_profile_resolution_failure_is_async_after_request_id`;
- `chat_persistence_failure_keeps_completed_answer_failure_message` — exact prefix `Answer completed but chat history could not be saved: ` followed by the unchanged current error text;
- `terminal_cleanup_removes_active_state_when_terminal_persistence_fails`;
- `report_start_preserves_acceptance_order_and_two_corpus_reads`;
- `analysis_wire_contract_serializes_commands_events_and_errors_unchanged` — exact event channel, field/casing/optional behavior, request-ID families, terminal strings, and `AppError { kind, message }` JSON.

These are characterization tests: add them before changing a seam and require them to pass against the current implementation. Run every new Rust identity exactly:

```powershell
$checkpoint1Tests = @(
    'analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels',
    'analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit',
    'analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot',
    'analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id',
    'analysis::chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message',
    'analysis::report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails',
    'analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads'
)
foreach ($testName in $checkpoint1Tests) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
Invoke-CheckedNative 'analysis wire characterization' {
    npm.cmd run test -- src/lib/analysis-application-contract.test.ts -t 'analysis_wire_contract_serializes_commands_events_and_errors_unchanged'
}
```

Keep and explicitly run these existing witnesses:

```powershell
Invoke-ExactRustTest extractum 'analysis::store::tests::read_model::list_analysis_run_summaries_applies_query_before_limit'
Invoke-ExactRustTest extractum 'analysis::store::tests::read_model::list_analysis_run_summaries_escapes_literal_like_characters'
Invoke-ExactRustTest extractum 'analysis::store::tests::read_model::list_analysis_run_summaries_matches_all_query_terms_across_any_field'
Invoke-ExactRustTest extractum 'analysis::store::tests::read_model::resolve_run_scope_label_prefers_frozen_value'
Invoke-ExactRustTest extractum 'analysis::tests::trace_data_roundtrips_through_zstd'
Invoke-ExactRustTest extractum 'analysis::trace::tests::decode_trace_data_returns_typed_internal_for_invalid_zstd'
Invoke-ExactRustTest extractum 'analysis::corpus::tests::snapshot::list_run_snapshot_messages_page_does_not_fall_back_to_live_source'
Invoke-ExactRustTest extractum 'analysis::report::tests::lifecycle::request_analysis_run_cancel_running_but_inactive_keeps_conflict_message'
Invoke-ExactRustTest extractum 'analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent'
Invoke-ExactRustTest extractum 'projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources'
Invoke-ExactRustTest extractum 'projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout'
Invoke-ExactRustTest extractum 'account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned'
Invoke-ExactRustTest extractum 'diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates'
Invoke-ExactRustTest extractum 'notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id'
```

If a listed identity differs in executable output, use the exact current full identity while preserving the named leaf and record the correction in the plan execution notes; a `0 tests` run is a failure.

- [ ] **Step 6: Pin the exact Family 1 inclusion boundary.**

Source and behavior contracts must state that borrowed-connection composition is required only for:

```text
list_analysis_runs                foreign-term matching + returned-label enrichment
list_active_analysis_runs         returned-label enrichment
get_analysis_run                  returned-label enrichment
ask_analysis_run_question         only null/blank snapshot legacy-label fallback
```

Explicitly keep these as ordinary pool paths: `list_analysis_run_messages`, `get_analysis_run_trace`, `resolve_analysis_trace_refs`, `delete_analysis_run`, `list_analysis_chat_messages`, `clear_analysis_chat_messages`, duplicate lookup, snapshot/capture, cancellation/status/existence, startup cleanup, and terminal lifecycle reads.

- [ ] **Step 7: Run the checkpoint and advance its status atomically.**

```powershell
Invoke-CheckedNative 'Checkpoint 1 source contracts' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 1 format' { cargo fmt --manifest-path src-tauri/Cargo.toml --all }
Invoke-CheckedNative 'Checkpoint 1 package check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 1 package tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

Only after green, change the Phase 7 roadmap heading and shell-cap exact-state expectation to `preparation Checkpoint 1 retained`, then run:

```powershell
Invoke-CheckedNative 'Checkpoint 1 status contract' { npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 1 diff check' { git diff --check }
git status --short
$task1Files = @(
    'src/lib/analysis-contract-paths.ts',
    'src/lib/analysis-application-contract.test.ts',
    'src/lib/analysis-redesign-safety-contract.test.ts',
    'src/lib/crate-extraction-shell-cap-contract.test.ts',
    'src-tauri/src/analysis/chat.rs',
    'src-tauri/src/analysis/tests_application.rs',
    'src-tauri/src/analysis/mod.rs',
    'src-tauri/src/analysis/report/tests/lifecycle.rs',
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
)
Add-ScopedChanges -Label 'Checkpoint 1' -Allowed $task1Files
Invoke-CheckedNative 'commit Checkpoint 1' { git commit -m "test: freeze analysis extraction boundary" }
$checkpoint1Commit = (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'Checkpoint 1 commit'
```

---

### Task 2: Checkpoint 2 — Move Compression to Core and Add Safe Construction

**Files:**

- Modify: `src-tauri/src/analysis/trace.rs`
- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/store/read_model.rs`
- Modify: `src-tauri/src/analysis/store/snapshot.rs`
- Modify: `src-tauri/src/analysis/store/tests/read_model.rs`
- Modify: `src-tauri/src/analysis/store/tests/snapshot.rs`
- Modify: `src-tauri/src/analysis/report_commands.rs`
- Modify: `src-tauri/src/analysis/report/tests/capture.rs`
- Modify: `src-tauri/src/analysis/report/tests/harness.rs`
- Modify: `src-tauri/src/analysis/report/tests/scope.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `src-tauri/src/analysis/corpus/live.rs`
- Modify: `src-tauri/src/analysis/corpus/snapshot.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/harness.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/live.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/preflight.rs`
- Modify: `src-tauri/src/analysis/fixtures/tests/snapshot.rs`
- Modify: `src-tauri/src/projects/mod.rs`
- Modify: `src-tauri/src/projects/data_range.rs`
- Create: `src-tauri/src/analysis/domain_portable.rs`
- Create: `src-tauri/src/analysis/tests_portable.rs`
- Modify: `src/lib/analysis-application-contract.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`

- [ ] **Step 1: Prove direct `zstd` ownership before editing.**

```powershell
Invoke-CheckedNative 'direct zstd inventory' { rg -n "\bzstd::" src-tauri/src src-tauri/crates --glob '*.rs' }
```

Expected: app-side direct calls occur only in `src-tauri/src/analysis/trace.rs`; `extractum-core` remains the workspace compression owner. If another app call exists, stop and classify it before removing the app dependency.

- [ ] **Step 2: RED/GREEN the compression handoff.**

Extend the frozen trace tests to prove legacy bytes decode, new bytes round-trip, Telegram/YouTube ref JSON is byte-compatible, invalid zstd and invalid JSON retain their typed internal mapping, and compression failures preserve persisted/emitted text. Replace only direct `zstd::encode_all`/`decode_all` with:

```rust
extractum_core::compression::{compress_json_bytes, decompress_bytes}
```

Keep the frozen leaf names `trace_data_roundtrips_through_zstd` and `decode_trace_data_returns_typed_internal_for_invalid_zstd`.

- [ ] **Step 3: Add safe constructors without exposing fields.**

Before constructor behavior, extract this exact portable root set from `analysis/mod.rs` into compiled `domain_portable.rs`: private `extractum_core::time::now_secs`; template-kind/default-name constants; report run/scope/status constants; fallback chunk size; `validate_chat_turns`; `validate_chat_role`; and `default_report_template_body`. Keep only `ANALYSIS_RUN_EVENT` and `ANALYSIS_CHAT_EVENT` app-side for the Tauri adapter. The staging file uses `pub(crate)` only where sibling portable modules need an item and is included once at `analysis` module scope. App commands/fixtures migrate away from portable string constants to curated APIs or exact test literals; no public constant/function is added to the crate root.

Retarget every consumer using depth-stable relative imports (`super::domain` or `super::super::domain`) so the same source compiles under both `crate::analysis::*` before the move and crate-root modules after it. Direct shared capabilities use `extractum_core`/`extractum_llm`, never app aliases.

Add only the minimal compiling signatures needed by the first case, with behavior still returning the existing/default result. Then process these tests strictly one at a time: write one test with its unique assertion marker, run its exact runtime RED, implement that behavior, run `Invoke-ExactRustTest` GREEN, and only then write the next test. Never batch uncompiled tests.

```powershell
Assert-ExactRustRuntimeRed extractum 'analysis::report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes' 'src-tauri/src/analysis/report/tests/scope.rs' 'RED: CP2 report request constructors'
# Implement case 1, run its exact GREEN, then add case 2.
Invoke-ExactRustTest extractum 'analysis::report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes'
Assert-ExactRustRuntimeRed extractum 'analysis::store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes' 'src-tauri/src/analysis/store/tests/read_model.rs' 'RED: CP2 run list filters'
# Implement case 2, run its exact GREEN, then add case 3.
Invoke-ExactRustTest extractum 'analysis::store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes'
Assert-ExactRustRuntimeRed extractum 'analysis::report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities' 'src-tauri/src/analysis/report/tests/scope.rs' 'RED: CP2 scope identity cardinality'
# Implement case 3, run its exact GREEN, then add case 4.
Invoke-ExactRustTest extractum 'analysis::report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities'
Assert-ExactRustRuntimeRed extractum 'analysis::report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label' 'src-tauri/src/analysis/report/tests/scope.rs' 'RED: CP2 stable source order and label'
# Implement case 4 and run its exact GREEN.
Invoke-ExactRustTest extractum 'analysis::report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label'
```

Freeze these APIs while code still compiles in one package:

```rust
impl StartAnalysisReportRequest {
    pub fn from_command(
        source_id: Option<i64>,
        source_group_id: Option<i64>,
        project_id: Option<i64>,
        period_from: i64,
        period_to: i64,
        output_language: String,
        prompt_template_id: i64,
        model_override: Option<String>,
        profile_id: Option<String>,
        youtube_corpus_mode: Option<String>,
        include_migrated_history: bool,
    ) -> AppResult<Self>;

    pub fn for_source(
        source_id: i64,
        period_from: i64,
        period_to: i64,
        output_language: String,
        prompt_template_id: i64,
        model_override: Option<String>,
        profile_id: Option<String>,
        youtube_corpus_mode: Option<String>,
        include_migrated_history: bool,
    ) -> AppResult<Self>;

    pub fn for_source_group(
        source_group_id: i64,
        period_from: i64,
        period_to: i64,
        output_language: String,
        prompt_template_id: i64,
        model_override: Option<String>,
        profile_id: Option<String>,
        youtube_corpus_mode: Option<String>,
        include_migrated_history: bool,
    ) -> AppResult<Self>;

    pub fn for_project(
        project_id: i64,
        period_from: i64,
        period_to: i64,
        output_language: String,
        prompt_template_id: i64,
        model_override: Option<String>,
        profile_id: Option<String>,
        youtube_corpus_mode: Option<String>,
        include_migrated_history: bool,
    ) -> AppResult<Self>;
}

impl AnalysisRunListFilters {
    pub fn for_analysis(
        source_id: Option<i64>,
        source_group_id: Option<i64>,
        limit: i64,
        query: Option<String>,
        status: Option<String>,
        provider: Option<String>,
        model: Option<String>,
        template: Option<String>,
        date_from: Option<String>,
        date_to: Option<String>,
    ) -> AppResult<Self>;
    pub fn for_project(project_id: i64, limit: i64) -> Self;
    pub fn foreign_label_search_terms(&self) -> &[String];
}

impl YoutubeCorpusMode {
    pub fn from_wire(value: Option<&str>) -> Result<Self, String>;
    pub fn as_wire(self) -> &'static str;
    pub fn includes_description(self) -> bool;
    pub fn includes_comments(self) -> bool;
}

impl FromStr for YoutubeCorpusMode {
    type Err = String;
}

pub fn resolve_analysis_telegram_history_scope(
    include_migrated_history: bool,
    source_kind: AnalysisSourceKind,
) -> AppResult<(&'static str, bool)>;
```

`AnalysisRunListFilters::for_project` is intentionally infallible: the app has already validated the project identity, and this constructor carries only that ID plus the fixed/clamped project-list limit. `for_analysis` remains fallible because it normalizes and validates user-supplied query/date/filter values.

The report constructors accept the current common tail in this exact order: `period_from`, `period_to`, `output_language`, `prompt_template_id`, `model_override`, `profile_id`, `youtube_corpus_mode`, `include_migrated_history`. Fields remain private. All constructors validate period first and output language second; `from_command` then requires exactly one scope `Option` to be present and dispatches to the matching specialized constructor. It deliberately does not reject a zero or negative contained ID before `get_pool`; the later scope lookup/constructor preserves the current not-found/validation timing. `start_analysis_report` calls `from_command` before fallible `get_pool`; project entry points call their specialized constructor before their analysis pool/template step. No public unchecked constructor exists. Migrate every struct literal now, including project command consumers and test harnesses. Extend the constructor characterization to prove `Some(0)` survives this early phase while missing or multiple `Some` identities fail before pool access.

- [ ] **Step 4: Introduce and validate `ResolvedAnalysisScope`.**

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisScopeKind { SingleSource, SourceGroup, Project }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisSourceKind { Telegram, Youtube }

pub struct ResolvedAnalysisScope {
    scope_kind: AnalysisScopeKind,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
    source_kind: AnalysisSourceKind,
    source_ids: Vec<i64>,
    scope_label_snapshot: String,
}
```

Use these exact constructors. `for_source` receives resolved `source_ids` explicitly because a YouTube playlist identity may resolve to linked video IDs rather than `[source_id]`:

```rust
impl ResolvedAnalysisScope {
    pub fn for_source(
        source_id: i64,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
        scope_label_snapshot: String,
    ) -> AppResult<Self>;
    pub fn for_source_group(
        source_group_id: i64,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
        scope_label_snapshot: String,
    ) -> AppResult<Self>;
    pub fn for_project(
        project_id: i64,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
        scope_label_snapshot: String,
    ) -> AppResult<Self>;
}
```

Provide these exact accessors:

```rust
pub fn scope_kind(&self) -> AnalysisScopeKind;
pub fn source_id(&self) -> Option<i64>;
pub fn source_group_id(&self) -> Option<i64>;
pub fn project_id(&self) -> Option<i64>;
pub fn source_kind(&self) -> AnalysisSourceKind;
pub fn source_ids(&self) -> &[i64];
pub fn scope_label_snapshot(&self) -> &str;
```

Enforce exactly one positive scope identity, non-empty positive source IDs, stable de-duplication that preserves first-occurrence order, and a non-empty trimmed fallback label. Add a characterization using input `[20, 10, 20]` and require output `[20, 10]`; numerical sorting is forbidden. Checkpoint 3 wraps this value in the exact app-private `AppAnalysisScopeResolution` seam defined there. The skipped playlist count must not enter a report ticket or crate value.

Parsing database/app source strings into `AnalysisSourceKind` remains app-owned: only exact existing `telegram` and `youtube` values construct the enum, while unknown values retain the current path-specific validation/internal error instead of introducing an `Unknown` wire variant.

Migrate `projects/data_range.rs` in this checkpoint rather than during the mechanical move. Parse `youtube_corpus_mode` through the typed `YoutubeCorpusMode` API and convert its storage `source_type` to `AnalysisSourceKind` app-side before calling `resolve_analysis_telegram_history_scope`; an unknown storage value retains this path's exact current validation/internal mapping rather than gaining a new enum variant or generic parser error. Preserve the current early migrated-history rejection for a non-Telegram or unmaterialized YouTube project; do not wait for the Checkpoint 3 resolver split to recover it. This step changes no SQL ownership and introduces no exported app helper.

Move the eight frozen root domain tests from the mixed `mod.rs` test module into `tests_portable.rs`; keep command/foreign integration tests under the app module. Until extraction, the existing `#[cfg(test)] mod tests` in `analysis/mod.rs` contains exactly one `include!("tests_portable.rs")`, so identities remain `analysis::tests::<leaf>` and the staging file is not declared as a second module. Task 7 removes that include and moves the file to crate `tests.rs`. This is a test-file split only and preserves every Appendix A leaf name.

- [ ] **Step 5: RED/GREEN exact construction tests.**

```text
start_analysis_report_request_constructors_preserve_source_group_and_project_scopes
analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes
resolved_analysis_scope_rejects_zero_or_multiple_identities
resolved_analysis_scope_requires_nonempty_stable_sources_and_label
youtube_corpus_mode_parses_wire_values_and_defaults
```

Run each exactly under `extractum`, then the package check/test. Update roadmap and shell-cap to `preparation Checkpoint 2 retained` only after green.

```powershell
$checkpoint2Green = @(
    'analysis::report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes',
    'analysis::store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes',
    'analysis::report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities',
    'analysis::report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label',
    'analysis::corpus::tests::live::youtube_corpus_mode_parses_wire_values_and_defaults'
)
foreach ($testName in $checkpoint2Green) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
$projectDataRangeWitnesses = @(
    'projects::data_range::tests::project_data_range_returns_nulls_for_empty_project',
    'projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds',
    'projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested',
    'projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources',
    'projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project',
    'projects::data_range::tests::project_data_range_rejects_mixed_provider_project',
    'projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project',
    'projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram'
)
foreach ($testName in $projectDataRangeWitnesses) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
```

- [ ] **Step 6: Commit the green checkpoint.**

```powershell
Invoke-CheckedNative 'Checkpoint 2 contracts' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 2 format' { cargo fmt --manifest-path src-tauri/Cargo.toml --all }
Invoke-CheckedNative 'Checkpoint 2 package check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 2 package tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 2 diff check' { git diff --check }
$task2Files = @(
    'src-tauri/src/analysis/trace.rs',
    'src-tauri/src/analysis/models.rs',
    'src-tauri/src/analysis/report.rs',
    'src-tauri/src/analysis/corpus.rs',
    'src-tauri/src/analysis/store/read_model.rs',
    'src-tauri/src/analysis/store/snapshot.rs',
    'src-tauri/src/analysis/store/tests/read_model.rs',
    'src-tauri/src/analysis/store/tests/snapshot.rs',
    'src-tauri/src/analysis/report_commands.rs',
    'src-tauri/src/analysis/report/tests/capture.rs',
    'src-tauri/src/analysis/report/tests/harness.rs',
    'src-tauri/src/analysis/report/tests/scope.rs',
    'src-tauri/src/analysis/mod.rs',
    'src-tauri/src/analysis/chat.rs',
    'src-tauri/src/analysis/corpus/live.rs',
    'src-tauri/src/analysis/corpus/snapshot.rs',
    'src-tauri/src/analysis/corpus/tests/harness.rs',
    'src-tauri/src/analysis/corpus/tests/live.rs',
    'src-tauri/src/analysis/corpus/tests/preflight.rs',
    'src-tauri/src/analysis/fixtures/tests/snapshot.rs',
    'src-tauri/src/analysis/domain_portable.rs',
    'src-tauri/src/analysis/tests_portable.rs',
    'src-tauri/src/projects/mod.rs',
    'src-tauri/src/projects/data_range.rs',
    'src/lib/analysis-application-contract.test.ts',
    'src/lib/analysis-redesign-safety-contract.test.ts',
    'src/lib/crate-extraction-shell-cap-contract.test.ts',
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
)
Add-ScopedChanges -Label 'Checkpoint 2' -Allowed $task2Files
Invoke-CheckedNative 'commit Checkpoint 2' { git commit -m "refactor: prepare analysis value boundary" }
$checkpoint2Commit = (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'Checkpoint 2 commit'
```

---

### Task 3: Checkpoint 3 — Introduce Scope, Corpus, and Foreign-Label Adapters

**Files:**

- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/corpus/live.rs`
- Modify: `src-tauri/src/analysis/corpus/source_resolution.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/harness.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/live.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/mod.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/preflight.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/snapshot.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/source_resolution.rs`
- Modify: `src-tauri/src/analysis/store/read_model.rs`
- Modify: `src-tauri/src/analysis/store/tests/read_model.rs`
- Create: `src-tauri/src/analysis/corpus_portable.rs`
- Create: `src-tauri/src/analysis/corpus/source_resolution_policy.rs`
- Create: `src-tauri/src/analysis/corpus/tests/harness_portable.rs`
- Create: `src-tauri/src/analysis/corpus/tests/live_portable.rs`
- Create: `src-tauri/src/analysis/corpus/tests/mod_portable.rs`
- Create: `src-tauri/src/analysis/corpus/tests/preflight_portable.rs`
- Create: `src-tauri/src/analysis/corpus/tests/source_resolution_portable.rs`
- Create: `src-tauri/src/analysis/store/owned_read_model.rs`
- Create: `src-tauri/src/analysis/store/tests/read_model_portable.rs`
- Create: `src-tauri/src/analysis/report/tests/corpus_port.rs`
- Modify: `src-tauri/src/analysis/report/tests/mod.rs`
- Modify: `src-tauri/src/analysis/fixtures/tests/snapshot.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `src-tauri/src/projects/mod.rs`
- Modify: `src-tauri/src/projects/data_range.rs`
- Modify: `src/lib/analysis-application-contract.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`

- [ ] **Step 1: Define the owned corpus ABI and make the existing loader its app adapter.**

Create `report/tests/corpus_port.rs` and add the minimal compiling port seam first. Process one test at a time; each assertion contains the exact unique marker below, reaches `running 1 test`, turns GREEN before the next test is written, and never shares a compile failure with another case:

```powershell
Assert-ExactRustRuntimeRed extractum 'analysis::report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads' 'src-tauri/src/analysis/report/tests/corpus_port.rs' 'RED: CP3 distinct corpus reads'
# Implement case 1 and run GREEN before adding case 2.
Invoke-ExactRustTest extractum 'analysis::report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads'
Assert-ExactRustRuntimeRed extractum 'analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure' 'src-tauri/src/analysis/report/tests/corpus_port.rs' 'RED: CP3 empty capture event order'
# Implement case 2 and run GREEN before adding case 3.
Invoke-ExactRustTest extractum 'analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure'
Assert-ExactRustRuntimeRed extractum 'analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure' 'src-tauri/src/analysis/report/tests/corpus_port.rs' 'RED: CP3 failed capture event order'
# Implement case 3 and run GREEN.
Invoke-ExactRustTest extractum 'analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure'
```

Use these exact names and signatures:

```rust
pub type AnalysisPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;

pub trait AnalysisCorpusReader: Send + Sync + 'static {
    fn load_corpus(
        &self,
        request: AnalysisCorpusRequest,
    ) -> AnalysisPortFuture<'_, Vec<AnalysisCorpusMessage>>;
}

pub struct AnalysisCorpusRequest {
    source_kind: AnalysisSourceKind,
    source_ids: Vec<i64>,
    period_from: i64,
    period_to: i64,
    youtube_corpus_mode: YoutubeCorpusMode,
    include_migrated_history: bool,
}

pub struct AnalysisCorpusMessage {
    item_id: i64,
    source_id: i64,
    external_id: String,
    published_at: i64,
    author: Option<String>,
    content: String,
    r#ref: String,
    item_kind: Option<String>,
    source_type: Option<String>,
    source_subtype: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
}

pub struct AnalysisRunPreflightLimits {
    max_messages_per_run: usize,
    max_chunks_per_run: usize,
    max_estimated_input_chars_per_run: usize,
    max_background_requests_per_run: usize,
}

pub struct AnalysisRunPreflight {
    source_ids: Vec<i64>,
    message_count: usize,
    estimated_input_chars: usize,
    estimated_chunks: usize,
    limits: AnalysisRunPreflightLimits,
}

pub async fn preflight_analysis_corpus(
    reader: &dyn AnalysisCorpusReader,
    request: &AnalysisCorpusRequest,
    chunk_target_chars: usize,
    limits: AnalysisRunPreflightLimits,
) -> AppResult<AnalysisRunPreflight>;
```

`AnalysisCorpusRequest::new(source_kind, source_ids, period_from, period_to, youtube_corpus_mode, include_migrated_history) -> AppResult<Self>` validates the period range and stable non-empty source IDs. `AnalysisCorpusMessage::new(item_id, source_id, external_id, published_at, author, content, r#ref, item_kind, source_type, source_subtype, metadata_zstd) -> Self` is intentionally lossless and infallible: synthetic description rows may retain `item_id == 0`, and no new content/ref rejection is introduced. Rename `CorpusLoadRequest` and `CorpusMessage`; do not widen their fields.

The exact accessors are:

```rust
impl AnalysisCorpusRequest {
    pub fn source_kind(&self) -> AnalysisSourceKind;
    pub fn source_ids(&self) -> &[i64];
    pub fn period_from(&self) -> i64;
    pub fn period_to(&self) -> i64;
    pub fn youtube_corpus_mode(&self) -> YoutubeCorpusMode;
    pub fn include_migrated_history(&self) -> bool;
}

impl AnalysisCorpusMessage {
    pub fn item_id(&self) -> i64;
    pub fn source_id(&self) -> i64;
    pub fn external_id(&self) -> &str;
    pub fn published_at(&self) -> i64;
    pub fn author(&self) -> Option<&str>;
    pub fn content(&self) -> &str;
    pub fn reference(&self) -> &str;
    pub fn item_kind(&self) -> Option<&str>;
    pub fn source_type(&self) -> Option<&str>;
    pub fn source_subtype(&self) -> Option<&str>;
    pub fn metadata_zstd(&self) -> Option<&[u8]>;
}

impl AnalysisRunPreflightLimits {
    pub fn max_messages_per_run(&self) -> usize;
    pub fn max_chunks_per_run(&self) -> usize;
    pub fn max_estimated_input_chars_per_run(&self) -> usize;
    pub fn max_background_requests_per_run(&self) -> usize;
}

impl AnalysisRunPreflight {
    pub fn source_ids(&self) -> &[i64];
    pub fn message_count(&self) -> usize;
    pub fn estimated_input_chars(&self) -> usize;
    pub fn estimated_chunks(&self) -> usize;
    pub fn limits(&self) -> &AnalysisRunPreflightLimits;
}
```

`AnalysisRunPreflightLimits` keeps its existing `Default`. Fields on both preflight values are private outside their defining module. Add one non-public `pub(crate) fn AnalysisRunPreflight::from_observation(source_ids, message_count, estimated_input_chars, estimated_chunks, limits) -> Self` for crate pipeline/tests that currently construct the value from a sibling report module; it is not re-exported as a public constructor or test helper. The three retained app preflight integration tests call `preflight_analysis_corpus` with the real `AppAnalysisCorpusReader`, preserving their foreign-loader subject after extraction. Pure preflight calculations remain private crate helpers.

`AppAnalysisCorpusReader` lives app-side and owns all SQL against `analysis_documents`, `sources`, `items`, `telegram_messages`, YouTube playlist/video/transcript tables, document-kind predicates, typed metadata decoding, ordering, and evidence refs. It performs one complete current loader call per trait invocation and contains no cache.

Place the portable port/preflight implementation in compiled `corpus_portable.rs`; keep the adapter module root in `corpus.rs`. Split corpus policy/harness/tests into the exact prepared filenames listed in the frozen map, preserving all leaf names and current app integration paths.

Preserve pre-move Cargo identities with a temporary include seam, not a second `*_portable` module name. Each leaf test module (`corpus/tests/live.rs`, `preflight.rs`, `source_resolution.rs`, and `store/tests/read_model.rs`) contains exactly one `include!("<name>_portable.rs")` at module scope and retains only app-owned tests around it. The staging leaf is not also declared as a module. Thus portable tests still execute under their original `analysis::corpus::tests::*` or `analysis::store::tests::read_model::*` identity before extraction. `corpus/tests/mod_portable.rs` and `store/tests/mod_portable.rs` are declarative final module roots: they are not compiled before extraction, because including them would duplicate the app root's module declarations; the source contract parses their exact declarations instead. Task 7 moves all staging files to unsuffixed crate paths and removes the leaf includes from retained app files in the same mechanical diff.

- [ ] **Step 2: Move scope resolution behind the owned value.**

Refactor the app resolver to return `AppAnalysisScopeResolution`. Preserve:

- source/project existence checks;
- `project_sources` reads;
- source identity readiness only for `list_analysis_sources`;
- report/project-report start without a new readiness gate;
- YouTube playlist expansion and removal of unlinked/removed rows;
- project same-provider validation on report start;
- group same-provider validation only on create/update, not report start;
- stable source ordering and current no-linked-video messages.

The crate-facing report path receives only `ResolvedAnalysisScope`; `skipped_unlinked_playlist_items` remains app-private and appears only in the existing playlist characterization.

Freeze this app-only seam; it is not part of the crate root allowlist:

```rust
pub(crate) struct AppAnalysisScopeResolution {
    scope: ResolvedAnalysisScope,
    skipped_unlinked_playlist_items: usize,
}

impl AppAnalysisScopeResolution {
    pub(crate) fn scope(&self) -> &ResolvedAnalysisScope;
    pub(crate) fn skipped_unlinked_playlist_items(&self) -> usize;
    pub(crate) fn into_scope(self) -> ResolvedAnalysisScope;
}

pub(crate) async fn resolve_analysis_sources(
    pool: &SqlitePool,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
) -> Result<AppAnalysisScopeResolution, AnalysisSourceResolutionError>;
```

Migrate `get_project_data_range_in_pool` to this prepared app resolver and the typed `ResolvedAnalysisScope`. Preserve its exact order: project existence, typed corpus-mode parse, empty-project `None/None`, early migrated-history rejection from project source types, app scope resolution, typed `NoLinkedYoutubeVideos` to `None/None`, typed history policy, then range SQL. In particular, do not move the early history check behind scope resolution: an unmaterialized YouTube playlist must retain its validation error rather than become an empty range.

Callers that need diagnostics read `skipped_unlinked_playlist_items()` before consuming the wrapper. Report preparation then calls `into_scope()` to pass the owned scope into the crate ticket; `projects/data_range.rs` only borrows through `scope()`. Do not add `Clone` merely to escape the private wrapper.

Remove the imported `push_analysis_document_kind_filter`. `projects/data_range.rs` keeps its app-owned `analysis_documents`/foreign range SQL and defines a private, fixed-alias `d` predicate builder that matches `AnalysisSourceKind::{Telegram, Youtube}` and uses `YoutubeCorpusMode::{includes_description, includes_comments}` to emit the exact current document-kind predicates. It has no `pub`, no `AppResult`, no dynamic table alias, and no crate/API exposure. Make the corpus reader's original predicate helper module-private in `analysis/corpus/live.rs` and remove its re-exports from `analysis/corpus.rs` and `analysis/mod.rs`; only `AppAnalysisCorpusReader` calls it.

Add the standing case `keeps project data range on typed app scope without analysis SQL helper imports` to `analysis-application-contract.test.ts`. It requires the typed mode and app resolver, the typed `NoLinkedYoutubeVideos` branch without string matching, the private fixed-alias local predicate, and no import/re-export of the corpus SQL helper. The case remains GREEN before and after Task 7; it does not forbid app-owned `analysis_documents` SQL in this project adapter.

- [ ] **Step 3: Replace the broad run JOIN with typed staged enrichment.**

Add these constructed values with private fields:

```rust
pub struct AnalysisForeignLabelMatch {
    term: String,
    source_ids: Vec<i64>,
    project_ids: Vec<i64>,
}
pub struct AnalysisSourceLabel { source_id: i64, title: Option<String> }
pub struct AnalysisProjectLabel { project_id: i64, name: Option<String> }
pub struct AnalysisForeignLabels {
    sources: Vec<AnalysisSourceLabel>,
    projects: Vec<AnalysisProjectLabel>,
}
pub enum AnalysisForeignLabelRef { Source(i64), Project(i64) }
```

`AnalysisRunSummaryEnrichment`, `AnalysisRunDetailEnrichment`, `AnalysisChatRunEnrichment`, and `AnalysisChatRun` are public opaque handoff types with private field layouts and no serialization or row getters.

```rust
impl AnalysisChatRun {
    pub fn needs_legacy_foreign_label(&self) -> bool;
}

pub async fn load_analysis_chat_run(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<AnalysisChatRun>;
```

Required constructors and finishers:

```rust
impl AnalysisForeignLabelMatch {
    pub fn new(
        term: String,
        source_ids: Vec<i64>,
        project_ids: Vec<i64>,
    ) -> AppResult<Self>;
}

impl AnalysisForeignLabels {
    pub fn new(
        sources: Vec<AnalysisSourceLabel>,
        projects: Vec<AnalysisProjectLabel>,
    ) -> AppResult<Self>;
}

impl AnalysisSourceLabel {
    pub fn new(source_id: i64, title: Option<String>) -> AppResult<Self>;
    pub fn source_id(&self) -> i64;
    pub fn title(&self) -> Option<&str>;
}

impl AnalysisProjectLabel {
    pub fn new(project_id: i64, name: Option<String>) -> AppResult<Self>;
    pub fn project_id(&self) -> i64;
    pub fn name(&self) -> Option<&str>;
}

impl AnalysisRunSummaryEnrichment {
    pub fn foreign_label_refs(&self) -> Vec<AnalysisForeignLabelRef>;
    pub fn finish(self, labels: AnalysisForeignLabels)
        -> AppResult<Vec<AnalysisRunSummary>>;
}

impl AnalysisRunDetailEnrichment {
    pub fn foreign_label_refs(&self) -> Vec<AnalysisForeignLabelRef>;
    pub fn finish(self, labels: AnalysisForeignLabels)
        -> AppResult<Option<AnalysisRunDetail>>;
}

impl AnalysisChatRunEnrichment {
    pub fn foreign_label_refs(&self) -> Vec<AnalysisForeignLabelRef>;
    pub fn finish(self, labels: AnalysisForeignLabels)
        -> AppResult<Option<AnalysisChatRun>>;
}
```

The opaque enrichment objects may contain private SQL rows. They are not serializable and expose no raw row or field access. Their `finish` methods preserve `scope_label_snapshot` precedence, `source_title`, `project_name`, group/template labels, deleted-scope behavior, and legacy blank/null fallback.

- [ ] **Step 4: Implement the four Family 1 connection participants.**

```rust
pub async fn prepare_analysis_run_summaries(
    conn: &mut SqliteConnection,
    filters: AnalysisRunListFilters,
    matches: Vec<AnalysisForeignLabelMatch>,
) -> AppResult<AnalysisRunSummaryEnrichment>;

pub async fn prepare_active_analysis_run_summaries(
    conn: &mut SqliteConnection,
    run_ids: &HashSet<i64>,
) -> AppResult<AnalysisRunSummaryEnrichment>;

pub async fn prepare_analysis_run_detail(
    conn: &mut SqliteConnection,
    run_id: i64,
) -> AppResult<AnalysisRunDetailEnrichment>;

pub async fn prepare_legacy_analysis_chat_run(
    conn: &mut SqliteConnection,
    run_id: i64,
) -> AppResult<AnalysisChatRunEnrichment>;
```

All four take `&mut SqliteConnection` first. None has a pool overload, acquires a connection, begins/commits/rolls back, or calls an app helper. The app coordinators `list_analysis_runs_in_pool`, `list_active_analysis_runs_in_pool`, `get_analysis_run_in_pool`, and `resolve_legacy_analysis_chat_run_in_pool` each:

1. call `pool.begin()`;
2. perform foreign term matching or label loading with `&mut *transaction`;
3. call the matching crate participant with the same `&mut *transaction`;
4. load only labels referenced by the opaque result on that same connection;
5. finish the result;
6. alone commit or roll back.

`list_project_runs` must call the existing `list_analysis_runs_in_pool` coordinator with `AnalysisRunListFilters::for_project`; it is an indirect caller of the already-enumerated summaries participant, not a fifth participant or workflow family.

The legacy chat path first calls `load_analysis_chat_run` as an ordinary pool read. Only when `needs_legacy_foreign_label()` is true does the app open the named transaction and re-read all required run fields through `prepare_legacy_analysis_chat_run` before loading the foreign fallback label on the same snapshot.

- [ ] **Step 5: Keep excluded run paths on the pool.**

Split `fetch_run_row` into private, purpose-specific owned-table reads. The ordinary public wrappers for messages, trace, trace refs, run deletion, chat list/clear, cancellation, startup cleanup, duplicate lookup, capture, and terminal lifecycle accept `&SqlitePool`. The source contract must reject passing `&mut SqliteConnection` to those paths and reject a broad public `fetch_run_row` or `AnalysisRunRow`.

Place owned run-query/mapping logic and its 15 frozen portable tests in compiled `store/owned_read_model.rs` and `store/tests/read_model_portable.rs`; leave foreign matching/enrichment and its two frozen integration tests at the original app paths.

In the same checkpoint, migrate retained `fixtures/tests/snapshot.rs` away from `fetch_run_row`, `map_run_detail`, direct `list_analysis_run_summaries`, and `AnalysisRunListFilters` literals. Its four tests call the real app coordinators `get_analysis_run_in_pool` and `list_analysis_runs_in_pool` with public constructors, so they continue to cover enriched command-facing results without depending on crate-private rows/mappers. Run all four exactly:

```powershell
$fixtureSnapshotTests = @(
    'analysis::fixtures::tests::snapshot::capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report',
    'analysis::fixtures::tests::snapshot::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot',
    'analysis::fixtures::tests::snapshot::missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages',
    'analysis::fixtures::tests::snapshot::seeded_snapshot_runs_expose_captured_snapshot_state'
)
foreach ($testName in $fixtureSnapshotTests) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
```

Retarget every affected `analysis-redesign-safety-contract` selector now: its explicit `before` path is the prepared corpus/store portable filename, and its `after` path is the final crate-relative filename. Do not wait for Task 7 or delete an assertion during the split.

- [ ] **Step 6: RED/GREEN the independent A/B behavior.**

Add a stateful fake reader that returns distinct message sets for its first and second calls, records both requests, and can return empty/error B. Run these exact tests under `extractum`:

```text
report_execution_uses_distinct_preflight_and_capture_corpus_reads
started_load_items_uses_preflight_summary_before_empty_capture_failure
started_load_items_uses_preflight_summary_before_error_capture_failure
```

Assert exactly two calls; A controls synchronous acceptance and the exact later `Preflight passed: ...` started message; B alone is persisted/reloaded and controls chunk/map/reduce/trace. Empty/error B must publish `started/load_items` first, then preserve the current capture-failed status/event/error text.

```powershell
$checkpoint3Green = @(
    'analysis::report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads',
    'analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure',
    'analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure'
)
foreach ($testName in $checkpoint3Green) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
```

- [ ] **Step 7: Run all retained live/scope/label witnesses.**

```powershell
Invoke-CheckedNative 'Checkpoint 3 live corpus tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::corpus::tests::live::' -- --nocapture }
Invoke-CheckedNative 'Checkpoint 3 scope tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::corpus::tests::source_resolution::' -- --nocapture }
Invoke-CheckedNative 'Checkpoint 3 read-model tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::store::tests::read_model::' -- --nocapture }
$projectDataRangeWitnesses = @(
    'projects::data_range::tests::project_data_range_returns_nulls_for_empty_project',
    'projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds',
    'projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested',
    'projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources',
    'projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project',
    'projects::data_range::tests::project_data_range_rejects_mixed_provider_project',
    'projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project',
    'projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram'
)
foreach ($testName in $projectDataRangeWitnesses) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
Invoke-CheckedNative 'Checkpoint 3 data-range boundary contract' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts -t 'keeps project data range on typed app scope without analysis SQL helper imports' }
Invoke-CheckedNative 'Checkpoint 3 format' { cargo fmt --manifest-path src-tauri/Cargo.toml --all }
Invoke-CheckedNative 'Checkpoint 3 package check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 3 package tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

Confirm each filtered selection is non-empty. Update roadmap and shell-cap to `preparation Checkpoint 3 retained` only after green.

- [ ] **Step 8: Commit the green checkpoint.**

```powershell
Invoke-CheckedNative 'Checkpoint 3 contracts' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 3 diff check' { git diff --check }
$task3Files = @(
    'src-tauri/src/analysis/corpus.rs',
    'src-tauri/src/analysis/models.rs',
    'src-tauri/src/analysis/corpus/live.rs',
    'src-tauri/src/analysis/corpus/source_resolution.rs',
    'src-tauri/src/analysis/corpus/tests/harness.rs',
    'src-tauri/src/analysis/corpus/tests/live.rs',
    'src-tauri/src/analysis/corpus/tests/mod.rs',
    'src-tauri/src/analysis/corpus/tests/preflight.rs',
    'src-tauri/src/analysis/corpus/tests/snapshot.rs',
    'src-tauri/src/analysis/corpus/tests/source_resolution.rs',
    'src-tauri/src/analysis/store/read_model.rs',
    'src-tauri/src/analysis/store/tests/read_model.rs',
    'src-tauri/src/analysis/corpus_portable.rs',
    'src-tauri/src/analysis/corpus/source_resolution_policy.rs',
    'src-tauri/src/analysis/corpus/tests/harness_portable.rs',
    'src-tauri/src/analysis/corpus/tests/live_portable.rs',
    'src-tauri/src/analysis/corpus/tests/mod_portable.rs',
    'src-tauri/src/analysis/corpus/tests/preflight_portable.rs',
    'src-tauri/src/analysis/corpus/tests/source_resolution_portable.rs',
    'src-tauri/src/analysis/store/owned_read_model.rs',
    'src-tauri/src/analysis/store/tests/read_model_portable.rs',
    'src-tauri/src/analysis/report/tests/corpus_port.rs',
    'src-tauri/src/analysis/report/tests/mod.rs',
    'src-tauri/src/analysis/fixtures/tests/snapshot.rs',
    'src-tauri/src/analysis/mod.rs',
    'src-tauri/src/analysis/report.rs',
    'src-tauri/src/analysis/chat.rs',
    'src-tauri/src/projects/mod.rs',
    'src-tauri/src/projects/data_range.rs',
    'src/lib/analysis-application-contract.test.ts',
    'src/lib/analysis-redesign-safety-contract.test.ts',
    'src/lib/crate-extraction-shell-cap-contract.test.ts',
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
)
Add-ScopedChanges -Label 'Checkpoint 3' -Allowed $task3Files
Invoke-CheckedNative 'commit Checkpoint 3' { git commit -m "refactor: introduce analysis corpus boundary" }
$checkpoint3Commit = (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'Checkpoint 3 commit'
```

---

### Task 4: Checkpoint 4 — Introduce the Runtime and Event Seam

**Files:**

- Modify: `src-tauri/src/analysis/events.rs`
- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/report/capture.rs`
- Modify: `src-tauri/src/analysis/report/lifecycle.rs`
- Modify: `src-tauri/src/analysis/report/phases.rs`
- Modify: `src-tauri/src/analysis/report/requests.rs`
- Modify: `src-tauri/src/analysis/report/tests/harness.rs`
- Modify: `src-tauri/src/analysis/report/tests/lifecycle.rs`
- Modify: `src-tauri/src/analysis/report/tests/phases.rs`
- Modify: `src-tauri/src/analysis/report/tests/preflight.rs`
- Modify: `src-tauri/src/analysis/report/tests/requests.rs`
- Modify: `src-tauri/src/analysis/report/tests/scope.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `src-tauri/src/analysis/state.rs`
- Modify: `src-tauri/src/analysis/report_commands.rs`
- Create: `src-tauri/src/analysis/chat_engine.rs`
- Create: `src-tauri/src/analysis/report_engine.rs`
- Create: `src-tauri/src/analysis/report/lifecycle_portable.rs`
- Create: `src-tauri/src/analysis/report/tests/mod_portable.rs`
- Create: `src-tauri/src/analysis/report/tests/runtime.rs`
- Modify: `src-tauri/src/analysis/report/tests/mod.rs`
- Modify: `src-tauri/src/analysis/report/tests/architecture.rs`
- Modify: `src-tauri/src/analysis/fixtures.rs`
- Modify: `src-tauri/src/analysis/fixtures/tests/active_runs.rs`
- Modify: `src-tauri/src/analysis/tests_application.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/accounts.rs`
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/diagnostics/mod.rs`
- Modify: `src-tauri/src/prompt_packs/runtime_commands.rs`
- Modify: `src/lib/analysis-application-contract.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`
- Verify only: `src/lib/llm-crate-boundary-contract.test.ts`
- Verify only: `src/lib/prompt-pack-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`

- [ ] **Step 1: Make the single managed scheduler shareable and prove it green.**

Before editing the event/report/chat seams, migrate the single Tauri-managed scheduler to `Arc<LlmSchedulerState>` in `lib.rs`, accounts, LLM commands, diagnostics, prompt-pack runtime, and every analysis command/phase lookup. Characterize that all command paths and `AppHandle` lookups observe the same `Arc` allocation; reject managing both raw and `Arc` scheduler states. Clone the `Arc` only across detached/`JoinSet` ownership boundaries. This is a representation change to the existing capability, not a new scheduler abstraction or dependency.

Run the affected source contracts, full app check, and full app tests before proceeding:

```powershell
Invoke-CheckedNative 'Checkpoint 4 scheduler contracts' { npm.cmd run test -- src/lib/llm-crate-boundary-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/analysis-application-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 4 scheduler package check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 4 scheduler package tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

Keep this in the single approved Checkpoint 4 commit: the owned runtime signatures introduced below are the reason the scheduler must be clonable into `'static` work, and a separate retained commit/status would add a sixth preparation state not present in the approved design. If this isolated substep fails, fix or revert it before starting Step 2; do not debug both blast radii together.

- [ ] **Step 2: Define typed synchronous events.**

Create `report/tests/runtime.rs` and the new chat test around a minimal compiling runtime seam. Process one case at a time; require its unique runtime marker and exact GREEN before adding the next test:

```powershell
Assert-ExactRustRuntimeRed extractum 'analysis::report::tests::runtime::report_execution_publishes_typed_events_in_existing_order' 'src-tauri/src/analysis/report/tests/runtime.rs' 'RED: CP4 report event order'
# Implement case 1 and run GREEN before adding case 2.
Invoke-ExactRustTest extractum 'analysis::report::tests::runtime::report_execution_publishes_typed_events_in_existing_order'
Assert-ExactRustRuntimeRed extractum 'analysis::chat::tests::chat_execution_persists_turns_before_completed_event' 'src-tauri/src/analysis/chat.rs' 'RED: CP4 chat persistence before completed'
# Implement case 2 and run GREEN before adding case 3.
Invoke-ExactRustTest extractum 'analysis::chat::tests::chat_execution_persists_turns_before_completed_event'
Assert-ExactRustRuntimeRed extractum 'analysis::report::tests::runtime::terminal_cleanup_always_removes_active_report_state' 'src-tauri/src/analysis/report/tests/runtime.rs' 'RED: CP4 terminal cleanup'
# Implement case 3 and run GREEN before adding case 4.
Invoke-ExactRustTest extractum 'analysis::report::tests::runtime::terminal_cleanup_always_removes_active_report_state'
Assert-ExactRustRuntimeRed extractum 'analysis::tests_application::report_profile_resolution_failure_prevents_run_creation' 'src-tauri/src/analysis/tests_application.rs' 'RED: CP4 profile precedence'
# Implement case 4 and run GREEN.
Invoke-ExactRustTest extractum 'analysis::tests_application::report_profile_resolution_failure_prevents_run_creation'
```

```rust
pub trait AnalysisEventSink: Send + Sync + 'static {
    fn publish_run(&self, event: AnalysisRunEvent);
    fn publish_chat(&self, event: AnalysisChatEvent);
}
```

Move event construction into portable code. `TauriAnalysisEventSink` remains in `src-tauri/src/analysis/events.rs` and each method does only typed in-memory payload mapping followed by one best-effort `AppHandle::emit` to `analysis://run` or `analysis://chat`. The source contract rejects `async`, `.await`, `block_on`, pool/SQL access, sleep/retry, locks, channels, spawn/join, filesystem/network I/O, or delegation to an unapproved helper in the adapter.

- [ ] **Step 3: Freeze the report tickets and execution API.**

```rust
impl AnalysisReportPreparationTicket {
    pub fn requested_profile_id(&self) -> Option<&str>;
    pub fn model_override(&self) -> Option<&str>;
    pub fn resolve_youtube_corpus_mode(self) -> AppResult<AnalysisReportScopeTicket>;
}

impl AnalysisReportScopeTicket {
    pub fn scope_kind(&self) -> AnalysisScopeKind;
    pub fn scope_id(&self) -> i64;
    pub fn youtube_corpus_mode(&self) -> YoutubeCorpusMode;
}

impl AnalysisReportExecutionTicket {
    pub fn run_id(&self) -> i64;
}

pub async fn prepare_analysis_report(
    pool: &SqlitePool,
    request: StartAnalysisReportRequest,
) -> AppResult<AnalysisReportPreparationTicket>;

pub async fn prepare_analysis_report_execution(
    pool: &SqlitePool,
    state: &AnalysisState,
    reader: &dyn AnalysisCorpusReader,
    preparation: AnalysisReportScopeTicket,
    scope: ResolvedAnalysisScope,
    resolved_profile: ResolvedLlmProfile,
    effective_model: String,
    model_input_token_limit: Option<usize>,
) -> AppResult<AnalysisReportExecutionTicket>;

pub async fn execute_analysis_report(
    pool: &SqlitePool,
    state: &AnalysisState,
    scheduler: Arc<LlmSchedulerState>,
    reader: &dyn AnalysisCorpusReader,
    sink: Arc<dyn AnalysisEventSink>,
    ticket: AnalysisReportExecutionTicket,
) -> Result<(), AnalysisExecutionError>;

pub async fn capture_analysis_corpus(
    pool: &SqlitePool,
    reader: &dyn AnalysisCorpusReader,
    run_id: i64,
    scope_label: &str,
    request: &AnalysisCorpusRequest,
) -> Result<Vec<AnalysisCorpusMessage>, AnalysisExecutionError>;

pub async fn finalize_analysis_report_execution(
    pool: Option<&SqlitePool>,
    state: &AnalysisState,
    sink: &dyn AnalysisEventSink,
    run_id: i64,
    outcome: Result<(), AnalysisExecutionError>,
);
```

`AnalysisReportExecutionTicket` may own `ResolvedLlmProfile`; it exposes no profile or secret getter, has no `Debug`/`Serialize`, and is never logged or persisted. `AnalysisExecutionError` is exactly:

```rust
pub enum AnalysisExecutionError {
    Cancelled(String),
    CaptureFailed(String),
    Failed(String),
}
```

`AnalysisReportPreparationTicket`, `AnalysisReportScopeTicket`, and `AnalysisReportExecutionTicket` are public opaque types with private field layouts, no public construction, and no `Serialize`. Only the consuming transition/accessors above are public.

The app order is mechanically fixed: construct/validate `StartAnalysisReportRequest` before `get_pool`; `prepare_analysis_report` performs template loading only; resolve profile/effective model/input limit; consume the preparation ticket through `resolve_youtube_corpus_mode`; resolve scope from the resulting scope ticket; then call `prepare_analysis_report_execution` (reader A/preflight/duplicate/insert/state), capture `run_id`, spawn one detached task, perform the current post-spawn pool lookup, execute, perform the current terminal lookup, then finalize. The ticket transition makes it impossible to parse YouTube mode before profile or after scope without violating the source contract. `Option<&SqlitePool>` represents the existing fallible terminal lookup; event publication and state cleanup still happen if it is `None`.

`report_start_preserves_acceptance_order_and_two_corpus_reads` uses conflicting failures to pin precedence: invalid period, invalid language, and missing/multiple scope `Option`s fail before pool access; pool/template failure beats profile; invalid profile beats invalid YouTube mode; invalid YouTube mode beats nonpositive/nonexistent scope resolution; only then may preflight/read A fail. It also retains the distinct read-A/read-B assertions.

The retained app `capture_report_corpus_returns_reloaded_snapshot_before_provider_phases` integration test calls public production seam `capture_analysis_corpus` with the real app reader and a real pool. This keeps the adapter-to-snapshot subject; no public test harness or fake substitution is introduced.

- [ ] **Step 4: Freeze the chat tickets and execution API.**

```rust
pub struct AskAnalysisRunQuestionRequest {
    run_id: i64,
    question: String,
    model_override: Option<String>,
    profile_id: Option<String>,
}

impl AskAnalysisRunQuestionRequest {
    pub fn new(
        run_id: i64,
        question: String,
        model_override: Option<String>,
        profile_id: Option<String>,
    ) -> AppResult<Self>;
}

impl AnalysisChatExecutionTicket {
    pub fn request_id(&self) -> &str;
    pub fn profile_id(&self) -> &str;
}

impl AnalysisChatCompletionTicket {
    pub fn request_id(&self) -> &str;
    pub fn run_id(&self) -> i64;
}

pub async fn prepare_analysis_chat(
    pool: &SqlitePool,
    request: AskAnalysisRunQuestionRequest,
    run: AnalysisChatRun,
) -> AppResult<AnalysisChatExecutionTicket>;

pub async fn execute_analysis_chat(
    scheduler: Arc<LlmSchedulerState>,
    sink: Arc<dyn AnalysisEventSink>,
    ticket: AnalysisChatExecutionTicket,
    resolved_profile: ResolvedLlmProfile,
) -> Result<AnalysisChatCompletionTicket, AnalysisExecutionError>;

pub async fn complete_analysis_chat(
    pool: &SqlitePool,
    sink: &dyn AnalysisEventSink,
    completion: AnalysisChatCompletionTicket,
) -> AppResult<()>;

pub fn publish_analysis_chat_execution_error(
    sink: &dyn AnalysisEventSink,
    request_id: &str,
    run_id: i64,
    error: &AnalysisExecutionError,
);

pub fn publish_analysis_chat_persistence_error(
    sink: &dyn AnalysisEventSink,
    request_id: &str,
    run_id: i64,
    error: &AppError,
);
```

`AnalysisChatExecutionTicket` and `AnalysisChatCompletionTicket` are public opaque types with private field layouts, no public construction, and no `Serialize`. Only the accessors above are public.

The two execute functions take owned `Arc<LlmSchedulerState>` and `Arc<dyn AnalysisEventSink>`. The map `JoinSet` and `extractum_llm::LlmSchedulerState::run_request` queue callbacks require owned `'static` captures. The app changes its one Tauri managed value from `LlmSchedulerState::new()` to `Arc::new(LlmSchedulerState::new())`; every internal consumer uses `State<'_, Arc<LlmSchedulerState>>` or `handle.state::<Arc<LlmSchedulerState>>()` and borrows through `as_ref()` when no clone escapes. No second scheduler is constructed. The app creates one `Arc<TauriAnalysisEventSink>` per execution and clones it into detached execution and queue callbacks. This preserves current scheduler identity, `JoinSet` concurrency, and immediate queue/run event publication without a new port, global, buffer, or changed ordering. Non-escaping lifecycle/finalization helpers continue to borrow `&LlmSchedulerState` and `&dyn AnalysisEventSink`.

The app prepares and immediately returns the request ID, spawns, resolves the profile inside that task, executes with interactive priority, then performs the same second pool lookup. `complete_analysis_chat` writes user and assistant turns in one transaction and publishes `completed` only after commit. Lookup/persistence failure publishes the exact current failure prefix.

- [ ] **Step 5: Make portable capabilities explicit.**

Remove `AppHandle` and `get_pool` from portable report/chat/state/phases/capture code. Pass `SqlitePool`, `AnalysisState`, `Arc<LlmSchedulerState>`, `ResolvedLlmProfile`, reader, and sink explicitly. The app alone owns the outer `tokio::spawn`. Internal map `JoinSet`, child tokens, request control, priorities, and request-ID construction remain crate logic.

Compile the prepared portable halves as `chat_engine.rs`, `report_engine.rs`, `report/lifecycle_portable.rs`, and `report/tests/mod_portable.rs`; the original `chat.rs`, `report.rs`, lifecycle wrapper, and test root retain only app integration. The original `report/tests/mod.rs` uses exactly one module-scope `include!("mod_portable.rs")` so moved baseline leaf identities retain the `analysis::report::tests::*` prefix before extraction; the staging file is never declared as `mod mod_portable`. Task 7 removes the include and renames the prepared file mechanically.

Retarget `analysis_report_workflow_file_has_no_tauri_command_adapters` in Checkpoint 4 from runtime `fs::read_to_string("src/analysis/report.rs")` to compile-time `include_str!("../../report_engine.rs")`, so it continues to inspect the portable subject after the split. The dual-owner source contract asserts that exact pre-move subject. Task 7 changes only that literal to final `include_str!("../../report.rs")`; the test identity and assertion body remain unchanged.

Retarget portable chat/report assertions in `analysis-redesign-safety-contract.test.ts` with these explicit pairs: `chat_engine.rs -> crate chat.rs`, `report_engine.rs -> crate report.rs`, and `report/lifecycle_portable.rs -> crate report/lifecycle.rs`. App-adapter assertions remain explicitly app-owned. Run this contract in the same checkpoint.

Normalize imports in every whole-moving report source and test now, while both sides still compile in `extractum`: use depth-stable relative paths for analysis-owned siblings and direct `extractum_core` / `extractum_llm` paths for lower-crate capabilities. In particular, `report/requests.rs`, `report/tests/{harness,lifecycle,phases,preflight,requests,scope}.rs`, and the portable report staging units must contain no `crate::analysis`, `crate::error`, `crate::compression`, `crate::llm`, `crate::time`, or `crate::db` path. Do not defer one of these edits to Task 7.

Widen only these prepared state methods:

```rust
pub async fn insert_active_report_run(&self, run_id: i64);
pub async fn remove_active_report_run(&self, run_id: i64);
pub async fn active_report_run_ids(&self) -> HashSet<i64>;
pub async fn request_report_run_cancel(&self, run_id: i64) -> bool;
pub async fn prepare_report_run_cancellation_wait(
    &self,
    run_id: i64,
) -> Option<AnalysisReportCancellationWait>;

impl AnalysisReportCancellationWait {
    pub async fn cancelled(self);
}
```

`prepare_report_run_cancellation_wait` is awaited before `tokio::spawn`, captures the existing child token inside an opaque non-serializable ticket, and returns `None` if the run has no token. The app then moves the armed ticket into the fixture waiter task and awaits `cancelled()`. This preserves the current capture-before-spawn race guarantee even if cancellation and active-state removal happen before the spawned task is first polled. The token and token map never cross the boundary. Keep `is_report_run_cancelled`, `report_run_child_token`, and `ensure_report_run_token` crate-private. Migrate `spawn_fixture_cancellation_waiters` and its frozen waiter test to this exact handshake before the move.

- [ ] **Step 6: RED/GREEN runtime behavior.**

Add the first four tests below and run them exactly. The fifth (`chat_profile_resolution_failure_is_async_after_request_id`) was added in Checkpoint 1 and is an explicit regression rerun, not a duplicate test:

```text
report_execution_publishes_typed_events_in_existing_order
chat_execution_persists_turns_before_completed_event
terminal_cleanup_always_removes_active_report_state
report_profile_resolution_failure_prevents_run_creation
chat_profile_resolution_failure_is_async_after_request_id
```

Also re-run report phases, lifecycle, chat, and state suites. Assert exact event kinds/phases, progress/message/delta/error fields, request IDs, persistence-before-completed, and cleanup even when terminal persistence/event emission fails.

```powershell
$checkpoint4Green = @(
    'analysis::report::tests::runtime::report_execution_publishes_typed_events_in_existing_order',
    'analysis::chat::tests::chat_execution_persists_turns_before_completed_event',
    'analysis::report::tests::runtime::terminal_cleanup_always_removes_active_report_state',
    'analysis::tests_application::report_profile_resolution_failure_prevents_run_creation',
    'analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id'
)
foreach ($testName in $checkpoint4Green) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
```

- [ ] **Step 7: Run the checkpoint and commit.**

```powershell
Invoke-CheckedNative 'Checkpoint 4 report tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::report::tests::' -- --nocapture }
Invoke-CheckedNative 'Checkpoint 4 chat tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::chat::tests::' -- --nocapture }
Invoke-CheckedNative 'Checkpoint 4 state tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::state::tests::' -- --nocapture }
Invoke-CheckedNative 'Checkpoint 4 source contracts' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 4 format' { cargo fmt --manifest-path src-tauri/Cargo.toml --all }
Invoke-CheckedNative 'Checkpoint 4 package check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 4 package tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

Confirm each selection is non-empty. Update roadmap and shell-cap to `preparation Checkpoint 4 retained`, run the shell-cap contract, inspect, and commit:

```powershell
Invoke-CheckedNative 'Checkpoint 4 status contract' { npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 4 diff check' { git diff --check }
$task4Files = @(
    'src-tauri/src/analysis/events.rs',
    'src-tauri/src/analysis/models.rs',
    'src-tauri/src/analysis/report.rs',
    'src-tauri/src/analysis/report/capture.rs',
    'src-tauri/src/analysis/report/lifecycle.rs',
    'src-tauri/src/analysis/report/phases.rs',
    'src-tauri/src/analysis/report/requests.rs',
    'src-tauri/src/analysis/report/tests/harness.rs',
    'src-tauri/src/analysis/report/tests/lifecycle.rs',
    'src-tauri/src/analysis/report/tests/phases.rs',
    'src-tauri/src/analysis/report/tests/preflight.rs',
    'src-tauri/src/analysis/report/tests/requests.rs',
    'src-tauri/src/analysis/report/tests/scope.rs',
    'src-tauri/src/analysis/chat.rs',
    'src-tauri/src/analysis/state.rs',
    'src-tauri/src/analysis/report_commands.rs',
    'src-tauri/src/analysis/chat_engine.rs',
    'src-tauri/src/analysis/report_engine.rs',
    'src-tauri/src/analysis/report/lifecycle_portable.rs',
    'src-tauri/src/analysis/report/tests/mod_portable.rs',
    'src-tauri/src/analysis/report/tests/runtime.rs',
    'src-tauri/src/analysis/report/tests/mod.rs',
    'src-tauri/src/analysis/report/tests/architecture.rs',
    'src-tauri/src/analysis/fixtures.rs',
    'src-tauri/src/analysis/fixtures/tests/active_runs.rs',
    'src-tauri/src/analysis/tests_application.rs',
    'src-tauri/src/lib.rs',
    'src-tauri/src/accounts.rs',
    'src-tauri/src/llm/mod.rs',
    'src-tauri/src/diagnostics/mod.rs',
    'src-tauri/src/prompt_packs/runtime_commands.rs',
    'src/lib/analysis-application-contract.test.ts',
    'src/lib/analysis-redesign-safety-contract.test.ts',
    'src/lib/crate-extraction-shell-cap-contract.test.ts',
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
)
Add-ScopedChanges -Label 'Checkpoint 4' -Allowed $task4Files
Invoke-CheckedNative 'commit Checkpoint 4' { git commit -m "refactor: introduce analysis runtime boundary" }
$checkpoint4Commit = (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'Checkpoint 4 commit'
```

---

## Frozen 54-File Disposition Map

`A` remains under `src-tauri/src/analysis`; `C` moves to the same relative path under `src-tauri/crates/extractum-analysis/src`; `S` is split during a green preparation checkpoint and leaves both app and crate owners. This map covers the current 54 files exactly. Baseline `mod.rs` is the sole non-one-to-one row: it yields retained app `mod.rs` plus crate `domain.rs` and `tests.rs`; `domain.rs` is counted exactly once in the baseline crate subtotal below. Four plan-added paths sit outside the 54-row map: retained app `tests_application.rs`, plus crate `report/tests/corpus_port.rs`, `report/tests/runtime.rs`, and `test_schema.rs`. The generated crate root `src/lib.rs` is a separate final path created in Task 7.

| # | Current relative path | LOC | Final disposition | Frozen tests |
| ---: | --- | ---: | --- | ---: |
| 1 | `chat.rs` | 695 | S: app command/spawn adapter + `C/chat.rs` engine/store | 6 C |
| 2 | `corpus.rs` | 216 | S: app adapter root + `C/corpus.rs` port/preflight | — |
| 3 | `corpus/live.rs` | 318 | A: `AppAnalysisCorpusReader` | — |
| 4 | `corpus/snapshot.rs` | 316 | C | — |
| 5 | `corpus/source_resolution.rs` | 314 | S: app foreign resolver + C snapshot-first policy | — |
| 6 | `corpus/tests/harness.rs` | 548 | S: app integration harness + crate-private harness | — |
| 7 | `corpus/tests/live.rs` | 683 | S at same relative paths | 1 C / 15 A |
| 8 | `corpus/tests/mod.rs` | 5 | S at same relative paths | — |
| 9 | `corpus/tests/preflight.rs` | 257 | S at same relative paths | 7 C / 3 A |
| 10 | `corpus/tests/snapshot.rs` | 484 | C | 11 C |
| 11 | `corpus/tests/source_resolution.rs` | 234 | S at same relative paths | 1 C / 5 A |
| 12 | `events.rs` | 12 | A: `TauriAnalysisEventSink` | — |
| 13 | `fixtures.rs` | 374 | A | — |
| 14 | `fixtures/seed.rs` | 611 | A | — |
| 15 | `fixtures/seed/runs.rs` | 407 | A | — |
| 16 | `fixtures/tests/active_runs.rs` | 75 | A | 2 A |
| 17 | `fixtures/tests/clear.rs` | 321 | A | 3 A |
| 18 | `fixtures/tests/harness.rs` | 54 | A | 1 A |
| 19 | `fixtures/tests/mod.rs` | 6 | A | — |
| 20 | `fixtures/tests/seed.rs` | 366 | A | 7 A |
| 21 | `fixtures/tests/snapshot.rs` | 226 | A | 4 A |
| 22 | `fixtures/tests/summary.rs` | 25 | A | 1 A |
| 23 | `groups.rs` | 315 | S: app commands/foreign validation/enrichment + `C/groups.rs` domain/CRUD | 3 A |
| 24 | `mod.rs` | 552 | S: app facade/commands + `C/domain.rs` root behavior + `C/tests.rs` root tests | 8 C |
| 25 | `models.rs` | 300 | C | — |
| 26 | `report.rs` | 552 | S: app prepare/spawn/profile/scope + `C/report.rs` engine | — |
| 27 | `report/capture.rs` | 37 | C | — |
| 28 | `report/lifecycle.rs` | 135 | S: app pool wrappers + C lifecycle/storage/cancellation | — |
| 29 | `report/phases.rs` | 389 | C | — |
| 30 | `report/requests.rs` | 266 | C | — |
| 31 | `report/tests/architecture.rs` | 10 | C | 1 C |
| 32 | `report/tests/capture.rs` | 102 | A: real adapter-to-snapshot integration | 1 A |
| 33 | `report/tests/harness.rs` | 135 | C after fixture replacement | — |
| 34 | `report/tests/lifecycle.rs` | 99 | C | 4 C |
| 35 | `report/tests/mod.rs` | 8 | S: app registers capture; crate registers portable modules | — |
| 36 | `report/tests/phases.rs` | 69 | C | 5 C |
| 37 | `report/tests/preflight.rs` | 48 | C | 3 C |
| 38 | `report/tests/requests.rs` | 81 | C | 6 C |
| 39 | `report/tests/scope.rs` | 89 | C | 5 C |
| 40 | `report_commands.rs` | 56 | A | — |
| 41 | `state.rs` | 94 | C | 1 C |
| 42 | `store.rs` | 24 | S: app integration-store root + C owned-store root | — |
| 43 | `store/read_model.rs` | 456 | S: app foreign matcher/enricher + C owned query/mapping | — |
| 44 | `store/runs.rs` | 208 | C | 7 C |
| 45 | `store/setup.rs` | 144 | S: app foreign enrichment + C template/group-owned setup | 1 C / 1 A |
| 46 | `store/snapshot.rs` | 297 | C | 5 C |
| 47 | `store/tests/harness.rs` | 2 | C | — |
| 48 | `store/tests/mod.rs` | 5 | S at same relative paths | — |
| 49 | `store/tests/read_model.rs` | 708 | S at same relative paths | 15 C / 2 A |
| 50 | `store/tests/runs.rs` | 532 | C | 7 C |
| 51 | `store/tests/setup.rs` | 62 | S at same relative paths | 1 C / 1 A |
| 52 | `store/tests/snapshot.rs` | 208 | C | 5 C |
| 53 | `templates.rs` | 200 | S: app commands + `C/templates.rs` validation/CRUD | — |
| 54 | `trace.rs` | 457 | C after core-compression preparation | 8 C |

Reconciliation is exact: 14 whole app files / 2,953 LOC, 20 split files / 6,113 LOC, 20 whole crate files / 4,121 LOC, total 54 / 13,187. The baseline yields 34 current-path app survivors and 41 crate paths because `mod.rs` produces both `domain.rs` and `tests.rs`. Three plan-added crate test/fixture files bring the mechanical move to 44 crate files; the new crate root `lib.rs` makes the expected final topology 35 app files and 45 crate files. Physical lines are measured rather than predicted; Task 9 records actual counts and the boundary contract reports baseline, added, and generated-root paths separately.

Split the mixed tests exactly as follows:

- `corpus/tests/live.rs`: only `youtube_corpus_mode_parses_wire_values_and_defaults` moves; 15 live-adapter tests remain.
- `corpus/tests/preflight.rs`: seven pure calculation/policy tests move; the three loader-backed tests remain.
- `corpus/tests/source_resolution.rs`: `resolve_run_source_ids_prefers_snapshot_over_live_group_membership` moves; five foreign scope/playlist tests remain.
- `store/tests/read_model.rs`: `list_analysis_run_summaries_filters_project_runs` and `list_analysis_run_summaries_matches_all_query_terms_across_any_field` remain; the other 15 frozen identities move.
- `store/tests/setup.rs`: `ensure_sources_exist_returns_typed_not_found_error` remains; `fetch_prompt_template_returns_typed_not_found_error` moves.
- `groups.rs`'s three source-type validation tests remain.
- `report/tests/capture.rs` remains.
- all 18 `fixtures/tests/**` identities remain.
- the three moving no-live-fallback tests may seed foreign sentinel rows privately: `list_run_snapshot_messages_page_does_not_fall_back_to_live_source`, `trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot`, and `load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows`.

Preparation uses explicit temporary filenames so Task 7 performs moves rather than behavioral splits. Create and compile them in these checkpoints:

| Checkpoint | Prepared portable compilation units |
| --- | --- |
| 2 | `domain_portable.rs` for the exact private root domain set; `tests_portable.rs` for the eight root domain tests |
| 3 | `corpus_portable.rs`, `corpus/source_resolution_policy.rs`, `corpus/tests/harness_portable.rs`, `corpus/tests/live_portable.rs`, `corpus/tests/mod_portable.rs`, `corpus/tests/preflight_portable.rs`, `corpus/tests/source_resolution_portable.rs`, `store/owned_read_model.rs`, `store/tests/read_model_portable.rs`, `report/tests/corpus_port.rs` |
| 4 | `chat_engine.rs`, `report_engine.rs`, `report/lifecycle_portable.rs`, `report/tests/mod_portable.rs`, `report/tests/runtime.rs` |
| 5 | `groups_store.rs`, `store_portable.rs`, `store/owned_setup.rs`, `store/tests/mod_portable.rs`, `store/tests/setup_portable.rs`, `templates_store.rs` |

The original mixed path is reduced to its app half in the same green checkpoint. Compile a staging body into its original unsuffixed namespace with exactly one module-scope `include!`, using this exhaustive map:

```text
chat.rs                              -> include!("chat_engine.rs")
analysis/mod.rs module scope         -> include!("domain_portable.rs")
corpus.rs                            -> include!("corpus_portable.rs")
corpus/source_resolution.rs          -> include!("source_resolution_policy.rs")
corpus/tests/harness.rs              -> include!("harness_portable.rs")
corpus/tests/live.rs                 -> include!("live_portable.rs")
corpus/tests/preflight.rs            -> include!("preflight_portable.rs")
corpus/tests/source_resolution.rs    -> include!("source_resolution_portable.rs")
groups.rs                            -> include!("groups_store.rs")
analysis/mod.rs test module          -> include!("tests_portable.rs")
report.rs                            -> include!("report_engine.rs")
report/lifecycle.rs                  -> include!("lifecycle_portable.rs")
report/tests/mod.rs                  -> include!("mod_portable.rs")
store.rs                             -> include!("store_portable.rs")
store/read_model.rs                  -> include!("owned_read_model.rs")
store/setup.rs                       -> include!("owned_setup.rs")
store/tests/read_model.rs            -> include!("read_model_portable.rs")
store/tests/setup.rs                 -> include!("setup_portable.rs")
templates.rs                         -> include!("templates_store.rs")
```

Only `corpus/tests/mod_portable.rs` and `store/tests/mod_portable.rs` remain uncompiled declaration roots until Task 7, because their `live`/`preflight` and `read_model`/`setup` declarations would collide with the same mixed leaf modules still declared by the retained app roots. `report/tests/mod_portable.rs` may compile through `include!` because its portable declarations do not overlap the sole retained app leaf, `capture`. Do not “symmetrize” these three roots. The contract rejects any other undeclared staging unit, double include, suffixed module declaration, or production copy. Temporary filenames and all temporary `include!` lines disappear in Task 7; no shim or copied implementation remains at acceptance.

## Frozen 143-Test Ownership

The implementation does not maintain a second hand-copied identity registry. `analysis-application-contract.test.ts` and the final boundary contract parse the exact leaves and prefixes from Appendix A of the approved specification, then compare them with executable Cargo inventories.

| Final owner/prefix | Count |
| --- | ---: |
| `extractum-analysis::chat::tests` | 6 |
| `extractum-analysis::corpus::tests::live` | 1 |
| `extractum-analysis::corpus::tests::preflight` | 7 |
| `extractum-analysis::corpus::tests::snapshot` | 11 |
| `extractum-analysis::corpus::tests::source_resolution` | 1 |
| `extractum-analysis::report::tests::architecture` | 1 |
| `extractum-analysis::report::tests::lifecycle` | 4 |
| `extractum-analysis::report::tests::phases` | 5 |
| `extractum-analysis::report::tests::preflight` | 3 |
| `extractum-analysis::report::tests::requests` | 6 |
| `extractum-analysis::report::tests::scope` | 5 |
| `extractum-analysis::state::tests` | 1 |
| `extractum-analysis::store::tests::read_model` | 15 |
| `extractum-analysis::store::tests::runs` | 7 |
| `extractum-analysis::store::tests::setup` | 1 |
| `extractum-analysis::store::tests::snapshot` | 5 |
| `extractum-analysis::tests` | 8 |
| `extractum-analysis::trace::tests` | 8 |
| **Crate subtotal** | **95** |
| `extractum::analysis::corpus::tests::live` | 15 |
| `extractum::analysis::corpus::tests::preflight` | 3 |
| `extractum::analysis::corpus::tests::source_resolution` | 5 |
| `extractum::analysis::groups::tests` | 3 |
| `extractum::analysis::report::tests::capture` | 1 |
| `extractum::analysis::store::tests::read_model` | 2 |
| `extractum::analysis::store::tests::setup` | 1 |
| `extractum::analysis::fixtures::tests::active_runs` | 2 |
| `extractum::analysis::fixtures::tests::clear` | 3 |
| `extractum::analysis::fixtures::tests::harness` | 1 |
| `extractum::analysis::fixtures::tests::seed` | 7 |
| `extractum::analysis::fixtures::tests::snapshot` | 4 |
| `extractum::analysis::fixtures::tests::summary` | 1 |
| **App subtotal** | **48** |
| **Frozen total** | **143** |

The contract builds each final full identity as `final prefix + leaf`, proves each once, and rejects renames, disabled tests, duplicates, and copied legacy tests. New tests retain their own identities outside the frozen 143.

## Frozen Public API and Visibility

`src-tauri/crates/extractum-analysis/src/lib.rs` uses private `mod` declarations and explicit `pub use` statements only. The exhaustive public root type allowlist is:

```text
Existing wire/domain DTOs:
AnalysisSourceOption
AnalysisPromptTemplate
AnalysisSourceGroupMember
AnalysisSourceGroup
AnalysisTraceRef
AnalysisTraceData
AnalysisSnapshotState
AnalysisRunSummary
AnalysisRunDetail
AnalysisRunMessageCursor
AnalysisRunMessage
AnalysisRunMessagesPage
AnalysisRunEvent
AnalysisChunkSummaryEvent
AnalysisChatEvent
AnalysisChatTurn
AnalysisChatMessage

Portable values and ports:
AnalysisScopeKind
AnalysisSourceKind
ResolvedAnalysisScope
YoutubeCorpusMode
AnalysisCorpusRequest
AnalysisCorpusMessage
AnalysisRunPreflightLimits
AnalysisRunPreflight
AnalysisPortFuture
AnalysisCorpusReader
AnalysisEventSink
StartAnalysisReportRequest
AnalysisRunListFilters
AskAnalysisRunQuestionRequest
AnalysisReportPreparationTicket
AnalysisReportScopeTicket
AnalysisReportExecutionTicket
AnalysisChatExecutionTicket
AnalysisChatCompletionTicket
AnalysisExecutionError
AnalysisState
AnalysisReportCancellationWait

Foreign composition values:
AnalysisForeignLabelMatch
AnalysisSourceLabel
AnalysisProjectLabel
AnalysisForeignLabels
AnalysisForeignLabelRef
AnalysisRunSummaryEnrichment
AnalysisRunDetailEnrichment
AnalysisChatRunEnrichment
AnalysisChatRun
AnalysisSourceGroupInput
AnalysisSourceGroupRecord
ProjectAnalysisRunAggregate
AnalysisRunDiagnosticCount
```

The exhaustive public root function allowlist is:

```text
prepare_analysis_report
prepare_analysis_report_execution
preflight_analysis_corpus
capture_analysis_corpus
execute_analysis_report
finalize_analysis_report_execution
prepare_analysis_chat
execute_analysis_chat
complete_analysis_chat
publish_analysis_chat_execution_error
publish_analysis_chat_persistence_error

prepare_analysis_run_summaries
prepare_active_analysis_run_summaries
prepare_analysis_run_detail
prepare_legacy_analysis_chat_run
load_analysis_source_groups_for_enrichment
load_analysis_source_group_for_enrichment
load_project_analysis_run_aggregates
delete_project_analysis_runs

list_analysis_prompt_templates
create_analysis_prompt_template
update_analysis_prompt_template
delete_analysis_prompt_template
create_analysis_source_group
update_analysis_source_group
delete_analysis_source_group
get_analysis_source_group_record
list_analysis_run_messages
get_analysis_run_trace
delete_analysis_run
resolve_analysis_trace_refs
list_analysis_chat_messages
clear_analysis_chat_messages
load_analysis_chat_run
analysis_run_ids_depending_on_sources
load_analysis_run_diagnostics
mark_interrupted_analysis_runs
request_analysis_run_cancel
resolve_analysis_telegram_history_scope
```

Constructors and inherent methods named earlier are part of their types, not additional root functions. The app facade may alias crate functions to avoid names colliding with Tauri commands; it must not re-export the crate root publicly.

The cross-domain DTO shapes are exact and contain no implementation rows:

```rust
pub struct AnalysisSourceGroupInput {
    name: String,
    source_kind: AnalysisSourceKind,
    source_ids: Vec<i64>,
}

pub struct AnalysisSourceGroupRecord {
    id: i64,
    name: String,
    source_kind: AnalysisSourceKind,
    member_source_ids: Vec<i64>,
    created_at: i64,
    updated_at: i64,
}

pub struct ProjectAnalysisRunAggregate {
    project_id: i64,
    latest_run_status: Option<String>,
    last_run_at: Option<i64>,
    has_active_run: bool,
}

pub struct AnalysisRunDiagnosticCount {
    provider: String,
    run_type: String,
    scope_type: String,
    status: String,
    snapshot_state: String,
    error_kind: String,
    count: i64,
}
```

Constructors and read-only accessors are exact:

```rust
impl AnalysisSourceGroupInput {
    pub fn new(
        name: String,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
    ) -> AppResult<Self>;
    pub fn name(&self) -> &str;
    pub fn source_kind(&self) -> AnalysisSourceKind;
    pub fn source_ids(&self) -> &[i64];
}

impl AnalysisSourceGroupRecord {
    pub fn id(&self) -> i64;
    pub fn name(&self) -> &str;
    pub fn source_kind(&self) -> AnalysisSourceKind;
    pub fn member_source_ids(&self) -> &[i64];
    pub fn created_at(&self) -> i64;
    pub fn updated_at(&self) -> i64;
}

impl ProjectAnalysisRunAggregate {
    pub fn project_id(&self) -> i64;
    pub fn latest_run_status(&self) -> Option<&str>;
    pub fn last_run_at(&self) -> Option<i64>;
    pub fn has_active_run(&self) -> bool;
}

impl AnalysisRunDiagnosticCount {
    pub fn provider(&self) -> &str;
    pub fn run_type(&self) -> &str;
    pub fn scope_type(&self) -> &str;
    pub fn status(&self) -> &str;
    pub fn snapshot_state(&self) -> &str;
    pub fn error_kind(&self) -> &str;
    pub fn count(&self) -> i64;
}
```

Exact direct visibility changes are:

| Current item | Final visibility |
| --- | --- |
| `AnalysisState::{insert_active_report_run, remove_active_report_run, active_report_run_ids}` | `pub` methods |
| `AnalysisState::request_report_run_cancel` (`pub(super)`) | `pub` narrow lifecycle method |
| new `AnalysisState::prepare_report_run_cancellation_wait` | `pub` narrow fixture lifecycle method returning opaque `AnalysisReportCancellationWait`; no token escape |
| new `AnalysisReportCancellationWait::cancelled` | `pub` consuming wait method; no token getter or serialization |
| `YoutubeCorpusMode` and parser/wire/predicate methods | `pub`, fields/SQL helper remain private |
| `AnalysisRunPreflightLimits` | `pub` opaque type with `Default` and the exact read-only accessors; fields remain private |
| `AnalysisRunPreflight` | `pub` opaque type with the exact read-only accessors; fields remain private |
| `preflight_analysis_run` | renamed `preflight_analysis_corpus` and widened from `pub(crate)` to the curated public corpus-port function |
| `capture_report_corpus` | renamed `capture_analysis_corpus` and widened from `pub(super)` to the curated public corpus-port function |
| `StartAnalysisReportRequest` | `pub` type, private fields |
| `AnalysisRunListFilters` | `pub` type, private fields |
| `resolve_analysis_telegram_history_scope` | `pub` policy function |
| `mark_interrupted_analysis_runs` | `pub` pool function |
| refactored `request_analysis_run_cancel` | `pub` explicit-capability function |
| `CorpusLoadRequest` | renamed `AnalysisCorpusRequest`, constructed public value |
| `CorpusMessage` | renamed `AnalysisCorpusMessage`, constructed public value |
| `ListRunSnapshotMessagesRequest` | stays private; public `list_analysis_run_messages` accepts command-shaped arguments and constructs it internally |

No other `pub(crate) -> pub` or `pub(super) -> pub` widening is allowed. In particular, `AnalysisRunRow`, `AnalysisSourceGroupRow`, `StoredRunSnapshotRow`, `ChunkSummary`, SQL request structs, `ReportPipelineContext`, `ReportRunInput`, `ChatRequestParams`, `AnalysisRunInsert`, `DuplicateRunLookup`, `fetch_run_row`, `map_run_*`, and `set_run_status` remain private or disappear behind curated wrappers.

## Frozen SQL and Transaction API Map

Production crate SQL allowlist, exact:

```text
analysis_runs
analysis_run_messages
analysis_chat_messages
analysis_prompt_templates
analysis_source_groups
analysis_source_group_members
```

Production crate foreign-table denylist includes `analysis_documents`, `sources`, `items`, `telegram_messages`, every YouTube table, `projects`, `project_sources`, prompt-pack/provider/settings/account tables, and every other table not in the six-item allowlist.

Ordinary app-facing crate APIs accept `&SqlitePool`. The only public borrowed-connection participants are:

| Family | Participant, first argument always `&mut SqliteConnection` | App coordinator owning `pool.begin()` and commit/rollback |
| --- | --- | --- |
| 1 | `prepare_analysis_run_summaries` | `list_analysis_runs_in_pool`; also reused by `list_project_runs` through this coordinator |
| 1 | `prepare_active_analysis_run_summaries` | `list_active_analysis_runs_in_pool` |
| 1 | `prepare_analysis_run_detail` | `get_analysis_run_in_pool` |
| 1 | `prepare_legacy_analysis_chat_run` | only legacy branch of `resolve_legacy_analysis_chat_run_in_pool` |
| 2 | `load_analysis_source_groups_for_enrichment` | `list_analysis_source_groups_in_pool` |
| 2 | `load_analysis_source_group_for_enrichment` | `get_analysis_source_group_response_in_pool` and `load_export_source_group_in_pool` |
| 3 | `load_project_analysis_run_aggregates` | existing `projects::read_model::list_research_projects_in_pool` |
| 4 | `delete_project_analysis_runs` | existing `projects::delete_project_in_pool` |

Final participant signatures for Families 2–4 are:

```rust
pub async fn load_analysis_source_groups_for_enrichment(
    conn: &mut SqliteConnection,
) -> AppResult<Vec<AnalysisSourceGroupRecord>>;

pub async fn load_analysis_source_group_for_enrichment(
    conn: &mut SqliteConnection,
    group_id: i64,
) -> AppResult<Option<AnalysisSourceGroupRecord>>;

pub async fn load_project_analysis_run_aggregates(
    conn: &mut SqliteConnection,
    project_ids: &[i64],
) -> AppResult<Vec<ProjectAnalysisRunAggregate>>;

pub async fn delete_project_analysis_runs(
    conn: &mut SqliteConnection,
    project_id: i64,
) -> AppResult<()>;
```

Every app coordinator threads the same `&mut *transaction` through each participating app and crate SQL step and alone commits or rolls back. A bare acquired connection, autocommit sequence, participant pool overload, internal `acquire`, or internal transaction lifecycle is forbidden. A fifth family requires an approved design amendment.

## Frozen Manifest and Test-Schema Contract

The crate manifest is exact:

```toml
[package]
name = "extractum-analysis"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
extractum-core = { path = "../extractum-core" }
extractum-llm = { path = "../extractum-llm" }
serde.workspace = true
serde_json.workspace = true
sqlx.workspace = true
tokio = { workspace = true, features = ["macros", "rt", "sync"] }
tokio-util.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["time"] }
```

No other direct root or crate-local feature is permitted. Specifically reject the app, Tauri/build/plugins, prompt-packs, gemini-browser, direct zstd/reqwest/secrecy/parking_lot, Apalis, Grammers, Windows/process roots, application modules, and tempfile/sha2/time. `sqlx.workspace = true` deliberately inherits the canonical feature set.

The private test fixture has exactly one owner at a time:

```text
pre-move:  src-tauri/src/analysis/test_schema.rs
           include root ../../migrations/
post-move: src-tauri/crates/extractum-analysis/src/test_schema.rs
           include root ../../../migrations/
```

Its Rust `ANALYSIS_TEST_MIGRATIONS: [(&str, &str); 12]` tuple is the single executable allowlist and contains the ordered non-Apalis registry prefix `0001` through `0012`. `analysis-migration-fixture-contract.test.ts` parses that tuple, parses `build_migrations()` up to `apalis_sqlite_migrations()`, resolves every registration to canonical SQL, and asserts exact ordered equality, one include, and one application per entry. It rejects both fixture owners, neither owner, duplicates, unparsed syntax, wrong include root, or unwired entries. TypeScript must not duplicate the twelve filenames. The existing prompt-pack migration parity contract must stay green in the same change.

---

### Task 5: Checkpoint 5 — Isolate Owned SQL, Cross-Domain Transactions, and the Canonical Fixture

**Files:**

- Create: `src-tauri/src/analysis/test_schema.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Create: `src/lib/analysis-migration-fixture-contract.test.ts`
- Create: `src-tauri/src/analysis/groups_store.rs`
- Create: `src-tauri/src/analysis/store_portable.rs`
- Create: `src-tauri/src/analysis/store/owned_setup.rs`
- Create: `src-tauri/src/analysis/store/tests/mod_portable.rs`
- Create: `src-tauri/src/analysis/store/tests/setup_portable.rs`
- Create: `src-tauri/src/analysis/templates_store.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/store/read_model.rs`
- Modify: `src-tauri/src/analysis/store/runs.rs`
- Modify: `src-tauri/src/analysis/store/setup.rs`
- Modify: `src-tauri/src/analysis/store/snapshot.rs`
- Modify: `src-tauri/src/analysis/store/tests/harness.rs`
- Modify: `src-tauri/src/analysis/store/tests/mod.rs`
- Modify: `src-tauri/src/analysis/store/tests/read_model.rs`
- Modify: `src-tauri/src/analysis/store/tests/runs.rs`
- Modify: `src-tauri/src/analysis/store/tests/setup.rs`
- Modify: `src-tauri/src/analysis/store/tests/snapshot.rs`
- Modify: `src-tauri/src/analysis/templates.rs`
- Modify: `src-tauri/src/analysis/groups.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `src-tauri/src/analysis/corpus/snapshot.rs`
- Modify: `src-tauri/src/analysis/report/lifecycle.rs`
- Modify: `src-tauri/src/projects/read_model.rs`
- Modify: `src-tauri/src/projects/mod.rs`
- Modify: `src-tauri/src/account_deletion.rs`
- Modify: `src-tauri/src/diagnostics/database.rs`
- Modify: `src-tauri/src/notebooklm_export/query.rs`
- Modify: `src/lib/analysis-application-contract.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`
- Verify only: `src/lib/prompt-pack-crate-boundary-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`

- [ ] **Step 1: Add the separately green migration-fixture contract first.**

Write `analysis-migration-fixture-contract.test.ts` with the exact sentinel title `requires exactly one analysis fixture owner` and parser-unit titles `rejects duplicate and malformed migration fixture syntax` and `parses the non-Apalis registry prefix fail closed`. The parser tests use inline strings and must be GREEN before any fixture exists. Run only the sentinel to prove the intended missing-owner RED:

```powershell
Invoke-CheckedNative 'fixture parser rejects malformed syntax' { npm.cmd run test -- src/lib/analysis-migration-fixture-contract.test.ts -t 'rejects duplicate and malformed migration fixture syntax' }
Invoke-CheckedNative 'fixture parser reads registry prefix fail closed' { npm.cmd run test -- src/lib/analysis-migration-fixture-contract.test.ts -t 'parses the non-Apalis registry prefix fail closed' }
$fixtureRed = @(npm.cmd run test -- src/lib/analysis-migration-fixture-contract.test.ts -t 'requires exactly one analysis fixture owner' 2>&1)
$fixtureExit = $LASTEXITCODE
$fixtureRed | Out-Host
if ($fixtureExit -eq 0) { throw 'Expected missing-fixture RED' }
if (($fixtureRed -join "`n") -match 'No test files found|0 tests') { throw 'Fixture contract did not execute' }
$fixtureRedText = ($fixtureRed -join "`n")
if ($fixtureRedText -notmatch 'expected exactly one analysis fixture owner; found 0') { throw 'Unexpected fixture RED count' }
if ($fixtureRedText -notmatch 'analysis/test_schema.rs') { throw 'Unexpected fixture RED path' }
```

- [ ] **Step 2: Create the private canonical fixture and turn its contract GREEN.**

Create `test_schema.rs`, register it privately under `#[cfg(test)]`, and add a minimal compiling fixture seam. Write and finish the first exact runtime RED/GREEN before writing the second; each test uses the unique marker below:

```powershell
Assert-ExactRustRuntimeRed extractum 'analysis::test_schema::tests::canonical_fixture_applies_analysis_consumed_schema' 'src-tauri/src/analysis/test_schema.rs' 'RED: CP5 consumed canonical schema'
# Implement case 1 and run GREEN before adding case 2.
Invoke-ExactRustTest extractum 'analysis::test_schema::tests::canonical_fixture_applies_analysis_consumed_schema'
Assert-ExactRustRuntimeRed extractum 'analysis::test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys' 'src-tauri/src/analysis/test_schema.rs' 'RED: CP5 canonical indexes and foreign keys'
# Implement case 2 and run GREEN.
Invoke-ExactRustTest extractum 'analysis::test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys'
```

At `src-tauri/src/analysis/test_schema.rs`, define this private list exactly:

```rust
#[cfg(test)]
const ANALYSIS_TEST_MIGRATIONS: [(&str, &str); 12] = [
    ("src-tauri/migrations/0001_current_schema_baseline.sql", include_str!("../../migrations/0001_current_schema_baseline.sql")),
    ("src-tauri/migrations/0002_migrated_history_opt_in_schema.sql", include_str!("../../migrations/0002_migrated_history_opt_in_schema.sql")),
    ("src-tauri/migrations/0003_analysis_telegram_history_scope.sql", include_str!("../../migrations/0003_analysis_telegram_history_scope.sql")),
    ("src-tauri/migrations/0004_source_delete_cascade_indexes.sql", include_str!("../../migrations/0004_source_delete_cascade_indexes.sql")),
    ("src-tauri/migrations/0005_projects_mvp.sql", include_str!("../../migrations/0005_projects_mvp.sql")),
    ("src-tauri/migrations/0006_prompt_pack_mvp.sql", include_str!("../../migrations/0006_prompt_pack_mvp.sql")),
    ("src-tauri/migrations/0007_prompt_pack_run_idempotency.sql", include_str!("../../migrations/0007_prompt_pack_run_idempotency.sql")),
    ("src-tauri/migrations/0008_prompt_pack_run_labels.sql", include_str!("../../migrations/0008_prompt_pack_run_labels.sql")),
    ("src-tauri/migrations/0009_prompt_pack_intermediate_entities_artifacts.sql", include_str!("../../migrations/0009_prompt_pack_intermediate_entities_artifacts.sql")),
    ("src-tauri/migrations/0010_prompt_pack_runtime_provider.sql", include_str!("../../migrations/0010_prompt_pack_runtime_provider.sql")),
    ("src-tauri/migrations/0011_prompt_pack_stage_browser_provenance.sql", include_str!("../../migrations/0011_prompt_pack_stage_browser_provenance.sql")),
    ("src-tauri/migrations/0012_projects_redesign.sql", include_str!("../../migrations/0012_projects_redesign.sql")),
];
```

Apply each SQL text once in order with `sqlx::raw_sql` inside one fixture-owned transaction, and commit only after all entries succeed. Do not copy SQL or import the application migration runner.

Add exact Rust tests:

```text
canonical_fixture_applies_analysis_consumed_schema
canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys
```

The first characterizes only columns/shapes consumed by crate-owned code. The second pins relevant indexes and cross-domain foreign-key/cascade behavior without claiming the crate owns migrations.

```powershell
Invoke-ExactRustTest -Package extractum -TestName 'analysis::test_schema::tests::canonical_fixture_applies_analysis_consumed_schema'
Invoke-ExactRustTest -Package extractum -TestName 'analysis::test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys'
Invoke-CheckedNative 'fixture and independent migration contracts' { npm.cmd run test -- src/lib/analysis-migration-fixture-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts }
```

Expected: both independent registry-parity contracts pass.

- [ ] **Step 3: Convert crate-owned tests to the canonical fixture.**

Use `test_schema` for the moving root/report/corpus/store tests. Retain minimal hand-written schemas only in explicitly named isolated failure/transaction tests and mark each with a `// partial schema:` comment that states its concrete isolation reason. The three no-live-fallback tests may insert foreign sentinel rows after applying canonical SQL, but must not call an app source builder. The 18 app fixture identities continue to use the full app migration helper.

Resolve the two canonical-FK edge cases without adding foreign-table SQL. `resolve_run_source_ids_prefers_snapshot_over_live_group_membership` compares a non-empty stored snapshot with an empty live membership and still proves snapshot precedence. `source_group_membership_drift_after_capture_does_not_change_saved_run_corpus` uses a private test-only helper on an isolated `max_connections(1)` pool: acquire its sole connection, set `PRAGMA foreign_keys=OFF`, insert only the orphan row in owned `analysis_source_group_members`, restore `PRAGMA foreign_keys=ON` on both success and failure paths, assert the pragma reads `1`, then release the connection. The contract permits this PRAGMA helper only for that exact identity, rejects any foreign-table token in it, and rejects disabled foreign keys anywhere else. It is not exported or shared with production.

- [ ] **Step 4: Isolate all ordinary six-table storage APIs.**

Move production SQL over the six allowlisted tables behind the curated pool functions while code remains in the app package. Required ordinary signatures include:

```rust
pub async fn list_analysis_prompt_templates(
    pool: &SqlitePool,
    template_kind: Option<String>,
) -> AppResult<Vec<AnalysisPromptTemplate>>;

pub async fn create_analysis_prompt_template(
    pool: &SqlitePool,
    name: String,
    template_kind: String,
    body: String,
) -> AppResult<AnalysisPromptTemplate>;

pub async fn update_analysis_prompt_template(
    pool: &SqlitePool,
    template_id: i64,
    name: String,
    body: String,
) -> AppResult<AnalysisPromptTemplate>;

pub async fn delete_analysis_prompt_template(
    pool: &SqlitePool,
    template_id: i64,
) -> AppResult<()>;

pub async fn create_analysis_source_group(
    pool: &SqlitePool,
    input: AnalysisSourceGroupInput,
) -> AppResult<i64>;

pub async fn update_analysis_source_group(
    pool: &SqlitePool,
    group_id: i64,
    input: AnalysisSourceGroupInput,
) -> AppResult<()>;

pub async fn delete_analysis_source_group(
    pool: &SqlitePool,
    group_id: i64,
) -> AppResult<()>;

pub async fn get_analysis_source_group_record(
    pool: &SqlitePool,
    group_id: i64,
) -> AppResult<Option<AnalysisSourceGroupRecord>>;
```

`AnalysisSourceGroupInput::new(name, source_kind, source_ids)` preserves the current normalization exactly: trim and validate the name, discard nonpositive IDs, sort IDs numerically, deduplicate, then require at least one ID. Pin `[4, 2, 4, -1, 2] -> [2, 4]`. This is intentionally distinct from `ResolvedAnalysisScope`, whose source order remains stable first-seen. The app validates foreign IDs/types before writes. Ordinary run message, trace, trace-ref, run-delete, chat-list/clear, report lifecycle, duplicate, snapshot, and persistence APIs also accept the pool; their existing command parameters and responses remain unchanged.

The ordinary command-facing signatures are:

```rust
pub async fn list_analysis_run_messages(
    pool: &SqlitePool,
    run_id: i64,
    after: Option<AnalysisRunMessageCursor>,
    limit: Option<i64>,
    source_id: Option<i64>,
    around_ref: Option<String>,
) -> AppResult<AnalysisRunMessagesPage>;

pub async fn get_analysis_run_trace(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<AnalysisTraceData>;

pub async fn delete_analysis_run(
    pool: &SqlitePool,
    state: &AnalysisState,
    run_id: i64,
) -> AppResult<()>;

pub async fn resolve_analysis_trace_refs(
    pool: &SqlitePool,
    run_id: i64,
    refs: Vec<String>,
) -> AppResult<Vec<AnalysisTraceRef>>;

pub async fn list_analysis_chat_messages(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<AnalysisChatMessage>>;

pub async fn clear_analysis_chat_messages(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<()>;
```

Compile owned group/template/store roots and portable test bodies through `groups_store.rs`, `templates_store.rs`, `store_portable.rs`, `store/owned_setup.rs`, and `store/tests/setup_portable.rs`. `store/tests/mod_portable.rs` is the contract-validated, not-yet-compiled final crate declaration root. Original mixed paths retain only app commands, foreign validation/enrichment, and app tests. Before extraction, original `store/tests/setup.rs` includes `setup_portable.rs` exactly once at module scope and does not declare a suffixed module; Task 7 removes that include while moving both staging files to unsuffixed crate paths. The boundary contract proves exact Appendix A identities both before and after the switch.

- [ ] **Step 5: Implement Family 2 group enrichment transactions.**

Refactor `list_analysis_source_groups`, create/update response reload, and NotebookLM group export so each named app coordinator opens one explicit read transaction, calls the appropriate group-record participant, loads `sources` titles/types and `items` counts on the same `&mut *transaction`, constructs the unchanged response, and commits. Report scope resolution uses ordinary `get_analysis_source_group_record(&SqlitePool, id)` because it consumes owned membership IDs without foreign enrichment.

Run the three retained group-validation tests and the NotebookLM ordered-member test. Pin that creation/update writes stay crate-owned pool operations and only their enriched response reload uses Family 2.

- [ ] **Step 6: Implement Family 3 project-list composition.**

In `projects::read_model::list_research_projects_in_pool`, open one transaction; read app-owned project rows and material counts with `&mut *transaction`; call `load_project_analysis_run_aggregates(&mut *transaction, &project_ids)`; compose unchanged `ProjectSummary` values; commit. `ProjectAnalysisRunAggregate` exposes only `project_id`, latest status, last-run timestamp, and active flag. Remove raw `analysis_runs` SQL from the project module.

- [ ] **Step 7: Implement Family 4 project deletion.**

Keep `projects::delete_project_in_pool` as transaction owner. Within the same transaction, call `delete_project_analysis_runs(&mut *transaction, project_id)`, then delete app-owned membership/project rows, and alone commit. Do not give the participant a pool overload or its own transaction.

- [ ] **Step 8: Replace the remaining cross-domain analysis SQL.**

Use these ordinary pool APIs:

```rust
pub async fn analysis_run_ids_depending_on_sources(
    pool: &SqlitePool,
    candidate_run_ids: &HashSet<i64>,
    owned_source_ids: &[i64],
) -> AppResult<BTreeSet<i64>>;

pub async fn load_analysis_run_diagnostics(
    pool: &SqlitePool,
) -> AppResult<Vec<AnalysisRunDiagnosticCount>>;

pub async fn mark_interrupted_analysis_runs(
    pool: &SqlitePool,
) -> AppResult<()>;

pub async fn request_analysis_run_cancel(
    pool: &SqlitePool,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    sink: &dyn AnalysisEventSink,
    run_id: i64,
) -> AppResult<()>;
```

Account deletion supplies current active run IDs and account-owned source IDs; preserve the known project-scope blind spot. Diagnostics receive only coarse `error_kind` counts and never raw error text. Startup cleanup remains silent. Dev fixtures stay app-owned and may use their dev-only cross-domain SQL, but use the narrow state lifecycle helper instead of token-map mutation.

- [ ] **Step 9: Prove ownership with fail-closed source scans.**

`analysis-application-contract.test.ts` must prove that every production query/DML token for the six tables is in a prepared portable owner only; app transaction coordinators may call borrowed crate participants but may contain no raw SQL naming those six tables. Every foreign-table token remains in app adapters. Exclude only parsed `#[cfg(test)]` and `#[cfg(dev)]` fixture/setup regions, never entire production files. Assert all eight participants' first parameter, no participant pool/acquire/begin/commit/rollback, and all nine coordinator call chains (four Family 1, three Family 2 callers, one Family 3, one Family 4).

Add the standing case `keeps every mechanical-move source portable before extraction`. It consumes the exact whole-move and prepared-staging source allowlists from the frozen disposition/include maps, fails closed when a listed file is absent or an undeclared portable source appears, and rejects `crate::analysis`, `crate::error`, `crate::compression`, `crate::llm`, `crate::time`, `crate::db`, Tauri, `AppHandle`, or `get_pool` in every listed source. Analysis-owned siblings use depth-stable relative imports; lower capabilities use `extractum_core` or `extractum_llm` directly. Assert specifically that `store/owned_read_model.rs` imports `ymd_to_unix_midnight` from `extractum_core::time`. This case is GREEN in Checkpoint 5 and stays GREEN before the intentional boundary RED; Task 7 may not repair an import in a whole-moving file.

Scan crate tests separately rather than ignoring all `#[cfg(test)]`. Foreign-table SQL is allowed only inside the setup bodies of the three named no-live-fallback identities, only for inserting/deleting negative sentinel rows after canonical schema application. The separate `PRAGMA foreign_keys=OFF/ON` allowance belongs only to the private helper called by `source_group_membership_drift_after_capture_does_not_change_saved_run_corpus`, must restore and assert `ON`, and may contain no foreign-table SQL. Any foreign query in another crate test, foreign-keys disablement elsewhere, read of a foreign sentinel by production code, or import/call to an app source builder is RED.

- [ ] **Step 10: Run all 143 baseline identities and cross-domain witnesses.**

```powershell
Invoke-CheckedNative 'Checkpoint 5 contracts' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 5 portable-source contract' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts -t 'keeps every mechanical-move source portable before extraction' }
Invoke-CheckedNative 'Checkpoint 5 identity list' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::' -- --list }
Invoke-CheckedNative 'Checkpoint 5 format' { cargo fmt --manifest-path src-tauri/Cargo.toml --all }
Invoke-CheckedNative 'Checkpoint 5 package tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 5 package check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
$checkpoint5Witnesses = @(
    'analysis::groups::tests::validate_group_source_type_accepts_matching_provider_membership',
    'analysis::groups::tests::validate_group_source_type_rejects_mixed_provider_membership',
    'analysis::groups::tests::validate_group_source_type_rejects_unknown_group_type',
    'analysis::corpus::tests::snapshot::list_run_snapshot_messages_page_does_not_fall_back_to_live_source',
    'analysis::corpus::tests::snapshot::load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows',
    'analysis::corpus::tests::snapshot::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot',
    'analysis::corpus::tests::snapshot::source_group_membership_drift_after_capture_does_not_change_saved_run_corpus',
    'analysis::corpus::tests::source_resolution::resolve_run_source_ids_prefers_snapshot_over_live_group_membership',
    'analysis::report::tests::lifecycle::request_analysis_run_cancel_running_but_inactive_keeps_conflict_message',
    'projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources',
    'projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout',
    'account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned',
    'diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates',
    'notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id',
    'analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent'
)
foreach ($testName in $checkpoint5Witnesses) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
```

The contract must report all frozen 143 once under the still-app package plus the new tests separately. The exact array above covers group validation, project deletion/list, account deletion, diagnostics, NotebookLM, fixture clear, cancellation, the three no-live-fallback tests, and both canonical-FK edge cases; the full package run covers the remaining fixture seed identities.

- [ ] **Step 11: Advance status and commit the green checkpoint.**

Update roadmap and shell-cap to `preparation Checkpoint 5 retained`, then:

```powershell
Invoke-CheckedNative 'Checkpoint 5 status contracts' { npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts }
Invoke-CheckedNative 'Checkpoint 5 diff check' { git diff --check }
git status --short
$task5Files = @(
    'src-tauri/src/analysis/test_schema.rs',
    'src-tauri/src/analysis/mod.rs',
    'src-tauri/src/analysis/groups_store.rs',
    'src-tauri/src/analysis/store_portable.rs',
    'src-tauri/src/analysis/store/owned_setup.rs',
    'src-tauri/src/analysis/store/tests/mod_portable.rs',
    'src-tauri/src/analysis/store/tests/setup_portable.rs',
    'src-tauri/src/analysis/templates_store.rs',
    'src-tauri/src/analysis/store.rs',
    'src-tauri/src/analysis/store/read_model.rs',
    'src-tauri/src/analysis/store/runs.rs',
    'src-tauri/src/analysis/store/setup.rs',
    'src-tauri/src/analysis/store/snapshot.rs',
    'src-tauri/src/analysis/store/tests/harness.rs',
    'src-tauri/src/analysis/store/tests/mod.rs',
    'src-tauri/src/analysis/store/tests/read_model.rs',
    'src-tauri/src/analysis/store/tests/runs.rs',
    'src-tauri/src/analysis/store/tests/setup.rs',
    'src-tauri/src/analysis/store/tests/snapshot.rs',
    'src-tauri/src/analysis/templates.rs',
    'src-tauri/src/analysis/groups.rs',
    'src-tauri/src/analysis/chat.rs',
    'src-tauri/src/analysis/corpus/snapshot.rs',
    'src-tauri/src/analysis/report/lifecycle.rs',
    'src-tauri/src/projects/read_model.rs',
    'src-tauri/src/projects/mod.rs',
    'src-tauri/src/account_deletion.rs',
    'src-tauri/src/diagnostics/database.rs',
    'src-tauri/src/notebooklm_export/query.rs',
    'src/lib/analysis-application-contract.test.ts',
    'src/lib/analysis-redesign-safety-contract.test.ts',
    'src/lib/analysis-migration-fixture-contract.test.ts',
    'src/lib/crate-extraction-shell-cap-contract.test.ts',
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
)
Add-ScopedChanges -Label 'Checkpoint 5' -Allowed $task5Files
Invoke-CheckedNative 'commit Checkpoint 5' { git commit -m "refactor: isolate analysis storage boundary" }
$checkpoint5Commit = (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'Checkpoint 5 commit'
```

---

### Task 6: Checkpoint 6 — Commit the Intentional RED Crate Boundary Contract

**Files:**

- Create: `src/lib/analysis-crate-boundary-contract.test.ts`
- Modify: `src/lib/rust-workspace-core-contract.test.ts`
- Modify: `src/lib/gemini-browser-crate-boundary-contract.test.ts`
- Modify: `src/lib/llm-crate-boundary-contract.test.ts`
- Modify: `src/lib/prompt-pack-crate-boundary-contract.test.ts`
- Modify: `src/lib/development-loop-performance-contract.test.ts`
- Modify: `src/lib/focused-rust-loop-contract.test.ts`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`
- Modify: `src/lib/analysis-contract-paths.ts`

- [ ] **Step 1: Implement the exact boundary cases.**

Use these exact test titles:

```text
requires the extractum-analysis manifest and mechanical move
declares one app edge and the exact locked dependency surface
keeps a curated crate API and exhaustive visibility allowlist
moves every frozen baseline identity to its approved 95/48 owner exactly once
rejects disabled renamed or copied legacy analysis tests
keeps production SQL in the exact six-table owner
pins pool APIs and exactly four borrowed-connection workflow families
threads one app-owned transaction through every approved coordinator
pins resolved scope corpus and distinct read A/read B handoffs
keeps commands events spawning migrations and fixtures app-owned
keeps the event adapter synchronous bounded and nonblocking
keeps the separately green migration fixture contract
keeps trace compression core-owned and the app free of direct zstd
keeps command event and AppError wire contracts unchanged
moves only pre-normalized portable sources
```

The contract consumes the frozen maps in this plan and Appendix A directly. It asserts all 54 baseline dispositions, including both crate outputs of baseline `mod.rs`, separately from the four plan-added paths and the generated crate root `lib.rs`; post-move it requires 35 app Rust files and 45 crate Rust files. It rejects public modules/globs/test helpers/rows, forbidden dependencies, reverse lower-crate edges, foreign production SQL, a fifth transaction family, copied source files, disabled/renamed tests, wrong lock roots, missing `Cargo.lock` hunk, and weakened dual-path safety assertions.

`moves only pre-normalized portable sources` reuses the exact fail-closed source set from the Checkpoint 5 standing contract. Before extraction it inspects every whole-move and staging source; afterward it inspects every corresponding crate destination. It rejects the six app-root prefixes, Tauri/`AppHandle`/`get_pool`, and a missing or extra mapped source in both states, including the direct `extractum_core::time::ymd_to_unix_midnight` ownership assertion.

- [ ] **Step 2: Update existing workspace contracts with an exact two-state branch.**

Before `src-tauri/crates/extractum-analysis/Cargo.toml` exists, require the current exact member/dependency lists. After it exists, require exactly one appended `crates/extractum-analysis` member, exactly one app path dependency, and no reverse lower-crate edge. This is not a permissive union.

Update core, Gemini Browser, LLM, prompt-pack, focused-loop, development-loop, shell-cap, and analysis safety contracts. Do not add a `package.json` script: the approved Phase 7 design defines none. Keep the status state machine independent from manifest existence because Checkpoint 7 exists while the roadmap still truthfully says Checkpoint 5 until final evidence.

- [ ] **Step 3: Assert the future manifest and lockfile exactly.**

The contract requires the frozen manifest, one lock package named `extractum-analysis` with dependencies exactly `extractum-core`, `extractum-llm`, `serde`, `serde_json`, `sqlx`, `tokio`, and `tokio-util`, and no registry `source`/`checksum`. It requires the app lock package to gain `extractum-analysis`, the app manifest to lose direct `zstd`, and every lower package to reject an analysis edge.

- [ ] **Step 4: Prove RED for one intended reason only.**

```powershell
$boundaryRed = @(npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t 'requires the extractum-analysis manifest and mechanical move' 2>&1)
$boundaryExit = $LASTEXITCODE
$boundaryRed | Out-Host
if ($boundaryExit -eq 0) { throw 'Expected analysis boundary RED' }
if (($boundaryRed -join "`n") -match 'No test files found|0 tests') { throw 'Boundary contract did not execute' }
if (($boundaryRed -join "`n") -notmatch 'extractum-analysis Cargo.toml is intentionally absent before the mechanical move') {
    throw 'Unexpected analysis boundary RED reason'
}
```

Prove that every non-sentinel case is GREEN in the pre-move state; do not rely on a whole-file result that mixes failures:

```powershell
$boundaryGreenTitles = @(
    'declares one app edge and the exact locked dependency surface',
    'keeps a curated crate API and exhaustive visibility allowlist',
    'moves every frozen baseline identity to its approved 95/48 owner exactly once',
    'rejects disabled renamed or copied legacy analysis tests',
    'keeps production SQL in the exact six-table owner',
    'pins pool APIs and exactly four borrowed-connection workflow families',
    'threads one app-owned transaction through every approved coordinator',
    'pins resolved scope corpus and distinct read A/read B handoffs',
    'keeps commands events spawning migrations and fixtures app-owned',
    'keeps the event adapter synchronous bounded and nonblocking',
    'keeps the separately green migration fixture contract',
    'keeps trace compression core-owned and the app free of direct zstd',
    'keeps command event and AppError wire contracts unchanged',
    'moves only pre-normalized portable sources'
)
foreach ($title in $boundaryGreenTitles) {
    Invoke-CheckedNative "pre-move boundary case: $title" { npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t $title }
}
```

Then prove every standing contract remains green:

```powershell
Invoke-CheckedNative 'pre-move standing contracts' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/development-loop-performance-contract.test.ts src/lib/focused-rust-loop-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts }
Invoke-CheckedNative 'pre-move app check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

- [ ] **Step 5: Commit RED separately.**

```powershell
Invoke-CheckedNative 'RED contract diff check' { git diff --check }
git status --short
$task6Files = @(
    'src/lib/analysis-crate-boundary-contract.test.ts',
    'src/lib/rust-workspace-core-contract.test.ts',
    'src/lib/gemini-browser-crate-boundary-contract.test.ts',
    'src/lib/llm-crate-boundary-contract.test.ts',
    'src/lib/prompt-pack-crate-boundary-contract.test.ts',
    'src/lib/development-loop-performance-contract.test.ts',
    'src/lib/focused-rust-loop-contract.test.ts',
    'src/lib/crate-extraction-shell-cap-contract.test.ts',
    'src/lib/analysis-redesign-safety-contract.test.ts',
    'src/lib/analysis-contract-paths.ts'
)
Add-ScopedChanges -Label 'RED boundary contract' -Allowed $task6Files
Invoke-CheckedNative 'commit RED boundary contract' { git commit -m "test: define analysis crate boundary" }
$redContractCommit = (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'RED boundary contract commit'
```

Do not leave any other failing test in this commit. If work pauses here, ordinary `git revert` this RED commit before handing back a green branch.

---

### Task 7: Checkpoint 7 — Perform the Mechanical Crate Extraction

**Files:**

- Create: `src-tauri/crates/extractum-analysis/Cargo.toml`
- Create: `src-tauri/crates/extractum-analysis/src/lib.rs`
- Move: every crate-owned file and prepared portable half in the frozen map
- Retain/modify: every app-owned file and prepared app half in the frozen map
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: contract files from Task 6 only for their prepared owner switch

- [ ] **Step 1: Reconfirm the clean prepared state.**

```powershell
Assert-CleanWorktree 'mechanical move start'
Invoke-CheckedNative 'prepared standing contracts' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts }
$boundaryRed = @(npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t 'requires the extractum-analysis manifest and mechanical move' 2>&1)
$boundaryExit = $LASTEXITCODE
if ($boundaryExit -eq 0) { throw 'Boundary sentinel must still be RED before the move' }
if (($boundaryRed -join "`n") -match 'No test files found|0 tests') { throw 'Boundary sentinel did not execute' }
if (($boundaryRed -join "`n") -notmatch 'extractum-analysis Cargo.toml is intentionally absent before the mechanical move') { throw 'Boundary RED drifted' }
Invoke-CheckedNative 'prepared app check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

- [ ] **Step 2: Verify every split is already a separate green compilation unit.**

Before moving anything, require these exact preparation staging files, created by Checkpoints 1–5. Production units and portable leaf test bodies are compiled by the app package; the collision-prone declaration roots `corpus/tests/mod_portable.rs` and `store/tests/mod_portable.rs` are parsed exactly by the contract but remain uncompiled until the move:

```text
chat_engine.rs
domain_portable.rs
corpus_portable.rs
corpus/source_resolution_policy.rs
corpus/tests/harness_portable.rs
corpus/tests/live_portable.rs
corpus/tests/mod_portable.rs
corpus/tests/preflight_portable.rs
corpus/tests/source_resolution_portable.rs
groups_store.rs
tests_portable.rs
report_engine.rs
report/lifecycle_portable.rs
report/tests/mod_portable.rs
report/tests/corpus_port.rs
report/tests/runtime.rs
store_portable.rs
store/owned_read_model.rs
store/owned_setup.rs
store/tests/mod_portable.rs
store/tests/read_model_portable.rs
store/tests/setup_portable.rs
templates_store.rs
```

The corresponding original paths already contain only the retained app halves. Before moving, the contract proves the exhaustive include map above and separately proves the exact declarations in the two uncompiled root staging files. After each compiled staging `git mv`, remove only its matching temporary `include!` line from the retained app file; the moved crate module owns the same body directly. If any portable behavior is still interleaved with an app adapter, stop and amend the owning green checkpoint; do not split it during this task.

- [ ] **Step 3: Create the exact manifest and private crate root.**

Create the frozen manifest verbatim. In `lib.rs`, declare every implementation module with private `mod`, gate only `test_schema` with `#[cfg(test)]`, and write explicit `pub use` statements matching the frozen public allowlists. Reject `pub mod`, glob re-exports, and public test support.

- [ ] **Step 4: Create exact destination directories and move all whole crate files.**

```powershell
$crateSource = (Resolve-Path 'src-tauri').Path + '\crates\extractum-analysis\src'
$destinationDirs = @(
    $crateSource,
    "$crateSource\corpus",
    "$crateSource\corpus\tests",
    "$crateSource\report",
    "$crateSource\report\tests",
    "$crateSource\store",
    "$crateSource\store\tests"
)
foreach ($directory in $destinationDirs) {
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
}

$wholeMoves = @(
    @('src-tauri/src/analysis/corpus/snapshot.rs', 'src-tauri/crates/extractum-analysis/src/corpus/snapshot.rs'),
    @('src-tauri/src/analysis/corpus/tests/snapshot.rs', 'src-tauri/crates/extractum-analysis/src/corpus/tests/snapshot.rs'),
    @('src-tauri/src/analysis/models.rs', 'src-tauri/crates/extractum-analysis/src/models.rs'),
    @('src-tauri/src/analysis/report/capture.rs', 'src-tauri/crates/extractum-analysis/src/report/capture.rs'),
    @('src-tauri/src/analysis/report/phases.rs', 'src-tauri/crates/extractum-analysis/src/report/phases.rs'),
    @('src-tauri/src/analysis/report/requests.rs', 'src-tauri/crates/extractum-analysis/src/report/requests.rs'),
    @('src-tauri/src/analysis/report/tests/architecture.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/architecture.rs'),
    @('src-tauri/src/analysis/report/tests/harness.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/harness.rs'),
    @('src-tauri/src/analysis/report/tests/lifecycle.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/lifecycle.rs'),
    @('src-tauri/src/analysis/report/tests/phases.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/phases.rs'),
    @('src-tauri/src/analysis/report/tests/preflight.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/preflight.rs'),
    @('src-tauri/src/analysis/report/tests/requests.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/requests.rs'),
    @('src-tauri/src/analysis/report/tests/scope.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/scope.rs'),
    @('src-tauri/src/analysis/report/tests/corpus_port.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/corpus_port.rs'),
    @('src-tauri/src/analysis/report/tests/runtime.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/runtime.rs'),
    @('src-tauri/src/analysis/state.rs', 'src-tauri/crates/extractum-analysis/src/state.rs'),
    @('src-tauri/src/analysis/store/runs.rs', 'src-tauri/crates/extractum-analysis/src/store/runs.rs'),
    @('src-tauri/src/analysis/store/snapshot.rs', 'src-tauri/crates/extractum-analysis/src/store/snapshot.rs'),
    @('src-tauri/src/analysis/store/tests/harness.rs', 'src-tauri/crates/extractum-analysis/src/store/tests/harness.rs'),
    @('src-tauri/src/analysis/store/tests/runs.rs', 'src-tauri/crates/extractum-analysis/src/store/tests/runs.rs'),
    @('src-tauri/src/analysis/store/tests/snapshot.rs', 'src-tauri/crates/extractum-analysis/src/store/tests/snapshot.rs'),
    @('src-tauri/src/analysis/trace.rs', 'src-tauri/crates/extractum-analysis/src/trace.rs'),
    @('src-tauri/src/analysis/test_schema.rs', 'src-tauri/crates/extractum-analysis/src/test_schema.rs')
)
foreach ($move in $wholeMoves) {
    if (-not (Test-Path -LiteralPath $move[0])) { throw "Missing frozen move source: $($move[0])" }
    git mv $move[0] $move[1]
    if ($LASTEXITCODE -ne 0) { throw "Failed frozen move: $($move[0])" }
}
```

The two new report test files and the final fixture move are additional to the 54-file baseline. The boundary contract counts their new tests separately from Appendix A. Make exactly two content/path adjustments after the moves:

1. change the fixture include root from `../../migrations/` to `../../../migrations/`;
2. in moved `report/tests/architecture.rs`, change the prepared `include_str!("../../report_engine.rs")` literal to `include_str!("../../report.rs")` and update only its path-bearing failure text to the crate-relative owner.

The boundary contract rejects the stale app runtime path and asserts the final compile-time include. No other whole-move content rewrite is authorized here.

- [ ] **Step 5: Move every prepared portable half to its final crate path.**

```powershell
$splitMoves = @(
    @('src-tauri/src/analysis/chat_engine.rs', 'src-tauri/crates/extractum-analysis/src/chat.rs'),
    @('src-tauri/src/analysis/domain_portable.rs', 'src-tauri/crates/extractum-analysis/src/domain.rs'),
    @('src-tauri/src/analysis/corpus_portable.rs', 'src-tauri/crates/extractum-analysis/src/corpus.rs'),
    @('src-tauri/src/analysis/corpus/source_resolution_policy.rs', 'src-tauri/crates/extractum-analysis/src/corpus/source_resolution.rs'),
    @('src-tauri/src/analysis/corpus/tests/harness_portable.rs', 'src-tauri/crates/extractum-analysis/src/corpus/tests/harness.rs'),
    @('src-tauri/src/analysis/corpus/tests/live_portable.rs', 'src-tauri/crates/extractum-analysis/src/corpus/tests/live.rs'),
    @('src-tauri/src/analysis/corpus/tests/mod_portable.rs', 'src-tauri/crates/extractum-analysis/src/corpus/tests/mod.rs'),
    @('src-tauri/src/analysis/corpus/tests/preflight_portable.rs', 'src-tauri/crates/extractum-analysis/src/corpus/tests/preflight.rs'),
    @('src-tauri/src/analysis/corpus/tests/source_resolution_portable.rs', 'src-tauri/crates/extractum-analysis/src/corpus/tests/source_resolution.rs'),
    @('src-tauri/src/analysis/groups_store.rs', 'src-tauri/crates/extractum-analysis/src/groups.rs'),
    @('src-tauri/src/analysis/tests_portable.rs', 'src-tauri/crates/extractum-analysis/src/tests.rs'),
    @('src-tauri/src/analysis/report_engine.rs', 'src-tauri/crates/extractum-analysis/src/report.rs'),
    @('src-tauri/src/analysis/report/lifecycle_portable.rs', 'src-tauri/crates/extractum-analysis/src/report/lifecycle.rs'),
    @('src-tauri/src/analysis/report/tests/mod_portable.rs', 'src-tauri/crates/extractum-analysis/src/report/tests/mod.rs'),
    @('src-tauri/src/analysis/store_portable.rs', 'src-tauri/crates/extractum-analysis/src/store.rs'),
    @('src-tauri/src/analysis/store/owned_read_model.rs', 'src-tauri/crates/extractum-analysis/src/store/read_model.rs'),
    @('src-tauri/src/analysis/store/owned_setup.rs', 'src-tauri/crates/extractum-analysis/src/store/setup.rs'),
    @('src-tauri/src/analysis/store/tests/mod_portable.rs', 'src-tauri/crates/extractum-analysis/src/store/tests/mod.rs'),
    @('src-tauri/src/analysis/store/tests/read_model_portable.rs', 'src-tauri/crates/extractum-analysis/src/store/tests/read_model.rs'),
    @('src-tauri/src/analysis/store/tests/setup_portable.rs', 'src-tauri/crates/extractum-analysis/src/store/tests/setup.rs'),
    @('src-tauri/src/analysis/templates_store.rs', 'src-tauri/crates/extractum-analysis/src/templates.rs')
)
foreach ($move in $splitMoves) {
    if (-not (Test-Path -LiteralPath $move[0])) { throw "Missing prepared split source: $($move[0])" }
    git mv $move[0] $move[1]
    if ($LASTEXITCODE -ne 0) { throw "Failed prepared split move: $($move[0])" }
}
```

Do not copy a portable file and leave the staging source behind.

- [ ] **Step 6: Rewire the private app facade mechanically.**

`src-tauri/src/analysis/mod.rs` retains all command re-exports used by `src-tauri/src/lib.rs`, app adapters, fixtures, and app-owned tests. Import curated crate items explicitly; do not expose `extractum_analysis` as a public module and do not change Tauri command signatures or registration. Replace only module/import paths required by the move.

`projects/data_range.rs` is already behaviorally complete from Checkpoints 2–3. Its Task 7 allowance covers only a mechanically necessary typed crate/facade import retarget; the standing contract must still reject the corpus SQL-helper import/re-export, a new port, public helper, changed query predicate, or reordered error path.

- [ ] **Step 7: Update manifests and generate the lockfile through Cargo.**

In `src-tauri/Cargo.toml`:

- append `crates/extractum-analysis` to the exact workspace member list;
- add `extractum-analysis = { path = "crates/extractum-analysis" }` to app dependencies;
- remove the app's direct `zstd.workspace = true` after `rg -n "\bzstd::" src-tauri/src --glob '*.rs'` returns no app call;
- retain workspace `zstd` because `extractum-core` owns compression.

Then run:

```powershell
Invoke-CheckedNative 'new crate manifest check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets }
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 | Out-Null
$metadataExit = $LASTEXITCODE
if ($metadataExit -ne 0) { throw 'Locked workspace metadata failed' }
```

Do not hand-edit registry packages in `Cargo.lock`.

- [ ] **Step 8: Turn every boundary and standing contract GREEN.**

```powershell
Invoke-CheckedNative 'post-move boundary and standing contracts' { npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/development-loop-performance-contract.test.ts src/lib/focused-rust-loop-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts }
```

Expected: exact manifest/lock roots, fixture owner/root, public API, six-table SQL ownership, four workflow families, 95/48 ownership, moved-not-copied files, nonblocking sink, and unchanged wire assertions all pass.

- [ ] **Step 9: Prove exact post-move inventories.**

```powershell
Invoke-CheckedNative 'post-move crate identity list' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --lib -- --list }
Invoke-CheckedNative 'post-move app identity list' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::' -- --list }
```

The boundary contract must find all 95 frozen crate identities under their final prefixes and all 48 retained identities under `analysis::`, each exactly once. New tests are reported separately.

- [ ] **Step 10: Run focused package and immediate-consumer gates.**

```powershell
Invoke-ExactRustTest extractum-analysis 'trace::tests::decode_trace_data_returns_typed_internal_for_invalid_zstd'
Invoke-ExactRustTest extractum-analysis 'corpus::tests::snapshot::list_run_snapshot_messages_page_does_not_fall_back_to_live_source'
Invoke-ExactRustTest extractum-analysis 'report::tests::lifecycle::request_analysis_run_cancel_running_but_inactive_keeps_conflict_message'
Invoke-ExactRustTest extractum-analysis 'store::tests::read_model::list_analysis_run_summaries_applies_query_before_limit'
Invoke-ExactRustTest extractum 'analysis::corpus::tests::live::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref'
Invoke-ExactRustTest extractum 'analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows'
Invoke-ExactRustTest extractum 'analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent'
$newCrateTests = @(
    'chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message',
    'report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails',
    'report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes',
    'store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes',
    'report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities',
    'report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label',
    'report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads',
    'report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure',
    'report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure',
    'report::tests::runtime::report_execution_publishes_typed_events_in_existing_order',
    'chat::tests::chat_execution_persists_turns_before_completed_event',
    'report::tests::runtime::terminal_cleanup_always_removes_active_report_state',
    'test_schema::tests::canonical_fixture_applies_analysis_consumed_schema',
    'test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys'
)
foreach ($testName in $newCrateTests) {
    Invoke-ExactRustTest -Package extractum-analysis -TestName $testName
}
$newAppTests = @(
    'analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels',
    'analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit',
    'analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot',
    'analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id',
    'analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads',
    'analysis::tests_application::report_profile_resolution_failure_prevents_run_creation'
)
foreach ($testName in $newAppTests) {
    Invoke-ExactRustTest -Package extractum -TestName $testName
}
Invoke-CheckedNative 'Checkpoint 7 format' { cargo fmt --manifest-path src-tauri/Cargo.toml --all }
Invoke-CheckedNative 'Checkpoint 7 crate check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets }
Invoke-CheckedNative 'Checkpoint 7 crate tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets }
Invoke-CheckedNative 'Checkpoint 7 app check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'Checkpoint 7 app tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

- [ ] **Step 11: Review mechanical scope and commit.**

```powershell
Invoke-CheckedNative 'Checkpoint 7 diff check' { git diff --check }
git diff --summary
Invoke-CheckedNative 'parsed crate API/runtime ownership scan' { npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t 'keeps commands events spawning migrations and fixtures app-owned' }
Invoke-CheckedNative 'parsed production/test SQL ownership scan' { npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t 'keeps production SQL in the exact six-table owner' }
Assert-NoMatches 'app direct zstd' { rg -n "\bzstd::" src-tauri/src --glob '*.rs' }
git status --short
```

The fail-closed TypeScript contract, not a generic text search, is authoritative for parsed `#[cfg(test)]` regions and the exact three foreign-sentinel setup bodies. Stage only a changed-path set validated against the exact frozen move arrays and named retained files:

```powershell
$retainedAnalysisFiles = @(
    'src-tauri/src/analysis/chat.rs',
    'src-tauri/src/analysis/corpus.rs',
    'src-tauri/src/analysis/corpus/live.rs',
    'src-tauri/src/analysis/corpus/source_resolution.rs',
    'src-tauri/src/analysis/corpus/tests/harness.rs',
    'src-tauri/src/analysis/corpus/tests/live.rs',
    'src-tauri/src/analysis/corpus/tests/mod.rs',
    'src-tauri/src/analysis/corpus/tests/preflight.rs',
    'src-tauri/src/analysis/corpus/tests/source_resolution.rs',
    'src-tauri/src/analysis/events.rs',
    'src-tauri/src/analysis/fixtures.rs',
    'src-tauri/src/analysis/fixtures/seed.rs',
    'src-tauri/src/analysis/fixtures/seed/runs.rs',
    'src-tauri/src/analysis/fixtures/tests/active_runs.rs',
    'src-tauri/src/analysis/fixtures/tests/clear.rs',
    'src-tauri/src/analysis/fixtures/tests/harness.rs',
    'src-tauri/src/analysis/fixtures/tests/mod.rs',
    'src-tauri/src/analysis/fixtures/tests/seed.rs',
    'src-tauri/src/analysis/fixtures/tests/snapshot.rs',
    'src-tauri/src/analysis/fixtures/tests/summary.rs',
    'src-tauri/src/analysis/groups.rs',
    'src-tauri/src/analysis/mod.rs',
    'src-tauri/src/analysis/report.rs',
    'src-tauri/src/analysis/report/lifecycle.rs',
    'src-tauri/src/analysis/report/tests/capture.rs',
    'src-tauri/src/analysis/report/tests/mod.rs',
    'src-tauri/src/analysis/report_commands.rs',
    'src-tauri/src/analysis/store.rs',
    'src-tauri/src/analysis/store/read_model.rs',
    'src-tauri/src/analysis/store/setup.rs',
    'src-tauri/src/analysis/store/tests/mod.rs',
    'src-tauri/src/analysis/store/tests/read_model.rs',
    'src-tauri/src/analysis/store/tests/setup.rs',
    'src-tauri/src/analysis/templates.rs',
    'src-tauri/src/analysis/tests_application.rs'
)
$task7Singletons = @(
    'src-tauri/Cargo.toml',
    'src-tauri/Cargo.lock',
    'src-tauri/crates/extractum-analysis/Cargo.toml',
    'src-tauri/crates/extractum-analysis/src/lib.rs',
    'src-tauri/src/projects/mod.rs',
    'src-tauri/src/projects/read_model.rs',
    'src-tauri/src/projects/data_range.rs',
    'src-tauri/src/account_deletion.rs',
    'src-tauri/src/diagnostics/database.rs',
    'src-tauri/src/notebooklm_export/query.rs',
    'src/lib/analysis-crate-boundary-contract.test.ts',
    'src/lib/analysis-application-contract.test.ts',
    'src/lib/analysis-migration-fixture-contract.test.ts',
    'src/lib/analysis-redesign-safety-contract.test.ts',
    'src/lib/analysis-contract-paths.ts',
    'src/lib/rust-workspace-core-contract.test.ts',
    'src/lib/gemini-browser-crate-boundary-contract.test.ts',
    'src/lib/llm-crate-boundary-contract.test.ts',
    'src/lib/prompt-pack-crate-boundary-contract.test.ts',
    'src/lib/development-loop-performance-contract.test.ts',
    'src/lib/focused-rust-loop-contract.test.ts',
    'src/lib/crate-extraction-shell-cap-contract.test.ts'
)
$movePaths = @(@($wholeMoves + $splitMoves) | ForEach-Object { $_[0]; $_[1] })
$computedTask7Allowed = @($retainedAnalysisFiles + $task7Singletons + $movePaths | Sort-Object -Unique)
$task7Allowed = @(Get-Task7AllowedPaths)
$allowlistDrift = @(Compare-Object -ReferenceObject $task7Allowed -DifferenceObject $computedTask7Allowed)
if ($allowlistDrift.Count -ne 0) { throw "Task 7 move arrays disagree with the committed plan allowlist: $($allowlistDrift -join ', ')" }
Add-ScopedChanges -Label 'Checkpoint 7 mechanical extraction' -Allowed $task7Allowed
Invoke-CheckedNative 'commit mechanical extraction' { git commit -m "refactor: extract analysis domain crate" }
$extractionCommit = (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'Checkpoint 7 extraction commit'
```

---

### Task 8: Run Completion Gates and Capture the Single Advisory Timing

**Files:**

- Create temporarily outside the repository: `%TEMP%\extractum-phase7-analysis-workspace-check.txt`
- Modify only if a mechanical wiring defect is found: files in the Task 7 extraction commit

- [ ] **Step 1: Require a clean committed candidate.**

```powershell
Assert-CleanWorktree 'completion gate start'
git rev-parse HEAD
Invoke-CheckedNative 'completion boundary contracts' { npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts }
```

- [ ] **Step 2: Reconfirm exact package ownership and package gates.**

```powershell
Invoke-CheckedNative 'completion crate identity list' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --lib -- --list }
Invoke-CheckedNative 'completion app identity list' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::' -- --list }
Invoke-CheckedNative 'completion crate check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets }
Invoke-CheckedNative 'completion crate tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets }
Invoke-CheckedNative 'completion app check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'completion app tests' { cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
```

Expected: 95 frozen crate identities, 48 frozen app identities, all new tests reported separately, and all four package commands green.

- [ ] **Step 3: Run rustfmt, then time one ordinary mandatory workspace check.**

```powershell
Invoke-CheckedNative 'completion rustfmt gate' { npm.cmd run check:rustfmt }

$workspaceWatch = [System.Diagnostics.Stopwatch]::StartNew()
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
$workspaceExit = $LASTEXITCODE
$workspaceWatch.Stop()
if ($workspaceExit -ne 0) { throw 'workspace check failed' }
$workspaceMilliseconds = $workspaceWatch.ElapsedMilliseconds
"phase=7`ncommand=cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`nmilliseconds=$workspaceMilliseconds" |
    Set-Content -LiteralPath "$env:TEMP\extractum-phase7-analysis-workspace-check.txt"
"Phase 7 ordinary workspace check: $workspaceMilliseconds ms"
```

This is the only Phase 7 timing observation. Do not run a focused probe, warm-up, sample series, source mutation, quiet-window scan, process coordinator, retry, A/B harness, alternate target, or ledger. If correctness later requires rerunning workspace check, label it a gate rerun in verification; never treat it as a second sample or select the faster result.

Compare the observation with Phase 6's recorded `11,669 ms` only for the coarse adjacent rule. Since Phase 6 is below `15,000 ms`, Phase 7 alone cannot trigger the two-adjacent-slices investigation rule. Record the Phase 7 number regardless; it cannot veto or revert the extraction.

- [ ] **Step 4: Run the remaining full completion gates.**

```powershell
Invoke-CheckedNative 'completion workspace tests' { cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets }
Invoke-CheckedNative 'completion repository verify' { npm.cmd run verify }
```

Expected: both pass. If a failure is a mechanical import/path/visibility defect already authorized by the frozen boundary, fix only that defect, rerun the affected exact/package/full gates, and commit `fix: complete analysis crate wiring`. Before that commit, enumerate changed/untracked paths exactly as in Task 7, reject any path outside `$task7Allowed`, stage only the validated `$wiringChanged` array, and append the resulting SHA to `$wiringFixCommits`. A new seam, behavior change, dependency, public field, or SQL exception is not a wiring fix; stop and amend preparation/design.

```powershell
# Run only after an authorized mechanical wiring fix and its rerun gates are green.
if ($null -eq $wiringFixCommits) { $wiringFixCommits = @() }
$task7Allowed = @(Get-Task7AllowedPaths)
Add-ScopedChanges -Label 'authorized wiring fix' -Allowed $task7Allowed
Invoke-CheckedNative 'commit wiring fix' { git commit -m 'fix: complete analysis crate wiring' }
$wiringFixCommits += (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'wiring fix commit'
```

- [ ] **Step 5: Confirm no completion residue.**

```powershell
Assert-CleanWorktree 'pre-release candidate'
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 | Out-Null
$finalMetadataExit = $LASTEXITCODE
if ($finalMetadataExit -ne 0) { throw 'Final locked metadata failed' }
```

The worktree must be clean before release evidence.

---

### Task 9: Prove Release Startup and Record the Retained Result

**Files:**

- Create: `docs/superpowers/verification/2026-07-22-extractum-analysis-extraction.md`
- Modify: `docs/superpowers/specs/2026-07-22-analysis-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `docs/project.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`

- [ ] **Step 1: Build the release executable without a bundle.**

```powershell
npm.cmd run tauri -- build --no-bundle
if ($LASTEXITCODE -ne 0) { throw '[completion] release build failed' }
$releaseExe = (Resolve-Path -LiteralPath 'src-tauri/target/release/extractum.exe').Path
```

Do not build an installer merely for this slice.

- [ ] **Step 2: Run a bounded exact-PID startup smoke.**

```powershell
$existing = @(Get-Process -Name extractum -ErrorAction SilentlyContinue)
if ($existing.Count -ne 0) { throw '[infrastructure] pre-existing extractum process prevents exact ownership proof' }

$ownedProcess = $null
$startupFailure = $null
try {
    try {
        $ownedProcess = Start-Process -FilePath $releaseExe -PassThru -WindowStyle Hidden
    } catch {
        throw "[infrastructure] release launch failed: $($_.Exception.Message)"
    }
    if ($null -eq $ownedProcess) { throw '[infrastructure] launch returned no process' }

    Start-Sleep -Seconds 5
    try {
        $ownedProcess.Refresh()
    } catch {
        throw "[infrastructure] exact PID refresh failed: $($_.Exception.Message)"
    }
    if ($ownedProcess.HasExited) {
        throw "[completion] release process exited during the 5-second startup window with code $($ownedProcess.ExitCode)"
    }
    try {
        $ownedPath = (Get-Process -Id $ownedProcess.Id -ErrorAction Stop).Path
    } catch {
        throw "[infrastructure] exact PID path inspection failed: $($_.Exception.Message)"
    }
    if ([System.IO.Path]::GetFullPath($ownedPath) -ne [System.IO.Path]::GetFullPath($releaseExe)) {
        throw '[infrastructure] exact PID path does not match the built executable'
    }
    "startup_pid=$($ownedProcess.Id) path=$ownedPath observed_seconds=5"
} catch {
    $startupFailure = $_
} finally {
    if ($null -ne $ownedProcess) {
        try {
            $ownedProcess.Refresh()
            if (-not $ownedProcess.HasExited) { Stop-Process -Id $ownedProcess.Id -Force -ErrorAction Stop }
            $deadline = [DateTime]::UtcNow.AddSeconds(10)
            while ((Get-Process -Id $ownedProcess.Id -ErrorAction SilentlyContinue) -and [DateTime]::UtcNow -lt $deadline) {
                Start-Sleep -Milliseconds 100
            }
            if (Get-Process -Id $ownedProcess.Id -ErrorAction SilentlyContinue) {
                throw 'owned PID did not exit within cleanup bound'
            }
        } catch {
            if ($null -eq $startupFailure) {
                $startupFailure = [System.Management.Automation.RuntimeException]::new("[infrastructure] exact PID cleanup failed: $($_.Exception.Message)")
            }
        }
    }

    $residue = @(Get-Process -Name extractum -ErrorAction SilentlyContinue | Where-Object {
        try { [System.IO.Path]::GetFullPath($_.Path) -eq [System.IO.Path]::GetFullPath($releaseExe) } catch { $false }
    })
    if ($residue.Count -ne 0 -and $null -eq $startupFailure) {
        $startupFailure = [System.Management.Automation.RuntimeException]::new('[infrastructure] matching release process residue remains')
    }
}
if ($null -ne $startupFailure) { throw $startupFailure }
```

Early application exit is a completion failure. Launch, inspection, ownership, and cleanup failures are infrastructure failures and do not by themselves prove a bad crate boundary.

- [ ] **Step 3: Keep optional live smoke ordered and non-mutating.**

The release startup proof above is required. Do not issue live credentialed LLM/provider requests or mutate accounts. If the owner also requests both live MCP and self-managed `npm.cmd run smoke:analysis`, run live MCP first, then prove all app ports are free before the self-managed smoke. Do not introduce a new process-control harness.

- [ ] **Step 4: Write the durable verification document.**

Capture the final app/crate inventory with physical-line counting that includes blank lines:

```powershell
$finalAppFiles = Get-ChildItem -LiteralPath src-tauri/src/analysis -Recurse -File -Filter '*.rs'
$finalAppLines = ($finalAppFiles | ForEach-Object { @(Get-Content -LiteralPath $_.FullName).Count } | Measure-Object -Sum).Sum
$finalCrateFiles = Get-ChildItem -LiteralPath src-tauri/crates/extractum-analysis/src -Recurse -File -Filter '*.rs'
$finalCrateLines = ($finalCrateFiles | ForEach-Object { @(Get-Content -LiteralPath $_.FullName).Count } | Measure-Object -Sum).Sum
"app_files=$($finalAppFiles.Count) app_lines=$finalAppLines crate_files=$($finalCrateFiles.Count) crate_lines=$finalCrateLines"
if ($finalAppFiles.Count -ne 35) { throw "Unexpected final app Rust file count: $($finalAppFiles.Count)" }
if ($finalCrateFiles.Count -ne 45) { throw "Unexpected final crate Rust file count: $($finalCrateFiles.Count)" }
```

Record all of the following, with raw commands and outcomes:

- actual starting commit, Checkpoint 1–5 commits, RED contract commit, extraction commit, and any authorized wiring-fix commit;
- final 54-file disposition including both `mod.rs` crate outputs, the four separately named plan-added paths, generated crate root `lib.rs`, exact `35` app / `45` crate Rust-file topology, and actual physical-line counts;
- exact frozen 95/48 inventory plus separately counted new tests;
- 21 analysis, three project, and three dev command inventory;
- public root and visibility allowlists;
- exact six-table SQL allowlist/foreign denylist, all eight participant signatures, and all nine coordinator call chains under four families;
- proof that excluded Family 1 paths remain pool calls;
- two-read A/B, report/chat profile timing, event adapter, cancellation, cleanup, and wire evidence;
- manifest, lockfile, reverse-edge, direct-zstd, moved-not-copied, and fixture parity evidence;
- focused/package/workspace/repository gate results;
- the one ordinary workspace-check duration from `%TEMP%\extractum-phase7-analysis-workspace-check.txt`, explicitly advisory;
- release build path, exact startup PID/path, five-second observation, cleanup, and residue result;
- retained/non-retained decision and explicit statement that Phase 8 remains unauthorized.

- [ ] **Step 5: Update authorities to the retained state.**

Only after all evidence passes:

- set the Phase 7 design status to implemented and retained and link the verification document;
- set the roadmap heading to exact `done: retained`, record the advisory duration and final measured app file/line count, and keep Phase 8 unapproved;
- change the shell-cap contract's exact expected Phase 7 state to `done: retained` without weakening the status vocabulary or timing assertions;
- update `docs/project.md` architecture and reading order for `extractum-analysis`.

Do not edit `docs/value-registry.md`.

- [ ] **Step 6: Run documentation/source contracts and commit the result.**

```powershell
Invoke-CheckedNative 'retained documentation contracts' { npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/development-loop-performance-contract.test.ts src/lib/focused-rust-loop-contract.test.ts }
Invoke-CheckedNative 'retained documentation diff check' { git diff --check }
git status --short
$task9Files = @(
    'docs/superpowers/verification/2026-07-22-extractum-analysis-extraction.md',
    'docs/superpowers/specs/2026-07-22-analysis-crate-boundary-design.md',
    'docs/superpowers/specs/2026-07-17-crate-roadmap.md',
    'docs/project.md',
    'src/lib/crate-extraction-shell-cap-contract.test.ts'
)
Add-ScopedChanges -Label 'retained evidence' -Allowed $task9Files
Invoke-CheckedNative 'commit retained evidence' { git commit -m "docs: record analysis crate extraction" }
$verificationCommit = (git rev-parse HEAD).Trim()
Assert-CleanWorktree 'retained evidence commit'
```

- [ ] **Step 7: Final clean-state proof.**

```powershell
Assert-CleanWorktree 'final Phase 7 state'
git log -10 --oneline
```

Expected: clean worktree and a reviewable sequence of five green preparation commits, one RED boundary contract commit, one mechanical extraction commit, optional authorized wiring fix, and one verification/documentation commit.

## Pause and Rollback Ladder

1. Checkpoints 1–5 are separately green and useful. A legitimate pause retains them and records the exact last checkpoint in the roadmap.
2. Checkpoint 6 is intentionally RED and must remain a separate commit. If pausing before extraction, revert it with ordinary `git revert`.
3. Checkpoint 7 is separately mechanical. If extraction or completion gates fail, preserve the failed candidate in Git history, revert any later wiring fix newest-first, revert the extraction commit, then revert the RED contract commit.
4. Decide whether to retain or revert each green preparation commit independently, newest-first. Do not discard all preparation automatically.
5. Use ordinary `git revert`; never use `git reset`, destructive checkout, forced branch deletion, or manual evidence deletion.
6. If the candidate is not retained, create a durable verification disposition and set the roadmap to truthful `not retained` or the last retained preparation checkpoint.
7. Timing failure or slowdown never triggers rollback. Infrastructure failure is recorded separately from an application completion failure.
8. Treat `E0433`, an absolute app-root import, or a duplicate-module error originating in a moved/staging source as evidence that its owning green preparation checkpoint was incomplete. Revert the extraction/RED chain and amend that checkpoint; do not relabel it as an authorized Task 8 wiring fix. Only a defect confined to the Task 7-created crate root, app facade, destination path, manifest, or lock wiring may use the narrow wiring-fix path.

Use this executable sequence for a failed extraction candidate. It never discards a dirty diff: it first validates the exact candidate path set, commits it for durable evidence, then reverts that preservation commit before the already-committed wiring/extraction/RED chain. Rehydrate exact SHAs by unique commit subject if execution variables were lost between sessions, verify ancestry, and stop on any ambiguity or conflict.

```powershell
$redMatches = @(git log --format='%H' --grep='^test: define analysis crate boundary$')
if ($LASTEXITCODE -ne 0 -or $redMatches.Count -ne 1) { throw 'Cannot identify one RED boundary commit' }
$redContractCommit = $redMatches[0].Trim()
$task7Allowed = @(Get-Task7AllowedPaths)
$extractionMatches = @(git log --format='%H' --grep='^refactor: extract analysis domain crate$')
if ($LASTEXITCODE -ne 0 -or $extractionMatches.Count -gt 1) { throw 'Expected zero or one extraction commit' }
$extractionCommit = $null
$wiringFixCommits = @()
if ($extractionMatches.Count -eq 1) {
    $extractionCommit = $extractionMatches[0].Trim()
    Invoke-CheckedNative 'verify RED/extraction ancestry' { git merge-base --is-ancestor $redContractCommit $extractionCommit }
    $wiringFixCommits = @(git log --format='%H' --grep='^fix: complete analysis crate wiring$' "$extractionCommit..HEAD")
    if ($LASTEXITCODE -ne 0) { throw 'Cannot enumerate wiring fix commits' }
}

$preservationCommit = $null
$rollbackChanged = @(Get-ChangedPaths)
if ($rollbackChanged.Count -ne 0) {
    Add-ScopedChanges -Label 'failed candidate evidence' -Allowed $task7Allowed
    Invoke-CheckedNative 'preserve failed candidate evidence' { git commit -m 'chore: preserve failed analysis extraction candidate' }
    $preservationCommit = (git rev-parse HEAD).Trim()
    Assert-CleanWorktree 'failed candidate preservation commit'
}

$revertNewestFirst = @()
if ($null -ne $preservationCommit) { $revertNewestFirst += $preservationCommit }
$revertNewestFirst += $wiringFixCommits
if ($null -ne $extractionCommit) { $revertNewestFirst += $extractionCommit }
$revertNewestFirst += $redContractCommit
foreach ($commit in $revertNewestFirst) {
    Invoke-CheckedNative "revert candidate commit $commit" { git revert --no-edit $commit }
}
Invoke-CheckedNative 'post-revert prepared app check' { cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets }
Invoke-CheckedNative 'post-revert standing contracts' { npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts }
Assert-CleanWorktree 'post-revert prepared state'
```

If a revert conflicts, stop with the conflict intact and investigate; do not use `reset`, checkout-based discard, or force. After a clean successful rollback, add the durable non-retained verification/roadmap disposition as its own exact-path documentation commit.

## Final Manual Review

- [ ] Every authority and status line agrees; Phase 8 remains unauthorized.
- [ ] Frozen counts reconcile: 54 current files / 13,187 LOC; 95 crate + 48 app = 143; 21 analysis + 3 project + 3 dev = 27; six owned tables; four transaction families/eight participants/nine coordinators.
- [ ] Final file inventory is 35 app Rust files and 45 crate Rust files: baseline `mod.rs` contributes crate `domain.rs` and `tests.rs` exactly once; plan-added paths are app `tests_application.rs` and crate `report/tests/corpus_port.rs`, `report/tests/runtime.rs`, and `test_schema.rs`; generated crate root `lib.rs` is counted separately.
- [ ] Every current file has one final disposition and every split used its prepared staging file.
- [ ] Every Appendix A identity exists exactly once under its final Cargo owner; new tests are not substituted for frozen identities.
- [ ] Family 1 includes only list, active, get, and legacy blank/null chat fallback; trace, delete, chat list/clear, and lifecycle reads remain pool calls.
- [ ] All eight participants borrow `&mut SqliteConnection` first, never take a pool, and never own transaction lifecycle; app coordinators thread one explicit transaction.
- [ ] `ResolvedAnalysisScope`, corpus/event ports, report/chat tickets, state APIs, and constructors match their frozen signatures.
- [ ] The crate root matches the exhaustive allowlists and has no public modules, globs, internal rows, secret getter, or test helper.
- [ ] Manifest, lockfile, direct roots/features, one app edge, no reverse edge, and removal of app direct zstd match exactly.
- [ ] Production crate SQL uses only six tables; foreign SQL and migrations remain app-owned.
- [ ] The migration fixture has one owner, the correct relative root, and ordered parity with both the app registry and independent prompt-pack contract.
- [ ] Two corpus reads, profile timing, task spawning, event ordering/messages, persistence ordering, cancellation, cleanup, request IDs, and `AppError` JSON remain unchanged.
- [ ] Package, workspace, repository, release, and bounded startup gates have fresh passing evidence.
- [ ] Exactly one advisory workspace-check duration is recorded; no timing machinery or retention threshold was introduced.
- [ ] `git diff --check` passes and the final worktree is clean.
