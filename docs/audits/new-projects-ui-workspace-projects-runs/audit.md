# New Projects UI Audit: Workspace, Projects, Runs

Date: 2026-06-24  
Capture source: running Tauri MCP window, no app restart, no code changes.  
Scope: Workspace, Projects list, global Project Runs screen, Workspace Runs tab.  
Focus: style consistency, CSS duplication candidates, UX density, accessibility risks.

## Evidence

1. `01-current-window.png` вЂ” Workspace / Sources tab.
2. `02-projects-list.png` вЂ” Projects list route.
3. `03-project-runs.png` вЂ” global Project Runs route with selected run result.
4. `04-workspace-runs-tab.png` вЂ” Workspace / Runs tab.

## Step Notes

### 1. Workspace / Sources

Evidence: `01-current-window.png`

Health: usable, but visually dense.

Strengths:
- The screen exposes a useful end-to-end project work surface: project tree, run controls, sources table, queue status, and inspector.
- The source table now uses full type labels such as `YouTube / Video`, which is more readable than provider-only labels.
- The fixed inspector makes project-level actions discoverable.

Issues:
- The page has three strong command zones at once: top command bar, workspace action row, and inspector action stack. All compete visually.
- The left project tree, center workspace header, and right inspector repeat the same selected project name, which makes the screen feel heavier than the actual task requires.
- Primary blue is overused for different action meanings: `Run`, `Connect from Library`, `Run project analysis`, `Edit project`, and `Delete project` all read as similarly prominent.
- The source table title column appears clipped in the captured state (`T.` header and first cell truncation), so the first column needs a stronger minimum width or grid sizing rule.

Accessibility risks:
- The grid has no visible or DOM-reported `aria-label`; screen-reader users may hear a generic grid without context.
- The disabled `Sync all` action depends on a title tooltip for the reason. Tooltips are not reliably available to keyboard and touch users.
- The active project and selected navigation states lean heavily on blue color and subtle fills; verify focus and selected states with keyboard navigation.

### 2. Projects List

Evidence: `02-projects-list.png`

Health: clear list, but style hierarchy is inconsistent.

Strengths:
- Search and `Create project` are easy to find.
- Project cards summarize source/material counts directly.
- The selected project remains connected to the central workspace and inspector.

Issues:
- Two projects (`Project 2`, `Проект 3`) appear as full blue cards, while the selected project appears pale. This reverses the expected emphasis: unselected items look more primary than the active item.
- The Projects route still renders the full Workspace center and inspector. As a result, `Projects` feels like an alternate sidebar mode rather than a distinct page.
- The left list uses card-style project rows while the Workspace route uses tree rows; both represent the same entity but with different visual systems.
- `Create project` is a full primary button above the list, while edit/delete are hidden icon affordances in other project navigation contexts. The action model changes by page.

Accessibility risks:
- Icon-only edit/delete controls in project navigation should have explicit accessible names, not just `title`.
- Active state should not rely only on fill color; add a stronger text/icon state or selected marker.

### 3. Global Project Runs

Evidence: `03-project-runs.png`

Health: powerful, but high cognitive load.

Strengths:
- The table plus selected-run detail creates a useful inspector workflow for completed Prompt Pack runs.
- Result, progress, diagnostics, and artifact tabs are visible in the same screen.
- Status chips and stage timeline make the run state scannable.

Issues:
- The screen mixes a management table, editable run label controls, result preview, and diagnostics in one viewport. It is efficient for power users but dense for routine review.
- There are multiple refresh/update/delete controls in the first viewport, and their scope is not visually obvious at a glance.
- The result preview and diagnostics panel use many nested bordered panels and chips, which makes the page feel busier than the Workspace screens.
- Horizontal and vertical density is tight: the table, long date columns, and report workspace all fight for space.

Accessibility risks:
- Nested scroll areas are visible in the runs table and page. Keyboard users can get trapped or lose context if scroll focus is not clear.
- The selected run row uses a slim blue left indicator and subtle row fill. Verify it has a non-color cue and proper `aria-selected`.
- Some small chips (`Gemini Browser`, `complete`, `status ok`, artifact names) may be hard to parse at high zoom.

### 4. Workspace / Runs Tab

Evidence: `04-workspace-runs-tab.png`

Health: useful summary, but naming and action scope need tightening.

Strengths:
- Separating `Project runs` and `Prompt Pack runs` helps explain the two run systems.
- Empty state for project runs is plain and clear.
- Recent Prompt Pack runs are easy to scan, with `View result` as the primary action.

Issues:
- There are two `Refresh` buttons close together: one for project runs and one for prompt-pack runs. Their scope is only clear after reading section labels.
- `Project runs 0` and `Prompt Pack runs 6` sit in the same tab but represent different concepts; this is correct technically, but the UI needs stronger grouping.
- The prompt-pack run cards repeat the same status chips and delete affordance that also appear in the global Runs screen, but with a different layout.
- `Recent runs` in the inspector says `No project runs`, while the center tab shows six prompt-pack runs. This can feel contradictory unless the user already understands the run taxonomy.

Accessibility risks:
- Icon-only delete buttons have accessible labels in the DOM inventory for prompt-pack runs, which is good. Keep this pattern consistent with project edit/delete controls.
- Repeated `Refresh` buttons should have scope-specific accessible names such as `Refresh project runs` and `Refresh prompt pack runs`.

## Cross-Screen Findings

### Style Consistency

1. Primary blue is carrying too many meanings.
   - Evidence: `01-current-window.png`, `02-projects-list.png`, `04-workspace-runs-tab.png`.
   - Blue represents selected navigation, selected project rows, primary actions, secondary actions, edit, delete, and status chips.
   - Recommendation: reserve solid primary blue for the one main action per region. Use outline/neutral for edit, danger for delete, and lighter status tokens for chips.

2. Project navigation has three visual languages.
   - Evidence: `01-current-window.png`, `02-projects-list.png`.
   - Workspace tree rows, Projects card rows, and top/global nav icons all use different selected-state patterns.
   - Recommendation: define one project-row component or token set for selected, hover, active, and count metadata states.

3. The same page title hierarchy repeats in several zones.
   - Evidence: `01-current-window.png`.
   - `Research projects`, `Workspace`, `Research Projects`, `Project workspace`, and the selected project title all appear above the fold.
   - Recommendation: reduce one layer of headings on Workspace. Keep global space title, then project title, then tabs.

4. Mixed Russian and English labels are visible in the same operational surface.
   - Evidence: all screenshots.
   - Examples: `Workspace`, `Sources`, `Evidence`, `Create project`, alongside `Проекты`, `Категории`, `В процессе`.
   - Recommendation: choose one locale per surface or add an explicit localization pass.

### CSS Duplication Candidates

Observed via source inspection, not from screenshots alone.

1. Panel/card shell recipe is repeated.
   - Pattern: `border: 1px solid var(--extractum-border)`, `border-radius: var(--extractum-radius)`, `background: var(--extractum-surface-raised)`, `padding: 12px`.
   - Seen in: `src/lib/components/research-projects/ProjectInspector.svelte`, `src/lib/components/research-projects/ProjectWorkspace.svelte`, `src/lib/components/research-projects/ProjectRunsTab.svelte`, `src/lib/components/research-projects/ConnectFromLibrary.svelte`, `src/lib/components/research-projects/ProjectSourceSummary.svelte`, `src/lib/components/research-projects/ProjectRunReportPanel.svelte`, `src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte`.
   - Candidate: shared `.extractum-panel` / `.extractum-card` utility or component-level shell.

2. Tab styling is locally overridden in multiple places.
   - Seen in: `src/lib/components/research-projects/ProjectWorkspace.svelte` workspace tabs and `src/lib/components/research-projects/ProjectRunReportPanel.svelte` sidebar tabs.
   - Candidate: shared tab variants for line tabs, sidebar tabs, and compact result tabs.

3. Run/result cards are duplicated across Workspace Runs and global Runs.
   - Seen in: `src/lib/components/research-projects/ProjectRunsTab.svelte`, `src/lib/components/research-projects/ProjectRunsScreen.svelte`, `src/lib/components/research-projects/ProjectRunReportPanel.svelte`, `src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte`.
   - Candidate: shared `RunStatusChip`, `RunCard`, and `RunActions` primitives.

4. Project row/list styling is split.
   - Seen in: `src/lib/components/research-projects/ProjectsShell.svelte`, `src/lib/components/research-projects/ProjectRail.svelte`, Projects list route state.
   - Candidate: one `ProjectListItem` style recipe with tree and card density variants.

### UX Density

1. Workspace tries to be navigation, command center, inspector, and data table at once.
   - Keep it for power use, but reduce redundant project labels and secondary actions.

2. Projects route should decide whether it is a project picker or a full workspace mode.
   - Current state is both: a project list rail plus the same central workspace and inspector.

3. Runs route has the richest data, but it needs clearer visual separation between run selection and run result.
   - The current table/detail split is useful; stronger section boundaries and fewer visible controls would reduce scan cost.

4. Workspace Runs tab should make the difference between `Project runs` and `Prompt Pack runs` explicit.
   - The current wording is accurate but easy to misread because the inspector says no recent project runs while prompt-pack runs are visible.

### Accessibility Risks

1. Some icon-only controls rely on `title` or visual icon meaning.
   - Project edit/delete buttons need explicit `aria-label` consistency.

2. Grids should have accessible labels.
   - DOM inventory reported a grid with `label: null` on Workspace.

3. Disabled actions need accessible explanations.
   - `Sync all` and `Export` expose reasons through title/description patterns; verify keyboard and screen-reader behavior.

4. Repeated same-name controls need scoped names.
   - Multiple `Refresh` buttons appear in Workspace Runs. Use section-specific accessible labels.

5. Color is doing too much semantic work.
   - Especially for selected project rows, primary actions, status chips, and destructive actions.

## Recommended Fix Order

1. Normalize action hierarchy.
   - Make destructive actions visually destructive.
   - Reduce secondary actions from solid blue.

2. Add accessible labels to repeated/icon controls and grids.
   - Quick win with low visual risk.

3. Extract panel/card and run chip styles.
   - Good first CSS cleanup because repeated patterns are obvious and local.

4. Clarify Workspace Runs taxonomy.
   - Rename or annotate `Project runs` vs `Prompt Pack runs`, and scope refresh buttons.

5. Decide the Projects route role.
   - Either make it a focused project management list, or make it an alternate navigation rail inside Workspace. Today it reads as both.

## Limits

- This audit is based on screenshots, DOM snapshots/inventory, and light source inspection only.
- I did not run automated contrast checks, keyboard-only traversal, screen-reader testing, or responsive viewport sweeps.
- I did not restart the app, change code, or mutate app data.
