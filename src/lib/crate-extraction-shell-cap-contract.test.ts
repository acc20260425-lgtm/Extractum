import { describe, expect, it } from "vitest";

import focusedLoopDesignRaw from "../../docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md?raw";
import crateRoadmapRaw from "../../docs/superpowers/specs/2026-07-17-crate-roadmap.md?raw";
import processBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md?raw";
import geminiBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-19-gemini-browser-crate-boundary-design.md?raw";
import llmBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-20-llm-crate-boundary-design.md?raw";
import promptPacksBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-20-prompt-packs-crate-boundary-design.md?raw";
import llmVerificationRaw from "../../docs/superpowers/verification/2026-07-20-extractum-llm-extraction.md?raw";
import shellCapRevisionRaw from "../../docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md?raw";
import anomalyV2DesignRaw from "../../docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md?raw";
import reapplicationPlanRaw from "../../docs/superpowers/plans/2026-07-18-extractum-process-reapplication.md?raw";
import cancellationDispositionRaw from "../../docs/superpowers/verification/2026-07-19-extractum-process-reapplication-cancellation.md?raw";

const normalize = (value: string) => value.replace(/\r\n/g, "\n");
const compact = (value: string) =>
  normalize(value).replace(/\n>\s?/g, "\n").replace(/\s+/g, " ");
const sectionBetween = (value: string, start: string, end: string) => {
  const startIndex = value.indexOf(start);
  const endIndex = value.indexOf(end, startIndex + start.length);
  if (startIndex < 0 || endIndex < 0) {
    throw new Error(`Missing policy section: ${start} -> ${end}`);
  }
  return value.slice(startIndex, endIndex);
};

const focusedLoopDesign = normalize(focusedLoopDesignRaw);
const crateRoadmap = normalize(crateRoadmapRaw);
const processBoundaryDesign = compact(processBoundaryDesignRaw);
const geminiBoundaryDesign = compact(geminiBoundaryDesignRaw);
const llmBoundaryDesign = compact(llmBoundaryDesignRaw);
const promptPacksBoundaryDesign = compact(promptPacksBoundaryDesignRaw);
const llmVerification = compact(llmVerificationRaw);
const shellCapRevision = compact(shellCapRevisionRaw);
const anomalyV2Design = compact(anomalyV2DesignRaw);
const reapplicationPlan = compact(reapplicationPlanRaw);
const cancellationDisposition = compact(cancellationDispositionRaw);
const samplingPolicy = compact(
  sectionBetween(
    focusedLoopDesign,
    "### Sampling",
    "### Advisory interpretation",
  ),
);
const advisoryPolicy = compact(
  sectionBetween(
    focusedLoopDesign,
    "### Advisory interpretation",
    "## Failure Classification",
  ),
);
const failurePolicy = compact(
  sectionBetween(
    focusedLoopDesign,
    "## Failure Classification",
    "## Repository Enforcement",
  ),
);
const roadmapTiming = compact(
  sectionBetween(
    crateRoadmap,
    "## Roadmap Timing Signals",
    "## Target Crate Map",
  ),
);
const phase3Roadmap = compact(
  sectionBetween(crateRoadmap, "### Phase 3 —", "### Phase 4 —"),
);
const phase4Roadmap = compact(
  sectionBetween(crateRoadmap, "### Phase 4 —", "### Phase 5 —"),
);
const phase5Roadmap = compact(
  sectionBetween(crateRoadmap, "### Phase 5 —", "### Phase 6 —"),
);
const phase6Roadmap = compact(
  sectionBetween(crateRoadmap, "### Phase 6", "### Phase 7"),
);
const appOwnedGeminiBaselineTests = [
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
];

describe("crate extraction timing policy", () => {
  it("keeps focused timing small and advisory", () => {
    expect(focusedLoopDesign).toContain(
      "**Status:** Approved; timing policy simplified 2026-07-19",
    );
    expect(focusedLoopDesign).toContain(
      "[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)",
    );
    expect(samplingPolicy).toContain("one discarded warm-up");
    expect(samplingPolicy).toContain("three recorded samples");
    expect(samplingPolicy).toContain("raw values and median of three");
    expect(samplingPolicy).toContain("probe restoration in a `finally` path");
    expect(samplingPolicy).toContain("one SHA-256 source check");
    expect(samplingPolicy).toContain("one clean-worktree check");
    expect(samplingPolicy).toContain("no separate application-shell A/B series");
    expect(samplingPolicy).toContain(
      "Record the duration emitted by the mandatory end-of-slice workspace check",
    );
    expect(samplingPolicy).toContain("Do not add an active-process scanner");
    expect(samplingPolicy).not.toContain("five recorded samples");
    expect(samplingPolicy).not.toContain("300 ms");
    expect(samplingPolicy).toContain("quiet-window coordinator");
    expect(advisoryPolicy).toContain(
      "do not automatically retain, reject, or revert a correct slice",
    );
    expect(advisoryPolicy).toContain(
      "historical 25% / 2.0-second focused gate, 2,000 ms / 20% shell cap, and cumulative ledger are no longer active policy",
    );
    expect(advisoryPolicy).toContain(
      "one completed crate-extraction slice contributes one ordinary workspace result",
    );
    expect(advisoryPolicy).toContain(
      "Two consecutive completed crate-extraction slices whose ordinary workspace results are each at or above 15,000 ms trigger a separate owner-approved performance investigation",
    );
    expect(advisoryPolicy).toContain(
      "successful mandatory end-of-slice `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`",
    );
    expect(advisoryPolicy).toContain(
      "A completed result below 15,000 ms breaks the sequence; failed, canceled, and incomplete slices contribute no result",
    );
    expect(advisoryPolicy).toContain(
      "historical measurements do not seed the sequence",
    );
    expect(advisoryPolicy).toContain(
      "Focused checks, tests, diagnostics, and same-slice reruns do not count",
    );
    expect(advisoryPolicy).toContain(
      "Do not rerun the check or add timing samples for this rule",
    );
    expect(roadmapTiming).toContain(
      "Two consecutive completed crate-extraction slices whose ordinary workspace results are each at or above 15,000 ms trigger a separate owner-approved performance investigation",
    );
    expect(roadmapTiming).toContain(
      "Consecutive means adjacent completed extraction slices in roadmap order",
    );
    expect(failurePolicy).toContain("There is no protocol-mandated retry");
    expect(failurePolicy).toContain(
      "Timing alone cannot reject or revert the slice",
    );
    expect(focusedLoopDesign).not.toContain("### Retention gates");
  });

  it("records the canceled Phase 3 and approved independent Phase 4 boundary", () => {
    expect(crateRoadmap).toContain(
      "**Status:** Strategic reference; revised and owner-approved 2026-07-19",
    );
    expect(crateRoadmap).not.toContain("Implementation is pending.");
    expect(roadmapTiming).toContain("There is no cumulative shell ledger");
    expect(roadmapTiming).toContain(
      "| Historical Phase 3 candidate | 10,177 ms | candidate reverted and not retained |",
    );
    expect(roadmapTiming).not.toContain("Reapplied Phase 3");
    expect(roadmapTiming).toContain(
      "record the duration of the mandatory workspace check",
    );
    expect(phase3Roadmap).toContain(
      "Phase 3 — `extractum-process` (closed: not retained)",
    );
    expect(phase3Roadmap).toContain(
      "The first attempt stopped before candidate replay",
    );
    expect(phase3Roadmap).toContain(
      "The corrected second attempt created an exact but unmerged candidate commit",
    );
    expect(phase3Roadmap).toContain(
      "No process crate, post-reapplication baseline, or cumulative-ledger entry exists",
    );
    expect(phase3Roadmap).toContain(
      "the replay and measurement machinery had grown beyond the value of the decision",
    );
    expect(phase3Roadmap).toContain("This records no broader owner intent");
    expect(phase3Roadmap).toContain(
      "2026-07-19-extractum-process-reapplication-cancellation.md",
    );
    expect(phase3Roadmap).toContain(
      "Any future `extractum-process` attempt starts as a new phase",
    );
    expect(phase4Roadmap).toContain(
      "Phase 4 — `extractum-gemini-browser` (done: retained)",
    );
    expect(phase4Roadmap).toContain(
      "2026-07-19-gemini-browser-crate-boundary-design.md",
    );
    expect(phase4Roadmap).toContain(
      "2026-07-19-extractum-gemini-browser-extraction.md",
    );
    expect(phase4Roadmap).toContain(
      "28 of 39 (71.8%) touched no other categorized Rust domain",
    );
    expect(phase4Roadmap).toContain(
      "all concrete sidecar/CDP spawn, handles, containment, kill/reap, and shutdown remain in `extractum`",
    );
    expect(phase4Roadmap).toContain(
      "A permanent domain-level `BrowserExecutor`",
    );
    expect(phase4Roadmap).toContain("It does not recreate `extractum-process`");
    expect(phase4Roadmap).toContain(
      "It has no Phase 3 timing or reapplication prerequisite",
    );
    expect(geminiBoundaryDesign).toContain(
      "**Status:** Implemented and retained; [verification](../verification/2026-07-19-extractum-gemini-browser-extraction.md)",
    );
    expect(geminiBoundaryDesign).toContain(
      "2026-07-19-extractum-gemini-browser-extraction.md",
    );
    expect(geminiBoundaryDesign).toContain(
      "supersedes only the Phase 4 architecture, dependency, measurement, and execution clauses",
    );
    expect(geminiBoundaryDesign).toContain(
      "There is no dependency from `extractum-gemini-browser` back to `extractum` and no dependency on `extractum-process`",
    );
    expect(geminiBoundaryDesign).toContain(
      "PID values, `Child`, `Command`, stdin/stdout handles, `ProcessTreeGuard`, shutdown-admission types, `windows-sys`, and process-tree operations never cross the new crate's public API",
    );
    expect(geminiBoundaryDesign).toContain(
      "The approved disposition is 75 tests in `extractum-gemini-browser` and 19 in `extractum`",
    );
    expect(appOwnedGeminiBaselineTests).toHaveLength(19);
    for (const testName of appOwnedGeminiBaselineTests) {
      expect(geminiBoundaryDesign).toContain(`\`${testName}\``);
    }
    expect(geminiBoundaryDesign).toContain(
      "use a frozen set of all 94 baseline names",
    );
    expect(geminiBoundaryDesign).toContain(
      "The crate does not pass its `CancellationToken` into `BrowserExecutor`",
    );
    expect(geminiBoundaryDesign).toContain(
      "A response that completes after cancellation is ignored",
    );
    for (const legacyMessage of [
      "Gemini Browser job timed out waiting for worker result",
      "Gemini Browser job timed out after {seconds}s",
      "Cancelled",
    ]) {
      expect(geminiBoundaryDesign).toContain(`\`"${legacyMessage}"\``);
    }
    expect(geminiBoundaryDesign).toContain(
      "persisted pretty-JSON run-log bytes for both timeout paths and for queued and active cancellation",
    );
    expect(geminiBoundaryDesign).toContain(
      "inventory `types.rs`, `run_log.rs`, and every moved fragment for direct or facade-backed `extractum-core` API use",
    );
    expect(geminiBoundaryDesign).toContain(
      "one discarded warm-up and three recorded samples",
    );
    expect(geminiBoundaryDesign).toContain(
      "timing alone never rejects, reverts, or retains the slice",
    );
    expect(geminiBoundaryDesign).toContain(
      "Two adjacent completed crate-extraction slices whose ordinary workspace-check results are each at or above 15,000 ms trigger a separate owner-approved performance investigation",
    );
    expect(geminiBoundaryDesign).toContain(
      "must not build a new quiet-window, Job Object, or process-scanning measurement harness",
    );
    expect(geminiBoundaryDesign).toContain(
      "npm.cmd run smoke:gemini-browser-sidecar:binary",
    );
    expect(geminiBoundaryDesign).toContain(
      "npm.cmd run smoke:gemini-browser-sidecar:resume:node -- --expect-manual-action=start_chrome_cdp",
    );
    expect(geminiBoundaryDesign).not.toContain("applicable process-smoke");
    expect(shellCapRevision).toContain(
      "Superseded 2026-07-19; historical policy record",
    );
    expect(shellCapRevision).toContain(
      "must not be used as current execution authority",
    );
    expect(shellCapRevision).toContain(
      "canceled before completion and never retained",
    );
    expect(processBoundaryDesign).toContain(
      "execution authority withdrawn 2026-07-19",
    );
    expect(processBoundaryDesign).toContain("not authority to replay");
    expect(processBoundaryDesign).toContain("Phase 3 or implement Phase 4");
    expect(processBoundaryDesign).toContain(
      "2026-07-19-gemini-browser-crate-boundary-design.md",
    );
    expect(processBoundaryDesign).toContain(
      "canceled before completion and never retained",
    );
    expect(reapplicationPlan).toContain(
      "CANCELED 2026-07-19 — DO NOT EXECUTE OR RESUME",
    );
    expect(reapplicationPlan).toContain(
      "The first attempt stopped before candidate replay",
    );
    expect(reapplicationPlan).toContain(
      "A corrected second attempt reached an exact, isolated candidate replay",
    );
    expect(reapplicationPlan).toContain(
      "the workflow did not complete and the replay was not merged",
    );
    expect(reapplicationPlan).toContain(
      "2026-07-19-extractum-process-reapplication-cancellation.md",
    );
    expect(reapplicationPlan).not.toContain(
      "withdrew the complete plan before any task was executed",
    );
    expect(phase3Roadmap).not.toMatch(/canceled.{0,80}before execution/);
    expect(shellCapRevision).not.toContain(
      "reapplication was canceled before execution",
    );
    expect(processBoundaryDesign).not.toContain(
      "reapplication plan was canceled before execution",
    );
    expect(cancellationDisposition).toContain(
      "18 and then 16 idle `@hypothesi/tauri-mcp-server` processes",
    );
    expect(cancellationDisposition).toContain(
      "No candidate path was changed in that first attempt",
    );
    expect(cancellationDisposition).toContain(
      "`f9274194111977b4cb722937bde62bf5f2bc6be2`",
    );
    expect(cancellationDisposition).toContain(
      "`49b596d3e21cfc8f07904caf97a9673d4b6418e0`",
    );
    expect(cancellationDisposition).toContain(
      "`6c431a54aef00c1e2f2f9be6693f7660f942fedf`",
    );
    expect(cancellationDisposition).toContain(
      "matches historical candidate `b364756c`",
    );
    expect(cancellationDisposition).toContain(
      "canonical no-renames stable patch ID",
    );
    expect(cancellationDisposition).toContain(
      "`fb767db0e8d2a9c6e743da4446b1f4da2c43f775`",
    );
    for (const correctionCommit of [
      "791912785d1e62179a93658c3e72e16895c36439",
      "0f4b040a5e45a0dc50be1378ac15b1e1fc6b32f3",
      "4a2bb11ea0a351754f6c56a1ee5f0329b9ef40e0",
      "9bcd2cfea6ad961eae2f6437fa2c59b161b89e23",
    ]) {
      expect(cancellationDisposition).toContain(`\`${correctionCommit}\``);
    }
    expect(cancellationDisposition).toContain(
      "extractum-process-reapplication-20260719T141033776-f11b55c13fae45c8a20c5ad35d927d8a",
    );
    expect(cancellationDisposition).toContain(
      "extractum-process-reapplication-20260719T152723364-1fb2e3afe159491bbe23ee5b13c34e7c",
    );
    expect(cancellationDisposition).toContain(
      "never merged into `main`",
    );
    expect(anomalyV2Design).toContain("`moot` for the current crate roadmap");
  });

  it("records retained Phase 5 ownership, advisory timing, and Phase 6 authorization", () => {
    expect(llmBoundaryDesign).toContain(
      "**Status:** Implemented and retained; [verification](../verification/2026-07-20-extractum-llm-extraction.md)",
    );
    expect(phase5Roadmap).toContain(
      "Phase 5 — `extractum-llm` (done: retained)",
    );
    expect(phase5Roadmap).toContain("2026-07-20-extractum-llm-extraction.md");
    for (const dependency of [
      "extractum-core",
      "reqwest",
      "secrecy",
      "serde",
      "serde_json",
      "tokio",
      "tokio-util",
    ]) {
      expect(phase5Roadmap).toContain(`\`${dependency}\``);
    }
    expect(phase5Roadmap).toContain("owned exactly 36/15 by crate/app");
    expect(phase5Roadmap).toContain("one-shot focused timing series were incomplete");
    expect(phase5Roadmap).toContain("there is no median and no performance conclusion");
    expect(phase5Roadmap).toContain("Timing was advisory and did not decide retention");
    expect(phase5Roadmap).toContain("10,410 ms, below 15,000 ms");
    expect(phase5Roadmap).toContain("Phase 4's 1,620 ms result");
    expect(phase5Roadmap).toContain("Phase 6 `extractum-prompt-packs` remains next");
    expect(phase5Roadmap).toContain("fresh JIT boundary design is now owner-approved");
    expect(phase5Roadmap).toContain("implementation is not authorized");
    expect(llmVerification).toContain("BASELINE_RAW_MS=[]");
    expect(llmVerification).toContain("CANDIDATE_RAW_MS=[]");
    expect(llmVerification).toContain("no median / no performance conclusion");
    expect(llmVerification).toContain(
      "Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.41s",
    );
    expect(llmVerification).toContain("The mechanical result is 10,410 ms");
    expect(llmVerification).toContain("Timing did not decide retention");
    for (const disallowed of [
      "shell A/B",
      "quiet-window coordinator",
      "active-process scanner",
      "Job Object",
      "protocol-mandated retry",
      "cumulative ledger",
    ]) {
      expect(llmVerification).not.toContain(disallowed);
      expect(phase5Roadmap).not.toContain(disallowed);
    }
  });

  it("records the approved Phase 6 prompt-pack boundary without authorizing implementation", () => {
    expect(promptPacksBoundaryDesign).toContain(
      "**Status:** Owner-approved; implementation not started",
    );
    expect(phase6Roadmap).toContain(
      "design approved; implementation not started",
    );
    expect(phase6Roadmap).toContain(
      "2026-07-20-prompt-packs-crate-boundary-design.md",
    );
    expect(phase6Roadmap).toContain(
      "46 files / 19,037 lines and 225 baseline Rust test identities",
    );
    expect(phase6Roadmap).toContain("118 commits touched `prompt_packs`");
    expect(phase6Roadmap).toContain("92 (78.0%)");
    expect(phase6Roadmap).toContain(
      "223 identities in the new crate and two foreign-source SQL-adapter identities in the app",
    );
    expect(phase6Roadmap).toContain("32 prompt-pack-owned tables");
    expect(phase6Roadmap).toContain(
      "The app retains Tauri commands/events/spawning, `get_pool`, migrations, profile/secret resolution, foreign source reads, and concrete Gemini Browser operations",
    );
    expect(phase6Roadmap).toContain(
      "Implementation requires a separate plan and an explicit owner instruction",
    );
    expect(phase6Roadmap).toContain(
      "one warm-up plus three focused samples per state",
    );
    expect(phase6Roadmap).toContain(
      "no quiet-window, retry, shell A/B, or cumulative-ledger machinery",
    );

    expect(promptPacksBoundaryDesign).toContain(
      "The approved final partition is 223 identities in `extractum-prompt-packs` and two in `extractum`",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "transcript_text_for_source_uses_segment_renderer",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "comment_snapshot_selection_is_deterministic_when_enabled",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "There is no production prompt-pack query of `projects`",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "standing source-contract assertion, not a one-time planning check",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "initial `vec![...]` prefix of `build_migrations()`",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "requires exact ordered equality",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "Any added, removed, reordered, or newly registered non-Apalis migration therefore fails the contract",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "Checkpoints 1–4 each end in a separately identifiable green commit",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "The slice may stop after any completed green checkpoint and retain the independently useful preparation",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "revert only that unique RED commit and retain Checkpoints 1–4",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "decide each earlier green preparation checkpoint independently",
    );
    expect(promptPacksBoundaryDesign).toContain(
      "Timing remains deliberately small and cannot veto a correct extraction",
    );
  });
});
