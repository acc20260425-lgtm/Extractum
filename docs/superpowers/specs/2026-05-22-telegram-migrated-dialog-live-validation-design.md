# Telegram Migrated Dialog Live Validation Design

Date: 2026-05-22

## Goal

Validate the remaining Telegram runtime/private-source backlog risk for a real
dialog-backed source that started as a regular small group and was migrated by
Telegram into a supergroup.

The primary acceptance is runtime behavior: Add Source must classify the current
dialog as a `supergroup`, persist typed identity, and normal `sync_source` must
resolve and ingest the current supergroup peer without crossing back into the
old small-group identity boundary.

The secondary acceptance is a narrow Takeout smoke on the same fixture, when
runtime conditions allow it. The smoke verifies the current detect-and-defer
contract for migrated small-group history. It must not enable old migrated
history import or expand into the broader Takeout validation matrix.

## Scope

Use the controlled migrated fixture available to local Telegram account `11`.
Do not record the fixture title, username, message text, phone number, session
data, API data, API hash, access-hash value, or auth material in tracked docs.

Primary runtime scope:

1. List account `11` Telegram dialogs through the normal app path.
2. Select the controlled migrated small-group-to-supergroup fixture.
3. Add or reuse the source through `add_telegram_source`.
4. Verify the stored `sources` and `telegram_sources` rows use typed current
   supergroup identity.
5. Run normal `sync_source(source_id)`.
6. Verify sync resolves the current supergroup peer, inserts or skips only
   same-source current-history messages, preserves the source subtype boundary,
   and does not produce wrong-peer evidence.

Secondary Takeout smoke scope:

1. Start Takeout import for the same source if account/session/runtime limits
   make this practical.
2. Verify `channels.getFullChannel`-based migration detection observes
   migrated small-group history.
3. Verify old migrated small-group history is not imported.
4. Verify persisted provenance records the deferment:
   `migrated_history_detected = 1`,
   `migrated_history_imported = 0`, warning
   `migrated_history_deferred`, and partial or mixed-partial completeness
   according to the current provenance model.

Out of scope:

- enabling import of migrated small-group history;
- representative Takeout import validation for all Telegram source kinds;
- `CHANNEL_PRIVATE` fallback validation;
- shifted export DC fallback validation;
- incomplete-import recovery UX or policy;
- forum-topic refresh behavior after Takeout;
- product or schema changes unless the live probe exposes a real bug.

## Existing Runtime Contract

Add Source lists Telegram dialogs and stores Telegram source subtype in
`sources.source_subtype`. Operational Telegram identity lives in
`telegram_sources`, including `source_subtype`, `peer_kind`, `peer_id`,
optional username/access-hash hints, and `resolution_strategy`.

For supergroups, the stored peer contract is:

```text
sources.source_subtype = supergroup
telegram_sources.source_subtype = supergroup
telegram_sources.peer_kind = channel
telegram_sources.peer_id = current supergroup peer id
access_hash present when Telegram exposes it
resolution_strategy = dialog for dialog-picked sources
```

Normal sync builds item identity from the resolved history peer:

```text
telegram_messages.source_id
telegram_messages.history_peer_kind
telegram_messages.history_peer_id
telegram_messages.telegram_message_id
```

For current supergroup history, `history_peer_kind = channel` and
`history_peer_id` should match the current resolved supergroup peer. Migrated
small-group history has a separate Telegram peer boundary and must not be
silently mixed into current-history runtime sync evidence.

Takeout import already has a defensive migrated-history contract for
supergroups. When `detect_supergroup_migration` finds `migrated_from_chat_id`,
the current behavior records the migrated-history warning/provenance and defers
old small-group history import.

## Primary Pass Criteria

The runtime migrated-dialog validation passes when all of these are true:

- Account `11` is ready.
- The controlled fixture is visible through normal dialog listing and is
  classified as `supergroup`.
- `add_telegram_source` creates a new row or safely reuses an existing row for
  the same account/source identity.
- Stored identity records `source_subtype = supergroup`,
  `peer_kind = channel`, current `peer_id`, access-hash presence when exposed by
  Telegram, username presence only as a boolean, and
  `resolution_strategy = dialog`.
- Normal `sync_source(source_id)` returns a typed success result or a typed,
  explainable no-new-items result for the current source.
- Any inserted `telegram_messages` rows for the source use
  `history_peer_kind = channel` and the current supergroup `history_peer_id`.
- Source sync state and item rows mutate only for the selected source.
- Duplicate handling remains source-scoped and history-peer-scoped; no evidence
  shows message id collisions between current supergroup history and old
  small-group history.
- No evidence shows resolver switching to an old regular-group peer, another
  public username, another dialog, or another account.

If this primary flow passes, the backlog row
`verify behavior for migrated small-group-to-supergroup dialogs` can be closed
even if the secondary Takeout smoke is blocked by Telegram-side or runtime
conditions.

## Secondary Smoke Criteria

The Takeout smoke passes when all of these are true:

- `start_takeout_source_import(source_id)` can start for the same source without
  unrelated account/session blockers.
- The job records migrated-history detection for the current supergroup.
- Persisted provenance has `migrated_history_detected = 1` and
  `migrated_history_imported = 0`.
- A warning with kind `migrated_history_deferred` is persisted or surfaced in
  job evidence.
- Completeness is `partial` or the current mixed-partial equivalent used by the
  provenance layer.
- No old small-group migrated-history rows are imported for the source as part
  of this smoke.

The Takeout smoke cannot fail the runtime slice unless it exposes a wrong-peer
or runtime typed-identity bug that also invalidates the primary acceptance.
Flood wait, Takeout session limits, fixture mismatch, export DC oddities, or
other Telegram-side blockers should be recorded as Takeout follow-up evidence
without blocking closure of the runtime/private-source backlog row.

## Evidence To Capture

Runtime-only evidence may be written under ignored `reference/*`. Tracked docs
must stay sanitized.

Capture before and after primary sync:

```sql
SELECT id, account_id, source_type, source_subtype, external_id,
       title IS NOT NULL AND TRIM(title) <> '' AS title_present,
       last_sync_state, last_synced_at, is_active, is_member
FROM sources
WHERE id = ?;
```

```sql
SELECT source_id, account_id, source_subtype, peer_kind, peer_id,
       access_hash IS NOT NULL AS has_access_hash,
       username IS NOT NULL AND TRIM(username) <> '' AS has_username,
       resolution_strategy,
       identity_refreshed_at
FROM telegram_sources
WHERE source_id = ?;
```

```sql
SELECT COUNT(*) AS item_count,
       MAX(CAST(external_id AS INTEGER)) AS max_external_id,
       MAX(published_at) AS max_published_at
FROM items
WHERE source_id = ?;
```

```sql
SELECT history_peer_kind, history_peer_id, COUNT(*) AS item_count,
       MIN(telegram_message_id) AS min_message_id,
       MAX(telegram_message_id) AS max_message_id
FROM telegram_messages
WHERE source_id = ?
GROUP BY history_peer_kind, history_peer_id
ORDER BY history_peer_kind, history_peer_id;
```

For Takeout smoke, capture sanitized provenance rows and warning kinds only.
Record the existence of `migrated_from_chat_id` detection without writing the
old small-group title or message content.

## Outcome Classification

- `passed`: primary runtime flow satisfies all primary pass criteria. Secondary
  Takeout smoke either passes or is recorded separately as non-blocking.
- `passed with Takeout smoke`: primary runtime flow passes and the narrow
  detect-and-defer smoke also satisfies its criteria.
- `blocked`: the controlled fixture is unavailable, account `11` is not ready,
  Add Source cannot safely select the fixture, or baseline sync cannot be
  interpreted.
- `needs follow-up`: primary runtime evidence is mostly healthy but ambiguous,
  such as no messages/items to prove history-peer grouping or unclear
  Telegram-side migration shape.
- `failed`: Add Source misclassifies the current migrated fixture, typed
  identity is missing or legacy-only, sync resolves the old small-group peer,
  sync mutates another source/account, duplicate identity collapses unsafe peer
  boundaries, or errors are raw/internal and not user-actionable.

## Documentation Updates

If the primary flow passes, update
`docs/superpowers/verification/telegram-runtime-private-source-validation.md`:

- Change `Migrated small group -> supergroup` from `not run` to `passed`.
- Add a dated migrated-dialog live-run note.
- Record only sanitized account/source ids, subtype, peer kind/id, access-hash
  presence, username presence, sync counts, warning kinds, and wrong-peer check.
- Include the secondary Takeout smoke result if it was attempted.

Then update `docs/backlog.md` section `3.1` by removing
`verify behavior for migrated small-group-to-supergroup dialogs` if primary
runtime acceptance passed. If Takeout smoke was blocked or inconclusive, keep
or add the specific Takeout follow-up in section `3.3`, not in the runtime
private-source validation row.

If primary runtime flow is blocked, failed, or needs follow-up, keep the runtime
backlog row open or replace it with the concrete sanitized follow-up discovered
by the probe.

## Verification

Before committing design or result docs, run:

```text
git diff --check
```

If production code changes become necessary, stop the live-validation docs flow
and switch to a test-first bugfix plan.
