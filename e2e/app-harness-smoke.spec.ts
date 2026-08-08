import { expect, test } from "@playwright/test";

import { installTauriScenario } from "./fixtures/tauri";

test("app e2e harness smoke mounts the real application shell", async ({ page }) => {
  await installTauriScenario(page);

  await page.goto("/");

  await expect(page.getByRole("main")).toBeVisible({ timeout: 20_000 });
});
