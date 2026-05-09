# Session Context: YouTube Sources Planning

Updated: 2026-05-09
Workspace: `g:\Develop\Extractum`
User language: Russian
Current task state: planning only. No application implementation has been executed in this planning thread.

## Purpose

This file captures the current session context so a future Codex session can continue without losing the thread.

The user asked to study `reference\youtube` and create a detailed, split implementation plan for adding YouTube sources to Extractum. The plan was first created as one large file, then split into six independently consistent parts. Since then, the user has been feeding LLM review comments for each open plan part, and the plan files have been refined accordingly.

The latest work in this session refined Part 5 and Part 6 after LLM review feedback, then rewrote this context snapshot.

## Current IDE Context

Open files/tabs reported by the IDE:

- Active: `docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`

## Git/Workspace State

Before rewriting this file in the latest request:

```text
git status --short
```

returned no output.

`git ls-files` showed these files are tracked:

- `docs/session-context-2026-05-03.md`
- `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`
- `docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md`

Earlier in the planning thread, the plan directory was reported as untracked. The current observed git state is clean/tracked before this context rewrite.

## Files Studied

Reference documents read:

- `reference/youtube/youtube_sources_mvp_v_1_specification.md`
- `reference/youtube/youtube_sources_implementation_plan.md`
- `reference/youtube/Р РµРєРѕРјРµРЅРґСѓРµРјС‹Р№ РїРѕСЂСЏРґРѕРє СЂР°Р±РѕС‚.txt`

Note: the third file initially printed mojibake until read with UTF-8 encoding.

Relevant codebase areas inspected during planning:

- `src-tauri/src/sources/types.rs`
- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/sources/items/query.rs`
- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/sources/settings.rs`
- `src-tauri/src/sources/test_support.rs`
- `src-tauri/src/secret_store.rs`
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
- `src/lib/analysis-source-state.ts`
- `src/lib/analysis-scope-state.ts`
- `src/lib/analysis-state.ts`
- `src/lib/components/source-row.svelte`
- `src/lib/components/source-messages-panel.svelte`
- `src/lib/components/analysis/source-context-panel.svelte`
- `src/lib/components/analysis/source-group-editor.svelte`
- `src/lib/components/analysis/trace-panel.svelte`
- `src/lib/components/analysis/workspace-main.svelte`
- `src/lib/components/analysis/workspace-rail.svelte`
- `src/routes/analysis/+page.svelte`

## Plan Files

Current plan files under `docs/superpowers/plans/`:

- `2026-05-09-youtube-sources.md`: split-plan index and acceptance checklist.
- `2026-05-09-youtube-sources-01-foundation.md`: schema, semantic item kinds, YouTube DTOs and URL parsing.
- `2026-05-09-youtube-sources-02-preview-and-add.md`: yt-dlp preview, source creation, playlist membership, frontend add flow.
- `2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`: source jobs, metadata sync, transcripts.
- `2026-05-09-youtube-sources-04-comments-and-analysis.md`: comments, provider-safe analysis groups, YouTube corpus loading, snapshots, trace refs.
- `2026-05-09-youtube-sources-05-auth-and-settings.md`: auth/cookies/settings.
- `2026-05-09-youtube-sources-06-ui-hardening-docs.md`: UI polish, hardening, documentation.

All six parts now have review-driven hardening. Parts 1-4 were already refined earlier; Parts 5-6 were refined in the latest thread.

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
   - YouTube corpus modes.
   - Playlist expansion for analysis.
   - Saved snapshots with YouTube metadata.
   - YouTube-aware trace refs.

5. Auth and Settings
   - Secure cookie storage.
   - Netscape cookie validation.
   - Auth-aware `yt-dlp` temp cookie files.
   - YouTube settings commands.
   - Settings UI and frontend wrappers.

6. UI Hardening and Docs
   - YouTube source cards/details.
   - Runtime/detail APIs.
   - Job and analysis controls.
   - Manual hardening matrix.
   - Documentation and final verification.

Each part is intended to leave the project in a consistent state after completion.

## Important Codebase Facts Found

- `src-tauri/src/sources/types.rs` already has YouTube-related source type constants/types, including `SourceType::Youtube` and `YOUTUBE_SOURCE_TYPE`.
- `src-tauri/src/sources/store.rs::source_record_from_row` currently decodes all `metadata_zstd` as Telegram metadata through `decode_source_metadata`; this must be made provider-safe before storing YouTube metadata.
- `src-tauri/src/sources/sync.rs::sync_source` is Telegram-only. UI must not call `sync_source` for YouTube.
- `src/lib/source-capabilities.ts` currently returns `canSync: source.sourceSubtype === "playlist"` for YouTube in the pre-implementation code; Part 2 keeps YouTube sync disabled and Part 3 later sets YouTube video/playlist `canSync` true.
- `src/lib/analysis-source-state.ts::sourceSyncDisabledReason` currently only knows Telegram account runtime checks. Part 6 adds YouTube `yt-dlp` runtime status.
- `src/lib/analysis-scope-state.ts::currentAnalysisScopeSummary` currently says `synced messages`; Part 6 changes wording to provider-neutral `synced items`.
- `src-tauri/src/analysis/corpus.rs::load_corpus_messages` currently loads all rows with `content_zstd IS NOT NULL` by source id and does not filter by `item_kind`. Part 4 requires provider-aware item filtering.
- `src-tauri/src/analysis/corpus.rs::preflight_analysis_run` currently has its own SQL path and must be updated to use the same provider-aware corpus loader as report execution.
- `src-tauri/src/analysis/store.rs::persist_run_snapshot` currently inserts only old snapshot columns and must bind `item_kind`, `source_type`, `source_subtype`, `metadata_zstd`.
- `src-tauri/src/analysis/trace.rs::normalize_ref` currently accepts only `s{digits}-i{digits}` and `s{digits}-m{digits}`. Part 3 extends it with optional timestamp suffixes for item refs.
- `src-tauri/src/takeout_import/state.rs` locks active Takeout jobs by `source_id`. Part 3 intentionally makes YouTube source jobs lock by `(source_id, job_type, related_source_id)`.
- `src/lib/analysis-state.ts` already defines `sourceActionPending` and `clearSourceActionPending`; Part 3 tells implementers to reuse them.
- `src/lib/components/analysis/trace-panel.svelte` currently displays trace details only. It does not load source items by `item_id`.
- `src/lib/components/analysis/source-group-editor.svelte` receives `AnalysisSourceOption[]`, not full `Source[]`; Part 4 adds `source_type` to `AnalysisSourceOption`.
- `src-tauri/Cargo.toml` currently has `tempfile = "3"` only under `[dev-dependencies]`; Part 3 moves it to runtime dependencies for captions.
- `src-tauri/src/secret_store.rs` has an existing `secret_ids_are_stable` test; Part 5 adds the YouTube cookies secret key to that test.
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

- `youtube_playlist_items.availability_status` has a `CHECK` constraint.
- Migration registration test checks all four new `analysis_run_messages` columns.
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

- `run_ytdlp` handles missing binary and non-zero exit codes.
- Non-zero exit code error mapping classifies auth/private/member/age/geo/unavailable/deleted/network/rate-limit where possible.
- Preview commands use `tokio::time::timeout`, currently 30s for preview.
- Playlist preview limits items with `--playlist-items 1-50`.
- Command-builder helpers:
  - `preview_video_args(canonical_url: &str) -> Vec<String>`
  - `preview_playlist_args(canonical_url: &str) -> Vec<String>`
- Test asserts `--playlist-items` and `1-50` are adjacent separate args, not `--playlist-items=1-50`.
- Fixture tests cover missing optionals, availability mapping, and non-zero fake runner errors.
- YouTube source upsert uses the partial unique index shape:
  - `ON CONFLICT(source_type, source_subtype, external_id) WHERE source_type = 'youtube' AND source_subtype = 'video'`
  - playlist equivalent for `source_subtype = 'playlist'`.
- Playlist item video source creation is race-safe.
- `add_youtube_source` persistence happens in a transaction after metadata fetch.
- `source_record_from_row` does not decode YouTube metadata as Telegram metadata.
- Frontend flow uses `youtube-source-add-panel.svelte`.
- YouTube `canSync` remains false in Part 2 for both video and playlist.
- Frontend API wrappers `previewYoutubeSource` and `addYoutubeSource` require explicit tests.

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
- UI `syncSelectedSource` branches:
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
- `upsert_youtube_transcript_item` updates text and returns item id.
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

- `--extractor-args` and `youtube:max_comments=<limit>` are separate command args.
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
- `list_analysis_sources` selects and groups by `sources.source_type`.
- Create/update group commands accept `source_type`.
- Existing groups rely on migration default `telegram`.
- In-memory `analysis_source_groups` test tables include `source_type TEXT NOT NULL DEFAULT 'telegram'`.
- Membership validation happens in `create_analysis_source_group` and `update_analysis_source_group`, after `ensure_sources_exist` and before write transaction.
- Helper `validate_group_source_type` is specified.
- `source-group-editor.svelte` needs group type select:
  - Telegram
  - YouTube
- `src/routes/analysis/+page.svelte` tracks `groupSourceType` and passes it to group editor.
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
- `load_corpus_messages` uses `CorpusLoadRequest`, not raw ids.
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

- `preflight_analysis_run` uses the exact same `CorpusLoadRequest` and `load_corpus_messages` path as execution.
- Do not keep a separate SQL path that counts all items by source id.
- `ReportRunInput` carries `corpus_request: CorpusLoadRequest`, not raw `source_ids: Vec<i64>`.

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
- YouTube transcript corpus loads segment-level messages from `youtube_transcript_segments`.
- Segment refs look like:
  - `s12-i400@754000ms`
- Segment metadata includes video id, canonical URL, title, channel, handle, caption language/kind, segment times, item kind.
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

## Part 5: Auth and Settings Current Content

Part 5 was reviewed and hardened in the latest session. It now covers:

### Task 1: Cookie Secret Boundary

- Create `src-tauri/src/youtube/cookies.rs`.
- Modify `src-tauri/src/youtube/ytdlp.rs`.
- Modify `src-tauri/src/secret_store.rs`.
- Modify `src-tauri/src/youtube/mod.rs`.
- Add stable helper:

```rust
pub(crate) fn youtube_default_cookies_secret() -> String {
    "youtube.auth.default.cookies".to_string()
}
```

- Extend `secret_store::tests::secret_ids_are_stable` with the YouTube key.
- Implement:
  - `read_youtube_cookies`
  - `save_youtube_cookies`
  - `clear_youtube_cookies`
- Raw cookie text is allowed only inside backend code that writes the temporary cookie file.
- Raw cookies must never appear in logs, IPC responses, job records, Tauri events, or `AppError.message`.
- Add minimal Netscape cookie validation:
  - empty/whitespace-only cookies are validation errors;
  - accept blank lines and comment/header lines;
  - `#HttpOnly_` rows are cookie rows after stripping that prefix;
  - require exactly 7 tab-separated fields;
  - require valid domain, TRUE/FALSE flags, path starting `/`, integer expiry, non-empty name;
  - cookie value may be empty;
  - do not reject non-YouTube domains.
- Add authenticated `yt-dlp` execution:
  - `YtdlpRunOptions { timeout: Duration, cookies: Option<String> }`;
  - `run_ytdlp_with_options`;
  - write cookies to temp Netscape cookie file;
  - pass `--cookies` and path as separate args;
  - temp file lives until process exits or times out;
  - existing `run_ytdlp(args)` keeps 30s preview timeout with no cookies.
- Timeout policy:
  - cookie injection never creates unbounded `yt-dlp` paths;
  - preview with/without cookies uses 30s;
  - sync callers pass their Part 3/4 timeout constants.
- Tests cover validation, empty cookies, sanitized errors, stable secret id, secure read/write/delete, and no raw cookies in command args.

### Task 2: YouTube Settings Commands

- Create `src-tauri/src/youtube/settings.rs`.
- Add `settings` module to `youtube/mod.rs`.
- Register commands in `src-tauri/src/lib.rs`.
- Create frontend files:
  - `src/lib/types/youtube.ts`
  - `src/lib/api/youtube-settings.ts`
  - `src/lib/api/youtube-settings.test.ts`
- Explicit app setting keys match migration defaults:
  - `youtube.auth.enabled`
  - `youtube.captions.preferred_language`
  - `youtube.sync.delay_between_requests_ms`
  - `youtube.sync.max_parallel_video_syncs`
  - `youtube.sync.max_parallel_comment_syncs`
  - `youtube.sync.pause_on_auth_challenge`
  - `youtube.sync.daily_soft_limit`
  - `youtube.sync.retry_backoff_ms`
  - `youtube.sync.stop_after_consecutive_failures`
- DTOs:
  - `YoutubeSettingsDto` with `#[serde(rename_all = "camelCase")]`
  - `YoutubeAuthStatusDto` with `#[serde(rename_all = "camelCase")]`
- Validation ranges:
  - `auth_enabled: bool`
  - `preferred_captions_language: "original"` or 2-32 ASCII letters/digits/hyphen/underscore chars
  - `delay_between_requests_ms: i64, 0..=60000`, 0 means no deliberate delay
  - `max_parallel_video_syncs: i64, 1..=4`
  - `max_parallel_comment_syncs: i64, 1..=2`
  - `pause_on_auth_challenge: bool`
  - `daily_soft_limit: i64, 0..=10000`, 0 means no daily soft limit
  - `retry_backoff_ms: i64, 0..=300000`
  - `stop_after_consecutive_failures: i64, 1..=50`
- Missing `app_settings` rows use defaults.
- Invalid stored bool/int values return validation errors naming the setting key.
- Write policy validates entire DTO before writing anything.
- Auth status messages are concrete:
  - `Auth disabled`
  - `Cookies stored`
  - `No cookies configured`
- Backend behavior branches on `enabled` and `has_cookies`, not free-form message text.
- Commands:
  - `get_youtube_settings`
  - `save_youtube_settings`
  - `get_youtube_auth_status`
  - `save_youtube_cookies`
  - `clear_youtube_auth`
- `save_youtube_cookies` rejects empty/invalid text, stores valid cookies, writes `youtube.auth.enabled = "true"`, and returns status.
- `clear_youtube_auth` deletes the secret, writes `youtube.auth.enabled = "false"`, and returns status.
- Empty cookie text is validation error, not alias for clear.
- Internal helpers are specified:
  - `default_youtube_settings`
  - `load_youtube_settings_from_pool`
  - `save_youtube_settings_to_pool`
  - `youtube_auth_status_from_state`
  - `save_youtube_cookies_to_state`
  - `clear_youtube_auth_in_state`
- Rust tests include:
  - defaults when app settings missing;
  - validation rejects out-of-range values;
  - cookie save enables auth and clear disables it;
  - `youtube_settings_serializes_with_camel_case_keys`, verifying frontend contract.
- Frontend wrappers live in dedicated `src/lib/api/youtube-settings.ts`, not `src/lib/api/sources.ts`.
- `RawYoutubeSettings` intentionally mirrors `YoutubeSettings` because the Rust DTO serializes camelCase.
- Vitest tests cover get/save settings, auth status, save cookies, clear auth.

### Task 3: Settings UI

- Create `src/lib/components/settings/youtube-settings-panel.svelte`.
- Mount it from `src/routes/settings/+page.svelte`.
- Load settings and auth status together.
- Auth status text:
  - `Auth disabled`
  - `Cookies stored`
  - `No cookies configured`
- Cookie UX:
  - hidden-by-default multiline textarea;
  - never render stored cookie text back into an input;
  - keep pasted cookies only in local component state before save;
  - clear textarea after successful save;
  - disable save-cookies button for empty/whitespace input;
  - backend still validates and rejects empty text.
- Controls:
  - enable YouTube auth
  - paste/update cookies
  - save cookies
  - cancel cookie edit
  - clear YouTube auth
  - preferred captions language
  - delay between requests
  - max parallel video syncs
  - max parallel comment syncs
  - pause on auth challenge
  - daily soft limit
  - retry backoff
  - stop after consecutive failures
  - save settings
- Frontend input constraints match backend validation.
- Verification includes `npm test -- youtube-settings` and `npm run check`.

## Part 6: UI Hardening and Docs Current Content

Part 6 was reviewed and hardened in the latest session. It now covers:

### Task 1: Source Cards, Runtime Status, and Workspace Detail

- Create backend runtime/detail modules:
  - `src-tauri/src/youtube/runtime.rs`
  - `src-tauri/src/youtube/detail.rs`
- Add `get_youtube_runtime_status`:
  - runs `yt-dlp --version`;
  - 5s timeout;
  - reports `ytdlp_available`, `ytdlp_version`, and message;
  - handles missing binary, failed check, and timeout.
- Add read-only YouTube detail DTOs:
  - `YoutubeContentSyncState`
  - `YoutubeContentStatusDto`
  - `YoutubeSourceSummaryDto`
  - `YoutubePlaylistMembershipDto`
  - `YoutubeVideoDetailDto`
  - `YoutubePlaylistItemDetailDto`
  - `YoutubePlaylistDetailDto`
- Matching TypeScript types live in `src/lib/types/youtube.ts`.
- Commands:
  - `list_youtube_source_summaries(source_ids: Vec<i64>)`
  - `get_youtube_video_detail(source_id: i64)`
  - `get_youtube_playlist_detail(source_id: i64)`
- Captions/comments status source of truth:
  - captions from `items.item_kind = 'youtube_transcript'`;
  - transcript segment count from `youtube_transcript_segments`;
  - comments from `items.item_kind = 'youtube_comment'`;
  - playlist rows from `youtube_playlist_items`;
  - source metadata decoded as YouTube metadata, not Telegram metadata.
- Playlist counts:
  - linked count: `video_source_id IS NOT NULL AND is_removed_from_playlist = 0`;
  - unavailable count: unavailable availability statuses or removed rows.
- Status policy:
  - transcript item count > 0 and segment count >= 0 -> synced;
  - known no-captions/unavailable availability -> unavailable;
  - otherwise not synced;
  - comment item count > 0 -> synced;
  - latest failed in-memory job can be displayed frontend-side.
- Add `src/lib/api/youtube-detail.ts` and `src/lib/api/youtube-detail.test.ts`.
- `sourceSyncDisabledReason` accepts optional `YoutubeRuntimeStatus`; YouTube sync is disabled with a useful message when `yt-dlp` unavailable.
- `currentAnalysisScopeSummary` changes `synced messages` to provider-neutral `synced items`.
- `source-row.svelte` and `workspace-rail.svelte` accept optional `YoutubeSourceSummary` and `YoutubeRuntimeStatus`.
- YouTube row display includes channel/handle, duration, published date, captions label, comments label, availability, direct YouTube link, playlist membership badge, linked child video count, unavailable count, etc.
- Define a single props contract for:
  - `src/lib/components/analysis/youtube-source-detail.svelte`
  - `src/lib/components/analysis/youtube-playlist-detail.svelte`
- Video detail tabs:
  - Overview
  - Transcript
  - Comments
  - Jobs
- Playlist detail shows ordered rows and row actions.
- `workspace-main.svelte` routes:
  - YouTube video -> `YoutubeSourceDetail`
  - YouTube playlist -> `YoutubePlaylistDetail`
  - otherwise -> `SourceContextPanel`

### Task 2: Job Controls and Analysis Controls

- Extend `src/lib/api/source-jobs.ts` with wrappers:
  - `syncYoutubePlaylistVideo`
  - `retryFailedYoutubePlaylistVideos`
  - `cancelSourceJob`
- Tests assert exact camelCase argument names.
- Video job controls map to exact commands:
  - sync metadata -> `syncYoutubeSource(source.id, { metadata: true, transcripts: false, comments: false })`
  - sync transcript -> `syncYoutubeSource(source.id, { metadata: false, transcripts: true, comments: false })`
  - sync comments -> `syncYoutubeSource(source.id, { metadata: false, transcripts: false, comments: true })`
  - cancel current job -> `cancelSourceJob(job.job_id)`
- Playlist job controls:
  - sync all playlist videos -> `syncYoutubeSource(playlist.id, { metadata: true, transcripts: true, comments: false })`
  - sync failed videos only -> `retryFailedYoutubePlaylistVideos(playlist.id, { metadata: false, transcripts: true, comments: false })`
  - cancel current playlist job -> `cancelSourceJob(job.job_id)`
- `sync all playlist videos` refreshes metadata, creates/links child video sources, and syncs transcripts. It does not fetch comments by default.
- Per-video playlist row actions:
  - open source -> select `videoSourceId`;
  - sync this video -> `syncYoutubePlaylistVideo(..., { metadata: true, transcripts: true, comments: false })`;
  - retry this video -> `syncYoutubePlaylistVideo(..., { metadata: false, transcripts: true, comments: false })`.
- `retry this video` is a single-row retry, not the aggregate retry command.
- Add YouTube corpus mode selector for YouTube scopes only:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`
- Telegram scopes send default `transcript_description` but backend ignores it.
- Hide Telegram topic controls for YouTube:
  - do not call `listSourceForumTopics` when `sourceCapabilities(source).hasTopics` is false;
  - reset `sourceTopics` and `selectedTopicKey` when selecting YouTube;
  - pass `topicFilter: null` for non-topic sources;
  - `SourceContextPanel` uses provider-aware `contentLabel`;
  - topic select only renders when `showTopicSelector` is true.

### Task 3: Manual Hardening Matrix

- Create/update `docs/youtube-manual-verification.md`.
- Manual matrix columns:
  - Scenario
  - Input URL or source fixture
  - Steps
  - Expected result
  - Result
  - Notes
- Scenarios with pass/fail criteria:
  - public video with manual captions;
  - public video with auto captions;
  - video with no captions;
  - Shorts URL;
  - live URL;
  - upcoming or live-ended source;
  - public playlist;
  - playlist with removed/private/unavailable entries;
  - direct video first, then playlist containing same video;
  - comments-heavy video with cancellation;
  - saved analysis run, then resync transcript in another caption language;
  - app restart during active YouTube job.
- Restart behavior is explicit:
  - YouTube source jobs are in memory and are not restored after app restart;
  - after restart, `list_source_jobs` returns no active job from previous process;
  - UI must not leave `syncingIds`, active badges, or disabled sync buttons stuck;
  - completed DB writes remain visible;
  - user can start a new sync after restart;
  - no MVP attempt to resume interrupted `yt-dlp`.
- Every deterministic defect fixed gets a unit/component/API/Rust test.
- Live-provider-only behavior gets a manual verification note.
- Task 3 verification includes:

```powershell
cd src-tauri
cargo test --lib
cargo clippy --all-targets -- -D warnings
cd ..
npm test
npm run check
npm run build
```

### Task 4: Documentation

- Update:
  - `README.md`
  - `docs/database-schema.md`
  - `docs/architecture-deep-dive.md`
  - `docs/backlog.md`
  - `docs/youtube-manual-verification.md`
- README notes:
  - `yt-dlp` must be installed and on PATH;
  - no YouTube audio/video binary downloads in MVP;
  - auth-gated content requires cookies configured in Settings;
  - YouTube sync jobs are in-memory and not resumed after restart.
- Schema docs:
  - `items.item_kind`
  - `youtube_playlist_items`
  - `youtube_transcript_segments`
  - `analysis_run_messages` YouTube snapshot columns
  - `analysis_source_groups.source_type`
  - YouTube partial unique indexes on sources
- Architecture docs:
  - `youtube/` Rust module;
  - `yt-dlp` adapter boundary and runtime check;
  - source jobs and in-memory restart behavior;
  - playlist expansion;
  - timestamp evidence refs;
  - secure cookie handling;
  - read-only YouTube detail/summary commands.
- Backlog clarified:
  - future item is YouTube-specific NotebookLM export enrichment, not the existing generic NotebookLM export;
  - speech-to-text fallback;
  - live chat ingest;
  - media-aware analysis only if a future setting explicitly allows media downloads;
  - persistent/resumable YouTube source jobs across restart.
- Do not move existing generic NotebookLM export to backlog; it already exists and must keep working for Telegram sources.

### Final Verification

Part 6 final verification now includes:

```powershell
cd src-tauri
cargo test --lib
cargo clippy --all-targets -- -D warnings
cd ..
npm test
npm run check
npm run build
```

Acceptance checklist includes:

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
- Use `TempDir` or equivalent RAII for caption/cookie temp files; do not leak temporary files.
- Keep raw cookie values out of logs, events, errors, command args, and IPC responses.
- Keep authenticated `yt-dlp` paths bounded by explicit timeouts.
- Keep old refs `sN-iM` and `sN-mM` working.
- Keep every plan part consistent after each task.
- YouTube jobs are in memory for the MVP; restart does not restore them.

## Verification Already Done During Planning

Only plan-level checks were run. No project tests were executed because application code has not been changed in this planning thread.

Plan checks run repeatedly:

```powershell
Select-String -Path docs\superpowers\plans\2026-05-09-youtube-sources-XX-*.md -Pattern 'TBD|TODO|implement later|fill in details|Add appropriate|Write tests for the above|Similar to Task'
```

Latest targeted checks:

- Part 5 placeholder scan returned no matches.
- Part 5 targeted scans confirmed:
  - `secret_ids_are_stable`
  - `validate_netscape_cookie_file`
  - `YtdlpRunOptions`
  - validation ranges
  - `youtube_settings_serializes_with_camel_case_keys`
  - `youtube-settings.test.ts`
  - hidden-by-default multiline textarea
- Part 6 placeholder scan returned no matches.
- Part 6 targeted scans confirmed:
  - `get_youtube_runtime_status`
  - `YoutubeVideoDetailDto`
  - single props contracts for YouTube detail components
  - `syncYoutubePlaylistVideo`
  - `retryFailedYoutubePlaylistVideos`
  - `synced items`
  - restart behavior
  - YouTube-specific NotebookLM export enrichment
  - `cargo clippy --all-targets -- -D warnings`
- A scan confirmed the old incorrect backlog wording that treated generic `NotebookLM export` itself as post-MVP is gone.

Earlier scans also checked for old conflicting strings such as:

- `sync_comments`
- `load_corpus_messages(pool, &input.source_ids`
- `sourceCatalog.sources`
- old `list_source_jobs() -> Vec<SourceJobRecord>`
- old transcript external id shape `transcript:<video_id>`

The latest scans for Parts 3 and 4 returned no matches for the targeted placeholder/conflict patterns.

## Current User Request

The user asked to overwrite `docs\session-context-2026-05-03.md` with all information needed to restore the current session context and to provide a commit message.

This file is the requested session context snapshot.

## Suggested Commit Message

```text
docs: refresh youtube sources planning context
```

## Recommended Next Steps

1. If more LLM review feedback arrives, keep using:
   - `superpowers:receiving-code-review`
   - `superpowers:writing-plans`
2. If planning is accepted, begin implementation from Part 1 and follow each plan task-by-task.
3. Before implementation, use the execution skills requested in the plan headers:
   - `superpowers:subagent-driven-development`, or
   - `superpowers:executing-plans`
4. At implementation time, verify each task with the exact commands in the corresponding plan part.
5. At final MVP verification, include `cargo clippy --all-targets -- -D warnings` in addition to Rust tests and frontend checks.
