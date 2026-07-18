export const PROTOCOL = Object.freeze({
  version: 1,
  baselineCommit: "24c313a767a25284123b24ea3a4b8c083007c817",
  candidateCommit: "b364756c7b5768d644321afeaeb81ec04e2481a4",
  baseSequence: Object.freeze(["A0", "B", "A1", "C", "A2", "D", "A3"]),
  conditionalSequence: Object.freeze(["E", "A4"]),
  samplesPerBlock: 7,
  warmupsPerBlock: 2,
  effectThresholdMs: 500,
  anchorRangeLimitMs: 300,
  sampleBandMs: 300,
  samplesRequiredInBand: 5,
  shellPercentCap: 5,
  commandTimeoutMs: 30 * 60 * 1_000,
  expectedCheckedPackage: "extractum",
  probeSuffix: "\n// process-shell-diagnostic-probe\n",
  cargoArgs: Object.freeze([
    "check",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--workspace",
    "--all-targets",
  ]),
});

function finiteNumber(value, label) {
  if (!Number.isFinite(value)) throw new Error(`${label} must be finite`);
  return value;
}

export function median(values) {
  if (values.length === 0) throw new Error("median requires samples");
  const sorted = values.map((value, index) => finiteNumber(value, `sample ${index}`)).sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? sorted[middle]
    : (sorted[middle - 1] + sorted[middle]) / 2;
}

export function summarizeBlock(samplesMs) {
  if (samplesMs.length !== PROTOCOL.samplesPerBlock) {
    throw new Error(`expected ${PROTOCOL.samplesPerBlock} samples, got ${samplesMs.length}`);
  }
  const canonical = [...samplesMs];
  const medianMs = median(canonical);
  const samplesWithinBand = canonical.filter(
    (sample) => Math.abs(sample - medianMs) <= PROTOCOL.sampleBandMs,
  ).length;
  return {
    samplesMs: canonical,
    medianMs,
    samplesWithinBand,
    stable: samplesWithinBand >= PROTOCOL.samplesRequiredInBand,
  };
}

function range(values) {
  return Math.max(...values) - Math.min(...values);
}

function metric(variant, leftAnchor, rightAnchor) {
  const aReferenceMs = (leftAnchor.medianMs + rightAnchor.medianMs) / 2;
  const deltaMs = variant.medianMs - aReferenceMs;
  const percentDelta = (100 * deltaMs) / aReferenceMs;
  return {
    variantMedianMs: variant.medianMs,
    aReferenceMs,
    deltaMs,
    percentDelta,
    material: deltaMs >= PROTOCOL.effectThresholdMs,
    shellCapFailed:
      deltaMs > PROTOCOL.effectThresholdMs || percentDelta > PROTOCOL.shellPercentCap,
  };
}

function summarizeRequired(blockSamples, names) {
  const summaries = {};
  for (const name of names) {
    if (!Object.hasOwn(blockSamples, name)) throw new Error(`missing block ${name}`);
    summaries[name] = summarizeBlock(blockSamples[name]);
  }
  return summaries;
}

function stabilityReasons(summaries, anchorNames) {
  const reasons = [];
  for (const [name, summary] of Object.entries(summaries)) {
    if (!summary.stable) reasons.push(`block_unstable:${name}`);
  }
  const anchorRangeMs = range(anchorNames.map((name) => summaries[name].medianMs));
  if (anchorRangeMs > PROTOCOL.anchorRangeLimitMs) reasons.push("anchor_range_exceeded");
  return { reasons, anchorRangeMs };
}

function buildMetrics(summaries, includeE) {
  const metrics = {
    B: metric(summaries.B, summaries.A0, summaries.A1),
    C: metric(summaries.C, summaries.A1, summaries.A2),
    D: metric(summaries.D, summaries.A2, summaries.A3),
  };
  if (includeE) metrics.E = metric(summaries.E, summaries.A3, summaries.A4);
  return metrics;
}

function contrasts(metrics) {
  return {
    membershipMs: metrics.B.deltaMs,
    edgeAfterMembershipMs: metrics.C.deltaMs - metrics.B.deltaMs,
    manifestAfterCMs: metrics.E ? metrics.E.deltaMs - metrics.C.deltaMs : null,
    dSpecificCompositeMs: metrics.E ? metrics.D.deltaMs - metrics.E.deltaMs : null,
    dAfterCCompositeMs: metrics.E ? null : metrics.D.deltaMs - metrics.C.deltaMs,
  };
}

export function evaluateAttempt(blockSamples) {
  const summaries = summarizeRequired(blockSamples, PROTOCOL.baseSequence);
  let stability = stabilityReasons(summaries, ["A0", "A1", "A2", "A3"]);
  if (stability.reasons.length > 0) {
    return { kind: "stability_invalid", eRequired: false, summaries, ...stability };
  }

  let metrics = buildMetrics(summaries, false);
  const eRequired = !metrics.B.material && !metrics.C.material && metrics.D.material;
  const hasE = Object.hasOwn(blockSamples, "E") || Object.hasOwn(blockSamples, "A4");
  if (eRequired && !hasE) {
    return { kind: "needs_e", eRequired: true, summaries, metrics, ...stability };
  }
  if (!eRequired && hasE) throw new Error("E/A4 present when E is not permitted");

  if (eRequired) {
    Object.assign(summaries, summarizeRequired(blockSamples, ["E", "A4"]));
    stability = stabilityReasons(summaries, ["A0", "A1", "A2", "A3", "A4"]);
    if (stability.reasons.length > 0) {
      return { kind: "stability_invalid", eRequired: true, summaries, ...stability };
    }
    metrics = buildMetrics(summaries, true);
  }

  const disagreement = Object.entries(metrics)
    .filter(([, value]) => value.material !== value.shellCapFailed)
    .map(([name]) => name);

  let classification;
  if (disagreement.length > 0) classification = "threshold_disagreement";
  else if (metrics.B.material) classification = "membership_configuration";
  else if (metrics.C.material) classification = "edge_related_configuration";
  else if (metrics.D.material && metrics.E?.material) classification = "manifest_related";
  else if (metrics.D.material) classification = "boundary_composite";
  else classification = "not_reproduced";

  return {
    kind: "valid",
    classification,
    disagreement,
    eRequired,
    summaries,
    metrics,
    contrasts: contrasts(metrics),
    ...stability,
  };
}

export function reduceRetry(state, invalidation) {
  if (state.terminal) throw new Error("retry state is terminal");
  const current = state.unexplainedStabilityInvalidCount;
  if (invalidation.kind === "valid") {
    const nextState = { unexplainedStabilityInvalidCount: current, terminal: true };
    return { action: "complete", unexplainedStabilityInvalidCount: current, state: nextState };
  }
  if (invalidation.kind === "stability_invalid") {
    if (invalidation.objectiveCauseCorrected) {
      return { action: "retry", unexplainedStabilityInvalidCount: current, state: { ...state } };
    }
    const nextCount = current + 1;
    if (nextCount >= 2) {
      const nextState = { unexplainedStabilityInvalidCount: nextCount, terminal: true };
      return {
        action: "environment_precision_insufficient",
        unexplainedStabilityInvalidCount: nextCount,
        state: nextState,
      };
    }
    const nextState = { unexplainedStabilityInvalidCount: nextCount, terminal: false };
    return { action: "retry", unexplainedStabilityInvalidCount: nextCount, state: nextState };
  }
  if (invalidation.kind === "infrastructure_invalid") {
    const action = invalidation.objectiveCauseCorrected ? "retry" : "await_correction";
    return { action, unexplainedStabilityInvalidCount: current, state: { ...state } };
  }
  throw new Error(`unknown invalidation kind: ${invalidation.kind}`);
}
