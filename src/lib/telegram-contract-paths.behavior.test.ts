import { expect, it } from "vitest";

import {
  phase8BCheckpointNumber,
  resolveTelegramLifecyclePath,
  telegramLifecycleFromStatus,
  type TelegramLifecycle,
  type TelegramLifecycleSource,
} from "./telegram-contract-paths";

it("recognizes the retained Phase 8 lifecycle vocabulary and paths", () => {
    expect([
      phase8BCheckpointNumber("8b-checkpoint-1"),
      phase8BCheckpointNumber("8b-checkpoint-8"),
      phase8BCheckpointNumber("8b-preparation"),
      phase8BCheckpointNumber("8a-retained"),
      phase8BCheckpointNumber("8c-extracted"),
      phase8BCheckpointNumber("unknown" as TelegramLifecycle),
    ]).toEqual([1, 8, 8, undefined, undefined, undefined]);

    expect(telegramLifecycleFromStatus("8A preparation Checkpoint 3 retained")).toBe("8a-checkpoint-3");
    expect(telegramLifecycleFromStatus("8B preparation Checkpoint 7 retained")).toBe("8b-checkpoint-7");
    expect(telegramLifecycleFromStatus("8B preparation retained; 8C pending")).toBe("8b-preparation");
    expect(telegramLifecycleFromStatus("done: retained")).toBe("8c-extracted");
    expect(() => telegramLifecycleFromStatus("unknown Phase 8 status")).toThrow(/Unsupported Phase 8 status/);

    const extracted: TelegramLifecycleSource = {
      baselinePath: "src-tauri/src/telegram/dto.rs",
      stagedPath: "src-tauri/src/telegram_impl/dto.rs",
      finalOwner: "extractum-telegram",
    };
    expect(resolveTelegramLifecyclePath(extracted, "8a-checkpoint-2")).toBe(extracted.baselinePath);
    expect(resolveTelegramLifecyclePath(extracted, "8a-checkpoint-3")).toBe("src-tauri/src/telegram/dto.rs");
    expect(resolveTelegramLifecyclePath(extracted, "8b-checkpoint-2")).toBe(extracted.baselinePath);
    expect(resolveTelegramLifecyclePath(extracted, "8b-checkpoint-3")).toBe(extracted.stagedPath);
    expect(resolveTelegramLifecyclePath(extracted, "8c-extracted")).toBe("src-tauri/crates/extractum-telegram/src/dto.rs");
    expect(resolveTelegramLifecyclePath(extracted, "unknown" as TelegramLifecycle)).toBe(extracted.baselinePath);

    expect(resolveTelegramLifecyclePath({ ...extracted, finalOwner: "extractum" }, "8c-extracted"))
      .toBe(extracted.baselinePath);
    expect(() => resolveTelegramLifecyclePath(
      { ...extracted, stagedPath: "src-tauri/src/telegram_impl/unknown.rs" },
      "8c-extracted",
    )).toThrow(/Unknown Phase 8C staged owner path/);
});
