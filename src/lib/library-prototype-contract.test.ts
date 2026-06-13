import { describe, expect, it } from "vitest";
import routeSource from "../routes/projects/library/+page.svelte?raw";
import screenSource from "./components/research-projects/LibraryScreen.svelte?raw";
import filterRailSource from "./components/research-projects/LibraryFilterRail.svelte?raw";
import workspaceSource from "./components/research-projects/LibraryWorkspace.svelte?raw";
import inspectorSource from "./components/research-projects/LibraryInspector.svelte?raw";

describe("Library prototype contract", () => {
  it("renders Library as a separate route backed by the current workflow", () => {
    expect(routeSource).toContain('data-ui-route="library-prototype"');
    expect(routeSource).toContain("createLibraryCatalogWorkflow");
    expect(routeSource).toContain("listLibrarySources");
    expect(routeSource).toContain("<LibraryScreen");
  });

  it("uses the TreeDataGrid wrapper for the collapsible filter rail", () => {
    expect(filterRailSource).toContain("ExtractumTreeDataGrid");
    expect(filterRailSource).toContain('data-ui-region="library-filter-rail"');
    expect(filterRailSource).toContain("collapsed");
    expect(filterRailSource).toContain("onSelectedFilterIdChange");
    expect(filterRailSource).not.toContain("@svar-ui/");
  });

  it("renders source CRUD commands and disables selected-source commands without a source", () => {
    expect(workspaceSource).toContain("ExtractumDataGrid");
    expect(workspaceSource).toContain('data-ui-region="library-workspace"');
    expect(workspaceSource).toContain('data-ui-action="library-add"');
    expect(workspaceSource).toContain('data-ui-action="library-edit"');
    expect(workspaceSource).toContain('data-ui-action="library-delete"');
    expect(workspaceSource).toContain('disabled={!selectedSource}');
    expect(workspaceSource).not.toContain("@svar-ui/");
    expect(workspaceSource).not.toContain("$lib/components/ui/");
  });

  it("keeps the Inspector bound to selected source context", () => {
    expect(inspectorSource).toContain('data-ui-region="library-inspector"');
    expect(inspectorSource).toContain("selectedSource");
    expect(inspectorSource).toContain("No source selected");
    expect(inspectorSource).toContain("aria-label=\"Inspector commands\"");
  });

  it("coordinates filter selection, row selection, and Inspector resizing in the screen component", () => {
    expect(screenSource).toContain("buildLibraryCatalogFilterTree");
    expect(screenSource).toContain("filterLibraryCatalogSources");
    expect(screenSource).toContain("reconcileLibraryCatalogSourceSelection");
    expect(screenSource).toContain("inspectorWidth");
    expect(screenSource).toContain("clampInspectorWidth");
    expect(screenSource).toContain('role="separator"');
    expect(screenSource).toContain("onpointerdown");
  });
});
