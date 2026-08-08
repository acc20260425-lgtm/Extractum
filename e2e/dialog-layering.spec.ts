import { expect, test } from "@playwright/test";

import { installTauriScenario } from "./fixtures/tauri";

test("dialog-content-visible-interactive-above-overlay", async ({ page }) => {
  await installTauriScenario(page, {
    invokes: {
      list_accounts: [],
      get_youtube_settings: {
        authEnabled: false,
        preferredCaptionsLanguage: "original",
        delayBetweenRequestsMs: 1000,
        maxParallelVideoSyncs: 1,
        maxParallelCommentSyncs: 1,
        pauseOnAuthChallenge: true,
        dailySoftLimit: 0,
        retryBackoffMs: 3000,
        stopAfterConsecutiveFailures: 3,
      },
      get_youtube_auth_status: { enabled: false, hasCookies: false, message: "Not configured" },
    },
  });

  await page.goto("/accounts");
  await page.getByRole("button", { name: "Add", exact: true }).click();

  const dialog = page.getByRole("dialog", { name: "New Telegram account" });
  const overlay = page.locator(".dialog-backdrop");
  await expect(dialog).toBeVisible();
  await expect(overlay).toBeVisible();
  const contentLayer = await dialog.evaluate((element) => Number(getComputedStyle(element).zIndex));
  const overlayLayer = await overlay.evaluate((element) => Number(getComputedStyle(element).zIndex));
  expect(contentLayer).toBeGreaterThan(overlayLayer);

  await page.getByPlaceholder("Personal").fill("Browser fixture");
  await expect(page.getByPlaceholder("Personal")).toHaveValue("Browser fixture");
  await page.getByRole("button", { name: "Close dialog" }).click();
  await expect(dialog).toBeHidden();
});
