import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const artifactPath = path.join(
  repoRoot,
  "src/lib/telegram-grammers-feature-baseline.json",
);
const revision = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa";
const exactSource =
  `git+https://codeberg.org/Lonami/grammers?rev=${revision}#${revision}`;
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
 * @property {string} [source]
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
 * @typedef {object} FeatureBaseline
 * @property {number} schemaVersion
 * @property {string} revision
 * @property {FeatureBaselinePackage[]} packages
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
  const extractumPackages = metadataPackages.filter(
    (candidate) => candidate?.name === "extractum",
  );
  if (extractumPackages.length !== 1) {
    fail(`expected one extractum package, found ${extractumPackages.length}`);
  }
  const extractumNodes = metadataNodes.filter(
    (candidate) => candidate?.id === extractumPackages[0].id,
  );
  if (extractumNodes.length !== 1) {
    fail(
      `expected one resolved extractum node, found ${extractumNodes.length}`,
    );
  }
  const extractumNode = extractumNodes[0];
  if (!Array.isArray(extractumNode.deps)) {
    fail("missing resolved extractum node");
  }

  const directPackageIds = extractumNode.deps
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
    fail("extractum must have four distinct direct Grammers package IDs");
  }

  const packages = directPackageIds.map((packageId) => {
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
    if (packageRecord.source !== exactSource) {
      fail(`source drifted for ${packageRecord.name}`);
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
    packages.map(({ name }) => name).join("\n")
    !== [...packageNames].sort().join("\n")
  ) {
    fail("direct Grammers package set drifted");
  }
  return {
    schemaVersion: 1,
    revision,
    packages,
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
