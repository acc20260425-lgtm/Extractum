import path from "node:path";
import { describe, expect, it, vi } from "vitest";
import {
  RETAINED_RUNS,
  RUST_TEST_NAME,
  WARMUP_RUNS,
  appendMutation,
  buildRunSchedule,
  parseCargoArtifacts,
  parseExactLibtest,
  runRustFeasibility,
} from "./slice-1-rust-feasibility.mjs";

const ORIGINAL_SOURCE = Buffer.from("pub fn readiness() {}\r\n", "utf8");

function cargoArtifact({
  fresh,
  test,
  executable = null,
  packageName = "extractum",
  targetName = "extractum_lib",
  kind = ["staticlib", "cdylib", "rlib"],
  crateTypes = ["staticlib", "cdylib", "rlib"],
}: {
  fresh: boolean;
  test: boolean;
  executable?: string | null;
  packageName?: string;
  targetName?: string;
  kind?: string[];
  crateTypes?: string[];
}) {
  return `${JSON.stringify({
    reason: "compiler-artifact",
    package_id: `path+file:///repo/src-tauri#${packageName}@0.1.0`,
    manifest_path: "G:\\repo\\src-tauri\\Cargo.toml",
    target: {
      name: targetName,
      kind,
      crate_types: crateTypes,
      src_path: "G:\\repo\\src-tauri\\src\\lib.rs",
      edition: "2021",
      doc: true,
      doctest: true,
      test: true,
    },
    profile: { test },
    fresh,
    executable,
  })}\n`;
}

function exactPass() {
  return `\nrunning 1 test\ntest ${RUST_TEST_NAME} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 42 filtered out; finished in 0.00s\n`;
}

describe("slice one Rust feasibility schedule", () => {
  it("discloses one warm-up and retains three observations for every shape", () => {
    expect(WARMUP_RUNS).toBe(1);
    expect(RETAINED_RUNS).toBe(3);

    const schedule = buildRunSchedule();
    const shapes = ["noopCheck", "invalidatedCheck", "noRun", "directBinary", "endToEnd"];
    for (const shape of shapes) {
      expect(schedule.filter((entry) => entry.phase === "warmup" && entry.shape === shape)).toHaveLength(1);
      expect(schedule.filter((entry) => entry.phase === "retained" && entry.shape === shape)).toHaveLength(3);
    }
    expect(schedule.filter((entry) => entry.phase === "retained")).toHaveLength(15);
  });

  it("warms the paired executable before end-to-end and alternates retained invalidating cohorts", () => {
    const schedule = buildRunSchedule();
    expect(schedule.filter((entry) => entry.phase === "warmup").map((entry) => entry.shape)).toEqual([
      "noopCheck",
      "invalidatedCheck",
      "noRun",
      "directBinary",
      "endToEnd",
    ]);

    const retained = schedule.filter((entry) => entry.phase === "retained");
    expect(retained.slice(0, 3).map((entry) => entry.shape)).toEqual(["noopCheck", "noopCheck", "noopCheck"]);
    expect(retained.filter((entry) => entry.cohort === "retained-1").map((entry) => entry.shape)).toEqual([
      "invalidatedCheck", "noRun", "directBinary", "endToEnd",
    ]);
    expect(retained.filter((entry) => entry.cohort === "retained-2").map((entry) => entry.shape)).toEqual([
      "endToEnd", "noRun", "directBinary", "invalidatedCheck",
    ]);
    expect(retained.filter((entry) => entry.cohort === "retained-3").map((entry) => entry.shape)).toEqual([
      "noRun", "directBinary", "invalidatedCheck", "endToEnd",
    ]);
  });
});

describe("slice one Rust feasibility proof parsers", () => {
  const repoRoot = path.resolve("G:/repo");
  const executable = path.join(repoRoot, "src-tauri", "target", "debug", "deps", "extractum_lib-deadbeef.exe");

  it("requires the expected freshness, root package, target, and profile", () => {
    expect(parseCargoArtifacts(cargoArtifact({ fresh: true, test: false }), {
      repoRoot,
      expectedFresh: true,
      expectedTestProfile: false,
    })).toMatchObject({ fresh: true, executable: null });

    expect(parseCargoArtifacts(cargoArtifact({ fresh: false, test: false }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: false,
    })).toMatchObject({ fresh: false, executable: null });

    expect(parseCargoArtifacts(cargoArtifact({ fresh: false, test: true, executable }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: true,
      requireExecutable: true,
    })).toMatchObject({ fresh: false, executable });

    expect(() => parseCargoArtifacts(cargoArtifact({ fresh: true, test: false }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: false,
    })).toThrow(/fresh/i);
    expect(() => parseCargoArtifacts(cargoArtifact({ fresh: false, test: false, packageName: "extractum_storage" }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: false,
    })).toThrow(/extractum/i);
    expect(() => parseCargoArtifacts(cargoArtifact({ fresh: false, test: false, targetName: "other" }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: false,
    })).toThrow(/extractum_lib/i);
    expect(() => parseCargoArtifacts(cargoArtifact({ fresh: false, test: true, executable }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: false,
    })).toThrow(/profile/i);
  });

  it("accepts exactly one canonical Cargo 1.95 multi-crate-type root artifact", () => {
    const actualCargo195Shape = cargoArtifact({ fresh: false, test: false });
    expect(parseCargoArtifacts(actualCargo195Shape, {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: false,
    })).toMatchObject({ package: "extractum", target: "extractum_lib", fresh: false, testProfile: false });

    for (const legacySyntheticShape of [
      cargoArtifact({ fresh: false, test: false, kind: ["lib"] }),
      cargoArtifact({ fresh: false, test: false, crateTypes: ["lib"] }),
    ]) {
      expect(() => parseCargoArtifacts(legacySyntheticShape, {
        repoRoot,
        expectedFresh: false,
        expectedTestProfile: false,
      })).toThrow(/extractum_lib/i);
    }
    expect(() => parseCargoArtifacts(`${actualCargo195Shape}${actualCargo195Shape}`, {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: false,
    })).toThrow(/exactly one/i);
  });

  it("accepts only a root test executable below canonical src-tauri target", () => {
    expect(parseCargoArtifacts(cargoArtifact({ fresh: false, test: true, executable }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: true,
      requireExecutable: true,
    }).executable).toBe(executable);

    const outside = path.join(repoRoot, "other-target", "debug", "deps", "extractum_lib-deadbeef.exe");
    expect(() => parseCargoArtifacts(cargoArtifact({ fresh: false, test: true, executable: outside }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: true,
      requireExecutable: true,
    })).toThrow(/canonical/i);
    expect(() => parseCargoArtifacts(cargoArtifact({ fresh: false, test: true }), {
      repoRoot,
      expectedFresh: false,
      expectedTestProfile: true,
      requireExecutable: true,
    })).toThrow(/executable/i);
  });

  it("accepts exactly one named passing test and rejects empty, multiple, ignored, or failed runs", () => {
    expect(parseExactLibtest(exactPass())).toEqual({ passed: 1, failed: 0, ignored: 0 });

    const invalid = [
      "running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.00s",
      `running 2 tests\ntest ${RUST_TEST_NAME} ... ok\ntest other ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`,
      `running 1 test\ntest ${RUST_TEST_NAME} ... ignored\n\ntest result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s`,
      `running 1 test\ntest ${RUST_TEST_NAME} ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`,
      `running 1 test\ntest ${RUST_TEST_NAME} ... ok\ntest ${RUST_TEST_NAME} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`,
    ];
    for (const output of invalid) expect(() => parseExactLibtest(output)).toThrow(/exactly one passing test/i);
  });
});

function createHarness({
  failRetainedInvalidated = false,
  restoreFailure,
  staleReport = false,
}: {
  failRetainedInvalidated?: boolean;
  restoreFailure?: "read" | "write" | "write-when-mutated";
  staleReport?: boolean;
} = {}) {
  const repoRoot = path.resolve("G:/virtual-repo");
  const sourcePath = path.join(repoRoot, "src-tauri", "src", "readiness.rs");
  const outputPath = path.join(repoRoot, "artifacts", "testing", "slice-1", "rust-feasibility.json");
  const executable = path.join(repoRoot, "src-tauri", "target", "debug", "deps", "extractum_lib-cafebabe.exe");
  const files = new Map<string, Buffer>([[sourcePath, Buffer.from(ORIGINAL_SOURCE)]]);
  if (staleReport) files.set(outputPath, Buffer.from('{"valid":true,"classification":"STALE"}\n'));
  const mtimes = new Map<string, number>([[executable, 100]]);
  const writes: Array<{ file: string; bytes: Buffer }> = [];
  let uuid = 0;
  let invalidatedCalls = 0;
  let sourceReads = 0;

  const filesystem = {
    readFile: vi.fn(async (file: string) => {
      if (file === sourcePath) {
        sourceReads += 1;
        if (restoreFailure === "read" && sourceReads > 1) throw new Error("persistent restore read failure");
      }
      const value = files.get(file);
      if (!value) throw Object.assign(new Error(`ENOENT: ${file}`), { code: "ENOENT" });
      return Buffer.from(value);
    }),
    writeFile: vi.fn(async (file: string, value: string | Uint8Array) => {
      const bytes = Buffer.isBuffer(value) ? Buffer.from(value) : Buffer.from(value);
      const currentSourceIsMutated = file === sourcePath && !files.get(sourcePath)?.equals(ORIGINAL_SOURCE);
      if (file === sourcePath
        && bytes.equals(ORIGINAL_SOURCE)
        && (restoreFailure === "write" || (restoreFailure === "write-when-mutated" && currentSourceIsMutated))) {
        throw new Error("persistent restore write failure");
      }
      files.set(file, bytes);
      writes.push({ file, bytes });
    }),
    mkdir: vi.fn(async () => undefined),
    rm: vi.fn(async (file: string) => { files.delete(file); }),
    stat: vi.fn(async (file: string) => ({ mtimeMs: mtimes.get(file) ?? 0 })),
  };

  const runCommand = vi.fn(async ({ command, args }: { command: string; args: string[] }) => {
    const source = files.get(sourcePath) ?? Buffer.alloc(0);
    const mutated = !source.equals(ORIGINAL_SOURCE);
    const base = {
      command: [command, ...args].join(" "),
      startedAt: "2026-08-02T10:11:12.123Z",
      duration: 100 + runCommand.mock.calls.length,
      exitCode: 0,
      termination: "exit",
      stdout: "",
      stderr: "",
    };
    if (command === executable) return { ...base, stdout: exactPass() };
    if (args[0] === "check") {
      if (mutated) invalidatedCalls += 1;
      if (mutated && failRetainedInvalidated && invalidatedCalls === 2) {
        return { ...base, exitCode: 1, stderr: "compiler error" };
      }
      return { ...base, stdout: cargoArtifact({ fresh: !mutated, test: false }) };
    }
    if (args.includes("--no-run")) {
      mtimes.set(executable, (mtimes.get(executable) ?? 0) + 10);
      return { ...base, stdout: cargoArtifact({ fresh: false, test: true, executable }) };
    }
    if (args[0] === "test") {
      mtimes.set(executable, (mtimes.get(executable) ?? 0) + 10);
      return { ...base, stdout: exactPass(), stderr: "   Compiling extractum v0.1.0 (G:/virtual-repo/src-tauri)\n" };
    }
    throw new Error(`unexpected command: ${command} ${args.join(" ")}`);
  });

  return {
    repoRoot,
    sourcePath,
    outputPath,
    executable,
    files,
    writes,
    filesystem,
    runCommand,
    randomUUID: () => `00000000-0000-4000-8000-${String(++uuid).padStart(12, "0")}`,
  };
}

describe("slice one Rust feasibility driver", () => {
  it("uses byte-preserving newline-aware inert mutations", () => {
    const mutated = appendMutation(Buffer.from("one\r\ntwo\r\n"), "retained-1:uuid");
    expect(mutated.subarray(0, 10)).toEqual(Buffer.from("one\r\ntwo\r\n"));
    expect(mutated.toString("utf8")).toBe("one\r\ntwo\r\n// extractum-slice-1-probe:retained-1:uuid\r\n");
  });

  it("proves every rebuild, uses unique invalidation tokens, and emits mechanical medians", async () => {
    const harness = createHarness();
    const report = await runRustFeasibility({
      repoRoot: harness.repoRoot,
      runCommand: harness.runCommand,
      filesystem: harness.filesystem,
      randomUUID: harness.randomUUID,
    });

    expect(report.exitCode).toBe(0);
    expect(report.valid).toBe(true);
    expect(report.samples).toHaveLength(20);
    expect(report.samples.filter((entry: { retained: boolean }) => entry.retained)).toHaveLength(15);
    expect(report.samples.filter((entry: { warmup: boolean }) => entry.warmup)).toHaveLength(5);
    const invalidating = report.samples.filter((entry: { retained: boolean; token?: string; shape: string }) =>
      entry.retained && ["invalidatedCheck", "noRun", "endToEnd"].includes(entry.shape));
    expect(new Set(invalidating.map((entry: { token: string }) => entry.token)).size).toBe(9);

    const endToEnd = report.samples.filter((entry: { shape: string }) => entry.shape === "endToEnd");
    expect(endToEnd).toHaveLength(4);
    expect(endToEnd.every((entry: { proof: { compiledExtractum: boolean; executableMtimeIncreased: boolean; exactTest: unknown } }) =>
      entry.proof.compiledExtractum && entry.proof.executableMtimeIncreased && entry.proof.exactTest)).toBe(true);
    expect(harness.runCommand.mock.calls.filter(([call]) => call.command === harness.executable).every(([call]) =>
      call.args.join(" ") === `${RUST_TEST_NAME} --exact --nocapture`)).toBe(true);

    expect(report.summary).toMatchObject({
      checkFloorMs: expect.any(Number),
      combinedTestBuildMs: expect.any(Number),
      testBuildOverCheckMs: expect.any(Number),
      directHarnessMs: expect.any(Number),
      cargoEndToEndMs: expect.any(Number),
    });
    expect(report.summary.testBuildOverCheckExplanation).toBe("The test-build-over-check delta includes cfg(test), root test code, dev-dependencies, app-test-support, different compiler units, code generation, link, and cache/process noise; it is not pure link time.");
    expect(["PACKAGE_BOUNDARY_OR_SLOW", "SMALLER_TEST_TARGET_REQUIRED", "HARNESS_OPTIMIZATION_REQUIRED", "BOUNDED_FAST_OWNER_PLAUSIBLE"])
      .toContain(report.classification);
    expect(harness.files.get(harness.sourcePath)).toEqual(ORIGINAL_SOURCE);
    expect(report.restoration).toMatchObject({
      originalLength: ORIGINAL_SOURCE.length,
      restoredLength: ORIGINAL_SOURCE.length,
      verified: true,
    });
    expect(JSON.parse(harness.files.get(harness.outputPath)!.toString("utf8"))).toMatchObject({ valid: true, exitCode: 0 });

    const calls = harness.runCommand.mock.calls.map(([call]) => call);
    const checkArgs = ["check", "--manifest-path", "src-tauri/Cargo.toml", "-p", "extractum", "--lib", "--message-format=json"];
    const noRunArgs = ["test", "--manifest-path", "src-tauri/Cargo.toml", "-p", "extractum", "--lib", "--no-run", "--message-format=json"];
    const endToEndArgs = ["test", "--manifest-path", "src-tauri/Cargo.toml", "-p", "extractum", "--lib", RUST_TEST_NAME, "--", "--exact"];
    expect(calls.filter((call) => call.command === "cargo" && JSON.stringify(call.args) === JSON.stringify(checkArgs))).toHaveLength(8);
    expect(calls.filter((call) => call.command === "cargo" && JSON.stringify(call.args) === JSON.stringify(noRunArgs))).toHaveLength(4);
    expect(calls.filter((call) => call.command === "cargo" && JSON.stringify(call.args) === JSON.stringify(endToEndArgs))).toHaveLength(4);
    expect(calls.filter((call) => call.command === "cargo" && call.mirror)).toHaveLength(4);
  });

  it("restores exact source bytes in finally and retains a failed sample without replacement", async () => {
    const harness = createHarness({ failRetainedInvalidated: true });
    const report = await runRustFeasibility({
      repoRoot: harness.repoRoot,
      runCommand: harness.runCommand,
      filesystem: harness.filesystem,
      randomUUID: harness.randomUUID,
    });

    expect(report.exitCode).toBe(1);
    expect(report.samples.filter((entry: { retained: boolean }) => entry.retained)).toHaveLength(15);
    expect(report.samples.filter((entry: { retained: boolean; shape: string }) =>
      entry.retained && entry.shape === "invalidatedCheck")).toHaveLength(3);
    expect(report.samples).toContainEqual(expect.objectContaining({
      retained: true,
      shape: "invalidatedCheck",
      exitCode: 1,
      valid: false,
    }));
    expect(harness.files.get(harness.sourcePath)).toEqual(ORIGINAL_SOURCE);
    const sourceWrites = harness.writes.filter((write) => write.file === harness.sourcePath);
    expect(sourceWrites.at(-1)?.bytes).toEqual(ORIGINAL_SOURCE);
    expect(report.summary).toMatchObject({
      checkFloorMs: null,
      combinedTestBuildMs: null,
      testBuildOverCheckMs: null,
      directHarnessMs: null,
      cargoEndToEndMs: null,
    });
    expect(report.classification).toBeNull();
    expect(report.classificationUnavailableReason).toMatch(/invalid|incomplete/i);
  });

  it("invalidates the report with exit 3 when restored bytes do not match", async () => {
    const harness = createHarness();
    let corruptNextRestore = true;
    const writeFile = harness.filesystem.writeFile;
    harness.filesystem.writeFile = vi.fn(async (file: string, value: string | Uint8Array) => {
      if (file === harness.sourcePath && Buffer.from(value).equals(ORIGINAL_SOURCE) && corruptNextRestore) {
        corruptNextRestore = false;
        await writeFile(file, Buffer.from("corrupt"));
        return;
      }
      await writeFile(file, value);
    });

    const report = await runRustFeasibility({
      repoRoot: harness.repoRoot,
      runCommand: harness.runCommand,
      filesystem: harness.filesystem,
      randomUUID: harness.randomUUID,
    });

    expect(report).toMatchObject({ exitCode: 3, valid: false, restoration: { verified: false } });
    expect(report.classification).toBeNull();
  });

  it("restores and verifies an active mutation before an injected SIGINT completes", async () => {
    const harness = createHarness();
    let signalHandler: ((signal: string) => Promise<number>) | undefined;
    const removeSignalHandlers = vi.fn();
    const installSignalHandlers = vi.fn((handler) => {
      signalHandler = handler;
      return removeSignalHandlers;
    });
    const baseRunCommand = harness.runCommand;
    let signalled = false;
    harness.runCommand = vi.fn(async (call) => {
      const mutated = !harness.files.get(harness.sourcePath)!.equals(ORIGINAL_SOURCE);
      if (mutated && !signalled) {
        signalled = true;
        expect(signalHandler).toBeTypeOf("function");
        await signalHandler!("SIGINT");
        expect(harness.files.get(harness.sourcePath)).toEqual(ORIGINAL_SOURCE);
        return {
          command: [call.command, ...call.args].join(" "),
          startedAt: "2026-08-02T10:11:12.123Z",
          duration: 7,
          exitCode: 130,
          termination: "signal",
          signal: "SIGINT",
          stdout: "",
          stderr: "",
        };
      }
      return baseRunCommand(call);
    });

    const report = await runRustFeasibility({
      repoRoot: harness.repoRoot,
      runCommand: harness.runCommand,
      filesystem: harness.filesystem,
      randomUUID: harness.randomUUID,
      installSignalHandlers,
    });

    expect(report).toMatchObject({ exitCode: 130, valid: false, restoration: { verified: true } });
    expect(report.classification).toBeNull();
    expect(harness.files.get(harness.sourcePath)).toEqual(ORIGINAL_SOURCE);
    expect(installSignalHandlers).toHaveBeenCalledTimes(1);
    expect(removeSignalHandlers).toHaveBeenCalledTimes(1);
  });

  it.each(["read", "write"] as const)("replaces a stale valid report after persistent restoration %s failure", async (restoreFailure) => {
    const harness = createHarness({ restoreFailure, staleReport: true });
    const staleAtFirstCommand: string[] = [];
    const baseRunCommand = harness.runCommand;
    harness.runCommand = vi.fn(async (call) => {
      staleAtFirstCommand.push(harness.files.get(harness.outputPath)?.toString("utf8") ?? "missing");
      return baseRunCommand(call);
    });

    const report = await runRustFeasibility({
      repoRoot: harness.repoRoot,
      runCommand: harness.runCommand,
      filesystem: harness.filesystem,
      randomUUID: harness.randomUUID,
      installSignalHandlers: () => vi.fn(),
    });

    expect(staleAtFirstCommand[0]).not.toContain('"valid":true');
    expect(report).toMatchObject({ exitCode: 3, valid: false, classification: null });
    expect(report.summary).toMatchObject({
      checkFloorMs: null,
      combinedTestBuildMs: null,
      testBuildOverCheckMs: null,
      directHarnessMs: null,
      cargoEndToEndMs: null,
    });
    const written = JSON.parse(harness.files.get(harness.outputPath)!.toString("utf8"));
    expect(written).toMatchObject({ exitCode: 3, valid: false, classification: null });
    expect(written.classificationUnavailableReason).toMatch(/invalid|incomplete/i);
  });

  it("gives restoration failure precedence over injected interruption", async () => {
    const harness = createHarness({ restoreFailure: "write-when-mutated" });
    let signalHandler: ((signal: string) => Promise<number>) | undefined;
    let signalCalls = 0;
    const baseRunCommand = harness.runCommand;
    harness.runCommand = vi.fn(async (call) => {
      if (!harness.files.get(harness.sourcePath)!.equals(ORIGINAL_SOURCE)) {
        await signalHandler!("SIGTERM");
        return {
          command: [call.command, ...call.args].join(" "),
          startedAt: "2026-08-02T10:11:12.123Z",
          duration: 7,
          exitCode: 130,
          termination: "signal",
          signal: "SIGTERM",
          stdout: "",
          stderr: "",
        };
      }
      return baseRunCommand(call);
    });

    const report = await runRustFeasibility({
      repoRoot: harness.repoRoot,
      runCommand: harness.runCommand,
      filesystem: harness.filesystem,
      randomUUID: harness.randomUUID,
      installSignalHandlers: (handler) => {
        signalHandler = async (signal) => {
          signalCalls += 1;
          return handler(signal);
        };
        return vi.fn();
      },
    });

    expect(report).toMatchObject({ exitCode: 3, valid: false, classification: null });
    expect(signalCalls).toBe(1);
  });

  it("rewrites the final report when SIGINT arrives during its write", async () => {
    const harness = createHarness();
    let signalHandler: ((signal: string) => Promise<number>) | undefined;
    let outputWrites = 0;
    const writeFile = harness.filesystem.writeFile;
    harness.filesystem.writeFile = vi.fn(async (file: string, value: string | Uint8Array) => {
      if (file === harness.outputPath) {
        outputWrites += 1;
        if (outputWrites === 2) await signalHandler!("SIGINT");
      }
      await writeFile(file, value);
    });

    const report = await runRustFeasibility({
      repoRoot: harness.repoRoot,
      runCommand: harness.runCommand,
      filesystem: harness.filesystem,
      randomUUID: harness.randomUUID,
      installSignalHandlers: (handler) => {
        signalHandler = handler;
        return vi.fn();
      },
    });

    expect(report).toMatchObject({ exitCode: 130, valid: false, classification: null });
    expect(JSON.parse(harness.files.get(harness.outputPath)!.toString("utf8"))).toMatchObject({
      exitCode: 130,
      valid: false,
      classification: null,
    });
    expect(outputWrites).toBe(3);
  });
});
