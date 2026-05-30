# Run Snapshot Source Browser Design

> Date: 2026-05-30
> Status: draft design, pending review
> Scope: saved run snapshot source browsing in the `/analysis` Source canvas.

## Summary

Move available saved run snapshots into the shared source browser model while
preserving frozen snapshot semantics. Live source browsing and live source group
browsing already use `SourceBrowserShell`; saved run snapshots still render
specialized readers directly from `ReportSourceSurface`.

This slice should make snapshot browsing feel structurally consistent with live
browsing without pretending a frozen run corpus is a live source. Snapshot rows
remain route-owned, paged through run snapshot APIs, and free of live source
actions, source jobs, sync controls, and group-scoped activity.

## Current Context

Live browsing currently uses the shared browser shell:

- Telegram live source: `Timeline | Items | Metadata | Activity`;
- YouTube video live source: `Transcript | Comments | Items | Metadata | Activity`;
- YouTube playlist live source: `Videos | Items | Metadata | Activity`;
- live source group: `Sources | Items | Metadata | Activity`.

Saved run snapshot browsing still has a separate branch in
`ReportSourceSurface` for `sourceViewBasis === "run_snapshot"`:

- source group snapshots render `SourceGroupReader`;
- YouTube transcript snapshots render `YoutubeTranscriptReader`;
- Telegram and fallback snapshots render `TelegramTimelineReader`;
- unavailable, pending, and checking snapshots render status messages;
- `SourceReaderHeader` owns the visible `Run snapshot` basis label and the
  `View live source` action.

Snapshot rows are loaded as `AnalysisRunMessage[]` and converted to
`SourceReaderItem[]` through `analysisRunMessageToReaderItem`. These rows are a
frozen run corpus, not live `SourceItem[]`.

## Goals

- Add snapshot-aware browser subjects to the existing source browser model.
- Route available saved run snapshots through browser tabs instead of direct
  reader branches.
- Preserve the existing `Run snapshot` header and `View live source` route
  transition.
- Keep `/analysis` as the owner of snapshot rows, selected snapshot source,
  snapshot source options, paging callbacks, availability state, and evidence
  navigation.
- Use provider-aware primary tabs where the frozen rows support them.
- Provide a snapshot `Items` view over loaded frozen rows without converting
  them into live `SourceItem[]`.
- Provide a lightweight snapshot metadata view based only on route-owned run and
  snapshot fields.
- Keep source activity, source jobs, sync CTAs, Takeout controls, playlist
  actions, and live source metadata out of snapshot browsing.

## Non-Goals

- Do not change snapshot capture, persistence, or backend query semantics.
- Do not add backend-global search across all captured rows.
- Do not add live source loading, source sync, retry, or job cancellation from a
  snapshot tab.
- Do not make unavailable or pending snapshots enter the browser shell.
- Do not require the live source or live source group to still exist in order to
  browse a captured snapshot.
- Do not migrate the separate run snapshot messages panel unless it is already
  part of the Source canvas snapshot branch.
- Do not add a second shell-like wrapper unless implementation proves the shared
  shell cannot stay readable.

## Subject Model

Keep the current live subject shapes working, and add a frozen snapshot subject
rather than overloading live `source` or `source_group` subjects.

Recommended shape:

```ts
export type SourceBrowserSubject =
  | { kind: "source"; source: Source }
  | { kind: "source_group"; group: AnalysisSourceGroup }
  | { kind: "run_snapshot"; snapshot: RunSnapshotBrowserSubject };

export type RunSnapshotBrowserKind =
  | "source_group"
  | "telegram_timeline"
  | "youtube_transcript"
  | "generic_items";

export interface RunSnapshotBrowserSubject {
  runId: number;
  scopeType: "source" | "source_group";
  scopeLabel: string;
  readerKind: RunSnapshotBrowserKind;
  sourceType: string | null;
  sourceSubtype: string | null;
}
```

The snapshot subject is an identity and tab-routing object. It should not carry
the loaded snapshot rows, member source list, paging state, or action callbacks.
Those remain route-owned props.

`readerKind` is derived by the route from already-loaded snapshot rows and
already-known run/source metadata:

- `source_group` for saved group snapshots;
- `youtube_transcript` when the snapshot can render transcript rows;
- `telegram_timeline` when the snapshot can render Telegram timeline rows;
- `generic_items` when no primary provider reader is safe for the loaded rows.

If metadata and loaded rows disagree, prefer the safer `generic_items` fallback
instead of inventing a partial provider reader.

## Tabs

Snapshot subjects intentionally do not expose `Activity`, `Comments`, or
`Videos` in this slice. Those tabs have live-source semantics today.

Tabs by snapshot reader kind:

| Snapshot reader kind | Tabs |
| --- | --- |
| `source_group` | `Sources | Items | Metadata` |
| `telegram_timeline` | `Timeline | Items | Metadata` |
| `youtube_transcript` | `Transcript | Items | Metadata` |
| `generic_items` | `Items | Metadata` |

Smart defaults:

| Snapshot reader kind | Default tab |
| --- | --- |
| `source_group` | `Sources` |
| `telegram_timeline` | `Timeline` |
| `youtube_transcript` | `Transcript` |
| `generic_items` | `Items` |

Tab reconciliation:

- live source/group -> run snapshot: preserve `items` and `metadata`; otherwise
  use the snapshot smart default;
- run snapshot -> live source/group: preserve `items` and `metadata` when the
  target supports them; otherwise use the target smart default;
- run snapshot -> run snapshot: preserve the active tab when supported by the
  next snapshot subject; otherwise use the next snapshot smart default;
- `activity`, `comments`, and `videos` never survive into a snapshot subject in
  this slice.

## Data Flow

`ReportSourceSurface` remains the owner of snapshot state:

- `snapshotReaderItems`;
- `selectedSnapshotSourceId`;
- `snapshotSourceOptions`;
- `loadingRunSnapshotMessages`;
- `hasMoreRunSnapshotMessages`;
- `selectedTraceRef`;
- `onChangeSelectedSnapshotSourceId`;
- `onLoadMoreRunSnapshotMessages`;
- `onViewLiveSource`;
- snapshot availability and snapshot error state.

Only available snapshots enter the browser shell. Pending, unavailable, and
checking snapshots keep the existing status surfaces.

The shell receives snapshot data through a grouped prop, for example:

```ts
snapshotBrowserData?: {
  readerItems: SourceReaderItem[];
  selectedSourceId: number | null;
  sourceOptions: SourceFilterOption[];
  loading: boolean;
  hasMore: boolean;
  selectedTraceRef: string | null;
  onLoadMore: () => void | Promise<void>;
}
```

The shell and snapshot leaf components must not import API wrappers and must not
call `invoke`.

## UI Contract

The existing `SourceReaderHeader` can remain outside the browser shell. The
header continues to show:

- `Run snapshot`;
- frozen source material subtitle;
- source focus selector for saved group snapshots;
- `View live source` when the route can attempt a live transition.

The browser tabs appear below that header for available snapshots.

### Sources

For group snapshots, `Sources` reuses `SourceGroupSourcesView` or an equivalent
leaf over `SourceReaderItem[]`.

Important differences from live groups:

- paging is run-snapshot paging, not true per-source live paging;
- `Load older snapshot messages` remains a global frozen-run action;
- member source attribution comes from frozen snapshot rows and source filter
  options, not live group membership refreshes;
- selected evidence refs must still highlight or scroll into the matching
  source section.

### Timeline

For Telegram snapshots, `Timeline` renders frozen `SourceReaderItem[]` with
`TelegramTimelineReader` or an equivalent leaf. It must not show Takeout
controls, source sync state, topic refresh controls, or live source diagnostics.

### Transcript

For YouTube transcript snapshots, `Transcript` renders frozen
`SourceReaderItem[]` with `YoutubeTranscriptReader` in snapshot mode. It must
not show transcript sync, metadata sync, comments sync, playlist actions, or
live YouTube job state.

### Items

Snapshot `Items` is a flat loaded-window view across frozen `SourceReaderItem[]`.
Do not feed these rows into `UniversalItemsView` unless there is an explicit,
typed adapter. A focused `SnapshotItemsView` is preferred if the existing
universal items view remains `SourceItem[]`-oriented.

The view should support the same user expectations as live loaded-window items:

- search within loaded rows;
- item kind chips;
- newest/oldest sort;
- member source labels for group snapshots;
- selected evidence highlighting when `selectedTraceRef` matches a row ref;
- `Load older snapshot messages` when more captured rows are available.

Help copy:

```text
Snapshot items are limited to frozen rows loaded for this run. Load older snapshot messages to fetch more captured rows.
```

### Metadata

Snapshot `Metadata` is run-snapshot metadata, not live source metadata.

Show optional-safe route-owned fields only:

- run id and run title;
- source basis: `Run snapshot`;
- snapshot availability;
- scope type and scope label;
- source type/subtype if already known;
- loaded row count;
- member source count and source option list for group snapshots;
- run created/completed timestamps when already exposed to the frontend;
- snapshot error when the current availability state already exposes it.

Do not decode live source metadata blobs or item raw payloads for this tab.

## Error And Empty States

- Pending snapshot: keep the existing `Snapshot pending` status, no tabs.
- Unavailable snapshot: keep the existing `Snapshot unavailable` status and
  error text, no tabs.
- Checking snapshot: keep the existing checking status, no tabs.
- Available snapshot with zero loaded rows: show an empty snapshot browser with
  `Items | Metadata` if the route cannot infer a provider primary tab.
- Missing live source or deleted group: snapshot browsing still works from
  frozen rows and run labels; live transition remains a separate route-owned
  decision.

## Invariants

- Run snapshot subjects are frozen browsing subjects, not live source subjects.
- Snapshot tabs never render `SourceActivityView`.
- Snapshot tabs never render source sync CTAs, Takeout CTAs, retry actions, or
  cancel job actions.
- Snapshot `Items` operates on `SourceReaderItem[]`, not live `SourceItem[]`.
- Snapshot `Sources`, `Timeline`, and `Transcript` use already-loaded snapshot
  rows and route callbacks only.
- Snapshot browser components do not import API wrappers and do not call
  `invoke`.
- Evidence navigation stays route-owned and is passed down as `selectedTraceRef`
  or precomputed `SourceReaderItem.selected`.
- `View live source` changes the source basis/selection through the route; it
  does not create a nested live browser inside the snapshot browser.
- Live source and live source group behavior remains unchanged.

## Testing Strategy

Model tests:

- snapshot subjects get the expected tabs and smart defaults;
- live source/group tab behavior is unchanged;
- reconciliation preserves `items` and `metadata` across live/snapshot
  transitions;
- `activity`, `comments`, and `videos` fall back when entering snapshot
  subjects;
- snapshot-to-snapshot transitions preserve supported primary tabs only when
  both subjects support them.

Component and route contract tests:

- available run snapshots enter `SourceBrowserShell`;
- pending/unavailable/checking snapshots stay on status surfaces;
- saved group snapshots use `Sources | Items | Metadata`;
- saved Telegram snapshots use `Timeline | Items | Metadata`;
- saved YouTube transcript snapshots use `Transcript | Items | Metadata`;
- generic snapshots use `Items | Metadata`;
- snapshot branches do not render `SourceActivityView`;
- snapshot branches do not pass source job props or sync callbacks into reader
  leaves;
- `selectedTraceRef` reaches group and single-source snapshot readers;
- snapshot `Items` preserves source attribution for group rows.

Manual Tauri smoke:

- seed analysis redesign fixtures;
- open `__analysis_redesign_fixture__ Group Snapshot Run`;
- switch to Source mode and verify `Run snapshot` header plus browser tabs;
- verify `Sources` default for group snapshots;
- verify `Items` shows frozen rows, source labels, and snapshot help copy;
- verify `Metadata` shows run snapshot fields;
- verify no Activity tab, source job card, sync CTA, or Takeout CTA appears;
- verify `View live source` still transitions out of snapshot browsing.

## Rollout

Implement this as one focused slice after this spec is approved:

1. Extend the browser model with snapshot subjects and tests.
2. Add snapshot-specific `Items` and `Metadata` leaves.
3. Add snapshot data props to `SourceBrowserShell` without API imports.
4. Route available snapshot branches in `ReportSourceSurface` through the shell.
5. Keep unavailable/pending/checking status branches unchanged.
6. Run focused frontend tests, full verification, and Tauri smoke.

This prepares the browser model for frozen run corpus navigation without
changing live source browsing semantics.
