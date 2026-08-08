import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";

import * as gitState from "./git-state.mjs";

const temporaryRoots: string[] = [];

afterEach(async () => {
  await Promise.all(temporaryRoots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

describe("process shell diagnostic Git states", () => {
  it("requires every generated B/C/E patch to carry a Cargo.lock text hunk", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "extractum-patch-contract-"));
    temporaryRoots.push(root);
    const patchFiles = await Promise.all(["B", "C", "E"].map(async (state) => {
      const file = path.join(root, `${state}.patch`);
      await writeFile(file, [
        "diff --git a/src-tauri/Cargo.lock b/src-tauri/Cargo.lock",
        "--- a/src-tauri/Cargo.lock",
        "+++ b/src-tauri/Cargo.lock",
        "@@ -1 +1 @@",
        "-old",
        "+new",
      ].join("\n"));
      return file;
    }));

    const validateGeneratedPatchFiles = (gitState as {
      validateGeneratedPatchFiles?: (files: string[]) => Promise<void>;
    }).validateGeneratedPatchFiles;

    expect(validateGeneratedPatchFiles).toBeTypeOf("function");
    await expect(validateGeneratedPatchFiles?.(patchFiles)).resolves.toBeUndefined();

    await writeFile(patchFiles[1], "diff --git a/src-tauri/Cargo.toml b/src-tauri/Cargo.toml\n");
    await expect(validateGeneratedPatchFiles?.(patchFiles)).rejects.toThrow("Cargo.lock text hunk");
  });
});
