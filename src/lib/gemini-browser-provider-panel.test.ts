import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { statusLabel } from "./gemini-browser-provider-panel-contract";

const componentSource = readFileSync(
  path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "components/settings/gemini-browser-provider-panel.svelte",
  ),
  "utf8",
);

describe("gemini browser provider panel copy contract", () => {
  it("maps provider statuses to compact operator labels", () => {
    expect(statusLabel("ready", null)).toBe("Ready");
    expect(statusLabel("needs_login", "login")).toBe("Login required");
    expect(statusLabel("needs_manual_action", "account_picker")).toBe("Choose account");
    expect(statusLabel("needs_manual_action", "start_chrome_cdp")).toBe("Start Chrome");
    expect(statusLabel("running", null)).toBe("Running");
    expect(statusLabel("failed", null)).toBe("Failed");
    expect(statusLabel("not_started", null)).toBe("Not started");
  });

  it("routes mount, commands, and polling through the shared refresh scheduler", () => {
    expect(componentSource).toContain("createGeminiBrowserRefreshScheduler");
    expect(componentSource).toContain("createGeminiBrowserPollingController");
    expect(componentSource).toContain("const refreshScheduler");
    expect(componentSource).toContain("geminiBridgeStatusSnapshot");
    expect(componentSource).toContain('scheduleRefresh({ mode: "light" })');
    expect(componentSource).toContain('scheduleRefresh({ mode: "full" })');
    expect(componentSource).not.toContain("listenToGeminiBrowserRunChanges");
    expect(componentSource).not.toContain("listenToGeminiBrowserRuns");
    expect(componentSource).not.toContain("@tauri-apps/api/event");
    expect(componentSource).not.toContain("onclick={refresh}");
    expect(componentSource).not.toContain("payload.");
  });

  it("does not assign authoritative panel state from command return values", () => {
    expect(componentSource).not.toContain("status = await geminiBridgeOpenBrowser");
    expect(componentSource).not.toContain("status = await geminiBridgeResume");
    expect(componentSource).not.toContain("result = await geminiBridgeSendSingle");
    expect(componentSource).not.toMatch(/\bstatus\s*=\s*(opened|resumed)\b/);
    expect(componentSource).not.toMatch(/\bresult\s*=\s*completed\b/);
  });

  it("exposes env-free CDP attach controls in Settings", () => {
    expect(componentSource).toContain("browserProviderMode");
    expect(componentSource).toContain("Attach Chrome");
    expect(componentSource).toContain("Start Chrome");
    expect(componentSource).toContain("CDP endpoint");
    expect(componentSource).toContain("localStorage");
  });

  it("passes browser config to status, open, resume, and send calls", () => {
    expect(componentSource).toContain("loadStatus: () => geminiBridgeStatus(browserConfig())");
    expect(componentSource).toContain("loadStatusSnapshot: () => geminiBridgeStatusSnapshot()");
    expect(componentSource).toContain("loadRun: (runId) => geminiBridgeGetRun(runId)");
    expect(componentSource).toContain("geminiBridgeOpenBrowser(browserConfig())");
    expect(componentSource).toContain("geminiBridgeStartCdpChrome(browserConfig())");
    expect(componentSource).toContain("geminiBridgeResume(browserConfig())");
    expect(componentSource).toContain("browserConfig: browserConfig()");
  });

  it("recovers slow prompt results from the refreshed run log", () => {
    expect(componentSource).toContain("runResultForActivePrompt");
    expect(componentSource).toContain("let activeTestRunId");
    expect(componentSource).toContain("activeTestRunId = runId;");
    expect(componentSource).toContain("syncActivePromptResult(nextRuns)");
    expect(componentSource).not.toContain("syncActivePromptResult(log.runs)");
  });

  it("renders inline run inspector controls and sanitized diagnostics actions", () => {
    expect(componentSource).toContain("Run inspector");
    expect(componentSource).toContain("selectRunForHistory");
    expect(componentSource).toContain("copyableRunDiagnostics");
    expect(componentSource).toContain("sanitizeDiagnosticMessage");
    expect(componentSource).toContain("Copy diagnostics");
    expect(componentSource).toContain("Open run folder");
    expect(componentSource).toContain("geminiBridgeOpenRunFolder");
  });

  it("shows debug summary fields without reading artifact files in the panel", () => {
    expect(componentSource).toContain("generation_busy_observed");
    expect(componentSource).toContain("answer_selector");
    expect(componentSource).toContain("answer_completion_reason");
    expect(componentSource).toContain("Partial risk");
    expect(componentSource).toContain("Answer extraction");
    expect(componentSource).toContain("raw_candidate_count");
    expect(componentSource).toContain("grouped_candidate_count");
    expect(componentSource).toContain("isPartialRiskBrowserResult");
    expect(componentSource).toContain("resultTextLength");
    expect(componentSource).toContain("debugFinalTextLength");
    expect(componentSource).toContain("waited_for_send_ms");
    expect(componentSource).toContain("waited_for_answer_ms");
    expect(componentSource).not.toContain("page.html");
    expect(componentSource).not.toContain("page.png");
  });

  it("renders run history filters and selectable rows for the inline inspector", () => {
    expect(componentSource).toContain("Run history");
    expect(componentSource).toContain("runHistoryFilter");
    expect(componentSource).toContain("filterRunHistoryRows");
    expect(componentSource).toContain("selectRunForHistory");
    expect(componentSource).toContain('data-filter="all"');
    expect(componentSource).toContain('data-filter="problems"');
    expect(componentSource).toContain('data-filter="partial_risk"');
    expect(componentSource).toContain('data-filter="manual_action"');
    expect(componentSource).toContain('data-filter="failed"');
    expect(componentSource).toContain("selectHistoryRun(row.run.run_id)");
    expect(componentSource).toContain("class:selected={selectedInspectorRun?.run_id === row.run.run_id}");
    expect(componentSource).toContain("row.badge");
    expect(componentSource).toContain("row.answerCompletionReason");
    expect(componentSource).not.toContain("{run.prompt_preview}</p>");
  });

  it("renders the actionable setup checklist before test prompt", () => {
    expect(componentSource).toContain("Setup checklist");
    expect(componentSource).toContain("deriveGeminiBrowserSetupChecks");
    expect(componentSource).toContain("setupChecks");
    expect(componentSource).toContain("handleSetupCheckAction(check)");
    expect(componentSource).toContain("setupCheckStateLabel(check.state)");
    expect(componentSource).toContain("setupCheckActionLabel(check.action)");
    expect(componentSource).toContain("Sidecar");
    expect(componentSource).toContain("Mode");
    expect(componentSource).toContain("Chrome CDP");
    expect(componentSource).toContain("Gemini tab");
    expect(componentSource).toContain("Gemini readiness");
    expect(componentSource).toContain("Last test run");
    expect(componentSource).toContain('class="setup-checklist"');
    expect(componentSource).toContain("focusCdpEndpoint");
    expect(componentSource).toContain("selectHistoryRun(check.runId)");
    expect(componentSource).toContain("sendTestPrompt");
    expect(componentSource).not.toContain("{check.run?.result?.text}");
    expect(componentSource).not.toContain("{check.run?.result?.artifacts.run_dir}");
  });

  it("starts active polling before awaiting the terminal test prompt result", () => {
    const sendIndex = componentSource.indexOf("const sendPromise = geminiBridgeSendSingle");
    const pendingIndex = componentSource.indexOf("pollingController.setLocalPendingRun(runId)");
    const refreshIndex = componentSource.indexOf('await scheduleRefresh({ mode: "light" })');
    const awaitIndex = componentSource.indexOf("const completed = await sendPromise");

    expect(sendIndex).toBeGreaterThan(-1);
    expect(pendingIndex).toBeGreaterThan(-1);
    expect(refreshIndex).toBeGreaterThan(-1);
    expect(awaitIndex).toBeGreaterThan(-1);
    expect(pendingIndex).toBeLessThan(awaitIndex);
    expect(refreshIndex).toBeLessThan(awaitIndex);
  });

  it("uses light post-terminal refresh for test prompt completion", () => {
    const awaitIndex = componentSource.indexOf("const completed = await sendPromise");
    const finalRefreshIndex = componentSource.indexOf("await ensurePostTerminalRefresh(runId)", awaitIndex);
    const finalFullIndex = componentSource.indexOf(
      'await scheduleRefresh({ mode: "full", forceTrailing: true })',
      awaitIndex,
    );

    expect(finalRefreshIndex).toBeGreaterThan(awaitIndex);
    expect(finalFullIndex).toBe(-1);
  });

  it("creates polling controller synchronously before scheduler and send actions", () => {
    const pollingIndex = componentSource.indexOf(
      "const pollingController = createGeminiBrowserPollingController",
    );
    const schedulerIndex = componentSource.indexOf(
      "const refreshScheduler = createGeminiBrowserRefreshScheduler",
    );
    const sendIndex = componentSource.indexOf("async function sendTestPrompt");

    expect(pollingIndex).toBeGreaterThan(-1);
    expect(schedulerIndex).toBeGreaterThan(pollingIndex);
    expect(sendIndex).toBeGreaterThan(schedulerIndex);
    expect(componentSource).not.toContain("pollingController?.setLocalPendingRun");
  });

  it("routes selected run detail through scheduler token guard", () => {
    expect(componentSource).toContain("selectedDetailRequestToken");
    expect(componentSource).toContain("applySelectedRunFromScheduler");
    expect(componentSource).toContain("getSelectedDetailToken: () => selectedDetailRequestToken");
    expect(componentSource).not.toContain("latestSelectedRunUpdatedAt");
  });

  it("uses activity snapshots instead of raw active-work booleans", () => {
    expect(componentSource).toContain("getPollingActivitySnapshot");
    expect(componentSource).toContain("runLogSignals");
    expect(componentSource).toContain("statusSignal");
    expect(componentSource).not.toContain("hasActiveGeminiBrowserWork");
    expect(componentSource).not.toContain("hasActiveWork:");
  });

  it("discovers prompt-pack Gemini Browser runs through idle polling list_runs", () => {
    expect(componentSource).toContain("pollingController.start()");
    expect(componentSource).toContain("loadRuns: () => geminiBridgeListRuns()");
    expect(componentSource).toContain("applyRuns: (nextRuns) =>");
    expect(componentSource).toContain("runs = nextRuns");
    expect(componentSource).not.toContain("listenToGeminiBrowserRunChanges");
  });

  it("does not route polling through the background refresh wrapper", () => {
    const controllerIndex = componentSource.indexOf(
      "const pollingController = createGeminiBrowserPollingController",
    );
    const controllerBlock = componentSource.slice(
      controllerIndex,
      componentSource.indexOf("const refreshScheduler"),
    );

    expect(controllerBlock).toContain("scheduleRefresh,");
    expect(controllerBlock).not.toContain("scheduleRefreshInBackground");
  });

  it("records initial mount refresh outcome for polling degradation", () => {
    expect(componentSource).toContain(
      'scheduleRefreshInBackground({ mode: "light" }, { recordPollingOutcome: true })',
    );
  });

  it("keeps rejected pending test runs until terminal, not-found confirmation, or grace expiry", () => {
    const sendPromptIndex = componentSource.indexOf("async function sendTestPrompt()");
    const sendPromptSource = componentSource.slice(
      sendPromptIndex,
      componentSource.indexOf("async function resumeProvider()", sendPromptIndex),
    );
    const completedIndex = sendPromptSource.indexOf("const completed = await sendPromise");
    const successRefreshIndex = sendPromptSource.indexOf(
      "await ensurePostTerminalRefresh(runId)",
      completedIndex,
    );
    const catchIndex = sendPromptSource.indexOf("} catch (error) {");
    const rejectedIndex = sendPromptSource.indexOf(
      "pollingController.markLocalPendingRunRejected(runId)",
      catchIndex,
    );
    const finalRefreshIndex = sendPromptSource.indexOf(
      "await ensurePostTerminalRefresh(runId)",
      catchIndex,
    );
    const unavailableIndex = componentSource.indexOf("applySelectedRunUnavailable");
    const notFoundIndex = componentSource.indexOf(
      "pollingController.confirmPendingRunNotFound(runId)",
      unavailableIndex,
    );

    expect(sendPromptIndex).toBeGreaterThan(-1);
    expect(completedIndex).toBeGreaterThan(-1);
    expect(successRefreshIndex).toBeGreaterThan(completedIndex);
    expect(catchIndex).toBeGreaterThan(-1);
    expect(rejectedIndex).toBeGreaterThan(catchIndex);
    expect(finalRefreshIndex).toBeGreaterThan(rejectedIndex);
    expect(notFoundIndex).toBeGreaterThan(unavailableIndex);
    expect(componentSource).toContain("pollingController.hasLocalPendingRun(runId)");
    expect(componentSource).toContain("pollingController.confirmPendingRunTerminal");
  });
});
