import { describe, expect, it } from "vitest";
import libSource from "../../src-tauri/src/lib.rs?raw";
import coordinatorSource from "../../src-tauri/crates/extractum-process/src/external_process.rs?raw";
import processTreeSource from "../../src-tauri/crates/extractum-process/src/process_tree.rs?raw";
import sidecarSource from "../../src-tauri/src/gemini_browser/sidecar.rs?raw";
import sidecarLaunchSource from "../../src-tauri/src/gemini_browser/sidecar_launch.rs?raw";
import cdpChromeSource from "../../src-tauri/src/gemini_browser/cdp_chrome.rs?raw";
import geminiCommandsSource from "../../src-tauri/src/gemini_browser/commands.rs?raw";
import youtubeProcessRuntimeSource from "../../src-tauri/src/youtube/process_runtime.rs?raw";
import cargoSource from "../../src-tauri/Cargo.toml?raw";
import tauriConfigSource from "../../src-tauri/tauri.conf.json?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

describe("external process lifecycle contract", () => {
  it("registers a runtime-only shutdown coordinator with explicit timing budgets", () => {
    const lib = normalized(libSource);
    const coordinator = normalized(coordinatorSource);

    expect(lib).toMatch(
      /mod\s+external_process\s*\{[\s\S]*pub\(crate\)\s+use\s+extractum_process::external_process::\*;/,
    );
    expect(lib).toContain(".manage(ExternalProcessShutdownState::new())");
    expect(coordinator).toContain("Running");
    expect(coordinator).toContain("ShuttingDown");
    expect(coordinator).toContain("Completed");
    expect(coordinator).toContain("GRACEFUL_SHUTDOWN_TIMEOUT");
    expect(coordinator).toContain("SHUTDOWN_WATCHDOG_TIMEOUT");
    expect(coordinator).not.toMatch(/serde|Serialize|Deserialize|tauri::command/i);
  });

  it("contains Windows child trees using owned child handles", () => {
    const processTree = normalized(processTreeSource);

    expect(processTree).toContain("JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE");
    expect(processTree).toContain("AsRawHandle");
    expect(processTree).toContain("AssignProcessToJobObject");
    expect(processTree).not.toContain("OpenProcess");
    expect(processTree).not.toMatch(/\.pid\s*\(/);
  });

  it("owns the Gemini sidecar with Tokio rather than the shell plugin", () => {
    const sidecar = normalized(sidecarSource);
    const launch = normalized(sidecarLaunchSource);
    const lib = normalized(libSource);

    expect(sidecar).toContain("tokio::process::Command");
    expect(sidecar).toContain("hide_console_window");
    expect(launch).toContain("current_exe");
    expect(tauriConfigSource).toContain("gemini-browser-sidecar");
    expect(sidecar).not.toContain("CommandChild");
    expect(sidecar).not.toContain("CommandEvent");
    expect(sidecar).not.toContain("request_shell");
    expect(lib).not.toContain("tauri_plugin_shell");
    expect(normalized(cargoSource)).not.toContain("tauri-plugin-shell");

    const admission = sidecar.indexOf(".try_admit()");
    const launchDispatch = sidecar.indexOf("match resolve_launch_mode(");
    expect(admission).toBeGreaterThanOrEqual(0);
    expect(admission).toBeLessThan(launchDispatch);

    const commandConfigurator = sidecar.match(
      /fn configure_sidecar_command[\s\S]*?^}/m,
    )?.[0];
    expect(commandConfigurator).toContain("hide_console_window");
    expect(commandConfigurator).toContain("kill_on_drop(true)");
    expect(sidecar.match(/configure_sidecar_command\(&mut command\)/g)).toHaveLength(2);
  });

  it("owns CDP Chrome through the spawned child and its process tree", () => {
    const cdpChrome = normalized(cdpChromeSource);
    const commands = normalized(geminiCommandsSource);

    expect(cdpChrome).toContain("ProcessTreeGuard");
    expect(cdpChrome).toContain("assign_std");
    expect(cdpChrome).toContain("fn shutdown");
    expect(commands).toContain("spawn_blocking");
    expect(cdpChrome).not.toMatch(/taskkill|CreateToolhelp32Snapshot|Process32First|Process32Next|sysinfo/);
    expect(cdpChrome.indexOf("ProcessTreeGuard::new()"))
      .toBeLessThan(cdpChrome.indexOf(".spawn()"));
  });

  it("delegates Tauri exit decisions to the shutdown coordinator", () => {
    const lib = normalized(libSource);
    const coordinator = normalized(coordinatorSource);

    expect(lib).toContain("RunEvent::ExitRequested");
    expect(lib).toMatch(
      /ShutdownStart::Started\(run\)\s*=>\s*\{[\s\S]*?api\.prevent_exit\(\)[\s\S]*?run\.coordinate/,
    );
    expect(lib).toMatch(
      /ShutdownStart::AlreadyShuttingDown\s*=>\s*api\.prevent_exit\(\)/,
    );
    expect(lib).toMatch(/ShutdownStart::Completed\s*=>\s*\{\}/);
    expect(lib).not.toContain("GRACEFUL_SHUTDOWN_TIMEOUT");
    expect(lib).not.toContain("SHUTDOWN_WATCHDOG_TIMEOUT");
    expect(lib).not.toContain("shutdown.wait_for_startups()");
    expect(lib).not.toContain("shutdown.complete()");
    expect(coordinator).not.toContain("fn begin_shutdown(");
    expect(coordinator).not.toMatch(/pub\(crate\) fn complete\(/);

    const warningHelper = coordinator.match(
      /fn warn_shutdown_stage\(operation_id: u64, stage: &'static str\)[\s\S]*?^}/m,
    )?.[0];
    expect(warningHelper).toBeDefined();
    expect(warningHelper).not.toMatch(
      /args|cookie|prompt|stdout|stderr|profile|executable|path|error/i,
    );
    const coordinatorWarning = coordinator.match(
      /fn warn_shutdown_coordinator_stage\(stage: &'static str\)[\s\S]*?^}/m,
    )?.[0];
    expect(coordinatorWarning).toBeDefined();
    expect(coordinatorWarning).not.toMatch(
      /operation_id|args|cookie|prompt|stdout|stderr|profile|executable|path|error/i,
    );
    const shutdownRunImpl = coordinator.match(
      /impl ShutdownRun \{[\s\S]*?^}/m,
    )?.[0];
    expect(shutdownRunImpl).toBeDefined();
    expect(shutdownRunImpl).toContain("warn_shutdown_coordinator_stage(");
    expect(shutdownRunImpl).not.toContain("warn_shutdown_stage(");
    expect(normalized(youtubeProcessRuntimeSource)).toContain(
      "warn_shutdown_stage(operation_id, \"yt_dlp_reap_detached\")",
    );
  });
});
