import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportWorkspaceToolsSource from "./components/analysis/report-workspace-tools.svelte?raw";
import sourceSwitcherPanelSource from "./components/analysis/source-switcher-panel.svelte?raw";
import sourceActivityViewSource from "./components/analysis/source-activity-view.svelte?raw";
import universalItemsViewSource from "./components/analysis/universal-items-view.svelte?raw";
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";

describe("analysis priority UX contract", () => {
  it("keeps the report canvas top chrome compact and action-oriented", () => {
    expect(reportCanvasSource).toContain('class="canvas-context-bar"');
    expect(reportCanvasSource).toContain('aria-label="Analysis context"');
    expect(reportCanvasSource).toContain('class="canvas-actions-row"');
    expect(reportCanvasSource).toContain("showInlineWorkspaceTools");
    expect(reportWorkspaceToolsSource).toContain("compact = false");
    expect(reportWorkspaceToolsSource).toContain("class:compact={compact}");
    expect(reportWorkspaceToolsSource).toContain('aria-label="Workspace actions"');
  });

  it("keeps the source switcher primarily focused on source selection", () => {
    expect(sourceSwitcherPanelSource).toContain('class="source-row-operations"');
    expect(sourceSwitcherPanelSource).toContain("<summary>Source operations</summary>");
    expect(sourceSwitcherPanelSource).toContain("Manage operational state in the Activity tab.");
    expect(sourceSwitcherPanelSource).not.toContain('class="row-actions"');
  });

  it("makes source activity the visible home for source operations", () => {
    expect(sourceActivityViewSource).toContain('class="activity-action-grid"');
    expect(sourceActivityViewSource).toContain("Sync source");
    expect(sourceActivityViewSource).toContain("Start Takeout import");
    expect(sourceActivityViewSource).toContain("Detailed jobs");
  });

  it("turns loaded items into a reader instead of a raw dump", () => {
    expect(universalItemsViewSource).toContain("function itemPreviewText");
    expect(universalItemsViewSource).toContain("function itemContextLine");
    expect(universalItemsViewSource).toContain('class="item-preview"');
    expect(universalItemsViewSource).toContain('class:media-only={!item.content && item.hasMedia}');
    expect(universalItemsViewSource).toContain("Media-only item");
  });

  it("keeps run filters progressive when no runs exist", () => {
    expect(runsTabSource).toContain("hasAnyRuns");
    expect(runsTabSource).toContain("showRunsToolbar");
    expect(runsTabSource).toContain('class="runs-empty-guidance"');
    expect(runsTabSource).toContain("Run a report to create the first saved workspace.");
  });
});
