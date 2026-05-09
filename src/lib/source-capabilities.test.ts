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
    telegramSourceKind: "channel",
    accountId: 1,
    externalId: "123",
    title: "Source",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
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
      canImportArchive: false,
      hasTopics: false,
      requiresAccount: true,
      hasMembershipState: true,
      contentLabel: "messages",
    });
    expect(sourceKindLabel(source({}))).toBe("channel");
    expect(membershipLabel(source({ isMember: true }))).toBe("subscribed");
    expect(membershipLabel(source({ isMember: false }))).toBe("not subscribed");
  });

  it("marks Telegram supergroups as topic and Takeout capable", () => {
    const capabilities = sourceCapabilities(source({
      sourceSubtype: "supergroup",
      telegramSourceKind: "supergroup",
    }));

    expect(capabilities.hasTopics).toBe(true);
    expect(capabilities.canImportArchive).toBe(true);
    expect(sourceKindLabel(source({
      sourceSubtype: "supergroup",
      telegramSourceKind: "supergroup",
    }))).toBe("supergroup");
  });

  it("syncs manual YouTube videos without Telegram assumptions", () => {
    const video = source({
      sourceType: "youtube",
      sourceSubtype: "video",
      telegramSourceKind: null,
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
      telegramSourceKind: null,
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
