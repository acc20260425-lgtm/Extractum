# Telegram Migrated Dialog History Proof Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-run the controlled migrated small-group-to-supergroup runtime validation after fresh post-migration messages and close the slice as `passed` only if persisted item/history rows prove the current channel peer is used.

**Architecture:** This is a validation/documentation slice, not an application-code change. Runtime evidence is captured in ignored `reference/*` JSON/log artifacts, while only sanitized conclusions are written to tracked docs.

**Tech Stack:** Tauri dev runtime, Tauri MCP bridge on port `9223`, app IPC commands, SQLite read-only snapshots, PowerShell, Vitest for final regression verification.

---

## Privacy Boundary

Do not write private Telegram titles, usernames, message text, phone numbers, session data, API data, API hashes, access-hash values, or auth material to tracked files.

Tracked docs may contain account id `11`, source id `115`, source subtype, peer kind, peer id, boolean presence fields, sync counts, warning codes, Takeout history-scope values, and sanitized pass/fail conclusions.

Ignored runtime files under `reference/*` may contain local operator context when needed for fixture selection.

## Files

- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md`
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- Modify: `docs/backlog.md`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-context.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-before.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-sync-result.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-after.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-evaluation.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-takeout-result.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-takeout-evaluation.json`
- Runtime only: `reference/tauri-dev-history-proof.stdout.log`
- Runtime only: `reference/tauri-dev-history-proof.stderr.log`
- Runtime only: `reference/tauri-dev-history-proof.pid`

### Task 1: Branch Baseline And Plan Checkpoint

**Files:**
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md`

- [x] **Step 1: Confirm branch and clean tracked state**

Run: `git status --short --branch`

Expected: branch `telegram-migrated-dialog-history-proof` with no tracked changes except this plan before the checkpoint commit.

- [x] **Step 2: Verify the existing validation baseline**

Run:

```powershell
rg -n "Migrated small group -> supergroup|needs follow-up|source `115`|history-peer proof" docs\superpowers\verification\telegram-runtime-private-source-validation.md docs\backlog.md
```

Expected: tracked docs still record the previous result as `needs follow-up` because no persisted item/history rows existed.

- [x] **Step 3: Mark Task 1 complete**

Update this task's checkboxes to `[x]`.

- [x] **Step 4: Commit the plan checkpoint**

Run:

```powershell
git add docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md
git commit -m "docs: plan telegram migrated-dialog history proof"
```

Expected: commit succeeds on branch `telegram-migrated-dialog-history-proof`.

### Task 2: Start Runtime And Confirm Account

**Files:**
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md`
- Runtime only: `reference/tauri-dev-history-proof.stdout.log`
- Runtime only: `reference/tauri-dev-history-proof.stderr.log`
- Runtime only: `reference/tauri-dev-history-proof.pid`

- [x] **Step 1: Confirm no stale runtime processes**

Run:

```powershell
Get-Process | Where-Object { $_.ProcessName -in @('extractum','cargo') } | Select-Object ProcessName, Id, Path
```

Expected: no rows. If rows exist, stop them before starting the new runtime.

- [x] **Step 2: Start the Tauri dev app**

Run:

```powershell
$out = Join-Path (Get-Location) 'reference\tauri-dev-history-proof.stdout.log'
$err = Join-Path (Get-Location) 'reference\tauri-dev-history-proof.stderr.log'
$pidFile = Join-Path (Get-Location) 'reference\tauri-dev-history-proof.pid'
$p = Start-Process -FilePath npm.cmd -ArgumentList @('run','tauri','dev') -WorkingDirectory (Get-Location) -RedirectStandardOutput $out -RedirectStandardError $err -PassThru -WindowStyle Hidden
$p.Id | Set-Content -Encoding ascii $pidFile
$p.Id
```

Expected: a process id is printed and the bridge logs eventually show `WebSocket server listening on: 0.0.0.0:9223`.

- [x] **Step 3: Connect the Tauri MCP bridge**

Run the Tauri MCP driver session start on port `9223`.

Expected: MCP session connects to app `org.ai.extractum`.

- [x] **Step 4: Confirm account 11 is ready**

Run app IPC command `tg_get_account_statuses` with `accountIds = [11]`.

Expected sanitized result includes `{"account_id":11,"status":"ready"}`.

- [x] **Step 5: Mark Task 2 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md
git commit -m "docs: checkpoint telegram migrated-dialog history runtime"
```

### Task 3: Sync Source 115 And Capture Persistent Rows

**Files:**
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-context.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-before.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-sync-result.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-after.json`

- [x] **Step 1: Resolve source 115 from the app database**

Capture sanitized context for source `115`: account id, source subtype, external id, peer kind, peer id, access-hash presence, username presence, and resolution strategy.

Expected: source `115` belongs to account `11`, has `source_subtype = supergroup`, `peer_kind = channel`, and current channel `peer_id`.

- [x] **Step 2: Capture the before snapshot**

Read the app database in read-only mode and save counts/groups for:

- `items` rows for source `115`
- `telegram_messages` rows for source `115`
- grouped `history_peer_kind`, `history_peer_id`, and row counts
- `last_sync_state`

Expected: snapshot saved to `reference/telegram-migrated-dialog-history-proof-before.json`.

- [x] **Step 3: Run normal source sync**

Run app IPC command `sync_source` with `source_id = 115`.

Expected: command returns `ok: true`. A successful empty sync is allowed only if the later evaluation remains `needs_follow_up`.

- [x] **Step 4: Save the sync result**

Save sanitized sync result to `reference/telegram-migrated-dialog-history-proof-sync-result.json`.

Expected: file contains sync counts and warning codes without private message text.

- [x] **Step 5: Capture the after snapshot**

Repeat the read-only database snapshot from Step 2 after sync completes.

Expected: snapshot saved to `reference/telegram-migrated-dialog-history-proof-after.json`.

- [x] **Step 6: Mark Task 3 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md
git commit -m "docs: checkpoint telegram migrated-dialog history sync"
```

### Task 4: Evaluate Primary History-Peer Proof

**Files:**
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-evaluation.json`

- [x] **Step 1: Evaluate persisted proof**

Create `reference/telegram-migrated-dialog-history-proof-evaluation.json` with:

- `classification = "passed"` only when sync succeeds, stored identity remains `supergroup`/`channel`, `items.source_id = 115` has at least one row, `telegram_messages.source_id = 115` has at least one row, every history group uses `history_peer_kind = channel`, every history group uses the current source peer id, and no `chat` history group appears.
- `classification = "needs_follow_up"` when sync succeeds but persisted history rows are still absent.
- `classification = "failed"` when wrong-peer rows, stale chat rows, source mutation, or sync failure are observed.

- [x] **Step 2: Stop on non-pass**

If classification is not `passed`, skip Takeout, stop runtime in Task 6, and document the non-sensitive reason.

Expected: no Takeout smoke runs unless primary classification is `passed`.

- [x] **Step 3: Mark Task 4 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md
git commit -m "docs: checkpoint telegram migrated-dialog history evaluation"
```

### Task 5: Narrow Takeout Smoke After Primary Pass

**Files:**
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-takeout-result.json`
- Runtime only: `reference/telegram-migrated-dialog-history-proof-takeout-evaluation.json`

- [x] **Step 1: Check the primary classification**

Read `reference/telegram-migrated-dialog-history-proof-evaluation.json`.

Expected: continue only when `classification = "passed"`.

- [x] **Step 2: Run the narrow Takeout smoke**

Run the existing app Takeout import flow for the controlled migrated fixture only.

Expected: Takeout either records migrated-history deferment safely or reports a controlled blocked status before unsafe writes.

- [x] **Step 3: Evaluate Takeout result**

Save `reference/telegram-migrated-dialog-history-proof-takeout-evaluation.json`.

Expected clean result: no unsafe old `chat` history rows are imported as if they were current supergroup history; migrated history is deferred with sanitized warning/provenance when detected.

Task 5 checkpoint: primary classification was `passed`; narrow Takeout smoke started job `takeout-1` but Telegram returned `TAKEOUT_INIT_DELAY` before unsafe writes. Evaluation classified this as `blocked_start`, with no new history groups and no `chat` history group after the smoke.

- [x] **Step 4: Mark Task 5 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md
git commit -m "docs: checkpoint telegram migrated-dialog takeout smoke"
```

### Task 6: Stop Runtime

**Files:**
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md`

- [x] **Step 1: Stop the Tauri MCP session**

Run Tauri MCP driver session stop.

Expected: all Tauri MCP sessions are stopped.

- [x] **Step 2: Stop the Tauri dev process tree**

Stop the process id recorded in `reference/tauri-dev-history-proof.pid` and any child `cargo` or `extractum` processes started by this run.

Expected: no `extractum` or `cargo` processes remain for this workspace.

Task 6 checkpoint: MCP sessions were stopped. Non-escalated process-tree lookup hit Windows `Get-CimInstance` access denied, so cleanup was rerun with permission and the remaining `cargo`/`extractum` processes from the Tauri dev run were stopped.

- [x] **Step 3: Mark Task 6 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md
git commit -m "docs: checkpoint telegram migrated-dialog history cleanup"
```

### Task 7: Document Result And Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md`
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Update the verification row**

If primary classification is `passed`, update the migrated small-group-to-supergroup row to `passed` and include sanitized proof: source id, source subtype, peer kind, peer id, item count, telegram message count, and absence of wrong-peer groups.

If primary classification is not `passed`, keep the row as needs follow-up and document the sanitized blocker.

- [ ] **Step 2: Update backlog**

If primary classification is `passed`, close or remove the backlog follow-up that requested a fixture with persisted rows. If Takeout smoke is blocked or failed, leave a narrow Takeout follow-up.

If primary classification is not `passed`, keep a concrete runtime follow-up.

- [ ] **Step 3: Run formatting and test verification**

Run:

```powershell
git diff --check
npm.cmd test
git status --short --branch
```

Expected: no whitespace errors, `55` Vitest files pass, and tracked changes are limited to this plan plus sanitized docs.

- [ ] **Step 4: Mark Task 7 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-history-proof.md docs/superpowers/verification/telegram-runtime-private-source-validation.md docs/backlog.md
git commit -m "docs: validate telegram migrated-dialog history proof"
```
