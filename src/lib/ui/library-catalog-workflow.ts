import type {
  LibraryCatalogFilterCount,
  LibraryCatalogRecord,
  LibraryCatalogResponse,
} from "$lib/types/library-sources";
import {
  buildLibraryCatalogSourcesView,
  type LibraryCatalogSourceView,
} from "./library-catalog-model";

export interface LibraryCatalogWorkflowState {
  catalogRecords: LibraryCatalogRecord[];
  filterCounts: LibraryCatalogFilterCount[];
  sources: LibraryCatalogSourceView[];
  loading: boolean;
  status: string;
}

export interface LibraryCatalogWorkflowDeps {
  getState(): LibraryCatalogWorkflowState;
  patch(patch: Partial<LibraryCatalogWorkflowState>): void;
  listCatalog(): Promise<LibraryCatalogResponse>;
  formatError(action: string, error: unknown): string;
}

export function createLibraryCatalogWorkflow(deps: LibraryCatalogWorkflowDeps) {
  function refreshDerivedState() {
    const state = deps.getState();
    deps.patch({
      sources: buildLibraryCatalogSourcesView(state.catalogRecords),
    });
  }

  async function loadLibrary() {
    deps.patch({ loading: true, status: "" });
    try {
      const catalog = await deps.listCatalog();
      deps.patch({
        catalogRecords: catalog.sources,
        filterCounts: catalog.filter_counts,
      });
      refreshDerivedState();
    } catch (error) {
      deps.patch({ status: deps.formatError("loading library catalog", error) });
    } finally {
      deps.patch({ loading: false });
    }
  }

  return {
    refreshDerivedState,
    loadLibrary,
  };
}
