import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const NORMAL_KINDS = new Set(["normal", "build", "dev"]);
const isObject = (value) => Boolean(value) && typeof value === "object" && !Array.isArray(value);
const canonical = (value) => JSON.stringify(value);
const sorted = (values, key) => [...values].sort((left, right) => key(left).localeCompare(key(right)));
const nonEmptyString = (value) => typeof value === "string" && value.length > 0;

export function cargoRequirementIdentity(entry) { return canonical([entry.package, entry.dependency, entry.rename, entry.kind, entry.target]); }
export function npmRequirementIdentity(entry) { return canonical([entry.owner, entry.name, entry.kind]); }

function validCargo(entry, label, errors) {
  if (!isObject(entry) || !nonEmptyString(entry.package) || !nonEmptyString(entry.dependency) || !NORMAL_KINDS.has(entry.kind)
    || !(entry.rename === null || typeof entry.rename === "string") || !(entry.target === null || typeof entry.target === "string")) { errors.push(`${label} has invalid Cargo identity`); return false; }
  return true;
}
function validNpm(entry, label, errors) {
  if (!isObject(entry) || !nonEmptyString(entry.owner) || !nonEmptyString(entry.name) || !["dependencies", "devDependencies"].includes(entry.kind)) { errors.push(`${label} has invalid npm identity`); return false; }
  return true;
}
function validateReviewedAuthority(value) {
  const errors = [];
  if (!isObject(value) || value.schemaVersion !== 2) errors.push("schemaVersion must be exactly 2");
  const toolchain = value?.toolchain;
  if (!isObject(toolchain) || !["channel", "rustVersion", "edition", "target"].every((key) => nonEmptyString(toolchain?.[key])) || !Array.isArray(toolchain?.workspacePackages) || toolchain.workspacePackages.some((name) => !nonEmptyString(name)) || canonical(sorted(toolchain?.workspacePackages ?? [], String)) !== canonical(toolchain?.workspacePackages ?? []) || new Set(toolchain?.workspacePackages ?? []).size !== (toolchain?.workspacePackages ?? []).length) errors.push("toolchain is malformed or unsorted");
  const family = value?.tauriFamily;
  if (!isObject(family) || !Array.isArray(family.pairs) || !Array.isArray(family.cargoOnlyRequirements)) return [...errors, "tauriFamily must contain pairs and cargoOnlyRequirements arrays"];
  const ids = new Set(); const cargos = new Set(); const npms = new Set();
  for (const [index, pair] of family.pairs.entries()) {
    const label = `tauriFamily.pairs[${index}]`;
    if (!isObject(pair) || !nonEmptyString(pair.id) || !nonEmptyString(pair.cargoRequirement) || !nonEmptyString(pair.npmRequirement)) { errors.push(`${label} is malformed`); continue; }
    if (ids.has(pair.id)) errors.push("tauriFamily pair IDs must be unique"); ids.add(pair.id);
    if (validCargo(pair.cargo, `${label}.cargo`, errors)) { const key = cargoRequirementIdentity(pair.cargo); if (cargos.has(key)) errors.push("tauriFamily Cargo identities must be unique"); cargos.add(key); }
    if (validNpm(pair.npm, `${label}.npm`, errors)) { const key = npmRequirementIdentity(pair.npm); if (npms.has(key)) errors.push("tauriFamily npm identities must be unique"); npms.add(key); }
  }
  const onlyIds = [];
  for (const [index, row] of family.cargoOnlyRequirements.entries()) {
    const label = `tauriFamily.cargoOnlyRequirements[${index}]`;
    if (!isObject(row) || !nonEmptyString(row.id) || !nonEmptyString(row.cargoRequirement)) { errors.push(`${label} is malformed`); continue; }
    onlyIds.push(row.id); validCargo(row.cargo, `${label}.cargo`, errors);
  }
  if (new Set(onlyIds).size !== onlyIds.length) errors.push("tauriFamily cargo-only IDs must be unique");
  if (canonical(sorted(onlyIds, String)) !== canonical(onlyIds)) errors.push("tauriFamily cargo-only IDs must be sorted");
  return errors;
}
function normalizedKind(kind) { return kind === null || kind === undefined ? "normal" : kind; }
function cargoDirectRequirements(metadata) {
  const members = new Set(metadata?.workspace_members ?? []); const inventory = [];
  for (const pkg of metadata?.packages ?? []) if (members.has(pkg.id)) for (const dependency of pkg.dependencies ?? []) {
    const kind = normalizedKind(dependency.kind); if (NORMAL_KINDS.has(kind)) inventory.push({ package: pkg.name, dependency: dependency.name, rename: dependency.rename ?? null, kind, target: dependency.target ?? null, requirement: dependency.req });
  }
  return sorted(inventory, cargoRequirementIdentity);
}
function npmDirectRequirements(packageJson) {
  const inventory = [];
  for (const kind of ["dependencies", "devDependencies"]) for (const [name, requirement] of Object.entries(packageJson?.[kind] ?? {})) inventory.push({ owner: packageJson?.name ?? "extractum", name, kind, requirement });
  return sorted(inventory, npmRequirementIdentity);
}
function prereleaseFacts(requirements) {
  const facts = [];
  for (const entry of requirements) {
    const tokens = [...String(entry.requirement).matchAll(/\d+\.\d+\.\d+-([0-9A-Za-z.-]+)/g)].map((match) => match[1]);
    if (new Set(tokens).size > 1) throw new Error(`${entry.dependency}: ambiguous prerelease requirement`);
    if (tokens.length) facts.push({ package: entry.package, dependency: entry.dependency, rename: entry.rename, kind: entry.kind, target: entry.target, approvedPrerelease: tokens[0] });
  }
  return sorted(facts, cargoRequirementIdentity);
}
export function generateRustDependencyPolicy({ metadata, packageJson, reviewed }) {
  const errors = validateReviewedAuthority(reviewed); if (errors.length) throw new Error(errors.join("\n"));
  const directRequirements = cargoDirectRequirements(metadata);
  return { schemaVersion: 2, toolchain: reviewed.toolchain, directRequirements, npmRequirements: npmDirectRequirements(packageJson), approvedPrereleases: prereleaseFacts(directRequirements), tauriFamily: reviewed.tauriFamily };
}
const same = (left, right) => canonical(left) === canonical(right);
export function validateRustDependencyPolicy({ generated, committed }) {
  const errors = validateReviewedAuthority(committed);
  if (!isObject(generated) || generated.schemaVersion !== 2) errors.push("generated policy schemaVersion must be exactly 2");
  if (errors.length) return errors;
  for (const key of ["toolchain", "directRequirements", "npmRequirements", "approvedPrereleases", "tauriFamily"]) if (!same(generated[key], committed[key])) errors.push(`${key} drifted`);
  const cargoTauri = (generated.directRequirements ?? []).filter((entry) => entry.package === "extractum" && entry.dependency.startsWith("tauri") && entry.dependency !== "tauri-plugin-mcp-bridge");
  const reviewedCargo = committed.tauriFamily.pairs.map(({ cargo }) => cargo);
  if (!same(sorted(cargoTauri.map(({ requirement, ...entry }) => entry), cargoRequirementIdentity), sorted(reviewedCargo, cargoRequirementIdentity))) errors.push("paired Cargo bijection drifted");
  const npmTauri = (generated.npmRequirements ?? []).filter(({ name }) => name.startsWith("@tauri-apps/"));
  const reviewedNpm = committed.tauriFamily.pairs.map(({ npm }) => npm);
  if (!same(sorted(npmTauri.map(({ requirement, ...entry }) => entry), npmRequirementIdentity), sorted(reviewedNpm, npmRequirementIdentity))) errors.push("paired npm bijection drifted");
  for (const pair of committed.tauriFamily.pairs) {
    const cargo = generated.directRequirements?.find((entry) => cargoRequirementIdentity(entry) === cargoRequirementIdentity(pair.cargo));
    if (!cargo || cargo.requirement !== pair.cargoRequirement) errors.push(`${pair.id}: Cargo requirement drifted`);
    const npm = generated.npmRequirements?.find((entry) => npmRequirementIdentity(entry) === npmRequirementIdentity(pair.npm));
    if (!npm || npm.requirement !== pair.npmRequirement) errors.push(`${pair.id}: npm requirement drifted`);
  }
  for (const row of committed.tauriFamily.cargoOnlyRequirements) {
    const cargo = generated.directRequirements?.find((entry) => cargoRequirementIdentity(entry) === cargoRequirementIdentity(row.cargo));
    if (!cargo || cargo.requirement !== row.cargoRequirement) errors.push(`${row.id}: Cargo requirement drifted`);
  }
  return [...new Set(errors)];
}
function main(args) {
  if (args.length !== 2 || args[0] !== "--write") throw new Error("Usage: node scripts/rust-dependency-policy.mjs --write <path>");
  const outputPath = path.resolve(ROOT, args[1]); const reviewed = JSON.parse(readFileSync(outputPath, "utf8"));
  const errors = validateReviewedAuthority(reviewed); if (errors.length) throw new Error(errors.join("\n"));
  const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--manifest-path", "src-tauri/Cargo.toml", "--locked", "--format-version", "1"], { cwd: ROOT, encoding: "utf8", windowsHide: true, maxBuffer: 256 * 1024 * 1024 }));
  const packageJson = JSON.parse(readFileSync(path.join(ROOT, "package.json"), "utf8"));
  writeFileSync(outputPath, `${JSON.stringify(generateRustDependencyPolicy({ metadata, packageJson, reviewed }), null, 2)}\n`);
}
if (process.argv[1] === fileURLToPath(import.meta.url)) main(process.argv.slice(2));
