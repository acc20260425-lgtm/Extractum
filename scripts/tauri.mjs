import { spawn } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

export function buildTauriArgs(args) {
  const delimiter = args.indexOf("--");
  const commandArgs = args.slice(0, delimiter < 0 ? args.length : delimiter);
  const hasConfig = commandArgs.some(
    (arg) => arg === "--config" || arg === "-c" || arg.startsWith("--config=") || arg.startsWith("-c="),
  );

  if (args[0] !== "dev" || hasConfig) return [...args];

  return ["dev", "--config", "src-tauri/tauri.mcp.conf.json", ...args.slice(1)];
}

function runTauri() {
  const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const tauriCli = path.join(repoRoot, "node_modules", "@tauri-apps", "cli", "tauri.js");
  const child = spawn(process.execPath, [tauriCli, ...buildTauriArgs(process.argv.slice(2))], {
    cwd: repoRoot,
    env: process.env,
    stdio: "inherit",
  });

  child.on("error", (error) => {
    throw error;
  });

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }

    process.exit(code ?? 1);
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runTauri();
}
