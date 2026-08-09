import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import reviewArtifact from "../../testing/source-contract-redisposition-review.json";
import currentLedger from "../../testing/source-contract-ledger.json";
import {
  applyReview,
  canonicalJson,
  deriveTailFamilyManifests,
  loadBaseLedger,
  loadBaseTrackedPaths,
  loadCurrentPresentTrackedPaths,
  loadReviewEvidence,
  resolutionForDecision,
  sha256Text,
  validateRuleAdjudications,
  validateCandidateBindingAdjudications,
  validatePacketPolicyBindings,
  validateTailFingerprints,
  validateTailIterationProtocol,
  validateTailOutputScaffold,
  validateTailPacketScaffold,
  validateReview,
} from "./source-contract-redisposition-review.mjs";

const REVIEW_BASE = "a54507d63420bb870c3870c91d7e22b050abae3e";
const hash = (value: unknown) => createHash("sha256").update(String(value)).digest("hex");
const clone = <T>(value: T): T => structuredClone(value);
const evidenceText = (value: unknown) => Buffer.isBuffer(value) || value instanceof Uint8Array
  ? Buffer.from(value).toString("utf8")
  : String(value);
const packetSha256 = "a".repeat(64);
const outputSha256 = "b".repeat(64);
const mandatoryP0Ids = ["SC-000420", "SC-000511", "SC-000512", "SC-000513", "SC-000514", "SC-000515", "SC-000555", "SC-000556", "SC-000557", "SC-000558", "SC-000559", "SC-000560"];
const approvedPlaywrightOwners = new Map([
  ["SC-000312", "test:playwright:e2e/app-shell-responsive.spec.ts#mobile-menu-trigger-responsive-visibility"],
  ["SC-000344", "test:playwright:e2e/research-projects-sources-filter-row.spec.ts#filters-available-across-responsive-layouts"],
  ["SC-000385", "test:playwright:e2e/dialog-layering.spec.ts#dialog-content-visible-interactive-above-overlay"],
]);
const approvedSc000464CargoOwners = [
  "test:cargo:extractum-prompt-packs::runtime::tests::start_service_preserves_idempotency_readiness_preflight_queue_and_execution_order",
  "test:cargo:extractum::prompt_packs::runtime_commands::tests::build_youtube_summary_execution_task_defers_profile_resolution_until_spawned_future_is_polled",
];
const firstCorrectionSc000464CargoOwners = [
  "test:cargo:extractum-prompt-packs::runtime::tests::start_service_returns_existing_before_browser_or_source_ports",
  "test:cargo:extractum-prompt-packs::runtime::tests::browser_runtime_start_gate_maps_unready_status_to_preflight_failure",
  "test:cargo:extractum-prompt-packs::runtime::tests::start_service_issues_ticket_after_queued_event_and_new_tracking",
  "test:cargo:extractum::prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket",
  "test:cargo:extractum::prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task",
];
const preCorrectionSc000464Owner = "test:vitest:src/lib/prompt-packs/start-youtube-summary-run.behavior.test.ts#prompt pack application boundary > keeps start idempotency readiness preflight queued-event spawn and profile-resolution order";
const historicalSc000515BehaviorOwner = "test:vitest:scripts/testing/extractum-grid-wrapper-boundary.behavior.test.ts#SVAR grid APIs stay inside Extractum wrappers";
const sc000515BehaviorOwner = "test:vitest:src/lib/components/extractum-ui/extractum-grid-wrapper-boundary.behavior.component.test.ts#SVAR grid APIs stay inside Extractum wrappers";
const sc000087DeletionReason = "D2 implementation-shape: The exact app-owned TauriAnalysisEventSink bodies and absence of waits, helpers, extra statements, and extra emits are refactor-sensitive body shape; AnalysisEventSink trait conformance is compiler-owned and retained runtime/chat tests cover event outcomes.";
const sc000441CargoOwner = "test:cargo:extractum-llm::public_api_tests::curated_api_keeps_credentials_non_serializable_and_inaccessible";
const sc000441AuthorReason = "The invariant is mixed: stable Cargo compilation truthfully proves the credential-bearing types' negative serialization/debug/secret-exposure traits and URL-userinfo rejection, while exhaustive absence of additional public modules, exports, fields, inherent accessors, and credential-returning routes is a closed architecture surface owned by the structured repository rule.";
const sc000441ArchitectureInvariant = "The extractum-llm public architecture remains exactly curated: internal modules stay private, root exports and public type/method sets are closed, credential-bearing structs remain opaque with no public fields or credential-returning accessors, and internal helper names do not leak.";
const sc000441BehaviorInvariant = "LlmProviderAccess and ResolvedLlmProfile do not implement Serialize, Deserialize, Debug, or ExposeSecret<String>, and OpenAI-compatible base URL normalization rejects embedded userinfo.";
const approvedSc000441Subgroups = [
  {
    assertionOrdinals: [1, 2, 3, 6, 8, 9, 10, 11, 12, 13],
    disposition: "architecture",
    invariant: sc000441ArchitectureInvariant,
    replacementIds: ["rule:extractum-llm-public-api-boundary"],
  },
  {
    assertionOrdinals: [4, 5, 7],
    disposition: "behavior",
    invariant: sc000441BehaviorInvariant,
    replacementIds: [sc000441CargoOwner],
  },
];
const finalTask4OwnerCorrections = new Map([
  ["SC-000025", {
    before: "test:vitest:src/lib/accounts-route-add-account-modal.behavior.test.ts#accounts route add-account modal > keeps the account creation form behind a configured-accounts header action",
    after: "test:vitest:src/routes/accounts/accounts-route-add-account-modal.behavior.component.test.ts#accounts route add-account modal > keeps the account creation form behind a configured-accounts header action",
  }],
  ["SC-000435", {
    before: "test:vitest:src/routes/projects/library/library-page.behavior.test.ts#library prototype contract > renders Library as a separate route backed by the current workflow",
    after: "test:vitest:src/routes/projects/library/library-page.behavior.component.test.ts#library prototype contract > renders Library as a separate route backed by the current workflow",
  }],
  ["SC-000552", {
    before: "test:vitest:src/routes/settings/settings-focus.behavior.test.ts#keeps Settings focused on LLM configuration",
    after: "test:vitest:src/routes/settings/settings-focus.behavior.component.test.ts#keeps Settings focused on LLM configuration",
  }],
]);
const approvedSc000515Subgroups = [
  {
    assertionOrdinals: [1, 11, 19],
    disposition: "architecture",
    invariant: "Direct SVAR imports and scoped SVAR tree styles remain inside the approved Extractum grid wrapper boundary.",
    replacementIds: ["rule:extractum-grid-wrapper-boundary"],
  },
  {
    assertionOrdinals: [2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16, 17, 18, 20, 21],
    disposition: "behavior",
    invariant: "Extractum grid wrappers retain locale, theme, overlay, date, tree, and selection runtime behavior.",
    replacementIds: [sc000515BehaviorOwner],
  },
];
const approvedRuleAdjudications = [
  {
    rule: "D1 applies only when the invariant is exhausted by a completed event and no future repository change can violate it as a defect. Exact current source, symbol, file, or test topology without an independent contract is D2.",
    rowIds: ["SC-000077", "SC-000080", "SC-000081", "SC-000092", "SC-000203"],
    finalClass: "D2_IMPLEMENTATION_SHAPE",
  },
  {
    rule: "A resolved base-closed owner that exactly proves the full invariant makes the legacy evidence D4 before A1.",
    rowIds: ["SC-000356", "SC-000357", "SC-000358"],
    finalClass: "D4_DUPLICATE_EVIDENCE",
  },
  {
    rule: "B2/B3 is determined by the truthful observation seam, not the runner prefix; real browser or OS-process observation is B3.",
    rowIds: ["SC-000420"],
    finalClass: "B3_PROTECTED_EXPENSIVE_BEHAVIOR",
  },
  {
    rule: "A retained owner must prove the complete frozen invariant. Adjacent behavior evidence does not own a no-copy or placement topology assertion.",
    rowIds: ["SC-000449"],
    finalClass: "D2_IMPLEMENTATION_SHAPE",
  },
];
const approvedRuleDisagreements = [
  { rowIds: approvedRuleAdjudications[0].rowIds, oldClass: "D1_COMPLETED_HISTORY_ONLY", newClass: "D2_IMPLEMENTATION_SHAPE", groupRuleChange: approvedRuleAdjudications[0].rule },
  { rowIds: approvedRuleAdjudications[1].rowIds, oldClass: "A1_EXISTING_STRUCTURED_OWNER", newClass: "D4_DUPLICATE_EVIDENCE", groupRuleChange: approvedRuleAdjudications[1].rule },
  { rowIds: approvedRuleAdjudications[2].rowIds, oldClass: "B2_NEW_CHEAP_BEHAVIOR", newClass: "B3_PROTECTED_EXPENSIVE_BEHAVIOR", groupRuleChange: approvedRuleAdjudications[2].rule },
  { rowIds: approvedRuleAdjudications[3].rowIds, oldClass: "B2_NEW_CHEAP_BEHAVIOR", newClass: "D2_IMPLEMENTATION_SHAPE", groupRuleChange: approvedRuleAdjudications[3].rule },
];
const approvedCandidateBindingAdjudications = {
  exactD4Owners: {
    rule: "D4 requires the complete exact base-closed owner set; candidate capability must be derived from the owner identity rather than inflated to the reviewed row invariant.",
    rows: [
      {
        id: "SC-000085",
        ownerEvidence: ["test:cargo:extractum-analysis::report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads"],
      },
      {
        id: "SC-000316",
        ownerEvidence: [
          "test:vitest:src/lib/analysis-source-readers.behavior.component.test.ts#analysis source readers > renders source group metadata from route-owned group fields",
          "test:vitest:src/lib/analysis-source-readers.behavior.component.test.ts#analysis source readers > renders source group activity without source job cards",
          "test:vitest:src/lib/analysis-source-readers.behavior.component.test.ts#analysis source readers > renders source group sources as a route-free tab leaf",
        ],
      },
    ],
  },
  exactFutureOwners: {
    rule: "A retained future-owner class requires the complete exact future owner; a resolved partial candidate cannot satisfy B2.",
    rows: [{
      id: "SC-000090",
      ownerEvidence: ["test:cargo:extractum::analysis::tests_application::analysis_command_event_and_app_error_wire_contracts_are_exact"],
    }],
  },
  retainedClasses: [
    {
      rule: "A resolved adjacent owner does not make evidence duplicate; retain B2 when the exact complete invariant still requires the approved future owner.",
      rowIds: ["SC-000090", "SC-000130", "SC-000136", "SC-000192", "SC-000210", "SC-000289", "SC-000416", "SC-000423", "SC-000453", "SC-000511"],
      finalClass: "B2_NEW_CHEAP_BEHAVIOR",
    },
    {
      rule: "Current topology or completed history remains D2 when no complete exact independent owner proves the frozen invariant.",
      rowIds: ["SC-000088", "SC-000091", "SC-000194"],
      finalClass: "D2_IMPLEMENTATION_SHAPE",
    },
  ],
  postEvidenceUserAdjudications: {
    rule: "Post-evidence user adjudication excludes only SC-000087 and SC-000441 from author/blind fingerprint equality while preserving their exact historical evidence and binding each final resolution fail-closed.",
    rows: [{
      id: "SC-000087",
      historicalClass: "B2_NEW_CHEAP_BEHAVIOR",
      historicalOwnerEvidence: ["test:cargo:extractum-analysis::events::tests::event_adapter_is_bounded_and_nonblocking"],
      historicalCandidateOwnerIds: [
        "test:vitest:src/lib/analysis-report-canvas.behavior.component.test.ts#report canvas component contract > keeps run snapshot reading bounded and snapshot-only",
        "test:cargo:extractum-analysis::events::tests::event_adapter_is_bounded_and_nonblocking",
      ],
      finalClass: "D2_IMPLEMENTATION_SHAPE",
      deletionReason: sc000087DeletionReason,
    }, {
      id: "SC-000441",
      historicalClass: "B2_NEW_CHEAP_BEHAVIOR",
      historicalOwnerEvidence: [sc000441CargoOwner],
      historicalCandidateOwnerIds: [sc000441CargoOwner],
      finalClass: "B2_NEW_CHEAP_BEHAVIOR",
      finalResolution: { subgroups: approvedSc000441Subgroups },
    }],
  },
  protectedCriticality: {
    rule: "Every protected author and blind fingerprint retains the exact mandatory criticalityRef.",
    rows: reviewArtifact.protectedRows.map(({ id, criticalityRef }) => ({ id, criticalityRef })),
  },
};
const approvedPacketPolicyBindings = {
  criticality: [
    { id: "SC-000015", criticalityRef: "TESTING_COVERAGE_FLAKE_QUARANTINE" },
    { id: "SC-000023", criticalityRef: "TESTING_COVERAGE_FLAKE_QUARANTINE" },
    { id: "SC-000387", criticalityRef: "TESTING_COVERAGE_FLAKE_QUARANTINE" },
    { id: "SC-000389", criticalityRef: "TESTING_COVERAGE_FLAKE_QUARANTINE" },
    ...reviewArtifact.protectedRows.map(({ id, criticalityRef }) => ({ id, criticalityRef })),
  ].sort((left, right) => Number(left.id.slice(3)) - Number(right.id.slice(3))),
  rowAdjudications: [
    ...approvedRuleAdjudications.flatMap(({ rule, rowIds, finalClass }) => rowIds.map((id) => ({ id, finalClass, rule }))),
    ...approvedCandidateBindingAdjudications.exactD4Owners.rows.map(({ id }) => ({ id, finalClass: "D4_DUPLICATE_EVIDENCE", rule: approvedCandidateBindingAdjudications.exactD4Owners.rule })),
    ...approvedCandidateBindingAdjudications.retainedClasses.flatMap(({ rule, rowIds, finalClass }) => rowIds.map((id) => ({ id, finalClass, rule }))),
    { id: "SC-000087", finalClass: "B2_NEW_CHEAP_BEHAVIOR", rule: "A resolved adjacent owner does not make evidence duplicate; retain B2 when the exact complete invariant still requires the approved future owner." },
    { id: "SC-000323", finalClass: "D3_NON_OBSERVABLE_VISUAL", rule: "Exact focus/selection styling is a non-normative rendered visual affordance with no truthful non-browser seam." },
    { id: "SC-000333", finalClass: "D2_IMPLEMENTATION_SHAPE", rule: "The declaration pins the exact open-state selector string rather than independently observing the trigger's active state." },
    { id: "SC-000352", finalClass: "D3_NON_OBSERVABLE_VISUAL", rule: "Two-line title clipping is a non-normative rendered-layout detail with no truthful non-browser seam." },
    { id: "SC-000353", finalClass: "D3_NON_OBSERVABLE_VISUAL", rule: "Cell centering and overflow ellipsis are non-normative rendered-layout details with no truthful non-browser seam." },
  ].sort((left, right) => Number(left.id.slice(3)) - Number(right.id.slice(3))),
};
const approvedCalibrationPairs = reviewArtifact.independentReview.calibrations.map(({ class: reasonClass, adjacentClass }) => ({ class: reasonClass, adjacentClass }));
const approvedMandatoryCohortNames = reviewArtifact.independentReview.mandatoryCohorts.map(({ name }) => name);

function row(id: string, path: string, resolution: Record<string, unknown> = { disposition: "behavior", replacementIds: ["test:vitest:src/existing.test.ts#existing"] }) {
  return {
    id,
    path,
    title: id,
    sourceHash: hash(`${id}:source`),
    assertionCount: 2,
    lineage: [],
    invariant: `${id} retains its executable behavior.`,
    ...resolution,
  };
}

const baseLedger = {
  schemaVersion: 1,
  frozenAtCommit: "f".repeat(40),
  sourceReaderExceptions: [],
  rows: [
    row("SC-000001", "src/open-one.test.ts"),
    row("SC-000002", "src/open-two.test.ts"),
    row("SC-000355", "src/path-present-closed-one.test.ts", { disposition: "delete", deletionReason: "closed historical row" }),
    row("SC-000366", "src/path-present-closed-two.test.ts", { disposition: "delete", deletionReason: "closed historical row" }),
    row("SC-000900", "src/path-absent-closed.test.ts", { disposition: "behavior", replacementIds: ["test:vitest:src/closed.test.ts#owner"] }),
    ...mandatoryP0Ids.map((id) => row(id, `src/${id}.test.ts`)),
  ],
};

const baseTrackedPaths = new Set([
  "src/open-one.test.ts",
  "src/open-two.test.ts",
  "src/path-present-closed-one.test.ts",
  "src/path-present-closed-two.test.ts",
  ...mandatoryP0Ids.map((id) => `src/${id}.test.ts`),
]);

const baseOpenIds = new Set(["SC-000001", "SC-000002", ...mandatoryP0Ids]);
const closedDigest = sha256Text(canonicalJson(baseLedger.rows.filter((item) => !baseOpenIds.has(item.id))));
const classIds = [
  "D1_COMPLETED_HISTORY_ONLY", "D2_IMPLEMENTATION_SHAPE", "D3_NON_OBSERVABLE_VISUAL", "D4_DUPLICATE_EVIDENCE",
  "A1_EXISTING_STRUCTURED_OWNER", "T1_EXISTING_TOOL_OWNER", "B1_EXISTING_BEHAVIOR_OWNER", "B2_NEW_CHEAP_BEHAVIOR",
  "B3_PROTECTED_EXPENSIVE_BEHAVIOR", "D5_ACCEPTED_LOSS",
];
const criticalitySources = [{ id: "AGENTS_SECURITY", citation: "AGENTS.md §7" }];

const immutableReasonClasses = clone(reviewArtifact.reasonClasses);
const immutableCriticalitySources = clone(reviewArtifact.criticalitySources);
const componentReplacementCommand = "node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/projects-workspace.behavior.component.test.ts src/lib/analysis-report-canvas.behavior.component.test.ts src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts";
const componentStartupCommand = "node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/SourceStatusCell.component.test.ts";
const verifyGateInventory = [
  "npm run check:gemini-browser-sidecar-binary", "node scripts/validate-testing-transition.mjs", "npm run test:unit",
  "npm run test:component", "npm run test:architecture", "npm run test:legacy-contract", "npm run test:integration:os",
  "npm run test:e2e", "npm run check", "npm run check:rustfmt",
  "cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets",
  "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets", "git diff HEAD --check",
];

function timingFixture() {
  const componentReplacement = { command: componentReplacementCommand, warmupExitCode: 0, retainedSeconds: [28, 29, 30], medianSeconds: 29 };
  const componentStartup = { command: componentStartupCommand, warmupExitCode: 0, retainedSeconds: [4, 5, 6], medianSeconds: 5 };
  const legacyOwner = { command: "npm.cmd run test:legacy-contract", warmupExitCode: 0, retainedSeconds: [10, 11, 12], medianSeconds: 11, baseFiles: clone(reviewArtifact.mechanisms.legacyOwner.baseFiles) };
  const successfulBaseline = { executionOrder: 2, seconds: 244.6, exitCode: 0, authorization: "explicit user-authorized compensating verify after complete GREEN" };
  const verify = {
    command: "npm.cmd run verify", seconds: successfulBaseline.seconds, exitCode: successfulBaseline.exitCode,
    historicalSeconds: [208.1, 321.3, 383.4], gateInventory: verifyGateInventory, successfulBaseline,
    observations: [
      { executionOrder: 1, seconds: 53.987, exitCode: 1, failedGate: "npm run test:unit", failure: "Static draft integration RED: source-contract-redisposition-review.json did not exist." },
      clone(successfulBaseline),
    ],
  };
  const scalablePerRow = (componentReplacement.medianSeconds - componentStartup.medianSeconds) / 46;
  return {
    mechanisms: { componentReplacement, componentStartup, legacyOwner, verify },
    forecast: {
      beforeByDisposition: {}, afterByDisposition: {}, futureOwnersByMechanism: {},
      proposedNewJsdomRows: 0, proposedNewJsdomOrdinals: 0, removableLegacyFiles: 0,
      removableLegacySeconds: legacyOwner.medianSeconds, upperBoundPerRow: componentReplacement.medianSeconds / 46,
      scalablePerRow, upperForecastSeconds: 0, scalableForecastSeconds: 0, netGateForecastSeconds: 0,
      replacementUnitCeiling: 46, legacyDominatesFullUnitCeiling: legacyOwner.medianSeconds >= scalablePerRow * 46,
      scalableForecastPercent: 0, netGateForecastPercent: 0,
    },
  };
}

function defaultDecisions() {
  return [
  {
    id: "SC-000001", sourceHash: baseLedger.rows[0].sourceHash, authorRunId: "author-run", class: "B2_NEW_CHEAP_BEHAVIOR",
    reason: "B2_NEW_CHEAP_BEHAVIOR moves the cheap behavior to its focused owner.",
    resolution: { disposition: "behavior", replacementIds: ["test:vitest:src/future.test.ts#one"] },
  },
  {
    id: "SC-000002", sourceHash: baseLedger.rows[1].sourceHash, authorRunId: "author-run", class: "B3_PROTECTED_EXPENSIVE_BEHAVIOR",
    reason: "B3_PROTECTED_EXPENSIVE_BEHAVIOR retains the protected behavior at the declared seam.", criticalityRef: "AGENTS_SECURITY",
    resolution: { disposition: "behavior", replacementIds: ["test:vitest:src/future.test.ts#two"] },
  },
  ...mandatoryP0Ids.map((id) => ({
    id, sourceHash: baseLedger.rows.find((item) => item.id === id)?.sourceHash, authorRunId: "author-run", class: "B3_PROTECTED_EXPENSIVE_BEHAVIOR",
    reason: "B3_PROTECTED_EXPENSIVE_BEHAVIOR retains the mandatory protected behavior.", criticalityRef: "AGENTS_SECURITY",
    resolution: { disposition: "behavior", replacementIds: [`test:vitest:src/future.test.ts#${id}`] },
  })),
  ] as any[];
}

function artifactFor(decisions = defaultDecisions()) {
  const ordinary = decisions.filter((decision) => !["B3_PROTECTED_EXPENSIVE_BEHAVIOR", "D5_ACCEPTED_LOSS"].includes(decision.class) && !decision.resolution?.subgroups);
  const population = ordinary.map((decision) => decision.id).sort();
  const sample = [...population].sort((a, b) => hash(a).localeCompare(hash(b))).slice(0, Math.ceil(population.length * 0.1));
  const blindResults = [...new Set([...sample, ...mandatoryP0Ids])].map((id) => decisions.find((decision) => decision.id === id)).filter(Boolean).map((decision) => ({
    id: decision.id, sourceHash: decision.sourceHash, reviewerRunId: "reviewer-run", class: decision.class,
    reason: `independent ${decision.reason}`, ...(decision.ownerEvidence ? { ownerEvidence: decision.ownerEvidence } : {}), ...(decision.criticalityRef ? { criticalityRef: decision.criticalityRef } : {}),
  })).sort((left, right) => Number(left.id.slice(3)) - Number(right.id.slice(3)));
  const shardRowIds = blindResults.map((result) => result.id).sort((left, right) => Number(left.slice(3)) - Number(right.slice(3)));
  const shardPacketSha256 = sha256Text("fixture shard packet");
  const shardOutputSha256 = sha256Text("fixture shard output");
  const artifact: any = {
    schemaVersion: 1,
    reviewBaseCommit: REVIEW_BASE,
    ledgerFrozenAtCommit: baseLedger.frozenAtCommit,
    scope: { openRows: 14, closedRows: 3, closedRowsDigest: closedDigest, pathPresentClosedRowIds: ["SC-000355", "SC-000366"] },
    reasonClasses: clone(immutableReasonClasses),
    protectedRows: mandatoryP0Ids.map((id) => ({ id, criticalityRef: "AGENTS_SECURITY" })),
    protectedRowsDigest: sha256Text(canonicalJson(mandatoryP0Ids.map((id) => ({ id, criticalityRef: "AGENTS_SECURITY" })))),
    criticalitySources: clone(immutableCriticalitySources),
    ...timingFixture(),
    decisions,
    independentReview: {
      authorRunId: "author-run",
      reviewer: {
        agentTaskId: "reviewer-agent-task",
        reviewerRunId: "reviewer-run",
        contextPolicy: "blind-no-proposed-class-or-reason",
        packetPath: `.superpowers/sdd/packet.${packetSha256}.json`,
        packetSha256,
        outputPath: `.superpowers/sdd/output.${outputSha256}.json`,
        outputSha256,
      },
      validIterations: 1,
      shards: [{
        index: 0,
        rowIds: shardRowIds,
        packetPath: `.superpowers/sdd/shard-packet.${shardPacketSha256}.json`,
        packetSha256: shardPacketSha256,
        outputPath: `.superpowers/sdd/shard-output.${shardOutputSha256}.json`,
        outputSha256: shardOutputSha256,
      }],
      mergedOutputSha256: sha256Text(canonicalJson(blindResults)),
      blindResults,
      calibrations: [], mandatoryCohorts: [{ name: "mandatory P0", rowIds: mandatoryP0Ids, comparison: "agree" }],
      deterministicSample: { algorithm: "sha256-id-lowest-10-percent", population, populationDigest: sha256Text(canonicalJson(population)), rowIds: sample, iterations: 1, comparison: "agree" },
      disagreements: [],
    },
    acceptedLoss: { rows: 0, assertionOrdinals: 0, items: [] },
  };
  const rowById = new Map(baseLedger.rows.map((item) => [item.id, item]));
  const futureByMechanism = new Map<string, { rows: Set<string>; assertionOrdinals: number }>();
  const proposedRows = new Set<string>();
  let proposedOrdinals = 0;
  for (const decision of decisions) {
    if (!["B2_NEW_CHEAP_BEHAVIOR", "B3_PROTECTED_EXPENSIVE_BEHAVIOR"].includes(decision.class)) continue;
    const row = rowById.get(decision.id) as any;
    const groups = decision.resolution?.subgroups ?? [decision.resolution];
    for (const group of groups) {
      const mechanisms = new Set<string>((group?.replacementIds ?? []).flatMap((id: string) => {
        if (id.startsWith("test:vitest:")) {
          const ownerPath = id.slice("test:vitest:".length).split("#", 1)[0].replaceAll("\\", "/");
          return [ownerPath.endsWith(".component.test.ts")
            || (ownerPath.startsWith("src/lib/components/") && ownerPath.endsWith(".behavior.test.ts"))
            ? "jsdom"
            : "node"];
        }
        if (id.startsWith("test:cargo:")) return ["cargo"];
        return [];
      }));
      if (!mechanisms.size) continue;
      const ordinals = decision.resolution?.subgroups ? group.assertionOrdinals : Array.from({ length: row.assertionCount }, (_, index) => index + 1);
      for (const mechanism of mechanisms) {
        const summary = futureByMechanism.get(mechanism) ?? { rows: new Set<string>(), assertionOrdinals: 0 };
        summary.rows.add(decision.id);
        summary.assertionOrdinals += ordinals.length;
        futureByMechanism.set(mechanism, summary);
      }
      if (decision.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR" && mechanisms.has("jsdom")) {
        proposedRows.add(decision.id);
        proposedOrdinals += ordinals.length;
      }
    }
  }
  const replacement = artifact.mechanisms.componentReplacement.medianSeconds;
  const startup = artifact.mechanisms.componentStartup.medianSeconds;
  const legacy = artifact.mechanisms.legacyOwner.medianSeconds;
  const verify = artifact.mechanisms.verify.seconds;
  const scalable = Math.max(0, replacement - startup) / 46;
  artifact.forecast = {
    ...artifact.forecast,
    futureOwnersByMechanism: Object.fromEntries([...futureByMechanism].map(([mechanism, summary]) => [mechanism, {
      rows: summary.rows.size,
      assertionOrdinals: summary.assertionOrdinals,
    }])),
    proposedNewJsdomRows: proposedRows.size,
    proposedNewJsdomOrdinals: proposedOrdinals,
    upperBoundPerRow: replacement / 46,
    scalablePerRow: scalable,
    upperForecastSeconds: replacement / 46 * proposedRows.size,
    scalableForecastSeconds: scalable * proposedRows.size,
    netGateForecastSeconds: Math.max(0, scalable * proposedRows.size - legacy),
    legacyDominatesFullUnitCeiling: legacy >= scalable * 46,
    scalableForecastPercent: scalable * proposedRows.size / verify * 100,
    netGateForecastPercent: Math.max(0, scalable * proposedRows.size - legacy) / verify * 100,
  };
  return artifact;
}

function p0Review() {
  return review();
}

function withFirstDecision(decision: any) {
  return artifactFor().decisions.map((item: any) => item.id === "SC-000001" ? decision : item);
}

function review(overrides: Record<string, unknown> = {}) {
  return {
    artifact: artifactFor(), baseLedger: clone(baseLedger), currentLedger: clone(baseLedger), baseTrackedPaths,
    ...overrides,
  } as any;
}

function reviewWithApprovedPlaywright() {
  const browserRows = [
    { id: "SC-000312", assertionCount: 8 },
    { id: "SC-000344", assertionCount: 6 },
    { id: "SC-000385", assertionCount: 7 },
  ].map(({ id, assertionCount }) => ({
    ...row(id, `src/${id}.test.ts`, { disposition: "behavior", replacementIds: [approvedPlaywrightOwners.get(id)] }),
    assertionCount,
  }));
  const extendedBase = clone(baseLedger);
  extendedBase.rows.push(...browserRows);
  const extendedTracked = new Set([...baseTrackedPaths, ...browserRows.map((item) => item.path)]);
  const artifact = artifactFor();
  const browserDecisions = browserRows.map((item) => ({
    id: item.id,
    sourceHash: item.sourceHash,
    authorRunId: artifact.independentReview.authorRunId,
    class: "B3_PROTECTED_EXPENSIVE_BEHAVIOR",
    reason: `${item.id} requires the approved browser-owned behavior seam.`,
    criticalityRef: "TESTING_BROWSER_COMPONENT_OWNERSHIP",
    resolution: { disposition: "behavior", replacementIds: [approvedPlaywrightOwners.get(item.id)] },
  }));
  artifact.scope.openRows += browserRows.length;
  artifact.decisions.push(...browserDecisions);
  artifact.independentReview.mandatoryCohorts.push({ name: "approved-browser", rowIds: [...approvedPlaywrightOwners.keys()], comparison: "agree" });
  artifact.independentReview.blindResults.push(...browserDecisions.map((decision) => ({
    id: decision.id,
    sourceHash: decision.sourceHash,
    reviewerRunId: artifact.independentReview.reviewer.reviewerRunId,
    class: decision.class,
    reason: `independent ${decision.reason}`,
    criticalityRef: decision.criticalityRef,
  })));
  artifact.independentReview.blindResults.sort((left: any, right: any) => Number(left.id.slice(3)) - Number(right.id.slice(3)));
  artifact.independentReview.shards[0].rowIds = artifact.independentReview.blindResults.map((result: any) => result.id);
  artifact.independentReview.mergedOutputSha256 = sha256Text(canonicalJson(artifact.independentReview.blindResults));
  artifact.forecast.futureOwnersByMechanism = {
    ...artifact.forecast.futureOwnersByMechanism,
    playwright: { rows: 3, assertionOrdinals: 21 },
  };
  return { artifact, baseLedger: extendedBase, currentLedger: clone(extendedBase), baseTrackedPaths: extendedTracked } as any;
}

describe("source-contract redisposition review", () => {
  it("derives the exact seven frozen Task 4 tail families", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
    ]);

    const families = deriveTailFamilyManifests({ baseLedger: realBaseLedger, baseTrackedPaths: realBaseTrackedPaths });
    expect(families.map(({ name, rowIds, paths }: any) => ({ name, rows: rowIds.length, files: paths.length }))).toEqual([
      { name: "testing/process infrastructure", rows: 24, files: 7 },
      { name: "analysis", rows: 152, files: 24 },
      { name: "projects/library", rows: 73, files: 19 },
      { name: "prompt packs/YouTube", rows: 53, files: 11 },
      { name: "Gemini browser", rows: 29, files: 3 },
      { name: "Rust/security boundaries", rows: 25, files: 6 },
      { name: "other product", rows: 54, files: 14 },
    ]);
    expect(new Set(families.flatMap(({ rowIds }: any) => rowIds)).size).toBe(410);
    expect(new Set(families.flatMap(({ paths }: any) => paths)).size).toBe(84);
  });

  it("requires the artifact to pin the recomputed tail-family manifests", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
    ]);
    const changed = clone(reviewArtifact) as any;
    changed.tailFamilies[0].rowIds.pop();
    expect(validateReview({
      artifact: changed,
      baseLedger: realBaseLedger,
      currentLedger: realBaseLedger,
      baseTrackedPaths: realBaseTrackedPaths,
    })).toContain("tailFamilies: manifests must equal the frozen first-match partition");
  });

  it("validates the packet-only tail shards without author decisions", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const realBaseLedger = await loadBaseLedger({ repoRoot, commit: REVIEW_BASE });
    const { evidenceBytes } = await loadReviewEvidence({ repoRoot, artifact: reviewArtifact });
    const packetBytes = new Map((reviewArtifact as any).tailReview.packetShards.map((shard: any) => [
      shard.packetPath,
      evidenceBytes.get(shard.packetPath),
    ]));
    expect(validateTailPacketScaffold({ artifact: reviewArtifact, baseLedger: realBaseLedger, packetBytes })).toEqual([]);

    const tampered = new Map(packetBytes);
    const first = (reviewArtifact as any).tailReview.packetShards[0];
    const packet = JSON.parse(tampered.get(first.packetPath) as string);
    packet.rows[0].class = "D2_IMPLEMENTATION_SHAPE";
    tampered.set(first.packetPath, `${JSON.stringify(packet, null, 2)}\n`);
    expect(validateTailPacketScaffold({ artifact: reviewArtifact, baseLedger: realBaseLedger, packetBytes: tampered })).toEqual(expect.arrayContaining([
      "tailReview: packet shard 0 byte hash mismatch",
      `${packet.rows[0].id}: tail packet row schema leaks author/reviewer fields`,
    ]));
  });

  it("keeps Task 3 tail mismatches coverage-only while Task 4 fingerprints fail closed", () => {
    const decisions = [
      { id: "SC-000001", authorRunId: "task-3-author", class: "D2_IMPLEMENTATION_SHAPE" },
      { id: "SC-000002", authorRunId: "task-4-author", class: "B2_NEW_CHEAP_BEHAVIOR" },
    ];
    const tailReview: any = {
      authorRunId: "task-4-author",
      tailValidIterations: 1,
      status: "accepted",
      coverageOnlyTask3MismatchIds: ["SC-000001"],
      blindResults: [
        { id: "SC-000001", class: "B2_NEW_CHEAP_BEHAVIOR" },
        { id: "SC-000002", class: "B2_NEW_CHEAP_BEHAVIOR" },
      ],
    };
    expect(validateTailFingerprints({ tailReview, decisions })).toEqual([]);

    tailReview.blindResults[1].class = "D2_IMPLEMENTATION_SHAPE";
    expect(validateTailFingerprints({ tailReview, decisions })).toContain(
      "tailReview: Task 4-authored fingerprint mismatches are not accepted: SC-000002",
    );

    decisions[1].class = "B2_NEW_CHEAP_BEHAVIOR";
    decisions[1].resolution = { disposition: "behavior", replacementIds: ["test:vitest:expected.test.ts#owner"] };
    tailReview.blindResults[1] = {
      id: "SC-000002",
      class: "B2_NEW_CHEAP_BEHAVIOR",
      ownerEvidence: ["test:vitest:different.test.ts#owner"],
    };
    expect(validateTailFingerprints({ tailReview, decisions })).toContain(
      "tailReview: Task 4-authored fingerprint mismatches are not accepted: SC-000002",
    );
  });

  it("binds blind tail owners, statuses, citations, and content-addressed output paths before adoption", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const loaded = await loadReviewEvidence({ repoRoot, artifact: reviewArtifact });
    const baseline = clone(reviewArtifact) as any;
    expect(validateTailOutputScaffold({
      artifact: baseline,
      outputs: baseline.tailReview.packetShards,
      outputBytes: loaded.evidenceBytes,
      packetBytes: loaded.evidenceBytes,
    })).toEqual([]);

    const mutateOutput = (artifact: any, bytes: Map<string, unknown>, mutate: (output: any) => void, retainOldPath = false) => {
      const reference = artifact.tailReview.packetShards[0];
      const oldPath = reference.outputPath;
      const output = JSON.parse(evidenceText(bytes.get(oldPath)));
      mutate(output);
      const body = `${JSON.stringify(output, null, 2)}\n`;
      const outputHash = sha256Text(body);
      const outputPath = retainOldPath ? oldPath : oldPath.replace(/[0-9a-f]{64}\.json$/, `${outputHash}.json`);
      bytes.set(outputPath, body);
      reference.outputSha256 = outputHash;
      reference.outputPath = outputPath;
      return output;
    };
    const validateChanged = (artifact: any, bytes: Map<string, unknown>) => validateTailOutputScaffold({
      artifact,
      outputs: artifact.tailReview.packetShards,
      outputBytes: bytes,
      packetBytes: bytes,
    });

    const fabricatedArtifact = clone(reviewArtifact) as any;
    const fabricatedBytes = new Map<string, unknown>(loaded.evidenceBytes);
    const fabricated = mutateOutput(fabricatedArtifact, fabricatedBytes, (output) => {
      output.results[0].ownerEvidence = ["test:vitest:fabricated.test.ts#owner"];
    });
    expect(validateChanged(fabricatedArtifact, fabricatedBytes)).toContain(
      `${fabricated.results[0].id}: tail output selected an invented candidate owner`,
    );

    const statusArtifact = clone(reviewArtifact) as any;
    const statusBytes = new Map<string, unknown>(loaded.evidenceBytes);
    const statusReference = statusArtifact.tailReview.packetShards[0];
    const oldPacketPath = statusReference.packetPath;
    const statusPacket = JSON.parse(evidenceText(statusBytes.get(oldPacketPath)));
    statusPacket.rows[0].candidateOwners[0].status = "resolved";
    const packetBody = `${JSON.stringify(statusPacket, null, 2)}\n`;
    const packetHash = sha256Text(packetBody);
    const packetPath = oldPacketPath.replace(/[0-9a-f]{64}\.json$/, `${packetHash}.json`);
    statusBytes.set(packetPath, packetBody);
    statusReference.packetPath = packetPath;
    statusReference.packetSha256 = packetHash;
    const statusOutput = mutateOutput(statusArtifact, statusBytes, (output) => { output.packetSha256 = packetHash; });
    expect(validateChanged(statusArtifact, statusBytes)).toContain(
      `${statusOutput.results[0].id}: tail output future-owner class requires future candidate membership`,
    );

    const citationArtifact = clone(reviewArtifact) as any;
    const citationBytes = new Map<string, unknown>(loaded.evidenceBytes);
    const citationOutput = mutateOutput(citationArtifact, citationBytes, (output) => { output.results[0].criticalityRef = "INVENTED_SOURCE"; });
    expect(validateChanged(citationArtifact, citationBytes)).toContain(
      `${citationOutput.results[0].id}: tail output citation is invalid`,
    );

    const pathArtifact = clone(reviewArtifact) as any;
    const pathBytes = new Map<string, unknown>(loaded.evidenceBytes);
    mutateOutput(pathArtifact, pathBytes, (output) => { output.results[0].reason += " coherent mutation"; }, true);
    expect(validateChanged(pathArtifact, pathBytes)).toContain(
      "tailReview: output shard 0 outputPath must contain its SHA-256",
    );
  });

  it("validates the general bounded tail iteration protocol without Task 3 adjudication reuse", async () => {
    const validate = (artifact: any) => validateTailIterationProtocol({ tailReview: artifact.tailReview });
    const entry = (tail: any, iteration: number, changes: Partial<any> = {}) => ({
      iteration,
      requiredBlindRowIds: clone(tail.requiredBlindRowIds),
      samplePopulation: clone(tail.deterministicSample.population),
      sampleRowIds: clone(tail.deterministicSample.rowIds),
      requiredPopulationDigest: tail.requiredPopulationDigest,
      samplePopulationDigest: tail.deterministicSample.populationDigest,
      resultingRequiredBlindRowIds: clone(tail.requiredBlindRowIds),
      resultingSamplePopulation: clone(tail.deterministicSample.population),
      resultingSampleRowIds: clone(tail.deterministicSample.rowIds),
      resultingRequiredPopulationDigest: tail.requiredPopulationDigest,
      resultingSamplePopulationDigest: tail.deterministicSample.populationDigest,
      task4RuleChanged: false,
      samplePopulationChanged: false,
      result: "fixed_point",
      ...changes,
    });

    const iteration2 = clone(reviewArtifact) as any;
    iteration2.tailReview.tailValidIterations = 2;
    iteration2.tailReview.deterministicSample.iterations = 2;
    iteration2.tailReview.iterationHistory = [
      entry(iteration2.tailReview, 1, { task4RuleChanged: true, result: "rule_changed" }),
      entry(iteration2.tailReview, 2),
    ];
    expect(validate(iteration2)).toEqual([]);

    const iteration3 = clone(reviewArtifact) as any;
    const priorDigest = "e".repeat(64);
    iteration3.tailReview.tailValidIterations = 3;
    iteration3.tailReview.deterministicSample.iterations = 3;
    iteration3.tailReview.iterationHistory = [
      entry(iteration3.tailReview, 1, { task4RuleChanged: true, result: "rule_changed" }),
      entry(iteration3.tailReview, 2, { task4RuleChanged: true, result: "rule_changed" }),
      entry(iteration3.tailReview, 3),
    ];
    expect(validate(iteration3)).toEqual([]);

    const skipped = clone(iteration2); skipped.tailReview.iterationHistory[1].iteration = 3;
    expect(validate(skipped)).toContain("tailReview: iteration history must be contiguous and complete");
    const duplicate = clone(iteration2); duplicate.tailReview.iterationHistory[1].iteration = 1;
    expect(validate(duplicate)).toContain("tailReview: iteration history must be contiguous and complete");

    const stale = clone(iteration2); stale.tailReview.iterationHistory[1].samplePopulationDigest = priorDigest;
    expect(validate(stale)).toContain("tailReview: final iteration must compare the current tail population");

    const thirdChangingAccepted = clone(iteration3);
    thirdChangingAccepted.tailReview.iterationHistory[2] = entry(thirdChangingAccepted.tailReview, 3, { task4RuleChanged: true, result: "rule_changed" });
    expect(validate(thirdChangingAccepted)).toContain("tailReview: accepted tail history must end at an unchanged fixed point");

    const blocked = clone(thirdChangingAccepted); blocked.tailReview.status = "blocked";
    expect(validate(blocked)).toContain("tailReview: third valid tail iteration changed rules or population; explicit rule amendment required");
    expect(validate(blocked)).not.toContain("tailReview: unsupported status");

    const iteration4 = clone(iteration3);
    iteration4.tailReview.tailValidIterations = 4;
    iteration4.tailReview.deterministicSample.iterations = 4;
    iteration4.tailReview.iterationHistory.push(entry(iteration4.tailReview, 4));
    expect(validate(iteration4)).toEqual(expect.arrayContaining([
      "tailReview: tailValidIterations must be 0..3 and equal sample iterations",
      "tailReview: a fourth valid tail iteration is forbidden",
    ]));

    const reused = clone(reviewArtifact) as any;
    reused.tailReview.ruleAdjudications = clone(reused.independentReview.ruleAdjudications);
    expect(validate(reused)).toContain("tailReview: Task 3 adjudications cannot authorize tail disagreements");

    const invalidAttempts = clone(reviewArtifact) as any;
    invalidAttempts.tailReview.invalidAttempts.push({ shardIndex: 1, sha256: "d".repeat(64), reason: "second invalid non-counting transport" });
    expect(validate(invalidAttempts)).toEqual([]);
  });

  it("binds every valid tail iteration to distinct reconstructable evidence", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const validate = (artifact: any, evidenceBytes = loaded.evidenceBytes) => validateReview({
      artifact, baseLedger: realBaseLedger, currentLedger: realBaseLedger, baseTrackedPaths: realBaseTrackedPaths, evidenceBytes,
    });
    expect(validate(reviewArtifact)).toEqual([]);

    const remint = (artifact: any, evidenceBytes: Map<string, unknown>, iteration: number, reviewerRunId: string, changes: any = {}) => {
      const sourceTail = reviewArtifact.tailReview as any;
      const packetShards = sourceTail.packetShards.map((source: any, index: number) => {
        const packet = JSON.parse(evidenceText(loaded.evidenceBytes.get(source.packetPath)));
        packet.reviewerRunId = reviewerRunId;
        const packetBody = `${JSON.stringify(packet, null, 2)}\n`;
        const packetSha256 = sha256Text(packetBody);
        const packetPath = `testing/source-contract-redisposition-evidence/fixture-${reviewerRunId}-shard-${index}-packet.${packetSha256}.json`;
        evidenceBytes.set(packetPath, Buffer.from(packetBody));
        const output = JSON.parse(evidenceText(loaded.evidenceBytes.get(source.outputPath)));
        output.reviewerRunId = reviewerRunId;
        output.packetSha256 = packetSha256;
        const outputBody = `${JSON.stringify(output, null, 2)}\n`;
        const outputSha256 = sha256Text(outputBody);
        const outputPath = `testing/source-contract-redisposition-evidence/fixture-${reviewerRunId}-shard-${index}-output.${outputSha256}.json`;
        evidenceBytes.set(outputPath, Buffer.from(outputBody));
        return { index, rowIds: clone(source.rowIds), packetPath, packetSha256, outputPath, outputSha256 };
      });
      const mergedPacket = JSON.parse(evidenceText(loaded.evidenceBytes.get(sourceTail.reviewer.packetPath)));
      mergedPacket.reviewerRunId = reviewerRunId;
      const mergedPacketBody = `${JSON.stringify(mergedPacket, null, 2)}\n`;
      const mergedPacketSha256 = sha256Text(mergedPacketBody);
      const mergedPacketPath = `testing/source-contract-redisposition-evidence/fixture-${reviewerRunId}-merged-packet.${mergedPacketSha256}.json`;
      evidenceBytes.set(mergedPacketPath, Buffer.from(mergedPacketBody));
      const mergedOutput = JSON.parse(evidenceText(loaded.evidenceBytes.get(sourceTail.reviewer.outputPath)));
      mergedOutput.reviewerRunId = reviewerRunId;
      mergedOutput.packetSha256 = mergedPacketSha256;
      const mergedOutputBody = `${JSON.stringify(mergedOutput, null, 2)}\n`;
      const mergedOutputFileSha256 = sha256Text(mergedOutputBody);
      const mergedOutputPath = `testing/source-contract-redisposition-evidence/fixture-${reviewerRunId}-merged-output.${mergedOutputFileSha256}.json`;
      evidenceBytes.set(mergedOutputPath, Buffer.from(mergedOutputBody));
      const reviewer = {
        agentTaskId: `/fixture/reviewer-${iteration}`,
        reviewerRunId,
        contextPolicy: "blind-no-proposed-class-or-reason",
        packetPath: mergedPacketPath,
        packetSha256: mergedPacketSha256,
        outputPath: mergedOutputPath,
        outputSha256: mergedOutputFileSha256,
      };
      const blindResults = mergedOutput.results.map((result: any) => ({ ...result, reviewerRunId }));
      const task4Disagreements = changes.task4Disagreements ?? [];
      const samplePopulationChanged = changes.samplePopulationChanged ?? false;
      const samplePopulation = clone(changes.samplePopulation ?? sourceTail.deterministicSample.population);
      const sampleRowIds = clone(changes.sampleRowIds ?? sourceTail.deterministicSample.rowIds);
      const resultingRequiredBlindRowIds = clone(changes.resultingRequiredBlindRowIds ?? sourceTail.requiredBlindRowIds);
      const resultingSamplePopulation = clone(changes.resultingSamplePopulation ?? sourceTail.deterministicSample.population);
      const resultingSampleRowIds = clone(changes.resultingSampleRowIds ?? sourceTail.deterministicSample.rowIds);
      return {
        iteration,
        reviewerRunId,
        requiredBlindRowIds: clone(sourceTail.requiredBlindRowIds),
        samplePopulation,
        sampleRowIds,
        requiredPopulationDigest: changes.requiredPopulationDigest ?? sourceTail.requiredPopulationDigest,
        samplePopulationDigest: changes.samplePopulationDigest ?? sourceTail.deterministicSample.populationDigest,
        resultingRequiredBlindRowIds,
        resultingSamplePopulation,
        resultingSampleRowIds,
        resultingRequiredPopulationDigest: changes.resultingRequiredPopulationDigest ?? sourceTail.requiredPopulationDigest,
        resultingSamplePopulationDigest: changes.resultingSamplePopulationDigest ?? sourceTail.deterministicSample.populationDigest,
        packetShards,
        reviewer,
        mergedOutputSha256: sha256Text(canonicalJson(blindResults)),
        task4Disagreements,
        coverageOnlyTask3MismatchIds: clone(sourceTail.coverageOnlyTask3MismatchIds),
        task4RuleChanged: task4Disagreements.length > 0,
        samplePopulationChanged,
        result: task4Disagreements.length || samplePopulationChanged ? "rule_changed" : "fixed_point",
      };
    };
    const bindFinal = (artifact: any, entry: any, iterations: number) => {
      const tail = artifact.tailReview;
      tail.tailValidIterations = iterations;
      tail.deterministicSample.iterations = iterations;
      tail.reviewerRunId = entry.reviewerRunId;
      tail.packetShards = clone(entry.packetShards);
      tail.reviewer = clone(entry.reviewer);
      tail.mergedOutputSha256 = entry.mergedOutputSha256;
      tail.requiredBlindRowIds = clone(entry.requiredBlindRowIds);
      tail.requiredPopulationDigest = entry.requiredPopulationDigest;
      tail.coverageOnlyTask3MismatchIds = clone(entry.coverageOnlyTask3MismatchIds);
      tail.blindResults = (reviewArtifact.tailReview as any).blindResults.map((result: any) => ({ ...clone(result), reviewerRunId: entry.reviewerRunId }));
    };
    const disagreement = [{
      id: "SC-000325",
      authorFingerprint: { class: "D2_IMPLEMENTATION_SHAPE", ownerEvidence: [], criticalityRef: null },
    }];
    const makeIteration2 = () => {
      const artifact = clone(reviewArtifact) as any;
      const bytes = new Map<string, unknown>(loaded.evidenceBytes);
      const first = remint(artifact, bytes, 1, "fixture-tail-iteration-1", { task4Disagreements: disagreement });
      const second = remint(artifact, bytes, 2, "fixture-tail-iteration-2");
      artifact.tailReview.iterationHistory = [first, second];
      bindFinal(artifact, second, 2);
      return { artifact, bytes, first, second };
    };

    const valid2 = makeIteration2();
    expect(validate(valid2.artifact, valid2.bytes)).toEqual([]);

    const reused = makeIteration2();
    reused.artifact.tailReview.iterationHistory[0].reviewerRunId = reused.second.reviewerRunId;
    reused.artifact.tailReview.iterationHistory[0].reviewer = clone(reused.second.reviewer);
    reused.artifact.tailReview.iterationHistory[0].packetShards = clone(reused.second.packetShards);
    expect(validate(reused.artifact, reused.bytes)).toContain("tailReview: iteration reviewerRunIds and evidence references must be unique");

    const missing = makeIteration2(); missing.artifact.tailReview.iterationHistory[0].packetShards.pop();
    expect(validate(missing.artifact, missing.bytes)).toContain("tailReview: iteration 1 packet shards must exactly cover its required blind IDs");
    const duplicate = makeIteration2(); duplicate.artifact.tailReview.iterationHistory[0].packetShards[1].index = 0;
    expect(validate(duplicate.artifact, duplicate.bytes)).toContain("tailReview: iteration 1 packet shard indices must be contiguous and unique");

    const tampered = makeIteration2();
    const priorOutput = tampered.first.packetShards[0].outputPath;
    tampered.bytes.set(priorOutput, Buffer.concat([Buffer.from(tampered.bytes.get(priorOutput) as Uint8Array), Buffer.from(" ")]));
    expect(validate(tampered.artifact, tampered.bytes)).toContain("tailReview: iteration 1 output shard 0 byte hash mismatch");

    const digest = makeIteration2(); digest.artifact.tailReview.iterationHistory[0].mergedOutputSha256 = "f".repeat(64);
    expect(validate(digest.artifact, digest.bytes)).toContain("tailReview: iteration 1 merged blind-result digest mismatch");
    const facts = makeIteration2(); facts.artifact.tailReview.iterationHistory[0].task4Disagreements = [];
    expect(validate(facts.artifact, facts.bytes)).toContain("tailReview: iteration 1 task4RuleChanged is not supported by reconstructed disagreements");
    const population = makeIteration2(); population.artifact.tailReview.iterationHistory[0].samplePopulationChanged = true;
    expect(validate(population.artifact, population.bytes)).toContain("tailReview: iteration 1 samplePopulationChanged is not supported by resulting population digests");

    const arbitraryResultDigest = clone(reviewArtifact) as any;
    arbitraryResultDigest.tailReview.iterationHistory[0].resultingRequiredPopulationDigest = "f".repeat(64);
    expect(validate(arbitraryResultDigest)).toContain("tailReview: iteration 1 resulting required population digest mismatch");
    const resultSetHash = makeIteration2(); resultSetHash.artifact.tailReview.iterationHistory[0].resultingRequiredBlindRowIds.pop();
    expect(validate(resultSetHash.artifact, resultSetHash.bytes)).toContain("tailReview: iteration 1 resulting required population digest mismatch");
    const fixedSets = makeIteration2();
    fixedSets.second.resultingRequiredBlindRowIds = fixedSets.second.resultingRequiredBlindRowIds.slice(1);
    fixedSets.second.resultingRequiredPopulationDigest = sha256Text(canonicalJson(fixedSets.second.resultingRequiredBlindRowIds));
    fixedSets.artifact.tailReview.iterationHistory[1] = fixedSets.second;
    expect(validate(fixedSets.artifact, fixedSets.bytes)).toContain("tailReview: iteration 2 fixed point must preserve all input population sets");
    const nondeterministic = makeIteration2();
    nondeterministic.second.resultingSampleRowIds = [...nondeterministic.second.resultingSampleRowIds].reverse();
    nondeterministic.artifact.tailReview.iterationHistory[1] = nondeterministic.second;
    expect(validate(nondeterministic.artifact, nondeterministic.bytes)).toContain("tailReview: iteration 2 resulting sample selection must equal deterministic ten percent");
    const nextInput = makeIteration2();
    nextInput.first.resultingRequiredBlindRowIds = nextInput.first.resultingRequiredBlindRowIds.slice(1);
    nextInput.first.resultingRequiredPopulationDigest = sha256Text(canonicalJson(nextInput.first.resultingRequiredBlindRowIds));
    nextInput.artifact.tailReview.iterationHistory[0] = nextInput.first;
    expect(validate(nextInput.artifact, nextInput.bytes)).toContain("tailReview: iteration 2 input population sets must equal iteration 1 results");

    const finalBinding = makeIteration2(); finalBinding.artifact.tailReview.reviewerRunId = "not-the-final-run";
    expect(validate(finalBinding.artifact, finalBinding.bytes)).toContain("tailReview: final top-level state must equal the last fixed-point iteration");
    const finalSets = makeIteration2(); finalSets.artifact.tailReview.deterministicSample.rowIds = finalSets.artifact.tailReview.deterministicSample.rowIds.slice(1);
    expect(validate(finalSets.artifact, finalSets.bytes)).toContain("tailReview: final top-level population sets must equal the last fixed-point results");

    const valid3 = makeIteration2();
    const sourceSample = (reviewArtifact.tailReview as any).deterministicSample;
    const priorSamplePopulation = sourceSample.population.slice(1);
    const priorSampleRowIds = [...priorSamplePopulation].sort((left, right) => hash(left).localeCompare(hash(right))).slice(0, Math.ceil(priorSamplePopulation.length * 0.1));
    const priorSampleDigest = sha256Text(canonicalJson(priorSamplePopulation));
    const middle = remint(valid3.artifact, valid3.bytes, 2, "fixture-tail-iteration-2-changing", {
      samplePopulation: priorSamplePopulation,
      sampleRowIds: priorSampleRowIds,
      samplePopulationDigest: priorSampleDigest,
      resultingSamplePopulation: sourceSample.population,
      resultingSampleRowIds: sourceSample.rowIds,
      resultingSamplePopulationDigest: sourceSample.populationDigest,
      samplePopulationChanged: true,
    });
    const third = remint(valid3.artifact, valid3.bytes, 3, "fixture-tail-iteration-3");
    valid3.artifact.tailReview.iterationHistory = [valid3.first, middle, third];
    valid3.first.samplePopulation = clone(priorSamplePopulation);
    valid3.first.sampleRowIds = clone(priorSampleRowIds);
    valid3.first.samplePopulationDigest = priorSampleDigest;
    valid3.first.resultingSamplePopulation = clone(priorSamplePopulation);
    valid3.first.resultingSampleRowIds = clone(priorSampleRowIds);
    valid3.first.resultingSamplePopulationDigest = priorSampleDigest;
    bindFinal(valid3.artifact, third, 3);
    expect(validate(valid3.artifact, valid3.bytes)).toEqual([]);
  }, 30_000);

  it("loads and validates accepted content-addressed tail evidence", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    expect(loaded.issues).toEqual([]);
    const input: any = { artifact: reviewArtifact, baseLedger: realBaseLedger, currentLedger: realBaseLedger, baseTrackedPaths: realBaseTrackedPaths, evidenceBytes: loaded.evidenceBytes };
    expect(validateReview(input)).toEqual([]);

    const changed = new Map(loaded.evidenceBytes);
    const first = (reviewArtifact as any).tailReview.packetShards[0];
    changed.set(first.packetPath, `${changed.get(first.packetPath)} `);
    expect(validateReview({ ...input, evidenceBytes: changed })).toContain("tailReview: packet shard 0 byte hash mismatch");
  });

  it("pins the approved SC-000464 comprehensive owner and SC-000515 mixed ordinal split", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const input: any = {
      artifact: reviewArtifact,
      baseLedger: realBaseLedger,
      currentLedger: realBaseLedger,
      baseTrackedPaths: realBaseTrackedPaths,
      evidenceBytes: loaded.evidenceBytes,
    };
    const decision = (reviewArtifact as any).decisions.find((item: any) => item.id === "SC-000464");
    const gridDecision = (reviewArtifact as any).decisions.find((item: any) => item.id === "SC-000515");

    expect(decision.class).toBe("B2_NEW_CHEAP_BEHAVIOR");
    expect(decision.resolution.replacementIds).toEqual(approvedSc000464CargoOwners);
    expect(gridDecision.resolution).toEqual({ subgroups: approvedSc000515Subgroups });
    for (const [id, owners] of finalTask4OwnerCorrections) {
      expect((reviewArtifact as any).decisions.find((item: any) => item.id === id).resolution.replacementIds).toEqual([owners.after]);
    }

    const partial = clone(reviewArtifact) as any;
    partial.decisions.find((item: any) => item.id === "SC-000464").resolution.replacementIds = firstCorrectionSc000464CargoOwners;
    expect(validateReview({ ...input, artifact: partial })).toContain("SC-000464: replacementIds must equal the approved comprehensive Cargo owner");

    const incompleteGrid = clone(reviewArtifact) as any;
    incompleteGrid.decisions.find((item: any) => item.id === "SC-000515").resolution.subgroups[0].assertionOrdinals = [1, 11];
    expect(validateReview({ ...input, artifact: incompleteGrid })).toEqual(expect.arrayContaining([
      "SC-000515: resolution must equal the approved architecture/behavior ordinal split",
      "SC-000515: incomplete subgroup assertion ordinals",
    ]));

    const helperOnlyOwner = clone(reviewArtifact) as any;
    helperOnlyOwner.decisions.find((item: any) => item.id === "SC-000025").resolution.replacementIds = [finalTask4OwnerCorrections.get("SC-000025")?.before];
    expect(validateReview({ ...input, artifact: helperOnlyOwner })).toContain("SC-000025: replacementIds must equal the approved component owner");

    const preCorrectionArtifact = clone(reviewArtifact) as any;
    preCorrectionArtifact.decisions.find((item: any) => item.id === "SC-000464").resolution.replacementIds = approvedSc000464CargoOwners.slice(0, 1);
    preCorrectionArtifact.decisions.find((item: any) => item.id === "SC-000515").resolution.subgroups[1].replacementIds = [historicalSc000515BehaviorOwner];
    for (const [id, owners] of finalTask4OwnerCorrections) {
      preCorrectionArtifact.decisions.find((item: any) => item.id === id).resolution.replacementIds = [owners.before];
    }
    const preCorrectionLedger = applyReview({
      artifact: preCorrectionArtifact,
      baseLedger: realBaseLedger,
      currentLedger: realBaseLedger,
      baseTrackedPaths: realBaseTrackedPaths,
    }).ledger;
    expect(validateReview({ ...input, currentLedger: preCorrectionLedger })).toEqual([]);

    const unapprovedLedger = clone(preCorrectionLedger) as any;
    unapprovedLedger.rows.find((item: any) => item.id === "SC-000464").replacementIds = [preCorrectionSc000464Owner];
    expect(validateReview({ ...input, currentLedger: unapprovedLedger })).toContain("ledger: rows changed outside approved review apply");
  });

  it("pins the exact SC-000087 post-evidence user adjudication and atomic ledger transition", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, currentPresentPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadCurrentPresentTrackedPaths({ repoRoot }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const decision = (reviewArtifact as any).decisions.find((item: any) => item.id === "SC-000087");
    expect(decision).toMatchObject({
      id: "SC-000087",
      sourceHash: "7628287415527ca9e7f1c1a7c53169e3faaa9c7b0e7a21cc5fb8235a865e31b2",
      class: "D2_IMPLEMENTATION_SHAPE",
      reason: sc000087DeletionReason,
      resolution: { disposition: "delete", deletionReason: sc000087DeletionReason },
    });
    expect(decision.ownerEvidence).toBeUndefined();
    expect(decision.resolution.replacementIds).toBeUndefined();
    expect((reviewArtifact as any).forecast.afterByDisposition).toEqual({ behavior: 279, delete: 153, mixed: 4 });
    expect((reviewArtifact as any).forecast.futureOwnersByMechanism.cargo).toEqual({ rows: 18, assertionOrdinals: 147 });

    const input: any = {
      artifact: reviewArtifact,
      baseLedger: realBaseLedger,
      currentLedger,
      baseTrackedPaths: realBaseTrackedPaths,
      currentPresentPaths,
      evidenceBytes: loaded.evidenceBytes,
    };
    expect(validateReview(input)).toEqual([]);

    for (const mutate of [
      (artifact: any) => { artifact.decisions.find((item: any) => item.id === "SC-000087").class = "B2_NEW_CHEAP_BEHAVIOR"; },
      (artifact: any) => { artifact.decisions.find((item: any) => item.id === "SC-000087").resolution.replacementIds = ["test:cargo:extractum-analysis::events::tests::event_adapter_is_bounded_and_nonblocking"]; },
      (artifact: any) => { artifact.decisions.find((item: any) => item.id === "SC-000087").resolution.deletionReason += " mutated"; },
    ]) {
      const artifact = clone(reviewArtifact) as any;
      mutate(artifact);
      expect(validateReview({ ...input, artifact })).toContain(
        "SC-000087: decision must equal the approved post-evidence user adjudication",
      );
    }

    const correctedLedger = applyReview(input).ledger;
    expect(validateReview({ ...input, currentLedger: correctedLedger })).toEqual([]);
    const partialLedger = clone(correctedLedger) as any;
    partialLedger.rows.find((item: any) => item.id === "SC-000087").deletionReason += " mutated";
    expect(validateReview({ ...input, currentLedger: partialLedger })).toContain(
      "ledger: SC-000087 resolution must equal the exact pre-adjudication or corrected state",
    );
  }, 30_000);

  it("pins the exact SC-000441 mixed post-evidence adjudication and atomic ledger transition", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, currentPresentPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadCurrentPresentTrackedPaths({ repoRoot }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const decision = (reviewArtifact as any).decisions.find((item: any) => item.id === "SC-000441");
    expect(decision).toEqual({
      id: "SC-000441",
      sourceHash: "093561de910b2cd887cc9a23796094020d13d9fa1c1d234af875a01276b58700",
      authorRunId: "redisposition-task-3-final-fix-author-run",
      class: "B2_NEW_CHEAP_BEHAVIOR",
      reason: sc000441AuthorReason,
      resolution: { subgroups: approvedSc000441Subgroups },
      criticalityRef: "AGENTS_SECURITY",
    });
    expect((reviewArtifact as any).forecast.afterByDisposition).toEqual({ behavior: 279, delete: 153, mixed: 4 });
    expect((reviewArtifact as any).forecast.futureOwnersByMechanism.cargo).toEqual({ rows: 18, assertionOrdinals: 147 });

    const input: any = {
      artifact: reviewArtifact,
      baseLedger: realBaseLedger,
      currentLedger,
      baseTrackedPaths: realBaseTrackedPaths,
      currentPresentPaths,
      evidenceBytes: loaded.evidenceBytes,
    };
    expect(validateReview(input)).toEqual([]);

    for (const mutate of [
      (artifact: any) => { artifact.decisions.find((item: any) => item.id === "SC-000441").class = "D2_IMPLEMENTATION_SHAPE"; },
      (artifact: any) => { artifact.decisions.find((item: any) => item.id === "SC-000441").reason += " mutated"; },
      (artifact: any) => { artifact.decisions.find((item: any) => item.id === "SC-000441").resolution.subgroups[0].assertionOrdinals.pop(); },
      (artifact: any) => { artifact.decisions.find((item: any) => item.id === "SC-000441").resolution.subgroups[0].replacementIds = ["rule:llm-crate-boundary"]; },
      (artifact: any) => { artifact.decisions.find((item: any) => item.id === "SC-000441").resolution.subgroups[1].invariant += " mutated"; },
    ]) {
      const artifact = clone(reviewArtifact) as any;
      mutate(artifact);
      expect(validateReview({ ...input, artifact })).toContain(
        "SC-000441: decision must equal the approved post-evidence user adjudication",
      );
    }

    const correctedLedger = applyReview(input).ledger;
    expect(validateReview({ ...input, currentLedger: correctedLedger })).toEqual([]);
    const partialLedger = clone(correctedLedger) as any;
    partialLedger.rows.find((item: any) => item.id === "SC-000441").subgroups[0].replacementIds = ["rule:llm-crate-boundary"];
    expect(validateReview({ ...input, currentLedger: partialLedger })).toContain(
      "ledger: SC-000441 resolution must equal the exact pre-adjudication or corrected state",
    );
  }, 30_000);

  it("permits only the exact Task 9 truthful-owner amendment and its atomic ledger transition", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, currentPresentPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadCurrentPresentTrackedPaths({ repoRoot }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const aIds = new Set(["SC-000093", "SC-000094", "SC-000108", "SC-000114", "SC-000117", "SC-000119", "SC-000124", "SC-000126", "SC-000127"]);
    const cIds = new Set(["SC-000071", "SC-000106", "SC-000107"]);
    const dIds = new Set(["SC-000072", "SC-000073", "SC-000074", "SC-000075", "SC-000076", "SC-000105"]);
    const task9Ids = new Set([
      ...Array.from({ length: 14 }, (_, index) => `SC-${String(60 + index).padStart(6, "0")}`).filter((id) => id !== "SC-000064"),
      "SC-000074", "SC-000075", "SC-000076", "SC-000093", "SC-000094", "SC-000097", "SC-000098",
      ...Array.from({ length: 6 }, (_, index) => `SC-${String(100 + index).padStart(6, "0")}`),
      ...Array.from({ length: 9 }, (_, index) => `SC-${String(106 + index).padStart(6, "0")}`),
      "SC-000117", "SC-000119", "SC-000120", "SC-000122", "SC-000123", "SC-000124", "SC-000125", "SC-000126", "SC-000127",
    ]);
    const rowById = new Map(realBaseLedger.rows.map((row: any) => [row.id, row]));
    const decisions = (reviewArtifact as any).decisions.filter((decision: any) => task9Ids.has(decision.id));
    const ordinalTotal = (ids: Set<string>) => [...ids].reduce((total, id) => total + (rowById.get(id) as any).assertionCount, 0);
    const bIds = new Set(decisions.map((decision: any) => decision.id).filter((id: string) => !aIds.has(id) && !cIds.has(id) && !dIds.has(id)));

    expect(decisions).toHaveLength(44);
    expect([aIds.size, ordinalTotal(aIds)]).toEqual([9, 63]);
    expect([bIds.size, ordinalTotal(bIds)]).toEqual([26, 212]);
    expect([cIds.size, ordinalTotal(cIds)]).toEqual([3, 21]);
    expect([dIds.size, ordinalTotal(dIds)]).toEqual([6, 45]);

    for (const decision of decisions) {
      const groups = decision.resolution?.subgroups ?? [decision.resolution];
      const owners = groups.flatMap((group: any) => group?.replacementIds ?? []);
      if (aIds.has(decision.id)) {
        expect(decision.class).toBe("B2_NEW_CHEAP_BEHAVIOR");
        expect(owners.every((owner: string) => owner.startsWith("test:vitest:src/lib/analysis-") && owner.includes(".behavior.test.ts#"))).toBe(true);
      } else if (dIds.has(decision.id)) {
        expect(decision.class).toBe("D3_NON_OBSERVABLE_VISUAL");
        expect(decision.resolution.disposition).toBe("delete");
        expect(decision.resolution.deletionReason).toContain("D3_NON_OBSERVABLE_VISUAL");
        expect(owners).toEqual([]);
      } else {
        expect(decision.class).toBe("B2_NEW_CHEAP_BEHAVIOR");
        expect(owners.length).toBeGreaterThan(0);
        expect(owners.every((owner: string) => owner.startsWith("test:vitest:") && owner.includes(".behavior.component.test.ts#"))).toBe(true);
      }
    }

    for (const id of ["SC-000071", "SC-000106"]) {
      const row = rowById.get(id) as any;
      const decision = decisions.find((item: any) => item.id === id);
      expect(Object.keys(decision.resolution)).toEqual(["subgroups"]);
      const ordinals = decision.resolution.subgroups.flatMap((group: any) => group.assertionOrdinals).sort((left: number, right: number) => left - right);
      expect(ordinals).toEqual(Array.from({ length: row.assertionCount }, (_, index) => index + 1));
      expect(new Set(ordinals).size).toBe(row.assertionCount);
      expect(decision.resolution.subgroups.every((group: any) => typeof group.invariant === "string" && group.invariant.length > 0)).toBe(true);
      expect(decision.resolution.subgroups.filter((group: any) => group.disposition === "delete").every((group: any) => group.deletionReason.includes("D3_NON_OBSERVABLE_VISUAL"))).toBe(true);
    }

    const companionLayoutDecision = decisions.find((item: any) => item.id === "SC-000076");
    const companionLayoutReason = "D3_NON_OBSERVABLE_VISUAL: Absence of companion-width-specific inner layout rules in Chat, Chunks, and Runs is a non-normative CSS/visual constraint with no truthful jsdom seam, and the approved correction adds no Playwright owner.";
    expect(companionLayoutDecision).toEqual({
      id: "SC-000076",
      sourceHash: "9feecb79fe7819496b19f4b299ebfc19de69a990e0b4595443108ad496fcb4df",
      authorRunId: "redisposition-task-4-author-run",
      class: "D3_NON_OBSERVABLE_VISUAL",
      reason: companionLayoutReason,
      resolution: {
        disposition: "delete",
        deletionReason: companionLayoutReason,
      },
    });

    expect((reviewArtifact as any).forecast.futureOwnersByMechanism).toEqual({
      node: { rows: 139, assertionOrdinals: 787 },
      cargo: { rows: 18, assertionOrdinals: 147 },
      jsdom: { rows: 123, assertionOrdinals: 864 },
      playwright: { rows: 3, assertionOrdinals: 21 },
    });
    expect((reviewArtifact as any).forecast.afterByDisposition).toEqual({ behavior: 279, delete: 153, mixed: 4 });

    const input: any = {
      artifact: reviewArtifact,
      baseLedger: realBaseLedger,
      currentLedger,
      baseTrackedPaths: realBaseTrackedPaths,
      currentPresentPaths,
      evidenceBytes: loaded.evidenceBytes,
    };
    expect(validateReview(input)).toEqual([]);

    const mutatedReason = clone(reviewArtifact) as any;
    mutatedReason.decisions.find((item: any) => item.id === "SC-000060").reason += " mutated";
    expect(validateReview({ ...input, artifact: mutatedReason })).toContain("Task 9: approved amendment table digest mismatch");

    const old060ReplacementIds = [
      "test:vitest:src/lib/analysis-compact-source-rail.behavior.test.ts#compact analysis source rail > keeps the collapsed rail compact and source-scoped",
    ];
    const partialArtifact = clone(reviewArtifact) as any;
    const partialDecision = partialArtifact.decisions.find((item: any) => item.id === "SC-000060");
    partialDecision.ownerEvidence = clone(old060ReplacementIds);
    partialDecision.resolution = { disposition: "behavior", replacementIds: clone(old060ReplacementIds) };
    expect(validateReview({ ...input, artifact: partialArtifact })).toContain("Task 9: approved amendment table digest mismatch");

    const correctedLedger = applyReview(input).ledger;
    expect(validateReview({ ...input, currentLedger: correctedLedger })).toEqual([]);
    const partialLedger = clone(correctedLedger) as any;
    const partialLedgerRow = partialLedger.rows.find((item: any) => item.id === "SC-000060");
    partialLedgerRow.disposition = "behavior";
    partialLedgerRow.replacementIds = clone(old060ReplacementIds);
    expect(validateReview({ ...input, currentLedger: partialLedger })).toContain("ledger: Task 9 resolutions must equal the exact pre-correction or corrected state");
  }, 30_000);

  it("pins the exact Task 9 decision shapes including optional and subgroup fields", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, currentPresentPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadCurrentPresentTrackedPaths({ repoRoot }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const input: any = {
      artifact: reviewArtifact,
      baseLedger: realBaseLedger,
      currentLedger,
      baseTrackedPaths: realBaseTrackedPaths,
      currentPresentPaths,
      evidenceBytes: loaded.evidenceBytes,
    };
    const mutations: Array<[string, (artifact: any) => void]> = [
      ["inapplicable criticalityRef", (artifact) => {
        artifact.decisions.find((item: any) => item.id === "SC-000060").criticalityRef = "TESTING_SOURCE_CONTRACT_REPLACEMENT";
      }],
      ["inapplicable lostBehavior", (artifact) => {
        artifact.decisions.find((item: any) => item.id === "SC-000060").lostBehavior = [{ assertionOrdinals: [1], behavior: "invented loss" }];
      }],
      ["arbitrary decision field", (artifact) => {
        artifact.decisions.find((item: any) => item.id === "SC-000060").arbitraryField = true;
      }],
      ["omitted required decision field", (artifact) => {
        delete artifact.decisions.find((item: any) => item.id === "SC-000060").ownerEvidence;
      }],
      ["extra subgroup field", (artifact) => {
        artifact.decisions.find((item: any) => item.id === "SC-000071").resolution.subgroups[0].arbitraryField = true;
      }],
      ["omitted required subgroup field", (artifact) => {
        delete artifact.decisions.find((item: any) => item.id === "SC-000071").resolution.subgroups[0].invariant;
      }],
    ];

    for (const [label, mutate] of mutations) {
      const artifact = clone(reviewArtifact) as any;
      mutate(artifact);
      expect.soft(validateReview({ ...input, artifact }), label).toContain("Task 9: exact decision shape mismatch");
    }
  }, 30_000);

  it("recomputes final disposition forecasts and accepted loss from decisions", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const input: any = { baseLedger: realBaseLedger, currentLedger: realBaseLedger, baseTrackedPaths: realBaseTrackedPaths, evidenceBytes: loaded.evidenceBytes };

    const forecast = clone(reviewArtifact) as any;
    forecast.forecast.beforeByDisposition.behavior -= 1;
    expect(validateReview({ ...input, artifact: forecast })).toContain("forecast: beforeByDisposition does not match frozen decisions");

    const loss = clone(reviewArtifact) as any;
    loss.acceptedLoss.rows = 1;
    expect(validateReview({ ...input, artifact: loss })).toContain("acceptedLoss: summary does not match D5 decisions");
  });

  it("pins every real base-open row in the static partial-review artifact", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
    ]);
    const baseOpenRows = realBaseLedger.rows.filter((item: any) =>
      realBaseTrackedPaths.has(item.path) && !["SC-000355", "SC-000366"].includes(item.id));
    const baseOpenById = new Map(baseOpenRows.map((item: any) => [item.id, item]));
    const closedRows = realBaseLedger.rows.filter((item: any) => !baseOpenById.has(item.id));
    const decisions = reviewArtifact.decisions;
    const decisionIds = decisions.map((item) => item.id);
    const mandatoryP0Ids = ["SC-000420", "SC-000511", "SC-000512", "SC-000513", "SC-000514", "SC-000515", "SC-000555", "SC-000556", "SC-000557", "SC-000558", "SC-000559", "SC-000560"];

    expect(decisions).toHaveLength(436);
    expect(baseOpenRows).toHaveLength(436);
    expect(closedRows).toHaveLength(235);
    expect(new Set(decisionIds).size).toBe(436);
    expect(decisionIds).toEqual([...decisionIds].sort((left, right) => Number(left.slice(3)) - Number(right.slice(3))));
    expect(decisions.map((item) => [item.id, item.sourceHash])).toEqual(baseOpenRows
      .sort((left: any, right: any) => Number(left.id.slice(3)) - Number(right.id.slice(3)))
      .map((item: any) => [item.id, item.sourceHash]));
    expect(reviewArtifact.protectedRows.map((item) => item.id)).toEqual(expect.arrayContaining(mandatoryP0Ids));
    expect(decisions.reduce<Record<string, number>>((counts, item) => {
      counts[item.class] = (counts[item.class] ?? 0) + 1;
      return counts;
    }, {})).toEqual({
      B2_NEW_CHEAP_BEHAVIOR: 275,
      B3_PROTECTED_EXPENSIVE_BEHAVIOR: 8,
      D1_COMPLETED_HISTORY_ONLY: 6,
      D2_IMPLEMENTATION_SHAPE: 123,
      D3_NON_OBSERVABLE_VISUAL: 16,
      D4_DUPLICATE_EVIDENCE: 8,
    });
    expect(reviewArtifact.protectedRows).toHaveLength(14);
    expect(reviewArtifact.independentReview.blindResults).toHaveLength(95);
    expect(reviewArtifact.independentReview.mandatoryCohorts.find((item) => item.name === "emerging-b3")?.rowIds).toEqual([
      "SC-000015", "SC-000023", "SC-000312", "SC-000344", "SC-000385", "SC-000387", "SC-000389", "SC-000420",
    ]);
    expect(reviewArtifact.independentReview.calibrations.find((item) => item.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR")?.rowIds).toEqual([
      "SC-000312", "SC-000344", "SC-000385",
    ]);
    expect(reviewArtifact.independentReview.calibrations.find((item) => item.class === "D3_NON_OBSERVABLE_VISUAL")?.rowIds).toEqual([
      "SC-000323", "SC-000352", "SC-000353",
    ]);
    expect(reviewArtifact.independentReview.calibrations.find((item) => item.class === "B1_EXISTING_BEHAVIOR_OWNER")?.rowIds).toEqual([]);
    expect(decisions.find((item) => item.id === "SC-000323")?.resolution?.deletionReason).toBe(
      "D3 non-observable-visual: Visible active-row focus/selection styling is a non-normative visual affordance with no truthful non-browser seam.",
    );
    expect(decisions.find((item) => item.id === "SC-000352")?.resolution?.deletionReason).toBe(
      "D3 non-observable-visual: Two-line title clipping is a non-normative rendered-layout detail with no truthful non-browser seam.",
    );
    expect(decisions.find((item) => item.id === "SC-000353")?.resolution?.deletionReason).toBe(
      "D3 non-observable-visual: Cell centering and overflow ellipsis are non-normative rendered-layout details with no truthful non-browser seam.",
    );
  });

  it("derives timing forecasts from retained measurements and rejects invalid baseline or jsdom proposals", () => {
    expect(validateReview(review())).toEqual([]);

    const wrongMedian = artifactFor();
    wrongMedian.mechanisms.componentReplacement.medianSeconds = 28;
    expect(validateReview(review({ artifact: wrongMedian }))).toContain("mechanisms.componentReplacement: medianSeconds must equal the retained median");

    const wrongForecast = artifactFor();
    wrongForecast.forecast.scalablePerRow = 1;
    expect(validateReview(review({ artifact: wrongForecast }))).toContain("forecast: scalablePerRow does not match retained timing arithmetic");

    for (const field of ["upperBoundPerRow", "upperForecastSeconds", "scalableForecastSeconds", "netGateForecastSeconds", "removableLegacySeconds"] as const) {
      const perturbed = artifactFor();
      perturbed.forecast[field] += 1;
      expect(validateReview(review({ artifact: perturbed }))).toContain(`forecast: ${field} does not match retained timing arithmetic`);
    }
    const wrongPercent = artifactFor();
    wrongPercent.forecast.netGateForecastPercent = 1;
    expect(validateReview(review({ artifact: wrongPercent }))).toContain("forecast: netGateForecastPercent does not match the fresh verify baseline");
    const wrongDominance = artifactFor();
    wrongDominance.forecast.legacyDominatesFullUnitCeiling = !wrongDominance.forecast.legacyDominatesFullUnitCeiling;
    expect(validateReview(review({ artifact: wrongDominance }))).toContain("forecast: legacyDominatesFullUnitCeiling mismatch");
    const wrongGrouping = artifactFor();
    wrongGrouping.forecast.futureOwnersByMechanism = {};
    expect(validateReview(review({ artifact: wrongGrouping }))).toContain("forecast: futureOwnersByMechanism does not match owner grouping");

    const cargoDecisions = withFirstDecision({
      ...artifactFor().decisions[0],
      resolution: { disposition: "behavior", replacementIds: ["test:cargo:extractum::tests::future_behavior_owner"] },
    });
    const cargoArtifact = artifactFor(cargoDecisions);
    cargoArtifact.forecast.futureOwnersByMechanism = {
      ...cargoArtifact.forecast.futureOwnersByMechanism,
      cargo: { rows: 1, assertionOrdinals: 2 },
    };
    expect(validateReview(review({ artifact: cargoArtifact }))).toEqual([]);

    const overCeiling = artifactFor();
    overCeiling.forecast.proposedNewJsdomRows = 47;
    expect(validateReview(review({ artifact: overCeiling }))).toContain("forecast: proposedNewJsdomRows exceeds replacementUnitCeiling");

    const unsortedBaseline = artifactFor();
    unsortedBaseline.mechanisms.legacyOwner.baseFiles.reverse();
    expect(validateReview(review({ artifact: unsortedBaseline }))).toContain("mechanisms.legacyOwner: baseFiles must be sorted normalized repository-relative paths");

    const playwrightProposal = artifactFor();
    playwrightProposal.decisions[0].resolution.replacementIds = ["test:playwright:future/proposal.spec.ts"];
    expect(validateReview(review({ artifact: playwrightProposal }))).toContain("SC-000001: browser owner requires program amendment");
  });

  it("derives the execution owner forecast from the frozen component path convention", async () => {
    const decisions = defaultDecisions();
    decisions.find((decision) => decision.id === "SC-000001").resolution.replacementIds = [
      "test:vitest:src/lib/example.behavior.test.ts#unit node behavior owner",
    ];
    decisions.find((decision) => decision.id === "SC-000002").resolution.replacementIds = [
      "test:vitest:src/lib/components/example.behavior.test.ts#component behavior owner",
    ];
    const artifact = artifactFor(decisions);

    expect(artifact.forecast.futureOwnersByMechanism).toEqual({
      node: { rows: 13, assertionOrdinals: 26 },
      jsdom: { rows: 1, assertionOrdinals: 2 },
    });
    expect(artifact.forecast.proposedNewJsdomRows).toBe(1);
    expect(artifact.forecast.proposedNewJsdomOrdinals).toBe(2);
    expect(validateReview(review({ artifact }))).toEqual([]);

    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [baseLedger, baseTrackedPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const executionForecast = (reviewArtifact as any).forecast.futureOwnersByMechanism;

    expect(executionForecast).toEqual({
      node: { rows: 139, assertionOrdinals: 787 },
      cargo: { rows: 18, assertionOrdinals: 147 },
      jsdom: { rows: 123, assertionOrdinals: 864 },
      playwright: { rows: 3, assertionOrdinals: 21 },
    });
    expect((reviewArtifact as any).forecast.proposedNewJsdomRows).toBe(0);
    expect((reviewArtifact as any).forecast.proposedNewJsdomOrdinals).toBe(0);
    expect(loaded.issues).toEqual([]);
    expect(validateReview({ artifact: reviewArtifact, baseLedger, currentLedger: baseLedger, baseTrackedPaths, evidenceBytes: loaded.evidenceBytes })).toEqual([]);
  });

  it("allows only the exact approved three-row Playwright mapping outside jsdom timing", () => {
    const approved = reviewWithApprovedPlaywright();
    expect(validateReview(approved)).toEqual([]);
    expect(approved.artifact.forecast.futureOwnersByMechanism.playwright).toEqual({ rows: 3, assertionOrdinals: 21 });

    const alteredOwner = clone(approved.artifact);
    alteredOwner.decisions.find((item: any) => item.id === "SC-000312").resolution.replacementIds = ["test:playwright:e2e/app-shell-responsive.spec.ts#wrong-owner"];
    expect(validateReview({ ...approved, artifact: alteredOwner })).toContain("SC-000312: browser owner requires program amendment");

    const wrongClass = clone(approved.artifact);
    wrongClass.decisions.find((item: any) => item.id === "SC-000344").class = "B2_NEW_CHEAP_BEHAVIOR";
    expect(validateReview({ ...approved, artifact: wrongClass })).toContain("SC-000344: approved browser owner requires B3_PROTECTED_EXPENSIVE_BEHAVIOR");

    const wrongCitation = clone(approved.artifact);
    wrongCitation.decisions.find((item: any) => item.id === "SC-000385").criticalityRef = "AGENTS_SECURITY";
    expect(validateReview({ ...approved, artifact: wrongCitation })).toContain("SC-000385: approved browser owner requires TESTING_BROWSER_COMPONENT_OWNERSHIP");

    const missingOwner = clone(approved.artifact);
    missingOwner.decisions.find((item: any) => item.id === "SC-000385").resolution = { disposition: "behavior", replacementIds: ["test:cargo:extractum::tests::wrong_seam"] };
    expect(validateReview({ ...approved, artifact: missingOwner })).toContain("forecast: approved Playwright owner mapping must equal 3 rows / 21 assertion ordinals");

    const extraOwner = clone(approved.artifact);
    extraOwner.decisions[0].resolution.replacementIds.push("test:playwright:e2e/unapproved.spec.ts#extra-owner");
    expect(validateReview({ ...approved, artifact: extraOwner })).toContain("SC-000001: browser owner requires program amendment");

    const wrongCount = clone(approved.artifact);
    wrongCount.forecast.futureOwnersByMechanism.playwright.assertionOrdinals = 20;
    expect(validateReview({ ...approved, artifact: wrongCount })).toContain("forecast: futureOwnersByMechanism does not match owner grouping");
  });

  it("fails closed for exact verify evidence, immutable catalogs, and the pinned legacy listing", () => {
    const alteredFailure = artifactFor();
    alteredFailure.mechanisms.verify.observations[0].failure = "another failure";
    expect(validateReview(review({ artifact: alteredFailure }))).toContain("mechanisms.verify: observations must match the approved execution record");

    const reorderedObservations = artifactFor();
    reorderedObservations.mechanisms.verify.observations.reverse();
    expect(validateReview(review({ artifact: reorderedObservations }))).toContain("mechanisms.verify: observations must match the approved execution record");

    const missingBaseline = artifactFor();
    delete missingBaseline.mechanisms.verify.successfulBaseline;
    expect(validateReview(review({ artifact: missingBaseline }))).toContain("mechanisms.verify: successfulBaseline must match the approved execution record");

    const changedReasonCatalog = artifactFor();
    changedReasonCatalog.reasonClasses[0].rule = "rewritten";
    expect(validateReview(review({ artifact: changedReasonCatalog }))).toContain("reasonClasses: must equal the approved catalog");

    const changedCriticalityCatalog = artifactFor();
    changedCriticalityCatalog.criticalitySources[0].citation = "rewritten";
    expect(validateReview(review({ artifact: changedCriticalityCatalog }))).toContain("criticalitySources: must equal the approved catalog");

    const changedLegacyListing = artifactFor();
    changedLegacyListing.mechanisms.legacyOwner.baseFiles[0] = "aaa.test.ts";
    expect(validateReview(review({ artifact: changedLegacyListing }))).toContain("mechanisms.legacyOwner: baseFiles do not match the pinned legacy listing");

    const wrongCeiling = artifactFor();
    wrongCeiling.forecast.replacementUnitCeiling = 45;
    expect(validateReview(review({ artifact: wrongCeiling }))).toContain("forecast: replacementUnitCeiling must equal 46");

    const wrongScalablePercent = artifactFor();
    wrongScalablePercent.forecast.scalableForecastPercent += 1;
    expect(validateReview(review({ artifact: wrongScalablePercent }))).toContain("forecast: scalableForecastPercent does not match the fresh verify baseline");
  });

  it("pins the base and scopes every independently derived open row", () => {
    expect(validateReview(review())).toEqual([]);
    expect(validateReview(review({ artifact: artifactFor([]) }))).toContain("scope: expected one decision for every base-open row");

    const wrongBase = artifactFor(); wrongBase.reviewBaseCommit = "0".repeat(40);
    expect(validateReview(review({ artifact: wrongBase }))).toContain(`artifact: reviewBaseCommit must be ${REVIEW_BASE}`);
    const wrongFrozen = artifactFor(); wrongFrozen.ledgerFrozenAtCommit = "0".repeat(40);
    expect(validateReview(review({ artifact: wrongFrozen }))).toContain("artifact: ledgerFrozenAtCommit must equal base ledger frozenAtCommit");
    const duplicate = artifactFor(); duplicate.decisions.push(clone(duplicate.decisions[0]));
    expect(validateReview(review({ artifact: duplicate }))).toContain("scope: duplicate decision id: SC-000001");
    const extra = artifactFor(); extra.decisions[1].id = "SC-000355";
    expect(validateReview(review({ artifact: extra }))).toContain("scope: decision is not a base-open row: SC-000355");
    const drift = artifactFor(); drift.decisions[0].sourceHash = "0".repeat(64);
    expect(validateReview(review({ artifact: drift }))).toContain("SC-000001: decision sourceHash drift");
  });

  it("fails closed for tracked-path exceptions and frozen bytes", () => {
    expect(validateReview(review({ baseTrackedPaths: new Set(["src/open-one.test.ts"]) }))).toContain("scope: expected 14 base-open rows, found 1");
    const exceptions = artifactFor(); exceptions.scope.pathPresentClosedRowIds = ["SC-000366", "SC-000355"];
    expect(validateReview(review({ artifact: exceptions }))).toContain("scope: pathPresentClosedRowIds must equal SC-000355, SC-000366");
    const badClosed = clone(baseLedger); badClosed.rows[2].invariant = "rewritten";
    expect(validateReview(review({ currentLedger: badClosed }))).toContain("SC-000355: immutable row field drift");
    const badDigest = artifactFor(); badDigest.scope.closedRowsDigest = "0".repeat(64);
    expect(validateReview(review({ artifact: badDigest }))).toContain("scope: closedRowsDigest mismatch");
    const badEnvelope = clone(baseLedger); badEnvelope.sourceReaderExceptions.push({ path: "src/x", sourceRange: "1:1-1:2", reason: "x", owner: "x" });
    expect(validateReview(review({ currentLedger: badEnvelope }))).toContain("ledger: sourceReaderExceptions changed");
  });

  it("allows only the deletion-coupled removal of the exact sole source-reader exception after its reader path is absent", () => {
    const exactException = {
      path: "src/lib/analysis-migration-fixture-contract.test.ts",
      sourceRange: "14:13-14:74",
      reason: "test-only migration schema candidates are intentional fixture authorities",
      owner: "analysis migration fixture contract",
    };
    const lifecycleBase = clone(baseLedger);
    lifecycleBase.sourceReaderExceptions = [exactException];
    const absentPaths = new Set(baseTrackedPaths);
    absentPaths.delete(exactException.path);
    const presentPaths = new Set([...baseTrackedPaths, exactException.path]);
    const removed = clone(lifecycleBase);
    removed.sourceReaderExceptions = [];
    expect(validateReview(review({ baseLedger: lifecycleBase, currentLedger: removed, currentPresentPaths: absentPaths }))).toEqual([]);

    expect(validateReview(review({ baseLedger: lifecycleBase, currentLedger: removed, currentPresentPaths: presentPaths }))).toContain("ledger: sourceReaderExceptions changed");

    const added = clone(lifecycleBase);
    added.sourceReaderExceptions.push({ ...exactException, path: "src/other-reader.test.ts" });
    expect(validateReview(review({ baseLedger: lifecycleBase, currentLedger: added, currentPresentPaths: absentPaths }))).toContain("ledger: sourceReaderExceptions changed");

    const mutated = clone(lifecycleBase);
    mutated.sourceReaderExceptions[0].owner = "mutated owner";
    expect(validateReview(review({ baseLedger: lifecycleBase, currentLedger: mutated, currentPresentPaths: absentPaths }))).toContain("ledger: sourceReaderExceptions changed");

    const reorderedBase = clone(baseLedger);
    reorderedBase.sourceReaderExceptions = [exactException, { ...exactException, path: "src/other-reader.test.ts" }];
    const reordered = clone(reorderedBase);
    reordered.sourceReaderExceptions.reverse();
    expect(validateReview(review({ baseLedger: reorderedBase, currentLedger: reordered, currentPresentPaths: absentPaths }))).toContain("ledger: sourceReaderExceptions changed");

    const rowMutation = clone(removed);
    rowMutation.rows[0].replacementIds = ["test:vitest:src/mutated.test.ts#owner"];
    expect(validateReview(review({ baseLedger: lifecycleBase, currentLedger: rowMutation, currentPresentPaths: absentPaths }))).toContain("ledger: rows changed outside approved review apply");
  });

  it("treats only ENOENT as a missing tracked path and reports every other access failure", async () => {
    const denied = Object.assign(new Error("access denied"), { code: "EACCES" });
    await expect(loadCurrentPresentTrackedPaths({
      repoRoot: process.cwd(),
      trackedPaths: ["src/protected-reader.test.ts"],
      accessFile: async () => { throw denied; },
    })).rejects.toThrow("src/protected-reader.test.ts");

    const missing = Object.assign(new Error("missing"), { code: "ENOENT" });
    await expect(loadCurrentPresentTrackedPaths({
      repoRoot: process.cwd(),
      trackedPaths: ["src/missing-reader.test.ts"],
      accessFile: async () => { throw missing; },
    })).resolves.toEqual(new Set());
  });

  it("requires current open resolutions to remain all-or-nothing before or after apply", () => {
    const reordered = clone(baseLedger); [reordered.rows[0], reordered.rows[1]] = [reordered.rows[1], reordered.rows[0]];
    expect(validateReview(review({ currentLedger: reordered }))).toContain("ledger: current row ID order drift from review base");
    const mutated = clone(baseLedger); mutated.rows[0].replacementIds = ["test:vitest:src/mutated.test.ts#owner"];
    expect(validateReview(review({ currentLedger: mutated }))).toContain("ledger: base-open resolutions must collectively equal either the review-base or fully-applied state");
    const replaced = clone(baseLedger); replaced.rows[0].subgroups = [{ assertionOrdinals: [1, 2], invariant: "replacement", disposition: "behavior", replacementIds: ["test:vitest:src/replaced.test.ts#owner"] }]; delete (replaced.rows[0] as any).disposition; delete (replaced.rows[0] as any).replacementIds;
    expect(validateReview(review({ currentLedger: replaced }))).toContain("ledger: base-open resolutions must collectively equal either the review-base or fully-applied state");

    const d3 = artifactFor(); d3.decisions[0] = {
      ...d3.decisions[0], class: "D3_NON_OBSERVABLE_VISUAL", reason: "D3_NON_OBSERVABLE_VISUAL removes this visual-only assertion.",
      resolution: { disposition: "delete", deletionReason: "D3_NON_OBSERVABLE_VISUAL removes the visual-only assertion." },
    };
    expect(validateReview(review({ artifact: artifactFor(d3.decisions) }))).toEqual([]);

    const frozenPlaywrightBase = clone(baseLedger); frozenPlaywrightBase.rows[1].replacementIds = ["test:playwright:legacy/frozen.spec.ts"];
    const frozenPlaywrightArtifact = artifactFor(); frozenPlaywrightArtifact.scope.closedRowsDigest = sha256Text(canonicalJson(frozenPlaywrightBase.rows.filter((item) => !baseOpenIds.has(item.id))));
    frozenPlaywrightArtifact.decisions[1] = {
      ...frozenPlaywrightArtifact.decisions[1], class: "D3_NON_OBSERVABLE_VISUAL", criticalityRef: undefined,
      reason: "D3_NON_OBSERVABLE_VISUAL removes the obsolete visual assertion.",
      resolution: { disposition: "delete", deletionReason: "D3_NON_OBSERVABLE_VISUAL removes the obsolete visual assertion." },
    };
    expect(validateReview(review({ artifact: artifactFor(frozenPlaywrightArtifact.decisions), baseLedger: frozenPlaywrightBase, currentLedger: clone(frozenPlaywrightBase) }))).toEqual([]);

    const proposedPlaywright = artifactFor(); proposedPlaywright.decisions[0].resolution = { disposition: "behavior", replacementIds: ["test:playwright:future/new.spec.ts"] };
    const simulated = applyReview(review({ artifact: proposedPlaywright }));
    expect(validateReview(review({ artifact: proposedPlaywright, currentLedger: simulated.ledger }))).toContain("SC-000001: browser owner requires program amendment");
  });

  it("enforces the class ladder, resolution allowlist, criticality, owner, and D5 loss rules", () => {
    for (const [reasonClass, disposition] of [
      ["D1_COMPLETED_HISTORY_ONLY", "delete"], ["D2_IMPLEMENTATION_SHAPE", "delete"], ["D3_NON_OBSERVABLE_VISUAL", "delete"],
      ["D4_DUPLICATE_EVIDENCE", "delete"], ["D5_ACCEPTED_LOSS", "delete"], ["A1_EXISTING_STRUCTURED_OWNER", "architecture"],
      ["T1_EXISTING_TOOL_OWNER", "tool_owned"], ["B1_EXISTING_BEHAVIOR_OWNER", "behavior"], ["B2_NEW_CHEAP_BEHAVIOR", "behavior"],
      ["B3_PROTECTED_EXPENSIVE_BEHAVIOR", "behavior"],
    ] as const) {
      const decision: any = { id: "SC-000001", sourceHash: baseLedger.rows[0].sourceHash, authorRunId: "author-run", class: reasonClass, reason: `${reasonClass} row-specific reason` };
      if (disposition === "delete") {
        decision.resolution = { disposition, deletionReason: `${reasonClass} specifically accepts this historical behavior.` };
        if (reasonClass === "D5_ACCEPTED_LOSS") decision.lostBehavior = [{ assertionOrdinals: [1, 2], behavior: "the two asserted legacy behaviors" }];
        if (reasonClass === "D4_DUPLICATE_EVIDENCE") decision.ownerEvidence = ["test:vitest:src/closed.test.ts#owner"];
      } else {
        decision.resolution = { disposition, replacementIds: ["test:vitest:src/future.test.ts#owner"] };
        if (["A1_EXISTING_STRUCTURED_OWNER", "T1_EXISTING_TOOL_OWNER", "B1_EXISTING_BEHAVIOR_OWNER"].includes(reasonClass)) {
          decision.resolution = { disposition, replacementIds: ["test:vitest:src/closed.test.ts#owner"] };
          decision.ownerEvidence = ["test:vitest:src/closed.test.ts#owner"];
        }
        if (reasonClass === "B3_PROTECTED_EXPENSIVE_BEHAVIOR") decision.criticalityRef = "AGENTS_SECURITY";
      }
      const decisions = withFirstDecision(decision);
      expect(validateReview(review({ artifact: artifactFor(decisions) }))).toEqual([]);
    }

    const unclassified = artifactFor(); unclassified.decisions[0] = { ...unclassified.decisions[0], class: "UNCLASSIFIED", reason: undefined, resolution: undefined };
    expect(validateReview(review({ artifact: artifactFor(unclassified.decisions) }))).toContain("SC-000001: UNCLASSIFIED blocks apply");
    const classOnly = artifactFor(); classOnly.decisions[0] = { ...classOnly.decisions[0], class: "D1_COMPLETED_HISTORY_ONLY", reason: "D1_COMPLETED_HISTORY_ONLY", resolution: { disposition: "delete", deletionReason: "D1_COMPLETED_HISTORY_ONLY" } };
    expect(validateReview(review({ artifact: artifactFor(classOnly.decisions) }))).toContain("SC-000001: deletion reason must contain row-specific text");
    const wrongResolution = artifactFor(); wrongResolution.decisions[0].resolution = { disposition: "delete", replacementIds: ["test:vitest:x#y"], deletionReason: "B2_NEW_CHEAP_BEHAVIOR wrong" };
    expect(validateReview(review({ artifact: artifactFor(wrongResolution.decisions) }))).toContain("SC-000001: B2_NEW_CHEAP_BEHAVIOR requires behavior disposition");
    const missingOwner = artifactFor(); missingOwner.decisions[0] = { ...missingOwner.decisions[0], class: "B1_EXISTING_BEHAVIOR_OWNER", reason: "B1_EXISTING_BEHAVIOR_OWNER named existing owner" };
    expect(validateReview(review({ artifact: artifactFor(missingOwner.decisions) }))).toContain("SC-000001: B1_EXISTING_BEHAVIOR_OWNER requires exact owner evidence");
    const d5 = artifactFor(); d5.decisions[0] = { ...d5.decisions[0], class: "D5_ACCEPTED_LOSS", reason: "D5_ACCEPTED_LOSS accepts this exact loss", resolution: { disposition: "delete", deletionReason: "D5_ACCEPTED_LOSS accepts only the first assertion" }, lostBehavior: [{ assertionOrdinals: [1], behavior: "first" }] };
    expect(validateReview(review({ artifact: artifactFor(d5.decisions) }))).toContain("SC-000001: D5 lostBehavior must cover exactly its deleted assertion ordinals");
  });

  it("checks blind review identity, deterministic samples, and mixed partitions", () => {
    const sameRun = artifactFor(); sameRun.independentReview.reviewer.agentTaskId = "author-run";
    expect(validateReview(review({ artifact: sameRun }))).toContain("independentReview: authorRunId and reviewer agentTaskId must differ");
    const reusedRun = artifactFor(); reusedRun.independentReview.reviewer.reviewerRunId = "author-run";
    expect(validateReview(review({ artifact: reusedRun }))).toContain("independentReview: authorRunId, reviewer agentTaskId, and reviewerRunId must be distinct");
    const invalidPacketHash = artifactFor(); invalidPacketHash.independentReview.reviewer.packetSha256 = "not-a-sha256";
    expect(validateReview(review({ artifact: invalidPacketHash }))).toContain("independentReview: reviewer packetSha256 must be lowercase SHA-256");
    const invalidOutputHash = artifactFor(); invalidOutputHash.independentReview.reviewer.outputSha256 = "C".repeat(64);
    expect(validateReview(review({ artifact: invalidOutputHash }))).toContain("independentReview: reviewer outputSha256 must be lowercase SHA-256");
    const reusedEvidenceHash = artifactFor(); reusedEvidenceHash.independentReview.reviewer.outputSha256 = reusedEvidenceHash.independentReview.reviewer.packetSha256;
    expect(validateReview(review({ artifact: reusedEvidenceHash }))).toContain("independentReview: packet and output SHA-256 values must differ");
    const missingEvidencePath = artifactFor(); delete missingEvidencePath.independentReview.reviewer.packetPath;
    expect(validateReview(review({ artifact: missingEvidencePath }))).toContain("independentReview: reviewer packetPath and outputPath are required");
    const mismatchedEvidencePath = artifactFor(); mismatchedEvidencePath.independentReview.reviewer.packetPath = ".superpowers/sdd/packet.wrong.json";
    expect(validateReview(review({ artifact: mismatchedEvidencePath }))).toContain("independentReview: reviewer evidence paths must contain their SHA-256 values");
    const staleBlind = artifactFor(); staleBlind.independentReview.blindResults[0].sourceHash = "0".repeat(64);
    expect(validateReview(review({ artifact: staleBlind }))).toContain("SC-000001: blind sourceHash drift");
    const staleSample = artifactFor(); staleSample.independentReview.deterministicSample.population = [];
    expect(validateReview(review({ artifact: staleSample }))).toContain("independentReview: deterministic sample population is stale");

    const mixed = artifactFor(); mixed.decisions[0].resolution = { subgroups: [
      { assertionOrdinals: [1], invariant: "first invariant", disposition: "delete", deletionReason: "B2_NEW_CHEAP_BEHAVIOR wrong class" },
      { assertionOrdinals: [2], invariant: "second invariant", disposition: "behavior", replacementIds: ["test:vitest:src/future.test.ts#two"] },
    ] };
    expect(validateReview(review({ artifact: artifactFor(mixed.decisions) }))).toContain("SC-000001: B2_NEW_CHEAP_BEHAVIOR requires behavior disposition");
    const brokenMixed = artifactFor(); brokenMixed.decisions[0].resolution = { subgroups: [
      { assertionOrdinals: [1, 1], invariant: "first", disposition: "behavior", replacementIds: ["test:vitest:src/future.test.ts#one"] },
      { assertionOrdinals: [3], invariant: "second", disposition: "behavior", replacementIds: ["test:vitest:src/future.test.ts#two"] },
    ] };
    expect(validateReview(review({ artifact: artifactFor(brokenMixed.decisions) }))).toEqual(expect.arrayContaining([
      "SC-000001: duplicate subgroup assertion ordinal: 1", "SC-000001: invalid subgroup assertion ordinal: 3", "SC-000001: incomplete subgroup assertion ordinals",
    ]));
  });

  it("requires the exact mandatory P0 catalog and fixed criticality catalog", () => {
    const p0 = p0Review();
    expect(validateReview(p0)).toEqual([]);
    const omitted = clone(p0.artifact); omitted.protectedRows.pop(); omitted.protectedRowsDigest = sha256Text(canonicalJson(omitted.protectedRows));
    expect(validateReview({ ...p0, artifact: omitted })).toContain("protectedRows: mandatory P0 seed set mismatch");
    for (const reasonClass of ["D1_COMPLETED_HISTORY_ONLY", "D2_IMPLEMENTATION_SHAPE", "D3_NON_OBSERVABLE_VISUAL", "D4_DUPLICATE_EVIDENCE", "D5_ACCEPTED_LOSS"]) {
      const deletion = clone(p0.artifact); const seed = deletion.decisions.find((item: any) => item.id === "SC-000420");
      seed.class = reasonClass; seed.reason = `${reasonClass} incorrectly deletes a P0 row`; seed.criticalityRef = undefined; seed.resolution = { disposition: "delete", deletionReason: `${reasonClass} incorrectly deletes the protected behavior` };
      if (reasonClass === "D5_ACCEPTED_LOSS") seed.lostBehavior = [{ assertionOrdinals: [1, 2], behavior: "loss" }];
      expect(validateReview({ ...p0, artifact: deletion })).toContain("SC-000420: protected row cannot use a deletion class");
    }
    const invented = clone(p0.artifact); invented.criticalitySources = [{ id: "INVENTED", citation: "anything" }];
    expect(validateReview({ ...p0, artifact: invented })).toContain("criticalitySources: invalid built-in mapping: INVENTED");
    const duplicateSource = clone(p0.artifact); duplicateSource.criticalitySources.push(clone(duplicateSource.criticalitySources[0]));
    expect(validateReview({ ...p0, artifact: duplicateSource })).toContain("criticalitySources: duplicate id: AGENTS_WINDOWS_SANDBOX");
  });

  it("accepts only base-closed resolved owner evidence and exact blind scope", () => {
    const openOnly = artifactFor();
    openOnly.decisions[0] = { ...openOnly.decisions[0], class: "B1_EXISTING_BEHAVIOR_OWNER", reason: "B1_EXISTING_BEHAVIOR_OWNER names an open-only owner", replacementIds: undefined, ownerEvidence: ["test:vitest:src/existing.test.ts#existing"], resolution: { disposition: "behavior", replacementIds: ["test:vitest:src/existing.test.ts#existing"] } };
    expect(validateReview(review({ artifact: artifactFor(openOnly.decisions) }))).toContain("SC-000001: B1_EXISTING_BEHAVIOR_OWNER requires exact owner evidence");
    const accepted = artifactFor();
    accepted.decisions[0] = { ...accepted.decisions[0], class: "B1_EXISTING_BEHAVIOR_OWNER", reason: "B1_EXISTING_BEHAVIOR_OWNER names the base-closed owner", ownerEvidence: ["test:vitest:src/closed.test.ts#owner"], resolution: { disposition: "behavior", replacementIds: ["test:vitest:src/closed.test.ts#owner"] } };
    expect(validateReview(review({ artifact: artifactFor(accepted.decisions) }))).toEqual([]);

    const blind = artifactFor(); blind.independentReview.mandatoryCohorts = [{ name: "must review", rowIds: ["SC-000002"], comparison: "agree" }];
    expect(validateReview(review({ artifact: blind }))).toContain("SC-000002: missing blind result");
    const extraneous = artifactFor(); extraneous.independentReview.blindResults.push({ id: "SC-000002", sourceHash: extraneous.decisions[1].sourceHash, reviewerRunId: "reviewer-run", class: extraneous.decisions[1].class, reason: "extra" });
    expect(validateReview(review({ artifact: extraneous }))).toContain("SC-000002: extraneous blind result");
    const duplicateBlind = artifactFor(); duplicateBlind.independentReview.blindResults.push(clone(duplicateBlind.independentReview.blindResults[0]));
    expect(validateReview(review({ artifact: duplicateBlind }))).toContain("independentReview: duplicate blind result: SC-000001");
  });

  it("requires complete numeric contiguous hashed shard coverage and valid iteration accounting", () => {
    const exact = artifactFor();
    expect(validateReview(review({ artifact: exact }))).toEqual([]);

    const invalidIterations = clone(exact);
    invalidIterations.independentReview.validIterations = 4;
    expect(validateReview(review({ artifact: invalidIterations }))).toContain("independentReview: validIterations must be 0..3 and equal deterministicSample.iterations");

    const incomplete = clone(exact);
    incomplete.independentReview.shards[0].rowIds.pop();
    expect(validateReview(review({ artifact: incomplete }))).toContain("independentReview: shard rows must exactly cover required blind IDs in numeric order");

    const overlapping = clone(exact);
    overlapping.independentReview.shards.push({ ...clone(overlapping.independentReview.shards[0]), index: 1 });
    expect(validateReview(review({ artifact: overlapping }))).toContain("independentReview: shard rows must exactly cover required blind IDs in numeric order");

    const reordered = clone(exact);
    [reordered.independentReview.shards[0].rowIds[0], reordered.independentReview.shards[0].rowIds[1]] = [reordered.independentReview.shards[0].rowIds[1], reordered.independentReview.shards[0].rowIds[0]];
    expect(validateReview(review({ artifact: reordered }))).toContain("independentReview: shard rows must exactly cover required blind IDs in numeric order");

    const oversized = clone(exact);
    oversized.independentReview.shards[0].rowIds = Array.from({ length: 25 }, (_, index) => `SC-${String(index + 1).padStart(6, "0")}`);
    expect(validateReview(review({ artifact: oversized }))).toContain("independentReview: shard 0 must contain 1..24 rows");

    const unhashed = clone(exact);
    unhashed.independentReview.shards[0].packetSha256 = "not-a-hash";
    expect(validateReview(review({ artifact: unhashed }))).toContain("independentReview: shard 0 requires content-addressed packet/output evidence");

    const staleMerge = clone(exact);
    staleMerge.independentReview.mergedOutputSha256 = "f".repeat(64);
    expect(validateReview(review({ artifact: staleMerge }))).toContain("independentReview: merged output digest mismatch");
  });

  it("derives blind comparisons and disagreement IDs from class and evidence fingerprints", () => {
    const exact = artifactFor();
    exact.independentReview.calibrations = [{ class: "B2_NEW_CHEAP_BEHAVIOR", rowIds: ["SC-000001"], adjacentClass: "B1_EXISTING_BEHAVIOR_OWNER", result: "agree" }];
    expect(validateReview(review({ artifact: exact }))).toEqual([]);

    const nonProtectedCitation = artifactFor();
    nonProtectedCitation.independentReview.blindResults[0].criticalityRef = "TESTING_SOURCE_CONTRACT_REPLACEMENT";
    nonProtectedCitation.independentReview.mergedOutputSha256 = sha256Text(canonicalJson(nonProtectedCitation.independentReview.blindResults));
    expect(validateReview(review({ artifact: nonProtectedCitation }))).toEqual([]);

    const futureCandidateEvidence = artifactFor();
    futureCandidateEvidence.independentReview.blindResults[0].ownerEvidence = ["test:vitest:src/future.test.ts#one"];
    futureCandidateEvidence.independentReview.mergedOutputSha256 = sha256Text(canonicalJson(futureCandidateEvidence.independentReview.blindResults));
    expect(validateReview(review({ artifact: futureCandidateEvidence }))).toEqual([]);

    const criticalityMismatch = artifactFor();
    const criticalBlind = criticalityMismatch.independentReview.blindResults.find((item: any) => item.id === "SC-000420");
    criticalBlind.criticalityRef = "AGENTS_WINDOWS_SANDBOX";
    criticalityMismatch.independentReview.mergedOutputSha256 = sha256Text(canonicalJson(criticalityMismatch.independentReview.blindResults));
    expect(validateReview(review({ artifact: criticalityMismatch }))).toEqual(expect.arrayContaining([
      "mandatoryCohorts: recorded comparison must be rule_changed",
      "independentReview: disagreements must equal fingerprint mismatch IDs",
    ]));

    criticalityMismatch.independentReview.mandatoryCohorts[0].comparison = "rule_changed";
    criticalityMismatch.independentReview.disagreements = [{ rowIds: ["SC-000420"], oldClass: "B3_PROTECTED_EXPENSIVE_BEHAVIOR", newClass: "B3_PROTECTED_EXPENSIVE_BEHAVIOR", groupRuleChange: "criticality citation differs" }];
    expect(validateReview(review({ artifact: criticalityMismatch }))).toEqual([]);

    const ownerBase = clone(baseLedger); ownerBase.rows[4].replacementIds = ["test:vitest:src/closed.test.ts#owner", "test:vitest:src/closed.test.ts#alternate"];
    const ownerMismatch = artifactFor(); ownerMismatch.scope.closedRowsDigest = sha256Text(canonicalJson(ownerBase.rows.filter((item) => !baseOpenIds.has(item.id))));
    ownerMismatch.decisions[0] = { ...ownerMismatch.decisions[0], class: "B1_EXISTING_BEHAVIOR_OWNER", reason: "B1_EXISTING_BEHAVIOR_OWNER has an existing owner.", ownerEvidence: ["test:vitest:src/closed.test.ts#owner"], resolution: { disposition: "behavior", replacementIds: ["test:vitest:src/closed.test.ts#owner"] } };
    const ownerArtifact = artifactFor(ownerMismatch.decisions); ownerArtifact.scope.closedRowsDigest = ownerMismatch.scope.closedRowsDigest;
    const ownerBlind = ownerArtifact.independentReview.blindResults.find((item: any) => item.id === "SC-000001");
    ownerBlind.ownerEvidence = ["test:vitest:src/closed.test.ts#alternate"];
    expect(validateReview(review({ artifact: ownerArtifact, baseLedger: ownerBase, currentLedger: clone(ownerBase) }))).toContain("deterministicSample: recorded comparison must be rule_changed");
  });

  it("records no_match only for an empty calibration cohort", () => {
    const empty = artifactFor();
    empty.independentReview.calibrations = [{ class: "T1_EXISTING_TOOL_OWNER", rowIds: [], adjacentClass: "A1_EXISTING_STRUCTURED_OWNER", result: "no_match" }];
    expect(validateReview(review({ artifact: empty }))).toEqual([]);

    const wrongEmptyResult = clone(empty);
    wrongEmptyResult.independentReview.calibrations[0].result = "agree";
    expect(validateReview(review({ artifact: wrongEmptyResult }))).toContain("calibrations: empty calibration must record no_match");

    const nonEmpty = artifactFor();
    nonEmpty.independentReview.calibrations = [{ class: "B2_NEW_CHEAP_BEHAVIOR", rowIds: ["SC-000001"], adjacentClass: "B1_EXISTING_BEHAVIOR_OWNER", result: "no_match" }];
    expect(validateReview(review({ artifact: nonEmpty }))).toContain("calibrations: recorded comparison must be agree");

    for (const invalidRowIds of [undefined, null, "SC-000001"]) {
      const invalid = artifactFor();
      invalid.independentReview.calibrations = [{
        class: "T1_EXISTING_TOOL_OWNER",
        ...(invalidRowIds === undefined ? {} : { rowIds: invalidRowIds }),
        adjacentClass: "A1_EXISTING_STRUCTURED_OWNER",
        result: "no_match",
      }];
      expect(validateReview(review({ artifact: invalid }))).toContain("calibrations: rowIds must be an array");
    }

    const wrongMembership = artifactFor();
    wrongMembership.independentReview.calibrations = [{
      class: "B3_PROTECTED_EXPENSIVE_BEHAVIOR",
      rowIds: ["SC-000001"],
      adjacentClass: "B2_NEW_CHEAP_BEHAVIOR",
      result: "agree",
    }];
    expect(validateReview(review({ artifact: wrongMembership }))).toContain(
      "calibrations: SC-000001 is B2_NEW_CHEAP_BEHAVIOR, not B3_PROTECTED_EXPENSIVE_BEHAVIOR",
    );

    const falseNoMatch = artifactFor();
    falseNoMatch.independentReview.calibrations = [{
      class: "B3_PROTECTED_EXPENSIVE_BEHAVIOR",
      rowIds: [],
      adjacentClass: "B2_NEW_CHEAP_BEHAVIOR",
      result: "no_match",
    }];
    expect(validateReview(review({ artifact: falseNoMatch }))).toContain(
      "calibrations: B3_PROTECTED_EXPENSIVE_BEHAVIOR cannot record no_match while classified examples exist",
    );
  });

  it("requires the complete ordered calibration registry and mandatory cohort names", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
    ]);
    const validateMutation = (mutate: (artifact: any) => void) => {
      const artifact = clone(reviewArtifact) as any;
      mutate(artifact);
      return validateReview({ artifact, baseLedger: realBaseLedger, currentLedger: realBaseLedger, baseTrackedPaths: realBaseTrackedPaths });
    };

    expect(approvedCalibrationPairs).toHaveLength(10);
    expect(approvedMandatoryCohortNames).toEqual([
      "protected-p0", "security", "import-boundary", "process-lifecycle", "large-contract-26",
      "emerging-b3", "emerging-d5", "emerging-mixed", "known-browser-five",
    ]);
    for (const mutate of [
      (artifact: any) => artifact.independentReview.calibrations.splice(5, 1),
      (artifact: any) => artifact.independentReview.calibrations.push(clone(artifact.independentReview.calibrations[0])),
      (artifact: any) => artifact.independentReview.calibrations.push({ class: "EXTRA", adjacentClass: "D1_COMPLETED_HISTORY_ONLY", rowIds: [], result: "no_match" }),
      (artifact: any) => { artifact.independentReview.calibrations[0].adjacentClass = "D3_NON_OBSERVABLE_VISUAL"; },
    ]) {
      expect(validateMutation(mutate)).toContain("calibrations: class/adjacentClass pairs must equal the approved ordered registry");
    }
    for (const mutate of [
      (artifact: any) => artifact.independentReview.mandatoryCohorts.pop(),
      (artifact: any) => { artifact.independentReview.mandatoryCohorts[4].name = "large-contract"; },
      (artifact: any) => artifact.independentReview.mandatoryCohorts.push(clone(artifact.independentReview.mandatoryCohorts[0])),
      (artifact: any) => artifact.independentReview.mandatoryCohorts.push({ name: "extra", rowIds: [], comparison: "agree" }),
    ]) {
      expect(validateMutation(mutate)).toContain("mandatoryCohorts: names must equal the approved ordered registry");
    }

    for (const [name, mutateRows] of [
      ["known-browser-five", (rowIds: string[]) => rowIds.splice(0)],
      ["protected-p0", (rowIds: string[]) => rowIds.splice(0, 1)],
      ["security", (rowIds: string[]) => rowIds.push(rowIds[0])],
      ["import-boundary", (rowIds: string[]) => rowIds.reverse()],
      ["process-lifecycle", (rowIds: string[]) => rowIds.push("SC-999999")],
      ["large-contract-26", (rowIds: string[]) => rowIds.splice(10, 1)],
    ] as const) {
      expect(validateMutation((artifact) => {
        mutateRows(artifact.independentReview.mandatoryCohorts.find((cohort: any) => cohort.name === name).rowIds);
      })).toContain(`mandatoryCohorts: ${name} rowIds must equal approved sorted membership`);
    }
  });

  it("requires exact emerging cohort membership", () => {
    const missing = artifactFor();
    missing.independentReview.mandatoryCohorts.push({
      name: "emerging-b3",
      rowIds: [],
      comparison: "agree",
    });
    expect(validateReview(review({ artifact: missing }))).toContain(
      "mandatoryCohorts: emerging-b3 must exactly equal the classified B3 rows",
    );
  });

  it("accepts valid iteration three only through the exact max-three rule adjudication allowlist", () => {
    const finalClassById = new Map(approvedRuleAdjudications.flatMap((item) => item.rowIds.map((id) => [id, item.finalClass])));
    const exact: any = {
      independentReview: {
        validIterations: 3,
        ruleAdjudications: clone(approvedRuleAdjudications),
        disagreements: clone(approvedRuleDisagreements),
        blindResults: [...finalClassById].map(([id, reasonClass]) => ({ id, class: reasonClass })),
      },
      decisions: [...finalClassById].map(([id, reasonClass]) => ({ id, class: reasonClass })),
    };
    expect(validateRuleAdjudications(exact)).toEqual([]);

    for (const mutate of [
      (value: any) => value.independentReview.ruleAdjudications.pop(),
      (value: any) => value.independentReview.ruleAdjudications.push(clone(value.independentReview.ruleAdjudications[0])),
      (value: any) => { value.independentReview.ruleAdjudications[0].rule += " changed"; },
      (value: any) => value.independentReview.ruleAdjudications[0].rowIds.pop(),
      (value: any) => { value.independentReview.ruleAdjudications[0].finalClass = "D1_COMPLETED_HISTORY_ONLY"; },
    ]) {
      const invalid = clone(exact);
      mutate(invalid);
      expect(validateRuleAdjudications(invalid)).toContain("independentReview: ruleAdjudications must equal the approved max-three allowlist");
    }

    const wrongDecision = clone(exact);
    wrongDecision.decisions[0].class = "D1_COMPLETED_HISTORY_ONLY";
    expect(validateRuleAdjudications(wrongDecision)).toContain("SC-000077: adjudicated final class must be D2_IMPLEMENTATION_SHAPE");

    const wrongBlind = clone(exact);
    wrongBlind.independentReview.blindResults[0].class = "D1_COMPLETED_HISTORY_ONLY";
    expect(validateRuleAdjudications(wrongBlind)).toContain("SC-000077: retained blind fingerprint must equal adjudicated final class D2_IMPLEMENTATION_SHAPE");

    const missingDisagreement = clone(exact);
    missingDisagreement.independentReview.disagreements.pop();
    expect(validateRuleAdjudications(missingDisagreement)).toContain("independentReview: retained adjudication disagreements must equal the approved groups");

    const wrongIteration = clone(exact);
    wrongIteration.independentReview.validIterations = 2;
    expect(validateRuleAdjudications(wrongIteration)).toContain("independentReview: ruleAdjudications require validIterations 3");

    expect(validateRuleAdjudications({ independentReview: { validIterations: 3, ruleAdjudications: [], disagreements: [], blindResults: [] }, decisions: [] })).toContain(
      "independentReview: ruleAdjudications must equal the approved max-three allowlist",
    );
  });

  it("loads and validates the exact committed content-addressed review evidence", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    expect(loaded.issues).toEqual([]);
    const realInput = {
      artifact: reviewArtifact,
      baseLedger: realBaseLedger,
      currentLedger: realBaseLedger,
      baseTrackedPaths: realBaseTrackedPaths,
      evidenceBytes: loaded.evidenceBytes,
    } as any;
    expect(validateReview(realInput)).toEqual([]);

    const firstPacketPath = reviewArtifact.independentReview.shards[0].packetPath;
    const tamperedBytes = new Map(loaded.evidenceBytes);
    tamperedBytes.set(firstPacketPath, `${tamperedBytes.get(firstPacketPath)} `);
    expect(validateReview({ ...realInput, evidenceBytes: tamperedBytes })).toContain(
      "independentReview: shard 0 packet byte hash mismatch",
    );

    const traversal = clone(reviewArtifact) as any;
    traversal.independentReview.shards[0].packetPath = `testing/source-contract-redisposition-evidence/../${traversal.independentReview.shards[0].packetPath.split("/").at(-1)}`;
    expect(validateReview({ ...realInput, artifact: traversal })).toContain(
      "independentReview: shard 0 packetPath must be confined to testing/source-contract-redisposition-evidence",
    );

    const missingBytes = new Map(loaded.evidenceBytes);
    missingBytes.delete(firstPacketPath);
    expect(validateReview({ ...realInput, evidenceBytes: missingBytes })).toContain(
      "independentReview: shard 0 packet evidence file is missing",
    );

    const malformedBytes = new Map(loaded.evidenceBytes);
    malformedBytes.set(firstPacketPath, "{\n");
    expect(validateReview({ ...realInput, evidenceBytes: malformedBytes })).toEqual(expect.arrayContaining([
      "independentReview: shard 0 packet byte hash mismatch",
      "independentReview: shard 0 packet is malformed JSON",
    ]));

    const declarationArtifact = clone(reviewArtifact) as any;
    const declarationBytes = new Map(loaded.evidenceBytes);
    const rewritePair = (packetReference: any, outputReference: any, mutatePacket: (packet: any) => void) => {
      const packet = JSON.parse(declarationBytes.get(packetReference.packetPath) as string);
      mutatePacket(packet);
      const packetBody = `${JSON.stringify(packet, null, 2)}\n`;
      const packetHash = sha256Text(packetBody);
      const packetPath = packetReference.packetPath.replace(/[0-9a-f]{64}\.json$/, `${packetHash}.json`);
      const output = JSON.parse(declarationBytes.get(outputReference.outputPath) as string);
      output.packetSha256 = packetHash;
      const outputBody = `${JSON.stringify(output, null, 2)}\n`;
      const outputHash = sha256Text(outputBody);
      const outputPath = outputReference.outputPath.replace(/[0-9a-f]{64}\.json$/, `${outputHash}.json`);
      declarationBytes.set(packetPath, packetBody);
      declarationBytes.set(outputPath, outputBody);
      packetReference.packetPath = packetPath;
      packetReference.packetSha256 = packetHash;
      outputReference.outputPath = outputPath;
      outputReference.outputSha256 = outputHash;
    };
    const targetId = declarationArtifact.independentReview.shards[0].rowIds[0];
    rewritePair(declarationArtifact.independentReview.shards[0], declarationArtifact.independentReview.shards[0], (packet) => {
      packet.rows.find((row: any) => row.id === targetId).declaration += " /* tampered */";
    });
    rewritePair(declarationArtifact.independentReview.reviewer, declarationArtifact.independentReview.reviewer, (packet) => {
      packet.rows.find((row: any) => row.id === targetId).declaration += " /* tampered */";
    });
    expect(validateReview({ ...realInput, artifact: declarationArtifact, evidenceBytes: declarationBytes })).toContain(
      `${targetId}: blind packet declaration hash must equal sourceHash`,
    );
  });

  it("binds Task 3 packet candidates and blind owner evidence to authoritative owner facts", async () => {
    const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
    const [realBaseLedger, realBaseTrackedPaths, loaded] = await Promise.all([
      loadBaseLedger({ repoRoot, commit: REVIEW_BASE }),
      loadBaseTrackedPaths({ repoRoot, commit: REVIEW_BASE }),
      loadReviewEvidence({ repoRoot, artifact: reviewArtifact }),
    ]);
    const realInput = {
      artifact: reviewArtifact,
      baseLedger: realBaseLedger,
      currentLedger: realBaseLedger,
      baseTrackedPaths: realBaseTrackedPaths,
    } as any;
    const firstShard = (reviewArtifact as any).independentReview.shards[0];
    const validatePacketMutation = (mutate: (packet: any) => void) => {
      const bytes = new Map(loaded.evidenceBytes);
      const packet = JSON.parse(evidenceText(bytes.get(firstShard.packetPath)));
      mutate(packet);
      bytes.set(firstShard.packetPath, `${JSON.stringify(packet, null, 2)}\n`);
      return validateReview({ ...realInput, evidenceBytes: bytes });
    };

    const invented = validatePacketMutation((packet) => {
      packet.rows.find((row: any) => row.id === "SC-000004").candidateOwners.push({
        id: "test:vitest:scripts/invented.test.ts#invented",
        mechanism: "node",
        capability: "invented",
        status: "future",
      });
    });
    expect(invented).toContain("SC-000004: blind packet candidate inventory mismatch");

    const omitted = validatePacketMutation((packet) => {
      packet.rows.find((row: any) => row.id === "SC-000004").candidateOwners = [];
    });
    expect(omitted).toContain("SC-000004: blind packet candidate inventory mismatch");

    for (const mutate of [
      (candidate: any) => { delete candidate.capability; },
      (candidate: any) => { candidate.capability = "arbitrary capability"; },
      (candidate: any) => { delete candidate.mechanism; },
      (candidate: any) => { candidate.mechanism = "arbitrary mechanism"; },
      (candidate: any) => { delete candidate.status; },
      (candidate: any) => { candidate.status = "arbitrary status"; },
    ]) {
      const issues = validatePacketMutation((packet) => {
        mutate(packet.rows.find((row: any) => row.id === "SC-000004").candidateOwners[0]);
      });
      expect(issues).toContain("SC-000004: blind packet candidate status/citation mismatch");
    }

    const wrongResolvedSemantics = validatePacketMutation((packet) => {
      const candidate = packet.rows.find((row: any) => row.id === "SC-000064").candidateOwners[0];
      candidate.status = "future";
      candidate.resolvedByBaseClosedRow = false;
      delete candidate.closedRowIds;
    });
    expect(wrongResolvedSemantics).toContain("SC-000064: blind packet candidate status/citation mismatch");

    const unboundBytes = new Map(loaded.evidenceBytes);
    const packet = JSON.parse(evidenceText(unboundBytes.get(firstShard.packetPath)));
    const output = JSON.parse(evidenceText(unboundBytes.get(firstShard.outputPath)));
    const packetRow = packet.rows.find((row: any) => row.id === "SC-000004");
    const outputRow = output.results.find((row: any) => row.id === "SC-000004");
    const unboundOwner = "test:vitest:scripts/invented.test.ts#invented";
    packetRow.candidateOwners = [{ id: unboundOwner, mechanism: "node", capability: packetRow.invariant, status: "future" }];
    outputRow.ownerEvidence = [unboundOwner];
    unboundBytes.set(firstShard.packetPath, `${JSON.stringify(packet, null, 2)}\n`);
    unboundBytes.set(firstShard.outputPath, `${JSON.stringify(output, null, 2)}\n`);
    expect(validateReview({ ...realInput, evidenceBytes: unboundBytes })).toContain(
      "SC-000004: blind output selected an invented candidate owner",
    );
  });

  it("pins the approved candidate-binding owner, retained-class, and protected-criticality adjudications", () => {
    expect(validateCandidateBindingAdjudications({
      candidateBindingAdjudications: approvedCandidateBindingAdjudications,
    })).toEqual([]);

    for (const mutate of [
      (value: any) => { value.exactD4Owners.rows[1].ownerEvidence.pop(); },
      (value: any) => { value.exactFutureOwners.rows[0].ownerEvidence = ["test:cargo:extractum::analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects"]; },
      (value: any) => { value.retainedClasses[0].rowIds = value.retainedClasses[0].rowIds.filter((id: string) => id !== "SC-000416"); },
      (value: any) => { value.protectedCriticality.rows[0].criticalityRef = "AGENTS_SECURITY"; },
      (value: any) => { value.extra = true; },
    ]) {
      const changed = clone(approvedCandidateBindingAdjudications) as any;
      mutate(changed);
      expect(validateCandidateBindingAdjudications({ candidateBindingAdjudications: changed })).toContain(
        "independentReview: candidateBindingAdjudications must equal the approved exact registry",
      );
    }

    const incompleteAuthorOwners = clone(reviewArtifact.decisions) as any[];
    incompleteAuthorOwners.find((decision) => decision.id === "SC-000316").ownerEvidence =
      incompleteAuthorOwners.find((decision) => decision.id === "SC-000316").ownerEvidence.slice(0, 1);
    expect(validateCandidateBindingAdjudications({
      candidateBindingAdjudications: approvedCandidateBindingAdjudications,
      decisions: incompleteAuthorOwners,
      blindResults: reviewArtifact.independentReview.blindResults,
      candidateOwnerInventory: reviewArtifact.independentReview.candidateOwnerInventory,
    })).toContain("SC-000316: approved candidate-binding adjudication requires D4 with the exact author owner set");
  });

  it("pins fail-closed packet policy facts independently of author decisions", () => {
    expect(validatePacketPolicyBindings({ packetPolicyBindings: approvedPacketPolicyBindings })).toEqual([]);
    for (const mutate of [
      (value: any) => { value.criticality.shift(); },
      (value: any) => { value.criticality[0].criticalityRef = "AGENTS_SECURITY"; },
      (value: any) => { value.rowAdjudications[0].finalClass = "D3_NON_OBSERVABLE_VISUAL"; },
      (value: any) => { value.criticality[0].id = "SC-999999"; },
    ]) {
      const changed = clone(approvedPacketPolicyBindings) as any;
      mutate(changed);
      expect(validatePacketPolicyBindings({ packetPolicyBindings: changed })).toContain(
        "independentReview: packetPolicyBindings must equal the approved exact registry",
      );
    }
  });

  it("applies only resolutions in stable order, serializes canonically, and is idempotent", () => {
    const input = review();
    const first = applyReview(input);
    const second = applyReview({ ...input, currentLedger: first.ledger });
    expect(first.changedPaths).toEqual(expect.arrayContaining(["rows[SC-000001].replacementIds", "rows[SC-000002].replacementIds"]));
    expect(first.changedPaths).toHaveLength(14);
    expect(canonicalJson(second.ledger)).toBe(canonicalJson(first.ledger));
    expect(`${canonicalJson(first.ledger)}\n`).toMatch(/[^\n]\n$/);
    expect(first.ledger.rows.map((item: any) => item.id)).toEqual(baseLedger.rows.map((item) => item.id));
    expect(first.ledger.rows[0].invariant).toBe(baseLedger.rows[0].invariant);
    expect(resolutionForDecision(input.artifact.decisions[0])).toEqual(input.artifact.decisions[0].resolution);

    const mixedArtifact = artifactFor(); mixedArtifact.decisions[0].resolution = { subgroups: [
      { assertionOrdinals: [1], invariant: "first", disposition: "behavior", replacementIds: ["test:vitest:src/future.test.ts#one"] },
      { assertionOrdinals: [2], invariant: "second", disposition: "behavior", replacementIds: ["test:vitest:src/future.test.ts#two"] },
    ] };
    expect(applyReview(review({ artifact: mixedArtifact })).changedPaths.filter((item) => item.includes("SC-000001") || item.includes("SC-000002"))).toEqual([
      "rows[SC-000001].disposition", "rows[SC-000001].replacementIds", "rows[SC-000001].subgroups", "rows[SC-000002].replacementIds",
    ]);
    const mixedCurrent = clone(baseLedger); mixedCurrent.rows[0] = { ...mixedCurrent.rows[0], subgroups: mixedArtifact.decisions[0].resolution.subgroups }; delete (mixedCurrent.rows[0] as any).disposition; delete (mixedCurrent.rows[0] as any).replacementIds;
    expect(applyReview(review({ currentLedger: mixedCurrent })).changedPaths.filter((item) => item.includes("SC-000001") || item.includes("SC-000002"))).toEqual([
      "rows[SC-000001].disposition", "rows[SC-000001].replacementIds", "rows[SC-000001].subgroups", "rows[SC-000002].replacementIds",
    ]);
  });

  it("accepts only the exact review-base or fully-applied open resolution state", () => {
    const input = review();
    const applied = applyReview(input).ledger;
    expect(validateReview({ ...input, currentLedger: applied })).toEqual([]);

    const hybrid = clone(input.currentLedger);
    hybrid.rows[0] = clone(applied.rows[0]);
    expect(validateReview({ ...input, currentLedger: hybrid })).toContain(
      "ledger: base-open resolutions must collectively equal either the review-base or fully-applied state",
    );
  });
});
