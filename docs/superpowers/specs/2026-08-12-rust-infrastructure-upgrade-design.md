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
the feature-resolved Windows build graph emitted by `cargo tree --manifest-path
src-tauri/Cargo.toml --locked --target x86_64-pc-windows-msvc --workspace
--prefix none --format "{p}"`.
The generator extracts and uniquifies only `name@version`; it does not parse
tree frames or `(*)` markers. The current graph contains 448 package versions,
400 unique names, 32 names resolved at more than one version, 80
package-version instances across those names, and 48 versions in excess of one
per name.

The previous `cargo tree --manifest-path src-tauri/Cargo.toml --locked -d`
observation of 51 names and 151 rendered records is superseded because it
counted build units, including host/target and feature slices, rather than
unique versions. A later Cargo metadata measurement of 37
names and 91 instances is also superseded: `resolve.nodes` included optional
packages not reached by the active feature graph, including duplicate branches
for `ahash`, `const-oid`, `hmac`, `schemars`, and `toml_datetime`. The complete
lockfile count is evidence, not a deduplication target.

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

The diagnostic Clippy inventory, run without `--all-features` and without
fail-fast behavior so one package cannot hide later findings, currently reports
eight violations across three crates:

- four `clippy::large_enum_variant` findings: public `CancelRunOutcome`,
  private `ExecutionSelection`, public sidecar wire type
  `GeminiBrowserSidecarResponse`, and private Telegram `ListedPeer`;
- `clippy::new_without_default` for `LlmSchedulerState`;
- `clippy::new_without_default` for `TelegramRuntime`;
- one ignored `?Sized` bound in the LLM deserialize API probe;
- one `clippy::items_after_test_module` in the Telegram runtime tests.

The feature choice matters: enabling `app-test-support` hides the Telegram
finding. Baselines must therefore be measured with the exact accepted command,
not with `--all-features`.

The existing `npm.cmd run verify` already runs frontend, sidecar, Playwright,
format, workspace Cargo check, and workspace Cargo test gates. Wave 0 removes
the redundant workspace Cargo check: the following
`cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
--locked` builds the same targets, while the fast Clippy gate checks that
surface more strictly. `verify` requires a
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

Discover the baseline in three stages. First, classify the eight initial
findings before suppressing or changing code. After those are resolved,
preserve the same `--keep-going --message-format=short` discovery command and
its second inventory: 15 post-frontier target diagnostics at 14 unique source
locations in `extractum-analysis` and `extractum-prompt-packs`
(`snapshots.rs:335` is reported for lib and lib-test but has one source fix).
After those are resolved, capture the root `extractum` frontier as the third
stage. The baseline commit and verification record retain all three commands
and all three inventories.

The post-frontier inventory is: `extractum-analysis/report.rs:355`
(`prepare_analysis_report_execution`, `too_many_arguments`),
`extractum-analysis/state.rs:22` (`AnalysisState::new`,
`new_without_default`), `extractum-analysis/store/read_model.rs:53`
(`resolve_run_scope_label_parts`, `too_many_arguments`),
`extractum-analysis/report/tests/corpus_port.rs:136` (`vec_init_then_push`),
`extractum-analysis/tests.rs:64` (`explicit_auto_deref`),
`extractum-prompt-packs/dto.rs:10` (`PromptPackRuntimeProvider`,
`derivable_impls`), `extractum-prompt-packs/youtube_summary/gem_analysis.rs:644`
(`needless_return`),
`extractum-prompt-packs/youtube_summary/result_validation.rs:622`
(`needless_lifetimes`),
`extractum-prompt-packs/youtube_summary/snapshots.rs:292`
(`needless_borrow`),
`extractum-prompt-packs/youtube_summary/snapshots.rs:335`
(`insert_material`, `too_many_arguments`),
`extractum-prompt-packs/youtube_summary/synthesis_input.rs:87`
(`redundant_closure`),
`extractum-prompt-packs/result_builder.rs:1114`
(`insert_intermediate_entities_artifact`, `too_many_arguments`),
`extractum-prompt-packs/runtime.rs:866` (`needless_maybe_sized`), and
`extractum-prompt-packs/lib.rs:152` (`assertions_on_constants`).

For `large_enum_variant`, boxing is allowed only when it preserves serialization
and behavior. `GeminiBrowserSidecarResponse::RunResult` is a public
`Serialize`/`Deserialize` sidecar protocol type; changing the Rust field shape
must be accompanied by golden serialization/deserialization tests proving its
JSON wire representation is unchanged and by focused tests of the immediate
cross-crate consumer. Private `ExecutionSelection` changes require focused
behavior tests but no public API claim. Public `CancelRunOutcome` receives a
focused owner test for its queued-cancellation payload. Private `ListedPeer`
is owned by the Telegram package checkpoint.

For `new_without_default`, adding `Default` is not automatic.
`LlmSchedulerState` and `TelegramRuntime` expose public cross-crate APIs, and a
default runtime handle may be semantically invalid. Each finding is resolved
either by a meaningful `Default` implementation or by a narrowly scoped
`#[allow(clippy::new_without_default)]` with a comment explaining why no valid
default exists. The Clippy gate is sufficient evidence for a behavior-neutral
`Default` delegating to `new()`; synthetic tests that merely assert the trait
exists are not required. Global allows and unrelated refactors are forbidden.

The ignored LLM `?Sized` bound is removed from both paired deserialize-probe
impls because `DeserializeOwned` already constrains the tested type to `Sized`.
The Telegram test-only impl blocks move
before `mod tests` so the test module remains the final item. Neither cleanup
changes production behavior.

The post-frontier baseline resolves ten non-argument-count diagnostics with
behavior-neutral mechanical code changes. `AnalysisState::default()` delegates
to `new()` without a synthetic trait-only test. `PromptPackRuntimeProvider`
derives `Default` with existing `Api` marked `#[default]`; a focused test proves
the default remains `Api` and preserves serialization. The tautological
`assert!(cfg!(test));` is removed from
`tests::cancellation_smoke_services_remain_test_only`, while that focused test
retains the feature-off/public-surface probe.

The four established functions with `too_many_arguments` receive only a
narrowly scoped owning-function `#[allow(clippy::too_many_arguments)]` and a
nearby ownership comment: public `prepare_analysis_report_execution`, private
`resolve_run_scope_label_parts`, private `insert_material`, and test-only
`insert_intermediate_entities_artifact`. Preserving their established call
shapes is preferable in Wave 0; no parameter structs, public API changes, or
module/crate/global allows are permitted. The remaining mechanical locations
are `report/tests/corpus_port.rs:136` (`vec_init_then_push`), `tests.rs:64`
(`explicit_auto_deref`), `gem_analysis.rs:644` (`needless_return`),
`result_validation.rs:622` (`needless_lifetimes`), `snapshots.rs:292`
(`needless_borrow`), `synthesis_input.rs:87` (`redundant_closure`), and
`runtime.rs:866` (`needless_maybe_sized`).

The baseline discovery has a third stage after those package findings are
resolved. A package-scoped diagnostic run for root `extractum` reports exactly
62 lib-test target diagnostics at 60 unique file-and-line locations in 35 files. The
library target emits 50 occurrences; lib-test repeats those 50 and adds 12
test-only findings, for 112 occurrences across both compilations. Three
generated IPC diagnostics share the one macro-definition location
`src/lib.rs:402`; rustc's short output may coalesce the two 11/7 expansions,
but the diagnostic total remains 62. The accepted
lib-test totals are 22 `explicit_auto_deref`, 17 `too_many_arguments`, three
`type_complexity`, three `unnecessary_literal_unwrap`, two each of
`bool_assert_comparison`, `redundant_pattern_matching`, and
`unwrap_or_default`, plus one each of `duplicate_mod`, `nonminimal_bool`,
`manual_repeat_n`, `needless_question_mark`, `needless_borrow`,
`unnecessary_unwrap`, `implied_bounds_in_impls`, `match_result_ok`,
`field_reassign_with_default`, `get_first`, and `await_holding_lock`.

The authorized application scope is exactly these 35 files:
`prompt_packs/source_adapter.rs`, `apalis_jobs.rs`, `ingest_provenance.rs`,
`job_helpers.rs`, `projects/read_model.rs`, `projects/mod.rs`,
`topic_memberships.rs`, `prompt_packs/runtime_commands.rs`, `accounts.rs`,
`takeout_import/recovery.rs`, `sources/items/query.rs`, `sources/items.rs`,
`youtube/captions.rs`, `youtube/process_runtime.rs`,
`youtube/source_metadata.rs`, `notebooklm_export/query.rs`, `llm/profiles.rs`,
`llm/mod.rs`, `gemini_browser/jobs.rs`, `gemini_browser/sidecar.rs`,
`analysis/fixtures/seed/runs.rs`, `analysis/fixtures/seed.rs`,
`analysis/fixtures.rs`, `analysis/store/read_model.rs`,
`analysis/store/setup.rs`, `analysis/mod.rs`, `lib.rs`, `external_process.rs`,
`diagnostics/database.rs`, `library_sources/mod.rs`, `migrations.rs`,
`telegram.rs`, `takeout_import/mod.rs`, `sources/store.rs`, and
`analysis/tests_application.rs`, all below `src-tauri/src/`.

Of the 62 third-stage target diagnostics, 43 are resolved only by
behavior-preserving mechanical cleanup. The 17 `too_many_arguments`
diagnostics at 15 recorded locations receive only owning-function or affected
application-command-inventory-entry allows with nearby ownership comments.
The three macro-generated IPC diagnostics are annotated at their affected
inventory entries: `start_project_analysis` (11 arguments),
`list_analysis_runs` (11 arguments), and `start_analysis_report` (12
arguments), never at the macro, module, or crate. Parameter objects,
public Rust or Tauri/IPC signature changes, and module/crate/global lint allows
are forbidden.

The remaining two `redundant_pattern_matching` diagnostics in the managed
yt-dlp process lifecycle require an explicit temporary/drop-order audit.
`is_err()` is used only if existing lifecycle tests prove equivalent ordering;
otherwise the established pattern remains under the smallest function-local
allow with an ordering comment. Focused non-empty RED/GREEN evidence covers
the job-filter predicate, configured-key/base-URL option selection, process
termination/reaping order, and release of the takeout event-recorder lock
before the next await. The application package then runs its all-targets check
and test checkpoint before the fail-fast workspace Clippy gate and full
verifier.

The Wave 0 verification record retains all three discovery commands and exact
inventories. For the third stage it additionally records the 60-location
table, library/lib-test duplication relationship, mechanical versus local-
allow disposition, ownership comments, drop-order audit decision, focused
test results, and the exact staged file set.

The normal Clippy gate intentionally does not use `--all-features`, because
`csp-verification`, `prompt-pack-dev-fixtures`, and `app-test-support` alter the
surface under test. The separate `cargo check --manifest-path
src-tauri/Cargo.toml -p <producer> --lib --no-default-features --locked`
commands prove that selected producer libraries compile feature-off; they do
not lint those feature-off configurations. This is an explicit coverage
limitation, not an accidental claim.

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
- 32 unique duplicated package names;
- 80 package-version instances across duplicated names;
- a sorted mapping from each duplicated name to its version count, such as
  `getrandom: 4`, `hashbrown: 4`, and `windows-sys: 5`.

The generator reads the compact normalized `cargo tree` output, extracts and
uniquifies package identities from the `{p}` records, groups by name, and
applies stable ordering. It records cardinality per duplicated name but omits
the version values themselves: compatible version replacement creates no
artifact noise, while a newly duplicated name or third/fourth version is
self-localizing. A repository rule fails if either duplicate count grows or a
per-name cardinality grows. A decrease is accepted and should lower the
committed baseline in the same dependency wave. A changed package set at the
same aggregate count is reported for review but does not fail solely because
one upstream duplicate replaced another.

A baseline increase has an explicit reviewed path. The dependency wave updates
the committed counters and adds a supply-chain artifact entry for every newly
duplicated name or additional version, identifying the upstream owner/cause,
why avoiding the duplicate is not currently practical, and a review date. The
same wave must include its release/focused evidence where the owning dependency
policy requires it. This policy makes growth exceptional and attributable
without claiming that Extractum can collapse mutually incompatible upstream
major versions.

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
- exact pins, prerelease dependencies, and their approved ownership policy.

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
`cargo update --manifest-path src-tauri/Cargo.toml -p <package> --precise
<version>`. For a SemVer-major change, modify the manifest requirement first,
then run the precise update. A cheap changed-package-count check may be added
later if review evidence proves insufficient; it is not Wave 0 scope.

#### Windows CI and Release Executor

Wave 0 creates Windows GitHub Actions automation with three execution paths.

One repository-owned PowerShell script downloads the pinned cargo-deny Windows
binary, verifies the recorded checksum, extracts it, and exposes it on `PATH`.
Both local instructions and the composite CI action call that script; neither
duplicates the bootstrap implementation. The fast workflow runs on
`pull_request` for branch work and on `push` only for `main`, avoiding duplicate
Windows runs for commits already covered by a PR. It otherwise requires no npm
installation and runs:

1. `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` under the
   pinned root toolchain;
2. `cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets
   --locked -- -D warnings`;
3. the canonical `cargo check --manifest-path src-tauri/Cargo.toml -p
   <producer> --lib --no-default-features --locked` checks for the two
   producers;
4. `cargo deny --config deny.toml --manifest-path src-tauri/Cargo.toml check
   bans licenses sources`.

Repository-rule tests remain in the full PR job through `npm.cmd run verify`;
they are routed through the `unit-node` Vitest project and are not duplicated
in the fast job.

The full PR job installs dependencies with `npm.cmd ci`, restores the canonical
`src-tauri/target` cache, invokes the shared cargo-deny setup action, runs
`npm.cmd run bootstrap:testing`, installs Playwright Chromium, runs the fast
Rust gate, and then runs `npm.cmd run verify`. `bootstrap:testing` owns
`svelte-kit sync` before sidecar compilation, so this documented `npm ci` then
`bootstrap:testing` order works in a fresh checkout. `scripts/verify.mjs` removes its
redundant workspace Cargo check and adds `--locked` to the workspace Cargo test
step. The job does not repeat that expensive workspace command outside
`verify`.

The release/bundle path is an explicit workflow job available through
`workflow_dispatch` and release tags. It runs the full PR gate, current
advisories, `npm.cmd run build:tauri-prereqs`, and a Windows Tauri release build,
then uploads MSI and NSIS with separate `actions/upload-artifact@v4` steps whose
`if-no-files-found` policy is `error`, and performs the documented
application/sidecar smoke. The successful build and executable startup smoke
prove the application executable exists; no separate artifact-discovery script
is added. A dependency wave touching Tauri, `windows-sys`,
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
that Cargo necessarily moves with it. The known `serde` closure includes
`serde_core` and must appear in that group's lockfile summary. The Tauri group
updates the Rust crates,
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
manual `cargo update --manifest-path src-tauri/Cargo.toml -p <package>
--precise <version>` operations suggested by the initial snapshot.

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
run the Edition 2024 migration lints and `cargo fix --manifest-path
src-tauri/Cargo.toml --workspace --all-targets --edition` while manifests still
declare Edition 2021. Review generated source changes, run focused tests, then
switch `[workspace.package].edition` to `"2024"` and make every package inherit
it consistently.

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
cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> --locked -- --exact
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
