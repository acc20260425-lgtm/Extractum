# Library Add Source MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Library prototype Add placeholder with a centered Add Source modal that supports YouTube URL import, YouTube playlist-to-video import, and Telegram dialog import.

**Architecture:** Keep `/projects/library` on the existing Library catalog workflow and add Library-owned modal components around existing APIs. Put reusable UI primitives behind `extractum-ui` wrappers, keep provider-specific logic in small TypeScript model/workflow helpers, and keep backend changes out of scope.

**Tech Stack:** Svelte 5, SvelteKit/Tauri SPA, TypeScript, Vitest, shadcn-svelte/Bits UI through `extractum-ui`, existing Tauri invoke wrappers.

---

## Approved Design Inputs

- Spec: `docs/superpowers/specs/2026-06-13-library-add-source-mvp-design.md`
- Current Library screen: `src/lib/components/research-projects/LibraryScreen.svelte`
- Current Library toolbar: `src/lib/components/research-projects/LibraryWorkspace.svelte`
- Current Library catalog model: `src/lib/ui/library-catalog-model.ts`
- Current Library workflow: `src/lib/ui/library-catalog-workflow.ts`
- Existing source APIs: `src/lib/api/sources.ts`
- Existing account APIs: `src/lib/api/accounts.ts`
- Existing YouTube detail API: `src/lib/api/youtube-detail.ts`
- Existing shadcn dialog primitives: `src/lib/components/ui/dialog/index.ts`
- Existing extractum wrappers: `src/lib/components/extractum-ui/index.ts`

## Scope Check

This plan is a frontend Library Add Source MVP. It uses existing backend commands:

- `preview_youtube_source`;
- `add_youtube_source`;
- `get_youtube_playlist_detail`;
- `list_accounts`;
- `tg_get_account_statuses`;
- `list_telegram_sources`;
- `add_telegram_source`.

No new Rust command, durable table, YouTube channel ingestion, or Telegram URL import is part of this plan.

## File Structure

### UI Wrapper Layer

- Create: `src/lib/components/extractum-ui/Dialog.svelte`
  - Centered dialog wrapper around `$lib/components/ui/dialog/index.js`.
- Create: `src/lib/components/extractum-ui/StatusMessage.svelte`
  - Product wrapper around the existing status message primitive so Library Add Source components do not import from `$lib/components/ui/*` directly.
- Modify: `src/lib/components/extractum-ui/index.ts`
  - Export `ExtractumDialog`, `ExtractumStatusMessage`, and dialog subparts.

### Library Add Source Model And Workflow

- Create: `src/lib/ui/library-add-source-model.ts`
  - Pure helpers for YouTube URL classification, playlist import rows, selection limits, result summaries, and Telegram dialog add input.
- Create: `src/lib/ui/library-add-source-model.test.ts`
  - Unit coverage for the pure helpers.
- Create: `src/lib/ui/library-add-source-workflow.ts`
  - Sequential YouTube playlist video add runner.
- Create: `src/lib/ui/library-add-source-workflow.test.ts`
  - Unit coverage for sequential add success, disabled selection, and partial failure.

### Library Add Source Components

- Create: `src/lib/components/research-projects/LibraryAddSourceDialog.svelte`
  - Centered modal shell with top-level provider tabs.
- Create: `src/lib/components/research-projects/LibraryYoutubeAddPanel.svelte`
  - YouTube provider panel with inner mode tabs.
- Create: `src/lib/components/research-projects/LibraryYoutubeSmartImport.svelte`
  - URL input, YouTube detection, preview, add.
- Create: `src/lib/components/research-projects/LibraryYoutubePlaylistImport.svelte`
  - Existing playlist selector, playlist item selection, add selected.
- Create: `src/lib/components/research-projects/LibraryTelegramDialogImport.svelte`
  - Account selector, dialog loader, dialog filtering, add selected.
- Modify: `src/lib/components/research-projects/LibraryScreen.svelte`
  - Replace Add prototype feedback with the modal.

### Contract Tests

- Modify: `src/lib/research-projects-import-boundary.test.ts`
  - Require dialog wrapper to live in `extractum-ui`.
- Modify: `src/lib/library-prototype-contract.test.ts`
  - Prove Add opens the dialog and no longer uses prototype feedback.
- Create: `src/lib/library-add-source-contract.test.ts`
  - Raw-source contract checks for the Add Source components and wrapper imports.

---

## Task 0: Baseline

**Files:**
- No file changes.

- [ ] **Step 1: Confirm branch and clean tracked worktree**

Run:

```powershell
git status --short --branch
```

Expected: branch is `main` and no tracked files are modified. If untracked local scratch files appear, do not stage them.

- [ ] **Step 2: Run focused Library baseline tests**

Run:

```powershell
npm.cmd run test -- src/lib/library-prototype-contract.test.ts src/lib/research-projects-import-boundary.test.ts src/lib/ui/library-catalog-model.test.ts src/lib/ui/library-catalog-workflow.test.ts
```

Expected: PASS. If this fails before changes, stop and record the existing failure in this plan.

- [ ] **Step 3: Run Svelte check baseline**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

---

## Task 1: Add Extractum Dialog And Status Message Wrappers

**Files:**
- Create: `src/lib/components/extractum-ui/Dialog.svelte`
- Create: `src/lib/components/extractum-ui/StatusMessage.svelte`
- Modify: `src/lib/components/extractum-ui/index.ts`
- Modify: `src/lib/research-projects-import-boundary.test.ts`

- [ ] **Step 1: Extend the import-boundary test first**

Modify `src/lib/research-projects-import-boundary.test.ts`.

In the test named `"allows lower-level library imports only in the product wrapper layer"`, add these expectations after the existing sheet expectation:

```ts
    expect(wrapperSources).toContain("$lib/components/ui/dialog/index.js");
    expect(wrapperSources).toContain("$lib/components/ui/StatusMessage.svelte");
    expect(wrapperSources).toContain("ExtractumDialog");
    expect(wrapperSources).toContain("ExtractumStatusMessage");
```

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
```

Expected: FAIL because no `ExtractumDialog` or `ExtractumStatusMessage` wrapper exists yet.

- [ ] **Step 2: Create the wrapper**

Create `src/lib/components/extractum-ui/Dialog.svelte`:

```svelte
<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from "$lib/components/ui/dialog/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps, Snippet } from "svelte";

  let {
    open = $bindable(false),
    title = "",
    description = "",
    class: className,
    contentClass = "",
    children,
    ...rest
  }: ComponentProps<typeof Dialog> & {
    title?: string;
    description?: string;
    class?: string;
    contentClass?: string;
    children?: Snippet;
  } = $props();
</script>

<Dialog bind:open {...rest}>
  <DialogContent
    class={cn(
      "extractum-dialog max-h-[min(760px,calc(100vh-48px))] w-[min(960px,calc(100vw-48px))] max-w-none overflow-hidden rounded-[var(--extractum-radius)] border border-[var(--extractum-border)] bg-[var(--extractum-surface)] p-0 text-[var(--extractum-text)] shadow-xl sm:max-w-none",
      className,
      contentClass,
    )}
  >
    {#if title || description}
      <DialogHeader class="border-b border-[var(--extractum-border)] px-4 py-3">
        {#if title}
          <DialogTitle>{title}</DialogTitle>
        {/if}
        {#if description}
          <DialogDescription>{description}</DialogDescription>
        {/if}
      </DialogHeader>
    {/if}

    <div class="extractum-dialog-body">
      {@render children?.()}
    </div>
  </DialogContent>
</Dialog>

<style>
  .extractum-dialog-body {
    min-height: 0;
    overflow: auto;
    padding: 16px;
  }
</style>
```

- [ ] **Step 3: Create the status message wrapper**

Create `src/lib/components/extractum-ui/StatusMessage.svelte`:

```svelte
<script lang="ts">
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let {
    className = "",
    ...rest
  }: ComponentProps<typeof StatusMessage> = $props();
</script>

<StatusMessage className={cn("extractum-status-message", className)} {...rest} />
```

- [ ] **Step 4: Export wrappers and dialog subparts**

Modify `src/lib/components/extractum-ui/index.ts`.

Add near the other wrapper exports:

```ts
export { default as ExtractumDialog } from "./Dialog.svelte";
export { default as ExtractumStatusMessage } from "./StatusMessage.svelte";
```

Add near the sheet re-exports:

```ts
export {
  DialogClose as ExtractumDialogClose,
  DialogDescription as ExtractumDialogDescription,
  DialogFooter as ExtractumDialogFooter,
  DialogHeader as ExtractumDialogHeader,
  DialogTitle as ExtractumDialogTitle,
} from "$lib/components/ui/dialog/index.js";
```

- [ ] **Step 5: Verify wrapper boundary**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
```

Expected: PASS.

- [ ] **Step 6: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 7: Commit wrappers**

Run:

```powershell
git add src/lib/components/extractum-ui/Dialog.svelte src/lib/components/extractum-ui/StatusMessage.svelte src/lib/components/extractum-ui/index.ts src/lib/research-projects-import-boundary.test.ts
git commit -m "feat: add extractum dialog wrappers"
```

---

## Task 2: Add Library Add Source Model Helpers

**Files:**
- Create: `src/lib/ui/library-add-source-model.ts`
- Create: `src/lib/ui/library-add-source-model.test.ts`

- [ ] **Step 1: Write model tests**

Create `src/lib/ui/library-add-source-model.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { LibraryCatalogSourceView } from "./library-catalog-model";
import type { TelegramDialogSource } from "$lib/types/sources";
import type { YoutubePlaylistDetail, YoutubePlaylistItemDetail } from "$lib/types/youtube";
import {
  YOUTUBE_PLAYLIST_IMPORT_LIMIT,
  buildPlaylistImportRows,
  classifyYoutubeImportInput,
  libraryYoutubePlaylistSources,
  playlistSelectionLimitMessage,
  selectedAddablePlaylistRows,
  telegramDialogAddInput,
} from "./library-add-source-model";

function source(overrides: Partial<LibraryCatalogSourceView> = {}): LibraryCatalogSourceView {
  return {
    id: "source:1",
    sourceId: 1,
    provider: "youtube",
    sourceSubtype: "playlist",
    title: "Playlist",
    subtitle: null,
    typeLabel: "YouTube / Playlist",
    status: "active",
    statusDetail: null,
    projectCount: 0,
    itemCount: 0,
    itemCountLabel: "0 items",
    addedAtLabel: "01/01/2026, 10:00 AM",
    lastSyncedLabel: "Never",
    canonicalUrl: "https://www.youtube.com/playlist?list=PL1",
    externalId: "PL1",
    youtube: {
      video_form: null,
      duration_seconds: null,
      playlist_video_count: 2,
      channel_title: "Channel",
      availability_status: "available",
    },
    telegram: null,
    ...overrides,
  };
}

function playlistItem(overrides: Partial<YoutubePlaylistItemDetail> = {}): YoutubePlaylistItemDetail {
  return {
    position: 1,
    videoId: "video-1",
    videoSourceId: null,
    title: "Video 1",
    canonicalUrl: "https://www.youtube.com/watch?v=video-1",
    thumbnailUrl: null,
    durationSeconds: 120,
    publishedAt: null,
    availabilityStatus: "available",
    isRemovedFromPlaylist: false,
    captions: {
      state: "not_synced",
      itemCount: 0,
      segmentCount: 0,
      lastSyncedAt: null,
      label: "Not synced",
    },
    comments: {
      state: "not_synced",
      itemCount: 0,
      segmentCount: 0,
      lastSyncedAt: null,
      label: "Not synced",
    },
    ...overrides,
  };
}

function playlistDetail(items: YoutubePlaylistItemDetail[]): YoutubePlaylistDetail {
  return {
    summary: {
      sourceId: 10,
      sourceSubtype: "playlist",
      title: "Playlist",
      channelTitle: "Channel",
      channelHandle: "@channel",
      canonicalUrl: "https://www.youtube.com/playlist?list=PL1",
      thumbnailUrl: null,
      durationSeconds: null,
      publishedAt: null,
      availabilityStatus: "available",
      videoCount: items.length,
      linkedVideoCount: items.filter((item) => item.videoSourceId !== null).length,
      unavailableCount: 0,
      captions: {
        state: "not_synced",
        itemCount: 0,
        segmentCount: 0,
        lastSyncedAt: null,
        label: "Not synced",
      },
      comments: {
        state: "not_synced",
        itemCount: 0,
        segmentCount: 0,
        lastSyncedAt: null,
        label: "Not synced",
      },
    },
    items,
  };
}

describe("library add source model", () => {
  it("classifies YouTube video playlist and channel inputs", () => {
    expect(classifyYoutubeImportInput("https://www.youtube.com/watch?v=abc123")).toMatchObject({
      kind: "video",
      supported: true,
    });
    expect(classifyYoutubeImportInput("https://youtu.be/abc123")).toMatchObject({
      kind: "video",
      supported: true,
    });
    expect(classifyYoutubeImportInput("https://www.youtube.com/playlist?list=PLabc")).toMatchObject({
      kind: "playlist",
      supported: true,
    });
    expect(classifyYoutubeImportInput("https://www.youtube.com/@tech_trends")).toEqual({
      provider: "youtube",
      kind: "channel",
      supported: false,
      reason: "YouTube channel import is not supported yet.",
    });
    expect(classifyYoutubeImportInput("https://www.youtube.com/channel/UCabc")).toMatchObject({
      provider: "youtube",
      kind: "channel",
      supported: false,
    });
  });

  it("classifies unsupported and Telegram input without switching provider", () => {
    expect(classifyYoutubeImportInput("https://t.me/ai_news")).toEqual({
      provider: "telegram",
      kind: "unsupported",
      supported: false,
      reason: "Telegram sources are added from the Telegram tab.",
    });
    expect(classifyYoutubeImportInput("https://example.com/post")).toEqual({
      provider: "unknown",
      kind: "unsupported",
      supported: false,
      reason: "Enter a YouTube video or playlist URL.",
    });
  });

  it("filters full Library catalog to YouTube playlists only", () => {
    expect(
      libraryYoutubePlaylistSources([
        source({ sourceId: 1, sourceSubtype: "playlist" }),
        source({ sourceId: 2, sourceSubtype: "video" }),
        source({ sourceId: 3, provider: "telegram", sourceSubtype: "channel" }),
      ]).map((row) => row.sourceId),
    ).toEqual([1]);
  });

  it("marks playlist rows that cannot be added", () => {
    const rows = buildPlaylistImportRows(
      playlistDetail([
        playlistItem({ videoId: "ready" }),
        playlistItem({ videoId: "linked", videoSourceId: 22 }),
        playlistItem({ videoId: "missing-url", canonicalUrl: null }),
      ]),
    );

    expect(rows).toMatchObject([
      { id: "ready", addable: true, disabledReason: null },
      { id: "linked", addable: false, disabledReason: "Already in Library" },
      { id: "missing-url", addable: false, disabledReason: "Missing video URL" },
    ]);
  });

  it("returns only selected addable playlist rows and enforces the MVP selection limit", () => {
    const rows = buildPlaylistImportRows(
      playlistDetail([
        playlistItem({ videoId: "a" }),
        playlistItem({ videoId: "b", videoSourceId: 10 }),
        playlistItem({ videoId: "c" }),
      ]),
    );

    expect(selectedAddablePlaylistRows(rows, new Set(["a", "b", "c"])).map((row) => row.id)).toEqual([
      "a",
      "c",
    ]);
    expect(playlistSelectionLimitMessage(YOUTUBE_PLAYLIST_IMPORT_LIMIT + 1)).toBe(
      `Select ${YOUTUBE_PLAYLIST_IMPORT_LIMIT} or fewer videos for one import run.`,
    );
    expect(playlistSelectionLimitMessage(YOUTUBE_PLAYLIST_IMPORT_LIMIT)).toBeNull();
  });

  it("builds Telegram dialog add input from the selected dialog", () => {
    const dialog: TelegramDialogSource = {
      id: 456,
      title: "Forum",
      username: "forum",
      sourceSubtype: "supergroup",
      isMember: true,
      photoDataUrl: null,
    };

    expect(telegramDialogAddInput(3, dialog)).toEqual({
      accountId: 3,
      sourceRef: "456",
      expectedSubtype: "supergroup",
    });
  });
});
```

- [ ] **Step 2: Run model tests to verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-add-source-model.test.ts
```

Expected: FAIL because `src/lib/ui/library-add-source-model.ts` does not exist.

- [ ] **Step 3: Implement model helpers**

Create `src/lib/ui/library-add-source-model.ts`:

```ts
import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
import type { AddTelegramSourceInput, TelegramDialogSource } from "$lib/types/sources";
import type { YoutubePlaylistDetail, YoutubePlaylistItemDetail } from "$lib/types/youtube";

export const YOUTUBE_PLAYLIST_IMPORT_LIMIT = 10;

export type YoutubeSmartImportProvider = "youtube" | "telegram" | "unknown";
export type YoutubeSmartImportKind = "video" | "playlist" | "channel" | "unsupported";

export interface YoutubeSmartImportClassification {
  provider: YoutubeSmartImportProvider;
  kind: YoutubeSmartImportKind;
  supported: boolean;
  reason: string | null;
}

export interface PlaylistImportRow {
  id: string;
  item: YoutubePlaylistItemDetail;
  addable: boolean;
  disabledReason: string | null;
}

export interface PlaylistImportItemResult {
  id: string;
  title: string;
  canonicalUrl: string | null;
  status: "added" | "skipped" | "failed";
  sourceId: number | null;
  message: string | null;
}

export interface PlaylistImportSummary {
  added: number;
  skipped: number;
  failed: number;
  results: PlaylistImportItemResult[];
}

function youtubeHost(host: string) {
  const normalized = host.toLocaleLowerCase();
  return normalized === "youtu.be" || normalized === "youtube.com" || normalized.endsWith(".youtube.com");
}

function telegramHost(host: string) {
  const normalized = host.toLocaleLowerCase();
  return normalized === "t.me" || normalized === "telegram.me" || normalized.endsWith(".telegram.org");
}

function firstNonEmptySegment(url: URL) {
  return url.pathname.split("/").find((segment) => segment.trim().length > 0) ?? "";
}

export function classifyYoutubeImportInput(input: string): YoutubeSmartImportClassification {
  const trimmed = input.trim();
  if (!trimmed) {
    return {
      provider: "unknown",
      kind: "unsupported",
      supported: false,
      reason: "Enter a YouTube video or playlist URL.",
    };
  }

  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    return {
      provider: "unknown",
      kind: "unsupported",
      supported: false,
      reason: "Enter a valid URL.",
    };
  }

  const host = parsed.host.toLocaleLowerCase();
  if (telegramHost(host)) {
    return {
      provider: "telegram",
      kind: "unsupported",
      supported: false,
      reason: "Telegram sources are added from the Telegram tab.",
    };
  }

  if (!youtubeHost(host)) {
    return {
      provider: "unknown",
      kind: "unsupported",
      supported: false,
      reason: "Enter a YouTube video or playlist URL.",
    };
  }

  const firstSegment = firstNonEmptySegment(parsed);
  if (firstSegment.startsWith("@") || firstSegment === "channel" || firstSegment === "c" || firstSegment === "user") {
    return {
      provider: "youtube",
      kind: "channel",
      supported: false,
      reason: "YouTube channel import is not supported yet.",
    };
  }

  if (parsed.searchParams.get("v") || host === "youtu.be" || firstSegment === "shorts" || firstSegment === "live") {
    return { provider: "youtube", kind: "video", supported: true, reason: null };
  }

  if (parsed.searchParams.get("list")) {
    return { provider: "youtube", kind: "playlist", supported: true, reason: null };
  }

  return {
    provider: "youtube",
    kind: "unsupported",
    supported: false,
    reason: "Enter a YouTube video or playlist URL.",
  };
}

export function libraryYoutubePlaylistSources(sources: LibraryCatalogSourceView[]) {
  return sources.filter((source) => source.provider === "youtube" && source.sourceSubtype === "playlist");
}

function playlistRowDisabledReason(item: YoutubePlaylistItemDetail) {
  if (item.videoSourceId !== null) return "Already in Library";
  if (!item.canonicalUrl) return "Missing video URL";
  return null;
}

export function buildPlaylistImportRows(detail: YoutubePlaylistDetail | null): PlaylistImportRow[] {
  return (detail?.items ?? []).map((item) => {
    const disabledReason = playlistRowDisabledReason(item);
    return {
      id: item.videoId,
      item,
      addable: disabledReason === null,
      disabledReason,
    };
  });
}

export function selectedAddablePlaylistRows(rows: PlaylistImportRow[], selectedIds: Set<string>) {
  return rows.filter((row) => selectedIds.has(row.id) && row.addable);
}

export function playlistSelectionLimitMessage(selectedAddableCount: number) {
  if (selectedAddableCount <= YOUTUBE_PLAYLIST_IMPORT_LIMIT) return null;
  return `Select ${YOUTUBE_PLAYLIST_IMPORT_LIMIT} or fewer videos for one import run.`;
}

export function emptyPlaylistImportSummary(): PlaylistImportSummary {
  return { added: 0, skipped: 0, failed: 0, results: [] };
}

export function summarizePlaylistImportResults(results: PlaylistImportItemResult[]): PlaylistImportSummary {
  return {
    added: results.filter((result) => result.status === "added").length,
    skipped: results.filter((result) => result.status === "skipped").length,
    failed: results.filter((result) => result.status === "failed").length,
    results,
  };
}

export function telegramDialogAddInput(
  accountId: number,
  dialog: TelegramDialogSource,
): AddTelegramSourceInput {
  return {
    accountId,
    sourceRef: String(dialog.id),
    expectedSubtype: dialog.sourceSubtype,
  };
}
```

- [ ] **Step 4: Verify model tests**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-add-source-model.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit model helpers**

Run:

```powershell
git add src/lib/ui/library-add-source-model.ts src/lib/ui/library-add-source-model.test.ts
git commit -m "feat: add library source import model"
```

---

## Task 3: Add Playlist Import Workflow

**Files:**
- Create: `src/lib/ui/library-add-source-workflow.ts`
- Create: `src/lib/ui/library-add-source-workflow.test.ts`

- [ ] **Step 1: Write workflow tests**

Create `src/lib/ui/library-add-source-workflow.test.ts`:

```ts
import { describe, expect, it, vi } from "vitest";
import type { Source } from "$lib/types/sources";
import type { PlaylistImportRow } from "./library-add-source-model";
import { addSelectedYoutubePlaylistVideos } from "./library-add-source-workflow";

function source(id: number): Source {
  return {
    id,
    sourceType: "youtube",
    sourceSubtype: "video",
    accountId: null,
    externalId: `video-${id}`,
    title: `Video ${id}`,
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: false,
    isActive: true,
    createdAt: 1_700_000_000,
    telegramUsername: null,
    avatarDataUrl: null,
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
  };
}

function row(overrides: Partial<PlaylistImportRow> = {}): PlaylistImportRow {
  const videoId = overrides.id ?? "video-1";
  return {
    id: videoId,
    addable: true,
    disabledReason: null,
    item: {
      position: 1,
      videoId,
      videoSourceId: null,
      title: `Video ${videoId}`,
      canonicalUrl: `https://www.youtube.com/watch?v=${videoId}`,
      thumbnailUrl: null,
      durationSeconds: null,
      publishedAt: null,
      availabilityStatus: "available",
      isRemovedFromPlaylist: false,
      captions: {
        state: "not_synced",
        itemCount: 0,
        segmentCount: 0,
        lastSyncedAt: null,
        label: "Not synced",
      },
      comments: {
        state: "not_synced",
        itemCount: 0,
        segmentCount: 0,
        lastSyncedAt: null,
        label: "Not synced",
      },
    },
    ...overrides,
  };
}

describe("library add source workflow", () => {
  it("adds selected playlist videos sequentially", async () => {
    const addYoutubeSource = vi.fn()
      .mockResolvedValueOnce(source(101))
      .mockResolvedValueOnce(source(102));

    const summary = await addSelectedYoutubePlaylistVideos({
      rows: [row({ id: "a" }), row({ id: "b" })],
      addYoutubeSource,
      formatError: (_action, error) => String(error),
    });

    expect(addYoutubeSource).toHaveBeenNthCalledWith(1, "https://www.youtube.com/watch?v=a");
    expect(addYoutubeSource).toHaveBeenNthCalledWith(2, "https://www.youtube.com/watch?v=b");
    expect(summary.added).toBe(2);
    expect(summary.failed).toBe(0);
    expect(summary.results.map((result) => result.sourceId)).toEqual([101, 102]);
  });

  it("skips rows that become non-addable before execution", async () => {
    const addYoutubeSource = vi.fn();

    const summary = await addSelectedYoutubePlaylistVideos({
      rows: [
        row({
          id: "linked",
          addable: false,
          disabledReason: "Already in Library",
        }),
      ],
      addYoutubeSource,
      formatError: (_action, error) => String(error),
    });

    expect(addYoutubeSource).not.toHaveBeenCalled();
    expect(summary.skipped).toBe(1);
    expect(summary.results[0]).toMatchObject({
      id: "linked",
      status: "skipped",
      message: "Already in Library",
    });
  });

  it("reports partial failure without stopping later rows", async () => {
    const addYoutubeSource = vi.fn()
      .mockResolvedValueOnce(source(101))
      .mockRejectedValueOnce(new Error("network down"))
      .mockResolvedValueOnce(source(103));

    const summary = await addSelectedYoutubePlaylistVideos({
      rows: [row({ id: "a" }), row({ id: "b" }), row({ id: "c" })],
      addYoutubeSource,
      formatError: (_action, error) => error instanceof Error ? error.message : String(error),
    });

    expect(addYoutubeSource).toHaveBeenCalledTimes(3);
    expect(summary.added).toBe(2);
    expect(summary.failed).toBe(1);
    expect(summary.results[1]).toMatchObject({
      id: "b",
      status: "failed",
      sourceId: null,
      message: "network down",
    });
  });
});
```

- [ ] **Step 2: Run workflow tests to verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-add-source-workflow.test.ts
```

Expected: FAIL because `src/lib/ui/library-add-source-workflow.ts` does not exist.

- [ ] **Step 3: Implement workflow**

Create `src/lib/ui/library-add-source-workflow.ts`:

```ts
import type { Source } from "$lib/types/sources";
import {
  summarizePlaylistImportResults,
  type PlaylistImportItemResult,
  type PlaylistImportRow,
} from "./library-add-source-model";

export interface AddSelectedYoutubePlaylistVideosInput {
  rows: PlaylistImportRow[];
  addYoutubeSource(url: string): Promise<Source>;
  formatError(action: string, error: unknown): string;
}

function resultTitle(row: PlaylistImportRow) {
  return row.item.title ?? row.item.videoId;
}

export async function addSelectedYoutubePlaylistVideos({
  rows,
  addYoutubeSource,
  formatError,
}: AddSelectedYoutubePlaylistVideosInput) {
  const results: PlaylistImportItemResult[] = [];

  for (const row of rows) {
    if (!row.addable || !row.item.canonicalUrl) {
      results.push({
        id: row.id,
        title: resultTitle(row),
        canonicalUrl: row.item.canonicalUrl,
        status: "skipped",
        sourceId: null,
        message: row.disabledReason ?? "Video cannot be added.",
      });
      continue;
    }

    try {
      const source = await addYoutubeSource(row.item.canonicalUrl);
      results.push({
        id: row.id,
        title: resultTitle(row),
        canonicalUrl: row.item.canonicalUrl,
        status: "added",
        sourceId: source.id,
        message: source.title ?? source.externalId,
      });
    } catch (error) {
      results.push({
        id: row.id,
        title: resultTitle(row),
        canonicalUrl: row.item.canonicalUrl,
        status: "failed",
        sourceId: null,
        message: formatError(`adding ${resultTitle(row)}`, error),
      });
    }
  }

  return summarizePlaylistImportResults(results);
}
```

- [ ] **Step 4: Verify workflow tests**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-add-source-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run combined model/workflow tests**

Run:

```powershell
npm.cmd run test -- src/lib/ui/library-add-source-model.test.ts src/lib/ui/library-add-source-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit workflow**

Run:

```powershell
git add src/lib/ui/library-add-source-workflow.ts src/lib/ui/library-add-source-workflow.test.ts
git commit -m "feat: add library playlist import workflow"
```

---

## Task 4: Add Component Contract Tests

**Files:**
- Create: `src/lib/library-add-source-contract.test.ts`
- Modify: `src/lib/library-prototype-contract.test.ts`

- [ ] **Step 1: Add raw-source contract tests for new components**

Create `src/lib/library-add-source-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import dialogSource from "./components/research-projects/LibraryAddSourceDialog.svelte?raw";
import youtubePanelSource from "./components/research-projects/LibraryYoutubeAddPanel.svelte?raw";
import smartImportSource from "./components/research-projects/LibraryYoutubeSmartImport.svelte?raw";
import playlistImportSource from "./components/research-projects/LibraryYoutubePlaylistImport.svelte?raw";
import telegramImportSource from "./components/research-projects/LibraryTelegramDialogImport.svelte?raw";

describe("Library Add Source contract", () => {
  const addSourceComponentSources = [
    dialogSource,
    youtubePanelSource,
    smartImportSource,
    playlistImportSource,
    telegramImportSource,
  ];

  it("uses extractum wrappers for dialog and tabs", () => {
    expect(dialogSource).toContain("ExtractumDialog");
    expect(dialogSource).toContain("ExtractumTabs");
    expect(dialogSource).toContain("ExtractumTabsList");
    expect(dialogSource).toContain("ExtractumTabsTrigger");
    expect(dialogSource).toContain("ExtractumTabsContent");
    expect(dialogSource).not.toContain("$lib/components/ui/");
    expect(dialogSource).not.toContain("bits-ui");
  });

  it("keeps all Add Source components behind extractum-ui wrappers", () => {
    for (const source of addSourceComponentSources) {
      expect(source).not.toContain("$lib/components/ui/");
      expect(source).not.toContain("bits-ui");
      expect(source).not.toContain("@svar-ui/");
    }
  });

  it("keeps YouTube mode tabs inside the YouTube panel", () => {
    expect(youtubePanelSource).toContain("Smart import");
    expect(youtubePanelSource).toContain("From existing data");
    expect(youtubePanelSource).toContain("LibraryYoutubeSmartImport");
    expect(youtubePanelSource).toContain("LibraryYoutubePlaylistImport");
  });

  it("classifies YouTube smart import before calling backend preview", () => {
    expect(smartImportSource).toContain("classifyYoutubeImportInput");
    expect(smartImportSource).toContain("previewYoutubeSource");
    expect(smartImportSource).toContain("addYoutubeSource");
    expect(smartImportSource).toContain("Not supported yet");
  });

  it("adds selected videos from existing playlist details", () => {
    expect(playlistImportSource).toContain("getYoutubePlaylistDetail");
    expect(playlistImportSource).toContain("addSelectedYoutubePlaylistVideos");
    expect(playlistImportSource).toContain("YOUTUBE_PLAYLIST_IMPORT_LIMIT");
    expect(playlistImportSource).toContain("Already in Library");
  });

  it("adds Telegram sources only from selected account dialogs", () => {
    expect(telegramImportSource).toContain("listAccounts");
    expect(telegramImportSource).toContain("getAccountRuntimeStatuses");
    expect(telegramImportSource).toContain("listTelegramSources");
    expect(telegramImportSource).toContain("telegramDialogAddInput");
    expect(telegramImportSource).toContain("addTelegramSource");
  });
});
```

- [ ] **Step 2: Add LibraryScreen contract for replacing prototype Add**

Modify `src/lib/library-prototype-contract.test.ts`.

In the final test `"coordinates filter selection, row selection, and Inspector resizing in the screen component"`, add:

```ts
    expect(screenSource).toContain("LibraryAddSourceDialog");
    expect(screenSource).toContain("addSourceDialogOpen");
    expect(screenSource).not.toContain('prototypeFeedback("Add source")');
```

- [ ] **Step 3: Run contract tests to verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/library-add-source-contract.test.ts src/lib/library-prototype-contract.test.ts
```

Expected: FAIL because Add Source components do not exist and `LibraryScreen.svelte` still uses prototype Add feedback.

Do not commit yet. The tests should pass in later tasks.

---

## Task 5: Add YouTube Smart Import Component

**Files:**
- Create: `src/lib/components/research-projects/LibraryYoutubeSmartImport.svelte`

- [ ] **Step 1: Create the component**

Create `src/lib/components/research-projects/LibraryYoutubeSmartImport.svelte`:

```svelte
<script lang="ts">
  import { Eye, Plus } from "@lucide/svelte";
  import {
    ExtractumBadge,
    ExtractumButton,
    ExtractumStatusMessage,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import { addYoutubeSource, previewYoutubeSource } from "$lib/api/sources";
  import { formatAppError } from "$lib/app-error";
  import { classifyYoutubeImportInput } from "$lib/ui/library-add-source-model";
  import type { YoutubePreview } from "$lib/types/sources";

  let {
    onSourcesChanged,
    onStatus,
  }: {
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let youtubeUrl = $state("");
  let preview = $state<YoutubePreview | null>(null);
  let previewedUrl = $state("");
  let previewing = $state(false);
  let adding = $state(false);
  let status = $state("");

  const trimmedUrl = $derived(youtubeUrl.trim());
  const classification = $derived(classifyYoutubeImportInput(trimmedUrl));
  const canPreview = $derived(Boolean(trimmedUrl) && classification.supported && !previewing && !adding);
  const canAdd = $derived(Boolean(preview) && !previewing && !adding);

  function updateUrl(value: string) {
    youtubeUrl = value;
    status = "";
    if (value.trim() !== previewedUrl) preview = null;
  }

  async function previewSource() {
    if (!canPreview) return;
    previewing = true;
    status = "";
    try {
      preview = await previewYoutubeSource(trimmedUrl);
      previewedUrl = trimmedUrl;
    } catch (error) {
      preview = null;
      status = formatAppError("previewing the YouTube source", error);
    } finally {
      previewing = false;
    }
  }

  async function addSource() {
    if (!preview || adding) return;
    adding = true;
    status = "";
    try {
      const source = await addYoutubeSource(previewedUrl || trimmedUrl);
      onStatus(`Source "${source.title ?? source.externalId}" added.`);
      await onSourcesChanged(source.id);
      youtubeUrl = "";
      preview = null;
      previewedUrl = "";
    } catch (error) {
      status = formatAppError("adding the YouTube source", error);
    } finally {
      adding = false;
    }
  }

  function formatDuration(seconds: number | null) {
    if (seconds === null) return null;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${String(remainingSeconds).padStart(2, "0")}`;
  }
</script>

<section class="library-youtube-smart-import" aria-label="YouTube smart import">
  <div class="entry-row">
    <label>
      <span>YouTube URL</span>
      <ExtractumTextInput
        value={youtubeUrl}
        placeholder="https://www.youtube.com/watch?v=..."
        disabled={previewing || adding}
        oninput={(event) => updateUrl((event.currentTarget as HTMLInputElement).value)}
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            void previewSource();
          }
        }}
      />
    </label>

    <ExtractumButton onclick={previewSource} disabled={!canPreview}>
      <Eye size={14} aria-hidden="true" />
      {previewing ? "Previewing..." : "Preview"}
    </ExtractumButton>
  </div>

  {#if classification.reason && trimmedUrl}
    <ExtractumStatusMessage tone={classification.kind === "channel" ? "info" : "muted"}>
      {classification.kind === "channel" ? "Not supported yet. " : ""}{classification.reason}
    </ExtractumStatusMessage>
  {/if}

  {#if status}
    <ExtractumStatusMessage tone={status.startsWith("Error") ? "error" : "default"}>
      {status}
    </ExtractumStatusMessage>
  {/if}

  {#if preview}
    <article class="preview-card">
      <div class="preview-media" aria-hidden="true">
        {#if preview.thumbnailUrl}
          <img src={preview.thumbnailUrl} alt="" loading="lazy" />
        {:else}
          <span>{preview.kind === "playlist" ? "PL" : "YT"}</span>
        {/if}
      </div>
      <div class="preview-copy">
        <div class="badges">
          <ExtractumBadge>{preview.kind}</ExtractumBadge>
          <ExtractumBadge>{preview.availabilityStatus.replaceAll("_", " ")}</ExtractumBadge>
          {#if preview.playlistVideoCount !== null}
            <ExtractumBadge>{preview.playlistVideoCount} videos</ExtractumBadge>
          {/if}
          {#if formatDuration(preview.durationSeconds)}
            <ExtractumBadge>{formatDuration(preview.durationSeconds)}</ExtractumBadge>
          {/if}
        </div>
        <strong>{preview.title ?? preview.externalId}</strong>
        <p>{preview.channelTitle ?? preview.channelHandle ?? preview.canonicalUrl}</p>
        <div class="actions">
          <span>{preview.canonicalUrl}</span>
          <ExtractumButton onclick={addSource} disabled={!canAdd}>
            <Plus size={14} aria-hidden="true" />
            {adding ? "Adding..." : "Add source"}
          </ExtractumButton>
        </div>
      </div>
    </article>
  {/if}
</section>

<style>
  .library-youtube-smart-import {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .entry-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 8px;
    align-items: end;
  }

  label {
    display: grid;
    gap: 4px;
    color: var(--extractum-muted);
    font-size: 13px;
  }

  .preview-card {
    display: grid;
    grid-template-columns: minmax(150px, 220px) minmax(0, 1fr);
    gap: 12px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 10px;
  }

  .preview-media {
    display: grid;
    place-items: center;
    aspect-ratio: 16 / 9;
    overflow: hidden;
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-subtle);
    color: var(--extractum-muted);
    font-weight: 700;
  }

  .preview-media img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .preview-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .badges,
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }

  .actions {
    justify-content: space-between;
  }

  .actions span,
  p {
    margin: 0;
    min-width: 0;
    overflow-wrap: anywhere;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  @media (max-width: 760px) {
    .entry-row,
    .preview-card {
      grid-template-columns: 1fr;
    }
  }
</style>
```

- [ ] **Step 2: Run contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/library-add-source-contract.test.ts
```

Expected: still FAIL because other Add Source components do not exist.

- [ ] **Step 3: Run Svelte check for this component**

Run:

```powershell
npm.cmd run check
```

Expected: PASS or fail only on later missing imports if another task has already added references early. If this component fails, fix before moving on.

Do not commit yet; commit after the full dialog/component slice is wired.

---

## Task 6: Add YouTube Playlist Import Component

**Files:**
- Create: `src/lib/components/research-projects/LibraryYoutubePlaylistImport.svelte`

- [ ] **Step 1: Create the component**

Create `src/lib/components/research-projects/LibraryYoutubePlaylistImport.svelte`:

```svelte
<script lang="ts">
  import { Plus, RefreshCw } from "@lucide/svelte";
  import {
    ExtractumBadge,
    ExtractumButton,
    ExtractumStatusMessage,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import { addYoutubeSource } from "$lib/api/sources";
  import { getYoutubePlaylistDetail } from "$lib/api/youtube-detail";
  import { formatAppError } from "$lib/app-error";
  import {
    YOUTUBE_PLAYLIST_IMPORT_LIMIT,
    buildPlaylistImportRows,
    libraryYoutubePlaylistSources,
    playlistSelectionLimitMessage,
    selectedAddablePlaylistRows,
    type PlaylistImportSummary,
  } from "$lib/ui/library-add-source-model";
  import { addSelectedYoutubePlaylistVideos } from "$lib/ui/library-add-source-workflow";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import type { YoutubePlaylistDetail } from "$lib/types/youtube";

  let {
    sources,
    onSourcesChanged,
    onStatus,
  }: {
    sources: LibraryCatalogSourceView[];
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let playlistQuery = $state("");
  let selectedPlaylistId = $state<number | null>(null);
  let detail = $state<YoutubePlaylistDetail | null>(null);
  let loadingDetail = $state(false);
  let adding = $state(false);
  let selectedVideoIds = $state<Set<string>>(new Set());
  let status = $state("");
  let summary = $state<PlaylistImportSummary | null>(null);

  const playlists = $derived(libraryYoutubePlaylistSources(sources));
  const filteredPlaylists = $derived.by(() => {
    const query = playlistQuery.trim().toLocaleLowerCase();
    if (!query) return playlists;
    return playlists.filter((source) =>
      `${source.title} ${source.subtitle ?? ""} ${source.externalId ?? ""}`.toLocaleLowerCase().includes(query),
    );
  });
  const rows = $derived(buildPlaylistImportRows(detail));
  const selectedRows = $derived(selectedAddablePlaylistRows(rows, selectedVideoIds));
  const limitMessage = $derived(playlistSelectionLimitMessage(selectedRows.length));
  const canAddSelected = $derived(selectedRows.length > 0 && !limitMessage && !adding);

  async function loadPlaylist(sourceId: number) {
    selectedPlaylistId = sourceId;
    detail = null;
    summary = null;
    status = "";
    selectedVideoIds = new Set();
    loadingDetail = true;
    try {
      detail = await getYoutubePlaylistDetail(sourceId);
    } catch (error) {
      status = formatAppError("loading YouTube playlist", error);
    } finally {
      loadingDetail = false;
    }
  }

  function toggleVideo(id: string) {
    const next = new Set(selectedVideoIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedVideoIds = next;
  }

  async function addSelected() {
    if (!canAddSelected) return;
    adding = true;
    status = "";
    try {
      summary = await addSelectedYoutubePlaylistVideos({
        rows: selectedRows,
        addYoutubeSource,
        formatError: formatAppError,
      });
      if (summary.added > 0) {
        onStatus(`Added ${summary.added} YouTube video source${summary.added === 1 ? "" : "s"}.`);
        await onSourcesChanged(summary.results.find((result) => result.sourceId !== null)?.sourceId ?? undefined);
      }
    } finally {
      adding = false;
    }
  }
</script>

<section class="library-youtube-playlist-import" aria-label="YouTube playlist import">
  <div class="playlist-picker">
    <label>
      <span>Playlist search</span>
      <ExtractumTextInput
        value={playlistQuery}
        placeholder="Search playlists"
        oninput={(event) => (playlistQuery = (event.currentTarget as HTMLInputElement).value)}
      />
    </label>
    <ExtractumBadge>{filteredPlaylists.length} playlists</ExtractumBadge>
  </div>

  {#if playlists.length === 0}
    <ExtractumStatusMessage tone="muted">No YouTube playlists are in Library yet.</ExtractumStatusMessage>
  {:else}
    <div class="playlist-list" aria-label="Existing YouTube playlists">
      {#each filteredPlaylists as playlist (playlist.id)}
        <button
          type="button"
          class:selected={playlist.sourceId === selectedPlaylistId}
          onclick={() => void loadPlaylist(playlist.sourceId)}
        >
          <strong>{playlist.title}</strong>
          <span>{playlist.subtitle ?? playlist.externalId ?? "YouTube playlist"}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#if status}
    <ExtractumStatusMessage tone={status.startsWith("Error") ? "error" : "default"}>{status}</ExtractumStatusMessage>
  {/if}

  {#if loadingDetail}
    <ExtractumStatusMessage tone="muted">Loading playlist videos...</ExtractumStatusMessage>
  {:else if detail}
    <div class="video-toolbar">
      <div>
        <strong>{detail.summary.title ?? "Playlist videos"}</strong>
        <span>{selectedRows.length} selected, limit {YOUTUBE_PLAYLIST_IMPORT_LIMIT}</span>
      </div>
      <ExtractumButton onclick={addSelected} disabled={!canAddSelected}>
        {#if adding}
          <RefreshCw size={14} aria-hidden="true" />
          Adding...
        {:else}
          <Plus size={14} aria-hidden="true" />
          Add selected
        {/if}
      </ExtractumButton>
    </div>

    {#if limitMessage}
      <ExtractumStatusMessage tone="error">{limitMessage}</ExtractumStatusMessage>
    {/if}

    <div class="video-list" aria-label="Playlist videos">
      {#each rows as row (row.id)}
        <label class:disabled={!row.addable}>
          <input
            type="checkbox"
            checked={selectedVideoIds.has(row.id)}
            disabled={!row.addable || adding}
            onchange={() => toggleVideo(row.id)}
          />
          <span>
            <strong>{row.item.title ?? row.item.videoId}</strong>
            <small>{row.disabledReason ?? row.item.canonicalUrl}</small>
          </span>
          {#if row.disabledReason}
            <ExtractumBadge>{row.disabledReason}</ExtractumBadge>
          {/if}
        </label>
      {/each}
    </div>
  {/if}

  {#if summary}
    <ExtractumStatusMessage tone={summary.failed > 0 ? "error" : "default"}>
      Added {summary.added}, skipped {summary.skipped}, failed {summary.failed}.
    </ExtractumStatusMessage>
  {/if}
</section>

<style>
  .library-youtube-playlist-import {
    display: grid;
    gap: 12px;
  }

  .playlist-picker,
  .video-toolbar {
    display: flex;
    gap: 8px;
    align-items: end;
    justify-content: space-between;
  }

  label {
    display: grid;
    gap: 4px;
    color: var(--extractum-muted);
    font-size: 13px;
  }

  .playlist-list,
  .video-list {
    display: grid;
    gap: 6px;
    max-height: 280px;
    overflow: auto;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 6px;
  }

  .playlist-list button,
  .video-list label {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 6px;
    align-items: center;
    border: 1px solid transparent;
    border-radius: var(--extractum-radius);
    padding: 8px;
    background: transparent;
    color: var(--extractum-text);
    text-align: left;
  }

  .playlist-list button.selected,
  .playlist-list button:hover,
  .video-list label:hover {
    border-color: var(--extractum-border);
    background: var(--extractum-surface-subtle);
  }

  .video-list label {
    grid-template-columns: auto minmax(0, 1fr) auto;
  }

  .video-list label.disabled {
    opacity: 0.66;
  }

  strong,
  small,
  .video-toolbar span {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  small,
  .playlist-list span,
  .video-toolbar span {
    color: var(--extractum-muted);
    font-size: 12px;
  }
</style>
```

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS or fail only because later Add Source components are still missing if references were added early. Fix syntax/type errors in this component before continuing.

Do not commit yet.

---

## Task 7: Add YouTube Provider Panel

**Files:**
- Create: `src/lib/components/research-projects/LibraryYoutubeAddPanel.svelte`

- [ ] **Step 1: Create the provider panel**

Create `src/lib/components/research-projects/LibraryYoutubeAddPanel.svelte`:

```svelte
<script lang="ts">
  import {
    ExtractumTabs,
    ExtractumTabsContent,
    ExtractumTabsList,
    ExtractumTabsTrigger,
  } from "$lib/components/extractum-ui";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import LibraryYoutubePlaylistImport from "./LibraryYoutubePlaylistImport.svelte";
  import LibraryYoutubeSmartImport from "./LibraryYoutubeSmartImport.svelte";

  let {
    sources,
    onSourcesChanged,
    onStatus,
  }: {
    sources: LibraryCatalogSourceView[];
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let mode = $state<"smart" | "existing">("smart");
</script>

<section class="library-youtube-add-panel" aria-label="YouTube Add Source">
  <ExtractumTabs bind:value={mode}>
    <ExtractumTabsList aria-label="YouTube import modes">
      <ExtractumTabsTrigger value="smart">Smart import</ExtractumTabsTrigger>
      <ExtractumTabsTrigger value="existing">From existing data</ExtractumTabsTrigger>
    </ExtractumTabsList>

    <ExtractumTabsContent value="smart">
      <LibraryYoutubeSmartImport {onSourcesChanged} {onStatus} />
    </ExtractumTabsContent>

    <ExtractumTabsContent value="existing">
      <LibraryYoutubePlaylistImport {sources} {onSourcesChanged} {onStatus} />
    </ExtractumTabsContent>
  </ExtractumTabs>
</section>

<style>
  .library-youtube-add-panel {
    min-height: 0;
  }

  .library-youtube-add-panel :global([data-slot="tabs"]) {
    min-height: 0;
  }
</style>
```

- [ ] **Step 2: Run contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/library-add-source-contract.test.ts
```

Expected: still FAIL because dialog and Telegram components are not present.

Do not commit yet.

---

## Task 8: Add Telegram Dialog Import Component

**Files:**
- Create: `src/lib/components/research-projects/LibraryTelegramDialogImport.svelte`

- [ ] **Step 1: Create the component**

Create `src/lib/components/research-projects/LibraryTelegramDialogImport.svelte`:

```svelte
<script lang="ts">
  import { Plus, RefreshCw } from "@lucide/svelte";
  import {
    ExtractumBadge,
    ExtractumButton,
    ExtractumStatusMessage,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import { getAccountRuntimeStatuses, listAccounts } from "$lib/api/accounts";
  import { addTelegramSource, listTelegramSources } from "$lib/api/sources";
  import { formatAppError } from "$lib/app-error";
  import { telegramDialogAddInput } from "$lib/ui/library-add-source-model";
  import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
  import type { DialogKindFilter, TelegramDialogSource } from "$lib/types/sources";

  let {
    onSourcesChanged,
    onStatus,
  }: {
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let accounts = $state<AccountRecord[]>([]);
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let selectedAccountId = $state("");
  let kindFilter = $state<DialogKindFilter>("all");
  let query = $state("");
  let dialogs = $state<TelegramDialogSource[]>([]);
  let selectedDialogId = $state<number | null>(null);
  let loadingAccounts = $state(false);
  let loadingDialogs = $state(false);
  let adding = $state(false);
  let status = $state("");

  const activeAccountId = $derived.by(() => {
    const selected = Number(selectedAccountId);
    if (accounts.some((account) => account.id === selected)) return selected;
    return accounts[0]?.id ?? null;
  });
  const selectedAccount = $derived(accounts.find((account) => account.id === activeAccountId) ?? null);
  const selectedRuntime = $derived(selectedAccount ? accountStatuses[selectedAccount.id] ?? null : null);
  const selectedAccountReady = $derived(selectedRuntime?.status === "ready");
  const selectedDialog = $derived(dialogs.find((dialog) => dialog.id === selectedDialogId) ?? null);
  const filteredDialogs = $derived.by(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    return dialogs.filter((dialog) => {
      if (kindFilter !== "all" && dialog.sourceSubtype !== kindFilter) return false;
      if (!normalizedQuery) return true;
      return `${dialog.title} ${dialog.username ?? ""} ${dialog.id}`.toLocaleLowerCase().includes(normalizedQuery);
    });
  });
  const canLoadDialogs = $derived(Boolean(activeAccountId) && selectedAccountReady && !loadingDialogs);
  const canAdd = $derived(Boolean(activeAccountId && selectedDialog) && selectedAccountReady && !adding);

  async function loadAccountsAndStatuses() {
    loadingAccounts = true;
    status = "";
    try {
      accounts = await listAccounts();
      selectedAccountId = String(accounts[0]?.id ?? "");
      if (accounts.length > 0) {
        const statuses = await getAccountRuntimeStatuses(accounts.map((account) => account.id));
        accountStatuses = Object.fromEntries(statuses.map((runtime) => [runtime.account_id, runtime]));
      } else {
        accountStatuses = {};
      }
    } catch (error) {
      status = formatAppError("loading Telegram accounts", error);
    } finally {
      loadingAccounts = false;
    }
  }

  async function loadDialogs() {
    if (!activeAccountId || !selectedAccountReady) return;
    loadingDialogs = true;
    status = "";
    selectedDialogId = null;
    try {
      dialogs = await listTelegramSources(activeAccountId);
    } catch (error) {
      dialogs = [];
      status = formatAppError("loading Telegram dialogs", error);
    } finally {
      loadingDialogs = false;
    }
  }

  async function addSelectedDialog() {
    if (!activeAccountId || !selectedDialog || adding) return;
    adding = true;
    status = "";
    try {
      const source = await addTelegramSource(telegramDialogAddInput(activeAccountId, selectedDialog));
      onStatus(`Source "${source.title ?? source.externalId}" added.`);
      await onSourcesChanged(source.id);
    } catch (error) {
      status = formatAppError("adding Telegram source", error);
    } finally {
      adding = false;
    }
  }

  $effect(() => {
    if (accounts.length === 0 && !loadingAccounts) {
      void loadAccountsAndStatuses();
    }
  });
</script>

<section class="library-telegram-dialog-import" aria-label="Telegram dialog import">
  <div class="toolbar">
    <label>
      <span>Account</span>
      <select
        bind:value={selectedAccountId}
        disabled={loadingAccounts || adding}
        onchange={() => {
          dialogs = [];
          selectedDialogId = null;
        }}
      >
        {#if accounts.length === 0}
          <option value="">No accounts configured</option>
        {/if}
        {#each accounts as account (account.id)}
          <option value={String(account.id)}>{account.label}</option>
        {/each}
      </select>
    </label>

    <label>
      <span>Kind</span>
      <select bind:value={kindFilter} disabled={loadingDialogs || adding}>
        <option value="all">All</option>
        <option value="channel">Channels</option>
        <option value="supergroup">Supergroups</option>
        <option value="group">Groups</option>
      </select>
    </label>

    <ExtractumButton onclick={loadDialogs} disabled={!canLoadDialogs}>
      <RefreshCw size={14} aria-hidden="true" />
      {loadingDialogs ? "Loading..." : "Load dialogs"}
    </ExtractumButton>
  </div>

  {#if !selectedAccount}
    <ExtractumStatusMessage tone="muted">Add and sign in to a Telegram account before adding Telegram sources.</ExtractumStatusMessage>
  {:else if !selectedAccountReady}
    <ExtractumStatusMessage tone="muted">
      Sign in to "{selectedAccount.label}" before loading Telegram dialogs.
    </ExtractumStatusMessage>
  {/if}

  {#if status}
    <ExtractumStatusMessage tone={status.startsWith("Error") ? "error" : "default"}>{status}</ExtractumStatusMessage>
  {/if}

  <ExtractumTextInput
    value={query}
    placeholder="Search dialogs"
    aria-label="Search Telegram dialogs"
    disabled={loadingDialogs || dialogs.length === 0}
    oninput={(event) => (query = (event.currentTarget as HTMLInputElement).value)}
  />

  <div class="dialog-list" aria-label="Telegram dialogs">
    {#if loadingDialogs}
      <ExtractumStatusMessage tone="muted">Loading Telegram dialogs...</ExtractumStatusMessage>
    {:else if dialogs.length === 0}
      <ExtractumStatusMessage tone="muted">Load dialogs from a ready account.</ExtractumStatusMessage>
    {:else}
      {#each filteredDialogs as dialog (dialog.id)}
        <button
          type="button"
          class:selected={dialog.id === selectedDialogId}
          onclick={() => (selectedDialogId = dialog.id)}
        >
          <span>
            <strong>{dialog.title}</strong>
            <small>{dialog.username ? `@${dialog.username}` : dialog.id}</small>
          </span>
          <ExtractumBadge>{dialog.sourceSubtype}</ExtractumBadge>
        </button>
      {/each}
    {/if}
  </div>

  <div class="footer">
    <ExtractumBadge>{filteredDialogs.length} visible</ExtractumBadge>
    <ExtractumButton onclick={addSelectedDialog} disabled={!canAdd}>
      <Plus size={14} aria-hidden="true" />
      {adding ? "Adding..." : "Add selected"}
    </ExtractumButton>
  </div>
</section>

<style>
  .library-telegram-dialog-import {
    display: grid;
    gap: 12px;
  }

  .toolbar,
  .footer {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: end;
    justify-content: space-between;
  }

  label {
    display: grid;
    gap: 4px;
    color: var(--extractum-muted);
    font-size: 13px;
  }

  select {
    height: 32px;
    min-width: 150px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    color: var(--extractum-text);
    font-size: 13px;
  }

  .dialog-list {
    display: grid;
    gap: 6px;
    max-height: 340px;
    overflow: auto;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 6px;
  }

  .dialog-list button {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    align-items: center;
    border: 1px solid transparent;
    border-radius: var(--extractum-radius);
    padding: 8px;
    background: transparent;
    color: var(--extractum-text);
    text-align: left;
  }

  .dialog-list button.selected,
  .dialog-list button:hover {
    border-color: var(--extractum-border);
    background: var(--extractum-surface-subtle);
  }

  .dialog-list span {
    min-width: 0;
    display: grid;
    gap: 2px;
  }

  small {
    color: var(--extractum-muted);
    font-size: 12px;
  }
</style>
```

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS or fail only because later dialog shell is still missing if references were added early. Fix syntax/type errors before continuing.

Do not commit yet.

---

## Task 9: Add Add Source Dialog Shell And Wire LibraryScreen

**Files:**
- Create: `src/lib/components/research-projects/LibraryAddSourceDialog.svelte`
- Modify: `src/lib/components/research-projects/LibraryScreen.svelte`

- [ ] **Step 1: Create dialog shell**

Create `src/lib/components/research-projects/LibraryAddSourceDialog.svelte`:

```svelte
<script lang="ts">
  import {
    ExtractumDialog,
    ExtractumTabs,
    ExtractumTabsContent,
    ExtractumTabsList,
    ExtractumTabsTrigger,
  } from "$lib/components/extractum-ui";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import LibraryTelegramDialogImport from "./LibraryTelegramDialogImport.svelte";
  import LibraryYoutubeAddPanel from "./LibraryYoutubeAddPanel.svelte";

  let {
    open = $bindable(false),
    sources,
    onSourcesChanged,
    onStatus,
  }: {
    open: boolean;
    sources: LibraryCatalogSourceView[];
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let provider = $state<"youtube" | "telegram">("youtube");
</script>

<ExtractumDialog
  bind:open
  title="Add source"
  description="Add YouTube sources or import Telegram sources from an authorized account."
>
  <section class="library-add-source-dialog" data-ui-region="library-add-source-dialog">
    <ExtractumTabs bind:value={provider}>
      <ExtractumTabsList aria-label="Source providers">
        <ExtractumTabsTrigger value="youtube">YouTube</ExtractumTabsTrigger>
        <ExtractumTabsTrigger value="telegram">Telegram</ExtractumTabsTrigger>
      </ExtractumTabsList>

      <ExtractumTabsContent value="youtube">
        <LibraryYoutubeAddPanel {sources} {onSourcesChanged} {onStatus} />
      </ExtractumTabsContent>

      <ExtractumTabsContent value="telegram">
        <LibraryTelegramDialogImport {onSourcesChanged} {onStatus} />
      </ExtractumTabsContent>
    </ExtractumTabs>
  </section>
</ExtractumDialog>

<style>
  .library-add-source-dialog {
    min-height: 520px;
    display: grid;
  }

  .library-add-source-dialog :global([data-slot="tabs"]) {
    min-height: 0;
  }
</style>
```

- [ ] **Step 2: Wire the dialog in LibraryScreen**

Modify `src/lib/components/research-projects/LibraryScreen.svelte`.

Add import:

```svelte
  import LibraryAddSourceDialog from "./LibraryAddSourceDialog.svelte";
```

Add state next to `status`:

```svelte
  let addSourceDialogOpen = $state(false);
```

Remove the `prototypeFeedback(action: string)` function and replace it with:

```svelte
  async function handleSourcesChanged(sourceId?: number) {
    await onRefresh();
    if (sourceId) {
      selectedSourceId = `source:${sourceId}`;
    }
  }
```

In the `LibraryWorkspace` props, replace:

```svelte
    onAdd={() => prototypeFeedback("Add source")}
```

with:

```svelte
    onAdd={() => (addSourceDialogOpen = true)}
```

Keep Edit/Delete placeholders by replacing their callbacks with inline status updates:

```svelte
    onEdit={() => (status = "Edit source flow is not implemented in this prototype.")}
    onDelete={() => (status = "Delete source flow is not implemented in this prototype.")}
```

Add the dialog before the status block:

```svelte
  <LibraryAddSourceDialog
    bind:open={addSourceDialogOpen}
    sources={workflowState.sources}
    onSourcesChanged={handleSourcesChanged}
    onStatus={(message) => (status = message)}
  />
```

- [ ] **Step 3: Run contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/library-add-source-contract.test.ts src/lib/library-prototype-contract.test.ts
```

Expected: PASS.

- [ ] **Step 4: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 5: Commit component slice**

Run:

```powershell
git add src/lib/components/research-projects/LibraryAddSourceDialog.svelte src/lib/components/research-projects/LibraryYoutubeAddPanel.svelte src/lib/components/research-projects/LibraryYoutubeSmartImport.svelte src/lib/components/research-projects/LibraryYoutubePlaylistImport.svelte src/lib/components/research-projects/LibraryTelegramDialogImport.svelte src/lib/components/research-projects/LibraryScreen.svelte src/lib/library-add-source-contract.test.ts src/lib/library-prototype-contract.test.ts
git commit -m "feat: add library source import dialog"
```

---

## Task 10: Tighten Boundary And Behavior Tests

**Files:**
- Modify: `src/lib/research-projects-import-boundary.test.ts`
- Modify: `src/lib/library-add-source-contract.test.ts`

- [ ] **Step 1: Extend boundary tests to cover Add Source files**

In `src/lib/research-projects-import-boundary.test.ts`, the existing feature-file scan already covers `src/lib/components/research-projects`. Add this explicit check to the `"keeps Library route and feature screens out of direct shadcn and SVAR imports"` test after `offenders` is computed:

```ts
    expect(libraryFiles.some((file) => path.basename(file) === "LibraryAddSourceDialog.svelte")).toBe(true);
```

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
```

Expected: PASS.

- [ ] **Step 2: Add contract checks for MVP limit and full catalog**

Modify `src/lib/library-add-source-contract.test.ts`.

In `"adds selected videos from existing playlist details"`, add:

```ts
    expect(playlistImportSource).toContain("playlistSelectionLimitMessage");
    expect(playlistImportSource).toContain("sources: LibraryCatalogSourceView[]");
```

In `"uses extractum wrappers for dialog and tabs"`, add:

```ts
    expect(dialogSource).toContain('data-ui-region="library-add-source-dialog"');
```

Run:

```powershell
npm.cmd run test -- src/lib/library-add-source-contract.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run full focused Library Add Source tests**

Run:

```powershell
npm.cmd run test -- src/lib/library-add-source-contract.test.ts src/lib/library-prototype-contract.test.ts src/lib/research-projects-import-boundary.test.ts src/lib/ui/library-add-source-model.test.ts src/lib/ui/library-add-source-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 4: Commit test tightening**

Run:

```powershell
git add src/lib/research-projects-import-boundary.test.ts src/lib/library-add-source-contract.test.ts
git commit -m "test: cover library add source boundaries"
```

---

## Task 11: Browser Verification And Polish

**Files:**
- Modify only files from earlier tasks if verification exposes defects.

- [ ] **Step 1: Run full frontend tests**

Run:

```powershell
npm.cmd run test
```

Expected: PASS.

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 3: Start or reuse dev server**

If no dev server is running, start one:

```powershell
npm.cmd run dev -- --host 127.0.0.1
```

Expected: Vite prints a local URL. Use the existing project port if already running.

- [ ] **Step 4: Browser verify `/projects/library`**

Open `/projects/library` in the browser.

Verify:

- Library route loads.
- Clicking `Add` opens a centered modal.
- Top provider tabs are `YouTube` and `Telegram`.
- YouTube tab has `Smart import` and `From existing data`.
- Pasting a YouTube channel URL such as `https://www.youtube.com/@tech_trends` shows `Not supported yet` without calling preview.
- YouTube playlist import shows existing playlist sources from the full Library catalog, not just visible table rows.
- Already linked playlist videos show `Already in Library` and are disabled.
- Add selected is disabled when no addable videos are selected.
- Telegram tab loads accounts and explains sign-in-required state for non-ready accounts.
- No direct shadcn/SVAR imports were introduced in Library feature files.
- There is no horizontal overflow at a desktop width around 1366px.

- [ ] **Step 5: Commit any verification fixes**

If browser verification required code changes, stage only the affected files and commit:

```powershell
git add <changed-files>
git commit -m "fix: polish library add source dialog"
```

If no code changes were needed, do not create an empty commit.

---

## Final Verification

Run:

```powershell
npm.cmd run test
```

Expected: PASS.

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

Run:

```powershell
git status --short --branch
```

Expected: clean tracked worktree.

## Completion Criteria

- Library Add opens a centered modal instead of prototype feedback.
- Add Source dialog uses provider tabs: YouTube and Telegram.
- YouTube uses inner tabs: Smart import and From existing data.
- YouTube Smart import supports video and playlist URL preview/add.
- YouTube channel URLs show `Not supported yet`.
- YouTube From existing data adds selected playlist videos as standalone video sources using `addYoutubeSource(canonicalUrl)`.
- YouTube playlist import uses the full Library catalog and enforces the 10-video MVP selection limit.
- Telegram import loads accounts/statuses, loads dialogs for a ready account, and adds the selected dialog with `sourceRef = String(dialog.id)`.
- Dialog and tabs are imported by Library feature files only through `extractum-ui`.
- Existing Connect from library workflow is unchanged.
- Focused tests, full Vitest suite, and Svelte check pass.
