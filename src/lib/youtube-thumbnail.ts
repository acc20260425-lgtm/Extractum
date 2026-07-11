import { invoke } from "@tauri-apps/api/core";

const CACHE_LIMIT = 128;
const cache = new Map<string, string | null>();

type ThumbnailResult = {
  kind: "success" | "terminal_error" | "transient_error";
  dataUrl?: string;
};

function remember(url: string, value: string | null) {
  cache.delete(url);
  cache.set(url, value);
  if (cache.size > CACHE_LIMIT) cache.delete(cache.keys().next().value!);
  return value;
}

export async function resolveYoutubeThumbnail(url: string): Promise<string | null> {
  if (cache.has(url)) return cache.get(url)!;

  try {
    const result = await invoke<ThumbnailResult>("resolve_youtube_thumbnail", { url });
    if (result.kind === "success") return remember(url, result.dataUrl ?? null);
    if (result.kind === "terminal_error") return remember(url, null);
  } catch {
    // Network and IPC failures are intentionally retried on the next mount.
  }
  return null;
}

export function resetYoutubeThumbnailCache() {
  cache.clear();
}
