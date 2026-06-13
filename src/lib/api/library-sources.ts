import { invoke } from "@tauri-apps/api/core";
import type { LibrarySourceRecord } from "$lib/types/library-sources";

export function listLibrarySources() {
  return invoke<LibrarySourceRecord[]>("list_library_sources");
}
