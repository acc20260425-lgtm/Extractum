import { describe, expect, it } from "vitest";
import libSource from "../../src-tauri/src/lib.rs?raw";
import childProcessSource from "../../src-tauri/src/child_process.rs?raw";
import sidecarSource from "../../src-tauri/src/gemini_browser/sidecar.rs?raw";
import cdpChromeSource from "../../src-tauri/src/gemini_browser/cdp_chrome.rs?raw";

describe("hidden child process contract", () => {
  it("defines the Windows-only hidden-console helper without affecting Gemini browser processes", () => {
    expect(libSource).toContain("mod child_process;");
    expect(childProcessSource).toContain(
      "pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;",
    );
    expect(childProcessSource).toMatch(
      /#\[cfg\(windows\)\][\s\S]*creation_flags\(CREATE_NO_WINDOW\)/,
    );
    expect(childProcessSource).toContain("command\n}");
    expect(childProcessSource).toContain("assert_eq!(CREATE_NO_WINDOW, 0x0800_0000)");
    expect(sidecarSource).not.toContain("hide_console_window");
    expect(cdpChromeSource).not.toContain("hide_console_window");
  });
});
