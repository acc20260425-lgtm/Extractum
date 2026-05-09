# Session Context: YouTube Sources MVP

Updated: 2026-05-09
Workspace: `g:\Develop\Extractum`
Shell: PowerShell
Timezone: Europe/Minsk
User language: Russian
Branch: `main`
Checkout type: normal repository checkout, not a linked git worktree (`git rev-parse --git-dir` and `git rev-parse --git-common-dir` both returned `.git`)

## Purpose

This file is the restore point for the current Codex session. A future session should be able to continue YouTube Sources implementation without reading the whole chat history.

Extractum is a Tauri 2 + Svelte 5 app. The YouTube Sources MVP is split into six implementation plans under `docs/superpowers/plans/`.

## Current IDE Context

Latest IDE context reported by the user:

- Active file: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-02-preview-and-add.md`

## Current Git State

Before rewriting this file, the working tree was clean.

Latest commits:

```text
3707090 feat: ingest youtube comments
50ac301 test: update youtube sync disabled expectation
ee82cfc feat: ingest youtube transcripts
0c121c7 feat: sync youtube metadata
f3510c5 feat: add source job orchestration
85b35ba docs: refresh session context after youtube parts 1 and 2
4075dfe feat: add youtube source management flow
8ce5c46 feat: save youtube sources
667ee11 feat: preview youtube sources
7ffc25f feat: add youtube backend contracts
bef7178 feat: expose semantic item kinds
9406c1a feat: add youtube schema foundation
```

After this request, the expected only modified file is:

- `docs/session-context-2026-05-03.md`

Suggested commit message for this context refresh:

```text
docs: refresh session context after youtube part 4 task 1
```

## Session Rules And Workflow Notes

- Use Superpowers skills when relevant. In implementation turns, the session used:
  - `superpowers:using-superpowers`
  - `superpowers:using-git-worktrees`
  - `superpowers:executing-plans`
  - `superpowers:test-driven-development`
  - `superpowers:systematic-debugging`
  - `superpowers:verification-before-completion`
  - `superpowers:finishing-a-development-branch`
- Do not spawn subagents unless the user explicitly asks for delegation or parallel agent work. No subagents were used in this session.
- The user asked to do the next part one task at a time. The garbled phrase `Gjnjv ;ltim erfpfybq` was interpreted as keyboard-layout text meaning roughly "Then wait for instructions." Current operating rule: complete one task, commit it, stop and wait.
- The user previously allowed continuing directly on `main`; previous YouTube work has been committed on `main`.
- `git add` / `git commit` usually require escalation because sandboxed git writes to `.git/index.lock` can fail with permission errors. Use `sandbox_permissions: "require_escalated"` for those commands.
- Use `rg` / `rg --files` for search.
- Use `apply_patch` for manual edits.
- In PowerShell, prefer `npm.cmd` instead of `npm`.
- Svelte code must be checked with the Svelte MCP autofixer before sending code back or finalizing Svelte edits.
- Frontend Vitest has a sandbox cwd quirk. Targeted Vitest can fail before tests with `TypeError: Cannot read properties of undefined (reading 'config')`.
- Reliable full frontend command:

```powershell
node --trace-uncaught ./node_modules/vitest/vitest.mjs run --reporter dot
```

- Reliable targeted frontend command shape:

```powershell
node --trace-uncaught ./node_modules/vitest/vitest.mjs run src/lib/api/sources.test.ts --reporter dot
```

## yt-dlp Runtime Status

During this session, previewing a YouTube source failed with:

```text
Error previewing the YouTube source (validation): yt-dlp is not available on PATH
```

Investigation:

- `yt-dlp --version` failed in PowerShell because `yt-dlp` was not found.
- `python --version` works and returned Python 3.14.3.
- `python -m pip --version` works.
- `Python314\Scripts` and `Python314` are already in `PATH`.
- `winget`, `py`, and `pipx` were not available.
- A request to run `python -m pip install -U yt-dlp` with escalation was rejected by the user.

Current plan state:

- The MVP plan does not install `yt-dlp` from the UI.
- Part 6 adds runtime status detection so the UI can report missing `yt-dlp` before starting jobs.
- A future improvement could add UI copy with installation commands and a "recheck" button, but true in-app installation is not currently in the plan.

## Plans

Plan files:

- `docs/superpowers/plans/2026-05-09-youtube-sources.md`
- `docs/superpowers/plans/2026-05-09-youtube-sources-01-foundation.md`
- `docs/superpowers/plans/2026-05-09-youtube-sources-02-preview-and-add.md`
- `docs/superpowers/plans/2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`
- `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`
- `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`
- `docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md`

Completed implementation:

- Part 1: Schema and Contracts
- Part 2: Preview and Add Source
- Part 3: Jobs, Metadata, and Transcripts
- Part 4 Task 1: Comments Ingest

Current next implementation task:

- Part 4 Task 2: Provider-Safe Analysis Groups
- Plan file: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`

Before starting any new task, read the relevant plan task and use the relevant Superpowers skills again because skill use does not carry across turns.

## Verification State

Latest fresh verification after Part 4 Task 1:

```powershell
cd src-tauri
cargo test youtube::comments --lib
```

Result: 5 passed, 0 failed.

```powershell
cd src-tauri
cargo test youtube::jobs --lib
```

Result: 4 passed, 0 failed.

```powershell
cd src-tauri
cargo test sources::items --lib
```

Result: 8 passed, 0 failed.

```powershell
cd src-tauri
cargo test --lib
```

Result: 244 passed, 0 failed.

Latest full frontend verification after Part 3:

```powershell
node --trace-uncaught ./node_modules/vitest/vitest.mjs run --reporter dot
```

Result: 25 test files passed, 204 tests passed.

```powershell
npm.cmd run check
```

Result: `svelte-check found 0 errors and 0 warnings`.

Manual YouTube verification has not been run because it requires installed `yt-dlp`, network access, and manual UI interaction.

## UI Dependency Policy

The plans contain an explicit targeted UI dependency policy:

- Keep Extractum's local UI system as the visual source of truth.
- Do not replace existing shared components with a full external UI kit.
- Approved targeted additions:
  - `@lucide/svelte` for icons in new action buttons, status rows, and provider-specific controls.
  - `bits-ui` for accessible headless primitives in new components, especially tabs, tooltips, popovers, dropdown menus, switches, and complex dialog behavior.
  - `@tanstack/svelte-table` only for genuinely complex tables.
  - `paneforge` only for a new workspace shell that needs resizable panes.
- Guardrails:
  - Wrap external primitives behind local components in `src/lib/components/ui` when reused.
  - Keep styling in local CSS and existing design tokens.
  - Do not add Tailwind-first UI kits such as Skeleton, DaisyUI, or Flowbite.
  - Do not retrofit existing components just to use a new library.
  - Direct imports from new UI dependencies should stay inside new components or local wrappers created for this MVP.
  - Include `package.json` and `package-lock.json` changes in the same task commit that first uses a new UI dependency.

So far, no new frontend dependency was added for the YouTube MVP.

## Part 1 Completed: Foundation

Commits:

```text
9406c1a feat: add youtube schema foundation
bef7178 feat: expose semantic item kinds
7ffc25f feat: add youtube backend contracts
```

Implemented:

- Migration `src-tauri/migrations/16.sql`.
- Registered migration 16 in `src-tauri/src/migrations.rs`.
- `items.item_kind TEXT NOT NULL DEFAULT 'telegram_message'`.
- `idx_items_source_kind_published`.
- `youtube_playlist_items`.
- `youtube_transcript_segments`.
- `analysis_run_messages` columns:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- `analysis_source_groups.source_type TEXT NOT NULL DEFAULT 'telegram'`.
- `idx_analysis_source_groups_source_type`.
- YouTube partial unique indexes on `sources`:
  - `idx_sources_unique_youtube_video`
  - `idx_sources_unique_youtube_playlist`
- YouTube app setting defaults in `app_settings`.
- In-memory test schema updates in:
  - `src-tauri/src/sources/test_support.rs`
  - `src-tauri/src/analysis/corpus.rs`
- Item kind constants:
  - `ITEM_KIND_TELEGRAM_MESSAGE = "telegram_message"`
  - `ITEM_KIND_YOUTUBE_TRANSCRIPT = "youtube_transcript"`
  - `ITEM_KIND_YOUTUBE_COMMENT = "youtube_comment"`
- `StoredItemRow`, `ItemRecord`, `SourceItemInsert` now carry `item_kind`.
- Telegram sync and takeout import write Telegram items as `telegram_message`.
- Frontend `SourceItem` has `itemKind`.
- Backend YouTube contracts:
  - `src-tauri/src/youtube/mod.rs`
  - `src-tauri/src/youtube/dto.rs`
  - `src-tauri/src/youtube/errors.rs`
  - `src-tauri/src/youtube/url.rs`
- Direct Rust dependency `url = "2"`.
- URL parser supports:
  - `youtube.com/watch?v=...`
  - `youtu.be/...`
  - `youtube.com/playlist?list=...`
  - `youtube.com/shorts/...`
  - `youtube.com/live/...`
  - list parameter priority over video parameter.

Important YouTube wire values:

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

## Part 2 Completed: Preview And Add

Commits:

```text
667ee11 feat: preview youtube sources
8ce5c46 feat: save youtube sources
4075dfe feat: add youtube source management flow
```

Implemented:

- `src-tauri/src/youtube/ytdlp.rs`
- `src-tauri/src/youtube/metadata.rs`
- `src-tauri/src/youtube/preview.rs`
- `src-tauri/src/youtube/playlist.rs`
- `preview_youtube_source`
- `add_youtube_source`
- Both commands registered in `src-tauri/src/lib.rs`.

`yt-dlp` adapter:

```rust
pub(crate) struct YtdlpOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) async fn run_ytdlp(args: &[String]) -> crate::error::AppResult<YtdlpOutput>;
```

Implementation details:

- Uses `tokio::process::Command`.
- Passes args as separate values; no shell.
- Timeout: `YTDLP_PREVIEW_TIMEOUT = Duration::from_secs(30)`.
- Missing binary maps to `AppError::validation("yt-dlp is not available on PATH")`.
- Non-zero provider failures are classified in `youtube::errors::classify_ytdlp_failure`.

Preview command shapes:

```text
yt-dlp --dump-single-json --skip-download <canonical_url>
yt-dlp --dump-single-json --flat-playlist --playlist-items 1-50 --skip-download <canonical_playlist_url>
```

Playlist preview:

- Bounded to first 50 entries.
- Uses separate adjacent args `--playlist-items` and `1-50`.
- Adds a warning when preview only includes the first 50 playlist entries.

Source persistence:

- YouTube source rows:
  - `source_type = youtube`
  - `source_subtype = video | playlist`
  - `external_id = video_id | playlist_id`
  - `account_id = NULL`
  - `telegram_source_kind = NULL`
  - `metadata_zstd = normalized YouTube metadata JSON`
- Source upsert helpers:
  - `upsert_youtube_video_source`
  - `upsert_youtube_playlist_source`
- Playlist membership helper:
  - `upsert_playlist_items`
- Playlist membership rules:
  - Upsert by `(playlist_source_id, video_id)`.
  - Reuse canonical video source when it exists.
  - Create video source for available playlist entries.
  - Keep unavailable entries with `video_source_id = NULL`.
  - Mark missing rows as `removed_from_playlist`.
- Non-Telegram source rows skip Telegram metadata decoding for avatars.

Frontend:

- Added YouTube preview/add wrappers in `src/lib/api/sources.ts`.
- Added YouTube source types in `src/lib/types/sources.ts`.
- Added provider tabs in `source-management-dialog.svelte`:
  - Telegram
  - YouTube
- Created `src/lib/components/analysis/youtube-source-add-panel.svelte`.
- YouTube sync was intentionally disabled in capabilities in Part 2.

## Part 3 Completed: Jobs, Metadata, Transcripts

Commits:

```text
f3510c5 feat: add source job orchestration
0c121c7 feat: sync youtube metadata
ee82cfc feat: ingest youtube transcripts
50ac301 test: update youtube sync disabled expectation
```

### Part 3 Task 1: Source Job State

Created:

- `src-tauri/src/youtube/jobs.rs`
- `src/lib/api/source-jobs.ts`
- `src/lib/api/source-jobs.test.ts`

Modified:

- `src-tauri/src/youtube/mod.rs`
- `src-tauri/src/lib.rs`
- `src/lib/types/sources.ts`
- `src/routes/analysis/+page.svelte`

Backend:

- Added `SourceJobState` using `tokio::sync::Mutex`.
- Added statuses:
  - `queued`
  - `running`
  - `succeeded`
  - `failed`
  - `cancel_requested`
  - `cancelled`
- Added job types:
  - `youtube_video_metadata_sync`
  - `youtube_video_transcript_sync`
  - `youtube_video_comments_sync`
  - `youtube_video_full_sync`
  - `youtube_playlist_metadata_sync`
  - `youtube_playlist_full_sync`
  - `youtube_playlist_video_sync`
- Active lock key:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SourceJobKey {
    source_id: i64,
    job_type: SourceJobType,
    related_source_id: Option<i64>,
}
```

- Duplicate active `SourceJobKey` returns `AppError::conflict`.
- Event constant:
  - `sources://source-job`
- Commands:
  - `sync_youtube_source`
  - `sync_youtube_playlist_video`
  - `cancel_source_job`
  - `list_source_jobs`
  - `retry_failed_youtube_playlist_videos`
- `list_source_jobs` defaults to limit 100, clamps to 500, filters before truncating, sorts newest first.
- Retryable playlist rows:
  - `is_removed_from_playlist = 0`
  - `availability_status IN ('live_ended_transcript_pending','no_captions','unavailable_unknown')`

Frontend:

- Added source job TS types.
- Added wrappers:
  - `listSourceJobs`
  - `syncYoutubeSource`
  - `syncYoutubePlaylistVideo`
  - `retryFailedYoutubePlaylistVideos`
  - `cancelSourceJob`
  - `listenToSourceJobEvents`
- `src/routes/analysis/+page.svelte` now listens for source job events and keeps `sourceJobsBySource`.

### Part 3 Task 2: Metadata Sync

Modified:

- `src-tauri/src/youtube/metadata.rs`
- `src-tauri/src/youtube/jobs.rs`
- `src/lib/source-capabilities.ts`
- `src/lib/source-capabilities.test.ts`
- `src/routes/analysis/+page.svelte`

Backend:

- Added metadata fetch helpers:
  - `fetch_video_metadata`
  - `fetch_playlist_metadata`
  - `fetch_playlist_metadata_page`
  - `video_metadata_args`
  - `playlist_metadata_page_args`
- Playlist metadata page size:
  - `PLAYLIST_METADATA_PAGE_SIZE = 200`
- Video metadata command:

```text
yt-dlp --dump-single-json --skip-download <canonical_url>
```

- Playlist metadata page command:

```text
yt-dlp --dump-single-json --flat-playlist --skip-download --playlist-items <range> <playlist_url>
```

- Full playlist metadata sync loops page ranges, combines `items`, and rewrites raw `entries`.
- Metadata sync fetches via `yt-dlp` before opening the transaction.
- Playlist metadata sync refreshes playlist source, membership rows, and `last_synced_at`.
- Video metadata sync refreshes video source and `last_synced_at`.
- `sync_youtube_source` selects job type from source subtype and options.

Frontend:

- YouTube `canSync` is now true.
- YouTube sync button calls `syncYoutubeSource`.
- Telegram still uses `syncSource`.

### Part 3 Task 3: Transcript Ingest

Created:

- `src-tauri/src/youtube/captions.rs`

Modified:

- `src-tauri/src/youtube/mod.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/src/youtube/jobs.rs`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/sources/mod.rs`
- `src-tauri/src/analysis/trace.rs`

Cargo:

- `tempfile = "3"` is now a runtime dependency because captions need temp files.

Caption selection policy:

1. explicit override language: manual then auto
2. original language: manual then auto
3. app preferred language unless `original`: manual then auto
4. English manual
5. English auto
6. any manual
7. any auto

Current override source:

- `metadata.raw_metadata_json["caption_language_override"]`
- No UI/settings for this override yet.

Preferred language:

- `app_settings['youtube.captions.preferred_language']`
- default: `original`

Caption command:

```text
yt-dlp --skip-download --write-subs --write-auto-subs --sub-langs <lang> --sub-format json3/vtt --output <temp-template> <url>
```

Temp files:

- Uses `tempfile::TempDir`.
- Does not call `TempDir::into_path()`.
- RAII cleanup is preserved.

Transcript parsing:

- Prefer `json3`, fallback to `vtt`.
- Helpers:
  - `parse_json3_transcript`
  - `parse_vtt_transcript`
  - `transcript_external_id`
  - `replace_transcript_segments`
  - `caption_tracks_from_metadata`
  - `select_caption_track`
  - `caption_download_args`
  - `fetch_transcript_for_video`

Transcript item persistence:

- Added `upsert_youtube_transcript_item`.
- `items.item_kind = youtube_transcript`
- `items.external_id = transcript:<video_id>:<language-or-und>:<manual|auto|unknown>`
- `items.content_kind = text_only`
- Raw transcript payload is compressed JSON.
- Transcript segments are replaced via `DELETE + INSERT` in the same transaction.

Jobs transcript integration:

- `run_source_job_steps` uses `related_source_id.unwrap_or(source_id)`.
- If `options.transcripts`, syncs transcript.
- If video metadata is missing, transcript sync first runs metadata sync and reloads source.
- Playlist retry job now calls transcript sync for retryable child video source rows.

Trace refs:

- `normalize_ref` accepts:
  - `s12-i400@754000ms`
  - `s12-i400@754000-790000ms`
  - bracketed forms
- Keeps:
  - `s12-i845`
  - `s12-m845`
- Rejects:
  - timestamp suffix on `-m`
  - reversed ranges
  - non-digit components

## Part 4 In Progress: Comments And Analysis

Plan file:

- `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`

Completed:

- Task 1: Comments Ingest

Next:

- Task 2: Provider-Safe Analysis Groups

### Part 4 Task 1 Completed: Comments Ingest

Commit:

```text
3707090 feat: ingest youtube comments
```

Files changed:

- Created `src-tauri/src/youtube/comments.rs`
- Modified `src-tauri/src/youtube/mod.rs`
- Modified `src-tauri/src/youtube/dto.rs`
- Modified `src-tauri/src/youtube/jobs.rs`
- Modified `src-tauri/src/sources/items.rs`
- Modified `src-tauri/src/sources/mod.rs`

Backend details:

- Added `comments` module to `src-tauri/src/youtube/mod.rs`.
- Added runtime constant:

```rust
pub(crate) const DEFAULT_MAX_COMMENTS_PER_VIDEO: usize = 1_000;
```

- Added comments fetch command builder:

```rust
pub(crate) fn comments_fetch_args(canonical_url: &str, max_comments: usize) -> Vec<String>
```

Command shape:

```text
yt-dlp --dump-single-json --write-comments --skip-download --extractor-args youtube:max_comments=<limit> <video_url>
```

Important:

- `--extractor-args` and `youtube:max_comments=<limit>` are separate adjacent args.
- No custom YouTube comment pager was added.
- Rust enforces the same limit after parsing:
  - `raw_comments.into_iter().take(max_comments)`
  - warning: `Comment sync truncated at {max_comments} comments.`

Normalization:

- Added `YoutubeCommentsIngest`.
- Added `normalize_comments_from_ytdlp`.
- Added `comment_published_at`.
- Flattens top-level comments and replies into `YoutubeComment`.
- Reads timestamps as numeric or string.
- Uses video timestamp as fallback, then sync start timestamp if video timestamp is absent.
- Adds a warning for comments/replies with missing or invalid timestamps.
- Skips comments without id or text.
- Captures:
  - comment id
  - parent comment id
  - reply state
  - author
  - author channel id
  - author channel url
  - published timestamp
  - text
  - like count
  - pinned state
  - creator heart/reaction state
  - raw provider payload

DTO:

- `YoutubeComment.published_at` changed from `Option<String>` to `i64`, matching persisted item timestamps.

Persistence:

- Added `upsert_youtube_comment_item`.
- Re-exported it from `src-tauri/src/sources/mod.rs`.
- SQL uses:
  - `items.item_kind = youtube_comment`
  - `items.external_id = comment:<comment_id>`
  - `items.content_kind = text_only`
  - `items.has_media = 0`
  - `items.reaction_count = like_count`
- `ON CONFLICT(source_id, external_id) DO UPDATE` updates existing comment rows rather than duplicating.
- Raw data stores the serialized normalized `YoutubeComment`, including raw provider payload.

Jobs:

- `run_source_job_steps` now runs comments only when `YoutubeSyncOptions.comments = true`.
- Old deferred warning was removed.
- Comments sync:
  - requires a YouTube video source;
  - runs metadata sync first if video metadata is missing;
  - fetches comments with `DEFAULT_MAX_COMMENTS_PER_VIDEO`;
  - upserts every comment in one DB transaction;
  - updates `last_synced_at`;
  - returns warnings from normalization/truncation.

Tests added:

- `youtube::comments`:
  - bounded `yt-dlp` args
  - default max comments constant
  - timestamp numeric/string/fallback
  - top-level comments + replies flattening
  - timestamp fallback warnings
  - truncation warning
- `sources::items`:
  - `upsert_youtube_comment_item` updates existing text and reaction count, returns the same id.
- `youtube::jobs`:
  - comments-only video sync maps to `YoutubeVideoCommentsSync`;
  - full video sync remains `YoutubeVideoFullSync`.

Verification:

```powershell
cd src-tauri
cargo test youtube::comments --lib
cargo test youtube::jobs --lib
cargo test sources::items --lib
cargo test --lib
```

Results:

- `youtube::comments`: 5 passed.
- `youtube::jobs`: 4 passed.
- `sources::items`: 8 passed.
- Full `cargo test --lib`: 244 passed, 0 failed.

### Part 4 Task 2 Next: Provider-Safe Analysis Groups

Files listed in plan:

- `src-tauri/src/analysis/models.rs`
- `src-tauri/src/analysis/mod.rs`
- `src-tauri/src/analysis/groups.rs`
- `src-tauri/src/analysis/store.rs`
- `src/lib/types/analysis.ts`
- `src/lib/api/analysis-source-groups.ts`
- `src/lib/components/analysis/source-group-editor.svelte`
- `src/routes/analysis/+page.svelte`

Task 2 core requirements:

- Add `source_type` to `AnalysisSourceGroup` and `AnalysisSourceGroupRow`.
- Add `source_type` to `AnalysisSourceOption`.
- Update `list_analysis_sources` query to select `sources.source_type`.
- TypeScript:
  - `AnalysisGroupSourceType = "telegram" | "youtube"`
  - `AnalysisSourceGroup.source_type`
  - `AnalysisSourceOption.source_type`
- Update create/update commands to accept `source_type`.
- Existing groups use migration default `telegram`.
- Update in-memory `analysis_source_groups` test tables to include `source_type TEXT NOT NULL DEFAULT 'telegram'`.
- Validate group membership:
  - Telegram group -> only `source_type = telegram`
  - YouTube group -> only `source_type = youtube`
- Validation should run after `ensure_sources_exist` and before opening the write transaction.
- Add helper:

```rust
async fn validate_group_source_type(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    group_source_type: &str,
    source_ids: &[i64],
) -> AppResult<()>
```

- Create SQL should include:

```sql
INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)
```

- Update SQL should include:

```sql
UPDATE analysis_source_groups SET name = ?, source_type = ?, updated_at = ?
```

Frontend Task 2:

- Add group type selector:
  - Telegram
  - YouTube
- Filter candidate sources by selected group provider.
- In `src/routes/analysis/+page.svelte`, maintain:

```ts
let groupSourceType = $state<AnalysisGroupSourceType>("telegram");
```

- When selecting existing group, copy `selectedGroup.source_type`.
- When starting new group, default to `telegram`.
- Save calls pass `sourceType: groupSourceType`.

Planned verification:

```powershell
cd src-tauri
cargo test analysis::groups analysis::store --lib
cd ..
npm test -- analysis-source-groups
npm run check
```

Note: The plan's cargo command with two filters may need to be split because local `cargo test` accepts one test filter. Use separate commands if needed:

```powershell
cd src-tauri
cargo test analysis::groups --lib
cargo test analysis::store --lib
```

Expected commit:

```text
feat: enforce provider-specific analysis groups
```

## Part 4 Later Tasks Reminder

### Task 3: YouTube Corpus Loading And Playlist Expansion

Key requirements:

- Add YouTube corpus modes:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`
- Add `youtube_corpus_mode` to `start_analysis_report`.
- Resolve source IDs:
  - single video -> that video source
  - single playlist -> linked child `video_source_id` rows
  - YouTube group -> direct videos plus expanded playlist children
- Exclude `video_source_id IS NULL` and removed playlist rows.
- If expansion yields zero source ids, return:

```text
No linked YouTube videos are available for analysis in this scope
```

- Load corpus by item kind:
  - Telegram -> `telegram_message`
  - YouTube transcript -> `youtube_transcript`
  - YouTube comments -> `youtube_comment`
- Add synthetic description messages only when mode includes description.
- Synthetic description ref:

```text
s{source_id}-i0
```

- `item_id = 0` is synthetic and must not be used as a DB row id.

### Task 4: Timestamp Trace Refs And Run Snapshots

Key requirements:

- Extend `CorpusMessage` with:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- Persist these fields into `analysis_run_messages`.
- For YouTube transcript corpus, load segment-level messages from `youtube_transcript_segments` rather than sending one giant transcript item.
- Segment ref shape:

```text
s12-i400@754000ms
```

- Extend `AnalysisTraceRef` with:
  - `youtube_url`
  - `youtube_timestamp_seconds`
  - `youtube_display_label`
  - `is_synthetic`
- Resolve timestamp refs first, then fallback to base item refs.
- Mark `youtube_description` / `item_id = 0` refs as synthetic.
- Keep old `s12-i400` and `s12-m400` refs working.

## Part 5 Reminder: Auth And Settings

Plan file:

- `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`

Key reminders:

- Cookie secret key helper:

```rust
pub(crate) fn youtube_default_cookies_secret() -> String {
    "youtube.auth.default.cookies".to_string()
}
```

- Raw cookies are allowed only inside backend code that writes temporary cookie files.
- Raw cookies must never appear in logs, IPC responses, job records, events, or `AppError.message`.
- Validate Netscape cookie rows:
  - accept blank/comment/header lines;
  - strip `#HttpOnly_` prefix for cookie rows;
  - require exactly 7 tab-separated fields;
  - cookie value may be empty;
  - do not reject non-YouTube domains.
- Existing `run_ytdlp(args)` keeps 30s preview timeout without cookies.
- Authenticated paths use explicit bounded timeouts.
- Planned commands:
  - `get_youtube_settings`
  - `save_youtube_settings`
  - `get_youtube_auth_status`
  - `save_youtube_cookies`
  - `clear_youtube_auth`
- Settings UI must never render stored cookie text back into an input.

## Part 6 Reminder: UI Hardening And Docs

Plan file:

- `docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md`

Key reminders:

- Add runtime/detail APIs:
  - `src-tauri/src/youtube/runtime.rs`
  - `src-tauri/src/youtube/detail.rs`
- `get_youtube_runtime_status` runs `yt-dlp --version` with a 5s timeout.
- YouTube source detail components planned:
  - `src/lib/components/analysis/youtube-source-detail.svelte`
  - `src/lib/components/analysis/youtube-playlist-detail.svelte`
- Source summaries should use provider-neutral `synced items`, not `synced messages`.
- Hide Telegram topic controls for YouTube sources.
- Documentation updates:
  - `docs/youtube-manual-verification.md`
  - `README.md`
  - `docs/database-schema.md`
  - `docs/architecture-deep-dive.md`
  - `docs/backlog.md`

## Important Gotchas

- Do not call Telegram-only `sync_source` for YouTube. Use `sync_youtube_source`.
- Do not decode YouTube `metadata_zstd` with Telegram `decode_source_metadata`.
- Do not fetch full playlists in one unbounded preview call.
- Do not fetch comments without `max_comments`.
- Do not rely on `items.content_zstd IS NOT NULL` alone for analysis corpus.
- Do not mix Telegram and YouTube sources in one analysis group.
- Do not include YouTube playlist rows with `video_source_id IS NULL` in corpus.
- Do not use `item_id = 0` as a database item id.
- Use `TempDir` or equivalent RAII for caption/cookie temp files.
- Keep raw cookie values out of logs, events, errors, command args, and IPC responses.
- Keep authenticated `yt-dlp` paths bounded by explicit timeouts.
- Keep old refs `sN-iM` and `sN-mM` working.
- YouTube jobs are in memory for the MVP; restart does not restore them.
- New UI dependencies must stay inside new components or new local wrappers created for this MVP.
- Do not add a full Tailwind-first UI kit.

## Recommended Next Steps

1. Commit this context refresh with:

```text
docs: refresh session context after youtube part 4 task 1
```

2. If the user asks to continue, start Part 4 Task 2 from:

```text
docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md
```

3. Follow the one-task cadence:
   - read the task critically;
   - use TDD;
   - implement only that task;
   - run specified targeted verification;
   - run broader verification when a shared DTO/schema changes;
   - commit the task;
   - stop and wait.
