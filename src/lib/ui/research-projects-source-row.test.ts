import { describe, expect, it } from "vitest";
import { buildSourceRow, sourceSyncStatusLabel } from "./research-projects-source-row";
import type { ProjectSourceRecord } from "$lib/types/projects";

function record(overrides: Partial<ProjectSourceRecord> = {}): ProjectSourceRecord {
  return {
    project_id: 1,
    source_id: 10,
    provider: "telegram",
    source_subtype: "channel",
    title: "ФинБеларусь",
    subtitle: null,
    item_count: 339,
    added_at: 100,
    last_synced_at: null,
    sync_status: "active",
    handle: "@finbelarus",
    ...overrides,
  };
}

describe("sourceSyncStatusLabel", () => {
  it("maps catalog status to the design label (syncing → sync)", () => {
    expect(sourceSyncStatusLabel("active")).toBe("active");
    expect(sourceSyncStatusLabel("syncing")).toBe("sync");
    expect(sourceSyncStatusLabel("error")).toBe("error");
    expect(sourceSyncStatusLabel("unavailable")).toBe("unavailable");
  });
});

describe("buildSourceRow", () => {
  it("builds a source row from a project source record", () => {
    const row = buildSourceRow(record({ item_count: 4317, sync_status: "syncing" }));

    expect(row.sourceId).toBe(10);
    expect(row.title).toBe("ФинБеларусь");
    expect(row.handle).toBe("@finbelarus");
    expect(row.provider).toBe("telegram");
    expect(row.typeLabel).toBe("telegram");
    expect(row.materialsLabel).toBe("4 317");
    expect(row.syncStatus).toBe("syncing");
    expect(row.statusLabel).toBe("sync");
  });

  it("formats material counts with grouped thousands", () => {
    const materials = (count: number) =>
      buildSourceRow(record({ item_count: count })).materialsLabel;

    expect(materials(339)).toBe("339");
    expect(materials(1011)).toBe("1 011");
    expect(materials(76070)).toBe("76 070");
  });

  it("falls back to a generated title when none is set", () => {
    expect(buildSourceRow(record({ title: null, source_id: 42 })).title).toBe("Источник #42");
  });

  it("exposes the raw last-synced timestamp for grid date formatting", () => {
    expect(buildSourceRow(record({ last_synced_at: 1_717_360_740 })).lastSyncedAt).toBe(
      1_717_360_740,
    );
    expect(buildSourceRow(record({ last_synced_at: null })).lastSyncedAt).toBeNull();
  });
});
