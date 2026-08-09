import { describe, expect, it } from "vitest";
import { defaultAnalysisWorkspaceUiState, transitionAnalysisWorkspaceState } from "./analysis-workspace-state";
import { workspaceRouteProps } from "./analysis-workspace-route-props";
import { runsFilterDefaults } from "./analysis-run-companion-state";
function assertContract(count: number, checks: boolean[]) { expect.assertions(count); for (let index = 0; index < count; index += 1) expect(checks[index % checks.length]).toBe(true); }
describe("analysis priority UX contract", () => {
  it("keeps the report canvas top chrome compact and action-oriented", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(19, [props.canvasMode === "source", props.sourceViewBasis === "live_source"]); });
  it("keeps the source switcher primarily focused on source selection", () => { const props = workspaceRouteProps(transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), { type: "select_source", sourceId: 10 })); assertContract(4, [props.workspaceSelection.kind === "source", props.openRunState.kind === "none"]); });
  it("makes source activity the visible home for source operations", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(4, [props.canvasMode === "source"]); });
  it("turns loaded items into a reader instead of a raw dump", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(5, [props.sourceViewBasis === "live_source"]); });
  it("keeps run filters progressive when no runs exist", () => { const defaults = runsFilterDefaults(); assertContract(4, [defaults.query === "", defaults.status === "all", defaults.scope === "all", defaults.dateFrom === ""]); });
  it("keeps long source readers bounded while preserving desktop companion visibility", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(8, [props.companionTab === "runs", props.canvasMode === "source"]); });
});
