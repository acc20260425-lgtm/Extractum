export type ApalisJsonValue =
  | null
  | boolean
  | number
  | string
  | ApalisJsonValue[]
  | { [key: string]: ApalisJsonValue };

export interface ApalisJobsListRequest {
  limit?: number | null;
  status?: string | null;
  jobType?: string | null;
  search?: string | null;
}

export interface ApalisJobsListResponse {
  jobs: ApalisJobRow[];
  totalMatching: number;
  statusCounts: ApalisJobStatusCount[];
  jobTypeCounts: ApalisJobTypeCount[];
  refreshedAt: string;
  limit: number;
}

export interface ApalisJobsPruneTerminalRequest {
  olderThanHours?: number | null;
}

export interface ApalisJobsPruneTerminalResponse {
  deletedCount: number;
  cutoffAt: string;
  olderThanHours: number;
}

export interface ApalisJobRow {
  id: string;
  jobType: string;
  status: string;
  attempts: number;
  maxAttempts: number | null;
  runAt: string | null;
  lockAt: string | null;
  lockBy: string | null;
  doneAt: string | null;
  lastActivityAt: string | null;
  priority: number | null;
  idempotencyKey: string | null;
  jobPreview: string | null;
  jobTruncated: boolean;
  jobJson: ApalisJsonValue | null;
  lastResult: ApalisJsonValue | null;
  lastResultTruncated: boolean;
  metadata: ApalisJsonValue | null;
  metadataTruncated: boolean;
}

export interface ApalisJobStatusCount {
  status: string;
  count: number;
}

export interface ApalisJobTypeCount {
  jobType: string;
  count: number;
}
