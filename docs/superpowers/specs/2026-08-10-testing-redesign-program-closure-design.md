# Testing Redesign Program Closure Design

**If a closure goal requires a new mechanism, exclude the goal; do not add the mechanism.**

**Status:** Design boundary approved; written specification awaiting user review.

**Baseline:** `c076c002a8debcce4a5bc71226077c03884367e7`, the clean commit that records Slice 3C completion.

## Summary

Slice 3C completed the source-contract migration at 671 ledger rows / 0 open,
removed all 86 legacy files and the legacy Vitest runner, closed all 79 connected
components, and passed the complete repository gate in 818.8 seconds. The
Testing Redesign program has no further feature or architecture slice.

One final closure-only slice will remove the transition apparatus, reduce the
custom RepositoryIndex surface to three durable rules, remove stale program
documents from the repository, and leave one current description of the daily
testing loop in `docs/project.md`. The slice changes no product behavior and
creates no replacement testing architecture.

The closure is deliberately subtractive. Git history is the archive for the
removed program corpus. The final commit message names the baseline OID above
so the complete historical corpus can be recovered without searching.

## Baseline and Net-Negative Contract

The exact post-3C baseline is measured from Git objects at `c076c002`; the
closure slice must not add a measurement script, baseline registry, or evidence
schema.

| Surface | Baseline | Required closure state |
| --- | ---: | --- |
| `testing/` | 29 files, 2,892,320 bytes, 30,043 physical lines | absent |
| `scripts/testing/` | 21 files, 722,288 bytes, 13,349 physical lines | strictly fewer files and lines |
| Registered RepositoryIndex rules | 11 | exactly 3 |
| Active Testing Redesign corpus | 18 files, 527,699 bytes, 7,901 physical lines | absent from the repository |
| Final complete gate | PASS in 818.8 seconds | PASS; duration recorded, not used as a threshold |

The categories are evaluated independently. Documentation deletion cannot
offset growth in executable testing code, and deletion of generated JSON bytes
cannot offset growth in permanent scripts. Logical JSON rows, tracked files,
blob bytes, and physical lines remain separate observations; the minified
671-row ledger must not be treated as one semantic line.

The closure creates no new test file, rule ID, runner, project, dependency,
command family, status vocabulary, structured manifest, allowlist, registry,
ledger, evidence carrier, cache, selector, scheduler, watcher, or timing store.

## Decision Defaults

The closed redisposition taxonomy is historical. In particular,
`D5_ACCEPTED_LOSS` is not a closure status and must not be reused.

Every audited item receives one of two final outcomes:

1. Delete a transition-only or implementation-shaped artifact when current
   product behavior has a cheaper existing owner or the artifact has no product
   consumer.
2. Retain an existing capability when it protects a current durable invariant
   and evidence is insufficient to remove it safely. The closure may only
   subtract transition imports, branches, rules, or assertions from a retained
   file; it may not add or generalize its mechanism.

There is no deferred third state and no closure backlog. If improving,
rewriting, or replacing an item would require a new mechanism, that improvement
is dropped. Product code and product behavior are never deleted by default.

## Why the Inventory Is Manual

The source-contract ledger proves that an obligation moved, but it does not
prove that either the subject or the replacement owner is not itself temporary
test scaffolding. Slice 3C created behavior replacements for modules that exist
only for tests. Therefore the coupled-artifact inventory is a named human
current-state audit, not a derivation from the ledger, review carrier, runner
census, or RepositoryIndex.

## Transition Apparatus Removal

### Delete `testing/` completely

Delete all 29 tracked objects:

- `testing/runner-census.json`;
- `testing/source-contract-ledger.json`;
- `testing/source-contract-redisposition-review.json`;
- all 26 files under `testing/source-contract-redisposition-evidence/`.

The four Vitest projects and two Playwright owners remain authoritative through
their real configurations and package commands. The repository no longer keeps
a second JSON census or validates completed migration evidence on every
`verify`.

### Delete transition-only scripts and tests

Delete from `scripts/testing/`:

- `extract-source-contract-ledger.mjs`;
- `run-observation.mjs` and `run-observation.behavior.test.ts`;
- `slice-1-baseline.mjs` and `slice-1-baseline.behavior.test.ts`;
- `slice-1-rust-feasibility.mjs` and `slice-1-rust-feasibility.test.ts`;
- `source-contract-redisposition-review.mjs` and
  `source-contract-redisposition-review.test.ts`;
- `testing-transition.mjs`;
- `timing-log.mjs` and `timing-log.test.ts`;
- `validate-testing-transition.test.ts`.

Delete the top-level `scripts/validate-testing-transition.mjs`. Remove its step
from `scripts/verify.mjs` and its expectation from `scripts/verify.test.ts`.
Preserve every remaining `verify` gate.

Retain and simplify only as necessary to remove transition imports:

- `repository-index.mjs` and `repository-index.test.ts`;
- `repository-rules.mjs` and `repository-rules.test.ts`;
- `test-conventions.test.ts`;
- `research-projects-grid-date-formatting.behavior.test.ts`;
- `research-projects-import-boundary.behavior.test.ts`;
- `setup-component-tests.ts`.

The retained tests must no longer import the ledger or extractor merely to
prove that the deleted transition authorities agree with one another.

### Delete dead focused commands and the one-time stability audit

Remove `test:project-runs` and `verify:project-runs` from `package.json`.
`test:project-runs` names the deleted
`src/lib/project-runs-screen-contract.test.ts`; its composite command is already
broken. Keep `test:rust:prompt-pack-runs` as a valid focused Rust command and
remove the stale project-runs prose from `docs/project.md`.

Delete `scripts/verify-stability.mjs`, `scripts/verify-stability.test.ts`, and
the `verify:stability` package command. Outside its own test and the historical
Slice 2B plan, the fixed 20-run `chromium-lifecycle` audit has no current
workflow documentation or recurring use. It is an investigation artifact, not
a product invariant or daily-loop command.

Retain `scripts/run-vitest.mjs` and `scripts/run-vitest.behavior.test.ts`.

## RepositoryIndex Audit

RepositoryIndex remains only as the existing engine for three current
cross-repository boundaries. The closure adds no evaluator ability and no rule.

### Retain exactly three rules

1. `rule:extractum-grid-wrapper-boundary` retains the repository-wide rule that
   direct `@svar-ui/` imports stay inside the approved wrappers.
2. `rule:telegram-crate-dependency-ownership` retains the current Grammers
   dependency and feature-closure boundary.
3. `rule:telegram-crate-manifest-boundary` retains the current resolver-v2 and
   test-support dependency seam.

The dependency-ownership rule retains its complete authority pair explicitly:

- `scripts/telegram-grammers-feature-baseline.mjs`;
- `src/lib/telegram-grammers-feature-baseline.json`.

The corresponding `.gitattributes` entry remains. These files are current rule
authority after closure and must not be swept up with Phase 8B artifacts.

### Delete eight rules

Delete all six `/analysis` rules:

- `rule:analysis-evidence-highlight-token-styling`;
- `rule:analysis-source-browser-canonical-composition`;
- `rule:analysis-source-browser-explicit-subject-contract`;
- `rule:analysis-source-group-activity-boundary`;
- `rule:analysis-source-group-tab-leaf-boundary`;
- `rule:analysis-source-reader-surface-composition`.

`/analysis` remains a live product surface and its future is outside this
slice. These rules leave because they freeze CSS tokens, exact component or AST
shape, completed cutover exclusions, or behavior already covered by rendered
component tests and compiler-visible interfaces. Their removal does not depend
on retiring `/analysis`.

Delete `rule:telegram-phase-8b-authority-integrity`. It validates frozen 8A/8B
plan artifacts and a staging tree whose 19-file inventory now has only the
compatibility facade `src-tauri/src/telegram_impl/lib.rs` left. The current
Grammers dependency boundary remains owned by the separate retained rule.

Delete `rule:extractum-llm-public-api-boundary`. It contains a real credential
concern, but implements it as a closed inventory of every production Rust
module, root export, field, method, alias, macro, and attribute, backed by a
custom Rust parser. Ordinary module addition or rename fails even when no
security boundary changes. Retaining that mechanism would preserve the testing
framework by calling implementation shape a product invariant.

The existing Rust compile/trait test in
`src-tauri/crates/extractum-llm/src/lib.rs` continues to prove the curated
external API, private credential fields and named accessors, and the absence of
serialization, deserialization, debug, and secret-exposure traits. Closure
explicitly stops promising fail-closed detection of an arbitrarily newly named
future credential accessor. This is a documented limitation of the current
test loop, not a resurrected `D5` ledger disposition.

## Coupled Artifact Inventory

### Delete contract-path scaffolding after its apparatus consumers

Delete:

- `src/lib/analysis-contract-paths.ts`;
- `src/lib/prompt-pack-contract-paths.ts`;
- `src/lib/telegram-contract-paths.ts`;
- `src/lib/telegram-contract-paths.behavior.test.ts`.

The analysis and prompt-pack modules have no live import consumer. Every
consumer or textual authority for the Telegram module is inside the transition
apparatus. Remove those consumers first, then delete the modules and behavior
test. Retain `src/lib/telegram-checkpoint-2.behavior.test.ts`, which exercises a
live frontend and Telegram IPC boundary despite its historical filename.

### Delete the complete Phase 8B authority set

Delete:

- `src/lib/telegram-8b-staging-sha256.json`;
- `src/lib/telegram-8b-symbol-map.json`;
- `src/lib/telegram-8b-test-identities.json`;
- `scripts/telegram-staging-sha256.mjs`;
- `scripts/telegram-8b-symbol-map.mjs`;
- `scripts/telegram-8b-test-identities.mjs`.

The staging SHA generator is included even though it has no inbound consumer:
it creates the deleted staging JSON and is itself part of the completed Phase
8B apparatus.

### Clean coupled metadata

Delete `.superpowers/sdd/reviewfix-rules-report.md`; it records a stale open-row
state and obsolete validator command.

From `.gitattributes`, remove only entries for the deleted Phase 8B JSON files
and `testing/source-contract-redisposition-evidence/*.json`. Preserve the
Grammers baseline entry.

In `docs/value-registry.md`:

- remove the Telegram crate-preparation agent lifecycle section, including
  `8b-checkpoint-1` through `8b-checkpoint-8`, their retained-status strings,
  and the terminal `8c-extracted` / retained-done values;
- do not add omitted historical values merely to delete them;
- change stale five-owner wording to the current four Vitest projects and
  remove `legacy-contract` and runner-census authority wording;
- remove the never-executed Nx decision values;
- remove completed ledger disposition values.

Update `vitest.config.ts`, `test-conventions.test.ts`, package scripts, and
current documentation only to remove deleted paths and transition assumptions.
Do not create replacement registries or current-state inventories.

## Program Corpus Removal

Delete, rather than archive, the following 18 baseline documents.

Specs:

- `docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md`;
- `docs/superpowers/specs/2026-08-05-testing-redesign-slice-3b-design.md`;
- `docs/superpowers/specs/2026-08-08-testing-redesign-slice-3c-design.md`.

Plans:

- `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`;
- `docs/superpowers/plans/2026-08-02-testing-redesign-slice-1-measurement.md`;
- `docs/superpowers/plans/2026-08-02-testing-redesign-slice-2a-migration-preflight.md`;
- `docs/superpowers/plans/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md`;
- `docs/superpowers/plans/2026-08-03-testing-redesign-slice-3a-source-contracts.md`;
- `docs/superpowers/plans/2026-08-05-testing-redesign-slice-3b-component-contracts.md`;
- `docs/superpowers/plans/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md`;
- `docs/superpowers/plans/2026-08-08-testing-redesign-slice-3c-source-contract-completion.md`.

Verification records:

- `docs/superpowers/verification/2026-08-02-testing-redesign-slice-1-measurement.md`;
- `docs/superpowers/verification/2026-08-02-testing-redesign-slice-2a-migration-preflight.md`;
- `docs/superpowers/verification/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md`;
- `docs/superpowers/verification/2026-08-03-testing-redesign-slice-3a-source-contracts.md`;
- `docs/superpowers/verification/2026-08-05-testing-redesign-slice-3b-component-contracts.md`;
- `docs/superpowers/verification/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md`;
- `docs/superpowers/verification/2026-08-08-testing-redesign-slice-3c.md`.

A repository-wide inbound-reference search outside this corpus is empty. The
documents contain stale forward authority for Slice 4, TestingManifest, Nx,
and nonexistent commands; retaining them under `archive/` would leave that
obsolete map in the working tree. Git at `c076c002` is the recovery mechanism.

This closure design and its implementation plan are temporary active program
artifacts. The final closure commit deletes both after their instructions have
been executed and before the final complete gate.

## Current-State Documentation

Do not create a closure verification record. Update the existing testing and
verification section of `docs/project.md` so it alone describes:

- the four Vitest projects and two Playwright owner commands;
- `test:changed`, `test:changed:last`, and `test:related` as Vitest-only
  accelerators;
- bootstrap and sidecar prerequisites;
- the full `verify` composition;
- the existing focused Rust loops and their completion role.

Keep the corresponding daily frontend and focused Rust loops in `AGENTS.md`,
with every named command checked against `package.json`.

The current-state documentation must state the deliberately absent guarantees:

- there is no 15-second fail-closed cross-stack selector;
- changed/related Vitest selection does not cover Rust, Playwright, or OS
  process surfaces and may under-select dynamic relationships;
- there is no TestingManifest, NoTestAllowlist, runner census, source-contract
  ledger, Nx integration, coverage ratchet, quarantine system, repeated
  performance series, or stability-audit command;
- an empty or unexpectedly small accelerated selection requires an explicit
  owner command or wider run;
- dependency-cruiser is outside this program and may return only as a separate
  decision that replaces a larger custom mechanism rather than adding one.

## Execution Order

1. Reconfirm the clean baseline OID and derive baseline counts from Git without
   adding a script.
2. Remove transition validation from `verify` and sever ledger, census,
   extractor, review, and evidence imports from retained tests.
3. Delete the transition apparatus and the complete `testing/` tree.
4. Reduce RepositoryIndex to the three approved rules and update its existing
   tests without adding a rule, engine ability, or substitute authority.
5. Delete contract-path and Phase 8B scaffolding, then clean package commands,
   Vitest configuration, `.gitattributes`, the value registry, and other named
   coupled artifacts.
6. Update `docs/project.md` and `AGENTS.md` to the current loop and explicit
   limitations.
7. Delete the 18 historical program documents plus this temporary design and
   its implementation plan.
8. Confirm independent net-negative counts and run the intended final complete
   `npm.cmd run verify`.
9. Commit the closure with the baseline OID, final net reduction, and actual
   successful `verify` duration in the commit body.

## Failure Handling

Focused existing owner commands may be used while removing dependencies and
fixing dangling references. They are implementation feedback, not completion
evidence.

If deleting an item would uncover a durable invariant that cannot remain owned
without a new mechanism, retain the existing item unchanged and drop that
deletion goal. Do not introduce a replacement owner or deferred testing queue.

The intended final `verify` is one completion attempt, not a prohibition on
recovering from failure. If it fails, the closure remains open. Diagnose with
the smallest existing non-empty owner, correct or revert the responsible
deletion, and let the next fresh complete run become the final attempt. Do not
create content-addressed evidence, percentage comparisons, a retry protocol,
or a verification document.

## Acceptance Criteria

The program is closed only when all of the following are true:

- `testing/` is absent;
- `scripts/testing/` has fewer than 21 files and fewer than 13,349 physical
  lines;
- RepositoryIndex registers exactly the three approved rule IDs;
- the 18 baseline program documents and the temporary closure design/plan are
  absent from the repository;
- all named dead commands, contract-path modules, Phase 8B authorities, and
  coupled registry/configuration references are absent;
- the Grammers feature-baseline script, JSON authority, and retained dependency
  rule remain mutually consistent;
- no new testing dependency, project, rule, command family, manifest, registry,
  status, or evidence artifact has been introduced;
- `docs/project.md`, `AGENTS.md`, `package.json`, Vitest configuration, and the
  executable commands agree on the current daily and completion loops;
- a fresh `npm.cmd run verify` passes every retained gate;
- the final commit body records baseline OID `c076c002a8debcce4a5bc71226077c03884367e7`,
  the actual net reduction, and the successful complete-gate duration.

## Explicit Non-Goals

- Reconsidering or retiring `/analysis`.
- Changing product behavior or product architecture.
- Building Slice 4, TestingManifest, NoTestAllowlist, or a smart cross-stack
  selector.
- Adding Nx, dependency-cruiser, coverage ratchets, quarantine, timing
  databases, performance series, new Vitest projects, or new test runners.
- Replacing removed transition evidence with another historical authority.
- Treating elapsed `verify` time as a product-value or stop criterion.
