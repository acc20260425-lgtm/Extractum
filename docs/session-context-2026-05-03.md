# Session Context: YouTube Sources MVP

Updated: 2026-05-09
Workspace: `g:\Develop\Extractum`
Shell: PowerShell
Timezone: Europe/Minsk
User language: Russian
Branch: `main`

This file is the restore point for the current Codex session. A future session should be able to continue YouTube Sources work without reading the full chat history.

## Current User Request

The latest user request was:

- Rewrite `docs\session-context-2026-05-03.md` with all information needed to restore the current session.
- The file may be overwritten.
- Provide a commit message.

The user did not explicitly ask to commit this documentation update.

Suggested commit message for this documentation update:

```text
docs: refresh session context after youtube part 4
```

## Current IDE Context

Latest IDE context reported by the user:

- Active file: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`
- Open tab: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`

## Workflow And Session Rules

- Use relevant Superpowers skills on future implementation turns.
- This session used:
  - `superpowers:using-superpowers`
  - `superpowers:executing-plans`
  - `superpowers:test-driven-development`
  - `superpowers:systematic-debugging`
  - `superpowers:verification-before-completion`
- Do not spawn subagents unless the user explicitly asks for delegation or parallel agents.
- Work is being done directly on `main`.
- Use `rg` / `rg --files` for search.
- Use `apply_patch` for manual file edits.
- In PowerShell, prefer `npm.cmd`.
- Svelte edits should be checked with Svelte autofixer before finalizing.
- Before claiming completion, rerun relevant verification and inspect output.
- If committing later, `git add` and `git commit` may require escalation.

## Current Git State

Before this documentation rewrite, `git status --short` returned no output.

Latest commits before this documentation rewrite:

```text
52891eb fix: add youtube sources on upgraded databases
a861fe0 docs: refresh session context after youtube part 4 task 4
87f12b2 feat: resolve youtube timestamp evidence
3e4f890 feat: load youtube analysis corpus
d55aa90 feat: enforce provider-specific analysis groups
138b6c1 docs: refresh session context after youtube part 4 task 1
3707090 feat: ingest youtube comments
50ac301 test: update youtube sync disabled expectation
```

After this request, `docs/session-context-2026-05-03.md` is modified and should be the only intended documentation change unless the user also asks to update the plan checkboxes.

## Important Status Summary

YouTube Sources Part 4 is functionally complete:

- Task 1: comments ingest completed and committed.
- Task 2: provider-safe analysis groups completed and committed.
- Task 3: YouTube corpus loading and playlist expansion completed and committed.
- Task 4: timestamp trace refs and run snapshots completed and committed.
- Manual verification for Part 4 was completed.
- A real manual-verification bug for upgraded databases was found, fixed with TDD, verified, and committed.

Important caveat:

- The active Part 4 markdown plan file still has unchecked checklist boxes. The implementation and manual verification are complete, but the plan document itself has not been physically marked complete.

## Active Plan

Active plan file:

```text
docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md
```

Goal of Part 4:

- Sync YouTube comments.
- Make YouTube videos, playlists, and YouTube-only groups analyzable.
- Prevent provider-mixed analysis groups.
- Expand playlists into canonical child video sources for analysis.
- Support YouTube corpus modes.
- Store stable run snapshots with metadata.
- Resolve timestamp trace refs and synthetic description refs.

## Part 4 Task 1: Comments Ingest

Commit:

```text
3707090 feat: ingest youtube comments
```

Implemented:

- Created `src-tauri/src/youtube/comments.rs`.
- Added `comments` module.
- Added bounded comments fetch:

```text
yt-dlp --dump-single-json --write-comments --skip-download --extractor-args youtube:max_comments=<limit> <video_url>
```

- Added `DEFAULT_MAX_COMMENTS_PER_VIDEO = 1_000`.
- Normalized top-level comments and replies into `YoutubeComment`.
- Timestamp policy:
  - accept numeric timestamp;
  - accept string timestamp;
  - fallback to video timestamp;
  - fallback to sync start timestamp;
  - warn on missing or unparseable comment timestamp.
- Added `upsert_youtube_comment_item`.
- Persisted YouTube comments as:
  - `item_kind = youtube_comment`
  - `external_id = comment:<comment_id>`
  - `content_kind = text_only`
  - `reaction_count = like_count`
- Stored parent id, reply state, like count, pinned state, creator reaction, author metadata, and raw provider payload in compressed raw data.
- Comment upsert updates existing rows instead of duplicating.
- Jobs run comments only when `YoutubeSyncOptions.comments = true`.

Verification at completion:

```powershell
cd src-tauri
cargo test youtube::comments --lib
cargo test youtube::jobs --lib
cargo test sources::items --lib
cargo test --lib
```

Known results:

- `youtube::comments`: 5 passed.
- `youtube::jobs`: 4 passed.
- `sources::items`: 8 passed.
- Full `cargo test --lib`: 244 passed.

## Part 4 Task 2: Provider-Safe Analysis Groups

Commit:

```text
d55aa90 feat: enforce provider-specific analysis groups
```

Implemented backend:

- Added `source_type` to:
  - `AnalysisSourceOption`
  - `AnalysisSourceGroup`
  - `AnalysisSourceGroupRow`
- Updated `list_analysis_sources` to select `sources.source_type`.
- Updated `fetch_source_group` to load group `source_type`.
- Updated create/update analysis group commands to accept `source_type`.
- Added validation helper in `src-tauri/src/analysis/groups.rs`:

```rust
async fn validate_group_source_type(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    group_source_type: &str,
    source_ids: &[i64],
) -> AppResult<()>
```

- Validation allows only `telegram` or `youtube`.
- Validation rejects sources whose `sources.source_type` does not match the group type.
- Validation runs after `ensure_sources_exist` and before the write transaction.
- In-memory test schemas were updated for `analysis_source_groups.source_type`.

Implemented frontend:

- Added `AnalysisGroupSourceType`.
- Added `source_type` to analysis source/group TS types.
- Added `sourceType` to create/update group inputs.
- Group editor has a Telegram/YouTube type select.
- Group editor filters candidate sources by selected provider.
- Route state keeps `groupSourceType`.
- Selecting an existing group copies its `source_type`.
- Starting a new group defaults to `telegram`.
- Save calls pass group type.

Task 2 RED checks:

- `cargo test analysis::groups --lib` initially failed because provider validation was missing.
- Vitest workflow test initially failed because `sourceType` was not passed.

Task 2 final verification:

```powershell
cd src-tauri
cargo test analysis::groups --lib
cargo test analysis::store --lib
cd ..
npm.cmd test -- analysis-source-groups
npm.cmd run check
```

Results:

- `analysis::groups`: 3 passed.
- `analysis::store`: 3 passed.
- `analysis-source-groups`: 24 passed.
- `svelte-check found 0 errors and 0 warnings`.

Svelte autofixer:

- Ran on `source-group-editor.svelte`.
- No issues.

## Part 4 Task 3: YouTube Corpus Loading And Playlist Expansion

Commit:

```text
3e4f890 feat: load youtube analysis corpus
```

Implemented backend:

- Added `YoutubeCorpusMode`:
  - `TranscriptOnly`
  - `TranscriptDescription`
  - `TranscriptDescriptionComments`
- Wire values:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`
- Added `YoutubeCorpusMode::from_wire`.
- Added `CorpusLoadRequest`.
- Added `ResolvedAnalysisSources`.
- Added provider-aware resolver:
  - Telegram single source -> direct source id.
  - Telegram group -> direct member source ids.
  - YouTube single video -> direct video source id.
  - YouTube single playlist -> linked non-removed `youtube_playlist_items.video_source_id`.
  - YouTube group -> direct videos plus expanded playlist children.
  - Excludes `video_source_id IS NULL`.
  - Excludes removed playlist rows.
  - Counts skipped unlinked playlist rows in `skipped_unlinked_playlist_items`.
  - If YouTube expansion yields zero ids, returns validation error:

```text
No linked YouTube videos are available for analysis in this scope
```

- Replaced raw `source_ids` corpus path with provider-aware `CorpusLoadRequest`.
- Corpus item kind filters:
  - Telegram -> `telegram_message`
  - YouTube transcript modes -> `youtube_transcript`
  - YouTube comments mode -> `youtube_transcript` and `youtube_comment`
- Added synthetic YouTube description messages when mode includes descriptions and metadata has a non-empty description inside the analysis period.
- Synthetic description shape:
  - `item_id = 0`
  - `external_id = description:<video_id>`
  - `ref = s<source_id>-i0`
- `preflight_analysis_run` now calls the same corpus loader used by execution.
- `ReportRunInput` carries `corpus_request`.
- `start_analysis_report` accepts `youtube_corpus_mode`.

Implemented frontend:

- Added TS `YoutubeCorpusMode`.
- Added `youtubeCorpusMode` to `AnalysisReportStartCommand`.
- Added `youtubeCorpusMode` to analysis start state.
- Route state defaults to `transcript_description`.
- YouTube corpus select added for YouTube single-source/group scopes.
- Updated `workspace-main.svelte`.
- Updated `run-controls.svelte`.
- Updated analysis run workflow/API/state tests.

Task 3 RED checks:

- `cargo test analysis::corpus --lib` failed before implementation due missing resolver/request/mode and old loader signatures.

Task 3 final verification:

```powershell
cd src-tauri
cargo test analysis::corpus --lib
cargo test analysis::report --lib
cd ..
npm.cmd test -- analysis-run-workflow analysis-state analysis-runs
npm.cmd run check
git diff --check
```

Results:

- `analysis::corpus`: 19 passed.
- `analysis::report`: 12 passed.
- TS tests: 3 files passed, 65 tests passed.
- `svelte-check found 0 errors and 0 warnings`.
- `git diff --check` had no whitespace errors.

Svelte autofixer:

- Ran on `run-controls.svelte`.
- No issues.

## Part 4 Task 4: Timestamp Trace Refs And Run Snapshots

Commit:

```text
87f12b2 feat: resolve youtube timestamp evidence
```

Implemented Rust model changes:

- Extended `AnalysisTraceRef` with:
  - `youtube_url: Option<String>`
  - `youtube_timestamp_seconds: Option<i64>`
  - `youtube_display_label: Option<String>`
  - `is_synthetic: bool`
- Extended `StoredAnalysisItemRow` with:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- Extended `StoredRunSnapshotRow` with the same fields.
- Extended `CorpusMessage` with the same snapshot metadata fields.

Implemented Rust store/corpus changes:

- `persist_run_snapshot` inserts:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- `load_run_snapshot_messages` selects those fields and restores them into `CorpusMessage`.
- Live item loader joins `sources` and selects:
  - `items.item_kind`
  - `sources.source_type`
  - `sources.source_subtype`
  - `items.media_metadata_zstd AS metadata_zstd`
- Added segment-level YouTube transcript loading from `youtube_transcript_segments`.
- YouTube transcript corpus messages now use refs like:

```text
s<source_id>-i<transcript_item_id>@<segment_start_ms>ms
```

- Added segment metadata JSON compression with at least:
  - `video_id`
  - `canonical_url`
  - `title`
  - `channel_title`
  - `channel_handle`
  - `caption_language`
  - `caption_track_kind`
  - `segment_start_ms`
  - `segment_end_ms`
  - `item_kind = youtube_transcript`
- Synthetic YouTube descriptions now set:
  - `item_id = 0`
  - `item_kind = youtube_description`
  - `source_type = youtube`
  - `source_subtype = video`
  - `metadata_zstd` with YouTube metadata.

Implemented Rust trace changes:

- `build_trace_refs` resolves exact refs first.
- Fallback matches same `source_id` and `item_id`, keeping old `sN-iM` and `sN-mM` refs working.
- Timestamp refs parse `@<ms>ms`.
- `youtube_timestamp_seconds` is `ms / 1000`.
- `youtube_url` appends `t=<seconds>`.
- `youtube_display_label` formats as:
  - `Video title at 12:34` when title exists.
  - `YouTube at 12:34` otherwise.
- `is_synthetic = true` when:
  - `message.item_id == 0`, or
  - `message.item_kind == Some("youtube_description")`.

Implemented TypeScript/frontend changes:

- Extended `AnalysisTraceRef` in `src/lib/types/analysis.ts` with:
  - `youtube_url: string | null`
  - `youtube_timestamp_seconds: number | null`
  - `youtube_display_label: string | null`
  - `is_synthetic: boolean`
- Updated trace fixtures in:
  - `src/lib/analysis-state.test.ts`
  - `src/lib/analysis-trace-workflow.test.ts`
  - `src/lib/api/analysis-trace.test.ts`
- Added regression test in `analysis-trace-workflow.test.ts`:
  - saved synthetic YouTube description ref has `item_id = 0` and `is_synthetic = true`;
  - remains selectable;
  - does not call `resolveRefs`.
- Updated `trace-panel.svelte` to render:
  - YouTube link when `youtube_url` exists.
  - `youtube_display_label` when available.
  - `Saved synthetic evidence` for synthetic refs.

Task 4 final verification:

```powershell
cd src-tauri
cargo test analysis::trace --lib
cargo test analysis::corpus --lib
cargo test analysis::store --lib
cd ..
npm.cmd test -- analysis-trace analysis-trace-workflow
npm.cmd run check
git diff --cached --check
```

Results:

- `analysis::trace`: 7 passed.
- `analysis::corpus`: 19 passed.
- `analysis::store`: 3 passed.
- `analysis-trace` and `analysis-trace-workflow`: 13 tests passed.
- `svelte-check`: 0 errors and 0 warnings.
- `git diff --cached --check`: clean.

Svelte autofixer:

- Ran on `trace-panel.svelte`.
- No issues.

## Legacy Database YouTube Source Fix

Commit:

```text
52891eb fix: add youtube sources on upgraded databases
```

Manual verification found a real bug on an upgraded database:

```text
NOT NULL constraint failed: sources.telegram_source_kind
```

Root cause:

- Old migration made `sources.telegram_source_kind TEXT NOT NULL DEFAULT 'channel'`.
- YouTube upsert inserted `NULL` for this Telegram compatibility column.
- Fresh test schemas did not expose the bug.

TDD tests added in `src-tauri/src/sources/store.rs`:

- `source_record_parts_hides_non_telegram_compatibility_kind`
- `upsert_youtube_video_source_handles_legacy_not_null_telegram_kind`
- `upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind`

RED result:

- `cargo test sources::store::tests --lib` failed on the three new tests.

Fix:

- YouTube source upsert now inserts `telegram_source_kind = ''` for video/playlist rows so legacy NOT NULL databases work.
- `source_record_from_row_parts` hides non-Telegram compatibility kind by returning `telegram_source_kind: None` when `source_type != "telegram"`.

Verification:

```powershell
cd src-tauri
cargo test sources::store::tests --lib
cargo fmt
cargo test sources::store --lib
cargo test youtube::preview --lib
cargo test youtube::jobs --lib
cargo test --lib
```

Results:

- `sources::store::tests`: 6 passed.
- `sources::store`: passed.
- `youtube::preview`: passed.
- `youtube::jobs`: passed.
- Full `cargo test --lib`: 261 passed.

## Manual Verification Of Part 4

Manual verification was performed with the Tauri MCP bridge and the app running at:

```text
http://localhost:1420/analysis
```

Tauri app id:

```text
org.ai.extractum
```

Tauri bridge:

```text
localhost:9223
```

The dev app was started with:

```powershell
Start-Process -FilePath npm.cmd -ArgumentList @('run','tauri','--','dev') -WorkingDirectory 'G:\Develop\Extractum' -WindowStyle Hidden
```

The app and bridge may or may not still be running in a future session. Check before using:

```text
mcp__tauri__.driver_session status
```

`yt-dlp` was verified:

```powershell
yt-dlp --version
```

Result:

```text
2026.03.17
```

### Video Used For Reliable Manual Verification

Initial test video:

```text
https://www.youtube.com/watch?v=dQw4w9WgXcQ
```

Notes:

- Preview worked.
- After the legacy DB fix, adding the source worked as source id `28`.
- Full sync failed on comments with `yt-dlp preview timed out after 30 seconds`, likely due comment volume on a popular video.

Reliable test video:

```text
https://www.youtube.com/watch?v=M7lc1UVf-VE
```

Source:

- Source id: `30`
- Title: `YouTube Developers Live: Embedded Web Player Customization`
- Preview available.
- Manual and auto captions available.

Full sync for source `30`:

- Metadata + transcript + comments succeeded.
- Job id: `source-job-2`
- Status: `succeeded`
- DB counts:
  - `youtube_transcript`: 1 item
  - transcript segments: 466
  - `youtube_comment`: 94 items
- Reply-like comment ids were present, including ids with dot suffixes.

Comments-only resync:

- Job id: `source-job-3`
- Status: `succeeded`
- `youtube_comment` count remained 94, confirming idempotent upsert behavior.

### Provider-Safe Analysis Group Verification

Created YouTube group:

- Group id: `1`
- Name: `Manual Verification YouTube`
- Member source: `30`

Telegram source used for negative test:

- Source id: `17`
- Title: `chat aigenis invest`

Attempt to update group with `[17, 30]` was rejected:

```text
Source 17 has type 'telegram' and cannot be added to a 'youtube' analysis group
```

Membership before and after rejection remained only source `30`.

### Analysis Mode Verification

Run settings:

- Source: `30`
- Period: `1360000000` to `1778331400`
- Template id: `1`
- Profile: `test_profile`
- Provider: `omniroute`
- Model: `gemini/gemini-3-flash-preview`

Runs:

- Run `54`: `transcript_only`
  - Status: completed.
  - Snapshot: 466 `youtube_transcript` docs.
- Run `55`: `transcript_description`
  - Status: completed.
  - Snapshot: 466 `youtube_transcript` docs + 1 `youtube_description`.
- Run `56`: `transcript_description_comments`
  - Status: completed.
  - Snapshot: 466 transcript docs + 1 description + 94 comment docs.
  - Report included transcript refs and comment refs, for example `s30-i134725`.

### Playlist Expansion Verification

Created local synthetic manual playlist source:

- Source id: `32`
- Title: `Manual Verification Playlist`

Inserted playlist rows:

- Linked row:
  - `video_source_id = 30`
  - `video_id = 'M7lc1UVf-VE'`
- Unlinked row:
  - `video_source_id = NULL`
  - `video_id = 'manual-unlinked-video'`
  - `availability_status = 'no_captions'`

Run:

- Run `57`
- Scope: playlist source `32`
- Mode: `transcript_only`
- Status: completed.

Result:

- Snapshot contained only source `30` transcript docs.
- Query for `external_id LIKE '%manual-unlinked-video%'` returned count 0.
- No empty documents were created for `video_source_id IS NULL`.

### Saved-Run Stability Verification

Before resync, resolved refs for run `56`:

- Timestamp ref:
  - Ref: `s30-i134703@14030ms`
  - `youtube_url`: `https://www.youtube.com/watch?v=M7lc1UVf-VE&t=14`
  - `youtube_timestamp_seconds`: `14`
  - Display label ended with `at 0:14`.
- Synthetic description ref:
  - Ref: `s30-i0`
  - `item_id = 0`
  - `is_synthetic = true`
  - Saved excerpt came from description.
  - `youtube_url` was base video URL.

Then source `30` was resynced with transcripts and comments:

- Job id: `source-job-4`
- Status: `succeeded`

After resync:

- Current source items remained:
  - 94 comments
  - 1 transcript item
- Run `56` snapshot remained:
  - 94 comments
  - 1 description
  - 466 transcript docs
- Same timestamp and synthetic refs resolved identically after resync.

### Trace UI Verification

Run `54` trace panel:

- Timestamp ref `s30-i134703@14030ms` rendered with YouTube link:

```text
https://www.youtube.com/watch?v=M7lc1UVf-VE&t=14
```

To force a synthetic description citation, prompt template id `2` was created:

- Name: `Manual YouTube description trace`
- Body:

```text
Create a very short report using only the YouTube video description document. Cite the description evidence ref exactly once. Do not cite transcript or comment refs.
```

Run:

- Run `58`
- Mode: `transcript_description`
- Template id: `2`
- Status: completed.
- Result cited `[s30-i0]`.

Trace panel for run `58` showed:

- `s30-i0`
- Link: `https://www.youtube.com/watch?v=M7lc1UVf-VE`
- Text: `Saved synthetic evidence`
- Saved excerpt from YouTube description
- No source-item lookup was needed because `item_id = 0`.

## Local Data Created During Manual Verification

The following data was created in the user's local app/database during manual verification. Do not delete it unless the user asks.

- YouTube source `28`: Rick Astley test video, added after legacy DB fix.
- YouTube source `30`: `YouTube Developers Live: Embedded Web Player Customization`.
- YouTube playlist source `32`: `Manual Verification Playlist`.
- Analysis group `1`: `Manual Verification YouTube`.
- Prompt template `2`: `Manual YouTube description trace`.
- Analysis runs `54`, `55`, `56`, `57`, `58`.
- Source jobs including `source-job-2`, `source-job-3`, `source-job-4`.

## Part 4 Completion Answer Already Given

When the user asked whether Part 4 was fully complete, the answer given was:

- Yes, functionally Part 4 is fully complete.
- Code implementation, automated verification, manual verification, and the upgraded-database fix are done.
- The only documentation caveat is that the markdown plan checkboxes were still not physically updated, and this session context needed refreshing.

## Recommended Next Steps

If the user wants documentation cleanup:

1. Update `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md` to mark Part 4 tasks and manual verification complete.
2. Commit this session context update and optional plan checkbox update.

If the user wants implementation to continue:

1. Move to Part 5.
2. Read `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`.
3. Use `superpowers:executing-plans`.
4. Use `superpowers:test-driven-development` for behavior changes.
5. Keep one commit per task if the user continues the previous working style.

## Part 5 Reminder: Auth And Settings

Plan file:

```text
docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md
```

Key reminders:

- Raw cookies are allowed only inside backend code that writes temporary cookie files.
- Raw cookies must never appear in logs, IPC responses, job records/events, or `AppError.message`.
- Settings UI must never render stored cookie text back into an input.
- Existing `run_ytdlp(args)` keeps 30s preview timeout without cookies.
- Authenticated paths use explicit bounded timeouts.

## Part 6 Reminder: UI Hardening And Docs

Plan file:

```text
docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md
```

Key reminders:

- Add runtime/detail APIs:
  - `src-tauri/src/youtube/runtime.rs`
  - `src-tauri/src/youtube/detail.rs`
- `get_youtube_runtime_status` runs `yt-dlp --version` with a 5s timeout.
- Source summaries should use provider-neutral `synced items`, not `synced messages`.
- Hide Telegram topic controls for YouTube sources.

## Verification Command Reference

Useful backend commands:

```powershell
cd src-tauri
cargo test analysis::groups --lib
cargo test analysis::store --lib
cargo test analysis::corpus --lib
cargo test analysis::report --lib
cargo test analysis::trace --lib
cargo test youtube::comments --lib
cargo test youtube::preview --lib
cargo test youtube::jobs --lib
cargo test sources::items --lib
cargo test sources::store --lib
cargo test --lib
```

Useful frontend commands:

```powershell
cd g:\Develop\Extractum
npm.cmd test -- analysis-source-groups
npm.cmd test -- analysis-run-workflow analysis-state analysis-runs
npm.cmd test -- analysis-trace analysis-trace-workflow
npm.cmd run check
```

Whitespace checks:

```powershell
git diff --check
git diff --cached --check
```

If `npm.cmd test -- <filter>` has environment issues, use:

```powershell
node --trace-uncaught ./node_modules/vitest/vitest.mjs run <path-or-filter> --reporter dot
```

## General Gotchas

- Do not call Telegram-only `sync_source` for YouTube. Use `sync_youtube_source`.
- Do not decode YouTube `metadata_zstd` with Telegram metadata helpers.
- Do not fetch full playlists in one unbounded preview call.
- Do not fetch comments without `max_comments`.
- Do not rely on `items.content_zstd IS NOT NULL` alone for analysis corpus.
- Do not mix Telegram and YouTube sources in one analysis group.
- Do not include YouTube playlist rows with `video_source_id IS NULL` in corpus.
- Do not use `item_id = 0` as a database item id.
- Keep old refs `sN-iM` and `sN-mM` working.
- Keep raw cookie values out of logs, events, errors, command args, and IPC responses.
- YouTube jobs are in memory for the MVP; restart does not restore them.
