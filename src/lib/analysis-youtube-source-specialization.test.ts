import { describe, expect, it } from "vitest";
import analysisStateSource from "./analysis-state.ts?raw";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import sourceBrowserShellSource from "./components/analysis/source-browser-shell.svelte?raw";
import sourceActivityViewSource from "./components/analysis/source-activity-view.svelte?raw";
import universalItemsViewSource from "./components/analysis/universal-items-view.svelte?raw";
import youtubeCommentsViewSource from "./components/analysis/youtube-comments-view.svelte?raw";
import youtubePlaylistVideosViewSource from "./components/analysis/youtube-playlist-videos-view.svelte?raw";
import youtubeTranscriptReaderSource from "./components/analysis/youtube-transcript-reader.svelte?raw";

describe("analysis youtube source specialization", () => {
  it("keeps youtube detail errors scoped to the selected source", () => {
    expect(analysisPageSource).toContain("youtubeDetailError");
    expect(analysisPageSource).toContain("YoutubeDetailErrorState");
    expect(analysisPageSource).toContain("youtubeDetailError = null");
    expect(analysisPageSource).toContain("youtubeDetailError = {");
    expect(analysisPageSource).toContain("sourceId: source.id");
    expect(analysisPageSource).toContain("[source.id]: detail.summary");
    expect(analysisPageSource).not.toContain('status = formatAppError("loading YouTube detail", error)');
  });

  it("uses the scoped youtube detail problem in report preflight copy", () => {
    expect(analysisStateSource).toContain("youtubeDetailProblemReason");
    expect(analysisStateSource).toContain("return state.youtubeDetailProblemReason");
    expect(analysisPageSource).toContain("youtubeDetailProblemReason: currentYoutubeDetailProblemReason()");
  });

  it("threads youtube detail error into report setup and source browser", () => {
    expect(analysisPageSource).toContain("{youtubeDetailError}");
    expect(reportCanvasSource).toContain("youtubeDetailError?: YoutubeDetailErrorState");
    expect(reportCanvasSource).toContain("{youtubeDetailError}");
    expect(reportSetupPanelSource).toContain("youtubeDetailError");
    expect(reportSourceSurfaceSource).toContain("youtubeDetailError");
    expect(sourceBrowserShellSource).toContain("youtubeDetailError");
  });

  it("promotes youtube corpus into a provider-specific report decision block", () => {
    expect(reportSetupPanelSource).toContain("youtubeCorpusOptionViews");
    expect(reportSetupPanelSource).toContain('class="youtube-corpus-panel"');
    expect(reportSetupPanelSource).toContain("Audience comments are user-generated evidence");
    expect(reportSetupPanelSource).not.toContain("<label>YouTube corpus");
  });

  it("renders invalid playlists as problem states instead of empty playlists", () => {
    expect(youtubePlaylistVideosViewSource).toContain("playlistDetailError");
    expect(youtubePlaylistVideosViewSource).toContain("Playlist metadata needs attention");
    expect(youtubePlaylistVideosViewSource).toContain("This is not an empty playlist.");
    expect(youtubePlaylistVideosViewSource).toContain("Retry playlist sync");
  });

  it("uses compact youtube status copy in transcript and comments readers", () => {
    expect(youtubeTranscriptReaderSource).toContain("youtubeProviderHeaderSummary");
    expect(youtubeTranscriptReaderSource).toContain("youtubeContentStatusLine");
    expect(youtubeTranscriptReaderSource).not.toContain("Comments {summary.comments.label}");
    expect(youtubeCommentsViewSource).toContain("Search comments");
    expect(youtubeCommentsViewSource).not.toContain("Search loaded comments");
  });

  it("renders youtube items as evidence inventory and activity as provider steps", () => {
    expect(universalItemsViewSource).toContain("Evidence inventory");
    expect(universalItemsViewSource).toContain("youtubeEvidenceRoleLabel");
    expect(sourceActivityViewSource).toContain("YouTube provider steps");
    expect(sourceActivityViewSource).toContain("Metadata");
    expect(sourceActivityViewSource).toContain("Transcript");
    expect(sourceActivityViewSource).toContain("Comments");
  });
});
