import { spawnSync } from "node:child_process";
import { realpathSync } from "node:fs";
import path from "node:path";
import process from "node:process";

const realCwd = realpathSync.native(process.cwd());
process.chdir(realCwd);

const DEFAULT_EXCLUDES = ["research/gemini_browser_adapter/tests/**"];
const defaultExcludeArgs = DEFAULT_EXCLUDES.flatMap((glob) => ["--exclude", glob]);
const vitestCli = path.join(realCwd, "node_modules", "vitest", "vitest.mjs");
const result = spawnSync(process.execPath, [vitestCli, ...process.argv.slice(2), ...defaultExcludeArgs], {
  cwd: realCwd,
  env: process.env,
  stdio: "inherit",
});

if (result.error) {
  throw result.error;
}

process.exit(result.status ?? 1);
