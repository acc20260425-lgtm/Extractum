import { defineConfig } from "@playwright/test";

const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";

export default defineConfig({
  testDir: ".",
  fullyParallel: false,
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:4178",
    browserName: "chromium",
    screenshot: { mode: "only-on-failure", fullPage: true },
    trace: "retain-on-first-failure",
  },
  webServer: {
    command: `${npmCommand} run dev -- --host 127.0.0.1 --port 4178 --strictPort`,
    url: "http://127.0.0.1:4178",
    timeout: 120_000,
    reuseExistingServer: !process.env.CI,
  },
});
