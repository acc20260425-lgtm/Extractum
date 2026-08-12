# Rust Infrastructure Upgrade Design

**Date:** 2026-08-12

**Status:** Approved for implementation planning

## Goal

Upgrade Extractum's complete Rust development, dependency, verification, CI,
and release infrastructure to a reproducible Rust 1.95 / Edition 2024 baseline
without combining toolchain, edition, Tauri, and broad lockfile changes into an
undiagnosable migration.

The work uses a balanced risk profile: current stable tooling is pinned,
Edition 2024 is adopted, SemVer-compatible dependencies are updated in bounded
groups, incompatible dependency changes remain separate, and Windows is the
only required build and release platform. Linux and macOS release support are
outside this design.

## Current State and Measured Baseline

The Rust workspace lives at `src-tauri/Cargo.toml` and contains the application
package plus six extracted crates:

- `extractum`;
- `extractum-core`;
- `extractum-gemini-browser`;
- `extractum-llm`;
- `extractum-prompt-packs`;
- `extractum-analysis`;
- `extractum-telegram`.

All seven packages use Edition 2021 and none declares `rust-version`. There is
no `rust-toolchain.toml`, no `deny.toml`, and no GitHub Actions workflow. The
local baseline is `rustc 1.95.0 (59807616e 2026-04-14)` on
`stable-x86_64-pc-windows-msvc`; `rustfmt` and Clippy are installed. Pinning
1.95.0 therefore freezes the observed compiler rather than changing it.

`src-tauri/Cargo.lock` is synchronized with the manifests and contains 736
cross-platform packages and no Git sources. The actionable duplicate metric is
target-specific: `cargo tree -d --target x86_64-pc-windows-msvc` currently
contains 51 unique duplicated package names and 151 duplicate-tree records.
The complete lockfile count is evidence, not a deduplication target.

A Cargo dry run against the current manifests found 185 SemVer-compatible
lockfile changes, including Tauri `2.10.3` to `2.11.5`. This volume requires
bounded dependency groups rather than one unconstrained `cargo update`.

The accepted Clippy command, without `--all-features`, currently reports five
violations across three crates:

- three `clippy::large_enum_variant` findings across
  `extractum-gemini-browser` and `extractum-llm`;
- `clippy::new_without_default` for `LlmSchedulerState`;
- `clippy::new_without_default` for `TelegramRuntime`.

The feature choice matters: enabling `app-test-support` hides the Telegram
finding. Baselines must therefore be measured with the exact accepted command,
not with `--all-features`.

The existing `npm.cmd run verify` already runs frontend, sidecar, Playwright,
format, workspace Cargo check, and workspace Cargo test gates. It requires a
prebuilt Gemini Browser sidecar binary. Both Playwright configurations use
Chromium against a Vite development server and require neither a Tauri bundle
nor live credentials, so they can run on a Windows GitHub-hosted runner after
bootstrap and browser installation.

## Non-Goals

This program does not:

- add Linux or macOS as supported release targets;
- upgrade exact Grammers `=0.10.0` pins automatically;
- move Apalis away from `=1.0.0-rc.8` automatically;
- combine `tauri-plugin-mcp-bridge` `0.11` to `0.12` with compatible updates;
- eliminate every transitive duplicate owned by Tauri, Wry, cryptography, or
  other upstream graphs;
- prove feature-off configurations with an all-features Clippy run;
- change product behavior, persisted data, sidecar JSON wire formats, or
  frontend contracts as an incidental consequence of infrastructure work.

## Upgrade Program

### Wave 0: Reproducible Infrastructure Baseline

Wave 0 precedes every compiler, edition, and dependency upgrade. It is an
infrastructure bootstrap composed of reviewable commits, not an atomic
dependency-update wave.

#### Toolchain and MSRV

Create `rust-toolchain.toml` at the repository root. Rustup discovers overrides
from the current directory upward and does not follow Cargo's `--manifest-path`,
so placing the file under `src-tauri/` would not affect root-issued commands.
The canonical file is:

```toml
[toolchain]
channel = "1.95.0"
components = ["rustfmt", "clippy"]
targets = ["x86_64-pc-windows-msvc"]
profile = "minimal"
```

Add `rust-version = "1.95"` to `[workspace.package]`. The root application
package and every one of the six extracted crates explicitly inherit it with
`rust-version.workspace = true`. Edition remains 2021 until its own wave.

A repository rule checks the exact correspondence among the root toolchain
channel, workspace MSRV, all seven package inheritance declarations, and the
Windows target. It also verifies that every Cargo command in the canonical
verification scripts uses `src-tauri/Cargo.toml` and that check/test commands
which consume the committed resolution use `--locked`.

A future toolchain bump is its own migration wave. Within that wave, the commit
that changes `rust-toolchain.toml` changes no other file; a CI changed-commit
rule inspects each commit rather than only the aggregate PR diff. Any Clippy
baseline adjustment, MSRV update, or source fix is a subsequent explicit
commit in the same wave, with fresh lint evidence. This preserves attribution
without pretending a compiler update is behavior-neutral.

#### Clippy Baseline

Classify all five existing findings before suppressing or changing code. The
baseline commit records the accepted command and the classification of each
finding.

For `large_enum_variant`, boxing is allowed only when it preserves serialization
and behavior. `GeminiBrowserSidecarResponse::RunResult` is a public
`Serialize`/`Deserialize` sidecar protocol type; changing the Rust field shape
must be accompanied by golden serialization/deserialization tests proving its
JSON wire representation is unchanged and by focused tests of the immediate
cross-crate consumer. Private `ExecutionSelection` changes require focused
behavior tests but no public API claim.

For `new_without_default`, adding `Default` is not automatic.
`LlmSchedulerState` and `TelegramRuntime` expose public cross-crate APIs, and a
default runtime handle may be semantically invalid. Each finding is resolved
either by a meaningful, tested `Default` implementation or by a narrowly scoped
`#[allow(clippy::new_without_default)]` with a comment explaining why no valid
default exists. Global allows and unrelated refactors are forbidden.

The normal Clippy gate intentionally does not use `--all-features`, because
`csp-verification`, `prompt-pack-dev-fixtures`, and `app-test-support` alter the
surface under test. The separate `cargo check --no-default-features` commands
prove that selected producer libraries compile feature-off; they do not lint
those feature-off configurations. This is an explicit coverage limitation, not
an accidental claim.

#### Supply-Chain Tooling

Use `cargo-deny` 0.20.2 as the sole dependency-policy tool. Do not add
`cargo-audit`: both consume the RustSec advisory database, while cargo-deny also
owns bans, licenses, and sources. Record the cargo-deny version and Windows
release-asset checksum in a repository-owned tool manifest. CI downloads the
prebuilt pinned binary and validates its checksum; it does not compile the tool
with `cargo install` on every run.

Add root `deny.toml` with the Windows-only dependency graph:

```toml
[graph]
targets = ["x86_64-pc-windows-msvc"]
```

The deterministic push and PR policy runs only:

```powershell
cargo deny --manifest-path src-tauri/Cargo.toml check bans licenses sources
```

Advisories remain a moving external data source and run separately on a
schedule and before a release:

```powershell
cargo deny --manifest-path src-tauri/Cargo.toml check advisories
```

A newly published advisory creates a security task and does not retroactively
block an unrelated PR. A release cannot proceed while the advisory job is red
unless a reviewed, time-bounded advisory exception is committed with owner,
reason, and review date.

#### License Policy

Wave 0 includes a distinct review of every resolved Windows-target license; an
empty cargo-deny allowlist is invalid because it rejects the graph. The initial
global allowlist is limited to license identifiers required by packages that
cannot be satisfied through an already allowed branch of an `OR` expression:

- `Apache-2.0`;
- `Apache-2.0 WITH LLVM-exception`;
- `BSD-3-Clause`;
- `CDLA-Permissive-2.0`;
- `ISC`;
- `MIT`;
- `MIT-0`;
- `MPL-2.0`;
- `Unicode-3.0`;
- `Zlib`.

`[licenses.private] ignore = true` excludes the seven unpublished workspace
packages, which currently have no license expression. The review explicitly
checks `ring`, `unicode-ident`, and `rustls-webpki`: their currently declared
expressions are respectively `Apache-2.0 AND ISC`,
`(MIT OR Apache-2.0) AND Unicode-3.0`, and `ISC`. If cargo-deny 0.20.2 cannot
validate the packaged license texts from those expressions, add version-bound
`[[licenses.clarify]]` entries with verified license-file hashes. A
clarification is accepted only with a verification record identifying the
inspected crate archive and hash; it is not used to broaden the global
allowlist.

License exceptions are crate- and version-bound. Each carries a reason, owner,
and review date in the supply-chain baseline artifact. `unused-allowed-license`
and unused exceptions warn during bootstrap and become errors after the initial
policy is green.

#### Duplicate Policy

Set `bans.multiple-versions = "warn"`; do not hand-maintain roughly one hundred
versioned `bans.skip` records for 51 transitive duplicate names. A generated
Windows duplicate-baseline JSON records:

- schema version;
- target `x86_64-pc-windows-msvc`;
- 51 unique duplicated package names;
- 151 duplicate-tree records;
- the generated package names and resolved versions used as audit evidence.

The generator uses locked Cargo metadata/tree data and stable ordering. A
repository rule fails if either count grows. A decrease is accepted and should
lower the committed baseline in the same dependency wave. A changed package
set at the same count is reported for review but does not fail solely because
one upstream duplicate replaced another. This policy blocks graph growth
without claiming that Extractum can collapse mutually incompatible upstream
major versions.

Wildcard dependencies, unknown registries, and unknown Git sources are denied.
The canonical crates.io registry is allowed. Exact pins, prerelease versions,
and any source exceptions are represented in a generated dependency-policy
artifact so policy drift is executable rather than prose-only.

#### Executable Repository Rules

Extend the existing repository-index and repository-rule framework used by the
Grammers feature baseline. Wave 0 adds rules for:

- root toolchain, workspace MSRV, and inheritance by all seven packages;
- locked manifest/lockfile consistency through `cargo metadata --locked`;
- matching Tauri Rust and npm package families;
- exact pins, prerelease dependencies, and their approved ownership policy;
- canonical cargo-deny version and checksum;
- the Windows duplicate baseline and license-policy metadata.

Two policies require CI-aware changed-state checks rather than a Vitest
repository rule:

- every commit that changes `rust-toolchain.toml` changes no other file;
- a dependency-wave PR declares the packages it owns, and its manifest plus
  `Cargo.lock` diff may resolve only those packages and unavoidable transitive
  descendants. The check compares the resolved before/after graphs against the
  declaration; it does not attempt to infer which Cargo command was typed.

This replaces the unenforceable prose rule "never run bare cargo update" with
an observable lockfile-scope contract. The documented operator command remains
`cargo update -p <package> --precise <version>`. For a SemVer-major change,
modify the manifest requirement first, then run the precise update.

#### Windows CI and Release Executor

Wave 0 creates Windows GitHub Actions automation with three execution paths.

The fast push job runs:

1. toolchain and locked-metadata policy checks;
2. `cargo fmt --check`;
3. workspace Clippy with `--locked` and `-D warnings`;
4. the two producer `--no-default-features --locked` checks;
5. deterministic cargo-deny checks for bans, licenses, and sources.

The full PR job installs dependencies with `npm.cmd ci`, restores the canonical
`src-tauri/target` cache, downloads and verifies the pinned cargo-deny binary,
runs `npm.cmd run bootstrap:testing`, installs Playwright Chromium, runs the
fast Rust gate, and then runs `npm.cmd run verify`. `scripts/verify.mjs` adds
`--locked` to its workspace Cargo check and test steps. The job does not repeat
those two expensive workspace commands outside `verify`.

The release/bundle path is an explicit workflow job available through
`workflow_dispatch` and release tags. It runs the full PR gate, current
advisories, `npm.cmd run build:tauri-prereqs`, and a Windows Tauri release build,
then verifies the expected NSIS/MSI artifacts and performs the documented
application/sidecar smoke. A dependency wave touching Tauri, `windows-sys`,
Keyring, or SQLx must manually dispatch this job and link its run in that
wave's verification record before acceptance. Release tags invoke it
automatically. This gives the release rule an executor rather than relying on
operator memory.

A scheduled workflow runs `cargo deny ... check advisories` against the current
default branch. Failure opens or updates the security follow-up process; it
does not rewrite dependency policy automatically.

#### Wave 0 Verification and Rollback

Wave 0 ends with a verification record containing exact tool versions,
tool-download checksums, commands, CI run links, license decisions, duplicate
metrics, Clippy classifications, and observed results. It also updates
`AGENTS.md`, `docs/project.md`, and developer setup instructions because the
canonical workflow changes.

Wave 0 is intentionally multi-commit and rolls back as a reviewed commit range.
Each commit remains independently attributable: toolchain/MSRV, Clippy
baseline, supply-chain policy, repository rules, CI, documentation, and final
evidence are not collapsed into one change.

### Wave 1: Edition 2024

Run the edition migration separately from all dependency updates. First run
the Edition 2024 migration lints and `cargo fix --edition` while manifests
still declare Edition 2021. Review generated source changes, run focused tests,
then switch `[workspace.package].edition` to `"2024"` and make every package
inherit it consistently.

Edition 2024 requires Rust 1.85 or newer, so the pinned 1.95/MSRV baseline is
compatible. The wave owns only edition-related manifests, source migrations,
tests, repository rules, documentation, and the lockfile only if Cargo proves
an edition-related change unavoidable. It does not update Tauri or other
dependencies.

### Wave 2: Compatible Lockfile Refresh

Refresh SemVer-compatible dependencies in declared groups, using precise
package updates and a lockfile-scope declaration for each group. Suggested
groups are:

1. build/proc-macro and platform-neutral leaf dependencies;
2. async/network runtime dependencies;
3. data, serialization, compression, and cryptography dependencies;
4. Tauri core/runtime/plugin family;
5. remaining compatible transitive changes.

Group boundaries may be tightened after Cargo resolution. They may not be
collapsed into one 185-package update merely because every version satisfies
its existing manifest requirement.

The Tauri group updates the Rust crates, `tauri-build`, npm CLI/API, and
same-name frontend plugins as one compatibility family. Official Tauri guidance
expects `tauri`, `tauri-build`, and the CLI to remain on compatible current
minor releases. The group closes with the release/bundle job.

### Wave 3: Runtime, Data, and Security Dependencies

Treat Tokio, SQLx, Reqwest/rustls, Keyring, cryptography, and `windows-sys` as
risk-owned groups even when Cargo considers their updates compatible. Each
group receives focused owning-package tests and immediate-dependent checks.
SQLx, Keyring, cryptography, and Windows integration changes include release
build evidence; database changes preserve the single backend-owned SQLite path
and additive migration policy.

Exact Grammers pins and Apalis RC pins remain unchanged unless separately
approved. A prerelease move requires upstream release/API review, focused
runtime evidence, and its own design or explicitly scoped amendment.

### Wave 4: Incompatible Dependency Upgrades

Handle known incompatible candidates as independent slices after compatible
updates are stable. Initial candidates include `jsonschema 0.46` to `0.49` and
`tauri-plugin-mcp-bridge 0.11` to `0.12`.

Before changing `tauri-plugin-mcp-bridge`, decide its production ownership.
Although plugin registration is under `#[cfg(dev)]`, the current unconditional
normal dependency compiles into the production graph, lockfile, advisories,
and duplicate metrics. The slice must either document that production cost as
intentional or move the dependency behind a feature/dev-only boundary and
prove release behavior. The version upgrade cannot silently make that
architectural decision.

### Wave 5: Maintenance Automation

After the upgrade waves are green, establish an ongoing cadence:

- scheduled advisory monitoring;
- scheduled dependency inventory reporting without automatic manifest writes;
- periodic toolchain-bump waves with isolated pin commits and fresh Clippy
  classification;
- periodic downward refresh of duplicate and license baselines;
- release/bundle verification on tags and manually for high-risk waves.

Automation may propose updates but does not merge dependency, toolchain, or
policy changes without the same gates used by manual waves.

## Canonical Acceptance Contract

The fast deterministic Rust contract, issued from the repository root, is:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features --locked
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib --no-default-features --locked
cargo deny --manifest-path src-tauri/Cargo.toml check bans licenses sources
```

The full ordinary-wave contract then runs:

```powershell
npm.cmd run verify
```

`verify` owns the workspace Cargo check/test invocations and both use
`--locked`; callers do not duplicate them. A high-risk or release wave also
runs the release/bundle job. Release acceptance additionally requires:

```powershell
cargo deny --manifest-path src-tauri/Cargo.toml check advisories
```

For changes to a public cross-crate interface, the owning package checkpoint
and immediate dependent checkpoint are mandatory. Producers with test-support
features retain separate production-surface checks with
`--lib --no-default-features`; consumer tests prove the feature-on seam.

## Rust Verification Loops

Every implementation plan derived from this design names affected packages and
uses the project's focused Rust loop.

For a source change, begin with a non-empty exact RED/GREEN test in the owning
package:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> -- --exact
```

Then run the focused package check and checkpoint:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets --locked
cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets --locked
```

For `extractum-telegram` and `extractum-prompt-packs`, feature-off production
evidence and feature-on consumer evidence remain distinct. At the end of every
Rust slice, run `npm.cmd run verify`; a filtered Cargo run reporting zero tests
is never completion evidence.

## Documentation and Evidence

Implementation updates:

- `AGENTS.md` for canonical local and CI loops;
- `docs/project.md` for toolchain, dependency, supply-chain, and release policy;
- `README.md` or the active developer setup page for bootstrap prerequisites;
- `docs/value-registry.md` only if implementation introduces or changes a
  registry-owned string value;
- a verification record per completed wave under
  `docs/superpowers/verification/`.

Generated policy artifacts use explicit schema versions, canonical ordering,
and repository-rule tests. Historical plans and verification records are not
rewritten to reflect the new policy.

## Success Criteria

The program is complete when:

- every developer and CI run uses the root-pinned Rust 1.95.0 toolchain and
  declared MSRV 1.95;
- all seven workspace packages build and test on Edition 2024;
- accepted Clippy, production-surface, locked-resolution, supply-chain, and
  full verification gates are green;
- license decisions and the Windows duplicate baseline are machine-enforced;
- dependency waves are bounded, attributable, and reversible;
- Tauri and high-risk platform/data updates have Windows release evidence;
- scheduled advisories and release tags have explicit CI executors;
- exact/prerelease and incompatible upgrades remain isolated unless separately
  approved;
- documentation describes the same executable workflow enforced by CI.
