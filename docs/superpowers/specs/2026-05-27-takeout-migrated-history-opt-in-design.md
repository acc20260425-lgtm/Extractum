# Takeout Migrated-History Opt-In Design

## Goal

Define the explicit opt-in behavior required before Extractum can import old
Telegram small-group history that existed before a group was upgraded to a
supergroup.

This design builds on the 2026-05-26 policy decision: migrated small-group
history is a separate historical scope and must not be silently imported by a
normal current-history Takeout rerun.

The goal of this slice is to specify the product, storage, provenance, and test
contract. It does not implement the importer yet.

## Reference Model

Telegram Desktop treats migrated group history as a sibling history, not as the
same native history.

The client records a two-way relationship between the old group chat and the
new megagroup:

- old `ChatData` points to `migrateToChannel`;
- new `ChannelData` points to `migrateFromChat`;
- the UI can render current history and migrated history as one continuous
  conversation.

For export, Telegram Desktop also keeps the old input peer distinct. Its export
flow stores a `migratedFromInput`, attaches migrated slices to the current
dialog, and adjusts presentation message ids for migrated messages to avoid
collisions.

Extractum should adopt the same conceptual boundary, but not the same
presentation-id trick as primary storage. Extractum is a durable local archive,
so native peer identity and historical scope should remain explicit in storage
and provenance.

## Current State

Normal Takeout import already detects migrated supergroups and records
deferment:

- `migrated_history_detected = 1`;
- `migrated_history_imported = 0`;
- warning code `migrated_history_deferred`;
- `history_scope = current_history_with_migrated_deferred`;
- one durable warning per batch;
- zero old `chat` history rows.

Source `115` / batch `18` is the representative safe-deferment evidence. It
validates detection and deferment only. It does not validate migrated-history
import enablement.

## Chosen Approach

Use one local source with separate history domains.

The current supergroup remains the source the user sees and reruns by default.
Old small-group history is modeled as an opt-in historical domain attached to
that source:

```text
source
  current_history
  migrated_small_group_history
```

Normal Takeout reruns continue to import only `current_history`. A separate
explicit action imports `migrated_small_group_history` when the user chooses to
bring in messages from before the group upgrade.

This keeps analysis and recovery centered on one Telegram source while
preserving the data-integrity boundary between old `chat` history and current
`channel` / supergroup history.

## User-Facing Contract

The opt-in action should be phrased as importing older history from before the
group was upgraded to a supergroup. It must not look like a generic retry.

The user-facing contract is:

- normal reruns refresh or re-import current supergroup history;
- older small-group history remains deferred unless explicitly imported;
- the historical import may add older messages to the archive;
- imported historical rows remain labeled as pre-upgrade / migrated history;
- cancelling or failing the historical import does not corrupt current history.

Recovery copy should continue to explain `migrated_history_deferred` as an
intentional historical-scope boundary, not as a transient failure.

## Data Model Contract

The primary storage identity should remain native and scoped. The importer must
not rewrite old `chat` messages into current `channel` identity.

The primary duplicate identity remains native Telegram identity:

```text
source_id
history_peer_kind
history_peer_id
telegram_message_id
```

Existing fields such as `history_peer_kind`, `history_peer_id`,
`telegram_message_id`, `migration_domain`, and `is_migrated_history` should be
used where possible. Schema changes are acceptable only if the implementation
proves the current model cannot express the historical-domain contract clearly.

Historical domain is a required classification and invariant, not part of the
dedupe key by default. It must not be added to a unique index unless an
implementation proves that one native Telegram message
`(history_peer_kind, history_peer_id, telegram_message_id)` can legitimately
belong to two historical domains inside the same source.

Expected history-domain semantics:

- `current_history` for the current supergroup history;
- `migrated_small_group_history` for the old group history.

It must be valid for the same Telegram `message_id` to exist once in current
history and once in migrated history, because those ids belong to different
native Telegram histories.

For migrated small-group import, the native identity invariant is stricter:

- `migrated_small_group_history` rows must use `history_peer_kind = chat`;
- `history_peer_id` must be the old `migrated_from_chat_id` history peer, not
  the current supergroup/channel peer;
- `is_migrated_history = 1` rows must not be normalized to the current source
  peer identity.

The current SQLite schema protects duplicate identity, but it cannot by itself
prove that a migrated row used the old chat peer rather than the current
channel peer. The first implementation should enforce this in Rust identity
construction and deterministic tests. A database `CHECK` such as "migrated rows
must use chat peer kind" can be added only if the implementation commits
`is_migrated_history` to mean this migrated-from-chat domain exclusively.

If a merged timeline needs a single display id later, that id should be a
read-model or export-model projection, not the primary storage key.

## Schema Impact

The first implementation should prefer existing columns:

- `history_peer_kind`;
- `history_peer_id`;
- `telegram_message_id`;
- `migration_domain`;
- `is_migrated_history`.

`migration_domain` is currently documented as diagnostic / future-proofing
metadata and is not used for duplicate detection, topic matching, or reference
resolution. If the historical importer makes `migration_domain` a functional
storage contract, the implementation plan must explicitly decide:

- whether to reuse `migration_domain` or add a separate field;
- the allowed values;
- whether a database `CHECK` constraint is needed;
- which schema docs and tests need updates;
- whether analysis rows, archive read models, export DTOs, and diagnostics
  should expose, filter, or preserve the domain marker.

For this design, the only allowed migrated-history domain value is:

```text
migrated_from_chat
```

The implementation should introduce this as a shared Rust constant rather than
spelling the string in multiple call sites. Additional domain values require a
separate design update because they may have different peer-kind and
dedupe-safety rules.

If the first implementation keeps `migration_domain` as classification metadata
only, tests must still prove that old rows are marked consistently with
`is_migrated_history = 1` and native old-`chat` identity.

## Provenance Contract

The historical import needs its own batch provenance. A completed historical
run should be distinguishable from a normal current-history run.

Recommended provenance shape:

```text
history_scope = migrated_small_group_history
trigger = explicit_user_opt_in
migrated_history_detected = 1
migrated_history_imported = 1 only after rows are actually imported
```

The existing current-history batch should remain truthful:

```text
history_scope = current_history_with_migrated_deferred
migrated_history_detected = 1
migrated_history_imported = 0
warning = migrated_history_deferred
```

Historical import failure must not retroactively mark a current-history batch
as failed. It should produce its own failed or partial historical batch with
sanitized warning codes.

## Historical Capability And Availability

The opt-in UI and backend need durable knowledge that a migrated historical
scope exists. The first implementation plan must choose where this availability
state lives.

Acceptable first version:

- current normal Takeout detection records durable provenance that
  `migrated_history_detected = 1`;
- UI can offer historical import after restart from sanitized source recovery
  or provenance state;
- starting historical import revalidates the current supergroup with
  `channels.getFullChannel`;
- row writes only proceed when `migrated_from_chat_id` is currently available
  and the old chat input can be opened.

If a previous batch detected migrated history but a later validation no longer
returns `migrated_from_chat_id`, the historical command should fail or remain
disabled with sanitized unavailable-scope state. It must not guess old peer
identity from docs, UI text, warning bodies, or private payloads.

If storing old-chat access hints becomes necessary for restart-safe import, the
implementation must treat them as private source capability state. They may be
used internally, but must not appear in tracked docs, diagnostics, recovery
DTOs, or UI copy.

## Import Flow

Normal flow remains:

```text
resolve current supergroup
detect migrated_from_chat_id
record migrated_history_deferred when present
import current supergroup history
finalize current-history batch
```

Opt-in historical flow should be separate:

```text
resolve current supergroup
confirm migrated_from_chat_id is available
ask for or receive explicit historical import intent
open old chat input peer
import old chat messages into migrated_small_group_history
mark migrated rows with native chat identity and migrated flags
finalize historical-scope batch
```

The historical flow should be idempotent. Repeating it should update or skip
already-imported rows without duplicating them and without changing current
history rows.

## Watermark Semantics

Historical import must not advance the current-history source watermark.

For the first implementation, historical import should write historical rows
and historical batch provenance only. It should not update
`sources.last_sync_state` or `sources.last_synced_at` as though current
supergroup history had been synchronized.

If a later design needs historical watermarks, they should be separate from the
current-history `sources.last_sync_state` contract.

## Command And Locking Contract

Historical import should have its own backend command instead of a flag on the
normal command. The intended shape is:

```text
start_takeout_migrated_history_import(source_id) -> { job_id }
```

This keeps `start_takeout_source_import(source_id)` current-history-only and
makes accidental old-history import harder.

Historical import must use the same same-source ingest lock as normal sync,
normal Takeout, and delete:

- no current Takeout and historical Takeout may run for the same source at the
  same time;
- no normal `sync_source` may run for the same source during historical import;
- no `delete_source` may run for the same source during historical import;
- different sources may continue to ingest independently.

Batch or job records should be created only after the same-source lock is
acquired, matching the normal Takeout lock contract.

## Read Model

The first implementation does not need to build a fully merged timeline.

Minimum acceptable read behavior:

- current history remains visible exactly as today;
- historical rows, once imported, can be identified as older migrated history;
- current history remains the default corpus for browsing, analysis, reports,
  and NotebookLM export;
- historical rows are visible in diagnostics and may be included by an explicit
  domain/scope option when a reader or exporter supports it.

A later merged view can combine current and migrated history for reading, but
that view must keep provenance and native peer identity recoverable.

## Error Handling

The following are safe outcomes:

- current-history Takeout completes as partial with
  `migrated_history_deferred`;
- historical opt-in import completes independently;
- historical opt-in import fails or is cancelled without changing current
  history state;
- repeated historical imports are idempotent.
- historical opt-in import leaves current-history `sources.last_sync_state` and
  `sources.last_synced_at` unchanged.

The following are data-integrity failures:

- normal current-history Takeout imports old `chat` rows;
- old `chat` rows are stored as current `channel` rows;
- historical rows lose `is_migrated_history` or equivalent domain labeling;
- `migrated_history_imported = 1` is recorded before actual historical rows are
  imported;
- a failed historical import rewrites the current-history batch status.
- historical import changes current-history source watermarks.

## Privacy Boundary

Diagnostics and docs may mention local source ids, batch ids, history scopes,
warning codes, status, completeness, and aggregate counters.

Diagnostics and docs must not expose Telegram message text, source titles,
usernames, phone numbers, raw peer ids beyond existing local-safe ids, access
hashes, session material, raw TL/provider payloads, headers, cookies, or raw
warning bodies.

## Test Strategy

The implementation plan should start with deterministic tests.

Required tests before enabling row writes:

- normal migrated-supergroup Takeout still records
  `migrated_history_deferred` and writes zero old `chat` rows;
- the deferment warning remains once-only per batch;
- overlapping Telegram message ids are accepted when native history peer
  identity differs;
- the same native Telegram message is still deduplicated even if domain metadata
  is inconsistent;
- migrated-history identity construction rejects or cannot produce rows
  normalized to the current channel peer;
- `migrated_from_chat` is centralized as the only domain marker for this
  historical scope;
- recovery copy describes migrated history as a separate historical scope.

Required tests for historical import enablement:

- historical import uses a separate backend command and requires explicit
  opt-in intent;
- historical import revalidates or loads durable availability for the migrated
  historical scope;
- old rows are written under native `chat` identity and migrated domain flags;
- current rows and historical rows with the same `telegram_message_id` do not
  conflict;
- rerunning historical import is idempotent;
- failed or cancelled historical import leaves current-history provenance
  unchanged;
- historical import does not advance current-history `sources.last_sync_state`
  or `sources.last_synced_at`;
- same-source ingest locking blocks concurrent current Takeout, historical
  Takeout, sync, and delete for the same source;
- sanitized diagnostics expose warning codes and counters only.

## Non-Goals

- Do not enable historical import in this design-only slice.
- Do not make normal Takeout reruns import old small-group history.
- Do not add destructive recovery actions.
- Do not merge old `chat` identity into current `channel` identity.
- Do not design full merged timeline UI in this slice.
- Do not include historical rows in default analysis, reports, or NotebookLM
  export in the first implementation.
- Do not clear old Telegram metadata blobs as part of this work.

## Acceptance

- The opt-in behavior is defined as a separate historical import action, not a
  retry of current-history Takeout.
- Storage keeps current supergroup history and old small-group history in
  separate native history identities, with historical domain as a required
  classification invariant.
- Provenance can distinguish current-history deferment from historical import.
- Historical import does not advance current-history source watermarks.
- Historical import uses a separate command and the same source ingest lock.
- Current history remains the default read and analysis corpus until an
  explicit historical-domain option exists.
- The design preserves existing safe behavior until explicit implementation
  work enables historical row writes.
- The next implementation plan can be written from this contract without
  deciding product semantics again.
