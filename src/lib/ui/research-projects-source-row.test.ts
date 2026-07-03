import { describe, expect, it } from "vitest";
import {
  buildSourceGridRows,
  buildSourceRow,
  compareMaterialsLabels,
  compareNullableTimestamps,
  compareRuStrings,
  sourceGridColumns,
  sourceSyncStatusLabel,
} from "./research-projects-source-row";
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

describe("sourceGridColumns", () => {
  it("defines the v10 source columns with Russian headers", () => {
    const columns = sourceGridColumns();

    expect(columns.map((column) => column.id)).toEqual([
      "title",
      "typeLabel",
      "materialsLabel",
      "lastSyncedAt",
      "statusLabel",
    ]);
    expect(columns.map((column) => column.header)).toEqual([
      "Источник",
      "Тип",
      "Материалы",
      "Последний сбор",
      "Статус",
    ]);
  });

  it("formats the last-collected column as a date-time", () => {
    const lastSync = sourceGridColumns().find((column) => column.id === "lastSyncedAt");

    expect(lastSync?.dateTimeFormat).toBe("datetime");
  });
});

describe("buildSourceGridRows", () => {
  it("maps records to grid rows keyed by a string source id", () => {
    const rows = buildSourceGridRows([
      record({ source_id: 10, title: "ФинБеларусь" }),
      record({ source_id: 11, title: "СтатусБанк" }),
    ]);

    expect(rows.map((row) => row.id)).toEqual(["10", "11"]);
    expect(rows[0].title).toBe("ФинБеларусь");
    expect(rows[0].sourceId).toBe(10);
  });
});

describe("sort comparators", () => {
  it("compareRuStrings orders Cyrillic case-insensitively", () => {
    expect(compareRuStrings("аист", "Бобр")).toBeLessThan(0);
    expect(compareRuStrings("Яблоко", "аист")).toBeGreaterThan(0);
    expect(compareRuStrings("ФИН", "фин")).toBe(0);
  });

  it("compareMaterialsLabels compares formatted numbers numerically", () => {
    expect(compareMaterialsLabels("4 317", "339")).toBeGreaterThan(0);
    expect(compareMaterialsLabels("76 070", "4 317")).toBeGreaterThan(0);
    expect(compareMaterialsLabels("10", "10")).toBe(0);
  });

  it("compareNullableTimestamps treats null as the oldest value", () => {
    expect(compareNullableTimestamps(null, 100)).toBeLessThan(0);
    expect(compareNullableTimestamps(100, null)).toBeGreaterThan(0);
    expect(compareNullableTimestamps(null, null)).toBe(0);
    expect(compareNullableTimestamps(200, 100)).toBeGreaterThan(0);
  });

  it("sourceGridColumns enables sorting on every data column", () => {
    const columns = sourceGridColumns();
    const byId = new Map(columns.map((c) => [String(c.id), c]));
    expect(byId.get("title")?.sort).toBe(compareRuStrings);
    expect(byId.get("typeLabel")?.sort).toBe(true);
    expect(byId.get("materialsLabel")?.sort).toBe(compareMaterialsLabels);
    expect(byId.get("lastSyncedAt")?.sort).toBe(compareNullableTimestamps);
    expect(byId.get("statusLabel")?.sort).toBe(true);
  });
});
