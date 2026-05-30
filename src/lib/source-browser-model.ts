import type { Source, SourceItem } from "$lib/types/sources";

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

export interface SourceItemKindChip {
  kind: string;
  label: string;
  count: number;
}

export interface LoadedSourceItemFilter {
  kind: string | null;
  search: string;
}

export type LoadedSourceItemSort = "newest" | "oldest";

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

export function sourceItemKindChips(items: SourceItem[]): SourceItemKindChip[] {
  const counts = new Map<string, number>();
  for (const item of items) {
    counts.set(item.itemKind, (counts.get(item.itemKind) ?? 0) + 1);
  }
  return Array.from(counts, ([kind, count]) => ({
    kind,
    label: sourceItemKindLabel(kind),
    count,
  }));
}

export function filterLoadedSourceItems(items: SourceItem[], filter: LoadedSourceItemFilter): SourceItem[] {
  const search = filter.search.trim().toLocaleLowerCase();
  return items.filter((item) => {
    if (filter.kind !== null && item.itemKind !== filter.kind) return false;
    if (!search) return true;
    return [item.content, item.author]
      .some((value) => value?.toLocaleLowerCase().includes(search));
  });
}

export function sortLoadedSourceItems(items: SourceItem[], sort: LoadedSourceItemSort): SourceItem[] {
  const direction = sort === "newest" ? -1 : 1;
  return [...items].sort((left, right) => {
    const timestampOrder = (left.publishedAt - right.publishedAt) * direction;
    return timestampOrder || left.id - right.id;
  });
}

function sourceItemKindLabel(kind: string) {
  const [first = "", ...rest] = kind.split("_");
  return [
    first === "youtube" ? "YouTube" : capitalize(first),
    ...rest,
  ].join(" ");
}

function capitalize(value: string) {
  if (!value) return value;
  return value.charAt(0).toLocaleUpperCase() + value.slice(1);
}
