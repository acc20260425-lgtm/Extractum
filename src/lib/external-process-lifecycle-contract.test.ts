import { describe, expect, it } from "vitest";
import libSource from "../../src-tauri/src/lib.rs?raw";
import coordinatorSource from "../../src-tauri/src/external_process.rs?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

describe("external process lifecycle contract", () => {
  it("registers a runtime-only shutdown coordinator with explicit timing budgets", () => {
    const lib = normalized(libSource);
    const coordinator = normalized(coordinatorSource);

    expect(lib).toContain("mod external_process;");
    expect(lib).toContain(".manage(ExternalProcessShutdownState::new())");
    expect(coordinator).toContain("Running");
    expect(coordinator).toContain("ShuttingDown");
    expect(coordinator).toContain("Completed");
    expect(coordinator).toContain("GRACEFUL_SHUTDOWN_TIMEOUT");
    expect(coordinator).toContain("SHUTDOWN_WATCHDOG_TIMEOUT");
    expect(coordinator).not.toMatch(/serde|Serialize|Deserialize|tauri::command/i);
  });
});
