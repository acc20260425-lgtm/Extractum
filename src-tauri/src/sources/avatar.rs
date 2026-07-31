use base64::{engine::general_purpose, Engine as _};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub(super) const TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS: u64 = 4_000;
const TELEGRAM_SOURCE_AVATAR_CACHE_DIR: &str = "source_avatars";

pub(super) fn photo_bytes_data_url(bytes: Vec<u8>) -> String {
    format!(
        "data:image/jpeg;base64,{}",
        general_purpose::STANDARD.encode(bytes)
    )
}

fn source_avatar_cache_key(account_id: i64, source_subtype: &str, external_id: &str) -> String {
    format!("{account_id}_{source_subtype}_{external_id}.jpg")
}

fn source_avatar_cache_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    Ok(handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(e.to_string()))?
        .join(TELEGRAM_SOURCE_AVATAR_CACHE_DIR))
}

pub(super) fn cache_source_avatar(
    handle: &AppHandle,
    account_id: i64,
    source_subtype: &str,
    external_id: &str,
    bytes: &[u8],
) -> AppResult<Option<String>> {
    if bytes.is_empty() {
        return Ok(None);
    }

    let cache_key = source_avatar_cache_key(account_id, source_subtype, external_id);
    let cache_dir = source_avatar_cache_dir(handle)?;
    fs::create_dir_all(&cache_dir).map_err(|e| AppError::internal(e.to_string()))?;
    fs::write(cache_dir.join(&cache_key), bytes).map_err(|e| AppError::internal(e.to_string()))?;
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
