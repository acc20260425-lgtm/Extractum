import { existsSync, readFileSync, readdirSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const artifactDir = "research/gemini_browser_adapter/artifacts";
const inputPath = path.join(artifactDir, "playwright-results.json");
const matrixPath = "research/gemini_browser_adapter/matrix-cases.json";
const outputPath = path.join(artifactDir, "matrix-report.md");
const matrixResultDir = path.join(artifactDir, "matrix");

if (!existsSync(matrixPath)) {
  console.error(`Missing matrix metadata at ${matrixPath}`);
  process.exit(1);
}

const matrixDefinition = JSON.parse(readFileSync(matrixPath, "utf8"));
const expectedPairs = matrixDefinition.adapterVariants.flatMap((variant) =>
  matrixDefinition.scenarios.map((scenario) => `${variant} / ${scenario.id}`),
);

function collectSpecs(suite, rows = []) {
  for (const spec of suite.specs || []) {
    const tests = spec.tests || [];
    for (const test of tests) {
      const result = test.results?.[0];
      rows.push({
        title: [...(suite.titlePath || []), spec.title].filter(Boolean).join(" / "),
        status: test.outcome || result?.status || "unknown",
        duration: result?.duration || 0,
      });
    }
  }
  for (const child of suite.suites || []) {
    collectSpecs({ ...child, titlePath: [...(suite.titlePath || []), child.title] }, rows);
  }
  return rows;
}

function collectResultFiles(dir, files = []) {
  if (!existsSync(dir)) return files;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) collectResultFiles(entryPath, files);
    else if (entry.name === "result.json") files.push(entryPath);
  }
  return files;
}

function average(values) {
  if (values.length === 0) return 0;
  return Math.round(values.reduce((sum, value) => sum + value, 0) / values.length);
}

function isSuccessStatus(status) {
  return status === "ok" || status === "ready";
}

if (!existsSync(inputPath)) {
  console.error(`Missing Playwright JSON results at ${inputPath}`);
  process.exit(1);
}

const json = JSON.parse(readFileSync(inputPath, "utf8"));
const rows = collectSpecs({ suites: json.suites || [], titlePath: [] });
const passed = rows.filter((row) => row.status === "expected" || row.status === "passed").length;
const failed = rows.length - passed;
const worst = rows.reduce((max, row) => Math.max(max, row.duration), 0);
const missingPairs = expectedPairs.filter((pair) => !rows.some((row) => row.title.endsWith(pair)));
const observedPairs = expectedPairs.length - missingPairs.length;
const resultRows = collectResultFiles(matrixResultDir).map((filePath) => JSON.parse(readFileSync(filePath, "utf8")));
const resultPairs = new Set(resultRows.map((row) => `${row.variant} / ${row.scenarioId}`));
const missingResultPairs = expectedPairs.filter((pair) => !resultPairs.has(pair));
const successCount = resultRows.filter((row) => isSuccessStatus(row.status)).length;
const okCount = resultRows.filter((row) => row.status === "ok").length;
const readyCount = resultRows.filter((row) => row.status === "ready").length;
const cleanTypedFailureCount = resultRows.filter((row) => !isSuccessStatus(row.status) && row.expectedStatuses.includes(row.status)).length;
const unexpectedFailureCount = resultRows.filter((row) => row.unexpectedStatus).length;
const timeoutOrHangCount = resultRows.filter((row) => row.timeoutOrHang).length;
const falseCompletionCount = resultRows.filter((row) => row.falseCompletion).length;
const artifactIncompleteCount = resultRows.filter((row) =>
  Object.entries(row.expectedArtifacts).some(([name, required]) => required && !row.artifacts[name]),
).length;
const averageElapsedMs = average(resultRows.map((row) => row.elapsedMs));
const worstElapsedMs = resultRows.reduce((max, row) => Math.max(max, row.elapsedMs ?? 0), 0);
const variants = matrixDefinition.adapterVariants.map((variant) => {
  const rowsForVariant = resultRows.filter((row) => row.variant === variant);
  return {
    variant,
    success: rowsForVariant.filter((row) => isSuccessStatus(row.status)).length,
    ok: rowsForVariant.filter((row) => row.status === "ok").length,
    ready: rowsForVariant.filter((row) => row.status === "ready").length,
    typedFailure: rowsForVariant.filter((row) => !isSuccessStatus(row.status) && row.expectedStatuses.includes(row.status)).length,
    unexpected: rowsForVariant.filter((row) => row.unexpectedStatus).length,
    falseCompletion: rowsForVariant.filter((row) => row.falseCompletion).length,
    averageElapsedMs: average(rowsForVariant.map((row) => row.elapsedMs)),
    worstElapsedMs: rowsForVariant.reduce((max, row) => Math.max(max, row.elapsedMs ?? 0), 0),
  };
});

const report = [
  "# Gemini Browser Adapter Matrix Report",
  "",
  `Generated: ${new Date().toISOString()}`,
  "",
  `Total tests: ${rows.length}`,
  `Passed tests: ${passed}`,
  `Failed or unexpected tests: ${failed}`,
  `Expected matrix pairs: ${expectedPairs.length}`,
  `Observed matrix pairs: ${observedPairs}`,
  `Missing matrix pairs: ${missingPairs.length}`,
  `Missing result files: ${missingResultPairs.length}`,
  `Worst Playwright duration ms: ${worst}`,
  "",
  "## Adapter Result Metrics",
  "",
  `Success count: ${successCount}`,
  `OK count: ${okCount}`,
  `Ready count: ${readyCount}`,
  `Clean typed failure count: ${cleanTypedFailureCount}`,
  `Unexpected failure count: ${unexpectedFailureCount}`,
  `Timeout/hang count: ${timeoutOrHangCount}`,
  `Required artifact incomplete count: ${artifactIncompleteCount}`,
  `False completion count: ${falseCompletionCount}`,
  `Average elapsed ms: ${averageElapsedMs}`,
  `Worst elapsed ms: ${worstElapsedMs}`,
  "",
  "| Variant | Success | OK | Ready | Clean Typed Failure | Unexpected | False Completion | Avg ms | Worst ms |",
  "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
  ...variants.map((row) =>
    `| ${row.variant} | ${row.success} | ${row.ok} | ${row.ready} | ${row.typedFailure} | ${row.unexpected} | ${row.falseCompletion} | ${row.averageElapsedMs} | ${row.worstElapsedMs} |`,
  ),
  "",
  "## Matrix Coverage",
  "",
  missingPairs.length === 0 && missingResultPairs.length === 0
    ? "All expected variant/scenario pairs and result files were present."
    : [...missingPairs.map((pair) => `- Missing Playwright row: ${pair}`), ...missingResultPairs.map((pair) => `- Missing result file: ${pair}`)].join("\n"),
  "",
  "| Test | Status | Duration ms |",
  "| --- | --- | ---: |",
  ...rows.map((row) => `| ${row.title.replaceAll("|", "\\|")} | ${row.status} | ${row.duration} |`),
  "",
].join("\n");

await mkdir(artifactDir, { recursive: true });
await writeFile(outputPath, report, "utf8");
console.log(`Wrote ${outputPath}`);
if (failed > 0 || missingPairs.length > 0 || missingResultPairs.length > 0 || unexpectedFailureCount > 0 || artifactIncompleteCount > 0 || falseCompletionCount > 0) process.exit(1);
