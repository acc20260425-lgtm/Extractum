# YouTube Sources Part 5: Auth and Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add optional YouTube auth/cookie support and user-configurable YouTube sync settings.

**Architecture:** Non-secret settings live in `app_settings`; cookie/session material lives only in OS secure storage through the existing `SecretStoreState`. `yt-dlp` receives cookies through temporary Netscape cookie files, never raw command-line cookie values, IPC responses, logs, or Tauri events.

**Tech Stack:** Tauri 2, Rust 2021, keyring-backed secure storage, sqlx SQLite, `yt-dlp`, Svelte 5, Vitest.

---

## Consistent End State

After this part:

- Public YouTube preview/sync still works without auth.
- Auth-sensitive runs can opt into stored YouTube cookies.
- Cookies are stored in OS secure storage and are never returned to the frontend after save.
- Invalid or empty cookie text is rejected with sanitized validation errors.
- Authenticated `yt-dlp` calls keep the same timeout guarantees as unauthenticated calls.
- Rate limit and captions settings roundtrip through backend validation and settings UI.
- Frontend YouTube settings wrappers live in a dedicated API module.

---

## Task 1: Cookie Secret Boundary

**Files:**

- Create: `src-tauri/src/youtube/cookies.rs`
- Modify: `src-tauri/src/youtube/ytdlp.rs`
- Modify: `src-tauri/src/secret_store.rs`
- Modify: `src-tauri/src/youtube/mod.rs`

- [ ] Add stable secret key helper in `src-tauri/src/secret_store.rs` next to the existing secret key helpers:

```rust
pub(crate) fn youtube_default_cookies_secret() -> String {
    "youtube.auth.default.cookies".to_string()
}
```

- [ ] Extend the existing `secret_store::tests::secret_ids_are_stable` test with the YouTube key:

```rust
assert_eq!(
    youtube_default_cookies_secret(),
    "youtube.auth.default.cookies"
);
```

- [ ] Add `cookies` module to `src-tauri/src/youtube/mod.rs`.

- [ ] In `src-tauri/src/youtube/cookies.rs`, implement the secure-storage boundary:

```rust
use crate::error::{AppError, AppResult};
use crate::secret_store::{youtube_default_cookies_secret, SecretStoreState};

pub(crate) async fn read_youtube_cookies(
    secrets: &SecretStoreState,
) -> AppResult<Option<String>> {
    secrets.get_secret(youtube_default_cookies_secret()).await
}

pub(crate) async fn save_youtube_cookies(
    secrets: &SecretStoreState,
    cookies: String,
) -> AppResult<()> {
    validate_netscape_cookie_file(&cookies)?;
    secrets
        .set_secret(youtube_default_cookies_secret(), cookies)
        .await
}

pub(crate) async fn clear_youtube_cookies(
    secrets: &SecretStoreState,
) -> AppResult<()> {
    secrets.delete_secret(youtube_default_cookies_secret()).await
}
```

Security boundary rules:

- `read_youtube_cookies` returns raw cookie text only inside backend code that writes the temporary cookie file.
- Raw cookie text must never be logged, included in `AppError.message`, returned from IPC, included in job records, or emitted through Tauri events.
- Validation errors may include a line number and reason, but must not include the line content or cookie value.
- Unit tests should assert returned command args contain only `--cookies` and the temp file path, not cookie names or cookie values.

- [ ] Add minimal Netscape cookie format validation before saving:

```rust
pub(crate) fn validate_netscape_cookie_file(cookies: &str) -> AppResult<()> {
    let mut cookie_rows = 0usize;

    if cookies.trim().is_empty() {
        return Err(AppError::validation(
            "YouTube cookies cannot be empty; use clear_youtube_auth to remove stored cookies",
        ));
    }

    for (index, line) in cookies.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') && !trimmed.starts_with("#HttpOnly_") {
            continue;
        }

        let cookie_line = trimmed.strip_prefix("#HttpOnly_").unwrap_or(trimmed);
        let fields: Vec<&str> = cookie_line.split('\t').collect();
        if fields.len() != 7 {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: expected 7 tab-separated fields",
            )));
        }

        let domain = fields[0].trim();
        let include_subdomains = fields[1].trim();
        let path = fields[2].trim();
        let secure = fields[3].trim();
        let expires = fields[4].trim();
        let name = fields[5].trim();

        if domain.is_empty() {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: domain is empty",
            )));
        }
        if include_subdomains != "TRUE" && include_subdomains != "FALSE" {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: include-subdomains must be TRUE or FALSE",
            )));
        }
        if !path.starts_with('/') {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: path must start with /",
            )));
        }
        if secure != "TRUE" && secure != "FALSE" {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: secure must be TRUE or FALSE",
            )));
        }
        if expires.parse::<i64>().is_err() {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: expires must be an integer timestamp",
            )));
        }
        if name.is_empty() {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: cookie name is empty",
            )));
        }

        cookie_rows += 1;
    }

    if cookie_rows == 0 {
        return Err(AppError::validation(
            "YouTube cookies must contain at least one Netscape cookie row",
        ));
    }

    Ok(())
}
```

Validation policy:

- Accept blank lines and comment/header lines.
- Treat `#HttpOnly_` rows as cookie rows after stripping only the `#HttpOnly_` prefix.
- Require exactly 7 tab-separated fields for cookie rows: domain, include-subdomains, path, secure, expires, name, value.
- `value` may be empty because Netscape cookie files permit empty values.
- Do not reject non-YouTube domains in this part; `yt-dlp` can ignore unrelated cookies exported by a browser.

- [ ] In `src-tauri/src/youtube/ytdlp.rs`, add authenticated execution support without changing existing unauthenticated call sites:

```rust
use std::time::Duration;

pub(crate) struct YtdlpRunOptions {
    pub(crate) timeout: Duration,
    pub(crate) cookies: Option<String>,
}

pub(crate) async fn run_ytdlp_with_options(
    args: &[String],
    options: YtdlpRunOptions,
) -> AppResult<YtdlpOutput>;
```

Execution requirements:

- If `options.cookies` is `Some`, validate it again with `validate_netscape_cookie_file`.
- Write cookie text to a `tempfile::NamedTempFile` or `TempDir`-owned file in Netscape format.
- Add `--cookies` and the temp file path as two separate command args.
- Never pass raw cookie content on the command line.
- Keep the temp file alive until the `yt-dlp` child process exits.
- Drop the temp file after the process completes or times out.
- Wrap process execution with `tokio::time::timeout(options.timeout, ...)`.
- Existing `run_ytdlp(args)` should call `run_ytdlp_with_options(args, YtdlpRunOptions { timeout: Duration::from_secs(30), cookies: None })` so Part 2 preview behavior stays unchanged.

Timeout policy:

- Cookie injection must not create an unbounded `yt-dlp` path.
- Preview with or without cookies uses the existing 30 second preview timeout.
- Metadata, transcript, playlist, and comment sync callers must pass their existing Part 3/Part 4 timeout constants. If a caller does not yet have a named timeout, define one near that caller and use it for both auth and no-auth execution.

- [ ] Add tests using `InMemorySecretStore` from `secret_store.rs`.

Minimum backend tests:

```rust
#[test]
fn validates_netscape_cookie_rows_without_exposing_values() {
    let cookies = "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n";
    assert!(validate_netscape_cookie_file(cookies).is_ok());

    let err = validate_netscape_cookie_file(".youtube.com TRUE / TRUE 1893456000 SID secret-value")
        .expect_err("space separated cookies should fail");
    assert!(!err.message.contains("secret-value"));
}

#[test]
fn rejects_empty_cookie_text() {
    let err = validate_netscape_cookie_file("  \n\t")
        .expect_err("empty cookie text should fail");
    assert!(err.message.contains("cannot be empty"));
}
```

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::cookies secret_store --lib
```

Expected: cookie read/write/delete passes, `secret_ids_are_stable` includes the YouTube key, cookie validation passes, invalid cookie errors are sanitized, and raw cookie content is not part of command args.

- [ ] Commit:

```powershell
git add src-tauri/src/youtube src-tauri/src/secret_store.rs
git commit -m "feat: store youtube cookies securely"
```

---

## Task 2: YouTube Settings Commands

**Files:**

- Modify: `src-tauri/src/youtube/cookies.rs`
- Create: `src-tauri/src/youtube/settings.rs`
- Modify: `src-tauri/src/youtube/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/lib/types/youtube.ts`
- Create: `src/lib/api/youtube-settings.ts`
- Create: `src/lib/api/youtube-settings.test.ts`

- [ ] Execute this task in two internal checkpoints so the settings contract stays reviewable:

```text
Checkpoint A: Rust DTOs, validation, app_settings helpers, shared internal helpers, Tauri commands, and Rust tests.
Checkpoint B: TypeScript types, frontend API wrappers, and Vitest contract tests.
```

- [ ] Add `settings` module to `src-tauri/src/youtube/mod.rs`.

- [ ] In `src-tauri/src/youtube/settings.rs`, define explicit app setting keys matching the migration added in Part 1:

```rust
const AUTH_ENABLED_KEY: &str = "youtube.auth.enabled";
const PREFERRED_CAPTIONS_LANGUAGE_KEY: &str = "youtube.captions.preferred_language";
const DELAY_BETWEEN_REQUESTS_MS_KEY: &str = "youtube.sync.delay_between_requests_ms";
const MAX_PARALLEL_VIDEO_SYNCS_KEY: &str = "youtube.sync.max_parallel_video_syncs";
const MAX_PARALLEL_COMMENT_SYNCS_KEY: &str = "youtube.sync.max_parallel_comment_syncs";
const PAUSE_ON_AUTH_CHALLENGE_KEY: &str = "youtube.sync.pause_on_auth_challenge";
const DAILY_SOFT_LIMIT_KEY: &str = "youtube.sync.daily_soft_limit";
const RETRY_BACKOFF_MS_KEY: &str = "youtube.sync.retry_backoff_ms";
const STOP_AFTER_CONSECUTIVE_FAILURES_KEY: &str =
    "youtube.sync.stop_after_consecutive_failures";
```

- [ ] Add typed Rust DTOs with concrete ranges:

```rust
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSettingsDto {
    pub auth_enabled: bool,
    pub preferred_captions_language: String,
    pub delay_between_requests_ms: i64,
    pub max_parallel_video_syncs: i64,
    pub max_parallel_comment_syncs: i64,
    pub pause_on_auth_challenge: bool,
    pub daily_soft_limit: i64,
    pub retry_backoff_ms: i64,
    pub stop_after_consecutive_failures: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeAuthStatusDto {
    pub enabled: bool,
    pub has_cookies: bool,
    pub message: String,
}
```

Validation ranges:

```text
auth_enabled: bool
preferred_captions_language: "original" or 2-32 ASCII letters/digits/hyphen/underscore chars
delay_between_requests_ms: i64, 0..=60000, where 0 means no deliberate delay
max_parallel_video_syncs: i64, 1..=4
max_parallel_comment_syncs: i64, 1..=2
pause_on_auth_challenge: bool
daily_soft_limit: i64, 0..=10000, where 0 means no daily soft limit
retry_backoff_ms: i64, 0..=300000
stop_after_consecutive_failures: i64, 1..=50
```

Default values when `app_settings` rows are missing:

```rust
YoutubeSettingsDto {
    auth_enabled: false,
    preferred_captions_language: "original".to_string(),
    delay_between_requests_ms: 1_000,
    max_parallel_video_syncs: 1,
    max_parallel_comment_syncs: 1,
    pause_on_auth_challenge: true,
    daily_soft_limit: 0,
    retry_backoff_ms: 3_000,
    stop_after_consecutive_failures: 3,
}
```

- [ ] Implement `validate_youtube_settings(settings: YoutubeSettingsDto) -> AppResult<YoutubeSettingsDto>`:

```rust
fn validate_range(value: i64, min: i64, max: i64, label: &str) -> AppResult<i64> {
    if value < min || value > max {
        return Err(AppError::validation(format!(
            "{label} must be between {min} and {max}"
        )));
    }
    Ok(value)
}

fn validate_preferred_captions_language(value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("original") {
        return Ok("original".to_string());
    }
    let valid = (2..=32).contains(&trimmed.len())
        && trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !valid {
        return Err(AppError::validation(
            "Preferred captions language must be 'original' or a 2-32 character language code",
        ));
    }
    Ok(trimmed.to_ascii_lowercase())
}
```

Invalid values such as `delay_between_requests_ms = -1`, `max_parallel_video_syncs = 0`, or `max_parallel_video_syncs = 100` must return `AppError::validation` and must not write partial settings.

- [ ] Implement typed `app_settings` helpers in `settings.rs`, following the existing pattern in `src-tauri/src/sources/settings.rs`:

```rust
async fn read_setting(pool: &sqlx::Pool<sqlx::Sqlite>, key: &str) -> AppResult<Option<String>>;
async fn write_setting(pool: &sqlx::Pool<sqlx::Sqlite>, key: &str, value: &str) -> AppResult<()>;
fn parse_bool_setting(value: Option<String>, default: bool, key: &str) -> AppResult<bool>;
fn parse_i64_setting(value: Option<String>, default: i64, key: &str) -> AppResult<i64>;
```

Read policy:

- Missing rows use defaults.
- Invalid stored bool or integer values return `AppError::validation` naming the setting key, not an internal panic.
- Loaded settings are passed through `validate_youtube_settings` before returning.

Write policy:

- Validate the whole DTO before writing anything.
- Persist bools as `"true"` or `"false"`.
- Persist integers as decimal strings.
- Persist `preferred_captions_language` as the normalized string from validation.

- [ ] Add internal helpers so commands and tests can share the same logic without duplicating Tauri command setup:

```rust
pub(crate) fn default_youtube_settings() -> YoutubeSettingsDto;

pub(crate) async fn load_youtube_settings_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> AppResult<YoutubeSettingsDto>;

async fn save_youtube_settings_to_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    settings: &YoutubeSettingsDto,
) -> AppResult<YoutubeSettingsDto>;

pub(crate) async fn youtube_auth_status_from_state(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secrets: &SecretStoreState,
) -> AppResult<YoutubeAuthStatusDto>;

pub(crate) async fn save_youtube_cookies_to_state(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secrets: &SecretStoreState,
    cookies: String,
) -> AppResult<YoutubeAuthStatusDto>;

pub(crate) async fn clear_youtube_auth_in_state(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    secrets: &SecretStoreState,
) -> AppResult<YoutubeAuthStatusDto>;
```

Helper behavior:

- `default_youtube_settings` returns the default DTO shown above.
- `load_youtube_settings_from_pool` reads all setting keys, applies missing-row defaults, parses types, and validates the result.
- `save_youtube_settings_to_pool` validates first, writes all non-secret settings, then returns the normalized DTO.
- `youtube_auth_status_from_state` loads settings, checks only `read_youtube_cookies(...).await?.is_some()`, and returns `YoutubeAuthStatusDto`.
- `save_youtube_cookies_to_state` validates and stores cookies through `youtube::cookies`, writes `youtube.auth.enabled = "true"`, and returns fresh auth status.
- `clear_youtube_auth_in_state` deletes cookies through `youtube::cookies`, writes `youtube.auth.enabled = "false"`, and returns fresh auth status.

- [ ] Add auth status message policy:

```rust
fn auth_status_message(enabled: bool, has_cookies: bool) -> &'static str {
    match (enabled, has_cookies) {
        (false, _) => "Auth disabled",
        (true, true) => "Cookies stored",
        (true, false) => "No cookies configured",
    }
}
```

Frontend may display `message`, but must not branch on free-form message text. Backend behavior branches on `enabled` and `has_cookies`.

- [ ] Implement commands:

```rust
#[tauri::command]
pub async fn get_youtube_settings(handle: AppHandle) -> AppResult<YoutubeSettingsDto>;

#[tauri::command]
pub async fn save_youtube_settings(
    handle: AppHandle,
    settings: YoutubeSettingsDto,
) -> AppResult<YoutubeSettingsDto>;

#[tauri::command]
pub async fn get_youtube_auth_status(
    handle: AppHandle,
    secrets: tauri::State<'_, SecretStoreState>,
) -> AppResult<YoutubeAuthStatusDto>;

#[tauri::command]
pub async fn save_youtube_cookies(
    handle: AppHandle,
    secrets: tauri::State<'_, SecretStoreState>,
    cookies: String,
) -> AppResult<YoutubeAuthStatusDto>;

#[tauri::command]
pub async fn clear_youtube_auth(
    handle: AppHandle,
    secrets: tauri::State<'_, SecretStoreState>,
) -> AppResult<YoutubeAuthStatusDto>;
```

Command behavior:

- `get_youtube_settings` returns validated non-secret settings only.
- `save_youtube_settings` persists non-secret settings only and never reads or writes cookie content.
- `get_youtube_auth_status` reads only whether the secret exists, not the secret value for IPC.
- `save_youtube_cookies` rejects empty or invalid text using `validate_netscape_cookie_file`, stores valid cookies, writes `youtube.auth.enabled = "true"`, and returns `enabled = true`, `has_cookies = true`, `message = "Cookies stored"`.
- `clear_youtube_auth` deletes the cookie secret, writes `youtube.auth.enabled = "false"`, and returns `enabled = false`, `has_cookies = false`, `message = "Auth disabled"`.
- Empty or whitespace-only `cookies` is a validation error, not an alias for `clear_youtube_auth`.

- [ ] Register commands in `src-tauri/src/lib.rs`.

- [ ] Add Rust tests.

Minimum backend tests:

```rust
#[tokio::test]
async fn youtube_settings_default_when_app_settings_are_missing() {
    let pool = crate::sources::test_support::memory_pool().await;
    let settings = load_youtube_settings_from_pool(&pool)
        .await
        .expect("load defaults");
    assert_eq!(settings.delay_between_requests_ms, 1_000);
    assert_eq!(settings.daily_soft_limit, 0);
    assert!(!settings.auth_enabled);
}

#[test]
fn validate_youtube_settings_rejects_out_of_range_values() {
    let mut settings = default_youtube_settings();
    settings.delay_between_requests_ms = -1;
    assert!(validate_youtube_settings(settings).is_err());

    let mut settings = default_youtube_settings();
    settings.max_parallel_video_syncs = 100;
    assert!(validate_youtube_settings(settings).is_err());
}

#[test]
fn youtube_settings_serializes_with_camel_case_keys() {
    let value = serde_json::to_value(default_youtube_settings())
        .expect("serialize youtube settings");
    assert!(value.get("authEnabled").is_some());
    assert!(value.get("preferredCaptionsLanguage").is_some());
    assert!(value.get("delayBetweenRequestsMs").is_some());
    assert!(value.get("maxParallelVideoSyncs").is_some());
    assert!(value.get("maxParallelCommentSyncs").is_some());
    assert!(value.get("pauseOnAuthChallenge").is_some());
    assert!(value.get("dailySoftLimit").is_some());
    assert!(value.get("retryBackoffMs").is_some());
    assert!(value.get("stopAfterConsecutiveFailures").is_some());
    assert!(value.get("auth_enabled").is_none());
}

#[tokio::test]
async fn saving_cookies_enables_auth_and_clear_disables_it() {
    let pool = crate::sources::test_support::memory_pool().await;
    let store = std::sync::Arc::new(crate::secret_store::tests::InMemorySecretStore::new());
    let secrets = SecretStoreState::new(store);
    let cookies = ".youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n".to_string();

    let saved = save_youtube_cookies_to_state(&pool, &secrets, cookies)
        .await
        .expect("save cookies");
    assert!(saved.enabled);
    assert!(saved.has_cookies);
    assert_eq!(saved.message, "Cookies stored");

    let cleared = clear_youtube_auth_in_state(&pool, &secrets)
        .await
        .expect("clear auth");
    assert!(!cleared.enabled);
    assert!(!cleared.has_cookies);
    assert_eq!(cleared.message, "Auth disabled");
}
```

- [ ] Add frontend types in `src/lib/types/youtube.ts`:

```ts
export interface YoutubeSettings {
  authEnabled: boolean;
  preferredCaptionsLanguage: string;
  delayBetweenRequestsMs: number;
  maxParallelVideoSyncs: number;
  maxParallelCommentSyncs: number;
  pauseOnAuthChallenge: boolean;
  dailySoftLimit: number;
  retryBackoffMs: number;
  stopAfterConsecutiveFailures: number;
}

export interface YoutubeAuthStatus {
  enabled: boolean;
  hasCookies: boolean;
  message: "Auth disabled" | "Cookies stored" | "No cookies configured" | string;
}
```

- [ ] Add frontend API wrappers in `src/lib/api/youtube-settings.ts` instead of adding more YouTube settings code to `src/lib/api/sources.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { YoutubeAuthStatus, YoutubeSettings } from "$lib/types/youtube";

interface RawYoutubeSettings {
  authEnabled: boolean;
  preferredCaptionsLanguage: string;
  delayBetweenRequestsMs: number;
  maxParallelVideoSyncs: number;
  maxParallelCommentSyncs: number;
  pauseOnAuthChallenge: boolean;
  dailySoftLimit: number;
  retryBackoffMs: number;
  stopAfterConsecutiveFailures: number;
}

// This intentionally mirrors YoutubeSettings because the Rust DTO serializes
// with #[serde(rename_all = "camelCase")]. Keep the Rust serialization test
// and the Vitest wrapper test in place so a future rename mismatch fails at
// the API boundary.

export function getYoutubeSettings() {
  return invoke<RawYoutubeSettings>("get_youtube_settings").then(mapYoutubeSettings);
}

export function saveYoutubeSettings(settings: YoutubeSettings) {
  return invoke<RawYoutubeSettings>("save_youtube_settings", { settings }).then(
    mapYoutubeSettings,
  );
}

export function getYoutubeAuthStatus() {
  return invoke<YoutubeAuthStatus>("get_youtube_auth_status");
}

export function saveYoutubeCookies(cookies: string) {
  return invoke<YoutubeAuthStatus>("save_youtube_cookies", { cookies });
}

export function clearYoutubeAuth() {
  return invoke<YoutubeAuthStatus>("clear_youtube_auth");
}

function mapYoutubeSettings(settings: RawYoutubeSettings): YoutubeSettings {
  return {
    authEnabled: settings.authEnabled,
    preferredCaptionsLanguage: settings.preferredCaptionsLanguage,
    delayBetweenRequestsMs: settings.delayBetweenRequestsMs,
    maxParallelVideoSyncs: settings.maxParallelVideoSyncs,
    maxParallelCommentSyncs: settings.maxParallelCommentSyncs,
    pauseOnAuthChallenge: settings.pauseOnAuthChallenge,
    dailySoftLimit: settings.dailySoftLimit,
    retryBackoffMs: settings.retryBackoffMs,
    stopAfterConsecutiveFailures: settings.stopAfterConsecutiveFailures,
  };
}
```

- [ ] Add explicit Vitest tests in `src/lib/api/youtube-settings.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  clearYoutubeAuth,
  getYoutubeAuthStatus,
  getYoutubeSettings,
  saveYoutubeCookies,
  saveYoutubeSettings,
} from "./youtube-settings";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("youtube settings API", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads youtube settings", async () => {
    invokeMock.mockResolvedValueOnce({
      authEnabled: false,
      preferredCaptionsLanguage: "original",
      delayBetweenRequestsMs: 1000,
      maxParallelVideoSyncs: 1,
      maxParallelCommentSyncs: 1,
      pauseOnAuthChallenge: true,
      dailySoftLimit: 0,
      retryBackoffMs: 3000,
      stopAfterConsecutiveFailures: 3,
    });

    await expect(getYoutubeSettings()).resolves.toMatchObject({
      authEnabled: false,
      delayBetweenRequestsMs: 1000,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_settings");
  });

  it("saves youtube settings with a settings argument", async () => {
    const settings = {
      authEnabled: true,
      preferredCaptionsLanguage: "en",
      delayBetweenRequestsMs: 500,
      maxParallelVideoSyncs: 2,
      maxParallelCommentSyncs: 1,
      pauseOnAuthChallenge: true,
      dailySoftLimit: 200,
      retryBackoffMs: 5000,
      stopAfterConsecutiveFailures: 4,
    };
    invokeMock.mockResolvedValueOnce(settings);

    await expect(saveYoutubeSettings(settings)).resolves.toMatchObject(settings);
    expect(invokeMock).toHaveBeenLastCalledWith("save_youtube_settings", {
      settings,
    });
  });

  it("reads auth status without exposing cookies", async () => {
    invokeMock.mockResolvedValueOnce({
      enabled: true,
      hasCookies: true,
      message: "Cookies stored",
    });

    await expect(getYoutubeAuthStatus()).resolves.toMatchObject({
      enabled: true,
      hasCookies: true,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_auth_status");
  });

  it("saves and clears youtube cookies through dedicated commands", async () => {
    invokeMock.mockResolvedValueOnce({
      enabled: true,
      hasCookies: true,
      message: "Cookies stored",
    });

    await saveYoutubeCookies("cookie text");
    expect(invokeMock).toHaveBeenLastCalledWith("save_youtube_cookies", {
      cookies: "cookie text",
    });

    invokeMock.mockResolvedValueOnce({
      enabled: false,
      hasCookies: false,
      message: "Auth disabled",
    });

    await clearYoutubeAuth();
    expect(invokeMock).toHaveBeenLastCalledWith("clear_youtube_auth");
  });
});
```

- [ ] Run:

```powershell
cd src-tauri
cargo test youtube::settings youtube::cookies secret_store --lib
cd ..
npm test -- youtube-settings
npm run check
```

Expected: settings persist in `app_settings`, out-of-range inputs fail before writes, auth status reflects secure cookie state without returning cookie content, frontend API wrapper tests pass, and the frontend compiles.

- [ ] Commit:

```powershell
git add src-tauri/src/youtube src-tauri/src/lib.rs src/lib/types/youtube.ts src/lib/api/youtube-settings.ts src/lib/api/youtube-settings.test.ts
git commit -m "feat: add youtube settings commands"
```

---

## Task 3: Settings UI

**Files:**

- Modify: `src/routes/settings/+page.svelte`
- Create: `src/lib/components/settings/youtube-settings-panel.svelte`
- Modify: `src/lib/api/youtube-settings.ts`
- Modify: `src/lib/types/youtube.ts`

- [ ] Add a focused `src/lib/components/settings/youtube-settings-panel.svelte` component and mount it from `/settings` instead of putting all YouTube settings state directly in `src/routes/settings/+page.svelte`.

- [ ] On panel mount, load both:

```ts
const [settings, authStatus] = await Promise.all([
  getYoutubeSettings(),
  getYoutubeAuthStatus(),
]);
```

- [ ] Show auth status from `YoutubeAuthStatus.message`:

```text
Auth disabled
Cookies stored
No cookies configured
```

Display rules:

- If `authStatus.hasCookies` is true, show only the status text and an "Update cookies" control.
- Never render stored cookie text back into an input.
- Keep pasted cookies only in local component state before save.
- After a successful save, clear the textarea state and reload auth status.
- Empty or whitespace-only textarea values keep the save-cookies button disabled; backend still validates and rejects empty text.

- [ ] Add cookie input UX as a hidden-by-default multiline textarea:

```svelte
{#if editingCookies}
  <textarea
    bind:value={cookieText}
    rows="8"
    spellcheck="false"
    autocomplete="off"
    autocapitalize="off"
  ></textarea>
{/if}
```

UI controls:

```text
Enable YouTube auth
Paste/update cookies
Save cookies
Cancel cookie edit
Clear YouTube auth
Preferred captions language
Delay between requests
Max parallel video syncs
Max parallel comment syncs
Pause on auth challenge
Daily soft limit
Retry backoff
Stop after consecutive failures
Save settings
```

- [ ] Match frontend input constraints to backend validation:

```text
preferred captions language: text input, placeholder "original", max length 32
delay between requests: number input, min 0, max 60000, step 100
max parallel video syncs: number input, min 1, max 4, step 1
max parallel comment syncs: number input, min 1, max 2, step 1
daily soft limit: number input, min 0, max 10000, step 1
retry backoff: number input, min 0, max 300000, step 100
stop after consecutive failures: number input, min 1, max 50, step 1
pause on auth challenge: checkbox
auth enabled: checkbox
```

Use `0` helper copy in labels only where needed: daily soft limit `0` means no soft limit, delay `0` means no deliberate delay, retry backoff `0` means no wait before retry.

- [ ] Save non-secret values via `saveYoutubeSettings`.

- [ ] Save cookies via `saveYoutubeCookies`; display only the returned `YoutubeAuthStatus.message`.

- [ ] Clear cookies via `clearYoutubeAuth`; this should also set `authEnabled` false in the local settings state after the command returns.

- [ ] Keep API wrapper tests from Task 2 in the verification command for this UI task so new wiring is not untested.

- [ ] Run:

```powershell
npm test -- youtube-settings
npm run check
```

Expected: settings UI typechecks, does not render stored cookies, uses the dedicated YouTube settings API module, and API mapping tests pass.

- [ ] Commit:

```powershell
git add src/routes/settings/+page.svelte src/lib/components/settings/youtube-settings-panel.svelte src/lib/api/youtube-settings.ts src/lib/types/youtube.ts
git commit -m "feat: add youtube settings UI"
```

---

## Manual Verification

- [ ] Clear auth and preview a public video.
- [ ] Save an empty cookie textarea and confirm the UI prevents submission; if called directly, backend returns a validation error.
- [ ] Save malformed cookie text and confirm the backend error names the line/reason without echoing cookie content.
- [ ] Save valid Netscape cookies and confirm `get_youtube_auth_status` reports `enabled = true`, `has_cookies = true`, `message = "Cookies stored"`.
- [ ] Confirm the settings UI never displays the stored cookie text after save or reload.
- [ ] Run an authenticated preview/sync path with temporary cookie file support and confirm `yt-dlp` receives `--cookies <temp-path>`.
- [ ] Confirm authenticated `yt-dlp` preview still times out after the preview timeout instead of running unbounded.
- [ ] Clear auth and confirm cookies are removed from secure storage and `youtube.auth.enabled` becomes `false`.
