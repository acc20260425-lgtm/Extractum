import { expect, it } from "vitest";

it("keeps Settings focused on LLM configuration", async () => {
  const modulePath = "./settings-focus";
  const { SETTINGS_FOCUS } = await import(/* @vite-ignore */ modulePath);

  expect(SETTINGS_FOCUS.sections).toEqual(["llm-provider-profiles", "provider-test-runs"]);
  expect(SETTINGS_FOCUS.description).toBe("Settings stay focused on LLM provider profiles and test runs.");
});
