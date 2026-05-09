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
docs: refresh session context after youtube part 5
```

## Current IDE Context

Latest IDE context reported by the user:

- Active file: `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`
- Open tabs:
  - `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`
  - `docs/superpowers/plans/2026-05-09-youtube-sources-04-comments-and-analysis.md`

## Workflow And Session Rules

- Use relevant Superpowers skills on future implementation turns.
- This session used:
  - `superpowers:using-superpowers`
  - `superpowers:executing-plans`
  - `superpowers:test-driven-development`
  - `superpowers:systematic-debugging`
  - `superpowers:verification-before-completion`
  - `superpowers:finishing-a-development-branch`
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
46a10d9 docs: mark youtube part 5 complete
67dd93a fix: write youtube cookies with netscape header
54ee015 feat: use youtube cookies for yt-dlp runs
3c03cca feat: add youtube settings UI
08012d0 feat: add youtube settings commands
45e1c30 feat: store youtube cookies securely
d103aea docs: refresh session context after youtube part 4
52891eb fix: add youtube sources on upgraded databases
a861fe0 docs: refresh session context after youtube part 4 task 4
87f12b2 feat: resolve youtube timestamp evidence
3e4f890 feat: load youtube analysis corpus
d55aa90 feat: enforce provider-specific analysis groups
```

After this request, `docs/session-context-2026-05-03.md` is modified and should be the only intended change unless the user asks for additional documentation cleanup or a commit.

## Important Status Summary

YouTube Sources Part 4 and Part 5 are functionally complete.

Part 4 completed:

- Comments ingest.
- Provider-safe analysis groups.
- YouTube corpus loading and playlist expansion.
- Timestamp trace refs and saved run snapshots.
- Manual verification.
- Legacy upgraded-database fix for `sources.telegram_source_kind NOT NULL`.

Part 5 completed:

- Optional YouTube cookie/auth storage through `SecretStoreState`.
- Validated Netscape cookie input with sanitized errors.
- Auth/non-auth `yt-dlp` execution with temporary cookie files and bounded timeouts.
- YouTube settings commands backed by `app_settings`.
- Dedicated frontend YouTube settings API module.
- Settings UI panel mounted on `/settings`.
- Real manual verification, including an authenticated preview path.
- Part 5 plan checkboxes physically marked complete.

Important caveats:

- The Part 4 plan file may still have unchecked boxes from an earlier doc caveat, even though Part 4 implementation and manual verification are complete.
- Part 5 plan file is fully checked off.
- The Tauri dev app was started during Part 5 manual verification and may still be running. Check before starting another instance.

## Active Plan

Current completed plan file:

```text
docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md
```

Part 5 goal:

- Add optional YouTube auth/cookie support.
- Add user-configurable YouTube sync settings.
- Keep non-secret settings in `app_settings`.
- Keep raw cookies only in OS secure storage and temporary backend files.
- Never expose raw cookies through IPC, logs, command args, app errors, job records, or UI.

Next plan candidate:

```text
docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md
```

## Part 5 Task 1: Cookie Secret Boundary

Commit:

```text
45e1c30 feat: store youtube cookies securely
```

Implemented:

- Created `src-tauri/src/youtube/cookies.rs`.
- Added `pub(crate) fn youtube_default_cookies_secret() -> String`.
- Stable secret key:

```text
youtube.auth.default.cookies
```

- Added `cookies` module in `src-tauri/src/youtube/mod.rs`.
- Implemented:
  - `read_youtube_cookies`
  - `save_youtube_cookies`
  - `clear_youtube_cookies`
  - `validate_netscape_cookie_file`
- Validation policy:
  - Reject empty or whitespace-only cookie text.
  - Accept blank lines.
  - Accept comments/header lines.
  - Treat `#HttpOnly_` rows as cookie rows after stripping only that prefix.
  - Require exactly 7 tab-separated fields.
  - Require non-empty domain and cookie name.
  - Require `TRUE`/`FALSE` for include-subdomains and secure.
  - Require path starting with `/`.
  - Require integer `expires`.
  - Allow empty cookie value.
  - Do not reject non-YouTube domains.
- Validation errors include line number and reason but never echo cookie value or line content.
- Extended `src-tauri/src/youtube/ytdlp.rs` with:
  - `YtdlpRunOptions { timeout, cookies }`
  - `run_ytdlp_with_options`
  - `ytdlp_command_args`
- `run_ytdlp(args)` still uses the existing 30s preview timeout and no cookies.
- Cookie content is written to a temporary file and passed as `--cookies <temp-path>`.
- Raw cookies are not placed on the process command line.
- Temp file is kept alive until the child process exits or times out.

Task 1 tests added:

- Cookie validation accepts Netscape rows and `#HttpOnly_` rows.
- Cookie validation accepts empty cookie values.
- Cookie validation rejects empty text and comments-only files.
- Invalid cookie errors do not contain `secret-value`.
- `InMemorySecretStore` read/write/delete roundtrip.
- Invalid cookie text is rejected before saving.
- Authenticated command args include `--cookies <temp-path>` but not cookie names or values.

Task 1 verification:

```powershell
cd src-tauri
cargo test youtube::cookies --lib
cargo test secret_store --lib
cargo test youtube::ytdlp --lib
git diff --check
```

Known results at Task 1 completion:

- `youtube::cookies`: 7 passed.
- `secret_store`: 8 passed.
- `youtube::ytdlp`: 3 passed at Task 1 time.
- `git diff --check`: no whitespace errors.

Note:

- The plan's literal `cargo test youtube::cookies secret_store --lib` command is not directly valid for Cargo because Cargo accepts one test filter. The checks were run as separate filters.

## Part 5 Task 2: YouTube Settings Commands

Commit:

```text
08012d0 feat: add youtube settings commands
```

Implemented backend:

- Created `src-tauri/src/youtube/settings.rs`.
- Added `settings` module in `src-tauri/src/youtube/mod.rs`.
- Registered commands in `src-tauri/src/lib.rs`.
- Defined app setting keys:
  - `youtube.auth.enabled`
  - `youtube.captions.preferred_language`
  - `youtube.sync.delay_between_requests_ms`
  - `youtube.sync.max_parallel_video_syncs`
  - `youtube.sync.max_parallel_comment_syncs`
  - `youtube.sync.pause_on_auth_challenge`
  - `youtube.sync.daily_soft_limit`
  - `youtube.sync.retry_backoff_ms`
  - `youtube.sync.stop_after_consecutive_failures`
- Added DTOs:
  - `YoutubeSettingsDto`
  - `YoutubeAuthStatusDto`
- DTOs serialize with camelCase for IPC.
- Default settings:

```text
authEnabled = false
preferredCaptionsLanguage = original
delayBetweenRequestsMs = 1000
maxParallelVideoSyncs = 1
maxParallelCommentSyncs = 1
pauseOnAuthChallenge = true
dailySoftLimit = 0
retryBackoffMs = 3000
stopAfterConsecutiveFailures = 3
```

- Validation ranges:
  - preferred captions language: `original` or 2-32 ASCII letters/digits/hyphen/underscore chars.
  - delay: `0..=60000`.
  - max parallel video syncs: `1..=4`.
  - max parallel comment syncs: `1..=2`.
  - daily soft limit: `0..=10000`.
  - retry backoff: `0..=300000`.
  - stop after consecutive failures: `1..=50`.
- Preferred captions language is normalized to lowercase unless it is `original`.
- Settings read policy:
  - Missing rows use defaults.
  - Invalid stored bool/int values return validation errors naming the setting key.
  - Loaded settings are validated before return.
- Settings write policy:
  - Validate whole DTO before writing anything.
  - Persist bools as `true`/`false`.
  - Persist integers as decimal strings.
  - Persist normalized captions language.
- Added helpers:
  - `default_youtube_settings`
  - `load_youtube_settings_from_pool`
  - `save_youtube_settings_to_pool`
  - `youtube_auth_status_from_state`
  - `save_youtube_cookies_to_state`
  - `clear_youtube_auth_in_state`
- Added auth status messages:
  - `Auth disabled`
  - `Cookies stored`
  - `No cookies configured`
- Commands:
  - `get_youtube_settings`
  - `save_youtube_settings`
  - `get_youtube_auth_status`
  - `save_youtube_cookies`
  - `clear_youtube_auth`

Implemented frontend:

- Created `src/lib/types/youtube.ts`.
- Created `src/lib/api/youtube-settings.ts`.
- Created `src/lib/api/youtube-settings.test.ts`.
- Frontend wrappers:
  - `getYoutubeSettings`
  - `saveYoutubeSettings`
  - `getYoutubeAuthStatus`
  - `saveYoutubeCookies`
  - `clearYoutubeAuth`
- YouTube settings wrappers intentionally live outside `src/lib/api/sources.ts`.

Task 2 tests added:

- Defaults load when `app_settings` rows are missing.
- Out-of-range values are rejected.
- Preferred captions language normalizes.
- DTO serializes with camelCase keys.
- Settings roundtrip through `app_settings`.
- Invalid settings do not partially write.
- Invalid stored settings return validation errors with key names.
- Saving cookies enables auth; clearing auth disables it.
- Vitest contract tests for all wrapper commands and payload shapes.

Task 2 verification:

```powershell
cd src-tauri
cargo test youtube::settings --lib
cargo test youtube::cookies --lib
cargo test secret_store --lib
cd ..
npm.cmd test -- youtube-settings
npm.cmd run check
git diff --check
```

Known results at Task 2 completion:

- `youtube::settings`: 8 passed.
- `youtube::cookies`: 7 passed.
- `secret_store`: 8 passed.
- `youtube-settings` Vitest: 4 passed.
- `svelte-check found 0 errors and 0 warnings`.
- `git diff --check`: no whitespace errors.

## Part 5 Task 3: Settings UI

Commit:

```text
3c03cca feat: add youtube settings UI
```

Implemented:

- Created `src/lib/components/settings/youtube-settings-panel.svelte`.
- Mounted `YoutubeSettingsPanel` from `src/routes/settings/+page.svelte`.
- Used existing local UI components as baseline:
  - `Badge`
  - `Button`
  - `CheckboxRow`
  - `Input`
  - `StatusMessage`
- Did not add new UI dependencies.
- On mount, panel loads settings and auth status via:

```ts
const [loadedSettings, loadedAuthStatus] = await Promise.all([
  getYoutubeSettings(),
  getYoutubeAuthStatus(),
]);
```

- UI controls include:
  - Enable YouTube auth.
  - Paste/update cookies.
  - Save cookies.
  - Cancel cookie edit.
  - Clear YouTube auth.
  - Preferred captions language.
  - Delay between requests.
  - Max parallel video syncs.
  - Max parallel comment syncs.
  - Pause on auth challenge.
  - Daily soft limit.
  - Retry backoff.
  - Stop after consecutive failures.
  - Save settings.
  - Reload.
- Cookie textarea is hidden by default.
- If cookies are stored, the panel shows only status and an `Update cookies` control.
- Stored cookie text is never rendered back into an input.
- Pasted cookies only live in local component state before save.
- After successful cookie save, the textarea state is cleared and auth status is reloaded.
- Empty/whitespace cookie textarea disables the save-cookies button; backend still validates and rejects empty direct IPC calls.
- Clear auth sets local `authEnabled` false after command returns.
- Numeric input constraints match backend ranges.

Svelte autofixer:

- Ran on `youtube-settings-panel.svelte`.
- Ran on `src/routes/settings/+page.svelte`.
- No issues.

Task 3 verification:

```powershell
npm.cmd test -- youtube-settings
npm.cmd run check
git diff --check
```

Known results:

- `youtube-settings` Vitest: 4 passed.
- `svelte-check found 0 errors and 0 warnings`.
- `git diff --check`: no whitespace errors.

## Cross-Cutting Part 5 Auth Path

Commit:

```text
54ee015 feat: use youtube cookies for yt-dlp runs
```

Reason:

- After Task 3, a review before manual verification found that `run_ytdlp_with_options` existed but real preview/sync paths still used unauthenticated `run_ytdlp`.
- The implementation was extended so stored cookies are actually used by real YouTube paths when auth is enabled.

Implemented:

- Added `load_youtube_auth_cookies_from_state(pool, secrets)` in `src-tauri/src/youtube/settings.rs`.
- Helper returns `None` when `youtube.auth.enabled = false`.
- Helper returns stored cookie text only when auth is enabled.
- `preview_youtube_source` and `add_youtube_source` now accept `AppHandle` and `SecretStoreState`.
- Preview/add source load cookie text from settings/secret state and call `run_ytdlp_with_options`.
- `fetch_video_metadata`, `fetch_playlist_metadata`, and `fetch_playlist_metadata_page` accept `cookies: Option<String>`.
- `fetch_transcript_for_video` accepts `cookies: Option<String>`.
- `fetch_comments_for_video` accepts `cookies: Option<String>`.
- Background YouTube sync jobs load cookies via `handle.state::<SecretStoreState>()` and pass them to:
  - metadata sync,
  - transcript sync,
  - comments sync,
  - playlist metadata sync.
- Added named timeout constants near callers:
  - `YOUTUBE_METADATA_TIMEOUT`
  - `YOUTUBE_CAPTION_DOWNLOAD_TIMEOUT`
  - `YOUTUBE_COMMENTS_FETCH_TIMEOUT`
- These currently use the existing 30s preview timeout constant, keeping all auth paths bounded.
- Added test:
  - auth cookies load only when auth is enabled.

Verification:

```powershell
cd src-tauri
cargo test youtube::settings --lib
cargo test youtube::preview --lib
cargo test youtube::metadata --lib
cargo test youtube::captions --lib
cargo test youtube::comments --lib
cargo test youtube::jobs --lib
cargo test youtube::ytdlp --lib
git diff --check
```

Known results:

- `youtube::settings`: 9 passed.
- `youtube::preview`: 2 passed.
- `youtube::metadata`: 5 passed.
- `youtube::captions`: 9 passed.
- `youtube::comments`: 5 passed.
- `youtube::jobs`: 4 passed.
- `youtube::ytdlp`: 3 passed at that checkpoint.
- `git diff --check`: no whitespace errors.

## Part 5 Manual Verification Bug Fix

Commit:

```text
67dd93a fix: write youtube cookies with netscape header
```

Manual verification found a real bug:

```text
YouTube preview failed: '<temp path>' does not look like a Netscape format cookies file
```

Root cause:

- The app validation intentionally accepted a bare valid Netscape cookie row.
- `yt-dlp` requires the temp cookie file to include the standard Netscape header.
- The app wrote the bare row unchanged into the temp file.

Fix:

- Added `ytdlp_cookie_file_content(cookies: &str) -> String`.
- If the user-provided cookie text already contains `# Netscape HTTP Cookie File`, it is preserved unchanged.
- If the header is missing, the app prepends:

```text
# Netscape HTTP Cookie File
```

- `run_ytdlp_with_options` writes the normalized file content to the temporary file.

Tests:

- `cookie_file_content_adds_netscape_header_when_missing`
- `cookie_file_content_preserves_existing_netscape_header`

RED/GREEN:

- New regression test first failed because `ytdlp_cookie_file_content` did not exist.
- After implementation, `cargo test youtube::ytdlp --lib` passed.

Verification:

```powershell
cd src-tauri
cargo test youtube::ytdlp --lib
git diff --check
```

Known results:

- `youtube::ytdlp`: 5 passed.
- `git diff --check`: no whitespace errors.

## Part 5 Plan Completion Docs

Commit:

```text
46a10d9 docs: mark youtube part 5 complete
```

Implemented:

- Updated `docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md`.
- Marked all Task 1, Task 2, Task 3, and Manual Verification checkboxes complete.
- `rg "^- \[ \]" docs/superpowers/plans/2026-05-09-youtube-sources-05-auth-and-settings.md -n` returned no matches.

## Manual Verification Of Part 5

Manual verification was performed with Tauri MCP bridge and the dev app.

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

The app may still be running. Check with:

```text
mcp__tauri__.driver_session status
```

Manual verification URL:

```text
http://localhost:1420/settings
```

Reliable public preview video:

```text
https://www.youtube.com/watch?v=M7lc1UVf-VE
```

Manual checks completed:

- Cleared YouTube auth through `clear_youtube_auth`.
- Confirmed response:

```json
{"enabled":false,"hasCookies":false,"message":"Auth disabled"}
```

- Previewed public video without auth; preview succeeded.
- Direct IPC save with whitespace cookie text returned validation error:

```text
YouTube cookies cannot be empty; use clear_youtube_auth to remove stored cookies
```

- Direct IPC save with malformed space-separated cookie text returned sanitized validation error:

```text
Invalid YouTube cookie file at line 1: expected 7 tab-separated fields
```

- Confirmed malformed error did not contain `secret-value`.
- Saved valid cookie text:

```text
.youtube.com	TRUE	/	TRUE	1893456000	SID	secret-value
```

- Confirmed `save_youtube_cookies` response:

```json
{"enabled":true,"hasCookies":true,"message":"Cookies stored"}
```

- Confirmed `get_youtube_auth_status` response:

```json
{"enabled":true,"hasCookies":true,"message":"Cookies stored"}
```

- Reloaded `/settings`.
- Confirmed UI showed `Cookies stored`.
- Confirmed UI did not contain `secret-value`.
- Confirmed no textarea values existed after reload until editing cookies.
- Clicked `Update cookies`; textarea appeared empty and `Save cookies` was disabled while empty.
- Authenticated preview initially failed due missing Netscape header; that was fixed in commit `67dd93a`.
- After fix, authenticated preview succeeded with stored cookies.
- Cleared auth again.
- Confirmed final auth status and settings:

```json
{
  "settings": {
    "authEnabled": false,
    "dailySoftLimit": 0,
    "delayBetweenRequestsMs": 1000,
    "maxParallelCommentSyncs": 1,
    "maxParallelVideoSyncs": 1,
    "pauseOnAuthChallenge": true,
    "preferredCaptionsLanguage": "original",
    "retryBackoffMs": 3000,
    "stopAfterConsecutiveFailures": 3
  },
  "status": {
    "enabled": false,
    "hasCookies": false,
    "message": "Auth disabled"
  }
}
```

- Reloaded `/settings`.
- Confirmed UI showed `Auth disabled`.
- Confirmed UI did not contain `secret-value`.

Notes:

- The manual cookie used for verification was a fake value (`secret-value`) with a valid Netscape shape. It was cleared at the end.
- IPC monitor did not capture command logs in this session (`ipc_get_captured` returned `[]`), so the `--cookies <temp-path>` behavior was verified through code, unit tests, and the real authenticated preview path after fixing the temp file header.
- The preview timeout policy was verified by code path: authenticated preview uses `YTDLP_PREVIEW_TIMEOUT` through `run_ytdlp_with_options`, same as unauthenticated preview.

## Final Verification For Part 5

Fresh final verification after all Part 5 commits:

```powershell
cd src-tauri
cargo test --lib
cd ..
npm.cmd test -- youtube-settings
npm.cmd run check
git status --short
```

Results:

- `cargo test --lib`: 280 passed, 0 failed.
- `npm.cmd test -- youtube-settings`: 1 test file passed, 4 tests passed.
- `npm.cmd run check`: `svelte-check found 0 errors and 0 warnings`.
- `git status --short`: no output before this session-context rewrite.

## Current Local Data / Runtime State

Existing local data from Part 4 manual verification still exists unless the user deleted it:

- YouTube source `28`: Rick Astley test video.
- YouTube source `30`: `YouTube Developers Live: Embedded Web Player Customization`.
- YouTube playlist source `32`: `Manual Verification Playlist`.
- Analysis group `1`: `Manual Verification YouTube`.
- Prompt template `2`: `Manual YouTube description trace`.
- Analysis runs `54`, `55`, `56`, `57`, `58`.
- Source jobs including `source-job-2`, `source-job-3`, `source-job-4`.

Part 5 local data:

- Fake manual cookie value used for verification was saved temporarily and cleared at the end.
- Final checked auth state was:
  - `authEnabled = false`
  - `hasCookies = false`
  - message `Auth disabled`
- Tauri dev app may still be running from manual verification.

Do not delete local verification data unless the user asks.

## Recommended Next Steps

If the user wants to commit this session-context rewrite:

```powershell
git add docs/session-context-2026-05-03.md
git commit -m "docs: refresh session context after youtube part 5"
```

If the user wants implementation to continue:

1. Move to Part 6.
2. Read `docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md`.
3. Use `superpowers:executing-plans`.
4. Use `superpowers:test-driven-development` for behavior changes.
5. Keep one commit per task if continuing the previous working style.

## Part 6 Reminder: UI Hardening And Docs

Plan file:

```text
docs/superpowers/plans/2026-05-09-youtube-sources-06-ui-hardening-docs.md
```

Known reminders from earlier context:

- Add runtime/detail APIs:
  - `src-tauri/src/youtube/runtime.rs`
  - `src-tauri/src/youtube/detail.rs`
- `get_youtube_runtime_status` should run `yt-dlp --version` with a 5s timeout.
- Source summaries should use provider-neutral `synced items`, not `synced messages`.
- Hide Telegram topic controls for YouTube sources.

## Useful Verification Commands

Backend:

```powershell
cd src-tauri
cargo test youtube::cookies --lib
cargo test youtube::settings --lib
cargo test youtube::ytdlp --lib
cargo test youtube::preview --lib
cargo test youtube::metadata --lib
cargo test youtube::captions --lib
cargo test youtube::comments --lib
cargo test youtube::jobs --lib
cargo test analysis::groups --lib
cargo test analysis::store --lib
cargo test analysis::corpus --lib
cargo test analysis::report --lib
cargo test analysis::trace --lib
cargo test sources::items --lib
cargo test sources::store --lib
cargo test --lib
```

Frontend:

```powershell
cd g:\Develop\Extractum
npm.cmd test -- youtube-settings
npm.cmd test -- analysis-source-groups
npm.cmd test -- analysis-run-workflow analysis-state analysis-runs
npm.cmd test -- analysis-trace analysis-trace-workflow
npm.cmd run check
```

Whitespace:

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
- Stored cookie text must never render back into the settings UI.
- `save_youtube_settings` must not read or write raw cookie content.
- `get_youtube_auth_status` may check only whether cookies exist.
- `load_youtube_auth_cookies_from_state` is the backend boundary for reading raw cookies for temp-file execution.
- `yt-dlp` temp cookie files need a Netscape header even when the user pasted only valid cookie rows.
- YouTube jobs are in memory for the MVP; restart does not restore them.
