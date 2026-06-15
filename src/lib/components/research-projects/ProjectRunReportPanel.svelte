<script lang="ts">
  import { onMount } from "svelte";
  import { Braces, FileJson, Layers, RefreshCw } from "@lucide/svelte";
  import {
    getPromptPackResult,
    getPromptPackStageArtifact,
    getPromptPackValidationFindings,
    listPromptPackAuditEvents,
    listPromptPackRunStages,
    listPromptPackStageArtifacts,
  } from "$lib/api/prompt-packs";
  import { formatAppError } from "$lib/app-error";
  import { ExtractumBadge, ExtractumButton } from "$lib/components/extractum-ui";
  import type {
    PromptPackAuditEvent,
    PromptPackResult,
    PromptPackRunListItem,
    PromptPackStageArtifact,
    PromptPackStageArtifactSummary,
    PromptPackStageRun,
    PromptPackValidationFinding,
  } from "$lib/types/prompt-packs";

  let { run }: { run: PromptPackRunListItem | null } = $props();

  let result = $state<PromptPackResult | null>(null);
  let stages = $state<PromptPackStageRun[]>([]);
  let artifactsByStage = $state<Record<number, PromptPackStageArtifactSummary[]>>({});
  let selectedArtifact = $state<PromptPackStageArtifact | null>(null);
  let findings = $state<PromptPackValidationFinding[]>([]);
  let auditEvents = $state<PromptPackAuditEvent[]>([]);
  let loading = $state(false);
  let loadingArtifact = $state(false);
  let error = $state("");

  const runId = $derived(run?.runId ?? null);
  const canonical = $derived((result?.canonical ?? {}) as Record<string, unknown>);
  const youtubeSummary = $derived(recordAt(recordAt(recordAt(canonical, "outputs"), "pack_data"), "youtube_summary"));
  const summary = $derived(recordAt(recordAt(canonical, "outputs"), "summary"));
  const videos = $derived(arrayAt(youtubeSummary, "videos"));
  const segments = $derived(arrayAt(youtubeSummary, "segments"));
  const keyPoints = $derived(arrayAt(youtubeSummary, "key_points"));
  const quotes = $derived(arrayAt(youtubeSummary, "quotes"));
  const actionItems = $derived(arrayAt(youtubeSummary, "action_items"));
  const openQuestions = $derived(arrayAt(youtubeSummary, "open_questions"));
  const synthesisItems = $derived(arrayAt(youtubeSummary, "synthesis_items"));
  const claims = $derived(arrayAt(canonical, "claims"));
  const evidence = $derived(arrayAt(canonical, "evidence"));
  const sourceRefs = $derived(arrayAt(canonical, "source_refs"));
  const warnings = $derived(arrayAt(canonical, "warnings"));
  const limitations = $derived(arrayAt(canonical, "limitations"));
  const qualityFlags = $derived(arrayAt(canonical, "quality_flags").concat(arrayAt(canonical, "qualityFlags")));

  onMount(() => {
    void loadRunReport();
  });

  async function loadRunReport() {
    if (!runId) {
      clearReport();
      return;
    }

    loading = true;
    error = "";
    selectedArtifact = null;
    try {
      const [nextResult, nextStages, nextFindings, nextAuditEvents] = await Promise.all([
        getPromptPackResult(runId),
        listPromptPackRunStages(runId),
        getPromptPackValidationFindings(runId),
        listPromptPackAuditEvents(runId),
      ]);
      result = nextResult;
      stages = nextStages;
      findings = nextFindings;
      auditEvents = nextAuditEvents;
      const artifactPairs = await Promise.all(
        nextStages.map(async (stage) => [
          stage.stageRunId,
          await listPromptPackStageArtifacts(stage.stageRunId),
        ] as const),
      );
      artifactsByStage = Object.fromEntries(artifactPairs);
    } catch (cause) {
      clearReport(false);
      error = formatAppError("loading project run report", cause);
    } finally {
      loading = false;
    }
  }

  function clearReport(clearError = true) {
    result = null;
    stages = [];
    artifactsByStage = {};
    selectedArtifact = null;
    findings = [];
    auditEvents = [];
    if (clearError) error = "";
  }

  async function openArtifact(artifact: PromptPackStageArtifactSummary) {
    loadingArtifact = true;
    try {
      selectedArtifact = await getPromptPackStageArtifact({
        stageRunId: artifact.stageRunId,
        artifactKind: artifact.artifactKind,
        attemptNumber: artifact.attemptNumber,
        artifactIndex: artifact.artifactIndex,
      });
    } catch (cause) {
      error = formatAppError("loading project run artifact", cause);
    } finally {
      loadingArtifact = false;
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
    return typeof next === "string" && next.trim() ? next : fallback;
  }

  function numberAt(value: Record<string, unknown>, key: string) {
    const next = value[key];
    return typeof next === "number" ? next : null;
  }

  function formatSeconds(value: number | null) {
    if (value === null) return "";
    const minutes = Math.floor(value / 60);
    const seconds = Math.floor(value % 60).toString().padStart(2, "0");
    return `${minutes}:${seconds}`;
  }

  function jsonPreview(value: unknown) {
    return JSON.stringify(value, null, 2);
  }
</script>

<section class="project-run-report" aria-label="Project run report">
  <header class="report-header">
    <div>
      <span>Report workspace</span>
      <h2>{run ? (run.runLabel?.trim() || `Run #${run.runId}`) : "No run selected"}</h2>
    </div>
    <div class="report-actions">
      {#if run}
        <ExtractumBadge>{run.runStatus}</ExtractumBadge>
        <ExtractumBadge>{run.resultStatus ?? "none"}</ExtractumBadge>
      {/if}
      <ExtractumButton variant="outline" disabled={!runId || loading} onclick={() => void loadRunReport()}>
        <RefreshCw size={14} aria-hidden="true" />
        Refresh
      </ExtractumButton>
    </div>
  </header>

  {#if error}
    <div class="report-message error">{error}</div>
  {:else if !run}
    <div class="report-message">Select a project run in the grid.</div>
  {:else if loading && !result}
    <div class="report-message">Loading project run report...</div>
  {:else}
    <div class="report-layout">
      <section class="report-column primary">
        <div class="report-section overview-section">
          <div class="section-title">
            <FileJson size={15} aria-hidden="true" />
            <h3>Result</h3>
          </div>
          <div class="metric-strip">
            <span>Sources <strong>{sourceRefs.length}</strong></span>
            <span>Videos <strong>{videos.length}</strong></span>
            <span>Claims <strong>{claims.length}</strong></span>
            <span>Evidence <strong>{evidence.length}</strong></span>
            <span>Findings <strong>{findings.length}</strong></span>
          </div>
          {#if textAt(summary, "summary_text")}
            <p class="summary-text">{textAt(summary, "summary_text")}</p>
          {/if}
        </div>

        <div class="report-section">
          <h3>Videos</h3>
          {#if videos.length === 0}
            <p class="muted">No video summaries.</p>
          {:else}
            <div class="item-list">
              {#each videos as video, index (`video-${index}`)}
                <article>
                  <strong>{textAt(video, "title", `Video ${index + 1}`)}</strong>
                  <p>{textAt(video, "summary_text", "No summary text.")}</p>
                </article>
              {/each}
            </div>
          {/if}
        </div>

        <div class="report-section two-column">
          <div>
            <h3>Key Points</h3>
            {@render CompactList({ items: keyPoints, textKey: "text", fallbackPrefix: "Key point" })}
          </div>
          <div>
            <h3>Quotes</h3>
            {@render CompactList({ items: quotes, textKey: "text", fallbackPrefix: "Quote" })}
          </div>
        </div>

        <div class="report-section two-column">
          <div>
            <h3>Claims</h3>
            {@render CompactList({ items: claims, textKey: "text", fallbackPrefix: "Claim" })}
          </div>
          <div>
            <h3>Evidence</h3>
            {@render CompactList({ items: evidence, textKey: "text", fallbackPrefix: "Evidence" })}
          </div>
        </div>

        <div class="report-section two-column">
          <div>
            <h3>Action Items</h3>
            {@render CompactList({ items: actionItems, textKey: "text", fallbackPrefix: "Action" })}
          </div>
          <div>
            <h3>Open Questions</h3>
            {@render CompactList({ items: openQuestions, textKey: "text", fallbackPrefix: "Question" })}
          </div>
        </div>

        <div class="report-section">
          <h3>Synthesis</h3>
          {@render CompactList({ items: synthesisItems, textKey: "text", fallbackPrefix: "Synthesis" })}
        </div>

        <div class="report-section">
          <h3>Timeline Segments</h3>
          {#if segments.length === 0}
            <p class="muted">No timeline segments.</p>
          {:else}
            <div class="segment-list">
              {#each segments as segment, index (`segment-${index}`)}
                <article>
                  <strong>
                    {formatSeconds(numberAt(segment, "start_seconds"))}
                    {#if numberAt(segment, "end_seconds") !== null}
                      - {formatSeconds(numberAt(segment, "end_seconds"))}
                    {/if}
                  </strong>
                  <p>{textAt(segment, "text", "No segment text.")}</p>
                </article>
              {/each}
            </div>
          {/if}
        </div>
      </section>

      <aside class="report-column secondary">
        <div class="report-section">
          <div class="section-title">
            <Layers size={15} aria-hidden="true" />
            <h3>Stages and Artifacts</h3>
          </div>
          {#if stages.length === 0}
            <p class="muted">No stage runs.</p>
          {:else}
            <div class="stage-list">
              {#each stages as stage (stage.stageRunId)}
                <article>
                  <div class="stage-title">
                    <strong>{stage.stageName}</strong>
                    <ExtractumBadge>{stage.stageStatus}</ExtractumBadge>
                  </div>
                  {#if stage.latestMessage}
                    <p>{stage.latestMessage}</p>
                  {/if}
                  <div class="artifact-list">
                    {#each artifactsByStage[stage.stageRunId] ?? [] as artifact (`${artifact.artifactKind}-${artifact.attemptNumber}-${artifact.artifactIndex}`)}
                      <button type="button" disabled={loadingArtifact} onclick={() => void openArtifact(artifact)}>
                        {artifact.artifactKind} #{artifact.artifactIndex}
                      </button>
                    {/each}
                  </div>
                </article>
              {/each}
            </div>
          {/if}
        </div>

        <div class="report-section">
          <h3>Selected Artifact</h3>
          {#if selectedArtifact}
            <pre>{jsonPreview(selectedArtifact.content)}</pre>
          {:else}
            <p class="muted">Select a stage artifact.</p>
          {/if}
        </div>

        <div class="report-section">
          <h3>Warnings</h3>
          {@render CompactList({
            items: warnings.concat(limitations).concat(qualityFlags),
            textKey: "message",
            fallbackPrefix: "Warning",
          })}
        </div>

        <div class="report-section">
          <h3>Validation Findings</h3>
          {#if findings.length === 0}
            <p class="muted">No validation findings.</p>
          {:else}
            <div class="item-list">
              {#each findings as finding (`${finding.createdAt}-${finding.code}`)}
                <article>
                  <strong>{finding.severity}: {finding.code}</strong>
                  <p>{finding.message}</p>
                </article>
              {/each}
            </div>
          {/if}
        </div>

        <div class="report-section">
          <h3>Audit Events</h3>
          {#if auditEvents.length === 0}
            <p class="muted">No audit events.</p>
          {:else}
            <div class="item-list">
              {#each auditEvents as event (`${event.createdAt}-${event.eventKind}`)}
                <article>
                  <strong>{event.eventKind}</strong>
                  <p>{event.message ?? event.createdAt}</p>
                </article>
              {/each}
            </div>
          {/if}
        </div>

        <div class="report-section">
          <div class="section-title">
            <Braces size={15} aria-hidden="true" />
            <h3>Canonical JSON</h3>
          </div>
          {#if result}
            <pre>{jsonPreview(canonical)}</pre>
          {:else}
            <p class="muted">No canonical result.</p>
          {/if}
        </div>
      </aside>
    </div>
  {/if}
</section>

{#snippet CompactList({ items, textKey, fallbackPrefix }: { items: Record<string, unknown>[]; textKey: string; fallbackPrefix: string })}
  {#if items.length === 0}
    <p class="muted">None.</p>
  {:else}
    <div class="item-list compact">
      {#each items as item, index (`${fallbackPrefix}-${index}`)}
        <article>
          <p>{textAt(item, textKey, `${fallbackPrefix} ${index + 1}`)}</p>
        </article>
      {/each}
    </div>
  {/if}
{/snippet}

<style>
  .project-run-report {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: hidden;
  }

  .report-header,
  .report-actions,
  .section-title,
  .stage-title,
  .metric-strip {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .report-header {
    justify-content: space-between;
  }

  .report-header span,
  .report-section h3 {
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  .report-header h2,
  .report-section h3,
  p {
    margin: 0;
  }

  .report-header h2 {
    margin-top: 2px;
    font-size: 18px;
    letter-spacing: 0;
  }

  .report-layout {
    display: grid;
    grid-template-columns: minmax(0, 1.45fr) minmax(320px, 0.85fr);
    min-width: 0;
    min-height: 0;
    gap: 14px;
    overflow: hidden;
  }

  .report-column {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: auto;
  }

  .report-section,
  .report-message {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    padding: 12px;
  }

  .report-section {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 10px;
  }

  .overview-section {
    background: var(--extractum-surface-subtle);
  }

  .metric-strip {
    flex-wrap: wrap;
  }

  .metric-strip span {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    padding: 5px 8px;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .metric-strip strong {
    color: var(--extractum-text);
  }

  .summary-text,
  .muted {
    color: var(--extractum-muted);
    font-size: 13px;
    line-height: 1.5;
  }

  .item-list,
  .stage-list,
  .segment-list {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 8px;
  }

  .item-list article,
  .stage-list article,
  .segment-list article {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    padding: 10px;
  }

  .item-list p,
  .stage-list p,
  .segment-list p {
    margin-top: 5px;
    color: var(--extractum-muted);
    font-size: 13px;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .item-list.compact article p {
    margin-top: 0;
  }

  .two-column {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }

  .two-column > div {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 8px;
  }

  .artifact-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .artifact-list button {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-subtle);
    padding: 4px 7px;
    color: var(--extractum-text);
    font-size: 12px;
  }

  pre {
    max-height: 360px;
    overflow: auto;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    padding: 10px;
    font-size: 12px;
    line-height: 1.45;
    white-space: pre-wrap;
  }

  .report-message {
    color: var(--extractum-muted);
  }

  .report-message.error {
    color: var(--extractum-danger);
  }

  @media (max-width: 1180px) {
    .report-layout {
      grid-template-columns: 1fr;
      overflow: visible;
    }

    .report-column {
      overflow: visible;
    }
  }

  @media (max-width: 760px) {
    .report-header {
      align-items: stretch;
      flex-direction: column;
    }

    .report-actions {
      flex-wrap: wrap;
    }

    .two-column {
      grid-template-columns: 1fr;
    }
  }
</style>
