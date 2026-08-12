# Rust Infrastructure Upgrade Design

**Date:** 2026-08-12

**Status:** Approved for implementation planning

## Goal

Upgrade Extractum's complete Rust development, dependency, verification, CI,
and release infrastructure to a reproducible Rust 1.95 baseline, then modernize
dependencies and adopt Edition 2024 without combining toolchain, edition,
Tauri, and broad lockfile changes into an undiagnosable migration.

The work uses a balanced risk profile: current stable tooling is pinned,
SemVer-compatible direct dependencies are updated by ownership, compatible
transitive resolution is refreshed separately, incompatible changes and
Edition 2024 remain later waves, and Windows is the only required build and
release platform. Linux and macOS release support are outside this design.

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
cross-platform packages and no Git sources. Duplicate policy is derived from
the versioned JSON returned by `cargo metadata --locked --filter-platform
x86_64-pc-windows-msvc`, not from `cargo tree` rendering. The current filtered
resolve graph contains 503 reachable package nodes, 37 names resolved at more
than one version, 91 package-version instances across those names, and 54
versions in excess of one per name. The previous `cargo tree` observation of
51 names and 151 rendered records is superseded because rendered lines and
`(*)` markers are not stable graph entities. The complete lockfile count is
evidence, not a deduplication target.

A Cargo dry run on 2026-08-12 against the current manifests and then-current
crates.io index found 157 updates, 28 additions,
and 46 removals: 185 packages enter or change version and 231 lockfile
positions change overall. Only 12 directly declared dependencies participate:
`anyhow`, `jsonschema`, `reqwest`, `serde`, `serde_json`, `time`, `tokio-util`,
`tauri`, `tauri-build`, `tauri-plugin-dialog`, `tauri-plugin-opener`, and
`tauri-plugin-mcp-bridge`. The other 145 updates are transitive. Direct changes
therefore receive ownership decisions, while the residual transitive mass is
reviewed as a lockfile refresh rather than 145 artificial precise updates.
These counts and the 12-package inventory are planning evidence, not future
acceptance constants. Every dependency wave re-measures its available direct
updates and residual lockfile delta and records the new snapshot in that
wave's verification record.

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
format, workspace Cargo check, and workspace Cargo test gates. Wave 0 removes
the redundant workspace `cargo check`: the following workspace
`cargo test --all-targets` builds the same targets, while the fast Clippy gate
checks that surface more strictly. `verify` requires a
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
`rust-version.workspace = true`. Add `publish = false` to the root `extractum`
package; the six extracted crates already declare it. This makes all seven
internal packages explicitly unpublished and allows the private-license policy
to treat them consistently. Edition remains 2021 until its own wave.

A repository rule checks the exact correspondence among the root toolchain
channel, workspace MSRV, all seven package inheritance declarations, and the
Windows target. The existing `scripts/verify.test.ts`, which owns
`createVerifySteps`, asserts directly that the remaining Cargo test step uses
`src-tauri/Cargo.toml` and `--locked`; this does not require a new repository
rule.

A future toolchain bump is its own migration wave. Within that wave, the commit
that changes `rust-toolchain.toml` changes no other file. Any Clippy baseline
adjustment, MSRV update, or source fix is a subsequent explicit commit in the
same wave, with fresh lint evidence. Wave 0 documents this constraint in
`AGENTS.md`; commit-range automation is deferred to Wave 5 because it provides
no protection until the next compiler bump. This preserves attribution without
pretending a compiler update is behavior-neutral.

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
prebuilt pinned binary and validates its checksum; this CI step is the
enforcement mechanism, so no repository rule compares the manifest with
itself. CI does not compile the tool with `cargo install` on every run.

Add root `deny.toml` with the Windows-only dependency graph:

```toml
[graph]
targets = ["x86_64-pc-windows-msvc"]
```

Every cargo-deny invocation passes the root config explicitly. This avoids any
dependency on how cargo-deny resolves its default config relative to the
current directory or `--manifest-path`. The deterministic push and PR policy
runs only:

```powershell
cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check bans licenses sources
```

Advisories remain a moving external data source and run separately on a
schedule and before a release:

```powershell
cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check advisories
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

`[licenses.private] ignore = true` excludes the seven explicitly unpublished
workspace packages, which currently have no license expression. The review explicitly
checks `ring`, `unicode-ident`, and `rustls-webpki`: their currently declared
expressions are respectively `Apache-2.0 AND ISC`,
`(MIT OR Apache-2.0) AND Unicode-3.0`, and `ISC`. If cargo-deny 0.20.2 cannot
validate the packaged license texts from those expressions, add version-bound
`[[licenses.clarify]]` entries with verified license-file hashes. The hash lives
canonically in `deny.toml`; the supply-chain artifact links the inspected crate
archive without duplicating the hash. A clarification is not used to broaden
the global allowlist.

License exceptions are crate- and version-bound. Each carries a reason, owner,
and review date in the supply-chain baseline artifact. `unused-allowed-license`
and unused exceptions warn during bootstrap and become errors after the initial
policy is green.

#### Duplicate Policy

Set `bans.multiple-versions = "warn"`; do not hand-maintain a large versioned
`bans.skip` inventory for transitive duplicates. A generated Windows
duplicate-baseline JSON records:

- schema version;
- target `x86_64-pc-windows-msvc`;
- 37 unique duplicated package names;
- 91 package-version instances across duplicated names;
- the sorted 37-name duplicate inventory used as audit evidence.

The generator uses `cargo metadata --locked --filter-platform
x86_64-pc-windows-msvc --format-version 1`, the filtered `resolve.nodes`, and
stable ordering. It deliberately
omits resolved versions, which would add noise to every dependency wave without
participating in the gate. A repository rule fails if either count grows. A
decrease is accepted and should lower the committed baseline in the same
dependency wave. A changed package set at the same count is reported for review
but does not fail solely because one upstream duplicate replaced another. This
policy blocks graph growth without claiming that Extractum can collapse
mutually incompatible upstream major versions.

Wildcard dependencies, unknown registries, and unknown Git sources are denied.
The canonical crates.io registry is allowed. Exact pins, prerelease versions,
and any source exceptions are represented in a generated dependency-policy
artifact so policy drift is executable rather than prose-only.

#### Executable Repository Rules

Extend the existing repository-index and repository-rule framework used by the
Grammers feature baseline only where Cargo and CI do not already enforce the
contract. Wave 0 adds three policy rules:

- root toolchain, workspace MSRV, and inheritance by all seven packages;
- matching Tauri Rust and npm package families;
- exact pins, prerelease dependencies, and their approved ownership policy;

The Windows duplicate artifact has its own small generated-baseline evaluator.
License acceptance is enforced directly by cargo-deny; exception ownership and
review dates live in the supply-chain artifact rather than a second parser.
Manifest/lockfile consistency is already enforced by every locked Cargo gate,
and the cargo-deny checksum is enforced while downloading the CI binary.

Wave 0 does not build a before/after Cargo-graph comparator. `--locked` already
detects manifest/lockfile drift, and lockfile scope remains visible in review.
Every dependency-wave verification record instead includes the generated
lockfile diff summary: number of additions, updates, removals, and the directly
declared packages affected. The documented operator command remains
`cargo update -p <package> --precise <version>`. For a SemVer-major change,
modify the manifest requirement first, then run the precise update. A cheap
changed-package-count check may be added later if review evidence proves
insufficient; it is not Wave 0 scope.

#### Windows CI and Release Executor

Wave 0 creates Windows GitHub Actions automation with three execution paths.

The fast push job requires no npm installation and runs:

1. `cargo fmt --check` under the pinned root toolchain;
2. workspace Clippy with `--locked` and `-D warnings`;
3. the two producer `--no-default-features --locked` checks;
4. deterministic cargo-deny checks for bans, licenses, and sources.

Repository-rule tests remain in the full PR job through `npm.cmd run verify`;
they are routed through the `unit-node` Vitest project and are not duplicated
in the fast job.

The full PR job installs dependencies with `npm.cmd ci`, restores the canonical
`src-tauri/target` cache, downloads and verifies the pinned cargo-deny binary,
runs `npm.cmd run bootstrap:testing`, installs Playwright Chromium, runs the
fast Rust gate, and then runs `npm.cmd run verify`. `scripts/verify.mjs` removes
its redundant workspace Cargo check and adds `--locked` to the workspace Cargo
test step. The job does not repeat that expensive workspace command outside
`verify`.

The release/bundle path is an explicit workflow job available through
`workflow_dispatch` and release tags. It runs the full PR gate, current
advisories, `npm.cmd run build:tauri-prereqs`, and a Windows Tauri release build,
then verifies the expected NSIS/MSI artifacts and performs the documented
application/sidecar smoke. A dependency wave touching Tauri, `windows-sys`,
Keyring, or SQLx must manually dispatch this job and link its run in that
wave's verification record before acceptance. Release tags invoke it
automatically. This gives the release rule an executor rather than relying on
operator memory.

A scheduled workflow runs `cargo deny --config deny.toml --manifest-path
src-tauri/Cargo.toml check advisories` against the current default branch.
Failure opens or updates the security follow-up process; it does not rewrite
dependency policy automatically.

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

### Wave 1: Compatible Direct Dependencies

Make decisions for directly declared dependencies rather than pretending the
measured 145 transitive updates are independently owned. Wave 1 has two
reviewable groups with an initial measured inventory:

1. low-risk leaves: `anyhow`, `serde`, `serde_json`, and `time`;
2. the Tauri family: `tauri`, `tauri-build`, `tauri-plugin-dialog`,
   `tauri-plugin-opener`, and `tauri-plugin-mcp-bridge` within `0.11`.

Each direct package uses a precise update and records the transitive closure
that Cargo necessarily moves with it. The Tauri group updates the Rust crates,
`tauri-build`, npm CLI/API, and same-name frontend plugins as one compatibility
family. Official Tauri guidance expects `tauri`, `tauri-build`, and the CLI to
remain on compatible current minor releases. The group closes with the
release/bundle job. `tauri-plugin-mcp-bridge` may move within `0.11`; its
incompatible `0.12` boundary remains Wave 3 scope.

### Wave 2: Risk-Owned Directs and Residual Transitives

The measured direct scope is `reqwest` and `tokio-util`. Treat them as
risk-owned even when Cargo considers their updates compatible; each receives
focused owning-package tests and immediate-dependent checks. Tokio, SQLx,
Keyring, `windows-sys`, secrecy, rustls, and cryptography remain future
risk-owned policy categories, but the 2026-08-12 snapshot contains no pending
compatible direct update for them and the implementation plan must not invent
empty slices. If a re-measurement finds a new direct update in these categories,
the verification record declares it and the corresponding focused/release gate
applies. Database changes preserve the single backend-owned SQLite path and
additive migration policy.

Exact Grammers pins and Apalis RC pins remain unchanged unless separately
approved. A prerelease move requires upstream release/API review, focused
runtime evidence, and its own design or explicitly scoped amendment.

After all direct compatible decisions in Waves 1 and 2 are green, refresh the
remaining compatible transitive resolution in one explicit `transitive
refresh` final commit of Wave 2. Its verification record carries the re-measured
lockfile diff summary and the ordinary acceptance contract. This commit is
attributable as the residual Cargo resolution, not decomposed into the 145
manual `--precise` operations suggested by the initial snapshot.

### Wave 3: Incompatible Dependency Upgrades

Handle known incompatible candidates as independent slices after compatible
updates are stable. Initial candidates include `jsonschema 0.46` to `0.49` and
`tauri-plugin-mcp-bridge 0.11` to `0.12`. Skip the available compatible
`jsonschema 0.46.x` refresh in Wave 1 so the package is reviewed and tested
only once at its intended incompatible boundary.

Before changing `tauri-plugin-mcp-bridge`, decide its production ownership.
Although plugin registration is under `#[cfg(dev)]`, the current unconditional
normal dependency compiles into the production graph, lockfile, advisories,
and duplicate metrics. The slice must either document that production cost as
intentional or move the dependency behind a feature/dev-only boundary and
prove release behavior. The version upgrade cannot silently make that
architectural decision.

### Wave 4: Edition 2024

Run the edition migration after the infrastructure and dependency waves. First
run the Edition 2024 migration lints and `cargo fix --edition` while manifests
still declare Edition 2021. Review generated source changes, run focused tests,
then switch `[workspace.package].edition` to `"2024"` and make every package
inherit it consistently.

Edition 2024 requires Rust 1.85 or newer, so the pinned 1.95/MSRV baseline is
compatible. The wave owns only edition-related manifests, source migrations,
tests, repository rules, documentation, and the lockfile only if Cargo proves
an edition-related change unavoidable. It does not update Tauri or other
dependencies. Moving it here keeps the mass source migration off the critical
path to a reproducible infrastructure baseline.

### Wave 5: Maintenance Automation

After the upgrade waves are green, establish an ongoing cadence:

- scheduled advisory monitoring;
- scheduled dependency inventory reporting without automatic manifest writes;
- periodic toolchain-bump waves with isolated pin commits and fresh Clippy
  classification;
- commit-range enforcement for isolated toolchain-pin commits if the first
  post-baseline compiler migration shows the documentation rule is insufficient;
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
cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check bans licenses sources
```

The full ordinary-wave contract then runs:

```powershell
npm.cmd run verify
```

`verify` owns the workspace Cargo test invocation and uses `--locked`; callers
do not duplicate it. The preceding fast Clippy job covers all workspace targets
more strictly than the removed Cargo check. A high-risk or release wave also
runs the release/bundle job. Release acceptance additionally requires:

```powershell
cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check advisories
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
- a verification record per completed wave under
  `docs/superpowers/verification/`.

Generated policy artifacts use explicit schema versions, canonical ordering,
and repository-rule tests. Historical plans and verification records are not
rewritten to reflect the new policy.

## Success Criteria

The program is complete when:

- every developer and CI run uses the root-pinned Rust 1.95.0 toolchain and
  declared MSRV 1.95;
- Wave 0 produces a green reproducible baseline without depending on the later
  Edition 2024 migration;
- after Wave 4, all seven workspace packages build and test on Edition 2024;
- accepted Clippy, production-surface, locked-resolution, supply-chain, and
  full verification gates are green;
- license decisions and the Windows duplicate baseline are machine-enforced;
- dependency waves are bounded, attributable, and reversible;
- Tauri and high-risk platform/data updates have Windows release evidence;
- scheduled advisories and release tags have explicit CI executors;
- exact/prerelease and incompatible upgrades remain isolated unless separately
  approved;
- documentation describes the same executable workflow enforced by CI.
