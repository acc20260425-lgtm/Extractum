# Takeout Representative Validation And Fallback Coverage

> Status: reusable manual validation matrix. Rows start as `not run` until real
> Telegram accounts and representative sources are available.

Updated: 2026-05-23

Current matrix summary: `2 passed`, `3 needs follow-up`, `3 blocked`,
`2 not run`.

Covered highlights:

- completed small-group Takeout for source `118` / batch `6`;
- completed repeated Takeout-after-Takeout duplicate validation for source `73`
  / batch `3`;
- bounded partial public supergroup and dialog-backed no-username supergroup
  runs for sources `21` / `110`;
- normal-sync-before-Takeout attempt for source `113`, blocked by
  `TAKEOUT_INIT_DELAY` before Takeout observations were written;
- post-cancel watermark observations for cancelled partial batches `4` and `5`.

## Safety Boundary

Do not paste message text, source titles, usernames, phone numbers, account
labels that identify a person/source, session data, auth material, headers,
cookies, raw TL payloads, raw provider payloads, compressed dumps, warning
message bodies, or screenshots that reveal private content.

Paste only sanitized diagnostic output from the Takeout validation helpers:
aggregate counts, local numeric ids, source subtype, warning codes, flags,
typed/coarse terminal outcomes, and stable capped sample ids.

## Status Values

- `not run`
- `passed`
- `failed`
- `blocked`
- `needs follow-up`

## Matrix

| Case | Status | Source id | Batch id | Evidence to paste | Result notes |
| --- | --- | --- | --- | --- | --- |
| Public channel Takeout | needs follow-up | 73 | 3 | existing durable source summary and batch summary | Existing durable Takeout batch is complete with no warnings, but this matrix did not capture a dedicated before/after snapshot pair for the run |
| Public supergroup Takeout | needs follow-up | 21 | 4 | before/after source summary, cancelled partial Takeout batch summary, topic/reply/thread aggregate shape, warning visibility | Bounded live run imported partial history and was cancelled before full completion because the Takeout estimate was large |
| Private or dialog-backed supergroup Takeout | needs follow-up | 110 | 5 | before/after source summary, cancelled partial Takeout batch summary, warning visibility | Dialog-backed no-username supergroup path imported partial history and was cancelled before full completion because the Takeout estimate was large |
| Small group Takeout | passed | 118 | 6 | source subtype and peer-kind shape, before/after source summary, batch summary, watermark before/after | Completed cleanly for a dialog-backed `group` / `chat` source with no username or access hash |
| Repeated Takeout after normal sync | blocked | 113 | 7 | normal sync before snapshot and failed Takeout batch summary | Normal sync succeeded, but Takeout failed before observations with `TAKEOUT_INIT_DELAY`, so duplicate and row-fidelity comparison could not run |
| Repeated Takeout after previous Takeout | passed | 73 | 3 | duplicate observation summary and latest batch summary | Batch 3 followed prior Takeout batch 1 for the same source; latest batch completed with all observations classified as duplicates |
| `CHANNEL_PRIVATE` fallback | not run |  |  | `only_my_messages_fallback` warning code, partial/incomplete evidence, typed/coarse terminal outcome if present |  |
| Shifted export DC fallback | blocked |  |  | export DC attempted/fallback flags, `export_dc_fallback` warning code, typed/coarse terminal outcome if present | Requires an environment that naturally triggers local transport/session fallback |
| Migrated small-group-to-supergroup smoke | blocked | 115 | 2 | failed early Takeout batch summary | Existing smoke reached a failed/unknown terminal batch with zero observations before migrated-history evidence could be collected |
| Forum-topic decision input | not run |  |  | topic membership/catalog aggregate deltas after successful Takeout | No behavior decision in this validation slice |

## Procedure

1. Record the app commit and whether the working tree is clean.
2. Record the local `source_id`, coarse source classification, and source
   subtype.
3. Capture `takeout_validation_source_snapshot` before the run.
4. Run normal sync or Takeout manually through the existing app flow.
5. Capture the relevant source snapshot, Takeout batch summary, duplicate
   summary, row-fidelity comparison, snapshot delta, and warning visibility.
6. Paste only sanitized helper output into the row notes or dated run notes.
7. Mark the row `passed`, `failed`, `blocked`, or `needs follow-up`.

## Run Notes

Add dated notes below this heading. Keep each note sanitized and reference only
local numeric ids, aggregate counters, warning codes, flags, and typed/coarse
outcomes.

### 2026-05-23 Existing Durable Takeout Baseline

App commit: `3ccd8ec`. Working tree was clean before this note.

Sanitized source inventory:

| Source class | Count |
| --- | ---: |
| Telegram channel | 8 |
| Telegram supergroup | 8 |

Representative local source candidates:

| Source id | Subtype | Peer kind | Has username | Has access hash | Resolution strategy | Item count | Topic membership count |
| ---: | --- | --- | ---: | ---: | --- | ---: | ---: |
| 17 | supergroup | channel | 1 | 1 | dialog | 4634 | 4166 |
| 18 | channel | channel | 1 | 1 | dialog | 42 | 0 |
| 19 | supergroup | channel | 1 | 1 | dialog | 1950 | 0 |
| 21 | supergroup | channel | 1 | 1 | dialog | 375 | 370 |
| 27 | channel | channel | 0 | 1 | dialog | 1052 | 0 |
| 73 | channel | channel | 1 | 1 | dialog | 417 | 0 |
| 110 | supergroup | channel | 0 | 1 | dialog | 384 | 0 |
| 115 | supergroup | channel | 0 | 1 | dialog | 1 | 0 |

Existing source `73` snapshot:

| Field | Value |
| --- | ---: |
| item_count | 417 |
| telegram_message_count | 417 |
| topic_membership_count | 0 |
| reply_count | 4 |
| thread_count | 0 |
| reaction_item_count | 303 |

Source `73` aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 79 |
| content_kind | text_only | 131 |
| content_kind | text_with_media | 207 |
| media_kind | none | 131 |
| media_kind | photo | 187 |
| media_kind | poll | 27 |
| media_kind | webpage | 72 |
| history_peer_kind | channel | 417 |

Existing Takeout batches:

| Batch id | Source id | Status | Completeness | Subtype | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages |
| ---: | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 73 | completed | complete | channel | 415 | 416 | 1 | 0 | 0 | 1 | 0 | 0 | 0 |
| 3 | 73 | completed | complete | channel | 0 | 417 | 417 | 0 | 0 | 1 | 0 | 0 | 0 |
| 2 | 115 | failed | unknown | supergroup | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |

Batch `1` outcome counts:

| Outcome | Count |
| --- | ---: |
| duplicate_observed | 1 |
| inserted | 415 |

Batch `3` outcome counts:

| Outcome | Count |
| --- | ---: |
| duplicate_observed | 417 |

Warning codes for batches `1`, `2`, and `3`: none.

### 2026-05-23 Public Supergroup Bounded Live Takeout

App commit: `02d6cb5`. Working tree was clean before this run.

Source `21` before snapshot:

| Field | Value |
| --- | ---: |
| item_count | 375 |
| telegram_message_count | 375 |
| topic_membership_count | 370 |
| reply_count | 367 |
| thread_count | 269 |
| reaction_item_count | 94 |

Source `21` after bounded run:

| Field | Value |
| --- | ---: |
| item_count | 16085 |
| telegram_message_count | 16085 |
| topic_membership_count | 8885 |
| reply_count | 10737 |
| thread_count | 7464 |
| reaction_item_count | 281 |

Explicit snapshot delta for source `21`:

| Field | Delta |
| --- | ---: |
| item_count | 15710 |
| telegram_message_count | 15710 |
| topic_membership_count | 8515 |
| reply_count | 10370 |
| thread_count | 7195 |
| reaction_item_count | 187 |

Source `21` after-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 320 |
| content_kind | text_only | 14714 |
| content_kind | text_with_media | 1051 |
| media_kind | audio | 25 |
| media_kind | document | 26 |
| media_kind | image | 28 |
| media_kind | none | 14714 |
| media_kind | photo | 437 |
| media_kind | poll | 7 |
| media_kind | sticker | 57 |
| media_kind | video | 29 |
| media_kind | voice | 2 |
| media_kind | webpage | 760 |
| history_peer_kind | channel | 16085 |

Batch `4` summary:

| Batch id | Source id | Status | Completeness | Subtype | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages | Message count estimate |
| ---: | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 21 | cancelled | partial | supergroup | 15710 | 15710 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 132886 |

Batch `4` outcome counts:

| Outcome | Count |
| --- | ---: |
| inserted | 15710 |

Warning codes for batch `4`: none.

Latest durable recovery state for source `21`: `cancelled` / `partial`, warning
count `0`.

### 2026-05-23 Dialog-Backed Supergroup Bounded Live Takeout

App commit: `2721cf9`. Working tree was clean before this run.

Source `110` identity shape:

| Field | Value |
| --- | --- |
| source_subtype | supergroup |
| peer_kind | channel |
| has_username | 0 |
| has_access_hash | 1 |
| resolution_strategy | dialog |

Source `110` before snapshot:

| Field | Value |
| --- | ---: |
| item_count | 384 |
| telegram_message_count | 384 |
| topic_membership_count | 0 |
| reply_count | 306 |
| thread_count | 242 |
| reaction_item_count | 65 |

Source `110` after bounded run:

| Field | Value |
| --- | ---: |
| item_count | 12279 |
| telegram_message_count | 12279 |
| topic_membership_count | 0 |
| reply_count | 5128 |
| thread_count | 242 |
| reaction_item_count | 69 |

Explicit snapshot delta for source `110`:

| Field | Delta |
| --- | ---: |
| item_count | 11895 |
| telegram_message_count | 11895 |
| topic_membership_count | 0 |
| reply_count | 4822 |
| thread_count | 0 |
| reaction_item_count | 4 |

Source `110` after-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 171 |
| content_kind | text_only | 11734 |
| content_kind | text_with_media | 374 |
| media_kind | document | 10 |
| media_kind | image | 3 |
| media_kind | none | 11734 |
| media_kind | photo | 91 |
| media_kind | poll | 2 |
| media_kind | sticker | 84 |
| media_kind | video | 17 |
| media_kind | voice | 2 |
| media_kind | webpage | 336 |
| history_peer_kind | channel | 12279 |

Batch `5` summary:

| Batch id | Source id | Status | Completeness | Subtype | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages | Message count estimate |
| ---: | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 110 | cancelled | partial | supergroup | 11895 | 11895 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 77469 |

Batch `5` outcome counts:

| Outcome | Count |
| --- | ---: |
| inserted | 11895 |

Warning codes for batch `5`: none.

Latest durable recovery state for source `110`: `cancelled` / `partial`,
warning count `0`.

### 2026-05-23 Cancelled Takeout Watermark Observation

The cancelled bounded runs above did not capture `last_sync_state` /
`last_synced_at` before starting, so this note does not claim an exact
before/after equality proof. It records the post-cancellation source watermark
state and compares it to each cancelled Takeout batch time.

| Source id | Batch id | Batch status | Batch completeness | Batch started | Batch finished | Post-cancel last_sync_state | Post-cancel last_synced_at |
| ---: | ---: | --- | --- | --- | --- | ---: | --- |
| 21 | 4 | cancelled | partial | 2026-05-23 11:05:32 | 2026-05-23 11:08:17 | 179811 | 2026-05-03T16:48:05Z |
| 110 | 5 | cancelled | partial | 2026-05-23 11:28:18 | 2026-05-23 11:30:06 | 92374 | 2026-05-21T16:59:00Z |

Observation: both post-cancel `last_synced_at` values predate their cancelled
Takeout batch windows. This is consistent with partial rows being persisted
without advancing the normal source sync watermark on cancellation.

Follow-up: for the next bounded cancellation validation, capture
`last_sync_state` and `last_synced_at` immediately before starting the Takeout
job and immediately after terminalization, then record the exact before/after
watermark equality.

### 2026-05-23 Small Group Completed Live Takeout

App commit: `ce5c300`. Working tree was clean before this run.

Source `118` identity shape:

| Field | Before | After |
| --- | --- | --- |
| source_subtype | group | group |
| peer_kind | chat | chat |
| has_username | 0 | 0 |
| has_access_hash | 0 | 0 |
| resolution_strategy | dialog | dialog |
| last_sync_state | null | 241 |
| last_synced_at | null | 1779536615 |

Source `118` snapshot:

| Field | Before | After | Delta |
| --- | ---: | ---: | ---: |
| item_count | 0 | 2 | 2 |
| telegram_message_count | 0 | 2 | 2 |
| topic_membership_count | 0 | 0 | 0 |
| reply_count | 0 | 0 | 0 |
| thread_count | 0 | 0 | 0 |
| reaction_item_count | 0 | 0 | 0 |

Source `118` after-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | text_only | 2 |
| media_kind | none | 2 |
| history_peer_kind | chat | 2 |

Batch `6` summary:

| Batch id | Source id | Status | Completeness | Subtype | Started | Finished | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages | Message count estimate | Max message id |
| ---: | ---: | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 6 | 118 | completed | complete | group | 2026-05-23 11:43:33 | 2026-05-23 11:43:35 | 2 | 2 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 4 | 241 |

Batch `6` outcome counts:

| Outcome | Count |
| --- | ---: |
| inserted | 2 |

Warning codes for batch `6`: none.

Watermark observation: because this run completed, `last_sync_state` advanced
from `null` to `241`, matching the batch `max_message_id`. `last_synced_at`
advanced from `null` to `1779536615`, matching the completed job finish time.

### 2026-05-23 Normal Sync Before Takeout Attempt

App commit: `55e07bc`. Working tree was clean before this run.

Source `113` identity shape:

| Field | Value |
| --- | --- |
| source_subtype | channel |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| resolution_strategy | dialog |

Normal sync result before Takeout:

| Field | Value |
| --- | ---: |
| inserted | 1 |
| skipped | 0 |
| last_message_id | 515 |
| warning_count | 0 |

Source `113` snapshots:

| Field | Before normal sync | After normal sync | After Takeout attempt |
| --- | ---: | ---: | ---: |
| item_count | 28 | 29 | 29 |
| telegram_message_count | 28 | 29 | 29 |
| topic_membership_count | 0 | 0 | 0 |
| reply_count | 16 | 16 | 16 |
| thread_count | 5 | 5 | 5 |
| reaction_item_count | 21 | 22 | 22 |
| last_sync_state | 514 | 515 | 515 |
| last_synced_at | 1779414158 | 1779537575 | 1779537575 |

Source `113` after-sync aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 1 |
| content_kind | text_only | 8 |
| content_kind | text_with_media | 20 |
| media_kind | none | 8 |
| media_kind | photo | 6 |
| media_kind | webpage | 15 |

Batch `7` summary:

| Batch id | Source id | Status | Completeness | Subtype | Started | Finished | Inserted | Observed | Duplicates | Skipped | Warnings | Terminal error class | Used export DC | Fallback used | Migrated detected | Only my messages |
| ---: | ---: | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 7 | 113 | failed | unknown | channel | 2026-05-23 11:59:56 | 2026-05-23 11:59:57 | 0 | 0 | 0 | 0 | 0 | TAKEOUT_INIT_DELAY | 1 | 0 | 0 | 0 |

Batch `7` outcome counts: none.

Warning codes for batch `7`: none.

Result: this run proves the normal-sync-before-Takeout setup for source `113`
and records a clean failed Takeout attempt before observations. It does not
prove duplicate-after-normal-sync or row-fidelity comparison because Telegram
blocked Takeout session initialization with `TAKEOUT_INIT_DELAY`.
