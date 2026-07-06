import { describe, expect, it } from "vitest";
import source from "./+page.svelte?raw";

describe("/projects/next inspector integration", () => {
  it("passes v11 source details and open-source action to the inspector", () => {
    expect(source).toContain("typeLabel: row.typeLabel");
    expect(source).toContain("typeDot: row.typeDot");
    expect(source).toContain("openDisabled: activeSourceOpenUrl === null");
    expect(source).toContain("onOpen: () => void openActiveSource()");
    expect(source).toContain("function youtubeProjectSourceUrl");
    expect(source).toContain("openUrl(url)");
  });
});
