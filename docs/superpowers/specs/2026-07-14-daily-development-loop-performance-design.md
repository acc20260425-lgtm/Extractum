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
| `vitest run --changed`, clean tree | 4.11 s | Code 0, `No test files found` |
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

The full 156-file Vitest inventory comprises 145 files under `src`, five
sidecar unit-test files, four research-adapter unit-test files, and two script
test files. The separately managed research-adapter e2e directory is excluded
by `run-vitest.mjs`. The node/jsdom count below refers only to the 145 files
under `src`, not to the complete 156-file suite.

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
    +-- dirty frontend change -> working-tree changed Vitest set
    +-- checkpoint commit -----> last-commit changed Vitest set
    +-- explicit source -------> related Vitest set
    +-- Svelte/UI change ------> applicable focused set + svelte-check
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
"test:changed": "node scripts/run-vitest.mjs run --changed",
"test:changed:last": "node scripts/run-vitest.mjs run --changed=HEAD~1",
"test:related": "node scripts/run-vitest.mjs related --run",
"test:rust": "cargo test --manifest-path src-tauri/Cargo.toml --lib"
```

Example usage:

```powershell
npm.cmd run test:changed
npm.cmd run test:changed:last
npm.cmd run test:related -- src/lib/some-model.ts
npm.cmd run test:rust -- prompt_packs::runtime_config
```

`test:changed` covers staged, unstaged, and untracked Git changes.
`test:changed:last` adds the most recent checkpoint commit (`HEAD~1...HEAD`) to
that working-tree set. The second form is necessary in this repository because
small checkpoint commits leave the working tree clean frequently. For an
older branch or comparison point, the equivalent explicit form is
`npm.cmd run test -- --changed=<base>`.

`test:related` remains the explicit tool for one or more known source paths.
No changed-file command is universally primary: use the working-tree form
before a checkpoint and the last-commit/base form after one.

`run-vitest.mjs` exports a pure `normalizeRelatedFileArgs` helper and uses it
before spawning Vitest. For the `related` command, an argument is treated as a
file operand when it does not begin with `-` and resolves to an existing file
under the supplied/current working directory. Those operands are normalized
from Windows backslashes to forward slashes. Both
`src/lib/some-model.ts` and `src\lib\some-model.ts` therefore address the same
file. Options, non-file test-name patterns such as `-t "foo\bar"`, and
arguments to commands other than `related` remain unchanged. The wrapper gains
the same guarded-entrypoint pattern already used by `scripts/tauri.mjs`, so it
can be imported without starting Vitest.

`test:changed` and `test:related` are accelerators, not correctness gates.
Vitest derives related tests from static imports; dynamic or external
relationships may not be visible. In Vitest 4.1.5, `--changed` has been
observed and must be reverified to exit successfully with code 0 when no
related tests are found. A developer must use an explicit
test file, a wider focused test, or the full suite when the relationship is not
represented in the module graph or the selected set is unexpectedly empty.

The existing `test:rust:prompt-pack-runs` script keeps its behavior and filter
but drops its slice-specific `--target-dir`. It reuses the canonical Cargo
cache.

The slice does not split Vitest into node and DOM projects. Vitest already uses
`node` by default: 126 of the current 145 frontend test files use that default,
while 19 DOM component files opt into jsdom with per-file
`@vitest-environment jsdom` directives. A project split would add
configuration and startup surfaces without removing a repository-wide DOM
cost, because no such global cost exists.

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
`CARGO_PROFILE_DEV_DEBUG=2` alone is not a valid escape hatch: Cargo's
`[profile.dev.package."*"]` override has higher precedence and would keep
dependency debug information disabled.

When dependency-level native inspection is required, the supported procedure
is an explicit temporary edit of both permanent settings:

1. Start from a clean worktree.
2. Temporarily change `[profile.dev] debug` to `2` and
   `[profile.dev.package."*"] debug` to `2` in `src-tauri/Cargo.toml`.
3. Point `CARGO_TARGET_DIR` at an absolute, isolated native-debug directory.
4. Start the normal MCP-enabled Tauri wrapper.
5. After the session, restore both Cargo settings and verify that the only
   remaining diff is intentional work.

For example, after the temporary manifest edit:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location).Path "src-tauri/target/native-debug"
npm.cmd run tauri dev
```

This procedure is intentionally manual rather than hidden in a script: the
manifest diff makes the exceptional compilation mode visible and reviewable.
It retains the normal MCP-enabled wrapper. The isolated target must not become
an input to ordinary checks and should be removed manually when no longer
needed. The temporary profile edit must never be committed.

## Documentation and Ownership

Update `AGENTS.md` with:

- the subsystem-based inner-loop matrix;
- the canonical-target rule;
- the fact that `test:changed` and `test:related` are not full gates;
- the native-debug escape hatch;
- continued use of `npm.cmd` on Windows.

Update `docs/project.md` verification guidance with the same public workflow
at a less agent-specific level. `scripts/verify.mjs` keeps its sequence and
coverage. It benefits from the Vitest and Cargo configuration automatically.

Both documentation files receive a stable
`<!-- daily-development-loop -->` anchor. The source contract checks only this
anchor, not the surrounding prose, so editorial rewrites do not break tests.

## Source Contract

Add `src/lib/development-loop-performance-contract.test.ts`. It should read
the relevant files as raw text, normalize CRLF before textual assertions, and
parse `package.json` for script assertions.

The contract verifies evaluated configuration, machine-readable configuration,
and stable documentation anchors. It does not assert human prose. Specifically,
it verifies that:

- importing the default `vite.config.js` export and resolving its async config
  factory produces `config.test.pool === "threads"` and a `test` object without
  an own `maxWorkers` property;
- no separate root `vitest.config.{js,ts,mjs,mts,cjs,cts}` exists that could
  silently supersede the selected Vite configuration;
- `test:changed` uses the existing `run-vitest.mjs` wrapper and Vitest's
  `run --changed` option;
- `test:changed:last` uses `run --changed=HEAD~1`;
- `test:related` uses the existing `run-vitest.mjs` wrapper and Vitest's
  `related --run` command;
- `test:rust` uses the canonical manifest, `--lib`, and no `--target-dir`;
- `test:rust:prompt-pack-runs` contains no `--target-dir`;
- the Cargo dev profile uses `line-tables-only` for workspace code and disables
  dependency debug information;
- `AGENTS.md` and `docs/project.md` contain the stable
  `<!-- daily-development-loop -->` anchor.

The contract checks repository-owned configuration, not historical documents
or filesystem contents under ignored `src-tauri/target`.

The same test file imports `normalizeRelatedFileArgs` as ordinary executable
code. Behavioral cases prove that an existing backslash-separated related
path is normalized, `-t "foo\bar"` is unchanged, a non-existing operand is
unchanged, and the same existing path is unchanged for a non-`related`
command. These are unit tests of the wrapper API, not textual source-contract
assertions.

## Failure Handling

### Thread-pool regressions

Before committing the pool change, scan test sources for `process.chdir()` and
direct assignment to or deletion from `process.env`. Worker threads do not
support `process.chdir()`, and direct environment mutation may leak between
files executed by the same worker. The current test tree contains neither
pattern; read-only references and source-contract string fixtures are not
violations.

If repeated full-suite runs reveal a test that depends on process-global
state, do not revert the entire suite to forks and do not disable isolation
globally. Identify the state owner and either fix the test or isolate only the
affected files in a sequential/forked Vitest project.

### Changed/related-test gaps

No-tests-found or an unexpectedly small changed/related set is a signal to run
a known test file or the full suite. A clean tree is the common expected cause
of an empty `test:changed` set in this checkpoint-heavy repository; use
`test:changed:last` or an explicit base immediately after a commit. Vitest
4.1.5 currently exits successfully on an empty changed set, and verification
locks down that observed behavior. None of the focused commands may be
presented as a merge gate.

### Cargo cache cleanup

Cache cleanup is an explicit operational step, not application logic. Before
running it, verify that the worktree is safe and no Cargo or Tauri process is
using the target directory. Failure or interruption can only remove
rebuildable artifacts; it must not touch source files, application data, or
`node_modules`.

Because cleanup is destructive and makes the next build cold, execution
requires a separate user confirmation. Project scripts do not call
`cargo clean` automatically.

## Implementation Sequencing Constraint

The implementation plan must request the cleanup decision before the first
Cargo invocation that sees the new profile settings.

1. Implement and commit the frontend pool, scripts, wrapper behavior, and
   their frontend tests without changing the Cargo profile.
2. On that clean checkpoint, verify the empty-tree `test:changed` result and
   `test:changed:last`. Then make one isolated reversible source edit, verify
   the working-tree changed set, and restore the edit.
3. Ask whether the one-time Cargo cleanup is approved.
4. Only then apply the permanent Cargo profile edit.
5. If cleanup is approved, immediately confirm process safety and clean before
   any Cargo check/test/build. If it is declined, proceed directly to the one
   profile-triggered cold rebuild and do not clean the newly warmed cache in
   the same execution.
6. Run Cargo correctness checks, timings, and the full verification gate only
   after the selected cleanup branch is complete.

This ordering prevents the automated-correctness section from warming the new
profile and then deleting it.

## Verification Strategy

### Automated correctness

Execute these checks in the phases defined by the Implementation Sequencing
Constraint above. Their list order does not authorize an early Cargo run with
the new profile.

- Run the new source-contract test.
- Run a preflight source scan proving there is no `process.chdir()` call or
  direct `process.env` mutation in the test inventory. The scan must match
  executable mutation syntax, not harmless reads or quoted fixture text.
- Run the full Vitest suite three times with the committed thread-pool
  configuration. The passing file and test inventories must not decrease from
  the pre-change baseline; additions made by this slice are expected.
- After committing the frontend configuration and returning to a clean tree,
  run `test:changed` and record its exit code and no-tests output.
- On that same checkpoint, run `test:changed:last`, then make a reversible
  uncommitted change to a known source file and confirm that `test:changed`
  executes a nonzero set. Restore the probe edit immediately afterward.
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
- changed-test duration and executed test count;
- focused related-test duration and executed test count;
- no-op Cargo check and test durations;
- the first Cargo check/test after the committed profile change;
- the current `cargo check --timings` report and its dominant compilation
  units;
- target-directory size before and after any separately approved cleanup and
  canonical cache warm-up.

The previously observed 60-70-second Vitest range is contextual evidence for
the baseline machine, not a portable pass/fail threshold. Acceptance depends
on a material median improvement over the same-machine pre-change baseline.

### One-time cleanup and warm-up

After separate approval:

1. Apply the permanent profile settings, but do not run Cargo with the new
   profile yet.
2. Confirm there are no running Cargo, rustc, Tauri, or application processes
   using `src-tauri/target`.
3. Run `cargo clean --manifest-path src-tauri/Cargo.toml`.
4. Rebuild the canonical cache with `cargo check --timings` and full
   `cargo test`. Preserve the generated report under
   `src-tauri/target/cargo-timings` as performance evidence before any later
   cache maintenance.
5. Record cold and subsequent no-op durations.
6. Record the rebuilt target size and confirm that normal commands create no
   `codex-*` target directories.

The cold rebuild is expected to be slow and is not compared directly with the
old partially warmed measurements. The profile change and cleanup should form
one cold-cache event: do not warm the new profile and then immediately delete
that new cache. If cleanup is declined, accept the profile-triggered cold
rebuild and leave old artifacts for a later, separately chosen maintenance
window.

## Acceptance Criteria

1. The full Vitest suite passes repeatedly with the thread pool and retains
   its complete test inventory.
2. The baseline machine's median full Vitest duration is materially below its
   same-machine pre-change baseline. The previously observed 60-70-second
   range is evidence, not a hard acceptance threshold.
3. Working-tree, last-checkpoint, explicit-related, and focused Rust commands
   are documented and executable; Rust commands reuse the canonical Cargo
   target.
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

1. Analyze the stable Cargo timing report captured during this slice.
2. Profile the approximately 19-second Rust test execution floor; evaluate
   disk-backed SQLite fixtures, real sleeps/timeouts, serialization, and
   whether `cargo-nextest` improves safe test-level parallelism.
3. Benchmark the bundled `rust-lld` linker against the default MSVC linker in
   an isolated target and verify both focused test linking and a complete Tauri
   release smoke before considering repository configuration. The current
   toolchain contains `rust-lld.exe` and a small `-C linker=rust-lld` probe
   succeeds, but neither fact proves compatibility or improvement for the full
   Extractum binary.
4. Evaluate workstation-level `sccache` for branch switches and cold rebuilds.
5. Extract genuinely independent backend domains into workspace crates only
   if timings show the root crate remains the dominant incremental cost.

These follow-ups require their own design and must not be folded into this
configuration slice without new evidence.
