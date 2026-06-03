import { describe, expect, it } from "vitest";
import {
  buildModeTone,
  diagnosticRowHasIssue,
  emptySectionRows,
  filterDiagnosticIssueRows,
  formatDiagnosticError,
  formatSummaryGeneratedAt,
  labelFromKey,
  privacyExcludedDataClasses,
  privacyFallbackNote,
  sortCountRows,
  statusTone,
} from "./diagnostics-view-model";

describe("diagnostics view model helpers", () => {
  it("maps known statuses into allow-listed badge tones", () => {
    for (const status of [
      "available",
      "current",
      "synced",
      "ready",
      "succeeded",
      "completed",
      "complete",
      "none",
    ]) {
      expect(statusTone(status)).toBe("success");
    }
    for (const status of ["pending", "queued", "running", "cancel_requested", "partial", "present"]) {
      expect(statusTone(status)).toBe("info");
    }
    for (const status of [
      "never_synced",
      "missing_key",
      "not_configured",
      "unavailable",
      "not_found",
      "timed_out",
      "cancelled",
    ]) {
      expect(statusTone(status)).toBe("warning");
    }
    for (const status of ["failed", "check_failed", "error", "internal", "network", "auth", "validation"]) {
      expect(statusTone(status)).toBe("danger");
    }
  });

  it("uses neutral tone for unknown, empty, and new statuses", () => {
    expect(statusTone("brand_new_backend_state")).toBe("neutral");
    expect(statusTone("")).toBe("neutral");
    expect(statusTone(null)).toBe("neutral");
  });

  it("maps build mode as factual metadata", () => {
    expect(buildModeTone("release")).toBe("success");
    expect(buildModeTone("debug")).toBe("info");
    expect(buildModeTone("profile")).toBe("neutral");
  });

  it("formats coarse keys without resolving hidden detail", () => {
    expect(labelFromKey("never_synced")).toBe("Never synced");
    expect(labelFromKey("youtube_video_full_sync")).toBe("Youtube video full sync");
    expect(labelFromKey(null)).toBe("Unknown");
    expect(labelFromKey("")).toBe("Unknown");
  });

  it("formats summary generation time as absolute UTC", () => {
    expect(formatSummaryGeneratedAt(1_717_300_000)).toBe(
      "Summary generated 2024-06-02 03:46:40 UTC",
    );
  });

  it("falls back for invalid generated-at values", () => {
    expect(formatSummaryGeneratedAt("2024-06-02T03:46:40Z")).toBe("Summary generated Unknown");
    expect(formatSummaryGeneratedAt(Number.NaN)).toBe("Summary generated Unknown");
    expect(formatSummaryGeneratedAt(Number.POSITIVE_INFINITY)).toBe("Summary generated Unknown");
    expect(formatSummaryGeneratedAt(null)).toBe("Summary generated Unknown");
  });

  it("sorts count rows by grouping keys before count", () => {
    const rows = [
      { provider: "zeta", state: "running", count: 1 },
      { provider: "alpha", state: "queued", count: 5 },
      { provider: "alpha", state: "queued", count: 2 },
      { provider: "alpha", state: "completed", count: 9 },
    ];

    expect(sortCountRows(rows, ["provider", "state"])).toEqual([
      { provider: "alpha", state: "completed", count: 9 },
      { provider: "alpha", state: "queued", count: 2 },
      { provider: "alpha", state: "queued", count: 5 },
      { provider: "zeta", state: "running", count: 1 },
    ]);
  });

  it("returns a quiet empty-section row", () => {
    expect(emptySectionRows([])).toEqual([{ empty: true, label: "No diagnostic counts reported" }]);
    expect(emptySectionRows([{ count: 1 }])).toEqual([]);
  });

  it("formats privacy excluded data classes and fallback note", () => {
    expect(privacyExcludedDataClasses(["source_content", "api_keys"])).toEqual([
      "Source content",
      "Api keys",
    ]);
    expect(privacyExcludedDataClasses([])).toEqual([]);
    expect(privacyExcludedDataClasses(null)).toEqual([]);
    expect(privacyFallbackNote(["api_keys"])).toBe("");
    expect(privacyFallbackNote([])).toBe(
      "This diagnostics view is designed to show sanitized fields only. The backend did not report excluded data classes for this summary.",
    );
  });

  it("delegates recognized AppError values to formatAppError", () => {
    expect(formatDiagnosticError("loading diagnostics", { kind: "validation", message: "Bad state" })).toBe(
      "Error loading diagnostics (validation): Bad state",
    );
    expect(
      formatDiagnosticError(
        "loading diagnostics",
        JSON.stringify({ kind: "not_found", message: "Summary missing" }),
      ),
    ).toBe("Error loading diagnostics (not_found): Summary missing");
  });

  it("does not delegate AppError-shaped objects that carry detail-ish fields", () => {
    const message = formatDiagnosticError("loading diagnostics", {
      kind: "internal",
      message: "private raw payload",
      payload: "secret payload",
      stack: "private stack",
    });

    expect(message).toBe("Error loading diagnostics: Diagnostics could not be loaded.");
    expect(message).not.toContain("private raw payload");
    expect(message).not.toContain("secret payload");
    expect(message).not.toContain("private stack");
  });

  it("uses a generic fallback for unknown non-app errors without leaking fields", () => {
    const message = formatDiagnosticError("loading diagnostics", {
      message: "raw object message",
      stack: "private stack",
      payload: "private payload",
      url: "https://private.example",
      path: "C:/Users/private/db.sqlite",
      raw: "raw payload",
      log: "raw log",
      baseUrl: "https://llm.private/v1",
      sourceId: 42,
      profileId: "private-profile",
    });

    expect(message).toBe("Error loading diagnostics: Diagnostics could not be loaded.");
    expect(message).not.toContain("raw object message");
    expect(message).not.toContain("private stack");
    expect(message).not.toContain("private payload");
    expect(message).not.toContain("private.example");
    expect(message).not.toContain("raw log");
    expect(message).not.toContain("llm.private");
    expect(message).not.toContain("private-profile");
    expect(message).not.toContain("[object Object]");
  });

  it("detects issue diagnostic rows without treating healthy rows as issues", () => {
    expect(diagnosticRowHasIssue({ status: "Failed", error: "Internal", count: 1 })).toBe(true);
    expect(diagnosticRowHasIssue({ status: "Completed", error: "None", count: 11 })).toBe(false);
  });

  it("filters diagnostic rows down to issue rows", () => {
    const rows = [
      { status: "Completed", error: "None", count: 11 },
      { status: "Failed", error: "Internal", count: 1 },
      { status: "Cancelled", completeness: "Partial", count: 4 },
    ];

    expect(filterDiagnosticIssueRows(rows)).toEqual([
      { status: "Failed", error: "Internal", count: 1 },
      { status: "Cancelled", completeness: "Partial", count: 4 },
    ]);
  });
});
