# Lucide Direct Import Validation Design

## Status

Conversational design approved on 2026-07-15. Pending written-spec review
before implementation planning.

## Context

The development-loop profiling evidence in
`docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md`
identified `@lucide/svelte/dist/icons/index.js` as the largest measured import:
13.77 seconds of self time and 15.52 seconds total in the instrumented Vitest
run. The slow import appeared in paths that included
`ProjectRailPanel.svelte` and `SourcesTab.svelte`.

Both components currently import named icons from the package root:

```ts
import { Search, X } from "@lucide/svelte";
```

The installed `@lucide/svelte` 1.17.0 package exports individual icon modules
through `@lucide/svelte/icons/*`, and the repository already uses those direct
exports in other components. The package marks itself as side-effect-free, but
the measured Vitest import path still evaluates the generated icon index. This
slice tests whether replacing the two measured root imports removes that path
without changing UI behavior or making the complete suite slower.

## Goal

Validate a narrowly scoped direct-import optimization in the two measured
research-project components, retain it only when the barrel path disappears
and correctness and performance remain stable, and record reproducible A/B
evidence.

## Non-Goals

- No repository-wide Lucide import migration.
- No migration of other files in `research-projects`.
- No local icon facade or replacement barrel.
- No Vite, Vitest, SvelteKit, TypeScript, package, or lock-file change.
- No change to component markup, props, events, styling, icon names, or public
  behavior.
- No change to the `extractum-ui` barrel.
- No portable timing guarantee or minimum speedup assertion.
- No new domain string value and therefore no change to
  `docs/value-registry.md`.

## Selected Approach

Change only these files during the A/B candidate:

- `src/lib/components/research-projects/ProjectRailPanel.svelte`
- `src/lib/components/research-projects/SourcesTab.svelte`

Replace each named root import with a default import from the package's direct
icon export. Preserve the local component identifiers and every use site.

`ProjectRailPanel.svelte` uses:

| Local identifier | Direct module |
| --- | --- |
| `List` | `@lucide/svelte/icons/list` |
| `Plus` | `@lucide/svelte/icons/plus` |
| `RefreshCw` | `@lucide/svelte/icons/refresh-cw` |
| `Search` | `@lucide/svelte/icons/search` |
| `X` | `@lucide/svelte/icons/x` |

`SourcesTab.svelte` uses:

| Local identifier | Direct module |
| --- | --- |
| `Library` | `@lucide/svelte/icons/library` |
| `RefreshCw` | `@lucide/svelte/icons/refresh-cw` |
| `Download` | `@lucide/svelte/icons/download` |
| `Trash2` | `@lucide/svelte/icons/trash-2` |
| `X` | `@lucide/svelte/icons/x` |
| `Plus` | `@lucide/svelte/icons/plus` |

All eight installed module paths were verified during design. Direct imports
are the package's documented performance-oriented API and are already used in
the repository. A local facade is rejected because it could recreate the
barrel behavior being measured. A Vite optimization is rejected because it is
broader, less causal, and unnecessary for this package-supported path.

## Measurement Protocol

### Environment

Use the same Windows workstation and committed configuration for both sides.
Require a clean worktree before the baseline, record the commit and relevant
configuration hashes, and confirm no active Vitest, Vite, Tauri development,
or other benchmark process is competing for the machine. Run all measurements
sequentially. Record the active power scheme and Microsoft Defender real-time
protection state without changing either setting.

Store raw JSON reports and import-duration logs under an absolute system
temporary path outside the repository. Commit only the summarized evidence.

### Baseline

Before modifying either component:

1. Run the complete Vitest suite three times through the repository wrapper,
   capturing JSON output and wall time for each run.
2. Run one complete instrumented suite with import-duration reporting and
   capture the import breakdown.
3. Record the complete file/test inventory, the complete-suite median, the
   target component-test durations, and the import chain through
   `@lucide/svelte/dist/icons/index.js`.

The import-duration mechanism follows the already validated profiling
protocol: first verify and try Vitest's installed
`--experimental.importDurations.print` option. If it is unavailable, rejected,
or produces no breakdown, temporarily add the documented equivalent to the
owned Vitest config, run the instrumented suite, then restore the config
byte-for-byte and verify its hash.

### Candidate

Apply only the two direct-import edits and repeat the same three complete runs
and one instrumented run under the same conditions. Do not add the contract
test until the A/B decision is complete; baseline and candidate performance
runs must execute the same committed test inventory.

Compare:

- median complete-suite wall time;
- all file and test counts;
- the two target test-file durations;
- the Lucide import-duration entries and import paths;
- pass/fail and runtime errors.

Individual file and import durations overlap under parallel execution. They
are diagnostic evidence and must not be subtracted directly from wall time.

## Retention Decision

Retain the candidate only when all of the following hold:

1. Every baseline and candidate measurement run passes.
2. Candidate performance runs execute the same file and test inventory as the
   baseline. Live counts are authoritative; the previous 157-file and
   1,264-test observation is context, not a hard-coded assertion.
3. Both target components have no root import from `@lucide/svelte`.
4. The instrumented target paths no longer pass through
   `@lucide/svelte/dist/icons/index.js`.
5. The candidate complete-suite median is no more than 5% slower than the
   baseline median.
6. Targeted component tests and static/type checks remain green.

The 5% value is a non-regression tolerance for workstation noise, not a
required acceleration. Record the observed delta but require no minimum
speedup. Removing the measured barrel path with statistically indistinguishable
wall time is still a successful result because it reduces known import work
without a demonstrated regression.

If any retention condition fails, restore both component files byte-for-byte,
do not add the contract test, and commit only a verification document that
records the negative result and limitations.

## Contract Protection

After a successful retention decision, add
`src/lib/lucide-direct-import-contract.test.ts`. The focused source contract
reads the two Svelte files, normalizes CRLF/LF differences, and asserts:

- neither file imports from the root `@lucide/svelte` export;
- every identifier listed in the mapping tables uses its exact direct module;
- no local facade is substituted for the direct package path.

The contract intentionally covers only the two measured files. It must not
ban root imports repository-wide or turn this experiment into an unreviewed
mass migration.

Adding the contract after the performance decision increases the final test
inventory. That final inventory change is expected and is not compared with
the pre-contract A/B counts.

## Verification

For a retained candidate:

1. Run the new Lucide source contract.
2. Run `ProjectRailPanel.test.ts` and `SourcesTab.test.ts` explicitly.
3. Run `npm.cmd run check`.
4. Confirm the candidate measurement protocol and retention decision from the
   captured artifacts.
5. Run `npm.cmd run verify` as the final repository gate.
6. Review the final diff and confirm that application-code changes are limited
   to the two import blocks and that no package or configuration file changed.

Do not claim a pass from an empty related-test selection. Focused commands are
accelerators; the explicit tests and final gate remain authoritative.

## Evidence and Deliverables

Create
`docs/superpowers/verification/2026-07-15-lucide-direct-import-validation.md`
with:

- starting commit and environment details;
- baseline and candidate command lines;
- all raw-artifact temporary paths;
- per-run wall times, medians, inventories, and target-file timings;
- before/after import-duration excerpts or structured summaries;
- the calculated percentage delta;
- every retention criterion and its observed result;
- the final retain/reject decision and limitations.

If retained, the implementation deliverables are the two Svelte changes, the
focused contract, and the evidence document. If rejected, only the evidence
document is retained. The design and later implementation plan remain as the
audit trail in either case.

## Risks and Follow-Ups

- Three runs reduce but do not eliminate machine noise; the 5% tolerance is a
  local decision boundary, not a universal benchmark.
- A package upgrade may change direct export paths. The contract will make
  that change explicit rather than silently falling back to a barrel.
- The two files may account for only part of the measured Lucide index cost.
  A broader migration requires separate evidence and review.
- If the barrel disappears but wall time remains unchanged, other expensive
  imports or test execution dominate the gate. Use the verification evidence,
  rather than expanding scope inside this slice, to choose a follow-up.
