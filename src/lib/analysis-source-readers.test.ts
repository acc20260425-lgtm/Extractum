import { describe, expect, it } from "vitest";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import sourceGroupReaderSource from "./components/analysis/source-group-reader.svelte?raw";
import telegramMediaCardSource from "./components/analysis/telegram-media-card.svelte?raw";
import telegramTimelineSource from "./components/analysis/telegram-timeline-reader.svelte?raw";
import youtubePlaylistSource from "./components/analysis/youtube-playlist-reader.svelte?raw";
import youtubeTranscriptSource from "./components/analysis/youtube-transcript-reader.svelte?raw";

describe("analysis source readers", () => {
  it("replaces transitional source panels in ReportSourceSurface", () => {
    expect(reportSourceSurfaceSource).toContain("<TelegramTimelineReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubeTranscriptReader");
    expect(reportSourceSurfaceSource).toContain("<YoutubePlaylistReader");
    expect(reportSourceSurfaceSource).toContain("<SourceGroupReader");
    expect(reportSourceSurfaceSource).not.toContain("<SourceContextPanel");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubeSourceDetail");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistDetail");
    expect(reportSourceSurfaceSource).not.toContain("<RunCompanionTabs");
  });

  it("keeps live source and run snapshot basis visible", () => {
    expect(sourceReaderHeaderSource).toContain("sourceViewBasis");
    expect(sourceReaderHeaderSource).toContain("Live source");
    expect(sourceReaderHeaderSource).toContain("Run snapshot");
    expect(sourceReaderHeaderSource).toContain("Back to run snapshot");
    expect(sourceReaderHeaderSource).toContain("View live source");
  });

  it("uses a compact source reader heading instead of repeating the selected title", () => {
    expect(sourceReaderHeaderSource).toContain("surfaceLabel");
    expect(sourceReaderHeaderSource).not.toContain("<h2>{title}</h2>");
  });

  it("renders Telegram as a metadata-rich timeline without binary previews", () => {
    expect(telegramTimelineSource).toContain('class="telegram-timeline-reader"');
    expect(telegramTimelineSource).toContain("groupReaderItemsByDay");
    expect(telegramTimelineSource).toContain("topicLabel");
    expect(telegramTimelineSource).toContain("replyLabel");
    expect(telegramTimelineSource).toContain("reactionLabel");
    expect(telegramTimelineSource).toContain("<TelegramMediaCard");
    expect(telegramMediaCardSource).toContain("media-card");
    expect(telegramMediaCardSource).toContain("media.fileName");
    expect(telegramMediaCardSource).toContain("media.mimeType");
    expect(telegramMediaCardSource).not.toContain("<img");
    expect(telegramMediaCardSource).not.toContain("<video");
    expect(telegramMediaCardSource).not.toContain("<audio");
  });

  it("keeps sticky date labels below overlay source switching UI", () => {
    expect(telegramTimelineSource).toContain(".day-label");
    expect(telegramTimelineSource).toContain("position: sticky;");
    expect(telegramTimelineSource).toContain("z-index: 0;");
    expect(telegramTimelineSource).not.toContain("z-index: 1;");
  });

  it("uses TDesktop-inspired Telegram message geometry and typography", () => {
    expect(telegramTimelineSource).toContain('class="telegram-message-bubble"');
    expect(telegramTimelineSource).toContain('class="telegram-message-text"');
    expect(telegramTimelineSource).toContain('class="telegram-message-time"');
    expect(telegramTimelineSource).toContain("max-width: 460px;");
    expect(telegramTimelineSource).toContain("border-radius: 12px;");
    expect(telegramTimelineSource).toContain("font-size: 0.9375rem;");
    expect(telegramTimelineSource).toContain("font-size: 0.8125rem;");
  });

  it("centers Telegram message bubbles under sticky day labels", () => {
    expect(telegramTimelineSource).toMatch(/li\s*{\s*display: flex;\s*justify-content: center;/);
  });

  it("keeps Telegram bubbles and attachments visually compact", () => {
    expect(telegramTimelineSource).toContain("padding: 0.5rem 0.625rem 0.4375rem;");
    expect(telegramTimelineSource).toContain("line-height: 1.4;");
    expect(telegramTimelineSource).toContain("box-shadow: 0 0 0 1px");
    expect(telegramMediaCardSource).toContain("padding: 0.375rem 0.5rem;");
    expect(telegramMediaCardSource).toContain("border-radius: 7px;");
    expect(telegramMediaCardSource).toContain("font-size: 0.8125rem;");
  });

  it("allows Telegram message text to hyphenate long words", () => {
    expect(telegramTimelineSource).toContain('lang="ru"');
    expect(telegramTimelineSource).toContain("hyphens: auto;");
    expect(telegramTimelineSource).toContain("overflow-wrap: break-word;");
    expect(telegramTimelineSource).toContain("word-break: normal;");
  });

  it("renders YouTube videos as transcript-first source readers", () => {
    expect(youtubeTranscriptSource).toContain('class="youtube-transcript-reader"');
    expect(youtubeTranscriptSource).toContain("formatYoutubeTime");
    expect(youtubeTranscriptSource).toContain("youtubeTimestampUrl");
    expect(youtubeTranscriptSource).toContain("Copy timestamp link");
    expect(youtubeTranscriptSource).toContain("Search transcript");
    expect(youtubeTranscriptSource).toContain("Load more transcript");
    expect(youtubeTranscriptSource).not.toContain("<iframe");
    expect(youtubeTranscriptSource).not.toContain("<video");
  });

  it("renders transcript search as one compact input shell", () => {
    expect(youtubeTranscriptSource).toContain('placeholder="Search transcript"');
    expect(youtubeTranscriptSource).toContain('class="search-icon"');
    expect(youtubeTranscriptSource).toContain('class="search-input-wrap"');
  });

  it("keeps YouTube playlist reading playlist-first", () => {
    expect(youtubePlaylistSource).toContain('class="youtube-playlist-reader"');
    expect(youtubePlaylistSource).toContain("playlist.items");
    expect(youtubePlaylistSource).toContain("onOpenSource");
    expect(youtubePlaylistSource).toContain("onSyncPlaylistVideo");
    expect(youtubePlaylistSource).toContain("onRetryPlaylistVideo");
  });

  it("groups source group material by source", () => {
    expect(sourceGroupReaderSource).toContain('class="source-group-reader"');
    expect(sourceGroupReaderSource).toContain("groupReaderItemsBySource");
    expect(sourceGroupReaderSource).toContain("youtubeItems");
    expect(sourceGroupReaderSource).toContain("telegramItems");
    expect(sourceGroupReaderSource).toContain('item.kind === "youtube_transcript"');
    expect(sourceGroupReaderSource).not.toContain("snapshotItems={group.items}");
    expect(sourceGroupReaderSource).toContain("source-heading");
    expect(sourceGroupReaderSource).toContain("selectedGroupSourceId");
    expect(reportSourceSurfaceSource).toContain("onChangeSelectedSourceId={onChangeSelectedGroupSourceId}");
  });

  it("keeps source focus controls in one reader header location", () => {
    expect(sourceReaderHeaderSource).toContain("<span>Source focus</span>");
    expect(sourceGroupReaderSource).not.toContain("<span>Source focus</span>");
    expect(sourceGroupReaderSource).not.toContain("group-filter");
  });
});
