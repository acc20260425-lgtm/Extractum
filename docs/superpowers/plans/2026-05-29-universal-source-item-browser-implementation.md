# Universal Source Item Browser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the approved live single-source Source Browser in small review slices: shell/wrappers, Activity, Items, Comments, and Metadata.

**Architecture:** `/analysis/+page.svelte` remains the data owner and passes route state/callbacks through `ReportSourceSurface` into a new shell. `SourceBrowserShell.svelte` owns only local tab state and delegates data loading through props. Backend changes are limited to optional DTO enrichment for YouTube comments and source-level YouTube metadata detail.

**Tech Stack:** Svelte 5 runes, SvelteKit, Vitest raw-component contract tests, Tauri v2 commands, Rust/sqlx/SQLite.

---

## Reference Documents

- Spec: `docs/superpowers/specs/2026-05-29-universal-source-item-browser-design.md`
- Existing route surface: `src/lib/components/analysis/report-source-surface.svelte`
- Existing live readers: `src/lib/components/analysis/telegram-timeline-reader.svelte`, `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Existing source item model: `src/lib/source-reader-model.ts`
- Existing source API mapping: `src/lib/api/sources.ts`
- Existing backend source item command: `src-tauri/src/sources/items.rs`
- Existing YouTube detail command: `src-tauri/src/youtube/detail.rs`

## Review Slices

Implement and review these slices in order:

1. Shell + wrappers
2. Activity
3. Universal Items
4. Comments backend DTO
5. Comments UI
6. Metadata

Each slice should end with a commit. Run the focused tests in the slice, then run `npm run check` for frontend slices and the named `cargo test` command for backend slices. Run `npm run verify` before merging a combined branch.

Decision: in Slice 1, `SourceBrowserShell` handles Telegram live sources and
YouTube video live sources only. YouTube playlist live sources keep the
existing `YoutubePlaylistReader` path, like source groups and saved run
snapshots. Playlist migration is a follow-up under richer playlist/video nested
browsing.

## File Map

Create:

- `src/lib/source-browser-model.ts`: tab ids, tab derivation, active-tab reconciliation, item helpers, comment helpers, metadata/raw-json helpers.
- `src/lib/source-browser-model.test.ts`: model tests for every helper added in this plan.
- `src/lib/components/analysis/source-browser-shell.svelte`: live single-source tab shell with local active tab state.
- `src/lib/components/analysis/source-browser-shell.test.ts`: raw source architecture guards for shell props, provider wrappers, and no direct API imports.
- `src/lib/components/analysis/source-activity-view.svelte`: provider-neutral source activity/status/actions tab.
- `src/lib/components/analysis/universal-items-view.svelte`: loaded-window item browser.
- `src/lib/components/analysis/youtube-comments-view.svelte`: YouTube comments tab.
- `src/lib/components/analysis/source-metadata-view.svelte`: structured source metadata tab.
- `src/lib/components/analysis/raw-json-panel.svelte`: collapsed, bounded raw JSON panel.

Modify:

- `src/lib/components/analysis/report-source-surface.svelte`: render `SourceBrowserShell` only for Telegram and YouTube video live single-source surfaces.
- `src/lib/components/analysis/youtube-transcript-reader.svelte`: keep transcript reader focused on transcript content and contextual CTAs.
- `src/lib/components/analysis/youtube-playlist-reader.svelte`: keep playlist live sources on the existing reader path in this plan.
- `src/lib/components/analysis/youtube-source-activity.svelte`: keep as a compatibility source while moving reusable job-card behavior into `SourceActivityView`.
- `src/lib/types/sources.ts`: add optional `youtubeComment` to `SourceItem`.
- `src/lib/types/youtube.ts`: add safe metadata detail fields and optional raw metadata JSON when backend exposes them.
- `src/lib/api/sources.ts`: map `raw.youtube_comment` to `SourceItem.youtubeComment`.
- `src/lib/api/sources.test.ts`: assert `youtubeComment` mapping and no separate comments endpoint.
- `src/lib/api/youtube-detail.ts`: keep command name, map any newly added detail fields.
- `src/lib/api/youtube-detail.test.ts`: assert raw metadata detail mapping.
- `src/lib/analysis-source-readers.test.ts`: update reader contract tests as each slice moves UI.
- `src/lib/analysis-source-readers-route.test.ts`: assert route prop boundary and no `activeSourceBrowserTab`.
- `src-tauri/src/sources/items.rs`: add optional YouTube comment enrichment to `ItemRecord`.
- `src-tauri/src/youtube/detail.rs`: expose safe source-level metadata fields and optional sanitized raw metadata JSON.

## Slice 1: Shell + Wrappers

### Task 1.1: Add the source browser tab model

**Files:**
- Create: `src/lib/source-browser-model.ts`
- Create: `src/lib/source-browser-model.test.ts`

- [ ] **Step 1: Write the failing tab-model tests**

Add `src/lib/source-browser-model.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  reconcileSourceBrowserTab,
  sourceBrowserShellAppliesToSource,
  sourceBrowserTabsForSource,
  smartDefaultSourceBrowserTab,
  type SourceBrowserTabId,
} from "./source-browser-model";
import type { Source } from "./types/sources";

function source(overrides: Partial<Source>): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "supergroup",
    accountId: 10,
    externalId: "demo",
    title: "Demo",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 1,
    telegramUsername: null,
    avatarDataUrl: null,
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
    ...overrides,
  };
}

describe("source browser model", () => {
  it("derives canonical tabs for supported source types", () => {
    expect(sourceBrowserTabsForSource(source({ sourceType: "telegram" })).map((tab) => tab.id))
      .toEqual(["timeline", "items", "metadata", "activity"]);
    expect(sourceBrowserTabsForSource(source({ sourceType: "youtube", sourceSubtype: "video" })).map((tab) => tab.id))
      .toEqual(["transcript", "comments", "items", "metadata", "activity"]);
    expect(sourceBrowserTabsForSource(source({ sourceType: "rss", sourceSubtype: "feed" })).map((tab) => tab.id))
      .toEqual(["items", "metadata", "activity"]);
  });

  it("selects smart defaults by canonical tab id", () => {
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "telegram" }))).toBe("timeline");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "youtube", sourceSubtype: "video" }))).toBe("transcript");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "forum", sourceSubtype: "thread" }))).toBe("items");
  });

  it("preserves an active tab across source changes only when supported", () => {
    const youtube = source({ id: 2, sourceType: "youtube", sourceSubtype: "video" });
    const telegram = source({ id: 3, sourceType: "telegram", sourceSubtype: "supergroup" });
    const active: SourceBrowserTabId = "comments";

    expect(reconcileSourceBrowserTab(active, youtube)).toBe("comments");
    expect(reconcileSourceBrowserTab(active, telegram)).toBe("timeline");
    expect(reconcileSourceBrowserTab("metadata", telegram)).toBe("metadata");
  });

  it("routes only Telegram and YouTube video live sources into the shell in this slice", () => {
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "telegram", sourceSubtype: "supergroup" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "youtube", sourceSubtype: "video" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "youtube", sourceSubtype: "playlist" }))).toBe(false);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "rss", sourceSubtype: "feed" }))).toBe(false);
  });
});
```

- [ ] **Step 2: Run the failing test**

Run: `npm run test -- src/lib/source-browser-model.test.ts`

Expected: FAIL because `src/lib/source-browser-model.ts` does not exist.

- [ ] **Step 3: Implement the minimal tab model**

Create `src/lib/source-browser-model.ts`:

```ts
import type { Source } from "$lib/types/sources";

export type SourceBrowserTabId =
  | "timeline"
  | "transcript"
  | "comments"
  | "items"
  | "metadata"
  | "activity";

export interface SourceBrowserTab {
  id: SourceBrowserTabId;
  label: string;
}

const TAB_LABELS: Record<SourceBrowserTabId, string> = {
  timeline: "Timeline",
  transcript: "Transcript",
  comments: "Comments",
  items: "Items",
  metadata: "Metadata",
  activity: "Activity",
};

export function sourceBrowserTabsForSource(source: Pick<Source, "sourceType" | "sourceSubtype">): SourceBrowserTab[] {
  const ids: SourceBrowserTabId[] =
    source.sourceType === "youtube" && source.sourceSubtype === "video"
      ? ["transcript", "comments", "items", "metadata", "activity"]
      : source.sourceType === "telegram"
        ? ["timeline", "items", "metadata", "activity"]
        : ["items", "metadata", "activity"];
  return ids.map((id) => ({ id, label: TAB_LABELS[id] }));
}

export function sourceBrowserShellAppliesToSource(source: Pick<Source, "sourceType" | "sourceSubtype">): boolean {
  return source.sourceType === "telegram"
    || (source.sourceType === "youtube" && source.sourceSubtype === "video");
}

export function smartDefaultSourceBrowserTab(source: Pick<Source, "sourceType" | "sourceSubtype">): SourceBrowserTabId {
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") return "transcript";
  if (source.sourceType === "telegram") return "timeline";
  return "items";
}

export function reconcileSourceBrowserTab(
  activeTab: SourceBrowserTabId | null,
  source: Pick<Source, "sourceType" | "sourceSubtype">,
): SourceBrowserTabId {
  const tabs = sourceBrowserTabsForSource(source);
  return activeTab && tabs.some((tab) => tab.id === activeTab)
    ? activeTab
    : smartDefaultSourceBrowserTab(source);
}
```

- [ ] **Step 4: Run the focused test**

Run: `npm run test -- src/lib/source-browser-model.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit Slice 1 model**

```bash
git add src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts
git commit -m "feat: add source browser tab model"
```

### Task 1.2: Add `SourceBrowserShell` with existing reader wrappers

**Files:**
- Create: `src/lib/components/analysis/source-browser-shell.svelte`
- Create: `src/lib/components/analysis/source-browser-shell.test.ts`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-source-readers-route.test.ts`

- [ ] **Step 1: Write shell and route contract tests**

Add `src/lib/components/analysis/source-browser-shell.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import shellSource from "./source-browser-shell.svelte?raw";

describe("source browser shell component contract", () => {
  it("uses the source browser model and keeps data fetching outside the shell", () => {
    expect(shellSource).toContain("sourceBrowserTabsForSource");
    expect(shellSource).toContain("reconcileSourceBrowserTab");
    expect(shellSource).not.toContain("$lib/api/");
    expect(shellSource).not.toContain("invoke(");
  });

  it("renders existing provider readers as first-slice wrappers", () => {
    expect(shellSource).toContain("<TelegramTimelineReader");
    expect(shellSource).toContain("<YoutubeTranscriptReader");
    expect(shellSource).toContain("timeline");
    expect(shellSource).toContain("transcript");
  });
});
```

Update `src/lib/analysis-source-readers-route.test.ts` with:

```ts
it("keeps source browser tab state out of the analysis route", () => {
  expect(analysisPageSource).not.toContain("activeSourceBrowserTab");
});
```

Update `src/lib/analysis-source-readers.test.ts` so the first reader test expects `<SourceBrowserShell` in `ReportSourceSurface` while still expecting `<SourceGroupReader` for groups and snapshot branches.

Add this route decision guard:

```ts
it("routes only Telegram and YouTube video live sources through SourceBrowserShell", () => {
  expect(reportSourceSurfaceSource).toContain("sourceBrowserShellAppliesToSource(currentSource)");
  expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
  expect(reportSourceSurfaceSource).toContain("<YoutubePlaylistReader");
  expect(reportSourceSurfaceSource).toContain('sourceSubtype === "playlist"');
});
```

Add this regression guard:

```ts
it("preserves the existing Telegram timeline controls through the shell", () => {
  expect(sourceBrowserShellSource).toContain("telegramHistoryScopeOptions");
  expect(sourceBrowserShellSource).toContain("onChangeTelegramHistoryScope");
  expect(sourceBrowserShellSource).toContain("showTopicSelector");
  expect(sourceBrowserShellSource).toContain("onChangeSelectedTopicKey");
  expect(sourceBrowserShellSource).toContain("<TelegramTimelineReader");
  expect(sourceBrowserShellSource).toContain("liveReaderItems");
});
```

- [ ] **Step 2: Run the failing tests**

Run:

```bash
npm run test -- src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts
```

Expected: FAIL because the shell does not exist and `ReportSourceSurface` still renders readers directly.

- [ ] **Step 3: Copy the current Telegram live-reader contract before simplifying**

Before writing the shell, inspect the current live single-source Telegram branch in `src/lib/components/analysis/report-source-surface.svelte` and copy its behavior into `SourceBrowserShell`:

- migrated-history availability notice;
- history-scope select with `current`, `migrated`, and `merged`;
- topic selector for current supergroup history;
- `TelegramTimelineReader` receives the already-normalized `liveReaderItems` so selected trace scrolling, media cards, reply labels, reactions, and history badges keep working;
- `onChangeTelegramHistoryScope`, `onChangeSelectedTopicKey`, and `onLoadMoreSourceItems` keep the existing route callbacks.

Acceptance for Slice 1: `TelegramTimelineReader` receives every prop it currently needs for topic filters, history-scope controls, selected trace ref scrolling, and media/reply/reaction display. Shell integration must not reduce the existing Telegram timeline feature set.

- [ ] **Step 4: Implement the shell**

Create `src/lib/components/analysis/source-browser-shell.svelte` with a typed prop surface copied from the current live single-source branch of `ReportSourceSurface`. The first version should render only `timeline` and `transcript` with the existing readers, and render muted disabled states for `comments`, `items`, `metadata`, and `activity`:

```svelte
<script lang="ts">
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import {
    reconcileSourceBrowserTab,
    sourceBrowserTabsForSource,
    type SourceBrowserTabId,
  } from "$lib/source-browser-model";
  import type {
    Source,
    SourceForumTopic,
    SourceItem,
    SourceJobRecord,
    TelegramHistoryScope,
    YoutubeTranscriptSegment,
  } from "$lib/types/sources";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";
  import type { SourceReaderItem } from "$lib/source-reader-model";

  type Props = {
    source: Source;
    liveReaderItems: SourceReaderItem[];
    sourceItems: SourceItem[];
    sourceItemsHasMore: boolean;
    loadingItems: boolean;
    sourceTopics: SourceForumTopic[];
    loadingSourceTopics: boolean;
    selectedTopicKey: string;
    showTopicSelector: boolean;
    youtubeVideoDetail: YoutubeVideoDetail | null;
    youtubeTranscriptSegments: YoutubeTranscriptSegment[];
    youtubeTranscriptSearch: string;
    youtubeTranscriptHasMore: boolean;
    loadingYoutubeTranscriptSegments: boolean;
    loadingYoutubeDetail: boolean;
    sourceJobs: SourceJobRecord[];
    selectedTraceRef: string | null;
    telegramHistoryScope: TelegramHistoryScope;
    currentSourceContentLabel: string;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreSourceItems: () => void | Promise<void>;
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onChangeTelegramHistoryScope: (scope: TelegramHistoryScope) => void;
    onChangeTranscriptSearch: (value: string) => void;
    onLoadMoreYoutubeTranscriptSegments: () => void | Promise<void>;
    onSyncYoutubeMetadata: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeTranscript: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeComments: (sourceId: number) => void | Promise<void>;
    onCancelSourceJob: (jobId: string) => void | Promise<void>;
  };

  let {
    source,
    liveReaderItems,
    sourceItems,
    sourceItemsHasMore,
    loadingItems,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    youtubeVideoDetail,
    youtubeTranscriptSegments,
    youtubeTranscriptSearch,
    youtubeTranscriptHasMore,
    loadingYoutubeTranscriptSegments,
    loadingYoutubeDetail,
    sourceJobs,
    selectedTraceRef,
    telegramHistoryScope,
    currentSourceContentLabel,
    formatTimestamp,
    onLoadMoreSourceItems,
    onChangeSelectedTopicKey,
    onChangeTelegramHistoryScope,
    onChangeTranscriptSearch,
    onLoadMoreYoutubeTranscriptSegments,
    onSyncYoutubeMetadata,
    onSyncYoutubeTranscript,
    onSyncYoutubeComments,
    onCancelSourceJob,
  }: Props = $props();

  let activeTab = $state<SourceBrowserTabId | null>(null);
  let lastSourceId = $state<number | null>(null);
  const tabs = $derived(sourceBrowserTabsForSource(source));
  const sortedSourceTopics = $derived([...sourceTopics].sort(compareTopics));
  const telegramHistoryScopeOptions = $derived.by(() => {
    if (source.sourceType !== "telegram") return [];
    if (source.migratedHistoryRowCount <= 0) return [];
    return [
      { value: "current" as const, label: "Current supergroup history" },
      { value: "migrated" as const, label: "Migrated small-group history" },
      { value: "merged" as const, label: "Merged timeline" },
    ];
  });

  $effect(() => {
    if (lastSourceId !== source.id || !activeTab || !tabs.some((tab) => tab.id === activeTab)) {
      activeTab = reconcileSourceBrowserTab(activeTab, source);
      lastSourceId = source.id;
    }
  });

  function compareTopics(left: SourceForumTopic, right: SourceForumTopic) {
    if (left.kind !== right.kind) return left.kind === "topic" ? -1 : 1;
    if (left.isDeleted !== right.isDeleted) return left.isDeleted ? 1 : -1;
    const titleOrder = left.title.localeCompare(right.title, undefined, {
      sensitivity: "base",
      numeric: true,
    });
    return titleOrder || left.key.localeCompare(right.key, undefined, {
      sensitivity: "base",
      numeric: true,
    });
  }

  function changeSelectedTopic(event: Event) {
    onChangeSelectedTopicKey((event.currentTarget as HTMLSelectElement).value);
  }

  function changeTelegramHistoryScope(event: Event) {
    onChangeTelegramHistoryScope((event.currentTarget as HTMLSelectElement).value as TelegramHistoryScope);
  }
</script>

<section class="source-browser-shell">
  <nav class="source-browser-tabs" aria-label="Source browser tabs">
    {#each tabs as tab (tab.id)}
      <Button
        type="button"
        variant={activeTab === tab.id ? "primary" : "ghost"}
        ariaSelected={activeTab === tab.id}
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
      </Button>
    {/each}
  </nav>

  {#if activeTab === "timeline"}
    {#if source.sourceType === "telegram" && source.migratedHistoryStatus === "available" && !source.migratedHistoryImportCompleted}
      <StatusMessage tone="info">
        Migrated small-group history is detected but has not been imported for browsing yet.
      </StatusMessage>
    {/if}
    {#if telegramHistoryScopeOptions.length > 0}
      <label class="history-scope-control">
        <span>History scope</span>
        <Select value={telegramHistoryScope} onchange={changeTelegramHistoryScope}>
          {#each telegramHistoryScopeOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </Select>
      </label>
    {/if}
    {#if showTopicSelector && telegramHistoryScope === "current"}
      <label class="topic-filter">
        <span>Topic view</span>
        <Select value={selectedTopicKey} disabled={loadingSourceTopics} onchange={changeSelectedTopic}>
          <option value="__all_topics__">All topics</option>
          {#if loadingSourceTopics && sourceTopics.length === 0}
            <option value="__loading_topics__" disabled>Loading topics...</option>
          {:else}
            {#each sortedSourceTopics as topic (topic.key)}
              <option value={topic.key}>{topic.title} ({topic.messageCount})</option>
            {/each}
          {/if}
        </Select>
      </label>
    {/if}
    <TelegramTimelineReader
      items={liveReaderItems}
      loading={loadingItems}
      hasMore={sourceItemsHasMore}
      contentLabel={currentSourceContentLabel}
      {formatTimestamp}
      onLoadMore={onLoadMoreSourceItems}
    />
  {:else if activeTab === "transcript"}
    <YoutubeTranscriptReader
      detail={youtubeVideoDetail}
      segments={youtubeTranscriptSegments}
      snapshotItems={[]}
      loading={loadingYoutubeTranscriptSegments || loadingYoutubeDetail}
      hasMore={youtubeTranscriptHasMore}
      transcriptSearch={youtubeTranscriptSearch}
      sourceTitle={source.title ?? source.externalId}
      {selectedTraceRef}
      {formatTimestamp}
      onChangeTranscriptSearch={onChangeTranscriptSearch}
      onLoadMore={onLoadMoreYoutubeTranscriptSegments}
      onSyncTranscript={() => onSyncYoutubeTranscript(source.id)}
      onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
      {sourceJobs}
      onSyncComments={() => onSyncYoutubeComments(source.id)}
      onCancelSourceJob={onCancelSourceJob}
    />
  {:else}
    <StatusMessage tone="muted">
      {activeTab} source browser tab is disabled in this review slice.
    </StatusMessage>
  {/if}
</section>
```

The disabled-state text exists only inside Slice 1. Slice 2 replaces `activity`; Slices 3-5 replace the remaining disabled tab states.

- [ ] **Step 5: Wire shell into `ReportSourceSurface` live single-source branch**

Modify `src/lib/components/analysis/report-source-surface.svelte`:

- import `SourceBrowserShell`;
- import `sourceBrowserShellAppliesToSource` from `$lib/source-browser-model`;
- keep existing run snapshot and source group branches unchanged;
- in `liveSourceSurface()`, render `SourceBrowserShell` only when
  `analysisScope === "single_source" && currentSource && sourceBrowserShellAppliesToSource(currentSource)`;
- keep the existing `YoutubePlaylistReader` branch for
  `currentSource.sourceType === "youtube" && currentSource.sourceSubtype === "playlist"`;
- pass the current `liveReaderItems`, `sourceItems`, Telegram topic/history props, transcript props, job props, and callbacks.

- [ ] **Step 6: Run shell tests**

Run:

```bash
npm run test -- src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts
npm run check
```

Expected: all commands exit 0.

- [ ] **Step 7: Commit Slice 1 shell**

```bash
git add src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/source-browser-shell.test.ts src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts
git commit -m "feat: add live source browser shell"
```

### Slice 1 Acceptance

- Existing Telegram timeline behavior is preserved, including topic filtering,
  migrated-history scope controls, selected trace ref scrolling, media cards,
  reply labels, reactions, and history badges.
- Existing YouTube transcript behavior is preserved while it is wrapped by the
  shell.
- Existing YouTube playlist behavior is preserved on the old playlist reader
  path; playlists do not enter `SourceBrowserShell` in Slice 1.
- Source groups and saved run snapshots still use their previous readers.
- Active tab state is owned by `SourceBrowserShell`, not `/analysis/+page.svelte`
  or a global store.
- `source-browser-shell.test.ts` guards that the shell does not import
  `$lib/api/` and does not call `invoke(` directly.

## Slice 2: Activity

### Task 2.1: Move detailed job cards into Activity

**Files:**
- Create: `src/lib/components/analysis/source-activity-view.svelte`
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Write failing Activity contract tests**

Update `src/lib/analysis-source-readers.test.ts`:

```ts
import sourceActivityViewSource from "./components/analysis/source-activity-view.svelte?raw";
import sourceBrowserShellSource from "./components/analysis/source-browser-shell.svelte?raw";
```

Add:

```ts
it("moves detailed source job cards into the Activity tab", () => {
  expect(sourceBrowserShellSource).toContain("activity");
  expect(sourceBrowserShellSource).toContain("<SourceActivityView");
  expect(sourceActivityViewSource).toContain("SourceJobRecord");
  expect(sourceActivityViewSource).toContain("Progress");
  expect(sourceActivityViewSource).toContain("Warnings");
  expect(sourceActivityViewSource).toContain("Error");
  expect(sourceActivityViewSource).toContain("Cancel");
});

it("keeps provider tabs to contextual CTAs instead of detailed job cards", () => {
  expect(youtubeTranscriptSource).not.toContain("<YoutubeSourceActivity");
  expect(youtubeTranscriptSource).not.toContain("SourceJobRecord");
  expect(youtubeTranscriptSource).toContain("Sync comments");
  expect(youtubeTranscriptSource).toContain("Sync metadata");
});

it("covers Telegram source activity without adding backend job APIs", () => {
  expect(sourceActivityViewSource).toContain("takeoutRecovery");
  expect(sourceActivityViewSource).toContain("sourceSyncDisabledReason");
  expect(sourceActivityViewSource).toContain("onStartTakeoutImport");
  expect(sourceActivityViewSource).toContain("onStartMigratedHistoryImport");
  expect(sourceActivityViewSource).toContain("Migrated history");
  expect(sourceActivityViewSource).toContain("Takeout");
});
```

- [ ] **Step 2: Run failing tests**

Run: `npm run test -- src/lib/analysis-source-readers.test.ts`

Expected: FAIL because `SourceActivityView` does not exist and transcript reader still owns detailed activity.

- [ ] **Step 3: Implement `SourceActivityView`**

Create `src/lib/components/analysis/source-activity-view.svelte` by extracting the useful job-card rendering from `youtube-source-activity.svelte` and adding source-level action buttons. Keep all detailed job rows here: job type, status badge, started/finished timestamps, progress, warnings, errors, cancel.

Activity acceptance:

- YouTube video sources show metadata/transcript/comments jobs and related
  playlist-video jobs/actions already available for the selected video.
- YouTube playlist live sources keep the existing playlist reader/activity path
  in this plan.
- Telegram sources show basic sync state, `sourceSyncDisabledReason`, Takeout recovery, migrated-history import state, and route-level actions that already exist.
- If full Telegram job lists are not already available in route state, Slice 2 may show only basic Telegram sync/recovery status as a transitional state; that limitation must be visible in the component copy and tests.
- Do not add new backend/background job APIs in this slice.

The component prop surface should include:

```ts
source: Source;
jobs: SourceJobRecord[];
takeoutRecovery: TakeoutImportRecoveryState | null;
sourceSyncDisabledReason: (source: Source) => string | null;
formatTimestamp: (value: number | null) => string;
onSyncSource: (sourceId: number) => void | Promise<void>;
onSyncMetadata: (sourceId: number) => void | Promise<void>;
onSyncTranscript: (sourceId: number) => void | Promise<void>;
onSyncComments: (sourceId: number) => void | Promise<void>;
onStartTakeoutImport: (sourceId: number) => void | Promise<void>;
onStartMigratedHistoryImport: (sourceId: number) => void | Promise<void>;
onCancelSourceJob: (jobId: string) => void | Promise<void>;
```

If a callback is not yet passed through `ReportSourceSurface`, thread the existing route callback down from `/analysis/+page.svelte` through `ReportCanvas` and `ReportSourceSurface`. Do not create a new store for these actions.

- [ ] **Step 4: Render Activity from the shell**

Modify `source-browser-shell.svelte`:

```svelte
{:else if activeTab === "activity"}
  <SourceActivityView
    source={source}
    jobs={sourceJobs}
    takeoutRecovery={takeoutRecovery}
    sourceSyncDisabledReason={sourceSyncDisabledReason}
    {formatTimestamp}
    onSyncSource={() => onSyncSource(source.id)}
    onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
    onSyncTranscript={() => onSyncYoutubeTranscript(source.id)}
    onSyncComments={() => onSyncYoutubeComments(source.id)}
    onStartTakeoutImport={() => onStartTakeoutImport(source.id)}
    onStartMigratedHistoryImport={() => onStartMigratedHistoryImport(source.id)}
    onCancelSourceJob={onCancelSourceJob}
  />
```

- [ ] **Step 5: Remove detailed activity from transcript provider view**

Modify `youtube-transcript-reader.svelte` so it keeps compact contextual CTAs (`Sync transcript`, `Sync metadata`, `Sync comments`) but no longer imports or renders `YoutubeSourceActivity`.

- [ ] **Step 6: Run Activity tests**

Run:

```bash
npm run test -- src/lib/analysis-source-readers.test.ts src/lib/components/analysis/source-browser-shell.test.ts
npm run check
```

Expected: all commands exit 0.

- [ ] **Step 7: Commit Slice 2**

```bash
git add src/routes/analysis/+page.svelte src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/components/analysis/source-activity-view.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/youtube-transcript-reader.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: move source activity into browser tab"
```

### Slice 2 Acceptance

- Detailed jobs/status render only in `Activity`.
- Provider tabs keep contextual CTAs but do not render detailed job cards.
- YouTube playlist live sources still use `YoutubePlaylistReader` in this
  slice.
- Telegram and YouTube source status remain visible enough to avoid regression;
  Telegram may be transitional if only basic route-owned sync/recovery state is
  currently available.
- No new backend/background job APIs are introduced.

## Slice 3: Universal Items

### Task 3.1: Add loaded-window item helpers

**Files:**
- Modify: `src/lib/source-browser-model.ts`
- Modify: `src/lib/source-browser-model.test.ts`

- [ ] **Step 1: Write failing loaded-window helper tests**

Add tests for:

- `sourceItemKindChips(items)` derives chips only from loaded rows;
- `filterLoadedSourceItems(items, { kind, search })` searches loaded content and author only;
- `sortLoadedSourceItems(items, "newest" | "oldest")` sorts loaded rows only.

Use existing `SourceItem` shape from `source-reader-model.test.ts` as the fixture.

- [ ] **Step 2: Run failing tests**

Run: `npm run test -- src/lib/source-browser-model.test.ts`

Expected: FAIL because item helpers do not exist.

- [ ] **Step 3: Implement helpers in `source-browser-model.ts`**

Implement helpers as pure functions. Do not call any API from the helper layer.

- [ ] **Step 4: Run helper tests**

Run: `npm run test -- src/lib/source-browser-model.test.ts`

Expected: PASS.

### Task 3.2: Add `UniversalItemsView`

**Files:**
- Create: `src/lib/components/analysis/universal-items-view.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Write failing component contract tests**

Add to `src/lib/analysis-source-readers.test.ts`:

```ts
import universalItemsViewSource from "./components/analysis/universal-items-view.svelte?raw";
```

Add:

```ts
it("renders universal Items as a loaded-window browser", () => {
  expect(universalItemsViewSource).toContain("Search loaded items");
  expect(universalItemsViewSource).toContain("All");
  expect(universalItemsViewSource).toContain("Load more items");
  expect(universalItemsViewSource).toContain("Unknown item kind");
});
```

- [ ] **Step 2: Run failing tests**

Run: `npm run test -- src/lib/analysis-source-readers.test.ts`

Expected: FAIL because the view does not exist.

- [ ] **Step 3: Implement `UniversalItemsView`**

The component receives:

```ts
items: SourceItem[];
loading: boolean;
hasMore: boolean;
formatTimestamp: (value: number | null) => string;
onLoadMore: () => void | Promise<void>;
```

It owns local loaded-window UI state: search string, selected kind chip, sort mode. It must label the input `Search loaded items`, derive chips from `items`, render a generic card for unknown kinds, and call `onLoadMore` for pagination.

- [ ] **Step 4: Wire Items tab**

Modify `source-browser-shell.svelte`:

```svelte
{:else if activeTab === "items"}
  <UniversalItemsView
    items={sourceItems}
    loading={loadingItems}
    hasMore={sourceItemsHasMore}
    {formatTimestamp}
    onLoadMore={onLoadMoreSourceItems}
  />
```

- [ ] **Step 5: Run Slice 3 tests**

Run:

```bash
npm run test -- src/lib/source-browser-model.test.ts src/lib/analysis-source-readers.test.ts
npm run check
```

Expected: all commands exit 0.

- [ ] **Step 6: Commit Slice 3**

```bash
git add src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts src/lib/components/analysis/universal-items-view.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: add universal loaded items browser"
```

### Slice 3 Acceptance

- Items search, filter, and sort are explicitly loaded-window only.
- `Load more` appends another source item window through the existing
  `list_source_items` route-owned pagination path.
- Item kind chips are derived only from loaded rows.
- Unknown item kinds render a generic fallback card.

## Slice 4a: Comments Backend DTO

### Task 4.1: Add optional `youtubeComment` enrichment to source items

**Files:**
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`
- Modify: `src/lib/api/sources.test.ts`

- [ ] **Step 1: Write failing frontend API mapping test**

Add to `src/lib/api/sources.test.ts`:

```ts
it("maps optional YouTube comment enrichment on source items", async () => {
  invokeMock.mockResolvedValueOnce([
    {
      id: 1,
      source_id: 7,
      external_id: "comment:c1",
      item_kind: "youtube_comment",
      author: "Alice",
      published_at: 1700000000,
      content: "Hello",
      content_kind: "text_only",
      has_media: false,
      media_kind: null,
      media_summary: null,
      media_file_name: null,
      media_mime_type: null,
      has_raw_data: true,
      forum_topic_id: null,
      forum_topic_title: null,
      forum_topic_top_message_id: null,
      reply_to_msg_id: null,
      reply_to_peer_kind: null,
      reply_to_peer_id: null,
      reply_to_top_id: null,
      reaction_count: 5,
      history_scope: "current",
      is_migrated_history: false,
      migration_domain: null,
      history_scope_label: "Current supergroup history",
      page_cursor: "cursor",
      youtube_comment: {
        comment_id: "c1",
        parent_comment_id: null,
        is_reply: false,
        like_count: 5,
        is_pinned: true,
        is_hearted: false,
        author_channel_url: "https://www.youtube.com/@alice",
      },
    },
  ]);

  await expect(listSourceItems({
    sourceId: 7,
    limit: 50,
    beforePublishedAt: null,
    topicFilter: null,
  })).resolves.toMatchObject([
    {
      youtubeComment: {
        commentId: "c1",
        isPinned: true,
        authorChannelUrl: "https://www.youtube.com/@alice",
      },
    },
  ]);
});
```

- [ ] **Step 2: Run failing frontend test**

Run: `npm run test -- src/lib/api/sources.test.ts`

Expected: FAIL because `youtube_comment` is not mapped.

- [ ] **Step 3: Add frontend types and mapper**

In `src/lib/types/sources.ts`, add:

```ts
export interface YoutubeCommentItem {
  commentId: string | null;
  parentCommentId: string | null;
  isReply: boolean;
  likeCount: number | null;
  isPinned: boolean;
  isHearted: boolean;
  authorChannelUrl: string | null;
}
```

Add `youtubeComment?: YoutubeCommentItem` to `SourceItem`.

In `src/lib/api/sources.ts`, add `youtube_comment?: RawYoutubeCommentItem | null` to `RawSourceItem` and map it to camelCase.

- [ ] **Step 4: Write backend tests**

Add Rust tests in `src-tauri/src/sources/items.rs` near existing YouTube comment item tests:

- one test inserts a `YoutubeComment`, calls the internal list path, and asserts `youtube_comment.comment_id`, `parent_comment_id`, `like_count`, `is_pinned`, `is_hearted`, and `author_channel_url`;
- one test corrupts `raw_data_zstd` for a YouTube comment item and asserts the base item row still returns with `youtube_comment = None`.

- [ ] **Step 5: Implement backend enrichment**

In `src-tauri/src/sources/items.rs`:

- add `YoutubeCommentItemRecord` using snake_case serialized field names to match existing Tauri command DTOs consumed by `src/lib/api/sources.ts`;
- add `youtube_comment: Option<YoutubeCommentItemRecord>` to `ItemRecord`;
- after `load_item_rows_from_pool`, collect ids where `item_kind == ITEM_KIND_YOUTUBE_COMMENT`;
- load only those ids from `items.raw_data_zstd`;
- decompress and deserialize into `crate::youtube::dto::YoutubeComment`;
- build a `HashMap<i64, YoutubeCommentItemRecord>`;
- pass enrichment into `item_record_from_row`.

Keep malformed raw payloads non-fatal: log-free `None` is enough for this command because the base row remains valid.
Frontend mapping owns the case conversion: backend returns snake_case `youtube_comment.comment_id`, `parent_comment_id`, `like_count`, `is_pinned`, `is_hearted`, and `author_channel_url`; `src/lib/api/sources.ts` maps that object to camelCase `SourceItem.youtubeComment`.

- [ ] **Step 6: Run backend and frontend tests**

Run:

```bash
npm run test -- src/lib/api/sources.test.ts
cargo test --manifest-path src-tauri/Cargo.toml youtube_comment
```

Expected: both commands exit 0.

- [ ] **Step 7: Commit Slice 4a**

```bash
git add src-tauri/src/sources/items.rs src/lib/types/sources.ts src/lib/api/sources.ts src/lib/api/sources.test.ts
git commit -m "feat: expose youtube comment item enrichment"
```

## Slice 4b: Comments UI

### Task 4.2: Add comment model helpers

**Files:**
- Modify: `src/lib/source-browser-model.ts`
- Modify: `src/lib/source-browser-model.test.ts`

- [ ] **Step 1: Write failing comment helper tests**

Add tests for:

- `commentsCoverageState({ items, detail, jobs, routeError, loadingItems })` returns `unknown`, `not_synced`, `syncing`, `failed`, `synced_empty`, and `synced_with_rows`;
- `groupLoadedYoutubeComments(items)` keeps orphan replies visible with a `parentLoaded: false` marker;
- `filterLoadedYoutubeComments(...)` searches loaded text and author;
- `sortLoadedYoutubeComments(..., "most_liked")` sorts loaded comments by `youtubeComment.likeCount`.

- [ ] **Step 2: Run failing tests**

Run: `npm run test -- src/lib/source-browser-model.test.ts`

Expected: FAIL because helpers do not exist.

- [ ] **Step 3: Implement pure comment helpers**

Use an object input so later route state can be added without changing every call site:

```ts
export interface CommentsCoverageInput {
  items: SourceItem[];
  detail: YoutubeVideoDetail | null;
  jobs: SourceJobRecord[];
  routeError: string | null;
  loadingItems: boolean;
}
```

Derive coverage from loaded `SourceItem[]`, `detail?.summary.comments`, relevant `SourceJobRecord[]`, `routeError`, and `loadingItems`. Do not add a comments API wrapper.

- [ ] **Step 4: Run helper tests**

Run: `npm run test -- src/lib/source-browser-model.test.ts`

Expected: PASS.

### Task 4.3: Add `YoutubeCommentsView`

**Files:**
- Create: `src/lib/components/analysis/youtube-comments-view.svelte`
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Write failing component contract tests**

Add:

```ts
import youtubeCommentsViewSource from "./components/analysis/youtube-comments-view.svelte?raw";
```

Add:

```ts
it("renders YouTube comments as a loaded-window browser", () => {
  expect(youtubeCommentsViewSource).toContain("Search loaded comments");
  expect(youtubeCommentsViewSource).toContain("Threaded");
  expect(youtubeCommentsViewSource).toContain("Flat");
  expect(youtubeCommentsViewSource).toContain("Most liked");
  expect(youtubeCommentsViewSource).toContain("parent not loaded");
  expect(youtubeCommentsViewSource).toContain("Sync comments");
});
```

- [ ] **Step 2: Run failing test**

Run: `npm run test -- src/lib/analysis-source-readers.test.ts`

Expected: FAIL because the component does not exist.

- [ ] **Step 3: Implement `YoutubeCommentsView`**

The component receives `items`, `detail`, `sourceJobs`, `routeError`, `loading`, `hasMore`, `formatTimestamp`, `onLoadMore`, `onSyncComments`, and `onSyncMetadata`. It calls `commentsCoverageState({ items, detail, jobs: sourceJobs, routeError, loadingItems: loading })`, uses `Search loaded comments`, and keeps detailed job information out of the tab.

- [ ] **Step 4: Wire Comments tab**

Add a route-owned `sourceItemsError: string | null` or equivalent scoped error
for source item loading in `/analysis/+page.svelte`, pass it through
`ReportCanvas` and `ReportSourceSurface`, and expose it to
`SourceBrowserShell` as `sourceRouteError: string | null`. Do not reuse an
unrelated global workspace status if it can contain errors from other panels.

Modify `source-browser-shell.svelte`:

```svelte
{:else if activeTab === "comments"}
  <YoutubeCommentsView
    items={sourceItems}
    detail={youtubeVideoDetail}
    {sourceJobs}
    routeError={sourceRouteError}
    loading={loadingItems}
    hasMore={sourceItemsHasMore}
    {formatTimestamp}
    onLoadMore={onLoadMoreSourceItems}
    onSyncComments={() => onSyncYoutubeComments(source.id)}
    onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
  />
```

- [ ] **Step 5: Run Slice 4b tests**

Run:

```bash
npm run test -- src/lib/source-browser-model.test.ts src/lib/analysis-source-readers.test.ts
npm run check
```

Expected: all commands exit 0.

- [ ] **Step 6: Commit Slice 4b**

```bash
git add src/routes/analysis/+page.svelte src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts src/lib/components/analysis/youtube-comments-view.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: add youtube comments browser tab"
```

### Slice 4 Acceptance

- Malformed YouTube comment raw payloads do not fail source item listing; the
  base item row remains visible without `youtubeComment` enrichment.
- Comments and Items share the same `list_source_items` pagination and loaded
  window.
- No separate comments endpoint or comments-specific pagination model is added.
- Comments search, filters, and sort communicate loaded-window scope.

## Slice 5: Metadata

### Task 5.1: Extend YouTube video detail with source-level metadata

**Files:**
- Modify: `src-tauri/src/youtube/detail.rs`
- Modify: `src/lib/types/youtube.ts`
- Modify: `src/lib/api/youtube-detail.test.ts`

- [ ] **Step 1: Write backend tests for safe raw metadata**

In `src-tauri/src/youtube/detail.rs`, add tests that assert `get_youtube_video_detail_from_pool` includes:

- summary fields already present;
- source-level technical metadata fields needed by `SourceMetadataView`;
- sanitized raw metadata JSON when available;
- no item-level `raw_data_zstd` payloads.

- [ ] **Step 2: Implement backend detail extension**

Add optional fields to `YoutubeVideoDetailDto` using existing typed metadata rows. Reuse already-sanitized `raw_metadata_zstd` from `youtube_video_sources`; do not read from `items.raw_data_zstd`.

- [ ] **Step 3: Update frontend type and API test**

In `src/lib/types/youtube.ts`, extend `YoutubeVideoDetail` with a nested source-level metadata object. In `src/lib/api/youtube-detail.test.ts`, assert `getYoutubeVideoDetail` passes through the new object.

- [ ] **Step 4: Run detail tests**

Run:

```bash
npm run test -- src/lib/api/youtube-detail.test.ts
cargo test --manifest-path src-tauri/Cargo.toml youtube::detail
```

Expected: both commands exit 0.

### Task 5.2: Add structured metadata tab

**Files:**
- Create: `src/lib/components/analysis/source-metadata-view.svelte`
- Create: `src/lib/components/analysis/raw-json-panel.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/source-browser-model.ts`
- Modify: `src/lib/source-browser-model.test.ts`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Write raw JSON helper tests**

Add tests for:

- `formatRawJsonPreview(value, maxChars)` pretty-prints small values;
- large payloads return `{ preview, truncated: true }`;
- invalid or missing raw value returns null.

- [ ] **Step 2: Implement raw JSON helpers**

Implement pure helpers in `source-browser-model.ts`. The helpers must not run during initial tab render unless `RawJsonPanel` is expanded.

- [ ] **Step 3: Write component contract tests**

Add:

```ts
import sourceMetadataViewSource from "./components/analysis/source-metadata-view.svelte?raw";
import rawJsonPanelSource from "./components/analysis/raw-json-panel.svelte?raw";
```

Add:

```ts
it("renders source metadata in structured sections with bounded raw JSON", () => {
  expect(sourceMetadataViewSource).toContain("Summary");
  expect(sourceMetadataViewSource).toContain("Source state");
  expect(sourceMetadataViewSource).toContain("Technical");
  expect(sourceMetadataViewSource).toContain("<RawJsonPanel");
  expect(rawJsonPanelSource).toContain("Show raw JSON");
  expect(rawJsonPanelSource).toContain("Copy");
  expect(rawJsonPanelSource).toContain("Large payload");
});
```

- [ ] **Step 4: Implement components**

`SourceMetadataView` receives `source`, `youtubeVideoDetail`, `sourceTopics`, `loadingYoutubeDetail`, `formatTimestamp`, and sync callbacks. It renders:

- Summary section for source identity;
- Source state section for sync, comments/captions, membership, topics, migrated history;
- Technical section for ids and provider/runtime fields;
- Raw JSON section only for source-level YouTube metadata.

`RawJsonPanel` starts collapsed, formats on expansion, constrains height, shows a copy button, and truncates visible preview for large payloads.

- [ ] **Step 5: Wire Metadata tab**

Modify `source-browser-shell.svelte`:

```svelte
{:else if activeTab === "metadata"}
  <SourceMetadataView
    source={source}
    youtubeVideoDetail={youtubeVideoDetail}
    sourceTopics={sourceTopics}
    loading={loadingYoutubeDetail}
    {formatTimestamp}
    onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
  />
```

`sourceTopics` is the same route-owned prop introduced in Slice 1 for Telegram timeline controls; do not add a new metadata-only fetch path in this slice.

- [ ] **Step 6: Run Slice 5 tests**

Run:

```bash
npm run test -- src/lib/source-browser-model.test.ts src/lib/analysis-source-readers.test.ts src/lib/api/youtube-detail.test.ts
npm run check
cargo test --manifest-path src-tauri/Cargo.toml youtube::detail
```

Expected: all commands exit 0.

- [ ] **Step 7: Commit Slice 5**

```bash
git add src-tauri/src/youtube/detail.rs src/lib/types/youtube.ts src/lib/api/youtube-detail.test.ts src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts src/lib/components/analysis/source-metadata-view.svelte src/lib/components/analysis/raw-json-panel.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: add source metadata browser tab"
```

### Slice 5 Acceptance

- Raw JSON is source-level YouTube metadata only, never arbitrary item
  `raw_data_zstd`.
- Raw JSON starts collapsed, renders in a bounded area, and exposes copy for the
  full payload when available.
- Large raw JSON is not formatted during initial Metadata tab render.
- Telegram Metadata uses route-owned source/topic state already available to the
  shell and does not add a metadata-only topic fetch path.

## Final Verification

- [ ] **Step 1: Run final shell architecture boundary test**

Run: `npm run test -- src/lib/components/analysis/source-browser-shell.test.ts`

Expected: exit 0. The test must include:

```ts
expect(shellSource).not.toContain("$lib/api/");
expect(shellSource).not.toContain("invoke(");
```

- [ ] **Step 2: Run full project verification**

Run: `npm run verify`

Expected: exit 0.

- [ ] **Step 3: Run focused manual smoke in the app**

Start the app using the normal dev workflow, open `/analysis`, and verify:

- Telegram live source opens `timeline`;
- YouTube video opens `transcript`;
- YouTube playlist live source still opens the old playlist reader;
- source groups still render the old group reader;
- saved run snapshots still render the old snapshot readers;
- switching YouTube video to YouTube video preserves `comments`, `items`, `metadata`, or `activity`;
- `items` and `comments` labels say loaded-window search;
- Activity owns detailed job cards;
- Metadata raw JSON stays collapsed and bounded.

- [ ] **Step 4: Update docs/backlog if this feature closes an item**

Only update `docs/backlog.md` if a matching open backlog item exists. Do not edit unrelated documentation.
