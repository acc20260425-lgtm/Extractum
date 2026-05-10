# Analysis Result-First Redesign Verification

Date: 2026-05-10
Scope: `/analysis` result-first redesign, Parts 1-7

## Automated Verification

| Check | Command | Result | Notes |
| --- | --- | --- | --- |
| Final workflow scenarios | `npm.cmd test -- src/lib/analysis-redesign-workflow-scenarios.test.ts` | PASS | Covered by combined Part 7 run: 3 files, 23 tests |
| Final route contract | `npm.cmd test -- src/lib/analysis-redesign-route-contract.test.ts` | PASS | Covered by combined Part 7 run: 3 files, 23 tests |
| Final safety contract | `npm.cmd test -- src/lib/analysis-redesign-safety-contract.test.ts` | PASS | Covered by combined Part 7 run: 3 files, 23 tests |
| Focused redesign frontend tests | `npm.cmd test -- src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-route-workspace-state.test.ts src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-access-placement.test.ts src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-canvas-route.test.ts src/lib/source-reader-model.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/analysis-run-companion-state.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-run-companion-route.test.ts src/lib/analysis-redesign-workflow-scenarios.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts` | PASS | 17 files, 108 tests |
| Svelte and TypeScript | `npm.cmd run check` | PASS | 0 errors, 0 warnings |
| Full frontend suite | `npm.cmd test` | PASS | 50 files, 348 tests |
| Focused backend safety tests | Split `cargo test --manifest-path src-tauri/Cargo.toml <filter>` commands for the Task 5 backend filters | PASS | 7 tests across chat, corpus, YouTube transcript reader, and source item query filters |
| Full backend suite | `cargo test --manifest-path src-tauri/Cargo.toml` | PASS | 298 tests |
| Boundary searches | Windows-safe `rg` equivalents for Task 5 Step 7 | PASS | Active route/components clear; legacy inactive files and absence-assertion tests may still contain marker strings |
| Whitespace | `git diff --check` | PASS | Exit 0; CRLF warning only |

## Browser Smoke Verification

Run a dev server:

```powershell
npm.cmd run dev -- --host 127.0.0.1 --port 5173
```

Then verify `http://127.0.0.1:5173/analysis` in desktop, narrow desktop, and mobile-width viewports.

| Viewport | Result | Notes |
| --- | --- | --- |
| Desktop 1440x900 | Pending browser execution | Check three-zone layout, canvas dominance, no overlap |
| Wide desktop 1920x1080 | Pending browser execution | Check report readability and companion width |
| Narrow 1180x900 | Pending browser execution | Check companion fallback and compact rail usability |
| Mobile 390x844 | Pending browser execution | Check stacked canvas/companion and source switching access |

## Browser Scenarios

| Scenario | Result | Notes |
| --- | --- | --- |
| No source state shows central onboarding for Telegram and YouTube | Pending browser execution | No global-sidebar duplication in compact rail |
| Selecting source clears opened run and shows live source + Runs | Pending browser execution | Rail selection, canvas, companion aligned |
| Completed saved run opens Report + Evidence and aligns rail if live scope exists | Pending browser execution | Header metadata visible |
| Completed saved run with missing snapshot does not resolve evidence/chat against live source | Pending browser execution | Source mode shows unavailable snapshot state |
| Running run opens Report and Source shows pending snapshot | Pending browser execution | Chat disabled until completion |
| Failed/cancelled run shows snapshot if available, otherwise explicit live source option | Pending browser execution | Not visually styled as completed |
| Trace ref click activates Evidence | Pending browser execution | Focus/selection visible |
| Show in source prefers run snapshot and highlights message/segment | Pending browser execution | Live source clearly labeled when allowed for non-completed states |
| Chat tab activates only on explicit tab selection or question submit | Pending browser execution | Textarea focus alone does not switch tab |
| Runs search/status/scope filters work and exclude source ingest jobs | Pending browser execution | Current-scope filter updates after workspace switch |
| Telegram timeline shows groups, metadata, and media placeholders only | Pending browser execution | No binary previews |
| YouTube video reader shows transcript timestamps and copy/open actions | Pending browser execution | No embedded player |
| YouTube playlist reader shows playlist item list before transcript reading | Pending browser execution | Per-video source navigation reachable |
| Source group reader groups by source with counts | Pending browser execution | No pseudo-chat merge |
| Workspace persistence restores source/group and UI context without opening a run | Pending browser execution | Run-bound tabs normalize to Runs |

## Residual Risks

- Browser scenarios depend on local data fixtures. Record any missing fixture as a verification gap with the smallest reproducible setup.
- Raw-source tests intentionally protect architectural contracts. If they fail because implementation names changed, update the assertion string while preserving the tested behavior.
