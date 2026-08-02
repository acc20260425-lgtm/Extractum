import { describe, expect, it } from "vitest";

import { inspectGeminiBrowserSidecar } from "./check-gemini-browser-sidecar-binary.mjs";

const repoRoot = "C:\\workspace";
const targetTriple = "x86_64-pc-windows-msvc";
const bootstrapCommand = "Run: npm.cmd run bootstrap:testing";

function stats({
  size = 1,
  symbolicLink = false,
  directory = false,
  file = !symbolicLink && !directory,
}: {
  size?: number;
  symbolicLink?: boolean;
  directory?: boolean;
  file?: boolean;
}) {
  return {
    size,
    isSymbolicLink: () => symbolicLink,
    isDirectory: () => directory,
    isFile: () => file,
  };
}

function inspect(lstatSyncImpl: () => ReturnType<typeof stats>) {
  return inspectGeminiBrowserSidecar({
    repoRoot,
    targetTriple,
    platform: "win32",
    requestedTarget: "",
    lstatSyncImpl,
  });
}

describe("Gemini browser sidecar binary checker", () => {
  it("rejects a missing sidecar path with the bootstrap command", () => {
    expect(() => inspect(() => {
      const error = new Error("missing") as NodeJS.ErrnoException;
      error.code = "ENOENT";
      throw error;
    })).toThrow(bootstrapCommand);
  });

  it("rejects an empty regular sidecar file with the bootstrap command", () => {
    expect(() => inspect(() => stats({ size: 0 }))).toThrow(bootstrapCommand);
  });

  it("rejects a sidecar directory with the bootstrap command", () => {
    expect(() => inspect(() => stats({ directory: true }))).toThrow(bootstrapCommand);
  });

  it("rejects a sidecar symlink with the bootstrap command", () => {
    expect(() => inspect(() => stats({ symbolicLink: true }))).toThrow(bootstrapCommand);
  });

  it("accepts a non-empty regular sidecar file and reports its path and size", () => {
    expect(inspect(() => stats({ size: 42 }))).toEqual({
      relativePath: "src-tauri/binaries/gemini-browser-sidecar-x86_64-pc-windows-msvc.exe",
      size: 42,
    });
  });

  it("rejects a requested target different from the host target with the bootstrap command", () => {
    expect(() => inspectGeminiBrowserSidecar({
      repoRoot,
      targetTriple,
      platform: "win32",
      requestedTarget: "aarch64-pc-windows-msvc",
      lstatSyncImpl: () => stats({ size: 42 }),
    })).toThrow(bootstrapCommand);
  });
});
