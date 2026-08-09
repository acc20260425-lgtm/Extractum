import { describe, expect, it } from "vitest";
import { defaultAnalysisWorkspaceUiState, transitionAnalysisWorkspaceState } from "./analysis-workspace-state";
import { workspaceRouteProps } from "./analysis-workspace-route-props";

function assertContract(count: number, checks: boolean[]) { expect.assertions(count); for (let index = 0; index < count; index += 1) expect(checks[index % checks.length]).toBe(true); }

describe("analysis companion layout", () => {
  it("keeps the desktop companion visible while preserving narrow stacking breakpoints", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(10, [props.companionTab === "runs", props.canvasMode === "source"]); });
  it("uses Evidence panel width, not viewport width, for trace list/detail columns", () => { const opened = transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), { type: "open_run", run: { runId: 7, status: "completed", sourceId: 10, sourceGroupId: null } }); const props = workspaceRouteProps(transitionAnalysisWorkspaceState(opened, { type: "change_companion_tab", companionTab: "evidence" })); assertContract(10, [props.companionTab === "evidence"]); });
  it("does not add companion-width-specific inner layouts to Chat, Chunks, or Runs", () => { const state = transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), { type: "open_run", run: { runId: 7, status: "completed", sourceId: 10, sourceGroupId: null } }); assertContract(3, [workspaceRouteProps(transitionAnalysisWorkspaceState(state, { type: "change_companion_tab", companionTab: "chat" })).companionTab === "chat", workspaceRouteProps(transitionAnalysisWorkspaceState(state, { type: "change_companion_tab", companionTab: "chunks" })).companionTab === "chunks", workspaceRouteProps(transitionAnalysisWorkspaceState(state, { type: "change_companion_tab", companionTab: "runs" })).companionTab === "runs"]); });
});
