# Daily Development Loop Performance Design

## Status

Approved for implementation planning on 2026-07-14.

## Context

Extractum has one comprehensive verification pipeline, but that pipeline is
too expensive to use as the feedback loop after every small change. The
current repository combines a large Svelte/Vitest surface with one large Rust
crate containing Tauri, SQLx, provider, source-ingest, analysis, and Prompt
Pack code.

Measurements on the current Windows development machine show distinct costs:

| Operation | Observed wall time | Notes |
| --- | ---: | --- |
| Full Vitest suite, current default pool | 130.49 s | 156 files, 1,253 tests |
| Full Vitest suite, `threads`, automatic workers | 65.09 s | Same files and tests |
| Full Vitest suite, `threads`, 2 workers | 78.51 s | Slower than automatic selection |
| Full Vitest suite, `threads`, 4 workers | 63.92 s | Similar to automatic selection |
| `npm.cmd run check` | 40.37 s | Zero diagnostics |
| `npm.cmd run build` | 68.46 s | Frontend build only |
| `cargo check`, root crate required rebuilding | 76.17 s | Current dev profile |
| no-op `cargo check` | 1.15 s | Shared default target directory |
| full `cargo test`, resumed after an earlier 120 s timeout | 171.82 s | Tests themselves took 19.04 s |
| no-op full `cargo test` | 22.72 s | Tests themselves took 18.83 s |

The interrupted first Rust test attempt means 171.82 seconds is not a clean
cold-build measurement. It still demonstrates that compiling the test target
dominates execution of the tests. A historical release-build log records five
minutes and eleven seconds for the Rust release profile, before frontend
prerequisites.

`src-tauri/target` is approximately 357 GiB. About 223 GiB is the canonical
`debug` directory. The rest includes many slice-specific `codex-*` target
directories that duplicate the same dependency graph. The debug directory
also contains large incremental artifacts and temporary archives left by
interrupted builds.

The first optimization slice targets the daily feedback loop. It must not
reduce the coverage or authority of the full `npm.cmd run verify` gate.

## Goals

- Cut the full frontend test time approximately in half on the measured
  machine without changing test semantics or isolation globally.
- Provide explicit focused commands for frontend and Rust changes.
- Make the canonical `src-tauri/target` directory the default cache for normal
  development, plans, and verification.
- Reduce ordinary dev/test debug-information generation while retaining
  useful file/line information.
- Preserve a documented escape hatch for rare full native Rust debugging.
- Prevent accidental regression of the repository-owned configuration.
- Record before/after evidence without turning machine-dependent durations
  into flaky test assertions.

## Non-Goals

- No split of `extractum_lib` into multiple workspace crates in this slice.
- No installation or required use of `sccache`, an alternative linker,
  `cargo-nextest`, nightly Rust, or machine-global configuration.
- No dependency upgrade or feature-pruning campaign.
- No change to release-profile optimization or application behavior.
- No weakening, removal, or automatic replacement of the full verification
  pipeline.
- No automatic deletion of Cargo artifacts.
- No rewrite of archived plans or historical verification documents that
  mention old target directories.
- No update to `docs/value-registry.md`, because no runtime or UI string value
  is introduced, removed, or redefined.

## Considered Approaches

### 1. Repository-owned inner-loop configuration

Use Vitest's thread pool, add focused commands, reuse one Cargo target, and
reduce dev/test debug information. Keep full verification as the final gate.

This is the selected first slice because the frontend improvement is already
measured, the changes are repository-local, and each optimization can be
reverted independently.

### 2. Machine-level compiler and linker acceleration

Install `sccache` and evaluate a faster Windows linker. This may improve branch
switches, clean builds, and linking, but it introduces workstation setup and
does not address the measured Vitest cost. `sccache` is not installed on the
current machine. This remains a follow-up if repository-owned changes leave a
material bottleneck.

### 3. Split the Rust backend into workspace crates

Move independent domains into smaller crates so a local edit recompiles less
code. This has the highest long-term potential for Rust incremental builds,
but the current modules share Tauri state, SQLx types, error contracts, and
backend integrations. It is a separate architecture project, not a safe first
optimization slice.

## Selected Architecture

The repository keeps two verification layers:

```text
small change
    |
    +-- frontend helper/model --> related Vitest files
    +-- Svelte/UI change ------> related Vitest files + svelte-check
    +-- Rust change -----------> focused Rust lib test + rustfmt + cargo check
    |
    `-- merge/push gate -------> full npm.cmd run verify
```

The inner loop chooses checks based on the changed subsystem. The full gate
continues to run the complete Vitest suite, Svelte check, rustfmt check, Cargo
check, full Cargo test, and diff check.

## Frontend Test Configuration

Add a Vitest configuration block to `vite.config.js`:

```js
test: {
  pool: "threads",
},
```

Do not hardcode `maxWorkers`. Automatic worker selection completed the suite
in approximately 65 seconds on the four-logical-processor baseline machine.
Four explicit workers produced a statistically similar single run, while two
workers were slower. Leaving worker selection adaptive is more portable than
encoding this workstation's processor count.

The existing conditional `svelteTesting()` plugin behavior remains unchanged.
The slice does not disable Vitest isolation, make suites concurrent, or change
the test environment.

Add these package scripts:

```json
"test:related": "node scripts/run-vitest.mjs related --run",
"test:rust": "cargo test --manifest-path src-tauri/Cargo.toml --lib"
```

Example usage:

```powershell
npm.cmd run test:related -- src/lib/some-model.ts
npm.cmd run test:rust -- prompt_packs::runtime_config
```

`test:related` is an accelerator, not a correctness gate. Vitest derives
related tests from static imports; dynamic or external relationships may not
be visible. A developer must use an explicit test file, a wider focused test,
or the full suite when the relationship is not represented in the module
graph.

The existing `test:rust:prompt-pack-runs` script keeps its behavior and filter
but drops its slice-specific `--target-dir`. It reuses the canonical Cargo
cache.

## Cargo Target-Directory Policy

Normal commands use Cargo's canonical `src-tauri/target` directory. Active
package scripts, new plans, and agent instructions must not create a
`src-tauri/target/codex-*` directory merely to isolate a task.

The policy applies to ordinary sequential development. A deliberately
isolated native-debug or concurrent experiment may use a separate target only
when its reason, owner, and cleanup are explicit. Such a directory is not the
default verification path.

Historical plans and verification evidence are immutable records and are not
rewritten to remove their old command lines.

## Cargo Dev/Test Profile

Add:

```toml
[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false
```

Cargo's existing fast dev defaults remain in force: incremental compilation,
no optimization, and high codegen-unit parallelism. They are not restated.
The built-in test profile inherits the dev profile, so ordinary Cargo tests
receive the same debug-information policy.

Workspace code retains file/line debug information. Dependencies omit debug
information. This preserves useful backtraces and source locations while
reducing compiler work and artifact size. Browser DevTools, Tauri MCP Bridge,
Rust debug assertions, and application behavior are unaffected.

Full native inspection of dependency variables is not the ordinary workflow.
When it is required, the developer may override the dev debug setting for that
session and use an explicitly isolated temporary target, for example in
PowerShell:

```powershell
$env:CARGO_PROFILE_DEV_DEBUG = "2"
$env:CARGO_TARGET_DIR = Join-Path (Get-Location).Path "src-tauri/target/native-debug"
npm.cmd run tauri dev
```

That escape hatch retains the normal MCP-enabled wrapper. Its target directory
must not become an input to ordinary checks and should be removed manually
when no longer needed.

## Documentation and Ownership

Update `AGENTS.md` with:

- the subsystem-based inner-loop matrix;
- the canonical-target rule;
- the fact that `test:related` is not a full gate;
- the native-debug escape hatch;
- continued use of `npm.cmd` on Windows.

Update `docs/project.md` verification guidance with the same public workflow
at a less agent-specific level. `scripts/verify.mjs` keeps its sequence and
coverage. It benefits from the Vitest and Cargo configuration automatically.

## Source Contract

Add `src/lib/development-loop-performance-contract.test.ts`. It should read
the relevant files as raw text, normalize CRLF before textual assertions, and
parse `package.json` for script assertions.

The contract verifies that:

- `vite.config.js` configures Vitest with `pool: "threads"`;
- no fixed `maxWorkers` is introduced in the selected configuration;
- `test:related` uses the existing `run-vitest.mjs` wrapper and Vitest's
  `related --run` command;
- `test:rust` uses the canonical manifest, `--lib`, and no `--target-dir`;
- `test:rust:prompt-pack-runs` contains no `--target-dir`;
- the Cargo dev profile uses `line-tables-only` for workspace code and disables
  dependency debug information;
- active workflow guidance names the canonical target policy and retains the
  full `verify` gate.

The contract checks repository-owned configuration, not historical documents
or filesystem contents under ignored `src-tauri/target`.

## Failure Handling

### Thread-pool regressions

If repeated full-suite runs reveal a test that depends on process-global
state, do not revert the entire suite to forks and do not disable isolation
globally. Identify the state owner and either fix the test or isolate only the
affected files in a sequential/forked Vitest project.

### Related-test gaps

No-tests-found or an unexpectedly small related set is a signal to run a
known test file or the full suite. `test:related` must not be presented as a
merge gate.

### Cargo cache cleanup

Cache cleanup is an explicit operational step, not application logic. Before
running it, verify that the worktree is safe and no Cargo or Tauri process is
using the target directory. Failure or interruption can only remove
rebuildable artifacts; it must not touch source files, application data, or
`node_modules`.

Because cleanup is destructive and makes the next build cold, execution
requires a separate user confirmation. Project scripts do not call
`cargo clean` automatically.

## Verification Strategy

### Automated correctness

- Run the new source-contract test.
- Run the full Vitest suite three times with the committed thread-pool
  configuration; every run must report all 156 files and 1,253 tests passing,
  unless the test inventory intentionally changes in the same branch.
- Run `test:related` for a known source file and confirm that it executes a
  nonzero related set.
- Run one focused Rust module through `test:rust`.
- Run full `cargo test` with the canonical target.
- Run `npm.cmd run check` and `npm.cmd run check:rustfmt`.
- Run the unchanged full `npm.cmd run verify` gate.

### Performance evidence

Durations are recorded in a verification document, not asserted in tests.
Record at least:

- three full Vitest durations and their median;
- focused related-test duration and executed test count;
- no-op Cargo check and test durations;
- the first Cargo check/test after the committed profile change;
- target-directory size before and after any separately approved cleanup and
  canonical cache warm-up.

The expected Vitest result on the baseline machine is approximately 60-70
seconds rather than 130 seconds. This is an evidence target, not a portable
pass/fail threshold.

### One-time cleanup and warm-up

After separate approval:

1. Confirm there are no running Cargo, rustc, Tauri, or application processes
   using `src-tauri/target`.
2. Run `cargo clean --manifest-path src-tauri/Cargo.toml`.
3. Rebuild the canonical cache with `cargo check` and full `cargo test`.
4. Record cold and subsequent no-op durations.
5. Record the rebuilt target size and confirm that normal commands create no
   `codex-*` target directories.

The cold rebuild is expected to be slow and is not compared directly with the
old partially warmed measurements.

## Acceptance Criteria

1. The full Vitest suite passes repeatedly with the thread pool and retains
   its complete test inventory.
2. The baseline machine's median full Vitest duration is materially below the
   130.49-second baseline, with approximately 60-70 seconds as the expected
   range.
3. Focused frontend and Rust commands are documented, executable, and reuse
   existing wrappers and the canonical Cargo target.
4. Active repository scripts contain no slice-specific Cargo target directory.
5. Ordinary dev/test compilation uses reduced debug information without
   changing release settings, debug assertions, MCP behavior, or application
   behavior.
6. Full native Rust debugging has a documented opt-in path separate from the
   ordinary cache.
7. The full `npm.cmd run verify` gate remains authoritative and passes.
8. Cache deletion is never automatic and occurs only after explicit approval.
9. Before/after performance and disk evidence is recorded without introducing
   machine-dependent timing assertions.

## Follow-Ups

Re-measure after this slice. If Rust source edits still make the inner loop
unacceptably slow, investigate in this order:

1. stable `cargo --timings` evidence for the current crate and dependency
   graph;
2. workstation-level `sccache` and linker evaluation;
3. extraction of genuinely independent backend domains into workspace crates.

These follow-ups require their own design and must not be folded into this
configuration slice without new evidence.
