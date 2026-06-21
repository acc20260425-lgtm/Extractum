import type { GeminiBrowserRun, GeminiBrowserRunResult } from "./types/gemini-browser";

export function runResultForActivePrompt(
  runs: GeminiBrowserRun[],
  activeRunId: string | null,
): GeminiBrowserRunResult | null {
  if (!activeRunId) return null;
  return runs.find((run) => run.run_id === activeRunId)?.result ?? null;
}
