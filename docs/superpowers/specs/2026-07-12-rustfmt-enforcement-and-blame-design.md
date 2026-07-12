# Rustfmt Enforcement and Blame Hygiene Design

## Goal

Protect the clean repository-wide Rust formatting baseline from new drift and
keep the mechanical formatting commit from obscuring useful `git blame`
history.

## Current State

Commit `acbe5bfd2105f4930063f1c8a204e57a9f47c86f` formatted the complete Rust
workspace and established a baseline that currently passes:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

The repository has no CI workflow or shared Git hook. `package.json` exposes
frontend and aggregate verification commands. The aggregate `verify` pipeline
already runs frontend tests and checks followed by `cargo check`, `cargo test`,
and a Git whitespace check, but it does not run rustfmt. `AGENTS.md` requires
`cargo check` after Rust changes but does not require a formatting check.
Consequently, the baseline is clean but is not protected by either the normal
Rust validation convention or a full aggregate verification run.

The mechanical style commit also becomes the apparent author of many lines in
plain `git blame`, even though it changed their layout rather than their
meaning.

## Selected Design

Add this package script:

```json
"check:rustfmt": "cargo fmt --manifest-path src-tauri/Cargo.toml -- --check"
```

This provides one short, cross-platform developer entrypoint. On Windows the
documented invocation is:

```powershell
npm.cmd run check:rustfmt
```

The command is check-only: it must not rewrite source files. Developers may
run `cargo fmt --manifest-path src-tauri/Cargo.toml` separately when a repair
is needed.

Update the Rust validation rule in `AGENTS.md` so that Rust or Tauri backend
changes require both `npm.cmd run check:rustfmt` and `cargo check` when no
Superpowers workflow supplies its own validation steps. This is an explicit
project verification convention, not an automatic commit or server-side gate.

Add `npm run check:rustfmt` to `scripts/verify.mjs` as a normal aggregate
verification step before `cargo check`. The aggregate must call the package
script through its existing `npmStep` helper rather than duplicate the Cargo
arguments. This makes every full `npm.cmd run verify` exercise the same public
formatting command documented for focused Rust validation.

Add `.git-blame-ignore-revs` containing the full formatting commit hash and a
short comment explaining why the revision is ignored. Document this optional,
per-clone setup command in `AGENTS.md`:

```powershell
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

The configuration command is documentation only. The implementation must not
execute it automatically or modify a developer's local Git configuration.

## Scope

The implementation owns only:

- `package.json`;
- `scripts/verify.mjs`;
- `AGENTS.md`;
- `.git-blame-ignore-revs`.

It does not modify Rust source, Git hooks, CI workflows, formatter
configuration, application behavior, or runtime dependencies. The only change
to the aggregate `npm.cmd run verify` pipeline is inserting the formatting
check before `cargo check`. It does not update `docs/value-registry.md` because
no runtime or persisted string value changes.

## Rejected Alternatives

- A repository Git hook would run automatically for developers who install
  it, but hooks are not installed when a repository is cloned and can create
  a misleading impression of universal enforcement.
- A new GitHub Actions workflow would provide server-side enforcement, but the
  repository currently has no CI workflow. Introducing CI, runner policy, and
  dependency-cache behavior is a separate infrastructure decision.
- Adding Rust formatting to the existing frontend `check` command would blur
  that command's established Svelte/TypeScript purpose.
- Relying only on the package script and `AGENTS.md` convention would leave the
  repository's existing aggregate verification pipeline able to pass without
  checking the Rust formatting baseline.
- Automatically setting `blame.ignoreRevsFile` would mutate local developer
  configuration without explicit consent.

## Failure Behavior

`npm.cmd run check:rustfmt` exits nonzero and prints rustfmt's diff whenever
Rust source discovered through the Cargo workspace does not match the active
formatter. It does not repair the files.

If a Git client has not enabled `blame.ignoreRevsFile`, blame continues to
work normally and may show the style commit. If it has enabled the setting,
Git attempts to attribute mechanically changed lines to earlier meaningful
commits. Neither state changes repository history.

## Verification

- Confirm the worktree is clean before implementation.
- Mechanically verify that the full 40-character hash in
  `.git-blame-ignore-revs` resolves to the `style: format rust sources` commit.
- Run `npm.cmd run check:rustfmt`; require exit 0 and no formatting diff.
- Prove the command's negative behavior with an intentionally malformed copy
  of one workspace Rust line: preserve the file's exact original bytes, make a
  controlled formatting-only edit, require `npm.cmd run check:rustfmt` to exit
  nonzero and leave those malformed bytes unchanged, then restore the original
  bytes in an unconditional cleanup step. Confirm the worktree returns exactly
  to its pre-probe state. Do not use this probe on a pre-existing dirty file.
- Run `npm.cmd run check`; require the unchanged frontend validation command
  to pass, because `package.json` was edited.
- Run `npm.cmd run verify`; require its output to include the new
  `npm run check:rustfmt` step and require the complete aggregate pipeline to
  pass.
- Run
  `git blame --ignore-revs-file .git-blame-ignore-revs -- src-tauri/src/youtube/process_runtime.rs`;
  require exit 0 and compare style-commit attribution counts with and without
  the flag. The ignored count must decrease and must not exceed the larger of
  10 lines or 10% of the unignored count. Git may retain the ignored commit for
  rustfmt-created lines that have no unambiguous parent attribution. This
  differential check verifies the ignore file without changing Git config.
- Run `git diff --check`.
- Inspect the final diff and require exactly the four scoped files.
- Confirm `git config --get blame.ignoreRevsFile` is not changed by the
  implementation; the optional setup remains a user action.

## Acceptance Criteria

- `npm.cmd run check:rustfmt` is a documented, check-only project command.
- `AGENTS.md` requires the Rust formatting check after Rust or Tauri backend
  changes and documents the optional blame configuration command.
- `npm.cmd run verify` runs the public `check:rustfmt` package script before
  `cargo check`.
- `.git-blame-ignore-revs` contains the exact full hash of the isolated style
  commit and works with Git's command-line `--ignore-revs-file` option.
- Applying the ignore file substantially reduces style-commit blame
  attribution under the documented differential threshold; zero residual
  attribution is not required.
- The positive and negative checks prove that `check:rustfmt` detects drift
  without rewriting the malformed file.
- No hook, CI workflow, Rust source, runtime behavior, unrelated aggregate
  verify behavior, or local Git configuration changes.
- All specified validation commands pass.
