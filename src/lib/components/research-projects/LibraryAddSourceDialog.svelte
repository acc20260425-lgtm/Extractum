<script lang="ts">
  import {
    ExtractumDialog,
    ExtractumTabs,
    ExtractumTabsContent,
    ExtractumTabsList,
    ExtractumTabsTrigger,
  } from "$lib/components/extractum-ui";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import LibraryTelegramDialogImport from "./LibraryTelegramDialogImport.svelte";
  import LibraryYoutubeAddPanel from "./LibraryYoutubeAddPanel.svelte";

  let {
    open = $bindable(false),
    sources,
    onSourcesChanged,
    onStatus,
  }: {
    open?: boolean;
    sources: LibraryCatalogSourceView[];
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let provider = $state<"youtube" | "telegram">("youtube");
</script>

<ExtractumDialog
  bind:open
  title="Add source"
  description="Add YouTube sources or import Telegram sources from an authorized account."
>
  <section class="library-add-source-dialog" data-ui-region="library-add-source-dialog">
    <ExtractumTabs bind:value={provider}>
      <ExtractumTabsList aria-label="Source providers">
        <ExtractumTabsTrigger value="youtube">YouTube</ExtractumTabsTrigger>
        <ExtractumTabsTrigger value="telegram">Telegram</ExtractumTabsTrigger>
      </ExtractumTabsList>

      <ExtractumTabsContent value="youtube">
        <LibraryYoutubeAddPanel {sources} {onSourcesChanged} {onStatus} />
      </ExtractumTabsContent>

      <ExtractumTabsContent value="telegram">
        <LibraryTelegramDialogImport {onSourcesChanged} {onStatus} />
      </ExtractumTabsContent>
    </ExtractumTabs>
  </section>
</ExtractumDialog>

<style>
  .library-add-source-dialog {
    min-height: 520px;
    display: grid;
  }

  .library-add-source-dialog :global([data-slot="tabs"]) {
    min-height: 0;
  }
</style>
