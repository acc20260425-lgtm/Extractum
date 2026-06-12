<script lang="ts">
  import { Download, Play } from "@lucide/svelte";
  import { ExtractumButton, ProviderBadge } from "$lib/components/extractum-ui";
  import type { ResearchProjectView } from "$lib/ui/research-projects-model";

  let {
    project,
    loading = false,
  }: {
    project: ResearchProjectView | null;
    loading?: boolean;
  } = $props();
</script>

<div class="command-bar">
  <div class="project-context">
    <span>Research Projects</span>
    <strong>{project?.title ?? "No project selected"}</strong>
  </div>

  <div class="command-controls">
    <label>
      <span>Period</span>
      <select aria-label="Project period">
        <option>{project?.periodLabel ?? "01.01.2024 - 31.05.2025"}</option>
      </select>
    </label>
    <label>
      <span>Prompt</span>
      <select aria-label="Prompt preset">
        <option>Evidence brief</option>
        <option>Risk monitor</option>
      </select>
    </label>
    <label>
      <span>Model</span>
      <select aria-label="Model">
        <option>GPT-4.1</option>
        <option>Local profile</option>
      </select>
    </label>
    <ProviderBadge provider="telegram" label="Library" />
    <ExtractumButton disabled={loading || !project}>
      <Play size={14} aria-hidden="true" />
      Run
    </ExtractumButton>
    <ExtractumButton variant="outline" disabled={!project}>
      <Download size={14} aria-hidden="true" />
      Export
    </ExtractumButton>
  </div>
</div>

<style>
  .command-bar {
    display: flex;
    min-height: 58px;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--extractum-border);
    background: var(--extractum-surface-raised);
  }

  .project-context {
    display: flex;
    min-width: 180px;
    flex-direction: column;
    gap: 2px;
  }

  .project-context span,
  .command-controls label span {
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  .project-context strong {
    font-size: 15px;
  }

  .command-controls {
    display: flex;
    align-items: end;
    gap: 8px;
  }

  .command-controls label {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .command-controls select {
    height: 32px;
    min-width: 132px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    color: var(--extractum-text);
    font-size: 13px;
  }
</style>
