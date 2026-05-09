# Session Context: YouTube Sources Planning

Updated: 2026-05-09
Workspace: `g:\Develop\Extractum`
User language: Russian
Current task state: planning only, no application implementation yet.

## Purpose

This file captures the current session context so another Codex session can continue without losing the thread.

The user asked to study `reference\youtube` and create a detailed, split implementation plan for adding YouTube sources to Extractum. The plan was first created as one large file, then split into independently consistent parts. Since then, the user has been feeding LLM review comments for each open plan part, and the plan files have been refined accordingly.

## Files Studied

Reference documents read:

- `reference/youtube/youtube_sources_mvp_v_1_specification.md`
- `reference/youtube/youtube_sources_implementation_plan.md`
- `reference/youtube/Рекомендуемый порядок работ.txt`

Note: the third file initially printed mojibake until read with UTF-8 encoding.

Relevant codebase areas inspected:

- `src-tauri/src/sources/types.rs`
- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/sources/items/query.rs`
- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/sources/test_support.rs`
- `src-tauri/src/analysis/models.rs`
- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/groups.rs`
- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/trace.rs`
- `src-tauri/src/analysis/mod.rs`
- `src-tauri/src/takeout_import/state.rs`
- `src/lib/types/analysis.ts`
- `src/lib/types/sources.ts`
- `src/lib/api/sources.ts`
- `src/lib/api/analysis-source-groups.ts`
- `src/lib/api/analysis-runs.ts`
- `src/lib/api/analysis-trace.ts`
- `src/lib/source-capabilities.ts`
- `src/lib/components/source-row.svelte`
- `src/lib/components/analysis/source-group-editor.svelte`
- `src/lib/components/analysis/trace-panel.svelte`
- `src/routes/analysis/+page.svelte`

## Plan Files

Created/updated under `docs/superpowers/plans/`:

- `2026-05-09-youtube-sources.md`: split-plan index and acceptance checklist.
- `2026-05-09-youtube-sources-01-foundation.md`: schema, semantic item kinds, YouTube DTOs and URL parsing.
- `2026-05-09-youtube-sources-02-preview-and-add.md`: yt-dlp preview, source creation, playlist membership, frontend add flow.
- `2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`: source jobs, metadata sync, transcripts.
- `2026-05-09-youtube-sources-04-comments-and-analysis.md`: comments, provider-safe analysis groups, YouTube corpus loading, snapshots, trace refs.
- `2026-05-09-youtube-sources-05-auth-and-settings.md`: auth/cookies/settings.
- `2026-05-09-youtube-sources-06-ui-hardening-docs.md`: UI polish, hardening, documentation.

Current git status before writing this file:

```text
?? docs/superpowers/
```

The plan directory is currently untracked. Application source code has not been modified in this planning session.

## High-Level Plan Architecture

The YouTube implementation is split into six consistent parts:

1. Foundation
   - Migration 16.
   - `items.item_kind`.
   - YouTube playlist and transcript tables.
   - Analysis snapshot schema extensions.
   - YouTube DTOs and URL parser.

2. Preview and Add
   - `yt-dlp` adapter.
   - Preview command.
   - Add YouTube video/playlist sources.
   - Playlist membership persistence.
   - Frontend add flow.

3. Jobs, Metadata, Transcripts
   - In-memory YouTube source jobs.
   - Metadata refresh.
   - Playlist metadata paging.
   - Transcript ingest.
   - Timestamp refs in parser.

4. Comments and Analysis
   - Comments ingest.
   - Provider-safe analysis groups.
   - YouTube corpus loading and playlist expansion.
   - Saved snapshots with YouTube metadata.
   - YouTube-aware trace refs.

5. Auth and Settings
   - Cookies/browser auth.
   - YouTube settings.
   - Auth-aware retries and failure handling.

6. UI Hardening and Docs
   - YouTube detail views.
   - Job visibility.
   - Manual QA.
   - Documentation.

Each part is intended to leave the project in a consistent state after completion.

## Important Codebase Facts Found

- `src-tauri/src/sources/types.rs` already has YouTube-related source type constants/types, including `SourceType::Youtube` and `YOUTUBE_SOURCE_TYPE`.
- `src-tauri/src/sources/store.rs::source_record_from_row` currently decodes all `metadata_zstd` as Telegram metadata through `decode_source_metadata`; this must be made provider-safe before storing YouTube metadata.
- `src-tauri/src/sources/sync.rs::sync_source` is Telegram-only. UI must not call `sync_source` for YouTube.
- `src/lib/source-capabilities.ts` currently returns `canSync: source.sourceSubtype === "playlist"` for YouTube, which conflicts with Part 2 and Part 3 transitional behavior. Plans now address this.
- `src-tauri/src/analysis/corpus.rs::load_corpus_messages` currently loads all rows with `content_zstd IS NOT NULL` by source id and does not filter by `item_kind`. Part 4 now requires provider-aware item filtering.
- `src-tauri/src/analysis/corpus.rs::preflight_analysis_run` currently has its own SQL path and must be updated to use the same provider-aware corpus loader as report execution.
- `src-tauri/src/analysis/store.rs::persist_run_snapshot` currently inserts only old snapshot columns and must bind `item_kind`, `source_type`, `source_subtype`, `metadata_zstd`.
- `src-tauri/src/analysis/trace.rs::normalize_ref` currently accepts only `s{digits}-i{digits}` and `s{digits}-m{digits}`. Part 3 extends it with optional timestamp suffixes for item refs.
- `src-tauri/src/takeout_import/state.rs` locks active Takeout jobs by `source_id`. Part 3 intentionally makes YouTube source jobs lock by `(source_id, job_type, related_source_id)`.
- `src/lib/analysis-state.ts` already defines `sourceActionPending` and `clearSourceActionPending`; Part 3 now explicitly tells implementers to reuse them.
- `src/lib/components/analysis/trace-panel.svelte` currently displays trace details only. It does not load source items by `item_id`.
- `src/lib/components/analysis/source-group-editor.svelte` receives `AnalysisSourceOption[]`, not full `Source[]`; therefore Part 4 adds `source_type` to `AnalysisSourceOption`.
- `src-tauri/Cargo.toml` currently has `tempfile = "3"` only under `[dev-dependencies]`; Part 3 moves it to runtime dependencies for captions.
- No `src-tauri/src/youtube` module exists yet in the current codebase before execution of the plans.

## Part 1: Foundation Current Content

Part 1 covers:

- Create `src-tauri/migrations/16.sql`.
- Register migration 16 in `src-tauri/src/migrations.rs`.
- Add `items.item_kind TEXT NOT NULL DEFAULT 'telegram_message'`.
- Add `idx_items_source_kind_published`.
- Create `youtube_playlist_items`.
- Create `youtube_transcript_segments`.
- Extend `analysis_run_messages` with:
  - `item_kind TEXT`
  - `source_type TEXT`
  - `source_subtype TEXT`
  - `metadata_zstd BLOB`
- Add `analysis_source_groups.source_type TEXT NOT NULL DEFAULT 'telegram'`.
- Add partial unique indexes:
  - `idx_sources_unique_youtube_video`
  - `idx_sources_unique_youtube_playlist`
- Insert YouTube app settings defaults.
- Add semantic item kind constants:
  - `ITEM_KIND_TELEGRAM_MESSAGE`
  - `ITEM_KIND_YOUTUBE_TRANSCRIPT`
  - `ITEM_KIND_YOUTUBE_COMMENT`
- Update `SourceItemInsert`, `StoredItemRow`, item queries, frontend source item types.
- Add YouTube DTOs and URL parser.

Review feedback already incorporated:

- `youtube_playlist_items.availability_status` now has a `CHECK` constraint.
- Migration registration test now checks all four new `analysis_run_messages` columns.
- `src-tauri/src/analysis/corpus.rs::snapshot_pool()` is explicitly included for schema updates.
- `app_settings` migration-only caveat documented for tests.
- `insert_source_item` SQL with `item_kind` and bind order specified.
- `items/query.rs` must select `items.item_kind`.
- Existing direct `SourceItemInsert` test constructions must be updated.
- Manual `StoredItemRow` SELECT in `items.rs` must include `item_kind`.
- `tempfile` removed from Part 1 and moved later to captions.
- URL parser rule documented: `list` parameter has priority over `v`.
- Task 1 commit command includes `src-tauri/src/analysis/corpus.rs`.

Important availability status values in Part 1:

```text
available
upcoming
live_now
live_ended_transcript_pending
no_captions
private_or_auth_required
members_only
age_restricted
geo_blocked
deleted
removed_from_playlist
unavailable_unknown
```

## Part 2: Preview and Add Current Content

Part 2 covers:

- Add `yt-dlp` adapter.
- Add preview commands for video and playlist.
- Add source creation helpers for YouTube video/playlist.
- Add playlist membership persistence.
- Add frontend YouTube add flow.

Review feedback already incorporated:

- `run_ytdlp` must handle missing binary and non-zero exit codes.
- Non-zero exit code error mapping must classify auth/private/member/age/geo/unavailable/deleted/network/rate-limit where possible.
- Preview commands must use `tokio::time::timeout`, currently 30s for preview.
- Playlist preview must limit items with `--playlist-items 1-50`.
- Command-builder helpers added:
  - `preview_video_args(canonical_url: &str) -> Vec<String>`
  - `preview_playlist_args(canonical_url: &str) -> Vec<String>`
- Test must assert `--playlist-items` and `1-50` are adjacent separate args, not `--playlist-items=1-50`.
- Fixture tests must cover missing optionals, availability mapping, and non-zero fake runner errors.
- YouTube source upsert must use the partial unique index shape:
  - `ON CONFLICT(source_type, source_subtype, external_id) WHERE source_type = 'youtube' AND source_subtype = 'video'`
  - playlist equivalent for `source_subtype = 'playlist'`.
- Playlist item video source creation must be race-safe.
- `add_youtube_source` persistence must happen in a transaction after metadata fetch.
- `source_record_from_row` must not decode YouTube metadata as Telegram metadata.
- Frontend flow uses a separate `youtube-source-add-panel.svelte`.
- YouTube `canSync` remains false in Part 2 for both video and playlist.
- Frontend API wrappers `previewYoutubeSource` and `addYoutubeSource` require tests.

## Part 3: Jobs, Metadata, Transcripts Current Content

Part 3 covers:

- Create `src-tauri/src/youtube/jobs.rs`.
- Add source job state and Tauri commands.
- Add frontend source job API.
- Add metadata sync.
- Add transcript ingest.
- Update trace ref normalization for timestamps.

Review feedback already incorporated:

- Job active lock is by `SourceJobKey { source_id, job_type, related_source_id }`, not source id alone.
- This allows metadata and transcript sync jobs for the same video to coexist while rejecting duplicate same-scope jobs.
- `list_source_jobs` takes `SourceJobListFilter`:
  - `source_id`
  - `status`
  - `limit`, default 100, max 500
- `retry_failed_youtube_playlist_videos` is specified as one aggregate retry job.
- Retryable playlist statuses:
  - `live_ended_transcript_pending`
  - `no_captions`
  - `unavailable_unknown`
- Non-retryable until Part 5:
  - `private_or_auth_required`
  - `members_only`
  - `age_restricted`
  - `geo_blocked`
  - `deleted`
  - `removed_from_playlist`
- Source job frontend wiring is specified in `src/routes/analysis/+page.svelte`.
- Existing `sourceActionPending` and `clearSourceActionPending` helpers from `$lib/analysis-state` must be reused.
- Playlist metadata sync is paged with `PLAYLIST_METADATA_PAGE_SIZE = 200`.
- Full playlist sync uses `--playlist-items` and `1-200` as separate args.
- Removed playlist rows are marked with `availability_status = 'removed_from_playlist'`.
- UI `syncSelectedSource` must branch:
  - YouTube calls `syncYoutubeSource`.
  - Telegram keeps `syncSource`.
- `canSync` becomes true for YouTube video/playlist in Part 3.
- Transcript ingest uses `TempDir` RAII cleanup.
- Caption parser policy:
  - prefer `json3`
  - fall back to `vtt`
- Caption parser helpers:
  - `parse_json3_transcript`
  - `parse_vtt_transcript`
- Transcript external id format:
  - `transcript:<video_id>:<language-or-und>:<manual|auto|unknown>`
- `upsert_youtube_transcript_item` must update text and return item id.
- Transcript segment replacement is explicit `DELETE + INSERT` in one transaction.
- `normalize_ref` accepts:
  - `s12-i400@754000ms`
  - `s12-i400@754000-790000ms`
- Old refs remain valid:
  - `s12-i400`
  - `s12-m400`
- Timestamp suffixes are allowed only for `-i`, not `-m`.

## Part 4: Comments and Analysis Current Content

Part 4 covers:

- Create `src-tauri/src/youtube/comments.rs`.
- Add comments ingest.
- Add provider-safe analysis groups.
- Add YouTube corpus modes.
- Add playlist expansion for analysis.
- Add saved snapshot metadata.
- Add YouTube-aware trace refs.

Review feedback already incorporated:

### Comments ingest

- Comment fetch command is bounded:

```text
yt-dlp --dump-single-json --write-comments --skip-download --extractor-args youtube:max_comments=<limit> <video_url>
```

- Runtime constant:

```rust
const DEFAULT_MAX_COMMENTS_PER_VIDEO: usize = 1_000;
```

- `--extractor-args` and `youtube:max_comments=<limit>` must be separate command args.
- Rust also truncates parsed comments with `.take(max_comments)`.
- Timestamp policy uses `timestamp` as integer or parseable string, fallback to video upload timestamp, then sync start timestamp.
- Missing/unparsable comment timestamps produce warnings.
- `upsert_youtube_comment_item` SQL is specified with `ON CONFLICT(source_id, external_id) DO UPDATE`.
- Comments run only when `YoutubeSyncOptions.comments = true`, not the old `sync_comments`.

### Provider-safe analysis groups

- Add `source_type` to:
  - `AnalysisSourceGroup`
  - `AnalysisSourceGroupRow`
  - `AnalysisSourceOption`
  - TypeScript analysis group/source option types.
- `list_analysis_sources` must select and group by `sources.source_type`.
- Create/update group commands accept `source_type`.
- Existing groups rely on migration default `telegram`.
- In-memory `analysis_source_groups` test tables must include `source_type TEXT NOT NULL DEFAULT 'telegram'`.
- Membership validation happens in `create_analysis_source_group` and `update_analysis_source_group`, after `ensure_sources_exist` and before write transaction.
- Helper `validate_group_source_type` is specified.
- `source-group-editor.svelte` needs group type select:
  - Telegram
  - YouTube
- `src/routes/analysis/+page.svelte` must track `groupSourceType` and pass it to group editor.
- Group editor filters candidate sources by `AnalysisSourceOption.source_type`.

### YouTube corpus loading

- `YoutubeCorpusMode` wire values:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`
- Rust enum `YoutubeCorpusMode` and `from_wire` are specified.
- `start_analysis_report` accepts `youtube_corpus_mode: Option<String>`.
- TypeScript `AnalysisReportStartCommand` includes `youtubeCorpusMode`.
- Telegram runs ignore `youtube_corpus_mode`.
- YouTube runs default to `transcript_description` when omitted/null.
- Resolver returns `ResolvedAnalysisSources`:
  - `source_type`
  - `source_ids`
  - `skipped_unlinked_playlist_items`
- YouTube playlist expansion includes only `video_source_id IS NOT NULL` and `is_removed_from_playlist = 0`.
- `video_source_id IS NULL` rows are excluded from corpus and preflight, not represented as empty documents.
- If YouTube expansion yields zero source ids, return validation error:
  - `No linked YouTube videos are available for analysis in this scope`
- `load_corpus_messages` must use `CorpusLoadRequest`, not raw ids.
- Item filtering:
  - Telegram: `items.item_kind = 'telegram_message'`
  - YouTube transcript modes: `items.item_kind = 'youtube_transcript'`
  - YouTube comments mode: `items.item_kind IN ('youtube_transcript', 'youtube_comment')`
- Description is a synthetic `CorpusMessage`, not appended to transcript text.
- Synthetic description ref:
  - `s{source_id}-i0`
- `item_id = 0` is reserved for synthetic description documents and is not a database row id.
- Frontend must not call item-loading APIs with `item_id = 0`.

### Preflight

- `preflight_analysis_run` must use the exact same `CorpusLoadRequest` and `load_corpus_messages` path as execution.
- Do not keep a separate SQL path that counts all items by source id.
- `ReportRunInput` should carry `corpus_request: CorpusLoadRequest`, not raw `source_ids: Vec<i64>`.

### Snapshots and trace refs

- `CorpusMessage` gets optional snapshot metadata:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- Every existing `CorpusMessage { ... }` struct literal must be updated.
- `StoredAnalysisItemRow` and `StoredRunSnapshotRow` get the same fields.
- Live corpus queries select:
  - `items.item_kind`
  - `sources.source_type`
  - `sources.source_subtype`
  - `items.media_metadata_zstd AS metadata_zstd`
- Snapshot queries select the new fields from `analysis_run_messages`.
- `persist_run_snapshot` inserts/binds:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- YouTube transcript corpus should load segment-level messages from `youtube_transcript_segments`.
- Segment refs look like:
  - `s12-i400@754000ms`
- Segment metadata should include video id, canonical URL, title, channel, handle, caption language/kind, segment times, item kind.
- `AnalysisTraceRef` gets:
  - `youtube_url: Option<String>`
  - `youtube_timestamp_seconds: Option<i64>`
  - `youtube_display_label: Option<String>`
  - `is_synthetic: bool`
- TypeScript `AnalysisTraceRef` gets nullable YouTube fields and `is_synthetic: boolean`.
- `build_trace_refs` resolves exact timestamp refs first, then old base item refs.
- For timestamp refs, milliseconds become integer YouTube timestamp seconds.
- `youtube_url` appends `t=<seconds>` to canonical URL.
- Synthetic refs have `is_synthetic = true` when `CorpusMessage.item_id == 0` or `item_kind == Some("youtube_description")`.
- `AnalysisTraceRef.item_id` stays `0` for synthetic refs for shape compatibility, but means "no database row".

## Part 5 and Part 6 Current State

Parts 5 and 6 were created during the initial split but have not yet been reviewed/refined in this later LLM-review loop.

Part 5 currently covers:

- YouTube auth settings.
- Cookie/browser auth support.
- Auth-aware yt-dlp args.
- Settings UI.
- Retry behavior for auth-dependent statuses.

Part 6 currently covers:

- Video detail view.
- Playlist detail view.
- Job status panels.
- Analysis UI polish.
- Error handling and docs.
- Manual verification.

Expect these two parts may still need the same kind of review-driven hardening as Parts 1 through 4.

## Important Implementation Gotchas

- Do not call `sync_source` for YouTube. Use `sync_youtube_source`.
- Do not decode YouTube `metadata_zstd` with Telegram `decode_source_metadata`.
- Do not fetch full playlists in one unbounded preview call.
- Do not fetch full comments without `max_comments` bound.
- Do not rely on `items.content_zstd IS NOT NULL` alone for analysis corpus.
- Do not mix Telegram and YouTube sources in an analysis group.
- Do not include YouTube playlist rows with `video_source_id IS NULL` in corpus.
- Do not use `item_id = 0` as a database item id. It is synthetic only.
- Do not add duplicate dependencies without checking `Cargo.toml`.
- Use `TempDir` RAII for caption downloads; do not leak temporary files.
- Keep old refs `sN-iM` and `sN-mM` working.
- Keep every plan part consistent after each task.

## Verification Already Done During Planning

Only plan-level checks were run. No project tests were executed because application code has not been changed.

Plan checks run repeatedly:

```powershell
Select-String -Path docs\superpowers\plans\2026-05-09-youtube-sources-XX-*.md -Pattern 'TBD|TODO|implement later|fill in details|Add appropriate|Write tests for the above|Similar to Task'
```

Specific scans also checked for old conflicting strings such as:

- `sync_comments`
- `load_corpus_messages(pool, &input.source_ids`
- `sourceCatalog.sources`
- old `list_source_jobs() -> Vec<SourceJobRecord>`
- old transcript external id shape `transcript:<video_id>`

The latest scans for Parts 3 and 4 returned no matches for the targeted placeholder/conflict patterns.

## Current User Request

The user asked:

```text
в файл docs\session-context-2026-05-03.md запиши всю информацию, по которой можно восстановить контекст текущей сессии. Файл можно просто перезаписать. Сформируй commit message
```

This file is the requested session context snapshot.

## Suggested Commit Message

```text
docs: capture youtube sources planning session context
```

## Recommended Next Steps

1. If the user continues with LLM review feedback, keep using:
   - `superpowers:receiving-code-review`
   - `superpowers:writing-plans`
2. Review and harden Part 5 next if the open file changes to auth/settings.
3. Review and harden Part 6 after Part 5.
4. Before any implementation, use the execution skills requested in the plan headers:
   - `superpowers:subagent-driven-development`, or
   - `superpowers:executing-plans`
5. At implementation time, verify each task with the exact commands in the corresponding plan part.
