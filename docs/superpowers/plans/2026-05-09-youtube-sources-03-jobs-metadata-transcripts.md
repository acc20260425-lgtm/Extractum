# YouTube Sources Part 3: Jobs, Metadata, and Transcripts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add cancellable YouTube source jobs, metadata sync, and transcript ingest with timestamp segments.

**Architecture:** Jobs are in-memory and provider-neutral in shape, following the existing Takeout job pattern. Transcript sync writes one text item per video plus separate timestamp segment rows; analysis integration waits until Part 4.

**Tech Stack:** Tauri 2, Rust 2021, sqlx SQLite, zstd, `yt-dlp`, Svelte 5.

---

## Consistent End State

After this part:

- YouTube video and playlist sources can start sync jobs.
- Metadata sync refreshes source metadata.
- Transcript sync creates `youtube_transcript` items and `youtube_transcript_segments`.
- Jobs can be listed and cancelled.
- UI exposes sync status without requiring analysis to support YouTube yet.

---

## Task 1: Source Job State

**Files:**

- Create: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/youtube/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/types/sources.ts`
- Create: `src/lib/api/source-jobs.ts`
- Modify: `src/routes/analysis/+page.svelte`

- [ ] Create `SourceJobState` with the same lifecycle style as `TakeoutImportState`.

- [ ] Define statuses:

```text
queued
running
succeeded
failed
cancel_requested
cancelled
```

- [ ] Define job types:

```text
youtube_video_metadata_sync
youtube_video_transcript_sync
youtube_video_comments_sync
youtube_video_full_sync
youtube_playlist_metadata_sync
youtube_playlist_full_sync
youtube_playlist_video_sync
```

- [ ] Add matching TypeScript types to `src/lib/types/sources.ts`:

```ts
export type SourceJobStatus =
  | "queued"
  | "running"
  | "succeeded"
  | "failed"
  | "cancel_requested"
  | "cancelled";

export type SourceJobType =
  | "youtube_video_metadata_sync"
  | "youtube_video_transcript_sync"
  | "youtube_video_comments_sync"
  | "youtube_video_full_sync"
  | "youtube_playlist_metadata_sync"
  | "youtube_playlist_full_sync"
  | "youtube_playlist_video_sync";

export interface YoutubeSyncOptions {
  metadata: boolean;
  transcripts: boolean;
  comments: boolean;
}

export interface SourceJobRecord {
  job_id: string;
  source_id: number;
  related_source_id: number | null;
  job_type: SourceJobType;
  status: SourceJobStatus;
  message: string | null;
  progress_current: number | null;
  progress_total: number | null;
  started_at: number;
  finished_at: number | null;
  warnings: string[];
  error: string | null;
}

export type SourceJobEvent = SourceJobRecord;
```

- [ ] Use active-job locking by YouTube job scope, not by `source_id` alone. This allows a video source to run `youtube_video_metadata_sync` and `youtube_video_transcript_sync` at the same time, while rejecting duplicate jobs for the same operation.

Use this key shape in `src-tauri/src/youtube/jobs.rs`:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SourceJobKey {
    source_id: i64,
    job_type: SourceJobType,
    related_source_id: Option<i64>,
}
```

Rules:

- `sync_youtube_source(source_id, ...)` uses `SourceJobKey { source_id, job_type, related_source_id: None }`.
- `sync_youtube_playlist_video(playlist_source_id, video_source_id, ...)` uses `SourceJobKey { source_id: playlist_source_id, job_type: SourceJobType::YoutubePlaylistVideoSync, related_source_id: Some(video_source_id) }`.
- `finish_job` removes the exact `SourceJobKey` that was stored on the job record.
- Duplicate active keys return `AppError::conflict`.

- [ ] Emit events with:

```rust
const SOURCE_JOB_EVENT: &str = "sources://source-job";
```

- [ ] Register managed state in `src-tauri/src/lib.rs`:

```rust
.manage(SourceJobState::new())
```

- [ ] Add commands:

```rust
sync_youtube_source(source_id: i64, options: YoutubeSyncOptions) -> SourceJobRecord
sync_youtube_playlist_video(playlist_source_id: i64, video_source_id: i64, options: YoutubeSyncOptions) -> SourceJobRecord
cancel_source_job(job_id: String) -> ()
list_source_jobs(filter: SourceJobListFilter) -> Vec<SourceJobRecord>
retry_failed_youtube_playlist_videos(source_id: i64, options: YoutubeSyncOptions) -> SourceJobRecord
```

- [ ] Add the list filter type in `src-tauri/src/youtube/jobs.rs`:

```rust
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceJobListFilter {
    pub source_id: Option<i64>,
    pub status: Option<SourceJobStatus>,
    pub limit: Option<usize>,
}
```

`list_source_jobs` must default to `limit = 100`, clamp the limit to `500`, sort newest first, and apply `source_id` and `status` filters before truncating.

- [ ] Define `retry_failed_youtube_playlist_videos` as one aggregate retry job:

```text
job_type = youtube_playlist_full_sync
source_id = playlist source id
options.metadata = false
options.transcripts = true
options.comments = false
```

The command finds current playlist rows where:

```sql
playlist_source_id = ?
AND is_removed_from_playlist = 0
AND availability_status IN (
    'live_ended_transcript_pending',
    'no_captions',
    'unavailable_unknown'
)
```

It retries those rows sequentially by calling the same per-video sync path used by `youtube_playlist_video_sync`. It must not retry `private_or_auth_required`, `members_only`, `age_restricted`, `geo_blocked`, `deleted`, or `removed_from_playlist` until Part 5 adds auth/settings controls. The aggregate job reports `progress_total` as the number of retryable rows and `progress_current` after each video.

- [ ] Add `src/lib/api/source-jobs.ts` with command wrappers and `listenToSourceJobEvents`.

Use these wrapper signatures:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { SourceJobRecord, SourceJobStatus, YoutubeSyncOptions } from "$lib/types/sources";

export interface SourceJobListFilter {
  sourceId?: number;
  status?: SourceJobStatus;
  limit?: number;
}

export function listSourceJobs(filter: SourceJobListFilter = {}) {
  return invoke<SourceJobRecord[]>("list_source_jobs", { filter });
}

export function syncYoutubeSource(sourceId: number, options: YoutubeSyncOptions) {
  return invoke<SourceJobRecord>("sync_youtube_source", { sourceId, options });
}

export function listenToSourceJobEvents(callback: (event: SourceJobRecord) => void) {
  return listen<SourceJobRecord>("sources://source-job", (event) => callback(event.payload));
}
```

- [ ] In `src/routes/analysis/+page.svelte`, add the first source-job UI wiring:

```ts
import {
  listSourceJobs,
  listenToSourceJobEvents,
  syncYoutubeSource,
  type SourceJobRecord,
} from "$lib/api/source-jobs";
```

Maintain active YouTube jobs next to the existing Takeout state:

```ts
let sourceJobsBySource = $state<Record<number, SourceJobRecord[]>>({});

function isActiveSourceJob(job: SourceJobRecord) {
  return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
}

function applySourceJob(job: SourceJobRecord) {
  sourceJobsBySource = {
    ...sourceJobsBySource,
    [job.source_id]: [
      job,
      ...(sourceJobsBySource[job.source_id] ?? []).filter((existing) => existing.job_id !== job.job_id),
    ],
  };
}
```

On mount, load only recent jobs:

```ts
for (const job of await listSourceJobs({ limit: 100 })) {
  applySourceJob(job);
}
```

Subscribe to `sources://source-job` and update `syncingIds` while a job is active:

Reuse the existing `sourceActionPending` and `clearSourceActionPending` helpers imported from `$lib/analysis-state`; they are already used by Telegram sync and Takeout import in this route. If the import block was refactored before this task is implemented, add them back to the `$lib/analysis-state` import rather than redefining local copies.

```ts
detachSourceJobListener = await listenToSourceJobEvents((job) => {
  applySourceJob(job);
  syncingIds = isActiveSourceJob(job)
    ? sourceActionPending(syncingIds, job.source_id)
    : clearSourceActionPending(syncingIds, job.source_id);
});
```

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::jobs --lib
cd ..
npm test -- source-jobs
npm run check
```

Expected: duplicate active jobs with the same `SourceJobKey` are rejected, different job types on the same source can coexist, filtered job listing works, retry chooses only retryable playlist rows, and event types compile.

- [ ] Commit:

```powershell
git add src-tauri/src/youtube src-tauri/src/lib.rs src/lib/api/source-jobs.ts src/lib/types/sources.ts src/routes/analysis/+page.svelte
git commit -m "feat: add source job orchestration"
```

---

## Task 2: Metadata Sync

**Files:**

- Modify: `src-tauri/src/youtube/metadata.rs`
- Modify: `src-tauri/src/youtube/playlist.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src/lib/source-capabilities.ts`
- Modify: `src/routes/analysis/+page.svelte`

- [ ] Implement video metadata refresh via:

```text
yt-dlp --dump-single-json --skip-download <canonical_url>
```

- [ ] Persist refreshed title, channel metadata, description, duration, chapters, tags, view/like/comment counts, thumbnail, availability, captions estimate, and video form.

- [ ] Implement playlist metadata refresh via:

```text
yt-dlp --dump-single-json --flat-playlist --skip-download --playlist-items <start>-<end> <playlist_url>
```

- [ ] Page full playlist metadata sync instead of fetching unbounded playlists in one process. Add this constant in `src-tauri/src/youtube/metadata.rs`:

```rust
const PLAYLIST_METADATA_PAGE_SIZE: i64 = 200;
```

Loop over ranges:

```rust
let mut start = 1_i64;
loop {
    let end = start + PLAYLIST_METADATA_PAGE_SIZE - 1;
    let range = format!("{start}-{end}");
    let page = fetch_playlist_metadata_page(playlist_url, &range).await?;
    if page.items.is_empty() {
        break;
    }
    all_items.extend(page.items);
    if all_items.len() % PLAYLIST_METADATA_PAGE_SIZE as usize != 0 {
        break;
    }
    start = end + 1;
}
```

The helper must call `yt-dlp` with `--playlist-items` and the range as two separate command arguments. Tests should assert that the generated command contains adjacent entries `--playlist-items` and `1-200`.

- [ ] Refresh playlist membership rows and mark removed entries in one transaction. Upsert every seen `video_id`, set `last_seen_at` to the sync timestamp, then mark rows absent from the current full refresh:

```sql
UPDATE youtube_playlist_items
SET is_removed_from_playlist = 1,
    availability_status = 'removed_from_playlist',
    updated_at = strftime('%s','now')
WHERE playlist_source_id = ?
  AND video_id NOT IN (...)
```

- [ ] Route YouTube source sync buttons in `src/routes/analysis/+page.svelte` to `sync_youtube_source`; keep Telegram sources on `sync_source`.

Update `syncSelectedSource` to branch on the selected source:

```ts
async function syncSelectedSource(sourceId: number) {
  const source = sourceCatalog.find((candidate) => candidate.id === sourceId);
  syncingIds = sourceActionPending(syncingIds, sourceId);
  try {
    if (!source) {
      throw new Error("Source is not loaded.");
    }
    if (source.sourceType === "youtube") {
      await syncYoutubeSource(sourceId, {
        metadata: true,
        transcripts: source.sourceSubtype === "video",
        comments: false,
      });
      status = "YouTube sync started.";
    } else {
      const result = await syncSource(sourceId);
      status = sourceSyncStatus(result);
      await Promise.all([loadSourceCatalog(), loadActiveRuns(), loadRuns()]);
      if (selectedSourceId === String(sourceId)) {
        await loadSourceTopics(sourceId, { preserveSelection: true });
        await loadItems(sourceId);
      }
    }
  } catch (error) {
    status = formatAppError("syncing the source", error);
  } finally {
    syncingIds = clearSourceActionPending(syncingIds, sourceId);
  }
}
```

Do not call `syncSource` for `source.sourceType === "youtube"`; that command remains Telegram-only.

- [ ] Set `canSync: true` for YouTube videos and playlists in `src/lib/source-capabilities.ts`.

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::metadata youtube::playlist youtube::jobs --lib
cd ..
npm test -- source-capabilities
npm run check
```

Expected: metadata sync tests pass and UI typecheck passes.

- [ ] Commit:

```powershell
git add src-tauri/src/youtube src/lib/source-capabilities.ts src/routes/analysis/+page.svelte
git commit -m "feat: sync youtube metadata"
```

---

## Task 3: Transcript Ingest

**Files:**

- Create: `src-tauri/src/youtube/captions.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/analysis/trace.rs`

- [ ] Add `captions` module to `src-tauri/src/youtube/mod.rs`.

- [ ] Move `tempfile = "3"` from `[dev-dependencies]` to `[dependencies]` in `src-tauri/Cargo.toml` if it is still dev-only. Captions need temporary output files at runtime.

- [ ] Implement caption selection policy:

```text
1. explicit video-source override
2. original-language manual captions
3. original-language auto captions
4. app preferred-language manual captions
5. app preferred-language auto captions
6. English manual captions
7. English auto captions
8. any manual track
9. any auto track
10. no_captions
```

- [ ] Fetch captions without media:

```text
yt-dlp --skip-download --write-subs --write-auto-subs --sub-langs <lang> --sub-format json3/vtt --output <temp-template> <url>
```

- [ ] Use `tempfile::TempDir` for caption downloads. Create the output template inside the temp dir and keep the `TempDir` value alive until after parsing finishes:

```rust
let temp_dir = tempfile::TempDir::new()?;
let output_template = temp_dir.path().join("%(id)s.%(ext)s");
```

Do not call `TempDir::into_path()`. Successful parsing, `yt-dlp` errors, and Rust error returns must all drop `TempDir` and remove downloaded caption files through RAII cleanup.

- [ ] Parse caption payloads into `YoutubeTranscriptSegment`.

Parser policy:

- Prefer `json3` because it preserves millisecond timing and structured cue segments.
- Fall back to `vtt` only when `yt-dlp` did not produce a `json3` file for the selected track.
- Detect the parser by file extension after the `yt-dlp` process exits.

Implement these helpers in `src-tauri/src/youtube/captions.rs`:

```rust
pub(crate) fn parse_json3_transcript(
    video_id: &str,
    language: Option<String>,
    track_kind: YoutubeCaptionTrackKind,
    payload: &str,
) -> AppResult<YoutubeTranscript>;

pub(crate) fn parse_vtt_transcript(
    video_id: &str,
    language: Option<String>,
    track_kind: YoutubeCaptionTrackKind,
    payload: &str,
) -> AppResult<YoutubeTranscript>;
```

`parse_json3_transcript` should read `events[*].tStartMs`, optional `events[*].dDurationMs`, and concatenate `events[*].segs[*].utf8` text. `parse_vtt_transcript` should parse cue headers in this form:

```text
00:12:34.000 --> 00:12:37.500
```

Unit tests must cover `json3` with multiple `segs`, `json3` with missing optional duration, `vtt` fallback, blank cue text being skipped, and invalid timing returning `AppError::validation`.

- [ ] Persist transcript in one transaction:

```text
items.item_kind = youtube_transcript
items.external_id = transcript:<video_id>:<language-or-und>:<manual|auto|unknown>
items.content_kind = text_only
youtube_transcript_segments rows replace previous active segments with DELETE + INSERT
```

- [ ] Add a transcript external id helper in `src-tauri/src/youtube/captions.rs` so multi-language tracks cannot collide on `UNIQUE(source_id, external_id)`:

```rust
pub(crate) fn transcript_external_id(
    video_id: &str,
    language: Option<&str>,
    track_kind: &YoutubeCaptionTrackKind,
) -> String {
    let language = language
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("und")
        .replace(':', "_");
    let kind = match track_kind {
        YoutubeCaptionTrackKind::Manual => "manual",
        YoutubeCaptionTrackKind::Auto => "auto",
        YoutubeCaptionTrackKind::Unknown => "unknown",
    };
    format!("transcript:{video_id}:{language}:{kind}")
}
```

- [ ] Add `upsert_youtube_transcript_item` in `src-tauri/src/sources/items.rs`; do not reuse `insert_source_item`, because transcript sync must update existing transcript text and return the item id.

Use this SQL shape:

```sql
INSERT INTO items (
    source_id,
    external_id,
    item_kind,
    author,
    published_at,
    ingested_at,
    content_zstd,
    raw_data_zstd,
    content_kind,
    has_media,
    media_kind,
    media_metadata_zstd
)
VALUES (?, ?, 'youtube_transcript', ?, ?, strftime('%s','now'), ?, ?, 'text_only', 0, NULL, NULL)
ON CONFLICT(source_id, external_id) DO UPDATE SET
    item_kind = excluded.item_kind,
    author = excluded.author,
    published_at = excluded.published_at,
    ingested_at = excluded.ingested_at,
    content_zstd = excluded.content_zstd,
    raw_data_zstd = excluded.raw_data_zstd,
    content_kind = excluded.content_kind,
    has_media = excluded.has_media,
    media_kind = excluded.media_kind,
    media_metadata_zstd = excluded.media_metadata_zstd
RETURNING id
```

- [ ] Replace transcript segments with an explicit delete-and-insert pattern in the same transaction. Part 3 does not keep historical segment versions.

```sql
DELETE FROM youtube_transcript_segments WHERE item_id = ?;
```

Then insert the new segments with `segment_index` taken from the parsed segment order:

```sql
INSERT INTO youtube_transcript_segments (
    item_id,
    source_id,
    segment_index,
    start_ms,
    end_ms,
    text,
    chapter_index,
    caption_language,
    caption_track_kind,
    is_auto_generated,
    metadata_zstd
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
```

- [ ] Update `normalize_ref` in `src-tauri/src/analysis/trace.rs` to accept:

```text
s12-i400@754000ms
s12-i400@754000-790000ms
```

Keep backward compatibility:

- `s12-i400` and `s12-m400` still normalize exactly as before.
- `@...` suffix is optional.
- Timestamp suffixes are accepted only for `-i` item refs, not legacy `-m` refs.
- Accepted suffix forms are `@<start_ms>ms` and `@<start_ms>-<end_ms>ms`; both numbers must be ASCII digits and `end_ms >= start_ms`.

Add tests:

```rust
assert_eq!(normalize_ref("[s12-i845]").as_deref(), Some("s12-i845"));
assert_eq!(normalize_ref("s12-m845").as_deref(), Some("s12-m845"));
assert_eq!(
    normalize_ref("s12-i400@754000ms").as_deref(),
    Some("s12-i400@754000ms")
);
assert_eq!(
    normalize_ref("[s12-i400@754000-790000ms]").as_deref(),
    Some("s12-i400@754000-790000ms")
);
assert_eq!(normalize_ref("s12-m400@754000ms"), None);
assert_eq!(normalize_ref("s12-i400@790000-754000ms"), None);
```

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::captions analysis::trace sources::items --lib
```

Expected: transcript parsing, persistence, and timestamp ref normalization pass.

- [ ] Commit:

```powershell
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/youtube src-tauri/src/sources src-tauri/src/analysis/trace.rs
git commit -m "feat: ingest youtube transcripts"
```

---

## Manual Verification

- [ ] Sync metadata for a public video.
- [ ] Sync transcript for a public video with manual captions.
- [ ] Sync transcript for a public video with auto captions.
- [ ] Sync a playlist metadata job and confirm membership rows update.
- [ ] Cancel a playlist job and confirm completed child work remains visible.
