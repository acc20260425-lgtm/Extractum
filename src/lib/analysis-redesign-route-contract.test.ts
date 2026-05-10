import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import compactRailSource from "./components/analysis/compact-source-rail.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupSource from "./components/analysis/report-setup-panel.svelte?raw";
import reportRunHeaderSource from "./components/analysis/report-run-header.svelte?raw";
import runCompanionSource from "./components/analysis/run-companion-tabs.svelte?raw";
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";

describe("analysis redesign final route contract", () => {
  it("renders the approved three-zone analysis workspace", () => {
    expect(analysisPageSource).toContain(
      'import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";',
    );
    expect(analysisPageSource).toContain(
      'import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";',
    );
    expect(analysisPageSource).toContain(
      'import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";',
    );
    expect(analysisPageSource).toContain("<CompactSourceRail");
    expect(analysisPageSource).toContain("<ReportCanvas");
    expect(analysisPageSource).toContain("<RunCompanionTabs");
    expect(analysisPageSource).toContain('class="analysis-workspace"');
    expect(analysisPageSource).toContain(".analysis-workspace");
    expect(analysisPageSource).toContain("grid-template-columns");
    expect(analysisPageSource).toContain("companion-slot");
  });

  it("does not render the legacy wide analysis workspace surfaces", () => {
    expect(analysisPageSource).not.toContain(
      'import WorkspaceRail from "$lib/components/analysis/workspace-rail.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceMain from "$lib/components/analysis/workspace-main.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceInspector from "$lib/components/analysis/workspace-inspector.svelte";',
    );
    expect(analysisPageSource).not.toContain("<WorkspaceRail");
    expect(analysisPageSource).not.toContain("<WorkspaceMain");
    expect(analysisPageSource).not.toContain("<WorkspaceInspector");
    expect(analysisPageSource).not.toContain("inspectorMode");
  });

  it("keeps source switching, run opening, and canvas switching on separate state paths", () => {
    expect(analysisPageSource).toContain("selectSourceWorkspace");
    expect(analysisPageSource).toContain("selectSourceGroupWorkspace");
    expect(analysisPageSource).toContain("openRunWorkspaceState");
    expect(analysisPageSource).toContain("workspaceUiState.canvasMode");
    expect(analysisPageSource).toContain("workspaceUiState.sourceViewBasis");
    expect(analysisPageSource).toContain("workspaceUiState.companionTab");
    expect(analysisPageSource).toContain("function changeCanvasMode");
    expect(analysisPageSource).toContain("function viewLiveSource");
    expect(analysisPageSource).toContain("function backToRunSnapshot");
    expect(analysisPageSource).not.toContain("clearCurrentRunForCanvasSwitch");
    expect(analysisPageSource).not.toContain("clearCurrentRunForSourceFilter");
  });

  it("keeps the collapsed rail source-scoped and quiet", () => {
    expect(compactRailSource).toContain('class="compact-source-rail"');
    expect(compactRailSource).toContain("workspaceSelection: WorkspaceSelection");
    expect(compactRailSource).toContain('ariaLabel="Open source switcher"');
    expect(compactRailSource).toContain("context-primary-action");
    expect(compactRailSource).toContain("criticalSourceStatus");
    expect(compactRailSource).not.toContain("AppSidebar");
    expect(compactRailSource).not.toContain("Settings");
    expect(compactRailSource).not.toContain("Accounts");
    expect(compactRailSource).not.toContain("Manage sources</Button>");
    expect(compactRailSource).not.toContain("Transcript unavailable");
    expect(compactRailSource).not.toContain("Comments unavailable");
  });

  it("keeps ReportCanvas as the report/source mode owner", () => {
    expect(reportCanvasSource).toContain('class="report-canvas"');
    expect(reportCanvasSource).toContain('role="tablist"');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("report")');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("source")');
    expect(reportCanvasSource).toContain("<ReportRunHeader");
    expect(reportCanvasSource).toContain("<ReportSetupPanel");
    expect(reportCanvasSource).toContain("<ReportSourceSurface");
    expect(reportCanvasSource).not.toContain("<RunCompanionTabs");
    expect(reportCanvasSource).not.toContain("<WorkspaceInspector");
  });

  it("keeps report setup out of the primary opened-run reading surface", () => {
    expect(reportCanvasSource).toContain("currentRun");
    expect(reportCanvasSource).toContain("{#if currentRun}");
    expect(reportCanvasSource).toContain("{:else}");
    expect(reportSetupSource).toContain("template");
    expect(reportSetupSource).toContain("Run report");
    expect(reportSetupSource).toContain("template-editor-drawer");
    expect(reportSetupSource).toContain("group-editor-drawer");
    expect(reportRunHeaderSource).toContain("prompt_template_name");
    expect(reportRunHeaderSource).toContain("prompt_template_version");
    expect(reportRunHeaderSource).toContain("provider_profile");
    expect(reportRunHeaderSource).toContain("youtube_corpus_mode");
    expect(reportRunHeaderSource).not.toContain("Template editor");
  });

  it("keeps companion tabs as Evidence, Chat, and Runs only", () => {
    expect(runCompanionSource).toContain('role="tablist"');
    expect(runCompanionSource).toContain('aria-label="Run companion tabs"');
    expect(runCompanionSource).toContain('onChangeCompanionTab("evidence")');
    expect(runCompanionSource).toContain('onChangeCompanionTab("chat")');
    expect(runCompanionSource).toContain('onChangeCompanionTab("runs")');
    expect(runCompanionSource).toContain("<RunEvidenceTab");
    expect(runCompanionSource).toContain("<RunChatTab");
    expect(runCompanionSource).toContain("<RunCompanionRunsTab");
    expect(runCompanionSource).not.toContain("Source activity");
  });

  it("keeps Runs focused on analysis report runs and durable filters", () => {
    expect(runsTabSource).toContain("filterCompanionRuns");
    expect(runsTabSource).toContain("Search runs");
    expect(runsTabSource).toContain("Current scope");
    expect(runsTabSource).toContain("Date range");
    expect(runsTabSource).toContain("Provider filter");
    expect(runsTabSource).toContain("Template filter");
    expect(runsTabSource).not.toContain("SourceJobRecord");
    expect(runsTabSource).not.toContain("takeoutJobs");
    expect(runsTabSource).not.toContain("sourceJobs");
  });
});
