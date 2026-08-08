import { spawnSync as defaultSpawnSync } from "node:child_process";
import { existsSync, realpathSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

const DEFAULT_EXCLUDES = [
  ".worktrees/**",
  "research/gemini_browser_adapter/tests/**",
];

/**
 * @param {string[]} args
 * @param {string} [cwd]
 * @returns {string[]}
 */
export function normalizeRelatedFileArgs(args, cwd = process.cwd()) {
  if (args[0] !== "related") {
    return [...args];
  }

  return args.map((arg, index) => {
    if (index === 0 || arg.startsWith("-")) {
      return arg;
    }

    const normalized = arg.replaceAll("\\", "/");
    return existsSync(path.resolve(cwd, normalized)) ? normalized : arg;
  });
}

/** @param {unknown} output */
export function normalizeVitestListOutput(output) {
  return String(output).replace(/^\[[^\]]+\]\s+/gm, "");
}

export function runVitest(options) {
  const {
    argv = process.argv.slice(2),
    cwd = process.cwd(),
    spawnSync = defaultSpawnSync,
  } = options ?? {};
  const cli = options === undefined;
  const realCwd = realpathSync.native(cwd);
  if (cli) process.chdir(realCwd);
  const defaultExcludeArgs = DEFAULT_EXCLUDES.flatMap((glob) => ["--exclude", glob]);
  const vitestCli = path.join(realCwd, "node_modules", "vitest", "vitest.mjs");
  const args = normalizeRelatedFileArgs(argv, realCwd);
  const filesOnlyList = args[0] === "list" && args.includes("--filesOnly");
  const childArgs = [vitestCli, "--config", "vitest.config.ts", ...args, ...defaultExcludeArgs];
  const result = spawnSync(process.execPath, childArgs, {
    cwd: realCwd,
    env: process.env,
    stdio: filesOnlyList ? "pipe" : "inherit",
  });

  if (result.error) {
    throw result.error;
  }
  const observation = {
    status: result.status ?? 1,
    args: childArgs,
    filesOnlyList,
    stdout: result.stdout,
    stderr: result.stderr,
  };
  if (cli) {
    if (observation.filesOnlyList) {
      process.stdout.write(normalizeVitestListOutput(observation.stdout));
      process.stderr.write(observation.stderr);
    }
    process.exit(observation.status);
  }
  return observation;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runVitest();
}
