<script lang="ts">
  import {
    ExtractumTabs,
    ExtractumTabsContent,
    ExtractumTabsList,
    ExtractumTabsTrigger,
  } from "$lib/components/extractum-ui";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
  import LibraryYoutubePlaylistImport from "./LibraryYoutubePlaylistImport.svelte";
  import LibraryYoutubeSmartImport from "./LibraryYoutubeSmartImport.svelte";

  let {
    sources,
    onSourcesChanged,
    onStatus,
    projectContext,
  }: {
    sources: LibraryCatalogSourceView[];
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
    projectContext?: ProjectAddSourceContext;
  } = $props();

  let mode = $state<"smart" | "existing">("smart");
</script>

<section class="library-youtube-add-panel" aria-label="YouTube Add Source">
  <ExtractumTabs bind:value={mode}>
    <ExtractumTabsList aria-label="YouTube import modes">
      <ExtractumTabsTrigger value="smart">Smart import</ExtractumTabsTrigger>
      <ExtractumTabsTrigger value="existing">From existing data</ExtractumTabsTrigger>
    </ExtractumTabsList>

    <ExtractumTabsContent value="smart">
      <LibraryYoutubeSmartImport {sources} {onSourcesChanged} {onStatus} {projectContext} />
    </ExtractumTabsContent>

    <ExtractumTabsContent value="existing">
      <LibraryYoutubePlaylistImport {sources} {onSourcesChanged} {onStatus} {projectContext} />
    </ExtractumTabsContent>
  </ExtractumTabs>
</section>

<style>
  .library-youtube-add-panel {
    min-height: 0;
  }

  .library-youtube-add-panel :global([data-slot="tabs"]) {
    min-height: 0;
  }
</style>
