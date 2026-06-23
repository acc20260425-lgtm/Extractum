import { describe, expect, it, vi } from "vitest";
import type { LibraryCatalogRecord, LibrarySourceRecord } from "$lib/types/library-sources";
import {
  createLibraryCatalogWorkflow,
  type LibraryCatalogWorkflowState,
} from "./library-catalog-workflow";

function record(overrides: Partial<LibrarySourceRecord> = {}): LibrarySourceRecord {
  return {
    source_id: 1,
    provider: "youtube",
    source_subtype: "video",
    account_id: null,
    external_id: "vid-1",
    title: "Video title",
    subtitle: "Channel title",
    canonical_url: "https://youtu.be/vid-1",
    created_at: 1_716_000_000,
    last_synced_at: 1_717_000_000,
    item_count: 10,
    project_count: 2,
    youtube: {
      video_form: "longform",
      duration_seconds: 120,
      playlist_video_count: null,
      channel_title: "Channel title",
      availability_status: "available",
    },
    telegram: null,
    ...overrides,
  };
}

function catalogRecord(overrides: Partial<LibraryCatalogRecord> = {}): LibraryCatalogRecord {
  return {
    source: record(),
    latest_job: null,
    status: "active",
    status_detail: null,
    capabilities: {
      can_refresh_source: true,
      can_delete: true,
      can_edit: false,
      can_connect_to_project: true,
    },
    disabled_reasons: {
      refresh_source: null,
      delete: null,
      edit: "Source editing is not available yet.",
      connect_to_project: null,
    },
    ...overrides,
  };
}

function createHarness(initial: Partial<LibraryCatalogWorkflowState> = {}) {
  const state: LibraryCatalogWorkflowState = {
    catalogRecords: [],
    filterCounts: [],
    sources: [],
    loading: false,
    status: "",
    ...initial,
  };
  const deps = {
    getState: () => state,
    patch: vi.fn((patch: Partial<LibraryCatalogWorkflowState>) => Object.assign(state, patch)),
    listCatalog: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };
  return { state, deps, workflow: createLibraryCatalogWorkflow(deps) };
}

describe("library catalog workflow", () => {
  it("loads backend catalog records into catalog rows", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listCatalog.mockResolvedValueOnce({
      sources: [
        catalogRecord({
          source: record(),
          status: "syncing",
          status_detail: "Syncing",
        }),
      ],
      filter_counts: [
        {
          provider: "youtube",
          source_subtype: "video",
          count: 1,
          disabled: false,
          disabled_reason: null,
        },
      ],
    });

    await workflow.loadLibrary();

    expect(deps.listCatalog).toHaveBeenCalledTimes(1);
    expect(state.catalogRecords).toHaveLength(1);
    expect(state.filterCounts).toHaveLength(1);
    expect(state.sources[0]).toEqual(
      expect.objectContaining({
        sourceId: 1,
        title: "Video title",
        status: "syncing",
      }),
    );
    expect(state.loading).toBe(false);
    expect(state.status).toBe("");
  });

  it("keeps previous rows and reports a load error", async () => {
    const { state, deps, workflow } = createHarness({
      sources: [
        {
          id: "source:9",
          sourceId: 9,
          provider: "telegram",
          sourceSubtype: "group",
          title: "Cached",
          subtitle: null,
          typeLabel: "Telegram / Group",
          status: "active",
          statusDetail: null,
          projectCount: 0,
          itemCount: 0,
          itemCountLabel: "0 items",
          addedAtLabel: "Unknown",
          lastSyncedLabel: "Never",
          canonicalUrl: null,
          externalId: null,
          createdAt: null,
          lastSyncedAt: null,
          youtube: null,
          telegram: { account_id: null },
        },
      ],
    });
    deps.listCatalog.mockRejectedValueOnce(new Error("offline"));

    await workflow.loadLibrary();

    expect(state.sources.map((source) => source.id)).toEqual(["source:9"]);
    expect(state.status).toBe("Error loading library catalog: Error: offline");
    expect(state.loading).toBe(false);
  });
});
