# YouTube Playlist Source Browser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move live YouTube playlist sources into the shared Source Browser with a playlist-specific `Videos` tab.

**Architecture:** `/analysis/+page.svelte` and `ReportSourceSurface` remain the data owners. `SourceBrowserShell` gets playlist detail and playlist callbacks through props, owns only local tab state, and renders a new leaf `YoutubePlaylistVideosView` for playlist membership rows. Source groups and saved run snapshots stay on their existing reader paths.

**Tech Stack:** Svelte 5 runes, SvelteKit, Vitest raw-component contract tests, existing Tauri v2 YouTube playlist detail commands.

---

## Reference Documents

- Spec: `docs/superpowers/specs/2026-05-30-youtube-playlist-source-browser-design.md`
- Existing source browser model: `src/lib/source-browser-model.ts`
- Existing shell: `src/lib/components/analysis/source-browser-shell.svelte`
- Existing route surface: `src/lib/components/analysis/report-source-surface.svelte`
- Existing playlist reader to replace: `src/lib/components/analysis/youtube-playlist-reader.svelte`
- Existing metadata view: `src/lib/components/analysis/source-metadata-view.svelte`
- Existing items view: `src/lib/components/analysis/universal-items-view.svelte`
- YouTube types: `src/lib/types/youtube.ts`

## File Map

Create:

- `src/lib/components/analysis/youtube-playlist-videos-view.svelte`: playlist-specific `Videos` tab leaf. It renders already-loaded playlist detail and contextual playlist/video callbacks. It does not own tab state, route state, source jobs, source activity, or data loading.
- `src/lib/youtube-source-policy.ts`: tiny provider-policy helper for retryable YouTube availability statuses, typed against the existing `YoutubeAvailabilityStatus` union.
- `src/lib/youtube-source-policy.test.ts`: focused unit coverage for the YouTube retryability helper.

Modify:

- `src/lib/source-browser-model.ts`: add `videos` tab id, label, playlist tab derivation, playlist smart default, and playlist shell applicability.
- `src/lib/source-browser-model.test.ts`: add playlist tabs and transition matrix tests.
- `src/lib/components/analysis/source-browser-shell.svelte`: accept `youtubePlaylistDetail`, render `YoutubePlaylistVideosView` for `videos`, pass playlist callbacks, and pass playlist-specific empty copy to `UniversalItemsView`.
- `src/lib/components/analysis/source-browser-shell.test.ts`: assert shell still has no direct API calls and renders playlist videos leaf.
- `src/lib/components/analysis/report-source-surface.svelte`: route YouTube playlist live sources into `SourceBrowserShell` and remove the direct playlist reader branch.
- `src/lib/analysis-source-readers.test.ts`: update contract tests from legacy playlist reader to Source Browser playlist contracts.
- `src/lib/components/analysis/source-metadata-view.svelte`: accept `youtubePlaylistDetail` and render optional-safe playlist metadata fields without raw JSON.
- `src/lib/components/analysis/universal-items-view.svelte`: add optional `emptyDescription` prop for playlist-specific empty copy.

Delete:

- `src/lib/components/analysis/youtube-playlist-reader.svelte`: remove after `YoutubePlaylistVideosView` and shell wiring replace its only route usage. If a local reference remains, stop and remove that reference rather than leaving a stale activity-owning reader.

## Review Slices

1. Model tabs and reconciliation.
2. Playlist `Videos` leaf.
3. Shell and route wiring.
4. Metadata and playlist `Items` empty copy.
5. Final verification and manual smoke.

Each slice ends with a commit. Before committing, run the focused tests named in that slice plus `git diff --check`.

Execution tracking rule: after each task, mark the completed steps in this plan
before committing. Include this plan file in the task commit whenever checkbox
state changed.

---

## Slice 1: Model Tabs And Reconciliation

### Task 1.1: Add playlist tab model tests

**Files:**
- Modify: `src/lib/source-browser-model.test.ts`

- [x] **Step 1: Write failing playlist model tests**

In `src/lib/source-browser-model.test.ts`, update the existing tab/default/applicability tests and add a transition matrix test:

```ts
  it("derives canonical tabs for supported source types", () => {
    expect(sourceBrowserTabsForSource(source({ sourceType: "telegram" })).map((tab) => tab.id))
      .toEqual(["timeline", "items", "metadata", "activity"]);
    expect(sourceBrowserTabsForSource(source({ sourceType: "youtube", sourceSubtype: "video" })).map((tab) => tab.id))
      .toEqual(["transcript", "comments", "items", "metadata", "activity"]);
    expect(sourceBrowserTabsForSource(source({ sourceType: "youtube", sourceSubtype: "playlist" })).map((tab) => tab.id))
      .toEqual(["videos", "items", "metadata", "activity"]);
    expect(sourceBrowserTabsForSource(source({ sourceType: "rss", sourceSubtype: "feed" })).map((tab) => tab.id))
      .toEqual(["items", "metadata", "activity"]);
  });

  it("selects smart defaults by canonical tab id", () => {
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "telegram" }))).toBe("timeline");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "youtube", sourceSubtype: "video" }))).toBe("transcript");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "youtube", sourceSubtype: "playlist" }))).toBe("videos");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "forum", sourceSubtype: "thread" }))).toBe("items");
  });

  it("routes Telegram YouTube video and YouTube playlist live sources into the shell", () => {
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "telegram", sourceSubtype: "supergroup" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "youtube", sourceSubtype: "video" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "youtube", sourceSubtype: "playlist" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "rss", sourceSubtype: "feed" }))).toBe(false);
  });

  it("reconciles playlist tab transitions by canonical tab support", () => {
    const youtubeVideo = source({ id: 2, sourceType: "youtube", sourceSubtype: "video" });
    const youtubePlaylist = source({ id: 3, sourceType: "youtube", sourceSubtype: "playlist" });
    const telegram = source({ id: 4, sourceType: "telegram", sourceSubtype: "supergroup" });

    expect(reconcileSourceBrowserTab("metadata", youtubePlaylist)).toBe("metadata");
    expect(reconcileSourceBrowserTab("items", youtubePlaylist)).toBe("items");
    expect(reconcileSourceBrowserTab("activity", youtubePlaylist)).toBe("activity");
    expect(reconcileSourceBrowserTab("transcript", youtubePlaylist)).toBe("videos");
    expect(reconcileSourceBrowserTab("comments", youtubePlaylist)).toBe("videos");
    expect(reconcileSourceBrowserTab("videos", youtubeVideo)).toBe("transcript");
    expect(reconcileSourceBrowserTab("videos", telegram)).toBe("timeline");
  });
```

- [x] **Step 2: Run the failing model test**

Run:

```bash
npm run test -- src/lib/source-browser-model.test.ts
```

Expected: FAIL because `SourceBrowserTabId` does not include `videos`, playlist tabs are not derived, and playlist applicability is still false.

### Task 1.2: Implement playlist tab model

**Files:**
- Modify: `src/lib/source-browser-model.ts`

- [x] **Step 1: Add the `videos` tab id, label, playlist tabs, default, and applicability**

Modify `src/lib/source-browser-model.ts` so the top tab model section reads:

```ts
export type SourceBrowserTabId =
  | "timeline"
  | "transcript"
  | "comments"
  | "videos"
  | "items"
  | "metadata"
  | "activity";

const TAB_LABELS: Record<SourceBrowserTabId, string> = {
  timeline: "Timeline",
  transcript: "Transcript",
  comments: "Comments",
  videos: "Videos",
  items: "Items",
  metadata: "Metadata",
  activity: "Activity",
};
```

Update these functions:

```ts
export function sourceBrowserTabsForSource(source: Pick<Source, "sourceType" | "sourceSubtype">): SourceBrowserTab[] {
  const ids: SourceBrowserTabId[] =
    source.sourceType === "youtube" && source.sourceSubtype === "video"
      ? ["transcript", "comments", "items", "metadata", "activity"]
      : source.sourceType === "youtube" && source.sourceSubtype === "playlist"
        ? ["videos", "items", "metadata", "activity"]
        : source.sourceType === "telegram"
          ? ["timeline", "items", "metadata", "activity"]
          : ["items", "metadata", "activity"];

  return ids.map((id) => ({ id, label: TAB_LABELS[id] }));
}

export function sourceBrowserShellAppliesToSource(source: Pick<Source, "sourceType" | "sourceSubtype">): boolean {
  return source.sourceType === "telegram"
    || (source.sourceType === "youtube" && (source.sourceSubtype === "video" || source.sourceSubtype === "playlist"));
}

export function smartDefaultSourceBrowserTab(source: Pick<Source, "sourceType" | "sourceSubtype">): SourceBrowserTabId {
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") return "transcript";
  if (source.sourceType === "youtube" && source.sourceSubtype === "playlist") return "videos";
  if (source.sourceType === "telegram") return "timeline";
  return "items";
}
```

- [x] **Step 2: Run the model test**

Run:

```bash
npm run test -- src/lib/source-browser-model.test.ts
```

Expected: PASS.

- [x] **Step 3: Commit Slice 1**

Run:

```bash
git diff --check
git add src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts
git commit -m "feat: add playlist source browser tabs"
```

---

## Slice 2: Playlist Videos Leaf

### Task 2.0: Preflight existing YouTube and UI contracts

**Files:**
- Inspect: `src/lib/types/youtube.ts`
- Inspect: `src/lib/types/sources.ts`
- Inspect: `src/lib/components/ui/Badge.svelte`
- Inspect: `src/lib/components/ui/Button.svelte`
- Inspect: `src/lib/components/analysis/youtube-playlist-reader.svelte`

- [x] **Step 1: Verify the DTO and UI fields used by the leaf**

Run:

```bash
rg -n "linkedVideoCount|unavailableCount|durationSeconds|captions: YoutubeContentStatus|comments: YoutubeContentStatus|YoutubeAvailabilityStatus|ariaLabel|variant\\?: BadgeVariant|variant\\?: ButtonVariant" src/lib/types/youtube.ts src/lib/types/sources.ts src/lib/components/ui/Badge.svelte src/lib/components/ui/Button.svelte
```

Expected: output includes `linkedVideoCount`, `unavailableCount`, `durationSeconds`, `captions`, `comments`, the `YoutubeAvailabilityStatus` union, `ariaLabel`, and typed `Badge` / `Button` variants. If this command fails, stop and update the code snippets in this plan to the current field and prop names before continuing.

- [x] **Step 2: Verify the old reader owns activity state that the new leaf must not carry**

Run:

```bash
rg -n "YoutubeSourceActivity|sourceJobs|onCancelSourceJob|onRetryFailed" src/lib/components/analysis/youtube-playlist-reader.svelte
```

Expected: output includes those names in the legacy reader. The new `YoutubePlaylistVideosView` must keep playlist CTAs and row actions, but must not import activity components, accept `sourceJobs`, or accept `onCancelSourceJob`.

### Task 2.1: Add playlist videos contract tests

**Files:**
- Create: `src/lib/youtube-source-policy.test.ts`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [x] **Step 1: Add retryability helper tests**

Create `src/lib/youtube-source-policy.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { isRetryableYoutubeAvailabilityStatus } from "./youtube-source-policy";

describe("youtube source policy", () => {
  it("classifies retryable YouTube availability statuses", () => {
    expect(isRetryableYoutubeAvailabilityStatus("live_ended_transcript_pending")).toBe(true);
    expect(isRetryableYoutubeAvailabilityStatus("no_captions")).toBe(true);
    expect(isRetryableYoutubeAvailabilityStatus("unavailable_unknown")).toBe(true);
    expect(isRetryableYoutubeAvailabilityStatus("available")).toBe(false);
    expect(isRetryableYoutubeAvailabilityStatus("private_or_auth_required")).toBe(false);
    expect(isRetryableYoutubeAvailabilityStatus(null)).toBe(false);
    expect(isRetryableYoutubeAvailabilityStatus(undefined)).toBe(false);
  });
});
```

- [x] **Step 2: Replace the legacy playlist reader raw import**

In `src/lib/analysis-source-readers.test.ts`, replace:

```ts
import youtubePlaylistSource from "./components/analysis/youtube-playlist-reader.svelte?raw";
```

with:

```ts
import youtubePlaylistVideosViewSource from "./components/analysis/youtube-playlist-videos-view.svelte?raw";
```

- [x] **Step 3: Replace legacy playlist reader tests with leaf contract tests**

Replace the two tests named `"keeps YouTube playlist reading playlist-first"` and `"renders YouTube playlist source activity and cancellation"` with:

```ts
  it("renders YouTube playlist videos as a job-free leaf view", () => {
    expect(youtubePlaylistVideosViewSource).toContain('aria-label="YouTube playlist videos"');
    expect(youtubePlaylistVideosViewSource).toContain("playlist.items");
    expect(youtubePlaylistVideosViewSource).toContain("onOpenSource");
    expect(youtubePlaylistVideosViewSource).toContain("onSyncPlaylist");
    expect(youtubePlaylistVideosViewSource).toContain("onRetryFailedPlaylistVideos");
    expect(youtubePlaylistVideosViewSource).toContain("onSyncPlaylistVideo");
    expect(youtubePlaylistVideosViewSource).toContain("onRetryPlaylistVideo");
    expect(youtubePlaylistVideosViewSource).toContain("isRetryableYoutubeAvailabilityStatus");
    expect(youtubePlaylistVideosViewSource).not.toContain("retryableStatuses");
    expect(youtubePlaylistVideosViewSource).not.toContain("SourceActivityView");
    expect(youtubePlaylistVideosViewSource).not.toContain("YoutubeSourceActivity");
    expect(youtubePlaylistVideosViewSource).not.toContain("sourceJobs");
    expect(youtubePlaylistVideosViewSource).not.toContain("onCancelSourceJob");
    expect(youtubePlaylistVideosViewSource).not.toContain("$lib/api/");
    expect(youtubePlaylistVideosViewSource).not.toContain("invoke(");
  });

  it("keeps playlist video opening as source selection instead of nested browsing", () => {
    expect(youtubePlaylistVideosViewSource).toContain("onOpenSource");
    expect(youtubePlaylistVideosViewSource).toContain("videoSourceId");
    expect(youtubePlaylistVideosViewSource).not.toContain("<YoutubeTranscriptReader");
    expect(youtubePlaylistVideosViewSource).not.toContain("<SourceBrowserShell");
    expect(youtubePlaylistVideosViewSource).not.toContain("SourceActivityView");
    expect(youtubePlaylistVideosViewSource).not.toContain("$lib/api/");
  });
```

- [x] **Step 4: Run the failing reader contract test**

Run:

```bash
npm run test -- src/lib/youtube-source-policy.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because `youtube-source-policy.ts` and `youtube-playlist-videos-view.svelte` do not exist and the legacy reader tests no longer match.

### Task 2.2: Create `YoutubePlaylistVideosView`

**Files:**
- Create: `src/lib/youtube-source-policy.ts`
- Create: `src/lib/components/analysis/youtube-playlist-videos-view.svelte`

- [x] **Step 1: Add a typed YouTube retry policy helper**

Create `src/lib/youtube-source-policy.ts`:

```ts
import type { YoutubeAvailabilityStatus } from "$lib/types/sources";

const retryableYoutubeAvailabilityStatuses = new Set<string>([
  "live_ended_transcript_pending",
  "no_captions",
  "unavailable_unknown",
] satisfies YoutubeAvailabilityStatus[]);

export function isRetryableYoutubeAvailabilityStatus(status: string | null | undefined): boolean {
  return status !== null && status !== undefined && retryableYoutubeAvailabilityStatuses.has(status);
}
```

- [x] **Step 2: Add the playlist videos leaf component**

Create `src/lib/components/analysis/youtube-playlist-videos-view.svelte`:

```svelte
<script lang="ts">
  import { ExternalLink, RefreshCw, RotateCcw, Video } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import { isRetryableYoutubeAvailabilityStatus } from "$lib/youtube-source-policy";
  import type { YoutubePlaylistDetail, YoutubePlaylistItemDetail } from "$lib/types/youtube";

  let {
    sourceTitle,
    playlist,
    loading,
    formatTimestamp,
    onOpenSource,
    onSyncPlaylist,
    onRetryFailedPlaylistVideos,
    onSyncPlaylistVideo,
    onRetryPlaylistVideo,
  }: {
    sourceTitle: string;
    playlist: YoutubePlaylistDetail | null;
    loading: boolean;
    formatTimestamp: (value: number | null) => string;
    onOpenSource: (sourceId: number) => void | Promise<void>;
    onSyncPlaylist: () => void | Promise<void>;
    onRetryFailedPlaylistVideos: () => void | Promise<void>;
    onSyncPlaylistVideo: (videoSourceId: number) => void | Promise<void>;
    onRetryPlaylistVideo: (videoSourceId: number) => void | Promise<void>;
  } = $props();

  const summary = $derived(playlist?.summary ?? null);
  const items = $derived(playlist?.items ?? []);

  function availabilityLabel(value: string | null | undefined) {
    return value ? value.replaceAll("_", " ") : "unknown";
  }

  function formatDuration(value: number | null) {
    if (value === null) return "";
    const minutes = Math.floor(value / 60);
    const seconds = value % 60;
    return `${minutes}:${String(seconds).padStart(2, "0")}`;
  }

  function canSyncItem(item: YoutubePlaylistItemDetail) {
    return item.videoSourceId !== null && !item.isRemovedFromPlaylist;
  }

  function canRetryItem(item: YoutubePlaylistItemDetail) {
    return canSyncItem(item) && isRetryableYoutubeAvailabilityStatus(item.availabilityStatus);
  }
</script>

<section class="youtube-playlist-videos-view" aria-label="YouTube playlist videos">
  <div class="playlist-header">
    <div class="playlist-title">
      <span class="eyebrow">YouTube playlist</span>
      <h3>{summary?.title ?? sourceTitle}</h3>
      <div class="playlist-meta">
        <Badge variant="info">{summary?.channelHandle ?? summary?.channelTitle ?? "YouTube"}</Badge>
        <Badge variant="neutral">{summary?.videoCount ?? playlist?.items.length ?? 0} videos</Badge>
        <Badge variant="neutral">{summary?.linkedVideoCount ?? 0} linked</Badge>
        {#if (summary?.unavailableCount ?? 0) > 0}
          <Badge variant="warning">{summary?.unavailableCount} unavailable</Badge>
        {/if}
      </div>
    </div>
    <div class="playlist-actions">
      <Button size="sm" variant="secondary" onclick={onSyncPlaylist}>
        <RefreshCw size={14} aria-hidden="true" /> Sync all
      </Button>
      <Button size="sm" variant="secondary" onclick={onRetryFailedPlaylistVideos}>
        <RotateCcw size={14} aria-hidden="true" /> Retry failed
      </Button>
    </div>
  </div>

  {#if loading}
    <StatusMessage tone="muted" surface={false}>Loading YouTube playlist...</StatusMessage>
  {:else if !playlist || !summary}
    <StatusMessage tone="muted" surface={false}>YouTube playlist detail is not loaded.</StatusMessage>
  {:else}
    <div class="playlist-status">
      {@render detailField("Captions", `${summary.captions.label} - ${formatTimestamp(summary.captions.lastSyncedAt)}`)}
      {@render detailField("Comments", `${summary.comments.label} - ${formatTimestamp(summary.comments.lastSyncedAt)}`)}
      {@render detailField("Availability", availabilityLabel(summary.availabilityStatus))}
    </div>

    {#if playlist.items.length === 0}
      <StatusMessage tone="muted" surface={false}>
        No linked videos are available for this playlist. Sync the playlist to load video rows.
      </StatusMessage>
    {:else}
      <div class="playlist-items">
        {#each items as item (item.videoId)}
          <article class:removed={item.isRemovedFromPlaylist} class="playlist-row">
            <div class="playlist-thumb" aria-hidden="true">
              {#if item.thumbnailUrl}
                <img src={item.thumbnailUrl} alt="" loading="lazy" />
              {:else}
                <Video size={18} />
              {/if}
            </div>
            <div class="playlist-copy">
              <div class="playlist-title-line">
                <strong>{item.position !== null ? `${item.position}. ` : ""}{item.title ?? item.videoId}</strong>
                {#if item.durationSeconds !== null}
                  <span>{formatDuration(item.durationSeconds)}</span>
                {/if}
              </div>
              <div class="playlist-meta">
                <Badge variant={item.availabilityStatus === "available" ? "neutral" : "warning"}>
                  {availabilityLabel(item.availabilityStatus)}
                </Badge>
                <Badge variant={item.captions.state === "synced" ? "success" : item.captions.state === "unavailable" ? "warning" : "neutral"}>
                  {item.captions.label}
                </Badge>
                <Badge variant={item.comments.state === "synced" ? "success" : "neutral"}>
                  {item.comments.label}
                </Badge>
                {#if item.publishedAt !== null}
                  <span>{formatTimestamp(item.publishedAt)}</span>
                {/if}
              </div>
            </div>
            <div class="playlist-row-actions">
              <Button
                size="sm"
                variant="ghost"
                ariaLabel="Open video source"
                title="Open video source"
                disabled={item.videoSourceId === null}
                onclick={() => item.videoSourceId !== null && onOpenSource(item.videoSourceId)}
              >
                <ExternalLink size={15} aria-hidden="true" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                ariaLabel="Sync this video"
                title="Sync this video"
                disabled={!canSyncItem(item)}
                onclick={() => item.videoSourceId !== null && onSyncPlaylistVideo(item.videoSourceId)}
              >
                <RefreshCw size={15} aria-hidden="true" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                ariaLabel="Retry this video"
                title="Retry this video"
                disabled={!canRetryItem(item)}
                onclick={() => item.videoSourceId !== null && onRetryPlaylistVideo(item.videoSourceId)}
              >
                <RotateCcw size={15} aria-hidden="true" />
              </Button>
            </div>
          </article>
        {/each}
      </div>
    {/if}
  {/if}
</section>

{#snippet detailField(label: string, value: string)}
  <div class="detail-field">
    <span>{label}</span>
    <strong>{value}</strong>
  </div>
{/snippet}

<style>
  .youtube-playlist-videos-view {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
  }

  .playlist-header,
  .playlist-row {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .playlist-header {
    justify-content: space-between;
  }

  .playlist-title {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  h3 {
    margin: 0;
  }

  .playlist-meta,
  .playlist-actions,
  .playlist-row-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    align-items: center;
  }

  .playlist-status {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.6rem;
  }

  .detail-field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.75rem;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--panel-strong);
    min-width: 0;
  }

  .detail-field span,
  .playlist-meta span {
    color: var(--muted);
    font-size: 0.75rem;
  }

  .detail-field strong,
  .playlist-title-line strong {
    overflow-wrap: anywhere;
  }

  .playlist-items {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .playlist-row {
    padding: 0.65rem;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--panel);
  }

  .playlist-row.removed {
    opacity: 0.72;
  }

  .playlist-thumb {
    flex: 0 0 4.5rem;
    width: 4.5rem;
    aspect-ratio: 16 / 9;
    border-radius: 6px;
    overflow: hidden;
    background: var(--panel-hover);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--muted);
  }

  .playlist-thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .playlist-copy {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .playlist-title-line {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
  }

  @media (max-width: 840px) {
    .playlist-header,
    .playlist-row {
      flex-direction: column;
    }

    .playlist-status {
      grid-template-columns: 1fr;
    }

    .playlist-thumb {
      width: 100%;
      flex-basis: auto;
    }
  }
</style>
```

- [x] **Step 3: Run the reader contract test**

Run:

```bash
npm run test -- src/lib/youtube-source-policy.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS for the new leaf tests. Other playlist routing tests may still fail until Slice 3; if only the route tests fail, continue to Slice 3 before committing. If the leaf tests fail, fix `YoutubePlaylistVideosView` or `src/lib/youtube-source-policy.ts` first.

---

## Slice 3: Shell And Route Wiring

### Task 3.1: Add shell and route wiring tests

**Files:**
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/components/analysis/source-browser-shell.test.ts`

- [x] **Step 1: Update route contract tests**

In `src/lib/analysis-source-readers.test.ts`, update the first two route tests:

```ts
  it("replaces transitional source panels in ReportSourceSurface", () => {
    expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
    expect(reportSourceSurfaceSource).toContain("<TelegramTimelineReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubeTranscriptReader");
    expect(reportSourceSurfaceSource).toContain("<SourceGroupReader");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistReader");
    expect(reportSourceSurfaceSource).not.toContain("<SourceContextPanel");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubeSourceDetail");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistDetail");
    expect(reportSourceSurfaceSource).not.toContain("<RunCompanionTabs");
  });

  it("routes Telegram YouTube video and YouTube playlist live sources through SourceBrowserShell", () => {
    expect(reportSourceSurfaceSource).toContain("sourceBrowserShellAppliesToSource(currentSource)");
    expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
    expect(reportSourceSurfaceSource).toContain("{youtubePlaylistDetail}");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistReader");
  });
```

Add a shell playlist test:

```ts
  it("renders YouTube playlist videos through SourceBrowserShell", () => {
    expect(sourceBrowserShellSource).toContain("<YoutubePlaylistVideosView");
    expect(sourceBrowserShellSource).toContain('activeTab === "videos"');
    expect(sourceBrowserShellSource).toContain("youtubePlaylistDetail");
    expect(sourceBrowserShellSource).toContain("onRetryFailedPlaylistVideos");
    expect(sourceBrowserShellSource).toContain("onRetryPlaylistVideo");
  });
```

- [x] **Step 2: Update shell architecture test**

In `src/lib/components/analysis/source-browser-shell.test.ts`, update the second test:

```ts
  it("renders provider readers and playlist videos through route-owned props", () => {
    expect(shellSource).toContain("<TelegramTimelineReader");
    expect(shellSource).toContain("<YoutubeTranscriptReader");
    expect(shellSource).toContain("<YoutubePlaylistVideosView");
    expect(shellSource).toContain("timeline");
    expect(shellSource).toContain("transcript");
    expect(shellSource).toContain("videos");
    expect(shellSource).toContain("youtubePlaylistDetail");
  });
```

- [x] **Step 3: Run failing route/shell tests**

Run:

```bash
npm run test -- src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because shell does not yet accept playlist detail or render `YoutubePlaylistVideosView`, and route still has a direct playlist branch.

### Task 3.2: Wire playlist props through `SourceBrowserShell`

**Files:**
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`

- [x] **Step 1: Add imports and props**

In `src/lib/components/analysis/source-browser-shell.svelte`, add:

```svelte
  import YoutubePlaylistVideosView from "$lib/components/analysis/youtube-playlist-videos-view.svelte";
```

Update the YouTube type import:

```ts
  import type { YoutubePlaylistDetail, YoutubeVideoDetail } from "$lib/types/youtube";
```

Add props to `type Props`:

```ts
    youtubePlaylistDetail: YoutubePlaylistDetail | null;
    onOpenSource: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylist: (sourceId: number) => void | Promise<void>;
    onRetryFailedYoutubePlaylistVideos: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onRetryYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
```

Destructure those props in the `$props()` block:

```ts
    youtubePlaylistDetail,
    onOpenSource,
    onSyncYoutubePlaylist,
    onRetryFailedYoutubePlaylistVideos,
    onSyncYoutubePlaylistVideo,
    onRetryYoutubePlaylistVideo,
```

- [x] **Step 2: Add the `videos` branch**

Add this branch before `activity`:

```svelte
  {:else if activeTab === "videos"}
    <YoutubePlaylistVideosView
      sourceTitle={source.title ?? source.externalId}
      playlist={youtubePlaylistDetail}
      loading={loadingYoutubeDetail}
      {formatTimestamp}
      onOpenSource={onOpenSource}
      onSyncPlaylist={() => onSyncYoutubePlaylist(source.id)}
      onRetryFailedPlaylistVideos={() => onRetryFailedYoutubePlaylistVideos(source.id)}
      onSyncPlaylistVideo={(videoSourceId) => onSyncYoutubePlaylistVideo(source.id, videoSourceId)}
      onRetryPlaylistVideo={(videoSourceId) => onRetryYoutubePlaylistVideo(source.id, videoSourceId)}
    />
```

- [x] **Step 3: Keep Activity wired to playlist source jobs**

Do not add playlist jobs to `YoutubePlaylistVideosView`. In this slice, Activity renders detailed playlist job state, cancellation, and the existing generic source sync/YouTube metadata actions only. Playlist-level CTAs (`onSyncYoutubePlaylist` and `onRetryFailedYoutubePlaylistVideos`) remain in `Videos` and are not duplicated in Activity.

The existing Activity branch must still pass:

```svelte
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

### Task 3.3: Route playlists through the shell

**Files:**
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Delete: `src/lib/components/analysis/youtube-playlist-reader.svelte`

- [x] **Step 1: Remove the old playlist reader import and type alias**

In `src/lib/components/analysis/report-source-surface.svelte`, delete:

```svelte
  import YoutubePlaylistReader from "$lib/components/analysis/youtube-playlist-reader.svelte";
```

Delete:

```ts
  type YoutubePlaylistReaderProps = ComponentProps<typeof YoutubePlaylistReader>;
```

Delete the `ComponentProps` import if it becomes unused:

```ts
  import type { ComponentProps } from "svelte";
```

Update the YouTube type import:

```ts
  import type { YoutubePlaylistDetail, YoutubeVideoDetail } from "$lib/types/youtube";
```

Change the prop type:

```ts
    youtubePlaylistDetail: YoutubePlaylistDetail | null;
```

- [x] **Step 2: Pass playlist props into `SourceBrowserShell`**

Inside the `<SourceBrowserShell ... />` call, add:

```svelte
        {youtubePlaylistDetail}
        {onOpenSource}
        {onSyncYoutubePlaylist}
        onRetryFailedYoutubePlaylistVideos={onRetryFailedYoutubePlaylistVideos}
        {onSyncYoutubePlaylistVideo}
        {onRetryYoutubePlaylistVideo}
```

Keep the existing playlist-level retry callback named:

```ts
    onRetryFailedYoutubePlaylistVideos: (sourceId: number) => void | Promise<void>;
```

Do not introduce a second alias named `onRetryFailed`; row-level retry stays on `onRetryYoutubePlaylistVideo`.

- [x] **Step 3: Remove the direct playlist branch**

In the `{:else}` branch under `analysisScope === "single_source"`, remove the entire direct playlist reader block:

```svelte
        {#if currentSource.sourceType === "youtube" && currentSource.sourceSubtype === "playlist"}
          ...
          <YoutubePlaylistReader ... />
        {:else}
          <StatusMessage tone="muted" surface={false}>This source type is not browsable yet.</StatusMessage>
        {/if}
```

Replace it with:

```svelte
        <StatusMessage tone="muted" surface={false}>This source type is not browsable yet.</StatusMessage>
```

Because `sourceBrowserShellAppliesToSource(currentSource)` now handles playlists, this fallback remains for unsupported future source types only.

- [x] **Step 4: Delete the legacy reader file**

Delete:

```bash
git rm src/lib/components/analysis/youtube-playlist-reader.svelte
```

If `git rm` reports remaining references, run:

```bash
rg -n "youtube-playlist-reader|YoutubePlaylistReader" src
```

Remove those references before retrying the delete.

- [x] **Step 5: Run shell and reader contract tests**

Run:

```bash
npm run test -- src/lib/source-browser-model.test.ts src/lib/youtube-source-policy.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [x] **Step 6: Run Svelte check**

Run:

```bash
npm run check
```

Expected: PASS.

- [x] **Step 7: Commit Slices 2 and 3**

Run:

```bash
git diff --check
git add -A src/lib/youtube-source-policy.ts src/lib/youtube-source-policy.test.ts src/lib/components/analysis/youtube-playlist-videos-view.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/components/analysis/youtube-playlist-reader.svelte
git commit -m "feat: route playlists through source browser"
```

If `youtube-playlist-reader.svelte` was deleted with `git rm`, the `git add` command will stage the deletion.

---

## Slice 4: Playlist Metadata And Items Empty Copy

### Task 4.1: Add metadata and empty-copy tests

**Files:**
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Add contract expectations for playlist metadata and empty copy**

In `src/lib/analysis-source-readers.test.ts`, extend the metadata and items tests:

```ts
  it("renders universal Items as a loaded-window browser", () => {
    expect(universalItemsViewSource).toContain("Search loaded items");
    expect(universalItemsViewSource).toContain("All");
    expect(universalItemsViewSource).toContain("Load more items");
    expect(universalItemsViewSource).toContain("Unknown item kind");
    expect(universalItemsViewSource).toContain("emptyDescription");
  });

  it("renders source metadata in structured sections with bounded raw JSON", () => {
    expect(sourceMetadataViewSource).toContain("Summary");
    expect(sourceMetadataViewSource).toContain("Source state");
    expect(sourceMetadataViewSource).toContain("Technical");
    expect(sourceMetadataViewSource).toContain("<RawJsonPanel");
    expect(sourceMetadataViewSource).toContain("youtubePlaylistDetail");
    expect(sourceMetadataViewSource).toContain("Playlist ID");
    expect(sourceMetadataViewSource).toContain("Linked videos");
    expect(sourceMetadataViewSource).not.toContain("items.raw_data_zstd");
    expect(rawJsonPanelSource).toContain("Show raw JSON");
    expect(rawJsonPanelSource).toContain("Copy");
    expect(rawJsonPanelSource).toContain("Large payload");
  });
```

Add shell expectations:

```ts
  it("passes playlist detail into metadata and playlist-specific empty copy into Items", () => {
    expect(sourceBrowserShellSource).toContain("youtubePlaylistDetail={youtubePlaylistDetail}");
    expect(sourceBrowserShellSource).toContain("Playlist videos live in the Videos tab");
    expect(sourceBrowserShellSource).toContain("emptyDescription=");
  });
```

- [ ] **Step 2: Run failing contract tests**

Run:

```bash
npm run test -- src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because `UniversalItemsView` has no `emptyDescription` prop and `SourceMetadataView` has no playlist detail prop.

### Task 4.2: Add playlist empty copy to `UniversalItemsView`

**Files:**
- Modify: `src/lib/components/analysis/universal-items-view.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`

- [ ] **Step 1: Add optional `emptyDescription` prop**

In `src/lib/components/analysis/universal-items-view.svelte`, add `emptyDescription` to props:

```ts
  let {
    items,
    loading,
    hasMore,
    emptyDescription = "No loaded items are available for this source window.",
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceItem[];
    loading: boolean;
    hasMore: boolean;
    emptyDescription?: string;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();
```

Change the empty state:

```svelte
  {#if !loading && items.length === 0}
    <EmptyState description={emptyDescription} />
```

- [ ] **Step 2: Pass playlist-specific copy from shell**

In `src/lib/components/analysis/source-browser-shell.svelte`, update the `items` branch:

```svelte
  {:else if activeTab === "items"}
    <UniversalItemsView
      items={sourceItems}
      loading={loadingItems}
      hasMore={sourceItemsHasMore}
      emptyDescription={source.sourceType === "youtube" && source.sourceSubtype === "playlist"
        ? "Playlist videos live in the Videos tab. This Items tab only shows generic archived items loaded for this playlist source."
        : "No loaded items are available for this source window."}
      {formatTimestamp}
      onLoadMore={onLoadMoreSourceItems}
    />
```

### Task 4.3: Add playlist metadata to `SourceMetadataView`

**Files:**
- Modify: `src/lib/components/analysis/source-metadata-view.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`

- [ ] **Step 1: Add playlist detail prop**

In `src/lib/components/analysis/source-metadata-view.svelte`, change the YouTube type import:

```ts
  import type { YoutubePlaylistDetail, YoutubeVideoDetail } from "$lib/types/youtube";
```

Add `youtubePlaylistDetail` to props:

```ts
    youtubePlaylistDetail = null,
```

and to the type:

```ts
    youtubePlaylistDetail?: YoutubePlaylistDetail | null;
```

Add derived helpers:

```ts
  const playlistSummary = $derived(youtubePlaylistDetail?.summary ?? null);
  const youtubeSummary = $derived(summary ?? playlistSummary);
```

- [ ] **Step 2: Use video or playlist summary in Summary and Source state**

Replace summary title/canonical URL reads with `youtubeSummary` where source-level YouTube summary is appropriate:

```svelte
<dd>{textValue(youtubeSummary?.title ?? source.title)}</dd>
```

For canonical URL:

```svelte
{#if youtubeSummary?.canonicalUrl ?? youtubeMetadata?.canonicalUrl}
  <a href={youtubeSummary?.canonicalUrl ?? youtubeMetadata?.canonicalUrl} target="_blank" rel="noreferrer">
    {youtubeSummary?.canonicalUrl ?? youtubeMetadata?.canonicalUrl}
  </a>
{:else}
  Not available
{/if}
```

Use `youtubeSummary?.captions` and `youtubeSummary?.comments` for badges.

- [ ] **Step 3: Render playlist Source state fields without item raw payloads**

Inside the Source state `<dl>`, add a playlist branch before the existing generic YouTube summary branch:

```svelte
      {:else if source.sourceType === "youtube" && source.sourceSubtype === "playlist" && playlistSummary}
        <div>
          <dt>Captions</dt>
          <dd>{playlistSummary.captions.label}</dd>
        </div>
        <div>
          <dt>Comments</dt>
          <dd>{playlistSummary.comments.label}</dd>
        </div>
        <div>
          <dt>Videos</dt>
          <dd>{textValue(playlistSummary.videoCount)}</dd>
        </div>
        <div>
          <dt>Linked videos</dt>
          <dd>{textValue(playlistSummary.linkedVideoCount)}</dd>
        </div>
        <div>
          <dt>Unavailable videos</dt>
          <dd>{textValue(playlistSummary.unavailableCount)}</dd>
        </div>
        <div>
          <dt>Availability</dt>
          <dd>{textValue(playlistSummary.availabilityStatus)}</dd>
        </div>
```

- [ ] **Step 4: Render playlist Technical fields**

Inside the Technical `<dl>`, add this branch before `{#if youtubeMetadata}`:

```svelte
      {#if source.sourceType === "youtube" && source.sourceSubtype === "playlist"}
        <div>
          <dt>Playlist ID</dt>
          <dd>{source.externalId}</dd>
        </div>
        <div>
          <dt>Channel title</dt>
          <dd>{textValue(playlistSummary?.channelTitle)}</dd>
        </div>
        <div>
          <dt>Channel handle</dt>
          <dd>{textValue(playlistSummary?.channelHandle)}</dd>
        </div>
        <div>
          <dt>Canonical URL</dt>
          <dd>{textValue(playlistSummary?.canonicalUrl)}</dd>
        </div>
      {:else if youtubeMetadata}
```

Close the branch by changing the existing `{/if}` after video metadata fields so it still matches the new `{:else if youtubeMetadata}` block.

- [ ] **Step 5: Hide Raw JSON for playlists only**

Change the raw JSON section condition so existing non-playlist YouTube raw JSON behavior is preserved while playlist metadata never exposes raw playlist JSON or item payloads:

```svelte
  {#if source.sourceType === "youtube" && source.sourceSubtype !== "playlist"}
    <section class="metadata-section" aria-labelledby="metadata-raw-title">
      <h4 id="metadata-raw-title">Raw JSON</h4>
      <RawJsonPanel value={rawJson} />
    </section>
  {/if}
```

- [ ] **Step 6: Pass playlist detail into metadata from shell**

In `src/lib/components/analysis/source-browser-shell.svelte`, update the metadata branch:

```svelte
    <SourceMetadataView
      source={source}
      youtubeVideoDetail={youtubeVideoDetail}
      youtubePlaylistDetail={youtubePlaylistDetail}
      sourceTopics={sourceTopics}
      loading={loadingYoutubeDetail}
      {formatTimestamp}
      onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
    />
```

- [ ] **Step 7: Run focused frontend tests and Svelte check**

Run:

```bash
npm run test -- src/lib/analysis-source-readers.test.ts src/lib/components/analysis/source-browser-shell.test.ts
npm run check
```

Expected: PASS.

- [ ] **Step 8: Commit Slice 4**

Run:

```bash
git diff --check
git add src/lib/components/analysis/universal-items-view.svelte src/lib/components/analysis/source-metadata-view.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: add playlist browser metadata polish"
```

---

## Slice 5: Final Verification And Manual Smoke

### Task 5.1: Run full automated verification

**Files:**
- No source edits expected.

- [ ] **Step 1: Run focused browser tests**

Run:

```bash
npm run test -- src/lib/source-browser-model.test.ts src/lib/youtube-source-policy.test.ts src/lib/components/analysis/source-browser-shell.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run full verification**

Run:

```bash
npm run verify
```

Expected: PASS.

### Task 5.2: Run Tauri acceptance smoke

**Files:**
- No source edits expected unless smoke finds a regression.

- [ ] **Step 1: Start the dev app**

Run:

```bash
npm run tauri dev
```

Expected: Tauri app opens and Vite serves `/analysis`.

- [ ] **Step 2: Seed analysis redesign fixtures**

Use the existing Tauri fixture command path from prior smoke:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

Expected: fixture Telegram sources, YouTube video, YouTube playlist, and saved snapshot runs are available.

- [ ] **Step 3: Verify playlist live browser behavior**

In `/analysis`, select `__analysis_redesign_fixture__ YouTube Playlist`.

Expected:

- Source Browser tabs show `Videos`, `Items`, `Metadata`, `Activity`.
- `Videos` is selected by default.
- `Videos` shows playlist rows and contextual `Sync all` / `Retry failed`.
- Detailed source jobs are not visible in `Videos`.
- `Activity` shows source job/status/cancellation area and does not duplicate playlist `Sync all` / `Retry failed` CTAs.
- `Items` shows `Search loaded items`; if empty, the copy says:
  `Playlist videos live in the Videos tab. This Items tab only shows generic archived items loaded for this playlist source.`
- `Metadata` shows Summary, Source state, Technical, playlist id, linked videos, and no Raw JSON panel for playlist.
- `Open video source` opens the linked YouTube video source, not a nested view.

- [ ] **Step 4: Verify preserved old paths**

Expected:

- Telegram live source still opens `Timeline`.
- YouTube video live source still opens `Transcript`.
- Source group still renders the existing group reader.
- Saved run snapshot still renders the existing snapshot reader.

- [ ] **Step 5: Stop the dev app**

Stop the Tauri dev process and confirm no `cargo`, `rustc`, or `extractum` dev process is left running.

### Task 5.3: Record final verification commit

**Files:**
- Modify: `docs/superpowers/specs/2026-05-30-youtube-playlist-source-browser-design.md`

- [ ] **Step 1: Update the spec status**

Change the header status from:

```markdown
> Status: approved design, pending implementation plan
```

to:

```markdown
> Status: implemented on 2026-05-30; pending merge
```

- [ ] **Step 2: Run final clean checks**

Run:

```bash
git diff --check
git status --short --branch
```

Expected: no whitespace errors; only the spec status change is unstaged.

- [ ] **Step 3: Commit final verification marker**

Run:

```bash
git add docs/superpowers/specs/2026-05-30-youtube-playlist-source-browser-design.md
git commit -m "test: verify playlist source browser"
```

## Final Acceptance

- Live YouTube playlist sources enter `SourceBrowserShell`.
- Playlist tabs are `Videos | Items | Metadata | Activity`.
- `Videos` consumes only already-loaded playlist detail and callback props.
- `Videos` owns no tab, route selection, job, activity, or data loading state.
- Playlist-level retry and row-level retry use distinct callback props.
- `Open video source` changes selected source and does not nest video browsing.
- `Items` keeps loaded-window semantics and has playlist-specific empty copy.
- `Metadata` uses optional-safe playlist detail/source fields and does not expose playlist raw JSON or item raw payloads.
- Detailed playlist jobs and cancellation render in `Activity`, not in `Videos`.
- Playlist-level CTAs render in `Videos` and are not duplicated in `Activity` in this slice.
- Source groups and saved snapshots remain on their existing readers.
- Focused tests and `npm run verify` pass.
