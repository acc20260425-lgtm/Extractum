<script lang="ts">
  import { Download, Folder, SquarePen } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";

  let {
    compact = false,
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
    compact?: boolean;
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
  const notebookLmExportLabel = $derived(exportingNotebookLm ? "Exporting..." : "Export for NotebookLM");
  const templateEditorLabel = $derived(templateEditorOpen ? "Hide templates" : "Edit templates");
  const groupEditorLabel = $derived(groupEditorOpen ? "Hide groups" : "Edit groups");
</script>

<section
  class="report-workspace-tools"
  class:compact={compact}
  aria-label="Workspace actions"
  data-smoke-id="analysis-workspace-tools"
>
  {#if !compact}
    <div class="workspace-tools-copy">
      <span class="eyebrow">Workspace tools</span>
    </div>
  {/if}

  <div class="workspace-tools-actions">
    {#if showNotebookLmExport}
      <div class="workspace-tool-action">
        <Button
          type="button"
          variant="secondary"
          size={compact ? "sm" : "md"}
          iconOnly={compact}
          onclick={onOpenNotebookLmExport}
          disabled={!canExportNotebookLm}
          ariaLabel={compact ? notebookLmExportLabel : undefined}
          ariaDescribedby={!compact && exportDisabledReason ? exportReasonId : undefined}
          smokeId="notebooklm-export-button"
          title={exportDisabledReason ?? (compact ? notebookLmExportLabel : undefined)}
        >
          <Download size={15} aria-hidden="true" />
          {#if !compact}{notebookLmExportLabel}{/if}
        </Button>
        {#if !compact && exportDisabledReason}
          <span id={exportReasonId} class="workspace-tool-helper" data-smoke-id="notebooklm-export-disabled-reason">
            {exportDisabledReason}
          </span>
        {/if}
      </div>
    {/if}

    <Button
      type="button"
      variant="secondary"
      size={compact ? "sm" : "md"}
      iconOnly={compact}
      ariaLabel={compact ? templateEditorLabel : undefined}
      ariaExpanded={templateEditorOpen}
      title={compact ? templateEditorLabel : undefined}
      onclick={onToggleTemplateEditor}
    >
      <SquarePen size={15} aria-hidden="true" />
      {#if !compact}{templateEditorLabel}{/if}
    </Button>

    <Button
      type="button"
      variant="secondary"
      size={compact ? "sm" : "md"}
      iconOnly={compact}
      ariaLabel={compact ? groupEditorLabel : undefined}
      ariaExpanded={groupEditorOpen}
      title={compact ? groupEditorLabel : undefined}
      onclick={onToggleGroupEditor}
    >
      <Folder size={15} aria-hidden="true" />
      {#if !compact}{groupEditorLabel}{/if}
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

  .report-workspace-tools.compact {
    border: 0;
    padding: 0;
    background: transparent;
    box-shadow: none;
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

  .report-workspace-tools.compact .workspace-tools-actions {
    justify-content: flex-end;
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

    .report-workspace-tools.compact {
      align-items: stretch;
    }

    .workspace-tools-actions {
      justify-content: flex-start;
    }
  }
</style>
