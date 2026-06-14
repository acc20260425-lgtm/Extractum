# YouTube Summary Prompt Pack MVP Implementation Plan Index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement these plans task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the approved YouTube Summary Prompt Pack MVP design into small implementation plans that can be executed and reviewed independently.

**Architecture:** Build the new Prompt Pack stack in four layers: DB/library foundation, run runtime, LLM execution/result persistence, and UI. Each layer produces software that can be tested before the next layer depends on it.

**Tech Stack:** Rust/Tauri 2, SQLite via `tauri-plugin-sql` and `sqlx`, zstd-compressed JSON blobs, existing LLM provider backend, Svelte 5/SvelteKit, Vitest.

---

## Source Spec

- `docs/superpowers/specs/2026-06-14-youtube-summary-prompt-pack-mvp-design.md`
- `docs/prompt-packs/prompt_pack_json_contract_v1_draft.md`
- `docs/prompt-packs/youtube_summary_pack_spec.md`
- `docs/prompt-packs/stage_io_contracts.md`
- `docs/prompt-packs/validation_rules.md`

## Plan Files

1. `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-foundation.md`
   - Creates Prompt Pack DB schema, bundled pack library seed, schema asset seed, and read-only library commands.
   - Produces: migrated DB and seeded `youtube_summary` pack version.

2. `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-runtime.md`
   - Adds YouTube Summary preflight, deterministic source/material snapshots, run skeleton creation, active run state, events, and cancellation.
   - Depends on: foundation.
   - Produces: runs can be created, inspected, cancelled, and tracked without making LLM calls.

3. `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-execution-result.md`
   - Adds combined `youtube_summary/transcript_analysis` execution, stage output validation, canonical result assembly, transactional projections, and projection repair.
   - Depends on: foundation and runtime.
   - Produces: fake-provider and real-provider paths can create canonical results.

4. `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-ui.md`
   - Adds frontend API/types, active run event subscription, YouTube Summary launch UI, queue/runs display, and result viewer projections.
   - Depends on: backend commands from runtime and execution/result.
   - Produces: a usable MVP screen and Library entry point.

## Execution Order

Tracking rules:

- Mark an index step complete only after every checkbox in the child plan is
  complete, the child plan acceptance gate passes, and all child task commits
  are present.
- If a child plan acceptance gate finds a failure, leave the index step
  unchecked until the fix is committed and the gate is rerun successfully.
- Keep `git status --short` output visible at every gate. If unrelated user
  changes appear, do not include them in Prompt Pack commits.

- [x] **Step 0: Prepare implementation branch**

Run before Step 1:

```powershell
git status --short
git branch --show-current
```

Expected:

- current branch and dirty state are known;
- any dirty files are classified before work starts.

Branch policy:

- If the workspace is clean, create the implementation branch:

```powershell
git switch -c feat/youtube-summary-prompt-pack-mvp
```

- If the branch already exists, switch to it:

```powershell
git switch feat/youtube-summary-prompt-pack-mvp
```

- If the workspace has unrelated user changes, stop and document them in the
  task notes. Do not stash, commit, reset, or overwrite them without explicit
  owner approval.

Mark Step 0 complete only after the branch is selected and the dirty-state
policy is recorded in the task notes.

Task notes:

- 2026-06-14: Started from `main` with `git status --short --branch` showing a clean tree.
- 2026-06-14: `git switch -c feat/youtube-summary-prompt-pack-mvp` failed because Git could not create `refs/heads/feat/youtube-summary-prompt-pack-mvp`; selected `codex/youtube-summary-prompt-pack-mvp` instead.
- 2026-06-14: No unrelated dirty files were present before implementation work started.

- [ ] **Step 1: Execute foundation plan**

Run and commit every task from `2026-06-14-youtube-summary-mvp-foundation.md`.

Acceptance gate:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib
npm test -- --run src/lib/api/prompt-packs.test.ts
git status --short
```

- [ ] **Step 2: Execute runtime plan**

Run and commit every task from `2026-06-14-youtube-summary-mvp-runtime.md`.

Acceptance gate:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
npm test -- --run src/lib/api/prompt-packs.test.ts
git status --short
```

- [ ] **Step 3: Execute execution/result plan**

Run and commit every task from `2026-06-14-youtube-summary-mvp-execution-result.md`.

Acceptance gate:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
npm test -- --run src/lib/api/prompt-packs.test.ts
git status --short
```

- [ ] **Step 4: Execute UI plan**

Run and commit every task from `2026-06-14-youtube-summary-mvp-ui.md`.

Acceptance gate:

```powershell
npm test -- --run src/lib/api/prompt-packs.test.ts src/lib/ui/youtube-summary-workflow.test.ts src/lib/youtube-summary-launch-contract.test.ts src/lib/youtube-summary-result-view-contract.test.ts
npm test -- --run src/lib/youtube-summary-smoke-fixture-contract.test.ts
npm run check
git status --short
```

Then complete the UI browser smoke from
`2026-06-14-youtube-summary-mvp-ui.md` Task 6 Step 4, including the desktop
launch-dialog and result-view screenshots. If the local DB has no synced
YouTube source, use the child plan's dev-only smoke fixture fallback.

## Commit Cadence

Commit at the end of each task inside a plan. Do not wait until a whole plan is complete. Preferred commit scopes:

- `feat: add prompt pack schema foundation`
- `feat: seed youtube summary prompt pack`
- `feat: add youtube summary preflight`
- `feat: create prompt pack run snapshots`
- `feat: execute youtube summary combined stage`
- `feat: persist prompt pack result projections`
- `feat: add youtube summary run UI`

## Boundaries

- Do not modify legacy `analysis_runs` persistence for this MVP.
- Do not route YouTube Summary results through `analysis_run_messages`.
- Do not add URL ingest to the YouTube Summary flow.
- Do not enable automatic provider fallback.
- Do not build a prompt-pack editor UI in this MVP.

## Final Verification

After all plans are executed and before merge or final handoff:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations
cargo test --manifest-path src-tauri/Cargo.toml --lib
npm test
npm run check
git status --short
```

Expected:

- Rust tests pass.
- Vitest tests pass.
- Svelte check passes.
- Any final verification fixes are committed before handoff.
- `git status --short` is clean or contains only explicitly documented,
  intentionally uncommitted handoff changes.
