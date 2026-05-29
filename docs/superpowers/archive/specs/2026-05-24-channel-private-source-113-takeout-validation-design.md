# Channel Private Source 113 Takeout Validation Design

## Goal

Run a controlled, sanitized `CHANNEL_PRIVATE` fallback validation for local
Telegram `source_id=113`.

This slice is validation-only. It should not change runtime code, mutate source
identity, decode private payloads, or record raw Telegram/provider data.

## Context

The representative Takeout matrix still has `CHANNEL_PRIVATE fallback` as
`not run`. Offline inventory found no prior local
`only_my_messages_fallback` evidence. Source `113` remains the strongest
available fallback candidate because it is a `channel` source with local
membership marked false.

Current sanitized candidate shape:

| Field | Value |
| --- | --- |
| source_id | 113 |
| source_subtype | channel |
| account_id | 11 |
| peer_kind | channel |
| has_username | 1 |
| has_access_hash | 1 |
| is_member | 0 |
| last_sync_state | 515 |
| last_synced_at | 1779537575 |
| item_count | 29 |
| telegram_message_count | 29 |

Prior Takeout attempts for this source:

| Batch id | Status | Completeness | Terminal error class | Observed | Inserted | Warnings |
| ---: | --- | --- | --- | ---: | ---: | ---: |
| 7 | failed | unknown | TAKEOUT_INIT_DELAY | 0 | 0 | 0 |
| 8 | failed | unknown | TAKEOUT_INIT_DELAY | 0 | 0 | 0 |
| 9 | failed | unknown | TAKEOUT_INIT_DELAY | 0 | 0 | 0 |

Why source `113`:

- it is the strongest private/left-shape candidate in the current sanitized
  inventory;
- it has a normal-sync baseline, so any observations can be checked against
  canonical source rows;
- previous Takeout retries failed before observations, so no fallback evidence
  has been captured yet;
- its local row shape directly targets the open `CHANNEL_PRIVATE fallback`
  matrix row.

Limitations:

- source `113` may again be blocked by Telegram `TAKEOUT_INIT_DELAY`;
- a successful fallback can prove only the only-my-messages access path, not a
  full-history import;
- the validation must not infer fallback behavior from source shape alone.

## Approach

Use the same controlled pattern as prior Takeout validations:

1. Capture and commit pre-run sanitized source and durable Takeout baseline.
2. Confirm app readiness separately from the documentation setup.
3. Pause for explicit live authorization before triggering Takeout.
4. Trigger the existing app flow for `source_id=113`.
5. Monitor only coarse terminal state, aggregate counters, and warning codes.
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
- the note must state that this fallback evidence does not prove a full-history
  import if the run is partial or only-my-messages scoped.

If Takeout completes without fallback:

- update only the repeated-after-normal-sync or row-fidelity evidence that the
  captured observations support;
- keep `CHANNEL_PRIVATE fallback` as `not run`.

If Takeout is blocked, cancelled, partial, or fails:

- record only the supported outcome;
- keep `CHANNEL_PRIVATE fallback` as `not run` unless fallback warning/flag
  evidence exists;
- keep or update related rows conservatively based on the exact terminal class
  and captured observations.

## Non-Goals

- Do not change runtime code.
- Do not add or modify Tauri commands.
- Do not delete or manually edit Takeout batches, observations, source rows, or
  item rows.
- Do not bypass Telegram `TAKEOUT_INIT_DELAY`.
- Do not mark shifted export DC fallback complete unless `export_dc_fallback`
  warning evidence exists.
- Do not mark forum-topic behavior complete from this channel validation.
- Do not update the local handoff file unless the user explicitly asks for it.
