# Takeout Provenance Foundation Design

Date: 2026-05-17

## Summary

This slice adds durable ingest provenance for Telegram Takeout imports without
enabling migrated-history import and without changing normal `sync_source`
runtime behavior.

The schema is generic enough for later sync and provider ingestion work:
`ingest_batches`, `ingest_item_observations`, and `ingest_batch_warnings`.
Runtime wiring in this slice is intentionally Takeout-only. Takeout receives a
Telegram-specific batch detail table, records item-level observations for both
new inserts and duplicate observations, and stores terminal status for
completed, failed, and cancelled imports.

The goal is to make partial Takeout imports explainable after restart while
keeping the product behavior stable.

## Current State

Takeout import is a second ingest path next to `sync_source`.

Current durable writes are:

- `items`
- `telegram_messages`
- `item_topic_memberships` when topic resolution state is ready/current
- `sources.last_sync_state` and `sources.last_synced_at` only after successful
  Takeout finish

Current runtime job state is in memory. If the app restarts, active job records
are lost, while already inserted rows remain. Failed and cancelled imports can
therefore leave partial `items` rows without durable batch provenance.

Telegram native item identity and topic membership materialization already
exist. The remaining gap is not whether duplicate messages can be represented,
but which ingest attempt inserted or later observed them.

## Goals

1. Add generic durable ingest batch tables.
2. Wire the new provenance only into Telegram Takeout runtime.
3. Record terminal Takeout status: completed, failed, or cancelled.
4. Preserve crash-interrupted batches as persisted `status = running`.
5. Distinguish complete and partial terminal runs.
6. Record structured warnings as durable rows.
7. Record item observations for inserted, duplicate-observed, skipped, and
   failed item-level outcomes.
8. Link duplicate observations to the existing `item_id` when typed Telegram
   identity can resolve it.
9. Keep normal `sync_source` out of scope for this slice.
10. Keep migrated-history import disabled while durably recording migrated
    history detection/deferment.

## Non-Goals

- Do not enable migrated-history Takeout import.
- Do not add a Takeout history UI.
- Do not make normal sync write ingest provenance.
- Do not change Telegram item identity, topic membership, or source identity
  semantics.
- Do not change YouTube source/job provenance.
- Do not add a runner-managed migration.
- Do not store raw Telegram TL payloads, session data, secrets, cookies, or
  request headers in provenance tables.

## Chosen Approach

Use a regular SQL migration, `23.sql`, to add append-only provenance tables.
No existing data is backfilled and no existing table is rebuilt, so a
runner-managed migration is unnecessary.

The schema is generic, but the runtime wiring is Takeout-only:

- `provider = 'telegram'`
- `ingest_kind = 'takeout'`
- `provider_item_kind = 'telegram_message'`
- `provider_identity_kind = 'telegram_message'`

Normal sync can adopt the same tables later without renaming or reshaping the
schema.

## Schema

### `ingest_batches`

```sql
CREATE TABLE ingest_batches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,

  provider TEXT NOT NULL,
  ingest_kind TEXT NOT NULL,

  status TEXT NOT NULL,
  completeness TEXT NOT NULL DEFAULT 'unknown',

  started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at TEXT,

  item_inserted_count INTEGER NOT NULL DEFAULT 0,
  item_observed_count INTEGER NOT NULL DEFAULT 0,
  item_duplicate_count INTEGER NOT NULL DEFAULT 0,
  item_skipped_count INTEGER NOT NULL DEFAULT 0,
  warning_count INTEGER NOT NULL DEFAULT 0,

  terminal_error TEXT,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (provider IN ('telegram', 'youtube')),
  CHECK (ingest_kind IN (
    'takeout',
    'sync',
    'youtube_metadata',
    'youtube_transcript',
    'youtube_comments',
    'youtube_playlist'
  )),
  CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
  CHECK (completeness IN ('unknown', 'complete', 'partial')),
  CHECK (item_inserted_count >= 0),
  CHECK (item_observed_count >= 0),
  CHECK (item_duplicate_count >= 0),
  CHECK (item_skipped_count >= 0),
  CHECK (warning_count >= 0),
  CHECK (
    item_observed_count >=
    item_inserted_count + item_duplicate_count + item_skipped_count
  )
);
```

`completeness` is separate from terminal status:

- `completed + complete`: selected current history completed normally.
- `completed + partial`: only-my-messages fallback or migrated history was
  detected and deferred.
- `failed + partial`: the run inserted or observed some items before failing.
- `cancelled + partial`: the user cancelled after partial writes.
- `running + unknown`: active or crash-interrupted run.

`terminal_error` is bounded and sanitized. It must not contain raw Telegram
payloads, secrets, auth material, or compressed data.

Counter semantics:

- `item_observed_count`: total durable item-level observation rows.
- `item_inserted_count`: subset with `outcome = 'inserted'`.
- `item_duplicate_count`: subset with `outcome = 'duplicate_observed'`.
- `item_skipped_count`: subset with `outcome = 'skipped'`.

If `outcome = 'failed'` is used, it contributes to `item_observed_count` but
not to the three narrower counters unless a later migration adds a dedicated
failed counter. This intentional `>=` relationship must be documented in
`docs/database-schema.md` after implementation so a later reader does not treat
the difference between observed and summed narrow counters as drift by default.

### `telegram_takeout_batches`

```sql
CREATE TABLE telegram_takeout_batches (
  batch_id INTEGER PRIMARY KEY REFERENCES ingest_batches(id) ON DELETE CASCADE,

  account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
  source_subtype TEXT NOT NULL,

  resolved_peer_kind TEXT,
  resolved_peer_id INTEGER,
  history_peer_kind TEXT,
  history_peer_id INTEGER,

  takeout_id INTEGER,
  export_dc_id INTEGER,
  used_export_dc INTEGER NOT NULL DEFAULT 0,
  fallback_used INTEGER NOT NULL DEFAULT 0,

  history_scope TEXT NOT NULL DEFAULT 'unknown',

  migrated_history_detected INTEGER NOT NULL DEFAULT 0,
  migrated_history_imported INTEGER NOT NULL DEFAULT 0,
  only_my_messages INTEGER NOT NULL DEFAULT 0,

  split_count INTEGER,
  selected_split_count INTEGER,
  message_count_estimate INTEGER,
  max_message_id INTEGER,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (source_subtype IN ('channel', 'supergroup', 'group')),
  CHECK (resolved_peer_kind IS NULL OR resolved_peer_kind IN ('channel', 'chat')),
  CHECK (history_peer_kind IS NULL OR history_peer_kind IN ('channel', 'chat', 'user')),
  CHECK (history_scope IN (
    'unknown',
    'current_history',
    'current_history_with_migrated_deferred',
    'partial_private_history',
    'mixed_partial'
  )),
  CHECK (used_export_dc IN (0, 1)),
  CHECK (fallback_used IN (0, 1)),
  CHECK (migrated_history_detected IN (0, 1)),
  CHECK (migrated_history_imported IN (0, 1)),
  CHECK (only_my_messages IN (0, 1))
);
```

`migrated_history_imported` is not hard-coded to `0` at the schema level. In
this foundation-only slice, runtime and tests must keep it `0`. A future
migrated-history enablement slice can set it to `1` without rebuilding this
table just to relax a CHECK.

`message_count_estimate` is nullable and best-effort. It may be based on split
ranges and count probes, but it must not drive correctness. Takeout pagination
correctness remains owned by the explicit TDesktop-first cursor state machine
and the current descending fallback rules.

### `ingest_item_observations`

```sql
CREATE TABLE ingest_item_observations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  batch_id INTEGER NOT NULL REFERENCES ingest_batches(id) ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,

  item_id INTEGER REFERENCES items(id) ON DELETE SET NULL,

  provider_item_kind TEXT NOT NULL,
  provider_identity_kind TEXT NOT NULL,
  provider_identity TEXT NOT NULL,
  provider_identity_version INTEGER NOT NULL DEFAULT 1,

  outcome TEXT NOT NULL,
  reason_code TEXT,

  observed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (provider_item_kind IN ('telegram_message')),
  CHECK (provider_identity_version >= 1),
  CHECK (outcome IN ('inserted', 'duplicate_observed', 'skipped', 'failed'))
);
```

Observations are intentionally not unique. If pagination, fallback, or retry
behavior sees the same provider identity multiple times in one batch, the
database should preserve those observations instead of silently collapsing them.

For Telegram messages:

```text
provider_identity_kind = telegram_message
provider_identity = telegram:history_peer:<kind>:<id>:message:<message_id>
provider_identity_version = 1
```

The `<kind>/<id>` pair is `telegram_messages.history_peer_kind` and
`telegram_messages.history_peer_id`, not the current resolved
`telegram_sources.peer_kind` / `telegram_sources.peer_id`. This matters for
migrated history domains.

The normal duplicate path should resolve the existing `telegram_messages.item_id`
and store it in `item_id`. If an exceptional compatibility case cannot resolve
the existing item id, the observation may store `item_id = NULL` with a
`reason_code`, but that is not the expected path.

The generic text identity is an intentional foundation-slice compromise. It
keeps `ingest_item_observations` provider-neutral instead of adding
Telegram-only columns. If observation volume or identity lookups become a
storage/performance problem, a later migration can add typed provider identity
columns such as `history_peer_kind`, `history_peer_id`, and
`telegram_message_id` with narrower indexes.

### `ingest_batch_warnings`

```sql
CREATE TABLE ingest_batch_warnings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  batch_id INTEGER NOT NULL REFERENCES ingest_batches(id) ON DELETE CASCADE,

  code TEXT NOT NULL,
  message TEXT NOT NULL,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Warnings are not unique. For example, pagination fallback on two different
splits should produce two rows.

Initial warning codes:

- `export_dc_fallback`
- `pagination_descending_fallback`
- `only_my_messages_fallback`
- `migrated_history_deferred`
- `finish_takeout_failed`
- `generic_warning`

`message` must be bounded and sanitized. It must not contain raw Telegram
payloads, secrets, auth material, session contents, or compressed blobs.

### Indexes

```sql
CREATE INDEX idx_ingest_batches_source_started
ON ingest_batches(source_id, started_at DESC);

CREATE INDEX idx_ingest_batches_status
ON ingest_batches(status);

CREATE INDEX idx_telegram_takeout_batches_account
ON telegram_takeout_batches(account_id);

CREATE INDEX idx_ingest_item_observations_batch
ON ingest_item_observations(batch_id);

CREATE INDEX idx_ingest_item_observations_item
ON ingest_item_observations(item_id)
WHERE item_id IS NOT NULL;

CREATE INDEX idx_ingest_item_observations_identity
ON ingest_item_observations(source_id, provider_identity_kind, provider_identity);

CREATE INDEX idx_ingest_item_observations_batch_outcome
ON ingest_item_observations(batch_id, outcome);

CREATE INDEX idx_ingest_batch_warnings_batch
ON ingest_batch_warnings(batch_id);
```

## Runtime Flow

### Start And Locking

`start_takeout_source_import` should validate source/account and acquire
`SourceIngestLocks` for the source before creating the in-memory job and
durable batch rows.

This preserves the existing same-source writer invariant shared by normal sync,
Takeout import, and delete. It also prevents durable `running` batches for work
that never actually started because of lock conflict.

On lock conflict, the command returns the existing conflict error and creates
no durable side effects:

- no `ingest_batches`
- no `telegram_takeout_batches`
- no warnings
- no observations

After lock acquisition:

- create the in-memory `TakeoutImportJobRecord`;
- create `ingest_batches` and the matching `telegram_takeout_batches` row in
  one transaction. If the detail insert fails, no orphan `ingest_batches` row
  may remain.

The `ingest_batches` row starts with `provider = 'telegram'`,
`ingest_kind = 'takeout'`, `status = 'running'`, and
`completeness = 'unknown'`.

The in-memory job record should gain `batch_id` as a durable correlation id.
This does not require a batch-history UI in this slice.

### Source Resolution And Takeout Session

After `resolve_and_refresh_peer`, update `telegram_takeout_batches`:

- `resolved_peer_kind`
- `resolved_peer_id`
- `history_peer_kind`
- `history_peer_id`

For foundation-only Takeout, the imported history domain is the current
requested history domain. Migrated history may be detected and deferred, but is
not imported.

After `initTakeoutSession`, write `takeout_id`.

`used_export_dc` and `fallback_used` are updated when the relevant export-DC
path is actually attempted:

- when the shifted export DC path is attempted, set `export_dc_id` and
  `used_export_dc = 1`;
- when fallback to the home DC happens, set `fallback_used = 1` and insert an
  `export_dc_fallback` warning.

Telegram RPC errors must not be hidden by export-DC fallback.

### Splits, Counts, And History Scope

After split range loading and selection:

- write `split_count`;
- write `selected_split_count`;
- write nullable `message_count_estimate` when count probes succeed.

If `detect_supergroup_migration` detects migrated small-group history:

- `migrated_history_detected = 1`;
- `migrated_history_imported = 0`;
- `history_scope = 'current_history_with_migrated_deferred'`;
- insert a `migrated_history_deferred` warning.

If the only-my-messages fallback is used:

- `only_my_messages = 1`;
- `history_scope = 'partial_private_history'`;
- insert an `only_my_messages_fallback` warning.

If both kinds of partiality happen, use `history_scope = 'mixed_partial'`.

### Item Observations

The existing `insert_telegram_source_item(...) -> bool` is too narrow for
Takeout provenance. The implementation should add a boundary that records
provenance without duplicating storage logic, for example:

```rust
insert_telegram_source_item(...) -> TelegramItemInsertOutcome
record_ingest_observation(tx, batch_id, outcome)
```

or a single helper:

```rust
insert_telegram_source_item_with_observation(...)
```

The required invariant is stronger than the exact function name:

```text
item insert / duplicate lookup / telegram_messages write /
topic membership write / observation write happen in one SQLite writer
transaction.
```

Outcomes:

- `inserted { item_id }`
- `duplicate_observed { item_id }`
- `skipped { reason_code }`
- `failed { reason_code }` for item-level non-fatal failures if implemented

Parse-empty or unsupported items should produce `skipped` observations when the
provider identity is known. If provider identity cannot be derived safely, the
implementation may either skip observation with a batch-level warning or write
a `failed` observation only if a safe `provider_identity` exists.

### Counters

Runtime may increment counters for cheap progress display, but terminal batch
updates must recalculate counters from durable rows:

- `item_observed_count`
- `item_inserted_count`
- `item_duplicate_count`
- `item_skipped_count`
- `warning_count`

This avoids drift between batch summaries and observation/warning rows.

### Terminal Status

Success path:

1. `finishTakeoutSession(success=true)`.
2. `finalize_sync(...)`.
3. Recalculate durable counters and warning count.
4. Set batch `status = 'completed'`.
5. Set `finished_at`.
6. Set `completeness`.
7. Emit the final in-memory job record.

`sources.last_sync_state` and `sources.last_synced_at` must still advance only
after successful Takeout finish.

Failure/cancel path:

1. Attempt `finishTakeoutSession(success=false)` if a Takeout session started.
2. If finish fails, insert a `finish_takeout_failed` warning.
3. Recalculate durable counters and warning count.
4. Set batch `status = 'failed'` or `status = 'cancelled'`.
5. Set `finished_at`.
6. Set sanitized bounded `terminal_error` for failures.
7. Set `completeness`.
8. Emit the final in-memory job record.

Completeness rules:

- `complete`: terminal `completed`, no partial flags, selected current history
  imported successfully.
- `partial`: any terminal run with observations and failure/cancel.
- `partial`: completed run with `only_my_messages = 1`.
- `partial`: completed run with
  `migrated_history_detected = 1 AND migrated_history_imported = 0`.
- `partial`: completed run with `history_scope = 'mixed_partial'`.
- `unknown`: no observations, or still `running`.

The migrated-history rule is intentionally strict: current history may have
completed, but the Telegram lineage is incomplete.

### Crash Behavior

No automatic startup repair is required in this slice.

Persisted `status = 'running'` survives restart. The UI or query layer may
derive and display an interrupted state when a durable running batch has no
active in-memory job after restart, but the schema does not persist
`interrupted`.

The implementation plan must choose where that derived state is computed:
startup restoration, lazy source-detail loading, source-list loading, or an
internal query helper. This design only fixes the persisted schema behavior.

## API And Query Boundaries

Public API remains intentionally minimal.

- `start_takeout_source_import` still returns `{ job_id }`.
- `cancel_takeout_source_import` remains unchanged.
- `list_takeout_source_import_jobs` still returns current-session in-memory
  jobs.
- `TakeoutImportJobRecord` gains `batch_id` as a durable correlation id.

Adding `batch_id` does not mean the UI must show batch history in this slice.

Internal storage/query helpers may be added for tests and future UI:

- `list_ingest_batches_for_source(source_id, limit)`
- `load_ingest_batch_details(batch_id)`

These are not product API yet. Expose them as Tauri commands only if an
immediate implementation need appears.

## Testing Strategy

### Migration And Schema Tests

- Migration `23.sql` is registered.
- Migration 23 is regular SQL, not runner-managed.
- Fresh schema contains all four tables and expected indexes.
- `CHECK` constraints reject invalid provider, ingest kind, status,
  completeness, outcome, source subtype, and history scope.
- Foreign keys cascade on source delete.
- Observations allow duplicate rows for the same batch/provider identity.
- Warning rows allow duplicate codes for the same batch.

### Runtime Tests

- Takeout creates `ingest_batches` and `telegram_takeout_batches` only after
  acquiring the same-source lock.
- Lock conflict creates no durable side effects in provenance tables.
- Two concurrent `start_takeout_source_import` calls for the same source allow
  at most one durable batch and reject the other through the existing
  same-source lock/conflict behavior.
- Successful Takeout marks the batch `completed`.
- Failure after partial item writes marks the batch `failed`,
  `completeness = 'partial'`, and does not advance source watermark.
- Cancellation after partial item writes marks the batch `cancelled`,
  `completeness = 'partial'`.
- Only-my-messages fallback writes warning, `only_my_messages = 1`,
  `history_scope = 'partial_private_history'`, and terminal partial
  completeness.
- Migrated supergroup detection writes `migrated_history_detected = 1`,
  `migrated_history_imported = 0`,
  `history_scope = 'current_history_with_migrated_deferred'`, and terminal
  partial completeness when completed.
- Inserted item writes `outcome = 'inserted'`, `provider_identity` from
  `telegram_messages.history_peer_*`, and `item_id`.
- Duplicate item writes `outcome = 'duplicate_observed'` and normally resolves
  the existing `item_id`.
- Skipped parse/empty item writes `outcome = 'skipped'` with `reason_code`
  when safe provider identity exists.
- Terminal batch update recalculates counters and warning count from durable
  rows.
- Failed `finishTakeoutSession(success=false)` cleanup writes
  `finish_takeout_failed`.
- Sanitized terminal errors and warning messages are bounded in length and do
  not contain raw JSON/TL-looking payloads, for example raw messages starting
  with `{`, compressed payload dumps, auth material, or secrets.

### Containment Tests And Scans

- Normal `sync_source` does not write `ingest_batches`.
- Migrated-history import remains disabled.
- No observation uniqueness silently collapses repeated observations.
- Warning rows are sanitized and bounded.
- No provenance table stores raw TL payloads, compressed payloads, secrets,
  session contents, auth material, cookies, or headers.

The implementation plan should set concrete maximum lengths for
`terminal_error` and warning `message` values and test those bounds.

## Acceptance Criteria

The slice is complete when:

- Database schema has generic durable ingest batch tables plus Telegram Takeout
  detail table.
- Takeout runtime writes one durable batch per actually-started locked ingest
  attempt.
- Public API remains minimal, with `batch_id` added only as a durable
  correlation id on current-session job records.
- Terminal Takeout outcomes survive restart as persisted `completed`, `failed`,
  `cancelled`, or `running` for crash-interrupted runs.
- UI/query code may derive interrupted display state from `running` with no
  active in-memory job, but the schema does not persist `interrupted`.
- Item-level provenance records inserted, duplicate observed, and skipped
  outcomes.
- Duplicate observations normally link to the existing `item_id`.
- Source watermark still advances only after successful Takeout finish.
- Migrated history remains disabled.
- Migrated-history detection/deferment is durably recorded.
- Documentation explains that this is provenance foundation, not
  migrated-history enablement.
- Targeted Takeout/storage tests pass.
- Full Rust test suite passes.

## Documentation Updates

Update `docs/database-schema.md` after implementation:

- document `ingest_batches`;
- document `telegram_takeout_batches`;
- document `ingest_item_observations`;
- document `ingest_batch_warnings`;
- explain that normal sync does not write provenance yet;
- explain that Takeout uses durable provenance while job events remain
  current-session runtime UI state.
- explain why `item_observed_count` may be greater than inserted + duplicate +
  skipped counts when item-level failed observations exist.

Update `docs/takeout-source-import.md` after implementation:

- replace the current provenance gap section with the new durable batch model;
- document complete vs partial semantics;
- document `running` as crash-interrupted candidate after restart;
- document migrated-history detection/deferment.

Update `docs/database-schema-legacy-analysis.md` only if the remaining backlog
wording needs clarification.

## Follow-Up Work

1. Add a UI for batch history and batch detail inspection.
2. Add startup/query derived interrupted display state.
3. Decide normal `sync_source` provenance granularity: one batch per
   incremental sync, per page, or per source session.
4. Wire normal `sync_source` into generic ingest provenance after that design
   question is settled.
5. Define retention/compaction policy for high-volume observation rows,
   especially repeated Takeout runs over large channels.
6. Enable migrated-history Takeout import after real-data validation.
7. Use provenance rows to drive repair or cleanup tooling for partial imports.
