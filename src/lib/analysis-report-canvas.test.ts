import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportRunHeaderSource from "./components/analysis/report-run-header.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import runSnapshotMessagesPanelSource from "./components/analysis/run-snapshot-messages-panel.svelte?raw";

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
    expect(reportCanvasSource).not.toContain('class="temporary-follow-up"');
    expect(reportCanvasSource).not.toContain("<ChatPanel");
    expect(reportSetupPanelSource).toContain("TemplateEditor");
    expect(reportSetupPanelSource).toContain("SourceGroupEditor");
    expect(reportSetupPanelSource).toContain("{#if !startingReport && !selectedRunIsActive}");
    expect(reportSetupPanelSource).toContain('class="template-editor-drawer"');
    expect(reportSetupPanelSource).toContain('class="group-editor-drawer"');
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
    expect(reportSourceSurfaceSource).toContain("Live source");
    expect(sourceReaderHeaderSource).toContain("View live source");
    expect(sourceReaderHeaderSource).toContain("Back to run snapshot");
    expect(reportSourceSurfaceSource).toContain("Snapshot pending");
    expect(reportSourceSurfaceSource).toContain("Snapshot unavailable");
    expect(reportSourceSurfaceSource).toContain("<SourceReaderHeader");
    expect(reportSourceSurfaceSource).toContain("<TelegramTimelineReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubeTranscriptReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubePlaylistReader");
  });

  it("keeps run snapshot reading bounded and snapshot-only", () => {
    expect(runSnapshotMessagesPanelSource).toContain("AnalysisRunMessage");
    expect(runSnapshotMessagesPanelSource).toContain("Load older snapshot messages");
    expect(runSnapshotMessagesPanelSource).toContain("hasMoreRunSnapshotMessages");
    expect(runSnapshotMessagesPanelSource).not.toContain("listSourceItems");
    expect(runSnapshotMessagesPanelSource).not.toContain("SourceMessagesPanel");
  });
});
