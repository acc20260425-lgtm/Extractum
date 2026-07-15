# Lucide Direct Import Validation Design

## Status

Conversational design approved on 2026-07-15. Revised after written-spec
review; pending approval before implementation planning.

## Context

The development-loop profiling evidence in
`docs/superpowers/verification/2026-07-14-development-loop-performance-profiling.md`
identified `@lucide/svelte/dist/icons/index.js` as the largest measured import
in one instrumented Vitest run: approximately 14 seconds of self time and 16
seconds total. These single-run values establish an order of magnitude, not a
repeatable timing baseline. The slow import appeared in paths that included
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

This is intentionally a small experiment. At design revision time, 22 Svelte
files in `research-projects` imported the package root, so 20 remain outside
the two-file candidate, and many slow test paths traverse shared components
that continue to do so. A two-file change is therefore unlikely to move
complete-suite wall time beyond the workstation's observed run-to-run noise.
The sensitive evidence is the two target test-file durations and the
disappearance of the barrel from their import paths. Complete-suite wall time
is only a non-regression gate.

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

An ESLint `no-restricted-imports` rule is the preferred scalable enforcement
mechanism for a future broad migration, but this repository currently has no
ESLint dependency or configuration. Adding a lint stack for two files would
expand this experiment beyond its performance question. The retained
candidate therefore uses the repository's existing source-contract convention
and final diff review; a later migration may replace that contract with lint.

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

### Paired A/B Runs

Treat the unchanged components as A and the direct-import candidate as B.
Before measuring, record the baseline commit and byte hashes of both files,
create the candidate as a reversible patch stored outside the repository, and
verify that applying and reversing it produces the expected A and B hashes.

Run one discarded warm-up under A and one discarded warm-up under B. Then run
the recorded sequence A/B/A/B/A/B, capturing complete-suite JSON output and
wall time for every run. Apply or reverse only the owned candidate patch
between runs and verify the corresponding hashes each time. This alternating
order distributes gradual machine drift across both configurations; the
discarded warm-ups reduce cold-start and Defender/cache asymmetry.

For A and B separately, record the complete file/test inventory, the
complete-suite median, and the median duration of
`ProjectRailPanel.test.ts` and `SourcesTab.test.ts`. The target-file medians
are the primary quantitative metric. The complete-suite median is not used to
claim a measurable speedup from this narrow change.

After the recorded sequence, run one complete instrumented suite under A and
one under B and capture both import breakdowns. These single runs answer a
qualitative question: whether the target chains still attribute work to
`@lucide/svelte/dist/icons/index.js`. Do not use their duration difference as
a quantitative speedup estimate.

The import-duration mechanism follows the already validated profiling
protocol: first verify and try Vitest's installed
`--experimental.importDurations.print` option. If it is unavailable, rejected,
or produces no breakdown, temporarily add the documented equivalent to the
owned Vitest config, run the instrumented suite, then restore the config
byte-for-byte and verify its hash.

### Candidate Isolation

Do not add the contract test until the A/B decision is complete. A and B
performance runs must execute the same committed test inventory, and the two
component imports must be the only source difference between them.

Compare:

- median complete-suite wall time;
- all file and test counts;
- the two target test-file durations;
- the qualitative Lucide import entries and import paths;
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
4. The B instrumented target paths no longer pass through
   `@lucide/svelte/dist/icons/index.js`; no conclusion depends on the numeric
   difference between the single A and B import-duration runs.
5. The candidate complete-suite median is no more than 5% slower than the
   baseline median.
6. Targeted component tests and static/type checks remain green.

The 5% value is a non-regression tolerance for workstation noise, not a
required acceleration. Record the observed delta but require no minimum
speedup. Removing the measured barrel path with statistically indistinguishable
wall time is still a successful result because it reduces known import work
without a demonstrated regression.

If any retention condition fails, restore both owned component paths from the
recorded baseline commit with a path-scoped Git restore, verify their original
byte hashes, do not add the contract test, and commit only a verification
document that records the negative result and limitations. Do not manually
rewrite the imports or line endings during restoration.

## Contract Protection

After a successful retention decision, add
`src/lib/lucide-direct-import-contract.test.ts`. The focused source contract
reads the two Svelte files, normalizes CRLF/LF differences, and asserts:

- neither file imports from the root `@lucide/svelte` export;
- every Lucide package import present in those files uses an
  `@lucide/svelte/icons/*` direct path.

The contract intentionally covers only the two measured files. It must not
ban root imports repository-wide or turn this experiment into an unreviewed
mass migration. The exact mapping tables are implementation guidance, not a
permanent test inventory: ordinary feature work may add, remove, or rename an
icon without editing a list in the contract. The implementation scope and
final diff review, rather than a brittle attempt to infer icon semantics from
source text, enforce that this slice does not add a local facade.

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
- discarded warm-ups and the recorded A/B execution order;
- all raw-artifact temporary paths;
- per-run wall times, medians, inventories, and target-file timings;
- qualitative before/after import-duration excerpts or structured summaries;
- the calculated percentage delta;
- every retention criterion and its observed result;
- the final retain/reject decision and limitations.

If retained, the implementation deliverables are the two Svelte changes, the
focused contract, and the evidence document. If rejected, only the evidence
document is retained. The design and later implementation plan remain as the
audit trail in either case.

## Risks and Follow-Ups

- Warm-ups and alternating runs reduce but do not eliminate machine noise. The
  5% complete-suite tolerance is a local non-regression boundary, not evidence
  of a speedup, and noise may still reject a correct candidate.
- A package upgrade may change direct export paths. The contract will make
  that change explicit rather than silently falling back to a barrel.
- The two files account for only part of the measured Lucide index cost. The
  target-file medians and qualitative import-chain result are the sensitive
  measures; a broader migration requires separate evidence and review.
- If the barrel disappears but wall time remains unchanged, other expensive
  imports or test execution dominate the gate. Use the verification evidence,
  rather than expanding scope inside this slice, to choose a follow-up.
