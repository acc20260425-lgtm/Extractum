import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repoRoot = process.cwd();
const readSource = (relativePath: string) =>
  readFileSync(path.join(repoRoot, relativePath), "utf8").replace(/\r\n/g, "\n");
const readOptionalSource = (relativePath: string) =>
  existsSync(path.join(repoRoot, relativePath)) ? readSource(relativePath) : "";

const rootCargo = readSource("src-tauri/Cargo.toml");
const rootLib = readSource("src-tauri/src/lib.rs");
const processCargo = readOptionalSource("src-tauri/crates/extractum-process/Cargo.toml");
const processLib = readOptionalSource("src-tauri/crates/extractum-process/src/lib.rs");
const externalProcess = readOptionalSource("src-tauri/crates/extractum-process/src/external_process.rs");
const childProcess = readOptionalSource("src-tauri/crates/extractum-process/src/child_process.rs");
const processTree = readOptionalSource("src-tauri/crates/extractum-process/src/process_tree.rs");
const oldImplementations = [
  readOptionalSource("src-tauri/src/external_process.rs"),
  readOptionalSource("src-tauri/src/child_process.rs"),
  readOptionalSource("src-tauri/src/process_tree.rs"),
].join("");

const publicNames = (source: string) =>
  Array.from(new Set(Array.from(source.matchAll(/^\s*pub\s+(?:async\s+)?(?:type|struct|enum|fn|const)\s+([A-Za-z_]\w*)/gm), (match) => match[1]))).sort();

const processTests = [
  "create_no_window_matches_win32_process_creation_flags",
  "admission_wait_consumes_the_shared_graceful_budget",
  "cleanup_tasks_start_concurrently_and_isolate_error_and_panic",
  "concurrent_watchdogs_invoke_exit_once",
  "exhausted_admission_budget_skips_the_cleanup_factory",
  "injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback",
  "permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown",
  "permits_acquired_before_shutdown_are_waited_for",
  "repeated_start_does_not_replace_code_or_schedule_again",
  "start_reports_completed_after_watchdog_claims_exit",
  "start_returns_started_and_schedules_one_watchdog",
  "timing_exposes_the_graceful_and_watchdog_budgets",
  "watchdog_exits_with_the_preserved_code_unless_cleanup_completed",
  "assigns_a_directly_owned_std_child",
  "creates_a_job_object",
  "dropping_the_guard_closes_the_job_and_kills_its_children",
  "process_tree_guard_can_be_owned_by_async_application_state",
  "terminate_failure_remains_reportable_and_retryable",
  "terminate_is_idempotent",
  "terminates_a_descendant_created_after_assignment",
];

describe("extractum process crate boundary", () => {
  it("defines one minimal workspace process package", () => {
    expect(rootCargo).toMatch(/members\s*=\s*\[[\s\S]*"\."[\s\S]*"crates\/extractum-core"[\s\S]*"crates\/extractum-process"[\s\S]*\]/);
    expect(processCargo).toBe([
      "[package]", 'name = "extractum-process"', "version.workspace = true",
      "edition.workspace = true", "publish = false", "", "[dependencies]",
      "anyhow.workspace = true", "parking_lot.workspace = true", "tokio.workspace = true", "",
      "[target.'cfg(windows)'.dependencies]", "windows-sys.workspace = true", "",
      "[dev-dependencies]", 'tokio = { workspace = true, features = ["test-util"] }', "",
    ].join("\n"));
    expect(processLib).toBe(["pub mod child_process;", "pub mod external_process;", "pub mod process_tree;", ""].join("\n"));
    expect(processLib).not.toMatch(/pub\s+use\s+[^;]*\*/);
  });

  it("keeps dependency roots exact and application concerns out", () => {
    for (const dependency of ["anyhow", "parking_lot", "tokio", "windows-sys"]) {
      expect(rootCargo).toMatch(new RegExp(`\\[workspace\\.dependencies\\][\\s\\S]*${dependency}`));
    }
    const processSource = [externalProcess, childProcess, processTree].join("\n");
    for (const forbidden of ["tauri", "sqlx", "apalis", "extractum_core", "gemini_browser", "youtube", "job_helpers"]) {
      expect(processCargo).not.toContain(forbidden);
      expect(processSource).not.toContain(forbidden);
    }
  });

  it("exposes only the reviewed cross-crate API", () => {
    expect(publicNames(externalProcess)).toEqual([
      "AdmissionPermit", "AdmissionRejected", "CleanupFactory", "ExitCallback",
      "ExternalProcessShutdownState", "MonotonicClock", "ShutdownCleanup",
      "ShutdownCleanupError", "ShutdownRun", "ShutdownStart", "ShutdownTiming",
      "WatchdogScheduler", "WatchdogTask", "coordinate", "new",
      "os_thread_watchdog_scheduler", "start", "system_monotonic_clock", "try_admit",
      "warn_shutdown_stage",
    ].sort());
    expect(externalProcess).toMatch(/pub\s+graceful:\s*Duration/);
    expect(externalProcess).toMatch(/pub\s+watchdog:\s*Duration/);
    expect(publicNames(childProcess)).toEqual(["CREATE_NO_WINDOW", "hide_console_window"].sort());
    expect(publicNames(processTree)).toEqual(["ProcessTreeGuard", "assign_std", "assign_tokio", "new", "terminate"].sort());
    expect([externalProcess, childProcess, processTree].join("\n")).not.toContain("pub(crate)");
  });

  it("preserves private app-side glob facades and existing consumer paths", () => {
    for (const moduleName of ["external_process", "child_process", "process_tree"]) {
      expect(rootLib).not.toContain(`mod ${moduleName};`);
      expect(rootLib).toMatch(new RegExp(`mod\\s+${moduleName}\\s*\\{[\\s\\S]*?pub\\(crate\\)\\s+use\\s+extractum_process::${moduleName}::\\*;[\\s\\S]*?\\}`));
    }
    expect(readSource("src-tauri/src/youtube/process_runtime.rs")).toContain("crate::external_process");
    expect(readSource("src-tauri/src/youtube/process_runtime.rs")).toContain("crate::process_tree");
    expect(readSource("src-tauri/src/diagnostics/runtime.rs")).toContain("crate::child_process");
    expect(readSource("src-tauri/src/gemini_browser/sidecar.rs")).toContain("crate::child_process::hide_console_window");
  });

  it("moves implementations and all twenty tests instead of copying them", () => {
    expect(oldImplementations).toBe("");
    const movedSource = [externalProcess, childProcess, processTree].join("\n");
    for (const testName of processTests) {
      expect(movedSource).toContain(`fn ${testName}(`);
      expect(oldImplementations).not.toContain(`fn ${testName}(`);
    }
  });
});
