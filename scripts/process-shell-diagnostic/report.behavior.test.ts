import { access, link, mkdtemp, open, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";

import { assertProtocolWorktreeStatus } from "./freeze.mjs";
import { cleanupOwnedAtomicTemps, publishReportPair, verifyReportProtocol } from "./report.mjs";
import { writeAtomicBytesExclusive } from "./runtime.mjs";

const roots: string[] = [];

async function readBytes(file: string) {
  const handle = await open(file, "r");
  try {
    return await handle.readFile();
  } finally {
    await handle.close();
  }
}

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

describe("process shell diagnostic report", () => {
  it("allows only its output and replays index-link and between-publication crashes", async () => {
    const outputRelative = "docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md";
    expect(() => assertProtocolWorktreeStatus(`?? ${outputRelative}`, [outputRelative])).not.toThrow();
    expect(() => assertProtocolWorktreeStatus(` M ${outputRelative}`, [outputRelative])).toThrow();
    expect(() => assertProtocolWorktreeStatus("?? unrelated.txt", [outputRelative])).toThrow();

    const root = await mkdtemp(path.join(tmpdir(), "extractum-report-replay-"));
    roots.push(root);
    const artifactIndex = { path: path.join(root, "artifact-index.json"), content: Buffer.from("index\n") };
    const output = path.join(root, "verification.md");
    const reportBytes = Buffer.from("report\n");
    const sessionManifest = {
      protocolRoot: root,
      protocol: {
        protocolCommit: "a".repeat(40),
        lockPath: "protocol.lock.json",
        lockBlob: "b".repeat(40),
        lockSha256: "c".repeat(64),
        protocolVersion: 1,
      },
      protocolLock: { protocolVersion: 1 },
    };
    const verified = { ...sessionManifest.protocol, protocolLock: sessionManifest.protocolLock };
    const verifyFn = vi.fn(async () => verified);
    await expect(verifyReportProtocol({
      sessionManifest,
      output: path.join(root, ...outputRelative.split("/")),
      runningProtocolRoot: root,
      verifyFn,
    })).resolves.toBeUndefined();
    expect(verifyFn).toHaveBeenCalledWith({ repoRoot: root, allowedUntrackedPaths: [outputRelative] });
    await expect(verifyReportProtocol({
      sessionManifest,
      output: path.join(root, "unrelated.md"),
      runningProtocolRoot: root,
      verifyFn,
    })).rejects.toThrow("report output must equal");
    expect(verifyFn).toHaveBeenCalledTimes(1);
    const staleOutputTemp = `${output}.4242.123e4567-e89b-12d3-a456-426614174000.tmp`;
    const unrelatedTemp = `${output}.not-owned.tmp`;
    await writeFile(staleOutputTemp, "partial");
    await writeFile(unrelatedTemp, "keep");
    await cleanupOwnedAtomicTemps(output, { processAliveFn: () => false });
    await expect(access(staleOutputTemp)).rejects.toMatchObject({ code: "ENOENT" });
    await expect(access(unrelatedTemp)).resolves.toBeUndefined();

    const strandedIndexTemp = `${artifactIndex.path}.4242.123e4567-e89b-12d3-a456-426614174001.tmp`;
    await writeFile(strandedIndexTemp, artifactIndex.content);
    await link(strandedIndexTemp, artifactIndex.path);
    await cleanupOwnedAtomicTemps(artifactIndex.path, { processAliveFn: () => false });
    await expect(access(strandedIndexTemp)).rejects.toMatchObject({ code: "ENOENT" });
    expect(await readBytes(artifactIndex.path)).toEqual(artifactIndex.content);

    const peerRoot = await mkdtemp(path.join(tmpdir(), "extractum-report-peer-replay-"));
    roots.push(peerRoot);
    const peerArtifactIndex = { path: path.join(peerRoot, "artifact-index.json"), content: artifactIndex.content };
    const peerOutput = path.join(peerRoot, "verification.md");
    let writes = 0;
    await expect(publishReportPair({ artifactIndex: peerArtifactIndex, output: peerOutput, reportBytes }, async (target, bytes) => {
      await writeAtomicBytesExclusive(target, bytes);
      writes += 1;
      if (writes === 1) throw new Error("simulated publication crash");
    })).rejects.toThrow("simulated publication crash");
    expect(await readBytes(peerArtifactIndex.path)).toEqual(peerArtifactIndex.content);
    await expect(access(peerOutput)).rejects.toMatchObject({ code: "ENOENT" });

    await publishReportPair({ artifactIndex: peerArtifactIndex, output: peerOutput, reportBytes });
    expect(await readBytes(peerArtifactIndex.path)).toEqual(peerArtifactIndex.content);
    expect(await readBytes(peerOutput)).toEqual(reportBytes);
  });
});
