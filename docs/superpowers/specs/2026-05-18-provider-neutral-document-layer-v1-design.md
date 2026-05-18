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

Migration 24 adds:

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
ON analysis_documents(source_id, published_at, id);

CREATE INDEX idx_analysis_documents_kind_source_published
ON analysis_documents(document_kind, source_id, published_at, id);

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

Each transcript segment becomes one `youtube_transcript` document:

- `source_id = items.source_id`;
- `item_id = items.id`;
- `document_key = item:<item_id>:segment:<segment_index>`;
- `source_type = 'youtube'`;
- `source_subtype = 'video'`;
- `external_id = items.external_id`;
- `author = items.author`;
- `published_at = items.published_at`;
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
documents.

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
metadata.

## Upgrade And Backfill

Migration 24 is runner-managed. The migration version is considered complete
only after schema creation and initial document backfill have run
successfully. The app must not reach a state where the schema exists, documents
are empty, and the analysis reader is switched to `analysis_documents`.

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
matching document in the same storage boundary. Any code path that deletes a
source or item relies on FK cascade and/or source rebuild.

## Runtime Maintenance

The implementation must synchronously maintain documents for these write paths:

- Telegram normal sync insert;
- Telegram Takeout insert;
- YouTube transcript item and transcript segment writes;
- YouTube comment item write;
- YouTube video metadata upsert for description documents.

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
ORDER BY published_at ASC, source_id ASC, ref ASC
```

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
- optional envelope version fields for future migration.

Telegram message documents do not need heavy metadata in v1 unless the existing
analysis corpus path requires a value that is not already represented by the
document row.

## Testing Requirements

Migration tests must cover:

- migration 24 registration and runner-managed behavior;
- fresh schema includes table, constraints, and indexes;
- item-backed vs synthetic CHECK constraints reject mixed semantics;
- migration 24 backfills Telegram text messages;
- migration 24 backfills YouTube transcript segment and comment documents;
- migration 24 creates YouTube description documents for non-empty video
  descriptions;
- media-only rows do not become documents;
- re-running source rebuild is idempotent.

Reader tests must cover:

- Telegram single-source corpus equivalence;
- Telegram source-group corpus equivalence;
- YouTube corpus mode inclusion/exclusion;
- playlist linked-video expansion and unavailable/unlinked exclusion;
- chronological ordering by `published_at ASC, source_id ASC, ref ASC`;
- period filtering by `analysis_documents.published_at`;
- YouTube transcript timestamp evidence still resolves through
  `youtube_transcript_segments`;
- saved run snapshot persistence still writes `analysis_run_messages`.

Runtime tests must cover document maintenance for:

- Telegram normal sync insertion or the shared Telegram item insert helper;
- Telegram Takeout insertion through the same observation-aware path;
- YouTube transcript item and segment writes;
- YouTube comment item writes;
- YouTube video metadata upsert when description is inserted, changed, or
  cleared.

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

Update `docs/database-schema-legacy-analysis.md` and `docs/backlog.md`:

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
  deterministic description fallback.
- Snapshot regressions: mitigated by keeping `analysis_run_messages` as the
  frozen corpus snapshot.
- Scope creep: mitigated by leaving source browsing and NotebookLM export on
  existing paths in v1.
