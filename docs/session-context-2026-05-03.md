# Session Context - 2026-05-09

This file contains enough context to resume the current Extractum session without relying on chat history.

## Environment

- Workspace: `G:\Develop\Extractum`
- Shell: PowerShell
- User timezone: `Europe/Minsk`
- User language preference in this session: Russian
- Current branch at the time this context was written: `main`
- Working tree before this context-file update: clean
- Current date during session: 2026-05-09

Important active working rules:

- Use `rg` for search.
- Use `apply_patch` for manual file edits.
- Do not revert user changes unless explicitly asked.
- Git operations that mutate `.git` may require escalation under the sandbox.
- When Superpowers skills apply, read and follow their `SKILL.md` instructions.
- For Svelte code, use Svelte docs/autofixer when writing or changing Svelte components.

## Current Git State

Latest commits observed before rewriting this file:

```text
7524895 (HEAD -> main) fix(analysis): prevent WebView OOM from tracked run loading
5558c45 docs(llm): document report preflight limits
a2a117b test(analysis): cover report preflight validation
a8ce2fd feat(analysis): enforce report preflight limits
2dc5058 feat(analysis): preflight report corpus size
53fd0b0 feat(analysis): add report preflight policy types
38e453f docs(session): capture llm preflight planning context
08e8c79 docs(llm): address preflight policy review
```

Status before this file update:

```text
## main
```

The feature branch `feat/analysis-preflight-limits` was merged into `main` locally and deleted earlier in this session.

## Completed Secure Storage Work

The secure secret storage work is complete and merged into `main`.

Core behavior:

- `src-tauri/src/secret_store.rs` owns OS secure storage through Rust `keyring`.
- Service name: `org.ai.extractum`.
- LLM API keys are stored outside `app_settings`.
- Telegram account `api_hash` values are stored outside SQLite.
- Stable secret ids:
  - `llm.profile.<profile_id>.api_key`
  - `telegram.account.<account_id>.api_hash`
  - `telegram.account.<account_id>.session_key`
- Legacy plaintext values migrate lazily:
  - write secure secret first;
  - blank/delete plaintext only after successful secure write;
  - fail closed and leave legacy plaintext untouched if secure storage fails.

## Completed Telegram Session Encryption

Telegram session JSON encryption is complete and merged into `main`.

Relevant files:

- `src-tauri/src/telegram_session_store.rs`
- `src-tauri/src/telegram.rs`
- `src-tauri/src/accounts.rs`
- `src-tauri/src/secret_store.rs`
- `src-tauri/Cargo.toml`

Behavior:

- Session files remain in app data as `telegram_<account_id>.session.json`.
- File contents are encrypted JSON envelopes:

```json
{
  "version": 1,
  "algorithm": "XChaCha20-Poly1305",
  "nonce": "<base64-url-no-pad nonce>",
  "ciphertext": "<base64-url-no-pad ciphertext>"
}
```

- The encryption key is a random 256-bit per-account session key stored in OS secure storage under `telegram.account.<account_id>.session_key`.
- Associated data:

```text
org.ai.extractum.telegram.session.v1.account.<account_id>
```

- Legacy plaintext session JSON migrates lazily on load after successful parse and successful keyring write.
- If encrypted file exists but the key is missing, loading fails closed instead of falling back to `MemorySession::default()`.
- Wrong account id fails decryption through associated data.
- Account logout clears session file and session key.
- Account deletion clears runtime/session artifacts and then deletes the Telegram `api_hash` secret, surfacing cleanup errors after row/runtime cleanup.

Manual validation observed:

- `telegram_1.session.json` used encrypted envelope format.
- No plaintext `home_dc`, `dc_options`, or `updates_state` was present.
- UI reached `Account ready` and `This account is ready to sync sources.`
- Private supergroup source `WBChat` was visible:
  - category `Life`
  - kind `supergroup`
  - `73102 msgs`
  - membership `member`
- Sync on private supergroup `WBChat` succeeded without re-login.
- User confirmed `WBChat` is private.
- User later performed logout and re-login, and the session file was again encrypted.
- Manual account delete cleanup was skipped by user decision; automated tests cover it.

## Completed LLM Concurrency / Analysis Preflight Work

This track is complete, merged into `main`, and verified.

Original approved limits:

- `max_messages_per_run = 10_000`
- `max_chunks_per_run = 80`
- `max_estimated_input_chars_per_run = 1_500_000`
- `max_background_requests_per_run = 80`

Completed implementation commits:

```text
53fd0b0 feat(analysis): add report preflight policy types
2dc5058 feat(analysis): preflight report corpus size
a8ce2fd feat(analysis): enforce report preflight limits
a2a117b test(analysis): cover report preflight validation
5558c45 docs(llm): document report preflight limits
```

Main implemented behavior:

- Scheduler policy remains:
  - `2` running LLM requests per `(provider, profile)`;
  - interactive requests jump ahead of background requests inside the same scheduler key;
  - requests with different provider/profile keys may run independently;
  - cancellation remains request-scoped or run-scoped.
- Analysis report runs now get backend preflight before run insertion.
- Preflight estimates:
  - selected source ids;
  - eligible text message count;
  - estimated input chars;
  - estimated chunks;
  - configured limits.
- Oversized scopes return `AppError::validation` before inserting an `analysis_runs` row.
- Empty corpus is rejected before insertion, with the existing user-facing validation message.
- Passing preflight emits early progress summary:

```text
Preflight passed: {message_count} documents, {estimated_chunks} estimated chunks, {estimated_input_chars} estimated input characters.
```

Important files changed in this track:

- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/report.rs`
- `docs/backlog.md`
- `docs/project.md`
- `docs/design-document.md`
- `docs/architecture-deep-dive.md`

Post-merge verification on `main`:

```powershell
cargo test
npm.cmd test
npm.cmd run check
cargo fmt --check
git diff --check
```

Observed results:

- `cargo test`: 195 passed.
- `npm.cmd test`: 196 passed.
- `npm.cmd run check`: 0 errors, 0 warnings.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.

## OmniRoute / Invalid API Key Debugging

During the preflight work, the user reported:

```text
OpenAI-compatible request failed with HTTP 401: Invalid API key ("AUTH_002")
```

User clarified they were using OmniRoute and pasted OmniRoute logs showing successful upstream activity.

Investigation:

- `openai_compat.rs` sends requests to `{base_url}/chat/completions` with `Authorization: Bearer <saved key>`.
- Local non-secret settings showed:
  - `llm.active_provider_profile=test_profile`
  - `llm.profile.test_profile.base_url=http://localhost:20128/v1`
  - `llm.profile.test_profile.default_model=gemini/gemini-3-flash-preview`
  - `llm.profile.test_profile.provider=omniroute`
- No legacy `llm.%.api_key` rows were found in SQLite.
- Windows Credential Manager had stored credentials:
  - `llm.profile.default.api_key.org.ai.extractum`
  - `llm.profile.test_profile.api_key.org.ai.extractum`

Conclusion:

- Root cause was most likely a stale/incorrect saved OS secure-storage key for `llm.profile.test_profile.api_key`, not endpoint routing or analysis preflight.

User action:

- User updated/cleared/saved the key through settings.
- User confirmed: `заработало`.

## WebView Out Of Memory Investigation And Fix

The user asked to connect to the running app through Tauri MCP and investigate intermittent out-of-memory failures.

MCP app state:

- App identifier: `org.ai.extractum`
- App name: `extractum`
- Tauri version: `2.10.3`
- Window URL during investigation: `http://localhost:1420/analysis`
- Runtime: Windows / Edge WebView2

Investigation skill used:

- `superpowers:systematic-debugging`

Important report file:

- `docs/superpowers/specs/2026-05-09-analysis-webview-oom-investigation.md`

Root cause and fix commit:

```text
7524895 fix(analysis): prevent WebView OOM from tracked run loading
```

### OOM Symptoms

The intermittent OOM happened in the WebView renderer, not in Rust.

Evidence:

- `extractum.exe` stayed around 50-60 MB.
- `msedgewebview2.exe` renderer grew into multi-gigabyte memory usage.
- DOM was small:
  - around 487-578 nodes;
  - body text around 11 KB.
- The local DB did not contain huge saved artifacts:
  - DB size about 60.7 MB;
  - `items`: 89,488 rows;
  - `analysis_runs`: 8 rows;
  - `analysis_chat_messages`: 0 rows;
  - largest saved report/trace artifacts only a few KB.

Route comparison:

- `/settings` stayed normal:
  - JS heap around 8-12 MB;
  - renderer working set around 150-210 MB.
- `/analysis` before the fix:
  - JS heap reached about 3.0 GB;
  - renderer working set reached 5+ GB.

Passive memory reproduction before fix:

```text
07:20:21  WorkingSetMB 1235.7  PrivateMB 1195.6
07:20:26  WorkingSetMB 1345.5  PrivateMB 1305.6
07:20:31  WorkingSetMB 1659.7  PrivateMB 1620.5
07:20:36  WorkingSetMB 2009.1  PrivateMB 1972.0
07:20:41  WorkingSetMB 2351.9  PrivateMB 2316.8
07:20:46  WorkingSetMB 2690.6  PrivateMB 2656.2
07:20:51  WorkingSetMB 3018.3  PrivateMB 2983.6
07:20:56  WorkingSetMB 3297.8  PrivateMB 3266.2
07:21:16  WorkingSetMB 3455.8  PrivateMB 3420.5
```

IPC monitoring did not show a flood of `list_analysis_runs` calls, reducing confidence in a backend polling-loop explanation.

### Root Cause

Problematic file:

- `src/routes/analysis/+page.svelte`

Problematic pattern:

```ts
$effect(() => {
  if (historyScopeParams === null) {
    runs = [];
    return;
  }

  void loadRuns();
});
```

In Svelte 5, `$effect` tracks synchronous reads inside its body, including reads inside functions it calls. `loadRuns()` delegates to workflow code that reads and patches route/workflow state. Calling it directly from the `$effect` allowed those internal reads to become dependencies of the effect.

The intended dependency was only `historyScopeParams`.

### Fix

Changed:

```ts
import { onMount } from "svelte";
```

To:

```ts
import { onMount, untrack } from "svelte";
```

And changed:

```ts
void loadRuns();
```

To:

```ts
void untrack(() => loadRuns());
```

This keeps the effect dependent on `historyScopeParams` while preventing state reads inside `loadRuns()` from being tracked.

### OOM Fix Verification

Commands run:

```powershell
npm.cmd run check
npm.cmd test -- analysis
git diff --check
```

Observed:

- `npm.cmd run check`: 0 errors, 0 warnings.
- `npm.cmd test -- analysis`: 16 files passed, 160 tests passed.
- `git diff --check`: no whitespace errors; only Git line-ending warning for `src/routes/analysis/+page.svelte`.

Live memory verification after fix:

Initial post-fix `/analysis` sample:

- JS heap: about 16-17 MB.
- Renderer working set: about 216 MB, then settled lower.

Passive sampling after fix:

```text
07:24:46  WorkingSetMB 179.3  PrivateMB 136.3
07:24:51  WorkingSetMB 162.8  PrivateMB 120.1
07:24:56  WorkingSetMB 162.8  PrivateMB 120.1
07:25:01  WorkingSetMB 162.8  PrivateMB 120.3
07:25:06  WorkingSetMB 157.9  PrivateMB 115.4
07:25:11  WorkingSetMB 157.9  PrivateMB 115.4
07:25:16  WorkingSetMB 157.9  PrivateMB 115.4
07:25:21  WorkingSetMB 157.9  PrivateMB 115.4
07:25:26  WorkingSetMB 157.9  PrivateMB 115.4
07:25:31  WorkingSetMB 157.9  PrivateMB 115.4
07:25:36  WorkingSetMB 157.5  PrivateMB 115.0
07:25:41  WorkingSetMB 157.5  PrivateMB 115.0
```

Final direct WebView check:

```text
JS heap total: about 21 MB
JS heap used:  about 17 MB
Renderer working set: about 172.8 MB
Renderer private memory: about 131.7 MB
```

## Current Files Of Interest

Most relevant files for recent work:

- `docs/session-context-2026-05-03.md`
- `docs/superpowers/specs/2026-05-09-analysis-webview-oom-investigation.md`
- `src/routes/analysis/+page.svelte`
- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/llm/scheduler.rs`

## Superpowers Skills Used In This Session

Skills read/used during the broader session:

- `superpowers:using-superpowers`
- `superpowers:brainstorming`
- `superpowers:writing-plans`
- `superpowers:executing-plans`
- `superpowers:test-driven-development`
- `superpowers:verification-before-completion`
- `superpowers:systematic-debugging`
- `superpowers:finishing-a-development-branch`
- `superpowers:receiving-code-review`

For future code changes:

- Use `superpowers:systematic-debugging` for bugs/performance failures.
- Use `superpowers:test-driven-development` for behavior changes.
- Use `superpowers:verification-before-completion` before claiming fixes are complete.

## User Decisions And Preferences

User decisions in this session:

- Preferred branch workflow over git worktree for the preflight feature.
- Approved local merge back to `main`.
- Confirmed OmniRoute key issue was fixed.
- Asked to connect to the running app through MCP to investigate OOM.
- Asked to write a detailed OOM investigation report to file.
- Asked to overwrite this session context file with enough information to restore the session.

User prefers Russian for interactive responses.

## Current Request Being Fulfilled

Latest user request:

- Overwrite `docs\session-context-2026-05-03.md` with all information needed to restore the current session context.
- Form a commit message.

This file was overwritten to satisfy that request.

Suggested commit message for this context-file update:

```text
docs(session): refresh current session recovery context

Overwrite the session recovery note with the current 2026-05-09 state,
including the merged analysis preflight work, OmniRoute key debugging,
and the WebView OOM investigation/fix.

Capture the latest main branch state, verification evidence, important
files, and follow-up context so a future session can resume without
depending on chat history.
```
