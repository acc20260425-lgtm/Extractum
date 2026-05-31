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

  function functionSlice(name: string, nextName: string) {
    const start = analysisPageSource.indexOf(`  ${name}`);
    const end = analysisPageSource.indexOf(`\n  ${nextName}`, start + 1);

    expect(start, `${name} should exist`).toBeGreaterThan(-1);
    expect(end, `${nextName} should follow ${name}`).toBeGreaterThan(start);

    return analysisPageSource.slice(start, end);
  }

  function loadSourcePageAroundTraceFunction() {
    return functionSlice(
      "async function loadSourcePageAroundTrace",
      "async function showSelectedTraceInSource",
    );
  }

  function expectOrder(source: string, first: string, second: string, message: string) {
    const firstIndex = source.indexOf(first);
    const secondIndex = source.indexOf(second);

    expect(firstIndex, `${message}: missing first marker`).toBeGreaterThan(-1);
    expect(secondIndex, `${message}: missing second marker`).toBeGreaterThan(-1);
    expect(firstIndex, message).toBeLessThan(secondIndex);
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

  it("keeps evidence source route state and active return context local to the route", () => {
    expect(analysisPageSource).toContain('} from "$lib/analysis-evidence-source-navigation";');
    expect(analysisPageSource).toContain("canonicalEvidenceTraceRef,");
    expect(analysisPageSource).toContain("focusedLiveSourceTargetForTrace,");
    expect(analysisPageSource).toContain("sourceReturnContextIsActive,");
    expect(analysisPageSource).toContain("sourceScopeForEvidence,");
    expect(analysisPageSource).toContain("type EvidenceHighlightToken,");
    expect(analysisPageSource).toContain("type PendingEvidenceSourceFocus,");
    expect(analysisPageSource).toContain("type SourceReturnContext,");
    expect(analysisPageSource).toContain("let sourceReturnContext = $state<SourceReturnContext>(null);");
    expect(analysisPageSource).toContain(
      "let pendingEvidenceSourceFocus = $state<PendingEvidenceSourceFocus | null>(null);",
    );
    expect(analysisPageSource).toContain(
      "let transientSourceHighlight = $state<EvidenceHighlightToken | null>(null);",
    );
    expect(analysisPageSource).toContain("let evidenceSourceFocusSequence = 0;");
    expect(analysisPageSource).toContain(
      "let sourceHighlightClearTimer: ReturnType<typeof setTimeout> | null = null;",
    );
    expect(analysisPageSource).toContain("const activeSourceReturnContext = $derived.by(() => {");
    expect(analysisPageSource).toContain("return sourceReturnContextIsActive(sourceReturnContext, {");
  });

  it("clears evidence source navigation when route context changes", () => {
    const functionPairs = [
      ["function clearCurrentRunForWorkspaceSwitch", "function liveScopeExistsForRun"],
      ["function viewLiveSourceForOpenedRun", "function backToRunSnapshot"],
      ["function backToRunSnapshot", "function returnToEvidenceReview"],
      ["async function focusTraceRef", "function currentEvidenceSourceScope"],
      ["function changeSelectedGroupSourceId", "async function selectSource"],
      ["async function selectSource", "function selectGroup"],
      ["function selectGroup", "async function changeSelectedTopicKey"],
      ["function changeSelectedSnapshotSourceId", "async function loadChatMessages"],
    ];

    for (const [name, nextName] of functionPairs) {
      expect(functionSlice(name, nextName), `${name} should clear navigation state`).toContain(
        "clearEvidenceSourceNavigation();",
      );
    }
  });

  it("checks pending focus before assigning focused snapshot state", () => {
    const focusedLoad = loadSourcePageAroundTraceFunction();
    const snapshotBranch = focusedLoad.slice(
      focusedLoad.indexOf('if (decision.kind === "run_snapshot")'),
      focusedLoad.indexOf("const liveTarget = focusedLiveSourceTargetForTrace(trace);"),
    );

    expect(snapshotBranch).toContain("if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {");
    expectOrder(
      snapshotBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "lastSnapshotLoadKey =",
      "snapshot stale guard must precede load-key assignment",
    );
    expectOrder(
      snapshotBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "selectedSnapshotSourceId =",
      "snapshot stale guard must precede source selection assignment",
    );
    expectOrder(
      snapshotBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "applySnapshotPage(",
      "snapshot stale guard must precede applying the snapshot page",
    );
  });

  it("checks pending focus before assigning focused group-live state", () => {
    const focusedLoad = loadSourcePageAroundTraceFunction();
    const groupBranch = focusedLoad.slice(
      focusedLoad.indexOf('if (analysisScope === "source_group")'),
      focusedLoad.indexOf('if (source.sourceType === "telegram")'),
    );

    expect(groupBranch).toContain("if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {");
    expectOrder(
      groupBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "selectedGroupSourceId =",
      "group-live stale guard must precede source selection assignment",
    );
    expectOrder(
      groupBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "groupLiveItemsBySource =",
      "group-live stale guard must precede item assignment",
    );
    expectOrder(
      groupBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "groupLiveCursorsBySource =",
      "group-live stale guard must precede cursor assignment",
    );
    expectOrder(
      groupBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "groupLiveHasMoreBySource =",
      "group-live stale guard must precede has-more assignment",
    );
  });

  it("checks pending focus before assigning focused single-source and transcript state", () => {
    const focusedLoad = loadSourcePageAroundTraceFunction();
    const singleSourceBranch = focusedLoad.slice(
      focusedLoad.indexOf('if (source.sourceType === "telegram")'),
      focusedLoad.indexOf('if (source.sourceType === "youtube"'),
    );
    const transcriptBranch = focusedLoad.slice(
      focusedLoad.indexOf('if (source.sourceType === "youtube"'),
      focusedLoad.indexOf("} catch (error)"),
    );

    expect(singleSourceBranch).toContain("if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {");
    expectOrder(
      singleSourceBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "applySourceItemsPage(",
      "single-source stale guard must precede applying source items",
    );

    expect(transcriptBranch).toContain("if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {");
    expectOrder(
      transcriptBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "youtubeTranscriptSegments =",
      "transcript stale guard must precede segment assignment",
    );
    expectOrder(
      transcriptBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "youtubeTranscriptCursor =",
      "transcript stale guard must precede cursor assignment",
    );
    expectOrder(
      transcriptBranch,
      "if (!pendingFocusMatchesCurrent(pendingEvidenceSourceFocus, {",
      "youtubeTranscriptHasMore =",
      "transcript stale guard must precede has-more assignment",
    );
  });
});
