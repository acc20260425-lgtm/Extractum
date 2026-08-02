import { appendFile as defaultAppendFile, mkdir as defaultMkdir } from "node:fs/promises";
import path from "node:path";
import { spawnSync as defaultSpawnSync } from "node:child_process";

const FIELD_NAMES = ["command", "startedAt", "duration", "exitCode", "commit"];

function quoteCommandPart(value) {
  const text = String(value);
  if (text.length === 0 || /[\s"]/u.test(text)) {
    return `"${text.replace(/(["\\])/gu, "\\$1")}"`;
  }
  return text;
}

export function formatCommand(command, args) {
  return [command, ...args].map(quoteCommandPart).join(" ");
}

export function createTimingRow(input) {
  const row = {
    command: input.command,
    startedAt: input.startedAt,
    duration: Math.max(0, Math.round(input.duration)),
    exitCode: input.exitCode,
    commit: input.commit,
  };
  if (typeof row.command !== "string" || row.command.length === 0) throw new TypeError("command");
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(row.startedAt)) throw new TypeError("startedAt");
  if (!Number.isInteger(row.duration) || !Number.isInteger(row.exitCode)) throw new TypeError("duration/exitCode");
  if (!/^[0-9a-f]{40,64}$/i.test(row.commit)) throw new TypeError("commit");
  if (Object.keys(row).join("\0") !== FIELD_NAMES.join("\0")) throw new TypeError("timing fields");
  return row;
}

export function readHeadCommit(repoRoot, spawnSyncImpl = defaultSpawnSync) {
  const result = spawnSyncImpl("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
    shell: false,
  });
  if (result.error) throw result.error;
  const commit = result.stdout?.trim();
  if (result.status !== 0 || !/^[0-9a-f]{40,64}$/i.test(commit)) {
    throw new Error("Unable to read HEAD commit");
  }
  return commit;
}

export async function appendTimingRow(row, options = {}) {
  const validatedRow = createTimingRow(row);
  const repoRoot = options.repoRoot ?? process.cwd();
  const outputPath = path.join(repoRoot, "artifacts", "testing", "timings.jsonl");
  const mkdir = options.mkdir ?? defaultMkdir;
  const appendFile = options.appendFile ?? defaultAppendFile;

  await mkdir(path.dirname(outputPath), { recursive: true });
  await appendFile(outputPath, `${JSON.stringify(validatedRow)}\n`, "utf8");
  return true;
}

export async function recordTimingBestEffort(row, options = {}) {
  try {
    await appendTimingRow(row, options);
    return true;
  } catch (error) {
    const warn = options.warn ?? console.warn;
    const message = error instanceof Error ? error.message : String(error);
    warn(`Timing log warning: ${message}`);
    return false;
  }
}
