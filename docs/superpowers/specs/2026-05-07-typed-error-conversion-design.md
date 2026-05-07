# Boundary-First Typed Error Conversion Design

## Summary

Tighten backend typed error conversion for the remaining DB, Telegram, LLM, and
validation paths without rewriting the whole error system.

The chosen scope is boundary-first: command/service boundaries and directly
adjacent helpers should return `AppResult<T>` with explicit `AppError`
constructors. Internal parser, compression, streaming, and provider-event paths
may keep `Result<T, String>` when the string is intentionally user-facing event
text or is not yet part of a command boundary.

## Goals

- Preserve the current frontend-facing error wire shape: `{ kind, message }`.
- Preserve the existing `AppErrorKind` set:
  `validation`, `not_found`, `auth`, `network`, `conflict`, `internal`.
- Reduce reliance on `src-tauri/src/error.rs` substring classification for
  targeted DB, Telegram, LLM, and validation command paths.
- Keep LLM streamed event payloads as plain text `error` fields.
- Keep the work split into one top-level task and one commit per user turn.

## Non-Goals

- Do not remove every internal `Result<T, String>` from the backend.
- Do not introduce new error kinds or a large error framework.
- Do not change frontend `src/lib/app-error.ts` behavior.
- Do not refactor already-tightened source, Takeout import, or NotebookLM export
  paths except for shared helper imports if compilation requires it.
- Do not clean up completed report-actions plan/spec files in this workstream.

## Error Mapping

Add these explicit helper constructors to `src-tauri/src/error.rs`:

```rust
pub fn database(error: impl std::fmt::Display) -> Self {
    Self::internal(format!("Database error: {error}"))
}

pub fn telegram_network(error: impl std::fmt::Display) -> Self {
    Self::network(format!("Telegram request failed: {error}"))
}

pub fn llm_network(error: impl std::fmt::Display) -> Self {
    Self::network(format!("LLM request failed: {error}"))
}
```

Use existing constructors for semantic errors:

- invalid user/config/request input: `AppError::validation(...)`;
- missing persisted entities: `AppError::not_found(...)`;
- unauthenticated or not-initialized Telegram runtime: `AppError::auth(...)`;
- duplicate/active/cannot-edit/cannot-delete states: `AppError::conflict(...)`;
- local filesystem, serialization, poisoned lock, and unexpected internal
  failures: `AppError::internal(...)`.

Keep `From<String>` and `classify_message` as compatibility fallbacks for
legacy paths that this workstream does not touch.

## Targeted Boundaries

### Error Foundation

`src-tauri/src/error.rs` owns explicit helper constructors and unit tests. This
is the only shared API addition.

### Accounts DB

`src-tauri/src/accounts.rs` should map SQL failures through
`AppError::database(...)`. Account commands already return `AppResult<T>`, so
this is a narrow conversion from raw string mapping to explicit typed mapping.

### Analysis Validation and Store Boundaries

In `src-tauri/src/analysis/templates.rs`, `groups.rs`, and command-facing
helpers in `mod.rs`/`chat.rs`, convert validators that feed commands from
`Result<T, String>` to `AppResult<T>` and use `AppError::validation(...)`.

SQL calls in the same command paths should use `AppError::database(...)`.
Existing `not_found` and `conflict` decisions remain unchanged.

### Telegram

`src-tauri/src/telegram.rs` should use explicit mappings at the command/runtime
boundary:

- missing account runtime or required login token: `auth`;
- missing persisted account credentials: `not_found`;
- Telegram client calls such as authorization checks, sending codes, sign-in,
  and sign-out: `telegram_network`;
- session path, JSON serialization, session file writes, and account credential
  SQL calls: `internal` or `database` as appropriate.

Best-effort cleanup paths may continue swallowing errors where they already do
so intentionally.

### LLM

`src-tauri/src/llm/mod.rs`, `profiles.rs`, and `runner.rs` should use explicit
typed errors at command boundaries:

- unsupported provider, invalid base URL, invalid profile id, empty default
  model, empty request id, empty messages, and empty model override:
  `validation`;
- missing saved profile in `set_active_profile_in_pool`: `not_found`;
- model-listing timeouts and provider HTTP/client failures at command boundary:
  `llm_network`;
- profile storage SQL failures: `database`.

`run_llm_collect_with_profile`, `run_llm_stream_with_profile`, scheduler
failures, and streamed LLM event payloads may continue returning string errors
because their consumer emits plain text progress/failure events.

## Verification

Focused verification after each implementation task:

```powershell
cd src-tauri
cargo test error
cargo test accounts
cargo test analysis
cargo test telegram
cargo test llm
```

Full verification before final handoff:

```powershell
cd src-tauri
cargo test
cd ..
npm.cmd test
npm.cmd run check
git diff --check
```

Success means targeted command paths no longer depend on substring
classification for DB, Telegram, LLM, or validation failures, while frontend
structured errors keep the same shape and all checks pass.
