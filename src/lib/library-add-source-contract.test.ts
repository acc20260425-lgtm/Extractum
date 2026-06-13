import { describe, expect, it } from "vitest";
import dialogSource from "./components/research-projects/LibraryAddSourceDialog.svelte?raw";
import youtubePanelSource from "./components/research-projects/LibraryYoutubeAddPanel.svelte?raw";
import smartImportSource from "./components/research-projects/LibraryYoutubeSmartImport.svelte?raw";
import playlistImportSource from "./components/research-projects/LibraryYoutubePlaylistImport.svelte?raw";
import telegramImportSource from "./components/research-projects/LibraryTelegramDialogImport.svelte?raw";

describe("Library Add Source contract", () => {
  const addSourceComponentSources = [
    dialogSource,
    youtubePanelSource,
    smartImportSource,
    playlistImportSource,
    telegramImportSource,
  ];

  it("uses extractum wrappers for dialog and tabs", () => {
    expect(dialogSource).toContain("ExtractumDialog");
    expect(dialogSource).toContain("ExtractumTabs");
    expect(dialogSource).toContain("ExtractumTabsList");
    expect(dialogSource).toContain("ExtractumTabsTrigger");
    expect(dialogSource).toContain("ExtractumTabsContent");
    expect(dialogSource).toContain('data-ui-region="library-add-source-dialog"');
    expect(dialogSource).not.toContain("$lib/components/ui/");
    expect(dialogSource).not.toContain("bits-ui");
  });

  it("keeps all Add Source components behind extractum-ui wrappers", () => {
    for (const source of addSourceComponentSources) {
      expect(source).not.toContain("$lib/components/ui/");
      expect(source).not.toContain("bits-ui");
      expect(source).not.toContain("@svar-ui/");
    }
  });

  it("keeps YouTube mode tabs inside the YouTube panel", () => {
    expect(youtubePanelSource).toContain("Smart import");
    expect(youtubePanelSource).toContain("From existing data");
    expect(youtubePanelSource).toContain("LibraryYoutubeSmartImport");
    expect(youtubePanelSource).toContain("LibraryYoutubePlaylistImport");
  });

  it("classifies YouTube smart import before calling backend preview", () => {
    expect(smartImportSource).toContain("classifyYoutubeImportInput");
    expect(smartImportSource).toContain("backendUrl");
    expect(smartImportSource).toContain("previewYoutubeSource");
    expect(smartImportSource).toContain("addYoutubeSource");
    expect(smartImportSource).toContain('formatAppError("previewing the YouTube source"');
    expect(smartImportSource).toContain('formatAppError("adding the YouTube source"');
    expect(smartImportSource).toContain("Not supported yet");
  });

  it("adds selected videos from existing playlist details", () => {
    expect(playlistImportSource).toContain("getYoutubePlaylistDetail");
    expect(playlistImportSource).toContain("addSelectedYoutubePlaylistVideos");
    expect(playlistImportSource).toContain("YOUTUBE_PLAYLIST_IMPORT_LIMIT");
    expect(playlistImportSource).toContain("playlistSelectionLimitMessage");
    expect(playlistImportSource).toContain("sources: LibraryCatalogSourceView[]");
    expect(playlistImportSource).toContain("Already in Library");
    expect(playlistImportSource).toContain("import-result-list");
    expect(playlistImportSource).toContain("{#each summary.results as result");
  });

  it("adds Telegram sources only from selected account dialogs", () => {
    expect(telegramImportSource).toContain("onMount");
    expect(telegramImportSource).toContain("listAccounts");
    expect(telegramImportSource).toContain("getAccountRuntimeStatuses");
    expect(telegramImportSource).toContain("listTelegramSources");
    expect(telegramImportSource).toContain("telegramDialogAddInput");
    expect(telegramImportSource).toContain("addTelegramSource");
    expect(telegramImportSource).toContain("No accounts configured");
    expect(telegramImportSource).toContain("before loading Telegram dialogs");
    expect(telegramImportSource).toContain('formatAppError("adding Telegram source"');
  });
});
