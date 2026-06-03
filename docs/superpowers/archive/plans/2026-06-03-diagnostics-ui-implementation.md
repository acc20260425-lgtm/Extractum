# Diagnostics UI Implementation Plan

> Historical execution record. The Diagnostics UI shipped on 2026-06-03;
> current behavior is summarized in root docs such as `docs/project.md`,
> `docs/design-document.md`, and `docs/architecture-deep-dive.md`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a read-only `/diagnostics` frontend surface that renders the existing sanitized `get_diagnostic_summary` backend contract.

**Architecture:** Keep the Tauri command behind one API wrapper, keep display derivations in a pure view-model helper file, and keep the Svelte route as a client-only renderer with manual refresh. Navigation adds a dedicated diagnostics entry and topbar label while Settings remains focused on LLM provider configuration.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, TypeScript, Tauri 2 `invoke`, Vitest, lucide-svelte UI icons, existing local UI components.

---

## File Structure

- Create `src/lib/types/diagnostics.ts`: frontend DTO interfaces mirroring the backend camelCase diagnostic summary.
- Create `src/lib/api/diagnostics.ts`: narrow Tauri boundary for `get_diagnostic_summary`; no mapping beyond typing.
- Create `src/lib/api/diagnostics.test.ts`: API wrapper invocation, as-is DTO return, and API source privacy checks.
- Create `src/lib/diagnostics-view-model.ts`: pure display helpers for labels, tones, UTC timestamp formatting, sorting, empty rows, privacy fallback, and safe diagnostics error formatting.
- Create `src/lib/diagnostics-view-model.test.ts`: helper unit tests independent of Svelte.
- Create `src/lib/components/diagnostics/DiagnosticCountTable.svelte`: repeated count-table presentation with per-section empty rows.
- Create `src/routes/diagnostics/+page.svelte`: client-only diagnostics page; calls `loadDiagnosticSummary()` from `onMount` and manual Refresh only.
- Modify `src/routes/+layout.svelte`: add `ShieldCheck` sidebar entry and `/diagnostics` topbar route label.
- Create `src/lib/diagnostics-route-contract.test.ts`: production source scans for route/API boundary, no raw/log/copy affordances, navigation contract, and Settings separation.

---

### Task 1: Diagnostics DTO Types And API Wrapper

**Files:**
- Create: `src/lib/types/diagnostics.ts`
- Create: `src/lib/api/diagnostics.ts`
- Create: `src/lib/api/diagnostics.test.ts`

- [x] **Step 1: Write the failing API wrapper test**

Create `src/lib/api/diagnostics.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import diagnosticsApiSource from "./diagnostics.ts?raw";
import { loadDiagnosticSummary } from "./diagnostics";
import type { DiagnosticSummaryDto } from "$lib/types/diagnostics";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

function diagnosticFixture(): DiagnosticSummaryDto {
  return {
    app: {
      appName: "extractum",
      appVersion: "0.1.0",
      buildMode: "debug",
      generatedAtUnix: 1_717_300_000,
    },
    database: {
      sqliteAvailable: true,
      migrations: {
        status: "current",
        expectedVersions: [1, 2, 3],
        appliedVersions: [1, 2, 3],
        pendingVersions: [],
        failedVersions: [],
      },
      accountCount: 2,
    },
    providers: {
      activeProvider: "gemini",
      profilesByProvider: [
        {
          provider: "gemini",
          configuredCount: 1,
          missingKeyCount: 0,
        },
      ],
    },
    runtimes: {
      ytdlp: {
        status: "available",
        available: true,
        version: "2026.01.01",
        summary: null,
      },
      secureStorage: {
        status: "available",
        available: true,
        version: null,
        summary: null,
      },
    },
    telegram: {
      accountCount: 2,
      runtimeStatuses: [{ status: "ready", count: 2 }],
    },
    sources: {
      counts: [
        {
          sourceType: "telegram",
          sourceSubtype: "supergroup",
          active: true,
          syncState: "synced",
          count: 3,
        },
      ],
    },
    items: {
      counts: [
        {
          sourceType: "youtube",
          sourceSubtype: "video",
          itemKind: "youtube_comment",
          contentKind: "text_only",
          hasContent: true,
          hasMedia: false,
          mediaKind: null,
          count: 7,
        },
      ],
    },
    analysisRuns: {
      counts: [
        {
          provider: "gemini",
          runType: "report",
          scopeType: "single_source",
          status: "failed",
          snapshotState: "not_captured",
          errorKind: "network",
          count: 1,
        },
      ],
    },
    llmRequests: {
      counts: [{ provider: "gemini", kind: "analysis_report_map", state: "running", count: 1 }],
    },
    youtubeJobs: {
      counts: [
        {
          jobType: "youtube_video_full_sync",
          status: "failed",
          warningState: "none",
          errorKind: "network",
          count: 1,
        },
      ],
    },
    ingest: {
      batches: [
        {
          provider: "telegram",
          ingestKind: "takeout",
          status: "completed",
          completeness: "complete",
          errorKind: "none",
          count: 1,
        },
      ],
      warnings: [
        {
          provider: "telegram",
          ingestKind: "takeout",
          status: "completed",
          warningCode: "export_dc_fallback",
          count: 2,
        },
      ],
    },
    privacy: {
      excludedDataClasses: ["source_content", "api_keys", "local_database_path"],
    },
  };
}

function collectKeys(value: unknown, keys = new Set<string>()) {
  if (!value || typeof value !== "object") return keys;
  if (Array.isArray(value)) {
    for (const item of value) collectKeys(item, keys);
    return keys;
  }
  for (const [key, child] of Object.entries(value)) {
    keys.add(key);
    collectKeys(child, keys);
  }
  return keys;
}

describe("diagnostics api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads the diagnostic summary through the dedicated Tauri command", async () => {
    const fixture = diagnosticFixture();
    invokeMock.mockResolvedValueOnce(fixture);

    await expect(loadDiagnosticSummary()).resolves.toBe(fixture);

    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith("get_diagnostic_summary");
    expect(typeof fixture.app.generatedAtUnix).toBe("number");
  });

  it("keeps the API wrapper narrow and does not add detail-ish fields", async () => {
    const fixture = diagnosticFixture();
    invokeMock.mockResolvedValueOnce(fixture);

    const result = await loadDiagnosticSummary();

    expect(result).toEqual(fixture);
    expect(collectKeys(result)).not.toContain("sourceId");
    expect(collectKeys(result)).not.toContain("profileId");
    expect(collectKeys(result)).not.toContain("baseUrl");
    expect(collectKeys(result)).not.toContain("url");
    expect(collectKeys(result)).not.toContain("path");
    expect(collectKeys(result)).not.toContain("raw");
    expect(collectKeys(result)).not.toContain("payload");
    expect(collectKeys(result)).not.toContain("log");
    expect(collectKeys(result)).not.toContain("stack");
  });

  it("does not log or map raw unknown command errors in the wrapper", () => {
    expect(diagnosticsApiSource).not.toContain("console.error");
    expect(diagnosticsApiSource).not.toContain("JSON.stringify");
    expect(diagnosticsApiSource).not.toContain(".then(");
  });
});
```

- [x] **Step 2: Run the API test to verify it fails**

Run: `npm.cmd run test -- src/lib/api/diagnostics.test.ts`

Expected: FAIL because `src/lib/api/diagnostics.ts` and `src/lib/types/diagnostics.ts` do not exist yet.

- [x] **Step 3: Add DTO types**

Create `src/lib/types/diagnostics.ts`:

```ts
export interface DiagnosticSummaryDto {
  app: DiagnosticAppInfo;
  database: DiagnosticDatabaseInfo;
  providers: DiagnosticProvidersInfo;
  runtimes: DiagnosticRuntimeInfo;
  telegram: DiagnosticTelegramInfo;
  sources: DiagnosticSourcesInfo;
  items: DiagnosticItemsInfo;
  analysisRuns: DiagnosticAnalysisRunsInfo;
  llmRequests: DiagnosticLlmRequestsInfo;
  youtubeJobs: DiagnosticYoutubeJobsInfo;
  ingest: DiagnosticIngestInfo;
  privacy: DiagnosticPrivacyInfo;
}

export interface DiagnosticAppInfo {
  appName: string;
  appVersion: string;
  buildMode: string;
  generatedAtUnix: number;
}

export interface DiagnosticDatabaseInfo {
  sqliteAvailable: boolean;
  migrations: DiagnosticMigrationInfo;
  accountCount: number;
}

export interface DiagnosticMigrationInfo {
  status: string;
  expectedVersions: number[];
  appliedVersions: number[];
  pendingVersions: number[];
  failedVersions: number[];
}

export interface DiagnosticProvidersInfo {
  activeProvider: string | null;
  profilesByProvider: DiagnosticProviderProfileCount[];
}

export interface DiagnosticProviderProfileCount {
  provider: string;
  configuredCount: number;
  missingKeyCount: number;
}

export interface DiagnosticRuntimeInfo {
  ytdlp: DiagnosticRuntimeCheck;
  secureStorage: DiagnosticRuntimeCheck;
}

export interface DiagnosticRuntimeCheck {
  status: string;
  available: boolean;
  version: string | null;
  summary: string | null;
}

export interface DiagnosticTelegramInfo {
  accountCount: number;
  runtimeStatuses: DiagnosticStatusCount[];
}

export interface DiagnosticStatusCount {
  status: string;
  count: number;
}

export interface DiagnosticSourcesInfo {
  counts: DiagnosticSourceCount[];
}

export interface DiagnosticSourceCount {
  sourceType: string;
  sourceSubtype: string | null;
  active: boolean;
  syncState: string;
  count: number;
}

export interface DiagnosticItemsInfo {
  counts: DiagnosticItemCount[];
}

export interface DiagnosticItemCount {
  sourceType: string;
  sourceSubtype: string | null;
  itemKind: string;
  contentKind: string;
  hasContent: boolean;
  hasMedia: boolean;
  mediaKind: string | null;
  count: number;
}

export interface DiagnosticAnalysisRunsInfo {
  counts: DiagnosticAnalysisRunCount[];
}

export interface DiagnosticAnalysisRunCount {
  provider: string;
  runType: string;
  scopeType: string;
  status: string;
  snapshotState: string;
  errorKind: string;
  count: number;
}

export interface DiagnosticLlmRequestsInfo {
  counts: DiagnosticLlmRequestCount[];
}

export interface DiagnosticLlmRequestCount {
  provider: string;
  kind: string;
  state: string;
  count: number;
}

export interface DiagnosticYoutubeJobsInfo {
  counts: DiagnosticYoutubeJobCount[];
}

export interface DiagnosticYoutubeJobCount {
  jobType: string;
  status: string;
  warningState: string;
  errorKind: string;
  count: number;
}

export interface DiagnosticIngestInfo {
  batches: DiagnosticIngestBatchCount[];
  warnings: DiagnosticIngestWarningCount[];
}

export interface DiagnosticIngestBatchCount {
  provider: string;
  ingestKind: string;
  status: string;
  completeness: string;
  errorKind: string;
  count: number;
}

export interface DiagnosticIngestWarningCount {
  provider: string;
  ingestKind: string;
  status: string;
  warningCode: string;
  count: number;
}

export interface DiagnosticPrivacyInfo {
  excludedDataClasses: string[];
}
```

- [x] **Step 4: Add the narrow API wrapper**

Create `src/lib/api/diagnostics.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { DiagnosticSummaryDto } from "$lib/types/diagnostics";

export function loadDiagnosticSummary() {
  return invoke<DiagnosticSummaryDto>("get_diagnostic_summary");
}
```

- [x] **Step 5: Run the API wrapper test**

Run: `npm.cmd run test -- src/lib/api/diagnostics.test.ts`

Expected: PASS.

- [x] **Step 6: Commit API boundary**

```bash
git add src/lib/types/diagnostics.ts src/lib/api/diagnostics.ts src/lib/api/diagnostics.test.ts
git commit -m "feat: add diagnostics API wrapper"
```

---

### Task 2: Diagnostics View-Model Helpers

**Files:**
- Create: `src/lib/diagnostics-view-model.ts`
- Create: `src/lib/diagnostics-view-model.test.ts`

- [x] **Step 1: Write the failing helper tests**

Create `src/lib/diagnostics-view-model.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  buildModeTone,
  emptySectionRows,
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
    for (const status of ["available", "current", "synced", "ready", "succeeded", "completed", "complete", "none"]) {
      expect(statusTone(status)).toBe("success");
    }
    for (const status of ["pending", "queued", "running", "cancel_requested", "partial", "present"]) {
      expect(statusTone(status)).toBe("info");
    }
    for (const status of ["never_synced", "missing_key", "not_configured", "unavailable", "not_found", "timed_out", "cancelled"]) {
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
});
```

- [x] **Step 2: Run helper tests to verify they fail**

Run: `npm.cmd run test -- src/lib/diagnostics-view-model.test.ts`

Expected: FAIL because `src/lib/diagnostics-view-model.ts` does not exist yet.

- [x] **Step 3: Add the pure view-model helpers**

Create `src/lib/diagnostics-view-model.ts`:

```ts
import { formatAppError, type AppErrorKind } from "$lib/app-error";
import type { BadgeVariant } from "$lib/components/ui/types";

type CountRowValue = string | number | boolean | null | undefined;
type CountRow = Record<string, CountRowValue> & { count: number };
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

export const DIAGNOSTICS_PRIVACY_FALLBACK_NOTE =
  "This diagnostics view is designed to show sanitized fields only. The backend did not report excluded data classes for this summary.";

function normalizedKey(value: unknown) {
  return typeof value === "string" ? value.trim().toLowerCase() : "";
}

function isAppErrorKind(value: unknown): value is AppErrorKind {
  return typeof value === "string" && APP_ERROR_KINDS.has(value as AppErrorKind);
}

function toDiagnosticAppErrorObject(value: unknown): DiagnosticAppErrorPayload | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;

  const candidate = value as Record<string, unknown>;
  const keys = Object.keys(candidate);
  if (keys.some((key) => DETAILISH_ERROR_KEYS.has(key))) return null;
  if (keys.some((key) => key !== "kind" && key !== "message")) return null;
  if (!isAppErrorKind(candidate.kind) || typeof candidate.message !== "string") return null;

  const message = candidate.message.trim();
  if (!message) return null;
  return { kind: candidate.kind, message };
}

function toDiagnosticAppError(value: unknown): DiagnosticAppErrorPayload | null {
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) return null;
    try {
      return toDiagnosticAppErrorObject(JSON.parse(trimmed));
    } catch {
      return null;
    }
  }

  return toDiagnosticAppErrorObject(value);
}

function pad2(value: number) {
  return String(value).padStart(2, "0");
}

export function statusTone(status: string | null | undefined): BadgeVariant {
  const key = normalizedKey(status);
  if (SUCCESS_STATUSES.has(key)) return "success";
  if (INFO_STATUSES.has(key)) return "info";
  if (WARNING_STATUSES.has(key)) return "warning";
  if (DANGER_STATUSES.has(key)) return "danger";
  return "neutral";
}

export function buildModeTone(buildMode: string | null | undefined): BadgeVariant {
  const key = normalizedKey(buildMode);
  if (key === "release") return "success";
  if (key === "debug") return "info";
  return "neutral";
}

export function availabilityTone(available: boolean): BadgeVariant {
  return available ? "success" : "danger";
}

export function availabilityLabel(available: boolean) {
  return available ? "Available" : "Unavailable";
}

export function labelFromKey(value: string | null | undefined) {
  const key = (value ?? "").trim();
  if (!key) return "Unknown";
  const label = key.replace(/[_-]+/g, " ");
  return `${label.charAt(0).toUpperCase()}${label.slice(1)}`;
}

export function yesNo(value: boolean) {
  return value ? "Yes" : "No";
}

export function formatSummaryGeneratedAt(value: unknown) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return "Summary generated Unknown";
  }

  const date = new Date(value * 1000);
  if (!Number.isFinite(date.getTime())) {
    return "Summary generated Unknown";
  }

  const timestamp = [
    date.getUTCFullYear(),
    pad2(date.getUTCMonth() + 1),
    pad2(date.getUTCDate()),
  ].join("-");
  const time = [
    pad2(date.getUTCHours()),
    pad2(date.getUTCMinutes()),
    pad2(date.getUTCSeconds()),
  ].join(":");

  return `Summary generated ${timestamp} ${time} UTC`;
}

function comparable(value: CountRowValue) {
  if (typeof value === "boolean") return value ? "1" : "0";
  if (value === null || value === undefined) return "";
  return String(value).trim().toLowerCase();
}

export function sortCountRows<T extends CountRow>(rows: readonly T[], keys: readonly (keyof T)[]) {
  return [...rows]
    .map((row, index) => ({ row, index }))
    .sort((left, right) => {
      for (const key of keys) {
        const compared = comparable(left.row[key]).localeCompare(comparable(right.row[key]));
        if (compared !== 0) return compared;
      }
      if (left.row.count !== right.row.count) return left.row.count - right.row.count;
      return left.index - right.index;
    })
    .map(({ row }) => row);
}

export function emptySectionRows<T>(rows: readonly T[], message = "No diagnostic counts reported") {
  return rows.length === 0 ? [{ empty: true, label: message }] : [];
}

export function privacyExcludedDataClasses(value: unknown) {
  if (!Array.isArray(value)) return [];
  return value
    .filter((item): item is string => typeof item === "string" && item.trim().length > 0)
    .map(labelFromKey);
}

export function privacyFallbackNote(value: unknown) {
  return privacyExcludedDataClasses(value).length === 0 ? DIAGNOSTICS_PRIVACY_FALLBACK_NOTE : "";
}

export function formatDiagnosticError(action: string, error: unknown) {
  const appError = toDiagnosticAppError(error);
  if (appError !== null) {
    return formatAppError(action, appError);
  }
  return `Error ${action}: Diagnostics could not be loaded.`;
}
```

- [x] **Step 4: Run helper tests**

Run: `npm.cmd run test -- src/lib/diagnostics-view-model.test.ts`

Expected: PASS.

`emptySectionRows()` is intentionally kept as a pure view-model helper even though the
first `DiagnosticCountTable` implementation may render its empty row through Svelte's
`{:else}` branch. It gives future diagnostics sections and source-contract tests a
single reusable empty-row contract without moving table rendering logic into the route.

- [x] **Step 5: Commit view-model helpers**

```bash
git add src/lib/diagnostics-view-model.ts src/lib/diagnostics-view-model.test.ts
git commit -m "feat: add diagnostics view model helpers"
```

---

### Task 3: Diagnostics Source Contracts

**Files:**
- Create: `src/lib/diagnostics-route-contract.test.ts`

- [x] **Step 1: Write the failing production-source contract tests**

Create `src/lib/diagnostics-route-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import layoutSource from "../routes/+layout.svelte?raw";
import diagnosticsPageSource from "../routes/diagnostics/+page.svelte?raw";
import diagnosticsTableSource from "./components/diagnostics/DiagnosticCountTable.svelte?raw";
import settingsPageSource from "../routes/settings/+page.svelte?raw";

const productionSources = [diagnosticsPageSource, diagnosticsTableSource];

describe("diagnostics frontend source contracts", () => {
  it("keeps Tauri invocation inside the diagnostics API wrapper", () => {
    expect(diagnosticsPageSource).toContain(
      'import { loadDiagnosticSummary } from "$lib/api/diagnostics";',
    );
    expect(diagnosticsPageSource).not.toContain("invoke(");
  });

  it("keeps raw payload, log, and copy affordances out of diagnostics production UI", () => {
    const forbidden = [
      "JSON.stringify",
      "Raw JSON",
      "Copy payload",
      "Copy JSON",
      "Copy logs",
      "Copy table",
      "Copy section",
      "Copy summary",
      "stack trace",
      "console.error",
    ];

    for (const source of productionSources) {
      for (const token of forbidden) {
        expect(source).not.toContain(token);
      }
    }
  });

  it("loads diagnostics only from mount and manual refresh state", () => {
    expect(diagnosticsPageSource).toMatch(/import\s*\{\s*onMount\s*\}\s*from\s*"svelte"/);
    expect(diagnosticsPageSource).toMatch(/onMount\s*\(\s*\(\)\s*=>/);
    expect(diagnosticsPageSource).toMatch(/refreshDiagnostics\s*\(\s*true\s*\)/);
    expect(diagnosticsPageSource).toMatch(/refreshDiagnostics\s*\(\s*false\s*\)/);
    expect(diagnosticsPageSource).not.toContain("export const load");
    expect(diagnosticsPageSource).not.toContain("setInterval");
  });

  it("keeps refresh failure state separate from the last successful summary", () => {
    expect(diagnosticsPageSource).toMatch(/let\s+summary\s*=\s*\$state(?:\s*<[^>]+>)?\s*\(\s*null\s*\)/);
    expect(diagnosticsPageSource).toMatch(/let\s+loading\s*=\s*\$state\s*\(\s*true\s*\)/);
    expect(diagnosticsPageSource).toMatch(/let\s+refreshing\s*=\s*\$state\s*\(\s*false\s*\)/);
    expect(diagnosticsPageSource).toMatch(/let\s+status\s*=\s*\$state\s*\(\s*""\s*\)/);
    expect(diagnosticsPageSource).toMatch(/let\s+error\s*=\s*\$state(?:\s*<[^>]+>)?\s*\(\s*null\s*\)/);
    expect(diagnosticsPageSource).toMatch(/if\s*\(\s*initial\s*\)\s*(?:\{\s*)?summary\s*=\s*null\s*;/);
  });

  it("does not render duplicate initial loading status", () => {
    expect(diagnosticsPageSource).toMatch(/if\s*\(\s*initial\s*\)\s*\{[\s\S]*status\s*=\s*"";/);
    expect(diagnosticsPageSource).toMatch(/else\s*\{[\s\S]*status\s*=\s*"Refreshing\.\.\.";/);
  });

  it("keeps privacy fallback tolerant of partial privacy payloads", () => {
    expect(diagnosticsPageSource).toContain("function privacyLabels");
    expect(diagnosticsPageSource).toContain("function privacyNote");
    expect(diagnosticsPageSource).toMatch(/summary\.privacy\?\.\s*excludedDataClasses/);
    expect(diagnosticsPageSource).not.toContain("{@const excludedClasses");
    expect(diagnosticsPageSource).not.toContain("{@const fallbackNote");
  });

  it("adds Diagnostics navigation without moving diagnostics into Settings", () => {
    expect(layoutSource).toContain("ShieldCheck");
    expect(layoutSource).toContain('label: "Diagnostics"');
    expect(layoutSource).toContain('caption: "Local health"');
    expect(layoutSource).toContain('pathname.startsWith("/diagnostics")');
    expect(layoutSource).toContain("Diagnostics");
    expect(settingsPageSource).not.toContain("$lib/api/diagnostics");
    expect(settingsPageSource).not.toContain("/diagnostics");
    expect(settingsPageSource).not.toContain("loadDiagnosticSummary");
  });
});
```

- [x] **Step 2: Run source contract tests to verify they fail**

Run: `npm.cmd run test -- src/lib/diagnostics-route-contract.test.ts`

Expected: FAIL because the diagnostics route and diagnostics table component do not exist yet, and navigation lacks `Diagnostics`.

- [x] **Step 3: Commit the failing contract tests only if your workflow records red tests**

If the implementation session keeps red tests in commits, run:

```bash
git add src/lib/diagnostics-route-contract.test.ts
git commit -m "test: cover diagnostics frontend contracts"
```

If the implementation session does not commit red tests, keep this file staged or unstaged until Task 6 passes.

---

### Task 4: Navigation And App Topbar

**Files:**
- Modify: `src/routes/+layout.svelte`
- Test: `src/lib/diagnostics-route-contract.test.ts`

- [x] **Step 1: Add `ShieldCheck` to the layout imports**

Modify the lucide import in `src/routes/+layout.svelte`:

```svelte
import { LayoutDashboard, Menu, Moon, Settings, ShieldCheck, Sun, UserRound } from "@lucide/svelte";
```

- [x] **Step 2: Add Diagnostics to `navItems`**

Insert this entry between Accounts and Settings in `src/routes/+layout.svelte`:

```ts
    {
      href: "/diagnostics",
      label: "Diagnostics",
      caption: "Local health",
      icon: ShieldCheck,
      active: (pathname: string) => pathname.startsWith("/diagnostics"),
    },
```

- [x] **Step 3: Add the app topbar route label**

Update the topbar route label branch in `src/routes/+layout.svelte` so the diagnostics case is before Settings:

```svelte
              {#if page.url.pathname.startsWith("/analysis")}
                Analysis workspace
              {:else if page.url.pathname.startsWith("/accounts") || page.url.pathname.startsWith("/auth")}
                Source access
              {:else if page.url.pathname.startsWith("/diagnostics")}
                Diagnostics
              {:else if page.url.pathname.startsWith("/settings")}
                Settings
              {:else}
                Extractum
              {/if}
```

- [x] **Step 4: Run the navigation contract test**

Run: `npm.cmd run test -- src/lib/diagnostics-route-contract.test.ts`

Expected: still FAIL because the diagnostics route and diagnostics table component are not created yet, but the navigation assertions should pass once the missing imports are resolved in later tasks.

- [x] **Step 5: Commit navigation when paired with passing route contracts**

Do not commit this task by itself if `src/lib/diagnostics-route-contract.test.ts` is still failing. Commit it together with Task 6 after the full contract test passes.

---

### Task 5: Diagnostics Count Table Component

**Files:**
- Create: `src/lib/components/diagnostics/DiagnosticCountTable.svelte`
- Test: `src/lib/diagnostics-route-contract.test.ts`

- [x] **Step 1: Create the diagnostics component directory and table component**

Create `src/lib/components/diagnostics/DiagnosticCountTable.svelte`:

```svelte
<script lang="ts">
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";

  type DiagnosticTableValue = string | number;
  type DiagnosticTableRow = Record<string, DiagnosticTableValue>;
  type DiagnosticTableColumn = {
    key: string;
    label: string;
    align?: "start" | "end";
  };

  let {
    title,
    description = "",
    columns,
    rows,
    emptyMessage = "No diagnostic counts reported",
  }: {
    title: string;
    description?: string;
    columns: DiagnosticTableColumn[];
    rows: DiagnosticTableRow[];
    emptyMessage?: string;
  } = $props();

  function cellValue(row: DiagnosticTableRow, key: string) {
    return row[key] ?? "";
  }

  function rowKey(row: DiagnosticTableRow, index: number) {
    const key = columns.map((column) => String(cellValue(row, column.key))).join("|");
    return key || String(index);
  }
</script>

<SurfaceCard {title} meta={description} className="diagnostic-count-table">
  <div class="table-scroll">
    <table>
      <thead>
        <tr>
          {#each columns as column (column.key)}
            <th class:align-end={column.align === "end"}>{column.label}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each rows as row, index (rowKey(row, index))}
          <tr>
            {#each columns as column (column.key)}
              <td class:align-end={column.align === "end"}>{cellValue(row, column.key)}</td>
            {/each}
          </tr>
        {:else}
          <tr>
            <td class="empty-row" colspan={columns.length}>{emptyMessage}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</SurfaceCard>

<style>
  :global(.diagnostic-count-table.ui-surface-card) {
    gap: 0.7rem;
  }

  .table-scroll {
    overflow-x: auto;
  }

  table {
    width: 100%;
    min-width: 520px;
    border-collapse: collapse;
    font-size: 0.86rem;
  }

  th,
  td {
    padding: 0.55rem 0.45rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
    text-align: left;
    vertical-align: top;
  }

  th {
    color: var(--muted);
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  tr:last-child td {
    border-bottom: 0;
  }

  .align-end {
    text-align: right;
  }

  .empty-row {
    color: var(--muted);
    text-align: left;
  }
</style>
```

- [x] **Step 2: Run Svelte check for the new component**

Run: `npm.cmd run check`

Expected: may still FAIL until the route is created if the source contract test import is present; if `svelte-check` reports a component syntax error, fix the component before moving on.

- [x] **Step 3: Keep component changes for Task 6**

Do not commit this component by itself while the source contract test still fails on the missing route.

---

### Task 6: Diagnostics Route

**Files:**
- Create: `src/routes/diagnostics/+page.svelte`
- Test: `src/lib/diagnostics-route-contract.test.ts`

- [x] **Step 0: Confirm existing UI component contracts**

Before writing the route, check `src/lib/components/ui/StatusMessage.svelte`,
`src/lib/components/ui/Badge.svelte`, and `src/lib/components/ui/types.ts`. The current
codebase supports `StatusMessage tone="error"`, `StatusMessage tone="muted"`,
`surface={false}`, and `Badge variant="neutral"`. If the implementation branch has
renamed these props or tones, adapt the route to the existing component contracts
without introducing new tone names.

- [x] **Step 1: Create the diagnostics route**

Create `src/routes/diagnostics/+page.svelte` with this structure:

```svelte
<script lang="ts">
  import { RefreshCw } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { loadDiagnosticSummary } from "$lib/api/diagnostics";
  import DiagnosticCountTable from "$lib/components/diagnostics/DiagnosticCountTable.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import MetaCell from "$lib/components/ui/MetaCell.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";
  import {
    availabilityLabel,
    availabilityTone,
    buildModeTone,
    formatDiagnosticError,
    formatSummaryGeneratedAt,
    labelFromKey,
    privacyExcludedDataClasses,
    privacyFallbackNote,
    sortCountRows,
    statusTone,
    yesNo,
  } from "$lib/diagnostics-view-model";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { DiagnosticRuntimeCheck, DiagnosticSummaryDto } from "$lib/types/diagnostics";

  type StatusStripItem = {
    label: string;
    value: string;
    tone: BadgeVariant;
    meta: string;
  };

  const sourceColumns = [
    { key: "sourceType", label: "Source" },
    { key: "sourceSubtype", label: "Subtype" },
    { key: "active", label: "Active" },
    { key: "syncState", label: "Sync" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const itemColumns = [
    { key: "sourceType", label: "Source" },
    { key: "sourceSubtype", label: "Subtype" },
    { key: "itemKind", label: "Item kind" },
    { key: "contentKind", label: "Content" },
    { key: "hasContent", label: "Has content" },
    { key: "hasMedia", label: "Has media" },
    { key: "mediaKind", label: "Media" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const runColumns = [
    { key: "provider", label: "Provider" },
    { key: "runType", label: "Run" },
    { key: "scopeType", label: "Scope" },
    { key: "status", label: "Status" },
    { key: "snapshotState", label: "Snapshot" },
    { key: "errorKind", label: "Error" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const llmColumns = [
    { key: "provider", label: "Provider" },
    { key: "kind", label: "Kind" },
    { key: "state", label: "State" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const youtubeJobColumns = [
    { key: "jobType", label: "Job" },
    { key: "status", label: "Status" },
    { key: "warningState", label: "Warning" },
    { key: "errorKind", label: "Error" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const ingestBatchColumns = [
    { key: "provider", label: "Provider" },
    { key: "ingestKind", label: "Kind" },
    { key: "status", label: "Status" },
    { key: "completeness", label: "Completeness" },
    { key: "errorKind", label: "Error" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const ingestWarningColumns = [
    { key: "provider", label: "Provider" },
    { key: "ingestKind", label: "Kind" },
    { key: "status", label: "Status" },
    { key: "warningCode", label: "Warning" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const providerColumns = [
    { key: "provider", label: "Provider" },
    { key: "configuredCount", label: "Configured", align: "end" as const },
    { key: "missingKeyCount", label: "Missing keys", align: "end" as const },
  ];

  const telegramColumns = [
    { key: "status", label: "Runtime status" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  let summary = $state<DiagnosticSummaryDto | null>(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let status = $state("");
  let error = $state<string | null>(null);

  async function refreshDiagnostics(initial: boolean) {
    if (initial) {
      loading = true;
      status = "";
    } else {
      refreshing = true;
      status = "Refreshing...";
    }
    error = null;

    try {
      summary = await loadDiagnosticSummary();
      status = "";
    } catch (caught) {
      error = formatDiagnosticError("loading diagnostics", caught);
      status = "";
      if (initial) summary = null;
    } finally {
      if (initial) {
        loading = false;
      } else {
        refreshing = false;
      }
    }
  }

  onMount(() => {
    void refreshDiagnostics(true);
  });

  function runtimeMeta(runtime: DiagnosticRuntimeCheck) {
    return runtime.version ?? runtime.summary ?? labelFromKey(runtime.status);
  }

  function statusStripItems(current: DiagnosticSummaryDto): StatusStripItem[] {
    return [
      {
        label: "SQLite",
        value: availabilityLabel(current.database.sqliteAvailable),
        tone: availabilityTone(current.database.sqliteAvailable),
        meta: `${current.database.accountCount} accounts`,
      },
      {
        label: "Migrations",
        value: labelFromKey(current.database.migrations.status),
        tone: statusTone(current.database.migrations.status),
        meta: `${current.database.migrations.appliedVersions.length}/${current.database.migrations.expectedVersions.length} applied`,
      },
      {
        label: "Secure storage",
        value: labelFromKey(current.runtimes.secureStorage.status),
        tone: statusTone(current.runtimes.secureStorage.status),
        meta: availabilityLabel(current.runtimes.secureStorage.available),
      },
      {
        label: "yt-dlp",
        value: labelFromKey(current.runtimes.ytdlp.status),
        tone: statusTone(current.runtimes.ytdlp.status),
        meta: runtimeMeta(current.runtimes.ytdlp),
      },
    ];
  }

  function providerRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.providers.profilesByProvider, ["provider"]).map((row) => ({
      provider: labelFromKey(row.provider),
      configuredCount: row.configuredCount,
      missingKeyCount: row.missingKeyCount,
    }));
  }

  function telegramRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.telegram.runtimeStatuses, ["status"]).map((row) => ({
      status: labelFromKey(row.status),
      count: row.count,
    }));
  }

  function sourceRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.sources.counts, ["sourceType", "sourceSubtype", "active", "syncState"]).map((row) => ({
      sourceType: labelFromKey(row.sourceType),
      sourceSubtype: labelFromKey(row.sourceSubtype),
      active: yesNo(row.active),
      syncState: labelFromKey(row.syncState),
      count: row.count,
    }));
  }

  function itemRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.items.counts, ["sourceType", "sourceSubtype", "itemKind", "contentKind", "hasContent", "hasMedia", "mediaKind"]).map((row) => ({
      sourceType: labelFromKey(row.sourceType),
      sourceSubtype: labelFromKey(row.sourceSubtype),
      itemKind: labelFromKey(row.itemKind),
      contentKind: labelFromKey(row.contentKind),
      hasContent: yesNo(row.hasContent),
      hasMedia: yesNo(row.hasMedia),
      mediaKind: labelFromKey(row.mediaKind),
      count: row.count,
    }));
  }

  function runRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.analysisRuns.counts, ["provider", "runType", "scopeType", "status", "snapshotState", "errorKind"]).map((row) => ({
      provider: labelFromKey(row.provider),
      runType: labelFromKey(row.runType),
      scopeType: labelFromKey(row.scopeType),
      status: labelFromKey(row.status),
      snapshotState: labelFromKey(row.snapshotState),
      errorKind: labelFromKey(row.errorKind),
      count: row.count,
    }));
  }

  function llmRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.llmRequests.counts, ["provider", "kind", "state"]).map((row) => ({
      provider: labelFromKey(row.provider),
      kind: labelFromKey(row.kind),
      state: labelFromKey(row.state),
      count: row.count,
    }));
  }

  function youtubeRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.youtubeJobs.counts, ["jobType", "status", "warningState", "errorKind"]).map((row) => ({
      jobType: labelFromKey(row.jobType),
      status: labelFromKey(row.status),
      warningState: labelFromKey(row.warningState),
      errorKind: labelFromKey(row.errorKind),
      count: row.count,
    }));
  }

  function ingestBatchRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.ingest.batches, ["provider", "ingestKind", "status", "completeness", "errorKind"]).map((row) => ({
      provider: labelFromKey(row.provider),
      ingestKind: labelFromKey(row.ingestKind),
      status: labelFromKey(row.status),
      completeness: labelFromKey(row.completeness),
      errorKind: labelFromKey(row.errorKind),
      count: row.count,
    }));
  }

  function ingestWarningRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.ingest.warnings, ["provider", "ingestKind", "status", "warningCode"]).map((row) => ({
      provider: labelFromKey(row.provider),
      ingestKind: labelFromKey(row.ingestKind),
      status: labelFromKey(row.status),
      warningCode: labelFromKey(row.warningCode),
      count: row.count,
    }));
  }

  function privacyLabels(current: DiagnosticSummaryDto) {
    return privacyExcludedDataClasses(current.privacy?.excludedDataClasses);
  }

  function privacyNote(current: DiagnosticSummaryDto) {
    return privacyFallbackNote(current.privacy?.excludedDataClasses);
  }
</script>

<section class="page-shell diagnostics-page">
  <header class="page-hero">
    <div class="page-hero-copy">
      <span class="page-eyebrow">Operator diagnostics</span>
      <h1>Diagnostics</h1>
      <p>Sanitized local health summary for app, storage, runtimes, providers, sources, and ingest.</p>
      {#if summary}
        <p class="diagnostics-meta">
          v{summary.app.appVersion} - {labelFromKey(summary.app.buildMode)} - {formatSummaryGeneratedAt(summary.app.generatedAtUnix)}
        </p>
      {/if}
    </div>
    <div class="page-hero-meta">
      {#if summary}
        <Badge variant={buildModeTone(summary.app.buildMode)}>{labelFromKey(summary.app.buildMode)}</Badge>
        <Badge variant="neutral">{summary.app.appName}</Badge>
      {/if}
      <Button
        size="sm"
        variant="secondary"
        disabled={loading || refreshing}
        onclick={() => void refreshDiagnostics(false)}
      >
        <RefreshCw size={14} aria-hidden="true" />
        Refresh
      </Button>
    </div>
  </header>

  {#if status}
    <StatusMessage tone="info" className="page-status">{status}</StatusMessage>
  {/if}

  {#if error}
    <StatusMessage tone="error" className="page-status">{error}</StatusMessage>
  {/if}

  {#if summary}
    <div class="status-strip" aria-label="Diagnostics health overview">
      {#each statusStripItems(summary) as item (item.label)}
        <div class="status-tile">
          <span>{item.label}</span>
          <strong>{item.value}</strong>
          <Badge variant={item.tone}>{item.meta}</Badge>
        </div>
      {/each}
    </div>

    <div class="diagnostics-grid">
      <SurfaceCard title="App and build" meta="Factual diagnostic summary metadata">
        <div class="meta-grid">
          <MetaCell label="App">{summary.app.appName}</MetaCell>
          <MetaCell label="Version">{summary.app.appVersion}</MetaCell>
          <MetaCell label="Build">{labelFromKey(summary.app.buildMode)}</MetaCell>
          <MetaCell label="Generated">{formatSummaryGeneratedAt(summary.app.generatedAtUnix).replace("Summary generated ", "")}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Database" meta="SQLite availability and migration state">
        <div class="meta-grid">
          <MetaCell label="SQLite">{availabilityLabel(summary.database.sqliteAvailable)}</MetaCell>
          <MetaCell label="Migrations">{labelFromKey(summary.database.migrations.status)}</MetaCell>
          <MetaCell label="Accounts">{summary.database.accountCount}</MetaCell>
          <MetaCell label="Pending versions">{summary.database.migrations.pendingVersions.length}</MetaCell>
          <MetaCell label="Failed versions">{summary.database.migrations.failedVersions.length}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Runtimes" meta="Backend-reported runtime checks">
        <div class="meta-grid">
          <MetaCell label="Secure storage">{labelFromKey(summary.runtimes.secureStorage.status)}</MetaCell>
          <MetaCell label="Secure storage available">{availabilityLabel(summary.runtimes.secureStorage.available)}</MetaCell>
          <MetaCell label="yt-dlp">{labelFromKey(summary.runtimes.ytdlp.status)}</MetaCell>
          <MetaCell label="yt-dlp available">{availabilityLabel(summary.runtimes.ytdlp.available)}</MetaCell>
          <MetaCell label="yt-dlp version">{summary.runtimes.ytdlp.version ?? "Unknown"}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Privacy boundary" meta="Data classes intentionally excluded by backend diagnostics">
        {#if privacyLabels(summary).length > 0}
          <div class="privacy-chips">
            {#each privacyLabels(summary) as item (item)}
              <Badge variant="neutral">{item}</Badge>
            {/each}
          </div>
        {:else}
          <StatusMessage tone="muted" surface={false}>{privacyNote(summary)}</StatusMessage>
        {/if}
      </SurfaceCard>
    </div>

    <div class="diagnostics-tables">
      <DiagnosticCountTable title="Provider profiles" description="Configured profile counts by provider" columns={providerColumns} rows={providerRows(summary)} />
      <DiagnosticCountTable title="Telegram runtimes" description="Account runtime statuses by coarse state" columns={telegramColumns} rows={telegramRows(summary)} />
      <DiagnosticCountTable title="Sources" description="Source counts by type, subtype, active state, and sync state" columns={sourceColumns} rows={sourceRows(summary)} />
      <DiagnosticCountTable title="Items" description="Item counts by coarse source and content fields" columns={itemColumns} rows={itemRows(summary)} />
      <DiagnosticCountTable title="Analysis runs" description="Run counts by provider, scope, status, snapshot state, and error kind" columns={runColumns} rows={runRows(summary)} />
      <DiagnosticCountTable title="LLM requests" description="Request counts by provider, kind, and state" columns={llmColumns} rows={llmRows(summary)} />
      <DiagnosticCountTable title="YouTube jobs" description="Job aggregates by type, status, warning state, and error kind" columns={youtubeJobColumns} rows={youtubeRows(summary)} />
      <DiagnosticCountTable title="Ingest batches" description="Batch aggregates by provider, kind, status, completeness, and error kind" columns={ingestBatchColumns} rows={ingestBatchRows(summary)} />
      <DiagnosticCountTable title="Ingest warnings" description="Warning aggregates by provider, kind, status, and warning code" columns={ingestWarningColumns} rows={ingestWarningRows(summary)} />
    </div>
  {:else if loading}
    <StatusMessage tone="muted" className="page-status">Loading diagnostics...</StatusMessage>
  {/if}
</section>

<style>
  .diagnostics-page {
    gap: 0.95rem;
  }

  .diagnostics-meta {
    font-size: 0.86rem;
  }

  .status-strip {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.7rem;
  }

  .status-tile {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.85rem;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--panel);
    box-shadow: var(--shadow-soft);
  }

  .status-tile span {
    color: var(--muted);
    font-size: 0.76rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .status-tile strong {
    font-size: 0.98rem;
  }

  .diagnostics-grid,
  .diagnostics-tables {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.9rem;
    align-items: start;
  }

  .meta-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.65rem;
  }

  .privacy-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  @media (max-width: 980px) {
    .status-strip,
    .diagnostics-grid,
    .diagnostics-tables {
      grid-template-columns: 1fr;
    }

    .meta-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
```

- [x] **Step 2: Run Svelte autofixer or Svelte check for route syntax**

Run: `npm.cmd run check`

Expected: PASS for Svelte syntax. If `svelte-check` reports only source-contract test failures, run the Vitest command in the next step to see the exact contract assertion.

- [x] **Step 3: Run diagnostics source contracts**

Run: `npm.cmd run test -- src/lib/diagnostics-route-contract.test.ts`

Expected: PASS.

- [x] **Step 4: Commit navigation, component, route, and source contracts**

```bash
git add src/routes/+layout.svelte src/routes/diagnostics/+page.svelte src/lib/components/diagnostics/DiagnosticCountTable.svelte src/lib/diagnostics-route-contract.test.ts
git commit -m "feat: add diagnostics route"
```

---

### Task 7: Focused Diagnostics Test Run

**Files:**
- Test: `src/lib/api/diagnostics.test.ts`
- Test: `src/lib/diagnostics-view-model.test.ts`
- Test: `src/lib/diagnostics-route-contract.test.ts`

- [x] **Step 1: Run all targeted diagnostics tests**

Run:

```bash
npm.cmd run test -- src/lib/api/diagnostics.test.ts src/lib/diagnostics-view-model.test.ts src/lib/diagnostics-route-contract.test.ts
```

Expected: PASS for all three diagnostics test files.

- [x] **Step 2: Fix any targeted failure at the smallest owner**

Use this routing:

- API wrapper failure: change only `src/lib/api/diagnostics.ts` or `src/lib/types/diagnostics.ts`.
- View-model failure: change only `src/lib/diagnostics-view-model.ts`.
- Source contract failure about navigation: change only `src/routes/+layout.svelte`.
- Source contract failure about forbidden production strings: change only `src/routes/diagnostics/+page.svelte` or `src/lib/components/diagnostics/DiagnosticCountTable.svelte`.

- [x] **Step 3: Commit focused fixes if the previous task commits did not already include them**

```bash
git add src/lib/api/diagnostics.ts src/lib/types/diagnostics.ts src/lib/diagnostics-view-model.ts src/routes/+layout.svelte src/routes/diagnostics/+page.svelte src/lib/components/diagnostics/DiagnosticCountTable.svelte src/lib/api/diagnostics.test.ts src/lib/diagnostics-view-model.test.ts src/lib/diagnostics-route-contract.test.ts
git commit -m "test: cover diagnostics frontend"
```

If there is no diff after Step 1, skip this commit.

---

### Task 8: Full Verification Gate

**Files:**
- No planned source changes.

- [x] **Step 1: Run Svelte/TypeScript check**

Run: `npm.cmd run check`

Expected: PASS.

- [x] **Step 2: Run project tests**

Run: `npm.cmd run test`

Expected: PASS.

- [x] **Step 3: Run full project verification**

Run: `npm.cmd run verify`

Expected: PASS.

- [x] **Step 4: Inspect final git state**

Run:

```bash
git status --short --branch
git log --oneline -6
```

Expected: branch is clean except for intentional uncommitted files the user explicitly asked to keep; recent commits include diagnostics API wrapper, view-model helpers, and diagnostics route work.

---

## Self-Review Checklist

- Spec coverage: API wrapper owns `invoke`, route calls wrapper only, view-model owns display helpers, route is client-only via `onMount`, Refresh is manual and disabled while loading, refresh failure keeps old summary, privacy panel remains visible, Settings stays separate, source contracts forbid raw/log/copy affordances.
- Privacy coverage: no route `invoke(`, no raw JSON rendering, no copy buttons, no frontend environment probes, no raw unknown error logging, no hidden URL/path/profile/source lookups.
- Type consistency: backend `generated_at_unix: i64` maps to frontend `app.generatedAtUnix: number`; `formatSummaryGeneratedAt(value)` validates runtime input; known status tone buckets and build-mode tones match the design spec.
- Verification order: targeted Vitest tests, Svelte check, full `npm.cmd run verify`.
