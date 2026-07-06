<script lang="ts">
  import {
    ExtractumDialog,
    ExtractumTabs,
    ExtractumTabsContent,
    ExtractumTabsList,
    ExtractumTabsTrigger,
  } from "$lib/components/extractum-ui";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
  import LibraryTelegramDialogImport from "./LibraryTelegramDialogImport.svelte";
  import LibraryYoutubeAddPanel from "./LibraryYoutubeAddPanel.svelte";

  let {
    open = $bindable(false),
    sources,
    onSourcesChanged,
    onStatus,
    projectContext,
  }: {
    open?: boolean;
    sources: LibraryCatalogSourceView[];
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
    projectContext?: ProjectAddSourceContext;
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
        <LibraryYoutubeAddPanel {sources} {onSourcesChanged} {onStatus} {projectContext} />
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

  .library-add-source-dialog :global([data-slot="tabs-list"]) {
    gap: 3px;
    height: auto;
    padding: 3px;
    border: 1px solid color-mix(in srgb, var(--extractum-primary) 28%, var(--extractum-border));
    border-radius: calc(var(--extractum-radius) + 4px);
    background: color-mix(in srgb, var(--extractum-primary) 16%, var(--extractum-surface));
  }

  .library-add-source-dialog :global([data-slot="tabs-trigger"]) {
    min-height: 32px;
    border: 1px solid transparent;
    border-radius: var(--extractum-radius);
    padding-inline: 16px;
    background: color-mix(in srgb, var(--extractum-primary) 62%, var(--extractum-surface));
    color: rgb(255 255 255 / 0.76);
    opacity: 0.78;
    box-shadow: none;
  }

  .library-add-source-dialog :global([data-slot="tabs-trigger"]:hover) {
    opacity: 0.9;
    color: rgb(255 255 255 / 0.92);
  }

  .library-add-source-dialog :global([data-slot="tabs-trigger"][data-state="active"]) {
    border-color: color-mix(in srgb, var(--extractum-primary) 42%, white);
    background: var(--extractum-primary);
    color: white;
    opacity: 1;
    box-shadow:
      inset 0 0 0 1px rgb(255 255 255 / 0.22),
      0 2px 7px rgb(15 23 42 / 0.18);
  }
</style>
