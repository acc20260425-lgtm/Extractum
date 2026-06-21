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

  it("treats Resume as an open-or-reattach command that returns provider status", () => {
    expect(componentSource).toContain("status = await geminiBridgeResume(browserConfig());");
  });

  it("exposes env-free CDP attach controls in Settings", () => {
    expect(componentSource).toContain("browserProviderMode");
    expect(componentSource).toContain("Attach Chrome");
    expect(componentSource).toContain("Start Chrome");
    expect(componentSource).toContain("CDP endpoint");
    expect(componentSource).toContain("localStorage");
  });

  it("passes browser config to status, open, resume, and send calls", () => {
    expect(componentSource).toContain("geminiBridgeStatus(browserConfig())");
    expect(componentSource).toContain("geminiBridgeOpenBrowser(browserConfig())");
    expect(componentSource).toContain("geminiBridgeStartCdpChrome(browserConfig())");
    expect(componentSource).toContain("geminiBridgeResume(browserConfig())");
    expect(componentSource).toContain("browserConfig: browserConfig()");
  });

  it("recovers slow prompt results from the refreshed run log", () => {
    expect(componentSource).toContain("runResultForActivePrompt");
    expect(componentSource).toContain("let activeTestRunId");
    expect(componentSource).toContain("activeTestRunId = runId;");
    expect(componentSource).toContain("syncActivePromptResult(log.runs)");
  });

  it("renders inline run inspector controls and sanitized diagnostics actions", () => {
    expect(componentSource).toContain("Run inspector");
    expect(componentSource).toContain("selectedRunForInspector");
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
    expect(componentSource).toContain("resultTextLength");
    expect(componentSource).toContain("debugFinalTextLength");
    expect(componentSource).toContain("waited_for_send_ms");
    expect(componentSource).toContain("waited_for_answer_ms");
    expect(componentSource).not.toContain("page.html");
    expect(componentSource).not.toContain("page.png");
  });
});
