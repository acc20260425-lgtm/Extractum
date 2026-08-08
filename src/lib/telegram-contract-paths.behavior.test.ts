import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  createTelegramContractPathResolver,
  phase8BCheckpointNumber,
  resolveTelegramLifecyclePath,
  telegramLifecycleFromStatus,
  type TelegramLifecycle,
  type TelegramLifecycleSource,
} from "./telegram-contract-paths";

const lifecycleStatuses = [
  ["design drafted; awaiting owner approval", "baseline"],
  ["design approved; implementation not started", "baseline"],
  ["8A preparation Checkpoint 1 retained", "8a-checkpoint-1"],
  ["8A preparation Checkpoint 2 retained", "8a-checkpoint-2"],
  ["8A preparation Checkpoint 3 retained", "8a-checkpoint-3"],
  ["8A preparation Checkpoint 4 retained", "8a-checkpoint-4"],
  ["8A preparation Checkpoint 5 retained", "8a-checkpoint-5"],
  ["8A preparation retained", "8a-retained"],
  ["8B preparation Checkpoint 1 retained", "8b-checkpoint-1"],
  ["8B preparation Checkpoint 2 retained", "8b-checkpoint-2"],
  ["8B preparation Checkpoint 3 retained", "8b-checkpoint-3"],
  ["8B preparation Checkpoint 4 retained", "8b-checkpoint-4"],
  ["8B preparation Checkpoint 5 retained", "8b-checkpoint-5"],
  ["8B preparation Checkpoint 6 retained", "8b-checkpoint-6"],
  ["8B preparation Checkpoint 7 retained", "8b-checkpoint-7"],
  ["8B preparation Checkpoint 8 retained", "8b-checkpoint-8"],
  ["8B preparation retained; 8C pending", "8b-preparation"],
  ["done: retained", "8c-extracted"],
  ["not retained", "baseline"],
] as const satisfies ReadonlyArray<readonly [string, TelegramLifecycle]>;

const invalidLifecycleStatuses = [
  "",
  "8A preparation Checkpoint 0 retained",
  "8A preparation Checkpoint 6 retained",
  "8B preparation Checkpoint 0 retained",
  "8B preparation Checkpoint 9 retained",
  "8B preparation checkpoint 1 retained",
  "Approved; 8B preparation Checkpoint 1 retained",
  "8B preparation Checkpoint 1",
  "done: retained ",
  "unknown Phase 8 status",
] as const;

function acceptedUnsafePaths(
  resolve: (relativePath: string) => string,
  rejectedPaths: readonly string[],
) {
  return rejectedPaths.filter((relativePath) => {
    try {
      resolve(relativePath);
      return true;
    } catch {
      return false;
    }
  });
}

describe("Telegram contract paths", () => {
  it("recognizes the retained Phase 8 lifecycle vocabulary and paths", () => {
    for (const statusAndLifecycle of lifecycleStatuses) {
      expect(telegramLifecycleFromStatus(statusAndLifecycle[0]), statusAndLifecycle[0])
        .toBe(statusAndLifecycle[1]);
    }
    for (const status of invalidLifecycleStatuses) {
      expect(() => telegramLifecycleFromStatus(status), status).toThrow(/Unsupported Phase 8 status/);
    }
    for (let checkpoint = 1; checkpoint <= 8; checkpoint += 1) {
      expect(phase8BCheckpointNumber(`8b-checkpoint-${checkpoint}` as TelegramLifecycle)).toBe(checkpoint);
    }
    for (const nonCheckpointLifecycle of [
      "baseline",
      "8a-checkpoint-1",
      "8a-checkpoint-2",
      "8a-checkpoint-3",
      "8a-checkpoint-4",
      "8a-checkpoint-5",
      "8a-retained",
      "8c-extracted",
      "unknown" as TelegramLifecycle,
    ] as const satisfies ReadonlyArray<TelegramLifecycle>) {
      expect(phase8BCheckpointNumber(nonCheckpointLifecycle), nonCheckpointLifecycle).toBeUndefined();
    }
    expect(phase8BCheckpointNumber("8b-preparation")).toBe(8);

    const extracted: TelegramLifecycleSource = {
      baselinePath: "src-tauri/src/telegram/dto.rs",
      stagedPath: "src-tauri/src/telegram_impl/dto.rs",
      finalOwner: "extractum-telegram",
    };
    const expectedPaths = new Map<TelegramLifecycle, string>([
      ["baseline", extracted.baselinePath],
      ["8a-checkpoint-1", extracted.baselinePath],
      ["8a-checkpoint-2", extracted.baselinePath],
      ["8a-checkpoint-3", "src-tauri/src/telegram/dto.rs"],
      ["8a-checkpoint-4", "src-tauri/src/telegram/dto.rs"],
      ["8a-checkpoint-5", "src-tauri/src/telegram/dto.rs"],
      ["8a-retained", "src-tauri/src/telegram/dto.rs"],
      ["8b-checkpoint-1", extracted.baselinePath],
      ["8b-checkpoint-2", extracted.baselinePath],
      ["8b-checkpoint-3", extracted.stagedPath],
      ["8b-checkpoint-4", extracted.stagedPath],
      ["8b-checkpoint-5", extracted.stagedPath],
      ["8b-checkpoint-6", extracted.stagedPath],
      ["8b-checkpoint-7", extracted.stagedPath],
      ["8b-checkpoint-8", extracted.stagedPath],
      ["8b-preparation", extracted.stagedPath],
      ["8c-extracted", "src-tauri/crates/extractum-telegram/src/dto.rs"],
    ]);
    for (const pathLifecycle of expectedPaths.keys()) {
      expect(resolveTelegramLifecyclePath(extracted, pathLifecycle), pathLifecycle)
        .toBe(expectedPaths.get(pathLifecycle));
    }
    expect(resolveTelegramLifecyclePath(extracted, "unknown" as TelegramLifecycle)).toBe(extracted.baselinePath);

    expect(resolveTelegramLifecyclePath({ ...extracted, finalOwner: "extractum" }, "8c-extracted"))
      .toBe(extracted.baselinePath);
    expect(() => resolveTelegramLifecyclePath(
      { ...extracted, stagedPath: "src-tauri/src/telegram_impl/unknown.rs" },
      "8c-extracted",
    )).toThrow(/Unknown Phase 8C staged owner path/);
  });

  it("reads only existing repository-relative files and rejects escapes", () => {
    const root = path.resolve("telegram-contract-path-fixture");
    const validPath = path.join(root, "docs", "valid.md");
    const outside = path.resolve(root, "..", "outside");
    const crossDrive = path.parse(root).root.toLowerCase() === "c:\\"
      ? "D:\\outside\\absolute.md"
      : "C:\\outside\\absolute.md";
    const existing = new Set([
      validPath,
      path.join(root, "links", "parent"),
      path.join(root, "links", "prefix"),
      path.join(root, "links", "absolute"),
      path.join(root, "links", "symlink"),
      path.join(root, "links", "junction"),
    ]);
    const canonical = new Map([
      [path.join(root, "links", "parent"), path.resolve(root, "..")],
      [path.join(root, "links", "prefix"), path.join(outside, "prefix.md")],
      [path.join(root, "links", "absolute"), crossDrive],
      [path.join(root, "links", "symlink"), path.join(outside, "symlink.md")],
      [path.join(root, "links", "junction"), path.join(outside, "junction.md")],
    ]);
    const resolver = createTelegramContractPathResolver({
      root,
      existsSync: (selected) => existing.has(String(selected)),
      realpathSync: (selected) => canonical.get(String(selected)) ?? String(selected),
      readFileSync: (selected) => {
        if (String(selected) !== validPath) throw new Error(`unexpected read: ${String(selected)}`);
        return "valid\r\nrepository\rfile\n";
      },
    });

    expect(resolver.resolve("docs/valid.md")).toBe(validPath);
    expect(resolver.read("docs/valid.md")).toBe("valid\nrepository\nfile\n");
    const rejectedPaths = [
      "",
      ".",
      "..",
      "../outside.md",
      "src/../outside.md",
      "src/./inside.md",
      path.join(root, "docs", "valid.md"),
      "docs\\valid.md",
      "docs/missing.md",
      "links/parent",
      "links/prefix",
      "links/absolute",
      "links/symlink",
      "links/junction",
    ];
    expect(acceptedUnsafePaths(resolver.resolve, rejectedPaths)).toEqual([]);
  });

  it("fails the path-safety contract for an early-return resolver mutation", () => {
    const root = path.resolve("telegram-contract-path-fixture");
    const earlyReturnMutation = (relativePath: string) => path.resolve(root, relativePath);

    expect(acceptedUnsafePaths(earlyReturnMutation, ["", ".", "..", "../outside.md"]))
      .toEqual(["", ".", "..", "../outside.md"]);
  });
});
