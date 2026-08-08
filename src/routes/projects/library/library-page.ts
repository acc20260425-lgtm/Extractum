import {
  createLibraryCatalogWorkflow,
  type LibraryCatalogWorkflowState,
} from "$lib/ui/library-catalog-workflow";

export const LIBRARY_PAGE_ROUTE = Object.freeze({ id: "library-prototype", href: "/projects/library" });

export function createLibraryPageState(): LibraryCatalogWorkflowState {
  return { catalogRecords: [], filterCounts: [], sources: [], loading: false, status: "" };
}

export function createLibraryPageWorkflow({
  state,
  listCatalog,
  formatError,
}: {
  state: LibraryCatalogWorkflowState;
  listCatalog: Parameters<typeof createLibraryCatalogWorkflow>[0]["listCatalog"];
  formatError: Parameters<typeof createLibraryCatalogWorkflow>[0]["formatError"];
}) {
  return createLibraryCatalogWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listCatalog,
    formatError,
  });
}
