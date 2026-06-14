<script lang="ts">
  import { Library } from "@lucide/svelte";
  import {
    ExtractumButton,
    ExtractumDataGrid,
    ProviderBadge,
    StatusBadge,
  } from "$lib/components/extractum-ui";
  import type {
    LibrarySourceView,
    ProjectSourceLinkView,
    ResearchProjectView,
  } from "$lib/ui/research-projects-model";
  import type { LibrarySourceProvider } from "$lib/types/library-sources";
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
    selectedSourceId,
    onSelectedSourceIdChange,
    onOpenConnectLibrary,
  }: {
    project: ResearchProjectView | null;
    projectSourceLinks: ProjectSourceLinkView[];
    librarySources: LibrarySourceView[];
    selectedSourceId: string | null;
    onSelectedSourceIdChange: (sourceId: string | null) => void;
    onOpenConnectLibrary: () => void;
  } = $props();

  const columns = [
    { id: "title", header: "Title", flexgrow: 1, cell: LibrarySourceCell },
    { id: "provider", header: "Provider", width: 120 },
    { id: "subtype", header: "Subtype", width: 120 },
    { id: "localCopyLabel", header: "Details", width: 140 },
    { id: "addedAtLabel", header: "Added to project at", width: 180 },
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

  let provider = $derived<LibrarySourceProvider>(rows[0]?.provider ?? "other");
</script>

<section class="sources-tab">
  <header class="sources-toolbar">
    <div class="sources-context">
      <ProviderBadge {provider} />
      <StatusBadge status={rows.length > 0 ? "connected" : "unavailable"} />
    </div>
    <ExtractumButton data-ui-action="connect-library" onclick={onOpenConnectLibrary}>
      <Library size={14} aria-hidden="true" />
      Connect from Library
    </ExtractumButton>
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
      selectedRowIds={selectedSourceId ? [selectedSourceId] : []}
      onSelectedRowIdsChange={(ids) => onSelectedSourceIdChange(ids[0] ?? null)}
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
    justify-content: space-between;
    gap: 12px;
  }

  .sources-context {
    display: flex;
    align-items: center;
    gap: 8px;
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
