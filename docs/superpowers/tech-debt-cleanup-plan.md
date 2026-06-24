# Tech-Debt Cleanup Plan — Extractum

> Status: **APPROVED. The §3 cleanup itself (dead-code/dedup/barrel) is NOT started.** Several *separate* audit bugs (cancellation/leaks, listener guards, sidecar reap) were already fixed by direct commits since approval and are no longer tracked here.
> Created: 2026-06-24. Last updated: 2026-06-24. Mirror of plan file `~/.claude/plans/jazzy-dreaming-rossum.md`, enriched with the full audit context so this document alone can restore the working context.

---

## 0. How we got here (session context)

1. User asked to analyze the project → produced a project overview (Tauri 2 + SvelteKit/Svelte 5 desktop app for local-first ingest & LLM analysis of Telegram + YouTube).
2. User chose **"Найти проблемы"** → ran a 3-dimension read-only audit (security / Rust correctness / frontend & tech-debt).
3. User chose **"Уборка тех-долга"** → scoped a low-risk cleanup. Confirmed retiring legacy `/analysis` is **not feasible now**.
4. Plan approved via ExitPlanMode. This file is the recoverable record.

---

## 1. Audit findings — still open (reference)

These are NOT all in scope for this cleanup — recorded so the context is complete. Only the tech-debt items (§3) are being executed under THIS plan.

### 🔴 Critical / High (open)
- **#1 Legacy `/analysis` is the default un-migrated UI** — `src/routes/+layout.svelte:18` defaults `uiMode="legacy"`; `/` → `/analysis` (3365-line monolith). Three parallel UI layers coexist. (Out of scope — see §2.)
- **#2 Silent swallow of LLM-profile load** — `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte:109`, `ProjectsShell.svelte` (~104). `catch` only `console.error`; profile dropdown silently empty.
- **#3 Custom form primitives lack accessible-name / aria-live** — `src/lib/components/ui/select/Select.svelte`, `ui/Textarea.svelte`, `ui/StatusMessage.svelte` (inherited by ~9 screens). Correct pattern already exists in projects-mode settings.

### 🟡 Medium (backend, open)
- Migration 0002 FK rebuild risk — `src-tauri/migrations/0002_migrated_history_opt_in_schema.sql` (no `legacy_alter_table=ON`/`foreign_keys=OFF`).
- Idempotency check not atomic — `src-tauri/src/prompt_packs/runtime.rs:204` (SELECT-then-INSERT outside txn).

### 🟡 Medium (frontend, open)
- Duplicate routes `projects/+page.svelte` vs `projects/list/+page.svelte` (→ in scope: §3.5).
- `$state<any>` for SVAR grid API — `DataGrid.svelte:46`, `TreeDataGrid.svelte`.
- 10 dead vendored shadcn subtrees (→ in scope: §3.1/§3.3).

### 🟢 Verified-good
- Security: secrets in OS keyring + `secrecy::SecretString`; Telegram sessions XChaCha20-Poly1305 with per-write nonce; diagnostics redaction with regression tests. No high-sev vulns. No shell in process spawn. SQL fully parameterized. CDP endpoint validated (loopback only). Zero `{@html}`, safe markdown parser.

---

## 2. Decision: retiring `/analysis` is OUT of scope

Scoping confirmed `/projects` lacks features that live only in `/analysis`: chat/follow-up, evidence/trace navigation, live source readers, Takeout import UI, NotebookLM export, snapshot inspection, mixed-provider runs, legacy source groups. Architecture docs explicitly keep them parallel: *"Keep analysis_source_groups as legacy. Do not migrate or delete them in this MVP."* Retiring would be a 6–8 week rewrite. **Not now.**

This cleanup is deliberately low-risk: delete verified dead code, collapse a byte-identical route duplication, prune dead barrel exports. No change to `/analysis` behavior, DB, or features.

---

## 3. Scope & file changes (THE PLAN)

### 3.1 Delete fully-dead vendored shadcn subtrees
Under `src/lib/components/ui/` — zero production refs (internal-only ones imported only by other dead ones):
- `command/` (imports `input-group/` + `dialog/`; nothing imports `command/`)
- `input-group/` (only used by `command/`)
- `textarea/` (lowercase subtree — only used by `input-group/`; **NOT** the live `ui/Textarea.svelte` PascalCase wrapper — keep that)
- `dropdown-menu/`, `sonner/`, `scroll-area/`, `label/`, `tooltip/` (zero refs)

**Keep:** `ui/dialog/`, `ui/sheet/`, `ui/tabs/` — LIVE via extractum-ui wrappers.

### 3.2 Delete dead components
- `src/lib/components/extractum-ui/Select.svelte` — barrel-only, 0 consumers.
- `src/lib/components/analysis/youtube-playlist-detail.svelte` — zero refs (hits are the TS type, not component).
- `src/lib/components/analysis/youtube-source-detail.svelte` — zero refs (only a negative test assertion).
- `src/lib/components/analysis/youtube-source-activity.svelte` — test-only (`?raw` import by `analysis-source-readers.test.ts`); delete + update test (§3.6).

### 3.3 Remove `ui/select/` + `ui/separator/` subtrees
Removable **only together with** the dead `ExtractumSelect*` barrel exports (§3.4):
- `ui/select/` (used only by dead `extractum-ui/Select.svelte` + barrel)
- `ui/separator/` (used only by `ui/select/select-separator.svelte`)

### 3.4 Prune dead barrel exports — `src/lib/components/extractum-ui/index.ts`
Remove (0 prod, 0 test). Keep everything else:
- Line 3: `ExtractumSelect`
- Lines 17–23: `ExtractumSelectContent/Group/Item/Label/Trigger`
- Lines 31–37: `ExtractumSheetClose/Description/Footer/Header/Title` (keep `ExtractumSheet` default)
- Lines 39–45: `ExtractumDialogClose/Description/Footer/Header/Title` (keep `ExtractumDialog` default)
- **KEEP lines 25–29** (`ExtractumTabsContent/List/Trigger`) — used in production.

### 3.5 Collapse byte-identical route duplication
`src/routes/projects/+page.svelte` and `src/routes/projects/list/+page.svelte` are **identical except line 159** (`showRail={false}` vs `showRail={true}`). Both reachable from `+layout.svelte` nav — consolidate, don't delete.

Extract shared script+markup (~170 lines) into `src/lib/components/research-projects/ProjectsRoutePage.svelte` with a `showRail: boolean` prop. Each route becomes:
```svelte
<script lang="ts">
  import ProjectsRoutePage from "$lib/components/research-projects/ProjectsRoutePage.svelte";
</script>
<ProjectsRoutePage showRail={false} />   <!-- /projects: false, /projects/list: true -->
```
Keep using `formatAppError` (`$lib/app-error`) and `createResearchProjectsWorkflow` (`$lib/ui/research-projects-workflow`) exactly as now, just relocated.

### 3.6 Update contract tests broken by deletions
- `src/lib/analysis-source-readers.test.ts` — remove `youtube-source-activity.svelte?raw` import (line 17) + assertion (line 453).
- `src/lib/research-projects-import-boundary.test.ts` — keep `ui/sheet`, `ui/dialog`, `TreeDataGrid`, `GridSelectCell` assertions; update/remove removed `ui/select` path.
- `src/lib/research-projects-foundation-contract.test.ts` — asserts `ui/sheet` + `ui/tabs` exist (both kept; verify still passes).

---

## 4. Out of scope (deferred to separate plans)
- Retiring/migrating `/analysis` (#1) and migrating its components to `extractum-ui` primitives.
- a11y / error-handling #2–#3.
- Remaining open mediums: Migration 0002 FK rebuild risk, non-atomic idempotency check, `$state<any>` SVAR grid API.

---

## 5. Verification
1. **Static**: `npm run check` — no new errors (catches dangling imports).
2. **Tests**: vitest suite — the three §3.6 tests pass after updates.
3. **Build**: `npm run build` — static build succeeds (catches broken barrel imports).
4. **Manual smoke**: `npm run dev`; visit `/projects` (rail hidden) + `/projects/list` (rail shown) render identically; legacy `/analysis` unaffected.
5. **Grep guard** over `src/` returns no production hits:
   `ExtractumSelect|components/ui/(command|dropdown-menu|sonner|scroll-area|label|tooltip|select|separator|input-group)\b|youtube-source-detail|youtube-playlist-detail|youtube-source-activity`

---

## 6. Pre-execution reference re-check (done 2026-06-24)
Grep confirmed before any deletion:
- `ui/select` referenced only by `extractum-ui/Select.svelte:2` + barrel `index.ts:23` (both being removed). `ui/separator` only by `ui/select/select-separator.svelte`. `ui/input-group` only by `ui/command/command-input.svelte`. ✅ matches plan.
- `youtube-source-activity` referenced only by `analysis-source-readers.test.ts:17,453`. ✅
- `ExtractumSelect*` only in barrel `index.ts`. ✅
- No production references to any deletion target. Safe to proceed.
