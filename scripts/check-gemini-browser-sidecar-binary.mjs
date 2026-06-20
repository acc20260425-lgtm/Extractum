import { existsSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = process.cwd();
const extension = process.platform === "win32" ? ".exe" : "";
const result = spawnSync("rustc", ["--print", "host-tuple"], {
  cwd: repoRoot,
  encoding: "utf8",
  shell: process.platform === "win32",
});

if (result.status !== 0) {
  console.error(result.stderr);
  process.exit(result.status ?? 1);
}

const targetTriple = result.stdout.trim();
const requestedTarget =
  process.env.GEMINI_BROWSER_SIDECAR_TARGET ?? process.env.CARGO_BUILD_TARGET ?? "";
if (requestedTarget && requestedTarget !== targetTriple) {
  console.error(
    `Gemini browser sidecar packaging is host-target only in v1. ` +
      `Requested ${requestedTarget}, host is ${targetTriple}.`,
  );
  process.exit(1);
}
const expectedPath = path.join(
  repoRoot,
  "src-tauri",
  "binaries",
  `gemini-browser-sidecar-${targetTriple}${extension}`,
);

if (!existsSync(expectedPath)) {
  console.error(`Missing Gemini browser sidecar binary: ${expectedPath}`);
  console.error("Run: npm.cmd run build:gemini-browser-sidecar");
  process.exit(1);
}

console.log(`Found ${path.relative(repoRoot, expectedPath)}`);
