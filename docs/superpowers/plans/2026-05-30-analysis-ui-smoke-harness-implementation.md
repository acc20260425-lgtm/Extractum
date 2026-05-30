# Analysis UI Smoke Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an opt-in `npm.cmd run smoke:analysis` command that launches the real debug Tauri `/analysis` UI, seeds deterministic fixtures, exercises Workspace Parity and Source Browser surfaces, preserves failure artifacts, and always cleans up fixture data and spawned processes.

**Architecture:** Use the existing debug-only Tauri MCP bridge over WebSocket instead of adding Playwright. Keep stable smoke hooks small and user-contract oriented, centralize bridge/DOM/process/artifact behavior in smoke helpers, and keep scenarios as deterministic workflow steps. The default command launches the app; attach mode is deferred for this implementation.

**Tech Stack:** Node 24 built-in `WebSocket`, Tauri debug MCP bridge, Svelte 5/SvelteKit 2, Vitest raw/source contract tests, Rust fixture tests, `npm.cmd run verify`, opt-in manual smoke verification.

---

## Execution Protocol

- Start from branch `analysis-ui-smoke-harness-spec`. If executing from `main`, first bring in the spec commits before starting Task 0.
- Do not implement attach mode in this plan. `--attach` remains deferred.
- After each task, mark completed checkboxes in this plan and commit the task.
- Keep `npm.cmd run verify` free of the smoke command.
- Associated disabled reason contract: the NotebookLM export button must have `aria-describedby={exportReasonId}` when disabled by a source-group reason, and the helper text must use the same `id` plus `data-smoke-id="notebooklm-export-disabled-reason"`.
- Run snapshot Source Browser tab contract is exact order `Sources | Items | Metadata`. `Activity` is not allowed on run snapshots.
- Bridge discovery contract: wait up to 90 seconds for the debug bridge by repeating short scans across ports `9223..9322`; report `app-identifier-mismatch` separately from `bridge-unavailable`.
- Bridge probe contract: before running UI scenarios, probe `get_backend_state`, `resize_window`, `execute_js`, and screenshot command dispatch. Screenshot capture is best-effort after dispatch is proven.
- Probe-only safety contract: `--probe-only` must not seed, verify, clean, or navigate fixture data; fixture cleanup runs only after fixture lifecycle has started.
- UI click contract: mode switches and row actions must be scoped by smoke id or row text. Do not use broad `clickByText("Open")`, `clickByText("Source")`, or `clickByText("Report")` for navigation-critical steps.
- Source/group selection contract: after clicking a switcher result, wait for `data-smoke-id="analysis-current-context"` to contain the selected label; do not treat a label that is still visible inside the switcher as completed navigation.
- Opened-run navigation contract: after clicking a run row action, wait for an opened-run canvas surface such as `.report-viewer` or `.report-run-header`; do not use broad `Run #` text because the runs list may already contain it.
- Fixture DOM presence contract: seed verification checks the known expected fixture labels against body text; do not parse fixture labels from `innerText` with a broad regex.
- Cleanup verification contract: fixture cleanup is verified by debug fixture command summaries, not by stale DOM labels after same-route navigation.
- Cleanup summary contract: empty-cleanup verification requires every known fixture summary key to be present and equal `0`.
- GUI smoke gate contract: `npm.cmd run smoke:analysis` is required only in GUI-capable environments. In non-GUI environments, record the exact reason as `not run in this environment`, continue non-GUI verification, and never claim smoke pass.
- Source hook contract: run snapshot headers are identified by an explicit `smokeId` prop from `ReportSourceSurface`, not by comparing visible title copy.
- Raw-source tests should verify semantic hook ownership with stable tokens or regex-style checks; avoid exact-line assertions for formatted Svelte attributes.
- Fixture cleanup must remain marker-scoped. Any cleanup broad enough to delete non-fixture rows is a blocker.
- Use deterministic smoke step names such as `source-browser.youtube-video-tabs`; use the same names in console output and artifact paths.

## Files

- Create: `scripts/analysis-smoke-helpers.mjs`
  - Own WebSocket bridge requests, bridge response classification, app launch/kill helpers, artifact helpers, fixture summary validation, tab assertion helpers, and DOM helper script execution.
- Create: `scripts/analysis-smoke.mjs`
  - Own the opt-in command, deterministic smoke step list, bridge probe, fixture lifecycle, Source Browser scenarios, Workspace Parity scenarios, teardown, and process exit code.
- Create: `src/lib/analysis-smoke-helpers.test.ts`
  - Unit test pure helper behavior without launching Tauri.
- Create: `src/lib/analysis-ui-smoke-contract.test.ts`
  - Raw/source contract tests for package scripts, smoke selectors, disabled reason association, exact snapshot tabs, and selector containment.
- Modify: `package.json`
  - Add `"smoke:analysis": "node scripts/analysis-smoke.mjs"`.
- Modify: `.gitignore`
  - Ignore `/tmp/analysis-smoke/`.
- Modify: `src/lib/components/ui/Button.svelte`
  - Add an optional `smokeId` prop that renders `data-smoke-id`.
- Modify: `src/lib/components/desktop-dialog.svelte`
  - Add an optional `smokeId` prop that renders `data-smoke-id` on the dialog card.
- Modify: `src/lib/components/analysis/report-workspace-tools.svelte`
  - Add smoke ids to workspace tools, NotebookLM export button, and disabled reason. Keep `aria-describedby` as the technical association.
- Modify: `src/lib/components/analysis/report-canvas.svelte`
  - Add smoke ids to the canvas, report/source mode buttons, template drawer, and group drawer.
- Modify: `src/lib/components/analysis/report-setup-panel.svelte`
  - Add `data-smoke-id="analysis-report-setup"`.
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
  - Add `data-smoke-id="analysis-source-surface"` and pass explicit reader header smoke ids.
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
  - Add `data-smoke-id="source-browser-tabs"` to the tab container.
- Modify: `src/lib/components/analysis/source-reader-header.svelte`
  - Add an optional `smokeId` prop that renders `data-smoke-id` on the header.
- Modify: `src/lib/components/analysis/notebooklm-export-dialog.svelte`
  - Pass `smokeId="notebooklm-export-dialog"` to `DesktopDialog`.
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
  - Add smoke ids for the source switcher trigger and current context button.
- Modify: `src/lib/components/analysis/source-switcher-panel.svelte`
  - Add smoke ids for the panel and search input wrapper.
- Modify: `src/lib/components/analysis/run-companion-tabs.svelte`
  - Add smoke id to the Runs tab button.
- Modify: `src/lib/components/analysis/run-companion-runs-tab.svelte`
  - Add smoke ids for the runs tab surface and search input wrapper.
- Modify: `src-tauri/src/analysis/fixtures.rs`
  - Rename the source-group fixture label to `__analysis_redesign_fixture__ Telegram Source Group`.
  - Add an explicit marker-scoped cleanup test for non-fixture groups and members.
- Modify: `docs/superpowers/specs/2026-05-30-analysis-ui-smoke-harness-design.md`
  - Mark implementation status after verification.
- Modify: `docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md`
  - Track task completion until final removal.
- Modify: `docs/superpowers/plans/README.md`
  - List this active implementation plan while work is in progress.
- Delete: `docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md`
  - Remove the completed implementation plan from active plans after final verification is recorded in tests, docs, verification archive, and Git history.

---

### Task 0: Preflight Audit

**Files:**
- Modify: `docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md`

- [ ] **Step 1: Confirm branch and cleanliness**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## analysis-ui-smoke-harness-spec
```

Stop if there are unrelated uncommitted changes.

- [ ] **Step 2: Confirm bridge command availability in the local crate source**

Run:

```powershell
rg -n "execute_js|resize_window|capture_native_screenshot|get_backend_state|dispatch_command" C:\Users\Dima\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tauri-plugin-mcp-bridge-0.11.0\src
```

Expected facts:

```text
websocket.rs dispatches execute_js
websocket.rs dispatches resize_window
websocket.rs dispatches capture_native_screenshot
websocket.rs proxies plugin:mcp-bridge|get_backend_state through invoke_tauri
```

Stop if the installed crate version is different or one of these commands is missing. Update the plan before implementation if the protocol differs.

- [ ] **Step 3: Confirm current fixture cleanup already uses marker/prefix patterns**

Run:

```powershell
rg -n "FIXTURE_MARKER|FIXTURE_EXTERNAL_PREFIX|clear_analysis_redesign_fixtures_in_pool|DELETE FROM analysis_source_groups|DELETE FROM sources" src-tauri/src/analysis/fixtures.rs
```

Expected facts:

```text
clear_analysis_redesign_fixtures_in_pool exists
marker_pattern = format!("{FIXTURE_MARKER}%") is used
external_pattern = format!("{FIXTURE_EXTERNAL_PREFIX}%") is used
source and source-group cleanup predicates are marker-scoped
```

- [ ] **Step 4: Confirm fixture test insert patterns before writing cleanup tests**

Run:

```powershell
rg -n "fixture_pool|insert_minimal_clear_fixture|INSERT INTO accounts \\(label, api_id, api_hash, created_at\\)|INSERT INTO sources \\(" src-tauri/src/analysis/fixtures.rs
```

Expected facts:

```text
fixture_pool exists
insert_minimal_clear_fixture exists
existing tests insert accounts with the current minimal account schema: label, api_id, api_hash, created_at
existing tests insert sources through current sources columns rather than guessed account/status columns
```

When adding the cleanup-scope test, reuse these patterns or return ids from inserts. Do not hand-write account/source columns that are not already used by the current fixture test helpers.

- [ ] **Step 5: Record actual audit result in this plan**

Add a short note under this step with the exact bridge crate version and fixture cleanup facts observed. Use this format:

```text
Actual audit result:
- Bridge crate: tauri-plugin-mcp-bridge 0.11.0.
- WebSocket dispatcher supports: execute_js, resize_window, capture_native_screenshot.
- Backend probe command: invoke_tauri -> plugin:mcp-bridge|get_backend_state.
- Fixture cleanup uses marker_pattern and external_pattern in clear_analysis_redesign_fixtures_in_pool.
- Fixture tests use current helper insert patterns from fixture_pool / insert_minimal_clear_fixture.
```

- [ ] **Step 6: Commit preflight note**

Run:

```powershell
git add docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
git commit -m "docs: record analysis smoke preflight"
```

Expected: commit succeeds.

---

### Task 1: Failing Contract Tests

**Files:**
- Create: `src/lib/analysis-ui-smoke-contract.test.ts`
- Create: `src/lib/analysis-smoke-helpers.test.ts`
- Modify: `src-tauri/src/analysis/fixtures.rs`

- [ ] **Step 1: Add raw/source UI smoke contract tests**

Create `src/lib/analysis-ui-smoke-contract.test.ts`:

```ts
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import packageJson from "../../package.json";
import smokeScriptSource from "../../scripts/analysis-smoke.mjs?raw";
import helperSource from "../../scripts/analysis-smoke-helpers.mjs?raw";
import verifyScriptSource from "../../scripts/verify.mjs?raw";
import desktopDialogSource from "./components/desktop-dialog.svelte?raw";
import notebookLmExportDialogSource from "./components/analysis/notebooklm-export-dialog.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import reportWorkspaceToolsSource from "./components/analysis/report-workspace-tools.svelte?raw";
import sourceBrowserShellSource from "./components/analysis/source-browser-shell.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import sourceSwitcherPanelSource from "./components/analysis/source-switcher-panel.svelte?raw";
import compactSourceRailSource from "./components/analysis/compact-source-rail.svelte?raw";
import runCompanionTabsSource from "./components/analysis/run-companion-tabs.svelte?raw";
import runCompanionRunsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";
import { sourceBrowserTabsForSubject } from "./source-browser-model";

const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));
const sharedSmokePrimitiveFiles = new Set([
  "src/lib/components/ui/Button.svelte",
  "src/lib/components/desktop-dialog.svelte",
]);

function collectSourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const fullPath = path.join(dir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) return collectSourceFiles(fullPath);
    return fullPath;
  });
}

describe("analysis UI smoke harness contract", () => {
  it("exposes the smoke command as opt-in and keeps verify free of it", () => {
    expect(packageJson.scripts["smoke:analysis"]).toBe("node scripts/analysis-smoke.mjs");
    expect(verifyScriptSource).not.toContain("smoke:analysis");
    expect(verifyScriptSource).not.toContain("analysis-smoke");
  });

  it("keeps the smoke runner organized around deterministic named steps", () => {
    expect(smokeScriptSource).toContain("sourceBrowserSmokeSteps");
    expect(smokeScriptSource).toContain("analysisWorkspaceParitySteps");
    expect(smokeScriptSource).toContain("source-browser.youtube-video-tabs");
    expect(smokeScriptSource).toContain("workspace-parity.source-group-disabled-export");
    expect(smokeScriptSource).toContain("workspace-parity.opened-single-run-tools");
    expect(smokeScriptSource).toContain("assertOpenedRunNotebookLmExportContract");
    expect(smokeScriptSource).toContain('clickRowActionByText(ctx.socket, "run-companion-runs-panel"');
    expect(smokeScriptSource).toContain("assertEmptyFixtureSummary(verificationSummary)");
    expect(smokeScriptSource).toContain("expected.filter((label) => text.includes(label))");
    expect(smokeScriptSource).toContain("waitForCurrentContext");
    expect(smokeScriptSource).toContain("waitForOpenedRunSurface");
    expect(smokeScriptSource).toContain("analysis-current-context");
    expect(smokeScriptSource).toContain("runSmokeSteps");
    expect(smokeScriptSource).toContain("finally");
    expect(smokeScriptSource).toContain("fixturesTouched");
    expect(smokeScriptSource).toContain("if (fixturesTouched && ctx?.socket)");
    expect(smokeScriptSource).toContain("cleanupFixtures");
  });

  it("keeps bridge, helper, assertion, and artifact behavior centralized", () => {
    expect(helperSource).toContain("bridgeRequest");
    expect(helperSource).toContain("executeJs");
    expect(helperSource).toContain("waitForText");
    expect(helperSource).toContain("clickByText");
    expect(helperSource).toContain("clickByTextWithinSmokeId");
    expect(helperSource).toContain("clickRowActionByText");
    expect(helperSource).toContain("clickBySmokeId");
    expect(helperSource).toContain("getVisibleTextSummary");
    expect(helperSource).toContain("assertTabOrderLabels");
    expect(helperSource).toContain("fixtureSummaryKeys");
    expect(helperSource).toContain("assertEmptyFixtureSummary");
    expect(helperSource).toContain("assertDisabledWithReason");
    expect(helperSource).toContain("captureArtifacts");
    expect(helperSource).toContain("capture_native_screenshot");
    expect(helperSource).toContain("resize_window");
    expect(helperSource).toContain("startupTimeoutMs = 90000");
    expect(helperSource).toContain("app-identifier-mismatch");
  });

  it("associates source-group NotebookLM disabled reason through aria-describedby", () => {
    expect(reportWorkspaceToolsSource).toContain("const exportReasonId = \"notebooklm-export-disabled-reason\"");
    expect(reportWorkspaceToolsSource).toContain('smokeId="notebooklm-export-button"');
    expect(reportWorkspaceToolsSource).toContain("ariaDescribedby={exportDisabledReason ? exportReasonId : undefined}");
    expect(reportWorkspaceToolsSource).toContain('id={exportReasonId}');
    expect(reportWorkspaceToolsSource).toContain('data-smoke-id="notebooklm-export-disabled-reason"');
  });

  it("renders smoke hooks only for stable analysis UI contracts", () => {
    expect(reportWorkspaceToolsSource).toContain('data-smoke-id="analysis-workspace-tools"');
    expect(reportCanvasSource).toContain('smokeId="report-canvas-mode-report"');
    expect(reportCanvasSource).toContain('smokeId="report-canvas-mode-source"');
    expect(reportSetupPanelSource).toContain('data-smoke-id="analysis-report-setup"');
    expect(reportSourceSurfaceSource).toContain('data-smoke-id="analysis-source-surface"');
    expect(sourceBrowserShellSource).toContain('data-smoke-id="source-browser-tabs"');
    expect(sourceReaderHeaderSource).toMatch(/smokeId\s*=\s*"source-browser-header"/);
    expect(sourceReaderHeaderSource).toContain("data-smoke-id={smokeId}");
    expect(reportSourceSurfaceSource).toContain('smokeId="run-snapshot-header"');
    expect(reportSourceSurfaceSource).toContain('smokeId="source-browser-header"');
    expect(reportCanvasSource).toContain('data-smoke-id="template-editor-drawer"');
    expect(reportCanvasSource).toContain('data-smoke-id="source-group-editor-drawer"');
    expect(desktopDialogSource).toContain("smokeId");
    expect(desktopDialogSource).toContain("data-smoke-id={smokeId}");
    expect(notebookLmExportDialogSource).toContain('smokeId="notebooklm-export-dialog"');
    expect(compactSourceRailSource).toContain('smokeId="analysis-source-switcher-trigger"');
    expect(compactSourceRailSource).toContain('data-smoke-id="analysis-current-context"');
    expect(sourceSwitcherPanelSource).toContain('data-smoke-id="source-switcher-panel"');
    expect(sourceSwitcherPanelSource).toContain('data-smoke-id="source-switcher-search"');
    expect(runCompanionTabsSource).toContain('smokeId="run-companion-runs-tab"');
    expect(runCompanionRunsTabSource).toContain('data-smoke-id="run-companion-runs-panel"');
    expect(runCompanionRunsTabSource).toContain('data-smoke-id="runs-search"');
  });

  it("keeps run snapshot source-browser tabs exact and activity-free", () => {
    const labels = sourceBrowserTabsForSubject({
      kind: "run_snapshot",
      snapshot: {
        runId: 1,
        scopeType: "source_group",
        scopeLabel: "Fixture snapshot",
        readerKind: "source_group",
        sourceType: "telegram",
        sourceSubtype: "supergroup",
      },
    }).map((tab) => tab.label);

    expect(labels).toEqual(["Sources", "Items", "Metadata"]);
    expect(labels).not.toContain("Activity");
  });

  it("keeps smoke selectors out of non-analysis source files", () => {
    const srcDir = path.join(repoRoot, "src");
    const offenders = collectSourceFiles(srcDir)
      .filter((file) => !file.endsWith(".test.ts"))
      .filter((file) => existsSync(file))
      .filter((file) => {
        const normalized = file.replaceAll("\\", "/");
        return !normalized.includes("/src/lib/components/analysis/")
          && !normalized.includes("/src/routes/analysis/")
          && !sharedSmokePrimitiveFiles.has(path.relative(repoRoot, file).replaceAll("\\", "/"));
      })
      .filter((file) => readdirSync(path.dirname(file)).includes(path.basename(file)))
      .filter((file) => {
        const content = readFileSync(file, "utf8");
        return content.includes("data-smoke-id") || content.includes("smokeId=");
      });

    expect(offenders.map((file) => path.relative(repoRoot, file))).toEqual([]);
    expect(desktopDialogSource).not.toContain('data-smoke-id="');
    expect(reportWorkspaceToolsSource).toContain('data-smoke-id="analysis-workspace-tools"');
  });
});
```

- [ ] **Step 2: Add helper unit tests**

Create `src/lib/analysis-smoke-helpers.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  SmokeAssertionError,
  SmokeBridgeError,
  assertEmptyFixtureSummary,
  assertTabOrderLabels,
  bridgePortCandidates,
  classifyBridgeFailure,
  executeJs,
  expectedFixtureLabels,
  sanitizeArtifactName,
  validateFixtureLabels,
  validateFixtureSummary,
} from "../../scripts/analysis-smoke-helpers.mjs";

describe("analysis smoke helper contracts", () => {
  it("builds the MCP bridge port range deterministically", () => {
    expect(bridgePortCandidates(9223, 9226)).toEqual([9223, 9224, 9225, 9226]);
  });

  it("sanitizes deterministic step names for artifact paths", () => {
    expect(sanitizeArtifactName("source-browser.youtube-video-tabs")).toBe("source-browser.youtube-video-tabs");
    expect(sanitizeArtifactName("Workspace Parity: Source Group")).toBe("workspace-parity-source-group");
  });

  it("asserts exact tab order", () => {
    expect(() => assertTabOrderLabels(["Sources", "Items", "Metadata"], ["Sources", "Items", "Metadata"]))
      .not.toThrow();
    expect(() => assertTabOrderLabels(["Sources", "Metadata", "Items"], ["Sources", "Items", "Metadata"]))
      .toThrow(SmokeAssertionError);
  });

  it("validates required fixture labels", () => {
    expect(validateFixtureLabels(expectedFixtureLabels)).toEqual(expectedFixtureLabels);
    expect(() => validateFixtureLabels(expectedFixtureLabels.filter((label) => !label.includes("YouTube Video"))))
      .toThrow(SmokeAssertionError);
  });

  it("validates deterministic fixture summary minimums", () => {
    expect(validateFixtureSummary({
      accounts: 1,
      chatMessages: 2,
      llmProfiles: 1,
      promptTemplates: 1,
      runs: 6,
      snapshotMessages: 4,
      sourceGroups: 1,
      sources: 4,
      youtubePlaylistItems: 2,
      youtubeTranscriptSegments: 3,
    })).toBe(true);

    expect(() => validateFixtureSummary({
      accounts: 1,
      chatMessages: 2,
      llmProfiles: 1,
      promptTemplates: 1,
      runs: 6,
      snapshotMessages: 4,
      sourceGroups: 0,
      sources: 4,
      youtubePlaylistItems: 2,
      youtubeTranscriptSegments: 3,
    })).toThrow(SmokeAssertionError);
  });

  it("validates empty fixture cleanup summaries", () => {
    const emptySummary = {
      accounts: 0,
      chatMessages: 0,
      llmProfiles: 0,
      promptTemplates: 0,
      runs: 0,
      snapshotMessages: 0,
      sourceGroups: 0,
      sources: 0,
      youtubePlaylistItems: 0,
      youtubeTranscriptSegments: 0,
    };

    expect(assertEmptyFixtureSummary(emptySummary)).toBe(true);

    expect(() => assertEmptyFixtureSummary({
      ...emptySummary,
      snapshotMessages: 1,
    })).toThrow(SmokeAssertionError);

    const { accounts, ...missingAccountSummary } = emptySummary;
    expect(() => assertEmptyFixtureSummary(missingAccountSummary)).toThrow(SmokeAssertionError);
  });

  it("classifies bridge failures distinctly", () => {
    expect(classifyBridgeFailure(new SmokeBridgeError("bridge unavailable", "bridge-unavailable")).kind)
      .toBe("bridge-unavailable");
    expect(classifyBridgeFailure(new Error("ASSERT: missing tab")).kind).toBe("assertion");
    expect(classifyBridgeFailure(new Error("Script execution timeout")).kind).toBe("script-timeout");
  });

  it("keeps executeJs assertion failures typed as smoke assertions", async () => {
    await expect(executeJs(fakeSocketResponse({
      id: "execute_js-1",
      success: false,
      error: "ASSERT: missing source-browser-tabs",
    }), "return true;")).rejects.toThrow(SmokeAssertionError);
  });

  it("classifies app identifier mismatch separately from unavailable bridge", () => {
    expect(classifyBridgeFailure(new SmokeBridgeError("unexpected app identifier", "app-identifier-mismatch")).kind)
      .toBe("app-identifier-mismatch");
  });
});

function fakeSocketResponse(response: Record<string, unknown>) {
  const listeners = new Map<string, Set<(event: { data?: string }) => void>>();
  const socket = {
    addEventListener(type: string, listener: (event: { data?: string }) => void) {
      const set = listeners.get(type) ?? new Set();
      set.add(listener);
      listeners.set(type, set);
    },
    removeEventListener(type: string, listener: (event: { data?: string }) => void) {
      listeners.get(type)?.delete(listener);
    },
    send(message: string) {
      const request = JSON.parse(message);
      queueMicrotask(() => {
        const next = { ...response, id: request.id };
        listeners.get("message")?.forEach((listener) => listener({ data: JSON.stringify(next) }));
      });
    },
  };
  return socket;
}
```

- [ ] **Step 3: Add a failing Rust cleanup contract test and fixture label expectation**

In `src-tauri/src/analysis/fixtures.rs`, update the source-group label expectation in `seed_creates_safe_account_prompt_profile_sources_and_group` from:

```rust
"SELECT COUNT(*) FROM analysis_source_groups WHERE name = '__analysis_redesign_fixture__ Telegram Group'"
```

to:

```rust
"SELECT COUNT(*) FROM analysis_source_groups WHERE name = '__analysis_redesign_fixture__ Telegram Source Group'"
```

Add this test inside the existing `#[cfg(test)] mod tests`:

```rust
#[tokio::test]
async fn clear_preserves_non_fixture_groups_and_members() {
    let pool = fixture_pool().await;

    let real_account_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (label, api_id, api_hash, created_at)
         VALUES ('Personal', 1, '', 1)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("insert non-fixture account");

    let real_source_id: i64 = sqlx::query_scalar(
        "INSERT INTO sources (
            source_type, source_subtype, account_id, external_id, title,
            is_active, is_member, created_at
         )
         VALUES ('telegram', 'channel', ?, 'real-source', 'Real Source', 1, 1, 1)
         RETURNING id",
    )
    .bind(real_account_id)
    .fetch_one(&pool)
    .await
    .expect("insert non-fixture source");

    let real_group_id: i64 = sqlx::query_scalar(
        "INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)
         VALUES ('Real Group', 'telegram', 1, 1)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("insert non-fixture group");

    sqlx::query(
        "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
         VALUES (?, ?, 1)",
    )
    .bind(real_group_id)
    .bind(real_source_id)
    .execute(&pool)
    .await
    .expect("insert non-fixture group member");

    insert_minimal_clear_fixture(&pool).await;

    clear_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("clear fixtures");

    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM analysis_source_groups WHERE name = 'Real Group'").await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM analysis_source_group_members member
             JOIN analysis_source_groups group_row ON group_row.id = member.group_id
             JOIN sources source_row ON source_row.id = member.source_id
             WHERE group_row.name = 'Real Group' AND source_row.title = 'Real Source'",
        )
        .await,
        1
    );
}
```

- [ ] **Step 4: Run the new failing tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-ui-smoke-contract.test.ts src/lib/analysis-smoke-helpers.test.ts
```

Expected: FAIL because `scripts/analysis-smoke.mjs`, `scripts/analysis-smoke-helpers.mjs`, package script, and smoke selectors do not exist yet.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml clear_preserves_non_fixture_groups_and_members
```

Expected: FAIL because the fixture label has not been renamed and the new cleanup test may expose row-id assumptions.

- [ ] **Step 5: Commit failing tests**

Run:

```powershell
git add src/lib/analysis-ui-smoke-contract.test.ts src/lib/analysis-smoke-helpers.test.ts src-tauri/src/analysis/fixtures.rs docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
git commit -m "test: add analysis smoke harness contracts"
```

Expected: commit succeeds with intentionally failing tests.

---

### Task 2: Stable Smoke Selectors

**Files:**
- Modify: `src/lib/components/ui/Button.svelte`
- Modify: `src/lib/components/desktop-dialog.svelte`
- Modify: `src/lib/components/analysis/report-workspace-tools.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-setup-panel.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/source-reader-header.svelte`
- Modify: `src/lib/components/analysis/notebooklm-export-dialog.svelte`
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
- Modify: `src/lib/components/analysis/source-switcher-panel.svelte`
- Modify: `src/lib/components/analysis/run-companion-tabs.svelte`
- Modify: `src/lib/components/analysis/run-companion-runs-tab.svelte`

- [ ] **Step 1: Add a `smokeId` prop to `Button`**

In `src/lib/components/ui/Button.svelte`, add `smokeId` to `$props()` and the prop type:

```svelte
    smokeId,
```

```ts
    smokeId?: string;
```

Render it on the native button:

```svelte
  data-smoke-id={smokeId}
```

The native button opening should include:

```svelte
<button
  {id}
  {type}
  {role}
  {disabled}
  {title}
  data-smoke-id={smokeId}
```

- [ ] **Step 2: Add a `smokeId` prop to `DesktopDialog`**

In `src/lib/components/desktop-dialog.svelte`, add:

```svelte
    smokeId,
```

and:

```ts
    smokeId?: string;
```

Render it on `.dialog-card`:

```svelte
            data-smoke-id={smokeId}
```

- [ ] **Step 3: Mark workspace tools and associated disabled reason**

In `src/lib/components/analysis/report-workspace-tools.svelte`, change:

```svelte
<section class="report-workspace-tools" aria-label="Workspace tools">
```

to:

```svelte
<section class="report-workspace-tools" aria-label="Workspace tools" data-smoke-id="analysis-workspace-tools">
```

Add `smokeId` to the export button:

```svelte
          smokeId="notebooklm-export-button"
```

Change the disabled reason span to:

```svelte
          <span id={exportReasonId} class="workspace-tool-helper" data-smoke-id="notebooklm-export-disabled-reason">
            {exportDisabledReason}
          </span>
```

Do not change `ariaDescribedby={exportDisabledReason ? exportReasonId : undefined}`.

- [ ] **Step 4: Mark canvas and drawers**

In `src/lib/components/analysis/report-canvas.svelte`, change:

```svelte
<section class="report-canvas">
```

to:

```svelte
<section class="report-canvas" data-smoke-id="analysis-report-canvas">
```

Change drawer wrappers:

```svelte
    <div class="workspace-template-editor-drawer" aria-label="Template editor drawer" data-smoke-id="template-editor-drawer">
```

```svelte
    <div class="workspace-group-editor-drawer" aria-label="Source group editor drawer" data-smoke-id="source-group-editor-drawer">
```

Add `smokeId` to the canvas mode buttons so smoke tests never click broad `Report` / `Source` text:

```svelte
        <Button
          type="button"
          role="tab"
          smokeId="report-canvas-mode-report"
          ...
        >
          Report
        </Button>
```

```svelte
        <Button
          type="button"
          role="tab"
          smokeId="report-canvas-mode-source"
          ...
        >
          Source
        </Button>
```

- [ ] **Step 5: Mark report setup and source surface**

In `src/lib/components/analysis/report-setup-panel.svelte`, change:

```svelte
<section class="report-setup-panel" aria-label="Report setup">
```

to:

```svelte
<section class="report-setup-panel" aria-label="Report setup" data-smoke-id="analysis-report-setup">
```

In `src/lib/components/analysis/report-source-surface.svelte`, change:

```svelte
<section class="report-source-surface" data-surface={canvasSurface}>
```

to:

```svelte
<section class="report-source-surface" data-surface={canvasSurface} data-smoke-id="analysis-source-surface">
```

- [ ] **Step 6: Mark source browser tabs and reader headers**

In `src/lib/components/analysis/source-browser-shell.svelte`, change:

```svelte
  <nav class="source-browser-tabs" aria-label="Source browser tabs">
```

to:

```svelte
  <nav class="source-browser-tabs" aria-label="Source browser tabs" data-smoke-id="source-browser-tabs">
```

In `src/lib/components/analysis/source-reader-header.svelte`, add a `smokeId` prop with a live-source default:

```svelte
    smokeId = "source-browser-header",
```

and add it to the props type:

```svelte
    smokeId?: string;
```

Then change:

```svelte
<header class="source-reader-header" aria-label={title}>
```

to:

```svelte
<header class="source-reader-header" aria-label={title} data-smoke-id={smokeId}>
```

In `src/lib/components/analysis/report-source-surface.svelte`, pass explicit header smoke ids from the owning surface:

```svelte
      <SourceReaderHeader
        smokeId="run-snapshot-header"
        title="Run snapshot"
        ...
      />
```

```svelte
      <SourceReaderHeader
        smokeId="source-browser-header"
        ...
      />
```

- [ ] **Step 7: Mark NotebookLM dialog**

In `src/lib/components/analysis/notebooklm-export-dialog.svelte`, add this prop to `DesktopDialog`:

```svelte
  smokeId="notebooklm-export-dialog"
```

- [ ] **Step 8: Mark source and run navigation surfaces**

In `src/lib/components/analysis/compact-source-rail.svelte`, add:

```svelte
      smokeId="analysis-source-switcher-trigger"
```

to the `Open source switcher` `Button`, and add:

```svelte
      data-smoke-id="analysis-current-context"
```

to the `.current-context-button`.

In `src/lib/components/analysis/source-switcher-panel.svelte`, change the root section to:

```svelte
<section class="source-switcher-panel" aria-label="Source switcher panel" data-smoke-id="source-switcher-panel">
```

Change the search label to:

```svelte
  <label class="search-field" data-smoke-id="source-switcher-search">
```

In `src/lib/components/analysis/run-companion-tabs.svelte`, add `smokeId="run-companion-runs-tab"` to the `Runs` tab button.

In `src/lib/components/analysis/run-companion-runs-tab.svelte`, change:

```svelte
<section class="run-companion-runs-tab">
```

to:

```svelte
<section class="run-companion-runs-tab" data-smoke-id="run-companion-runs-panel">
```

and change the search label to:

```svelte
    <label data-smoke-id="runs-search">
```

- [ ] **Step 9: Run selector contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-ui-smoke-contract.test.ts
```

Expected: still FAIL because scripts and helpers are not implemented, but selector-related assertions pass.

- [ ] **Step 10: Commit smoke selectors**

Run:

```powershell
git add src/lib/components/ui/Button.svelte src/lib/components/desktop-dialog.svelte src/lib/components/analysis/report-workspace-tools.svelte src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-setup-panel.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/source-reader-header.svelte src/lib/components/analysis/notebooklm-export-dialog.svelte src/lib/components/analysis/compact-source-rail.svelte src/lib/components/analysis/source-switcher-panel.svelte src/lib/components/analysis/run-companion-tabs.svelte src/lib/components/analysis/run-companion-runs-tab.svelte docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
git commit -m "test: add analysis smoke selectors"
```

Expected: commit succeeds.

---

### Task 3: Fixture Label And Marker-Scoped Cleanup

**Files:**
- Modify: `src-tauri/src/analysis/fixtures.rs`

- [ ] **Step 1: Rename the analysis source-group fixture label**

In `src-tauri/src/analysis/fixtures.rs`, change:

```rust
const TELEGRAM_GROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Group";
```

to:

```rust
const TELEGRAM_GROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Source Group";
```

- [ ] **Step 2: Keep the cleanup test independent of hard-coded row ids**

In `clear_preserves_non_fixture_groups_and_members`, confirm the non-fixture account/source/group inserts return their ids or assert through stable labels. Do not assert with assumed ids such as:

```rust
"SELECT COUNT(*) FROM analysis_source_group_members WHERE group_id = 1 AND source_id = 1",
```

Use a returned-id assertion:

```rust
&format!(
    "SELECT COUNT(*) FROM analysis_source_group_members WHERE group_id = {real_group_id} AND source_id = {real_source_id}",
),
```

If the existing local test helper does not accept `&String`, use:

```rust
let member_count_sql = format!(
    "SELECT COUNT(*) FROM analysis_source_group_members WHERE group_id = {real_group_id} AND source_id = {real_source_id}",
);
assert_eq!(count(&pool, &member_count_sql).await, 1);
```

or a join assertion through `Real Group` / `Real Source`, as shown in the Task 1 test snippet. The goal is to test cleanup scope, not SQLite row id allocation.

- [ ] **Step 3: Run Rust fixture tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests
```

Expected: PASS.

- [ ] **Step 4: Commit fixture cleanup contract**

Run:

```powershell
git add src-tauri/src/analysis/fixtures.rs docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
git commit -m "test: tighten analysis fixture cleanup scope"
```

Expected: commit succeeds.

---

### Task 4: Smoke Helper Layer

**Files:**
- Create: `scripts/analysis-smoke-helpers.mjs`
- Modify: `src/lib/analysis-smoke-helpers.test.ts`

- [ ] **Step 1: Implement pure and bridge helper foundations**

Create `scripts/analysis-smoke-helpers.mjs`:

```js
import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

export const bridgePortStart = 9223;
export const bridgePortEnd = 9322;
export const expectedAppIdentifier = "org.ai.extractum";
export const artifactRoot = path.join("tmp", "analysis-smoke");

export const fixtureLabels = {
  telegramChannel: "__analysis_redesign_fixture__ Telegram Channel",
  telegramSupergroup: "__analysis_redesign_fixture__ Telegram Supergroup",
  youtubeVideo: "__analysis_redesign_fixture__ YouTube Video",
  youtubePlaylist: "__analysis_redesign_fixture__ YouTube Playlist",
  telegramSourceGroup: "__analysis_redesign_fixture__ Telegram Source Group",
  completedSnapshotRun: "__analysis_redesign_fixture__ Completed Snapshot Run",
  groupSnapshotRun: "__analysis_redesign_fixture__ Group Snapshot Run",
};

export const expectedFixtureLabels = Object.values(fixtureLabels);

export const fixtureSummaryKeys = [
  "accounts",
  "chatMessages",
  "llmProfiles",
  "promptTemplates",
  "runs",
  "snapshotMessages",
  "sourceGroups",
  "sources",
  "youtubePlaylistItems",
  "youtubeTranscriptSegments",
];

export class SmokeAssertionError extends Error {
  constructor(message, details = {}) {
    super(message.startsWith("ASSERT:") ? message : `ASSERT: ${message}`);
    this.name = "SmokeAssertionError";
    this.kind = "assertion";
    this.details = details;
  }
}

export class SmokeBridgeError extends Error {
  constructor(message, kind = "bridge-error", details = {}) {
    super(message);
    this.name = "SmokeBridgeError";
    this.kind = kind;
    this.details = details;
  }
}

export function bridgePortCandidates(start = bridgePortStart, end = bridgePortEnd) {
  return Array.from({ length: end - start + 1 }, (_, index) => start + index);
}

export function sanitizeArtifactName(value) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    || "analysis-smoke-step";
}

export function assertTabOrderLabels(actual, expected) {
  const actualJoined = actual.join(" | ");
  const expectedJoined = expected.join(" | ");
  if (actualJoined !== expectedJoined) {
    throw new SmokeAssertionError(`tab order mismatch: expected ${expectedJoined}, got ${actualJoined}`, {
      actual,
      expected,
    });
  }
  return true;
}

export function validateFixtureLabels(labels) {
  const missing = expectedFixtureLabels.filter((label) => !labels.includes(label));
  if (missing.length > 0) {
    throw new SmokeAssertionError(`missing fixture labels: ${missing.join(", ")}`, { missing, labels });
  }
  return labels;
}

export function validateFixtureSummary(summary) {
  const expected = {
    accounts: 1,
    chatMessages: 2,
    llmProfiles: 1,
    promptTemplates: 1,
    runs: 6,
    snapshotMessages: 4,
    sourceGroups: 1,
    sources: 4,
    youtubePlaylistItems: 2,
    youtubeTranscriptSegments: 3,
  };
  const failures = Object.entries(expected)
    .filter(([key, value]) => Number(summary?.[key] ?? 0) < value)
    .map(([key, value]) => `${key}>=${value}`);
  if (failures.length > 0) {
    throw new SmokeAssertionError(`fixture summary below expected minimums: ${failures.join(", ")}`, {
      summary,
      expected,
    });
  }
  return true;
}

export function assertEmptyFixtureSummary(summary) {
  const missing = fixtureSummaryKeys.filter((key) => !Object.prototype.hasOwnProperty.call(summary ?? {}, key));
  if (missing.length > 0) {
    throw new SmokeAssertionError(`fixture cleanup summary missing keys: ${missing.join(", ")}`, {
      summary,
      missing,
    });
  }

  const nonEmpty = fixtureSummaryKeys
    .filter((key) => Number(summary[key] ?? 0) !== 0)
    .map((key) => `${key}=${summary[key]}`);
  const unexpectedNonEmpty = Object.entries(summary ?? {})
    .filter(([key, value]) => !fixtureSummaryKeys.includes(key) && Number(value ?? 0) !== 0)
    .map(([key, value]) => `${key}=${value}`);
  nonEmpty.push(...unexpectedNonEmpty);
  if (nonEmpty.length > 0) {
    throw new SmokeAssertionError(`fixture cleanup verification found remaining rows: ${nonEmpty.join(", ")}`, {
      summary,
    });
  }
  return true;
}

export function classifyBridgeFailure(error) {
  if (error instanceof SmokeAssertionError) return { kind: "assertion", message: error.message };
  if (error instanceof SmokeBridgeError) return { kind: error.kind, message: error.message };
  const message = error instanceof Error ? error.message : String(error);
  if (message.startsWith("ASSERT:")) return { kind: "assertion", message };
  if (message.includes("Script execution timeout")) return { kind: "script-timeout", message };
  if (message.includes("WebSocket") || message.includes("socket")) return { kind: "bridge-disconnect", message };
  return { kind: "app-contract", message };
}

export function createRequestId(prefix = "analysis-smoke") {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export async function bridgeRequest(socket, command, args = {}, timeoutMs = 5000) {
  const id = createRequestId(command);
  const request = JSON.stringify({ id, command, args });
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(new SmokeBridgeError(`bridge timeout waiting for ${command}`, "bridge-timeout", { command }));
    }, timeoutMs);

    function cleanup() {
      clearTimeout(timeout);
      socket.removeEventListener("message", onMessage);
      socket.removeEventListener("close", onClose);
      socket.removeEventListener("error", onError);
    }

    function onMessage(event) {
      const text = typeof event.data === "string" ? event.data : "";
      let response;
      try {
        response = JSON.parse(text);
      } catch {
        return;
      }
      if (response.id !== id) return;
      cleanup();
      resolve(response);
    }

    function onClose() {
      cleanup();
      reject(new SmokeBridgeError(`bridge disconnected during ${command}`, "bridge-disconnect", { command }));
    }

    function onError() {
      cleanup();
      reject(new SmokeBridgeError(`bridge socket error during ${command}`, "bridge-disconnect", { command }));
    }

    socket.addEventListener("message", onMessage);
    socket.addEventListener("close", onClose);
    socket.addEventListener("error", onError);
    socket.send(request);
  });
}

export async function executeJs(socket, script, timeoutMs = 5000) {
  const response = await bridgeRequest(socket, "execute_js", { script, windowLabel: "main" }, timeoutMs);
  if (!response.success) {
    const message = response.error ?? "execute_js failed";
    if (message.startsWith("ASSERT:")) {
      throw new SmokeAssertionError(message, { response });
    }
    throw new SmokeBridgeError(message, "script-failure", { response });
  }
  return response.data;
}

export function waitForSocketOpen(socket, timeoutMs = 1500) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(new SmokeBridgeError("bridge unavailable", "bridge-unavailable"));
    }, timeoutMs);
    function cleanup() {
      clearTimeout(timeout);
      socket.removeEventListener("open", onOpen);
      socket.removeEventListener("error", onError);
    }
    function onOpen() {
      cleanup();
      resolve(socket);
    }
    function onError() {
      cleanup();
      reject(new SmokeBridgeError("bridge unavailable", "bridge-unavailable"));
    }
    socket.addEventListener("open", onOpen);
    socket.addEventListener("error", onError);
  });
}

export async function discoverBridge({
  WebSocketCtor = globalThis.WebSocket,
  ports = bridgePortCandidates(),
  startupTimeoutMs = 90000,
} = {}) {
  if (typeof WebSocketCtor !== "function") {
    throw new SmokeBridgeError("Node runtime does not provide globalThis.WebSocket", "missing-websocket");
  }

  const deadline = Date.now() + startupTimeoutMs;
  let lastIdentifierMismatch = null;

  while (Date.now() < deadline) {
    for (const port of ports) {
      const socket = new WebSocketCtor(`ws://127.0.0.1:${port}`);
      try {
        await waitForSocketOpen(socket);
        const backend = await bridgeRequest(socket, "invoke_tauri", {
          command: "plugin:mcp-bridge|get_backend_state",
          args: { windowLabel: "main" },
        });
        if (backend.success && backend.data?.app?.identifier === expectedAppIdentifier) {
          return { socket, port, backendState: backend.data };
        }
        if (backend.success && backend.data?.app?.identifier) {
          lastIdentifierMismatch = {
            port,
            identifier: backend.data.app.identifier,
          };
        }
        socket.close();
      } catch {
        try {
          socket.close();
        } catch {
          // Ignore failed close while probing unavailable ports.
        }
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  if (lastIdentifierMismatch) {
    throw new SmokeBridgeError(
      `MCP bridge app identifier mismatch on port ${lastIdentifierMismatch.port}: ${lastIdentifierMismatch.identifier}`,
      "app-identifier-mismatch",
      lastIdentifierMismatch,
    );
  }

  throw new SmokeBridgeError(`No ${expectedAppIdentifier} MCP bridge found on ports ${ports[0]}-${ports.at(-1)}`, "bridge-unavailable");
}

export async function resizeWindow(socket, width = 1280, height = 860) {
  const response = await bridgeRequest(socket, "resize_window", {
    width,
    height,
    windowId: "main",
    logical: true,
  });
  if (!response.success || response.data?.success === false) {
    throw new SmokeBridgeError(response.error ?? response.data?.error ?? "resize_window failed", "bridge-command-failed", { response });
  }
  return response.data;
}

export async function captureNativeScreenshot(socket, maxWidth = 1280) {
  return bridgeRequest(socket, "capture_native_screenshot", {
    format: "png",
    maxWidth,
    windowLabel: "main",
  }, 8000);
}

export async function getVisibleTextSummary(socket) {
  return executeJs(socket, `
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    const chunks = [];
    while (walker.nextNode()) {
      const value = walker.currentNode.textContent.trim().replace(/\\s+/g, " ");
      if (value) chunks.push(value);
    }
    return chunks.join("\\n").slice(0, 12000);
  `);
}

export async function waitForText(socket, text, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const found = await executeJs(socket, `
      return document.body.innerText.includes(${JSON.stringify(text)});
    `);
    if (found) return true;
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new SmokeAssertionError(`text not found: ${text}`);
}

export async function clickBySmokeId(socket, smokeId) {
  return executeJs(socket, `
    const element = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!element) throw new Error('ASSERT: missing smoke id ${smokeId}');
    element.click();
    return true;
  `);
}

export async function clickByText(socket, text) {
  return executeJs(socket, `
    const targetText = ${JSON.stringify(text)};
    const candidates = Array.from(document.querySelectorAll('button, [role="button"], a, summary'));
    const match = candidates.find((element) => element.innerText.trim().includes(targetText)
      || element.getAttribute('aria-label')?.includes(targetText)
      || element.getAttribute('title')?.includes(targetText));
    if (!match) throw new Error('ASSERT: clickable text not found: ' + targetText);
    match.click();
    return true;
  `);
}

export async function clickByTextWithinSmokeId(socket, smokeId, text) {
  return executeJs(socket, `
    const container = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!container) throw new Error('ASSERT: missing smoke id ${smokeId}');
    const targetText = ${JSON.stringify(text)};
    const candidates = Array.from(container.querySelectorAll('button, [role="button"], a, summary'));
    const match = candidates.find((element) => element.innerText.trim().includes(targetText)
      || element.getAttribute('aria-label')?.includes(targetText)
      || element.getAttribute('title')?.includes(targetText));
    if (!match) throw new Error('ASSERT: scoped clickable text not found: ' + targetText);
    match.click();
    return true;
  `);
}

export async function clickRowActionByText(socket, containerSmokeId, rowText, actionText) {
  return executeJs(socket, `
    const container = document.querySelector('[data-smoke-id="${containerSmokeId}"]');
    if (!container) throw new Error('ASSERT: missing smoke id ${containerSmokeId}');
    const rowText = ${JSON.stringify(rowText)};
    const actionText = ${JSON.stringify(actionText)};
    const rowCandidates = Array.from(container.querySelectorAll('li, article, .source-row, .group-row, button, [role="row"]'));
    const row = rowCandidates.find((candidate) => candidate.innerText.includes(rowText));
    if (!row) throw new Error('ASSERT: row not found: ' + rowText);
    const action = Array.from(row.querySelectorAll('button, [role="button"], a'))
      .find((candidate) => candidate.innerText.trim().includes(actionText)
        || candidate.getAttribute('aria-label')?.includes(actionText)
        || candidate.getAttribute('title')?.includes(actionText));
    if (!action) throw new Error('ASSERT: row action not found: ' + actionText + ' in ' + rowText);
    action.click();
    return true;
  `);
}

export async function fillByLabel(socket, label, value) {
  return executeJs(socket, `
    const labelText = ${JSON.stringify(label)};
    const value = ${JSON.stringify(value)};
    const controls = Array.from(document.querySelectorAll('input, textarea'));
    const control = controls.find((element) => element.getAttribute('aria-label') === labelText
      || element.closest('label')?.innerText.includes(labelText)
      || element.getAttribute('placeholder') === labelText);
    if (!control) throw new Error('ASSERT: input not found: ' + labelText);
    control.focus();
    control.value = value;
    control.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: value }));
    control.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
  `);
}

export async function readTabLabels(socket, smokeId = "source-browser-tabs") {
  return executeJs(socket, `
    const container = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!container) throw new Error('ASSERT: missing tab container ${smokeId}');
    return Array.from(container.querySelectorAll('button, [role="tab"]')).map((button) => button.innerText.trim());
  `);
}

export async function assertSelectedTab(socket, expectedTab, smokeId = "source-browser-tabs") {
  return executeJs(socket, `
    const container = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!container) throw new Error('ASSERT: missing tab container ${smokeId}');
    const selected = Array.from(container.querySelectorAll('button, [role="tab"]'))
      .find((button) => button.getAttribute('aria-selected') === 'true'
        || button.classList.contains('selected')
        || button.classList.contains('primary'));
    const label = selected?.innerText.trim() ?? '';
    if (label !== ${JSON.stringify(expectedTab)}) {
      throw new Error('ASSERT: expected selected tab ${expectedTab}, got ' + label);
    }
    return true;
  `);
}

export async function assertDisabledWithReason(socket, buttonText, reasonText) {
  return executeJs(socket, `
    const buttonText = ${JSON.stringify(buttonText)};
    const reasonText = ${JSON.stringify(reasonText)};
    const button = Array.from(document.querySelectorAll('button')).find((candidate) => candidate.innerText.includes(buttonText));
    if (!button) throw new Error('ASSERT: disabled button not found: ' + buttonText);
    if (!button.disabled) throw new Error('ASSERT: button is not disabled: ' + buttonText);
    const describedBy = button.getAttribute('aria-describedby');
    if (!describedBy) throw new Error('ASSERT: disabled button missing aria-describedby');
    const reason = document.getElementById(describedBy);
    if (!reason) throw new Error('ASSERT: described disabled reason missing: ' + describedBy);
    if (reason.dataset.smokeId !== 'notebooklm-export-disabled-reason') {
      throw new Error('ASSERT: disabled reason missing smoke id');
    }
    if (!reason.innerText.includes(reasonText)) {
      throw new Error('ASSERT: disabled reason mismatch: ' + reason.innerText);
    }
    return true;
  `);
}

export async function captureArtifacts({ socket, artifactDir, stepName, error }) {
  await mkdir(artifactDir, { recursive: true });
  const failure = classifyBridgeFailure(error);
  await writeFile(path.join(artifactDir, "failure.json"), JSON.stringify({
    stepName,
    failure,
    error: error instanceof Error ? { name: error.name, message: error.message, stack: error.stack } : String(error),
  }, null, 2));

  try {
    const summary = await getVisibleTextSummary(socket);
    await writeFile(path.join(artifactDir, "visible-text.txt"), summary);
  } catch (summaryError) {
    await writeFile(path.join(artifactDir, "visible-text-error.txt"), String(summaryError?.message ?? summaryError));
  }

  try {
    const dom = await executeJs(socket, `
      return Array.from(document.querySelectorAll('[data-smoke-id]')).map((element) => ({
        smokeId: element.dataset.smokeId,
        tag: element.tagName.toLowerCase(),
        text: element.innerText?.trim().slice(0, 500) ?? '',
      }));
    `);
    await writeFile(path.join(artifactDir, "smoke-dom.json"), JSON.stringify(dom, null, 2));
  } catch (domError) {
    await writeFile(path.join(artifactDir, "smoke-dom-error.txt"), String(domError?.message ?? domError));
  }

  try {
    const screenshot = await captureNativeScreenshot(socket);
    if (screenshot.success && typeof screenshot.data === "string") {
      await writeFile(path.join(artifactDir, "screenshot.data-url.txt"), screenshot.data);
    } else {
      await writeFile(path.join(artifactDir, "screenshot-error.txt"), screenshot.error ?? "screenshot unavailable");
    }
  } catch (screenshotError) {
    await writeFile(path.join(artifactDir, "screenshot-error.txt"), String(screenshotError?.message ?? screenshotError));
  }
}

export function spawnTauriDev({ command, args, cwd }) {
  return spawn(command, args, { cwd, shell: false, stdio: "inherit" });
}

export function killProcessTree(child) {
  if (!child?.pid) return Promise.resolve();
  if (process.platform === "win32") {
    return new Promise((resolve) => {
      const killer = spawn("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
      killer.on("close", () => resolve());
      killer.on("error", () => resolve());
    });
  }
  child.kill("SIGTERM");
  return Promise.resolve();
}
```

- [ ] **Step 2: Run helper unit tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-smoke-helpers.test.ts
```

Expected: PASS.

- [ ] **Step 3: Commit helper layer**

Run:

```powershell
git add scripts/analysis-smoke-helpers.mjs src/lib/analysis-smoke-helpers.test.ts docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
git commit -m "feat: add analysis smoke helper layer"
```

Expected: commit succeeds.

---

### Task 5: Smoke Runner, Bridge Probe, And Fixture Lifecycle

**Files:**
- Create: `scripts/analysis-smoke.mjs`
- Modify: `package.json`
- Modify: `.gitignore`
- Modify: `src/lib/analysis-ui-smoke-contract.test.ts`

- [ ] **Step 1: Add package script and artifact ignore**

In `package.json`, add:

```json
"smoke:analysis": "node scripts/analysis-smoke.mjs"
```

Keep it outside `verify`.

In `.gitignore`, add:

```gitignore
/tmp/analysis-smoke/
```

- [ ] **Step 2: Add smoke runner bootstrap**

Create `scripts/analysis-smoke.mjs`:

```js
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  SmokeAssertionError,
  artifactRoot,
  assertEmptyFixtureSummary,
  assertDisabledWithReason,
  assertSelectedTab,
  assertTabOrderLabels,
  bridgeRequest,
  captureArtifacts,
  clickBySmokeId,
  clickByText,
  clickByTextWithinSmokeId,
  clickRowActionByText,
  discoverBridge,
  executeJs,
  expectedAppIdentifier,
  fixtureLabels,
  fillByLabel,
  getVisibleTextSummary,
  killProcessTree,
  readTabLabels,
  resizeWindow,
  sanitizeArtifactName,
  spawnTauriDev,
  validateFixtureLabels,
  validateFixtureSummary,
  waitForText,
} from "./analysis-smoke-helpers.mjs";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const args = new Set(process.argv.slice(2));
const probeOnly = args.has("--probe-only");

export const sourceBrowserSmokeSteps = [];
export const analysisWorkspaceParitySteps = [];

function npmRunStep(scriptName) {
  if (process.env.npm_execpath) {
    return { command: process.execPath, args: [process.env.npm_execpath, "run", scriptName] };
  }
  if (process.platform === "win32") {
    return { command: "npm.cmd", args: ["run", scriptName] };
  }
  return { command: "npm", args: ["run", scriptName] };
}

function artifactDirFor(stepName) {
  const stamp = new Date().toISOString().replace(/[:.]/g, "-");
  return path.join(repoRoot, artifactRoot, `${stamp}-${sanitizeArtifactName(stepName)}`);
}

async function launchApp() {
  const step = npmRunStep("tauri");
  const child = spawnTauriDev({
    command: step.command,
    args: [...step.args, "dev"],
    cwd: repoRoot,
  });
  return child;
}

async function navigateAnalysis(ctx) {
  await executeJs(ctx.socket, `window.location.assign('/analysis'); return true;`, 1000).catch(() => true);
  await waitForText(ctx.socket, "Workspace tools", 30000);
}

async function probeBridgeCapabilities(ctx) {
  console.log("PASS bridge.get_backend_state");
  await resizeWindow(ctx.socket, 1280, 860);
  console.log("PASS bridge.resize_window");
  const title = await executeJs(ctx.socket, "return document.title;", 5000);
  if (typeof title !== "string") {
    throw new SmokeAssertionError("execute_js did not return document title");
  }
  console.log("PASS bridge.execute_js");
  const screenshot = await bridgeRequest(ctx.socket, "capture_native_screenshot", {
    format: "png",
    maxWidth: 320,
    windowLabel: "main",
  }, 8000);
  if (!screenshot.success && String(screenshot.error ?? "").includes("Unknown command")) {
    throw new SmokeAssertionError("capture_native_screenshot command is not registered");
  }
  console.log(screenshot.success ? "PASS bridge.capture_native_screenshot" : "WARN bridge.capture_native_screenshot best-effort failed");
}

async function invokeFixtureCommand(ctx, command) {
  return executeJs(ctx.socket, `
    return await window.__TAURI__.core.invoke(${JSON.stringify(command)});
  `, 30000);
}

async function fixtureLabelsFromDom(ctx) {
  const expected = Object.values(fixtureLabels);
  return executeJs(ctx.socket, `
    const text = document.body.innerText;
    const expected = ${JSON.stringify(expected)};
    return expected.filter((label) => text.includes(label));
  `, 5000);
}

async function seedFixtures(ctx) {
  await invokeFixtureCommand(ctx, "clear_analysis_redesign_fixtures");
  const summary = await invokeFixtureCommand(ctx, "seed_analysis_redesign_fixtures");
  validateFixtureSummary(summary);
  await navigateAnalysis(ctx);
  await waitForText(ctx.socket, "__analysis_redesign_fixture__", 30000);
  const labels = await fixtureLabelsFromDom(ctx);
  validateFixtureLabels(labels);
}

async function cleanupFixtures(ctx) {
  const removedSummary = await invokeFixtureCommand(ctx, "clear_analysis_redesign_fixtures");
  const verificationSummary = await invokeFixtureCommand(ctx, "clear_analysis_redesign_fixtures");
  assertEmptyFixtureSummary(verificationSummary);
  return { removedSummary, verificationSummary };
}

async function runSmokeSteps(ctx, steps) {
  for (const step of steps) {
    console.log(`\\nSTEP ${step.name}`);
    try {
      await step.run(ctx);
      console.log(`PASS ${step.name}`);
    } catch (error) {
      const dir = artifactDirFor(step.name);
      await captureArtifacts({ socket: ctx.socket, artifactDir: dir, stepName: step.name, error });
      console.error(`FAIL ${step.name}`);
      console.error(error instanceof Error ? error.stack ?? error.message : String(error));
      console.error(`Artifacts: ${dir}`);
      throw error;
    }
  }
}

async function main() {
  let child = null;
  let ctx = null;
  let failed = null;
  let cleanupFailed = null;
  let fixturesTouched = false;

  try {
    child = await launchApp();
    const bridge = await discoverBridge();
    ctx = { socket: bridge.socket, port: bridge.port, backendState: bridge.backendState };
    if (ctx.backendState.app.identifier !== expectedAppIdentifier) {
      throw new SmokeAssertionError(`unexpected app identifier ${ctx.backendState.app.identifier}`);
    }
    await probeBridgeCapabilities(ctx);
    if (probeOnly) return;
    fixturesTouched = true;
    await seedFixtures(ctx);
    await runSmokeSteps(ctx, [...sourceBrowserSmokeSteps, ...analysisWorkspaceParitySteps]);
  } catch (error) {
    failed = error;
  } finally {
    if (fixturesTouched && ctx?.socket) {
      try {
        await cleanupFixtures(ctx);
      } catch (error) {
        cleanupFailed = error;
        const dir = artifactDirFor("cleanup.failed");
        await mkdir(dir, { recursive: true });
        await writeFile(path.join(dir, "cleanup-error.txt"), error instanceof Error ? error.stack ?? error.message : String(error));
        console.error(`Cleanup failed. Artifacts: ${dir}`);
      }
    }

    if (ctx?.socket) {
      try {
        ctx.socket.close();
      } catch {
        // Ignore socket close failures after cleanup.
      }
    }
    await killProcessTree(child);
  }

  if (failed || cleanupFailed) {
    process.exit(1);
  }
}

await main();

export {
  assertDisabledWithReason,
  assertSelectedTab,
  assertTabOrderLabels,
  clickBySmokeId,
  clickByText,
  clickByTextWithinSmokeId,
  clickRowActionByText,
  executeJs,
  fixtureLabels,
  fillByLabel,
  getVisibleTextSummary,
  readTabLabels,
  waitForText,
};
```

- [ ] **Step 3: Run contract and probe-focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-ui-smoke-contract.test.ts src/lib/analysis-smoke-helpers.test.ts
```

Expected: `analysis-smoke-helpers.test.ts` PASS. `analysis-ui-smoke-contract.test.ts` may still FAIL because scenario names are not added yet.

- [ ] **Step 4: Run the opt-in bridge probe manually**

Run:

```powershell
npm.cmd run smoke:analysis -- --probe-only
```

Expected in a GUI-capable local environment:

```text
PASS bridge.get_backend_state
PASS bridge.resize_window
PASS bridge.execute_js
PASS bridge.capture_native_screenshot
```

Acceptable alternate screenshot line:

```text
WARN bridge.capture_native_screenshot best-effort failed
```

Stop if `get_backend_state`, `resize_window`, or `execute_js` fails. Confirm this mode does not seed, navigate for fixture verification, or call `clear_analysis_redesign_fixtures`; `fixturesTouched` must remain `false`.

- [ ] **Step 5: Commit runner bootstrap**

Run:

```powershell
git add package.json .gitignore scripts/analysis-smoke.mjs src/lib/analysis-ui-smoke-contract.test.ts docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
git commit -m "feat: add analysis smoke runner bootstrap"
```

Expected: commit succeeds.

---

### Task 6: Source Browser Smoke Scenarios

**Files:**
- Modify: `scripts/analysis-smoke.mjs`
- Modify: `src/lib/analysis-ui-smoke-contract.test.ts`

- [ ] **Step 1: Add reusable UI workflow helpers to `scripts/analysis-smoke.mjs`**

Add these functions above `sourceBrowserSmokeSteps`:

```js
async function closeTransientUi(ctx) {
  await executeJs(ctx.socket, `
    const closeButtons = Array.from(document.querySelectorAll('button'))
      .filter((button) => button.innerText.trim() === 'Close' || button.getAttribute('aria-label') === 'Close dialog');
    for (const button of closeButtons) button.click();
    return true;
  `).catch(() => true);
}

async function switchCanvasMode(ctx, mode) {
  await clickBySmokeId(ctx.socket, mode === "source" ? "report-canvas-mode-source" : "report-canvas-mode-report");
  await waitForText(ctx.socket, mode === "source" ? "Source material" : "Workspace tools");
}

async function openSourceSwitcher(ctx) {
  await clickBySmokeId(ctx.socket, "analysis-source-switcher-trigger");
  await waitForText(ctx.socket, "Switch source context");
}

async function waitForCurrentContext(ctx, label, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const selected = await executeJs(ctx.socket, `
      const current = document.querySelector('[data-smoke-id="analysis-current-context"]');
      return Boolean(current?.innerText.includes(${JSON.stringify(label)}));
    `).catch(() => false);
    if (selected) return true;
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new SmokeAssertionError(`current context did not switch to ${label}`);
}

async function selectSource(ctx, label) {
  await closeTransientUi(ctx);
  await openSourceSwitcher(ctx);
  await fillByLabel(ctx.socket, "Search sources or groups", label);
  await waitForText(ctx.socket, label);
  await clickByTextWithinSmokeId(ctx.socket, "source-switcher-panel", label);
  await waitForCurrentContext(ctx, label);
}

async function selectGroup(ctx, label) {
  await closeTransientUi(ctx);
  await openSourceSwitcher(ctx);
  await fillByLabel(ctx.socket, "Search sources or groups", label);
  await waitForText(ctx.socket, label);
  await clickByTextWithinSmokeId(ctx.socket, "source-switcher-panel", label);
  await waitForCurrentContext(ctx, label);
}

async function openRunsTab(ctx) {
  await clickBySmokeId(ctx.socket, "run-companion-runs-tab");
  await waitForText(ctx.socket, "Search runs");
}

async function waitForOpenedRunSurface(ctx, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const opened = await executeJs(ctx.socket, `
      return Boolean(document.querySelector('.report-viewer, .report-run-header'));
    `).catch(() => false);
    if (opened) return true;
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new SmokeAssertionError("opened run surface did not appear");
}

async function openRun(ctx, label) {
  await closeTransientUi(ctx);
  await openRunsTab(ctx);
  await fillByLabel(ctx.socket, "Search runs", label);
  await waitForText(ctx.socket, label);
  await clickRowActionByText(ctx.socket, "run-companion-runs-panel", label, "Open");
  await waitForOpenedRunSurface(ctx);
}

async function expectTabs(ctx, labels, selected) {
  const actual = await readTabLabels(ctx.socket);
  assertTabOrderLabels(actual, labels);
  await assertSelectedTab(ctx.socket, selected);
}
```

- [ ] **Step 2: Add Source Browser smoke steps**

Replace:

```js
export const sourceBrowserSmokeSteps = [];
```

with:

```js
export const sourceBrowserSmokeSteps = [
  {
    name: "source-browser.telegram-live-tabs",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await selectSource(ctx, fixtureLabels.telegramChannel);
      await switchCanvasMode(ctx, "source");
      await expectTabs(ctx, ["Timeline", "Items", "Metadata", "Activity"], "Timeline");
    },
  },
  {
    name: "source-browser.youtube-video-tabs",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await selectSource(ctx, fixtureLabels.youtubeVideo);
      await switchCanvasMode(ctx, "source");
      await expectTabs(ctx, ["Transcript", "Comments", "Items", "Metadata", "Activity"], "Transcript");
    },
  },
  {
    name: "source-browser.youtube-playlist-tabs",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await selectSource(ctx, fixtureLabels.youtubePlaylist);
      await switchCanvasMode(ctx, "source");
      await expectTabs(ctx, ["Videos", "Items", "Metadata", "Activity"], "Videos");
    },
  },
  {
    name: "source-browser.live-source-group-tabs",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await selectGroup(ctx, fixtureLabels.telegramSourceGroup);
      await switchCanvasMode(ctx, "source");
      await expectTabs(ctx, ["Sources", "Items", "Metadata", "Activity"], "Sources");
    },
  },
  {
    name: "source-browser.run-snapshot-tabs",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await openRun(ctx, fixtureLabels.completedSnapshotRun);
      await switchCanvasMode(ctx, "source");
      await waitForText(ctx.socket, "View live source");
      await executeJs(ctx.socket, `
        const header = document.querySelector('[data-smoke-id="run-snapshot-header"]');
        if (!header) throw new Error('ASSERT: run snapshot header missing');
        if (!header.innerText.includes('Run snapshot')) throw new Error('ASSERT: run snapshot header missing label');
        if (!header.innerText.includes('View live source')) throw new Error('ASSERT: run snapshot header missing View live source');
        return true;
      `);
      await expectTabs(ctx, ["Sources", "Items", "Metadata"], "Sources");
    },
  },
];
```

- [ ] **Step 3: Run Source Browser model and contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/source-browser-model.test.ts src/lib/analysis-smoke-helpers.test.ts
```

Expected: PASS for tab contracts and helper behavior. The full `analysis-ui-smoke-contract.test.ts` remains red until Workspace Parity smoke names are added in Task 7.

- [ ] **Step 4: Run opt-in smoke before Workspace Parity scenarios**

Run:

```powershell
npm.cmd run smoke:analysis
```

Expected in a GUI-capable local environment: Source Browser steps PASS. If a Source Browser step fails, inspect `tmp/analysis-smoke/` artifacts and fix before continuing. Workspace Parity checks are added in Task 7.

If the current environment is not GUI-capable, do not treat this as a smoke failure. Record the exact reason as `not run in this environment`, continue with non-GUI tests, and make sure the final verification archive uses `Result: not run in this environment` rather than `passed`.

- [ ] **Step 5: Commit Source Browser smoke scenarios**

Run:

```powershell
git add scripts/analysis-smoke.mjs src/lib/analysis-ui-smoke-contract.test.ts docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
git commit -m "feat: smoke analysis source browser surfaces"
```

Expected: commit succeeds.

---

### Task 7: Workspace Parity Smoke Scenarios

**Files:**
- Modify: `scripts/analysis-smoke.mjs`
- Modify: `src/lib/analysis-ui-smoke-contract.test.ts`

- [ ] **Step 1: Add Workspace Parity helpers**

Add these functions to `scripts/analysis-smoke.mjs`:

```js
async function assertWorkspaceToolsAboveBody(ctx, bodySmokeId) {
  return executeJs(ctx.socket, `
    const tools = document.querySelector('[data-smoke-id="analysis-workspace-tools"]');
    const body = document.querySelector('[data-smoke-id="${bodySmokeId}"]');
    if (!tools) throw new Error('ASSERT: workspace tools missing');
    if (!body) throw new Error('ASSERT: body missing ${bodySmokeId}');
    const position = tools.compareDocumentPosition(body);
    if (!(position & Node.DOCUMENT_POSITION_FOLLOWING)) {
      throw new Error('ASSERT: workspace tools do not precede ${bodySmokeId}');
    }
    return true;
  `);
}

async function openNotebookLmExportDialog(ctx) {
  await clickBySmokeId(ctx.socket, "notebooklm-export-button");
  await waitForText(ctx.socket, "Export for NotebookLM");
  await executeJs(ctx.socket, `
    const dialog = document.querySelector('[data-smoke-id="notebooklm-export-dialog"]');
    if (!dialog) throw new Error('ASSERT: NotebookLM export dialog missing');
    return true;
  `);
}

async function closeDialog(ctx) {
  await clickByText(ctx.socket, "Close dialog").catch(() => executeJs(ctx.socket, `
    const button = document.querySelector('button[aria-label="Close dialog"]');
    if (button) button.click();
    return true;
  `));
}

async function assertNoNotebookLmExportDialog(ctx, reason) {
  const reasonText = JSON.stringify(reason);
  await executeJs(ctx.socket, `
    const dialog = document.querySelector('[data-smoke-id="notebooklm-export-dialog"]');
    if (dialog) throw new Error('ASSERT: NotebookLM export dialog opened unexpectedly: ' + ${reasonText});
    return true;
  `);
}

async function assertOpenedRunNotebookLmExportContract(ctx) {
  const state = await executeJs(ctx.socket, `
    const button = document.querySelector('[data-smoke-id="notebooklm-export-button"]');
    return {
      exists: Boolean(button),
      disabled: Boolean(button?.disabled) || button?.getAttribute('aria-disabled') === 'true',
      text: button?.innerText ?? '',
    };
  `);

  if (!state.exists) {
    await assertNoNotebookLmExportDialog(ctx, "opened single-source run has no restored currentSource");
    return;
  }

  if (state.disabled) {
    await clickBySmokeId(ctx.socket, "notebooklm-export-button").catch(() => true);
    await assertNoNotebookLmExportDialog(ctx, "disabled opened-run export must not use saved-run metadata alone");
    return;
  }

  await openNotebookLmExportDialog(ctx);
  await closeDialog(ctx);
}

async function assertDrawer(ctx, triggerText, smokeId) {
  await clickByTextWithinSmokeId(ctx.socket, "analysis-workspace-tools", triggerText);
  await executeJs(ctx.socket, `
    const drawer = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!drawer) throw new Error('ASSERT: drawer missing ${smokeId}');
    return true;
  `);
  await clickByTextWithinSmokeId(ctx.socket, "analysis-workspace-tools", triggerText.replace("Edit", "Hide"));
}
```

- [ ] **Step 2: Add Workspace Parity smoke steps**

Replace:

```js
export const analysisWorkspaceParitySteps = [];
```

with:

```js
export const analysisWorkspaceParitySteps = [
  {
    name: "workspace-parity.single-source-setup-tools",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await selectSource(ctx, fixtureLabels.youtubeVideo);
      await switchCanvasMode(ctx, "report");
      await assertWorkspaceToolsAboveBody(ctx, "analysis-report-setup");
      await openNotebookLmExportDialog(ctx);
      await closeDialog(ctx);
      await assertDrawer(ctx, "Edit templates", "template-editor-drawer");
      await assertDrawer(ctx, "Edit groups", "source-group-editor-drawer");
      await waitForText(ctx.socket, "Run report");
      await waitForText(ctx.socket, "Sync source");
      await closeTransientUi(ctx);
    },
  },
  {
    name: "workspace-parity.source-group-disabled-export",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await selectGroup(ctx, fixtureLabels.telegramSourceGroup);
      await switchCanvasMode(ctx, "report");
      await assertWorkspaceToolsAboveBody(ctx, "analysis-report-setup");
      await assertDisabledWithReason(
        ctx.socket,
        "Export for NotebookLM",
        "Source-group NotebookLM export is not implemented yet.",
      );
      await assertDrawer(ctx, "Edit templates", "template-editor-drawer");
      await assertDrawer(ctx, "Edit groups", "source-group-editor-drawer");
      await closeTransientUi(ctx);
    },
  },
  {
    name: "workspace-parity.opened-single-run-tools",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await openRun(ctx, fixtureLabels.completedSnapshotRun);
      await switchCanvasMode(ctx, "report");
      await executeJs(ctx.socket, `
        const tools = document.querySelector('[data-smoke-id="analysis-workspace-tools"]');
        const report = document.querySelector('.report-viewer, .report-run-header');
        if (!tools) throw new Error('ASSERT: workspace tools missing for opened run');
        if (!report) throw new Error('ASSERT: opened run report body missing');
        return true;
      `);
      await assertOpenedRunNotebookLmExportContract(ctx);
    },
  },
  {
    name: "workspace-parity.opened-source-group-run-disabled-export",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await openRun(ctx, fixtureLabels.groupSnapshotRun);
      await assertDisabledWithReason(
        ctx.socket,
        "Export for NotebookLM",
        "Source-group NotebookLM export is not implemented yet.",
      );
    },
  },
  {
    name: "workspace-parity.source-mode-tools-placement",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await selectSource(ctx, fixtureLabels.telegramChannel);
      await switchCanvasMode(ctx, "source");
      await assertWorkspaceToolsAboveBody(ctx, "analysis-source-surface");
    },
  },
];
```

- [ ] **Step 3: Run focused frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts src/lib/analysis-ui-smoke-contract.test.ts src/lib/analysis-smoke-helpers.test.ts
```

Expected: PASS.

- [ ] **Step 4: Run opt-in smoke command**

Run:

```powershell
npm.cmd run smoke:analysis
```

Expected in a GUI-capable local environment: all Source Browser and Workspace Parity steps PASS.

If the current environment is not GUI-capable, do not treat this as a smoke failure. Record the exact reason as `not run in this environment`, continue with non-GUI tests, and make sure the final verification archive uses `Result: not run in this environment` rather than `passed`.

- [ ] **Step 5: Commit Workspace Parity smoke scenarios**

Run:

```powershell
git add scripts/analysis-smoke.mjs src/lib/analysis-ui-smoke-contract.test.ts docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
git commit -m "feat: smoke analysis workspace parity"
```

Expected: commit succeeds.

---

### Task 8: Cleanup, Artifacts, And Final Verification

**Files:**
- Modify: `scripts/analysis-smoke.mjs`
- Modify: `scripts/analysis-smoke-helpers.mjs`
- Modify: `docs/superpowers/specs/2026-05-30-analysis-ui-smoke-harness-design.md`
- Modify: `docs/superpowers/plans/README.md`
- Create: `docs/superpowers/archive/verification/2026-05-30-analysis-ui-smoke-harness.md`
- Delete: `docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md`

- [ ] **Step 1: Harden cleanup exit behavior**

In `scripts/analysis-smoke.mjs`, ensure `main()` exits nonzero when either `failed` or `cleanupFailed` is set. Keep the existing final condition:

```js
  if (failed || cleanupFailed) {
    process.exit(1);
  }
```

Add a success message before the final condition:

```js
  if (!failed && !cleanupFailed) {
    console.log("\\nAnalysis UI smoke passed.");
  }
```

- [ ] **Step 2: Ensure required artifacts are always attempted before cleanup**

In `runSmokeSteps`, confirm this order remains:

```js
      const dir = artifactDirFor(step.name);
      await captureArtifacts({ socket: ctx.socket, artifactDir: dir, stepName: step.name, error });
      console.error(`FAIL ${step.name}`);
```

Do not move artifact capture into `finally`; it must happen before fixture cleanup changes the DOM.

- [ ] **Step 3: Run full frontend and Rust verification**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-ui-smoke-contract.test.ts src/lib/analysis-smoke-helpers.test.ts src/lib/source-browser-model.test.ts src/lib/analysis-report-canvas.test.ts
```

Expected: PASS.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests
```

Expected: PASS.

Run:

```powershell
npm.cmd run check
```

Expected: Svelte check reports 0 errors.

- [ ] **Step 4: Run project verification**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS. Confirm output does not include `smoke:analysis`.

- [ ] **Step 5: Run opt-in smoke and archive result**

Run:

```powershell
npm.cmd run smoke:analysis
```

Expected in a GUI-capable environment:

```text
PASS source-browser.telegram-live-tabs
PASS source-browser.youtube-video-tabs
PASS source-browser.youtube-playlist-tabs
PASS source-browser.live-source-group-tabs
PASS source-browser.run-snapshot-tabs
PASS workspace-parity.single-source-setup-tools
PASS workspace-parity.source-group-disabled-export
PASS workspace-parity.opened-single-run-tools
PASS workspace-parity.opened-source-group-run-disabled-export
PASS workspace-parity.source-mode-tools-placement
Analysis UI smoke passed.
```

Create `docs/superpowers/archive/verification/2026-05-30-analysis-ui-smoke-harness.md`:

~~~markdown
# Analysis UI Smoke Harness Verification

Date: 2026-05-30

Command:

```powershell
npm.cmd run smoke:analysis
```

Result: passed

Covered surfaces:

- Source Browser: Telegram live source
- Source Browser: YouTube video
- Source Browser: YouTube playlist
- Source Browser: live source group
- Source Browser: run snapshot
- Workspace Parity: single-source setup tools
- Workspace Parity: source-group disabled export
- Workspace Parity: opened single-source run tools
- Workspace Parity: opened source-group run disabled export
- Workspace Parity: source mode tools placement

Notes:

- Smoke command remains opt-in.
- `npm.cmd run verify` does not run `smoke:analysis`.
- Fixture cleanup completed and second cleanup summary verified zero fixture rows.
~~~

If local GUI automation cannot run in the current environment, create the same file with `Result: not run in this environment` and include the exact failure reason. Keep the covered surfaces list as intended coverage, add a note that the opt-in GUI smoke still needs a GUI-capable host, and do not claim smoke pass without command output.

- [ ] **Step 6: Update spec status, active plan index, and completed plan file**

In `docs/superpowers/specs/2026-05-30-analysis-ui-smoke-harness-design.md`, change:

```markdown
Status: active design
```

to:

```markdown
Status: implemented
```

In `docs/superpowers/plans/README.md`, change active plan entry to `- None currently.` after all tasks are complete.

Remove the completed implementation plan from `docs/superpowers/plans/` after final verification is represented by tests, current docs, the verification archive, and Git history:

```powershell
git rm docs/superpowers/plans/2026-05-30-analysis-ui-smoke-harness-implementation.md
```

Do not leave the completed implementation plan in active plans. Archive the implementation plan only if the team explicitly wants the plan itself as historical context; otherwise Git history is the archive.

- [ ] **Step 7: Commit final verification docs**

Run:

```powershell
git add scripts/analysis-smoke.mjs scripts/analysis-smoke-helpers.mjs docs/superpowers/specs/2026-05-30-analysis-ui-smoke-harness-design.md docs/superpowers/plans/README.md docs/superpowers/archive/verification/2026-05-30-analysis-ui-smoke-harness.md
git commit -m "docs: verify analysis smoke harness"
```

Expected: commit succeeds.

---

## Final Check

- [ ] **Step 1: Confirm clean tree**

Run:

```powershell
git status --short --branch
```

Expected: clean working tree on the implementation branch.

- [ ] **Step 2: Confirm recent commits tell the story**

Run:

```powershell
git log --oneline -8
```

Expected commits include tests, selectors, fixture cleanup, helper layer, runner bootstrap, Source Browser smoke, Workspace Parity smoke, and verification docs.
