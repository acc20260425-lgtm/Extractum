import { invoke } from "@tauri-apps/api/core";
import type { DiagnosticSummaryDto } from "$lib/types/diagnostics";

export function loadDiagnosticSummary() {
  return invoke<DiagnosticSummaryDto>("get_diagnostic_summary");
}
