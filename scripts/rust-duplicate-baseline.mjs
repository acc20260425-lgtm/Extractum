import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const TARGET = "x86_64-pc-windows-msvc";
const BASELINE_PATH = "scripts/testing/rust-duplicate-baseline.json";
const EXCEPTIONS_PATH = "scripts/testing/rust-supply-chain-exceptions.json";
const CARGO_TREE_ARGS = ["tree", "--manifest-path", "src-tauri/Cargo.toml", "--locked", "--target", TARGET, "--workspace", "--prefix", "none", "--format", "{p}"];
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const json = (value) => JSON.stringify(value);
const isObject = (value) => Boolean(value) && typeof value === "object" && !Array.isArray(value);

export function generateRustDuplicateBaseline(treeText) {
  const identities = new Map();
  for (const line of String(treeText).split(/\r?\n/)) {
    const match = /^(\S+) v(\S+)/.exec(line.trim());
    if (match) identities.set(`${match[1]}@${match[2]}`, { name: match[1], version: match[2] });
  }
  const versions = new Map();
  for (const { name, version } of identities.values()) {
    if (!versions.has(name)) versions.set(name, new Set());
    versions.get(name).add(version);
  }
  const duplicateCardinality = Object.fromEntries([...versions].filter(([, values]) => values.size > 1).sort(([left], [right]) => left.localeCompare(right)).map(([name, values]) => [name, values.size]));
  return { schemaVersion: 1, target: TARGET, duplicateNameCount: Object.keys(duplicateCardinality).length, duplicateVersionInstanceCount: Object.values(duplicateCardinality).reduce((sum, count) => sum + count, 0), duplicateCardinality };
}

export function validateDuplicateBaseline(value) {
  const expectedKeys = ["duplicateCardinality", "duplicateNameCount", "duplicateVersionInstanceCount", "schemaVersion", "target"];
  if (!isObject(value) || json(Object.keys(value).sort()) !== json(expectedKeys)
    || value.schemaVersion !== 1 || value.target !== TARGET || !isObject(value.duplicateCardinality)) throw new Error("duplicate baseline schema is invalid");
  const entries = Object.entries(value.duplicateCardinality);
  if (entries.some(([name, count]) => !name || !Number.isInteger(count) || count < 2)) throw new Error("duplicate baseline cardinality is invalid");
  if (json(entries.map(([name]) => name)) !== json(entries.map(([name]) => name).sort((a, b) => a.localeCompare(b)))) throw new Error("duplicate baseline names must be sorted");
  if (value.duplicateNameCount !== entries.length) throw new Error("duplicateNameCount drifted");
  if (value.duplicateVersionInstanceCount !== entries.reduce((sum, [, count]) => sum + count, 0)) throw new Error("duplicateVersionInstanceCount drifted");
  return value;
}

function exceptionErrors(exceptions) {
  const topKeys = ["advisoryExceptions", "duplicateGrowthExceptions", "licenseExceptions", "schemaVersion"];
  if (!isObject(exceptions) || json(Object.keys(exceptions).sort()) !== json(topKeys)
    || exceptions.schemaVersion !== 1 || !Array.isArray(exceptions.licenseExceptions)
    || !Array.isArray(exceptions.advisoryExceptions) || !Array.isArray(exceptions.duplicateGrowthExceptions)) return ["duplicate exception schema is invalid"];
  const errors = [];
  const expectedEntryKeys = ["approvedCount", "owner", "package", "previousCount", "reason", "reviewAfter"];
  for (const [index, entry] of exceptions.duplicateGrowthExceptions.entries()) {
    if (!isObject(entry) || json(Object.keys(entry).sort()) !== json(expectedEntryKeys)
      || !["package", "owner", "reason", "reviewAfter"].every((key) => typeof entry[key] === "string" && entry[key].length > 0)
      || !Number.isInteger(entry.previousCount) || entry.previousCount < 1
      || !Number.isInteger(entry.approvedCount) || entry.approvedCount <= entry.previousCount) errors.push(`duplicate exception ${index} is invalid`);
  }
  const packages = exceptions.duplicateGrowthExceptions.map((entry) => entry.package);
  if (json(packages) !== json([...packages].sort((a, b) => String(a).localeCompare(String(b)))) || new Set(packages).size !== packages.length) errors.push("duplicate exception packages must be sorted and unique");
  return errors;
}

export function validateCurrentDuplicateState({ treeText, baseline, exceptions }) {
  const violations = [];
  let validBaseline;
  try { validBaseline = validateDuplicateBaseline(baseline); } catch (error) { violations.push(error.message); }
  violations.push(...exceptionErrors(exceptions));
  const current = generateRustDuplicateBaseline(treeText);
  if (validBaseline && json(current) !== json(validBaseline)) violations.push("current duplicate graph differs from committed baseline");
  return { current, baseline, exceptions, violations };
}

export function compareDuplicateGrowth({ current, base, exceptions }) {
  const errors = exceptionErrors(exceptions); if (errors.length) return errors;
  const violations = [];
  for (const name of Object.keys(current?.duplicateCardinality ?? {}).sort((a, b) => a.localeCompare(b))) {
    const count = current.duplicateCardinality[name]; const previousCount = base?.duplicateCardinality?.[name] ?? 1;
    if (count <= previousCount) continue;
    const approvals = exceptions.duplicateGrowthExceptions.filter((entry) => entry.package === name && entry.previousCount === previousCount && entry.approvedCount === count);
    if (approvals.length !== 1) violations.push(`${name}: duplicate growth ${previousCount} -> ${count} requires exact approval`);
  }
  return violations;
}

export function writeRustDuplicateBaseline({ treeText, path: outputPath }) {
  const baseline = generateRustDuplicateBaseline(treeText);
  writeFileSync(outputPath, `${JSON.stringify(baseline, null, 2)}\n`);
  return baseline;
}

function gitHas(cwd, revision, file) { try { execFileSync("git", ["cat-file", "-e", `${revision}:${file}`], { cwd, stdio: "ignore", windowsHide: true }); return true; } catch { return false; } }
function gitShow(cwd, revision, file) { return execFileSync("git", ["show", `${revision}:${file}`], { cwd, encoding: "utf8", windowsHide: true }); }

export function checkRustDuplicatePolicy({ base, cwd = root, treeText, stderr = process.stderr }) {
  const baseline = JSON.parse(readFileSync(path.join(cwd, BASELINE_PATH), "utf8"));
  const exceptions = JSON.parse(readFileSync(path.join(cwd, EXCEPTIONS_PATH), "utf8"));
  const tree = treeText ?? execFileSync("cargo", CARGO_TREE_ARGS, { cwd, encoding: "utf8", windowsHide: true });
  const state = validateCurrentDuplicateState({ treeText: tree, baseline, exceptions });
  if (state.violations.length) throw new Error(state.violations.join("\n"));
  if (!base) return { historicalSkipped: false };
  let revision;
  try { revision = execFileSync("git", ["rev-parse", "--verify", `${base}^{commit}`], { cwd, encoding: "utf8", windowsHide: true, stdio: ["ignore", "pipe", "ignore"] }).trim(); } catch { throw new Error(`revision ${base} does not resolve`); }
  if (!gitHas(cwd, revision, BASELINE_PATH) || !gitHas(cwd, revision, EXCEPTIONS_PATH)) { stderr.write("historical duplicate policy unavailable; skipping base comparison\n"); return { historicalSkipped: true }; }
  const historicalBaseline = JSON.parse(gitShow(cwd, revision, BASELINE_PATH));
  const historicalExceptions = JSON.parse(gitShow(cwd, revision, EXCEPTIONS_PATH));
  const historicalExceptionErrors = exceptionErrors(historicalExceptions);
  if (historicalExceptionErrors.length) throw new Error(historicalExceptionErrors.join("\n"));
  const violations = compareDuplicateGrowth({ current: state.current, base: validateDuplicateBaseline(historicalBaseline), exceptions });
  if (violations.length) throw new Error(violations.join("\n"));
  return { historicalSkipped: false };
}

function main(args) {
  if (args[0] === "--write" && args.length === 2) return writeRustDuplicateBaseline({ treeText: execFileSync("cargo", CARGO_TREE_ARGS, { cwd: root, encoding: "utf8", windowsHide: true }), path: path.resolve(root, args[1]) });
  if (args[0] === "--check" && (args.length === 1 || args.length === 3 && args[1] === "--base" && args[2])) return checkRustDuplicatePolicy({ base: args[2], cwd: root });
  throw new Error("Usage: node scripts/rust-duplicate-baseline.mjs --write <path> | --check [--base SHA]");
}
if (process.argv[1] === fileURLToPath(import.meta.url)) main(process.argv.slice(2));
