use serde_json::{Map, Value};

const REDACTED: &str = "[redacted]";
const MAX_SNIPPET_CHARS: usize = 240;
pub(crate) const MAX_SANITIZED_TEXT_CHARS: usize = MAX_SNIPPET_CHARS + 15;

const SENSITIVE_KEY_PARTS: &[&str] = &[
    "apikey",
    "api_key",
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn redact_sensitive_value(value: Value) -> Value {
    match value {
        Value::Number(_) | Value::Bool(_) | Value::Null => value,
        Value::String(_) | Value::Array(_) | Value::Object(_) => Value::String(REDACTED.to_string()),
    }
}

#[allow(dead_code)]
fn is_sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if SAFE_KEY_EXACT
        .iter()
        .any(|safe_key| normalized == *safe_key)
    {
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
        let marker_is_bounded_before =
            index == 0 || !chars[index - 1].is_ascii_alphanumeric() && chars[index - 1] != '_';
        let marker_is_bounded_after =
            end >= chars.len() || !chars[end].is_ascii_alphanumeric() && chars[end] != '_';
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
            let normalized = lower.replace('-', "_");
            let has_sensitive_key = SENSITIVE_KEY_PARTS
                .iter()
                .any(|part| normalized.contains(part));
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
            let looks_like_unix_path =
                !lower.contains("://") && (word.starts_with('/') || word.starts_with("~/"));
            let looks_like_secret_path = lower.contains("appdata")
                || lower.contains(".config")
                || lower.contains(".local")
                || lower.contains("extractum.db")
                || lower.contains("org.ai.extractum");
            if looks_like_windows_path || looks_like_unix_path || looks_like_secret_path {
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
    const SENTINEL_LOCAL_PATH: &str =
        "C:\\Users\\Dima\\AppData\\Roaming\\org.ai.extractum\\extractum.db";
    const SENTINEL_UNIX_PATH: &str = "/home/dima/.local/bin/yt-dlp";
    const SENTINEL_PROMPT: &str = "summarize my private prompt text";
    const SENTINEL_MESSAGE: &str = "private Telegram message body";
    const SENTINEL_PAYLOAD: &str = "raw provider payload with private message";
    const SENTINEL_URL: &str = "https://youtube.example/watch?v=private";

    #[test]
    fn redact_text_removes_secret_and_content_patterns() {
        let input = format!(
            "api_key={SENTINEL_API_KEY} Cookie: {SENTINEL_COOKIE}; Authorization: {SENTINEL_BEARER}; session {SENTINEL_SESSION_FILE}; path {SENTINEL_LOCAL_PATH}; binary {SENTINEL_UNIX_PATH}; url {SENTINEL_URL}; prompt: {SENTINEL_PROMPT}; message: {SENTINEL_MESSAGE}; payload: {SENTINEL_PAYLOAD}"
        );

        let output = redact_text(&input);

        for sentinel in [
            SENTINEL_API_KEY,
            "sentinel-cookie-value",
            "sentinel-bearer-token",
            SENTINEL_SESSION_FILE,
            SENTINEL_LOCAL_PATH,
            SENTINEL_UNIX_PATH,
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
            assert!(
                !json.contains(sentinel),
                "redacted json leaked {sentinel}: {json}"
            );
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

    #[test]
    fn sanitized_error_message_bounds_unicode_by_chars() {
        let long = format!(
            "runtime failed; message: private unicode body; path {SENTINEL_UNIX_PATH}; {}",
            "\u{1F642}".repeat(1000)
        );

        let output = sanitized_error_message(&long);

        assert!(!output.contains("private unicode body"));
        assert!(!output.contains(SENTINEL_UNIX_PATH));
        assert!(
            output.chars().count() <= MAX_SANITIZED_TEXT_CHARS,
            "bounded output was too long: {}",
            output.chars().count()
        );
        assert!(output.contains("[truncated]"));
    }
}
