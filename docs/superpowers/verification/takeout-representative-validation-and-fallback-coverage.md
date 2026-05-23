# Takeout Representative Validation And Fallback Coverage

> Status: reusable manual validation matrix. Rows start as `not run` until real
> Telegram accounts and representative sources are available.

Updated: 2026-05-23

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
| Public supergroup Takeout | not run |  |  | before/after source summary, Takeout batch summary, topic/reply/thread aggregate shape, warning visibility |  |
| Private or dialog-backed supergroup Takeout | not run |  |  | before/after source summary, fallback/warning evidence if applicable |  |
| Small group Takeout | not run |  |  | source subtype and peer-kind shape, before/after source summary, batch summary |  |
| Repeated Takeout after normal sync | not run |  |  | `duplicate_after_normal_sync` summary and row-fidelity comparison |  |
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
