import type { AnalysisRunSummary, ReportSegment } from "$lib/types/analysis";

export function defaultDateOffset(offsetDays: number) {
  const date = new Date();
  date.setDate(date.getDate() + offsetDays);
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function startOfDayUnix(dateString: string) {
  return Math.floor(new Date(`${dateString}T00:00:00`).getTime() / 1000);
}

export function endOfDayUnix(dateString: string) {
  return Math.floor(new Date(`${dateString}T23:59:59`).getTime() / 1000);
}

export function formatTimestamp(timestamp: number | null) {
  if (!timestamp) return "n/a";
  return new Date(timestamp * 1000).toLocaleString();
}

export function formatDay(timestamp: number | null) {
  if (!timestamp) return "n/a";
  return new Date(timestamp * 1000).toLocaleDateString();
}

export function formatPeriod(periodFromUnix: number, periodToUnix: number) {
  return `${formatDay(periodFromUnix)} - ${formatDay(periodToUnix)}`;
}

export function runTargetLabel(
  run: Pick<
    AnalysisRunSummary,
    "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label"
  >
) {
  if (run.scope_label.trim()) {
    return run.scope_label;
  }

  if (run.scope_type === "source_group") {
    return run.source_group_name ?? `Group ${run.source_group_id ?? "?"}`;
  }
  return run.source_title ?? `Source ${run.source_id ?? "?"}`;
}

export function phaseLabel(phase: string) {
  switch (phase) {
    case "queued":
      return "Queued";
    case "load_items":
      return "Loading items";
    case "chunking":
      return "Chunking corpus";
    case "map":
      return "Analyzing chunks";
    case "reduce":
      return "Writing report";
    case "persist":
      return "Saving run";
    case "completed":
      return "Completed";
    case "failed":
      return "Failed";
    case "running":
      return "Running";
    case "idle":
      return "Idle";
    default:
      return phase || "Idle";
  }
}

export function statusTone(status: string) {
  switch (status) {
    case "completed":
      return "success";
    case "failed":
      return "danger";
    case "running":
    case "queued":
      return "info";
    default:
      return "neutral";
  }
}

export function normalizeRef(candidate: string) {
  const trimmed = candidate.trim().replace(/^\[/, "").replace(/\]$/, "");
  return /^s\d+-m\d+$/.test(trimmed) ? trimmed : null;
}

export function parseReportSegments(line: string): ReportSegment[] {
  const segments: ReportSegment[] = [];
  const regex = /\[([^\]]+)\]/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null = null;

  while ((match = regex.exec(line)) !== null) {
    if (match.index > lastIndex) {
      segments.push({
        type: "text",
        value: line.slice(lastIndex, match.index),
        key: `text-${lastIndex}`,
      });
    }

    const refs = match[1]
      .split(",")
      .map((part) => normalizeRef(part))
      .filter((value): value is string => value !== null);

    if (refs.length === 0) {
      segments.push({
        type: "text",
        value: match[0],
        key: `text-${match.index}`,
      });
    } else {
      refs.forEach((ref, refIndex) => {
        segments.push({
          type: "ref",
          value: ref,
          key: `ref-${match?.index ?? 0}-${ref}-${refIndex}`,
        });
        if (refIndex < refs.length - 1) {
          segments.push({
            type: "text",
            value: ", ",
            key: `comma-${match?.index ?? 0}-${refIndex}`,
          });
        }
      });
    }

    lastIndex = regex.lastIndex;
  }

  if (lastIndex < line.length) {
    segments.push({
      type: "text",
      value: line.slice(lastIndex),
      key: `text-tail-${lastIndex}`,
    });
  }

  if (segments.length === 0) {
    segments.push({ type: "text", value: "", key: "empty-line" });
  }

  return segments;
}

export function reportLines(text: string) {
  return text.split("\n").map((line, index) => ({
    key: `line-${index}`,
    segments: parseReportSegments(line),
  }));
}
