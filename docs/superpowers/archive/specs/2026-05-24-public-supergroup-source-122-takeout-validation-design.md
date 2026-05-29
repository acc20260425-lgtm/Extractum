# Public Supergroup Source 122 Takeout Validation Design

## Goal

Run a controlled, sanitized public-supergroup Takeout validation for local
Telegram `source_id=122`.

This slice is validation-only. It should not change runtime code, mutate source
identity, decode private payloads, or record raw Telegram/provider data.

## Context

Sources `22` and `19` produced useful partial public-supergroup evidence, but
both were bounded-cancelled because their estimates were large. Source `122`
was added afterward and is now the strongest candidate for completed
public-supergroup evidence because it has a small normal-sync baseline.

Current sanitized candidate shape:

| Field | Value |
| --- | --- |
| source_id | 122 |
| source_subtype | supergroup |
| account_id | 1 |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 1 |
| resolution_strategy | dialog |
| last_sync_state | 12238 |
| last_synced_at | 1779640088 |
| item_count | 117 |
| telegram_message_count | 117 |
| max_telegram_message_id | 12238 |
| topic_membership_count | 0 |
| reply_count | 74 |
| thread_count | 53 |
| reaction_item_count | 18 |
| prior_takeout_batches | 0 |

Why source `122`:

- it is a newly added public/member supergroup with no prior Takeout batches;
- it has a very small normal-sync baseline compared with sources `19`, `21`,
  `22`, and `26`;
- it uses account `1`, which completed source `18` and produced partial
  evidence for sources `19` and `22`;
- it has reply, thread, reaction, and mixed media aggregates;
- it is a better candidate for a completed run than the previous large
  public-supergroup candidates.

Limitations:

- source `122` currently has no topic memberships or topic catalog state, so
  this slice should not close forum-topic behavior;
- a completed run can still close or strengthen `Public supergroup Takeout`
  if the captured before/after, batch, duplicate, row-fidelity, warning, and
  watermark evidence is complete.

## Approach

Use the same controlled pattern as prior Takeout validations:

1. Capture and commit pre-run sanitized source and durable Takeout baseline.
2. Pause for explicit live authorization before triggering Takeout.
3. Trigger the existing app flow for `source_id=122`.
4. Monitor only coarse terminal state and aggregate counters.
5. Continue to completion if the estimate/runtime stays reasonable; otherwise
   pause for user direction before bounded cancellation unless the app flow
   requires immediate safe cancellation.
6. Capture post-run source snapshot, batch summary, duplicate summary,
   row-fidelity comparison, warning visibility, and before/after delta.
7. Update the matrix and backlog only for rows supported by captured evidence.

## Safety Boundary

Allowed evidence:

- local numeric ids such as `source_id`, `account_id`, and `batch_id`;
- source subtype and peer kind;
- boolean identity flags such as `has_username`, `has_access_hash`, and
  `is_member`;
- aggregate counters;
- durable batch status, completeness, and warning codes;
- typed/coarse terminal error classes;
- source watermarks;
- source snapshot deltas;
- capped local sample ids from row-fidelity diagnostics if needed.

Forbidden evidence:

- message text;
- source titles;
- usernames;
- phone numbers;
- account labels identifying a person/source;
- session/auth material;
- headers/cookies;
- raw TL payloads;
- raw provider payloads;
- compressed dumps;
- warning message bodies;
- screenshots revealing private content;
- `sources.metadata_zstd` contents.

## Success Criteria

If Takeout completes cleanly:

- `Public supergroup Takeout` can be promoted to `passed`;
- the result note must include explicit before/after snapshots, batch summary,
  warning visibility, row-fidelity comparison, duplicate summary, and watermark
  behavior;
- `Forum-topic decision input` should remain unchanged unless this run
  unexpectedly produces direct topic catalog or membership evidence.

If Takeout is blocked, cancelled, partial, or fails:

- record only the supported outcome;
- keep `Public supergroup Takeout` as `needs follow-up`;
- do not infer shifted export DC or `CHANNEL_PRIVATE` fallback without exact
  warning code/flag evidence.

## Non-Goals

- Do not change runtime code.
- Do not add or modify Tauri commands.
- Do not delete or manually edit Takeout batches, observations, source rows, or
  item rows.
- Do not bypass Telegram `TAKEOUT_INIT_DELAY`.
- Do not mark shifted export DC fallback complete unless `export_dc_fallback`
  warning evidence exists.
- Do not mark `CHANNEL_PRIVATE` fallback complete from this public/member source
  unless a real `only_my_messages_fallback` warning/flag appears.
- Do not change forum-topic refresh behavior from this validation slice.
