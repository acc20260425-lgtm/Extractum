import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));

function npmStep(title, npmScript, { npmExecPath, platform }) {
  if (npmExecPath) return { title, command: process.execPath, args: [npmExecPath, "run", npmScript], npmScript };
  if (platform === "win32") throw new Error('Unable to locate npm CLI path. Run this command through "npm run verify".');
  return { title, command: "npm", args: ["run", npmScript], npmScript };
}

export function createVerifySteps({ npmExecPath = process.env.npm_execpath, platform = process.platform } = {}) {
  return [
    npmStep("npm run check:gemini-browser-sidecar-binary", "check:gemini-browser-sidecar-binary", { npmExecPath, platform }),
    { title: "node scripts/validate-testing-transition.mjs", command: process.execPath, args: ["scripts/validate-testing-transition.mjs"] },
    npmStep("npm run test:unit", "test:unit", { npmExecPath, platform }),
    npmStep("npm run test:component", "test:component", { npmExecPath, platform }),
    npmStep("npm run test:architecture", "test:architecture", { npmExecPath, platform }),
    npmStep("npm run test:legacy-contract", "test:legacy-contract", { npmExecPath, platform }),
    npmStep("npm run test:integration:os", "test:integration:os", { npmExecPath, platform }),
    npmStep("npm run test:e2e", "test:e2e", { npmExecPath, platform }),
    npmStep("npm run test:app:e2e", "test:app:e2e", { npmExecPath, platform }),
    npmStep("npm run check", "check", { npmExecPath, platform }),
    npmStep("npm run check:rustfmt", "check:rustfmt", { npmExecPath, platform }),
    { title: "cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets", command: "cargo", args: ['check', '--manifest-path', 'src-tauri/Cargo.toml', '--workspace', '--all-targets'] },
    { title: "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets", command: "cargo", args: ['test', '--manifest-path', 'src-tauri/Cargo.toml', '--workspace', '--all-targets'] },
    { title: "git diff HEAD --check", command: "git", args: ["diff", "HEAD", "--check"] },
  ];
}

function spawnStep(step) {
  return new Promise((resolve) => {
    let settled = false;
    const finish = (exitCode) => { if (!settled) { settled = true; resolve(exitCode); } };
    console.log(`\n=== ${step.title} ===`);
    const child = spawn(step.command, step.args, { cwd: repoRoot, shell: false, stdio: "inherit" });
    child.on("error", (error) => { console.error(`\nFailed to start "${step.command}": ${error.message}`); finish(1); });
    child.on("close", (code, signal) => {
      if (signal) { console.error(`\nCommand terminated by signal ${signal}: ${step.title}`); finish(1); return; }
      finish(code ?? 1);
    });
  });
}

export async function runVerification({ steps, runStep = spawnStep }) {
  for (const step of steps) {
    const exitCode = await runStep(step);
    if (exitCode !== 0) return { exitCode, failedStep: step.title };
  }
  return { exitCode: 0 };
}

async function main() {
  let steps;
  try { steps = createVerifySteps(); } catch (error) { console.error(error.message); return 1; }
  const result = await runVerification({ steps });
  if (result.exitCode !== 0) { console.error(`\nVerification failed during: ${result.failedStep}`); return result.exitCode; }
  console.log("\nAll verification checks passed.");
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) process.exit(await main());
