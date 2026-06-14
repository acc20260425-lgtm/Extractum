import { invoke } from "@tauri-apps/api/core";
import type { LibraryCatalogResponse, LibrarySourceRecord } from "$lib/types/library-sources";

export function listLibrarySources() {
  return invoke<LibrarySourceRecord[]>("list_library_sources");
}

export function listLibraryCatalog() {
  return invoke<LibraryCatalogResponse>("list_library_catalog");
}
