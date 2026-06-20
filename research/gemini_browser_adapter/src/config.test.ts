import { mkdtemp, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { DEFAULT_DOM_CONTRACT_CONFIG, loadDomContractConfig } from "./config";

describe("DOM contract config", () => {
  it("returns defaults when config is absent", async () => {
    const config = await loadDomContractConfig("missing-config.json");
    expect(config.answerSelectors).toEqual(DEFAULT_DOM_CONTRACT_CONFIG.answerSelectors);
  });

  it("loads local selector overrides", async () => {
    const dir = await mkdtemp(path.join(os.tmpdir(), "gemini-dom-contract-"));
    const configPath = path.join(dir, "gemini-dom-contract.config.json");
    await writeFile(
      configPath,
      JSON.stringify({
        promptSelectors: ["[data-custom-prompt]"],
        sendSelectors: ["[data-custom-send]"],
        answerSelectors: ["[data-custom-answer]"],
      }),
      "utf8",
    );

    const config = await loadDomContractConfig(configPath);
    expect(config.promptSelectors).toContain("[data-custom-prompt]");
    expect(config.sendSelectors).toContain("[data-custom-send]");
    expect(config.answerSelectors).toContain("[data-custom-answer]");
  });
});
