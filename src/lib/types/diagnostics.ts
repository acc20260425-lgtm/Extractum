export interface DiagnosticSummaryDto {
  app: DiagnosticAppInfo;
  database: DiagnosticDatabaseInfo;
  providers: DiagnosticProvidersInfo;
  runtimes: DiagnosticRuntimeInfo;
  telegram: DiagnosticTelegramInfo;
  sources: DiagnosticSourcesInfo;
  items: DiagnosticItemsInfo;
  analysisRuns: DiagnosticAnalysisRunsInfo;
  llmRequests: DiagnosticLlmRequestsInfo;
  youtubeJobs: DiagnosticYoutubeJobsInfo;
  ingest: DiagnosticIngestInfo;
  privacy: DiagnosticPrivacyInfo;
}

export interface DiagnosticAppInfo {
  appName: string;
  appVersion: string;
  buildMode: string;
  generatedAtUnix: number;
}

export interface DiagnosticDatabaseInfo {
  sqliteAvailable: boolean;
  migrations: DiagnosticMigrationInfo;
  accountCount: number;
}

export interface DiagnosticMigrationInfo {
  status: string;
  expectedVersions: number[];
  appliedVersions: number[];
  pendingVersions: number[];
  failedVersions: number[];
}

export interface DiagnosticProvidersInfo {
  activeProvider: string | null;
  profilesByProvider: DiagnosticProviderProfileCount[];
}

export interface DiagnosticProviderProfileCount {
  provider: string;
  configuredCount: number;
  missingKeyCount: number;
}

export interface DiagnosticRuntimeInfo {
  ytdlp: DiagnosticRuntimeCheck;
  secureStorage: DiagnosticRuntimeCheck;
}

export interface DiagnosticRuntimeCheck {
  status: string;
  available: boolean;
  version: string | null;
  summary: string | null;
}

export interface DiagnosticTelegramInfo {
  accountCount: number;
  runtimeStatuses: DiagnosticStatusCount[];
}

export interface DiagnosticStatusCount {
  status: string;
  count: number;
}

export interface DiagnosticSourcesInfo {
  counts: DiagnosticSourceCount[];
}

export interface DiagnosticSourceCount {
  sourceType: string;
  sourceSubtype: string | null;
  active: boolean;
  syncState: string;
  count: number;
}

export interface DiagnosticItemsInfo {
  counts: DiagnosticItemCount[];
}

export interface DiagnosticItemCount {
  sourceType: string;
  sourceSubtype: string | null;
  itemKind: string;
  contentKind: string;
  hasContent: boolean;
  hasMedia: boolean;
  mediaKind: string | null;
  count: number;
}

export interface DiagnosticAnalysisRunsInfo {
  counts: DiagnosticAnalysisRunCount[];
}

export interface DiagnosticAnalysisRunCount {
  provider: string;
  runType: string;
  scopeType: string;
  status: string;
  snapshotState: string;
  errorKind: string;
  count: number;
}

export interface DiagnosticLlmRequestsInfo {
  counts: DiagnosticLlmRequestCount[];
}

export interface DiagnosticLlmRequestCount {
  provider: string;
  kind: string;
  state: string;
  count: number;
}

export interface DiagnosticYoutubeJobsInfo {
  counts: DiagnosticYoutubeJobCount[];
}

export interface DiagnosticYoutubeJobCount {
  jobType: string;
  status: string;
  warningState: string;
  errorKind: string;
  count: number;
}

export interface DiagnosticIngestInfo {
  batches: DiagnosticIngestBatchCount[];
  warnings: DiagnosticIngestWarningCount[];
}

export interface DiagnosticIngestBatchCount {
  provider: string;
  ingestKind: string;
  status: string;
  completeness: string;
  errorKind: string;
  count: number;
}

export interface DiagnosticIngestWarningCount {
  provider: string;
  ingestKind: string;
  status: string;
  warningCode: string;
  count: number;
}

export interface DiagnosticPrivacyInfo {
  excludedDataClasses: string[];
}
