# Source Group Source Browser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move live source groups into the shared `SourceBrowserShell` with a group-specific `Sources` tab while leaving saved snapshots on their existing readers.

**Architecture:** Introduce a subject-aware source browser model, then route live groups through the existing shell using route-owned group data and callbacks. Keep group rendering in focused leaves: `SourceGroupSourcesView`, optional source labels in `UniversalItemsView`, `SourceGroupMetadataView`, and a lightweight `SourceGroupActivityView`.

**Tech Stack:** Svelte 5 runes, SvelteKit, TypeScript, Vitest raw-component contract tests, existing Tauri analysis fixtures for manual smoke.

---

## Files

- Modify: `src/lib/source-browser-model.ts`
  - Add `sources` tab id and subject-aware browser model.
  - Keep source-only helper wrappers behavior-compatible.
- Modify: `src/lib/source-browser-model.test.ts`
  - Add `AnalysisSourceGroup` fixture and subject/reconciliation tests.
- Create: `src/lib/components/analysis/source-group-sources-view.svelte`
  - New group `Sources` tab leaf, extracted from existing grouped reader behavior.
- Modify: `src/lib/components/analysis/source-group-reader.svelte`
  - Compatibility wrapper for snapshot/legacy paths.
- Modify: `src/lib/components/analysis/universal-items-view.svelte`
  - Add optional source-label hook for group `Items`.
- Create: `src/lib/components/analysis/source-group-metadata-view.svelte`
  - Group metadata leaf.
- Create: `src/lib/components/analysis/source-group-activity-view.svelte`
  - Lightweight group activity empty/status leaf.
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
  - Accept `SourceBrowserSubject`, render group tabs, preserve single-source behavior.
- Modify: `src/lib/components/analysis/source-browser-shell.test.ts`
  - Update shell contract tests for subjects and group leaves.
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
  - Route live source groups into `SourceBrowserShell`; keep snapshots outside shell.
- Modify: `src/lib/analysis-source-readers.test.ts`
  - Add raw contract tests for group shell routing, group leaf invariants, item attribution, and snapshot exclusion.
- Modify: `src/lib/analysis-report-canvas.test.ts`
  - Update source surface contract expectations for live groups through the shell while snapshot group reader expectations remain.
- Modify: `docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md`
  - Mark implemented after verification.

## Execution Notes

When executing this plan, start on a feature branch from clean `main`:

```bash
git checkout -b source-group-source-browser
```

After each task, mark completed steps in this file, then commit the task. Do not migrate saved run snapshots or saved group snapshots into `SourceBrowserShell`.

Commands in this plan use `npm.cmd` because the current execution environment is Windows. Use `npm run ...` instead of `npm.cmd run ...` if executing the same plan from macOS/Linux.

## Task 0: Type And Behavior Preflight

**Files:**
- Validate: `src/lib/types/analysis.ts`
- Validate: `src/lib/types/sources.ts`
- Validate: `src/lib/source-reader-model.ts`
- Validate: `src/lib/components/analysis/source-group-reader.svelte`
- Validate: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `docs/superpowers/plans/2026-05-30-source-group-source-browser-implementation.md` after marking checklist steps during execution

- [x] **Step 1: Confirm group and item DTO shapes**

Run:

```bash
rg -n "export interface AnalysisSourceGroup|export interface SourceItem|export interface SourceReaderItem|groupLiveItemsBySource" src/lib/types/analysis.ts src/lib/types/sources.ts src/lib/source-reader-model.ts src/routes/analysis/+page.svelte src/lib/components/analysis/report-source-surface.svelte
```

Expected:

```text
src/lib/types/analysis.ts:32:export interface AnalysisSourceGroup {
src/lib/types/sources.ts:75:export interface SourceItem {
src/lib/source-reader-model.ts:20:export interface SourceReaderItem {
src/routes/analysis/+page.svelte:<line>:  let groupLiveItemsBySource = $state<Record<number, SourceItem[]>>({});
```

Confirm these facts before continuing:

- `AnalysisSourceGroup` already exposes `id`, `name`, `source_type`, `members`, `created_at`, and `updated_at`; no backend or DTO expansion is needed for group metadata.
- `groupLiveItemsBySource` is `Record<number, SourceItem[]>`, so the group `Items` tab can feed `UniversalItemsView` directly from route-owned `SourceItem[]`.
- `SourceReaderItem` is a separate display model and must remain the input for the group `Sources` leaf.

- [x] **Step 2: Confirm evidence and YouTube detail behavior in the existing group reader path**

Run:

```bash
rg -n "selectedTraceRef|youtubeDetailsBySource|SourceGroupReader|groupLiveReaderItems|sourceItemToReaderItem" src/lib/components/analysis/source-group-reader.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/source-reader-model.ts
```

Expected:

```text
src/lib/components/analysis/source-group-reader.svelte:<line>:            selectedTraceRef={null}
src/lib/components/analysis/report-source-surface.svelte:<line>:  const groupLiveReaderItems = $derived.by(() =>
src/lib/components/analysis/report-source-surface.svelte:<line>:      return items.map((item) => sourceItemToReaderItem(item, { sourceTitle, selectedTraceRef }));
src/lib/components/analysis/report-source-surface.svelte:<line>:          youtubeDetailsBySource={{}}
src/lib/components/analysis/report-source-surface.svelte:<line>:      youtubeDetailsBySource={{}}
```

Confirm these facts before continuing:

- Existing live group reader items already preserve evidence selection through `sourceItemToReaderItem(..., selectedTraceRef)`.
- `SourceGroupReader` currently drops YouTube transcript `selectedTraceRef` by passing `null`; the new `SourceGroupSourcesView` must accept `selectedTraceRef` and pass it to `YoutubeTranscriptReader`.
- Current group routes pass `youtubeDetailsBySource={{}}`; keep that route-owned default in this slice unless a real route-owned detail map is already present.

- [x] **Step 3: Confirm snapshot chrome is not being removed accidentally**

Run:

```bash
rg -n "Source focus|sourceViewBasis === \"run_snapshot\"|<SourceGroupReader|source-group-reader" src/lib/components/analysis/source-group-reader.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts
```

Expected:

```text
src/lib/components/analysis/report-source-surface.svelte:<line>:  {#if sourceViewBasis === "run_snapshot" ...}
src/lib/components/analysis/report-source-surface.svelte:<line>:        <SourceGroupReader
src/lib/components/analysis/report-source-surface.svelte:<line>:    <SourceGroupReader
```

If this command prints `Source focus` inside `source-group-reader.svelte`, stop and preserve that chrome in `SourceGroupReader` instead of replacing the file with the simple wrapper in Task 2. In the current codebase, `Source focus` should not be part of `SourceGroupReader`, so the wrapper extraction is safe.

- [x] **Step 4: Record implementation decisions from the preflight**

Use these decisions for the rest of the plan:

- `SourceBrowserSubject` passed to `SourceBrowserShell` uses full `Source` and full `AnalysisSourceGroup` objects.
- The model helpers may use internal narrow `Pick<Source, "sourceType" | "sourceSubtype">` inputs where they only need source type/subtype.
- Group `Sources` uses `SourceReaderItem[]`.
- Group `Items` uses `SourceItem[]` from `groupLiveItemsBySource`, with no `SourceReaderItem -> SourceItem` mapper.
- `SourceGroupSourcesView` accepts `selectedTraceRef: string | null`.
- `SourceGroupReader` remains the snapshot/legacy compatibility component.
- Group YouTube detail data remains `youtubeDetailsBySource={{}}` in this slice unless the route already owns a real detail map.

- [x] **Step 5: Confirm the working tree is ready**

Run:

```bash
git status --short
```

Expected: no unrelated working tree changes. If only this plan file changes because checklist boxes were marked, continue.

- [x] **Step 6: Commit preflight checklist**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-source-group-source-browser-implementation.md
git commit -m "docs: record source group browser preflight"
```

## Task 1: Add Subject-Aware Browser Model

**Files:**
- Modify: `src/lib/source-browser-model.test.ts`
- Modify: `src/lib/source-browser-model.ts`

- [ ] **Step 1: Add group fixture and subject imports to model tests**

In `src/lib/source-browser-model.test.ts`, update imports:

```ts
import {
  commentsCoverageState,
  filterLoadedSourceItems,
  filterLoadedYoutubeComments,
  groupLoadedYoutubeComments,
  reconcileSourceBrowserTab,
  sortLoadedYoutubeComments,
  sourceBrowserShellAppliesToSource,
  sourceBrowserShellAppliesToSubject,
  sourceBrowserTabsForSubject,
  sourceItemKindChips,
  sourceBrowserTabsForSource,
  smartDefaultSourceBrowserTab,
  sortLoadedSourceItems,
  type SourceBrowserTabId,
} from "./source-browser-model";
import type { AnalysisSourceGroup } from "./types/analysis";
```

Add this fixture below `source(...)`:

```ts
function sourceGroup(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 100,
    name: "Research group",
    source_type: "telegram",
    members: [
      { source_id: 1, source_title: "Alpha", item_count: 12 },
      { source_id: 2, source_title: "Beta", item_count: 7 },
    ],
    created_at: 1710000000,
    updated_at: 1710000500,
    ...overrides,
  };
}
```

- [ ] **Step 2: Add failing subject model tests**

Add these tests inside `describe("source browser model", () => { ... })`, after the existing canonical tabs test:

```ts
  it("derives canonical tabs for live source group subjects", () => {
    const groupSubject = { kind: "source_group" as const, group: sourceGroup() };

    expect(sourceBrowserTabsForSubject(groupSubject).map((tab) => tab.id))
      .toEqual(["sources", "items", "metadata", "activity"]);
    expect(smartDefaultSourceBrowserTab(groupSubject)).toBe("sources");
    expect(sourceBrowserShellAppliesToSubject(groupSubject)).toBe(true);
  });

  it("keeps source helper behavior aligned with subject-aware helpers", () => {
    const samples = [
      source({ sourceType: "telegram", sourceSubtype: "supergroup" }),
      source({ sourceType: "youtube", sourceSubtype: "video" }),
      source({ sourceType: "youtube", sourceSubtype: "playlist" }),
      source({ sourceType: "rss", sourceSubtype: "feed" }),
    ];

    for (const candidate of samples) {
      const subject = { kind: "source" as const, source: candidate };
      expect(sourceBrowserTabsForSource(candidate)).toEqual(sourceBrowserTabsForSubject(subject));
      expect(smartDefaultSourceBrowserTab(candidate)).toBe(smartDefaultSourceBrowserTab(subject));
      expect(sourceBrowserShellAppliesToSource(candidate)).toBe(sourceBrowserShellAppliesToSubject(subject));
    }
  });

  it("reconciles source group tab transitions by subject support", () => {
    const groupSubject = { kind: "source_group" as const, group: sourceGroup() };
    const nextGroupSubject = {
      kind: "source_group" as const,
      group: sourceGroup({ id: 101, name: "Next group" }),
    };
    const telegramSubject = {
      kind: "source" as const,
      source: source({ id: 3, sourceType: "telegram", sourceSubtype: "supergroup" }),
    };
    const youtubeVideoSubject = {
      kind: "source" as const,
      source: source({ id: 4, sourceType: "youtube", sourceSubtype: "video" }),
    };
    const youtubePlaylistSubject = {
      kind: "source" as const,
      source: source({ id: 5, sourceType: "youtube", sourceSubtype: "playlist" }),
    };

    expect(reconcileSourceBrowserTab("items", groupSubject)).toBe("items");
    expect(reconcileSourceBrowserTab("metadata", groupSubject)).toBe("metadata");
    expect(reconcileSourceBrowserTab("activity", groupSubject)).toBe("activity");
    expect(reconcileSourceBrowserTab("timeline", groupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("transcript", groupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("comments", groupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("videos", groupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("sources", telegramSubject)).toBe("timeline");
    expect(reconcileSourceBrowserTab("sources", youtubeVideoSubject)).toBe("transcript");
    expect(reconcileSourceBrowserTab("sources", youtubePlaylistSubject)).toBe("videos");
    expect(reconcileSourceBrowserTab("sources", nextGroupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("items", nextGroupSubject)).toBe("items");
    expect(reconcileSourceBrowserTab("metadata", nextGroupSubject)).toBe("metadata");
    expect(reconcileSourceBrowserTab("activity", nextGroupSubject)).toBe("activity");
  });
```

- [ ] **Step 3: Run model tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/source-browser-model.test.ts
```

Expected: FAIL because `sourceBrowserTabsForSubject`, `sourceBrowserShellAppliesToSubject`, and tab id `sources` are not implemented yet.

- [ ] **Step 4: Implement subject-aware browser model**

In `src/lib/source-browser-model.ts`, add the group type import:

```ts
import type { AnalysisSourceGroup } from "$lib/types/analysis";
```

Extend the tab id union:

```ts
export type SourceBrowserTabId =
  | "timeline"
  | "transcript"
  | "comments"
  | "videos"
  | "sources"
  | "items"
  | "metadata"
  | "activity";
```

Add the subject type after `SourceBrowserTab`:

```ts
export type SourceBrowserSubject =
  | { kind: "source"; source: Source }
  | { kind: "source_group"; group: AnalysisSourceGroup };

type SourceBrowserSourceLike = Pick<Source, "sourceType" | "sourceSubtype">;
type SourceBrowserModelInput = SourceBrowserSubject | SourceBrowserSourceLike;
```

Add the `Sources` label:

```ts
const TAB_LABELS: Record<SourceBrowserTabId, string> = {
  timeline: "Timeline",
  transcript: "Transcript",
  comments: "Comments",
  videos: "Videos",
  sources: "Sources",
  items: "Items",
  metadata: "Metadata",
  activity: "Activity",
};
```

Replace the current tab/default/applicability helpers with subject-aware helpers and wrappers:

```ts
function isSourceBrowserSubject(input: SourceBrowserModelInput): input is SourceBrowserSubject {
  return "kind" in input && (input.kind === "source" || input.kind === "source_group");
}

function tabRecords(ids: SourceBrowserTabId[]): SourceBrowserTab[] {
  return ids.map((id) => ({ id, label: TAB_LABELS[id] }));
}

function sourceTabIds(source: SourceBrowserSourceLike): SourceBrowserTabId[] {
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") {
    return ["transcript", "comments", "items", "metadata", "activity"];
  }
  if (source.sourceType === "youtube" && source.sourceSubtype === "playlist") {
    return ["videos", "items", "metadata", "activity"];
  }
  if (source.sourceType === "telegram") {
    return ["timeline", "items", "metadata", "activity"];
  }
  return ["items", "metadata", "activity"];
}

export function sourceBrowserTabsForSubject(subject: SourceBrowserSubject): SourceBrowserTab[] {
  const ids: SourceBrowserTabId[] = subject.kind === "source_group"
    ? ["sources", "items", "metadata", "activity"]
    : sourceTabIds(subject.source);

  return tabRecords(ids);
}

export function sourceBrowserTabsForSource(source: SourceBrowserSourceLike): SourceBrowserTab[] {
  return tabRecords(sourceTabIds(source));
}

export function sourceBrowserShellAppliesToSubject(subject: SourceBrowserSubject): boolean {
  if (subject.kind === "source_group") return true;
  return sourceBrowserShellAppliesToSource(subject.source);
}

export function sourceBrowserShellAppliesToSource(source: SourceBrowserSourceLike): boolean {
  return source.sourceType === "telegram"
    || (source.sourceType === "youtube" && (source.sourceSubtype === "video" || source.sourceSubtype === "playlist"));
}

export function smartDefaultSourceBrowserTab(input: SourceBrowserModelInput): SourceBrowserTabId {
  if (isSourceBrowserSubject(input) && input.kind === "source_group") return "sources";
  const source = isSourceBrowserSubject(input) ? input.source : input;
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") return "transcript";
  if (source.sourceType === "youtube" && source.sourceSubtype === "playlist") return "videos";
  if (source.sourceType === "telegram") return "timeline";
  return "items";
}

export function reconcileSourceBrowserTab(
  activeTab: SourceBrowserTabId | null,
  input: SourceBrowserModelInput,
): SourceBrowserTabId {
  const tabs = isSourceBrowserSubject(input)
    ? sourceBrowserTabsForSubject(input)
    : sourceBrowserTabsForSource(input);
  return activeTab && tabs.some((tab) => tab.id === activeTab)
    ? activeTab
    : smartDefaultSourceBrowserTab(input);
}
```

- [ ] **Step 5: Run model tests and verify they pass**

Run:

```bash
npm.cmd run test -- src/lib/source-browser-model.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit model task**

Run:

```bash
git add src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts
git commit -m "feat: add source group browser subject model"
```

## Task 2: Extract the Group Sources Leaf

**Files:**
- Create: `src/lib/components/analysis/source-group-sources-view.svelte`
- Modify: `src/lib/components/analysis/source-group-reader.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Add raw contract tests for the extracted group leaf**

In `src/lib/analysis-source-readers.test.ts`, add this import near the other raw component imports:

```ts
import sourceGroupSourcesViewSource from "./components/analysis/source-group-sources-view.svelte?raw";
```

Add this test near the existing source-group reader tests:

```ts
  it("renders source group sources as a route-free tab leaf", () => {
    expect(sourceGroupSourcesViewSource).toContain('aria-label="Source group sources"');
    expect(sourceGroupSourcesViewSource).toContain("groupReaderItemsBySource");
    expect(sourceGroupSourcesViewSource).toContain("onLoadMoreSource");
    expect(sourceGroupSourcesViewSource).toContain("selectedGroupSourceId");
    expect(sourceGroupSourcesViewSource).toContain("selectedTraceRef");
    expect(sourceGroupSourcesViewSource).toContain("youtubeItems");
    expect(sourceGroupSourcesViewSource).toContain("telegramItems");
    expect(sourceGroupSourcesViewSource).not.toContain("$lib/api/");
    expect(sourceGroupSourcesViewSource).not.toContain("invoke(");
    expect(sourceGroupSourcesViewSource).not.toContain("SourceBrowserShell");
    expect(sourceGroupSourcesViewSource).not.toContain("SourceActivityView");
    expect(sourceGroupSourcesViewSource).not.toContain("<span>Source focus</span>");
  });

  it("keeps SourceGroupReader as a compatibility wrapper", () => {
    expect(sourceGroupReaderSource).toContain("<SourceGroupSourcesView");
    expect(sourceGroupReaderSource).not.toContain("$lib/api/");
    expect(sourceGroupReaderSource).not.toContain("invoke(");
  });
```

- [ ] **Step 2: Run reader tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because `source-group-sources-view.svelte` does not exist and `SourceGroupReader` is not yet a wrapper.

- [ ] **Step 3: Create `SourceGroupSourcesView`**

Create `src/lib/components/analysis/source-group-sources-view.svelte` with this content:

```svelte
<script lang="ts">
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import { groupReaderItemsBySource, type SourceReaderItem } from "$lib/source-reader-model";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    items,
    selectedGroupSourceId,
    loading,
    hasMoreBySource = {},
    hasMoreAll = false,
    loadMoreAllLabel = "Load more source material",
    youtubeDetailsBySource,
    selectedTraceRef = null,
    formatTimestamp,
    onLoadMoreSource,
    onLoadMoreAll = () => {},
  }: {
    items: SourceReaderItem[];
    selectedGroupSourceId: number | null;
    loading: boolean;
    hasMoreBySource?: Record<number, boolean>;
    hasMoreAll?: boolean;
    loadMoreAllLabel?: string;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
    selectedTraceRef?: string | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreSource: (sourceId: number) => void | Promise<void>;
    onLoadMoreAll?: () => void | Promise<void>;
  } = $props();

  const sourceGroups = $derived(
    groupReaderItemsBySource(
      selectedGroupSourceId === null
        ? items
        : items.filter((item) => item.sourceId === selectedGroupSourceId),
    ),
  );
</script>

<section class="source-group-sources-view" aria-label="Source group sources">
  {#if !loading && sourceGroups.length === 0}
    <EmptyState description="No source material is loaded for this group view." />
  {:else}
    {#each sourceGroups as group (group.sourceId)}
      {@const youtubeItems = group.items.filter((item) => item.kind === "youtube_transcript")}
      {@const telegramItems = group.items.filter((item) => item.kind !== "youtube_transcript")}
      <section class="source-bucket" aria-label={group.sourceTitle}>
        <div class="source-heading">
          <h3>{group.sourceTitle}</h3>
          <span>{group.items.length} loaded items</span>
        </div>

        {#if youtubeItems.length > 0}
          <YoutubeTranscriptReader
            detail={youtubeDetailsBySource[group.sourceId] ?? null}
            segments={[]}
            snapshotItems={youtubeItems}
            {loading}
            hasMore={hasMoreBySource[group.sourceId] ?? false}
            transcriptSearch=""
            showSyncActions={false}
            sourceTitle={group.sourceTitle}
            {selectedTraceRef}
            {formatTimestamp}
            onChangeTranscriptSearch={() => {}}
            onLoadMore={() => onLoadMoreSource(group.sourceId)}
            onSyncTranscript={() => {}}
            onSyncMetadata={() => {}}
          />
        {/if}

        {#if telegramItems.length > 0}
          <TelegramTimelineReader
            items={telegramItems}
            {loading}
            hasMore={hasMoreBySource[group.sourceId] ?? false}
            ariaLabel="Source material timeline"
            {formatTimestamp}
            onLoadMore={() => onLoadMoreSource(group.sourceId)}
          />
        {/if}
      </section>
    {/each}

    {#if hasMoreAll}
      <div class="source-group-footer">
        <Button
          type="button"
          variant="secondary"
          disabled={loading}
          onclick={onLoadMoreAll}
        >
          {loading ? "Loading..." : loadMoreAllLabel}
        </Button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .source-group-sources-view,
  .source-bucket {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .source-bucket {
    padding-top: 0.8rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
  }

  .source-heading {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .source-heading h3,
  .source-heading span {
    margin: 0;
  }

  .source-heading span {
    color: var(--muted);
    font-size: 0.82rem;
  }

  .source-group-footer {
    display: flex;
    justify-content: center;
  }

  @media (max-width: 760px) {
    .source-heading {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
```

- [ ] **Step 4: Preserve `SourceGroupReader` as a compatibility wrapper**

Task 0 should have confirmed that `SourceGroupReader` does not own snapshot-only chrome such as `Source focus`. Keep `SourceGroupReader` as the snapshot/legacy entry point and replace its inner grouped-reader body with this wrapper:

```svelte
<script lang="ts">
  import SourceGroupSourcesView from "$lib/components/analysis/source-group-sources-view.svelte";
  import type { SourceReaderItem } from "$lib/source-reader-model";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    items,
    selectedGroupSourceId,
    loading,
    hasMoreBySource = {},
    hasMoreAll = false,
    loadMoreAllLabel = "Load more source material",
    youtubeDetailsBySource,
    selectedTraceRef = null,
    formatTimestamp,
    onLoadMoreSource,
    onLoadMoreAll = () => {},
  }: {
    items: SourceReaderItem[];
    selectedGroupSourceId: number | null;
    loading: boolean;
    hasMoreBySource?: Record<number, boolean>;
    hasMoreAll?: boolean;
    loadMoreAllLabel?: string;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
    selectedTraceRef?: string | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreSource: (sourceId: number) => void | Promise<void>;
    onLoadMoreAll?: () => void | Promise<void>;
  } = $props();
</script>

<SourceGroupSourcesView
  {items}
  {selectedGroupSourceId}
  {loading}
  {hasMoreBySource}
  {hasMoreAll}
  {loadMoreAllLabel}
  {youtubeDetailsBySource}
  {selectedTraceRef}
  {formatTimestamp}
  {onLoadMoreSource}
  {onLoadMoreAll}
/>
```

- [ ] **Step 5: Run reader tests and verify they pass**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit group sources leaf task**

Run:

```bash
git add src/lib/components/analysis/source-group-sources-view.svelte src/lib/components/analysis/source-group-reader.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: extract source group sources view"
```

## Task 3: Add Source Attribution To Universal Items

**Files:**
- Modify: `src/lib/components/analysis/universal-items-view.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Add raw contract test for optional source labels**

In the existing `"renders universal Items as a loaded-window browser"` test in `src/lib/analysis-source-readers.test.ts`, add:

```ts
    expect(universalItemsViewSource).toContain("sourceLabelForItem");
    expect(universalItemsViewSource).toContain("Source #${item.sourceId}");
```

- [ ] **Step 2: Run reader tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because `UniversalItemsView` does not accept `sourceLabelForItem`.

- [ ] **Step 3: Add optional `sourceLabelForItem` prop**

In `src/lib/components/analysis/universal-items-view.svelte`, update the props destructuring:

```ts
  let {
    items,
    loading,
    hasMore,
    emptyDescription = "No loaded items are available for this source window.",
    sourceLabelForItem = null,
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceItem[];
    loading: boolean;
    hasMore: boolean;
    emptyDescription?: string;
    sourceLabelForItem?: ((item: SourceItem) => string | null) | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();
```

Inside the item loop, replace:

```svelte
              <Badge variant="neutral">Source #{item.sourceId}</Badge>
```

with:

```svelte
              {@const sourceLabel = sourceLabelForItem?.(item) ?? `Source #${item.sourceId}`}
              <Badge variant="neutral">{sourceLabel}</Badge>
```

- [ ] **Step 4: Run reader tests and verify they pass**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit universal items task**

Run:

```bash
git add src/lib/components/analysis/universal-items-view.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: add source labels to universal items"
```

## Task 4: Add Group Metadata And Activity Leaves

**Files:**
- Create: `src/lib/components/analysis/source-group-metadata-view.svelte`
- Create: `src/lib/components/analysis/source-group-activity-view.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Add raw imports and tests for group metadata/activity leaves**

In `src/lib/analysis-source-readers.test.ts`, add imports:

```ts
import sourceGroupActivityViewSource from "./components/analysis/source-group-activity-view.svelte?raw";
import sourceGroupMetadataViewSource from "./components/analysis/source-group-metadata-view.svelte?raw";
```

Add these tests near the source-group tests:

```ts
  it("renders source group metadata from route-owned group fields", () => {
    expect(sourceGroupMetadataViewSource).toContain('aria-label="Source group metadata"');
    expect(sourceGroupMetadataViewSource).toContain("group.name");
    expect(sourceGroupMetadataViewSource).toContain("group.source_type");
    expect(sourceGroupMetadataViewSource).toContain("group.members.length");
    expect(sourceGroupMetadataViewSource).toContain("member.item_count");
    expect(sourceGroupMetadataViewSource).toContain("formatTimestamp(group.created_at)");
    expect(sourceGroupMetadataViewSource).toContain("formatTimestamp(group.updated_at)");
    expect(sourceGroupMetadataViewSource).not.toContain("$lib/api/");
    expect(sourceGroupMetadataViewSource).not.toContain("invoke(");
  });

  it("renders source group activity without source job cards", () => {
    expect(sourceGroupActivityViewSource).toContain('aria-label="Source group activity"');
    expect(sourceGroupActivityViewSource).toContain("Group activity is not available yet. Source jobs are still tracked per source.");
    expect(sourceGroupActivityViewSource).not.toContain("SourceActivityView");
    expect(sourceGroupActivityViewSource).not.toContain("SourceJobRecord");
    expect(sourceGroupActivityViewSource).not.toContain("onCancelSourceJob");
    expect(sourceGroupActivityViewSource).not.toContain("$lib/api/");
    expect(sourceGroupActivityViewSource).not.toContain("invoke(");
  });
```

- [ ] **Step 2: Run reader tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because the new group metadata and activity components do not exist.

- [ ] **Step 3: Create `SourceGroupMetadataView`**

Task 0 should have confirmed that `AnalysisSourceGroup` already exposes `created_at` and `updated_at`. If that preflight fails in a future branch, omit the Created/Updated rows instead of expanding backend DTOs in this slice.

Create `src/lib/components/analysis/source-group-metadata-view.svelte`:

```svelte
<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import type { AnalysisSourceGroup } from "$lib/types/analysis";

  let {
    group,
    formatTimestamp,
  }: {
    group: AnalysisSourceGroup;
    formatTimestamp: (value: number | null) => string;
  } = $props();

  const sortedMembers = $derived([...group.members].sort((left, right) =>
    (left.source_title ?? `Source ${left.source_id}`).localeCompare(
      right.source_title ?? `Source ${right.source_id}`,
      undefined,
      { sensitivity: "base", numeric: true },
    ),
  ));
</script>

<section class="source-group-metadata-view" aria-label="Source group metadata">
  <div class="metadata-header">
    <div>
      <span class="eyebrow">Group metadata</span>
      <h3>{group.name}</h3>
    </div>
    <Badge variant="info">{group.members.length} sources</Badge>
  </div>

  <section class="metadata-section" aria-labelledby="source-group-summary-title">
    <h4 id="source-group-summary-title">Summary</h4>
    <dl class="metadata-grid">
      <div>
        <dt>Name</dt>
        <dd>{group.name}</dd>
      </div>
      <div>
        <dt>Provider type</dt>
        <dd>{group.source_type}</dd>
      </div>
      <div>
        <dt>Members</dt>
        <dd>{group.members.length}</dd>
      </div>
      <div>
        <dt>Total indexed items</dt>
        <dd>{group.members.reduce((total, member) => total + member.item_count, 0)}</dd>
      </div>
      <div>
        <dt>Created</dt>
        <dd>{formatTimestamp(group.created_at)}</dd>
      </div>
      <div>
        <dt>Updated</dt>
        <dd>{formatTimestamp(group.updated_at)}</dd>
      </div>
    </dl>
  </section>

  <section class="metadata-section" aria-labelledby="source-group-members-title">
    <h4 id="source-group-members-title">Members</h4>
    <ul class="member-list">
      {#each sortedMembers as member (member.source_id)}
        <li>
          <span>{member.source_title ?? `Source ${member.source_id}`}</span>
          <Badge variant="neutral">{member.item_count} items</Badge>
        </li>
      {/each}
    </ul>
  </section>
</section>

<style>
  .source-group-metadata-view {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
  }

  .metadata-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .metadata-header h3,
  .metadata-section h4,
  .metadata-grid,
  .metadata-grid dd {
    margin: 0;
  }

  .metadata-header h3 {
    font-size: 1.05rem;
  }

  .eyebrow {
    color: var(--muted);
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .metadata-section {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    min-width: 0;
    padding-top: 0.85rem;
    border-top: 1px solid var(--border);
  }

  .metadata-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
    gap: 0.7rem 1rem;
  }

  .metadata-grid dt {
    color: var(--muted);
    font-size: 0.72rem;
    line-height: 1.35;
  }

  .metadata-grid dd {
    color: var(--text);
    font-size: 0.9rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .member-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .member-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    padding: 0.55rem 0;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 72%, transparent);
  }

  .member-list span {
    overflow-wrap: anywhere;
  }
</style>
```

- [ ] **Step 4: Create `SourceGroupActivityView`**

Create `src/lib/components/analysis/source-group-activity-view.svelte`:

```svelte
<script lang="ts">
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
</script>

<section class="source-group-activity-view" aria-label="Source group activity">
  <EmptyState description="Group activity is not available yet. Source jobs are still tracked per source." />
</section>

<style>
  .source-group-activity-view {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
  }
</style>
```

- [ ] **Step 5: Run reader tests and verify they pass**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit group metadata/activity task**

Run:

```bash
git add src/lib/components/analysis/source-group-metadata-view.svelte src/lib/components/analysis/source-group-activity-view.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: add source group browser leaves"
```

## Task 5: Wire Source Groups Through `SourceBrowserShell`

**Files:**
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.test.ts`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Add shell contract tests for subjects and group tabs**

In `src/lib/components/analysis/source-browser-shell.test.ts`, update the first test:

```ts
  it("uses the subject-aware source browser model and keeps data fetching outside the shell", () => {
    expect(shellSource).toContain("sourceBrowserTabsForSubject");
    expect(shellSource).toContain("reconcileSourceBrowserTab");
    expect(shellSource).toContain("SourceBrowserSubject");
    expect(shellSource).not.toContain("$lib/api/");
    expect(shellSource).not.toContain("invoke(");
  });
```

Add this test:

```ts
  it("renders source group tabs through route-owned props", () => {
    expect(shellSource).toContain("<SourceGroupSourcesView");
    expect(shellSource).toContain("<SourceGroupMetadataView");
    expect(shellSource).toContain("<SourceGroupActivityView");
    expect(shellSource).toContain('activeTab === "sources"');
    expect(shellSource).toContain("groupBrowserData");
    expect(shellSource).toContain("liveReaderItems");
    expect(shellSource).toContain("sourceItems");
    expect(shellSource).toContain("sourceLabelForItem");
    expect(shellSource).toContain("Group items are limited to the source rows loaded in this browser session");
  });
```

In `src/lib/analysis-source-readers.test.ts`, add:

```ts
  it("keeps source group activity out of SourceActivityView", () => {
    expect(sourceBrowserShellSource).toContain("<SourceGroupActivityView");
    expect(sourceBrowserShellSource).toContain("<SourceActivityView");
    expect(sourceBrowserShellSource).toContain('activeTab === "activity" && groupSubject');
    expect(sourceBrowserShellSource).toContain('activeTab === "activity" && sourceSubject');
    expect(sourceBrowserShellSource).toContain('subject.kind === "source_group"');
    expect(sourceBrowserShellSource).toContain('subject.kind === "source"');
    expect(sourceBrowserShellSource).toContain("sourceSubject");
  });
```

- [ ] **Step 2: Run shell tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because the shell still uses source-only props and does not render group leaves.

- [ ] **Step 3: Add group imports and props to `SourceBrowserShell`**

In `src/lib/components/analysis/source-browser-shell.svelte`, add imports:

```ts
  import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
  import SourceGroupMetadataView from "$lib/components/analysis/source-group-metadata-view.svelte";
  import SourceGroupSourcesView from "$lib/components/analysis/source-group-sources-view.svelte";
```

Update model imports:

```ts
  import {
    reconcileSourceBrowserTab,
    sourceBrowserTabsForSubject,
    type SourceBrowserSubject,
    type SourceBrowserTabId,
  } from "$lib/source-browser-model";
```

Add the grouped browser data type above `Props`:

```ts
  type SourceGroupBrowserData = {
    liveReaderItems: SourceReaderItem[];
    sourceItems: SourceItem[];
    selectedSourceId: number | null;
    hasMoreBySource: Record<number, boolean>;
    sourceLabelForItem: (item: SourceItem) => string | null;
    onLoadSourcePage: (sourceId: number) => void | Promise<void>;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
  };
```

Update `Props` so `source` can be absent for group subjects and group data is route-owned:

```ts
    subject: SourceBrowserSubject;
    source: Source | null;
    groupBrowserData?: SourceGroupBrowserData | null;
```

Add these defaults to destructuring:

```ts
    subject,
    source,
    groupBrowserData = null,
```

Keep all existing single-source props unchanged.

- [ ] **Step 4: Derive subject-aware shell state**

Replace the source-only derived state block that starts with:

```ts
  let lastSourceId = $state<number | null>(null);
  const tabs = $derived(sourceBrowserTabsForSource(source));
```

and includes `sortedSourceTopics` / `telegramHistoryScopeOptions` with:

```ts
  let lastSubjectKey = $state<string | null>(null);
  const tabs = $derived(sourceBrowserTabsForSubject(subject));
  const sourceSubject = $derived(subject.kind === "source" ? subject.source : null);
  const groupSubject = $derived(subject.kind === "source_group" ? subject.group : null);
  const groupData = $derived(subject.kind === "source_group" ? groupBrowserData : null);
  const subjectKey = $derived(
    subject.kind === "source"
      ? `source:${subject.source.id}`
      : `source_group:${subject.group.id}`,
  );
  const itemsForActiveSubject = $derived(groupData?.sourceItems ?? sourceItems);
  const itemsEmptyDescription = $derived(
    subject.kind === "source_group"
      ? "Group items are limited to the source rows loaded in this browser session. Use Sources to load more rows for each member source."
      : sourceSubject?.sourceType === "youtube" && sourceSubject.sourceSubtype === "playlist"
        ? "Playlist videos live in the Videos tab. This Items tab only shows generic archived items loaded for this playlist source."
        : "No loaded items are available for this source window.",
  );
  const sortedSourceTopics = $derived(sourceSubject ? [...sourceTopics].sort(compareTopics) : []);
  const telegramHistoryScopeOptions = $derived.by(() => {
    if (!sourceSubject || sourceSubject.sourceType !== "telegram") return [];
    if (sourceSubject.migratedHistoryRowCount <= 0) return [];
    return [
      { value: "current" as const, label: "Current supergroup history" },
      { value: "migrated" as const, label: "Migrated small-group history" },
      { value: "merged" as const, label: "Merged timeline" },
    ];
  });
```

Replace the tab reconciliation effect:

```ts
  $effect(() => {
    if (lastSubjectKey !== subjectKey || !activeTab || !tabs.some((tab) => tab.id === activeTab)) {
      activeTab = reconcileSourceBrowserTab(activeTab, subject);
      lastSubjectKey = subjectKey;
    }
  });
```

Add a no-op load more helper:

```ts
  function loadMoreGroupItems() {
    return undefined;
  }

  function loadMoreGroupSourcePage(sourceId: number) {
    return groupData?.onLoadSourcePage(sourceId);
  }
```

- [ ] **Step 5: Render the group `Sources` tab**

Before the timeline branch in markup, add:

```svelte
  {#if activeTab === "sources" && groupSubject}
    <SourceGroupSourcesView
      items={groupData?.liveReaderItems ?? []}
      selectedGroupSourceId={groupData?.selectedSourceId ?? null}
      loading={loadingItems}
      hasMoreBySource={groupData?.hasMoreBySource ?? {}}
      youtubeDetailsBySource={groupData?.youtubeDetailsBySource ?? {}}
      {selectedTraceRef}
      {formatTimestamp}
      onLoadMoreSource={loadMoreGroupSourcePage}
    />
  {:else if activeTab === "timeline" && sourceSubject}
```

Then change the existing first branch from:

```svelte
  {#if activeTab === "timeline"}
```

to the `{:else if activeTab === "timeline" && sourceSubject}` branch expression shown above. Keep the existing timeline body inside that branch.

- [ ] **Step 6: Guard single-source branches and add group Items/Metadata/Activity**

Keep every source-scoped diagnostic and action behind `sourceSubject` checks. Topic controls, Takeout recovery, source job status, source sync actions, and `SourceActivityView` must render only when `subject.kind === "source"`.

Change transcript/videos/comments branches to require `sourceSubject`:

```svelte
  {:else if activeTab === "transcript" && sourceSubject}
```

```svelte
  {:else if activeTab === "videos" && sourceSubject}
```

```svelte
  {:else if activeTab === "comments" && sourceSubject}
```

Replace the activity branch with a group-first branch:

```svelte
  {:else if activeTab === "activity" && groupSubject}
    <SourceGroupActivityView />
  {:else if activeTab === "activity" && sourceSubject}
    <SourceActivityView
      source={sourceSubject}
      jobs={sourceJobs}
      takeoutRecovery={takeoutRecovery}
      sourceSyncDisabledReason={sourceSyncDisabledReason}
      {formatTimestamp}
      onSyncSource={() => onSyncSource(sourceSubject.id)}
      onSyncMetadata={() => onSyncYoutubeMetadata(sourceSubject.id)}
      onSyncTranscript={() => onSyncYoutubeTranscript(sourceSubject.id)}
      onSyncComments={() => onSyncYoutubeComments(sourceSubject.id)}
      onStartTakeoutImport={() => onStartTakeoutImport(sourceSubject.id)}
      onStartMigratedHistoryImport={() => onStartMigratedHistoryImport(sourceSubject.id)}
      onCancelSourceJob={onCancelSourceJob}
    />
```

Replace the `Items` branch with:

```svelte
  {:else if activeTab === "items"}
    <UniversalItemsView
      items={itemsForActiveSubject}
      loading={loadingItems}
      hasMore={subject.kind === "source_group" ? false : sourceItemsHasMore}
      emptyDescription={itemsEmptyDescription}
      sourceLabelForItem={subject.kind === "source_group" ? groupData?.sourceLabelForItem ?? null : null}
      {formatTimestamp}
      onLoadMore={subject.kind === "source_group" ? loadMoreGroupItems : onLoadMoreSourceItems}
    />
```

Replace the metadata branch with:

```svelte
  {:else if activeTab === "metadata" && groupSubject}
    <SourceGroupMetadataView group={groupSubject} {formatTimestamp} />
  {:else if activeTab === "metadata" && sourceSubject}
    <SourceMetadataView
      source={sourceSubject}
      youtubeVideoDetail={youtubeVideoDetail}
      youtubePlaylistDetail={youtubePlaylistDetail}
      sourceTopics={sourceTopics}
      loading={loadingYoutubeDetail}
      {formatTimestamp}
      onSyncMetadata={() => onSyncYoutubeMetadata(sourceSubject.id)}
    />
```

Within guarded source branches, replace `source.id`, `source.title`, `source.externalId`, `source.sourceType`, and `source.sourceSubtype` with `sourceSubject.id`, `sourceSubject.title`, `sourceSubject.externalId`, `sourceSubject.sourceType`, and `sourceSubject.sourceSubtype`.

In the final disabled-tab fallback, replace `Loaded rows: {sourceItems.length}` with `Loaded rows: {itemsForActiveSubject.length}`.

- [ ] **Step 7: Run shell tests and verify they pass**

Run:

```bash
npm.cmd run test -- src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [ ] **Step 8: Run Svelte/TypeScript check for shell wiring**

Run:

```bash
npm.cmd run check
```

Expected: PASS. This catches nullable `source`, grouped data typing, and guarded branch mistakes that raw component tests cannot catch.

- [ ] **Step 9: Commit shell task**

Run:

```bash
git add src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts
git commit -m "feat: support source group subjects in source browser shell"
```

## Task 6: Route Live Source Groups Through The Shell

**Files:**
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`

- [ ] **Step 1: Add route contract tests for live groups and snapshots**

In `src/lib/analysis-source-readers.test.ts`, update the first test name and expectations:

```ts
  it("routes live browsable sources and source groups through SourceBrowserShell", () => {
    expect(reportSourceSurfaceSource).toContain("sourceBrowserShellAppliesToSource(currentSource)");
    expect(reportSourceSurfaceSource).toContain("sourceBrowserShellAppliesToSubject");
    expect(reportSourceSurfaceSource).toContain('subject={{ kind: "source", source: currentSource }}');
    expect(reportSourceSurfaceSource).toContain('subject={{ kind: "source_group", group: currentGroup }}');
    expect(reportSourceSurfaceSource).toContain("groupLiveSourceItems");
    expect(reportSourceSurfaceSource).toContain("groupBrowserData");
    expect(reportSourceSurfaceSource).toContain("sourceLabelForGroupItem");
    expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistReader");
  });
```

Add this test:

```ts
  it("keeps saved snapshots outside SourceBrowserShell", () => {
    expect(reportSourceSurfaceSource).toContain('sourceViewBasis === "run_snapshot"');
    expect(reportSourceSurfaceSource).toContain("<SourceGroupReader");
    expect(reportSourceSurfaceSource).toContain('analysisScope === "source_group" && currentGroup');
    expect(reportSourceSurfaceSource).toContain('subject={{ kind: "source_group", group: currentGroup }}');
    expect(reportSourceSurfaceSource).not.toContain('sourceViewBasis === "run_snapshot" && sourceBrowserShellAppliesToSubject');
  });
```

In `src/lib/analysis-report-canvas.test.ts`, update `"keeps snapshot and live source basis explicit"` so it still expects `<SourceBrowserShell` and does not require live groups to render `SourceGroupReader` outside snapshots. Keep the existing snapshot-specific test `"keeps source-group run snapshots pageable through the grouped reader"` unchanged.

- [ ] **Step 2: Run route contract tests and verify they fail**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts
```

Expected: FAIL because live source groups still render `SourceGroupReader` directly.

- [ ] **Step 3: Import subject applicability in `ReportSourceSurface`**

In `src/lib/components/analysis/report-source-surface.svelte`, update the model import:

```ts
  import {
    sourceBrowserShellAppliesToSource,
    sourceBrowserShellAppliesToSubject,
  } from "$lib/source-browser-model";
```

- [ ] **Step 4: Derive group source item rows and labels**

Task 0 should have confirmed that `groupLiveItemsBySource` is `Record<number, SourceItem[]>`. Below `groupLiveReaderItems`, add:

```ts
  const groupLiveSourceItems = $derived.by(() =>
    Object.values(groupLiveItemsBySource).flat(),
  );
```

This feeds `UniversalItemsView` with the existing `SourceItem[]` route state. Do not derive group `Items` from `SourceReaderItem[]`.

Add a label helper near `groupMemberSource`:

```ts
  function sourceLabelForGroupItem(item: SourceItem) {
    const member = groupMemberSource(item.sourceId);
    return member?.source_title ?? `Source ${item.sourceId}`;
  }
```

- [ ] **Step 5: Pass `subject` and new group defaults to the existing single-source shell**

Inside the existing single-source `SourceBrowserShell` component invocation, add:

```svelte
        subject={{ kind: "source", source: currentSource }}
        groupBrowserData={null}
```

- [ ] **Step 6: Route live source groups through `SourceBrowserShell`**

Replace the live `source_group` branch:

```svelte
  {:else if analysisScope === "source_group" && currentGroup}
    <SourceGroupReader
      items={groupLiveReaderItems}
      {selectedGroupSourceId}
      loading={loadingItems}
      hasMoreBySource={groupLiveHasMoreBySource}
      youtubeDetailsBySource={{}}
      {formatTimestamp}
      onLoadMoreSource={onLoadLiveGroupSourcePage}
    />
```

with:

```svelte
  {:else if analysisScope === "source_group" && currentGroup}
    {#if sourceBrowserShellAppliesToSubject({ kind: "source_group", group: currentGroup })}
      <SourceBrowserShell
        subject={{ kind: "source_group", group: currentGroup }}
        source={null}
        liveReaderItems={[]}
        takeoutRecovery={null}
        sourceItems={[]}
        sourceRouteError={null}
        sourceItemsHasMore={false}
        {loadingItems}
        sourceTopics={[]}
        loadingSourceTopics={false}
        selectedTopicKey="__all_topics__"
        showTopicSelector={false}
        youtubeVideoDetail={null}
        youtubePlaylistDetail={null}
        youtubeTranscriptSegments={[]}
        youtubeTranscriptSearch=""
        youtubeTranscriptHasMore={false}
        loadingYoutubeTranscriptSegments={false}
        loadingYoutubeDetail={false}
        sourceJobs={[]}
        {selectedTraceRef}
        {telegramHistoryScope}
        currentSourceContentLabel="Source group material"
        sourceSyncDisabledReason={() => null}
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
        {onSyncSource}
        {onLoadMoreSourceItems}
        {onChangeSelectedTopicKey}
        {onChangeTelegramHistoryScope}
        {onChangeTranscriptSearch}
        {onLoadMoreYoutubeTranscriptSegments}
        {onOpenSource}
        {onSyncYoutubeMetadata}
        {onSyncYoutubeTranscript}
        {onSyncYoutubeComments}
        {onSyncYoutubePlaylist}
        onRetryFailedYoutubePlaylistVideos={onRetryFailedYoutubePlaylistVideos}
        {onSyncYoutubePlaylistVideo}
        {onRetryYoutubePlaylistVideo}
        {onStartTakeoutImport}
        {onStartMigratedHistoryImport}
        onCancelSourceJob={onCancelSourceJob}
      />
    {/if}
```

Do not change the snapshot branch that renders `SourceGroupReader` under `sourceViewBasis === "run_snapshot"`.

- [ ] **Step 7: Run route contract tests and verify they pass**

Run:

```bash
npm.cmd run test -- src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/components/analysis/source-browser-shell.test.ts
```

Expected: PASS.

- [ ] **Step 8: Run Svelte/TypeScript check for route wiring**

Run:

```bash
npm.cmd run check
```

Expected: PASS. This catches `SourceBrowserShell` prop mismatches from `ReportSourceSurface`.

- [ ] **Step 9: Commit route wiring task**

Run:

```bash
git add src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts
git commit -m "feat: route live source groups through source browser"
```

## Task 7: Frontend Verification And Tauri Smoke

**Files:**
- Modify: `docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-group-source-browser-implementation.md`

- [ ] **Step 1: Run focused frontend tests**

Run:

```bash
npm.cmd run test -- src/lib/source-browser-model.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/analysis-report-canvas.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run full verification**

Run:

```bash
npm.cmd run verify
```

Expected: PASS, including Vitest, `svelte-check`, Rust checks/tests, and `git diff HEAD --check`.

- [ ] **Step 3: Run Tauri acceptance smoke**

Start the app:

```bash
npm.cmd run tauri dev
```

Use the MCP bridge console to seed fixtures:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

In `/analysis`, verify:

- Selecting `__analysis_redesign_fixture__ Telegram Group` opens the Source Browser.
- Tabs are `Sources`, `Items`, `Metadata`, `Activity`.
- `Sources` is selected by default.
- `Sources` shows member source sections and per-source rows.
- `Items` shows loaded group rows with member source attribution.
- `Items` empty/help copy says:
  `Group items are limited to the source rows loaded in this browser session. Use Sources to load more rows for each member source.`
- `Metadata` shows group name, provider type, member count, and member list.
- `Activity` shows:
  `Group activity is not available yet. Source jobs are still tracked per source.`
- `Activity` does not show source sync CTAs or detailed source job cards.
- Saved group snapshot still renders through the existing snapshot path and does not enter `SourceBrowserShell`.

Stop the Tauri dev process after the smoke.

- [ ] **Step 4: Update spec status**

In `docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md`, change:

```markdown
> Status: approved design, pending implementation plan
```

to:

```markdown
> Status: implemented on 2026-05-30; pending merge
```

- [ ] **Step 5: Run final clean checks**

Run:

```bash
git diff --check
git status --short --branch
```

Expected: no whitespace errors; only the spec and this plan file have unstaged checkbox/status changes.

- [ ] **Step 6: Commit verification marker**

Run:

```bash
git add docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md docs/superpowers/plans/2026-05-30-source-group-source-browser-implementation.md
git commit -m "test: verify source group browser"
```

## Final Acceptance

- Live source groups enter `SourceBrowserShell`.
- `SourceBrowserShell` receives full `SourceBrowserSubject` objects, not narrow casts.
- Live source group tabs are `Sources | Items | Metadata | Activity`.
- `Sources` is the smart default for source groups.
- Source-only tab behavior remains unchanged.
- Source wrappers behavior matches subject-aware source calls.
- Group-to-group tab reconciliation preserves supported group tabs.
- Unsupported source-only tabs fall back to `Sources` for source groups.
- `Sources` uses a route-free group leaf and route-owned paging callbacks.
- `Sources` preserves route-owned evidence selection by passing `selectedTraceRef` into the group leaf.
- Group `Items` uses loaded-window semantics and preserves member source attribution.
- Group `Metadata` uses route-owned group/member fields without backend expansion.
- Group `Activity` does not render `SourceActivityView`, source job cards, or source-scoped CTAs.
- Saved run snapshots and saved group snapshots do not enter `SourceBrowserShell`.
- Shell and group leaves import no `$lib/api/*` modules and call no `invoke`.
