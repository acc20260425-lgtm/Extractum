import { describe, expect, it, vi } from "vitest";
import type { LibrarySourceRecord } from "$lib/types/library-sources";
import type { SourceJobRecord } from "$lib/types/sources";
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

function job(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 1,
    related_source_id: null,
    job_type: "youtube_video_full_sync",
    status: "running",
    message: "Syncing",
    progress_current: 1,
    progress_total: 2,
    started_at: 1_717_000_100,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

function createHarness(initial: Partial<LibraryCatalogWorkflowState> = {}) {
  const state: LibraryCatalogWorkflowState = {
    sourceRecords: [],
    sourceJobs: [],
    sources: [],
    loading: false,
    status: "",
    ...initial,
  };
  const deps = {
    getState: () => state,
    patch: vi.fn((patch: Partial<LibraryCatalogWorkflowState>) => Object.assign(state, patch)),
    listSources: vi.fn(),
    listSourceJobs: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };
  return { state, deps, workflow: createLibraryCatalogWorkflow(deps) };
}

describe("library catalog workflow", () => {
  it("loads library source records and source jobs into catalog rows", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listSources.mockResolvedValueOnce([record()]);
    deps.listSourceJobs.mockResolvedValueOnce([job()]);

    await workflow.loadLibrary();

    expect(state.sourceRecords).toHaveLength(1);
    expect(state.sourceJobs).toHaveLength(1);
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
          youtube: null,
          telegram: { account_id: null },
        },
      ],
    });
    deps.listSources.mockRejectedValueOnce(new Error("offline"));

    await workflow.loadLibrary();

    expect(state.sources.map((source) => source.id)).toEqual(["source:9"]);
    expect(state.status).toBe("Error loading library sources: Error: offline");
    expect(state.loading).toBe(false);
  });
});
