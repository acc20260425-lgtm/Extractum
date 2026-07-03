import { describe, expect, it } from "vitest";
import {
  buildProjectRailRow,
  filterProjectRail,
  groupProjectRail,
  projectRailRowMatches,
  relativeRunLabel,
} from "./research-projects-rail";
import type { ProjectRailRow } from "./research-projects-rail";
import type { ProjectSummary } from "$lib/types/projects";

function summary(overrides: Partial<ProjectSummary> = {}): ProjectSummary {
  return {
    id: 1,
    name: "Беларусь: медиаполе 2025",
    description: null,
    source_count: 10,
    material_count: 339,
    status: "ready",
    last_run_at: null,
    pinned: false,
    archived: false,
    updated_at: 1000,
    ...overrides,
  };
}

const NOW = 1_000_000_000;

describe("buildProjectRailRow", () => {
  it("builds a rail row from a project summary", () => {
    const row = buildProjectRailRow(summary({ source_count: 10, status: "ready" }), NOW);

    expect(row.id).toBe(1);
    expect(row.name).toBe("Беларусь: медиаполе 2025");
    expect(row.status).toBe("ready");
    expect(row.statusLabel).toBe("готов");
    expect(row.sourceCountLabel).toBe("10 источников");
    expect(row.pinned).toBe(false);
    expect(row.archived).toBe(false);
  });

  it("composes the rail meta string", () => {
    const meta = (overrides: Partial<ProjectSummary>) =>
      buildProjectRailRow(summary(overrides), NOW).meta;

    expect(meta({ source_count: 10, status: "ready", last_run_at: NOW - 2 * 3600 })).toBe(
      "10 источников · готов · 2 ч назад",
    );
    expect(meta({ source_count: 6, status: "running", last_run_at: NOW - 2 * 3600 })).toBe(
      "6 источников · идёт анализ",
    );
    expect(meta({ source_count: 0, status: "empty", last_run_at: null })).toBe("нет источников");
    expect(meta({ archived: true, status: "ready", source_count: 3 })).toBe("в архиве");
  });

  it("uses Russian plural forms for the source count", () => {
    const label = (count: number) =>
      buildProjectRailRow(summary({ source_count: count }), NOW).sourceCountLabel;

    expect(label(0)).toBe("0 источников");
    expect(label(1)).toBe("1 источник");
    expect(label(2)).toBe("2 источника");
    expect(label(4)).toBe("4 источника");
    expect(label(5)).toBe("5 источников");
    expect(label(11)).toBe("11 источников");
    expect(label(21)).toBe("21 источник");
    expect(label(22)).toBe("22 источника");
  });
});

describe("relativeRunLabel", () => {
  const now = 1_000_000_000;
  const ago = (seconds: number) => relativeRunLabel(now - seconds, now);

  it("returns null when there is no last run", () => {
    expect(relativeRunLabel(null, now)).toBeNull();
  });

  it("formats recent runs with abbreviated Russian units", () => {
    expect(ago(30)).toBe("только что");
    expect(ago(5 * 60)).toBe("5 мин назад");
    expect(ago(2 * 3600)).toBe("2 ч назад");
    expect(ago(5 * 86400)).toBe("5 дн назад");
    expect(ago(21 * 86400)).toBe("3 нед назад");
  });
});

describe("groupProjectRail", () => {
  it("splits projects into pinned, normal, and archived sections preserving order", () => {
    const summaries = [
      summary({ id: 1, pinned: true, archived: false }),
      summary({ id: 2, pinned: false, archived: false }),
      summary({ id: 3, pinned: false, archived: true }),
      summary({ id: 4, pinned: true, archived: true }),
    ];

    const sections = groupProjectRail(summaries, NOW);

    expect(sections.pinned.map((row) => row.id)).toEqual([1]);
    expect(sections.normal.map((row) => row.id)).toEqual([2]);
    expect(sections.archived.map((row) => row.id)).toEqual([3, 4]);
  });
});

describe("projectRailRowMatches / filterProjectRail", () => {
  const row = (over: Partial<ProjectRailRow> = {}): ProjectRailRow => ({
    id: 1,
    name: "Беларусь",
    status: "ready",
    statusLabel: "готов",
    sourceCountLabel: "3 источника",
    meta: "3 источника · готов",
    pinned: false,
    archived: false,
    ...over,
  });

  it("matches by name case-insensitively", () => {
    expect(projectRailRowMatches(row({ name: "Финтех-мониторинг" }), "финтех")).toBe(true);
    expect(projectRailRowMatches(row({ name: "Финтех" }), "зиг")).toBe(false);
  });

  it("matches by meta text", () => {
    expect(projectRailRowMatches(row({ meta: "6 источников · идёт анализ" }), "анализ")).toBe(true);
  });

  it("blank query matches everything", () => {
    expect(projectRailRowMatches(row(), "")).toBe(true);
    expect(projectRailRowMatches(row(), "   ")).toBe(true);
  });

  it("filterProjectRail filters each section and keeps empty query intact", () => {
    const sections = {
      pinned: [row({ id: 1, name: "Alpha" })],
      normal: [row({ id: 2, name: "Beta" }), row({ id: 3, name: "Gamma" })],
      archived: [row({ id: 4, name: "Beta-архив" })],
    };
    const out = filterProjectRail(sections, "beta");
    expect(out.pinned).toHaveLength(0);
    expect(out.normal.map((r) => r.id)).toEqual([2]);
    expect(out.archived.map((r) => r.id)).toEqual([4]);
    expect(filterProjectRail(sections, "")).toEqual(sections);
  });
});
