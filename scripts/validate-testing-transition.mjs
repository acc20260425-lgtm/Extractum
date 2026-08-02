import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { loadRunnerCensus, runTransitionValidation, validateCensusSchema, validateLiveRunnerCensus } from "./testing/testing-transition.mjs";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));

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
    checks: [async () => schemaIssues.length ? schemaIssues : validateLiveRunnerCensus(root, census)],
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const result = await validateTestingTransition();
  process.exit(result.exitCode);
}
