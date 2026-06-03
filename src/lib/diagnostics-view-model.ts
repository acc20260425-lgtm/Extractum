import { formatAppError, type AppErrorKind } from "$lib/app-error";
import type { BadgeVariant } from "$lib/components/ui/types";

type CountRowValue = string | number | boolean | null | undefined;
type DiagnosticAppErrorPayload = {
  kind: AppErrorKind;
  message: string;
};

const APP_ERROR_KINDS = new Set<AppErrorKind>([
  "validation",
  "not_found",
  "auth",
  "network",
  "conflict",
  "internal",
]);

const DETAILISH_ERROR_KEYS = new Set([
  "stack",
  "payload",
  "url",
  "path",
  "raw",
  "log",
  "baseUrl",
  "sourceId",
  "profileId",
]);

const SUCCESS_STATUSES = new Set([
  "available",
  "current",
  "synced",
  "ready",
  "succeeded",
  "completed",
  "complete",
  "none",
]);

const INFO_STATUSES = new Set([
  "pending",
  "queued",
  "running",
  "cancel_requested",
  "partial",
  "present",
]);

const WARNING_STATUSES = new Set([
  "never_synced",
  "missing_key",
  "not_configured",
  "unavailable",
  "not_found",
  "timed_out",
  "cancelled",
]);

const DANGER_STATUSES = new Set([
  "failed",
  "check_failed",
  "error",
  "internal",
  "network",
  "auth",
  "validation",
]);

const PRIVACY_FALLBACK_NOTE =
  "This diagnostics view is designed to show sanitized fields only. The backend did not report excluded data classes for this summary.";

const diagnosticIssuePattern = /failed|error|missing|unavailable|pending|warning|partial|cancelled/i;

function normalizedStatus(value: unknown) {
  return typeof value === "string" ? value.trim().toLowerCase() : "";
}

function isAppErrorKind(value: unknown): value is AppErrorKind {
  return typeof value === "string" && APP_ERROR_KINDS.has(value as AppErrorKind);
}

function hasDetailishKey(value: Record<string, unknown>) {
  return Object.keys(value).some((key) => DETAILISH_ERROR_KEYS.has(key));
}

function toDiagnosticAppError(value: unknown): DiagnosticAppErrorPayload | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }

  const candidate = value as Record<string, unknown>;
  const keys = Object.keys(candidate);
  if (keys.length !== 2 || hasDetailishKey(candidate)) {
    return null;
  }

  if (!keys.includes("kind") || !keys.includes("message")) {
    return null;
  }

  if (!isAppErrorKind(candidate.kind) || typeof candidate.message !== "string") {
    return null;
  }

  return {
    kind: candidate.kind,
    message: candidate.message.trim(),
  };
}

function parseDiagnosticAppError(value: string) {
  const trimmed = value.trim();
  if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) {
    return null;
  }

  try {
    return toDiagnosticAppError(JSON.parse(trimmed));
  } catch {
    return null;
  }
}

function diagnosticAppError(error: unknown) {
  if (typeof error === "string") {
    return parseDiagnosticAppError(error);
  }

  return toDiagnosticAppError(error);
}

function padUtc(value: number) {
  return value.toString().padStart(2, "0");
}

function rowValue(row: object, key: string): CountRowValue {
  const value = (row as Record<string, unknown>)[key];
  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean" ||
    value === null ||
    value === undefined
  ) {
    return value;
  }

  return undefined;
}

function sortValue(value: CountRowValue) {
  if (typeof value === "boolean") {
    return value ? "yes" : "no";
  }

  return labelFromKey(value).toLowerCase();
}

function countValue(row: object) {
  const value = rowValue(row, "count");
  return typeof value === "number" ? value : 0;
}

export function statusTone(status: unknown): BadgeVariant {
  const key = normalizedStatus(status);
  if (!key) return "neutral";
  if (SUCCESS_STATUSES.has(key)) return "success";
  if (INFO_STATUSES.has(key)) return "info";
  if (WARNING_STATUSES.has(key)) return "warning";
  if (DANGER_STATUSES.has(key)) return "danger";
  return "neutral";
}

export function buildModeTone(buildMode: unknown): BadgeVariant {
  const key = normalizedStatus(buildMode);
  if (key === "release") return "success";
  if (key === "debug") return "info";
  return "neutral";
}

export function diagnosticRowHasIssue(row: Record<string, string | number | undefined>) {
  return Object.entries(row).some(([key, value]) => {
    if (key.toLowerCase() === "count") return false;
    return diagnosticIssuePattern.test(String(value));
  });
}

export function filterDiagnosticIssueRows<T extends Record<string, string | number | undefined>>(rows: T[]) {
  return rows.filter((row) => diagnosticRowHasIssue(row));
}

export function labelFromKey(value: unknown) {
  if (typeof value !== "string") {
    return "Unknown";
  }

  const normalized = value.trim().replace(/[_-]+/g, " ").replace(/\s+/g, " ").toLowerCase();
  if (!normalized) {
    return "Unknown";
  }

  return normalized.charAt(0).toUpperCase() + normalized.slice(1);
}

export function formatSummaryGeneratedAt(value: unknown) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return "Summary generated Unknown";
  }

  const date = new Date(value * 1000);
  if (!Number.isFinite(date.getTime())) {
    return "Summary generated Unknown";
  }

  const year = date.getUTCFullYear();
  const month = padUtc(date.getUTCMonth() + 1);
  const day = padUtc(date.getUTCDate());
  const hour = padUtc(date.getUTCHours());
  const minute = padUtc(date.getUTCMinutes());
  const second = padUtc(date.getUTCSeconds());
  return `Summary generated ${year}-${month}-${day} ${hour}:${minute}:${second} UTC`;
}

export function sortCountRows<T extends object>(rows: readonly T[], groupingKeys: readonly string[]) {
  return [...rows].sort((left, right) => {
    for (const key of groupingKeys) {
      const leftValue = sortValue(rowValue(left, key));
      const rightValue = sortValue(rowValue(right, key));
      const compared = leftValue.localeCompare(rightValue);
      if (compared !== 0) {
        return compared;
      }
    }

    return countValue(left) - countValue(right);
  });
}

export function emptySectionRows(rows: readonly unknown[]) {
  return rows.length > 0 ? [] : [{ empty: true, label: "No diagnostic counts reported" }];
}

export function privacyExcludedDataClasses(value: unknown) {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.filter((item): item is string => typeof item === "string" && item.trim().length > 0).map(labelFromKey);
}

export function privacyFallbackNote(value: unknown) {
  return privacyExcludedDataClasses(value).length > 0 ? "" : PRIVACY_FALLBACK_NOTE;
}

export function formatDiagnosticError(action: string, error: unknown) {
  const appError = diagnosticAppError(error);
  if (appError) {
    return formatAppError(action, appError);
  }

  return `Error ${action}: Diagnostics could not be loaded.`;
}

export function yesNo(value: boolean) {
  return value ? "Yes" : "No";
}

export function availabilityLabel(value: boolean) {
  return value ? "Available" : "Unavailable";
}

export function availabilityTone(value: boolean): BadgeVariant {
  return value ? "success" : "warning";
}
