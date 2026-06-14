import { invoke } from "@tauri-apps/api/core";
import type { PromptPackLibrary } from "$lib/types/prompt-packs";

export function getPromptPackLibrary() {
  return invoke<PromptPackLibrary>("get_prompt_pack_library");
}
