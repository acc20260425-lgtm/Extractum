import { lstatSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const bootstrapCommand = "Run: npm.cmd run bootstrap:testing";

function prerequisiteError(message) {
  return new Error(`${message}\n${bootstrapCommand}`);
}

export function inspectGeminiBrowserSidecar({
  repoRoot,
  targetTriple,
  platform,
  requestedTarget,
  lstatSyncImpl = lstatSync,
}) {
  if (requestedTarget && requestedTarget !== targetTriple) {
    throw prerequisiteError(
      `Gemini browser sidecar packaging is host-target only in v1. ` +
        `Requested ${requestedTarget}, host is ${targetTriple}.`,
    );
  }

  const extension = platform === "win32" ? ".exe" : "";
  const expectedPath = path.join(
    repoRoot,
    "src-tauri",
    "binaries",
    `gemini-browser-sidecar-${targetTriple}${extension}`,
  );

  let sidecar;
  try {
    sidecar = lstatSyncImpl(expectedPath);
  } catch {
    throw prerequisiteError(`Missing Gemini browser sidecar binary: ${expectedPath}`);
  }

  if (sidecar.isSymbolicLink()) {
    throw prerequisiteError(`Gemini browser sidecar binary must not be a symlink: ${expectedPath}`);
  }
  if (sidecar.isDirectory()) {
    throw prerequisiteError(`Gemini browser sidecar binary must not be a directory: ${expectedPath}`);
  }
  if (!sidecar.isFile()) {
    throw prerequisiteError(`Gemini browser sidecar binary must be a regular file: ${expectedPath}`);
  }
  if (sidecar.size === 0) {
    throw prerequisiteError(`Gemini browser sidecar binary must not be empty: ${expectedPath}`);
  }

  return {
    relativePath: path.relative(repoRoot, expectedPath).split(path.sep).join("/"),
    size: sidecar.size,
  };
}

function main() {
  const repoRoot = process.cwd();
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
  try {
    const { relativePath, size } = inspectGeminiBrowserSidecar({
      repoRoot,
      targetTriple,
      platform: process.platform,
      requestedTarget,
    });
    console.log(`Found ${relativePath} (${size} bytes)`);
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main();
}
