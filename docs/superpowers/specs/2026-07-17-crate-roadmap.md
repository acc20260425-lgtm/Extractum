# Rust Crate Extraction Roadmap

**Status:** Strategic reference; revised and owner-approved 2026-07-19
**Date:** 2026-07-17
**Last revised:** 2026-07-20

## Purpose

This document is the strategic layer above the per-slice specs and plans. It
records the evidence, the target crate map, the phase order, and the standing
rules that every future extraction slice inherits. It authorizes nothing by
itself: each phase requires its own spec and implementation plan, a fresh
dependency map, and fresh co-change statistics before execution.

Completed and in-flight slices governed by their own documents:

- [`2026-07-17-process-and-gemini-browser-crate-boundary-design.md`](2026-07-17-process-and-gemini-browser-crate-boundary-design.md)
  — historical boundary proposal for phases 3 and 4. Phase 3 was measured,
  reverted, and not retained. Its later exact-reapplication workflow ran setup
  and an isolated exact replay, but was canceled before completion and never
  merged or retained. The durable
  [cancellation disposition](../verification/2026-07-19-extractum-process-reapplication-cancellation.md)
  records the recovered sequence. Its Phase 4 clauses are superseded by the
  current design below.
- [`2026-07-19-gemini-browser-crate-boundary-design.md`](2026-07-19-gemini-browser-crate-boundary-design.md)
  — owner-approved Phase 4 boundary. The portable Gemini domain engine moves
  behind a permanent browser-level executor port; all concrete sidecar/CDP
  process ownership remains in the application. Implementation is complete and
  retained.

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
  — focused package commands, advisory extraction timing, plan-shape
  requirements, and unchanged end-of-slice correctness gates. Enforced through
  `AGENTS.md` and the focused-loop source contracts.
- [`2026-07-18-crate-extraction-shell-cap-revision-design.md`](2026-07-18-crate-extraction-shell-cap-revision-design.md)
  — historical shell-cap revision. Its automatic retention gates, cumulative
  ledger, and Phase 3 reapplication authorization are superseded by this
  roadmap revision; its recorded historical measurements remain valid.

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
   paying for the application crate). Its use as the advisory compile-time
   metric for hot-module phases 4–6 is governed by
   `2026-07-17-focused-rust-loop-design.md`; it never replaces a failed
   full-workspace gate.

The commit-history ROI argument (hot modules gain the most) applies fully to
loop 2 and only conditionally to loop 1. This distinction is the roadmap's
main branch point.

## Decision Framework

- Rename-map inventory comparison, exact probe restoration, and documented
  negative outcomes remain common mechanics. For hot-module phases 4–6, the
  focused-loop spec is the normative source for commands, the small advisory
  sample, and failure classification.
- For decisions after 2026-07-19, compile-time measurements are advisory.
  Correctness and completion gates remain mandatory, but timing alone never
  automatically retains, rejects, or reverts a slice. Historical sessions keep
  the thresholds and decisions frozen in their own records.
- Hot-module phases measure only the focused package loop with one discarded
  warm-up and three recorded samples before and after extraction. Record the
  raw values and median. Do not require a quiet-window harness, Defender or
  power-profile metadata, a shell A/B series, a stability retry, or a
  cumulative timing ledger.
- Record the duration already emitted by the mandatory end-of-slice workspace
  check as an operational signal. One completed crate-extraction slice
  contributes one ordinary workspace result. Two consecutive completed
  crate-extraction slices whose ordinary workspace results are each at or
  above 15,000 ms trigger a separate owner-approved performance investigation;
  they do not fail or revert either extraction slice. Do not rerun the check or
  add timing samples for this rule.
- **Historical branch outcome:** Stage 0 returned `no_go` on 2026-07-17
  (app 9,090 ms vs core surrogate 9,100 ms, −0.11%; focused core 1,020 ms).
  The project owner selected the focused package loop plus architectural
  justification. On 2026-07-19 the timing part was simplified further: focused
  timing is advisory for hot-module phases, and phase 8 continues on
  dependency-hygiene justification.
  Development verification in this repository is performed by an LLM agent
  following Superpowers plans, so the focused-loop workflow is adopted through
  mandatory `AGENTS.md` policy and generated-plan structure. Universal
  Superpowers skills remain unchanged unless later evidence shows systematic
  policy violations.
  Full workspace gates remain unchanged as end-of-slice verification.
  - A failed gate never justifies weakening correctness gates, and a
    reverted candidate leaves no partial split behind.

## Roadmap Timing Signals

There is no cumulative shell ledger and no pending Phase 3 baseline. The
following values are historical context, not automatic retention gates:

| Checkpoint | Recorded duration | Disposition |
| --- | ---: | --- |
| Pre-Phase 3 | 9,135 ms | historical retained-workspace reference |
| Historical Phase 3 candidate | 10,177 ms | candidate reverted and not retained |
| Phase 4 `extractum-gemini-browser` | 1,620 ms | completed and retained; [verification](../verification/2026-07-19-extractum-gemini-browser-extraction.md) |

For future slices, record the duration of the mandatory workspace check without
adding a separate shell A/B experiment. For this rule, one completed
crate-extraction slice contributes one ordinary workspace result: the duration
already emitted by its successful mandatory end-of-slice
`cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`.
Two consecutive completed crate-extraction slices whose ordinary workspace
results are each at or above 15,000 ms trigger a separate owner-approved
performance investigation. A completed result below 15,000 ms breaks the
sequence; failed, canceled, and incomplete slices contribute no result. Do not
rerun the check or add timing samples for this rule. Consecutive means adjacent
completed extraction slices in roadmap order; historical measurements,
focused checks, tests, diagnostics, and same-slice reruns do not count.

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
  ├── extractum-notebooklm-render (phase 2 closed no_go; future fresh design)
  │     └── extractum-core
  ├── extractum-process           (phase 3 closed, not retained; future only
  │     └── extractum-core         through a fresh owner-approved design)
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

### Phase 3 — `extractum-process` (closed: not retained)

`external_process`, `child_process`, and `process_tree` become shared
OS-process infrastructure below Gemini Browser, YouTube, and diagnostics.
`job_helpers` stays app-side. Candidate `b364756c` improved the focused median
from 9,171 ms to 2,049 ms and regressed the app-shell median from 9,135 ms to
10,177 ms (+1,042 ms / +11.41%). It correctly failed the original 500 ms / 5%
gate and was reverted in `c47372dc`; that historical result remains unchanged.
The original evidence remains in
[`2026-07-17-extractum-process-extraction.md`](../verification/2026-07-17-extractum-process-extraction.md).

The later exact-reapplication plan
[`2026-07-18-extractum-process-reapplication.md`](../plans/2026-07-18-extractum-process-reapplication.md)
was partially executed. The first attempt stopped before candidate replay after
false-positive quiet-window failures. The corrected second attempt created an
exact but unmerged candidate commit in an isolated worktree. On 2026-07-19 the
project owner canceled the workflow before completion or retention because the
replay and measurement machinery had grown beyond the value of the decision.
This records no broader owner intent and no negative correctness judgment about
the isolated candidate.
The recovered sequence is preserved in the
[`2026-07-19 cancellation disposition`](../verification/2026-07-19-extractum-process-reapplication-cancellation.md).
No process crate, post-reapplication baseline, or cumulative-ledger entry exists.
The 2026-07-18 shell-cap revision no longer authorizes reapplication.

Any future `extractum-process` attempt starts as a new phase with a fresh
owner-approved spec, dependency evidence, plan, and advisory measurement. It
must not replay or resume the canceled plan implicitly. The v2/v3 anomaly
track remains moot and cancellation does not reactivate it.

### Phase 4 — `extractum-gemini-browser` (done: retained)

The fresh 2026-07-19 snapshot contains 10 files, approximately 6,770 lines,
and 94 Rust tests. Since 2026-06-01, 39 commits touched the module; under the
current Rust-domain classification, 28 of 39 (71.8%) touched no other
categorized Rust domain and 8 jointly touched `prompt_packs`. This method
excludes shell files and differs from the frozen 2026-07-17 repository-wide
bucket that produced the historical 59% / 7 figures.

The owner-approved
[`Gemini Browser crate boundary`](2026-07-19-gemini-browser-crate-boundary-design.md)
moves DTOs, protocol and run-log behavior, portable runtime state, submission
and status decisions, job lifecycle, typed timeout/cancellation, and startup
reconciliation into `extractum-gemini-browser`. Tauri commands, application
paths, SQLx/Apalis storage and worker registration, and all concrete
sidecar/CDP spawn, handles, containment, kill/reap, and shutdown remain in
`extractum`.

A permanent domain-level `BrowserExecutor` connects those owners without
exposing PID, child, pipe, process-tree, or Windows types. It does not recreate
`extractum-process`. The historical rejection of an app-side narrow interface
assumed an imminent retained process crate; that premise is void and the new
stable domain port is now selected explicitly.

Phase 4 is retained through the
[`Gemini Browser crate extraction verification`](../verification/2026-07-19-extractum-gemini-browser-extraction.md).
It has no Phase 3 timing or reapplication prerequisite and did not use the
v2/v3 anomaly protocol. The final crate production dependencies are
`parking_lot`, `serde`, `serde_json`, `time`, `tokio`, `tokio-util`, and `url`;
the application retains one path edge to the crate. The frozen 94-test
inventory is owned 75 tests by `extractum-gemini-browser` and 19 by
`extractum`. The retained ordinary workspace-check result is 1,620 ms, below
15,000 ms, so it breaks rather than advances the adjacent-slice investigation
sequence. Phase 5 is now implemented and retained under its owner-approved JIT
boundary design.

### Phase 5 — `extractum-llm` (done: retained)

The owner-approved [LLM crate boundary](2026-07-20-llm-crate-boundary-design.md)
is implemented and retained; see the
[verification](../verification/2026-07-20-extractum-llm-extraction.md).
`extractum-llm` now owns provider clients and model policy, request/completion
DTOs, streaming, execution, timeout/retry behavior, and scheduler/cancellation.
The application retains all nine Tauri commands, `llm://response` events, SQLx
profile persistence, `app_settings`, SecretStore/keyring credentials, profile
lifecycle, and diagnostics behind the private `crate::llm` facade.

The crate's exact direct production dependency roots are `extractum-core`,
`reqwest`, `secrecy`, `serde`, `serde_json`, `tokio`, and `tokio-util`.
`reqwest` and `secrecy` are canonical workspace dependencies inherited by both
packages. The frozen 51-name inventory is owned exactly 36/15 by crate/app;
five new characterization tests are recorded separately.

Both one-shot focused timing series were incomplete because the patch helper
could not execute `codex.exe` in the sandbox. Source SHA-256 and Git blob
identity, restoration, and clean-tree proofs passed; there is no median and no
performance conclusion. Timing was advisory and did not decide retention.

The ordinary mandatory workspace check completed in 10,410 ms, below 15,000
ms. It cannot form an adjacent above-threshold pair with Phase 4's 1,620 ms
result, so no performance investigation is triggered.

At Phase 5 completion, Phase 6 `extractum-prompt-packs` remained next. Its
fresh JIT boundary design was owner-approved, but implementation was not
authorized by the Phase 5 result or by the design document alone.

### Phase 6 — `extractum-prompt-packs` (preparation Checkpoint 2 retained)

The owner-approved
[Phase 6 boundary design](2026-07-20-prompt-packs-crate-boundary-design.md)
refreshes the current scope to 46 files / 19,037 lines and 225 baseline Rust
test identities. Since 2026-06-01, 118 commits touched `prompt_packs`; 92
(78.0%) touched no other categorized Rust domain. The frozen test partition is
223 identities in the new crate and two foreign-source SQL-adapter identities
in the app.

The selected preparation-first boundary gives the crate prompt-pack lifecycle,
YouTube Summary orchestration, validation, and SQL for the 32 prompt-pack-owned
tables. The app retains Tauri commands/events/spawning, `get_pool`, migrations,
profile/secret resolution, foreign source reads, and concrete Gemini Browser
operations. The crate depends downward on `extractum-llm`,
`extractum-gemini-browser`, and `extractum-core`; foreign source data crosses a
narrow owned-value reader, and Browser work crosses an object-safe app port.
The private app facade preserves current Rust consumer paths.

Implementation requires a separate plan and an explicit owner instruction.
Timing remains advisory: one warm-up plus three focused samples per state, with
no quiet-window, retry, shell A/B, or cumulative-ledger machinery.

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
2. Compile-time measurement is advisory and deliberately small. For hot-module
   phases, use the same logical focused-package probe before and after the
   extraction, with one discarded warm-up and three recorded samples per
   state. Record raw values and the median. Restore probe bytes in a `finally`
   path and verify the source hash plus clean worktree once after each series.
   Do not add a quiet-window harness, process-tree coordinator, Defender or
   power-profile capture, formal stability rule, automatic retry, shell A/B,
   or cumulative ledger. Timing never overrides correctness or automatically
   rejects a slice. Record one ordinary workspace result from each completed
   slice's mandatory workspace check. Two consecutive completed crate-extraction
   slices whose ordinary workspace results are each at or above 15,000 ms
   trigger a separate owner-approved performance investigation; do not rerun
   the check or add timing samples for this rule.
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
8. `lib.rs` churn is expected shell noise. Observe its cost through the
   mandatory workspace check rather than a separate shell A/B probe.

## Deferred Items and Triggers

| Item | Trigger |
| --- | --- |
| `sql_helpers` + `tx` into core (adds `sqlx`) | first extracted crate that needs them |
| `youtube::dto` → `sources` | immediately before `sources` or `youtube` becomes a crate |
| `analysis/trace.rs` direct `zstd::` → `core::compression` | phase 7 at the latest |
| `sources::test_support` ownership (fixture crate vs app-side integration tests) | first producer-domain extraction whose tests consume it |
| WiX MSI diagnosis (`light.exe` failure/hang, likely ICE validation in non-interactive sessions) | separate follow-up task; unblocks restoring full-bundle gates |
| Focused-loop metric respec | **Completed 2026-07-17; simplified 2026-07-19**: correctness commands remain mandatory; focused timing is advisory with one warm-up plus three samples per state; shell A/B, automatic timing vetoes, stability retries, and the cumulative ledger are retired. |
| Phase 3 exact reapplication | **Canceled 2026-07-19**: do not execute or resume the historical plan; any future process-crate attempt requires a fresh owner-approved design. |
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
