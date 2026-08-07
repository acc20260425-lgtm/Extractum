import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import reviewArtifact from "../../testing/source-contract-redisposition-review.json";
import {
  applyReview,
  canonicalJson,
  deriveTailFamilyManifests,
  loadBaseLedger,
  loadBaseTrackedPaths,
  loadReviewEvidence,
  resolutionForDecision,
  sha256Text,
  validateRuleAdjudications,
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
        if (id.startsWith("test:vitest:")) return ["jsdom"];
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
    const packetBytes = new Map(await Promise.all((reviewArtifact as any).tailReview.packetShards.map(async (shard: any) => [
      shard.packetPath,
      await readFile(fileURLToPath(new URL(`../../${shard.packetPath}`, import.meta.url)), "utf8"),
    ])));
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
      B2_NEW_CHEAP_BEHAVIOR: 284,
      B3_PROTECTED_EXPENSIVE_BEHAVIOR: 8,
      D1_COMPLETED_HISTORY_ONLY: 6,
      D2_IMPLEMENTATION_SHAPE: 121,
      D3_NON_OBSERVABLE_VISUAL: 11,
      D4_DUPLICATE_EVIDENCE: 6,
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
