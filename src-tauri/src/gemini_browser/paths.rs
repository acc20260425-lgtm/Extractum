use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::executor::domain_error_to_app;
use crate::error::{AppError, AppResult};
use extractum_gemini_browser::safe_run_id;

pub(crate) const GEMINI_BROWSER_DIR: &str = "gemini-browser";
pub(crate) const PROFILE_DIR: &str = "profile";
pub(crate) const CHROME_CDP_PROFILE_DIR: &str = "chrome-cdp-profile";
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

pub(crate) fn chrome_cdp_profile_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    let path = base_dir(handle)?.join(CHROME_CDP_PROFILE_DIR);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn runs_dir(handle: &AppHandle) -> AppResult<PathBuf> {
    let path = base_dir(handle)?.join(RUNS_DIR);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn run_dir(handle: &AppHandle, run_id: &str) -> AppResult<PathBuf> {
    let path = runs_dir(handle)?.join(safe_run_id(run_id).map_err(domain_error_to_app)?);
    fs::create_dir_all(&path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(path)
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
