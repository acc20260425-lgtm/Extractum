import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const read = (relativePath: string) => {
  const absolute = path.join(repoRoot, relativePath);
  return existsSync(absolute)
    ? readFileSync(absolute, "utf8").replace(/\r\n/g, "\n")
    : "";
};
const rustSources = (relativeDir: string): string[] => {
  const absolute = path.join(repoRoot, relativeDir);
  if (!existsSync(absolute)) return [];
  return readdirSync(absolute, { withFileTypes: true }).flatMap((entry) => {
    const child = path.join(relativeDir, entry.name).replaceAll("\\", "/");
    return entry.isDirectory()
      ? rustSources(child)
      : entry.isFile() && entry.name.endsWith(".rs")
        ? [read(child)]
        : [];
  });
};
const tomlSection = (source: string, heading: string) => {
  const marker = `[${heading}]`;
  const start = source.indexOf(marker);
  if (start < 0) return "";
  const bodyStart = start + marker.length;
  const next = source.slice(bodyStart).search(/^\[\[?[^\n]+\]?\]$/m);
  return source.slice(bodyStart, next < 0 ? undefined : bodyStart + next).trim();
};
const dependencyNames = (section: string) =>
  [...section.matchAll(/^([A-Za-z0-9_-]+)(?:\.workspace)?\s*=/gm)]
    .map((match) => match[1])
    .sort();
const lockPackage = (source: string, name: string) =>
  source
    .split(/(?=^\[\[package\]\]$)/m)
    .find((block) => block.includes(`\nname = "${name}"\n`)) ?? "";
const lockDependencies = (block: string) => {
  const match = block.match(/^dependencies = \[\n([\s\S]*?)^\]$/m);
  return match
    ? [...match[1].matchAll(/^ "([^"]+)",?$/gm)]
        .map((entry) => entry[1])
        .sort()
    : [];
};
const rustTestPattern =
  /#\[(?:tokio::)?test(?:\([^\]]*\))?\]\s*(?:#\[[^\]]+\]\s*)*(?:async\s+)?fn\s+([A-Za-z0-9_]+)/g;
const testNames = (sources: string[]) =>
  sources.flatMap((source) =>
    [...source.matchAll(new RegExp(rustTestPattern.source, "g"))].map(
      (match) => match[1],
    ),
  );

const crateOwnedBaselineTests = [
  "launch_spec_uses_endpoint_port_and_dedicated_profile",
  "launch_spec_rejects_remote_cdp_endpoint",
  "provider_status_uses_cached_snapshot_when_sidecar_is_busy",
  "provider_status_live_probe_does_not_mutate_cached_snapshot",
  "status_snapshot_core_returns_cached_status_without_polling_live_sidecar",
  "provider_status_snapshot_from_reconciled_runs_does_not_keep_stale_running_snapshot",
  "provider_status_snapshot_from_reconciled_runs_preserves_live_active_run",
  "provider_status_snapshot_from_reconciled_runs_ignores_stale_queued_rows",
  "provider_status_snapshot_read_core_writes_reconciled_snapshot_back",
  "provider_status_snapshot_read_core_skips_stale_write_back_when_snapshot_changed",
  "get_run_core_returns_exact_run_from_log",
  "provider_status_read_core_waits_for_startup_reconciliation_before_live_status",
  "send_single_prompt_handoff_writes_run_log_before_enqueue",
  "send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue",
  "send_single_prompt_rejects_duplicate_terminal_run_id_before_enqueue",
  "send_single_prompt_rejects_duplicate_waiter_before_enqueue",
  "send_single_prompt_marks_run_failed_when_enqueue_fails",
  "send_single_prompt_rejects_invalid_artifact_mode_before_side_effects",
  "failed_run_log_transition_returns_app_error_without_side_effects",
  "gemini_browser_job_serializes_queue_payload",
  "restart_reconciliation_degraded_leaves_queued_run_log_records",
  "restart_reconciliation_matrix_handles_supported_apalis_states",
  "restart_worker_entry_skips_terminal_cancelled_run_log",
  "restart_worker_entry_acknowledges_missing_run_log_without_sidecar",
  "degraded_apalis_queue_inspection_leaves_queued_run_log_records_for_worker_entry",
  "worker_status_blocks_enqueue_when_startup_failed",
  "worker_status_allows_enqueue_after_ready",
  "worker_status_times_out_while_starting",
  "waiter_receives_terminal_worker_result",
  "wait_for_result_removes_waiter_on_timeout",
  "wait_for_result_removes_waiter_when_worker_channel_closes",
  "register_waiter_rejects_duplicate_run_id",
  "complete_waiter_ignores_dropped_receiver",
  "runtime_tracks_and_clears_cancelled_run_ids",
  "worker_handler_marks_run_running_and_terminal",
  "worker_handler_converts_executor_error_to_terminal_failed_result",
  "cancel_gemini_browser_job_cancels_queued_run_and_waiter",
  "cancel_missing_run_returns_without_run_log_side_effects",
  "cancel_queued_run_updates_terminal_snapshot",
  "cancel_gemini_browser_job_requests_stop_for_active_run",
  "worker_startup_failure_marks_runtime_failed",
  "worker_run_failure_marks_runtime_failed",
  "worker_timeout_marks_run_failed_and_processes_next_job",
  "worker_timeout_clears_active_and_cancelled_state",
  "run_log_persists_queued_running_and_terminal_result",
  "read_run_returns_exact_run_by_id",
  "read_run_returns_validation_error_for_missing_run",
  "recorded_run_dir_requires_result_artifact_flag_and_returns_computed_dir",
  "list_runs_deletes_run_directories_outside_retention_window",
  "create_queued_run_prunes_expired_runs_before_writing_new_run",
  "recorded_run_dir_prunes_expired_run_before_opening_artifacts",
  "decode_sidecar_line_rejects_mismatched_ids",
  "decode_sidecar_line_accepts_ack_for_matching_id",
  "decode_sidecar_line_for_request_skips_stale_response_ids",
  "take_complete_jsonl_lines_handles_partial_and_multiple_chunks",
  "jsonl_transport_round_trips_a_duplex_request",
  "resume_response_classifies_legacy_ack_for_retry",
  "resolve_launch_mode_prefers_bundled_when_forced",
  "resolve_launch_mode_keeps_dev_node_fallback_for_debug_repo_runs",
  "resolve_launch_mode_uses_bundled_by_default_for_release_even_when_repo_dist_exists",
  "resolve_launch_mode_allows_explicit_dev_sidecar_override_in_release",
  "resolve_launch_mode_falls_back_to_bundled_when_debug_dev_script_is_absent",
  "bundled_sidecar_path_is_beside_the_packaged_executable",
  "state_tracks_active_run_and_cancellation",
  "status_snapshot_initializes_to_not_started_from_profile_dir",
  "update_status_snapshot_mutates_cached_status",
  "startup_reconciliation_gate_runs_once_after_success",
  "startup_reconciliation_gate_retries_after_failure",
  "set_status_snapshot_if_current_does_not_overwrite_newer_snapshot",
  "success_statuses_include_ready_and_ok",
  "sidecar_command_serializes_with_snake_case_tag",
  "manual_action_serializes_start_chrome_cdp",
  "resume_command_serializes_browser_profile_dir",
  "sidecar_command_serializes_browser_config",
  "run_result_serializes_optional_debug_summary",
] as const;

const appOwnedBaselineTests = [
  "explicit_shutdown_kills_and_reaps_the_owned_child_once",
  "drop_falls_back_to_owned_child_shutdown",
  "shutdown_does_not_claim_or_kill_an_already_exited_child",
  "shutdown_reaps_when_the_child_has_already_exited_during_kill",
  "wait_for_cdp_endpoint_accepts_json_version_response",
  "wait_for_cdp_endpoint_reports_unreachable_endpoint",
  "stderr_drain_consumes_sidecar_output_concurrently",
  "cancelled_run_marks_the_sidecar_transport_tainted",
  "apalis_storage_uses_shared_main_extractum_db_identity",
  "apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job",
  "apalis_storage_preserves_existing_sqlx_migration_history_table",
  "apalis_storage_shares_extractum_db_without_locking_app_pool",
  "enqueue_duplicate_run_id_returns_conflict",
  "enqueue_persists_job_before_worker_startup",
  "worker_picks_up_job_quickly_after_idle",
  "restart_worker_processes_pending_job_after_runtime_restart",
  "apalis_sqlite_status_probe_documents_actual_status_values",
  "gemini_browser_jobs_are_built_with_one_total_attempt",
  "failed_gemini_browser_job_is_not_retried",
] as const;

const curatedRoot = `mod cdp;
mod error;
mod execution;
mod executor;
mod protocol;
mod reconciliation;
mod run_id;
mod run_log;
mod runtime;
mod sidecar_launch;
mod state;
mod status;
mod submission;
mod types;

pub use cdp::{build_chrome_cdp_launch_spec, start_chrome_result, ChromeCdpLaunchSpec};
pub use error::{GeminiBrowserError, GeminiBrowserErrorKind, GeminiBrowserResult};
pub use execution::{
    cancel_run, execute_delivered_job, CancelRunOutcome, DeliveredJobInput, DeliveryOutcome,
};
pub use executor::{
    BrowserExecutor, BrowserExecutorFuture, BrowserRunContext, BrowserSessionContext,
    BrowserStopReason,
};
pub use protocol::{classify_resume_response, GeminiBrowserJsonlCodec, ResumeSidecarOutcome};
pub use reconciliation::{
    ensure_startup_reconciled, reconcile_startup, NormalizedQueueState, QueueInspectionSnapshot,
    ReconciliationAction, StartupReconciliationSnapshot,
};
pub use run_id::safe_run_id;
pub use run_log::{
    create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
};
pub use runtime::{
    run_registered_worker, GeminiBrowserArtifactMode, GeminiBrowserJob, GeminiBrowserJobRuntime,
};
pub use sidecar_launch::{
    bundled_sidecar_path, dev_sidecar_script, resolve_launch_mode, GeminiBrowserBuildProfile,
    GeminiBrowserSidecarLaunch, GEMINI_BROWSER_SIDECAR_NAME,
};
pub use state::GeminiBrowserDomainState;
pub use status::{
    open_provider, read_provider_status, read_reconciled_status_snapshot, resume_provider,
    StatusObserver,
};
pub use submission::{submit_and_wait, QueuedGeminiBrowserJob};
pub use types::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserAnswerExtractionDebug,
    GeminiBrowserAnswerGrouping, GeminiBrowserArtifactRefs, GeminiBrowserCandidateRejectReason,
    GeminiBrowserDebugErrorStage, GeminiBrowserManualAction, GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRun, GeminiBrowserRunDebugSummary, GeminiBrowserRunLogSummary,
    GeminiBrowserRunRequest, GeminiBrowserRunResult, GeminiBrowserRunStatus,
    GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope, GeminiBrowserSidecarResponse,
    GeminiBrowserStartChromeResult,
};
`;

const appFacade = `mod cdp_chrome;
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
`;

const rootCargo = read("src-tauri/Cargo.toml");
const crateCargo = read("src-tauri/crates/extractum-gemini-browser/Cargo.toml");
const cargoLock = read("src-tauri/Cargo.lock");
const crateLib = read("src-tauri/crates/extractum-gemini-browser/src/lib.rs");
const facade = read("src-tauri/src/gemini_browser/mod.rs");
const crateRust = rustSources("src-tauri/crates/extractum-gemini-browser/src");
const appRust = rustSources("src-tauri/src/gemini_browser");

describe("Gemini Browser crate boundary", () => {
  it("declares one app-to-domain edge and locked package", () => {
    expect(tomlSection(rootCargo, "workspace")).toContain(
      'members = [".", "crates/extractum-core", "crates/extractum-gemini-browser", "crates/extractum-llm"]',
    );
    expect(tomlSection(rootCargo, "dependencies")).toContain(
      'extractum-gemini-browser = { path = "crates/extractum-gemini-browser" }',
    );
    expect(tomlSection(rootCargo, "dependencies").match(/^extractum-gemini-browser\s*=/gm)).toHaveLength(1);
    expect(lockDependencies(lockPackage(cargoLock, "extractum-gemini-browser"))).toEqual(
      ["parking_lot", "serde", "serde_json", "tempfile", "time", "tokio", "tokio-util", "url"].sort(),
    );
    expect(lockDependencies(lockPackage(cargoLock, "extractum"))).toContain("extractum-gemini-browser");
  });

  it("keeps the exact portable dependency and feature allowlist", () => {
    expect(dependencyNames(tomlSection(rootCargo, "workspace.dependencies"))).toEqual(
      ["parking_lot", "reqwest", "secrecy", "serde", "serde_json", "tempfile", "time", "tokio", "tokio-util", "url", "zstd"].sort(),
    );
    expect(tomlSection(rootCargo, "workspace.dependencies")).toBe([
      'parking_lot = "0.12"',
      'reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }',
      'secrecy = "0.8"',
      'serde = { version = "1", features = ["derive"] }',
      'serde_json = "1"',
      'tempfile = "3"',
      'time = { version = "0.3", features = ["formatting", "parsing", "macros"] }',
      'tokio = "1"',
      'tokio-util = "0.7"',
      'url = "2"',
      'zstd = "0.13"',
    ].join("\n"));
    for (const inherited of ["parking_lot", "reqwest", "secrecy", "tokio-util", "url", "tempfile"]) {
      expect(tomlSection(rootCargo, "dependencies")).toMatch(new RegExp(`^${inherited}\\s*=\\s*\\{ workspace = true`, "m"));
    }
    expect(tomlSection(rootCargo, "dependencies")).toContain('tokio = { workspace = true, features = ["full"] }');
    expect(tomlSection(rootCargo, "dev-dependencies")).toContain('tokio = { workspace = true, features = ["test-util"] }');
    expect(dependencyNames(tomlSection(crateCargo, "dependencies"))).toEqual(
      ["parking_lot", "serde", "serde_json", "time", "tokio", "tokio-util", "url"].sort(),
    );
    expect(dependencyNames(tomlSection(crateCargo, "dev-dependencies"))).toEqual(["tempfile", "tokio"]);
    expect(tomlSection(crateCargo, "dependencies")).toContain('tokio = { workspace = true, features = ["macros", "sync", "time"] }');
    expect(tomlSection(crateCargo, "dev-dependencies")).toContain('tokio = { workspace = true, features = ["rt", "test-util"] }');
    expect(crateCargo).not.toContain("extractum-core");
    expect(crateCargo).not.toMatch(/^\[target\.|^\[profile\./m);
  });

  it("keeps a curated crate root and explicit private app facade", () => {
    expect(crateLib).toBe(curatedRoot);
    expect(facade).toBe(appFacade);
    expect(crateLib).not.toMatch(/pub\s+(?:use\s+[^;]*\*|mod\s+)/);
  });

  it("moves every frozen baseline test exactly once", () => {
    expect(new Set(crateOwnedBaselineTests).size).toBe(75);
    expect(new Set(appOwnedBaselineTests).size).toBe(19);
    const frozen = new Set([...crateOwnedBaselineTests, ...appOwnedBaselineTests]);
    expect(frozen.size).toBe(94);
    const all = [...testNames(crateRust), ...testNames(appRust)].filter((name) => frozen.has(name as never));
    for (const name of frozen) expect(all.filter((actual) => actual === name)).toHaveLength(1);
    expect(testNames(crateRust).filter((name) => frozen.has(name as never)).sort()).toEqual([...crateOwnedBaselineTests].sort());
    expect(testNames(appRust).filter((name) => frozen.has(name as never)).sort()).toEqual([...appOwnedBaselineTests].sort());
  });

  it("does not retain compile-disabled legacy domain copies app-side", () => {
    const appSource = appRust.join("\n");
    expect(appSource).not.toMatch(/#\s*\[\s*cfg\s*\(\s*any\s*\(\s*\)\s*\)\s*\]/);
    expect(appSource).not.toMatch(/\blegacy_disabled_[A-Za-z0-9_]+\b/);
  });

  it("keeps process Tauri SQL Apalis and worker infrastructure app-side", () => {
    const crateSource = crateRust.join("\n");
    const appSource = appRust.join("\n");
    expect(crateSource).not.toMatch(/(?:tauri|sqlx|apalis(?:_sqlite)?|tower|reqwest|windows_sys)::|AppHandle|AppError|AppResult|ProcessTreeGuard|std\.process|tokio::process|\bChild(?:Stdin|Stdout|Stderr)?\b|\bCommand\b|\bAsyncRead\b|\bAsyncWrite\b/);
    expect(crateSource).not.toMatch(/extractum_process|external_process|child_process|process_tree|\bJobs\b|"Pending"|"Killed"/);
    expect(appSource).toMatch(/tauri::|AppHandle/);
    expect(appSource).toMatch(/sqlx::/);
    expect(appSource).toMatch(/apalis/);
    expect(appSource).toMatch(/ProcessTreeGuard/);
  });

  it("keeps lifecycle transitions and cancellation ownership domain-side", () => {
    const crateSource = crateRust.join("\n");
    const appSource = appRust.join("\n");
    expect(appSource).not.toMatch(/CancellationToken|is_worker_timeout_result/);
    expect(crateSource).toContain("CancellationToken");
    for (const helper of ["new_for_test", "new_for_test_with_timeouts", "new_for_waiter_timeout_test", "has_waiter_for_test", "worker_status_for_test"]) {
      expect(crateSource).not.toMatch(new RegExp(`\\bpub\\s+(?:async\\s+)?fn\\s+${helper}\\b`));
    }
    expect(crateSource).not.toMatch(/\bpub\s+(?:struct|enum|fn)\s+(?:BlockingExecutor|RecordingStatusObserver|FakeExecutor|FakeObserver)\b/);
  });

  it("removes string timeout classification while preserving legacy output tests", () => {
    const crateSource = crateRust.join("\n");
    const appSource = appRust.join("\n");
    expect(`${appSource}\n${crateSource}`).not.toMatch(/(?:starts_with|strip_prefix|contains|ends_with)\s*\(\s*"Gemini Browser job timed out after /);
    for (const exactText of ["Gemini Browser job timed out waiting for worker result", "Gemini Browser job timed out after ", "Cancelled", "wait_for_result_removes_waiter_on_timeout", "worker_timeout_marks_run_failed_and_processes_next_job", "cancel_gemini_browser_job_cancels_queued_run_and_waiter", "cancel_gemini_browser_job_requests_stop_for_active_run"]) {
      expect(crateSource).toContain(exactText);
    }
    expect(testNames(appRust)).toContain("gemini_browser_error_maps_to_exact_legacy_app_error_json");
    expect(appSource).toContain("Gemini Browser job timed out waiting for worker result");
    expect(appSource).toMatch(/fn gemini_browser_error_maps_to_exact_legacy_app_error_json[\s\S]{0,2500}serde_json::to_string/);
  });
});
