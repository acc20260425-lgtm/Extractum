<script lang="ts">
  import type { ComponentProps } from "svelte";
  import type ReportCanvas from "$lib/components/analysis/report-canvas.svelte";

  type Props = Pick<
    ComponentProps<typeof ReportCanvas>,
    | "workspaceSelection"
    | "currentSource"
    | "currentGroup"
    | "exportDialogOpen"
    | "notebookLmExportForm"
    | "groupLiveTranscriptSegmentsBySource"
    | "highlightToken"
    | "onOpenNotebookLmExport"
    | "onChangeNotebookLmExportForm"
    | "onExportNotebookLm"
  >;

  let {
    workspaceSelection,
    currentSource,
    currentGroup,
    exportDialogOpen,
    notebookLmExportForm,
    groupLiveTranscriptSegmentsBySource,
    highlightToken,
    onOpenNotebookLmExport,
    onChangeNotebookLmExportForm,
    onExportNotebookLm,
  }: Props = $props();

  let receivedTarget = $state("");
  let receivedExportDialog = $state("");
  let receivedHighlightToken = $state("");
  let receivedGroupTranscripts = $state("");

  function readTarget() {
    receivedTarget = workspaceSelection.kind === "source"
      ? `source:${currentSource?.id ?? "missing"}`
      : workspaceSelection.kind === "source_group"
        ? `group:${currentGroup?.id ?? "missing"}:${currentGroup?.source_type ?? "missing"}`
        : "none";
  }

  function prepareExport() {
    onChangeNotebookLmExportForm({
      ...notebookLmExportForm,
      outputDir: " C:\\NotebookLM ",
      range: "entire_history",
      includeMediaPlaceholders: true,
      includeMigratedHistory: true,
      minMessageLength: 3,
      maxWordsPerFile: 300_000,
      maxBytesPerFile: 50_000_000,
      overwriteExisting: false,
    });
  }

  function readHighlightToken() {
    receivedHighlightToken = highlightToken
      ? `${highlightToken.traceRef}|${highlightToken.sourceViewBasis}|${highlightToken.sourceScope.kind}:${highlightToken.sourceScope.sourceId}`
      : "none";
  }

  function readGroupTranscripts() {
    receivedGroupTranscripts = Object.values(groupLiveTranscriptSegmentsBySource)
      .flat()
      .map((segment) => segment.text)
      .join("|") || "none";
  }
</script>

<section aria-label="Analysis route canvas receiver">
  <button aria-label="Read route target" onclick={readTarget}>Read target</button>
  <button aria-label="Open route NotebookLM export" onclick={onOpenNotebookLmExport}>Open export</button>
  <button aria-label="Prepare route NotebookLM export" onclick={prepareExport}>Prepare export</button>
  <button aria-label="Submit route NotebookLM export" onclick={onExportNotebookLm}>Submit export</button>
  <button
    aria-label="Read route export dialog"
    onclick={() => (receivedExportDialog = `open:${exportDialogOpen}`)}
  >
    Read export dialog
  </button>
  <button aria-label="Read route highlight token" onclick={readHighlightToken}>Read highlight</button>
  <button aria-label="Read route group transcripts" onclick={readGroupTranscripts}>Read transcripts</button>
  <output aria-label="Received route target">{receivedTarget}</output>
  <output aria-label="Received route export dialog">{receivedExportDialog}</output>
  <output aria-label="Received route highlight token">{receivedHighlightToken}</output>
  <output aria-label="Received route group transcripts">{receivedGroupTranscripts}</output>
</section>
