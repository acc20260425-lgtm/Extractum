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

  it("routes mount, commands, and run-change events through the shared refresh scheduler", () => {
    expect(componentSource).toContain("createGeminiBrowserRefreshScheduler");
    expect(componentSource).toContain("const refreshScheduler");
    expect(componentSource).toContain("function scheduleRefresh()");
    expect(componentSource).toContain("function scheduleRefreshInBackground()");
    expect(componentSource).toContain("void scheduleRefresh().catch(reportUnexpectedRefreshError);");
    expect(componentSource).toContain("await scheduleRefresh();");
    expect(componentSource).toMatch(/onclick=\{\s*scheduleRefreshInBackground\s*\}/);
    expect(componentSource).toContain("listenToGeminiBrowserRunChanges");
    expect(componentSource).not.toContain("listenToGeminiBrowserRuns");
    expect(componentSource).not.toContain("onclick={refresh}");
    expect(componentSource).not.toContain("payload.");
    expect(componentSource).not.toContain("payload.message");
    expect(componentSource).not.toContain("payload.status");
    expect(componentSource).not.toContain("payload.run_updated_at");
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
});
