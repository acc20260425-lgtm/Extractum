import { readFile } from "node:fs/promises";

export type DomContractConfig = {
  promptSelectors: string[];
  sendSelectors: string[];
  answerSelectors: string[];
  minPromptScore: number;
  minSendScore: number;
};

export const DEFAULT_DOM_CONTRACT_CONFIG: DomContractConfig = {
  promptSelectors: [],
  sendSelectors: [],
  answerSelectors: ["[data-testid=\"assistant-answer\"]"],
  minPromptScore: 5,
  minSendScore: 4,
};

export async function loadDomContractConfig(
  configPath = "research/gemini_browser_adapter/gemini-dom-contract.config.json",
): Promise<DomContractConfig> {
  const raw = await readFile(configPath, "utf8").catch(() => null);
  if (!raw) return DEFAULT_DOM_CONTRACT_CONFIG;
  const parsed = JSON.parse(raw) as Partial<DomContractConfig>;
  return {
    promptSelectors: parsed.promptSelectors ?? DEFAULT_DOM_CONTRACT_CONFIG.promptSelectors,
    sendSelectors: parsed.sendSelectors ?? DEFAULT_DOM_CONTRACT_CONFIG.sendSelectors,
    answerSelectors: parsed.answerSelectors ?? DEFAULT_DOM_CONTRACT_CONFIG.answerSelectors,
    minPromptScore: parsed.minPromptScore ?? DEFAULT_DOM_CONTRACT_CONFIG.minPromptScore,
    minSendScore: parsed.minSendScore ?? DEFAULT_DOM_CONTRACT_CONFIG.minSendScore,
  };
}
