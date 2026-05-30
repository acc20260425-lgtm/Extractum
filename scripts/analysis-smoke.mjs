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
  await executeJs(ctx.socket, `window.location.assign('/analysis?smoke=' + Date.now()); return true;`, 1000).catch(() => true);
  await waitForSmokeId(ctx, "analysis-source-switcher-trigger", 30000);
}

async function probeBridgeCapabilities(ctx) {
  console.log("PASS bridge.get_backend_state");
  await retryProbeCommand(() => resizeWindow(ctx.socket, 1280, 860), "resize_window");
  console.log("PASS bridge.resize_window");
  const title = await retryProbeCommand(() => executeJs(ctx.socket, "return document.title;", 5000), "execute_js");
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

async function retryProbeCommand(action, label, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;
  while (Date.now() < deadline) {
    try {
      return await action();
    } catch (error) {
      lastError = error;
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

async function seedFixtures(ctx) {
  await invokeFixtureCommand(ctx, "clear_analysis_redesign_fixtures");
  const summary = await invokeFixtureCommand(ctx, "seed_analysis_redesign_fixtures");
  validateFixtureSummary(summary);
  await navigateAnalysis(ctx);
  await openSourceSwitcher(ctx);
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
    labels = await fixtureLabelsFromDom(ctx).catch(() => []);
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
