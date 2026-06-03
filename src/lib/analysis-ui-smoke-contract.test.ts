// @ts-nocheck
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import packageJson from "../../package.json";
import smokeScriptSource from "../../scripts/analysis-smoke.mjs?raw";
import helperSource from "../../scripts/analysis-smoke-helpers.mjs?raw";
import verifyScriptSource from "../../scripts/verify.mjs?raw";
import desktopDialogSource from "./components/desktop-dialog.svelte?raw";
import notebookLmExportDialogSource from "./components/analysis/notebooklm-export-dialog.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import reportWorkspaceToolsSource from "./components/analysis/report-workspace-tools.svelte?raw";
import sourceBrowserShellSource from "./components/analysis/source-browser-shell.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import sourceSwitcherPanelSource from "./components/analysis/source-switcher-panel.svelte?raw";
import compactSourceRailSource from "./components/analysis/compact-source-rail.svelte?raw";
import runCompanionTabsSource from "./components/analysis/run-companion-tabs.svelte?raw";
import runCompanionRunsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";
import { sourceBrowserTabsForSubject } from "./source-browser-model";

const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));
const sharedSmokePrimitiveFiles = new Set([
  "src/lib/components/ui/Button.svelte",
  "src/lib/components/desktop-dialog.svelte",
]);

function collectSourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const fullPath = path.join(dir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) return collectSourceFiles(fullPath);
    return fullPath;
  });
}

describe("analysis UI smoke harness contract", () => {
  it("exposes the smoke command as opt-in and keeps verify free of it", () => {
    expect(packageJson.scripts["smoke:analysis"]).toBe("node scripts/analysis-smoke.mjs");
    expect(verifyScriptSource).not.toContain("smoke:analysis");
    expect(verifyScriptSource).not.toContain("analysis-smoke");
  });

  it("keeps the smoke runner organized around deterministic named steps", () => {
    expect(smokeScriptSource).toContain("sourceBrowserSmokeSteps");
    expect(smokeScriptSource).toContain("analysisWorkspaceParitySteps");
    expect(smokeScriptSource).toContain("savedRunAffordanceSmokeSteps");
    expect(smokeScriptSource).toContain("source-browser.youtube-video-tabs");
    expect(smokeScriptSource).toContain("saved-runs-affordance.rows");
    expect(smokeScriptSource).toContain("saved-runs-affordance.missing-legacy");
    expect(smokeScriptSource).toContain("saved-runs-affordance.capture-failed");
    expect(smokeScriptSource).toContain("workspace-parity.telegram-source-group-export");
    expect(smokeScriptSource).toContain("workspace-parity.opened-single-run-tools");
    expect(smokeScriptSource).toContain("assertOpenedRunNotebookLmExportContract");
    expect(smokeScriptSource).toContain("assertRunRowAffordance");
    expect(smokeScriptSource).toContain("openEvidenceForRun");
    expect(smokeScriptSource).toContain("assertShowInSourceDisabledReason");
    expect(smokeScriptSource).toContain("fixtureLabelsFromSeededUi");
    expect(smokeScriptSource).toContain("new Set([...sourceLabels, ...runLabels])");
    expect(smokeScriptSource).toContain("clearRunsFilters");
    expect(smokeScriptSource).toContain('setRunsSegment(ctx, "Runs scope", "All runs")');
    expect(smokeScriptSource).toContain('setRunsSegment(ctx, "Runs status", "All")');
    expect(smokeScriptSource).toContain('fillByLabel(ctx.socket, "Search runs", "__analysis_redesign_fixture__")');
    expect(smokeScriptSource).toContain("Legacy snapshot missing");
    expect(smokeScriptSource).toContain("Snapshot capture failed: fixture write boundary unavailable");
    expect(smokeScriptSource).toContain("This capture-failed fixture report remains readable.");
    expect(smokeScriptSource).toContain("...sourceBrowserSmokeSteps, ...savedRunAffordanceSmokeSteps, ...analysisWorkspaceParitySteps");
    expect(smokeScriptSource).toContain('clickRowActionByText(ctx.socket, "run-companion-runs-panel"');
    expect(smokeScriptSource).toContain("assertEmptyFixtureSummary(verificationSummary)");
    expect(smokeScriptSource).toContain("expected.filter((label) => text.includes(label))");
    expect(smokeScriptSource).toContain("waitForCurrentContext");
    expect(smokeScriptSource).toContain("waitForOpenedRunSurface");
    expect(smokeScriptSource).toContain("analysis-current-context");
    expect(smokeScriptSource).toContain("runSmokeSteps");
    expect(smokeScriptSource).toContain("finally");
    expect(smokeScriptSource).toContain("fixturesTouched");
    expect(smokeScriptSource).toContain("if (fixturesTouched && ctx?.socket)");
    expect(smokeScriptSource).toContain("cleanupFixtures");
    expect(smokeScriptSource).toContain("refreshBridgeConnection");
    expect(smokeScriptSource).toContain("retryProbeCommand(ctx");
    expect(smokeScriptSource).toContain("classifyBridgeFailure(error)");
    expect(smokeScriptSource).toContain("smokeRuntimeRoot");
    expect(smokeScriptSource).toContain("analysisWorkspaceStateKey");
    expect(smokeScriptSource).toContain("localStorage.removeItem(analysisWorkspaceStateKey)");
    expect(smokeScriptSource).toContain("restoreSmokeLocalStorage");
    expect(smokeScriptSource).toContain("APPDATA:");
    expect(smokeScriptSource).toContain("LOCALAPPDATA:");
    expect(smokeScriptSource).toContain("XDG_CONFIG_HOME:");
    expect(helperSource).toContain("env = process.env");
    expect(smokeScriptSource).toContain("ctx.socket = bridge.socket");
    expect(smokeScriptSource.indexOf("PASS bridge.execute_js"))
      .toBeLessThan(smokeScriptSource.indexOf("PASS bridge.resize_window"));
  });

  it("keeps bridge, helper, assertion, and artifact behavior centralized", () => {
    expect(helperSource).toContain("bridgeRequest");
    expect(helperSource).toContain("executeJs");
    expect(helperSource).toContain("waitForText");
    expect(helperSource).toContain("clickByText");
    expect(helperSource).toContain("clickByTextWithinSmokeId");
    expect(helperSource).toContain("clickRowActionByText");
    expect(helperSource).toContain("clickBySmokeId");
    expect(helperSource).toContain("getVisibleTextSummary");
    expect(helperSource).toContain("assertTabOrderLabels");
    expect(helperSource).toContain("fixtureSummaryKeys");
    expect(helperSource).toContain("assertEmptyFixtureSummary");
    expect(helperSource).toContain("assertDisabledWithReason");
    expect(helperSource).toContain("captureArtifacts");
    expect(helperSource).toContain("capture_native_screenshot");
    expect(helperSource).toContain("resize_window");
    expect(helperSource).toContain("startupTimeoutMs = 90000");
    expect(helperSource).toContain("app-identifier-mismatch");
  });

  it("associates source-group NotebookLM disabled reason through aria-describedby", () => {
    expect(reportWorkspaceToolsSource).toContain("const exportReasonId = \"notebooklm-export-disabled-reason\"");
    expect(reportWorkspaceToolsSource).toContain('smokeId="notebooklm-export-button"');
    expect(reportWorkspaceToolsSource).toContain("ariaDescribedby={!compact && exportDisabledReason ? exportReasonId : undefined}");
    expect(reportWorkspaceToolsSource).toContain("{#if !compact && exportDisabledReason}");
    expect(reportWorkspaceToolsSource).toContain("id={exportReasonId}");
    expect(reportWorkspaceToolsSource).toContain('data-smoke-id="notebooklm-export-disabled-reason"');
    expect(reportCanvasSource).toContain("YouTube source-group NotebookLM export is not implemented yet.");
  });

  it("renders smoke hooks only for stable analysis UI contracts", () => {
    expect(reportWorkspaceToolsSource).toContain('data-smoke-id="analysis-workspace-tools"');
    expect(reportCanvasSource).toContain('smokeId="report-canvas-mode-report"');
    expect(reportCanvasSource).toContain('smokeId="report-canvas-mode-source"');
    expect(reportSetupPanelSource).toContain('data-smoke-id="analysis-report-setup"');
    expect(reportSourceSurfaceSource).toContain('data-smoke-id="analysis-source-surface"');
    expect(sourceBrowserShellSource).toContain('data-smoke-id="source-browser-tabs"');
    expect(sourceReaderHeaderSource).toMatch(/smokeId\s*=\s*"source-browser-header"/);
    expect(sourceReaderHeaderSource).toContain("data-smoke-id={smokeId}");
    expect(reportSourceSurfaceSource).toContain('smokeId="run-snapshot-header"');
    expect(reportSourceSurfaceSource).toContain('smokeId="source-browser-header"');
    expect(reportCanvasSource).toContain('data-smoke-id="template-editor-drawer"');
    expect(reportCanvasSource).toContain('data-smoke-id="source-group-editor-drawer"');
    expect(desktopDialogSource).toContain("smokeId");
    expect(desktopDialogSource).toContain("data-smoke-id={smokeId}");
    expect(notebookLmExportDialogSource).toContain('smokeId="notebooklm-export-dialog"');
    expect(compactSourceRailSource).toContain('smokeId="analysis-source-switcher-trigger"');
    expect(compactSourceRailSource).toContain('data-smoke-id="analysis-current-context"');
    expect(sourceSwitcherPanelSource).toContain('data-smoke-id="source-switcher-panel"');
    expect(sourceSwitcherPanelSource).toContain('data-smoke-id="source-switcher-search"');
    expect(runCompanionTabsSource).toContain('smokeId="run-companion-runs-tab"');
    expect(runCompanionRunsTabSource).toContain('data-smoke-id="run-companion-runs-panel"');
    expect(runCompanionRunsTabSource).toContain('data-smoke-id="runs-search"');
  });

  it("keeps run snapshot source-browser tabs exact and activity-free", () => {
    const labels = sourceBrowserTabsForSubject({
      kind: "run_snapshot",
      snapshot: {
        runId: 1,
        scopeType: "source_group",
        scopeLabel: "Fixture snapshot",
        readerKind: "source_group",
        sourceType: "telegram",
        sourceSubtype: "supergroup",
      },
    }).map((tab) => tab.label);

    expect(labels).toEqual(["Sources", "Items", "Metadata"]);
    expect(labels).not.toContain("Activity");
  });

  it("keeps smoke selectors out of non-analysis source files", () => {
    const srcDir = path.join(repoRoot, "src");
    const offenders = collectSourceFiles(srcDir)
      .filter((file) => !file.endsWith(".test.ts"))
      .filter((file) => existsSync(file))
      .filter((file) => {
        const normalized = file.replaceAll("\\", "/");
        return !normalized.includes("/src/lib/components/analysis/")
          && !normalized.includes("/src/routes/analysis/")
          && !sharedSmokePrimitiveFiles.has(path.relative(repoRoot, file).replaceAll("\\", "/"));
      })
      .filter((file) => readdirSync(path.dirname(file)).includes(path.basename(file)))
      .filter((file) => {
        const content = readFileSync(file, "utf8");
        return content.includes("data-smoke-id") || content.includes("smokeId=");
      });

    expect(offenders.map((file) => path.relative(repoRoot, file))).toEqual([]);
    expect(desktopDialogSource).not.toContain('data-smoke-id="');
    expect(reportWorkspaceToolsSource).toContain('data-smoke-id="analysis-workspace-tools"');
  });
});
