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
- 179 Node/Vitest rows covering 1,109 assertion ordinals;
- 90 jsdom/Vitest rows covering 598 assertion ordinals;
- 18 Cargo owners covering 156 assertion ordinals;
- 3 Playwright owners covering 21 assertion ordinals;
- 72 Vitest target files, 18 Cargo identities, and 3 Playwright target files.

The decision artifact and the applied ledger are the source of truth. The
implementation plan must derive its row-to-owner tables mechanically from
those files rather than copying 290 identities into a second hand-maintained
map. The Node/jsdom split above is the one forecast correction approved by
this design: paths and declaration titles remain unchanged, while the 27
`src/lib/components/**/*.behavior.test.ts` targets move from the artifact's
filename-only Node forecast to the truthful component project.

## Connected Closure Components

The bipartite graph between legacy files and replacement targets is almost a
forest. Its 84 connected components are the unit of closure and rollback:

| Graph shape `(legacy files, replacement targets)` | Components |
| --- | ---: |
| `(1, 0)` | 23 |
| `(1, 1)` | 44 |
| `(1, 2)` | 7 |
| `(1, 3)` | 5 |
| `(1, 5)` | 2 |
| `(1, 6)` | 1 |
| `(2, 2)` | 2 |
| **Total** | **84** |

No thematic wave creates a correctness dependency between disconnected
components. Components may be implemented in parallel after checking that
their production seam edits do not overlap. The two `(2, 2)` components remain
indivisible because each connects two legacy files.

Replacement-only commits are allowed before cutover because the still-present
legacy declaration keeps the row open. A cutover commit deletes every legacy
file in its connected component only after all of that component's targets are
green. No commit deletes only part of a connected component. Reverting either
an additive replacement commit or a component cutover returns a green state.

## Execution Waves

Waves batch independent targets for implementation and components for cutover;
they do not redefine the graph. Among numbered targets, a component's cutover
wave is their maximum wave rank; any Playwright target assigns the component
to the separate browser cutover bucket. The target counts below therefore
describe implementation volume, not the number of legacy files deleted by
that wave.

1. **Wave 0 -- deletion only.** Delete the 23 `(1, 0)` legacy files, closing
   70 rows through their existing deletion reasons. This is one deletion
   commit plus transition validation, with no RED/GREEN cycle. The remaining
   76 accepted delete rows close incidentally when their 25 mixed legacy files
   later leave the graph.
2. **Wave 1 -- one-row Vitest targets.** Implement the 30 Vitest target files
   that each own one behavior row, batched in groups of roughly 8--10 by
   directory and shared fixture surface.
3. **Wave 2 -- multi-row Vitest targets.** Implement the remaining 42 Vitest
   target files. Large targets are independent batches rather than one
   analysis- or product-wide checkpoint.
4. **Wave 3 -- Cargo.** Implement the 18 exact Cargo identities, grouped by
   package and connected component.
5. **Browser track.** Start the three Playwright targets after Wave 0 and
   implement them independently of Waves 1--3. A component containing both
   browser and Vitest targets closes only after both sides are green, but the
   browser cutover may occur before or after Cargo according to actual
   readiness.

Ten components span more than one implementation wave: six span Waves 1 and
2, two span Waves 1 and 3, and two span a Vitest wave and the browser track.
Their additive targets may land earlier, but their legacy files are assigned
to the maximum numbered cutover wave, or to the browser track when they contain
a Playwright target. Consequently Wave 1 does not claim that all 30 one-row
targets immediately remove their legacy files, and Wave 3 may close a
component whose Node target was implemented in Wave 1.

The resulting cutover distribution is:

| Cutover bucket | Components | Legacy files | Rows closed |
| --- | ---: | ---: | ---: |
| Wave 0 | 23 | 23 | 70 |
| Wave 1 | 14 | 14 | 29 |
| Wave 2 | 35 | 37 | 271 |
| Wave 3 | 9 | 9 | 55 |
| Browser track | 3 | 3 | 11 |
| **Total** | **84** | **86** | **436** |

Wave 0 remains one special deletion-only batch. Wave 1, Wave 3, and the browser
track each remain one integration batch. Wave 2 is divided into four or five
cutover batches, each closing 7--10 legacy files without splitting a connected
component. Each batch receives one complete transition-validator run and one
integration review. This adds only the validator runs needed to keep the
271-row wave reviewable; the plan does not add a subset-validation mode or a
second Cargo-evidence path. The final complete `verify` still runs only once.

## Replacement and Cutover Contract

The pre-Slice 3C cutover already applied every final resolution to the ledger.
Slice 3C therefore does not ordinarily edit ledger resolution metadata. Wave 0
has one approved deletion-coupled envelope cleanup: once
`src/lib/analysis-migration-fixture-contract.test.ts` is absent, remove only
its top-level `sourceReaderExceptions` tuple at `14:13-14:74`. The carrier
derives a live current-present set from tracked paths whose files exist in the
worktree and fails closed unless that exact sole base tuple is absent only when
that exact reader path is absent; it rejects a present reader path, any other
exception addition/removal/mutation/reordering, and any `rows[]` change. This
does not alter a row resolution, class, invariant, or deletion reason. Its
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
- Except for the component-project correction explicitly approved here, a
  required path, title, mechanism, disposition, invariant, or criticality
  change stops the affected connected component for a reviewed design
  amendment. It does not stop unrelated components and is never corrected
  opportunistically during implementation.

No new migration carrier, ledger, scanner, or general-purpose test framework
is introduced. The redisposition carrier may be used to prove that the
artifact and ledger have not drifted, but Slice 3C does not extend it into an
execution scheduler.

## Vitest and Process Ownership

Ordinary `*.behavior.test.ts` replacements outside `src/lib/components/`
belong to `unit-node`. The component project adds the narrow include pattern
`src/lib/components/**/*.behavior.test.ts`, and `unit-node` excludes the same
pattern. This places all 27 component behavior targets -- 90 rows and 598
assertion ordinals -- under jsdom without changing any frozen replacement
path or title. No existing tracked file currently matches that new pattern.
Those targets inherit the component project's existing
`scripts/testing/setup-component-tests.ts` setup and
`svelteTestingOptions: { autoCleanup: false }`; Slice 3C adds no second setup
file. The existing `src/lib/telegram-checkpoint-2.behavior.test.ts` and
`src/lib/telegram-contract-paths.behavior.test.ts` files are outside
`src/lib/components/` and remain in `unit-node`.

The implementation's first runner-ownership change updates the redisposition
forecast and its forecast validator from 269 Node rows and 0 jsdom rows to the
corrected 179 Node rows and 90 jsdom rows. Historical packet transport remains
content-addressed evidence of the earlier review convention and is not
rewritten or re-reviewed; the carrier uses a separate execution-owner
classifier for this approved forecast correction.

Replacement files that spawn, interrupt, or audit real child-process trees
belong to `os-integration` and retain its fork pool and process-safety policy.
Pure wrapper and serialization tests remain in `unit-node` even when they
share a directory with process tests. A new process target enters the explicit
`os-integration` inventory in the same additive commit that creates that
target. Its corresponding legacy path remains in the inventory until the
component cutover commit deletes the legacy file and removes the old entry.
The overlap is intentional: at no point may a process-owning replacement fall
back to the threaded unit project. Process preflight runs before retained OS,
Cargo, or Playwright verification, and postflight checks look for
repository-owned descendants rather than unrelated user processes.

## Application Playwright Owner

The three approved browser rows use a new, separate
`playwright:app-e2e` census owner:

- `e2e/app-shell-responsive.spec.ts`;
- `e2e/dialog-layering.spec.ts`;
- `e2e/research-projects-sources-filter-row.spec.ts`.

The independent browser track adds `e2e/playwright.config.ts`, the
`test:app:e2e` npm script, the new census owner, and a separate `verify` gate.
The existing Gemini
browser adapter config, owner, and command remain unchanged.

The app config launches a real Vite route on an explicit strict test port; it
does not assume port 5173. A pre-navigation init script supplies the narrow
Tauri IPC responses required by each scenario. The tests exercise the real
application components and styles rather than reproducing their markup in a
fixture.

The browser track begins with one minimal route smoke that proves this harness
and its cleanup. Failure to mount the real route without a production-only
browser harness does not stop development of unrelated Vitest or Cargo
targets, but it delays cutover of the two mixed browser/Vitest components,
cutover of the one browser-only `(1, 1)` component, and therefore final removal
of the legacy runner. In particular, the app-sidebar component also needs its
jsdom target, while the dialog component also needs both desktop-dialog and
modal-host jsdom targets. After the browser command, a process audit rejects
leaked Vite or Playwright Chromium descendants by PID and command-line marker.

## Cargo Ownership

Wave 3 implements the 18 exact Cargo identities in their frozen packages and
modules. A Rust implementation task names its affected packages and uses
the repository's focused Rust verification loop:

- an exact non-empty RED/GREEN test;
- `cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
- `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
- the final workspace Cargo gates through `npm.cmd run verify`.

The two Windows process-tree B3 owners run only where their OS contract can be
observed truthfully. A skipped platform case is not accepted as the sole proof
of the frozen invariant.

## Legacy Runner Removal

The `legacy-contract` project remains present and non-empty until the final
connected component is ready to close. The commit that deletes the last legacy
file -- whichever cutover batch is factually last -- atomically removes:

- the `legacy-contract` Vitest project;
- `LEGACY_TEST_FILES` and the ledger-derived unit exclusions;
- the `vitest:legacy-contract` census owner;
- `test:legacy-contract` from `package.json`;
- the legacy gate from `scripts/verify.mjs`.

The resulting runner census must be non-empty, total, and disjoint. Newly
created replacement files must remain owned after the ledger-derived legacy
exclusions disappear.

## Batch Verification

Wave 0 is intentionally smaller than the behavior waves: delete its 23 files,
run the complete transition validator once, inspect the deletion-only diff,
and commit. It has no RED/GREEN cycle, production check, Cargo checkpoint, or
replacement review.

Each behavior target or target batch follows the bounded development cycle:

1. one focused RED containing the target file's complete frozen declaration
   set, rather than one process cycle per ledger row;
2. minimal implementation and focused GREEN;
3. all owner files for the batch, with at most two frontend commands in
   parallel;
4. `npm.cmd run check` only when Svelte or TypeScript production files changed;
5. affected Cargo checks and package checkpoints, not concurrent with the
   transition validator;
6. `git diff --check` and focused review.

The complete `node scripts/validate-testing-transition.mjs` runs once before a
batch cutover deletes its ready connected-component legacy files. Independent
targets may be added in earlier green commits while those legacy declarations
remain present. A cutover batch receives one read-only integration review and
at most one combined fix wave before commit.

Five components mix jsdom and Node targets. Such a batch reserves both
frontend command slots for its focused `component` and `unit-node` owners; no
third frontend command overlaps them. Multiple mixed components may share the
same two commands only when their exact file paths are passed together.

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
