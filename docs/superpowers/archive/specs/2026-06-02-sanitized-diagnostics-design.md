# Sanitized Diagnostic Summary Contract

> Historical design record. The backend sanitized diagnostics command shipped
> before the 2026-06-03 Diagnostics UI; current behavior is summarized in root
> docs such as `docs/project.md`, `docs/design-document.md`, and
> `docs/architecture-deep-dive.md`.

> Date: 2026-06-02
> Status: approved design

## Purpose

Extractum needs a small local diagnostic surface that helps explain runtime
health without exposing local archive content, prompts, provider credentials,
cookies, Telegram session material, or raw provider payloads. The first slice
adds a backend diagnostic summary contract and tests. It does not add a
settings screen, support-bundle ZIP, structured log store, crash reporting, or
remote telemetry.

The diagnostic summary is local-only data returned by a Tauri command. Future
UI or support-bundle work can reuse the contract, but this slice should stay
focused on the safe backend surface.

## Core Safety Model

The primary safety mechanism is an allow-list DTO. `get_diagnostic_summary`
must construct a typed response from explicitly permitted aggregate fields.
It must not collect broad database rows, source records, profile records,
provider payloads, raw logs, or arbitrary JSON and then rely on redaction to
make them safe.

Redaction is defense-in-depth. Every free-form string, error snippet, status
snippet, or diagnostic JSON value that still enters the DTO must pass through
the shared redaction helper. Redaction does not make raw provider payloads,
source content, prompt text, cookies, or session material eligible for the DTO.
Those data classes are excluded before redaction is considered.

## Diagnostic DTO

Add a backend `diagnostics` module with a typed, serializable
`DiagnosticSummary` returned by a Tauri command named `get_diagnostic_summary`.
The command only reads existing app state and returns JSON-serializable data.
It does not write files, collect logs, package archives, or upload anything.

Allowed fields:

- app/build: app name, app version, debug/release mode, generated-at timestamp;
- database: SQLite availability, migration status derived from
  `_sqlx_migrations` and the current `build_migrations()` list, and explicit
  allow-listed aggregate counts needed to explain app health;
- providers: provider kinds, configured/not-configured booleans, active
  provider kind when available, model/catalog availability state when already
  represented as non-secret metadata;
- external runtime checks: `yt-dlp` availability/status and secure-storage
  availability/status where the app can determine them without exposing paths
  or secret values;
- Telegram account state: account counts and coarse runtime status
  distributions, without phone numbers, labels, API hashes, session paths, or
  dialog/source names;
- sources: counts grouped by `source_type`, `source_subtype`, and coarse sync
  state;
- jobs/runs: counts grouped by provider, kind, status, terminal state,
  `error_kind`, and warning code;
- privacy metadata: an explicit `excluded_data_classes: string[]` list.

Forbidden by default:

- source titles, group names, Telegram usernames, channel names, YouTube URLs,
  playlist URLs, video titles, channel handles, and display labels;
- user-entered LLM profile names if they are not purely technical ids;
- full provider `base_url` values;
- raw terminal error text;
- raw frontend/backend logs;
- raw provider payloads, raw `yt-dlp` output, raw Telegram RPC payloads, raw LLM
  requests or responses;
- local DB paths, session paths, key paths, cookie-file paths, or temp-file
  paths;
- source content, message bodies, transcript text, comment text, prompt text,
  report body text, chat question text, and chat answer text.

Opaque internal numeric ids may appear only when they match the project's
existing sanitized-evidence convention and materially help debugging. The first
implementation should not include itemized ids except schema or migration
version numbers.

## Data Collection Rules

The command must not perform live provider calls or expensive source refreshes.
It may only read existing local runtime state, existing database aggregate
state, and cheap local availability checks such as secure-storage availability
and `yt-dlp --version` or equivalent presence checks. It must not call model
listing, provider smoke tests, YouTube metadata extraction, Telegram dialog
refresh, Telegram sync, or LLM analysis/chat APIs.

Database aggregation must use explicit allow-listed queries. The implementation
must not enumerate every table, serialize arbitrary rows, or count every table
automatically. The first implementation should limit database aggregates to:

- account count;
- source counts grouped by `source_type`, `source_subtype`, active flag, and
  coarse sync state;
- item counts grouped by provider/source type, item kind, content kind, and
  media/content presence;
- analysis run counts grouped by run type, scope type, status, and snapshot
  state;
- LLM request/job counts grouped by provider, status, and typed error kind;
- YouTube source-job counts grouped by job type and status from existing
  in-memory state;
- Takeout/import recovery counts grouped by provider, ingest kind, status,
  completeness, and warning code;
- migration versions from `_sqlx_migrations` compared against
  `build_migrations()`.

Failure paths must be sanitized too. Command-level `AppError` messages exposed
to the frontend must not include raw provider errors, local paths, source
content, prompt text, cookies, session material, raw database paths, Telegram
RPC detail, or raw payload fragments. Map failures into typed, bounded,
sanitized messages before returning them across the Tauri command boundary.

## Redaction Helpers

Add shared backend helpers in the diagnostics module:

- `redact_text(text) -> String`
- `redact_json_value(value) -> serde_json::Value`

The helpers should cover:

- key-name driven redaction for names such as `api_key`, `apiHash`,
  `api_hash`, `cookie`, `cookies`, `session`, `session_key`, `token`,
  `bearer`, `authorization`, `password`, `secret`, `prompt`, `content`,
  `transcript`, `comment`, `message`, and `payload`;
- text-pattern redaction for bearer tokens, cookie-like assignments, obvious
  API key fragments, session-file names, PEM/key material markers, and local
  secret/session paths;
- bounded output for diagnostic snippets so large strings cannot accidentally
  become content dumps.

The helper should preserve useful non-secret categories, status words, counts,
warning codes, and typed error kinds.

## Error And Warning Policy

Diagnostic summaries should report terminal state as structured fields:

- `status`
- `error_kind`
- `error_code`
- `warning_codes`
- bounded sanitized `summary`, only when a short explanation is useful

They must not include raw terminal error text from providers, `yt-dlp`,
Telegram RPC, LLM APIs, database drivers, or frontend status strings. Such text
can contain URLs, cookies, request fragments, file paths, or user content.

## Privacy Metadata

`DiagnosticSummary` should include an explicit
`excluded_data_classes: string[]` field. The first implementation should list:

- `source_content`
- `message_bodies`
- `transcript_text`
- `comment_text`
- `prompt_text`
- `report_text`
- `chat_text`
- `api_keys`
- `telegram_api_hashes`
- `youtube_cookies`
- `telegram_sessions`
- `raw_provider_payloads`
- `local_secret_paths`
- `local_database_path`

This field is both a product promise and a test target. If future diagnostics
need a new sensitive class, the class must be added to this list and excluded
from the DTO.

## Command Boundary

Register `get_diagnostic_summary` as a backend Tauri command. It should be
safe to call from the frontend or through local debugging tools. The command
should fail with typed `AppError` values, and any error summary exposed through
the diagnostic DTO must follow the redaction and bounded-snippet rules.

No frontend UI is required in this slice. A later settings or diagnostics
screen can call the command after this backend contract is implemented and
tested.

## Testing

Tests must cover both the redaction helpers and the final serialized DTO.

Required Rust tests:

- `redact_text` removes sentinel API keys, cookies, bearer tokens, Telegram
  session filenames, local secret paths, prompt text, and message/content text;
- `redact_json_value` recursively redacts sensitive keys and nested values
  while preserving allowed status/count/warning-code fields;
- a fixture diagnostic summary seeded with sentinel API key, cookie, session,
  prompt, message, transcript, comment, provider payload, and local path values
  serializes without those sentinel strings;
- command failure fixtures map sentinel provider errors, local paths, and raw
  payload fragments into typed sanitized `AppError` messages;
- the same serialized summary still contains useful allowed data such as
  counts, provider kinds, coarse statuses, error kinds, and warning codes;
- source titles, usernames, URLs, profile display names, provider base URLs,
  raw terminal errors, raw payloads, and local DB/session paths are absent from
  the final serialized summary.

The key assertion is against the whole output:

```rust
let json = serde_json::to_string(&summary).expect("serialize summary");
assert!(!json.contains(SENTINEL_API_KEY));
```

This protects against future DTO changes that accidentally bypass a narrow
unit test.

## Non-Goals

This slice does not:

- add a visible diagnostics/settings UI;
- add support-bundle ZIP generation;
- add frontend or backend log capture;
- add crash reporting;
- add remote telemetry;
- add settings search or command-palette actions;
- expose full profile ids, source labels, source URLs, source titles, raw error
  text, raw payloads, or local file paths.

## Implementation Plan Shape

The follow-up implementation plan should be small:

1. Add `src-tauri/src/diagnostics/` with DTO types and redaction helpers.
2. Build `get_diagnostic_summary` from allow-listed aggregate queries and
   runtime checks.
3. Register the Tauri command.
4. Add Rust tests for redaction and serialized DTO safety.
5. Run `npm run verify`.
