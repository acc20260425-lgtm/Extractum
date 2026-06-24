# New Projects UI Audit: Library, Diagnostics, Settings

Date: 2026-06-24  
Capture source: existing connected Tauri MCP window, no restart, no code changes.  
Scope: top-level Library, Diagnostics, Settings screens (with shared top-level sidebar/top bar context).  
Focus: style consistency, CSS duplication candidates, UX density and information hierarchy, accessibility risks.

## 1) Evidence

1. [01-library.png](G:\Develop\Extractum\docs/audits/new-projects-ui-library-diagnostics-settings/01-library.png) - Library route (`/projects/library`).
2. [02-diagnostics.png](G:\Develop\Extractum\docs/audits/new-projects-ui-library-diagnostics-settings/02-diagnostics.png) - Diagnostics route (`/diagnostics`).
3. [03-settings.png](G:\Develop\Extractum\docs/audits/new-projects-ui-library-diagnostics-settings/03-settings.png) - Settings route (`/settings`).

## 2) Screen-by-screen Notes

### 2.1 Library (`/projects/library`)

- Strengths
  - Strong structure with clear three-zone layout: filter rail, grid workspace, inspector.
  - Main table uses `ExtractumDataGrid`, and timestamps already use `dateTimeFormat: "datetime"`.
  - Resize separator has keyboard semantics (`role="separator"` and arrow-key logic).

- Style consistency issues
  - `LibraryScreen`, `LibraryWorkspace`, and `LibraryInspector` use local shell/toolbar/card rules instead of shared system primitives (`extractum-panel-shell`, `extractum-grid-frame`, `extractum-toolbar-row`).
  - Inspector metadata and command area have custom visual treatment not aligned with shared compact row/card language used in Workspace/Projects/Runs.
  - Delete action is visually similar to neutral actions and does not inherit an explicit destructive treatment variant.

- CSS duplication candidates
  - Repeated shell pattern (`border`, `border-radius`, `padding`, background) across three Library components and other screens.
  - Toolbar spacing and border rules repeat patterns already present in other top-level pages.
  - Metadata list layout (`dl` rows, label/value spacing, separators) duplicates shared ideas already used in detail views.

- UX density and hierarchy
  - Toolbar controls in workspace (Add/Edit/Delete/Refresh) are same-weight and can blur action priority.
  - Inspector content is useful but dense for one selected-item workflow; section labels could be stronger.
  - Split between selected state, metadata, and commands is functional but can feel crowded on smaller heights.

- Accessibility risks
  - Repeated command verbs should be scope-labeled for screen readers: e.g. "Run report for selected source".
  - Empty state and status labels are mostly visual; verify AT context is explicit when selection changes.

---

### 2.2 Diagnostics (`/diagnostics`)

- Strengths
  - Clear page rhythm: hero -> toolbar -> status strip -> summary cards -> detail tables.
  - Good top-level summary and explicit loading/error statuses.

- Style consistency issues
  - `status-tile` and `DiagnosticCountTable` use local card/table styles despite existing shared visual tokens and existing `SurfaceCard` ecosystem.
  - Table blocks still use custom HTML table styling with many local utility-like declarations.

- CSS duplication candidates
  - `DiagnosticCountTable` table styles duplicate patterns that can be shared with other diagnostic-like tables.
  - `.diagnostics-overview-area`, `.meta-grid`, and chip spacing can be normalized under shared spacing/layout primitives.

- UX density and hierarchy
  - Section boundaries are visually present but still dense for dense error-first scanning.
  - Toggle controls are compact but not scope-disambiguated in repeated global context.
  - Horizontal table width in some environments increases horizontal work even when open issue count is low.

- Accessibility risks
  - Repeated `Refresh` should be scope-labeled (`Refresh diagnostics`) to avoid ambiguity.
  - Some tables are technically accessible but rely on compact text; row/column orientation should be consistently obvious.

---

### 2.3 Settings (`/settings`)

- Strengths
  - Good top-level discoverability with clear tabs and profile workflows.
  - Explicit status messaging and modal form lifecycle are understandable.

- Style consistency issues
  - Heaviest divergence from the existing UI system: local tab styles, buttons, table shell, modal shell, and cards.
  - Mixed custom buttons + native buttons + icon-only controls without a single shared action language.
  - Table action affordances and destructive patterns are visually custom and not consistently mirrored against Runs/Library.

- CSS duplication candidates
  - `profiles-table` + shared action controls + modal/button/form styles all re-declare design tokens already provided by shared atoms.
  - Many repeated local variants for card header, action buttons, status banners, and pills.

- UX density and hierarchy
  - Route packs many domains with one dominant table (LLM) and mixed secondary workflows.
  - Profile actions currently mix "safe" and "destructive" actions tightly in one micro-row; this increases cognitive cost.

- Accessibility risks
  - Icon-only table actions currently require stronger labeling guarantees beyond `title`.
  - Modal and action controls need explicit scope (`Edit profile <id>`, `Delete profile <id>`) for clarity in AT.

## 3) Cross-screen findings

1. Shared visual primitives are underused across all three screens relative to existing shared page building blocks.
2. Action hierarchy is not consistent between screens: same verb appears with different visual weight and no always-reliable scope labeling.
3. CSS is fragmented at leaf level (cards, tables, controls) creating visual drift and maintenance overhead.
4. Scope-aware repeated controls guidance is not consistently applied between Library, Diagnostics, Settings.
5. Settings is the most diverged from shared primitives and should be the highest priority after low-risk accessibility work.

## 4) Implementation plan

Small, executable, commit-sized slices:

### Slice 1 - Accessibility and repeated-control labels
- Files:
  - `src/lib/components/research-projects/LibraryWorkspace.svelte`
  - `src/lib/components/research-projects/LibraryInspector.svelte`
  - `src/routes/diagnostics/+page.svelte`
  - `src/lib/components/settings/projects-settings.svelte`
- Change target:
  - add explicit scope-specific labels for repeated commands and icon actions (`aria-label`, descriptive button text where needed).
- Expected check:
  - Manual keyboard/AT flow + smoke click of all toolbar actions.
- Visual evidence:
  - refreshed `01-library.png`, `02-diagnostics.png`, `03-settings.png` and note no silent ambiguous controls.

### Slice 2 - Library surface alignment with shared shells
- Files:
  - `src/lib/components/research-projects/LibraryScreen.svelte`
  - `src/lib/components/research-projects/LibraryWorkspace.svelte`
  - `src/lib/components/research-projects/LibraryInspector.svelte`
- Change target:
  - replace local shell + toolbar + panel styles with shared `extractum-*` primitives.
- Expected check:
  - unchanged interactions: filter selection, inspector selection, commands, resize behavior.
- Visual evidence:
  - before/after `01-library.png` around toolbar, cards, and inspector spacing.

### Slice 3 - Diagnostics consistency hardening
- Files:
  - `src/routes/diagnostics/+page.svelte`
  - `src/lib/components/diagnostics/DiagnosticCountTable.svelte`
- Change target:
  - align cards with shared styling tokens and reduce duplicated table CSS.
- Expected check:
  - issue filter and totals remain stable; no row loss.
- Visual evidence:
  - `02-diagnostics.png` with cleaner tile hierarchy and consistent spacing.

### Slice 4 - Settings unification pass
- Files:
  - `src/lib/components/settings/projects-settings.svelte`
- Change target:
  - map local controls to shared action/button/table patterns gradually; keep behavior unchanged.
  - keep delete actions visually destructive and semantically explicit.
- Expected check:
  - flows: add/edit/delete profile, set active, save sync settings, open modal.
- Visual evidence:
  - `03-settings.png` and modal screenshots after each sub-change.

### Slice 5 - Cross-page polish (optional, low risk)
- Files:
  - touched files from slices 1..4.
- Change target:
  - add scope-level affordance for action clusters (header toolbars, panel actions, destructive labels).
- Expected check:
  - local navigation remains stable and no unexpected visual regressions.
- Visual evidence:
  - optional short capture set for action labels.

## 5) Change visibility

- Nearly invisible:
  - explicit labels and minor accessibility attribute updates.
- Noticeable:
  - shared primitive migration in Library/Diagnostics/Settings and action styling normalization.

## 6) Constraints and baseline

- No code was changed while producing this audit.
- Existing `ExtractumDataGrid` timestamp behavior in Library is already aligned with existing shared practice (`dateTimeFormat` usage).
- Any planned Settings normalization should be staged carefully due to scope and high local density.
