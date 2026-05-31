# Takeout Small-Group Rich Fixture Checklist

> Status: reusable validation checklist. Source `118` / batch `22` closed the
> first richer small-group fixture on 2026-05-31; detailed sanitized evidence
> lives in `takeout-representative-validation-and-fallback-coverage.md`.

Updated: 2026-05-31

## Purpose

Use this checklist when another completed Telegram small-group Takeout fixture
contains richer reply, media, reaction, forward, or service-message shapes than
the current source `118` / batch `22` baseline.

The goal is to compare Extractum's local storage against the shape Telegram
Desktop's exporter preserves, while keeping Extractum's current product
boundary intact: metadata-first ingest, no media byte downloads, and sanitized
evidence only.

## Reference Anchors

Telegram Desktop export behavior to keep in mind:

- `reference/tdesktop-dev/Telegram/SourceFiles/export/export_api_wrap.cpp`
  initializes Takeout, wraps history requests in `InvokeWithTakeout`, uses
  `InvokeWithMessagesRange`, and falls back to `messages.search(from_id=self)`
  for private channel/supergroup history.
- `reference/tdesktop-dev/Telegram/SourceFiles/export/data/export_data_types.cpp`
  parses reply ids, reply peer ids, reactions, media summaries, polls, service
  actions, and migrated-history message id adjustment.
- `reference/tdesktop-dev/Telegram/SourceFiles/export/output/export_output_json.cpp`
  serializes reply, media, poll, service-action, and reaction fields into
  export output.

Extractum comparison anchors:

- `src-tauri/src/takeout_import/pagination.rs`
- `src-tauri/src/takeout_import/raw_parse.rs`
- `src-tauri/src/takeout_import/mod.rs`
- `src-tauri/src/source_ingest.rs`

## Safety Boundary

Do not paste message text, source titles, usernames, phone numbers, account
labels that identify a person/source, session data, auth material, headers,
cookies, raw TL payloads, compressed payload dumps, warning bodies, or
screenshots that reveal private content.

Paste only sanitized aggregate counts, local numeric ids, source subtype,
warning codes, coarse terminal outcomes, flags, and capped sample ids.

## Fixture Selection

A useful richer small-group fixture should be a Telegram source with:

- `sources.source_type = telegram`;
- `sources.source_subtype = group`;
- `telegram_sources.peer_kind = chat`;
- a completed Takeout run, not a failed/cancelled/partial run;
- at least one richer shape beyond plain text, preferably from the checklist
  below.

Prefer fixtures that contain several of these shapes:

- replies inside the small group;
- reply metadata with `reply_to_msg_id`;
- thread/topic-like `reply_to_top_id` if Telegram exposes it;
- media-bearing rows: photo, document/file, image, video, voice, audio,
  sticker, poll, webpage, contact, location, venue, or dice;
- aggregate reactions, including explicit zero vs unavailable distinction;
- forwarded metadata or saved-from context;
- service actions such as pin, group title/photo edits, user add/delete, clear
  history, or migrate-to-supergroup.

## Required Evidence

Capture before and after:

- app commit and clean/dirty working tree state;
- source identity shape;
- source item snapshot;
- Takeout batch summary;
- warning-code summary;
- duplicate observation summary;
- row-fidelity comparison for the completed batch;
- content-kind distribution;
- media-kind distribution;
- reply/reply-top/reaction aggregate counts;
- topic membership counts, expected to stay zero for ordinary small groups.

## Storage Checks

For the completed batch, verify:

- `telegram_takeout_batches.status = completed`;
- `completeness = complete`;
- `history_scope = current_history`;
- `only_my_messages = 0`;
- `migrated_history_detected = 0`;
- `migrated_history_imported = 0`;
- `warnings = 0`, unless the fixture intentionally tests a warning path;
- `sources.last_sync_state` and `last_synced_at` advance only after successful
  Takeout finish;
- every observed inserted or duplicate row has a matching
  `telegram_messages` row;
- `telegram_messages.history_peer_kind = chat` for the small-group history;
- duplicate identity uses `(source_id, history_peer_kind, history_peer_id,
  telegram_message_id)`, not only `(source_id, external_id)`;
- row-fidelity comparison reports no missing canonical identities.

## Metadata Checks

For rows where Telegram exposes the field, verify:

- text rows keep trimmed content and correct `content_kind`;
- media-only rows are stored when useful media metadata exists;
- text-with-media rows preserve both text and media metadata;
- `items.has_media`, `media_kind`, and `media_metadata_zstd` match the coarse
  media shape;
- photo metadata keeps best available width, height, and size when exposed;
- document metadata keeps file name, mime type, size, dimensions, duration, and
  derived kind when exposed;
- poll rows are represented at least as media kind `poll`;
- `reply_to_msg_id`, `reply_to_peer_kind`, `reply_to_peer_id`, and
  `reply_to_top_id` are populated when exposed;
- `reaction_count = 0` means Telegram explicitly exposed zero reactions;
- `reaction_count IS NULL` means unavailable or not exposed;
- raw payload storage remains compressed and is not copied into validation
  notes.

## Known Product Gaps

Do not fail the checklist solely because Extractum does not currently preserve
Telegram Desktop's full export surface for:

- media bytes, thumbnails, or custom emoji document downloads;
- detailed reaction actor lists;
- full poll answer/vote details;
- inline button rows;
- forwarded/saved-from metadata;
- service-action-specific structured fields beyond current stored text/media
  context.

If the richer fixture shows that one of these gaps matters for browsing,
analysis, or export, open a separate backlog item or spec before changing the
ingest contract.

## Result Template

```text
Date:
App commit:
Working tree:
Source id:
Batch id:
Fixture shape:
Completed Takeout:
Warnings:
Inserted / duplicate / skipped / observed:
Row-fidelity result:
Reply metadata result:
Media metadata result:
Reaction metadata result:
Product gap notes:
Decision:
```
