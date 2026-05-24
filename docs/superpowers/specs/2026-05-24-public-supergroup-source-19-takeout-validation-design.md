# Public Supergroup Source 19 Takeout Validation Design

## Goal

Run a controlled, sanitized public-supergroup Takeout validation for local
Telegram `source_id=19`.

This slice is validation-only. It should not change runtime code, mutate source
identity, decode private payloads, or record raw Telegram/provider data.

## Context

Source `22` / batch `11` produced useful partial public-supergroup evidence,
but the run was bounded-cancelled after `11162` observations because the
message count estimate was `125813`. Completed public-supergroup evidence is
still open.

A read-only sanitized inventory selected `source_id=19` as the next candidate:

| Field | Value |
| --- | --- |
| source_subtype | supergroup |
| account_id | 1 |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 1 |
| resolution_strategy | dialog |
| last_sync_state | 63721 |
| last_synced_at | 1777826899 |
| item_count | 1950 |
| telegram_message_count | 1950 |
| max_telegram_message_id | 63721 |
| topic_membership_count | 0 |
| reply_count | 920 |
| thread_count | 551 |
| reaction_item_count | 249 |
| prior_takeout_batches | 0 |

Why source `19`:

- it is the smallest current public/member supergroup candidate with no prior
  Takeout batches by `max_telegram_message_id`;
- it uses account `1`, which completed source `18` successfully and produced
  source `22` partial evidence;
- it has reply, thread, reaction, and mixed media aggregates;
- it avoids immediately retrying source `22`, whose estimate was large;
- it avoids account `11`, which has recent `TAKEOUT_INIT_DELAY` history.

Limitations:

- source `19` currently has no topic memberships and the topic resolver state
  is `never_run`, so this slice should not close forum-topic behavior;
- a completed run can still close or strengthen `Public supergroup Takeout`
  if the captured before/after, batch, duplicate, row-fidelity, warning, and
  watermark evidence is complete.

## Approach

Use the same controlled pattern as source `18` and source `22`:

1. Capture and commit pre-run sanitized source and durable Takeout baseline.
2. Pause for explicit live authorization before triggering Takeout.
3. Trigger the existing app flow for `source_id=19`.
4. Monitor only coarse terminal state and aggregate counters.
5. Perform bounded cancellation through the normal app flow if the estimate or
   runtime is impractical for the session.
6. Capture post-run source snapshot, batch summary, duplicate summary,
   row-fidelity comparison, warning visibility, and before/after delta.
7. Update the matrix and backlog only for rows supported by captured evidence.

## Alternatives Considered

Recommended: source `19`.

Tradeoff: it is less topic-rich than source `17` or source `23`, but it has the
lowest current size proxy among public/member supergroups with no prior
Takeout batches.

Alternative: source `17`.

Tradeoff: it has topic membership evidence and no prior Takeout batches, but
its size proxy is higher (`max_telegram_message_id = 101949`), making bounded
cancellation more likely.

Alternative: source `23`.

Tradeoff: it is topic-rich and has no prior Takeout batches, but its size proxy
is close to source `22` (`max_telegram_message_id = 152627`), so it is a weaker
candidate for a completed run.

Alternative: retry source `22`.

Tradeoff: it already has strong partial evidence, but its latest estimate
(`125813`) makes an immediate completed retry impractical for a bounded
session.

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
- `Forum-topic decision input` should remain `needs follow-up` unless this run
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
