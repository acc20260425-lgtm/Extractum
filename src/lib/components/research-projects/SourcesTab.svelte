<script lang="ts">
  import { Library, RefreshCw, Download, Trash2, X } from "@lucide/svelte";
  import {
    ExtractumButton,
    ExtractumDataGrid,
  } from "$lib/components/extractum-ui";
  import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui";
  import type {
    LibrarySourceView,
    ProjectSourceLinkView,
    ResearchProjectView,
  } from "$lib/ui/research-projects-model";
  import { selectedProjectSourcesSyncDisabledReason } from "$lib/ui/research-projects-model";
  import LibrarySourceCell from "./LibrarySourceCell.svelte";
  import ProjectSourceSummary from "./ProjectSourceSummary.svelte";

  type ProjectSourceGridRow = ProjectSourceLinkView & {
    id: string;
    subtitle: string | null;
    localCopyLabel: string;
    connectable: boolean;
  };

  let {
    project,
    projectSourceLinks,
    librarySources,
    selectedSourceIds,
    saving = false,
    onSelectedSourceIdsChange,
    onOpenConnectLibrary,
    onRemoveSource,
    onSyncSelectedSources,
  }: {
    project: ResearchProjectView | null;
    projectSourceLinks: ProjectSourceLinkView[];
    librarySources: LibrarySourceView[];
    selectedSourceIds: string[];
    saving?: boolean;
    onSelectedSourceIdsChange: (sourceIds: string[]) => void;
    onOpenConnectLibrary: () => void;
    onRemoveSource: (sourceId: number | number[]) => void | Promise<void>;
    onSyncSelectedSources: (sourceIds: number[]) => void | Promise<void>;
  } = $props();

  const columns: ExtractumDataGridColumn[] = [
    { id: "title", header: "Title", flexgrow: 1, cell: LibrarySourceCell },
    { id: "typeLabel", header: "Type", width: 150 },
    { id: "localCopyLabel", header: "Details", width: 140 },
    { id: "addedAt", header: "Added to project at", width: 180, dateTimeFormat: "datetime" },
  ];

  let libraryById = $derived(new Map(librarySources.map((source) => [source.id, source])));
  let rows = $derived<ProjectSourceGridRow[]>(
    projectSourceLinks
      .filter((link) => !project || link.projectId === project.id)
      .map((link) => {
        const librarySource = libraryById.get(link.sourceId);
        return {
          ...link,
          id: link.sourceId,
          provider: librarySource?.provider ?? link.provider,
          subtitle: librarySource?.subtitle ?? link.filterSummary,
          localCopyLabel: librarySource?.localCopyLabel ?? link.localCopyLabel,
          connectable: true,
        };
      }),
  );

  let selectedRows = $derived(
    rows.filter((row) => selectedSourceIds.includes(row.id))
  );
  let selectedRowIdsInProject = $derived(selectedRows.map((row) => row.id));
  let syncDisabledReason = $derived(selectedProjectSourcesSyncDisabledReason(selectedRows));

  async function handleSyncSelected() {
    if (syncDisabledReason) return;
    await onSyncSelectedSources(selectedRows.map((row) => row.sourceNumericId));
  }

  async function handleRemoveSelected() {
    if (selectedRows.length === 0) return;
    const count = selectedRows.length;
    const message = count === 1
      ? `Are you sure you want to remove "${selectedRows[0].title}" from this project?`
      : `Are you sure you want to remove ${count} selected sources from this project?`;

    if (confirm(message)) {
      const sourceNumericIds = selectedRows.map((row) => row.sourceNumericId);
      await onRemoveSource(sourceNumericIds);
      onSelectedSourceIdsChange([]);
    }
  }
</script>

<section class="sources-tab">
  <header class="sources-toolbar">
    {#if selectedRows.length > 0}
      <div class="contextual-action-bar">
        <div class="selection-info">
          <span class="selection-count">{selectedRows.length}</span>
          <span>selected {selectedRows.length === 1 ? 'source' : 'sources'}</span>
        </div>
        <div class="action-buttons">
          <ExtractumButton
            variant="secondary"
            disabled={saving || syncDisabledReason !== null}
            title={syncDisabledReason ?? "Sync selected sources"}
            onclick={handleSyncSelected}
          >
            <RefreshCw size={12} aria-hidden="true" />
            {saving ? "Syncing..." : "Sync"}
          </ExtractumButton>
          <ExtractumButton variant="secondary" disabled={true} title="Export selected sources (not implemented)">
            <Download size={12} aria-hidden="true" />
            Export
          </ExtractumButton>
          <ExtractumButton variant="destructive" onclick={handleRemoveSelected}>
            <Trash2 size={12} aria-hidden="true" />
            Remove
          </ExtractumButton>
          <button class="clear-selection-btn" onclick={() => onSelectedSourceIdsChange([])} aria-label="Clear selection">
            <X size={14} aria-hidden="true" />
          </button>
        </div>
      </div>
    {:else}
      <div class="global-action-bar">
        <ExtractumButton variant="outline" disabled={true} title="Sync all sources (not implemented)">
          <RefreshCw size={12} aria-hidden="true" />
          Sync all
        </ExtractumButton>
        <ExtractumButton data-ui-action="connect-library" onclick={onOpenConnectLibrary}>
          <Library size={14} aria-hidden="true" />
          Connect from Library
        </ExtractumButton>
      </div>
    {/if}
  </header>

  <ProjectSourceSummary
    {project}
    connectedCount={rows.length}
    materialCount={project?.materialCount ?? 0}
    libraryCount={librarySources.length}
  />

  <div class="sources-grid-region">
    <ExtractumDataGrid
      rows={rows}
      {columns}
      selectedRowIds={selectedRowIdsInProject}
      multiselect={true}
      onSelectedRowIdsChange={onSelectedSourceIdsChange}
      height="100%"
      overlay="No project sources"
    />
  </div>
</section>

<style>
  .sources-tab {
    display: flex;
    min-height: 0;
    min-width: 0;
    max-width: 100%;
    flex: 1;
    flex-direction: column;
    gap: 12px;
    overflow: hidden;
    padding-top: 12px;
  }

  .sources-toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 44px;
  }

  .contextual-action-bar,
  .global-action-bar {
    display: flex;
    align-items: center;
    width: 100%;
  }

  .global-action-bar {
    justify-content: flex-end;
    gap: 8px;
  }

  .contextual-action-bar {
    justify-content: space-between;
    background: color-mix(in srgb, var(--extractum-primary) 8%, var(--extractum-surface-subtle));
    border: 1px solid color-mix(in srgb, var(--extractum-primary) 15%, var(--extractum-border));
    border-radius: var(--extractum-radius);
    padding: 6px 12px;
    animation: slideIn 0.15s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .selection-info {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--extractum-text);
    font-size: 12px;
    font-weight: 500;
  }

  .selection-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    border-radius: 999px;
    background: var(--extractum-primary);
    color: white;
    font-size: 10px;
    font-weight: 700;
  }

  .action-buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  :global(.action-buttons .extractum-button) {
    min-height: 28px !important;
    height: 28px !important;
    padding: 0 10px !important;
    font-size: 11px !important;
    display: inline-flex !important;
    align-items: center !important;
    gap: 6px !important;
  }

  .clear-selection-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: transparent;
    border: none;
    color: var(--extractum-muted);
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
    padding: 0;
  }

  .clear-selection-btn:hover {
    background: color-mix(in srgb, var(--extractum-border) 40%, transparent);
    color: var(--extractum-text);
  }

  .clear-selection-btn :global(svg) {
    width: 14px;
    height: 14px;
    stroke: currentColor;
    display: block;
  }

  .sources-grid-region {
    min-height: 240px;
    min-width: 0;
    max-width: 100%;
    flex: 1;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    overflow: hidden;
  }
</style>
