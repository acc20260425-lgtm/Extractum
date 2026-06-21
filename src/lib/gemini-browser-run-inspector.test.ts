import { describe, expect, it } from "vitest";
import {
  artifactAvailability,
  copyableRunDiagnostics,
  debugFinalTextLength,
  filterRunHistoryRows,
  isPartialRiskBrowserResult,
  resultTextLength,
  runHistoryRow,
  selectRunForHistory,
  selectedRunForInspector,
  type GeminiBrowserRunHistoryFilter,
} from "./gemini-browser-run-inspector";
import type { GeminiBrowserRun, GeminiBrowserRunResult } from "./types/gemini-browser";

function result(overrides: Partial<GeminiBrowserRunResult> = {}): GeminiBrowserRunResult {
  return {
    run_id: "run-1",
    status: "ok",
    text: "answer text",
    message:
      "Failed near C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1/page.html, file:///C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1/page.html, /Users/dima/Extractum/private.txt, /home/dima/.config/extractum/private.txt, \\\\server\\share\\secret.txt, %APPDATA%\\Extractum\\secret.txt, %LOCALAPPDATA%\\Extractum\\secret.txt, https://gemini.google.com/app?authuser=dima@example.com&hl=ru#private, and dima@example.com " +
      "x".repeat(2_000),
    manual_action: null,
    artifacts: {
      run_dir: "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1",
      html: null,
      screenshot: null,
      telemetry:
        "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1/telemetry.json",
      answer_extraction:
        "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1/answer-extraction.json",
      artifact_write_error: null,
    },
    elapsed_ms: 16_309,
    debug_summary: {
      mode: "cdp_attach",
      composer_found: true,
      send_button_found: true,
      generation_busy_observed: true,
      answer_found: true,
      answer_selector: "message-content",
      waited_for_send_ms: 15_000,
      waited_for_answer_ms: 10_000,
      answer_stable_ms: 8_000,
      answer_completion_reason: "stable",
      final_text_length: 11,
      error_stage: null,
      extraction: {
        raw_candidate_count: 3,
        grouped_candidate_count: 1,
        selected_candidate_length: 95,
        returned_text_length: 95,
        selected_grouping: "assistant_turn",
        selected_candidate_rank: 1,
        selected_score: 120,
        largest_candidate_length: 95,
        larger_valid_candidate_available: false,
        larger_rejected_candidate_count: 1,
        larger_rejected_reasons: ["composer"],
        top_candidate_lengths: [95, 14],
        busy_visible_at_completion: false,
        last_growth_elapsed_ms: 8_000,
        candidate_signature_changed_count: 2,
        stable_poll_count_after_last_candidate_change: 3,
      },
    },
    ...overrides,
  };
}

function run(overrides: Partial<GeminiBrowserRun> = {}): GeminiBrowserRun {
  return {
    run_id: "run-1",
    source: "settings_test",
    status: "ok",
    prompt_preview: "prompt preview",
    created_at: "2026-06-21T00:00:00Z",
    updated_at: "2026-06-21T00:00:20Z",
    result: result(),
    ...overrides,
  };
}

describe("gemini browser run inspector", () => {
  it("selects the active run before falling back to the newest run", () => {
    const newest = run({ run_id: "newest", result: result({ run_id: "newest" }) });
    const active = run({ run_id: "active", result: result({ run_id: "active" }) });

    expect(selectedRunForInspector([newest, active], "active")?.run_id).toBe("active");
    expect(selectedRunForInspector([newest, active], null)?.run_id).toBe("newest");
    expect(selectedRunForInspector([], null)).toBeNull();
  });

  it("reports artifact availability without exposing full paths", () => {
    expect(artifactAvailability(result())).toEqual({
      run_dir: true,
      html: false,
      screenshot: false,
      telemetry: true,
      answer_extraction: true,
      artifact_write_error: false,
    });
  });

  it("copies sanitized diagnostics with debug facts and no local paths", () => {
    const selectedRun = run();
    const diagnostics = copyableRunDiagnostics(selectedRun);

    expect(diagnostics).toContain("run_id: run-1");
    expect(diagnostics).toContain("status: ok");
    expect(diagnostics).toContain("result_status: ok");
    expect(diagnostics).toContain("elapsed_ms: 16309");
    expect(diagnostics).toContain("result_text_length: 11");
    expect(diagnostics).toContain("debug_final_text_length: 11");
    expect(diagnostics).toContain("generation_busy_observed: true");
    expect(diagnostics).toContain("answer_selector: message-content");
    expect(diagnostics).toContain("answer_completion_reason: stable");
    expect(diagnostics).not.toContain(selectedRun.result?.artifacts.run_dir ?? "missing-run-dir");
    expect(diagnostics).not.toContain(selectedRun.result?.artifacts.telemetry ?? "missing-telemetry");
    expect(diagnostics).not.toContain("C:/Users/Dima");
    expect(diagnostics).not.toContain("file:///C:/Users/Dima");
    expect(diagnostics).not.toContain("/Users/dima");
    expect(diagnostics).not.toContain("/home/dima");
    expect(diagnostics).not.toContain("\\\\server\\share");
    expect(diagnostics).not.toContain("%APPDATA%");
    expect(diagnostics).not.toContain("%LOCALAPPDATA%");
    expect(diagnostics).not.toContain("authuser");
    expect(diagnostics).not.toContain("dima@example.com");
    expect(diagnostics).toContain("https://gemini.google.com/app?[redacted]");
    expect(diagnostics).toContain("[truncated]");
    expect(diagnostics).not.toContain("answer text");
  });

  it("copies extraction diagnostics without artifact paths or answer text", () => {
    const diagnostics = copyableRunDiagnostics(run());

    expect(diagnostics).toContain("answer_extraction_artifact_available: true");
    expect(diagnostics).toContain("extraction_raw_candidate_count: 3");
    expect(diagnostics).toContain("extraction_grouped_candidate_count: 1");
    expect(diagnostics).toContain("extraction_selected_grouping: assistant_turn");
    expect(diagnostics).toContain("extraction_larger_valid_candidate_available: false");
    expect(diagnostics).not.toContain("answer-extraction.json");
    expect(diagnostics).not.toContain("answer text");
  });

  it("detects timeout_latest as partial risk", () => {
    const partial = result({
      debug_summary: {
        ...result().debug_summary!,
        answer_completion_reason: "timeout_latest",
      },
    });

    expect(isPartialRiskBrowserResult(partial)).toBe(true);
    expect(isPartialRiskBrowserResult(result())).toBe(false);
  });

  it("reports result and debug text lengths separately", () => {
    const mismatched = result({
      text: "short",
      debug_summary: { ...result().debug_summary!, final_text_length: 42 },
    });

    expect(resultTextLength(mismatched)).toBe(5);
    expect(debugFinalTextLength(mismatched)).toBe(42);
  });

  it("copies a clear marker when debug summary is unavailable", () => {
    const diagnostics = copyableRunDiagnostics(
      run({ result: result({ debug_summary: null, text: null }) }),
    );

    expect(diagnostics).toContain("debug_summary: unavailable");
  });

  function runWithResult(
    runId: string,
    runStatus: GeminiBrowserRun["status"],
    resultOverrides: Partial<GeminiBrowserRunResult> = {},
  ): GeminiBrowserRun {
    return run({
      run_id: runId,
      status: runStatus,
      prompt_preview: `${runId} prompt`,
      updated_at: `2026-06-21T00:00:${runId.length.toString().padStart(2, "0")}Z`,
      result: result({ run_id: runId, ...resultOverrides }),
    });
  }

  it("derives compact history rows without exposing answer text or artifact paths", () => {
    const row = runHistoryRow(
      runWithResult("stable-run", "ok", {
        elapsed_ms: 24_660,
        text: "full answer text",
        debug_summary: {
          ...result().debug_summary!,
          answer_completion_reason: "stable",
          final_text_length: 16,
        },
      }),
    );

    expect(row.run.run_id).toBe("stable-run");
    expect(row.status).toBe("ok");
    expect(row.badge).toBe("stable");
    expect(row.isProblem).toBe(false);
    expect(row.isPartialRisk).toBe(false);
    expect(row.elapsedMs).toBe(24_660);
    expect(row.resultTextLength).toBe(16);
    expect(row.answerCompletionReason).toBe("stable");
  });

  it("classifies partial risk, manual action, failed, running, queued, and stable rows", () => {
    const partial = runWithResult("partial", "ok", {
      debug_summary: {
        ...result().debug_summary!,
        answer_completion_reason: "timeout_latest",
      },
    });
    const manual = runWithResult("manual", "needs_manual_action", {
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
    });
    const failed = runWithResult("failed", "failed", { status: "failed" });
    const blocked = runWithResult("blocked", "blocked", { status: "blocked" });
    const running = run({ run_id: "running", status: "running", result: null });
    const queued = run({ run_id: "queued", status: "queued", result: null });
    const stable = runWithResult("stable", "ok");

    expect(runHistoryRow(partial).badge).toBe("partial");
    expect(runHistoryRow(partial).isProblem).toBe(true);
    expect(runHistoryRow(manual).badge).toBe("manual");
    expect(runHistoryRow(manual).isProblem).toBe(true);
    expect(runHistoryRow(failed).badge).toBe("failed");
    expect(runHistoryRow(blocked).badge).toBe("failed");
    expect(runHistoryRow(running).badge).toBe("running");
    expect(runHistoryRow(queued).badge).toBe("queued");
    expect(runHistoryRow(stable).badge).toBe("stable");
  });

  it("filters history rows by operator-focused buckets", () => {
    const runs: GeminiBrowserRun[] = [
      runWithResult("stable", "ok"),
      runWithResult("partial", "ok", {
        debug_summary: {
          ...result().debug_summary!,
          answer_completion_reason: "timeout_latest",
        },
      }),
      runWithResult("manual", "needs_manual_action", {
        status: "needs_manual_action",
        manual_action: "login",
      }),
      runWithResult("timeout", "timeout", { status: "timeout" }),
      run({ run_id: "running", status: "running", result: null }),
    ];

    const ids = (filter: GeminiBrowserRunHistoryFilter) =>
      filterRunHistoryRows(runs, filter).map((row) => row.run.run_id);

    expect(ids("all")).toEqual(["stable", "partial", "manual", "timeout", "running"]);
    expect(ids("problems")).toEqual(["partial", "manual", "timeout"]);
    expect(ids("partial_risk")).toEqual(["partial"]);
    expect(ids("manual_action")).toEqual(["manual"]);
    expect(ids("failed")).toEqual(["timeout"]);
  });

  it("preserves selected history run across refresh and falls back within the visible filter", () => {
    const newest = runWithResult("newest", "ok");
    const active = run({ run_id: "active", status: "running", result: null });
    const partial = runWithResult("partial", "ok", {
      debug_summary: {
        ...result().debug_summary!,
        answer_completion_reason: "timeout_latest",
      },
    });
    const failed = runWithResult("failed", "failed", { status: "failed" });
    const runs = [newest, active, partial, failed];

    expect(selectRunForHistory(runs, "active", null, "all")?.run_id).toBe("active");
    expect(selectRunForHistory(runs, "active", "partial", "all")?.run_id).toBe("partial");
    expect(selectRunForHistory(runs, "active", "partial", "partial_risk")?.run_id).toBe(
      "partial",
    );
    expect(selectRunForHistory(runs, "active", "newest", "partial_risk")?.run_id).toBe(
      "partial",
    );
    expect(selectRunForHistory(runs, null, "missing", "failed")?.run_id).toBe("failed");
    expect(selectRunForHistory([], null, "missing", "all")).toBeNull();
  });

  it("tolerates old run records without debug summary", () => {
    const oldRun = run({
      run_id: "old",
      status: "ok",
      result: result({ debug_summary: null, text: null }),
    });

    const row = runHistoryRow(oldRun);

    expect(row.badge).toBe("ok");
    expect(row.answerCompletionReason).toBeNull();
    expect(row.resultTextLength).toBe(0);
  });
});
