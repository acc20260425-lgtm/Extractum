# Testing Redesign Slice 3A: Source Contracts Verification

## Checkpoint Context

- Final verified implementation checkpoint: `9640cb89` (`test: remove
  duplicate Grammers metadata check`).
- The completion audit also removed a duplicate full-corpus source-reader scan
  from the unit suite and made the single transition-validator scan include
  every census-owned Vitest path. It did not add a test owner, selector,
  scheduler, timing state, Nx configuration, CI, or hosted tooling.
- The authoritative full gate was run outside the sandbox so its Windows
  process-tree checks exercised the real operating-system behavior.

## Migration Result

Slice 3A closed 187 frozen ledger rows while preserving their stable IDs,
hashes, lineage, assertion counts, and ordinal-complete mixed-row partitions:

| Removed legacy path | Closed rows | Result |
| --- | ---: | --- |
| `src/lib/telegram-crate-boundary-contract.test.ts` | SC-000561--SC-000658 (98) | Direct Vitest/Cargo behavior, four narrow structured rules, a bounded transition-only Cargo identity comparison, and explicit source-shape deletions. |
| `src/lib/analysis-source-readers.test.ts` | SC-000221--SC-000278 (58) | 47 rendered component behavior contracts, six parsed TypeScript/Svelte rules, and explicit deletion of assertions that jsdom cannot prove truthfully. |
| `src/lib/analysis-application-contract.test.ts` | SC-000029--SC-000059 (31) | Exact Cargo/Vitest identities, one direct 21/3/3 Tauri command-inventory test, and explicit ordinal-complete deletion of residual source shape. |

The live validator now reports 671 total rows; the open count fell from 670
before the first cutover to 483. It also reports 194 runner candidates: 187
Vitest files and 7 Playwright files, with bidirectional ownership intact.

## Structured Authority

The final registry is exactly these ten implemented IDs:

```text
rule:analysis-evidence-highlight-token-styling
rule:analysis-source-browser-canonical-composition
rule:analysis-source-browser-explicit-subject-contract
rule:analysis-source-group-activity-boundary
rule:analysis-source-group-tab-leaf-boundary
rule:analysis-source-reader-surface-composition
rule:telegram-crate-dependency-ownership
rule:telegram-crate-manifest-boundary
rule:telegram-phase-8b-authority-integrity
rule:telegram-repository-path-safety
```

The completeness test derives the permitted IDs from the three migrated
ranges, asserts exact set equality with the registry, and requires an
independent positive fixture and non-empty violating mutation for every ID.
The rejected `rule:repository-index:analysis-application-boundary` and the
unanchored `rule:analysis-crate-manifest-boundary` are not implemented.

The broad placeholder still appears only in open SC-000115, SC-000116, and
SC-000128 under `src/lib/analysis-redesign-safety-contract.test.ts`. Those
rows were outside the completed ranges; the placeholder is unresolved future
ledger work, not Slice 3A evidence.

## Truthful Retentions and Accepted Losses

Telegram retains behavior and semantic package/dependency boundaries without
a Node Rust parser or persistent Cargo census. Checkpoint-relative paths,
exact item/file placement, private call counts, forbidden-token scans, and
other historical source shape were explicitly retired rather than replaced
with nominal tests.

The three Telegram losses called out by the migration plan are explicit:

- SC-000628 retires static proof that the private plaintext credential row
  cannot acquire a `Debug` derive; secret-store and encrypted-envelope behavior
  remains tested.
- SC-000655 retires the exact application-facade path and occurrence count;
  package edges and direct Grammers dependency ownership remain enforced.
- SC-000657 retires exact fixture item names, aliases, `cfg` placement,
  visibility, and producer-consumer inventory; durable Cargo feature-edge
  ownership remains enforced.

Analysis source-reader behavior is exercised through rendered components.
Six rules use parsed TypeScript/Svelte facts and fail closed on malformed
declared inputs. The scoped CSS layout/hyphenation assertions that jsdom could
not observe were deleted; no production CSS text scan or new browser harness
was introduced.

The application cutover keeps direct outcomes and the generic
`application_command_inventory!` evidence used by both the real Tauri handler
and `analysis_command_registration_inventory_is_exact`. It deliberately does
not add a Rust/SQL parser, fingerprints, a persistent census, or a broad
manifest rule. The accepted losses include:

- SC-000032 exact event-sink body and hidden-side-effect source shape;
- SC-000053--SC-000055 exact transaction topology while retaining the direct
  observable transaction outcomes;
- SC-000058 global Rust/SQL reachability, SQL reconstruction, ownership, and
  transaction-control static proof;
- SC-000031 ordinal 1, SC-000036 ordinal 1, and SC-000057 ordinal 9, corrected
  from an unanchored manifest claim to explicit source-shape deletion.

SC-000058 is the highest accepted architecture risk: the project no longer has
a global static proof that no portable analysis path bypasses the intended SQL
owner. Recreating that proof would require the parser/scanner infrastructure
that this simplification intentionally rejects.

## Fresh Verification Evidence

Focused final-state checks:

| Command | Result |
| --- | --- |
| `node scripts/run-vitest.mjs run scripts/testing/repository-rules.test.ts` | PASS, 1 file and 15 tests in 9.97 s; exact ten-ID registry and per-ID mutations. |
| `node scripts/validate-testing-transition.mjs` | PASS; 194 candidates, 671 rows, 483 open. |
| `git diff --check` | PASS. |

Task 4's focused package evidence at the implementation checkpoint also
passed: `extractum-analysis` check and 112/112 tests, `extractum` check and
674/674 tests, and the exact analysis command inventory 1/1.

After `9640cb89`, the final authoritative unsandboxed `npm.cmd run verify`
exited `0` in 321.3 seconds. Its non-empty owner results were:

| Owner | Files | Tests | Observed duration |
| --- | ---: | ---: | ---: |
| `test:unit` | 85 | 808 | 19.57 s |
| `test:component` | 20 | 168 | 44.12 s |
| `test:architecture` | 1 | 2 | 0.25 s |
| `test:legacy-contract` | 75 | 522 | 13.07 s |
| `test:integration:os` | 6 | 87 | 6.13 s |
| `test:e2e` | 7 census-owned files | 79 | PASS; separate duration not retained in the truncated console result |

The same run passed sidecar validation, transition validation, Svelte check,
rustfmt, Cargo workspace check/test, and the Git diff gate. Earlier successful
observations remain recorded rather than substituted: 366.9 seconds at the
Task 4 checkpoint and 481.5 seconds after `6cfc9679` (unit 807, component 168,
architecture 2, legacy 522, OS 87, and Playwright 79).

The completion audit also retained its failed observations. One attempt hit
the external 903.4-second command limit and closed Cargo's output pipe; a clean
retry then exposed a five-second timeout in the unit convention test's
duplicate full-corpus reader scan. After moving live coverage solely to the
transition validator, while expanding it to census-owned Vitest specs, unit
passed 808/808. The next full run exposed a separate 15-second timeout in a
legacy Grammers baseline test that launched live Cargo metadata twice. The
legacy test now checks the pure generator and fixed artifact, while the
transition rule remains the single live locked-Cargo comparison. Its owning
file passed 12/12 before the final full PASS.

No automated 90-second timing warning was printed by the current sequential
`verify` implementation; adding that observational warning remains future
scheduler work. The measured command nevertheless exceeded 90 seconds without
changing its successful result. The focused Task 3 component behavior took
24.59--26.46 seconds; the component project was 82.20 seconds cold-ish and
39.54 seconds warm, then 79.29 and 103.31 seconds in prior full runs and 44.12
seconds in the final run. Therefore the current component owner is
unambiguously `slow` for Slice 4's 15-second feedback contract.

## Remaining Cohort Handoff

The next plan must re-read the live ledger. At this checkpoint the two largest
remaining path cohorts are:

1. `src/lib/research-projects-route-contract.test.ts` -- 26 rows;
2. `src/lib/analysis-report-canvas.test.ts` -- 20 rows.

This record does not plan, classify, or migrate either cohort. Slice 3B
planning is the next program activity. Nx remains unselected and deferred
until Slice 4 is complete.
