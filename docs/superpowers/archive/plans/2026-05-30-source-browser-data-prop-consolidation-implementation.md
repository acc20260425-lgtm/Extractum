# Source Browser Data Prop Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move live single-source `SourceBrowserShell` inputs into a `sourceBrowserData` object without changing browser behavior.

**Architecture:** Keep `SourceBrowserShell` as the canonical shell and preserve the existing subject model. Only the live single-source prop shape changes; `groupBrowserData` and `snapshotBrowserData` stay named and shaped as they are. Route state and callbacks remain owned by `ReportSourceSurface`.

**Tech Stack:** Svelte 5, SvelteKit 2, TypeScript, Vitest raw component contract tests, `svelte-check`, project verification through `npm.cmd run verify`.

---

## Execution Protocol

- Start from `main`.
- Create a branch before Task 0.
- After each task, mark the completed checkboxes in this plan and commit the task.
- This is a mechanical prop-shape cleanup. Do not change tabs, defaults, reconciliation, route state ownership, backend APIs, UI layout, or copy.
- Do not rename `groupBrowserData`.
- Do not rename `snapshotBrowserData`.
- Do not split `sourceBrowserData` into nested `items`, `youtube`, `activity`, or `actions` objects.

## Files

- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
  - Add local `SourceBrowserData`.
  - Replace live single-source top-level props with `sourceBrowserData`.
  - Keep `groupBrowserData` and `snapshotBrowserData` unchanged.
  - Keep `loadingItems?: boolean` as the live group loading prop because `groupBrowserData` shape stays unchanged in this slice.
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
  - Build `sourceBrowserData={{ ... }}` only in the live single-source branch.
  - Remove source-only dummy values from group and snapshot shell invocations.
- Modify: `src/lib/components/analysis/source-browser-shell.test.ts`
  - Assert shell has grouped source data and no flattened source-only props in `Props`.
- Modify: `src/lib/analysis-source-readers.test.ts`
  - Assert route wiring uses `sourceBrowserData`, keeps group/snapshot data objects, and removes dummy source-only props.
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md`
  - Mark implemented after final verification.
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md`
  - Track task checkboxes during execution.

---

### Task 0: Preflight

**Files:**
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md`

- [x] **Step 1: Create the feature branch**

Run:

```bash
git switch -c source-browser-data-prop-consolidation
```

Expected: branch changes from `main` to `source-browser-data-prop-consolidation`.

- [x] **Step 2: Confirm current type names and callback signatures**

Run:

```bash
rg -n "onCancelSourceJob|SourceJobRecord|TakeoutImportRecoveryState|SourceForumTopic|type Props =|type SourceGroupBrowserData|type SnapshotBrowserData|explicitSubject|const subject =" src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/types/sources.ts
```

Expected current facts:

```text
SourceBrowserShell:
- imports SourceForumTopic, SourceJobRecord, TakeoutImportRecoveryState
- has local SourceGroupBrowserData
- has local SnapshotBrowserData
- has onCancelSourceJob: (jobId: string) => void | Promise<void>
- has existing subject fallback: explicitSubject ?? (source ? { kind: "source" as const, source } : null)

ReportSourceSurface:
- has onCancelSourceJob: (jobId: string) => void | Promise<void>

src/lib/types/sources.ts:
- exports SourceForumTopic
- exports SourceJobRecord
- exports TakeoutImportRecoveryState
```

Do not change any of those type names or callback signatures in this slice.

- [x] **Step 3: Confirm the current source-only dummy props in non-source invocations**

Run:

```bash
rg -n "sourceJobs=\{\[\]\}|takeoutRecovery=\{null\}|sourceItems=\{\[\]\}|liveReaderItems=\{\[\]\}|sourceTopics=\{\[\]\}|youtubeVideoDetail=\{null\}|youtubePlaylistDetail=\{null\}|sourceSyncDisabledReason=\{\(\) => null\}" src/lib/components/analysis/report-source-surface.svelte
```

Expected current facts:

```text
The run snapshot and live source-group SourceBrowserShell calls pass source-only dummy values.
These should disappear after sourceBrowserData is introduced.
```

- [x] **Step 4: Record the actual audit result**

Add this note under Task 0 while executing:

```md
Actual preflight on 2026-05-30:
- `onCancelSourceJob` uses `string`; no type change.
- Real type names are `SourceForumTopic`, `SourceJobRecord`, and `TakeoutImportRecoveryState`.
- Keep the existing `subject` derived fallback from `explicitSubject` and `source`.
- `groupBrowserData` has no loading field; keep optional top-level `loadingItems` for live group loading.
- `groupBrowserData` and `snapshotBrowserData` shapes stay unchanged.
```

Actual preflight on 2026-05-30:
- `onCancelSourceJob` uses `string`; no type change.
- Real type names are `SourceForumTopic`, `SourceJobRecord`, and `TakeoutImportRecoveryState`.
- Keep the existing `subject` derived fallback from `explicitSubject` and `source`.
- `groupBrowserData` has no loading field; keep optional top-level `loadingItems` for live group loading.
- `groupBrowserData` and `snapshotBrowserData` shapes stay unchanged.

- [x] **Step 5: Commit preflight plan state**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md
git commit -m "docs: record source browser data prop preflight"
```

Expected: commit contains only this plan file.

---

### Task 1: SourceBrowserShell Contract And Migration

**Files:**
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.test.ts`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md`

Execution note: Task 1 and Task 2 were committed as one green checkpoint because `SourceBrowserShell` prop narrowing makes existing `ReportSourceSurface` invocations fail `svelte-check` until route wiring is updated.

- [x] **Step 1: Add raw helpers to `source-browser-shell.test.ts`**

Add these helpers after the `shellSource` import:

```ts
function sourceBetween(source: string, start: string, end: string) {
  const startIndex = source.indexOf(start);
  expect(startIndex).toBeGreaterThanOrEqual(0);
  const endIndex = source.indexOf(end, startIndex);
  expect(endIndex).toBeGreaterThan(startIndex);
  return source.slice(startIndex, endIndex);
}

function sourcePropsBlock() {
  return sourceBetween(shellSource, "type Props = {", "  };");
}
```

- [x] **Step 2: Add grouped source data shell contract**

Add this test to `src/lib/components/analysis/source-browser-shell.test.ts`:

```ts
it("groups live single-source data behind sourceBrowserData", () => {
  const propsBlock = sourcePropsBlock();

  expect(shellSource).toContain("type SourceBrowserData =");
  expect(propsBlock).toContain("sourceBrowserData?: SourceBrowserData | null");
  expect(propsBlock).toContain("groupBrowserData?: SourceGroupBrowserData | null");
  expect(propsBlock).toContain("snapshotBrowserData?: SnapshotBrowserData | null");
  expect(propsBlock).toContain("loadingItems?: boolean");
  expect(propsBlock).not.toContain("sourceItems: SourceItem[]");
  expect(propsBlock).not.toContain("sourceJobs: SourceJobRecord[]");
  expect(propsBlock).not.toContain("youtubeVideoDetail: YoutubeVideoDetail | null");
  expect(propsBlock).not.toContain("onSyncYoutubeTranscript");
  expect(shellSource).toContain('subject && subject.kind === "source" ? sourceBrowserData : null');
});
```

This test expects `loadingItems` to remain as an optional top-level prop for live group loading only.

- [x] **Step 3: Run the shell contract and confirm it fails**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts
```

Expected: FAIL because `sourceBrowserData` has not been implemented yet.

- [x] **Step 4: Add local `SourceBrowserData` and simplify `Props`**

In `src/lib/components/analysis/source-browser-shell.svelte`, add `SourceBrowserData` after `SnapshotBrowserData`:

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
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onChangeTelegramHistoryScope: (scope: TelegramHistoryScope) => void;
    onChangeTranscriptSearch: (value: string) => void;
    onLoadMoreYoutubeTranscriptSegments: () => void | Promise<void>;
    onOpenSource: (sourceId: number) => void | Promise<void>;
    onSyncSource: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeMetadata: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeTranscript: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeComments: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylist: (sourceId: number) => void | Promise<void>;
    onRetryFailedYoutubePlaylistVideos: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onRetryYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onStartTakeoutImport: (sourceId: number) => void | Promise<void>;
    onStartMigratedHistoryImport: (sourceId: number) => void | Promise<void>;
    onCancelSourceJob: (jobId: string) => void | Promise<void>;
  };
```

Replace `type Props` with:

```ts
  type Props = {
    subject?: SourceBrowserSubject | null;
    source?: Source | null;
    sourceBrowserData?: SourceBrowserData | null;
    groupBrowserData?: SourceGroupBrowserData | null;
    snapshotBrowserData?: SnapshotBrowserData | null;
    selectedTraceRef?: string | null;
    loadingItems?: boolean;
    formatTimestamp: (value: number | null) => string;
  };
```

`loadingItems` remains top-level because live source groups use it and `groupBrowserData` shape is intentionally unchanged.

- [x] **Step 5: Replace `$props()` destructuring**

Replace the existing destructuring block with:

```ts
  let {
    subject: explicitSubject = null,
    source = null,
    sourceBrowserData = null,
    groupBrowserData = null,
    snapshotBrowserData = null,
    selectedTraceRef = null,
    loadingItems = false,
    formatTimestamp,
  }: Props = $props();
```

- [x] **Step 6: Preserve the existing subject fallback**

Keep the existing `subject` derived value immediately after `let activeTab` and `let lastSubjectKey`.

The required code is:

```ts
  const subject = $derived(explicitSubject ?? (source ? { kind: "source" as const, source } : null));
```

This preserves compatibility for any caller that still passes `source` instead of an explicit subject. Group and snapshot calls should pass explicit subjects and do not need `source={null}`.

- [x] **Step 7: Add subject-scoped derived data**

Replace the old `groupData` and `snapshotData` derived declarations with:

```ts
  const sourceData = $derived(subject && subject.kind === "source" ? sourceBrowserData : null);
  const groupData = $derived(subject && subject.kind === "source_group" ? groupBrowserData : null);
  const snapshotData = $derived(subject && subject.kind === "run_snapshot" ? snapshotBrowserData : null);
  const groupLoading = $derived(subject && subject.kind === "source_group" ? loadingItems : false);
```

- [x] **Step 8: Update shared derived state**

Replace `itemsForActiveSubject`, `itemsEmptyDescription`, and `sortedSourceTopics` with:

```ts
  const itemsForActiveSubject = $derived(groupData?.sourceItems ?? sourceData?.sourceItems ?? []);
  const itemsLoading = $derived(
    subject && subject.kind === "source_group" ? groupLoading : sourceData?.loadingItems ?? false,
  );
  const itemsHasMore = $derived(
    subject && subject.kind === "source_group" ? false : sourceData?.sourceItemsHasMore ?? false,
  );
  const itemsEmptyDescription = $derived(
    subject && subject.kind === "source_group"
      ? "Group items are limited to the source rows loaded in this browser session. Use Sources to load more rows for each member source."
      : sourceSubject?.sourceType === "youtube" && sourceSubject.sourceSubtype === "playlist"
        ? "Playlist videos live in the Videos tab. This Items tab only shows generic archived items loaded for this playlist source."
        : "No loaded items are available for this source window.",
  );
  const sortedSourceTopics = $derived(sourceSubject && sourceData
    ? [...sourceData.sourceTopics].sort(compareTopics)
    : []);
```

- [x] **Step 9: Update shell event helpers**

Replace `changeSelectedTopic`, `changeTelegramHistoryScope`, and add `loadMoreSourceItems`:

```ts
  function changeSelectedTopic(event: Event) {
    return sourceData?.onChangeSelectedTopicKey((event.currentTarget as HTMLSelectElement).value);
  }

  function changeTelegramHistoryScope(event: Event) {
    return sourceData?.onChangeTelegramHistoryScope(
      (event.currentTarget as HTMLSelectElement).value as TelegramHistoryScope,
    );
  }

  function loadMoreSourceItems() {
    return sourceData?.onLoadMoreSourceItems();
  }
```

Keep `loadMoreGroupItems` and `loadMoreGroupSourcePage`.

- [x] **Step 10: Update source-only branches to read from `sourceData`**

Require `sourceData` in each live source branch condition:

```svelte
{:else if activeTab === "timeline" && sourceSubject && sourceData}
```

```svelte
{:else if activeTab === "transcript" && sourceSubject && sourceData}
```

```svelte
{:else if activeTab === "videos" && sourceSubject && sourceData}
```

```svelte
{:else if activeTab === "activity" && sourceSubject && sourceData}
```

```svelte
{:else if activeTab === "comments" && sourceSubject && sourceData}
```

```svelte
{:else if activeTab === "metadata" && sourceSubject && sourceData}
```

Inside those branches, replace old flattened prop reads with `sourceData.<field>`:

```text
liveReaderItems -> sourceData.liveReaderItems
sourceItems -> sourceData.sourceItems
sourceRouteError -> sourceData.sourceRouteError
sourceItemsHasMore -> sourceData.sourceItemsHasMore
loadingItems -> sourceData.loadingItems
sourceTopics -> sourceData.sourceTopics
loadingSourceTopics -> sourceData.loadingSourceTopics
selectedTopicKey -> sourceData.selectedTopicKey
showTopicSelector -> sourceData.showTopicSelector
youtubeVideoDetail -> sourceData.youtubeVideoDetail
youtubePlaylistDetail -> sourceData.youtubePlaylistDetail
youtubeTranscriptSegments -> sourceData.youtubeTranscriptSegments
youtubeTranscriptSearch -> sourceData.youtubeTranscriptSearch
youtubeTranscriptHasMore -> sourceData.youtubeTranscriptHasMore
loadingYoutubeTranscriptSegments -> sourceData.loadingYoutubeTranscriptSegments
loadingYoutubeDetail -> sourceData.loadingYoutubeDetail
sourceJobs -> sourceData.sourceJobs
takeoutRecovery -> sourceData.takeoutRecovery
telegramHistoryScope -> sourceData.telegramHistoryScope
currentSourceContentLabel -> sourceData.currentSourceContentLabel
sourceSyncDisabledReason -> sourceData.sourceSyncDisabledReason
onSyncSource -> sourceData.onSyncSource
onLoadMoreSourceItems -> sourceData.onLoadMoreSourceItems
onChangeTranscriptSearch -> sourceData.onChangeTranscriptSearch
onLoadMoreYoutubeTranscriptSegments -> sourceData.onLoadMoreYoutubeTranscriptSegments
onOpenSource -> sourceData.onOpenSource
onSyncYoutubeMetadata -> sourceData.onSyncYoutubeMetadata
onSyncYoutubeTranscript -> sourceData.onSyncYoutubeTranscript
onSyncYoutubeComments -> sourceData.onSyncYoutubeComments
onSyncYoutubePlaylist -> sourceData.onSyncYoutubePlaylist
onRetryFailedYoutubePlaylistVideos -> sourceData.onRetryFailedYoutubePlaylistVideos
onSyncYoutubePlaylistVideo -> sourceData.onSyncYoutubePlaylistVideo
onRetryYoutubePlaylistVideo -> sourceData.onRetryYoutubePlaylistVideo
onStartTakeoutImport -> sourceData.onStartTakeoutImport
onStartMigratedHistoryImport -> sourceData.onStartMigratedHistoryImport
onCancelSourceJob -> sourceData.onCancelSourceJob
```

- [x] **Step 11: Update group branch loading and `UniversalItemsView`**

In the source group `Sources` branch, replace:

```svelte
loading={loadingItems}
```

with:

```svelte
loading={groupLoading}
```

Replace the current `UniversalItemsView` props with:

```svelte
<UniversalItemsView
  items={itemsForActiveSubject}
  loading={itemsLoading}
  hasMore={itemsHasMore}
  emptyDescription={itemsEmptyDescription}
  helpDescription={subject && subject.kind === "source_group" ? itemsEmptyDescription : null}
  sourceLabelForItem={subject && subject.kind === "source_group" ? groupData?.sourceLabelForItem ?? null : null}
  {formatTimestamp}
  onLoadMore={subject && subject.kind === "source_group" ? loadMoreGroupItems : loadMoreSourceItems}
/>
```

- [x] **Step 12: Run shell tests**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts
```

Expected: PASS.

- [x] **Step 13: Run Svelte/type checks**

Run:

```bash
npm.cmd run check
```

Expected: PASS with 0 errors.

- [x] **Step 14: Commit shell migration**

Run:

```bash
git add src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/source-browser-shell.test.ts docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md
git commit -m "refactor: group source browser shell data"
```

Expected: commit contains shell migration, shell contract tests, and plan checkbox updates.

---

### Task 2: ReportSourceSurface Wiring And Route Contracts

**Files:**
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md`

- [x] **Step 1: Add route raw helper**

Add these helpers near the existing top-level raw constants in `src/lib/analysis-source-readers.test.ts`:

```ts
function matchCount(source: string, pattern: RegExp) {
  return source.match(pattern)?.length ?? 0;
}

function sourceBetween(source: string, start: string, end: string) {
  const startIndex = source.indexOf(start);
  expect(startIndex).toBeGreaterThanOrEqual(0);
  const endIndex = source.indexOf(end, startIndex);
  expect(endIndex).toBeGreaterThan(startIndex);
  return source.slice(startIndex, endIndex + end.length);
}

function sourceBrowserShellCall(marker: string) {
  const markerIndex = reportSourceSurfaceSource.indexOf(marker);
  expect(markerIndex).toBeGreaterThanOrEqual(0);
  const openIndex = reportSourceSurfaceSource.lastIndexOf("<SourceBrowserShell", markerIndex);
  expect(openIndex).toBeGreaterThanOrEqual(0);
  return sourceBetween(reportSourceSurfaceSource.slice(openIndex), "<SourceBrowserShell", "/>");
}
```

- [x] **Step 2: Update live route contract**

In the test named `"routes live browsable sources and source groups through SourceBrowserShell"`, add these expectations:

```ts
const liveSourceShellCall = sourceBrowserShellCall('subject={{ kind: "source", source: currentSource }}');
const liveGroupShellCall = sourceBrowserShellCall('subject={{ kind: "source_group", group: currentGroup }}');

expect(reportSourceSurfaceSource).toContain("sourceBrowserData={{");
expect(reportSourceSurfaceSource).toContain("groupBrowserData={{");
expect(matchCount(reportSourceSurfaceSource, /sourceBrowserData=\{\{/g)).toBe(1);
expect(matchCount(reportSourceSurfaceSource, /groupBrowserData=\{\{/g)).toBe(1);
expect(liveSourceShellCall).toContain("sourceBrowserData={{");
expect(liveGroupShellCall).toContain("groupBrowserData={{");
expect(liveGroupShellCall).not.toContain("sourceBrowserData={{");
expect(liveGroupShellCall).not.toContain("liveReaderItems={[]}");
expect(liveGroupShellCall).not.toContain("sourceItems={[]}");
expect(liveGroupShellCall).not.toContain("sourceJobs={[]}");
expect(liveGroupShellCall).not.toContain("sourceTopics={[]}");
expect(liveGroupShellCall).not.toContain("youtubeVideoDetail={null}");
expect(liveGroupShellCall).not.toContain("youtubePlaylistDetail={null}");
expect(liveGroupShellCall).not.toContain("sourceSyncDisabledReason={() => null}");
```

- [x] **Step 3: Update run snapshot route contract**

In the test named `"routes available run snapshots through SourceBrowserShell while keeping the header route-owned"`, add:

```ts
const snapshotShellCall = sourceBrowserShellCall("subject={runSnapshotSubject}");

expect(reportSourceSurfaceSource).toContain("snapshotBrowserData={{");
expect(matchCount(reportSourceSurfaceSource, /snapshotBrowserData=\{\{/g)).toBe(1);
expect(snapshotShellCall).toContain("snapshotBrowserData={{");
expect(snapshotShellCall).not.toContain("sourceBrowserData={{");
```

Replace the body of the test named `"keeps snapshot shell data frozen-only and live props empty"` with:

```ts
const snapshotShellCall = sourceBrowserShellCall("subject={runSnapshotSubject}");

expect(reportSourceSurfaceSource).toContain("deriveRunSnapshotBrowserKind");
expect(reportSourceSurfaceSource).toContain("allSnapshotReaderItems");
expect(snapshotShellCall).toContain("snapshotBrowserData={{");
expect(snapshotShellCall).not.toContain("sourceJobs={[]}");
expect(snapshotShellCall).not.toContain("takeoutRecovery={null}");
expect(snapshotShellCall).not.toContain("sourceSyncDisabledReason={() => null}");
expect(snapshotShellCall).not.toContain("liveReaderItems={[]}");
expect(snapshotShellCall).not.toContain("sourceItems={[]}");
expect(snapshotShellCall).not.toContain("sourceTopics={[]}");
expect(snapshotShellCall).not.toContain("youtubeVideoDetail={null}");
expect(snapshotShellCall).not.toContain("youtubePlaylistDetail={null}");
```

- [x] **Step 4: Run route contracts and confirm they fail**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because `ReportSourceSurface` has not been wired to `sourceBrowserData` yet.

Actual execution note: this isolated red run was skipped after Task 1 `npm.cmd run check` proved the shell prop change cannot be checkpointed before route wiring. The route contracts were verified after Step 9 with the focused test command.

- [x] **Step 5: Update run snapshot shell invocation**

In `src/lib/components/analysis/report-source-surface.svelte`, reduce the run snapshot shell call to:

```svelte
<SourceBrowserShell
  subject={runSnapshotSubject}
  snapshotBrowserData={{
    run: currentRun,
    readerItems: snapshotReaderItems,
    selectedSourceId: selectedSnapshotSourceId,
    sourceOptions: snapshotSourceOptions,
    loading: loadingRunSnapshotMessages,
    hasMore: hasMoreRunSnapshotMessages,
    availability: snapshotAvailability,
    error: runSnapshotError,
    selectedTraceRef,
    onLoadMore: onLoadMoreRunSnapshotMessages,
  }}
  {selectedTraceRef}
  {formatTimestamp}
/>
```

Do not pass source-only dummy values such as `sourceItems={[]}`, `sourceJobs={[]}`, `youtubeVideoDetail={null}`, or `sourceSyncDisabledReason={() => null}`.

- [x] **Step 6: Update live single-source shell invocation**

In the live single-source branch, pass `sourceBrowserData={{ ... }}`:

```svelte
<SourceBrowserShell
  subject={{ kind: "source", source: currentSource }}
  source={currentSource}
  sourceBrowserData={{
    liveReaderItems,
    takeoutRecovery,
    sourceItems,
    sourceRouteError: sourceItemsError,
    sourceItemsHasMore,
    loadingItems,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    youtubeVideoDetail,
    youtubePlaylistDetail,
    youtubeTranscriptSegments,
    youtubeTranscriptSearch,
    youtubeTranscriptHasMore,
    loadingYoutubeTranscriptSegments,
    loadingYoutubeDetail,
    sourceJobs,
    telegramHistoryScope,
    currentSourceContentLabel,
    sourceSyncDisabledReason,
    onSyncSource,
    onLoadMoreSourceItems,
    onChangeSelectedTopicKey,
    onChangeTelegramHistoryScope,
    onChangeTranscriptSearch,
    onLoadMoreYoutubeTranscriptSegments,
    onOpenSource,
    onSyncYoutubeMetadata,
    onSyncYoutubeTranscript,
    onSyncYoutubeComments,
    onSyncYoutubePlaylist,
    onRetryFailedYoutubePlaylistVideos,
    onSyncYoutubePlaylistVideo,
    onRetryYoutubePlaylistVideo,
    onStartTakeoutImport,
    onStartMigratedHistoryImport,
    onCancelSourceJob,
  }}
  {selectedTraceRef}
  {formatTimestamp}
/>
```

- [x] **Step 7: Update live source-group shell invocation**

In the live source-group branch, keep `groupBrowserData` unchanged and pass only group-loading/shared props:

```svelte
<SourceBrowserShell
  subject={{ kind: "source_group", group: currentGroup }}
  loadingItems={loadingItems}
  {selectedTraceRef}
  {formatTimestamp}
  groupBrowserData={{
    liveReaderItems: groupLiveReaderItems,
    sourceItems: groupLiveSourceItems,
    selectedSourceId: selectedGroupSourceId,
    hasMoreBySource: groupLiveHasMoreBySource,
    sourceLabelForItem: sourceLabelForGroupItem,
    onLoadSourcePage: onLoadLiveGroupSourcePage,
    youtubeDetailsBySource: {},
  }}
/>
```

Do not pass source-only dummy values or source-only callbacks in this branch.

- [x] **Step 8: Verify dummy source props are gone**

Run:

```bash
rg -n "sourceJobs=\{\[\]\}|takeoutRecovery=\{null\}|sourceItems=\{\[\]\}|liveReaderItems=\{\[\]\}|sourceTopics=\{\[\]\}|youtubeVideoDetail=\{null\}|youtubePlaylistDetail=\{null\}|sourceSyncDisabledReason=\{\(\) => null\}" src/lib/components/analysis/report-source-surface.svelte
```

Expected: no output. `rg` exits with code `1` when no matches are found.

- [x] **Step 9: Run focused route/shell tests**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/source-browser-model.test.ts
```

Expected: PASS.

- [x] **Step 10: Run Svelte/type checks**

Run:

```bash
npm.cmd run check
```

Expected: PASS with 0 errors.

- [x] **Step 11: Commit route wiring**

Run:

```bash
git add src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md
git commit -m "refactor: pass grouped source browser data"
```

Expected: commit contains route wiring, route contract tests, and plan checkbox updates.

---

### Task 3: Final Verification And Spec Status

**Files:**
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md`

- [x] **Step 1: Verify canonical shell invocation counts**

Run:

```bash
rg -n "sourceBrowserData=\{\{|groupBrowserData=\{\{|snapshotBrowserData=\{\{" src/lib/components/analysis/report-source-surface.svelte
```

Expected:

```text
Exactly one sourceBrowserData={{ match.
Exactly one groupBrowserData={{ match.
Exactly one snapshotBrowserData={{ match.
```

- [x] **Step 2: Verify no source-only dummy props remain in report surface**

Run:

```bash
rg -n "sourceJobs=\{\[\]\}|takeoutRecovery=\{null\}|sourceItems=\{\[\]\}|liveReaderItems=\{\[\]\}|sourceTopics=\{\[\]\}|youtubeVideoDetail=\{null\}|youtubePlaylistDetail=\{null\}|sourceSyncDisabledReason=\{\(\) => null\}" src/lib/components/analysis/report-source-surface.svelte
```

Expected: no output. `rg` exits with code `1` when no matches are found.

- [x] **Step 3: Run focused tests**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/source-browser-model.test.ts
```

Expected: PASS.

- [x] **Step 4: Run Svelte/type checks**

Run:

```bash
npm.cmd run check
```

Expected: PASS with 0 errors.

- [x] **Step 5: Run full verification**

Run:

```bash
npm.cmd run verify
```

Expected: PASS, including frontend tests, Svelte checks, Rust checks/tests, and `git diff HEAD --check`.

- [x] **Step 6: Mark design spec implemented**

In `docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md`, replace:

```md
> Status: approved design, pending implementation plan
```

with:

```md
> Status: implemented on 2026-05-30; pending merge
```

- [x] **Step 7: Check whitespace**

Run:

```bash
git diff --check
```

Expected: no output.

- [x] **Step 8: Commit verification status**

Run:

```bash
git add docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md
git commit -m "docs: mark source browser data prop consolidation verified"
```

Expected: commit includes only spec status and final plan checkbox updates.

---

## Acceptance Checklist

- [x] `SourceBrowserShell` has `sourceBrowserData?: SourceBrowserData | null`.
- [x] Live single-source branches in `SourceBrowserShell` read from `sourceData`.
- [x] `ReportSourceSurface` passes `sourceBrowserData={{ ... }}` exactly for live single-source browsing.
- [x] `ReportSourceSurface` keeps `groupBrowserData={{ ... }}` for live source groups.
- [x] `ReportSourceSurface` keeps `snapshotBrowserData={{ ... }}` for run snapshots.
- [x] `ReportSourceSurface` no longer passes source-only dummy values to group/snapshot shell calls.
- [x] `groupBrowserData` name and shape are unchanged.
- [x] `snapshotBrowserData` name and shape are unchanged.
- [x] `loadingItems` remains top-level only for live group loading compatibility.
- [x] Source browser model tests are unchanged and passing.
- [x] `SourceBrowserShell` still imports no `$lib/api/*` modules and calls no `invoke`.
- [x] `npm.cmd run verify` passes.
