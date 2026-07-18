import { describe, expect, it } from "vitest";

import focusedLoopDesignRaw from "../../docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md?raw";
import crateRoadmapRaw from "../../docs/superpowers/specs/2026-07-17-crate-roadmap.md?raw";
import shellCapRevisionRaw from "../../docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md?raw";
import anomalyV2DesignRaw from "../../docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md?raw";
import valueRegistryRaw from "../../docs/value-registry.md?raw";
import processBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md?raw";

const normalize = (value: string) => value.replace(/\r\n/g, "\n");
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
const shellCapRevision = normalize(shellCapRevisionRaw);
const anomalyV2Design = normalize(anomalyV2DesignRaw);
const valueRegistry = normalize(valueRegistryRaw);
const processBoundaryDesign = normalize(processBoundaryDesignRaw);
const samplingPolicy = sectionBetween(
  focusedLoopDesign,
  "### Sampling",
  "### Retention gates",
);
const retentionPolicy = sectionBetween(
  focusedLoopDesign,
  "### Retention gates",
  "## Failure Classification",
);
const failurePolicy = sectionBetween(
  focusedLoopDesign,
  "## Failure Classification",
  "## Repository Enforcement",
);

describe("crate extraction shell-cap repository policy", () => {
  it("pins the revised focused-loop thresholds and validity rule", () => {
    expect(focusedLoopDesign).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(retentionPolicy).toContain("no more than both");
    expect(retentionPolicy).toContain("- 20%; and");
    expect(retentionPolicy).toContain("- 2,000 ms in absolute median wall time");
    expect(retentionPolicy).toContain("Values exactly at 2,000 ms / 20% pass");
    expect(retentionPolicy).toContain("9,135 ms");
    expect(retentionPolicy).toContain("15,000 ms");
    expect(retentionPolicy).toContain("no marginal-performance repeat");
    expect(retentionPolicy).toContain("exact Phase 3 reapplication");
    expect(retentionPolicy).toContain("cannot produce a performance no-go");
    expect(samplingPolicy).toContain("at least four of the five samples");
    expect(samplingPolicy).toContain("within 300 ms of its own median");
    expect(samplingPolicy).toContain("not a performance failure");
    expect(samplingPolicy).toContain("quiet-window preflight");
    expect(failurePolicy).toContain("Measurement invalidation");
    expect(failurePolicy).toContain("15,000 ms cumulative shell ceiling");
    expect(focusedLoopDesign).toContain(
      "2026-07-17-process-and-gemini-browser-crate-boundary-design.md",
    );
    expect(focusedLoopDesign).toContain(
      "shell-cap and marginal-repeat clauses",
    );
    expect(retentionPolicy).toContain("at least 25%");
    expect(retentionPolicy).toContain("at least 2.0 seconds");
    expect(processBoundaryDesign).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(processBoundaryDesign).toContain("architecture and correctness");
    expect(processBoundaryDesign).toContain("requirements remain active");
  });

  it("records the cumulative roadmap and moot anomaly disposition", () => {
    expect(crateRoadmap).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(crateRoadmap).toContain("15,000 ms");
    expect(crateRoadmap).toContain("5,865 ms");
    expect(crateRoadmap).toContain(
      "| Reapplied Phase 3 | pending valid post-reapplication median |",
    );
    expect(crateRoadmap).toContain(
      "Phase 3 — `extractum-process` (approved for exact-candidate reapplication)",
    );
    expect(crateRoadmap).toContain("non-gating before/after");
    expect(crateRoadmap).toContain("shell samples and validity counts");
    expect(crateRoadmap).toContain(
      "If reconstruction differs materially from `b364756c`",
    );
    expect(crateRoadmap).toContain(
      "requires a separately approved plan",
    );
    expect(crateRoadmap).toContain("fresh preregistered timing under the revised");
    expect(crateRoadmap).toContain(
      "Phase 4 remains blocked until the exact Phase 3 candidate is integrated",
    );
    expect(anomalyV2Design).toContain(
      "**Status:** `moot` for the current crate roadmap",
    );
    expect(anomalyV2Design).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(anomalyV2Design).toContain("current v1 harness is not");
    expect(anomalyV2Design).toContain("production-ready infrastructure");
    expect(valueRegistry).toContain("| `moot` | roadmap disposition |");
    expect(valueRegistry).toContain("`moot` is documentation-only");
    expect(shellCapRevision).toContain(
      "**Status:** Implemented; current shell-cap authority",
    );
  });
});
