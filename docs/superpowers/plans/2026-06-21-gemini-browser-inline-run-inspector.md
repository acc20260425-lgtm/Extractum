# Gemini Browser Inline Run Inspector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an inline run inspector to Browser Providers so recent Gemini Browser runs expose sanitized diagnostics directly in the settings panel.

**Architecture:** The TypeScript sidecar produces an optional sanitized `debug_summary` on every new run result. Rust mirrors and persists that result through the existing file-backed run log and exposes a narrow command to open recorded run folders. The Svelte panel renders a compact inspector from the existing run-log refresh path and copies sanitized diagnostics without reading artifact files.

**Tech Stack:** Svelte 5, Tauri 2 commands, Rust serde DTOs, TypeScript sidecar, Playwright test doubles, Vitest, Cargo tests.

---

## File Structure

- Modify `sidecars/gemini-browser/src/protocol.ts`: add the sidecar `debug_summary` DTO.
- Modify `sidecars/gemini-browser/src/adapter.ts`: collect per-run debug facts and attach them to all run results.
- Modify `sidecars/gemini-browser/src/adapter.test.ts`: add sidecar diagnostics regression coverage.
- Modify `src-tauri/src/gemini_browser/types.rs`: mirror `debug_summary` in Rust.
- Modify `src-tauri/src/gemini_browser/run_log.rs`: preserve debug summary and expose a safe recorded-run-dir resolver.
- Modify `src-tauri/src/gemini_browser/sidecar.rs`: include `debug_summary: None` in sidecar-unavailable results.
- Modify `src-tauri/src/gemini_browser/commands.rs`: add `gemini_bridge_open_run_folder`.
- Modify `src-tauri/src/gemini_browser/mod.rs`: export the new command/helper.
- Modify `src-tauri/src/lib.rs`: register the new Tauri command.
- Modify `src/lib/types/gemini-browser.ts`: mirror frontend `debug_summary`.
- Modify `src/lib/api/gemini-browser.ts`: add `geminiBridgeOpenRunFolder`.
- Create `src/lib/gemini-browser-run-inspector.ts`: pure inspector selection, artifact availability, and copy-diagnostics formatting.
- Create `src/lib/gemini-browser-run-inspector.test.ts`: fast frontend state tests.
- Modify `src/lib/components/settings/gemini-browser-provider-panel.svelte`: render the inline inspector.
- Modify `src/lib/gemini-browser-provider-panel.test.ts`: source-level UI contract coverage.

---

### Task 1: Sidecar Debug Summary Contract And Adapter Instrumentation

**Files:**
- Modify: `sidecars/gemini-browser/src/protocol.ts`
- Modify: `sidecars/gemini-browser/src/adapter.ts`
- Modify: `sidecars/gemini-browser/src/adapter.test.ts`

- [ ] **Step 1: Add failing sidecar tests for debug summaries**

Append these expectations to existing `adapter.test.ts` cases instead of creating a new mock framework.

In the successful streaming-answer test, extend the resolved match:

```ts
await expect(
  adapter.sendSingle({
    browserProfileDir: "C:/Extractum/gemini-browser/profile",
    artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
    request: {
      run_id: "run-1",
      prompt,
      source: "settings_test",
      artifact_mode: "reduced",
    },
  }),
).resolves.toMatchObject({
  status: "ok",
  text: finalAnswer,
  debug_summary: {
    mode: "managed",
    composer_found: true,
    send_button_found: true,
    answer_found: true,
    answer_selector: "[data-response-index]",
    final_text_length: finalAnswer.length,
    error_stage: null,
  },
});
```

Add a send-failure test near the existing DOM-contract tests:

```ts
it("adds sanitized debug summary to send-button failures", async () => {
  const prompt = "private prompt must not appear in debug summary";
  const composer = {
    count: async () => 1,
    nth: () => composer,
    isVisible: async () => true,
    fill: vi.fn(async () => undefined),
  };
  const empty = {
    count: async () => 0,
    nth: () => empty,
    isVisible: async () => false,
    allTextContents: async () => [],
  };
  const page = {
    isClosed: () => false,
    locator: (selector: string) => {
      if (selector === "rich-textarea textarea") return composer;
      return empty;
    },
    waitForTimeout: async () => undefined,
  };
  const adapter = new GeminiBrowserAdapter({ env: {} });
  adapter.__setTestPage(page as never);

  const result = await adapter.sendSingle({
    browserProfileDir: "C:/Extractum/gemini-browser/profile",
    artifactDir: "artifacts/gemini-browser-adapter-test/run-send-fail",
    request: {
      run_id: "run-send-fail",
      prompt,
      source: "settings_test",
      artifact_mode: "reduced",
    },
  });

  expect(result).toMatchObject({
    status: "needs_manual_action",
    message: "Send button was not found.",
    debug_summary: {
      mode: "managed",
      composer_found: true,
      send_button_found: false,
      answer_found: false,
      answer_selector: null,
      final_text_length: 0,
      error_stage: "send",
    },
  });
  expect(JSON.stringify(result.debug_summary)).not.toContain(prompt);
});
```

In the existing previous-generation wait test, extend the result expectation:

```ts
await expect(
  adapter.sendSingle({
    browserProfileDir: "C:/Extractum/gemini-browser/profile",
    artifactDir: "artifacts/gemini-browser-adapter-test/run-4",
    request: {
      run_id: "run-4",
      prompt: "ответь на прошлый вопрос",
      source: "settings_test",
      artifact_mode: "reduced",
    },
  }),
).resolves.toMatchObject({
  status: "ok",
  text: finalAnswer,
  debug_summary: {
    generation_busy_observed: true,
    send_button_found: true,
  },
});
```

- [ ] **Step 2: Run sidecar tests to verify they fail**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:unit -- sidecars/gemini-browser/src/adapter.test.ts
```

Expected: FAIL because `debug_summary` is not present on `GeminiBrowserRunResult`.

- [ ] **Step 3: Add TypeScript protocol types**

In `sidecars/gemini-browser/src/protocol.ts`, add these types above `GeminiBrowserRunResult`:

```ts
export type GeminiBrowserDebugErrorStage =
  | "setup"
  | "composer"
  | "send"
  | "answer"
  | "artifacts"
  | "transport";

export interface GeminiBrowserRunDebugSummary {
  mode: GeminiBrowserProviderMode;
  composer_found: boolean;
  send_button_found: boolean;
  generation_busy_observed: boolean;
  answer_found: boolean;
  answer_selector: string | null;
  waited_for_send_ms: number;
  waited_for_answer_ms: number;
  answer_stable_ms: number;
  final_text_length: number;
  error_stage: GeminiBrowserDebugErrorStage | null;
}
```

Then add the field to `GeminiBrowserRunResult`:

```ts
debug_summary?: GeminiBrowserRunDebugSummary | null;
```

The field is optional in the TypeScript DTO because older run JSON and tests may
not have it. New sidecar results must still populate it.

- [ ] **Step 4: Add adapter diagnostics helpers**

In `sidecars/gemini-browser/src/adapter.ts`, import the new types:

```ts
  GeminiBrowserRunDebugSummary,
  GeminiBrowserDebugErrorStage,
```

Add these helper types/functions near `emptyArtifacts()`:

```ts
interface WaitForFirstVisibleResult {
  locator: Locator | null;
  selector: string | null;
  waitedMs: number;
  keptWaitingObserved: boolean;
}

function emptyDebugSummary(mode: GeminiBrowserProviderConfig["mode"]): GeminiBrowserRunDebugSummary {
  return {
    mode,
    composer_found: false,
    send_button_found: false,
    generation_busy_observed: false,
    answer_found: false,
    answer_selector: null,
    waited_for_send_ms: 0,
    waited_for_answer_ms: 0,
    answer_stable_ms: ANSWER_STABLE_MS,
    final_text_length: 0,
    error_stage: null,
  };
}

type RunResultWithoutDebug = Omit<GeminiBrowserRunResult, "debug_summary">;

function finalizeRunResult(
  result: RunResultWithoutDebug,
  debugSummary: GeminiBrowserRunDebugSummary,
): GeminiBrowserRunResult {
  return {
    ...result,
    debug_summary: debugSummary,
  };
}

function markErrorStage(
  debugSummary: GeminiBrowserRunDebugSummary,
  errorStage: GeminiBrowserDebugErrorStage,
): GeminiBrowserRunDebugSummary {
  return {
    ...debugSummary,
    error_stage: errorStage,
  };
}
```

Add a diagnostic variant of the visible wait:

```ts
async function waitForFirstVisibleWithDiagnostics(
  page: Pick<Page, "locator" | "waitForTimeout">,
  selectors: string[],
  options: {
    timeoutMs?: number;
    intervalMs?: number;
    keepWaitingWhileVisible?: string[];
    idleGraceMs?: number;
  } = {},
): Promise<WaitForFirstVisibleResult> {
  const timeoutMs = options.timeoutMs ?? 20_000;
  const intervalMs = options.intervalMs ?? 250;
  const idleGraceMs = options.idleGraceMs ?? timeoutMs;
  const maxAttempts = Math.max(1, Math.ceil(timeoutMs / Math.max(intervalMs, 1)) + 1);
  let idleElapsedMs = 0;
  let waitedMs = 0;
  let keptWaitingObserved = false;

  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    for (const selector of selectors) {
      const locator = page.locator(selector);
      const count = await locator.count();
      for (let index = count - 1; index >= 0; index -= 1) {
        const candidate = locator.nth(index);
        if (await candidate.isVisible().catch(() => false)) {
          return { locator: candidate, selector, waitedMs, keptWaitingObserved };
        }
      }
    }
    const shouldKeepWaiting =
      options.keepWaitingWhileVisible &&
      (await hasVisibleLocator(page, options.keepWaitingWhileVisible));
    if (shouldKeepWaiting) {
      keptWaitingObserved = true;
    } else {
      idleElapsedMs += intervalMs;
      if (idleElapsedMs >= idleGraceMs) {
        return { locator: null, selector: null, waitedMs, keptWaitingObserved };
      }
    }
    if (attempt < maxAttempts - 1) {
      await page.waitForTimeout(intervalMs);
      waitedMs += intervalMs;
    }
  }
  return { locator: null, selector: null, waitedMs, keptWaitingObserved };
}
```

Change `waitForFirstVisible()` to delegate to this helper:

```ts
export async function waitForFirstVisible(
  page: Pick<Page, "locator" | "waitForTimeout">,
  selectors: string[],
  options: {
    timeoutMs?: number;
    intervalMs?: number;
    keepWaitingWhileVisible?: string[];
    idleGraceMs?: number;
  } = {},
): Promise<Locator | null> {
  return (await waitForFirstVisibleWithDiagnostics(page, selectors, options)).locator;
}
```

- [ ] **Step 5: Return answer diagnostics**

Replace the answer state interfaces/functions in `adapter.ts` with selector-aware variants:

```ts
interface AnswerEntry {
  selector: string;
  text: string;
}

interface AnswerState {
  entries: AnswerEntry[];
}

interface AnswerResult {
  text: string;
  selector: string;
  waitedMs: number;
}
```

Update `waitForAnswerText()` to return `AnswerResult | null`:

```ts
async function waitForAnswerText(
  page: Page,
  prompt: string,
  baseline: AnswerState,
): Promise<AnswerResult | null> {
  const deadline = Date.now() + ANSWER_TIMEOUT_MS;
  let latestAnswer: AnswerEntry | null = null;
  let lastChangedAt = Date.now();
  let firstSeenAt: number | null = null;
  let waitedMs = 0;

  while (Date.now() < deadline) {
    const state = await captureAnswerState(page, prompt);
    const answer = bestNewAnswerText(state, baseline);
    const now = Date.now();
    if (answer && answer.text !== latestAnswer?.text) {
      latestAnswer = answer;
      lastChangedAt = now;
      firstSeenAt ??= now;
    }
    if (latestAnswer) {
      const stableForMs = now - lastChangedAt;
      if (
        firstSeenAt !== null &&
        stableForMs >= ANSWER_STABLE_MS &&
        now - firstSeenAt >= ANSWER_STABLE_MS
      ) {
        return { ...latestAnswer, waitedMs };
      }
    }
    await page.waitForTimeout(ANSWER_POLL_INTERVAL_MS);
    waitedMs += ANSWER_POLL_INTERVAL_MS;
  }

  return latestAnswer ? { ...latestAnswer, waitedMs } : null;
}
```

Update `captureAnswerState()`:

```ts
async function captureAnswerState(page: Page, prompt: string): Promise<AnswerState> {
  const entries: AnswerEntry[] = [];
  const seen = new Set<string>();
  for (const selector of answerCandidates.map((candidate) => candidate.selector)) {
    const rawTexts = await page.locator(selector).allTextContents().catch(() => []);
    for (const rawText of rawTexts) {
      const text = rawText.trim();
      if (text.length === 0 || text === prompt || seen.has(text)) continue;
      seen.add(text);
      entries.push({ selector, text });
    }
  }

  return { entries };
}
```

Update `bestNewAnswerText()`:

```ts
function bestNewAnswerText(current: AnswerState, baseline: AnswerState): AnswerEntry | null {
  const baselineTexts = new Set(baseline.entries.map((entry) => entry.text));
  const newEntries = current.entries.filter((entry) => !baselineTexts.has(entry.text));
  if (newEntries.length === 0) return null;

  return newEntries.reduce((best, entry) => (entry.text.length >= best.text.length ? entry : best));
}
```

- [ ] **Step 6: Attach debug summary to every adapter result**

At the start of `sendSingle()` after `mode` is resolved:

```ts
const debugSummary = emptyDebugSummary(mode.type);
```

All `sendSingle()` return paths must go through `finalizeRunResult(...)`. Do not
return a raw object from `sendSingle()` after this step. This covers:

- already closed CDP page before send;
- managed page was not created;
- CDP attach setup error returned from `attachCdpBrowser`;
- CDP connected but no Gemini tab exists;
- composer not found;
- send button not found;
- answer timeout;
- successful answer;
- CDP closed-target catch branch;
- generic catch branch.

For early results before a page exists, wrap results with `finalizeRunResult(...)`. Example for closed CDP page:

```ts
return finalizeRunResult(
  {
    run_id: input.request.run_id,
    status: "browser_crashed",
    text: null,
    message: "Chrome CDP page closed before the run could send.",
    manual_action: null,
    artifacts: emptyArtifacts(input.artifactDir),
    elapsed_ms: Date.now() - start,
  },
  markErrorStage(debugSummary, "transport"),
);
```

For the managed-page-not-created branch:

```ts
return finalizeRunResult(
  {
    run_id: input.request.run_id,
    status: "failed",
    text: null,
    message: "Gemini browser page was not created.",
    manual_action: null,
    artifacts: emptyArtifacts(input.artifactDir),
    elapsed_ms: Date.now() - start,
  },
  markErrorStage(debugSummary, "setup"),
);
```

For the CDP attach setup error branch that uses `attachStatus`:

```ts
return finalizeRunResult(
  {
    run_id: input.request.run_id,
    status: "needs_manual_action",
    text: null,
    message: attachStatus.latest_message,
    manual_action: attachStatus.manual_action,
    artifacts: emptyArtifacts(input.artifactDir),
    elapsed_ms: Date.now() - start,
  },
  markErrorStage(debugSummary, "setup"),
);
```

For the attached-CDP-without-Gemini-tab branch:

```ts
return finalizeRunResult(
  {
    run_id: input.request.run_id,
    status: "needs_manual_action",
    text: null,
    message: "Open Gemini in the attached Chrome profile or use Open to create a Gemini tab.",
    manual_action: "start_chrome_cdp",
    artifacts: emptyArtifacts(input.artifactDir),
    elapsed_ms: Date.now() - start,
  },
  markErrorStage(debugSummary, "setup"),
);
```

For composer lookup:

```ts
const composerLookup = await waitForFirstVisibleWithDiagnostics(
  page,
  composerCandidates.map((candidate) => candidate.selector),
  { timeoutMs: 30_000, intervalMs: 500 },
);
const composer = composerLookup.locator;
debugSummary.composer_found = Boolean(composer);
if (!composer) {
  return this.failure(
    page,
    input.request,
    input.artifactDir,
    "needs_login",
    "Composer was not found.",
    start,
    markErrorStage(debugSummary, "composer"),
  );
}
```

For send lookup:

```ts
const sendLookup = await waitForFirstVisibleWithDiagnostics(
  page,
  sendCandidates.map((candidate) => candidate.selector),
  {
    timeoutMs: 75_000,
    intervalMs: 250,
    keepWaitingWhileVisible: generationBusySelectors,
    idleGraceMs: 10_000,
  },
);
const send = sendLookup.locator;
debugSummary.send_button_found = Boolean(send);
debugSummary.generation_busy_observed = sendLookup.keptWaitingObserved;
debugSummary.waited_for_send_ms = sendLookup.waitedMs;
if (!send) {
  return this.failure(
    page,
    input.request,
    input.artifactDir,
    "needs_manual_action",
    "Send button was not found.",
    start,
    markErrorStage(debugSummary, "send"),
  );
}
```

For successful answer:

```ts
const answer = await waitForAnswerText(page, input.request.prompt, answerBaseline);
if (!answer) {
  return this.failure(
    page,
    input.request,
    input.artifactDir,
    "timeout",
    "Answer did not appear before timeout.",
    start,
    markErrorStage(debugSummary, "answer"),
  );
}
debugSummary.answer_found = true;
debugSummary.answer_selector = answer.selector;
debugSummary.waited_for_answer_ms = answer.waitedMs;
debugSummary.final_text_length = answer.text.length;

return finalizeRunResult(
  {
    run_id: input.request.run_id,
    status: "ok",
    text: answer.text,
    message: null,
    manual_action: null,
    artifacts: {
      run_dir: input.artifactDir,
      html: null,
      screenshot: null,
      telemetry: null,
      artifact_write_error: null,
    },
    elapsed_ms: Date.now() - start,
  },
  debugSummary,
);
```

Change `failure()` signature:

```ts
private async failure(
  page: Page,
  request: GeminiBrowserRunRequest,
  artifactDir: string,
  status: GeminiBrowserRunResult["status"],
  message: string,
  start: number,
  debugSummary: GeminiBrowserRunDebugSummary,
): Promise<GeminiBrowserRunResult> {
  return finalizeRunResult(
    {
      run_id: request.run_id,
      status,
      text: null,
      message,
      manual_action: status === "needs_login" ? "login" : null,
      artifacts: await captureFailureArtifacts({ page, artifactDir, request, status, message }),
      elapsed_ms: Date.now() - start,
    },
    debugSummary,
  );
}
```

Update every `this.failure(...)` call to pass a stage-marked `debugSummary`.
Update catch branches too:

```ts
if (this.session?.type === "cdp_attach" && isClosedTargetError(error)) {
  return this.failure(
    page,
    input.request,
    input.artifactDir,
    "browser_crashed",
    "Chrome CDP connection closed during the run.",
    start,
    markErrorStage(debugSummary, "transport"),
  );
}
return this.failure(
  page,
  input.request,
  input.artifactDir,
  "failed",
  String(error),
  start,
  markErrorStage(debugSummary, "transport"),
);
```

After implementation, search `adapter.ts` for `return {` inside `sendSingle()`.
There should be no raw `GeminiBrowserRunResult` returns left in that method.

- [ ] **Step 7: Run sidecar checks**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:typecheck
npm.cmd run test:gemini-browser-sidecar:unit -- sidecars/gemini-browser/src/adapter.test.ts
npm.cmd run test:gemini-browser-sidecar:build
```

Expected: PASS.

- [ ] **Step 8: Commit Task 1**

Run:

```powershell
git add sidecars/gemini-browser/src/protocol.ts sidecars/gemini-browser/src/adapter.ts sidecars/gemini-browser/src/adapter.test.ts
git commit -m "feat: add Gemini browser run debug summaries"
```

---

### Task 2: Rust DTOs, Run Log Preservation, And Open Run Folder Command

**Files:**
- Modify: `src-tauri/src/gemini_browser/types.rs`
- Modify: `src-tauri/src/gemini_browser/run_log.rs`
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/api/gemini-browser.ts`

- [ ] **Step 1: Add failing Rust DTO and run-log tests**

In `src-tauri/src/gemini_browser/types.rs`, add a test:

```rust
#[test]
fn run_result_serializes_optional_debug_summary() {
    let result = GeminiBrowserRunResult {
        run_id: "run-1".to_string(),
        status: GeminiBrowserRunStatus::Ok,
        text: Some("answer".to_string()),
        message: None,
        manual_action: None,
        artifacts: GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 42,
        debug_summary: Some(GeminiBrowserRunDebugSummary {
            mode: GeminiBrowserProviderMode::CdpAttach,
            composer_found: true,
            send_button_found: true,
            generation_busy_observed: true,
            answer_found: true,
            answer_selector: Some("message-content".to_string()),
            waited_for_send_ms: 15_000,
            waited_for_answer_ms: 12_000,
            answer_stable_ms: 8_000,
            final_text_length: 6,
            error_stage: None,
        }),
    };

    let json = serde_json::to_value(&result).expect("serialize result");
    assert_eq!(json["debug_summary"]["mode"], "cdp_attach");
    assert_eq!(json["debug_summary"]["generation_busy_observed"], true);

    let decoded: GeminiBrowserRunResult =
        serde_json::from_value(json).expect("deserialize result");
    assert_eq!(
        decoded.debug_summary.expect("debug summary").answer_selector,
        Some("message-content".to_string())
    );
}
```

In `src-tauri/src/gemini_browser/run_log.rs`, extend `run_log_persists_queued_running_and_terminal_result()` by setting `debug_summary` on `result` and asserting it survives `list_runs()`:

```rust
debug_summary: Some(crate::gemini_browser::GeminiBrowserRunDebugSummary {
    mode: crate::gemini_browser::GeminiBrowserProviderMode::Managed,
    composer_found: true,
    send_button_found: true,
    generation_busy_observed: false,
    answer_found: true,
    answer_selector: Some("message-content".to_string()),
    waited_for_send_ms: 0,
    waited_for_answer_ms: 8_000,
    answer_stable_ms: 8_000,
    final_text_length: 6,
    error_stage: None,
}),
```

Then assert:

```rust
assert_eq!(
    listed.runs[0]
        .result
        .as_ref()
        .and_then(|result| result.debug_summary.as_ref())
        .and_then(|summary| summary.answer_selector.as_deref()),
    Some("message-content")
);
```

- [ ] **Step 2: Run Rust tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser::types::tests::run_result_serializes_optional_debug_summary
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser::run_log::tests::run_log_persists_queued_running_and_terminal_result
```

Expected: FAIL because `GeminiBrowserRunDebugSummary` and `debug_summary` do not exist.

- [ ] **Step 3: Add Rust debug summary types**

In `src-tauri/src/gemini_browser/types.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserDebugErrorStage {
    Setup,
    Composer,
    Send,
    Answer,
    Artifacts,
    Transport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunDebugSummary {
    pub mode: GeminiBrowserProviderMode,
    pub composer_found: bool,
    pub send_button_found: bool,
    pub generation_busy_observed: bool,
    pub answer_found: bool,
    pub answer_selector: Option<String>,
    pub waited_for_send_ms: u64,
    pub waited_for_answer_ms: u64,
    pub answer_stable_ms: u64,
    pub final_text_length: usize,
    pub error_stage: Option<GeminiBrowserDebugErrorStage>,
}
```

Add the field to `GeminiBrowserRunResult`:

```rust
#[serde(default)]
pub debug_summary: Option<GeminiBrowserRunDebugSummary>,
```

Update all Rust `GeminiBrowserRunResult` literals to include:

```rust
debug_summary: None,
```

This includes `sidecar_unavailable_result()` and existing tests that construct a result without diagnostics.

- [ ] **Step 4: Export Rust debug summary types**

In `src-tauri/src/gemini_browser/mod.rs`, add the new types to the `pub use types::{ ... }` list:

```rust
GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
```

- [ ] **Step 5: Add recorded run directory resolver**

In `src-tauri/src/gemini_browser/run_log.rs`, add:

```rust
pub(crate) fn recorded_run_dir(runs_dir: &Path, run_id: &str) -> AppResult<PathBuf> {
    let dir = runs_dir.join(safe_run_id(run_id)?);
    let result_path = dir.join(RUN_FILE);
    if !result_path.exists() {
        return Err(AppError::validation("Gemini browser run was not found"));
    }
    let run = read_run_file(&result_path)?;
    let recorded = run
        .result
        .as_ref()
        .and_then(|result| result.artifacts.run_dir.as_deref())
        .ok_or_else(|| AppError::validation("Gemini browser run folder is not available"))?;
    let expected = dir
        .canonicalize()
        .map_err(|error| AppError::internal(error.to_string()))?;
    let recorded = PathBuf::from(recorded)
        .canonicalize()
        .map_err(|_| AppError::validation("Gemini browser run folder is not available"))?;
    if recorded != expected {
        return Err(AppError::validation(
            "Gemini browser run folder does not match the recorded run",
        ));
    }
    Ok(expected)
}
```

Add a test:

```rust
#[test]
fn recorded_run_dir_accepts_only_result_artifact_run_dir() {
    let temp = tempdir().expect("tempdir");
    let runs_dir = temp.path();
    create_queued_run(runs_dir, "run-1", "settings_test", "hello Gemini")
        .expect("create queued run");
    assert!(super::recorded_run_dir(runs_dir, "run-1").is_err());

    let run_dir = runs_dir.join("run-1");
    let result = GeminiBrowserRunResult {
        run_id: "run-1".to_string(),
        status: GeminiBrowserRunStatus::Ok,
        text: Some("answer".to_string()),
        message: None,
        manual_action: None,
        artifacts: GeminiBrowserArtifactRefs {
            run_dir: Some(run_dir.to_string_lossy().to_string()),
            ..Default::default()
        },
        elapsed_ms: 25,
        debug_summary: None,
    };
    finish_run(runs_dir, "run-1", result).expect("finish run");

    let dir = super::recorded_run_dir(runs_dir, "run-1").expect("recorded run dir");
    assert_eq!(dir.file_name().and_then(|name| name.to_str()), Some("run-1"));

    create_queued_run(runs_dir, "run-2", "settings_test", "hello Gemini")
        .expect("create queued run");
    let outside = temp.path().join("outside-run-dir");
    std::fs::create_dir_all(&outside).expect("outside dir");
    let mismatched = GeminiBrowserRunResult {
        run_id: "run-2".to_string(),
        status: GeminiBrowserRunStatus::Ok,
        text: Some("answer".to_string()),
        message: None,
        manual_action: None,
        artifacts: GeminiBrowserArtifactRefs {
            run_dir: Some(outside.to_string_lossy().to_string()),
            ..Default::default()
        },
        elapsed_ms: 25,
        debug_summary: None,
    };
    finish_run(runs_dir, "run-2", mismatched).expect("finish run");
    assert!(super::recorded_run_dir(runs_dir, "run-2").is_err());
    assert!(super::recorded_run_dir(runs_dir, "../bad").is_err());
    assert!(super::recorded_run_dir(runs_dir, "missing-run").is_err());
}
```

- [ ] **Step 6: Add Tauri command to open a run folder**

In `src-tauri/src/gemini_browser/commands.rs`, import opener:

```rust
use tauri_plugin_opener::OpenerExt;
```

Import `recorded_run_dir` from `super` by updating the `use super::{ ... }` list:

```rust
recorded_run_dir,
```

Add the command:

```rust
#[tauri::command]
pub async fn gemini_bridge_open_run_folder(
    handle: AppHandle,
    run_id: String,
) -> AppResult<()> {
    let dir = recorded_run_dir(&runs_dir(&handle)?, &run_id)?;
    handle
        .opener()
        .open_path(path_string(&dir), None::<&str>)
        .map_err(|error| {
            AppError::internal(format!("Failed to open Gemini browser run folder: {error}"))
        })?;
    Ok(())
}
```

In `src-tauri/src/gemini_browser/mod.rs`, export the command and helper:

```rust
gemini_bridge_open_run_folder,
```

```rust
pub(crate) use run_log::{create_queued_run, finish_run, list_runs, mark_running, recorded_run_dir};
```

In `src-tauri/src/lib.rs`, import and register the command:

```rust
gemini_bridge_open_run_folder,
```

and in `tauri::generate_handler![ ... ]` add:

```rust
gemini_bridge_open_run_folder,
```

- [ ] **Step 7: Add frontend API wrapper**

In `src/lib/api/gemini-browser.ts`, add:

```ts
export function geminiBridgeOpenRunFolder(runId: string) {
  return invoke<void>("gemini_bridge_open_run_folder", { runId });
}
```

- [ ] **Step 8: Run Rust/API checks**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
npm.cmd run test -- --run src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-provider-panel.test.ts
```

Expected: PASS.

- [ ] **Step 9: Commit Task 2**

Run:

```powershell
git add src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/run_log.rs src-tauri/src/gemini_browser/sidecar.rs src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/mod.rs src-tauri/src/lib.rs src/lib/api/gemini-browser.ts
git commit -m "feat: expose Gemini browser run diagnostics data"
```

---

### Task 3: Frontend Inspector View Model And Sanitized Copy Diagnostics

**Files:**
- Modify: `src/lib/types/gemini-browser.ts`
- Create: `src/lib/gemini-browser-run-inspector.ts`
- Create: `src/lib/gemini-browser-run-inspector.test.ts`

- [ ] **Step 1: Add failing frontend view-model tests**

Create `src/lib/gemini-browser-run-inspector.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  artifactAvailability,
  copyableRunDiagnostics,
  selectedRunForInspector,
} from "./gemini-browser-run-inspector";
import type { GeminiBrowserRun, GeminiBrowserRunResult } from "./types/gemini-browser";

function result(overrides: Partial<GeminiBrowserRunResult> = {}): GeminiBrowserRunResult {
  return {
    run_id: "run-1",
    status: "ok",
    text: "answer text",
    message: null,
    manual_action: null,
    artifacts: {
      run_dir: "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1",
      html: null,
      screenshot: null,
      telemetry: "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1/telemetry.json",
      artifact_write_error: null,
    },
    elapsed_ms: 16_309,
    debug_summary: {
      mode: "cdp_attach",
      composer_found: true,
      send_button_found: true,
      generation_busy_observed: true,
      answer_found: true,
      answer_selector: "message-content",
      waited_for_send_ms: 15_000,
      waited_for_answer_ms: 10_000,
      answer_stable_ms: 8_000,
      final_text_length: 11,
      error_stage: null,
    },
    ...overrides,
  };
}

function run(overrides: Partial<GeminiBrowserRun> = {}): GeminiBrowserRun {
  return {
    run_id: "run-1",
    source: "settings_test",
    status: "ok",
    prompt_preview: "prompt preview",
    created_at: "2026-06-21T00:00:00Z",
    updated_at: "2026-06-21T00:00:20Z",
    result: result(),
    ...overrides,
  };
}

describe("gemini browser run inspector", () => {
  it("selects the active run before falling back to the newest run", () => {
    const newest = run({ run_id: "newest", result: result({ run_id: "newest" }) });
    const active = run({ run_id: "active", result: result({ run_id: "active" }) });

    expect(selectedRunForInspector([newest, active], "active")?.run_id).toBe("active");
    expect(selectedRunForInspector([newest, active], null)?.run_id).toBe("newest");
    expect(selectedRunForInspector([], null)).toBeNull();
  });

  it("reports artifact availability without exposing full paths", () => {
    expect(artifactAvailability(result())).toEqual({
      run_dir: true,
      html: false,
      screenshot: false,
      telemetry: true,
      artifact_write_error: false,
    });
  });

  it("copies sanitized diagnostics with debug facts and no local paths", () => {
    const diagnostics = copyableRunDiagnostics(run());

    expect(diagnostics).toContain("run_id: run-1");
    expect(diagnostics).toContain("status: ok");
    expect(diagnostics).toContain("result_status: ok");
    expect(diagnostics).toContain("elapsed_ms: 16309");
    expect(diagnostics).toContain("answer_length: 11");
    expect(diagnostics).toContain("generation_busy_observed: true");
    expect(diagnostics).toContain("answer_selector: message-content");
    expect(diagnostics).not.toContain("C:/Users/Dima");
    expect(diagnostics).not.toContain("answer text");
  });

  it("copies a clear marker when debug summary is unavailable", () => {
    const diagnostics = copyableRunDiagnostics(
      run({ result: result({ debug_summary: null, text: null }) }),
    );

    expect(diagnostics).toContain("debug_summary: unavailable");
  });
});
```

- [ ] **Step 2: Run frontend tests to verify they fail**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts
```

Expected: FAIL because `gemini-browser-run-inspector.ts` and frontend `debug_summary` types do not exist.

- [ ] **Step 3: Add frontend debug summary types**

In `src/lib/types/gemini-browser.ts`, add:

```ts
export type GeminiBrowserDebugErrorStage =
  | "setup"
  | "composer"
  | "send"
  | "answer"
  | "artifacts"
  | "transport";

export interface GeminiBrowserRunDebugSummary {
  mode: GeminiBrowserProviderMode;
  composer_found: boolean;
  send_button_found: boolean;
  generation_busy_observed: boolean;
  answer_found: boolean;
  answer_selector: string | null;
  waited_for_send_ms: number;
  waited_for_answer_ms: number;
  answer_stable_ms: number;
  final_text_length: number;
  error_stage: GeminiBrowserDebugErrorStage | null;
}
```

Add to `GeminiBrowserRunResult`:

```ts
debug_summary?: GeminiBrowserRunDebugSummary | null;
```

Update local test factories in `src/lib/gemini-browser-provider-panel-state.test.ts` so `result()` includes:

```ts
debug_summary: null,
```

- [ ] **Step 4: Implement inspector view model**

Create `src/lib/gemini-browser-run-inspector.ts`:

```ts
import type { GeminiBrowserRun, GeminiBrowserRunResult } from "./types/gemini-browser";

export function selectedRunForInspector(
  runs: GeminiBrowserRun[],
  activeRunId: string | null,
): GeminiBrowserRun | null {
  if (activeRunId) {
    const active = runs.find((run) => run.run_id === activeRunId);
    if (active) return active;
  }
  return runs[0] ?? null;
}

export function artifactAvailability(result: GeminiBrowserRunResult | null) {
  return {
    run_dir: Boolean(result?.artifacts.run_dir),
    html: Boolean(result?.artifacts.html),
    screenshot: Boolean(result?.artifacts.screenshot),
    telemetry: Boolean(result?.artifacts.telemetry),
    artifact_write_error: Boolean(result?.artifacts.artifact_write_error),
  };
}

export function answerLength(result: GeminiBrowserRunResult | null): number {
  return result?.debug_summary?.final_text_length ?? result?.text?.length ?? 0;
}

export function copyableRunDiagnostics(run: GeminiBrowserRun): string {
  const result = run.result;
  const availability = artifactAvailability(result);
  const lines = [
    "Gemini Browser Run Diagnostics",
    `run_id: ${run.run_id}`,
    `source: ${run.source}`,
    `status: ${run.status}`,
    `result_status: ${result?.status ?? "unavailable"}`,
    `created_at: ${run.created_at}`,
    `updated_at: ${run.updated_at}`,
    `elapsed_ms: ${result?.elapsed_ms ?? "unavailable"}`,
    `answer_length: ${answerLength(result)}`,
    `message: ${result?.message ?? "none"}`,
    `manual_action: ${result?.manual_action ?? "none"}`,
    `artifact_run_dir_available: ${availability.run_dir}`,
    `artifact_html_available: ${availability.html}`,
    `artifact_screenshot_available: ${availability.screenshot}`,
    `artifact_telemetry_available: ${availability.telemetry}`,
    `artifact_write_error: ${result?.artifacts.artifact_write_error ? "present" : "none"}`,
  ];

  if (!result?.debug_summary) {
    lines.push("debug_summary: unavailable");
    return lines.join("\n");
  }

  const debug = result.debug_summary;
  lines.push(
    `debug_mode: ${debug.mode}`,
    `composer_found: ${debug.composer_found}`,
    `send_button_found: ${debug.send_button_found}`,
    `generation_busy_observed: ${debug.generation_busy_observed}`,
    `answer_found: ${debug.answer_found}`,
    `answer_selector: ${debug.answer_selector ?? "none"}`,
    `waited_for_send_ms: ${debug.waited_for_send_ms}`,
    `waited_for_answer_ms: ${debug.waited_for_answer_ms}`,
    `answer_stable_ms: ${debug.answer_stable_ms}`,
    `final_text_length: ${debug.final_text_length}`,
    `error_stage: ${debug.error_stage ?? "none"}`,
  );

  return lines.join("\n");
}
```

- [ ] **Step 5: Run frontend view-model tests**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel-state.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

Run:

```powershell
git add src/lib/types/gemini-browser.ts src/lib/gemini-browser-run-inspector.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel-state.test.ts
git commit -m "feat: add Gemini browser run inspector model"
```

---

### Task 4: Inline Inspector UI In Browser Providers Panel

**Files:**
- Modify: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`

- [ ] **Step 1: Add failing panel source-contract tests**

In `src/lib/gemini-browser-provider-panel.test.ts`, add:

```ts
it("renders inline run inspector controls and sanitized diagnostics actions", () => {
  expect(componentSource).toContain("Run inspector");
  expect(componentSource).toContain("selectedRunForInspector");
  expect(componentSource).toContain("copyableRunDiagnostics");
  expect(componentSource).toContain("Copy diagnostics");
  expect(componentSource).toContain("Open run folder");
  expect(componentSource).toContain("geminiBridgeOpenRunFolder");
});

it("shows debug summary fields without reading artifact files in the panel", () => {
  expect(componentSource).toContain("generation_busy_observed");
  expect(componentSource).toContain("answer_selector");
  expect(componentSource).toContain("waited_for_send_ms");
  expect(componentSource).toContain("waited_for_answer_ms");
  expect(componentSource).not.toContain("page.html");
  expect(componentSource).not.toContain("page.png");
});
```

- [ ] **Step 2: Run panel tests to verify they fail**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-provider-panel.test.ts
```

Expected: FAIL because the panel does not yet render the inspector.

- [ ] **Step 3: Add imports and derived inspector state**

In `src/lib/components/settings/gemini-browser-provider-panel.svelte`, update icon imports:

```ts
import { Clipboard, ExternalLink, FolderOpen, Play, RefreshCw, Send, Square } from "@lucide/svelte";
```

Import the new API and view model:

```ts
  geminiBridgeOpenRunFolder,
```

```ts
import {
  answerLength,
  artifactAvailability,
  copyableRunDiagnostics,
  selectedRunForInspector,
} from "$lib/gemini-browser-run-inspector";
```

Add state:

```ts
let inspectorMessage = $state("");
const selectedInspectorRun = $derived(selectedRunForInspector(runs, activeTestRunId));
const selectedInspectorResult = $derived(selectedInspectorRun?.result ?? null);
const selectedArtifactAvailability = $derived(artifactAvailability(selectedInspectorResult));
```

- [ ] **Step 4: Add inspector actions**

Add these functions in the `<script>` block:

```ts
async function copyDiagnostics() {
  if (!selectedInspectorRun) {
    inspectorMessage = "No browser run is selected.";
    return;
  }
  try {
    await navigator.clipboard.writeText(copyableRunDiagnostics(selectedInspectorRun));
    inspectorMessage = "Diagnostics copied.";
  } catch (error) {
    inspectorMessage = formatAppError("copying Gemini browser diagnostics", error);
  }
}

async function openSelectedRunFolder() {
  if (!selectedInspectorRun?.result?.artifacts.run_dir) {
    inspectorMessage = "Run folder is not available.";
    return;
  }
  try {
    await geminiBridgeOpenRunFolder(selectedInspectorRun.run_id);
    inspectorMessage = "Run folder opened.";
  } catch (error) {
    inspectorMessage = formatAppError("opening Gemini browser run folder", error);
  }
}
```

- [ ] **Step 5: Render inline inspector markup**

Add this block between the test prompt card and recent runs list:

```svelte
<section class="run-inspector" aria-label="Run inspector">
  <div class="row inspector-head">
    <div>
      <h3>Run inspector</h3>
      <p>Latest Browser Provider run diagnostics.</p>
    </div>
    <div class="actions">
      <button type="button" onclick={refresh} disabled={busy} title="Refresh run diagnostics">
        <RefreshCw size={14} />
        <span>Refresh</span>
      </button>
      <button type="button" onclick={copyDiagnostics} disabled={!selectedInspectorRun}>
        <Clipboard size={14} />
        <span>Copy diagnostics</span>
      </button>
      <button
        type="button"
        onclick={openSelectedRunFolder}
        disabled={!selectedInspectorResult?.artifacts.run_dir}
      >
        <FolderOpen size={14} />
        <span>Open run folder</span>
      </button>
    </div>
  </div>

  {#if selectedInspectorRun}
    <div class="inspector-grid">
      <div>
        <span class="fact-label">Run</span>
        <code>{selectedInspectorRun.run_id}</code>
      </div>
      <div>
        <span class="fact-label">Status</span>
        <strong>{selectedInspectorRun.status}</strong>
      </div>
      <div>
        <span class="fact-label">Result</span>
        <strong>{selectedInspectorResult?.status ?? "pending"}</strong>
      </div>
      <div>
        <span class="fact-label">Elapsed</span>
        <span>{selectedInspectorResult?.elapsed_ms ?? 0} ms</span>
      </div>
      <div>
        <span class="fact-label">Answer length</span>
        <span>{answerLength(selectedInspectorResult)}</span>
      </div>
      <div>
        <span class="fact-label">Manual action</span>
        <span>{selectedInspectorResult?.manual_action ?? "none"}</span>
      </div>
    </div>

    {#if selectedInspectorResult?.message}
      <p class="message">{selectedInspectorResult.message}</p>
    {/if}

    <div class="inspector-grid compact">
      <div>
        <span class="fact-label">Run folder</span>
        <span>{selectedArtifactAvailability.run_dir ? "available" : "missing"}</span>
      </div>
      <div>
        <span class="fact-label">Telemetry</span>
        <span>{selectedArtifactAvailability.telemetry ? "available" : "missing"}</span>
      </div>
      <div>
        <span class="fact-label">HTML</span>
        <span>{selectedArtifactAvailability.html ? "available" : "not captured"}</span>
      </div>
      <div>
        <span class="fact-label">Screenshot</span>
        <span>{selectedArtifactAvailability.screenshot ? "available" : "not captured"}</span>
      </div>
    </div>

    {#if selectedInspectorResult?.debug_summary}
      <div class="inspector-grid compact">
        <div>
          <span class="fact-label">Mode</span>
          <span>{selectedInspectorResult.debug_summary.mode}</span>
        </div>
        <div>
          <span class="fact-label">Composer</span>
          <span>{selectedInspectorResult.debug_summary.composer_found ? "found" : "missing"}</span>
        </div>
        <div>
          <span class="fact-label">Send</span>
          <span>{selectedInspectorResult.debug_summary.send_button_found ? "found" : "missing"}</span>
        </div>
        <div>
          <span class="fact-label">Busy observed</span>
          <span>{selectedInspectorResult.debug_summary.generation_busy_observed ? "yes" : "no"}</span>
        </div>
        <div>
          <span class="fact-label">Answer selector</span>
          <code>{selectedInspectorResult.debug_summary.answer_selector ?? "none"}</code>
        </div>
        <div>
          <span class="fact-label">Send wait</span>
          <span>{selectedInspectorResult.debug_summary.waited_for_send_ms} ms</span>
        </div>
        <div>
          <span class="fact-label">Answer wait</span>
          <span>{selectedInspectorResult.debug_summary.waited_for_answer_ms} ms</span>
        </div>
        <div>
          <span class="fact-label">Error stage</span>
          <span>{selectedInspectorResult.debug_summary.error_stage ?? "none"}</span>
        </div>
      </div>
    {:else}
      <p class="empty">Debug summary unavailable for this run.</p>
    {/if}
  {:else}
    <p class="empty">No browser run selected.</p>
  {/if}

  {#if inspectorMessage}
    <p class="message">{inspectorMessage}</p>
  {/if}
</section>
```

- [ ] **Step 6: Add compact inspector styles**

Add CSS to the component:

```css
.run-inspector {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 14px;
  background: var(--card);
}

.inspector-head {
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.inspector-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-top: 10px;
}

.inspector-grid.compact {
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.inspector-grid > div {
  min-width: 0;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px;
  background: var(--background);
}

.fact-label {
  display: block;
  color: var(--muted-foreground);
  font-size: 11px;
  font-weight: 700;
  margin-bottom: 4px;
}
```

Extend the existing media query:

```css
@media (max-width: 820px) {
  .provider-grid,
  .inspector-grid,
  .inspector-grid.compact {
    grid-template-columns: 1fr;
  }
}
```

- [ ] **Step 7: Run frontend tests and Svelte check**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel-state.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 8: Commit Task 4**

Run:

```powershell
git add src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/gemini-browser-provider-panel.test.ts
git commit -m "feat: show Gemini browser run inspector"
```

---

### Task 5: Full Verification And Documentation Touch-Up

**Files:**
- Modify: `docs/browser-providers-llm-troubleshooting.md`
- Modify: `docs/architecture-deep-dive.md`

- [ ] **Step 1: Update troubleshooting docs**

In `docs/browser-providers-llm-troubleshooting.md`, add a short section after `Status And Run Lifecycle`:

```md
## Inline Run Inspector

The Browser Providers panel includes an inline run inspector for the selected
active run or the newest recent run. Use it before opening app-data manually.

The inspector shows status, elapsed time, answer length, artifact availability,
manual action, and sidecar `debug_summary` facts such as composer/send/answer
selection and wait durations.

`Copy diagnostics` intentionally omits full local artifact paths, prompt text,
answer text, raw DOM, screenshots, cookies, and account identifiers. It is the
preferred first payload to paste into an LLM debugging session.
```

- [ ] **Step 2: Update architecture docs**

In `docs/architecture-deep-dive.md`, extend the Browser Providers run-log paragraph with:

```md
New run results may include an optional sanitized `debug_summary` generated by
the sidecar. The summary is a UI/debugging contract, not a persistence authority:
it records selector-stage facts, wait timings, and final text length while
excluding prompt text, answer text, raw DOM, screenshots, cookies, and account
identifiers.
```

- [ ] **Step 3: Run full relevant automated checks**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel-state.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
git diff --check
```

Expected: PASS for all commands and no whitespace errors.

- [ ] **Step 4: Rebuild packaged sidecar binary**

Run:

```powershell
npm.cmd run build:gemini-browser-sidecar
```

Expected: PASS and writes `src-tauri\binaries\gemini-browser-sidecar-x86_64-pc-windows-msvc.exe` on Windows.

- [ ] **Step 5: Manual validation**

Run the app and validate:

```powershell
npm.cmd run tauri dev
```

In the app:

1. Open Settings -> Browser Providers.
2. Select `Attach Chrome`.
3. Click `Start Chrome`.
4. Open or resume Gemini.
5. Send `Reply with one short sentence confirming the browser provider is connected.`
6. Confirm the response appears.
7. Confirm `Run inspector` shows `status: ok`, elapsed time, answer length, artifact availability, and debug facts.
8. Click `Copy diagnostics` and confirm the copied text has no full local paths and no answer text.
9. Click `Open run folder` and confirm the recorded run directory opens.

- [ ] **Step 6: Commit Task 5**

Run:

```powershell
git add docs/browser-providers-llm-troubleshooting.md docs/architecture-deep-dive.md
git commit -m "docs: document Gemini browser run inspector"
```

---

## Self-Review Checklist

- Spec coverage: Tasks cover sidecar `debug_summary`, Rust pass-through and persistence, open folder command, frontend DTOs, sanitized copy diagnostics, inline inspector UI, tests, docs, and manual validation.
- Scope check: No new route, no artifact viewer, no DOM/HTML rendering in the frontend, and no CDP security changes.
- Type consistency: `debug_summary` is the field name in TypeScript, Rust serde JSON, frontend DTOs, and run-log JSON. Error stages use `setup`, `composer`, `send`, `answer`, `artifacts`, and `transport`.
- Privacy check: copied diagnostics uses artifact availability flags and debug facts, not prompt text, answer text, full local paths, raw DOM, screenshots, cookies, or account identifiers.
