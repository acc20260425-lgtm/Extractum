# Session Context: YouTube Sources Implementation

Updated: 2026-05-09
Workspace: `g:\Develop\Extractum`
Shell: PowerShell
User language: Russian
Branch: `main`
Checkout type: normal repository checkout, not a linked git worktree (`git rev-parse --git-dir` and `git rev-parse --git-common-dir` both returned `.git`)

## Purpose

This file is the restore point for the current Codex session. A future session should be able to continue YouTube Sources implementation without reading the whole chat history.

The project is Extractum, a Tauri 2 + Svelte 5 app. The YouTube Sources MVP is split into six implementation plans under `docs/superpowers/plans/`.

## Current IDE Context

Reported by the IDE in the latest user message:

- Active file: `docs/superpowers/plans/2026-05-09-youtube-sources-02-preview-and-add.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-02-preview-and-add.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-01-foundation.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md`

## Current Git State

Before rewriting this file, the working tree was clean.

Recent commits:

```text
4075dfe feat: add youtube source management flow
8ce5c46 feat: save youtube sources
667ee11 feat: preview youtube sources
7ffc25f feat: add youtube backend contracts
bef7178 feat: expose semantic item kinds
9406c1a feat: add youtube schema foundation
db193e8 docs: refresh session context
fbad82f docs: add targeted ui dependency policy
6558a38 docs: tighten youtube sources implementation plans
450295c docs: refresh youtube sources planning context
5857e6f docs: capture youtube sources planning session context
180d783 docs: move WebView OOM report to docs root
```

After this request, the expected only modified file is:

- `docs/session-context-2026-05-03.md`

Suggested commit message for this documentation refresh:

```text
docs: refresh session context after youtube parts 1 and 2
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
- Do not spawn subagents unless the user explicitly asks for delegation or parallel agent work. This session did not use subagents.
- The user explicitly asked to continue implementation on the next Part after Part 1, so implementation continued directly on `main`.
- `git add` initially failed under sandbox with `Unable to create ... .git/index.lock: Permission denied`; use escalation for `git add` / `git commit` if that happens again.
- In PowerShell, prefer `npm.cmd` instead of `npm` because direct `npm` can hit script execution policy.
- Frontend Vitest has an environment quirk in this Codex sandbox:
  - `npm.cmd test` or `node ./node_modules/vitest/vitest.mjs ...` can sometimes run with cwd shown as `C:/Users/CodexSandboxOffline/.codex/.sandbox/cwd/...` and fail before tests with `TypeError: Cannot read properties of undefined (reading 'config')`.
  - The reliable full frontend command used in this session was:

```powershell
node --trace-uncaught ./node_modules/vitest/vitest.mjs run --reporter dot
```

  - The reliable targeted frontend command shape was:

```powershell
node --trace-uncaught ./node_modules/vitest/vitest.mjs run src/lib/api/sources.test.ts src/lib/source-capabilities.test.ts --reporter dot
```

## Verification State

Fresh verification after Part 2:

```text
cd src-tauri
cargo test --lib
```

Result: 223 passed, 0 failed.

```text
node --trace-uncaught ./node_modules/vitest/vitest.mjs run --reporter dot
```

Result: 24 test files passed, 201 tests passed.

```text
npm.cmd run check
```

Result: `svelte-check found 0 errors and 0 warnings`.

Manual YouTube verification was not run because it requires installed `yt-dlp`, network access, and manual UI interaction.

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

- Part 1: Foundation
- Part 2: Preview and Add

Likely next implementation request:

- Part 3: `docs/superpowers/plans/2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`

Before starting Part 3, read the plan file and use the relevant Superpowers skills again because skill use does not carry across turns.

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
  - Wrap external primitives behind local components in `src/lib/components/ui` when they will be reused.
  - Keep styling in local CSS and existing design tokens.
  - Do not add Tailwind-first UI kits such as Skeleton, DaisyUI, or Flowbite for this MVP.
  - Do not retrofit existing components just to use a new library.
  - Existing components may render new YouTube child components, but direct imports from new UI dependencies should stay inside new components or new local wrappers.
  - Include `package.json` and `package-lock.json` changes in the same task commit that first uses a new UI dependency.

Part 2 did not add new frontend dependencies.

## Part 1 Completed: Foundation

Commits:

```text
9406c1a feat: add youtube schema foundation
bef7178 feat: expose semantic item kinds
7ffc25f feat: add youtube backend contracts
```

### Part 1 Task 1: Schema Foundation

Implemented:

- Added migration `src-tauri/migrations/16.sql`.
- Registered migration 16 in `src-tauri/src/migrations.rs`.
- Added `items.item_kind TEXT NOT NULL DEFAULT 'telegram_message'`.
- Added `idx_items_source_kind_published`.
- Created `youtube_playlist_items`.
- Created `youtube_transcript_segments`.
- Extended `analysis_run_messages` with:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- Added `analysis_source_groups.source_type TEXT NOT NULL DEFAULT 'telegram'`.
- Added `idx_analysis_source_groups_source_type`.
- Added partial unique indexes:
  - `idx_sources_unique_youtube_video`
  - `idx_sources_unique_youtube_playlist`
- Inserted YouTube app settings defaults in `app_settings`.
- Updated in-memory schemas in:
  - `src-tauri/src/sources/test_support.rs`
  - `src-tauri/src/analysis/corpus.rs`

Tests added/used:

- `migrations::tests::includes_youtube_source_foundation_migration`
- `sources::test_support`
- `analysis::corpus`

### Part 1 Task 2: Semantic Item Kinds

Implemented:

- Added constants in `src-tauri/src/sources/types.rs`:
  - `ITEM_KIND_TELEGRAM_MESSAGE = "telegram_message"`
  - `ITEM_KIND_YOUTUBE_TRANSCRIPT = "youtube_transcript"`
  - `ITEM_KIND_YOUTUBE_COMMENT = "youtube_comment"`
- Added `item_kind` to:
  - `StoredItemRow`
  - `ItemRecord`
  - `SourceItemInsert`
- `insert_source_item` now writes `item_kind`.
- Item query SELECT includes `items.item_kind`.
- Telegram sync and takeout import insert Telegram items as `telegram_message`.
- `src-tauri/src/sources/mod.rs` re-exports `ITEM_KIND_TELEGRAM_MESSAGE` for takeout import.
- Frontend:
  - `SourceItem` has `itemKind`.
  - `RawSourceItem` has `item_kind`.
  - `mapSourceItem` maps `itemKind`.

Tests added/used:

- `sources::items`
- `sources::sync`
- `sources::items::query`
- `sources::types`
- `src/lib/api/sources.test.ts`

### Part 1 Task 3: Backend YouTube Contracts

Implemented:

- Added direct dependency `url = "2"` in `src-tauri/Cargo.toml`.
- Added `mod youtube;` in `src-tauri/src/lib.rs`.
- Created:
  - `src-tauri/src/youtube/mod.rs`
  - `src-tauri/src/youtube/dto.rs`
  - `src-tauri/src/youtube/errors.rs`
  - `src-tauri/src/youtube/url.rs`
- DTOs include:
  - `YoutubeAvailabilityStatus`
  - `YoutubePreviewKind`
  - `YoutubePreview`
  - `YoutubeCaptionsEstimate`
  - `YoutubeVideoForm`
  - `YoutubeChapter`
  - `YoutubeVideoMetadata`
  - `YoutubePlaylistMetadata`
  - `YoutubePlaylistItemMetadata`
  - `YoutubeCaptionTrackKind`
  - `YoutubeCaptionTrack`
  - `YoutubeTranscriptSegment`
  - `YoutubeTranscript`
  - `YoutubeComment`
- URL parser supports:
  - `youtube.com/watch?v=...`
  - `youtu.be/...`
  - `youtube.com/playlist?list=...`
  - `youtube.com/shorts/...`
  - `youtube.com/live/...`
  - list parameter priority over video parameter.
- Supported hosts:
  - `youtu.be`
  - `youtube.com`
  - any `.youtube.com` subdomain.

Important enum wire values:

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

Tests added/used:

- `youtube::url`
- `youtube::dto`
- `youtube::errors`

## Part 2 Completed: Preview And Add

Commits:

```text
667ee11 feat: preview youtube sources
8ce5c46 feat: save youtube sources
4075dfe feat: add youtube source management flow
```

### Part 2 Task 1: yt-dlp Adapter And Preview Command

Implemented files:

- `src-tauri/src/youtube/ytdlp.rs`
- `src-tauri/src/youtube/metadata.rs`
- `src-tauri/src/youtube/preview.rs`

Modified:

- `src-tauri/src/youtube/mod.rs`
- `src-tauri/src/youtube/errors.rs`
- `src-tauri/src/lib.rs`

Important contracts:

```rust
pub(crate) struct YtdlpOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) async fn run_ytdlp(args: &[String]) -> crate::error::AppResult<YtdlpOutput>;
```

Implementation details:

- Uses `tokio::process::Command`.
- Passes args as separate values; no shell invocation.
- Defines `YTDLP_PREVIEW_TIMEOUT = Duration::from_secs(30)`.
- Wraps command execution in `tokio::time::timeout`.
- Missing binary maps to validation error:
  - `yt-dlp is not available on PATH`
- Non-zero provider failures are classified in `youtube::errors::classify_ytdlp_failure`:
  - private/auth/members/age/geo text -> `AppError::auth`
  - unavailable/deleted/not found/removed -> `AppError::not_found`
  - timeout/network/rate-limit -> `AppError::network`
  - other provider failures -> `AppError::validation`
- Fixed false positives while classifying:
  - `downloading` must not match `login`
  - `webpage` must not match `age`

Command builders:

```rust
pub(crate) fn preview_video_args(canonical_url: &str) -> Vec<String>;
pub(crate) fn preview_playlist_args(canonical_url: &str) -> Vec<String>;
```

Command shapes:

```text
yt-dlp --dump-single-json --skip-download <canonical_url>
yt-dlp --dump-single-json --flat-playlist --playlist-items 1-50 --skip-download <canonical_playlist_url>
```

Playlist preview:

- Does not enumerate unbounded playlists.
- Uses adjacent separate args `--playlist-items` and `1-50`, not `--playlist-items=1-50`.
- Adds warning if preview only includes the first 50 playlist entries.

Metadata normalizers:

- `video_metadata_from_ytdlp`
- `playlist_metadata_from_ytdlp`
- `video_preview_from_metadata`
- `playlist_preview_from_metadata`

Preview command:

```rust
#[tauri::command]
pub async fn preview_youtube_source(url: String) -> AppResult<YoutubePreview>
```

Registered in `tauri::generate_handler!`.

Tests added/used:

- `youtube::metadata`
- `youtube::preview`
- `youtube::errors`
- `youtube::ytdlp`
- Full `cargo test youtube --lib`

### Part 2 Task 2: Source Creation And Playlist Membership

Implemented:

- Created `src-tauri/src/youtube/playlist.rs`.
- Added source upsert helpers in `src-tauri/src/sources/store.rs`:

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

- Upserts use the migration 16 partial unique indexes:
  - `ON CONFLICT(source_type, source_subtype, external_id) WHERE source_type = 'youtube' AND source_subtype = 'video'`
  - `ON CONFLICT(source_type, source_subtype, external_id) WHERE source_type = 'youtube' AND source_subtype = 'playlist'`
- Persisted YouTube sources use:
  - `source_type = youtube`
  - `source_subtype = video | playlist`
  - `account_id = NULL`
  - `telegram_source_kind = NULL`
  - `external_id = video_id | playlist_id`
  - `metadata_zstd = normalized YouTube metadata JSON`
- Added `load_source_record` helper to return saved `SourceRecord`.
- Added `upsert_playlist_items` in `src-tauri/src/youtube/playlist.rs`.
- Playlist rules implemented:
  - Upsert by `(playlist_source_id, video_id)`.
  - Reuse canonical video source when it already exists.
  - Create video source for available playlist entries.
  - Keep unavailable/auth/private entries as membership rows with `video_source_id = NULL`.
  - Mark missing rows as `is_removed_from_playlist = 1` and `availability_status = 'removed_from_playlist'`.
- Added command:

```rust
#[tauri::command]
pub async fn add_youtube_source(handle: AppHandle, url: String) -> AppResult<SourceRecord>
```

- `add_youtube_source` behavior:
  - Parse URL.
  - Fetch fresh metadata via adapter before opening DB transaction.
  - Open one transaction for persistence.
  - Save video or playlist source inside transaction.
  - Save playlist membership rows inside same transaction.
  - Commit only after all persistence succeeds.
  - Return saved `SourceRecord`.
  - Does not create `items` rows.
- Registered `add_youtube_source` in `tauri::generate_handler!`.

Provider-safe source record metadata:

- Added helper:

```rust
fn source_avatar_cache_key_from_row(row: &SourceRecordRow) -> AppResult<Option<String>>
```

- Non-Telegram rows skip Telegram metadata decoding and return `None` for avatar cache key.
- YouTube rows return `avatar_data_url = None` in this part.

Tests added/used:

- `youtube::playlist::tests::upsert_playlist_items_reuses_existing_video_source_and_keeps_unavailable_null`
- `youtube::playlist::tests::upsert_playlist_items_marks_missing_rows_removed`
- `sources::store::tests::avatar_cache_key_skips_non_telegram_metadata`
- `cargo test youtube::playlist --lib`
- `cargo test sources::store --lib`
- `cargo test youtube --lib`

### Part 2 Task 3: Frontend Add Flow

Modified:

- `src/lib/types/sources.ts`
- `src/lib/api/sources.ts`
- `src/lib/api/sources.test.ts`
- `src/lib/source-capabilities.ts`
- `src/lib/source-capabilities.test.ts`
- `src/lib/components/analysis/source-management-dialog.svelte`

Created:

- `src/lib/components/analysis/youtube-source-add-panel.svelte`

Frontend types added:

- `YoutubePreviewKind`
- `YoutubeAvailabilityStatus`
- `YoutubeCaptionsEstimate`
- `YoutubePreview`

API wrappers added:

```ts
export function previewYoutubeSource(url: string)
export function addYoutubeSource(url: string)
```

Mapping:

- `RawYoutubePreview` snake_case fields map to `YoutubePreview` camelCase.
- `previewYoutubeSource` invokes `preview_youtube_source` with `{ url }`.
- `addYoutubeSource` invokes `add_youtube_source` with `{ url }`.

Capabilities:

- `sourceCapabilities` now returns `canSync: false` for all YouTube sources in Part 2.
- Playlist `canSync` was intentionally changed from `true` to `false` until Part 3 wires jobs.

Dialog/UI:

- `source-management-dialog.svelte` now has provider tabs:
  - Telegram
  - YouTube
- Existing Telegram account/dialog/manual add flow remains under Telegram tab.
- YouTube state/UI lives in `youtube-source-add-panel.svelte`.
- Switching tabs preserves in-progress typed values because panels keep local state while mounted in the dialog selection flow.
- YouTube panel state:

```ts
let youtubeUrl = $state("");
let youtubePreview = $state<YoutubePreview | null>(null);
let previewingYoutube = $state(false);
let addingYoutube = $state(false);
let youtubeStatus = $state("");
let previewedUrl = $state("");
```

State behavior:

- Changing URL clears existing preview if it no longer matches `previewedUrl`.
- Preview errors write to `youtubeStatus`.
- Add errors write to `youtubeStatus` without clearing valid preview.
- Successful add calls `onSourcesChanged(source.id)`, clears URL/preview, and sends parent status.
- Telegram local status and YouTube status are separate.

Svelte validation:

- Ran Svelte autofixer on:
  - `youtube-source-add-panel.svelte`
  - `source-management-dialog.svelte`
- Both returned no issues.

Tests added/used:

- `src/lib/api/sources.test.ts`
  - previews YouTube sources with URL argument.
  - adds YouTube sources with URL argument.
- `src/lib/source-capabilities.test.ts`
  - YouTube playlist `canSync` is false.
- Targeted frontend tests:

```powershell
node --trace-uncaught ./node_modules/vitest/vitest.mjs run src/lib/api/sources.test.ts src/lib/source-capabilities.test.ts --reporter dot
```

Result: 12 passed.

## Important Codebase Facts After Parts 1 And 2

- `src-tauri/src/youtube` now exists with:
  - `dto.rs`
  - `errors.rs`
  - `metadata.rs`
  - `mod.rs`
  - `playlist.rs`
  - `preview.rs`
  - `url.rs`
  - `ytdlp.rs`
- `src-tauri/src/lib.rs` imports:

```rust
use youtube::preview::{add_youtube_source, preview_youtube_source};
```

- Both commands are registered in `tauri::generate_handler!`.
- `src-tauri/Cargo.toml` has direct `url = "2"`.
- No runtime `tempfile` dependency has been added yet. Part 3 will need to move/add `tempfile = "3"` to runtime dependencies for captions if the plan still says so.
- `src/lib/components/analysis/youtube-source-add-panel.svelte` uses only local UI components, no external UI dependencies.
- YouTube preview/add currently rely on `yt-dlp` being installed on PATH.
- YouTube source creation stores metadata and playlist membership only; it does not create transcript/comment `items` rows.
- YouTube sync remains disabled from the frontend through `sourceCapabilities`.
- `sync_source` remains Telegram-only and rejects manual YouTube video sources.

## Part 3 Next: Jobs, Metadata, Transcripts

Before implementing Part 3, read:

- `docs/superpowers/plans/2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`

Part 3 likely tasks:

- Create `src-tauri/src/youtube/jobs.rs`.
- Add in-memory source job state and Tauri commands.
- Add frontend source job API.
- Add metadata sync.
- Add transcript ingest.
- Update trace ref normalization for timestamps.

Important Part 3 expectations from the plan:

- Job active lock is by:

```rust
SourceJobKey { source_id, job_type, related_source_id }
```

not source id alone.

- This allows metadata and transcript sync jobs for the same video to coexist while rejecting duplicate same-scope jobs.
- `list_source_jobs` takes `SourceJobListFilter` with `source_id`, `status`, and bounded `limit`.
- `retry_failed_youtube_playlist_videos` is one aggregate retry job.
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
- Existing `sourceActionPending` and `clearSourceActionPending` helpers from `$lib/analysis-state` should be reused.
- Playlist metadata sync is paged with `PLAYLIST_METADATA_PAGE_SIZE = 200`.
- Removed playlist rows are marked with `availability_status = 'removed_from_playlist'`.
- UI `syncSelectedSource` branches:
  - YouTube calls `syncYoutubeSource`.
  - Telegram keeps `syncSource`.
- `canSync` becomes true for YouTube video/playlist in Part 3.
- Transcript ingest uses `TempDir` RAII cleanup.
- Caption parser policy prefers `json3`, falls back to `vtt`.
- Transcript external id format:

```text
transcript:<video_id>:<language-or-und>:<manual|auto|unknown>
```

- Transcript segment replacement is `DELETE + INSERT` in one transaction.
- Timestamp refs accepted:
  - `s12-i400@754000ms`
  - `s12-i400@754000-790000ms`
- Old refs remain valid:
  - `s12-i400`
  - `s12-m400`
- Timestamp suffixes are allowed only for `-i`, not `-m`.

## Part 4 Reminder: Comments And Analysis

Part 4 plan file:

- `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`

Key reminders:

- Comments command shape:

```text
yt-dlp --dump-single-json --write-comments --skip-download --extractor-args youtube:max_comments=<limit> <video_url>
```

- `--extractor-args` and `youtube:max_comments=<limit>` must be separate args.
- Runtime default:

```rust
const DEFAULT_MAX_COMMENTS_PER_VIDEO: usize = 1_000;
```

- Rust also truncates parsed comments with `.take(max_comments)`.
- Provider-safe analysis groups add `source_type` to group/source option DTOs.
- Do not mix Telegram and YouTube sources in an analysis group.
- YouTube corpus modes:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`
- `load_corpus_messages` should use a provider-aware request, not raw source ids.
- Item filtering:
  - Telegram: `items.item_kind = 'telegram_message'`
  - YouTube transcript modes: `items.item_kind = 'youtube_transcript'`
  - YouTube comments mode: `items.item_kind IN ('youtube_transcript', 'youtube_comment')`
- Synthetic description ref:

```text
s{source_id}-i0
```

- `item_id = 0` is synthetic only and must not be treated as a DB row id.

## Part 5 Reminder: Auth And Settings

Part 5 plan file:

- `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`

Key reminders:

- Cookie secret key helper:

```rust
pub(crate) fn youtube_default_cookies_secret() -> String {
    "youtube.auth.default.cookies".to_string()
}
```

- Raw cookie text is allowed only inside backend code that writes temporary cookie files.
- Raw cookies must never appear in logs, IPC responses, job records, events, or `AppError.message`.
- Validate Netscape cookie rows:
  - accept blank/comment/header lines;
  - strip `#HttpOnly_` prefix for cookie rows;
  - require exactly 7 tab-separated fields;
  - cookie value may be empty;
  - do not reject non-YouTube domains.
- Existing `run_ytdlp(args)` keeps 30s preview timeout without cookies.
- Authenticated paths use explicit bounded timeouts.
- Commands planned:
  - `get_youtube_settings`
  - `save_youtube_settings`
  - `get_youtube_auth_status`
  - `save_youtube_cookies`
  - `clear_youtube_auth`
- Settings UI must never render stored cookie text back into an input.

## Part 6 Reminder: UI Hardening And Docs

Part 6 plan file:

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
- Create/update:
  - `docs/youtube-manual-verification.md`
  - `README.md`
  - `docs/database-schema.md`
  - `docs/architecture-deep-dive.md`
  - `docs/backlog.md`

## Important Gotchas

- Do not call `sync_source` for YouTube. Use planned `sync_youtube_source`.
- Do not decode YouTube `metadata_zstd` with Telegram `decode_source_metadata`.
- Do not fetch full playlists in one unbounded preview call.
- Do not fetch full comments without a `max_comments` bound.
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
docs: refresh session context after youtube parts 1 and 2
```

2. If the user asks for the next part, start Part 3 from `docs/superpowers/plans/2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`.
3. For Part 3, follow the same pattern used for Parts 1 and 2:
   - read the plan critically;
   - use TDD;
   - commit after each task;
   - run the specified targeted verification;
   - run final full verification before claiming completion.
