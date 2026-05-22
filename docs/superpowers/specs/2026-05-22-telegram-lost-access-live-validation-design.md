# Telegram Lost Access Live Validation Design

Date: 2026-05-22

## Goal

Validate the Telegram sync failure path for a source that already exists in
SQLite but is no longer accessible to the owning Telegram account.

The app must keep the source explainable, preserve typed Telegram identity, and
avoid silently resolving or ingesting data from a different peer. This closes the
`No-longer-member, left, or private access lost` row in
`docs/superpowers/verification/telegram-runtime-private-source-validation.md`
and the corresponding `docs/backlog.md` section `3.1` item when the live result
passes.

## Preferred Live Scenario

Use one controlled private Telegram source owned or administered by another
account, while account A is only a normal member:

1. Private supergroup from dialogs, preferred.
2. Private channel from dialogs, fallback if the supergroup fixture is not
   practical.

Do not start with a regular small group. Small groups remain dialog-dependent
and do not exercise the stored channel/supergroup peer identity path with an
access hash.

The fixture must be private and must not have a public username recorded for the
probe. A public username fallback can mask the access-loss behavior this probe
is meant to exercise.

## Existing Runtime Contract

The relevant sync path is:

```text
sync_source(source_id)
-> load_source(source_id)
-> source.account_id
-> get_authorized_runtime(account_id)
-> resolve_and_refresh_peer(...)
-> persist_items(...)
-> finalize_sync(...)
```

`resolve_and_refresh_peer` loads typed identity from `telegram_sources` and
tries resolution through the existing typed plan. For dialog-backed
channel/supergroup rows with an access hash, the stored peer identity is tried
before dialog scan. If the source cannot be resolved, the current code returns a
typed `AppErrorKind::NotFound` with a user-actionable message for private
sources that disappeared from dialogs. If Telegram accepts the stored peer but
history access fails while iterating messages, the error is expected to surface
as a typed `network` error from the grammers request.

Other typed `AppError` kinds may be acceptable if the message is
user-actionable, the source remains explainable, and identity/state/item
invariants hold. Untyped, raw, or `internal` errors are not acceptable unless
they expose a real bug to fix.

`finalize_sync` runs only after peer resolution and message ingest complete.
Therefore, a failed sync should not advance `sources.last_sync_state` or
`sources.last_synced_at`.

Current `sync_source` does not update `sources.is_member` on access-loss
failure. This field is captured as an explainability diagnostic, not as a
requirement that the app automatically marks the source as left. A change from
member to non-member is acceptable only if the live evidence shows a deliberate,
same-peer safe refresh or explicit source-state update.

`identity_refreshed_at` is expected not to change when resolution or history
access fails before a safe refresh. If it changes during this probe, the result
needs follow-up unless snapshots and logs prove the same peer was safely
refreshed.

## Probe Flow

1. Confirm the tracked workspace is clean and record the current commit.
2. Start the Tauri app and confirm account A is `ready`.
3. Select or create a controlled private supergroup/channel visible in account
   A's Telegram dialogs.
4. Add or reuse the source through the normal app path:
   `list_telegram_sources(account_id)` and `add_telegram_source`.
   If reusing an existing source, first verify it is dialog-backed, private,
   shaped as `channel` or `supergroup`, has access hash present, has no username
   recorded, and baseline sync succeeds before access loss.
5. Run baseline `sync_source(source_id)` while account A still has access. A
   reused source may be caught up and return `inserted = 0`; that is acceptable
   if the sync succeeds and the source identity remains the intended private
   peer.
6. Capture the before-loss snapshot.
7. Human gate: remove account A from the controlled private source or otherwise
   revoke access.
8. Optionally post one post-removal canary message from an admin account after
   the human removal gate and before the post-loss `sync_source` attempt. If a
   canary message id is available, store the numeric id only in ignored
   `reference/*` context. Tracked docs must record only whether post-removal
   content was observed locally, not the message text.
9. Check whether the source still appears in `list_telegram_sources(account_id)`.
10. Run `sync_source(source_id)` and capture either the `SyncResult` or typed
   `AppError`.
11. Capture the after-loss snapshot.
12. Evaluate identity, state, item-count, and wrong-peer invariants.
13. Stop runtime processes and update tracked verification docs.

## Snapshot Evidence

Runtime details are written only to ignored `reference/*` files. Tracked docs
must not include private titles, usernames, message text, phone numbers, session
data, API credentials, or auth material.

If the probe reuses an existing source instead of creating a new one, record
that it was a reused existing dialog-backed private source and capture only the
sanitized identity fields below.

Before and after the access-loss sync attempt, capture:

```sql
SELECT id, account_id, source_type, source_subtype, external_id, title,
       last_sync_state, last_synced_at, is_active, is_member
FROM sources
WHERE id = ?;
```

```sql
SELECT source_id, account_id, source_subtype, peer_kind, peer_id,
       access_hash IS NOT NULL AS has_access_hash,
       username IS NOT NULL AND TRIM(username) <> '' AS has_username,
       resolution_strategy,
       identity_refreshed_at,
       created_at,
       updated_at
FROM telegram_sources
WHERE source_id = ?;
```

```sql
SELECT COUNT(*) AS item_count,
       MAX(CAST(external_id AS INTEGER)) AS max_external_id,
       MAX(created_at) AS max_created_at
FROM items
WHERE source_id = ?;
```

If a post-removal canary message id is available, also capture:

```sql
SELECT COUNT(*) AS canary_item_count
FROM items
WHERE source_id = ?
  AND external_id = ?;
```

Also capture whether the source's typed peer appears in
`list_telegram_sources(account_id)` after the human removal gate. Record this as
presence/absence only, without private labels.

Do not expand this probe to `analysis_documents`, archive read-model rows, or
other derived tables. The validation target is runtime source resolution, source
sync state, and item insertion boundaries. `telegram_sources.updated_at` is
captured only as supporting diagnostic context; `identity_refreshed_at` is the
identity-refresh signal used for pass/follow-up interpretation.

## Pass Criteria

The probe passes when all of these are true:

- Baseline sync succeeds before access is revoked.
- After access is revoked, `sync_source(source_id)` returns a typed,
  explainable error or warning for inaccessible/private/lost peer behavior.
- `telegram_sources.source_subtype`, `peer_kind`, `peer_id`, access-hash
  presence, username presence, `resolution_strategy`, and
  `identity_refreshed_at` do not change unless a clearly successful safe refresh
  occurred. For this lost-access probe, any `identity_refreshed_at` change is
  `needs follow-up` unless logs and snapshots prove a safe refresh of the same
  peer.
- `sources.is_member` either remains unchanged or changes only through an
  explainable, same-peer source-state update. It is not required to become `0`
  for this probe because current sync does not automatically write that field on
  access-loss failure.
- `sources.last_sync_state` and `sources.last_synced_at` do not advance after a
  failed sync.
- Item count for the source does not increase after a failed sync.
- If a canary id was captured, `canary_item_count` remains `0`; if no canary id
  was captured, `max_external_id` and `max_created_at` provide the coarse
  post-removal ingest check alongside item count.
- No evidence shows resolver switching to another public username, another
  dialog, or another peer.
- `list_sources` or direct source queries still show an explainable source row;
  the source does not disappear as if it never existed.

## Outcome Classification

- `passed`: the failure path is typed/explainable and all identity/state/item
  invariants hold.
- `blocked`: account A could not be removed, the private fixture could not be
  created, account A was not ready, baseline sync failed before access loss,
  access could not be confidently revoked, or membership/access state remained
  ambiguous enough that the sync result cannot be interpreted.
- `needs follow-up`: the sync unexpectedly succeeds after removal but snapshots
  show no wrong-peer mutation; this may reflect Telegram retaining history
  access for the tested state and needs a narrower fixture.
- `failed`: sync inserts new items from a wrong peer, mutates typed identity
  unsafely, advances sync state after a failed attempt, returns an unhelpful
  internal/raw error, or makes the source unexplained/unavailable in the app
  state.

A successful post-removal sync is not automatically a pass. It is pass-like only
if Telegram still permits legitimate history access to the same peer and no new
post-removal or canary content is ingested. Otherwise classify as
`needs follow-up` or `failed` according to the evidence.

## Documentation Updates

If passed, update
`docs/superpowers/verification/telegram-runtime-private-source-validation.md`:

- Change `No-longer-member, left, or private access lost` from `not run` to
  `passed`.
- Add `2026-05-22 Lost Access Follow-Up`.
- Record source kind, account label, sanitized stored identity, baseline sync
  result, post-loss typed error/warning, before/after snapshots, dialog
  visibility, and wrong-peer check.

Then update `docs/backlog.md` section `3.1` by removing the
lost-access/no-longer-member row, leaving only the migrated small-group to
supergroup validation row open.

If blocked, failed, or needs follow-up, keep the backlog item open or replace it
with the concrete follow-up discovered by the probe.
