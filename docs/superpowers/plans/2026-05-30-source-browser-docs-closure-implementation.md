# Source Browser Docs Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the shipped Source Browser documentation loop after manual smoke verification.

**Architecture:** Keep this as a docs-only slice. Record the fresh Tauri smoke result in tracked verification notes, update shipped Source Browser specs from `pending merge` to merged status, and make the active specs index stop presenting shipped Source Browser work as active.

**Tech Stack:** Markdown documentation, Superpowers plans/specs/verification folders, Git.

---

## Execution Protocol

- Start from branch `source-browser-docs-closure`.
- Keep edits docs-only.
- After each task:
  - mark completed checkboxes in this plan;
  - run the task verification;
  - commit exactly the files listed by that task.
- Do not archive or move specs/plans in this slice.

## File Map

- `docs/superpowers/verification/2026-05-30-source-browser-smoke.md`: fresh manual Source Browser smoke record.
- `docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md`: shipped status.
- `docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md`: shipped status.
- `docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md`: shipped status.
- `docs/superpowers/specs/README.md`: active/shipped Source Browser spec index cleanup.
- `docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md`: checkbox tracking.

## Task 0: Plan The Docs Closure

**Files:**
- Create: `docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md`

- [x] **Step 1: Confirm branch and clean tree**

Run:

```bash
git status --short --branch
```

Expected: branch is `source-browser-docs-closure`; only this new plan is untracked or modified.

- [x] **Step 2: Inspect current Source Browser doc statuses**

Run:

```bash
rg -n "pending merge|Status:|Source Browser|source-browser" docs/superpowers/specs docs/superpowers/specs/README.md
```

Expected findings:

```text
The legacy wrapper cleanup, data prop consolidation, and explicit subject contract specs still say pending merge.
The specs README still lists explicit subject and source group specs as active.
```

- [x] **Step 3: Commit the plan**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md
git commit -m "docs: plan source browser docs closure"
```

## Task 1: Record Source Browser Smoke

**Files:**
- Create: `docs/superpowers/verification/2026-05-30-source-browser-smoke.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md`

- [ ] **Step 1: Add the smoke verification note**

Create `docs/superpowers/verification/2026-05-30-source-browser-smoke.md` with this content:

```markdown
# Source Browser Manual Smoke

> Date: 2026-05-30
> Branch: `main`
> Scope: canonical Source Browser surfaces after explicit subject contract cleanup.

## Setup

The Tauri dev app was started from `main` with:

```bash
npm.cmd run tauri dev
```

Fixtures were reset through the MCP bridge:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

Fixture summary:

```text
accounts: 1
chatMessages: 2
llmProfiles: 1
promptTemplates: 1
runs: 6
snapshotMessages: 4
sourceGroups: 1
sources: 4
youtubePlaylistItems: 2
youtubeTranscriptSegments: 3
```

## Results

| Surface | Result | Evidence |
| --- | --- | --- |
| Telegram live source | PASS | `Timeline | Items | Metadata | Activity`; `Timeline` selected by default. |
| YouTube video live source | PASS | `Transcript | Comments | Items | Metadata | Activity`; `Transcript` selected by default. |
| YouTube playlist live source | PASS | `Videos | Items | Metadata | Activity`; `Videos` selected by default. |
| Live source group | PASS | `Sources | Items | Metadata | Activity`; `Sources` selected by default. |
| Run snapshot | PASS | Header showed `Run snapshot` and `View live source`; tabs were `Sources | Items | Metadata`; no `Activity` tab. |

## Additional Checks

- The run snapshot `View live source` action transitioned back to the live group Source Browser.
- Webview console logs contained only MCP bridge info lines.
- The Tauri dev process was stopped after the smoke.
- Final working tree was clean on `main`.

## Notes

The expected YouTube video live-source tab order is:

```text
Transcript | Comments | Items | Metadata | Activity
```
```

- [ ] **Step 2: Run note verification**

Run:

```bash
rg -n "Transcript \\| Comments \\| Items \\| Metadata \\| Activity|Run snapshot|View live source|no `Activity`" docs/superpowers/verification/2026-05-30-source-browser-smoke.md
git diff --check
```

Expected: the smoke note contains the verified tab order and snapshot header/action checks; no whitespace errors.

- [ ] **Step 3: Commit the smoke note**

Run:

```bash
git add docs/superpowers/verification/2026-05-30-source-browser-smoke.md docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md
git commit -m "docs: record source browser smoke"
```

## Task 2: Mark Shipped Source Browser Specs

**Files:**
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md`
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md`
- Modify: `docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md`

- [ ] **Step 1: Update shipped spec statuses**

In each of these specs:

```text
docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md
docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md
docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md
```

replace:

```markdown
> Status: implemented on 2026-05-30; pending merge
```

with:

```markdown
> Status: merged into main on 2026-05-30
```

- [ ] **Step 2: Verify no active spec status still says pending merge**

Run:

```bash
rg -n "^> Status: implemented on 2026-05-30; pending merge" docs/superpowers/specs
git diff --check
```

Expected: `rg` returns no matches; `git diff --check` has no whitespace errors.

- [ ] **Step 3: Commit the status update**

Run:

```bash
git add docs/superpowers/specs/2026-05-30-source-browser-legacy-wrapper-cleanup-design.md docs/superpowers/specs/2026-05-30-source-browser-data-prop-consolidation-design.md docs/superpowers/specs/2026-05-30-source-browser-explicit-subject-contract-design.md docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md
git commit -m "docs: mark source browser specs merged"
```

## Task 3: Clean Active Specs Index

**Files:**
- Modify: `docs/superpowers/specs/README.md`
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md`

- [ ] **Step 1: Replace the active Source Browser spec list**

Replace the `Active specs:` section in `docs/superpowers/specs/README.md` with:

```markdown
Active specs:

- None currently.

Shipped Source Browser specs that remain here as architecture references:

- `2026-05-30-source-group-source-browser-design.md`: merged into `main` on
  2026-05-30.
- `2026-05-30-run-snapshot-source-browser-design.md`: merged into `main` on
  2026-05-30.
- `2026-05-30-source-browser-legacy-wrapper-cleanup-design.md`: merged into
  `main` on 2026-05-30.
- `2026-05-30-source-browser-data-prop-consolidation-design.md`: merged into
  `main` on 2026-05-30.
- `2026-05-30-source-browser-explicit-subject-contract-design.md`: merged into
  `main` on 2026-05-30.
```

Leave the surrounding archive guidance intact.

- [ ] **Step 2: Verify active README no longer presents shipped work as active**

Run:

```bash
rg -n "Active specs:|None currently|pending merge|explicit-subject|source-group-source-browser|run-snapshot-source-browser|data-prop|legacy-wrapper" docs/superpowers/specs/README.md
git diff --check
```

Expected: README says `None currently`, lists shipped Source Browser specs as architecture references, and contains no `pending merge`.

- [ ] **Step 3: Commit the README cleanup**

Run:

```bash
git add docs/superpowers/specs/README.md docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md
git commit -m "docs: clean active source browser specs index"
```

## Task 4: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md`

- [ ] **Step 1: Verify shipped Source Browser docs are consistent**

Run:

```bash
rg -n "pending merge|Transcript \\| Items \\| Metadata \\| Comments|Source Browser Manual Smoke|None currently" docs/superpowers/specs docs/superpowers/verification docs/superpowers/specs/README.md
git diff --check
git status --short --branch
```

Expected:

```text
No stale YouTube tab order appears.
No active spec status says pending merge.
The smoke verification note exists.
The specs README says None currently.
Only this plan file is modified before the final commit.
```

- [ ] **Step 2: Commit final plan checkbox updates**

Run:

```bash
git add docs/superpowers/plans/2026-05-30-source-browser-docs-closure-implementation.md
git commit -m "docs: mark source browser docs closure verified"
```
