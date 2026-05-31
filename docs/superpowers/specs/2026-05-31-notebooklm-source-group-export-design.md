# NotebookLM Source Group Export Design

> **Status:** Draft for review
> **Date:** 2026-05-31
> **Scope:** Telegram source-group NotebookLM export follow-up

## Context

Extractum already ships single-source NotebookLM export for Telegram sources. The
export is local-only: it reads local SQLite/archive state, renders Markdown
files, writes a manifest and glossary, and does not make live Telegram, LLM,
network link-enrichment, or media-download calls.

Analysis source groups already exist in the workspace and are constrained to a
single provider family for normal app-created groups. Today the NotebookLM
export button is visible for source groups but disabled with the reason that
source-group export is not implemented.

This slice implements the first NotebookLM export follow-up by enabling
NotebookLM export for Telegram source groups only.

## Goals

- Enable `Export for NotebookLM` for selected Telegram source groups.
- Produce one group-level NotebookLM package that preserves member source
  boundaries.
- Reuse existing single-source Telegram export loading, rendering, chunking,
  reply/thread/reaction metadata, topic metadata, and migrated-history behavior.
- Keep export local-only and deterministic.
- Keep YouTube source-group export disabled until YouTube-specific export
  enrichment is designed and implemented.

## Non-Goals

- No YouTube NotebookLM export enrichment.
- No mixed Telegram/YouTube export package.
- No live Telegram fetches during export.
- No LLM calls during export.
- No network link enrichment or link preview cache.
- No media downloads or media byte export.
- No forward metadata enrichment.
- No new topic grouping beyond existing materialized per-message topic context.
- No new global package-level message cap.

## Product Behavior

When a selected source group has `source_type = "telegram"`, the workspace
enables `Export for NotebookLM`. The existing export dialog remains the primary
UI. It keeps the existing controls:

- output folder;
- entire history vs current period;
- media placeholders;
- migrated history opt-in;
- minimum message length;
- max words per file;
- max bytes per file;
- overwrite deterministic export folder.

The dialog description should identify the selected group and member count. The
result surface can remain compact and use the existing file/message/warning
summary. This slice does not require a result DTO expansion. Detailed per-member
facts belong in `.extractum-notebooklm-export.json`, and frontend result UI
should not depend on new response fields.

When a selected source group has `source_type = "youtube"`, the button remains
disabled with a clear reason such as:

```text
YouTube source-group NotebookLM export is not implemented yet.
```

## Request Contract

The frontend and backend request shape becomes scope-neutral:

```ts
type NotebookLmExportRequest = {
  export_id: string | null;
  source_id: number | null;
  source_group_id: number | null;
  output_dir: string;
  period_from: number | null;
  period_to: number | null;
  include_media_placeholders: boolean;
  include_migrated_history: boolean;
  min_message_length: number;
  max_words_per_file: number;
  max_bytes_per_file: number;
  overwrite_existing: boolean;
};
```

Validation rules:

- exactly one of `source_id` or `source_group_id` must be present;
- the `source_id` path preserves current single-source behavior;
- the `source_group_id` path is allowed only for groups whose
  `analysis_source_groups.source_type = "telegram"`;
- unsupported provider groups return a typed validation error with user-facing
  copy;
- group membership is defensively revalidated even though app-created groups
  reject mixed-provider membership.

The Tauri command name may stay `export_source_to_notebooklm` for compatibility,
but the backend model should be internally scope-neutral, for example
`NotebookLmExportScope`.

## Backend Flow

Single-source export remains the foundation. The group path orchestrates
multiple single-source export plans rather than rewriting Telegram rendering for
groups.

1. Validate the request scope.
2. Load the source group row and ordered members.
3. Reject groups whose `source_type` is not `telegram` as unsupported.
4. For each member in deterministic order:
   - load source identity with existing `load_export_source`;
   - skip dirty-data members whose source row is not Telegram with a warning;
   - load current messages through existing `load_export_messages`;
   - if migrated history is enabled, load migrated messages for that member
     through the existing migrated path;
   - filter and render messages using existing `should_export_message`,
     `render_message_block`, chunker, and renderer behavior;
   - record per-member summary and warnings.
5. If no member produces exportable messages, return a hard validation error.
6. Write the group package using the existing overwrite-marker policy.
7. Write a group manifest and group glossary.
8. Return the existing compact result summary with aggregate warnings. Per-member
   details are stored in `.extractum-notebooklm-export.json`, not in new
   required response fields.

## Output Contract

The output folder remains controlled by the existing deterministic export folder
policy. The generated package is one NotebookLM package for the group.

Files should preserve source boundaries:

```text
<export-root>/
  .extractum-notebooklm-export.json
  glossary.md
  sources/
    001-source-42-<source-slug>-part-001.md
    001-source-42-<source-slug>-part-002.md
    002-source-77-<source-slug>-part-001.md
```

Naming rules:

- member ordering follows the existing group member display/query order:
  `COALESCE(sources.title, '')`, then `sources.id`;
- the same order is used for export execution, filenames, and manifest member
  summaries;
- every member file prefix includes the one-based member index and `source_id`;
- the title slug uses existing filename sanitization;
- equal source titles cannot collide because `source_id` is part of the prefix;
- chunk part numbering restarts per member;
- current and migrated history sections remain inside that member's file family;
- overwrite behavior follows the existing marked-folder safety policy.

The exact directory names may follow existing export conventions, but the
implementation must keep the collision-safe member prefix.

## Manifest Contract

The manifest is group-level. It should include enough information to explain a
best-effort export without reading logs.

Minimum fields/content:

- export scope: `source_group`;
- group id and group name;
- generated-at timestamp;
- date range;
- migrated history option;
- media placeholder option;
- min/max limit options;
- total member count;
- exported member count;
- skipped member count;
- total exported message count;
- total skipped message count;
- aggregate warnings;
- per-member source id, title, subtype, exported message count, skipped message
  count, generated chunk files, and skip/warning reason.

The existing `.extractum-notebooklm-export.json` marker remains the
authoritative manifest for overwrite safety and generated-file tracking. The
group metadata should be added to that JSON. No new human-readable `manifest.md`
is required in this slice. The implementation should not hide member skip
reasons only in logs.

Generated-file tracking must include every file written by the group export,
including `glossary.md` and each `sources/...part-XXX.md` file. Existing
overwrite cleanup and child-path validation must continue to work for files in
the `sources/` subdirectory.

## Glossary Contract

The group glossary aggregates participants across exported members. It should
preserve enough source context to avoid confusing identical author names from
different Telegram sources.

Minimum acceptable behavior for this slice:

- aggregate using existing participant summary behavior;
- write one group-level glossary;
- keep source-scoped Markdown files as the primary provenance boundary.

If participant names collide across sources, exact disambiguation can remain a
future enhancement as long as source file boundaries and manifest member
summaries are clear. The implementation plan should not add new author
collision handling in this slice.

## Limits Semantics

Existing single-source limits apply per member:

- `min_message_length` filters each member's messages independently;
- `max_words_per_file` applies to each generated chunk file;
- `max_bytes_per_file` applies to each generated chunk file;
- date range applies equally to every member;
- chunk numbering restarts per member.

There is no new global cap across the entire source group in this slice. A
future export-profile design can add global package caps if users need them.

## Migrated History Semantics

Migrated history remains explicit opt-in and current-history-only remains the
default.

When `include_migrated_history = true` for a Telegram source group:

- the option applies independently to each Telegram member;
- current and migrated history remain separate sections for each member;
- a member with no matching migrated messages produces a warning, not a group
  failure;
- a member without migrated history availability does not block other members;
- migrated-history loading continues to use the existing items-path behavior.

## Warning And Error Taxonomy

Warnings skip or annotate a member while allowing the group export to continue:

- a group has `source_type = "telegram"`, but a dirty-data member source row is
  not Telegram;
- a member has zero exportable messages after date and length filters;
- migrated history was requested but no migrated messages matched for that
  member;
- a member falls back from archive read model to the existing items path, if
  that condition is surfaced by the current export loader;
- a member has local data that cannot contribute to NotebookLM Markdown but does
  not prevent other members from exporting.

Hard errors stop the export:

- both `source_id` and `source_group_id` are missing;
- both `source_id` and `source_group_id` are provided;
- the selected group does not exist;
- the selected group provider is unsupported, including YouTube groups;
- the selected group has no valid Telegram members, with user-facing copy:
  `No Telegram sources found in this source group.`;
- no Telegram member produces any exportable messages, with user-facing copy:
  `No exportable Telegram messages found for this source group.`;
- output folder validation fails;
- overwrite safety policy rejects the target folder;
- filesystem writes fail.

Hard errors may leave partial files if they happen mid-write, matching the
existing single-source export cleanup posture. This slice does not add a new
transactional filesystem cleanup layer.

## Frontend Wiring

The workspace should derive NotebookLM export availability from the active
selection:

- selected single source: existing behavior unchanged;
- selected Telegram source group: export enabled;
- selected YouTube source group: export visible but disabled with a provider
  reason;
- no source or group: export hidden or disabled according to existing workspace
  behavior.

The request builder should accept either a source id or source group id and set
the other field to `null`.

Progress events remain compatible and best-effort. Exact per-member progress is
not required in this slice. Progress messages may include member display names
only if they are safe user-facing source titles already visible in the app.

## Testing

Backend tests:

- request validation rejects no scope and both scopes;
- single-source request still follows the current path;
- Telegram source-group request loads members in deterministic order;
- YouTube source-group request returns a validation error;
- dirty non-Telegram member in an otherwise Telegram group is skipped with a
  warning;
- empty member is skipped with a warning;
- all-empty group fails with a no-exportable-messages error;
- member filenames include member index, source id, and sanitized title slug;
- duplicate source titles do not collide;
- migrated-history opt-in applies per member;
- `.extractum-notebooklm-export.json` records group scope and per-member
  summaries;
- existing single-source export tests still pass.

Frontend tests:

- NotebookLM export is enabled for selected Telegram groups;
- NotebookLM export remains disabled for selected YouTube groups with explicit
  reason;
- request builder emits `source_group_id` for group selection and `source_id`
  for single-source selection;
- dialog description can render group name/member count;
- existing single-source dialog and API wrapper tests still pass.

Smoke/manual checks:

- use an existing Telegram source group fixture with at least two synced
  members;
- export the current period and entire history;
- verify the package contains `.extractum-notebooklm-export.json`, `glossary.md`,
  and `sources/` files;
- verify source boundaries are readable in file names and document headings;
- verify warnings are visible when a member has no matching exportable messages.

## Acceptance Criteria

- Telegram source groups can be exported for NotebookLM from the analysis
  workspace.
- YouTube source groups remain unsupported with clear user-facing copy.
- Single-source Telegram NotebookLM export behavior is unchanged.
- The group package preserves member source boundaries and does not merge
  separate Telegram sources into one chronological chat.
- Best-effort member skips are recorded as warnings and in the manifest.
- An all-empty group fails instead of writing a misleading empty package.
- Existing local-only export boundaries are preserved.
