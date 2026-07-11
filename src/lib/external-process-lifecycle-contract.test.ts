import { describe, expect, it } from "vitest";
import libSource from "../../src-tauri/src/lib.rs?raw";
import coordinatorSource from "../../src-tauri/src/external_process.rs?raw";
import processTreeSource from "../../src-tauri/src/process_tree.rs?raw";

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

  it("contains Windows child trees using owned child handles", () => {
    const processTree = normalized(processTreeSource);

    expect(processTree).toContain("JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE");
    expect(processTree).toContain("AsRawHandle");
    expect(processTree).toContain("AssignProcessToJobObject");
    expect(processTree).not.toContain("OpenProcess");
    expect(processTree).not.toMatch(/\.pid\s*\(/);
  });
});
