# Source Provider Readiness Design

## Purpose

Extractum will add new source families after Telegram: manual YouTube video
links first, YouTube playlists later, and possibly RSS feeds and forums after
that. This design prepares the existing source model and source UI for those
families without implementing any new provider yet.

The goal is to remove the shared assumption that every source is Telegram while
keeping the current Telegram behavior stable.

## Decisions From The Design Discussion

- Manual YouTube links should be supported before playlist sync.
- YouTube playlists may become syncable sources later.
- YouTube analysis content should use transcript, description, and metadata
  when an ingest provider is available.
- The transcript and metadata retrieval mechanism is intentionally abstract for
  now. The design must not depend on YouTube Data API, scraping, or an external
  pipeline.
- The accepted architecture is a shared source core with provider capabilities,
  not a full plugin platform.

## Source Core Model

The shared source record should distinguish provider identity from
provider-specific details.

- `source_type` is the top-level provider family. Existing Telegram sources use
  `telegram`; future values include `youtube`, `rss`, and `forum`.
- `source_subtype` is an optional provider-local subtype. Telegram uses
  `channel`, `supergroup`, and `group`; YouTube can later use `video` and
  `playlist`; RSS can use `feed`; forums can use `thread`, `board`, or `site`.
- `telegram_source_kind` should stop being required by frontend and shared
  backend contracts. It can remain as compatibility storage while Telegram code
  is migrated behind provider-specific boundaries.
- `account_id` remains nullable and means "provider account" rather than
  "Telegram account" in shared code. Manual YouTube video links can use `null`.
- `external_id` remains a provider-local identifier. Shared code must not assume
  it is numeric or Telegram-specific.
- `metadata_zstd` stores provider-specific details such as Telegram peer
  identity, YouTube URL, thumbnail, channel information, duration, transcript
  language, and provider cursor data.
- `last_sync_state` should be treated as a provider cursor in shared code. If a
  future migration renames it to `sync_cursor`, that should be a compatibility
  refactor rather than a requirement for the first readiness pass.

## Source Capabilities

Shared UI and workflow code should decide which actions to display from source
capabilities rather than checking Telegram-specific fields.

Suggested capability shape:

```ts
interface SourceCapabilities {
  canSync: boolean;
  canDelete: boolean;
  canImportArchive: boolean;
  hasTopics: boolean;
  requiresAccount: boolean;
  hasMembershipState: boolean;
  contentLabel: "messages" | "videos" | "posts" | "items";
}
```

Examples:

- Telegram channel or group: syncable, account-backed, membership-aware.
- Telegram supergroup: syncable, account-backed, membership-aware, topic-aware,
  and archive-import capable.
- Manual YouTube video: deletable, not syncable, no account required.
- Future YouTube playlist: syncable and video-labeled, without Telegram runtime.
- Future RSS feed: syncable and post-labeled.
- Future forum source: likely topic-aware and post-labeled.

Capabilities may initially be derived in frontend helper code from
`source_type`, `source_subtype`, and Telegram compatibility fields. The backend
can later return capabilities directly if duplication becomes a drift risk.

## Backend Boundaries

The backend should be split around a shared source core and provider modules.
This avoids a full provider plugin registry while giving each source family a
clear place to live.

Target responsibilities:

- `sources/core.rs`: shared persistence and source item primitives such as
  listing sources, deleting sources, loading source records, building shared
  `SourceRecord` values, and inserting common corpus items.
- `sources/telegram/*`: Grammers runtime access, Telegram account requirements,
  peer resolution, Telegram sync, Telegram forum topics, source avatars, and
  Takeout import support.
- `sources/youtube/*`: future URL parsing, metadata/transcript provider
  adapters, manual video ingest, and playlist sync.

Commands should remain explicit at provider boundaries:

- Keep `add_telegram_source` for Telegram.
- Add a future `add_youtube_source` or `add_url_source` for YouTube/manual URL
  ingestion.
- Do not introduce a broad `add_source({ sourceType, sourceRef })` command
  before a second provider proves the shared command shape.

`sync_source` should become a small dispatcher:

- `telegram` dispatches to the current Telegram sync flow.
- `youtube/video` returns a validation error because manual videos are not
  syncable.
- `youtube/playlist` can dispatch to future playlist sync.
- Unknown source families return validation errors.

The existing `SourceIngestLocks` model should stay keyed by `source_id` so
future sync/import/delete flows share the same active-work coordination model.

## Corpus And Analysis Model

The shared `items` table remains the common analysis corpus, but shared naming
and references should stop treating all items as Telegram messages.

- `items.external_id` is the provider-local item identifier: Telegram message
  id, YouTube video id, RSS entry guid, or forum post id.
- `items.content_zstd` contains the primary text used by analysis. For YouTube
  this can be a composed document containing title, description, metadata, and
  transcript text.
- `items.content_kind` should become or be interpreted as a document shape,
  such as `message`, `video_transcript`, `feed_entry`, or `forum_post`.
- `media_metadata_zstd` can still store shared media metadata such as thumbnail
  and duration. Rich provider-specific metadata should remain in `raw_data_zstd`
  or source metadata.
- Telegram-specific forum topic fields and `telegram_forum_topics` should not
  become the general forum model. A future forum provider should introduce its
  own topic/thread taxonomy when needed.

Analysis and export code should gradually move from "messages" terminology to
"items" or "documents" where behavior is provider-neutral. Compatibility names
can stay during incremental refactors as long as new shared code uses neutral
terms.

Analysis refs should become provider-neutral. The current format is effectively
Telegram-shaped because it uses source id plus provider external id. A safer
future ref is based on the local item identity, for example `s{source_id}-i{item_id}`,
or another snapshot-stored stable ref that does not assume numeric Telegram
message ids.

## Error Handling

Unsupported provider operations should be explicit validation or conflict
errors rather than silent no-ops.

Examples:

- Syncing a manual YouTube video returns a validation error.
- Starting Takeout for a non-Telegram source returns a validation error.
- Listing topics for a source without `hasTopics` returns an empty list only if
  the UI contract treats "not topic-aware" as empty; otherwise it returns a
  validation error.
- Deleting a source should continue to coordinate with active ingest locks.

## Testing Strategy

Readiness tests should prove extensibility without implementing YouTube ingest.

Recommended test coverage:

- `SourceType` and frontend `Source` accept a non-Telegram provider such as
  `youtube`.
- Shared source mapping works when `telegram_source_kind` is absent or null.
- Capability helpers hide or disable Sync, Takeout, topics, and membership UI
  from non-capable sources.
- `sync_source` returns a validation error for a non-syncable source subtype.
- Corpus refs do not depend on numeric Telegram external ids.
- Existing Telegram source listing, sync, forum topic, Takeout, and analysis
  behavior remains covered by current focused tests.

## Non-Goals

This design does not add YouTube, RSS, or forum ingestion. It also does not
choose a YouTube metadata or transcript retrieval mechanism, introduce a
provider plugin platform, migrate every analysis variable name in one pass, or
replace the existing Telegram Takeout implementation.

## Recommended Next Planning Scope

The next implementation plan should target provider readiness only:

- shared source type and record compatibility;
- source capabilities;
- frontend action gating from capabilities;
- backend `sync_source` dispatch and validation;
- provider-neutral corpus refs where practical.

It should avoid adding concrete YouTube/RSS/forum provider functionality.
