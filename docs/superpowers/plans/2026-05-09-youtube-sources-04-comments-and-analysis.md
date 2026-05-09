# YouTube Sources Part 4: Comments and Analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add YouTube comments and make YouTube sources fully analyzable with provider-safe groups, playlist expansion, corpus modes, snapshots, and timestamp trace refs.

**Architecture:** Comments are ordinary text corpus items with `item_kind = youtube_comment`. Analysis stays provider-aware: Telegram and YouTube groups never mix, playlist sources expand into canonical video sources, and saved runs keep frozen YouTube metadata.

**Tech Stack:** Tauri 2, Rust 2021, sqlx SQLite, zstd, `yt-dlp`, Svelte 5, LLM analysis pipeline.

---

## Consistent End State

After this part:

- YouTube comments can be synced when selected.
- YouTube videos, playlists, and YouTube-only groups can be analyzed.
- Telegram and YouTube sources cannot mix in one group.
- Saved analysis runs remain stable after later transcript/comment resyncs.
- Timestamp refs resolve to YouTube-aware trace entries.
- Focused manual verification covers live comments ingest, provider-safe analysis scopes, and saved-run/trace stability before moving to Part 5.

---

## Task 1: Comments Ingest

**Files:**

- Create: `src-tauri/src/youtube/comments.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/sources/items.rs`

- [ ] Add `comments` module to `src-tauri/src/youtube/mod.rs`.

- [ ] Fetch comments with:

```text
yt-dlp --dump-single-json --write-comments --skip-download --extractor-args youtube:max_comments=<limit> <video_url>
```

- [ ] Keep comment ingest bounded. Add this runtime constant in `src-tauri/src/youtube/comments.rs`:

```rust
const DEFAULT_MAX_COMMENTS_PER_VIDEO: usize = 1_000;
```

Use `--extractor-args` and `youtube:max_comments=<limit>` as two separate command arguments. `yt-dlp` handles YouTube comment pagination internally up to that limit; this task must not attempt a custom YouTube comments pager.

After parsing the `comments` array, enforce the same limit in Rust before normalization:

```rust
let comments = raw_comments.into_iter().take(max_comments).collect::<Vec<_>>();
if raw_total > max_comments {
    warnings.push(format!("Comment sync truncated at {max_comments} comments."));
}
```

This keeps the `--dump-single-json --write-comments` payload bounded for common videos and protects the app if a future `yt-dlp` version ignores the extractor argument.

- [ ] Normalize top-level comments and replies into `YoutubeComment`.

Read comment timestamps with this policy:

```rust
fn comment_published_at(raw: &serde_json::Value, fallback_timestamp: i64) -> i64 {
    raw.get("timestamp")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse::<i64>().ok()))
        .unwrap_or(fallback_timestamp)
}
```

Use the video upload timestamp from the enclosing `yt-dlp` JSON as `fallback_timestamp`; if the video timestamp is also missing, use the sync start timestamp. Add a warning for every comment whose own `timestamp` is missing or unparsable.

- [ ] Persist each comment/reply as an item:

```text
item_kind = youtube_comment
external_id = comment:<comment_id>
content_kind = text_only
author = comment author
published_at = comment publication timestamp
```

- [ ] Store parent id, reply state, like count, pinned state, creator reaction, author metadata, and raw provider payload in compressed raw data.

- [ ] Add `upsert_youtube_comment_item` in `src-tauri/src/sources/items.rs`; comments must update existing rows instead of creating duplicates.

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
VALUES (?, ?, 'youtube_comment', ?, ?, strftime('%s','now'), ?, ?, 'text_only', 0, NULL, NULL, NULL, NULL, NULL, NULL, ?)
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
    media_metadata_zstd = excluded.media_metadata_zstd,
    reaction_count = excluded.reaction_count
RETURNING id
```

Add a test that runs `upsert_youtube_comment_item` twice with the same `(source_id, external_id)` and asserts there is one row with updated content and reaction count.

- [ ] Run comments only when `YoutubeSyncOptions.comments = true`.

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::comments youtube::jobs sources::items --lib
```

Expected: comment command args include bounded extractor args, timestamp normalization handles numeric/string/missing values, comments and replies upsert idempotently, and the job path only runs comments when `YoutubeSyncOptions.comments` is true.

- [ ] Commit:

```powershell
git add src-tauri/src/youtube src-tauri/src/sources
git commit -m "feat: ingest youtube comments"
```

---

## Task 2: Provider-Safe Analysis Groups

**Files:**

- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/groups.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-source-groups.ts`
- Modify: `src/lib/components/analysis/source-group-editor.svelte`
- Modify: `src/routes/analysis/+page.svelte`

- [ ] Add `source_type` to `AnalysisSourceGroup` and `AnalysisSourceGroupRow`.

Also add `source_type` to `AnalysisSourceOption`; `source-group-editor.svelte` receives `AnalysisSourceOption[]`, so it needs provider information to filter candidate sources.

```rust
pub struct AnalysisSourceOption {
    pub id: i64,
    pub account_id: Option<i64>,
    pub source_type: String,
    pub title: Option<String>,
    pub item_count: i64,
    pub last_synced_at: Option<i64>,
}
```

Update `list_analysis_sources` in `src-tauri/src/analysis/mod.rs`:

```sql
SELECT
    sources.id,
    sources.account_id,
    sources.source_type,
    sources.title,
    COUNT(items.content_zstd) AS item_count,
    sources.last_synced_at
FROM sources
LEFT JOIN items ON items.source_id = sources.id
GROUP BY sources.id, sources.account_id, sources.source_type, sources.title, sources.last_synced_at
```

Update `AnalysisSourceOption` in `src/lib/types/analysis.ts` with `source_type: "telegram" | "youtube" | "rss" | "forum"`.

Rust shape in `src-tauri/src/analysis/models.rs`:

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct AnalysisSourceGroup {
    pub id: i64,
    pub name: String,
    pub source_type: String,
    pub members: Vec<AnalysisSourceGroupMember>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow)]
pub(crate) struct AnalysisSourceGroupRow {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) source_type: String,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}
```

TypeScript shape in `src/lib/types/analysis.ts`:

```ts
export type AnalysisGroupSourceType = "telegram" | "youtube";

export interface AnalysisSourceGroup {
  id: number;
  name: string;
  source_type: AnalysisGroupSourceType;
  members: AnalysisSourceGroupMember[];
  created_at: number;
  updated_at: number;
}
```

- [ ] Update create/update commands to accept `source_type`.

Command signatures:

```rust
create_analysis_source_group(
    handle: AppHandle,
    name: String,
    source_type: String,
    source_ids: Vec<i64>,
) -> AppResult<AnalysisSourceGroup>

update_analysis_source_group(
    handle: AppHandle,
    group_id: i64,
    name: String,
    source_type: String,
    source_ids: Vec<i64>,
) -> AppResult<AnalysisSourceGroup>
```

Input types:

```ts
export interface CreateAnalysisSourceGroupInput {
  name: string;
  sourceType: AnalysisGroupSourceType;
  sourceIds: number[];
}

export interface UpdateAnalysisSourceGroupInput extends CreateAnalysisSourceGroupInput {
  groupId: number;
}
```

- [ ] Existing groups use migration default `telegram`.

Update any in-memory `analysis_source_groups` test tables to include:

```sql
source_type TEXT NOT NULL DEFAULT 'telegram'
```

- [ ] Validate group membership:

```text
Telegram group -> only source_type = telegram
YouTube group -> only source_type = youtube
```

- [ ] Perform membership validation inside `create_analysis_source_group` and `update_analysis_source_group`, after `ensure_sources_exist` and before opening the write transaction.

Add a helper in `src-tauri/src/analysis/groups.rs`:

```rust
async fn validate_group_source_type(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    group_source_type: &str,
    source_ids: &[i64],
) -> AppResult<()> {
    if !matches!(group_source_type, "telegram" | "youtube") {
        return Err(AppError::validation("Analysis group source_type must be telegram or youtube"));
    }

    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT id, source_type FROM sources WHERE id IN (",
    );
    {
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        source_type: String,
    }

    let rows: Vec<Row> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    if let Some(row) = rows.iter().find(|row| row.source_type != group_source_type) {
        return Err(AppError::validation(format!(
            "Source {} has type '{}' and cannot be added to a '{}' analysis group",
            row.id, row.source_type, group_source_type
        )));
    }

    Ok(())
}
```

Use `INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)` for create and `UPDATE analysis_source_groups SET name = ?, source_type = ?, updated_at = ?` for update.

- [ ] Add frontend group type selector:

```text
Telegram
YouTube
```

- [ ] Filter candidate sources in the group editor by selected group provider.

Wire editor state through `src/routes/analysis/+page.svelte`:

```ts
let groupSourceType = $state<AnalysisGroupSourceType>("telegram");
```

When selecting an existing group, copy `selectedGroup.source_type` into `groupSourceType`. When starting a new group, default to `"telegram"` and clear selected member ids.

Pass these props to `source-group-editor.svelte`:

```svelte
groupSourceType={groupSourceType}
onChangeGroupSourceType={(value) => {
  groupSourceType = value;
  groupMemberSourceIds = groupMemberSourceIds.filter((sourceId) =>
    sourceMetrics[sourceId]?.source_type === value
  );
}}
```

Inside `source-group-editor.svelte`, render a select before the member list:

```svelte
<label>Group type
  <select
    value={groupSourceType}
    onchange={(event) => onChangeGroupSourceType((event.currentTarget as HTMLSelectElement).value as AnalysisGroupSourceType)}
  >
    <option value="telegram">Telegram</option>
    <option value="youtube">YouTube</option>
  </select>
</label>
```

Use a derived filtered list:

```ts
const candidateSources = $derived(sources.filter((source) => source.source_type === groupSourceType));
```

Render `candidateSources` instead of `sources`. Save calls must pass `sourceType: groupSourceType` to `createAnalysisSourceGroup` and `updateAnalysisSourceGroup`.

- [ ] Run:

```powershell
cd src-tauri
cargo test analysis::groups analysis::store --lib
cd ..
npm test -- analysis-source-groups
npm run check
```

Expected: mixed groups are rejected and group editor typechecks.

- [ ] Commit:

```powershell
git add src-tauri/src/analysis src/lib/types/analysis.ts src/lib/api/analysis-source-groups.ts src/lib/components/analysis/source-group-editor.svelte
git add src/routes/analysis/+page.svelte
git commit -m "feat: enforce provider-specific analysis groups"
```

---

## Task 3: YouTube Corpus Loading and Playlist Expansion

**Files:**

- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-runs.ts`
- Modify: `src/lib/components/analysis/run-controls.svelte`

- [ ] Add `YoutubeCorpusMode` wire values:

```text
transcript_only
transcript_description
transcript_description_comments
```

Rust type in `src-tauri/src/analysis/corpus.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum YoutubeCorpusMode {
    TranscriptOnly,
    TranscriptDescription,
    TranscriptDescriptionComments,
}

impl YoutubeCorpusMode {
    pub(crate) fn from_wire(value: Option<&str>) -> Result<Self, String> {
        match value.unwrap_or("transcript_description") {
            "transcript_only" => Ok(Self::TranscriptOnly),
            "transcript_description" => Ok(Self::TranscriptDescription),
            "transcript_description_comments" => Ok(Self::TranscriptDescriptionComments),
            other => Err(format!("Unsupported youtube_corpus_mode '{other}'")),
        }
    }

    pub(crate) fn includes_description(self) -> bool {
        matches!(self, Self::TranscriptDescription | Self::TranscriptDescriptionComments)
    }

    pub(crate) fn includes_comments(self) -> bool {
        matches!(self, Self::TranscriptDescriptionComments)
    }
}
```

- [ ] Add `youtube_corpus_mode` to `start_analysis_report` command input.

Rust command signature:

```rust
start_analysis_report(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template_id: i64,
    model_override: Option<String>,
    profile_id: Option<String>,
    youtube_corpus_mode: Option<String>,
) -> AppResult<i64>
```

TypeScript input:

```ts
export type YoutubeCorpusMode =
  | "transcript_only"
  | "transcript_description"
  | "transcript_description_comments";

export interface AnalysisReportStartCommand {
  sourceId: number | null;
  sourceGroupId: number | null;
  periodFrom: number;
  periodTo: number;
  outputLanguage: string;
  promptTemplateId: number;
  modelOverride: string | null;
  profileId: string | null;
  youtubeCorpusMode: YoutubeCorpusMode;
}
```

Telegram runs should ignore `youtube_corpus_mode`. YouTube runs default to `transcript_description` if the frontend sends `null` or an older client omits the field.

- [ ] Resolve source IDs for YouTube scopes:

```text
single video -> that video source
single playlist -> linked child video_source_id values from youtube_playlist_items
YouTube group -> direct videos plus expanded playlist children
```

Implement this in `src-tauri/src/analysis/corpus.rs` with a resolver that returns canonical analysis source IDs and skipped playlist entries:

```rust
pub(crate) struct ResolvedAnalysisSources {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    pub(crate) skipped_unlinked_playlist_items: usize,
}
```

Rules:

- Telegram single source returns that source id.
- Telegram group returns its direct members.
- YouTube single video returns that video source id.
- YouTube single playlist returns only rows from `youtube_playlist_items` where `video_source_id IS NOT NULL` and `is_removed_from_playlist = 0`.
- YouTube group expands each playlist member with the same rule and includes direct video members as-is.
- Rows with `video_source_id IS NULL` are excluded from corpus and preflight; they do not create empty `CorpusMessage` values. Count them in `skipped_unlinked_playlist_items` so the UI/logging path can later expose a warning.
- If expansion yields zero source ids, return `AppError::validation("No linked YouTube videos are available for analysis in this scope")`.

Use this SQL for playlist expansion:

```sql
SELECT video_source_id
FROM youtube_playlist_items
WHERE playlist_source_id = ?
  AND video_source_id IS NOT NULL
  AND is_removed_from_playlist = 0
ORDER BY COALESCE(position, 9223372036854775807), video_id
```

In `start_analysis_report`, replace the current `(scope_type, resolved_source_id, resolved_group_id, scope_label, source_ids)` tuple with `(scope_type, resolved_source_id, resolved_group_id, scope_label, resolved_sources)`. Parse `youtube_corpus_mode` once, pass it to the resolver, then construct one `CorpusLoadRequest` and reuse it for preflight and `ReportRunInput`.

- [ ] Load corpus by item kind:

```text
Telegram -> telegram_message
YouTube transcript -> youtube_transcript
YouTube comments -> youtube_comment
```

Replace the unfiltered item query in `load_corpus_messages`; never load every `content_zstd IS NOT NULL` row by source id alone.

Use a provider-aware request shape:

```rust
pub(crate) struct CorpusLoadRequest {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
}
```

For Telegram, add:

```sql
AND items.item_kind = 'telegram_message'
```

For YouTube transcript-only and transcript+description modes, add:

```sql
AND items.item_kind = 'youtube_transcript'
```

For YouTube transcript+description+comments mode, add:

```sql
AND items.item_kind IN ('youtube_transcript', 'youtube_comment')
```

- [ ] Append YouTube description text from source metadata only when corpus mode includes description.

Descriptions are separate synthetic `CorpusMessage` values, not appended to transcript text. Decode each video source `metadata_zstd` into `YoutubeVideoMetadata` and include a description message only when:

- `youtube_corpus_mode` is `transcript_description` or `transcript_description_comments`;
- metadata contains a non-empty `description`;
- the video `published_at` timestamp is inside the requested analysis period.

Synthetic description message shape:

```rust
CorpusMessage {
    item_id: 0,
    source_id,
    external_id: format!("description:{video_id}"),
    published_at: video_published_at,
    author: metadata.channel_title.clone(),
    content: format!(
        "YouTube video description\nTitle: {title}\nChannel: {channel}\nURL: {url}\n\n{description}",
        title = metadata.title.unwrap_or_else(|| video_id.clone()),
        channel = metadata.channel_title.unwrap_or_else(|| "unknown".to_string()),
        url = metadata.canonical_url,
        description = description
    ),
    r#ref: format!("s{source_id}-i0"),
    item_kind: Some("youtube_description".to_string()),
    source_type: Some("youtube".to_string()),
    source_subtype: Some("video".to_string()),
    metadata_zstd: Some(compressed_description_metadata),
}
```

`s{source_id}-i0` is reserved for the synthetic description document. It is valid for trace lookup from saved corpus but must never be used for persisted `items` rows.

Because `item_id = 0` is not a database row id, treat description messages as synthetic documents throughout trace handling. Frontend code must not call item-loading APIs with `item_id = 0`; use `AnalysisTraceRef.is_synthetic` from Task 4 to display these refs as saved evidence only.

- [ ] Include comments only when corpus mode includes comments and comments have been synced.

- [ ] Update `preflight_analysis_run` to use the same provider-aware corpus loader as report execution. Preflight must count the exact documents that would be sent to the LLM: transcripts, optional synthetic descriptions, and optional comments. Do not keep a separate SQL path that counts all items by `source_id`.

Implementation pattern:

```rust
let corpus_request = CorpusLoadRequest {
    source_type: resolved_sources.source_type.clone(),
    source_ids: resolved_sources.source_ids.clone(),
    period_from,
    period_to,
    youtube_corpus_mode,
};

let corpus = load_corpus_messages(pool, &corpus_request).await?;

let message_sizes = corpus
    .iter()
    .map(|message| {
        estimate_message_input_chars(
            &message.content,
            &message.r#ref,
            message.author.as_deref(),
        )
    })
    .collect::<Vec<_>>();
```

Update `ReportRunInput` in `src-tauri/src/analysis/report.rs` to carry `corpus_request: CorpusLoadRequest` instead of a raw `source_ids: Vec<i64>`. Both preflight and `run_report_pipeline` must call `load_corpus_messages` with the same `CorpusLoadRequest`.

- [ ] Add tests in `src-tauri/src/analysis/corpus.rs`:

```text
load_corpus_messages filters Telegram to item_kind = telegram_message
load_corpus_messages filters YouTube transcript_only to youtube_transcript
load_corpus_messages includes youtube_comment only in transcript_description_comments mode
playlist expansion excludes video_source_id NULL rows
description mode creates one synthetic description CorpusMessage with ref s<source>-i0
preflight count matches load_corpus_messages count for each YouTube corpus mode
```

- [ ] Run:

```powershell
cd src-tauri
cargo test analysis::corpus analysis::report --lib
cd ..
npm test -- analysis-run-workflow
npm run check
```

Expected: playlist expansion, item_kind filtering, description synthetic messages, and corpus-mode-aware preflight tests pass.

- [ ] Commit:

```powershell
git add src-tauri/src/analysis src/lib/types/analysis.ts src/lib/api/analysis-runs.ts src/lib/components/analysis/run-controls.svelte
git commit -m "feat: load youtube analysis corpus"
```

---

## Task 4: Timestamp Trace Refs and Run Snapshots

**Files:**

- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/trace.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-trace.ts`
- Modify: `src/lib/analysis-trace-workflow.test.ts`

- [ ] Extend `CorpusMessage` with optional snapshot metadata fields:

```rust
pub(crate) struct CorpusMessage {
    pub(crate) item_id: i64,
    pub(crate) source_id: i64,
    pub(crate) external_id: String,
    pub(crate) published_at: i64,
    pub(crate) author: Option<String>,
    pub(crate) content: String,
    pub(crate) r#ref: String,
    pub(crate) item_kind: Option<String>,
    pub(crate) source_type: Option<String>,
    pub(crate) source_subtype: Option<String>,
    pub(crate) metadata_zstd: Option<Vec<u8>>,
}
```

- [ ] Update every existing `CorpusMessage { ... }` struct literal after adding these fields. Grep first:

```powershell
rg -n "CorpusMessage \\{" src-tauri/src
```

For Telegram test fixtures and legacy live messages, add:

```rust
item_kind: Some("telegram_message".to_string()),
source_type: Some("telegram".to_string()),
source_subtype: None,
metadata_zstd: None,
```

For tests where provider is irrelevant, using `None` for all four fields is acceptable only when the test does not exercise persistence, trace output, or provider filtering.

- [ ] Extend `StoredAnalysisItemRow` and `StoredRunSnapshotRow` in `src-tauri/src/analysis/models.rs` with the same fields. Live corpus queries should select:

```sql
items.item_kind,
sources.source_type,
sources.source_subtype,
items.media_metadata_zstd AS metadata_zstd
```

Snapshot queries should select:

```sql
item_kind,
source_type,
source_subtype,
metadata_zstd
```

from `analysis_run_messages`.

- [ ] Persist these fields into `analysis_run_messages`.

Update `persist_run_snapshot` in `src-tauri/src/analysis/store.rs`:

```sql
INSERT INTO analysis_run_messages (
    run_id,
    item_id,
    source_id,
    external_id,
    author,
    published_at,
    ref,
    content_zstd,
    item_kind,
    source_type,
    source_subtype,
    metadata_zstd
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
```

Bind:

```rust
.bind(message.item_kind.as_deref())
.bind(message.source_type.as_deref())
.bind(message.source_subtype.as_deref())
.bind(message.metadata_zstd.as_deref())
```

Update every in-memory `analysis_run_messages` table in tests, including `snapshot_pool()` in `src-tauri/src/analysis/corpus.rs`, so those four columns exist.

- [ ] For YouTube transcript snapshots, include compressed metadata with video id, URL, title, channel, handle, caption language, caption kind, and item kind.

- [ ] Build transcript segment refs in corpus text:

```text
[s12-i400@754000ms] segment text
[s12-i400@790000ms] next segment text
```

For YouTube transcript corpus, load segment-level messages from `youtube_transcript_segments` instead of sending one giant transcript item. Each segment becomes one `CorpusMessage`:

```rust
CorpusMessage {
    item_id: transcript_item_id,
    source_id,
    external_id: transcript_external_id,
    published_at,
    author,
    content: segment_text,
    r#ref: format!("s{source_id}-i{transcript_item_id}@{}ms", segment.start_ms),
    item_kind: Some("youtube_transcript".to_string()),
    source_type: Some("youtube".to_string()),
    source_subtype: Some("video".to_string()),
    metadata_zstd: Some(segment_metadata_zstd),
}
```

`segment_metadata_zstd` should contain JSON with at least:

```json
{
  "video_id": "abc123",
  "canonical_url": "https://www.youtube.com/watch?v=abc123",
  "title": "Video title",
  "channel_title": "Channel",
  "channel_handle": "@channel",
  "caption_language": "en",
  "caption_track_kind": "manual",
  "segment_start_ms": 754000,
  "segment_end_ms": 790000,
  "item_kind": "youtube_transcript"
}
```

- [ ] Resolve timestamp refs to:

```text
ref
item_id
source_id
external_id
published_at
excerpt
youtube_url
youtube_timestamp_seconds
youtube_display_label
is_synthetic
```

- [ ] Add optional YouTube fields to `AnalysisTraceRef` in `src-tauri/src/analysis/models.rs`:

```rust
pub struct AnalysisTraceRef {
    pub r#ref: String,
    pub item_id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub published_at: i64,
    pub excerpt: String,
    pub youtube_url: Option<String>,
    pub youtube_timestamp_seconds: Option<i64>,
    pub youtube_display_label: Option<String>,
    pub is_synthetic: bool,
}
```

Update `src/lib/types/analysis.ts` with matching nullable fields:

```ts
export interface AnalysisTraceRef {
  ref: string;
  item_id: number;
  source_id: number;
  external_id: string;
  published_at: number;
  excerpt: string;
  youtube_url: string | null;
  youtube_timestamp_seconds: number | null;
  youtube_display_label: string | null;
  is_synthetic: boolean;
}
```

- [ ] Update `build_trace_refs` in `src-tauri/src/analysis/trace.rs` to resolve exact timestamp refs first, then fall back to base item refs for older citations.

For a ref like `s12-i400@754000ms`:

- exact match a segment-level `CorpusMessage.r#ref` when present;
- parse `754000ms` into `youtube_timestamp_seconds = Some(754)`;
- build `youtube_url` by appending `t=754` to the canonical URL from `metadata_zstd`;
- set `youtube_display_label` to `"Video title at 12:34"` when title metadata exists, otherwise `"YouTube at 12:34"`.

For old refs `s12-i400` and `s12-m400`, keep existing behavior and set all YouTube fields to `None` unless the matched `CorpusMessage` has YouTube metadata.

Set `is_synthetic = true` when the matched `CorpusMessage.item_id == 0` or `item_kind == Some("youtube_description")`. For synthetic refs, `AnalysisTraceRef.item_id` remains `0` for backward-compatible shape, but it must be documented and treated as "no database row". Add a focused regression test in `src/lib/analysis-trace-workflow.test.ts` that loads or resolves a synthetic YouTube description ref with `item_id: 0` and `is_synthetic: true`, keeps it selectable in trace state, and proves the trace workflow does not require a source-item lookup keyed by `item_id` to display the saved excerpt. Do not add a `listSourceItems`-style lookup in `trace-panel.svelte` or `src/routes/analysis/+page.svelte` for synthetic refs.

- [ ] Keep old `s12-i400` and legacy `s12-m400` refs working.

Add tests in `src-tauri/src/analysis/trace.rs`:

```text
build_trace_refs resolves exact YouTube timestamp refs from segment corpus messages
build_trace_refs converts milliseconds to integer YouTube timestamp seconds
build_trace_refs appends t=<seconds> to canonical YouTube URLs
build_trace_refs leaves old s12-i400 and s12-m400 refs working
AnalysisTraceRef serializes YouTube fields as null for Telegram refs
build_trace_refs marks youtube_description refs with item_id 0 as is_synthetic
```

- [ ] Run:

```powershell
cd src-tauri
cargo test analysis::trace analysis::corpus analysis::store --lib
cd ..
npm test -- analysis-trace analysis-trace-workflow
npm run check
```

Expected: saved run snapshots include YouTube metadata and timestamp trace refs resolve.

- [ ] Commit:

```powershell
git add src-tauri/src/analysis src/lib/types/analysis.ts src/lib/api/analysis-trace.ts src/lib/analysis-trace-workflow.test.ts
git commit -m "feat: resolve youtube timestamp evidence"
```

---

## Manual Verification

- [ ] Sync comments for a public video that already has transcript data and confirm top-level comments plus replies appear as `youtube_comment` items. Rerun comment sync and confirm the item count stays stable instead of duplicating rows.
- [ ] Create a YouTube-only analysis group and confirm adding a Telegram source is rejected with a validation error before any group membership write occurs.
- [ ] Run analysis for one YouTube video in all three corpus modes:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`

Confirm preflight and actual execution reflect the expected document set for each mode.

- [ ] Run analysis for a playlist that includes at least one unavailable or unlinked row and confirm only linked, non-removed child video sources enter the corpus. The run must not create empty documents for `video_source_id IS NULL` rows.
- [ ] Save a YouTube analysis run, resync transcript or comments, then reopen the saved run and confirm the old snapshot excerpt, metadata, and trace resolution remain unchanged.
- [ ] Open both a timestamp trace ref and a synthetic description ref. Confirm the timestamp ref produces a YouTube URL with `t=<seconds>`, and confirm the synthetic ref renders the saved excerpt without any source-item lookup using `item_id = 0`.
