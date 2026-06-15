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
  let resultError = $state("");
  let selectedRef = $state<string | null>(null);
  let artifactCopied = $state(false);

  const runId = $derived(run?.runId ?? null);
  const expectedMissingResult = $derived(
    Boolean(run && !result && (run.resultStatus ?? "none") !== "complete" && canLackCanonicalResult(run)),
  );
  const resultUnavailableMessage = $derived(describeMissingResult(run));
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
  const synthesis = $derived(recordAt(youtubeSummary, "synthesis"));
  const hasCanonicalSynthesis = $derived(Object.keys(synthesis).length > 0);
  const crossVideoThemes = $derived(arrayAt(synthesis, "cross_video_themes"));
  const commonClaims = $derived(arrayAt(synthesis, "common_claims"));
  const contradictions = $derived(arrayAt(synthesis, "contradictions_across_videos"));
  const synthesisSourceRefs = $derived(stringArrayAt(synthesis, "source_refs"));
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
    resultError = "";
    selectedArtifact = null;
    selectedRef = null;
    artifactCopied = false;
    try {
      await Promise.all([loadRunResult(runId), loadRunDiagnostics(runId)]);
    } catch (cause) {
      clearDiagnostics();
      error = formatAppError("loading project run diagnostics", cause);
    } finally {
      loading = false;
    }
  }

  async function loadRunResult(nextRunId: number) {
    try {
      result = await getPromptPackResult(nextRunId);
    } catch (cause) {
      result = null;
      if (!canLackCanonicalResult(run)) {
        resultError = formatAppError("loading project run report", cause);
      }
    }
  }

  async function loadRunDiagnostics(nextRunId: number) {
    const [nextStages, nextFindings, nextAuditEvents] = await Promise.all([
      listPromptPackRunStages(nextRunId),
      getPromptPackValidationFindings(nextRunId),
      listPromptPackAuditEvents(nextRunId),
    ]);
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
  }

  function clearReport(clearError = true) {
    result = null;
    clearDiagnostics();
    if (clearError) {
      error = "";
      resultError = "";
    }
    selectedRef = null;
  }

  function clearDiagnostics() {
    stages = [];
    artifactsByStage = {};
    selectedArtifact = null;
    artifactCopied = false;
    findings = [];
    auditEvents = [];
  }

  async function openArtifact(artifact: PromptPackStageArtifactSummary) {
    loadingArtifact = true;
    artifactCopied = false;
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

  async function copySelectedArtifactJson() {
    if (!selectedArtifact) return;
    try {
      await navigator.clipboard.writeText(jsonPreview(selectedArtifact));
      artifactCopied = true;
      setTimeout(() => {
        artifactCopied = false;
      }, 1400);
    } catch (cause) {
      error = formatAppError("copying project run artifact", cause);
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

  function stringArrayAt(value: unknown, key: string): string[] {
    if (!value || typeof value !== "object" || Array.isArray(value)) return [];
    const next = (value as Record<string, unknown>)[key];
    return Array.isArray(next) ? next.filter((item): item is string => typeof item === "string" && item.trim().length > 0) : [];
  }

  function isRecord(value: unknown): value is Record<string, unknown> {
    return Boolean(value && typeof value === "object" && !Array.isArray(value));
  }

  function textAt(value: Record<string, unknown>, key: string, fallback = "") {
    const next = value[key];
    return typeof next === "string" && next.trim() ? next : fallback;
  }

  function stringAt(value: Record<string, unknown>, key: string) {
    const next = value[key];
    return typeof next === "string" && next.trim() ? next : "";
  }

  function numberAt(value: Record<string, unknown>, key: string) {
    const next = value[key];
    return typeof next === "number" ? next : null;
  }

  function artifactTitle(artifactKind: string) {
    switch (artifactKind) {
      case "prompt_input":
        return "Prompt input";
      case "raw_output":
        return "Raw output";
      case "parsed_output":
        return "Parsed output";
      case "metrics":
        return "Metrics";
      default:
        return artifactKind;
    }
  }

  function artifactPreview(value: unknown) {
    if (typeof value === "string") return value;
    return jsonPreview(value);
  }

  function textFromKeys(value: Record<string, unknown>, keys: string[], fallback = "") {
    for (const key of keys) {
      const next = textAt(value, key);
      if (next) return next;
    }
    return fallback;
  }

  function refsForItem(value: Record<string, unknown>) {
    return uniqueStrings([
      ...stringArrayAt(value, "video_refs"),
      ...stringArrayAt(value, "source_refs"),
      ...stringArrayAt(value, "claim_refs"),
      ...stringArrayAt(value, "evidence_refs"),
      ...stringArrayAt(value, "relation_refs"),
    ]);
  }

  function refTargetsForItem(value: Record<string, unknown>, identityKeys: string[] = []) {
    return uniqueStrings([...identityKeys.map((key) => stringAt(value, key)).filter(Boolean), ...refsForItem(value)]);
  }

  function refTargetAttr(value: Record<string, unknown>, identityKeys: string[] = []) {
    return refTargetsForItem(value, identityKeys).join(" ");
  }

  function matchesSelectedRef(value: Record<string, unknown>, identityKeys: string[] = []) {
    return selectedRef !== null && refTargetsForItem(value, identityKeys).includes(selectedRef);
  }

  function toggleSelectedRef(refId: string) {
    selectedRef = selectedRef === refId ? null : refId;
    if (selectedRef) {
      setTimeout(() => scrollMatchingRefIntoView(selectedRef), 0);
    }
  }

  function scrollMatchingRefIntoView(refId: string | null) {
    if (!refId) return;
    const target = Array.from(document.querySelectorAll<HTMLElement>("[data-ref-targets]")).find((element) =>
      (element.dataset.refTargets ?? "").split(" ").includes(refId),
    );
    target?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }

  function uniqueStrings(values: string[]) {
    const result: string[] = [];
    for (const value of values) {
      if (!result.includes(value)) result.push(value);
    }
    return result;
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

  function canLackCanonicalResult(nextRun: PromptPackRunListItem | null) {
    if (!nextRun) return false;
    return ["queued", "running", "failed", "cancelled", "interrupted"].includes(nextRun.runStatus);
  }

  function describeMissingResult(nextRun: PromptPackRunListItem | null) {
    if (!nextRun) return "";
    if (nextRun.runStatus === "cancelled") {
      return "Run was cancelled before producing a canonical result.";
    }
    if (nextRun.runStatus === "failed") {
      return "Run failed before producing a canonical result.";
    }
    if (nextRun.runStatus === "running") {
      return "Run is still in progress. A canonical result is not available yet.";
    }
    if (nextRun.runStatus === "queued") {
      return "Run is queued. A canonical result is not available yet.";
    }
    if (nextRun.runStatus === "interrupted") {
      return "Run was interrupted before producing a canonical result.";
    }
    return "No canonical result is available for this run.";
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
          {#if resultError}
            <p class="result-state error">{resultError}</p>
          {:else if expectedMissingResult}
            <p class="result-state">{resultUnavailableMessage}</p>
          {:else if textAt(summary, "summary_text")}
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
                <article
                  data-ref-targets={refTargetAttr(video, ["video_id", "source_ref_id"])}
                  class:ref-target={matchesSelectedRef(video, ["video_id", "source_ref_id"])}
                >
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
            {@render CompactList({
              items: claims,
              textKey: "text",
              fallbackPrefix: "Claim",
              refKeys: ["claim_id", "source_ref_id"],
            })}
          </div>
          <div>
            <h3>Evidence</h3>
            {@render CompactList({
              items: evidence,
              textKey: "text",
              fallbackPrefix: "Evidence",
              refKeys: ["evidence_id", "source_ref_id"],
            })}
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
          {#if hasCanonicalSynthesis}
            <div class="metric-strip">
              <span>Themes <strong>{crossVideoThemes.length}</strong></span>
              <span>Claims <strong>{commonClaims.length}</strong></span>
              <span>Contradictions <strong>{contradictions.length}</strong></span>
            </div>
            {#if selectedRef}
              <div class="selected-ref-bar">
                <span>Selected ref <strong>{selectedRef}</strong></span>
                <button type="button" onclick={() => (selectedRef = null)}>Clear</button>
              </div>
            {/if}
            <div class="synthesis-layout">
              <div class="synthesis-main">
                {@render SynthesisGroup({
                  title: "Cross-video themes",
                  items: crossVideoThemes,
                  textKeys: ["theme_text", "text"],
                  fallbackPrefix: "Theme",
                })}
                {@render SynthesisGroup({
                  title: "Common claims",
                  items: commonClaims,
                  textKeys: ["summary_text", "text"],
                  fallbackPrefix: "Claim",
                })}
                {@render SynthesisGroup({
                  title: "Contradictions",
                  items: contradictions,
                  textKeys: ["description", "text"],
                  fallbackPrefix: "Contradiction",
                })}
              </div>
              <aside class="synthesis-ref-rail" aria-label="Synthesis references">
                <h4>Refs</h4>
                {#if synthesisSourceRefs.length === 0}
                  <p class="muted">None.</p>
                {:else}
                  <div class="ref-chip-list">
                    {#each synthesisSourceRefs as sourceRef (`synthesis-ref-${sourceRef}`)}
                      {@render RefButton({ refId: sourceRef })}
                    {/each}
                  </div>
                {/if}
              </aside>
            </div>
          {:else}
            {@render CompactList({ items: synthesisItems, textKey: "text", fallbackPrefix: "Synthesis" })}
          {/if}
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
            {@render ArtifactDetail({ artifact: selectedArtifact })}
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
          {:else if expectedMissingResult}
            <p class="muted">{resultUnavailableMessage}</p>
          {:else}
            <p class="muted">No canonical result.</p>
          {/if}
        </div>
      </aside>
    </div>
  {/if}
</section>

{#snippet CompactList({
  items,
  textKey,
  fallbackPrefix,
  refKeys = [],
}: {
  items: Record<string, unknown>[];
  textKey: string;
  fallbackPrefix: string;
  refKeys?: string[];
})}
  {#if items.length === 0}
    <p class="muted">None.</p>
  {:else}
    <div class="item-list compact">
      {#each items as item, index (`${fallbackPrefix}-${index}`)}
        <article data-ref-targets={refTargetAttr(item, refKeys)} class:ref-target={matchesSelectedRef(item, refKeys)}>
          <p>{textAt(item, textKey, `${fallbackPrefix} ${index + 1}`)}</p>
        </article>
      {/each}
    </div>
  {/if}
{/snippet}

{#snippet ArtifactDetail({ artifact }: { artifact: PromptPackStageArtifact })}
  <div class="artifact-detail">
    <div class="artifact-detail-header">
      <div>
        <h4>{artifactTitle(artifact.artifactKind)}</h4>
        <p>
          stage #{artifact.stageRunId} - attempt {artifact.attemptNumber} - item {artifact.artifactIndex}
        </p>
      </div>
      <button type="button" onclick={() => void copySelectedArtifactJson()}>
        {artifactCopied ? "Copied" : "Copy JSON"}
      </button>
    </div>

    <div class="artifact-meta">
      <span>{artifact.artifactKind}</span>
      <span>{artifact.contentType}</span>
      <span>{artifact.createdAt}</span>
    </div>

    <div class="artifact-preview">
      <h4>{artifactTitle(artifact.artifactKind)}</h4>
      <pre>{artifactPreview(artifact.content)}</pre>
    </div>

    <details class="artifact-json">
      <summary>Full JSON</summary>
      <pre>{jsonPreview(artifact)}</pre>
    </details>
  </div>
{/snippet}

{#snippet SynthesisGroup({
  title,
  items,
  textKeys,
  fallbackPrefix,
}: {
  title: string;
  items: Record<string, unknown>[];
  textKeys: string[];
  fallbackPrefix: string;
})}
  <section class="synthesis-group">
    <h4>{title}</h4>
    {#if items.length === 0}
      <p class="muted">None.</p>
    {:else}
      <div class="synthesis-items">
        {#each items as item, index (`${fallbackPrefix}-${index}`)}
          {@const refs = refsForItem(item)}
          <article data-ref-targets={refTargetAttr(item)} class:ref-target={matchesSelectedRef(item)}>
            <p>{textFromKeys(item, textKeys, `${fallbackPrefix} ${index + 1}`)}</p>
            {#if refs.length > 0}
              <div class="synthesis-item-refs">
                {#each refs as refId (`${fallbackPrefix}-${index}-${refId}`)}
                  {@render RefButton({ refId })}
                {/each}
              </div>
            {/if}
          </article>
        {/each}
      </div>
    {/if}
  </section>
{/snippet}

{#snippet RefButton({ refId }: { refId: string })}
  <button
    type="button"
    class="ref-chip"
    class:ref-selected={selectedRef === refId}
    aria-pressed={selectedRef === refId}
    onclick={() => toggleSelectedRef(refId)}
  >
    {refId}
  </button>
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
  .result-state,
  .muted {
    color: var(--extractum-muted);
    font-size: 13px;
    line-height: 1.5;
  }

  .result-state.error {
    color: var(--extractum-danger);
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

  .synthesis-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(150px, 0.32fr);
    gap: 12px;
  }

  .synthesis-main,
  .synthesis-group,
  .synthesis-items,
  .synthesis-ref-rail,
  .ref-chip-list {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 8px;
  }

  .synthesis-group h4,
  .synthesis-ref-rail h4 {
    margin: 0;
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  .synthesis-items article,
  .synthesis-ref-rail {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    padding: 10px;
  }

  .synthesis-items article p {
    color: var(--extractum-text);
    font-size: 13px;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .synthesis-item-refs,
  .ref-chip-list {
    flex-flow: row wrap;
    gap: 6px;
  }

  .synthesis-item-refs {
    margin-top: 8px;
  }

  .selected-ref-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border: 1px solid color-mix(in srgb, var(--extractum-primary) 28%, var(--extractum-border));
    border-radius: var(--extractum-radius);
    background: color-mix(in srgb, var(--extractum-primary) 8%, var(--extractum-surface));
    padding: 7px 9px;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .selected-ref-bar strong {
    color: var(--extractum-text);
  }

  .selected-ref-bar button,
  .ref-chip {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-subtle);
    cursor: pointer;
  }

  .selected-ref-bar button {
    padding: 4px 7px;
    color: var(--extractum-text);
    font-size: 12px;
  }

  .ref-chip {
    padding: 3px 6px;
    color: var(--extractum-muted);
    font-size: 11px;
    line-height: 1.2;
    overflow-wrap: anywhere;
    text-align: left;
  }

  .selected-ref-bar button:hover,
  .ref-chip:hover,
  .ref-chip.ref-selected {
    border-color: color-mix(in srgb, var(--extractum-primary) 50%, var(--extractum-border));
    background: color-mix(in srgb, var(--extractum-primary) 12%, var(--extractum-surface));
    color: var(--extractum-primary);
  }

  .ref-target {
    border-color: color-mix(in srgb, var(--extractum-primary) 55%, var(--extractum-border)) !important;
    background: color-mix(in srgb, var(--extractum-primary) 7%, var(--extractum-surface)) !important;
    box-shadow: inset 3px 0 0 color-mix(in srgb, var(--extractum-primary) 72%, transparent);
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

  .artifact-detail,
  .artifact-preview,
  .artifact-json {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 8px;
  }

  .artifact-detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .artifact-detail-header h4,
  .artifact-preview h4 {
    margin: 0;
    font-size: 13px;
    letter-spacing: 0;
  }

  .artifact-detail-header p {
    margin-top: 3px;
    color: var(--extractum-muted);
    font-size: 12px;
    line-height: 1.35;
  }

  .artifact-detail-header button {
    flex: 0 0 auto;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-subtle);
    padding: 5px 8px;
    color: var(--extractum-text);
    cursor: pointer;
    font-size: 12px;
  }

  .artifact-detail-header button:hover {
    border-color: color-mix(in srgb, var(--extractum-primary) 45%, var(--extractum-border));
    color: var(--extractum-primary);
  }

  .artifact-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .artifact-meta span {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-subtle);
    padding: 4px 6px;
    color: var(--extractum-muted);
    font-size: 11px;
    line-height: 1.2;
    overflow-wrap: anywhere;
  }

  .artifact-preview {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    padding: 10px;
  }

  .artifact-preview pre,
  .artifact-json pre {
    max-height: 260px;
    margin: 0;
  }

  .artifact-json summary {
    color: var(--extractum-muted);
    cursor: pointer;
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

    .synthesis-layout {
      grid-template-columns: 1fr;
    }
  }
</style>
