import { beforeEach, describe, expect, it, vi } from "vitest";
import { listLibraryCatalog, listLibrarySources } from "./library-sources";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("library source api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("lists enriched library source records", async () => {
    const records = [
      {
        source_id: 1,
        provider: "youtube",
        source_subtype: "video",
        account_id: null,
        external_id: "vid-1",
        title: "Video title",
        subtitle: "Channel title",
        canonical_url: "https://youtu.be/vid-1",
        created_at: 1_717_000_000,
        last_synced_at: 1_717_000_100,
        item_count: 12,
        project_count: 2,
        youtube: {
          video_form: "short",
          duration_seconds: 45,
          playlist_video_count: null,
          channel_title: "Channel title",
          availability_status: "available",
        },
        telegram: null,
      },
    ];
    invokeMock.mockResolvedValueOnce(records);

    await expect(listLibrarySources()).resolves.toEqual(records);

    expect(invokeMock).toHaveBeenLastCalledWith("list_library_sources");
  });

  it("lists backend-owned library catalog records", async () => {
    const response = {
      sources: [
        {
          source: {
            source_id: 1,
            provider: "youtube",
            source_subtype: "video",
            account_id: null,
            external_id: "vid-1",
            title: "Video title",
            subtitle: "Channel title",
            canonical_url: "https://youtu.be/vid-1",
            created_at: 1_717_000_000,
            last_synced_at: 1_717_000_100,
            item_count: 12,
            project_count: 2,
            youtube: null,
            telegram: null,
          },
          latest_job: null,
          status: "active",
          status_detail: null,
          capabilities: {
            can_refresh_source: true,
            can_delete: false,
            can_edit: false,
            can_connect_to_project: true,
          },
          disabled_reasons: {
            refresh_source: null,
            delete: "Source 1 is used by 2 project(s). Remove it from projects first.",
            edit: "Source editing is not available yet.",
            connect_to_project: null,
          },
        },
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
    };
    invokeMock.mockResolvedValueOnce(response);

    await expect(listLibraryCatalog()).resolves.toEqual(response);

    expect(invokeMock).toHaveBeenLastCalledWith("list_library_catalog");
  });
});
