# Migrated History Scope Product Behavior Design

## Goal

Define how users browse, analyze, and export Telegram migrated
small-group history after explicit historical-scope import is available.

This design applies to the historical scope generally, not only to the current
source `115` validation fixture. Source `115` / batch `20` proved the storage
and import path: explicit migrated-history import can write migrated rows with
no bad migrated-domain rows and without adding those rows to default
`analysis_documents` or `archive_read_items` projections. The remaining product
question is how a user intentionally uses that historical scope.

## Chosen Policy

Default behavior remains current history only.

Migrated small-group history is a separate historical scope. It is not part of
normal current supergroup reruns, and it must not become part of default
browsing, analysis, or export by surprise. Any view or workflow that includes
migrated history must require an explicit user choice.

The stable scope labels are:

- Current supergroup history
- Migrated small-group history
- Merged timeline

Compact UI may shorten these labels visually, but DTOs, APIs, tests, and
stored metadata use stable enum values:

```ts
type TelegramHistoryScope = 'current' | 'migrated' | 'merged';
```

Rows from the migrated historical scope must carry a visible label when mixed
with current history, such as `Migrated from old group` or
`Historical small-group history`.

## Browsing

The source reader opens in `Current supergroup history` by default.

When a source has imported migrated rows, the reader shows a scope control:

```text
Current supergroup history | Migrated small-group history | Merged timeline
```

Scope behavior:

- `current` / `Current supergroup history` shows only current supergroup rows.
- `Migrated small-group history` shows only rows with
  `migration_domain = migrated_from_chat` and `is_migrated_history = 1`.
- `Merged timeline` combines current and migrated rows, but every migrated row
  must be visibly labeled.

`Merged timeline` is never the default. It is an explicit viewing mode for users
who want conversational continuity and understand that old rows came from the
pre-upgrade small-group history.

If migrated history is only available but not imported yet, browsing keeps the
existing explicit import action/status and does not expose an empty historical
scope mode.

Capability and import-state UI rules:

- `available` but not imported: show a CTA/status such as
  `Migrated small-group history available`.
- imported with `migrated_row_count > 0`: show the scope control.
- imported and completed with `migrated_row_count = 0`: show an informational
  empty state such as `Migrated small-group history imported; no messages were
  found`. Do not show the normal scope control unless a direct route/state
  requests the historical scope; in that case, render the same informational
  empty state.

## API And DTO Contract

Backend defaults must match UI defaults. Direct command calls without a scope
parameter return current history only.

Source-reader item commands accept an optional history-scope parameter:

```ts
history_scope?: 'current' | 'migrated' | 'merged'
```

The backend default is `current`.

Returned item DTOs expose backend-owned scope metadata so the frontend does not
infer migrated labels from ad hoc heuristics:

```ts
history_scope: 'current' | 'migrated'
is_migrated_history: boolean
migration_domain: null | 'migrated_from_chat'
history_scope_label: 'Current supergroup history' | 'Migrated small-group history'
```

For `merged` queries, each row still has row-level `history_scope` of `current`
or `migrated`. The query-level scope is the request mode, not the row label.

## Merged Timeline Ordering

Merged timeline ordering must be deterministic across paging, around-item
loading, snapshots, and export.

Order rows by:

1. `published_at`;
2. history-scope order, with current supergroup rows before migrated
   small-group rows when timestamps tie;
3. native Telegram identity, using `history_peer_kind`, `history_peer_id`, and
   `telegram_message_id`;
4. local `item_id` as the final tie-breaker.

The implementation may reverse the primary direction for newest-first readers,
but the same tie-breakers must remain stable within that direction.

## Analysis

Default corpus selection remains current history only.

Report setup adds an explicit option such as:

```text
Include migrated historical scope
```

When this option is off, migrated rows stay excluded from analysis corpus
construction. When it is on, corpus capture includes migrated rows and records
the scope decision in saved-run metadata. Follow-up chat, evidence, run detail,
and saved-run inspection must be able to explain whether historical rows were
included.

The first implementation avoids adding a new persistent projection table. The
analysis corpus loader takes an `include_migrated_history` flag and, when
enabled, adds a direct scoped query over `items` plus `telegram_messages` for
migrated rows. The saved snapshot stores the resulting corpus and the metadata
decision, so completed runs remain reproducible without changing the default
`analysis_documents` projection.

Run-level snapshot decision storage uses a new nullable field on
`analysis_runs`:

```text
telegram_history_scope = current | current_plus_migrated
```

This is the justified schema addition for the first implementation. Existing
`analysis_runs` has snapshot fields and `youtube_corpus_mode`, but no general
run-level metadata blob. A dedicated nullable field keeps the Telegram scope
decision queryable, easy to expose in saved-run DTOs, and aligned with the
existing YouTube corpus-mode pattern.

Message-level metadata stays in `analysis_run_messages.metadata_zstd` and
includes historical-scope markers for Telegram rows:

```yaml
history_scope: current | migrated
migration_domain: null | migrated_from_chat
history_peer_kind: channel | chat
history_peer_id: <private numeric peer id>
```

Docs and UI expose only sanitized labels and booleans. Raw peer ids remain
internal snapshot metadata for deterministic evidence resolution and must not be
rendered in tracked docs or user-facing copy.

The snapshot contract is important: a completed run must not later look as if
migrated history was part of the ordinary source history. The saved snapshot
must preserve the explicit inclusion decision and expose migrated evidence with
a historical-scope marker.

## Export

Default export remains current history only.

NotebookLM and related export flows add an explicit option such as:

```text
Include migrated historical scope
```

When included, export output must make the historical scope visible. Acceptable
first-slice output uses separate sections:

```text
# Current supergroup history
...

# Migrated small-group history
...
```

Exported migrated rows carry markers equivalent to:

```yaml
history_scope: migrated_small_group_history
migration_domain: migrated_from_chat
```

The export must not silently flatten migrated small-group history into one
continuous current-history source, because downstream tools would treat that as
ordinary source history.

Merged export is deferred. It can be designed later if users need a continuity
export after the safer sectioned export exists.

## Data And Read-Model Contract

The existing storage markers remain the source of truth:

- `telegram_messages.is_migrated_history = 1`
- `telegram_messages.migration_domain = migrated_from_chat`
- native old-history identity in `history_peer_kind` / `history_peer_id`

Default `analysis_documents` and `archive_read_items` projections continue to
exclude migrated rows. Opted-in browsing, analysis, and export may query or
build scoped projections, but they must not weaken the default exclusion
contract.

Schema changes are non-goals unless implementation proves that explicit scope
selection cannot be represented safely with existing columns and snapshot/export
metadata.

## Reply And Topic Semantics

Reply lookup respects history scope.

- Replies inside migrated rows look up targets inside the original old-history
  domain.
- Replies inside current rows look up targets inside current supergroup
  history.
- Cross-scope replies are not guessed. A migrated row and a current row are
  connected only through an explicit native identity match or a later designed
  bridge.

Forum topic semantics also stay scoped. Current supergroup topic filters do not
automatically apply to migrated small-group history, because the old group did
not necessarily have forum topics. A source reader or export can show migrated
rows under a historical-scope section, but it must not infer current forum-topic
membership for those rows.

## Implementation Slicing

Implement in this order:

1. Browsing first: backend scope parameter, DTO markers, deterministic merged
   ordering, source reader segmented control, and tests for `current`,
   `migrated`, and `merged`.
2. Export second: explicit opt-in and separate current/migrated sections.
   Tests cover default exclusion and marker inclusion.
3. Analysis third: report setup opt-in, preflight counts, corpus loader flag,
   run-level `telegram_history_scope`, snapshot metadata markers, and evidence
   labels.
4. Docs/backlog cleanup after implementation: move completed behavior from
   backlog into current-state docs and archive superseded planning notes only
   when they remain useful as historical context.

## Non-Goals

- Do not make merged timeline the default browsing mode.
- Do not include migrated rows in default analysis corpus selection.
- Do not include migrated rows in default NotebookLM/export output.
- Do not create default `analysis_documents` or `archive_read_items` rows for
  migrated history.
- Do not add a persistent scoped analysis projection table in the first
  implementation.
- Do not rewrite migrated old-chat rows into current supergroup identity.
- Do not design deletion, purge, or unimport behavior in this slice.
- Do not implement merged NotebookLM/export output in the first slice.

## Tests And Validation

The implementation plan includes tests proving:

- default browsing/read-model paths exclude migrated rows;
- backend direct calls default to `current` when `history_scope` is omitted;
- opted-in migrated-history browsing returns migrated rows with labels;
- merged timeline includes both domains and labels migrated rows;
- merged timeline ordering is stable across paging and around-item loading;
- available-but-not-imported and imported-zero-row states render explanatory
  status instead of empty broken readers;
- default analysis corpus excludes migrated rows;
- opted-in analysis corpus includes migrated rows and persists the decision in
  `analysis_runs.telegram_history_scope`;
- opted-in snapshot rows include message-level historical markers in
  `analysis_run_messages.metadata_zstd`;
- default export excludes migrated rows;
- opted-in export writes separate current and migrated sections with visible
  historical-scope markers;
- reply and topic lookup does not infer current supergroup topic/reply semantics
  for migrated rows;
- existing source `115` E2E evidence remains valid: migrated rows have no bad
  migrated-domain flags and default projections contain zero migrated rows.
