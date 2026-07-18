import { describe, expect, it } from "vitest";

import {
  PROTOCOL,
  evaluateAttempt,
  reduceRetry,
  summarizeBlock,
} from "./protocol.mjs";

const samples = (value: number) => Array(PROTOCOL.samplesPerBlock).fill(value);

function base(overrides: Record<string, number[]> = {}) {
  return {
    A0: samples(9_000),
    B: samples(9_000),
    A1: samples(9_000),
    C: samples(9_000),
    A2: samples(9_000),
    D: samples(9_000),
    A3: samples(9_000),
    ...overrides,
  };
}

describe("process shell diagnostic protocol", () => {
  it("accepts five clustered samples and keeps both outliers", () => {
    const summary = summarizeBlock([9_000, 9_020, 9_040, 9_060, 9_080, 10_000, 11_000]);
    expect(summary).toEqual({
      samplesMs: [9_000, 9_020, 9_040, 9_060, 9_080, 10_000, 11_000],
      medianMs: 9_060,
      samplesWithinBand: 5,
      stable: true,
    });
  });

  it("rejects a four-versus-three bimodal block", () => {
    expect(summarizeBlock([9_000, 9_000, 9_000, 9_000, 10_000, 10_000, 10_000]).stable).toBe(false);
  });

  it("invalidates anchor drift above 300 ms", () => {
    const result = evaluateAttempt(base({ A3: samples(9_301) }));
    expect(result.kind).toBe("stability_invalid");
    expect(result.reasons).toContain("anchor_range_exceeded");
  });

  it("requests E only when B and C are fast and D crosses 500 ms", () => {
    const result = evaluateAttempt(base({ D: samples(9_500) }));
    expect(result.kind).toBe("needs_e");
    expect(result.eRequired).toBe(true);
  });

  it("classifies a fast E and slow D as the D-specific boundary composite", () => {
    const result = evaluateAttempt({
      ...base({ D: samples(9_500) }),
      E: samples(9_000),
      A4: samples(9_000),
    });
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("boundary_composite");
    expect(result.contrasts.dSpecificCompositeMs).toBe(500);
  });

  it("classifies a slow E as manifest-related", () => {
    const result = evaluateAttempt({
      ...base({ D: samples(9_700) }),
      E: samples(9_600),
      A4: samples(9_000),
    });
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("manifest_related");
  });

  it("classifies B at the threshold as membership configuration", () => {
    const result = evaluateAttempt(base({ B: samples(9_500) }));
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("membership_configuration");
  });

  it("classifies C at the threshold as edge-related configuration", () => {
    const result = evaluateAttempt(base({ C: samples(9_500) }));
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("edge_related_configuration");
  });

  it("routes a 5-percent versus 500-ms disagreement to anomaly", () => {
    const result = evaluateAttempt(base({ B: samples(9_480) }));
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("threshold_disagreement");
    expect(result.metrics.B.material).toBe(false);
    expect(result.metrics.B.shellCapFailed).toBe(true);
  });

  it("records not reproduced only when all declared variants stay below both caps", () => {
    const result = evaluateAttempt(base());
    expect(result.kind).toBe("valid");
    expect(result.classification).toBe("not_reproduced");
  });

  it("keeps unexplained stability count through infrastructure retries", () => {
    const first = reduceRetry(
      { unexplainedStabilityInvalidCount: 0, terminal: false },
      { kind: "stability_invalid", objectiveCauseCorrected: false },
    );
    expect(first).toMatchObject({ action: "retry", unexplainedStabilityInvalidCount: 1 });

    const blocked = reduceRetry(first.state, {
      kind: "infrastructure_invalid",
      objectiveCauseCorrected: false,
    });
    expect(blocked).toMatchObject({ action: "await_correction", unexplainedStabilityInvalidCount: 1 });

    const corrected = reduceRetry(blocked.state, {
      kind: "infrastructure_invalid",
      objectiveCauseCorrected: true,
    });
    expect(corrected).toMatchObject({ action: "retry", unexplainedStabilityInvalidCount: 1 });

    const second = reduceRetry(corrected.state, {
      kind: "stability_invalid",
      objectiveCauseCorrected: false,
    });
    expect(second).toMatchObject({
      action: "environment_precision_insufficient",
      unexplainedStabilityInvalidCount: 2,
      state: { terminal: true },
    });
  });

  it("does not count a stability failure with a corrected objective cause", () => {
    const result = reduceRetry(
      { unexplainedStabilityInvalidCount: 1, terminal: false },
      { kind: "stability_invalid", objectiveCauseCorrected: true },
    );
    expect(result).toMatchObject({ action: "retry", unexplainedStabilityInvalidCount: 1 });
  });

  it("completes a valid attempt and forbids another reduction", () => {
    const complete = reduceRetry(
      { unexplainedStabilityInvalidCount: 0, terminal: false },
      { kind: "valid", objectiveCauseCorrected: false },
    );
    expect(complete).toMatchObject({ action: "complete", state: { terminal: true } });
    expect(() => reduceRetry(complete.state, {
      kind: "valid",
      objectiveCauseCorrected: false,
    })).toThrow("retry state is terminal");
  });
});
