import { describe, expect, it } from "vitest";
import {
  deriveGeminiBrowserSetupChecks,
  setupCheckActionLabel,
  setupCheckStateLabel,
  type GeminiBrowserSetupCheck,
} from "./gemini-browser-setup-status";
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunResult,
} from "./types/gemini-browser";

function providerStatus(
  overrides: Partial<GeminiBrowserProviderStatus> = {},
): GeminiBrowserProviderStatus {
  return {
    status: "not_started",
    manual_action: null,
    active_run_id: null,
    queue_depth: 0,
    browser_profile_dir: "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/profile",
    latest_message: "Browser has not been opened.",
    ...overrides,
  };
}

function result(overrides: Partial<GeminiBrowserRunResult> = {}): GeminiBrowserRunResult {
  return {
    run_id: "run-1",
    status: "ok",
    text: "full answer text",
    message: "Finished near C:/Users/Dima/private/path and dima@example.com",
    manual_action: null,
    artifacts: {
      run_dir: "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1",
      html: null,
      screenshot: null,
      telemetry: null,
      answer_extraction: null,
      artifact_write_error: null,
    },
    elapsed_ms: 10_000,
    debug_summary: {
      mode: "cdp_attach",
      composer_found: true,
      send_button_found: true,
      generation_busy_observed: false,
      answer_found: true,
      answer_selector: "message-content",
      waited_for_send_ms: 500,
      waited_for_answer_ms: 8_000,
      answer_stable_ms: 8_000,
      answer_completion_reason: "stable",
      final_text_length: 16,
      error_stage: null,
      extraction: null,
    },
    ...overrides,
  };
}

function run(overrides: Partial<GeminiBrowserRun> = {}): GeminiBrowserRun {
  return {
    run_id: "run-1",
    source: "settings_test",
    status: "ok",
    prompt_preview: "Reply with one short sentence",
    created_at: "2026-06-21T00:00:00Z",
    updated_at: "2026-06-21T00:00:20Z",
    result: result(),
    ...overrides,
  };
}

function checkById(checks: GeminiBrowserSetupCheck[], id: GeminiBrowserSetupCheck["id"]) {
  const check = checks.find((candidate) => candidate.id === id);
  if (!check) throw new Error(`Missing setup check ${id}`);
  return check;
}

describe("gemini browser setup status", () => {
  it("derives managed-mode first-run guidance without requiring CDP", () => {
    const checks = deriveGeminiBrowserSetupChecks({
      status: providerStatus({ status: "not_started", latest_message: "Browser has not been opened." }),
      providerMode: "managed",
      cdpEndpoint: "http://127.0.0.1:9222",
      runs: [],
      selectedRun: null,
      busy: false,
      statusLoadError: null,
    });

    expect(checkById(checks, "sidecar")).toMatchObject({
      state: "ready",
      action: "refresh",
    });
    expect(checkById(checks, "mode")).toMatchObject({
      state: "ready",
      action: null,
    });
    expect(checkById(checks, "chrome_cdp")).toMatchObject({
      state: "not_applicable",
      action: null,
    });
    expect(checkById(checks, "gemini_tab")).toMatchObject({
      state: "unknown",
      action: "open",
    });
    expect(checkById(checks, "gemini_readiness")).toMatchObject({
      state: "unknown",
      action: "open",
    });
    expect(checkById(checks, "last_test_run")).toMatchObject({
      state: "unknown",
      action: null,
    });
  });

  it("maps attach-mode CDP setup action to Start Chrome guidance", () => {
    const checks = deriveGeminiBrowserSetupChecks({
      status: providerStatus({
        status: "needs_manual_action",
        manual_action: "start_chrome_cdp",
        latest_message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
      }),
      providerMode: "cdp_attach",
      cdpEndpoint: "http://127.0.0.1:9222",
      runs: [],
      selectedRun: null,
      busy: false,
      statusLoadError: null,
    });

    expect(checkById(checks, "mode")).toMatchObject({ state: "ready", action: null });
    expect(checkById(checks, "chrome_cdp")).toMatchObject({
      state: "action_needed",
      action: "start_chrome",
    });
    expect(checkById(checks, "gemini_tab")).toMatchObject({
      state: "action_needed",
      action: "start_chrome",
    });
  });

  it("flags invalid-looking attach endpoint without weakening backend validation", () => {
    const checks = deriveGeminiBrowserSetupChecks({
      status: providerStatus({ status: "not_started" }),
      providerMode: "cdp_attach",
      cdpEndpoint: "http://192.168.1.20:9222",
      runs: [],
      selectedRun: null,
      busy: false,
      statusLoadError: null,
    });

    expect(checkById(checks, "mode")).toMatchObject({
      state: "action_needed",
      action: "focus_endpoint",
    });
  });

  it("marks ready attach state and asks for a test prompt when no run exists", () => {
    const checks = deriveGeminiBrowserSetupChecks({
      status: providerStatus({
        status: "ready",
        latest_message: "Chrome CDP attached.",
      }),
      providerMode: "cdp_attach",
      cdpEndpoint: "http://localhost:9222",
      runs: [],
      selectedRun: null,
      busy: false,
      statusLoadError: null,
    });

    expect(checkById(checks, "chrome_cdp")).toMatchObject({ state: "ready", action: null });
    expect(checkById(checks, "gemini_tab")).toMatchObject({ state: "ready", action: null });
    expect(checkById(checks, "gemini_readiness")).toMatchObject({
      state: "warning",
      action: "send_test",
    });
  });

  it("classifies stable, partial-risk, manual-action, failed, running, and old runs", () => {
    const stable = run({ run_id: "stable", result: result({ run_id: "stable" }) });
    const partial = run({
      run_id: "partial",
      result: result({
        run_id: "partial",
        debug_summary: {
          ...result().debug_summary!,
          answer_completion_reason: "timeout_latest",
        },
      }),
    });
    const manual = run({
      run_id: "manual",
      status: "needs_manual_action",
      result: result({
        run_id: "manual",
        status: "needs_manual_action",
        manual_action: "login",
        debug_summary: null,
      }),
    });
    const failed = run({
      run_id: "failed",
      status: "failed",
      result: result({ run_id: "failed", status: "failed", debug_summary: null }),
    });
    const running = run({ run_id: "running", status: "running", result: null });
    const old = run({
      run_id: "old",
      status: "ok",
      result: result({ run_id: "old", debug_summary: null, text: null }),
    });

    for (const [candidate, expectedState] of [
      [stable, "ready"],
      [partial, "warning"],
      [manual, "action_needed"],
      [failed, "failed"],
      [running, "running"],
      [old, "unknown"],
    ] as const) {
      const checks = deriveGeminiBrowserSetupChecks({
        status: providerStatus({ status: "ready" }),
        providerMode: "managed",
        cdpEndpoint: "http://127.0.0.1:9222",
        runs: [candidate],
        selectedRun: candidate,
        busy: false,
        statusLoadError: null,
      });

      expect(checkById(checks, "last_test_run")).toMatchObject({
        state: expectedState,
        action: "view_run",
        runId: candidate.run_id,
      });
    }
  });

  it("uses status load errors without exposing sensitive run text or artifact paths", () => {
    const checks = deriveGeminiBrowserSetupChecks({
      status: null,
      providerMode: "managed",
      cdpEndpoint: "http://127.0.0.1:9222",
      runs: [run()],
      selectedRun: run(),
      busy: false,
      statusLoadError:
        "Failed near C:/Users/Dima/AppData/Roaming/org.ai.extractum and dima@example.com",
    });
    const joined = checks.map((check) => check.message).join("\n");

    expect(checkById(checks, "sidecar")).toMatchObject({
      state: "failed",
      action: "refresh",
    });
    expect(joined).not.toContain("C:/Users/Dima");
    expect(joined).not.toContain("dima@example.com");
    expect(joined).not.toContain("full answer text");
    expect(joined).not.toContain("runs/run-1");
  });

  it("provides display labels for states and actions", () => {
    expect(setupCheckStateLabel("action_needed")).toBe("Action needed");
    expect(setupCheckStateLabel("not_applicable")).toBe("Not applicable");
    expect(setupCheckActionLabel("start_chrome")).toBe("Start Chrome");
    expect(setupCheckActionLabel("send_test")).toBe("Send test");
    expect(setupCheckActionLabel("focus_endpoint")).toBe("Edit endpoint");
  });
});
