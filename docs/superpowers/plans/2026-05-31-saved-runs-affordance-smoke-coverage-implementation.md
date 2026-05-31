# Saved Runs Affordance Smoke Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic Tauri GUI smoke coverage for missing-legacy and capture-failed saved-run snapshot affordances.

**Architecture:** Extend the existing analysis redesign fixture seed with one capture-failed saved run and make fixture DTO expectations explicit in Rust tests. Then extend the existing `npm.cmd run smoke:analysis` harness with saved-run affordance steps that reuse the real Runs, Source, Evidence, and Chat UI flows, plus a fresh verification note after the smoke passes.

**Tech Stack:** Rust, SQLx, Tauri, Node smoke harness, Svelte UI driven through the MCP bridge, Vitest raw script/helper contracts, full project verification through `npm.cmd run verify`.

---

## Execution Protocol

- Start on branch `saved-runs-affordance-smoke-coverage`; do not create a new branch for this slice.
- Before implementation code, use `superpowers:test-driven-development`.
- Execute tasks in order. Each task leaves focused tests passing before committing.
- Mark completed checkboxes in this plan as work progresses.
- Do not change production snapshot classification, report execution behavior, saved-run affordance helper copy, migrations, cleanup flows, or UI component behavior unless a test exposes a genuine mismatch.
- If the first `npm.cmd run smoke:analysis` fails because the Rust/Tauri build is still compiling and no MCP bridge is discovered, rerun the command once after the build is warm. Treat a second bridge-discovery failure as a real issue.

## Files

- Modify: `src-tauri/src/analysis/fixtures.rs`
  - Add the capture-failed fixture run and fixture-level DTO tests for missing-legacy and capture-failed snapshot states.
- Modify: `scripts/analysis-smoke-helpers.mjs`
  - Expose degraded saved-run fixture labels and update the expected seeded run minimum.
- Modify: `scripts/analysis-smoke.mjs`
  - Add saved-run affordance smoke steps and local helpers for row-scoped checks, opened-run text, Source mode checks, Evidence disabled reasons, and Chat disabled copy.
- Modify: `src/lib/analysis-smoke-helpers.test.ts`
  - Add fast contract tests for the new fixture labels and `runs >= 7` summary minimum.
- Modify: `src/lib/analysis-ui-smoke-contract.test.ts`
  - Add fast contract tests for the new saved-run smoke step group and required helper names/text fragments.
- Create: `docs/superpowers/verification/2026-05-31-saved-runs-affordance-smoke.md`
  - Record the smoke result, fixture summary, pass table, cleanup confirmation, and cold-build caveat.
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md`
  - Track completed checkboxes during execution.

---

### Task 1: Fixture Contracts And Capture-Failed Run

**Files:**
- Modify: `src-tauri/src/analysis/fixtures.rs`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md`

- [x] **Step 1: Write failing fixture contract tests**

In `src-tauri/src/analysis/fixtures.rs`, update `summary_serializes_with_camel_case_keys` so the sample summary uses `runs: 7`:

```rust
        let summary = AnalysisRedesignFixtureSummary {
            accounts: 1,
            llm_profiles: 1,
            sources: 4,
            source_groups: 1,
            prompt_templates: 1,
            runs: 7,
            snapshot_messages: 4,
            chat_messages: 2,
            youtube_transcript_segments: 3,
            youtube_playlist_items: 2,
        };
```

In `seed_creates_fixture_runs_with_statuses_templates_and_snapshots`, change these expected counts:

```rust
        assert_eq!(summary.runs, 7);
```

```rust
            7
```

for the `prompt_template_id IS NOT NULL` assertion, and:

```rust
            2
```

for the `status = 'failed'` assertion.

In `seed_twice_keeps_one_deterministic_fixture_set`, change the
`scope_label_snapshot LIKE '__analysis_redesign_fixture__%'` run count to:

```rust
            7
```

Replace `missing_snapshot_run_has_trace_but_no_saved_messages` with this stricter test:

```rust
    #[tokio::test]
    async fn missing_snapshot_run_exposes_legacy_state_but_no_saved_messages() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(MISSING_SNAPSHOT_RUN_LABEL)
                .fetch_one(&pool)
                .await
                .expect("load missing snapshot run");

        let summaries = crate::analysis::store::list_analysis_run_summaries(
            &pool,
            crate::analysis::store::AnalysisRunListFilters {
                query: Some(MISSING_SNAPSHOT_RUN_LABEL.to_string()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .expect("list fixture runs");
        let summary = summaries
            .iter()
            .find(|run| run.scope_label == MISSING_SNAPSHOT_RUN_LABEL)
            .expect("missing snapshot summary");
        assert_eq!(
            summary.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::MissingLegacy)
        );

        let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
            .await
            .expect("fetch missing snapshot run")
            .map(crate::analysis::store::map_run_detail)
            .expect("missing snapshot run exists");
        assert_eq!(
            detail.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::MissingLegacy)
        );
        assert_eq!(detail.snapshot_error, None);

        assert_eq!(
            count(
                &pool,
                &format!("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = {run_id}")
            )
            .await,
            0
        );
        assert_eq!(
            count(
                &pool,
                &format!(
                    "SELECT COUNT(*) FROM analysis_runs WHERE id = {run_id} AND trace_data_zstd IS NOT NULL"
                )
            )
            .await,
            1
        );
    }
```

After that test, add the capture-failed fixture test:

```rust
    #[tokio::test]
    async fn capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(CAPTURE_FAILED_SNAPSHOT_RUN_LABEL)
                .fetch_one(&pool)
                .await
                .expect("load capture failed snapshot run");

        let summaries = crate::analysis::store::list_analysis_run_summaries(
            &pool,
            crate::analysis::store::AnalysisRunListFilters {
                query: Some(CAPTURE_FAILED_SNAPSHOT_RUN_LABEL.to_string()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .expect("list fixture runs");
        let summary = summaries
            .iter()
            .find(|run| run.scope_label == CAPTURE_FAILED_SNAPSHOT_RUN_LABEL)
            .expect("capture failed summary");
        assert_eq!(
            summary.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
        );
        assert_eq!(
            summary.snapshot_error.as_deref(),
            Some(CAPTURE_FAILED_SNAPSHOT_ERROR)
        );

        let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
            .await
            .expect("fetch capture failed snapshot run")
            .map(crate::analysis::store::map_run_detail)
            .expect("capture failed snapshot run exists");
        assert_eq!(detail.status, "failed");
        assert!(detail
            .result_markdown
            .as_deref()
            .unwrap_or_default()
            .contains("This capture-failed fixture report remains readable."));
        assert_eq!(
            detail.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
        );
        assert_eq!(
            detail.snapshot_error.as_deref(),
            Some(CAPTURE_FAILED_SNAPSHOT_ERROR)
        );
        assert_eq!(detail.snapshot_captured_at, None);

        assert_eq!(
            count(
                &pool,
                &format!("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = {run_id}")
            )
            .await,
            0
        );
        assert_eq!(
            count(
                &pool,
                &format!(
                    "SELECT COUNT(*) FROM analysis_runs WHERE id = {run_id} AND trace_data_zstd IS NOT NULL"
                )
            )
            .await,
            1
        );
    }
```

- [x] **Step 2: Run fixture tests and verify the red state**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures
```

Expected: FAIL. The run counts still report `6`, and `CAPTURE_FAILED_SNAPSHOT_RUN_LABEL` / `CAPTURE_FAILED_SNAPSHOT_ERROR` are not defined.

- [x] **Step 3: Add capture-failed fixture constants**

In `src-tauri/src/analysis/fixtures.rs`, add these constants after `MISSING_SNAPSHOT_RUN_LABEL`:

```rust
const CAPTURE_FAILED_SNAPSHOT_RUN_LABEL: &str =
    "__analysis_redesign_fixture__ Capture Failed Snapshot Run";
const CAPTURE_FAILED_SNAPSHOT_ERROR: &str =
    "Snapshot capture failed: fixture write boundary unavailable";
```

- [x] **Step 4: Add a fixture helper for snapshot capture failure**

After `mark_fixture_snapshot_captured`, add:

```rust
async fn mark_fixture_snapshot_capture_failed(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    run_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE analysis_runs
         SET snapshot_captured_at = NULL, snapshot_error = ?
         WHERE id = ?",
    )
    .bind(CAPTURE_FAILED_SNAPSHOT_ERROR)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [x] **Step 5: Seed the capture-failed run**

In `insert_analysis_runs`, after the existing `Missing Snapshot Run` insertion and before the `for (label, status, error, completed_at)` loop, add:

```rust
    let capture_failed_ref = format!("s{}-i999998", ids.telegram_channel_id);
    let capture_failed_run_id = insert_run(
        tx,
        CAPTURE_FAILED_SNAPSHOT_RUN_LABEL,
        "single_source",
        Some(ids.telegram_channel_id),
        None,
        ids.prompt_template_id,
        "failed",
        Some(&format!(
            "# {CAPTURE_FAILED_SNAPSHOT_RUN_LABEL}\n\nProvider fixture: {LLM_PROFILE_LABEL}.\n\nThis capture-failed fixture report remains readable.\n\nThis report cites capture-failed saved evidence [{capture_failed_ref}]."
        )),
        Some(trace_zstd(serde_json::json!([{
            "ref": capture_failed_ref,
            "item_id": 999998,
            "source_id": ids.telegram_channel_id,
            "external_id": "capture-failed-fixture-item",
            "published_at": FIXTURE_PERIOD_FROM + 200,
            "excerpt": "Capture failed fixture evidence",
            "youtube_url": null,
            "youtube_timestamp_seconds": null,
            "youtube_display_label": null,
            "is_synthetic": false
        }]))?),
        None,
        Some(FIXTURE_NOW + 40),
    )
    .await?;
    mark_fixture_snapshot_capture_failed(tx, capture_failed_run_id).await?;
```

Then update the existing failed/cancelled loop timestamps to keep fixture chronology readable:

```rust
        (
            FAILED_RUN_LABEL,
            "failed",
            Some("Fixture failure: provider request failed without changing user data"),
            Some(FIXTURE_NOW + 50),
        ),
        (
            CANCELLED_RUN_LABEL,
            "cancelled",
            Some("Fixture cancellation: run was cancelled before snapshot capture"),
            Some(FIXTURE_NOW + 60),
        ),
```

Update the source-group snapshot run completion timestamp from `FIXTURE_NOW + 60` to:

```rust
        Some(FIXTURE_NOW + 70),
```

Update the chat fixture timestamps from `FIXTURE_NOW + 70` and `FIXTURE_NOW + 71` to:

```rust
        ("user", "Summarize the strongest fixture evidence.", FIXTURE_NOW + 80),
```

and:

```rust
            FIXTURE_NOW + 81,
```

- [x] **Step 6: Update seeded summary count**

In `seed_analysis_redesign_fixtures_in_pool`, update the returned summary:

```rust
        runs: 7,
```

- [x] **Step 7: Run fixture tests and verify green**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures
```

Expected: PASS. The output includes all `analysis::fixtures` tests passing.

- [x] **Step 8: Commit fixture changes**

Run:

```powershell
git add src-tauri/src/analysis/fixtures.rs docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md
git commit -m "feat: add capture failed smoke fixture"
```

Expected: commit succeeds.

---

### Task 2: Smoke Helper Labels And Fixture Summary Contract

**Files:**
- Modify: `scripts/analysis-smoke-helpers.mjs`
- Modify: `src/lib/analysis-smoke-helpers.test.ts`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md`

- [ ] **Step 1: Write failing helper contract tests**

In `src/lib/analysis-smoke-helpers.test.ts`, extend `validates required fixture labels` with:

```ts
    expect(expectedFixtureLabels).toEqual(expect.arrayContaining([
      "__analysis_redesign_fixture__ Missing Snapshot Run",
      "__analysis_redesign_fixture__ Failed Run",
      "__analysis_redesign_fixture__ Cancelled Run",
      "__analysis_redesign_fixture__ Capture Failed Snapshot Run",
    ]));
```

In `validates deterministic fixture summary minimums`, change the positive summary to `runs: 7`:

```ts
      runs: 7,
```

After the positive fixture summary assertion, add this negative assertion:

```ts
    expect(() => validateFixtureSummary({
      accounts: 1,
      chatMessages: 2,
      llmProfiles: 1,
      promptTemplates: 1,
      runs: 6,
      snapshotMessages: 4,
      sourceGroups: 1,
      sources: 4,
      youtubePlaylistItems: 2,
      youtubeTranscriptSegments: 3,
    })).toThrow(SmokeAssertionError);
```

- [ ] **Step 2: Run helper tests and verify the red state**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-smoke-helpers.test.ts
```

Expected: FAIL because the helper does not expose the degraded saved-run labels and still accepts `runs: 6`.

- [ ] **Step 3: Add degraded saved-run labels**

In `scripts/analysis-smoke-helpers.mjs`, extend `fixtureLabels` to:

```js
export const fixtureLabels = {
  telegramChannel: "__analysis_redesign_fixture__ Telegram Channel",
  telegramSupergroup: "__analysis_redesign_fixture__ Telegram Supergroup",
  youtubeVideo: "__analysis_redesign_fixture__ YouTube Video",
  youtubePlaylist: "__analysis_redesign_fixture__ YouTube Playlist",
  telegramSourceGroup: "__analysis_redesign_fixture__ Telegram Source Group",
  completedSnapshotRun: "__analysis_redesign_fixture__ Completed Snapshot Run",
  missingSnapshotRun: "__analysis_redesign_fixture__ Missing Snapshot Run",
  failedRun: "__analysis_redesign_fixture__ Failed Run",
  cancelledRun: "__analysis_redesign_fixture__ Cancelled Run",
  captureFailedSnapshotRun: "__analysis_redesign_fixture__ Capture Failed Snapshot Run",
  groupSnapshotRun: "__analysis_redesign_fixture__ Group Snapshot Run",
};
```

- [ ] **Step 4: Update smoke fixture summary minimum**

In `scripts/analysis-smoke-helpers.mjs`, change the `runs` expected minimum in `validateFixtureSummary`:

```js
    runs: 7,
```

- [ ] **Step 5: Run helper tests and verify green**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-smoke-helpers.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit helper label changes**

Run:

```powershell
git add scripts/analysis-smoke-helpers.mjs src/lib/analysis-smoke-helpers.test.ts docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md
git commit -m "test: expose degraded saved run smoke fixtures"
```

Expected: commit succeeds.

---

### Task 3: Saved-Run Affordance Smoke Steps

**Files:**
- Modify: `scripts/analysis-smoke.mjs`
- Modify: `src/lib/analysis-ui-smoke-contract.test.ts`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md`

- [ ] **Step 1: Write failing smoke runner contract**

In `src/lib/analysis-ui-smoke-contract.test.ts`, extend `keeps the smoke runner organized around deterministic named steps` with:

```ts
    expect(smokeScriptSource).toContain("savedRunAffordanceSmokeSteps");
    expect(smokeScriptSource).toContain("saved-runs-affordance.rows");
    expect(smokeScriptSource).toContain("saved-runs-affordance.missing-legacy");
    expect(smokeScriptSource).toContain("saved-runs-affordance.capture-failed");
    expect(smokeScriptSource).toContain("assertRunRowAffordance");
    expect(smokeScriptSource).toContain("openEvidenceForRun");
    expect(smokeScriptSource).toContain("assertShowInSourceDisabledReason");
    expect(smokeScriptSource).toContain("Legacy snapshot missing");
    expect(smokeScriptSource).toContain("Snapshot capture failed: fixture write boundary unavailable");
    expect(smokeScriptSource).toContain("This capture-failed fixture report remains readable.");
    expect(smokeScriptSource).toContain("...sourceBrowserSmokeSteps, ...savedRunAffordanceSmokeSteps, ...analysisWorkspaceParitySteps");
```

- [ ] **Step 2: Run smoke contract test and verify the red state**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-ui-smoke-contract.test.ts
```

Expected: FAIL because `scripts/analysis-smoke.mjs` does not define the saved-run affordance step group or helpers.

- [ ] **Step 3: Add smoke constants**

In `scripts/analysis-smoke.mjs`, after `const probeOnly = args.has("--probe-only");`, add:

```js
const captureFailedSnapshotErrorText = "Snapshot capture failed: fixture write boundary unavailable";
const captureFailedReportText = "This capture-failed fixture report remains readable.";
```

- [ ] **Step 4: Add saved-run affordance smoke steps**

After `sourceBrowserSmokeSteps`, add:

```js
export const savedRunAffordanceSmokeSteps = [
  {
    name: "saved-runs-affordance.rows",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await assertRunRowAffordance(
        ctx,
        fixtureLabels.missingSnapshotRun,
        ["Legacy snapshot missing"],
        [captureFailedSnapshotErrorText, "fixture write boundary unavailable"],
      );
      await assertRunRowAffordance(
        ctx,
        fixtureLabels.captureFailedSnapshotRun,
        ["Snapshot capture failed"],
        [captureFailedSnapshotErrorText, "fixture write boundary unavailable"],
      );
      await assertRunLabelDiscoverable(ctx, fixtureLabels.failedRun);
      await assertRunLabelDiscoverable(ctx, fixtureLabels.cancelledRun);
    },
  },
  {
    name: "saved-runs-affordance.missing-legacy",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await openRun(ctx, fixtureLabels.missingSnapshotRun);
      await switchCanvasMode(ctx, "report");
      await openRunDetails(ctx);
      const opened = await openedRunText(ctx);
      assertTextContains(
        opened.header,
        "Saved report is readable, but this legacy run has no saved source snapshot.",
        "missing legacy opened header",
      );
      assertTextContains(opened.header, "Legacy run has no saved snapshot", "missing legacy details");

      await switchToSourceSurface(ctx);
      const sourceText = await sourceSurfaceText(ctx);
      assertTextContains(
        sourceText,
        "Older saved runs may not include frozen source rows",
        "missing legacy Source detail",
      );
      assertTextContains(sourceText, "Snapshot unavailable", "missing legacy Source badge");
      assertTextOmits(sourceText, "Run snapshot\nSources", "missing legacy Source saved snapshot browser");
      await assertLiveSourceClarificationIfAvailable(ctx);

      await assertEvidenceDisabledForRun(
        ctx,
        fixtureLabels.missingSnapshotRun,
        "legacy run has no saved source snapshot",
      );
      await assertChatDisabledForOpenedRun(ctx, "Older saved runs may not include frozen source rows");
    },
  },
  {
    name: "saved-runs-affordance.capture-failed",
    async run(ctx) {
      await navigateAnalysis(ctx);
      await openRun(ctx, fixtureLabels.captureFailedSnapshotRun);
      await switchCanvasMode(ctx, "report");
      await openRunDetails(ctx);
      const opened = await openedRunText(ctx);
      assertTextContains(
        opened.header,
        "Saved report is readable, but Extractum could not save the frozen source context for this run.",
        "capture failed opened header",
      );
      assertTextContains(opened.header, "Snapshot capture failed", "capture failed details");
      assertTextContains(opened.header, captureFailedSnapshotErrorText, "capture failed details error");
      assertTextContains(opened.report, captureFailedReportText, "capture failed report body");

      await switchToSourceSurface(ctx);
      const sourceText = await sourceSurfaceText(ctx);
      assertTextContains(
        sourceText,
        "Extractum could not save the frozen source context",
        "capture failed Source detail",
      );
      assertTextContains(sourceText, captureFailedSnapshotErrorText, "capture failed Source error");
      await assertLiveSourceClarificationIfAvailable(ctx);
      await clickLiveSourceIfAvailable(ctx);

      await assertEvidenceDisabledForRun(
        ctx,
        fixtureLabels.captureFailedSnapshotRun,
        "snapshot capture failed",
      );
      await assertChatDisabledForOpenedRun(ctx, "could not save the frozen source context");
    },
  },
];
```

- [ ] **Step 5: Add smoke text assertion helpers**

After `runSmokeSteps`, add:

```js
function assertTextContains(text, fragment, label) {
  if (!String(text ?? "").includes(fragment)) {
    throw new SmokeAssertionError(`${label} missing text: ${fragment}`);
  }
}

function assertTextOmits(text, fragment, label) {
  if (String(text ?? "").includes(fragment)) {
    throw new SmokeAssertionError(`${label} unexpectedly included text: ${fragment}`);
  }
}
```

- [ ] **Step 6: Add row-scoped Runs helpers**

After `openRunsTab`, add:

```js
async function runRowText(ctx, label) {
  await closeTransientUi(ctx);
  await openRunsTab(ctx);
  await fillByLabel(ctx.socket, "Search runs", label);
  await waitForText(ctx.socket, label);
  return executeJs(ctx.socket, `
    const panel = document.querySelector('[data-smoke-id="run-companion-runs-panel"]');
    if (!panel) throw new Error('ASSERT: runs panel missing');
    const label = ${JSON.stringify(label)};
    const rowCandidates = Array.from(panel.querySelectorAll('li, article, .source-row, .group-row, button, [role="row"]'));
    const row = rowCandidates.find((candidate) => candidate.innerText.includes(label));
    if (!row) throw new Error('ASSERT: run row missing: ' + label);
    return row.innerText;
  `);
}

async function assertRunRowAffordance(ctx, label, expectedFragments, forbiddenFragments = []) {
  const text = await runRowText(ctx, label);
  assertTextContains(text, label, `${label} row`);
  for (const fragment of expectedFragments) {
    assertTextContains(text, fragment, `${label} row`);
  }
  for (const fragment of forbiddenFragments) {
    assertTextOmits(text, fragment, `${label} row`);
  }
}

async function assertRunLabelDiscoverable(ctx, label) {
  const text = await runRowText(ctx, label);
  assertTextContains(text, label, `${label} row`);
}
```

- [ ] **Step 7: Add opened-run and Source helpers**

After `waitForOpenedRunSurface`, add:

```js
async function openRunDetails(ctx) {
  return executeJs(ctx.socket, `
    const details = document.querySelector('.report-run-header details.run-details');
    if (!details) throw new Error('ASSERT: run details missing');
    details.open = true;
    return true;
  `);
}

async function openedRunText(ctx) {
  return executeJs(ctx.socket, `
    return {
      header: document.querySelector('.report-run-header')?.innerText ?? "",
      report: document.querySelector('.report-viewer')?.innerText ?? "",
      source: document.querySelector('[data-smoke-id="analysis-source-surface"]')?.innerText ?? "",
      companion: document.querySelector('#run-companion-panel')?.innerText ?? "",
    };
  `);
}

async function switchToSourceSurface(ctx) {
  await clickBySmokeId(ctx.socket, "report-canvas-mode-source");
  await waitForSmokeId(ctx, "analysis-source-surface", 30000);
}

async function sourceSurfaceText(ctx) {
  return executeJs(ctx.socket, `
    const surface = document.querySelector('[data-smoke-id="analysis-source-surface"]');
    if (!surface) throw new Error('ASSERT: source surface missing');
    return surface.innerText;
  `);
}

async function assertLiveSourceClarificationIfAvailable(ctx) {
  const text = await sourceSurfaceText(ctx);
  if (!text.includes("View live source")) return;
  assertTextContains(
    text,
    "View live source opens current source data. This is live data, not the saved run snapshot.",
    "live source clarification",
  );
}

async function clickLiveSourceIfAvailable(ctx) {
  const clicked = await executeJs(ctx.socket, `
    const surface = document.querySelector('[data-smoke-id="analysis-source-surface"]');
    if (!surface) throw new Error('ASSERT: source surface missing');
    const button = Array.from(surface.querySelectorAll('button'))
      .find((candidate) => candidate.innerText.includes('View live source'));
    if (!button) return false;
    button.click();
    return true;
  `);

  if (!clicked) return false;
  await waitForText(ctx.socket, "Live source");
  const text = await sourceSurfaceText(ctx);
  assertTextContains(text, "Live source", "live source header");
  assertTextOmits(text, captureFailedSnapshotErrorText, "live source after capture failed switch");
  assertTextOmits(text, "Legacy run has no saved snapshot", "live source after missing legacy switch");
  return true;
}
```

- [ ] **Step 8: Add companion tab and Evidence/Chat helpers**

After `openRun`, add:

```js
async function openCompanionTab(ctx, label) {
  await executeJs(ctx.socket, `
    const tablist = document.querySelector('[aria-label="Run companion tabs"]');
    if (!tablist) throw new Error('ASSERT: run companion tablist missing');
    const label = ${JSON.stringify(label)};
    const tab = Array.from(tablist.querySelectorAll('button, [role="tab"]'))
      .find((candidate) => candidate.innerText.trim().includes(label));
    if (!tab) throw new Error('ASSERT: companion tab missing: ' + label);
    tab.click();
    return true;
  `);
}

async function selectFirstTraceRefIfNeeded(ctx) {
  await executeJs(ctx.socket, `
    const panel = document.querySelector('.run-evidence-tab');
    if (!panel) throw new Error('ASSERT: evidence panel missing');
    if (panel.querySelector('.trace-detail')) return true;
    const firstTrace = panel.querySelector('.trace-link');
    if (!firstTrace) throw new Error('ASSERT: trace ref missing for evidence');
    firstTrace.click();
    return true;
  `);
  await waitForText(ctx.socket, "Show in source");
}

async function openEvidenceForRun(ctx, runLabel) {
  await openRun(ctx, runLabel);
  await openCompanionTab(ctx, "Evidence");
  await waitForText(ctx.socket, "Traceability");
  await selectFirstTraceRefIfNeeded(ctx);
}

async function assertShowInSourceDisabledReason(ctx, reasonFragment) {
  const buttonState = await executeJs(ctx.socket, `
    const panel = document.querySelector('.run-evidence-tab');
    if (!panel) throw new Error('ASSERT: evidence panel missing');
    const button = Array.from(panel.querySelectorAll('button'))
      .find((candidate) => candidate.innerText.includes('Show in source'));
    if (!button) throw new Error('ASSERT: Show in source button missing');
    return {
      disabled: Boolean(button.disabled),
      title: button.getAttribute('title') ?? "",
      text: button.innerText,
    };
  `);

  if (!buttonState.disabled) {
    throw new SmokeAssertionError("Show in source button is not disabled");
  }
  assertTextContains(buttonState.title, reasonFragment, "Show in source disabled reason");
}

async function assertEvidenceDisabledForRun(ctx, runLabel, reasonFragment) {
  await openEvidenceForRun(ctx, runLabel);
  const companionText = (await openedRunText(ctx)).companion;
  assertTextContains(companionText, "Snapshot unavailable:", `${runLabel} Evidence unavailable copy`);
  await assertShowInSourceDisabledReason(ctx, reasonFragment);
}

async function assertChatDisabledForOpenedRun(ctx, descriptionFragment) {
  await openCompanionTab(ctx, "Chat");
  await waitForText(ctx.socket, "Saved context unavailable");
  const companionText = (await openedRunText(ctx)).companion;
  assertTextContains(companionText, "Saved context unavailable", "Chat disabled title");
  assertTextContains(companionText, descriptionFragment, "Chat disabled description");
}
```

- [ ] **Step 9: Include saved-run steps in smoke execution**

In `main`, replace:

```js
    await runSmokeSteps(ctx, [...sourceBrowserSmokeSteps, ...analysisWorkspaceParitySteps]);
```

with:

```js
    await runSmokeSteps(ctx, [...sourceBrowserSmokeSteps, ...savedRunAffordanceSmokeSteps, ...analysisWorkspaceParitySteps]);
```

- [ ] **Step 10: Run smoke contract test and verify green**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-ui-smoke-contract.test.ts
```

Expected: PASS.

- [ ] **Step 11: Run focused helper and smoke contract tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-smoke-helpers.test.ts src/lib/analysis-ui-smoke-contract.test.ts
```

Expected: PASS.

- [ ] **Step 12: Run GUI smoke**

Run:

```powershell
npm.cmd run smoke:analysis
```

Expected: PASS with these saved-run steps in the output:

```text
PASS saved-runs-affordance.rows
PASS saved-runs-affordance.missing-legacy
PASS saved-runs-affordance.capture-failed
Analysis UI smoke passed.
```

If the first attempt fails with `No org.ai.extractum MCP bridge found on ports 9223-9322` while Rust/Tauri is still compiling, rerun once:

```powershell
npm.cmd run smoke:analysis
```

Expected for the warmed rerun: PASS with `Analysis UI smoke passed.`

- [ ] **Step 13: Commit smoke steps**

Run:

```powershell
git add scripts/analysis-smoke.mjs src/lib/analysis-ui-smoke-contract.test.ts docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md
git commit -m "test: smoke degraded saved run affordances"
```

Expected: commit succeeds.

---

### Task 4: Verification Note And Full Verification

**Files:**
- Create: `docs/superpowers/verification/2026-05-31-saved-runs-affordance-smoke.md`
- Modify: `docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md`

- [ ] **Step 1: Run required focused frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-run-snapshot-affordance.test.ts src/lib/analysis-run-companion-state.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run required fixture tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures
```

Expected: PASS.

- [ ] **Step 3: Run full verification**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS. The output should include frontend tests, `svelte-check`, Rust tests, and verification script completion.

- [ ] **Step 4: Create verification note**

Create `docs/superpowers/verification/2026-05-31-saved-runs-affordance-smoke.md` with:

````markdown
# Saved Runs Affordance Smoke Verification

> Date: 2026-05-31
> Branch: `saved-runs-affordance-smoke-coverage`

## Commands

```powershell
npm.cmd run smoke:analysis
npm.cmd run test -- src/lib/analysis-run-snapshot-affordance.test.ts src/lib/analysis-run-companion-state.test.ts
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures
npm.cmd run verify
```

## Result

The analysis UI smoke passed with the saved-run affordance step group included.

The accepted smoke run includes:

- `PASS saved-runs-affordance.rows`
- `PASS saved-runs-affordance.missing-legacy`
- `PASS saved-runs-affordance.capture-failed`
- `Analysis UI smoke passed.`

## Fixture Summary

The seeded analysis redesign fixture set includes at least:

- `runs: 7`
- `snapshotMessages: 4`
- `chatMessages: 2`
- `sourceGroups: 1`
- `sources: 4`

The degraded saved-run labels covered by smoke are:

- `__analysis_redesign_fixture__ Missing Snapshot Run`
- `__analysis_redesign_fixture__ Capture Failed Snapshot Run`
- `__analysis_redesign_fixture__ Failed Run`
- `__analysis_redesign_fixture__ Cancelled Run`

## PASS Table

| Area | Evidence |
| --- | --- |
| Runs rows | Missing legacy and capture-failed rows show degraded badges, with error details omitted from row-scoped text. |
| Missing legacy | Opened-run details, Source, Evidence, and Chat expose helper-derived missing-snapshot affordances. |
| Capture failed | Opened report remains readable and details/Source show sanitized snapshot error text. |
| Live source explicitness | Degraded Source view explains that View live source opens live data, not the saved run snapshot. |
| Existing Source Browser smoke | Telegram, YouTube video, YouTube playlist, live source group, and captured run snapshot tab checks still pass. |

## Startup Caveat

A cold Rust/Tauri build can exceed the smoke harness 90-second MCP bridge discovery window before the app finishes compiling. A single warmed rerun after a compile-only bridge timeout is acceptable; a second bridge-discovery failure should be treated as a real smoke harness or app startup failure.

## Cleanup

The smoke harness cleaned analysis redesign fixtures and stopped the Tauri dev process after the accepted run.
````

- [ ] **Step 5: Run verification-note grep checks**

Run:

```powershell
rg -n "saved-runs-affordance|runs: 7|Capture Failed Snapshot Run|Startup Caveat" docs/superpowers/verification/2026-05-31-saved-runs-affordance-smoke.md
```

Expected: output includes all four searched fragments.

- [ ] **Step 6: Commit verification note**

Run:

```powershell
git add docs/superpowers/verification/2026-05-31-saved-runs-affordance-smoke.md docs/superpowers/plans/2026-05-31-saved-runs-affordance-smoke-coverage-implementation.md
git commit -m "docs: record saved run affordance smoke"
```

Expected: commit succeeds.

---

## Acceptance Checklist

- [ ] Fixture tests assert `Missing Snapshot Run` exposes `snapshot_state: "missing_legacy"` in summary and detail DTOs.
- [ ] Fixture tests assert `Capture Failed Snapshot Run` exposes `snapshot_state: "capture_failed"` and `snapshot_error: "Snapshot capture failed: fixture write boundary unavailable"` in summary and detail DTOs.
- [ ] Capture-failed fixture is `failed`, has readable `result_markdown`, has trace data, has no saved snapshot rows, and has no `snapshot_captured_at`.
- [ ] Smoke labels include `Missing Snapshot Run`, `Failed Run`, `Cancelled Run`, and `Capture Failed Snapshot Run`.
- [ ] Runs row smoke checks are row-scoped and verify degraded badges without exposing sanitized error details.
- [ ] Missing-legacy smoke covers opened header details, Source copy, Evidence disabled reason, and Chat disabled copy.
- [ ] Capture-failed smoke covers readable report body, details sanitized error, Source sanitized error, Evidence disabled reason, and Chat disabled copy.
- [ ] Live-source clarification is asserted only when `View live source` is available, and clicking it proves the UI switches to live source basis.
- [ ] Fresh verification note records smoke evidence, fixture summary, cleanup, and cold-build caveat.
- [ ] Required commands pass: focused helper tests, `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures`, `npm.cmd run smoke:analysis`, and `npm.cmd run verify`.
