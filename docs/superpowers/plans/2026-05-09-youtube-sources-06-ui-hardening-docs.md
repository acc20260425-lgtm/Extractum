# YouTube Sources Part 6: UI Polish, Hardening, and Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the YouTube MVP user experience, run the manual hardening matrix with pass/fail criteria, and document the new provider.

**Architecture:** Add read-only YouTube runtime/detail APIs on top of the storage created in Parts 1-5, then wire provider-aware UI components to those DTOs. This part must not add new persistence concepts: captions/comments status comes from `items`, `youtube_transcript_segments`, `youtube_playlist_items`, source metadata, and in-memory source jobs.

**Tech Stack:** Svelte 5, Tauri 2, Vitest, Rust test suite, sqlx SQLite, `yt-dlp`.

---

## Consistent End State

After this part:

- YouTube source rows are informative and do not use Telegram-only terminology.
- Video detail shows overview, transcript status, comments status, and jobs.
- Playlist detail shows ordered membership rows and maps row actions to Part 3 commands.
- Analysis controls expose YouTube corpus modes and hide Telegram topic controls for YouTube.
- Restart behavior for in-memory YouTube jobs is explicit and manually verified.
- README and architecture/schema docs describe the MVP and distinguish existing NotebookLM export from future YouTube-specific enhancements.
- Full Rust and frontend checks, including clippy, pass before the MVP is considered complete.

---

## Task 1: Source Cards, Runtime Status, and Workspace Detail

**Files:**

- Create: `src-tauri/src/youtube/detail.rs`
- Create: `src-tauri/src/youtube/runtime.rs`
- Modify: `src-tauri/src/youtube/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/types/youtube.ts`
- Create: `src/lib/api/youtube-detail.ts`
- Create: `src/lib/api/youtube-detail.test.ts`
- Modify: `src/lib/components/source-row.svelte`
- Create: `src/lib/components/analysis/youtube-source-detail.svelte`
- Create: `src/lib/components/analysis/youtube-playlist-detail.svelte`
- Modify: `src/lib/components/analysis/workspace-main.svelte`
- Modify: `src/lib/components/analysis/workspace-rail.svelte`
- Modify: `src/lib/components/analysis/source-context-panel.svelte`
- Modify: `src/lib/analysis-source-state.ts`
- Modify: `src/lib/analysis-source-state.test.ts`
- Modify: `src/lib/analysis-scope-state.ts`
- Modify: `src/lib/analysis-scope-state.test.ts`

- [ ] Execute this task in two internal checkpoints so runtime/detail contracts stabilize before the Svelte wiring lands:

```text
Checkpoint A: runtime status command, detail DTOs, backend detail queries, frontend API wrappers, and their tests.
Checkpoint B: source row/workspace/detail components, provider-aware labels, and analysis state wiring.
```

- [ ] Add a runtime status command so sync disable reasons can mention missing `yt-dlp` before the user starts a job.

In `src-tauri/src/youtube/runtime.rs`:

```rust
use std::time::Duration;

use serde::Serialize;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

const YTDLP_RUNTIME_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeRuntimeStatusDto {
    pub ytdlp_available: bool,
    pub ytdlp_version: Option<String>,
    pub message: String,
}

#[tauri::command]
pub async fn get_youtube_runtime_status() -> AppResult<YoutubeRuntimeStatusDto> {
    let output = tokio::time::timeout(
        YTDLP_RUNTIME_CHECK_TIMEOUT,
        Command::new("yt-dlp").arg("--version").output(),
    )
    .await;

    match output {
        Ok(Ok(output)) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(YoutubeRuntimeStatusDto {
                ytdlp_available: true,
                ytdlp_version: if version.is_empty() { None } else { Some(version) },
                message: "yt-dlp is available".to_string(),
            })
        }
        Ok(Ok(output)) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(YoutubeRuntimeStatusDto {
                ytdlp_available: false,
                ytdlp_version: None,
                message: if stderr.is_empty() {
                    "yt-dlp is not available on PATH".to_string()
                } else {
                    format!("yt-dlp check failed: {stderr}")
                },
            })
        }
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(YoutubeRuntimeStatusDto {
                ytdlp_available: false,
                ytdlp_version: None,
                message: "yt-dlp is not available on PATH".to_string(),
            })
        }
        Ok(Err(error)) => Err(AppError::internal(format!("yt-dlp check failed: {error}"))),
        Err(_) => Ok(YoutubeRuntimeStatusDto {
            ytdlp_available: false,
            ytdlp_version: None,
            message: "yt-dlp runtime check timed out".to_string(),
        }),
    }
}
```

Register `runtime` in `youtube/mod.rs` and register `get_youtube_runtime_status` in `src-tauri/src/lib.rs`.

- [ ] Add read-only detail DTOs in `src-tauri/src/youtube/detail.rs` and matching TypeScript types in `src/lib/types/youtube.ts`.

Rust DTO shape:

```rust
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YoutubeContentSyncState {
    NotSynced,
    Synced,
    Unavailable,
    Failed,
    Unknown,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeContentStatusDto {
    pub state: YoutubeContentSyncState,
    pub item_count: i64,
    pub segment_count: i64,
    pub last_synced_at: Option<i64>,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSourceSummaryDto {
    pub source_id: i64,
    pub source_subtype: String,
    pub title: Option<String>,
    pub channel_title: Option<String>,
    pub channel_handle: Option<String>,
    pub canonical_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i64>,
    pub published_at: Option<i64>,
    pub availability_status: Option<String>,
    pub video_count: Option<i64>,
    pub linked_video_count: Option<i64>,
    pub unavailable_count: Option<i64>,
    pub captions: YoutubeContentStatusDto,
    pub comments: YoutubeContentStatusDto,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePlaylistMembershipDto {
    pub playlist_source_id: i64,
    pub playlist_title: Option<String>,
    pub position: Option<i64>,
    pub availability_status: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeVideoDetailDto {
    pub summary: YoutubeSourceSummaryDto,
    pub playlist_memberships: Vec<YoutubePlaylistMembershipDto>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePlaylistItemDetailDto {
    pub position: Option<i64>,
    pub video_id: String,
    pub video_source_id: Option<i64>,
    pub title: Option<String>,
    pub canonical_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i64>,
    pub published_at: Option<i64>,
    pub availability_status: String,
    pub is_removed_from_playlist: bool,
    pub captions: YoutubeContentStatusDto,
    pub comments: YoutubeContentStatusDto,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePlaylistDetailDto {
    pub summary: YoutubeSourceSummaryDto,
    pub items: Vec<YoutubePlaylistItemDetailDto>,
}
```

TypeScript DTO shape:

```ts
export type YoutubeContentSyncState =
  | "not_synced"
  | "synced"
  | "unavailable"
  | "failed"
  | "unknown";

export interface YoutubeRuntimeStatus {
  ytdlpAvailable: boolean;
  ytdlpVersion: string | null;
  message: string;
}

export interface YoutubeContentStatus {
  state: YoutubeContentSyncState;
  itemCount: number;
  segmentCount: number;
  lastSyncedAt: number | null;
  label: string;
}

export interface YoutubeSourceSummary {
  sourceId: number;
  sourceSubtype: string;
  title: string | null;
  channelTitle: string | null;
  channelHandle: string | null;
  canonicalUrl: string | null;
  thumbnailUrl: string | null;
  durationSeconds: number | null;
  publishedAt: number | null;
  availabilityStatus: string | null;
  videoCount: number | null;
  linkedVideoCount: number | null;
  unavailableCount: number | null;
  captions: YoutubeContentStatus;
  comments: YoutubeContentStatus;
}

export interface YoutubePlaylistMembership {
  playlistSourceId: number;
  playlistTitle: string | null;
  position: number | null;
  availabilityStatus: string;
}

export interface YoutubeVideoDetail {
  summary: YoutubeSourceSummary;
  playlistMemberships: YoutubePlaylistMembership[];
}

export interface YoutubePlaylistItemDetail {
  position: number | null;
  videoId: string;
  videoSourceId: number | null;
  title: string | null;
  canonicalUrl: string | null;
  thumbnailUrl: string | null;
  durationSeconds: number | null;
  publishedAt: number | null;
  availabilityStatus: string;
  isRemovedFromPlaylist: boolean;
  captions: YoutubeContentStatus;
  comments: YoutubeContentStatus;
}

export interface YoutubePlaylistDetail {
  summary: YoutubeSourceSummary;
  items: YoutubePlaylistItemDetail[];
}
```

- [ ] Implement detail commands in `src-tauri/src/youtube/detail.rs`:

```rust
#[tauri::command]
pub async fn list_youtube_source_summaries(
    handle: AppHandle,
    source_ids: Vec<i64>,
) -> AppResult<Vec<YoutubeSourceSummaryDto>>;

#[tauri::command]
pub async fn get_youtube_video_detail(
    handle: AppHandle,
    source_id: i64,
) -> AppResult<YoutubeVideoDetailDto>;

#[tauri::command]
pub async fn get_youtube_playlist_detail(
    handle: AppHandle,
    source_id: i64,
) -> AppResult<YoutubePlaylistDetailDto>;
```

Data source policy:

- Decode source `metadata_zstd` through the YouTube metadata decoder from Part 2, never through Telegram metadata decoding.
- If YouTube metadata is missing, fall back to `sources.title`, `sources.external_id`, and `sources.avatar_data_url` where available.
- Captions status for a video comes from `items` rows where `items.source_id = ? AND items.item_kind = 'youtube_transcript'`.
- Transcript segment count comes from `youtube_transcript_segments` joined by transcript `item_id`.
- Comments status for a video comes from `items` rows where `items.source_id = ? AND items.item_kind = 'youtube_comment'`.
- Playlist rows come from `youtube_playlist_items` ordered by `position IS NULL, position, video_id`.
- Playlist linked count is `COUNT(*) WHERE video_source_id IS NOT NULL AND is_removed_from_playlist = 0`.
- Playlist unavailable count is `COUNT(*) WHERE availability_status NOT IN ('available', 'live_now', 'live_ended_transcript_pending') OR is_removed_from_playlist = 1`.
- Last transcript sync uses the maximum `items.ingested_at` among transcript items for the source.
- Last comments sync uses the maximum `items.ingested_at` among comment items for the source.

Use this status helper policy:

```text
transcript item count > 0 and segment count > 0 -> synced
transcript item count > 0 and segment count = 0 -> synced
known no-captions or unavailable availability -> unavailable
otherwise -> not_synced

comment item count > 0 -> synced
comments sync job failed most recently -> failed
otherwise -> not_synced
```

The MVP may ignore failed job state in SQL because source jobs are in memory; the frontend can display the latest failed `SourceJobRecord` beside the persisted status.

- [ ] Add frontend API wrappers in `src/lib/api/youtube-detail.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type {
  YoutubePlaylistDetail,
  YoutubeRuntimeStatus,
  YoutubeSourceSummary,
  YoutubeVideoDetail,
} from "$lib/types/youtube";

export function getYoutubeRuntimeStatus() {
  return invoke<YoutubeRuntimeStatus>("get_youtube_runtime_status");
}

export function listYoutubeSourceSummaries(sourceIds: number[]) {
  return invoke<YoutubeSourceSummary[]>("list_youtube_source_summaries", { sourceIds });
}

export function getYoutubeVideoDetail(sourceId: number) {
  return invoke<YoutubeVideoDetail>("get_youtube_video_detail", { sourceId });
}

export function getYoutubePlaylistDetail(sourceId: number) {
  return invoke<YoutubePlaylistDetail>("get_youtube_playlist_detail", { sourceId });
}
```

- [ ] Add explicit API tests in `src/lib/api/youtube-detail.test.ts`:

```ts
it("checks youtube runtime status", async () => {
  invokeMock.mockResolvedValueOnce({
    ytdlpAvailable: false,
    ytdlpVersion: null,
    message: "yt-dlp is not available on PATH",
  });

  await expect(getYoutubeRuntimeStatus()).resolves.toMatchObject({
    ytdlpAvailable: false,
  });
  expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_runtime_status");
});

it("lists youtube summaries with source ids", async () => {
  invokeMock.mockResolvedValueOnce([]);
  await listYoutubeSourceSummaries([10, 11]);
  expect(invokeMock).toHaveBeenLastCalledWith("list_youtube_source_summaries", {
    sourceIds: [10, 11],
  });
});

it("loads youtube video and playlist detail", async () => {
  invokeMock.mockResolvedValueOnce({ summary: { sourceId: 10 }, playlistMemberships: [] });
  await getYoutubeVideoDetail(10);
  expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_video_detail", { sourceId: 10 });

  invokeMock.mockResolvedValueOnce({ summary: { sourceId: 20 }, items: [] });
  await getYoutubePlaylistDetail(20);
  expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_playlist_detail", { sourceId: 20 });
});
```

- [ ] Update `src/lib/analysis-source-state.ts` so `sourceSyncDisabledReason` accepts YouTube runtime state:

```ts
import type { YoutubeRuntimeStatus } from "$lib/types/youtube";

export function sourceSyncDisabledReason(
  source: Source,
  accountStatuses: Record<number, AccountRuntimeStatus>,
  youtubeRuntimeStatus: YoutubeRuntimeStatus | null = null,
) {
  const capabilities = sourceCapabilities(source);
  if (!capabilities.canSync) return "This source type is not syncable.";

  if (source.sourceType === "youtube") {
    if (youtubeRuntimeStatus && !youtubeRuntimeStatus.ytdlpAvailable) {
      return youtubeRuntimeStatus.message || "yt-dlp is not available on PATH.";
    }
    return null;
  }

  if (!capabilities.requiresAccount) return null;
  // keep existing Telegram account checks unchanged
}
```

Add tests:

```ts
expect(sourceSyncDisabledReason(youtubeVideoSource, {}, {
  ytdlpAvailable: false,
  ytdlpVersion: null,
  message: "yt-dlp is not available on PATH",
})).toBe("yt-dlp is not available on PATH");

expect(sourceSyncDisabledReason(youtubeVideoSource, {}, {
  ytdlpAvailable: true,
  ytdlpVersion: "2026.01.01",
  message: "yt-dlp is available",
})).toBeNull();
```

- [ ] Update `src/lib/analysis-scope-state.ts` to remove Telegram-only wording:

```ts
if (metrics) {
  return `${metrics.item_count} synced items available locally for analysis.`;
}
return "This source is available in the workspace but has no synced item count yet.";
```

Update `analysis-scope-state.test.ts` expected strings from `synced messages` to `synced items`.

- [ ] Make `src/lib/components/source-row.svelte` and `workspace-rail.svelte` accept optional YouTube summaries.

Props to add:

```ts
youtubeSummary: YoutubeSourceSummary | null;
youtubeRuntimeStatus: YoutubeRuntimeStatus | null;
```

Display rules:

- For YouTube video rows, show channel handle/title, duration, published date, captions label, comments label, availability, direct YouTube link, and playlist membership badge when `YoutubeVideoDetail.playlistMemberships.length > 0`.
- For YouTube playlist rows, show channel, video count, linked child videos count, unavailable count, last transcript/comments sync from summary statuses, and availability.
- For non-YouTube rows, preserve existing Telegram/RSS/forum behavior.
- Thumbnail image source is `youtubeSummary.thumbnailUrl ?? source.avatarDataUrl`; fallback is `sourceInitial(source)`.
- Use `sourceCapabilities(source).contentLabel` for count labels instead of hard-coded `msgs`.

- [ ] Define the single props contract for `src/lib/components/analysis/youtube-source-detail.svelte`:

```ts
import type { SourceJobRecord } from "$lib/types/sources";
import type { Source } from "$lib/types/sources";
import type { YoutubeVideoDetail } from "$lib/types/youtube";

let {
  source,
  detail,
  jobs,
  loadingDetail,
  formatTimestamp,
  onSyncMetadata,
  onSyncTranscript,
  onSyncComments,
  onCancelJob,
}: {
  source: Source;
  detail: YoutubeVideoDetail | null;
  jobs: SourceJobRecord[];
  loadingDetail: boolean;
  formatTimestamp: (value: number | null) => string;
  onSyncMetadata: (sourceId: number) => void | Promise<void>;
  onSyncTranscript: (sourceId: number) => void | Promise<void>;
  onSyncComments: (sourceId: number) => void | Promise<void>;
  onCancelJob: (jobId: string) => void | Promise<void>;
} = $props();
```

Tabs:

```text
Overview
Transcript
Comments
Jobs
```

Data displayed:

- Overview: title, channel, handle, published date, duration, availability, canonical link.
- Transcript: `detail.summary.captions.label`, item count, segment count, last synced timestamp.
- Comments: `detail.summary.comments.label`, item count, last synced timestamp.
- Jobs: active and recent jobs filtered to this `source.id`.

- [ ] Define the single props contract for `src/lib/components/analysis/youtube-playlist-detail.svelte`:

```ts
import type { SourceJobRecord } from "$lib/types/sources";
import type { Source } from "$lib/types/sources";
import type { YoutubePlaylistDetail } from "$lib/types/youtube";

let {
  source,
  detail,
  jobs,
  loadingDetail,
  formatTimestamp,
  onOpenSource,
  onSyncPlaylist,
  onRetryFailed,
  onSyncPlaylistVideo,
  onRetryPlaylistVideo,
  onCancelJob,
}: {
  source: Source;
  detail: YoutubePlaylistDetail | null;
  jobs: SourceJobRecord[];
  loadingDetail: boolean;
  formatTimestamp: (value: number | null) => string;
  onOpenSource: (sourceId: number) => void | Promise<void>;
  onSyncPlaylist: (sourceId: number) => void | Promise<void>;
  onRetryFailed: (sourceId: number) => void | Promise<void>;
  onSyncPlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
  onRetryPlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
  onCancelJob: (jobId: string) => void | Promise<void>;
} = $props();
```

Playlist row behavior:

- `open source` is enabled only when `videoSourceId !== null`.
- `sync this video` is enabled only when `videoSourceId !== null` and `isRemovedFromPlaylist === false`.
- `retry this video` is enabled only when `videoSourceId !== null`, `isRemovedFromPlaylist === false`, and `availabilityStatus` is one of:

```text
live_ended_transcript_pending
no_captions
unavailable_unknown
```

- [ ] Wire `workspace-main.svelte`:

```text
currentSource.sourceType === "youtube" && currentSource.sourceSubtype === "video"
  -> YoutubeSourceDetail

currentSource.sourceType === "youtube" && currentSource.sourceSubtype === "playlist"
  -> YoutubePlaylistDetail

otherwise
  -> SourceContextPanel
```

Keep Telegram/RSS/forum `SourceContextPanel` behavior unchanged except for provider-aware labels.

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::detail youtube::runtime --lib
cd ..
npm test -- youtube-detail analysis-source-state analysis-scope-state source-capabilities
npm run check
```

Expected: runtime and detail DTO tests pass, YouTube sync is disabled when `yt-dlp` is unavailable, scope summaries say synced items, and Svelte typecheck passes.

- [ ] Commit:

```powershell
git add src-tauri/src/youtube src-tauri/src/lib.rs src/lib/types/youtube.ts src/lib/api/youtube-detail.ts src/lib/api/youtube-detail.test.ts src/lib/components src/lib/analysis-source-state.ts src/lib/analysis-source-state.test.ts src/lib/analysis-scope-state.ts src/lib/analysis-scope-state.test.ts
git commit -m "feat: polish youtube source workspace"
```

---

## Task 2: Job Controls and Analysis Controls

**Files:**

- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/run-controls.svelte`
- Modify: `src/lib/components/analysis/workspace-main.svelte`
- Modify: `src/lib/components/analysis/source-context-panel.svelte`
- Modify: `src/lib/api/source-jobs.ts`
- Modify: `src/lib/api/source-jobs.test.ts`
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/lib/types/analysis.ts`

- [ ] Extend `src/lib/api/source-jobs.ts` with the Part 3 commands that were not exposed in the first job wiring:

```ts
export function syncYoutubePlaylistVideo(
  playlistSourceId: number,
  videoSourceId: number,
  options: YoutubeSyncOptions,
) {
  return invoke<SourceJobRecord>("sync_youtube_playlist_video", {
    playlistSourceId,
    videoSourceId,
    options,
  });
}

export function retryFailedYoutubePlaylistVideos(sourceId: number, options: YoutubeSyncOptions) {
  return invoke<SourceJobRecord>("retry_failed_youtube_playlist_videos", {
    sourceId,
    options,
  });
}

export function cancelSourceJob(jobId: string) {
  return invoke<void>("cancel_source_job", { jobId });
}
```

Add `source-jobs.test.ts` assertions that each wrapper sends the exact camelCase argument names above.

- [ ] Map video job controls to exact commands:

```text
sync metadata
  -> syncYoutubeSource(source.id, { metadata: true, transcripts: false, comments: false })

sync transcript
  -> syncYoutubeSource(source.id, { metadata: false, transcripts: true, comments: false })

sync comments
  -> syncYoutubeSource(source.id, { metadata: false, transcripts: false, comments: true })

cancel current job
  -> cancelSourceJob(job.job_id)
```

Only show `cancel current job` for jobs whose status is `queued`, `running`, or `cancel_requested`; disable the button while status is `cancel_requested`.

- [ ] Map playlist job controls to exact commands:

```text
sync all playlist videos
  -> syncYoutubeSource(playlist.id, { metadata: true, transcripts: true, comments: false })

sync failed videos only
  -> retryFailedYoutubePlaylistVideos(playlist.id, { metadata: false, transcripts: true, comments: false })

cancel current playlist job
  -> cancelSourceJob(job.job_id)
```

`sync all playlist videos` means: refresh playlist metadata, create/link any child video sources, and sync transcripts for linked child videos through the Part 3 playlist full-sync path. It does not fetch comments by default; comments remain an explicit video-level action to avoid a surprising large comment crawl.

- [ ] Map per-video playlist row actions:

```text
open source
  -> onSelectSource(videoSourceId)

sync this video
  -> syncYoutubePlaylistVideo(playlistSourceId, videoSourceId, {
       metadata: true,
       transcripts: true,
       comments: false,
     })

retry this video
  -> syncYoutubePlaylistVideo(playlistSourceId, videoSourceId, {
       metadata: false,
       transcripts: true,
       comments: false,
     })
```

`retry this video` is a single-row retry, not the aggregate `retryFailedYoutubePlaylistVideos` command.

- [ ] Add YouTube corpus mode selector for YouTube scopes only.

TypeScript state:

```ts
import type { YoutubeCorpusMode } from "$lib/types/analysis";

let youtubeCorpusMode = $state<YoutubeCorpusMode>("transcript_description");
```

Selector values:

```text
transcript_only -> transcript only
transcript_description -> transcript + description
transcript_description_comments -> transcript + description + comments
```

Command mapping:

```ts
startAnalysisReport({
  // existing fields
  youtubeCorpusMode: isYoutubeAnalysisScope ? youtubeCorpusMode : "transcript_description",
});
```

Telegram scopes ignore the field backend-side, but frontend still sends the default value to keep the command shape stable.

- [ ] Hide Telegram topic controls for YouTube sources with concrete component changes:

```text
src/routes/analysis/+page.svelte
  - Do not call listSourceForumTopics when sourceCapabilities(source).hasTopics is false.
  - Reset sourceTopics to [] and selectedTopicKey to "__all_topics__" when selecting a YouTube source.
  - Pass topicFilter: null to listSourceItems for non-topic sources.

src/lib/components/analysis/source-context-panel.svelte
  - Rename visible strings from "Recent synced messages" to provider-aware "Recent synced {contentLabel}".
  - Render the Topic view <Select> only when showTopicSelector is true.
  - Show "{count} {contentLabel}" instead of "{count} messages".

src/lib/analysis-state.ts
  - shouldShowTopicSelector remains false for YouTube because sourceCapabilities(source).hasTopics is false.
```

Add tests:

```ts
expect(shouldShowTopicSelector(youtubeVideoSource, "single_source", false, [])).toBe(false);
expect(currentTopicFilter("__all_topics__", [])).toBeNull();
```

- [ ] Run:

```powershell
npm test -- source-jobs analysis-run-workflow analysis-state analysis-source-state analysis-scope-state
npm run check
```

Expected: wrapper tests prove exact command mappings, YouTube controls render only for YouTube scopes, Telegram topic controls remain unchanged for Telegram supergroups, and YouTube sources do not load or display forum topic controls.

- [ ] Commit:

```powershell
git add src/routes/analysis/+page.svelte src/lib/components/analysis src/lib/api/source-jobs.ts src/lib/api/source-jobs.test.ts src/lib/analysis-state.ts src/lib/analysis-state.test.ts src/lib/types/analysis.ts
git commit -m "feat: complete youtube workspace controls"
```

---

## Task 3: Manual Hardening Matrix

**Files:**

- Modify only files required by defects discovered during this task.
- Add focused automated tests next to the changed code for every defect that can be reproduced without live YouTube/network access.
- If a defect depends on live provider behavior, add a concise manual verification note to `docs/youtube-manual-verification.md`.

- [ ] Create or update `docs/youtube-manual-verification.md` with a table containing these columns:

```text
Scenario
Input URL or source fixture
Steps
Expected result
Result
Notes
```

- [ ] Test public video with manual captions.

Pass criteria:

- Preview succeeds.
- Add creates one YouTube video source.
- Transcript sync creates one `youtube_transcript` item.
- `youtube_transcript_segments` has at least one row.
- Video detail shows captions state `synced`.
- Analysis with `transcript_only` can start.

- [ ] Test public video with auto captions.

Pass criteria:

- Transcript sync falls back to auto captions when manual captions are absent.
- Caption metadata records `auto`.
- Video detail still shows captions state `synced`.

- [ ] Test video with no captions.

Pass criteria:

- Transcript sync completes without panicking.
- Availability/status becomes `no_captions` or an equivalent unavailable caption state from earlier parts.
- Video detail shows captions state `unavailable`.
- Analysis preflight either excludes the source or returns a clear validation error instead of sending an empty transcript.

- [ ] Test Shorts URL.

Pass criteria:

- URL parser canonicalizes the Shorts URL to the video source.
- Add dedupes against the same canonical video id.
- Detail view direct link opens the canonical YouTube URL.

- [ ] Test live URL.

Pass criteria:

- Metadata sync records `live_now` or the current availability status.
- Transcript sync does not spin indefinitely.
- UI shows an availability badge and any provider warning.

- [ ] Test upcoming or live-ended source if available.

Pass criteria:

- Upcoming videos are not treated as deleted.
- Live-ended transcript-pending videos are marked `live_ended_transcript_pending`.
- Retry controls are enabled only for statuses listed as retryable in Part 3.

- [ ] Test public playlist.

Pass criteria:

- Playlist preview is bounded to the Part 2 limit.
- Full playlist metadata sync pages through entries.
- Playlist detail shows ordered rows.
- Linked child videos count matches `video_source_id IS NOT NULL` active rows.
- `sync all playlist videos` starts one playlist full-sync job.

- [ ] Test playlist with removed/private/unavailable entries.

Pass criteria:

- Removed rows show `removed_from_playlist`.
- Private/auth/member/age/geo/deleted rows are not retried by the aggregate retry command until auth/settings allow it.
- Unlinked rows do not appear in analysis corpus.
- Playlist detail displays unavailable count.

- [ ] Test direct video first, then playlist containing the same video.

Pass criteria:

- The playlist row links to the existing canonical video source.
- No duplicate video source is created.
- `open source` from the playlist row selects the existing video source.

- [ ] Test comments-heavy video with cancellation.

Pass criteria:

- Comment fetch uses the configured max comments limit.
- Cancellation sets the job to `cancel_requested` then `cancelled` or a clear terminal failed state.
- Partial comment rows, if any, remain valid `youtube_comment` items.
- UI clears active pending state after terminal job status.

- [ ] Test saved analysis run, then resync transcript in a different caption language.

Pass criteria:

- Saved run still displays the original snapshot metadata and trace refs.
- New transcript sync updates live source items without mutating saved run snapshot rows.
- Timestamp trace links from the saved run still resolve.

- [ ] Test app restart during active YouTube job.

Expected MVP behavior:

- YouTube source jobs are in memory and are not restored after app restart.
- After restart, `list_source_jobs` returns no active job from the previous process.
- The UI must not leave `syncingIds`, active job badges, or disabled sync buttons stuck from the old process.
- Completed database writes from before shutdown remain visible.
- The user can start a new sync job for the same source after restart.
- No attempt is made in this MVP to resume an interrupted `yt-dlp` process.

- [ ] For every defect fixed, add one of:

```text
unit/component/API test for deterministic behavior
Rust unit test for parser/query/state behavior
manual verification note in docs/youtube-manual-verification.md for live-provider-only behavior
```

- [ ] Run:

```powershell
cd src-tauri
cargo test --lib
cargo clippy --all-targets -- -D warnings
cd ..
npm test
npm run check
npm run build
```

Expected: all automated checks pass, including clippy, and the manual matrix contains pass/fail notes for every scenario.

- [ ] Commit:

```powershell
git add src-tauri src docs/youtube-manual-verification.md
git commit -m "fix: harden youtube source workflows"
```

---

## Task 4: Documentation

**Files:**

- Modify: `README.md`
- Modify: `docs/database-schema.md`
- Modify: `docs/architecture-deep-dive.md`
- Modify: `docs/backlog.md`
- Modify: `docs/youtube-manual-verification.md`

- [ ] Add README notes:

```text
yt-dlp must be installed and available on PATH for YouTube source support.
Extractum does not download YouTube audio/video binaries in the MVP.
Auth-gated content requires YouTube cookies configured in Settings.
YouTube sync jobs are in memory in the MVP and are not resumed after app restart.
```

- [ ] Document schema additions:

```text
items.item_kind
youtube_playlist_items
youtube_transcript_segments
analysis_run_messages YouTube snapshot columns
analysis_source_groups.source_type
YouTube partial unique indexes on sources
```

- [ ] Document architecture additions:

```text
youtube/ Rust module
yt-dlp adapter boundary and runtime check
source jobs and in-memory restart behavior
playlist expansion
timestamp evidence refs
secure cookie handling
read-only YouTube detail/summary commands
```

- [ ] Move future YouTube work to backlog without implying existing NotebookLM export is post-MVP.

Backlog entries:

```text
YouTube-specific NotebookLM export enrichment: include transcript segment timestamps, canonical video links, and playlist membership metadata in NotebookLM export output.
Speech-to-text fallback for videos without captions.
Live chat ingest.
Media-aware analysis over thumbnails or downloaded media if a future setting explicitly allows media downloads.
Persistent/resumable YouTube source jobs across app restart.
```

Do not move the existing generic NotebookLM export feature to backlog; it already exists and must keep working for Telegram sources.

- [ ] Run:

```powershell
git diff -- README.md docs/database-schema.md docs/architecture-deep-dive.md docs/backlog.md docs/youtube-manual-verification.md
```

Expected: docs describe the implemented MVP, restart behavior, runtime requirement, schema additions, and future work clearly.

- [ ] Commit:

```powershell
git add README.md docs/database-schema.md docs/architecture-deep-dive.md docs/backlog.md docs/youtube-manual-verification.md
git commit -m "docs: document youtube source MVP"
```

---

## Final Verification

- [ ] Run:

```powershell
cd src-tauri
cargo test --lib
cargo clippy --all-targets -- -D warnings
cd ..
npm test
npm run check
npm run build
```

- [ ] Confirm the MVP acceptance checklist:

```text
preview video
preview playlist
add video
add playlist
canonical video dedupe
playlist membership
metadata sync
transcript sync
comments sync
YouTube-only groups
playlist analysis expansion
timestamp trace refs
saved run stability
secure cookie handling
yt-dlp unavailable state shown before sync
provider-aware synced item labels
no Telegram topic controls for YouTube sources
manual restart behavior verified
no audio/video binary downloads
```
