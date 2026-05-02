<script lang="ts">
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import CheckboxRow from "$lib/components/ui/CheckboxRow.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import type { NotebookLmExportResult, SourceRecord } from "$lib/types/sources";

  export type NotebookLmExportForm = {
    outputDir: string;
    fromDate: string;
    toDate: string;
    includeMediaPlaceholders: boolean;
    minMessageLength: number;
    maxWordsPerFile: number;
    maxBytesPerFile: number;
    overwriteExisting: boolean;
  };

  let {
    open,
    source,
    form,
    exporting,
    result,
    onClose,
    onChooseFolder,
    onExport,
    onChangeForm,
  }: {
    open: boolean;
    source: SourceRecord | null;
    form: NotebookLmExportForm;
    exporting: boolean;
    result: NotebookLmExportResult | null;
    onClose: () => void;
    onChooseFolder: () => void | Promise<void>;
    onExport: () => void | Promise<void>;
    onChangeForm: (form: NotebookLmExportForm) => void;
  } = $props();

  function updateForm(patch: Partial<NotebookLmExportForm>) {
    onChangeForm({ ...form, ...patch });
  }
</script>

<DesktopDialog
  {open}
  title="Export for NotebookLM"
  description={source ? `Prepare Markdown files for ${source.title ?? source.external_id}.` : ""}
  width="44rem"
  onClose={onClose}
>
  <div class="export-form">
    <div class="folder-row">
      <label>Output folder
        <Input
          value={form.outputDir}
          readonly={true}
          placeholder="Choose a folder"
          ariaLabel="NotebookLM export output folder"
        />
      </label>
      <Button variant="secondary" onclick={onChooseFolder} disabled={exporting}>
        Choose
      </Button>
    </div>

    <div class="field-grid">
      <label>From
        <Input
          type="date"
          value={form.fromDate}
          oninput={(event) => updateForm({ fromDate: (event.currentTarget as HTMLInputElement).value })}
          disabled={exporting}
        />
      </label>
      <label>To
        <Input
          type="date"
          value={form.toDate}
          oninput={(event) => updateForm({ toDate: (event.currentTarget as HTMLInputElement).value })}
          disabled={exporting}
        />
      </label>
      <label>Minimum length
        <Input
          type="number"
          min="0"
          value={form.minMessageLength}
          oninput={(event) => updateForm({ minMessageLength: Number((event.currentTarget as HTMLInputElement).value) })}
          disabled={exporting}
        />
      </label>
      <label>Max words
        <Input
          type="number"
          min="1"
          value={form.maxWordsPerFile}
          oninput={(event) => updateForm({ maxWordsPerFile: Number((event.currentTarget as HTMLInputElement).value) })}
          disabled={exporting}
        />
      </label>
      <label>Max bytes
        <Input
          type="number"
          min="1"
          value={form.maxBytesPerFile}
          oninput={(event) => updateForm({ maxBytesPerFile: Number((event.currentTarget as HTMLInputElement).value) })}
          disabled={exporting}
        />
      </label>
    </div>

    <div class="checks">
      <CheckboxRow
        title="Include media placeholders"
        description="Render stored media metadata as attachment notes."
        checked={form.includeMediaPlaceholders}
        disabled={exporting}
        onchange={(event) => updateForm({ includeMediaPlaceholders: (event.currentTarget as HTMLInputElement).checked })}
      />
      <CheckboxRow
        title="Overwrite deterministic export folder"
        description="Only replaces files in a marked Extractum NotebookLM export folder."
        checked={form.overwriteExisting}
        disabled={exporting}
        onchange={(event) => updateForm({ overwriteExisting: (event.currentTarget as HTMLInputElement).checked })}
      />
    </div>

    {#if result}
      <div class="result-box">
        <strong>Export complete</strong>
        <span>{result.files.length} files, {result.exported_message_count} messages, {result.skipped_message_count} skipped.</span>
        <span class="path">{result.output_dir}</span>
        {#if result.warnings.length > 0}
          <span>{result.warnings.length} warnings. First: {result.warnings[0]}</span>
        {/if}
      </div>
    {/if}

    <div class="actions">
      <Button variant="ghost" onclick={onClose} disabled={exporting}>Close</Button>
      <Button onclick={onExport} disabled={exporting || !source || !form.outputDir.trim()}>
        {exporting ? "Exporting..." : "Export"}
      </Button>
    </div>
  </div>
</DesktopDialog>

<style>
  .export-form {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    color: var(--muted);
    font-size: 0.83rem;
  }

  .folder-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
    gap: 0.65rem;
  }

  .field-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.75rem;
  }

  .checks {
    display: grid;
    gap: 0.65rem;
  }

  .result-box {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.85rem;
    border: 1px solid color-mix(in srgb, var(--success) 30%, var(--border));
    border-radius: 10px;
    background: color-mix(in srgb, var(--success) 8%, var(--panel));
    font-size: 0.86rem;
  }

  .path {
    color: var(--muted);
    overflow-wrap: anywhere;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.55rem;
  }

  @media (max-width: 680px) {
    .folder-row,
    .field-grid {
      grid-template-columns: 1fr;
    }

    .actions {
      justify-content: stretch;
    }
  }
</style>
