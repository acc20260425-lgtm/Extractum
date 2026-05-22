# Telegram Runtime Private Source Validation

> Status: manual validation matrix with live evidence for rows marked `passed`.

Updated: 2026-05-22

## Scope

Use this checklist after backend pure/storage prep is green and real Telegram
accounts are available. Do not record secrets, phone numbers, session data, or
private message content here.

## Matrix

Validation status values: `not run`, `passed`, `failed`, `blocked`,
`needs follow-up`.

Privacy classification values: `public username`, `dialog-backed likely private`,
`dialog-dependent group`, `access-limited`, `unknown`.

| Case | Validation status | Privacy classification | Add Source result | Stored identity to record | Sync result | Wrong-peer risk check | Warnings/errors |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Public channel | passed | public username | Live dialog scan returned channel dialogs; representative existing source `18` has `source_subtype = channel` and username present | `peer_kind = channel`, `access_hash` present, `username` present, `resolution_strategy = dialog` | `sync_source(18)` inserted 19, skipped 0, `last_message_id = 514` | Stored peer identity resolved and sync completed without redirecting to another peer | No warnings |
| Stored peer before username fallback on public channel | passed | public username | Source `18` was probed with a local sentinel username only; original username value was not recorded | `peer_kind = channel`, `peer_id = 1686571520`, `access_hash` present, original username present, sentinel `extractum_validation_missing_username_20260521` survived the successful sync probe, `resolution_strategy = dialog` | `sync_source(18)` inserted 0, skipped 0, `last_message_id = 514` | Stored peer identity was sufficient while cached username was unusable; strict resolver order remains covered by backend tests | No warnings; no real Telegram username reassignment was performed |
| Public supergroup | passed | public username | Live dialog scan returned supergroup dialogs; representative existing source `17` has `source_subtype = supergroup` and username present | `peer_kind = channel`, `access_hash` present, `username` present, `resolution_strategy = dialog` | `sync_source(17)` inserted 1261, skipped 5, `last_message_id = 101949` | Stored peer identity resolved as supergroup source and sync completed | No warnings |
| Private channel from dialogs | passed | dialog-backed likely private | Representative existing dialog-backed source `27` has `source_subtype = channel` and no username | `peer_kind = channel`, `access_hash` present, `username` absent, `resolution_strategy = dialog` | `sync_source(27)` inserted 6, skipped 0, `last_message_id = 1165` | Stored peer identity resolved without public username fallback | No warnings |
| Private supergroup from dialogs | passed | dialog-backed likely private | Dialog-picked source `Pure C` was added as source `110` with `source_subtype = supergroup` and no username | `peer_kind = channel`, `peer_id = 1335891301`, `access_hash` present, `username` absent, `resolution_strategy = dialog` | `sync_source(110)` inserted 384, skipped 0, `last_message_id = 92374` | Stored peer identity resolved without public username fallback | No warnings |
| Regular small group | passed | dialog-dependent group | Dialog-picked source `Test group` was added as source `111` with `source_subtype = group` and no username | `peer_kind = chat`, `peer_id = 5241485550`, `access_hash` absent, `username` absent, `resolution_strategy = dialog` | `sync_source(111)` inserted 2, skipped 0, `last_message_id = 233` | Resolver remained dialog-dependent and did not create a stored channel/supergroup peer path | No warnings |
| Migrated small group -> supergroup | not run | unknown | Migrated dialog is classified with the observed current subtype | `source_subtype`, `peer_kind`, `peer_id`, `access_hash` presence, provenance notes | Sync/import behavior matches the chosen migration policy | No history is imported across an unsafe identity boundary | Record migration identifiers and warnings |
| No-longer-member, left, or private access lost | passed | access-limited | Controlled private channel was added from account `11` dialogs as source `114`; after access was revoked it disappeared from the dialog list but the stored source row remained explainable | `source_subtype = channel`, `peer_kind = channel`, `peer_id = 3914917549`, access-hash present, username absent, `resolution_strategy = dialog`; before/after snapshots kept identity unchanged | Baseline `sync_source(114)` succeeded with inserted 0, skipped 1, `last_message_id = 1`; post-loss `sync_source(114)` returned typed `network` error with `CHANNEL_PRIVATE` from `messages.getHistory` | Snapshots showed no identity mutation, no sync-state advancement, no item-count growth, and no wrong-peer evidence | Canary id not recorded; coarse count/max-id/max-timestamp growth checks stayed unchanged |
| Same source on account A and account B | passed | public username | Selected one public channel; account A reused source `18` and account B created source `113` through `add_telegram_source` | Same `peer_kind = channel` and `peer_id = 1686571520` recorded under different `account_id` values; access-hash and username recorded by presence only | `sync_source(18)` inserted 0, skipped 0, `last_message_id = 514`; `sync_source(113)` inserted 28, skipped 0, `last_message_id = 514` | Manual dispatch audit confirmed `sync_source` selects the grammers client by source `account_id`; snapshots showed each sync mutated only its own source state/items | No warnings |

## Slice Order

Started `telegram-runtime-private-source-manual-validation` with the first five
source-kind rows: public channel, public supergroup, private channel from
dialogs, private supergroup from dialogs, and regular small group.

The stored-peer username fallback probe then validated that a usable stored peer
identity is sufficient when the cached public username is locally unusable.

Keep migrated dialogs for a follow-up slice.

## 2026-05-21 Live Run Notes

- App commit: `9c49af0` plus running worktree state at start of live slice.
- Account label: account 1 only; no credentials, phone numbers, usernames, or
  message content recorded.
- Runtime status: account 1 was `ready`.
- Live `list_telegram_sources(account_id = 1)` returned 111 dialogs:
  83 channels and 28 supergroups; no regular `group` dialogs were returned.
- Existing persisted Telegram sources for account 1 before adding `Pure C`: 12
  total, all `resolution_strategy = dialog`; six `channel`, six `supergroup`.
- Added dialog-backed private supergroup `Pure C` as source `110`; account 1
  then had 13 persisted Telegram sources.
- Representative sync commands run: `sync_source(18)`, `sync_source(17)`,
  `sync_source(27)`, `sync_source(110)`.

## 2026-05-21 Regular Small Group Follow-Up

- Account 1 was still `ready`.
- Live `list_telegram_sources(account_id = 1)` returned 112 dialogs:
  83 channels, 28 supergroups, and one regular group.
- Added dialog-picked regular small group `Test group` as source `111`.
- Stored identity for source `111`: `source_subtype = group`,
  `peer_kind = chat`, `peer_id = 5241485550`, `access_hash` absent,
  `username` absent, `resolution_strategy = dialog`.
- `sync_source(111)` inserted 2, skipped 0, `last_message_id = 233`, with no
  warnings.

## 2026-05-21 Stored Peer Username Fallback Probe

- Account label: account 1 only; no credentials, phone numbers, session data,
  private message content, or original username value recorded.
- App commit: `886460b docs: clean telegram fallback validation plan`.
- Probe source: source `18`, public channel, original username present,
  `peer_kind = channel`, `peer_id = 1686571520`, `access_hash` present,
  `resolution_strategy = dialog`.
- Before the probe, the SQLite DB file was copied to ignored `reference/`.
- Local probe changed only `telegram_sources.username` to
  `extractum_validation_missing_username_20260521`.
- Account 1 runtime status was `ready`.
- `sync_source(18)` returned inserted 0, skipped 0, `last_message_id = 514`,
  and warnings `[]`.
- Post-sync check before restore: the sentinel remained in
  `telegram_sources.username`.
- Post-restore check: original username restored; `peer_kind`, `peer_id`,
  `access_hash`, and `resolution_strategy` were unchanged.
- This run did not perform a real Telegram username reassignment. It
  temporarily corrupted the local cached username only. The live evidence proves
  that a usable stored peer identity is sufficient for sync when the cached
  username is unusable. The strict resolver order is covered by backend
  resolver tests.

## 2026-05-22 Cross-Account Isolation Probe

- Account labels: account A `1`; account B `11`. No credentials, phone
  numbers, session data, private message content, private titles, private
  usernames, or public username value recorded.
- App commit: `1d60a19 docs: plan telegram cross-account validation`.
- Manual session-dispatch audit checked `src-tauri/src/sources/sync.rs`
  (`sync_source -> sync_telegram_source`), `src-tauri/src/telegram.rs`
  (`get_authorized_runtime`, `init_account_client`), and
  `src-tauri/src/telegram_session_store.rs` (`load_session(account_id)`).
  The sync path loads the source row, dispatches through that row's
  `account_id`, and initializes each grammers client from that account's
  session file.
- Selected public peer: public channel; username value not recorded.
- Stored identity:
  - Account A source `18`: `account_id = 1`, `source_subtype = channel`,
    `peer_kind = channel`, `peer_id = 1686571520`, access-hash present,
    username present, `resolution_strategy = dialog`.
  - Account B source `113`: `account_id = 11`, `source_subtype = channel`,
    `peer_kind = channel`, `peer_id = 1686571520`, access-hash present,
    username present, `resolution_strategy = dialog`.
- Add Source result: account B created a new source row through
  `add_telegram_source`.
- Snapshot order: `before sync A`, `after sync A`, `after sync B`.
- Snapshot SQL used:
  `SELECT id, account_id, last_sync_state, last_synced_at FROM sources WHERE id IN (...)`
  and
  `SELECT source_id, COUNT(*) AS item_count FROM items WHERE source_id IN (...) GROUP BY source_id`.
- Snapshot counts/states:
  - `before sync A`: source `18` state `514`, item count `42`; source `113`
    state `null`, item count `0`.
  - `after sync A`: source `18` state `514`, item count `42`; source `113`
    state `null`, item count `0`.
  - `after sync B`: source `18` state `514`, item count `42`; source `113`
    state `514`, item count `28`.
- Sync A result: `sync_source(18)` inserted 0, skipped 0,
  `last_message_id = 514`, warnings `[]`.
- Sync B result: `sync_source(113)` used initial sync policy `last 30 days`,
  inserted 28, skipped 0, `last_message_id = 514`, warnings `[]`.
- Isolation result: `source_id A != source_id B`, `account_id A != account_id B`,
  `peer_kind A == peer_kind B`, `peer_id A == peer_id B`, sync A did not
  mutate source B state/items, and sync B did not mutate source A state/items.

## 2026-05-22 Lost Access Follow-Up

- Account label: account `11`; no credentials, phone numbers, session data,
  private message content, private titles, private usernames, public username
  values, or access-hash values recorded.
- App commit: `7aad765 docs: plan telegram lost-access validation`.
- Fixture: controlled private `channel` selected from dialogs; username absent,
  access-hash present, `resolution_strategy = dialog`.
- Source row: source `114`, `account_id = 11`, `source_subtype = channel`,
  `peer_kind = channel`, `peer_id = 3914917549`, title present by boolean only.
- Baseline sync: `sync_source(114)` succeeded with initial policy
  `last 30 days`, inserted 0, skipped 1, `last_message_id = 1`, warnings `[]`.
- Access revocation: operator confirmed account `11` access was revoked;
  post-loss dialog visibility was absent.
- Canary: no canary id was captured; local item checks used coarse count,
  max-id, and max-timestamp invariants.
- Post-loss sync: `ok = false`, typed error kind `network`, sanitized message
  `request error: rpc error 400: CHANNEL_PRIVATE caused by messages.getHistory`.
- Snapshots: before/after `last_sync_state = 1`,
  `last_synced_at = 1779418693`, `identity_refreshed_at = 1779418372`,
  `is_member = true`, item count 0, max item id `null`, max timestamp `null`.
- Result: `passed`; typed access-loss behavior was explainable, stored identity
  stayed unchanged, sync state did not advance, items did not grow, and no
  wrong-peer evidence appeared.

## Evidence Template

For each case, record:

- Date and app build/commit.
- Account label A/B, without credentials or phone numbers.
- Validation status and privacy classification.
- Add Source path: username, `t.me`, numeric/manual, or dialog-picked.
- Stored fields: `source_id`, `account_id`, `source_subtype`, `peer_kind`,
  `peer_id`, `access_hash` presence, `username`, `resolution_strategy`.
- Sync outcome: inserted count, last state, warnings, and typed error if any.
- Wrong-peer check: how the observed peer identity was confirmed.
- Follow-up issue, if the observed behavior differs from the expected row.
