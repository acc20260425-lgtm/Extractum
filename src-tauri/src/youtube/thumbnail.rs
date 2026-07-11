use std::{collections::HashMap, sync::Arc, time::Duration};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppResult;
use tokio::sync::{watch, Mutex, Semaphore};

const ALLOWED_THUMBNAIL_HOSTS: [&str; 4] = [
    "i.ytimg.com",
    "i9.ytimg.com",
    "img.youtube.com",
    "yt3.ggpht.com",
];
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const FETCH_CONCURRENCY: usize = 6;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum YoutubeThumbnailResult {
    Success {
        #[serde(rename = "dataUrl")]
        data_url: String,
    },
    TerminalError {
        message: String,
    },
    TransientError {
        message: String,
    },
}

pub(crate) struct YoutubeThumbnailState {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
    in_flight: Mutex<HashMap<String, watch::Sender<Option<YoutubeThumbnailResult>>>>,
}

impl YoutubeThumbnailState {
    pub(crate) fn new() -> Self {
        Self {
            client: thumbnail_client(),
            semaphore: Arc::new(Semaphore::new(FETCH_CONCURRENCY)),
            in_flight: Mutex::new(HashMap::new()),
        }
    }
}

fn thumbnail_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .build()
        .expect("thumbnail HTTP client configuration must be valid")
}

#[tauri::command]
pub(crate) async fn resolve_youtube_thumbnail(
    url: String,
    state: State<'_, YoutubeThumbnailState>,
) -> AppResult<YoutubeThumbnailResult> {
    let url = match validate_thumbnail_url(&url) {
        Ok(url) => url,
        Err(message) => return Ok(YoutubeThumbnailResult::TerminalError { message }),
    };
    let key = url.to_string();

    let (sender, mut receiver, is_leader) = {
        let mut in_flight = state.in_flight.lock().await;
        if let Some(sender) = in_flight.get(&key) {
            (sender.clone(), sender.subscribe(), false)
        } else {
            let (sender, receiver) = watch::channel(None);
            in_flight.insert(key.clone(), sender.clone());
            (sender, receiver, true)
        }
    };

    if !is_leader {
        let _ = receiver.changed().await;
        return Ok(receiver.borrow().clone().unwrap_or_else(|| {
            YoutubeThumbnailResult::TransientError {
                message: "YouTube thumbnail request was cancelled".to_string(),
            }
        }));
    }

    let result = fetch_thumbnail(&state.client, state.semaphore.clone(), url).await;
    let _ = sender.send(Some(result.clone()));
    state.in_flight.lock().await.remove(&key);
    Ok(result)
}

async fn fetch_thumbnail(
    client: &reqwest::Client,
    semaphore: Arc<Semaphore>,
    url: reqwest::Url,
) -> YoutubeThumbnailResult {
    let _permit = match semaphore.acquire_owned().await {
        Ok(permit) => permit,
        Err(_) => {
            return YoutubeThumbnailResult::TransientError {
                message: "YouTube thumbnail downloader is unavailable".to_string(),
            }
        }
    };

    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(error) => return transient_error(error),
    };
    if !response.status().is_success() {
        return YoutubeThumbnailResult::TransientError {
            message: format!(
                "YouTube thumbnail request returned HTTP {}",
                response.status()
            ),
        };
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return YoutubeThumbnailResult::TerminalError {
            message: "YouTube thumbnail response exceeds 1 MiB".to_string(),
        };
    }

    let mut response = response;
    let mut bytes = Vec::new();
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                if bytes.len() + chunk.len() > MAX_RESPONSE_BYTES {
                    return YoutubeThumbnailResult::TerminalError {
                        message: "YouTube thumbnail response exceeds 1 MiB".to_string(),
                    };
                }
                bytes.extend_from_slice(&chunk);
            }
            Ok(None) => break,
            Err(error) => return transient_error(error),
        }
    }

    let Some(mime_type) = classify_image_bytes(&bytes) else {
        return YoutubeThumbnailResult::TerminalError {
            message: "YouTube thumbnail response is not a supported image".to_string(),
        };
    };
    YoutubeThumbnailResult::Success {
        data_url: format!("data:{mime_type};base64,{}", STANDARD.encode(bytes)),
    }
}

fn transient_error(error: reqwest::Error) -> YoutubeThumbnailResult {
    YoutubeThumbnailResult::TransientError {
        message: format!("YouTube thumbnail request failed: {error}"),
    }
}

fn validate_thumbnail_url(value: &str) -> Result<reqwest::Url, String> {
    let url =
        reqwest::Url::parse(value).map_err(|_| "Invalid YouTube thumbnail URL".to_string())?;
    let host = url
        .host_str()
        .ok_or_else(|| "YouTube thumbnail URL must include a host".to_string())?;
    if url.scheme() != "https" || !ALLOWED_THUMBNAIL_HOSTS.contains(&host) {
        return Err("YouTube thumbnail URL is not allowlisted".to_string());
    }
    Ok(url)
}

fn classify_image_bytes(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_image_bytes, thumbnail_client, validate_thumbnail_url, MAX_RESPONSE_BYTES,
    };

    #[test]
    fn accepts_only_allowlisted_https_thumbnail_urls() {
        assert!(validate_thumbnail_url("https://i.ytimg.com/vi/id/hqdefault.jpg").is_ok());
        assert!(validate_thumbnail_url("http://i.ytimg.com/vi/id/hqdefault.jpg").is_err());
        assert!(validate_thumbnail_url("https://example.com/image.jpg").is_err());
    }

    #[test]
    fn recognizes_supported_image_magic_bytes() {
        assert_eq!(
            classify_image_bytes(&[0xFF, 0xD8, 0xFF]),
            Some("image/jpeg")
        );
        assert_eq!(
            classify_image_bytes(b"\x89PNG\r\n\x1a\n"),
            Some("image/png")
        );
        assert_eq!(classify_image_bytes(b"RIFFxxxxWEBP"), Some("image/webp"));
        assert_eq!(classify_image_bytes(b"not an image"), None);
    }

    #[test]
    fn bounds_thumbnail_responses_to_one_mib() {
        assert_eq!(MAX_RESPONSE_BYTES, 1024 * 1024);
    }

    #[test]
    fn builds_the_dedicated_thumbnail_client() {
        let _client = thumbnail_client();
    }
}
