import { createHash } from "node:crypto";
import { describe, expect, it } from "vitest";

import {
  applyReview,
  canonicalJson,
  resolutionForDecision,
  sha256Text,
  validateReview,
} from "./source-contract-redisposition-review.mjs";

const REVIEW_BASE = "a54507d63420bb870c3870c91d7e22b050abae3e";
const hash = (value: unknown) => createHash("sha256").update(String(value)).digest("hex");
const clone = <T>(value: T): T => structuredClone(value);

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
    row("SC-000900", "src/path-absent-closed.test.ts", { disposition: "delete", deletionReason: "closed historical row" }),
  ],
};

const baseTrackedPaths = new Set([
  "src/open-one.test.ts",
  "src/open-two.test.ts",
  "src/path-present-closed-one.test.ts",
  "src/path-present-closed-two.test.ts",
]);

const baseOpenIds = new Set(["SC-000001", "SC-000002"]);
const closedDigest = sha256Text(canonicalJson(baseLedger.rows.filter((item) => !baseOpenIds.has(item.id))));
const classIds = [
  "D1_COMPLETED_HISTORY_ONLY", "D2_IMPLEMENTATION_SHAPE", "D3_NON_OBSERVABLE_VISUAL", "D4_DUPLICATE_EVIDENCE",
  "A1_EXISTING_STRUCTURED_OWNER", "T1_EXISTING_TOOL_OWNER", "B1_EXISTING_BEHAVIOR_OWNER", "B2_NEW_CHEAP_BEHAVIOR",
  "B3_PROTECTED_EXPENSIVE_BEHAVIOR", "D5_ACCEPTED_LOSS",
];

function artifactFor(decisions = [
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
] as any[]) {
  const ordinary = decisions.filter((decision) => !["B3_PROTECTED_EXPENSIVE_BEHAVIOR", "D5_ACCEPTED_LOSS"].includes(decision.class) && !decision.resolution?.subgroups);
  const population = ordinary.map((decision) => decision.id).sort();
  const sample = [...population].sort((a, b) => hash(a).localeCompare(hash(b))).slice(0, Math.ceil(population.length * 0.1));
  return {
    schemaVersion: 1,
    reviewBaseCommit: REVIEW_BASE,
    ledgerFrozenAtCommit: baseLedger.frozenAtCommit,
    scope: { openRows: 2, closedRows: 3, closedRowsDigest: closedDigest, pathPresentClosedRowIds: ["SC-000355", "SC-000366"] },
    reasonClasses: classIds.map((id) => ({ id, rule: `${id} named rule` })),
    protectedRows: [{ id: "SC-000002", criticalityRef: "AGENTS_SECURITY" }],
    protectedRowsDigest: sha256Text(canonicalJson([{ id: "SC-000002", criticalityRef: "AGENTS_SECURITY" }])),
    criticalitySources: [{ id: "AGENTS_SECURITY", citation: "AGENTS.md section 7" }],
    mechanisms: {},
    decisions,
    independentReview: {
      authorRunId: "author-run",
      reviewer: { agentTaskId: "reviewer-run", contextPolicy: "blind-no-proposed-class-or-reason" },
      blindResults: decisions.map((decision) => ({
        id: decision.id, sourceHash: decision.sourceHash, reviewerRunId: "reviewer-run", class: decision.class,
        reason: `independent ${decision.reason}`, ...(decision.criticalityRef ? { criticalityRef: decision.criticalityRef } : {}),
      })),
      calibrations: [], mandatoryCohorts: [],
      deterministicSample: { algorithm: "sha256-id-lowest-10-percent", population, populationDigest: sha256Text(canonicalJson(population)), rowIds: sample, iterations: 1, comparison: "agree" },
      disagreements: [],
    },
    forecast: {}, acceptedLoss: { rows: 0, assertionOrdinals: 0, items: [] },
  };
}

function review(overrides: Record<string, unknown> = {}) {
  return {
    artifact: artifactFor(), baseLedger: clone(baseLedger), currentLedger: clone(baseLedger), baseTrackedPaths,
    ...overrides,
  } as any;
}

describe("source-contract redisposition review", () => {
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
    expect(validateReview(review({ baseTrackedPaths: new Set(["src/open-one.test.ts"]) }))).toContain("scope: expected 2 base-open rows, found 1");
    const exceptions = artifactFor(); exceptions.scope.pathPresentClosedRowIds = ["SC-000366", "SC-000355"];
    expect(validateReview(review({ artifact: exceptions }))).toContain("scope: pathPresentClosedRowIds must equal SC-000355, SC-000366");
    const badClosed = clone(baseLedger); badClosed.rows[2].invariant = "rewritten";
    expect(validateReview(review({ currentLedger: badClosed }))).toContain("SC-000355: immutable row field drift");
    const badDigest = artifactFor(); badDigest.scope.closedRowsDigest = "0".repeat(64);
    expect(validateReview(review({ artifact: badDigest }))).toContain("scope: closedRowsDigest mismatch");
    const badEnvelope = clone(baseLedger); badEnvelope.sourceReaderExceptions.push({ path: "src/x", sourceRange: "1:1-1:2", reason: "x", owner: "x" });
    expect(validateReview(review({ currentLedger: badEnvelope }))).toContain("ledger: sourceReaderExceptions changed");
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
        if (reasonClass === "D4_DUPLICATE_EVIDENCE") decision.ownerEvidence = ["test:vitest:src/existing.test.ts#existing"];
      } else {
        decision.resolution = { disposition, replacementIds: ["test:vitest:src/future.test.ts#owner"] };
        if (["A1_EXISTING_STRUCTURED_OWNER", "T1_EXISTING_TOOL_OWNER", "B1_EXISTING_BEHAVIOR_OWNER"].includes(reasonClass)) {
          decision.resolution = { disposition, replacementIds: ["test:vitest:src/existing.test.ts#existing"] };
          decision.ownerEvidence = ["test:vitest:src/existing.test.ts#existing"];
        }
        if (reasonClass === "B3_PROTECTED_EXPENSIVE_BEHAVIOR") decision.criticalityRef = "AGENTS_SECURITY";
      }
      const decisions = [decision, clone(artifactFor().decisions[1])];
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

  it("applies only resolutions in stable order, serializes canonically, and is idempotent", () => {
    const input = review();
    const first = applyReview(input);
    const second = applyReview({ ...input, currentLedger: first.ledger });
    expect(first.changedPaths).toEqual([
      "rows[SC-000001].disposition", "rows[SC-000001].replacementIds",
      "rows[SC-000002].disposition", "rows[SC-000002].replacementIds",
    ]);
    expect(canonicalJson(second.ledger)).toBe(canonicalJson(first.ledger));
    expect(`${canonicalJson(first.ledger)}\n`).toMatch(/[^\n]\n$/);
    expect(first.ledger.rows.map((item: any) => item.id)).toEqual(baseLedger.rows.map((item) => item.id));
    expect(first.ledger.rows[0].invariant).toBe(baseLedger.rows[0].invariant);
    expect(resolutionForDecision(input.artifact.decisions[0])).toEqual(input.artifact.decisions[0].resolution);
  });
});
