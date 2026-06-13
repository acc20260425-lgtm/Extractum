import type { LibrarySourceRecord } from "$lib/types/library-sources";
import type { SourceJobRecord } from "$lib/types/sources";
import {
  buildLibraryCatalogSourcesView,
  type LibraryCatalogSourceView,
} from "./library-catalog-model";

export interface LibraryCatalogWorkflowState {
  sourceRecords: LibrarySourceRecord[];
  sourceJobs: SourceJobRecord[];
  sources: LibraryCatalogSourceView[];
  loading: boolean;
  status: string;
}

export interface LibraryCatalogWorkflowDeps {
  getState(): LibraryCatalogWorkflowState;
  patch(patch: Partial<LibraryCatalogWorkflowState>): void;
  listSources(): Promise<LibrarySourceRecord[]>;
  listSourceJobs(): Promise<SourceJobRecord[]>;
  formatError(action: string, error: unknown): string;
}

export function createLibraryCatalogWorkflow(deps: LibraryCatalogWorkflowDeps) {
  function refreshDerivedState() {
    const state = deps.getState();
    deps.patch({
      sources: buildLibraryCatalogSourcesView(state.sourceRecords, state.sourceJobs),
    });
  }

  async function loadLibrary() {
    deps.patch({ loading: true, status: "" });
    try {
      const [sourceRecords, sourceJobs] = await Promise.all([
        deps.listSources(),
        deps.listSourceJobs(),
      ]);
      deps.patch({ sourceRecords, sourceJobs });
      refreshDerivedState();
    } catch (error) {
      deps.patch({ status: deps.formatError("loading library sources", error) });
    } finally {
      deps.patch({ loading: false });
    }
  }

  return {
    refreshDerivedState,
    loadLibrary,
  };
}
