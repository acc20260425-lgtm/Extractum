import { randomUUID as defaultRandomUUID, createHash } from "node:crypto";
import { mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
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

function hasCanonicalExtractumRootShape(target) {
  const crateTypes = ["staticlib", "cdylib", "rlib"];
  return Array.isArray(target?.kind)
    && target.kind.length === crateTypes.length
    && target.kind.every((kind, index) => kind === crateTypes[index])
    && Array.isArray(target.crate_types)
    && target.crate_types.length === crateTypes.length
    && target.crate_types.every((crateType, index) => crateType === crateTypes[index]);
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
  const { repoRoot, freshExpectation, expectedTestProfile, requireExecutable = false } = expectation ?? {};
  if (typeof repoRoot !== "string"
    || !["must-be-fresh", "must-rebuild", "record"].includes(freshExpectation)
    || typeof expectedTestProfile !== "boolean") {
    throw new TypeError("Cargo artifact expectation requires repoRoot, freshExpectation, and expectedTestProfile");
  }
  const messages = String(text).split(/\r?\n/u).filter((line) => line.trim().length > 0).map((line) => {
    try {
      return JSON.parse(line);
    } catch {
      throw new Error("Malformed Cargo JSON artifact output");
    }
  });
  const candidates = messages.filter((message) => message?.reason === "compiler-artifact"
    && packageIsExtractum(message.package_id)
    && message.target?.name === "extractum_lib");
  if (candidates.length !== 1) throw new Error("Expected exactly one extractum/extractum_lib Cargo artifact");
  const artifact = candidates[0];
  if (!hasCanonicalExtractumRootShape(artifact.target)) {
    throw new Error("extractum_lib target.kind and target.crate_types did not match the canonical Cargo 1.95 root shape");
  }
  if (typeof artifact.fresh !== "boolean") throw new Error("Cargo artifact fresh proof must be boolean");
  if (freshExpectation === "must-be-fresh" && artifact.fresh !== true) {
    throw new Error("Cargo artifact fresh proof did not equal true");
  }
  if (freshExpectation === "must-rebuild" && artifact.fresh !== false) {
    throw new Error("Cargo artifact fresh proof did not equal false");
  }
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
  const namedPasses = [...output.matchAll(new RegExp(`^test ${escapeRegExp(RUST_TEST_NAME)} \\.\\.\\. ok\\s*$`, "gmu"))];
  if (running.length !== 1
    || Number(running[0][1]) !== 1
    || summaries.length !== 1
    || summaries[0][1] !== "ok"
    || Number(summaries[0][2]) !== 1
    || Number(summaries[0][3]) !== 0
    || Number(summaries[0][4]) !== 0
    || namedPasses.length !== 1) {
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

async function verifyOriginal(filesystem, sourcePath, original, originalHash) {
  try {
    const restored = Buffer.from(await filesystem.readFile(sourcePath));
    const restoredHash = sha256(restored);
    return {
      verified: restoredHash === originalHash && restored.equals(original),
      restoredHash,
    };
  } catch (error) {
    return { verified: false, restoredHash: null, error: errorMessage(error) };
  }
}

function createSourceMutationGuard(filesystem, sourcePath, original, originalHash) {
  let mutationWrite = Promise.resolve();
  let restorationPromise = null;
  let stopRequested = false;
  let mutationAttempted = false;

  return {
    async mutate(mutated) {
      if (stopRequested) throw new Error("Study interrupted before source mutation");
      mutationAttempted = true;
      mutationWrite = filesystem.writeFile(sourcePath, mutated);
      await mutationWrite;
      if (stopRequested) {
        await this.restore();
        throw new Error("Study interrupted during source mutation");
      }
    },
    requestStop() {
      stopRequested = true;
      return this.restore();
    },
    restore() {
      if (!restorationPromise) {
        restorationPromise = (async () => {
          try {
            await mutationWrite;
          } catch {
            // Restoration still has to run after a failed mutation write.
          }
          const restoration = mutationAttempted
            ? await restoreOriginal(filesystem, sourcePath, original, originalHash)
            : await verifyOriginal(filesystem, sourcePath, original, originalHash);
          if (restoration.verified) mutationAttempted = false;
          return restoration;
        })().finally(() => {
          restorationPromise = null;
        });
      }
      return restorationPromise;
    },
  };
}

function defaultInstallSignalHandlers(handler) {
  const listeners = new Map(["SIGINT", "SIGTERM"].map((signal) => {
    const listener = () => {
      void handler(signal).then((exitCode) => {
        process.exitCode = exitCode;
      }, () => {
        process.exitCode = 3;
      });
    };
    process.on(signal, listener);
    return [signal, listener];
  }));
  return () => {
    for (const [signal, listener] of listeners) process.off(signal, listener);
  };
}

function unavailableSummary() {
  return {
    checkFloorMs: null,
    combinedTestBuildMs: null,
    testBuildOverCheckMs: null,
    testBuildOverCheckExplanation: DELTA_EXPLANATION,
    directHarnessMs: null,
    cargoEndToEndMs: null,
  };
}

function preflightReport(failure = "Study is in progress; no decision is available") {
  return {
    schemaVersion: 1,
    testName: RUST_TEST_NAME,
    warmupRuns: WARMUP_RUNS,
    retainedRuns: RETAINED_RUNS,
    valid: false,
    exitCode: 3,
    failure,
    samples: [],
    summary: unavailableSummary(),
    classification: null,
    classificationUnavailableReason: "Study is incomplete or invalid; no authoritative classification is available",
  };
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
  filesystem = { mkdir, readFile, rm, stat, writeFile },
  randomUUID = defaultRandomUUID,
  installSignalHandlers = defaultInstallSignalHandlers,
} = {}) {
  if (typeof repoRoot !== "string" || repoRoot.length === 0) throw new TypeError("repoRoot");
  if (outputPath !== OUTPUT_PATH) throw new Error(`Rust feasibility output must be ${OUTPUT_PATH}`);
  const root = path.resolve(repoRoot);
  const sourcePath = path.join(root, ...SOURCE_PATH.split("/"));
  const target = path.join(root, ...OUTPUT_PATH.split("/"));
  await filesystem.mkdir(path.dirname(target), { recursive: true });
  let removalFailure;
  try {
    await filesystem.rm(target, { force: true });
  } catch (error) {
    removalFailure = errorMessage(error);
  }
  const initialReport = preflightReport(removalFailure
    ? `Unable to remove prior report: ${removalFailure}`
    : undefined);
  await filesystem.writeFile(target, `${JSON.stringify(initialReport, null, 2)}\n`, "utf8");
  if (removalFailure) return initialReport;

  let original;
  try {
    original = Buffer.from(await filesystem.readFile(sourcePath));
  } catch (error) {
    const report = preflightReport(`Unable to capture original readiness.rs bytes: ${errorMessage(error)}`);
    await filesystem.writeFile(target, `${JSON.stringify(report, null, 2)}\n`, "utf8");
    return report;
  }
  const originalHash = sha256(original);
  const sourceGuard = createSourceMutationGuard(filesystem, sourcePath, original, originalHash);
  const observations = [];
  const pairedExecutables = new Map();
  let latestExecutable = null;
  let infrastructureFailure = false;
  let observedFailure = false;
  let interrupted = false;
  let cancellationFailure = false;
  let restorationFailure = false;
  let restoration = { verified: true, restoredHash: originalHash };
  let signalGeneration = 0;
  let signalRestoration = Promise.resolve(130);
  let activeObservation = null;
  const removeSignalHandlers = installSignalHandlers((signal) => {
    signalGeneration += 1;
    interrupted = true;
    signalRestoration = (async () => {
      const active = activeObservation;
      if (active) {
        active.controller.abort(signal);
        let activeResult;
        try {
          activeResult = await active.settlement;
        } catch (error) {
          cancellationFailure = true;
          infrastructureFailure = true;
          restoration = {
            verified: false,
            restoredHash: null,
            cancellationError: errorMessage(error),
          };
          return 3;
        }
        if (activeResult?.cancellationConfirmed !== true) {
          cancellationFailure = true;
          infrastructureFailure = true;
          restoration = {
            verified: false,
            restoredHash: null,
            cancellationError: "Active command tree termination was not confirmed",
          };
          return 3;
        }
      }
      restoration = await sourceGuard.requestStop();
      if (!restoration.verified) {
        restorationFailure = true;
        infrastructureFailure = true;
        return 3;
      }
      return 130;
    })();
    return signalRestoration;
  });

  try {
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
        await sourceGuard.mutate(appendMutation(original, token));
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

      const controller = new AbortController();
      const settlement = Promise.resolve().then(() => runCommand({
        command,
        args,
        cwd: root,
        repoRoot: root,
        capture: true,
        mirror: entry.shape === "endToEnd",
        signal: controller.signal,
      }));
      const active = { controller, settlement };
      activeObservation = active;
      try {
        result = await settlement;
      } finally {
        if (activeObservation === active) activeObservation = null;
      }
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
        proof = parseCargoArtifacts(result.stdout, {
          repoRoot: root,
          freshExpectation: entry.phase === "warmup" ? "record" : "must-be-fresh",
          expectedTestProfile: false,
        });
      } else if (entry.shape === "invalidatedCheck") {
        proof = parseCargoArtifacts(result.stdout, {
          repoRoot: root,
          freshExpectation: "must-rebuild",
          expectedTestProfile: false,
        });
      } else if (entry.shape === "noRun") {
        proof = parseCargoArtifacts(result.stdout, {
          repoRoot: root,
          freshExpectation: "must-rebuild",
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
      if (interrupted) await signalRestoration;
      if (!cancellationFailure) restoration = await sourceGuard.restore();
      if (cancellationFailure) {
        valid = false;
        failureKind = "cancellation";
        failure = "Active command tree termination was not confirmed; source restoration was withheld";
        infrastructureFailure = true;
      } else if (!restoration.verified) {
        valid = false;
        failureKind = "restoration";
        failure = "Original readiness.rs bytes could not be verified after restoration";
        infrastructureFailure = true;
        restorationFailure = true;
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

    let finalBytes = null;
    let finalReadFailure;
    let verificationGeneration;
    do {
      await signalRestoration;
      verificationGeneration = signalGeneration;
      try {
        finalBytes = Buffer.from(await filesystem.readFile(sourcePath));
        finalReadFailure = undefined;
      } catch (error) {
        finalBytes = null;
        finalReadFailure = errorMessage(error);
        infrastructureFailure = true;
        restorationFailure = true;
      }
      await signalRestoration;
    } while (verificationGeneration !== signalGeneration);
    const finalHash = finalBytes ? sha256(finalBytes) : null;
    const finalRestored = Boolean(finalBytes?.equals(original) && finalHash === originalHash);
    if (!finalRestored) {
      infrastructureFailure = true;
      restorationFailure = true;
    }
    const buildReport = () => {
      const valid = !infrastructureFailure && !observedFailure && !interrupted;
      const completeValidStudy = valid
        && observations.length === buildRunSchedule().length
        && observations.every((observation) => observation.valid);
      const checkFloorMs = completeValidStudy ? retainedMedian(observations, "invalidatedCheck") : null;
      const combinedTestBuildMs = completeValidStudy ? retainedMedian(observations, "noRun") : null;
      const directHarnessMs = completeValidStudy ? retainedMedian(observations, "directBinary") : null;
      const cargoEndToEndMs = completeValidStudy ? retainedMedian(observations, "endToEnd") : null;
      const metrics = completeValidStudy ? {
        checkFloorMs,
        combinedTestBuildMs,
        testBuildOverCheckMs: combinedTestBuildMs - checkFloorMs,
        testBuildOverCheckExplanation: DELTA_EXPLANATION,
        directHarnessMs,
        cargoEndToEndMs,
      } : unavailableSummary();
      const exitCode = restorationFailure || cancellationFailure
        ? 3
        : interrupted ? 130 : infrastructureFailure ? 3 : observedFailure ? 1 : 0;
      const samples = observations.map((observation) => ({
        ...observation,
        warmup: observation.phase === "warmup",
        retained: observation.phase === "retained",
      }));
      return {
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
          restoredLength: finalBytes?.length ?? null,
          verified: restoration.verified && finalRestored,
          ...(finalReadFailure ? { verificationError: finalReadFailure } : {}),
          ...(restoration.recoveryVerified !== undefined ? { recoveryVerified: restoration.recoveryVerified } : {}),
        },
        samples,
        summary: metrics,
        classification: completeValidStudy ? classify(metrics) : null,
        ...(!completeValidStudy ? {
          classificationUnavailableReason: "Study is incomplete or invalid; no authoritative classification is available",
        } : {}),
      };
    };

    while (true) {
      await signalRestoration;
      const reportGeneration = signalGeneration;
      const report = buildReport();
      await filesystem.writeFile(target, `${JSON.stringify(report, null, 2)}\n`, "utf8");
      await signalRestoration;
      if (reportGeneration === signalGeneration) return report;
    }
  } finally {
    removeSignalHandlers();
  }
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
