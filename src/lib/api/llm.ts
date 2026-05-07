import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AskLlmStreamInput,
  ListLlmProviderModelsInput,
  LlmProfilesState,
  LlmProviderModel,
  LlmStreamEnvelope,
  LlmStreamEvent,
  SaveLlmProfileInput,
} from "$lib/types/llm";

export const LLM_RESPONSE_EVENT = "llm://response";

export function getLlmProfiles() {
  return invoke<LlmProfilesState>("get_llm_profiles");
}

export function saveLlmProfile(input: SaveLlmProfileInput) {
  return invoke<LlmProfilesState>("save_llm_profile", { ...input });
}

export function listLlmProviderModels(input: ListLlmProviderModelsInput) {
  return invoke<LlmProviderModel[]>("list_llm_provider_models", { ...input });
}

export function askLlmStream(input: AskLlmStreamInput) {
  return invoke<void>("ask_llm_stream", { ...input });
}

export function cancelLlmRequest(requestId: string) {
  return invoke<void>("cancel_llm_request", { requestId });
}

export function listenToLlmResponses(
  handler: (event: Event<LlmStreamEvent>) => void,
): Promise<UnlistenFn> {
  return listen<LlmStreamEvent>(
    LLM_RESPONSE_EVENT,
    (event: LlmStreamEnvelope<LlmStreamEvent> & Event<LlmStreamEvent>) => handler(event),
  );
}
