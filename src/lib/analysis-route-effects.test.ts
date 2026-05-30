import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis route effects", () => {
  function historyScopeEffect() {
    const paramsStart = analysisPageSource.indexOf("const params = historyScopeParams;");
    const effectStart = analysisPageSource.lastIndexOf("  $effect(() => {", paramsStart);
    const nextEffectStart = analysisPageSource.indexOf("\n  $effect(() => {", effectStart + 1);

    expect(paramsStart, "analysis route should read explicit history-scope params").toBeGreaterThan(-1);
    expect(effectStart, "analysis route should define a history-scope effect").toBeGreaterThan(-1);
    expect(
      nextEffectStart,
      "history-scope effect should be followed by another effect",
    ).toBeGreaterThan(effectStart);

    return analysisPageSource.slice(effectStart, nextEffectStart);
  }

  function syncSelectedSourceFunction() {
    const functionStart = analysisPageSource.indexOf("  async function syncSelectedSource");
    const nextFunctionStart = analysisPageSource.indexOf(
      "\n  async function startYoutubeJob",
      functionStart + 1,
    );

    expect(
      functionStart,
      "analysis route should define a selected-source sync function",
    ).toBeGreaterThan(-1);
    expect(
      nextFunctionStart,
      "selected-source sync function should be followed by startYoutubeJob",
    ).toBeGreaterThan(functionStart);

    return analysisPageSource.slice(functionStart, nextFunctionStart);
  }

  function runSnapshotEffect() {
    const callStart = analysisPageSource.lastIndexOf("void loadRunSnapshotFirstPage(currentRun.id);");
    const effectStart = analysisPageSource.lastIndexOf("  $effect(() => {", callStart);
    const nextEffectStart = analysisPageSource.indexOf("\n  $effect(() => {", effectStart + 1);

    expect(callStart, "analysis route should load run snapshot data for opened runs").toBeGreaterThan(-1);
    expect(effectStart, "snapshot loading should be owned by an effect").toBeGreaterThan(-1);
    expect(nextEffectStart, "snapshot effect should be followed by another effect").toBeGreaterThan(effectStart);

    return analysisPageSource.slice(effectStart, nextEffectStart);
  }

  function youtubeDetailFunction() {
    const functionStart = analysisPageSource.indexOf("  async function loadYoutubeDetail");
    const nextFunctionStart = analysisPageSource.indexOf(
      "\n  async function loadTemplates",
      functionStart + 1,
    );

    expect(
      functionStart,
      "analysis route should define a YouTube detail loader",
    ).toBeGreaterThan(-1);
    expect(
      nextFunctionStart,
      "YouTube detail loader should be followed by loadTemplates",
    ).toBeGreaterThan(functionStart);

    return analysisPageSource.slice(functionStart, nextFunctionStart);
  }

  it("schedules saved run history loading from explicit scope params and runs filters", () => {
    const effect = historyScopeEffect();

    expect(effect, "history-scope effect should read only the explicit scope params").toContain(
      "const params = historyScopeParams;",
    );
    expect(effect, "history-scope effect should read the canonical companion runs filter").toContain(
      "const filter = runsFilter;",
    );
    expect(effect, "history-scope effect should schedule the explicit-scope loader").toContain(
      "scheduleSavedRunsLoad(params, filter);",
    );
    expect(effect, "history-scope effect must not call the broad wrapper directly").not.toContain(
      "loadRuns();",
    );
    expect(effect, "history-scope effect should not need untrack after explicit params").not.toContain(
      "untrack(",
    );
  });

  it("debounces saved run reloads and clears pending timers on teardown", () => {
    expect(analysisPageSource).toContain("let savedRunsLoadTimer: ReturnType<typeof setTimeout> | null = null;");
    expect(analysisPageSource).toContain("const savedRunsLoadDelayMs = 250;");
    expect(analysisPageSource).toContain("function scheduleSavedRunsLoad(");
    expect(analysisPageSource).toContain("clearSavedRunsLoadTimer();");
    expect(analysisPageSource).toContain("void runWorkflow.loadRunsForScope(params, filter);");
  });

  it("includes YouTube comments when syncing a video source from the main sync action", () => {
    const syncFunction = syncSelectedSourceFunction();

    expect(syncFunction).toContain("if (source.sourceType === \"youtube\")");
    expect(syncFunction).toContain("transcripts: source.sourceSubtype === \"video\"");
    expect(syncFunction).toContain("comments: source.sourceSubtype === \"video\"");
  });

  it("probes opened-run snapshot availability before the user switches to Source mode", () => {
    const effect = runSnapshotEffect();

    expect(effect).toContain("currentRun");
    expect(effect).toContain("void loadRunSnapshotFirstPage(currentRun.id);");
    expect(effect).not.toContain('workspaceUiState.canvasMode === "source"');
    expect(effect).not.toContain('workspaceUiState.sourceViewBasis === "run_snapshot"');
  });

  it("ignores stale YouTube detail responses after the selected source changes", () => {
    const detailFunction = youtubeDetailFunction();

    expect(analysisPageSource).toContain("let youtubeDetailRequestKey = $state(\"\");");
    expect(detailFunction).toContain("const requestKey = `${source.id}:${source.sourceSubtype}`;");
    expect(detailFunction).toContain("youtubeDetailRequestKey = requestKey;");
    expect(detailFunction).toContain("if (youtubeDetailRequestKey !== requestKey) {");
    expect(detailFunction).toContain("status = formatAppError(\"loading YouTube detail\", error);");
    expect(detailFunction).toContain("loadingYoutubeDetail = false;");
  });
});
