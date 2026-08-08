# Testing Redesign Slice 3C Design

## Purpose

Close the complete validator-open source-contract remainder from the approved
pre-Slice 3C redisposition checkpoint. Slice 3C implements the exact future
owners already frozen in `testing/source-contract-redisposition-review.json`,
deletes accepted legacy evidence, removes the empty `legacy-contract` runner,
and hands a zero-open-row inventory to Slice 4.

This slice does not repeat the redisposition review. It does not change a
reviewed class, invariant, disposition, replacement identity, deletion reason,
or criticality citation. Nx remains outside the repository until the separate
post-Slice-4 decision gate.

## Frozen Scope

The scope is exactly 436 validator-open rows in 86 legacy files:

- 290 behavior rows with exact future owner identities;
- 146 accepted deletion rows;
- 269 Node/Vitest owners covering 1,707 assertion ordinals;
- 18 Cargo owners covering 156 assertion ordinals;
- 3 Playwright owners covering 21 assertion ordinals;
- 0 jsdom owners.

The decision artifact and the applied ledger are the source of truth. The
implementation plan must derive its row-to-owner tables mechanically from
those files rather than copying 290 identities into a second hand-maintained
map.

## Closure Waves

Slice 3C is one implementation slice with five integration checkpoints. The
checkpoints are not independent program sub-slices and therefore do not each
pay for a complete `verify` run.

| Checkpoint | Scope | Legacy files | Rows | Behavior | Delete |
| --- | --- | ---: | ---: | ---: | ---: |
| A | Every delete-only legacy file | 23 | 70 | 0 | 70 |
| B | Testing and process infrastructure | 7 | 24 | 22 | 2 |
| C | Analysis | 21 | 149 | 124 | 25 |
| D | Projects/library, Gemini browser, and other product UI | 24 | 128 | 114 | 14 |
| E | Prompt packs/YouTube, Rust/security, and the analysis-crate remainder | 11 | 65 | 30 | 35 |
| **Total** |  | **86** | **436** | **290** | **146** |

Checkpoint B contains the two Node process-runtime B3 rows. Checkpoint D
contains the three approved Playwright rows. Checkpoint E contains all 18
Cargo owners.

Each checkpoint closes complete legacy files. Replacement evidence, deletion
of the corresponding legacy files, and the resulting transition state form
one integration commit. A checkpoint does not leave a partially cut-over
legacy file across commits. Reverting that commit restores the previous green
checkpoint.

## Replacement and Cutover Contract

The pre-Slice 3C cutover already applied every final resolution to the ledger.
Slice 3C therefore does not ordinarily edit ledger resolution metadata. Its
data flow is:

```text
approved decision artifact plus applied ledger
  -> mechanically derived exact path and declaration title
  -> focused RED
  -> observable replacement behavior
  -> focused GREEN and runner ownership proof
  -> deletion of the complete legacy file
  -> transition validation
```

The following rules are fail-closed:

- An owner resolves a row only through the exact frozen path and complete
  declaration title.
- Every replacement file has exactly one runner-census owner whose public
  owner script is included by `verify`.
- Replacement tests do not use `?raw`, `readFileSync`, or another direct
  production-source reader.
- A small behavior-neutral extraction of a pure function or adapter is
  permitted when needed to expose an observable seam. Unrelated production
  refactoring is outside scope.
- D1 through D4 rows close by deleting legacy evidence. D4 does not cause an
  already existing owner to be reimplemented.
- A required path, title, mechanism, disposition, invariant, or criticality
  change stops the checkpoint for a reviewed design amendment. It is never
  corrected opportunistically during implementation.

No new migration carrier, ledger, scanner, or general-purpose test framework
is introduced. The redisposition carrier may be used to prove that the
artifact and ledger have not drifted, but Slice 3C does not extend it into an
execution scheduler.

## Vitest and Process Ownership

Ordinary `*.behavior.test.ts` replacements belong to `unit-node`. Replacement
files that spawn, interrupt, or audit real child-process trees belong to
`os-integration` and retain its fork pool and process-safety policy. Pure
wrapper and serialization tests remain in `unit-node` even when they share a
directory with process tests.

Checkpoint B updates the explicit `os-integration` file inventory at the same
time that it deletes the replaced process legacy files. No process-owning test
may silently fall back to the threaded unit project. Process preflight runs
before retained OS, Cargo, or Playwright verification, and postflight checks
look for repository-owned descendants rather than unrelated user processes.

## Application Playwright Owner

The three approved browser rows use a new, separate
`playwright:app-e2e` census owner:

- `e2e/app-shell-responsive.spec.ts`;
- `e2e/dialog-layering.spec.ts`;
- `e2e/research-projects-sources-filter-row.spec.ts`.

Checkpoint D adds `e2e/playwright.config.ts`, the `test:app:e2e` npm script,
the new census owner, and a separate `verify` gate. The existing Gemini
browser adapter config, owner, and command remain unchanged.

The app config launches a real Vite route on an explicit strict test port; it
does not assume port 5173. A pre-navigation init script supplies the narrow
Tauri IPC responses required by each scenario. The tests exercise the real
application components and styles rather than reproducing their markup in a
fixture.

Checkpoint D begins with one minimal route smoke that proves this harness and
its cleanup. Failure to mount the real route without a production-only browser
harness stops the checkpoint before the three frozen scenarios are written.
After the browser command, a process audit rejects leaked Vite or Playwright
Chromium descendants by PID and command-line marker.

## Cargo Ownership

Checkpoint E implements the 18 exact Cargo identities in their frozen packages
and modules. A Rust implementation task names its affected packages and uses
the repository's focused Rust verification loop:

- an exact non-empty RED/GREEN test;
- `cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
- `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
- the final workspace Cargo gates through `npm.cmd run verify`.

The two Windows process-tree B3 owners run only where their OS contract can be
observed truthfully. A skipped platform case is not accepted as the sole proof
of the frozen invariant.

## Legacy Runner Removal

The `legacy-contract` project remains present and non-empty until checkpoint E
has closed the last legacy file. The final checkpoint then removes:

- the `legacy-contract` Vitest project;
- `LEGACY_TEST_FILES` and the ledger-derived unit exclusions;
- the `vitest:legacy-contract` census owner;
- `test:legacy-contract` from `package.json`;
- the legacy gate from `scripts/verify.mjs`.

The resulting runner census must be non-empty, total, and disjoint. Newly
created replacement files must remain owned after the ledger-derived legacy
exclusions disappear.

## Checkpoint Verification

Every checkpoint follows the same bounded cycle:

1. exact focused RED for each new owner;
2. minimal implementation and focused GREEN;
3. all owner files for the checkpoint, with at most two frontend commands in
   parallel;
4. `npm.cmd run check` only when Svelte or TypeScript production files changed;
5. affected Cargo checks and package checkpoints, not concurrent with the
   transition validator;
6. `node scripts/validate-testing-transition.mjs`;
7. `git diff --check` and one read-only review followed by at most one combined
   fix wave;
8. one integration commit for the complete-file checkpoint.

An empty or unexpectedly small selection is a failure. A correctness failure
is preserved and fixed rather than replaced by an unexplained green rerun.
The final complete `verify` may be repeated only after a demonstrated spawn,
sandbox, or process-infrastructure error; both observations remain in the
verification record.

## Minimal Timing Policy

Slice 3C adds no timing writer, timing artifact, threshold, warmup, paired run,
median, or per-owner benchmark. Focused command durations are not retained as
performance evidence.

The only retained timing observation is the wall time of one final fresh
unsandboxed `npm.cmd run verify`. It is informational and cannot change
correctness, owner classification, or the future Slice 4 fast/slow decision.

## Completion Evidence

Slice 3C is complete only when:

- validator-open rows fall from 436 to 0;
- all 86 legacy files are absent;
- all 290 behavior rows resolve through their exact owners and all 146 delete
  rows close through their accepted deletion reasons;
- the legacy Vitest project, census owner, npm script, and verify gate are
  absent;
- Vitest and Playwright census validation is total, disjoint, and non-empty;
- all affected Rust packages and the final workspace gates pass;
- one final fresh unsandboxed `npm.cmd run verify` exits zero;
- the verification document records row and file counts, command exit codes,
  the one final wall-time observation, process-audit results, commit IDs, and
  the Slice 4 handoff.

The program index is updated only at final evidence handoff. Slice 4 proceeds
without Nx. The disposable Slice 2C Nx decision gate remains deferred until
Slice 4 has committed.
