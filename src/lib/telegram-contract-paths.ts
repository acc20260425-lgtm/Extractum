import { existsSync, readFileSync, realpathSync } from "node:fs";
import path from "node:path";

export type TelegramLifecycle =
  | "baseline"
  | "8a-checkpoint-1"
  | "8a-checkpoint-2"
  | "8a-checkpoint-3"
  | "8a-checkpoint-4"
  | "8a-checkpoint-5"
  | "8a-retained"
  | "8b-preparation"
  | "8c-extracted";

export type TelegramLifecycleSource = {
  baselinePath: string;
  stagedPath: string;
  finalOwner: "extractum" | "extractum-telegram";
};

const repositoryRoot = realpathSync(path.resolve(import.meta.dirname, "../.."));
const stagedRoot = "src-tauri/src/telegram_impl/";
const crateRoot = "src-tauri/crates/extractum-telegram/src/";

const preparedLeaves = new Map<
  string,
  { checkpoint: number; preparedPath: string }
>([
  [
    "src-tauri/src/telegram_impl/dto.rs",
    { checkpoint: 3, preparedPath: "src-tauri/src/telegram/dto.rs" },
  ],
  [
    "src-tauri/src/telegram_impl/media.rs",
    { checkpoint: 3, preparedPath: "src-tauri/src/telegram/media.rs" },
  ],
  [
    "src-tauri/src/telegram_impl/session.rs",
    { checkpoint: 4, preparedPath: "src-tauri/src/telegram/session.rs" },
  ],
  [
    "src-tauri/src/telegram_impl/runtime.rs",
    { checkpoint: 5, preparedPath: "src-tauri/src/telegram/runtime.rs" },
  ],
]);

const lifecycleByStatus = new Map<string, TelegramLifecycle>([
  ["design drafted; awaiting owner approval", "baseline"],
  ["design approved; implementation not started", "baseline"],
  ["8A preparation Checkpoint 1 retained", "8a-checkpoint-1"],
  ["8A preparation Checkpoint 2 retained", "8a-checkpoint-2"],
  ["8A preparation Checkpoint 3 retained", "8a-checkpoint-3"],
  ["8A preparation Checkpoint 4 retained", "8a-checkpoint-4"],
  ["8A preparation Checkpoint 5 retained", "8a-checkpoint-5"],
  ["8A preparation retained", "8a-retained"],
  ["8B preparation retained; 8C pending", "8b-preparation"],
  ["done: retained", "8c-extracted"],
  ["not retained", "baseline"],
]);

function assertRepositoryRelative(relativePath: string): string {
  if (
    !relativePath
    || path.isAbsolute(relativePath)
    || relativePath.includes("\\")
  ) {
    throw new Error(
      `Telegram contract path must be repository-relative: ${relativePath}`,
    );
  }
  if (
    relativePath.split("/").some((segment) => segment === "." || segment === "..")
  ) {
    throw new Error(
      `Telegram contract path contains a dot path segment: ${relativePath}`,
    );
  }

  const selected = path.resolve(repositoryRoot, relativePath);
  const relative = path.relative(repositoryRoot, selected);
  if (
    relative === ""
    || relative === ".."
    || relative.startsWith(`..${path.sep}`)
    || path.isAbsolute(relative)
  ) {
    throw new Error(
      `Telegram contract path escapes repository root: ${relativePath}`,
    );
  }
  return selected;
}

export function normalizeTelegramContractSourceText(source: string): string {
  return source.replace(/\r\n?/g, "\n");
}

export function resolveTelegramContractPath(relativePath: string): string {
  const selected = assertRepositoryRelative(relativePath);
  if (!existsSync(selected)) {
    throw new Error(`Telegram contract path is missing: ${relativePath}`);
  }

  const realSelected = realpathSync(selected);
  const realRelative = path.relative(repositoryRoot, realSelected);
  if (
    realRelative === ".."
    || realRelative.startsWith(`..${path.sep}`)
    || path.isAbsolute(realRelative)
  ) {
    throw new Error(
      `Telegram contract path escapes repository root: ${relativePath}`,
    );
  }
  return realSelected;
}

export function readTelegramContractFile(relativePath: string): string {
  return normalizeTelegramContractSourceText(
    readFileSync(resolveTelegramContractPath(relativePath), "utf8"),
  );
}

export function resolveTelegramLifecyclePath(
  source: TelegramLifecycleSource,
  lifecycle: TelegramLifecycle,
): string {
  assertRepositoryRelative(source.baselinePath);
  assertRepositoryRelative(source.stagedPath);

  if (source.finalOwner === "extractum") {
    return source.baselinePath;
  }
  if (!source.stagedPath.startsWith(stagedRoot)) {
    throw new Error(
      `Future Telegram owner lies outside the staging tree: ${source.stagedPath}`,
    );
  }

  if (lifecycle === "8b-preparation") {
    return source.stagedPath;
  }
  if (lifecycle === "8c-extracted") {
    return `${crateRoot}${source.stagedPath.slice(stagedRoot.length)}`;
  }

  const checkpoint =
    lifecycle === "8a-retained"
      ? 5
      : /^8a-checkpoint-(\d)$/.exec(lifecycle)?.[1];
  const prepared = preparedLeaves.get(source.stagedPath);
  if (
    prepared
    && checkpoint !== undefined
    && Number(checkpoint) >= prepared.checkpoint
  ) {
    return prepared.preparedPath;
  }
  return source.baselinePath;
}

export function telegramLifecycleFromStatus(status: string): TelegramLifecycle {
  const lifecycle = lifecycleByStatus.get(status);
  if (!lifecycle) {
    throw new Error(`Unsupported Phase 8 status: ${status}`);
  }
  return lifecycle;
}
