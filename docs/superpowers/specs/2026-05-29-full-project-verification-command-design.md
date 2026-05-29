# Full Project Verification Command Design

> Date: 2026-05-29
> Status: approved design
> Scope: local verification command and documentation; no CI, dependency
> pinning, lint policy, or runtime behavior changes.

## Goal

Add one documented full-project verification entry point that developers can
run before committing or merging work.

The command should wrap the verification steps already used manually on this
project, make the expected gate easy to remember, and reduce drift between
frontend, backend, and whitespace checks.

## Current Context

The repository already has:

- `package-lock.json`;
- `src-tauri/Cargo.lock`;
- npm scripts for frontend tests and Svelte checks;
- no checked-in `.github` workflow directory;
- no existing `scripts/` verification helper.

The active backlog tracks several stabilization items. This design covers only
the first one:

```text
add a single documented full-project verification command or script
```

CI, Rust formatting, Rust linting, `grammers-*` dependency pinning, event-driven
runtime checks, and secret-safety audits remain separate backlog slices.

## Chosen Approach

Add a cross-platform Node runner exposed through:

```text
npm run verify
```

The runner executes the known-good manual verification gates in order:

1. `npm run test`
2. `npm run check`
3. `cargo check --manifest-path src-tauri/Cargo.toml`
4. `cargo test --manifest-path src-tauri/Cargo.toml`
5. `git diff --check`

This keeps the first stabilization slice small and useful. It also gives a
future CI workflow a single local contract to call instead of duplicating a
long command list immediately.

## Command Contract

The verification command should:

- run from the repository root;
- print a clear section header before each step;
- stop at the first failing command;
- forward the failing command exit code;
- avoid hiding stdout or stderr from the underlying tools;
- work on Windows by selecting `npm.cmd` when invoking npm subprocesses;
- work on non-Windows platforms by selecting `npm`;
- avoid downloading or installing dependencies itself.

The command is an automation wrapper, not a new test framework. It should not
reinterpret test output, retry failures, mutate source files, or make policy
decisions about warnings.

## Documentation

The implementation should document `npm run verify` in the project docs near
the existing development or verification guidance, then mark the backlog item
for a single documented verification command as complete.

Documentation should keep platform guidance concise. Historical notes already
record that Windows PowerShell users may prefer `npm.cmd`; the new npm script
should hide that detail for the common case.

## Non-Goals

- Do not add CI in this slice.
- Do not add `cargo fmt --check` until formatting policy is explicitly
  accepted and the current tree is known to pass.
- Do not add `cargo clippy` until warning policy is explicit and the current
  tree is known to pass.
- Do not pin `grammers-*` dependencies in this slice.
- Do not add Playwright, browser runtime checks, Telegram live validation, or
  LLM event-flow verification.
- Do not change frontend, backend, database, or product behavior.

## Validation

The implementation is considered complete when:

- `npm run verify` runs the five intended gates in order;
- a failure in any gate stops subsequent gates and returns a non-zero exit;
- the project docs point developers to the command;
- `docs/backlog.md` marks the covered backlog item complete while leaving the
  remaining stabilization items open;
- `npm.cmd run verify` passes on the local Windows environment;
- `git diff --check` passes after documentation and script changes.
