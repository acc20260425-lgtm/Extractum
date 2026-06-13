import { describe, expect, it } from "vitest";
import {
  LIBRARY_ALL_FILTER_ID,
  YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
  buildLibraryFilterTree,
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  buildSourceGroupUpdateInput,
  connectableSelection,
  filterLibrarySourcesForLibrary,
  filterLibrarySources,
  reconcileLibrarySourceSelection,
  type LibrarySourceView,
} from "./research-projects-model";
import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
import type { SourceJobRecord } from "$lib/types/sources";

function source(overrides: Partial<AnalysisSourceOption> = {}): AnalysisSourceOption {
  return {
    id: 1,
    account_id: 10,
    source_type: "telegram",
    title: "Radar BPLA",
    item_count: 128,
    last_synced_at: 1_717_000_000,
    ...overrides,
  };
}

function job(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 3,
    related_source_id: null,
    job_type: "youtube_video_full_sync",
    status: "running",
    message: "Syncing",
    progress_current: 1,
    progress_total: 3,
    started_at: 1_717_000_100,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 100,
    name: "Рынок БПЛА",
    source_type: "telegram",
    members: [{ source_id: 1, source_title: "Radar BPLA", item_count: 128 }],
    created_at: 1_716_000_000,
    updated_at: 1_717_000_000,
    ...overrides,
  };
}

describe("research projects model", () => {
  it("projects source groups as research projects without leaking source-group wording", () => {
    const projects = buildResearchProjectsView([group()], []);

    expect(projects).toEqual([
      expect.objectContaining({
        id: "source-group:100",
        title: "Рынок БПЛА",
        sourceCount: 1,
        materialCount: 128,
        backing: { kind: "source_group", groupId: 100, sourceType: "telegram" },
        status: "ready",
      }),
    ]);
  });

  it("marks already connected and unsupported library rows as non-connectable", () => {
    const [telegram, rss] = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "rss", title: "Новости БПЛА" }),
      ],
      [group()],
      "source-group:100",
    );

    expect(telegram.alreadyConnected).toBe(true);
    expect(telegram.connectable).toBe(false);
    expect(telegram.disabledReason).toBe("Источник уже подключен к этому проекту.");
    expect(rss.provider).toBe("rss");
    expect(rss.connectable).toBe(false);
    expect(rss.disabledReason).toBe(
      "Подключение RSS к проектам будет доступно после миграции библиотеки.",
    );
  });

  it("marks active or failed source jobs on library rows before generic provider decisions", () => {
    const [syncing, failed] = buildLibrarySourcesView(
      [
        source({ id: 3, source_type: "youtube", title: "Alpha Drones" }),
        source({ id: 4, source_type: "youtube", title: "Broken Channel" }),
      ],
      [group({ source_type: "youtube", members: [] })],
      "source-group:100",
      [
        job({ source_id: 3, status: "running" }),
        job({ source_id: 4, status: "failed", error: "API quota exceeded" }),
      ],
    );

    expect(syncing.status).toBe("syncing");
    expect(syncing.connectable).toBe(false);
    expect(syncing.disabledReason).toBe("Источник сейчас синхронизируется.");
    expect(failed.status).toBe("error");
    expect(failed.disabledReason).toBe(
      "Последняя синхронизация завершилась ошибкой: API quota exceeded",
    );
  });

  it("filters the library by search text and provider chips", () => {
    const rows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
      ],
      [],
      null,
    );

    expect(filterLibrarySources(rows, { query: "alpha", providers: [] }).map((row) => row.id))
      .toEqual(["source:2"]);
    expect(filterLibrarySources(rows, { query: "", providers: ["telegram"] }).map((row) => row.id))
      .toEqual(["source:1"]);
  });

  it("builds the Library filter tree with disabled YouTube subtype rows", () => {
    const rows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
        source({ id: 3, source_type: "youtube", title: "Research Playlist" }),
      ],
      [],
      null,
    );

    expect(buildLibraryFilterTree(rows)).toEqual([
      expect.objectContaining({ id: "all", label: "All sources", count: 3 }),
      expect.objectContaining({
        id: "provider:youtube",
        label: "YouTube",
        provider: "youtube",
        count: 2,
        data: [
          {
            id: "provider:youtube/subtype:video",
            label: "Videos",
            provider: "youtube",
            subtype: "video",
            count: 0,
            disabled: true,
            disabledReason: YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
          },
          {
            id: "provider:youtube/subtype:playlist",
            label: "Playlists",
            provider: "youtube",
            subtype: "playlist",
            count: 0,
            disabled: true,
            disabledReason: YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
          },
          {
            id: "provider:youtube/subtype:channel",
            label: "Channels",
            provider: "youtube",
            subtype: "channel",
            count: 0,
            disabled: true,
            disabledReason: YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
          },
        ],
      }),
      expect.objectContaining({ id: "provider:telegram", label: "Telegram", provider: "telegram", count: 1 }),
    ]);
  });

  it("filters Library sources by selected tree row and search query", () => {
    const rows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
        source({ id: 3, source_type: "youtube", title: "Research Playlist" }),
      ],
      [],
      null,
    );

    expect(filterLibrarySourcesForLibrary(rows, { filterId: LIBRARY_ALL_FILTER_ID, query: "alpha" }).map((row) => row.id))
      .toEqual(["source:2"]);
    expect(filterLibrarySourcesForLibrary(rows, { filterId: "provider:youtube", query: "" }).map((row) => row.id))
      .toEqual(["source:2", "source:3"]);
    expect(filterLibrarySourcesForLibrary(rows, { filterId: "provider:youtube/subtype:video", query: "" }))
      .toEqual([]);
  });

  it("reconciles selected Library source with the visible rows", () => {
    const rows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
      ],
      [],
      null,
    );

    expect(reconcileLibrarySourceSelection(rows, "source:2")).toBe("source:2");
    expect(reconcileLibrarySourceSelection([rows[0]], "source:2")).toBe("source:1");
    expect(reconcileLibrarySourceSelection([], "source:2")).toBeNull();
  });

  it("counts only connectable selected rows", () => {
    const rows: LibrarySourceView[] = [
      {
        id: "source:1",
        sourceId: 1,
        provider: "telegram",
        title: "Connectable",
        subtitle: null,
        projectCount: 0,
        lastCollectedLabel: null,
        localCopyLabel: "10 материалов",
        status: "active",
        disabledReason: null,
        alreadyConnected: false,
        connectable: true,
      },
      {
        id: "source:2",
        sourceId: 2,
        provider: "rss",
        title: "Unsupported",
        subtitle: null,
        projectCount: 0,
        lastCollectedLabel: null,
        localCopyLabel: "5 материалов",
        status: "unavailable",
        disabledReason: "RSS is not persistable.",
        alreadyConnected: false,
        connectable: false,
      },
    ];

    expect(connectableSelection(rows, new Set(["source:1", "source:2"]))).toEqual([rows[0]]);
  });

  it("builds a provider-safe source-group update command", () => {
    const project = buildResearchProjectsView([group()], [])[0];
    const libraryRows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 3, source_type: "telegram", title: "Drone News" }),
        source({ id: 4, source_type: "youtube", title: "Alpha Drones" }),
      ],
      [group()],
      "source-group:100",
    );

    expect(buildSourceGroupUpdateInput(project, group(), new Set(["source:3", "source:4"]), libraryRows))
      .toEqual({
        ok: true,
        input: {
          groupId: 100,
          name: "Рынок БПЛА",
          sourceType: "telegram",
          sourceIds: [1, 3],
        },
        connectedCount: 1,
        refusedCount: 1,
      });
  });

  it("renders project source links from already connected library rows", () => {
    const rows = buildLibrarySourcesView([source()], [group()], "source-group:100");

    expect(buildProjectSourceLinksView("source-group:100", rows)).toEqual([
      expect.objectContaining({
        projectId: "source-group:100",
        sourceId: "source:1",
        provider: "telegram",
        connectionStatus: "connected",
      }),
    ]);
  });
});
