# Gemini Browser Setup Status UX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an actionable Setup checklist to the Browser Providers settings panel so operators can see what is ready, what needs attention, and which safe action to take next.

**Architecture:** Implement this first slice as frontend-derived state only. Add a focused pure helper module that maps existing provider status, mode, endpoint text, selected/latest run, and load errors into setup rows; then render those rows in the existing Svelte panel using existing actions (`Refresh`, `Start Chrome`, `Open`, `Resume`, `Send test`, `View run`). Keep Rust, sidecar protocol, run logging, prompt-pack runtime, answer extraction, retry/cancel, and artifact contents unchanged.

**Tech Stack:** Svelte 5 runes, TypeScript, Vitest source/helper tests, existing Gemini Browser Provider DTOs and Settings panel.

---

## Scope And File Map

Create:

- `src/lib/gemini-browser-setup-status.ts`
  - Owns setup-check types, endpoint pre-validation, last-run classification, state/action labels, and pure `deriveGeminiBrowserSetupChecks(...)`.

- `src/lib/gemini-browser-setup-status.test.ts`
  - Covers managed/attach mode setup rows, endpoint guidance, manual-action mapping, run classification, old DTO tolerance, and privacy boundaries.

Modify:

- `src/lib/components/settings/gemini-browser-provider-panel.svelte`
  - Imports the new helper, derives `setupChecks`, renders a `Setup checklist` section between provider controls and test prompt, and routes row actions to existing component functions.

- `src/lib/gemini-browser-provider-panel.test.ts`
  - Adds source-contract coverage for the checklist labels, helper usage, action wiring, and privacy-sensitive non-rendering.

- `docs/browser-providers-llm-troubleshooting.md`
  - Documents the new setup checklist as the first diagnostic surface before Run Inspector and Run History.

Do not modify:

- `src-tauri/src/gemini_browser/**`
- `sidecars/gemini-browser/**`
- `src-tauri/src/prompt_packs/**`
- run log schemas
- answer extraction contracts

Execution setup:

- Create a feature branch before implementation:

```powershell
git switch -c gemini-browser-setup-status-ux
```

- After each task: mark completed checkboxes in this plan, run the task verification command, then commit.

---

### Task 1: Pure Setup Status Helper

**Files:**

- Create: `src/lib/gemini-browser-setup-status.ts`
- Create: `src/lib/gemini-browser-setup-status.test.ts`

- [x] **Step 1: Add failing setup status helper tests**

Create `src/lib/gemini-browser-setup-status.test.ts`:

```ts
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
```

- [x] **Step 2: Run helper tests and verify they fail**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-setup-status.test.ts
```

Expected: FAIL because `src/lib/gemini-browser-setup-status.ts` does not exist.

- [x] **Step 3: Implement the setup status helper**

Create `src/lib/gemini-browser-setup-status.ts`:

```ts
import {
  isPartialRiskBrowserResult,
  sanitizeDiagnosticMessage,
} from "./gemini-browser-run-inspector";
import type {
  GeminiBrowserProviderMode,
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunStatus,
} from "./types/gemini-browser";

export type GeminiBrowserSetupCheckState =
  | "ready"
  | "action_needed"
  | "running"
  | "warning"
  | "failed"
  | "unknown"
  | "not_applicable";

export type GeminiBrowserSetupCheckAction =
  | "refresh"
  | "start_chrome"
  | "open"
  | "resume"
  | "send_test"
  | "view_run"
  | "focus_endpoint";

export type GeminiBrowserSetupCheckId =
  | "sidecar"
  | "mode"
  | "chrome_cdp"
  | "gemini_tab"
  | "gemini_readiness"
  | "last_test_run";

export interface GeminiBrowserSetupCheck {
  id: GeminiBrowserSetupCheckId;
  label: string;
  state: GeminiBrowserSetupCheckState;
  message: string;
  action: GeminiBrowserSetupCheckAction | null;
  runId?: string | null;
}

export interface GeminiBrowserSetupStatusInput {
  status: GeminiBrowserProviderStatus | null;
  providerMode: GeminiBrowserProviderMode;
  cdpEndpoint: string;
  runs: GeminiBrowserRun[];
  selectedRun: GeminiBrowserRun | null;
  busy: boolean;
  statusLoadError: string | null;
}

const FAILED_RUN_STATUSES = new Set<GeminiBrowserRunStatus>([
  "failed",
  "timeout",
  "browser_crashed",
  "blocked",
]);

const MANUAL_ACTION_RUN_STATUSES = new Set<GeminiBrowserRunStatus>([
  "needs_login",
  "needs_manual_action",
]);

export function deriveGeminiBrowserSetupChecks(
  input: GeminiBrowserSetupStatusInput,
): GeminiBrowserSetupCheck[] {
  const lastRun = input.selectedRun ?? input.runs[0] ?? null;
  return [
    sidecarCheck(input),
    modeCheck(input),
    chromeCdpCheck(input),
    geminiTabCheck(input),
    geminiReadinessCheck(input, lastRun),
    lastTestRunCheck(lastRun),
  ];
}

export function setupCheckStateLabel(state: GeminiBrowserSetupCheckState): string {
  if (state === "ready") return "Ready";
  if (state === "action_needed") return "Action needed";
  if (state === "running") return "Running";
  if (state === "warning") return "Warning";
  if (state === "failed") return "Failed";
  if (state === "not_applicable") return "Not applicable";
  return "Unknown";
}

export function setupCheckActionLabel(action: GeminiBrowserSetupCheckAction): string {
  if (action === "refresh") return "Refresh";
  if (action === "start_chrome") return "Start Chrome";
  if (action === "open") return "Open";
  if (action === "resume") return "Resume";
  if (action === "send_test") return "Send test";
  if (action === "view_run") return "View run";
  return "Edit endpoint";
}

function sidecarCheck(input: GeminiBrowserSetupStatusInput): GeminiBrowserSetupCheck {
  if (input.statusLoadError) {
    return {
      id: "sidecar",
      label: "Sidecar",
      state: "failed",
      message: `Status failed: ${sanitizeDiagnosticMessage(input.statusLoadError)}`,
      action: "refresh",
    };
  }
  if (!input.status) {
    return {
      id: "sidecar",
      label: "Sidecar",
      state: "unknown",
      message: "Refresh provider status to check the sidecar.",
      action: "refresh",
    };
  }
  return {
    id: "sidecar",
    label: "Sidecar",
    state: "ready",
    message: "Sidecar status responded.",
    action: "refresh",
  };
}

function modeCheck(input: GeminiBrowserSetupStatusInput): GeminiBrowserSetupCheck {
  if (input.providerMode === "managed") {
    return {
      id: "mode",
      label: "Mode",
      state: "ready",
      message: "Managed browser profile is selected.",
      action: null,
    };
  }
  if (!isLocalHttpCdpEndpoint(input.cdpEndpoint)) {
    return {
      id: "mode",
      label: "Mode",
      state: "action_needed",
      message: "Attach Chrome requires a local HTTP endpoint such as http://127.0.0.1:9222.",
      action: "focus_endpoint",
    };
  }
  return {
    id: "mode",
    label: "Mode",
    state: "ready",
    message: "Attach Chrome is selected with a local CDP endpoint.",
    action: null,
  };
}

function chromeCdpCheck(input: GeminiBrowserSetupStatusInput): GeminiBrowserSetupCheck {
  if (input.providerMode === "managed") {
    return {
      id: "chrome_cdp",
      label: "Chrome CDP",
      state: "not_applicable",
      message: "Managed mode does not use Chrome CDP attach.",
      action: null,
    };
  }
  if (!input.status) {
    return {
      id: "chrome_cdp",
      label: "Chrome CDP",
      state: "unknown",
      message: "Refresh status to check Chrome CDP.",
      action: "refresh",
    };
  }
  if (input.status.status === "ready") {
    return {
      id: "chrome_cdp",
      label: "Chrome CDP",
      state: "ready",
      message: "Chrome CDP is attached.",
      action: null,
    };
  }
  if (input.status.manual_action === "start_chrome_cdp") {
    return {
      id: "chrome_cdp",
      label: "Chrome CDP",
      state: "action_needed",
      message: sanitizeDiagnosticMessage(input.status.latest_message) || "Start Chrome with remote debugging enabled.",
      action: chromeActionFromMessage(input.status.latest_message),
    };
  }
  return {
    id: "chrome_cdp",
    label: "Chrome CDP",
    state: "unknown",
    message: "Resume the provider to attach to Chrome CDP.",
    action: "resume",
  };
}

function geminiTabCheck(input: GeminiBrowserSetupStatusInput): GeminiBrowserSetupCheck {
  if (!input.status) {
    return {
      id: "gemini_tab",
      label: "Gemini tab",
      state: "unknown",
      message: "Refresh status to check the Gemini tab.",
      action: "refresh",
    };
  }
  if (input.status.status === "ready") {
    return {
      id: "gemini_tab",
      label: "Gemini tab",
      state: "ready",
      message: "A usable Gemini page is available.",
      action: null,
    };
  }
  if (input.providerMode === "managed") {
    return {
      id: "gemini_tab",
      label: "Gemini tab",
      state: "unknown",
      message: "Open the managed browser to load Gemini.",
      action: "open",
    };
  }
  if (input.status.manual_action === "start_chrome_cdp") {
    return {
      id: "gemini_tab",
      label: "Gemini tab",
      state: "action_needed",
      message: sanitizeDiagnosticMessage(input.status.latest_message) || "Start or attach Chrome, then open Gemini.",
      action: chromeActionFromMessage(input.status.latest_message),
    };
  }
  return {
    id: "gemini_tab",
    label: "Gemini tab",
    state: "unknown",
    message: "Open Gemini in the attached browser or resume the provider.",
    action: "open",
  };
}

function geminiReadinessCheck(
  input: GeminiBrowserSetupStatusInput,
  lastRun: GeminiBrowserRun | null,
): GeminiBrowserSetupCheck {
  if (input.status?.status === "needs_login") {
    return {
      id: "gemini_readiness",
      label: "Gemini readiness",
      state: "action_needed",
      message: "Gemini needs login or another browser-side manual step.",
      action: input.providerMode === "cdp_attach" ? "resume" : "open",
    };
  }
  if (lastRun?.result?.manual_action) {
    return {
      id: "gemini_readiness",
      label: "Gemini readiness",
      state: "action_needed",
      message: `Manual action required: ${lastRun.result.manual_action}.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  const debug = lastRun?.result?.debug_summary;
  if (debug?.composer_found && debug.send_button_found) {
    return {
      id: "gemini_readiness",
      label: "Gemini readiness",
      state: "ready",
      message: "Composer and send button were found in the latest inspected run.",
      action: "send_test",
    };
  }
  if (input.status?.status === "ready") {
    return {
      id: "gemini_readiness",
      label: "Gemini readiness",
      state: "warning",
      message: "Send a test prompt to confirm Gemini is ready.",
      action: "send_test",
    };
  }
  return {
    id: "gemini_readiness",
    label: "Gemini readiness",
    state: "unknown",
    message: "Browser setup must be ready before testing Gemini.",
    action: input.providerMode === "cdp_attach" ? "resume" : "open",
  };
}

function lastTestRunCheck(lastRun: GeminiBrowserRun | null): GeminiBrowserSetupCheck {
  if (!lastRun) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "unknown",
      message: "No Browser Provider test run is loaded yet.",
      action: null,
      runId: null,
    };
  }
  if (lastRun.status === "running" || lastRun.status === "queued") {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "running",
      message: `Run ${lastRun.run_id} is ${lastRun.status}.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  const result = lastRun.result;
  if (!result?.debug_summary) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: result ? "unknown" : "running",
      message: result ? `Run ${lastRun.run_id} has no debug summary.` : `Run ${lastRun.run_id} is pending.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  if (result.status === "ok" && result.debug_summary.answer_completion_reason === "stable") {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "ready",
      message: `Run ${lastRun.run_id} completed with a stable answer.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  if (isPartialRiskBrowserResult(result)) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "warning",
      message: `Run ${lastRun.run_id} is partial-risk (timeout_latest).`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  if (MANUAL_ACTION_RUN_STATUSES.has(result.status) || result.manual_action) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "action_needed",
      message: `Run ${lastRun.run_id} needs manual action.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  if (FAILED_RUN_STATUSES.has(result.status)) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "failed",
      message: `Run ${lastRun.run_id} ended with ${result.status}.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  return {
    id: "last_test_run",
    label: "Last test run",
    state: "unknown",
    message: `Run ${lastRun.run_id} needs inspection.`,
    action: "view_run",
    runId: lastRun.run_id,
  };
}

function chromeActionFromMessage(message: string | null): GeminiBrowserSetupCheckAction {
  const normalized = message?.toLowerCase() ?? "";
  if (normalized.includes("attached") || normalized.includes("open gemini")) return "open";
  if (normalized.includes("configured but not attached")) return "resume";
  return "start_chrome";
}

function isLocalHttpCdpEndpoint(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed) return false;
  try {
    const url = new URL(trimmed);
    const host = url.hostname;
    return (
      url.protocol === "http:" &&
      (host === "127.0.0.1" || host === "localhost" || host === "[::1]" || host === "::1") &&
      Boolean(url.port) &&
      url.pathname === "/" &&
      !url.search &&
      !url.hash &&
      !url.username &&
      !url.password
    );
  } catch {
    return false;
  }
}
```

- [x] **Step 4: Run helper tests and verify they pass**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-setup-status.test.ts
```

Expected: PASS.

- [x] **Step 5: Mark Task 1 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-06-21-gemini-browser-setup-status-ux-plan.md src/lib/gemini-browser-setup-status.ts src/lib/gemini-browser-setup-status.test.ts
git commit -m "feat: derive Gemini browser setup checks"
```

---

### Task 2: Settings Panel Setup Checklist UI

**Files:**

- Modify: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`

- [ ] **Step 1: Add failing source-contract tests for the checklist UI**

Append this test inside `describe("gemini browser provider panel copy contract", () => { ... })` in `src/lib/gemini-browser-provider-panel.test.ts`:

```ts
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
```

- [ ] **Step 2: Run panel source tests and verify they fail**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-provider-panel.test.ts
```

Expected: FAIL because the component does not yet import or render setup checks.

- [ ] **Step 3: Import setup helper and derive checklist rows**

Modify imports in `src/lib/components/settings/gemini-browser-provider-panel.svelte`:

```ts
  import {
    deriveGeminiBrowserSetupChecks,
    setupCheckActionLabel,
    setupCheckStateLabel,
    type GeminiBrowserSetupCheck,
  } from "$lib/gemini-browser-setup-status";
```

Add status-load error state after `let message = $state("");`:

```ts
  let statusLoadError = $state<string | null>(null);
```

Add a derived checklist after `const selectedPartialRisk = $derived(...)`:

```ts
  const setupChecks = $derived(
    deriveGeminiBrowserSetupChecks({
      status,
      providerMode: browserProviderMode,
      cdpEndpoint,
      runs,
      selectedRun: selectedInspectorRun,
      busy,
      statusLoadError,
    }),
  );
```

Modify `refresh()` so success clears `statusLoadError` and failure stores it:

```ts
  async function refresh() {
    try {
      const [nextStatus, log] = await Promise.all([
        geminiBridgeStatus(browserConfig()),
        geminiBridgeListRuns(8),
      ]);
      statusLoadError = null;
      status = nextStatus;
      runs = log.runs;
      message = nextStatus.latest_message ?? "";
      syncActivePromptResult(log.runs);
    } catch (error) {
      const formatted = formatAppError("loading Gemini browser provider", error);
      statusLoadError = formatted;
      message = formatted;
    }
  }
```

- [ ] **Step 4: Add checklist action helpers**

Add these functions near the existing UI helper functions in `src/lib/components/settings/gemini-browser-provider-panel.svelte`:

```ts
  function focusCdpEndpoint() {
    if (typeof document === "undefined") return;
    document.getElementById("gemini-browser-cdp-endpoint")?.focus();
  }

  async function handleSetupCheckAction(check: GeminiBrowserSetupCheck) {
    if (!check.action) return;
    if (check.action === "refresh") {
      await refresh();
      return;
    }
    if (check.action === "start_chrome") {
      await startCdpChrome();
      return;
    }
    if (check.action === "open") {
      await openBrowser();
      return;
    }
    if (check.action === "resume") {
      await resumeProvider();
      return;
    }
    if (check.action === "send_test") {
      await sendTestPrompt();
      return;
    }
    if (check.action === "view_run" && check.runId) {
      selectHistoryRun(check.runId);
      return;
    }
    if (check.action === "focus_endpoint") {
      focusCdpEndpoint();
    }
  }
```

- [ ] **Step 5: Render the Setup checklist between provider controls and test prompt**

In `src/lib/components/settings/gemini-browser-provider-panel.svelte`, after the closing `</div>` of the first provider card and before the test prompt provider card, insert:

```svelte
    <section class="setup-checklist" aria-label="Setup checklist">
      <div class="row setup-head">
        <div>
          <h3>Setup checklist</h3>
          <p>Safe next steps for making the Browser Provider usable.</p>
        </div>
        <button type="button" onclick={refresh} disabled={busy} title="Refresh setup checklist">
          <RefreshCw size={14} />
          <span>Refresh</span>
        </button>
      </div>

      <div class="setup-grid">
        {#each setupChecks as check (check.id)}
          <div class="setup-row" class:warning={check.state === "warning" || check.state === "action_needed"} class:failed={check.state === "failed"}>
            <div>
              <span class="fact-label">{check.label}</span>
              <strong>{setupCheckStateLabel(check.state)}</strong>
              <p>{check.message}</p>
            </div>
            {#if check.action}
              <button type="button" onclick={() => handleSetupCheckAction(check)} disabled={busy}>
                <span>{setupCheckActionLabel(check.action)}</span>
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </section>
```

Keep the existing `.provider-grid` wrapper. The inserted
`<section class="setup-checklist">` must be a child of `.provider-grid`
between the first provider card and the test prompt provider card, so the CSS
rule `grid-column: 1 / -1` makes it span both columns.

- [ ] **Step 6: Add checklist styles**

In the component `<style>` block, replace:

```css
  .provider-card button,
  .run-inspector button,
  .history-filters button {
```

with:

```css
  .provider-card button,
  .setup-checklist button,
  .run-inspector button,
  .history-filters button {
```

Add these styles near `.provider-card`:

```css
  .setup-checklist {
    grid-column: 1 / -1;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    background: var(--card);
  }

  .setup-head {
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 12px;
  }

  .setup-head h3 {
    margin: 0;
    font-size: 16px;
  }

  .setup-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .setup-row {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    min-width: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px;
    background: var(--background);
  }

  .setup-row > div {
    min-width: 0;
  }

  .setup-row p {
    margin: 4px 0 0;
    color: var(--muted-foreground);
    font-size: 12px;
    overflow-wrap: anywhere;
  }

  .setup-row.failed {
    border-color: color-mix(in srgb, var(--destructive) 70%, var(--border));
  }
```

In the existing media query, include `.setup-grid` and `.setup-row`:

```css
  @media (max-width: 820px) {
    .provider-grid,
    .setup-grid,
    .inspector-grid,
    .inspector-grid.compact,
    .run-row {
      grid-template-columns: 1fr;
    }

    .setup-row {
      flex-direction: column;
    }
  }
```

- [ ] **Step 7: Run UI tests and Svelte check**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-setup-status.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
```

Expected:

- Vitest PASS for both files.
- `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 8: Mark Task 2 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-06-21-gemini-browser-setup-status-ux-plan.md src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/gemini-browser-provider-panel.test.ts
git commit -m "feat: show Gemini browser setup checklist"
```

---

### Task 3: Documentation, Verification, And Manual Validation

**Files:**

- Modify: `docs/browser-providers-llm-troubleshooting.md`
- Modify: `docs/superpowers/plans/2026-06-21-gemini-browser-setup-status-ux-plan.md`

- [ ] **Step 1: Document the setup checklist workflow**

In `docs/browser-providers-llm-troubleshooting.md`, find `## Inline Run Inspector` and insert this new section immediately before it:

```md
## Setup Checklist

The Browser Providers panel shows `Setup checklist` between provider controls
and the test prompt. Use it before opening run folders or reading artifacts.

Rows:

- `Sidecar`: confirms that provider status can be loaded from the sidecar.
- `Mode`: confirms managed mode or a local-looking CDP endpoint in attach mode.
- `Chrome CDP`: shows whether attach mode needs `Start Chrome`, `Resume`, or no
  action.
- `Gemini tab`: shows whether a usable Gemini page is available or whether the
  operator should open/resume the browser.
- `Gemini readiness`: uses provider status and latest run debug facts to decide
  whether login/manual action is likely, or whether a test prompt should be
  sent.
- `Last test run`: classifies the selected/latest run as stable, partial-risk,
  manual-action, failed, running, or unknown.

Checklist actions reuse existing safe controls. They do not implement retry,
cancel, prompt-pack routing, login automation, artifact reading, or remote CDP.
If a row points to `View run`, inspect the selected run through Run Inspector
and Run History.
```

- [ ] **Step 2: Run final automated verification**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-setup-status.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
git diff --check
```

Expected:

- Vitest PASS for both test files.
- `svelte-check found 0 errors and 0 warnings`.
- `git diff --check` exits 0.

- [ ] **Step 3: Manual validation in running app**

Start the app if it is not already running:

```powershell
npm.cmd run tauri dev
```

Validate manually:

1. Open Settings -> Browser Providers.
2. Confirm `Setup checklist` appears between provider controls and test prompt.
3. In managed mode with no browser opened, confirm checklist points to `Open`.
4. Switch to Attach Chrome with no Chrome attached, confirm checklist points to
   `Start Chrome` or `Resume` according to the current status message.
5. Click `Start Chrome`; confirm the checklist updates after refresh.
6. Click `Resume`; confirm attach-ready states update when Chrome/Gemini is
   available.
7. With a ready browser and no fresh run, confirm `Gemini readiness` points to
   `Send test`.
8. Send the one-sentence test prompt and confirm `Last test run` becomes
   `Ready` for a stable result, `Warning` for `timeout_latest`, `Action needed`
   for manual-action, or `Failed` for failed/timeout/browser-crashed results.
9. Click `View run` and confirm Run Inspector selects that run.
10. Confirm checklist rows do not show full prompt text, answer text, artifact
    paths, raw URLs, account identifiers, screenshots, or DOM.

If live Gemini validation is blocked by Google login or verification, record the
observed manual-action state and do not change code in this task.

- [ ] **Step 4: Record manual validation outcome in this plan**

Append a `## Manual Validation Result` section near the end of this plan:

```md
## Manual Validation Result

- Date:
- Mode(s):
- Setup checklist visible:
- Managed-mode result:
- Attach-mode result:
- Test run id:
- Last test run classification:
- Notes:
```

Fill the fields with the observed values.

- [ ] **Step 5: Mark Task 3 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/browser-providers-llm-troubleshooting.md docs/superpowers/plans/2026-06-21-gemini-browser-setup-status-ux-plan.md
git commit -m "docs: document Gemini browser setup checklist"
```

---

## Final Checklist

- [ ] `src/lib/gemini-browser-setup-status.ts` exists and contains pure setup-check derivation.
- [ ] Helper tests cover managed mode, attach mode, endpoint guidance, manual actions, run classifications, old DTOs, action labels, and privacy boundaries.
- [ ] Browser Providers settings renders `Setup checklist` between provider controls and test prompt.
- [ ] Checklist rows include `Sidecar`, `Mode`, `Chrome CDP`, `Gemini tab`, `Gemini readiness`, and `Last test run`.
- [ ] Checklist actions reuse existing component actions and do not add retry/cancel semantics.
- [ ] Run Inspector and Run History behavior remains intact.
- [ ] `npm.cmd run test -- --run src/lib/gemini-browser-setup-status.test.ts src/lib/gemini-browser-provider-panel.test.ts` passes.
- [ ] `npm.cmd run check` passes.
- [ ] `git diff --check` passes.
- [ ] Manual validation outcome is recorded.
