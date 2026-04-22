export type AppErrorKind =
  | "validation"
  | "not_found"
  | "auth"
  | "network"
  | "conflict"
  | "internal";

interface AppErrorPayload {
  kind: AppErrorKind;
  message: string;
}

const APP_ERROR_KINDS: AppErrorKind[] = [
  "validation",
  "not_found",
  "auth",
  "network",
  "conflict",
  "internal",
];

function isAppErrorKind(value: unknown): value is AppErrorKind {
  return typeof value === "string" && APP_ERROR_KINDS.includes(value as AppErrorKind);
}

function parseJsonError(value: string): AppErrorPayload | null {
  const trimmed = value.trim();
  if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) {
    return null;
  }

  try {
    return toAppErrorPayload(JSON.parse(trimmed));
  } catch {
    return null;
  }
}

function toAppErrorPayload(value: unknown): AppErrorPayload | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const maybeError = value as Record<string, unknown>;
  if (!isAppErrorKind(maybeError.kind) || typeof maybeError.message !== "string") {
    return null;
  }

  return {
    kind: maybeError.kind,
    message: maybeError.message.trim(),
  };
}

export function errorKind(error: unknown): AppErrorKind | null {
  if (typeof error === "string") {
    return parseJsonError(error)?.kind ?? null;
  }

  return toAppErrorPayload(error)?.kind ?? null;
}

export function describeError(error: unknown): string {
  if (typeof error === "string") {
    const parsed = parseJsonError(error);
    return parsed?.message || error.trim() || "Unknown error";
  }

  const structured = toAppErrorPayload(error);
  if (structured) {
    return structured.message || "Unknown error";
  }

  if (error instanceof Error) {
    return error.message.trim() || "Unknown error";
  }

  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) {
      return message.trim();
    }
  }

  return "Unknown error";
}

export function formatAppError(action: string, error: unknown): string {
  const kind = errorKind(error);
  const message = describeError(error);

  if (kind && kind !== "internal") {
    return `Error ${action} (${kind}): ${message}`;
  }

  return `Error ${action}: ${message}`;
}
