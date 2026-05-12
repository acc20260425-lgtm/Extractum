import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import telegramMediaCardSource from "./components/analysis/telegram-media-card.svelte?raw";
import telegramTimelineSource from "./components/analysis/telegram-timeline-reader.svelte?raw";
import youtubeTranscriptSource from "./components/analysis/youtube-transcript-reader.svelte?raw";
import youtubePlaylistSource from "./components/analysis/youtube-playlist-reader.svelte?raw";
import sourceGroupReaderSource from "./components/analysis/source-group-reader.svelte?raw";
import evidenceTabSource from "./components/analysis/run-evidence-tab.svelte?raw";
import chatTabSource from "./components/analysis/run-chat-tab.svelte?raw";
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";
import reportRunHeaderSource from "./components/analysis/report-run-header.svelte?raw";
import runCompanionStateSource from "./analysis-run-companion-state.ts?raw";
import analysisUtilsSource from "./analysis-utils.ts?raw";
import workspaceStateSource from "./analysis-workspace-state.ts?raw";
import chatBackendSource from "../../src-tauri/src/analysis/chat.rs?raw";
import corpusBackendSource from "../../src-tauri/src/analysis/corpus.rs?raw";
import storeBackendSource from "../../src-tauri/src/analysis/store.rs?raw";

describe("analysis redesign final safety contract", () => {
  it("keeps run snapshot and live source basis explicit in Source mode", () => {
    expect(reportSourceSurfaceSource).toContain("sourceViewBasis");
    expect(reportSourceSurfaceSource).toContain("run_snapshot");
    expect(reportSourceSurfaceSource).toContain("live_source");
    expect(sourceReaderHeaderSource).toContain("Live source");
    expect(sourceReaderHeaderSource).toContain("Run snapshot");
    expect(sourceReaderHeaderSource).toContain("View live source");
    expect(sourceReaderHeaderSource).toContain("Back to run snapshot");
    expect(reportSourceSurfaceSource).not.toContain("sourceViewBasis = \"run_snapshot\"");
    expect(analysisPageSource).not.toContain("autoSwitchToRunSnapshot");
  });

  it("does not resolve completed-run evidence through live source fallback", () => {
    expect(evidenceTabSource).toContain("evidenceSourceActionDecision");
    expect(evidenceTabSource).toContain("Show in source");
    expect(evidenceTabSource).toContain("Snapshot unavailable");
    expect(evidenceTabSource).not.toContain("listSourceItems");
    expect(corpusBackendSource).toContain("list_run_snapshot_messages_page");
    expect(corpusBackendSource).toContain("analysis_run_messages");
    expect(corpusBackendSource).toContain("load_trace_resolution_messages");
    expect(corpusBackendSource).toContain('run.status == "completed"');
    expect(corpusBackendSource).toContain("return Ok(Vec::new())");
    expect(corpusBackendSource).not.toContain("completed_run_live_source_fallback");
  });

  it("gates completed-run chat on saved run context instead of live source context", () => {
    expect(chatTabSource).toContain("chatAvailability");
    expect(runCompanionStateSource).toContain("completed");
    expect(runCompanionStateSource).toContain("snapshot");
    expect(chatTabSource).not.toContain("onfocus");
    expect(chatTabSource).not.toContain("onFocus");
    expect(chatBackendSource).toContain("load_run_snapshot_messages");
    expect(chatBackendSource).toContain("ensure_completed_chat_context");
    expect(chatBackendSource).toContain("completed");
    expect(chatBackendSource).not.toContain("load_run_corpus_messages(&pool, &run)");
  });

  it("keeps source ingest activity out of analysis Runs", () => {
    expect(runsTabSource).not.toContain("SourceJobRecord");
    expect(runsTabSource).not.toContain("sourceJobs");
    expect(runsTabSource).not.toContain("takeoutJobs");
    expect(runsTabSource).not.toContain("Sync transcript");
    expect(runsTabSource).not.toContain("Takeout import");
    expect(reportSourceSurfaceSource).toContain("sourceJobs: SourceJobRecord[]");
    expect(reportSourceSurfaceSource).toContain("onSyncYoutubeTranscript");
    expect(reportSourceSurfaceSource).toContain("onSyncYoutubePlaylist");
    expect(reportSourceSurfaceSource).toContain("onCancelSourceJob");
  });

  it("renders Telegram source material as metadata-first timeline without binary previews", () => {
    expect(telegramTimelineSource).toContain('class="telegram-timeline-reader"');
    expect(telegramTimelineSource).toContain("topicLabel");
    expect(telegramTimelineSource).toContain("replyLabel");
    expect(telegramTimelineSource).toContain("reactionLabel");
    expect(telegramTimelineSource).toContain("<TelegramMediaCard");
    expect(telegramTimelineSource).toContain("item.mediaCards as media, index");
    expect(telegramMediaCardSource).toContain("media.fileName");
    expect(telegramMediaCardSource).toContain("media.mimeType");
    expect(telegramMediaCardSource).not.toContain("<img");
    expect(telegramMediaCardSource).not.toContain("<video");
    expect(telegramMediaCardSource).not.toContain("<audio");
  });

  it("renders YouTube source material as transcript and playlist readers without an embedded player", () => {
    expect(youtubeTranscriptSource).toContain('class="youtube-transcript-reader"');
    expect(youtubeTranscriptSource).toContain("Search transcript");
    expect(youtubeTranscriptSource).toContain("Copy timestamp link");
    expect(youtubeTranscriptSource).toContain("youtubeTimestampUrl");
    expect(youtubeTranscriptSource).toContain("navigator.clipboard");
    expect(youtubeTranscriptSource).toContain("catch");
    expect(youtubeTranscriptSource).toContain('rel="noopener noreferrer"');
    expect(youtubeTranscriptSource).not.toContain("<iframe");
    expect(youtubeTranscriptSource).not.toContain("<video");
    expect(youtubePlaylistSource).toContain('class="youtube-playlist-reader"');
    expect(youtubePlaylistSource).toContain("playlist.items");
    expect(youtubePlaylistSource).toContain("onOpenSource");
  });

  it("keeps source groups grouped by source instead of merged into one pseudo-chat", () => {
    expect(sourceGroupReaderSource).toContain('class="source-group-reader"');
    expect(sourceGroupReaderSource).toContain("groupReaderItemsBySource");
    expect(sourceGroupReaderSource).toContain("youtubeItems");
    expect(sourceGroupReaderSource).toContain("telegramItems");
    expect(sourceGroupReaderSource).toContain("source-heading");
    expect(sourceGroupReaderSource).toContain("selectedGroupSourceId");
    expect(sourceGroupReaderSource).not.toContain("mergedTimeline");
    expect(sourceGroupReaderSource).not.toContain("pseudoChat");
  });

  it("keeps missing or deleted run scope labeling visible in the run header", () => {
    expect(storeBackendSource).toContain("scope_label_snapshot");
    expect(storeBackendSource).toContain("resolve_run_scope_label");
    expect(analysisUtilsSource).toContain("run.scope_label.trim()");
    expect(reportRunHeaderSource).toContain("missing");
    expect(reportRunHeaderSource).toContain("runTargetLabel(currentRun)");
    expect(workspaceStateSource).toContain("liveScopeExists");
    expect(reportRunHeaderSource).toContain("Source basis");
    expect(reportRunHeaderSource).toContain("youtube_corpus_mode");
    expect(reportRunHeaderSource).toContain("promptTemplateLabel");
    expect(reportRunHeaderSource).toContain("snapshotBadgeVariant");
    expect(reportRunHeaderSource).toContain('availability === "unavailable"');
    expect(reportRunHeaderSource).toContain('return "warning"');
  });

  it("does not hide completed chat persistence failures", () => {
    expect(chatBackendSource).toContain("persist_chat_exchange");
    expect(chatBackendSource).not.toContain("let _ = persist_chat_exchange");
  });

  it("uses stable run filter normalization instead of locale-sensitive casing", () => {
    expect(runCompanionStateSource).toContain(".toLowerCase()");
    expect(runCompanionStateSource).not.toContain(".toLocaleLowerCase()");
  });
});
