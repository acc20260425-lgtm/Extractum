use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::time::Duration;

use url::Url;

use crate::error::{AppError, AppResult};
use crate::process_tree::ProcessTreeGuard;

use super::{
    path_string, GeminiBrowserProviderConfig, GeminiBrowserProviderMode,
    GeminiBrowserStartChromeResult,
};

const DEFAULT_CDP_ENDPOINT: &str = "http://127.0.0.1:9222";
const GEMINI_URL: &str = "https://gemini.google.com/app";
const CDP_READY_TIMEOUT: Duration = Duration::from_secs(10);
const CDP_READY_POLL_INTERVAL: Duration = Duration::from_millis(200);

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

trait ChromeChild: Send {
    fn kill(&mut self) -> std::io::Result<()>;
    fn wait(&mut self) -> std::io::Result<std::process::ExitStatus>;
    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>>;
}

struct SystemChromeChild {
    child: Child,
    process_tree: ProcessTreeGuard,
}

impl ChromeChild for SystemChromeChild {
    fn kill(&mut self) -> std::io::Result<()> {
        let _ = self.process_tree.terminate();
        self.child.kill()
    }

    fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait()
    }

    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }
}

pub(crate) struct ChromeCdpProcess {
    child: Box<dyn ChromeChild>,
    shut_down: bool,
}

impl ChromeCdpProcess {
    fn new(child: Box<dyn ChromeChild>) -> Self {
        Self {
            child,
            shut_down: false,
        }
    }

    pub(crate) fn shutdown(&mut self) -> std::io::Result<()> {
        if self.shut_down {
            return Ok(());
        }
        self.shut_down = true;
        if self.child.try_wait()?.is_none() {
            let kill_result = self.child.kill();
            let wait_result = self.child.wait();
            if let Err(error) = kill_result {
                if error.kind() != std::io::ErrorKind::InvalidInput {
                    return Err(error);
                }
            }
            wait_result?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn with_test_child(child: Box<dyn ChromeChild>) -> Self {
        Self::new(child)
    }
}

impl Drop for ChromeCdpProcess {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

pub(crate) fn spawn_chrome_cdp(spec: &ChromeCdpLaunchSpec) -> AppResult<ChromeCdpProcess> {
    let mut child = Command::new(&spec.chrome_path)
        .args(&spec.args)
        .spawn()
        .map_err(|error| {
            AppError::internal(format!(
                "Failed to start Chrome with remote debugging enabled: {error}"
            ))
        })?;

    let process_tree = ProcessTreeGuard::new()
        .map_err(|_| AppError::internal("Failed to contain Chrome process tree"))?;
    if process_tree.assign_std(&child).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(AppError::internal("Failed to contain Chrome process tree"));
    }

    Ok(ChromeCdpProcess::new(Box::new(SystemChromeChild {
        child,
        process_tree,
    })))
}

pub(crate) async fn shutdown_cdp_chrome(state: &super::GeminiBrowserState) {
    let process = state.cdp_chrome_process().await.take();
    if let Some(mut process) = process {
        let _ = tokio::task::spawn_blocking(move || process.shutdown()).await;
    }
}

pub(crate) async fn wait_for_cdp_endpoint(endpoint: &str) -> AppResult<()> {
    wait_for_cdp_endpoint_core(endpoint, CDP_READY_TIMEOUT, CDP_READY_POLL_INTERVAL).await
}

async fn wait_for_cdp_endpoint_core(
    endpoint: &str,
    timeout: Duration,
    poll_interval: Duration,
) -> AppResult<()> {
    let version_url = format!("{}/json/version", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(750))
        .build()
        .map_err(|error| {
            AppError::internal(format!("Failed to build CDP probe client: {error}"))
        })?;
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        let probe_error = match client.get(&version_url).send().await {
            Ok(response) if response.status().is_success() => return Ok(()),
            Ok(response) => format!("HTTP {}", response.status()),
            Err(error) => error.to_string(),
        };

        if tokio::time::Instant::now() >= deadline {
            return Err(AppError::internal(format!(
                "Chrome was started but CDP endpoint did not become reachable at {endpoint}. Last probe error: {probe_error}"
            )));
        }

        tokio::time::sleep(poll_interval).await;
    }
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
    use std::sync::{Arc, Mutex};

    struct FakeChromeChild {
        events: Arc<Mutex<Vec<&'static str>>>,
        exited: bool,
        kill_error: Option<std::io::ErrorKind>,
    }

    impl ChromeChild for FakeChromeChild {
        fn kill(&mut self) -> std::io::Result<()> {
            self.events.lock().expect("events lock").push("kill");
            match self.kill_error {
                Some(kind) => Err(std::io::Error::from(kind)),
                None => Ok(()),
            }
        }

        fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
            self.events.lock().expect("events lock").push("wait");
            Ok(success_exit_status())
        }

        fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
            Ok(self.exited.then(success_exit_status))
        }
    }

    #[test]
    fn explicit_shutdown_kills_and_reaps_the_owned_child_once() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let child = FakeChromeChild { events: events.clone(), exited: false, kill_error: None };
        let mut process = ChromeCdpProcess::with_test_child(Box::new(child));

        process.shutdown().expect("first shutdown");
        process.shutdown().expect("second shutdown is idempotent");

        assert_eq!(*events.lock().expect("events lock"), ["kill", "wait"]);
    }

    #[test]
    fn drop_falls_back_to_owned_child_shutdown() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let child = FakeChromeChild { events: events.clone(), exited: false, kill_error: None };

        drop(ChromeCdpProcess::with_test_child(Box::new(child)));

        assert_eq!(*events.lock().expect("events lock"), ["kill", "wait"]);
    }

    #[test]
    fn shutdown_does_not_claim_or_kill_an_already_exited_child() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let child = FakeChromeChild { events: events.clone(), exited: true, kill_error: None };
        let mut process = ChromeCdpProcess::with_test_child(Box::new(child));

        process.shutdown().expect("shutdown observes child exit");

        assert!(events.lock().expect("events lock").is_empty());
    }

    #[test]
    fn shutdown_reaps_when_the_child_has_already_exited_during_kill() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let child = FakeChromeChild {
            events: events.clone(),
            exited: false,
            kill_error: Some(std::io::ErrorKind::InvalidInput),
        };
        let mut process = ChromeCdpProcess::with_test_child(Box::new(child));

        process.shutdown().expect("already-exited child remains a successful shutdown");

        assert_eq!(*events.lock().expect("events lock"), ["kill", "wait"]);
    }

    #[cfg(windows)]
    fn success_exit_status() -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }

    #[cfg(not(windows))]
    fn success_exit_status() -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }

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

    #[tokio::test]
    async fn wait_for_cdp_endpoint_accepts_json_version_response() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test cdp endpoint");
        let addr = listener.local_addr().expect("read listener address");
        let server = tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            let (mut stream, _) = listener.accept().await.expect("accept cdp probe");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await.expect("read cdp probe");
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"Browser\":\"test\"}",
                )
                .await
                .expect("write cdp response");
        });

        wait_for_cdp_endpoint_core(
            &format!("http://{addr}"),
            std::time::Duration::from_secs(1),
            std::time::Duration::from_millis(10),
        )
        .await
        .expect("cdp endpoint becomes ready");
        server.await.expect("server task joins");
    }

    #[tokio::test]
    async fn wait_for_cdp_endpoint_reports_unreachable_endpoint() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind unused cdp endpoint");
        let addr = listener.local_addr().expect("read listener address");
        drop(listener);

        let error = wait_for_cdp_endpoint_core(
            &format!("http://{addr}"),
            std::time::Duration::from_millis(25),
            std::time::Duration::from_millis(5),
        )
        .await
        .expect_err("unreachable cdp endpoint fails");

        assert!(error
            .to_string()
            .contains("CDP endpoint did not become reachable"));
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
