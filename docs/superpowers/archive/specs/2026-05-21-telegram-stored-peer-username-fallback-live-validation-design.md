# Telegram Stored Peer Username Fallback Live Validation Design

## Goal

Validate the remaining Telegram 3.1 backlog risk that a public Telegram source
with a usable stored peer identity resolves through that stored identity before
falling back to its cached public username.

The slice name is:

```text
telegram-stored-peer-username-fallback-live-validation
```

## Scope

Use account 1 and one existing public Telegram channel or public supergroup that
already has:

- `username` present;
- `access_hash` present;
- `peer_kind = channel`;
- `resolution_strategy = dialog` or `username`;
- a successful prior sync in the runtime validation record.

The preferred candidates are source `17` or source `18`, already recorded in
`docs/superpowers/verification/telegram-runtime-private-source-validation.md`.
Run the probe on exactly one source. A second source adds little confidence and
increases the chance of manual restore mistakes.

This slice does not use a second Telegram account, lost-access fixtures,
migrated group fixtures, or any real Telegram username reassignment.

## Validation Approach

Use a DB-only username probe:

1. Record the original typed identity and source sync fields for the chosen
   source: `source_id`, `source_subtype`, `peer_kind`, `peer_id`, `access_hash`
   presence, original `username`, `resolution_strategy`, `last_sync_state`, and
   `last_synced_at`.
2. Back up the SQLite database file or record exact restore SQL before editing.
3. Stop the app before direct SQLite edits if it is running.
4. Temporarily replace only `telegram_sources.username` with:

   ```text
   extractum_validation_missing_username_20260521
   ```

5. Start the app, confirm account 1 is `ready`, and call `sync_source(source_id)`
   through the normal Tauri IPC path.
6. Restore the original username immediately after the probe.
7. Re-read the typed identity and confirm `username` is restored and
   `peer_kind`, `peer_id`, `access_hash`, and `resolution_strategy` were not
   unexpectedly changed.
8. Record the sync result, warnings, wrong-peer check, and limitation in the
   verification document and backlog.

Practical execution note: before the probe update, close the app and make a
copy of the SQLite DB file. After the probe update, start the app only for the
sync run. Restore the username immediately after sync completes, then re-open
or re-read the row.

The probe proves operational independence from the cached username: a source
with a bad cached username still syncs when its stored peer identity is usable.
The strict resolver order is covered by backend resolver tests, including
`typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists`
and the dialog channel/supergroup stored-peer tests.

## Pre-Flight And Restore SQL

Use SQL as a checklist, not as a blind copy-paste target. Replace `:source_id`
and `:original_username` with the selected source id and the exact username
read before the probe.

Pre-flight:

```sql
SELECT
  ts.source_id,
  ts.account_id,
  ts.source_subtype,
  ts.peer_kind,
  ts.peer_id,
  ts.resolution_strategy,
  ts.username,
  ts.access_hash,
  s.last_sync_state,
  s.last_synced_at
FROM telegram_sources ts
JOIN sources s ON s.id = ts.source_id
WHERE ts.source_id = :source_id;
```

Required pre-flight conditions:

- account 1 is `ready`;
- selected source has no unresolved runtime warnings before the probe;
- `username IS NOT NULL`;
- `access_hash IS NOT NULL`;
- `peer_kind = 'channel'`;
- `source_subtype` is `channel` or `supergroup`;
- only one source is selected.

Abort the probe if any required pre-flight condition is false.

Probe update:

```sql
UPDATE telegram_sources
SET username = 'extractum_validation_missing_username_20260521'
WHERE source_id = :source_id;
```

Restore:

```sql
UPDATE telegram_sources
SET username = :original_username
WHERE source_id = :source_id;
```

Post-restore comparison:

```sql
SELECT
  source_id,
  account_id,
  source_subtype,
  peer_kind,
  peer_id,
  resolution_strategy,
  username,
  access_hash
FROM telegram_sources
WHERE source_id = :source_id;
```

The post-restore row must match the pre-flight row for `peer_kind`, `peer_id`,
`access_hash`, and `resolution_strategy`, and must contain the original
username again.

## Safety

The probe must not change Telegram server state. It only mutates a local cached
username and restores it before documentation updates.

If the username cannot be restored, restore the SQLite database from the backup
before continuing.

If sync refreshes typed identity before history resolution and overwrites the
sentinel with the true username, the result is weaker. In that case, document
the run as `needs follow-up` instead of `passed`, because sync may have used the
refreshed username path.

## Evidence To Record

Record the following without credentials, phone numbers, session data, private
message content, or private usernames:

- date and commit;
- account label `account 1`;
- source id and source subtype;
- original username presence, not the username value;
- `peer_kind`, `peer_id`, `access_hash` presence, and `resolution_strategy`;
- full sentinel username value used for the local probe;
- `sync_source` inserted/skipped/last-message result and warnings;
- post-restore typed identity check;
- wrong-peer check;
- limitation statement:

  ```text
  This run did not perform a real Telegram username reassignment. It temporarily
  corrupted the local cached username only. The live evidence proves that a
  usable stored peer identity is sufficient for sync when the cached username is
  unusable. The strict resolver order is covered by backend resolver tests.
  ```

## Expected Documentation Changes

Update:

- `docs/superpowers/verification/telegram-runtime-private-source-validation.md`
- `docs/backlog.md`

The validation matrix should mark the stored-peer username fallback case as
`passed` only if the sentinel remains in the typed identity through the
successful sync probe and is restored afterward. Otherwise, mark it as
`needs follow-up`.

The backlog should keep only the remaining 3.1 rows after this slice:

- cross-account isolation;
- lost-access behavior;
- migrated small-group-to-supergroup behavior.

## Verification

Before committing the validation result, run:

```text
git diff --check
```

If any code changes become necessary, stop and switch to RED test first. This
slice is expected to be live validation and documentation only.
