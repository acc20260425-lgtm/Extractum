# Takeout Source 113 Retry Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run a controlled, sanitized live Takeout retry validation for local Telegram `source_id=113` after the previous `TAKEOUT_INIT_DELAY` blocked batches.

**Architecture:** This is a validation-only slice. Use existing app flows for the live Takeout action, and use existing sanitized diagnostics to capture before/after source state, durable batch summaries, duplicate evidence, row-fidelity evidence, warning visibility, and recovery state. Do not change runtime code or decode private content.

**Tech Stack:** Tauri app flow, Rust/Tauri backend diagnostics, SQLite, existing Takeout provenance tables, Markdown verification docs.

---

## Goal

Validate the next Takeout attempt for `source_id=113` in a way that can update
the `3.1 Takeout Source Import Follow-Ups` backlog with precise evidence.

The run should clarify one or more of these open rows:

- repeated Takeout after normal sync;
- `CHANNEL_PRIVATE` / only-my-messages fallback;
- row-fidelity comparison between normal-sync canonical rows and Takeout
  observations;
- failed or cancelled Takeout recovery behavior.

## Target Source And Why

Target local source:

```text
source_id = 113
```

Current known shape from prior sanitized notes:

```text
source_subtype = channel
peer_kind = channel
has_username = 1
has_access_hash = 1
resolution_strategy = dialog
is_member = 0
```

Why this source:

- it already completed the normal-sync setup before Takeout;
- prior Takeout attempts produced batches `7` and `8`;
- both batches failed before observations with `TAKEOUT_INIT_DELAY`;
- it is the strongest current private/left-shape candidate for
  `CHANNEL_PRIVATE` fallback validation.

## Safety Boundary

Allowed evidence:

- local numeric ids such as `source_id`, `batch_id`, and capped sample ids;
- source subtype and peer kind;
- boolean identity flags such as `has_username`, `has_access_hash`, and
  `is_member`;
- aggregate counters;
- durable batch status, completeness, and warning codes;
- typed/coarse terminal error class, such as `TAKEOUT_INIT_DELAY`;
- `last_sync_state` and `last_synced_at`;
- source snapshot deltas.

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
- screenshots revealing private content.

## Outcome Decision Table

| Outcome | What to record | Matrix status impact |
| --- | --- | --- |
| `TAKEOUT_INIT_DELAY` again | typed/coarse error class, batch/recovery state, zero observations, before/after watermark equality | keep repeated-after-normal-sync `blocked`; add retry note; `CHANNEL_PRIVATE` remains `not run` |
| Takeout starts and observations are written | batch summary, duplicate summary, row-fidelity comparison, warning visibility, snapshot delta | depends on terminal state |
| `CHANNEL_PRIVATE` fallback | `only_my_messages_fallback` warning code, `only_my_messages` flag, status/completeness, partial/incomplete evidence | can close fallback evidence if warning/flag evidence is present; does not by itself close full import |
| `completed` / `complete` | before/after watermark, duplicate-after-normal-sync summary, row-fidelity comparison, warning visibility | can promote repeated-after-normal-sync if duplicates/fidelity match expectations |
| bounded cancel | partial rows, recovery state, warning visibility, before/after watermark equality | `needs follow-up`; useful cancellation evidence |
| failed non-delay | typed/coarse terminal class, sanitized batch state, warning visibility, no raw body | `failed` or `needs follow-up` depending on whether the failure is expected/provider-limited |

## Non-Goals

- Do not change runtime code.
- Do not add or modify Tauri commands.
- Do not decode or log private payloads.
- Do not paste message text, source title, username, phone number, or warning
  body.
- Do not delete Takeout batches, observations, source rows, or item rows.
- Do not try to bypass Telegram `TAKEOUT_INIT_DELAY`.
- Do not mark `CHANNEL_PRIVATE` validation complete without warning/flag
  evidence.
- Do not treat `CHANNEL_PRIVATE` fallback evidence as proof of the normal
  repeated-after-normal-sync happy path.

---

### Task 1: Pre-Run Evidence Capture

**Files:**
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Optional local-only: `reference/session-context-2026-05-10-analysis-redesign.md`

- [x] **Step 1: Confirm repository state**

Run:

```powershell
git status --short --branch
git log --oneline -5
```

Expected:

```text
## main
```

Record the current `HEAD` commit in the run note.

- [x] **Step 2: Capture the current sanitized source snapshot for source 113**

Use the existing Tauri app diagnostic path or backend helper path that returns
the `takeout_validation_source_snapshot` DTO for:

```text
source_id = 113
```

Record only sanitized fields:

```text
source_id
source_type
source_subtype
account_id
last_sync_state
last_synced_at
item_count
telegram_message_count
topic_membership_count
reply_count
thread_count
reaction_item_count
content/media/history aggregate distributions
```

Expected from previous note, before any new Takeout changes:

```text
item_count = 29
telegram_message_count = 29
reply_count = 16
thread_count = 5
reaction_item_count = 22
last_sync_state = 515
last_synced_at = 1779537575
```

If values differ, record the actual sanitized values and treat the difference
as part of the pre-run context.

- [x] **Step 3: Capture current Takeout recovery/batch baseline for source 113**

Use existing durable recovery or batch diagnostics for `source_id=113`.

Record:

```text
latest batch id
latest durable status
latest completeness
terminal error class if present
warning count
warning codes
observed count
duplicate count
inserted count
skipped count
```

Expected prior known batches:

```text
batch 7 = failed / unknown / TAKEOUT_INIT_DELAY / 0 observations
batch 8 = failed / unknown / TAKEOUT_INIT_DELAY / 0 observations
```

- [x] **Step 4: Add a dated pre-run note**

In `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`,
under `## Run Notes`, add:

```md
### 2026-05-23 Source 113 Takeout Retry Pre-Run

App commit: `bcf167c` or the actual current `HEAD` from Step 1. Working tree
was clean before this run.

Source `113` pre-run identity shape:

| Field | Value |
| --- | --- |
| source_subtype | channel |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 0 |
| resolution_strategy | dialog |

Source `113` pre-run snapshot:

| Field | Value |
| --- | ---: |
| item_count | use the captured value |
| telegram_message_count | use the captured value |
| topic_membership_count | use the captured value |
| reply_count | use the captured value |
| thread_count | use the captured value |
| reaction_item_count | use the captured value |
| last_sync_state | use the captured value |
| last_synced_at | use the captured value |

Latest pre-run Takeout state for source `113`:

| Batch id | Status | Completeness | Terminal error class | Observed | Inserted | Duplicates | Skipped | Warnings |
| ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| use the latest captured batch id | use captured status | use captured completeness | use captured terminal class or `none` | use captured observed count | use captured inserted count | use captured duplicate count | use captured skipped count | use captured warning count |
```

Replace every "use captured..." cell with the actual sanitized value before
checking the step.

- [x] **Step 5: Commit pre-run evidence**

Run:

```powershell
git diff --check
git add docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md
git commit -m "docs: record source 113 takeout retry pre-run"
```

Expected: commit succeeds. `git diff --check` may print only existing CRLF
warnings and must exit 0.

---

### Task 2: Live Takeout Retry

**Files:**
- Modify after live run: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Optional local-only: `reference/session-context-2026-05-10-analysis-redesign.md`

- [x] **Step 1: Start the existing app flow**

Use the already-running app if it is available. If it is not available, start
the normal local Tauri dev flow used in this project.

Do not stop user-started app processes unless explicitly instructed.

- [x] **Step 2: Trigger Takeout for source 113**

Through the existing application flow, start a Takeout import for:

```text
source_id = 113
```

Do not alter source identity, account settings, app code, or database rows by
hand.

- [x] **Step 3: Monitor only coarse terminal state**

Watch for one of these outcomes:

```text
TAKEOUT_INIT_DELAY
observations written
CHANNEL_PRIVATE / only-my-messages fallback
completed / complete
bounded cancellation
failed non-delay
```

Do not copy raw provider errors. Record only typed/coarse terminal classes and
warning codes.

- [x] **Step 4: If the run grows too large, perform bounded cancellation**

If the estimate or runtime makes a complete run impractical, cancel through the
normal app flow.

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

---

### Task 3: Post-Run Evidence Capture

**Files:**
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Optional local-only: `reference/session-context-2026-05-10-analysis-redesign.md`

- [x] **Step 1: Capture post-run source snapshot**

Capture `takeout_validation_source_snapshot` for:

```text
source_id = 113
```

Record the same fields captured pre-run so watermark and aggregate deltas can
be compared directly.

- [x] **Step 2: Capture latest batch summary**

Capture `takeout_validation_batch_summary` for the new batch id.

Record:

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
terminal_error_class
used_export_dc
fallback_used
migrated_detected
only_my_messages
message_count_estimate
max_message_id
```

- [x] **Step 3: Capture duplicate summary when observations exist**

If `observed > 0`, capture the duplicate summary for the new batch.

Record:

```text
inserted observations
duplicate observations
skipped observations
failed observations
```

If `observed = 0`, record:

```text
Duplicate summary not applicable because the batch wrote zero observations.
```

- [x] **Step 4: Capture row-fidelity comparison when observations exist**

If `observed > 0`, capture row fidelity in the relevant mode:

```text
duplicate_after_normal_sync
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

---

### Task 4: Matrix And Backlog Update

**Files:**
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Modify if status changes: `docs/backlog.md`

- [x] **Step 1: Add a post-run note**

Append a dated note to the verification doc:

```md
### 2026-05-23 Source 113 Takeout Retry Result

App commit: use the current `HEAD` recorded before the live run. Working tree
was clean before this run unless Step 1 showed otherwise.

Outcome: write exactly one decision-table outcome category:
`TAKEOUT_INIT_DELAY`, `observations_written`,
`CHANNEL_PRIVATE_fallback`, `completed_complete`, `bounded_cancel`, or
`failed_non_delay`.

Source `113` before/after snapshot:

| Field | Before | After | Delta |
| --- | ---: | ---: | ---: |
| item_count | pre-run value | post-run value | numeric delta |
| telegram_message_count | pre-run value | post-run value | numeric delta |
| topic_membership_count | pre-run value | post-run value | numeric delta |
| reply_count | pre-run value | post-run value | numeric delta |
| thread_count | pre-run value | post-run value | numeric delta |
| reaction_item_count | pre-run value | post-run value | numeric delta |
| last_sync_state | pre-run value | post-run value | `unchanged`, `advanced`, or `changed unexpectedly` |
| last_synced_at | pre-run value | post-run value | `unchanged`, `advanced`, or `changed unexpectedly` |

Batch summary:

| Batch id | Source id | Status | Completeness | Terminal error class | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages |
| ---: | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| new batch id | 113 | captured status | captured completeness | captured terminal class or `none` | captured inserted count | captured observed count | captured duplicate count | captured skipped count | captured warning count | 0 or 1 | 0 or 1 | 0 or 1 | 0 or 1 |

Warning codes for the new batch: write `none` or the sorted warning codes.

Duplicate summary: write `not applicable` when `observed = 0`; otherwise write
the sanitized aggregate counts.

Row-fidelity comparison: write `not applicable` when `observed = 0`; otherwise
write the sanitized aggregate comparison.

Result: write one sentence that maps the outcome to the matrix status impact.
```

Replace every "captured..." and "pre/post-run value" cell with the actual
sanitized value before checking the step.

- [x] **Step 2: Update the matrix row statuses conservatively**

Apply the decision table:

- If `TAKEOUT_INIT_DELAY` repeats, keep `Repeated Takeout after normal sync`
  `blocked`; keep `CHANNEL_PRIVATE fallback` `not run`.
- If observations are written but the run is cancelled, mark relevant rows
  `needs follow-up` unless a specific fallback warning is proven.
- If `only_my_messages_fallback` warning and flag are present, update the
  `CHANNEL_PRIVATE fallback` row with source/batch id and evidence, but do not
  claim full import validation.
- If the run completes cleanly and duplicate/fidelity evidence is good, update
  `Repeated Takeout after normal sync` from `blocked` to `passed`.
- If a non-delay provider failure occurs, mark the relevant row `failed` or
  `needs follow-up` based on the typed/coarse terminal class and captured
  evidence.

- [x] **Step 3: Update backlog notes only if status changes**

In `docs/backlog.md`, update the nested notes under section `3.1` only if new
evidence changes the current state.

Allowed examples:

```md
- Source `113` retry batch `9` remained blocked before observations with
  `TAKEOUT_INIT_DELAY`.
```

```md
- Source `113` retry batch `9` produced `only_my_messages_fallback`
  evidence; full repeated-after-normal-sync import validation remains open.
```

```md
- Source `113` retry batch `9` completed and duplicate/fidelity diagnostics
  matched the normal-sync baseline.
```

- [x] **Step 4: Run documentation checks**

Run:

```powershell
rg -n "Source 113 Takeout Retry|source `113`|TAKEOUT_INIT_DELAY|only_my_messages_fallback|Repeated Takeout after normal sync" docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/backlog.md
git diff --check
```

Expected:

- the new notes are present;
- no forbidden private content was added;
- `git diff --check` exits 0. Existing CRLF warnings are acceptable if the exit
  code is 0.

- [x] **Step 5: Commit validation result**

Run:

```powershell
git add docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/backlog.md
git commit -m "docs: record source 113 takeout retry validation"
```

If `docs/backlog.md` did not change, omit it from `git add`.

---

### Task 5: Final Verification And Handoff

**Files:**
- Optional local-only: `reference/session-context-2026-05-10-analysis-redesign.md`

- [x] **Step 1: Verify final status**

Run:

```powershell
git status --short --branch
git log --oneline -6
```

Expected:

```text
## main
```

The latest commit should be the validation result commit.

- [x] **Step 2: Update local handoff context**

Update `reference/session-context-2026-05-10-analysis-redesign.md` with:

```text
- source_id 113 retry validation outcome;
- new batch id if any;
- matrix rows changed;
- remaining blocked/follow-up rows;
- exact next recommended 3.1 step;
- reminder of safety boundary.
```

This file is ignored by Git and should remain local-only.

- [x] **Step 3: Report outcome**

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

- Spec coverage: The plan covers pre-run watermark proof, live source `113`
  retry, outcome decision table, post-run diagnostics, conservative matrix
  updates, backlog notes, and handoff.
- Safety: The plan forbids private content, raw payloads, warning bodies,
  session/auth material, source titles, usernames, and message text.
- Scope: The plan does not change runtime code, Tauri commands, recovery
  behavior, forum-topic behavior, migrated-history import, or database rows by
  hand.
- Evidence: The plan requires explicit before/after snapshots and does not
  infer pre-run state after the fact.
- Status discipline: The plan does not combine `CHANNEL_PRIVATE` fallback
  evidence with normal repeated-after-sync completion evidence.
