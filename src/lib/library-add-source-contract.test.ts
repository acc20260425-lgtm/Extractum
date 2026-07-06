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
    expect(smartImportSource).toContain("existingYoutubeSmartImportSource");
    expect(smartImportSource).toContain("backendUrl");
    expect(smartImportSource).toContain("previewYoutubeSource");
    expect(smartImportSource).toContain("addYoutubeSource");
    expect(smartImportSource).toContain('formatAppError("previewing the YouTube source"');
    expect(smartImportSource).toContain('formatAppError("adding the YouTube source"');
    expect(smartImportSource).toContain("Not supported yet");
  });

  it("does not materialize playlist videos from Library smart import", () => {
    expect(smartImportSource).toContain("materializePlaylistVideos");
    expect(smartImportSource).toContain("materializePlaylistVideos: preview.kind !== \"playlist\"");
  });

  it("keeps duplicate YouTube smart import feedback inside the modal", () => {
    expect(youtubePanelSource).toContain("<LibraryYoutubeSmartImport {sources}");
    expect(smartImportSource).toContain("Already in Library");
    expect(smartImportSource).toContain("existingSmartImportSource");
    expect(smartImportSource).toContain("canAdd = $derived(Boolean(preview) && !existingSmartImportSource");
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

  it("keeps the standalone scalar onSourcesChanged contract while accepting project context", () => {
    expect(dialogSource).toContain("projectContext?: ProjectAddSourceContext");
    expect(dialogSource).toContain("onSourcesChanged: (sourceId?: number) => void | Promise<void>");
    expect(youtubePanelSource).toContain("projectContext?: ProjectAddSourceContext");
    expect(youtubePanelSource).toContain("onSourcesChanged: (sourceId?: number) => void | Promise<void>");
    expect(playlistImportSource).toContain("onSourcesChanged: (sourceId?: number) => void | Promise<void>");
    expect(playlistImportSource).not.toContain("onSourcesChanged: (sourceIds: number[])");
  });

  it("passes project context through the YouTube add-source tree", () => {
    expect(dialogSource).toMatch(/<LibraryYoutubeAddPanel[\s\S]*\{projectContext\}/);
    expect(youtubePanelSource).toMatch(/<LibraryYoutubeSmartImport[\s\S]*\{projectContext\}/);
    expect(youtubePanelSource).toMatch(/<LibraryYoutubePlaylistImport[\s\S]*\{projectContext\}/);
  });

  it("allows Smart import duplicates to connect existing Library sources in project mode", () => {
    expect(smartImportSource).toContain("canConnectExistingSmartImportSource");
    expect(smartImportSource).toContain("projectContext.onConnectExistingSource(existingSmartImportSource.sourceId)");
    expect(smartImportSource).toContain("Connect to project");
    expect(smartImportSource).toContain("Connecting...");
    expect(smartImportSource).toContain("Already connected to this project");
  });

  it("keeps Smart import playlists on the scalar source callback path", () => {
    expect(smartImportSource).toContain('materializePlaylistVideos: preview.kind !== "playlist"');
    expect(smartImportSource).toContain("await onSourcesChanged(source.id)");
  });

  it("connects all added playlist video source IDs through the project batch callback", () => {
    expect(playlistImportSource).toContain('result.status === "added"');
    expect(playlistImportSource).toContain("projectContext.onConnectAddedSources(addedSourceIds)");
    expect(playlistImportSource).toContain("await onSourcesChanged(");
    expect(playlistImportSource).toContain("summary.results.find((result) => result.sourceId !== null)?.sourceId ?? undefined");
  });

  it("keeps Telegram project connection on the scalar callback path", () => {
    expect(dialogSource).toMatch(/<LibraryTelegramDialogImport[\s\S]*\{onSourcesChanged\}[\s\S]*\{onStatus\}/);
    expect(dialogSource).not.toMatch(/<LibraryTelegramDialogImport[\s\S]*\{projectContext\}/);
    expect(telegramImportSource).toContain("await onSourcesChanged(source.id)");
  });
});
