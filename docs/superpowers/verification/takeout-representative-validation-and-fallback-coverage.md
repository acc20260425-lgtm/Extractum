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
| Public channel Takeout | not run |  |  | before/after source summary, Takeout batch summary, duplicate summary, warning visibility |  |
| Public supergroup Takeout | not run |  |  | before/after source summary, Takeout batch summary, topic/reply/thread aggregate shape, warning visibility |  |
| Private or dialog-backed supergroup Takeout | not run |  |  | before/after source summary, fallback/warning evidence if applicable |  |
| Small group Takeout | not run |  |  | source subtype and peer-kind shape, before/after source summary, batch summary |  |
| Repeated Takeout after normal sync | not run |  |  | `duplicate_after_normal_sync` summary and row-fidelity comparison |  |
| Repeated Takeout after previous Takeout | not run |  |  | duplicate observation summary and latest batch summary |  |
| `CHANNEL_PRIVATE` fallback | not run |  |  | `only_my_messages_fallback` warning code, partial/incomplete evidence, typed/coarse terminal outcome if present |  |
| Shifted export DC fallback | blocked |  |  | export DC attempted/fallback flags, `export_dc_fallback` warning code, typed/coarse terminal outcome if present | Requires an environment that naturally triggers local transport/session fallback |
| Migrated small-group-to-supergroup smoke | not run |  |  | migrated-history detected, `migrated_history_deferred`, partial completeness, no old `chat` history imported |  |
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
