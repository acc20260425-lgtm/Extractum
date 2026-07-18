import { mkdtemp, mkdir, readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  aRestoreArgs,
  D_BLOB_ANCHORS,
  STATE_TREE_ANCHORS,
  dRestoreArgs,
  validateLockDelta,
  validateStateManifests,
  verifyTargetIsolation,
} from "./git-state.mjs";

const emptyManifest = `[package]
name = "extractum-process"
version.workspace = true
edition.workspace = true
publish = false
`;

const eManifest = `${emptyManifest}
[dependencies]
anyhow.workspace = true
parking_lot.workspace = true
tokio.workspace = true

[target.'cfg(windows)'.dependencies]
windows-sys.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
`;

const root = (edge: boolean, migrated: boolean) => `[workspace]
members = [".", "crates/extractum-core", "crates/extractum-process"]
resolver = "2"

[workspace.dependencies]
${migrated ? `anyhow = "1.0"
parking_lot = "0.12"
tokio = { version = "1", features = ["full"] }
windows-sys = { version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }
` : ""}
[dependencies]
${migrated ? `anyhow = { workspace = true }
parking_lot = { workspace = true }
tokio = { workspace = true }
` : ""}${edge ? `extractum-process = { path = "crates/extractum-process" }
` : ""}
[target.'cfg(windows)'.dependencies]
windows-sys = ${migrated ? `{ workspace = true }` : `{ version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_JobObjects", "Win32_System_Threading"] }`}
`;

const processPaths = [
  "src-tauri/crates/extractum-process/Cargo.toml",
  "src-tauri/crates/extractum-process/src/lib.rs",
];

const lock = ({
  rootDependencies = ["anyhow", "serde"],
  processDependencies = null as null | string[],
  anyhowVersion = "1.0.100",
} = {}) => `version = 4

[[package]]
name = "anyhow"
version = "${anyhowVersion}"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fixed"

[[package]]
name = "extractum"
version = "0.2.0"
dependencies = [
${rootDependencies.map((name) => ` "${name}",`).join("\n")}
]

${processDependencies === null ? "" : `[[package]]
name = "extractum-process"
version = "0.2.0"
dependencies = [
${processDependencies.map((name) => ` "${name}",`).join("\n")}
]

`}[[package]]
name = "serde"
version = "1.0.228"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fixed-serde"
`;

describe("process shell diagnostic Git states", () => {
  it("freezes commit, subtree, and D blob anchors", () => {
    expect(STATE_TREE_ANCHORS).toEqual({
      A: "fd9711a041432ef420e7b09d56a46131a2a52a2a",
      D: "77e2d163ccc8bddf3ea051cb995909888cae9aba",
    });
    expect(D_BLOB_ANCHORS).toEqual({
      "src-tauri/Cargo.lock": "6368e32cd3a3853d4a7114ce256258e834bafdd4",
      "src-tauri/Cargo.toml": "c2037473a1257dd33a8e5b5fe81905e77dad084a",
      "src-tauri/crates/extractum-process/Cargo.toml": "3e078647dc293d95f401e15b8842776fae003ddb",
      "src-tauri/crates/extractum-process/src/child_process.rs": "9599017ed2ad826bc73f8e72f084042eacd8b58a",
      "src-tauri/crates/extractum-process/src/external_process.rs": "3cf7f073923b513381df09b7443090a4a41adc11",
      "src-tauri/crates/extractum-process/src/lib.rs": "4f7819ef7d2773b735b5edc61e162e4e034efb66",
      "src-tauri/crates/extractum-process/src/process_tree.rs": "365283e9f8accf4db91feca73bd8437db3b08c50",
      "src-tauri/src/lib.rs": "d84b653870eda9378c0d490894801850a97db68d",
    });
  });

  it("requires every generated B/C/E patch to carry a Cargo.lock text hunk", async () => {
    for (const state of ["B", "C", "E"]) {
      const patch = await readFile(new URL(`./states/${state}.patch`, import.meta.url), "utf8");
      expect(patch).toContain("diff --git a/src-tauri/Cargo.lock b/src-tauri/Cargo.lock");
      expect(patch).toMatch(/--- a\/src-tauri\/Cargo\.lock\r?\n\+\+\+ b\/src-tauri\/Cargo\.lock\r?\n@@ /);
    }
  });

  it("uses the exact approved A and D restore commands", () => {
    expect(aRestoreArgs()).toEqual([
      "restore",
      "--source=24c313a767a25284123b24ea3a4b8c083007c817",
      "--staged",
      "--worktree",
      "--",
      "src-tauri",
    ]);
    expect(dRestoreArgs()).toEqual([
      "restore",
      "--source=b364756c7b5768d644321afeaeb81ec04e2481a4",
      "--staged",
      "--worktree",
      "--",
      "src-tauri",
    ]);
  });

  it("accepts B only with a dependency-free empty crate and no app edge", () => {
    expect(() => validateStateManifests({
      state: "B",
      rootManifest: root(false, false),
      processManifest: emptyManifest,
      processPaths,
    })).not.toThrow();
    expect(() => validateStateManifests({
      state: "B",
      rootManifest: root(true, false),
      processManifest: emptyManifest,
      processPaths,
    })).toThrow("B must not contain the app edge");
  });

  it("accepts C only with the path edge and the same empty crate", () => {
    expect(() => validateStateManifests({
      state: "C",
      rootManifest: root(true, false),
      processManifest: emptyManifest,
      processPaths,
    })).not.toThrow();
    expect(() => validateStateManifests({
      state: "C",
      rootManifest: root(false, false),
      processManifest: emptyManifest,
      processPaths,
    })).toThrow("C must contain exactly the app path edge");
  });

  it("accepts E's four named roots and only tokio/test-util as dev input", () => {
    expect(() => validateStateManifests({
      state: "E",
      rootManifest: root(true, true),
      processManifest: eManifest,
      processPaths,
    })).not.toThrow();
  });

  it("rejects an E manifest that omits target-specific windows-sys", () => {
    expect(() => validateStateManifests({
      state: "E",
      rootManifest: root(true, true),
      processManifest: eManifest.replace("windows-sys.workspace = true\n", ""),
      processPaths,
    })).toThrow("unexpected target dependency keys");
  });

  it("rejects any moved process source in B, C, or E", () => {
    expect(() => validateStateManifests({
      state: "E",
      rootManifest: root(true, true),
      processManifest: eManifest,
      processPaths: [...processPaths, "src-tauri/crates/extractum-process/src/process_tree.rs"],
    })).toThrow("unexpected process crate paths");
  });

  it("accepts only the state-local root/process lock delta", () => {
    const baselineLock = lock();
    expect(() => validateLockDelta({
      state: "B",
      baselineLock,
      stateLock: lock({ processDependencies: [] }),
    })).not.toThrow();
    expect(() => validateLockDelta({
      state: "C",
      baselineLock,
      stateLock: lock({ rootDependencies: ["anyhow", "extractum-process", "serde"], processDependencies: [] }),
    })).not.toThrow();
    expect(() => validateLockDelta({
      state: "E",
      baselineLock,
      stateLock: lock({
        rootDependencies: ["anyhow", "extractum-process", "serde"],
        processDependencies: ["anyhow", "parking_lot", "tokio", "windows-sys"],
      }),
    })).not.toThrow();
  });

  it("rejects any third-party lock resolution drift", () => {
    expect(() => validateLockDelta({
      state: "C",
      baselineLock: lock(),
      stateLock: lock({
        rootDependencies: ["anyhow", "extractum-process", "serde"],
        processDependencies: [],
        anyhowVersion: "1.0.101",
      }),
    })).toThrow("third-party lock package changed");
  });

  it("accepts only the exact worktree-local target directory", async () => {
    const parent = await mkdtemp(path.join(os.tmpdir(), "extractum-psd-target-"));
    const worktree = path.join(parent, "attempt");
    const mainRoot = path.join(parent, "main");
    await mkdir(path.join(worktree, "src-tauri", "target"), { recursive: true });
    await mkdir(path.join(mainRoot, "src-tauri", "target"), { recursive: true });
    await expect(verifyTargetIsolation({
      metadata: {
        workspace_root: path.join(worktree, "src-tauri"),
        target_directory: path.join(worktree, "src-tauri", "target"),
      },
      worktree,
      mainRoot,
    })).resolves.toBeUndefined();
    await expect(verifyTargetIsolation({
      metadata: {
        workspace_root: path.join(worktree, "src-tauri"),
        target_directory: path.join(mainRoot, "src-tauri", "target"),
      },
      worktree,
      mainRoot,
    })).rejects.toMatchObject({ kind: "target_not_isolated" });
  });
});
