use std::time::Duration;

use serde::Serialize;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

const YTDLP_RUNTIME_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeRuntimeStatusDto {
    pub ytdlp_available: bool,
    pub ytdlp_version: Option<String>,
    pub message: String,
}

#[tauri::command]
pub async fn get_youtube_runtime_status() -> AppResult<YoutubeRuntimeStatusDto> {
    let output = tokio::time::timeout(
        YTDLP_RUNTIME_CHECK_TIMEOUT,
        Command::new("yt-dlp").arg("--version").output(),
    )
    .await;

    match output {
        Ok(Ok(output)) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(YoutubeRuntimeStatusDto {
                ytdlp_available: true,
                ytdlp_version: if version.is_empty() {
                    None
                } else {
                    Some(version)
                },
                message: "yt-dlp is available".to_string(),
            })
        }
        Ok(Ok(output)) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(YoutubeRuntimeStatusDto {
                ytdlp_available: false,
                ytdlp_version: None,
                message: if stderr.is_empty() {
                    "yt-dlp is not available on PATH".to_string()
                } else {
                    format!("yt-dlp check failed: {stderr}")
                },
            })
        }
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(YoutubeRuntimeStatusDto {
                ytdlp_available: false,
                ytdlp_version: None,
                message: "yt-dlp is not available on PATH".to_string(),
            })
        }
        Ok(Err(error)) => Err(AppError::internal(format!("yt-dlp check failed: {error}"))),
        Err(_) => Ok(YoutubeRuntimeStatusDto {
            ytdlp_available: false,
            ytdlp_version: None,
            message: "yt-dlp runtime check timed out".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::YoutubeRuntimeStatusDto;

    #[test]
    fn runtime_status_serializes_with_camel_case_keys() {
        let status = YoutubeRuntimeStatusDto {
            ytdlp_available: false,
            ytdlp_version: None,
            message: "yt-dlp is not available on PATH".to_string(),
        };

        let json = serde_json::to_value(status).expect("serialize runtime status");

        assert_eq!(json["ytdlpAvailable"], false);
        assert_eq!(json["ytdlpVersion"], serde_json::Value::Null);
        assert_eq!(json["message"], "yt-dlp is not available on PATH");
    }
}
