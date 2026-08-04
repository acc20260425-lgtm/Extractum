import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  createCliGitMetadata,
  discoverSourceReaders,
  discoverTestDeclarations,
  validateSourceContractLedger,
} from "./testing/extract-source-contract-ledger.mjs";
import { createRepositoryIndex } from "./testing/repository-index.mjs";
import { evaluateRule, registeredRuleIds } from "./testing/repository-rules.mjs";
import { loadRunnerCensus, runTransitionValidation, validateCensusSchema, validateLiveRunnerCensus } from "./testing/testing-transition.mjs";
import { createVerifySteps } from "./verify.mjs";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));

function normalizePath(value) {
  return value.replaceAll("\\", "/");
}

function loadSourceContractLedger(root) {
  return JSON.parse(readFileSync(path.join(root, "testing", "source-contract-ledger.json"), "utf8"));
}

function listedTestCounts(result, packageName, violations) {
  if (!result || result.error || result.exitCode !== 0) {
    violations.push(`${packageName}: Cargo test list failed${result?.error ? `: ${result.error.message ?? result.error}` : ""}`);
    return new Map();
  }
  const counts = new Map();
  for (const line of String(result.stdout ?? "").split(/\r?\n/)) {
    const match = /^(.*): test$/.exec(line.trim());
    if (!match) continue;
    counts.set(match[1], (counts.get(match[1]) ?? 0) + 1);
  }
  return counts;
}

function verifyOwnsCargoPackage(verifySteps, packageName) {
  return verifySteps.some((step) => {
    if (step?.command !== "cargo" || step?.args?.[0] !== "test") return false;
    const packageIndex = step.args.indexOf("-p");
    return step.args.includes("--workspace") || (packageIndex >= 0 && step.args[packageIndex + 1] === packageName);
  });
}

export function evaluateTelegramCargoTestIdentityOwnership({ authority, listResults, verifySteps = [] }) {
  const violations = [];
  const stagedDeclared = [...(authority?.preNewStaged ?? []), ...(authority?.phase8BNewStaged ?? [])];
  const expectedByPackage = {
    extractum: [...(authority?.preNewApp ?? []), ...(authority?.phase8BNewApp ?? [])],
    "extractum-telegram": stagedDeclared.map((identity) => identity.replace(/^telegram_impl::/, "")),
  };
  const countsByPackage = {
    extractum: listedTestCounts(listResults?.extractum, "extractum", violations),
    "extractum-telegram": listedTestCounts(listResults?.["extractum-telegram"], "extractum-telegram", violations),
  };

  for (const packageName of ["extractum", "extractum-telegram"]) {
    if (!verifyOwnsCargoPackage(verifySteps, packageName)) {
      violations.push(`missing verify owner for ${packageName}`);
    }
    const otherPackage = packageName === "extractum" ? "extractum-telegram" : "extractum";
    for (const identity of expectedByPackage[packageName]) {
      const count = countsByPackage[packageName].get(identity) ?? 0;
      if (count === 0) violations.push(`${packageName}: missing declared identity ${identity}`);
      else if (count !== 1) violations.push(`${packageName}: duplicate declared identity ${identity}`);
      if ((countsByPackage[otherPackage].get(identity) ?? 0) !== 0) {
        violations.push(`${otherPackage}: wrong package for declared identity ${identity}`);
      }
    }
  }
  for (const identity of stagedDeclared) {
    if ((countsByPackage.extractum.get(identity) ?? 0) !== 0) {
      violations.push(`extractum: declared staged identity must be absent after extraction: ${identity}`);
    }
  }
  return violations;
}

function ledgerReplacementIds(ledger) {
  return (ledger?.rows ?? []).flatMap((row) => [
    ...(row?.replacementIds ?? []),
    ...(row?.subgroups ?? []).flatMap((subgroup) => subgroup?.replacementIds ?? []),
  ]);
}

export function collectTrackedTestSources({ root, tracked, vitestFiles = {}, authorizedMissingPaths = new Set(), readSource = readFileSync }) {
  const tests = [];
  const issues = [];
  const vitestPaths = new Set(Object.values(vitestFiles).flat().map(normalizePath));
  for (const item of tracked.filter((candidate) => candidate.endsWith(".test.ts") || vitestPaths.has(candidate))) {
    try {
      tests.push({ path: item, source: readSource(path.join(root, item), "utf8") });
    } catch (error) {
      if (error?.code === "ENOENT" && authorizedMissingPaths.has(item)) continue;
      if (error?.code === "ENOENT") {
        issues.push(`missing tracked test without ledger ownership: ${item}`);
        continue;
      }
      throw error;
    }
  }
  return { tests, issues };
}

export function createLedgerLiveCensus({ census, runnerResult }) {
  return {
    vitestOwners: census.vitestOwners,
    vitestFiles: runnerResult?.vitestFiles ?? {},
  };
}

export function collectTelegramCargoReplacementEvidence({ ledger, authority, verifySteps = [], runCargoList }) {
  const referenced = new Set(ledgerReplacementIds(ledger));
  const packages = new Set();
  for (const id of referenced) {
    const match = /^test:cargo:([^:]+)::/.exec(id);
    if (match) packages.add(match[1]);
  }
  const toolId = "tool:telegram-cargo-test-identity-ownership";
  if (referenced.has(toolId)) {
    packages.add("extractum");
    packages.add("extractum-telegram");
  }

  const listResults = {};
  for (const packageName of [...packages].sort()) listResults[packageName] = runCargoList(packageName);
  const issues = [];
  const countsByPackage = {};
  for (const packageName of packages) {
    countsByPackage[packageName] = listedTestCounts(listResults[packageName], packageName, issues);
  }
  const resolvedReplacementIds = new Set();
  for (const id of referenced) {
    const match = /^test:cargo:([^:]+)::(.+)$/.exec(id);
    if (!match) continue;
    const [, packageName, identity] = match;
    if (verifyOwnsCargoPackage(verifySteps, packageName)
      && (countsByPackage[packageName]?.get(identity) ?? 0) === 1) {
      resolvedReplacementIds.add(id);
    }
  }
  if (referenced.has(toolId)) {
    const toolIssues = evaluateTelegramCargoTestIdentityOwnership({ authority, listResults, verifySteps });
    for (const issue of toolIssues) if (!issues.includes(issue)) issues.push(issue);
    if (toolIssues.length === 0) resolvedReplacementIds.add(toolId);
  }
  return { issues, listResults, resolvedReplacementIds };
}

function validateLiveSourceContractLedger(root, ledger, liveCensus) {
  const tracked = execFileSync("git", ["ls-files", "-z"], { cwd: root, encoding: "utf8" })
    .split("\0").filter(Boolean).map(normalizePath);
  const ignored = execFileSync("git", ["ls-files", "--others", "--ignored", "--exclude-standard", "-z"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  }).split("\0").filter(Boolean).map(normalizePath);
  const authorizedMissingPaths = new Set((ledger.rows ?? []).map((row) => row.path));
  const { tests, issues: discoveryIssues } = collectTrackedTestSources({
    root,
    tracked,
    vitestFiles: liveCensus.vitestFiles,
    authorizedMissingPaths,
  });
  const runnerTitlesByPath = {};
  for (const row of ledger.rows ?? []) {
    if (!row?.manual?.runnerTitles) continue;
    const titles = runnerTitlesByPath[row.path] ??= [];
    for (const title of row.manual.runnerTitles) if (!titles.includes(title)) titles.push(title);
  }
  for (const titles of Object.values(runnerTitlesByPath)) titles.sort();
  const declarationInventory = discoverTestDeclarations(tests);
  const sourceReaders = discoverSourceReaders(
    tests,
    createCliGitMetadata(new Set(tracked), new Set(ignored)),
  );
  const verifySteps = createVerifySteps({ npmExecPath: "npm-cli.js" });
  const referencedIds = new Set(ledgerReplacementIds(ledger));
  const index = createRepositoryIndex({ root });
  const resolvedReplacementIds = new Set();
  const evidenceIssues = [];
  for (const id of registeredRuleIds) {
    if (!referencedIds.has(id)) continue;
    const result = evaluateRule({ id, index });
    if (result.violations.length === 0) resolvedReplacementIds.add(id);
    else evidenceIssues.push(...result.violations.map((violation) => `${id}: ${violation}`));
  }
  const needsCargoEvidence = [...referencedIds].some((id) => id.startsWith("test:cargo:") || id === "tool:telegram-cargo-test-identity-ownership");
  if (needsCargoEvidence) {
    const cargoEvidence = collectTelegramCargoReplacementEvidence({
      ledger,
      authority: index.getJson("src/lib/telegram-8b-test-identities.json"),
      verifySteps,
      runCargoList(packageName) {
        try {
          const stdout = execFileSync("cargo", [
            "test", "--manifest-path", "src-tauri/Cargo.toml", "-p", packageName, "--lib", "--", "--list",
          ], { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024, windowsHide: true });
          return { exitCode: 0, stdout };
        } catch (error) {
          return {
            exitCode: Number.isInteger(error?.status) ? error.status : 1,
            stdout: error?.stdout?.toString?.() ?? "",
            error,
          };
        }
      },
    });
    for (const id of cargoEvidence.resolvedReplacementIds) resolvedReplacementIds.add(id);
    evidenceIssues.push(...cargoEvidence.issues);
  }
  const result = validateSourceContractLedger({
    ledger,
    declarationInventory,
    sourceReaders,
    runnerTitlesByPath,
    liveCensus,
    verifySteps,
    resolvedReplacementIds,
  });
  return {
    issues: [...discoveryIssues, ...result.issues, ...evidenceIssues],
    summary: `Source-contract ledger: ${ledger.rows?.length ?? 0} rows, ${result.rows.filter((row) => row.state === "open").length} open`,
  };
}

export async function validateTestingTransition({ root = repoRoot, stdout = process.stdout, stderr = process.stderr } = {}) {
  let census;
  try {
    census = loadRunnerCensus(root);
  } catch (error) {
    stderr.write(`unable to load runner census: ${error.message}\n`);
    return { exitCode: 1 };
  }
  const schemaIssues = validateCensusSchema(census);
  let liveRunnerCensus;
  return runTransitionValidation({
    repoRoot: root,
    stdout,
    stderr,
    checks: [
      async () => {
        if (schemaIssues.length) return schemaIssues;
        liveRunnerCensus = await validateLiveRunnerCensus(root, census);
        return liveRunnerCensus;
      },
      async () => {
        try {
          return validateLiveSourceContractLedger(
            root,
            loadSourceContractLedger(root),
            createLedgerLiveCensus({ census, runnerResult: liveRunnerCensus }),
          );
        } catch (error) {
          return [`unable to load or validate source-contract ledger: ${error.message}`];
        }
      },
    ],
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const result = await validateTestingTransition();
  process.exit(result.exitCode);
}
