import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import SourceActivityView from "./source-activity-view.svelte";
import type { Source } from "$lib/types/sources";

afterEach(cleanup);

function telegramSource(): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "supergroup",
    accountId: 7,
    externalId: "research-channel",
    title: "Research channel",
    lastSyncState: 4,
    lastSyncedAt: 1_700_000_000,
    isMember: true,
    isActive: true,
    createdAt: 1_699_000_000,
    telegramUsername: "research",
    avatarDataUrl: null,
    migratedHistoryStatus: "available",
    migratedHistoryDetectedAt: 1_699_500_000,
    migratedHistoryRefreshedAt: 1_700_000_000,
    migratedHistoryRowCount: 8,
    migratedHistoryImportCompleted: false,
  };
}

describe("analysis priority UX contract", () => {
  it("makes source activity the visible home for source operations", () => {
    render(SourceActivityView, {
      props: {
        source: telegramSource(),
        jobs: [],
        takeoutRecovery: null,
        sourceSyncDisabledReason: () => null,
        formatTimestamp: (value) => value === null ? "Never" : `time:${value}`,
        onSyncSource: vi.fn(),
        onSyncMetadata: vi.fn(),
        onSyncTranscript: vi.fn(),
        onSyncComments: vi.fn(),
        onStartTakeoutImport: vi.fn(),
        onStartMigratedHistoryImport: vi.fn(),
        onCancelSourceJob: vi.fn(),
      },
    });

    expect(screen.getByRole("region", { name: "Source activity" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync source" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Start Takeout import" })).toBeTruthy();
    expect(screen.getByRole("region", { name: "Detailed source jobs" })).toBeTruthy();
  });
});
