import { afterEach, describe, expect, it, vi } from "vitest";
import {
  defaultDateOffset,
  endOfDayUnix,
  formatPeriod,
  formatTimestamp,
  normalizeRef,
  parseReportSegments,
  phaseLabel,
  reportLines,
  runTargetLabel,
  startOfDayUnix,
  statusTone,
} from "./analysis-utils";

describe("analysis-utils", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("formats date offsets from the current local day", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 4, 3, 12, 0, 0));

    expect(defaultDateOffset(-30)).toBe("2026-04-03");
    expect(defaultDateOffset(0)).toBe("2026-05-03");
    expect(defaultDateOffset(1)).toBe("2026-05-04");
  });

  it("maps date strings to local start and end unix seconds", () => {
    expect(startOfDayUnix("2026-05-03")).toBe(
      Math.floor(new Date("2026-05-03T00:00:00").getTime() / 1000),
    );
    expect(endOfDayUnix("2026-05-03")).toBe(
      Math.floor(new Date("2026-05-03T23:59:59").getTime() / 1000),
    );
  });

  it("formats timestamps and periods with n/a fallbacks", () => {
    expect(formatTimestamp(null)).toBe("n/a");
    expect(formatPeriod(0, 0)).toBe("n/a - n/a");
  });

  it("prefers frozen scope labels before source or group fallbacks", () => {
    expect(
      runTargetLabel({
        scope_type: "single_source",
        source_id: 7,
        source_title: "Live Source",
        source_group_id: null,
        source_group_name: null,
        scope_label: "Frozen label",
      }),
    ).toBe("Frozen label");

    expect(
      runTargetLabel({
        scope_type: "source_group",
        source_id: null,
        source_title: null,
        source_group_id: 3,
        source_group_name: null,
        scope_label: "",
      }),
    ).toBe("Group 3");

    expect(
      runTargetLabel({
        scope_type: "single_source",
        source_id: 9,
        source_title: null,
        source_group_id: null,
        source_group_name: null,
        scope_label: "",
      }),
    ).toBe("Source 9");
  });

  it("labels project analysis runs from scope label and project fallbacks", () => {
    expect(
      runTargetLabel({
        scope_type: "project",
        project_id: 7,
        project_name: "Alpha",
        source_id: null,
        source_title: null,
        source_group_id: null,
        source_group_name: null,
        scope_label: "Alpha snapshot",
      }),
    ).toBe("Alpha snapshot");

    expect(
      runTargetLabel({
        scope_type: "project",
        project_id: 7,
        project_name: "Alpha",
        source_id: null,
        source_title: null,
        source_group_id: null,
        source_group_name: null,
        scope_label: "",
      }),
    ).toBe("Alpha");
  });

  it("maps phases and statuses to user-facing labels and badge tones", () => {
    expect(phaseLabel("map")).toBe("Analyzing chunks");
    expect(phaseLabel("")).toBe("Idle");
    expect(phaseLabel("custom_phase")).toBe("custom_phase");

    expect(statusTone("completed")).toBe("success");
    expect(statusTone("failed")).toBe("danger");
    expect(statusTone("running")).toBe("info");
    expect(statusTone("cancelled")).toBe("neutral");
  });

  it("normalizes only supported trace refs", () => {
    expect(normalizeRef("[s12-i34]")).toBe("s12-i34");
    expect(normalizeRef(" s1-i2 ")).toBe("s1-i2");
    expect(normalizeRef("s20-i4@754000ms")).toBe("s20-i4@754000ms");
    expect(normalizeRef("s20-i4@754000-790000ms")).toBe("s20-i4@754000-790000ms");
    expect(normalizeRef("s12-m34")).toBeNull();
    expect(normalizeRef("source-1-message-2")).toBeNull();
  });

  it("parses report refs while preserving surrounding text", () => {
    expect(parseReportSegments("See [s1-i2, nope, s3-i4] now")).toEqual([
      { type: "text", value: "See ", key: "text-0" },
      { type: "ref", value: "s1-i2", key: "ref-4-s1-i2-0" },
      { type: "text", value: ", ", key: "comma-4-0" },
      { type: "ref", value: "s3-i4", key: "ref-4-s3-i4-1" },
      { type: "text", value: " now", key: "text-tail-24" },
    ]);

    expect(parseReportSegments("No refs [bad]")).toEqual([
      { type: "text", value: "No refs ", key: "text-0" },
      { type: "text", value: "[bad]", key: "text-8" },
    ]);
  });

  it("splits reports into stable keyed lines", () => {
    expect(reportLines("first\n[s1-i2]")).toEqual([
      {
        key: "line-0",
        segments: [{ type: "text", value: "first", key: "text-tail-0" }],
      },
      {
        key: "line-1",
        segments: [{ type: "ref", value: "s1-i2", key: "ref-0-s1-i2-0" }],
      },
    ]);
  });
});
