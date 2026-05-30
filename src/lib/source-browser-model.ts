import type { Source } from "$lib/types/sources";

export type SourceBrowserTabId =
  | "timeline"
  | "transcript"
  | "comments"
  | "items"
  | "metadata"
  | "activity";

export interface SourceBrowserTab {
  id: SourceBrowserTabId;
  label: string;
}

const TAB_LABELS: Record<SourceBrowserTabId, string> = {
  timeline: "Timeline",
  transcript: "Transcript",
  comments: "Comments",
  items: "Items",
  metadata: "Metadata",
  activity: "Activity",
};

export function sourceBrowserTabsForSource(source: Pick<Source, "sourceType" | "sourceSubtype">): SourceBrowserTab[] {
  const ids: SourceBrowserTabId[] =
    source.sourceType === "youtube" && source.sourceSubtype === "video"
      ? ["transcript", "comments", "items", "metadata", "activity"]
      : source.sourceType === "telegram"
        ? ["timeline", "items", "metadata", "activity"]
        : ["items", "metadata", "activity"];

  return ids.map((id) => ({ id, label: TAB_LABELS[id] }));
}

export function sourceBrowserShellAppliesToSource(source: Pick<Source, "sourceType" | "sourceSubtype">): boolean {
  return source.sourceType === "telegram"
    || (source.sourceType === "youtube" && source.sourceSubtype === "video");
}

export function smartDefaultSourceBrowserTab(source: Pick<Source, "sourceType" | "sourceSubtype">): SourceBrowserTabId {
  if (source.sourceType === "youtube" && source.sourceSubtype === "video") return "transcript";
  if (source.sourceType === "telegram") return "timeline";
  return "items";
}

export function reconcileSourceBrowserTab(
  activeTab: SourceBrowserTabId | null,
  source: Pick<Source, "sourceType" | "sourceSubtype">,
): SourceBrowserTabId {
  const tabs = sourceBrowserTabsForSource(source);
  return activeTab && tabs.some((tab) => tab.id === activeTab)
    ? activeTab
    : smartDefaultSourceBrowserTab(source);
}
