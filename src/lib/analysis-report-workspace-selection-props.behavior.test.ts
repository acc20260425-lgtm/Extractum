import { describe, expect, it } from "vitest";
import {
  reportCanvasWorkspaceProps,
} from "$lib/analysis-report-props";
import {
  reportSetupCompatibility,
  reportSourceCompatibility,
} from "$lib/analysis-report-leaf-compatibility";

describe("analysis report workspace selection props", () => {
  it("passes workspace selection into the report canvas instead of legacy scope ids", () => {
    const selection = { kind: "source_group" as const, sourceGroupId: 23 };
    const props = reportCanvasWorkspaceProps(selection);

    expect(props.workspaceSelection).toBe(selection);
    expect(props.workspaceSelection.kind === "source_group" && props.workspaceSelection.sourceGroupId).toBe(23);
  });

  it("keeps setup and source compatibility projections inside leaf components", () => {
    const setupSource = reportSetupCompatibility({ kind: "source", sourceId: 17 });
    const setupGroup = reportSetupCompatibility({ kind: "source_group", sourceGroupId: 23 });
    const sourceSource = reportSourceCompatibility({ kind: "source", sourceId: 17 });
    const sourceGroup = reportSourceCompatibility({ kind: "source_group", sourceGroupId: 23 });
    const setupNone = reportSetupCompatibility({ kind: "none" });
    const sourceNone = reportSourceCompatibility({ kind: "none" });

    expect(setupSource.analysisScope).toBe("single_source");
    expect(setupSource.workspaceEyebrow).toBe("Source workspace");
    expect(setupSource.analysisModeLabel).toBe("Single source");
    expect(setupSource.runDescription).toBe("Run against the selected source.");
    expect(setupGroup.analysisScope).toBe("source_group");
    expect(setupGroup.workspaceEyebrow).toBe("Source group workspace");
    expect(setupGroup.analysisModeLabel).toBe("Group analysis");
    expect(setupGroup.runDescription).toBe("Run across the saved group.");
    expect(setupNone.analysisScope).toBe("single_source");
    expect(setupNone.workspaceEyebrow).toBe("Source workspace");
    expect(setupNone.analysisModeLabel).toBe("Single source");
    expect(setupNone.runDescription).toBe("Run against the selected source.");
    expect(sourceSource.readerSurfaceLabel).toBe("Source material");
    expect(sourceSource.showSingleSource).toBe(true);
    expect(sourceSource.showSourceGroup).toBe(false);
    expect(sourceGroup.readerSurfaceLabel).toBe("Group sources");
    expect(sourceGroup.showSingleSource).toBe(false);
    expect(sourceGroup.showSourceGroup).toBe(true);
    expect(sourceNone.readerSurfaceLabel).toBe("Source material");
  });
});
