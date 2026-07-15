# Lucide Direct Import Validation Verification

## Scope and Starting State

The retained candidate changes only the Lucide imports in:

- `src/lib/components/research-projects/ProjectRailPanel.svelte`;
- `src/lib/components/research-projects/SourcesTab.svelte`.

The clean measurement session started from commit
`8ee2f310d3eafb81bdb6043eb69aca5a0d920d48` on `main`. Earlier scratch
sessions used while hardening the executable plan were discarded and were not
combined with these measurements. The measured test inventory was 157 files,
329 suites, and 1,264 tests for every A and B run.

## Environment

- OS: Microsoft Windows 11 Enterprise LTSC 10.0.26100.
- CPU inventory: 4 logical cores.
- Memory: 63.94 GiB.
- Power scheme: Balanced (`381b4222-f694-41f0-9685-ff5bb260df2e`).
- Microsoft Defender real-time protection: enabled.
- Node: v24.13.1.
- npm: 11.12.1.
- Vitest: 4.1.5 on win32-x64.

No competing Vitest, Vite, Tauri development, or Extractum process was active
when the session began. The power scheme and Defender state were recorded but
not changed.

## Candidate Patch

Variant A used root imports from `@lucide/svelte`. Variant B replaced only
those two import blocks with direct default imports from
`@lucide/svelte/icons/*`; markup, props, events, styles, and icon identifiers
were unchanged.

The reversible patch was proven by two apply/reverse cycles. Canonical hashes
used by the runners were:

| File | A SHA-256 | B SHA-256 |
| --- | --- | --- |
| `ProjectRailPanel.svelte` | `165E0368BF535A8B890B7C844DCE24C58B60C0DFB193C1341F6A96BF80FC4E4D` | `A6B90C4C66469263BBAF00223419982746A07A1CD042634A823727C2085E1E13` |
| `SourcesTab.svelte` | `F8D551876AB04BF621F7169D6E1089A6AE6A007C05FA175B09CB378A146037F2` | `E3E4B6A12BDE94881546B9EE067050B247CD68F23B854E93936286A6C55CDDC0` |

## Warm-Ups

Warm-ups were discarded from aggregates but required to pass:

| Variant | Wall time | Files / suites / tests | Result |
| --- | ---: | --- | --- |
| A | 71.949 s | 157 / 329 / 1,264 | pass |
| B | 67.552 s | 157 / 329 / 1,264 | pass |

## Recorded A/B Runs

The predeclared alternating order was A/B/A/B/A/B:

| Label | Variant | Wall time | `ProjectRailPanel.test.ts` | `SourcesTab.test.ts` | Inventory |
| --- | --- | ---: | ---: | ---: | --- |
| `recorded-01-A` | A | 67.510 s | 921.970 ms | 531.987 ms | 157 / 1,264 |
| `recorded-02-B` | B | 65.187 s | 979.874 ms | 643.301 ms | 157 / 1,264 |
| `recorded-03-A` | A | 67.361 s | 890.573 ms | 559.703 ms | 157 / 1,264 |
| `recorded-04-B` | B | 64.901 s | 1,013.995 ms | 564.124 ms | 157 / 1,264 |
| `recorded-05-A` | A | 66.540 s | 841.972 ms | 534.407 ms | 157 / 1,264 |
| `recorded-06-B` | B | 62.900 s | 970.706 ms | 587.436 ms | 157 / 1,264 |

Complete-suite medians were 67.361 s for A and 64.901 s for B. The observed B
delta was -3.652%. This is non-regression evidence, not a speedup claim.

## Target Test-File Medians

The sensitive per-file medians were:

| Test file | A median | B median |
| --- | ---: | ---: |
| `ProjectRailPanel.test.ts` | 890.573 ms | 979.874 ms |
| `SourcesTab.test.ts` | 534.407 ms | 587.436 ms |

These overlapping parallel-run durations are diagnostic only. The candidate
was retained because the known barrel path disappeared without a complete-suite
regression; no claim is made that the two individual test files became faster.

## Import Mechanism

Vitest 4.1.5 exposed both import-duration CLI options. The session used
`--experimental.importDurations.print` with a limit of 2,000 imports. No
temporary Vite configuration fallback was needed, and the final
`vite.config.js` SHA-256 remained
`D527A334BF9B42FAF25106EA46F0F68B5060A6BE898E5088678FC7C9FFA43696`.

## Target Import Trees

The qualitative target-root attribution was:

| Variant | Root | Rows | Contains `icons/index.js` |
| --- | --- | ---: | --- |
| A | `ProjectRailPanel.test.ts` | 52 | yes |
| A | `SourcesTab.test.ts` | 51 | yes |
| B | `ProjectRailPanel.test.ts` | 124 | no |
| B | `SourcesTab.test.ts` | 96 | no |

The A trees explicitly traversed
`node_modules/@lucide/svelte/dist/icons/index.js`. The B trees instead showed
direct modules such as `dist/icons/list.js` and `dist/icons/library.js`. Global
presence of the barrel through unrelated consumers was intentionally ignored.

## Retention Criteria

| Gate | Result | Evidence |
| --- | --- | --- |
| Every measurement run passed | pass | 2 warm-ups and 6 recorded runs exited 0 with nonempty inventories |
| Identical A/B inventory | pass | every recorded run was 157 files / 1,264 tests |
| No root import in the two B components | pass | source gate and later contract both passed |
| B target paths exclude `icons/index.js` | pass | both B attribution rows are `False` |
| B complete-suite median is no more than 5% slower | pass | delta -3.652% |
| Focused behavior and static checks | pass | focused exit 0; `npm.cmd run check` exit 0 |

## Retry Decision

No retry ran. The initial delta was below the +5% non-regression threshold, so
the predeclared `(5%; 8%]` retry condition was false.

## Final Decision

Decision: **retained** (`reason = protocol_completed`). All performance runs
passed and all source, import, inventory, timing, and correctness gates were
true. The code was committed as `9c32404a` (`perf: use direct lucide icon
imports`).

## Correctness Verification

The source contract was verified with a real RED/GREEN cycle:

- exact A: 2 contract cases failed on the root-import assertion;
- exact B: 2 contract cases passed;
- contract plus both existing component suites: 16 tests passed;
- `npm.cmd run check`: 0 errors and 0 warnings.

The final `npm.cmd run verify` gate passed:

- frontend: 158 test files and 1,266 tests passed;
- Svelte/type checking: 0 errors and 0 warnings;
- Rust formatting and `cargo check`: passed;
- Rust: 1,125 tests passed;
- final Git whitespace check: passed.

The final frontend inventory is two tests larger than the measurement inventory
because the retained source-contract file was deliberately added only after the
A/B decision.

## Limitations

- Complete-suite timing remains workstation-sensitive and could reject a
  correct narrow candidate in a noisier session.
- Single import-duration runs are qualitative attribution evidence, not a
  quantitative speedup comparison.
- The target-file durations overlap other parallel work and cannot be
  subtracted from suite wall time.
- Approximately 20 other `@lucide/svelte` root-import consumers remain outside
  this slice, so the global barrel is still present in the complete suite.
- Raw JSON reports, default-reporter logs, runner scripts, and full import trees
  remain in the external system temporary scratch and were not committed.
