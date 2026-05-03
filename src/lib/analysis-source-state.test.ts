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
import type { SourceRecord } from "./types/sources";

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

function source(overrides: Partial<SourceRecord>): SourceRecord {
  return {
    id: 1,
    source_type: "telegram",
    telegram_source_kind: "channel",
    account_id: 1,
    external_id: "@extractum",
    title: "Extractum",
    last_sync_state: null,
    last_synced_at: null,
    is_member: true,
    is_active: true,
    created_at: 100,
    avatar_data_url: null,
    ...overrides,
  };
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
    expect(sourceKindLabel("channel")).toBe("channel");
    expect(sourceKindLabel("supergroup")).toBe("supergroup");
    expect(sourceKindLabel("group")).toBe("group");
    expect(sourceKindLabel("unknown")).toBe("telegram");

    expect(membershipLabel("channel", true)).toBe("subscribed");
    expect(membershipLabel("channel", false)).toBe("not subscribed");
    expect(membershipLabel("group", true)).toBe("member");
    expect(membershipLabel("group", false)).toBe("not a member");
  });

  it("derives a stable source initial from title, external id, or fallback", () => {
    expect(sourceInitial(source({ title: " alpha" }))).toBe("A");
    expect(sourceInitial(source({ title: null, external_id: " beta" }))).toBe("B");
    expect(sourceInitial(source({ title: "   ", external_id: "   " }))).toBe("#");
  });

  it("explains why sync is disabled until the linked account is ready", () => {
    expect(sourceSyncDisabledReason(source({ account_id: null }), {}))
      .toBe("Source is not linked to an account.");
    expect(sourceSyncDisabledReason(source({ account_id: 7 }), {}))
      .toBe("Initialize this account before syncing.");
    expect(sourceSyncDisabledReason(
      source({ account_id: 7 }),
      { 7: runtime({ account_id: 7, status: "not_initialized" }) },
    )).toBe("Initialize this account before syncing.");
    expect(sourceSyncDisabledReason(
      source({ account_id: 7 }),
      { 7: runtime({ account_id: 7, status: "restoring" }) },
    )).toBe("This account is still restoring.");
    expect(sourceSyncDisabledReason(
      source({ account_id: 7 }),
      { 7: runtime({ account_id: 7, status: "reauth_required" }) },
    )).toBe("Sign in to this account again before syncing.");
    expect(sourceSyncDisabledReason(
      source({ account_id: 7 }),
      { 7: runtime({ account_id: 7, status: "restore_failed", message: "expired" }) },
    )).toBe("expired");
    expect(sourceSyncDisabledReason(
      source({ account_id: 7 }),
      { 7: runtime({ account_id: 7, status: "restore_failed", message: null }) },
    )).toBe("The saved Telegram session could not be restored.");
    expect(sourceSyncDisabledReason(
      source({ account_id: 7 }),
      { 7: runtime({ account_id: 7, status: "ready" }) },
    )).toBeNull();
  });
});
