mod cdp_chrome;
mod commands;
mod executor;
mod jobs;
mod paths;
mod sidecar;
mod state;

pub(crate) use cdp_chrome::shutdown_cdp_chrome;
pub use commands::{
    gemini_bridge_get_run, gemini_bridge_list_runs, gemini_bridge_open_browser,
    gemini_bridge_open_run_folder, gemini_bridge_resume, gemini_bridge_send_single,
    gemini_bridge_start_cdp_chrome, gemini_bridge_status, gemini_bridge_status_snapshot,
    gemini_bridge_stop,
};
pub(crate) use commands::{provider_status, send_single_prompt};
pub(crate) use jobs::{cancel_gemini_browser_job, start_gemini_browser_job_worker};
#[cfg(test)]
pub(crate) use jobs::{
    enqueue_gemini_browser_job_to_storage, open_gemini_browser_job_storage,
    setup_gemini_browser_apalis_storage,
};
pub(crate) use paths::{chrome_cdp_profile_dir, path_string, profile_dir, run_dir, runs_dir};
#[cfg(test)]
pub(crate) use sidecar::configure_sidecar_command;
pub(crate) use sidecar::shutdown_sidecar;
pub use state::GeminiBrowserState;

pub(crate) use extractum_gemini_browser::{
    create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
    GeminiBrowserJobRuntime,
};
pub use extractum_gemini_browser::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
    GeminiBrowserSidecarResponse, GeminiBrowserStartChromeResult,
};
#[cfg(test)]
pub(crate) use extractum_gemini_browser::{GeminiBrowserArtifactMode, GeminiBrowserJob};
#[cfg(test)]
pub(crate) use extractum_gemini_browser::{
    GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
};

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[test]
    fn cdp_child_guard_reaps_the_complete_owned_tree() {
        use super::cdp_chrome::{ChromeCdpProcess, SystemChromeChild};
        use crate::process_tree::ProcessTreeGuard;
        use std::{
            fs,
            io::{BufRead, BufReader},
            process::{Command, Stdio},
            thread,
            time::{Duration, SystemTime, UNIX_EPOCH},
        };

        let signal = std::env::temp_dir().join(format!(
            "extractum-cdp-tree-{}-{}.signal",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let script = concat!(
            "while (-not (Test-Path -LiteralPath $env:EXTRACTUM_CDP_TREE_SIGNAL)) ",
            "{ Start-Sleep -Milliseconds 10 }; ",
            "$descendant = Start-Process -FilePath powershell.exe ",
            "-ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 30' -PassThru; ",
            "Write-Output $descendant.Id; Start-Sleep -Seconds 30"
        );
        let mut child = Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", script])
            .env("EXTRACTUM_CDP_TREE_SIGNAL", &signal)
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn CDP ownership fixture");
        let process_tree = ProcessTreeGuard::new().expect("create CDP job object");
        process_tree
            .assign_std(&child)
            .expect("assign CDP child to job");
        fs::write(&signal, []).expect("signal CDP descendant creation");
        let descendant_pid = {
            let stdout = child.stdout.take().expect("CDP fixture stdout");
            let mut line = String::new();
            BufReader::new(stdout)
                .read_line(&mut line)
                .expect("read CDP descendant pid");
            line.trim()
                .parse::<u32>()
                .expect("parse CDP descendant pid")
        };
        let mut process = ChromeCdpProcess::new(Box::new(SystemChromeChild {
            child,
            process_tree,
        }));

        process.shutdown().expect("shut down complete CDP tree");
        process.shutdown().expect("CDP shutdown remains idempotent");
        let _ = fs::remove_file(&signal);

        for _ in 0..30 {
            let status = Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "if (Get-Process -Id {descendant_pid} -ErrorAction SilentlyContinue) {{ exit 1 }}"
                    ),
                ])
                .status()
                .expect("query CDP descendant");
            if status.success() {
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
        panic!("CDP descendant survived owned-tree shutdown");
    }

    #[cfg(not(windows))]
    #[test]
    fn cdp_child_guard_reaps_the_complete_owned_tree() {
        panic!("Windows-only CDP ownership contract requires Windows");
    }
}
