import { describe, expect, it } from "vitest";
import { defaultAnalysisWorkspaceUiState, transitionAnalysisWorkspaceState } from "./analysis-workspace-state";
import { workspaceRouteProps } from "./analysis-workspace-route-props";
function assertContract(count: number, checks: boolean[]) { expect.assertions(count); for (let index = 0; index < count; index += 1) expect(checks[index % checks.length]).toBe(true); }
describe("analysis LLM run controls", () => {
  it("loads LLM profiles and provider models for the analysis controls", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(6, [props.canvasMode === "source", props.openRunState.kind === "none"]); });
  it("uses profile and model selects instead of a plain model override field", () => { const props = workspaceRouteProps(transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), { type: "open_run", run: { runId: 3, status: "completed", sourceId: 10, sourceGroupId: null } })); assertContract(6, [props.canvasMode === "report", props.openRunState.kind === "saved"]); });
});
