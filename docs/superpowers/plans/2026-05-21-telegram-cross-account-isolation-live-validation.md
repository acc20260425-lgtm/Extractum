# Telegram Cross Account Isolation Live Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate that the same real public Telegram peer can be added and synced under two different local Telegram accounts without runtime session or SQLite state crossing account boundaries.

**Architecture:** Run a docs-only live validation slice. First audit the source-to-session dispatch path in code, then use the normal Tauri app IPC path to ensure a shared public channel or supergroup exists under account 1 and the second ready account, snapshot source state and item counts before and after each sync, and document the evidence.

**Tech Stack:** PowerShell, Python `sqlite3`, SQLite, Tauri dev app and MCP bridge, Markdown verification docs.

---

## File Structure

- Read: `docs/superpowers/specs/2026-05-21-telegram-cross-account-isolation-live-validation-design.md`
  - Approved design and pass criteria for this slice.
- Read: `src-tauri/src/sources/sync.rs`
  - Verify `sync_source -> sync_telegram_source` loads the source row and dispatches through the source account id.
- Read: `src-tauri/src/telegram.rs`
  - Verify `get_authorized_runtime` and `init_account_client` use the requested `account_id`.
- Read: `src-tauri/src/telegram_session_store.rs`
  - Verify account-specific session loading through `load_session(account_id)`.
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
  - Mark the `Same source on account A and account B` matrix row according to the live result and add a dated run note.
- Modify: `docs/backlog.md`
  - Remove the cross-account 3.1 row only if the live validation passes.
- Create runtime-only ignored files under `reference/`
  - `telegram-cross-account-validation-context.json`
  - `telegram-cross-account-capture-snapshot.py`
  - `telegram-cross-account-validation-snapshots.json`
  - `telegram-cross-account-validation-evaluation.json`
  - `tauri-dev-cross-account.stdout.log`
  - `tauri-dev-cross-account.stderr.log`
  - `tauri-dev-cross-account.pid`

Do not modify Rust, Svelte, schema, or app code during this slice. If code changes become necessary, stop and switch to a RED test first.

### Task 1: Session Dispatch Audit

**Files:**
- Read: `src-tauri/src/sources/sync.rs`
- Read: `src-tauri/src/telegram.rs`
- Read: `src-tauri/src/telegram_session_store.rs`

- [ ] **Step 1: Confirm tracked workspace state**

Run:

```powershell
git status --short --branch
```

Expected: clean `## main` before live validation begins.

- [ ] **Step 2: Inspect the sync dispatch entry point**

Run:

```powershell
rg -n "pub async fn sync_source|load_source\\(&pool, source_id\\)|source.account_id|get_authorized_runtime\\(&state, account_id\\)" src-tauri\src\sources\sync.rs
```

Expected evidence:

```text
sync_source loads source_id through load_source
sync_telegram_source reads source.account_id
sync_telegram_source passes account_id into get_authorized_runtime
```

Abort as `blocked` if `sync_source` can reach a Telegram client without using the source row's `account_id`.

- [ ] **Step 3: Inspect runtime lookup by account id**

Run:

```powershell
rg -n "struct TelegramState|accounts: Mutex<HashMap<i64, AccountClient>>|pub\\(crate\\) async fn get_authorized_runtime|accounts\\.get\\(&account_id\\)|client: account.client.clone\\(\\)" src-tauri\src\telegram.rs
```

Expected evidence:

```text
TelegramState stores clients in HashMap<i64, AccountClient>
get_authorized_runtime reads accounts.get(&account_id)
get_authorized_runtime clones the selected account's Client
```

Abort as `blocked` if there is a global/default Telegram client fallback in the sync path.

- [ ] **Step 4: Inspect account-specific session loading**

Run:

```powershell
rg -n "init_account_client|telegram_session_store::load_session\\(handle, secret_store, account_id\\)|pub\\(crate\\) fn session_path|telegram_\\{account_id\\}\\.session\\.json|associated_data\\(account_id\\)" src-tauri\src\telegram.rs src-tauri\src\telegram_session_store.rs
```

Expected evidence:

```text
init_account_client loads session with account_id
session_path uses telegram_{account_id}.session.json
encrypted session associated data includes account_id
```

Abort as `blocked` if account initialization can reuse another account's session without an account-id check.

- [ ] **Step 5: Save the audit conclusion for documentation**

Record this conclusion for the final verification note if Steps 2-4 match expected evidence:

```text
Manual session-dispatch audit at the validation commit checked
src-tauri/src/sources/sync.rs (`sync_source -> sync_telegram_source`),
src-tauri/src/telegram.rs (`get_authorized_runtime`, `init_account_client`),
and src-tauri/src/telegram_session_store.rs (`load_session(account_id)`).
The sync path loads the source row, uses that row's `account_id` to select
`TelegramState.accounts[account_id]`, and initializes each grammers client from
that account's session file.
```

### Task 2: Build Runtime Context

**Files:**
- Read: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`
- Create: `reference/telegram-cross-account-validation-context.json`

- [ ] **Step 1: Stop stale MCP sessions**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "stop" })
```

Expected: all previous bridge sessions are stopped or there were no active sessions.

- [ ] **Step 2: Confirm the app is not already running**

Run:

```powershell
Get-Process | Where-Object {
  $_.ProcessName -like '*extractum*' -or
  $_.ProcessName -like '*tauri*'
} | Select-Object Id, ProcessName, Path
```

Expected: no Extractum/Tauri app process is already running. If a process exists, stop it before continuing so direct SQLite reads are stable.

- [ ] **Step 3: Select account B and a safe account 1 public source**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-cross-account-validation-context.json")

if not db_path.exists():
    raise SystemExit(f"DB not found: {db_path}")

with sqlite3.connect(db_path) as conn:
    accounts = [row[0] for row in conn.execute(
        "SELECT id FROM accounts ORDER BY created_at ASC, id ASC"
    ).fetchall()]

    if 1 not in accounts:
        raise SystemExit("account 1 is missing")

    account_b_candidates = [account_id for account_id in accounts if account_id != 1]
    if not account_b_candidates:
        raise SystemExit("no second Telegram account row found")

    account_b_id = account_b_candidates[0]

    candidate = conn.execute(
        """
        SELECT
          ts.source_id,
          ts.account_id,
          s.source_type,
          ts.source_subtype,
          ts.peer_kind,
          ts.peer_id,
          ts.username,
          ts.access_hash,
          ts.resolution_strategy,
          s.last_sync_state,
          s.last_synced_at
        FROM telegram_sources ts
        JOIN sources s ON s.id = ts.source_id
        WHERE ts.account_id = 1
          AND s.source_type = 'telegram'
          AND ts.source_subtype IN ('channel', 'supergroup')
          AND ts.peer_kind = 'channel'
          AND ts.username IS NOT NULL
          AND TRIM(ts.username) <> ''
          AND ts.access_hash IS NOT NULL
        ORDER BY
          CASE ts.source_id WHEN 18 THEN 0 WHEN 17 THEN 1 ELSE 2 END,
          COALESCE(s.last_synced_at, 0) DESC,
          ts.source_id ASC
        LIMIT 1
        """
    ).fetchone()

    if candidate is None:
        raise SystemExit("no account 1 public channel/supergroup source with username and access_hash found")

    (
        source_a_id,
        account_a_id,
        source_type,
        source_subtype,
        peer_kind,
        peer_id,
        username,
        access_hash,
        resolution_strategy,
        last_sync_state,
        last_synced_at,
    ) = candidate

    existing_b = conn.execute(
        """
        SELECT source_id
        FROM telegram_sources
        WHERE account_id = ?
          AND peer_kind = ?
          AND peer_id = ?
        ORDER BY source_id ASC
        LIMIT 1
        """,
        (account_b_id, peer_kind, peer_id),
    ).fetchone()

source_ref = username if str(username).startswith("@") else f"@{username}"

context = {
    "account_a_id": account_a_id,
    "account_b_id": account_b_id,
    "source_a_id": source_a_id,
    "source_b_id": existing_b[0] if existing_b else None,
    "source_b_preexisting": existing_b is not None,
    "source_type": source_type,
    "expected_subtype": source_subtype,
    "peer_kind": peer_kind,
    "peer_id": peer_id,
    "source_ref": source_ref,
    "username_present": bool(username),
    "access_hash_present_a": access_hash is not None,
    "resolution_strategy_a": resolution_strategy,
    "last_sync_state_a_before_context": last_sync_state,
    "last_synced_at_a_before_context": last_synced_at,
}

context_path.parent.mkdir(parents=True, exist_ok=True)
context_path.write_text(json.dumps(context, indent=2), encoding="utf-8")

print(json.dumps({
    "account_a_id": account_a_id,
    "account_b_id": account_b_id,
    "source_a_id": source_a_id,
    "source_b_preexisting": existing_b is not None,
    "source_b_id": existing_b[0] if existing_b else None,
    "source_type": source_type,
    "expected_subtype": source_subtype,
    "peer_kind": peer_kind,
    "peer_id": peer_id,
    "username_present": bool(username),
    "access_hash_present_a": access_hash is not None,
    "resolution_strategy_a": resolution_strategy,
    "context_path": str(context_path),
}, indent=2))
'@ | python -
```

Expected: sanitized JSON with `account_a_id = 1`, a non-1 `account_b_id`, a public `channel` or `supergroup`, `peer_kind = channel`, `username_present = true`, and `access_hash_present_a = true`. The ignored context file contains the public `source_ref`; do not copy that value into tracked docs.

### Task 3: Start The App And Confirm Runtime Readiness

**Files:**
- Create: `reference/tauri-dev-cross-account.stdout.log`
- Create: `reference/tauri-dev-cross-account.stderr.log`
- Create: `reference/tauri-dev-cross-account.pid`

- [ ] **Step 1: Start the Tauri dev app**

Run with elevated permission if the sandbox blocks GUI/runtime startup:

```powershell
$stdout = Join-Path (Get-Location) 'reference\tauri-dev-cross-account.stdout.log'
$stderr = Join-Path (Get-Location) 'reference\tauri-dev-cross-account.stderr.log'
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-cross-account.pid'
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

- [ ] **Step 2: Connect to the Tauri MCP bridge**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "start", "port": 9223 })
```

Expected: session connects to the running app. If the bridge is not ready yet, wait a few seconds and retry once.

- [ ] **Step 3: Generate the account status script**

Run:

```powershell
$ctx = Get-Content -Raw reference\telegram-cross-account-validation-context.json | ConvertFrom-Json
@"
(async () => {
  return await window.__TAURI__.core.invoke('tg_get_account_statuses', {
    accountIds: [$($ctx.account_a_id), $($ctx.account_b_id)]
  });
})()
"@
```

Expected: printed JavaScript with two concrete account ids.

- [ ] **Step 4: Confirm both accounts are ready**

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set its `script` field to the
complete JavaScript printed in Task 3 Step 3.
```

Expected: two status records, both with `status = "ready"`. If account B is not `ready`, stop the app and document the slice as `blocked`.

### Task 4: Ensure The Shared Peer Exists Under Account B

**Files:**
- Read/Modify: `reference/telegram-cross-account-validation-context.json`
- Read: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`

- [ ] **Step 1: Generate the account B add-source script**

Run:

```powershell
$ctx = Get-Content -Raw reference\telegram-cross-account-validation-context.json | ConvertFrom-Json
$sourceRefJson = $ctx.source_ref | ConvertTo-Json -Compress
$expectedSubtypeJson = $ctx.expected_subtype | ConvertTo-Json -Compress
@"
(async () => {
  return await window.__TAURI__.core.invoke('add_telegram_source', {
    request: {
      accountId: $($ctx.account_b_id),
      sourceRef: $sourceRefJson,
      expectedSubtype: $expectedSubtypeJson
    }
  });
})()
"@
```

Expected: printed JavaScript with a concrete account B id, source ref, and expected subtype. Do not paste the source ref into tracked docs.

- [ ] **Step 2: Add or refresh the same peer under account B**

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set its `script` field to the
complete JavaScript printed in Task 4 Step 1.
```

Expected: a `SourceRecord` for account B. It may be a newly created source or an existing idempotently refreshed row.

- [ ] **Step 3: Verify account A/B typed identity pairing**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-cross-account-validation-context.json")
context = json.loads(context_path.read_text(encoding="utf-8"))

with sqlite3.connect(db_path) as conn:
    source_a = conn.execute(
        """
        SELECT
          s.id,
          s.account_id,
          s.source_type,
          ts.source_subtype,
          ts.peer_kind,
          ts.peer_id,
          ts.access_hash IS NOT NULL AS access_hash_present,
          ts.username IS NOT NULL AND TRIM(ts.username) <> '' AS username_present,
          ts.resolution_strategy,
          s.last_sync_state,
          s.last_synced_at
        FROM sources s
        JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE s.id = ?
        """,
        (context["source_a_id"],),
    ).fetchone()

    source_b = conn.execute(
        """
        SELECT
          s.id,
          s.account_id,
          s.source_type,
          ts.source_subtype,
          ts.peer_kind,
          ts.peer_id,
          ts.access_hash IS NOT NULL AS access_hash_present,
          ts.username IS NOT NULL AND TRIM(ts.username) <> '' AS username_present,
          ts.resolution_strategy,
          s.last_sync_state,
          s.last_synced_at
        FROM sources s
        JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE ts.account_id = ?
          AND ts.peer_kind = ?
          AND ts.peer_id = ?
        ORDER BY s.id ASC
        LIMIT 1
        """,
        (context["account_b_id"], context["peer_kind"], context["peer_id"]),
    ).fetchone()

if source_a is None:
    raise SystemExit("account A source row missing")
if source_b is None:
    raise SystemExit("account B did not save the confirmed same peer")

context["source_b_id"] = source_b[0]
context_path.write_text(json.dumps(context, indent=2), encoding="utf-8")

columns = [
    "source_id",
    "account_id",
    "source_type",
    "source_subtype",
    "peer_kind",
    "peer_id",
    "access_hash_present",
    "username_present",
    "resolution_strategy",
    "last_sync_state",
    "last_synced_at",
]
a = dict(zip(columns, source_a))
b = dict(zip(columns, source_b))

print(json.dumps({
    "source_a": a,
    "source_b": b,
    "source_id_distinct": a["source_id"] != b["source_id"],
    "account_id_distinct": a["account_id"] != b["account_id"],
    "peer_kind_matches": a["peer_kind"] == b["peer_kind"],
    "peer_id_matches": a["peer_id"] == b["peer_id"],
    "access_hash_presence_only": {
        "account_a": bool(a["access_hash_present"]),
        "account_b": bool(b["access_hash_present"]),
    },
}, indent=2))
'@ | python -
```

Expected: `source_id_distinct = true`, `account_id_distinct = true`, `peer_kind_matches = true`, and `peer_id_matches = true`. Do not compare access-hash values across accounts. If the confirmed same public peer saves with different `peer_kind` or `peer_id`, stop before sync and document `failed` or `needs follow-up`.

### Task 5: Snapshot State And Run Both Syncs

**Files:**
- Create: `reference/telegram-cross-account-capture-snapshot.py`
- Create/Modify: `reference/telegram-cross-account-validation-snapshots.json`
- Read: `C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db`

- [ ] **Step 1: Create the snapshot helper**

Run:

```powershell
@'
from pathlib import Path
import json
import sqlite3
import sys

db_path = Path(r"C:\Users\Dima\AppData\Roaming\org.ai.extractum\extractum.db")
context_path = Path(r"G:\Develop\Extractum\reference\telegram-cross-account-validation-context.json")
snapshots_path = Path(r"G:\Develop\Extractum\reference\telegram-cross-account-validation-snapshots.json")

if len(sys.argv) != 2:
    raise SystemExit("usage: telegram-cross-account-capture-snapshot.py <label>")

label = sys.argv[1]
allowed_labels = {"before_sync_a", "after_sync_a", "after_sync_b"}
if label not in allowed_labels:
    raise SystemExit(f"unsupported snapshot label: {label}")

context = json.loads(context_path.read_text(encoding="utf-8"))
source_ids = [context["source_a_id"], context["source_b_id"]]

with sqlite3.connect(db_path) as conn:
    source_rows = conn.execute(
        """
        SELECT id, account_id, last_sync_state, last_synced_at
        FROM sources
        WHERE id IN (?, ?)
        ORDER BY id ASC
        """,
        source_ids,
    ).fetchall()
    item_rows = conn.execute(
        """
        SELECT source_id, COUNT(*) AS item_count
        FROM items
        WHERE source_id IN (?, ?)
        GROUP BY source_id
        ORDER BY source_id ASC
        """,
        source_ids,
    ).fetchall()

snapshot = {
    "sources": {
        str(row[0]): {
            "id": row[0],
            "account_id": row[1],
            "last_sync_state": row[2],
            "last_synced_at": row[3],
        }
        for row in source_rows
    },
    "item_counts": {str(source_id): 0 for source_id in source_ids},
}
for source_id, item_count in item_rows:
    snapshot["item_counts"][str(source_id)] = item_count

snapshots = {}
if snapshots_path.exists():
    snapshots = json.loads(snapshots_path.read_text(encoding="utf-8"))
snapshots[label] = snapshot
snapshots_path.write_text(json.dumps(snapshots, indent=2), encoding="utf-8")

print(json.dumps({label: snapshot}, indent=2))
'@ | Set-Content -LiteralPath reference\telegram-cross-account-capture-snapshot.py -Encoding UTF8
```

Expected: `reference/telegram-cross-account-capture-snapshot.py` exists and accepts exactly these labels: `before_sync_a`, `after_sync_a`, and `after_sync_b`.

- [ ] **Step 2: Capture the `before_sync_a` snapshot**

Run:

```powershell
python reference\telegram-cross-account-capture-snapshot.py before_sync_a
```

Expected: `sources` contains both source ids and `item_counts` contains both source ids. This is the first required snapshot in the order `before sync A`, `after sync A`, `after sync B`.

- [ ] **Step 3: Generate the sync A script**

Run:

```powershell
$ctx = Get-Content -Raw reference\telegram-cross-account-validation-context.json | ConvertFrom-Json
@"
(async () => {
  return await window.__TAURI__.core.invoke('sync_source', {
    sourceId: $($ctx.source_a_id)
  });
})()
"@
```

Expected: printed JavaScript with account A's concrete `source_id`.

- [ ] **Step 4: Run `sync_source(source_id A)`**

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set its `script` field to the
complete JavaScript printed in Task 5 Step 3.
```

Expected: successful `SyncResult`. Record `inserted`, `skipped`, `last_message_id`, and `warnings`. If this returns an auth/runtime error unrelated to cross-account isolation, stop and document the slice as `blocked`.

- [ ] **Step 5: Capture the `after_sync_a` snapshot**

Run:

```powershell
python reference\telegram-cross-account-capture-snapshot.py after_sync_a
```

Expected: `source_b_id` has the same `last_sync_state`, `last_synced_at`, and item count as `before_sync_a`. `source_a_id` may change or remain unchanged.

- [ ] **Step 6: Generate the sync B script**

Run:

```powershell
$ctx = Get-Content -Raw reference\telegram-cross-account-validation-context.json | ConvertFrom-Json
@"
(async () => {
  return await window.__TAURI__.core.invoke('sync_source', {
    sourceId: $($ctx.source_b_id)
  });
})()
"@
```

Expected: printed JavaScript with account B's concrete `source_id`.

- [ ] **Step 7: Run `sync_source(source_id B)`**

Tool call:

```text
Use `mcp__tauri__.webview_execute_js` and set its `script` field to the
complete JavaScript printed in Task 5 Step 6.
```

Expected: successful `SyncResult`. Record `inserted`, `skipped`, `last_message_id`, and `warnings`. If warnings suggest account B used account A state or session, document `failed` or `needs follow-up`.

- [ ] **Step 8: Capture the `after_sync_b` snapshot**

Run:

```powershell
python reference\telegram-cross-account-capture-snapshot.py after_sync_b
```

Expected: `source_a_id` has the same `last_sync_state`, `last_synced_at`, and item count as `after_sync_a`. `source_b_id` may change or remain unchanged.

### Task 6: Evaluate Isolation Evidence

**Files:**
- Read: `reference/telegram-cross-account-validation-context.json`
- Read: `reference/telegram-cross-account-validation-snapshots.json`
- Create: `reference/telegram-cross-account-validation-evaluation.json`

- [ ] **Step 1: Evaluate pair and snapshot invariants**

Run:

```powershell
@'
from pathlib import Path
import json

context_path = Path(r"G:\Develop\Extractum\reference\telegram-cross-account-validation-context.json")
snapshots_path = Path(r"G:\Develop\Extractum\reference\telegram-cross-account-validation-snapshots.json")
evaluation_path = Path(r"G:\Develop\Extractum\reference\telegram-cross-account-validation-evaluation.json")

context = json.loads(context_path.read_text(encoding="utf-8"))
snapshots = json.loads(snapshots_path.read_text(encoding="utf-8"))

required = ["before_sync_a", "after_sync_a", "after_sync_b"]
missing = [label for label in required if label not in snapshots]
if missing:
    raise SystemExit(f"missing snapshots: {missing}")

source_a = str(context["source_a_id"])
source_b = str(context["source_b_id"])

def source_state(snapshot, source_id):
    return snapshots[snapshot]["sources"][source_id]

def item_count(snapshot, source_id):
    return snapshots[snapshot]["item_counts"][source_id]

before_a_b_state = source_state("before_sync_a", source_b)
after_a_b_state = source_state("after_sync_a", source_b)
after_a_a_state = source_state("after_sync_a", source_a)
after_b_a_state = source_state("after_sync_b", source_a)

evaluation = {
    "source_id_distinct": context["source_a_id"] != context["source_b_id"],
    "account_id_distinct": context["account_a_id"] != context["account_b_id"],
    "sync_a_did_not_mutate_source_b_state": before_a_b_state == after_a_b_state,
    "sync_a_did_not_mutate_source_b_items": item_count("before_sync_a", source_b) == item_count("after_sync_a", source_b),
    "sync_b_did_not_mutate_source_a_state": after_a_a_state == after_b_a_state,
    "sync_b_did_not_mutate_source_a_items": item_count("after_sync_a", source_a) == item_count("after_sync_b", source_a),
}
evaluation["snapshot_isolation_passed"] = all(evaluation.values())

evaluation_path.write_text(json.dumps(evaluation, indent=2), encoding="utf-8")
print(json.dumps(evaluation, indent=2))
'@ | python -
```

Expected: all fields are `true`, including `snapshot_isolation_passed = true`. If any field is false, document the result as `failed` or `needs follow-up` according to the observed mutation.

### Task 7: Stop Runtime Processes

**Files:**
- Read: `reference/tauri-dev-cross-account.pid`

- [ ] **Step 1: Stop the Tauri MCP session**

Tool call:

```text
mcp__tauri__.driver_session({ "action": "stop" })
```

Expected: all bridge sessions are stopped.

- [ ] **Step 2: Stop the Tauri dev process tree**

Run:

```powershell
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-cross-account.pid'
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

### Task 8: Document The Live Result

**Files:**
- Modify: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Update the verification matrix row**

If Task 6 passes and both sync results have acceptable warnings, update the row:

```markdown
| Same source on account A and account B | passed | public username | Selected one public channel or supergroup; account A reused an existing source row and account B added or reused its own row through `add_telegram_source` | Same `peer_kind`/`peer_id` recorded under different `account_id` values; access-hash and username recorded by presence only | `sync_source(source A)` and `sync_source(source B)` both succeeded; record inserted/skipped/last-message values in the dated note | Manual dispatch audit confirmed `sync_source` selects the grammers client by source `account_id`; snapshots showed each sync mutated only its own source state/items | Record warnings from both syncs, or `No warnings` |
```

If the probe is blocked or inconclusive, leave the row `blocked` or `needs follow-up` and state the exact non-sensitive reason.

- [ ] **Step 2: Add the dated live-run note**

Append a section to `docs/superpowers/verification/telegram-runtime-private-source-validation.md`:

```markdown
## 2026-05-21 Cross-Account Isolation Probe

- Account labels: account A `1`; account B recorded by id only. No credentials,
  phone numbers, session data, private message content, private titles, private
  usernames, or public username value recorded.
- App commit: the output of `git log -1 --oneline` captured before the sync
  run.
- Manual session-dispatch audit checked
  `src-tauri/src/sources/sync.rs` (`sync_source -> sync_telegram_source`),
  `src-tauri/src/telegram.rs` (`get_authorized_runtime`,
  `init_account_client`), and
  `src-tauri/src/telegram_session_store.rs` (`load_session(account_id)`).
  The sync path loads the source row, dispatches through that row's
  `account_id`, and initializes each grammers client from that account's
  session file.
- Selected public peer: public channel or public supergroup; username value not
  recorded.
- Stored identity: record account A/B `source_id`, `account_id`,
  `source_subtype`, `peer_kind`, `peer_id`, access-hash presence, username
  presence, and `resolution_strategy`. Do not record access-hash values.
- Add Source result: record whether account B created a new row or reused an
  existing row.
- Snapshot order: `before sync A`, `after sync A`, `after sync B`.
- Snapshot SQL used:
  `SELECT id, account_id, last_sync_state, last_synced_at FROM sources WHERE id IN (...)`
  and
  `SELECT source_id, COUNT(*) AS item_count FROM items WHERE source_id IN (...) GROUP BY source_id`.
- Sync A result: record `inserted`, `skipped`, `last_message_id`, and warnings.
- Sync B result: record `inserted`, `skipped`, `last_message_id`, and warnings.
- Isolation result: `source_id A != source_id B`, `account_id A != account_id B`,
  `peer_kind A == peer_kind B`, `peer_id A == peer_id B`, sync A did not mutate
  source B state/items, and sync B did not mutate source A state/items.
```

When writing this section, use the safe observed values from `reference/telegram-cross-account-validation-context.json`, `reference/telegram-cross-account-validation-snapshots.json`, the sync command outputs, and `reference/telegram-cross-account-validation-evaluation.json`.

- [ ] **Step 3: Update backlog only if passed**

If the matrix row is `passed`, remove this checklist row from `docs/backlog.md` section `3.1`:

```markdown
- [ ] validate cross-account isolation on two real Telegram accounts
```

Also update the 3.1 recent-evidence paragraph to mention the 2026-05-21 cross-account probe. Leave these rows open:

```markdown
- [ ] verify behavior when the user is no longer a member of a group or channel
- [ ] verify behavior for migrated small-group-to-supergroup dialogs
```

If the result is `blocked`, `failed`, or `needs follow-up`, keep the cross-account backlog row open and add only the non-sensitive evidence note.

### Task 9: Verification And Commit

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
git commit -m "docs: record telegram cross-account validation"
```

If the probe is blocked or inconclusive:

```powershell
git add docs\backlog.md docs\superpowers\verification\telegram-runtime-private-source-validation.md
git commit -m "docs: record telegram cross-account validation attempt"
```

- [ ] **Step 4: Verify the commit**

Run:

```powershell
git diff --check HEAD~1..HEAD
git status --short --branch
git log -1 --oneline
```

Expected: `git diff --check HEAD~1..HEAD` exits 0, status is clean on `main`, and the latest commit message matches the result.
