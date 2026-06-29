import type { ProjectStatus, ProjectSummary } from "$lib/types/projects";

export interface ProjectRailRow {
  id: number;
  name: string;
  status: ProjectStatus;
  statusLabel: string;
  sourceCountLabel: string;
  meta: string;
  pinned: boolean;
  archived: boolean;
}

const STATUS_LABELS: Record<ProjectStatus, string> = {
  ready: "готов",
  running: "идёт анализ",
  needs_attention: "требует внимания",
  empty: "нет источников",
};

function pluralRu(count: number, one: string, few: string, many: string): string {
  const mod10 = count % 10;
  const mod100 = count % 100;
  if (mod10 === 1 && mod100 !== 11) return one;
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return few;
  return many;
}

function sourceCountLabel(count: number): string {
  return `${count} ${pluralRu(count, "источник", "источника", "источников")}`;
}

const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;

export function relativeRunLabel(lastRunAt: number | null, now: number): string | null {
  if (lastRunAt === null) return null;
  const diff = Math.max(0, now - lastRunAt);
  if (diff < MINUTE) return "только что";
  if (diff < HOUR) return `${Math.floor(diff / MINUTE)} мин назад`;
  if (diff < DAY) return `${Math.floor(diff / HOUR)} ч назад`;
  if (diff < WEEK) return `${Math.floor(diff / DAY)} дн назад`;
  return `${Math.floor(diff / WEEK)} нед назад`;
}

function projectMeta(summary: ProjectSummary, now: number): string {
  if (summary.archived) return "в архиве";
  const statusLabel = STATUS_LABELS[summary.status];
  if (summary.status === "empty") return statusLabel;

  const parts = [sourceCountLabel(summary.source_count), statusLabel];
  if (summary.status !== "running") {
    const lastRun = relativeRunLabel(summary.last_run_at, now);
    if (lastRun) parts.push(lastRun);
  }
  return parts.join(" · ");
}

export interface ProjectRailSections {
  pinned: ProjectRailRow[];
  normal: ProjectRailRow[];
  archived: ProjectRailRow[];
}

export function groupProjectRail(
  summaries: ProjectSummary[],
  now: number,
): ProjectRailSections {
  const sections: ProjectRailSections = { pinned: [], normal: [], archived: [] };
  for (const summary of summaries) {
    const row = buildProjectRailRow(summary, now);
    if (row.archived) sections.archived.push(row);
    else if (row.pinned) sections.pinned.push(row);
    else sections.normal.push(row);
  }
  return sections;
}

export function buildProjectRailRow(summary: ProjectSummary, now: number): ProjectRailRow {
  return {
    id: summary.id,
    name: summary.name,
    status: summary.status,
    statusLabel: STATUS_LABELS[summary.status],
    sourceCountLabel: sourceCountLabel(summary.source_count),
    meta: projectMeta(summary, now),
    pinned: summary.pinned,
    archived: summary.archived,
  };
}
