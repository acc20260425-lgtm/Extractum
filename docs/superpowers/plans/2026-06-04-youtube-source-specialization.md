# YouTube Source Specialization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make YouTube sources in Analysis workspace source-scoped, provider-aware, and report-ready, with invalid playlist detail shown as a problem state instead of a misleading empty state.

**Architecture:** Keep `/analysis` as the owner of Tauri calls and loading state. Add one pure YouTube view-model helper module, then thread source-scoped error and YouTube display data through `ReportCanvas`, `ReportSetupPanel`, `ReportSourceSurface`, `SourceBrowserShell`, and existing YouTube leaf components. Use raw-source contract tests and pure helper tests before Svelte edits.

**Tech Stack:** Svelte 5, SvelteKit, Tauri 2, Vitest, existing `$lib/components/ui/*` primitives, `@lucide/svelte`, `npm.cmd run test`, `npm.cmd run check`, `npm.cmd run smoke:analysis`.

---

## Design Inputs

- Spec: `docs/superpowers/specs/2026-06-04-youtube-source-specialization-design.md`
- Sketches:
  - `reference/ux-panel-sketches-2026-06-04/youtube-source-overview.html`
  - `reference/ux-panel-sketches-2026-06-04/youtube-report-corpus.html`
  - `reference/ux-panel-sketches-2026-06-04/youtube-playlist-problem.html`
  - `reference/ux-panel-sketches-2026-06-04/youtube-evidence-activity.html`

## File Structure

Expected new files:

- Create: `src/lib/youtube-source-view-model.ts`
- Create: `src/lib/youtube-source-view-model.test.ts`
- Create: `src/lib/analysis-youtube-source-specialization.test.ts`

Expected modified files:

- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/report-setup-panel.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Modify: `src/lib/components/analysis/youtube-comments-view.svelte`
- Modify: `src/lib/components/analysis/youtube-playlist-videos-view.svelte`
- Modify: `src/lib/components/analysis/universal-items-view.svelte`
- Modify: `src/lib/components/analysis/source-activity-view.svelte`
- Modify: `src/lib/source-browser-model.ts`
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `src/lib/analysis-report-setup-props.test.ts`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/source-browser-model.test.ts`
- Modify: `src/lib/analysis-state.test.ts`

---

### Task 1: Add YouTube View-Model Helpers And Contracts

**Files:**
- Create: `src/lib/youtube-source-view-model.ts`
- Create: `src/lib/youtube-source-view-model.test.ts`
- Create: `src/lib/analysis-youtube-source-specialization.test.ts`

- [x] **Step 1: Write failing helper tests**

Create `src/lib/youtube-source-view-model.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  detailErrorForYoutubeSource,
  formatYoutubeDuration,
  youtubeContentStatusLine,
  youtubeCorpusOptionViews,
  youtubeProviderHeaderSummary,
  type YoutubeDetailErrorState,
} from "./youtube-source-view-model";
import type { Source } from "$lib/types/sources";
import type { YoutubeVideoDetail } from "$lib/types/youtube";

function youtubeSource(overrides: Partial<Source> = {}): Source {
  return {
    id: 66,
    sourceType: "youtube",
    sourceSubtype: "video",
    accountId: null,
    externalId: "2ZMbW3Qiv6U",
    title: "Gemma video",
    lastSyncState: null,
    lastSyncedAt: 1_800_000_000,
    isMember: null,
    isActive: true,
    createdAt: 1_779_916_800,
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

function videoDetail(overrides: Partial<YoutubeVideoDetail["summary"]> = {}): YoutubeVideoDetail {
  return {
    summary: {
      sourceId: 66,
      sourceSubtype: "video",
      title: "Gemma 4 Desktop Coder by Google",
      channelTitle: "AI Stack Engineer",
      channelHandle: "@AIStackEngineer",
      canonicalUrl: "https://www.youtube.com/watch?v=2ZMbW3Qiv6U",
      thumbnailUrl: null,
      durationSeconds: 581,
      publishedAt: 1_779_916_800,
      availabilityStatus: "available",
      videoCount: null,
      linkedVideoCount: null,
      unavailableCount: null,
      captions: {
        state: "synced",
        itemCount: 1,
        segmentCount: 239,
        lastSyncedAt: 1_800_000_000,
        label: "Captions synced",
      },
      comments: {
        state: "synced",
        itemCount: 43,
        segmentCount: 0,
        lastSyncedAt: 1_800_000_100,
        label: "Comments synced",
      },
      ...overrides,
    },
    sourceMetadata: {
      sourceId: 66,
      videoId: "2ZMbW3Qiv6U",
      canonicalUrl: "https://www.youtube.com/watch?v=2ZMbW3Qiv6U",
      title: "Gemma 4 Desktop Coder by Google",
      channelTitle: "AI Stack Engineer",
      channelId: "UCdemo",
      channelHandle: "@AIStackEngineer",
      channelUrl: "https://www.youtube.com/@AIStackEngineer",
      authorDisplay: "AI Stack Engineer",
      publishedAt: 1_779_916_800,
      durationSeconds: 581,
      description: "Demo description",
      thumbnailUrl: null,
      viewCount: 24_355,
      likeCount: 527,
      commentCount: 43,
      category: "Science & Technology",
      videoForm: "regular",
      availabilityStatus: "available",
      captionLanguageOverride: null,
      rawMetadataVersion: 1,
      rawMetadataJson: null,
    },
    playlistMemberships: [],
  };
}

describe("youtube source view model", () => {
  it("formats youtube durations for videos and playlists", () => {
    expect(formatYoutubeDuration(null)).toBeNull();
    expect(formatYoutubeDuration(581)).toBe("9:41");
    expect(formatYoutubeDuration(8064)).toBe("2:14:24");
  });

  it("keeps detail errors scoped to the selected source", () => {
    const error: YoutubeDetailErrorState = {
      sourceId: 32,
      sourceSubtype: "playlist",
      message: "Source 32 has missing or invalid typed YouTube playlist metadata",
    };

    expect(detailErrorForYoutubeSource(error, youtubeSource({ id: 32, sourceSubtype: "playlist" })))
      .toBe(error.message);
    expect(detailErrorForYoutubeSource(error, youtubeSource({ id: 66 }))).toBeNull();
    expect(detailErrorForYoutubeSource(null, youtubeSource({ id: 32 }))).toBeNull();
  });

  it("builds compact content status lines without repeated prefixes", () => {
    const detail = videoDetail();

    expect(youtubeContentStatusLine("comments", detail.summary.comments, () => "2026-05-16")).toEqual({
      state: "synced",
      label: "Comments synced",
      countLabel: "43 comments",
      lastSyncedLabel: "Synced 2026-05-16",
    });
  });

  it("builds a provider header summary from typed YouTube detail", () => {
    const header = youtubeProviderHeaderSummary(youtubeSource(), videoDetail(), (value) => String(value));

    expect(header).toMatchObject({
      sourceKind: "video",
      title: "Gemma 4 Desktop Coder by Google",
      channelLabel: "@AIStackEngineer",
      durationLabel: "9:41",
      availabilityLabel: "available",
      captionsCountLabel: "239 segments",
      commentsCountLabel: "43 comments",
    });
  });

  it("marks corpus options with counts availability and audience warnings", () => {
    const options = youtubeCorpusOptionViews(videoDetail());

    expect(options.map((option) => option.value)).toEqual([
      "transcript_only",
      "transcript_description",
      "transcript_description_comments",
    ]);
    expect(options[0]).toMatchObject({ available: true, countLabel: "239 segments" });
    expect(options[1]).toMatchObject({ available: true, countLabel: "239 segments + description" });
    expect(options[2]).toMatchObject({
      available: true,
      countLabel: "239 segments + description + 43 comments",
      evidenceWarning: "Audience comments are user-generated evidence.",
    });
  });
});
```

- [x] **Step 2: Run helper tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/youtube-source-view-model.test.ts
```

Expected: FAIL because `src/lib/youtube-source-view-model.ts` does not exist.

- [x] **Step 3: Implement the helper module**

Create `src/lib/youtube-source-view-model.ts`:

```ts
import type { Source } from "$lib/types/sources";
import type {
  YoutubeContentStatus,
  YoutubePlaylistDetail,
  YoutubeVideoDetail,
} from "$lib/types/youtube";
import type { YoutubeCorpusMode } from "$lib/types/analysis";

export type YoutubeDetailErrorState = {
  sourceId: number;
  sourceSubtype: string | null;
  message: string;
} | null;

export type YoutubeProviderHeaderSummary = {
  sourceKind: "video" | "playlist";
  title: string;
  channelLabel: string;
  durationLabel: string | null;
  publishedLabel: string | null;
  canonicalUrl: string | null;
  thumbnailUrl: string | null;
  availabilityLabel: string;
  captionsLabel: string;
  captionsCountLabel: string;
  commentsLabel: string;
  commentsCountLabel: string;
};

export type YoutubeContentStatusLine = {
  state: YoutubeContentStatus["state"];
  label: string;
  countLabel: string;
  lastSyncedLabel: string | null;
};

export type YoutubeCorpusOptionView = {
  value: YoutubeCorpusMode;
  label: string;
  description: string;
  countLabel: string;
  available: boolean;
  disabledReason: string | null;
  evidenceWarning: string | null;
};

type YoutubeDetail = YoutubeVideoDetail | YoutubePlaylistDetail | null;

export function formatYoutubeDuration(value: number | null | undefined) {
  if (value === null || value === undefined) return null;
  const hours = Math.floor(value / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  const seconds = value % 60;
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

export function detailErrorForYoutubeSource(error: YoutubeDetailErrorState, source: Pick<Source, "id"> | null) {
  if (!error || !source || error.sourceId !== source.id) return null;
  return error.message;
}

export function youtubeContentStatusLine(
  kind: "captions" | "comments",
  status: YoutubeContentStatus,
  formatTimestamp: (value: number | null) => string,
): YoutubeContentStatusLine {
  const unit = kind === "captions"
    ? status.segmentCount === 1 ? "segment" : "segments"
    : status.itemCount === 1 ? "comment" : "comments";
  const count = kind === "captions" ? status.segmentCount : status.itemCount;
  return {
    state: status.state,
    label: status.label,
    countLabel: `${count} ${unit}`,
    lastSyncedLabel: status.lastSyncedAt === null ? null : `Synced ${formatTimestamp(status.lastSyncedAt)}`,
  };
}

export function youtubeProviderHeaderSummary(
  source: Pick<Source, "sourceSubtype" | "title" | "externalId">,
  detail: YoutubeDetail,
  formatTimestamp: (value: number | null) => string,
): YoutubeProviderHeaderSummary {
  const summary = detail?.summary ?? null;
  const title = summary?.title ?? source.title ?? source.externalId;
  const captions = summary?.captions ?? null;
  const comments = summary?.comments ?? null;
  return {
    sourceKind: source.sourceSubtype === "playlist" ? "playlist" : "video",
    title,
    channelLabel: summary?.channelHandle ?? summary?.channelTitle ?? "YouTube",
    durationLabel: formatYoutubeDuration(summary?.durationSeconds),
    publishedLabel: summary?.publishedAt === null || summary?.publishedAt === undefined
      ? null
      : formatTimestamp(summary.publishedAt),
    canonicalUrl: summary?.canonicalUrl ?? null,
    thumbnailUrl: summary?.thumbnailUrl ?? null,
    availabilityLabel: (summary?.availabilityStatus ?? "unknown").replaceAll("_", " "),
    captionsLabel: captions?.label ?? "Captions unknown",
    captionsCountLabel: captions ? youtubeContentStatusLine("captions", captions, formatTimestamp).countLabel : "0 segments",
    commentsLabel: comments?.label ?? "Comments unknown",
    commentsCountLabel: comments ? youtubeContentStatusLine("comments", comments, formatTimestamp).countLabel : "0 comments",
  };
}

export function youtubeCorpusOptionViews(detail: YoutubeVideoDetail | null): YoutubeCorpusOptionView[] {
  const captions = detail?.summary.captions ?? null;
  const description = detail?.sourceMetadata.description?.trim() ?? "";
  const comments = detail?.summary.comments ?? null;
  const transcriptAvailable = (captions?.segmentCount ?? 0) > 0;
  const descriptionAvailable = description.length > 0;
  const commentsAvailable = (comments?.itemCount ?? 0) > 0;
  const segmentLabel = `${captions?.segmentCount ?? 0} ${(captions?.segmentCount ?? 0) === 1 ? "segment" : "segments"}`;

  return [
    {
      value: "transcript_only",
      label: "Transcript",
      description: "Use only timestamp-backed video transcript evidence.",
      countLabel: segmentLabel,
      available: transcriptAvailable,
      disabledReason: transcriptAvailable ? null : "Transcript segments are not loaded.",
      evidenceWarning: null,
    },
    {
      value: "transcript_description",
      label: "Transcript + description",
      description: "Use transcript evidence plus author-provided description context.",
      countLabel: `${segmentLabel} + description`,
      available: transcriptAvailable && descriptionAvailable,
      disabledReason: !transcriptAvailable
        ? "Transcript segments are not loaded."
        : descriptionAvailable ? null : "Video description is not loaded.",
      evidenceWarning: null,
    },
    {
      value: "transcript_description_comments",
      label: "Transcript + description + comments",
      description: "Use transcript, description, and audience reactions.",
      countLabel: `${segmentLabel} + description + ${comments?.itemCount ?? 0} ${(comments?.itemCount ?? 0) === 1 ? "comment" : "comments"}`,
      available: transcriptAvailable && descriptionAvailable && commentsAvailable,
      disabledReason: !transcriptAvailable
        ? "Transcript segments are not loaded."
        : !descriptionAvailable
          ? "Video description is not loaded."
          : commentsAvailable ? null : "Comments are not loaded.",
      evidenceWarning: "Audience comments are user-generated evidence.",
    },
  ];
}
```

- [x] **Step 4: Run helper tests and verify they pass**

Run:

```powershell
npm.cmd run test -- src/lib/youtube-source-view-model.test.ts
```

Expected: PASS.

- [x] **Step 5: Write failing raw-source contract tests**

Create `src/lib/analysis-youtube-source-specialization.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import analysisStateSource from "./analysis-state.ts?raw";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import sourceBrowserShellSource from "./components/analysis/source-browser-shell.svelte?raw";
import sourceActivityViewSource from "./components/analysis/source-activity-view.svelte?raw";
import universalItemsViewSource from "./components/analysis/universal-items-view.svelte?raw";
import youtubeCommentsViewSource from "./components/analysis/youtube-comments-view.svelte?raw";
import youtubePlaylistVideosViewSource from "./components/analysis/youtube-playlist-videos-view.svelte?raw";
import youtubeTranscriptReaderSource from "./components/analysis/youtube-transcript-reader.svelte?raw";

describe("analysis youtube source specialization", () => {
  it("keeps youtube detail errors scoped to the selected source", () => {
    expect(analysisPageSource).toContain("youtubeDetailError");
    expect(analysisPageSource).toContain("YoutubeDetailErrorState");
    expect(analysisPageSource).toContain("youtubeDetailError = null");
    expect(analysisPageSource).toContain("youtubeDetailError = {");
    expect(analysisPageSource).toContain("sourceId: source.id");
    expect(analysisPageSource).toContain("[source.id]: detail.summary");
    expect(analysisPageSource).not.toContain('status = formatAppError("loading YouTube detail", error)');
  });

  it("uses the scoped youtube detail problem in report preflight copy", () => {
    expect(analysisStateSource).toContain("youtubeDetailProblemReason");
    expect(analysisStateSource).toContain("return state.youtubeDetailProblemReason");
    expect(analysisPageSource).toContain("youtubeDetailProblemReason: currentYoutubeDetailProblemReason()");
  });

  it("threads youtube detail error into report setup and source browser", () => {
    expect(analysisPageSource).toContain("{youtubeDetailError}");
    expect(reportCanvasSource).toContain("youtubeDetailError?: YoutubeDetailErrorState");
    expect(reportCanvasSource).toContain("{youtubeDetailError}");
    expect(reportSetupPanelSource).toContain("youtubeDetailError");
    expect(reportSourceSurfaceSource).toContain("youtubeDetailError");
    expect(sourceBrowserShellSource).toContain("youtubeDetailError");
  });

  it("promotes youtube corpus into a provider-specific report decision block", () => {
    expect(reportSetupPanelSource).toContain("youtubeCorpusOptionViews");
    expect(reportSetupPanelSource).toContain('class="youtube-corpus-panel"');
    expect(reportSetupPanelSource).toContain("Audience comments are user-generated evidence");
    expect(reportSetupPanelSource).not.toContain("<label>YouTube corpus");
  });

  it("renders invalid playlists as problem states instead of empty playlists", () => {
    expect(youtubePlaylistVideosViewSource).toContain("playlistDetailError");
    expect(youtubePlaylistVideosViewSource).toContain("Playlist metadata needs attention");
    expect(youtubePlaylistVideosViewSource).toContain("This is not an empty playlist.");
    expect(youtubePlaylistVideosViewSource).toContain("Retry playlist sync");
  });

  it("uses compact youtube status copy in transcript and comments readers", () => {
    expect(youtubeTranscriptReaderSource).toContain("youtubeProviderHeaderSummary");
    expect(youtubeTranscriptReaderSource).toContain("youtubeContentStatusLine");
    expect(youtubeTranscriptReaderSource).not.toContain("Comments {summary.comments.label}");
    expect(youtubeCommentsViewSource).toContain("Search comments");
    expect(youtubeCommentsViewSource).not.toContain("Search loaded comments");
  });

  it("renders youtube items as evidence inventory and activity as provider steps", () => {
    expect(universalItemsViewSource).toContain("Evidence inventory");
    expect(universalItemsViewSource).toContain("youtubeEvidenceRoleLabel");
    expect(sourceActivityViewSource).toContain("YouTube provider steps");
    expect(sourceActivityViewSource).toContain("Metadata");
    expect(sourceActivityViewSource).toContain("Transcript");
    expect(sourceActivityViewSource).toContain("Comments");
  });
});
```

- [x] **Step 6: Run raw-source contract tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-youtube-source-specialization.test.ts
```

Expected: FAIL because route props and component copy have not been updated.

- [x] **Step 7: Commit contracts**

Run:

```powershell
git add src/lib/youtube-source-view-model.ts src/lib/youtube-source-view-model.test.ts src/lib/analysis-youtube-source-specialization.test.ts
git commit -m "test: capture youtube source specialization contract"
```

---

### Task 2: Scope YouTube Detail Errors To The Selected Source

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/youtube-playlist-videos-view.svelte`
- Modify: `src/lib/analysis-state.ts`
- Test: `src/lib/analysis-youtube-source-specialization.test.ts`
- Test: `src/lib/analysis-report-canvas.test.ts`
- Test: `src/lib/analysis-state.test.ts`

- [x] **Step 1: Add route-level YouTube detail error state**

In `src/routes/analysis/+page.svelte`, import the type:

```ts
import type { YoutubeDetailErrorState } from "$lib/youtube-source-view-model";
```

Add state near the existing YouTube detail state:

```ts
let youtubeDetailError = $state<YoutubeDetailErrorState>(null);
```

- [x] **Step 2: Clear detail error during YouTube state reset**

In `resetYoutubeDetailState()`, add:

```ts
youtubeDetailError = null;
```

The function should include:

```ts
function resetYoutubeDetailState() {
  youtubeVideoDetail = null;
  youtubePlaylistDetail = null;
  youtubeDetailError = null;
  loadingYoutubeDetail = false;
  youtubeDetailRequestKey = "";
}
```

- [x] **Step 3: Replace global YouTube detail status with scoped error**

In `loadYoutubeDetail(source)`, clear the error at request start and set it in the catch block:

```ts
async function loadYoutubeDetail(source: Source) {
  const requestKey = `${source.id}:${source.sourceSubtype}`;
  youtubeDetailRequestKey = requestKey;
  youtubeDetailError = null;
  loadingYoutubeDetail = true;
  try {
    if (source.sourceSubtype === "playlist") {
      const detail = await getYoutubePlaylistDetail(source.id);
      if (youtubeDetailRequestKey !== requestKey) {
        return;
      }
      youtubePlaylistDetail = detail;
      youtubeVideoDetail = null;
      youtubeSummaries = {
        ...youtubeSummaries,
        [source.id]: detail.summary,
      };
      youtubeDetailError = null;
    } else {
      const detail = await getYoutubeVideoDetail(source.id);
      if (youtubeDetailRequestKey !== requestKey) {
        return;
      }
      youtubeVideoDetail = detail;
      youtubePlaylistDetail = null;
      youtubeSummaries = {
        ...youtubeSummaries,
        [source.id]: detail.summary,
      };
      youtubeDetailError = null;
    }
  } catch (error) {
    if (youtubeDetailRequestKey !== requestKey) {
      return;
    }
    youtubeVideoDetail = null;
    youtubePlaylistDetail = null;
    youtubeDetailError = {
      sourceId: source.id,
      sourceSubtype: source.sourceSubtype,
      message: formatAppError("loading YouTube detail", error),
    };
  } finally {
    if (youtubeDetailRequestKey === requestKey) {
      loadingYoutubeDetail = false;
    }
  }
}
```

The `youtubeSummaries` update is required so the source switcher and opened source detail stop showing contradictory YouTube status after a fresh detail load.

- [x] **Step 4: Add the scoped YouTube detail problem to report preflight**

In `src/lib/analysis-state.ts`, extend `ReportLaunchPreflightState`:

```ts
youtubeDetailProblemReason?: string | null;
```

In `reportLaunchDisabledReason(state)`, return this reason for the selected YouTube source before the generic "sync this source" check:

```ts
if (state.currentSource.sourceType === "youtube" && state.youtubeDetailProblemReason) {
  return state.youtubeDetailProblemReason;
}
```

In `src/routes/analysis/+page.svelte`, import `detailErrorForYoutubeSource` and add a helper near `currentReportLaunchState()`:

```ts
function currentYoutubeDetailProblemReason() {
  const source = currentSource();
  if (source?.sourceType !== "youtube") {
    return null;
  }
  return detailErrorForYoutubeSource(youtubeDetailError, source);
}
```

Add the helper output to `currentReportLaunchState()`:

```ts
youtubeDetailProblemReason: currentYoutubeDetailProblemReason(),
```

In `src/lib/analysis-state.test.ts`, add a preflight test that a selected YouTube source with `youtubeDetailProblemReason` returns that exact message before the generic sync-disabled reason.

- [x] **Step 5: Pass `youtubeDetailError` through canvas props**

In `src/lib/components/analysis/report-canvas.svelte`, import:

```ts
import type { YoutubeDetailErrorState } from "$lib/youtube-source-view-model";
```

Add prop:

```ts
youtubeDetailError?: YoutubeDetailErrorState;
```

Default it:

```ts
youtubeDetailError = null,
```

Pass it to `<ReportSetupPanel />` and `<ReportSourceSurface />`:

```svelte
{youtubeDetailError}
```

- [x] **Step 6: Pass `youtubeDetailError` through source surface and shell**

In `report-source-surface.svelte`, import the type, add prop, default it to `null`, and include it in `sourceBrowserData={{ ... }}`:

```svelte
youtubeDetailError,
```

In `source-browser-shell.svelte`, add to `SourceBrowserData`:

```ts
youtubeDetailError: YoutubeDetailErrorState;
```

Import the type from `$lib/youtube-source-view-model`.

- [x] **Step 7: Render playlist detail errors in the playlist videos view**

In `source-browser-shell.svelte`, pass:

```svelte
playlistDetailError={sourceData.youtubeDetailError?.sourceId === sourceSubject.id ? sourceData.youtubeDetailError.message : null}
```

to `<YoutubePlaylistVideosView />`.

In `youtube-playlist-videos-view.svelte`, add prop:

```ts
playlistDetailError?: string | null;
```

Default it:

```ts
playlistDetailError = null,
```

Render the problem state before the normal `!playlist || !summary` branch:

```svelte
{:else if playlistDetailError}
  <StatusMessage tone="error" surface={false}>
    <strong>Playlist metadata needs attention</strong>
    <span>This is not an empty playlist. {playlistDetailError}</span>
  </StatusMessage>
  <div class="playlist-actions">
    <Button size="sm" variant="secondary" onclick={onSyncPlaylist}>
      <RefreshCw size={14} aria-hidden="true" /> Retry playlist sync
    </Button>
  </div>
```

- [x] **Step 8: Run focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-state.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS for the scoped error and prop-threading assertions.

- [x] **Step 9: Commit**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-state.ts src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/youtube-playlist-videos-view.svelte src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-state.test.ts src/lib/analysis-source-readers.test.ts
git commit -m "fix(analysis): scope youtube detail errors"
```

---

### Task 3: Promote YouTube Corpus In Report Setup

**Files:**
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-setup-panel.svelte`
- Test: `src/lib/analysis-youtube-source-specialization.test.ts`
- Test: `src/lib/analysis-report-setup-props.test.ts`

- [x] **Step 1: Pass existing YouTube detail props into report setup**

`report-canvas.svelte` already receives `youtubeVideoDetail` and `youtubePlaylistDetail`. If Task 2 has not already done it, also add:

```ts
youtubeDetailError?: YoutubeDetailErrorState;
```

Ensure `<ReportSetupPanel />` receives the existing detail props and the new error prop:

```svelte
{youtubeVideoDetail}
{youtubePlaylistDetail}
{youtubeDetailError}
```

- [x] **Step 2: Import corpus helpers in setup panel**

In `report-setup-panel.svelte`, import:

```ts
import {
  detailErrorForYoutubeSource,
  youtubeCorpusOptionViews,
} from "$lib/youtube-source-view-model";
import type { YoutubeDetailErrorState } from "$lib/youtube-source-view-model";
import type { YoutubePlaylistDetail, YoutubeVideoDetail } from "$lib/types/youtube";
```

Add props:

```ts
youtubeVideoDetail?: YoutubeVideoDetail | null;
youtubePlaylistDetail?: YoutubePlaylistDetail | null;
youtubeDetailError?: YoutubeDetailErrorState;
```

Default them to `null`.

- [x] **Step 3: Derive setup state**

Add derived values:

```ts
const isYoutubeVideoScope = $derived(
  analysisScope === "single_source" &&
    currentSource?.sourceType === "youtube" &&
    currentSource.sourceSubtype === "video",
);
const isYoutubePlaylistScope = $derived(
  analysisScope === "single_source" &&
    currentSource?.sourceType === "youtube" &&
    currentSource.sourceSubtype === "playlist",
);
const isYoutubeGroupScope = $derived(
  analysisScope === "source_group" && currentGroup?.source_type === "youtube",
);
const selectedYoutubeError = $derived(
  currentSource?.sourceType === "youtube"
    ? detailErrorForYoutubeSource(youtubeDetailError, currentSource)
    : null,
);
const youtubeCorpusOptions = $derived(
  isYoutubeVideoScope ? youtubeCorpusOptionViews(youtubeVideoDetail ?? null) : [],
);
const youtubePlaylistLinkedCountLabel = $derived(
  youtubePlaylistDetail?.summary.linkedVideoCount !== null &&
    youtubePlaylistDetail?.summary.linkedVideoCount !== undefined
    ? `${youtubePlaylistDetail.summary.linkedVideoCount} linked videos`
    : "Playlist linked-video evidence",
);
```

- [x] **Step 4: Replace the plain `YouTube corpus` select with option cards**

Replace the current `label>YouTube corpus` block with a provider section. Render counted availability cards for a single YouTube video; render a compact fallback control for YouTube playlists and YouTube source groups so the UI never shows an empty option grid.

```svelte
{#if isYoutubeScope}
  <section class="youtube-corpus-panel" aria-label="YouTube corpus">
    <div class="youtube-corpus-heading">
      <div>
        <span class="eyebrow">Primary YouTube decision</span>
        <h3>YouTube corpus</h3>
        <p>Choose which evidence the report can use.</p>
      </div>
      {#if selectedYoutubeError}
        <Badge variant="danger">source problem</Badge>
      {/if}
    </div>

    {#if selectedYoutubeError}
      <StatusMessage tone="error">{selectedYoutubeError}</StatusMessage>
    {/if}

    {#if isYoutubeVideoScope}
      <div class="youtube-corpus-options">
        {#each youtubeCorpusOptions as option (option.value)}
          <button
            type="button"
            class:selected={youtubeCorpusMode === option.value}
            class="youtube-corpus-option"
            disabled={!option.available || !!selectedYoutubeError}
            title={option.disabledReason ?? undefined}
            onclick={() => option.available && !selectedYoutubeError && onChangeYoutubeCorpusMode(option.value)}
          >
            <strong>{option.label}</strong>
            <span>{option.description}</span>
            <small>{option.countLabel}</small>
            {#if option.evidenceWarning}
              <em>{option.evidenceWarning}</em>
            {/if}
          </button>
        {/each}
      </div>
    {:else}
      <div class="youtube-corpus-options compact">
        {#each [
          ["transcript_only", "Transcript"],
          ["transcript_description", "Transcript + description"],
          ["transcript_description_comments", "Transcript + description + comments"],
        ] as [value, label] (value)}
          <button
            type="button"
            class:selected={youtubeCorpusMode === value}
            class="youtube-corpus-option"
            disabled={!!selectedYoutubeError}
            onclick={() => !selectedYoutubeError && onChangeYoutubeCorpusMode(value as YoutubeCorpusMode)}
          >
            <strong>{label}</strong>
            <span>
              {isYoutubePlaylistScope
                ? "Applies to synced linked videos in this playlist."
                : "Applies to each synced YouTube source in this group."}
            </span>
            <small>{isYoutubeGroupScope ? "Group-level source counts" : youtubePlaylistLinkedCountLabel}</small>
            {#if value === "transcript_description_comments"}
              <em>Audience comments are user-generated evidence.</em>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </section>
{/if}
```

- [x] **Step 5: Add scoped CSS**

In `report-setup-panel.svelte`, add:

```css
.youtube-corpus-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.85rem;
  border: 1px solid color-mix(in srgb, var(--primary) 24%, var(--border));
  border-radius: 8px;
  background: color-mix(in srgb, var(--primary) 6%, var(--panel));
}

.youtube-corpus-heading,
.youtube-corpus-options {
  display: flex;
  gap: 0.65rem;
  align-items: flex-start;
}

.youtube-corpus-heading {
  justify-content: space-between;
}

.youtube-corpus-options {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.youtube-corpus-options.compact {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.youtube-corpus-option {
  display: flex;
  min-height: 8rem;
  align-items: flex-start;
  justify-content: flex-start;
  text-align: left;
  flex-direction: column;
  white-space: normal;
  gap: 0.35rem;
  padding: 0.75rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel);
  color: inherit;
  cursor: pointer;
}

.youtube-corpus-option.selected {
  border-color: var(--primary);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 18%, transparent);
}

.youtube-corpus-option:disabled {
  cursor: not-allowed;
  opacity: 0.62;
}

.youtube-corpus-option span,
.youtube-corpus-option small,
.youtube-corpus-option em {
  color: var(--muted);
  font-size: 0.78rem;
  line-height: 1.35;
}

.youtube-corpus-option em {
  font-style: normal;
  color: #a15c00;
}
```

In the existing `@media (max-width: 1100px)` block, add:

```css
.youtube-corpus-options {
  grid-template-columns: 1fr;
}
```

- [x] **Step 6: Run focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-report-setup-props.test.ts src/lib/analysis-report-canvas.test.ts
```

Expected: PASS.

- [x] **Step 7: Commit**

Run:

```powershell
git add src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-setup-panel.svelte src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-report-setup-props.test.ts src/lib/analysis-report-canvas.test.ts
git commit -m "feat(analysis): promote youtube report corpus"
```

---

### Task 4: Compact YouTube Video Readers

**Files:**
- Modify: `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Modify: `src/lib/components/analysis/youtube-comments-view.svelte`
- Test: `src/lib/analysis-youtube-source-specialization.test.ts`
- Test: `src/lib/analysis-source-readers.test.ts`

- [x] **Step 1: Import header/status helpers in transcript reader**

In `youtube-transcript-reader.svelte`, import:

```ts
import {
  youtubeContentStatusLine,
  youtubeProviderHeaderSummary,
} from "$lib/youtube-source-view-model";
```

Add derived values:

```ts
const providerHeader = $derived(
  youtubeProviderHeaderSummary(
    { sourceSubtype: "video", title: sourceTitle, externalId: sourceTitle },
    detail,
    formatTimestamp,
  ),
);
const captionsStatus = $derived(summary ? youtubeContentStatusLine("captions", summary.captions, formatTimestamp) : null);
const commentsStatus = $derived(summary ? youtubeContentStatusLine("comments", summary.comments, formatTimestamp) : null);
```

- [x] **Step 2: Replace duplicated status badges**

In the transcript header meta, replace the direct `summary.captions.*` and `summary.comments.*` badges with:

```svelte
{#if captionsStatus}
  <Badge variant={captionsStatus.state === "synced" ? "success" : captionsStatus.state === "unavailable" ? "warning" : "neutral"}>
    {captionsStatus.label}
  </Badge>
  <Badge variant="neutral">{captionsStatus.countLabel}</Badge>
  {#if captionsStatus.lastSyncedLabel}<Badge variant="neutral">{captionsStatus.lastSyncedLabel}</Badge>{/if}
{/if}
{#if commentsStatus}
  <Badge variant={commentsStatus.state === "synced" ? "success" : commentsStatus.state === "failed" ? "danger" : "neutral"}>
    {commentsStatus.label}
  </Badge>
  <Badge variant="neutral">{commentsStatus.countLabel}</Badge>
  {#if commentsStatus.lastSyncedLabel}<Badge variant="neutral">{commentsStatus.lastSyncedLabel}</Badge>{/if}
{/if}
```

Keep the `h3`, but change it to:

```svelte
<h3>{providerHeader.title}</h3>
```

- [x] **Step 3: Rename comments search copy**

In `youtube-comments-view.svelte`, change label, placeholder, and aria label from `Search loaded comments` to `Search comments`:

```svelte
<span>Search comments</span>
placeholder="Search comments"
ariaLabel="Search comments"
```

- [x] **Step 4: Add audience evidence note and long-thread affordance**

In `youtube-comments-view.svelte`, after the comments toolbar, add:

```svelte
<p class="comments-evidence-note">Audience comments are user-generated evidence and should be cited separately from transcript claims.</p>
```

Add CSS:

```css
.comments-evidence-note {
  margin: -0.25rem 0 0;
  color: var(--muted);
  font-size: 0.78rem;
}

.reply-list {
  max-height: 22rem;
  overflow: auto;
}
```

- [x] **Step 5: Run focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [x] **Step 6: Commit**

Run:

```powershell
git add src/lib/components/analysis/youtube-transcript-reader.svelte src/lib/components/analysis/youtube-comments-view.svelte src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-source-readers.test.ts
git commit -m "feat(analysis): compact youtube readers"
```

---

### Task 5: Make YouTube Items Read As Evidence Inventory

**Files:**
- Modify: `src/lib/source-browser-model.ts`
- Modify: `src/lib/source-browser-model.test.ts`
- Modify: `src/lib/components/analysis/universal-items-view.svelte`
- Test: `src/lib/analysis-youtube-source-specialization.test.ts`

- [x] **Step 1: Add evidence role helper tests**

In `src/lib/source-browser-model.test.ts`, extend the import list with:

```ts
youtubeEvidenceRoleLabel,
youtubeEvidenceContextLine,
```

Add tests:

```ts
it("labels YouTube item evidence roles for inventory views", () => {
  expect(youtubeEvidenceRoleLabel(sourceItem({ itemKind: "youtube_transcript" }))).toBe("Transcript");
  expect(youtubeEvidenceRoleLabel(youtubeCommentItem())).toBe("Comment");
  expect(youtubeEvidenceRoleLabel(sourceItem({ itemKind: "youtube_description" }))).toBe("Description");
});

it("keeps raw YouTube identifiers secondary in evidence context lines", () => {
  expect(youtubeEvidenceContextLine(
    youtubeCommentItem({ author: "@demo", externalId: "comment:abc" }),
    "Source #66",
  )).toBe("@demo - Source #66");
});
```

- [x] **Step 2: Run tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/source-browser-model.test.ts
```

Expected: FAIL because helper functions are missing.

- [x] **Step 3: Implement evidence helper functions**

In `src/lib/source-browser-model.ts`, add exports:

```ts
export function youtubeEvidenceRoleLabel(item: Pick<SourceItem, "itemKind" | "youtubeComment">): string {
  if (item.youtubeComment || item.itemKind === "youtube_comment") {
    return item.youtubeComment?.isReply ? "Reply" : "Comment";
  }
  if (item.itemKind === "youtube_transcript") return "Transcript";
  if (item.itemKind === "youtube_description") return "Description";
  return sourceItemKindLabel(item.itemKind);
}

export function youtubeEvidenceContextLine(
  item: Pick<SourceItem, "author" | "hasMedia" | "mediaKind">,
  sourceLabel: string,
): string {
  return [
    item.author,
    sourceLabel,
    item.hasMedia ? item.mediaKind ?? "media" : null,
  ].filter(Boolean).join(" - ");
}
```

- [x] **Step 4: Use evidence labels in UniversalItemsView for YouTube rows**

In `universal-items-view.svelte`, import:

```ts
youtubeEvidenceContextLine,
youtubeEvidenceRoleLabel,
```

Where item title currently uses `sourceItemKindLabel` or `item.itemKind`, derive:

```svelte
{@const isYoutubeItem = item.itemKind.startsWith("youtube_") || !!item.youtubeComment}
<strong>{isYoutubeItem ? youtubeEvidenceRoleLabel(item) : sourceItemKindLabel(item.itemKind)}</strong>
```

Where the context line is rendered, use:

```svelte
{isYoutubeItem
  ? youtubeEvidenceContextLine(item, sourceLabelForItem?.(item) ?? `Source #${item.sourceId}`)
  : sourceItemContextLine(item, sourceLabelForItem?.(item) ?? `Source #${item.sourceId}`)}
```

Add a visible section heading for YouTube rows:

```svelte
{#if items.some((item) => item.itemKind.startsWith("youtube_") || item.youtubeComment)}
  <div class="evidence-inventory-label">Evidence inventory</div>
{/if}
```

- [x] **Step 5: Add small CSS for evidence label**

In `universal-items-view.svelte`, add:

```css
.evidence-inventory-label {
  color: var(--muted);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}
```

- [x] **Step 6: Run focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/source-browser-model.test.ts src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [x] **Step 7: Commit**

Run:

```powershell
git add src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts src/lib/components/analysis/universal-items-view.svelte src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-source-readers.test.ts
git commit -m "feat(analysis): present youtube items as evidence"
```

---

### Task 6: Add YouTube Provider-Step Activity

**Files:**
- Modify: `src/lib/components/analysis/source-activity-view.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Test: `src/lib/analysis-youtube-source-specialization.test.ts`
- Test: `src/lib/analysis-source-readers.test.ts`

- [x] **Step 1: Pass detail into SourceActivityView**

In `source-browser-shell.svelte`, pass the current YouTube details into `<SourceActivityView />`:

```svelte
youtubeVideoDetail={sourceData.youtubeVideoDetail}
youtubePlaylistDetail={sourceData.youtubePlaylistDetail}
youtubeDetailError={sourceData.youtubeDetailError}
```

In `source-activity-view.svelte`, import the types and add props:

```ts
import type { YoutubeDetailErrorState } from "$lib/youtube-source-view-model";
import type { YoutubePlaylistDetail, YoutubeVideoDetail } from "$lib/types/youtube";

youtubeVideoDetail?: YoutubeVideoDetail | null;
youtubePlaylistDetail?: YoutubePlaylistDetail | null;
youtubeDetailError?: YoutubeDetailErrorState;
```

Default them to `null`.

- [x] **Step 2: Add YouTube provider steps section**

In `source-activity-view.svelte`, before the detailed jobs section, add:

```svelte
{#if source.sourceType === "youtube"}
  {@const youtubeSummary = youtubeVideoDetail?.summary ?? youtubePlaylistDetail?.summary ?? null}
  <section class="activity-section" aria-label="YouTube provider steps">
    <div class="section-heading">
      <span class="eyebrow">YouTube provider steps</span>
      <Badge variant={youtubeDetailError?.sourceId === source.id ? "danger" : "neutral"}>
        {youtubeDetailError?.sourceId === source.id ? "attention" : "current source"}
      </Badge>
    </div>
    {#if youtubeDetailError?.sourceId === source.id}
      <StatusMessage tone="error">{youtubeDetailError.message}</StatusMessage>
    {/if}
    <div class="provider-step-grid">
      <div class="provider-step">
        <strong>Metadata</strong>
        <span>{youtubeSummary ? "Detail loaded" : "Detail not loaded"}</span>
      </div>
      <div class="provider-step">
        <strong>Transcript</strong>
        <span>{youtubeSummary ? `${youtubeSummary.captions.label} - ${youtubeSummary.captions.segmentCount} segments` : "Unknown"}</span>
      </div>
      <div class="provider-step">
        <strong>Comments</strong>
        <span>{youtubeSummary ? `${youtubeSummary.comments.label} - ${youtubeSummary.comments.itemCount} comments` : "Unknown"}</span>
      </div>
    </div>
  </section>
{/if}
```

- [x] **Step 3: Add provider step CSS**

In `source-activity-view.svelte`, add:

```css
.provider-step-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.55rem;
}

.provider-step {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  padding: 0.65rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel);
}

.provider-step span {
  color: var(--muted);
  font-size: 0.78rem;
  overflow-wrap: anywhere;
}
```

Add to the existing mobile block:

```css
.provider-step-grid {
  grid-template-columns: 1fr;
}
```

- [x] **Step 4: Run focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit**

Run:

```powershell
git add src/lib/components/analysis/source-activity-view.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/analysis-youtube-source-specialization.test.ts src/lib/analysis-source-readers.test.ts
git commit -m "feat(analysis): explain youtube source activity"
```

---

### Task 7: Final Verification

**Files:**
- Modify only files already touched by Tasks 1-6 if verification reveals a regression.

- [x] **Step 1: Run focused YouTube tests**

Run:

```powershell
npm.cmd run test -- src/lib/youtube-source-view-model.test.ts src/lib/analysis-youtube-source-specialization.test.ts src/lib/source-browser-model.test.ts src/lib/analysis-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-setup-props.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [x] **Step 2: Run full frontend tests**

Run:

```powershell
npm.cmd run test
```

Expected: all Vitest suites pass.

- [x] **Step 3: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 4: Run Analysis smoke**

Run:

```powershell
npm.cmd run smoke:analysis
```

Expected: Analysis smoke passes with exit code `0`.

- [x] **Step 5: Inspect the running app**

Use the running Tauri app and verify:

- Open an invalid YouTube playlist. The source shows a problem-first state and `Run report` uses the same root cause.
- Switch from that playlist to a normal YouTube video. The playlist error disappears.
- Open a YouTube video with transcript and comments. The transcript status does not show duplicated words.
- Open Report mode for a YouTube video. Corpus choices appear as provider-specific options with counts.
- Open Items. YouTube rows read as evidence inventory.
- Open Activity. Metadata, Transcript, Comments, and detail error state are visible before recent jobs.

- [x] **Step 6: Commit verification fixes when files changed**

When verification requires changes in files already touched by this plan, run:

```powershell
git add src
git commit -m "fix: polish youtube source specialization"
```

When verification produces no file changes, do not create an empty commit.

---

## Self-Review

- Spec coverage: every acceptance criterion maps to Tasks 1-7.
- Open-marker scan: no unresolved implementation markers are left in the plan.
- Type consistency: `YoutubeDetailErrorState`, `YoutubeProviderHeaderSummary`, and `YoutubeCorpusOptionView` are defined in Task 1 and referenced consistently in subsequent tasks.
- Scope check: backend ingestion and database work remain out of scope.
