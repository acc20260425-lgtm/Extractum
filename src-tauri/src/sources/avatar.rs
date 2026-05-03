use base64::{engine::general_purpose, Engine as _};
use grammers_client::peer::Peer;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};
use tokio::time::{timeout, Duration};

const TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS: u64 = 750;
pub(super) const TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS: u64 = 4_000;
const TELEGRAM_SOURCE_AVATAR_CACHE_DIR: &str = "source_avatars";

pub(super) async fn peer_photo_data_url_with_timeout(
    client: &grammers_client::Client,
    peer: &Peer,
) -> Option<String> {
    peer_photo_bytes_with_timeout(client, peer)
        .await
        .map(photo_bytes_data_url)
}

pub(super) async fn peer_photo_bytes_with_timeout(
    client: &grammers_client::Client,
    peer: &Peer,
) -> Option<Vec<u8>> {
    timeout(
        Duration::from_millis(TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS),
        peer_photo_bytes(client, peer),
    )
    .await
    .ok()
    .and_then(Result::ok)
    .flatten()
}

async fn peer_photo_bytes(
    client: &grammers_client::Client,
    peer: &Peer,
) -> Result<Option<Vec<u8>>, String> {
    let Some(photo) = peer.photo(false).await else {
        return Ok(None);
    };

    let mut bytes = Vec::new();
    let mut download = client.iter_download(&photo).chunk_size(4 * 1024);
    while let Some(chunk) = download.next().await.map_err(|e| e.to_string())? {
        bytes.extend(chunk);
    }

    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(bytes))
}

fn photo_bytes_data_url(bytes: Vec<u8>) -> String {
    format!(
        "data:image/jpeg;base64,{}",
        general_purpose::STANDARD.encode(bytes)
    )
}

fn source_avatar_cache_key(
    account_id: i64,
    telegram_source_kind: &str,
    external_id: &str,
) -> String {
    format!("{account_id}_{telegram_source_kind}_{external_id}.jpg")
}

fn source_avatar_cache_dir(handle: &AppHandle) -> Result<PathBuf, String> {
    Ok(handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(TELEGRAM_SOURCE_AVATAR_CACHE_DIR))
}

pub(super) fn cache_source_avatar(
    handle: &AppHandle,
    account_id: i64,
    telegram_source_kind: &str,
    external_id: &str,
    bytes: &[u8],
) -> Result<Option<String>, String> {
    if bytes.is_empty() {
        return Ok(None);
    }

    let cache_key = source_avatar_cache_key(account_id, telegram_source_kind, external_id);
    let cache_dir = source_avatar_cache_dir(handle)?;
    fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    fs::write(cache_dir.join(&cache_key), bytes).map_err(|e| e.to_string())?;
    Ok(Some(cache_key))
}

pub(super) fn read_source_avatar_data_url(handle: &AppHandle, cache_key: &str) -> Option<String> {
    if cache_key.contains(['/', '\\']) {
        return None;
    }

    let path = source_avatar_cache_dir(handle).ok()?.join(cache_key);
    let bytes = fs::read(path).ok()?;
    if bytes.is_empty() {
        return None;
    }

    Some(photo_bytes_data_url(bytes))
}
