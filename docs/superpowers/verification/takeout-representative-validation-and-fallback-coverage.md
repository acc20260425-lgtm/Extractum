# Takeout Representative Validation And Fallback Coverage

> Status: reusable manual validation matrix. Rows start as `not run` until real
> Telegram accounts and representative sources are available.

Updated: 2026-05-24

Current matrix summary: `5 passed`, `2 needs follow-up`, `2 blocked`,
`1 not run`.

Covered highlights:

- completed public-channel Takeout for source `18` / batch `10`, with explicit
  before/after snapshots, duplicate-after-normal-sync evidence, and row-fidelity
  comparison;
- completed small-group Takeout for source `118` / batch `6`;
- completed repeated Takeout-after-Takeout duplicate validation for source `73`
  / batch `3`;
- completed public-supergroup Takeout for source `122` / batch `13`, with
  explicit before/after snapshots, duplicate-after-normal-sync evidence, and
  row-fidelity comparison;
- bounded partial public supergroup and dialog-backed no-username supergroup
  runs for sources `19` / `21` / `22` / `110`;
- normal-sync-before-Takeout attempts for source `113`, most recently batch
  `9`, blocked by
  `TAKEOUT_INIT_DELAY` before Takeout observations were written; source `18`
  later completed the public-channel after-normal-sync validation path;
- post-cancel watermark observations for cancelled partial batches `4` and `5`;
- forum-topic decision input from source `21` / batch `4` aggregate catalog,
  membership, and resolver-state counters.

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
| Public channel Takeout | passed | 18 | 10 | before/after source summary, completed Takeout batch summary, duplicate summary, row-fidelity comparison, warning visibility, watermark before/after | Completed cleanly for a public/member channel with explicit before/after snapshots and no warnings |
| Public supergroup Takeout | passed | 122 | 13 | before/after source summary, completed Takeout batch summary, duplicate summary, warning visibility, row-fidelity comparison | Completed cleanly for a public/member supergroup with explicit before/after snapshots, duplicate evidence against the normal-sync baseline, full row-fidelity match, and no warnings |
| Private or dialog-backed supergroup Takeout | needs follow-up | 110 | 5 | before/after source summary, cancelled partial Takeout batch summary, warning visibility | Dialog-backed no-username supergroup path imported partial history and was cancelled before full completion because the Takeout estimate was large |
| Small group Takeout | passed | 118 | 6 | source subtype and peer-kind shape, before/after source summary, batch summary, watermark before/after | Completed cleanly for a dialog-backed `group` / `chat` source with no username or access hash |
| Repeated Takeout after normal sync | passed | 18 | 10 | duplicate observation summary, row-fidelity comparison, before/after source summary, latest batch summary | Batch 10 followed an existing normal-sync baseline for source 18; 42 observations were classified as duplicates and all 467 observed identities matched canonical rows |
| Repeated Takeout after previous Takeout | passed | 73 | 3 | duplicate observation summary and latest batch summary | Batch 3 followed prior Takeout batch 1 for the same source; latest batch completed with all observations classified as duplicates |
| `CHANNEL_PRIVATE` fallback | not run |  |  | `only_my_messages_fallback` warning code, partial/incomplete evidence, typed/coarse terminal outcome if present | Offline inventory found no prior fallback evidence; a live `CHANNEL_PRIVATE` observation is still needed |
| Shifted export DC fallback | blocked |  |  | export DC attempted/fallback flags, `export_dc_fallback` warning code, typed/coarse terminal outcome if present | Requires an environment that naturally triggers local transport/session fallback |
| Migrated small-group-to-supergroup smoke | blocked | 115 | 2 | failed early Takeout batch summary | Existing smoke reached a failed/unknown terminal batch with zero observations before migrated-history evidence could be collected |
| Forum-topic decision input | needs follow-up | 22 | 11 | topic catalog/membership aggregate counters from bounded partial Takeout | Bounded partial runs materially increased topic memberships without refreshing the topic catalog, which is useful decision input; completed supergroup Takeout evidence is still needed before changing behavior |

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

### 2026-05-24 Source 122 Public Supergroup Takeout Pre-Run

App commit: `aea8ffd`. Working tree was clean before this run on branch
`takeout-source-122-public-supergroup-validation-plan`.

Source `122` pre-run identity shape:

| Field | Value |
| --- | --- |
| source_type | telegram |
| source_subtype | supergroup |
| account_id | 1 |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 1 |
| resolution_strategy | dialog |

Source `122` pre-run snapshot:

| Field | Value |
| --- | ---: |
| item_count | 117 |
| telegram_message_count | 117 |
| topic_membership_count | 0 |
| topic_membership_topic_count | 0 |
| reply_count | 74 |
| thread_count | 53 |
| reaction_item_count | 18 |
| reaction_count_sum | 23 |
| content_zstd_present_count | 115 |
| max_telegram_message_id | 12238 |
| last_sync_state | 12238 |
| last_synced_at | 1779640088 |

Source `122` pre-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 2 |
| content_kind | text_only | 101 |
| content_kind | text_with_media | 14 |
| media_kind | none | 101 |
| media_kind | photo | 6 |
| media_kind | video | 1 |
| media_kind | webpage | 9 |
| history_peer_kind | channel | 117 |

Source `122` pre-run topic catalog aggregate:

| Field | Value |
| --- | ---: |
| topic_catalog_count | 0 |
| distinct_topic_ids | 0 |
| closed_count | 0 |
| pinned_count | 0 |
| hidden_count | 0 |
| deleted_count | 0 |
| max_last_seen_at | null |
| max_updated_at | null |

Source `122` pre-run topic resolver state: none.

Latest pre-run Takeout state for source `122`: none.

Prior Takeout batch count for source `122`: `0`.

Warning codes for prior source `122` Takeout batches: none.

### 2026-05-24 Source 122 Public Supergroup Takeout Result

App commit: `aea8ffd`. Working tree was clean before this run on branch
`takeout-source-122-public-supergroup-validation-plan`.

Outcome: completed / complete.

Source `122` before/after snapshot:

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

Source `122` after-run aggregate distributions:

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

Batch summary:

| Batch id | Source id | Status | Completeness | Terminal error present | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages | Message count estimate | Max message id |
| ---: | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 13 | 122 | completed | complete | 0 | 4447 | 4564 | 117 | 0 | 0 | 1 | 0 | 0 | 0 | 5768 | null |

Warning codes for batch `13`: none.

Duplicate summary:

| Field | Value |
| --- | ---: |
| inserted observations | 4447 |
| duplicate observations | 117 |
| skipped observations | 0 |
| failed observations | 0 |
| duplicate identity count | 117 |
| has duplicate-after-normal-sync evidence | 1 |

Row-fidelity comparison:

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

Matched content-kind distribution:

| Key | Count |
| --- | ---: |
| media_only | 314 |
| text_only | 3815 |
| text_with_media | 435 |

Matched media-kind distribution:

| Key | Count |
| --- | ---: |
| animation | 1 |
| document | 33 |
| image | 23 |
| none | 3815 |
| photo | 305 |
| poll | 3 |
| sticker | 151 |
| video | 13 |
| webpage | 220 |

Row-fidelity mismatch categories: none.

Warning visibility:

| Field | Value |
| --- | --- |
| provenance warning codes | none |
| recovery candidate warning codes | none |
| latest batch for source | yes |
| durable recovery kind | none |

Result: the public-supergroup Takeout row is now `passed` because source `122`
/ batch `13` completed cleanly with duplicate evidence against the normal-sync
baseline, a full canonical row-fidelity match, warning visibility, and explicit
before/after snapshots. `Forum-topic decision input` remains unchanged because
source `122` had no topic membership or topic catalog evidence.

### 2026-05-24 Source 19 Public Supergroup Takeout Pre-Run

App commit: `18c2cb8`. Working tree was clean before this run on branch
`takeout-source-19-public-supergroup-validation-plan`.

Source `19` pre-run identity shape:

| Field | Value |
| --- | --- |
| source_type | telegram |
| source_subtype | supergroup |
| account_id | 1 |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 1 |
| resolution_strategy | dialog |

Source `19` pre-run snapshot:

| Field | Value |
| --- | ---: |
| item_count | 1950 |
| telegram_message_count | 1950 |
| topic_membership_count | 0 |
| topic_membership_topic_count | 0 |
| reply_count | 920 |
| thread_count | 551 |
| reaction_item_count | 249 |
| reaction_count_sum | 461 |
| content_zstd_present_count | 1922 |
| max_telegram_message_id | 63721 |
| last_sync_state | 63721 |
| last_synced_at | 1777826899 |

Source `19` pre-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 28 |
| content_kind | text_only | 1807 |
| content_kind | text_with_media | 115 |
| media_kind | document | 1 |
| media_kind | none | 1807 |
| media_kind | photo | 60 |
| media_kind | sticker | 7 |
| media_kind | video | 4 |
| media_kind | voice | 1 |
| media_kind | webpage | 70 |
| history_peer_kind | channel | 1950 |

Source `19` pre-run topic catalog aggregate:

| Field | Value |
| --- | ---: |
| topic_catalog_count | 0 |
| distinct_topic_ids | 0 |
| closed_count | 0 |
| pinned_count | 0 |
| hidden_count | 0 |
| deleted_count | 0 |
| max_last_seen_at | null |
| max_updated_at | null |

Source `19` pre-run topic resolver state:

| Field | Value |
| --- | --- |
| resolver_version | 1 |
| status | never_run |
| catalog_refreshed_at | null |
| memberships_refreshed_at | null |
| unresolved_count | 0 |
| pending_item_count | 0 |
| has_last_error | 0 |
| updated_at | 1779038483 |

Latest pre-run Takeout state for source `19`: none.

Prior Takeout batch count for source `19`: `0`.

Warning codes for prior source `19` Takeout batches: none.

### 2026-05-24 Source 19 Public Supergroup Takeout Result

App commit: `18c2cb8`. Working tree was clean before this run on branch
`takeout-source-19-public-supergroup-validation-plan`.

Outcome: cancelled / partial.

Source `19` before/after snapshot:

| Field | Before | After | Delta |
| --- | ---: | ---: | ---: |
| item_count | 1950 | 22347 | 20397 |
| telegram_message_count | 1950 | 22347 | 20397 |
| topic_membership_count | 0 | 0 | 0 |
| reply_count | 920 | 10509 | 9589 |
| thread_count | 551 | 5858 | 5307 |
| reaction_item_count | 249 | 3558 | 3309 |
| last_sync_state | 63721 | 63721 | unchanged |
| last_synced_at | 1777826899 | 1777826899 | unchanged |

Source `19` after-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 275 |
| content_kind | text_only | 20601 |
| content_kind | text_with_media | 1471 |
| media_kind | document | 19 |
| media_kind | image | 15 |
| media_kind | none | 20601 |
| media_kind | photo | 647 |
| media_kind | poll | 4 |
| media_kind | sticker | 39 |
| media_kind | video | 30 |
| media_kind | voice | 1 |
| media_kind | webpage | 991 |
| history_peer_kind | channel | 22347 |

Source `19` topic catalog and resolver state after batch `12`:

| Field | Value |
| --- | --- |
| topic_catalog_count | 0 |
| distinct_topic_ids | 0 |
| resolver_version | 1 |
| resolver_status | never_run |
| catalog_refreshed_at | null |
| memberships_refreshed_at | null |
| unresolved_count | 0 |
| pending_item_count | 0 |
| has_last_error | 0 |
| resolver_updated_at | 1779038483 |

Batch summary:

| Batch id | Source id | Status | Completeness | Terminal error present | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages | Message count estimate | Max message id |
| ---: | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 12 | 19 | cancelled | partial | 0 | 20397 | 20397 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 52723 | null |

Warning codes for batch `12`: none.

Duplicate summary:

| Field | Value |
| --- | ---: |
| inserted observations | 20397 |
| duplicate observations | 0 |
| skipped observations | 0 |
| failed observations | 0 |
| duplicate identity count | 0 |
| has duplicate-after-normal-sync evidence | 0 |

Row-fidelity comparison:

| Field | Value |
| --- | ---: |
| observed_identity_count | 20397 |
| matched_canonical_identity_count | 20397 |
| missing_canonical_identity_count | 0 |
| canonical_without_observation_count | 1950 |
| matched_content_zstd_present_count | 20150 |
| matched_reply_to_msg_id_present_count | 9589 |
| matched_reply_to_top_id_present_count | 5307 |
| matched_reaction_count_present_count | 3309 |

Matched content-kind distribution:

| Key | Count |
| --- | ---: |
| media_only | 247 |
| text_only | 18794 |
| text_with_media | 1356 |

Matched media-kind distribution:

| Key | Count |
| --- | ---: |
| document | 18 |
| image | 15 |
| none | 18794 |
| photo | 587 |
| poll | 4 |
| sticker | 32 |
| video | 26 |
| webpage | 921 |

Row-fidelity mismatch categories:

| Category | Count | Sample ids |
| --- | ---: | --- |
| canonical_identity_missing_observation | 1950 | 43079, 43080, 43081, 43082, 43083, 43084, 43085, 43086, 43087, 43088 |

Interpretation: all observed Takeout identities matched canonical source rows.
The canonical rows without observations are the pre-run normal-sync baseline
rows, which is expected in this bounded partial run.

Warning visibility:

| Field | Value |
| --- | --- |
| provenance warning codes | none |
| recovery candidate warning codes | none |
| latest batch for source | yes |
| durable recovery kind | cancelled |

Result: the public-supergroup Takeout row remains `needs follow-up` because
the run was bounded-cancelled as partial. The row now points to source `19` /
batch `12` as the latest public-supergroup evidence. `Forum-topic decision
input` remains unchanged because source `19` had no topic membership or topic
catalog evidence.

### 2026-05-24 Source 22 Public Supergroup Takeout Pre-Run

App commit: `e54b61c`. Working tree was clean before this run on branch
`takeout-public-supergroup-validation-plan`.

Source `22` pre-run identity shape:

| Field | Value |
| --- | --- |
| source_type | telegram |
| source_subtype | supergroup |
| account_id | 1 |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 1 |
| resolution_strategy | dialog |

Source `22` pre-run snapshot:

| Field | Value |
| --- | ---: |
| item_count | 1137 |
| telegram_message_count | 1137 |
| topic_membership_count | 1136 |
| topic_membership_topic_count | 21 |
| reply_count | 1133 |
| thread_count | 760 |
| reaction_item_count | 115 |
| reaction_count_sum | 118 |
| content_zstd_present_count | 1102 |
| max_telegram_message_id | 157979 |
| last_sync_state | 157979 |
| last_synced_at | 1777917482 |

Source `22` pre-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 35 |
| content_kind | text_only | 997 |
| content_kind | text_with_media | 105 |
| media_kind | document | 10 |
| media_kind | image | 5 |
| media_kind | none | 997 |
| media_kind | photo | 104 |
| media_kind | webpage | 21 |
| history_peer_kind | channel | 1137 |
| membership_match_kind | general_fallback | 5 |
| membership_match_kind | reply_to_msg_id | 365 |
| membership_match_kind | reply_to_top_id | 759 |
| membership_match_kind | typed_root_top_message_id | 7 |

Source `22` pre-run topic catalog aggregate:

| Field | Value |
| --- | ---: |
| topic_catalog_count | 23 |
| distinct_topic_ids | 23 |
| closed_count | 0 |
| pinned_count | 0 |
| hidden_count | 0 |
| deleted_count | 0 |
| max_last_seen_at | 1777917481 |
| max_updated_at | 1777917481 |

Source `22` pre-run topic resolver state:

| Field | Value |
| --- | --- |
| resolver_version | 1 |
| status | ready |
| catalog_refreshed_at | 1777917481 |
| memberships_refreshed_at | 1779038483 |
| unresolved_count | 1 |
| pending_item_count | 0 |
| has_last_error | 0 |
| updated_at | 1779038483 |

Latest pre-run Takeout state for source `22`: none.

Prior Takeout batch count for source `22`: `0`.

### 2026-05-24 Source 22 Public Supergroup Takeout Result

App commit: `e54b61c`. Working tree was clean before this run on branch
`takeout-public-supergroup-validation-plan`.

Outcome: cancelled / partial.

Source `22` before/after snapshot:

| Field | Before | After | Delta |
| --- | ---: | ---: | ---: |
| item_count | 1137 | 12299 | 11162 |
| telegram_message_count | 1137 | 12299 | 11162 |
| topic_membership_count | 1136 | 11166 | 10030 |
| reply_count | 1133 | 5973 | 4840 |
| thread_count | 760 | 1892 | 1132 |
| reaction_item_count | 115 | 269 | 154 |
| last_sync_state | 157979 | 157979 | unchanged |
| last_synced_at | 1777917482 | 1777917482 | unchanged |

Source `22` after-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 306 |
| content_kind | text_only | 11550 |
| content_kind | text_with_media | 443 |
| media_kind | document | 26 |
| media_kind | image | 30 |
| media_kind | location | 1 |
| media_kind | none | 11550 |
| media_kind | photo | 483 |
| media_kind | poll | 1 |
| media_kind | sticker | 9 |
| media_kind | video | 6 |
| media_kind | voice | 1 |
| media_kind | webpage | 192 |
| history_peer_kind | channel | 12299 |
| membership_match_kind | general_fallback | 10035 |
| membership_match_kind | reply_to_msg_id | 365 |
| membership_match_kind | reply_to_top_id | 759 |
| membership_match_kind | typed_root_top_message_id | 7 |

Source `22` topic catalog and resolver state after batch `11`:

| Field | Before | After |
| --- | ---: | ---: |
| topic_catalog_count | 23 | 23 |
| distinct_topic_ids | 23 | 23 |
| closed_count | 0 | 0 |
| pinned_count | 0 | 0 |
| hidden_count | 0 | 0 |
| deleted_count | 0 | 0 |
| max_topic_last_seen_at | 1777917481 | 1777917481 |
| max_topic_updated_at | 1777917481 | 1777917481 |
| resolver_unresolved_count | 1 | 1133 |
| resolver_pending_item_count | 0 | 0 |
| resolver_has_last_error | 0 | 0 |

Batch summary:

| Batch id | Source id | Status | Completeness | Terminal error present | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages | Message count estimate | Max message id |
| ---: | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 11 | 22 | cancelled | partial | 0 | 11162 | 11162 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 125813 | null |

Warning codes for batch `11`: none.

Duplicate summary:

| Field | Value |
| --- | ---: |
| inserted observations | 11162 |
| duplicate observations | 0 |
| skipped observations | 0 |
| failed observations | 0 |
| duplicate identity count | 0 |
| has duplicate-after-normal-sync evidence | 0 |

Row-fidelity comparison:

| Field | Value |
| --- | ---: |
| observed_identity_count | 11162 |
| matched_canonical_identity_count | 11162 |
| missing_canonical_identity_count | 0 |
| canonical_without_observation_count | 1137 |
| matched_content_zstd_present_count | 10891 |
| matched_reply_to_msg_id_present_count | 4840 |
| matched_reply_to_top_id_present_count | 1132 |
| matched_reaction_count_present_count | 154 |

Matched content-kind distribution:

| Key | Count |
| --- | ---: |
| media_only | 271 |
| text_only | 10553 |
| text_with_media | 338 |

Matched media-kind distribution:

| Key | Count |
| --- | ---: |
| document | 16 |
| image | 25 |
| location | 1 |
| none | 10553 |
| photo | 379 |
| poll | 1 |
| sticker | 9 |
| video | 6 |
| voice | 1 |
| webpage | 171 |

Row-fidelity mismatch categories:

| Category | Count | Sample ids |
| --- | ---: | --- |
| canonical_identity_missing_observation | 1137 | 48335, 48336, 48337, 48338, 48339, 48340, 48341, 48342, 48343, 48344 |

Interpretation: all observed Takeout identities matched canonical source rows.
The canonical rows without observations are the pre-run normal-sync baseline
rows, which is expected in this bounded partial run.

Warning visibility:

| Field | Value |
| --- | --- |
| provenance warning codes | none |
| recovery candidate warning codes | none |
| latest batch for source | yes |
| durable recovery kind | cancelled |

Result: the public-supergroup Takeout row remains `needs follow-up` because
batch `11` was cancelled / partial, but the run strengthens partial
public-supergroup evidence with row-fidelity, warning visibility, and explicit
watermark behavior. The forum-topic decision input row remains
`needs follow-up`: the partial run added `10030` topic memberships while the
topic catalog aggregate stayed unchanged.

### 2026-05-24 Source 18 Public Channel Takeout Pre-Run

App commit: `2502923`. Working tree was clean before this run on branch
`takeout-source-18-public-channel-validation`.

Source `18` pre-run identity shape:

| Field | Value |
| --- | --- |
| source_type | telegram |
| source_subtype | channel |
| account_id | 1 |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 1 |
| resolution_strategy | dialog |

Source `18` pre-run snapshot:

| Field | Value |
| --- | ---: |
| item_count | 42 |
| telegram_message_count | 42 |
| topic_membership_count | 0 |
| topic_membership_topic_count | 0 |
| reply_count | 20 |
| thread_count | 6 |
| reaction_item_count | 28 |
| reaction_count_sum | 128 |
| content_zstd_present_count | 37 |
| max_telegram_message_id | 514 |
| last_sync_state | 514 |
| last_synced_at | 1779414142 |

Source `18` pre-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 5 |
| content_kind | text_only | 12 |
| content_kind | text_with_media | 25 |
| media_kind | none | 12 |
| media_kind | photo | 13 |
| media_kind | webpage | 17 |
| history_peer_kind | channel | 42 |

Latest pre-run Takeout state for source `18`: none.

Prior Takeout batch count for source `18`: `0`.

### 2026-05-24 Source 18 Public Channel Takeout Result

App commit: `2502923`. Working tree was clean before this run on branch
`takeout-source-18-public-channel-validation`.

Outcome: completed / complete.

Source `18` before/after snapshot:

| Field | Before | After | Delta |
| --- | ---: | ---: | ---: |
| item_count | 42 | 467 | 425 |
| telegram_message_count | 42 | 467 | 425 |
| topic_membership_count | 0 | 0 | 0 |
| reply_count | 20 | 155 | 135 |
| thread_count | 6 | 38 | 32 |
| reaction_item_count | 28 | 317 | 289 |
| last_sync_state | 514 | 515 | advanced |
| last_synced_at | 1779414142 | 1779627419 | advanced |

Source `18` after-run aggregate distributions:

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

Batch summary:

| Batch id | Source id | Status | Completeness | Terminal error present | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages | Message count estimate | Max message id |
| ---: | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 18 | completed | complete | 0 | 425 | 467 | 42 | 0 | 0 | 1 | 0 | 0 | 0 | 475 | 515 |

Warning codes for batch `10`: none.

Duplicate summary:

| Field | Value |
| --- | ---: |
| inserted observations | 425 |
| duplicate observations | 42 |
| skipped observations | 0 |
| failed observations | 0 |
| duplicate identity count | 42 |
| has duplicate-after-normal-sync evidence | 1 |

Row-fidelity comparison:

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

Matched content-kind distribution:

| Key | Count |
| --- | ---: |
| media_only | 47 |
| text_only | 164 |
| text_with_media | 256 |

Matched media-kind distribution:

| Key | Count |
| --- | ---: |
| animation | 1 |
| document | 8 |
| none | 164 |
| photo | 70 |
| poll | 1 |
| video | 5 |
| webpage | 218 |

Row-fidelity mismatch categories: none.

Warning visibility:

| Field | Value |
| --- | --- |
| provenance warning codes | none |
| recovery candidate warning codes | none |
| latest batch for source | yes |

Result: the public-channel Takeout row is promoted to `passed`, and the
Takeout-after-normal-sync comparison row is promoted to `passed` because batch
`10` completed with duplicate evidence against the normal-sync baseline and a
full canonical row-fidelity match.

### 2026-05-23 Source 113 Takeout Retry Pre-Run

App commit: `c2c7e4c`. Working tree was clean before this run on branch
`takeout-source-113-retry-validation`.

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
| item_count | 29 |
| telegram_message_count | 29 |
| topic_membership_count | 0 |
| reply_count | 16 |
| thread_count | 5 |
| reaction_item_count | 22 |
| reaction_count_sum | 86 |
| content_zstd_present_count | 28 |
| max_telegram_message_id | 515 |
| last_sync_state | 515 |
| last_synced_at | 1779537575 |

Source `113` pre-run aggregate distributions:

| Distribution | Key | Count |
| --- | --- | ---: |
| content_kind | media_only | 1 |
| content_kind | text_only | 8 |
| content_kind | text_with_media | 20 |
| media_kind | none | 8 |
| media_kind | photo | 6 |
| media_kind | webpage | 15 |
| history_peer_kind | channel | 29 |

Latest pre-run Takeout state for source `113`:

| Batch id | Status | Completeness | Terminal error class | Observed | Inserted | Duplicates | Skipped | Warnings |
| ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 8 | failed | unknown | TAKEOUT_INIT_DELAY | 0 | 0 | 0 | 0 | 0 |

Pre-run warning codes for batch `8`: none.

### 2026-05-23 Source 113 Takeout Retry Result

App commit: `c2c7e4c`. Working tree was clean before this run on branch
`takeout-source-113-retry-validation`.

Outcome: `TAKEOUT_INIT_DELAY`.

Source `113` before/after snapshot:

| Field | Before | After | Delta |
| --- | ---: | ---: | ---: |
| item_count | 29 | 29 | 0 |
| telegram_message_count | 29 | 29 | 0 |
| topic_membership_count | 0 | 0 | 0 |
| reply_count | 16 | 16 | 0 |
| thread_count | 5 | 5 | 0 |
| reaction_item_count | 22 | 22 | 0 |
| last_sync_state | 515 | 515 | unchanged |
| last_synced_at | 1779537575 | 1779537575 | unchanged |

Batch summary:

| Batch id | Source id | Status | Completeness | Terminal error class | Inserted | Observed | Duplicates | Skipped | Warnings | Used export DC | Fallback used | Migrated detected | Only my messages |
| ---: | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 9 | 113 | failed | unknown | TAKEOUT_INIT_DELAY | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |

Warning codes for batch `9`: none.

Duplicate summary: not applicable because the batch wrote zero observations.

Row-fidelity comparison: not applicable because the batch wrote zero
observations.

Result: the repeated Takeout-after-normal-sync row remains `blocked`, and the
`CHANNEL_PRIVATE` fallback row remains `not run` because batch `9` failed
before observations or fallback warnings.

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

### 2026-05-23 Forum-Topic Decision Input From Partial Takeout

App commit: `9967932`. Working tree was clean before this note.

This note uses source `21` / batch `4` because it is the live bounded public
supergroup Takeout run that materially changed topic membership aggregates. The
batch was `cancelled` / `partial`, so this is decision input only; it does not
prove final behavior for a completed supergroup Takeout.

Batch `4` window and item counters:

| Field | Value |
| --- | ---: |
| started_unix | 1779534332 |
| finished_unix | 1779534497 |
| inserted | 15710 |
| observed | 15710 |
| message_count_estimate | 132886 |

Source `21` topic catalog after batch `4`:

| Field | Value |
| --- | ---: |
| topic_catalog_count | 10 |
| distinct_topic_ids | 10 |
| closed_count | 1 |
| pinned_count | 2 |
| hidden_count | 0 |
| deleted_count | 0 |
| topics_updated_in_batch_window | 0 |
| topics_seen_in_batch_window | 0 |

Source `21` topic resolver state after batch `4`:

| Field | Value |
| --- | --- |
| resolver_version | 1 |
| status | ready |
| catalog_refreshed_at | 1777826884 |
| memberships_refreshed_at | 1779038483 |
| unresolved_count | 7200 |
| pending_item_count | 0 |
| has_last_error | 0 |
| updated_at | 1779534497 |

Source `21` topic memberships:

| Field | Before batch `4` | After batch `4` | Delta |
| --- | ---: | ---: | ---: |
| membership_count | 370 | 8885 | 8515 |
| distinct_membership_topics |  | 7 |  |
| created_in_batch_window |  | 8515 |  |
| updated_in_batch_window |  | 8515 |  |

Membership match-kind distribution after batch `4`:

| Match kind | Count |
| --- | ---: |
| general_fallback | 8530 |
| reply_to_top_id | 264 |
| reply_to_msg_id | 90 |
| typed_root_top_message_id | 1 |

Batch `4` observed item membership shape:

| Field | Value |
| --- | ---: |
| observed_with_item_id | 15710 |
| observed_items_with_membership | 8515 |
| observed_distinct_membership_topics | 1 |

Observation: the partial Takeout run added `8515` topic memberships while the
forum-topic catalog had `0` rows updated or seen during the batch window. The
resolver state remained `ready` with `pending_item_count=0`, but the source
still has unresolved items. This supports a later product decision about
whether completed Takeout should refresh topic catalog state, but does not
justify changing behavior from this validation slice alone.

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

### 2026-05-23 Normal Sync Before Takeout Retry

App commit: `5def5fc`. Working tree was clean before this retry.

Source `113` before retry matched the previous post-normal-sync state:

| Field | Value |
| --- | ---: |
| item_count | 29 |
| telegram_message_count | 29 |
| reply_count | 16 |
| thread_count | 5 |
| reaction_item_count | 22 |
| last_sync_state | 515 |
| last_synced_at | 1779537575 |

Batch `8` summary:

| Batch id | Source id | Status | Completeness | Subtype | Started | Finished | Inserted | Observed | Duplicates | Skipped | Warnings | Terminal error class | Used export DC | Fallback used | Migrated detected | Only my messages |
| ---: | ---: | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 8 | 113 | failed | unknown | channel | 2026-05-23 12:18:47 | 2026-05-23 12:18:49 | 0 | 0 | 0 | 0 | 0 | TAKEOUT_INIT_DELAY | 1 | 0 | 0 | 0 |

Batch `8` outcome counts: none.

Warning codes for batch `8`: none.

Result: this retry remained blocked before observations with
`TAKEOUT_INIT_DELAY`. The `Repeated Takeout after normal sync` row stays
`blocked`.

### 2026-05-23 `CHANNEL_PRIVATE` Fallback Candidate Inventory

App commit: `d369158`. Working tree was clean before this inventory.

The fallback only applies to `channel` and `supergroup` sources when Telegram
returns `CHANNEL_PRIVATE` during history loading. This inventory used local DB
identity flags and previous Takeout provenance only; it did not call Telegram
and cannot prove that a source will trigger `CHANNEL_PRIVATE`.

Prior fallback evidence in local Takeout provenance:

| Field | Count |
| --- | ---: |
| sources_with_prior_only_my_messages | 0 |
| sources_with_prior_only_my_messages_warning | 0 |
| sources_with_prior_channel_private_terminal_error | 0 |

Most relevant sanitized candidates:

| Source id | Subtype | Peer kind | Has username | Has access hash | Is member | Items | Prior Takeout batches | Prior Takeout delay |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 113 | channel | channel | 1 | 1 | 0 | 29 | 2 | 1 |
| 114 | channel | channel | 0 | 1 | 1 | 0 | 0 | 0 |
| 115 | supergroup | channel | 0 | 1 | 1 | 1 | 1 | 1 |
| 27 | channel | channel | 0 | 1 | 1 | 1052 | 0 | 0 |
| 110 | supergroup | channel | 0 | 1 | 1 | 12279 | 1 | 0 |

Observation: source `113` is the strongest private/left-shape candidate because
it is a `channel` with `is_member=0`, but recent Takeout attempts for that
source are blocked before observations by `TAKEOUT_INIT_DELAY`. The other
no-username channel/supergroup sources have access hashes but local membership
flags do not indicate private-history failure. The matrix row remains `not run`
until a live Takeout reaches a `CHANNEL_PRIVATE` fallback warning or a typed
terminal outcome.
