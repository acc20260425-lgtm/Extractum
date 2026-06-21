import type { GeminiBrowserRun, GeminiBrowserRunResult } from "./types/gemini-browser";

export function selectedRunForInspector(
  runs: GeminiBrowserRun[],
  activeRunId: string | null,
): GeminiBrowserRun | null {
  if (activeRunId) {
    const active = runs.find((run) => run.run_id === activeRunId);
    if (active) return active;
  }
  return runs[0] ?? null;
}

export function artifactAvailability(result: GeminiBrowserRunResult | null) {
  return {
    run_dir: Boolean(result?.artifacts.run_dir),
    html: Boolean(result?.artifacts.html),
    screenshot: Boolean(result?.artifacts.screenshot),
    telemetry: Boolean(result?.artifacts.telemetry),
    answer_extraction: Boolean(result?.artifacts.answer_extraction),
    artifact_write_error: Boolean(result?.artifacts.artifact_write_error),
  };
}

export function isPartialRiskBrowserResult(result: GeminiBrowserRunResult | null): boolean {
  return result?.status === "ok" && result.debug_summary?.answer_completion_reason === "timeout_latest";
}

export function resultTextLength(result: GeminiBrowserRunResult | null): number {
  return result?.text?.length ?? 0;
}

export function debugFinalTextLength(result: GeminiBrowserRunResult | null): number {
  return result?.debug_summary?.final_text_length ?? 0;
}

const MAX_DIAGNOSTIC_MESSAGE_LENGTH = 300;

export function sanitizeDiagnosticMessage(message: string | null | undefined): string {
  if (!message) return "none";
  const sanitized = message
    .replace(/file:\/\/\/[^\s]+/gi, "[path]")
    .replace(/https?:\/\/[^\s]+/gi, (rawUrl) => {
      try {
        const url = new URL(rawUrl);
        const suffix = url.search || url.hash ? "?[redacted]" : "";
        return `${url.origin}${url.pathname}${suffix}`;
      } catch {
        return "[url]";
      }
    })
    .replace(/(^|[^A-Za-z0-9])[A-Za-z]:[\\/][^\s]+/g, "$1[path]")
    .replace(/\\\\[^\s\\]+\\[^\s]+/g, "[path]")
    .replace(/\/Users\/[^\s]+/g, "[path]")
    .replace(/\/home\/[^\s]+/g, "[path]")
    .replace(/%(?:APPDATA|LOCALAPPDATA)%[\\/][^\s]+/gi, "[path]")
    .replace(/[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/gi, "[account]");
  if (sanitized.length <= MAX_DIAGNOSTIC_MESSAGE_LENGTH) return sanitized;
  return `${sanitized.slice(0, MAX_DIAGNOSTIC_MESSAGE_LENGTH)}...[truncated]`;
}

export function copyableRunDiagnostics(run: GeminiBrowserRun): string {
  const result = run.result;
  const availability = artifactAvailability(result);
  const lines = [
    "Gemini Browser Run Diagnostics",
    `run_id: ${run.run_id}`,
    `source: ${run.source}`,
    `status: ${run.status}`,
    `result_status: ${result?.status ?? "unavailable"}`,
    `created_at: ${run.created_at}`,
    `updated_at: ${run.updated_at}`,
    `elapsed_ms: ${result?.elapsed_ms ?? "unavailable"}`,
    `result_text_length: ${resultTextLength(result)}`,
    `debug_final_text_length: ${debugFinalTextLength(result)}`,
    `message: ${sanitizeDiagnosticMessage(result?.message)}`,
    `manual_action: ${result?.manual_action ?? "none"}`,
    `artifact_run_dir_available: ${availability.run_dir}`,
    `artifact_html_available: ${availability.html}`,
    `artifact_screenshot_available: ${availability.screenshot}`,
    `artifact_telemetry_available: ${availability.telemetry}`,
    `answer_extraction_artifact_available: ${availability.answer_extraction}`,
    `partial_risk: ${isPartialRiskBrowserResult(result)}`,
    `artifact_write_error: ${result?.artifacts.artifact_write_error ? "present" : "none"}`,
  ];

  if (!result?.debug_summary) {
    lines.push("debug_summary: unavailable");
    return lines.join("\n");
  }

  const debug = result.debug_summary;
  lines.push(
    `debug_mode: ${debug.mode}`,
    `composer_found: ${debug.composer_found}`,
    `send_button_found: ${debug.send_button_found}`,
    `generation_busy_observed: ${debug.generation_busy_observed}`,
    `answer_found: ${debug.answer_found}`,
    `answer_selector: ${debug.answer_selector ?? "none"}`,
    `answer_completion_reason: ${debug.answer_completion_reason}`,
    `waited_for_send_ms: ${debug.waited_for_send_ms}`,
    `waited_for_answer_ms: ${debug.waited_for_answer_ms}`,
    `answer_stable_ms: ${debug.answer_stable_ms}`,
    `final_text_length: ${debug.final_text_length}`,
    `error_stage: ${debug.error_stage ?? "none"}`,
  );

  if (debug.extraction) {
    lines.push(
      `extraction_raw_candidate_count: ${debug.extraction.raw_candidate_count}`,
      `extraction_grouped_candidate_count: ${debug.extraction.grouped_candidate_count}`,
      `extraction_selected_candidate_length: ${debug.extraction.selected_candidate_length}`,
      `extraction_returned_text_length: ${debug.extraction.returned_text_length}`,
      `extraction_selected_grouping: ${debug.extraction.selected_grouping}`,
      `extraction_selected_candidate_rank: ${debug.extraction.selected_candidate_rank ?? "none"}`,
      `extraction_largest_candidate_length: ${debug.extraction.largest_candidate_length}`,
      `extraction_larger_valid_candidate_available: ${debug.extraction.larger_valid_candidate_available}`,
      `extraction_larger_rejected_candidate_count: ${debug.extraction.larger_rejected_candidate_count}`,
      `extraction_larger_rejected_reasons: ${debug.extraction.larger_rejected_reasons.join(",") || "none"}`,
      `extraction_busy_visible_at_completion: ${debug.extraction.busy_visible_at_completion}`,
      `extraction_candidate_signature_changed_count: ${debug.extraction.candidate_signature_changed_count}`,
      `extraction_stable_poll_count_after_last_candidate_change: ${debug.extraction.stable_poll_count_after_last_candidate_change}`,
    );
  }

  return lines.join("\n");
}
