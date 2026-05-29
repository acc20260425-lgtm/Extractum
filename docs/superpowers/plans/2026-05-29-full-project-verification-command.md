# Full Project Verification Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `npm run verify` as the baseline full-project verification command.

**Architecture:** A small Node runner in `scripts/verify.mjs` owns the command list and subprocess behavior. `package.json` exposes it as `npm run verify`, while docs/backlog record the baseline verification contract and leave the other stabilization work open.

**Tech Stack:** Node ESM, npm scripts, Vitest, SvelteKit/svelte-check, Cargo, Git.

---

## File Structure

- Create `scripts/verify.mjs`: thin cross-platform command runner with explicit ordered steps, inherited stdio, fail-fast exit handling, and Windows `npm.cmd` selection.
- Modify `package.json`: add `"verify": "node scripts/verify.mjs"` to the existing scripts block.
- Modify `docs/project.md`: add concise baseline verification guidance.
- Modify `docs/backlog.md`: mark only the single documented full-project verification command item complete.

## Task 1: Add Verification Runner

**Files:**
- Create: `scripts/verify.mjs`
- Modify: `package.json`

- [ ] **Step 1: Run the missing command to confirm the current gap**

Run:

```powershell
npm.cmd run verify
```

Expected: FAIL with npm reporting that the `verify` script is missing.

- [ ] **Step 2: Create the runner**

Create `scripts/verify.mjs` with exactly this content:

```js
import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm';

const steps = [
  {
    title: 'npm run test',
    command: npmCommand,
    args: ['run', 'test']
  },
  {
    title: 'npm run check',
    command: npmCommand,
    args: ['run', 'check']
  },
  {
    title: 'cargo check --manifest-path src-tauri/Cargo.toml',
    command: 'cargo',
    args: ['check', '--manifest-path', 'src-tauri/Cargo.toml']
  },
  {
    title: 'cargo test --manifest-path src-tauri/Cargo.toml',
    command: 'cargo',
    args: ['test', '--manifest-path', 'src-tauri/Cargo.toml']
  },
  {
    title: 'git diff HEAD --check',
    command: 'git',
    args: ['diff', 'HEAD', '--check']
  }
];

function runStep(step) {
  return new Promise((resolve) => {
    let settled = false;
    const finish = (exitCode) => {
      if (settled) {
        return;
      }

      settled = true;
      resolve(exitCode);
    };

    console.log(`\n=== ${step.title} ===`);

    const child = spawn(step.command, step.args, {
      cwd: repoRoot,
      shell: false,
      stdio: 'inherit'
    });

    child.on('error', (error) => {
      console.error(`\nFailed to start "${step.command}": ${error.message}`);
      finish(1);
    });

    child.on('close', (code, signal) => {
      if (signal) {
        console.error(`\nCommand terminated by signal ${signal}: ${step.title}`);
        finish(1);
        return;
      }

      finish(code ?? 1);
    });
  });
}

for (const step of steps) {
  const exitCode = await runStep(step);

  if (exitCode !== 0) {
    console.error(`\nVerification failed during: ${step.title}`);
    process.exit(exitCode);
  }
}

console.log('\nAll verification checks passed.');
```

- [ ] **Step 3: Add the npm script**

Modify the `scripts` object in `package.json` so it becomes:

```json
"scripts": {
  "dev": "vite dev",
  "build": "vite build",
  "preview": "vite preview",
  "test": "vitest run",
  "test:watch": "vitest",
  "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
  "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
  "verify": "node scripts/verify.mjs",
  "tauri": "tauri"
}
```

- [ ] **Step 4: Run the baseline verification command**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS. The command prints these section headers in order:

```text
=== npm run test ===
=== npm run check ===
=== cargo check --manifest-path src-tauri/Cargo.toml ===
=== cargo test --manifest-path src-tauri/Cargo.toml ===
=== git diff HEAD --check ===
All verification checks passed.
```

- [ ] **Step 5: Commit the runner**

Run:

```powershell
git add package.json scripts\verify.mjs
git commit -m "chore: add full project verification command"
```

## Task 2: Document The Baseline Verification Contract

**Files:**
- Modify: `docs/project.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Add verification guidance to project docs**

In `docs/project.md`, insert this section after the `## Stack` list and before `## Product slice`:

````md
## Verification

Run baseline full-project verification before committing or merging:

```bash
npm run verify
```

This command runs frontend tests, Svelte checks, Rust check/tests, and
`git diff HEAD --check`. It is a baseline local gate; CI, Rust formatting/lint
policy, live Telegram/LLM flows, dependency pinning, and secret-safety audit
coverage remain separate stabilization work.
````

- [ ] **Step 2: Mark the covered backlog item complete**

In `docs/backlog.md`, change only this line:

```md
- [ ] add a single documented full-project verification command or script
```

to:

```md
- [x] add a single documented full-project verification command or script
```

Leave the CI, dependency pinning, runtime-flow verification, and secret-safety
audit items open.

- [ ] **Step 3: Run final verification**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS with the same five section headers from Task 1.

- [ ] **Step 4: Commit the docs**

Run:

```powershell
git add docs\project.md docs\backlog.md
git commit -m "docs: document baseline verification command"
```

## Task 3: Final Status Check

**Files:**
- Inspect: repository status only

- [ ] **Step 1: Confirm the worktree is clean**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## main
```

- [ ] **Step 2: Confirm the final commits**

Run:

```powershell
git log --oneline -3
```

Expected: the newest commits include:

```text
docs: document baseline verification command
chore: add full project verification command
docs: refine verification command design
```
