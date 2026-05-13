# Analysis Snapshot Preflight Stabilization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `/analysis` honest about run snapshot provenance and prevent report launch when required source/runtime/LLM prerequisites are missing.

**Architecture:** Keep the change narrow. Put report launch preflight rules in pure analysis state helpers, pass a single disabled reason into `ReportSetupPanel`, and make source-reader components explicitly distinguish live source actions from snapshot viewing.

**Tech Stack:** Svelte 5 runes, TypeScript, Vitest raw-source contract tests plus pure helper tests.

**Status:** Implemented on 2026-05-13.

**Implementation Summary:**
- Added `reportLaunchDisabledReason` as a pure preflight helper in `src/lib/analysis-state.ts`.
- Routed report launch disabled reasons from `/analysis` through `ReportCanvas` into `ReportSetupPanel`.
- Disabled `Run report` and surfaced a central preflight diagnostic when LLM/source prerequisites are not usable.
- Added `showSyncActions` to `YoutubeTranscriptReader` and disabled live sync actions in snapshot/readonly transcript views.
- Added explicit source-basis badge state so unavailable/pending snapshots no longer render as successful snapshots.
- Added a central live YouTube runtime diagnostic in `ReportSourceSurface`.

---

### Task 1: Report Launch Preflight Helper

**Files:**
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-setup-panel.svelte`

- [x] Add failing tests for `reportLaunchDisabledReason`:
  - no LLM profiles -> `Set up an LLM profile in Settings before running reports.`
  - selected/active profile has `api_key_configured: false` -> settings guidance
  - single source has a runtime disabled reason -> same reason
  - source group with an unavailable member source -> same reason with source label
  - valid source/profile/template selection -> `null`
- [x] Run `npm.cmd test -- --run src/lib/analysis-state.test.ts` and confirm the new tests fail because the helper is missing.
- [x] Implement `reportLaunchDisabledReason` in `analysis-state.ts` using existing `analysisReportStartCommand` validation plus LLM/source runtime checks.
- [x] Wire the helper in `+page.svelte` using current source, current group, source catalog, `sourceSyncDisabledReason`, and LLM profiles.
- [x] Pass `reportLaunchDisabledReason` through `ReportCanvas` into `ReportSetupPanel`.
- [x] Disable `Run report`, set its `title`, and show a central preflight `StatusMessage` when the reason is non-null.

### Task 2: Snapshot Reader Action Separation

**Files:**
- Modify: `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/source-group-reader.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`

- [x] Add failing raw-source tests that require `YoutubeTranscriptReader` to accept `showSyncActions` and require snapshot calls to pass `showSyncActions={false}`.
- [x] Run targeted tests and confirm they fail.
- [x] Add `showSyncActions = true` to `YoutubeTranscriptReader`.
- [x] Render metadata/transcript sync buttons only when `showSyncActions` is true.
- [x] Pass `showSyncActions={false}` from run snapshot and source group snapshot/readonly usages.
- [x] Keep live single-source YouTube transcript actions visible.

### Task 3: Snapshot Header State

**Files:**
- Modify: `src/lib/components/analysis/source-reader-header.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`

- [x] Add failing tests that require the header to accept `sourceBasisState` or equivalent and avoid success styling for unavailable/pending snapshots.
- [x] Run targeted tests and confirm failure.
- [x] Add an explicit badge state to `SourceReaderHeader`.
- [x] Map live source to warning, available snapshot to success, pending/unknown snapshot to neutral, and unavailable snapshot to danger.
- [x] Pass `sourceBasisState={canvasSurface}` from `ReportSourceSurface`.

### Task 4: Central YouTube Runtime Diagnostic

**Files:**
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [x] Add failing raw-source tests that require `ReportSourceSurface` to receive `sourceSyncDisabledReason` and render a central YouTube runtime diagnostic in live YouTube source mode.
- [x] Run targeted tests and confirm failure.
- [x] Pass `sourceSyncDisabledReason` into `ReportSourceSurface`.
- [x] In live YouTube source mode, show a `StatusMessage tone="error"` when the selected source has a disabled sync reason.
- [x] Leave snapshot mode free of live runtime diagnostic unless the user explicitly views live source.

### Task 5: Verification

**Files:**
- No production file changes unless verification exposes an issue.

- [x] Run `mcp__svelte_server__.svelte_autofixer` for changed Svelte components.
- [x] Run targeted tests:
  - `npm.cmd test -- --run src/lib/analysis-state.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts`
- [x] Run `npm.cmd run check`.
- [x] Run `npm.cmd test -- --run`.
- [x] Run `git diff --check`.
- [x] If the Tauri app is running, inspect `/analysis` with Tauri MCP for disabled run action and source-basis badges.

Verification evidence from 2026-05-13:

```text
npm.cmd test -- --run src/lib/analysis-state.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts
Test Files  3 passed (3)
Tests       59 passed (59)

npm.cmd run check
svelte-check found 0 errors and 0 warnings

npm.cmd test -- --run
Test Files  50 passed (50)
Tests       395 passed (395)

git diff --check
Exit 0, with only CRLF conversion warnings on Windows.
```

Runtime smoke:
- Connected to Tauri MCP on `localhost:9223`.
- Verified app `org.ai.extractum`, Tauri `2.10.3`, window URL `http://localhost:1420/analysis`.
- Source mode rendered the live YouTube transcript reader with live sync actions.
- Report mode rendered the report setup surface and Run report controls without route/runtime crashes.
