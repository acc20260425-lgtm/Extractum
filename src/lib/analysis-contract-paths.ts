import { existsSync, readFileSync, realpathSync } from "node:fs";
import path from "node:path";

export type AnalysisContractSource = {
  before: string;
  after: { owner: "app" | "crate"; path: string };
};

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const appRoot = path.join(repositoryRoot, "src-tauri/src/analysis");
const crateRoot = path.join(
  repositoryRoot,
  "src-tauri/crates/extractum-analysis/src",
);
const crateManifest = path.join(
  repositoryRoot,
  "src-tauri/crates/extractum-analysis/Cargo.toml",
);

export function normalizeAnalysisContractSourceText(source: string): string {
  return source.replace(/\r\n?/g, "\n");
}

export function isAnalysisCrateExtracted(): boolean {
  const extracted = existsSync(crateManifest);
  assertLayout(extracted);
  return extracted;
}

function assertLayout(extracted: boolean): void {
  const appExists = existsSync(appRoot);
  const crateExists = existsSync(crateRoot);

  if (!appExists) {
    throw new Error("Unexpected analysis layout: application analysis root is missing");
  }
  if (extracted !== crateExists) {
    throw new Error(
      extracted
        ? "Unexpected analysis layout: crate manifest exists without its source root"
        : "Unexpected analysis layout: crate source root exists before its manifest",
    );
  }
}

function readSelected(root: string, relativePath: string): string {
  if (!relativePath || path.isAbsolute(relativePath)) {
    throw new Error(`Analysis contract path must be relative: ${relativePath}`);
  }

  const resolvedRoot = realpathSync(root);
  const selected = path.resolve(resolvedRoot, relativePath);
  const relative = path.relative(resolvedRoot, selected);
  if (
    relative === "" ||
    relative.startsWith(`..${path.sep}`) ||
    relative === ".." ||
    path.isAbsolute(relative)
  ) {
    throw new Error(`Analysis contract path escapes selected root: ${relativePath}`);
  }
  if (!existsSync(selected)) {
    throw new Error(`Selected analysis contract source is missing: ${relativePath}`);
  }

  const realSelected = realpathSync(selected);
  const realRelative = path.relative(resolvedRoot, realSelected);
  if (
    realRelative.startsWith(`..${path.sep}`) ||
    realRelative === ".." ||
    path.isAbsolute(realRelative)
  ) {
    throw new Error(`Analysis contract path escapes selected root: ${relativePath}`);
  }
  return normalizeAnalysisContractSourceText(
    readFileSync(realSelected, "utf8"),
  );
}

export function readAppAnalysisSource(relativePath: string): string {
  isAnalysisCrateExtracted();
  return readSelected(appRoot, relativePath);
}

export function readCrateAnalysisSource(relativePath: string): string {
  const extracted = isAnalysisCrateExtracted();
  if (!extracted) {
    throw new Error("Cannot read an analysis crate path before extraction");
  }
  return readSelected(crateRoot, relativePath);
}

export function readAnalysisContractSource(
  source: AnalysisContractSource,
): string {
  const extracted = isAnalysisCrateExtracted();
  if (!extracted) return readSelected(appRoot, source.before);
  return source.after.owner === "app"
    ? readSelected(appRoot, source.after.path)
    : readSelected(crateRoot, source.after.path);
}
