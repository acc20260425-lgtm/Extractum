import { describe, expect, it } from "vitest";
import { buildPeriodPresets } from "./research-projects-period";

const at = (iso: string) => Math.floor(Date.parse(iso) / 1000);

describe("buildPeriodPresets", () => {
  it("returns an empty list when the project has no data range", () => {
    expect(buildPeriodPresets({ from: null, to: null }, at("2025-06-01T00:00:00Z"))).toEqual([]);
  });

  it("includes a full-range preset spanning the data extent", () => {
    const range = { from: at("2024-03-03T00:00:00Z"), to: at("2025-06-28T00:00:00Z") };

    const presets = buildPeriodPresets(range, at("2025-06-28T12:00:00Z"));

    expect(presets[0]).toEqual({
      id: "all",
      label: "Весь период",
      from: range.from,
      to: range.to,
    });
  });

  it("adds a clamped preset for each calendar year, newest first", () => {
    const range = { from: at("2024-03-03T00:00:00Z"), to: at("2025-06-28T00:00:00Z") };

    const years = buildPeriodPresets(range, at("2025-06-28T12:00:00Z")).filter((preset) =>
      preset.id.startsWith("year:"),
    );

    expect(years.map((preset) => preset.label)).toEqual(["2025", "2024"]);

    const y2024 = years.find((preset) => preset.label === "2024");
    expect(y2024?.from).toBe(range.from); // clamped to data start
    expect(y2024?.to).toBe(at("2024-12-31T23:59:59Z"));

    const y2025 = years.find((preset) => preset.label === "2025");
    expect(y2025?.from).toBe(at("2025-01-01T00:00:00Z"));
    expect(y2025?.to).toBe(range.to); // clamped to data end
  });

  it("adds last-N-days presets anchored to the latest data", () => {
    const range = { from: at("2024-03-03T00:00:00Z"), to: at("2025-06-28T00:00:00Z") };

    const last30 = buildPeriodPresets(range, at("2025-06-28T12:00:00Z")).find(
      (preset) => preset.id === "last:30",
    );

    expect(last30).toEqual({
      id: "last:30",
      label: "Последние 30 дней",
      from: at("2025-05-29T00:00:00Z"),
      to: range.to,
    });
  });

  it("clamps a last-N-days window to the data start when data is shorter", () => {
    const range = { from: at("2025-06-20T00:00:00Z"), to: at("2025-06-28T00:00:00Z") };

    const last30 = buildPeriodPresets(range, at("2025-06-28T12:00:00Z")).find(
      (preset) => preset.id === "last:30",
    );

    expect(last30?.from).toBe(range.from);
  });
});
