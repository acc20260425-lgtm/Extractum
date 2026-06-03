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
