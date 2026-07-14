import { spawnSync } from "node:child_process";
import { existsSync, realpathSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

const DEFAULT_EXCLUDES = ["research/gemini_browser_adapter/tests/**"];

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

function runVitest() {
  const realCwd = realpathSync.native(process.cwd());
  process.chdir(realCwd);

  const defaultExcludeArgs = DEFAULT_EXCLUDES.flatMap((glob) => ["--exclude", glob]);
  const vitestCli = path.join(realCwd, "node_modules", "vitest", "vitest.mjs");
  const args = normalizeRelatedFileArgs(process.argv.slice(2), realCwd);
  const result = spawnSync(process.execPath, [vitestCli, ...args, ...defaultExcludeArgs], {
    cwd: realCwd,
    env: process.env,
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }

  process.exit(result.status ?? 1);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runVitest();
}
