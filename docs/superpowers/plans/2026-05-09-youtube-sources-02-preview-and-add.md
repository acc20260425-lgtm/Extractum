# YouTube Sources Part 2: Preview and Add Source Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users preview and save YouTube video or playlist sources without syncing transcripts/comments.

**Architecture:** Add the first `yt-dlp` adapter slice and keep all raw provider output behind normalized Rust DTOs. Source creation stores metadata and playlist membership only; analysis and sync still remain disabled for unfinished YouTube corpus paths.

**Tech Stack:** Tauri 2, Rust 2021, sqlx SQLite, `yt-dlp`, Svelte 5, Vitest.

---

## Consistent End State

After this part:

- Users can paste a YouTube URL, see preview metadata, and save a source.
- Video sources are canonical by `video_id`.
- Playlist sources own membership rows and reuse existing video sources.
- No transcript/comment items are created.
- Telegram sync and analysis still work.

---

## Task 1: `yt-dlp` Adapter and Preview Command

**Files:**

- Create: `src-tauri/src/youtube/ytdlp.rs`
- Create: `src-tauri/src/youtube/metadata.rs`
- Create: `src-tauri/src/youtube/preview.rs`
- Modify: `src-tauri/src/youtube/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] Add `ytdlp`, `metadata`, and `preview` modules to `src-tauri/src/youtube/mod.rs`.

- [ ] In `ytdlp.rs`, create a process boundary:

```rust
pub(crate) struct YtdlpOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) async fn run_ytdlp(args: &[String]) -> crate::error::AppResult<YtdlpOutput>;
```

Implementation requirements:

- Use `tokio::process::Command`.
- Pass arguments as separate values.
- Do not invoke a shell.
- Map missing binary to `AppError::validation("yt-dlp is not available on PATH")`.
- Define `const YTDLP_PREVIEW_TIMEOUT: Duration = Duration::from_secs(30);` in `ytdlp.rs`.
- Wrap process execution with `tokio::time::timeout(YTDLP_PREVIEW_TIMEOUT, ...)`.
- Keep the preview timeout exactly 30 seconds. Part 5 reuses this same preview contract for authenticated and unauthenticated `yt-dlp` runs.
- If the process exits with a non-zero status, map stderr through `youtube::errors`:
  - private/auth/members/age/geo text -> `AppError::auth`.
  - unavailable/deleted/not found text -> `AppError::not_found` or a preview availability status when raw JSON is still available.
  - timeout/network/rate-limit text -> `AppError::network`.
  - all other provider failures -> `AppError::validation` with a concise stderr message.

- [ ] Add command-builder helpers that can be unit tested without starting `yt-dlp`:

```rust
pub(crate) fn preview_video_args(canonical_url: &str) -> Vec<String>;
pub(crate) fn preview_playlist_args(canonical_url: &str) -> Vec<String>;
```

- [ ] In `metadata.rs`, add normalizers from small `serde_json::Value` samples to:

```text
YoutubePreview
YoutubeVideoMetadata
YoutubePlaylistMetadata
YoutubePlaylistItemMetadata
```

- [ ] In `preview.rs`, implement:

```rust
#[tauri::command]
pub async fn preview_youtube_source(url: String) -> AppResult<YoutubePreview> {
    let parsed = parse_youtube_url(&url)?;
    fetch_preview(parsed).await
}
```

- [ ] Use these command patterns:

```text
yt-dlp --dump-single-json --skip-download <canonical_url>
yt-dlp --dump-single-json --flat-playlist --playlist-items 1-50 --skip-download <canonical_playlist_url>
```

- Playlist preview must not enumerate unbounded playlists. Use `--playlist-items 1-50` for preview, map `playlist_count` or equivalent full-count metadata when `yt-dlp` provides it, and add a warning when only the first 50 entries are present in preview data.

- [ ] Register `preview_youtube_source` in `src-tauri/src/lib.rs`.

- [ ] Add unit tests with inline video and playlist JSON fixtures. Tests must not call YouTube.

Minimum assertions:

- video fixture maps `id`, `webpage_url`, `title`, `channel`, `channel_id`, `channel_url`, thumbnail, duration, upload date, and availability;
- missing optional fields map to `None` and do not panic;
- playlist fixture maps `playlist_id`, title, channel, count, first entries, and availability;
- `availability` values from `yt-dlp` map to `YoutubeAvailabilityStatus`;
- preview playlist args include `--playlist-items` and `1-50` as two adjacent `Vec<String>` entries, not a combined `--playlist-items=1-50` string;
- non-zero process output from a fake runner maps through `youtube::errors`.

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::metadata youtube::preview youtube::errors --lib
```

Expected: preview normalization passes without network.

- [ ] Commit:

```powershell
git add src-tauri/src/youtube src-tauri/src/lib.rs
git commit -m "feat: preview youtube sources"
```

---

## Task 2: Source Creation and Playlist Membership

**Files:**

- Create: `src-tauri/src/youtube/playlist.rs`
- Modify: `src-tauri/src/youtube/metadata.rs`
- Modify: `src-tauri/src/youtube/preview.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] Add transactional helpers for source upsert:

```rust
pub(crate) async fn upsert_youtube_video_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    metadata: &YoutubeVideoMetadata,
) -> AppResult<i64>;

pub(crate) async fn upsert_youtube_playlist_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<i64>;
```

Use SQLite upsert targets that match the partial unique indexes from migration 16. Do not use the Telegram uniqueness constraint because YouTube sources have `account_id = NULL`.

Video source upsert shape:

```sql
INSERT INTO sources (
    source_type,
    source_subtype,
    telegram_source_kind,
    account_id,
    external_id,
    title,
    metadata_zstd,
    is_active,
    is_member,
    created_at
)
VALUES ('youtube', 'video', NULL, NULL, ?, ?, ?, 1, 0, ?)
ON CONFLICT(source_type, source_subtype, external_id)
WHERE source_type = 'youtube' AND source_subtype = 'video'
DO UPDATE SET
    title = excluded.title,
    metadata_zstd = excluded.metadata_zstd,
    is_active = 1
RETURNING id
```

Playlist source upsert uses the same shape with `source_subtype = 'playlist'` and:

```sql
ON CONFLICT(source_type, source_subtype, external_id)
WHERE source_type = 'youtube' AND source_subtype = 'playlist'
DO UPDATE SET
    title = excluded.title,
    metadata_zstd = excluded.metadata_zstd,
    is_active = 1
RETURNING id
```

- [ ] Persist YouTube source rows with:

```text
source_type = youtube
source_subtype = video | playlist
external_id = video_id | playlist_id
account_id = NULL
telegram_source_kind = NULL
metadata_zstd = normalized YouTube metadata JSON
```

- [ ] Add `upsert_playlist_items` in `playlist.rs`.

Rules:

- Upsert by `(playlist_source_id, video_id)`.
- Reuse canonical video source if it already exists.
- Create a video source for available playlist entries when metadata is reliable by calling the same race-safe `upsert_youtube_video_source` helper.
- Handle concurrent playlist additions by relying on `ON CONFLICT ... DO UPDATE RETURNING id` for canonical video source creation.
- Keep unavailable entries as membership rows with `video_source_id = NULL`.
- Mark rows not seen in the latest playlist sync as `is_removed_from_playlist = 1`.

- [ ] Expose:

```rust
#[tauri::command]
pub async fn add_youtube_source(handle: AppHandle, url: String) -> AppResult<SourceRecord>;
```

Behavior:

- Parse URL.
- Fetch fresh metadata via the adapter before opening the database transaction.
- Open one database transaction for persistence.
- Save video or playlist source inside the transaction.
- For playlists, save membership rows inside the same transaction.
- Commit only after source and membership persistence succeed.
- Roll back automatically on any persistence error so a failed playlist add does not leave a playlist source without membership rows.
- Return the saved `SourceRecord`.
- Do not create `items` rows.

- [ ] Register `add_youtube_source` in `src-tauri/src/lib.rs`.

- [ ] Fix `source_record_from_row` so non-Telegram source metadata is not decoded as Telegram metadata.

Current `source_record_from_row` calls `decode_source_metadata` from `peer_resolution.rs`, which expects Telegram-shaped metadata. Refactor so only Telegram rows use that decoder for `avatar_cache_key`; YouTube rows should return `avatar_data_url = None` in this part.

Introduce a small helper that can be tested without a Tauri `AppHandle`:

```rust
fn source_avatar_cache_key_from_row(row: &SourceRecordRow) -> AppResult<Option<String>> {
    if row.source_type != TELEGRAM_SOURCE_TYPE {
        return Ok(None);
    }

    let metadata = decode_source_metadata(row.metadata_zstd.as_deref())?;
    Ok(metadata.avatar_cache_key)
}
```

Then `source_record_from_row` should call this helper and pass the returned cache key into `read_source_avatar_data_url`.

Add a store test that creates a YouTube `SourceRecordRow` with compressed YouTube metadata JSON and proves the helper skips Telegram decoding:

```rust
#[test]
fn avatar_cache_key_skips_non_telegram_metadata() {
    let metadata_zstd = crate::compression::compress_json_bytes(
        br#"{"youtube":{"video_id":"abc123","title":"Demo"}}"#,
    )
    .expect("compress youtube metadata");

    let row = SourceRecordRow {
        id: 10,
        source_type: "youtube".to_string(),
        source_subtype: Some("video".to_string()),
        telegram_source_kind: None,
        account_id: None,
        external_id: "abc123".to_string(),
        title: Some("Demo".to_string()),
        metadata_zstd: Some(metadata_zstd),
        last_sync_state: None,
        last_synced_at: None,
        is_active: true,
        is_member: false,
        created_at: 1,
    };

    assert_eq!(source_avatar_cache_key_from_row(&row).unwrap(), None);
}
```

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::playlist youtube::metadata sources::store --lib
```

Expected: canonical video dedupe and playlist membership tests pass.

- [ ] Commit:

```powershell
git add src-tauri/src/youtube src-tauri/src/sources/store.rs src-tauri/src/lib.rs
git commit -m "feat: save youtube sources"
```

---

## Task 3: Frontend Add Flow

**Files:**

- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`
- Modify: `src/lib/source-capabilities.ts`
- Modify: `src/lib/components/analysis/source-management-dialog.svelte`
- Create: `src/lib/components/analysis/youtube-source-add-panel.svelte`

- [ ] Add `YoutubePreview`, `YoutubePreviewKind`, and `YoutubeAvailabilityStatus` types to `src/lib/types/sources.ts`.

- [ ] Add API wrappers:

```ts
export function previewYoutubeSource(url: string) {
  return invoke<RawYoutubePreview>("preview_youtube_source", { url }).then(mapYoutubePreview);
}

export function addYoutubeSource(url: string) {
  return invoke<RawSource>("add_youtube_source", { url }).then(mapSource);
}
```

- [ ] Add explicit tests in `src/lib/api/sources.test.ts`:

```ts
it("previews youtube sources with a url argument", async () => {
  invokeMock.mockResolvedValueOnce({
    kind: "video",
    external_id: "abc123",
    canonical_url: "https://www.youtube.com/watch?v=abc123",
    title: "Demo",
    channel_title: "Channel",
    channel_id: "UC1",
    channel_handle: "@channel",
    channel_url: "https://www.youtube.com/@channel",
    thumbnail_url: null,
    duration_seconds: 120,
    published_at: "2026-05-01",
    playlist_video_count: null,
    captions_estimate: null,
    availability_status: "available",
    warnings: [],
  });

  await expect(previewYoutubeSource("https://youtu.be/abc123")).resolves.toMatchObject({
    kind: "video",
    externalId: "abc123",
    canonicalUrl: "https://www.youtube.com/watch?v=abc123",
  });
  expect(invokeMock).toHaveBeenLastCalledWith("preview_youtube_source", {
    url: "https://youtu.be/abc123",
  });
});

it("adds youtube sources with a url argument", async () => {
  invokeMock.mockResolvedValueOnce({
    id: 10,
    source_type: "youtube",
    source_subtype: "video",
    account_id: null,
    external_id: "abc123",
    title: "Demo",
    last_sync_state: null,
    last_synced_at: null,
    is_member: false,
    is_active: true,
    created_at: 1,
    avatar_data_url: null,
  });

  await expect(addYoutubeSource("https://youtu.be/abc123")).resolves.toMatchObject({
    id: 10,
    sourceType: "youtube",
    externalId: "abc123",
  });
  expect(invokeMock).toHaveBeenLastCalledWith("add_youtube_source", {
    url: "https://youtu.be/abc123",
  });
});
```

- [ ] Convert `source-management-dialog.svelte` to provider tabs:

```text
Telegram
YouTube
```

- [ ] Keep the current Telegram account/dialog/manual add flow under the Telegram tab.

- [ ] Move YouTube-specific state and UI into `youtube-source-add-panel.svelte` instead of doubling the size of `source-management-dialog.svelte`.

Panel state:

```ts
let youtubeUrl = $state("");
let youtubePreview = $state<YoutubePreview | null>(null);
let previewingYoutube = $state(false);
let addingYoutube = $state(false);
let youtubeStatus = $state("");
let previewedUrl = $state("");
```

State rules:

- changing `youtubeUrl` clears `youtubePreview` when it no longer matches `previewedUrl`;
- preview errors write to `youtubeStatus` without touching Telegram local status;
- add errors write to `youtubeStatus` without clearing a valid preview;
- successful add calls `onSourcesChanged(source.id)`, clears URL/preview, and sends parent status;
- switching between Telegram and YouTube tabs preserves in-progress typed values but does not share status messages between providers.

- [ ] Add YouTube URL input, preview action, preview card, warnings, and confirm action in the new YouTube panel.

- [ ] Change current YouTube capabilities so both video and playlist return `canSync: false` in this part. The current code already returns `true` for YouTube playlists; this must be corrected until Part 3 wires real YouTube jobs.

- [ ] Add/update `source-capabilities.test.ts` assertions that YouTube video and playlist `canSync` are both false in Part 2.

- [ ] Run:

```powershell
npm test -- sources source-capabilities
npm run check
```

Expected: frontend source API tests and typecheck pass.

- [ ] Commit:

```powershell
git add src/lib src/lib/components/analysis/source-management-dialog.svelte src/lib/components/analysis/youtube-source-add-panel.svelte
git commit -m "feat: add youtube source management flow"
```

---

## Manual Verification

- [ ] With `yt-dlp` installed, preview a public video.
- [ ] Preview a public playlist.
- [ ] Add a video directly.
- [ ] Add a playlist containing that same video and confirm no duplicate canonical video source is created.
- [ ] Confirm source creation does not create transcript/comment items.
