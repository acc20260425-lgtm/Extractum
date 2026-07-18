# Rust Crate Extraction Roadmap

**Status:** Strategic reference, approved in conversation
**Date:** 2026-07-17

## Purpose

This document is the strategic layer above the per-slice specs and plans. It
records the evidence, the target crate map, the phase order, and the standing
rules that every future extraction slice inherits. It authorizes nothing by
itself: each phase requires its own spec and implementation plan, a fresh
dependency map, and fresh co-change statistics before execution.

Completed and in-flight slices governed by their own documents:

- [`2026-07-17-process-and-gemini-browser-crate-boundary-design.md`](2026-07-17-process-and-gemini-browser-crate-boundary-design.md)
  — approved just-in-time boundary for phase 3 `extractum-process` and the
  phase 4 focused Gemini Browser engine. The Phase 3 implementation plan is
  [`2026-07-17-extractum-process-extraction.md`](../plans/2026-07-17-extractum-process-extraction.md);
  execution was measured and not retained; see the verification record linked
  in Phase 3.

- `2026-07-15-rust-workspace-crate-extraction-design.md` — workspace +
  `extractum-core` (`error`, `time`, `compression`). Done.
- `2026-07-16-media-metadata-core-boundary-design.md` — pure
  `media_metadata` into core. Done.
- `2026-07-17-notebooklm-render-crate-boundary-design.md` — Stage 0
  surrogate preflight, then conditional `extractum-notebooklm-render`.
  Closed 2026-07-17 with a `no_go` preflight (commit `1cf1485b`): the
  full-workspace hypothesis is falsified; the crate was not created. See
  `docs/superpowers/verification/2026-07-17-notebooklm-render-crate-boundary.md`.
- [`2026-07-17-focused-rust-loop-design.md`](2026-07-17-focused-rust-loop-design.md)
  — focused package commands, extraction-retention thresholds, plan-shape
  requirements, and unchanged end-of-slice workspace gates. Enforced through
  `AGENTS.md` and `src/lib/focused-rust-loop-contract.test.ts`.
- [`2026-07-18-crate-extraction-shell-cap-revision-design.md`](2026-07-18-crate-extraction-shell-cap-revision-design.md)
  — current 2,000 ms / 20% per-slice cap, 15,000 ms cumulative ceiling,
  measurement-validity rule, and Phase 3/v2 disposition.

## Evidence Base

### Commit-history analysis

587 backend commits (`src-tauri/src`, 2026-04-20 through 2026-07-17) were
mapped to top-level modules. 384 commits (65%) touch exactly one module: the
domain boundaries are already respected by development discipline, so crate
boundaries mostly formalize existing seams rather than invent them.

Activity and autonomy per module (solo = commits touching only that module;
recent = commits in the 30 days before 2026-07-17):

| Module | Commits | Solo | Solo % | Recent 30d | Last commit | Lines (approx) |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| `sources` | 139 | 51 | 37% | 3 | 2026-07-06 | 11,700 |
| `analysis` | 133 | 86 | 65% | 40 | 2026-07-11 | 13,200 |
| `prompt_packs` | 116 | 92 | 79% | 50 | 2026-07-14 | 19,000 |
| `lib` | 113 | 6 | 5% | 29 | 2026-07-16 | shell |
| `youtube` | 59 | 15 | 25% | 16 | 2026-07-12 | 10,800 |
| `migrations` | 54 | 20 | 37% | 5 | 2026-07-12 | schema |
| `takeout_import` | 49 | 22 | 45% | 4 | 2026-07-11 | 7,700 |
| `llm` | 46 | 19 | 41% | 10 | 2026-07-12 | 4,700 |
| `notebooklm_export` | 40 | 20 | 50% | 0 | 2026-05-31 | 5,900 |
| `gemini_browser` | 39 | 23 | 59% | 39 | 2026-07-12 | 6,800 |
| `telegram` | 23 | 5 | 22% | 2 | 2026-06-24 | client |

The hot development frontier is `prompt_packs`, `analysis`, and
`gemini_browser` (129 of the last ~30 days' commits). `notebooklm_export` is
dormant. `telegram` and its account infrastructure are dormant. `lib` changes
with every domain because it registers commands; its churn is shell noise, not
coupling, and is never used as a grouping signal.

Strongest co-change pairs (commits touching 2–6 modules; `lib` and
`migrations` excluded as global noise):

| Pair | Joint commits | Reading |
| --- | ---: | --- |
| `sources` + `takeout_import` | 18 | ingest web |
| `notebooklm_export` + `sources` | 16 | historical, during NotebookLM development |
| `sources` + `youtube` | 15 | ingest web; do not separate across crates |
| `analysis` + `sources` | 13 | analysis reads corpus read-models |
| `analysis` + `llm` | 12 | analysis drives LLM calls |
| `prompt_packs` + `gemini_browser` | 7 | thin, despite runtime dependency |
| `llm` + `telegram` | 6 | shared client/auth concerns |
| `accounts` + `telegram` | 5 | one account cluster |

### Dependency findings (verified in source)

- Foundation modules were leaf-clean and are now in `extractum-core`
  (`error`, `time`, `compression`, `media_metadata`).
- `sources ↔ youtube` module cycle is asymmetric: `sources` consumes only
  `youtube::dto` types; `youtube` consumes `sources` behavior. The declared
  fix (move the DTOs into `sources`) is deferred until either domain becomes a
  crate.
- The four `grammers-*` crates are git dependencies pinned to a Codeberg
  revision. Their users are `telegram.rs`, the media adapter half of
  `media.rs`, and session storage. Removing them from the application crate is
  a standalone cold-build and dependency-hygiene win.
- `analysis/trace.rs` calls `zstd::` directly, bypassing
  `core::compression` (recorded backlog item).
- `sources::test_support` is consumed by 10+ modules across domains; any
  producer-domain extraction must resolve test-fixture ownership first
  (dev-dependencies cannot form cycles).

### Measured compile-time evidence so far

From the workspace/core verification (2026-07-15, no-bundle protocol):

- 1,125 baseline Rust tests (1,126 after the media slice); no-op workspace
  check ≈ 1.0 s; app-domain probe 7,608 ms; app-shell probe 7,592 ms.
- After the core extraction the same probes measured 7,632 ms (+0.32%) and
  7,634 ms (+0.55%): the first split did **not** accelerate edits to files
  that stayed in the application crate. This is expected — the application
  crate itself did not shrink meaningfully — and it is why the render slice
  begins with a Stage 0 preflight instead of another extraction on faith.
- Full MSI bundling is excluded from all gates (documented pre-existing WiX
  `light.exe` failure); release evidence uses
  `npm.cmd run tauri -- build --no-bundle` plus a startup smoke.

## The Two Compile Loops

Every phase must state which loop it claims to improve. They are different
products:

1. **Full workspace loop** —
   `cargo check --manifest-path src-tauri/Cargo.toml --workspace
   --all-targets`. Editing an extracted dependency still re-checks every crate
   above it, including the large application crate. An extraction improves
   this loop only if it materially shrinks what the application crate itself
   recompiles. The Stage 0 surrogate in the render spec measures exactly this
   claim against the already-extracted `extractum-core`.
2. **Focused package loop** — `cargo check --manifest-path
   src-tauri/Cargo.toml -p <crate> --all-targets` and focused package tests.
   This loop benefits from extraction almost by definition (the domain stops
   paying for the application crate). Its use as the acceptance metric for
   hot-module phases 4–6 is governed by
   `2026-07-17-focused-rust-loop-design.md`; it never replaces a failed
   full-workspace gate.

The commit-history ROI argument (hot modules gain the most) applies fully to
loop 2 and only conditionally to loop 1. This distinction is the roadmap's
main branch point.

## Decision Framework

- Byte-for-byte probe restoration, warmed median sampling, session
  invalidation on infrastructure failure, and documented negative outcomes
  are inherited as common mechanics from the render spec. For hot-module
  phases 4–6, the focused-loop spec is the normative source for commands,
  thresholds, and failure classification.
- For decisions after 2026-07-18, hot-module retention and shell-regression
  gates come from `2026-07-18-crate-extraction-shell-cap-revision-design.md`
  and the updated focused-loop specification. Historical sessions retain their
  originally frozen thresholds.
- Every slice must pass both its 2,000 ms / 20% local shell cap and the
  15,000 ms cumulative shell ceiling. A ceiling crossing requires a separate
  owner-approved policy revision before retention.
- **Branch on the render slice outcome:**
  - **Stage 0 go, Stage 1 retain** — the full-workspace hypothesis holds on
    this machine. Continue the phase sequence below with the same gates.
  - **Stage 0 no-go (or Stage 1 revert)** — the full-workspace hypothesis is
    falsified for this repository shape. Before any further extraction,
    choose explicitly, with a user-approved spec revision, one of:
    (a) adopt the focused package loop as the acceptance metric for hot-module
    phases and recalibrate thresholds for it;
    (b) proceed on architectural justification only (boundary enforcement,
    dependency hygiene, grammers removal), with performance recorded as
    diagnostic evidence, not a gate;
    (c) stop extracting and keep the workspace at core + retained slices.
- **Outcome recorded 2026-07-17:** Stage 0 returned `no_go`
  (app 9,090 ms vs core surrogate 9,100 ms, −0.11%; focused core 1,020 ms).
  The project owner selected **(a) + (b)**: the focused package loop becomes
  the acceptance metric for the hot-module phases 4–6 under the approved
  focused-loop respec, and phase 8 proceeds on
  dependency-hygiene justification with performance as diagnostic evidence.
  Development verification in this repository is performed by an LLM agent
  following Superpowers plans, so the focused-loop workflow is adopted through
  mandatory `AGENTS.md` policy and generated-plan structure. Universal
  Superpowers skills remain unchanged unless later evidence shows systematic
  policy violations.
  Full workspace gates remain unchanged as end-of-slice verification.
  - A failed gate never justifies weakening correctness gates, and a
    reverted candidate leaves no partial split behind.

## Roadmap Shell Budget

The cumulative ceiling is measured on the same workstation/probe family as the
canonical pre-Phase 3 shell median. Only valid series enter this ledger.

| Checkpoint | Valid shell median | Remaining to 15,000 ms | Disposition |
| --- | ---: | ---: | --- |
| Pre-Phase 3 | 9,135 ms | 5,865 ms | canonical anchor |
| Reapplied Phase 3 | pending valid post-reapplication median | pending | non-gating measurement required before Phase 4 timing |

## Target Crate Map (end state, revisable per phase)

```text
extractum (app shell: commands, state, db, migrations, apalis_jobs,
           diagnostics, cross-domain integration tests)
  ├── extractum-prompt-packs      (phase 6)
  │     ├── extractum-gemini-browser   (phase 4)
  │     ├── extractum-llm              (phase 5)
  │     └── extractum-core
  ├── extractum-analysis          (phase 7)
  │     ├── extractum-llm
  │     └── extractum-core
  ├── extractum-telegram          (phase 8: telegram, accounts,
  │     └── extractum-core         session store, secret store,
  │                                grammers media adapter)
  ├── extractum-notebooklm-render (phase 2, conditional on gates)
  │     └── extractum-core
  ├── extractum-process           (phase 3, if justified: external_process,
  │     └── extractum-core         child_process, process_tree)
  ├── extractum-sources           (phase 9+: storage/read-model layer)
  │     └── extractum-core
  └── extractum-core              (done: error, time, compression,
                                   media_metadata; sql_helpers/tx by trigger)
```

Recorded decisions (2026-07-17, project owner):

1. Order after the render slice follows hot-module ROI:
   `gemini_browser` → `llm` → `prompt_packs`.
2. The prompt-pack stack is layered as three crates, not one vertical crate.
3. The telegram cluster is extracted after the hot modules, primarily to
   remove the grammers git dependencies from the application crate.
4. The ingest web (`sources`, `youtube`, `takeout_import`, satellites) is
   eventually decomposed too — gradually, bottom-up, and only as its API
   stabilizes; co-change evidence (18/15/13 joint commits) makes it the last
   and most cautious phase, with an explicit stop option if cross-crate
   paired edits become frequent.

## Phase Sequence

Each phase = fresh co-change/dependency snapshot → spec → plan → execution
with gates. Phases are not started concurrently.

### Phase 0 — Workspace + `extractum-core` (done)

`error`, `time`, `compression`; wrapper-module facades preserve `crate::`
paths; profiles stay in the workspace root; canonical target proven.

### Phase 1 — `media_metadata` into core (done)

Pure model/codec/label split from the grammers adapter; rename-map inventory
protocol established.

### Phase 2 — `extractum-notebooklm-render` (closed: negative preflight)

Seven pure modules (~1,593 lines, 22 tests) behind a Stage 0 surrogate
preflight and Stage 1 retention gates. The gatekeeper experiment for the
full-workspace hypothesis returned `no_go` on 2026-07-17: editing the
already-extracted `extractum-core` was not faster than editing application
code under the full workspace check (−0.11%, both gates missed by orders of
magnitude), while the focused package check measured ~1.0 s vs ~9.1 s. No
crate was created; the branch selected in "Decision Framework" governs all
later phases. The seven-module boundary remains a valid future candidate
under the focused-loop metric if a phase ever needs it.

### Phase 3 — `extractum-process` (approved for exact-candidate reapplication)

`external_process`, `child_process`, and `process_tree` become shared
OS-process infrastructure below Gemini Browser, YouTube, and diagnostics.
`job_helpers` stays app-side. Candidate `b364756c` improved the focused median
from 9,171 ms to 2,049 ms and regressed the app-shell median from 9,135 ms to
10,177 ms (+1,042 ms / +11.41%). It correctly failed the original 500 ms / 5%
gate and was reverted in `c47372dc`; that historical result remains unchanged.

The approved 2026-07-18 shell-cap revision accepts the same evidence under the
new 2,000 ms / 20% rule. Reapply the exact historical candidate without a new
gating performance experiment, verify its historical tree/blob identity, and
rerun current correctness and completion gates. Record non-gating before/after
shell samples and validity counts; a valid post median seeds the cumulative
ledger. If those diagnostics are invalid, Phase 3 may still retain after all
correctness/completion gates pass, but Phase 4 must establish a valid baseline
before its own performance decision.

The v2/v3 anomaly track is moot for this roadmap. It is not a prerequisite for
Phase 3 or Phase 4, and its reviewed harness remediation remains deferred.

If reconstruction differs materially from `b364756c`, stop the exact-candidate
path. The result is a new candidate and requires a separately approved plan
with fresh preregistered timing under the revised local, cumulative, and
validity rules.

### Phase 4 — `extractum-gemini-browser` (boundary approved)

~6,800 lines, 39 commits in its first month, 59% solo, structurally
self-contained (only 7 joint commits with `prompt_packs`). Depends on
process infrastructure + core. The approved design keeps Tauri commands,
application paths, SQL/Apalis storage, and worker registration app-side while
extracting the portable automation engine.
Phase 4 remains blocked until the exact Phase 3 candidate is integrated and a
valid shell baseline exists for Phase 4 measurement. No additional v2/v3
diagnostic approval is required.

### Phase 5 — `extractum-llm`

~4,700 lines. Shared layer for `prompt_packs`, `analysis`, and `telegram`
(12/… joint commits with analysis). Extracted before its consumers so they
can depend on it as a crate. Watch the `llm + telegram` seam (6 joint
commits): auth/client responsibilities may need to stay app-side or move to
the telegram phase.

### Phase 6 — `extractum-prompt-packs`

~19,000 lines, 79% solo, 50 commits in the last month: the highest-value
extraction in the repository under either loop metric. Depends on
gemini-browser + llm + core; its `sources`/`db` access is the main JIT design
work (pool-level functions, narrow read interfaces). Expect the largest
visibility-widening review; the layered decision (three crates) is already
fixed.

### Phase 7 — `extractum-analysis`

~13,200 lines, 65% solo, 40 recent commits. Depends on llm + core and on
`sources` read-models (13 joint commits) — the boundary design must decide
which read-model contracts move, stay, or get duplicated as views. Its
`trace.rs` direct `zstd::` usage is consolidated behind `core::compression`
here at the latest.

### Phase 8 — `extractum-telegram`

Dormant cluster: `telegram`, `accounts`, `telegram_session_store`,
`secret_store`, plus the grammers half of `media.rs` (the adapter that
converts `grammers_client::Media` into pure payloads). Outcome: the four
pinned grammers git dependencies leave the application crate entirely. The
extraction must design the payload hand-off into `sources` so ingest keeps
receiving pure values.

### Phase 9+ — Ingest decomposition (gradual, bottom-up)

Order inside the web, each step separately gated and abortable:

1. `youtube::dto` moves down into `sources` (declared backlog trigger:
   immediately before `sources`/`youtube` become separate crates);
2. `extractum-sources` — the storage/read-model layer all producers depend
   on; extracted only once its API churn flattens (currently 3 recent
   commits — trending stable);
3. `takeout_import`, then `youtube`, as producer crates above
   `extractum-sources`; `archive_read_model`, `ingest_provenance`,
   `topic_memberships`, `forum_topics`, `library_sources` assigned
   per-snapshot at that time.

Standing stop rule for this phase: if two consecutive producer slices show
frequent paired edits across the new crate boundary (co-change reappearing in
history), stop decomposing and record the boundary as organizational, not
physical.

### Permanently app-side

`lib.rs` (command registration, state, plugins), `db.rs` (Tauri-plugin pool
adapter), `migrations`, `apalis_jobs`, `diagnostics`, cross-domain
integration tests, WiX/MSI packaging concerns.

## Standing Rules for Every Phase

1. Fresh evidence first: recompute the module co-change matrix and the
   candidate's fan-in/fan-out before writing the spec; history shifts (e.g.
   `gemini_browser` did not exist three months ago).
2. Common measurement mechanics, rename-map inventory comparison, and
   negative-outcome documentation are inherited from the render and focused-
   loop specs. For Phase 4 and later, every shell decision uses five-sample
   baseline/candidate series, requires at least four values within 300 ms of
   each series median, and applies both the 2,000 ms / 20% per-slice cap and
   the 15,000 ms cumulative ceiling. An unstable series invalidates the
   session rather than failing the candidate. The exact historical Phase 3
   reapplication instead records non-gating diagnostics under its explicit
   owner exception.
3. Mechanical moves only inside a slice: facade modules preserve `crate::`
   paths; consumers are never mass-rewritten in the same slice.
4. Every `pub(crate)` → `pub` widening is enumerated in the spec and checked
   by a source contract; glob exports are forbidden in crate roots and public
   API facades. A private app-side compatibility facade may use a glob only
   when its phase spec explicitly authorizes it and the source contract proves
   that the facade remains private. Test-only helpers are not exported.
5. Each new crate gets a Vitest source-boundary contract (dependency roots,
   curated `lib.rs`, forbidden imports, moved-not-copied tests) and the
   workspace-member allowlists in existing contracts are updated in the same
   slice.
6. Core stays pure: no Tauri, sqlx (until the declared `sql_helpers`/`tx`
   trigger), grammers, tokio, or OS-process code.
7. Release evidence = `--no-bundle` build + startup smoke; MSI stays excluded
   until the WiX follow-up lands.
8. `lib.rs` churn is expected shell noise; the shell probe exists to bound
   it, not to forbid it.

## Deferred Items and Triggers

| Item | Trigger |
| --- | --- |
| `sql_helpers` + `tx` into core (adds `sqlx`) | first extracted crate that needs them |
| `youtube::dto` → `sources` | immediately before `sources` or `youtube` becomes a crate |
| `analysis/trace.rs` direct `zstd::` → `core::compression` | phase 7 at the latest |
| `sources::test_support` ownership (fixture crate vs app-side integration tests) | first producer-domain extraction whose tests consume it |
| WiX MSI diagnosis (`light.exe` failure/hang, likely ICE validation in non-interactive sessions) | separate follow-up task; unblocks restoring full-bundle gates |
| Focused-loop metric respec | **Completed 2026-07-17; revised 2026-07-18**: focused commands and the domain gate remain; shell retention now uses 2,000 ms / 20%, the 15,000 ms cumulative ceiling, and the 4/5-within-300-ms validity rule. |
| Process-shell anomaly v2/v3 | **Moot for the current roadmap**: reopen only for a separately approved task requiring sub-second precision or causal attribution; v1 harness remediation remains deferred. |

## Non-Goals

- No big-bang commands/service/store refactor across all domains.
- No global `crate::` → `super::` rewrites.
- No domain error enums before a concrete consumer requires one.
- No `extractum-model`/DTO dumping-ground crate; shared types live with their
  lowest owning domain or in core only when core-pure.
- No build-tooling changes (sccache, nextest, linker, target dirs) inside
  extraction slices; if desired, they are separate experiments.
- No performance claims from cold-cache or cross-machine comparisons.

## Appendix: Raw Analysis Notes

- Commit window: 2026-04-20 → 2026-07-17, 587 non-merge commits touching
  `src-tauri/src`.
- Commit size distribution: 384×1 module, 132×2, 43×3, 9×4, 6×5, 6×6, 4×7,
  2×8, 1×15 (the 15-module commit is the 2026-05-19 sweep; the 8-module
  commits are 2026-06-15 and 2026-04-22 — all three are global refactors and
  were excluded from pair statistics).
- Top raw pairs including shell noise: `lib+sources` 29, `analysis+lib` 21,
  `sources+takeout_import` 18, `notebooklm_export+sources` 16,
  `lib+youtube` 16, `sources+youtube` 15, `migrations+sources` 13,
  `analysis+sources` 13, `analysis+llm` 12, `lib+llm` 11.
- Method: files mapped to their top-level module under `src-tauri/src`;
  per-commit module sets deduplicated; pairs counted over commits touching
  2–6 modules.
