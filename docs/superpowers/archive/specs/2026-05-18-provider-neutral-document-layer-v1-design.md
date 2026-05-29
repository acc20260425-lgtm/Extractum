# Provider-Neutral Document Layer v1 Design

> Date: 2026-05-18
> Status: approved direction, pending implementation plan
> Scope: Database Schema Simplification slice, analysis-first reader

## Summary

This slice introduces `analysis_documents`, a provider-neutral materialized
read model for live analysis corpus loading.

`analysis_documents` is not the source of truth. Provider/archive truth remains
in `items` plus typed provider tables such as `telegram_messages`,
`youtube_video_sources`, `youtube_playlist_sources`, and
`youtube_transcript_segments`. The new table is rebuildable from that state and
exists to remove provider-specific branching from the live analysis corpus
reader.

Version 1 is analysis-first:

- add the document read model;
- backfill existing Telegram and YouTube analysis text units;
- synchronously maintain documents from current ingest/write paths;
- switch `load_corpus_messages` to read `analysis_documents`;
- keep source browsing and NotebookLM export on their current readers.

Saved runs still snapshot the loaded live corpus into `analysis_run_messages`.
Later document rebuilds must not affect saved run follow-up, saved evidence, or
trace behavior.

## Current State

`items` is still the shared archive container for Telegram messages, YouTube
transcripts, YouTube comments, media-bearing rows, raw provider payloads, and
provider-specific context fields. Recent simplification slices moved major
provider identity into typed tables:

- Telegram source identity is in `telegram_sources`;
- Telegram message identity is in `telegram_messages`;
- YouTube source runtime metadata is in `youtube_video_sources` and
  `youtube_playlist_sources`;
- Telegram forum topic membership is materialized in
  `item_topic_memberships`.

The live analysis corpus loader still assembles a provider-neutral corpus
directly from provider/archive state. It needs to know which `items.item_kind`
values are analysis text units, how to synthesize YouTube description rows,
how to include or exclude YouTube comments by corpus mode, and how to preserve
YouTube timestamp evidence metadata.

NotebookLM export also reads from `items`, but it has richer Telegram-specific
requirements in this slice: reply snippets, media placeholders, topic labels,
reaction counts, and export-specific rendering. It remains out of scope for v1.

## Goals

1. Add a provider-neutral `analysis_documents` read model for live analysis.
2. Keep provider/archive truth in `items` and typed provider tables.
3. Backfill existing Telegram and YouTube text-bearing analysis units.
4. Make backfill and source rebuild idempotent and deterministic.
5. Preserve current live evidence ref semantics.
6. Switch only `load_corpus_messages` to the new table in v1.
7. Preserve `analysis_run_messages` as the frozen saved-run corpus snapshot.
8. Keep source browsing, source item APIs, and NotebookLM export unchanged.

## Non-Goals

- Do not make `analysis_documents` editable product state.
- Do not remove provider-specific fields from `items`.
- Do not remove `items.content_zstd`, `items.raw_data_zstd`, or
  provider-specific typed tables.
- Do not switch source browsing or `get_items` to `analysis_documents`.
- Do not switch NotebookLM export in v1.
- Do not redesign YouTube playlist entity/removal state.
- Do not replace `analysis_run_messages`.
- Do not introduce a separate `analysis_document_state` table in v1.
- Do not use `analysis_documents.id` as a public evidence ref.
- Do not enable migrated-history Takeout import.

## Data Model

Migration 24 adds the following table. All integer timestamps in
`analysis_documents` use Unix epoch seconds in UTC, matching `items.published_at`
and `analysis_run_messages.published_at`.

```sql
CREATE TABLE analysis_documents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  item_id INTEGER REFERENCES items(id) ON DELETE CASCADE,

  document_key TEXT NOT NULL,
  document_kind TEXT NOT NULL,

  source_type TEXT NOT NULL,
  source_subtype TEXT,
  external_id TEXT NOT NULL,

  author TEXT,
  published_at INTEGER NOT NULL,
  document_order INTEGER NOT NULL DEFAULT 0,

  ref TEXT NOT NULL,
  content_zstd BLOB NOT NULL,
  metadata_zstd BLOB,

  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,

  CHECK (document_kind IN (
    'telegram_message',
    'youtube_transcript',
    'youtube_comment',
    'youtube_description'
  )),
  CHECK (source_type IN ('telegram', 'youtube')),
  CHECK (
    (document_kind = 'telegram_message' AND source_type = 'telegram')
    OR
    (document_kind IN (
      'youtube_transcript',
      'youtube_comment',
      'youtube_description'
    ) AND source_type = 'youtube')
  ),
  CHECK (
    (source_type = 'telegram'
      AND COALESCE(source_subtype, '')
        IN ('channel', 'supergroup', 'group'))
    OR
    (source_type = 'youtube' AND COALESCE(source_subtype, '') = 'video')
  ),
  CHECK (
    (document_kind IN (
      'telegram_message',
      'youtube_transcript',
      'youtube_comment'
    ) AND item_id IS NOT NULL)
    OR
    (document_kind = 'youtube_description' AND item_id IS NULL)
  ),
  CHECK (
    (document_kind IN (
      'telegram_message',
      'youtube_transcript',
      'youtube_comment'
    ) AND document_key LIKE 'item:%')
    OR
    (document_kind = 'youtube_description'
      AND document_key = 'youtube:description')
  )
);

CREATE UNIQUE INDEX idx_analysis_documents_source_key
ON analysis_documents(source_id, document_key);

CREATE INDEX idx_analysis_documents_source_published
ON analysis_documents(source_id, published_at, document_order, id);

CREATE INDEX idx_analysis_documents_kind_source_published
ON analysis_documents(document_kind, source_id, published_at, document_order, id);

CREATE INDEX idx_analysis_documents_ref
ON analysis_documents(ref);
```

`source_id` owns source-level cleanup. `item_id ON DELETE CASCADE` is cleanup
convenience for item-backed documents only; synthetic documents have
`item_id = NULL` and are owned by source rebuild/upsert logic.

Document keys are deterministic:

- `item:<item_id>` for item-backed documents;
- `item:<item_id>:segment:<segment_index>` for YouTube transcript segment
  documents;
- `youtube:description` for the synthetic YouTube video description document.

The unique identity for rebuild/upsert is `(source_id, document_key)`.
`analysis_documents.id` is internal and must not become a public evidence ref.
The constant `youtube:description` key is intentionally source-scoped by that
unique index: many video sources can each have one description document with
the same per-source key.

`document_order` is the deterministic order key inside one
`(published_at, source_id)` bucket. It must not be derived by string-sorting
`ref`, because YouTube transcript timestamp suffixes such as `@10000ms` and
`@900ms` do not sort chronologically as text. Version 1 uses:

- `items.id` for Telegram message documents;
- `youtube_transcript_segments.segment_index` for YouTube transcript segment
  documents;
- `items.id` for YouTube comment documents;
- `-1` for YouTube description documents, preserving the current behavior
  where the synthetic description precedes transcript/comment rows for the
  same video publish timestamp.

SQLite CHECK constraints cannot validate cross-table consistency. Runtime,
backfill, and rebuild code must maintain these invariants:

- item-backed documents must have `analysis_documents.source_id =
  items.source_id`;
- item-backed documents must mirror the backing source's `source_type` and
  `source_subtype`;
- synthetic YouTube description documents must mirror
  `youtube_video_sources.source_id` and the matching `sources` row.

## Document Kinds

### `telegram_message`

Derived from `items` rows where:

```sql
items.item_kind = 'telegram_message'
AND items.content_zstd IS NOT NULL
AND items.content_kind IN ('text_only', 'text_with_media')
```

The document inherits:

- `source_id`;
- `item_id`;
- `source_type = 'telegram'`;
- `source_subtype`;
- `external_id = items.external_id`;
- `author`;
- `published_at = items.published_at`;
- `document_order = items.id`;
- `ref = s{source_id}-i{item_id}`;
- `content_zstd = items.content_zstd`.

No heavy Telegram metadata is required in `metadata_zstd` for v1. Reply, topic,
reaction, and media-aware source browsing data remains in the archive/provider
tables.

### `youtube_transcript`

Derived from `youtube_transcript_segments` joined to their
`youtube_transcript` item rows. This preserves the current segment-level
analysis corpus instead of collapsing a transcript into one document.

Eligible rows require:

```sql
items.item_kind = 'youtube_transcript'
AND youtube_transcript_segments.text IS NOT NULL
```

The current live corpus query does not apply an extra
`TRIM(youtube_transcript_segments.text) <> ''` filter. Version 1 should keep
that effective reader behavior for stored rows, while preserving the existing
runtime caption-parser behavior that skips blank cues before segment insertion.

Each transcript segment becomes one `youtube_transcript` document:

- `source_id = items.source_id`;
- `item_id = items.id`;
- `document_key = item:<item_id>:segment:<segment_index>`;
- `source_type = 'youtube'`;
- `source_subtype = 'video'`;
- `external_id = items.external_id`;
- `author = items.author`;
- `published_at = items.published_at`;
- `document_order = youtube_transcript_segments.segment_index`;
- `ref = s{source_id}-i{item_id}@{start_ms}ms`;
- `content_zstd` is the compressed segment text.

Existing timestamp evidence behavior must continue to resolve through
`youtube_transcript_segments`. Segment rows remain authoritative; the document
stores the analysis text unit and the metadata needed for trace resolution.

For YouTube transcript documents, `metadata_zstd` must preserve or reference
the data currently needed by trace/evidence resolution, such as selected
caption language, caption track kind, segment start/end timestamps, canonical
URL, title/channel snapshot, and segment evidence availability.

### `youtube_comment`

Derived from `items` rows where:

```sql
items.item_kind = 'youtube_comment'
AND items.content_zstd IS NOT NULL
AND items.content_kind IN ('text_only', 'text_with_media')
```

Comments are materialized independently of the selected run's
`youtube_corpus_mode`. The live corpus reader includes them only for
`transcript_description_comments`.

Comment documents inherit `external_id`, `author`, `published_at`, `ref`, and
`content_zstd` from the backing item in the same way as Telegram message
documents. `document_order = items.id`.

### `youtube_description`

Derived from YouTube video sources, not from `items`.

Create a `youtube_description` document for a YouTube video source when
`youtube_video_sources.description` is non-empty after trimming. Playlist
analysis continues to expand linked `video_source_id` rows, so description
documents belong to the linked video sources, not to playlist source rows.

The document uses:

- `source_id = youtube_video_sources.source_id`;
- `item_id = NULL`;
- `document_key = 'youtube:description'`;
- `source_type = 'youtube'`;
- `source_subtype = 'video'`;
- `external_id = description:<video_id>`;
- `published_at` is parsed from `youtube_video_sources.published_at`;
- `document_order = -1`;
- sources without a parseable video publish date are skipped in v1, matching
  the current synthetic description loader;
- `ref = s{source_id}-i0`;
- `content_zstd` produced from the typed description text.

The selected `youtube_corpus_mode` controls whether `load_corpus_messages`
includes these documents for a run. Backfill does not depend on a selected run
mode.

`metadata_zstd` should store a small analysis metadata envelope with values
needed by current trace and source context behavior, such as canonical video
URL, title/channel snapshot, and metadata envelope version. It must not store
raw provider payloads.

## Ref Semantics

Version 1 preserves current live ref semantics:

- item-backed documents use `s{source_id}-i{item_id}`;
- `analysis_documents.id` is not exposed as a public ref;
- YouTube transcript timestamp suffixes continue to resolve through existing
  timestamp evidence logic and `youtube_transcript_segments`;
- synthetic YouTube description preserves the current stable synthetic ref
  `s{source_id}-i0` and maps to `CorpusMessage.item_id = 0`;
- legacy saved Telegram refs such as `s{source_id}-m{message_id}` remain saved
  run compatibility behavior and are not reintroduced as live document refs.

`idx_analysis_documents_ref` exists for lookup support, but `ref` is not unique
in v1. Future document kinds may share base refs or generate suffix refs from
metadata. Any database lookup by `ref` must additionally constrain
`source_id`, `document_kind`, or another caller-owned scope when uniqueness
matters. The index is an access path, not an identity contract.

## Upgrade And Backfill

Migration 24 is runner-managed. The migration version is considered complete
only after schema creation and initial document backfill have run
successfully. The app must not reach a state where the schema exists, documents
are empty, and the analysis reader is switched to `analysis_documents`.

The runner-managed migration must be transactional where SQLite allows it, or
explicitly restart-safe and idempotent if interrupted before recording
migration 24.

The runner-managed upgrade should:

1. Create `analysis_documents` and its indexes.
2. Backfill Telegram message and YouTube comment documents from existing
   `items`.
3. Backfill YouTube transcript segment documents from
   `youtube_transcript_segments`.
4. Backfill synthetic YouTube description documents from
   `youtube_video_sources`.
5. Validate key constraints and source/document counts in representative
   migration tests.
6. Record migration 24 only after schema and backfill finish successfully.

Backfill is idempotent and rebuildable. It may delete/reinsert all documents
for a source or upsert deterministically by `(source_id, document_key)`.
Telegram message and YouTube comment documents may copy `items.content_zstd`
without decompression or recompression. YouTube transcript segment documents
compress segment text from `youtube_transcript_segments`. Synthetic YouTube
description documents use Rust helpers to compress derived text and encode the
metadata envelope.

## Rebuild And Freshness

Version 1 does not add an `analysis_document_state` table. Instead, the backend
must expose an internal helper:

```text
rebuild_analysis_documents_for_source(source_id)
```

The invariant is:

```text
After a source rebuild, analysis_documents rows for that source exactly equal
the text-bearing analysis units derivable from archive/provider state.
```

`rebuild_analysis_documents_for_source(source_id)` must run under the same
same-source ingest coordination used by sync and Takeout, or an equivalent
per-source write guard, so it cannot race an ingest task that is inserting or
updating documents for the same source. The rebuild itself should use one
SQLite transaction for delete/reinsert or deterministic upsert work for that
source.

Rows that are no longer derivable are removed:

- if an item-backed row loses `content_zstd`, the document is deleted;
- if an item-backed row changes text content, the matching document is updated;
- if YouTube transcript segments are replaced, transcript segment documents for
  that transcript item are rebuilt;
- if an item is deleted, the item-backed document is deleted by cascade or by
  rebuild;
- if a YouTube description becomes empty, `youtube:description` is deleted;
- if a YouTube description changes, content, metadata, and `updated_at` are
  updated.

Any code path that changes text-bearing content must update or delete the
matching document in the same SQLite transaction or explicitly guarded storage
boundary. Any code path that deletes a source or item relies on FK cascade
and/or source rebuild.

## Runtime Maintenance

The implementation must synchronously maintain documents for these write paths:

- Telegram normal sync insert;
- Telegram Takeout insert;
- YouTube transcript item and transcript segment writes;
- YouTube comment item write;
- YouTube video metadata upsert for description documents.

For item-backed writes, the item insert/update and matching document
upsert/delete must be part of one SQLite transaction wherever the current
writer already owns a transaction. Telegram message edits are not a normal
runtime update path in v1, but the shared Telegram insert helper must still
write the document in the same transaction as the item and `telegram_messages`
row. YouTube transcript refreshes may replace transcript item content and
segment rows; those refreshes must rebuild the transcript segment documents for
the item in the same SQLite transaction or guarded transaction boundary as the
replacement. YouTube comment refreshes must upsert or delete comment documents
alongside comment item writes. YouTube video metadata upsert must insert,
update, or delete the `youtube:description` document alongside the typed
metadata row when the description changes or becomes empty.

Normal source browsing and source item APIs remain on `items`, because `items`
still stores archive rows, media-bearing rows, and media-only rows that are not
part of text-first analysis.

## Reader Semantics

`load_corpus_messages` is the only production reader switched to
`analysis_documents` in v1.

Period filtering uses `analysis_documents.published_at`. Backfill/runtime must
ensure this matches the previous corpus filtering behavior:

- item-backed documents inherit `items.published_at`;
- YouTube transcript segment documents inherit their transcript
  `items.published_at`;
- YouTube description documents parse `youtube_video_sources.published_at` the
  same way the current synthetic description loader does;
- YouTube description documents are not written when that date is missing or
  unparseable in v1.

The reader returns documents ordered chronologically:

```sql
ORDER BY published_at ASC, source_id ASC, document_order ASC, id ASC
```

The reader must not use `ref` as a tie-breaker for ordering. `ref` is evidence
identity text, not a numeric order key.

The effective corpus must remain equivalent to the current reader:

- Telegram single-source corpus returns the same text-bearing Telegram
  messages for the same period.
- Telegram source-group corpus returns the same union across group members.
- YouTube `transcript_only` includes only `youtube_transcript` segment
  documents.
- YouTube `transcript_description` includes `youtube_transcript` and
  `youtube_description`.
- YouTube `transcript_description_comments` includes `youtube_transcript`,
  `youtube_description`, and `youtube_comment`.
- Playlist analysis still expands linked `video_source_id` rows and excludes
  unavailable or unlinked playlist entries.

For playlist scopes, source resolution remains the current two-step behavior:
the selected playlist source expands through `youtube_playlist_items` to linked
non-removed `video_source_id` rows, then `load_corpus_messages` reads
`analysis_documents` for those video source ids. `analysis_documents` rows are
not stored under the playlist source id in v1.

Saved run snapshot flow remains:

```text
analysis_documents -> live corpus load -> provider call -> analysis_run_messages snapshot
```

`analysis_run_messages` stores the exact corpus loaded from
`analysis_documents` for the run. Later document rebuilds must not affect saved
run follow-up, saved trace resolution, or report artifacts.

## Metadata Policy

`analysis_documents.metadata_zstd` is a small analysis metadata envelope. It is
not a raw provider archive and must not store raw Telegram TL payloads, raw
`yt-dlp` payloads, cookies, auth headers, session data, or large compressed
provider dumps.

Version 1 metadata should include only values needed by current analysis trace
and evidence behavior:

- YouTube transcript evidence metadata and segment lookup hints;
- YouTube description source context and canonical URL metadata;
- an explicit metadata envelope version field, starting at `1`, for every
  non-empty v1 metadata envelope.

Telegram message documents do not need heavy metadata in v1 unless the existing
analysis corpus path requires a value that is not already represented by the
document row.

## Testing Requirements

Migration tests must cover:

- migration 24 registration and runner-managed behavior;
- fresh schema includes table, constraints, and indexes;
- item-backed vs synthetic CHECK constraints reject mixed semantics;
- `document_order` exists, participates in the published-order indexes, and is
  populated deterministically during backfill;
- migration 24 backfills Telegram text messages;
- migration 24 backfills YouTube transcript segment and comment documents;
- migration 24 creates YouTube description documents for non-empty video
  descriptions;
- item-backed backfill verifies `analysis_documents.source_id` equals
  `items.source_id` and that document source type/subtype mirror `sources`;
- media-only rows do not become documents;
- re-running source rebuild is idempotent;
- runner-managed migration 24 is restart-safe or idempotent if interrupted
  before recording the migration version.

Reader tests must cover:

- Telegram single-source corpus equivalence;
- Telegram source-group corpus equivalence;
- YouTube corpus mode inclusion/exclusion;
- playlist linked-video expansion and unavailable/unlinked exclusion;
- chronological ordering by
  `published_at ASC, source_id ASC, document_order ASC, id ASC`;
- YouTube transcript segments with start timestamps such as `900ms` and
  `10000ms` retain segment order and are not ordered by lexicographic `ref`;
- period filtering by `analysis_documents.published_at`;
- ref lookup tests must not assume `ref` is globally unique;
- YouTube transcript timestamp evidence still resolves through
  `youtube_transcript_segments`;
- saved run snapshot persistence still writes `analysis_run_messages`.

Runtime tests must cover document maintenance for:

- Telegram normal sync insertion or the shared Telegram item insert helper;
- Telegram Takeout insertion through the same observation-aware path;
- YouTube transcript item and segment writes;
- YouTube comment item writes;
- YouTube video metadata upsert when description is inserted, changed, or
  cleared;
- source rebuild does not race same-source ingest and is idempotent;
- item-backed runtime writes verify document `source_id`, `source_type`, and
  `source_subtype` consistency with backing `items` and `sources` rows.

Containment scans must verify:

- source browsing and `get_items` still read `items`;
- NotebookLM export still reads its current query path;
- no production reader besides `load_corpus_messages` is switched in v1.

## Documentation Updates

Update `docs/database-schema.md`:

- document `analysis_documents` as a materialized read model;
- state that provider/archive truth remains in `items` and typed provider
  tables;
- document document kinds, refs, indexes, and rebuild semantics;
- state that source browsing and NotebookLM export remain outside v1.

Update `docs/archive/database-schema-legacy-analysis.md` and `docs/backlog.md`:

- mark provider-neutral analysis document layer v1 as shipped after
  implementation;
- keep NotebookLM export switching, source browsing changes, and current-schema
  baseline as follow-up work.

## Risks And Mitigations

- Stale documents: mitigated by same-boundary writer updates, source rebuild
  helper, idempotent backfill, and tests for content deletion/update.
- Ref regressions: mitigated by preserving item-backed refs and keeping
  document ids internal.
- YouTube period drift: mitigated by explicit `published_at` inheritance and
  by matching the current description loader behavior: skip descriptions whose
  typed video publish date is missing or unparseable.
- Snapshot regressions: mitigated by keeping `analysis_run_messages` as the
  frozen corpus snapshot.
- Scope creep: mitigated by leaving source browsing and NotebookLM export on
  existing paths in v1.
