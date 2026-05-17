# YouTube Typed Source Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move normal YouTube runtime metadata ownership from `sources.metadata_zstd` into typed YouTube source tables while preserving managed backfill and inert legacy diagnostics.

**Architecture:** Add runner-managed migration 20 to create/backfill `youtube_video_sources` and `youtube_playlist_sources`, then make write paths atomically update `sources` plus the matching typed row. Detail, jobs, and analysis load validated typed rows; legacy `sources.metadata_zstd` and typed `raw_metadata_zstd` are not runtime fallbacks. Raw provider payload remains optional, versioned, and sanitized before persistence.

**Tech Stack:** Rust, Tauri commands, SQLx SQLite, zstd JSON compression helpers, existing YouTube DTOs, existing runner-managed migration pattern from migration 19.

---

## File Structure

- Create `src-tauri/src/youtube/source_metadata.rs`
  - Own typed YouTube source metadata table DDL, enum wire helpers, raw payload sanitation/compression, typed row validation, typed row upserts, and typed row loaders.
  - Expose small functions used by migration, source upsert, detail, jobs, and analysis.
- Modify `src-tauri/src/youtube/mod.rs`
  - Register the new `source_metadata` module.
- Create `src-tauri/migrations/20.sql`
  - Sentinel migration for runner-managed YouTube typed metadata backfill.
- Create `src-tauri/src/migrations/youtube_typed_source_metadata.rs`
  - Apply migration 20 in Rust, record the sentinel checksum, backfill valid legacy YouTube blobs, clear `sources.metadata_zstd` only after a typed row write succeeds, and leave invalid blobs inert.
- Modify `src-tauri/src/migrations.rs`
  - Register migration 20 and call the runner-managed migration before the SQL plugin can run the sentinel.
- Modify `src-tauri/src/sources/test_support.rs`
  - Add a focused helper to create YouTube typed source metadata tables in module tests that use hand-built schemas.
- Modify `src-tauri/src/sources/store.rs`
  - Stop encoding YouTube source metadata into `sources.metadata_zstd`.
  - Upsert source rows and typed metadata rows in the caller transaction.
  - Clear existing YouTube source blobs on successful typed writes.
- Modify `src-tauri/src/youtube/playlist.rs`
  - Ensure playlist item upserts still create typed linked video source rows via `upsert_youtube_video_source`.
  - Keep `youtube_playlist_items.metadata_zstd` unchanged and in scope only for playlist entry payloads.
- Modify `src-tauri/src/youtube/detail.rs`
  - Replace source blob decode with typed table joins/loaders.
  - Keep generic `sources.title` fallback for source summaries when typed rows are missing.
  - Return controlled validation errors for detail requests that require typed metadata.
- Modify `src-tauri/src/youtube/jobs.rs`
  - Replace source blob decode with typed metadata loaders.
  - Let explicit metadata job flows refresh missing/invalid typed rows, then reload typed rows before transcript/comment work.
  - Use the typed `caption_language_override` column.
- Modify `src-tauri/src/analysis/corpus.rs`
  - Replace source blob decode for YouTube descriptions and transcript evidence context with typed table reads.
  - Keep transcript timing/caption evidence in `youtube_transcript_segments`.
- Modify `docs/database-schema.md`
  - Document `youtube_video_sources`, `youtube_playlist_sources`, raw payload policy, and the new YouTube `sources.metadata_zstd` boundary.
- Modify `docs/backlog.md`
  - Mark the YouTube typed metadata ownership slice as planned/implemented according to the local backlog wording.
- Modify this plan as tasks complete.

## Task 0: Branch Guard And Baseline

**Files:**
- Read: `docs/superpowers/specs/2026-05-17-youtube-typed-source-metadata-design.md`
- Verify: Git status and focused baseline tests

- [x] **Step 1: Confirm clean starting point**

Run:

```powershell
git status --short --branch
git --no-pager log -5 --oneline --decorate
```

Expected:

```text
## main
0e1f074 (HEAD -> main) docs: clarify youtube metadata fallback boundaries
```

If the working tree contains user changes, inspect them and keep them intact.

- [x] **Step 2: Create an implementation branch or worktree**

Use `superpowers:using-git-worktrees` before execution. A safe branch name is:

```powershell
git switch -c feature/youtube-typed-source-metadata
```

If using a linked worktree, use:

```powershell
git worktree add .worktrees/youtube-typed-source-metadata -b feature/youtube-typed-source-metadata
```

- [x] **Step 3: Run focused baseline tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources:: youtube:: analysis::corpus::
```

Expected:

```text
test result: ok
```

- [x] **Step 4: Confirm no baseline changes**

Run:

```powershell
git status --short --branch
```

Expected: only the branch header, with no modified files.

## Task 1: Schema Sentinel And Typed Metadata Helper Skeleton

**Files:**
- Create: `src-tauri/migrations/20.sql`
- Create: `src-tauri/src/youtube/source_metadata.rs`
- Modify: `src-tauri/src/youtube/mod.rs`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/sources/test_support.rs`

- [x] **Step 1: Add RED tests for migration 20 registration**

In `src-tauri/src/migrations.rs`, inside `#[cfg(test)] mod tests`, add:

```rust
#[test]
fn includes_runner_managed_youtube_typed_source_metadata_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 20)
        .expect("version 20 migration is registered");

    assert_eq!(migration.description, "add youtube typed source metadata");
    assert!(
        migration
            .sql
            .contains("extractum_runner_managed_migration_20"),
        "v20 must fail if plugin-managed SQL applies it directly"
    );
}

#[test]
fn plugin_migration_list_keeps_v20_as_sentinel_only() {
    let migration = build_migrations()
        .into_iter()
        .find(|migration| migration.version == 20)
        .expect("version 20 migration is registered");

    assert!(!migration.sql.contains("CREATE TABLE youtube_video_sources"));
    assert!(!migration.sql.contains("CREATE TABLE youtube_playlist_sources"));
    assert!(!migration.sql.contains("INSERT INTO youtube_video_sources"));
}
```

Update the existing version-list assertion:

```rust
assert_eq!(versions, (1_i64..=20_i64).collect::<Vec<_>>());
```

- [x] **Step 2: Run migration registration tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_runner_managed_youtube_typed_source_metadata_migration migrations::tests::plugin_migration_list_keeps_v20_as_sentinel_only migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation
```

Expected: fail because migration 20 is not registered yet.

- [x] **Step 3: Add the sentinel SQL file**

Create `src-tauri/migrations/20.sql`:

```sql
-- Version 20 is applied by src-tauri/src/migrations/youtube_typed_source_metadata.rs.
-- This sentinel is registered so SQLx validates the applied checksum, but
-- direct plugin-managed execution must fail because v20 needs Rust-side zstd
-- JSON decode, typed validation, and transactional source-blob clearing.
SELECT extractum_runner_managed_migration_20();
```

- [x] **Step 4: Register migration 20**

In `src-tauri/src/migrations.rs`, append this entry to `build_migrations()` after version 19:

```rust
Migration {
    version: 20,
    description: "add youtube typed source metadata",
    sql: include_str!("../migrations/20.sql"),
    kind: MigrationKind::Up,
},
```

- [x] **Step 5: Add the typed metadata module skeleton**

In `src-tauri/src/youtube/mod.rs`, add:

```rust
pub(crate) mod source_metadata;
```

Create `src-tauri/src/youtube/source_metadata.rs` with the schema DDL and test helper entrypoint:

```rust
use sqlx::{Executor, Sqlite};

use crate::error::{AppError, AppResult};

pub(crate) const YOUTUBE_RAW_METADATA_VERSION: i64 = 1;

pub(crate) const YOUTUBE_TYPED_SOURCE_TABLES_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS youtube_video_sources (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    video_id TEXT NOT NULL,
    canonical_url TEXT NOT NULL,
    title TEXT,
    channel_title TEXT,
    channel_id TEXT,
    channel_handle TEXT,
    channel_url TEXT,
    author_display TEXT,
    published_at TEXT,
    duration_seconds INTEGER,
    description TEXT,
    thumbnail_url TEXT,
    tags_json TEXT NOT NULL DEFAULT '[]',
    chapters_json TEXT NOT NULL DEFAULT '[]',
    view_count INTEGER,
    like_count INTEGER,
    comment_count INTEGER,
    category TEXT,
    video_form TEXT NOT NULL,
    availability_status TEXT NOT NULL,
    caption_language_override TEXT,
    raw_metadata_version INTEGER,
    raw_metadata_zstd BLOB,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (video_form IN ('regular', 'short', 'live')),
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
    ))
);

CREATE INDEX IF NOT EXISTS idx_youtube_video_sources_video_id
    ON youtube_video_sources(video_id);

CREATE TABLE IF NOT EXISTS youtube_playlist_sources (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    playlist_id TEXT NOT NULL,
    canonical_url TEXT NOT NULL,
    title TEXT,
    channel_title TEXT,
    channel_id TEXT,
    channel_handle TEXT,
    channel_url TEXT,
    thumbnail_url TEXT,
    video_count INTEGER,
    availability_status TEXT NOT NULL,
    raw_metadata_version INTEGER,
    raw_metadata_zstd BLOB,
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
    ))
);

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_sources_playlist_id
    ON youtube_playlist_sources(playlist_id);
"#;

pub(crate) async fn create_youtube_typed_source_tables<'e, E>(executor: E) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::raw_sql(YOUTUBE_TYPED_SOURCE_TABLES_SQL)
        .execute(executor)
        .await
        .map_err(AppError::database)?;
    Ok(())
}
```

- [x] **Step 6: Add test support table creation**

In `src-tauri/src/sources/test_support.rs`, add:

```rust
pub(crate) async fn create_youtube_typed_source_tables(pool: &sqlx::SqlitePool) {
    crate::youtube::source_metadata::create_youtube_typed_source_tables(pool)
        .await
        .expect("create youtube typed source metadata tables");
}
```

- [x] **Step 7: Run migration registration tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_runner_managed_youtube_typed_source_metadata_migration migrations::tests::plugin_migration_list_keeps_v20_as_sentinel_only migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation
```

Expected:

```text
test result: ok
```

- [x] **Step 8: Commit**

```powershell
git add src-tauri/migrations/20.sql src-tauri/src/youtube/mod.rs src-tauri/src/youtube/source_metadata.rs src-tauri/src/migrations.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: add youtube typed metadata migration sentinel"
```

## Task 2: Typed Metadata Conversion, Validation, And Raw Payload Policy

**Files:**
- Modify: `src-tauri/src/youtube/source_metadata.rs`
- Test: `src-tauri/src/youtube/source_metadata.rs`

- [x] **Step 1: Add RED tests for typed conversion and validation**

At the bottom of `src-tauri/src/youtube/source_metadata.rs`, add:

```rust
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::youtube::dto::{
        YoutubeAvailabilityStatus, YoutubeChapter, YoutubePlaylistMetadata, YoutubeVideoForm,
        YoutubeVideoMetadata,
    };

    #[test]
    fn video_metadata_columns_include_wire_values_arrays_caption_override_and_sanitized_raw() {
        let metadata = video_metadata(json!({
            "id": "video01",
            "caption_language_override": "en",
            "http_headers": { "cookie": "secret" },
            "command_args": ["--cookies", "secret"]
        }));

        let columns = YoutubeVideoSourceColumns::try_from_metadata(&metadata)
            .expect("convert video metadata");

        assert_eq!(columns.video_form, "short");
        assert_eq!(columns.availability_status, "available");
        assert_eq!(columns.caption_language_override.as_deref(), Some("en"));
        assert_eq!(columns.tags_json, r#"["tag-one"]"#);
        assert!(columns.chapters_json.contains("\"start_ms\":1000"));
        assert_eq!(columns.raw_metadata_version, Some(YOUTUBE_RAW_METADATA_VERSION));

        let raw = decode_raw_payload_for_test(columns.raw_metadata_zstd.as_deref().unwrap());
        assert_eq!(raw["caption_language_override"], "en");
        assert!(raw.get("http_headers").is_none());
        assert!(raw.get("command_args").is_none());
        assert!(!raw.to_string().contains("secret"));
    }

    #[test]
    fn video_metadata_rejects_wrong_canonical_url_shape() {
        let mut metadata = video_metadata(json!({ "id": "video01" }));
        metadata.canonical_url = "https://example.com/watch?v=video01".to_string();

        let error = YoutubeVideoSourceColumns::try_from_metadata(&metadata)
            .expect_err("invalid url rejected");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.to_string().contains("canonical_url"));
    }

    #[test]
    fn playlist_metadata_columns_are_versioned_and_secret_safe() {
        let metadata = YoutubePlaylistMetadata {
            playlist_id: "PLdemo".to_string(),
            canonical_url: "https://www.youtube.com/playlist?list=PLdemo".to_string(),
            title: Some("Demo playlist".to_string()),
            channel_title: Some("Demo channel".to_string()),
            channel_id: Some("channel-1".to_string()),
            channel_handle: Some("@demo".to_string()),
            channel_url: Some("https://www.youtube.com/@demo".to_string()),
            thumbnail_url: None,
            video_count: Some(2),
            items: Vec::new(),
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: json!({ "id": "PLdemo", "headers": { "cookie": "secret" } }),
        };

        let columns = YoutubePlaylistSourceColumns::try_from_metadata(&metadata)
            .expect("convert playlist metadata");

        assert_eq!(columns.playlist_id, "PLdemo");
        assert_eq!(columns.availability_status, "available");
        assert_eq!(columns.raw_metadata_version, Some(YOUTUBE_RAW_METADATA_VERSION));
        let raw = decode_raw_payload_for_test(columns.raw_metadata_zstd.as_deref().unwrap());
        assert!(raw.get("headers").is_none());
    }

    fn video_metadata(raw_metadata_json: serde_json::Value) -> YoutubeVideoMetadata {
        YoutubeVideoMetadata {
            video_id: "video01".to_string(),
            canonical_url: "https://www.youtube.com/shorts/video01".to_string(),
            title: Some("Demo video".to_string()),
            channel_title: Some("Demo channel".to_string()),
            channel_id: Some("channel-1".to_string()),
            channel_handle: Some("@demo".to_string()),
            channel_url: Some("https://www.youtube.com/@demo".to_string()),
            author_display: Some("Demo channel".to_string()),
            published_at: Some("2026-05-17".to_string()),
            duration_seconds: Some(42),
            description: Some("Description".to_string()),
            thumbnail_url: Some("https://img.youtube.com/vi/video01/hqdefault.jpg".to_string()),
            tags: vec!["tag-one".to_string()],
            chapters: vec![YoutubeChapter {
                index: 0,
                title: "Intro".to_string(),
                start_ms: 1000,
                end_ms: Some(2000),
            }],
            view_count: Some(10),
            like_count: Some(5),
            comment_count: Some(2),
            category: Some("Education".to_string()),
            video_form: YoutubeVideoForm::Short,
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json,
        }
    }

    fn decode_raw_payload_for_test(bytes: &[u8]) -> serde_json::Value {
        let decoded = crate::compression::decompress_bytes(bytes).expect("decompress raw");
        serde_json::from_slice(&decoded).expect("parse raw")
    }
}
```

- [x] **Step 2: Run the new tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::source_metadata::
```

Expected: fail because the conversion structs and helpers do not exist.

- [x] **Step 3: Implement wire helpers, validation, and raw payload sanitation**

In `src-tauri/src/youtube/source_metadata.rs`, replace the imports with:

```rust
use serde_json::Value;
use sqlx::{Executor, Row, Sqlite};

use crate::compression::compress_json_bytes;
use crate::error::{AppError, AppResult};

use super::dto::{
    YoutubeAvailabilityStatus, YoutubeChapter, YoutubePlaylistMetadata, YoutubeVideoForm,
    YoutubeVideoMetadata,
};
use super::url::{parse_youtube_url, YoutubeUrlKind};
```

Add these structs:

```rust
#[derive(Clone, Debug)]
pub(crate) struct YoutubeVideoSourceColumns {
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
    pub(crate) tags_json: String,
    pub(crate) chapters_json: String,
    pub(crate) view_count: Option<i64>,
    pub(crate) like_count: Option<i64>,
    pub(crate) comment_count: Option<i64>,
    pub(crate) category: Option<String>,
    pub(crate) video_form: String,
    pub(crate) availability_status: String,
    pub(crate) caption_language_override: Option<String>,
    pub(crate) raw_metadata_version: Option<i64>,
    pub(crate) raw_metadata_zstd: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub(crate) struct YoutubePlaylistSourceColumns {
    pub(crate) playlist_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) video_count: Option<i64>,
    pub(crate) availability_status: String,
    pub(crate) raw_metadata_version: Option<i64>,
    pub(crate) raw_metadata_zstd: Option<Vec<u8>>,
}
```

Add these conversion helpers:

```rust
impl YoutubeVideoSourceColumns {
    pub(crate) fn try_from_metadata(metadata: &YoutubeVideoMetadata) -> AppResult<Self> {
        validate_video_canonical_url(&metadata.video_id, &metadata.canonical_url)?;
        let tags_json = serde_json::to_string(&metadata.tags)
            .map_err(|error| AppError::internal(error.to_string()))?;
        let chapters_json = serde_json::to_string(&metadata.chapters)
            .map_err(|error| AppError::internal(error.to_string()))?;
        let (raw_metadata_version, raw_metadata_zstd) =
            raw_metadata_columns(&metadata.raw_metadata_json)?;

        Ok(Self {
            video_id: metadata.video_id.clone(),
            canonical_url: metadata.canonical_url.clone(),
            title: metadata.title.clone(),
            channel_title: metadata.channel_title.clone(),
            channel_id: metadata.channel_id.clone(),
            channel_handle: metadata.channel_handle.clone(),
            channel_url: metadata.channel_url.clone(),
            author_display: metadata.author_display.clone(),
            published_at: metadata.published_at.clone(),
            duration_seconds: metadata.duration_seconds,
            description: metadata.description.clone(),
            thumbnail_url: metadata.thumbnail_url.clone(),
            tags_json,
            chapters_json,
            view_count: metadata.view_count,
            like_count: metadata.like_count,
            comment_count: metadata.comment_count,
            category: metadata.category.clone(),
            video_form: video_form_wire(&metadata.video_form).to_string(),
            availability_status: availability_status_wire(&metadata.availability_status).to_string(),
            caption_language_override: caption_language_override_from_raw(&metadata.raw_metadata_json),
            raw_metadata_version,
            raw_metadata_zstd,
        })
    }
}

impl YoutubePlaylistSourceColumns {
    pub(crate) fn try_from_metadata(metadata: &YoutubePlaylistMetadata) -> AppResult<Self> {
        validate_playlist_canonical_url(&metadata.playlist_id, &metadata.canonical_url)?;
        let (raw_metadata_version, raw_metadata_zstd) =
            raw_metadata_columns(&metadata.raw_metadata_json)?;

        Ok(Self {
            playlist_id: metadata.playlist_id.clone(),
            canonical_url: metadata.canonical_url.clone(),
            title: metadata.title.clone(),
            channel_title: metadata.channel_title.clone(),
            channel_id: metadata.channel_id.clone(),
            channel_handle: metadata.channel_handle.clone(),
            channel_url: metadata.channel_url.clone(),
            thumbnail_url: metadata.thumbnail_url.clone(),
            video_count: metadata.video_count,
            availability_status: availability_status_wire(&metadata.availability_status).to_string(),
            raw_metadata_version,
            raw_metadata_zstd,
        })
    }
}
```

Add validation and raw helpers:

```rust
fn validate_video_canonical_url(video_id: &str, canonical_url: &str) -> AppResult<()> {
    let parsed = parse_youtube_url(canonical_url).map_err(|_| {
        AppError::validation(format!(
            "YouTube video metadata canonical_url is invalid for video {video_id}"
        ))
    })?;
    let parsed_id = match parsed.kind {
        YoutubeUrlKind::Video { video_id }
        | YoutubeUrlKind::Short { video_id }
        | YoutubeUrlKind::Live { video_id } => video_id,
        YoutubeUrlKind::Playlist { .. } => {
            return Err(AppError::validation(format!(
                "YouTube video metadata canonical_url is not a video URL for video {video_id}"
            )));
        }
    };
    if parsed_id != video_id {
        return Err(AppError::validation(format!(
            "YouTube video metadata canonical_url id does not match video {video_id}"
        )));
    }
    Ok(())
}

fn validate_playlist_canonical_url(playlist_id: &str, canonical_url: &str) -> AppResult<()> {
    let parsed = parse_youtube_url(canonical_url).map_err(|_| {
        AppError::validation(format!(
            "YouTube playlist metadata canonical_url is invalid for playlist {playlist_id}"
        ))
    })?;
    match parsed.kind {
        YoutubeUrlKind::Playlist { playlist_id: parsed_id } if parsed_id == playlist_id => Ok(()),
        _ => Err(AppError::validation(format!(
            "YouTube playlist metadata canonical_url id does not match playlist {playlist_id}"
        ))),
    }
}

pub(crate) fn video_form_wire(form: &YoutubeVideoForm) -> &'static str {
    match form {
        YoutubeVideoForm::Regular => "regular",
        YoutubeVideoForm::Short => "short",
        YoutubeVideoForm::Live => "live",
    }
}

pub(crate) fn availability_status_wire(status: &YoutubeAvailabilityStatus) -> &'static str {
    match status {
        YoutubeAvailabilityStatus::Available => "available",
        YoutubeAvailabilityStatus::Upcoming => "upcoming",
        YoutubeAvailabilityStatus::LiveNow => "live_now",
        YoutubeAvailabilityStatus::LiveEndedTranscriptPending => {
            "live_ended_transcript_pending"
        }
        YoutubeAvailabilityStatus::NoCaptions => "no_captions",
        YoutubeAvailabilityStatus::PrivateOrAuthRequired => "private_or_auth_required",
        YoutubeAvailabilityStatus::MembersOnly => "members_only",
        YoutubeAvailabilityStatus::AgeRestricted => "age_restricted",
        YoutubeAvailabilityStatus::GeoBlocked => "geo_blocked",
        YoutubeAvailabilityStatus::Deleted => "deleted",
        YoutubeAvailabilityStatus::RemovedFromPlaylist => "removed_from_playlist",
        YoutubeAvailabilityStatus::UnavailableUnknown => "unavailable_unknown",
    }
}

fn raw_metadata_columns(raw: &Value) -> AppResult<(Option<i64>, Option<Vec<u8>>)> {
    if raw.is_null() {
        return Ok((None, None));
    }
    let sanitized = sanitize_raw_metadata(raw);
    let json = serde_json::to_vec(&sanitized)
        .map_err(|error| AppError::internal(error.to_string()))?;
    let compressed = compress_json_bytes(&json).map_err(AppError::internal)?;
    Ok((Some(YOUTUBE_RAW_METADATA_VERSION), Some(compressed)))
}

fn sanitize_raw_metadata(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| !is_secret_raw_key(key))
                .map(|(key, value)| (key.clone(), sanitize_raw_metadata(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(sanitize_raw_metadata).collect()),
        other => other.clone(),
    }
}

fn is_secret_raw_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "cookie"
            | "cookies"
            | "headers"
            | "http_headers"
            | "request_headers"
            | "command_args"
            | "argv"
            | "auth_diagnostics"
            | "logs"
            | "stderr"
            | "stdout"
    )
}

fn caption_language_override_from_raw(raw: &Value) -> Option<String> {
    raw.get("caption_language_override")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
```

- [x] **Step 4: Run typed helper tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::source_metadata::
```

Expected:

```text
test result: ok
```

- [x] **Step 5: Commit**

```powershell
git add src-tauri/src/youtube/source_metadata.rs
git commit -m "feat: add youtube typed metadata conversion"
```

## Task 3: Runner-Managed Migration 20 Backfill

**Files:**
- Create: `src-tauri/src/migrations/youtube_typed_source_metadata.rs`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/youtube/source_metadata.rs`
- Test: `src-tauri/src/migrations/youtube_typed_source_metadata.rs`

- [x] **Step 1: Add RED migration 20 tests**

Create `src-tauri/src/migrations/youtube_typed_source_metadata.rs` with constants and tests first:

```rust
use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(super) const YOUTUBE_TYPED_SOURCE_METADATA_VERSION: i64 = 20;
pub(super) const YOUTUBE_TYPED_SOURCE_METADATA_DESCRIPTION: &str =
    "add youtube typed source metadata";
pub(super) const YOUTUBE_TYPED_SOURCE_METADATA_SENTINEL_SQL: &str =
    include_str!("../../migrations/20.sql");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_json_bytes;
    use crate::migrations::build_migrations;
    use crate::youtube::dto::{
        YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
    };
    use serde_json::json;

    #[tokio::test]
    async fn migration_20_backfills_valid_video_and_playlist_metadata_and_clears_source_blobs() {
        let mut conn = memory_conn_with_history_through_19().await;
        insert_legacy_video_source(&mut conn, 101, "video01", "Video title").await;
        insert_legacy_playlist_source(&mut conn, 201, "PLdemo", "Playlist title").await;

        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("apply v20");

        let video: (String, Option<String>, String, Option<Vec<u8>>) = sqlx::query_as(
            "SELECT video_id, title, canonical_url, raw_metadata_zstd FROM youtube_video_sources WHERE source_id = 101",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load video typed row");
        assert_eq!(video.0, "video01");
        assert_eq!(video.1.as_deref(), Some("Video title"));
        assert_eq!(video.2, "https://www.youtube.com/watch?v=video01");
        assert!(video.3.is_some());

        let playlist: (String, Option<String>, String) = sqlx::query_as(
            "SELECT playlist_id, title, canonical_url FROM youtube_playlist_sources WHERE source_id = 201",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load playlist typed row");
        assert_eq!(playlist.0, "PLdemo");
        assert_eq!(playlist.1.as_deref(), Some("Playlist title"));
        assert_eq!(playlist.2, "https://www.youtube.com/playlist?list=PLdemo");

        let remaining_blobs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sources WHERE id IN (101, 201) AND metadata_zstd IS NOT NULL",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count source blobs");
        assert_eq!(remaining_blobs, 0);
    }

    #[tokio::test]
    async fn migration_20_skips_corrupt_wrong_shape_and_mismatched_blobs_without_failing() {
        let mut conn = memory_conn_with_history_through_19().await;
        insert_corrupt_youtube_source(&mut conn, 301, "video", "bad-video").await;
        insert_mismatched_video_source(&mut conn, 302, "source-video", "metadata-video").await;
        insert_wrong_shape_playlist_source(&mut conn, 303, "PLshape").await;

        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("apply v20");

        let typed_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM youtube_video_sources WHERE source_id IN (301, 302, 303)",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count video typed rows");
        assert_eq!(typed_count, 0);

        let inert_blob_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sources WHERE id IN (301, 302, 303) AND metadata_zstd IS NOT NULL",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count inert blobs");
        assert_eq!(inert_blob_count, 3);
    }

    #[tokio::test]
    async fn migration_20_is_idempotent_when_checksum_matches() {
        let mut conn = memory_conn_with_history_through_19().await;

        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("first v20");
        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("second v20");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 20")
                .fetch_one(&mut conn)
                .await
                .expect("count v20 history");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn migration_20_sentinel_checksum_is_recorded() {
        let mut conn = memory_conn_with_history_through_19().await;

        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("apply v20");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 20",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v20 history");

        assert_eq!(row.0, YOUTUBE_TYPED_SOURCE_METADATA_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_migration_20_checksum());
    }
}
```

Add helper functions in the same test module after the tests:

```rust
async fn memory_conn_with_history_through_19() -> SqliteConnection {
    let mut conn = SqliteConnection::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    for migration in build_migrations()
        .into_iter()
        .filter(|migration| migration.version < 19)
    {
        sqlx::raw_sql(migration.sql)
            .execute(&mut conn)
            .await
            .unwrap_or_else(|error| panic!("apply migration {}: {error}", migration.version));
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)",
        )
        .bind(migration.version)
        .bind(migration.description)
        .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
        .execute(&mut conn)
        .await
        .expect("record migration");
    }

    crate::migrations::source_identity_cleanup::apply_source_identity_cleanup_on_connection(
        &mut conn,
    )
    .await
    .expect("apply v19");

    conn
}

async fn insert_legacy_video_source(conn: &mut SqliteConnection, id: i64, video_id: &str, title: &str) {
    let metadata = YoutubeVideoMetadata {
        video_id: video_id.to_string(),
        canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
        title: Some(title.to_string()),
        channel_title: Some("Demo channel".to_string()),
        channel_id: Some("channel-1".to_string()),
        channel_handle: Some("@demo".to_string()),
        channel_url: Some("https://www.youtube.com/@demo".to_string()),
        author_display: Some("Demo channel".to_string()),
        published_at: Some("2026-05-17".to_string()),
        duration_seconds: Some(123),
        description: Some("Description".to_string()),
        thumbnail_url: None,
        tags: Vec::new(),
        chapters: Vec::new(),
        view_count: None,
        like_count: None,
        comment_count: None,
        category: None,
        video_form: YoutubeVideoForm::Regular,
        availability_status: YoutubeAvailabilityStatus::Available,
        raw_metadata_json: json!({ "id": video_id }),
    };
    insert_legacy_source_blob(conn, id, "video", video_id, title, &metadata).await;
}

async fn insert_legacy_playlist_source(conn: &mut SqliteConnection, id: i64, playlist_id: &str, title: &str) {
    let metadata = YoutubePlaylistMetadata {
        playlist_id: playlist_id.to_string(),
        canonical_url: format!("https://www.youtube.com/playlist?list={playlist_id}"),
        title: Some(title.to_string()),
        channel_title: Some("Demo channel".to_string()),
        channel_id: Some("channel-1".to_string()),
        channel_handle: Some("@demo".to_string()),
        channel_url: Some("https://www.youtube.com/@demo".to_string()),
        thumbnail_url: None,
        video_count: Some(0),
        items: Vec::new(),
        availability_status: YoutubeAvailabilityStatus::Available,
        raw_metadata_json: json!({ "id": playlist_id }),
    };
    insert_legacy_source_blob(conn, id, "playlist", playlist_id, title, &metadata).await;
}

async fn insert_legacy_source_blob<T: serde::Serialize>(
    conn: &mut SqliteConnection,
    id: i64,
    source_subtype: &str,
    external_id: &str,
    title: &str,
    metadata: &T,
) {
    let json = serde_json::to_vec(metadata).expect("serialize metadata");
    let blob = compress_json_bytes(&json).expect("compress metadata");
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (?, 'youtube', ?, ?, ?, ?, 1, 0, 1)",
    )
    .bind(id)
    .bind(source_subtype)
    .bind(external_id)
    .bind(title)
    .bind(blob)
    .execute(conn)
    .await
    .expect("insert legacy source");
}
```

- [x] **Step 2: Run migration 20 tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::youtube_typed_source_metadata::
```

Expected: fail because the module is not registered and migration functions are not implemented.

- [x] **Step 3: Register the migration module and runner call**

At the top of `src-tauri/src/migrations.rs`, add:

```rust
pub(crate) mod youtube_typed_source_metadata;
```

Keep `source_identity_cleanup` visible to the v20 test helper:

```rust
pub(crate) mod source_identity_cleanup;
```

In `patch_migrations`, after the v19 call, add:

```rust
source_identity_cleanup::apply_source_identity_cleanup_if_needed(&url).await?;
youtube_typed_source_metadata::apply_youtube_typed_source_metadata_if_needed(&url).await
```

In `apply_all_migrations_for_test_pool`, after the v19 call, add:

```rust
youtube_typed_source_metadata::apply_youtube_typed_source_metadata_on_connection(conn).await
```

- [x] **Step 4: Implement migration 20 recording, table creation, and backfill**

In `src-tauri/src/migrations/youtube_typed_source_metadata.rs`, implement the public runner functions:

```rust
pub(super) async fn apply_youtube_typed_source_metadata_if_needed(db_url: &str) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_youtube_typed_source_metadata_on_connection(&mut conn).await
}

pub(super) async fn apply_youtube_typed_source_metadata_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_previous_migration_recorded(conn).await?;
    if migration_20_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = async {
        crate::youtube::source_metadata::create_youtube_typed_source_tables(&mut *conn).await?;
        backfill_youtube_source_metadata(conn).await
    }
    .await;

    match result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            return Err(error);
        }
    }

    record_migration_success(
        conn,
        YOUTUBE_TYPED_SOURCE_METADATA_VERSION,
        YOUTUBE_TYPED_SOURCE_METADATA_DESCRIPTION,
        expected_migration_20_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await
}
```

Add local migration-history helpers equivalent to v19, with v20 names:

```rust
async fn ensure_previous_migration_recorded(conn: &mut SqliteConnection) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 19 AND success = 1",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::validation(
            "YouTube typed source metadata migration 20 requires migration 19",
        ));
    }
    Ok(())
}

async fn migration_20_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_migration_20_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(YOUTUBE_TYPED_SOURCE_METADATA_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 20 checksum does not match the runner-managed YouTube typed source metadata sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 20 is marked as failed in _sqlx_migrations",
        )),
    }
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    version: i64,
    description: &str,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, ?)",
    )
    .bind(version)
    .bind(description)
    .bind(checksum)
    .bind(execution_time)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn expected_migration_20_checksum() -> Vec<u8> {
    Sha384::digest(YOUTUBE_TYPED_SOURCE_METADATA_SENTINEL_SQL.as_bytes()).to_vec()
}
```

Add a backfill loop that never exposes raw payload bytes in errors:

```rust
#[derive(sqlx::FromRow)]
struct LegacyYoutubeSourceRow {
    id: i64,
    source_subtype: String,
    external_id: String,
    metadata_zstd: Option<Vec<u8>>,
}

async fn backfill_youtube_source_metadata(conn: &mut SqliteConnection) -> AppResult<()> {
    let rows: Vec<LegacyYoutubeSourceRow> = sqlx::query_as(
        r#"
        SELECT id, source_subtype, external_id, metadata_zstd
        FROM sources
        WHERE source_type = 'youtube'
          AND source_subtype IN ('video', 'playlist')
          AND metadata_zstd IS NOT NULL
        ORDER BY id
        "#,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    for row in rows {
        let Some(bytes) = row.metadata_zstd.as_deref() else {
            continue;
        };
        match row.source_subtype.as_str() {
            "video" => {
                if let Some(metadata) =
                    crate::youtube::source_metadata::decode_legacy_video_source_metadata(bytes)
                {
                    if metadata.video_id == row.external_id {
                        crate::youtube::source_metadata::insert_video_source_metadata_on_connection(
                            conn,
                            row.id,
                            &metadata,
                        )
                        .await?;
                        clear_source_blob(conn, row.id).await?;
                    }
                }
            }
            "playlist" => {
                if let Some(metadata) =
                    crate::youtube::source_metadata::decode_legacy_playlist_source_metadata(bytes)
                {
                    if metadata.playlist_id == row.external_id {
                        crate::youtube::source_metadata::insert_playlist_source_metadata_on_connection(
                            conn,
                            row.id,
                            &metadata,
                        )
                        .await?;
                        clear_source_blob(conn, row.id).await?;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn clear_source_blob(conn: &mut SqliteConnection, source_id: i64) -> AppResult<()> {
    sqlx::query("UPDATE sources SET metadata_zstd = NULL WHERE id = ?")
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(())
}
```

- [x] **Step 5: Add connection insert and legacy decode helpers**

In `src-tauri/src/youtube/source_metadata.rs`, add:

```rust
pub(crate) fn decode_legacy_video_source_metadata(bytes: &[u8]) -> Option<YoutubeVideoMetadata> {
    let json = crate::compression::decompress_bytes(bytes).ok()?;
    serde_json::from_slice(&json).ok()
}

pub(crate) fn decode_legacy_playlist_source_metadata(bytes: &[u8]) -> Option<YoutubePlaylistMetadata> {
    let json = crate::compression::decompress_bytes(bytes).ok()?;
    serde_json::from_slice(&json).ok()
}

pub(crate) async fn insert_video_source_metadata_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
    metadata: &YoutubeVideoMetadata,
) -> AppResult<()> {
    let columns = YoutubeVideoSourceColumns::try_from_metadata(metadata)?;
    insert_video_source_columns(conn, source_id, &columns).await
}

pub(crate) async fn insert_playlist_source_metadata_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<()> {
    let columns = YoutubePlaylistSourceColumns::try_from_metadata(metadata)?;
    insert_playlist_source_columns(conn, source_id, &columns).await
}
```

Add private `insert_video_source_columns` and `insert_playlist_source_columns` SQL helpers with `ON CONFLICT(source_id) DO UPDATE SET` for every typed field plus `updated_at = strftime('%s','now')`. Bind `tags_json`, `chapters_json`, `raw_metadata_version`, and `raw_metadata_zstd` from the columns structs.

- [x] **Step 6: Run migration 20 tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::youtube_typed_source_metadata::
```

Expected:

```text
test result: ok
```

- [x] **Step 7: Run full migration tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::
```

Expected:

```text
test result: ok
```

- [x] **Step 8: Commit**

```powershell
git add src-tauri/src/migrations.rs src-tauri/src/migrations/youtube_typed_source_metadata.rs src-tauri/src/youtube/source_metadata.rs
git commit -m "feat: backfill youtube typed source metadata"
```

## Task 4: YouTube Source Upserts Write Typed Rows And Clear Source Blobs

**Files:**
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/youtube/source_metadata.rs`
- Test: `src-tauri/src/sources/store.rs`

- [x] **Step 1: Add RED write-path tests**

In `src-tauri/src/sources/store.rs`, inside `#[cfg(test)] mod tests`, add:

```rust
#[tokio::test]
async fn upsert_youtube_video_source_writes_typed_row_and_null_source_metadata() {
    let pool = memory_pool_with_sources().await;
    create_youtube_typed_source_tables(&pool).await;
    create_youtube_unique_indexes(&pool).await;
    let mut tx = pool.begin().await.expect("begin tx");

    let source_id = upsert_youtube_video_source(&mut tx, &youtube_video_metadata())
        .await
        .expect("upsert youtube video");
    tx.commit().await.expect("commit");

    let source_metadata: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT metadata_zstd FROM sources WHERE id = ?")
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .expect("load source metadata");
    assert_eq!(source_metadata, None);

    let typed: (String, Option<String>, String, String, Option<Vec<u8>>) = sqlx::query_as(
        "SELECT video_id, title, canonical_url, availability_status, raw_metadata_zstd FROM youtube_video_sources WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_one(&pool)
    .await
    .expect("load typed video source");
    assert_eq!(typed.0, "dQw4w9WgXcQ");
    assert_eq!(typed.1.as_deref(), Some("Demo video"));
    assert_eq!(typed.2, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    assert_eq!(typed.3, "available");
    assert!(typed.4.is_some());
}

#[tokio::test]
async fn upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata() {
    let pool = memory_pool_with_sources().await;
    create_youtube_typed_source_tables(&pool).await;
    create_youtube_unique_indexes(&pool).await;
    let mut tx = pool.begin().await.expect("begin tx");

    let source_id = upsert_youtube_playlist_source(&mut tx, &youtube_playlist_metadata())
        .await
        .expect("upsert youtube playlist");
    tx.commit().await.expect("commit");

    let source_metadata: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT metadata_zstd FROM sources WHERE id = ?")
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .expect("load source metadata");
    assert_eq!(source_metadata, None);

    let typed: (String, Option<String>, String, i64) = sqlx::query_as(
        "SELECT playlist_id, title, canonical_url, video_count FROM youtube_playlist_sources WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_one(&pool)
    .await
    .expect("load typed playlist source");
    assert_eq!(typed.0, "PLdemo");
    assert_eq!(typed.1.as_deref(), Some("Demo playlist"));
    assert_eq!(typed.2, "https://www.youtube.com/playlist?list=PLdemo");
    assert_eq!(typed.3, 0);
}

#[tokio::test]
async fn upsert_youtube_video_source_conflict_clears_existing_legacy_blob() {
    let pool = memory_pool_with_sources().await;
    create_youtube_typed_source_tables(&pool).await;
    create_youtube_unique_indexes(&pool).await;
    let legacy_blob = crate::compression::compress_json_bytes(br#"{"legacy":true}"#)
        .expect("compress legacy");
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (77, 'youtube', 'video', 'dQw4w9WgXcQ', 'Old', ?, 1, 0, 1)",
    )
    .bind(legacy_blob)
    .execute(&pool)
    .await
    .expect("insert legacy source");
    let mut tx = pool.begin().await.expect("begin tx");

    let source_id = upsert_youtube_video_source(&mut tx, &youtube_video_metadata())
        .await
        .expect("upsert existing youtube video");
    tx.commit().await.expect("commit");

    assert_eq!(source_id, 77);
    let source_metadata: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT metadata_zstd FROM sources WHERE id = 77")
            .fetch_one(&pool)
            .await
            .expect("load source metadata");
    assert_eq!(source_metadata, None);
}

#[tokio::test]
async fn upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row() {
    let pool = memory_pool_with_sources().await;
    create_youtube_typed_source_tables(&pool).await;
    create_youtube_unique_indexes(&pool).await;
    let mut metadata = youtube_video_metadata();
    metadata.canonical_url = "https://example.com/watch?v=dQw4w9WgXcQ".to_string();
    let mut tx = pool.begin().await.expect("begin tx");

    let error = upsert_youtube_video_source(&mut tx, &metadata)
        .await
        .expect_err("invalid metadata rejected");
    tx.rollback().await.expect("rollback");

    assert_eq!(error.kind, AppErrorKind::Validation);
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE external_id = 'dQw4w9WgXcQ'")
            .fetch_one(&pool)
            .await
            .expect("count source rows");
    assert_eq!(count, 0);
}
```

Add test imports:

```rust
use crate::sources::test_support::create_youtube_typed_source_tables;
use crate::error::AppErrorKind;
```

Add helper:

```rust
async fn create_youtube_unique_indexes(pool: &sqlx::SqlitePool) {
    sqlx::query(
        "CREATE UNIQUE INDEX idx_sources_unique_youtube_video ON sources(source_type, source_subtype, external_id) WHERE source_type = 'youtube' AND source_subtype = 'video'",
    )
    .execute(pool)
    .await
    .expect("create video index");
    sqlx::query(
        "CREATE UNIQUE INDEX idx_sources_unique_youtube_playlist ON sources(source_type, source_subtype, external_id) WHERE source_type = 'youtube' AND source_subtype = 'playlist'",
    )
    .execute(pool)
    .await
    .expect("create playlist index");
}
```

- [x] **Step 2: Run source-store tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::store::upsert_youtube
```

Expected: fail because upserts still write source blobs and do not create typed rows.

- [x] **Step 3: Add transaction upsert helpers**

In `src-tauri/src/youtube/source_metadata.rs`, add transaction wrappers:

```rust
pub(crate) async fn upsert_video_source_metadata(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
    metadata: &YoutubeVideoMetadata,
) -> AppResult<()> {
    let columns = YoutubeVideoSourceColumns::try_from_metadata(metadata)?;
    insert_video_source_columns(&mut **tx, source_id, &columns).await
}

pub(crate) async fn upsert_playlist_source_metadata(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<()> {
    let columns = YoutubePlaylistSourceColumns::try_from_metadata(metadata)?;
    insert_playlist_source_columns(&mut **tx, source_id, &columns).await
}
```

- [x] **Step 4: Update video source upsert**

In `src-tauri/src/sources/store.rs`, remove `use crate::compression::compress_json_bytes;` if it is only used by YouTube source upserts, and add:

```rust
use crate::youtube::source_metadata::{
    upsert_playlist_source_metadata, upsert_video_source_metadata,
};
```

Replace `upsert_youtube_video_source` with:

```rust
pub(crate) async fn upsert_youtube_video_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    metadata: &YoutubeVideoMetadata,
) -> AppResult<i64> {
    let _validated = crate::youtube::source_metadata::YoutubeVideoSourceColumns::try_from_metadata(metadata)?;
    let now = now_secs();

    let source_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO sources (
            source_type,
            source_subtype,
            account_id,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            created_at
        )
        VALUES ('youtube', 'video', NULL, ?, ?, NULL, 1, 0, ?)
        ON CONFLICT(source_type, source_subtype, external_id)
        WHERE source_type = 'youtube' AND source_subtype = 'video'
        DO UPDATE SET
            title = excluded.title,
            metadata_zstd = NULL,
            is_active = 1
        RETURNING id
        "#,
    )
    .bind(&metadata.video_id)
    .bind(&metadata.title)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    upsert_video_source_metadata(tx, source_id, metadata).await?;
    Ok(source_id)
}
```

- [x] **Step 5: Update playlist source upsert**

Replace `upsert_youtube_playlist_source` with:

```rust
pub(crate) async fn upsert_youtube_playlist_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<i64> {
    let _validated =
        crate::youtube::source_metadata::YoutubePlaylistSourceColumns::try_from_metadata(metadata)?;
    let now = now_secs();

    let source_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO sources (
            source_type,
            source_subtype,
            account_id,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            created_at
        )
        VALUES ('youtube', 'playlist', NULL, ?, ?, NULL, 1, 0, ?)
        ON CONFLICT(source_type, source_subtype, external_id)
        WHERE source_type = 'youtube' AND source_subtype = 'playlist'
        DO UPDATE SET
            title = excluded.title,
            metadata_zstd = NULL,
            is_active = 1
        RETURNING id
        "#,
    )
    .bind(&metadata.playlist_id)
    .bind(&metadata.title)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    upsert_playlist_source_metadata(tx, source_id, metadata).await?;
    Ok(source_id)
}
```

- [x] **Step 6: Update legacy NOT NULL tests**

In `legacy_not_null_telegram_kind_pool()`, call typed table creation after creating the YouTube unique indexes:

```rust
create_youtube_typed_source_tables(&pool).await;
```

- [x] **Step 7: Run source-store tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::store::upsert_youtube sources::store::telegram_source_upsert
```

Expected:

```text
test result: ok
```

- [x] **Step 8: Commit**

```powershell
git add src-tauri/src/sources/store.rs src-tauri/src/youtube/source_metadata.rs
git commit -m "feat: write youtube typed source metadata"
```

## Task 5: Detail And Listing Read Typed Rows

**Files:**
- Modify: `src-tauri/src/youtube/detail.rs`
- Modify: `src-tauri/src/youtube/source_metadata.rs`
- Test: `src-tauri/src/youtube/detail.rs`

- [x] **Step 1: Add RED detail/listing tests**

In `src-tauri/src/youtube/detail.rs`, update the test fixture helpers so they call `create_youtube_typed_source_tables(&pool).await` and insert typed rows for valid YouTube source metadata.

Add these tests:

```rust
#[tokio::test]
async fn summaries_use_typed_video_metadata_with_corrupt_source_blob() {
    let pool = detail_pool().await;
    let source_id = insert_youtube_video_source(&pool, "video01", "Generic title", "available").await;
    sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = ?")
        .bind(source_id)
        .execute(&pool)
        .await
        .expect("corrupt source blob");

    let summaries = list_youtube_source_summaries_from_pool(&pool, vec![source_id])
        .await
        .expect("list summaries");

    assert_eq!(summaries[0].title.as_deref(), Some("Typed video title"));
    assert_eq!(
        summaries[0].canonical_url.as_deref(),
        Some("https://www.youtube.com/watch?v=video01")
    );
    assert_eq!(summaries[0].availability_status.as_deref(), Some("available"));
}

#[tokio::test]
async fn source_summary_missing_typed_metadata_uses_generic_title_without_blob_decode() {
    let pool = detail_pool().await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (901, 'youtube', 'video', 'missing01', 'Generic fallback', x'00', 1, 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert source");

    let summaries = list_youtube_source_summaries_from_pool(&pool, vec![901])
        .await
        .expect("list summaries");

    assert_eq!(summaries[0].title.as_deref(), Some("Generic fallback"));
    assert_eq!(
        summaries[0].canonical_url.as_deref(),
        Some("https://www.youtube.com/watch?v=missing01")
    );
    assert_eq!(summaries[0].availability_status, None);
}

#[tokio::test]
async fn video_detail_missing_typed_metadata_returns_controlled_error() {
    let pool = detail_pool().await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (902, 'youtube', 'video', 'missing02', 'Generic fallback', x'00', 1, 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert source");

    let error = get_youtube_video_detail_from_pool(&pool, 902)
        .await
        .expect_err("missing typed metadata rejected");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert!(error.to_string().contains("typed YouTube video metadata"));
    assert!(!error.to_string().contains("metadata_zstd"));
}

#[tokio::test]
async fn playlist_detail_uses_typed_linked_video_metadata_with_corrupt_source_blob() {
    let pool = detail_pool().await;
    let playlist_id = insert_youtube_playlist_source(&pool, "PLdemo", "Generic playlist").await;
    let video_id = insert_youtube_video_source(&pool, "video02", "Generic linked video", "available").await;
    sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = ?")
        .bind(video_id)
        .execute(&pool)
        .await
        .expect("corrupt source blob");
    insert_playlist_item(&pool, playlist_id, Some(video_id), "video02", "Snapshot title").await;

    let detail = get_youtube_playlist_detail_from_pool(&pool, playlist_id)
        .await
        .expect("playlist detail");

    assert_eq!(detail.items[0].title.as_deref(), Some("Typed video title"));
    assert_eq!(
        detail.items[0].canonical_url.as_deref(),
        Some("https://www.youtube.com/watch?v=video02")
    );
}
```

- [x] **Step 2: Run detail tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::detail::
```

Expected: fail because detail still decodes `sources.metadata_zstd`.

- [x] **Step 3: Add typed metadata runtime row loaders**

In `src-tauri/src/youtube/source_metadata.rs`, add runtime structs:

```rust
#[derive(Clone, Debug)]
pub(crate) struct YoutubeVideoSourceMetadata {
    pub(crate) source_id: i64,
    pub(crate) video_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) author_display: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) description: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) video_form: String,
    pub(crate) availability_status: String,
    pub(crate) caption_language_override: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct YoutubePlaylistSourceMetadata {
    pub(crate) source_id: i64,
    pub(crate) playlist_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) video_count: Option<i64>,
    pub(crate) availability_status: String,
}
```

Add loaders:

```rust
pub(crate) async fn load_video_source_metadata_map(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
) -> AppResult<std::collections::HashMap<i64, YoutubeVideoSourceMetadata>> {
    if source_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let mut query = sqlx::QueryBuilder::new(
        r#"
        SELECT
            s.id AS source_id,
            s.source_subtype,
            s.external_id,
            yvs.video_id,
            yvs.canonical_url,
            yvs.title,
            yvs.channel_title,
            yvs.channel_handle,
            yvs.author_display,
            yvs.published_at,
        yvs.duration_seconds,
        yvs.description,
        yvs.thumbnail_url,
        yvs.video_form,
        yvs.availability_status,
        yvs.caption_language_override
        FROM sources s
        JOIN youtube_video_sources yvs ON yvs.source_id = s.id
        WHERE s.source_type = 'youtube'
          AND s.source_subtype = 'video'
          AND s.id IN (
        "#,
    );
    push_i64_list_for_source_metadata(&mut query, source_ids);
    query.push(")");
    let rows = query.build().fetch_all(pool).await.map_err(AppError::database)?;
    video_metadata_rows_to_map(rows)
}
```

Add the equivalent `load_playlist_source_metadata_map`. In both row parsers, treat rows as invalid and omit them when source subtype/id/canonical URL/availability checks fail. Invalid JSON checks for `tags_json` and `chapters_json` should be done in SQL row parsers when those columns are loaded in later analysis code.

- [x] **Step 4: Update summary loading**

In `src-tauri/src/youtube/detail.rs`:

- Remove `use crate::compression::decompress_bytes;`.
- Replace `use super::dto::{YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoMetadata};` with:

```rust
use crate::youtube::source_metadata::{
    load_playlist_source_metadata_map, load_video_source_metadata_map,
    YoutubePlaylistSourceMetadata, YoutubeVideoSourceMetadata,
};
```

- Remove `metadata_zstd` from `SourceSummaryRow`.
- Change `load_source_rows` to select only:

```sql
SELECT id, source_subtype, external_id, title
FROM sources
WHERE source_type = 'youtube' AND id IN (
```

- In `list_youtube_source_summaries_from_pool`, load typed maps:

```rust
let video_metadata = load_video_source_metadata_map(pool, &source_ids_from_rows).await?;
let playlist_metadata = load_playlist_source_metadata_map(pool, &source_ids_from_rows).await?;
```

- Change `summary_from_row` signature to receive:

```rust
video_metadata: Option<&YoutubeVideoSourceMetadata>,
playlist_metadata: Option<&YoutubePlaylistSourceMetadata>,
```

- Build playlist summaries from `playlist_metadata` when present, otherwise use `sources.title`, generated playlist URL, and `None` for provider fields.
- Build video summaries from `video_metadata` when present, otherwise use `sources.title`, generated video URL, and `None` for provider fields.

- [x] **Step 5: Update detail missing-metadata behavior**

In `get_youtube_video_detail_from_pool`, after checking subtype, enforce typed metadata:

```rust
let typed = load_video_source_metadata_map(pool, &[source_id]).await?;
if !typed.contains_key(&source_id) {
    return Err(AppError::validation(format!(
        "Source {source_id} has missing or invalid typed YouTube video metadata"
    )));
}
```

In `get_youtube_playlist_detail_from_pool`, enforce playlist typed metadata the same way:

```rust
let typed = load_playlist_source_metadata_map(pool, &[source_id]).await?;
if !typed.contains_key(&source_id) {
    return Err(AppError::validation(format!(
        "Source {source_id} has missing or invalid typed YouTube playlist metadata"
    )));
}
```

- [x] **Step 6: Update playlist item detail query**

Replace `sources.metadata_zstd AS video_metadata_zstd` with a left join to `youtube_video_sources`:

```sql
LEFT JOIN youtube_video_sources yvs ON yvs.source_id = youtube_playlist_items.video_source_id
```

Select typed fields needed for item display:

```sql
yvs.title AS typed_video_title,
yvs.canonical_url AS typed_video_canonical_url,
yvs.thumbnail_url AS typed_video_thumbnail_url,
yvs.duration_seconds AS typed_video_duration_seconds,
yvs.published_at AS typed_video_published_at
```

Update `PlaylistItemRow` and item mapping so typed values win, then generic `sources.title`, then playlist item snapshots. Remove `decode_youtube_metadata` and the local `availability_status_wire` helper if unused.

- [x] **Step 7: Run detail tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::detail::
```

Expected:

```text
test result: ok
```

- [x] **Step 8: Commit**

```powershell
git add src-tauri/src/youtube/detail.rs src-tauri/src/youtube/source_metadata.rs
git commit -m "feat: read youtube detail from typed metadata"
```

## Task 6: Analysis Corpus Reads Typed YouTube Metadata

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/youtube/source_metadata.rs`
- Test: `src-tauri/src/analysis/corpus.rs`

- [x] **Step 1: Add RED analysis tests**

In `src-tauri/src/analysis/corpus.rs`, update YouTube test schemas to create `youtube_video_sources`. Add:

```rust
#[tokio::test]
async fn youtube_description_rows_use_typed_metadata_with_corrupt_source_blob() {
    let pool = youtube_corpus_pool().await;
    insert_youtube_video_source_with_typed_metadata(
        &pool,
        401,
        "video401",
        "Typed title",
        Some("Typed description"),
        Some("2026-05-17"),
    )
    .await;
    sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = 401")
        .execute(&pool)
        .await
        .expect("corrupt source blob");

    let request = CorpusLoadRequest {
        source_ids: vec![401],
        period_from: 1,
        period_to: i64::MAX,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
    };
    let messages = load_youtube_description_messages(&pool, &request)
        .await
        .expect("load descriptions");

    assert_eq!(messages.len(), 1);
    assert!(messages[0].content.contains("Typed description"));
    assert!(messages[0].content.contains("URL: https://www.youtube.com/watch?v=video401"));
}

#[tokio::test]
async fn youtube_description_missing_typed_metadata_skips_without_decoding_source_blob() {
    let pool = youtube_corpus_pool().await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (402, 'youtube', 'video', 'video402', 'Generic title', x'00', 1, 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert source");

    let request = CorpusLoadRequest {
        source_ids: vec![402],
        period_from: 1,
        period_to: i64::MAX,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
    };
    let messages = load_youtube_description_messages(&pool, &request)
        .await
        .expect("load descriptions");

    assert!(messages.is_empty());
}

#[tokio::test]
async fn youtube_transcript_segment_evidence_uses_typed_source_context() {
    let pool = youtube_corpus_pool().await;
    insert_youtube_video_source_with_typed_metadata(
        &pool,
        403,
        "video403",
        "Typed title",
        None,
        Some("2026-05-17"),
    )
    .await;
    insert_youtube_transcript_segment(&pool, 403, 9001, 12_000, "segment text").await;
    sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = 403")
        .execute(&pool)
        .await
        .expect("corrupt source blob");

    let request = CorpusLoadRequest {
        source_ids: vec![403],
        period_from: 1,
        period_to: i64::MAX,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptOnly,
    };
    let messages = load_youtube_transcript_segment_messages(&pool, &request)
        .await
        .expect("load transcript segments");

    let metadata_json = decode_message_metadata_for_test(&messages[0]);
    assert_eq!(metadata_json["video_id"], "video403");
    assert_eq!(
        metadata_json["canonical_url"],
        "https://www.youtube.com/watch?v=video403"
    );
    assert_eq!(metadata_json["title"], "Typed title");
    assert_eq!(metadata_json["segment_start_ms"], 12_000);
}
```

- [x] **Step 2: Run analysis tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::youtube_
```

Expected: fail because corpus still decodes `sources.metadata_zstd`.

- [x] **Step 3: Add description metadata loader**

In `src-tauri/src/youtube/source_metadata.rs`, add:

```rust
#[derive(Clone, Debug)]
pub(crate) struct YoutubeVideoDescriptionMetadata {
    pub(crate) source_id: i64,
    pub(crate) video_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) description: Option<String>,
}

pub(crate) async fn load_video_description_metadata(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
) -> AppResult<Vec<YoutubeVideoDescriptionMetadata>> {
    if source_ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = sqlx::QueryBuilder::new(
        r#"
        SELECT
            s.id AS source_id,
            s.external_id,
            yvs.video_id,
            yvs.canonical_url,
            yvs.title,
            yvs.channel_title,
            yvs.channel_handle,
            yvs.published_at,
            yvs.description,
            yvs.tags_json,
            yvs.chapters_json,
            yvs.availability_status
        FROM sources s
        JOIN youtube_video_sources yvs ON yvs.source_id = s.id
        WHERE s.source_type = 'youtube'
          AND s.source_subtype = 'video'
          AND s.id IN (
        "#,
    );
    push_i64_list_for_source_metadata(&mut query, source_ids);
    query.push(") ORDER BY s.id ASC");
    let rows = query.build().fetch_all(pool).await.map_err(AppError::database)?;
    rows.into_iter()
        .filter_map(valid_description_metadata_from_row)
        .collect::<AppResult<Vec<_>>>()
}
```

The `valid_description_metadata_from_row` helper must reject rows whose provider id does not equal `sources.external_id`, whose canonical URL is invalid, whose availability wire value is unsupported, or whose `tags_json`/`chapters_json` do not parse as arrays.

- [x] **Step 4: Update transcript segment query and metadata builder**

In `load_youtube_transcript_segment_messages`, replace `sources.metadata_zstd AS source_metadata_zstd` with typed fields:

```sql
yvs.video_id AS typed_video_id,
yvs.canonical_url AS typed_canonical_url,
yvs.title AS typed_title,
yvs.channel_title AS typed_channel_title,
yvs.channel_handle AS typed_channel_handle
```

Add the join:

```sql
LEFT JOIN youtube_video_sources yvs ON yvs.source_id = sources.id
```

Update `YoutubeTranscriptSegmentRow` fields:

```rust
typed_video_id: Option<String>,
typed_canonical_url: Option<String>,
typed_title: Option<String>,
typed_channel_title: Option<String>,
typed_channel_handle: Option<String>,
```

Replace `youtube_segment_metadata_zstd` so it never decodes source blobs:

```rust
fn youtube_segment_metadata_zstd(row: &YoutubeTranscriptSegmentRow) -> Result<Vec<u8>, String> {
    let video_id = row
        .typed_video_id
        .as_deref()
        .unwrap_or(row.source_external_id.as_str());
    let canonical_url = row
        .typed_canonical_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={video_id}"));
    let title = row.typed_title.as_deref().or(row.source_title.as_deref());
    let metadata = serde_json::json!({
        "video_id": video_id,
        "canonical_url": canonical_url,
        "title": title,
        "channel_title": &row.typed_channel_title,
        "channel_handle": &row.typed_channel_handle,
        "caption_language": &row.caption_language,
        "caption_track_kind": &row.caption_track_kind,
        "segment_start_ms": row.start_ms,
        "segment_end_ms": row.end_ms,
        "item_kind": "youtube_transcript",
    });
    let json = serde_json::to_vec(&metadata).map_err(|e| e.to_string())?;
    compress_json_bytes(&json)
}
```

- [x] **Step 5: Update description rows**

Replace `YoutubeSourceMetadataRow` and `load_youtube_description_messages` source-blob query with `load_video_description_metadata(pool, &request.source_ids)`. Build messages from typed metadata:

```rust
for metadata in crate::youtube::source_metadata::load_video_description_metadata(
    pool,
    &request.source_ids,
)
.await
.map_err(|error| error.to_string())?
{
    let Some(description) = metadata.description.as_deref().map(str::trim) else {
        continue;
    };
    if description.is_empty() {
        continue;
    }
    let Some(published_at) = metadata.published_at.as_deref().and_then(ymd_to_unix_midnight) else {
        continue;
    };
    if published_at < request.period_from || published_at > request.period_to {
        continue;
    }

    let title = metadata.title.clone().unwrap_or_else(|| metadata.video_id.clone());
    let channel = metadata
        .channel_title
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let content = format!(
        "YouTube video description\nTitle: {title}\nChannel: {channel}\nURL: {url}\n\n{description}",
        url = metadata.canonical_url,
    );

    messages.push(CorpusMessage {
        item_id: 0,
        source_id: metadata.source_id,
        external_id: format!("description:{}", metadata.video_id),
        published_at,
        author: metadata.channel_title.clone(),
        content,
        r#ref: format!("s{}-i0", metadata.source_id),
        item_kind: Some("youtube_description".to_string()),
        source_type: Some("youtube".to_string()),
        source_subtype: Some("video".to_string()),
        metadata_zstd: Some(youtube_description_metadata_zstd(&metadata)?),
    });
}
```

Change `youtube_description_metadata_zstd` to accept `&YoutubeVideoDescriptionMetadata`.

- [x] **Step 6: Remove source blob decoder from analysis corpus**

Delete `decode_youtube_video_metadata` and remove the top-level `use crate::youtube::dto::YoutubeVideoMetadata;` unless tests still need DTO fixtures. Test fixtures that generate old source blobs should be replaced with typed table inserts for normal analysis tests.

- [x] **Step 7: Run analysis tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::
```

Expected:

```text
test result: ok
```

- [x] **Step 8: Commit**

```powershell
git add src-tauri/src/analysis/corpus.rs src-tauri/src/youtube/source_metadata.rs
git commit -m "feat: read youtube analysis corpus from typed metadata"
```

## Task 7: Jobs Use Typed Metadata And Reload After Explicit Refresh

**Files:**
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/youtube/source_metadata.rs`
- Test: `src-tauri/src/youtube/jobs.rs`

- [ ] **Step 1: Add RED job helper tests**

In `src-tauri/src/youtube/jobs.rs`, inside tests, add:

```rust
#[tokio::test]
async fn jobs_reload_missing_typed_video_metadata_after_refresh_callback() {
    let pool = crate::sources::test_support::memory_pool_with_sources().await;
    crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (701, 'youtube', 'video', 'jobvideo', 'Job video', x'00', 1, 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert source");

    let refreshed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let refreshed_for_callback = refreshed.clone();
    let metadata = super::load_video_metadata_or_refresh(&pool, 701, || {
        let pool = pool.clone();
        async move {
            refreshed_for_callback.store(true, std::sync::atomic::Ordering::SeqCst);
            insert_typed_video_metadata_for_job_test(&pool, 701, "jobvideo").await;
            Ok(())
        }
    })
    .await
    .expect("load refreshed metadata");

    assert!(refreshed.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(metadata.video_id, "jobvideo");
    assert_eq!(metadata.canonical_url, "https://www.youtube.com/watch?v=jobvideo");
}

#[tokio::test]
async fn jobs_missing_typed_video_metadata_errors_after_failed_refresh() {
    let pool = crate::sources::test_support::memory_pool_with_sources().await;
    crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (702, 'youtube', 'video', 'jobmissing', 'Job missing', x'00', 1, 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert source");

    let error = super::load_video_metadata_or_refresh(&pool, 702, || async { Ok(()) })
        .await
        .expect_err("missing typed metadata rejected");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert!(error.to_string().contains("typed YouTube video metadata"));
    assert!(!error.to_string().contains("metadata_zstd"));
}

#[test]
fn source_jobs_no_longer_decode_source_metadata_blobs() {
    let source = std::fs::read_to_string("src/youtube/jobs.rs").expect("read jobs.rs");
    assert!(!source.contains("decode_youtube_metadata"));
    assert!(!source.contains("decompress_bytes"));
}
```

Add a helper in the jobs test module:

```rust
async fn insert_typed_video_metadata_for_job_test(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    video_id: &str,
) {
    let metadata = crate::youtube::dto::YoutubeVideoMetadata {
        video_id: video_id.to_string(),
        canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
        title: Some("Job typed video".to_string()),
        channel_title: Some("Job channel".to_string()),
        channel_id: None,
        channel_handle: None,
        channel_url: None,
        author_display: Some("Job channel".to_string()),
        published_at: Some("2026-05-17".to_string()),
        duration_seconds: Some(30),
        description: None,
        thumbnail_url: None,
        tags: Vec::new(),
        chapters: Vec::new(),
        view_count: None,
        like_count: None,
        comment_count: None,
        category: None,
        video_form: crate::youtube::dto::YoutubeVideoForm::Regular,
        availability_status: crate::youtube::dto::YoutubeAvailabilityStatus::Available,
        raw_metadata_json: serde_json::json!({ "id": video_id, "caption_language_override": "en" }),
    };
    crate::youtube::source_metadata::insert_video_source_metadata_for_pool_test(
        pool,
        source_id,
        &metadata,
    )
    .await;
}
```

- [ ] **Step 2: Run job tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::jobs::
```

Expected: fail because jobs still decode `sources.metadata_zstd` and reload helper does not exist.

- [ ] **Step 3: Add a test-only pool insert helper**

In `src-tauri/src/youtube/source_metadata.rs`, add:

```rust
#[cfg(test)]
pub(crate) async fn insert_video_source_metadata_for_pool_test(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    metadata: &YoutubeVideoMetadata,
) {
    let columns = YoutubeVideoSourceColumns::try_from_metadata(metadata)
        .expect("valid video metadata");
    insert_video_source_columns(pool, source_id, &columns)
        .await
        .expect("insert typed video metadata");
}
```

- [ ] **Step 4: Replace jobs metadata decode with typed loaders**

In `src-tauri/src/youtube/jobs.rs`:

- Remove `use crate::compression::decompress_bytes;`.
- Replace `use super::dto::{YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata};` with:

```rust
use super::dto::{YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata};
use super::source_metadata::{
    load_playlist_source_metadata_map, load_video_source_metadata_map, YoutubeVideoSourceMetadata,
};
```

Delete:

```rust
fn decode_video_metadata(...)
fn decode_playlist_metadata(...)
fn decode_youtube_metadata(...)
fn caption_language_override(metadata: &YoutubeVideoMetadata) -> Option<String>
```

Add:

```rust
async fn load_video_metadata_or_refresh<F, Fut>(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    refresh: F,
) -> AppResult<YoutubeVideoSourceMetadata>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<()>>,
{
    if let Some(metadata) = load_video_source_metadata_map(pool, &[source_id])
        .await?
        .remove(&source_id)
    {
        return Ok(metadata);
    }

    refresh().await?;

    load_video_source_metadata_map(pool, &[source_id])
        .await?
        .remove(&source_id)
        .ok_or_else(|| {
            AppError::validation(format!(
                "Source {source_id} has missing or invalid typed YouTube video metadata"
            ))
        })
}
```

Add playlist equivalent if `sync_youtube_metadata` needs typed playlist canonical URL:

```rust
async fn load_playlist_metadata_for_refresh(
    pool: &sqlx::SqlitePool,
    source_id: i64,
) -> AppResult<Option<crate::youtube::source_metadata::YoutubePlaylistSourceMetadata>> {
    Ok(load_playlist_source_metadata_map(pool, &[source_id])
        .await?
        .remove(&source_id))
}
```

- [ ] **Step 5: Update metadata refresh canonical URL selection**

In `sync_youtube_metadata`, replace source blob decode usage:

```rust
let payload = match source.source_subtype.as_deref() {
    Some("playlist") => {
        let typed = load_playlist_metadata_for_refresh(&pool, source_id).await?;
        let canonical_url = typed
            .as_ref()
            .map(|metadata| metadata.canonical_url.clone())
            .unwrap_or_else(|| playlist_canonical_url(&source));
        MetadataSyncPayload::Playlist(fetch_playlist_metadata(&canonical_url, cookies).await?)
    }
    _ => {
        let typed = load_video_source_metadata_map(&pool, &[source_id])
            .await?
            .remove(&source_id);
        let canonical_url = typed
            .as_ref()
            .map(|metadata| metadata.canonical_url.clone())
            .unwrap_or_else(|| video_canonical_url(&source));
        let video_form = typed
            .as_ref()
            .and_then(|metadata| metadata.video_form_for_provider())
            .unwrap_or(YoutubeVideoForm::Regular);
        MetadataSyncPayload::Video(fetch_video_metadata(&canonical_url, video_form, cookies).await?)
    }
};
```

Add `YoutubeVideoSourceMetadata::video_form_for_provider()` in `source_metadata.rs`:

```rust
pub(crate) fn video_form_for_provider(&self) -> Option<YoutubeVideoForm> {
    match self.video_form.as_str() {
        "regular" => Some(YoutubeVideoForm::Regular),
        "short" => Some(YoutubeVideoForm::Short),
        "live" => Some(YoutubeVideoForm::Live),
        _ => None,
    }
}
```

- [ ] **Step 6: Update transcript/comment sync dependent metadata**

In `sync_youtube_transcript`, replace the missing-metadata branch and decode with:

```rust
let metadata = load_video_metadata_or_refresh(&pool, source_id, || {
    sync_youtube_metadata(handle, source_id)
})
.await?;
```

Use a DTO adapter before calling caption/comment functions:

```rust
let metadata_for_provider = metadata.to_provider_metadata();
let transcript = fetch_transcript_for_video(
    &metadata_for_provider,
    Some(preferred_language.as_str()),
    metadata.caption_language_override.as_deref(),
    cookies,
)
.await?;
```

Add `YoutubeVideoSourceMetadata::to_provider_metadata()` in `source_metadata.rs`; populate fields that exist in typed columns and use `serde_json::Value::Null` for `raw_metadata_json` because normal runtime must not decode raw payload.

Apply the same adapter in `sync_youtube_comments`.

- [ ] **Step 7: Run job tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::jobs::
```

Expected:

```text
test result: ok
```

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/youtube/jobs.rs src-tauri/src/youtube/source_metadata.rs
git commit -m "feat: load youtube jobs from typed metadata"
```

## Task 8: Playlist Integration, Docs, And Containment Verification

**Files:**
- Modify: `src-tauri/src/youtube/playlist.rs`
- Modify: `docs/database-schema.md`
- Modify: `docs/backlog.md`
- Verify: containment scans

- [ ] **Step 1: Add RED playlist integration assertion**

In `src-tauri/src/youtube/playlist.rs`, extend the existing playlist tests with:

```rust
#[tokio::test]
async fn playlist_item_video_source_upsert_writes_typed_video_metadata_not_source_blob() {
    let pool = crate::sources::test_support::memory_pool_with_sources().await;
    crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
    create_youtube_playlist_items_table(&pool).await;
    create_youtube_unique_indexes(&pool).await;
    let mut tx = pool.begin().await.expect("begin tx");
    let playlist_source_id = upsert_youtube_playlist_source(&mut tx, &playlist_metadata(vec![
        playlist_item("video01", YoutubeAvailabilityStatus::Available),
    ]))
    .await
    .expect("upsert playlist source");

    upsert_playlist_items(
        &mut tx,
        playlist_source_id,
        &playlist_metadata(vec![playlist_item("video01", YoutubeAvailabilityStatus::Available)]),
    )
    .await
    .expect("upsert playlist items");
    tx.commit().await.expect("commit");

    let row: (Option<Vec<u8>>, String) = sqlx::query_as(
        r#"
        SELECT sources.metadata_zstd, youtube_video_sources.video_id
        FROM youtube_playlist_items
        JOIN sources ON sources.id = youtube_playlist_items.video_source_id
        JOIN youtube_video_sources ON youtube_video_sources.source_id = sources.id
        WHERE youtube_playlist_items.video_id = 'video01'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("load linked typed video");

    assert_eq!(row.0, None);
    assert_eq!(row.1, "video01");
}
```

If the helper names in the file differ, use the local helpers already present in `playlist.rs` and add only the missing typed table creation call.

- [ ] **Step 2: Run playlist tests and verify RED or GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::playlist::
```

Expected: if Task 4 already covers the behavior through `upsert_youtube_video_source`, this may pass immediately. If it fails, add typed table creation to playlist tests or fix the playlist fixture schema.

- [ ] **Step 3: Update docs**

In `docs/database-schema.md`:

- Replace the note that says YouTube source rows keep metadata in `sources.metadata_zstd` with:

```markdown
- YouTube source rows keep `sources.metadata_zstd` `NULL` after successful typed writes. Existing invalid or unbackfillable legacy YouTube blobs may remain inert, but normal YouTube listing, detail, jobs, and analysis do not decode them.
- YouTube video and playlist runtime metadata lives in `youtube_video_sources` and `youtube_playlist_sources`.
```

- Add sections after `telegram_sources`:

```markdown
### 1.3 `youtube_video_sources`

Stores typed runtime metadata for direct YouTube video sources. Generic identity and display snapshot fields remain in `sources`; provider-specific title, channel, thumbnail, canonical URL, availability, description, and provider-work hints live here.

Important fields:

- `source_id`
- `video_id`
- `canonical_url`
- `title`
- `channel_title`
- `channel_id`
- `channel_handle`
- `channel_url`
- `author_display`
- `published_at`
- `duration_seconds`
- `description`
- `thumbnail_url`
- `tags_json`
- `chapters_json`
- `video_form`
- `availability_status`
- `caption_language_override`
- `raw_metadata_version`
- `raw_metadata_zstd`

Notes:

- `source_id` references `sources(id)` with `ON DELETE CASCADE`.
- `video_id` must match `sources.external_id`; Rust upsert/backfill code validates this cross-table invariant.
- `raw_metadata_zstd` is optional archive/debug/reparse/migration payload only. Normal listing, detail, jobs, and analysis do not decode it.

### 1.4 `youtube_playlist_sources`

Stores typed runtime metadata for YouTube playlist sources.

Important fields:

- `source_id`
- `playlist_id`
- `canonical_url`
- `title`
- `channel_title`
- `channel_id`
- `channel_handle`
- `channel_url`
- `thumbnail_url`
- `video_count`
- `availability_status`
- `raw_metadata_version`
- `raw_metadata_zstd`

Notes:

- `playlist_id` must match `sources.external_id`; Rust upsert/backfill code validates this cross-table invariant.
- Playlist entry payloads remain in `youtube_playlist_items.metadata_zstd`.
```

Renumber later headings if the document uses sequential section numbers.

- [ ] **Step 4: Update backlog**

In `docs/backlog.md`, find the YouTube typed metadata ownership item and mark it complete, or add this completed note under the YouTube backend section if no matching item exists:

```markdown
- [x] Move YouTube source runtime metadata from generic `sources.metadata_zstd` into typed video/playlist source tables; keep raw provider payload optional and out of normal listing/detail/jobs/analysis paths.
```

- [ ] **Step 5: Run containment scans**

Run:

```powershell
rg -n "decode_youtube_metadata|decode_video_metadata|decode_playlist_metadata|decompress_bytes" src-tauri/src/youtube src-tauri/src/analysis/corpus.rs src-tauri/src/sources/store.rs
rg -n "raw_metadata_zstd" src-tauri/src/youtube src-tauri/src/analysis/corpus.rs src-tauri/src/sources/store.rs
rg -n "metadata_zstd = excluded.metadata_zstd|VALUES \\('youtube'.*metadata_zstd" src-tauri/src/sources/store.rs
rg -n "sources\\.metadata_zstd" src-tauri/src/youtube/detail.rs src-tauri/src/youtube/jobs.rs src-tauri/src/analysis/corpus.rs
```

Expected:

- Legacy decode helpers only remain in migration/debug/test code and are named `decode_legacy_*`.
- `raw_metadata_zstd` appears in schema, typed upsert/backfill, and tests, not in normal detail/jobs/analysis decode logic.
- No YouTube source upsert writes `metadata_zstd = excluded.metadata_zstd`.
- No normal detail/jobs/analysis query selects `sources.metadata_zstd` for YouTube provider metadata.

- [ ] **Step 6: Run focused test suites**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources:: youtube:: analysis::corpus:: migrations::
```

Expected:

```text
test result: ok
```

- [ ] **Step 7: Run full verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected:

```text
test result: ok
```

`cargo fmt --check` and `git diff --check` should exit 0.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/youtube/playlist.rs docs/database-schema.md docs/backlog.md
git commit -m "docs: document youtube typed source metadata"
```

## Task 9: Final Review And Session Context

**Files:**
- Modify: `reference/session-context-2026-05-10-analysis-redesign.md`
- Verify: final status and commit log

- [ ] **Step 1: Update session context**

Append a concise entry to `reference/session-context-2026-05-10-analysis-redesign.md`:

```markdown
### 2026-05-17 YouTube Typed Source Metadata

- Implemented typed `youtube_video_sources` and `youtube_playlist_sources`.
- Added runner-managed migration 20 for schema creation and valid legacy blob backfill.
- Normal YouTube listing/detail/jobs/analysis read typed columns and no longer decode `sources.metadata_zstd`.
- `raw_metadata_zstd` is optional, versioned, secret-safe, and not decoded by normal runtime paths.
- Telegram metadata behavior remains unchanged from the Telegram cleanup slice.
```

- [ ] **Step 2: Run final verification before claiming completion**

Use `superpowers:verification-before-completion`, then run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git status --short --branch
```

Expected:

```text
test result: ok
```

`cargo fmt --check` and `git diff --check` exit 0. `git status` shows only expected committed branch state.

- [ ] **Step 3: Commit session context**

```powershell
git add reference/session-context-2026-05-10-analysis-redesign.md
git commit -m "docs: update session context for youtube typed metadata"
```

- [ ] **Step 4: Finish the branch**

Use `superpowers:finishing-a-development-branch`. Present the merge/push/keep/discard options after all verification passes.

## Self-Review

- Spec coverage: Tasks 1-3 cover schema, managed migration, valid/invalid backfill, and sentinel behavior. Task 4 covers atomic write path and clearing `sources.metadata_zstd`. Tasks 5-7 cover listing/detail/jobs/analysis typed reads and missing typed metadata behavior. Task 8 covers playlist integration, docs, raw/source blob containment scans, and full verification. Task 9 updates the session context and hands off branch completion.
- Placeholder scan: This plan contains no unresolved placeholder keywords, no undefined "do later" steps, and no generic "write tests" step without named tests and commands.
- Type consistency: The plan consistently uses `youtube_video_sources`, `youtube_playlist_sources`, `YoutubeVideoSourceColumns`, `YoutubePlaylistSourceColumns`, `YoutubeVideoSourceMetadata`, `YoutubePlaylistSourceMetadata`, `YOUTUBE_RAW_METADATA_VERSION`, and migration version 20.
