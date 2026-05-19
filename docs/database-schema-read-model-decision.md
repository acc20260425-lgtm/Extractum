# Provider-neutral archive read model decision

## Context

`analysis_documents` is the provider-neutral materialized read model for live
analysis corpus loading. It is intentionally text-first: it stores the units
that can enter report prompts, trace evidence, and saved analysis snapshots.
Provider/archive truth remains in `items` plus typed provider tables such as
`telegram_messages`, `youtube_video_sources`, `youtube_playlist_sources`,
`youtube_transcript_segments`, `item_topic_memberships`, and
`telegram_forum_topics`.

Database schema simplification had one open read-model question: whether
NotebookLM export and source browsing should reuse/extend `analysis_documents`,
or whether they need a neighboring provider-neutral archive/read UI model. The
decision slice chose the neighboring archive/read UI model. The first
implementation slice has now shipped source browsing as the first gated
consumer; NotebookLM export migration remains pending.

The decision also recognizes the cost of a new materialized boundary. Any
follow-up implementation must define update semantics before adding schema:
when rows are built, how existing databases are backfilled, what stays
canonical, how builder failures are handled, and how stale rows are detected or
repaired.

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

Current required fidelity covers Telegram NotebookLM export and source
browsing. YouTube NotebookLM export rows below are future-facing constraints:
they must inform the archive-model boundary, but must not force schema fields
unless the next implementation slice explicitly includes YouTube export
enrichment.

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
| Source browsing | Transcript segment navigation, search, and selected-time paging | `youtube_transcript_segments` | Yes for analysis evidence documents | Browse UI needs reader-specific segment paging/search semantics, not just prompt/evidence refs | Archive read model with typed segment support or a paired segment reader |
| Source browsing | YouTube comments | `items.item_kind = 'youtube_comment'` plus comment raw/detail fields | Yes for text corpus | Browse display still needs archive item semantics and comment metadata | Archive read model |
| Source browsing | Video description display context | `youtube_video_sources` and `analysis_documents` synthetic description | Partial | Description text is useful for analysis, but browse/export semantics need source-level display context and should not force description into item paging | Archive read model for display summary; typed source detail remains canonical |
| Source browsing | Playlist membership | `youtube_playlist_items`, typed YouTube source tables | No | Playlist position, linked video, availability, and removal state are archive UI context, not LLM corpus text | Archive read model |
| Source browsing | Playlist removal state | `youtube_playlist_items.availability_status`, `youtube_playlist_items.is_removed_from_playlist` | No | Existing schema has two representations; read model should expose one derived display state while canonical cleanup remains a separate YouTube simplification slice | Archive read model derived field |
| Source browsing | Linked/unlinked playlist entries | `youtube_playlist_items.video_source_id`, `video_id`, `availability_status` | No | Browse UI needs unavailable/unlinked entries without pretending they are corpus documents | Archive read model |
| Future NotebookLM export | Transcript timestamps and canonical links | `youtube_transcript_segments`, `youtube_video_sources` | Partial | Export formatting needs explicit source/display semantics, not just corpus refs | Archive read model |
| Future NotebookLM export | Comment metadata and reactions/likes | `items`, YouTube comment raw/detail fields | Partial | Needs export-specific rendering contract | Archive read model |
| Future NotebookLM export | Playlist context | `youtube_playlist_items`, typed YouTube source tables | No | Export enrichment must preserve playlist position and availability without expanding `analysis_documents` | Archive read model |

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

The implementation spec must choose explicit maintenance semantics. The
preferred default is synchronous builder maintenance in the same transaction as
normal `items` and typed-provider writes, plus scoped rebuild/backfill helpers
for migrations, repairs, and debug fixtures. SQLite triggers should be avoided
unless Rust-side maintenance proves insufficient, because the builder contract
needs ordinary Rust tests and provider-specific validation. Large existing
databases may require chunked or resumable backfill before any consumer switches
to the new model.

For normal runtime writes, archive builder failure rolls back the whole
transaction. Committing an `items` row while silently skipping the derived
archive row would make the consumer boundary stale at the moment it is created.
The builder must therefore be deterministic, local-only, and narrowly tested so
rollback-on-failure is reserved for real storage/validation defects rather than
ordinary provider variance. For migration backfill, repair, or manual rebuild,
failure must not delete canonical provider/archive data; it should mark the
source/model scope stale or failed and keep consumers gated until a successful
rebuild.

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

## Implementation Status

Implemented first slice:

- `analysis_documents` remains the provider-neutral LLM corpus read model.
- `archive_read_model_state` is the source-scoped readiness gate.
- `archive_read_items` stores item-level provider-neutral archive rows.
- Current builder contract is `model_version = 1`.
- Source browsing is the first consumer and is gated: it reads archive rows only
  when source state is `ready` and current, otherwise it falls back to the
  existing `items` plus topic joins path.
- Normal single Telegram item writes maintain ready archive rows in the same
  transaction.
- Bulk ingest/Takeout and YouTube refresh paths mark the source archive model
  stale instead of rebuilding every row inline.
- YouTube transcript segment navigation remains on the paired typed segment
  reader for this slice; transcript segments are not `archive_read_items` rows.
- NotebookLM export still reads the provider/archive path and requires a later
  export parity slice before migration.

## Parity Test Inventory

Future migrations must keep these behaviors green before switching consumers
to any new read model.

The first migration tests should compare old and new query paths on the same
seeded data. For source browsing, compare normalized item DTOs and paging/filter
results. For NotebookLM export, compare the loaded export message model and
focused rendered blocks; broad markdown snapshots should be used sparingly
because renderer-only wording changes should not invalidate storage parity.
Behavioral invariant tests remain useful, but the initial gate should be
old-path versus new-path output comparison.

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

The source browsing implementation should not choose a schema that is known to
exclude required Telegram NotebookLM export fields. It may leave export-specific
rendering migration for a later slice, but the data-model boundary must already
account for reply, topic, reaction, and media fidelity. This is intentionally
softer than requiring browsing to complete the export migration: before
NotebookLM export switches, those fields must be populated and verified by
export parity tests.

## Preliminary Constraints For The Follow-up Implementation Spec

These are constraints for the next task, not runtime work in this decision
slice. The next implementation spec must settle these rules before creating a
table:

- Canonical truth remains in `items` plus typed provider tables. The archive
  read model is rebuildable derived state, not the owner of provider data.
- Normal item/provider writes should update archive read rows synchronously in
  the same transaction that writes `items`, `telegram_messages`,
  `youtube_transcript_segments`, or YouTube typed source/playlist state.
- For a single item/provider write, if synchronous builder maintenance fails,
  the whole write transaction rolls back. The app must not commit canonical
  rows while leaving their archive read rows missing or stale.
- Bulk ingest paths such as normal source sync and Telegram Takeout import
  should not make every item insertion depend on per-item archive builder
  success. They should commit canonical item/provenance rows according to the
  ingest transaction policy, mark the source archive model `stale` or
  `building`, and rebuild/switch readiness at source scope after the batch.
- For bulk rebuild/backfill, failure marks the source/model scope stale or
  failed and does not mutate canonical provider/archive data.
- For large provider metadata refreshes that touch many derived rows, the next
  spec must choose either one transaction with rollback or staged rebuild plus
  readiness switch before consumers observe the new rows.
- A scoped rebuild helper must exist for one source and for all sources. It is
  used by migrations, repair paths, fixtures, and manual recovery.
- The implementation spec should define archive read-model readiness metadata,
  likely source-scoped:
  - `source_id`
  - `model_version`
  - `status`: `never_built`, `building`, `ready`, `stale`, or `failed`
  - `built_at`
  - `item_count` / `row_count`
  - `last_error`
- `model_version` is a monotonic builder contract version, not only a schema
  migration number. Any change that affects row shape, derived metadata,
  filtering semantics, or backfill correctness must bump the current archive
  model version. Consumers may use archive rows only when readiness is `ready`
  and `model_version` matches the current builder version; older readiness
  records are stale and require rebuild before migration or use.
- Existing database backfill should prefer a gated lazy per-source rebuild:
  migration creates the schema and readiness metadata, consumers remain on old
  paths until a source has a successful archive-model rebuild, and the first
  source access can trigger or request that scoped rebuild. Full eager startup
  backfill should be reserved for small/fixture databases or explicit repair
  commands.
- If the backfill can be large, the implementation plan must include chunking
  or progress state before any consumer switches by default.
- YouTube transcript granularity must be chosen before schema work. The next
  spec should either keep one archive item row per transcript with a paired
  typed segment reader, or materialize segment rows in the archive model; it
  must not leave segment ownership ambiguous.
- Staleness handling must be explicit. Either consumers only switch after a
  successful backfill/rebuild marker, or the read path must detect and reject
  stale/missing derived rows instead of silently falling back to provider joins.
- Provider-specific branching belongs in the builder/backfill layer. Consumer
  query paths should read provider-neutral rows plus typed nested metadata.
  A paired YouTube transcript segment reader is the allowed exception if the
  next spec chooses item-level archive rows plus typed segment navigation; in
  that case, segment reads remain a typed provider-neutral reader owned beside
  the archive model, not ad hoc joins in UI/export code.

## Consequences

- `analysis_documents` stays small enough to explain as the live analysis
  corpus model.
- Archive read-model work can optimize for source browsing and NotebookLM
  export without forcing LLM corpus semantics onto archive UI state.
- Provider-specific joins do not disappear; they move into a builder/backfill
  layer with focused tests.
- Single-write builder failures are write failures, not warnings. Bulk ingest,
  rebuild, and backfill failures become source/model readiness failures and do
  not mutate canonical provider data.
- Existing databases should avoid blocking startup on a full archive-model
  backfill; lazy per-source rebuild plus consumer gating is the preferred
  rollout shape.
- The first implementation consumer is source browsing, because its item-list
  parity is easier to validate than full NotebookLM export output.
- NotebookLM export should migrate only after export-specific parity tests prove
  the archive/read UI model preserves reply/topic/reaction/media fidelity.
- "Proves fidelity" means the model does not exclude the required Telegram
  NotebookLM export fields listed above, even if export rendering remains on
  the old path until a later slice.
- Current-schema baseline work should wait until the archive read-model
  boundary is stable.
- Legacy Telegram blob cleanup remains blocked on real Telegram
  runtime/private-source validation.

## Dependency Map

| Work area | Relationship to archive read model |
| --- | --- |
| Source browsing migration | First consumer. It can validate paging, filtering, media visibility, and provider item semantics. |
| NotebookLM export migration | Blocked until the archive model includes export-required reply/topic/reaction/media fields and browsing parity has passed. |
| Takeout provenance | Can proceed in parallel while it writes provenance tables only. If it adds new archive-display fields or origin semantics that consumers need, the archive builder contract and readiness/backfill rules must be updated before consumer migration. |
| YouTube playlist simplification | Related but separate. The archive read model may expose one derived playlist display state, but canonical cleanup of `availability_status` versus `is_removed_from_playlist` belongs to a later YouTube slice. |
| Current-schema baseline | Blocked until the archive read-model boundary and migration/backfill rules are stable. |
| Legacy Telegram metadata blob cleanup | Blocked on typed repair plus real private/dialog-backed source validation, not on the archive read model. |

## Follow-up Implementation Slices

1. [x] Define update semantics, builder-failure rollback rules, lazy
   per-source backfill/gating, YouTube transcript segment ownership,
   stale-row handling, and the full export-ready field set before adding
   schema.
2. [x] Build the archive/read UI model for source browsing first.
3. [x] Migrate source browsing behind old-path versus new-path parity tests.
4. [ ] Migrate NotebookLM export after export parity tests pass.
5. [ ] Decide whether future YouTube playlist-entry browsing needs archive rows
   or typed detail only.
6. [ ] Consider a current-schema baseline after the read-model boundary
   settles.
7. [ ] Consider legacy Telegram metadata blob cleanup only after typed repair
   and real private/dialog-backed source validation are proven safe.

## Resolved Implementation Questions

- Table names: `archive_read_model_state` and `archive_read_items`.
- First row granularity: item-level archive rows.
- Readiness gate: source-scoped state with current `model_version`.
- Telegram export-required day-one data is represented in the archive row
  boundary, but export migration remains pending.
- YouTube export fields remain future-facing and are not required by the first
  source-browsing slice.
- Media metadata is copied as compressed display metadata; raw payload bytes are
  not copied and are represented by `has_raw_data`.
- Builder contract changes that affect row shape, derived metadata, filtering
  semantics, or backfill correctness require a `model_version` bump.

## Historical Non-goals For The Decision Slice

- no NotebookLM export migration;
- no source browsing migration;
- no runtime behavior change;
- no schema migration for a new table;
- no cleanup of old Telegram `sources.metadata_zstd` blobs;
- no current-schema baseline.
