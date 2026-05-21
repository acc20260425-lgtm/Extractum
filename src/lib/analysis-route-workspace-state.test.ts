import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

function functionSlice(name: string, nextName: string) {
  const start = analysisPageSource.indexOf(`  ${name}`);
  const end = analysisPageSource.indexOf(`\n  ${nextName}`, start + 1);

  expect(start, `${name} should exist`).toBeGreaterThan(-1);
  expect(end, `${nextName} should follow ${name}`).toBeGreaterThan(start);

  return analysisPageSource.slice(start, end);
}

describe("analysis route workspace state", () => {
  it("imports workspace state and persistence helpers", () => {
    expect(analysisPageSource).toContain("defaultAnalysisWorkspaceUiState");
    expect(analysisPageSource).toContain("transitionAnalysisWorkspaceState");
    expect(analysisPageSource).toContain("loadPersistedAnalysisWorkspaceState");
    expect(analysisPageSource).toContain("savePersistedAnalysisWorkspaceState");
    expect(analysisPageSource).toContain("fallbackWorkspaceSelection");
  });

  it("owns result-first workspace state without rendering the new layout yet", () => {
    expect(analysisPageSource).toContain("let workspaceUiState = $state<AnalysisWorkspaceUiState>(");
    expect(analysisPageSource).toContain("let workspacePersistenceReady = $state(false);");
    expect(analysisPageSource).toContain("let restoredWorkspaceSelection = $state<WorkspaceSelection | null>(null);");
    expect(analysisPageSource).toContain("defaultAnalysisWorkspaceUiState()");
  });

  it("derives legacy scope ids from the workspace selection instead of mirroring mutable state", () => {
    expect(analysisPageSource).toContain("const legacyWorkspaceSelection = $derived(");
    expect(analysisPageSource).toContain("const analysisScope = $derived(legacyWorkspaceSelection.analysisScope);");
    expect(analysisPageSource).toContain("const selectedSourceId = $derived(legacyWorkspaceSelection.selectedSourceId);");
    expect(analysisPageSource).toContain("const selectedGroupId = $derived(legacyWorkspaceSelection.selectedGroupId);");
    expect(analysisPageSource).toContain("let selectedGroupEditorId = $state");
    expect(analysisPageSource).not.toContain("let selectedSourceId = $state");
    expect(analysisPageSource).not.toContain("let selectedGroupId = $state");
    expect(analysisPageSource).not.toContain("let analysisScope = $state");
  });

  it("dispatches workspace events through a single route boundary", () => {
    const dispatcher = functionSlice(
      "function dispatchWorkspaceEvent",
      "function changeCanvasMode",
    );
    const handlers = analysisPageSource.slice(
      analysisPageSource.indexOf("  function changeCanvasMode"),
    );

    expect(analysisPageSource).toContain("type AnalysisWorkspaceEvent");
    expect(dispatcher).toContain("event: AnalysisWorkspaceEvent");
    expect(dispatcher).toContain("transitionAnalysisWorkspaceState(workspaceUiState, event)");
    expect(handlers).toContain("dispatchWorkspaceEvent({");
    expect(handlers).not.toContain("transitionAnalysisWorkspaceState(workspaceUiState");
  });

  it("restores persisted workspace state before loading active runs", () => {
    const mount = analysisPageSource.slice(
      analysisPageSource.indexOf("  onMount(() => {"),
      analysisPageSource.indexOf("</script>"),
    );

    expect(mount).toContain("restorePersistedWorkspaceState();");
    expect(mount).toContain("await Promise.all([loadSourceCatalog(), loadGroups()]);");
    expect(mount).toContain("await applyRestoredWorkspaceSelection();");
    expect(mount.indexOf("restorePersistedWorkspaceState();"))
      .toBeLessThan(mount.indexOf("await Promise.all([loadSourceCatalog(), loadGroups()]);"));
    expect(mount.indexOf("await applyRestoredWorkspaceSelection();"))
      .toBeLessThan(mount.indexOf("void loadActiveRuns();"));
  });

  it("persists durable workspace state and excludes run-bound transient state", () => {
    const saveFunction = functionSlice(
      "function persistWorkspaceState()",
      "function applyWorkspaceUiState",
    );

    expect(saveFunction).toContain("savePersistedAnalysisWorkspaceState(window.localStorage");
    expect(saveFunction).toContain("persistableAnalysisWorkspaceState(workspaceUiState");
    expect(saveFunction).toContain("historyScope");
    expect(saveFunction).toContain("runFilter");
    expect(saveFunction).not.toContain("currentRun");
    expect(saveFunction).not.toContain("activeRunId");
    expect(saveFunction).not.toContain("selectedTraceRef");
    expect(saveFunction).not.toContain("chatQuestion");
    expect(saveFunction).not.toContain("sourceManagerOpen");
  });

  it("uses workspace transition events for source, group, and run opening", () => {
    const sourceFunction = functionSlice(
      "async function selectSource",
      "function selectGroup",
    );
    const groupFunction = functionSlice(
      "function selectGroup",
      "async function changeSelectedTopicKey",
    );
    const runFunction = functionSlice(
      "function alignWorkspaceToOpenedRun",
      "async function loadChatMessages",
    );

    expect(sourceFunction).toContain('type: "select_source"');
    expect(sourceFunction).toContain("clearCurrentRunForWorkspaceSwitch();");
    expect(groupFunction).toContain('type: "select_source_group"');
    expect(groupFunction).toContain("clearCurrentRunForWorkspaceSwitch();");
    expect(runFunction).toContain('type: "open_run"');
    expect(runFunction).not.toContain("legacyScopeFromWorkspaceSelection");
  });

  it("saves workspace state from a guarded effect after restore is complete", () => {
    expect(analysisPageSource).toContain("if (!workspacePersistenceReady) {");
    expect(analysisPageSource).toContain("persistWorkspaceState();");
  });
});
