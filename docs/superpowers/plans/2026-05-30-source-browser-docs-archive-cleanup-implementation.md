# Source Browser Docs Archive Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move completed Source Browser Superpowers artifacts out of active folders and into the archive.

**Architecture:** This is a docs-only archive cleanup. Keep root current-state docs as the authority, move completed plans/specs/smoke notes into archive directories, and leave active Superpowers folders with README guidance only.

**Tech Stack:** Markdown documentation, Git moves, Superpowers docs folders.

---

## Execution Protocol

- Start from branch `source-browser-docs-archive-cleanup`.
- Use `git mv` for tracked file moves.
- Keep changes docs-only.
- After each task:
  - mark completed checkboxes in this plan;
  - run the task verification;
  - commit exactly the files listed by that task.
- Move this implementation plan itself to `docs/superpowers/archive/plans/` in the final task.

## File Map

- `docs/superpowers/plans/README.md`: active plan guidance.
- `docs/superpowers/archive/README.md`: archive directory guidance.
- `docs/superpowers/archive/plans/README.md`: new archive plan guidance.
- `docs/superpowers/specs/README.md`: active specs guidance after Source Browser specs move out.
- `docs/superpowers/archive/specs/README.md`: archived specs guidance.
- `docs/superpowers/verification/README.md`: active verification guidance.
- `docs/superpowers/archive/verification/README.md`: archived verification guidance.
- `docs/superpowers/plans/2026-05-30-*-source-browser-*.md`: completed Source Browser implementation plans to archive.
- `docs/superpowers/specs/2026-05-30-*-source-browser-*.md`: shipped Source Browser specs to archive.
- `docs/superpowers/verification/2026-05-30-source-browser-smoke.md`: historical Source Browser smoke note to archive.

## Task 0: Plan Archive Cleanup

**Files:**
- Create: `docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md`

- [x] **Step 1: Confirm branch and active artifacts**

Run:

```bash
git status --short --branch
Get-ChildItem -Path 'docs\superpowers\plans','docs\superpowers\specs','docs\superpowers\verification' -File | Select-Object FullName
```

Expected: branch is `source-browser-docs-archive-cleanup`; Source Browser plans/specs and the smoke note are still in active folders.

- [x] **Step 2: Commit this plan**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md
git commit -m "docs: plan source browser archive cleanup"
```

## Task 1: Add Plans Archive Guidance

**Files:**
- Modify: `docs/superpowers/archive/README.md`
- Create: `docs/superpowers/archive/plans/README.md`
- Modify: `docs/superpowers/plans/README.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md`

- [ ] **Step 1: Update active plans guidance**

Replace `docs/superpowers/plans/README.md` with:

```markdown
# Active Plans

Keep only in-progress implementation plans here.

Completed plans should move to `docs/superpowers/archive/plans/` when they are
still useful as execution history. Otherwise, Git history is enough.
```

- [ ] **Step 2: Add archived plans README**

Create `docs/superpowers/archive/plans/README.md` with:

```markdown
# Archived Plans

This directory stores completed Superpowers implementation plans that are still
useful as execution history.

Use these files for historical task sequencing and verification context only.
Current product and architecture state belongs in root docs such as
`docs/project.md`, `docs/design-document.md`, and
`docs/frontend-architecture-evolution-analysis.md`.

Active implementation plans belong in `docs/superpowers/plans/`.
```

- [ ] **Step 3: Update archive README**

In `docs/superpowers/archive/README.md`, replace:

```markdown
- `specs/`: designs for shipped or superseded work. See
  `specs/README.md`.
- `verification/`: historical manual verification records. See
  `verification/README.md`.

Completed implementation plans are not archived here; they are deleted from the
working tree and remain available through Git history.
```

with:

```markdown
- `plans/`: completed implementation plans that are still useful as execution
  history. See `plans/README.md`.
- `specs/`: designs for shipped or superseded work. See
  `specs/README.md`.
- `verification/`: historical manual verification records. See
  `verification/README.md`.
```

- [ ] **Step 4: Verify plan archive guidance**

Run:

```bash
rg -n "Archived Plans|archive/plans|completed implementation plans|Active Plans" docs/superpowers/plans/README.md docs/superpowers/archive/README.md docs/superpowers/archive/plans/README.md
git diff --check
```

Expected: new archive guidance is present and no whitespace errors.

- [ ] **Step 5: Commit plan archive guidance**

Run:

```bash
git add docs/superpowers/plans/README.md docs/superpowers/archive/README.md docs/superpowers/archive/plans/README.md docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md
git commit -m "docs: add archived plans guidance"
```

## Task 2: Archive Completed Source Browser Plans

**Files:**
- Move: `docs/superpowers/plans/2026-05-30-source-group-source-browser-implementation.md` to `docs/superpowers/archive/plans/2026-05-30-source-group-source-browser-implementation.md`
- Move: `docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md` to `docs/superpowers/archive/plans/2026-05-30-run-snapshot-source-browser-implementation.md`
- Move: `docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md` to `docs/superpowers/archive/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md`
- Move: `docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md` to `docs/superpowers/archive/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md`
- Move: `docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md` to `docs/superpowers/archive/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md`
- Move: `docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md` to `docs/superpowers/archive/plans/2026-05-30-source-browser-docs-closure-implementation.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md`

- [ ] **Step 1: Move completed plans**

Run:

```bash
git mv docs/superpowers/plans/2026-05-30-source-group-source-browser-implementation.md docs/superpowers/archive/plans/2026-05-30-source-group-source-browser-implementation.md
git mv docs/superpowers/plans/2026-05-30-run-snapshot-source-browser-implementation.md docs/superpowers/archive/plans/2026-05-30-run-snapshot-source-browser-implementation.md
git mv docs/superpowers/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md docs/superpowers/archive/plans/2026-05-30-source-browser-legacy-wrapper-cleanup-implementation.md
git mv docs/superpowers/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md docs/superpowers/archive/plans/2026-05-30-source-browser-data-prop-consolidation-implementation.md
git mv docs/superpowers/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md docs/superpowers/archive/plans/2026-05-30-source-browser-explicit-subject-contract-implementation.md
git mv docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md docs/superpowers/archive/plans/2026-05-30-source-browser-docs-closure-implementation.md
```

- [ ] **Step 2: Verify active plans are reduced to README plus this plan**

Run:

```bash
Get-ChildItem -Path 'docs\superpowers\plans' -File | Select-Object Name
Get-ChildItem -Path 'docs\superpowers\archive\plans' -File | Select-Object Name
git diff --check
```

Expected: active plans contain `README.md` and this archive cleanup plan only; archived plans contain the six completed Source Browser plans plus `README.md`.

- [ ] **Step 3: Commit archived completed plans**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md docs/superpowers/plans docs/superpowers/archive/plans
git commit -m "docs: archive completed source browser plans"
```

## Task 3: Archive Shipped Source Browser Specs

**Files:**
- Move: `docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md` to `docs/superpowers/archive/specs/2026-05-30-source-group-source-browser-design.md`
- Move: `docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md` to `docs/superpowers/archive/specs/2026-05-30-run-snapshot-source-browser-design.md`
- Move: `docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md` to `docs/superpowers/archive/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md`
- Move: `docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md` to `docs/superpowers/archive/specs/2026-05-30-source-browser-data-prop-consolidation-design.md`
- Move: `docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md` to `docs/superpowers/archive/specs/2026-05-30-source-browser-explicit-subject-contract-design.md`
- Modify: `docs/superpowers/specs/README.md`
- Modify: `docs/superpowers/archive/specs/README.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md`

- [ ] **Step 1: Move shipped specs**

Run:

```bash
git mv docs/superpowers/specs/2026-05-30-source-group-source-browser-design.md docs/superpowers/archive/specs/2026-05-30-source-group-source-browser-design.md
git mv docs/superpowers/specs/2026-05-30-run-snapshot-source-browser-design.md docs/superpowers/archive/specs/2026-05-30-run-snapshot-source-browser-design.md
git mv docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md docs/superpowers/archive/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md
git mv docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md docs/superpowers/archive/specs/2026-05-30-source-browser-data-prop-consolidation-design.md
git mv docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md docs/superpowers/archive/specs/2026-05-30-source-browser-explicit-subject-contract-design.md
```

- [ ] **Step 2: Simplify active specs README**

Replace `docs/superpowers/specs/README.md` with:

```markdown
# Active Specs

Keep only active or still-relevant design specs here.

Specs for shipped or superseded work should move to
`docs/superpowers/archive/specs/`.

Active specs:

- None currently.

Recently shipped or superseded specs are archived under
`docs/superpowers/archive/specs/`.
```

- [ ] **Step 3: Update archive specs README**

Append this paragraph to `docs/superpowers/archive/specs/README.md`:

```markdown
The 2026-05-29 and 2026-05-30 Source Browser specs are historical rationale for
the shipped Source Browser architecture. Current behavior is summarized in
`docs/project.md`, `docs/design-document.md`, and
`docs/frontend-architecture-evolution-analysis.md`.
```

- [ ] **Step 4: Verify active specs are reduced to README**

Run:

```bash
Get-ChildItem -Path 'docs\superpowers\specs' -File | Select-Object Name
rg -n "Source Browser specs that remain here|2026-05-30-source|2026-05-30-run-snapshot|None currently|Source Browser specs are historical" docs/superpowers/specs/README.md docs/superpowers/archive/specs/README.md docs/superpowers/archive/specs
git diff --check
```

Expected: active specs contain `README.md` only; archived specs contain the moved Source Browser specs; active README no longer lists Source Browser references.

- [ ] **Step 5: Commit archived specs**

Run:

```bash
git add docs/superpowers/specs docs/superpowers/archive/specs docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md
git commit -m "docs: archive shipped source browser specs"
```

## Task 4: Archive Source Browser Smoke Note

**Files:**
- Move: `docs/superpowers/verification/2026-05-30-source-browser-smoke.md` to `docs/superpowers/archive/verification/2026-05-30-source-browser-smoke.md`
- Modify: `docs/superpowers/verification/README.md`
- Modify: `docs/superpowers/archive/verification/README.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md`

- [ ] **Step 1: Move the smoke note**

Run:

```bash
git mv docs/superpowers/verification/2026-05-30-source-browser-smoke.md docs/superpowers/archive/verification/2026-05-30-source-browser-smoke.md
```

- [ ] **Step 2: Update active verification README**

Replace `docs/superpowers/verification/README.md` with:

```markdown
# Active Verification Notes

Keep only active or reusable verification notes here.

Historical verification records for shipped work should move to
`docs/superpowers/archive/verification/`.
```

- [ ] **Step 3: Update archive verification README**

Append this paragraph to `docs/superpowers/archive/verification/README.md`:

```markdown
Source Browser smoke notes in this directory are historical acceptance records.
Run a fresh Tauri smoke before using them as release evidence.
```

- [ ] **Step 4: Verify smoke note location**

Run:

```bash
Test-Path docs/superpowers/verification/2026-05-30-source-browser-smoke.md
Test-Path docs/superpowers/archive/verification/2026-05-30-source-browser-smoke.md
rg -n "Source Browser Manual Smoke|Source Browser smoke notes|Historical verification records" docs/superpowers/verification/README.md docs/superpowers/archive/verification/README.md docs/superpowers/archive/verification/2026-05-30-source-browser-smoke.md
git diff --check
```

Expected: first `Test-Path` is `False`, second is `True`, and archive verification guidance mentions Source Browser smoke notes.

- [ ] **Step 5: Commit archived smoke note**

Run:

```bash
git add docs/superpowers/verification docs/superpowers/archive/verification docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md
git commit -m "docs: archive source browser smoke note"
```

## Task 5: Archive This Cleanup Plan And Verify

**Files:**
- Move: `docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md` to `docs/superpowers/archive/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md`

- [ ] **Step 1: Verify active folders before archiving this plan**

Run:

```bash
Get-ChildItem -Path 'docs\superpowers\plans','docs\superpowers\specs' -File | Select-Object FullName
rg -n "2026-05-30-source-browser|2026-05-30-source-group|2026-05-30-run-snapshot|Source Browser Manual Smoke" docs/superpowers/plans docs/superpowers/specs docs/superpowers/verification
git diff --check
```

Expected: active plans contain README and this plan only; active specs contain README only; active verification no longer contains the Source Browser smoke note; `rg` only finds this plan before it is moved.

- [ ] **Step 2: Move this plan to the archive**

Run:

```bash
git mv docs/superpowers/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md docs/superpowers/archive/plans/2026-05-30-source-browser-docs-archive-cleanup-implementation.md
```

- [ ] **Step 3: Verify active folders after archiving this plan**

Run:

```bash
Get-ChildItem -Path 'docs\superpowers\plans','docs\superpowers\specs' -File | Select-Object FullName
rg -n "2026-05-30-source-browser|2026-05-30-source-group|2026-05-30-run-snapshot|Source Browser Manual Smoke" docs/superpowers/plans docs/superpowers/specs docs/superpowers/verification
git diff --check
git status --short --branch
```

Expected: active plans/specs contain only README files; active verification contains no Source Browser smoke note; `rg` returns no matches in active folders; only this move is staged or modified before the final commit.

- [ ] **Step 4: Commit final archive cleanup**

Run:

```bash
git add docs/superpowers/plans docs/superpowers/archive/plans
git commit -m "docs: archive source browser archive cleanup plan"
```
