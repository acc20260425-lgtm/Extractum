<script lang="ts">
  import { ExtractumButton, ExtractumDialog, ExtractumTextInput } from "$lib/components/extractum-ui";
  import type { ResearchProjectView } from "$lib/ui/research-projects-model";

  let {
    open = $bindable(false),
    project = null,
    saving = false,
    error = "",
    onSubmit,
  }: {
    open?: boolean;
    project?: ResearchProjectView | null;
    saving?: boolean;
    error?: string;
    onSubmit: (input: { name: string; description: string | null }) => void | Promise<void>;
  } = $props();

  let name = $state("");
  let description = $state("");

  $effect(() => {
    if (open) {
      name = project?.title ?? "";
      description = project?.description ?? "";
    }
  });

  async function submit() {
    await onSubmit({ name: name.trim(), description: description.trim() || null });
    open = false;
  }
</script>

<ExtractumDialog bind:open title={project ? "Edit project" : "Create project"}>
  <form class="project-editor" onsubmit={(event) => { event.preventDefault(); void submit(); }}>
    <label>
      <span>Name</span>
      <ExtractumTextInput bind:value={name} aria-label="Project name" />
    </label>
    <label>
      <span>Description</span>
      <textarea bind:value={description} aria-label="Project description"></textarea>
    </label>
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <footer>
      <ExtractumButton type="button" variant="outline" onclick={() => (open = false)}>Cancel</ExtractumButton>
      <ExtractumButton type="submit" disabled={saving || name.trim().length === 0}>
        {project ? "Save" : "Create"}
      </ExtractumButton>
    </footer>
  </form>
</ExtractumDialog>

<style>
  .project-editor {
    display: flex;
    min-width: min(420px, calc(100vw - 96px));
    flex-direction: column;
    gap: 12px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label span {
    color: var(--extractum-muted);
    font-size: 12px;
  }

  textarea {
    min-height: 96px;
    resize: vertical;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    color: var(--extractum-text);
    padding: 8px;
  }

  .error {
    color: var(--extractum-danger);
    font-size: 13px;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
