export type AccountRuntimeState =
  | "not_initialized"
  | "restoring"
  | "ready"
  | "reauth_required"
  | "restore_failed";

export interface AccountRecord {
  id: number;
  label: string;
  api_id: number;
  api_hash: string;
  phone: string | null;
  created_at: number;
}

export interface AccountRuntimeStatus {
  account_id: number;
  status: AccountRuntimeState;
  message: string | null;
}
