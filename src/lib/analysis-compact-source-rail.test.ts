import { describe, expect, it } from "vitest";
import compactRailSource from "./components/analysis/compact-source-rail.svelte?raw";
import sourceSwitcherPanelSource from "./components/analysis/source-switcher-panel.svelte?raw";

describe("compact analysis source rail", () => {
  it("keeps the collapsed rail compact and source-scoped", () => {
    expect(compactRailSource).toContain('class="compact-source-rail"');
    expect(compactRailSource).toContain("workspaceSelection: WorkspaceSelection");
    expect(compactRailSource).toContain("sourceSwitcherOpen");
    expect(compactRailSource).toContain('ariaLabel="Open source switcher"');
    expect(compactRailSource).toContain('title="Open source switcher"');
    expect(compactRailSource).toContain("context-primary-action");
    expect(compactRailSource).toContain("criticalSourceStatus");
    expect(compactRailSource).toContain("selected={isSelectedSource(source.id)}");
    expect(compactRailSource).toContain("selected={isSelectedGroup(group.id)}");
    expect(compactRailSource).not.toContain("<h1>Workspace</h1>");
    expect(compactRailSource).not.toContain("Research context");
    expect(compactRailSource).not.toContain("Manage sources");
    expect(compactRailSource).not.toContain("Transcript unavailable");
    expect(compactRailSource).not.toContain("Comments unavailable");
  });

  it("puts full list, search, management, and detailed status in the expanded source panel", () => {
    expect(sourceSwitcherPanelSource).toContain('class="source-switcher-panel"');
    expect(sourceSwitcherPanelSource).toContain('aria-label="Source switcher panel"');
    expect(sourceSwitcherPanelSource).toContain("Search sources or groups");
    expect(sourceSwitcherPanelSource).toContain("Manage sources");
    expect(sourceSwitcherPanelSource).toContain("New source");
    expect(sourceSwitcherPanelSource).toContain("filteredSourceCatalog");
    expect(sourceSwitcherPanelSource).toContain("filteredGroups");
    expect(sourceSwitcherPanelSource).toContain("youtubeSummary.captions.label");
    expect(sourceSwitcherPanelSource).toContain("youtubeSummary.comments.label");
    expect(sourceSwitcherPanelSource).toContain("takeout-status");
    expect(sourceSwitcherPanelSource).toContain("sourceJobsBySource");
    expect(sourceSwitcherPanelSource).toContain("onSyncSource(source.id)");
    expect(sourceSwitcherPanelSource).toContain("onStartTakeoutImport(source.id)");
    expect(sourceSwitcherPanelSource).toContain("onStartMigratedHistoryImport(source.id)");
    expect(sourceSwitcherPanelSource).toContain("migratedHistoryActionLabel()");
    expect(sourceSwitcherPanelSource).not.toContain("Retry migrated history");
    expect(sourceSwitcherPanelSource).not.toContain("Sync migrated history");
    expect(sourceSwitcherPanelSource).toContain("onDeleteSource(source)");
  });

  it("passes migrated history action state through the compact rail", () => {
    expect(compactRailSource).toContain("startingMigratedHistorySourceIds");
    expect(compactRailSource).toContain("onStartMigratedHistoryImport");
    expect(sourceSwitcherPanelSource).toContain("migratedHistoryActionDisabledReason");
    expect(sourceSwitcherPanelSource).toContain("migratedHistoryActionLabel");
  });

  it("keeps detailed Takeout import progress in the expanded source panel", () => {
    expect(sourceSwitcherPanelSource).toContain("takeoutProgressLabel(takeoutJob)");
    expect(sourceSwitcherPanelSource).toContain("takeoutProgressValue(takeoutJob)");
    expect(sourceSwitcherPanelSource).toContain("takeoutSummary(takeoutJob)");
    expect(sourceSwitcherPanelSource).toContain('class:terminal={!takeoutActive}');
    expect(sourceSwitcherPanelSource).toContain('<progress max="100" value={progressValue}');
    expect(sourceSwitcherPanelSource).toContain("<progress></progress>");
    expect(sourceSwitcherPanelSource).toContain("takeoutJob.error");
    expect(sourceSwitcherPanelSource).toContain("takeout-issue");
    expect(sourceSwitcherPanelSource).toContain("takeoutJob.warnings.length");
  });

  it("uses the shared takeout recovery notice in source rows", () => {
    expect(sourceSwitcherPanelSource).toContain("TakeoutRecoveryNotice");
    expect(sourceSwitcherPanelSource).toContain("visibleTakeoutRecoveryForSource");
  });

  it("keeps YouTube video duration visible in expanded source metadata", () => {
    expect(sourceSwitcherPanelSource).toContain("formatDuration(summary.durationSeconds)");
    expect(sourceSwitcherPanelSource).toContain("youtubeMetaLine(youtubeSummary)");
  });

  it("keeps Telegram username and sync freshness visible in expanded source metadata", () => {
    expect(sourceSwitcherPanelSource).toContain("telegramMetaLine(source)");
    expect(sourceSwitcherPanelSource).toContain("source.telegramUsername");
    expect(sourceSwitcherPanelSource).toContain("formatTimestamp(metrics.last_synced_at)");
  });

  it("keeps source and group switching callback-based", () => {
    expect(compactRailSource).toContain("onclick={() => onSelectSource(source.id)}");
    expect(compactRailSource).toContain("onclick={() => onSelectGroup(group.id)}");
    expect(sourceSwitcherPanelSource).toContain("onclick={() => onSelectSource(source.id)}");
    expect(sourceSwitcherPanelSource).toContain("onclick={() => onSelectGroup(group.id)}");
  });

  it("closes the expanded switcher after quick source or group selection", () => {
    expect(compactRailSource).toContain("selectSourceAndClose");
    expect(compactRailSource).toContain("selectGroupAndClose");
  });

  it("keeps destructive source deletion out of the compact rail but available in the expanded panel", () => {
    expect(compactRailSource).toContain("onDeleteSource");
    expect(compactRailSource).toContain("{onDeleteSource}");
    expect(compactRailSource).not.toContain("onDeleteSource(source)");
    expect(sourceSwitcherPanelSource).toContain("onDeleteSource: (source: Source) => void");
    expect(sourceSwitcherPanelSource).toContain("onDeleteSource(source)");
    expect(sourceSwitcherPanelSource).toContain("Delete");
    expect(sourceSwitcherPanelSource).toContain("Manage sources");
  });

  it("keeps icon-only controls accessible without hover-only status", () => {
    expect(compactRailSource).toContain("ariaLabel={sourceButtonLabel(source)}");
    expect(compactRailSource).toContain("title={sourceButtonLabel(source)}");
    expect(compactRailSource).toContain("ariaLabel={groupButtonLabel(group)}");
    expect(compactRailSource).toContain("title={groupButtonLabel(group)}");
    expect(compactRailSource).toContain("title={criticalStatusLabel}");
    expect(sourceSwitcherPanelSource).toContain("aria-pressed={isSelectedSource(source.id)}");
    expect(sourceSwitcherPanelSource).toContain("aria-pressed={isSelectedGroup(group.id)}");
  });

  it("uses a compact mobile source context bar", () => {
    expect(compactRailSource).toContain("mobile-current-label");
    expect(compactRailSource).toContain("quick-list-scroll");
    expect(compactRailSource).toContain("@media (max-width: 720px)");
  });

  it("reduces rail chrome without widening the rail", () => {
    expect(compactRailSource).toContain(
      "padding: 0.35rem;",
    );
    expect(compactRailSource).toContain("z-index: 30;");
    expect(compactRailSource).toContain(
      "border: 1px solid color-mix(in srgb, var(--border) 38%, transparent);",
    );
    expect(compactRailSource).toContain("border: 0;");
    expect(compactRailSource).toContain("box-shadow: 0 0 0 1px color-mix(in srgb, var(--primary) 44%, transparent);");
    expect(compactRailSource).not.toContain("width: 4rem");
  });

  it("shows mini source logos without cropping them", () => {
    expect(compactRailSource).toContain(".mini-avatar img");
    expect(compactRailSource).toContain(".quick-list :global(.ui-button.icon-only.sm)");
    expect(compactRailSource).toContain("width: 3.75rem;");
    expect(compactRailSource).toContain("height: 3rem;");
    expect(compactRailSource).toContain("width: 2.5rem;");
    expect(compactRailSource).toContain("height: 2.5rem;");
    expect(compactRailSource).toContain("object-fit: contain;");
    expect(compactRailSource).toContain("background: var(--panel);");
  });
});
