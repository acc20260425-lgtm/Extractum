import type {
  GeminiBrowserManualAction,
  GeminiBrowserProviderStatusKind,
} from "$lib/types/gemini-browser";

export function statusLabel(
  status: GeminiBrowserProviderStatusKind,
  manualAction: GeminiBrowserManualAction | null,
) {
  if (status === "ready") return "Ready";
  if (status === "needs_login") return "Login required";
  if (status === "needs_manual_action" && manualAction === "account_picker") {
    return "Choose account";
  }
  if (status === "needs_manual_action") return "Manual action";
  if (status === "running") return "Running";
  if (status === "failed") return "Failed";
  if (status === "stopped") return "Stopped";
  return "Not started";
}
