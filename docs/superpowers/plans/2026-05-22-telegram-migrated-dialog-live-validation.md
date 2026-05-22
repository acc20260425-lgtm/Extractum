# Telegram Migrated Dialog Live Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate account `11` migrated small-group-to-supergroup dialog behavior through Add Source and normal sync, with a secondary Takeout detect-and-defer smoke when runtime conditions allow it.

**Architecture:** Run a docs-only live validation slice against one controlled migrated Telegram fixture. Use the normal Tauri IPC path to list dialogs, add or reuse the source, sync it, snapshot SQLite before/after, optionally start a bounded Takeout smoke, evaluate sanitized invariants, and update verification/backlog docs.

**Tech Stack:** PowerShell, Python `sqlite3`, SQLite, Tauri dev app and MCP bridge, Markdown verification docs.

---

## File Structure

- Read: `docs/superpowers/specs/2026-05-22-telegram-migrated-dialog-live-validation-design.md`
  - Approved design, guardrails, pass criteria, and outcome classification.
- Read: `src-tauri/src/sources/store.rs`
  - Verify Add Source stores canonical typed identity and source subtype.
- Read: `src-tauri/src/sources/sync.rs`
  - Verify normal sync resolves a typed Telegram peer and records current-history identity.
- Read: `src-tauri/src/takeout_import/mod.rs`
  - Verify migrated supergroup detection calls `mark_takeout_migrated_history_deferred`.
- Read: `src-tauri/src/ingest_provenance.rs`
  - Verify Takeout provenance fields, warning code, and completeness classification.
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`
  - Mark completed task checkboxes during execution.
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
  - Mark the `Migrated small group -> supergroup` matrix row according to live evidence and add a dated note.
- Modify: `docs/backlog.md`
  - Remove the runtime migrated-dialog row only if primary runtime acceptance passes. Keep or add Takeout follow-up evidence in section `3.3` if the secondary smoke is blocked or fails.
- Create runtime-only ignored files under `reference/`
  - `telegram-migrated-dialog-validation-context.json`
  - `telegram-migrated-dialog-candidates.json`
  - `telegram-migrated-dialog-selected-index.txt`
  - `telegram-migrated-dialog-add-result.json`
  - `telegram-migrated-dialog-sync-results.json`
  - `telegram-migrated-dialog-capture-snapshot.py`
  - `telegram-migrated-dialog-validation-snapshots.json`
  - `telegram-migrated-dialog-primary-evaluation.json`
  - `telegram-migrated-dialog-takeout-result.json`
  - `telegram-migrated-dialog-takeout-evaluation.json`
  - `tauri-dev-migrated-dialog.stdout.log`
  - `tauri-dev-migrated-dialog.stderr.log`
  - `tauri-dev-migrated-dialog.pid`

Do not modify Rust, Svelte, schema, or app code during this slice. If production code changes become necessary, stop and switch to a test-first bugfix plan.

## Privacy Rules

- Do not write private Telegram titles, usernames, message text, phone numbers, session data, API data, API hashes, access-hash values, or auth material to tracked docs.
- Runtime-only `reference/*` files may contain local operator context, including private local titles when needed for fixture selection.
- Tracked docs may record account id `11`, source ids, `source_subtype`, `peer_kind`, `peer_id`, boolean username/access-hash presence, sync counts, warning codes/kinds, `history_scope`, and sanitized wrong-peer conclusions.
- `external_id`, raw `peer_id`, raw message id ranges, and candidate titles stay in ignored local evidence unless the verification note explicitly treats numeric ids as sanitized evidence.

## Execution Rules

- Start execution from a clean worktree on `main`.
- Create and use branch `telegram-migrated-dialog-live-validation` during live execution.
- After each task, mark completed checkboxes in this plan file and commit a checkpoint. Commit only tracked docs/plan changes; ignored `reference/*` evidence remains local.
- Avoid blocking `Read-Host` prompts. When a task reaches a human gate, ask the operator in chat, then run the next command with the provided value embedded non-interactively.
- If a task classifies the primary runtime flow as `blocked`, `failed`, or `needs_follow_up`, stop runtime processes before documenting the result.

### Task 1: Static Pre-Flight Audit

**Files:**
- Read: `docs/superpowers/specs/2026-05-22-telegram-migrated-dialog-live-validation-design.md`
- Read: `src-tauri/src/sources/store.rs`
- Read: `src-tauri/src/sources/sync.rs`
- Read: `src-tauri/src/takeout_import/mod.rs`
- Read: `src-tauri/src/ingest_provenance.rs`
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`

- [x] **Step 1: Confirm tracked workspace state**

Run:

```powershell
git status --short --branch
git log -1 --oneline
```

Expected: clean `## main` before branch creation. Record the commit line for the dated verification note.

- [x] **Step 2: Create or switch to the validation branch**

Run:

```powershell
$branch = 'telegram-migrated-dialog-live-validation'
$current = git branch --show-current
if ($current -ne $branch) {
  git show-ref --verify --quiet "refs/heads/$branch"
  if ($LASTEXITCODE -eq 0) {
    git switch $branch
  } else {
    git switch -c $branch
  }
}
git status --short --branch
```

Expected: clean `## telegram-migrated-dialog-live-validation`.

- [x] **Step 3: Verify Add Source typed identity boundary**

Run:

```powershell
rg -n "pub async fn add_telegram_source|upsert_telegram_source_row|ON CONFLICT\\(account_id, source_type, source_subtype, external_id\\)|upsert_telegram_source_identity_from_resolved|source_subtype = excluded.source_subtype|peer_kind|resolution_strategy" src-tauri\src\sources\store.rs
```

Expected evidence:

```text
add_telegram_source resolves the dialog/source ref through the active account client.
sources uniqueness is account_id + source_type + source_subtype + external_id for Telegram.
telegram_sources is updated from the resolved typed identity.
supergroup rows should store source_subtype = supergroup and peer_kind = channel.
```

Abort as `blocked` if Add Source cannot be audited to store typed source identity.

- [x] **Step 4: Verify normal sync history-peer identity boundary**

Run:

```powershell
rg -n "fallback_peer_identity|history_peer_kind|history_peer_id|persist_items|sync_telegram_source|ResolvedSyncPeer" src-tauri\src\sources\sync.rs src-tauri\src\sources\items.rs
```

Expected evidence:

```text
normal sync builds TelegramMessageIdentity from the resolved current history peer.
current supergroup sync should persist history_peer_kind = channel and current peer_id.
telegram_messages native identity is source_id + history_peer_kind + history_peer_id + telegram_message_id.
```

- [x] **Step 5: Verify Takeout migrated-history deferment contract**

Run:

```powershell
rg -n "detect_supergroup_migration|migrated_from_chat_id|mark_takeout_migrated_history_deferred|migrated_history_deferred|history_scope|classify_completeness|mixed_partial|current_history_with_migrated_deferred" src-tauri\src\takeout_import\mod.rs src-tauri\src\ingest_provenance.rs src-tauri\migrations\0001_current_schema_baseline.sql
```

Expected evidence:

```text
Takeout detects migrated_from_chat_id for supergroups.
Detection calls mark_takeout_migrated_history_deferred.
Provenance records migrated_history_detected = 1 and migrated_history_imported = 0.
Warning code migrated_history_deferred is inserted into ingest_batch_warnings.code.
Terminal completed jobs with deferment classify ingest_batches.completeness as partial.
```

- [x] **Step 6: Commit the Task 1 checkpoint**

Update this plan file by marking Task 1 checkboxes complete, then run:

```powershell
git add docs\superpowers\plans\2026-05-22-telegram-migrated-dialog-live-validation.md
git commit -m "docs: checkpoint telegram migrated-dialog preflight"
```

Expected: checkpoint commit succeeds on `telegram-migrated-dialog-live-validation`.

### Task 2: Start Runtime And Confirm Account 11

**Files:**
- Create: `reference/tauri-dev-migrated-dialog.stdout.log`
- Create: `reference/tauri-dev-migrated-dialog.stderr.log`
- Create: `reference/tauri-dev-migrated-dialog.pid`
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`

- [x] **Step 1: Stop stale MCP sessions**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "stop" })
```

Expected: all previous bridge sessions are stopped or there were no active sessions.

- [x] **Step 2: Confirm the app is not already running**

Run:

```powershell
Get-Process | Where-Object {
  $_.ProcessName -like '*extractum*' -or
  $_.ProcessName -like '*tauri*' -or
  $_.ProcessName -eq 'cargo'
} | Select-Object Id, ProcessName, Path
```

Expected: no Extractum/Tauri/Cargo process is already running for this workspace. If a stale process exists, stop it before continuing.

- [x] **Step 3: Start the Tauri dev app**

Run with escalated permission if sandboxing blocks GUI/runtime startup:

```powershell
$stdout = Join-Path (Get-Location) 'reference\tauri-dev-migrated-dialog.stdout.log'
$stderr = Join-Path (Get-Location) 'reference\tauri-dev-migrated-dialog.stderr.log'
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-migrated-dialog.pid'
$process = Start-Process -FilePath 'npm.cmd' `
  -ArgumentList @('run', 'tauri', 'dev') `
  -WorkingDirectory (Get-Location) `
  -RedirectStandardOutput $stdout `
  -RedirectStandardError $stderr `
  -WindowStyle Hidden `
  -PassThru
Set-Content -LiteralPath $pidPath -Value $process.Id
$process.Id
```

Expected: a process id is printed and the app starts the MCP bridge on port `9223`.

- [x] **Step 4: Connect to the Tauri MCP bridge**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "start", "port": 9223 })
```

Expected: session connects to the running app. If the bridge is not ready yet, wait a few seconds and retry once.

- [x] **Step 5: Confirm account 11 is ready**

Tool call:

```text
mcp__tauri__.webview_execute_js({
  "script": "(async () => { return await window.__TAURI__.core.invoke('tg_get_account_statuses', { accountIds: [11] }); })()"
})
```

Expected: one status record with `account_id = 11` and `status = "ready"`. If account `11` is not ready, stop the app and document the slice as `blocked`.

- [x] **Step 6: Commit the Task 2 checkpoint**

Update this plan file by marking Task 2 checkboxes complete, then run:

```powershell
git add docs\superpowers\plans\2026-05-22-telegram-migrated-dialog-live-validation.md
git commit -m "docs: checkpoint telegram migrated-dialog runtime"
```

Expected: checkpoint commit succeeds. Runtime files under `reference/` remain ignored.

### Task 3: Select Fixture And Add Source

**Files:**
- Create/Modify: `reference/telegram-migrated-dialog-candidates.json`
- Create/Modify: `reference/telegram-migrated-dialog-validation-context.json`
- Create/Modify: `reference/telegram-migrated-dialog-add-result.json`
- Read: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`

- [x] **Step 1: List account 11 supergroup dialog candidates**

Tool call:

```text
mcp__tauri__.webview_execute_js({
  "script": "(async () => {\n  const accountId = 11;\n  const sources = await window.__TAURI__.core.invoke('list_telegram_sources', { accountId });\n  return sources\n    .filter((source) => source.source_subtype === 'supergroup')\n    .map((source, index) => ({\n      candidate_index: index + 1,\n      source_ref: String(source.id),\n      source_subtype: source.source_subtype,\n      is_member: source.is_member,\n      username_present: Boolean(source.username),\n      title: source.title\n    }));\n})()"
})
```

Expected: a list of account `11` supergroup dialogs. The returned data may include private titles for local fixture selection only.

- [x] **Step 2: Save the complete candidate list locally**

Save the complete JSON array returned by Step 1 to:

```text
reference/telegram-migrated-dialog-candidates.json
```

Then run:

```powershell
Get-Content -Raw -LiteralPath reference\telegram-migrated-dialog-candidates.json |
  ConvertFrom-Json |
  Select-Object candidate_index, source_ref, source_subtype, is_member, username_present
```

Expected: sanitized candidate rows print without titles. If the controlled migrated fixture is not listed as `supergroup`, stop and document `blocked`.

- [x] **Step 3: Human gate for controlled fixture selection**

Ask the operator which `candidate_index` corresponds to the controlled migrated small-group-to-supergroup fixture for account `11`. Do not ask for or record the private title in tracked docs.

Expected: operator provides one numeric `candidate_index`. Save the numeric chat answer to ignored local file `reference/telegram-migrated-dialog-selected-index.txt` before running Step 4.

- [x] **Step 4: Build runtime context from the selected candidate**

After the operator provides the numeric value and it has been saved to `reference/telegram-migrated-dialog-selected-index.txt`, run:

```powershell
$selectedCandidateIndex = [int](Get-Content -Raw -LiteralPath reference\telegram-migrated-dialog-selected-index.txt)
$candidatePath = Join-Path (Get-Location) 'reference\telegram-migrated-dialog-candidates.json'
$contextPath = Join-Path (Get-Location) 'reference\telegram-migrated-dialog-validation-context.json'
$candidates = Get-Content -Raw -LiteralPath $candidatePath | ConvertFrom-Json
$selected = $candidates | Where-Object { $_.candidate_index -eq $selectedCandidateIndex } | Select-Object -First 1
if (-not $selected) { throw "Selected candidate not found: $selectedCandidateIndex" }
if ($selected.source_subtype -ne 'supergroup') { throw "Selected candidate is not a supergroup" }
if ($selected.is_member -ne $true) { throw "Selected candidate is not currently a member dialog" }
$context = [ordered]@{
  account_id = 11
  candidate_index = [int]$selected.candidate_index
  source_ref = [string]$selected.source_ref
  expected_subtype = 'supergroup'
  dialog_classified_subtype = [string]$selected.source_subtype
  dialog_username_present = [bool]$selected.username_present
  source_id = $null
  source_created = $null
  source_preexisting = $null
  stale_same_external_rows = @()
  primary_classification = $null
  takeout_classification = $null
}
$context | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $contextPath -Encoding UTF8
Get-Content -Raw -LiteralPath $contextPath | ConvertFrom-Json |
  Select-Object account_id, candidate_index, source_ref, expected_subtype, dialog_classified_subtype, dialog_username_present
```

Expected: context records account `11`, expected subtype `supergroup`, and the selected numeric `source_ref`.

- [x] **Step 5: Capture pre-add source rows for this external id**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-validation-context.json")
context = json.loads(context_path.read_text(encoding="utf-8-sig"))

with sqlite3.connect(db_path) as conn:
    rows = conn.execute(
        """
        SELECT s.id, s.account_id, s.source_type, s.source_subtype, s.external_id,
               ts.source_subtype AS typed_source_subtype,
               ts.peer_kind, ts.peer_id,
               ts.access_hash IS NOT NULL AS has_access_hash,
               ts.username IS NOT NULL AND TRIM(ts.username) <> '' AS has_username,
               ts.resolution_strategy,
               s.last_sync_state, s.last_synced_at
        FROM sources s
        LEFT JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE s.account_id = ?
          AND s.source_type = 'telegram'
          AND s.external_id = ?
        ORDER BY s.id ASC
        """,
        (context["account_id"], context["source_ref"]),
    ).fetchall()

columns = [
    "source_id", "account_id", "source_type", "source_subtype", "external_id",
    "typed_source_subtype", "peer_kind", "peer_id", "has_access_hash",
    "has_username", "resolution_strategy", "last_sync_state", "last_synced_at",
]
records = [dict(zip(columns, row)) for row in rows]
context["pre_add_rows"] = records
context_path.write_text(json.dumps(context, indent=2), encoding="utf-8")
print(json.dumps({
    "pre_add_row_count": len(records),
    "pre_add_sanitized": [
        {
            "source_id": row["source_id"],
            "source_subtype": row["source_subtype"],
            "typed_source_subtype": row["typed_source_subtype"],
            "peer_kind": row["peer_kind"],
            "has_access_hash": bool(row["has_access_hash"]),
            "has_username": bool(row["has_username"]),
            "resolution_strategy": row["resolution_strategy"],
        }
        for row in records
    ],
}, indent=2))
'@ | python -
```

Expected: pre-add rows are captured in ignored context. A pre-existing correct `supergroup` row may be reused; a stale non-supergroup row is follow-up unless it is reused by the probe.

- [x] **Step 6: Add or reuse the selected fixture through app IPC**

Generate the app script:

```powershell
$ctx = Get-Content -Raw reference\telegram-migrated-dialog-validation-context.json | ConvertFrom-Json
$sourceRefJson = $ctx.source_ref | ConvertTo-Json -Compress
@"
(async () => {
  return await window.__TAURI__.core.invoke('add_telegram_source', {
    request: {
      accountId: $($ctx.account_id),
      sourceRef: $sourceRefJson,
      expectedSubtype: 'supergroup'
    }
  });
})()
"@
```

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set `script` to the complete JavaScript printed above.
```

Expected: a `SourceRecord` for account `11` and `source_subtype = "supergroup"`.

- [x] **Step 7: Save the Add Source result**

Save the JSON object returned by Step 6 to:

```text
reference/telegram-migrated-dialog-add-result.json
```

Expected: local result includes the selected source id and has no effect on tracked docs.

- [x] **Step 8: Verify stored migrated-dialog typed identity**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-validation-context.json")
add_result_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-add-result.json")
context = json.loads(context_path.read_text(encoding="utf-8-sig"))
add_result = json.loads(add_result_path.read_text(encoding="utf-8-sig"))

returned_source_id = add_result.get("id")

with sqlite3.connect(db_path) as conn:
    correct_rows = conn.execute(
        """
        SELECT s.id, s.account_id, s.source_type, s.source_subtype, s.external_id,
               title IS NOT NULL AND TRIM(title) <> '' AS title_present,
               ts.source_subtype AS typed_source_subtype,
               ts.peer_kind, ts.peer_id,
               ts.access_hash IS NOT NULL AS has_access_hash,
               ts.username IS NOT NULL AND TRIM(ts.username) <> '' AS has_username,
               ts.resolution_strategy,
               ts.identity_refreshed_at,
               s.last_sync_state, s.last_synced_at
        FROM sources s
        JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE s.account_id = ?
          AND s.source_type = 'telegram'
          AND s.source_subtype = 'supergroup'
          AND s.external_id = ?
        ORDER BY s.id ASC
        """,
        (context["account_id"], context["source_ref"]),
    ).fetchall()
    stale_rows = conn.execute(
        """
        SELECT s.id, s.source_subtype, ts.source_subtype, ts.peer_kind
        FROM sources s
        LEFT JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE s.account_id = ?
          AND s.source_type = 'telegram'
          AND s.external_id = ?
          AND s.source_subtype <> 'supergroup'
        ORDER BY s.id ASC
        """,
        (context["account_id"], context["source_ref"]),
    ).fetchall()

if not correct_rows:
    raise SystemExit("Add Source did not persist a supergroup row for the selected migrated fixture")

columns = [
    "source_id", "account_id", "source_type", "source_subtype", "external_id",
    "title_present", "typed_source_subtype", "peer_kind", "peer_id",
    "has_access_hash", "has_username", "resolution_strategy",
    "identity_refreshed_at", "last_sync_state", "last_synced_at",
]
records = [dict(zip(columns, row)) for row in correct_rows]

selected = None
if returned_source_id is not None:
    selected = next((row for row in records if row["source_id"] == returned_source_id), None)
if selected is None:
    selected = records[0]

if selected["source_subtype"] != "supergroup":
    raise SystemExit("selected source row is not supergroup")
if selected["typed_source_subtype"] != "supergroup":
    raise SystemExit("typed source subtype is not supergroup")
if selected["peer_kind"] != "channel":
    raise SystemExit(f"typed peer kind is not channel: {selected['peer_kind']}")
if selected["resolution_strategy"] != "dialog":
    raise SystemExit(f"resolution strategy is not dialog: {selected['resolution_strategy']}")

pre_correct_ids = {
    row["source_id"]
    for row in context.get("pre_add_rows", [])
    if row.get("source_subtype") == "supergroup"
}
context["source_id"] = selected["source_id"]
context["source_preexisting"] = selected["source_id"] in pre_correct_ids
context["source_created"] = selected["source_id"] not in pre_correct_ids
context["stored_identity"] = {
    "source_id": selected["source_id"],
    "account_id": selected["account_id"],
    "source_subtype": selected["source_subtype"],
    "peer_kind": selected["peer_kind"],
    "peer_id": selected["peer_id"],
    "has_access_hash": bool(selected["has_access_hash"]),
    "has_username": bool(selected["has_username"]),
    "resolution_strategy": selected["resolution_strategy"],
    "identity_refreshed_at": selected["identity_refreshed_at"],
    "title_present": bool(selected["title_present"]),
}
context["stale_same_external_rows"] = [
    {
        "source_id": row[0],
        "source_subtype": row[1],
        "typed_source_subtype": row[2],
        "peer_kind": row[3],
    }
    for row in stale_rows
]
context_path.write_text(json.dumps(context, indent=2), encoding="utf-8")

print(json.dumps({
    "source_id": selected["source_id"],
    "source_created": context["source_created"],
    "source_preexisting": context["source_preexisting"],
    "source_subtype": selected["source_subtype"],
    "peer_kind": selected["peer_kind"],
    "peer_id": selected["peer_id"],
    "has_access_hash": bool(selected["has_access_hash"]),
    "has_username": bool(selected["has_username"]),
    "resolution_strategy": selected["resolution_strategy"],
    "stale_same_external_row_count": len(stale_rows),
}, indent=2))
'@ | python -
```

Expected: `source_subtype = supergroup`, `peer_kind = channel`, and `resolution_strategy = dialog`. Access-hash presence is recorded as a boolean. If this step fails, classify primary as `failed`.

- [x] **Step 9: Commit the Task 3 checkpoint**

Update this plan file by marking Task 3 checkboxes complete, then run:

```powershell
git add docs\superpowers\plans\2026-05-22-telegram-migrated-dialog-live-validation.md
git commit -m "docs: checkpoint telegram migrated-dialog source"
```

Expected: checkpoint commit succeeds. Runtime files under `reference/` remain ignored.

### Task 4: Run Primary Sync And Evaluate Runtime Invariants

**Files:**
- Create: `reference/telegram-migrated-dialog-capture-snapshot.py`
- Create/Modify: `reference/telegram-migrated-dialog-validation-snapshots.json`
- Create/Modify: `reference/telegram-migrated-dialog-sync-results.json`
- Create: `reference/telegram-migrated-dialog-primary-evaluation.json`
- Read: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`

- [x] **Step 1: Create the snapshot helper**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3
import sys

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-validation-context.json")
snapshots_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-validation-snapshots.json")

if len(sys.argv) != 2:
    raise SystemExit("usage: telegram-migrated-dialog-capture-snapshot.py <label>")

label = sys.argv[1]
allowed = {"before_primary_sync", "after_primary_sync", "before_takeout", "after_takeout"}
if label not in allowed:
    raise SystemExit(f"unsupported snapshot label: {label}")

context = json.loads(context_path.read_text(encoding="utf-8-sig"))
source_id = context["source_id"]
account_id = context["account_id"]

with sqlite3.connect(db_path) as conn:
    source_row = conn.execute(
        """
        SELECT id, account_id, source_type, source_subtype, external_id,
               title IS NOT NULL AND TRIM(title) <> '' AS title_present,
               last_sync_state, last_synced_at, is_active, is_member
        FROM sources
        WHERE id = ?
        """,
        (source_id,),
    ).fetchone()
    telegram_row = conn.execute(
        """
        SELECT source_id, account_id, source_subtype, peer_kind, peer_id,
               access_hash IS NOT NULL AS has_access_hash,
               username IS NOT NULL AND TRIM(username) <> '' AS has_username,
               resolution_strategy,
               identity_refreshed_at
        FROM telegram_sources
        WHERE source_id = ?
        """,
        (source_id,),
    ).fetchone()
    item_row = conn.execute(
        """
        SELECT COUNT(*) AS item_count,
               MAX(CAST(external_id AS INTEGER)) AS max_external_id,
               MAX(published_at) AS max_published_at
        FROM items
        WHERE source_id = ?
        """,
        (source_id,),
    ).fetchone()
    history_groups = conn.execute(
        """
        SELECT history_peer_kind, history_peer_id, COUNT(*) AS item_count,
               MIN(telegram_message_id) AS min_message_id,
               MAX(telegram_message_id) AS max_message_id
        FROM telegram_messages
        WHERE source_id = ?
        GROUP BY history_peer_kind, history_peer_id
        ORDER BY history_peer_kind, history_peer_id
        """,
        (source_id,),
    ).fetchall()
    account_sources = conn.execute(
        """
        SELECT s.id, s.last_sync_state, s.last_synced_at,
               (SELECT COUNT(*) FROM items WHERE source_id = s.id) AS item_count
        FROM sources s
        WHERE s.account_id = ?
          AND s.source_type = 'telegram'
        ORDER BY s.id
        """,
        (account_id,),
    ).fetchall()

if source_row is None:
    raise SystemExit("source row missing")
if telegram_row is None:
    raise SystemExit("telegram_sources row missing")

source_columns = [
    "id", "account_id", "source_type", "source_subtype", "external_id",
    "title_present", "last_sync_state", "last_synced_at", "is_active", "is_member",
]
telegram_columns = [
    "source_id", "account_id", "source_subtype", "peer_kind", "peer_id",
    "has_access_hash", "has_username", "resolution_strategy", "identity_refreshed_at",
]
history_columns = [
    "history_peer_kind", "history_peer_id", "item_count", "min_message_id", "max_message_id",
]

snapshot = {
    "source": dict(zip(source_columns, source_row)),
    "telegram_source": dict(zip(telegram_columns, telegram_row)),
    "items": {
        "item_count": item_row[0],
        "max_external_id": item_row[1],
        "max_published_at": item_row[2],
    },
    "history_groups": [dict(zip(history_columns, row)) for row in history_groups],
    "history_peer_count": len(history_groups),
    "account_source_guard": {
        str(row[0]): {
            "source_id": row[0],
            "last_sync_state": row[1],
            "last_synced_at": row[2],
            "item_count": row[3],
        }
        for row in account_sources
    },
}
snapshot["source"]["title_present"] = bool(snapshot["source"]["title_present"])
snapshot["source"]["is_active"] = bool(snapshot["source"]["is_active"])
snapshot["source"]["is_member"] = bool(snapshot["source"]["is_member"])
snapshot["telegram_source"]["has_access_hash"] = bool(snapshot["telegram_source"]["has_access_hash"])
snapshot["telegram_source"]["has_username"] = bool(snapshot["telegram_source"]["has_username"])

snapshots = {}
if snapshots_path.exists():
    snapshots = json.loads(snapshots_path.read_text(encoding="utf-8-sig"))
snapshots[label] = snapshot
snapshots_path.write_text(json.dumps(snapshots, indent=2), encoding="utf-8")
print(json.dumps({label: snapshot}, indent=2))
'@ | Set-Content -LiteralPath reference\telegram-migrated-dialog-capture-snapshot.py -Encoding UTF8
```

Expected: helper exists and accepts all four labels.

- [x] **Step 2: Capture the `before_primary_sync` snapshot**

Run:

```powershell
python reference\telegram-migrated-dialog-capture-snapshot.py before_primary_sync
```

Expected: snapshot includes source, typed identity, item stats, history groups, and account-wide mutation guard.

- [x] **Step 3: Run normal `sync_source`**

Generate the app script:

```powershell
$ctx = Get-Content -Raw reference\telegram-migrated-dialog-validation-context.json | ConvertFrom-Json
@"
(async () => {
  try {
    const result = await window.__TAURI__.core.invoke('sync_source', {
      sourceId: $($ctx.source_id)
    });
    return { ok: true, result };
  } catch (error) {
    return {
      ok: false,
      error: {
        kind: error && error.kind ? error.kind : null,
        message: error && error.message ? error.message : String(error)
      }
    };
  }
})()
"@
```

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set `script` to the complete JavaScript printed above.
```

Expected: `{ ok: true, result: <SyncResult> }`. If sync returns `{ ok: false, error: <typed error> }`, capture it and classify primary according to the evaluation script.

- [x] **Step 4: Save the primary sync result**

Save the complete JSON object returned by Step 3 to:

```text
reference/telegram-migrated-dialog-sync-results.json
```

Use this shape:

```json
{
  "primary_sync": {
    "ok": true,
    "result": {}
  }
}
```

Expected: local sync result is available for evaluation.

- [x] **Step 5: Capture the `after_primary_sync` snapshot**

Run:

```powershell
python reference\telegram-migrated-dialog-capture-snapshot.py after_primary_sync
```

Expected: after snapshot has the same selected source id and account `11` mutation guard.

- [x] **Step 6: Evaluate primary runtime invariants**

Run:

```powershell
@'
from pathlib import Path
import json

context_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-validation-context.json")
snapshots_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-validation-snapshots.json")
sync_results_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-sync-results.json")
evaluation_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-primary-evaluation.json")

context = json.loads(context_path.read_text(encoding="utf-8-sig"))
snapshots = json.loads(snapshots_path.read_text(encoding="utf-8-sig"))
sync_results = json.loads(sync_results_path.read_text(encoding="utf-8-sig"))

before = snapshots["before_primary_sync"]
after = snapshots["after_primary_sync"]
sync = sync_results["primary_sync"]
source_id = str(context["source_id"])
peer_id = context["stored_identity"]["peer_id"]

stored_identity_ok = (
    context["dialog_classified_subtype"] == "supergroup"
    and after["source"]["source_subtype"] == "supergroup"
    and after["telegram_source"]["source_subtype"] == "supergroup"
    and after["telegram_source"]["peer_kind"] == "channel"
    and after["telegram_source"]["peer_id"] == peer_id
    and after["telegram_source"]["resolution_strategy"] == "dialog"
)

sync_ok = bool(sync.get("ok"))
history_groups = after["history_groups"]
history_peer_count = after["history_peer_count"]
current_history_group_ok = (
    history_peer_count == 1
    and history_groups[0]["history_peer_kind"] == "channel"
    and history_groups[0]["history_peer_id"] == peer_id
)
zero_item_ambiguous = after["items"]["item_count"] == 0 and history_peer_count == 0
has_history_proof = current_history_group_ok and after["items"]["item_count"] > 0

before_guard = before["account_source_guard"]
after_guard = after["account_source_guard"]
other_source_mutations = []
for sid, before_row in before_guard.items():
    after_row = after_guard.get(sid)
    if after_row is None:
        other_source_mutations.append({"source_id": int(sid), "reason": "missing_after"})
        continue
    if sid != source_id and before_row != after_row:
        other_source_mutations.append({
            "source_id": int(sid),
            "before": before_row,
            "after": after_row,
        })
selected_source_present = source_id in after_guard
mutation_guard_ok = selected_source_present and not other_source_mutations

wrong_peer_groups = [
    group for group in history_groups
    if group["history_peer_kind"] != "channel" or group["history_peer_id"] != peer_id
]
wrong_peer_absent = not wrong_peer_groups

if not sync_ok:
    classification = "failed"
elif not stored_identity_ok:
    classification = "failed"
elif not mutation_guard_ok:
    classification = "failed"
elif not wrong_peer_absent:
    classification = "failed"
elif zero_item_ambiguous:
    classification = "needs_follow_up"
elif not has_history_proof:
    classification = "needs_follow_up"
else:
    classification = "passed"

evaluation = {
    "classification": classification,
    "source_id": context["source_id"],
    "account_id": context["account_id"],
    "source_created": context["source_created"],
    "source_preexisting": context["source_preexisting"],
    "dialog_classified_subtype": context["dialog_classified_subtype"],
    "source_subtype": after["telegram_source"]["source_subtype"],
    "peer_kind": after["telegram_source"]["peer_kind"],
    "peer_id": after["telegram_source"]["peer_id"],
    "has_access_hash": after["telegram_source"]["has_access_hash"],
    "has_username": after["telegram_source"]["has_username"],
    "resolution_strategy": after["telegram_source"]["resolution_strategy"],
    "sync_ok": sync_ok,
    "sync_result": sync,
    "stored_identity_ok": stored_identity_ok,
    "history_peer_count": history_peer_count,
    "history_groups": history_groups,
    "current_history_group_ok": current_history_group_ok,
    "zero_item_ambiguous": zero_item_ambiguous,
    "has_history_proof": has_history_proof,
    "mutation_guard_ok": mutation_guard_ok,
    "other_source_mutations": other_source_mutations,
    "wrong_peer_absent": wrong_peer_absent,
    "wrong_peer_groups": wrong_peer_groups,
    "before_item_count": before["items"]["item_count"],
    "after_item_count": after["items"]["item_count"],
    "before_last_sync_state": before["source"]["last_sync_state"],
    "after_last_sync_state": after["source"]["last_sync_state"],
    "before_last_synced_at": before["source"]["last_synced_at"],
    "after_last_synced_at": after["source"]["last_synced_at"],
    "stale_same_external_row_count": len(context.get("stale_same_external_rows", [])),
}

context["primary_classification"] = classification
context_path.write_text(json.dumps(context, indent=2), encoding="utf-8")
evaluation_path.write_text(json.dumps(evaluation, indent=2), encoding="utf-8")
print(json.dumps(evaluation, indent=2))
'@ | python -
```

Expected:

- `classification = "passed"` only when sync succeeds, stored identity is supergroup/channel, there is exactly one current channel history-peer group, at least one local item/history row proves the peer, and no other account `11` Telegram source mutates.
- `classification = "needs_follow_up"` for a successful but empty sync with no local history-peer proof.
- `classification = "failed"` for wrong subtype, wrong peer, cross-source mutation, untyped sync failure, or unsafe duplicate boundary evidence.

- [x] **Step 7: Commit the Task 4 checkpoint**

Update this plan file by marking Task 4 checkboxes complete, then run:

```powershell
git add docs\superpowers\plans\2026-05-22-telegram-migrated-dialog-live-validation.md
git commit -m "docs: checkpoint telegram migrated-dialog primary sync"
```

Expected: checkpoint commit succeeds.

### Task 5: Optional Takeout Detect-And-Defer Smoke

**Files:**
- Modify: `reference/telegram-migrated-dialog-validation-context.json`
- Modify: `reference/telegram-migrated-dialog-validation-snapshots.json`
- Create/Modify: `reference/telegram-migrated-dialog-takeout-result.json`
- Create: `reference/telegram-migrated-dialog-takeout-evaluation.json`
- Read: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`

- [x] **Step 1: Check whether primary passed**

Run:

```powershell
Get-Content -Raw reference\telegram-migrated-dialog-primary-evaluation.json |
  ConvertFrom-Json |
  Select-Object classification, source_id, history_peer_count, has_history_proof
```

Expected: proceed with Takeout smoke only if `classification = passed`. If primary is `blocked`, `failed`, or `needs_follow_up`, skip Takeout smoke, set `takeout_classification = "not_run_primary_not_passed"` in context, and continue to Task 6.

Task 5 checkpoint: primary was `needs_follow_up`, so Takeout smoke steps 2-10 were skipped and `takeout_classification = "not_run_primary_not_passed"` was recorded in ignored runtime context.

- [ ] **Step 2: Confirm no active Takeout job for the source**

Generate the app script:

```powershell
$ctx = Get-Content -Raw reference\telegram-migrated-dialog-validation-context.json | ConvertFrom-Json
@"
(async () => {
  const jobs = await window.__TAURI__.core.invoke('list_takeout_source_import_jobs');
  return jobs.filter((job) => job.source_id === $($ctx.source_id));
})()
"@
```

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set `script` to the complete JavaScript printed above.
```

Expected: no active `queued`, `running`, or `cancel_requested` job for this source. If an active job exists, classify the Takeout smoke as `blocked_active_job`.

- [ ] **Step 3: Capture `before_takeout` snapshot**

Run:

```powershell
python reference\telegram-migrated-dialog-capture-snapshot.py before_takeout
```

Expected: before-Takeout history groups match the primary post-sync current channel group.

- [ ] **Step 4: Start Takeout import**

Generate the app script:

```powershell
$ctx = Get-Content -Raw reference\telegram-migrated-dialog-validation-context.json | ConvertFrom-Json
@"
(async () => {
  try {
    const started = await window.__TAURI__.core.invoke('start_takeout_source_import', {
      sourceId: $($ctx.source_id)
    });
    return { ok: true, started };
  } catch (error) {
    return {
      ok: false,
      error: {
        kind: error && error.kind ? error.kind : null,
        message: error && error.message ? error.message : String(error)
      }
    };
  }
})()
"@
```

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set `script` to the complete JavaScript printed above.
```

Expected: either `{ ok: true, started: { job_id: "takeout-N" } }` or a typed Telegram/runtime blocker. If start is blocked before unsafe writes, record `takeout_classification = blocked_start` and continue to Task 6.

- [ ] **Step 5: Save Takeout start result**

Save the JSON object returned by Step 4 to:

```text
reference/telegram-migrated-dialog-takeout-result.json
```

Use this shape initially:

```json
{
  "start": {
    "ok": true,
    "started": {}
  },
  "terminal_job": null
}
```

Expected: local Takeout result exists.

- [ ] **Step 6: Poll Takeout job to terminal or bounded timeout**

Generate the app script after Step 5 succeeds:

```powershell
$takeout = Get-Content -Raw reference\telegram-migrated-dialog-takeout-result.json | ConvertFrom-Json
$jobId = $takeout.start.started.job_id
@"
(async () => {
  const jobId = "$jobId";
  const deadline = Date.now() + 15 * 60 * 1000;
  let last = null;
  while (Date.now() < deadline) {
    const jobs = await window.__TAURI__.core.invoke('list_takeout_source_import_jobs');
    last = jobs.find((job) => job.job_id === jobId) || null;
    if (last && ['completed', 'failed', 'cancelled'].includes(last.status)) {
      return { timeout: false, job: last };
    }
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }
  return { timeout: true, job: last };
})()
"@
```

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set `script` to the complete JavaScript printed above.
```

Expected: terminal job within 15 minutes for a small fixture. If `timeout = true`, cancel the job in Step 7 and record the smoke as blocked unless unsafe writes are observed.

- [ ] **Step 7: Cancel timed-out Takeout job if needed**

If Step 6 returned `timeout = true`, generate and run:

```powershell
$takeout = Get-Content -Raw reference\telegram-migrated-dialog-takeout-result.json | ConvertFrom-Json
$jobId = $takeout.start.started.job_id
@"
(async () => {
  return await window.__TAURI__.core.invoke('cancel_takeout_source_import', {
    jobId: "$jobId"
  });
})()
"@
```

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set `script` to the complete JavaScript printed above.
```

Expected: cancellation requested. Continue to snapshot/evaluation so unsafe writes can still be checked.

- [ ] **Step 8: Save terminal or timeout job result**

Merge the Step 6 result and optional Step 7 cancellation result into:

```text
reference/telegram-migrated-dialog-takeout-result.json
```

Expected shape:

```json
{
  "start": {},
  "terminal_job": {},
  "cancel": null
}
```

- [ ] **Step 9: Capture `after_takeout` snapshot**

Run:

```powershell
python reference\telegram-migrated-dialog-capture-snapshot.py after_takeout
```

Expected: local history groups are available for before/after comparison.

- [ ] **Step 10: Evaluate Takeout smoke**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-validation-context.json")
snapshots_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-validation-snapshots.json")
takeout_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-takeout-result.json")
evaluation_path = Path(r"G:\Develop\Extractum\reference\telegram-migrated-dialog-takeout-evaluation.json")

context = json.loads(context_path.read_text(encoding="utf-8-sig"))
snapshots = json.loads(snapshots_path.read_text(encoding="utf-8-sig"))
takeout = json.loads(takeout_path.read_text(encoding="utf-8-sig"))
source_id = context["source_id"]
current_peer_id = context["stored_identity"]["peer_id"]

start_ok = bool((takeout.get("start") or {}).get("ok"))
terminal_job = takeout.get("terminal_job") or {}
timeout = bool(terminal_job.get("timeout")) if isinstance(terminal_job, dict) else False
job = terminal_job.get("job") if isinstance(terminal_job, dict) else None
batch_id = job.get("batch_id") if isinstance(job, dict) else None

batch = None
warnings = []
observations = []
if batch_id is not None:
    with sqlite3.connect(db_path) as conn:
        row = conn.execute(
            """
            SELECT b.id, b.status, b.completeness, b.item_inserted_count,
                   b.item_observed_count, b.warning_count,
                   t.history_scope, t.migrated_history_detected,
                   t.migrated_history_imported, t.only_my_messages
            FROM ingest_batches b
            JOIN telegram_takeout_batches t ON t.batch_id = b.id
            WHERE b.id = ?
            """,
            (batch_id,),
        ).fetchone()
        if row is not None:
            columns = [
                "batch_id", "status", "completeness", "item_inserted_count",
                "item_observed_count", "warning_count", "history_scope",
                "migrated_history_detected", "migrated_history_imported",
                "only_my_messages",
            ]
            batch = dict(zip(columns, row))
        warnings = [
            {"code": code, "warning_count": count}
            for code, count in conn.execute(
                """
                SELECT code, COUNT(*) AS warning_count
                FROM ingest_batch_warnings
                WHERE batch_id = ?
                GROUP BY code
                ORDER BY code
                """,
                (batch_id,),
            ).fetchall()
        ]
        observations = [
            {"provider_identity": identity, "outcome": outcome, "count": count}
            for identity, outcome, count in conn.execute(
                """
                SELECT provider_identity, outcome, COUNT(*) AS count
                FROM ingest_item_observations
                WHERE batch_id = ?
                GROUP BY provider_identity, outcome
                ORDER BY provider_identity, outcome
                """,
                (batch_id,),
            ).fetchall()
        ]

before_groups = snapshots.get("before_takeout", {}).get("history_groups", [])
after_groups = snapshots.get("after_takeout", {}).get("history_groups", [])
before_group_keys = {
    (group["history_peer_kind"], group["history_peer_id"]) for group in before_groups
}
after_group_keys = {
    (group["history_peer_kind"], group["history_peer_id"]) for group in after_groups
}
new_group_keys = sorted(after_group_keys - before_group_keys)
unsafe_groups = [
    {"history_peer_kind": kind, "history_peer_id": peer_id}
    for kind, peer_id in new_group_keys
    if kind != "channel" or peer_id != current_peer_id
]
chat_group_present_after = any(group["history_peer_kind"] == "chat" for group in after_groups)
warning_codes = {warning["code"] for warning in warnings}

if not start_ok:
    classification = "blocked_start"
elif unsafe_groups or chat_group_present_after:
    classification = "failed"
elif timeout:
    classification = "blocked_timeout"
elif batch is None:
    classification = "blocked_no_batch"
elif (
    batch["status"] == "completed"
    and batch["completeness"] == "partial"
    and batch["migrated_history_detected"] == 1
    and batch["migrated_history_imported"] == 0
    and "migrated_history_deferred" in warning_codes
    and (
        batch["history_scope"] == "current_history_with_migrated_deferred"
        or (batch["history_scope"] == "mixed_partial" and batch["only_my_messages"] == 1)
    )
):
    classification = "passed"
else:
    classification = "needs_follow_up"

evaluation = {
    "classification": classification,
    "source_id": source_id,
    "start_ok": start_ok,
    "timeout": timeout,
    "job": job,
    "batch": batch,
    "warning_codes": sorted(warning_codes),
    "unsafe_groups": unsafe_groups,
    "chat_group_present_after": chat_group_present_after,
    "new_group_keys": [
        {"history_peer_kind": kind, "history_peer_id": peer_id}
        for kind, peer_id in new_group_keys
    ],
    "observation_count": len(observations),
}

context["takeout_classification"] = classification
context_path.write_text(json.dumps(context, indent=2), encoding="utf-8")
evaluation_path.write_text(json.dumps(evaluation, indent=2), encoding="utf-8")
print(json.dumps(evaluation, indent=2))
'@ | python -
```

Expected:

- `classification = "passed"` if completed Takeout records migrated-history detection/deferment and no non-current history peer group appears.
- `classification = "blocked_start"` or `blocked_timeout` for Telegram/runtime blockers before unsafe writes.
- `classification = "failed"` if old small-group rows, `history_peer_kind = chat`, or another non-current history peer group appears after the smoke.

- [x] **Step 11: Commit the Task 5 checkpoint**

Update this plan file by marking Task 5 checkboxes complete, then run:

```powershell
git add docs\superpowers\plans\2026-05-22-telegram-migrated-dialog-live-validation.md
git commit -m "docs: checkpoint telegram migrated-dialog takeout"
```

Expected: checkpoint commit succeeds.

### Task 6: Stop Runtime Processes

**Files:**
- Read: `reference/tauri-dev-migrated-dialog.pid`
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`

- [x] **Step 1: Stop the Tauri MCP session**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "stop" })
```

Expected: all bridge sessions are stopped.

- [x] **Step 2: Stop the Tauri dev process tree**

Run:

```powershell
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-migrated-dialog.pid'
if (Test-Path -LiteralPath $pidPath) {
  $rootPid = [int](Get-Content -LiteralPath $pidPath)
  function Stop-ProcessTree([int]$ProcessId) {
    Get-CimInstance Win32_Process |
      Where-Object { $_.ParentProcessId -eq $ProcessId } |
      ForEach-Object { Stop-ProcessTree -ProcessId ([int]$_.ProcessId) }
    Stop-Process -Id $ProcessId -Force -ErrorAction SilentlyContinue
  }
  Stop-ProcessTree -ProcessId $rootPid
}
```

Expected: the dev process tree exits. If access is denied, rerun the process-tree stop with escalated permission.

- [x] **Step 3: Confirm no runtime processes remain**

Run:

```powershell
Get-Process | Where-Object {
  $_.ProcessName -like '*extractum*' -or
  $_.ProcessName -like '*tauri*' -or
  $_.ProcessName -eq 'cargo'
} | Select-Object Id, ProcessName, Path
```

Expected: no leftover Extractum, Tauri, or Cargo process from this live run.

- [x] **Step 4: Commit the Task 6 checkpoint**

Update this plan file by marking Task 6 checkboxes complete, then run:

```powershell
git add docs\superpowers\plans\2026-05-22-telegram-migrated-dialog-live-validation.md
git commit -m "docs: checkpoint telegram migrated-dialog cleanup"
```

Expected: checkpoint commit succeeds.

### Task 7: Document The Live Result

**Files:**
- Read: `reference/telegram-migrated-dialog-validation-context.json`
- Read: `reference/telegram-migrated-dialog-primary-evaluation.json`
- Read: `reference/telegram-migrated-dialog-takeout-evaluation.json`
- Read: `reference/telegram-migrated-dialog-sync-results.json`
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`

- [ ] **Step 1: Read sanitized runtime evidence**

Run:

```powershell
Get-Content -Raw reference\telegram-migrated-dialog-primary-evaluation.json | ConvertFrom-Json
if (Test-Path reference\telegram-migrated-dialog-takeout-evaluation.json) {
  Get-Content -Raw reference\telegram-migrated-dialog-takeout-evaluation.json | ConvertFrom-Json
}
Get-Content -Raw reference\telegram-migrated-dialog-sync-results.json | ConvertFrom-Json
```

Expected: evidence is available for classification and docs. Do not copy private titles, usernames, access-hash values, or message text into tracked docs.

- [ ] **Step 2: Update the verification matrix row**

If primary `classification = passed`, update the row to this shape with observed values:

```markdown
| Migrated small group -> supergroup | passed | dialog-backed likely private | Account `11` controlled migrated dialog was listed as `supergroup`; `add_telegram_source` created or reused the observed source id through the dialog path | `source_subtype = supergroup`, `peer_kind = channel`, observed `peer_id`, access-hash presence recorded, username presence recorded, `resolution_strategy = dialog` | `sync_source(observed source id)` succeeded with inserted/skipped/last-message values from the live result | `telegram_messages` had exactly one current `channel` history-peer group; account `11` mutation guard showed only the selected source changed | Secondary Takeout smoke passed, was blocked before unsafe writes, was not run, or failed with a separate Takeout follow-up |
```

If primary classification is `needs_follow_up`, `blocked`, or `failed`, set that status and describe the non-sensitive reason in the row. If the secondary Takeout smoke imported unsafe rows, do not present the slice as a clean pass; use status wording that records `primary passed, Takeout failed`.

- [ ] **Step 3: Add the dated live-run note**

Append a section to `docs/superpowers/verification/telegram-runtime-private-source-validation.md`:

```markdown
## 2026-05-22 Migrated Dialog Follow-Up

- Account label: account `11`; no credentials, phone numbers, session data,
  private message content, private titles, private usernames, public username
  values, or access-hash values recorded.
- App commit: the exact `git log -1 --oneline` value captured in Task 1.
- Fixture: controlled dialog-backed migrated small-group-to-supergroup source;
  live dialog listing classified it as `supergroup`.
- Add Source result: record whether the observed source id was created or
  pre-existing. If stale same-external-id rows existed, record only the count
  and sanitized follow-up status.
- Stored identity: `source_subtype = supergroup`, `peer_kind = channel`,
  observed `peer_id`, access-hash presence, username presence, and
  `resolution_strategy = dialog`.
- Primary sync: record `inserted`, `skipped`, `last_message_id`, and warning
  count/codes from `sync_source(observed source id)`.
- Runtime wrong-peer check: record history peer count, current
  `history_peer_kind/history_peer_id`, before/after item counts, and account
  `11` mutation guard conclusion.
- Primary result: record `passed`, `needs follow-up`, `failed`, or `blocked`
  and the invariant that determined the classification.
- Secondary Takeout smoke: record `passed`, blocked reason, not-run reason, or
  failure. If attempted, record only sanitized `status`, `completeness`,
  `history_scope`, `migrated_history_detected`,
  `migrated_history_imported`, warning code presence, and local-only
  conclusion that no old small-group history rows were imported.
```

Use observed sanitized numeric values and conclusions from the ignored runtime evidence.

- [ ] **Step 4: Update backlog according to primary and secondary results**

If primary runtime classification is `passed`, remove this row from `docs/backlog.md` section `3.1`:

```markdown
- [ ] verify behavior for migrated small-group-to-supergroup dialogs
```

Also update the `Priority Snapshot` Telegram row. If section `3.1` becomes empty, remove the Telegram runtime/private-source row from the priority snapshot or change it to reflect only a concrete remaining follow-up discovered by this run.

If Takeout smoke is `blocked_start`, `blocked_timeout`, `needs_follow_up`, or `failed`, add or refine a concrete non-sensitive row in section `3.3 Takeout Source Import Follow-Ups`. Do not reopen the runtime/private-source row if primary passed and Takeout was merely blocked before unsafe writes. If Takeout imported unsafe rows, keep or add a high-priority Takeout data-integrity follow-up.

If primary runtime classification is `blocked`, `needs_follow_up`, or `failed`, keep or replace the runtime backlog item with the concrete sanitized follow-up discovered by the probe.

- [ ] **Step 5: Commit the Task 7 checkpoint**

Run:

```powershell
git diff --stat
git diff --name-only
git diff --check
```

Expected tracked files are:

```text
docs/backlog.md
docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md
docs/superpowers/verification/telegram-runtime-private-source-validation.md
```

Then run:

```powershell
git add docs\backlog.md docs\superpowers\plans\2026-05-22-telegram-migrated-dialog-live-validation.md docs\superpowers\verification\telegram-runtime-private-source-validation.md
git commit -m "docs: checkpoint telegram migrated-dialog result"
```

Expected: checkpoint commit succeeds.

### Task 8: Final Verification And Branch Completion

**Files:**
- Verify: `docs/backlog.md`
- Verify: `docs/superpowers/plans/2026-05-22-telegram-migrated-dialog-live-validation.md`
- Verify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`

- [ ] **Step 1: Check tracked diff scope**

Run:

```powershell
git diff --stat
git diff --name-only
```

Expected: no uncommitted tracked diff before final checkbox edit. Runtime files under `reference/` must not appear.

- [ ] **Step 2: Mark Task 8 checkboxes complete**

Update this plan file by marking Task 8 checkboxes complete.

- [ ] **Step 3: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: exit code 0. Known LF/CRLF warnings are acceptable if no whitespace errors are reported.

- [ ] **Step 4: Commit final plan checkpoint**

Run:

```powershell
git add docs\superpowers\plans\2026-05-22-telegram-migrated-dialog-live-validation.md
git commit -m "docs: checkpoint telegram migrated-dialog final verification"
```

Expected: final plan checkpoint commit succeeds.

- [ ] **Step 5: Verify the branch**

Run:

```powershell
git diff --check HEAD~1..HEAD
git status --short --branch
git log -5 --oneline
```

Expected: whitespace check exits 0, status is clean on `telegram-migrated-dialog-live-validation`, and the recent log shows the per-task checkpoint commits.

- [ ] **Step 6: Prepare merge handoff**

If all required verification passes, report:

```text
Primary runtime classification: use the `classification` value from `reference/telegram-migrated-dialog-primary-evaluation.json`
Secondary Takeout smoke classification: use the value from `reference/telegram-migrated-dialog-takeout-evaluation.json`, or `not_run` if the file was not created
Latest branch commit: use the output of `git log -1 --oneline`
Runtime evidence remains ignored under reference/*
```

Do not merge to `main` until the operator confirms branch completion.
