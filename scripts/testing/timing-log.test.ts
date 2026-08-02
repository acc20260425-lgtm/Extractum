import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createTimingRow,
  formatCommand,
  recordTimingBestEffort,
} from "./timing-log.mjs";

const roots: string[] = [];
afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

describe("minimal timing log", () => {
  it("writes exactly the approved five fields", async () => {
    const repoRoot = await mkdtemp(path.join(tmpdir(), "extractum-timing-"));
    roots.push(repoRoot);
    const row = createTimingRow({
      command: "node example.mjs --flag",
      startedAt: "2026-08-02T10:11:12.123Z",
      duration: 17.8,
      exitCode: 0,
      commit: "a".repeat(40),
    });

    await recordTimingBestEffort(row, { repoRoot });
    const text = await readFile(path.join(repoRoot, "artifacts/testing/timings.jsonl"), "utf8");
    expect(Object.keys(JSON.parse(text.trim()))).toEqual([
      "command", "startedAt", "duration", "exitCode", "commit",
    ]);
    expect(JSON.parse(text.trim()).duration).toBe(18);
  });

  it("quotes whitespace and embedded quotes deterministically", () => {
    expect(formatCommand("node", ["a b.mjs", "--name", 'a"b'])).toBe(
      'node "a b.mjs" --name "a\\"b"',
    );
  });

  it("warns without throwing when persistence fails", async () => {
    const warn = vi.fn();
    const appendFile = vi.fn().mockRejectedValue(new Error("disk unavailable"));
    const row = createTimingRow({
      command: "node example.mjs", startedAt: "2026-08-02T10:11:12.123Z",
      duration: 1, exitCode: 7, commit: "b".repeat(40),
    });
    await expect(recordTimingBestEffort(row, { appendFile, mkdir: vi.fn(), warn }))
      .resolves.toBe(false);
    expect(warn).toHaveBeenCalledOnce();
  });
});
