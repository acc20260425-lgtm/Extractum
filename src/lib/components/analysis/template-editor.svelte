<script lang="ts">
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import PanelHeader from "$lib/components/ui/PanelHeader.svelte";
  import Textarea from "$lib/components/ui/Textarea.svelte";
  import type { AnalysisPromptTemplate } from "$lib/types/analysis";

  let {
    compact = false,
    selectedTemplate,
    templateName,
    templateBody,
    savingTemplate,
    deletingTemplate,
    onSaveTemplateCopy,
    onSaveTemplateChanges,
    onDeleteTemplate,
  }: {
    compact?: boolean;
    selectedTemplate: AnalysisPromptTemplate | null;
    templateName: string;
    templateBody: string;
    savingTemplate: boolean;
    deletingTemplate: boolean;
    onSaveTemplateCopy: (name: string, body: string) => void | Promise<void>;
    onSaveTemplateChanges: (name: string, body: string) => void | Promise<void>;
    onDeleteTemplate: () => void | Promise<void>;
  } = $props();

  let editorOpen = $state(false);
  let editorMode = $state<"edit" | "new">("edit");
  let draftName = $state("");
  let draftBody = $state("");

  function canEditSelectedTemplate() {
    return !!selectedTemplate && selectedTemplate.is_builtin !== true;
  }

  function openEditor(mode: "edit" | "new") {
    editorMode = mode;
    draftName = mode === "new" ? "" : templateName;
    draftBody = mode === "new" ? "" : templateBody;
    editorOpen = true;
  }

  function closeEditor() {
    editorOpen = false;
  }

  function saveCopy() {
    return onSaveTemplateCopy(draftName, draftBody);
  }

  function saveChanges() {
    return onSaveTemplateChanges(draftName, draftBody);
  }
</script>

{#if compact}
  <div class="utility-card compact">
    <div class="compact-header">
      <div class="compact-copy">
        <span class="compact-kicker">Prompt template</span>
        <strong>{selectedTemplate ? `${selectedTemplate.name} - v${selectedTemplate.version}` : "No template selected"}</strong>
        <span class="compact-sub">
          {selectedTemplate
            ? selectedTemplate.is_builtin
              ? "Built-in template. Edit and save as a copy."
              : "Custom template ready for editing."
            : "Open the editor to create or adjust report instructions."}
        </span>
      </div>
      <div class="template-actions">
        <Button variant="secondary" size="sm" onclick={() => openEditor("new")} disabled={savingTemplate || deletingTemplate}>
          New
        </Button>
        <Button
          variant="secondary"
          size="sm"
          onclick={() => openEditor("edit")}
          disabled={savingTemplate || deletingTemplate || (!selectedTemplate && !templateName.trim() && !templateBody.trim())}
        >
          Edit
        </Button>
        <Button
          variant="danger-soft"
          size="sm"
          onclick={onDeleteTemplate}
          disabled={savingTemplate || deletingTemplate || !canEditSelectedTemplate()}
        >
          {deletingTemplate ? "Deleting..." : "Delete"}
        </Button>
      </div>
    </div>
  </div>
{:else}
  <Card>
    <div class="templates">
      <PanelHeader
        title="Prompt Template"
        subtitle={selectedTemplate
          ? `${selectedTemplate.name} - v${selectedTemplate.version}${selectedTemplate.is_builtin ? " - builtin (edit fields below, then save as copy)" : " - custom"}`
          : ""}
      >
        <div class="template-actions">
          <Button variant="secondary" onclick={() => openEditor("new")} disabled={savingTemplate || deletingTemplate}>
            New template
          </Button>
          <Button
            variant="secondary"
            onclick={() => openEditor("edit")}
            disabled={savingTemplate || deletingTemplate || (!selectedTemplate && !templateName.trim() && !templateBody.trim())}
          >
            {selectedTemplate ? "Edit template" : "Open editor"}
          </Button>
          <Button
            variant="danger-soft"
            onclick={onDeleteTemplate}
            disabled={savingTemplate || deletingTemplate || !canEditSelectedTemplate()}
          >
            {deletingTemplate ? "Deleting..." : "Delete"}
          </Button>
        </div>
      </PanelHeader>

      <div class="template-grid">
        <label>Template name
          <Input type="text" value={templateName} placeholder="Custom report" readonly />
        </label>

        <div class="template-preview">
          <div class="preview-header">
            <h4>Template Body</h4>
            <span>{templateBody.trim() ? `${templateBody.trim().split(/\s+/).length} words` : "Empty draft"}</span>
          </div>
          {#if templateBody.trim()}
            <p>{templateBody}</p>
          {:else}
            <EmptyState description="No template text yet. Open the editor to define the report instructions." />
          {/if}
        </div>
      </div>
    </div>
  </Card>
{/if}

<DesktopDialog
  open={editorOpen}
  title={editorMode === "new" ? "New Prompt Template" : "Edit Prompt Template"}
  description="Shape how reports are structured, prioritized, and phrased before each analysis run."
  labelledBy="template-editor-title"
  width="46rem"
  onClose={closeEditor}
>
  <div class="editor-grid">
    <label>Template name
      <Input
        type="text"
        value={draftName}
        placeholder="Custom report"
        oninput={(event) => (draftName = (event.currentTarget as HTMLInputElement).value)}
      />
    </label>

    <label>Template body
      <Textarea
        value={draftBody}
        rows={12}
        placeholder="Describe how the report should be structured and what it should emphasize."
        oninput={(event) => (draftBody = (event.currentTarget as HTMLTextAreaElement).value)}
        className="template-body-field"
      />
    </label>

    <footer class="modal-actions">
      <Button variant="secondary" type="button" onclick={closeEditor}>
        Cancel
      </Button>
      <Button variant="secondary" type="button" onclick={saveCopy} disabled={savingTemplate || deletingTemplate}>
        {savingTemplate ? "Saving..." : editorMode === "new" ? "Create template" : "Save as copy"}
      </Button>
      {#if editorMode === "edit"}
        <Button
          type="button"
          onclick={saveChanges}
          disabled={savingTemplate || deletingTemplate || !canEditSelectedTemplate()}
        >
          {savingTemplate ? "Saving..." : "Save changes"}
        </Button>
      {/if}
    </footer>
  </div>
</DesktopDialog>

<style>
  .templates {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .template-actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .utility-card {
    padding: 0.95rem 1rem;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .compact-header {
    display: flex;
    justify-content: space-between;
    gap: 0.8rem;
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .compact-copy {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .compact-kicker {
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .compact-copy strong {
    font-size: 0.92rem;
  }

  .compact-sub {
    color: var(--muted);
    font-size: 0.8rem;
    line-height: 1.4;
  }

  .template-grid {
    display: grid;
    grid-template-columns: minmax(260px, 360px) minmax(0, 1fr);
    gap: 1rem;
    align-items: start;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .template-grid :global(input[readonly]) {
    cursor: default;
    color: var(--text);
    background: var(--panel-strong);
  }

  .template-preview {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    min-height: 10rem;
    padding: 0.95rem 1rem;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--panel-strong);
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    color: var(--muted);
  }

  .preview-header h4 {
    margin: 0;
    font-size: 0.92rem;
    color: var(--text);
  }

  .preview-header span {
    font-size: 0.85rem;
    color: var(--muted);
  }

  .template-preview p {
    margin: 0;
    white-space: pre-wrap;
    line-height: 1.5;
    color: var(--text);
  }

  .editor-grid :global(.ui-textarea.template-body-field) {
    min-height: 10rem;
  }

  .editor-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding-top: 0.25rem;
    border-top: 1px solid var(--border);
    margin-top: 0.25rem;
  }

  @media (max-width: 1080px) {
    .template-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .compact-header,
    .modal-actions {
      flex-direction: column-reverse;
    }

    .compact-header .template-actions,
    .modal-actions :global(button) {
      width: 100%;
    }
  }
</style>
