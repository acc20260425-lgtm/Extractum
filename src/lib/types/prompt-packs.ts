export interface PromptPackLibrary {
  packs: PromptPack[];
}

export interface PromptPack {
  packId: string;
  displayName: string;
  activeVersion: PromptPackVersion | null;
}

export interface PromptPackVersion {
  packVersionId: number;
  packVersion: string;
  schemaVersion: string;
  lifecycleStatus: string;
  defaultControlPreset: string;
  defaultEvidenceMode: string;
  defaultIncludeComments: boolean;
  stages: PromptPackStageTemplate[];
  schemaAssets: PromptPackSchemaAsset[];
}

export interface PromptPackStageTemplate {
  stageName: string;
  stageOrder: number;
  providerFamily: string;
  inputSchemaId: string;
  outputSchemaId: string;
}

export interface PromptPackSchemaAsset {
  schemaId: string;
  schemaKind: string;
  contentHash: string;
}

export type PromptPackRunStatus =
  | "queued"
  | "running"
  | "complete"
  | "partial"
  | "failed"
  | "cancelled"
  | "interrupted";

export type PromptPackRunEventKind =
  | "queued"
  | "started"
  | "progress"
  | "stage_started"
  | "stage_completed"
  | "stage_failed"
  | "completed"
  | "partial"
  | "failed"
  | "cancelled"
  | "interrupted";

export type PromptPackRunEventPhase =
  | "preflight"
  | "snapshot"
  | "stage"
  | "validation"
  | "projection"
  | "persist"
  | "terminal";

export interface PreflightYoutubeSummaryRunInput {
  projectId: number | null;
  sourceIds: number[];
  profileId: string | null;
  modelOverride: string | null;
  outputLanguage: string;
  controlPreset: string;
  evidenceMode: string;
  includeComments: boolean;
}

export interface StartYoutubeSummaryRunInput {
  clientRequestId: string;
  projectId: number | null;
  sourceIds: number[];
  profileId: string | null;
  modelOverride: string | null;
  outputLanguage: string;
  controlPreset: string;
  evidenceMode: string;
  includeComments: boolean;
}

export type StartYoutubeSummaryRunOutcome =
  | { kind: "started"; run: PromptPackRunSummary }
  | { kind: "blocked"; preflight: YoutubeSummaryPreflightResponse };

export interface ListPromptPackRunsInput {
  projectId?: number | null;
  limit?: number;
}

export interface YoutubeSummaryPreflightResponse {
  packId: string;
  packVersion: string;
  includedVideos: YoutubeSummaryPreflightVideo[];
  skippedVideos: YoutubeSummaryPreflightSkippedVideo[];
  blockingFailures: YoutubeSummaryPreflightFailure[];
  estimatedInputTokens: number;
  selectedModelInputLimit: number | null;
}

export interface YoutubeSummaryPreflightVideo {
  sourceId: number;
  videoId: string;
  title: string;
  estimatedInputTokens: number;
}

export interface YoutubeSummaryPreflightSkippedVideo {
  sourceId?: number | null;
  videoId?: string | null;
  title?: string | null;
  reason: string;
}

export interface YoutubeSummaryPreflightFailure {
  sourceId?: number | null;
  reason: string;
  message?: string | null;
}

export interface PromptPackRunEvent {
  runId: number;
  requestId: string;
  kind: PromptPackRunEventKind;
  runStatus: PromptPackRunStatus;
  phase: PromptPackRunEventPhase;
  stageRunId: number | null;
  stageName: string | null;
  sourceSnapshotId: number | null;
  queuePosition: number | null;
  progressCurrent: number | null;
  progressTotal: number | null;
  message: string | null;
  error: string | null;
}

export interface PromptPackRunSummary {
  runId: number;
  projectId?: number | null;
  packId?: string;
  packVersion?: string;
  runStatus: PromptPackRunStatus;
  resultStatus?: string;
  createdAt?: string;
  startedAt?: string | null;
  completedAt?: string | null;
  latestMessage?: string | null;
  progressCurrent?: number | null;
  progressTotal?: number | null;
  queuePosition?: number | null;
}

export interface PromptPackStageRun {
  stageRunId: number;
  runId: number;
  sourceSnapshotId: number | null;
  stageName: string;
  stageOrder: number;
  stageStatus: string;
  latestMessage: string | null;
}

export interface PromptPackResult {
  runId: number;
  resultStatus: string;
  canonical: Record<string, unknown>;
  storageWarning?: string | null;
}

export interface PromptPackValidationFinding {
  runId: number;
  stageRunId: number | null;
  severity: string;
  code: string;
  message: string;
  objectPath: string | null;
  createdAt: string;
}

export interface PromptPackStageArtifactSummary {
  stageRunId: number;
  artifactKind: string;
  attemptNumber: number;
  artifactIndex: number;
  contentType: string;
  contentHash: string;
  createdAt: string;
}

export interface GetPromptPackStageArtifactInput {
  stageRunId: number;
  artifactKind: string;
  attemptNumber: number;
  artifactIndex: number;
}

export interface PromptPackStageArtifact {
  stageRunId: number;
  artifactKind: string;
  attemptNumber: number;
  artifactIndex: number;
  contentType: string;
  content: unknown;
  createdAt: string;
}

export interface PromptPackAuditEvent {
  runId: number;
  eventKind: string;
  message: string | null;
  payload: unknown | null;
  createdAt: string;
}
