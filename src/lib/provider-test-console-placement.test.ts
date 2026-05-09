import { describe, expect, it } from "vitest";
import settingsPageSource from "../routes/settings/+page.svelte?raw";

describe("provider test console placement", () => {
  it("opens the provider console from the LLM profile actions", () => {
    const profilesStart = settingsPageSource.indexOf("<h2>LLM profiles</h2>");
    const actionsStart = settingsPageSource.indexOf('<div class="actions">', profilesStart);
    const profilesEnd = settingsPageSource.indexOf("</section>", profilesStart);

    expect(profilesStart).toBeGreaterThanOrEqual(0);
    expect(actionsStart).toBeGreaterThan(profilesStart);
    expect(profilesEnd).toBeGreaterThan(actionsStart);

    const profileActionsSource = settingsPageSource.slice(actionsStart, profilesEnd);

    expect(profileActionsSource).toContain("Clear API key");
    expect(profileActionsSource).toContain("onclick={openTestDialog}");
    expect(profileActionsSource).toContain("Open test");
  });

  it("does not keep a duplicate smoke test panel outside the console dialog", () => {
    expect(settingsPageSource).not.toContain("<h2>Smoke test</h2>");
    expect(settingsPageSource).not.toContain("<span class=\"page-eyebrow\">Provider test</span>");
    expect(settingsPageSource).not.toContain('title="Latest response"');
    expect(settingsPageSource).not.toContain('className="summary-strip"');
    expect(settingsPageSource).not.toContain("Open test console");
  });

  it("shows provider test status inside the console dialog", () => {
    const dialogStart = settingsPageSource.indexOf('title="Provider Test Console"');
    const dialogEnd = settingsPageSource.indexOf("</DesktopDialog>", dialogStart);

    expect(dialogStart).toBeGreaterThanOrEqual(0);
    expect(dialogEnd).toBeGreaterThan(dialogStart);

    const dialogSource = settingsPageSource.slice(dialogStart, dialogEnd);

    expect(dialogSource).toContain("{#if testStatus}");
    expect(dialogSource).toContain("{testStatus}");
    expect(dialogSource).toContain(
      'tone={testStatus.startsWith("Provider test failed") || testStatus.startsWith("Error") ? "error" : "default"}',
    );
  });
});
