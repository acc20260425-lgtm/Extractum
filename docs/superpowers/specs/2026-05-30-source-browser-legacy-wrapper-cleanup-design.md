# Source Browser Legacy Wrapper Cleanup Design

> Date: 2026-05-30
> Status: merged into main on 2026-05-30
> Scope: cleanup after live source group and run snapshot migration to `SourceBrowserShell`.

## Summary

Remove legacy compatibility wrapper components that no longer participate in
production source browsing, but only after a hard preflight proves they have no
production imports or usages. The cleanup should make `SourceBrowserShell` the
clear canonical production render path for live source groups and available run
snapshots.

This is a cleanup slice, not a behavior slice. It must not change live source
group browsing, run snapshot browsing, tab sets, route state, backend APIs, or UI
layout.

## Current Context

The browser migration work moved these production paths into
`SourceBrowserShell`:

- live single-source browsing;
- live source group browsing;
- available run snapshot browsing.

Two legacy components may now be architectural noise:

- `src/lib/components/analysis/source-group-reader.svelte`;
- `src/lib/components/analysis/run-snapshot-messages-panel.svelte`.

They should be deleted only if the code audit shows that production code no
longer imports or renders them. Matches from raw contract tests and historical
docs are migration targets, not production blockers.

## Primary Invariant

`SourceBrowserShell` is the canonical production render path for live source
groups and available run snapshots.

Future production code should use the shell and canonical leaf components rather
than reintroducing wrapper readers around already-migrated browser surfaces.

## Goals

- Delete unused legacy wrapper components when preflight proves they are no
  longer used by production source code.
- Move tests away from deleted wrapper raw files and toward canonical browser
  contracts.
- Keep raw and behavioral tests focused on `SourceBrowserShell`,
  `SourceGroupSourcesView`, `SnapshotGroupSourcesView`, `SnapshotItemsView`,
  `RunSnapshotMetadataView`, `source-browser-model.ts`, and
  `ReportSourceSurface` routing.
- Update docs that still describe direct group or snapshot wrapper readers as a
  valid production path.
- Preserve all existing live and snapshot browser behavior.

## Non-Goals

- No backend changes.
- No new browser subjects, tabs, defaults, or reconciliation rules.
- No UI redesign.
- No shell prop repacking.
- No route state restructuring.
- No source item or snapshot item model changes.
- No changes to snapshot availability, snapshot paging, live group paging, or
  evidence navigation behavior.

## Guarded Deletion

Before deleting any candidate file, run an audit equivalent to:

```bash
rg -n "SourceGroupReader|source-group-reader|RunSnapshotMessagesPanel|run-snapshot-messages-panel" src docs
```

Classify matches as:

- production source usage: non-test files under `src`;
- raw-test usage: `*.test.ts` files;
- docs usage: files under `docs`.

Deletion is allowed only if production source usage is empty for the candidate
component. Raw-test imports and docs references do not block deletion; they must
be updated as part of the cleanup.

If any production usage remains, stop deletion for that component and downgrade
the slice to docs/test cleanup only for that component. Do not remove a wrapper
that production code still imports or renders.

## Test Contract Updates

Tests should stop treating deleted wrappers as protected compatibility surfaces.
The replacement contracts should assert the current canonical architecture:

- `ReportSourceSurface` routes live source groups and available run snapshots
  through `SourceBrowserShell`.
- Available run snapshots do not render `Activity`, source jobs, Takeout
  recovery, sync/retry/cancel CTAs, or legacy wrapper readers.
- Live source groups still use the `Sources | Items | Metadata | Activity`
  browser contract.
- Snapshot browser leaves import no live API wrappers and do not call `invoke`.
- Snapshot items remain based on frozen `SourceReaderItem` rows, not live
  `SourceItem` rows.
- `SourceBrowserShell` remains the only production shell for migrated source
  browser subjects.

The tests may still use raw source assertions where they guard architecture, but
they should read canonical files rather than deleted wrappers.

## Documentation Updates

Update current-state, architecture, specs, or implementation notes that still
describe `SourceGroupReader` or `RunSnapshotMessagesPanel` as active production
paths for live groups or available run snapshots.

Historical plan files may remain historical, but any current-state or
forward-looking docs should name the canonical shell and leaf components.

## Verification

The implementation plan should include:

- focused frontend tests for source browser model, shell routing, group browser
  leaves, snapshot browser leaves, and report surface routing;
- `npm.cmd run check`;
- full `npm.cmd run verify`;
- post-deletion search:

```bash
rg -n "SourceGroupReader|source-group-reader|RunSnapshotMessagesPanel|run-snapshot-messages-panel" src
```

Expected post-deletion result for `src`: no matches, except temporary matches
inside tests before those tests are migrated in the same task.

## Rollout

This cleanup should land as one small follow-up slice after the completed source
browser migrations. It should reduce dead architectural surface area while
leaving all user-visible browsing behavior unchanged.
