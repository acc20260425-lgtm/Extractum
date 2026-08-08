# Testing Redesign Pre-Slice 3C Redisposition Verification

Date: 2026-08-08

This is a human-readable acceptance record for the committed redisposition artifact and ledger cutover. The decision source of truth remains `testing/source-contract-redisposition-review.json`; this document copies or mechanically renders its values and records the required gate observations.

Correction note (2026-08-08): the final-fix wave corrected Vitest mechanism ownership, fail-closed Task 3 candidate/policy binding, the approved SC-000085/090/316/333/416/441 dispositions or owners, and the accepted content-addressed blind evidence. It did not rerun or replace the prior single 300.700-second final verify observation.

Forecast correction (2026-08-08): execution ownership now places the 90 rows / 598 assertion ordinals targeting `src/lib/components/**/*.behavior.test.ts` in the component/jsdom project. This corrects the Node/jsdom forecast to 179 / 1,109 and 90 / 598 respectively without changing the content-addressed historical packet mechanism bytes, decisions, review paths, hashes, or blind digests.

Task 4 correction (2026-08-08): the user-approved follow-up replaces SC-000464's insufficient five-owner joint mapping with one comprehensive real-orchestration Cargo owner, and converts SC-000515 to an exact mixed split: architecture ordinals `1, 11, 19` use `rule:extractum-grid-wrapper-boundary`; behavior ordinals `2-10, 12-18, 20-21` use the frozen Vitest owner. Final execution forecasts are Node `178 / 1,101`, jsdom `90 / 598`, Cargo `19 / 161`, and Playwright `3 / 21`; final dispositions are 289 behavior, 1 mixed, and 146 delete. Historical content-addressed evidence bytes remain unchanged.

Final Task 4 owner correction (2026-08-08): the explicitly approved final wave keeps SC-000464's comprehensive service owner and adds the application Cargo owner that drives the actual execution-task builder and proves profile resolution is deferred until the spawned future is polled. SC-000515's 18 behavior ordinals move to a component owner rendering the real wrappers; SC-000025, SC-000435, and SC-000552 likewise move from helper-only Node owners to real route/component owners. The resulting execution forecast is Node `174 / 1,065`, jsdom `94 / 634`, Cargo `19 / 161`, and Playwright `3 / 21`. The SC-000515 architecture subgroup remains unchanged, and historical content-addressed evidence bytes remain unchanged.

## Review pins and scope reconciliation

- Review base: `a54507d63420bb870c3870c91d7e22b050abae3e`.
- Frozen ledger commit: `0045abf6006f42fe735d5142e139375d702d4fe9`.
- Completed-review commit: `ccdda90815bcf080080687c9c33144afee61a5cd`.
- Atomic ledger-cutover commit: `1e7a57ebbe7fc11b9878c914f1ae28dc59e03c82`.
- Artifact bytes/SHA-256: `571281` / `65bedf06a35b00b5a585659ce859f23e30a424ceccb5c45429a08183b14fdcf1`.
- Scope: 671 ledger rows = 436 validator-open review rows + 235 closed rows. The 436 open rows span 86 files; 124 carry the extraction-only `manual` marker.
- The 26-row large-contract population is 16 rows from `src/lib/analysis-crate-boundary-contract.test.ts` plus 10 from `src/lib/crate-extraction-shell-cap-contract.test.ts`. The tail is the remaining 410 rows in 84 files.
- Closed-row SHA-256: `385c6f0596e5c3d25f26df077484e25918af68fba5bab138fc8e014c7eceb2d8`. The carrier confirms byte-equivalence for all 235 closed rows. `SC-000355` and `SC-000366` are the two path-present closed rows and remained outside the review with the other 233 closed rows.
- The applied carrier check reports no changed JSON paths. The Task 5 cutover audit recorded 171 rows and 451 changed paths, all confined to `disposition`, `replacementIds`, `deletionReason`, and `subgroups`, with zero immutable-field drift.

## Disposition and class results

| Disposition | Before | After | Change |
| --- | ---: | ---: | ---: |
| `architecture` | 144 | 0 | -144 |
| `behavior` | 276 | 290 | +14 |
| `delete` | 16 | 146 | +130 |
| mixed | 0 | 0 | 0 |
| **Total** | **436** | **436** | **0** |

| Ordered class | Rows |
| --- | ---: |
| `D1_COMPLETED_HISTORY_ONLY` | 6 |
| `D2_IMPLEMENTATION_SHAPE` | 122 |
| `D3_NON_OBSERVABLE_VISUAL` | 10 |
| `D4_DUPLICATE_EVIDENCE` | 8 |
| `A1_EXISTING_STRUCTURED_OWNER` | 0 |
| `T1_EXISTING_TOOL_OWNER` | 0 |
| `B1_EXISTING_BEHAVIOR_OWNER` | 0 |
| `B2_NEW_CHEAP_BEHAVIOR` | 282 |
| `B3_PROTECTED_EXPENSIVE_BEHAVIOR` | 8 |
| `D5_ACCEPTED_LOSS` | 0 |
| **Total** | **436** |

Existing-owner outcomes are eight `D4` duplicate-evidence rows; `A1`, `T1`, and `B1` have no rows.

| Existing owner mechanism | Rows | Assertion ordinals | SC IDs |
| --- | ---: | ---: | --- |
| jsdom/Vitest | 3 | 23 | `SC-000064`, `SC-000121`, `SC-000316` |
| Cargo | 2 | 28 | `SC-000085`, `SC-000118` |
| Structured rule | 3 | 3 | `SC-000356`--`SC-000358` |
| **Total** | **8** | **54** |  |

Future behavior owners are projected as follows:

| Future mechanism | B2 rows | B3 rows | Total rows | Assertion ordinals |
| --- | ---: | ---: | ---: | ---: |
| Node/Vitest | 177 | 2 | 179 | 1109 |
| Cargo | 15 | 3 | 18 | 156 |
| jsdom/Vitest | 90 | 0 | 90 | 598 |
| Playwright | 0 | 3 | 3 | 21 |
| **Total** | **282** | **8** | **290** | **1884** |

## Replacement cost evidence

Each mechanism used one unretained warm-up followed by the three retained unsandboxed observations shown here.

| Mechanism | Exact command | Warm-up exit | Retained seconds | Median seconds |
| --- | --- | ---: | --- | ---: |
| Component replacement | `node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/projects-workspace.behavior.component.test.ts src/lib/analysis-report-canvas.behavior.component.test.ts src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts` | 0 | 28.241, 28.402, 28.422 | 28.402 |
| Component startup | `node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/SourceStatusCell.component.test.ts` | 0 | 4.253, 4.211, 4.251 | 4.251 |
| Removable legacy owner | `npm.cmd run test:legacy-contract` | 0 | 11.574, 11.590, 11.515 | 11.574 |

The removable legacy inventory contains 73 files and contributes a measured median 11.574 seconds.

| Forecast quantity | Artifact value |
| --- | ---: |
| Proposed new jsdom rows / ordinals | 0 / 0 |
| Conservative upper bound per row | 0.6174347826086957 s |
| Scalable cost per row | 0.5250217391304348 s |
| Proposed-row upper forecast | 0 s |
| Proposed-row scalable forecast | 0 s |
| Net gate forecast after removable legacy time | 0 s |
| Scalable forecast as percent of the 244.600-second successful baseline | 0% |
| Net forecast as percent of that baseline | 0% |

The binding result is **0 proposed jsdom rows <= 46 rows**, leaving all 46 rows of ceiling headroom. At the full 46-row ceiling the mechanically derived scalable forecast remains 24.151 seconds. Because 11.574 seconds of removable legacy time does not exceed 24.151 seconds, `legacyDominatesFullUnitCeiling` is `false`; the zero proposed-row net forecast is disclosure, not the binding constraint.

### Approved Playwright disclosure

These are the only unresolved Playwright owners in the artifact. All three are B3 with `TESTING_BROWSER_COMPONENT_OWNERSHIP`; they consume no jsdom units and carry no timing estimate at this checkpoint.

| SC ID | Approved future owner | Ordinals |
| --- | --- | ---: |
| `SC-000312` | `test:playwright:e2e/app-shell-responsive.spec.ts#mobile-menu-trigger-responsive-visibility` | 3 |
| `SC-000344` | `test:playwright:e2e/research-projects-sources-filter-row.spec.ts#filters-available-across-responsive-layouts` | 6 |
| `SC-000385` | `test:playwright:e2e/dialog-layering.spec.ts#dialog-content-visible-interactive-above-overlay` | 12 |
| **Total** | **3 rows** | **21** |

No other decision proposes an unresolved `test:playwright:` replacement.

## Accepted D5 Losses

Total accepted D5 losses: **0 rows / 0 assertion ordinals**.

| SC ID | ordinals | lost behavior |
| --- | --- | --- |
| _No rows_ | _None_ | _No behavior was accepted for loss._ |

## Independent blind-review evidence

### Task 3 mandatory and calibrated populations

- Accepted corrective reviewer: `/root/redisposition_final_fixes/task3_final_complete_reviewer`; run `redisposition-task-3-final-complete-policy-binding`; policy `blind-no-proposed-class-or-reason`.
- Accepted blind results: 95 rows in contiguous shards of 24, 24, 24, and 23. `validIterations` remains 3. Merged packet SHA-256: `95dc9a221bd12973911921d61d2b33dd7d22da2514b10eebe0d36a95f4fdd408`; merged output SHA-256: `bcc9e9a26deb49be6c65580e1e66aa1fa55192c7cc147427505feb81edfc998b`; canonical merged-result digest: `c94d460fd91ff1bc611aa6aa17bde0dded8b88325dd2e953e705107f509fa971`.
- Deterministic sample: 42 of 415 rows, population digest `e64b4770724fbb2385bb816e28a5dc5d25a83e80a6451105cd025e1c4fd40279`, iteration 3, comparison `agree`.
- Sample IDs: `SC-000492`, `SC-000018`, `SC-000453`, `SC-000305`, `SC-000121`, `SC-000289`, `SC-000316`, `SC-000383`, `SC-000008`, `SC-000136`, `SC-000210`, `SC-000165`, `SC-000194`, `SC-000171`, `SC-000011`, `SC-000333`, `SC-000396`, `SC-000481`, `SC-000423`, `SC-000304`, `SC-000471`, `SC-000297`, `SC-000484`, `SC-000130`, `SC-000474`, `SC-000416`, `SC-000399`, `SC-000203`, `SC-000004`, `SC-000118`, `SC-000449`, `SC-000335`, `SC-000176`, `SC-000298`, `SC-000349`, `SC-000192`, `SC-000509`, `SC-000414`, `SC-000081`, `SC-000083`, `SC-000465`, `SC-000360`.

| Mandatory cohort | Exact row population |
| --- | --- |
| protected P0 | `SC-000020`, `SC-000420`, `SC-000441`, `SC-000511`--`SC-000515`, `SC-000555`--`SC-000560` |
| security | `SC-000441`, `SC-000443`, `SC-000555`--`SC-000560` |
| import boundary | `SC-000511`--`SC-000515` |
| process lifecycle | `SC-000015`, `SC-000017`, `SC-000018`, `SC-000023`, `SC-000386`--`SC-000390`, `SC-000420` |
| large contracts | `SC-000077`--`SC-000092`, `SC-000356`--`SC-000365` |
| emerging B3 | `SC-000015`, `SC-000023`, `SC-000312`, `SC-000344`, `SC-000385`, `SC-000387`, `SC-000389`, `SC-000420` |
| emerging D5 | zero rows |
| emerging mixed | zero rows |
| known browser five | `SC-000312`, `SC-000323`, `SC-000352`, `SC-000353`, `SC-000385` |

Calibration covered D1 (`SC-000361`--`SC-000363`), D2 (`SC-000077`, `SC-000080`, `SC-000349`), D3 (`SC-000323`, `SC-000352`, `SC-000353`), D4 (`SC-000064`, `SC-000356`, `SC-000357`), B2 (`SC-000297`, `SC-000390`, `SC-000441`), and B3 (`SC-000312`, `SC-000344`, `SC-000385`). A1, T1, B1, and D5 recorded `no_match` with zero rows.

The approved grouped disagreements were:

| Author class | Final class | Rows | Group adjudication |
| --- | --- | --- | --- |
| D1 | D2 | `SC-000077`, `SC-000080`, `SC-000081`, `SC-000092`, `SC-000203` | Completed-event D1 excludes future-defect topology; exact current topology without an independent contract is D2. |
| A1 | D4 | `SC-000356`--`SC-000358` | A resolved base-closed owner proving the full invariant makes the source evidence D4 before A1. |
| B2 | B3 | `SC-000420` | The truthful observation seam governs B2/B3; real browser or OS-process observation is B3. |
| B2 | D2 | `SC-000449` | A retained owner must prove the complete frozen invariant, not adjacent behavior. |

The final corrective binding also records the user-approved exact-owner and policy adjudications: `SC-000085` and `SC-000316` are D4 only with their complete base-closed owner sets; `SC-000090`, `SC-000416`, and `SC-000441` retain their exact approved future owners; `SC-000323`, `SC-000352`, and `SC-000353` retain D3; and `SC-000333` is D2 because it pins the selector string rather than independently observing active state. Every protected author and blind fingerprint retains its exact mandatory criticality reference.

### Task 4 tail population

- Tail partition: testing/process infrastructure 24 rows/7 files; analysis 152/24; projects/library 73/19; prompt packs/YouTube 53/11; Gemini browser 29/3; Rust/security boundaries 25/6; other product 54/14. Total: 410 rows/84 files.
- Accepted reviewer: `/root/redisposition_task4_impl/task4_blind_reviewer`; run `redisposition-task-4-tail-valid-iteration-1`; policy `blind-no-proposed-class-or-reason`.
- Required blind set: 46 rows, digest `23a1b0fabef8c9085b17ac850a8a3e56f62f2b626b7b82d836106d0cf5c518a3`: `SC-000004`, `SC-000008`, `SC-000011`, `SC-000015`, `SC-000023`, `SC-000118`, `SC-000121`, `SC-000130`, `SC-000136`, `SC-000165`, `SC-000171`, `SC-000176`, `SC-000192`, `SC-000194`, `SC-000203`, `SC-000210`, `SC-000289`, `SC-000298`, `SC-000304`, `SC-000305`, `SC-000312`, `SC-000316`, `SC-000325`, `SC-000333`, `SC-000335`, `SC-000344`, `SC-000383`, `SC-000385`, `SC-000387`, `SC-000389`, `SC-000396`, `SC-000399`, `SC-000406`, `SC-000414`, `SC-000416`, `SC-000420`, `SC-000423`, `SC-000449`, `SC-000453`, `SC-000465`, `SC-000471`, `SC-000474`, `SC-000481`, `SC-000484`, `SC-000492`, `SC-000509`.
- Deterministic sample: 38 of 377 rows, population digest `0358a631cce6dcab4b26a10ac14f5504db51b96a413d54066527388376219ed1`, one valid iteration, comparison `agree`.
- Risk cohorts: B3 is the eight-row set `SC-000015`, `SC-000023`, `SC-000312`, `SC-000344`, `SC-000385`, `SC-000387`, `SC-000389`, `SC-000420`; D5 and mixed are empty.
- Task 4 author disagreements: zero. Thirteen coverage-only Task 3 fingerprint differences are `SC-000118`, `SC-000121`, `SC-000203`, `SC-000316`, `SC-000333`, `SC-000344`, `SC-000387`, `SC-000389`, `SC-000414`, `SC-000420`, `SC-000474`, `SC-000484`, and `SC-000492`; `SC-000316` was added mechanically after the approved D4 correction and does not alter Task 4's accepted decisions.
- One invalid, non-counting review-transport attempt was preserved with SHA-256 `2b7af70617191155cd4b526730b38dc1e2efbee6a293fe39ee2bac3a6b0c4260`; it did not increment `tailValidIterations`. The accepted fixed point has `tailValidIterations: 1`.

## Verify observations and final commands

The artifact preserves both Task 2 observations and does not hide the execution-order RED:

| Observation | Result | Duration | Note |
| --- | --- | ---: | --- |
| Task 2, execution order 1 | exit 1 | 53.987 s | `npm run test:unit` failed because the static draft artifact intentionally did not yet exist. |
| Task 2, execution order 2 | exit 0 | 244.600 s | Explicitly user-authorized compensating verify after complete GREEN. |
| Task 6 fresh final gate | exit 0 | 300.700 s | Single unsandboxed run after a successful unsandboxed repository-process preflight; no retry. |

The fresh 300.700-second gate is 92.600 seconds (44.497838%) above 208.1 seconds, 20.600 seconds (6.411453%) below 321.3 seconds, 82.700 seconds (21.570162%) below 383.4 seconds, and 56.100 seconds (22.935405%) above the 244.600-second successful artifact baseline. The 90-second program value remains an objective rather than a B3 admissibility filter.

The corrective independent evidence review used the bounded final checkpoint commands. Their fresh results are:

1. `node scripts/run-vitest.mjs run --project unit-node scripts/testing/source-contract-redisposition-review.test.ts` — exit 0; 1 file and 33 tests passed.
2. `node scripts/testing/source-contract-redisposition-review.mjs check` — exit 0; class/path grouping printed and `changed JSON paths:` was empty.
3. `node scripts/validate-testing-transition.mjs` — exit 0; 196 filesystem candidates, 189 Vitest files, 7 Playwright files, and 671 ledger rows / 436 open.
4. `npm.cmd run check` — exit 0; 0 errors and 0 warnings.
5. `npm.cmd run verify` — the already-recorded single unsandboxed invocation remains exit 0 in 300.700 seconds; it was not rerun for this correction.

## Commit and handoff boundary

The user-authorized Task 4 schema amendment made tail convergence a separate fixed-point cycle: it preserves Task 3 `validIterations: 3`, shards, and adjudications; adds independently bounded `tailValidIterations` and content-addressed tail shards; and forbids reusing the Task 3 adjudication allowlist for a new tail disagreement. The accepted tail fixed point used `tailValidIterations: 1`.

Task 5 required two user-authorized amendments before the atomic cutover. The carrier now accepts only the complete review-base open-row resolution state or the complete all-at-once applied state and rejects every partial/hybrid state. The carrier test also reuses the production evidence loader instead of directly reading packet bytes, preserving the tamper/hash/schema assertions without adding a ledger row or source-reader exception.

The checkpoint commit chain, including review hardening and the Task 5 amendments, is:

| Commit | Subject | Role |
| --- | --- | --- |
| `a5e5e9b4` | `test: add source-contract redisposition carrier` | Initial carrier checkpoint. |
| `b85ae947` | `test: harden redisposition review carrier` | Carrier review hardening. |
| `b6c60ec7` | `test: lock redisposition review state` | Review-state pinning. |
| `c0c611b0` | `test: preserve redisposition row order` | Row-order validation. |
| `20fbd473` | `test: pin redisposition scope and timing evidence` | Measured draft. |
| `c71a3275` | `test: record compensating redisposition verify` | Preserved second Task 2 verify observation. |
| `16c533cf` | `test: harden redisposition evidence pins` | Catalog/evidence validation hardening. |
| `bf6a9db3` | `test: calibrate mandatory redisposition cohorts` | Calibrated review. |
| `fd8f0681` | `test: converge redisposition blind review` | Blind-review convergence. |
| `124d66af` | `test: finalize redisposition review evidence` | Accepted content-addressed Task 3 evidence and adjudications. |
| `5c29b812` | `test: close redisposition validation gaps` | Calibration/cohort/evidence binding follow-up. |
| `d558496c` | `test: pin redisposition cohort membership` | Mandatory-cohort membership follow-up. |
| `ccdda908` | `test: complete source-contract redisposition review` | Completed Task 4 review. |
| `0d3ea826` | `style: normalize source-contract ledger json` | Semantic-no-op formatting normalization. |
| `69c32b6d` | `fix: accept exact applied redisposition ledger` | User-authorized carrier amendment: accept only the complete base or complete applied resolution state. |
| `84bf665a` | `test: reuse redisposition evidence loader` | User-authorized source-reader amendment without a ledger row/exception. |
| `1e7a57eb` | `test: apply pre-slice 3c ledger redisposition` | Atomic resolution-only ledger cutover. |
| `f152ed8a` | `test: bind redisposition candidate evidence` | Final-fix carrier and TDD binding hardening. |
| `e79e7e52` | `test: accept corrected redisposition evidence` | Accepted corrective blind evidence, artifact, ledger, and verification update. |
| `28c0d3c3` | `test: add single-row node contract owners` | Initial additive Slice 3C Task 4 owners. |
| `746bdb45` | `test: fix single-row contract owners` | First Task 4 review fix and SC-000464 five-owner correction. |

The program index `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md` was intentionally not changed. This checkpoint does not write a Slice 3C implementation plan; that remains a separate post-approval action.
