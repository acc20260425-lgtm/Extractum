# Public Supergroup Source 22 Takeout Validation Design

## Goal

Run a controlled, sanitized completed public-supergroup Takeout validation for
local Telegram `source_id=22`.

This slice is validation-only. It should not change runtime code, mutate source
identity, decode private payloads, or record raw Telegram/provider data.

## Context

Source `18` / batch `10` closed the public-channel and
Takeout-after-normal-sync validation rows. The remaining representative
public-source gap is a completed public supergroup Takeout run.

The current best candidate is `source_id=22`:

| Field | Value |
| --- | --- |
| source_subtype | supergroup |
| account_id | 1 |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 1 |
| resolution_strategy | dialog |
| last_sync_state | 157979 |
| last_synced_at | 1777917482 |
| item_count | 1137 |
| telegram_message_count | 1137 |
| topic_membership_count | 1136 |
| reply_count | 1133 |
| thread_count | 760 |
| reaction_item_count | 115 |
| prior_takeout_batches | 0 |

Why source `22`:

- it is the smallest current public/member supergroup candidate with no prior
  Takeout batches;
- it uses account `1`, which just completed source `18` successfully;
- it has topic, reply, thread, and reaction shape useful for representative
  supergroup coverage;
- it avoids reusing partial/cancelled sources `21` and `110`;
- it avoids account `11`, which has recent `TAKEOUT_INIT_DELAY` history.

## Approach

Use the same controlled pattern as source `18`:

1. Capture and commit pre-run sanitized source and durable Takeout baseline.
2. Pause for explicit live authorization before triggering Takeout.
3. Trigger the existing app flow for `source_id=22`.
4. Monitor only coarse terminal state and aggregate counters.
5. Capture post-run source snapshot, batch summary, duplicate summary,
   row-fidelity comparison, warning visibility, and before/after delta.
6. Update the matrix and backlog only for rows supported by captured evidence.

## Alternatives Considered

Recommended: source `22`.

Tradeoff: it has over one thousand baseline rows and topic-rich shape, so it is
larger than source `18` but still much smaller than the existing partial
supergroup sources.

Alternative: source `19`.

Tradeoff: no prior Takeout batches and public/member shape, but it has no topic
memberships and is larger than source `22`, making it weaker representative
supergroup evidence.

Alternative: retry source `21`.

Tradeoff: it already has useful partial evidence, but its prior estimate was
large and the validation need is a completed run rather than another bounded
partial run.

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
- forum-topic decision input can be strengthened if topic membership/catalog
  evidence is captured, but behavior must not change from this validation slice.

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
