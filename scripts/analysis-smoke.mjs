// @ts-nocheck
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
  classifyBridgeFailure,
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
const analysisWorkspaceStateKey = "extractum.analysis.workspace.v1";
const captureFailedSnapshotErrorText = "Snapshot capture failed: fixture write boundary unavailable";
const captureFailedReportText = "This capture-failed fixture report remains readable.";

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
      await openRun(ctx, fixtureLabels.groupSnapshotRun);
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
export const savedRunAffordanceSmokeSteps = [
  {
    name: "saved-runs-affordance.rows",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await assertRunRowAffordance(
        ctx,
        fixtureLabels.missingSnapshotRun,
        ["Legacy snapshot missing"],
        [captureFailedSnapshotErrorText, "fixture write boundary unavailable"],
      );
      await assertRunRowAffordance(
        ctx,
        fixtureLabels.captureFailedSnapshotRun,
        ["Snapshot capture failed"],
        [captureFailedSnapshotErrorText, "fixture write boundary unavailable"],
      );
    },
  },
  {
    name: "saved-runs-affordance.missing-legacy",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await openRun(ctx, fixtureLabels.missingSnapshotRun);
      await switchCanvasMode(ctx, "report");
      await openRunDetails(ctx);
      const opened = await openedRunText(ctx);
      assertTextContains(
        opened.header,
        "Saved report is readable, but this legacy run has no saved source snapshot.",
        "missing legacy opened header",
      );
      assertTextContains(opened.header, "Legacy run has no saved snapshot", "missing legacy details");

      await switchToSourceSurface(ctx);
      const sourceText = await sourceSurfaceText(ctx);
      assertTextContains(
        sourceText,
        "Older saved runs may not include frozen source rows",
        "missing legacy Source detail",
      );
      assertTextContains(sourceText, "Snapshot unavailable", "missing legacy Source badge");
      assertTextOmits(sourceText, "Run snapshot\nSources", "missing legacy Source saved snapshot browser");
      await assertLiveSourceClarificationIfAvailable(ctx);

      await assertEvidenceDisabledForRun(
        ctx,
        fixtureLabels.missingSnapshotRun,
        "legacy run has no saved source snapshot",
      );
      await assertChatDisabledForOpenedRun(ctx, "Older saved runs may not include frozen source rows");
    },
  },
  {
    name: "saved-runs-affordance.capture-failed",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await openRun(ctx, fixtureLabels.captureFailedSnapshotRun);
      await switchCanvasMode(ctx, "report");
      await openRunDetails(ctx);
      const opened = await openedRunText(ctx);
      assertTextContains(
        opened.header,
        "Saved report is readable, but Extractum could not save the frozen source context for this run.",
        "capture failed opened header",
      );
      assertTextContains(opened.header, "Snapshot capture failed", "capture failed details");
      assertTextContains(opened.header, captureFailedSnapshotErrorText, "capture failed details error");
      assertTextContains(opened.report, captureFailedReportText, "capture failed report body");

      await switchToSourceSurface(ctx);
      const sourceText = await sourceSurfaceText(ctx);
      assertTextContains(
        sourceText,
        "Extractum could not save the frozen source context",
        "capture failed Source detail",
      );
      assertTextContains(sourceText, captureFailedSnapshotErrorText, "capture failed Source error");
      await assertLiveSourceClarificationIfAvailable(ctx);
      await clickLiveSourceIfAvailable(ctx);

      await assertEvidenceDisabledForRun(
        ctx,
        fixtureLabels.captureFailedSnapshotRun,
        "snapshot capture failed",
      );
      await assertChatDisabledForOpenedRun(ctx, "could not save the frozen source context");
    },
  },
];
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
      await assertSourceGroupNotebookLmExportUnavailable(ctx);
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
  const smokeRuntimeRoot = path.join(repoRoot, artifactRoot, "runtime", `${process.pid}-${Date.now()}`);
  const smokeAppDataDir = path.join(smokeRuntimeRoot, "appdata");
  const smokeLocalAppDataDir = path.join(smokeRuntimeRoot, "localappdata");
  const smokeConfigDir = path.join(smokeRuntimeRoot, "config");
  await Promise.all([
    mkdir(smokeAppDataDir, { recursive: true }),
    mkdir(smokeLocalAppDataDir, { recursive: true }),
    mkdir(smokeConfigDir, { recursive: true }),
  ]);
  const child = spawnTauriDev({
    command: step.command,
    args: [...step.args, "dev"],
    cwd: repoRoot,
    env: {
      ...process.env,
      APPDATA: smokeAppDataDir,
      LOCALAPPDATA: smokeLocalAppDataDir,
      XDG_CONFIG_HOME: smokeConfigDir,
    },
  });
  return child;
}

async function navigateAnalysis(ctx) {
  await executeJs(ctx.socket, `
    const analysisWorkspaceStateKey = ${JSON.stringify(analysisWorkspaceStateKey)};
    localStorage.removeItem(analysisWorkspaceStateKey);
    window.location.assign('/analysis?smoke=' + Date.now());
    return true;
  `, 1000).catch(() => true);
  await waitForSmokeId(ctx, "analysis-source-switcher-trigger", 30000);
}

async function captureSmokeLocalStorage(ctx) {
  ctx.initialAnalysisWorkspaceState = await executeJs(ctx.socket, `
    const analysisWorkspaceStateKey = ${JSON.stringify(analysisWorkspaceStateKey)};
    return localStorage.getItem(analysisWorkspaceStateKey);
  `, 5000).catch(() => null);
}

async function restoreSmokeLocalStorage(ctx) {
  if (!Object.prototype.hasOwnProperty.call(ctx, "initialAnalysisWorkspaceState")) return;
  await executeJs(ctx.socket, `
    const analysisWorkspaceStateKey = ${JSON.stringify(analysisWorkspaceStateKey)};
    const value = ${JSON.stringify(ctx.initialAnalysisWorkspaceState)};
    if (value === null) {
      localStorage.removeItem(analysisWorkspaceStateKey);
    } else {
      localStorage.setItem(analysisWorkspaceStateKey, value);
    }
    return true;
  `, 5000).catch(() => undefined);
}

async function probeBridgeCapabilities(ctx) {
  console.log("PASS bridge.get_backend_state");
  const title = await retryProbeCommand(ctx, (socket) => executeJs(socket, "return document.title;"), "execute_js");
  if (typeof title !== "string") {
    throw new SmokeAssertionError("execute_js did not return document title");
  }
  console.log("PASS bridge.execute_js");
  await retryProbeCommand(ctx, (socket) => resizeWindow(socket, 1280, 860), "resize_window");
  console.log("PASS bridge.resize_window");
  const screenshot = await retryProbeCommand(ctx, (socket) => bridgeRequest(socket, "capture_native_screenshot", {
    format: "png",
    maxWidth: 320,
    windowLabel: "main",
  }, 8000), "capture_native_screenshot");
  if (!screenshot.success && String(screenshot.error ?? "").includes("Unknown command")) {
    throw new SmokeAssertionError("capture_native_screenshot command is not registered");
  }
  console.log(screenshot.success ? "PASS bridge.capture_native_screenshot" : "WARN bridge.capture_native_screenshot best-effort failed");
}

async function refreshBridgeConnection(ctx, timeoutMs = 30000) {
  try {
    ctx.socket?.close();
  } catch {
    // Ignore stale socket close failures before rediscovery.
  }
  const bridge = await discoverBridge({ startupTimeoutMs: timeoutMs });
  ctx.socket = bridge.socket;
  ctx.port = bridge.port;
  ctx.backendState = bridge.backendState;
  return ctx;
}

async function retryProbeCommand(ctx, action, label, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;
  while (Date.now() < deadline) {
    try {
      return await action(ctx.socket);
    } catch (error) {
      lastError = error;
      const failure = classifyBridgeFailure(error);
      const canReconnect = ["bridge-disconnect", "bridge-timeout", "bridge-unavailable", "script-timeout"].includes(failure.kind);
      if (canReconnect && Date.now() < deadline) {
        const remainingMs = Math.max(1000, deadline - Date.now());
        await refreshBridgeConnection(ctx, remainingMs).catch((refreshError) => {
          lastError = refreshError;
        });
      }
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
  throw lastError ?? new SmokeAssertionError(`${label} did not complete before timeout`);
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

async function fixtureLabelsFromSeededUi(ctx) {
  await closeTransientUi(ctx);
  await openSourceSwitcher(ctx);
  const sourceLabels = await fixtureLabelsFromDom(ctx);
  await closeTransientUi(ctx);
  await openRunsTab(ctx);
  await clearRunsFilters(ctx);
  await showAllRuns(ctx);
  await fillByLabel(ctx.socket, "Search runs", "__analysis_redesign_fixture__");
  await waitForText(ctx.socket, fixtureLabels.captureFailedSnapshotRun);
  const runLabels = await fixtureLabelsFromDom(ctx);
  return Array.from(new Set([...sourceLabels, ...runLabels]));
}

async function seedFixtures(ctx) {
  await invokeFixtureCommand(ctx, "clear_analysis_redesign_fixtures");
  const summary = await invokeFixtureCommand(ctx, "seed_analysis_redesign_fixtures");
  validateFixtureSummary(summary);
  await invokeFixtureCommand(ctx, "clear_analysis_redesign_fixture_active_runs");
  await navigateAnalysis(ctx);
  const labels = await waitForFixtureLabels(ctx);
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
    console.log(`\nSTEP ${step.name}`);
    try {
      await step.run(ctx);
      console.log(`PASS ${step.name}`);
    } catch (error) {
      const dir = artifactDirFor(step.name);
      await captureArtifacts({ socket: ctx.socket, artifactDir: dir, stepName: step.name, error });
      if (error && typeof error === "object") {
        error.smokeArtifactsCaptured = true;
      }
      console.error(`FAIL ${step.name}`);
      console.error(error instanceof Error ? error.stack ?? error.message : String(error));
      console.error(`Artifacts: ${dir}`);
      throw error;
    }
  }
}

function assertTextContains(text, fragment, label) {
  if (!String(text ?? "").includes(fragment)) {
    throw new SmokeAssertionError(`${label} missing text: ${fragment}`);
  }
}

function assertTextOmits(text, fragment, label) {
  if (String(text ?? "").includes(fragment)) {
    throw new SmokeAssertionError(`${label} unexpectedly included text: ${fragment}`);
  }
}

async function closeTransientUi(ctx) {
  await executeJs(ctx.socket, `
    const closeButtons = Array.from(document.querySelectorAll('button'))
      .filter((button) => button.innerText.trim() === 'Close' || button.getAttribute('aria-label') === 'Close dialog');
    for (const button of closeButtons) button.click();
    return true;
  `).catch(() => true);
}

async function waitForSmokeId(ctx, smokeId, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const found = await executeJs(ctx.socket, `
      return Boolean(document.querySelector('[data-smoke-id="${smokeId}"]'));
    `).catch(() => false);
    if (found) return true;
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new SmokeAssertionError(`smoke id not found: ${smokeId}`);
}

async function waitForFixtureLabels(ctx, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  let labels = [];
  while (Date.now() < deadline) {
    labels = await fixtureLabelsFromSeededUi(ctx).catch(() => []);
    try {
      validateFixtureLabels(labels);
      return labels;
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
  return labels;
}

async function switchCanvasMode(ctx, mode) {
  await clickBySmokeId(ctx.socket, mode === "source" ? "report-canvas-mode-source" : "report-canvas-mode-report");
  await waitForSmokeId(ctx, mode === "source" ? "source-browser-tabs" : "analysis-workspace-tools", 30000);
}

async function openSourceSwitcher(ctx) {
  await clickBySmokeId(ctx.socket, "analysis-source-switcher-trigger");
  await waitForText(ctx.socket, "Switch source context");
  await waitForSourceSwitcherReady(ctx);
}

async function waitForSourceSwitcherReady(ctx, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const ready = await executeJs(ctx.socket, `
      const panel = document.querySelector('[data-smoke-id="source-switcher-panel"]');
      const text = panel?.innerText ?? "";
      return Boolean(panel) && !text.includes("Loading sources") && !text.includes("Loading groups");
    `).catch(() => false);
    if (ready) return true;
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new SmokeAssertionError("source switcher did not finish loading");
}

async function waitForCurrentContext(ctx, label, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const selected = await executeJs(ctx.socket, `
      const current = document.querySelector('[data-smoke-id="analysis-current-context"]');
      const accessibleName = [
        current?.innerText,
        current?.getAttribute('title'),
        current?.getAttribute('aria-label'),
      ].filter(Boolean).join(' ');
      return accessibleName.includes(${JSON.stringify(label)});
    `).catch(() => false);
    if (selected) return true;
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new SmokeAssertionError(`current context did not switch to ${label}`);
}

async function selectSource(ctx, label) {
  await selectSwitcherLabel(ctx, label);
  await waitForStableLiveContext(ctx, label, () => selectSwitcherLabel(ctx, label));
}

async function selectGroup(ctx, label) {
  await selectSwitcherLabel(ctx, label);
  await waitForStableLiveContext(ctx, label, () => selectSwitcherLabel(ctx, label));
}

async function selectSwitcherLabel(ctx, label) {
  await closeTransientUi(ctx);
  await openSourceSwitcher(ctx);
  await fillByLabel(ctx.socket, "Search sources or groups", label);
  await waitForText(ctx.socket, label);
  await clickByTextWithinSmokeId(ctx.socket, "source-switcher-panel", label);
  await waitForCurrentContext(ctx, label);
}

async function waitForStableLiveContext(ctx, label, retrySelection, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  let retries = 0;
  while (Date.now() < deadline) {
    const state = await executeJs(ctx.socket, `
      const current = document.querySelector('[data-smoke-id="analysis-current-context"]');
      const accessibleName = [
        current?.innerText,
        current?.getAttribute('title'),
        current?.getAttribute('aria-label'),
      ].filter(Boolean).join(' ');
      return {
        matches: accessibleName.includes(${JSON.stringify(label)}),
        openedRun: Boolean(document.querySelector('.report-run-header')),
      };
    `).catch(() => ({ matches: false, openedRun: true }));

    if (state.matches && !state.openedRun) return true;
    if (retries < 2) {
      retries += 1;
      await retrySelection();
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new SmokeAssertionError(`live context did not stabilize for ${label}`);
}

async function openRunsTab(ctx) {
  await clickBySmokeId(ctx.socket, "run-companion-runs-tab");
  await waitForText(ctx.socket, "Search runs");
}

async function showAllRuns(ctx) {
  await setRunsSegment(ctx, "Runs scope", "All runs");
  await setRunsSegment(ctx, "Runs status", "All");
}

async function clearRunsFilters(ctx) {
  const cleared = await executeJs(ctx.socket, `
    const panel = document.querySelector('[data-smoke-id="run-companion-runs-panel"]');
    if (!panel) throw new Error('ASSERT: runs panel missing');
    const button = Array.from(panel.querySelectorAll('button'))
      .find((candidate) => candidate.innerText.trim() === 'Clear filters');
    if (!button) return false;
    button.click();
    return true;
  `);
  if (!cleared) return false;

  await waitForRunsSearchValue(ctx, "");
  return true;
}

async function waitForRunsSearchValue(ctx, expectedValue, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const matches = await executeJs(ctx.socket, `
      const expectedValue = ${JSON.stringify(expectedValue)};
      const input = document.querySelector('[data-smoke-id="runs-search"] input');
      return input?.value === expectedValue;
    `).catch(() => false);
    if (matches) return true;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new SmokeAssertionError("runs search did not reset");
}

async function setRunsSegment(ctx, groupLabel, optionText) {
  await executeJs(ctx.socket, `
    const groupLabel = ${JSON.stringify(groupLabel)};
    const optionText = ${JSON.stringify(optionText)};
    const panel = document.querySelector('[data-smoke-id="run-companion-runs-panel"]');
    if (!panel) throw new Error('ASSERT: runs panel missing');
    const group = Array.from(panel.querySelectorAll('[aria-label]'))
      .find((candidate) => candidate.getAttribute('aria-label') === groupLabel);
    if (!group) throw new Error('ASSERT: runs segmented control missing: ' + groupLabel);
    const button = Array.from(group.querySelectorAll('button'))
      .find((candidate) => candidate.innerText.trim() === optionText);
    if (!button) throw new Error('ASSERT: runs segment option missing: ' + optionText);
    if (!button.classList.contains('selected')) button.click();
    return true;
  `);
  await waitForRunsSegment(ctx, groupLabel, optionText);
}

async function waitForRunsSegment(ctx, groupLabel, optionText, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const selected = await executeJs(ctx.socket, `
      const groupLabel = ${JSON.stringify(groupLabel)};
      const optionText = ${JSON.stringify(optionText)};
      const panel = document.querySelector('[data-smoke-id="run-companion-runs-panel"]');
      const group = Array.from(panel?.querySelectorAll('[aria-label]') ?? [])
        .find((candidate) => candidate.getAttribute('aria-label') === groupLabel);
      const button = Array.from(group?.querySelectorAll('button') ?? [])
        .find((candidate) => candidate.innerText.trim() === optionText);
      return Boolean(button?.classList.contains('selected'));
    `).catch(() => false);
    if (selected) return true;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new SmokeAssertionError(`runs segment did not select ${groupLabel}: ${optionText}`);
}

async function runRowText(ctx, label) {
  await closeTransientUi(ctx);
  await openRunsTab(ctx);
  await clearRunsFilters(ctx);
  await showAllRuns(ctx);
  await fillByLabel(ctx.socket, "Search runs", label);
  await waitForText(ctx.socket, label);
  return executeJs(ctx.socket, `
    const panel = document.querySelector('[data-smoke-id="run-companion-runs-panel"]');
    if (!panel) throw new Error('ASSERT: runs panel missing');
    const label = ${JSON.stringify(label)};
    const rowCandidates = Array.from(panel.querySelectorAll('li, article, .source-row, .group-row, button, [role="row"]'));
    const row = rowCandidates.find((candidate) => candidate.innerText.includes(label));
    if (!row) throw new Error('ASSERT: run row missing: ' + label);
    return row.innerText;
  `);
}

async function assertRunRowAffordance(ctx, label, expectedFragments, forbiddenFragments = []) {
  const text = await runRowText(ctx, label);
  assertTextContains(text, label, `${label} row`);
  for (const fragment of expectedFragments) {
    assertTextContains(text, fragment, `${label} row`);
  }
  for (const fragment of forbiddenFragments) {
    assertTextOmits(text, fragment, `${label} row`);
  }
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

async function openRunDetails(ctx) {
  return executeJs(ctx.socket, `
    const details = document.querySelector('.report-run-header details.run-details');
    if (!details) throw new Error('ASSERT: run details missing');
    details.open = true;
    return true;
  `);
}

async function openedRunText(ctx) {
  return executeJs(ctx.socket, `
    return {
      header: document.querySelector('.report-run-header')?.innerText ?? "",
      report: document.querySelector('.report-viewer')?.innerText ?? "",
      source: document.querySelector('[data-smoke-id="analysis-source-surface"]')?.innerText ?? "",
      companion: document.querySelector('#run-companion-panel')?.innerText ?? "",
    };
  `);
}

async function switchToSourceSurface(ctx) {
  await clickBySmokeId(ctx.socket, "report-canvas-mode-source");
  await waitForSmokeId(ctx, "analysis-source-surface", 30000);
}

async function sourceSurfaceText(ctx) {
  return executeJs(ctx.socket, `
    const surface = document.querySelector('[data-smoke-id="analysis-source-surface"]');
    if (!surface) throw new Error('ASSERT: source surface missing');
    return surface.innerText;
  `);
}

async function assertLiveSourceClarificationIfAvailable(ctx) {
  const text = await sourceSurfaceText(ctx);
  if (!text.includes("View live source")) return;
  assertTextContains(
    text,
    "View live source opens current source data. This is live data, not the saved run snapshot.",
    "live source clarification",
  );
}

async function clickLiveSourceIfAvailable(ctx) {
  const clicked = await executeJs(ctx.socket, `
    const surface = document.querySelector('[data-smoke-id="analysis-source-surface"]');
    if (!surface) throw new Error('ASSERT: source surface missing');
    const button = Array.from(surface.querySelectorAll('button'))
      .find((candidate) => candidate.innerText.includes('View live source'));
    if (!button) return false;
    button.click();
    return true;
  `);

  if (!clicked) return false;
  await waitForText(ctx.socket, "Live source");
  const text = await sourceSurfaceText(ctx);
  assertTextContains(text, "Live source", "live source header");
  assertTextOmits(text, captureFailedSnapshotErrorText, "live source after capture failed switch");
  assertTextOmits(text, "Legacy run has no saved snapshot", "live source after missing legacy switch");
  return true;
}

async function openRun(ctx, label) {
  await closeTransientUi(ctx);
  await openRunsTab(ctx);
  await clearRunsFilters(ctx);
  await showAllRuns(ctx);
  await fillByLabel(ctx.socket, "Search runs", label);
  await waitForText(ctx.socket, label);
  await clickRowActionByText(ctx.socket, "run-companion-runs-panel", label, "Open");
  await waitForOpenedRunSurface(ctx);
}

async function openCompanionTab(ctx, label) {
  await executeJs(ctx.socket, `
    const tablist = document.querySelector('[aria-label="Run companion tabs"]');
    if (!tablist) throw new Error('ASSERT: run companion tablist missing');
    const label = ${JSON.stringify(label)};
    const tab = Array.from(tablist.querySelectorAll('button, [role="tab"]'))
      .find((candidate) => candidate.innerText.trim().includes(label));
    if (!tab) throw new Error('ASSERT: companion tab missing: ' + label);
    tab.click();
    return true;
  `);
}

async function selectFirstTraceRefIfNeeded(ctx) {
  await executeJs(ctx.socket, `
    const panel = document.querySelector('.run-evidence-tab');
    if (!panel) throw new Error('ASSERT: evidence panel missing');
    if (panel.querySelector('.trace-detail')) return true;
    const firstTrace = panel.querySelector('.trace-link');
    if (!firstTrace) throw new Error('ASSERT: trace ref missing for evidence');
    firstTrace.click();
    return true;
  `);
  await waitForText(ctx.socket, "Show in source");
}

async function openEvidenceForRun(ctx, runLabel) {
  await navigateAnalysis(ctx);
  await openRun(ctx, runLabel);
  await openCompanionTab(ctx, "Evidence");
  await waitForText(ctx.socket, "Traceability");
  await selectFirstTraceRefIfNeeded(ctx);
}

async function assertShowInSourceDisabledReason(ctx, reasonFragment) {
  const buttonState = await executeJs(ctx.socket, `
    const panel = document.querySelector('.run-evidence-tab');
    if (!panel) throw new Error('ASSERT: evidence panel missing');
    const button = Array.from(panel.querySelectorAll('button'))
      .find((candidate) => candidate.innerText.includes('Show in source'));
    if (!button) throw new Error('ASSERT: Show in source button missing');
    return {
      disabled: Boolean(button.disabled),
      title: button.getAttribute('title') ?? "",
      text: button.innerText,
    };
  `);

  if (!buttonState.disabled) {
    throw new SmokeAssertionError("Show in source button is not disabled");
  }
  assertTextContains(buttonState.title, reasonFragment, "Show in source disabled reason");
}

async function assertEvidenceDisabledForRun(ctx, runLabel, reasonFragment) {
  await openEvidenceForRun(ctx, runLabel);
  const companionText = (await openedRunText(ctx)).companion;
  assertTextContains(companionText, "Snapshot unavailable:", `${runLabel} Evidence unavailable copy`);
  await assertShowInSourceDisabledReason(ctx, reasonFragment);
}

async function assertChatDisabledForOpenedRun(ctx, descriptionFragment) {
  await openCompanionTab(ctx, "Chat");
  await waitForText(ctx.socket, "Saved context unavailable");
  const companionText = (await openedRunText(ctx)).companion;
  assertTextContains(companionText, "Saved context unavailable", "Chat disabled title");
  assertTextContains(companionText, descriptionFragment, "Chat disabled description");
}

async function expectTabs(ctx, labels, selected) {
  const actual = await readTabLabels(ctx.socket);
  assertTabOrderLabels(actual, labels);
  await assertSelectedTab(ctx.socket, selected);
}

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

async function assertSourceGroupNotebookLmExportUnavailable(ctx) {
  const buttonExists = await executeJs(ctx.socket, `
    return Boolean(document.querySelector('[data-smoke-id="notebooklm-export-button"]'));
  `);

  if (buttonExists) {
    await assertDisabledWithReason(
      ctx.socket,
      "Export for NotebookLM",
      "Source-group NotebookLM export is not implemented yet.",
    );
    return;
  }

  await assertNoNotebookLmExportDialog(ctx, "opened source-group run has no restored currentGroup");
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
    await captureSmokeLocalStorage(ctx);
    if (probeOnly) return;
    fixturesTouched = true;
    await seedFixtures(ctx);
    await runSmokeSteps(ctx, [...sourceBrowserSmokeSteps, ...savedRunAffordanceSmokeSteps, ...analysisWorkspaceParitySteps]);
  } catch (error) {
    failed = error;
    if (ctx?.socket) {
      const artifactsAlreadyCaptured = Boolean(error && typeof error === "object" && error.smokeArtifactsCaptured);
      if (!artifactsAlreadyCaptured) {
        const dir = artifactDirFor("bootstrap.failed");
        await captureArtifacts({ socket: ctx.socket, artifactDir: dir, stepName: "bootstrap.failed", error }).catch(() => undefined);
        console.error(`Artifacts: ${dir}`);
      }
    }
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
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
      await restoreSmokeLocalStorage(ctx);
      try {
        ctx.socket.close();
      } catch {
        // Ignore socket close failures after cleanup.
      }
    }
    await killProcessTree(child);
  }

  if (!failed && !cleanupFailed) {
    console.log("\nAnalysis UI smoke passed.");
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
