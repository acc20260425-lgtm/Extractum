import { randomUUID as defaultRandomUUID, createHash } from "node:crypto";
import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runObservedCommand } from "./run-observation.mjs";

export const RUST_TEST_NAME = "readiness::tests::mark_failed_returns_failed_state";
export const WARMUP_RUNS = 1;
export const RETAINED_RUNS = 3;

const FAST_OWNER_EVIDENCE_LIMIT_MS = 13_000;
const OUTPUT_PATH = "artifacts/testing/slice-1/rust-feasibility.json";
const SOURCE_PATH = "src-tauri/src/readiness.rs";
const DELTA_EXPLANATION = "The test-build-over-check delta includes cfg(test), root test code, dev-dependencies, app-test-support, different compiler units, code generation, link, and cache/process noise; it is not pure link time.";
const INVALIDATING_SHAPES = new Set(["invalidatedCheck", "noRun", "endToEnd"]);
const cargoBase = ["--manifest-path", "src-tauri/Cargo.toml", "-p", "extractum", "--lib"];
const commands = {
  noopCheck: ["cargo", ["check", ...cargoBase, "--message-format=json"]],
  invalidatedCheck: ["cargo", ["check", ...cargoBase, "--message-format=json"]],
  noRun: ["cargo", ["test", ...cargoBase, "--no-run", "--message-format=json"]],
  endToEnd: ["cargo", ["test", ...cargoBase, RUST_TEST_NAME, "--", "--exact"]],
};

function scheduleEntry(phase, cohort, shape, pair = undefined) {
  return { phase, cohort, shape, ...(pair ? { pair } : {}) };
}

export function buildRunSchedule() {
  return [
    scheduleEntry("warmup", "warmup-noop", "noopCheck"),
    ...Array.from({ length: RETAINED_RUNS }, (_, index) => scheduleEntry("retained", "noop-controls", "noopCheck", `noop-${index + 1}`)),
    scheduleEntry("warmup", "warmup-invalidated", "invalidatedCheck"),
    scheduleEntry("warmup", "warmup-no-run", "noRun", "warmup-no-run"),
    scheduleEntry("warmup", "warmup-no-run", "directBinary", "warmup-no-run"),
    scheduleEntry("warmup", "warmup-end-to-end", "endToEnd"),
    scheduleEntry("retained", "retained-1", "invalidatedCheck"),
    scheduleEntry("retained", "retained-1", "noRun", "retained-1"),
    scheduleEntry("retained", "retained-1", "directBinary", "retained-1"),
    scheduleEntry("retained", "retained-1", "endToEnd"),
    scheduleEntry("retained", "retained-2", "endToEnd"),
    scheduleEntry("retained", "retained-2", "noRun", "retained-2"),
    scheduleEntry("retained", "retained-2", "directBinary", "retained-2"),
    scheduleEntry("retained", "retained-2", "invalidatedCheck"),
    scheduleEntry("retained", "retained-3", "noRun", "retained-3"),
    scheduleEntry("retained", "retained-3", "directBinary", "retained-3"),
    scheduleEntry("retained", "retained-3", "invalidatedCheck"),
    scheduleEntry("retained", "retained-3", "endToEnd"),
  ];
}

function packageIsExtractum(packageId) {
  return typeof packageId === "string" && /(?:#|\s)extractum(?:@|\s|$)/u.test(packageId);
}

function canonicalExecutable(executable, repoRoot) {
  if (typeof executable !== "string" || executable.length === 0) throw new Error("Missing root test executable");
  const canonicalTarget = path.resolve(repoRoot, "src-tauri", "target");
  const resolved = path.resolve(executable);
  const relative = path.relative(canonicalTarget, resolved);
  if (relative === "" || relative.startsWith(`..${path.sep}`) || relative === ".." || path.isAbsolute(relative)) {
    throw new Error("Root test executable is outside canonical src-tauri/target");
  }
  if (!/^extractum_lib-[^.]+(?:\.exe)?$/u.test(path.basename(resolved))) {
    throw new Error("Canonical executable is not the extractum_lib root test binary");
  }
  return resolved;
}

export function parseCargoArtifacts(text, expectation) {
  const { repoRoot, expectedFresh, expectedTestProfile, requireExecutable = false } = expectation ?? {};
  if (typeof repoRoot !== "string" || typeof expectedFresh !== "boolean" || typeof expectedTestProfile !== "boolean") {
    throw new TypeError("Cargo artifact expectation requires repoRoot, expectedFresh, and expectedTestProfile");
  }
  const messages = String(text).split(/\r?\n/u).filter((line) => line.trim().length > 0).map((line) => {
    try {
      return JSON.parse(line);
    } catch {
      throw new Error("Malformed Cargo JSON artifact output");
    }
  });
  const artifacts = messages.filter((message) => message?.reason === "compiler-artifact"
    && packageIsExtractum(message.package_id)
    && message.target?.name === "extractum_lib"
    && Array.isArray(message.target?.kind)
    && message.target.kind.includes("lib"));
  if (artifacts.length !== 1) throw new Error("Expected exactly one extractum/extractum_lib Cargo artifact");
  const artifact = artifacts[0];
  if (artifact.fresh !== expectedFresh) throw new Error(`Cargo artifact fresh proof did not equal ${expectedFresh}`);
  if (artifact.profile?.test !== expectedTestProfile) throw new Error(`Cargo artifact profile.test did not equal ${expectedTestProfile}`);
  const executable = artifact.executable == null
    ? null
    : canonicalExecutable(artifact.executable, repoRoot);
  if (requireExecutable && !executable) throw new Error("Missing root test executable");
  return {
    package: "extractum",
    target: "extractum_lib",
    fresh: artifact.fresh,
    testProfile: artifact.profile.test,
    executable,
  };
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

export function parseExactLibtest(text) {
  const output = String(text);
  const running = [...output.matchAll(/^running (\d+) tests?\s*$/gmu)];
  const summaries = [...output.matchAll(/^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; \d+ measured; \d+ filtered out;/gmu)];
  const namedPass = new RegExp(`^test ${escapeRegExp(RUST_TEST_NAME)} \\.\\.\\. ok\\s*$`, "mu").test(output);
  if (running.length !== 1
    || Number(running[0][1]) !== 1
    || summaries.length !== 1
    || summaries[0][1] !== "ok"
    || Number(summaries[0][2]) !== 1
    || Number(summaries[0][3]) !== 0
    || Number(summaries[0][4]) !== 0
    || !namedPass) {
    throw new Error("Expected exactly one passing test with no failures or ignored tests");
  }
  return { passed: 1, failed: 0, ignored: 0 };
}

export function appendMutation(original, token) {
  if (!Buffer.isBuffer(original)) throw new TypeError("original must be a Buffer");
  if (typeof token !== "string" || token.length === 0 || /[\r\n]/u.test(token)) throw new TypeError("mutation token");
  const newline = original.includes(Buffer.from("\r\n")) ? "\r\n" : "\n";
  const endsWithNewline = original.subarray(-Buffer.byteLength(newline)).equals(Buffer.from(newline));
  const marker = `// extractum-slice-1-probe:${token}`;
  const suffix = endsWithNewline ? `${marker}${newline}` : `${newline}${marker}`;
  return Buffer.concat([original, Buffer.from(suffix, "utf8")]);
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

async function restoreOriginal(filesystem, sourcePath, original, originalHash) {
  try {
    await filesystem.writeFile(sourcePath, original);
    const restored = Buffer.from(await filesystem.readFile(sourcePath));
    const restoredHash = sha256(restored);
    if (restoredHash === originalHash && restored.equals(original)) {
      return { verified: true, restoredHash };
    }
    try {
      await filesystem.writeFile(sourcePath, original);
      const recovered = Buffer.from(await filesystem.readFile(sourcePath));
      return { verified: false, restoredHash, recoveryVerified: recovered.equals(original) && sha256(recovered) === originalHash };
    } catch (recoveryError) {
      return { verified: false, restoredHash, recoveryVerified: false, recoveryError: errorMessage(recoveryError) };
    }
  } catch (error) {
    try {
      await filesystem.writeFile(sourcePath, original);
      const recovered = Buffer.from(await filesystem.readFile(sourcePath));
      return { verified: false, restoredHash: null, error: errorMessage(error), recoveryVerified: recovered.equals(original) && sha256(recovered) === originalHash };
    } catch (recoveryError) {
      return { verified: false, restoredHash: null, error: errorMessage(error), recoveryVerified: false, recoveryError: errorMessage(recoveryError) };
    }
  }
}

function median(values) {
  if (values.length !== RETAINED_RUNS || values.some((value) => !Number.isFinite(value))) return null;
  return [...values].sort((left, right) => left - right)[Math.floor(values.length / 2)];
}

function retainedMedian(observations, shape) {
  return median(observations
    .filter((observation) => observation.phase === "retained" && observation.shape === shape)
    .map((observation) => observation.duration));
}

function classify(metrics) {
  const { checkFloorMs, combinedTestBuildMs, directHarnessMs, cargoEndToEndMs } = metrics;
  if (![checkFloorMs, combinedTestBuildMs, directHarnessMs, cargoEndToEndMs].every(Number.isFinite)) return null;
  if (checkFloorMs > FAST_OWNER_EVIDENCE_LIMIT_MS) return "PACKAGE_BOUNDARY_OR_SLOW";
  if (combinedTestBuildMs > FAST_OWNER_EVIDENCE_LIMIT_MS || cargoEndToEndMs > FAST_OWNER_EVIDENCE_LIMIT_MS) {
    return "SMALLER_TEST_TARGET_REQUIRED";
  }
  if (directHarnessMs === Math.max(checkFloorMs, combinedTestBuildMs, directHarnessMs, cargoEndToEndMs)) {
    return "HARNESS_OPTIMIZATION_REQUIRED";
  }
  return "BOUNDED_FAST_OWNER_PLAUSIBLE";
}

function observationFrom(entry, result, extra = {}) {
  return {
    phase: entry.phase,
    cohort: entry.cohort,
    shape: entry.shape,
    ...(entry.pair ? { pair: entry.pair } : {}),
    ...(result ? {
      command: result.command,
      startedAt: result.startedAt,
      duration: result.duration,
      exitCode: result.exitCode,
      termination: result.termination,
    } : { duration: null, exitCode: 3, termination: "not-run" }),
    ...extra,
  };
}

export async function runRustFeasibility({
  repoRoot,
  outputPath = OUTPUT_PATH,
  runCommand = runObservedCommand,
  filesystem = { mkdir, readFile, stat, writeFile },
  randomUUID = defaultRandomUUID,
} = {}) {
  if (typeof repoRoot !== "string" || repoRoot.length === 0) throw new TypeError("repoRoot");
  if (outputPath !== OUTPUT_PATH) throw new Error(`Rust feasibility output must be ${OUTPUT_PATH}`);
  const root = path.resolve(repoRoot);
  const sourcePath = path.join(root, ...SOURCE_PATH.split("/"));
  const target = path.join(root, ...OUTPUT_PATH.split("/"));
  const original = Buffer.from(await filesystem.readFile(sourcePath));
  const originalHash = sha256(original);
  const observations = [];
  const pairedExecutables = new Map();
  let latestExecutable = null;
  let infrastructureFailure = false;
  let observedFailure = false;
  let interrupted = false;
  let restoration = { verified: true, restoredHash: originalHash };

  for (const entry of buildRunSchedule()) {
    let result = null;
    let observation;
    let token;
    let proof;
    let valid = true;
    let failureKind = null;
    let failure;
    let beforeMtime;
    let executableForAttempt;
    try {
      if (INVALIDATING_SHAPES.has(entry.shape)) {
        token = `${entry.cohort}:${randomUUID()}`;
        await filesystem.writeFile(sourcePath, appendMutation(original, token));
      }

      let command;
      let args;
      if (entry.shape === "directBinary") {
        executableForAttempt = pairedExecutables.get(entry.pair);
        if (!executableForAttempt) throw new Error("Paired no-run executable is unavailable");
        command = executableForAttempt;
        args = [RUST_TEST_NAME, "--exact", "--nocapture"];
      } else {
        [command, args] = commands[entry.shape];
      }
      if (entry.shape === "endToEnd") {
        executableForAttempt = latestExecutable;
        if (!executableForAttempt) throw new Error("No valid root test executable exists before end-to-end rebuild proof");
        beforeMtime = (await filesystem.stat(executableForAttempt)).mtimeMs;
      }

      result = await runCommand({
        command,
        args,
        cwd: root,
        capture: true,
        mirror: entry.shape === "endToEnd",
      });
      result.stdout ??= "";
      result.stderr ??= "";
      if (result.exitCode !== 0) {
        valid = false;
        failure = `Command exited ${result.exitCode}`;
        if (result.exitCode === 130) {
          failureKind = "interrupted";
          interrupted = true;
        } else if (result.termination === "spawn-error" || result.exitCode === 3) {
          failureKind = "infrastructure";
          infrastructureFailure = true;
        } else {
          failureKind = "command";
          observedFailure = true;
        }
      } else if (entry.shape === "noopCheck") {
        proof = parseCargoArtifacts(result.stdout, { repoRoot: root, expectedFresh: true, expectedTestProfile: false });
      } else if (entry.shape === "invalidatedCheck") {
        proof = parseCargoArtifacts(result.stdout, { repoRoot: root, expectedFresh: false, expectedTestProfile: false });
      } else if (entry.shape === "noRun") {
        proof = parseCargoArtifacts(result.stdout, {
          repoRoot: root,
          expectedFresh: false,
          expectedTestProfile: true,
          requireExecutable: true,
        });
        executableForAttempt = proof.executable;
        latestExecutable = proof.executable;
        pairedExecutables.set(entry.pair, proof.executable);
      } else if (entry.shape === "directBinary") {
        proof = { executable: executableForAttempt, exactTest: parseExactLibtest(result.stdout) };
      } else if (entry.shape === "endToEnd") {
        const compiledExtractum = /^\s*Compiling extractum v/mu.test(result.stderr);
        const afterMtime = (await filesystem.stat(executableForAttempt)).mtimeMs;
        const executableMtimeIncreased = afterMtime > beforeMtime;
        const exactTest = parseExactLibtest(result.stdout);
        if (!compiledExtractum) throw new Error("End-to-end stderr did not prove Compiling extractum");
        if (!executableMtimeIncreased) throw new Error("End-to-end root test executable mtime did not increase");
        proof = { executable: executableForAttempt, beforeMtime, afterMtime, compiledExtractum, executableMtimeIncreased, exactTest };
      }
    } catch (error) {
      valid = false;
      failureKind = "infrastructure";
      failure = errorMessage(error);
      infrastructureFailure = true;
    } finally {
      restoration = await restoreOriginal(filesystem, sourcePath, original, originalHash);
      if (!restoration.verified) {
        valid = false;
        failureKind = "restoration";
        failure = "Original readiness.rs bytes could not be verified after restoration";
        infrastructureFailure = true;
      }
    }

    observation = observationFrom(entry, result, {
      ...(token ? { token: `extractum-slice-1-probe:${token}` } : {}),
      valid,
      ...(proof ? { proof } : {}),
      ...(failure ? { failure, failureKind } : {}),
      restoration,
    });
    observations.push(observation);
    if (interrupted || !restoration.verified) break;
  }

  const checkFloorMs = retainedMedian(observations, "invalidatedCheck");
  const combinedTestBuildMs = retainedMedian(observations, "noRun");
  const directHarnessMs = retainedMedian(observations, "directBinary");
  const cargoEndToEndMs = retainedMedian(observations, "endToEnd");
  const metrics = {
    checkFloorMs,
    combinedTestBuildMs,
    testBuildOverCheckMs: Number.isFinite(checkFloorMs) && Number.isFinite(combinedTestBuildMs)
      ? combinedTestBuildMs - checkFloorMs
      : null,
    testBuildOverCheckExplanation: DELTA_EXPLANATION,
    directHarnessMs,
    cargoEndToEndMs,
  };
  const finalBytes = Buffer.from(await filesystem.readFile(sourcePath));
  const finalHash = sha256(finalBytes);
  const finalRestored = finalBytes.equals(original) && finalHash === originalHash;
  if (!finalRestored) infrastructureFailure = true;
  const valid = !infrastructureFailure && !observedFailure && !interrupted;
  const exitCode = interrupted ? 130 : infrastructureFailure ? 3 : observedFailure ? 1 : 0;
  const samples = observations.map((observation) => ({
    ...observation,
    warmup: observation.phase === "warmup",
    retained: observation.phase === "retained",
  }));
  const report = {
    schemaVersion: 1,
    testName: RUST_TEST_NAME,
    warmupRuns: WARMUP_RUNS,
    retainedRuns: RETAINED_RUNS,
    valid,
    exitCode,
    restoration: {
      sourcePath: SOURCE_PATH,
      originalHash,
      restoredHash: finalHash,
      originalLength: original.length,
      restoredLength: finalBytes.length,
      verified: restoration.verified && finalRestored,
      ...(restoration.recoveryVerified !== undefined ? { recoveryVerified: restoration.recoveryVerified } : {}),
    },
    samples,
    summary: metrics,
    classification: classify(metrics),
  };
  await filesystem.mkdir(path.dirname(target), { recursive: true });
  await filesystem.writeFile(target, `${JSON.stringify(report, null, 2)}\n`, "utf8");
  return report;
}

function cliOutputPath(argv) {
  if (argv.length !== 2 || argv[0] !== "--output" || argv[1] !== OUTPUT_PATH) {
    throw new Error(`Usage: slice-1-rust-feasibility.mjs --output ${OUTPUT_PATH}`);
  }
  return argv[1];
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const report = await runRustFeasibility({ repoRoot: process.cwd(), outputPath: cliOutputPath(process.argv.slice(2)) });
    process.exitCode = report.exitCode;
  } catch (error) {
    console.error(errorMessage(error));
    process.exitCode = 3;
  }
}
