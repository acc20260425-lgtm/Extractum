import { describe, expect, it } from "vitest";
import {
  membershipLabel,
  sourceCapabilities,
  sourceKindLabel,
} from "./source-capabilities";
import type { Source } from "./types/sources";

function source(overrides: Partial<Source>): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "channel",
    accountId: 1,
    externalId: "123",
    title: "Source",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
    telegramUsername: null,
    avatarDataUrl: null,
    ...overrides,
  };
}

describe("source capabilities", () => {
  it("keeps Telegram channel capabilities compatible", () => {
    const capabilities = sourceCapabilities(source({}));

    expect(capabilities).toEqual({
      canSync: true,
      canDelete: true,
      canImportArchive: true,
      hasTopics: false,
      requiresAccount: true,
      hasMembershipState: true,
      contentLabel: "messages",
    });
    expect(sourceKindLabel(source({}))).toBe("channel");
    expect(membershipLabel(source({ isMember: true }))).toBe("subscribed");
    expect(membershipLabel(source({ isMember: false }))).toBe("not subscribed");
  });

  it("allows Takeout import for every Telegram source kind", () => {
    for (const telegramSourceSubtype of ["channel", "supergroup", "group"] as const) {
      expect(sourceCapabilities(source({
        sourceSubtype: telegramSourceSubtype,
      })).canImportArchive).toBe(true);
    }
  });

  it("marks Telegram supergroups as topic and Takeout capable", () => {
    const capabilities = sourceCapabilities(source({
      sourceSubtype: "supergroup",
    }));

    expect(capabilities.hasTopics).toBe(true);
    expect(capabilities.canImportArchive).toBe(true);
    expect(sourceKindLabel(source({
      sourceSubtype: "supergroup",
    }))).toBe("supergroup");
  });

  it("derives Telegram behavior from canonical sourceSubtype", () => {
    const supergroup = source({
      sourceSubtype: "supergroup",
      isMember: false,
    });

    expect(sourceCapabilities(supergroup)).toMatchObject({
      canImportArchive: true,
      hasTopics: true,
    });
    expect(sourceKindLabel(supergroup)).toBe("supergroup");
    expect(membershipLabel(supergroup)).toBe("not a member");
  });

  it("syncs manual YouTube videos without Telegram assumptions", () => {
    const video = source({
      sourceType: "youtube",
      sourceSubtype: "video",
      accountId: null,
      externalId: "dQw4w9WgXcQ",
      isMember: false,
    });

    expect(sourceCapabilities(video)).toEqual({
      canSync: true,
      canDelete: true,
      canImportArchive: false,
      hasTopics: false,
      requiresAccount: false,
      hasMembershipState: false,
      contentLabel: "items",
    });
    expect(sourceKindLabel(video)).toBe("YouTube video");
    expect(membershipLabel(video)).toBe("");
  });

  it("syncs YouTube playlists through YouTube jobs", () => {
    const playlist = source({
      sourceType: "youtube",
      sourceSubtype: "playlist",
      accountId: null,
      isMember: false,
    });

    expect(sourceCapabilities(playlist)).toMatchObject({
      canSync: true,
      requiresAccount: false,
      contentLabel: "items",
    });
    expect(sourceKindLabel(playlist)).toBe("YouTube playlist");
  });
});
