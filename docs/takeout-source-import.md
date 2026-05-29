# Takeout Source Import

This document describes the current Takeout import path for existing Telegram sources.

## 1. Purpose

Takeout import is a second ingest path next to `sync_source`.

`sync_source` remains the fast incremental path. Takeout import is used when the app needs a fuller server-side history profile for one already registered Telegram source.

Both paths write to the same local tables:

- `sources`
- `items`
- `telegram_messages`
- `item_topic_memberships` when topic resolution state is ready/current

Telegram Takeout also writes durable ingest provenance after the same-source
ingest lock is acquired. Telegram duplicate rows are protected by typed native
identity in `telegram_messages`, not by `(source_id, external_id)` alone.

## 2. User-Facing Behavior

The analysis workspace exposes a `Takeout` action for each source next to the normal `Sync` action.

An active Takeout job:

- emits `sources://takeout-import` events;
- shows phase, progress, inserted/skipped counts, warnings, and terminal error state;
- can be cancelled cooperatively;
- disables `Sync`, `Takeout`, and `Delete` for the same source while it is active.

Job state is in memory. If the app restarts, active job records are lost, but any rows already inserted into `items` remain.

After restart, the analysis workspace can also show a read-only recovery notice
for the latest durable Telegram Takeout batch for a source. This notice is not a
job and has no cancel, resume, purge, or retry semantics.

Recovery notice priority is:

1. active in-memory Takeout job;
2. latest durable recovery state;
3. no Takeout notice.

The suggested recovery path is to run Takeout again. Existing messages already
saved locally are deduplicated by typed Telegram identity.

## 3. Backend Contract

Public commands:

- `start_takeout_source_import(source_id) -> { job_id }`
- `cancel_takeout_source_import(job_id) -> { cancelled }`
- `list_takeout_source_import_jobs() -> TakeoutImportJobRecord[]`

Diagnostic command:

- `run_takeout_export_dc_spike(source_id)`

The event payload for `sources://takeout-import` is the full latest `TakeoutImportJobRecord`, not a partial patch. Frontend state can replace the local job record by `job_id`.

The job record tracks:

- `batch_id` as the durable provenance correlation id;
- source/account identity;
- status and phase;
- inserted/skipped counts;
- optional bounded progress;
- cumulative warnings;
- terminal error text.

## 4. Same-Source Locking

`SourceIngestLocks` serializes mutating work for a single source.

The lock is shared by:

- normal `sync_source`;
- Takeout import;
- `delete_source`.

Different sources can still ingest independently. The important invariant is that two writers do not write or delete the same `source_id` at the same time.

Durable Takeout batch rows are created only after this same-source lock is
acquired. A lock conflict rejects the start without creating batch, detail,
observation, or warning rows.

## 5. Takeout Session Flow

The Takeout path uses raw Telegram TL requests after normal source resolution:

1. Load the existing source and account runtime.
2. Resolve and refresh the source peer through the same source-resolution path used by sync.
3. Start `account.initTakeoutSession` without `InvokeWithTakeout`.
4. Run peer validation and history requests through `InvokeWithTakeout`.
5. Wrap history page requests in `InvokeWithMessagesRange`.
6. Finish the session with `account.finishTakeoutSession(success=true)` on success.
7. Attempt `account.finishTakeoutSession(success=false)` after failure or cancellation once a Takeout session has started.

The implementation tries to use a Telegram Desktop-style export DC alias:

- `export_dc_id = home_dc_id + 4 * 10000`
- cloned home `DcOption` with the shifted id
- `client.invoke_in_dc(export_dc_id, request)`

If the shifted DC path fails for a local transport/session reason, the request falls back to the normal home DC and records a visible warning. Telegram RPC errors are not hidden by the export-DC fallback.

## 6. History Selection

Takeout import targets already registered source kinds:

- `channel`
- `supergroup`
- `group`

Split selection follows the current single-source export model:

- `channel`: import the last split only;
- `supergroup`: import the last split only;
- `group`: import all returned splits;
- empty `messages.getSplitRanges` response: use fallback range `1..i32::MAX`.

For supergroups, `channels.getFullChannel` is used to detect
`migrated_from_chat_id`. Migrated small-group history is treated as a separate
historical scope. Normal Takeout imports keep importing current supergroup
history only and record `migrated_history_deferred` when the historical scope is
detected. The storage layer can represent overlapping Telegram message ids
through `telegram_messages`, but importing old `chat` history requires the
explicit opt-in historical-scope action described below.

## Migrated Small-Group History Opt-In

Normal Takeout imports current source history only. When a supergroup exposes a
`migrated_from_chat_id`, Extractum records source-level availability and keeps
the old small-group history deferred.

The explicit command is:

- `start_takeout_migrated_history_import(source_id) -> { job_id }`

The command uses the same source ingest lock as sync, delete, and current
Takeout. It revalidates the current supergroup before opening the old
`InputPeerChat`. If revalidation fails, the backend returns a typed
`migrated_history_unavailable` conflict and records an internal availability
reason.

Historical import writes rows with native old chat identity:

- `history_peer_kind = chat`
- `history_peer_id = migrated_from_chat_id`
- `is_migrated_history = 1`
- `migration_domain = migrated_from_chat`

Historical import does not update `sources.last_sync_state` or
`sources.last_synced_at`. The first implementation does not materialize
historical rows into `analysis_documents`, `archive_read_items`, or
`item_topic_memberships`, and default browsing, analysis, reports, and
NotebookLM export stay current-history-only.

### Historical Scope Usage

Imported migrated small-group history remains outside default browsing,
analysis, and NotebookLM export. The source reader defaults to
`Current supergroup history`. Users can explicitly switch to
`Migrated small-group history` or `Merged timeline` when imported migrated rows
exist.

Analysis and export use separate explicit opt-ins. Analysis records the
run-level decision in `analysis_runs.telegram_history_scope`; exported
NotebookLM files render current and migrated history as separate sections.

## 7. Pagination Contract

The production path is `TDesktop` first with a per-split `DescendingFallback`.

`TDesktop` profile:

- every split starts with `largest_id_plus_one = 1`;
- each page uses `offset_id = largest_id_plus_one`, `add_offset = -100`, `limit = 100`;
- raw Telegram page order is normalized from newest-to-oldest into oldest-to-newest before persistence;
- the next cursor is `newest_message_id + 1`;
- the cursor resets when the import moves to the next split;
- `messages.Messages` is terminal for the split;
- `messagesNotModified` is a hard error.

This must not be simplified to only setting `add_offset = -100`. The cursor movement depends on the same state machine Telegram Desktop uses: request profile, raw response reversal, cursor advancement, and reset per split.

`DescendingFallback` profile:

- starts at `offset_id = range.max_id`;
- uses `add_offset = 0`;
- advances through `min(message_id)` from the accepted page.

Fallback is Extractum-specific safety behavior. It is used only when the first TDesktop page is empty despite a nonzero count, or when the TDesktop cursor does not advance. It restarts only the current split and keeps any already inserted rows.

## 8. Private History Fallback

For `channel` and `supergroup` sources, a `CHANNEL_PRIVATE` error switches the current history task to an only-my-messages fallback:

- `messages.getHistory` is replaced with `messages.search`;
- `from_id = inputPeerSelf`;
- the same pagination profile continues;
- a warning is recorded.

This can import only the current user's visible/outgoing subset. That warning should remain visible to the user because the archive is incomplete by Telegram permission design, not by local storage failure.

## 9. Persistence Semantics

Takeout import writes through the Telegram item insert helper and records
item-level observations for inserted, duplicate-observed, and skipped Telegram
messages.

Inserted rows preserve the same storage dimensions as normal sync where raw TL data exposes them:

- content text;
- content kind;
- lightweight media metadata;
- nullable reply/thread/reaction context;
- compressed raw payload.

Takeout import does not download media bytes, thumbnails, previews, custom emoji documents, or other Telegram Desktop export assets.

Successful Takeout import updates `sources.last_sync_state` and
`sources.last_synced_at` only after `finishTakeoutSession(success=true)` and
`finalize_sync(...)` succeed. Failed and cancelled jobs leave partial inserted
rows in `items`, but they do not advance the source watermark.

Inserted Telegram rows also receive typed native identity in
`telegram_messages`. If a source's forum topic resolver state is ready/current,
the insert helper resolves scoped `item_topic_memberships` in the same
transaction; otherwise topic membership waits for the next full source rebuild.

Telegram Takeout imports now create durable ingest provenance after the
same-source ingest lock is acquired. The in-memory job remains the current UI
state mechanism, and `batch_id` is a correlation id for tests and future UI.

Successful Takeout marks the batch `completed`. Failed and cancelled runs mark
the batch `failed` or `cancelled` and leave already inserted rows linked to the
batch through item observations. Source watermarks still advance only after
`finishTakeoutSession(success=true)` and `finalize_sync(...)` succeed.

`running` batches survive restart. The schema does not persist an
`interrupted` status. The recovery query derives `interrupted` when the latest
Telegram Takeout batch for a source is still `running` and no active in-memory
Takeout job exists for that source.

Recovery selection uses latest-attempt-wins semantics. Older failed or
cancelled Takeout batches are hidden if a newer complete Takeout batch exists.
A newer failed, cancelled, interrupted, or partial completed Takeout batch is
shown even when an older complete attempt exists.

The read-only recovery DTO exposes counts, sorted warning codes, bounded failed
terminal error detail, timestamps, durable status, and derived recovery kind. It
does not expose warning messages, Telegram payloads, message text, source
identity details, account/session/API data, or full provenance history.

Old small-group historical scope remains disabled for normal Takeout reruns.
When it is detected, Takeout records `migrated_history_detected = 1`,
`migrated_history_imported = 0`, a `migrated_history_deferred` warning, and
partial completeness. The selected policy treats old small-group history as a
separate historical scope, so read-only recovery state does not enable
migrated-history import, resume, purge, or automatic retry.

The current Takeout path finalizes source state after a successful import. The
completed supergroup Takeout path also reuses the forum-topic refresh helper
after successful Takeout finish and export-DC provenance, before source and
batch finalization. Failed, cancelled, and interrupted Takeout attempts do not
refresh forum topics. Actionable refresh failures keep the Takeout batch
completed and record durable warning code `forum_topic_refresh_failed`.

## 10. Current Validation Baseline

Recorded baseline from the Takeout pagination work:

- `cargo test takeout_import` passed with 20 tests.
- full `cargo test` passed with 130 tests.
- the tested live public channel imported without a descending fallback warning and had 1009 local `items` rows, 1008 of them text records.
- a later NotebookLM export with only 14 messages was traced to the selected 30-day analysis period, not to Takeout pagination.

Open validation still belongs in the backlog: broader real-account coverage for
supergroups, groups, private/left sources, and explicit opt-in behavior for the
migrated historical scope before old small-group history import is enabled.
