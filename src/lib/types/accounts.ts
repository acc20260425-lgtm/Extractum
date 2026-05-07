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
  phone: string | null;
  created_at: number;
}

export interface CreateAccountInput {
  label: string;
  apiId: number;
  apiHash: string;
}

export interface AccountIdInput {
  accountId: number;
}

export interface AccountPhoneInput extends AccountIdInput {
  phone: string;
}

export interface AccountCodeInput extends AccountIdInput {
  code: string;
}

export interface AccountRuntimeStatus {
  account_id: number;
  status: AccountRuntimeState;
  message: string | null;
}
