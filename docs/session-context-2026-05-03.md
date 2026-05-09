# Session Context: YouTube Sources MVP

Updated: 2026-05-09
Workspace: `g:\Develop\Extractum`
Shell: PowerShell
Timezone: Europe/Minsk
User language: Russian
Branch: `main`

This file is the restore point for the current Codex session. A future session should be able to continue YouTube Sources implementation without reading the full chat history.

## Current User Request

The latest user request was:

Rewrite `docs\session-context-2026-05-03.md` with all information needed to restore the current session. The file may be overwritten. Provide a commit message.

The user did not explicitly ask to commit the context file.

## Current IDE Context

Latest IDE context reported by the user:

- Active file: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`
- Open tab: `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`

## Workflow And Session Rules

- Use relevant Superpowers skills on future implementation turns. This session already used:
  - `superpowers:using-superpowers`
  - `superpowers:using-git-worktrees`
  - `superpowers:executing-plans`
  - `superpowers:test-driven-development`
  - `superpowers:verification-before-completion`
- Do not spawn subagents unless the user explicitly asks for delegation or parallel agents.
- Work is being done directly on `main`; the user had allowed this earlier and then asked to execute Part 4 fully.
- Current user instruction for Part 4: execute all of Part 4 fully, with one commit per task.
- `git add` and `git commit` may require escalation. Use `sandbox_permissions: "require_escalated"` for those commands when needed.
- Use `rg` / `rg --files` for search.
- Use `apply_patch` for manual file edits.
- In PowerShell, prefer `npm.cmd`.
- Svelte edits should be checked with Svelte autofixer before finalizing a task.
- If verification is interrupted, do not claim completion; rerun the interrupted commands.

## Current Git State

Latest commits before this context rewrite:

```text
3e4f890 feat: load youtube analysis corpus
d55aa90 feat: enforce provider-specific analysis groups
138b6c1 docs: refresh session context after youtube part 4 task 1
3707090 feat: ingest youtube comments
50ac301 test: update youtube sync disabled expectation
```

Important Part 4 commits already completed:

```text
3707090 feat: ingest youtube comments
d55aa90 feat: enforce provider-specific analysis groups
3e4f890 feat: load youtube analysis corpus
```

Current working tree before this context-file edit had uncommitted Task 4 partial work in:

```text
src-tauri/src/analysis/chat.rs
src-tauri/src/analysis/corpus.rs
src-tauri/src/analysis/mod.rs
src-tauri/src/analysis/models.rs
src-tauri/src/analysis/report.rs
src-tauri/src/analysis/store.rs
src-tauri/src/analysis/trace.rs
src/lib/analysis-state.test.ts
src/lib/analysis-trace-workflow.test.ts
src/lib/api/analysis-trace.test.ts
src/lib/components/analysis/trace-panel.svelte
src/lib/types/analysis.ts
```

After this request, `docs/session-context-2026-05-03.md` is also modified.

The current uncommitted Task 4 diff is intentionally not committed yet.

## Suggested Commit Message

For this context-file update:

```text
docs: refresh session context during youtube part 4 task 4
```

Do not use this message for the Task 4 implementation commit. The planned Task 4 implementation commit message remains:

```text
feat: resolve youtube timestamp evidence
```

## yt-dlp Status

The user later stated:

```text
yt-dlp установлен.
```

This was verified in the session:

```powershell
yt-dlp --version
```

Result:

```text
2026.03.17
```

Earlier notes about `yt-dlp` being missing are obsolete.

## Plan In Scope

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

## Part 4 Task 1 Completed

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
  - warn on missing/unparseable comment timestamp.
- Added `upsert_youtube_comment_item`.
- Persisted YouTube comments as:
  - `item_kind = youtube_comment`
  - `external_id = comment:<comment_id>`
  - `content_kind = text_only`
  - `reaction_count = like_count`
- Comment upsert updates existing rows instead of duplicating.
- Jobs run comments only when `YoutubeSyncOptions.comments = true`.

Verification:

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

## Part 4 Task 2 Completed

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
- In-memory test schemas updated for `analysis_source_groups.source_type`.

Implemented frontend:

- Added `AnalysisGroupSourceType`.
- Added `source_type` to analysis source/group TS types.
- Added `sourceType` to create/update group inputs.
- Group editor now has a Telegram/YouTube type select.
- Group editor filters candidate sources by selected provider.
- Route state now keeps `groupSourceType`.
- Selecting an existing group copies its `source_type`.
- Starting a new group defaults to `telegram`.
- Save calls pass group type.

Task 2 RED checks:

- `cargo test analysis::groups --lib` initially failed because provider validation was missing.
- `vitest` workflow test initially failed because `sourceType` was not passed.

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

## Part 4 Task 3 Completed

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
- `ReportRunInput` now carries `corpus_request`.
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

## Part 4 Task 4 In Progress

Planned commit message:

```text
feat: resolve youtube timestamp evidence
```

Task 4 files in plan:

- `src-tauri/src/analysis/models.rs`
- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/trace.rs`
- `src-tauri/src/analysis/corpus.rs`
- `src/lib/types/analysis.ts`
- `src/lib/api/analysis-trace.ts`
- `src/lib/analysis-trace-workflow.test.ts`

Actual uncommitted files currently modified:

```text
src-tauri/src/analysis/chat.rs
src-tauri/src/analysis/corpus.rs
src-tauri/src/analysis/mod.rs
src-tauri/src/analysis/models.rs
src-tauri/src/analysis/report.rs
src-tauri/src/analysis/store.rs
src-tauri/src/analysis/trace.rs
src/lib/analysis-state.test.ts
src/lib/analysis-trace-workflow.test.ts
src/lib/api/analysis-trace.test.ts
src/lib/components/analysis/trace-panel.svelte
src/lib/types/analysis.ts
```

### Task 4 Work Already Done

Added RED tests in `src-tauri/src/analysis/trace.rs` for:

- Exact YouTube timestamp refs.
- Milliseconds to integer YouTube timestamp seconds.
- Appending `t=<seconds>` to YouTube URLs.
- Falling back from old refs to base item refs.
- Serializing YouTube fields as `null` for Telegram refs.
- Marking YouTube description refs with `item_id = 0` as synthetic.

The initial RED failure showed missing fields on:

- `CorpusMessage`
- `AnalysisTraceRef`

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

- `persist_run_snapshot` now inserts:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`
- `load_run_snapshot_messages` now selects those fields and restores them into `CorpusMessage`.
- Live item loader now joins `sources` and selects:
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
  - `item_kind = youtube_description`
  - `source_type = youtube`
  - `source_subtype = video`
  - `metadata_zstd` with YouTube metadata

Implemented Rust trace changes:

- `build_trace_refs` resolves exact refs first.
- Fallback matches same `source_id` and `item_id`, keeping old `sN-iM` and `sN-mM` refs working.
- Timestamp refs parse `@<ms>ms`.
- `youtube_timestamp_seconds` is `ms / 1000`.
- `youtube_url` appends `t=<seconds>`.
- `youtube_display_label` formats as:
  - `"Video title at 12:34"` when title exists.
  - `"YouTube at 12:34"` otherwise.
- `is_synthetic = true` when:
  - `message.item_id == 0`, or
  - `message.item_kind == Some("youtube_description")`

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
  - "Saved synthetic evidence" for synthetic refs.

### Task 4 Verification Already Run

Before the latest tiny corpus test fixture assertion was added, these passed:

```powershell
cd src-tauri
cargo test analysis::trace --lib
cargo test analysis::corpus --lib
cargo test analysis::store --lib
```

Known results:

- `analysis::trace`: 7 passed.
- `analysis::corpus`: 19 passed.
- `analysis::store`: 3 passed.

After TypeScript/frontend changes, this passed:

```powershell
npm.cmd test -- analysis-trace analysis-trace-workflow
```

Result:

- 2 files passed.
- 13 tests passed.

### Important Interruption

After adding an extra assertion to prove run snapshots roundtrip YouTube metadata, `cargo fmt` was run. Then a parallel rerun of:

```powershell
cargo test analysis::trace --lib
cargo test analysis::corpus --lib
cargo test analysis::store --lib
```

was started, but the user intentionally interrupted the turn. Tool output said the parallel commands were aborted by user. Treat final Task 4 verification as not complete until rerun.

### Task 4 Next Steps

Continue from the current uncommitted diff. Recommended next actions:

1. Inspect current status:

```powershell
git status --short
```

2. Rerun Rust checks after the final fixture/assertion edits:

```powershell
cd src-tauri
cargo test analysis::trace --lib
cargo test analysis::corpus --lib
cargo test analysis::store --lib
```

3. Run frontend checks:

```powershell
cd ..
npm.cmd test -- analysis-trace analysis-trace-workflow
npm.cmd run check
```

4. Because `trace-panel.svelte` was modified, run Svelte autofixer on it before finalizing if possible.

5. Run:

```powershell
git diff --check
```

6. If all pass, commit Task 4:

```powershell
git add src-tauri/src/analysis src/lib/types/analysis.ts src/lib/api/analysis-trace.ts src/lib/analysis-trace-workflow.test.ts src/lib/analysis-state.test.ts src/lib/api/analysis-trace.test.ts src/lib/components/analysis/trace-panel.svelte
git commit -m "feat: resolve youtube timestamp evidence"
```

Note: Include `src-tauri/src/analysis/chat.rs`, `src-tauri/src/analysis/mod.rs`, and `src-tauri/src/analysis/report.rs` because test fixtures and trace serialization were updated there.

## Potential Task 4 Risks To Recheck

- `load_youtube_transcript_segment_messages` now depends on `youtube_transcript_segments`. Verify fallback behavior if a YouTube transcript item has no segment rows. The plan wants segment-level messages, so zero rows may be acceptable, but it changes Task 3 tests and behavior.
- `load_item_messages` uses a raw `item_kind_filter` string. It is currently passed only static internal strings, so it is not user-controlled.
- `append_youtube_timestamp` always appends `&t=` when URL already has query params. It does not remove an existing `t` param. Tests currently expect appending.
- `find_trace_message` fallback returns the first corpus message matching source/item. With multiple transcript segments for one item, old base refs map to the first segment by corpus order.
- `youtube_display_label` for synthetic description refs currently may show only title without timestamp.
- Need ensure all `AnalysisTraceRef` constructors in Rust/TS were updated.
- Need ensure all `CorpusMessage` literals were updated:

```powershell
rg -n "CorpusMessage \{" src-tauri/src
```

- Need ensure all in-memory `items` schemas used by touched tests include `media_metadata_zstd` if they exercise `load_corpus_messages`.
- Need ensure all in-memory `analysis_run_messages` schemas include:
  - `item_kind`
  - `source_type`
  - `source_subtype`
  - `metadata_zstd`

## Part 4 Remaining After Task 4

After committing Task 4, Part 4 still needs manual verification from the plan:

- Sync comments for a public video with transcript data and confirm top-level comments plus replies appear as `youtube_comment` items.
- Rerun comment sync and confirm item count stays stable.
- Create a YouTube-only analysis group and confirm adding a Telegram source is rejected before membership write.
- Run analysis for one YouTube video in all three corpus modes:
  - `transcript_only`
  - `transcript_description`
  - `transcript_description_comments`
- Run analysis for a playlist with at least one unavailable/unlinked row and confirm only linked, non-removed child videos enter corpus.
- Save a YouTube analysis run, resync transcript/comments, reopen saved run and confirm old snapshot excerpt, metadata, and trace resolution remain unchanged.
- Open timestamp trace ref and synthetic description ref:
  - timestamp ref should produce YouTube URL with `t=<seconds>`;
  - synthetic ref should render saved excerpt without source-item lookup using `item_id = 0`.

Manual verification may require the app running, network access, configured provider profile/LLM, and a suitable public YouTube video/playlist.

## Verification Command Reference

Useful commands:

```powershell
yt-dlp --version
```

```powershell
cd src-tauri
cargo test analysis::groups --lib
cargo test analysis::store --lib
cargo test analysis::corpus --lib
cargo test analysis::report --lib
cargo test analysis::trace --lib
cargo test youtube::comments --lib
cargo test youtube::jobs --lib
cargo test sources::items --lib
```

```powershell
cd g:\Develop\Extractum
npm.cmd test -- analysis-source-groups
npm.cmd test -- analysis-run-workflow analysis-state analysis-runs
npm.cmd test -- analysis-trace analysis-trace-workflow
npm.cmd run check
git diff --check
```

If `npm.cmd test -- <filter>` has environment issues, use:

```powershell
node --trace-uncaught ./node_modules/vitest/vitest.mjs run <path-or-filter> --reporter dot
```

## Part 5 Reminder: Auth And Settings

Plan file:

```text
docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md
```

Key reminders:

- Raw cookies are allowed only inside backend code that writes temporary cookie files.
- Raw cookies must never appear in logs, IPC responses, job records, events, or `AppError.message`.
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
