import { describe, expect, it } from "vitest";

import focusedLoopDesignRaw from "../../docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md?raw";
import crateRoadmapRaw from "../../docs/superpowers/specs/2026-07-17-crate-roadmap.md?raw";
import processBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md?raw";
import shellCapRevisionRaw from "../../docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md?raw";
import anomalyV2DesignRaw from "../../docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md?raw";
import reapplicationPlanRaw from "../../docs/superpowers/plans/2026-07-18-extractum-process-reapplication.md?raw";

const normalize = (value: string) => value.replace(/\r\n/g, "\n");
const compact = (value: string) => normalize(value).replace(/\s+/g, " ");
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
const shellCapRevision = compact(shellCapRevisionRaw);
const anomalyV2Design = compact(anomalyV2DesignRaw);
const reapplicationPlan = compact(reapplicationPlanRaw);
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
      "Repeated ordinary workspace checks at or above 15,000 ms trigger a separate owner-approved performance investigation",
    );
    expect(failurePolicy).toContain("There is no protocol-mandated retry");
    expect(failurePolicy).toContain(
      "Timing alone cannot reject or revert the slice",
    );
    expect(focusedLoopDesign).not.toContain("### Retention gates");
  });

  it("records the canceled Phase 3 and fresh-design requirement for Phase 4", () => {
    expect(crateRoadmap).toContain(
      "**Status:** Strategic reference; revised and owner-approved 2026-07-19",
    );
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
      "was canceled by the project owner before execution on 2026-07-19",
    );
    expect(phase3Roadmap).toContain(
      "No process crate, post-reapplication baseline, or cumulative-ledger entry exists",
    );
    expect(phase3Roadmap).toContain(
      "Any future `extractum-process` attempt starts as a new phase",
    );
    expect(phase4Roadmap).toContain(
      "Phase 4 — `extractum-gemini-browser` (awaiting a new boundary design)",
    );
    expect(phase4Roadmap).toContain(
      "requires a fresh owner-approved boundary",
    );
    expect(phase4Roadmap).toContain("It has no Phase 3 timing prerequisite");
    expect(shellCapRevision).toContain(
      "Superseded 2026-07-19; historical policy record",
    );
    expect(shellCapRevision).toContain(
      "must not be used as current execution authority",
    );
    expect(processBoundaryDesign).toContain(
      "execution authority withdrawn 2026-07-19",
    );
    expect(processBoundaryDesign).toContain("not authority to replay");
    expect(processBoundaryDesign).toContain("Phase 3 or start Phase 4");
    expect(reapplicationPlan).toContain(
      "CANCELED 2026-07-19 — DO NOT EXECUTE OR RESUME",
    );
    expect(reapplicationPlan).toContain(
      "withdrew the complete plan before any task was executed",
    );
    expect(anomalyV2Design).toContain("`moot` for the current crate roadmap");
  });
});
