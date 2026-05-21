# Telegram Runtime Private Source Validation

> Status: planned manual validation matrix. This file is not evidence that live Telegram validation has already run.

Updated: 2026-05-21

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
| Public supergroup | passed | public username | Live dialog scan returned supergroup dialogs; representative existing source `17` has `source_subtype = supergroup` and username present | `peer_kind = channel`, `access_hash` present, `username` present, `resolution_strategy = dialog` | `sync_source(17)` inserted 1261, skipped 5, `last_message_id = 101949` | Stored peer identity resolved as supergroup source and sync completed | No warnings |
| Private channel from dialogs | passed | dialog-backed likely private | Representative existing dialog-backed source `27` has `source_subtype = channel` and no username | `peer_kind = channel`, `access_hash` present, `username` absent, `resolution_strategy = dialog` | `sync_source(27)` inserted 6, skipped 0, `last_message_id = 1165` | Stored peer identity resolved without public username fallback | No warnings |
| Private supergroup from dialogs | passed | dialog-backed likely private | Dialog-picked source `Pure C` was added as source `110` with `source_subtype = supergroup` and no username | `peer_kind = channel`, `peer_id = 1335891301`, `access_hash` present, `username` absent, `resolution_strategy = dialog` | `sync_source(110)` inserted 384, skipped 0, `last_message_id = 92374` | Stored peer identity resolved without public username fallback | No warnings |
| Regular small group | passed | dialog-dependent group | Dialog-picked source `Test group` was added as source `111` with `source_subtype = group` and no username | `peer_kind = chat`, `peer_id = 5241485550`, `access_hash` absent, `username` absent, `resolution_strategy = dialog` | `sync_source(111)` inserted 2, skipped 0, `last_message_id = 233` | Resolver remained dialog-dependent and did not create a stored channel/supergroup peer path | No warnings |
| Migrated small group -> supergroup | not run | unknown | Migrated dialog is classified with the observed current subtype | `source_subtype`, `peer_kind`, `peer_id`, `access_hash` presence, provenance notes | Sync/import behavior matches the chosen migration policy | No history is imported across an unsafe identity boundary | Record migration identifiers and warnings |
| No-longer-member, left, or private access lost | not run | access-limited | Existing source remains explainable in UI/state | Stored identity remains unchanged unless refresh succeeds safely | Sync fails or warns predictably | Failure does not silently resolve to a different public username owner | Record exact typed error/warning |
| Same source on account A and account B | not run | unknown | Both accounts can add their own source rows | Same `peer_kind`/`peer_id` may exist under different `account_id` values | Sync for each account uses that account's session | Account A source never uses account B state, and vice versa | Record account-scoped differences |

## Slice Order

Start `telegram-runtime-private-source-manual-validation` with the first five
matrix rows: public channel, public supergroup, private channel from dialogs,
private supergroup from dialogs, and regular small group.

Keep migrated dialogs, lost access, and account A/B validation for a separate
follow-up slice.

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
