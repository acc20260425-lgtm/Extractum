import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import type { ComponentProps } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import CompactSourceRail from "./compact-source-rail.svelte";
import type { AnalysisSourceGroup } from "$lib/types/analysis";
import type { Source } from "$lib/types/sources";
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
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
    ...overrides,
  };
}

function group(): AnalysisSourceGroup {
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
  const synced = {
    state: "synced" as const,
    itemCount: 2,
    segmentCount: 3,
    lastSyncedAt: 1_700_000_000,
    label: "Synced",
  };
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
    videoCount: null,
    linkedVideoCount: null,
    unavailableCount: null,
    captions: synced,
    comments: synced,
  };
}

type RailProps = ComponentProps<typeof CompactSourceRail>;

function railProps(overrides: Partial<RailProps> = {}): RailProps {
  const telegram = source();
  const youtube = source({
    id: 2,
    sourceType: "youtube",
    sourceSubtype: "video",
    accountId: null,
    externalId: "video-2",
    title: "Fallback video",
    telegramUsername: null,
  });
  const sourceGroup = group();
  return {
    sourceCatalog: [telegram, youtube],
    groups: [sourceGroup],
    sourceMetrics: {
      1: { id: 1, account_id: 7, source_type: "telegram", title: "Research channel", item_count: 12, last_synced_at: 1_700_000_000 },
      2: { id: 2, account_id: null, source_type: "youtube", title: "Rendered video", item_count: 3, last_synced_at: 1_700_000_000 },
    },
    loadingSourceCatalog: false,
    loadingGroups: false,
    railQuery: "",
    filteredSourceCatalog: [telegram, youtube],
    filteredGroups: [sourceGroup],
    workspaceSelection: { kind: "source", sourceId: 1 },
    syncingIds: {},
    deletingSourceIds: {},
    startingTakeoutSourceIds: {},
    startingMigratedHistorySourceIds: {},
    takeoutJobsBySource: {},
    takeoutRecoveryBySource: {},
    sourceJobsBySource: {},
    youtubeSummaries: { 2: youtubeSummary() },
    youtubeRuntimeStatus: null,
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
    ...overrides,
  };
}

describe("compact analysis source rail", () => {
  it("keeps the collapsed rail compact and source-scoped", () => {
    render(CompactSourceRail, { props: railProps() });

    expect(screen.getByRole("complementary")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Open source switcher" })).toBeTruthy();
    expect(screen.getAllByRole("button", { name: "Research channel" })).toHaveLength(3);
    expect(screen.getByLabelText("Quick source choices")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Rendered video" })).toBeTruthy();
    expect(screen.getAllByRole("button", { name: "Research channel" }).some((button) => button.getAttribute("aria-pressed") === "true")).toBe(true);
    expect(screen.getByRole("button", { name: "Research group. 1 sources" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Research group. 1 sources" }).getAttribute("aria-pressed")).toBe("false");
    expect(screen.getByRole("button", { name: "Sync Research channel" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "New source" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Manage sources" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Delete" })).toBeNull();
    expect(screen.queryByText(/Transcript unavailable/)).toBeNull();
    expect(screen.queryByRole("region", { name: "Source switcher panel" })).toBeNull();
  });

  it("passes migrated history action state through the compact rail", async () => {
    const onStartMigratedHistoryImport = vi.fn();
    const migrated = source({ migratedHistoryStatus: "available", migratedHistoryRowCount: 8 });
    const view = render(CompactSourceRail, {
      props: railProps({
        sourceCatalog: [migrated],
        filteredSourceCatalog: [migrated],
        onStartMigratedHistoryImport,
      }),
    });

    await fireEvent.click(screen.getByRole("button", { name: "Open source switcher" }));
    const action = screen.getByRole("button", { name: "Import migrated history" });
    expect((action as HTMLButtonElement).disabled).toBe(false);
    await fireEvent.click(action);
    expect(onStartMigratedHistoryImport).toHaveBeenCalledWith(1);

    await view.rerender(railProps({
      sourceCatalog: [migrated],
      filteredSourceCatalog: [migrated],
      startingMigratedHistorySourceIds: { 1: true },
      onStartMigratedHistoryImport,
    }));
    expect(screen.getByRole("button", { name: "Starting historical import..." })).toBeTruthy();
    expect((screen.getByRole("button", { name: "Starting historical import..." }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("keeps source and group switching callback-based", async () => {
    const onSelectSource = vi.fn();
    const onSelectGroup = vi.fn();
    render(CompactSourceRail, { props: railProps({ onSelectSource, onSelectGroup }) });

    const sourceButton = screen.getByRole("button", { name: "Rendered video" });
    await fireEvent.click(sourceButton);
    expect(onSelectSource).toHaveBeenCalledWith(2);
    expect(onSelectGroup).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: "Research group. 1 sources" }));
    expect(onSelectGroup).toHaveBeenCalledWith(20);
    expect(onSelectSource).toHaveBeenCalledTimes(1);
  });

  it("closes the expanded switcher after quick source or group selection", async () => {
    render(CompactSourceRail, { props: railProps() });

    await fireEvent.click(screen.getByRole("button", { name: "Open source switcher" }));
    await fireEvent.click(within(screen.getByRole("region", { name: "Source switcher panel" })).getByRole("button", { name: /Rendered video/ }));
    expect(screen.queryByRole("region", { name: "Source switcher panel" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Open source switcher" }));
    await fireEvent.click(within(screen.getByRole("region", { name: "Source switcher panel" })).getByRole("button", { name: /Research group/ }));
    expect(screen.queryByRole("region", { name: "Source switcher panel" })).toBeNull();
  });

  it("keeps destructive source deletion out of the compact rail but available in the expanded panel", async () => {
    const onDeleteSource = vi.fn();
    const onOpenSourceManager = vi.fn();
    render(CompactSourceRail, { props: railProps({ onDeleteSource, onOpenSourceManager }) });

    expect(screen.queryByRole("button", { name: "Delete" })).toBeNull();
    expect(onDeleteSource).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: "Open source switcher" }));
    const panel = screen.getByRole("region", { name: "Source switcher panel" });
    await fireEvent.click(within(panel).getAllByText("Source operations")[0]);
    expect(within(panel).getAllByRole("button", { name: "Delete" })).toHaveLength(2);
    const deleteButton = within(panel).getAllByRole("button", { name: "Delete" })[0];
    expect((deleteButton as HTMLButtonElement).disabled).toBe(false);
    await fireEvent.click(deleteButton);
    expect(onDeleteSource.mock.calls[0]?.[0]?.id).toBe(1);
    expect(within(panel).getByRole("button", { name: "Manage sources" })).toBeTruthy();
    await fireEvent.click(within(panel).getByRole("button", { name: "Manage sources" }));
    expect(onOpenSourceManager).toHaveBeenCalledOnce();
  });

  it("keeps icon-only controls accessible without hover-only status", async () => {
    const props = railProps({ runtimeBadge: () => "Connection needs attention" });
    render(CompactSourceRail, { props });

    const sourceButtons = screen.getAllByRole("button", { name: "Research channel. Connection needs attention" });
    expect(sourceButtons).toHaveLength(2);
    expect(sourceButtons.some((button) => button.getAttribute("aria-pressed") === "true")).toBe(true);
    expect(screen.getByRole("button", { name: "Research group. 1 sources" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Research group. 1 sources" }).getAttribute("aria-pressed")).toBe("false");
    const trigger = screen.getByRole("button", { name: "Open source switcher" });
    expect(trigger.getAttribute("aria-expanded")).toBe("false");
    await fireEvent.click(trigger);
    expect(trigger.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByLabelText("Connection needs attention")).toBeTruthy();
  });

  it("uses a compact mobile source context bar", () => {
    render(CompactSourceRail, { props: railProps() });

    expect(screen.getAllByRole("button", { name: "Research channel" })).toHaveLength(3);
    expect(screen.getByLabelText("Quick source choices")).toBeTruthy();
  });
});

describe("analysis redesign final route contract", () => {
  it("keeps the collapsed rail source-scoped and quiet", () => {
    render(CompactSourceRail, { props: railProps() });

    expect(screen.getByRole("complementary")).toBeTruthy();
    expect(screen.getAllByRole("button", { name: "Research channel" })).toHaveLength(3);
    expect(screen.getByRole("button", { name: "Rendered video" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Research group. 1 sources" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Open source switcher" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync Research channel" })).toBeTruthy();
    expect(screen.queryByRole("link", { name: /Settings/ })).toBeNull();
    expect(screen.queryByRole("link", { name: /Accounts/ })).toBeNull();
    expect(screen.queryByRole("heading", { name: "Workspace" })).toBeNull();
    expect(screen.queryByText("Research context")).toBeNull();
    expect(screen.queryByRole("button", { name: "Manage sources" })).toBeNull();
    expect(screen.queryByText(/Transcript unavailable/)).toBeNull();
    expect(screen.queryByText(/Comments unavailable/)).toBeNull();
    expect(screen.queryByText("Source operations")).toBeNull();
    expect(screen.queryByRole("button", { name: "Delete" })).toBeNull();
  });
});
