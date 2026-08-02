# Testing Redesign Slice 2A Migration-Preflight Verification

## Checkpoint Identity

- Program-plan base: `72a00b3839082ed610991a6568cc0a189da8d5c5`
  (`docs: plan testing redesign slice 2a`).
- Slice 2A implementation base after its approved prerequisites:
  `f081cb90c589e80873b936960749626563e5511f`
  (`test: cover non-regular sidecar binary`).
- Final implementation checkpoint:
  `d533444539c664b2d6834cae9297667a853b1806`
  (`test: freeze source contract migration ledger`), whose exact parent is
  the docs-reconciliation and frozen-parent checkpoint
  `0045abf6006f42fe735d5142e139375d702d4fe9`
  (`docs: reconcile slice 2a source ledger freeze`).

This record describes the committed Slice 2A code checkpoint. The
documentation commit that adds this record is evidence about that checkpoint,
not a change to the census or ledger.

## Live Migration Preflight

The transition validator at the final checkpoint reported:

```text
Runner census: 190 filesystem candidates, 184 Vitest files, 6 Playwright files
Source-contract ledger: 671 rows, 671 open
```

The bidirectional census has zero runner intersections, exceptions, unowned
filesystem candidates, and collected non-candidate files. Each owner is
non-empty. The ledger is the reviewed, activated baseline and has:

| Ledger property | Count |
| --- | ---: |
| Rows | 671 |
| Behavior dispositions | 372 |
| Architecture dispositions | 277 |
| Delete dispositions | 22 |
| Manual rows | 206 |
| Static rows | 465 |
| Subgroups | 0 |
| Exact source-reader exceptions | 1 |

The one exception is the reviewed exact fixture-reader exception; it is not a
broad file-level exemption. Every current source-dependent declaration is
represented by one open row. The `sourceHash` and optional `authorityHash`
fields are obligation-identity fields for exact source slices and authority
text only; they are not performance fingerprints and do not affect selection
or eligibility.

## Fresh-Checkout Bootstrap Contract

Before the first verification in a fresh checkout or worktree, and after a
sidecar-packaging change, run:

```powershell
npm ci
npm.cmd run bootstrap:testing
npm.cmd run verify
```

`bootstrap:testing` builds the ignored Gemini Browser sidecar and validates the
resulting non-empty regular binary; it may download the `pkg` runtime cache.
`verify` checks that prerequisite first and never builds it. Release `tauri
build` retains its separate `build:tauri-prereqs` behavior.

## Verification Evidence

Fresh focused verification at the final implementation checkpoint passed:

```powershell
node scripts/run-vitest.mjs run scripts/check-gemini-browser-sidecar-binary.test.ts scripts/verify.test.ts scripts/testing/validate-testing-transition.test.ts scripts/testing/slice-1-rust-feasibility.test.ts
npm.cmd run check
node scripts/validate-testing-transition.mjs
git diff HEAD --check
```

The focused Vitest command passed 4 files and 79 tests. `npm.cmd run check`
reported zero errors and zero warnings. The transition command produced the
190-file census and 671-row ledger summary above, and the diff check passed.

The controller also obtained a fresh authoritative unsandboxed
`npm.cmd run verify` at the exact final implementation checkpoint. It exited
successfully in the required order: the sidecar prerequisite was first, the
transition validation was second, then the existing Vitest, Svelte, Rust, and
Git gates completed. That gate reported 184 of 184 Vitest files and 1,745 of
1,745 tests passing, zero Svelte errors and warnings, and successful rustfmt,
Cargo workspace check/test, and Git diff validation.

Earlier Windows full-Vitest lifecycle timeout observations remain classified
as the known process-tree baseline flake. Focused process suites passed outside
the sandbox, and the final uninterrupted authoritative public gate passed; no
out-of-scope process-lifecycle change was made in Slice 2A.

## Scope and Timing Guardrails

The implementation-range review found no diff for
`scripts/testing/timing-log.mjs`. A direct search found no `timing-log`,
`recordTiming`, or `appendTiming` reference in the transition validator,
transition carrier, extractor, or `testing/` tree. Slice 2A added no timing
mechanism and captured no duration.

The retained ignored Slice 2A artifacts are the approved preflight drafts under
`artifacts/testing/slice-2a/`; review reports remain under `.superpowers/sdd/`.
No temporary diagnostic runner, superseded draft outside that canonical
artifact location, or log remains. Slice 2A does not add GitHub Actions, branch
protection, a scheduler, a test manifest, or Nx configuration.
