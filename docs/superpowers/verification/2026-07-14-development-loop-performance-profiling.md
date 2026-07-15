# Development Loop Performance Profiling Verification

## Scope and Starting State

The measurement slice started from clean `main` at commit
`92dc0fa2a4fba2d2c086f6c4c12a6dbaeebf3a67`. It measured the existing
Vitest, Cargo, and Rust test behavior without committing any application,
test, build, or workflow change.

During execution, four defects in the measurement plan were corrected before
their derived data was used:

- the three-value PowerShell median index now uses `Floor` instead of rounding
  `1.5` to `2`;
- CSV decimal values are parsed with the current Windows culture;
- Vitest file inventory uses `testResults.length`, while
  `numTotalTestSuites` is retained separately as the suite count;
- Cargo timing JSON arrays are explicitly enumerated, and the temporary Rust
  probe restores the repository's CRLF bytes before checking SHA-256.

Successful raw measurements were retained across these documentation-only
corrections. No profiler or Cargo subcommand was installed. `cargo-nextest`
was not present and was not installed. All raw JSON, CSV, logs, and Cargo HTML
remain outside version control or in the existing ignored canonical
`src-tauri/target` directory.

## Environment

| Property | Observed value |
|---|---|
| OS | Microsoft Windows 11 Enterprise LTSC 10.0.26100 |
| Logical cores | 4 |
| Memory | 63.94 GiB |
| Power scheme | Balanced (`381b4222-f694-41f0-9685-ff5bb260df2e`) |
| Defender real-time protection | Enabled |
| Node | v24.13.1 |
| npm | 11.12.1 |
| Vitest | 4.1.5, win32-x64 |
| rustc | 1.95.0 (`59807616e`, 2026-04-14) |
| Cargo | 1.95.0 (`f2d3ce0bd`, 2026-03-21) |

Measurements were run sequentially with no active Cargo, rustc,
rust-analyzer, Tauri-dev, Vitest, or Extractum process. Machine timings in
this document are local observations, not portable thresholds.

## Vitest Baseline Distribution

All three complete runs passed the same inventory: 157 files, 329 internal
suites, and 1,264 tests.

| Run | Wall time | Files | Suites | Tests | Result |
|---:|---:|---:|---:|---:|---|
| 1 | 127.701 s | 157 | 329 | 1,264 | passed |
| 2 | 85.019 s | 157 | 329 | 1,264 | passed |
| 3 | 73.992 s | 157 | 329 | 1,264 | passed |

Run 1 enabled the documented CLI import-duration option, although that path
did not print a breakdown. Its extra instrumentation makes its wall time less
representative than runs 2 and 3. The three-run wall median is 85.019 s.

The per-file distribution contains 138 `node/default` files and 19 `jsdom`
files:

| Percentile | File duration |
|---|---:|
| p50 | 12.543 ms |
| p90 | 230.606 ms |
| p95 | 489.794 ms |

Top ten three-run file medians:

| File | Environment | Median | Run 1 | Run 2 | Run 3 |
|---|---|---:|---:|---:|---:|
| `src/lib/components/research-projects/ProjectRailPanel.test.ts` | jsdom | 1,104.277 ms | 1,034.852 | 1,318.605 | 1,104.277 |
| `src/lib/components/research-projects/ProjectRow.test.ts` | jsdom | 729.467 ms | 729.467 | 814.259 | 727.175 |
| `src/lib/components/research-projects/SourcesBulkBar.test.ts` | jsdom | 701.619 ms | 701.619 | 865.904 | 606.928 |
| `src/lib/components/research-projects/SourcesTab.test.ts` | jsdom | 621.884 ms | 621.884 | 709.318 | 592.650 |
| `src/lib/components/research-projects/ProjectToolbar.test.ts` | jsdom | 557.583 ms | 556.268 | 557.583 | 582.120 |
| `sidecars/gemini-browser/src/answer-extractor.test.ts` | node/default | 553.552 ms | 553.552 | 516.714 | 697.531 |
| `src/lib/components/research-projects/ResearchProjectsShell.test.ts` | jsdom | 515.532 ms | 499.698 | 644.548 | 515.532 |
| `src/lib/components/research-projects/SourcesFilterRow.test.ts` | jsdom | 489.794 ms | 474.490 | 546.508 | 489.794 |
| `src/lib/components/research-projects/PeriodPopover.test.ts` | jsdom | 420.207 ms | 397.146 | 459.191 | 420.207 |
| `src/lib/components/research-projects/ComboSelect.test.ts` | jsdom | 410.307 ms | 410.307 | 421.721 | 374.158 |

## Vitest Import Evidence

The installed CLI advertised `--experimental.importDurations.print`, but the
instrumented baseline member emitted no breakdown. The reversible
`vite.config.js` fallback succeeded in 75.301 s with the complete passing
inventory. The original config SHA-256
`D527A334BF9B42FAF25106EA46F0F68B5060A6BE898E5088678FC7C9FFA43696`
was restored before the result was interpreted.

The fallback reported 1,570 imports, 77.42 s aggregate self time, and 701.44 s
aggregate total time. The latter is overlapping module-graph time, not wall
time. Its top findings were:

| Module | Self | Total |
|---|---:|---:|
| `ProjectRailPanel.test.ts` | 336 ms | 24.35 s |
| `ProjectRailPanel.svelte` | 33 ms | 24.02 s |
| `@lucide/svelte/dist/lucide-svelte.js` | 856 ms | 22.10 s |
| `@lucide/svelte/dist/icons/index.js` | 13.77 s | 15.52 s |
| `SourcesTab.test.ts` | 34 ms | 11.02 s |
| `SourcesTab.svelte` | 22 ms | 10.99 s |
| `@lucide/svelte/dist/lucide-svelte.js` (SourcesTab path) | 27 ms | 9.26 s |
| `ProjectRow.test.ts` | 243 ms | 10.71 s |
| `ProjectRow.svelte` | 175 ms | 10.47 s |
| `src/lib/components/extractum-ui/index.ts` | 1.00 s | 10.29 s |

This is a concrete shared-import hypothesis: multiple files that remain in the
three-run slow tail traverse the `@lucide/svelte` barrel, including the
expensive `dist/icons/index.js` module.

## Conditional Isolation Experiment

The mechanical p90 node/default candidate contained
`answer-extractor.test.ts` and `adapter.test.ts`. Static review excluded
`adapter.test.ts` because it writes to a shared relative `artifacts/` tree.
`answer-extractor.test.ts` owns and closes its Chromium process and was retained
as the safe subset.

| Qualification run | Wall time | Files | Suites | Tests | Result |
|---:|---:|---:|---:|---:|---|
| 1 | 4.695 s | 1 | 2 | 6 | passed |
| 2 | 4.545 s | 1 | 2 | 6 | passed |
| 3 | 4.416 s | 1 | 2 | 6 | passed |

The normal-isolation median was 4.545 s, below the approved 10-second noise
floor. The alternating 3+3 `isolate`/`no-isolate` experiment was therefore
skipped. No isolation saving is claimed.

## Cargo Cold Report

Existing report:
`src-tauri/target/cargo-timings/cargo-timing-20260714T180231751Z-a54253738dfaee23.html`.
It represents a cold profile-triggered build after the approved cache cleanup,
not the daily incremental shape.

| Metric | Value |
|---|---:|
| Cargo duration | 235.00 s |
| Timing units | 731 |
| Maximum active | 5 |
| Maximum waiting | 76 |
| Samples with waiting work | 1,386 |

| Unit | Target | Duration | Start |
|---|---|---:|---:|
| windows 0.61.3 | check | 42.82 s | 111.51 s |
| tauri-utils 2.8.3 | default | 27.35 s | 84.69 s |
| extractum 0.2.0 | check | 26.22 s | 208.31 s |
| grammers-tl-types 0.9.0 | check | 18.89 s | 175.76 s |
| zstd-sys 2.0.16 | build-script run | 12.32 s | 175.41 s |
| tauri 2.10.3 | check | 9.92 s | 165.70 s |
| syn 2.0.117 | default | 9.76 s | 21.00 s |
| syn 1.0.109 | default | 9.75 s | 50.41 s |
| tokio 1.52.1 | check | 9.09 s | 83.88 s |
| windows-sys 0.61.2 | check | 8.57 s | 38.39 s |

## Cargo No-Op Control

| Run | Wall time | Result |
|---:|---:|---|
| 1 | 18.553 s | passed |
| 2 | 1.078 s | passed |
| 3 | 1.037 s | passed |

The median no-op wall time was 1.078 s. The first control incurred additional
one-time work and remains recorded rather than silently retried.

## Cargo Small-Edit Report

The inert root-crate comment produced one ignored timing report:
`src-tauri/target/cargo-timings/cargo-timing-20260715T021240513Z-a54253738dfaee23.html`.

| Metric | Value |
|---|---:|
| Command wall time | 39.710 s |
| Cargo duration | 40.000 s |
| Maximum active | 1 |
| Maximum waiting | 1 |
| Samples with waiting work | 1 |
| `extractum` library check | 38.260 s |
| `extractum` binary check | 0.320 s |

The dominant work was the root `extractum` library frontend (37.26 s) plus
codegen (1.00 s). This cost is absent from the 1.078 s median no-op control and
is therefore a real incremental source-edit candidate, not an inference from
the cold build.

The inert comment was removed, `runtime_config.rs` was normalized back to its
original CRLF representation, and SHA-256
`F73719D912CB205A16A66B280AFBDB8B9E4627865CEA8D3B4992BE346702CA35`
was restored before Rust test profiling.

## Rust Full and Sequential Harness

The exact inventory contained 1,125 unique library tests.

| Shape | Run | Wall time | Harness time | Tests | Result |
|---|---:|---:|---:|---:|---|
| parallel | 1 | 19.428 s | 18.360 s | 1,125 | passed |
| parallel | 2 | 19.110 s | 18.030 s | 1,125 | passed |
| parallel | 3 | 19.443 s | 18.390 s | 1,125 | passed |
| sequential | 1 | 37.110 s | 36.050 s | 1,125 | passed |

Parallel execution roughly halves the observed harness floor; the remaining
parallel median is 18.36 s.

## Rust Module Groups

Top-level exact inventory and valid timings:

| Module | Expected | Actual | Wall s | Harness s | Shape |
|---|---:|---:|---:|---:|---|
| account_deletion | 9 | 9 | 1.094 | 0.03 | filtered |
| accounts | 5 | 5 | 1.031 | 0.01 | filtered |
| analysis | 143 | 143 | — | — | exact-list inventory only |
| analysis_documents | 5 | 5 | 1.056 | 0.04 | filtered |
| apalis_jobs | 16 | 16 | 1.111 | 0.11 | filtered |
| archive_read_model | 4 | 4 | 1.025 | 0.02 | filtered |
| child_process | 1 | 1 | 1.015 | 0.00 | filtered |
| compression | 3 | 3 | 1.012 | 0.00 | filtered |
| diagnostics | 11 | 11 | — | — | exact-list inventory only |
| error | 8 | 8 | 1.017 | 0.00 | filtered |
| external_process | 12 | 12 | 1.022 | 0.02 | filtered |
| gemini_browser | 94 | 94 | 4.285 | 3.21 | filtered |
| ingest_provenance | 7 | 7 | 1.067 | 0.04 | filtered |
| job_helpers | 3 | 3 | 0.998 | 0.00 | filtered |
| library_sources | 4 | 4 | 1.017 | 0.01 | filtered |
| llm | 51 | 51 | 1.637 | 0.62 | filtered |
| media | 5 | 5 | — | — | exact-list inventory only |
| migrations | 30 | 30 | 3.396 | 2.35 | filtered |
| notebooklm_export | 76 | 76 | 1.251 | 0.21 | filtered |
| process_tree | 7 | 7 | 1.679 | 0.65 | filtered |
| projects | 23 | 23 | 2.822 | 1.80 | filtered |
| prompt_packs | 225 | 225 | 9.813 | 8.80 | filtered |
| readiness | 4 | 4 | 1.097 | 0.00 | filtered |
| secret_store | 3 | 3 | 1.007 | 0.00 | filtered |
| source_ingest | 4 | 4 | 1.006 | 0.00 | filtered |
| sources | 150 | 150 | — | — | exact-list inventory only |
| sql_helpers | 1 | 1 | 1.010 | 0.01 | filtered |
| takeout_import | 73 | 73 | 1.149 | 0.14 | filtered |
| telegram | 5 | 5 | 1.018 | 0.01 | filtered |
| telegram_session_store | 7 | 7 | 1.038 | 0.04 | filtered |
| time | 7 | 7 | — | — | exact-list inventory only |
| topic_memberships | 4 | 4 | 1.108 | 0.02 | filtered |
| tx | 8 | 8 | 1.083 | 0.06 | filtered |
| youtube | 117 | 117 | 2.187 | 1.18 | filtered |

Substring collisions affected `analysis`, `diagnostics`, `media`, `sources`,
and `time`. One exact full-name chunk per group validated 316 tests total;
their process timings are intentionally not compared with filtered groups.

`prompt_packs` exceeded 25% of the 18.36 s parallel median, so it was divided
one level further:

| Module | Tests | Wall s | Harness s |
|---|---:|---:|---:|
| prompt_packs::youtube_summary | 125 | 6.287 | 5.28 |
| prompt_packs::runtime | 40 | 2.226 | 1.22 |
| prompt_packs::result_builder | 11 | 1.960 | 0.95 |
| prompt_packs::projections | 5 | 1.486 | 0.48 |
| prompt_packs::seed | 5 | 1.493 | 0.46 |
| prompt_packs::stage_io | 3 | 1.317 | 0.26 |
| prompt_packs::validation | 25 | 1.244 | 0.24 |
| prompt_packs::store | 2 | 1.175 | 0.17 |
| prompt_packs::library | 1 | 1.130 | 0.13 |
| prompt_packs::completion_transport | 2 | 1.012 | 0.01 |
| prompt_packs::gemini_browser_stage | 3 | 1.004 | 0.00 |
| prompt_packs::dto | 2 | 1.014 | 0.00 |
| prompt_packs::stage_output_normalization | 1 | 1.003 | 0.00 |

## Rust Static Hypotheses

These are source-level hypotheses, not demonstrated causes:

- `prompt_packs::youtube_summary` contributes 5.28 s of the 8.80 s
  `prompt_packs` harness time. Its test support repeatedly constructs
  `sqlite::memory:` pools and seeds migrated run/source/transcript fixtures.
- `migrations` contributes 2.35 s and contains both many in-memory pools and a
  concurrent file-backed migration test with a 30-second guard.
- `gemini_browser` contributes 3.21 s and contains numerous temporary
  directories, SQLite-backed job fixtures, loopback listeners, and bounded
  timeout paths.

The evidence supports a focused test-fixture/setup profile before changing any
database strategy or timeout semantics.

## Limitations

- Defender was enabled and the Balanced power plan was active; background OS
  activity was not controlled beyond rejecting known build/test processes.
- The first Vitest and first Cargo no-op runs were slower than later runs and
  were retained as observed rather than retried.
- Vitest import totals overlap through the module graph and must not be added
  as wall-clock savings.
- The isolation experiment was not run because its only safe subset was below
  the approved noise floor.
- Rust group runs include about one second of repeated Cargo process startup;
  exact-list fallback timings are not comparable to normal filtered groups.
- Static scans identify candidate mechanisms but do not prove causality.
- Plan corrections changed only documentation and measurement parsing. The
  successful raw baselines and test runs were reused; no product behavior was
  changed.

## Selected Outcome

**Primary outcome: validate the Vitest shared-import hypothesis.**

The next slice should test whether replacing broad `@lucide/svelte` barrel
imports in the measured slow research-project components with supported direct
icon imports reduces import duration and full-suite wall time without changing
rendered behavior or public component APIs. This is selected because the
import breakdown shows one concrete shared expensive module across multiple
files that remain in the three-run slow tail, and the validation is narrower
and cheaper than restructuring the Rust crate.

This is an import-only recommendation. It does **not** claim saved seconds yet;
the follow-up must repeat comparable full-suite and import-duration runs.

## Rejected Outcomes

- **Commit `isolate: false`: rejected.** The safe subset median was only
  4.545 s, below the noise floor, so no A/B evidence exists.
- **Incremental Cargo as the primary follow-up: not selected, but retained as
  the strongest secondary candidate.** A small root-crate edit costs 39.710 s
  versus a 1.078 s no-op median, with 38.260 s in `extractum` check. The cost is
  material, but the evidence does not yet isolate a cheap structural remedy.
- **Rust-test optimization as the primary follow-up: not selected, but retained
  as a secondary candidate.** `prompt_packs::youtube_summary` is a concrete
  5.28 s group with a repeated SQLite-fixture hypothesis, but validating the
  shared frontend import is the cheaper first experiment.
- **Stop optimizing: rejected.** Both the shared Vitest import and incremental
  root-crate cost are material, evidence-backed candidates.

At completion, `vite.config.js` and `runtime_config.rs` matched their starting
SHA-256 values, the worktree was clean before this document was added, no
`codex-*` target existed, and no measurement tool was installed.
