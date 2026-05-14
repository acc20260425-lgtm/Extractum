import { describe, expect, it } from "vitest";
import companionTabsSource from "./components/analysis/run-companion-tabs.svelte?raw";
import chunkSummariesSource from "./components/analysis/chunk-summaries.svelte?raw";
import evidenceTabSource from "./components/analysis/run-evidence-tab.svelte?raw";
import chatTabSource from "./components/analysis/run-chat-tab.svelte?raw";
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";

describe("run companion tabs", () => {
  it("uses accessible Evidence, Chat, Chunks, and Runs tabs", () => {
    expect(companionTabsSource).toContain('class="run-companion-tabs"');
    expect(companionTabsSource).toContain('role="tablist"');
    expect(companionTabsSource).toContain('aria-label="Run companion tabs"');
    expect(companionTabsSource).toContain('onChangeCompanionTab("evidence")');
    expect(companionTabsSource).toContain('onChangeCompanionTab("chat")');
    expect(companionTabsSource).toContain('onChangeCompanionTab("chunks")');
    expect(companionTabsSource).toContain('onChangeCompanionTab("runs")');
    expect(companionTabsSource).toContain("<RunEvidenceTab");
    expect(companionTabsSource).toContain("<RunChatTab");
    expect(companionTabsSource).toContain("<ChunkSummaries");
    expect(companionTabsSource).toContain("<RunCompanionRunsTab");
    expect(companionTabsSource).toContain("chunkTabLabel");
    expect(companionTabsSource).toContain("chunksDisabled");
  });

  it("renders chunk summaries compactly inside the companion", () => {
    expect(companionTabsSource).toContain("focusedChunkSummaries");
    expect(companionTabsSource).toContain("selectedRunIsActive");
    expect(companionTabsSource).toContain("framed={false}");
    expect(chunkSummariesSource).toContain("framed = true");
    expect(chunkSummariesSource).toContain("terminalEmptyMessage");
    expect(chunkSummariesSource).toContain("Waiting for the first chunk summary.");
    expect(chunkSummariesSource).toContain(
      "Chunk summaries are only available while the run is streaming.",
    );
    expect(chunkSummariesSource).toContain("class:card={framed}");
  });

  it("keeps Evidence focused on trace refs and Show in source", () => {
    expect(evidenceTabSource).toContain("<TracePanel");
    expect(evidenceTabSource).toContain("Show in source");
    expect(evidenceTabSource).toContain("onShowSelectedTraceInSource");
    expect(evidenceTabSource).toContain("evidenceSourceActionDecision");
    expect(evidenceTabSource).toContain("Snapshot unavailable");
  });

  it("keeps Chat explicit and availability-gated", () => {
    expect(chatTabSource).toContain("<ChatPanel");
    expect(chatTabSource).toContain("chatAvailability");
    expect(chatTabSource).not.toContain("onfocus");
    expect(chatTabSource).not.toContain("onFocus");
  });

  it("contains only analysis report runs in the Runs tab", () => {
    expect(runsTabSource).toContain("filterCompanionRuns");
    expect(runsTabSource).toContain("target instanceof HTMLInputElement");
    expect(runsTabSource).toContain("queued/running");
    expect(runsTabSource).toContain("Search runs");
    expect(runsTabSource).toContain("Current scope");
    expect(runsTabSource).toContain("Date range");
    expect(runsTabSource).toContain("Provider filter");
    expect(runsTabSource).toContain("Template filter");
    expect(runsTabSource).not.toContain("SourceJobRecord");
    expect(runsTabSource).not.toContain("takeoutJobs");
    expect(runsTabSource).not.toContain("sourceJobs");
  });

  it("keeps dense run filters behind an advanced filters disclosure", () => {
    expect(runsTabSource).toContain("<summary>Advanced filters</summary>");
    expect(runsTabSource).toContain('class="advanced-filters"');
    expect(runsTabSource).toContain('ariaLabel="Runs from date"');
    expect(runsTabSource).toContain('ariaLabel="Provider filter"');
  });

  it("offers a clear path when restored run filters hide all runs", () => {
    expect(runsTabSource).toContain("hasActiveCompanionRunsFilter");
    expect(runsTabSource).toContain("Clear filters");
    expect(runsTabSource).toContain("runsFilterDefaults()");
  });

  it("removes temporary chat from ReportCanvas once companion Chat exists", () => {
    expect(reportCanvasSource).not.toContain("temporary-follow-up");
    expect(reportCanvasSource).not.toContain("<ChatPanel");
    expect(reportCanvasSource).not.toContain("onAskRunQuestion");
    expect(reportCanvasSource).not.toContain("onChangeChatQuestion");
  });
});
