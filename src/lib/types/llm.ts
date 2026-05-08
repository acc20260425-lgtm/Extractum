export interface LlmMessage {
  role: string;
  content: string;
}

export interface LlmProfile {
  profile_id: string;
  provider: string;
  default_model: string;
  api_key_configured: boolean;
  base_url: string;
}

export interface SaveLlmProfileInput {
  profileId: LlmProfile["profile_id"];
  provider: LlmProfile["provider"];
  defaultModel: LlmProfile["default_model"];
  apiKey: string | null;
  baseUrl: LlmProfile["base_url"] | null;
  setActive: boolean;
}

export interface LlmProfilesState {
  active_profile: string;
  profiles: LlmProfile[];
}

export interface LlmProviderModel {
  model: string;
  name: string;
  display_name: string;
  description: string;
  input_token_limit: number | null;
  output_token_limit: number | null;
  supported_generation_methods: string[];
}

export interface ListLlmProviderModelsInput {
  provider: string;
  profileId?: string | null;
  apiKey?: string | null;
  baseUrl?: string | null;
}

export interface AskLlmStreamInput {
  requestId: string;
  profileId: string | null;
  messages: LlmMessage[];
  modelOverride: string | null;
}

export interface LlmUsage {
  input_tokens: number | null;
  output_tokens: number | null;
  total_tokens: number | null;
}

export type LlmStreamEventKind =
  | "queued"
  | "started"
  | "delta"
  | "completed"
  | "failed"
  | "cancelled";

export interface LlmStreamEvent {
  request_id: string;
  kind: LlmStreamEventKind;
  queue_position: number | null;
  delta: string | null;
  text: string | null;
  provider: string;
  model: string;
  usage: LlmUsage | null;
  error: string | null;
}

export interface LlmStreamEnvelope<T> {
  payload: T;
}
