import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import type { ComponentProps } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import SourceSwitcherPanel from "./source-switcher-panel.svelte";
import type { AnalysisSourceGroup } from "$lib/types/analysis";
import type { Source, TakeoutImportJobRecord } from "$lib/types/sources";
import type { YoutubeSourceSummary } from "$lib/types/youtube";

afterEach(cleanup);

function source(overrides: Partial<Source> = {}): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "supergroup",
    accountId: 7,
    externalId: "research-channel",
    title: "Research channel",
    lastSyncState: 4,
    lastSyncedAt: 1_700_000_000,
    isMember: true,
    isActive: true,
    createdAt: 1_699_000_000,
    telegramUsername: "research",
    avatarDataUrl: null,
    migratedHistoryStatus: "available",
    migratedHistoryDetectedAt: 1_699_500_000,
    migratedHistoryRefreshedAt: 1_700_000_000,
    migratedHistoryRowCount: 8,
    migratedHistoryImportCompleted: false,
    ...overrides,
  };
}

function sourceGroup(): AnalysisSourceGroup {
  return {
    id: 20,
    name: "Research group",
    source_type: "telegram",
    members: [{ source_id: 1, source_title: "Research channel", item_count: 12 }],
    created_at: 1_699_000_000,
    updated_at: 1_700_000_000,
  };
}

function youtubeSummary(): YoutubeSourceSummary {
  return {
    sourceId: 2,
    sourceSubtype: "video",
    title: "Rendered video",
    channelTitle: "Research channel",
    channelHandle: "@research-video",
    canonicalUrl: "https://www.youtube.com/watch?v=video-2",
    thumbnailUrl: null,
    durationSeconds: 125,
    publishedAt: 1_700_000_000,
    availabilityStatus: "available",
    videoCount: 3,
    linkedVideoCount: 2,
    unavailableCount: 1,
    captions: { state: "synced", itemCount: 1, segmentCount: 3, lastSyncedAt: 1_700_000_000, label: "Captions ready" },
    comments: { state: "synced", itemCount: 2, segmentCount: 0, lastSyncedAt: 1_700_000_000, label: "Comments ready" },
  };
}

function takeoutJob(overrides: Partial<TakeoutImportJobRecord> = {}): TakeoutImportJobRecord {
  return {
    job_id: "takeout-1",
    source_id: 1,
    account_id: 7,
    batch_id: 11,
    history_scope: "current_history",
    status: "running",
    phase: "importing_history",
    message: "Halfway through archive",
    inserted: 3,
    skipped: 1,
    progress_current: 2,
    progress_total: 4,
    started_at: 1_700_000_000,
    finished_at: null,
    warnings: ["One recoverable warning"],
    error: "One row could not be decoded",
    ...overrides,
  };
}

type PanelProps = ComponentProps<typeof SourceSwitcherPanel>;

function panelProps(overrides: Partial<PanelProps> = {}): PanelProps {
  const telegram = source();
  const youtube = source({
    id: 2,
    sourceType: "youtube",
    sourceSubtype: "video",
    accountId: null,
    externalId: "video-2",
    title: "Fallback video",
    telegramUsername: null,
    migratedHistoryStatus: "none",
  });
  const group = sourceGroup();
  return {
    sourceCatalog: [telegram, youtube],
    groups: [group],
    sourceMetrics: {
      1: { id: 1, account_id: 7, source_type: "telegram", title: "Research channel", item_count: 12, last_synced_at: 1_700_000_000 },
      2: { id: 2, account_id: null, source_type: "youtube", title: "Rendered video", item_count: 3, last_synced_at: 1_700_000_000 },
    },
    loadingSourceCatalog: false,
    loadingGroups: false,
    railQuery: "",
    filteredSourceCatalog: [telegram, youtube],
    filteredGroups: [group],
    workspaceSelection: { kind: "source", sourceId: 1 },
    syncingIds: {},
    deletingSourceIds: {},
    startingTakeoutSourceIds: {},
    startingMigratedHistorySourceIds: {},
    takeoutJobsBySource: {},
    takeoutRecoveryBySource: {},
    sourceJobsBySource: {},
    youtubeSummaries: { 2: youtubeSummary() },
    youtubeRuntimeStatus: { ytdlpAvailable: true, ytdlpVersion: "2026.08", message: "Ready" },
    formatTimestamp: (value) => value === null ? "Never" : `time:${value}`,
    accountLabel: (accountId) => accountId === null ? "No account" : `Account ${accountId}`,
    sourceInitial: (value) => value.title?.charAt(0) ?? "?",
    runtimeStatus: () => null,
    runtimeBadge: () => "",
    sourceSyncDisabledReason: () => null,
    onChangeRailQuery: vi.fn(),
    onSelectSource: vi.fn(),
    onSelectGroup: vi.fn(),
    onSyncSource: vi.fn(),
    onStartTakeoutImport: vi.fn(),
    onStartMigratedHistoryImport: vi.fn(),
    onCancelTakeoutImport: vi.fn(),
    onCancelSourceJob: vi.fn(),
    onOpenSourceManager: vi.fn(),
    onDeleteSource: vi.fn(),
    onClose: vi.fn(),
    ...overrides,
  };
}

describe("compact analysis source rail", () => {
  it("puts full list, search, management, and detailed status in the expanded source panel", async () => {
    const onOpenSourceManager = vi.fn();
    const onChangeRailQuery = vi.fn();
    render(SourceSwitcherPanel, { props: panelProps({ onOpenSourceManager, onChangeRailQuery }) });

    expect(screen.getByRole("region", { name: "Source switcher panel" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Switch source context" })).toBeTruthy();
    expect(screen.getByRole("searchbox", { name: "Search sources or groups" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "New source" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Manage sources" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Close" })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Research channel/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Rendered video/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Research group/ })).toBeTruthy();
    expect(screen.getByText("@research - Account 7")).toBeTruthy();
    expect(screen.getByText("Captions ready")).toBeTruthy();
    expect(screen.getByText("Comments ready")).toBeTruthy();
    expect(screen.getByText("2/2")).toBeTruthy();
    expect(screen.getByText("1/1")).toBeTruthy();
    expect(screen.getByRole("button", { name: /Research channel/ }).getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByRole("button", { name: /Research group/ }).getAttribute("aria-pressed")).toBe("false");
    await fireEvent.click(screen.getByRole("button", { name: "New source" }));
    expect(onOpenSourceManager).toHaveBeenCalledOnce();
    await fireEvent.input(screen.getByRole("searchbox", { name: "Search sources or groups" }), { target: { value: "video" } });
    expect(onChangeRailQuery).toHaveBeenCalledWith("video");
  });

  it("keeps detailed Takeout import progress in the expanded source panel", async () => {
    const onCancelTakeoutImport = vi.fn();
    const telegram = source();
    render(SourceSwitcherPanel, {
      props: panelProps({
        sourceCatalog: [telegram],
        filteredSourceCatalog: [telegram],
        youtubeSummaries: {},
        takeoutJobsBySource: { 1: takeoutJob() },
        onCancelTakeoutImport,
      }),
    });

    expect(screen.getByText("Takeout importing history")).toBeTruthy();
    expect(screen.getByText("2/4")).toBeTruthy();
    const progress = screen.getByRole("progressbar");
    expect((progress as HTMLProgressElement).value).toBe(50);
    expect((progress as HTMLProgressElement).max).toBe(100);
    expect(screen.getByText("3 inserted, 1 skipped")).toBeTruthy();
    expect(screen.getByText("Halfway through archive")).toBeTruthy();
    expect(screen.getByText("One row could not be decoded")).toBeTruthy();
    expect(screen.getByText("One recoverable warning")).toBeTruthy();
    await fireEvent.click(screen.getByText("Source operations"));
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancelTakeoutImport).toHaveBeenCalledWith("takeout-1");
  });

  it("keeps YouTube video duration visible in expanded source metadata", () => {
    const youtube = source({ id: 2, sourceType: "youtube", sourceSubtype: "video", accountId: null, externalId: "video-2", telegramUsername: null });
    render(SourceSwitcherPanel, { props: panelProps({ sourceCatalog: [youtube], filteredSourceCatalog: [youtube] }) });

    expect(screen.getByText("Rendered video")).toBeTruthy();
    expect(screen.getByText("@research-video - 2:05 - published time:1700000000 - 3 videos - 2 linked - 1 unavailable")).toBeTruthy();
  });

  it("keeps Telegram username and sync freshness visible in expanded source metadata", () => {
    const telegram = source();
    render(SourceSwitcherPanel, { props: panelProps({ sourceCatalog: [telegram], filteredSourceCatalog: [telegram], youtubeSummaries: {} }) });

    expect(screen.getByRole("button", { name: /Research channel/ })).toBeTruthy();
    expect(screen.getByText("@research - Account 7")).toBeTruthy();
    expect(screen.getByText("Synced time:1700000000")).toBeTruthy();
  });

  it("keeps source and group switching callback-based", async () => {
    const onSelectSource = vi.fn();
    const onSelectGroup = vi.fn();
    render(SourceSwitcherPanel, { props: panelProps({ onSelectSource, onSelectGroup }) });

    await fireEvent.click(screen.getByRole("button", { name: /Rendered video/ }));
    expect(onSelectSource).toHaveBeenCalledWith(2);
    expect(onSelectGroup).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: /Research group/ }));
    expect(onSelectGroup).toHaveBeenCalledWith(20);
    expect(onSelectSource).toHaveBeenCalledTimes(1);
  });

  it("keeps destructive source deletion out of the compact rail but available in the expanded panel", async () => {
    const onDeleteSource = vi.fn();
    const onOpenSourceManager = vi.fn();
    render(SourceSwitcherPanel, { props: panelProps({ onDeleteSource, onOpenSourceManager }) });

    const operations = screen.getAllByText("Source operations")[0].closest("details") as HTMLDetailsElement;
    expect(operations).toBeTruthy();
    expect(operations.open).toBe(false);
    await fireEvent.click(operations.querySelector("summary") as HTMLElement);
    expect(within(operations).getByText("Manage operational state in the Activity tab.")).toBeTruthy();
    const deleteButton = within(operations).getByRole("button", { name: "Delete" });
    expect(deleteButton).toBeTruthy();
    expect((deleteButton as HTMLButtonElement).disabled).toBe(false);
    await fireEvent.click(deleteButton);
    expect(onDeleteSource.mock.calls[0]?.[0]?.id).toBe(1);
    await fireEvent.click(screen.getByRole("button", { name: "Manage sources" }));
    expect(onOpenSourceManager).toHaveBeenCalledOnce();
  });

  it("keeps icon-only controls accessible without hover-only status", async () => {
    render(SourceSwitcherPanel, { props: panelProps() });

    expect(screen.getByRole("button", { name: /Research channel/ }).getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByRole("button", { name: /Research group/ }).getAttribute("aria-pressed")).toBe("false");
    expect(screen.getByRole("button", { name: "Close" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "New source" })).toBeTruthy();
    const operations = screen.getAllByText("Source operations")[0].closest("details") as HTMLDetailsElement;
    await fireEvent.click(operations.querySelector("summary") as HTMLElement);
    expect(within(operations).getByRole("button", { name: "Sync" })).toBeTruthy();
    expect(within(operations).getByRole("button", { name: "Takeout" })).toBeTruthy();
    expect(within(operations).getByRole("button", { name: "Delete" })).toBeTruthy();
  });
});

describe("analysis priority UX contract", () => {
  it("keeps the source switcher primarily focused on source selection", () => {
    render(SourceSwitcherPanel, { props: panelProps() });

    expect(screen.getByRole("button", { name: /Research channel/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Research group/ })).toBeTruthy();
    const operations = screen.getAllByText("Source operations")[0].closest("details") as HTMLDetailsElement;
    expect(operations.open).toBe(false);
    expect(operations.textContent).toContain("Manage operational state in the Activity tab.");
  });
});
