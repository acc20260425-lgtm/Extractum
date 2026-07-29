import { existsSync, readFileSync, realpathSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";

export type TelegramLifecycle =
  | "baseline"
  | "8a-checkpoint-1"
  | "8a-checkpoint-2"
  | "8a-checkpoint-3"
  | "8a-checkpoint-4"
  | "8a-checkpoint-5"
  | "8a-retained"
  | "8b-checkpoint-1"
  | "8b-checkpoint-2"
  | "8b-checkpoint-3"
  | "8b-checkpoint-4"
  | "8b-checkpoint-5"
  | "8b-checkpoint-6"
  | "8b-checkpoint-7"
  | "8b-checkpoint-8"
  | "8b-preparation"
  | "8c-extracted";

export type TelegramLifecycleSource = {
  baselinePath: string;
  stagedPath: string;
  finalOwner: "extractum" | "extractum-telegram";
};

export function phase8BCheckpointNumber(
  lifecycle: TelegramLifecycle,
): number | undefined {
  if (lifecycle === "8b-preparation") return 8;
  const checkpoint = /^8b-checkpoint-([1-8])$/.exec(lifecycle)?.[1];
  return checkpoint === undefined ? undefined : Number(checkpoint);
}

const repositoryRoot = realpathSync(path.resolve(import.meta.dirname, "../.."));
const stagedRoot = "src-tauri/src/telegram_impl/";
const crateRoot = "src-tauri/crates/extractum-telegram/src/";

const phase8BFirstPhysicalOwner = new Map<string, number>([
  ["src-tauri/src/telegram_impl/lib.rs", 3],
  ["src-tauri/src/telegram_impl/dto.rs", 3],
  ["src-tauri/src/telegram_impl/media.rs", 3],
  ["src-tauri/src/telegram_impl/runtime.rs", 3],
  ["src-tauri/src/telegram_impl/session.rs", 3],
  ["src-tauri/src/telegram_impl/error.rs", 4],
  ["src-tauri/src/telegram_impl/live/mod.rs", 4],
  ["src-tauri/src/telegram_impl/live/avatar.rs", 4],
  ["src-tauri/src/telegram_impl/live/peer.rs", 4],
  ["src-tauri/src/telegram_impl/live/messages.rs", 5],
  ["src-tauri/src/telegram_impl/live/topics.rs", 5],
  ["src-tauri/src/telegram_impl/takeout/mod.rs", 7],
  ["src-tauri/src/telegram_impl/takeout/types.rs", 7],
  ["src-tauri/src/telegram_impl/takeout/transport.rs", 7],
  ["src-tauri/src/telegram_impl/takeout/export_dc.rs", 7],
  ["src-tauri/src/telegram_impl/takeout/operations.rs", 7],
  ["src-tauri/src/telegram_impl/takeout/pagination.rs", 7],
  ["src-tauri/src/telegram_impl/takeout/raw_parse.rs", 7],
  ["src-tauri/src/telegram_impl/takeout/forum_topics.rs", 7],
]);

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

export type TelegramContentAddressedSection = {
  relativePath: string;
  startHeading: string;
  endMarker: string;
  normalizedLfBytes: number;
  sha256: string;
};

export function readTelegramContentAddressedSection(
  authority: TelegramContentAddressedSection,
): string {
  if (
    !authority.startHeading
    || authority.startHeading.includes("\n")
    || !authority.endMarker.startsWith("\n")
    || authority.normalizedLfBytes <= 0
    || !/^[a-f0-9]{64}$/.test(authority.sha256)
  ) {
    throw new Error("Malformed Telegram content-addressed authority");
  }

  const source = readTelegramContractFile(authority.relativePath);
  const headingMarker = `${authority.startHeading}\n`;
  const headingMatches = source.split(headingMarker).length - 1;
  if (
    headingMatches !== 1
    || !(
      source.startsWith(headingMarker)
      || source.includes(`\n${headingMarker}`)
    )
  ) {
    throw new Error(
      `Malformed Telegram authority start heading: ${authority.startHeading}`,
    );
  }
  const startIndex = source.indexOf(authority.startHeading);
  const endIndex = source.indexOf(
    authority.endMarker,
    startIndex + authority.startHeading.length,
  );
  if (
    startIndex < 0
    || endIndex <= startIndex
  ) {
    throw new Error(
      `Malformed Telegram authority end marker: ${authority.endMarker}`,
    );
  }

  const section = source.slice(startIndex, endIndex);
  const byteLength = Buffer.byteLength(section, "utf8");
  if (byteLength !== authority.normalizedLfBytes) {
    throw new Error(
      `Telegram authority byte length drifted: ${byteLength} != ${authority.normalizedLfBytes}`,
    );
  }
  const actualSha256 = createHash("sha256").update(section).digest("hex");
  if (actualSha256 !== authority.sha256) {
    throw new Error(
      `Telegram authority SHA-256 drifted: ${actualSha256} != ${authority.sha256}`,
    );
  }
  return section;
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

  const phase8BCheckpoint = phase8BCheckpointNumber(lifecycle);
  if (phase8BCheckpoint !== undefined) {
    const firstPhysicalOwner = phase8BFirstPhysicalOwner.get(source.stagedPath);
    if (firstPhysicalOwner === undefined) {
      throw new Error(
        `Unknown Phase 8B staged owner path: ${source.stagedPath}`,
      );
    }
    if (phase8BCheckpoint < firstPhysicalOwner) {
      return preparedLeaves.get(source.stagedPath)?.preparedPath
        ?? source.baselinePath;
    }
    return source.stagedPath;
  }
  if (lifecycle === "8c-extracted") {
    if (!phase8BFirstPhysicalOwner.has(source.stagedPath)) {
      throw new Error(
        `Unknown Phase 8C staged owner path: ${source.stagedPath}`,
      );
    }
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
