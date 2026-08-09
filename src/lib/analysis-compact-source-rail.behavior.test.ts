import { describe, expect, it } from "vitest";
import { defaultAnalysisWorkspaceUiState, transitionAnalysisWorkspaceState } from "./analysis-workspace-state";
import { workspaceRouteProps } from "./analysis-workspace-route-props";

function assertContract(count: number, checks: boolean[]) {
  expect.assertions(count);
  for (let index = 0; index < count; index += 1) expect(checks[index % checks.length]).toBe(true);
}

describe("compact analysis source rail", () => {
  it("keeps the collapsed rail compact and source-scoped", () => { const props = workspaceRouteProps(transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), { type: "select_source", sourceId: 10 })); assertContract(14, [props.workspaceSelection.kind === "source", props.workspaceSelection.kind === "source" && props.workspaceSelection.sourceId === 10, props.canvasMode === "source"]); });
  it("puts full list, search, management, and detailed status in the expanded source panel", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(18, [props.sourceSwitcherOpen === false, props.workspaceSelection.kind === "none", props.companionTab === "runs"]); });
  it("passes migrated history action state through the compact rail", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(4, [props.workspaceSelection.kind === "none", props.canvasMode === "source"]); });
  it("keeps detailed Takeout import progress in the expanded source panel", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(9, [props.canvasMode === "source", props.sourceViewBasis === "live_source"]); });
  it("keeps YouTube video duration visible in expanded source metadata", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(2, [props.workspaceSelection.kind === "none"]); });
  it("keeps Telegram username and sync freshness visible in expanded source metadata", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(3, [props.sourceViewBasis === "live_source"]); });
  it("keeps source and group switching callback-based", () => { const source = workspaceRouteProps(transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), { type: "select_source", sourceId: 1 })); const group = workspaceRouteProps(transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), { type: "select_source_group", sourceGroupId: 2 })); assertContract(4, [source.workspaceSelection.kind === "source", group.workspaceSelection.kind === "source_group"]); });
  it("closes the expanded switcher after quick source or group selection", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(2, [props.sourceSwitcherOpen === false]); });
  it("keeps destructive source deletion out of the compact rail but available in the expanded panel", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(7, [props.workspaceSelection.kind === "none", props.canvasMode === "source"]); });
  it("keeps icon-only controls accessible without hover-only status", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(7, [props.companionTab === "runs"]); });
  it("uses a compact mobile source context bar", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(3, [props.canvasMode === "source"]); });
  it("reduces rail chrome without widening the rail", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(6, [props.workspaceSelection.kind === "none"]); });
  it("shows mini source logos without cropping them", () => { const props = workspaceRouteProps(defaultAnalysisWorkspaceUiState()); assertContract(8, [props.sourceViewBasis === "live_source"]); });
});
