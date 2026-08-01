import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { generateFeatureBaseline } from "../../scripts/telegram-grammers-feature-baseline.mjs";

import focusedLoopDesignRaw from "../../docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md?raw";
import crateRoadmapRaw from "../../docs/superpowers/specs/2026-07-17-crate-roadmap.md?raw";
import processBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md?raw";
import geminiBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-19-gemini-browser-crate-boundary-design.md?raw";
import llmBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-20-llm-crate-boundary-design.md?raw";
import promptPacksBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-20-prompt-packs-crate-boundary-design.md?raw";
import analysisBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-22-analysis-crate-boundary-design.md?raw";
import telegramBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md?raw";
import analysisExtractionPlanRaw from "../../docs/superpowers/plans/2026-07-22-extractum-analysis-extraction.md?raw";
import analysisExtractionVerificationRaw from "../../docs/superpowers/verification/2026-07-22-extractum-analysis-extraction.md?raw";
import llmVerificationRaw from "../../docs/superpowers/verification/2026-07-20-extractum-llm-extraction.md?raw";
import promptPacksVerificationRaw from "../../docs/superpowers/verification/2026-07-20-extractum-prompt-packs-extraction.md?raw";
import shellCapRevisionRaw from "../../docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md?raw";
import anomalyV2DesignRaw from "../../docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md?raw";
import reapplicationPlanRaw from "../../docs/superpowers/plans/2026-07-18-extractum-process-reapplication.md?raw";
import cancellationDispositionRaw from "../../docs/superpowers/verification/2026-07-19-extractum-process-reapplication-cancellation.md?raw";

const normalize = (value: string) => value.replace(/\r\n/g, "\n");
const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const promptPackCrateManifest = path.join(
  repoRoot,
  "src-tauri/crates/extractum-prompt-packs/Cargo.toml",
);
const promptPackCrateExtracted = existsSync(promptPackCrateManifest);
const analysisCrateManifest = path.join(
  repoRoot,
  "src-tauri/crates/extractum-analysis/Cargo.toml",
);
const analysisCrateExtracted = existsSync(analysisCrateManifest);
const packageJson = JSON.parse(
  readFileSync(path.join(repoRoot, "package.json"), "utf8"),
) as { scripts: Record<string, string> };
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
const analysisBoundaryDesign = compact(analysisBoundaryDesignRaw);
const telegramBoundaryDesign = compact(telegramBoundaryDesignRaw);
const telegramTypePerimeter = compact(
  sectionBetween(
    normalize(telegramBoundaryDesignRaw),
    "## Transitive Type Perimeter",
    "## Three Separately Green Sub-Slices",
  ),
);
const telegramSubSlices = compact(
  sectionBetween(
    normalize(telegramBoundaryDesignRaw),
    "## Three Separately Green Sub-Slices",
    "## Public Rust API",
  ),
);
const telegramVisibilityAllowlist = compact(
  sectionBetween(
    normalize(telegramBoundaryDesignRaw),
    "### Public visibility allowlist",
    "The public API must not contain:",
  ),
);
const telegramManifestContract = compact(
  sectionBetween(
    normalize(telegramBoundaryDesignRaw),
    "## Manifest and Dependency Contract",
    "## 8B Staging Layout",
  ),
);
const telegramStagingContract = compact(
  sectionBetween(
    normalize(telegramBoundaryDesignRaw),
    "## 8B Staging Layout",
    "## Current-File Disposition",
  ),
);
const telegramDisposition = compact(
  sectionBetween(
    normalize(telegramBoundaryDesignRaw),
    "## Current-File Disposition",
    "## Test Ownership and Boundary Contracts",
  ),
);
const telegramTestOwnership = compact(
  sectionBetween(
    normalize(telegramBoundaryDesignRaw),
    "## Test Ownership and Boundary Contracts",
    "## Draft and Approval Synchronization",
  ),
);
const analysisExtractionPlan = compact(analysisExtractionPlanRaw);
const analysisStagingDisposition = compact(
  sectionBetween(
    normalize(analysisExtractionPlanRaw),
    "## Post-execution staging disposition",
    "## Frozen 143-Test Ownership",
  ),
);
const analysisExtractionVerification = compact(
  analysisExtractionVerificationRaw,
);
const llmVerification = compact(llmVerificationRaw);
const promptPacksVerification = compact(promptPacksVerificationRaw);
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
const phase6Status = phase6Roadmap.match(
  /### Phase 6 — `extractum-prompt-packs` \(([^)]+)\)/,
)?.[1];
const phase7Roadmap = compact(
  sectionBetween(crateRoadmap, "### Phase 7", "### Phase 8"),
);
const phase7Status = phase7Roadmap.match(
  /### Phase 7 — `extractum-analysis` \(([^)]+)\)/,
)?.[1];
const phase8Roadmap = compact(
  sectionBetween(crateRoadmap, "### Phase 8", "### Phase 9+"),
);
const phase8Status = phase8Roadmap.match(
  /### Phase 8 — `extractum-telegram` \(([^)]+)\)/,
)?.[1];
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

type FeatureMetadataFixture = {
  packages: Array<{
    id: string;
    name: string;
    source?: string;
    features: Record<string, string[] | null>;
  }>;
  resolve: {
    nodes: Array<{
      id: string;
      deps: Array<{ pkg: string }>;
      features: string[];
    }>;
  };
};

function featureMetadataFixture(): FeatureMetadataFixture {
  const revision = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa";
  const source =
    `git+https://codeberg.org/Lonami/grammers?rev=${revision}#${revision}`;
  const artifact = JSON.parse(
    readFileSync(
      path.join(
        repoRoot,
        "src/lib/telegram-grammers-feature-baseline.json",
      ),
      "utf8",
    ),
  ) as {
    packages: Array<{
      name: string;
      required: string[];
      universe: string[];
    }>;
  };
  const packageRecords = artifact.packages.map((packageRecord) => ({
    id: `${packageRecord.name}-fixture`,
    name: packageRecord.name,
    source,
    features: Object.fromEntries(
      packageRecord.universe.map((feature) => [feature, []]),
    ),
  }));
  return {
    packages: [
      {
        id: "extractum-fixture",
        name: "extractum",
        features: {},
      },
      ...packageRecords,
    ],
    resolve: {
      nodes: [
        {
          id: "extractum-fixture",
          deps: packageRecords.map(({ id }) => ({ pkg: id })),
          features: [],
        },
        ...packageRecords.map(({ id, name }) => ({
          id,
          deps: [],
          features:
            artifact.packages.find((candidate) => candidate.name === name)
              ?.required ?? [],
        })),
      ],
    },
  };
}

describe("crate extraction timing policy", () => {
  it("checks the generated Grammers feature baseline", () => {
    const artifactPath = path.join(
      repoRoot,
      "src/lib/telegram-grammers-feature-baseline.json",
    );
    const artifact = JSON.parse(readFileSync(artifactPath, "utf8")) as {
      schemaVersion: number;
      revision: string;
      packages: Array<{
        name: string;
        required: string[];
        forbidden: string[];
        universe: string[];
      }>;
    };
    expect(artifact).toEqual(generateFeatureBaseline());
    expect(artifact).toEqual({
      schemaVersion: 1,
      revision: "1f901ce6e973fdcf0e74267f3d8efad5c729daaa",
      packages: [
        {
          name: "grammers-client",
          required: [],
          forbidden: [
            "default",
            "fs",
            "html",
            "html5ever",
            "markdown",
            "parse_invite_link",
            "proxy",
            "pulldown-cmark",
            "url",
          ],
          universe: [
            "default",
            "fs",
            "html",
            "html5ever",
            "markdown",
            "parse_invite_link",
            "proxy",
            "pulldown-cmark",
            "url",
          ],
        },
        {
          name: "grammers-mtsender",
          required: [],
          forbidden: [
            "hickory-resolver",
            "proxy",
            "tokio-socks",
            "url",
          ],
          universe: [
            "hickory-resolver",
            "proxy",
            "tokio-socks",
            "url",
          ],
        },
        {
          name: "grammers-session",
          required: ["serde"],
          forbidden: ["default", "sqlite-storage"],
          universe: ["default", "serde", "sqlite-storage"],
        },
        {
          name: "grammers-tl-types",
          required: [
            "default",
            "deserializable-functions",
            "impl-debug",
            "impl-from-enum",
            "impl-from-type",
            "tl-api",
            "tl-mtproto",
          ],
          forbidden: ["impl-serde"],
          universe: [
            "default",
            "deserializable-functions",
            "impl-debug",
            "impl-from-enum",
            "impl-from-type",
            "impl-serde",
            "tl-api",
            "tl-mtproto",
          ],
        },
      ],
    });
    for (const packageRecord of artifact.packages) {
      expect(packageRecord.required).toEqual(
        [...packageRecord.required].sort(),
      );
      expect(packageRecord.forbidden).toEqual(
        [...packageRecord.forbidden].sort(),
      );
      expect(packageRecord.universe).toEqual(
        [...packageRecord.universe].sort(),
      );
      expect(
        packageRecord.universe.filter(
          (feature) => !packageRecord.required.includes(feature),
        ),
      ).toEqual(packageRecord.forbidden);
    }
    expect(() => generateFeatureBaseline({})).toThrow(
      /missing package graph/,
    );
    const check = spawnSync(
      process.execPath,
      [
        path.join(
          repoRoot,
          "scripts/telegram-grammers-feature-baseline.mjs",
        ),
        "--check",
      ],
      { cwd: repoRoot, encoding: "utf8", shell: false },
    );
    expect(check.status, `${check.stdout}${check.stderr}`).toBe(0);
    const unsupported = spawnSync(
      process.execPath,
      [
        path.join(
          repoRoot,
          "scripts/telegram-grammers-feature-baseline.mjs",
        ),
        "--unsupported",
      ],
      { cwd: repoRoot, encoding: "utf8", shell: false },
    );
    expect(unsupported.status).not.toBe(0);
    expect(`${unsupported.stdout}${unsupported.stderr}`).toContain(
      "expected exactly one argument: --write or --check",
    );
  }, 15_000);

  it("feature authority rejects duplicate extractum resolve nodes", () => {
    const metadata = featureMetadataFixture();
    const extractumNode = metadata.resolve.nodes[0];
    metadata.resolve.nodes.push({
      ...extractumNode,
      deps: [...extractumNode.deps],
      features: [...extractumNode.features],
    });
    expect(() => generateFeatureBaseline(metadata)).toThrow(
      /expected one resolved extractum node/,
    );
  });

  it("feature authority rejects non-array feature definitions", () => {
    const metadata = featureMetadataFixture();
    const client = metadata.packages.find(
      ({ name }) => name === "grammers-client",
    );
    if (!client) throw new Error("missing fixture package");
    client.features.default = null;
    expect(() => generateFeatureBaseline(metadata)).toThrow(
      /feature definition.*string array/,
    );
  });

  it("feature authority rejects a revision-matching wrong repository URL", () => {
    const metadata = featureMetadataFixture();
    const client = metadata.packages.find(
      ({ name }) => name === "grammers-client",
    );
    if (!client) throw new Error("missing fixture package");
    client.source =
      "git+https://evil.example/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa#1f901ce6e973fdcf0e74267f3d8efad5c729daaa";
    expect(() => generateFeatureBaseline(metadata)).toThrow(
      /source drifted/,
    );
  });

  it("tracks the exact Prompt Pack package owner through the extraction move", () => {
    const appRuntime = path.join(
      repoRoot,
      "src-tauri/src/prompt_packs/runtime.rs",
    );
    const crateRuntime = path.join(
      repoRoot,
      "src-tauri/crates/extractum-prompt-packs/src/runtime.rs",
    );

    expect(existsSync(appRuntime)).toBe(!promptPackCrateExtracted);
    expect(existsSync(crateRuntime)).toBe(promptPackCrateExtracted);
    expect(packageJson.scripts["test:rust:prompt-pack-runs"]).toBe(
      promptPackCrateExtracted
        ? "cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib prompt_pack_run"
        : "cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_pack_run",
    );
  });

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
    expect(phase5Roadmap).toContain(
      "At Phase 5 completion, Phase 6 `extractum-prompt-packs` remained next",
    );
    expect(phase5Roadmap).toContain("fresh JIT boundary design was owner-approved");
    expect(phase5Roadmap).toContain("implementation was not authorized");
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

  it("records the retained Phase 6 prompt-pack boundary and closed lifecycle state", () => {
    expect(promptPacksBoundaryDesign).toContain(
      "**Status:** Implemented and retained; [verification](../verification/2026-07-20-extractum-prompt-packs-extraction.md)",
    );
    expect(phase6Status).toBeDefined();
    expect([
      "design approved; implementation not started",
      "preparation Checkpoint 1 retained",
      "preparation Checkpoint 2 retained",
      "preparation Checkpoint 3 retained",
      "preparation Checkpoint 4 retained",
      "done: retained",
      "not retained",
    ]).toContain(phase6Status);
    expect(phase6Status).toBe("done: retained");
    expect(phase6Roadmap).toContain(
      "2026-07-20-prompt-packs-crate-boundary-design.md",
    );
    expect(phase6Roadmap).toContain(
      "2026-07-20-extractum-prompt-packs-extraction.md",
    );
    expect(phase6Roadmap).toContain(
      "46 files / 19,037 lines and 225 baseline Rust test identities",
    );
    expect(phase6Roadmap).toContain("118 commits touched `prompt_packs`");
    expect(phase6Roadmap).toContain("92 (78.0%)");
    expect(phase6Roadmap).toContain(
      "exactly 223 crate identities and two foreign-source SQL-adapter identities in the app",
    );
    expect(phase6Roadmap).toContain("32 prompt-pack-owned tables");
    expect(phase6Roadmap).toContain(
      "The app retains Tauri commands/events/spawning, `get_pool`, migrations, profile/secret resolution, foreign source reads, and concrete Gemini Browser operations",
    );
    expect(phase6Roadmap).toContain(
      "25 new characterization tests and the app Prompt Pack modules run 10 new adapter characterizations",
    );
    expect(phase6Roadmap).toContain(
      "11,286 ms, 9,669 ms, and 9,006 ms",
    );
    expect(phase6Roadmap).toContain(
      "incomplete / no performance conclusion",
    );
    expect(phase6Roadmap).toContain("Timing was advisory and did not decide retention");
    expect(phase6Roadmap).toContain("11,669 ms, below 15,000 ms");
    expect(phase6Roadmap).toContain("Phase 5 was 10,410 ms");
    expect(phase6Roadmap).toContain(
      "Phase 7 is implemented and retained",
    );

    expect(promptPacksVerification).toContain(
      "**Result: implemented and retained.**",
    );
    expect(promptPacksVerification).toContain(
      "223 baseline identities in extractum-prompt-packs",
    );
    expect(promptPacksVerification).toContain(
      "total new Prompt Pack characterizations: 35",
    );
    expect(promptPacksVerification).toContain(
      "BASELINE_RAW_MS=[11286, 9669, 9006]",
    );
    expect(promptPacksVerification).toContain(
      "CANDIDATE_DISPOSITION=incomplete / no performance conclusion",
    );
    expect(promptPacksVerification).toContain(
      "workspace_check,1,11669",
    );
    expect(promptPacksVerification).toContain(
      "PID 13480 remained alive after five seconds",
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

  it("tracks the exact analysis package owner through the extraction move", () => {
    const appModels = path.join(repoRoot, "src-tauri/src/analysis/models.rs");
    const crateModels = path.join(
      repoRoot,
      "src-tauri/crates/extractum-analysis/src/models.rs",
    );

    expect(existsSync(appModels)).toBe(!analysisCrateExtracted);
    expect(existsSync(crateModels)).toBe(analysisCrateExtracted);
  });

  it("records the retained Phase 7 analysis boundary and closed lifecycle state", () => {
    expect(analysisBoundaryDesign).toContain(
      "**Status:** Implemented and retained; [verification](../verification/2026-07-22-extractum-analysis-extraction.md)",
    );
    expect(phase7Status).toBeDefined();
    expect([
      "design approved; implementation not started",
      "preparation Checkpoint 1 retained",
      "preparation Checkpoint 2 retained",
      "preparation Checkpoint 3 retained",
      "preparation Checkpoint 4 retained",
      "preparation Checkpoint 5 retained",
      "done: retained",
      "not retained",
    ]).toContain(phase7Status);
    expect(phase7Status).toBe("done: retained");
    expect(phase7Roadmap).toContain(
      "2026-07-22-analysis-crate-boundary-design.md",
    );
    expect(phase7Roadmap).toContain("54 Rust files / 13,187 physical lines");
    expect(phase7Roadmap).toContain(
      "143 executable tests split exactly 95 crate / 48 app",
    );
    expect(phase7Roadmap).toContain(
      "49 commits touched analysis; 41 categorized commits (83.7%) touched no other Rust domain",
    );
    expect(phase7Roadmap).toContain(
      "one ordinary mandatory workspace-check duration only",
    );
    expect(phase7Roadmap).toContain(
      "no focused probe, warm-up, or sample series",
    );
    expect(phase7Roadmap).toContain(
      "timing is advisory and cannot invalidate or roll back a correct ownership boundary",
    );
    expect(phase7Roadmap).toContain(
      "standing adjacent-results rule at 15,000 ms",
    );
    expect(phase7Roadmap).not.toContain(
      "Implementation requires a separate implementation plan and explicit owner instruction",
    );
    expect(phase7Roadmap).toContain(
      "2026-07-22-extractum-analysis-extraction.md",
    );
    expect(phase7Roadmap).toContain(
      "35 app Rust files / 7,230 physical lines and 45 crate Rust files / 11,336 physical lines",
    );
    expect(phase7Roadmap).toContain(
      "The check completed in 5,162 ms, below 15,000 ms",
    );
    expect(
      [
        "Phase 8 has an owner-approved boundary; 8A preparation is retained",
        "Phase 8 has an owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 1 authority are retained",
        "Phase 8 has an owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 2 authority are retained",
        "Phase 8 has an owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 3 authority are retained",
        "Phase 8 has an owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 4 authority are retained",
        "Phase 8 has an owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 5 authority are retained",
        "Phase 8 has an owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 6 authority are retained",
        "Phase 8 has an owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 7 authority are retained",
      ].some((status) => phase7Roadmap.includes(status)),
    ).toBe(true);
    expect(roadmapTiming).toContain(
      "Phase 7 `extractum-analysis` | 5,162 ms | completed and retained; below 15,000 ms",
    );
    expect(analysisBoundaryDesign).toContain(
      "used `#[path]` and plain `mod` declarations, not the implementation plan's stale model of 19 temporary `include!` seams",
    );
    expect(analysisBoundaryDesign).toContain(
      "`corpus/tests/harness_portable.rs`, `corpus/tests/mod_portable.rs`, and `store/tests/mod_portable.rs`",
    );
    expect(analysisBoundaryDesign).toContain(
      "35 app Rust files / 7,230 physical lines and 45 crate Rust files / 11,336 physical lines",
    );
    expect(analysisBoundaryDesign).toContain(
      "No focused-loop speedup is promised; zero improvement or a slower result does not invalidate the ownership boundary or trigger rollback",
    );
    expect(analysisBoundaryDesign).toContain(
      "Family 1 is not permission to route every run-row read through a borrowed connection",
    );
    expect(analysisExtractionPlan).toContain(
      "## Post-execution staging disposition",
    );
    expect(analysisExtractionPlan).toContain(
      "The 19-entry `include!` map above is the original pre-execution design, not execution evidence",
    );
    expect(analysisStagingDisposition).toContain(
      "Retained Checkpoint 5 commit `1a69e568` had 21 staging files",
    );
    expect(analysisStagingDisposition).toContain(
      "15 module-scope `include!` seams",
    );
    expect(analysisStagingDisposition).toContain(
      "two plain staging declarations, `corpus_portable` and `source_resolution_policy`",
    );
    expect(analysisStagingDisposition).toContain(
      "one staging `#[path]` declaration, compiling `domain_portable.rs` as `domain`",
    );
    const unreachableInventory = analysisStagingDisposition.match(
      /exactly three intentionally unreachable staging test files: (.*?)\. The analysis tree/,
    )?.[1];
    expect(unreachableInventory).toBeDefined();
    expect([
      ...(unreachableInventory ?? "").matchAll(/`([^`]+)`/g),
    ].map((match) => match[1])).toEqual([
      "corpus/tests/harness_portable.rs",
      "corpus/tests/mod_portable.rs",
      "store/tests/mod_portable.rs",
    ]);
    expect(analysisStagingDisposition).toContain(
      "ten `#[path]` declarations in total: that one staging declaration plus nine namespace adapters",
    );
    expect(analysisExtractionVerification).not.toContain(
      "Only pre-existing `dead_code`/unused warnings remain; no warning was widened into this slice.",
    );
    expect(analysisExtractionVerification).toContain(
      "The post-execution audit found Phase 7-introduced dead-code and unused-facade warnings",
    );
    expect(analysisExtractionVerification).toContain(
      "This remediation removed every warning introduced by the Phase 7 extraction",
    );
  });

  it("records the retained Phase 8 Telegram preparation disposition", () => {
    expect(
      [
        "**Status:** Approved; 8A preparation retained; 8B not started",
        "**Status:** Approved; 8B preparation Checkpoint 1 retained",
        "**Status:** Approved; 8B preparation Checkpoint 2 retained",
        "**Status:** Approved; 8B preparation Checkpoint 3 retained",
        "**Status:** Approved; 8B preparation Checkpoint 4 retained",
        "**Status:** Approved; 8B preparation Checkpoint 5 retained",
        "**Status:** Approved; 8B preparation Checkpoint 6 retained",
        "**Status:** Approved; 8B preparation Checkpoint 7 retained",
      ].some((status) => telegramBoundaryDesign.includes(status)),
    ).toBe(true);
    expect(
      [
        "owner-approved Phase 8 boundary; 8A preparation is retained; 8B has not started.",
        "owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 1 authority are retained.",
        "owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 2 authority are retained.",
        "owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 3 authority are retained.",
        "owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 4 authority are retained.",
        "owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 5 authority are retained.",
        "owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 6 authority are retained.",
        "owner-approved Phase 8 boundary; 8A preparation and 8B Checkpoint 7 authority are retained.",
      ].some((status) => compact(crateRoadmap).includes(status)),
    ).toBe(true);
    expect(compact(crateRoadmap)).not.toContain(
      "implementation has not started",
    );
    expect(phase8Status).toBeDefined();
    expect([
      "design drafted; awaiting owner approval",
      "design approved; implementation not started",
      "8A preparation Checkpoint 1 retained",
      "8A preparation Checkpoint 2 retained",
      "8A preparation Checkpoint 3 retained",
      "8A preparation Checkpoint 4 retained",
      "8A preparation Checkpoint 5 retained",
      "8A preparation retained",
      "8B preparation Checkpoint 1 retained",
      "8B preparation Checkpoint 2 retained",
      "8B preparation Checkpoint 3 retained",
      "8B preparation Checkpoint 4 retained",
      "8B preparation Checkpoint 5 retained",
      "8B preparation Checkpoint 6 retained",
      "8B preparation Checkpoint 7 retained",
      "8B preparation Checkpoint 8 retained",
      "8B preparation retained; 8C pending",
      "done: retained",
      "not retained",
    ]).toContain(phase8Status);
    expect([
      "8A preparation retained",
      "8B preparation Checkpoint 1 retained",
      "8B preparation Checkpoint 2 retained",
      "8B preparation Checkpoint 3 retained",
      "8B preparation Checkpoint 4 retained",
      "8B preparation Checkpoint 5 retained",
      "8B preparation Checkpoint 6 retained",
      "8B preparation Checkpoint 7 retained",
    ]).toContain(phase8Status);
    expect(phase8Roadmap).toContain(
      "2026-07-26-telegram-crate-boundary-design.md",
    );
    expect(phase8Roadmap).toContain(
      "14 production Rust files / 11,281 physical lines / 119 test attributes",
    );
    expect(phase8Roadmap).toContain(
      "19 files / 13,422 physical lines / 140 test attributes",
    );
    expect(phase8Roadmap).toContain(
      "The historical five-file ceiling was only 2,047 physical lines / 22 test attributes",
    );
    expect(phase8Roadmap).toContain(
      "8A freezes behavior, the 140-entry identity map, and prepares DTO",
    );
    expect(phase8Roadmap).toContain(
      "8B completes every live-source and Takeout pure-value seam inside the app",
    );
    expect(phase8Roadmap).toContain(
      "8C creates the crate, mechanically moves the complete prepared",
    );
    expect(phase8Roadmap).toContain(
      "The first separately green 8A checkpoint inventories core-facade path noise",
    );
    expect(phase8Roadmap).toContain(
      "43 `crate::error`, five `crate::compression`, and two `crate::time` references",
    );
    expect(phase8Roadmap).toContain(
      "8A directly normalizes only the six error paths inside the final-owner identity validation implementation/test",
    );
    expect(phase8Roadmap).toContain(
      "A separate GREEN 8B portable-tree checkpoint requires zero occurrences",
    );
    expect(phase8Roadmap).toContain(
      "App-owned facade consumers stay unchanged",
    );
    expect(phase8Roadmap).toContain(
      "moving runtime/session before raw Takeout would either expose `Client`/`MemorySession` publicly or duplicate the runtime",
    );
    expect(phase8Roadmap).toContain(
      "The historical `accounts.rs`/`secret_store.rs` whole-file ownership assumption is explicitly void",
    );
    expect(phase8Roadmap).toContain(
      "`extractum-telegram` owns the Telegram-ingest media payload and classification layer",
    );
    expect(phase8Roadmap).toContain(
      "`extractum_core::error::AppResult` directly",
    );
    expect(phase8Roadmap).toContain(
      "No public `TelegramError` or app-facade error conversion layer is introduced",
    );
    expect(phase8Roadmap).toContain(
      "sorted required/forbidden baseline generated from locked Cargo metadata",
    );
    expect(phase8Roadmap).toContain(
      "staged modules use only relative `self::`/`super::` paths",
    );
    expect(phase8Roadmap).toContain(
      "8C preserves staged files byte-for-byte",
    );
    expect(phase8Roadmap).toContain(
      "Consumer paths remain byte-identical, as required by the standing mechanical-move rule",
    );
    expect(phase8Roadmap).toContain(
      "8C makes no visibility change",
    );
    expect(phase8Roadmap).toContain(
      "`TelegramMessageIdentity`, `TelegramItemContext`, and the Telegram item-kind constant move",
    );
    expect(phase8Roadmap).toContain(
      "dead test-only generic `insert_source_item` and unused draft `external_id` are removed",
    );
    expect(phase8Roadmap).toContain(
      "`item_kind_constants_match_persisted_wire_values` identity retains the two YouTube assertions",
    );
    expect(phase8Roadmap).toContain(
      "`dto::tests::telegram_item_kind_constant_matches_persisted_wire_value`",
    );
    expect(phase8Roadmap).toContain(
      "three predeclared companion decompositions",
    );
    expect(phase8Roadmap).toContain(
      "140 baseline identities map to exactly 143 final executable identities",
    );
    expect(phase8Roadmap).toContain(
      "43 `sources::test_support` SQL/integration identities",
    );
    expect(phase8Roadmap).toContain(
      "8A must classify the residual 73 individually",
    );
    expect(phase8Roadmap).toContain(
      "`8B preparation retained; 8C pending`",
    );
    expect(phase8Roadmap).toContain(
      "There is no focused probe, warm-up, median, quiet-window, retry, worktree, or timing harness",
    );
    expect(phase8Roadmap).toContain(
      "only a completed 8C result becomes Phase 8's single input",
    );
    expect(phase8Roadmap).toContain(
      "A workspace check repeated internally by `npm.cmd run verify` is a correctness gate, not another admitted timing result",
    );
    expect(phase8Roadmap).toContain(
      "The approved design does not authorize implementation",
    );

    expect(telegramBoundaryDesign).toContain(
      "The current tree no longer supports the old whole-module idea",
    );
    expect(telegramBoundaryDesign).toContain(
      "`extractum-telegram` therefore has no SQLx production dependency",
    );
    expect(telegramBoundaryDesign).toContain(
      "There is no public generic `invoke<R: RemoteCall>` equivalent",
    );
    expect(telegramBoundaryDesign).toContain(
      "The generic secure-store implementation and its LLM/YouTube namespaces never move",
    );
    expect(telegramBoundaryDesign).toContain(
      "`TelegramApiHash`: API-hash secret material with zeroization",
    );
    expect(telegramBoundaryDesign).toContain(
      "`extractum-telegram` becomes the single owner of the Telegram-ingest payload",
    );
    expect(telegramBoundaryDesign).toContain(
      "current `ExtractedMediaPayload` is renamed once to public",
    );
    expect(telegramBoundaryDesign).toContain(
      "current `ExtractedItemPayload` is not retained as a second public DTO",
    );
    expect(telegramBoundaryDesign).toContain(
      "App SQL accepts `TelegramMessageDraft` directly",
    );
    expect(telegramBoundaryDesign).toContain(
      "the full known production move-and-touch surface is 19 files, 13,422 physical lines, and 140 test attributes",
    );
    expect(telegramBoundaryDesign).toContain(
      "The complete known disposition is:",
    );
    expect(telegramTypePerimeter).toContain(
      "Of the 21 type-closure identities",
    );
    expect(telegramTypePerimeter).toContain(
      "`sources::types::tests::telegram_message_identity_validation_rejects_invalid_values` moves with the identity",
    );
    expect(telegramTypePerimeter).toContain(
      "the other 20 baseline primaries remain app-owned",
    );
    expect(telegramTypePerimeter).toContain(
      "`sources::types::tests::item_kind_constants_match_persisted_wire_values`",
    );
    expect(telegramTypePerimeter).toContain(
      "`telegram_item_kind_constant_matches_persisted_wire_value` owns the `ITEM_KIND_TELEGRAM_MESSAGE == \"telegram_message\"` assertion",
    );
    expect(telegramTypePerimeter).toContain(
      "21 immutable baseline entries and 22 final executable identities",
    );
    expect(telegramTypePerimeter).toContain(
      "`Unsupported Telegram history peer kind '{}'`",
    );
    expect(telegramTypePerimeter).toContain(
      "`Telegram history peer id must be positive`",
    );
    expect(telegramTypePerimeter).toContain(
      "`Telegram message id must be positive`",
    );
    expect(telegramTypePerimeter).toContain(
      "Checks run in this exact order: supported peer kind, positive peer ID, then positive message ID; the first failing branch wins",
    );
    expect(telegramTypePerimeter).toContain(
      "Every branch returns `AppErrorKind::Validation`",
    );
    expect(telegramTypePerimeter).toContain(
      '`{"kind":"validation","message":"<the exact message above>"}`',
    );
    expect(telegramSubSlices).toContain(
      "#### First 8A checkpoint: core-facade inventory and identity-seam normalization",
    );
    expect(telegramSubSlices).toContain(
      "contains 166 `crate::<root>` references",
    );
    expect(telegramSubSlices).toContain(
      "43 `crate::error` references",
    );
    expect(telegramSubSlices).toContain(
      "five `crate::compression` references",
    );
    expect(telegramSubSlices).toContain(
      "two `crate::time` references",
    );
    expect(telegramSubSlices).toContain(
      "8A does not replace all 50 references across mixed and app-owned regions",
    );
    expect(telegramSubSlices).toContain(
      "changes exactly the six `crate::error` paths",
    );
    expect(telegramSubSlices).toContain(
      "`sources::types::tests::telegram_message_identity_validation_rejects_invalid_values` test to direct `extractum_core::error` paths",
    );
    expect(telegramSubSlices).toContain(
      "This includes both `AppErrorKind::Validation` assertions",
    );
    expect(telegramSubSlices).toContain(
      "The other 44 known core-facade references remain unchanged in this checkpoint",
    );
    expect(telegramSubSlices).toContain(
      "App-owned consumers retain their facade paths; future-owner references normalize only as their symbols enter staging during 8B",
    );
    expect(telegramSubSlices).toContain(
      "App-owned sentinels in `TelegramSourceKind`/`SourceItemsCursor`, item/peer/sync compression, and Takeout persistence time calls",
    );
    expect(telegramSubSlices).toContain(
      "During 8B, a separately green portable-tree import checkpoint requires zero `crate::error`, `crate::compression`, or `crate::time` references inside the exact `src-tauri/src/telegram_impl/**` staging tree",
    );
    expect(telegramSubSlices).toContain(
      "it does not mass-rewrite app-owned source",
    );
    const expectedTelegramDispositionPaths = [
      "takeout_import/mod.rs",
      "sources/items.rs",
      "sources/peer_resolution.rs",
      "ingest_provenance.rs",
      "sources/topics.rs",
      "telegram.rs",
      "takeout_import/raw_parse.rs",
      "takeout_import/pagination.rs",
      "sources/sync.rs",
      "telegram_session_store.rs",
      "sources/types.rs",
      "lib.rs",
      "sources/identity.rs",
      "takeout_import/export_dc.rs",
      "takeout_import/migrated_history.rs",
      "media.rs",
      "takeout_import/forum_topics.rs",
      "sources/avatar.rs",
      "sources/mod.rs",
    ];
    for (const currentPath of expectedTelegramDispositionPaths) {
      expect(telegramDisposition).toContain(`| \`${currentPath}\` |`);
    }
    expect(
      [...telegramDisposition.matchAll(/\| `([^`]+\.rs)` \|/g)].map(
        (match) => match[1],
      ),
    ).toEqual(expectedTelegramDispositionPaths);
    expect(telegramBoundaryDesign).toContain(
      "`TelegramMessageIdentity`, its `validate()` behavior",
    );
    expect(telegramBoundaryDesign).toContain(
      "`TelegramItemContext` moves from `sources/items.rs`",
    );
    expect(telegramBoundaryDesign).toContain(
      "`TelegramMessageDraft` has no `external_id` field",
    );
    expect(telegramBoundaryDesign).toContain(
      "`insert_telegram_source_item_writes_payload_and_skips_duplicates`",
    );
    expect(telegramBoundaryDesign).toContain(
      "`dto.rs` owns `TelegramMessageDraft`, `TelegramMessageIdentity`, `TelegramItemContext`",
    );
    expect(telegramBoundaryDesign).toContain(
      "`media.rs` owns `TelegramMediaPayload`, `DocumentSignals`",
    );
    expect(telegramStagingContract).toContain(
      "The filename is retained for that private failure-translation role and does not imply a public Telegram error taxonomy",
    );
    expect(telegramBoundaryDesign).toContain(
      "`extractum_core::error::{AppError, AppResult}` remains the Rust boundary",
    );
    expect(telegramBoundaryDesign).toContain(
      "No public `TelegramError` or parallel terminal-error taxonomy is introduced",
    );
    expect(telegramBoundaryDesign).toContain(
      "the app facade performs no error conversion",
    );
    expect(telegramBoundaryDesign).toContain(
      "The committed map is a literal immutable baseline",
    );
    expect(telegramTestOwnership).toContain(
      "`companion_final_ids` is normally empty. It is populated only when an existing mixed-subject baseline identity",
    );
    expect(telegramTestOwnership).toContain(
      "These are the three known mandatory decompositions",
    );
    expect(telegramTestOwnership).toContain(
      "`sources::types::tests::item_kind_constants_match_persisted_wire_values` with only `youtube_transcript` and `youtube_comment` persisted-wire assertions",
    );
    expect(telegramTestOwnership).toContain(
      "`dto::tests::telegram_item_kind_constant_matches_persisted_wire_value` with staged ID `telegram_impl::dto::tests::telegram_item_kind_constant_matches_persisted_wire_value` and the sole `ITEM_KIND_TELEGRAM_MESSAGE == \"telegram_message\"` assertion",
    );
    expect(telegramTestOwnership).toContain(
      "140 immutable baseline identities map to 143 final executable identities",
    );
    expect(telegramBoundaryDesign).toContain(
      "exactly 43 SQL/integration tests use one or more of the seven app-only",
    );
    expect(telegramBoundaryDesign).toContain(
      "the remaining 73 identities have no aggregate owner",
    );
    expect(telegramBoundaryDesign).toContain(
      "No app dev dependency, reverse dependency, or fixture crate is introduced",
    );
    expect(telegramBoundaryDesign).toContain(
      "`raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids`",
    );
    expect(telegramBoundaryDesign).toContain(
      "`raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id`",
    );
    expect(telegramBoundaryDesign).toContain(
      "`media-metadata-core-contract.test.ts` must remain GREEN throughout Phase 8",
    );
    expect(telegramBoundaryDesign).toContain(
      "The 2,828-line `takeout_import/mod.rs` is not moved or split ad hoc during 8C",
    );
    expect(telegramBoundaryDesign).toContain(
      '`#[path = "telegram_impl/lib.rs"] mod telegram_impl;`',
    );
    expect(telegramBoundaryDesign).toContain(
      "uses only `self::`/`super::` relative module paths",
    );
    expect(telegramBoundaryDesign).toContain(
      "every app-owned source outside the staging tree refers to staged public API only through the exact `crate::telegram_impl::` prefix",
    );
    expect(telegramBoundaryDesign).toContain(
      "The move preserves every staged source file byte-for-byte",
    );
    expect(telegramBoundaryDesign).toContain(
      "After moving the staged `lib.rs`, 8C creates a new private compatibility facade",
    );
    expect(telegramBoundaryDesign).toContain(
      "explicit curated `pub(crate) use extractum_telegram::{...};` allowlist; no glob is authorized",
    );
    expect(telegramBoundaryDesign).toContain(
      "every consumer `crate::telegram_impl::` path remain byte-identical",
    );
    expect(telegramBoundaryDesign).toContain(
      "requires exact path and byte-hash equality",
    );
    expect(telegramBoundaryDesign).toContain(
      "its four largest files are 61.9% and its eight largest are 83.8%",
    );
    expect(telegramBoundaryDesign).toContain(
      "The primary final absence proof parses JSON",
    );
    expect(telegramBoundaryDesign).toContain(
      "Grammers packages are intentionally allowed elsewhere in `packages` and as transitive app dependencies",
    );
    expect(telegramBoundaryDesign).toContain(
      "remove the app's immediate `extractum-telegram` edge from a copy of the resolve graph",
    );
    expect(telegramBoundaryDesign).toContain(
      "Resolved feature closure is not maintained as a literal array in this design",
    );
    expect(telegramBoundaryDesign).toContain(
      "generate and commit an order-independent feature baseline",
    );
    expect(telegramBoundaryDesign).toContain(
      "the required set from `resolve.nodes[].features`",
    );
    expect(telegramBoundaryDesign).toContain(
      "the forbidden set consisting of every key in `packages[].features` that is not required",
    );
    expect(telegramManifestContract).toContain(
      "The package-defined and resolved feature sets, including the effective `grammers-mtsender` result, are derived only from locked Cargo metadata",
    );
    expect(telegramManifestContract).toContain(
      "they are not hard-coded as prose assertions",
    );
    expect(telegramManifestContract).toContain(
      "require the current package feature-key universe to equal the recorded universe",
    );
    expect(telegramManifestContract).toContain(
      "Any change to the direct default-feature or explicit-feature declaration policy of the four pinned Grammers roots requires a design amendment",
    );
    expect(telegramManifestContract).not.toContain(
      "refine dependency features, without amending this design",
    );
    expect(telegramManifestContract).not.toContain(
      "`grammers-mtsender = []`",
    );
    expect(telegramBoundaryDesign).toContain(
      "A passing grep without the metadata proof is not dependency evidence",
    );
    expect(telegramBoundaryDesign).toContain(
      "8B contains a named, separately green **workspace dependency normalization**",
    );
    expect(telegramBoundaryDesign).toContain(
      "checkpoint must not fabricate a `Cargo.lock` hunk",
    );
    expect(telegramBoundaryDesign).toContain(
      'grammers-client = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", default-features = false }',
    );
    expect(telegramBoundaryDesign).toContain(
      'grammers-session = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", default-features = false, features = ["serde"] }',
    );
    expect(telegramBoundaryDesign).toContain(
      'grammers-mtsender = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa" }',
    );
    expect(telegramBoundaryDesign).toContain(
      'grammers-tl-types = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", features = ["deserializable-functions"] }',
    );
    expect(telegramBoundaryDesign).toContain(
      "The expected production Tokio features include `rt`, `sync`, and `time`",
    );
    expect(telegramBoundaryDesign).toContain(
      "`tokio-util` is not implied merely because adjacent crates use it",
    );
    expect(telegramBoundaryDesign).toContain(
      "parsed Cargo metadata proves the application package has one path edge",
    );
    expect(telegramBoundaryDesign).toContain(
      "`8B preparation retained; 8C pending` is not an extraction success or an independently claimed dependency benefit",
    );
    expect(telegramBoundaryDesign).toContain(
      "`8B preparation retained; 8C pending`",
    );
    expect(telegramBoundaryDesign).toContain(
      "8B and 8C are not combined into one implementation plan",
    );
    expect(telegramBoundaryDesign).toContain(
      "Intermediate 8A and 8B values are diagnostic only",
    );
    expect(telegramBoundaryDesign).toContain(
      "`npm.cmd run verify` may internally execute another workspace check as a correctness gate",
    );
    expect(telegramBoundaryDesign).toContain(
      "eliminating its duplicate correctness work requires a separate owner-approved verification-workflow change",
    );
    expect(telegramBoundaryDesign).toContain(
      "No implementation plan is written before the written specification is explicitly approved",
    );
    expect(telegramStagingContract).toContain(
      "No staged source contains `crate::`",
    );
    expect(telegramStagingContract).toContain(
      "Inside the moved tree, no module path, import, visibility, or source text changes in 8C",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "The complete existing-symbol widening/rename allowlist is:",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`SourceItemInsert` becomes public `TelegramMessageDraft`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "are exactly `telegram_identity`, `telegram_context`, `content`, `content_kind`, `author`, `published_at`, `raw_data`, `item_kind`, and `media`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`TelegramMessageIdentity` and its five fields",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`history_peer_kind`, `history_peer_id`, `telegram_message_id`, `migration_domain`, and `is_migrated_history`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`TelegramItemContext` and its five fields",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`reply_to_msg_id`, `reply_to_peer_kind`, `reply_to_peer_id`, `reply_to_top_id`, and `reaction_count`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`ExtractedMediaPayload` becomes public `TelegramMediaPayload`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "with exactly public `kind` and `metadata` fields",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "current `ResolvedTelegramSource` becomes public `PeerDescriptor`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`external_id`, `title`, `source_subtype`, `is_member`, `username`, `access_hash`, and `avatar_bytes`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "current `ForumTopicSnapshot`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`topic_id`, `top_message_id`, `title`, `icon_color`, `icon_emoji_id`, `is_closed`, `is_pinned`, `is_hidden`, and `sort_order`",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "`ITEM_KIND_TELEGRAM_MESSAGE` becomes one public constant",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "No existing free function is widened in place",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "8C changes no visibility",
    );
    expect(telegramVisibilityAllowlist).toContain(
      "reject any other `pub` item",
    );
    expect(telegramBoundaryDesign).not.toContain(
      "`TelegramError` is a Rust-domain error",
    );
    expect(telegramBoundaryDesign).not.toContain(
      "application modules or `AppError`",
    );
    expect(telegramBoundaryDesign).not.toContain(
      "The minimum disposition is:",
    );
    expect(telegramBoundaryDesign).not.toContain(
      "only the frozen module-path, import, visibility",
    );
    expect(telegramBoundaryDesign).not.toContain(
      "the exact prefix substitution `crate::telegram_impl::` to `extractum_telegram::`",
    );
    expect(telegramBoundaryDesign).not.toContain(
      "the implementation plan freezes the exact signatures and constructor visibility before 8A code begins",
    );
  });
});
