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
import { loadRunnerCensus, runTransitionValidation, validateCensusSchema, validateLiveRunnerCensus } from "./testing/testing-transition.mjs";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));

function normalizePath(value) {
  return value.replaceAll("\\", "/");
}

function loadSourceContractLedger(root) {
  return JSON.parse(readFileSync(path.join(root, "testing", "source-contract-ledger.json"), "utf8"));
}

function validateLiveSourceContractLedger(root, ledger) {
  const tracked = execFileSync("git", ["ls-files", "-z"], { cwd: root, encoding: "utf8" })
    .split("\0").filter(Boolean).map(normalizePath);
  const ignored = execFileSync("git", ["ls-files", "--others", "--ignored", "--exclude-standard", "-z"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  }).split("\0").filter(Boolean).map(normalizePath);
  const tests = tracked
    .filter((item) => item.endsWith(".test.ts"))
    .map((item) => ({ path: item, source: readFileSync(path.join(root, item), "utf8") }));
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
  const result = validateSourceContractLedger({
    ledger,
    declarationInventory,
    sourceReaders,
    runnerTitlesByPath,
  });
  return {
    issues: result.issues,
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
  return runTransitionValidation({
    repoRoot: root,
    stdout,
    stderr,
    checks: [
      async () => schemaIssues.length ? schemaIssues : validateLiveRunnerCensus(root, census),
      async () => {
        try {
          return validateLiveSourceContractLedger(root, loadSourceContractLedger(root));
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
