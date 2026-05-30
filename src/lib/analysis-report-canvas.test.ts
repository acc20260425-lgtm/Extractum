import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportRunHeaderSource from "./components/analysis/report-run-header.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import reportWorkspaceToolsSource from "./components/analysis/report-workspace-tools.svelte?raw";
import reportViewerSource from "./components/analysis/report-viewer.svelte?raw";
import snapshotGroupSourcesViewSource from "./components/analysis/snapshot-group-sources-view.svelte?raw";
import snapshotItemsViewSource from "./components/analysis/snapshot-items-view.svelte?raw";

const runCompanionTabsTag = "<" + "Run" + "CompanionTabs";

describe("report canvas component contract", () => {
  it("owns the central Report and Source modes", () => {
    expect(reportCanvasSource).toContain('class="report-canvas"');
    expect(reportCanvasSource).toContain('role="tablist"');
    expect(reportCanvasSource).toContain('aria-label="Report canvas mode"');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("report")');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("source")');
    expect(reportCanvasSource).toContain('{#if canvasMode === "report"}');
    expect(reportCanvasSource).toContain("<ReportSourceSurface");
    expect(reportCanvasSource).not.toContain(runCompanionTabsTag);
  });

  it("shows setup only when no run is open and report mode is selected", () => {
    expect(reportCanvasSource).toContain("{#if currentRun}");
    expect(reportCanvasSource).toContain("<ReportRunHeader");
    expect(reportCanvasSource).toContain("<ReportViewer");
    expect(reportCanvasSource).toContain("<ReportSetupPanel");
    expect(reportCanvasSource).toContain("reportLaunchDisabledReason");
    expect(reportSetupPanelSource).toContain("reportLaunchDisabledReason");
    expect(reportCanvasSource).not.toContain('class="temporary-follow-up"');
    expect(reportCanvasSource).not.toContain("<ChatPanel");
    expect(reportCanvasSource).toContain("<ReportWorkspaceTools");
    expect(reportSetupPanelSource).not.toContain("TemplateEditor");
    expect(reportSetupPanelSource).not.toContain("SourceGroupEditor");
    expect(reportSetupPanelSource).toContain("{#if !startingReport && !selectedRunIsActive}");
    expect(reportSetupPanelSource).not.toContain('class="template-editor-drawer"');
    expect(reportSetupPanelSource).not.toContain('class="group-editor-drawer"');
  });

  it("renders required opened-run header metadata", () => {
    expect(reportRunHeaderSource).toContain("Run #");
    expect(reportRunHeaderSource).toContain("runTargetLabel(currentRun)");
    expect(reportRunHeaderSource).toContain("currentRun.status");
    expect(reportRunHeaderSource).toContain('class="run-summary-strip"');
    expect(reportRunHeaderSource).toContain("<details");
    expect(reportRunHeaderSource).toContain("Run details");
    expect(reportRunHeaderSource).toContain("currentRun.created_at");
    expect(reportRunHeaderSource).toContain("currentRun.completed_at");
    expect(reportRunHeaderSource).toContain("promptTemplateLabel");
    expect(reportRunHeaderSource).toContain("prompt_template_name");
    expect(reportRunHeaderSource).toContain("prompt_template_version");
    expect(reportRunHeaderSource).toContain("currentRun.provider_profile");
    expect(reportRunHeaderSource).toContain("currentRun.provider");
    expect(reportRunHeaderSource).toContain("currentRun.model");
    expect(reportRunHeaderSource).toContain("sourceBasisLabel");
    expect(reportRunHeaderSource).toContain("youtubeCorpusModeLabel");
  });

  it("keeps report setup copy aware of existing saved runs", () => {
    expect(reportCanvasSource).toContain("{currentScopeHasSavedRuns}");
    expect(reportSetupPanelSource).toContain("currentScopeHasSavedRuns");
    expect(reportSetupPanelSource).toContain("Run another report");
    expect(reportSetupPanelSource).not.toContain("Build the first report for this workspace");
  });

  it("keeps snapshot and live source basis explicit", () => {
    expect(reportSourceSurfaceSource).toContain('sourceViewBasis === "live_source"');
    expect(reportSourceSurfaceSource).toContain('sourceViewBasis === "run_snapshot"');
    expect(reportSourceSurfaceSource).toContain("sourceBasisState={canvasSurface}");
    expect(reportSourceSurfaceSource).toContain("Live source");
    expect(sourceReaderHeaderSource).toContain("View live source");
    expect(sourceReaderHeaderSource).toContain("Back to run snapshot");
    expect(reportSourceSurfaceSource).toContain("Snapshot pending");
    expect(reportSourceSurfaceSource).toContain("Snapshot unavailable");
    expect(reportSourceSurfaceSource).toContain("<SourceReaderHeader");
    expect(reportSourceSurfaceSource).toContain("runSnapshotSubject");
    expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistReader");
  });

  it("passes YouTube comments and source activity callbacks through the report canvas", () => {
    expect(reportCanvasSource).toContain("onSyncYoutubeComments={onSyncYoutubeComments}");
    expect(reportCanvasSource).toContain("onCancelSourceJob={onCancelSourceJob}");
    expect(reportCanvasSource).toContain("{sourceJobs}");
    expect(reportSourceSurfaceSource).toContain("onSyncYoutubeComments");
  });

  it("passes Telegram topic state into the source surface", () => {
    expect(reportCanvasSource).toContain("{sourceTopics}");
    expect(reportCanvasSource).toContain("{loadingSourceTopics}");
    expect(reportCanvasSource).toContain("{selectedTopicKey}");
    expect(reportCanvasSource).toContain("{showTopicSelector}");
    expect(reportCanvasSource).toContain("onChangeSelectedTopicKey={onChangeSelectedTopicKey}");
  });

  it("labels source surfaces without repeating the selected workspace title", () => {
    expect(sourceReaderHeaderSource).toContain("surfaceLabel");
    expect(reportSourceSurfaceSource).toContain("surfaceLabel=");
    expect(reportSourceSurfaceSource).toContain("Group sources");
    expect(reportSourceSurfaceSource).toContain("Source material");
  });

  it("keeps run snapshot reading bounded and snapshot-only", () => {
    expect(reportSourceSurfaceSource).toContain("snapshotBrowserData");
    expect(reportSourceSurfaceSource).toContain("hasMoreRunSnapshotMessages");
    expect(reportSourceSurfaceSource).toContain("onLoadMoreRunSnapshotMessages");
    expect(snapshotItemsViewSource).toContain("SourceReaderItem");
    expect(snapshotItemsViewSource).toContain("Load older snapshot messages");
    expect(snapshotItemsViewSource).toContain("Snapshot items are limited to frozen rows loaded for this run");
    expect(snapshotItemsViewSource).not.toContain("SourceItem");
    expect(snapshotItemsViewSource).not.toContain("listSourceItems");
    expect(snapshotGroupSourcesViewSource).toContain("Load older snapshot messages");
    expect(snapshotGroupSourcesViewSource).not.toContain("hasMoreBySource");
  });

  it("keeps source-group run snapshots pageable through the snapshot browser", () => {
    expect(reportSourceSurfaceSource).toContain("snapshotBrowserData");
    expect(reportSourceSurfaceSource).toContain("hasMoreRunSnapshotMessages");
    expect(reportSourceSurfaceSource).toContain("onLoadMoreRunSnapshotMessages");
    expect(snapshotGroupSourcesViewSource).toContain("Load older snapshot messages");
    expect(reportSourceSurfaceSource).not.toContain("hasMoreBySource={{}}");
  });

  it("uses real chat availability in the report toolbar", () => {
    expect(reportCanvasSource).toContain("{chatAvailability}");
    expect(reportViewerSource).toContain("chatAvailability");
    expect(reportViewerSource).not.toContain('currentRun?.status === "completed" ? "Chat ready"');
  });

  it("keeps workspace tools reachable before setup report and source bodies", () => {
    const toolsStart = reportCanvasSource.indexOf("<ReportWorkspaceTools");
    const reportBodyStart = reportCanvasSource.indexOf('{#if canvasMode === "report"}');
    const sourceBodyStart = reportCanvasSource.indexOf("<ReportSourceSurface");

    expect(toolsStart).toBeGreaterThan(0);
    expect(reportBodyStart).toBeGreaterThan(0);
    expect(sourceBodyStart).toBeGreaterThan(0);
    expect(toolsStart).toBeLessThan(reportBodyStart);
    expect(toolsStart).toBeLessThan(sourceBodyStart);
    expect(reportCanvasSource).not.toContain('class="opened-run-management"');
    expect(reportCanvasSource).toContain("<TemplateEditor");
    expect(reportCanvasSource).toContain("<SourceGroupEditor");
    expect(reportCanvasSource.match(/<TemplateEditor/g)?.length ?? 0).toBe(1);
    expect(reportCanvasSource.match(/<SourceGroupEditor/g)?.length ?? 0).toBe(1);
    expect(reportCanvasSource).toContain("templateEditorOpen");
    expect(reportCanvasSource).toContain("groupEditorOpen");
    expect(reportWorkspaceToolsSource).toContain("Export for NotebookLM");
    expect(reportWorkspaceToolsSource).toContain("Edit templates");
    expect(reportWorkspaceToolsSource).toContain("Edit groups");
  });

  it("derives NotebookLM export availability from live canvas source or group", () => {
    expect(reportCanvasSource).toContain("showNotebookLmExport");
    expect(reportCanvasSource).toContain("currentSource !== null || currentGroup !== null");
    expect(reportCanvasSource).toContain("canExportNotebookLm");
    expect(reportCanvasSource).toContain("currentSource !== null && !exportingNotebookLm");
    expect(reportCanvasSource).toContain("currentGroup && !currentSource ? sourceGroupNotebookLmExportReason : null");
    expect(reportCanvasSource).toContain("Source-group NotebookLM export is not implemented yet.");
    expect(reportCanvasSource).toContain("<ReportWorkspaceTools");
    expect(reportCanvasSource).toContain("{showNotebookLmExport}");
    expect(reportCanvasSource).toContain("{canExportNotebookLm}");
    expect(reportCanvasSource).toContain("exportDisabledReason={notebookLmExportDisabledReason}");
    expect(reportCanvasSource).toContain("<NotebookLmExportDialog");
    expect(reportCanvasSource.match(/<NotebookLmExportDialog/g)?.length ?? 0).toBe(1);
    expect(reportCanvasSource).toContain("source={currentSource}");
  });
});
