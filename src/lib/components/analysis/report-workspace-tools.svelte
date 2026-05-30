<script lang="ts">
  import { Download, Folder, SquarePen } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";

  let {
    showNotebookLmExport,
    canExportNotebookLm,
    exportDisabledReason,
    exportingNotebookLm,
    templateEditorOpen,
    groupEditorOpen,
    onOpenNotebookLmExport,
    onToggleTemplateEditor,
    onToggleGroupEditor,
  }: {
    showNotebookLmExport: boolean;
    canExportNotebookLm: boolean;
    exportDisabledReason: string | null;
    exportingNotebookLm: boolean;
    templateEditorOpen: boolean;
    groupEditorOpen: boolean;
    onOpenNotebookLmExport: () => void;
    onToggleTemplateEditor: () => void;
    onToggleGroupEditor: () => void;
  } = $props();

  const exportReasonId = "notebooklm-export-disabled-reason";
</script>

<section class="report-workspace-tools" aria-label="Workspace tools" data-smoke-id="analysis-workspace-tools">
  <div class="workspace-tools-copy">
    <span class="eyebrow">Workspace tools</span>
  </div>

  <div class="workspace-tools-actions">
    {#if showNotebookLmExport}
      <div class="workspace-tool-action">
        <Button
          type="button"
          variant="secondary"
          onclick={onOpenNotebookLmExport}
          disabled={!canExportNotebookLm}
          ariaDescribedby={exportDisabledReason ? exportReasonId : undefined}
          smokeId="notebooklm-export-button"
          title={exportDisabledReason ?? undefined}
        >
          <Download size={15} aria-hidden="true" />
          {exportingNotebookLm ? "Exporting..." : "Export for NotebookLM"}
        </Button>
        {#if exportDisabledReason}
          <span id="notebooklm-export-disabled-reason" class="workspace-tool-helper" data-smoke-id="notebooklm-export-disabled-reason">
            {exportDisabledReason}
          </span>
        {/if}
      </div>
    {/if}

    <Button
      type="button"
      variant="secondary"
      ariaExpanded={templateEditorOpen}
      onclick={onToggleTemplateEditor}
    >
      <SquarePen size={15} aria-hidden="true" />
      {templateEditorOpen ? "Hide templates" : "Edit templates"}
    </Button>

    <Button
      type="button"
      variant="secondary"
      ariaExpanded={groupEditorOpen}
      onclick={onToggleGroupEditor}
    >
      <Folder size={15} aria-hidden="true" />
      {groupEditorOpen ? "Hide groups" : "Edit groups"}
    </Button>
  </div>
</section>

<style>
  .report-workspace-tools {
    display: flex;
    justify-content: space-between;
    gap: 0.8rem;
    align-items: center;
    padding: 0.85rem 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow-soft);
  }

  .workspace-tools-copy {
    min-width: 0;
  }

  .workspace-tools-actions {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .workspace-tool-action {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-items: flex-start;
  }

  .workspace-tool-helper {
    max-width: 18rem;
    color: var(--muted);
    font-size: 0.74rem;
    line-height: 1.35;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  @media (max-width: 720px) {
    .report-workspace-tools {
      flex-direction: column;
      align-items: stretch;
    }

    .workspace-tools-actions {
      justify-content: flex-start;
    }
  }
</style>
