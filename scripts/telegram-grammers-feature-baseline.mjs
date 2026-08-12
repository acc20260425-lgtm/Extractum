import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const artifactPath = path.join(
  repoRoot,
  "src/lib/telegram-grammers-feature-baseline.json",
);
const directOwnerName = "extractum-telegram";
const packageNames = [
  "grammers-client",
  "grammers-mtsender",
  "grammers-session",
  "grammers-tl-types",
];

/**
 * @typedef {object} CargoMetadataPackage
 * @property {string} id
 * @property {string} name
 * @property {string} version
 * @property {string | null} [source]
 * @property {Record<string, unknown>} features
 */

/**
 * @typedef {object} CargoMetadataDependency
 * @property {string} pkg
 */

/**
 * @typedef {object} CargoMetadataNode
 * @property {string} id
 * @property {CargoMetadataDependency[]} [deps]
 * @property {unknown} [features]
 */

/**
 * @typedef {object} CargoMetadata
 * @property {CargoMetadataPackage[]} [packages]
 * @property {{ nodes?: CargoMetadataNode[] }} [resolve]
 */

/**
 * @typedef {object} FeatureBaselinePackage
 * @property {string} name
 * @property {string[]} required
 * @property {string[]} forbidden
 * @property {string[]} universe
 */

/**
 * @typedef {object} ResolvedGrammersPackage
 * @property {string} name
 * @property {string} version
 * @property {string | null} source
 */

/**
 * @typedef {object} FeatureBaseline
 * @property {number} schemaVersion
 * @property {FeatureBaselinePackage[]} directPackages
 * @property {ResolvedGrammersPackage[]} resolvedPackages
 */

/**
 * @param {string} message
 * @returns {never}
 */
function fail(message) {
  throw new Error(`Telegram Grammers feature baseline: ${message}`);
}

/**
 * @param {unknown} values
 * @param {string} label
 * @returns {string[]}
 */
function sortedUnique(values, label) {
  if (!Array.isArray(values) || values.some((value) => typeof value !== "string")) {
    fail(`${label} is not a string array`);
  }
  const sorted = [.../** @type {string[]} */ (values)].sort();
  if (new Set(sorted).size !== sorted.length) {
    fail(`${label} contains duplicates`);
  }
  return sorted;
}

/**
 * @returns {CargoMetadata}
 */
function loadMetadata() {
  const result = spawnSync(
    "cargo",
    [
      "metadata",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--locked",
      "--format-version",
      "1",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 256 * 1024 * 1024,
      shell: false,
    },
  );
  if (result.error) fail(`unable to start locked Cargo metadata: ${result.error.message}`);
  if (result.status !== 0) {
    fail(`locked Cargo metadata failed: ${(result.stderr || "").trim()}`);
  }
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    fail(
      `locked Cargo metadata returned malformed JSON: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
}

/**
 * @param {CargoMetadata | null} [metadata]
 * @returns {FeatureBaseline}
 */
export function generateFeatureBaseline(metadata = loadMetadata()) {
  if (
    !metadata
    || !Array.isArray(metadata.packages)
    || !metadata.resolve
    || !Array.isArray(metadata.resolve.nodes)
  ) {
    fail("missing package graph");
  }
  const metadataPackages = metadata.packages;
  const metadataNodes = metadata.resolve.nodes;
  const directOwnerPackages = metadataPackages.filter(
    (candidate) => candidate?.name === directOwnerName,
  );
  if (directOwnerPackages.length !== 1) {
    fail(
      `expected one ${directOwnerName} package, found ${directOwnerPackages.length}`,
    );
  }
  const directOwnerNodes = metadataNodes.filter(
    (candidate) => candidate?.id === directOwnerPackages[0].id,
  );
  if (directOwnerNodes.length !== 1) {
    fail(
      `expected one resolved ${directOwnerName} node, found ${directOwnerNodes.length}`,
    );
  }
  const directOwnerNode = directOwnerNodes[0];
  if (!Array.isArray(directOwnerNode.deps)) {
    fail(`missing resolved ${directOwnerName} node`);
  }

  const directPackageIds = directOwnerNode.deps
    .map((dependency) => dependency?.pkg)
    .filter((packageId) => {
      const matched = metadataPackages.find(
        (candidate) => candidate?.id === packageId,
      );
      return matched !== undefined && packageNames.includes(matched.name);
    });
  if (
    directPackageIds.length !== packageNames.length
    || new Set(directPackageIds).size !== directPackageIds.length
  ) {
    fail(
      `${directOwnerName} must have four distinct direct Grammers package IDs`,
    );
  }

  const directPackages = directPackageIds.map((packageId) => {
    const matchingPackages = metadataPackages.filter(
      (candidate) => candidate?.id === packageId,
    );
    const matchingNodes = metadataNodes.filter(
      (candidate) => candidate?.id === packageId,
    );
    if (matchingPackages.length !== 1 || matchingNodes.length !== 1) {
      fail(`package graph is ambiguous for ${packageId}`);
    }
    const packageRecord = matchingPackages[0];
    if (!packageNames.includes(packageRecord.name)) {
      fail(`unexpected direct Grammers package ${packageRecord.name}`);
    }
    if (
      !packageRecord.features
      || Array.isArray(packageRecord.features)
      || typeof packageRecord.features !== "object"
    ) {
      fail(`missing feature universe for ${packageRecord.name}`);
    }
    for (const [feature, definition] of Object.entries(packageRecord.features)) {
      if (
        !Array.isArray(definition)
        || definition.some((value) => typeof value !== "string")
      ) {
        fail(
          `${packageRecord.name} feature definition ${feature} is not a string array`,
        );
      }
    }
    const universe = sortedUnique(
      Object.keys(packageRecord.features),
      `${packageRecord.name} feature universe`,
    );
    const required = sortedUnique(
      matchingNodes[0].features,
      `${packageRecord.name} resolved features`,
    );
    const unknownRequired = required.filter(
      (feature) => !universe.includes(feature),
    );
    if (unknownRequired.length !== 0) {
      fail(
        `${packageRecord.name} resolves features outside its universe: ${unknownRequired.join(", ")}`,
      );
    }
    const forbidden = universe.filter((feature) => !required.includes(feature));
    return {
      name: packageRecord.name,
      required,
      forbidden,
      universe,
    };
  }).sort((left, right) => left.name.localeCompare(right.name));

  if (
    directPackages.map(({ name }) => name).join("\n")
    !== [...packageNames].sort().join("\n")
  ) {
    fail("direct Grammers package set drifted");
  }
  const resolvedPackages = metadataPackages
    .filter((candidate) => candidate?.name?.startsWith("grammers-"))
    .map((candidate) => {
      if (typeof candidate.version !== "string") {
        fail(`missing version for ${candidate.name}`);
      }
      if (candidate.source !== null && typeof candidate.source !== "string") {
        fail(`invalid source for ${candidate.name}`);
      }
      return {
        name: candidate.name,
        version: candidate.version,
        source: candidate.source ?? null,
      };
    })
    .sort((left, right) =>
      left.name.localeCompare(right.name)
      || left.version.localeCompare(right.version)
      || String(left.source).localeCompare(String(right.source)));
  return {
    schemaVersion: 2,
    directPackages,
    resolvedPackages,
  };
}

/**
 * @param {unknown} value
 * @returns {string}
 */
function serialize(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

/**
 * @param {string} mode
 * @returns {void}
 */
function run(mode) {
  const rendered = serialize(generateFeatureBaseline());
  if (mode === "--write") {
    writeFileSync(artifactPath, rendered, "utf8");
    return;
  }
  if (mode === "--check") {
    /** @type {string} */
    let existing;
    try {
      existing = readFileSync(artifactPath, "utf8");
    } catch (error) {
      fail(
        `artifact is missing or unreadable: ${
          error instanceof Error ? error.message : String(error)
        }`,
      );
    }
    if (existing !== rendered) {
      fail("artifact content or formatting drifted; run with --write");
    }
    return;
  }
  fail("expected exactly one argument: --write or --check");
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : "";
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 3) {
      fail("expected exactly one argument: --write or --check");
    }
    run(process.argv[2]);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
