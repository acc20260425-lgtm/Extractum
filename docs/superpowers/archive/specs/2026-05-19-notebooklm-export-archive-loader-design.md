# NotebookLM Export Archive Loader Design

> Date: 2026-05-19
> Status: approved direction, pending implementation plan
> Scope: Database Schema Simplification slice, Telegram NotebookLM export parity

## Summary

This slice makes Telegram NotebookLM export the second readiness-gated consumer
of `archive_read_items`.

The export command selects its message loader once per source/export call. If
`archive_read_model_state` is `ready` for the current archive read-model
version, export uses an `archive_read_items` backed loader. Otherwise it
preserves the existing local provider/archive loader over `items`,
`item_topic_memberships`, and `telegram_forum_topics`.

After the archive loader is selected, it is authoritative for that call. Decode,
corrupt-row, or archive invariant failures are returned as typed errors and
must not silently fall back to the old items path.

This is intentionally narrow. It does not include YouTube playlist canonical
cleanup, current-schema baseline work, YouTube NotebookLM export enrichment,
source-group export, optional link enrichment, or renderer redesign.

## Current State

NotebookLM export is currently Telegram-only and local-only. It reads:

- `sources` for source identity and display title;
- `items` for message text, item identity, author/date, content kind, media
  metadata, reply metadata, and reaction count;
- `items` again for reply snippet lookup outside the selected export period;
- `item_topic_memberships` and `telegram_forum_topics` for materialized forum
  topic context.

The archive read model now materializes source-scoped item rows into
`archive_read_items` and gates consumers through `archive_read_model_state`.
Source browsing already uses this model only when the source state is ready for
the current `ARCHIVE_READ_MODEL_VERSION`; all other states preserve the old
items path.

NotebookLM export still reads the old path. This slice migrates the export
message loader behind the same readiness concept, with parity tests proving the
archive path preserves current Telegram export fidelity.

Before any archive/items loader selection, NotebookLM export must first validate
that the requested source exists and is a Telegram source. Non-Telegram sources
keep the existing validation error and do not enter archive/items loader
selection. This prevents a future YouTube source with ready archive rows from
accidentally entering the Telegram export loader.

## Goals

1. Add old-path versus archive-path parity tests for Telegram NotebookLM export
   messages.
2. Add an explicit loader selection enum for NotebookLM export observability and
   testability.
3. Add an `archive_read_items` backed NotebookLM export message loader.
4. Preserve old export behavior for missing, building, stale, failed, and old
   model-version archive states.
5. Treat archive loader failures as archive-model defects after the gate has
   selected the archive path.
6. Keep reply snippet lookup inside the selected archive read model path.
7. Update docs/backlog to mark this slice shipped and leave later schema
   simplification work explicit.

## Non-Goals

- Do not clean up YouTube playlist canonical state such as
  `availability_status` versus `is_removed_from_playlist`.
- Do not create a current-schema baseline.
- Do not add YouTube NotebookLM export enrichment for transcript timestamps,
  canonical video links, comments, likes, or playlist membership metadata.
- Do not add source-group export.
- Do not add optional link enrichment, network calls, media downloads, LLM
  calls, or live Telegram calls.
- Do not change NotebookLM renderer wording except where tests need focused
  parity coverage.
- Do not expand `analysis_documents` into an archive/export model.

## Loader Selection

NotebookLM export should not call a bare readiness boolean directly. It should
use a source-scoped selection helper:

```rust
pub(crate) enum ExportLoaderSelection {
    ArchiveReadModel { model_version: i64 },
    ItemsPath {
        reason: ArchiveReadinessFallbackReason,
    },
}

pub(crate) enum ArchiveReadinessFallbackReason {
    MissingState,
    NeverBuilt,
    Building,
    Stale,
    Failed,
    OldModelVersion { found: i64, current: i64 },
}
```

The exact Rust visibility can remain `pub(crate)`. The important contract is
that tests can assert why a non-archive path was selected, instead of treating
every fallback as `false`.

The gate is evaluated once per source/export call:

```text
ready/current -> ArchiveReadModel { model_version }
anything else -> ItemsPath { reason }
```

`load_export_messages` remains the public query entry point for the export
command and becomes a wrapper:

```text
selection = select_notebooklm_export_loader(pool, source_id)

ArchiveReadModel -> load_export_messages_from_archive(...)
ItemsPath        -> load_export_messages_from_items_path(...)
```

There is no catch-all fallback after `load_export_messages_from_archive` starts.
This selection happens only after source validation has proved the source is a
Telegram source.

## Archive Loader Semantics

The archive query is Telegram-export scoped. NotebookLM export validates the
source as Telegram before loader selection and the archive loader reads only
archive rows where `item_kind = 'telegram_message'`.

The archive loader returns the same `NotebookLmExportMessage` shape as the
current items-path loader:

- `item_id = archive_read_items.item_id`;
- `source_id`;
- `external_id`;
- `author`;
- `published_at`;
- decompressed `content_zstd` text;
- `content_kind`;
- `has_media`;
- `media_kind`;
- decoded `media_metadata_zstd`;
- rendered media placeholders;
- detected URLs from message text;
- reply metadata;
- reaction count;
- forum topic id/title/top message id.

Ordering and filtering must match current export behavior:

```sql
WHERE source_id = ?
  AND model_version = ARCHIVE_READ_MODEL_VERSION
  AND item_kind = 'telegram_message'
  AND optional period filters
ORDER BY published_at ASC, item_id ASC
```

Period filtering uses the same inclusive semantics as the current items path:

```text
period_from <= published_at <= period_to
```

when both bounds are provided, with one-sided filters preserving the current
behavior.

Current `items.published_at` and `archive_read_items.published_at` are both
schema-required. The implementation plan should make that invariant explicit.
If a legacy/manual fixture can construct rows with `NULL published_at`, parity
tests must preserve the current items-path behavior for unbounded and bounded
exports; otherwise the test should assert the schema/invariant rejects such
rows before either loader sees them.

After `ArchiveReadModel` is selected, missing required archive fields are
archive invariant failures, not reasons to join back to canonical `items`.
The archive loader must not join back to `items`, `item_topic_memberships`, or
`telegram_forum_topics` to fill missing archive fields after it has been
selected. This applies especially to media metadata, reply fields, and topic
fields.

## Reply Snippet Contract

If the archive loader is selected, reply snippets must also come from
`archive_read_items`. The export call must not be half-migrated by loading main
rows from the archive model and reply snippets from canonical `items`.

Archive reply snippet lookup contract:

- same `source_id`;
- same `ARCHIVE_READ_MODEL_VERSION`;
- target row may be outside the selected export period;
- `reply_to_msg_id` matches `archive_read_items.external_id` for Telegram rows;
- lookup must match
  `archive_read_items.external_id = CAST(reply_to_msg_id AS TEXT)`, not
  `CAST(archive_read_items.external_id AS INTEGER) = reply_to_msg_id`;
- non-numeric target `external_id` rows are ignored for numeric Telegram reply
  lookup, matching the current items-path behavior;
- text snippets use decompressed target `content_zstd`, collapsed whitespace,
  and the existing 280-character truncation rule;
- media-only targets render the existing media snippet fallback;
- missing targets keep the existing `"Original message unavailable"` behavior.
- corrupt or undecodable archive reply target rows are archive loader failures,
  not `"Original message unavailable"` and not reasons to fall back to `items`.

If current legacy numeric behavior depends on `items.external_id`, parity tests
must prove the archive path reproduces the expected result using
`archive_read_items.external_id`.

## Topic Context Boundary

The archive export loader must not re-run legacy topic inference or root
matching. Topic context is already materialized into:

- `archive_read_items.forum_topic_id`;
- `archive_read_items.forum_topic_title`;
- `archive_read_items.forum_topic_top_message_id`.

Parity fixtures should include non-numeric `external_id` cases to prove the
archive builder/materialized topic fields preserve current export behavior and
do not reintroduce legacy `CAST` or root-match mistakes. The export loader only
reads materialized topic fields.

## Failure Handling

Fallback happens only at gate selection time:

```text
state not ready/current -> old items path, normal current behavior
state ready/current     -> archive path
archive path failure    -> typed error, not silent fallback
```

Archive loader failures should surface as typed export/internal errors when the
selected archive rows are corrupt or violate archive-model invariants. Examples
include:

- corrupt compressed `content_zstd`;
- corrupt compressed `media_metadata_zstd`;
- row-level invariant violations discovered while mapping ready/current
  archive rows.

Optional readiness-state failure marking is allowed only for defects
attributable to derived archive rows. The implementation must not mark the
archive model failed for generic database outages, pool timeouts, cancelled
commands, filesystem/output errors, permission errors, or renderer failures.
Archive row decode and invariant failures occur during message loading and may
be classified as archive model defects. Renderer and output failures occur
after message loading and must not mark `archive_read_model_state` failed.

## Testing Requirements

The first implementation task should add failing tests before production code.

Loader selection tests:

- missing `archive_read_model_state` selects `ItemsPath { MissingState }`;
- `never_built` selects `ItemsPath { NeverBuilt }`;
- `building` selects `ItemsPath { Building }`;
- `stale` selects `ItemsPath { Stale }`;
- `failed` selects `ItemsPath { Failed }`;
- ready state with old `model_version` selects
  `ItemsPath { OldModelVersion { .. } }`;
- ready state with current version selects `ArchiveReadModel`.

Parity tests should seed one Telegram source with current fixture coverage and
compare normalized `NotebookLmExportMessage` vectors from:

```text
load_export_messages_from_items_path(...)
load_export_messages_from_archive(...)
```

The fixture must cover:

- full export and bounded period export;
- `published_at` invariants or legacy `NULL published_at` behavior, depending
  on what the schema permits in the tested fixture;
- reply snippet target outside the selected export period;
- reply peer kind/id/top id;
- reaction count;
- forum topic id/title/top message id;
- text-only rows;
- text-with-media rows;
- media-only rows and media placeholders;
- missing reply target fallback;
- non-numeric external ids proving no accidental topic root match.

Gate tests should prove:

- non-ready/current state calls the old items path and preserves existing
  behavior;
- ready/current state calls the archive path;
- ready/current archive path does not silently fall back when archive row
  decoding fails.
- wrapper selection is one-time: when ready/current selects the archive path,
  an injected archive decode failure returns an error and the old path result is
  not returned;
- stale and failed states return the same export message vector as a direct
  `load_export_messages_from_items_path` call for the same source and period
  bounds;
- archive reply lookup uses `archive_read_items`, proven by a fixture where the
  canonical `items` reply target text differs from the archive reply target
  text;
- a corrupt archive reply target row outside the export period fails the archive
  loader when that target is needed for a snippet.

Renderer snapshots should remain focused. The main parity gate is the loaded
export message model, with only focused rendered block assertions where needed
to prove media/reply/topic text still reaches export output.

## Documentation Updates

Update `docs/database-schema.md` to state that NotebookLM export is now a
readiness-gated archive-read consumer for ready/current sources and still falls
back to the old provider/archive path otherwise.

Update `docs/database-schema-read-model-decision.md` to mark the Telegram
NotebookLM export migration slice as implemented and to keep YouTube export
enrichment as future-facing work.

Update `docs/backlog.md` so Database schema simplification no longer lists the
Telegram NotebookLM export archive-model migration as open. Keep the YouTube
playlist-entry read-model decision and current-schema baseline as open.

## Rollout And Containment

This slice should require no first-run full backfill. Existing databases keep
working because not-ready/current archive states use the existing items path.
Sources with ready/current archive rows use the archive export loader.

Containment scans after implementation should verify:

- NotebookLM export does not call live Telegram, network, LLM, or media
  download APIs;
- YouTube NotebookLM export enrichment was not introduced;
- YouTube playlist canonical cleanup was not introduced;
- current-schema baseline work was not introduced;
- source-group export and link enrichment were not introduced.

## Risks And Mitigations

- Silent migration failure: mitigated by one-time gate selection and no
  catch-all fallback after archive loader selection.
- Behavior drift: mitigated by old-path versus archive-path parity tests over
  the loaded export message model.
- Half-migrated reply fidelity: mitigated by reading reply snippets from
  `archive_read_items` whenever the archive path is selected.
- Over-broad failure marking: mitigated by only marking derived archive defects,
  not generic infrastructure or renderer failures.
- Scope creep: mitigated by explicit non-goals for YouTube playlist cleanup,
  current-schema baseline, YouTube export enrichment, source-group export, and
  link enrichment.
