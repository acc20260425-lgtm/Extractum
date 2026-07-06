export type SourceKeyboardCommand =
  | { handled: false }
  | { handled: true; kind: "activate"; sourceId: string }
  | { handled: true; kind: "inspect"; sourceId: string }
  | {
      handled: true;
      kind: "toggleSelection";
      sourceId: string;
      selectedSourceIds: string[];
    }
  | { handled: true; kind: "escape" };

export function sourceGridRowIdsFromElement(host: Element | null): string[] {
  if (!host) return [];
  return Array.from(host.querySelectorAll<HTMLElement>(".wx-row[data-id]"))
    .map((row) => row.dataset.id ?? "")
    .map((id) => (id.startsWith(":") ? id.slice(1) : id))
    .filter(Boolean);
}

export function isSourceKeyboardEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable || target.closest("[contenteditable='true']")) return true;
  return ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
}

export function sourceKeyboardCommand({
  key,
  orderedSourceIds,
  activeSourceId,
  selectedSourceIds,
}: {
  key: string;
  orderedSourceIds: string[];
  activeSourceId: string | null;
  selectedSourceIds: string[];
}): SourceKeyboardCommand {
  if (key === "Escape") return { handled: true, kind: "escape" };
  if (orderedSourceIds.length === 0) return { handled: false };

  const activeIndex = activeSourceId ? orderedSourceIds.indexOf(activeSourceId) : -1;
  if (key === "ArrowDown") {
    const nextIndex = activeIndex < 0 ? 0 : Math.min(activeIndex + 1, orderedSourceIds.length - 1);
    return { handled: true, kind: "activate", sourceId: orderedSourceIds[nextIndex] };
  }
  if (key === "ArrowUp") {
    const nextIndex = activeIndex < 0 ? orderedSourceIds.length - 1 : Math.max(activeIndex - 1, 0);
    return { handled: true, kind: "activate", sourceId: orderedSourceIds[nextIndex] };
  }
  if (!activeSourceId || !orderedSourceIds.includes(activeSourceId)) return { handled: false };
  if (key === "Enter") return { handled: true, kind: "inspect", sourceId: activeSourceId };
  if (key === " " || key === "Spacebar") {
    const selected = new Set(selectedSourceIds);
    if (selected.has(activeSourceId)) {
      selected.delete(activeSourceId);
    } else {
      selected.add(activeSourceId);
    }
    return {
      handled: true,
      kind: "toggleSelection",
      sourceId: activeSourceId,
      selectedSourceIds: Array.from(selected),
    };
  }
  return { handled: false };
}
