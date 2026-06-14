<script lang="ts">
  import { onMount } from "svelte";
  import { AlertTriangle, FileText, RefreshCw } from "@lucide/svelte";
  import {
    getPromptPackResult,
    getPromptPackValidationFindings,
  } from "$lib/api/prompt-packs";
  import { ExtractumBadge, ExtractumButton } from "$lib/components/extractum-ui";
  import type {
    PromptPackResult,
    PromptPackRunListItem,
    PromptPackValidationFinding,
  } from "$lib/types/prompt-packs";

  let {
    run = null,
    runId = null,
  }: {
    run?: PromptPackRunListItem | null;
    runId?: number | null;
  } = $props();

  let result = $state<PromptPackResult | null>(null);
  let findings = $state<PromptPackValidationFinding[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const selectedRunId = $derived(run?.runId ?? runId);
  const canonical = $derived((result?.canonical ?? {}) as Record<string, unknown>);
  const outputData = $derived(recordAt(recordAt(recordAt(canonical, "outputs"), "pack_data"), "youtube_summary"));
  const videos = $derived(arrayAt(outputData, "videos"));
  const claims = $derived(arrayAt(canonical, "claims"));
  const evidence = $derived(arrayAt(canonical, "evidence"));
  const limitations = $derived(arrayAt(canonical, "limitations"));
  const warnings = $derived(arrayAt(canonical, "warnings"));
  const qualityFlags = $derived(arrayAt(canonical, "quality_flags").concat(arrayAt(canonical, "qualityFlags")));
  const sourceRefs = $derived(arrayAt(canonical, "source_refs"));

  onMount(() => {
    void loadResult();
  });

  async function loadResult() {
    if (!selectedRunId) return;
    loading = true;
    error = null;
    try {
      const [nextResult, nextFindings] = await Promise.all([
        getPromptPackResult(selectedRunId),
        getPromptPackValidationFindings(selectedRunId),
      ]);
      result = nextResult;
      findings = nextFindings;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  function recordAt(value: unknown, key: string): Record<string, unknown> {
    if (!value || typeof value !== "object" || Array.isArray(value)) return {};
    const next = (value as Record<string, unknown>)[key];
    return next && typeof next === "object" && !Array.isArray(next) ? (next as Record<string, unknown>) : {};
  }

  function arrayAt(value: unknown, key: string): Record<string, unknown>[] {
    if (!value || typeof value !== "object" || Array.isArray(value)) return [];
    const next = (value as Record<string, unknown>)[key];
    return Array.isArray(next) ? next.filter(isRecord) : [];
  }

  function isRecord(value: unknown): value is Record<string, unknown> {
    return Boolean(value && typeof value === "object" && !Array.isArray(value));
  }

  function textAt(value: Record<string, unknown>, key: string, fallback = "") {
    const next = value[key];
    return typeof next === "string" && next.length > 0 ? next : fallback;
  }
</script>

<section class="result-view" aria-label="YouTube Summary result">
  <div class="result-toolbar">
    <div>
      <span>YouTube Summary result</span>
      <strong>{selectedRunId ? `Run #${selectedRunId}` : "No run selected"}</strong>
    </div>
    <ExtractumButton variant="outline" disabled={!selectedRunId || loading} onclick={() => void loadResult()}>
      <RefreshCw size={14} aria-hidden="true" />
      Refresh
    </ExtractumButton>
  </div>

  {#if error}
    <div class="result-message error">
      <AlertTriangle size={15} aria-hidden="true" />
      {error}
    </div>
  {:else if !selectedRunId}
    <div class="result-message">Select a completed Prompt Pack run to inspect its report.</div>
  {:else if loading && !result}
    <div class="result-message">Loading result...</div>
  {:else if result}
    <div class="summary-meta">
      <span>Status: <strong>{result.resultStatus}</strong></span>
      <span>Pack: <strong>{textAt(canonical, "pack_version", run?.packVersion ?? "unknown")}</strong></span>
      <span>Sources: <strong>{sourceRefs.length}</strong></span>
      <span>Language: <strong>{textAt(canonical, "output_language", "unknown")}</strong></span>
    </div>

    <div class="result-section">
      <h3>Videos</h3>
      {#if videos.length === 0}
        <p class="muted">No video summaries were produced.</p>
      {:else}
        <ul>
          {#each videos as video, index (`video-${index}`)}
            <li>
              <strong>{textAt(video, "title", `Video ${index + 1}`)}</strong>
              <p>{textAt(video, "summary_text", "No summary text.")}</p>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <div class="result-section">
      <h3>Claims</h3>
      {#if claims.length === 0}
        <p class="muted">No claims extracted.</p>
      {:else}
        <ul>
          {#each claims as claim, index (`claim-${index}`)}
            <li>
              <strong>{textAt(claim, "claim_id", `claim_${index + 1}`)}</strong>
              <p>{textAt(claim, "text", "No claim text.")}</p>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <div class="result-section">
      <h3>Evidence</h3>
      {#if evidence.length === 0}
        <p class="muted">No evidence fragments extracted.</p>
      {:else}
        <ul>
          {#each evidence as item, index (`evidence-${index}`)}
            <li>
              <strong>{textAt(item, "evidence_id", `evidence_${index + 1}`)}</strong>
              <p>{textAt(item, "text", "No evidence text.")}</p>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <div class="result-section">
      <h3>Warnings and limitations</h3>
      {#if warnings.length === 0 && limitations.length === 0 && qualityFlags.length === 0}
        <p class="muted">No warnings, limitations, or quality flags.</p>
      {:else}
        <div class="badge-row">
          {#each warnings as warning, index (`warning-${index}`)}
            <ExtractumBadge>{textAt(warning, "message", textAt(warning, "code", "warning"))}</ExtractumBadge>
          {/each}
          {#each limitations as limitation, index (`limitation-${index}`)}
            <ExtractumBadge>{textAt(limitation, "message", textAt(limitation, "code", "limitation"))}</ExtractumBadge>
          {/each}
          {#each qualityFlags as flag, index (`quality-${index}`)}
            <ExtractumBadge>{textAt(flag, "message", textAt(flag, "code", "quality flag"))}</ExtractumBadge>
          {/each}
        </div>
      {/if}
    </div>

    <div class="result-section">
      <h3>Validation findings</h3>
      {#if findings.length === 0}
        <p class="muted">No validation findings.</p>
      {:else}
        <ul>
          {#each findings as finding (`finding-${finding.createdAt}-${finding.code}`)}
            <li>
              <strong>{finding.severity}: {finding.code}</strong>
              <p>{finding.message}</p>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {:else}
    <div class="result-message">
      <FileText size={15} aria-hidden="true" />
      Result is not available yet.
    </div>
  {/if}
</section>

<style>
  .result-view,
  .result-section,
  ul {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 10px;
  }

  .result-toolbar,
  .summary-meta,
  .badge-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }

  .result-toolbar {
    justify-content: space-between;
  }

  .result-toolbar div {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 2px;
  }

  .result-toolbar span,
  h3 {
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  h3,
  p,
  ul {
    margin: 0;
  }

  ul {
    padding: 0;
    list-style: none;
  }

  .summary-meta,
  .result-message,
  .result-section li {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    padding: 10px;
  }

  .summary-meta span {
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .summary-meta strong {
    color: var(--extractum-text);
  }

  .result-message {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--extractum-muted);
  }

  .result-message.error {
    color: var(--extractum-danger);
  }

  .result-section li {
    overflow-wrap: anywhere;
  }

  .result-section li p {
    margin-top: 5px;
    color: var(--extractum-muted);
    font-size: 13px;
    line-height: 1.45;
  }

  .muted {
    color: var(--extractum-muted);
    font-size: 13px;
  }
</style>
