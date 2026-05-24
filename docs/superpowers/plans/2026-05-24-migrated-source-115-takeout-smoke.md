# Migrated Source 115 Takeout Smoke Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Retry a controlled, sanitized Takeout smoke for local Telegram `source_id=115` and verify migrated small-group-to-supergroup deferment without importing unsafe old `chat` history.

**Architecture:** This is a validation-only slice using existing Tauri app flows, existing Takeout provenance, and read-only SQLite diagnostics. The run captures before/after source and batch state, classifies durable migrated-history evidence, and updates docs without changing runtime code.

**Tech Stack:** Tauri dev app, Tauri MCP bridge, PowerShell, Python `sqlite3`, SQLite, existing Takeout provenance tables, Markdown verification docs.

---

## Goal

Validate whether source `115` can now pass the Takeout migrated-history smoke
after the prior `TAKEOUT_INIT_DELAY` blocker.

Target source:

```text
source_id = 115
```

Current sanitized candidate shape at plan creation:

```text
source_type = telegram
source_subtype = supergroup
account_id = 11
peer_kind = channel
has_username = 0
has_access_hash = 1
is_active = 1
is_member = 1
resolution_strategy = dialog
last_sync_state = 2
last_synced_at = 1779472613
item_count = 1
telegram_message_count = 1
topic_membership_count = 0
reply_count = 0
thread_count = 0
reaction_item_count = 0
```

Existing Takeout blocker:

```text
batch_id = 2
status = failed
completeness = unknown
terminal_error_class = TAKEOUT_INIT_DELAY
observed = 0
inserted = 0
warnings = 0
migrated_history_detected = 0
migrated_history_imported = 0
history_scope = unknown
takeout_id_present = 0
```

Why this source:

- it is the existing controlled migrated small-group-to-supergroup smoke source;
- it is typed as `supergroup` while the Telegram history peer is a `channel`;
- it has a prior Takeout batch that failed before observations with
  `TAKEOUT_INIT_DELAY`, so a retry can now either collect the intended
  migrated-history deferment evidence or confirm the blocker persists;
- it has a very small current-history local baseline, reducing validation
  noise.

## Safety Boundary

Allowed tracked evidence:

- local numeric ids such as `source_id`, `account_id`, and `batch_id`;
- source subtype and peer kind;
- boolean identity flags such as `has_username`, `has_access_hash`, and
  `is_member`;
- aggregate counters;
- durable batch status, completeness, and warning codes;
- typed/coarse terminal error classes, such as `TAKEOUT_INIT_DELAY`;
- `history_scope`, `migrated_history_detected`,
  `migrated_history_imported`, and `only_my_messages`;
- `last_sync_state` and `last_synced_at`;
- before/after source snapshot deltas;
- capped local sample ids only if a diagnostics helper already emits them
  without content.

Forbidden tracked evidence:

- message text;
- source titles;
- usernames;
- phone numbers;
- account labels that identify a person/source;
- session/auth material;
- headers/cookies;
- raw TL payloads;
- raw provider payloads;
- compressed dumps;
- warning message bodies;
- screenshots revealing private content;
- `sources.metadata_zstd` contents;
- raw `external_id`, raw `peer_id`, raw `access_hash`, or raw message id ranges.

## Outcome Decision Table

| Outcome | What to record | Matrix status impact |
| --- | --- | --- |
| migrated-history deferment evidence | `migrated_history_detected = 1`, `migrated_history_imported = 0`, `migrated_history_deferred` warning code, `history_scope`, status/completeness, and no old `chat` history imported | move `Migrated small-group-to-supergroup smoke` from `blocked` to `passed` if the batch reaches durable deferment evidence without unsafe rows |
| `TAKEOUT_INIT_DELAY` again | typed/coarse error class, batch state, zero observations, before/after source watermark equality | keep the row `blocked`; add a dated retry note |
| completed without migrated-history evidence | before/after snapshots, batch summary, warning visibility, migrated flags, no unsafe old `chat` rows | mark `needs follow-up` unless the source no longer exposes migrated-history metadata to Takeout |
| failed non-delay | typed/coarse terminal class, sanitized batch state, warning visibility, no raw error body | mark `needs follow-up` or `failed` according to whether the failure reached migrated-history detection |
| active job or unavailable runtime | active job id/status only, or runtime blocker class | keep the row `blocked`; do not start a second import |

## Non-Goals

- Do not change Rust, Svelte, schema, or app code.
- Do not add or modify Tauri commands.
- Do not decode or log private payloads.
- Do not paste message text, source title, username, phone number, or warning
  body.
- Do not delete Takeout batches, observations, source rows, or item rows.
- Do not try to bypass Telegram `TAKEOUT_INIT_DELAY`.
- Do not enable migrated small-group history import.
- Do not mark migrated-history import as supported; this smoke only validates
  detection and deferment.
- Do not update the local handoff file unless explicitly requested.

---

### Task 1: Pre-Run Evidence Capture

**Files:**
- Modify: `docs/superpowers/plans/2026-05-24-migrated-source-115-takeout-smoke.md`
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`

- [x] **Step 1: Confirm repository state**

Run:

```powershell
git status --short --branch
git log --oneline -5
```

Expected branch for execution after the docs-prep commit:

```text
## takeout-source-115-migrated-smoke
```

Record the current `HEAD` commit in the run note.

- [x] **Step 2: Capture current sanitized source snapshot for source 115**

Run:

```powershell
@'
import os, sqlite3, json

path = os.path.join(os.environ["APPDATA"], "org.ai.extractum", "extractum.db")
source_id = 115
con = sqlite3.connect(path)
con.row_factory = sqlite3.Row
cur = con.cursor()

source = cur.execute("""
select
  s.id as source_id,
  s.source_type,
  s.source_subtype,
  s.account_id,
  s.last_sync_state,
  s.last_synced_at,
  s.is_active,
  s.is_member,
  ts.peer_kind,
  case when ts.username is not null and length(ts.username) > 0 then 1 else 0 end as has_username,
  case when ts.access_hash is not null then 1 else 0 end as has_access_hash,
  ts.resolution_strategy
from sources s
left join telegram_sources ts on ts.source_id = s.id
where s.id = ?
""", (source_id,)).fetchone()

counts = cur.execute("""
select
  count(*) as item_count,
  sum(case when item_kind = 'telegram_message' then 1 else 0 end) as telegram_message_count,
  sum(case when reply_to_msg_id is not null then 1 else 0 end) as reply_count,
  sum(case when reply_to_top_id is not null then 1 else 0 end) as thread_count,
  sum(case when reaction_count > 0 then 1 else 0 end) as reaction_item_count
from items
where source_id = ?
""", (source_id,)).fetchone()

topics = cur.execute("""
select
  count(*) as topic_membership_count,
  count(distinct topic_id) as topic_membership_topic_count
from item_topic_memberships
where source_id = ?
""", (source_id,)).fetchone()

print(json.dumps({
  "source": dict(source) if source else None,
  "counts": dict(counts),
  "topic_counts": dict(topics)
}, indent=2))
con.close()
'@ | python -
```

Expected: sanitized source shape only; no titles, usernames, raw ids, message
text, metadata blobs, or compressed payloads.

- [x] **Step 3: Capture prior Takeout batch summary for source 115**

Run:

```powershell
@'
import os, sqlite3, json

path = os.path.join(os.environ["APPDATA"], "org.ai.extractum", "extractum.db")
source_id = 115
con = sqlite3.connect(path)
con.row_factory = sqlite3.Row
cur = con.cursor()

def classify_error(value):
    text = value or ""
    for code in ["TAKEOUT_INIT_DELAY", "CHANNEL_PRIVATE", "FLOOD_WAIT", "AUTH_KEY"]:
        if code in text:
            return code
    return "present_unclassified" if text else None

rows = cur.execute("""
select
  b.id as batch_id,
  b.status,
  b.completeness,
  b.item_observed_count as observed,
  b.item_inserted_count as inserted,
  b.item_duplicate_count as duplicates,
  b.item_skipped_count as skipped,
  b.warning_count as warnings,
  b.terminal_error,
  t.source_subtype,
  t.resolved_peer_kind,
  t.used_export_dc,
  t.fallback_used,
  t.history_scope,
  t.migrated_history_detected,
  t.migrated_history_imported,
  t.only_my_messages,
  t.split_count,
  t.selected_split_count,
  t.message_count_estimate,
  case when t.takeout_id is null then 0 else 1 end as takeout_id_present,
  b.started_at,
  b.finished_at
from ingest_batches b
left join telegram_takeout_batches t on t.batch_id = b.id
where b.source_id = ? and b.ingest_kind = 'takeout'
order by b.id desc
limit 5
""", (source_id,)).fetchall()

batches = []
for row in rows:
    d = dict(row)
    d["terminal_error_class"] = classify_error(d.pop("terminal_error"))
    batches.append(d)

warnings = cur.execute("""
select b.id as batch_id, w.code, count(*) as count
from ingest_batches b
join ingest_batch_warnings w on w.batch_id = b.id
where b.source_id = ? and b.ingest_kind = 'takeout'
group by b.id, w.code
order by b.id desc, w.code
""", (source_id,)).fetchall()

print(json.dumps({
  "takeout_batches": batches,
  "warning_codes": [dict(row) for row in warnings]
}, indent=2))
con.close()
'@ | python -
```

Expected: prior batch `2` is visible as `failed / unknown` with
`terminal_error_class = TAKEOUT_INIT_DELAY`, zero observations, and no warning
codes.

- [x] **Step 4: Add a pre-run note**

Add a dated source `115` pre-run note to
`docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
with the sanitized source shape, previous batch summary, and chosen retry
criteria.

Expected: the matrix row remains `blocked` before the live retry.

### Task 2: Start Runtime And Connect Driver

**Files:**
- Create: `reference/tauri-dev-source-115-migrated-smoke.stdout.log`
- Create: `reference/tauri-dev-source-115-migrated-smoke.stderr.log`
- Create: `reference/tauri-dev-source-115-migrated-smoke.pid`

- [x] **Step 1: Start the Tauri dev app**

Run:

```powershell
New-Item -ItemType Directory -Force -Path reference | Out-Null
if (-not (Test-Path -LiteralPath node_modules)) {
  npm.cmd install --prefer-offline --no-audit --no-fund
  if ($LASTEXITCODE -ne 0) { throw "npm install failed" }
}
$stdout = Join-Path (Get-Location) 'reference\tauri-dev-source-115-migrated-smoke.stdout.log'
$stderr = Join-Path (Get-Location) 'reference\tauri-dev-source-115-migrated-smoke.stderr.log'
$pidPath = Join-Path (Get-Location) 'reference\tauri-dev-source-115-migrated-smoke.pid'
$p = Start-Process -FilePath 'npm.cmd' `
  -ArgumentList @('run', 'tauri', 'dev') `
  -WorkingDirectory (Get-Location) `
  -RedirectStandardOutput $stdout `
  -RedirectStandardError $stderr `
  -PassThru `
  -WindowStyle Hidden
$p.Id | Set-Content -LiteralPath $pidPath
```

Expected: a Tauri process starts and writes logs under ignored `reference/`
paths.

- [x] **Step 2: Connect the Tauri MCP bridge**

Tool call:

```text
Use mcp__tauri__.driver_session with action = "start".
```

Expected: driver session connects to the running app.

- [x] **Step 3: Confirm app state without private output**

Tool calls:

```text
Use mcp__tauri__.ipc_get_backend_state.
Use mcp__tauri__.webview_dom_snapshot with type = "accessibility".
```

Expected: app is responsive. Do not paste titles, account labels, or UI
screenshots into tracked docs.

### Task 3: Live Takeout Retry

**Files:**
- Create/Modify: `reference/source-115-migrated-smoke-result.json`

- [x] **Step 1: Confirm no active Takeout job for source 115**

Tool call:

```text
Use mcp__tauri__.webview_execute_js with:

(() => window.__TAURI__.core.invoke('list_takeout_source_import_jobs')
  .then((jobs) => jobs
    .filter((job) => job.source_id === 115)
    .map((job) => ({
      id: job.id,
      source_id: job.source_id,
      status: job.status,
      phase: job.phase,
      batch_id: job.batch_id || null,
      warning_count: job.warning_count || 0
    }))))
```

Expected: no `queued`, `running`, or `cancel_requested` job for source `115`.
If an active job exists, classify the smoke as `blocked_active_job`, skip
Step 2, and continue to Task 4.

- [x] **Step 2: Start Takeout import**

Tool call:

```text
Use mcp__tauri__.webview_execute_js with:

(async () => {
  try {
    const started = await window.__TAURI__.core.invoke('start_takeout_source_import', {
      sourceId: 115
    });
    return { ok: true, started };
  } catch (error) {
    return {
      ok: false,
      error: {
        kind: error && error.kind ? error.kind : null,
        code: error && error.code ? error.code : null
      }
    };
  }
})()
```

Expected: either `{ ok: true, started: ... }` or a typed blocker without raw
error message bodies. Save the returned object to
`reference/source-115-migrated-smoke-result.json`.

- [x] **Step 3: Poll the job to terminal state**

Tool call repeatedly, with a short wait between polls:

```text
Use mcp__tauri__.webview_execute_js with:

(() => window.__TAURI__.core.invoke('list_takeout_source_import_jobs')
  .then((jobs) => jobs
    .filter((job) => job.source_id === 115)
    .map((job) => ({
      id: job.id,
      source_id: job.source_id,
      status: job.status,
      phase: job.phase,
      batch_id: job.batch_id || null,
      warning_count: job.warning_count || 0,
      inserted_count: job.inserted_count || 0,
      observed_count: job.observed_count || 0,
      duplicate_count: job.duplicate_count || 0,
      skipped_count: job.skipped_count || 0
    }))))
```

Expected: terminal `completed`, `failed`, or `cancelled` state. If the job is
large and exceeds the agreed smoke window, cancel through the existing UI/app
flow and document the partial/cancelled state.

### Task 4: Durable Evidence Capture

**Files:**
- Create/Modify: `reference/source-115-migrated-smoke-summary.json`
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Modify: `docs/backlog.md`

- [x] **Step 1: Capture latest durable Takeout summary**

Run the query from Task 1 Step 3 again.

Expected: the newest batch for source `115` is present with sanitized status,
completeness, counters, warning codes, migrated-history flags, and terminal
class if any.

- [x] **Step 2: Check for unsafe migrated rows**

Run:

```powershell
@'
import os, sqlite3, json

path = os.path.join(os.environ["APPDATA"], "org.ai.extractum", "extractum.db")
source_id = 115
con = sqlite3.connect(path)
con.row_factory = sqlite3.Row
cur = con.cursor()

summary = cur.execute("""
select
  count(*) as telegram_message_count,
  sum(case when history_peer_kind = 'chat' then 1 else 0 end) as chat_history_rows,
  sum(case when is_migrated_history = 1 then 1 else 0 end) as migrated_history_rows,
  count(distinct history_peer_kind) as history_peer_kind_count
from telegram_messages
where source_id = ?
""", (source_id,)).fetchone()

print(json.dumps(dict(summary), indent=2))
con.close()
'@ | python -
```

Expected for a safe deferment smoke: `chat_history_rows = 0` and
`migrated_history_rows = 0`. A nonzero value is a `failed` smoke result until
the rows and provenance are investigated without exposing content.

- [x] **Step 3: Update the validation matrix**

Update
`docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`:

- keep `Migrated small-group-to-supergroup smoke` as `blocked` if the new run
  again fails before observations with `TAKEOUT_INIT_DELAY`;
- mark it `passed` only if durable provenance records
  `migrated_history_detected = 1`, `migrated_history_imported = 0`, a
  `migrated_history_deferred` warning code, and no unsafe old `chat` rows;
- mark it `needs follow-up` for successful current-history import without
  migrated deferment evidence;
- mark it `failed` for unsafe old `chat` rows or migrated rows imported before
  an explicit policy exists.

Expected: the dated run note contains only sanitized evidence.

- [x] **Step 4: Update backlog**

Update `docs/backlog.md`:

- if the smoke passes, keep migrated-history import enablement open and move
  the next Takeout focus to export-DC fallback and incomplete-import recovery;
- if the smoke remains blocked, record the latest batch id and blocker under
  the existing migrated smoke bullet;
- if the smoke needs follow-up or fails, name the durable reason without raw
  Telegram payloads.

### Task 5: Verification, Commit, And Runtime Cleanup

**Files:**
- Modify: `docs/superpowers/plans/2026-05-24-migrated-source-115-takeout-smoke.md`
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Modify: `docs/backlog.md`

- [x] **Step 1: Stop runtime process**

Tool call:

```text
Use mcp__tauri__.driver_session with action = "stop".
```

Then run:

```powershell
if (Test-Path -LiteralPath reference\tauri-dev-source-115-migrated-smoke.pid) {
  $pidValue = Get-Content -Raw -LiteralPath reference\tauri-dev-source-115-migrated-smoke.pid
  $pidValue = $pidValue.Trim()
  if ($pidValue) {
    Stop-Process -Id ([int]$pidValue) -ErrorAction SilentlyContinue
  }
}
```

Expected: no lingering Tauri dev app for this validation run.

- [x] **Step 2: Verify docs and workspace state**

Run:

```powershell
git diff --check
git status --short --branch
rg -n "Source 115|Migrated small-group-to-supergroup smoke|migrated_history_deferred|TAKEOUT_INIT_DELAY" docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/backlog.md
```

Expected: no whitespace errors; tracked docs show the sanitized outcome.

- [x] **Step 3: Commit the validation result**

Run:

```powershell
git add docs\superpowers\plans\2026-05-24-migrated-source-115-takeout-smoke.md docs\superpowers\verification\takeout-representative-validation-and-fallback-coverage.md docs\backlog.md
git commit -m "docs: record source 115 migrated takeout smoke"
```

Expected: commit succeeds with only tracked documentation changes.

## Self-Review

- Spec coverage: the plan covers source `115` retry selection, previous
  `TAKEOUT_INIT_DELAY` evidence, privacy guardrails, live Takeout start/poll,
  durable migrated-history deferment evidence, unsafe old `chat` row detection,
  matrix/backlog updates, verification, commit, and runtime cleanup.
- Placeholder scan: no `TBD`, `TODO`, or vague "add appropriate" steps remain.
- Type consistency: source id, warning code, migrated flags, history scope,
  batch fields, and doc paths match existing Takeout provenance vocabulary.
