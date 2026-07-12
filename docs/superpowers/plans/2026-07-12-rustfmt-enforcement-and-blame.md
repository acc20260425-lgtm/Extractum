# Rustfmt Enforcement and Blame Hygiene Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust formatting drift fail both a focused project command and the aggregate verification pipeline, while keeping the mechanical rustfmt baseline commit out of useful blame attribution.

**Architecture:** Expose Cargo's check-only rustfmt invocation as `check:rustfmt`, reuse that public package script from the existing `verify.mjs` npm-step abstraction, and document it as part of normal Rust validation. Store the isolated style commit in Git's standard ignore-revisions file and verify it through the command-line blame override without mutating local Git configuration.

**Tech Stack:** npm scripts, Node.js ESM, Cargo/rustfmt, Git, PowerShell, project workflow documentation.

## Global Constraints

- The formatting revision is exactly `acbe5bfd2105f4930063f1c8a204e57a9f47c86f`, whose subject must be `style: format rust sources`.
- `check:rustfmt` must execute exactly `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; it must never repair files.
- `scripts/verify.mjs` must invoke `check:rustfmt` through the existing `npmStep` helper before `cargo check`.
- Do not change Rust source permanently; the negative probe must restore the exact original bytes in `src-tauri/src/main.rs` from a `finally` block.
- Do not add a Git hook, CI workflow, formatter configuration, dependency, runtime behavior, or `docs/value-registry.md` entry.
- Do not execute `git config blame.ignoreRevsFile`; document it as an optional per-clone user action only.
- Final implementation scope is exactly `package.json`, `scripts/verify.mjs`, `AGENTS.md`, and `.git-blame-ignore-revs`. The plan document is committed separately before execution.

---

### Task 1: Add the Focused and Aggregate Rustfmt Checks

**Files:**
- Modify: `package.json:32-36`
- Modify: `scripts/verify.mjs:30-42`
- Temporarily create and remove: `scripts/rustfmt-check-only-probe.mjs`
- Temporarily mutate and restore: `src-tauri/src/main.rs`

**Interfaces:**
- Produces package script `check:rustfmt` with no arguments and process exit status inherited from Cargo.
- Extends `steps` in `scripts/verify.mjs` with `npmStep('npm run check:rustfmt', 'check:rustfmt')` immediately before the existing Cargo check object.
- Does not change `npm.cmd run check` or any Rust interface.

- [ ] **Step 1: Verify the clean-tree and style-commit preconditions**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
$hash = git rev-parse acbe5bfd2105f4930063f1c8a204e57a9f47c86f
$subject = git show -s --format='%s' $hash
"STATUS_COUNT=$($status.Count)"
"STYLE_HASH=$hash"
"STYLE_SUBJECT=$subject"
if ($status.Count -ne 0) { exit 1 }
if ($hash -ne 'acbe5bfd2105f4930063f1c8a204e57a9f47c86f') { exit 1 }
if ($subject -ne 'style: format rust sources') { exit 1 }
```

Expected: `STATUS_COUNT=0`, the full expected hash, and the exact expected
subject. Stop if the tree is dirty; do not overwrite user changes.

- [ ] **Step 2: Record the mechanical RED for both missing integrations**

Run:

```powershell
$package = Get-Content -Raw package.json | ConvertFrom-Json
$verify = Get-Content -Raw scripts/verify.mjs
$scriptCorrect = $package.scripts.'check:rustfmt' -eq 'cargo fmt --manifest-path src-tauri/Cargo.toml -- --check'
$verifyCorrect = $verify.Contains("npmStep('npm run check:rustfmt', 'check:rustfmt')")
"PACKAGE_SCRIPT_CORRECT=$scriptCorrect"
"VERIFY_STEP_CORRECT=$verifyCorrect"
if (-not $scriptCorrect -or -not $verifyCorrect) { exit 1 }
```

Expected: exit 1 with both values `False`, proving neither integration exists.

- [ ] **Step 3: Add the check-only package script**

In `package.json`, change the scripts excerpt to exactly:

```json
"test:watch": "node scripts/run-vitest.mjs watch",
"check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
"check:rustfmt": "cargo fmt --manifest-path src-tauri/Cargo.toml -- --check",
"check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
```

Expected: JSON remains valid. Do not alter the established frontend `check`
script.

- [ ] **Step 4: Add the package script to aggregate verification**

In `scripts/verify.mjs`, change the beginning of `steps` to exactly:

```javascript
const steps = [
  npmStep('npm run test', 'test'),
  npmStep('npm run check', 'check'),
  npmStep('npm run check:rustfmt', 'check:rustfmt'),
  {
    title: 'cargo check --manifest-path src-tauri/Cargo.toml',
```

Expected: the new npm step precedes Cargo check and uses `npmStep`; do not
duplicate the Cargo formatting command in `verify.mjs`.

- [ ] **Step 5: Verify the focused and aggregate source contracts are GREEN**

Run the Step 2 command again.

Expected: `PACKAGE_SCRIPT_CORRECT=True`, `VERIFY_STEP_CORRECT=True`, exit 0.

Then run:

```powershell
npm.cmd run check:rustfmt
```

Expected: exit 0 with no rustfmt diff output.

- [ ] **Step 6: Create the temporary negative-probe harness**

Create `scripts/rustfmt-check-only-probe.mjs` with exactly:

```javascript
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const target = path.join(repoRoot, 'src-tauri', 'src', 'main.rs');
const original = await readFile(target);
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');
let probeExit = 1;

try {
  const text = original.toString('utf8');
  const malformed = text.replace('fn main() {', 'fn main(){');
  if (malformed === text) {
    throw new Error('Expected formatted main declaration was not found');
  }

  const malformedBytes = Buffer.from(malformed, 'utf8');
  await writeFile(target, malformedBytes);
  const malformedHash = digest(malformedBytes);

  const result = spawnSync(
    process.env.ComSpec ?? 'cmd.exe',
    ['/d', '/s', '/c', 'npm.cmd run check:rustfmt'],
    {
      cwd: repoRoot,
      encoding: 'utf8',
      shell: false
    }
  );
  const afterCheck = await readFile(target);
  const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;

  console.log(`NEGATIVE_EXIT=${result.status}`);
  console.log(`MALFORMED_UNCHANGED=${digest(afterCheck) === malformedHash}`);
  console.log(output);

  if (result.status === 0) {
    throw new Error('check:rustfmt unexpectedly accepted malformed Rust');
  }
  if (digest(afterCheck) !== malformedHash) {
    throw new Error('check:rustfmt rewrote the malformed Rust file');
  }
  if (!output.includes('Diff in')) {
    throw new Error('check:rustfmt did not print a rustfmt diff');
  }

  probeExit = 0;
} finally {
  await writeFile(target, original);
  console.log(`ORIGINAL_RESTORED=${digest(await readFile(target)) === digest(original)}`);
}

process.exit(probeExit);
```

Expected: this temporary harness owns the controlled mutation and restores the
exact original bytes from `finally`. Do not stage or commit the harness.

- [ ] **Step 7: Run the negative probe and prove cleanup**

Run:

```powershell
$before = @(git status --short --untracked-files=all -- src-tauri/src/main.rs)
if ($before.Count -ne 0) { exit 1 }
node scripts/rustfmt-check-only-probe.mjs
$probeExit = $LASTEXITCODE
$after = @(git status --short --untracked-files=all -- src-tauri/src/main.rs)
"PROBE_EXIT=$probeExit"
"TARGET_STATUS_COUNT=$($after.Count)"
if ($probeExit -ne 0 -or $after.Count -ne 0) { exit 1 }
```

Expected: nested output includes `NEGATIVE_EXIT=1`,
`MALFORMED_UNCHANGED=true`, `ORIGINAL_RESTORED=true`; the wrapper prints
`PROBE_EXIT=0`, `TARGET_STATUS_COUNT=0` and exits 0.

- [ ] **Step 8: Remove the temporary harness and run focused validation**

Delete only `scripts/rustfmt-check-only-probe.mjs`, confirm it no longer
exists, then run:

```powershell
if (Test-Path scripts/rustfmt-check-only-probe.mjs) { exit 1 }
npm.cmd run check:rustfmt
npm.cmd run check
git diff --check
```

Expected: the temporary harness is absent and all three commands exit 0.

- [ ] **Step 9: Review and commit the enforcement integration**

Run:

```powershell
$changed = @(git status --porcelain=v1 --untracked-files=all | ForEach-Object {
    $_.Substring(3).Replace('\', '/')
})
$allowed = @('package.json', 'scripts/verify.mjs')
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
"CHANGED=$($changed -join ',')"
"UNEXPECTED=$($unexpected -join ',')"
if ($changed.Count -ne 2 -or $unexpected.Count -ne 0) { exit 1 }
git diff -- package.json scripts/verify.mjs
git add -- package.json scripts/verify.mjs
git commit -m "chore: enforce Rust formatting checks"
```

Expected: only the package script and aggregate verification step are in the
commit, and the working tree is clean afterward.

---

### Task 2: Document the Workflow and Configure Blame Attribution

**Files:**
- Modify: `AGENTS.md:39-44`
- Create: `.git-blame-ignore-revs`

**Interfaces:**
- Produces a repository convention requiring `npm.cmd run check:rustfmt` and `cargo check` after Rust or Tauri backend changes when no Superpowers workflow is active.
- Produces a standard Git ignore-revisions file consumable by `git blame --ignore-revs-file .git-blame-ignore-revs` or optional local `blame.ignoreRevsFile` configuration.
- Does not modify the user's local Git configuration.

- [ ] **Step 1: Record the documentation and blame-file RED**

Run:

```powershell
$agents = Get-Content -Raw AGENTS.md
$blameFileExists = Test-Path .git-blame-ignore-revs
$hasFmtRule = $agents.Contains('npm.cmd run check:rustfmt')
$hasBlameCommand = $agents.Contains('git config blame.ignoreRevsFile .git-blame-ignore-revs')
"BLAME_FILE_EXISTS=$blameFileExists"
"HAS_FMT_RULE=$hasFmtRule"
"HAS_BLAME_COMMAND=$hasBlameCommand"
if (-not $blameFileExists -or -not $hasFmtRule -or -not $hasBlameCommand) { exit 1 }
```

Expected: exit 1 with all three values `False`.

- [ ] **Step 2: Update the Rust validation and blame guidance**

In `AGENTS.md`, replace the existing Rust validation bullet with exactly:

```markdown
- When no Superpowers workflow is active, run `npm.cmd run check:rustfmt` and `cargo check` after Rust or Tauri backend changes.
```

Then add this bullet immediately after the validation statement that forbids
unverified pass claims:

```markdown
- To keep the repository-wide mechanical formatting commit out of local blame attribution, developers may run `git config blame.ignoreRevsFile .git-blame-ignore-revs` once per clone; do not change local Git configuration automatically.
```

Expected: Windows command conventions remain explicit, and the Git config
command is optional rather than an automated setup action.

- [ ] **Step 3: Create the standard blame-ignore file**

Create `.git-blame-ignore-revs` with exactly:

```text
# Repository-wide mechanical rustfmt baseline; ignore for line attribution.
acbe5bfd2105f4930063f1c8a204e57a9f47c86f
```

Expected: the file contains one comment and one full 40-character revision.

- [ ] **Step 4: Verify the documentation contract and exact revision**

Run the Step 1 command again.

Expected: all three values are `True`, exit 0.

Then run:

```powershell
$revisions = @(Get-Content .git-blame-ignore-revs | Where-Object {
    $_ -and -not $_.StartsWith('#')
})
$expected = 'acbe5bfd2105f4930063f1c8a204e57a9f47c86f'
$resolved = if ($revisions.Count -eq 1) { git rev-parse $revisions[0] } else { '' }
$subject = if ($resolved) { git show -s --format='%s' $resolved } else { '' }
"REVISION_COUNT=$($revisions.Count)"
"RESOLVED=$resolved"
"SUBJECT=$subject"
if ($revisions.Count -ne 1 -or $resolved -ne $expected -or $subject -ne 'style: format rust sources') { exit 1 }
```

Expected: one revision, the exact full hash, and the exact style-commit subject.

- [ ] **Step 5: Functionally verify blame without changing Git config**

Run:

```powershell
$configBefore = git config --local --get blame.ignoreRevsFile
$configBeforeExit = $LASTEXITCODE
$withoutOutput = git blame -- src-tauri/src/youtube/process_runtime.rs
$withoutExit = $LASTEXITCODE
$withOutput = git blame --ignore-revs-file .git-blame-ignore-revs -- src-tauri/src/youtube/process_runtime.rs
$withExit = $LASTEXITCODE
$without = @($withoutOutput | Select-String -SimpleMatch 'acbe5bfd').Count
$with = @($withOutput | Select-String -SimpleMatch 'acbe5bfd').Count
$limit = [math]::Max(10, [int]($without / 10))
$configAfter = git config --local --get blame.ignoreRevsFile
$configAfterExit = $LASTEXITCODE
"WITHOUT_EXIT=$withoutExit"
"WITH_EXIT=$withExit"
"WITHOUT_STYLE_ATTRIBUTION=$without"
"WITH_STYLE_ATTRIBUTION=$with"
"WITH_LIMIT=$limit"
"CONFIG_UNCHANGED=$(($configBeforeExit -eq $configAfterExit) -and ($configBefore -eq $configAfter))"
if ($withoutExit -ne 0 -or $withExit -ne 0) { exit 1 }
if ($without -le 0 -or $with -ge $without -or $with -gt $limit) { exit 1 }
if ($configBeforeExit -ne $configAfterExit -or $configBefore -ne $configAfter) { exit 1 }
```

Expected on the recorded baseline: both blame commands exit 0, attribution
drops from `600` lines without the ignore file to `9` with it, the permitted
limit is `60`, and `CONFIG_UNCHANGED=True`. The pass/fail rule uses the
differential threshold rather than requiring those informational counts to
remain frozen. Git may retain the ignored commit on rustfmt-created lines that
have no unambiguous parent. Do not run the documented `git config` command.

- [ ] **Step 6: Run the complete aggregate verification**

Run:

```powershell
npm.cmd run verify
```

Expected: output contains an `=== npm run check:rustfmt ===` section before
the Cargo check section, and the entire pipeline exits 0 with
`All verification checks passed.`

- [ ] **Step 7: Review exact final scope and commit**

Run:

```powershell
git diff --check
$changed = @(git status --porcelain=v1 --untracked-files=all | ForEach-Object {
    $_.Substring(3).Replace('\', '/')
})
$allowed = @('.git-blame-ignore-revs', 'AGENTS.md')
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
"CHANGED=$($changed -join ',')"
"UNEXPECTED=$($unexpected -join ',')"
if ($changed.Count -ne 2 -or $unexpected.Count -ne 0) { exit 1 }
git diff -- AGENTS.md
Get-Content .git-blame-ignore-revs
git add -- AGENTS.md .git-blame-ignore-revs
git commit -m "docs: preserve Rust formatting baseline"
git status --short --branch
```

Expected: only `AGENTS.md` and `.git-blame-ignore-revs` are committed. The
working tree is clean, and the branch contains the two focused implementation
commits after this plan commit.
