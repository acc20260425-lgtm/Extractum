# Research Projects Lucide Direct Imports Design

## Status

Approved for implementation planning on 2026-07-15 after written-spec review.

## Context

The completed two-file Lucide experiment is documented in
`docs/superpowers/verification/2026-07-15-lucide-direct-import-validation.md`.
It replaced root `@lucide/svelte` imports in `ProjectRailPanel.svelte` and
`SourcesTab.svelte`, removed the generated icon index from both measured import
chains, passed the complete verification gate, and improved the measured
complete-suite median from 67.361 seconds to 64.901 seconds. The observed
-3.652% delta is local evidence, not a portable speed guarantee.

After that slice, 56 Svelte source files still import from the package root.
Twenty are immediate files in
`src/lib/components/research-projects/`. Those files share test paths with some
of the slowest frontend tests and repeatedly evaluate the Lucide barrel under
Vitest isolation. The previous candidate established that the installed
`@lucide/svelte` 1.17.0 direct modules work in this repository. This slice
tests the next bounded unit: the remaining root imports in that one component
directory.

The candidate contains 20 component files and 34 unique icon identifiers. The
other 36 root-import consumers remain deliberately outside this design.

## Goal

Remove root `@lucide/svelte` imports from every immediate Svelte component in
the `research-projects` directory, retain the migration only when correctness
and performance stay within the predeclared bounds, and protect the resulting
directory-level import boundary.

## Non-Goals

- No repository-wide Lucide migration.
- No change to the remaining 36 root-import consumers outside the selected
  directory.
- No local icon facade, replacement barrel, or shared icon module.
- No package, lock-file, Vite, Vitest, SvelteKit, TypeScript, or ESLint change.
- No change to markup, props, events, styles, accessibility behavior, or icon
  semantics.
- No minimum speedup requirement or portable timing promise.
- No new domain string value and therefore no change to
  `docs/value-registry.md`.

## Selected Approach

Treat all 20 remaining root-import components in the directory as one
reversible performance candidate. Replace each named package-root import with
default imports from `@lucide/svelte/icons/*` while preserving the existing
local identifier. Do not change any use site.

The candidate owns exactly these files:

- `ConnectFromLibrary.svelte`
- `IconRail.svelte`
- `Inspector.svelte`
- `LibraryFilterRail.svelte`
- `LibraryInspector.svelte`
- `LibraryTelegramDialogImport.svelte`
- `LibraryWorkspace.svelte`
- `LibraryYoutubePlaylistImport.svelte`
- `LibraryYoutubeSmartImport.svelte`
- `ProjectInspector.svelte`
- `ProjectRail.svelte`
- `ProjectRunReportPanel.svelte`
- `ProjectRunsScreen.svelte`
- `ProjectRunsTab.svelte`
- `ProjectsShell.svelte`
- `RunDock.svelte`
- `TopCommandBar.svelte`
- `YoutubeSummaryResultView.svelte`
- `YoutubeSummaryRunDialog.svelte`
- `YoutubeSummaryRunsPanel.svelte`

The installed direct-module aliases were checked during design. Use these
identifier-to-module mappings:

| Identifier | Direct module | Identifier | Direct module |
| --- | --- | --- | --- |
| `Activity` | `activity` | `AlertTriangle` | `triangle-alert` |
| `BookOpen` | `book-open` | `Braces` | `braces` |
| `Check` | `check` | `ChevronDown` | `chevron-down` |
| `ChevronLeft` | `chevron-left` | `ChevronRight` | `chevron-right` |
| `Download` | `download` | `Edit3` | `pen-line` |
| `ExternalLink` | `external-link` | `Eye` | `eye` |
| `FileJson` | `file-json` | `FileText` | `file-text` |
| `Folder` | `folder` | `FolderKanban` | `folder-kanban` |
| `Layers` | `layers` | `Library` | `library` |
| `Link2` | `link-2` | `Minus` | `minus` |
| `PanelLeftClose` | `panel-left-close` | `PanelLeftOpen` | `panel-left-open` |
| `Pencil` | `pencil` | `Play` | `play` |
| `PlayCircle` | `circle-play` | `Plus` | `plus` |
| `RefreshCw` | `refresh-cw` | `Save` | `save` |
| `Search` | `search` | `Settings` | `settings` |
| `ShieldCheck` | `shield-check` | `Trash2` | `trash-2` |
| `X` | `x` | `XCircle` | `circle-x` |

The full module path is `@lucide/svelte/icons/<direct-module>`. The table is
implementation guidance, not permanent contract data.

Four existing local identifiers use deprecated Lucide aliases at the package
root. Import their canonical modules while preserving the local identifiers:
`AlertTriangle` uses `triangle-alert`, `Edit3` uses `pen-line`, `PlayCircle`
uses `circle-play`, and `XCircle` uses `circle-x`. In the installed package,
the deprecated direct aliases re-export these same Svelte components, so this
choice preserves rendering while avoiding paths already scheduled for removal
upstream. Do not use `alert-triangle`, `edit-3`, `play-circle`, or `x-circle` in
the candidate.

One candidate is preferred over three smaller packages because the first
two-file experiment already validated the direct-export mechanism. A single
directory-sized candidate removes the barrel from shared component paths and
requires one controlled A/B session instead of three. A static migration
without measurement is rejected because performance is the reason for the
change. A repository-wide migration is rejected because it would mix several
unrelated UI areas and make rollback and diagnosis less precise.

## Candidate Isolation and Switching

Require a clean worktree and exactly the 20 expected root-import files before
creating the candidate. If the live inventory differs, stop and revise the
scope rather than silently expanding or shrinking it.

Treat current `main` as A and the direct-import candidate as B. Before any
timing run:

1. record the baseline commit and environment;
2. copy the exact bytes of all 20 A files to an absolute scratch directory
   outside the repository and record their SHA-256 hashes;
3. create B as one focused source edit and save its exact file bytes and
   hashes in a separate scratch snapshot;
4. retain a reviewable Git patch for the candidate;
5. prove A -> B -> A -> B transitions before measurement.

The measurement runner switches variants by restoring the captured byte
snapshots, not by reconstructing imports or relying on line-ending conversion
through Git. After every switch it verifies all 20 hashes as one set. A copy
failure, partial set, or hash mismatch is an infrastructure failure: stop all
measurement, restore the full A snapshot, and investigate. No test or build
runs concurrently with a variant transition.

Do not add the directory contract test until the A/B retention decision is
complete. The recorded A and B runs must have identical committed test
inventories and differ only in the 20 component import blocks.

## Measurement Protocol

### Environment

Use the same Windows workstation and committed configuration for both sides.
Record the commit, Node and Vitest versions, active power scheme, Microsoft
Defender real-time protection state, and relevant configuration hashes without
changing machine policy. Confirm no competing Vitest, Vite, Tauri development,
or other benchmark process is active. Run all measurements sequentially.

Store JSON reports, logs, snapshots, patches, and import-duration output under
an absolute temporary path outside the repository. Commit only summarized
evidence.

### Complete-Suite Sequence

Run one discarded warm-up under A and one discarded warm-up under B. Then run
the recorded sequence A/B/A/B/A/B, switching and hash-verifying all 20 files
between runs. Capture complete-suite JSON output, readable failure logs, wall
time, file count, and test count for every recorded run.

Warm-up timings do not enter aggregates. A failed or empty warm-up invalidates
the session; investigate, restore A, and restart from both warm-ups.

For each side calculate the complete-suite median. This median is a
non-regression gate, not the primary proof of benefit: a 20-file change is
larger than the previous candidate, but workstation noise and shared imports
can still mask its wall-time effect.

As diagnostics, extract per-file durations for these representative roots:

- `ResearchProjectsShell.test.ts`;
- `Inspector.test.ts`;
- `RunDock.test.ts`.

Their medians may explain the result, but no individual-file speedup is a
retention requirement because Vitest reports overlap under parallel execution.

### Import-Duration Evidence

Run one complete instrumented suite under A and one under B. First use the
installed Vitest import-duration mechanism already validated by the previous
profiling slice. If the flag is unavailable, rejected, or produces no
breakdown, temporarily use the documented equivalent in the owned Vitest
configuration, then restore that configuration byte-for-byte and verify its
hash before interpreting results.

Inspect the per-file import trees rooted at `ResearchProjectsShell.test.ts`,
`Inspector.test.ts`, and `RunDock.test.ts`. In A, each selected tree must be
nonempty and pass through `@lucide/svelte/dist/icons/index.js`. In B, none may
pass through that index. Normalize slash direction before matching paths.
Searching the global import list is not sufficient because out-of-scope files
are expected to keep the Lucide index globally present.

The import-duration comparison is qualitative: the path disappears or it
does not. Do not infer a numeric speedup from one instrumented run per side.

## Failure Classification

Every recorded runner writes metadata that distinguishes a started Vitest run
from a failure before Vitest execution.

- Missing or unreadable metadata, failed variant switching, hash mismatch, or
  another pre-Vitest error is an infrastructure failure. Restore A and
  invalidate the session.
- A confirmed failed or empty A run invalidates the session; it says nothing
  about B.
- A confirmed failed or empty B run rejects the candidate.
- Apply the same classification to both the initial and retry sequences.

Preserve scratch evidence for diagnosis, but do not combine observations from
an invalidated session with a later valid session.

## Retention Decision

Retain B only when all of the following hold:

1. Every valid recorded A and B run passes and executes an identical file/test
   inventory.
2. No immediate `.svelte` file in `src/lib/components/research-projects/`
   imports from the root `@lucide/svelte` export.
3. Every Lucide import in that directory uses
   `@lucide/svelte/icons/*`; no local replacement facade is introduced.
4. The three B import trees no longer include
   `@lucide/svelte/dist/icons/index.js`.
5. The B complete-suite median is no more than 5% slower than A.
6. Focused research-project tests, static/type checks, and the final repository
   gate pass.

No minimum acceleration is required. Removing the barrel from all owned files
with statistically indistinguishable wall time is a valid result.

Predeclare one retry for criterion 5. If every other criterion passes but the
first B median is more than 5% and no more than 8% slower than A, run exactly
one additional A/B/A/B/A/B sequence. Pool the valid initial and retry data into
six observations per side and recompute medians. A first result above 8%, a
failure of another criterion, or a pooled B regression above 5% rejects the
candidate. No second retry is permitted.

If rejected, restore all 20 A files from the captured byte snapshots and
verify every baseline hash. Do not add the contract test. Commit only a
verification document recording the negative result and its limitations.

## Contract Protection

After a successful retention decision, add a focused source-contract test in
`src/lib/`. Use an eager Vite raw glob over
`./components/research-projects/*.svelte`, for example
`import.meta.glob(..., { query: "?raw", import: "default", eager: true })`, so
the directory's component sources participate in Vitest's dependency graph.
Normalize CRLF/LF before assertions.

The contract checks every discovered immediate Svelte file independently:

- no import source equals the root `@lucide/svelte` export;
- every Lucide package import starts with `@lucide/svelte/icons/`.

Before applying B, the new contract must fail and identify every current
root-import component. Implement this as an aggregate offender list followed
by `expect(offenders).toEqual([])`, rather than independent boolean assertions,
so the RED diff reports all violating paths in one run. After applying B it
must pass. The eager glob intentionally also covers components that do not use
Lucide and the two components migrated by the preceding slice; this protects
the directory boundary and automatically includes future immediate Svelte
files. The test must not hard-code file names, icon identifiers, the expected
offender count, or the mapping table. Ordinary feature work may add or remove
components and icons while preserving the directory boundary.

An ESLint `no-restricted-imports` rule remains the better future repository-wide
mechanism, but adding an ESLint stack is out of scope. The raw source contract
matches existing repository practice and protects exactly the retained
directory-level invariant.

## Final Verification

For a retained candidate:

1. demonstrate the contract's RED result against exact A, then restore exact B
   and verify all B hashes;
2. run the contract test on B;
3. run all tests under `src/lib/components/research-projects/` explicitly;
4. run `npm.cmd run check`;
5. confirm the measurement artifacts and retention calculation;
6. run `npm.cmd run verify` as the complete gate;
7. review the final diff and confirm application-code changes are limited to
   the 20 import blocks, plus the focused contract and evidence document.

An empty focused selection is a failure, not a pass. The final inventory may
increase after adding the contract test; this expected post-decision increase
is not compared with the pre-contract A/B inventory.

## Evidence and Deliverables

Create
`docs/superpowers/verification/2026-07-15-research-projects-lucide-direct-imports.md`
with:

- baseline commit, environment, versions, and configuration hashes;
- the exact 20-file A and B hash manifests and scratch location;
- warm-ups and recorded execution order;
- per-run wall times and file/test inventories;
- A and B medians, percentage delta, and any predeclared retry;
- representative test-file timings as diagnostic data;
- qualitative A/B import-tree summaries for the three selected roots;
- every retention criterion and its observed result;
- the retain/reject decision and remaining limitations.

If retained, the implementation is expected to use two commits: one focused
commit containing the 20 component imports and the directory contract, then a
separate evidence commit. If rejected, retain only the evidence commit.

## Risks and Limitations

- Full-suite timing remains noisy. Alternating variants and warm-ups reduce but
  do not remove workstation drift; the 5% threshold is a local non-regression
  boundary, not proof of acceleration.
- Shared test paths can overlap, so representative file durations are
  diagnostic rather than additive.
- The remaining 36 out-of-scope root imports keep the Lucide index elsewhere
  in the suite. This slice claims only the selected directory boundary.
- Package upgrades may rename direct exports. The candidate already avoids the
  four installed deprecated aliases in favor of their canonical modules; any
  later export change should be handled explicitly rather than bypassing the
  contract with a new barrel.
- Sequential byte-snapshot switching is not transactional. Hash verification
  and immediate A restoration turn a partial transition into an explicit
  infrastructure failure rather than a measured mixed variant.

## Follow-Up

Use the retained evidence to decide whether another bounded UI directory is
worth migrating. Do not expand the current candidate after observing results.
If later slices complete the repository-wide migration, replace the focused
source contracts with a centralized lint rule and remove the Lucide root
import everywhere in one separately reviewed enforcement change.
