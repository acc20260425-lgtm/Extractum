# Session Context: YouTube Sources Planning and UI Policy

Updated: 2026-05-09
Workspace: `g:\Develop\Extractum`
Branch: `main`
User language: Russian
Current task state: planning/documentation only. No application implementation has been executed in this session.

## Purpose

This file captures the current session context so a future Codex session can resume the same work without losing the thread.

The broader thread is planning the YouTube Sources MVP for Extractum. The implementation is split into six plan files under `docs/superpowers/plans/`. The latest completed work before this rewrite added an explicit UI dependency policy: external UI libraries are allowed only in new components or new local wrappers, not as a wholesale UI-kit migration.

## Current IDE Context

The IDE reported these open files:

- Active: `docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`
- Open: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`

## Current Git State

Observed before rewriting this file:

```text
git branch --show-current
main

git status --short
<no output>
```

Recent commits:

```text
fbad82f docs: add targeted ui dependency policy
6558a38 docs: tighten youtube sources implementation plans
450295c docs: refresh youtube sources planning context
5857e6f docs: capture youtube sources planning session context
180d783 docs: move WebView OOM report to docs root
```

After this request, the expected only modified file is:

- `docs/session-context-2026-05-03.md`

## Latest User Requests In This Session

1. User asked in Russian which UI libraries/components would fit the project.
2. Codex inspected the project and recommended:
   - keep Extractum's local UI system as the visual baseline;
   - add `bits-ui` only for accessible headless primitives;
   - add `@lucide/svelte` for icons;
   - use `@tanstack/svelte-table` only for genuinely complex tables;
   - use `paneforge` only if resizable workspace panes become necessary;
   - avoid full Tailwind-first UI kits such as Skeleton, DaisyUI, Flowbite;
   - do not migrate the whole UI to a ready-made kit.
3. User agreed and asked to add the UI-library decision to `docs\superpowers\plans`, but only targeted and only for new components, then provide a commit message.
4. Codex updated the plan files and suggested:

```text
docs: add targeted ui dependency policy
```

5. That change is now present in commit `fbad82f`.
6. Current request: overwrite this context file with all information needed to restore the session and provide a commit message.

## Project Stack Observed

`package.json` currently shows:

- Svelte 5 / SvelteKit 2
- Vite 6
- Tauri 2
- Vitest
- TypeScript
- No `bits-ui`, `@lucide/svelte`, `@tanstack/svelte-table`, or `paneforge` installed yet.

Current frontend dependency state:

```json
"dependencies": {
  "@tauri-apps/api": "^2",
  "@tauri-apps/plugin-dialog": "^2",
  "@tauri-apps/plugin-opener": "^2",
  "@tauri-apps/plugin-sql": "^2.4.0"
}
```

Existing local UI components include:

- `src/lib/components/ui/Button.svelte`
- `src/lib/components/ui/Input.svelte`
- `src/lib/components/ui/Textarea.svelte`
- `src/lib/components/ui/Select.svelte`
- `src/lib/components/ui/CheckboxRow.svelte`
- `src/lib/components/ui/Badge.svelte`
- `src/lib/components/ui/Card.svelte`
- `src/lib/components/ui/SurfaceCard.svelte`
- `src/lib/components/ui/PanelHeader.svelte`
- `src/lib/components/ui/StatusMessage.svelte`
- `src/lib/components/ui/EmptyState.svelte`
- `src/lib/components/ui/MetaCell.svelte`
- `src/lib/components/ui/MetaPill.svelte`
- `src/lib/components/ui/RefChip.svelte`

Important frontend style fact:

- The app uses local CSS and CSS variables such as `--panel`, `--border`, `--text`, `--primary`, `--danger`, etc.
- There is no Tailwind-based design system in the current project.

## UI Library Decision Now Captured In Plans

The main plan now contains `## Frontend UI Library Policy`.

Policy summary:

- Keep Extractum's existing local UI system as the visual source of truth.
- Do not replace or restyle existing shared components with a full external UI kit.
- Approved targeted additions:
  - `@lucide/svelte` for icons in new action buttons, status rows, and provider-specific controls.
  - `bits-ui` for accessible headless primitives in new components, especially tabs, tooltips, popovers, dropdown menus, switches, and complex dialog behavior.
  - `@tanstack/svelte-table` only if a new table component needs real sorting, filtering, keyboardable row selection, or column state. Do not use it for simple lists.
  - `paneforge` only if a new workspace shell requires resizable panes. Do not introduce it for static panel layouts.
- Guardrails:
  - Wrap external primitives behind local components in `src/lib/components/ui` when they will be reused.
  - Keep styling in local CSS and existing design tokens.
  - Do not add Tailwind-first UI kits such as Skeleton, DaisyUI, or Flowbite for this MVP.
  - Do not retrofit existing components solely to use a new library.
  - Existing components may render new YouTube child components, but direct imports from new UI dependencies should stay inside new components or new local wrappers created for this MVP.
  - Include `package.json` and `package-lock.json` changes in the same task commit that first uses a new UI dependency.

Plan files with specific UI policy references:

- `docs/superpowers/plans/2026-05-09-youtube-sources.md`
  - Tech stack mentions approved targeted UI dependencies.
  - Consistency gates restrict new UI dependencies to new Svelte components or new local wrappers.
  - Final checklist includes the UI dependency guard.
- `docs/superpowers/plans/2026-05-09-youtube-sources-02-preview-and-add.md`
  - Task 3 includes optional `package.json` / `package-lock.json` changes if first UI dependency is used.
  - `youtube-source-add-panel.svelte` may use `@lucide/svelte` for preview/add/warning affordances, but only inside the new panel or new wrappers.
- `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`
  - Task 3 includes optional package file changes if first UI dependency is used.
  - `youtube-settings-panel.svelte` may use `@lucide/svelte` for auth/cookie/save/clear icons.
  - `bits-ui` may be used for a switch or tooltip only if it avoids custom accessibility code.
  - Otherwise use native inputs and existing local components.
- `docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md`
  - Tech stack mentions targeted headless UI primitives for new components only.
  - Task 1 includes optional local wrappers:
    - `src/lib/components/ui/IconButton.svelte`
    - `src/lib/components/ui/Tooltip.svelte`
    - `src/lib/components/ui/Tabs.svelte`
  - Existing workspace components must not directly import new UI dependencies.
  - YouTube detail tabs may use a local `Tabs.svelte` wrapper over `bits-ui`, or accessible markup if no dependency has been introduced.
  - Playlist row actions should prefer icon buttons with tooltips through local wrappers backed by `@lucide/svelte` and, where useful, `bits-ui`.

Verified with:

```powershell
rg -n "Frontend UI Library Policy|approved UI-library policy|@lucide/svelte|bits-ui|paneforge|targeted UI" docs\superpowers\plans
```

## Plan Files

Current plan files under `docs/superpowers/plans/`:

- `2026-05-09-youtube-sources.md`: split-plan index, consistency gates, UI library policy, final acceptance checklist.
- `2026-05-09-youtube-sources-01-foundation.md`: schema, semantic item kinds, YouTube DTOs and URL parsing.
- `2026-05-09-youtube-sources-02-preview-and-add.md`: yt-dlp preview, source creation, playlist membership, frontend add flow.
- `2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md`: source jobs, metadata sync, transcripts.
- `2026-05-09-youtube-sources-04-comments-and-analysis.md`: comments, provider-safe analysis groups, YouTube corpus loading, snapshots, trace refs.
- `2026-05-09-youtube-sources-05-auth-and-settings.md`: auth/cookies/settings.
- `2026-05-09-youtube-sources-06-ui-hardening-docs.md`: UI polish, hardening, documentation.

All six parts are intended to leave the project in a consistent state after completion.

## High-Level YouTube MVP Architecture

The implementation is split into six parts:

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

## Important Codebase Facts Found During Planning

- `src-tauri/src/sources/types.rs` already has YouTube-related source type constants/types, including `SourceType::Youtube` and `YOUTUBE_SOURCE_TYPE`.
- `src-tauri/src/sources/store.rs::source_record_from_row` currently decodes all `metadata_zstd` as Telegram metadata through `decode_source_metadata`; this must be made provider-safe before storing YouTube metadata.
- `src-tauri/src/sources/sync.rs::sync_source` is Telegram-only. UI must not call `sync_source` for YouTube.
- `src/lib/source-capabilities.ts` previously returned `canSync: source.sourceSubtype === "playlist"` for YouTube in pre-implementation code; Part 2 keeps YouTube sync disabled and Part 3 later sets YouTube video/playlist `canSync` true.
- `src/lib/analysis-source-state.ts::sourceSyncDisabledReason` currently only knows Telegram account runtime checks. Part 6 adds YouTube `yt-dlp` runtime status.
- `src/lib/analysis-scope-state.ts::currentAnalysisScopeSummary` currently says `synced messages`; Part 6 changes wording to provider-neutral `synced items`.
- `src-tauri/src/analysis/corpus.rs::load_corpus_messages` currently loads all rows with `content_zstd IS NOT NULL` by source id and does not filter by `item_kind`. Part 4 requires provider-aware item filtering.
- `src-tauri/src/analysis/corpus.rs::preflight_analysis_run` currently has its own SQL path and must use the same provider-aware corpus loader as report execution.
- `src-tauri/src/analysis/store.rs::persist_run_snapshot` currently inserts only old snapshot columns and must bind `item_kind`, `source_type`, `source_subtype`, `metadata_zstd`.
- `src-tauri/src/analysis/trace.rs::normalize_ref` currently accepts only `s{digits}-i{digits}` and `s{digits}-m{digits}`. Part 3 extends it with optional timestamp suffixes for item refs.
- `src-tauri/src/takeout_import/state.rs` locks active Takeout jobs by `source_id`. Part 3 intentionally makes YouTube source jobs lock by `(source_id, job_type, related_source_id)`.
- `src/lib/analysis-state.ts` already defines `sourceActionPending` and `clearSourceActionPending`; Part 3 tells implementers to reuse them.
- `src/lib/components/analysis/trace-panel.svelte` currently displays trace details only. It does not load source items by `item_id`.
- `src/lib/components/analysis/source-group-editor.svelte` receives `AnalysisSourceOption[]`, not full `Source[]`; Part 4 adds `source_type` to `AnalysisSourceOption`.
- `src-tauri/Cargo.toml` currently has `tempfile = "3"` only under `[dev-dependencies]`; Part 3 moves it to runtime dependencies for captions.
- `src-tauri/src/secret_store.rs` has an existing `secret_ids_are_stable` test; Part 5 adds the YouTube cookies secret key to that test.
- No `src-tauri/src/youtube` module exists yet before execution of the plans.

## Part 1 Summary: Foundation

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

Important availability status values:

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

## Part 2 Summary: Preview and Add

Part 2 covers:

- Add `yt-dlp` adapter.
- Add preview commands for video and playlist.
- Add source creation helpers for YouTube video/playlist.
- Add playlist membership persistence.
- Add frontend YouTube add flow.

Important details:

- `run_ytdlp` handles missing binary and non-zero exit codes.
- Preview commands use `tokio::time::timeout`, currently 30s for preview.
- Playlist preview limits items with `--playlist-items 1-50`.
- Command-builder helpers:
  - `preview_video_args(canonical_url: &str) -> Vec<String>`
  - `preview_playlist_args(canonical_url: &str) -> Vec<String>`
- Test asserts `--playlist-items` and `1-50` are adjacent separate args.
- YouTube source upsert uses partial unique index conflict targets for video and playlist.
- Playlist item video source creation is race-safe.
- `add_youtube_source` persistence happens in a transaction after metadata fetch.
- `source_record_from_row` must not decode YouTube metadata as Telegram metadata.
- Frontend flow uses `youtube-source-add-panel.svelte`.
- YouTube `canSync` remains false in Part 2 for both video and playlist.
- UI dependency policy applies only inside the new YouTube panel or new local wrappers.

## Part 3 Summary: Jobs, Metadata, Transcripts

Part 3 covers:

- Create `src-tauri/src/youtube/jobs.rs`.
- Add source job state and Tauri commands.
- Add frontend source job API.
- Add metadata sync.
- Add transcript ingest.
- Update trace ref normalization for timestamps.

Important details:

- Job active lock is by `SourceJobKey { source_id, job_type, related_source_id }`, not source id alone.
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
- Existing `sourceActionPending` and `clearSourceActionPending` helpers from `$lib/analysis-state` must be reused.
- Playlist metadata sync is paged with `PLAYLIST_METADATA_PAGE_SIZE = 200`.
- Removed playlist rows are marked with `availability_status = 'removed_from_playlist'`.
- UI `syncSelectedSource` branches:
  - YouTube calls `syncYoutubeSource`.
  - Telegram keeps `syncSource`.
- `canSync` becomes true for YouTube video/playlist in Part 3.
- Transcript ingest uses `TempDir` RAII cleanup.
- Caption parser policy prefers `json3`, falls back to `vtt`.
- Transcript external id format:
  - `transcript:<video_id>:<language-or-und>:<manual|auto|unknown>`
- Transcript segment replacement is `DELETE + INSERT` in one transaction.
- Timestamp refs accepted:
  - `s12-i400@754000ms`
  - `s12-i400@754000-790000ms`
- Old refs remain valid:
  - `s12-i400`
  - `s12-m400`
- Timestamp suffixes are allowed only for `-i`, not `-m`.

## Part 4 Summary: Comments and Analysis

Part 4 covers:

- Create `src-tauri/src/youtube/comments.rs`.
- Add comments ingest.
- Add provider-safe analysis groups.
- Add YouTube corpus modes.
- Add playlist expansion for analysis.
- Add saved snapshot metadata.
- Add YouTube-aware trace refs.

Comments ingest:

- Command is bounded:

```text
yt-dlp --dump-single-json --write-comments --skip-download --extractor-args youtube:max_comments=<limit> <video_url>
```

- Runtime constant:

```rust
const DEFAULT_MAX_COMMENTS_PER_VIDEO: usize = 1_000;
```

- `--extractor-args` and `youtube:max_comments=<limit>` are separate command args.
- Rust also truncates parsed comments with `.take(max_comments)`.
- Timestamp policy uses comment timestamp as integer/string, fallback to video upload timestamp, then sync start timestamp.
- Missing/unparsable comment timestamps produce warnings.
- `upsert_youtube_comment_item` uses `ON CONFLICT(source_id, external_id) DO UPDATE`.
- Comments run only when `YoutubeSyncOptions.comments = true`.

Provider-safe analysis groups:

- Add `source_type` to:
  - `AnalysisSourceGroup`
  - `AnalysisSourceGroupRow`
  - `AnalysisSourceOption`
  - TypeScript analysis group/source option types.
- Create/update group commands accept `source_type`.
- Existing groups rely on migration default `telegram`.
- Membership validation happens after `ensure_sources_exist` and before write transaction.
- `source-group-editor.svelte` gets a group type select:
  - Telegram
  - YouTube
- Candidate sources are filtered by `AnalysisSourceOption.source_type`.

YouTube corpus loading:

- `YoutubeCorpusMode` wire values:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`
- `start_analysis_report` accepts `youtube_corpus_mode: Option<String>`.
- Telegram runs ignore `youtube_corpus_mode`.
- YouTube runs default to `transcript_description` when omitted/null.
- Resolver returns `ResolvedAnalysisSources`.
- YouTube playlist expansion includes only `video_source_id IS NOT NULL` and `is_removed_from_playlist = 0`.
- If expansion yields zero source ids, return:
  - `No linked YouTube videos are available for analysis in this scope`
- `load_corpus_messages` uses `CorpusLoadRequest`, not raw ids.
- Item filtering:
  - Telegram: `items.item_kind = 'telegram_message'`
  - YouTube transcript modes: `items.item_kind = 'youtube_transcript'`
  - YouTube comments mode: `items.item_kind IN ('youtube_transcript', 'youtube_comment')`
- Description is a synthetic `CorpusMessage`.
- Synthetic description ref:
  - `s{source_id}-i0`
- `item_id = 0` is synthetic only and is not a database item row.

Snapshots and trace refs:

- `CorpusMessage` gets optional snapshot metadata:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- `persist_run_snapshot` inserts/binds the new snapshot fields.
- YouTube transcript corpus loads segment-level messages from `youtube_transcript_segments`.
- Segment refs look like:
  - `s12-i400@754000ms`
- `AnalysisTraceRef` gets:
  - `youtube_url`
  - `youtube_timestamp_seconds`
  - `youtube_display_label`
  - `is_synthetic`
- `build_trace_refs` resolves exact timestamp refs first, then old base item refs.
- Synthetic refs have `is_synthetic = true` and `item_id = 0`.

## Part 5 Summary: Auth and Settings

Part 5 covers:

- Secure cookie storage.
- Netscape cookie validation.
- Authenticated `yt-dlp` execution with temp cookie files.
- YouTube settings commands.
- YouTube settings UI.

Cookie boundary:

- Add `src-tauri/src/youtube/cookies.rs`.
- Add secret key helper:

```rust
pub(crate) fn youtube_default_cookies_secret() -> String {
    "youtube.auth.default.cookies".to_string()
}
```

- Extend `secret_store::tests::secret_ids_are_stable`.
- Raw cookie text is allowed only inside backend code that writes the temporary cookie file.
- Raw cookies must never appear in logs, IPC responses, job records, Tauri events, or `AppError.message`.
- Validate Netscape cookie rows:
  - empty/whitespace-only cookies are validation errors;
  - accept blank lines and comment/header lines;
  - `#HttpOnly_` rows are cookie rows after stripping that prefix;
  - require exactly 7 tab-separated fields;
  - cookie value may be empty;
  - do not reject non-YouTube domains.
- Add `YtdlpRunOptions { timeout: Duration, cookies: Option<String> }`.
- Existing `run_ytdlp(args)` keeps 30s preview timeout with no cookies.
- All authenticated paths are explicitly bounded by timeouts.

Settings commands:

- Add explicit app setting keys matching migration defaults:
  - `youtube.auth.enabled`
  - `youtube.captions.preferred_language`
  - `youtube.sync.delay_between_requests_ms`
  - `youtube.sync.max_parallel_video_syncs`
  - `youtube.sync.max_parallel_comment_syncs`
  - `youtube.sync.pause_on_auth_challenge`
  - `youtube.sync.daily_soft_limit`
  - `youtube.sync.retry_backoff_ms`
  - `youtube.sync.stop_after_consecutive_failures`
- DTOs serialize camelCase.
- Validation ranges:
  - `preferred_captions_language`: `original` or 2-32 ASCII letters/digits/hyphen/underscore chars.
  - `delay_between_requests_ms`: 0..=60000.
  - `max_parallel_video_syncs`: 1..=4.
  - `max_parallel_comment_syncs`: 1..=2.
  - `daily_soft_limit`: 0..=10000.
  - `retry_backoff_ms`: 0..=300000.
  - `stop_after_consecutive_failures`: 1..=50.
- Write policy validates whole DTO before writing anything.
- Auth status messages:
  - `Auth disabled`
  - `Cookies stored`
  - `No cookies configured`
- Commands:
  - `get_youtube_settings`
  - `save_youtube_settings`
  - `get_youtube_auth_status`
  - `save_youtube_cookies`
  - `clear_youtube_auth`

Settings UI:

- Create `src/lib/components/settings/youtube-settings-panel.svelte`.
- Mount it from `src/routes/settings/+page.svelte`.
- Never render stored cookie text back into an input.
- Hidden-by-default cookie textarea.
- Disable save-cookies button for empty/whitespace input.
- Frontend input constraints match backend validation.
- UI policy allows `@lucide/svelte` and `bits-ui` only inside the new panel or new wrappers.

## Part 6 Summary: UI Hardening and Docs

Part 6 covers:

- YouTube runtime/detail APIs.
- Source cards and workspace details.
- Job controls.
- YouTube corpus selector.
- Manual hardening matrix.
- README/schema/architecture/backlog docs.

Runtime/detail:

- Add `src-tauri/src/youtube/runtime.rs`.
- Add `src-tauri/src/youtube/detail.rs`.
- `get_youtube_runtime_status` runs `yt-dlp --version` with a 5s timeout.
- Detail DTOs include:
  - `YoutubeContentSyncState`
  - `YoutubeContentStatusDto`
  - `YoutubeSourceSummaryDto`
  - `YoutubeVideoDetailDto`
  - `YoutubePlaylistDetailDto`
- Commands:
  - `list_youtube_source_summaries`
  - `get_youtube_video_detail`
  - `get_youtube_playlist_detail`
- Captions/comments status comes from persisted items and transcript segments.
- Source metadata must be decoded as YouTube metadata, never Telegram metadata.

Workspace UI:

- `sourceSyncDisabledReason` accepts optional `YoutubeRuntimeStatus`.
- YouTube sync is disabled with useful message when `yt-dlp` unavailable.
- Scope summaries use `synced items`, not `synced messages`.
- `source-row.svelte` and `workspace-rail.svelte` accept optional YouTube summaries.
- Create:
  - `src/lib/components/analysis/youtube-source-detail.svelte`
  - `src/lib/components/analysis/youtube-playlist-detail.svelte`
- Video detail tabs:
  - Overview
  - Transcript
  - Comments
  - Jobs
- Playlist detail shows ordered rows and row actions.
- `workspace-main.svelte` routes YouTube video/playlist sources to the new detail components.
- UI policy allows local wrappers such as:
  - `IconButton.svelte`
  - `Tooltip.svelte`
  - `Tabs.svelte`

Job and analysis controls:

- Extend `src/lib/api/source-jobs.ts` with:
  - `syncYoutubePlaylistVideo`
  - `retryFailedYoutubePlaylistVideos`
  - `cancelSourceJob`
- Video controls map to exact `syncYoutubeSource` option objects.
- Playlist controls map to exact playlist commands.
- `sync all playlist videos` does not fetch comments by default.
- Per-video playlist actions:
  - open source
  - sync this video
  - retry this video
- Add YouTube corpus selector:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`
- Hide Telegram topic controls for YouTube sources.

Manual hardening matrix:

- Create/update `docs/youtube-manual-verification.md`.
- Scenarios:
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
  - jobs are in memory;
  - no active job restored after restart;
  - UI must not keep stale pending state;
  - completed DB writes remain visible;
  - user can start a new sync after restart;
  - no MVP attempt to resume interrupted `yt-dlp`.

Documentation:

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
- Backlog item is YouTube-specific NotebookLM export enrichment, not the existing generic NotebookLM export.

Final verification in Part 6:

```powershell
cd src-tauri
cargo test --lib
cargo clippy --all-targets -- -D warnings
cd ..
npm test
npm run check
npm run build
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
- Do not add duplicate dependencies without checking `Cargo.toml` or `package.json`.
- Use `TempDir` or equivalent RAII for caption/cookie temp files.
- Keep raw cookie values out of logs, events, errors, command args, and IPC responses.
- Keep authenticated `yt-dlp` paths bounded by explicit timeouts.
- Keep old refs `sN-iM` and `sN-mM` working.
- Keep every plan part consistent after each task.
- YouTube jobs are in memory for the MVP; restart does not restore them.
- New UI dependencies must stay inside new components or new local wrappers created for this MVP.
- Do not add a full Tailwind-first UI kit.

## Verification Already Done During Planning

No application tests have been run in this context because no application implementation has been changed here.

Planning/documentation checks observed:

- `git diff --check -- docs\superpowers\plans` passed with exit code 0 before the UI policy commit. Git printed only CRLF warnings.
- `rg -n "Frontend UI Library Policy|approved UI-library policy|@lucide/svelte|bits-ui|paneforge|targeted UI" docs\superpowers\plans` confirms the UI policy is present.
- `git status --short` was clean before rewriting this context file.

Earlier plan-level scans checked for placeholder/conflict strings such as:

- `TBD`
- `TODO`
- `implement later`
- `fill in details`
- `Add appropriate`
- `Write tests for the above`
- `Similar to Task`
- old `sync_comments`
- old corpus loading paths using raw `source_ids`
- old list job shape
- old transcript external id shape

## Current User Request

The user asked in Russian to overwrite `docs\session-context-2026-05-03.md` with all information needed to restore the current session context and to provide a commit message.

This file is the requested overwritten context snapshot.

## Suggested Commit Message

```text
docs: refresh session context
```

## Recommended Next Steps

1. Commit this context refresh with the suggested commit message.
2. If more planning feedback arrives, update the relevant plan file and keep this context file current.
3. If planning is accepted, begin implementation from Part 1 and follow each plan task-by-task.
4. Before implementation, use the execution skills requested in the plan headers:
   - `superpowers:subagent-driven-development`, or
   - `superpowers:executing-plans`
5. At implementation time, verify each task with the exact commands in the corresponding plan part.
6. At final MVP verification, include `cargo clippy --all-targets -- -D warnings` in addition to Rust tests and frontend checks.
