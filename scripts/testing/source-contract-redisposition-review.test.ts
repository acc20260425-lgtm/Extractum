import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import reviewArtifact from "../../testing/source-contract-redisposition-review.json";
import {
  applyReview,
  canonicalJson,
  loadBaseLedger,
  loadBaseTrackedPaths,
  resolutionForDecision,
  sha256Text,
  validateReview,
} from "./source-contract-redisposition-review.mjs";

const REVIEW_BASE = "a54507d63420bb870c3870c91d7e22b050abae3e";
const hash = (value: unknown) => createHash("sha256").update(String(value)).digest("hex");
const clone = <T>(value: T): T => structuredClone(value);
const mandatoryP0Ids = ["SC-000420", "SC-000511", "SC-000512", "SC-000513", "SC-000514", "SC-000515", "SC-000555", "SC-000556", "SC-000557", "SC-000558", "SC-000559", "SC-000560"];

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
  const legacyOwner = { command: "npm.cmd run test:legacy-contract", warmupExitCode: 0, retainedSeconds: [10, 11, 12], medianSeconds: 11, baseFiles: ["src/a.test.ts", "src/z.test.ts"] };
  const verify = { command: "npm.cmd run verify", seconds: 200, exitCode: 0, historicalSeconds: [208.1, 321.3, 383.4], gateInventory: verifyGateInventory };
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
  const artifact: any = {
    schemaVersion: 1,
    reviewBaseCommit: REVIEW_BASE,
    ledgerFrozenAtCommit: baseLedger.frozenAtCommit,
    scope: { openRows: 14, closedRows: 3, closedRowsDigest: closedDigest, pathPresentClosedRowIds: ["SC-000355", "SC-000366"] },
    reasonClasses: classIds.map((id) => ({ id, rule: `${id} named rule` })),
    protectedRows: mandatoryP0Ids.map((id) => ({ id, criticalityRef: "AGENTS_SECURITY" })),
    protectedRowsDigest: sha256Text(canonicalJson(mandatoryP0Ids.map((id) => ({ id, criticalityRef: "AGENTS_SECURITY" })))),
    criticalitySources,
    ...timingFixture(),
    decisions,
    independentReview: {
      authorRunId: "author-run",
      reviewer: { agentTaskId: "reviewer-run", contextPolicy: "blind-no-proposed-class-or-reason" },
      blindResults: [...new Set([...sample, ...mandatoryP0Ids])].map((id) => decisions.find((decision) => decision.id === id)).filter(Boolean).map((decision) => ({
        id: decision.id, sourceHash: decision.sourceHash, reviewerRunId: "reviewer-run", class: decision.class,
        reason: `independent ${decision.reason}`, ...(decision.ownerEvidence ? { ownerEvidence: decision.ownerEvidence } : {}), ...(decision.criticalityRef ? { criticalityRef: decision.criticalityRef } : {}),
      })),
      calibrations: [], mandatoryCohorts: [{ name: "mandatory P0", rowIds: mandatoryP0Ids, comparison: "agree" }],
      deterministicSample: { algorithm: "sha256-id-lowest-10-percent", population, populationDigest: sha256Text(canonicalJson(population)), rowIds: sample, iterations: 1, comparison: "agree" },
      disagreements: [],
    },
    acceptedLoss: { rows: 0, assertionOrdinals: 0, items: [] },
  };
  const rowById = new Map(baseLedger.rows.map((item) => [item.id, item]));
  const futureRows = new Set<string>();
  const proposedRows = new Set<string>();
  let futureOrdinals = 0;
  let proposedOrdinals = 0;
  for (const decision of decisions) {
    if (!["B2_NEW_CHEAP_BEHAVIOR", "B3_PROTECTED_EXPENSIVE_BEHAVIOR"].includes(decision.class)) continue;
    const row = rowById.get(decision.id) as any;
    const groups = decision.resolution?.subgroups ?? [decision.resolution];
    for (const group of groups) {
      if (!group?.replacementIds?.some((id: string) => id.startsWith("test:vitest:"))) continue;
      const ordinals = decision.resolution?.subgroups ? group.assertionOrdinals : Array.from({ length: row.assertionCount }, (_, index) => index + 1);
      futureRows.add(decision.id);
      futureOrdinals += ordinals.length;
      if (decision.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR") {
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
    futureOwnersByMechanism: futureRows.size ? { jsdom: { rows: futureRows.size, assertionOrdinals: futureOrdinals } } : {},
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

describe("source-contract redisposition review", () => {
  it("pins every real base-open row in the static draft artifact", async () => {
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
    expect(decisions.every((item) => item.class === "UNCLASSIFIED" && !("reason" in item) && !("resolution" in item))).toBe(true);
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
    expect(validateReview(review({ artifact: wrongGrouping }))).toContain("forecast: futureOwnersByMechanism does not match jsdom owner grouping");

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

  it("requires current open resolutions to remain frozen before apply", () => {
    const reordered = clone(baseLedger); [reordered.rows[0], reordered.rows[1]] = [reordered.rows[1], reordered.rows[0]];
    expect(validateReview(review({ currentLedger: reordered }))).toContain("ledger: current row ID order drift from review base");
    const mutated = clone(baseLedger); mutated.rows[0].replacementIds = ["test:vitest:src/mutated.test.ts#owner"];
    expect(validateReview(review({ currentLedger: mutated }))).toContain("SC-000001: current resolution drift from review base");
    const replaced = clone(baseLedger); replaced.rows[0].subgroups = [{ assertionOrdinals: [1, 2], invariant: "replacement", disposition: "behavior", replacementIds: ["test:vitest:src/replaced.test.ts#owner"] }]; delete (replaced.rows[0] as any).disposition; delete (replaced.rows[0] as any).replacementIds;
    expect(validateReview(review({ currentLedger: replaced }))).toContain("SC-000001: current resolution drift from review base");

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
    expect(validateReview({ ...p0, artifact: duplicateSource })).toContain("criticalitySources: duplicate id: AGENTS_SECURITY");
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

  it("derives blind comparisons and disagreement IDs from class and evidence fingerprints", () => {
    const exact = artifactFor();
    exact.independentReview.calibrations = [{ class: "B2_NEW_CHEAP_BEHAVIOR", rowIds: ["SC-000001"], adjacentClass: "B1_EXISTING_BEHAVIOR_OWNER", result: "agree" }];
    expect(validateReview(review({ artifact: exact }))).toEqual([]);

    const criticalityMismatch = artifactFor();
    criticalityMismatch.criticalitySources.push({ id: "AGENTS_WINDOWS_SANDBOX", citation: "AGENTS.md §2" });
    const criticalBlind = criticalityMismatch.independentReview.blindResults.find((item: any) => item.id === "SC-000420");
    criticalBlind.criticalityRef = "AGENTS_WINDOWS_SANDBOX";
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
});
