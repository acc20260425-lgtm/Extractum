// @ts-nocheck
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

describe("youtube summary launch contract", () => {
  it("wires launch dialog through the library inspector", () => {
    const inspector = readFileSync("src/lib/components/research-projects/LibraryInspector.svelte", "utf8");
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");
    const source = `${inspector}\n${dialog}`;

    expect(source).toContain("YoutubeSummaryRunDialog");
    expect(source).toContain("preflightYoutubeSummaryRun");
    expect(source).toContain("startYoutubeSummaryRun");
  });

  it("wires project source launches with the selected project id", () => {
    const inspector = readFileSync("src/lib/components/research-projects/ProjectInspector.svelte", "utf8");
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");

    expect(inspector).toContain("YoutubeSummaryRunDialog");
    expect(inspector).toContain("projectId={project?.projectId ?? null}");
    expect(inspector).toContain("selectedSource.sourceNumericId");
    expect(dialog).toContain("projectId = null");
    expect(dialog).toContain("projectId,");
    expect(dialog).not.toContain("projectId: null");
  });

  it("wires the selected youtube summary mode into preflight and start requests", () => {
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");

    expect(dialog).toContain("let controlPreset = $state(\"detailed_report\")");
    expect(dialog).toContain("controlPreset = \"detailed_report\"");
    expect(dialog).toContain("Summary mode");
    expect(dialog).toContain('<option value="gem_analysis">Gem analysis</option>');
    expect(dialog).toContain("detailed_report");
    expect(dialog).toContain("controlPreset,");
    expect(dialog).not.toContain("controlPreset: \"standard\"");
  });

  it("forces comments into gem analysis launch requests", () => {
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");

    expect(dialog).toContain("effectiveIncludeComments");
    expect(dialog).toContain("controlPreset === \"gem_analysis\" ? true : includeComments");
    expect(dialog).toContain("function handleSummaryModeChange");
    expect(dialog).toContain("includeComments = true");
    expect(dialog).toContain("includeComments: effectiveIncludeComments");
    expect(dialog).toContain("onchange={handleSummaryModeChange}");
  });

  it("wires Gemini Browser runtime selector into preflight and start requests", () => {
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");

    expect(dialog).toContain("runtimeProvider = $state");
    expect(dialog).toContain("Gemini Browser");
    expect(dialog).toContain("geminiBridgeStatus");
    expect(dialog).toContain("deriveGeminiBrowserSetupChecks");
    expect(dialog).toContain("runtimeProvider,");
    expect(dialog).toContain("browserProviderConfig:");
  });

  it("restores runtime preferences before preflight and names the submitted provider", () => {
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");

    expect(dialog).toContain("loadYoutubeSummaryRuntimePreferences");
    expect(dialog).toContain("saveYoutubeSummaryRuntimeProvider");
    expect(dialog).toContain("saveYoutubeSummaryBrowserProviderMode");
    expect(dialog).toContain('runtimeProvider === "gemini_browser" ? "Run via Gemini Browser" : "Run via API"');
    expect(dialog).not.toContain('runtimeProvider = "api";');
    expect(dialog).toContain("function handleBrowserModeChange");
    expect(dialog).toContain("onchange={handleBrowserModeChange}");

    const openEffect = dialog.slice(dialog.indexOf("$effect(() =>"), dialog.indexOf("async function loadProfiles"));
    expect(openEffect.indexOf("loadYoutubeSummaryRuntimePreferences"))
      .toBeLessThan(openEffect.indexOf("runPreflight"));
    expect(openEffect).toContain('if (preferences.runtimeProvider === "gemini_browser")');
    expect(openEffect).toContain("queueMicrotask(() => void refreshBrowserStatus())");
    expect(openEffect).not.toContain('if (runtimeProvider === "gemini_browser") void refreshBrowserStatus();');

    const storageGuard = dialog.slice(
      dialog.indexOf("function runtimePreferenceStorage"),
      dialog.indexOf("async function loadProfiles"),
    );
    expect(storageGuard).toContain('typeof window === "undefined"');
    expect(storageGuard).toContain("try {");
    expect(storageGuard).toContain("return window.localStorage;");
    expect(storageGuard).toContain("catch {");
    expect(storageGuard).toContain("return null;");
  });

  it("surfaces Gemini Browser runtime provenance in prompt pack run diagnostics", () => {
    const types = readFileSync("src/lib/types/prompt-packs.ts", "utf8");
    const runsPanel = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte", "utf8");
    const reportPanel = readFileSync("src/lib/components/research-projects/ProjectRunReportPanel.svelte", "utf8");

    expect(types).toContain("browserRunId");
    expect(types).toContain("browserRunStatus");
    expect(types).toContain("browserCompletionReason");
    expect(types).toContain("browserProviderMode");
    expect(runsPanel).toContain("runtimeLabel(run.runtimeProvider)");
    expect(runsPanel).toContain("Gemini Browser");
    expect(reportPanel).toContain("Browser run");
    expect(reportPanel).toContain("stage.browserRunId");
    expect(reportPanel).toContain("stage.browserCompletionReason");
  });

  it("renders video summary text through the safe markdown renderer only in video sections", () => {
    const compactView = readFileSync("src/lib/components/research-projects/YoutubeSummaryResultView.svelte", "utf8");
    const reportPanel = readFileSync("src/lib/components/research-projects/ProjectRunReportPanel.svelte", "utf8");

    expect(compactView).toContain("SafeMarkdown");
    expect(compactView).toContain("<SafeMarkdown source={textAt(video, \"summary_text\", \"No summary text.\")} />");
    expect(compactView).not.toContain("<p>{textAt(video, \"summary_text\", \"No summary text.\")}</p>");

    expect(reportPanel).toContain("SafeMarkdown");
    expect(reportPanel).toContain("<SafeMarkdown source={textAt(video, \"summary_text\", \"No summary text.\")} />");
    expect(reportPanel).not.toContain("<p>{textAt(video, \"summary_text\", \"No summary text.\")}</p>");
  });
});
