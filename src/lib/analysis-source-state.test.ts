import { describe, expect, it } from "vitest";
import {
  accountLabel,
  membershipLabel,
  runtimeBadge,
  runtimeStatus,
  sourceInitial,
  sourceKindLabel,
  sourceSyncDisabledReason,
} from "./analysis-source-state";
import type { AccountRecord, AccountRuntimeStatus } from "./types/accounts";
import type { Source } from "./types/sources";

function account(overrides: Partial<AccountRecord>): AccountRecord {
  return {
    id: 1,
    label: "Main account",
    api_id: 12345,
    phone: null,
    created_at: 100,
    ...overrides,
  };
}

function runtime(overrides: Partial<AccountRuntimeStatus>): AccountRuntimeStatus {
  return {
    account_id: 1,
    status: "ready",
    message: null,
    ...overrides,
  };
}

function source(overrides: Partial<Source>): Source {
  return Object.assign({
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "channel",
    accountId: 1,
    externalId: "@extractum",
    title: "Extractum",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
    telegramUsername: null,
    avatarDataUrl: null,
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
  }, overrides);
}

describe("analysis-source-state", () => {
  it("labels linked, missing, and unknown accounts", () => {
    const accounts = [account({ id: 7, label: "Research" })];

    expect(accountLabel(7, accounts)).toBe("Research");
    expect(accountLabel(null, accounts)).toBe("No account");
    expect(accountLabel(9, accounts)).toBe("Account #9");
  });

  it("looks up runtime status only for linked sources", () => {
    const statuses = {
      7: runtime({ account_id: 7, status: "restoring" }),
    };

    expect(runtimeStatus(7, statuses)).toEqual(statuses[7]);
    expect(runtimeStatus(8, statuses)).toBeNull();
    expect(runtimeStatus(null, statuses)).toBeNull();
  });

  it("maps runtime states to compact badges", () => {
    expect(runtimeBadge(null)).toBe("");
    expect(runtimeBadge(runtime({ status: "ready" }))).toBe("");
    expect(runtimeBadge(runtime({ status: "restoring" }))).toBe("restoring");
    expect(runtimeBadge(runtime({ status: "reauth_required" }))).toBe("sign-in needed");
    expect(runtimeBadge(runtime({ status: "restore_failed" }))).toBe("restore failed");
    expect(runtimeBadge(runtime({ status: "not_initialized" }))).toBe("offline");
  });

  it("formats Telegram source kind and membership labels", () => {
    expect(sourceKindLabel(source({
      sourceSubtype: "channel",
    }))).toBe("channel");
    expect(sourceKindLabel(source({
      sourceSubtype: "supergroup",
    }))).toBe("supergroup");
    expect(sourceKindLabel(source({
      sourceSubtype: "group",
    }))).toBe("group");

    expect(membershipLabel(source({
      sourceSubtype: "channel",
      isMember: true,
    }))).toBe("subscribed");
    expect(membershipLabel(source({
      sourceSubtype: "channel",
      isMember: false,
    }))).toBe("not subscribed");
    expect(membershipLabel(source({
      sourceSubtype: "group",
      isMember: true,
    }))).toBe("member");
    expect(membershipLabel(source({
      sourceSubtype: "group",
      isMember: false,
    }))).toBe("not a member");
  });

  it("derives a stable source initial from title, external id, or fallback", () => {
    expect(sourceInitial(source({ title: " alpha" }))).toBe("A");
    expect(sourceInitial(source({ title: null, externalId: " beta" }))).toBe("B");
    expect(sourceInitial(source({ title: "   ", externalId: "   " }))).toBe("#");
  });

  it("explains why sync is disabled until the linked account is ready", () => {
    expect(sourceSyncDisabledReason(source({ accountId: null }), {}))
      .toBe("Source is not linked to an account.");
    expect(sourceSyncDisabledReason(source({ accountId: 7 }), {}))
      .toBe("Initialize this account before syncing.");
    expect(sourceSyncDisabledReason(
      source({ accountId: 7 }),
      { 7: runtime({ account_id: 7, status: "not_initialized" }) },
    )).toBe("Initialize this account before syncing.");
    expect(sourceSyncDisabledReason(
      source({ accountId: 7 }),
      { 7: runtime({ account_id: 7, status: "restoring" }) },
    )).toBe("This account is still restoring.");
    expect(sourceSyncDisabledReason(
      source({ accountId: 7 }),
      { 7: runtime({ account_id: 7, status: "reauth_required" }) },
    )).toBe("Sign in to this account again before syncing.");
    expect(sourceSyncDisabledReason(
      source({ accountId: 7 }),
      { 7: runtime({ account_id: 7, status: "restore_failed", message: "expired" }) },
    )).toBe("expired");
    expect(sourceSyncDisabledReason(
      source({ accountId: 7 }),
      { 7: runtime({ account_id: 7, status: "restore_failed", message: null }) },
    )).toBe("The saved Telegram session could not be restored.");
    expect(sourceSyncDisabledReason(
      source({ accountId: 7 }),
      { 7: runtime({ account_id: 7, status: "ready" }) },
    )).toBeNull();
  });

  it("does not require Telegram account runtime for syncable manual YouTube sources", () => {
    expect(sourceSyncDisabledReason(source({
      sourceType: "youtube",
      sourceSubtype: "video",
      accountId: null,
      isMember: false,
    }), {})).toBeNull();
  });

  it("uses YouTube runtime status before allowing YouTube sync", () => {
    const youtubeVideoSource = source({
      sourceType: "youtube",
      sourceSubtype: "video",
      accountId: null,
      isMember: false,
    });

    expect(sourceSyncDisabledReason(youtubeVideoSource, {}, {
      ytdlpAvailable: false,
      ytdlpVersion: null,
      message: "yt-dlp is not available on PATH",
    })).toBe("yt-dlp is not available on PATH");

    expect(sourceSyncDisabledReason(youtubeVideoSource, {}, {
      ytdlpAvailable: true,
      ytdlpVersion: "2026.01.01",
      message: "yt-dlp is available",
    })).toBeNull();
  });
});
