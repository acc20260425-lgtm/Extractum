# Analysis Redesign UX Polish Pass

Date: 2026-05-11
Scope: `/analysis` result-first redesign, post-merge UX pass with real Tauri app and seeded debug fixtures

## Context

This pass was intentionally exploratory rather than a pass/fail verification run. The goal was to open the real desktop app, seed the analysis redesign fixtures, and inspect `/analysis` as a user would: layout density, empty states, source/report/evidence transitions, and rough interaction edges.

The app was launched through Tauri dev mode:

```powershell
npm.cmd run tauri dev
```

The Tauri MCP bridge connected to the running debug app:

```text
identifier: org.ai.extractum
name: extractum
version: 0.1.0
debug: true
window: main
```

Fixtures were reset and seeded through the real Tauri webview bridge:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

Seed result:

```json
{
  "accounts": 1,
  "chatMessages": 2,
  "llmProfiles": 1,
  "promptTemplates": 1,
  "runs": 6,
  "snapshotMessages": 4,
  "sourceGroups": 1,
  "sources": 4,
  "youtubePlaylistItems": 2,
  "youtubeTranscriptSegments": 3
}
```

Runtime logs showed no app-side errors during the pass. `git status --short --branch` remained clean:

```text
## main
```

## Artifacts

Screenshots from this pass are saved under `artifacts/`:

```text
artifacts/analysis-ux-desktop-initial.png
artifacts/analysis-ux-desktop-companion-runs.png
artifacts/analysis-ux-desktop-run-list.png
artifacts/analysis-ux-desktop-completed-run-card.png
artifacts/analysis-ux-desktop-completed-report-top.png
artifacts/analysis-ux-desktop-completed-source.png
artifacts/analysis-ux-desktop-chat-panel.png
artifacts/analysis-ux-desktop-missing-snapshot-report.png
artifacts/analysis-ux-desktop-missing-snapshot-source.png
artifacts/analysis-ux-desktop-failed-report.png
artifacts/analysis-ux-desktop-failed-source.png
artifacts/analysis-ux-desktop-source-switcher.png
artifacts/analysis-ux-desktop-source-switcher-search.png
artifacts/analysis-ux-desktop-source-switcher-group-search.png
artifacts/analysis-ux-desktop-live-youtube.png
artifacts/analysis-ux-desktop-live-group.png
artifacts/analysis-ux-desktop-live-group-report-setup.png
artifacts/analysis-ux-narrow-group-report-setup.png
artifacts/analysis-ux-mobile-group-report-setup-top.png
artifacts/analysis-ux-mobile-group-report-setup-form.png
```

## Scenarios Covered

| Scenario | Observation |
| --- | --- |
| Initial `/analysis` after fixture seed | App opened the active running fixture run and kept the selected source rail aligned. |
| Running run report | Metadata, status, cancel action, live report empty state, and Runs companion were reachable. |
| All runs list | Active and saved fixture runs appeared after switching to `All runs` and clearing status filters. |
| Completed snapshot run | Report, Evidence, Chat, Source mode, snapshot transcript, and Show in source path were inspected. |
| Missing snapshot run | Report and Source mode showed unavailable snapshot states and disabled exact source resolution. |
| Failed run | Error state rendered in Report; Source mode showed unavailable snapshot plus live-source option. |
| Live YouTube source | Transcript segments, timestamp links, copy actions, and sync actions rendered. |
| Source switcher search | Source and group searches filtered fixture data and showed empty states for the other bucket. |
| Live Telegram group | Group source reader rendered all-source focus and per-source sections. |
| Report setup for group workspace | Form controls and next-step guidance rendered for a source group. |
| Narrow and mobile widths | Layout stacked without obvious horizontal overflow, but became very tall and icon-heavy. |

## UX Findings

### 1. Run Metadata Dominates The First Viewport

Severity: High

On running, completed, missing snapshot, and failed runs, the opened-run metadata grid consumes most of the first desktop viewport. The actual report output, source material, evidence, chat, and runs list often start below the fold. This makes the result-first screen feel metadata-first.

Examples:

- `artifacts/analysis-ux-desktop-initial.png`
- `artifacts/analysis-ux-desktop-completed-report-top.png`
- `artifacts/analysis-ux-desktop-failed-report.png`

Recommended direction:

- Replace the full metadata card with a compact run summary strip.
- Move detailed run metadata into a collapsible details section.
- Keep the primary artifact for the selected mode, report output or source reader, visible in the first viewport.

### 2. Snapshot Availability Copy Is Contradictory

Severity: High

Several states expose contradictory snapshot language:

- Completed Snapshot Run initially showed `Snapshot status unknown` and `Checking whether a frozen run snapshot exists.` in Report view.
- The same run showed a valid `Run snapshot` in Source mode.
- Missing/failed snapshot states showed a green `Run snapshot` badge next to `Snapshot unavailable`.

Examples:

- `artifacts/analysis-ux-desktop-completed-report-top.png`
- `artifacts/analysis-ux-desktop-completed-source.png`
- `artifacts/analysis-ux-desktop-missing-snapshot-source.png`
- `artifacts/analysis-ux-desktop-failed-source.png`

Recommended direction:

- Use one explicit source-basis status model in all modes: `Checking`, `Snapshot available`, `Snapshot unavailable`, `Live source`.
- Do not render a positive `Run snapshot` badge when the snapshot is unavailable.
- Once snapshot availability has resolved, update the Report header and metadata card as well as Source mode.

### 3. Runs Tab Is Too Heavy Before The List

Severity: High

The Runs companion tab places a large control block before the list: search, scope toggles, status filters, date inputs, provider/model/template filters, and active/saved toggles. On desktop, the actual run cards start below the first Runs viewport. On narrower widths, this cost grows.

Examples:

- `artifacts/analysis-ux-desktop-companion-runs.png`
- `artifacts/analysis-ux-desktop-run-list.png`

Recommended direction:

- Keep search, scope, and status visible.
- Collapse date/provider/model/template filters behind an `Advanced filters` disclosure.
- Consider moving Active/Saved into the same compact filter row as status or making it a small segmented control.

### 4. Source Switcher Mixes Fast Switching With Management

Severity: Medium

The source switcher works, but it feels more like a management panel than a fast context switcher:

- Source cards are tall.
- `Delete` is visually prominent beside ordinary source selection.
- The panel remains open after selecting a source or group.
- Searching a group leaves the source section empty above the group result, which is technically accurate but visually noisy.

Examples:

- `artifacts/analysis-ux-desktop-source-switcher.png`
- `artifacts/analysis-ux-desktop-source-switcher-search.png`
- `artifacts/analysis-ux-desktop-source-switcher-group-search.png`

Recommended direction:

- Separate quick switch actions from destructive management actions.
- Hide or demote `Delete` from the quick switcher path.
- Close the switcher after selecting a source/group unless the user opened explicit manage mode.
- For search results, consider grouping empty buckets more quietly.

### 5. Source Reader Headers And Controls Repeat The Same Context

Severity: Medium

Source mode repeats the selected title and basis in several nearby places:

- Workspace header.
- Source reader header.
- Specific reader title.
- Source/group focus controls.

The group reader also showed duplicate `Source focus` controls: one in the source reader header and another above the grouped timeline.

Examples:

- `artifacts/analysis-ux-desktop-live-youtube.png`
- `artifacts/analysis-ux-desktop-live-group.png`

Recommended direction:

- Keep one context title at the workspace level.
- Let the reader surface use compact subheaders such as `Transcript`, `Timeline`, or `Group sources`.
- Keep only one source focus control in group mode.

### 6. Reader Search Inputs Look Visually Broken

Severity: Medium

The YouTube transcript search control renders as a label, a separate loose search icon, and an empty input. It works structurally, but visually it reads as an unfinished input group.

Examples:

- `artifacts/analysis-ux-desktop-completed-source.png`
- `artifacts/analysis-ux-desktop-live-youtube.png`

Recommended direction:

- Put the search icon inside the input.
- Add an explicit placeholder like `Search transcript`.
- Keep label, icon, and input in one compact control.

### 7. Report Setup Empty-State Copy Is Not Run-Aware

Severity: Medium

The group report setup says `Build the first report for this workspace` even when saved fixture runs already exist for that workspace. This makes the user wonder whether history is being ignored.

Examples:

- `artifacts/analysis-ux-desktop-live-group-report-setup.png`
- `artifacts/analysis-ux-mobile-group-report-setup-form.png`

Recommended direction:

- If there are saved runs for the current scope, use `Start a new report` or `Run another report`.
- Optionally include a small link or affordance to view prior runs for the current scope.

### 8. Mobile And Narrow Layouts Are Technically Stable But Very Tall

Severity: Medium

At mobile width, the layout avoided obvious horizontal overflow, but the first screen was dominated by:

- the app header;
- an icon-only quick-source strip with little context;
- large repeated workspace titles;
- one metadata card per field.

Examples:

- `artifacts/analysis-ux-narrow-group-report-setup.png`
- `artifacts/analysis-ux-mobile-group-report-setup-top.png`
- `artifacts/analysis-ux-mobile-group-report-setup-form.png`

Recommended direction:

- Compress mobile headers and avoid repeating the selected workspace title.
- Replace the icon-only quick-source strip with a compact current-context button plus search/switch action.
- Group setup summary fields into fewer mobile rows or a compact summary block.

## Suggested Fix Order

1. Compress opened-run metadata so report/source/evidence content appears earlier.
2. Normalize snapshot availability state and copy across Report, Source, and Evidence.
3. Simplify Runs filters and collapse advanced filters.
4. Split quick source switching from destructive source management.
5. De-duplicate Source mode headers and source focus controls.
6. Polish reader search controls.
7. Make report setup empty-state copy aware of existing saved runs.
8. Tighten mobile and narrow layouts after the desktop information hierarchy is cleaner.

## Follow-Up Verification Targets

After implementing polish fixes, repeat the same fixture-backed pass and specifically verify:

- Completed Snapshot Run shows consistent snapshot-available copy in Report, Source, and Evidence.
- Missing Snapshot Run and Failed Run do not show a positive snapshot badge.
- First viewport for a saved completed run includes useful report or source content above the fold.
- Runs tab shows at least the first run card without scrolling on desktop.
- Source switcher selection closes or clearly changes mode after selecting a source/group.
- Mobile width has no title overflow and less repeated context chrome.

## Pass 1 Implementation Verification

Date: 2026-05-12
Branch: `polish/analysis-ux-pass-1`

Batch 1 fixes implemented:

- Opened-run metadata is now a compact summary strip with detailed metadata behind `Run details`.
- Snapshot copy now uses explicit states: `Checking snapshot`, `Snapshot pending`, `Snapshot available`, and `Snapshot unavailable`.
- Opened runs now probe snapshot availability before the user switches to Source mode, so Report and Evidence resolve the same snapshot state.
- Report setup copy is run-aware and shows `Run another report` when saved runs exist for the current workspace.

Automated verification:

```text
npm.cmd test
npm.cmd run check
git diff --check
```

Runtime verification used Tauri dev mode with the same fixture seed path:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

Seed result:

```json
{
  "accounts": 1,
  "chatMessages": 2,
  "llmProfiles": 1,
  "promptTemplates": 1,
  "runs": 6,
  "snapshotMessages": 4,
  "sourceGroups": 1,
  "sources": 4,
  "youtubePlaylistItems": 2,
  "youtubeTranscriptSegments": 3
}
```

New artifacts:

```text
artifacts/analysis-ux-pass-1-completed-report.png
artifacts/analysis-ux-pass-1-completed-source.png
artifacts/analysis-ux-pass-1-failed-report.png
artifacts/analysis-ux-pass-1-group-setup.png
```

Observed pass notes:

- Completed Snapshot Run showed `Snapshot available` in the Report header before switching to Source mode. Evidence enabled `Show in source`, and Source mode rendered the frozen run snapshot transcript.
- Missing Snapshot Run showed `Snapshot unavailable` and did not show a positive snapshot badge.
- Failed Run showed `Snapshot unavailable` beside the failed status and kept failure details readable.
- Group report setup showed `Run another report` instead of first-report copy when prior saved runs existed.

Residual polish remains for batch 2: Runs filter density, source switcher behavior, source reader deduplication, transcript search styling, and mobile source rail height.

## Pass 2 Implementation Verification

Date: 2026-05-12
Branch: `analysis-ux-pass-2`

Batch 2 fixes implemented:

- Runs companion now keeps search, scope, and status filters visible while date/profile/template filters live behind `Advanced filters`.
- Source switcher now closes after selecting a source or group, removes destructive `Delete` actions from quick-switch rows, and quiets empty filtered buckets.
- Source reader headers now use compact surface labels and avoid repeating the selected source title below the workspace header.
- Group source reader no longer repeats the source focus control.
- YouTube transcript search now uses one compact input shell with an inline search icon and `Search transcript` placeholder.
- Mobile source rail now shows the current context label and constrains the quick-source scroller at narrow widths.

Automated verification:

```text
npm.cmd test
npm.cmd run check
git diff --check
```

Result:

```text
50 test files passed
368 tests passed
svelte-check found 0 errors and 0 warnings
git diff --check passed with no output
```

Runtime verification used Tauri dev mode with the same fixture seed path:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

Seed result:

```json
{
  "accounts": 1,
  "chatMessages": 2,
  "llmProfiles": 1,
  "promptTemplates": 1,
  "runs": 6,
  "snapshotMessages": 4,
  "sourceGroups": 1,
  "sources": 4,
  "youtubePlaylistItems": 2,
  "youtubeTranscriptSegments": 3
}
```

New artifacts:

```text
artifacts/analysis-ux-pass-2-runs-filters.png
artifacts/analysis-ux-pass-2-source-switcher.png
artifacts/analysis-ux-pass-2-source-reader.png
artifacts/analysis-ux-pass-2-youtube-search.png
artifacts/analysis-ux-pass-2-mobile-rail.png
```

Observed pass notes:

- Runs companion rendered `Advanced filters` open on demand, keeping the main status/scope controls above the saved-run area.
- Source switcher listed fixture sources and groups without `Delete` in quick rows; selecting `__analysis_redesign_fixture__ Telegram Group` closed the switcher and changed the current context.
- Group Source mode rendered `GROUP SOURCES` without a duplicate `Source focus` control.
- Fixture YouTube source rendered transcript search with `aria-label="Search transcript"`, placeholder `Search transcript`, `.search-input-wrap`, and `.search-icon`.
- At `390x850`, the mobile rail showed `__analysis_redesign_fixture__ YouTube Video` as the current context and `documentElement.scrollWidth <= clientWidth`.

Residual polish remains outside this batch: the seeded fixture can surface a stale YouTube detail lookup warning after reseeding over prior local state, and run cards can still be hidden by persisted local filters until the user clears or changes the filter state.
