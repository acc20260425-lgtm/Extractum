use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub(crate) const GEMINI_BROWSER_DIR: &str = "gemini-browser";
pub(crate) const PROFILE_DIR: &str = "profile";
pub(crate) const RUNS_DIR: &str = "runs";

pub(crate) fn base_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    Ok(handle
        .path()
        .app_data_dir()
        .map_err(|error| AppError::internal(error.to_string()))?
        .join(GEMINI_BROWSER_DIR))
}

pub(crate) fn profile_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    let path = base_dir(handle)?.join(PROFILE_DIR);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn runs_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    let path = base_dir(handle)?.join(RUNS_DIR);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn run_dir(handle: &AppHandle, run_id: &str) -> AppResult<PathBuf> {
    let path = runs_dir(handle)?.join(safe_run_id(run_id)?);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn safe_run_id(run_id: &str) -> AppResult<String> {
    let candidate = run_id.trim();
    if candidate.is_empty() {
        return Err(AppError::validation("run_id cannot be empty"));
    }
    if candidate
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        Ok(candidate.to_string())
    } else {
        Err(AppError::validation(
            "run_id can only contain ASCII letters, numbers, dashes, and underscores",
        ))
    }
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
