# Public Supergroup Source 122 Takeout Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run a controlled, sanitized public-supergroup Takeout validation for local Telegram `source_id=122`.

**Architecture:** This is a validation-only slice using existing Tauri app flows and existing sanitized SQLite/diagnostic queries. The plan captures explicit pre-run and post-run evidence, pauses before live Telegram action, and updates docs conservatively based only on captured evidence.

**Tech Stack:** Tauri app flow, SQLite read-only diagnostics, existing Takeout provenance tables, Markdown verification docs.

---

## Goal

Validate a completed public supergroup Takeout path after sources `19` and `22`
produced bounded-cancelled partial evidence.

Target source:

```text
source_id = 122
```

Current sanitized candidate shape:

```text
source_subtype = supergroup
peer_kind = channel
account_id = 1
has_username = 1
has_access_hash = 1
is_member = 1
resolution_strategy = dialog
item_count = 117
telegram_message_count = 117
max_telegram_message_id = 12238
topic_membership_count = 0
topic_membership_topic_count = 0
reply_count = 74
thread_count = 53
reaction_item_count = 18
reaction_count_sum = 23
last_sync_state = 12238
last_synced_at = 1779640088
prior_takeout_batches = none
```

Why this source:

- it is a newly added public/member supergroup with no prior Takeout batches;
- it has a small normal-sync baseline and is the best current candidate for a
  completed public-supergroup Takeout run;
- it has mixed text/media, reply, thread, and reaction shape;
- it uses account `1`, which completed source `18` Takeout and produced partial
  evidence for sources `19` and `22`;
- it avoids account `11`, which has recent `TAKEOUT_INIT_DELAY` history.

Known limitation:

- source `122` currently has no topic memberships and no topic resolver row, so
  this plan should not close forum-topic behavior unless the live evidence
  unexpectedly proves that exact row.

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
| completed / complete | before/after snapshots, batch summary, duplicate summary, row-fidelity comparison, warning visibility, watermark behavior | promote public supergroup Takeout if evidence is complete |
| completed / partial or cancelled | partial row counts, duplicate/fidelity aggregates if observations exist, warning visibility, watermark equality/advance | keep public supergroup `needs follow-up`; useful partial evidence |
| `TAKEOUT_INIT_DELAY` | typed/coarse error class, batch state, zero observations, before/after watermark equality | keep public supergroup `needs follow-up`; add blocked retry note |
| failed non-delay | typed/coarse terminal class, sanitized batch state, warning visibility, no raw body | mark public supergroup `failed` or `needs follow-up` based on provider class |
| fallback warning | warning code and flags only | update only the exact row supported by warning code/flag evidence |

## Non-Goals

- Do not change runtime code.
- Do not add or modify Tauri commands.
- Do not decode or log private payloads.
- Do not paste message text, source title, username, phone number, or warning
  body.
- Do not delete Takeout batches, observations, source rows, or item rows.
- Do not try to bypass Telegram `TAKEOUT_INIT_DELAY`.
- Do not mark shifted export DC fallback complete unless `export_dc_fallback`
  warning evidence exists.
- Do not mark `CHANNEL_PRIVATE` fallback complete from this public/member source
  unless a real `only_my_messages_fallback` warning/flag appears.
- Do not change forum-topic refresh behavior from this validation slice.

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

Expected:

```text
## takeout-source-122-public-supergroup-validation-plan
```

Record the current `HEAD` commit in the run note.

- [x] **Step 2: Capture current sanitized source snapshot for source 122**

Use existing sanitized diagnostics or read-only SQLite aggregate queries for:

```text
source_id = 122
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
resolution_strategy
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
topic catalog and resolver aggregate counters if present
```

Expected from candidate inventory:

```text
source_subtype = supergroup
peer_kind = channel
account_id = 1
has_username = 1
has_access_hash = 1
is_member = 1
item_count = 117
telegram_message_count = 117
max_telegram_message_id = 12238
topic_membership_count = 0
topic_membership_topic_count = 0
reply_count = 74
thread_count = 53
reaction_item_count = 18
reaction_count_sum = 23
last_sync_state = 12238
last_synced_at = 1779640088
```

If values differ, record the actual sanitized values and treat the difference
as the pre-run context.

- [x] **Step 3: Capture current Takeout baseline for source 122**

Use durable batch diagnostics for `source_id=122`.

Record:

```text
prior batch count
latest batch id, if any
latest durable status, if any
latest completeness, if any
latest terminal error class/presence, if any
warning count and warning codes, if any
observed/inserted/duplicate/skipped counts, if any
```

Expected from candidate inventory:

```text
prior_takeout_batches = none
```

- [x] **Step 4: Add a dated pre-run note**

In `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`,
under `## Run Notes`, add:

```md
### 2026-05-24 Source 122 Public Supergroup Takeout Pre-Run
```

Write the note with the current `HEAD` from Step 1 and the actual values from
Steps 2-3. Include aggregate distributions and topic aggregate counters if
available.

The expected pre-run aggregate distributions are:

```text
content_kind: media_only 2, text_only 101, text_with_media 14
media_kind: none 101, photo 6, video 1, webpage 9
history_peer_kind: channel 117
membership_match_kind: none
topic_catalog_count = 0
topic_resolver = none
```

- [x] **Step 5: Commit pre-run evidence**

Run:

```powershell
git diff --check
git add docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md
git commit -m "docs: record source 122 takeout pre-run"
```

Expected: commit succeeds. `git diff --check` may print only existing CRLF
warnings and must exit `0`.

---

### Task 2: Live Takeout Run

**Files:**
- Modify after live run: `docs/superpowers/plans/2026-05-24-public-supergroup-source-122-takeout-validation.md`

- [x] **Step 1: Start or reconnect the existing app flow**

Use the already-running app if it is available. If it is not available, start
the normal local Tauri dev flow used in this project.

Do not stop user-started app processes unless explicitly instructed.

- [x] **Step 2: Pause for explicit live authorization**

Before triggering Takeout, stop and ask the user to authorize the live action
for:

```text
source_id = 122
```

Do not proceed until the user explicitly confirms.

- [x] **Step 3: Trigger Takeout for source 122**

Through the existing application flow, start a Takeout import for:

```text
source_id = 122
```

Preferred invocation when the Tauri bridge is connected:

```javascript
window.__TAURI__.core.invoke("start_takeout_source_import", { sourceId: 122 })
```

Do not alter source identity, account settings, app code, or database rows by
hand.

- [x] **Step 4: Monitor only coarse terminal state**

Watch for one of these outcomes:

```text
completed / complete
TAKEOUT_INIT_DELAY
observations written
completed / partial
bounded cancellation
failed non-delay
fallback warning
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
last_sync_state before/after
last_synced_at before/after
```

Do not manually delete partial rows.

- [x] **Step 6: Commit live-run marker**

After a terminal state is reached or bounded cancellation is completed, mark
Task 2 steps complete in this plan and commit:

```powershell
git diff --check
git add docs/superpowers/plans/2026-05-24-public-supergroup-source-122-takeout-validation.md
git commit -m "docs: mark source 122 live takeout run"
```

Task 2 sanitized live-run marker:

| Field | Value |
| --- | --- |
| source_id | 122 |
| job_id | takeout-1 |
| batch_id | 13 |
| status | completed |
| completeness | complete |
| observed | 4564 |
| inserted | 4447 |
| duplicates | 117 |
| skipped | 0 |
| warnings | 0 |
| terminal_error_present | 0 |
| used_export_dc | 1 |
| fallback_used | 0 |
| migrated_history_detected | 0 |
| migrated_history_imported | 0 |
| only_my_messages | 0 |
| takeout_id_present | 1 |
| message_count_estimate | 5768 |
| max_message_id | null |
| started_at | 1779640726 |
| finished_at | 1779640790 |

The run completed cleanly, so bounded cancellation was not needed.

---

### Task 3: Post-Run Evidence Capture

**Files:**
- Modify: `docs/superpowers/plans/2026-05-24-public-supergroup-source-122-takeout-validation.md`

- [x] **Step 1: Capture post-run source snapshot**

Capture the same sanitized source `122` fields recorded in Task 1.

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

- [x] **Step 3: Capture duplicate summary when observations exist**

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

- [x] **Step 4: Capture row-fidelity comparison when observations exist**

If `observed > 0`, capture row fidelity in the relevant mode:

```text
takeout_batch_vs_canonical_source
```

Record aggregate categories and capped local sample ids only.

If `observed = 0`, record:

```text
Row-fidelity comparison not applicable because the batch wrote zero observations.
```

- [x] **Step 5: Capture warning visibility**

Capture warning visibility for the new batch.

Record warning codes only, especially:

```text
only_my_messages_fallback
export_dc_fallback
migrated_history_deferred
finish_takeout_failed
```

Do not record warning messages.

- [x] **Step 6: Capture explicit before/after delta**

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

- [x] **Step 7: Commit post-run capture marker**

Mark Task 3 steps complete in this plan and commit:

```powershell
git diff --check
git add docs/superpowers/plans/2026-05-24-public-supergroup-source-122-takeout-validation.md
git commit -m "docs: mark source 122 post-run capture"
```

Task 3 sanitized post-run capture:

Source `122` post-run snapshot:

| Field | Value |
| --- | ---: |
| item_count | 4564 |
| telegram_message_count | 4564 |
| max_telegram_message_id | 12238 |
| content_zstd_present_count | 4250 |
| topic_membership_count | 0 |
| topic_membership_topic_count | 0 |
| reply_count | 1676 |
| thread_count | 654 |
| reaction_item_count | 288 |
| reaction_count_sum | 719 |
| last_sync_state | 12238 |
| last_synced_at | 1779640790 |

Source `122` post-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 314 |
| content_kind | text_only | 3815 |
| content_kind | text_with_media | 435 |
| media_kind | animation | 1 |
| media_kind | document | 33 |
| media_kind | image | 23 |
| media_kind | none | 3815 |
| media_kind | photo | 305 |
| media_kind | poll | 3 |
| media_kind | sticker | 151 |
| media_kind | video | 13 |
| media_kind | webpage | 220 |
| history_peer_kind | channel | 4564 |

Source `122` topic catalog and resolver state after batch `13`:

| Field | Value |
| --- | --- |
| topic_catalog_count | 0 |
| distinct_topic_ids | 0 |
| resolver_state | none |

Batch `13` summary:

| Field | Value |
| --- | --- |
| source_id | 122 |
| status | completed |
| completeness | complete |
| terminal_error_present | 0 |
| inserted | 4447 |
| observed | 4564 |
| duplicates | 117 |
| skipped | 0 |
| warnings | 0 |
| started_at | 1779640726 |
| finished_at | 1779640790 |
| used_export_dc | 1 |
| fallback_used | 0 |
| migrated_history_detected | 0 |
| migrated_history_imported | 0 |
| only_my_messages | 0 |
| takeout_id_present | 1 |
| message_count_estimate | 5768 |
| max_message_id | null |

Duplicate summary for batch `13`:

| Field | Value |
| --- | ---: |
| inserted_count | 4447 |
| duplicate_observed_count | 117 |
| skipped_count | 0 |
| failed_count | 0 |
| duplicate_identity_count | 117 |
| has_duplicate_after_normal_sync_evidence | 1 |

Row-fidelity comparison for batch `13`:

| Field | Value |
| --- | ---: |
| observed_identity_count | 4564 |
| matched_canonical_identity_count | 4564 |
| missing_canonical_identity_count | 0 |
| canonical_without_observation_count | 0 |
| matched_content_zstd_present_count | 4250 |
| matched_reply_to_msg_id_present_count | 1676 |
| matched_reply_to_top_id_present_count | 654 |
| matched_reaction_count_present_count | 288 |

Row-fidelity mismatch categories: none.

Warning visibility for batch `13`:

- provenance warning codes: none;
- recovery candidate warning codes: none;
- latest batch for source `122`: yes;
- durable recovery kind: none.

Explicit before/after delta:

| Field | Before | After | Delta |
| --- | ---: | ---: | ---: |
| item_count | 117 | 4564 | 4447 |
| telegram_message_count | 117 | 4564 | 4447 |
| topic_membership_count | 0 | 0 | 0 |
| reply_count | 74 | 1676 | 1602 |
| thread_count | 53 | 654 | 601 |
| reaction_item_count | 18 | 288 | 270 |
| last_sync_state | 12238 | 12238 | unchanged |
| last_synced_at | 1779640088 | 1779640790 | advanced |

---

### Task 4: Matrix And Backlog Update

**Files:**
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Modify if status changes: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-24-public-supergroup-source-122-takeout-validation.md`

- [x] **Step 1: Add a post-run note**

Append a dated note named:

```md
### 2026-05-24 Source 122 Public Supergroup Takeout Result
```

The note must include:

- exactly one outcome category from the decision table;
- a before/after snapshot table for `item_count`,
  `telegram_message_count`, `topic_membership_count`, `reply_count`,
  `thread_count`, `reaction_item_count`, `last_sync_state`, and
  `last_synced_at`;
- a batch summary table for the new source `122` batch using the captured
  status, completeness, typed terminal class/presence, aggregate counts, and
  flags;
- warning codes as `none` or sorted warning code names;
- duplicate summary as `not applicable` when `observed = 0`, otherwise the
  sanitized aggregate counts;
- row-fidelity comparison as `not applicable` when `observed = 0`, otherwise
  sanitized aggregate categories and capped sample ids;
- topic catalog/resolver aggregate notes if they were captured;
- one result sentence mapping the outcome to the matrix status impact.

- [x] **Step 2: Update matrix row statuses conservatively**

Apply the decision table:

- If the run completes cleanly and row-fidelity diagnostics match expectations,
  update `Public supergroup Takeout` to `passed`.
- If observations are written but the run is partial/cancelled, keep
  `Public supergroup Takeout` as `needs follow-up`.
- If `TAKEOUT_INIT_DELAY` occurs, keep public supergroup `needs follow-up` and
  add the new batch note.
- If fallback warnings appear, update only the exact row supported by warning
  code/flag evidence.
- Do not mark forum-topic behavior complete unless the evidence directly proves
  the decision being recorded.

- [x] **Step 3: Update backlog notes if evidence changes 3.1 status**

In `docs/backlog.md`, update section `3.1` only if new evidence changes the
current state.

Allowed examples:

```md
- Source `122` completed public-supergroup Takeout with explicit before/after
  snapshots and row-fidelity evidence.
```

```md
- Source `122` public-supergroup Takeout remained blocked before observations
  with `TAKEOUT_INIT_DELAY`.
```

- [x] **Step 4: Run documentation checks**

Run:

```powershell
rg -n 'Source 122 Public Supergroup Takeout|source `122`|TAKEOUT_INIT_DELAY|Public supergroup Takeout|row-fidelity|duplicate' docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/backlog.md
git diff --check
```

Expected:

- the new notes are present;
- no forbidden private content was added;
- `git diff --check` exits `0`. Existing CRLF warnings are acceptable if the
  exit code is `0`.

- [x] **Step 5: Commit validation result**

Run:

```powershell
git add docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/backlog.md docs/superpowers/plans/2026-05-24-public-supergroup-source-122-takeout-validation.md
git commit -m "docs: record source 122 public supergroup takeout validation"
```

If `docs/backlog.md` did not change, omit it from `git add`.

---

### Task 5: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-05-24-public-supergroup-source-122-takeout-validation.md`

- [x] **Step 1: Verify final status**

Run:

```powershell
git status --short --branch
git log --oneline -6
git diff --check
```

Expected:

```text
## takeout-source-122-public-supergroup-validation-plan
```

- [x] **Step 2: Do not update local handoff context**

Per the user's latest instruction, do not continue writing task-by-task context
to `reference/session-context-2026-05-10-analysis-redesign.md`.

- [x] **Step 3: Commit completed plan marker**

Mark Task 5 steps complete in this plan and commit:

```powershell
git diff --check
git add docs/superpowers/plans/2026-05-24-public-supergroup-source-122-takeout-validation.md
git commit -m "docs: complete source 122 public supergroup validation plan"
```

- [x] **Step 4: Report outcome**

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

- Spec coverage: The plan covers pre-run watermark proof, live source `122`
  Takeout, post-run diagnostics, conservative matrix updates, backlog notes,
  and final verification.
- Safety: The plan forbids private content, raw payloads, warning bodies,
  session/auth material, source titles, usernames, message text, and
  `sources.metadata_zstd` contents.
- Scope: The plan does not change runtime code, Tauri commands, recovery
  behavior, forum-topic behavior, migrated-history import, or database rows by
  hand.
- Evidence: The plan requires explicit before/after snapshots and does not
  infer pre-run state after the fact.
- Status discipline: The plan only updates matrix rows supported by exact
  captured evidence.
