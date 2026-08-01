import { createHash } from "node:crypto";
import {
  lstatSync,
  readFileSync,
  readdirSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootRelativePath = "src-tauri/src/telegram_impl";
const artifactRelativePath = "src/lib/telegram-8b-staging-sha256.json";
const exactRelativePaths = [
  "dto.rs",
  "error.rs",
  "lib.rs",
  "live/avatar.rs",
  "live/messages.rs",
  "live/mod.rs",
  "live/peer.rs",
  "live/topics.rs",
  "media.rs",
  "runtime.rs",
  "session.rs",
  "takeout/export_dc.rs",
  "takeout/forum_topics.rs",
  "takeout/mod.rs",
  "takeout/operations.rs",
  "takeout/pagination.rs",
  "takeout/raw_parse.rs",
  "takeout/transport.rs",
  "takeout/types.rs",
];

function fail(message) {
  throw new Error(`Telegram Phase 8B staging SHA-256: ${message}`);
}

function normalizedAbsolute(absolutePath) {
  const resolved = path.resolve(absolutePath);
  return process.platform === "win32" ? resolved.toLowerCase() : resolved;
}

function pathsEqual(left, right) {
  return normalizedAbsolute(left) === normalizedAbsolute(right);
}

function assertContained(parent, candidate, label) {
  const relative = path.relative(parent, candidate);
  if (
    relative === ""
    || (!path.isAbsolute(relative)
      && relative !== ".."
      && !relative.startsWith(`..${path.sep}`))
  ) {
    return;
  }
  fail(`${label} escapes its allowed root`);
}

function resolveRepositoryRoot() {
  const scriptPath = realpathSync(fileURLToPath(import.meta.url));
  const candidate = realpathSync(path.resolve(path.dirname(scriptPath), ".."));
  const expectedScriptPath = path.join(
    candidate,
    "scripts",
    "telegram-staging-sha256.mjs",
  );
  if (!pathsEqual(scriptPath, realpathSync(expectedScriptPath))) {
    fail("script path does not resolve below the repository scripts directory");
  }
  for (const marker of ["package.json", "src-tauri/Cargo.toml"]) {
    const markerPath = path.join(candidate, ...marker.split("/"));
    const markerStat = lstatSync(markerPath);
    if (markerStat.isSymbolicLink() || !markerStat.isFile()) {
      fail(`repository marker is not a regular file: ${marker}`);
    }
    assertContained(candidate, realpathSync(markerPath), marker);
  }
  return candidate;
}

function assertSafeExistingPath(repositoryRoot, absolutePath, kind, label) {
  assertContained(repositoryRoot, absolutePath, label);
  const relative = path.relative(repositoryRoot, absolutePath);
  let cursor = repositoryRoot;
  for (const segment of relative.split(path.sep).filter(Boolean)) {
    cursor = path.join(cursor, segment);
    const stat = lstatSync(cursor);
    if (stat.isSymbolicLink()) fail(`${label} contains a symbolic link`);
  }
  const stat = lstatSync(absolutePath);
  if (kind === "directory" ? !stat.isDirectory() : !stat.isFile()) {
    fail(`${label} is not a regular ${kind}`);
  }
  const canonical = realpathSync(absolutePath);
  assertContained(repositoryRoot, canonical, `${label} realpath`);
  if (!pathsEqual(absolutePath, canonical)) {
    fail(`${label} resolves through a redirected path`);
  }
}

function collectTreeFiles(repositoryRoot, treeRoot, directory = treeRoot) {
  assertSafeExistingPath(repositoryRoot, directory, "directory", "staging tree");
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) {
      fail("staging tree contains a symbolic link");
    }
    if (entry.isDirectory()) {
      files.push(...collectTreeFiles(repositoryRoot, treeRoot, absolutePath));
      continue;
    }
    if (!entry.isFile()) {
      fail("staging tree contains an unsupported filesystem entry");
    }
    assertSafeExistingPath(repositoryRoot, absolutePath, "file", "staged file");
    files.push(path.relative(treeRoot, absolutePath).split(path.sep).join("/"));
  }
  return files;
}

function buildManifest(repositoryRoot) {
  const treeRoot = path.join(repositoryRoot, ...rootRelativePath.split("/"));
  assertSafeExistingPath(repositoryRoot, treeRoot, "directory", "staging root");
  const observedPaths = collectTreeFiles(repositoryRoot, treeRoot).sort();
  if (
    observedPaths.length !== exactRelativePaths.length
    || observedPaths.some((relativePath, index) =>
      relativePath !== exactRelativePaths[index]
    )
  ) {
    fail(
      `staging tree paths drifted; expected ${exactRelativePaths.join(", ")}; observed ${observedPaths.join(", ")}`,
    );
  }
  const files = exactRelativePaths.map((relativePath) => {
    if (
      relativePath.includes("\\")
      || relativePath.split("/").some((segment) => !segment || segment === "." || segment === "..")
    ) {
      fail(`unsafe staged relative path: ${relativePath}`);
    }
    const absolutePath = path.join(treeRoot, ...relativePath.split("/"));
    assertSafeExistingPath(repositoryRoot, absolutePath, "file", relativePath);
    return {
      path: relativePath,
      sha256: createHash("sha256").update(readFileSync(absolutePath)).digest("hex"),
    };
  });
  return {
    schemaVersion: 1,
    algorithm: "sha256",
    root: rootRelativePath,
    files,
  };
}

function serialize(manifest) {
  return `${JSON.stringify(manifest, null, 2)}\n`;
}

function readAndValidateArtifact(artifactPath, expectedManifest) {
  let source;
  try {
    source = readFileSync(artifactPath, "utf8");
  } catch (error) {
    fail(
      `artifact is missing or unreadable: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
  let artifact;
  try {
    artifact = JSON.parse(source);
  } catch (error) {
    fail(
      `artifact is malformed JSON: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
  if (!artifact || Array.isArray(artifact) || typeof artifact !== "object") {
    fail("artifact root is not an object");
  }
  if (artifact.schemaVersion !== 1) fail("artifact schemaVersion drifted");
  if (artifact.algorithm !== "sha256") fail("artifact algorithm drifted");
  if (artifact.root !== rootRelativePath) fail("artifact root drifted");
  if (!Array.isArray(artifact.files)) fail("artifact files is not an array");
  const topLevelKeys = Object.keys(artifact);
  if (
    topLevelKeys.length !== 4
    || topLevelKeys.some(
      (key) => !["schemaVersion", "algorithm", "root", "files"].includes(key),
    )
  ) {
    fail("artifact top-level schema drifted");
  }
  const paths = [];
  for (const [index, record] of artifact.files.entries()) {
    if (!record || Array.isArray(record) || typeof record !== "object") {
      fail(`artifact record ${index + 1} is not an object`);
    }
    const keys = Object.keys(record);
    if (
      keys.length !== 2
      || keys.some((key) => !["path", "sha256"].includes(key))
      || typeof record.path !== "string"
      || typeof record.sha256 !== "string"
    ) {
      fail(`artifact record ${index + 1} schema drifted`);
    }
    if (!/^[0-9a-f]{64}$/.test(record.sha256)) {
      fail(`artifact record ${index + 1} has a non-lowercase SHA-256`);
    }
    paths.push(record.path);
  }
  if (new Set(paths).size !== paths.length) {
    fail("artifact contains duplicate records");
  }
  if (
    paths.length !== exactRelativePaths.length
    || paths.some((relativePath, index) => relativePath !== exactRelativePaths[index])
  ) {
    fail("artifact path set or order drifted");
  }
  if (source !== serialize(expectedManifest)) {
    fail("artifact bytes, hashes, or formatting drifted; run with --write");
  }
}

function run(mode) {
  const repositoryRoot = resolveRepositoryRoot();
  const artifactPath = path.join(
    repositoryRoot,
    ...artifactRelativePath.split("/"),
  );
  const artifactDirectory = path.dirname(artifactPath);
  assertSafeExistingPath(
    repositoryRoot,
    artifactDirectory,
    "directory",
    "artifact directory",
  );
  const manifest = buildManifest(repositoryRoot);
  if (mode === "--write") {
    try {
      assertSafeExistingPath(
        repositoryRoot,
        artifactPath,
        "file",
        "artifact",
      );
    } catch (error) {
      if (
        !(error instanceof Error)
        || !/ENOENT/.test(`${error.message} ${"cause" in error ? error.cause : ""}`)
      ) {
        throw error;
      }
    }
    writeFileSync(artifactPath, serialize(manifest), { encoding: "utf8", flag: "w" });
    return;
  }
  if (mode === "--check") {
    assertSafeExistingPath(repositoryRoot, artifactPath, "file", "artifact");
    readAndValidateArtifact(artifactPath, manifest);
    return;
  }
  fail("expected exactly one argument: --write or --check");
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : "";
if (pathsEqual(invokedPath, fileURLToPath(import.meta.url))) {
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
