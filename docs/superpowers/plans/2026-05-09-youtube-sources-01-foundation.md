# YouTube Sources Part 1: Schema and Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prepare Extractum's database, item model, and backend contracts for YouTube without exposing unfinished YouTube UI.

**Architecture:** This part only adds durable schema and compile-time contracts. Telegram ingestion, analysis, source listing, and sync must behave exactly as before after this part is complete.

**Tech Stack:** Tauri 2, Rust 2021, sqlx SQLite, Svelte 5, Vitest.

---

## Consistent End State

After this part:

- App starts and migrations apply.
- Existing Telegram sources and analysis still work.
- `items.item_kind` is present and all Telegram inserts use `telegram_message`.
- YouTube DTOs and URL parser compile and are tested.
- No YouTube source can be added from the UI yet.

---

## Task 1: YouTube Schema Foundation

**Files:**

- Create: `src-tauri/migrations/16.sql`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/sources/test_support.rs`

- [ ] Add this migration registration test to `src-tauri/src/migrations.rs`:

```rust
#[test]
fn includes_youtube_source_foundation_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 16)
        .expect("version 16 migration is registered");

    for fragment in [
        "ALTER TABLE items ADD COLUMN item_kind TEXT NOT NULL DEFAULT 'telegram_message'",
        "CREATE TABLE IF NOT EXISTS youtube_playlist_items",
        "CHECK (availability_status IN",
        "CREATE TABLE IF NOT EXISTS youtube_transcript_segments",
        "ALTER TABLE analysis_run_messages ADD COLUMN item_kind TEXT",
        "ALTER TABLE analysis_run_messages ADD COLUMN source_type TEXT",
        "ALTER TABLE analysis_run_messages ADD COLUMN source_subtype TEXT",
        "ALTER TABLE analysis_run_messages ADD COLUMN metadata_zstd BLOB",
        "ALTER TABLE analysis_source_groups ADD COLUMN source_type TEXT NOT NULL DEFAULT 'telegram'",
        "idx_sources_unique_youtube_video",
        "idx_sources_unique_youtube_playlist",
    ] {
        assert!(
            migration.sql.contains(fragment),
            "missing migration fragment {fragment}"
        );
    }
}
```

- [ ] Run the focused test and confirm it fails before adding the migration:

```powershell
cd src-tauri
cargo test migrations::tests::includes_youtube_source_foundation_migration --lib
```

Expected: the test fails because migration 16 is not registered.

- [ ] Create `src-tauri/migrations/16.sql`:

```sql
ALTER TABLE items ADD COLUMN item_kind TEXT NOT NULL DEFAULT 'telegram_message';

CREATE INDEX IF NOT EXISTS idx_items_source_kind_published
    ON items(source_id, item_kind, published_at DESC);

CREATE TABLE IF NOT EXISTS youtube_playlist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    video_source_id INTEGER REFERENCES sources(id) ON DELETE SET NULL,
    video_id TEXT NOT NULL,
    position INTEGER,
    title_snapshot TEXT,
    url TEXT,
    thumbnail_url TEXT,
    availability_status TEXT NOT NULL,
    is_removed_from_playlist INTEGER NOT NULL DEFAULT 0,
    last_seen_at INTEGER,
    metadata_zstd BLOB,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (availability_status IN (
        'available',
        'upcoming',
        'live_now',
        'live_ended_transcript_pending',
        'no_captions',
        'private_or_auth_required',
        'members_only',
        'age_restricted',
        'geo_blocked',
        'deleted',
        'removed_from_playlist',
        'unavailable_unknown'
    )),
    UNIQUE(playlist_source_id, video_id)
);

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_items_playlist_position
    ON youtube_playlist_items(playlist_source_id, position);

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_items_video_source
    ON youtube_playlist_items(video_source_id);

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_items_video_id
    ON youtube_playlist_items(video_id);

CREATE TABLE IF NOT EXISTS youtube_transcript_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    segment_index INTEGER NOT NULL,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER,
    text TEXT NOT NULL,
    chapter_index INTEGER,
    caption_language TEXT,
    caption_track_kind TEXT,
    is_auto_generated INTEGER NOT NULL DEFAULT 0,
    metadata_zstd BLOB,
    UNIQUE(item_id, segment_index)
);

CREATE INDEX IF NOT EXISTS idx_youtube_transcript_segments_item_time
    ON youtube_transcript_segments(item_id, start_ms);

CREATE INDEX IF NOT EXISTS idx_youtube_transcript_segments_source
    ON youtube_transcript_segments(source_id);

ALTER TABLE analysis_run_messages ADD COLUMN item_kind TEXT;
ALTER TABLE analysis_run_messages ADD COLUMN source_type TEXT;
ALTER TABLE analysis_run_messages ADD COLUMN source_subtype TEXT;
ALTER TABLE analysis_run_messages ADD COLUMN metadata_zstd BLOB;

ALTER TABLE analysis_source_groups ADD COLUMN source_type TEXT NOT NULL DEFAULT 'telegram';

CREATE INDEX IF NOT EXISTS idx_analysis_source_groups_source_type
    ON analysis_source_groups(source_type);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_youtube_video
    ON sources(source_type, source_subtype, external_id)
    WHERE source_type = 'youtube' AND source_subtype = 'video';

CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_youtube_playlist
    ON sources(source_type, source_subtype, external_id)
    WHERE source_type = 'youtube' AND source_subtype = 'playlist';

INSERT OR IGNORE INTO app_settings (key, value)
VALUES
    ('youtube.auth.enabled', 'false'),
    ('youtube.captions.preferred_language', 'original'),
    ('youtube.sync.delay_between_requests_ms', '1000'),
    ('youtube.sync.max_parallel_video_syncs', '1'),
    ('youtube.sync.max_parallel_comment_syncs', '1'),
    ('youtube.sync.pause_on_auth_challenge', 'true'),
    ('youtube.sync.daily_soft_limit', '0'),
    ('youtube.sync.retry_backoff_ms', '3000'),
    ('youtube.sync.stop_after_consecutive_failures', '3');
```

- [ ] Register migration 16 in `build_migrations()` after migration 15:

```rust
Migration {
    version: 16,
    description: "add youtube source foundation",
    sql: include_str!("../migrations/16.sql"),
    kind: MigrationKind::Up,
},
```

- [ ] Update in-memory source fixtures in `src-tauri/src/sources/test_support.rs` so the `items` table includes:

```sql
item_kind TEXT NOT NULL DEFAULT 'telegram_message'
```

- [ ] Update manual in-memory analysis schemas in `src-tauri/src/analysis/corpus.rs`, especially `snapshot_pool()`, so `analysis_run_messages` includes the new migration 16 columns:

```sql
item_kind TEXT,
source_type TEXT,
source_subtype TEXT,
metadata_zstd BLOB
```

- [ ] Keep `app_settings` defaults in the migration only. Unit tests that use `memory_pool()` without running migrations should not assume YouTube settings exist unless the test inserts them explicitly or uses the settings loader added in a later part.

- [ ] Run:

```powershell
cd src-tauri
cargo test migrations::tests::includes_youtube_source_foundation_migration --lib
cargo test sources::test_support --lib
cargo test analysis::corpus --lib
```

Expected: all pass.

- [ ] Commit:

```powershell
git add src-tauri/migrations/16.sql src-tauri/src/migrations.rs src-tauri/src/sources/test_support.rs src-tauri/src/analysis/corpus.rs
git commit -m "feat: add youtube schema foundation"
```

---

## Task 2: Semantic Item Kind Contract

**Files:**

- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`

- [ ] Add Rust constants in `src-tauri/src/sources/types.rs`:

```rust
pub(crate) const ITEM_KIND_TELEGRAM_MESSAGE: &str = "telegram_message";
pub(crate) const ITEM_KIND_YOUTUBE_TRANSCRIPT: &str = "youtube_transcript";
pub(crate) const ITEM_KIND_YOUTUBE_COMMENT: &str = "youtube_comment";
```

- [ ] Add `item_kind: String` to `StoredItemRow`.

- [ ] Add `item_kind: String` to `SourceItemInsert` in `src-tauri/src/sources/items.rs`.

- [ ] Update `insert_source_item` to insert `item_kind` into the `items` table.

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
    media_metadata_zstd,
    reply_to_msg_id,
    reply_to_peer_kind,
    reply_to_peer_id,
    reply_to_top_id,
    reaction_count
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(source_id, external_id) DO NOTHING
```

Bind `item.item_kind` immediately after `item.external_id`:

```rust
.bind(source_id)
.bind(&item.external_id)
.bind(&item.item_kind)
.bind(&item.author)
```

- [ ] Update the Telegram call site in `src-tauri/src/sources/sync.rs`:

```rust
SourceItemInsert {
    external_id: message_id.to_string(),
    item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
    author,
    published_at,
    payload: item_payload,
    raw_data,
    telegram_context,
}
```

- [ ] Update item-list queries to select `item_kind` and map it into `ItemRecord`.

In `src-tauri/src/sources/items/query.rs`, include:

```sql
items.item_kind,
```

between `items.external_id` and `items.author` in the `SELECT`.

- [ ] Update every direct `SourceItemInsert` construction in `src-tauri/src/sources/items.rs` tests with:

```rust
item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
```

- [ ] Update the manual `StoredItemRow` test query in `src-tauri/src/sources/items.rs` so it selects `item_kind`:

```sql
SELECT
    id, source_id, external_id, item_kind, author, published_at, content_kind, has_media,
    media_kind, content_zstd, media_metadata_zstd, raw_data_zstd,
    NULL AS forum_topic_id, NULL AS forum_topic_title, NULL AS forum_topic_top_message_id
FROM items
WHERE source_id = ? AND external_id = ?
```

- [ ] Update manual `INSERT INTO items` statements in `src-tauri/src/sources/items/query.rs` tests to either include `item_kind` explicitly or rely on the fixture default. Prefer explicit `item_kind` in tests that assert item row mapping.

- [ ] Add `itemKind: string` to `SourceItem` in `src/lib/types/sources.ts`.

- [ ] Add `item_kind: string` to `RawSourceItem` and map it in `src/lib/api/sources.ts`.

- [ ] Run:

```powershell
cd src-tauri
cargo test sources::items sources::sync --lib
cargo test sources::items::query --lib
cd ..
npm test -- sources
```

Expected: Telegram item insertion still passes and frontend source API tests compile.

- [ ] Commit:

```powershell
git add src-tauri/src/sources src/lib/types/sources.ts src/lib/api/sources.ts
git commit -m "feat: expose semantic item kinds"
```

---

## Task 3: YouTube DTOs and URL Parser

**Files:**

- Create: `src-tauri/src/youtube/mod.rs`
- Create: `src-tauri/src/youtube/dto.rs`
- Create: `src-tauri/src/youtube/url.rs`
- Create: `src-tauri/src/youtube/errors.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] Add dependencies:

```toml
url = "2"
```

- [ ] Check `src-tauri/Cargo.toml` before editing. If `url` already exists, reuse the existing dependency instead of adding a duplicate. Do not add `tempfile` in this part; it is introduced later when captions/cookies need temporary files.

- [ ] Add `mod youtube;` in `src-tauri/src/lib.rs`.

- [ ] Create `src-tauri/src/youtube/mod.rs`:

```rust
pub(crate) mod dto;
pub(crate) mod errors;
pub(crate) mod url;
```

- [ ] Define DTOs in `dto.rs` for:

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum YoutubeAvailabilityStatus {
    Available,
    Upcoming,
    LiveNow,
    LiveEndedTranscriptPending,
    NoCaptions,
    PrivateOrAuthRequired,
    MembersOnly,
    AgeRestricted,
    GeoBlocked,
    Deleted,
    RemovedFromPlaylist,
    UnavailableUnknown,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum YoutubePreviewKind {
    Video,
    Playlist,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubePreview {
    pub(crate) kind: YoutubePreviewKind,
    pub(crate) external_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) published_at: Option<String>,
    pub(crate) playlist_video_count: Option<i64>,
    pub(crate) captions_estimate: Option<YoutubeCaptionsEstimate>,
    pub(crate) availability_status: YoutubeAvailabilityStatus,
    pub(crate) warnings: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub(crate) struct YoutubeCaptionsEstimate {
    pub(crate) has_manual: bool,
    pub(crate) has_auto: bool,
    pub(crate) languages: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum YoutubeVideoForm {
    Regular,
    Short,
    Live,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeChapter {
    pub(crate) index: i64,
    pub(crate) title: String,
    pub(crate) start_ms: i64,
    pub(crate) end_ms: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeVideoMetadata {
    pub(crate) video_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) author_display: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) description: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) chapters: Vec<YoutubeChapter>,
    pub(crate) view_count: Option<i64>,
    pub(crate) like_count: Option<i64>,
    pub(crate) comment_count: Option<i64>,
    pub(crate) category: Option<String>,
    pub(crate) video_form: YoutubeVideoForm,
    pub(crate) availability_status: YoutubeAvailabilityStatus,
    pub(crate) raw_metadata_json: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubePlaylistMetadata {
    pub(crate) playlist_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) video_count: Option<i64>,
    pub(crate) items: Vec<YoutubePlaylistItemMetadata>,
    pub(crate) availability_status: YoutubeAvailabilityStatus,
    pub(crate) raw_metadata_json: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubePlaylistItemMetadata {
    pub(crate) video_id: String,
    pub(crate) position: Option<i64>,
    pub(crate) title_snapshot: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) availability_status: YoutubeAvailabilityStatus,
    pub(crate) raw_metadata_json: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum YoutubeCaptionTrackKind {
    Manual,
    Auto,
    Unknown,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeCaptionTrack {
    pub(crate) language: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) track_kind: YoutubeCaptionTrackKind,
    pub(crate) is_auto_generated: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeTranscriptSegment {
    pub(crate) index: i64,
    pub(crate) start_ms: i64,
    pub(crate) end_ms: Option<i64>,
    pub(crate) text: String,
    pub(crate) chapter_index: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeTranscript {
    pub(crate) video_id: String,
    pub(crate) language: Option<String>,
    pub(crate) track_kind: YoutubeCaptionTrackKind,
    pub(crate) is_auto_generated: bool,
    pub(crate) segments: Vec<YoutubeTranscriptSegment>,
    pub(crate) raw_payload: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct YoutubeComment {
    pub(crate) comment_id: String,
    pub(crate) parent_comment_id: Option<String>,
    pub(crate) is_reply: bool,
    pub(crate) author: Option<String>,
    pub(crate) author_channel_id: Option<String>,
    pub(crate) author_channel_url: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) text: String,
    pub(crate) like_count: Option<i64>,
    pub(crate) is_pinned: Option<bool>,
    pub(crate) is_hearted: Option<bool>,
    pub(crate) raw_payload: serde_json::Value,
}
```

- [ ] Implement `parse_youtube_url(input: &str) -> AppResult<YoutubeParsedUrl>` in `url.rs`.

Supported behavior:

- `youtube.com/watch?v=...` -> video.
- `youtu.be/...` -> video.
- `youtube.com/playlist?list=...` -> playlist.
- `youtube.com/shorts/...` -> short.
- `youtube.com/live/...` -> live.
- Any URL with `list=...` -> playlist.

The `list` parameter has priority over `v`: `youtube.com/watch?v=<video>&list=<playlist>` must parse as a playlist URL.

- [ ] Add URL parser tests for every supported URL shape, invalid host, empty input, and the `watch?v=...&list=...` playlist rule.

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::url youtube::dto --lib
```

Expected: parser and DTO tests pass.

- [ ] Commit:

```powershell
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src-tauri/src/youtube
git commit -m "feat: add youtube backend contracts"
```
