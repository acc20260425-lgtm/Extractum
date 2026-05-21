# Telegram Runtime Private Source Validation

> Status: planned manual validation matrix. This file is not evidence that live Telegram validation has already run.

Updated: 2026-05-21

## Scope

Use this checklist after backend pure/storage prep is green and real Telegram
accounts are available. Do not record secrets, phone numbers, session data, or
private message content here.

## Matrix

| Case | Add Source result | Stored identity to record | Sync result | Wrong-peer risk check | Warnings/errors |
| --- | --- | --- | --- | --- | --- |
| Public channel | Source appears with `source_subtype = channel` and expected title | `peer_kind = channel`, `peer_id`, `access_hash` presence, `username`, `resolution_strategy` | Messages insert for the channel | Username and stored peer resolve to the same channel, or stored peer wins when both exist | Record username changes or lookup failures |
| Public supergroup | Source appears with `source_subtype = supergroup` and expected title | `peer_kind = channel`, `peer_id`, `access_hash` presence, `username`, `resolution_strategy` | Messages insert for the supergroup | Stored peer does not resolve to a broadcast channel or unrelated group | Record topic/forum oddities |
| Private channel from dialogs | Dialog-picked source appears as `channel` | `peer_kind = channel`, stable `peer_id`, `access_hash`, optional `username`, `resolution_strategy = dialog` | Sync works without public username lookup | Stored peer identity is tried before username fallback | Record private/access errors |
| Private supergroup from dialogs | Dialog-picked source appears as `supergroup` | `peer_kind = channel`, stable `peer_id`, `access_hash`, optional `username`, `resolution_strategy = dialog` | Sync works without public username lookup | Stored peer identity is tried before username fallback | Record private/access errors |
| Regular small group | Dialog-picked source appears as `group` | `peer_kind = chat`, `peer_id`, `resolution_strategy = dialog`, no required `access_hash` | Sync works while the dialog is visible | Resolver remains dialog-dependent and does not create a stored channel peer path | Record missing-dialog behavior |
| Migrated small group -> supergroup | Migrated dialog is classified with the observed current subtype | `source_subtype`, `peer_kind`, `peer_id`, `access_hash` presence, provenance notes | Sync/import behavior matches the chosen migration policy | No history is imported across an unsafe identity boundary | Record migration identifiers and warnings |
| No-longer-member, left, or private access lost | Existing source remains explainable in UI/state | Stored identity remains unchanged unless refresh succeeds safely | Sync fails or warns predictably | Failure does not silently resolve to a different public username owner | Record exact typed error/warning |
| Same source on account A and account B | Both accounts can add their own source rows | Same `peer_kind`/`peer_id` may exist under different `account_id` values | Sync for each account uses that account's session | Account A source never uses account B state, and vice versa | Record account-scoped differences |

## Evidence Template

For each case, record:

- Date and app build/commit.
- Account label A/B, without credentials or phone numbers.
- Add Source path: username, `t.me`, numeric/manual, or dialog-picked.
- Stored fields: `source_id`, `account_id`, `source_subtype`, `peer_kind`,
  `peer_id`, `access_hash` presence, `username`, `resolution_strategy`.
- Sync outcome: inserted count, last state, warnings, and typed error if any.
- Wrong-peer check: how the observed peer identity was confirmed.
- Follow-up issue, if the observed behavior differs from the expected row.
