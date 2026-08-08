import { expect, test } from "@playwright/test";

import { installTauriScenario } from "./fixtures/tauri";

test("mobile-menu-trigger-responsive-visibility", async ({ page }) => {
  await installTauriScenario(page);
  const menu = page.getByRole("button", { name: "Open navigation" });

  await page.setViewportSize({ width: 1100, height: 800 });
  await page.goto("/");
  await expect(page.getByRole("main")).toBeVisible({ timeout: 20_000 });
  await expect(menu).toBeHidden();

  await page.setViewportSize({ width: 700, height: 800 });
  await expect(menu).toBeVisible();
  await menu.click();
  await expect(page.getByRole("complementary", { name: "App sidebar" })).toBeVisible();
});
