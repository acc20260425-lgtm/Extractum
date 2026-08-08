import { access, link, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";

import { assertProtocolWorktreeStatus } from "./freeze.mjs";
import { cleanupOwnedAtomicTemps, publishReportPair } from "./report.mjs";
import { writeAtomicBytesExclusive } from "./runtime.mjs";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

describe("process shell diagnostic report", () => {
  it("allows only its output and replays index-link and between-publication crashes", async () => {
    const outputRelative = "docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md";
    expect(() => assertProtocolWorktreeStatus(`?? ${outputRelative}`, [outputRelative])).not.toThrow();
    expect(() => assertProtocolWorktreeStatus("?? unrelated.txt", [outputRelative])).toThrow();

    const root = await mkdtemp(path.join(tmpdir(), "extractum-report-replay-"));
    roots.push(root);
    const artifactIndex = { path: path.join(root, "artifact-index.json"), content: Buffer.from("index\n") };
    const output = path.join(root, "verification.md");
    const reportBytes = Buffer.from("report\n");
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

    let writes = 0;
    await expect(publishReportPair({ artifactIndex, output, reportBytes }, async (target, bytes) => {
      await writeAtomicBytesExclusive(target, bytes);
      writes += 1;
      if (writes === 1) throw new Error("simulated publication crash");
    })).rejects.toThrow("simulated publication crash");

    await publishReportPair({ artifactIndex, output, reportBytes });
    expect(await readFile(artifactIndex.path)).toEqual(artifactIndex.content);
    expect(await readFile(output)).toEqual(reportBytes);
  });
});
