import { describe, expect, it } from "vitest";
import { runResultForActivePrompt } from "./gemini-browser-provider-panel-state";
import type { GeminiBrowserRun, GeminiBrowserRunResult } from "./types/gemini-browser";

function result(overrides: Partial<GeminiBrowserRunResult> = {}): GeminiBrowserRunResult {
  return {
    run_id: "run-1",
    status: "ok",
    text: "full slow answer",
    message: null,
    manual_action: null,
    artifacts: {
      run_dir: null,
      html: null,
      screenshot: null,
      telemetry: null,
      artifact_write_error: null,
    },
    elapsed_ms: 16_309,
    debug_summary: null,
    ...overrides,
  };
}

function run(overrides: Partial<GeminiBrowserRun> = {}): GeminiBrowserRun {
  return {
    run_id: "run-1",
    source: "settings_test",
    status: "ok",
    prompt_preview: "slow prompt",
    created_at: "2026-06-21T00:00:00Z",
    updated_at: "2026-06-21T00:00:20Z",
    result: result(),
    ...overrides,
  };
}

describe("gemini browser provider panel state", () => {
  it("recovers the completed active test result from refreshed run log", () => {
    expect(
      runResultForActivePrompt(
        [
          run({ run_id: "other-run", result: result({ run_id: "other-run", text: "old answer" }) }),
          run(),
        ],
        "run-1",
      ),
    ).toMatchObject({
      run_id: "run-1",
      status: "ok",
      text: "full slow answer",
    });
  });

  it("does not reuse another run result when the active id is absent", () => {
    expect(runResultForActivePrompt([run()], null)).toBeNull();
    expect(runResultForActivePrompt([run()], "missing-run")).toBeNull();
  });

  it("waits until the active run has a persisted result", () => {
    expect(runResultForActivePrompt([run({ result: null, status: "running" })], "run-1")).toBeNull();
  });
});
