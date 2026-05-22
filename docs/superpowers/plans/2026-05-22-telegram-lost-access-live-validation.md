# Telegram Lost Access Live Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate that a real private Telegram source remains explainable and does not mutate identity/state/items when the owning account loses access.

**Architecture:** Run a docs-only live validation slice against a controlled private supergroup or private channel. Add or reuse the source through the normal Tauri IPC path, baseline-sync it while accessible, capture snapshots, pause for the operator to revoke account access, attempt sync again, evaluate invariants, and document the sanitized result.

**Tech Stack:** PowerShell, Python `sqlite3`, SQLite, Tauri dev app and MCP bridge, Markdown verification docs.

---

## File Structure

- Read: `docs/superpowers/specs/2026-05-22-telegram-lost-access-live-validation-design.md`
  - Approved design, pass criteria, and outcome classification.
- Read: `src-tauri/src/sources/sync.rs`
  - Verify failed sync does not call `finalize_sync`.
- Read: `src-tauri/src/sources/peer_resolution.rs`
  - Verify typed private-source resolution failure wording and stored-peer-first behavior.
- Read: `src-tauri/src/error.rs`
  - Verify `AppErrorKind` values and typed error serialization.
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
  - Mark the `No-longer-member, left, or private access lost` matrix row according to the live result and add a dated note.
- Modify: `docs/backlog.md`
  - Remove the lost-access `3.1` row only if the live validation passes.
- Create runtime-only ignored files under `reference/`
  - `telegram-lost-access-validation-context.json`
  - `telegram-lost-access-dialog-candidates.json`
  - `telegram-lost-access-capture-snapshot.py`
  - `telegram-lost-access-validation-snapshots.json`
  - `telegram-lost-access-validation-evaluation.json`
  - `telegram-lost-access-sync-results.json`
  - `tauri-dev-lost-access.stdout.log`
  - `tauri-dev-lost-access.stderr.log`
  - `tauri-dev-lost-access.pid`

Do not modify Rust, Svelte, schema, or app code during this slice. If code changes become necessary, stop and switch to a RED test first.

## Privacy Rules

- Do not write private Telegram titles, usernames, message text, phone numbers, session data, API data, API hashes, access-hash values, or auth material to tracked docs.
- Runtime-only `reference/*` files may contain local operator context. Keep terminal/documentation output sanitized where possible.
- Tracked docs may record source ids, account id, `source_subtype`, `peer_kind`, `peer_id`, boolean presence fields, sync counts, typed error kind, and sanitized message text.

### Task 1: Static Pre-Flight Audit

**Files:**
- Read: `src-tauri/src/sources/sync.rs`
- Read: `src-tauri/src/sources/peer_resolution.rs`
- Read: `src-tauri/src/error.rs`

- [x] **Step 1: Confirm tracked workspace state**

Run:

```powershell
git status --short --branch
git log -1 --oneline
```

Expected: clean `## main` before live validation begins. Record the commit line for the dated verification note.

- [x] **Step 2: Verify sync failure boundary**

Run:

```powershell
rg -n "resolve_and_refresh_peer|persist_items|finalize_sync|UPDATE sources SET last_sync_state|identity_refreshed_at|UPDATE telegram_sources SET avatar_cache_key" src-tauri\src\sources\sync.rs
```

Expected evidence:

```text
sync_telegram_source calls resolve_and_refresh_peer before persist_items and finalize_sync
finalize_sync updates sources.last_sync_state and sources.last_synced_at
telegram_sources.identity_refreshed_at is updated only with avatar cache refresh after a successful resolved peer path
```

Abort as `blocked` if failed resolution can still advance `sources.last_sync_state`, `sources.last_synced_at`, or `telegram_sources.identity_refreshed_at`.

- [x] **Step 3: Verify private lost-source error surface**

Run:

```powershell
rg -n "typed_peer_resolution_failure|could not be resolved from typed peer identity or dialogs|could not be resolved from dialogs|AppError::not_found|AppErrorKind" src-tauri\src\sources\peer_resolution.rs src-tauri\src\error.rs
```

Expected evidence:

```text
private channel/supergroup resolution failure maps to AppErrorKind::NotFound with an actionable message
other AppErrorKind values remain possible for Telegram runtime/RPC failures
```

Record this as expected behavior, not as a hard requirement that the live result must be `not_found`.

### Task 2: Start Runtime And Confirm Account Readiness

**Files:**
- Create: `reference/tauri-dev-lost-access.stdout.log`
- Create: `reference/tauri-dev-lost-access.stderr.log`
- Create: `reference/tauri-dev-lost-access.pid`

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
  $_.ProcessName -like '*tauri*'
} | Select-Object Id, ProcessName, Path
```

Expected: no Extractum/Tauri app process is already running. If a process exists, stop it before continuing so direct SQLite reads are stable.

- [x] **Step 3: Start the Tauri dev app**

Run with escalated permission if the sandbox blocks GUI/runtime startup:

```powershell
$stdout = Join-Path (Get-Location) 'reference\tauri-dev-lost-access.stdout.log'
$stderr = Join-Path (Get-Location) 'reference\tauri-dev-lost-access.stderr.log'
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-lost-access.pid'
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

- [x] **Step 5: Confirm account A is ready**

Tool call:

```text
mcp__tauri__.webview_execute_js({
  "script": "(async () => { return await window.__TAURI__.core.invoke('tg_get_account_statuses', { accountIds: [1] }); })()"
})
```

Expected: one status record with `account_id = 1` and `status = "ready"`. If account 1 is not ready, stop the app and document the slice as `blocked`.

### Task 3: Select Or Add A Controlled Private Source

**Files:**
- Create/Modify: `reference/telegram-lost-access-validation-context.json`
- Create: `reference/telegram-lost-access-dialog-candidates.json`
- Read: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`

- [x] **Step 1: List candidate private channel/supergroup dialogs**

Tool call:

```text
mcp__tauri__.webview_execute_js({
  "script": "(async () => {\n  const accountId = 1;\n  const sources = await window.__TAURI__.core.invoke('list_telegram_sources', { accountId });\n  return sources\n    .filter((source) => ['channel', 'supergroup'].includes(source.source_subtype) && !source.username)\n    .map((source, index) => ({\n      candidate_index: index + 1,\n      source_ref: String(source.id),\n      source_subtype: source.source_subtype,\n      is_member: source.is_member,\n      title: source.title\n    }));\n})()"
})
```

Expected: a list of private channel/supergroup dialog candidates. Runtime output may include private titles for local operator selection only; do not copy private titles into tracked docs.

- [x] **Step 2: Save the candidate list to ignored reference context**

Copy the complete JSON array returned by Task 3 Step 1 to the clipboard, then run:

```powershell
$candidateJson = Get-Clipboard -Raw
$candidatePath = Join-Path (Get-Location) 'reference\telegram-lost-access-dialog-candidates.json'
Set-Content -LiteralPath $candidatePath -Value $candidateJson -Encoding UTF8
Get-Content -Raw -LiteralPath $candidatePath | ConvertFrom-Json | Select-Object candidate_index, source_ref, source_subtype, is_member
```

Expected: sanitized candidate rows are printed without titles. If no controlled private supergroup/channel is visible, ask the operator to create or join one, then rerun Step 1.

- [x] **Step 3: Choose the controlled fixture**

Human gate:

```text
Ask the operator which candidate_index corresponds to the controlled private supergroup/channel fixture.
Prefer a private supergroup. Use a private channel if a supergroup is not available.
Do not proceed with a public username source or regular small group.
```

Expected: operator identifies one candidate index. If the operator cannot identify a controlled fixture, stop and document `blocked`.

- [x] **Step 4: Build the runtime context from the selected candidate**

When the operator provides the chosen numeric candidate index, enter it at the prompt:

```powershell
$selectedCandidateIndex = [int](Read-Host 'candidate_index')
$candidatePath = Join-Path (Get-Location) 'reference\telegram-lost-access-dialog-candidates.json'
$contextPath = Join-Path (Get-Location) 'reference\telegram-lost-access-validation-context.json'
$candidates = Get-Content -Raw -LiteralPath $candidatePath | ConvertFrom-Json
$selected = $candidates | Where-Object { $_.candidate_index -eq $selectedCandidateIndex } | Select-Object -First 1
if (-not $selected) { throw "Selected candidate not found: $selectedCandidateIndex" }
if ($selected.source_subtype -notin @('supergroup', 'channel')) { throw "Selected candidate is not channel/supergroup" }
if ($selected.is_member -ne $true) { throw "Selected candidate is not currently a member dialog" }
$context = [ordered]@{
  account_id = 1
  source_ref = [string]$selected.source_ref
  expected_subtype = [string]$selected.source_subtype
  candidate_index = [int]$selected.candidate_index
  source_id = $null
  source_preexisting = $null
  canary_message_id = $null
  post_loss_dialog_visible = $null
  post_loss_sync = $null
}
$context | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $contextPath -Encoding UTF8
Get-Content -Raw -LiteralPath $contextPath | ConvertFrom-Json | Select-Object account_id, source_ref, expected_subtype, candidate_index
```

Expected: context has `account_id = 1`, numeric `source_ref`, and `expected_subtype` of `supergroup` or `channel`.

- [x] **Step 5: Add or reuse the selected source through app IPC**

Generate the app script:

```powershell
$ctx = Get-Content -Raw reference\telegram-lost-access-validation-context.json | ConvertFrom-Json
$sourceRefJson = $ctx.source_ref | ConvertTo-Json -Compress
$expectedSubtypeJson = $ctx.expected_subtype | ConvertTo-Json -Compress
@"
(async () => {
  return await window.__TAURI__.core.invoke('add_telegram_source', {
    request: {
      accountId: $($ctx.account_id),
      sourceRef: $sourceRefJson,
      expectedSubtype: $expectedSubtypeJson
    }
  });
})()
"@
```

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set `script` to the complete JavaScript printed above.
```

Expected: a `SourceRecord` for account 1. It may create a new row or idempotently refresh/reuse an existing row. Abort as `blocked` if add fails before access loss.

- [x] **Step 6: Verify stored private typed identity**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-lost-access-validation-context.json")
context = json.loads(context_path.read_text(encoding="utf-8"))

with sqlite3.connect(db_path) as conn:
    row = conn.execute(
        """
        SELECT
          s.id,
          s.account_id,
          s.source_type,
          s.source_subtype,
          s.external_id,
          s.last_sync_state,
          s.last_synced_at,
          s.is_active,
          s.is_member,
          ts.source_subtype,
          ts.peer_kind,
          ts.peer_id,
          ts.access_hash IS NOT NULL AS access_hash_present,
          ts.username IS NOT NULL AND TRIM(ts.username) <> '' AS username_present,
          ts.resolution_strategy,
          ts.identity_refreshed_at
        FROM sources s
        JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE s.account_id = ?
          AND s.source_type = 'telegram'
          AND s.source_subtype = ?
          AND s.external_id = ?
        ORDER BY s.id ASC
        LIMIT 1
        """,
        (context["account_id"], context["expected_subtype"], context["source_ref"]),
    ).fetchone()

if row is None:
    raise SystemExit("selected source was not persisted")

columns = [
    "source_id", "account_id", "source_type", "source_subtype", "external_id",
    "last_sync_state", "last_synced_at", "is_active", "is_member",
    "typed_source_subtype", "peer_kind", "peer_id", "access_hash_present",
    "username_present", "resolution_strategy", "identity_refreshed_at",
]
record = dict(zip(columns, row))

if record["source_type"] != "telegram":
    raise SystemExit("selected source is not telegram")
if record["source_subtype"] not in ("channel", "supergroup"):
    raise SystemExit(f"selected source subtype is unsupported: {record['source_subtype']}")
if record["typed_source_subtype"] != record["source_subtype"]:
    raise SystemExit("typed source subtype mismatch")
if record["peer_kind"] != "channel":
    raise SystemExit(f"expected channel peer kind, got {record['peer_kind']}")
if not bool(record["access_hash_present"]):
    raise SystemExit("selected private channel/supergroup lacks access hash")
if bool(record["username_present"]):
    raise SystemExit("selected source has username; use a private no-username fixture")
if record["resolution_strategy"] != "dialog":
    raise SystemExit(f"selected source is not dialog-backed: {record['resolution_strategy']}")

context["source_id"] = record["source_id"]
context["source_preexisting"] = record["last_synced_at"] is not None or record["last_sync_state"] is not None
context_path.write_text(json.dumps(context, indent=2), encoding="utf-8")

print(json.dumps({
    "source_id": record["source_id"],
    "account_id": record["account_id"],
    "source_subtype": record["source_subtype"],
    "peer_kind": record["peer_kind"],
    "peer_id": record["peer_id"],
    "access_hash_present": bool(record["access_hash_present"]),
    "username_present": bool(record["username_present"]),
    "resolution_strategy": record["resolution_strategy"],
    "identity_refreshed_at": record["identity_refreshed_at"],
    "source_preexisting": context["source_preexisting"],
}, indent=2))
'@ | python -
```

Expected: `username_present = false`, `access_hash_present = true`, `resolution_strategy = dialog`, `peer_kind = channel`, and `source_subtype` is `supergroup` or `channel`. Do not copy private title values into tracked docs.

### Task 4: Baseline Sync And Snapshot Helper

**Files:**
- Create: `reference/telegram-lost-access-capture-snapshot.py`
- Create/Modify: `reference/telegram-lost-access-validation-snapshots.json`
- Create/Modify: `reference/telegram-lost-access-sync-results.json`

- [x] **Step 1: Create the snapshot helper**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3
import sys

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-lost-access-validation-context.json")
snapshots_path = Path(r"G:\Develop\Extractum\reference\telegram-lost-access-validation-snapshots.json")

if len(sys.argv) != 2:
    raise SystemExit("usage: telegram-lost-access-capture-snapshot.py <label>")

label = sys.argv[1]
allowed_labels = {"before_loss", "after_loss"}
if label not in allowed_labels:
    raise SystemExit(f"unsupported snapshot label: {label}")

context = json.loads(context_path.read_text(encoding="utf-8"))
source_id = context["source_id"]
canary_message_id = context.get("canary_message_id")

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
               identity_refreshed_at,
               created_at,
               updated_at
        FROM telegram_sources
        WHERE source_id = ?
        """,
        (source_id,),
    ).fetchone()
    item_row = conn.execute(
        """
        SELECT COUNT(*) AS item_count,
               MAX(CAST(external_id AS INTEGER)) AS max_external_id,
               MAX(created_at) AS max_created_at
        FROM items
        WHERE source_id = ?
        """,
        (source_id,),
    ).fetchone()
    canary_item_count = None
    if canary_message_id is not None:
        canary_item_count = conn.execute(
            """
            SELECT COUNT(*) AS canary_item_count
            FROM items
            WHERE source_id = ?
              AND external_id = ?
            """,
            (source_id, str(canary_message_id)),
        ).fetchone()[0]

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
    "has_access_hash", "has_username", "resolution_strategy",
    "identity_refreshed_at", "created_at", "updated_at",
]

snapshot = {
    "source": dict(zip(source_columns, source_row)),
    "telegram_source": dict(zip(telegram_columns, telegram_row)),
    "items": {
        "item_count": item_row[0],
        "max_external_id": item_row[1],
        "max_created_at": item_row[2],
        "canary_item_count": canary_item_count,
    },
}
snapshot["source"]["title_present"] = bool(snapshot["source"]["title_present"])
snapshot["source"]["is_active"] = bool(snapshot["source"]["is_active"])
snapshot["source"]["is_member"] = bool(snapshot["source"]["is_member"])
snapshot["telegram_source"]["has_access_hash"] = bool(snapshot["telegram_source"]["has_access_hash"])
snapshot["telegram_source"]["has_username"] = bool(snapshot["telegram_source"]["has_username"])

snapshots = {}
if snapshots_path.exists():
    snapshots = json.loads(snapshots_path.read_text(encoding="utf-8"))
snapshots[label] = snapshot
snapshots_path.write_text(json.dumps(snapshots, indent=2), encoding="utf-8")

print(json.dumps({label: snapshot}, indent=2))
'@ | Set-Content -LiteralPath reference\telegram-lost-access-capture-snapshot.py -Encoding UTF8
```

Expected: helper exists and captures sanitized source, typed identity, item-count, max-id, max-created-at, and optional canary presence.

- [x] **Step 2: Run baseline sync while account A still has access**

Generate the app script:

```powershell
$ctx = Get-Content -Raw reference\telegram-lost-access-validation-context.json | ConvertFrom-Json
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

Expected: `{ ok: true, result: ... }`. `inserted = 0` is acceptable for a preexisting caught-up source. If baseline sync returns `{ ok: false, ... }`, stop and document `blocked`.

- [x] **Step 3: Save the baseline sync result**

Copy the complete JSON object returned by Task 4 Step 2 to the clipboard, then run:

```powershell
$baselineJson = Get-Clipboard -Raw
$syncResultsPath = Join-Path (Get-Location) 'reference\telegram-lost-access-sync-results.json'
$syncResults = [ordered]@{
  baseline_sync = $baselineJson | ConvertFrom-Json
  post_loss_sync = $null
}
$syncResults | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $syncResultsPath -Encoding UTF8
Get-Content -Raw -LiteralPath $syncResultsPath | ConvertFrom-Json | Select-Object -ExpandProperty baseline_sync
```

Expected: saved baseline sync result with `ok = true`.

- [x] **Step 4: Capture the `before_loss` snapshot**

Run:

```powershell
python reference\telegram-lost-access-capture-snapshot.py before_loss
```

Expected: source, typed identity, and item stats are present. `has_username = false`, `has_access_hash = true`, and `resolution_strategy = dialog`.

### Task 5: Revoke Access And Capture Post-Loss Signals

**Files:**
- Modify: `reference/telegram-lost-access-validation-context.json`

- [x] **Step 1: Human gate for access revocation**

Ask the operator:

```text
Please remove account A (account id 1) from the controlled private source now, or otherwise revoke its access. After removal, optionally post one canary message from an admin account. If you know the canary Telegram message id, provide it; otherwise say "no canary id".
```

Expected: operator confirms access was revoked confidently. If access state is ambiguous enough that the sync result cannot be interpreted, stop and document `blocked`.

- [x] **Step 2: Record optional canary id**

If the operator provided a numeric canary message id, enter it at the prompt. If not, press Enter.

```powershell
$rawCanaryMessageId = Read-Host 'canary_message_id_or_blank'
$canaryMessageId = if ([string]::IsNullOrWhiteSpace($rawCanaryMessageId)) { $null } else { [int64]$rawCanaryMessageId }
$contextPath = Join-Path (Get-Location) 'reference\telegram-lost-access-validation-context.json'
$ctx = Get-Content -Raw -LiteralPath $contextPath | ConvertFrom-Json
$ctx.canary_message_id = $canaryMessageId
$ctx | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $contextPath -Encoding UTF8
Get-Content -Raw -LiteralPath $contextPath | ConvertFrom-Json | Select-Object source_id, canary_message_id
```

Expected: context records either a numeric canary id or `null`.

- [x] **Step 3: Check post-loss dialog visibility**

Generate the app script:

```powershell
$ctx = Get-Content -Raw reference\telegram-lost-access-validation-context.json | ConvertFrom-Json
@"
(async () => {
  const sources = await window.__TAURI__.core.invoke('list_telegram_sources', {
    accountId: $($ctx.account_id)
  });
  const target = sources.find((source) =>
    String(source.id) === "$($ctx.source_ref)" &&
    source.source_subtype === "$($ctx.expected_subtype)"
  );
  return {
    visible: Boolean(target),
    source_ref: "$($ctx.source_ref)",
    expected_subtype: "$($ctx.expected_subtype)",
    visible_subtype: target ? target.source_subtype : null,
    visible_is_member: target ? target.is_member : null,
    visible_username_present: target ? Boolean(target.username) : false
  };
})()
"@
```

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set `script` to the complete JavaScript printed above.
```

Expected: sanitized visibility object. `visible = false` is the clearest lost-access signal. `visible = true` does not automatically block; use operator confidence and sync result to classify.

- [x] **Step 4: Save post-loss dialog visibility**

Copy the JSON object returned by Task 5 Step 3 to the clipboard, then run:

```powershell
$visibilityJson = Get-Clipboard -Raw
$contextPath = Join-Path (Get-Location) 'reference\telegram-lost-access-validation-context.json'
$ctx = Get-Content -Raw -LiteralPath $contextPath | ConvertFrom-Json
$visibility = $visibilityJson | ConvertFrom-Json
$ctx.post_loss_dialog_visible = [bool]$visibility.visible
$ctx.post_loss_dialog_visibility = $visibility
$ctx | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $contextPath -Encoding UTF8
Get-Content -Raw -LiteralPath $contextPath | ConvertFrom-Json | Select-Object source_id, post_loss_dialog_visible
```

Expected: context records the sanitized visibility signal.

### Task 6: Run Post-Loss Sync And Capture After Snapshot

**Files:**
- Modify: `reference/telegram-lost-access-sync-results.json`
- Modify: `reference/telegram-lost-access-validation-snapshots.json`

- [x] **Step 1: Run post-loss sync**

Generate the app script:

```powershell
$ctx = Get-Content -Raw reference\telegram-lost-access-validation-context.json | ConvertFrom-Json
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

Expected: usually `{ ok: false, error: { kind: ..., message: ... } }` with a typed, user-actionable error. `{ ok: true, ... }` is not automatically a pass; evaluate item/canary and identity invariants.

- [x] **Step 2: Save the post-loss sync result**

Copy the complete JSON object returned by Task 6 Step 1 to the clipboard, then run:

```powershell
$postLossJson = Get-Clipboard -Raw
$syncResultsPath = Join-Path (Get-Location) 'reference\telegram-lost-access-sync-results.json'
$syncResults = Get-Content -Raw -LiteralPath $syncResultsPath | ConvertFrom-Json
$syncResults.post_loss_sync = $postLossJson | ConvertFrom-Json
$syncResults | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $syncResultsPath -Encoding UTF8
Get-Content -Raw -LiteralPath $syncResultsPath | ConvertFrom-Json | Select-Object -ExpandProperty post_loss_sync
```

Expected: sync result is saved with either `ok = false` and typed error details or `ok = true` and a `SyncResult`.

- [x] **Step 3: Capture the `after_loss` snapshot**

Run:

```powershell
python reference\telegram-lost-access-capture-snapshot.py after_loss
```

Expected: snapshot includes the same source id and typed identity row.

### Task 7: Evaluate Invariants

**Files:**
- Read: `reference/telegram-lost-access-validation-context.json`
- Read: `reference/telegram-lost-access-validation-snapshots.json`
- Read: `reference/telegram-lost-access-sync-results.json`
- Create: `reference/telegram-lost-access-validation-evaluation.json`

- [ ] **Step 1: Create and run the evaluation script**

Run:

```powershell
@'
from pathlib import Path
import json

context_path = Path(r"G:\Develop\Extractum\reference\telegram-lost-access-validation-context.json")
snapshots_path = Path(r"G:\Develop\Extractum\reference\telegram-lost-access-validation-snapshots.json")
sync_results_path = Path(r"G:\Develop\Extractum\reference\telegram-lost-access-sync-results.json")
evaluation_path = Path(r"G:\Develop\Extractum\reference\telegram-lost-access-validation-evaluation.json")

context = json.loads(context_path.read_text(encoding="utf-8"))
snapshots = json.loads(snapshots_path.read_text(encoding="utf-8"))
sync_results = json.loads(sync_results_path.read_text(encoding="utf-8"))

for label in ["before_loss", "after_loss"]:
    if label not in snapshots:
        raise SystemExit(f"missing snapshot: {label}")
if not sync_results.get("baseline_sync"):
    raise SystemExit("missing baseline sync result")
if not sync_results.get("post_loss_sync"):
    raise SystemExit("missing post-loss sync result")

before = snapshots["before_loss"]
after = snapshots["after_loss"]
post_loss_sync = sync_results["post_loss_sync"]

identity_fields = [
    "source_subtype",
    "peer_kind",
    "peer_id",
    "has_access_hash",
    "has_username",
    "resolution_strategy",
    "identity_refreshed_at",
]
source_state_fields = ["last_sync_state", "last_synced_at"]

identity_unchanged = all(
    before["telegram_source"].get(field) == after["telegram_source"].get(field)
    for field in identity_fields
)
source_state_not_advanced = all(
    before["source"].get(field) == after["source"].get(field)
    for field in source_state_fields
)
item_count_unchanged = before["items"]["item_count"] == after["items"]["item_count"]
source_row_explainable = after["source"]["id"] == before["source"]["id"] and after["source"]["title_present"]
is_member_explainable = (
    before["source"]["is_member"] == after["source"]["is_member"]
    or after["source"]["is_member"] is False
)
canary_id = context.get("canary_message_id")
canary_absent = True
if canary_id is not None:
    canary_absent = after["items"].get("canary_item_count") == 0
coarse_no_post_loss_growth = (
    before["items"].get("max_external_id") == after["items"].get("max_external_id")
    and before["items"].get("max_created_at") == after["items"].get("max_created_at")
)

post_loss_ok = bool(post_loss_sync.get("ok"))
typed_error_kind = None
typed_error_message = None
typed_error_explainable = False
if not post_loss_ok:
    error = post_loss_sync.get("error") or {}
    typed_error_kind = error.get("kind")
    typed_error_message = error.get("message")
    typed_error_explainable = bool(typed_error_kind) and typed_error_kind != "internal" and bool(typed_error_message)

wrong_peer_evidence_absent = identity_unchanged and item_count_unchanged and canary_absent
failed_sync_pass = (
    not post_loss_ok
    and typed_error_explainable
    and identity_unchanged
    and source_state_not_advanced
    and item_count_unchanged
    and canary_absent
    and source_row_explainable
    and is_member_explainable
)
successful_sync_needs_follow_up = (
    post_loss_ok
    and identity_unchanged
    and item_count_unchanged
    and canary_absent
    and source_row_explainable
)
failed = (
    not identity_unchanged
    or not source_state_not_advanced
    or not item_count_unchanged
    or not canary_absent
    or not source_row_explainable
    or (not post_loss_ok and not typed_error_explainable)
)

if failed_sync_pass:
    classification = "passed"
elif successful_sync_needs_follow_up:
    classification = "needs_follow_up"
elif failed:
    classification = "failed"
else:
    classification = "needs_follow_up"

evaluation = {
    "classification": classification,
    "source_id": context["source_id"],
    "source_subtype": before["telegram_source"]["source_subtype"],
    "peer_kind": before["telegram_source"]["peer_kind"],
    "peer_id": before["telegram_source"]["peer_id"],
    "post_loss_dialog_visible": context.get("post_loss_dialog_visible"),
    "post_loss_sync_ok": post_loss_ok,
    "typed_error_kind": typed_error_kind,
    "typed_error_message": typed_error_message,
    "typed_error_explainable": typed_error_explainable,
    "identity_unchanged": identity_unchanged,
    "source_state_not_advanced": source_state_not_advanced,
    "item_count_unchanged": item_count_unchanged,
    "canary_id_recorded": canary_id is not None,
    "canary_absent": canary_absent,
    "coarse_no_post_loss_growth": coarse_no_post_loss_growth,
    "source_row_explainable": source_row_explainable,
    "is_member_explainable": is_member_explainable,
    "wrong_peer_evidence_absent": wrong_peer_evidence_absent,
    "before_item_count": before["items"]["item_count"],
    "after_item_count": after["items"]["item_count"],
    "before_last_sync_state": before["source"]["last_sync_state"],
    "after_last_sync_state": after["source"]["last_sync_state"],
    "before_last_synced_at": before["source"]["last_synced_at"],
    "after_last_synced_at": after["source"]["last_synced_at"],
    "before_identity_refreshed_at": before["telegram_source"]["identity_refreshed_at"],
    "after_identity_refreshed_at": after["telegram_source"]["identity_refreshed_at"],
}

evaluation_path.write_text(json.dumps(evaluation, indent=2), encoding="utf-8")
print(json.dumps(evaluation, indent=2))
'@ | python -
```

Expected:

- `classification = "passed"` for a typed failed post-loss sync with all invariants true.
- `classification = "needs_follow_up"` for a successful post-loss sync with no wrong-peer evidence.
- `classification = "failed"` for identity mutation, source-state advancement after failed sync, item-count growth, canary ingestion, untyped/internal error, or unexplained source disappearance.

### Task 8: Stop Runtime Processes

**Files:**
- Read: `reference/tauri-dev-lost-access.pid`

- [ ] **Step 1: Stop the Tauri MCP session**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "stop" })
```

Expected: all bridge sessions are stopped.

- [ ] **Step 2: Stop the Tauri dev process tree**

Run:

```powershell
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-lost-access.pid'
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

Expected: the dev process tree exits. If access is denied, rerun the stop command with escalated permission.

- [ ] **Step 3: Confirm no runtime processes remain**

Run:

```powershell
Get-Process | Where-Object {
  $_.ProcessName -like '*extractum*' -or
  $_.ProcessName -like '*tauri*' -or
  $_.ProcessName -eq 'cargo'
} | Select-Object Id, ProcessName, Path
```

Expected: no leftover Extractum, Tauri, or Cargo process from this live run.

### Task 9: Document The Live Result

**Files:**
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Read sanitized runtime evidence**

Run:

```powershell
Get-Content -Raw reference\telegram-lost-access-validation-evaluation.json | ConvertFrom-Json
Get-Content -Raw reference\telegram-lost-access-validation-snapshots.json | ConvertFrom-Json
Get-Content -Raw reference\telegram-lost-access-sync-results.json | ConvertFrom-Json
```

Expected: evidence is available for classification and docs. Do not copy private titles or message text into tracked docs.

- [ ] **Step 2: Update the verification matrix row**

If `classification = "passed"`, update the row to this shape with observed values:

```markdown
| No-longer-member, left, or private access lost | passed | access-limited | Controlled private channel/supergroup was added or reused from dialogs before access was revoked | Stored identity stayed on the same `source_subtype`, `peer_kind`, `peer_id`, access-hash presence, username absence, and `resolution_strategy = dialog` | Baseline sync succeeded; post-loss `sync_source(source_id)` returned a typed, explainable access/lost-source error | Snapshots showed no identity mutation, no sync-state advancement, no item-count growth, and no canary/post-removal ingest | Record typed error kind/message sanitized |
```

If `classification = "needs_follow_up"` or `failed`, set that status and describe the non-sensitive reason in the row.

- [ ] **Step 3: Add the dated live-run note**

Append a section to `docs/superpowers/verification/telegram-runtime-private-source-validation.md`:

```markdown
## 2026-05-22 Lost Access Follow-Up

- Account label: account `1`; no credentials, phone numbers, session data,
  private message content, private titles, private usernames, public username
  values, or access-hash values recorded.
- App commit: the exact `git log -1 --oneline` value captured in Task 1.
- Fixture: controlled private `channel` or `supergroup` selected from dialogs;
  username absent, access-hash present, `resolution_strategy = dialog`.
- Source row: source id, `account_id = 1`, observed `source_subtype`,
  `peer_kind = channel`, observed `peer_id`, and title present by boolean only.
- Baseline sync: record `inserted`, `skipped`, `last_message_id`, initial policy,
  and warnings.
- Access revocation: operator confirmed account `1` access was revoked. Record
  post-loss dialog visibility as present/absent/ambiguous without private label.
- Canary: record whether a canary id was captured and whether it appeared in
  local items. Do not record canary text.
- Post-loss sync: record `ok`, typed error kind and sanitized message, or
  `SyncResult` if it unexpectedly succeeded.
- Snapshots: record before/after `last_sync_state`, `last_synced_at`,
  `identity_refreshed_at`, `is_member`, item count, max item id, and max created
  timestamp. Do not record private title.
- Result: record `passed`, `needs follow-up`, `failed`, or `blocked` and the
  specific invariant evidence.
```

Use exact observed values from ignored runtime JSON. Keep the note concise and sanitized.

- [ ] **Step 4: Update backlog according to classification**

If `classification = "passed"`, remove this checklist row from `docs/backlog.md` section `3.1`:

```markdown
- [ ] verify behavior when the user is no longer a member of a group or channel
```

Update the `Priority Snapshot` Telegram row to mention only migrated-dialog behavior if it currently names lost-access. Update recent evidence to mention the 2026-05-22 lost-access probe. Leave this row open:

```markdown
- [ ] verify behavior for migrated small-group-to-supergroup dialogs
```

If classification is `blocked`, `needs_follow_up`, or `failed`, keep or replace the backlog item with a concrete non-sensitive follow-up.

### Task 10: Verification And Commit

**Files:**
- Verify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- Verify: `docs/backlog.md`

- [ ] **Step 1: Check tracked diff scope**

Run:

```powershell
git diff --stat
git diff --name-only
```

Expected tracked files for a completed live result:

```text
docs/backlog.md
docs/superpowers/verification/telegram-runtime-private-source-validation.md
```

Runtime files under `reference/` must not appear in tracked diff.

- [ ] **Step 2: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: exit code 0. Known LF/CRLF warnings are acceptable if no whitespace errors are reported.

- [ ] **Step 3: Commit the live validation result**

If the probe passed:

```powershell
git add docs\backlog.md docs\superpowers\verification\telegram-runtime-private-source-validation.md
git commit -m "docs: record telegram lost-access validation"
```

If the probe is blocked, failed, or inconclusive:

```powershell
git add docs\backlog.md docs\superpowers\verification\telegram-runtime-private-source-validation.md
git commit -m "docs: record telegram lost-access validation attempt"
```

- [ ] **Step 4: Verify the commit**

Run:

```powershell
git diff --check HEAD~1..HEAD
git status --short --branch
git log -1 --oneline
```

Expected: `git diff --check HEAD~1..HEAD` exits 0, status is clean on `main`, and the latest commit message matches the result.
