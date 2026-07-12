use std::io::Write;
use std::path::Path;
use std::time::Duration;

use tempfile::NamedTempFile;
use tokio_util::sync::CancellationToken;
use crate::error::{AppError, AppResult};
use crate::external_process::ExternalProcessShutdownState;

use super::cookies::validate_netscape_cookie_file;
use super::process_runtime::{run_ytdlp_managed_with_cancellation, CookieLifetimeGuard, YoutubeProcessRegistry};

pub(crate) const YTDLP_PREVIEW_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct YtdlpOutput {
    pub(crate) stdout: String,
    #[allow(dead_code)]
    pub(crate) stderr: String,
}

pub(crate) struct YtdlpRunOptions {
    pub(crate) timeout: Duration,
    pub(crate) cookies: Option<String>,
    pub(crate) cancellation: Option<CancellationToken>,
}

#[allow(dead_code)]
pub(crate) async fn run_ytdlp(
    registry: &YoutubeProcessRegistry,
    shutdown: &ExternalProcessShutdownState,
    args: &[String],
) -> AppResult<YtdlpOutput> {
    run_ytdlp_with_options(
        registry,
        shutdown,
        args,
        YtdlpRunOptions {
            timeout: YTDLP_PREVIEW_TIMEOUT,
            cookies: None,
            cancellation: None,
        },
    )
    .await
}

pub(crate) async fn run_ytdlp_with_options(
    registry: &YoutubeProcessRegistry,
    shutdown: &ExternalProcessShutdownState,
    args: &[String],
    options: YtdlpRunOptions,
) -> AppResult<YtdlpOutput> {
    let cookie_file = if let Some(cookies) = options.cookies {
        validate_netscape_cookie_file(&cookies)?;
        let cookie_file_content = ytdlp_cookie_file_content(&cookies);
        let mut file = NamedTempFile::new().map_err(|error| {
            AppError::internal(format!("Failed to create YouTube cookie file: {error}"))
        })?;
        file.write_all(cookie_file_content.as_bytes())
            .map_err(|error| {
                AppError::internal(format!("Failed to write YouTube cookie file: {error}"))
            })?;
        file.flush().map_err(|error| {
            AppError::internal(format!("Failed to write YouTube cookie file: {error}"))
        })?;
        Some(file)
    } else {
        None
    };
    let command_args = ytdlp_command_args(args, cookie_file.as_ref().map(|file| file.path()));
    let cookie_guard = cookie_file.map(CookieLifetimeGuard::new);

    let (stdout, stderr) = run_ytdlp_managed_with_cancellation(
        registry, shutdown, &command_args, options.timeout, timeout_message(options.timeout), cookie_guard, options.cancellation,
    ).await?;

    Ok(YtdlpOutput { stdout, stderr })
}

fn ytdlp_command_args(args: &[String], cookies_path: Option<&Path>) -> Vec<String> {
    let mut command_args = Vec::with_capacity(args.len() + 2);
    if let Some(path) = cookies_path {
        command_args.push("--cookies".to_string());
        command_args.push(path.to_string_lossy().to_string());
    }
    command_args.extend(args.iter().cloned());
    command_args
}

fn ytdlp_cookie_file_content(cookies: &str) -> String {
    let has_netscape_header = cookies.lines().any(|line| {
        line.trim()
            .eq_ignore_ascii_case("# Netscape HTTP Cookie File")
    });
    if has_netscape_header {
        cookies.to_string()
    } else {
        format!("# Netscape HTTP Cookie File\n{cookies}")
    }
}

fn timeout_message(timeout: Duration) -> String {
    if timeout == YTDLP_PREVIEW_TIMEOUT {
        "yt-dlp preview timed out after 30 seconds".to_string()
    } else {
        format!("yt-dlp timed out after {} seconds", timeout.as_secs())
    }
}

pub(crate) fn preview_video_args(canonical_url: &str) -> Vec<String> {
    vec![
        "--dump-single-json".to_string(),
        "--skip-download".to_string(),
        canonical_url.to_string(),
    ]
}

pub(crate) fn preview_playlist_args(canonical_url: &str) -> Vec<String> {
    vec![
        "--dump-single-json".to_string(),
        "--flat-playlist".to_string(),
        "--playlist-items".to_string(),
        "1-50".to_string(),
        "--skip-download".to_string(),
        canonical_url.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        preview_playlist_args, preview_video_args, ytdlp_command_args, ytdlp_cookie_file_content,
    };

    #[test]
    fn preview_video_args_use_dump_json_without_shell_fragments() {
        let args = preview_video_args("https://www.youtube.com/watch?v=abc123");

        assert_eq!(
            args,
            vec![
                "--dump-single-json",
                "--skip-download",
                "https://www.youtube.com/watch?v=abc123"
            ]
        );
    }

    #[test]
    fn preview_playlist_args_limit_entries_to_first_fifty() {
        let args = preview_playlist_args("https://www.youtube.com/playlist?list=PLabc123");

        assert_eq!(
            args,
            vec![
                "--dump-single-json",
                "--flat-playlist",
                "--playlist-items",
                "1-50",
                "--skip-download",
                "https://www.youtube.com/playlist?list=PLabc123"
            ]
        );
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--playlist-items", "1-50"]));
    }

    #[test]
    fn authenticated_command_args_include_cookie_file_path_without_cookie_content() {
        let base_args = vec![
            "--dump-single-json".to_string(),
            "https://www.youtube.com/watch?v=abc123".to_string(),
        ];
        let cookie_path = std::path::Path::new("C:\\Temp\\extractum-youtube-cookies.txt");
        let command_args = ytdlp_command_args(&base_args, Some(cookie_path));

        assert!(command_args
            .windows(2)
            .any(|pair| { pair == ["--cookies", "C:\\Temp\\extractum-youtube-cookies.txt"] }));
        assert!(!command_args.iter().any(|arg| arg.contains("SID")));
        assert!(!command_args.iter().any(|arg| arg.contains("secret-value")));
    }

    #[test]
    fn cookie_file_content_adds_netscape_header_when_missing() {
        let content = ytdlp_cookie_file_content(
            ".youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n",
        );

        assert!(content.starts_with("# Netscape HTTP Cookie File\n"));
        assert!(content.contains(".youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value"));
    }

    #[test]
    fn cookie_file_content_preserves_existing_netscape_header() {
        let cookies =
            "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n";

        assert_eq!(ytdlp_cookie_file_content(cookies), cookies);
    }
}
