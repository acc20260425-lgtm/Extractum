# Source Browser Data Prop Consolidation Design

> Date: 2026-05-30
> Status: approved design, pending implementation plan
> Scope: live single-source data prop consolidation in `SourceBrowserShell`.

## Summary

Consolidate live single-source inputs to `SourceBrowserShell` into one
route-owned `sourceBrowserData` object. This removes the largest remaining
prop-noise in the shell after the source group, run snapshot, playlist, and
legacy wrapper cleanup slices.

This is a prop-shape cleanup only. It must not change browser subjects, tab
sets, smart defaults, reconciliation, route state ownership, backend behavior,
loading semantics, sync/job callbacks, UI layout, or user-visible copy.

## Current Context

`SourceBrowserShell` is now the canonical production render path for:

- live Telegram sources;
- live YouTube video sources;
- live YouTube playlist sources;
- live source groups;
- available run snapshots.

The grouped paths already have data objects:

- `groupBrowserData` for live source groups;
- `snapshotBrowserData` for run snapshots.

The live single-source path still passes many standalone props into
`SourceBrowserShell`, including live reader rows, generic source items, topics,
YouTube detail/transcript/playlist state, source jobs, Takeout recovery, and
source-only callbacks. This makes the shell invocation noisy and forces
source-only dummy values into non-source branches.

## Primary Invariants

- This slice only changes prop shape for live single-source browser data.
- `sourceBrowserData` is used only when `subject.kind === "source"`.
- `groupBrowserData` remains the live source-group data object.
- `snapshotBrowserData` remains the run-snapshot data object.
- Group and snapshot branches in `ReportSourceSurface` must not pass
  source-only dummy values just to satisfy `SourceBrowserShell` props.

## Goals

- Add a `SourceBrowserData` type local to `SourceBrowserShell` or exported from a
  nearby analysis component module if implementation needs sharing.
- Move existing live single-source shell inputs into `sourceBrowserData`.
- Update `ReportSourceSurface` so the live single-source branch builds
  `sourceBrowserData={{ ... }}`.
- Keep `groupBrowserData` and `snapshotBrowserData` names and shapes unchanged.
- Remove source-only dummy values from live group and run snapshot shell
  invocations where they are no longer needed.
- Keep all existing browser behavior unchanged.
- Keep `SourceBrowserShell` API-free: no `$lib/api/*` imports and no `invoke`.

## Non-Goals

- Do not rename `groupBrowserData`.
- Do not rename `snapshotBrowserData`.
- Do not introduce `sourceGroupBrowserData` or `runSnapshotBrowserData`.
- Do not split `sourceBrowserData` into nested `items`, `youtube`, `activity`,
  or `actions` sub-objects in this slice.
- Do not change source browser subjects.
- Do not change tab availability, labels, smart defaults, or reconciliation.
- Do not move route state ownership into the shell.
- Do not change backend APIs or data loading.
- Do not change UI layout, copy, or visual styling.

## Proposed Prop Shape

`SourceBrowserShell` should keep only the small set of top-level props that are
shared across subject kinds or still needed for compatibility:

```ts
type Props = {
  subject?: SourceBrowserSubject | null;
  source?: Source | null;
  sourceBrowserData?: SourceBrowserData | null;
  groupBrowserData?: SourceGroupBrowserData | null;
  snapshotBrowserData?: SnapshotBrowserData | null;
  selectedTraceRef: string | null;
  formatTimestamp: (value: number | null) => string;
};
```

If `source` can be removed without widening the implementation, that is allowed,
but removing it is not a goal. Keeping it as compatibility or incremental
migration support is acceptable.

Recommended `SourceBrowserData` shape:

```ts
type SourceBrowserData = {
  liveReaderItems: SourceReaderItem[];
  sourceItems: SourceItem[];
  sourceRouteError: string | null;
  sourceItemsHasMore: boolean;
  loadingItems: boolean;

  sourceTopics: SourceForumTopic[];
  loadingSourceTopics: boolean;
  selectedTopicKey: string;
  showTopicSelector: boolean;

  youtubeVideoDetail: YoutubeVideoDetail | null;
  youtubePlaylistDetail: YoutubePlaylistDetail | null;
  youtubeTranscriptSegments: YoutubeTranscriptSegment[];
  youtubeTranscriptSearch: string;
  youtubeTranscriptHasMore: boolean;
  loadingYoutubeTranscriptSegments: boolean;
  loadingYoutubeDetail: boolean;

  sourceJobs: SourceJobRecord[];
  takeoutRecovery: TakeoutImportRecoveryState | null;
  sourceSyncDisabledReason: (source: Source) => string | null;

  telegramHistoryScope: TelegramHistoryScope;
  currentSourceContentLabel: string;

  onLoadMoreSourceItems: () => void | Promise<void>;
  onChangeSelectedTopicKey: (key: string) => void | Promise<void>;
  onChangeTelegramHistoryScope: (scope: TelegramHistoryScope) => void;
  onChangeTranscriptSearch: (query: string) => void;
  onLoadMoreYoutubeTranscriptSegments: () => void | Promise<void>;

  onOpenSource: (sourceId: number) => void | Promise<void>;
  onSyncSource: (sourceId: number) => void | Promise<void>;
  onSyncYoutubeMetadata: (sourceId: number) => void | Promise<void>;
  onSyncYoutubeTranscript: (sourceId: number) => void | Promise<void>;
  onSyncYoutubeComments: (sourceId: number) => void | Promise<void>;
  onSyncYoutubePlaylist: (sourceId: number) => void | Promise<void>;
  onRetryFailedYoutubePlaylistVideos: (sourceId: number) => void | Promise<void>;
  onSyncYoutubePlaylistVideo: (
    playlistSourceId: number,
    videoSourceId: number,
  ) => void | Promise<void>;
  onRetryYoutubePlaylistVideo: (
    playlistSourceId: number,
    videoSourceId: number,
  ) => void | Promise<void>;
  onStartTakeoutImport: (sourceId: number) => void | Promise<void>;
  onStartMigratedHistoryImport: (sourceId: number) => void | Promise<void>;
  onCancelSourceJob: (jobId: string) => void | Promise<void>;
};
```

This object is intentionally flat for this slice. If it becomes painful later,
a follow-up can split it into smaller internal objects with tests around the
new boundaries.

## Shell Behavior

`SourceBrowserShell` should derive:

- `sourceData` only for `subject.kind === "source"`;
- `groupData` only for `subject.kind === "source_group"`;
- `snapshotData` only for `subject.kind === "run_snapshot"`.

The shell may use safe fallbacks for missing data to avoid runtime crashes, but
tests should ensure `ReportSourceSurface` passes the correct data object for the
correct subject. Source-only UI branches should read from `sourceData`, not from
top-level props.

Group and snapshot branches should not read live source data except through
fallbacks that keep unexpected missing data from crashing. They should continue
to use `groupBrowserData` and `snapshotBrowserData`.

## Route Wiring

`ReportSourceSurface` should:

- pass `sourceBrowserData={{ ... }}` in the live single-source shell branch;
- pass `groupBrowserData={{ ... }}` in the live source-group branch;
- pass `snapshotBrowserData={{ ... }}` in the run snapshot branch;
- stop passing source-only dummy props in group and snapshot branches;
- keep `SourceReaderHeader`, Takeout recovery notice placement, and diagnostic
  placement unchanged.

## Testing Strategy

Focused tests should assert:

- `SourceBrowserShell` accepts and uses `sourceBrowserData`.
- `ReportSourceSurface` passes `sourceBrowserData={{ ... }}` for live
  single-source browsing.
- `ReportSourceSurface` keeps `groupBrowserData={{ ... }}` and
  `snapshotBrowserData={{ ... }}`.
- Group and snapshot shell invocations do not pass source-only dummy values just
  to satisfy shell props.
- `SourceBrowserShell` does not use `sourceBrowserData` in group or run snapshot
  branches except through safe fallbacks.
- `SourceBrowserShell` still imports no `$lib/api/*` modules and calls no
  `invoke`.
- Existing Telegram, YouTube video, YouTube playlist, live group, and run
  snapshot component contracts still pass.
- Existing source browser model tests remain unchanged, proving no tab/default
  or reconciliation movement.

Verification should include:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/source-browser-model.test.ts
npm.cmd run check
npm.cmd run verify
```

## Rollout

Land this as a narrow follow-up cleanup after the legacy wrapper deletion. If
implementation reveals that removing the compatibility `source` prop requires
larger route or model changes, keep the prop and finish only the
`sourceBrowserData` consolidation.
