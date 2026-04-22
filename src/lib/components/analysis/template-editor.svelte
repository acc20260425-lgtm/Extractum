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

  function canEditSelectedTemplate() {
    return !!selectedTemplate && selectedTemplate.is_builtin !== true;
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
      <button class="secondary" onclick={onSaveTemplateCopy} disabled={savingTemplate || deletingTemplate}>
        {savingTemplate ? "Saving..." : "Save as copy"}
      </button>
      <button onclick={onSaveTemplateChanges} disabled={savingTemplate || deletingTemplate || !canEditSelectedTemplate()}>
        {savingTemplate ? "Saving..." : "Save changes"}
      </button>
      <button class="danger-soft" onclick={onDeleteTemplate} disabled={savingTemplate || deletingTemplate || !canEditSelectedTemplate()}>
        {deletingTemplate ? "Deleting..." : "Delete"}
      </button>
    </div>
  </div>

  <div class="template-grid">
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
        rows="10"
        placeholder="Describe how the report should be structured and what it should emphasize."
        oninput={(event) => onChangeTemplateBody((event.currentTarget as HTMLTextAreaElement).value)}
      >{templateBody}</textarea>
    </label>
  </div>
</section>

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

  @media (max-width: 1080px) {
    .template-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
