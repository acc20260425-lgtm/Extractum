<script lang="ts">
  import { Check, Search, X } from "@lucide/svelte";
  import {
    ExtractumButton,
    ExtractumDataGrid,
    ExtractumSheet,
    ExtractumTextInput,
    GridSelectCell,
    ProviderBadge,
    StatusBadge,
  } from "$lib/components/extractum-ui";
  import {
    connectableSelection,
    filterLibrarySources,
    type LibrarySourceView,
    type ResearchProjectView,
  } from "$lib/ui/research-projects-model";
  import type { LibrarySourceProvider } from "$lib/types/library-sources";
  import LibrarySourceCell from "./LibrarySourceCell.svelte";

  let {
    open,
    project,
    librarySources,
    selectedSourceIds,
    saving,
    status,
    onOpenChange,
    onSelectedSourceIdsChange,
    onConnectSelectedSources,
  }: {
    open: boolean;
    project: ResearchProjectView | null;
    librarySources: LibrarySourceView[];
    selectedSourceIds: Set<string>;
    saving: boolean;
    status: string;
    onOpenChange: (open: boolean) => void;
    onSelectedSourceIdsChange: (ids: string[]) => void;
    onConnectSelectedSources: () => void | Promise<void>;
  } = $props();

  let query = $state("");
  let providerFilters = $state<LibrarySourceProvider[]>([]);

  const columns = [
    { id: "selected", header: "", width: 44, cell: GridSelectCell },
    { id: "title", header: "Источник", width: 260, cell: LibrarySourceCell },
    { id: "provider", header: "Тип", width: 96 },
    { id: "projectCount", header: "Проекты", width: 80 },
    { id: "lastCollectedLabel", header: "Последний сбор", width: 140 },
    { id: "localCopyLabel", header: "Локальная копия", width: 120 },
    { id: "status", header: "Статус", width: 100 },
  ];

  let providerOptions = $derived(Array.from(new Set(librarySources.map((source) => source.provider))));
  let filteredSources = $derived(filterLibrarySources(librarySources, { query, providers: providerFilters }));
  let rows = $derived(
    filteredSources.map((source) => ({
      ...source,
      selected: selectedSourceIds.has(source.id),
      disabledReason: source.disabledReason,
    })),
  );
  let selectedConnectableCount = $derived(connectableSelection(librarySources, selectedSourceIds).length);
  let alreadyConnectedRows = $derived(librarySources.filter((source) => source.alreadyConnected));
  let refusedSelectedRows = $derived(
    librarySources.filter((source) => selectedSourceIds.has(source.id) && !source.connectable),
  );
  let jobRows = $derived(
    librarySources.filter((source) => source.status === "syncing" || source.status === "error"),
  );

  function toggleProvider(provider: LibrarySourceProvider) {
    providerFilters = providerFilters.includes(provider)
      ? providerFilters.filter((current) => current !== provider)
      : [...providerFilters, provider];
  }

  async function connectSelected() {
    await onConnectSelectedSources();
    onOpenChange(false);
  }
</script>

<ExtractumSheet {open} title="Connect from Library" description={project?.title ?? "Research project"}>
  <section class="connect-layout" data-ui-panel="library-connect">
    <div class="library-column">
      <div class="library-toolbar">
        <label class="library-search">
          <Search size={14} aria-hidden="true" />
          <ExtractumTextInput
            bind:value={query}
            placeholder="Поиск по источникам..."
            aria-label="Search library sources"
          />
        </label>
        <div class="provider-filters" aria-label="Provider filters">
          {#each providerOptions as provider (provider)}
            <button
              type="button"
              class:active={providerFilters.includes(provider)}
              onclick={() => toggleProvider(provider)}
            >
              <ProviderBadge {provider} />
            </button>
          {/each}
        </div>
      </div>

      <div class="library-grid">
        <ExtractumDataGrid
          rows={rows}
          {columns}
          selectedRowIds={Array.from(selectedSourceIds)}
          multiselect={true}
          onSelectedRowIdsChange={onSelectedSourceIdsChange}
          height="100%"
          overlay="No library sources"
        />
      </div>
    </div>

    <aside class="side-panels">
      <section data-ui-panel="change-log" class="side-panel">
        <h2>Connection status</h2>
        <div class="status-stack">
          {#each alreadyConnectedRows as source (source.id)}
            <p><StatusBadge status="already_connected" /> {source.title}</p>
          {/each}
          {#each refusedSelectedRows as source (source.id)}
            <p><StatusBadge status={source.status} /> {source.disabledReason}</p>
          {/each}
          {#each jobRows as source (source.id)}
            <p><StatusBadge status={source.status} /> {source.title}</p>
          {/each}
          {#if status}
            <p>{status}</p>
          {/if}
        </div>
      </section>

      <footer class="connect-actions">
        <span>{selectedConnectableCount} connectable selected</span>
        <ExtractumButton variant="outline" onclick={() => onOpenChange(false)}>
          <X size={14} aria-hidden="true" />
          Close
        </ExtractumButton>
        <ExtractumButton disabled={selectedConnectableCount === 0 || saving} onclick={connectSelected}>
          <Check size={14} aria-hidden="true" />
          Подключить выбранные
        </ExtractumButton>
      </footer>
    </aside>
  </section>
</ExtractumSheet>

<style>
  .connect-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
    height: min(720px, calc(100vh - 120px));
    min-height: min(720px, calc(100vh - 120px));
    gap: 14px;
    padding-top: 14px;
  }

  .library-column,
  .side-panels {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    gap: 12px;
  }

  .side-panels {
    max-height: min(720px, calc(100vh - 120px));
    overflow-y: auto;
    padding-right: 2px;
  }

  .library-toolbar {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .library-search {
    position: relative;
    min-width: 280px;
  }

  .library-search :global(svg) {
    position: absolute;
    top: 9px;
    left: 8px;
    color: var(--extractum-muted);
    pointer-events: none;
  }

  .library-search :global(input) {
    padding-left: 28px;
  }

  .provider-filters,
  .connect-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .provider-filters button {
    border: 0;
    background: transparent;
    padding: 0;
    opacity: 0.64;
  }

  .provider-filters button.active,
  .provider-filters button:hover {
    opacity: 1;
  }

  .library-grid {
    min-height: 420px;
    flex: 1;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    overflow: hidden;
  }

  .side-panel {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    padding: 12px;
  }

  .side-panel h2 {
    margin: 0 0 10px;
    font-size: 13px;
    letter-spacing: 0;
  }

  .side-panel p {
    margin: 0;
  }

  .status-stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .connect-actions {
    position: sticky;
    bottom: 0;
    z-index: 1;
    justify-content: flex-end;
    margin-top: auto;
    border-top: 1px solid var(--extractum-border);
    background: var(--extractum-surface);
    padding-top: 10px;
  }

  .connect-actions span {
    margin-right: auto;
    color: var(--extractum-muted);
    font-size: 12px;
  }
</style>
