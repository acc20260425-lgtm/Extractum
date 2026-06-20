import { spawn } from "node:child_process";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = process.cwd();
const mode = process.argv.includes("--binary") ? "binary" : "node";
const playwrightSmoke = process.argv.includes("--playwright");

function hostTuple() {
  const result = spawnSync("rustc", ["--print", "host-tuple"], {
    cwd: repoRoot,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    throw new Error(result.stderr);
  }
  return result.stdout.trim();
}

const extension = process.platform === "win32" ? ".exe" : "";
const command =
  mode === "binary"
    ? path.join(
        repoRoot,
        "src-tauri",
        "binaries",
        `gemini-browser-sidecar-${hostTuple()}${extension}`,
      )
    : process.execPath;
const args =
  mode === "binary"
    ? []
    : [path.join(repoRoot, "sidecars", "gemini-browser", "dist", "index.js")];

if (playwrightSmoke) {
  const profileDir = path.join(repoRoot, "artifacts", `gemini-browser-playwright-smoke-${mode}`);
  args.push("--playwright-smoke", `--profile-dir=${profileDir}`);
}

const child = spawn(command, args, {
  cwd: repoRoot,
  stdio: ["pipe", "pipe", "pipe"],
});

if (playwrightSmoke) {
  let stdout = "";
  let stderr = "";
  const timeout = setTimeout(() => {
    child.kill();
    console.error("Timed out waiting for Playwright smoke response");
    process.exit(1);
  }, 15000);

  child.stdout.on("data", (chunk) => {
    stdout += chunk.toString();
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });
  child.on("exit", (code) => {
    clearTimeout(timeout);
    if (code !== 0) {
      console.error(stderr);
      process.exit(code ?? 1);
    }
    const line = stdout.split(/\r?\n/).find((entry) => entry.trim().length > 0);
    const parsed = line ? JSON.parse(line) : null;
    if (!parsed?.ok || parsed.title !== "Gemini Sidecar Smoke") {
      console.error(`Unexpected Playwright smoke output: ${stdout}`);
      process.exit(1);
    }
    console.log(line);
  });
} else {
  const request = {
    id: "smoke-1",
    command: {
      type: "status",
      browser_profile_dir: path.join(repoRoot, "artifacts", "gemini-browser-smoke-profile"),
    },
  };

  let stdout = "";
  let stderr = "";
  const timeout = setTimeout(() => {
    child.kill();
    console.error("Timed out waiting for sidecar status response");
    process.exit(1);
  }, 5000);

  child.stdout.on("data", (chunk) => {
    stdout += chunk.toString();
    const line = stdout.split(/\r?\n/).find((entry) => entry.trim().length > 0);
    if (!line) return;
    clearTimeout(timeout);
    child.kill();
    const parsed = JSON.parse(line);
    if (parsed.id !== "smoke-1") {
      console.error(`Unexpected response id: ${line}`);
      process.exit(1);
    }
    if (parsed.response?.type !== "status") {
      console.error(`Unexpected response type: ${line}`);
      process.exit(1);
    }
    console.log(line);
  });

  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });

  child.on("exit", (code) => {
    if (!stdout.trim()) {
      clearTimeout(timeout);
      console.error(stderr);
      process.exit(code ?? 1);
    }
  });

  child.stdin.write(`${JSON.stringify(request)}\n`);
}
