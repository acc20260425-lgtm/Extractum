# Research Projects Lucide Direct Imports Verification

## Scope and Starting State

The experiment started from commit `2317e9dd0823393adfaef0b395ca6b562bb6b859`
on `main`. Variant A was the byte-for-byte baseline. Variant B changed only the
Lucide import blocks in the 20 Svelte components under
`src/lib/components/research-projects` that still imported the package root.

The candidate replaced package-root imports with direct canonical icon modules.
The deprecated aliases `alert-triangle`, `edit-3`, `play-circle`, and `x-circle`
were not introduced; the corresponding local identifiers use
`triangle-alert`, `pen-line`, `circle-play`, and `circle-x`.

The retained code was committed as `21410b04` (`perf: migrate research project
lucide imports`) together with a directory-wide raw-source contract.

## Environment

- OS: Microsoft Windows 11 Enterprise LTSC, version `10.0.26100`
- Logical CPU cores: 4
- Memory: 63.94 GiB
- Power scheme: Balanced (`381b4222-f694-41f0-9685-ff5bb260df2e`)
- Microsoft Defender real-time protection: enabled
- Node.js: `v24.13.1`
- npm: `11.12.1`
- Vitest: `4.1.5`, win32-x64
- Vite import-duration CLI flags: available, including the display limit flag

The warm-up and recorded measurements used the same machine state. The first
warm-up was intentionally excluded from the retention medians.

## Candidate and Snapshot Integrity

- Baseline A snapshots: 20 files
- Candidate B snapshots: 20 files
- Candidate patch size: 14,558 bytes
- A-to-B-to-A-to-B switching: all per-file SHA-256 checks passed
- Final retained files: all 20 matched their recorded B hashes
- `vite.config.js`: restored to its preflight SHA-256
- Repository scratch leakage: none; raw reports remained outside the repository

The source gate found no package-root Lucide import, no non-direct Lucide import,
and no deprecated direct-module path in the selected directory.

## Warm-Ups

| Variant | Wall time | Files | Tests | Result |
| --- | ---: | ---: | ---: | --- |
| A | 113.606 s | 158 | 1266 | passed |
| B | 60.979 s | 158 | 1266 | passed |

Warm-ups established the caches only; they were not included in the decision.

## Recorded A/B Runs

The recorded sequence was strictly interleaved: A, B, A, B, A, B.

| Run | Variant | Wall time | Files | Tests |
| --- | --- | ---: | ---: | ---: |
| `recorded-01-A` | A | 67.713 s | 158 | 1266 |
| `recorded-02-B` | B | 56.262 s | 158 | 1266 |
| `recorded-03-A` | A | 66.919 s | 158 | 1266 |
| `recorded-04-B` | B | 58.141 s | 158 | 1266 |
| `recorded-05-A` | A | 68.632 s | 158 | 1266 |
| `recorded-06-B` | B | 57.873 s | 158 | 1266 |

All six runs passed with the same live inventory. The A median was 67.713 s;
the B median was 57.873 s. The candidate delta was **-14.532%**.

## Representative Test-File Medians

| Test file | A median | B median | Direction |
| --- | ---: | ---: | --- |
| `ResearchProjectsShell.test.ts` | 434.517 ms | 423.513 ms | improved |
| `Inspector.test.ts` | 301.419 ms | 306.160 ms | +1.57%, within run noise |
| `RunDock.test.ts` | 215.138 ms | 187.735 ms | improved |

These file durations are diagnostic evidence. The retention timing gate used the
interleaved full-suite wall-time medians, not an individual file duration.

## Import Mechanism and Target Trees

Vitest 4.1.5 import-duration output was captured through the CLI with a display
limit of 2000. The full instrumented runs passed with 158 files and 1266 tests:

- A: 67.803 s
- B: 57.550 s

The global top-2000 output did not include every fast B root. Therefore the full
logs were preserved as complete-suite evidence and a second, scoped run of the
three representative tests was used only for qualitative import attribution:

- scoped A: 27.925 s
- scoped B: 13.328 s

| Variant | Root | Captured subtree rows | `icons/index.js` in target tree |
| --- | --- | ---: | --- |
| A | `ResearchProjectsShell.test.ts` | 1146 | yes |
| A | `Inspector.test.ts` | 365 | yes |
| A | `RunDock.test.ts` | 489 | yes |
| B | `ResearchProjectsShell.test.ts` | 1056 | no |
| B | `Inspector.test.ts` | 119 | no |
| B | `RunDock.test.ts` | 111 | no |

The timing values from the single instrumented runs are not used as quantitative
performance proof. Their purpose is to show that the target import chains no
longer traverse the Lucide package barrel.

## Retention Criteria

All six required gates passed:

1. All recorded performance runs passed.
2. Every recorded run used the same `158/1266` inventory.
3. The selected directory source gate passed.
4. The target import-attribution gate passed.
5. Focused correctness and static checks passed.
6. B was not more than 5% slower than A; it was 14.532% faster by median.

## Retry Decision

The one permitted repeat sequence applied only to a marginal +5% to +8%
regression. The initial delta was -14.532%, so `retry_required=false`; no repeat
runs were performed and no discretionary rerun was used.

## Final Decision

**Retained.** The protocol completed normally with exact B restored. The final
decision recorded three runs per variant, all performance runs passed, identical
inventories, and all timing, source, import, and correctness gates true.

## Correctness Verification

- Pre-decision focused tests on B: 18 files, 118 tests, passed
- Contract TDD RED on exact A: failed for the expected reason and named all 20
  package-root import offenders
- Contract GREEN on exact B: 1 file, 1 test, passed
- Post-contract focused tests: 19 files, 119 tests, passed
- `npm.cmd run check`: 0 errors, 0 warnings
- Final `npm.cmd run verify`:
  - Vitest: 159 files, 1267 tests, passed in 53.20 s
  - Svelte/TypeScript check: 0 errors, 0 warnings
  - Rust formatting check: passed
  - `cargo check`: passed
  - Rust tests: 1125 passed, 0 failed, finished in 18.98 s
  - final whitespace check: passed

## Remaining Scope and Limitations

There are 36 package-root Lucide import consumers elsewhere under `src`. They
were intentionally excluded from this directory-scoped migration and remain a
candidate for later slices.

The measured wall times are local-machine evidence, not portable performance
thresholds. Defender was enabled and the Balanced power profile was active. The
single-run import-duration numbers are qualitative attribution evidence only;
the interleaved uninstrumented medians are the quantitative retention evidence.
