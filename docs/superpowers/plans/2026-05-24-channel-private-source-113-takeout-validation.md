# Channel Private Source 113 Takeout Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run a controlled, sanitized `CHANNEL_PRIVATE` fallback validation for local Telegram `source_id=113`.

**Architecture:** This is a validation-only slice using existing Tauri app flows and existing sanitized SQLite/diagnostic queries. The plan captures explicit pre-run and post-run evidence, pauses before live Telegram action, and updates docs conservatively based only on durable evidence.

**Tech Stack:** Tauri app flow, SQLite read-only diagnostics, existing Takeout provenance tables, Markdown verification docs.

---

## Goal

Validate whether source `113` reaches the Takeout `CHANNEL_PRIVATE` fallback
path and records durable only-my-messages evidence.

Target source:

```text
source_id = 113
```

Current sanitized candidate shape:

```text
source_subtype = channel
peer_kind = channel
account_id = 11
has_username = 1
has_access_hash = 1
is_member = 0
item_count = 29
telegram_message_count = 29
last_sync_state = 515
last_synced_at = 1779537575
prior_takeout_batches = 7, 8, 9
latest_terminal_error_class = TAKEOUT_INIT_DELAY
```

Why this source:

- it is the strongest private/left-shape candidate in the current sanitized
  inventory;
- the `CHANNEL_PRIVATE fallback` matrix row is still `not run`;
- batches `7`, `8`, and `9` failed before observations with
  `TAKEOUT_INIT_DELAY`, so no fallback evidence exists yet;
- a live run can either produce fallback evidence or refresh the blocked retry
  note with a new typed terminal class.

Known limitation:

- this source can validate only the fallback path and related durability. If the
  run enters only-my-messages mode, it does not prove complete source history.

Current local readiness at plan creation:

```text
tauri_driver_connected = false
extractum_processes_seen = 0
cargo_processes_seen = 0
```

The live task must start or reconnect the app before any Takeout action.

## Safety Boundary

Allowed evidence:

- local numeric ids such as `source_id`, `account_id`, and `batch_id`;
- source subtype and peer kind;
- boolean identity flags such as `has_username`, `has_access_hash`, and
  `is_member`;
- aggregate counters;
- durable batch status, completeness, and warning codes;
- typed/coarse terminal error classes, such as `TAKEOUT_INIT_DELAY`;
- `last_sync_state` and `last_synced_at`;
- source snapshot deltas;
- capped local sample ids from row-fidelity diagnostics if observations exist.

Forbidden evidence:

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
- `sources.metadata_zstd` contents.

## Outcome Decision Table

| Outcome | What to record | Matrix status impact |
| --- | --- | --- |
| `TAKEOUT_INIT_DELAY` again | typed/coarse error class, batch state, zero observations, before/after watermark equality | keep `CHANNEL_PRIVATE fallback` `not run`; add retry note |
| `CHANNEL_PRIVATE` fallback | `only_my_messages_fallback` warning code or equivalent durable flag, `only_my_messages` flag, fallback flag, status/completeness, partial/incomplete evidence | update `CHANNEL_PRIVATE fallback` from `not run` to the supported status |
| completed / complete without fallback | before/after snapshots, batch summary, duplicate summary, row-fidelity comparison, warning visibility, watermark behavior | keep `CHANNEL_PRIVATE fallback` `not run`; update only repeated/row-fidelity rows if supported |
| completed / partial or cancelled | partial row counts, duplicate/fidelity aggregates if observations exist, warning visibility, watermark equality/advance | keep fallback `not run` unless warning/flag evidence proves fallback |
| failed non-delay | typed/coarse terminal class, sanitized batch state, warning visibility, no raw body | mark fallback `failed` or `needs follow-up` only if the terminal class reached the relevant path |
| shifted export DC fallback warning | `export_dc_fallback` warning code and flags only | update only the shifted export DC row if exact warning evidence exists |

## Non-Goals

- Do not change runtime code.
- Do not add or modify Tauri commands.
- Do not decode or log private payloads.
- Do not paste message text, source title, username, phone number, or warning
  body.
- Do not delete Takeout batches, observations, source rows, or item rows.
- Do not try to bypass Telegram `TAKEOUT_INIT_DELAY`.
- Do not mark `CHANNEL_PRIVATE` fallback complete without warning/flag
  evidence.
- Do not treat fallback evidence as proof of full-history import.
- Do not update the local handoff file unless explicitly requested.

---

### Task 1: Pre-Run Evidence Capture

**Files:**
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`

- [x] **Step 1: Confirm repository state**

Run:

```powershell
git status --short --branch
git log --oneline -5
```

Expected branch:

```text
## takeout-source-113-channel-private-validation-plan
```

Record the current `HEAD` commit in the run note.

- [x] **Step 2: Capture current sanitized source snapshot for source 113**

Use existing sanitized diagnostics or read-only SQLite aggregate queries for:

```text
source_id = 113
```

Record only:

```text
source_id
source_type
source_subtype
account_id
peer_kind
has_username
has_access_hash
is_member
resolution_strategy if available
last_sync_state
last_synced_at
item_count
telegram_message_count
topic_membership_count
topic_membership_topic_count
reply_count
thread_count
reaction_item_count
reaction_count_sum
content_zstd_present_count
max_telegram_message_id
content/media/history aggregate distributions
```

Expected from the latest sanitized notes:

```text
source_subtype = channel
peer_kind = channel
account_id = 11
has_username = 1
has_access_hash = 1
is_member = 0
item_count = 29
telegram_message_count = 29
last_sync_state = 515
last_synced_at = 1779537575
```

If values differ, record the actual sanitized values and treat the difference
as the pre-run context.

- [x] **Step 3: Capture current Takeout baseline for source 113**

Use durable batch diagnostics for `source_id=113`.

Record:

```text
prior batch count
latest batch id
latest durable status
latest completeness
latest terminal error class/presence
warning count and warning codes
observed count
inserted count
duplicate count
skipped count
used_export_dc flag
fallback_used flag
only_my_messages flag
migrated_history_detected flag
message_count_estimate
max_message_id
```

Expected latest known batch:

```text
batch_id = 9
status = failed
completeness = unknown
terminal_error_class = TAKEOUT_INIT_DELAY
observed = 0
inserted = 0
duplicates = 0
skipped = 0
warnings = 0
used_export_dc = 1
fallback_used = 0
only_my_messages = 0
```

- [x] **Step 4: Add a dated pre-run note**

In `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`,
under `## Run Notes`, add:

```md
### 2026-05-24 Source 113 Channel Private Takeout Pre-Run
```

Write the note with the current `HEAD` from Step 1 and the actual values from
Steps 2-3. Include aggregate distributions if available.

The expected pre-run aggregate distributions from prior notes are:

```text
content_kind: media_only 1, text_only 8, text_with_media 20
media_kind: none 8, photo 6, webpage 15
history_peer_kind: channel 29
```

- [x] **Step 5: Commit pre-run evidence**

Run:

```powershell
git diff --check
git add docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md
git commit -m "docs: record source 113 channel-private pre-run"
```

Expected: commit succeeds. `git diff --check` must exit `0`.

---

### Task 2: Live Takeout Run

**Files:**
- Modify after live run: `docs/superpowers/plans/2026-05-24-channel-private-source-113-takeout-validation.md`

- [x] **Step 1: Start or reconnect the existing app flow**

Check Tauri bridge status:

```text
mcp__tauri__.driver_session({ "action": "status" })
```

If disconnected, start the normal local Tauri dev flow used in this project and
connect the bridge before invoking app commands. Do not stop user-started app
processes unless explicitly instructed.

- [x] **Step 2: Pause for explicit live authorization**

Before triggering Takeout, stop and ask the user to authorize the live action
for:

```text
source_id = 113
```

Do not proceed until the user explicitly confirms the live Takeout run.

- [x] **Step 3: Trigger Takeout for source 113**

Through the existing application flow, start a Takeout import for:

```text
source_id = 113
```

Preferred invocation when the Tauri bridge is connected:

```javascript
window.__TAURI__.core.invoke("start_takeout_source_import", { sourceId: 113 })
```

Do not alter source identity, account settings, app code, or database rows by
hand.

- [x] **Step 4: Monitor only coarse terminal state**

Watch for one of these outcomes:

```text
TAKEOUT_INIT_DELAY
CHANNEL_PRIVATE / only-my-messages fallback
only_my_messages_fallback warning
observations written
completed / complete
completed / partial
bounded cancellation
failed non-delay
export_dc_fallback warning
```

Do not copy raw provider errors. Record only typed/coarse terminal classes,
warning codes, aggregate counters, and local numeric ids.

- [x] **Step 5: If the run grows too large, pause before bounded cancellation**

If the estimate or runtime makes a complete run impractical, stop and ask the
user whether to keep waiting or cancel through the normal app flow.

Record:

```text
batch id
status
completeness
inserted
observed
duplicates
skipped
warnings
only_my_messages
fallback_used
last_sync_state before/after
last_synced_at before/after
```

Do not manually delete partial rows.

- [x] **Step 6: Commit live-run marker**

After a terminal state is reached or bounded cancellation is completed, mark
Task 2 steps complete in this plan and commit:

```powershell
git diff --check
git add docs/superpowers/plans/2026-05-24-channel-private-source-113-takeout-validation.md
git commit -m "docs: mark source 113 channel-private live run"
```

Task 2 sanitized live-run marker:

| Field | Value |
| --- | --- |
| source_id | 113 |
| job_id | takeout-1 |
| batch_id | 14 |
| status | completed |
| completeness | complete |
| observed | 467 |
| inserted | 438 |
| duplicates | 29 |
| skipped | 0 |
| warnings | 0 |
| terminal_error_class | none |
| used_export_dc | 1 |
| fallback_used | 0 |
| migrated_history_detected | 0 |
| migrated_history_imported | 0 |
| only_my_messages | 0 |
| takeout_id_present | 1 |
| message_count_estimate | 475 |
| max_message_id | null |
| started_at | 2026-05-24 17:14:17 |
| finished_at | 2026-05-24 17:14:29 |

The run completed cleanly without `CHANNEL_PRIVATE` fallback evidence, so
bounded cancellation was not needed.

---

### Task 3: Post-Run Evidence Capture

**Files:**
- Modify: `docs/superpowers/plans/2026-05-24-channel-private-source-113-takeout-validation.md`

- [x] **Step 1: Capture post-run source snapshot**

Capture the same sanitized source `113` fields recorded in Task 1.

- [x] **Step 2: Capture latest batch summary**

Capture the new Takeout batch summary and record:

```text
batch_id
source_id
status
completeness
started
finished
inserted
observed
duplicates
skipped
warnings
terminal_error_class_or_presence
used_export_dc
fallback_used
migrated_detected
only_my_messages
message_count_estimate
max_message_id
warning_codes
```

- [x] **Step 3: Capture fallback evidence when present**

If warning or flag evidence appears, record only:

```text
only_my_messages_fallback warning code presence
only_my_messages flag
fallback_used flag
observed count
inserted count
completeness
durable recovery kind if any
```

If no fallback evidence appears, record:

```text
No only-my-messages fallback warning or durable fallback flag was captured.
```

- [x] **Step 4: Capture duplicate summary when observations exist**

If `observed > 0`, capture duplicate summary:

```text
inserted observations
duplicate observations
skipped observations
failed observations
duplicate identity count
has duplicate-after-normal-sync evidence
```

If `observed = 0`, record:

```text
Duplicate summary not applicable because the batch wrote zero observations.
```

- [x] **Step 5: Capture row-fidelity comparison when observations exist**

If `observed > 0`, capture row fidelity in the relevant mode:

```text
takeout_batch_vs_canonical_source
```

Record aggregate categories and capped local sample ids only.

If `observed = 0`, record:

```text
Row-fidelity comparison not applicable because the batch wrote zero observations.
```

- [x] **Step 6: Capture warning visibility**

Capture warning visibility for the new batch.

Record warning codes only, especially:

```text
only_my_messages_fallback
export_dc_fallback
migrated_history_deferred
finish_takeout_failed
```

Do not record warning messages.

- [x] **Step 7: Capture explicit before/after delta**

Compare the pre-run and post-run sanitized source snapshots.

Record:

```text
item_count delta
telegram_message_count delta
topic_membership_count delta
reply_count delta
thread_count delta
reaction_item_count delta
last_sync_state before/after
last_synced_at before/after
```

For failed or cancelled runs, explicitly state whether `last_sync_state` and
`last_synced_at` stayed equal.

- [x] **Step 8: Commit post-run capture marker**

Mark Task 3 steps complete in this plan and commit:

```powershell
git diff --check
git add docs/superpowers/plans/2026-05-24-channel-private-source-113-takeout-validation.md
git commit -m "docs: mark source 113 channel-private post-run capture"
```

Task 3 sanitized post-run capture:

Source `113` post-run snapshot:

| Field | Value |
| --- | ---: |
| item_count | 467 |
| telegram_message_count | 467 |
| max_telegram_message_id | 515 |
| content_zstd_present_count | 420 |
| topic_membership_count | 0 |
| topic_membership_topic_count | 0 |
| reply_count | 155 |
| thread_count | 38 |
| reaction_item_count | 317 |
| reaction_count_sum | 1592 |
| last_sync_state | 515 |
| last_synced_at | 1779642869 |

Source `113` post-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 47 |
| content_kind | text_only | 164 |
| content_kind | text_with_media | 256 |
| media_kind | animation | 1 |
| media_kind | document | 8 |
| media_kind | none | 164 |
| media_kind | photo | 70 |
| media_kind | poll | 1 |
| media_kind | video | 5 |
| media_kind | webpage | 218 |
| history_peer_kind | channel | 467 |

Source `113` topic catalog and resolver state after batch `14`:

| Field | Value |
| --- | --- |
| topic_catalog_count | 0 |
| distinct_topic_ids | 0 |
| resolver_state | none |

Batch `14` summary:

| Field | Value |
| --- | --- |
| source_id | 113 |
| status | completed |
| completeness | complete |
| terminal_error_class | none |
| inserted | 438 |
| observed | 467 |
| duplicates | 29 |
| skipped | 0 |
| warnings | 0 |
| started_at | 2026-05-24 17:14:17 |
| finished_at | 2026-05-24 17:14:29 |
| used_export_dc | 1 |
| fallback_used | 0 |
| migrated_history_detected | 0 |
| migrated_history_imported | 0 |
| only_my_messages | 0 |
| takeout_id_present | 1 |
| message_count_estimate | 475 |
| max_message_id | null |

Fallback evidence for batch `14`:

- `only_my_messages_fallback` warning code: absent;
- `only_my_messages`: `0`;
- `fallback_used`: `0`;
- durable recovery kind: none.

Duplicate summary for batch `14`:

| Field | Value |
| --- | ---: |
| inserted_count | 438 |
| duplicate_observed_count | 29 |
| skipped_count | 0 |
| failed_count | 0 |
| duplicate_identity_count | 29 |
| has_duplicate_after_normal_sync_evidence | 1 |

Row-fidelity comparison for batch `14`:

| Field | Value |
| --- | ---: |
| observed_identity_count | 467 |
| matched_canonical_identity_count | 467 |
| missing_canonical_identity_count | 0 |
| canonical_without_observation_count | 0 |
| matched_content_zstd_present_count | 420 |
| matched_reply_to_msg_id_present_count | 155 |
| matched_reply_to_top_id_present_count | 38 |
| matched_reaction_count_present_count | 317 |

Matched content/media distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 47 |
| content_kind | text_only | 164 |
| content_kind | text_with_media | 256 |
| media_kind | animation | 1 |
| media_kind | document | 8 |
| media_kind | none | 164 |
| media_kind | photo | 70 |
| media_kind | poll | 1 |
| media_kind | video | 5 |
| media_kind | webpage | 218 |

Row-fidelity mismatch categories: none.

Warning visibility for batch `14`:

- provenance warning codes: none;
- recovery candidate warning codes: none;
- latest batch for source `113`: yes;
- durable recovery kind: none.

Explicit before/after delta:

| Field | Before | After | Delta |
| --- | ---: | ---: | ---: |
| item_count | 29 | 467 | 438 |
| telegram_message_count | 29 | 467 | 438 |
| topic_membership_count | 0 | 0 | 0 |
| reply_count | 16 | 155 | 139 |
| thread_count | 5 | 38 | 33 |
| reaction_item_count | 22 | 317 | 295 |
| last_sync_state | 515 | 515 | unchanged |
| last_synced_at | 1779537575 | 1779642869 | advanced |

---

### Task 4: Matrix And Backlog Update

**Files:**
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Modify if status changes: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-24-channel-private-source-113-takeout-validation.md`

- [x] **Step 1: Add a post-run note**

Append a dated note named:

```md
### 2026-05-24 Source 113 Channel Private Takeout Result
```

The note must include:

- exactly one outcome category from the decision table;
- a before/after snapshot table for `item_count`,
  `telegram_message_count`, `topic_membership_count`, `reply_count`,
  `thread_count`, `reaction_item_count`, `last_sync_state`, and
  `last_synced_at`;
- a batch summary table for the new source `113` batch using the captured
  status, completeness, typed terminal class/presence, aggregate counts, and
  flags;
- warning codes as `none` or sorted warning code names;
- fallback evidence as present or absent;
- duplicate summary as `not applicable` when `observed = 0`, otherwise the
  sanitized aggregate counts;
- row-fidelity comparison as `not applicable` when `observed = 0`, otherwise
  sanitized aggregate categories and capped sample ids;
- one result sentence mapping the outcome to the matrix status impact.

- [x] **Step 2: Update matrix row statuses conservatively**

Apply the decision table:

- If `TAKEOUT_INIT_DELAY` repeats, keep `CHANNEL_PRIVATE fallback` as
  `not run` and add the new blocked retry note.
- If `only_my_messages_fallback` warning and `only_my_messages`/fallback flags
  are present, update `CHANNEL_PRIVATE fallback` with source/batch id and the
  exact evidence.
- If observations are written without fallback, update only duplicate/fidelity
  rows supported by the captured evidence.
- If the run completes cleanly without fallback evidence, keep
  `CHANNEL_PRIVATE fallback` as `not run`.
- If a non-delay provider failure occurs, mark the relevant row `failed` or
  `needs follow-up` based on the typed/coarse terminal class and captured
  evidence.

- [x] **Step 3: Update backlog notes if evidence changes 3.1 status**

In `docs/backlog.md`, update section `3.1` only if new evidence changes the
current state.

Allowed examples:

```md
- Source `113` retry batch `N` remained blocked before observations with
  `TAKEOUT_INIT_DELAY`.
```

```md
- Source `113` retry batch `N` produced `only_my_messages_fallback`
  evidence; full-history import validation remains separate.
```

```md
- Source `113` retry batch `N` wrote observations without fallback evidence;
  `CHANNEL_PRIVATE` fallback remains unproven.
```

- [x] **Step 4: Run documentation checks**

Run:

```powershell
rg -n 'Source 113 Channel Private Takeout|source `113`|TAKEOUT_INIT_DELAY|only_my_messages_fallback|CHANNEL_PRIVATE fallback' docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/backlog.md
git diff --check
```

Expected:

- the new notes are present;
- no forbidden private content was added;
- `git diff --check` exits `0`.

- [x] **Step 5: Commit validation result**

Run:

```powershell
git add docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/backlog.md docs/superpowers/plans/2026-05-24-channel-private-source-113-takeout-validation.md
git commit -m "docs: record source 113 channel-private validation"
```

If `docs/backlog.md` did not change, omit it from `git add`.

---

### Task 5: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-05-24-channel-private-source-113-takeout-validation.md`

- [ ] **Step 1: Verify final status**

Run:

```powershell
git status --short --branch
git log --oneline -6
git diff --check
```

Expected branch:

```text
## takeout-source-113-channel-private-validation-plan
```

- [ ] **Step 2: Do not update local handoff context**

Per the user's latest preference, do not continue writing task-by-task context
to `reference/session-context-2026-05-10-analysis-redesign.md`.

- [ ] **Step 3: Commit completed plan marker**

Mark Task 5 steps complete in this plan and commit:

```powershell
git diff --check
git add docs/superpowers/plans/2026-05-24-channel-private-source-113-takeout-validation.md
git commit -m "docs: complete source 113 channel-private validation plan"
```

- [ ] **Step 4: Report outcome**

Report:

```text
- outcome category from the decision table;
- source id and batch id only;
- matrix/backlog changes;
- verification commands run;
- next recommended 3.1 action.
```

Do not include raw provider messages, usernames, source titles, message text,
or warning bodies.

---

## Self-Review Checklist

- Spec coverage: The plan covers pre-run watermark proof, source `113` live
  Takeout, fallback-specific evidence, post-run diagnostics, conservative
  matrix updates, backlog notes, and final verification.
- Safety: The plan forbids private content, raw payloads, warning bodies,
  session/auth material, source titles, usernames, message text, and
  `sources.metadata_zstd` contents.
- Scope: The plan does not change runtime code, Tauri commands, recovery
  behavior, forum-topic behavior, migrated-history import, or database rows by
  hand.
- Evidence: The plan requires explicit before/after snapshots and does not
  infer fallback behavior from source shape alone.
- Status discipline: The plan only updates matrix rows supported by exact
  captured warning/flag, batch, and observation evidence.
