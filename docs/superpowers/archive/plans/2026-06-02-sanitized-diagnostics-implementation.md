# Sanitized Diagnostics Implementation Plan

> Historical execution record. The backend sanitized diagnostics command shipped
> before the 2026-06-03 Diagnostics UI; current behavior is summarized in root
> docs such as `docs/project.md`, `docs/design-document.md`, and
> `docs/architecture-deep-dive.md`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a backend-only `get_diagnostic_summary` Tauri command that returns allow-listed runtime health aggregates without source content, prompts, credentials, local paths, raw payloads, or raw terminal errors.

**Architecture:** Create a focused `diagnostics` Rust module with DTOs, redaction helpers, database aggregate readers, runtime checks, and the Tauri command boundary. The command builds typed DTOs from explicit aggregate queries and small in-memory state snapshots; redaction is applied only as defense-in-depth for bounded strings that still enter the DTO or command error path.

**Tech Stack:** Rust/Tauri 2, SQLx SQLite, serde/serde_json, Tokio, existing `SecretStoreState`, existing scheduler/source-job/Telegram states, Cargo tests, canonical `npm.cmd run verify` on Windows or `npm run verify` elsewhere.

---

## Approved Spec

- `docs/superpowers/archive/specs/2026-06-02-sanitized-diagnostics-design.md`

Key constraints from the spec:

- Allow-list DTO is the primary safety boundary.
- Do not collect broad database rows, source records, profile records, provider payloads, raw logs, or arbitrary JSON and then rely on redaction.
- Redaction is defense-in-depth for free-form snippets and JSON values that still enter the DTO.
- No frontend UI, support bundle ZIP, log capture, crash reporting, remote telemetry, live provider calls, model listing, YouTube metadata extraction, Telegram dialog refresh, Telegram sync, or LLM calls.
- Database data must come from explicit allow-listed aggregate queries.
- Migration state must derive from `_sqlx_migrations` and `build_migrations()`.
- First slice should not serialize itemized ids except schema or migration version numbers.
- `last_sync_state` must not be serialized because it is an item/message position; use `last_synced_at IS NULL` to derive coarse source sync state.
- Provider `base_url`, profile display/id strings, source titles, usernames, URLs, raw terminal errors, raw provider payloads, DB/session paths, prompts, messages, comments, transcripts, and report/chat text are forbidden.
- Command-level `AppError` failure paths must be typed, bounded, and sanitized.

## File Map

Create:

- `src-tauri/src/diagnostics/mod.rs`
  - Own the `get_diagnostic_summary` Tauri command.
  - Re-export internal DTO/redaction pieces only where needed by sibling modules.
  - Map command-level failures through sanitized `AppError` values.

- `src-tauri/src/diagnostics/dto.rs`
  - Define the allow-list DTO.
  - Keep DTO fields typed and specific; avoid arbitrary maps for sensitive areas.
  - Store counts, provider kinds, coarse statuses, error kinds, warning codes, and migration versions only.

- `src-tauri/src/diagnostics/redaction.rs`
  - Provide `redact_text(text: &str) -> String`.
  - Provide `redact_json_value(value: serde_json::Value) -> serde_json::Value`.
  - Provide bounded snippet helpers for command errors and optional runtime summaries.

- `src-tauri/src/diagnostics/database.rs`
  - Load database/migration/account/source/item/analysis/ingest aggregate counts from explicit SQL queries.
  - Never enumerate all tables or serialize arbitrary rows.

- `src-tauri/src/diagnostics/runtime.rs`
  - Build safe runtime/provider/in-memory state aggregates.
  - Run cheap local checks for `yt-dlp` and secure storage availability without exposing raw stderr or secret values.
  - Keep the secure-storage check read-only; missing probe keys mean the store is reachable, not failed.

Modify:

- `src-tauri/src/lib.rs`
  - Add `mod diagnostics;`.
  - Import and register `get_diagnostic_summary` in `tauri::generate_handler!`.

- `src-tauri/src/telegram.rs`
  - Add a method that returns counts by runtime status for known account ids.
  - Do not return account ids or runtime messages.

- `src-tauri/src/youtube/jobs.rs`
  - Add a method that returns grouped diagnostic counts by job type, status, warning presence, and typed error kind.
  - Do not return job ids, source ids, raw warning strings, or raw errors.

- `src-tauri/src/llm/mod.rs`
  - Add a small diagnostics helper that loads profile state and returns provider/configured counts only.
  - Do not return profile ids, provider base URLs, API keys, or user-entered labels.

No new production dependency is required.

---

### Task 1: Redaction Helpers

**Files:**
- Create: `src-tauri/src/diagnostics/mod.rs`
- Create: `src-tauri/src/diagnostics/redaction.rs`

- [x] **Step 1: Create the diagnostics module shell**

Create `src-tauri/src/diagnostics/mod.rs`:

```rust
mod redaction;

pub(crate) use redaction::{
    redact_json_value, redact_text, sanitized_error_message, MAX_SANITIZED_TEXT_CHARS,
};
```

- [x] **Step 2: Write failing redaction tests**

Create `src-tauri/src/diagnostics/redaction.rs` with the tests first:

```rust
use serde_json::Value;

const REDACTED: &str = "[redacted]";
const MAX_SNIPPET_CHARS: usize = 240;
pub(crate) const MAX_SANITIZED_TEXT_CHARS: usize = MAX_SNIPPET_CHARS + 15;

pub(crate) fn redact_text(text: &str) -> String {
    text.to_string()
}

pub(crate) fn redact_json_value(value: Value) -> Value {
    value
}

pub(crate) fn sanitized_error_message(message: &str) -> String {
    redact_text(message)
}

#[cfg(test)]
mod tests {
    use super::{
        redact_json_value, redact_text, sanitized_error_message, MAX_SANITIZED_TEXT_CHARS,
        REDACTED,
    };
    use serde_json::json;

    const SENTINEL_API_KEY: &str = "sk-sentinel-1234567890abcdef";
    const SENTINEL_COOKIE: &str = "sessionid=sentinel-cookie-value";
    const SENTINEL_BEARER: &str = "Bearer sentinel-bearer-token";
    const SENTINEL_SESSION_FILE: &str = "telegram_42.session.json";
    const SENTINEL_LOCAL_PATH: &str = "C:\\Users\\Dima\\AppData\\Roaming\\org.ai.extractum\\extractum.db";
    const SENTINEL_PROMPT: &str = "summarize my private prompt text";
    const SENTINEL_MESSAGE: &str = "private Telegram message body";
    const SENTINEL_PAYLOAD: &str = "raw provider payload with private message";
    const SENTINEL_URL: &str = "https://youtube.example/watch?v=private";

    #[test]
    fn redact_text_removes_secret_and_content_patterns() {
        let input = format!(
            "api_key={SENTINEL_API_KEY} Cookie: {SENTINEL_COOKIE}; Authorization: {SENTINEL_BEARER}; session {SENTINEL_SESSION_FILE}; path {SENTINEL_LOCAL_PATH}; url {SENTINEL_URL}; prompt: {SENTINEL_PROMPT}; message: {SENTINEL_MESSAGE}; payload: {SENTINEL_PAYLOAD}"
        );

        let output = redact_text(&input);

        for sentinel in [
            SENTINEL_API_KEY,
            "sentinel-cookie-value",
            "sentinel-bearer-token",
            SENTINEL_SESSION_FILE,
            SENTINEL_LOCAL_PATH,
            "youtube.example",
            "watch?v=private",
            SENTINEL_PROMPT,
            SENTINEL_MESSAGE,
            SENTINEL_PAYLOAD,
        ] {
            assert!(
                !output.contains(sentinel),
                "redacted text leaked sentinel {sentinel}: {output}"
            );
        }
        assert!(output.contains(REDACTED));
    }

    #[test]
    fn redact_json_value_redacts_sensitive_keys_recursively() {
        let value = json!({
            "status": "failed",
            "count": 3,
            "warning_code": "export_dc_fallback",
            "nested": {
                "apiHash": "sentinel-api-hash",
                "payload": {
                    "message": "private nested message",
                    "safe_status": "running"
                }
            },
            "items": [
                { "comment": "private comment text" },
                { "provider": "telegram", "count": 2 }
            ],
            "comment_count": 12,
            "has_content": true
        });

        let output = redact_json_value(value);
        let json = serde_json::to_string(&output).expect("serialize redacted json");

        for sentinel in [
            "sentinel-api-hash",
            "private nested message",
            "private comment text",
        ] {
            assert!(!json.contains(sentinel), "redacted json leaked {sentinel}: {json}");
        }
        assert_eq!(output["status"], "failed");
        assert_eq!(output["count"], 3);
        assert_eq!(output["warning_code"], "export_dc_fallback");
        assert_eq!(output["items"][1]["provider"], "telegram");
        assert_eq!(output["comment_count"], 12);
        assert_eq!(output["has_content"], true);
    }

    #[test]
    fn sanitized_error_message_is_bounded() {
        let long = format!(
            "LLM request failed; api_key={SENTINEL_API_KEY}; path {SENTINEL_LOCAL_PATH}; url {SENTINEL_URL}; {}",
            "x".repeat(1000)
        );

        let output = sanitized_error_message(&long);

        assert!(!output.contains(SENTINEL_API_KEY));
        assert!(!output.contains(SENTINEL_LOCAL_PATH));
        assert!(!output.contains("youtube.example"));
        assert!(
            output.len() <= MAX_SANITIZED_TEXT_CHARS,
            "bounded output was too long: {}",
            output.len()
        );
        assert!(output.contains("[truncated]"));
    }
}
```

- [x] **Step 3: Run the redaction tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics::redaction -- --nocapture
```

Expected: failures in `redact_text_removes_secret_and_content_patterns`, `redact_json_value_redacts_sensitive_keys_recursively`, and `sanitized_error_message_is_bounded`.

- [x] **Step 4: Implement redaction helpers**

Replace the stub functions in `src-tauri/src/diagnostics/redaction.rs` with:

```rust
use serde_json::{Map, Value};

const REDACTED: &str = "[redacted]";
const MAX_SNIPPET_CHARS: usize = 240;
pub(crate) const MAX_SANITIZED_TEXT_CHARS: usize = MAX_SNIPPET_CHARS + 15;

const SENSITIVE_KEY_PARTS: &[&str] = &[
    "apikey",
    "apihash",
    "api_hash",
    "authorization",
    "bearer",
    "comment",
    "content",
    "cookie",
    "cookies",
    "message",
    "password",
    "payload",
    "prompt",
    "secret",
    "session",
    "sessionkey",
    "session_key",
    "token",
    "transcript",
];

const SAFE_KEY_EXACT: &[&str] = &[
    "contentkind",
    "content_kind",
    "commentcount",
    "comment_count",
    "hascontent",
    "has_content",
    "warningcode",
    "warning_code",
];

const SENSITIVE_SNIPPET_MARKERS: &[&str] = &[
    "comment",
    "content",
    "message",
    "payload",
    "prompt",
    "transcript",
];

pub(crate) fn redact_text(text: &str) -> String {
    let mut output = redact_sensitive_snippets(text);
    output = redact_token_after_markers(&output);
    output = redact_sensitive_key_values(&output);
    output = redact_url_tokens(&output);
    output = redact_session_filenames(&output);
    output = redact_local_paths(&output);
    bound_snippet(&output)
}

pub(crate) fn redact_json_value(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(redact_json_object(object)),
        Value::Array(values) => Value::Array(values.into_iter().map(redact_json_value).collect()),
        Value::String(text) => Value::String(redact_text(&text)),
        other => other,
    }
}

pub(crate) fn sanitized_error_message(message: &str) -> String {
    redact_text(message)
}

fn redact_json_object(object: Map<String, Value>) -> Map<String, Value> {
    object
        .into_iter()
        .map(|(key, value)| {
            if is_sensitive_key(&key) {
                (key, redact_sensitive_value(value))
            } else {
                (key, redact_json_value(value))
            }
        })
        .collect()
}

fn redact_sensitive_value(value: Value) -> Value {
    match value {
        Value::Number(_) | Value::Bool(_) | Value::Null => value,
        Value::String(_) | Value::Array(_) | Value::Object(_) => {
            Value::String(REDACTED.to_string())
        }
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if SAFE_KEY_EXACT.iter().any(|safe_key| normalized == *safe_key) {
        return false;
    }
    SENSITIVE_KEY_PARTS
        .iter()
        .any(|part| normalized.contains(part))
}

fn redact_sensitive_snippets(input: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(input.len());
    let mut index = 0usize;

    while index < chars.len() {
        if let Some((marker_len, separator_index)) = sensitive_marker_at(&chars, index) {
            for offset in 0..marker_len {
                output.push(chars[index + offset]);
            }
            index += marker_len;
            while index < separator_index {
                output.push(chars[index]);
                index += 1;
            }
            output.push(chars[index]);
            index += 1;
            if index < chars.len() && chars[index].is_whitespace() {
                output.push(chars[index]);
                index += 1;
            }
            output.push_str(REDACTED);
            while index < chars.len() && !matches!(chars[index], ';' | '\n' | '\r') {
                index += 1;
            }
            continue;
        }
        output.push(chars[index]);
        index += 1;
    }

    output
}

fn sensitive_marker_at(chars: &[char], index: usize) -> Option<(usize, usize)> {
    for marker in SENSITIVE_SNIPPET_MARKERS {
        let marker_chars = marker.chars().collect::<Vec<_>>();
        let end = index + marker_chars.len();
        if end > chars.len() {
            continue;
        }
        let matches_marker = marker_chars
            .iter()
            .enumerate()
            .all(|(offset, expected)| chars[index + offset].eq_ignore_ascii_case(expected));
        if !matches_marker {
            continue;
        }
        let marker_is_bounded_before = index == 0
            || !chars[index - 1].is_ascii_alphanumeric() && chars[index - 1] != '_';
        let marker_is_bounded_after = end >= chars.len()
            || !chars[end].is_ascii_alphanumeric() && chars[end] != '_';
        if !marker_is_bounded_before || !marker_is_bounded_after {
            continue;
        }

        let mut separator_index = end;
        while separator_index < chars.len() && chars[separator_index].is_whitespace() {
            separator_index += 1;
        }
        if separator_index < chars.len() && matches!(chars[separator_index], ':' | '=') {
            return Some((marker_chars.len(), separator_index));
        }
    }
    None
}

fn redact_token_after_markers(input: &str) -> String {
    let words = input.split_whitespace().collect::<Vec<_>>();
    let mut output = Vec::with_capacity(words.len());
    let mut redact_next_count = 0usize;

    for word in words {
        let lower = word
            .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .to_ascii_lowercase();
        if redact_next_count > 0 {
            output.push(REDACTED.to_string());
            redact_next_count -= 1;
            continue;
        }
        if lower == "authorization" {
            output.push(word.to_string());
            redact_next_count = 2;
            continue;
        }
        if lower == "bearer" {
            output.push(word.to_string());
            redact_next_count = 1;
            continue;
        }
        if matches!(
            lower.as_str(),
            "cookie" | "cookies" | "session" | "prompt" | "message"
        ) {
            output.push(word.to_string());
            redact_next_count = 1;
        } else if lower.starts_with("bearer") {
            output.push(REDACTED.to_string());
        } else {
            output.push(word.to_string());
        }
    }

    output.join(" ")
}

fn redact_url_tokens(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let lower = word.to_ascii_lowercase();
            if lower.contains("http://") || lower.contains("https://") {
                REDACTED.to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_sensitive_key_values(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let lower = word.to_ascii_lowercase();
            let has_sensitive_key = SENSITIVE_KEY_PARTS
                .iter()
                .any(|part| lower.replace('-', "_").contains(part));
            let has_assignment = lower.contains('=') || lower.contains(':');
            if has_sensitive_key && has_assignment {
                redact_assignment_word(word)
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_assignment_word(word: &str) -> String {
    if let Some((key, _)) = word.split_once('=') {
        format!("{key}={REDACTED}")
    } else if let Some((key, _)) = word.split_once(':') {
        format!("{key}:{REDACTED}")
    } else {
        REDACTED.to_string()
    }
}

fn redact_session_filenames(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let lower = word.to_ascii_lowercase();
            if lower.contains(".session") || lower.ends_with(".session.json") {
                REDACTED.to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_local_paths(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let lower = word.to_ascii_lowercase();
            let looks_like_windows_path = word.len() > 3
                && word.as_bytes().get(1) == Some(&b':')
                && (word.contains('\\') || word.contains('/'));
            let looks_like_secret_path = lower.contains("appdata")
                || lower.contains(".config")
                || lower.contains("extractum.db")
                || lower.contains("org.ai.extractum");
            if looks_like_windows_path || looks_like_secret_path {
                REDACTED.to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn bound_snippet(input: &str) -> String {
    let mut chars = input.chars();
    let prefix = chars.by_ref().take(MAX_SNIPPET_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{prefix} [truncated]")
    } else {
        prefix
    }
}
```

- [x] **Step 5: Run redaction tests and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics::redaction -- --nocapture
```

Expected: all `diagnostics::redaction` tests pass.

Commit:

```powershell
git add src-tauri/src/diagnostics/mod.rs src-tauri/src/diagnostics/redaction.rs
git commit -m "feat: add diagnostic redaction helpers"
```

---

### Task 2: Allow-List Diagnostic DTO

**Files:**
- Modify: `src-tauri/src/diagnostics/mod.rs`
- Create: `src-tauri/src/diagnostics/dto.rs`

- [x] **Step 1: Write failing DTO serialization safety test**

Add `mod dto;` and the re-export to `src-tauri/src/diagnostics/mod.rs`:

```rust
mod dto;
mod redaction;

pub(crate) use dto::*;
pub(crate) use redaction::{
    redact_json_value, redact_text, sanitized_error_message, MAX_SANITIZED_TEXT_CHARS,
};
```

Create `src-tauri/src/diagnostics/dto.rs` with the test fixture first. Do not add temporary DTO stubs in this step; the compile failure should come from the missing production DTO types that Step 3 adds.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL_SOURCE_TITLE: &str = "private source title";
    const SENTINEL_URL: &str = "https://youtube.example/watch?v=private";
    const SENTINEL_PROFILE_ID: &str = "my-private-profile";
    const SENTINEL_BASE_URL: &str = "https://llm.internal.example/v1";
    const SENTINEL_RAW_ERROR: &str = "raw provider error with prompt text";

    #[test]
    fn diagnostic_summary_fixture_serializes_without_forbidden_sentinels() {
        let summary = fixture_summary();

        let json = serde_json::to_string(&summary).expect("serialize summary");

        for sentinel in [
            SENTINEL_SOURCE_TITLE,
            SENTINEL_URL,
            SENTINEL_PROFILE_ID,
            SENTINEL_BASE_URL,
            SENTINEL_RAW_ERROR,
        ] {
            assert!(!json.contains(sentinel), "summary leaked {sentinel}: {json}");
        }
        assert!(json.contains("source_content"));
        assert!(json.contains("telegram"));
        assert!(json.contains("gemini"));
        assert!(json.contains("export_dc_fallback"));
        assert!(json.contains("sqliteAvailable"));
    }

    fn fixture_summary() -> DiagnosticSummary {
        DiagnosticSummary {
            app: DiagnosticAppInfo {
                app_name: "extractum".to_string(),
                app_version: "0.1.0".to_string(),
                build_mode: "debug".to_string(),
                generated_at_unix: 1_717_300_000,
            },
            database: DiagnosticDatabaseInfo {
                sqlite_available: true,
                migrations: DiagnosticMigrationInfo {
                    status: "current".to_string(),
                    expected_versions: vec![1, 2, 3],
                    applied_versions: vec![1, 2, 3],
                    pending_versions: Vec::new(),
                    failed_versions: Vec::new(),
                },
                account_count: 2,
            },
            providers: DiagnosticProvidersInfo {
                active_provider: Some("gemini".to_string()),
                profiles_by_provider: vec![DiagnosticProviderProfileCount {
                    provider: "gemini".to_string(),
                    configured_count: 1,
                    missing_key_count: 0,
                }],
            },
            runtimes: DiagnosticRuntimeInfo {
                ytdlp: DiagnosticRuntimeCheck {
                    status: "available".to_string(),
                    available: true,
                    version: Some("2026.01.01".to_string()),
                    summary: None,
                },
                secure_storage: DiagnosticRuntimeCheck {
                    status: "available".to_string(),
                    available: true,
                    version: None,
                    summary: None,
                },
            },
            telegram: DiagnosticTelegramInfo {
                account_count: 2,
                runtime_statuses: vec![DiagnosticStatusCount {
                    status: "ready".to_string(),
                    count: 1,
                }],
            },
            sources: DiagnosticSourcesInfo {
                counts: vec![DiagnosticSourceCount {
                    source_type: "telegram".to_string(),
                    source_subtype: Some("supergroup".to_string()),
                    active: true,
                    sync_state: "synced".to_string(),
                    count: 3,
                }],
            },
            items: DiagnosticItemsInfo {
                counts: vec![DiagnosticItemCount {
                    source_type: "youtube".to_string(),
                    source_subtype: Some("video".to_string()),
                    item_kind: "youtube_comment".to_string(),
                    content_kind: "text_only".to_string(),
                    has_content: true,
                    has_media: false,
                    media_kind: None,
                    count: 7,
                }],
            },
            analysis_runs: DiagnosticAnalysisRunsInfo {
                counts: vec![DiagnosticAnalysisRunCount {
                    provider: "gemini".to_string(),
                    run_type: "report".to_string(),
                    scope_type: "single_source".to_string(),
                    status: "failed".to_string(),
                    snapshot_state: "not_captured".to_string(),
                    error_kind: "network".to_string(),
                    count: 1,
                }],
            },
            llm_requests: DiagnosticLlmRequestsInfo {
                counts: vec![DiagnosticLlmRequestCount {
                    provider: "gemini".to_string(),
                    kind: "analysis_report_map".to_string(),
                    state: "running".to_string(),
                    count: 1,
                }],
            },
            youtube_jobs: DiagnosticYoutubeJobsInfo {
                counts: vec![DiagnosticYoutubeJobCount {
                    job_type: "youtube_video_full_sync".to_string(),
                    status: "failed".to_string(),
                    warning_state: "none".to_string(),
                    error_kind: "network".to_string(),
                    count: 1,
                }],
            },
            ingest: DiagnosticIngestInfo {
                batches: vec![DiagnosticIngestBatchCount {
                    provider: "telegram".to_string(),
                    ingest_kind: "takeout".to_string(),
                    status: "completed".to_string(),
                    completeness: "complete".to_string(),
                    error_kind: "none".to_string(),
                    count: 1,
                }],
                warnings: vec![DiagnosticIngestWarningCount {
                    provider: "telegram".to_string(),
                    ingest_kind: "takeout".to_string(),
                    status: "completed".to_string(),
                    warning_code: "export_dc_fallback".to_string(),
                    count: 2,
                }],
            },
            privacy: DiagnosticPrivacyInfo {
                excluded_data_classes: excluded_data_classes(),
            },
        }
    }
}
```

- [x] **Step 2: Run DTO test and verify it fails to compile**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics::dto -- --nocapture
```

Expected: compile failures for missing DTO types such as `DiagnosticSummary`, `DiagnosticAppInfo`, and `DiagnosticDatabaseInfo`.

- [x] **Step 3: Define DTO types and privacy list**

Above the test module in `src-tauri/src/diagnostics/dto.rs`, add:

```rust
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticSummary {
    pub app: DiagnosticAppInfo,
    pub database: DiagnosticDatabaseInfo,
    pub providers: DiagnosticProvidersInfo,
    pub runtimes: DiagnosticRuntimeInfo,
    pub telegram: DiagnosticTelegramInfo,
    pub sources: DiagnosticSourcesInfo,
    pub items: DiagnosticItemsInfo,
    pub analysis_runs: DiagnosticAnalysisRunsInfo,
    pub llm_requests: DiagnosticLlmRequestsInfo,
    pub youtube_jobs: DiagnosticYoutubeJobsInfo,
    pub ingest: DiagnosticIngestInfo,
    pub privacy: DiagnosticPrivacyInfo,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticAppInfo {
    pub app_name: String,
    pub app_version: String,
    pub build_mode: String,
    pub generated_at_unix: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticDatabaseInfo {
    pub sqlite_available: bool,
    pub migrations: DiagnosticMigrationInfo,
    pub account_count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticMigrationInfo {
    pub status: String,
    pub expected_versions: Vec<i64>,
    pub applied_versions: Vec<i64>,
    pub pending_versions: Vec<i64>,
    pub failed_versions: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticProvidersInfo {
    pub active_provider: Option<String>,
    pub profiles_by_provider: Vec<DiagnosticProviderProfileCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticProviderProfileCount {
    pub provider: String,
    pub configured_count: i64,
    pub missing_key_count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticRuntimeInfo {
    pub ytdlp: DiagnosticRuntimeCheck,
    pub secure_storage: DiagnosticRuntimeCheck,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticRuntimeCheck {
    pub status: String,
    pub available: bool,
    pub version: Option<String>,
    pub summary: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticTelegramInfo {
    pub account_count: i64,
    pub runtime_statuses: Vec<DiagnosticStatusCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticStatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticSourcesInfo {
    pub counts: Vec<DiagnosticSourceCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticSourceCount {
    pub source_type: String,
    pub source_subtype: Option<String>,
    pub active: bool,
    pub sync_state: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticItemsInfo {
    pub counts: Vec<DiagnosticItemCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticItemCount {
    pub source_type: String,
    pub source_subtype: Option<String>,
    pub item_kind: String,
    pub content_kind: String,
    pub has_content: bool,
    pub has_media: bool,
    pub media_kind: Option<String>,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticAnalysisRunsInfo {
    pub counts: Vec<DiagnosticAnalysisRunCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticAnalysisRunCount {
    pub provider: String,
    pub run_type: String,
    pub scope_type: String,
    pub status: String,
    pub snapshot_state: String,
    pub error_kind: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticLlmRequestsInfo {
    pub counts: Vec<DiagnosticLlmRequestCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticLlmRequestCount {
    pub provider: String,
    pub kind: String,
    pub state: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticYoutubeJobsInfo {
    pub counts: Vec<DiagnosticYoutubeJobCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticYoutubeJobCount {
    pub job_type: String,
    pub status: String,
    pub warning_state: String,
    pub error_kind: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticIngestInfo {
    pub batches: Vec<DiagnosticIngestBatchCount>,
    pub warnings: Vec<DiagnosticIngestWarningCount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticIngestBatchCount {
    pub provider: String,
    pub ingest_kind: String,
    pub status: String,
    pub completeness: String,
    pub error_kind: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticIngestWarningCount {
    pub provider: String,
    pub ingest_kind: String,
    pub status: String,
    pub warning_code: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticPrivacyInfo {
    pub excluded_data_classes: Vec<String>,
}

pub(crate) fn excluded_data_classes() -> Vec<String> {
    [
        "source_content",
        "message_bodies",
        "transcript_text",
        "comment_text",
        "prompt_text",
        "report_text",
        "chat_text",
        "api_keys",
        "telegram_api_hashes",
        "youtube_cookies",
        "telegram_sessions",
        "raw_provider_payloads",
        "local_secret_paths",
        "local_database_path",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}
```

- [x] **Step 4: Run DTO tests and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics::dto -- --nocapture
```

Expected: `diagnostic_summary_fixture_serializes_without_forbidden_sentinels` passes.

Commit:

```powershell
git add src-tauri/src/diagnostics/mod.rs src-tauri/src/diagnostics/dto.rs
git commit -m "feat: add diagnostic summary dto"
```

---

### Task 3: Database Aggregates

**Files:**
- Modify: `src-tauri/src/diagnostics/mod.rs`
- Create: `src-tauri/src/diagnostics/database.rs`

- [x] **Step 1: Wire database module and write failing aggregate tests**

Update `src-tauri/src/diagnostics/mod.rs`:

```rust
mod database;
mod dto;
mod redaction;

pub(crate) use database::{load_account_ids, load_database_diagnostics};
pub(crate) use dto::*;
pub(crate) use redaction::{
    redact_json_value, redact_text, sanitized_error_message, MAX_SANITIZED_TEXT_CHARS,
};
```

Create `src-tauri/src/diagnostics/database.rs` with tests and signatures:

```rust
use sqlx::{Pool, Row, Sqlite};

use crate::error::{AppError, AppResult};
use crate::migrations::build_migrations;

use super::{
    DiagnosticAnalysisRunCount, DiagnosticAnalysisRunsInfo, DiagnosticDatabaseInfo,
    DiagnosticIngestBatchCount, DiagnosticIngestInfo, DiagnosticIngestWarningCount,
    DiagnosticItemCount, DiagnosticItemsInfo, DiagnosticMigrationInfo, DiagnosticSourceCount,
    DiagnosticSourcesInfo,
};

pub(crate) async fn load_account_ids(_pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    Ok(Vec::new())
}

pub(crate) async fn load_database_diagnostics(
    _pool: &Pool<Sqlite>,
) -> AppResult<(
    DiagnosticDatabaseInfo,
    DiagnosticSourcesInfo,
    DiagnosticItemsInfo,
    DiagnosticAnalysisRunsInfo,
    DiagnosticIngestInfo,
)> {
    Err(AppError::internal("database diagnostics not loaded"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::{apply_all_migrations_for_test_pool, build_migrations};

    async fn memory_pool() -> Pool<Sqlite> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        reset_sqlx_migrations_for_test(&pool).await;
        let expected_versions = expected_migration_versions();
        seed_sqlx_migrations_for_test(&pool, &expected_versions, &[]).await;
        pool
    }

    fn expected_migration_versions() -> Vec<i64> {
        build_migrations()
            .into_iter()
            .map(|migration| migration.version)
            .collect()
    }

    async fn reset_sqlx_migrations_for_test(pool: &Pool<Sqlite>) {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .execute(pool)
        .await
        .expect("create _sqlx_migrations");
        sqlx::query("DELETE FROM _sqlx_migrations")
            .execute(pool)
            .await
            .expect("clear _sqlx_migrations");
    }

    async fn seed_sqlx_migrations_for_test(
        pool: &Pool<Sqlite>,
        applied: &[i64],
        failed: &[i64],
    ) {
        for version in applied {
            sqlx::query(
                "INSERT OR REPLACE INTO _sqlx_migrations
                 (version, description, success, checksum, execution_time)
                 VALUES (?, 'test', 1, X'00', 0)",
            )
            .bind(version)
            .execute(pool)
            .await
            .expect("insert applied migration");
        }
        for version in failed {
            sqlx::query(
                "INSERT OR REPLACE INTO _sqlx_migrations
                 (version, description, success, checksum, execution_time)
                 VALUES (?, 'test', 0, X'00', 0)",
            )
            .bind(version)
            .execute(pool)
            .await
            .expect("insert failed migration");
        }
    }

    #[tokio::test]
    async fn database_diagnostics_groups_only_allow_listed_aggregates() {
        let pool = memory_pool().await;
        seed_safe_rows(&pool).await;
        let expected_versions = expected_migration_versions();

        let (database, sources, items, analysis_runs, ingest) =
            load_database_diagnostics(&pool).await.expect("load diagnostics");
        let account_ids = load_account_ids(&pool).await.expect("load account ids");

        assert_eq!(database.sqlite_available, true);
        assert_eq!(database.account_count, 1);
        assert_eq!(database.migrations.status, "current");
        assert_eq!(database.migrations.expected_versions, expected_versions);
        assert_eq!(database.migrations.applied_versions, database.migrations.expected_versions);
        assert!(database.migrations.pending_versions.is_empty());
        assert!(database.migrations.failed_versions.is_empty());
        assert_eq!(account_ids, vec![10]);
        assert_eq!(sources.counts[0].source_type, "telegram");
        assert_eq!(sources.counts[0].source_subtype.as_deref(), Some("supergroup"));
        assert_eq!(sources.counts[0].sync_state, "synced");
        assert_eq!(items.counts[0].has_content, true);
        assert_eq!(analysis_runs.counts[0].error_kind, "network");
        assert_eq!(ingest.batches[0].error_kind, "internal");
        assert_eq!(ingest.warnings[0].warning_code, "export_dc_fallback");

        let json = serde_json::to_string(&(database, sources, items, analysis_runs, ingest))
            .expect("serialize aggregate tuple");
        for forbidden in [
            "Private Source Title",
            "private message body",
            "https://youtube.example/watch?v=private",
            "raw provider payload",
            "C:\\Users\\Dima\\AppData",
        ] {
            assert!(!json.contains(forbidden), "aggregate leaked {forbidden}: {json}");
        }
    }

    #[tokio::test]
    async fn migration_status_reports_pending_and_failed_versions() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        reset_sqlx_migrations_for_test(&pool).await;
        let expected_versions = expected_migration_versions();
        assert!(
            expected_versions.len() >= 2,
            "pending/failed migration test needs at least two migrations"
        );
        let applied_version = expected_versions[0];
        let failed_version = expected_versions[1];
        let expected_pending_versions = expected_versions
            .iter()
            .copied()
            .skip(2)
            .collect::<Vec<_>>();
        seed_sqlx_migrations_for_test(&pool, &[applied_version], &[failed_version]).await;

        let migrations = load_migration_info(&pool).await.expect("load migrations");

        assert_eq!(migrations.status, "failed");
        assert_eq!(migrations.applied_versions, vec![applied_version]);
        assert_eq!(migrations.failed_versions, vec![failed_version]);
        assert_eq!(migrations.pending_versions, expected_pending_versions);
        assert_eq!(migrations.expected_versions, expected_versions);
    }

    async fn seed_safe_rows(pool: &Pool<Sqlite>) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, phone, created_at)
             VALUES (10, 'Private Account', 1, '', '+10000000000', 1)",
        )
        .execute(pool)
        .await
        .expect("insert account");

        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
             ) VALUES (
                20, 'telegram', 'supergroup', 10, 'private-external-id',
                'Private Source Title', NULL, 123456, 1000, 1, 1, 1
             )",
        )
        .execute(pool)
        .await
        .expect("insert source");

        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media, media_kind, item_kind
             ) VALUES (
                30, 20, 'private-message-id', 'private author', 1, 2,
                X'00', X'00', 'text_only', 0, NULL, 'telegram_message'
             )",
        )
        .execute(pool)
        .await
        .expect("insert item");

        sqlx::query(
            "INSERT INTO analysis_runs (
                id, run_type, scope_type, source_id, period_from, period_to,
                output_language, prompt_template_version, provider_profile,
                provider, model, status, error, created_at
             ) VALUES (
                40, 'report', 'single_source', 20, 1, 2, 'Russian', 1,
                'my-private-profile', 'gemini', 'private-model', 'failed',
                'network timeout with raw provider payload and private message body', 3
             )",
        )
        .execute(pool)
        .await
        .expect("insert analysis run");

        let batch_id: i64 = sqlx::query_scalar(
            "INSERT INTO ingest_batches (
                source_id, provider, ingest_kind, status, completeness,
                finished_at, item_inserted_count, item_observed_count,
                terminal_error
             ) VALUES (
                20, 'telegram', 'takeout', 'failed', 'partial',
                CURRENT_TIMESTAMP, 0, 0,
                'C:\\Users\\Dima\\AppData\\raw terminal error'
             )
             RETURNING id",
        )
        .fetch_one(pool)
        .await
        .expect("insert ingest batch");

        sqlx::query(
            "INSERT INTO ingest_batch_warnings (batch_id, code, message)
             VALUES (?, 'export_dc_fallback', 'raw warning message with https://youtube.example/watch?v=private')",
        )
        .bind(batch_id)
        .execute(pool)
        .await
        .expect("insert warning");
    }
}
```

- [x] **Step 2: Run database tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics::database -- --nocapture
```

Expected: `database diagnostics not loaded` failure and missing `load_migration_info` compile failure.

- [x] **Step 3: Implement migration/account/source/item/analysis/ingest aggregate loaders**

Replace the stubs in `src-tauri/src/diagnostics/database.rs` with:

```rust
pub(crate) async fn load_account_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    sqlx::query_scalar("SELECT id FROM accounts ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
}

pub(crate) async fn load_database_diagnostics(
    pool: &Pool<Sqlite>,
) -> AppResult<(
    DiagnosticDatabaseInfo,
    DiagnosticSourcesInfo,
    DiagnosticItemsInfo,
    DiagnosticAnalysisRunsInfo,
    DiagnosticIngestInfo,
)> {
    let migrations = load_migration_info(pool).await?;
    let account_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    Ok((
        DiagnosticDatabaseInfo {
            sqlite_available: true,
            migrations,
            account_count,
        },
        DiagnosticSourcesInfo {
            counts: load_source_counts(pool).await?,
        },
        DiagnosticItemsInfo {
            counts: load_item_counts(pool).await?,
        },
        DiagnosticAnalysisRunsInfo {
            counts: load_analysis_run_counts(pool).await?,
        },
        DiagnosticIngestInfo {
            batches: load_ingest_batch_counts(pool).await?,
            warnings: load_ingest_warning_counts(pool).await?,
        },
    ))
}

async fn load_migration_info(pool: &Pool<Sqlite>) -> AppResult<DiagnosticMigrationInfo> {
    let expected_versions = build_migrations()
        .into_iter()
        .map(|migration| migration.version)
        .collect::<Vec<_>>();
    let applied_versions = sqlx::query_scalar::<_, i64>(
        "SELECT version FROM _sqlx_migrations WHERE success = 1 ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    let failed_versions = sqlx::query_scalar::<_, i64>(
        "SELECT version FROM _sqlx_migrations WHERE success = 0 ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    let pending_versions = expected_versions
        .iter()
        .copied()
        .filter(|version| !applied_versions.contains(version) && !failed_versions.contains(version))
        .collect::<Vec<_>>();
    let status = if !failed_versions.is_empty() {
        "failed"
    } else if pending_versions.is_empty() {
        "current"
    } else {
        "pending"
    };

    Ok(DiagnosticMigrationInfo {
        status: status.to_string(),
        expected_versions,
        applied_versions,
        pending_versions,
        failed_versions,
    })
}

async fn load_source_counts(pool: &Pool<Sqlite>) -> AppResult<Vec<DiagnosticSourceCount>> {
    let rows = sqlx::query(
        "SELECT
            source_type,
            source_subtype,
            COALESCE(is_active, 0) AS active,
            CASE WHEN last_synced_at IS NULL THEN 'never_synced' ELSE 'synced' END AS sync_state,
            COUNT(*) AS count
         FROM sources
         GROUP BY source_type, source_subtype, active, sync_state
         ORDER BY source_type, source_subtype, active DESC, sync_state",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticSourceCount {
                source_type: row.try_get("source_type").map_err(AppError::database)?,
                source_subtype: row.try_get("source_subtype").map_err(AppError::database)?,
                active: row.try_get::<i64, _>("active").map_err(AppError::database)? != 0,
                sync_state: row.try_get("sync_state").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

async fn load_item_counts(pool: &Pool<Sqlite>) -> AppResult<Vec<DiagnosticItemCount>> {
    let rows = sqlx::query(
        "SELECT
            s.source_type,
            s.source_subtype,
            i.item_kind,
            i.content_kind,
            CASE WHEN i.content_zstd IS NULL THEN 0 ELSE 1 END AS has_content,
            COALESCE(i.has_media, 0) AS has_media,
            i.media_kind,
            COUNT(*) AS count
         FROM items i
         JOIN sources s ON s.id = i.source_id
         GROUP BY s.source_type, s.source_subtype, i.item_kind, i.content_kind,
                  has_content, has_media, i.media_kind
         ORDER BY s.source_type, s.source_subtype, i.item_kind, i.content_kind,
                  has_content DESC, has_media DESC, i.media_kind",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticItemCount {
                source_type: row.try_get("source_type").map_err(AppError::database)?,
                source_subtype: row.try_get("source_subtype").map_err(AppError::database)?,
                item_kind: row.try_get("item_kind").map_err(AppError::database)?,
                content_kind: row.try_get("content_kind").map_err(AppError::database)?,
                has_content: row.try_get::<i64, _>("has_content").map_err(AppError::database)? != 0,
                has_media: row.try_get::<i64, _>("has_media").map_err(AppError::database)? != 0,
                media_kind: row.try_get("media_kind").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

async fn load_analysis_run_counts(pool: &Pool<Sqlite>) -> AppResult<Vec<DiagnosticAnalysisRunCount>> {
    // Raw analysis error text is read only to derive a coarse error_kind.
    // It must never be selected into, copied into, or summarized in the DTO.
    let rows = sqlx::query(
        "SELECT
            provider,
            run_type,
            scope_type,
            status,
            CASE
                WHEN snapshot_captured_at IS NOT NULL THEN 'captured'
                WHEN snapshot_error IS NOT NULL THEN 'failed'
                ELSE 'not_captured'
            END AS snapshot_state,
            CASE
                WHEN error IS NULL OR TRIM(error) = '' THEN 'none'
                WHEN LOWER(error) LIKE '%timeout%' OR LOWER(error) LIKE '%network%' THEN 'network'
                WHEN LOWER(error) LIKE '%unauthorized%' OR LOWER(error) LIKE '%forbidden%' OR LOWER(error) LIKE '%api key%' THEN 'auth'
                WHEN LOWER(error) LIKE '%invalid%' THEN 'validation'
                ELSE 'internal'
            END AS error_kind,
            COUNT(*) AS count
         FROM analysis_runs
         GROUP BY provider, run_type, scope_type, status, snapshot_state, error_kind
         ORDER BY provider, run_type, scope_type, status, snapshot_state, error_kind",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticAnalysisRunCount {
                provider: row.try_get("provider").map_err(AppError::database)?,
                run_type: row.try_get("run_type").map_err(AppError::database)?,
                scope_type: row.try_get("scope_type").map_err(AppError::database)?,
                status: row.try_get("status").map_err(AppError::database)?,
                snapshot_state: row.try_get("snapshot_state").map_err(AppError::database)?,
                error_kind: row.try_get("error_kind").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

async fn load_ingest_batch_counts(pool: &Pool<Sqlite>) -> AppResult<Vec<DiagnosticIngestBatchCount>> {
    // Raw terminal_error text is read only to derive a coarse error_kind.
    // It must never be selected into, copied into, or summarized in the DTO.
    let rows = sqlx::query(
        "SELECT
            provider,
            ingest_kind,
            status,
            completeness,
            CASE
                WHEN terminal_error IS NULL OR TRIM(terminal_error) = '' THEN 'none'
                WHEN LOWER(terminal_error) LIKE '%timeout%' OR LOWER(terminal_error) LIKE '%network%' THEN 'network'
                WHEN LOWER(terminal_error) LIKE '%unauthorized%' OR LOWER(terminal_error) LIKE '%forbidden%' OR LOWER(terminal_error) LIKE '%api key%' THEN 'auth'
                WHEN LOWER(terminal_error) LIKE '%invalid%' THEN 'validation'
                ELSE 'internal'
            END AS error_kind,
            COUNT(*) AS count
         FROM ingest_batches
         GROUP BY provider, ingest_kind, status, completeness, error_kind
         ORDER BY provider, ingest_kind, status, completeness, error_kind",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticIngestBatchCount {
                provider: row.try_get("provider").map_err(AppError::database)?,
                ingest_kind: row.try_get("ingest_kind").map_err(AppError::database)?,
                status: row.try_get("status").map_err(AppError::database)?,
                completeness: row.try_get("completeness").map_err(AppError::database)?,
                error_kind: row.try_get("error_kind").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

async fn load_ingest_warning_counts(pool: &Pool<Sqlite>) -> AppResult<Vec<DiagnosticIngestWarningCount>> {
    let rows = sqlx::query(
        "SELECT
            b.provider,
            b.ingest_kind,
            b.status,
            w.code AS warning_code,
            COUNT(*) AS count
         FROM ingest_batch_warnings w
         JOIN ingest_batches b ON b.id = w.batch_id
         GROUP BY b.provider, b.ingest_kind, b.status, w.code
         ORDER BY b.provider, b.ingest_kind, b.status, w.code",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticIngestWarningCount {
                provider: row.try_get("provider").map_err(AppError::database)?,
                ingest_kind: row.try_get("ingest_kind").map_err(AppError::database)?,
                status: row.try_get("status").map_err(AppError::database)?,
                warning_code: row.try_get("warning_code").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}
```

- [x] **Step 4: Run database aggregate tests and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics::database -- --nocapture
```

Expected: all `diagnostics::database` tests pass.

Commit:

```powershell
git add src-tauri/src/diagnostics/mod.rs src-tauri/src/diagnostics/database.rs
git commit -m "feat: add diagnostic database aggregates"
```

---

### Task 4: Runtime And Provider Aggregates

**Files:**
- Modify: `src-tauri/src/telegram.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/diagnostics/mod.rs`
- Create: `src-tauri/src/diagnostics/runtime.rs`

- [x] **Step 1: Add failing tests for Telegram runtime status counts**

In `src-tauri/src/telegram.rs`, add this test inside the existing `#[cfg(test)] mod tests` or create one if the module test area has none:

```rust
#[tokio::test]
async fn diagnostic_status_counts_do_not_return_account_ids_or_messages() {
    let state = TelegramState::new();
    set_account_status_for_test(&state, 10, "ready", Some("private phone +10000000000")).await;
    set_account_status_for_test(&state, 11, "restore_failed", Some("C:\\Users\\Dima\\session")).await;

    let counts = state.diagnostic_status_counts(&[10, 11, 12]).await;

    assert_eq!(counts.len(), 3);
    assert!(counts.contains(&("not_initialized".to_string(), 1)));
    assert!(counts.contains(&("ready".to_string(), 1)));
    assert!(counts.contains(&("restore_failed".to_string(), 1)));
    let value = serde_json::to_value(&counts).expect("serialize counts value");
    let entries = value.as_array().expect("counts serialize as array");
    for entry in entries {
        let pair = entry.as_array().expect("status count entry is a tuple");
        assert_eq!(pair.len(), 2);
        assert!(pair[0].as_str().is_some());
        assert!(pair[1].as_i64().is_some());
    }
    let json = serde_json::to_string(&counts).expect("serialize counts");
    assert!(!json.contains("account_id"));
    assert!(!json.contains("\"10\""));
    assert!(!json.contains("\"11\""));
    assert!(!json.contains("\"12\""));
    assert!(!json.contains("+10000000000"));
    assert!(!json.contains("C:\\Users\\Dima"));
}

async fn set_account_status_for_test(
    state: &TelegramState,
    account_id: i64,
    status: &str,
    message: Option<&str>,
) {
    state.statuses.lock().await.insert(
        account_id,
        AccountRuntimeStatus {
            account_id,
            status: status.to_string(),
            message: message.map(ToString::to_string),
        },
    );
}
```

- [x] **Step 2: Implement Telegram runtime status counts**

Add this method to `impl TelegramState` in `src-tauri/src/telegram.rs`:

```rust
    pub(crate) async fn diagnostic_status_counts(
        &self,
        account_ids: &[i64],
    ) -> Vec<(String, i64)> {
        let statuses = self.statuses.lock().await;
        let mut counts = std::collections::BTreeMap::<String, i64>::new();
        for account_id in account_ids {
            let status = statuses
                .get(account_id)
                .map(|status| status.status.clone())
                .unwrap_or_else(|| STATUS_NOT_INITIALIZED.to_string());
            *counts.entry(status).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    }
```

- [x] **Step 3: Add failing tests for YouTube source-job diagnostic counts**

In `src-tauri/src/youtube/jobs.rs`, add:

```rust
#[tokio::test]
async fn diagnostic_counts_group_source_jobs_without_ids_or_raw_errors() {
    let state = SourceJobState::new();
    let job = state
        .create_job(
            10,
            SourceJobType::YoutubeVideoFullSync,
            None,
            YoutubeSyncOptions {
                metadata: true,
                transcripts: true,
                comments: true,
            },
        )
        .await
        .expect("create job");
    state
        .finish_job(&job.job_id, |record| {
            record.status = SourceJobStatus::Failed;
            record.error = Some("timeout with https://youtube.example/watch?v=private".to_string());
            record.warnings = vec!["raw warning with private title".to_string()];
        })
        .await
        .expect("finish job");

    let counts = state.diagnostic_counts().await;

    assert_eq!(counts.len(), 1);
    assert_eq!(counts[0].job_type, "youtube_video_full_sync");
    assert_eq!(counts[0].status, "failed");
    assert_eq!(counts[0].warning_state, "present");
    assert_eq!(counts[0].error_kind, "network");
    assert_eq!(counts[0].count, 1);
    let json = serde_json::to_string(&counts).expect("serialize counts");
    assert!(!json.contains("source-job-"));
    assert!(!json.contains("source_id"));
    assert!(!json.contains("related_source_id"));
    assert!(!json.contains("youtube.example"));
    assert!(!json.contains("private title"));
}
```

- [x] **Step 4: Implement YouTube source-job diagnostic counts**

In `src-tauri/src/youtube/jobs.rs`, add a serializable count type near `SourceJobRecord`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceJobDiagnosticCount {
    pub(crate) job_type: String,
    pub(crate) status: String,
    pub(crate) warning_state: String,
    pub(crate) error_kind: String,
    pub(crate) count: i64,
}
```

Add explicit diagnostic key helpers and this method to `impl SourceJobState`:

```rust
fn source_job_type_diagnostic_key(job_type: &SourceJobType) -> &'static str {
    match job_type {
        SourceJobType::YoutubeVideoMetadataSync => "youtube_video_metadata_sync",
        SourceJobType::YoutubeVideoTranscriptSync => "youtube_video_transcript_sync",
        SourceJobType::YoutubeVideoCommentsSync => "youtube_video_comments_sync",
        SourceJobType::YoutubeVideoFullSync => "youtube_video_full_sync",
        SourceJobType::YoutubePlaylistMetadataSync => "youtube_playlist_metadata_sync",
        SourceJobType::YoutubePlaylistFullSync => "youtube_playlist_full_sync",
        SourceJobType::YoutubePlaylistVideoSync => "youtube_playlist_video_sync",
    }
}

fn source_job_status_diagnostic_key(status: &SourceJobStatus) -> &'static str {
    match status {
        SourceJobStatus::Queued => "queued",
        SourceJobStatus::Running => "running",
        SourceJobStatus::Succeeded => "succeeded",
        SourceJobStatus::Failed => "failed",
        SourceJobStatus::CancelRequested => "cancel_requested",
        SourceJobStatus::Cancelled => "cancelled",
    }
}

fn classify_diagnostic_error_kind(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if lower.trim().is_empty() {
        "none"
    } else if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("dns")
        || lower.contains("socket")
        || lower.contains("transport")
    {
        "network"
    } else if lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("api key")
        || lower.contains("not authenticated")
    {
        "auth"
    } else if lower.contains("invalid")
        || lower.contains("unsupported")
        || lower.contains("required")
        || lower.contains("cannot be empty")
    {
        "validation"
    } else {
        "internal"
    }
}

    pub(crate) async fn diagnostic_counts(&self) -> Vec<SourceJobDiagnosticCount> {
        let inner = self.inner.lock().await;
        let mut counts = std::collections::BTreeMap::<(String, String, String, String), i64>::new();
        for job in inner.jobs.values() {
            let key = (
                source_job_type_diagnostic_key(&job.job_type).to_string(),
                source_job_status_diagnostic_key(&job.status).to_string(),
                if job.warnings.is_empty() {
                    "none".to_string()
                } else {
                    "present".to_string()
                },
                job.error
                    .as_deref()
                    .map(classify_diagnostic_error_kind)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "none".to_string()),
            );
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .map(
                |((job_type, status, warning_state, error_kind), count)| {
                    SourceJobDiagnosticCount {
                        job_type,
                        status,
                        warning_state,
                        error_kind,
                        count,
                    }
                },
            )
            .collect()
    }
```

- [x] **Step 5: Add LLM diagnostic key and provider helpers**

In `src-tauri/src/llm/mod.rs`, add `LlmRequestSnapshotState` to the existing `pub(crate) use scheduler::{ ... }` list:

```rust
pub(crate) use scheduler::{
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmRequestSnapshot,
    LlmRequestSnapshotState, LlmSchedulerState,
};
```

Then add these helpers and types near `get_llm_profiles`:

```rust
pub(crate) fn llm_request_kind_diagnostic_key(kind: LlmRequestKind) -> &'static str {
    match kind {
        LlmRequestKind::ProviderTest => "provider_test",
        LlmRequestKind::AnalysisChat => "analysis_chat",
        LlmRequestKind::AnalysisReportMap => "analysis_report_map",
        LlmRequestKind::AnalysisReportReduce => "analysis_report_reduce",
    }
}

pub(crate) fn llm_request_state_diagnostic_key(state: LlmRequestSnapshotState) -> &'static str {
    match state {
        LlmRequestSnapshotState::Queued => "queued",
        LlmRequestSnapshotState::Running => "running",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LlmProviderDiagnosticCount {
    pub(crate) provider: String,
    pub(crate) configured_count: i64,
    pub(crate) missing_key_count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LlmProviderDiagnosticState {
    pub(crate) active_provider: Option<String>,
    pub(crate) profiles_by_provider: Vec<LlmProviderDiagnosticCount>,
}

pub(crate) async fn load_provider_diagnostics_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secret_store: &SecretStoreState,
) -> AppResult<LlmProviderDiagnosticState> {
    let state = load_profiles_state_from_pool(pool, secret_store).await?;
    let active_provider = state
        .profiles
        .iter()
        .find(|profile| profile.profile_id == state.active_profile)
        .map(|profile| profile.provider.clone());
    let mut counts = std::collections::BTreeMap::<String, (i64, i64)>::new();
    for profile in state.profiles {
        let entry = counts.entry(profile.provider).or_insert((0, 0));
        if profile.api_key_configured {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }
    Ok(LlmProviderDiagnosticState {
        active_provider,
        profiles_by_provider: counts
            .into_iter()
            .map(|(provider, (configured_count, missing_key_count))| {
                LlmProviderDiagnosticCount {
                    provider,
                    configured_count,
                    missing_key_count,
                }
            })
            .collect(),
    })
}
```

Add this test in the `#[cfg(test)] mod tests` in `src-tauri/src/llm/mod.rs`:

```rust
#[test]
fn llm_request_diagnostic_keys_are_stable_snake_case() {
    assert_eq!(
        llm_request_kind_diagnostic_key(LlmRequestKind::AnalysisChat),
        "analysis_chat"
    );
    assert_eq!(
        llm_request_kind_diagnostic_key(LlmRequestKind::AnalysisReportReduce),
        "analysis_report_reduce"
    );
    assert_eq!(
        llm_request_state_diagnostic_key(LlmRequestSnapshotState::Queued),
        "queued"
    );
    assert_eq!(
        llm_request_state_diagnostic_key(LlmRequestSnapshotState::Running),
        "running"
    );
}

#[tokio::test]
async fn provider_diagnostics_exclude_profile_ids_and_base_urls() {
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::secret_store::tests::InMemorySecretStore;
    use crate::secret_store::SecretStoreState;
    use std::sync::Arc;

    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");
    let store = Arc::new(InMemorySecretStore::new());
    let secret_store = SecretStoreState::new(store);

    save_profile_to_pool(
        &pool,
        &secret_store,
        "private-profile",
        "gemini",
        "private-model",
        Some("private-api-key"),
        "",
        true,
    )
    .await
    .expect("save profile");

    let diagnostics = load_provider_diagnostics_from_pool(&pool, &secret_store)
        .await
        .expect("load provider diagnostics");
    let json = serde_json::to_string(&diagnostics.profiles_by_provider)
        .expect("serialize provider diagnostics");

    assert_eq!(diagnostics.active_provider.as_deref(), Some("gemini"));
    assert!(json.contains("gemini"));
    assert!(!json.contains("private-profile"));
    assert!(!json.contains("private-model"));
    assert!(!json.contains("private-api-key"));
    assert!(!json.contains("base_url"));
}
```

- [x] **Step 6: Create runtime diagnostics module**

Update `src-tauri/src/diagnostics/mod.rs`:

```rust
mod database;
mod dto;
mod redaction;
mod runtime;

pub(crate) use database::{load_account_ids, load_database_diagnostics};
pub(crate) use dto::*;
pub(crate) use redaction::{
    redact_json_value, redact_text, sanitized_error_message, MAX_SANITIZED_TEXT_CHARS,
};
pub(crate) use runtime::{
    check_secure_storage, check_ytdlp_runtime, load_in_memory_runtime_diagnostics,
    load_provider_diagnostics, load_runtime_checks,
};
```

Create `src-tauri/src/diagnostics/runtime.rs`:

```rust
use std::collections::BTreeMap;
use std::time::Duration;

use tokio::process::Command;

use crate::error::AppResult;
use crate::llm::{
    llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key,
    load_provider_diagnostics_from_pool, LlmSchedulerState,
};
use crate::secret_store::SecretStoreState;
use crate::telegram::TelegramState;
use crate::youtube::jobs::SourceJobState;

use super::{
    DiagnosticLlmRequestCount, DiagnosticLlmRequestsInfo, DiagnosticProviderProfileCount,
    DiagnosticProvidersInfo, DiagnosticRuntimeCheck, DiagnosticRuntimeInfo, DiagnosticStatusCount,
    DiagnosticTelegramInfo, DiagnosticYoutubeJobCount, DiagnosticYoutubeJobsInfo,
};

const YTDLP_DIAGNOSTIC_TIMEOUT: Duration = Duration::from_secs(5);
const SECURE_STORAGE_READ_PROBE_KEY: &str = "__extractum_diagnostic_probe__";

pub(crate) async fn load_provider_diagnostics(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secret_store: &SecretStoreState,
) -> AppResult<DiagnosticProvidersInfo> {
    let state = load_provider_diagnostics_from_pool(pool, secret_store).await?;
    Ok(DiagnosticProvidersInfo {
        active_provider: state.active_provider,
        profiles_by_provider: state
            .profiles_by_provider
            .into_iter()
            .map(|count| DiagnosticProviderProfileCount {
                provider: count.provider,
                configured_count: count.configured_count,
                missing_key_count: count.missing_key_count,
            })
            .collect(),
    })
}

pub(crate) async fn load_in_memory_runtime_diagnostics(
    telegram_state: &TelegramState,
    source_job_state: &SourceJobState,
    llm_scheduler: &LlmSchedulerState,
    account_ids: &[i64],
    account_count: i64,
) -> (DiagnosticTelegramInfo, DiagnosticLlmRequestsInfo, DiagnosticYoutubeJobsInfo) {
    let runtime_statuses = telegram_state
        .diagnostic_status_counts(account_ids)
        .await
        .into_iter()
        .map(|(status, count)| DiagnosticStatusCount { status, count })
        .collect();

    let llm_requests = group_llm_request_snapshots(llm_scheduler.request_snapshots().await);

    let youtube_jobs = DiagnosticYoutubeJobsInfo {
        counts: source_job_state
            .diagnostic_counts()
            .await
            .into_iter()
            .map(|count| DiagnosticYoutubeJobCount {
                job_type: count.job_type,
                status: count.status,
                warning_state: count.warning_state,
                error_kind: count.error_kind,
                count: count.count,
            })
            .collect(),
    };

    (
        DiagnosticTelegramInfo {
            account_count,
            runtime_statuses,
        },
        llm_requests,
        youtube_jobs,
    )
}

fn group_llm_request_snapshots(
    snapshots: Vec<crate::llm::LlmRequestSnapshot>,
) -> DiagnosticLlmRequestsInfo {
    let mut counts = BTreeMap::<(String, String, String), i64>::new();
    for snapshot in snapshots {
        let kind = llm_request_kind_diagnostic_key(snapshot.kind).to_string();
        let state = llm_request_state_diagnostic_key(snapshot.state).to_string();
        *counts
            .entry((snapshot.provider, kind, state))
            .or_insert(0) += 1;
    }
    DiagnosticLlmRequestsInfo {
        counts: counts
            .into_iter()
            .map(|((provider, kind, state), count)| DiagnosticLlmRequestCount {
                provider,
                kind,
                state,
                count,
            })
            .collect(),
    }
}

pub(crate) async fn check_ytdlp_runtime() -> DiagnosticRuntimeCheck {
    // This intentionally does not cache because the first diagnostics slice is
    // called on demand. If a future UI polls this command, add caching above the
    // command boundary instead of spawning yt-dlp repeatedly.
    match tokio::time::timeout(
        YTDLP_DIAGNOSTIC_TIMEOUT,
        Command::new("yt-dlp").arg("--version").output(),
    )
    .await
    {
        Ok(Ok(output)) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DiagnosticRuntimeCheck {
                status: "available".to_string(),
                available: true,
                version: if version.is_empty() { None } else { Some(version) },
                summary: None,
            }
        }
        Ok(Ok(_)) => failed_runtime_check("yt-dlp check failed"),
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => DiagnosticRuntimeCheck {
            status: "not_found".to_string(),
            available: false,
            version: None,
            summary: Some("yt-dlp is not available on PATH".to_string()),
        },
        // Do not include std::io::Error text here. It can contain local binary
        // paths on Unix-like systems and adds little diagnostic value.
        Ok(Err(_)) => failed_runtime_check("yt-dlp check failed"),
        Err(_) => DiagnosticRuntimeCheck {
            status: "timed_out".to_string(),
            available: false,
            version: None,
            summary: Some("yt-dlp runtime check timed out".to_string()),
        },
    }
}

pub(crate) async fn check_secure_storage(secret_store: &SecretStoreState) -> DiagnosticRuntimeCheck {
    // Read-only availability probe: Ok(None) means the store responded and the
    // diagnostic key simply does not exist. Do not write a probe key from the
    // diagnostics command; the command must remain read-only.
    match secret_store
        .get_secret(SECURE_STORAGE_READ_PROBE_KEY)
        .await
    {
        Ok(Some(_)) | Ok(None) => DiagnosticRuntimeCheck {
            status: "available".to_string(),
            available: true,
            version: None,
            summary: None,
        },
        // Do not include OS/keychain error text here; it can contain local
        // paths or platform account details.
        Err(_) => failed_runtime_check("Secure storage check failed"),
    }
}

fn failed_runtime_check(summary: &str) -> DiagnosticRuntimeCheck {
    DiagnosticRuntimeCheck {
        status: "check_failed".to_string(),
        available: false,
        version: None,
        summary: Some(summary.to_string()),
    }
}

pub(crate) async fn load_runtime_checks(
    secret_store: &SecretStoreState,
) -> DiagnosticRuntimeInfo {
    DiagnosticRuntimeInfo {
        ytdlp: check_ytdlp_runtime().await,
        secure_storage: check_secure_storage(secret_store).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_store::tests::InMemorySecretStore;
    use std::sync::Arc;

    #[test]
    fn failed_runtime_check_uses_coarse_summary_without_os_error_text() {
        let check = failed_runtime_check("yt-dlp check failed");

        let json = serde_json::to_string(&check).expect("serialize runtime check");

        assert_eq!(check.status, "check_failed");
        assert_eq!(check.summary.as_deref(), Some("yt-dlp check failed"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/usr/local/bin"));
        assert!(!json.contains("os error"));
    }

    #[tokio::test]
    async fn secure_storage_failure_does_not_expose_store_error_text() {
        let store = Arc::new(InMemorySecretStore::new());
        store.fail_get("keychain failed for /home/user/.local/share/org.ai.extractum/session");
        let secret_store = SecretStoreState::new(store);

        let check = check_secure_storage(&secret_store).await;
        let json = serde_json::to_string(&check).expect("serialize runtime check");

        assert_eq!(check.status, "check_failed");
        assert_eq!(
            check.summary.as_deref(),
            Some("Secure storage check failed")
        );
        assert!(!json.contains("/home/user"));
        assert!(!json.contains("org.ai.extractum/session"));
        assert!(!json.contains("keychain failed"));
    }
}
```

- [x] **Step 7: Run runtime/provider tests and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostic_counts -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml llm_request_diagnostic_keys_are_stable_snake_case -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml provider_diagnostics_exclude_profile_ids_and_base_urls -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml failed_runtime_check_uses_coarse_summary_without_os_error_text -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml secure_storage_failure_does_not_expose_store_error_text -- --nocapture
```

Expected: all added tests pass.

Commit:

```powershell
git add src-tauri/src/telegram.rs src-tauri/src/youtube/jobs.rs src-tauri/src/llm/mod.rs src-tauri/src/diagnostics/mod.rs src-tauri/src/diagnostics/runtime.rs
git commit -m "feat: add diagnostic runtime aggregates"
```

---

### Task 5: Tauri Command Boundary

**Files:**
- Modify: `src-tauri/src/diagnostics/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Write failing command assembly and sanitized failure tests**

In `src-tauri/src/diagnostics/mod.rs`, replace the shell content with module imports plus these stubs and tests:

```rust
mod database;
mod dto;
mod redaction;
mod runtime;

use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::LlmSchedulerState;
use crate::secret_store::SecretStoreState;
use crate::telegram::TelegramState;
use crate::time::now_secs;
use crate::youtube::jobs::SourceJobState;

pub(crate) use database::{load_account_ids, load_database_diagnostics};
pub(crate) use dto::*;
pub(crate) use redaction::{
    redact_json_value, redact_text, sanitized_error_message, MAX_SANITIZED_TEXT_CHARS,
};
pub(crate) use runtime::{
    check_secure_storage, check_ytdlp_runtime, load_in_memory_runtime_diagnostics,
    load_provider_diagnostics, load_runtime_checks,
};

#[tauri::command]
pub(crate) async fn get_diagnostic_summary(
    handle: AppHandle,
    telegram_state: tauri::State<'_, TelegramState>,
    source_job_state: tauri::State<'_, SourceJobState>,
    llm_scheduler: tauri::State<'_, LlmSchedulerState>,
    secret_store: tauri::State<'_, SecretStoreState>,
) -> AppResult<DiagnosticSummary> {
    build_diagnostic_summary(
        &handle,
        telegram_state.inner(),
        source_job_state.inner(),
        llm_scheduler.inner(),
        secret_store.inner(),
    )
    .await
    .map_err(sanitize_diagnostic_error)
}

async fn build_diagnostic_summary(
    _handle: &AppHandle,
    _telegram_state: &TelegramState,
    _source_job_state: &SourceJobState,
    _llm_scheduler: &LlmSchedulerState,
    _secret_store: &SecretStoreState,
) -> AppResult<DiagnosticSummary> {
    Err(AppError::internal("diagnostic summary not assembled"))
}

pub(crate) fn sanitize_diagnostic_error(error: AppError) -> AppError {
    AppError::new(
        error.kind,
        format!("Diagnostic summary failed: {}", sanitized_error_message(&error.message)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppErrorKind;

    const SENTINEL_API_KEY: &str = "sk-sentinel-command-error";
    const SENTINEL_PATH: &str = "C:\\Users\\Dima\\AppData\\Roaming\\org.ai.extractum\\extractum.db";
    const SENTINEL_PAYLOAD: &str = "raw provider payload with private message";
    const COMMAND_ERROR_PREFIX: &str = "Diagnostic summary failed: ";

    #[test]
    fn sanitize_diagnostic_error_bounds_and_redacts_command_errors() {
        let unicode_tail =
            "\u{043F}\u{0440}\u{0438}\u{0432}\u{0430}\u{0442}\u{043D}\u{044B}\u{0439} \u{0444}\u{0440}\u{0430}\u{0433}\u{043C}\u{0435}\u{043D}\u{0442} ".repeat(100);
        let error = AppError::internal(format!(
            "Database error at {SENTINEL_PATH}; api_key={SENTINEL_API_KEY}; payload: {SENTINEL_PAYLOAD}; unicode context: {unicode_tail}"
        ));

        let sanitized = sanitize_diagnostic_error(error);

        assert_eq!(sanitized.kind, AppErrorKind::Internal);
        assert!(sanitized.message.starts_with(COMMAND_ERROR_PREFIX));
        assert!(!sanitized.message.contains(SENTINEL_API_KEY));
        assert!(!sanitized.message.contains(SENTINEL_PATH));
        assert!(!sanitized.message.contains(SENTINEL_PAYLOAD));
        assert!(
            sanitized.message.chars().count()
                <= COMMAND_ERROR_PREFIX.chars().count() + MAX_SANITIZED_TEXT_CHARS,
            "bounded command error was too long: {}",
            sanitized.message.chars().count()
        );
    }
}
```

- [x] **Step 2: Run command tests and verify the assembly stub fails only where expected**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics::tests::sanitize_diagnostic_error_bounds_and_redacts_command_errors -- --nocapture
```

Expected: the sanitized error test passes after Task 1 redaction is in place.

- [x] **Step 3: Implement summary assembly**

Replace `build_diagnostic_summary` in `src-tauri/src/diagnostics/mod.rs` with:

```rust
async fn build_diagnostic_summary(
    handle: &AppHandle,
    telegram_state: &TelegramState,
    source_job_state: &SourceJobState,
    llm_scheduler: &LlmSchedulerState,
    secret_store: &SecretStoreState,
) -> AppResult<DiagnosticSummary> {
    let pool = get_pool(handle).await?;
    let (
        database,
        sources,
        items,
        analysis_runs,
        ingest,
    ) = load_database_diagnostics(&pool).await?;
    let account_ids = load_account_ids(&pool).await?;
    let providers = load_provider_diagnostics(&pool, secret_store).await?;
    let runtimes = load_runtime_checks(secret_store).await;
    let (telegram, llm_requests, youtube_jobs) = load_in_memory_runtime_diagnostics(
        telegram_state,
        source_job_state,
        llm_scheduler,
        &account_ids,
        database.account_count,
    )
    .await;

    Ok(DiagnosticSummary {
        app: DiagnosticAppInfo {
            app_name: env!("CARGO_PKG_NAME").to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            build_mode: if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "release".to_string()
            },
            generated_at_unix: now_secs(),
        },
        database,
        providers,
        runtimes,
        telegram,
        sources,
        items,
        analysis_runs,
        llm_requests,
        youtube_jobs,
        ingest,
        privacy: DiagnosticPrivacyInfo {
            excluded_data_classes: excluded_data_classes(),
        },
    })
}
```

- [x] **Step 4: Register the Tauri command**

In `src-tauri/src/lib.rs`, add the module near the other backend modules:

```rust
mod diagnostics;
use diagnostics::get_diagnostic_summary;
```

Add `get_diagnostic_summary` to the `tauri::generate_handler![...]` list near `ping_db`:

```rust
            ping_db,
            get_diagnostic_summary,
```

- [x] **Step 5: Run compile/test checks for diagnostics command**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics -- --nocapture
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: diagnostics tests pass and `cargo check` passes.

Commit:

```powershell
git add src-tauri/src/diagnostics/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add diagnostic summary command"
```

---

### Task 6: Whole Summary Sentinel Test

**Files:**
- Modify: `src-tauri/src/diagnostics/mod.rs`

- [x] **Step 1: Add whole serialized summary sentinel test**

In `src-tauri/src/diagnostics/mod.rs`, extend `#[cfg(test)] mod tests` with a helper that builds a `DiagnosticSummary` from DTO pieces and asserts on the final serialized JSON:

```rust
#[test]
fn serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data() {
    let summary = DiagnosticSummary {
        app: DiagnosticAppInfo {
            app_name: "extractum".to_string(),
            app_version: "0.1.0".to_string(),
            build_mode: "debug".to_string(),
            generated_at_unix: 1,
        },
        database: DiagnosticDatabaseInfo {
            sqlite_available: true,
            migrations: DiagnosticMigrationInfo {
                status: "current".to_string(),
                expected_versions: vec![1, 2, 3],
                applied_versions: vec![1, 2, 3],
                pending_versions: Vec::new(),
                failed_versions: Vec::new(),
            },
            account_count: 1,
        },
        providers: DiagnosticProvidersInfo {
            active_provider: Some("gemini".to_string()),
            profiles_by_provider: vec![DiagnosticProviderProfileCount {
                provider: "gemini".to_string(),
                configured_count: 1,
                missing_key_count: 0,
            }],
        },
        runtimes: DiagnosticRuntimeInfo {
            ytdlp: DiagnosticRuntimeCheck {
                status: "check_failed".to_string(),
                available: false,
                version: None,
                summary: Some(sanitized_error_message(
                    "yt-dlp failed for https://youtube.example/watch?v=private",
                )),
            },
            secure_storage: DiagnosticRuntimeCheck {
                status: "available".to_string(),
                available: true,
                version: None,
                summary: None,
            },
        },
        telegram: DiagnosticTelegramInfo {
            account_count: 1,
            runtime_statuses: vec![DiagnosticStatusCount {
                status: "ready".to_string(),
                count: 1,
            }],
        },
        sources: DiagnosticSourcesInfo {
            counts: vec![DiagnosticSourceCount {
                source_type: "telegram".to_string(),
                source_subtype: Some("channel".to_string()),
                active: true,
                sync_state: "synced".to_string(),
                count: 1,
            }],
        },
        items: DiagnosticItemsInfo {
            counts: vec![DiagnosticItemCount {
                source_type: "telegram".to_string(),
                source_subtype: Some("channel".to_string()),
                item_kind: "telegram_message".to_string(),
                content_kind: "text_only".to_string(),
                has_content: true,
                has_media: false,
                media_kind: None,
                count: 12,
            }],
        },
        analysis_runs: DiagnosticAnalysisRunsInfo {
            counts: vec![DiagnosticAnalysisRunCount {
                provider: "gemini".to_string(),
                run_type: "report".to_string(),
                scope_type: "single_source".to_string(),
                status: "failed".to_string(),
                snapshot_state: "failed".to_string(),
                error_kind: "network".to_string(),
                count: 1,
            }],
        },
        llm_requests: DiagnosticLlmRequestsInfo {
            counts: vec![DiagnosticLlmRequestCount {
                provider: "gemini".to_string(),
                kind: "analysis_chat".to_string(),
                state: "queued".to_string(),
                count: 1,
            }],
        },
        youtube_jobs: DiagnosticYoutubeJobsInfo {
            counts: vec![DiagnosticYoutubeJobCount {
                job_type: "youtube_video_comments_sync".to_string(),
                status: "failed".to_string(),
                warning_state: "present".to_string(),
                error_kind: "network".to_string(),
                count: 1,
            }],
        },
        ingest: DiagnosticIngestInfo {
            batches: vec![DiagnosticIngestBatchCount {
                provider: "telegram".to_string(),
                ingest_kind: "takeout".to_string(),
                status: "completed".to_string(),
                completeness: "complete".to_string(),
                error_kind: "none".to_string(),
                count: 1,
            }],
            warnings: vec![DiagnosticIngestWarningCount {
                provider: "telegram".to_string(),
                ingest_kind: "takeout".to_string(),
                status: "completed".to_string(),
                warning_code: "export_dc_fallback".to_string(),
                count: 1,
            }],
        },
        privacy: DiagnosticPrivacyInfo {
            excluded_data_classes: excluded_data_classes(),
        },
    };

    let json = serde_json::to_string(&summary).expect("serialize summary");

    for allowed in [
        "gemini",
        "telegram",
        "channel",
        "synced",
        "network",
        "export_dc_fallback",
        "source_content",
        "message_bodies",
        "local_database_path",
    ] {
        assert!(json.contains(allowed), "missing allowed value {allowed}: {json}");
    }

    for forbidden in [
        "youtube.example",
        "private",
        "api_key",
        "apiHash",
        "baseUrl",
        "profileId",
        "source title",
        "raw provider payload",
        "extractum.db",
        "telegram_42.session.json",
    ] {
        assert!(!json.contains(forbidden), "summary leaked {forbidden}: {json}");
    }
}
```

- [x] **Step 2: Run whole summary tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data -- --nocapture
```

Expected: the whole serialized summary test passes.

- [x] **Step 3: Run loader outward-field safety scan**

Run:

```powershell
rg -n 'pub(\(crate\))?\s+.*\b(source_id|profile_id|base_url|title|url|error|message)\b' src-tauri/src/diagnostics src-tauri/src/youtube/jobs.rs src-tauri/src/llm/mod.rs
```

Expected: no matches in diagnostic DTOs or diagnostic count structs. Matches inside tests, redaction helpers, existing non-diagnostic app DTOs, or private implementation variables are not blockers; any public diagnostic output field named `source_id`, `profile_id`, `base_url`, `title`, `url`, `error`, or `message` must be removed or replaced with an allow-listed aggregate/status field.

- [x] **Step 4: Run all diagnostics tests and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diagnostics -- --nocapture
```

Expected: all diagnostics tests pass.

Commit:

```powershell
git add src-tauri/src/diagnostics/mod.rs
git commit -m "test: cover serialized diagnostic summary safety"
```

---

### Task 7: Full Verification

**Files:**
- No code changes unless verification exposes a compile, lint, or test failure.

- [x] **Step 1: Run the canonical project gate**

Run on Windows PowerShell:

```powershell
npm.cmd run verify
```

Run on non-Windows shells:

```bash
npm run verify
```

Expected:

- Vitest passes.
- `svelte-check` reports 0 errors and 0 warnings.
- `cargo check --manifest-path src-tauri/Cargo.toml` passes.
- `cargo test --manifest-path src-tauri/Cargo.toml` passes.
- `git diff HEAD --check` passes.

- [x] **Step 2: If verification fails, use systematic debugging** — Not needed; verification passed.

Use `superpowers:systematic-debugging` before changing code. Capture the exact failing command and first relevant error, then make the smallest fix that preserves the allow-list DTO and no-raw-data constraints.

- [x] **Step 3: Commit verification-only fixes** — Not needed; no verification fixes were required.

If fixes were needed:

```powershell
git add src-tauri/src/diagnostics src-tauri/src/lib.rs src-tauri/src/telegram.rs src-tauri/src/youtube/jobs.rs src-tauri/src/llm/mod.rs
git commit -m "fix: stabilize diagnostic summary verification"
```

If no fixes were needed, do not create an empty commit.

---

## Self-Review

Spec coverage:

- Allow-list DTO: Task 2 defines typed DTOs; Task 5 assembles only those fields.
- Redaction helpers: Task 1 implements `redact_text` and `redact_json_value`.
- `_sqlx_migrations` plus `build_migrations()`: Task 3 implements `load_migration_info`.
- Explicit database aggregates: Task 3 uses only named SQL queries for accounts, sources, items, analysis runs, ingest batches, and ingest warnings.
- No broad table enumeration or arbitrary row serialization: Task 3 uses explicit aggregate queries and does not enumerate tables in production code.
- No source/profile labels, URLs, raw errors, or payloads: Tasks 2, 3, 4, and 6 include sentinel serialization tests.
- Loader outward fields: Task 6 scans public diagnostic output structs for forbidden field names such as `source_id`, `profile_id`, `base_url`, `title`, `url`, `error`, and `message`.
- Stable diagnostic keys: Task 4 uses explicit helper functions for YouTube job and LLM request enum values instead of depending on serde output.
- Raw error handling: Task 3 comments make `analysis_runs.error` and `ingest_batches.terminal_error` classification-only inputs, not DTO fields or summaries.
- Runtime check caveats: Task 4 comments keep secure-storage probing read-only and mark `yt-dlp --version` as uncached on-demand diagnostics work.
- Runtime failure privacy: Task 4 uses coarse failure summaries for `yt-dlp` and secure storage checks instead of serializing OS/keychain error text that may contain local paths.
- No live provider calls or expensive source refreshes: Task 4 only uses local `yt-dlp --version`, secure-storage probe, and in-memory state snapshots.
- Command-level sanitized errors: Task 5 adds `sanitize_diagnostic_error`.
- Privacy metadata: Task 2 includes the required `excluded_data_classes`.
- Full verification: Task 7 runs `npm.cmd run verify` on Windows or `npm run verify` elsewhere.

Placeholder scan:

- The plan contains no placeholder implementation steps.
- Any intentional future work is named as out of scope in the approved spec, not as work inside this plan.

Type consistency:

- DTO names used by database/runtime/command tasks match `dto.rs`.
- Runtime helper names used by the command match `runtime.rs`.
- Database helper names used by the command match `database.rs`.
