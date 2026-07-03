import { describe, expect, it } from "vitest";
import {
  buildSourceFilterChips,
  countActiveSourceFilters,
  emptySourceFilters,
  filterProjectSources,
  removeSourceFilterChip,
  type SourceFilters,
} from "./research-projects-source-filters";
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
    added_at: 0,
    last_synced_at: Date.UTC(2026, 5, 2, 12, 0, 0) / 1000, // 2026-06-02 12:00 UTC
    sync_status: "active",
    handle: "@finbelarus",
    ...overrides,
  };
}

function filters(overrides: Partial<SourceFilters> = {}): SourceFilters {
  return { ...emptySourceFilters(), ...overrides };
}

describe("filterProjectSources", () => {
  it("returns the same array for empty filters", () => {
    const records = [record()];
    expect(filterProjectSources(records, emptySourceFilters())).toEqual(records);
  });

  it("filters by query over title and handle, case-insensitively", () => {
    const records = [
      record({ source_id: 1, title: "ФинБеларусь", handle: "@fin" }),
      record({ source_id: 2, title: "WhiteBird", handle: "@whitebird_io" }),
    ];
    expect(filterProjectSources(records, filters({ query: "финбел" })).map((r) => r.source_id)).toEqual([1]);
    expect(filterProjectSources(records, filters({ query: "BIRD_io" })).map((r) => r.source_id)).toEqual([2]);
  });

  it("filters by provider types and sync statuses", () => {
    const records = [
      record({ source_id: 1, provider: "telegram", sync_status: "active" }),
      record({ source_id: 2, provider: "youtube", sync_status: "error" }),
    ];
    expect(filterProjectSources(records, filters({ types: ["youtube"] })).map((r) => r.source_id)).toEqual([2]);
    expect(filterProjectSources(records, filters({ statuses: ["active"] })).map((r) => r.source_id)).toEqual([1]);
  });

  it("filters by materials range", () => {
    const records = [
      record({ source_id: 1, item_count: 10 }),
      record({ source_id: 2, item_count: 500 }),
    ];
    expect(filterProjectSources(records, filters({ materialsMin: 100 })).map((r) => r.source_id)).toEqual([2]);
    expect(filterProjectSources(records, filters({ materialsMax: 100 })).map((r) => r.source_id)).toEqual([1]);
  });

  it("filters by last-synced date range and drops null dates when range set", () => {
    const day = (iso: string) => new Date(`${iso}T12:00:00`).getTime() / 1000;
    const records = [
      record({ source_id: 1, last_synced_at: day("2026-05-10") }),
      record({ source_id: 2, last_synced_at: day("2026-06-20") }),
      record({ source_id: 3, last_synced_at: null }),
    ];
    expect(
      filterProjectSources(records, filters({ syncedFrom: "2026-06-01" })).map((r) => r.source_id),
    ).toEqual([2]);
    expect(
      filterProjectSources(records, filters({ syncedTo: "2026-05-31" })).map((r) => r.source_id),
    ).toEqual([1]);
    // границы включительно: запись, синхронизированная в тот же день, проходит
    expect(
      filterProjectSources(
        records,
        filters({ syncedFrom: "2026-05-10", syncedTo: "2026-05-10" }),
      ).map((r) => r.source_id),
    ).toEqual([1]);
  });
});

describe("chips", () => {
  it("counts active filters and builds chips with dots", () => {
    const f = filters({
      query: "фин",
      types: ["telegram"],
      statuses: ["error"],
      materialsMin: 10,
      syncedFrom: "2026-05-01",
    });
    expect(countActiveSourceFilters(f)).toBe(5);
    const chips = buildSourceFilterChips(f);
    expect(chips.map((c) => c.key)).toEqual([
      "query",
      "type:telegram",
      "status:error",
      "materials",
      "period",
    ]);
    expect(chips[0].label).toBe("Источник: фин");
    expect(chips[1].dot).toBe("var(--extractum-provider-telegram)");
    expect(chips[2].dot).toBe("var(--extractum-danger)");
    expect(chips[3].label).toBe("Материалы: 10–∞");
    expect(chips[4].label).toBe("Период: 01.05.2026–…");
  });

  it("removeSourceFilterChip resets only the matching part", () => {
    const f = filters({ query: "x", types: ["telegram", "youtube"], materialsMin: 1, materialsMax: 2 });
    expect(removeSourceFilterChip(f, "query").query).toBe("");
    expect(removeSourceFilterChip(f, "type:telegram").types).toEqual(["youtube"]);
    const noMaterials = removeSourceFilterChip(f, "materials");
    expect(noMaterials.materialsMin).toBeNull();
    expect(noMaterials.materialsMax).toBeNull();
    // остальное не тронуто
    expect(noMaterials.query).toBe("x");
  });

  it("empty filters produce no chips and zero count", () => {
    expect(buildSourceFilterChips(emptySourceFilters())).toEqual([]);
    expect(countActiveSourceFilters(emptySourceFilters())).toBe(0);
  });
});
