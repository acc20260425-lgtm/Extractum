<script lang="ts">
  import { PlayCircle } from "@lucide/svelte";
  import {
    ExtractumCheckbox,
    ExtractumButton,
    ExtractumDialog,
    ExtractumStatusMessage,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import {
    preflightYoutubeSummaryRun,
    startYoutubeSummaryRun,
  } from "$lib/api/prompt-packs";
  import { canStartYoutubeSummary, summarizePreflightPartitions } from "$lib/ui/youtube-summary-workflow";
  import type { YoutubeSummaryPreflightResponse } from "$lib/types/prompt-packs";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";

  let {
    open = $bindable(false),
    source,
    onStarted,
  }: {
    open?: boolean;
    source: LibraryCatalogSourceView | null;
    onStarted?: (runId: number) => void;
  } = $props();

  let outputLanguage = $state("en");
  let profileId = $state("");
  let modelOverride = $state("");
  let includeComments = $state(false);
  let loading = $state(false);
  let preflight = $state<YoutubeSummaryPreflightResponse | null>(null);
  let error = $state<string | null>(null);

  let partitionSummary = $derived(
    preflight ? summarizePreflightPartitions(preflight) : null,
  );

  $effect(() => {
    if (open && source) void runPreflight();
  });

  async function runPreflight() {
    if (!source) return;
    loading = true;
    error = null;
    try {
      preflight = await preflightYoutubeSummaryRun({
        projectId: null,
        sourceIds: [source.sourceId],
        profileId: profileId || null,
        modelOverride: modelOverride || null,
        outputLanguage,
        controlPreset: "standard",
        evidenceMode: "standard",
        includeComments,
      });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  async function startRun() {
    if (!source || !canStartYoutubeSummary(preflight)) return;
    loading = true;
    error = null;
    const clientRequestId = `youtube-summary-${source.sourceId}-${Date.now()}`;
    try {
      const outcome = await startYoutubeSummaryRun({
        clientRequestId,
        projectId: null,
        sourceIds: [source.sourceId],
        profileId: profileId || null,
        modelOverride: modelOverride || null,
        outputLanguage,
        controlPreset: "standard",
        evidenceMode: "standard",
        includeComments,
      });
      if (outcome.kind === "blocked") {
        preflight = outcome.preflight;
        return;
      }
      onStarted?.(outcome.run.runId);
      open = false;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }
</script>

<ExtractumDialog bind:open title="YouTube Summary">
  <form class="youtube-summary-dialog" onsubmit={(event) => { event.preventDefault(); void startRun(); }}>
    <header>
      <div>
        <p class="eyebrow">Prompt Pack</p>
        <h2>{source?.title ?? "No source selected"}</h2>
      </div>
      <PlayCircle size={18} aria-hidden="true" />
    </header>

    {#if error}
      <ExtractumStatusMessage tone="error">{error}</ExtractumStatusMessage>
    {/if}

    <label><span>Output language</span><ExtractumTextInput bind:value={outputLanguage} /></label>
    <label><span>LLM profile</span><ExtractumTextInput bind:value={profileId} placeholder="Default" /></label>
    <label><span>Model override</span><ExtractumTextInput bind:value={modelOverride} placeholder="Optional" /></label>
    <label class="checkbox-row">
      <ExtractumCheckbox bind:checked={includeComments} onchange={() => void runPreflight()} />
      <span>Include comments</span>
    </label>

    <section aria-label="Preflight">
      <h3>Preflight</h3>
      {#if loading && !preflight}
        <p>Checking source readiness...</p>
      {:else if partitionSummary}
        <dl>
          <div><dt>Ready videos</dt><dd>{partitionSummary.includedCount}</dd></div>
          <div><dt>Skipped</dt><dd>{partitionSummary.skippedCount}</dd></div>
          <div><dt>Blocking</dt><dd>{partitionSummary.blockingCount}</dd></div>
        </dl>
        {#if partitionSummary.hasPartialCoverage}
          <ExtractumStatusMessage tone="info">Some playlist videos will be skipped.</ExtractumStatusMessage>
        {/if}
        {#if preflight?.blockingFailures.length}
          <ExtractumStatusMessage tone="error">
            {preflight.blockingFailures.map((failure) => failure.reason).join(", ")}
          </ExtractumStatusMessage>
        {/if}
      {:else}
        <p>Open the dialog to check source readiness.</p>
      {/if}
    </section>

    <footer>
      <ExtractumButton type="button" variant="outline" onclick={() => void runPreflight()} disabled={!source || loading}>Refresh</ExtractumButton>
      <ExtractumButton type="submit" disabled={!source || loading || !canStartYoutubeSummary(preflight)}>Start</ExtractumButton>
    </footer>
  </form>
</ExtractumDialog>

<style>
  .youtube-summary-dialog {
    display: grid;
    min-width: min(560px, calc(100vw - 96px));
    gap: 12px;
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .eyebrow {
    margin: 0 0 4px;
    color: var(--extractum-muted);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
  }

  h2,
  h3,
  p {
    margin: 0;
  }

  h2 {
    font-size: 16px;
    line-height: 1.25;
  }

  h3 {
    font-size: 13px;
  }

  label {
    display: grid;
    gap: 6px;
  }

  label span,
  p,
  dt {
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  section,
  dl {
    display: grid;
    gap: 8px;
  }

  dl {
    grid-template-columns: repeat(3, minmax(0, 1fr));
    margin: 0;
  }

  dd {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
