import type { ProjectDataRange } from "$lib/types/projects";

export interface PeriodPreset {
  id: string;
  label: string;
  from: number;
  to: number;
}

const SECONDS_PER_DAY = 86_400;
const LAST_DAYS_WINDOWS = [7, 30, 90];

export function buildPeriodPresets(range: ProjectDataRange, now: number): PeriodPreset[] {
  if (range.from === null || range.to === null) return [];
  const from = range.from;
  const to = range.to;

  const presets: PeriodPreset[] = [{ id: "all", label: "Весь период", from, to }];

  const firstYear = new Date(from * 1000).getUTCFullYear();
  const lastYear = new Date(to * 1000).getUTCFullYear();
  for (let year = lastYear; year >= firstYear; year -= 1) {
    const yearStart = Math.floor(Date.UTC(year, 0, 1) / 1000);
    const yearEnd = Math.floor(Date.UTC(year + 1, 0, 1) / 1000) - 1;
    presets.push({
      id: `year:${year}`,
      label: String(year),
      from: Math.max(from, yearStart),
      to: Math.min(to, yearEnd),
    });
  }

  const anchor = Math.min(now, to);
  for (const days of LAST_DAYS_WINDOWS) {
    presets.push({
      id: `last:${days}`,
      label: `Последние ${days} дней`,
      from: Math.max(from, anchor - days * SECONDS_PER_DAY),
      to: anchor,
    });
  }

  return presets;
}
