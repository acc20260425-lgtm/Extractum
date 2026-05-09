import type {
  Source,
  SourceCapabilities,
  SourceSubtype,
  TelegramSourceKind,
} from "$lib/types/sources";

function telegramKind(source: Pick<Source, "telegramSourceKind" | "sourceSubtype">) {
  return source.telegramSourceKind ?? telegramSubtype(source.sourceSubtype);
}

function telegramSubtype(subtype: SourceSubtype | null): TelegramSourceKind | null {
  return subtype === "channel" || subtype === "supergroup" || subtype === "group"
    ? subtype
    : null;
}

export function sourceCapabilities(source: Source): SourceCapabilities {
  if (source.sourceType === "telegram") {
    const kind = telegramKind(source);
    return {
      canSync: true,
      canDelete: true,
      canImportArchive: kind === "supergroup",
      hasTopics: kind === "supergroup",
      requiresAccount: true,
      hasMembershipState: true,
      contentLabel: "messages",
    };
  }

  if (source.sourceType === "youtube") {
    return {
      canSync: true,
      canDelete: true,
      canImportArchive: false,
      hasTopics: false,
      requiresAccount: false,
      hasMembershipState: false,
      contentLabel: "videos",
    };
  }

  if (source.sourceType === "rss") {
    return {
      canSync: true,
      canDelete: true,
      canImportArchive: false,
      hasTopics: false,
      requiresAccount: false,
      hasMembershipState: false,
      contentLabel: "posts",
    };
  }

  if (source.sourceType === "forum") {
    return {
      canSync: true,
      canDelete: true,
      canImportArchive: false,
      hasTopics: true,
      requiresAccount: false,
      hasMembershipState: false,
      contentLabel: "posts",
    };
  }

  return {
    canSync: false,
    canDelete: true,
    canImportArchive: false,
    hasTopics: false,
    requiresAccount: false,
    hasMembershipState: false,
    contentLabel: "items",
  };
}

export function sourceKindLabel(source: Source) {
  if (source.sourceType === "telegram") {
    return telegramKind(source) ?? "telegram";
  }
  if (source.sourceType === "youtube") {
    return source.sourceSubtype === "playlist" ? "YouTube playlist" : "YouTube video";
  }
  if (source.sourceType === "rss") {
    return "RSS feed";
  }
  if (source.sourceType === "forum") {
    return source.sourceSubtype === "thread" ? "forum thread" : "forum";
  }
  return source.sourceType;
}

export function membershipLabel(source: Source) {
  if (!sourceCapabilities(source).hasMembershipState) {
    return "";
  }

  const kind = telegramKind(source);
  if (kind === "channel") {
    return source.isMember ? "subscribed" : "not subscribed";
  }
  return source.isMember ? "member" : "not a member";
}
