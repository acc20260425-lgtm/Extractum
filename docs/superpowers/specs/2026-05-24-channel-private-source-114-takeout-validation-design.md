# Channel Private Source 114 Takeout Validation Design

## Goal

Run a controlled, sanitized `CHANNEL_PRIVATE` Takeout fallback validation for
local Telegram `source_id=114`.

This slice is validation-only. It should not change runtime code, mutate source
identity, decode private payloads, or record raw Telegram/provider data.

## Context

The representative Takeout matrix still has `CHANNEL_PRIVATE fallback` as
`not run`. Source `113` was the previous strongest private/left-shape
candidate, but batch `14` completed without fallback warning or only-my-
messages flags. A fresh read-only inventory now makes source `114` the strongest
candidate because it already produced sanitized normal-sync `CHANNEL_PRIVATE`
evidence after access was revoked.

Current sanitized candidate shape:

| Field | Value |
| --- | --- |
| source_id | 114 |
| source_subtype | channel |
| account_id | 11 |
| peer_kind | channel |
| has_username | 0 |
| has_access_hash | 1 |
| is_active | 1 |
| is_member | 1 |
| resolution_strategy | dialog |
| last_sync_state | 1 |
| last_synced_at | 1779418693 |
| item_count | 0 |
| telegram_message_count | 0 |
| prior_takeout_batches | 0 |

Why source `114`:

- prior sanitized runtime validation recorded a normal-sync
  `CHANNEL_PRIVATE` error from `messages.getHistory`;
- it is a dialog-backed channel with no username and an access hash;
- it has no prior Takeout batches, so any Takeout fallback evidence will be
  fresh and easy to classify;
- it directly targets the open `CHANNEL_PRIVATE fallback` matrix row.

Limitations:

- source `114` may have no visible or outgoing messages available to the
  only-my-messages fallback;
- a fallback result can prove the access-limited fallback path but not
  full-history import completeness;
- the local `is_member` flag is stale or not decisive for access because the
  prior normal-sync result is stronger evidence than the local membership flag;
- Telegram can still return `TAKEOUT_INIT_DELAY` before history loading.

## Approach

Use the same controlled pattern as prior Takeout validations:

1. Capture and commit pre-run sanitized source and durable Takeout baseline.
2. Confirm app readiness separately from the documentation setup.
3. Pause for explicit live authorization before triggering Takeout.
4. Trigger the existing app flow for `source_id=114`.
5. Monitor only coarse terminal state, aggregate counters, and warning codes.
6. Capture post-run source snapshot, batch summary, fallback evidence, warning
   visibility, duplicate/row-fidelity diagnostics if observations exist, and
   before/after deltas.
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
- capped local sample ids from row-fidelity diagnostics if observations exist.

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

If `CHANNEL_PRIVATE` fallback is observed:

- the `CHANNEL_PRIVATE fallback` row can be updated from `not run` only when
  `only_my_messages_fallback` warning evidence or an equivalent durable flag is
  present;
- the result note must include batch id, status/completeness, warning codes,
  `only_my_messages`/fallback flags, observations/insertions, and before/after
  source deltas;
- the note must state that fallback evidence does not prove full-history import
  if the run is only-my-messages scoped or partial.

If Takeout is blocked before history loading:

- record the typed/coarse blocker such as `TAKEOUT_INIT_DELAY`;
- keep `CHANNEL_PRIVATE fallback` as `not run`;
- keep source `114` as a retry candidate unless the terminal class proves it is
  unsuitable.

If Takeout completes without fallback:

- record the completed evidence;
- keep `CHANNEL_PRIVATE fallback` as `not run`;
- document that prior normal-sync `CHANNEL_PRIVATE` did not reproduce in
  Takeout.

## Non-Goals

- Do not change runtime code.
- Do not add or modify Tauri commands.
- Do not delete or manually edit Takeout batches, observations, source rows, or
  item rows.
- Do not bypass Telegram `TAKEOUT_INIT_DELAY`.
- Do not mark shifted export DC fallback complete unless `export_dc_fallback`
  warning evidence exists.
- Do not mark migrated-history behavior complete from this channel validation.
- Do not update the local handoff file unless the user explicitly asks for it.
