import { existsSync, mkdirSync, renameSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = process.cwd();
const sidecarEntry = path.join(repoRoot, "sidecars", "gemini-browser", "src", "index.ts");
const sidecarDist = path.join(repoRoot, "sidecars", "gemini-browser", "dist", "index.js");
const binariesDir = path.join(repoRoot, "src-tauri", "binaries");
const packageWorkDir = path.join(repoRoot, "artifacts", "gemini-browser-sidecar-package");
const bundleOutput = path.join(packageWorkDir, "index.cjs");
const pkgConfigPath = path.join(packageWorkDir, "pkg.config.json");
const extension = process.platform === "win32" ? ".exe" : "";
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
const npxCommand = process.platform === "win32" ? "npx.cmd" : "npx";

function run(label, command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    throw new Error(`${label} failed with exit code ${result.status}`);
  }
}

function output(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed: ${result.stderr}`);
  }
  return result.stdout.trim();
}

function pkgTarget() {
  const platform =
    process.platform === "win32"
      ? "win"
      : process.platform === "darwin"
        ? "macos"
        : process.platform === "linux"
          ? "linux"
          : null;
  const arch =
    process.arch === "x64"
      ? "x64"
      : process.arch === "arm64"
        ? "arm64"
        : null;

  if (!platform || !arch) {
    throw new Error(`Unsupported pkg target platform: ${process.platform}/${process.arch}`);
  }

  return `node18-${platform}-${arch}`;
}

run("sidecar TypeScript build", npmCommand, ["run", "test:gemini-browser-sidecar:build"]);

if (!existsSync(sidecarDist)) {
  throw new Error(`Missing sidecar dist entry: ${sidecarDist}`);
}

const targetTriple = output("rustc", ["--print", "host-tuple"]);
if (!targetTriple) {
  throw new Error("rustc did not return a host tuple");
}
const requestedTarget =
  process.env.GEMINI_BROWSER_SIDECAR_TARGET ?? process.env.CARGO_BUILD_TARGET ?? "";
if (requestedTarget && requestedTarget !== targetTriple) {
  throw new Error(
    `Gemini browser sidecar packaging is host-target only in v1. ` +
      `Requested ${requestedTarget}, host is ${targetTriple}.`,
  );
}

mkdirSync(binariesDir, { recursive: true });
mkdirSync(packageWorkDir, { recursive: true });

const rawOutput = path.join(binariesDir, `gemini-browser-sidecar${extension}`);
const tauriOutput = path.join(
  binariesDir,
  `gemini-browser-sidecar-${targetTriple}${extension}`,
);
const browsersJsonAsset = path.relative(
  packageWorkDir,
  path.join(repoRoot, "node_modules", "playwright-core", "browsers.json"),
);

rmSync(rawOutput, { force: true });
rmSync(tauriOutput, { force: true });
rmSync(bundleOutput, { force: true });
writeFileSync(
  pkgConfigPath,
  JSON.stringify(
    {
      pkg: {
        assets: [browsersJsonAsset.replace(/\\/g, "/")],
      },
    },
    null,
    2,
  ),
);

run("sidecar CommonJS bundle", npxCommand, [
  "esbuild",
  sidecarEntry,
  "--bundle",
  "--platform=node",
  "--format=cjs",
  "--packages=external",
  `--outfile=${bundleOutput}`,
]);

run("Node sidecar binary packaging", npxCommand, [
  "pkg",
  bundleOutput,
  "--config",
  pkgConfigPath,
  "--targets",
  pkgTarget(),
  "--no-bytecode",
  "--public",
  "--public-packages",
  "*",
  "--output",
  rawOutput,
]);

if (!existsSync(rawOutput)) {
  throw new Error(`Sidecar packager did not create ${rawOutput}`);
}

renameSync(rawOutput, tauriOutput);
console.log(`Wrote ${path.relative(repoRoot, tauriOutput)}`);
