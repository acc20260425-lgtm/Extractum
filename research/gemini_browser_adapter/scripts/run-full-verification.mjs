import { rmSync } from "node:fs";
import { spawnSync } from "node:child_process";

const npm = process.platform === "win32" ? "npm.cmd" : "npm";
const staleArtifactPaths = [
  "research/gemini_browser_adapter/artifacts/matrix",
  "research/gemini_browser_adapter/artifacts/playwright-results.json",
];

function run(label, args) {
  console.log(`\n== ${label} ==`);
  const result = spawnSync(npm, args, { stdio: "inherit", shell: process.platform === "win32" });
  if (result.error) {
    console.error(`${label} failed to start: ${result.error.message}`);
    return 1;
  }
  return result.status ?? 1;
}

function clearStaleMatrixArtifacts() {
  for (const target of staleArtifactPaths) {
    rmSync(target, { recursive: true, force: true });
  }
}

for (const [label, args] of [
  ["research typecheck", ["run", "test:gemini-browser-adapter:typecheck"]],
  ["research unit tests", ["run", "test:gemini-browser-adapter:unit"]],
]) {
  const code = run(label, args);
  if (code !== 0) process.exit(code);
}

let e2eCode = 1;
let reportCode = 1;
try {
  clearStaleMatrixArtifacts();
  e2eCode = run("research Playwright e2e", ["run", "test:gemini-browser-adapter:e2e"]);
} finally {
  reportCode = run("research matrix report", ["run", "test:gemini-browser-adapter:report"]);
}

process.exit(e2eCode || reportCode);
