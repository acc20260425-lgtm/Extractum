# Session Context - 2026-05-08

This file captures enough context to resume the current Extractum session without relying on chat history.

## Environment

- Workspace: `G:\Develop\Extractum`
- Shell: PowerShell
- User timezone during session: `Europe/Minsk`
- User language preference in this session: Russian
- Current branch at capture time: `main`
- Working tree at capture time: clean
- Important active working rules:
  - use `rg` for search;
  - use `apply_patch` for manual edits;
  - do not revert user changes;
  - git index operations may require escalation because `.git/index.lock` can be blocked by sandbox permissions;
  - when Superpowers skills apply, follow their gates exactly.

## Latest Git State

Latest commits at capture time:

```text
08e8c79 docs(llm): address preflight policy review
96c8b57 docs(llm): plan analysis preflight limits
be8bd24 docs(llm): design analysis preflight limits
1b797bf docs(validation): record telegram re-login session check
6922606 docs(validation): record telegram runtime live results
e867b1b docs(validation): plan telegram runtime live checks
bc053e5 style: format telegram session store
064a8d7 docs(security): document encrypted telegram sessions
306b4c3 feat(security): use encrypted telegram sessions
f87aef7 test(security): cover telegram session migration
2fe1bf2 feat(security): encrypt telegram session files
4307f2c feat(security): add telegram session key id
```

The encrypted Telegram session storage feature branch was merged fast-forward into `main` and deleted. The only other local branch observed earlier was `desktop-ui`.

## Completed Secure Storage Work

The earlier secure storage implementation is complete and merged:

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

Verified earlier:

```powershell
cargo test secret_store::
cargo test llm::
cargo test accounts::
cargo test telegram::
cargo test migrations::
npm.cmd test
npm.cmd run check
git diff --check
cargo run
```

## Completed Telegram Session Encryption

Telegram session JSON security tail is implemented and merged.

Core files:

- `src-tauri/src/telegram_session_store.rs`
- `src-tauri/src/telegram.rs`
- `src-tauri/src/accounts.rs`
- `src-tauri/src/secret_store.rs`
- `src-tauri/Cargo.toml`

Behavior:

- Session files remain in app data as `telegram_<account_id>.session.json`.
- File contents are encrypted JSON envelope:

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

- Crypto dependencies:

```toml
chacha20poly1305 = { version = "0.10", features = ["std"] }
rand_core = { version = "0.6", features = ["getrandom"] }
```

- Legacy plaintext session JSON migrates lazily on load after successful parse and successful keyring write.
- If encrypted file exists but key is missing, load fails closed instead of falling back to `MemorySession::default()`.
- Wrong account id fails decryption through associated data.
- Account logout clears session file and session key.
- Account deletion clears runtime/session artifacts and then deletes the Telegram `api_hash` secret, surfacing cleanup errors after row/runtime cleanup.

Post-merge verification on `main`:

```powershell
cargo test
npm.cmd test
npm.cmd run check
cargo fmt --check
git diff --check
```

Observed results:

- `cargo test`: `184 passed`
- `npm.cmd test`: `196 passed`
- `npm.cmd run check`: `0 errors`, `0 warnings`
- formatting and diff checks passed.

## Telegram Runtime Live Validation

Validation checklist:

- `docs/superpowers/plans/2026-05-08-telegram-runtime-live-validation.md`

Recorded commits:

- `e867b1b docs(validation): plan telegram runtime live checks`
- `6922606 docs(validation): record telegram runtime live results`
- `1b797bf docs(validation): record telegram re-login session check`

Observed live results:

- `telegram_1.session.json` was observed as encrypted envelope with:
  - `version: 1`
  - `algorithm: XChaCha20-Poly1305`
  - `nonce`
  - `ciphertext`
  - no plaintext `home_dc`, `dc_options`, or `updates_state`
- UI reached:
  - `Account ready`
  - `This account is ready to sync sources.`
- Private supergroup source `WBChat` was visible:
  - category `Life`
  - kind `supergroup`
  - `73102 msgs`
  - membership `member`
- Sync on private supergroup `WBChat` succeeded without re-login:
  - timestamp changed from `08.05.2026, 22:16:20` to `08.05.2026, 22:17:18`
- User confirmed `WBChat` is private.
- User performed logout and re-login.
- After re-login, `telegram_1.session.json` was again observed as encrypted envelope with no plaintext session fields.
- No `restore_failed` state or auth error was observed during this validation slice.
- User decided not to manually validate account delete cleanup. This remains covered by automated tests but skipped manually by decision.
- Separate private `channel` case remains optional if a real case appears.

During validation, the app later showed `Код ошибки: Out of Memory` on `/analysis`. Investigation showed:

- `extractum.exe` was alive with modest memory usage;
- Tauri backend responded through MCP;
- WebView DOM/log/screenshot calls timed out;
- Windows Application log and Crashpad reports did not show a fresh extractum/WebView crash;
- likely WebView renderer page-kill/hang on `/analysis`, not Rust backend OOM.

This OOM observation helped motivate the next LLM analysis preflight work.

## Current LLM Concurrency / Analysis Preflight Track

User selected variant 2 first, then refined it to variant 3:

- scheduler policy refinement plus protection from large analysis runs;
- hard caps plus preflight summary.

Approved default limits:

- `max_messages_per_run = 10_000`
- `max_chunks_per_run = 80`
- `max_estimated_input_chars_per_run = 1_500_000`
- `max_background_requests_per_run = 80`

Design spec:

- `docs/superpowers/specs/2026-05-08-llm-concurrency-policy-design.md`
- Commit: `be8bd24 docs(llm): design analysis preflight limits`

Implementation plan:

- `docs/superpowers/plans/2026-05-08-llm-concurrency-policy.md`
- Commit: `96c8b57 docs(llm): plan analysis preflight limits`

Review feedback was received from another LLM and accepted with modifications:

- Commit: `08e8c79 docs(llm): address preflight policy review`
- Design now explicitly records that the first preflight implementation still scans and decompresses eligible messages to estimate input chars.
- This is a known limitation: it avoids creating a run row and spawning chunk workers for oversized work, but it does not avoid all corpus-read cost.
- Future optimization can add item text length metadata and use `COUNT`/`SUM` without decompression.
- Design now explicitly states that database/decompression errors during preflight fail before `analysis_runs` insertion.
- Plan removes the dead `exceeds_background` check from `preflight_limit_error`; `max_background_requests_per_run` remains documented and reserved for future retry-aware budgeting.
- Plan adds a shared `live_corpus_ref(source_id, item_id)` helper to keep `load_corpus_messages` and preflight ref accounting aligned.
- Plan adds `preflight_ref_format_matches_corpus_loader_ref_format`.

Current approved design summary:

- Existing scheduler policy remains:
  - `2` running LLM requests per `(provider, profile)`;
  - interactive requests jump ahead of background requests inside the same scheduler key;
  - requests with different provider/profile keys may run independently;
  - cancellation remains request-scoped or run-scoped.
- Analysis report runs get backend preflight before duplicate-run handling and run insertion.
- Preflight reports:
  - selected source ids;
  - eligible text message count;
  - estimated input chars;
  - estimated chunks;
  - configured limits.
- Oversized scopes return `AppError::validation` before inserting an `analysis_runs` row.
- Passing preflight emits early progress summary:

```text
Preflight passed: 2430 documents, 18 estimated chunks, 310000 estimated input characters.
```

## Current Implementation Plan Summary

Plan file:

- `docs/superpowers/plans/2026-05-08-llm-concurrency-policy.md`

Tasks:

1. Add pure preflight estimation helpers in `src-tauri/src/analysis/corpus.rs`.
   - Add `AnalysisRunPreflightLimits`.
   - Add `AnalysisRunPreflight`.
   - Add `estimate_message_input_chars`.
   - Add `live_corpus_ref`.
   - Add `estimate_preflight_chunk_count`.
   - Use TDD RED/GREEN.
   - Commit message in plan: `feat(analysis): add report preflight policy types`.

2. Add database-backed preflight in `src-tauri/src/analysis/corpus.rs`.
   - Add tests:
     - `preflight_counts_eligible_text_messages_for_sources`
     - `preflight_ref_format_matches_corpus_loader_ref_format`
     - `preflight_ignores_media_only_items_without_text_content`
   - Implement `preflight_analysis_run`.
   - Update `load_corpus_messages` to use `live_corpus_ref`.
   - Commit message in plan: `feat(analysis): preflight report corpus size`.

3. Enforce hard caps before run insertion.
   - Add `preflight_limit_error`.
   - Wire preflight into `start_analysis_report`.
   - Call preflight after resolving source ids and before `find_active_duplicate_run`/`insert_analysis_run`.
   - Add `preflight` to `ReportRunInput`.
   - Emit preflight summary in `run_report_pipeline`.
   - Commit message in plan: `feat(analysis): enforce report preflight limits`.

4. Add integration-level report start coverage via pure validation helper.
   - Add `validate_report_preflight`.
   - Replace inline checks in `start_analysis_report` with `validate_report_preflight(&preflight)?`.
   - Add tests:
     - empty corpus rejected;
     - oversized run rejected;
     - within-limits run allowed.
   - Commit message in plan: `test(analysis): cover report preflight validation`.

5. Update docs.
   - Modify:
     - `docs/backlog.md`
     - `docs/project.md`
     - `docs/design-document.md`
     - `docs/architecture-deep-dive.md`
   - State scheduler concurrency is intentional and analysis runs are capped by backend preflight.
   - Commit message in plan: `docs(llm): document report preflight limits`.

6. Final verification.
   - Run:

```powershell
cargo test analysis::corpus::
cargo test analysis::report::
cargo test llm::scheduler::
cargo test
npm.cmd test
npm.cmd run check
cargo fmt --check
git diff --check
git status --short
git log --oneline -8
```

## Important Code Notes For Next Session

Relevant files:

- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/mod.rs`
- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/llm/scheduler.rs`

Current code facts:

- `ANALYSIS_CHUNK_TARGET_CHARS` is defined in `src-tauri/src/analysis/mod.rs` as `16_000`.
- `report.rs` already imports it from `super`.
- Current `load_corpus_messages` uses:

```rust
r#ref: format!("s{}-i{}", row.source_id, row.id)
```

- The plan says to replace that with:

```rust
r#ref: live_corpus_ref(row.source_id, row.id)
```

- `start_analysis_report` currently:
  - validates date/language/scope/template;
  - resolves LLM profile and effective model;
  - resolves source ids;
  - checks duplicate active run;
  - inserts an `analysis_runs` row;
  - inserts active run id;
  - spawns `run_report_pipeline`.
- Preflight must be inserted after source ids are known and before duplicate-run handling / run insertion.
- Current `run_report_pipeline` still loads full corpus and checks empty corpus after load. Keep that empty-corpus guard as defense in depth even after preflight rejects empty scopes earlier.
- Current `run_map_phase` spawns one task per chunk. Scheduler limits concurrent execution, but task creation can still be large without preflight.
- Current scheduler constant:

```rust
const DEFAULT_CONCURRENCY_LIMIT: usize = 2;
```

## Superpowers Process State

Skills used in this flow:

- `superpowers:using-superpowers`
- `superpowers:brainstorming`
- `superpowers:writing-plans`
- `superpowers:executing-plans`
- `superpowers:test-driven-development`
- `superpowers:verification-before-completion`
- `superpowers:systematic-debugging`
- `superpowers:finishing-a-development-branch`
- `superpowers:receiving-code-review`

For next implementation:

- Use `superpowers:executing-plans` or `superpowers:subagent-driven-development` to execute `docs/superpowers/plans/2026-05-08-llm-concurrency-policy.md`.
- Use `superpowers:test-driven-development` because this is behavior change.
- The plan is already approved and self-reviewed after external feedback.

## User Decisions

User approved:

- finish Telegram session JSON security tail;
- encrypted app-data envelope with per-account OS-keyring key;
- merge that feature into `main`;
- skip manual account delete validation;
- move next to LLM concurrency policy refinement;
- choose combined scheduler policy plus large analysis run protection;
- use hard caps plus preflight summary;
- use conservative default limits:
  - `10_000` messages;
  - `80` chunks;
  - `1_500_000` estimated input chars;
  - `80` background requests;
- accept revised spec/plan after external LLM review.

Latest user request before this context file was written:

- overwrite `docs/session-context-2026-05-03.md` with all information needed to restore the current session context;
- form a commit message.

Suggested commit message:

```text
docs(session): capture llm preflight planning context
```
