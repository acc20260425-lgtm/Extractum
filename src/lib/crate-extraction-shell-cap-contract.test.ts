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
const sectionFrom = (value: string, start: string) => {
  const startIndex = value.indexOf(start);
  if (startIndex < 0) {
    throw new Error(`Missing policy section: ${start}`);
  }
  return value.slice(startIndex);
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
const roadmapBudget = sectionBetween(
  crateRoadmap,
  "## Roadmap Shell Budget",
  "## Target Crate Map",
);
const phase3Roadmap = sectionBetween(
  crateRoadmap,
  "### Phase 3 —",
  "### Phase 4 —",
);
const phase4Roadmap = sectionBetween(
  crateRoadmap,
  "### Phase 4 —",
  "### Phase 5 —",
);
const anomalyDisposition = sectionBetween(
  anomalyV2Design,
  "## Current Roadmap Disposition",
  "**Archived v1 protocol commit:**",
);
const phase3Revision = sectionBetween(
  shellCapRevision,
  "## Phase 3 Consequence",
  "## Documentation and Contract Changes",
);
const revisionContract = sectionBetween(
  shellCapRevision,
  "## Documentation and Contract Changes",
  "## Verification",
);
const processShellRegistry = sectionFrom(
  valueRegistry,
  "## Process-shell diagnostic artifact classifications",
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
    expect(retentionPolicy).toContain(
      "frozen historical tree/blob identity manifest",
    );
    expect(retentionPolicy).toContain("any mismatch");
    expect(samplingPolicy).toContain("at least four of the five samples");
    expect(samplingPolicy).toContain("absolute deviation <= 300 ms");
    expect(samplingPolicy).toContain("not a performance failure");
    expect(samplingPolicy).toContain("quiet-window preflight");
    expect(failurePolicy).toContain("Measurement invalidation");
    expect(failurePolicy).toContain("15,000 ms cumulative shell ceiling");
    expect(retentionPolicy).not.toContain("- 5%; and");
    expect(retentionPolicy).not.toContain(
      "0.5 seconds in absolute median wall time",
    );
    expect(retentionPolicy).not.toContain("800 ms / 8%");
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
    expect(roadmapBudget).toContain("15,000 ms");
    expect(roadmapBudget).toContain("5,865 ms");
    expect(roadmapBudget.match(/\| Reapplied Phase 3 \|/g)).toHaveLength(1);
    expect(roadmapBudget).toContain(
      "| Reapplied Phase 3 | pending valid post-reapplication median | " +
        "pending | non-gating measurement required before Phase 4 timing |",
    );
    expect(phase3Roadmap).toContain(
      "Phase 3 — `extractum-process` (approved for exact-candidate reapplication)",
    );
    expect(phase3Roadmap).toContain("non-gating before/after");
    expect(phase3Roadmap).toContain("shell samples and validity counts");
    expect(phase3Roadmap).toContain(
      "frozen historical tree/blob identity manifest",
    );
    expect(phase3Roadmap).toContain("any mismatch");
    expect(phase3Roadmap).toContain("stop the exact-candidate path");
    expect(phase3Roadmap).toContain(
      "requires a separately approved plan",
    );
    expect(phase3Roadmap).toContain(
      "fresh preregistered timing under the revised",
    );
    expect(phase3Roadmap).toContain(
      "2026-07-17-extractum-process-extraction.md",
    );
    expect(phase4Roadmap).toContain(
      "Phase 4 remains blocked until the exact Phase 3 candidate is integrated",
    );
    expect(phase4Roadmap).toContain(
      "valid shell baseline exists for Phase 4 measurement",
    );
    expect(phase4Roadmap).toContain(
      "No additional v2/v3 diagnostic approval is required",
    );
    expect(phase4Roadmap).not.toContain(
      "Additional v2/v3 diagnostic approval is required",
    );
    expect(anomalyV2Design).toContain(
      "**Status:** `moot` for the current crate roadmap",
    );
    expect(anomalyDisposition).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(anomalyDisposition).toContain("must not run as roadmap prerequisites");
    expect(anomalyDisposition).not.toContain("must run as roadmap prerequisites");
    expect(anomalyDisposition).toContain("current v1 harness is not");
    expect(anomalyDisposition).toContain("production-ready infrastructure");
    expect(processShellRegistry).toContain(
      "| `moot` | roadmap disposition | Moot | " +
        "The approved anomaly protocol no longer controls the crate roadmap after an explicit owner policy revision; " +
        "its design remains preserved for a separately approved precision/causality task. | " +
        "shell-cap revision / crate roadmap | terminal | none | n/a | yes | v2 design, crate roadmap |",
    );
    expect(processShellRegistry).toContain(
      "`moot` does not classify an experimental `decision.json`",
    );
    expect(processShellRegistry).toContain("`moot` is documentation-only");
    expect(processShellRegistry).toContain("not persisted in SQLite");
    expect(processShellRegistry).toContain("not exposed through a product API");
    expect(processShellRegistry).toContain("not rendered in the UI");
    expect(processShellRegistry).toContain("not used by product fixtures");
    expect(shellCapRevision).toContain(
      "**Status:** Implemented; current shell-cap authority",
    );
    expect(phase3Revision).toContain(
      "frozen historical tree/blob identity manifest",
    );
    expect(phase3Revision).toContain("any mismatch");
    expect(revisionContract).toContain(
      "document-only `moot` roadmap disposition",
    );
    expect(revisionContract).not.toContain("or value-registry entry");
  });
});
