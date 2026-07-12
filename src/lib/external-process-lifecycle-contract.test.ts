import { describe, expect, it } from "vitest";
import libSource from "../../src-tauri/src/lib.rs?raw";
import coordinatorSource from "../../src-tauri/src/external_process.rs?raw";
import processTreeSource from "../../src-tauri/src/process_tree.rs?raw";
import sidecarSource from "../../src-tauri/src/gemini_browser/sidecar.rs?raw";
import sidecarLaunchSource from "../../src-tauri/src/gemini_browser/sidecar_launch.rs?raw";
import cdpChromeSource from "../../src-tauri/src/gemini_browser/cdp_chrome.rs?raw";
import geminiCommandsSource from "../../src-tauri/src/gemini_browser/commands.rs?raw";
import cargoSource from "../../src-tauri/Cargo.toml?raw";
import tauriConfigSource from "../../src-tauri/tauri.conf.json?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

describe("external process lifecycle contract", () => {
  it("registers a runtime-only shutdown coordinator with explicit timing budgets", () => {
    const lib = normalized(libSource);
    const coordinator = normalized(coordinatorSource);

    expect(lib).toContain("mod external_process;");
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
  });

  it("owns CDP Chrome through the spawned child and its process tree", () => {
    const cdpChrome = normalized(cdpChromeSource);
    const commands = normalized(geminiCommandsSource);

    expect(cdpChrome).toContain("ProcessTreeGuard");
    expect(cdpChrome).toContain("assign_std");
    expect(cdpChrome).toContain("fn shutdown");
    expect(commands).toContain("spawn_blocking");
    expect(cdpChrome).not.toMatch(/taskkill|CreateToolhelp32Snapshot|Process32First|Process32Next|sysinfo/);
  });

  it("coordinates external-process cleanup from the Tauri exit event", () => {
    const lib = normalized(libSource);

    expect(lib).toContain("RunEvent::ExitRequested");
    expect(lib).toContain("prevent_exit");
    expect(lib).toContain("GRACEFUL_SHUTDOWN_TIMEOUT");
    expect(lib).toContain("SHUTDOWN_WATCHDOG_TIMEOUT");
    expect(lib).toContain("std::thread::spawn");
  });
});
