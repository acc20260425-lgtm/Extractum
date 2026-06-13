import { describe, expect, it } from "vitest";
import type { SourceJobRecord } from "$lib/types/sources";
import type { LibrarySourceRecord } from "$lib/types/library-sources";
import {
  LIBRARY_CATALOG_ALL_FILTER_ID,
  buildLibraryCatalogFilterTree,
  buildLibraryCatalogSourcesView,
  filterLibraryCatalogSources,
  reconcileLibraryCatalogSourceSelection,
} from "./library-catalog-model";

function record(overrides: Partial<LibrarySourceRecord> = {}): LibrarySourceRecord {
  return {
    source_id: 1,
    provider: "telegram",
    source_subtype: "supergroup",
    account_id: 10,
    external_id: "-1001",
    title: "Radar BPLA",
    subtitle: "Account #10",
    canonical_url: null,
    created_at: 1_716_000_000,
    last_synced_at: 1_717_000_000,
    item_count: 128,
    project_count: 2,
    youtube: null,
    telegram: { account_id: 10 },
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

describe("library catalog model", () => {
  it("maps source metadata into catalog rows without project-connect state", () => {
    const [row] = buildLibraryCatalogSourcesView(
      [
        record({
          source_id: 3,
          provider: "youtube",
          source_subtype: "video",
          title: "Alpha Drones",
          subtitle: null,
          canonical_url: "https://youtu.be/alpha",
          external_id: "alpha",
          item_count: 7,
          project_count: 1,
          youtube: {
            video_form: "longform",
            duration_seconds: 367,
            playlist_video_count: null,
            channel_title: null,
            availability_status: "available",
          },
          telegram: null,
        }),
      ],
      [],
    );

    expect(row).toEqual(
      expect.objectContaining({
        id: "source:3",
        sourceId: 3,
        provider: "youtube",
        sourceSubtype: "video",
        title: "Alpha Drones",
        subtitle: null,
        typeLabel: "YouTube / Video",
        status: "active",
        projectCount: 1,
        itemCount: 7,
        itemCountLabel: "7 items",
        addedAtLabel: expect.any(String),
        lastSyncedLabel: expect.any(String),
        canonicalUrl: "https://youtu.be/alpha",
        externalId: "alpha",
        youtube: {
          video_form: "longform",
          duration_seconds: 367,
          playlist_video_count: null,
          channel_title: null,
          availability_status: "available",
        },
        telegram: null,
      }),
    );
  });

  it("derives syncing and failed status from the latest source job", () => {
    const rows = buildLibraryCatalogSourcesView(
      [
        record({ source_id: 3, provider: "youtube", source_subtype: "video", title: "Running" }),
        record({ source_id: 4, provider: "youtube", source_subtype: "video", title: "Failed" }),
      ],
      [
        job({ source_id: 3, status: "running", started_at: 20 }),
        job({ source_id: 4, status: "failed", error: "Quota", started_at: 10 }),
      ],
    );

    expect(rows.find((row) => row.sourceId === 3)?.status).toBe("syncing");
    expect(rows.find((row) => row.sourceId === 4)?.status).toBe("error");
    expect(rows.find((row) => row.sourceId === 4)?.statusDetail).toBe("Quota");
  });

  it("builds active subtype filters for YouTube and Telegram while keeping YouTube channels disabled", () => {
    const rows = buildLibraryCatalogSourcesView(
      [
        record({ source_id: 1, provider: "youtube", source_subtype: "video", title: "Video" }),
        record({ source_id: 2, provider: "youtube", source_subtype: "playlist", title: "Playlist" }),
        record({ source_id: 3, provider: "telegram", source_subtype: "channel", title: "Channel" }),
        record({
          source_id: 4,
          provider: "telegram",
          source_subtype: "supergroup",
          title: "Supergroup",
        }),
        record({ source_id: 5, provider: "telegram", source_subtype: "group", title: "Group" }),
      ],
      [],
    );

    expect(buildLibraryCatalogFilterTree(rows)).toEqual([
      expect.objectContaining({ id: "all", count: 5 }),
      expect.objectContaining({
        id: "provider:youtube",
        count: 2,
        data: [
          expect.objectContaining({
            id: "provider:youtube/subtype:video",
            count: 1,
            disabled: false,
          }),
          expect.objectContaining({
            id: "provider:youtube/subtype:playlist",
            count: 1,
            disabled: false,
          }),
          expect.objectContaining({
            id: "provider:youtube/subtype:channel",
            count: 0,
            disabled: true,
          }),
        ],
      }),
      expect.objectContaining({
        id: "provider:telegram",
        count: 3,
        data: [
          expect.objectContaining({
            id: "provider:telegram/subtype:channel",
            count: 1,
            disabled: false,
          }),
          expect.objectContaining({
            id: "provider:telegram/subtype:supergroup",
            count: 1,
            disabled: false,
          }),
          expect.objectContaining({
            id: "provider:telegram/subtype:group",
            count: 1,
            disabled: false,
          }),
        ],
      }),
    ]);
  });

  it("filters by selected provider subtype and search query", () => {
    const rows = buildLibraryCatalogSourcesView(
      [
        record({ source_id: 1, provider: "youtube", source_subtype: "video", title: "Alpha Video" }),
        record({
          source_id: 2,
          provider: "youtube",
          source_subtype: "playlist",
          title: "Alpha Playlist",
        }),
        record({
          source_id: 3,
          provider: "telegram",
          source_subtype: "channel",
          title: "Alpha Channel",
        }),
      ],
      [],
    );

    expect(
      filterLibraryCatalogSources(rows, {
        filterId: LIBRARY_CATALOG_ALL_FILTER_ID,
        query: "alpha",
      }).map((row) => row.id),
    ).toEqual(["source:1", "source:2", "source:3"]);
    expect(
      filterLibraryCatalogSources(rows, {
        filterId: "provider:youtube/subtype:video",
        query: "",
      }).map((row) => row.id),
    ).toEqual(["source:1"]);
    expect(
      filterLibraryCatalogSources(rows, {
        filterId: "provider:telegram/subtype:channel",
        query: "",
      }).map((row) => row.id),
    ).toEqual(["source:3"]);
  });

  it("reconciles selected rows after filtering", () => {
    const rows = buildLibraryCatalogSourcesView(
      [record({ source_id: 1, title: "First" }), record({ source_id: 2, title: "Second" })],
      [],
    );

    expect(reconcileLibraryCatalogSourceSelection(rows, "source:2")).toBe("source:2");
    expect(reconcileLibraryCatalogSourceSelection([rows[0]], "source:2")).toBe("source:1");
    expect(reconcileLibraryCatalogSourceSelection([], "source:2")).toBeNull();
  });
});
