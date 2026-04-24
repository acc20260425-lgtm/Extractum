<script lang="ts">
  import type { AnalysisPromptTemplate } from "$lib/types/analysis";

  let {
    selectedTemplate,
    templateName,
    templateBody,
    savingTemplate,
    deletingTemplate,
    onChangeTemplateName,
    onChangeTemplateBody,
    onSaveTemplateCopy,
    onSaveTemplateChanges,
    onDeleteTemplate,
  }: {
    selectedTemplate: AnalysisPromptTemplate | null;
    templateName: string;
    templateBody: string;
    savingTemplate: boolean;
    deletingTemplate: boolean;
    onChangeTemplateName: (value: string) => void;
    onChangeTemplateBody: (value: string) => void;
    onSaveTemplateCopy: () => void | Promise<void>;
    onSaveTemplateChanges: () => void | Promise<void>;
    onDeleteTemplate: () => void | Promise<void>;
  } = $props();

  let editorOpen = $state(false);

  function canEditSelectedTemplate() {
    return !!selectedTemplate && selectedTemplate.is_builtin !== true;
  }

  function openEditor() {
    editorOpen = true;
  }

  function closeEditor() {
    editorOpen = false;
  }
</script>

<section class="card templates">
  <div class="panel-header">
    <div>
      <h3>Prompt Template</h3>
      {#if selectedTemplate}
        <p class="sub">
          {selectedTemplate.name} - v{selectedTemplate.version}
          {selectedTemplate.is_builtin ? " - builtin (edit fields below, then save as copy)" : " - custom"}
        </p>
      {/if}
    </div>
    <div class="template-actions">
      <button
        class="secondary"
        onclick={openEditor}
        disabled={savingTemplate || deletingTemplate || (!selectedTemplate && !templateName.trim() && !templateBody.trim())}
      >
        {selectedTemplate ? "Edit template" : "Open editor"}
      </button>
      <button class="danger-soft" onclick={onDeleteTemplate} disabled={savingTemplate || deletingTemplate || !canEditSelectedTemplate()}>
        {deletingTemplate ? "Deleting..." : "Delete"}
      </button>
    </div>
  </div>

  <div class="template-grid">
    <label>Template name
      <input type="text" value={templateName} placeholder="Custom report" readonly />
    </label>

    <div class="template-preview">
      <div class="preview-header">
        <h4>Template Body</h4>
        <span>{templateBody.trim() ? `${templateBody.trim().split(/\s+/).length} words` : "Empty draft"}</span>
      </div>
      {#if templateBody.trim()}
        <p>{templateBody}</p>
      {:else}
        <p class="empty-copy">No template text yet. Open the editor to define the report instructions.</p>
      {/if}
    </div>
  </div>
</section>

<svelte:window
  onkeydown={(event) => {
    if (editorOpen && event.key === "Escape") {
      event.preventDefault();
      closeEditor();
    }
  }}
/>

{#if editorOpen}
  <div class="modal-backdrop" role="presentation" onclick={(event) => event.target === event.currentTarget && closeEditor()}>
    <div class="modal-card" role="dialog" aria-modal="true" aria-labelledby="template-editor-title">
      <header class="modal-header">
        <div>
          <h4 id="template-editor-title">{selectedTemplate ? "Edit Prompt Template" : "New Prompt Template"}</h4>
          <p class="sub">
            Shape how reports are structured, prioritized, and phrased before each analysis run.
          </p>
        </div>
        <button class="ghost close-button" type="button" onclick={closeEditor} aria-label="Close dialog">
          Close
        </button>
      </header>

      <div class="modal-body">
        <div class="editor-grid">
          <label>Template name
            <input
              type="text"
              value={templateName}
              placeholder="Custom report"
              oninput={(event) => onChangeTemplateName((event.currentTarget as HTMLInputElement).value)}
            />
          </label>

          <label>Template body
            <textarea
              rows="12"
              placeholder="Describe how the report should be structured and what it should emphasize."
              oninput={(event) => onChangeTemplateBody((event.currentTarget as HTMLTextAreaElement).value)}
            >{templateBody}</textarea>
          </label>
        </div>
      </div>

      <footer class="modal-actions">
        <button class="secondary" type="button" onclick={closeEditor}>
          Cancel
        </button>
        <button class="secondary" type="button" onclick={onSaveTemplateCopy} disabled={savingTemplate || deletingTemplate}>
          {savingTemplate ? "Saving..." : "Save as copy"}
        </button>
        <button
          type="button"
          onclick={onSaveTemplateChanges}
          disabled={savingTemplate || deletingTemplate || !canEditSelectedTemplate()}
        >
          {savingTemplate ? "Saving..." : "Save changes"}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .templates {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .sub {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .template-actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
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

  .template-grid input[readonly] {
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

  .preview-header span,
  .empty-copy {
    font-size: 0.85rem;
    color: var(--muted);
  }

  .template-preview p {
    margin: 0;
    white-space: pre-wrap;
    line-height: 1.5;
    color: var(--text);
  }

  textarea {
    width: 100%;
    resize: vertical;
    min-height: 10rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.8rem;
    border-radius: 8px;
    font: inherit;
  }

  textarea:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    background: color-mix(in srgb, #0f172a 28%, transparent);
    backdrop-filter: blur(6px);
  }

  .modal-card {
    width: min(46rem, calc(100vw - 2rem));
    max-height: min(88vh, 56rem);
    display: flex;
    flex-direction: column;
    border-radius: 16px;
    background: color-mix(in srgb, var(--panel) 97%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 94%, transparent);
    box-shadow:
      0 18px 42px rgba(15, 23, 42, 0.18),
      0 3px 10px rgba(15, 23, 42, 0.08);
    overflow: hidden;
  }

  .modal-header,
  .modal-body,
  .modal-actions {
    padding-left: 1rem;
    padding-right: 1rem;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    padding-top: 0.95rem;
    padding-bottom: 0.8rem;
    border-bottom: 1px solid var(--border);
  }

  .modal-header h4 {
    margin: 0 0 0.25rem;
    font-size: 0.98rem;
    font-weight: 650;
  }

  .modal-body {
    padding-top: 1rem;
    padding-bottom: 1rem;
    overflow: auto;
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
    padding-top: 0.85rem;
    padding-bottom: 0.95rem;
    border-top: 1px solid var(--border);
    background: color-mix(in srgb, var(--panel-strong) 54%, transparent);
  }

  .close-button {
    white-space: nowrap;
  }

  @media (max-width: 1080px) {
    .template-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .modal-backdrop {
      padding: 1rem;
      align-items: flex-end;
    }

    .modal-card {
      width: 100%;
      max-height: 92vh;
      border-radius: 18px 18px 14px 14px;
    }

    .modal-actions {
      flex-direction: column-reverse;
    }

    .modal-actions :global(button) {
      width: 100%;
    }
  }
</style>
