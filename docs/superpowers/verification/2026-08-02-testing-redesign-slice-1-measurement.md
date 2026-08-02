# Testing Redesign Slice 1 Measurement Verification

## Scope and Starting Commit

This record captures the Slice 1 baseline and controlled Rust diagnostic. The
starting commit was `5d614a300dfce135b2094a0c2ab3d75801ba9f93`. The baseline
driver completed with `baselineStatus: "observed-failures"` and driver exit
code `0`: observed failing commands are evidence of current behavior, not a
reason to replace them with later passing samples.

The authoritative artifacts are `artifacts/testing/slice-1/current-baseline.json`
and `artifacts/testing/slice-1/rust-feasibility.json`. Earlier invalid Rust
reports are preserved audit history and are not used below.

## Environment and Limitations

Tool versions observed by the version commands:

| Command | Output |
| --- | --- |
| `node --version` | `v24.13.1` |
| `npm.cmd --version` | `11.12.1` |
| `cargo --version` | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| `rustc --version` | `rustc 1.95.0 (59807616e 2026-04-14)` |

This is a local, incremental-cache study, not a portable CPU or memory claim.
The diagnostic also emitted pre-existing unused-code warnings and a Rust warning
that a corrupt incremental compilation artifact was ignored and deleted. These
conditions are disclosed as local limitations; they do not replace any proof
recorded in the authoritative report.

## Five-Field Timing Writer

`artifacts/testing/timings.jsonl` had 91 JSONL rows. Validation found zero
rows with an unexpected schema; every row has exactly `command`, `duration`,
`exitCode`, `commit`, and `startedAt`.

## Current Test Inventory

The file-backed reporters recorded the following complete inventories.

| Reporter | Result |
| --- | --- |
| frontend Vitest (`inventory/frontend-vitest.json`) | 386 suites, 380 passed; 1,671 tests, 1,667 passed; 181 files |
| sidecar Vitest (`inventory/sidecar-vitest.json`) | 12 suites, 12 passed; 50 tests, 50 passed; 5 files |
| adapter Vitest (`inventory/adapter-vitest.json`) | 8 suites, 8 passed; 9 tests, 9 passed; 4 files |
| adapter Playwright (`inventory/adapter-playwright.json`) | 7 suites, 73 specs, 73 tests; 6 files |

### Frontend Vitest files (181)

```text
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/run-vitest.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/tauri.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/process-shell-diagnostic/attempt.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/process-shell-diagnostic/coordinator.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/process-shell-diagnostic/git-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/process-shell-diagnostic/protocol.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/process-shell-diagnostic/report.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/process-shell-diagnostic/runtime.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/testing/run-observation.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/testing/slice-1-baseline.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/testing/slice-1-rust-feasibility.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/scripts/testing/timing-log.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/accounts-route-add-account-modal.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/accounts-ux-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-application-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-chat-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-chat-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-compact-source-rail.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-companion-layout.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-crate-boundary-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-editor-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-evidence-source-navigation.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-group-editor-props.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-legacy-surfaces-cleanup.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-llm-run-controls.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-migration-fixture-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-priority-ux-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-redesign-route-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-redesign-safety-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-redesign-workflow-scenarios.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-report-canvas-route.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-report-canvas-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-report-canvas.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-report-setup-props.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-report-workspace-selection-props.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-route-effects.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-route-workspace-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-run-companion-route.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-run-companion-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-run-companion-tabs.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-run-snapshot-affordance.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-run-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-scope-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-smoke-helpers.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-source-access-placement.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-source-groups-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-source-readers-route.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-source-readers.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-source-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-state-legacy-selection-cleanup.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-trace-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-ui-smoke-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-utils.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-workspace-persistence.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-workspace-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-workspace-tools.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-workspace-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/analysis-youtube-source-specialization.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/apalis-jobs-route-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/app-error.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/app-sidebar-behavior.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/crate-extraction-shell-cap-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/development-loop-performance-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/diagnostics-route-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/diagnostics-ux-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/diagnostics-view-model.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/dialog-bits-ui-migration.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/external-process-lifecycle-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/focused-rust-loop-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/gemini-browser-crate-boundary-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/gemini-browser-polling.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/gemini-browser-provider-panel-state.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/gemini-browser-provider-panel.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/gemini-browser-refresh-scheduler.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/gemini-browser-run-inspector.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/gemini-browser-setup-status.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/hidden-child-process-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/library-add-source-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/library-prototype-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/llm-crate-boundary-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/lucide-direct-import-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/media-metadata-core-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/project-runs-screen-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/project-runs-tab-delete-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/prompt-pack-application-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/prompt-pack-completion-transport-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/prompt-pack-crate-boundary-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/prompt-pack-run-control-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/prompt-pack-run-store-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/prompt-pack-runtime-config-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/prompt-pack-stage-execution-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/prompt-pack-stage-request-policy-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/provider-test-console-placement.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/research-projects-foundation-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/research-projects-import-boundary.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/research-projects-lucide-import-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/research-projects-route-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/rust-workspace-core-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/settings-profile-ux-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/source-access-placement.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/source-browser-model.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/source-capabilities.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/source-reader-model.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/tauri-security-config-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/telegram-crate-boundary-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/youtube-source-policy.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/youtube-source-view-model.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/youtube-summary-launch-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/youtube-summary-result-view-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/youtube-summary-runtime-preferences.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/youtube-summary-smoke-fixture-contract.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/youtube-thumbnail.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/research/gemini_browser_adapter/src/config.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/research/gemini_browser_adapter/src/scoring.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/research/gemini_browser_adapter/src/status.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/research/gemini_browser_adapter/src/telemetry.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/adapter.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/answer-extractor.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/cdp-endpoint.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/cdp-pages.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/protocol.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/accounts.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/analysis-chat.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/analysis-runs.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/analysis-source-groups.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/analysis-trace.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/analysis-workspace.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/apalis-jobs.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/diagnostics.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/gemini-browser.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/library-sources.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/llm.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/notebooklm-export.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/projects.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/prompt-packs.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/source-jobs.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/sources.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/takeout-import.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/youtube-detail.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/api/youtube-settings.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/library-add-source-model.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/library-add-source-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/library-catalog-grid.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/library-catalog-model.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/library-catalog-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/project-add-source-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-model.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-period.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-project-source-grid.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-rail-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-rail.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-source-filters.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-source-keyboard.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-source-row.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/research-projects-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/safe-markdown.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/ui/youtube-summary-workflow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/routes/projects/next/page-inspector.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/routes/projects/next/page-keyboard.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/analysis/source-browser-shell.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/extractum-ui/data-grid-date-format.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/extractum-ui/DataGrid.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/ComboSelect.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/Inspector.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/OptionsPanel.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/PeriodPanel.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/PeriodPopover.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/ProjectRailPanel.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/ProjectRow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/ProjectTabs.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/ProjectToolbar.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/ResearchProjectsShell.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/RunDock.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/SourcesBulkBar.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/SourcesFilterBar.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/SourcesFilterRow.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/SourcesGrid.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/SourcesTab.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/SourceStatusCell.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/src/lib/components/research-projects/SourceTitleCell.test.ts
```

### Sidecar and adapter Vitest files

```text
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/adapter.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/answer-extractor.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/cdp-endpoint.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/cdp-pages.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/sidecars/gemini-browser/src/protocol.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/research/gemini_browser_adapter/src/config.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/research/gemini_browser_adapter/src/scoring.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/research/gemini_browser_adapter/src/status.test.ts
G:/Develop/Extractum/.worktrees/testing-redesign-slice-1/research/gemini_browser_adapter/src/telemetry.test.ts
```

### Adapter Playwright files

```text
dom-only-baseline.spec.ts
failure-artifacts.spec.ts
matrix.spec.ts
mock-gemini.spec.ts
resilient-scoring.spec.ts
telemetry-assisted.spec.ts
```

## Current Gate Durations

All values below are literal `duration` and `exitCode` values from
`current-baseline.json`.

| Command | Duration (ms) | Exit code |
| --- | ---: | ---: |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run test -- --reporter=json --outputFile=G:\Develop\Extractum\.worktrees\testing-redesign-slice-1\artifacts\testing\slice-1\inventory\frontend-vitest.json` | 106435 | 1 |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run check` | 30732 | 0 |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run test:gemini-browser-sidecar:typecheck` | 2725 | 0 |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run test:gemini-browser-sidecar:unit -- --reporter=json --outputFile=G:\Develop\Extractum\.worktrees\testing-redesign-slice-1\artifacts\testing\slice-1\inventory\sidecar-vitest.json` | 5479 | 0 |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run test:gemini-browser-sidecar:build` | 2370 | 0 |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run test:gemini-browser-adapter:typecheck` | 2855 | 0 |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run test:gemini-browser-adapter:unit -- --reporter=json --outputFile=G:\Develop\Extractum\.worktrees\testing-redesign-slice-1\artifacts\testing\slice-1\inventory\adapter-vitest.json` | 4808 | 0 |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run test:gemini-browser-adapter:e2e -- --reporter=json` | 65210 | 0 |
| `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets` | 69668 | 101 |
| `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets` | 113807 | 101 |
| `C:\Windows\system32\cmd.exe /d /s /c npm.cmd run verify` | 86408 | 1 |

The non-zero results are observed current behavior:

- Frontend Vitest (106435 ms, exit 1) recorded four failed tests:
  - `process shell diagnostic runtime does not return until an owned grandchild tree is dead`: `AssertionError: expected 'termination_unconfirmed' to be 'timeout' // Object.is equality`.
  - `crate extraction timing policy checks the generated Grammers feature baseline`: `AssertionError: Telegram Grammers feature baseline: artifact content or formatting drifted; run with --write`.
  - `Phase 8B generated structural authority materializes the exact Phase 8B test partitions from content-addressed authority`: `AssertionError: Telegram Phase 8B test identity authority: artifact content or formatting drifted; run with --write`.
  - `Phase 8B generated structural authority materializes the exact restricted-visibility allowlist without duplicates`: `AssertionError: Telegram Phase 8B symbol authority: artifact content or formatting drifted; run with --write`.
- Cargo check (69668 ms, exit 101) and Cargo test (113807 ms, exit 101) are recorded as non-zero baseline observations.
- Full verify (86408 ms, exit 1) is recorded as a non-zero baseline observation.

No later passing run is substituted for those observations. The file-backed
frontend reporter records the exact four failures above; it does not record an
`afterAll` lifecycle-timeout signature for this baseline sample.

## Rust Diagnostic Protocol

The authoritative diagnostic report has `schemaVersion: 1`, test name
`readiness::tests::mark_failed_returns_failed_state`, `warmupRuns: 1`,
`retainedRuns: 3`, `valid: true`, and `exitCode: 0`.

The command shapes were:

| Shape | Literal command |
| --- | --- |
| `noopCheck`, `invalidatedCheck` | `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --lib --message-format=json` |
| `noRun` | `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib --no-run --message-format=json` |
| `directBinary` | `G:\Develop\Extractum\.worktrees\testing-redesign-slice-1\src-tauri\target\debug\deps\extractum_lib-51295d1508d3c6a6.exe readiness::tests::mark_failed_returns_failed_state --exact --nocapture` |
| `endToEnd` | `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib readiness::tests::mark_failed_returns_failed_state -- --exact` |

Every row terminated with `exit` and exit code 0. The report marks every row
valid and records source restoration for every row.

## Rust Retained Samples

Warm-ups are intentionally separate from the retained sample set. Durations
are milliseconds. The warm-up no-op records its actual `fresh: false` value;
the retained no-op controls each prove `fresh: true`.

### Warm-ups (5)

| Cohort | Shape | Duration | Proof |
| --- | --- | ---: | --- |
| `warmup-noop` | `noopCheck` | 4024 | non-test profile; `fresh: false` |
| `warmup-invalidated` | `invalidatedCheck` | 3731 | non-test profile; `fresh: false` |
| `warmup-no-run` | `noRun` | 10674 | test profile; `fresh: false`; canonical executable |
| `warmup-no-run` | `directBinary` | 139 | exact test: 1 passed, 0 failed, 0 ignored |
| `warmup-end-to-end` | `endToEnd` | 13681 | `compiledExtractum: true`; executable mtime increased; exact test: 1 passed, 0 failed, 0 ignored |

### Retained rows (15)

| Cohort/pair | Shape | Duration | Proof |
| --- | --- | ---: | --- |
| `noop-1` | `noopCheck` | 1173 | non-test profile; `fresh: true` |
| `noop-2` | `noopCheck` | 1126 | non-test profile; `fresh: true` |
| `noop-3` | `noopCheck` | 1135 | non-test profile; `fresh: true` |
| `retained-1` | `invalidatedCheck` | 5199 | non-test profile; `fresh: false` |
| `retained-1` | `noRun` | 14175 | test profile; `fresh: false`; canonical executable |
| `retained-1` | `directBinary` | 17 | exact test: 1 passed, 0 failed, 0 ignored |
| `retained-1` | `endToEnd` | 12923 | `compiledExtractum: true`; executable mtime increased; exact test: 1 passed, 0 failed, 0 ignored |
| `retained-2` | `endToEnd` | 11907 | `compiledExtractum: true`; executable mtime increased; exact test: 1 passed, 0 failed, 0 ignored |
| `retained-2` | `noRun` | 12396 | test profile; `fresh: false`; canonical executable |
| `retained-2` | `directBinary` | 17 | exact test: 1 passed, 0 failed, 0 ignored |
| `retained-2` | `invalidatedCheck` | 3828 | non-test profile; `fresh: false` |
| `retained-3` | `noRun` | 10314 | test profile; `fresh: false`; canonical executable |
| `retained-3` | `directBinary` | 18 | exact test: 1 passed, 0 failed, 0 ignored |
| `retained-3` | `invalidatedCheck` | 3684 | non-test profile; `fresh: false` |
| `retained-3` | `endToEnd` | 12326 | `compiledExtractum: true`; executable mtime increased; exact test: 1 passed, 0 failed, 0 ignored |

Proof counts from the authoritative report:

| Requirement | Count |
| --- | --- |
| Present rows / warm-ups / retained | 20 / 5 / 15 |
| Valid rows | 20 / 20 |
| Warm-up no-op actual freshness boolean | 1 / 1; `false` |
| Retained no-op `fresh: true` | 3 / 3 |
| Invalidated-check `fresh: false` | 4 / 4 |
| No-run `fresh: false`, test profile, canonical executable | 4 / 4 |
| Direct executable exact-test proofs | 4 / 4; each 1 passed, 0 failed, 0 ignored |
| End-to-end compile, mtime, exact-test proofs | 4 / 4; each `compiledExtractum: true`, mtime increased, and 1 passed / 0 failed / 0 ignored |
| Per-row source restoration | 20 / 20 |

## Rust Decomposition

Medians are calculated from the three retained observations for each shape.

| Metric | Retained values (ms) | Median (ms) |
| --- | --- | ---: |
| check floor (`invalidatedCheck`) | 5199, 3828, 3684 | 3828 |
| combined test build (`noRun`) | 14175, 12396, 10314 | 12396 |
| test build over check | 8976, 8568, 6630 | 8568 |
| direct binary (`directBinary`) | 17, 17, 18 | 17 |
| Cargo end-to-end (`endToEnd`) | 12923, 11907, 12326 | 12326 |

The complete delta-confounder statement from the authoritative report is:

> The test-build-over-check delta includes cfg(test), root test code, dev-dependencies, app-test-support, different compiler units, code generation, link, and cache/process noise; it is not pure link time.

Consequently this evidence does not make a pure-link claim.

## Rust Fast-versus-Slow Decision

**`BOUNDED_FAST_OWNER_PLAUSIBLE`**: the measured root `extractum`
`endToEnd` command shape, `cargo test --manifest-path src-tauri/Cargo.toml -p
extractum --lib readiness::tests::mark_failed_returns_failed_state -- --exact`,
has a retained median of 12326 ms, at most 13000 ms. Carry that command shape
into Slice 4 planning. The remaining two seconds are reserved for
selector/reporting/cleanup overhead, and Slice 4's real feedback smoke remains
authoritative.

## Restoration and Verification

The diagnostic restored `src-tauri/src/readiness.rs` after every sample. Its
original and restored SHA-256 values are both
`11fa4691aa866b0ef2de1acf112a97d58165da3fff4121b7cdd71fbd34c7e135`; its
original and restored length are both 3455 bytes; report-level
`restoration.verified` is `true`.

Focused verification was run from restored source:

```powershell
node scripts/run-vitest.mjs run scripts/testing/timing-log.test.ts scripts/testing/run-observation.test.ts scripts/testing/slice-1-baseline.test.ts scripts/testing/slice-1-rust-feasibility.test.ts
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

The script suite passed 4 files and 37 tests. The focused Cargo check finished
successfully; it emitted the local incremental-cache and unused-code warnings
described above. The timing JSONL schema validation, focused script suite, and
focused Cargo check are the local verification evidence for this record.
