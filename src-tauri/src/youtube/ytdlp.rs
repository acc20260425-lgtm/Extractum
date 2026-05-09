use std::io::ErrorKind;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

use crate::error::{AppError, AppResult};

use super::errors::classify_ytdlp_failure;

pub(crate) const YTDLP_PREVIEW_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct YtdlpOutput {
    pub(crate) stdout: String,
    #[allow(dead_code)]
    pub(crate) stderr: String,
}

pub(crate) async fn run_ytdlp(args: &[String]) -> AppResult<YtdlpOutput> {
    let output = timeout(YTDLP_PREVIEW_TIMEOUT, async {
        let mut command = Command::new("yt-dlp");
        command.args(args);
        command.output().await
    })
    .await
    .map_err(|_| AppError::network("yt-dlp preview timed out after 30 seconds"))?
    .map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            AppError::validation("yt-dlp is not available on PATH")
        } else {
            AppError::network(format!("Failed to run yt-dlp: {error}"))
        }
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(classify_ytdlp_failure(&stderr));
    }

    Ok(YtdlpOutput { stdout, stderr })
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
    use super::{preview_playlist_args, preview_video_args};

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
        assert_eq!(
            args.windows(2)
                .any(|pair| pair == ["--playlist-items", "1-50"]),
            true
        );
    }
}
