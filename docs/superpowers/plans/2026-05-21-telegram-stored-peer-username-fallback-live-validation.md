# Telegram Stored Peer Username Fallback Live Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate that a live public Telegram source with a usable stored peer identity can sync when its cached username is locally unusable.

**Architecture:** Perform a one-source DB-only probe against account 1 and source `18`. Back up the SQLite database, replace only the cached typed username with a sentinel, run `sync_source(18)` through the normal Tauri IPC path, verify the sentinel survived the successful sync, restore from the backup value, and document the result.

**Tech Stack:** PowerShell, Python `sqlite3`, SQLite, Tauri dev app and MCP bridge, Markdown verification docs.

---

## File Structure

- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
  - Record the stored-peer username fallback probe and its limitation.
- Modify: `docs/backlog.md`
  - Remove the stored-peer username fallback checklist row only if the probe passes.
- Create runtime-only ignored files under `reference/`
  - `extractum-live-db-backup-source18-stored-peer-username-fallback-20260521.db`
  - `tauri-dev-stored-peer-fallback.stdout.log`
  - `tauri-dev-stored-peer-fallback.stderr.log`
  - `tauri-dev-stored-peer-fallback.pid`

Do not modify Rust, Svelte, schema, or app code unless the live probe exposes a bug. If code changes become necessary, stop and create a RED test first.

### Task 1: Pre-Flight State And DB Backup

**Files:**
- Read: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`
- Create: `reference/extractum-live-db-backup-source18-stored-peer-username-fallback-20260521.db`

- [ ] **Step 1: Confirm tracked workspace state**

Run:

```powershell
git status --short --branch
```

Expected: clean `## main` before the live probe begins. If the only changes are this plan commit, commit or handle them before continuing.

- [ ] **Step 2: Stop any stale Tauri MCP session**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "stop" })
```

Expected: no active bridge sessions remain.

- [ ] **Step 3: Confirm the app is not already running**

Run:

```powershell
Get-Process | Where-Object {
  $_.ProcessName -like '*extractum*' -or
  $_.ProcessName -like '*tauri*'
} | Select-Object Id, ProcessName, Path
```

Expected: no running Extractum/Tauri app process. If a process exists, stop it before direct DB edits.

- [ ] **Step 4: Back up the DB and apply the sentinel username**

Run from repo root with elevated filesystem permission, because it writes to the app data SQLite database outside the workspace:

```powershell
@'
from pathlib import Path
import json
import shutil
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
backup_path = Path(r"G:\Develop\Extractum\reference\extractum-live-db-backup-source18-stored-peer-username-fallback-20260521.db")
source_id = 18
sentinel = "extractum_validation_missing_username_20260521"

if not db_path.exists():
    raise SystemExit(f"DB not found: {db_path}")
if backup_path.exists():
    raise SystemExit(f"Backup already exists: {backup_path}")

columns = [
    "source_id",
    "account_id",
    "source_subtype",
    "peer_kind",
    "peer_id",
    "resolution_strategy",
    "username",
    "access_hash",
    "last_sync_state",
    "last_synced_at",
]

with sqlite3.connect(db_path) as conn:
    row = conn.execute(
        """
        SELECT
          ts.source_id,
          ts.account_id,
          ts.source_subtype,
          ts.peer_kind,
          ts.peer_id,
          ts.resolution_strategy,
          ts.username,
          ts.access_hash,
          s.last_sync_state,
          s.last_synced_at
        FROM telegram_sources ts
        JOIN sources s ON s.id = ts.source_id
        WHERE ts.source_id = ?
        """,
        (source_id,),
    ).fetchone()

if row is None:
    raise SystemExit(f"source_id {source_id} not found")

record = dict(zip(columns, row))
if record["account_id"] != 1:
    raise SystemExit(f"abort: account_id is {record['account_id']}, expected 1")
if record["source_subtype"] not in ("channel", "supergroup"):
    raise SystemExit(f"abort: source_subtype is {record['source_subtype']}")
if record["peer_kind"] != "channel":
    raise SystemExit(f"abort: peer_kind is {record['peer_kind']}")
if not record["username"]:
    raise SystemExit("abort: username is absent")
if record["access_hash"] is None:
    raise SystemExit("abort: access_hash is absent")

backup_path.parent.mkdir(parents=True, exist_ok=True)
shutil.copy2(db_path, backup_path)

with sqlite3.connect(db_path) as conn:
    conn.execute(
        "UPDATE telegram_sources SET username = ? WHERE source_id = ?",
        (sentinel, source_id),
    )
    conn.commit()
    updated = conn.execute(
        """
        SELECT username, peer_kind, peer_id, access_hash, resolution_strategy
        FROM telegram_sources
        WHERE source_id = ?
        """,
        (source_id,),
    ).fetchone()

if updated is None:
    raise SystemExit("abort: source disappeared after sentinel update")

print(json.dumps({
    "source_id": record["source_id"],
    "account_id": record["account_id"],
    "source_subtype": record["source_subtype"],
    "peer_kind": record["peer_kind"],
    "peer_id": record["peer_id"],
    "resolution_strategy": record["resolution_strategy"],
    "username_present_before_probe": bool(record["username"]),
    "access_hash_present": record["access_hash"] is not None,
    "last_sync_state_before_probe": record["last_sync_state"],
    "last_synced_at_before_probe": record["last_synced_at"],
    "sentinel_applied": updated[0] == sentinel,
    "backup_path": str(backup_path),
}, indent=2))
'@ | python -
```

Expected: JSON with `source_id = 18`, `account_id = 1`, `source_subtype = channel`, `peer_kind = channel`, `username_present_before_probe = true`, `access_hash_present = true`, and `sentinel_applied = true`. Abort if any condition fails.

### Task 2: Run Live Sync Probe

**Files:**
- Runtime logs under `reference/`

- [ ] **Step 1: Start the Tauri dev app**

Run with elevated permission because this launches a GUI app and uses the live app data directory:

```powershell
$stdout = Join-Path (Get-Location) 'reference\tauri-dev-stored-peer-fallback.stdout.log'
$stderr = Join-Path (Get-Location) 'reference\tauri-dev-stored-peer-fallback.stderr.log'
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-stored-peer-fallback.pid'
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

Expected: a process id is printed.

- [ ] **Step 2: Connect to the Tauri MCP bridge**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "start", "port": 9223 })
```

Expected: session connects to the running app.

- [ ] **Step 3: Confirm account 1 is ready**

Tool call:

```text
mcp__tauri__.ipc_execute_command({ "command": "tg_get_account_statuses" })
```

Expected: account 1 status is `ready`. If account 1 is not ready, stop the app, restore the username, and mark the probe blocked.

- [ ] **Step 4: Run the sync probe**

Tool call:

```text
mcp__tauri__.ipc_execute_command({ "command": "sync_source", "args": "{\"sourceId\":18}" })
```

Expected: command succeeds and returns a `SyncResult` with no warnings. Record `inserted`, `skipped`, and `last_message_id`.

- [ ] **Step 5: Check whether the sentinel survived the successful sync**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
source_id = 18
sentinel = "extractum_validation_missing_username_20260521"

with sqlite3.connect(db_path) as conn:
    row = conn.execute(
        """
        SELECT username, peer_kind, peer_id, access_hash, resolution_strategy
        FROM telegram_sources
        WHERE source_id = ?
        """,
        (source_id,),
    ).fetchone()

if row is None:
    raise SystemExit("source row missing after sync")

print(json.dumps({
    "source_id": source_id,
    "sentinel_still_present": row[0] == sentinel,
    "peer_kind": row[1],
    "peer_id": row[2],
    "access_hash_present": row[3] is not None,
    "resolution_strategy": row[4],
}, indent=2))
'@ | python -
```

Expected: `sentinel_still_present = true`. If it is false, continue to restore the DB, but record the validation outcome as `needs follow-up`.

- [ ] **Step 6: Stop the Tauri dev app**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "stop" })
```

Then run:

```powershell
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-stored-peer-fallback.pid'
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

Expected: the app started for the sync probe is no longer running.

### Task 3: Restore Typed Username

**Files:**
- Read: `reference/extractum-live-db-backup-source18-stored-peer-username-fallback-20260521.db`
- Modify: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`

- [ ] **Step 1: Restore the original username from the backup DB**

Run with elevated filesystem permission, because it writes to the app data SQLite database outside the workspace:

```powershell
@'
from pathlib import Path
import json
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
backup_path = Path(r"G:\Develop\Extractum\reference\extractum-live-db-backup-source18-stored-peer-username-fallback-20260521.db")
source_id = 18

with sqlite3.connect(backup_path) as backup:
    original = backup.execute(
        """
        SELECT username, peer_kind, peer_id, access_hash, resolution_strategy
        FROM telegram_sources
        WHERE source_id = ?
        """,
        (source_id,),
    ).fetchone()

if original is None:
    raise SystemExit("backup source row missing")

with sqlite3.connect(db_path) as conn:
    conn.execute(
        "UPDATE telegram_sources SET username = ? WHERE source_id = ?",
        (original[0], source_id),
    )
    conn.commit()
    restored = conn.execute(
        """
        SELECT username, peer_kind, peer_id, access_hash, resolution_strategy
        FROM telegram_sources
        WHERE source_id = ?
        """,
        (source_id,),
    ).fetchone()

if restored is None:
    raise SystemExit("restored source row missing")

print(json.dumps({
    "source_id": source_id,
    "username_restored": restored[0] == original[0],
    "peer_kind_unchanged": restored[1] == original[1],
    "peer_id_unchanged": restored[2] == original[2],
    "access_hash_unchanged": restored[3] == original[3],
    "resolution_strategy_unchanged": restored[4] == original[4],
}, indent=2))
'@ | python -
```

Expected: all fields print `true`. If any field is false, restore the whole DB file from the backup and stop before documenting success.

### Task 4: Document The Probe

**Files:**
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Update validation matrix**

If `sync_source(18)` succeeded and `sentinel_still_present = true`, add a matrix row after `Public channel`:

```markdown
| Stored peer before username fallback on public channel | passed | public username | Source `18` was probed with a local sentinel username only; original username value was not recorded | `peer_kind = channel`, `access_hash` present, original username present, sentinel `extractum_validation_missing_username_20260521` survived the successful sync probe, `resolution_strategy = dialog` | `sync_source(18)` result from the live run | Stored peer identity was sufficient while cached username was unusable; strict resolver order remains covered by backend tests | No warnings; no real Telegram username reassignment was performed |
```

If the sentinel did not survive, add the same row with `needs follow-up` and explain that runtime identity refresh may have restored the true username before history resolution.

- [ ] **Step 2: Add a live run note**

Append a section:

```markdown
## 2026-05-21 Stored Peer Username Fallback Probe

- Account label: account 1 only; no credentials, phone numbers, session data,
  private message content, or original username value recorded.
- App commit: the output of `git log -1 --oneline` captured immediately before
  the sync run.
- Probe source: source `18`, public channel, original username present,
  `peer_kind = channel`, `access_hash` present, `resolution_strategy = dialog`.
- Local probe changed only `telegram_sources.username` to
  `extractum_validation_missing_username_20260521`.
- `sync_source(18)` returned the observed `inserted`, `skipped`,
  `last_message_id`, and `warnings` values from Task 2 Step 4.
- Post-sync check before restore records the observed
  `sentinel_still_present` value from Task 2 Step 5.
- Post-restore check: original username restored; `peer_kind`, `peer_id`,
  `access_hash`, and `resolution_strategy` unchanged.
- This run did not perform a real Telegram username reassignment. It
  temporarily corrupted the local cached username only. The live evidence proves
  that a usable stored peer identity is sufficient for sync when the cached
  username is unusable. The strict resolver order is covered by backend resolver
  tests.
```

Replace the angle-bracket text with observed values before saving.

- [ ] **Step 3: Update backlog**

If the probe passed, remove this row from section `3.1`:

```markdown
- [ ] validate stored-peer resolution before username fallback on a live public source with `access_hash`
```

Update recent evidence to mention the stored-peer username fallback probe. Leave the cross-account, lost-access, and migrated rows.

If the probe is `needs follow-up`, keep the backlog row and add a recent evidence note explaining why the live result was inconclusive.

### Task 5: Verification And Commit

**Files:**
- Verify all tracked docs.

- [ ] **Step 1: Check docs-only diff**

Run:

```powershell
git diff --stat
git diff --name-only
```

Expected tracked files: only `docs/superpowers/verification/telegram-runtime-private-source-validation.md` and `docs/backlog.md` for the live validation result. The already committed spec and plan should not appear.

- [ ] **Step 2: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: exit 0, allowing only known LF/CRLF warnings if Git prints them.

- [ ] **Step 3: Commit docs result**

If the probe passed:

```powershell
git add docs\superpowers\verification\telegram-runtime-private-source-validation.md docs\backlog.md
git commit -m "docs: record telegram stored peer fallback validation"
```

If the probe needs follow-up:

```powershell
git add docs\superpowers\verification\telegram-runtime-private-source-validation.md docs\backlog.md
git commit -m "docs: record inconclusive telegram fallback probe"
```
