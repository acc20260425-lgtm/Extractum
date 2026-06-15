// @ts-nocheck
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));

function readProjectFile(relativePath: string) {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

describe("project runs screen", () => {
  it("adds a dedicated project runs route to the icon rail", () => {
    const iconRailSource = readProjectFile("src/lib/components/research-projects/IconRail.svelte");
    const routeSource = readProjectFile("src/routes/projects/runs/+page.svelte");

    expect(iconRailSource).toContain('href: "/projects/runs"');
    expect(routeSource).toContain("ProjectRunsScreen");
  });

  it("uses the Extractum SVAR grid for prompt-pack project runs with update and delete actions", () => {
    const screenSource = readProjectFile("src/lib/components/research-projects/ProjectRunsScreen.svelte");

    expect(screenSource).toContain("ExtractumDataGrid");
    expect(screenSource).toContain("listPromptPackRuns");
    expect(screenSource).toContain("updatePromptPackRun");
    expect(screenSource).toContain("deletePromptPackRun");
    expect(screenSource).not.toContain("listAnalysisRuns");
    expect(screenSource).not.toContain("ReportCanvas");
    expect(screenSource).not.toContain("ReportViewer");
  });

  it("uses the shared confirm modal before deleting or cancelling project runs", () => {
    const screenSource = readProjectFile("src/lib/components/research-projects/ProjectRunsScreen.svelte");

    expect(screenSource).toContain("openConfirmModal");
    expect(screenSource).toContain('title: "Delete project run?"');
    expect(screenSource).toContain('title: "Cancel active run?"');
    expect(screenSource).toContain('confirmLabel: "Delete"');
    expect(screenSource).toContain('confirmLabel: "Cancel run"');
    expect(screenSource).not.toContain("window.confirm");
  });

  it("renders a from-scratch prompt-pack report workspace under the grid", () => {
    const reportSource = readProjectFile("src/lib/components/research-projects/ProjectRunReportPanel.svelte");

    expect(reportSource).toContain("getPromptPackResult");
    expect(reportSource).toContain("listPromptPackRunStages");
    expect(reportSource).toContain("listPromptPackStageArtifacts");
    expect(reportSource).toContain("listPromptPackAuditEvents");
    expect(reportSource).toContain("getPromptPackValidationFindings");
    expect(reportSource).toContain("canonical");
    expect(reportSource).not.toContain("YoutubeSummaryResultView");
    expect(reportSource).not.toContain("report-viewer");
  });

  it("formats object-shaped prompt-pack report errors instead of rendering raw objects", () => {
    const reportSource = readProjectFile("src/lib/components/research-projects/ProjectRunReportPanel.svelte");

    expect(reportSource).toContain("formatAppError");
    expect(reportSource).toContain('formatAppError("loading project run report", cause)');
    expect(reportSource).toContain('formatAppError("loading project run artifact", cause)');
    expect(reportSource).not.toContain("String(cause)");
  });

  it("keeps diagnostics visible when a run has no canonical result yet", () => {
    const reportSource = readProjectFile("src/lib/components/research-projects/ProjectRunReportPanel.svelte");

    expect(reportSource).toContain("loadRunDiagnostics");
    expect(reportSource).toContain("expectedMissingResult");
    expect(reportSource).toContain("resultUnavailableMessage");
    expect(reportSource).toContain("Run was cancelled before producing a canonical result.");
    expect(reportSource).toContain("Run failed before producing a canonical result.");
    expect(reportSource).toContain("Run is still in progress. A canonical result is not available yet.");
  });
});
