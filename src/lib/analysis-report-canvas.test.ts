import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportRunHeaderSource from "./components/analysis/report-run-header.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import runSnapshotMessagesPanelSource from "./components/analysis/run-snapshot-messages-panel.svelte?raw";

describe("report canvas component contract", () => {
  it("owns the central Report and Source modes", () => {
    expect(reportCanvasSource).toContain('class="report-canvas"');
    expect(reportCanvasSource).toContain('role="tablist"');
    expect(reportCanvasSource).toContain('aria-label="Report canvas mode"');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("report")');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("source")');
    expect(reportCanvasSource).toContain('{#if canvasMode === "report"}');
    expect(reportCanvasSource).toContain("<ReportSourceSurface");
    expect(reportCanvasSource).not.toContain("<RunCompanionTabs");
  });

  it("shows setup only when no run is open and report mode is selected", () => {
    expect(reportCanvasSource).toContain("{#if currentRun}");
    expect(reportCanvasSource).toContain("<ReportRunHeader");
    expect(reportCanvasSource).toContain("<ReportViewer");
    expect(reportCanvasSource).toContain("<ReportSetupPanel");
    expect(reportCanvasSource).toContain('class="temporary-follow-up"');
    expect(reportSetupPanelSource).toContain("TemplateEditor");
    expect(reportSetupPanelSource).toContain("SourceGroupEditor");
    expect(reportSetupPanelSource).toContain('class="template-editor-drawer"');
    expect(reportSetupPanelSource).toContain('class="group-editor-drawer"');
  });

  it("renders required opened-run header metadata", () => {
    expect(reportRunHeaderSource).toContain("Run #");
    expect(reportRunHeaderSource).toContain("runTargetLabel(currentRun)");
    expect(reportRunHeaderSource).toContain("currentRun.status");
    expect(reportRunHeaderSource).toContain("currentRun.created_at");
    expect(reportRunHeaderSource).toContain("currentRun.completed_at");
    expect(reportRunHeaderSource).toContain("currentRun.prompt_template_name");
    expect(reportRunHeaderSource).toContain("currentRun.prompt_template_version");
    expect(reportRunHeaderSource).toContain("currentRun.provider_profile");
    expect(reportRunHeaderSource).toContain("currentRun.provider");
    expect(reportRunHeaderSource).toContain("currentRun.model");
    expect(reportRunHeaderSource).toContain("sourceBasisLabel");
    expect(reportRunHeaderSource).toContain("youtubeCorpusModeLabel");
  });

  it("keeps snapshot and live source basis explicit", () => {
    expect(reportSourceSurfaceSource).toContain('sourceViewBasis === "live_source"');
    expect(reportSourceSurfaceSource).toContain('sourceViewBasis === "run_snapshot"');
    expect(reportSourceSurfaceSource).toContain("Live source");
    expect(reportSourceSurfaceSource).toContain("View live source");
    expect(reportSourceSurfaceSource).toContain("Back to run snapshot");
    expect(reportSourceSurfaceSource).toContain("Snapshot pending");
    expect(reportSourceSurfaceSource).toContain("Snapshot unavailable");
    expect(reportSourceSurfaceSource).toContain("<RunSnapshotMessagesPanel");
    expect(reportSourceSurfaceSource).toContain("<SourceContextPanel");
    expect(reportSourceSurfaceSource).toContain("<YoutubeSourceDetail");
    expect(reportSourceSurfaceSource).toContain("<YoutubePlaylistDetail");
  });

  it("keeps run snapshot reading bounded and snapshot-only", () => {
    expect(runSnapshotMessagesPanelSource).toContain("AnalysisRunMessage");
    expect(runSnapshotMessagesPanelSource).toContain("Load older snapshot messages");
    expect(runSnapshotMessagesPanelSource).toContain("hasMoreRunSnapshotMessages");
    expect(runSnapshotMessagesPanelSource).not.toContain("listSourceItems");
    expect(runSnapshotMessagesPanelSource).not.toContain("SourceMessagesPanel");
  });
});
