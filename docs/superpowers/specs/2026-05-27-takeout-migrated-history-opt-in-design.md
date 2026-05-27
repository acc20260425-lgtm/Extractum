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

The durable uniqueness model should be equivalent to:

```text
source_id
history_peer_kind
history_peer_id
telegram_message_id
history_domain
```

Existing fields such as `history_peer_kind`, `history_peer_id`,
`telegram_message_id`, `migration_domain`, and `is_migrated_history` should be
used where possible. Schema changes are acceptable only if the implementation
proves the current model cannot express the historical-domain contract clearly.

Expected domains:

- `current_history` for the current supergroup history;
- `migrated_small_group_history` for the old group history.

It must be valid for the same Telegram `message_id` to exist once in current
history and once in migrated history, because those ids belong to different
native Telegram histories.

If a merged timeline needs a single display id later, that id should be a
read-model or export-model projection, not the primary storage key.

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

## Read Model

The first implementation does not need to build a fully merged timeline.

Minimum acceptable read behavior:

- current history remains visible exactly as today;
- historical rows, once imported, can be identified as older migrated history;
- analysis and diagnostics can include or exclude the historical domain
  deliberately.

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

The following are data-integrity failures:

- normal current-history Takeout imports old `chat` rows;
- old `chat` rows are stored as current `channel` rows;
- historical rows lose `is_migrated_history` or equivalent domain labeling;
- `migrated_history_imported = 1` is recorded before actual historical rows are
  imported;
- a failed historical import rewrites the current-history batch status.

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
- overlapping Telegram message ids are accepted across current and migrated
  domains;
- recovery copy describes migrated history as a separate historical scope.

Required tests for historical import enablement:

- historical import requires explicit opt-in intent;
- old rows are written under native `chat` identity and migrated domain flags;
- current rows and historical rows with the same `telegram_message_id` do not
  conflict;
- rerunning historical import is idempotent;
- failed or cancelled historical import leaves current-history provenance
  unchanged;
- sanitized diagnostics expose warning codes and counters only.

## Non-Goals

- Do not enable historical import in this design-only slice.
- Do not make normal Takeout reruns import old small-group history.
- Do not add destructive recovery actions.
- Do not merge old `chat` identity into current `channel` identity.
- Do not design full merged timeline UI in this slice.
- Do not clear old Telegram metadata blobs as part of this work.

## Acceptance

- The opt-in behavior is defined as a separate historical import action, not a
  retry of current-history Takeout.
- Storage keeps current supergroup history and old small-group history in
  separate native history domains.
- Provenance can distinguish current-history deferment from historical import.
- The design preserves existing safe behavior until explicit implementation
  work enables historical row writes.
- The next implementation plan can be written from this contract without
  deciding product semantics again.
