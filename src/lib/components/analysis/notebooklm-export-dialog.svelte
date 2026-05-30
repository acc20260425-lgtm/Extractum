<script lang="ts">
  import { Download, FolderOpen, X } from "@lucide/svelte";
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import CheckboxRow from "$lib/components/ui/CheckboxRow.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import type {
    NotebookLmExportEvent,
    NotebookLmExportResult,
    Source,
  } from "$lib/types/sources";

  export type NotebookLmExportRange = "entire_history" | "analysis_period";

  export type NotebookLmExportForm = {
    outputDir: string;
    range: NotebookLmExportRange;
    fromDate: string;
    toDate: string;
    includeMediaPlaceholders: boolean;
    includeMigratedHistory: boolean;
    minMessageLength: number;
    maxWordsPerFile: number;
    maxBytesPerFile: number;
    overwriteExisting: boolean;
  };

  export type NotebookLmExportProgressState = {
    phase: NotebookLmExportEvent["phase"];
    message: string;
    current: number | null;
    total: number | null;
  };

  let {
    open,
    source,
    form,
    exporting,
    progress,
    result,
    onClose,
    onChooseFolder,
    onExport,
    onChangeForm,
  }: {
    open: boolean;
    source: Source | null;
    form: NotebookLmExportForm;
    exporting: boolean;
    progress: NotebookLmExportProgressState | null;
    result: NotebookLmExportResult | null;
    onClose: () => void;
    onChooseFolder: () => void | Promise<void>;
    onExport: () => void | Promise<void>;
    onChangeForm: (form: NotebookLmExportForm) => void;
  } = $props();

  function updateForm(patch: Partial<NotebookLmExportForm>) {
    onChangeForm({ ...form, ...patch });
  }

  function positiveNumberInput(event: Event, fallback: number) {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    return Number.isFinite(value) && value >= 1 ? value : fallback;
  }

  function setRange(range: NotebookLmExportRange) {
    updateForm({ range });
  }

  function progressValue(current: number | null, total: number | null) {
    if (current === null || total === null || total <= 0) {
      return 0;
    }
    return Math.min(100, Math.round((current / total) * 100));
  }

  function hasKnownProgressTotal(current: number | null, total: number | null) {
    return current !== null && total !== null && total > 0;
  }

  function phaseLabel(phase: NotebookLmExportEvent["phase"]) {
    switch (phase) {
      case "loading":
        return "Loading";
      case "filtering":
        return "Filtering";
      case "chunking":
        return "Chunking";
      case "preparing_output":
        return "Preparing";
      case "writing":
        return "Writing";
      case "manifest":
        return "Manifest";
      case "completed":
        return "Completed";
      case "failed":
        return "Failed";
      default:
        return "Exporting";
    }
  }
</script>

<DesktopDialog
  {open}
  title="Export for NotebookLM"
  description={source ? `Prepare Markdown files for ${source.title ?? source.externalId}.` : ""}
  width="44rem"
  smokeId="notebooklm-export-dialog"
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
        <FolderOpen size={15} aria-hidden="true" />
        Choose
      </Button>
    </div>

    <div class="range-field">
      <span>Export range</span>
      <div class="segmented" role="group" aria-label="NotebookLM export range">
        <button
          type="button"
          class:active={form.range === "entire_history"}
          aria-pressed={form.range === "entire_history"}
          disabled={exporting}
          onclick={() => setRange("entire_history")}
        >
          Entire history
        </button>
        <button
          type="button"
          class:active={form.range === "analysis_period"}
          aria-pressed={form.range === "analysis_period"}
          disabled={exporting}
          onclick={() => setRange("analysis_period")}
        >
          Current period
        </button>
      </div>
    </div>

    <div class="field-grid">
      <label>From
        <Input
          type="date"
          value={form.fromDate}
          oninput={(event) => updateForm({ fromDate: (event.currentTarget as HTMLInputElement).value })}
          disabled={exporting || form.range === "entire_history"}
        />
      </label>
      <label>To
        <Input
          type="date"
          value={form.toDate}
          oninput={(event) => updateForm({ toDate: (event.currentTarget as HTMLInputElement).value })}
          disabled={exporting || form.range === "entire_history"}
        />
      </label>
      <label>Minimum length
        <Input
          type="number"
          min="1"
          value={form.minMessageLength}
          oninput={(event) => updateForm({ minMessageLength: positiveNumberInput(event, 1) })}
          disabled={exporting}
        />
      </label>
      <label>Max words
        <Input
          type="number"
          min="1"
          value={form.maxWordsPerFile}
          oninput={(event) => updateForm({ maxWordsPerFile: positiveNumberInput(event, 1) })}
          disabled={exporting}
        />
      </label>
      <label>Max bytes
        <Input
          type="number"
          min="1"
          value={form.maxBytesPerFile}
          oninput={(event) => updateForm({ maxBytesPerFile: positiveNumberInput(event, 1) })}
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
      {#if source?.sourceType === "telegram" && source.migratedHistoryRowCount > 0}
        <CheckboxRow
          title="Include migrated historical scope"
          description="Export current and migrated history as separate sections."
          checked={form.includeMigratedHistory}
          disabled={exporting}
          onchange={(event) => updateForm({ includeMigratedHistory: (event.currentTarget as HTMLInputElement).checked })}
        />
      {/if}
      <CheckboxRow
        title="Overwrite deterministic export folder"
        description="Only replaces files in a marked Extractum NotebookLM export folder."
        checked={form.overwriteExisting}
        disabled={exporting}
        onchange={(event) => updateForm({ overwriteExisting: (event.currentTarget as HTMLInputElement).checked })}
      />
    </div>

    {#if exporting && progress}
      <div class="progress-box">
        <div class="progress-head">
          <strong>{phaseLabel(progress.phase)}</strong>
          {#if hasKnownProgressTotal(progress.current, progress.total)}
            <span>{progress.current} / {progress.total}</span>
          {/if}
        </div>
        {#if progress.message}
          <span class="progress-message">{progress.message}</span>
        {/if}
        {#if hasKnownProgressTotal(progress.current, progress.total)}
          <progress max="100" value={progressValue(progress.current, progress.total)}>
            {progressValue(progress.current, progress.total)}%
          </progress>
        {:else}
          <progress></progress>
        {/if}
      </div>
    {/if}

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
      <Button variant="ghost" onclick={onClose} disabled={exporting}>
        <X size={15} aria-hidden="true" /> Close
      </Button>
      <Button onclick={onExport} disabled={exporting || !source || !form.outputDir.trim()}>
        <Download size={15} aria-hidden="true" />
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

  .range-field {
    display: grid;
    gap: 0.4rem;
  }

  .range-field > span {
    color: var(--muted);
    font-size: 0.83rem;
  }

  .segmented {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.25rem;
    padding: 0.25rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: color-mix(in srgb, var(--text) 4%, var(--panel));
  }

  .segmented button {
    min-height: 2.25rem;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--muted);
    font: inherit;
    font-size: 0.86rem;
    cursor: pointer;
  }

  .segmented button.active {
    background: var(--panel);
    color: var(--text);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--primary) 35%, var(--border));
  }

  .segmented button:disabled {
    cursor: not-allowed;
    opacity: 0.65;
  }

  .checks {
    display: grid;
    gap: 0.65rem;
  }

  .progress-box {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    padding: 0.85rem;
    border: 1px solid color-mix(in srgb, var(--primary) 24%, var(--border));
    border-radius: 10px;
    background: color-mix(in srgb, var(--primary) 7%, var(--panel));
    font-size: 0.86rem;
  }

  .progress-head {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .progress-head span,
  .progress-message {
    color: var(--muted);
  }

  progress {
    width: 100%;
    height: 0.55rem;
    accent-color: var(--primary);
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
    .field-grid,
    .segmented {
      grid-template-columns: 1fr;
    }

    .actions {
      justify-content: flex-end;
    }
  }
</style>
