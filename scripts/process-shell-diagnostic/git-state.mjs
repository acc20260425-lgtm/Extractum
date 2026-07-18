import { lstat, mkdir, open, readFile, realpath } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

import { PROTOCOL } from "./protocol.mjs";
import {
  assertCommandOk,
  ProtocolError,
  runWindowsProcess,
  sha256File,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

export const STATE_TREE_ANCHORS = Object.freeze({
  A: "fd9711a041432ef420e7b09d56a46131a2a52a2a",
  D: "77e2d163ccc8bddf3ea051cb995909888cae9aba",
});

export const D_BLOB_ANCHORS = Object.freeze({
  "src-tauri/Cargo.lock": "6368e32cd3a3853d4a7114ce256258e834bafdd4",
  "src-tauri/Cargo.toml": "c2037473a1257dd33a8e5b5fe81905e77dad084a",
  "src-tauri/crates/extractum-process/Cargo.toml": "3e078647dc293d95f401e15b8842776fae003ddb",
  "src-tauri/crates/extractum-process/src/child_process.rs": "9599017ed2ad826bc73f8e72f084042eacd8b58a",
  "src-tauri/crates/extractum-process/src/external_process.rs": "3cf7f073923b513381df09b7443090a4a41adc11",
  "src-tauri/crates/extractum-process/src/lib.rs": "4f7819ef7d2773b735b5edc61e162e4e034efb66",
  "src-tauri/crates/extractum-process/src/process_tree.rs": "365283e9f8accf4db91feca73bd8437db3b08c50",
  "src-tauri/src/lib.rs": "d84b653870eda9378c0d490894801850a97db68d",
});

export const DIAGNOSTIC_SIDECAR_PLACEHOLDER = Object.freeze({
  relativePath: "src-tauri/binaries/gemini-browser-sidecar-x86_64-pc-windows-msvc.exe",
  size: 0,
  sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
});

const WINDOWS_TABLE = "target.'cfg(windows)'.dependencies";
const PROCESS_PATHS = [
  "src-tauri/crates/extractum-process/Cargo.toml",
  "src-tauri/crates/extractum-process/src/lib.rs",
];

export function aRestoreArgs() {
  return [
    "restore",
    `--source=${PROTOCOL.baselineCommit}`,
    "--staged",
    "--worktree",
    "--",
    "src-tauri",
  ];
}

export function dRestoreArgs() {
  return [
    "restore",
    `--source=${PROTOCOL.candidateCommit}`,
    "--staged",
    "--worktree",
    "--",
    "src-tauri",
  ];
}

function normalizedState(state) {
  if (/^A(?:\d+|-final)?$/.test(state)) return "A";
  if (["B", "C", "D", "E"].includes(state)) return state;
  throw new ProtocolError("unknown_state", state);
}

export function parseTomlSections(text) {
  const sections = new Map([["", new Map()]]);
  let current = "";
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (line === "" || line.startsWith("#")) continue;
    const header = line.match(/^\[([^\]]+)\]$/);
    if (header) {
      current = header[1];
      if (!sections.has(current)) sections.set(current, new Map());
      continue;
    }
    const equals = line.indexOf("=");
    if (equals < 1) throw new Error(`unsupported TOML line: ${rawLine}`);
    const key = line.slice(0, equals).trim();
    const value = line.slice(equals + 1).trim();
    if (sections.get(current).has(key)) throw new Error(`duplicate TOML key ${current}.${key}`);
    sections.get(current).set(key, value);
  }
  return sections;
}

function table(sections, name) {
  return sections.get(name) ?? new Map();
}

function exactKeys(entries, expected, label) {
  const actual = [...entries.keys()].sort();
  const wanted = [...expected].sort();
  if (JSON.stringify(actual) !== JSON.stringify(wanted)) {
    throw new Error(`unexpected ${label} keys: ${actual.join(",")}`);
  }
}

function requireValue(entries, key, value, label) {
  if (entries.get(key) !== value) {
    throw new Error(`${label}.${key} must be ${value}, got ${entries.get(key) ?? "missing"}`);
  }
}

function lockPackages(text) {
  const normalized = text.replaceAll("\r\n", "\n");
  const firstPackage = normalized.indexOf("[[package]]");
  if (firstPackage < 0) throw new Error("Cargo.lock has no package records");
  const preamble = normalized.slice(0, firstPackage).trim();
  const blocks = normalized.slice(firstPackage).split(/(?=^\[\[package\]\]$)/m).filter(Boolean);
  const records = blocks.map((block) => {
    const name = block.match(/^name = "([^"]+)"$/m)?.[1];
    const version = block.match(/^version = "([^"]+)"$/m)?.[1];
    const source = block.match(/^source = "([^"]+)"$/m)?.[1] ?? "workspace";
    if (!name || !version) throw new Error("Cargo.lock package lacks name/version");
    const dependencyBody = block.match(/^dependencies = \[\n([\s\S]*?)^\]$/m)?.[1] ?? "";
    const dependencies = [...dependencyBody.matchAll(/"([^"]+)"/g)]
      .map((match) => match[1].split(" ")[0])
      .sort();
    return { key: `${name}\0${version}\0${source}`, name, version, source, block: block.trim(), dependencies };
  });
  return { preamble, records };
}

function onlyPackage(records, name) {
  const matches = records.filter((record) => record.name === name);
  if (matches.length !== 1) throw new Error(`expected one ${name} lock package, got ${matches.length}`);
  return matches[0];
}

function sameValues(actual, expected, label) {
  if (JSON.stringify([...actual].sort()) !== JSON.stringify([...expected].sort())) {
    throw new Error(`${label}: ${JSON.stringify(actual)} != ${JSON.stringify(expected)}`);
  }
}

export function validateLockDelta({ state, baselineLock, stateLock }) {
  const baseline = lockPackages(baselineLock);
  const candidate = lockPackages(stateLock);
  if (baseline.preamble !== candidate.preamble) throw new Error("Cargo.lock preamble changed");
  const excluded = new Set(["extractum", "extractum-process"]);
  const baselineThirdParty = new Map(
    baseline.records.filter((record) => !excluded.has(record.name)).map((record) => [record.key, record.block]),
  );
  const candidateThirdParty = new Map(
    candidate.records.filter((record) => !excluded.has(record.name)).map((record) => [record.key, record.block]),
  );
  if (JSON.stringify([...baselineThirdParty].sort()) !== JSON.stringify([...candidateThirdParty].sort())) {
    throw new Error("third-party lock package changed");
  }
  if (baseline.records.some((record) => record.name === "extractum-process")) {
    throw new Error("baseline unexpectedly contains extractum-process");
  }
  const baselineRoot = onlyPackage(baseline.records, "extractum");
  const candidateRoot = onlyPackage(candidate.records, "extractum");
  const processPackage = onlyPackage(candidate.records, "extractum-process");
  const expectedRoot = state === "B"
    ? baselineRoot.dependencies
    : [...baselineRoot.dependencies, "extractum-process"];
  sameValues(candidateRoot.dependencies, expectedRoot, `${state} root lock dependencies`);
  const expectedProcess = state === "E" ? ["anyhow", "parking_lot", "tokio", "windows-sys"] : [];
  sameValues(processPackage.dependencies, expectedProcess, `${state} process lock dependencies`);
}

export function validateStateManifests({
  state,
  rootManifest,
  processManifest,
  processPaths,
}) {
  if (!["B", "C", "E"].includes(state)) return;
  const rootSections = parseTomlSections(rootManifest);
  const processSections = parseTomlSections(processManifest);
  requireValue(
    table(rootSections, "workspace"),
    "members",
    '[".", "crates/extractum-core", "crates/extractum-process"]',
    "workspace",
  );
  if (JSON.stringify([...processPaths].sort()) !== JSON.stringify(PROCESS_PATHS)) {
    throw new Error(`unexpected process crate paths: ${processPaths.join(",")}`);
  }

  const packageTable = table(processSections, "package");
  exactKeys(packageTable, ["name", "version.workspace", "edition.workspace", "publish"], "package");
  requireValue(packageTable, "name", '"extractum-process"', "package");
  requireValue(packageTable, "version.workspace", "true", "package");
  requireValue(packageTable, "edition.workspace", "true", "package");
  requireValue(packageTable, "publish", "false", "package");

  const appDependencies = table(rootSections, "dependencies");
  const processDependencies = table(processSections, "dependencies");
  const processTargetDependencies = table(processSections, WINDOWS_TABLE);
  const processDevDependencies = table(processSections, "dev-dependencies");

  if (state === "B") {
    if (appDependencies.has("extractum-process")) throw new Error("B must not contain the app edge");
    exactKeys(processDependencies, [], "dependency");
    exactKeys(processTargetDependencies, [], "target dependency");
    exactKeys(processDevDependencies, [], "dev dependency");
    return;
  }

  if (state === "C") {
    if (appDependencies.get("extractum-process") !== '{ path = "crates/extractum-process" }') {
      throw new Error("C must contain exactly the app path edge");
    }
    exactKeys(processDependencies, [], "dependency");
    exactKeys(processTargetDependencies, [], "target dependency");
    exactKeys(processDevDependencies, [], "dev dependency");
    return;
  }

  requireValue(
    appDependencies,
    "extractum-process",
    '{ path = "crates/extractum-process" }',
    "dependencies",
  );

  const workspaceDependencies = table(rootSections, "workspace.dependencies");
  requireValue(workspaceDependencies, "anyhow", '"1.0"', "workspace.dependencies");
  requireValue(workspaceDependencies, "parking_lot", '"0.12"', "workspace.dependencies");
  requireValue(workspaceDependencies, "tokio", '{ version = "1", features = ["full"] }', "workspace.dependencies");
  requireValue(
    workspaceDependencies,
    "windows-sys",
    '{ version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }',
    "workspace.dependencies",
  );
  for (const key of ["anyhow", "parking_lot", "tokio"]) {
    requireValue(appDependencies, key, "{ workspace = true }", "dependencies");
  }
  requireValue(table(rootSections, WINDOWS_TABLE), "windows-sys", "{ workspace = true }", WINDOWS_TABLE);
  exactKeys(processDependencies, ["anyhow.workspace", "parking_lot.workspace", "tokio.workspace"], "dependency");
  for (const key of ["anyhow.workspace", "parking_lot.workspace", "tokio.workspace"]) {
    requireValue(processDependencies, key, "true", "dependencies");
  }
  exactKeys(processTargetDependencies, ["windows-sys.workspace"], "target dependency");
  requireValue(processTargetDependencies, "windows-sys.workspace", "true", WINDOWS_TABLE);
  exactKeys(processDevDependencies, ["tokio"], "dev dependency");
  requireValue(
    processDevDependencies,
    "tokio",
    '{ workspace = true, features = ["test-util"] }',
    "dev-dependencies",
  );
}

function normalizedPath(value) {
  return path.resolve(value).replaceAll("/", "\\").toLowerCase();
}

export async function ensureDiagnosticSidecarPlaceholder({ worktree }) {
  try {
    const resolvedWorktree = await realpath(worktree);
    const placeholderPath = path.join(
      resolvedWorktree,
      ...DIAGNOSTIC_SIDECAR_PLACEHOLDER.relativePath.split("/"),
    );
    const placeholderParent = path.dirname(placeholderPath);
    await mkdir(placeholderParent, { recursive: true });
    const resolvedParent = await realpath(placeholderParent);
    if (normalizedPath(resolvedParent) !== normalizedPath(placeholderParent)) {
      throw new ProtocolError(
        "environment_sidecar_placeholder_invalid",
        "sidecar placeholder parent resolves through a link",
        { placeholderParent, resolvedParent },
      );
    }
    try {
      const handle = await open(placeholderPath, "wx");
      await handle.close();
    } catch (error) {
      if (error?.code !== "EEXIST") throw error;
    }
    const info = await lstat(placeholderPath);
    if (!info.isFile() || info.isSymbolicLink() || info.size !== DIAGNOSTIC_SIDECAR_PLACEHOLDER.size) {
      throw new ProtocolError(
        "environment_sidecar_placeholder_invalid",
        "sidecar placeholder must be an empty regular file",
        { placeholderPath, size: info.size, symbolicLink: info.isSymbolicLink() },
      );
    }
    const resolvedPlaceholder = await realpath(placeholderPath);
    if (normalizedPath(resolvedPlaceholder) !== normalizedPath(placeholderPath)) {
      throw new ProtocolError(
        "environment_sidecar_placeholder_invalid",
        "sidecar placeholder resolves through a link",
        { placeholderPath, resolvedPlaceholder },
      );
    }
    const actualSha256 = await sha256File(placeholderPath);
    if (actualSha256 !== DIAGNOSTIC_SIDECAR_PLACEHOLDER.sha256) {
      throw new ProtocolError(
        "environment_sidecar_placeholder_invalid",
        "sidecar placeholder hash mismatch",
        { placeholderPath, actualSha256 },
      );
    }
    return {
      ...DIAGNOSTIC_SIDECAR_PLACEHOLDER,
      absolutePath: placeholderPath,
    };
  } catch (error) {
    if (error instanceof ProtocolError) throw error;
    throw new ProtocolError(
      "environment_sidecar_placeholder_invalid",
      error?.message ?? String(error),
      { worktree, code: error?.code ?? null },
    );
  }
}

async function rejectReparsePoint(candidate) {
  try {
    const information = await lstat(candidate);
    if (information.isSymbolicLink()) {
      throw new ProtocolError("target_not_isolated", `reparse point is forbidden: ${candidate}`);
    }
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
}

export async function verifyTargetIsolation({ metadata, worktree, mainRoot }) {
  const targetOverride = Object.keys(process.env).find(
    (key) => key.toUpperCase() === "CARGO_TARGET_DIR",
  );
  if (targetOverride !== undefined) {
    throw new ProtocolError("target_not_isolated", `CARGO_TARGET_DIR is set as ${targetOverride}`);
  }
  const workspaceRoot = metadata.workspace_root;
  const targetDirectory = metadata.target_directory;
  if (typeof workspaceRoot !== "string" || typeof targetDirectory !== "string") {
    throw new ProtocolError("target_metadata_missing", "cargo metadata omitted workspace_root or target_directory");
  }
  const expectedWorkspace = path.join(worktree, "src-tauri");
  const expected = path.join(worktree, "src-tauri", "target");
  const mainTarget = path.join(mainRoot, "src-tauri", "target");
  if (normalizedPath(workspaceRoot) !== normalizedPath(expectedWorkspace)) {
    throw new ProtocolError("target_not_isolated", `expected workspace ${expectedWorkspace}, got ${workspaceRoot}`);
  }
  if (normalizedPath(targetDirectory) !== normalizedPath(expected)) {
    throw new ProtocolError("target_not_isolated", `expected ${expected}, got ${targetDirectory}`);
  }
  if (normalizedPath(targetDirectory) === normalizedPath(mainTarget)) {
    throw new ProtocolError("target_not_isolated", "attempt target equals main target");
  }
  await rejectReparsePoint(path.join(worktree, "src-tauri"));
  await rejectReparsePoint(expected);
  try {
    const [resolvedTarget, resolvedMain] = await Promise.all([realpath(expected), realpath(mainTarget)]);
    if (normalizedPath(resolvedTarget) === normalizedPath(resolvedMain)) {
      throw new ProtocolError("target_not_isolated", "attempt target resolves to main target");
    }
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
}

function taskkillExe() {
  if (!process.env.SystemRoot) throw new ProtocolError("missing_system_root", "SystemRoot is required");
  return path.join(process.env.SystemRoot, "System32", "taskkill.exe");
}

async function git({ args, label, worktree, artifactDir, rawOutput = false }) {
  const result = await runWindowsProcess({
    label,
    command: "git.exe",
    args,
    cwd: worktree,
    env: process.env,
    artifactDir,
    timeoutMs: PROTOCOL.commandTimeoutMs,
    taskkillExe: taskkillExe(),
  });
  if (result.classification !== "ok") {
    throw new ProtocolError("git_command_failed", args.join(" "), { result });
  }
  if (rawOutput) return result.stdoutPath;
  return (await readFile(result.stdoutPath, "utf8")).trim().replaceAll("\r\n", "\n");
}

async function verifyOnlySrcTauriChanged(spec) {
  const names = new Set();
  for (const [suffix, args] of [
    ["unstaged", ["diff", "--name-only", "HEAD"]],
    ["staged", ["diff", "--cached", "--name-only", "HEAD"]],
  ]) {
    const output = await git({ ...spec, label: `${spec.label}.outside.${suffix}`, args });
    for (const name of output.split("\n").filter(Boolean)) names.add(name);
  }
  const outside = [...names].filter((name) => !name.startsWith("src-tauri/"));
  if (outside.length > 0) throw new ProtocolError("state_changed_outside_src_tauri", outside.join(","));
}

export async function installState({ state, worktree, mainRoot, protocolLock, artifactDir }) {
  const [resolvedWorktree, resolvedMainRoot] = await Promise.all([
    realpath(worktree),
    realpath(mainRoot),
  ]);
  if (normalizedPath(resolvedWorktree) === normalizedPath(resolvedMainRoot)) {
    throw new ProtocolError("worktree_not_isolated", "state installation cannot run in main");
  }
  const kind = normalizedState(state);
  const prefix = `state-${state}`;
  const shared = { worktree, artifactDir, label: prefix };
  await git({
    ...shared,
    label: `${prefix}.restore-a`,
    args: aRestoreArgs(),
  });

  if (["B", "C", "E"].includes(kind)) {
    const patchRelative = `scripts/process-shell-diagnostic/states/${kind}.patch`;
    const patchPath = await git({
      ...shared,
      label: `${prefix}.canonical-patch-blob`,
      args: ["cat-file", "blob", `HEAD:${patchRelative}`],
      rawOutput: true,
    });
    const actualPatchSha256 = await sha256File(patchPath);
    if (actualPatchSha256 !== protocolLock.states[kind].patchSha256) {
      throw new ProtocolError("state_patch_hash_mismatch", `${kind}: ${actualPatchSha256}`);
    }
    await git({ ...shared, label: `${prefix}.patch-check`, args: ["apply", "--check", "--index", patchPath] });
    await git({ ...shared, label: `${prefix}.patch-apply`, args: ["apply", "--index", patchPath] });
  } else if (kind === "D") {
    await git({ ...shared, label: `${prefix}.candidate-restore`, args: dRestoreArgs() });
  }

  await verifyOnlySrcTauriChanged(shared);
  await git({ ...shared, label: `${prefix}.worktree-index-clean`, args: ["diff", "--quiet"] });
  const rootTree = await git({ ...shared, label: `${prefix}.write-tree`, args: ["write-tree"] });
  const srcTauriTree = await git({
    ...shared,
    label: `${prefix}.subtree`,
    args: ["rev-parse", `${rootTree}:src-tauri`],
  });
  const expectedTree = protocolLock.states[kind].srcTauriTree;
  if (srcTauriTree !== expectedTree) {
    throw new ProtocolError("state_tree_mismatch", `${kind}: ${srcTauriTree} != ${expectedTree}`);
  }
  if (kind === "A" && srcTauriTree !== STATE_TREE_ANCHORS.A) {
    throw new ProtocolError("baseline_tree_mismatch", srcTauriTree);
  }

  const processPathsText = await git({
    ...shared,
    label: `${prefix}.process-paths`,
    args: ["ls-files", "--", "src-tauri/crates/extractum-process"],
  });
  const processPaths = processPathsText.split("\n").filter(Boolean).sort();
  if (["B", "C", "E"].includes(kind)) {
    const rootManifest = await readFile(path.join(worktree, "src-tauri", "Cargo.toml"), "utf8");
    const processManifest = await readFile(
      path.join(worktree, "src-tauri", "crates", "extractum-process", "Cargo.toml"),
      "utf8",
    );
    validateStateManifests({ state: kind, rootManifest, processManifest, processPaths });
    const baselineLockPath = await git({
      ...shared,
      label: `${prefix}.baseline-lock-blob`,
      args: ["cat-file", "blob", `${PROTOCOL.baselineCommit}:src-tauri/Cargo.lock`],
      rawOutput: true,
    });
    validateLockDelta({
      state: kind,
      baselineLock: await readFile(baselineLockPath, "utf8"),
      stateLock: await readFile(path.join(worktree, "src-tauri", "Cargo.lock"), "utf8"),
    });
  }

  if (kind === "D") {
    if (srcTauriTree !== STATE_TREE_ANCHORS.D) throw new ProtocolError("candidate_tree_mismatch", srcTauriTree);
    const expectedInventory = await git({
      ...shared,
      label: `${prefix}.expected-inventory`,
      args: ["ls-tree", "-r", PROTOCOL.candidateCommit, "--", "src-tauri"],
    });
    const actualInventory = await git({
      ...shared,
      label: `${prefix}.actual-inventory`,
      args: ["ls-tree", "-r", rootTree, "--", "src-tauri"],
    });
    if (actualInventory !== expectedInventory) {
      throw new ProtocolError("candidate_inventory_mismatch", "D path/mode/blob inventory differs");
    }
    for (const [filePath, blob] of Object.entries(D_BLOB_ANCHORS)) {
      if (!actualInventory.includes(`100644 blob ${blob}\t${filePath}`)) {
        throw new ProtocolError("candidate_blob_mismatch", `${filePath}:${blob}`);
      }
    }
    await git({
      ...shared,
      label: `${prefix}.required-diff`,
      args: ["diff", "--quiet", PROTOCOL.candidateCommit, "--", "src-tauri"],
    });
    await git({
      ...shared,
      label: `${prefix}.cached-diff`,
      args: ["diff", "--cached", "--quiet", PROTOCOL.candidateCommit, "--", "src-tauri"],
    });
  }

  const sourcePath = path.join(worktree, "src-tauri", "src", "lib.rs");
  const evidence = {
    schemaVersion: 1,
    state,
    kind,
    mainRoot,
    worktree,
    rootTree,
    srcTauriTree,
    canonicalLibSha256: await sha256File(sourcePath),
    processPaths,
  };
  await writeAtomicJsonExclusive(path.join(artifactDir, "states", `${state}.json`), evidence);
  return evidence;
}
