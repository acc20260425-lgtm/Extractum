<script lang="ts">
  import { Download, Play } from "@lucide/svelte";
  import { ExtractumButton } from "$lib/components/extractum-ui";
  import { projectRunDisabledReason, type ResearchProjectView } from "$lib/ui/research-projects-model";
  import type { ProjectSourceRecord } from "$lib/types/projects";
  import type { LlmProfile } from "$lib/types/llm";

  const PROJECT_EXPORT_DISABLED_REASON = "Project export is not available yet.";

  let {
    project,
    sources,
    loading = false,
    onRunProject,
    llmProfiles = [],
    selectedProfileId = $bindable(""),
  }: {
    project: ResearchProjectView | null;
    sources: Pick<ProjectSourceRecord, "provider">[];
    loading?: boolean;
    onRunProject: () => void;
    llmProfiles?: LlmProfile[];
    selectedProfileId?: string;
  } = $props();

  const runDisabledReason = $derived(projectRunDisabledReason(project, sources));
  const noProject = $derived(project === null);
</script>

<div class="command-bar">
  <div class="project-context">
    <span>Research Projects</span>
    <strong>{project?.title ?? "No project selected"}</strong>
  </div>

  <div class="command-controls">
    <label>
      <span>Period</span>
      <select aria-label="Project period" disabled={noProject} title={noProject ? "Select a project first" : undefined}>
        <option>{project?.periodLabel ?? "01.01.2024 - 31.05.2025"}</option>
      </select>
    </label>
    <label>
      <span>Prompt</span>
      <select aria-label="Prompt preset" disabled={noProject} title={noProject ? "Select a project first" : undefined}>
        <option>Evidence brief</option>
        <option>Risk monitor</option>
      </select>
    </label>
    <label>
      <span>LLM Profile</span>
      <select bind:value={selectedProfileId} aria-label="LLM Profile" disabled={noProject} title={noProject ? "Select a project first" : undefined}>
        {#each llmProfiles as profile (profile.profile_id)}
          <option value={profile.profile_id}>
            {profile.profile_id} ({profile.default_model})
          </option>
        {/each}
      </select>
    </label>
<ExtractumButton
      disabled={loading || runDisabledReason !== null}
      onclick={onRunProject}
      aria-label={runDisabledReason ?? "Run project analysis"}
      title={runDisabledReason ?? "Run project analysis"}
    >
      <Play size={14} aria-hidden="true" />
      Run
    </ExtractumButton>
    <ExtractumButton
      variant="outline"
      disabled={true}
      title={PROJECT_EXPORT_DISABLED_REASON}
      aria-label={PROJECT_EXPORT_DISABLED_REASON}
      data-disabled-reason={PROJECT_EXPORT_DISABLED_REASON}
    >
      <Download size={14} aria-hidden="true" />
      Export
    </ExtractumButton>
  </div>
</div>

<style>
  .command-bar {
    display: flex;
    min-width: 0;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--extractum-border);
    background: var(--extractum-surface-raised);
  }

  .project-context {
    display: flex;
    flex: 0 0 auto;
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
    min-width: 0;
    flex-wrap: wrap;
    align-items: end;
    justify-content: flex-start;
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

  .command-controls select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
