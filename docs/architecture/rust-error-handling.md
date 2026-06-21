# Rust Backend Error Handling

This document defines the error handling methodology for the Rust/Tauri backend.
New backend functionality should follow this model. Existing code should be
migrated gradually when it is touched for related work.

## Goals

- Keep a stable error contract for the frontend.
- Avoid classifying errors by parsing human-readable strings.
- Make expected application states explicit in DTOs instead of treating them as exceptions.
- Keep low-level implementation details useful for diagnostics without leaking secrets to UI.
- Allow domain-specific Rust errors inside subsystems while preserving a single Tauri boundary type.

## External Contract

Tauri commands should return:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

`AppError` is the serialized UI-facing error shape:

```rust
pub struct AppError {
    pub kind: AppErrorKind,
    pub message: String,
}
```

This contract should remain stable unless there is an explicit frontend migration plan.
The frontend should be able to rely on `kind` for coarse behavior and `message` for display.

The serialized frontend contract uses snake_case error kinds:

```json
{
  "kind": "validation",
  "message": "run_id cannot be empty"
}
```

Allowed serialized `kind` values:

- `validation`
- `not_found`
- `auth`
- `network`
- `conflict`
- `internal`

## Error Kinds

Use `AppErrorKind` consistently:

- `Validation`: invalid user input, invalid request shape, unsupported option, malformed user-provided value.
- `NotFound`: requested app entity, run, source, file, or record does not exist.
- `Auth`: missing credentials, expired session, missing API key, insufficient authorization.
- `Network`: network, HTTP, DNS, timeout, sidecar transport, remote provider, or retryable external service failure.
- `Conflict`: valid request conflicts with current state, duplicate entity, active run, locked resource, uniqueness violation.
- `Internal`: bug, broken invariant, corrupted app-controlled data, unexpected protocol shape, serialization failure for trusted internal data.

When in doubt, prefer the kind that best describes the action the user or app can take.
If the user can correct the input, use `Validation`. If the environment/service can be retried,
use `Network`. If the app state is impossible or corrupted, use `Internal`.

## Expected States Are Not Exceptions

Expected workflow states should be modeled as DTO/status values, not `AppError`.

Examples:

- A Browser Provider run that needs login should return a provider/run status with a manual action.
- A queued or running operation should return a queue/run state.
- A cancelled background run should be represented by the run status when cancellation is part of the workflow.

Use `AppError` when the command itself cannot be completed according to its contract.

Timeout and cancellation boundaries:

- user-cancelled workflow: DTO/status, not `AppError`;
- provider HTTP/API timeout: `AppError::network(...)`;
- sidecar transport timeout: `AppError::network(...)` when retryable, `AppError::internal(...)` when it indicates protocol breakage;
- internal polling timeout that is an expected workflow branch: DTO/status or manual action;
- internal timeout that violates an invariant: `AppError::internal(...)`.

## New Code Rules

New Rust backend code should follow these rules:

1. Tauri commands return `AppResult<T>`.
2. Validate inputs near the boundary and return `AppError::validation(...)`.
3. Use `AppError::not_found(...)` for explicit missing records or app-controlled artifacts.
4. Use `AppError::conflict(...)` for duplicate/active/locked state conflicts.
5. Use `AppError::network(...)` for external communication failures and retryable provider failures.
6. Use `AppError::internal(...)` for broken invariants, unexpected protocol data, or corrupted internal artifacts.
7. Do not add new `Err("...".into())` or rely on `String -> AppError` conversion.
8. Do not add new `map_err(|error| error.to_string())?` in functions returning `AppResult`.
9. Do not decide error kind by matching text from an error message in new code.
10. Do not put secrets, cookies, API keys, tokens, private prompts, or full account identifiers into `AppError.message`.
11. Treat `AppError.message` as a safe display string. If it contains external input or a lower-level error from a provider, OS, network, browser, or user-selected file, sanitize it before returning it.

Preferred style:

```rust
if run_id.trim().is_empty() {
    return Err(AppError::validation("run_id cannot be empty"));
}

let run = load_run(pool, run_id)
    .await?
    .ok_or_else(|| AppError::not_found(format!("Run {run_id} was not found")))?;
```

Avoid:

```rust
return Err("run not found".into());
```

Avoid:

```rust
let payload = serde_json::to_vec(value).map_err(|error| error.to_string())?;
```

Use:

```rust
let payload = serde_json::to_vec(value)
    .map_err(|error| AppError::internal(format!("serialize run payload: {error}")))?;
```

## Domain Errors

Subsystems with non-trivial behavior may define local typed errors. These errors should be
used internally and converted explicitly to `AppError` at the module or command boundary.

Use a local error enum when:

- multiple lower-level operations can fail in different meaningful ways;
- the subsystem has recovery or branching behavior based on error type;
- errors are passed through several internal layers;
- tests need to assert exact error categories;
- the subsystem produces both workflow statuses and fatal command failures.

Example:

```rust
#[derive(Debug, thiserror::Error)]
pub enum BrowserRunLogError {
    #[error("Browser run {0} was not found")]
    RunNotFound(String),

    #[error("Invalid browser run id")]
    InvalidRunId,

    #[error("Failed to read browser run log: {0}")]
    ReadFailed(std::io::Error),

    #[error("Browser run log JSON is invalid: {0}")]
    InvalidJson(serde_json::Error),
}

impl From<BrowserRunLogError> for AppError {
    fn from(error: BrowserRunLogError) -> Self {
        match error {
            BrowserRunLogError::RunNotFound(run_id) => {
                AppError::not_found(format!("Browser run {run_id} was not found"))
            }
            BrowserRunLogError::InvalidRunId => {
                AppError::validation("Invalid browser run id")
            }
            BrowserRunLogError::ReadFailed(error) => {
                AppError::internal(format!("Failed to read browser run log: {error}"))
            }
            BrowserRunLogError::InvalidJson(error) => {
                AppError::internal(format!("Browser run log is corrupted: {error}"))
            }
        }
    }
}
```

`thiserror` is appropriate for domain errors because it keeps internal Rust errors typed
without changing the frontend contract. Do not expose domain error enums directly from
Tauri commands.

Domain error enums may store sensitive details internally when that is useful for logs or
tests, but `From<DomainError> for AppError` must produce a safe UI message. Do not assume a
domain error's `Display` output is safe to expose to the frontend.

Safe mapping:

```rust
match error {
    ProviderError::Rejected { account_id, .. } => {
        tracing::warn!(account_id, "provider rejected request");
        AppError::auth("Provider authentication is required")
    }
}
```

Avoid exposing the raw detail:

```rust
AppError::auth(error.to_string())
```

The same rule applies to `tracing` fields. Raw emails, tokens, prompts, cookies, provider
responses, and local paths must not be logged directly because diagnostics exports may include
structured logs. Log a non-sensitive id, a stable hash, or a redacted value instead.

## Sanitization Standard

`AppError.message` must be safe to display and safe to store in normal UI-visible logs.
When an error message includes external input or output from an external system, sanitize it
through a shared helper before constructing `AppError`.

Use the existing diagnostics redaction helper:

```rust
crate::diagnostics::redaction::sanitized_error_message(message)
```

This is the canonical project helper and it bounds output by
`crate::diagnostics::redaction::MAX_SANITIZED_TEXT_CHARS`. When the rules below require
additional coverage, extend this helper and its tests instead of adding a second sanitizer with
overlapping behavior. If other modules need easier access, move or re-export the existing helper
from a shared module while preserving one implementation and one test corpus.

Feature-specific helpers may add context, but should reuse the shared sanitizer rather than
creating unrelated redaction rules. All UI-visible error messages derived from external input
should be single-line or line-collapsed and bounded to the shared maximum.

Redaction rules:

- URL query strings and fragments: remove by default. Preserve only scheme, host, and path
  when the path itself is not sensitive.
- URL credentials: always redact.
- Query parameters named like `token`, `key`, `api_key`, `auth`, `code`, `state`, `session`,
  `password`, or `secret`: always redact if a query is intentionally preserved.
- Cookies, bearer tokens, API keys, session ids, refresh tokens, and authorization headers:
  replace with `<redacted-secret>`.
- Email addresses, account hints, and full account identifiers: replace with
  `<redacted-account>` unless the UI explicitly needs a non-sensitive account label.
- Local absolute paths: replace with `<redacted-path>` or show only a safe basename.
- User prompts, transcript text, provider request bodies, provider response bodies, raw HTML,
  and page text: do not include in `AppError.message`; use a short operation label instead.
- External command arguments: include only known-safe flags; redact paths, cookies, URLs with
  query strings, and credentials.
- Long text, multiline text, and Unicode text: collapse or bound through the shared sanitizer;
  do not return unbounded provider/browser output even after secret redaction.

Examples:

```rust
let safe_detail = sanitized_error_message(&error.to_string());
AppError::network(format!(
    "Provider request failed: {}",
    safe_detail
))
```

```rust
AppError::internal("Failed to parse Browser Provider artifact")
```

Avoid:

```rust
AppError::network(format!("Provider request failed: {raw_response_body}"))
```

## Source-Specific Mapping

### SQL/database

Default database failures should map to `AppError::database(...)`, currently an `Internal`
error with database context. The target behavior for this helper is a safe, bounded UI message.
Raw database details should go to sanitized diagnostics or structured logs, not directly to
frontend-visible errors.

Map database errors explicitly only when they represent known product behavior:

- missing row after a user request: `NotFound`;
- known uniqueness or active-state violation: `Conflict`;
- malformed persisted data: `Internal`.

Do not classify SQL errors by matching human-readable database messages such as
`"unique constraint failed"`. For `sqlx::Error::Database`, use structured database metadata
where available:

- `code()`
- `constraint()`
- `table()`
- `is_unique_violation()`
- `is_foreign_key_violation()`
- `is_check_violation()`

If the driver does not provide enough structured metadata to identify a known product
condition, keep the error as `AppError::database(...)`.

Example:

```rust
match error {
    sqlx::Error::Database(db_error) if db_error.is_unique_violation() => {
        AppError::conflict("A record with these values already exists")
    }
    other => AppError::database(other),
}
```

### JSON and serialization

- User-provided JSON or settings: `Validation`.
- App-controlled persisted JSON/artifact corruption: `Internal`.
- External provider response JSON: usually `Network` or a provider-specific typed result,
  depending on whether the failure should be actionable as provider/runtime state.

### Filesystem

- User-selected missing path: `NotFound`.
- Invalid user-provided path or run id: `Validation`.
- App-controlled artifact missing when it should exist: usually `Internal`, unless the command
  is specifically a lookup command where `NotFound` is more useful.
- Permission or write failure in app-controlled storage: `Internal`.
- Path traversal, non-child canonical path, symlink escape, or access outside the command's
  allowed root: `Validation` when the request is malformed, or `Conflict` when state changed
  between validation and use. Return a generic safe message and do not echo the raw path.

For app-controlled paths, validate canonical containment before opening or deleting files when
the path includes a user-provided id, filename, or relative component.

### Network, providers, and sidecars

- HTTP/DNS/socket/timeout/provider transport failures: `Network`.
- Sidecar process unavailable or communication failed: usually `Network` if retryable by
  restarting the provider, `Internal` if the protocol is malformed or impossible.
- Unexpected response variant, response id mismatch, or invalid JSONL protocol: `Internal`.
- Login, consent, account selection, CAPTCHA, and other manual Browser Provider states should
  be provider/run status values, not `AppError`.

### Authentication and secrets

- Missing app credential, missing provider session, expired login, forbidden request: `Auth`.
- Missing API key: `Auth` when the key is absent, `Validation` when the configured value is
  malformed.
- Invalid credential configuration, unsupported credential type, or invalid endpoint setup:
  `Validation`.
- OS keyring or secure storage unavailable: `Internal` when the command cannot proceed because
  the platform service failed.
- Expected setup flows with dedicated UI, such as "start Chrome with CDP" or "sign in to
  provider", should use typed setup/manual-action DTOs instead of `AppError`.
- Never include secret values in error messages.

## Logging and Diagnostics

`AppError.message` is a user-facing or semi-user-facing message. It should be short and safe.

For diagnostic details:

- use run artifacts, debug summaries, or structured logs when available;
- sanitize URLs, prompts, cookies, tokens, and account hints;
- include stable operation labels such as `read browser run log`, `parse sidecar response`,
  or `open app data directory`.

Do not expand `AppError` into a general debug payload. Keep detailed diagnostics in the
feature-specific debug surface.

## Legacy Compatibility

`src-tauri/src/error.rs` currently contains string classification through `From<String>` and
`From<&str>`. Treat this as legacy compatibility only.

New code should not depend on it. Migration should be incremental:

1. Do not add new implicit string conversions to `AppError`.
2. When touching existing code, replace string conversions with explicit constructors.
3. Add tests for important mappings before changing behavior.
4. Once implicit conversions are no longer needed, remove or restrict `classify_message()`.

Useful searches during migration:

```powershell
rg 'Err\\(\".*\"\\.into\\(\\)\\)|map_err\\(\\|.*\\| .*to_string\\(\\)\\)' src-tauri/src
rg 'classify_message|impl From<String> for AppError|impl From<&str> for AppError' src-tauri/src
rg 'anyhow!|\\.context\\(|format!\\(\"\\{error\\}\"\\)|format!\\(\".*\\{error\\}' src-tauri/src
rg 'map_err\\(AppError::internal\\)|AppError::internal\\(format!' src-tauri/src
rg 'raw_response|response_body|provider body|prompt|cookie|authorization|api_key|token' src-tauri/src
```

## Testing Requirements

For new or migrated error handling:

- Unit-test domain error to `AppError` mapping for each meaningful branch.
- Test `AppErrorKind`, not only message text.
- Test that expected workflow states return DTO/status values rather than command errors.
- Test that security-sensitive messages are sanitized when the error can include external input.
- For frontend-facing commands, include at least one test or smoke path that validates the serialized shape.

The shared sanitizer test corpus should include:

- URL credentials, query strings, and fragments;
- bearer tokens, cookies, API keys, authorization headers, and session ids;
- email addresses and account hints;
- Windows paths, UNC paths, file URIs, Unix paths, and app data directories;
- long Unicode strings and multiline provider/browser bodies;
- prompt text, transcript text, raw HTML, provider request bodies, and provider response bodies.

## Initial Migration Pilot

The first recommended pilot area is Browser Provider backend code under:

```text
src-tauri/src/gemini_browser/
```

Reasons:

- it is actively changing;
- it has sidecar, CDP, filesystem, JSONL protocol, artifacts, and manual-action states;
- it benefits from separating workflow statuses from fatal command failures;
- it has visible UI impact when error kinds/messages are wrong.

Good first targets:

- sidecar protocol errors;
- run log filesystem/JSON errors;
- CDP Chrome launch/setup errors;
- open run folder security checks.

The pilot should preserve the existing Tauri command response shapes and migrate only the
internal mapping style.
