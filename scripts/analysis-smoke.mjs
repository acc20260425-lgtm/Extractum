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
    console.log(`\nSTEP ${step.name}`);
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
