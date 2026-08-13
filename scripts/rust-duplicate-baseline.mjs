import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const TARGET = "x86_64-pc-windows-msvc";
const CARGO_TREE_ARGUMENTS = [
  "tree", "--manifest-path", "src-tauri/Cargo.toml", "--locked",
  "--target", TARGET, "--workspace", "--prefix", "none", "--format", "{p}",
];

export function generateRustDuplicateBaseline(treeText) {
  const identities = new Map();
  for (const line of String(treeText).split(/\r?\n/)) {
    const match = /^(\S+) v(\S+)/.exec(line.trim());
    if (!match) continue;
    const [, name, version] = match;
    identities.set(`${name}@${version}`, { name, version });
  }
  const versionsByName = new Map();
  for (const { name, version } of identities.values()) {
    const versions = versionsByName.get(name) ?? new Set();
    versions.add(version);
    versionsByName.set(name, versions);
  }
  const duplicateCardinality = Object.fromEntries(
    [...versionsByName]
      .filter(([, versions]) => versions.size > 1)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, versions]) => [name, versions.size]),
  );
  return {
    schemaVersion: 1,
    target: TARGET,
    duplicateNameCount: Object.keys(duplicateCardinality).length,
    duplicateVersionInstanceCount: Object.values(duplicateCardinality).reduce((sum, count) => sum + count, 0),
    duplicateCardinality,
  };
}

function repositoryRoot() {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
}

function cargoTree(root) {
  return execFileSync("cargo", CARGO_TREE_ARGUMENTS, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    windowsHide: true,
  });
}

function main(cliArguments) {
  const [mode, outputPath] = cliArguments;
  if (cliArguments.length > 0 && (mode !== "--write" || !outputPath || cliArguments.length !== 2)) {
    throw new Error("Usage: node scripts/rust-duplicate-baseline.mjs [--write <path>]");
  }
  const baseline = generateRustDuplicateBaseline(cargoTree(repositoryRoot()));
  const serialized = `${JSON.stringify(baseline, null, 2)}\n`;
  if (mode === "--write") writeFileSync(path.resolve(repositoryRoot(), outputPath), serialized);
  else process.stdout.write(serialized);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main(process.argv.slice(2));
