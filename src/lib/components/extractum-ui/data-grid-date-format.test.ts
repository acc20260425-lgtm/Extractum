import { describe, expect, it } from "vitest";
import {
  enhanceDateTimeColumns,
  formatDataGridDateTimeValue,
  parseDataGridDateTimeValue,
  type ExtractumDataGridColumn,
} from "./data-grid-date-format";

describe("data grid date/time formatting", () => {
  it("formats ISO datetime values with localized date and time", () => {
    const formatted = formatDataGridDateTimeValue(
      "2026-06-22T21:24:51Z",
      "datetime",
      "en-US",
      "UTC",
    );

    expect(formatted).toBe("Jun 22, 2026, 21:24");
  });

  it("formats Unix seconds and milliseconds to the same instant", () => {
    const seconds = formatDataGridDateTimeValue(1_719_792_000, "datetime", "en-US", "UTC");
    const milliseconds = formatDataGridDateTimeValue(1_719_792_000_000, "datetime", "en-US", "UTC");

    expect(seconds).toBe("Jul 1, 2024, 00:00");
    expect(milliseconds).toBe(seconds);
  });

  it("formats date values without time", () => {
    const formatted = formatDataGridDateTimeValue("2026-06-22T21:24:51Z", "date", "en-US", "UTC");

    expect(formatted).toBe("Jun 22, 2026");
  });

  it("formats time values without date", () => {
    const formatted = formatDataGridDateTimeValue("2026-06-22T21:24:51Z", "time", "en-US", "UTC");

    expect(formatted).toBe("21:24");
  });

  it("returns invalid values unchanged", () => {
    expect(formatDataGridDateTimeValue("not-a-date", "datetime", "en-US", "UTC")).toBe("not-a-date");
    expect(formatDataGridDateTimeValue("", "datetime", "en-US", "UTC")).toBe("");
    expect(formatDataGridDateTimeValue(null, "datetime", "en-US", "UTC")).toBe(null);
  });

  it("parses Date instances, ISO strings, seconds, and milliseconds", () => {
    expect(parseDataGridDateTimeValue(new Date("2026-06-22T21:24:51Z"))?.toISOString()).toBe(
      "2026-06-22T21:24:51.000Z",
    );
    expect(parseDataGridDateTimeValue("2026-06-22T21:24:51Z")?.toISOString()).toBe(
      "2026-06-22T21:24:51.000Z",
    );
    expect(parseDataGridDateTimeValue(1_719_792_000)?.toISOString()).toBe("2024-07-01T00:00:00.000Z");
    expect(parseDataGridDateTimeValue(1_719_792_000_000)?.toISOString()).toBe("2024-07-01T00:00:00.000Z");
  });

  it("injects templates only for opted-in columns without existing templates", () => {
    const existingTemplate = (value: unknown) => `raw:${String(value)}`;
    const columns: ExtractumDataGridColumn[] = [
      { id: "name", header: "Name" },
      { id: "createdAt", header: "Created", dateTimeFormat: "datetime" },
      { id: "rawCreatedAt", header: "Raw Created", dateTimeFormat: false },
      { id: "publishedAt", header: "Published", dateTimeFormat: "date", template: existingTemplate },
    ];

    const enhanced = enhanceDateTimeColumns(columns, "en-US", "UTC");

    expect(enhanced[0]).toBe(columns[0]);
    expect(enhanced[1]).not.toBe(columns[1]);
    expect(enhanced[1].template?.("2026-06-22T21:24:51Z", {}, enhanced[1])).toBe("Jun 22, 2026, 21:24");
    expect(enhanced[2]).toBe(columns[2]);
    expect(enhanced[3].template).toBe(existingTemplate);
  });
});
