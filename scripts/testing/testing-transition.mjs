import { spawn } from "node:child_process";
import { lstatSync, readFileSync } from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

export {
  buildLedgerDraft,
  discoverSourceReaders,
  discoverTestDeclarations,
  validateSourceContractLedger,
} from "./extract-source-contract-ledger.mjs";

const TEST_FILE_PATTERN = /\.(test|spec)\.[cm]?[jt]sx?$/i;
const GLOB_PATTERN = /[*?\[\]{}]/;
const require = createRequire(import.meta.url);

function toText(value) {
  return Buffer.isBuffer(value) ? value.toString("utf8") : String(value ?? "");
}

function normalizeRoot(repoRoot) {
  return String(repoRoot).replaceAll("\\", "/").replace(/\/+$/, "");
}

export function normalizeRepoPath(repoRoot, input) {
  const root = normalizeRoot(repoRoot);
  let candidate = String(input).trim().replaceAll("\\", "/");
  if (!candidate) return { issue: "empty path" };

  const isWindowsAbsolute = /^[A-Za-z]:\//.test(candidate);
  const isPosixAbsolute = candidate.startsWith("/");
  if (!root && (isWindowsAbsolute || isPosixAbsolute)) return { issue: `repository escape: ${input}` };
  if (isWindowsAbsolute || isPosixAbsolute) {
    const insensitive = /^[A-Za-z]:\//.test(root);
    const compareRoot = insensitive ? root.toLowerCase() : root;
    const compareCandidate = insensitive ? candidate.toLowerCase() : candidate;
    if (compareCandidate === compareRoot) return { issue: `repository escape: ${input}` };
    if (!compareCandidate.startsWith(`${compareRoot}/`)) return { issue: `repository escape: ${input}` };
    candidate = candidate.slice(root.length + 1);
  }

  candidate = candidate.replace(/^\.\//, "");
  if (candidate.split("/").some((segment) => segment === ".." || segment === "")) {
    return { issue: `repository escape: ${input}` };
  }
  return { path: candidate };
}

function validateRelativeExactPath(kind, entry) {
  if (!entry || typeof entry !== "object" || Array.isArray(entry)) return [`invalid ${kind} exception`];
  const issues = [];
  for (const key of Object.keys(entry)) {
    if (!new Set(["path", "reason", "owner"]).has(key)) issues.push(`unknown ${kind} exception field: ${key}`);
  }
  for (const field of ["path", "reason", "owner"]) {
    if (typeof entry[field] !== "string" || !entry[field].trim()) issues.push(`invalid ${kind} exception ${field}`);
  }
  if (typeof entry.path === "string") {
    const normalized = normalizeRepoPath("", entry.path);
    if (normalized.issue || GLOB_PATTERN.test(entry.path) || entry.path.endsWith("/")) {
      issues.push(`invalid ${kind} exception path: ${entry.path}`);
    }
  }
  return issues;
}

export function validateCensusSchema(census) {
  const issues = [];
  if (!census || typeof census !== "object" || Array.isArray(census)) return ["invalid runner census"];
  const expected = new Set(["schemaVersion", "vitestOwners", "playwrightOwners", "nonstandardTests", "fixtureExceptions"]);
  for (const key of Object.keys(census)) if (!expected.has(key)) issues.push(`unknown runner census field: ${key}`);
  if (census.schemaVersion !== 1) issues.push("unsupported runner census schemaVersion");
  for (const key of ["vitestOwners", "playwrightOwners", "nonstandardTests", "fixtureExceptions"]) {
    if (!Array.isArray(census[key])) issues.push(`runner census ${key} must be an array`);
  }
  if (issues.length) return issues.sort();

  const ids = new Set();
  for (const owner of census.vitestOwners) {
    if (!owner || typeof owner !== "object" || Array.isArray(owner)) { issues.push("invalid Vitest owner"); continue; }
    for (const key of Object.keys(owner)) if (!new Set(["id", "args", "ownerScript"]).has(key)) issues.push(`unknown Vitest owner field: ${key}`);
    if (typeof owner.id !== "string" || !owner.id) issues.push("invalid Vitest owner id");
    if (!Array.isArray(owner.args) || owner.args.some((arg) => typeof arg !== "string")) issues.push(`invalid Vitest owner args: ${owner.id ?? "unknown"}`);
    if (typeof owner.ownerScript !== "string" || !owner.ownerScript) issues.push(`invalid Vitest ownerScript: ${owner.id ?? "unknown"}`);
    if (ids.has(owner.id)) issues.push(`duplicate owner id: ${owner.id}`);
    ids.add(owner.id);
  }
  for (const owner of census.playwrightOwners) {
    if (!owner || typeof owner !== "object" || Array.isArray(owner)) { issues.push("invalid Playwright owner"); continue; }
    for (const key of Object.keys(owner)) if (!new Set(["id", "config", "ownerScript"]).has(key)) issues.push(`unknown Playwright owner field: ${key}`);
    for (const key of ["id", "config", "ownerScript"]) if (typeof owner[key] !== "string" || !owner[key]) issues.push(`invalid Playwright owner ${key}`);
    if (ids.has(owner.id)) issues.push(`duplicate owner id: ${owner.id}`);
    ids.add(owner.id);
  }
  for (const [kind, entries] of [["nonstandard", census.nonstandardTests], ["fixture", census.fixtureExceptions]]) {
    const paths = new Set();
    for (const entry of entries) {
      issues.push(...validateRelativeExactPath(kind, entry));
      if (entry?.path && paths.has(entry.path)) issues.push(`duplicate ${kind} exception: ${entry.path}`);
      paths.add(entry?.path);
    }
  }
  return [...new Set(issues)].sort();
}

function defaultRunGit(args, repoRoot) {
  return runProcess("git", args, repoRoot);
}

export async function discoverFilesystemCandidates(repoRoot, runGit = defaultRunGit, lstat = lstatSync) {
  const result = await runGit(["ls-files", "--cached", "--others", "--exclude-standard", "-z"], repoRoot);
  const issues = [];
  if (result.exitCode !== 0 || result.error) {
    return { files: [], issues: [`git ls-files failed${result.error ? `: ${result.error.message}` : ""}`] };
  }
  const files = [];
  for (const listed of toText(result.stdout).split("\0").filter(Boolean)) {
    const normalized = normalizeRepoPath(repoRoot, listed);
    if (normalized.issue) { issues.push(normalized.issue); continue; }
    if (!TEST_FILE_PATTERN.test(normalized.path)) continue;
    let stats;
    try {
      stats = lstat(path.join(repoRoot, normalized.path));
    } catch (error) {
      if (error?.code === "ENOENT") continue;
      throw error;
    }
    if (stats.isSymbolicLink()) {
      issues.push(`unsupported candidate symlink: ${normalized.path}`);
      continue;
    }
    if (stats.isFile()) files.push(normalized.path);
  }
  return { files: [...new Set(files)].sort(), issues: [...new Set(issues)].sort() };
}

export function runProcess(command, args, cwd) {
  return new Promise((resolve) => {
    const child = spawn(command, args, { cwd, shell: false, windowsHide: true });
    const stdout = [];
    const stderr = [];
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", (error) => resolve({ exitCode: 1, stdout: Buffer.concat(stdout), stderr: Buffer.concat(stderr), error }));
    child.on("close", (code) => resolve({ exitCode: code ?? 1, stdout: Buffer.concat(stdout), stderr: Buffer.concat(stderr) }));
  });
}

function collectionResult(owner, result, repoRoot, parser) {
  const issues = [];
  if (result.error) issues.push(`runner spawn error: ${owner.id}: ${result.error.message}`);
  if (result.exitCode !== 0) issues.push(`runner failed: ${owner.id}`);
  const parsed = parser(toText(result.stdout));
  if (parsed.issue) issues.push(`${owner.id}: ${parsed.issue}`);
  const files = [];
  for (const value of parsed.files ?? []) {
    const normalized = normalizeRepoPath(repoRoot, value);
    if (normalized.issue) issues.push(`${owner.id}: ${normalized.issue}`);
    else files.push(normalized.path);
  }
  return { files: [...new Set(files)].sort(), issues };
}

export async function collectVitestFiles(owner, { repoRoot, runCommand = runProcess } = {}) {
  const result = await runCommand(process.execPath, ["scripts/run-vitest.mjs", "list", "--filesOnly", "--no-color", ...owner.args], repoRoot);
  return collectionResult(owner, result, repoRoot, (stdout) => ({ files: stdout.split(/\r?\n/).map((line) => line.trim()).filter(Boolean) }));
}

function walkPlaywright(node, files) {
  if (!node || typeof node !== "object") return;
  if (typeof node.file === "string") files.push(node.file);
  for (const child of node.suites ?? []) walkPlaywright(child, files);
  for (const spec of node.specs ?? []) if (typeof spec.file === "string") files.push(spec.file);
}

function playwrightRootDir(repoRoot, rootDir) {
  if (rootDir === undefined) return { issue: "missing Playwright config.rootDir" };
  if (typeof rootDir !== "string") return { issue: "invalid Playwright config.rootDir" };
  const value = rootDir.trim();
  if (!value) return { issue: "invalid Playwright config.rootDir" };
  const normalizedRoot = normalizeRoot(repoRoot);
  const candidate = value.replaceAll("\\", "/");
  const isAbsolute = /^[A-Za-z]:\//.test(candidate) || candidate.startsWith("/");
  if (isAbsolute) {
    const insensitive = /^[A-Za-z]:\//.test(normalizedRoot);
    const compareRoot = insensitive ? normalizedRoot.toLowerCase() : normalizedRoot;
    const compareCandidate = insensitive ? candidate.toLowerCase() : candidate;
    if (compareCandidate === compareRoot) return { path: "." };
    if (!compareCandidate.startsWith(`${compareRoot}/`)) {
      return { issue: `Playwright config.rootDir escapes repository: ${rootDir}` };
    }
    const relative = normalizeRepoPath("", candidate.slice(normalizedRoot.length + 1));
    return relative.issue ? { issue: `Playwright config.rootDir escapes repository: ${rootDir}` } : relative;
  }
  const relative = normalizeRepoPath("", candidate);
  return relative.issue ? { issue: `Playwright config.rootDir escapes repository: ${rootDir}` } : relative;
}

export async function collectPlaywrightFiles(owner, { repoRoot, runCommand = runProcess, resolveCli = (id) => require.resolve(id) } = {}) {
  let cli;
  try { cli = resolveCli("@playwright/test/cli"); } catch (error) { return { files: [], issues: [`runner spawn error: ${owner.id}: ${error.message}`] }; }
  const result = await runCommand(process.execPath, [cli, "test", "-c", owner.config, "--list", "--reporter=json"], repoRoot);
  return collectionResult(owner, result, repoRoot, (stdout) => {
    try {
      const report = JSON.parse(stdout);
      if (!Array.isArray(report.suites)) return { issue: "malformed Playwright JSON" };
      if ("errors" in report && (!Array.isArray(report.errors) || report.errors.length)) return { issue: "Playwright errors are not empty" };
      const configRoot = playwrightRootDir(repoRoot, report.config?.rootDir);
      if (configRoot.issue) return configRoot;
      const rawFiles = [];
      for (const suite of report.suites) walkPlaywright(suite, rawFiles);
      return { files: rawFiles.map((file) => /^[A-Za-z]:[\\/]|^\//.test(file) ? file : path.posix.join(configRoot.path, file)) };
    } catch { return { issue: "malformed Playwright JSON" }; }
  });
}

export function validateRunnerCensus({ census, filesystemFiles = [], vitestFiles = {}, playwrightFiles = {}, runnerIssues = [] }) {
  const issues = [...validateCensusSchema(census), ...runnerIssues];
  if (!census || !Array.isArray(census.vitestOwners) || !Array.isArray(census.playwrightOwners)
    || !Array.isArray(census.nonstandardTests) || !Array.isArray(census.fixtureExceptions)) return [...new Set(issues)].sort();
  const malformedOwner = [...census.vitestOwners, ...census.playwrightOwners].some((owner) =>
    !owner || typeof owner !== "object" || Array.isArray(owner),
  );
  if (malformedOwner) return [...new Set(issues)].sort();
  const malformedException = [...census.nonstandardTests, ...census.fixtureExceptions].some((entry) =>
    !entry || typeof entry !== "object" || Array.isArray(entry)
    || ["path", "reason", "owner"].some((field) => typeof entry[field] !== "string" || !entry[field].trim()),
  );
  if (malformedException) return [...new Set(issues)].sort();
  const candidates = new Set(filesystemFiles);
  const allOwners = [...census.vitestOwners, ...census.playwrightOwners];
  const byOwner = new Map(allOwners.map((owner) => [owner.id, [...(vitestFiles[owner.id] ?? playwrightFiles[owner.id] ?? [])]]));
  const nonstandard = new Map(census.nonstandardTests.map((entry) => [entry.path, entry]));
  const fixtures = new Map(census.fixtureExceptions.map((entry) => [entry.path, entry]));
  const ownership = new Map();

  for (const [owner, files] of byOwner) {
    if (!files.length) issues.push(`empty owner: ${owner}`);
    for (const file of files) {
      const owners = ownership.get(file) ?? [];
      owners.push(owner);
      ownership.set(file, owners);
      const exception = nonstandard.get(file);
      if (!candidates.has(file) && (!exception || exception.owner !== owner)) issues.push(`collected non-candidate: ${owner} -> ${file}`);
    }
  }
  for (const [file, owners] of ownership) if (owners.length > 1) issues.push(`duplicate ownership: ${file} -> ${owners.sort().join(", ")}`);
  for (const file of candidates) if (!ownership.has(file) && !fixtures.has(file)) issues.push(`unowned filesystem candidate: ${file}`);
  for (const entry of census.nonstandardTests) {
    const owners = ownership.get(entry.path) ?? [];
    if (candidates.has(entry.path) || !owners.includes(entry.owner)) issues.push(`stale nonstandard exception: ${entry.path}`);
  }
  for (const entry of census.fixtureExceptions) {
    if (!candidates.has(entry.path) || ownership.has(entry.path)) issues.push(`stale fixture exception: ${entry.path}`);
  }
  return [...new Set(issues)].sort();
}

export async function runTransitionValidation({ repoRoot, checks, stdout = process.stdout, stderr = process.stderr }) {
  const issues = [];
  const summaries = [];
  for (const check of checks) {
    const result = await check(repoRoot);
    if (Array.isArray(result)) issues.push(...result);
    else if (result) {
      issues.push(...(result.issues ?? []));
      if (result.summary) summaries.push(result.summary);
    }
  }
  for (const summary of summaries) stdout.write(`${summary}\n`);
  for (const issue of [...new Set(issues)].sort()) stderr.write(`${issue}\n`);
  return { exitCode: issues.length ? 1 : 0, issues: [...new Set(issues)].sort() };
}

export async function validateLiveRunnerCensus(repoRoot, census) {
  const discovered = await discoverFilesystemCandidates(repoRoot);
  const runnerIssues = [...discovered.issues];
  const vitestFiles = {};
  const playwrightFiles = {};
  for (const owner of census.vitestOwners) {
    const collection = await collectVitestFiles(owner, { repoRoot });
    vitestFiles[owner.id] = collection.files;
    runnerIssues.push(...collection.issues);
  }
  for (const owner of census.playwrightOwners) {
    const collection = await collectPlaywrightFiles(owner, { repoRoot });
    playwrightFiles[owner.id] = collection.files;
    runnerIssues.push(...collection.issues);
  }
  const vitestCount = Object.values(vitestFiles).flat().length;
  const playwrightCount = Object.values(playwrightFiles).flat().length;
  return {
    issues: validateRunnerCensus({ census, filesystemFiles: discovered.files, vitestFiles, playwrightFiles, runnerIssues }),
    summary: `Runner census: ${discovered.files.length} filesystem candidates, ${vitestCount} Vitest files, ${playwrightCount} Playwright files`,
    vitestFiles,
    playwrightFiles,
  };
}

export function loadRunnerCensus(repoRoot) {
  return JSON.parse(readFileSync(path.join(repoRoot, "testing", "runner-census.json"), "utf8"));
}
