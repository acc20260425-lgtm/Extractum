use std::path::PathBuf;
use std::process::Command;

use url::Url;

use crate::error::{AppError, AppResult};

use super::{
    path_string, GeminiBrowserProviderConfig, GeminiBrowserProviderMode,
    GeminiBrowserStartChromeResult,
};

const DEFAULT_CDP_ENDPOINT: &str = "http://127.0.0.1:9222";
const GEMINI_URL: &str = "https://gemini.google.com/app";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChromeCdpLaunchSpec {
    pub chrome_path: PathBuf,
    pub args: Vec<String>,
    pub browser_profile_dir: String,
    pub cdp_endpoint: String,
}

pub(crate) fn find_chrome_executable() -> PathBuf {
    candidate_chrome_paths()
        .into_iter()
        .find(|candidate| candidate.exists())
        .unwrap_or_else(|| {
            PathBuf::from(if cfg!(windows) {
                "chrome.exe"
            } else {
                "google-chrome"
            })
        })
}

fn candidate_chrome_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if cfg!(windows) {
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            candidates.push(
                PathBuf::from(program_files)
                    .join("Google")
                    .join("Chrome")
                    .join("Application")
                    .join("chrome.exe"),
            );
        }
        if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            candidates.push(
                PathBuf::from(program_files_x86)
                    .join("Google")
                    .join("Chrome")
                    .join("Application")
                    .join("chrome.exe"),
            );
        }
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            candidates.push(
                PathBuf::from(local_app_data)
                    .join("Google")
                    .join("Chrome")
                    .join("Application")
                    .join("chrome.exe"),
            );
        }
    }
    candidates
}

pub(crate) fn build_chrome_cdp_launch_spec(
    chrome_path: PathBuf,
    browser_profile_dir: PathBuf,
    config: Option<&GeminiBrowserProviderConfig>,
) -> AppResult<ChromeCdpLaunchSpec> {
    let endpoint = resolve_cdp_endpoint(config)?;
    let port = cdp_port(&endpoint)?;
    let browser_profile_dir = path_string(&browser_profile_dir);
    Ok(ChromeCdpLaunchSpec {
        chrome_path,
        args: vec![
            format!("--remote-debugging-port={port}"),
            format!("--user-data-dir={browser_profile_dir}"),
            GEMINI_URL.to_string(),
        ],
        browser_profile_dir,
        cdp_endpoint: endpoint,
    })
}

pub(crate) fn spawn_chrome_cdp(spec: &ChromeCdpLaunchSpec) -> AppResult<()> {
    Command::new(&spec.chrome_path)
        .args(&spec.args)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            AppError::internal(format!(
                "Failed to start Chrome with remote debugging enabled: {error}"
            ))
        })
}

pub(crate) fn start_chrome_result(spec: &ChromeCdpLaunchSpec) -> GeminiBrowserStartChromeResult {
    GeminiBrowserStartChromeResult {
        browser_profile_dir: spec.browser_profile_dir.clone(),
        cdp_endpoint: spec.cdp_endpoint.clone(),
        message: "Chrome was started with remote debugging enabled.".to_string(),
    }
}

fn resolve_cdp_endpoint(config: Option<&GeminiBrowserProviderConfig>) -> AppResult<String> {
    let raw = match config {
        Some(GeminiBrowserProviderConfig {
            mode: GeminiBrowserProviderMode::CdpAttach,
            cdp_endpoint,
        }) => cdp_endpoint
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(DEFAULT_CDP_ENDPOINT),
        _ => DEFAULT_CDP_ENDPOINT,
    };

    let url = Url::parse(raw.trim())
        .map_err(|_| AppError::validation("Chrome CDP endpoint must be a loopback HTTP URL."))?;
    if url.scheme() != "http" {
        return Err(AppError::validation("Chrome CDP endpoint must use http."));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::validation(
            "Chrome CDP endpoint must not contain credentials.",
        ));
    }
    let host = url
        .host_str()
        .ok_or_else(|| AppError::validation("Chrome CDP endpoint must include a host."))?;
    if !matches!(host, "127.0.0.1" | "localhost" | "::1") {
        return Err(AppError::validation(
            "Chrome CDP endpoint must be a loopback HTTP URL.",
        ));
    }
    let port = url
        .port()
        .ok_or_else(|| AppError::validation("Chrome CDP endpoint must include a non-zero port."))?;
    if port == 0 {
        return Err(AppError::validation(
            "Chrome CDP endpoint must include a non-zero port.",
        ));
    }
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err(AppError::validation(
            "Chrome CDP endpoint must be a base URL without path, query, or hash.",
        ));
    }

    let normalized_host = if host == "::1" { "[::1]" } else { host };
    Ok(format!("http://{normalized_host}:{port}"))
}

fn cdp_port(endpoint: &str) -> AppResult<u16> {
    Url::parse(endpoint)
        .ok()
        .and_then(|url| url.port())
        .ok_or_else(|| AppError::validation("Chrome CDP endpoint must include a non-zero port."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn launch_spec_uses_endpoint_port_and_dedicated_profile() {
        let config = GeminiBrowserProviderConfig {
            mode: GeminiBrowserProviderMode::CdpAttach,
            cdp_endpoint: Some("http://127.0.0.1:9333".to_string()),
        };

        let spec = build_chrome_cdp_launch_spec(
            PathBuf::from("C:/Chrome/chrome.exe"),
            PathBuf::from("C:/Extractum/gemini-browser/chrome-cdp-profile"),
            Some(&config),
        )
        .expect("build launch spec");

        assert_eq!(spec.cdp_endpoint, "http://127.0.0.1:9333");
        assert_eq!(
            spec.browser_profile_dir,
            "C:/Extractum/gemini-browser/chrome-cdp-profile"
        );
        assert_eq!(spec.chrome_path, PathBuf::from("C:/Chrome/chrome.exe"));
        assert!(spec
            .args
            .contains(&"--remote-debugging-port=9333".to_string()));
        assert!(spec.args.contains(
            &"--user-data-dir=C:/Extractum/gemini-browser/chrome-cdp-profile".to_string()
        ));
        assert!(spec
            .args
            .contains(&"https://gemini.google.com/app".to_string()));
    }

    #[test]
    fn launch_spec_rejects_remote_cdp_endpoint() {
        let config = GeminiBrowserProviderConfig {
            mode: GeminiBrowserProviderMode::CdpAttach,
            cdp_endpoint: Some("http://192.168.1.20:9222".to_string()),
        };

        let error = build_chrome_cdp_launch_spec(
            PathBuf::from("C:/Chrome/chrome.exe"),
            PathBuf::from("C:/Extractum/gemini-browser/chrome-cdp-profile"),
            Some(&config),
        )
        .unwrap_err();

        assert!(error.to_string().contains("loopback"));
    }
}
