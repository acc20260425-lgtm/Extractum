import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

import { D_BLOB_ANCHORS } from "./git-state.mjs";
import { PROTOCOL } from "./protocol.mjs";
import {
  ProtocolError,
  runWindowsProcess,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

export const LOCK_PATH = "scripts/process-shell-diagnostic/protocol-lock.json";
export const FROZEN_INPUTS = Object.freeze([
  "docs/superpowers/specs/2026-07-18-process-shell-regression-diagnostic-design.md",
  "docs/superpowers/plans/2026-07-18-process-shell-regression-diagnostic.md",
  "docs/value-registry.md",
  "scripts/process-shell-diagnostic/protocol.mjs",
  "scripts/process-shell-diagnostic/protocol.test.ts",
  "scripts/process-shell-diagnostic/runtime.mjs",
  "scripts/process-shell-diagnostic/runtime.test.ts",
  "scripts/process-shell-diagnostic/git-state.mjs",
  "scripts/process-shell-diagnostic/git-state.test.ts",
  "scripts/process-shell-diagnostic/attempt.mjs",
  "scripts/process-shell-diagnostic/attempt.test.ts",
  "scripts/process-shell-diagnostic/coordinator.mjs",
  "scripts/process-shell-diagnostic/coordinator.test.ts",
  "scripts/process-shell-diagnostic/freeze.mjs",
  "scripts/process-shell-diagnostic/report.mjs",
  "scripts/process-shell-diagnostic/report.test.ts",
  "scripts/process-shell-diagnostic/states/B.patch",
  "scripts/process-shell-diagnostic/states/C.patch",
  "scripts/process-shell-diagnostic/states/E.patch",
]);

const A_TREE = "fd9711a041432ef420e7b09d56a46131a2a52a2a";
const D_TREE = "77e2d163ccc8bddf3ea051cb995909888cae9aba";

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function taskkillExe() {
  if (!process.env.SystemRoot) throw new ProtocolError("missing_system_root", "SystemRoot is required");
  return path.join(process.env.SystemRoot, "System32", "taskkill.exe");
}

async function removeOwnedTemp(directory) {
  const resolved = path.resolve(directory);
  const temporaryRoot = `${path.resolve(tmpdir())}${path.sep}`.toLowerCase();
  if (!resolved.toLowerCase().startsWith(temporaryRoot)) {
    throw new ProtocolError("unsafe_temp_cleanup", resolved);
  }
  await rm(resolved, { recursive: true, force: true });
}

async function createGit(repoRoot) {
  const artifactDir = await mkdtemp(path.join(tmpdir(), "extractum-process-freeze-"));
  let sequence = 0;
  async function bytes(args, env = process.env) {
    sequence += 1;
    const result = await runWindowsProcess({
      label: `git-${String(sequence).padStart(3, "0")}`,
      command: "git.exe",
      args,
      cwd: repoRoot,
      env,
      artifactDir,
      timeoutMs: PROTOCOL.commandTimeoutMs,
      taskkillExe: taskkillExe(),
    });
    if (result.classification !== "ok") {
      throw new ProtocolError("freeze_git_failed", args.join(" "), { result });
    }
    return readFile(result.stdoutPath);
  }
  return {
    bytes,
    text: async (args, env) => (await bytes(args, env)).toString("utf8").trim(),
    close: async () => removeOwnedTemp(artifactDir),
  };
}

export function assertProtocolWorktreeStatus(status, allowedUntrackedPaths = []) {
  const allowed = new Set(allowedUntrackedPaths.map((value) => value.replaceAll("\\", "/")));
  const unexpected = status.split(/\r?\n/).filter(Boolean).filter((line) => {
    if (!line.startsWith("?? ")) return true;
    const candidate = line.slice(3).replaceAll("\\", "/");
    return !allowed.has(candidate);
  });
  if (unexpected.length) {
    throw new ProtocolError("protocol_worktree_dirty", unexpected.join("\n"));
  }
}

async function assertInputsCommitted(git, allowedUntrackedPaths = []) {
  const status = await git.text(["status", "--porcelain=v1", "--untracked-files=all"]);
  assertProtocolWorktreeStatus(status, allowedUntrackedPaths);
  const tracked = (await git.text(["ls-files", "--", "scripts/process-shell-diagnostic"]))
    .split(/\r?\n/).filter(Boolean).sort();
  const expected = FROZEN_INPUTS.filter((entry) => entry.startsWith("scripts/")).sort();
  const allowed = new Set([...expected, LOCK_PATH]);
  const missing = expected.filter((entry) => !tracked.includes(entry));
  const extra = tracked.filter((entry) => !allowed.has(entry));
  if (missing.length || extra.length) {
    throw new ProtocolError("protocol_input_inventory_mismatch", "frozen script inventory differs", { missing, extra });
  }
}

async function inputRecord(git, filePath) {
  const blob = await git.text(["rev-parse", `HEAD:${filePath}`]);
  const blobBytes = await git.bytes(["cat-file", "blob", blob]);
  return {
    path: filePath,
    size: blobBytes.length,
    sha256: sha256(blobBytes),
    gitBlob: blob,
  };
}

async function patchedState(repoRoot, git, name) {
  const indexRoot = await mkdtemp(path.join(tmpdir(), `extractum-process-${name}-index-`));
  const indexPath = path.join(indexRoot, "index");
  const environment = { ...process.env, GIT_INDEX_FILE: indexPath };
  const patch = `scripts/process-shell-diagnostic/states/${name}.patch`;
  try {
    const patchBlob = await git.text(["rev-parse", `HEAD:${patch}`]);
    const patchBytes = await git.bytes(["cat-file", "blob", patchBlob]);
    const canonicalPatch = path.join(indexRoot, `${name}.patch`);
    await writeFile(canonicalPatch, patchBytes, { flag: "wx" });
    await git.text(["read-tree", PROTOCOL.baselineCommit], environment);
    await git.text([
      "apply", "--cached", "--whitespace=nowarn", "--",
      canonicalPatch,
    ], environment);
    const rootTree = await git.text(["write-tree"], environment);
    const srcTauriTree = await git.text(["rev-parse", `${rootTree}:src-tauri`], environment);
    const changedPaths = (await git.text([
      "diff", "--cached", "--name-status", "--find-renames=50%",
      PROTOCOL.baselineCommit, "--", "src-tauri",
    ], environment)).split(/\r?\n/).filter(Boolean);
    return {
      source: patch,
      base: "A",
      patch,
      patchSha256: sha256(patchBytes),
      srcTauriTree,
      changedPaths,
    };
  } finally {
    await removeOwnedTemp(indexRoot);
  }
}

async function stateRecords(repoRoot, git) {
  const aTree = await git.text(["rev-parse", `${PROTOCOL.baselineCommit}:src-tauri`]);
  const dTree = await git.text(["rev-parse", `${PROTOCOL.candidateCommit}:src-tauri`]);
  const parentTree = await git.text(["rev-parse", `${PROTOCOL.candidateCommit}^:src-tauri`]);
  if (aTree !== A_TREE || dTree !== D_TREE || parentTree !== A_TREE) {
    throw new ProtocolError("historical_tree_mismatch", "A, D, or D parent differs", { aTree, dTree, parentTree });
  }
  const dChangedPaths = (await git.text([
    "diff", "--name-status", "--find-renames=50%",
    PROTOCOL.baselineCommit, PROTOCOL.candidateCommit, "--", "src-tauri",
  ])).split(/\r?\n/).filter(Boolean);
  const states = {
    A: { source: PROTOCOL.baselineCommit, srcTauriTree: aTree, changedPaths: [] },
    B: await patchedState(repoRoot, git, "B"),
    C: await patchedState(repoRoot, git, "C"),
    D: {
      source: PROTOCOL.candidateCommit,
      srcTauriTree: dTree,
      changedPaths: dChangedPaths,
      blobs: { ...D_BLOB_ANCHORS },
      absentPaths: [
        "src-tauri/src/child_process.rs",
        "src-tauri/src/external_process.rs",
        "src-tauri/src/process_tree.rs",
      ],
    },
    E: await patchedState(repoRoot, git, "E"),
  };
  if (new Set(Object.values(states).map((state) => state.srcTauriTree)).size !== 5) {
    throw new ProtocolError("state_tree_collision", "A/B/C/D/E must have five distinct trees", { states });
  }
  return states;
}

export async function buildProtocolLock({ repoRoot, allowedUntrackedPaths = [] }) {
  const git = await createGit(repoRoot);
  try {
    await assertInputsCommitted(git, allowedUntrackedPaths);
    const frozenInputs = [];
    for (const filePath of [...FROZEN_INPUTS].sort()) {
      frozenInputs.push(await inputRecord(git, filePath));
    }
    return {
      schemaVersion: 1,
      protocolVersion: PROTOCOL.version,
      baselineCommit: PROTOCOL.baselineCommit,
      candidateCommit: PROTOCOL.candidateCommit,
      generatedBy: "scripts/process-shell-diagnostic/freeze.mjs",
      frozenInputs,
      states: await stateRecords(repoRoot, git),
    };
  } finally {
    await git.close();
  }
}

export async function verifyFrozenProtocol({ repoRoot, allowedUntrackedPaths = [] }) {
  const actual = await buildProtocolLock({ repoRoot, allowedUntrackedPaths });
  const git = await createGit(repoRoot);
  try {
    const protocolCommit = await git.text(["rev-parse", "HEAD"]);
    const lockBlob = await git.text(["rev-parse", `${protocolCommit}:${LOCK_PATH}`]);
    const recordedBytes = await git.bytes(["cat-file", "blob", lockBlob]);
    const recorded = JSON.parse(recordedBytes.toString("utf8"));
    if (JSON.stringify(recorded) !== JSON.stringify(actual)) {
      throw new ProtocolError("protocol_lock_mismatch", "protocol-lock.json differs from canonical frozen Git blobs");
    }
    return {
      protocolCommit,
      lockPath: LOCK_PATH,
      lockBlob,
      lockSha256: sha256(recordedBytes),
      protocolVersion: recorded.protocolVersion,
      protocolLock: recorded,
    };
  } finally {
    await git.close();
  }
}

function option(name) {
  const index = process.argv.indexOf(name);
  if (index < 0 || !process.argv[index + 1]) throw new Error(`missing ${name}`);
  return process.argv[index + 1];
}

async function main() {
  const action = process.argv[2];
  const repoRoot = path.resolve(option("--repo-root"));
  if (action === "generate") {
    await writeAtomicJsonExclusive(
      path.join(repoRoot, ...LOCK_PATH.split("/")),
      await buildProtocolLock({ repoRoot }),
    );
    process.stdout.write(`${LOCK_PATH}\n`);
    return;
  }
  if (action === "verify") {
    const { protocolLock: _protocolLock, ...pin } = await verifyFrozenProtocol({ repoRoot });
    process.stdout.write(`${JSON.stringify(pin, null, 2)}\n`);
    return;
  }
  throw new Error(`expected generate or verify, got ${action ?? "missing"}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  await main();
}
