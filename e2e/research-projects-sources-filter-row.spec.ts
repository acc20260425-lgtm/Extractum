import { expect, test } from "@playwright/test";

import { installTauriScenario } from "./fixtures/tauri";

const project = {
  id: 7,
  name: "Responsive sources",
  description: "Browser contract fixture",
  source_count: 0,
  material_count: 0,
  status: "ready",
  last_run_at: null,
  pinned: false,
  archived: false,
  updated_at: 1_786_000_000,
};

test("filters-available-across-responsive-layouts", async ({ page }) => {
  await installTauriScenario(page, {
    invokes: {
      list_projects: [],
      list_research_projects: [project],
      list_project_sources: [],
      get_project_data_range: { from: null, to: null },
      list_analysis_prompt_templates: [],
    },
  });

  await page.setViewportSize({ width: 1280, height: 850 });
  await page.goto("/projects/next");
  const projectRow = page.locator('[role="option"]').filter({ hasText: "Responsive sources" });
  await expect(projectRow).toBeVisible({ timeout: 20_000 });
  await projectRow.click();
  const filters = page.locator(".sources-filter-bar__filters-btn");
  await filters.click();
  const search = page.locator(".sources-filter-row__search input");
  const typeFilter = page.locator(".sources-filter-row__type button");
  await expect(search).toBeVisible();
  await expect(typeFilter).toBeVisible();

  await page.setViewportSize({ width: 720, height: 850 });
  await expect(filters).toBeVisible();
  await expect(search).toBeVisible();
  await expect(typeFilter).toBeHidden();

  await page.setViewportSize({ width: 430, height: 850 });
  await expect(filters).toBeVisible();
  await expect(search).toBeVisible();
});
