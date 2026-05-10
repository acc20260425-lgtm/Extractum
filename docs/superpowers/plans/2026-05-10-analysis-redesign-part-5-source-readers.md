# Analysis Result-First Redesign Part 5 Source Readers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the transitional `ReportCanvas` source surface with real source readers for Telegram timelines, YouTube transcripts/playlists, and source groups while preserving the explicit run snapshot versus live source basis from Part 4.

**Architecture:** Add reader-specific data contracts and small normalization helpers, then render source material through focused Svelte components selected by source type, source subtype, scope, and source basis. Extend backend/frontend paging only where current APIs do not expose reader-critical data, especially Telegram reply/reaction metadata and YouTube transcript segments.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, TypeScript, Vitest raw-source and helper tests, Tauri commands, Rust/sqlx, existing Extractum UI components, lucide Svelte icons, Part 1 snapshot-only run message paging, Part 4 `ReportCanvas` and `ReportSourceSurface`.

---

## Prerequisites

Implement this part only after Parts 1 through 4 are implemented and committed, not merely planned.

This plan assumes these files already exist from earlier parts:

- `src/lib/components/analysis/report-canvas.svelte`
- `src/lib/components/analysis/report-source-surface.svelte`
- `src/lib/components/analysis/run-snapshot-messages-panel.svelte`
- `src/lib/analysis-report-canvas-state.ts`
- `src/lib/analysis-workspace-state.ts`
- `src/lib/api/analysis-runs.ts` with `listAnalysisRunMessages(...)`
- `src/lib/types/analysis.ts` with `AnalysisRunMessage`, `AnalysisRunMessageCursor`, and `AnalysisRunMessagesPage`

This plan also assumes Part 4 already wires:

- `canvasMode={workspaceUiState.canvasMode}`;
- `sourceViewBasis={workspaceUiState.sourceViewBasis}`;
- `runSnapshotMessages`;
- `loadingRunSnapshotMessages`;
- `runSnapshotError`;
- `onLoadMoreRunSnapshotMessages`;
- explicit `View live source` and `Back to run snapshot` actions.

If any prerequisite is missing, stop and implement the earlier part first.

This is **Part 5 of 7**. Stop after this part is implemented, verified, and committed. Continue to Part 6 only after explicit user approval.

## Part Boundary

Part 5 may:

- add backend and frontend paging for YouTube transcript segments;
- extend live `SourceItem` DTOs with already-stored Telegram reply/reaction metadata;
- extend run snapshot paging with optional source filtering if Part 1 did not already add it;
- add source-reader normalization helpers;
- create source-reader components used by `ReportSourceSurface`;
- render Telegram messages as a chronological timeline with media metadata cards;
- render YouTube videos as transcript-first readers with timestamp jump/copy actions;
- render YouTube playlists as playlist-first readers;
- render source groups grouped by source, both for live source browsing and run snapshot browsing;
- highlight a selected trace ref when that ref exists in the currently loaded reader items;
- keep source reader DOM bounded by paging and preview limits.

Part 5 must not:

- create or wire `RunCompanionTabs`;
- move evidence or chat out of Part 4's temporary locations;
- implement an embedded YouTube player;
- download or render local binary media previews;
- make live source data look like run snapshot data;
- resolve completed-run evidence against live source data when a snapshot is missing;
- close an opened run because the user filters or focuses a source inside a run snapshot;
- put source ingest jobs into saved or active analysis run history.

## File Structure

- Create: `src/lib/source-reader-model.ts`
  - Responsibility: normalize live `SourceItem`, run `AnalysisRunMessage`, YouTube transcript segment rows, media metadata, day groups, source groups, timestamps, and highlight decisions.
- Create: `src/lib/source-reader-model.test.ts`
  - Responsibility: helper coverage for Telegram grouping, media metadata, YouTube timestamp URLs, run snapshot grouping, source filtering, trace highlighting, and bounded preview labels.
- Create: `src/lib/analysis-source-readers.test.ts`
  - Responsibility: raw-source coverage for reader components, `ReportSourceSurface` wiring, live versus snapshot labeling, no `RunCompanionTabs`, no embedded player, and no binary media previews.
- Create: `src/lib/analysis-source-readers-route.test.ts`
  - Responsibility: raw-source coverage for route source-reader loading, live group paging, transcript paging, and snapshot source filtering.
- Modify: `src/lib/types/sources.ts`
  - Responsibility: expose reply/reaction metadata on `SourceItem` and define YouTube transcript page input/output types.
- Modify: `src/lib/api/sources.ts`
  - Responsibility: map new `SourceItem` fields and wrap `list_youtube_transcript_segments`.
- Modify: `src/lib/api/sources.test.ts`
  - Responsibility: verify new DTO mapping and command payloads.
- Modify: `src/lib/types/analysis.ts`
  - Responsibility: add optional `sourceId` to `ListAnalysisRunMessagesInput` when missing.
- Modify: `src/lib/api/analysis-runs.ts`
  - Responsibility: pass optional run snapshot `sourceId` through to Tauri.
- Modify: `src/lib/api/analysis-runs.test.ts`
  - Responsibility: verify snapshot source filtering payload.
- Modify: `src-tauri/src/sources/items.rs`
  - Responsibility: serialize reply/reaction metadata in `ItemRecord`.
- Modify: `src-tauri/src/sources/types.rs`
  - Responsibility: carry reply/reaction columns in `StoredItemRow`.
- Modify: `src-tauri/src/sources/items/query.rs`
  - Responsibility: select reply/reaction columns for live source readers.
- Create: `src-tauri/src/youtube/transcript_reader.rs`
  - Responsibility: expose paged YouTube transcript segments without loading entire transcripts.
- Modify: `src-tauri/src/youtube/mod.rs`
  - Responsibility: register `transcript_reader`.
- Modify: `src-tauri/src/lib.rs`
  - Responsibility: expose the new Tauri command.
- Modify: `src-tauri/src/analysis/corpus.rs`
  - Responsibility: add optional source filter to snapshot-only run message paging if Part 1 did not already add it.
- Modify: `src-tauri/src/analysis/mod.rs`
  - Responsibility: pass snapshot source filter into the snapshot-only command.
- Create: `src/lib/components/analysis/source-reader-header.svelte`
  - Responsibility: shared source reader title, basis badges, source count, selected-source filter controls, and reader search field.
- Create: `src/lib/components/analysis/telegram-media-card.svelte`
  - Responsibility: metadata-only media rendering for Telegram messages.
- Create: `src/lib/components/analysis/telegram-timeline-reader.svelte`
  - Responsibility: Telegram timeline grouped by day with topic, reply, reaction, media, and selected-ref highlighting.
- Create: `src/lib/components/analysis/youtube-transcript-reader.svelte`
  - Responsibility: transcript segment list with timestamp links, copy actions, search, sync status, paging, and selected-ref highlighting.
- Create: `src/lib/components/analysis/youtube-playlist-reader.svelte`
  - Responsibility: playlist-first source reader with per-video status and open/sync/retry actions.
- Create: `src/lib/components/analysis/source-group-reader.svelte`
  - Responsibility: group source material by source headings and delegate each group bucket to the correct reader.
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
  - Responsibility: use the new reader components for live and snapshot source material.
- Modify: `src/routes/analysis/+page.svelte`
  - Responsibility: load transcript pages, live source-group pages, and snapshot source-filter pages for the new readers.

## Task 1: Add Source Reader Contract Tests

**Files:**
- Create: `src/lib/source-reader-model.test.ts`
- Create: `src/lib/analysis-source-readers.test.ts`
- Create: `src/lib/analysis-source-readers-route.test.ts`

- [ ] **Step 1: Write failing helper tests**

Create `src/lib/source-reader-model.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  analysisRunMessageToReaderItem,
  formatYoutubeTime,
  groupReaderItemsByDay,
  groupReaderItemsBySource,
  sourceItemToReaderItem,
  youtubeTimestampUrl,
} from "./source-reader-model";
import type { AnalysisRunMessage } from "./types/analysis";
import type { SourceItem } from "./types/sources";

function sourceItem(overrides: Partial<SourceItem> = {}): SourceItem {
  return {
    id: 1,
    sourceId: 10,
    externalId: "100",
    itemKind: "telegram_message",
    author: "Alice",
    publishedAt: 1710000000,
    content: "Hello",
    contentKind: "text_only",
    hasMedia: false,
    mediaKind: null,
    mediaSummary: null,
    mediaFileName: null,
    mediaMimeType: null,
    hasRawData: true,
    forumTopicId: null,
    forumTopicTitle: null,
    forumTopicTopMessageId: null,
    replyToMessageId: null,
    replyToPeerKind: null,
    replyToPeerId: null,
    replyToTopMessageId: null,
    reactionCount: null,
    ...overrides,
  };
}

function runMessage(overrides: Partial<AnalysisRunMessage> = {}): AnalysisRunMessage {
  return {
    item_id: 4,
    source_id: 20,
    external_id: "transcript:v1:en:manual",
    author: "Demo Channel",
    published_at: 1710000200,
    ref: "s20-i4@754000ms",
    content: "Transcript text",
    item_kind: "youtube_transcript",
    source_type: "youtube",
    source_subtype: "video",
    metadata_json: {
      canonical_url: "https://www.youtube.com/watch?v=v1",
      start_ms: 754000,
      end_ms: 756500,
      caption_language: "en",
      caption_track_kind: "manual",
      item_kind: "youtube_transcript",
    },
    ...overrides,
  };
}

describe("source reader model", () => {
  it("normalizes live Telegram items with reply, reaction, topic, and media metadata", () => {
    const item = sourceItem({
      hasMedia: true,
      mediaKind: "photo",
      mediaSummary: "Image 1200x800",
      mediaFileName: "image.jpg",
      mediaMimeType: "image/jpeg",
      forumTopicId: 7,
      forumTopicTitle: "Announcements",
      replyToMessageId: 99,
      replyToTopMessageId: 7,
      reactionCount: 3,
    });

    const readerItem = sourceItemToReaderItem(item, { sourceTitle: "Telegram A" });

    expect(readerItem.kind).toBe("telegram_message");
    expect(readerItem.sourceId).toBe(10);
    expect(readerItem.sourceTitle).toBe("Telegram A");
    expect(readerItem.topicLabel).toBe("Announcements");
    expect(readerItem.replyLabel).toBe("Reply to #99");
    expect(readerItem.reactionLabel).toBe("3 reactions");
    expect(readerItem.mediaCards).toEqual([
      {
        kind: "photo",
        title: "Image",
        summary: "Image 1200x800",
        fileName: "image.jpg",
        mimeType: "image/jpeg",
      },
    ]);
  });

  it("normalizes run snapshot YouTube transcript metadata", () => {
    const readerItem = analysisRunMessageToReaderItem(runMessage(), { sourceTitle: "Video One" });

    expect(readerItem.kind).toBe("youtube_transcript");
    expect(readerItem.ref).toBe("s20-i4@754000ms");
    expect(readerItem.youtubeStartSeconds).toBe(754);
    expect(readerItem.youtubeUrl).toBe("https://www.youtube.com/watch?v=v1&t=754");
    expect(readerItem.captionLabel).toBe("en manual");
  });

  it("groups reader items by source without merging unrelated source material", () => {
    const groups = groupReaderItemsBySource([
      analysisRunMessageToReaderItem(runMessage({ source_id: 2, ref: "s2-i1" }), { sourceTitle: "Source 2" }),
      analysisRunMessageToReaderItem(runMessage({ source_id: 1, ref: "s1-i1" }), { sourceTitle: "Source 1" }),
    ]);

    expect(groups.map((group) => group.sourceId)).toEqual([1, 2]);
    expect(groups[0].sourceTitle).toBe("Source 1");
    expect(groups[1].sourceTitle).toBe("Source 2");
  });

  it("groups timeline items by UTC day", () => {
    const groups = groupReaderItemsByDay([
      sourceItemToReaderItem(sourceItem({ id: 1, publishedAt: 1710020000 }), { sourceTitle: "A" }),
      sourceItemToReaderItem(sourceItem({ id: 2, publishedAt: 1709900000 }), { sourceTitle: "A" }),
    ]);

    expect(groups).toHaveLength(2);
    expect(groups[0].items[0].id).toBe("live:1");
  });

  it("formats YouTube timestamps and appends canonical time links", () => {
    expect(formatYoutubeTime(0)).toBe("0:00");
    expect(formatYoutubeTime(754)).toBe("12:34");
    expect(formatYoutubeTime(3723)).toBe("1:02:03");
    expect(youtubeTimestampUrl("https://www.youtube.com/watch?v=v1", 754)).toBe(
      "https://www.youtube.com/watch?v=v1&t=754",
    );
    expect(youtubeTimestampUrl("https://www.youtube.com/watch?v=v1&t=1", 754)).toBe(
      "https://www.youtube.com/watch?v=v1&t=754",
    );
  });
});
```

- [ ] **Step 2: Write failing component raw-source tests**

Create `src/lib/analysis-source-readers.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import sourceGroupReaderSource from "./components/analysis/source-group-reader.svelte?raw";
import telegramMediaCardSource from "./components/analysis/telegram-media-card.svelte?raw";
import telegramTimelineSource from "./components/analysis/telegram-timeline-reader.svelte?raw";
import youtubePlaylistSource from "./components/analysis/youtube-playlist-reader.svelte?raw";
import youtubeTranscriptSource from "./components/analysis/youtube-transcript-reader.svelte?raw";

describe("analysis source readers", () => {
  it("replaces transitional source panels in ReportSourceSurface", () => {
    expect(reportSourceSurfaceSource).toContain("<TelegramTimelineReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubeTranscriptReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubePlaylistReader");
    expect(reportSourceSurfaceSource).toContain("<SourceGroupReader");
    expect(reportSourceSurfaceSource).not.toContain("<SourceContextPanel");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubeSourceDetail");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistDetail");
    expect(reportSourceSurfaceSource).not.toContain("<RunCompanionTabs");
  });

  it("keeps live source and run snapshot basis visible", () => {
    expect(sourceReaderHeaderSource).toContain("sourceViewBasis");
    expect(sourceReaderHeaderSource).toContain("Live source");
    expect(sourceReaderHeaderSource).toContain("Run snapshot");
    expect(sourceReaderHeaderSource).toContain("Back to run snapshot");
    expect(sourceReaderHeaderSource).toContain("View live source");
  });

  it("renders Telegram as a metadata-rich timeline without binary previews", () => {
    expect(telegramTimelineSource).toContain('class="telegram-timeline-reader"');
    expect(telegramTimelineSource).toContain("groupReaderItemsByDay");
    expect(telegramTimelineSource).toContain("topicLabel");
    expect(telegramTimelineSource).toContain("replyLabel");
    expect(telegramTimelineSource).toContain("reactionLabel");
    expect(telegramTimelineSource).toContain("<TelegramMediaCard");
    expect(telegramMediaCardSource).toContain("media-card");
    expect(telegramMediaCardSource).toContain("media.fileName");
    expect(telegramMediaCardSource).toContain("media.mimeType");
    expect(telegramMediaCardSource).not.toContain("<img");
    expect(telegramMediaCardSource).not.toContain("<video");
    expect(telegramMediaCardSource).not.toContain("<audio");
  });

  it("renders YouTube videos as transcript-first source readers", () => {
    expect(youtubeTranscriptSource).toContain('class="youtube-transcript-reader"');
    expect(youtubeTranscriptSource).toContain("formatYoutubeTime");
    expect(youtubeTranscriptSource).toContain("youtubeTimestampUrl");
    expect(youtubeTranscriptSource).toContain("Copy timestamp link");
    expect(youtubeTranscriptSource).toContain("Search transcript");
    expect(youtubeTranscriptSource).toContain("Load more transcript");
    expect(youtubeTranscriptSource).not.toContain("<iframe");
    expect(youtubeTranscriptSource).not.toContain("<video");
  });

  it("keeps YouTube playlist reading playlist-first", () => {
    expect(youtubePlaylistSource).toContain('class="youtube-playlist-reader"');
    expect(youtubePlaylistSource).toContain("playlist.items");
    expect(youtubePlaylistSource).toContain("onOpenSource");
    expect(youtubePlaylistSource).toContain("onSyncPlaylistVideo");
    expect(youtubePlaylistSource).toContain("onRetryPlaylistVideo");
  });

  it("groups source group material by source", () => {
    expect(sourceGroupReaderSource).toContain('class="source-group-reader"');
    expect(sourceGroupReaderSource).toContain("groupReaderItemsBySource");
    expect(sourceGroupReaderSource).toContain("source-heading");
    expect(sourceGroupReaderSource).toContain("selectedGroupSourceId");
    expect(sourceGroupReaderSource).toContain("onChangeSelectedGroupSourceId");
  });
});
```

- [ ] **Step 3: Write failing route raw-source tests**

Create `src/lib/analysis-source-readers-route.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis source reader route wiring", () => {
  it("loads live source group pages per member without closing the opened run", () => {
    expect(analysisPageSource).toContain("groupLiveItemsBySource");
    expect(analysisPageSource).toContain("loadLiveGroupSourcePage");
    expect(analysisPageSource).toContain("selectedGroupSourceId");
    expect(analysisPageSource).toContain("onChangeSelectedGroupSourceId={(sourceId) =>");
    expect(analysisPageSource).not.toContain("clearCurrentRunForWorkspaceSwitch(sourceId");
  });

  it("loads YouTube transcript segments through a paged API", () => {
    expect(analysisPageSource).toContain("listYoutubeTranscriptSegments");
    expect(analysisPageSource).toContain("youtubeTranscriptSegments");
    expect(analysisPageSource).toContain("youtubeTranscriptCursor");
    expect(analysisPageSource).toContain("loadYoutubeTranscriptFirstPage");
    expect(analysisPageSource).toContain("loadMoreYoutubeTranscriptSegments");
  });

  it("passes source reader props into ReportSourceSurface", () => {
    expect(analysisPageSource).toContain("{youtubeTranscriptSegments}");
    expect(analysisPageSource).toContain("{groupLiveItemsBySource}");
    expect(analysisPageSource).toContain("{selectedGroupSourceId}");
    expect(analysisPageSource).toContain("onLoadMoreYoutubeTranscriptSegments");
    expect(analysisPageSource).toContain("onLoadLiveGroupSourcePage");
    expect(analysisPageSource).toContain("onChangeSelectedGroupSourceId");
  });

  it("supports run snapshot source filtering through the snapshot-only API", () => {
    expect(analysisPageSource).toContain("selectedSnapshotSourceId");
    expect(analysisPageSource).toContain("sourceId: selectedSnapshotSourceId");
    expect(analysisPageSource).not.toContain("listSourceItems({ runId");
  });
});
```

- [ ] **Step 4: Run the focused tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/source-reader-model.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts
```

Expected: FAIL because the helper module and reader components do not exist and route wiring is not present.

- [ ] **Step 5: Commit the failing tests**

Run:

```powershell
git add src/lib/source-reader-model.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts
git commit -m "test: define analysis source reader contract"
```

## Task 2: Extend Source And Snapshot Data Contracts

**Files:**
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`
- Modify: `src/lib/api/sources.test.ts`
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-runs.ts`
- Modify: `src/lib/api/analysis-runs.test.ts`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/mod.rs`

- [ ] **Step 1: Add frontend source item and transcript page types**

In `src/lib/types/sources.ts`, extend `SourceItem` with Telegram context:

```ts
  replyToMessageId: number | null;
  replyToPeerKind: string | null;
  replyToPeerId: string | null;
  replyToTopMessageId: number | null;
  reactionCount: number | null;
```

Add YouTube transcript reader types:

```ts
export interface YoutubeTranscriptSegmentCursor {
  startMs: number;
  segmentId: number;
}

export interface YoutubeTranscriptSegment {
  id: number;
  sourceId: number;
  itemId: number;
  segmentIndex: number;
  startMs: number;
  endMs: number | null;
  text: string;
  captionLanguage: string | null;
  captionTrackKind: string | null;
  isAutoGenerated: boolean;
}

export interface YoutubeTranscriptSegmentsPage {
  segments: YoutubeTranscriptSegment[];
  nextCursor: YoutubeTranscriptSegmentCursor | null;
  hasMore: boolean;
}

export interface ListYoutubeTranscriptSegmentsInput {
  sourceId: number;
  after: YoutubeTranscriptSegmentCursor | null;
  limit: number;
  searchQuery: string | null;
}
```

In `src/lib/types/analysis.ts`, ensure `ListAnalysisRunMessagesInput` includes:

```ts
sourceId: number | null;
```

If Part 1 already added this field, keep the existing name and update later snippets in this plan to use that name.

- [ ] **Step 2: Map new frontend API fields**

In `src/lib/api/sources.ts`, import:

```ts
  ListYoutubeTranscriptSegmentsInput,
  YoutubeTranscriptSegmentsPage,
```

Extend `RawSourceItem`:

```ts
  reply_to_msg_id: number | null;
  reply_to_peer_kind: string | null;
  reply_to_peer_id: string | null;
  reply_to_top_id: number | null;
  reaction_count: number | null;
```

Add raw transcript types:

```ts
interface RawYoutubeTranscriptSegmentCursor {
  start_ms: number;
  segment_id: number;
}

interface RawYoutubeTranscriptSegment {
  id: number;
  source_id: number;
  item_id: number;
  segment_index: number;
  start_ms: number;
  end_ms: number | null;
  text: string;
  caption_language: string | null;
  caption_track_kind: string | null;
  is_auto_generated: boolean;
}

interface RawYoutubeTranscriptSegmentsPage {
  segments: RawYoutubeTranscriptSegment[];
  next_cursor: RawYoutubeTranscriptSegmentCursor | null;
  has_more: boolean;
}
```

Add the command name:

```ts
  listYoutubeTranscriptSegments: "list_youtube_transcript_segments",
```

Add the wrapper:

```ts
export function listYoutubeTranscriptSegments(input: ListYoutubeTranscriptSegmentsInput) {
  return invoke<RawYoutubeTranscriptSegmentsPage>(
    SOURCE_COMMANDS.listYoutubeTranscriptSegments,
    {
      request: {
        sourceId: input.sourceId,
        after: input.after
          ? {
              startMs: input.after.startMs,
              segmentId: input.after.segmentId,
            }
          : null,
        limit: input.limit,
        searchQuery: input.searchQuery,
      },
    },
  ).then(mapYoutubeTranscriptSegmentsPage);
}
```

Extend `mapSourceItem`:

```ts
    replyToMessageId: item.reply_to_msg_id,
    replyToPeerKind: item.reply_to_peer_kind,
    replyToPeerId: item.reply_to_peer_id,
    replyToTopMessageId: item.reply_to_top_id,
    reactionCount: item.reaction_count,
```

Add:

```ts
function mapYoutubeTranscriptSegmentsPage(
  page: RawYoutubeTranscriptSegmentsPage,
): YoutubeTranscriptSegmentsPage {
  return {
    segments: page.segments.map((segment) => ({
      id: segment.id,
      sourceId: segment.source_id,
      itemId: segment.item_id,
      segmentIndex: segment.segment_index,
      startMs: segment.start_ms,
      endMs: segment.end_ms,
      text: segment.text,
      captionLanguage: segment.caption_language,
      captionTrackKind: segment.caption_track_kind,
      isAutoGenerated: segment.is_auto_generated,
    })),
    nextCursor: page.next_cursor
      ? {
          startMs: page.next_cursor.start_ms,
          segmentId: page.next_cursor.segment_id,
        }
      : null,
    hasMore: page.has_more,
  };
}
```

- [ ] **Step 3: Add API wrapper tests**

In `src/lib/api/sources.test.ts`, extend the existing `listSourceItems` raw item fixture with:

```ts
reply_to_msg_id: 44,
reply_to_peer_kind: "channel",
reply_to_peer_id: "99",
reply_to_top_id: 12,
reaction_count: 5,
```

Assert mapped values:

```ts
expect(items[0].replyToMessageId).toBe(44);
expect(items[0].replyToPeerKind).toBe("channel");
expect(items[0].replyToPeerId).toBe("99");
expect(items[0].replyToTopMessageId).toBe(12);
expect(items[0].reactionCount).toBe(5);
```

Add a transcript wrapper test:

```ts
it("wraps paged YouTube transcript segment loading", async () => {
  invokeMock.mockResolvedValueOnce({
    segments: [
      {
        id: 9,
        source_id: 20,
        item_id: 4,
        segment_index: 2,
        start_ms: 754000,
        end_ms: 756500,
        text: "Transcript text",
        caption_language: "en",
        caption_track_kind: "manual",
        is_auto_generated: false,
      },
    ],
    next_cursor: {
      start_ms: 754000,
      segment_id: 9,
    },
    has_more: true,
  });

  await expect(listYoutubeTranscriptSegments({
    sourceId: 20,
    after: { startMs: 700000, segmentId: 8 },
    limit: 50,
    searchQuery: "text",
  })).resolves.toEqual({
    segments: [
      {
        id: 9,
        sourceId: 20,
        itemId: 4,
        segmentIndex: 2,
        startMs: 754000,
        endMs: 756500,
        text: "Transcript text",
        captionLanguage: "en",
        captionTrackKind: "manual",
        isAutoGenerated: false,
      },
    ],
    nextCursor: { startMs: 754000, segmentId: 9 },
    hasMore: true,
  });

  expect(invokeMock).toHaveBeenLastCalledWith("list_youtube_transcript_segments", {
    request: {
      sourceId: 20,
      after: { startMs: 700000, segmentId: 8 },
      limit: 50,
      searchQuery: "text",
    },
  });
});
```

In `src/lib/api/analysis-runs.test.ts`, extend the `listAnalysisRunMessages` test command payload:

```ts
sourceId: 20,
```

and assert the Tauri payload includes:

```ts
sourceId: 20,
```

- [ ] **Step 4: Run API tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/api/sources.test.ts src/lib/api/analysis-runs.test.ts
```

Expected: FAIL because DTO mappings and transcript wrapper are not implemented yet.

- [ ] **Step 5: Expose live Telegram context in Rust DTOs**

In `src-tauri/src/sources/types.rs`, extend `StoredItemRow`:

```rust
pub(super) reply_to_msg_id: Option<i64>,
pub(super) reply_to_peer_kind: Option<String>,
pub(super) reply_to_peer_id: Option<String>,
pub(super) reply_to_top_id: Option<i64>,
pub(super) reaction_count: Option<i64>,
```

In `src-tauri/src/sources/items/query.rs`, add these columns to the `SELECT`:

```sql
items.reply_to_msg_id,
items.reply_to_peer_kind,
items.reply_to_peer_id,
items.reply_to_top_id,
items.reaction_count,
```

In `src-tauri/src/sources/items.rs`, extend `ItemRecord`:

```rust
pub reply_to_msg_id: Option<i64>,
pub reply_to_peer_kind: Option<String>,
pub reply_to_peer_id: Option<String>,
pub reply_to_top_id: Option<i64>,
pub reaction_count: Option<i64>,
```

Extend `item_record_from_row`:

```rust
reply_to_msg_id: row.reply_to_msg_id,
reply_to_peer_kind: row.reply_to_peer_kind,
reply_to_peer_id: row.reply_to_peer_id,
reply_to_top_id: row.reply_to_top_id,
reaction_count: row.reaction_count,
```

Update tests in `src-tauri/src/sources/items/query.rs` so inserted rows bind a non-null `reaction_count` and assert the selected `StoredItemRow` contains the reply and reaction fields.

- [ ] **Step 6: Add YouTube transcript segment Tauri command**

Create `src-tauri/src/youtube/transcript_reader.rs`:

```rust
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Sqlite};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeTranscriptSegmentCursor {
    pub start_ms: i64,
    pub segment_id: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListYoutubeTranscriptSegmentsRequest {
    pub source_id: i64,
    pub after: Option<YoutubeTranscriptSegmentCursor>,
    pub limit: i64,
    pub search_query: Option<String>,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct YoutubeTranscriptSegmentDto {
    pub id: i64,
    pub source_id: i64,
    pub item_id: i64,
    pub segment_index: i64,
    pub start_ms: i64,
    pub end_ms: Option<i64>,
    pub text: String,
    pub caption_language: Option<String>,
    pub caption_track_kind: Option<String>,
    pub is_auto_generated: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct YoutubeTranscriptSegmentsPage {
    pub segments: Vec<YoutubeTranscriptSegmentDto>,
    pub next_cursor: Option<YoutubeTranscriptSegmentCursor>,
    pub has_more: bool,
}

pub(crate) async fn list_youtube_transcript_segments_from_pool(
    pool: &sqlx::SqlitePool,
    request: ListYoutubeTranscriptSegmentsRequest,
) -> AppResult<YoutubeTranscriptSegmentsPage> {
    let limit = request.limit.clamp(1, 200);
    let fetch_limit = limit + 1;
    let search = request
        .search_query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", value.replace('%', "\\%").replace('_', "\\_")));

    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            id,
            source_id,
            item_id,
            segment_index,
            start_ms,
            end_ms,
            text,
            caption_language,
            caption_track_kind,
            is_auto_generated
        FROM youtube_transcript_segments
        WHERE source_id =
        "#,
    );
    query.push_bind(request.source_id);

    if let Some(after) = request.after {
        query.push(" AND (start_ms, id) > (");
        query.push_bind(after.start_ms);
        query.push(", ");
        query.push_bind(after.segment_id);
        query.push(")");
    }

    if let Some(search) = search {
        query.push(" AND text LIKE ");
        query.push_bind(search);
        query.push(" ESCAPE '\\'");
    }

    query.push(" ORDER BY start_ms ASC, id ASC LIMIT ");
    query.push_bind(fetch_limit);

    let mut segments: Vec<YoutubeTranscriptSegmentDto> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    let has_more = segments.len() > limit as usize;
    if has_more {
        segments.truncate(limit as usize);
    }

    let next_cursor = if has_more {
        segments.last().map(|segment| YoutubeTranscriptSegmentCursor {
            start_ms: segment.start_ms,
            segment_id: segment.id,
        })
    } else {
        None
    };

    Ok(YoutubeTranscriptSegmentsPage {
        segments,
        next_cursor,
        has_more,
    })
}

#[tauri::command]
pub async fn list_youtube_transcript_segments(
    handle: AppHandle,
    request: ListYoutubeTranscriptSegmentsRequest,
) -> AppResult<YoutubeTranscriptSegmentsPage> {
    let pool = get_pool(&handle).await?;
    list_youtube_transcript_segments_from_pool(&pool, request).await
}
```

In `src-tauri/src/youtube/mod.rs`, add:

```rust
pub(crate) mod transcript_reader;
```

In `src-tauri/src/lib.rs`, import and register:

```rust
use youtube::transcript_reader::list_youtube_transcript_segments;
```

and add `list_youtube_transcript_segments` to the `tauri::generate_handler![...]` list.

- [ ] **Step 7: Add backend transcript reader tests**

In `src-tauri/src/youtube/transcript_reader.rs`, add a test module using an in-memory pool and this table shape:

```rust
#[cfg(test)]
mod tests {
    use super::{
        list_youtube_transcript_segments_from_pool, ListYoutubeTranscriptSegmentsRequest,
        YoutubeTranscriptSegmentCursor,
    };

    async fn transcript_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("create memory pool");
        sqlx::query(
            r#"
            CREATE TABLE youtube_transcript_segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                segment_index INTEGER NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER,
                text TEXT NOT NULL,
                chapter_index INTEGER,
                caption_language TEXT,
                caption_track_kind TEXT,
                is_auto_generated INTEGER NOT NULL DEFAULT 0,
                metadata_zstd BLOB
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create segments table");
        pool
    }

    async fn insert_segment(pool: &sqlx::SqlitePool, source_id: i64, start_ms: i64, text: &str) {
        sqlx::query(
            r#"
            INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text,
                caption_language, caption_track_kind, is_auto_generated
            ) VALUES (?, ?, ?, ?, ?, ?, 'en', 'manual', 0)
            "#,
        )
        .bind(10_i64)
        .bind(source_id)
        .bind(start_ms / 1000)
        .bind(start_ms)
        .bind(start_ms + 2000)
        .bind(text)
        .execute(pool)
        .await
        .expect("insert segment");
    }

    #[tokio::test]
    async fn list_youtube_transcript_segments_pages_by_time_and_id() {
        let pool = transcript_pool().await;
        insert_segment(&pool, 20, 1000, "first").await;
        insert_segment(&pool, 20, 2000, "second").await;
        insert_segment(&pool, 20, 3000, "third").await;
        insert_segment(&pool, 21, 1000, "other source").await;

        let first = list_youtube_transcript_segments_from_pool(
            &pool,
            ListYoutubeTranscriptSegmentsRequest {
                source_id: 20,
                after: None,
                limit: 2,
                search_query: None,
            },
        )
        .await
        .expect("load first page");

        assert_eq!(first.segments.len(), 2);
        assert!(first.has_more);
        assert_eq!(first.next_cursor.as_ref().map(|cursor| cursor.start_ms), Some(2000));

        let second = list_youtube_transcript_segments_from_pool(
            &pool,
            ListYoutubeTranscriptSegmentsRequest {
                source_id: 20,
                after: first.next_cursor,
                limit: 2,
                search_query: None,
            },
        )
        .await
        .expect("load second page");

        assert_eq!(second.segments.len(), 1);
        assert_eq!(second.segments[0].text, "third");
        assert!(!second.has_more);
    }

    #[tokio::test]
    async fn list_youtube_transcript_segments_filters_by_search() {
        let pool = transcript_pool().await;
        insert_segment(&pool, 20, 1000, "alpha topic").await;
        insert_segment(&pool, 20, 2000, "beta topic").await;

        let page = list_youtube_transcript_segments_from_pool(
            &pool,
            ListYoutubeTranscriptSegmentsRequest {
                source_id: 20,
                after: None,
                limit: 20,
                search_query: Some("beta".to_string()),
            },
        )
        .await
        .expect("search transcript");

        assert_eq!(page.segments.len(), 1);
        assert_eq!(page.segments[0].text, "beta topic");
    }
}
```

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::transcript_reader::tests
```

Expected: PASS.

- [ ] **Step 8: Add optional source filtering to snapshot message paging**

If Part 1 did not already add source filtering, update its snapshot-only request in `src-tauri/src/analysis/corpus.rs`:

```rust
pub(crate) struct ListRunSnapshotMessagesRequest {
    pub run_id: i64,
    pub after: Option<AnalysisRunMessageCursor>,
    pub limit: i64,
    pub source_id: Option<i64>,
}
```

In the query:

```rust
if let Some(source_id) = request.source_id {
    query.push(" AND source_id = ");
    query.push_bind(source_id);
}
```

In the Tauri command request in `src-tauri/src/analysis/mod.rs`, add:

```rust
source_id: request.source_id,
```

In `src/lib/api/analysis-runs.ts`, pass:

```ts
sourceId: input.sourceId,
```

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::list_run_snapshot_messages_page_reads_saved_snapshot_only
```

Expected: PASS.

- [ ] **Step 9: Run all data contract checks**

Run:

```powershell
npm.cmd test -- src/lib/api/sources.test.ts src/lib/api/analysis-runs.test.ts
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests youtube::transcript_reader::tests analysis::corpus::tests::list_run_snapshot_messages_page_reads_saved_snapshot_only
```

Expected: PASS.

- [ ] **Step 10: Commit data contract changes**

Run:

```powershell
git add src/lib/types/sources.ts src/lib/api/sources.ts src/lib/api/sources.test.ts src/lib/types/analysis.ts src/lib/api/analysis-runs.ts src/lib/api/analysis-runs.test.ts src-tauri/src/sources/items.rs src-tauri/src/sources/types.rs src-tauri/src/sources/items/query.rs src-tauri/src/youtube/transcript_reader.rs src-tauri/src/youtube/mod.rs src-tauri/src/lib.rs src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/mod.rs
git commit -m "feat: expose source reader data contracts"
```

## Task 3: Add Source Reader Model Helpers

**Files:**
- Create: `src/lib/source-reader-model.ts`

- [ ] **Step 1: Implement source reader model helpers**

Create `src/lib/source-reader-model.ts`:

```ts
import type { AnalysisRunMessage } from "$lib/types/analysis";
import type { SourceItem, YoutubeTranscriptSegment } from "$lib/types/sources";

export type SourceReaderBasis = "live_source" | "run_snapshot";
export type SourceReaderKind =
  | "telegram_message"
  | "youtube_transcript"
  | "youtube_comment"
  | "youtube_description"
  | "generic_item";

export interface SourceReaderMediaCard {
  kind: string;
  title: string;
  summary: string | null;
  fileName: string | null;
  mimeType: string | null;
}

export interface SourceReaderItem {
  id: string;
  sourceId: number;
  sourceTitle: string;
  externalId: string;
  ref: string | null;
  kind: SourceReaderKind;
  author: string | null;
  publishedAt: number;
  content: string;
  topicLabel: string | null;
  replyLabel: string | null;
  reactionLabel: string | null;
  mediaCards: SourceReaderMediaCard[];
  youtubeStartSeconds: number | null;
  youtubeEndSeconds: number | null;
  youtubeUrl: string | null;
  captionLabel: string | null;
  selected: boolean;
}

export interface SourceReaderDayGroup {
  key: string;
  label: string;
  items: SourceReaderItem[];
}

export interface SourceReaderSourceGroup {
  sourceId: number;
  sourceTitle: string;
  items: SourceReaderItem[];
}

export function sourceItemToReaderItem(
  item: SourceItem,
  {
    sourceTitle,
    selectedTraceRef = null,
  }: { sourceTitle: string; selectedTraceRef?: string | null },
): SourceReaderItem {
  const ref = null;
  return {
    id: `live:${item.id}`,
    sourceId: item.sourceId,
    sourceTitle,
    externalId: item.externalId,
    ref,
    kind: itemKind(item.itemKind),
    author: item.author,
    publishedAt: item.publishedAt,
    content: item.content ?? (item.hasMedia ? "Media-only message" : ""),
    topicLabel: item.forumTopicTitle,
    replyLabel: replyLabel(item),
    reactionLabel: reactionLabel(item.reactionCount),
    mediaCards: mediaCardsFromSourceItem(item),
    youtubeStartSeconds: null,
    youtubeEndSeconds: null,
    youtubeUrl: null,
    captionLabel: null,
    selected: false,
  };
}

export function analysisRunMessageToReaderItem(
  message: AnalysisRunMessage,
  {
    sourceTitle,
    selectedTraceRef = null,
  }: { sourceTitle: string; selectedTraceRef?: string | null },
): SourceReaderItem {
  const metadata = metadataObject(message.metadata_json);
  const startSeconds = millisecondsToSeconds(numberValue(metadata.start_ms));
  const endSeconds = millisecondsToSeconds(numberValue(metadata.end_ms));
  const canonicalUrl = stringValue(metadata.canonical_url);
  const captionLanguage = stringValue(metadata.caption_language);
  const captionTrackKind = stringValue(metadata.caption_track_kind);
  return {
    id: `snapshot:${message.ref}`,
    sourceId: message.source_id,
    sourceTitle,
    externalId: message.external_id,
    ref: message.ref,
    kind: itemKind(message.item_kind),
    author: message.author,
    publishedAt: message.published_at,
    content: message.content || "No text content captured for this snapshot row.",
    topicLabel: stringValue(metadata.forum_topic_title),
    replyLabel: null,
    reactionLabel: null,
    mediaCards: mediaCardsFromMetadata(metadata),
    youtubeStartSeconds: startSeconds,
    youtubeEndSeconds: endSeconds,
    youtubeUrl: canonicalUrl && startSeconds !== null ? youtubeTimestampUrl(canonicalUrl, startSeconds) : canonicalUrl,
    captionLabel: [captionLanguage, captionTrackKind].filter(Boolean).join(" ") || null,
    selected: selectedTraceRef !== null && message.ref === selectedTraceRef,
  };
}

export function youtubeSegmentToReaderItem(
  segment: YoutubeTranscriptSegment,
  {
    sourceTitle,
    canonicalUrl,
    selectedTraceRef = null,
  }: { sourceTitle: string; canonicalUrl: string | null; selectedTraceRef?: string | null },
): SourceReaderItem {
  const startSeconds = millisecondsToSeconds(segment.startMs);
  const endSeconds = millisecondsToSeconds(segment.endMs);
  return {
    id: `youtube-segment:${segment.id}`,
    sourceId: segment.sourceId,
    sourceTitle,
    externalId: `segment:${segment.segmentIndex}`,
    ref: null,
    kind: "youtube_transcript",
    author: null,
    publishedAt: startSeconds ?? 0,
    content: segment.text,
    topicLabel: null,
    replyLabel: null,
    reactionLabel: null,
    mediaCards: [],
    youtubeStartSeconds: startSeconds,
    youtubeEndSeconds: endSeconds,
    youtubeUrl: canonicalUrl && startSeconds !== null ? youtubeTimestampUrl(canonicalUrl, startSeconds) : null,
    captionLabel: [segment.captionLanguage, segment.captionTrackKind].filter(Boolean).join(" ") || null,
    selected: false,
  };
}

export function groupReaderItemsByDay(items: SourceReaderItem[]): SourceReaderDayGroup[] {
  const grouped = new Map<string, SourceReaderItem[]>();
  for (const item of [...items].sort(compareReaderItems)) {
    const key = new Date(item.publishedAt * 1000).toISOString().slice(0, 10);
    grouped.set(key, [...(grouped.get(key) ?? []), item]);
  }
  return [...grouped.entries()].map(([key, groupedItems]) => ({
    key,
    label: key,
    items: groupedItems,
  }));
}

export function groupReaderItemsBySource(items: SourceReaderItem[]): SourceReaderSourceGroup[] {
  const grouped = new Map<number, SourceReaderItem[]>();
  for (const item of [...items].sort(compareReaderItems)) {
    grouped.set(item.sourceId, [...(grouped.get(item.sourceId) ?? []), item]);
  }
  return [...grouped.entries()]
    .sort(([left], [right]) => left - right)
    .map(([sourceId, groupedItems]) => ({
      sourceId,
      sourceTitle: groupedItems[0]?.sourceTitle ?? `Source ${sourceId}`,
      items: groupedItems,
    }));
}

export function formatYoutubeTime(totalSeconds: number) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

export function youtubeTimestampUrl(canonicalUrl: string, seconds: number) {
  const url = new URL(canonicalUrl);
  url.searchParams.set("t", String(seconds));
  return url.toString();
}

function compareReaderItems(left: SourceReaderItem, right: SourceReaderItem) {
  return (
    left.publishedAt - right.publishedAt ||
    left.sourceId - right.sourceId ||
    left.id.localeCompare(right.id)
  );
}

function itemKind(value: string | null): SourceReaderKind {
  if (value === "telegram_message") return "telegram_message";
  if (value === "youtube_transcript") return "youtube_transcript";
  if (value === "youtube_comment") return "youtube_comment";
  if (value === "youtube_description") return "youtube_description";
  return "generic_item";
}

function replyLabel(item: Pick<SourceItem, "replyToMessageId" | "replyToTopMessageId">) {
  if (item.replyToMessageId !== null) return `Reply to #${item.replyToMessageId}`;
  if (item.replyToTopMessageId !== null) return `Thread #${item.replyToTopMessageId}`;
  return null;
}

function reactionLabel(value: number | null) {
  if (value === null || value <= 0) return null;
  return value === 1 ? "1 reaction" : `${value} reactions`;
}

function mediaCardsFromSourceItem(item: SourceItem): SourceReaderMediaCard[] {
  if (!item.hasMedia || !item.mediaKind) return [];
  return [
    {
      kind: item.mediaKind,
      title: mediaTitle(item.mediaKind),
      summary: item.mediaSummary,
      fileName: item.mediaFileName,
      mimeType: item.mediaMimeType,
    },
  ];
}

function mediaCardsFromMetadata(metadata: Record<string, unknown>): SourceReaderMediaCard[] {
  const mediaKind = stringValue(metadata.media_kind);
  if (!mediaKind) return [];
  return [
    {
      kind: mediaKind,
      title: mediaTitle(mediaKind),
      summary: stringValue(metadata.media_summary),
      fileName: stringValue(metadata.media_file_name),
      mimeType: stringValue(metadata.media_mime_type),
    },
  ];
}

function mediaTitle(kind: string) {
  if (kind.includes("photo") || kind.includes("image")) return "Image";
  if (kind.includes("video")) return "Video";
  if (kind.includes("document")) return "Document";
  return kind.replaceAll("_", " ");
}

function metadataObject(value: unknown): Record<string, unknown> {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return {};
}

function stringValue(value: unknown) {
  return typeof value === "string" && value.trim() ? value : null;
}

function numberValue(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function millisecondsToSeconds(value: number | null) {
  return value === null ? null : Math.floor(value / 1000);
}
```

- [ ] **Step 2: Run helper tests**

Run:

```powershell
npm.cmd test -- src/lib/source-reader-model.test.ts
```

Expected: PASS.

- [ ] **Step 3: Commit source reader helpers**

Run:

```powershell
git add src/lib/source-reader-model.ts src/lib/source-reader-model.test.ts
git commit -m "feat: add source reader model helpers"
```

## Task 4: Add Telegram Timeline Components

**Files:**
- Create: `src/lib/components/analysis/telegram-media-card.svelte`
- Create: `src/lib/components/analysis/telegram-timeline-reader.svelte`

- [ ] **Step 1: Create the Telegram media metadata card**

Create `src/lib/components/analysis/telegram-media-card.svelte`:

```svelte
<script lang="ts">
  import { FileText, Image, Paperclip, Video } from "@lucide/svelte";
  import type { SourceReaderMediaCard } from "$lib/source-reader-model";

  let { media }: { media: SourceReaderMediaCard } = $props();

  const Icon = $derived(iconForMedia(media.kind));

  function iconForMedia(kind: string) {
    if (kind.includes("photo") || kind.includes("image")) return Image;
    if (kind.includes("video")) return Video;
    if (kind.includes("document")) return FileText;
    return Paperclip;
  }
</script>

<div class="media-card">
  <Icon size={18} aria-hidden="true" />
  <div>
    <strong>{media.title}</strong>
    {#if media.summary}<span>{media.summary}</span>{/if}
    {#if media.fileName}<span>{media.fileName}</span>{/if}
    {#if media.mimeType}<span>{media.mimeType}</span>{/if}
  </div>
</div>
```

Add CSS:

```css
.media-card {
  display: flex;
  gap: 0.55rem;
  align-items: flex-start;
  padding: 0.65rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--panel-strong) 76%, transparent);
  color: var(--text);
}

.media-card div {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.media-card span {
  color: var(--muted);
  font-size: 0.78rem;
  overflow-wrap: anywhere;
}
```

- [ ] **Step 2: Create the Telegram timeline reader**

Create `src/lib/components/analysis/telegram-timeline-reader.svelte`:

```svelte
<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import TelegramMediaCard from "$lib/components/analysis/telegram-media-card.svelte";
  import { groupReaderItemsByDay, type SourceReaderItem } from "$lib/source-reader-model";

  let {
    items,
    loading,
    hasMore,
    contentLabel = "messages",
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceReaderItem[];
    loading: boolean;
    hasMore: boolean;
    contentLabel?: string;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();

  const dayGroups = $derived(groupReaderItemsByDay(items));
</script>

<section class="telegram-timeline-reader" aria-label="Telegram source timeline">
  {#if !loading && items.length === 0}
    <EmptyState description={`No synced ${contentLabel} are available for this source view.`} />
  {:else}
    <div class="timeline-days">
      {#each dayGroups as day (day.key)}
        <section class="timeline-day" aria-label={day.label}>
          <div class="day-label">{day.label}</div>
          <ul>
            {#each day.items as item (item.id)}
              <li class:selected={item.selected} tabindex={item.ref ? 0 : undefined}>
                <div class="message-meta">
                  <span>{formatTimestamp(item.publishedAt)}</span>
                  {#if item.author}<span>{item.author}</span>{/if}
                  {#if item.topicLabel}<Badge variant="neutral">{item.topicLabel}</Badge>{/if}
                  {#if item.replyLabel}<Badge variant="info">{item.replyLabel}</Badge>{/if}
                  {#if item.reactionLabel}<Badge variant="neutral">{item.reactionLabel}</Badge>{/if}
                  {#if item.ref}<Badge variant="neutral">{item.ref}</Badge>{/if}
                </div>
                <p>{item.content}</p>
                {#if item.mediaCards.length > 0}
                  <div class="media-list">
                    {#each item.mediaCards as media (`${item.id}:${media.kind}:${media.fileName ?? ""}`)}
                      <TelegramMediaCard {media} />
                    {/each}
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  {/if}

  {#if hasMore}
    <div class="reader-footer">
      <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
        {loading ? "Loading..." : "Load older messages"}
      </Button>
    </div>
  {/if}
</section>
```

Add CSS:

```css
.telegram-timeline-reader {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  min-width: 0;
}

.timeline-days,
.timeline-day,
.timeline-day ul {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.timeline-day ul {
  list-style: none;
  margin: 0;
  padding: 0;
}

.day-label {
  position: sticky;
  top: 0;
  z-index: 1;
  width: fit-content;
  padding: 0.25rem 0.55rem;
  border-radius: 999px;
  background: var(--panel);
  border: 1px solid var(--border);
  color: var(--muted);
  font-size: 0.75rem;
}

li {
  padding: 0.9rem 1rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel);
}

li.selected {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
}

.message-meta,
.media-list {
  display: flex;
  gap: 0.45rem;
  flex-wrap: wrap;
  align-items: center;
}

.message-meta {
  margin-bottom: 0.45rem;
  color: var(--muted);
  font-size: 0.78rem;
}

p {
  margin: 0;
  white-space: pre-wrap;
  line-height: 1.5;
}

.media-list {
  margin-top: 0.65rem;
}

.reader-footer {
  display: flex;
  justify-content: center;
}
```

- [ ] **Step 3: Run component checks**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers.test.ts
npm.cmd run check
```

Expected: `analysis-source-readers.test.ts` still FAILS until all reader components are present. `npm.cmd run check` PASS.

- [ ] **Step 4: Commit Telegram readers**

Run:

```powershell
git add src/lib/components/analysis/telegram-media-card.svelte src/lib/components/analysis/telegram-timeline-reader.svelte
git commit -m "feat: add telegram source timeline reader"
```

## Task 5: Add YouTube Source Readers

**Files:**
- Create: `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Create: `src/lib/components/analysis/youtube-playlist-reader.svelte`

- [ ] **Step 1: Create the YouTube transcript reader**

Create `src/lib/components/analysis/youtube-transcript-reader.svelte`:

```svelte
<script lang="ts">
  import { Copy, ExternalLink, Search } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import {
    formatYoutubeTime,
    youtubeSegmentToReaderItem,
    type SourceReaderItem,
  } from "$lib/source-reader-model";
  import type { SourceItem, YoutubeTranscriptSegment } from "$lib/types/sources";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    detail,
    segments,
    snapshotItems,
    loading,
    hasMore,
    transcriptSearch,
    sourceTitle,
    selectedTraceRef,
    formatTimestamp,
    onChangeTranscriptSearch,
    onLoadMore,
    onSyncTranscript,
    onSyncMetadata,
  }: {
    detail: YoutubeVideoDetail | null;
    segments: YoutubeTranscriptSegment[];
    snapshotItems: SourceReaderItem[];
    loading: boolean;
    hasMore: boolean;
    transcriptSearch: string;
    sourceTitle: string;
    selectedTraceRef: string | null;
    formatTimestamp: (value: number | null) => string;
    onChangeTranscriptSearch: (value: string) => void;
    onLoadMore: () => void | Promise<void>;
    onSyncTranscript: () => void | Promise<void>;
    onSyncMetadata: () => void | Promise<void>;
  } = $props();

  const summary = $derived(detail?.summary ?? null);
  const canonicalUrl = $derived(summary?.canonicalUrl ?? null);
  const liveItems = $derived(
    segments.map((segment) =>
      youtubeSegmentToReaderItem(segment, {
        sourceTitle,
        canonicalUrl,
        selectedTraceRef,
      }),
    ),
  );
  const readerItems = $derived(snapshotItems.length > 0 ? snapshotItems : liveItems);

  async function copyLink(item: SourceReaderItem) {
    if (!item.youtubeUrl || typeof navigator === "undefined") return;
    await navigator.clipboard.writeText(item.youtubeUrl);
  }
</script>

<section class="youtube-transcript-reader" aria-label="YouTube transcript reader">
  <div class="transcript-header">
    <div>
      <span class="eyebrow">YouTube transcript</span>
      <h3>{summary?.title ?? sourceTitle}</h3>
      <div class="transcript-meta">
        {#if summary}
          <Badge variant={summary.captions.state === "synced" ? "success" : summary.captions.state === "unavailable" ? "warning" : "neutral"}>
            {summary.captions.label}
          </Badge>
          <Badge variant="neutral">{summary.captions.segmentCount} segments</Badge>
          <Badge variant="neutral">Last synced {formatTimestamp(summary.captions.lastSyncedAt)}</Badge>
        {/if}
      </div>
    </div>
    <div class="transcript-actions">
      <Button type="button" size="sm" variant="secondary" onclick={onSyncMetadata}>Sync metadata</Button>
      <Button type="button" size="sm" variant="secondary" onclick={onSyncTranscript}>Sync transcript</Button>
    </div>
  </div>

  <label class="search-field">
    <span>Search transcript</span>
    <div class="search-shell">
      <Search size={15} aria-hidden="true" />
      <Input
        type="search"
        value={transcriptSearch}
        ariaLabel="Search transcript"
        oninput={(event) => onChangeTranscriptSearch((event.currentTarget as HTMLInputElement).value)}
      />
    </div>
  </label>

  {#if summary?.captions.state === "unavailable"}
    <StatusMessage tone="warning" surface={false}>
      Transcript unavailable for this video. Metadata and transcript sync actions remain available when the source supports retry.
    </StatusMessage>
  {/if}

  {#if !loading && readerItems.length === 0}
    <EmptyState description="No transcript segments are loaded for this source view." />
  {:else}
    <ol class="segment-list">
      {#each readerItems as item (item.id)}
        <li class:selected={item.selected} tabindex={item.ref ? 0 : undefined}>
          <div class="segment-time">
            {#if item.youtubeStartSeconds !== null && item.youtubeUrl}
              <a href={item.youtubeUrl} target="_blank" rel="noreferrer">
                {formatYoutubeTime(item.youtubeStartSeconds)}
                <ExternalLink size={13} aria-hidden="true" />
              </a>
            {:else if item.youtubeStartSeconds !== null}
              <span>{formatYoutubeTime(item.youtubeStartSeconds)}</span>
            {:else}
              <span>Transcript</span>
            {/if}
          </div>
          <p>{item.content}</p>
          <div class="segment-actions">
            {#if item.captionLabel}<Badge variant="neutral">{item.captionLabel}</Badge>{/if}
            {#if item.ref}<Badge variant="neutral">{item.ref}</Badge>{/if}
            {#if item.youtubeUrl}
              <Button type="button" size="sm" variant="ghost" ariaLabel="Copy timestamp link" title="Copy timestamp link" onclick={() => void copyLink(item)}>
                <Copy size={14} aria-hidden="true" />
              </Button>
            {/if}
          </div>
        </li>
      {/each}
    </ol>
  {/if}

  {#if hasMore}
    <div class="reader-footer">
      <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
        {loading ? "Loading..." : "Load more transcript"}
      </Button>
    </div>
  {/if}
</section>
```

Add CSS:

```css
.youtube-transcript-reader {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

.transcript-header {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  align-items: flex-start;
}

.eyebrow {
  display: inline-block;
  font-size: 0.68rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}

h3 {
  margin: 0.2rem 0 0 0;
}

.transcript-meta,
.transcript-actions,
.segment-actions,
.search-shell {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  align-items: center;
}

.search-field {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  color: var(--muted);
  font-size: 0.8rem;
}

.segment-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.segment-list li {
  display: grid;
  grid-template-columns: 5.5rem minmax(0, 1fr) auto;
  gap: 0.75rem;
  align-items: start;
  padding: 0.8rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel);
}

.segment-list li.selected {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
}

.segment-time a,
.segment-time span {
  display: inline-flex;
  gap: 0.25rem;
  align-items: center;
  color: var(--primary);
  text-decoration: none;
  font-variant-numeric: tabular-nums;
  font-weight: 700;
}

p {
  margin: 0;
  line-height: 1.5;
}

.reader-footer {
  display: flex;
  justify-content: center;
}

@media (max-width: 760px) {
  .transcript-header,
  .segment-list li {
    display: flex;
    flex-direction: column;
  }
}
```

- [ ] **Step 2: Create the YouTube playlist reader**

Create `src/lib/components/analysis/youtube-playlist-reader.svelte` by moving the playlist-first list from `YoutubePlaylistDetail` into a reader component that accepts:

```ts
sourceTitle: string;
playlist: YoutubePlaylistDetail | null;
loading: boolean;
formatTimestamp: (value: number | null) => string;
onOpenSource: (sourceId: number) => void | Promise<void>;
onSyncPlaylist: () => void | Promise<void>;
onRetryFailed: () => void | Promise<void>;
onSyncPlaylistVideo: (videoSourceId: number) => void | Promise<void>;
onRetryPlaylistVideo: (videoSourceId: number) => void | Promise<void>;
```

Use `StatusMessage`, `Badge`, and `Button`. Keep the primary surface as a playlist item list, not transcript rows. Keep per-video actions icon-based with `ariaLabel` and `title`:

```svelte
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
```

Show an empty state when `playlist?.items.length === 0`:

```svelte
<StatusMessage tone="muted" surface={false}>
  No linked videos are available for this playlist. Sync the playlist to load video rows.
</StatusMessage>
```

- [ ] **Step 3: Run YouTube reader checks**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers.test.ts
npm.cmd run check
```

Expected: `analysis-source-readers.test.ts` still FAILS until `SourceGroupReader` and `ReportSourceSurface` wiring are present. `npm.cmd run check` PASS.

- [ ] **Step 4: Commit YouTube readers**

Run:

```powershell
git add src/lib/components/analysis/youtube-transcript-reader.svelte src/lib/components/analysis/youtube-playlist-reader.svelte
git commit -m "feat: add youtube source readers"
```

## Task 6: Add Shared Reader Header And Source Group Reader

**Files:**
- Create: `src/lib/components/analysis/source-reader-header.svelte`
- Create: `src/lib/components/analysis/source-group-reader.svelte`

- [ ] **Step 1: Create the shared reader header**

Create `src/lib/components/analysis/source-reader-header.svelte`:

```svelte
<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { SourceViewBasis } from "$lib/analysis-workspace-state";

  let {
    title,
    subtitle,
    sourceViewBasis,
    canViewLiveSource,
    canBackToRunSnapshot,
    selectedSourceId,
    sourceOptions,
    onViewLiveSource,
    onBackToRunSnapshot,
    onChangeSelectedSourceId,
  }: {
    title: string;
    subtitle: string;
    sourceViewBasis: SourceViewBasis;
    canViewLiveSource: boolean;
    canBackToRunSnapshot: boolean;
    selectedSourceId: number | null;
    sourceOptions: Array<{ id: number; label: string; count: number }>;
    onViewLiveSource: () => void;
    onBackToRunSnapshot: () => void;
    onChangeSelectedSourceId: (sourceId: number | null) => void;
  } = $props();
</script>

<header class="source-reader-header">
  <div>
    <span class="eyebrow">Source reader</span>
    <h2>{title}</h2>
    <p>{subtitle}</p>
  </div>
  <div class="reader-actions">
    <Badge variant={sourceViewBasis === "live_source" ? "warning" : "success"}>
      {sourceViewBasis === "live_source" ? "Live source" : "Run snapshot"}
    </Badge>
    {#if sourceOptions.length > 1}
      <label>
        <span>Source focus</span>
        <Select
          value={selectedSourceId === null ? "__all_sources__" : String(selectedSourceId)}
          onchange={(event) => {
            const value = (event.currentTarget as HTMLSelectElement).value;
            onChangeSelectedSourceId(value === "__all_sources__" ? null : Number(value));
          }}
        >
          <option value="__all_sources__">All sources</option>
          {#each sourceOptions as option (option.id)}
            <option value={String(option.id)}>{option.label} ({option.count})</option>
          {/each}
        </Select>
      </label>
    {/if}
    {#if canViewLiveSource}
      <Button type="button" variant="secondary" onclick={onViewLiveSource}>View live source</Button>
    {/if}
    {#if canBackToRunSnapshot}
      <Button type="button" variant="secondary" onclick={onBackToRunSnapshot}>Back to run snapshot</Button>
    {/if}
  </div>
</header>
```

Add compact CSS with 8px radii.

- [ ] **Step 2: Create the source group reader**

Create `src/lib/components/analysis/source-group-reader.svelte`:

```svelte
<script lang="ts">
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import { groupReaderItemsBySource, type SourceReaderItem } from "$lib/source-reader-model";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    items,
    selectedGroupSourceId,
    loading,
    hasMoreBySource,
    youtubeDetailsBySource,
    formatTimestamp,
    onLoadMoreSource,
  }: {
    items: SourceReaderItem[];
    selectedGroupSourceId: number | null;
    loading: boolean;
    hasMoreBySource: Record<number, boolean>;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreSource: (sourceId: number) => void | Promise<void>;
  } = $props();

  const sourceGroups = $derived(groupReaderItemsBySource(
    selectedGroupSourceId === null
      ? items
      : items.filter((item) => item.sourceId === selectedGroupSourceId),
  ));
</script>

<section class="source-group-reader" aria-label="Source group reader">
  {#if !loading && sourceGroups.length === 0}
    <EmptyState description="No source material is loaded for this group view." />
  {:else}
    {#each sourceGroups as group (group.sourceId)}
      <section class="source-bucket" aria-label={group.sourceTitle}>
        <div class="source-heading">
          <h3>{group.sourceTitle}</h3>
          <span>{group.items.length} loaded items</span>
        </div>

        {#if group.items.some((item) => item.kind === "youtube_transcript")}
          <YoutubeTranscriptReader
            detail={youtubeDetailsBySource[group.sourceId] ?? null}
            segments={[]}
            snapshotItems={group.items}
            {loading}
            hasMore={hasMoreBySource[group.sourceId] ?? false}
            transcriptSearch=""
            sourceTitle={group.sourceTitle}
            selectedTraceRef={null}
            {formatTimestamp}
            onChangeTranscriptSearch={() => {}}
            onLoadMore={() => onLoadMoreSource(group.sourceId)}
            onSyncTranscript={() => {}}
            onSyncMetadata={() => {}}
          />
        {:else}
          <TelegramTimelineReader
            items={group.items}
            {loading}
            hasMore={hasMoreBySource[group.sourceId] ?? false}
            {formatTimestamp}
            onLoadMore={() => onLoadMoreSource(group.sourceId)}
          />
        {/if}
      </section>
    {/each}
  {/if}
</section>
```

Add CSS:

```css
.source-group-reader,
.source-bucket {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
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
```

The empty no-op callbacks above are acceptable only for snapshot group buckets. Live group YouTube bucket actions are wired in Task 7 when `ReportSourceSurface` knows whether it is displaying live source or a run snapshot.

- [ ] **Step 3: Run reader component tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers.test.ts
npm.cmd run check
```

Expected: `analysis-source-readers.test.ts` still FAILS until `ReportSourceSurface` wiring is updated. `npm.cmd run check` PASS.

- [ ] **Step 4: Commit shared reader components**

Run:

```powershell
git add src/lib/components/analysis/source-reader-header.svelte src/lib/components/analysis/source-group-reader.svelte
git commit -m "feat: add grouped source reader shell"
```

## Task 7: Wire Readers Into ReportSourceSurface

**Files:**
- Modify: `src/lib/components/analysis/report-source-surface.svelte`

- [ ] **Step 1: Add new props to `ReportSourceSurface`**

Add props:

```ts
youtubeTranscriptSegments: YoutubeTranscriptSegment[];
loadingYoutubeTranscriptSegments: boolean;
youtubeTranscriptHasMore: boolean;
youtubeTranscriptSearch: string;
groupLiveItemsBySource: Record<number, SourceItem[]>;
groupLiveHasMoreBySource: Record<number, boolean>;
selectedGroupSourceId: number | null;
selectedSnapshotSourceId: number | null;
onChangeTranscriptSearch: (value: string) => void;
onLoadMoreYoutubeTranscriptSegments: () => void | Promise<void>;
onLoadLiveGroupSourcePage: (sourceId: number) => void | Promise<void>;
onChangeSelectedGroupSourceId: (sourceId: number | null) => void;
onChangeSelectedSnapshotSourceId: (sourceId: number | null) => void;
```

Import new readers and helpers:

```ts
import SourceReaderHeader from "$lib/components/analysis/source-reader-header.svelte";
import SourceGroupReader from "$lib/components/analysis/source-group-reader.svelte";
import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
import YoutubePlaylistReader from "$lib/components/analysis/youtube-playlist-reader.svelte";
import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
import {
  analysisRunMessageToReaderItem,
  sourceItemToReaderItem,
  type SourceReaderItem,
} from "$lib/source-reader-model";
```

- [ ] **Step 2: Derive live and snapshot reader items**

Inside the script, add derived reader item arrays:

```ts
const liveReaderItems = $derived.by(() =>
  sourceItems.map((item) =>
    sourceItemToReaderItem(item, {
      sourceTitle: currentSource?.title ?? currentSource?.externalId ?? `Source ${item.sourceId}`,
      selectedTraceRef,
    }),
  ),
);

const snapshotReaderItems = $derived.by(() =>
  runSnapshotMessages
    .filter((message) => selectedSnapshotSourceId === null || message.source_id === selectedSnapshotSourceId)
    .map((message) =>
      analysisRunMessageToReaderItem(message, {
        sourceTitle: sourceTitleForSnapshotMessage(message.source_id),
        selectedTraceRef,
      }),
    ),
);

const groupLiveReaderItems = $derived.by(() =>
  Object.entries(groupLiveItemsBySource).flatMap(([sourceId, items]) => {
    const source = groupMemberSource(Number(sourceId));
    const sourceTitle = source?.source_title ?? `Source ${sourceId}`;
    return items.map((item) => sourceItemToReaderItem(item, { sourceTitle, selectedTraceRef }));
  }),
);
```

Add helper functions:

```ts
function sourceTitleForSnapshotMessage(sourceId: number) {
  if (currentSource?.id === sourceId) return currentSource.title ?? currentSource.externalId;
  const member = currentGroup?.members.find((candidate) => candidate.source_id === sourceId);
  return member?.source_title ?? `Source ${sourceId}`;
}

function groupMemberSource(sourceId: number) {
  return currentGroup?.members.find((member) => member.source_id === sourceId) ?? null;
}

function sourceFilterOptions(items: SourceReaderItem[]) {
  const counts = new Map<number, { label: string; count: number }>();
  for (const item of items) {
    const current = counts.get(item.sourceId) ?? { label: item.sourceTitle, count: 0 };
    counts.set(item.sourceId, { label: current.label, count: current.count + 1 });
  }
  return [...counts.entries()]
    .sort(([left], [right]) => left - right)
    .map(([id, value]) => ({ id, label: value.label, count: value.count }));
}
```

- [ ] **Step 3: Replace live source transitional snippet**

Replace the old `liveSourceSurface` snippet body:

- For single-source Telegram:

```svelte
<TelegramTimelineReader
  items={liveReaderItems}
  loading={loadingItems}
  hasMore={sourceItems.length >= 120}
  contentLabel={currentSourceContentLabel}
  {formatTimestamp}
  onLoadMore={onLoadMoreSourceItems}
/>
```

- For single-source YouTube video:

```svelte
<YoutubeTranscriptReader
  detail={youtubeVideoDetail}
  segments={youtubeTranscriptSegments}
  snapshotItems={[]}
  loading={loadingYoutubeTranscriptSegments || loadingYoutubeDetail}
  hasMore={youtubeTranscriptHasMore}
  transcriptSearch={youtubeTranscriptSearch}
  sourceTitle={currentSource.title ?? currentSource.externalId}
  {selectedTraceRef}
  {formatTimestamp}
  onChangeTranscriptSearch={onChangeTranscriptSearch}
  onLoadMore={onLoadMoreYoutubeTranscriptSegments}
  onSyncTranscript={() => onSyncYoutubeTranscript(currentSource.id)}
  onSyncMetadata={() => onSyncYoutubeMetadata(currentSource.id)}
/>
```

- For single-source YouTube playlist:

```svelte
<YoutubePlaylistReader
  sourceTitle={currentSource.title ?? currentSource.externalId}
  playlist={youtubePlaylistDetail}
  loading={loadingYoutubeDetail}
  {formatTimestamp}
  onOpenSource={onOpenSource}
  onSyncPlaylist={() => onSyncYoutubePlaylist(currentSource.id)}
  onRetryFailed={() => onRetryFailedYoutubePlaylistVideos(currentSource.id)}
  onSyncPlaylistVideo={(videoSourceId) => onSyncYoutubePlaylistVideo(currentSource.id, videoSourceId)}
  onRetryPlaylistVideo={(videoSourceId) => onRetryYoutubePlaylistVideo(currentSource.id, videoSourceId)}
/>
```

- For live source groups:

```svelte
<SourceGroupReader
  items={groupLiveReaderItems}
  selectedGroupSourceId={selectedGroupSourceId}
  loading={loadingItems}
  hasMoreBySource={groupLiveHasMoreBySource}
  youtubeDetailsBySource={{}}
  {formatTimestamp}
  onLoadMoreSource={onLoadLiveGroupSourcePage}
/>
```

- [ ] **Step 4: Replace snapshot source rendering**

When `sourceViewBasis === "run_snapshot"` and `snapshotAvailability === "available"`, render:

```svelte
<SourceReaderHeader
  title="Run snapshot"
  subtitle="Frozen source material captured for the opened run."
  {sourceViewBasis}
  canViewLiveSource={!!currentRun}
  canBackToRunSnapshot={false}
  selectedSourceId={selectedSnapshotSourceId}
  sourceOptions={sourceFilterOptions(snapshotReaderItems)}
  {onViewLiveSource}
  {onBackToRunSnapshot}
  onChangeSelectedSourceId={onChangeSelectedSnapshotSourceId}
/>

{#if currentRun?.scope_type === "source_group"}
  <SourceGroupReader
    items={snapshotReaderItems}
    selectedGroupSourceId={selectedSnapshotSourceId}
    loading={loadingRunSnapshotMessages}
    hasMoreBySource={{}}
    youtubeDetailsBySource={{}}
    {formatTimestamp}
    onLoadMoreSource={() => onLoadMoreRunSnapshotMessages()}
  />
{:else if snapshotReaderItems.some((item) => item.kind === "youtube_transcript")}
  <YoutubeTranscriptReader
    detail={youtubeVideoDetail}
    segments={[]}
    snapshotItems={snapshotReaderItems}
    loading={loadingRunSnapshotMessages}
    hasMore={hasMoreRunSnapshotMessages}
    transcriptSearch=""
    sourceTitle={currentScopeTitle}
    {selectedTraceRef}
    {formatTimestamp}
    onChangeTranscriptSearch={() => {}}
    onLoadMore={onLoadMoreRunSnapshotMessages}
    onSyncTranscript={() => {}}
    onSyncMetadata={() => {}}
  />
{:else}
  <TelegramTimelineReader
    items={snapshotReaderItems}
    loading={loadingRunSnapshotMessages}
    hasMore={hasMoreRunSnapshotMessages}
    {formatTimestamp}
    onLoadMore={onLoadMoreRunSnapshotMessages}
  />
{/if}
```

This keeps run snapshot source filtering local to the reader and does not call rail selection callbacks.

- [ ] **Step 5: Keep basis actions prominent**

At the top of `ReportSourceSurface`, render `SourceReaderHeader` for live source mode too:

```svelte
<SourceReaderHeader
  title={currentRun && sourceViewBasis === "live_source" ? "Live source" : currentScopeTitle}
  subtitle={sourceBasisDescription({ currentRun, sourceViewBasis, snapshotAvailability })}
  {sourceViewBasis}
  canViewLiveSource={currentRun && sourceViewBasis === "run_snapshot" && snapshotAvailability !== "available"}
  canBackToRunSnapshot={currentRun && sourceViewBasis === "live_source" && canReturnToRunSnapshot(snapshotAvailability)}
  selectedSourceId={analysisScope === "source_group" ? selectedGroupSourceId : null}
  sourceOptions={analysisScope === "source_group" ? sourceFilterOptions(groupLiveReaderItems) : []}
  {onViewLiveSource}
  {onBackToRunSnapshot}
  onChangeSelectedSourceId={onChangeSelectedGroupSourceId}
/>
```

- [ ] **Step 6: Run component tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers.test.ts
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 7: Commit source surface wiring**

Run:

```powershell
git add src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: wire source readers into report canvas"
```

## Task 8: Wire Reader Loading In `/analysis`

**Files:**
- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Add imports**

In `src/routes/analysis/+page.svelte`, add:

```ts
import { listYoutubeTranscriptSegments } from "$lib/api/sources";
import type {
  SourceItem,
  YoutubeTranscriptSegment,
  YoutubeTranscriptSegmentCursor,
} from "$lib/types/sources";
```

- [ ] **Step 2: Add route state**

Near source item state:

```ts
let youtubeTranscriptSegments = $state<YoutubeTranscriptSegment[]>([]);
let youtubeTranscriptCursor = $state<YoutubeTranscriptSegmentCursor | null>(null);
let youtubeTranscriptHasMore = $state(false);
let youtubeTranscriptSearch = $state("");
let loadingYoutubeTranscriptSegments = $state(false);
let youtubeTranscriptRequestKey = "";

let groupLiveItemsBySource = $state<Record<number, SourceItem[]>>({});
let groupLiveCursorsBySource = $state<Record<number, number | null>>({});
let groupLiveHasMoreBySource = $state<Record<number, boolean>>({});
let groupLiveLoadingBySource = $state<Record<number, boolean>>({});
let selectedGroupSourceId = $state<number | null>(null);
let selectedSnapshotSourceId = $state<number | null>(null);
```

- [ ] **Step 3: Add YouTube transcript loaders**

Add:

```ts
function resetYoutubeTranscriptReader() {
  youtubeTranscriptSegments = [];
  youtubeTranscriptCursor = null;
  youtubeTranscriptHasMore = false;
  loadingYoutubeTranscriptSegments = false;
  youtubeTranscriptRequestKey = "";
}

async function loadYoutubeTranscriptFirstPage(sourceId: number) {
  const requestKey = `${sourceId}:${youtubeTranscriptSearch.trim()}`;
  youtubeTranscriptRequestKey = requestKey;
  loadingYoutubeTranscriptSegments = true;
  try {
    const page = await listYoutubeTranscriptSegments({
      sourceId,
      after: null,
      limit: 80,
      searchQuery: youtubeTranscriptSearch.trim() || null,
    });
    if (youtubeTranscriptRequestKey !== requestKey) {
      return;
    }
    youtubeTranscriptSegments = page.segments;
    youtubeTranscriptCursor = page.nextCursor;
    youtubeTranscriptHasMore = page.hasMore;
  } catch (error) {
    if (youtubeTranscriptRequestKey === requestKey) {
      youtubeTranscriptSegments = [];
      youtubeTranscriptCursor = null;
      youtubeTranscriptHasMore = false;
      status = formatAppError("loading YouTube transcript", error);
    }
  } finally {
    if (youtubeTranscriptRequestKey === requestKey) {
      loadingYoutubeTranscriptSegments = false;
    }
  }
}

async function loadMoreYoutubeTranscriptSegments() {
  const source = currentSource();
  if (!source || source.sourceType !== "youtube" || source.sourceSubtype !== "video") return;
  if (!youtubeTranscriptCursor || loadingYoutubeTranscriptSegments) return;

  const requestKey = `${source.id}:${youtubeTranscriptSearch.trim()}`;
  loadingYoutubeTranscriptSegments = true;
  try {
    const page = await listYoutubeTranscriptSegments({
      sourceId: source.id,
      after: youtubeTranscriptCursor,
      limit: 80,
      searchQuery: youtubeTranscriptSearch.trim() || null,
    });
    if (youtubeTranscriptRequestKey !== requestKey) {
      return;
    }
    youtubeTranscriptSegments = [...youtubeTranscriptSegments, ...page.segments];
    youtubeTranscriptCursor = page.nextCursor;
    youtubeTranscriptHasMore = page.hasMore;
  } catch (error) {
    status = formatAppError("loading more YouTube transcript", error);
  } finally {
    if (youtubeTranscriptRequestKey === requestKey) {
      loadingYoutubeTranscriptSegments = false;
    }
  }
}

function changeYoutubeTranscriptSearch(value: string) {
  youtubeTranscriptSearch = value;
  const source = currentSource();
  if (source?.sourceType === "youtube" && source.sourceSubtype === "video") {
    void loadYoutubeTranscriptFirstPage(source.id);
  }
}
```

- [ ] **Step 4: Add live source group loaders**

Add:

```ts
async function loadLiveGroupSourcePage(sourceId: number) {
  if (groupLiveLoadingBySource[sourceId]) return;
  groupLiveLoadingBySource = { ...groupLiveLoadingBySource, [sourceId]: true };
  try {
    const beforePublishedAt = groupLiveCursorsBySource[sourceId] ?? null;
    const items = await listSourceItems({
      sourceId,
      limit: 40,
      beforePublishedAt,
      topicFilter: null,
    });
    groupLiveItemsBySource = {
      ...groupLiveItemsBySource,
      [sourceId]: [...(groupLiveItemsBySource[sourceId] ?? []), ...items],
    };
    groupLiveCursorsBySource = {
      ...groupLiveCursorsBySource,
      [sourceId]: items.at(-1)?.publishedAt ?? beforePublishedAt,
    };
    groupLiveHasMoreBySource = {
      ...groupLiveHasMoreBySource,
      [sourceId]: items.length === 40,
    };
  } catch (error) {
    status = formatAppError("loading group source material", error);
  } finally {
    const next = { ...groupLiveLoadingBySource };
    delete next[sourceId];
    groupLiveLoadingBySource = next;
  }
}

function resetGroupLiveReader() {
  groupLiveItemsBySource = {};
  groupLiveCursorsBySource = {};
  groupLiveHasMoreBySource = {};
  groupLiveLoadingBySource = {};
  selectedGroupSourceId = null;
}
```

- [ ] **Step 5: Load reader pages when source mode needs them**

Add effects:

```ts
$effect(() => {
  const source = currentSource();
  if (
    workspaceUiState.canvasMode === "source" &&
    workspaceUiState.sourceViewBasis === "live_source" &&
    source?.sourceType === "youtube" &&
    source.sourceSubtype === "video"
  ) {
    void loadYoutubeTranscriptFirstPage(source.id);
  }
});

$effect(() => {
  const group = currentGroup();
  if (
    workspaceUiState.canvasMode === "source" &&
    workspaceUiState.sourceViewBasis === "live_source" &&
    analysisScope === "source_group" &&
    group
  ) {
    for (const member of group.members.slice(0, 6)) {
      if (!groupLiveItemsBySource[member.source_id]) {
        void loadLiveGroupSourcePage(member.source_id);
      }
    }
  }
});
```

If these effects re-run too often after implementation, derive stable keys first and guard with explicit request keys, matching the existing route workflow patterns.

- [ ] **Step 6: Reset reader state on workspace switches**

Inside `selectSource(...)`, after clearing YouTube detail:

```ts
resetGroupLiveReader();
selectedSnapshotSourceId = null;
resetYoutubeTranscriptReader();
```

Inside `selectGroup(...)`:

```ts
resetGroupLiveReader();
selectedSnapshotSourceId = null;
resetYoutubeTranscriptReader();
```

Inside `alignWorkspaceToOpenedRun(...)`, reset only live reader paging:

```ts
resetGroupLiveReader();
resetYoutubeTranscriptReader();
selectedSnapshotSourceId = null;
```

Do not call `clearCurrentRunForWorkspaceSwitch()` from any reader-local source filter handler.

- [ ] **Step 7: Add source filter to snapshot loading**

In `loadRunSnapshotFirstPage(...)` and `loadMoreRunSnapshotMessages(...)`, include:

```ts
sourceId: selectedSnapshotSourceId,
```

When `selectedSnapshotSourceId` changes:

```ts
function changeSelectedSnapshotSourceId(sourceId: number | null) {
  selectedSnapshotSourceId = sourceId;
  resetRunSnapshotState();
  if (currentRun && workspaceUiState.canvasMode === "source" && workspaceUiState.sourceViewBasis === "run_snapshot") {
    void loadRunSnapshotFirstPage(currentRun.id);
  }
}
```

- [ ] **Step 8: Pass reader props to ReportSourceSurface**

Add to `<ReportSourceSurface ... />`:

```svelte
      {youtubeTranscriptSegments}
      {loadingYoutubeTranscriptSegments}
      {youtubeTranscriptHasMore}
      {youtubeTranscriptSearch}
      {groupLiveItemsBySource}
      {groupLiveHasMoreBySource}
      {selectedGroupSourceId}
      {selectedSnapshotSourceId}
      onChangeTranscriptSearch={changeYoutubeTranscriptSearch}
      onLoadMoreYoutubeTranscriptSegments={() => void loadMoreYoutubeTranscriptSegments()}
      onLoadLiveGroupSourcePage={(sourceId) => void loadLiveGroupSourcePage(sourceId)}
      onChangeSelectedGroupSourceId={(sourceId) => (selectedGroupSourceId = sourceId)}
      onChangeSelectedSnapshotSourceId={changeSelectedSnapshotSourceId}
```

- [ ] **Step 9: Run route reader tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-readers-route.test.ts src/lib/analysis-source-readers.test.ts
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 10: Commit route wiring**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-source-readers-route.test.ts
git commit -m "feat: load analysis source reader pages"
```

## Task 9: Run Part 5 Verification

**Files:**
- Verify all Part 5 files changed in Tasks 1-8.

- [ ] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/source-reader-model.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/api/sources.test.ts src/lib/api/analysis-runs.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run focused backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests youtube::transcript_reader::tests analysis::corpus::tests::list_run_snapshot_messages_page_reads_saved_snapshot_only
```

Expected: PASS.

- [ ] **Step 3: Run analysis state and route tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-canvas-route.test.ts src/lib/analysis-route-workspace-state.test.ts src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts
```

Expected: PASS.

- [ ] **Step 4: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 5: Run full frontend test suite**

Run:

```powershell
npm.cmd test
```

Expected: PASS.

- [ ] **Step 6: Run full backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 7: Check source reader boundaries**

Run:

```powershell
rg -n "RunCompanionTabs|<iframe|<video|<audio|listSourceItems\\(\\{ runId|sourceViewBasis: \"run_snapshot\", // automatic" src/lib/components/analysis src/routes/analysis/+page.svelte src/lib/source-reader-model.ts
```

Expected: no output for `RunCompanionTabs`, embedded media players, live-source snapshot fallback, or automatic snapshot switching.

Run:

```powershell
rg -n "SourceContextPanel|YoutubeSourceDetail|YoutubePlaylistDetail" src/lib/components/analysis/report-source-surface.svelte
```

Expected: no output. `ReportSourceSurface` should use Part 5 source readers.

- [ ] **Step 8: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [ ] **Step 9: Commit final fixes if needed**

If verification required fixes, commit them:

```powershell
git add src src-tauri
git commit -m "test: verify analysis source readers"
```

Skip this commit if Tasks 1-8 already have a clean verified tree.

- [ ] **Step 10: Stop for review**

Run:

```powershell
git status --short
```

Expected: clean working tree.

Report:

```text
Part 5 source readers are implemented and verified. Stopping before Part 6.
```

Do not begin Part 6 until the user explicitly approves continuing.

## Self-Review

- Spec coverage: this plan covers Telegram chronological timelines, topic/reply/reaction/media metadata, YouTube transcript-first reading, timestamp jump/copy actions, transcript search, playlist-first source view, source groups grouped by source, bounded paging, and selected trace highlighting hooks.
- Boundary check: this plan does not create `RunCompanionTabs`, does not move evidence or chat, does not embed a YouTube player, and does not implement binary media previews.
- Snapshot trust: snapshot readers use `AnalysisRunMessage` data from snapshot-only paging. Live readers use live source APIs only when `sourceViewBasis = "live_source"`.
- Data safety: large transcripts and source groups use explicit page loaders. The route does not hydrate whole archives into memory to enter `Source` mode.
- Type consistency: frontend `SourceItem`, `YoutubeTranscriptSegment`, `AnalysisRunMessage`, and backend DTO field names are mapped explicitly between snake_case and camelCase.
- Test coverage: tests cover helper normalization, reader component structure, route loading, API wrappers, backend transcript paging, and staged redesign boundaries.
