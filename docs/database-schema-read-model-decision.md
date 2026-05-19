# Provider-neutral archive read model decision

## Context

`analysis_documents` is the provider-neutral materialized read model for live
analysis corpus loading. It is intentionally text-first: it stores the units
that can enter report prompts, trace evidence, and saved analysis snapshots.
Provider/archive truth remains in `items` plus typed provider tables such as
`telegram_messages`, `youtube_video_sources`, `youtube_playlist_sources`,
`youtube_transcript_segments`, `item_topic_memberships`, and
`telegram_forum_topics`.

Database schema simplification still has one open read-model question:
whether NotebookLM export and source browsing should reuse/extend
`analysis_documents`, or whether they need a neighboring provider-neutral
archive/read UI model. This slice decides the boundary only. It does not move
NotebookLM export, source browsing, or any runtime query path.

## Consumers

### NotebookLM export

NotebookLM export is a local archive export for Telegram sources. It renders
conversation documents with message text, media placeholders, reply context,
topic context, reaction counts, participants, and period chunking. It must stay
local-only: export query/rendering must not introduce live Telegram calls, LLM
calls, network link enrichment, or media downloads.

### Source browsing

Source browsing is the live archive reader used by the analysis workspace and
source reader surfaces. It lists source items chronologically, supports topic
filters for Telegram forum sources, keeps media-only and text-with-media rows
visible, distinguishes provider item kinds, and exposes stable local item refs.

## Current Data Sources

NotebookLM export currently reads:

- `sources` for source identity and display title;
- `items` for message text, item identity, author/date, content kind, media
  metadata, reply metadata, and reaction count;
- `items` again for reply snippet lookups outside the export period;
- `item_topic_memberships` and `telegram_forum_topics` for materialized topic
  context.

Source browsing currently reads:

- `items` for chronological archive rows, provider item kind, text/media
  content kind, raw-data presence, media metadata, reply metadata, and reaction
  count;
- `item_topic_memberships`, `telegram_topic_resolution_state`, and
  `telegram_forum_topics` for topic filters and topic display context;
- YouTube transcript segment APIs for timestamped transcript browsing where
  the UI needs segment-level navigation.

`analysis_documents` currently reads/builds from:

- text-bearing Telegram messages and YouTube comments;
- YouTube transcript segments;
- synthetic YouTube video descriptions from typed YouTube source metadata.

It intentionally excludes media-only rows and does not own full archive display
metadata for replies, reactions, topic filters, raw-data presence, or browse UI
state.

## Fidelity Matrix

### Telegram

| Consumer | Required field / semantic | Current source | `analysis_documents` coverage | Gap | Recommended owner |
| --- | --- | --- | --- | --- | --- |
| NotebookLM export | Message text | `items.content_zstd` | Yes for text-bearing Telegram messages | None for text-bearing rows | Either model can read text |
| NotebookLM export | Media-only rows and media placeholders | `items.content_kind`, `items.has_media`, `items.media_kind`, `items.media_metadata_zstd` | No | Analysis corpus is text-first and skips media-only rows | Archive read model |
| NotebookLM export | Reply snippet outside export period | `items.reply_to_msg_id` plus reply lookup in `items` | No | Needs archive context outside the selected export page/window | Archive read model |
| NotebookLM export | Reply peer/thread metadata | `items.reply_to_peer_kind`, `items.reply_to_peer_id`, `items.reply_to_top_id` | No | Needed in rendered export metadata | Archive read model |
| NotebookLM export | Reaction count | `items.reaction_count` | No | Needed in rendered export metadata | Archive read model |
| NotebookLM export | Forum topic metadata | `item_topic_memberships`, `telegram_forum_topics` | No | Topic fidelity depends on materialized archive membership state | Archive read model |
| NotebookLM export | Local-only behavior | Current local SQLite query/render path | Compatible but not sufficient | Future model must not add live calls | Archive read model |
| Source browsing | Chronological item list | `items.published_at`, `items.id` | Partial, ascending corpus order only | Browsing uses archive paging and UI ordering semantics | Archive read model |
| Source browsing | Topic filters and uncategorized behavior | `item_topic_memberships`, `telegram_topic_resolution_state` | No | Filter semantics depend on topic resolver readiness | Archive read model |
| Source browsing | Media-only and text-with-media visibility | `items.content_kind`, media fields | No | Browsing must show archive rows that are not LLM corpus text | Archive read model |
| Source browsing | Stable local item refs | `items.source_id`, `items.id` | Yes for text documents | Missing media-only/archive-only rows | Archive read model |

### YouTube

| Consumer | Required field / semantic | Current source | `analysis_documents` coverage | Gap | Recommended owner |
| --- | --- | --- | --- | --- | --- |
| Source browsing | Transcript item vs comment distinction | `items.item_kind` and source reader model | Partial | `analysis_documents` splits transcript into segment documents, while browsing also needs item-level distinction | Archive read model |
| Source browsing | Timestamped transcript segment navigation | `youtube_transcript_segments` | Yes for analysis evidence documents | Browse UI needs reader-specific paging/search context | Archive read model with typed segment support |
| Source browsing | YouTube comments | `items.item_kind = 'youtube_comment'` | Yes for text corpus | Browse display still needs archive item semantics | Archive read model |
| Source browsing | Video description display context | `youtube_video_sources` and `analysis_documents` synthetic description | Partial | Description is useful for analysis, but browse/export semantics may need source-level display context | Archive read model or typed source detail |
| Source browsing | Playlist context | `youtube_playlist_items`, typed YouTube source tables | No | Playlist availability/removal/link state is archive UI context, not LLM corpus text | Archive read model |
| Future NotebookLM export | Transcript timestamps and canonical links | `youtube_transcript_segments`, `youtube_video_sources` | Partial | Export formatting needs explicit source/display semantics, not just corpus refs | Archive read model |
| Future NotebookLM export | Comment metadata and reactions/likes | `items`, YouTube comment raw/detail fields | Partial | Needs export-specific rendering contract | Archive read model |

## Options Considered

### Option A: Reuse and extend `analysis_documents`

This would minimize the number of materialized tables and reuse the already
shipped provider-neutral document builder. It is attractive for shared text
loading, YouTube transcript segment refs, and synthetic YouTube descriptions.

The downside is boundary drift. NotebookLM export and source browsing need
archive fidelity: media-only rows, reply snippets, topic filters, reactions,
raw/browse display state, playlist context, and provider item semantics.
Adding those concerns to `analysis_documents` would blur its current role as
the LLM corpus read model and risk turning it into a generic everything table.

### Option B: Create a separate provider-neutral archive/read UI model

This keeps `analysis_documents` focused on LLM corpus units and creates a
neighboring read model for archive consumers. Provider-specific complexity
would live in a builder/backfill layer that produces stable provider-neutral
rows for browsing and export. UI/export query paths would consume the read
model rather than reimplementing provider joins.

The downside is another materialized boundary to build, backfill, and verify.
That cost is justified only if parity tests prove the model preserves archive
fidelity.

### Option C: Add parity tests first and defer the model

This is the safest first step: lock down current behavior and decide the
boundary before changing runtime. It does not itself reduce schema/query
complexity, but it gives the next implementation slice a clear safety net.

## Decision

Decision: create a separate provider-neutral archive/read UI model, after
locking current behavior with parity tests.

`analysis_documents` remains the provider-neutral LLM corpus read model.
NotebookLM export and source browsing need archive-fidelity data with media,
topic, reply, reaction, raw/browse display, playlist, and provider item
semantics. Extending `analysis_documents` to cover those semantics would blur
the boundary and make the table harder to reason about.

The immediate slice is C -> B:

1. document the decision and fidelity matrix;
2. inventory parity tests that gate any future migration;
3. make no runtime behavior change in this slice;
4. implement the selected archive/read UI model in follow-up slices.

## Parity Test Inventory

Future migrations must keep these behaviors green before switching consumers
to any new read model.

NotebookLM export parity:

- preserves reply snippets, including reply targets outside the export period;
- preserves `reply_to_peer_kind`, `reply_to_peer_id`, and `reply_to_top_id`;
- preserves `reaction_count`;
- preserves forum topic id, title, and top-message metadata through
  materialized memberships;
- preserves media placeholders and media metadata rendering;
- keeps export local-only, with no live Telegram, LLM, network link
  enrichment, or media download calls;
- keeps period filtering, chunking, participant summaries, and deterministic
  output ordering stable.

Source browsing parity:

- chronological paging and around-item loading do not change;
- topic filters and uncategorized behavior do not change, including the
  resolver-readiness gate;
- media-only and text-with-media rows remain visible;
- `content_kind`, `item_kind`, and media summary semantics remain stable;
- Telegram reply/thread/reaction/topic fields remain stable;
- YouTube transcript and comment rows remain distinguishable;
- existing local refs remain stable where currently expected;
- YouTube transcript segment navigation remains backed by timestamped segment
  data.

## Consequences

- `analysis_documents` stays small enough to explain as the live analysis
  corpus model.
- Archive read-model work can optimize for source browsing and NotebookLM
  export without forcing LLM corpus semantics onto archive UI state.
- Provider-specific joins do not disappear; they move into a builder/backfill
  layer with focused tests.
- The first implementation consumer should be source browsing, because its
  item-list parity is easier to validate than full NotebookLM export output.
- NotebookLM export should migrate only after source browsing proves the
  archive/read UI model preserves fidelity.
- Current-schema baseline work should wait until the archive read-model
  boundary is stable.
- Legacy Telegram blob cleanup remains blocked on real Telegram
  runtime/private-source validation.

## Follow-up Implementation Slices

1. Build the archive/read UI model for source browsing first.
2. Migrate source browsing behind parity tests.
3. Migrate NotebookLM export after browsing proves the model.
4. Consider a current-schema baseline after the read-model boundary settles.
5. Consider legacy Telegram metadata blob cleanup only after typed repair and
   real private/dialog-backed source validation are proven safe.

## Non-goals For This Slice

- no NotebookLM export migration;
- no source browsing migration;
- no runtime behavior change;
- no schema migration for a new table;
- no cleanup of old Telegram `sources.metadata_zstd` blobs;
- no current-schema baseline.
