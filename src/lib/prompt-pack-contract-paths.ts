import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dirname, "../..");

export function readPromptPackDomainSource(
  relativePath: string,
  preparedAppRelativePath = relativePath,
): string {
  const cratePath = resolve(repositoryRoot, "src-tauri/crates/extractum-prompt-packs/src", relativePath);
  const preparedPath = resolve(repositoryRoot, "src-tauri/src/prompt_packs", preparedAppRelativePath);
  const rootTranslation = relativePath === "lib.rs" && preparedAppRelativePath === "mod.rs";
  if (existsSync(cratePath)) {
    if (!rootTranslation && existsSync(preparedPath)) {
      throw new Error(`duplicate Prompt Packs domain owner for ${relativePath}`);
    }
    return readFileSync(cratePath, "utf8");
  }
  if (!existsSync(preparedPath)) {
    throw new Error(`missing Prompt Packs domain owner for ${relativePath}`);
  }
  return readFileSync(preparedPath, "utf8");
}

export function readPromptPackAppFacade(): string {
  return readFileSync(resolve(repositoryRoot, "src-tauri/src/prompt_packs/mod.rs"), "utf8");
}
