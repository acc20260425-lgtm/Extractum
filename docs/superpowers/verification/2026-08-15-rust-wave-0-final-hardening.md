# Rust Wave 0 Final Hardening Verification

## Scope and Spec Authority

This record verifies the final-hardening implementation at the pre-evidence
HEAD `563783cce8c8495c22fb9469d4b7eb3705160155`. The approved design authority
is `docs/superpowers/specs/2026-08-14-rust-wave-0-final-hardening-design.md` at
commit `719043f63ebe158880e8cec90823c98acaef4e75`; the replacement implementation
plan is `docs/superpowers/plans/2026-08-14-rust-wave-0-final-hardening.md` at
commit `76e7db867fd7e2c5b6e9e0b1adc2c12b291993ca`.

The implementation covers the approved four boundaries: one authenticated
pinned cargo-deny runner, live direct-dependency and strict duplicate policy,
structural least-privilege workflow contracts, and the bounded alias/license/
wire-golden/secret-ownership/documentation corrections. It does not upgrade
Rust, Cargo dependencies, `Cargo.lock`, or existing npm dependencies. It adds
only the approved direct devDependency `yaml`.

All local commands were run from the repository root on Windows
`x86_64-pc-windows-msvc`. No push, pull request, workflow dispatch, bundle,
publication, or release action was performed.

## Environment and Tool Digests

The tested environment reported:

```text
rustc 1.95.0 (59807616e 2026-04-14)
rustc commit: 59807616e1fa2540724bfbac14d7976d7e4a3860
host: x86_64-pc-windows-msvc
LLVM: 22.1.2
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
node v24.13.1
npm 11.12.1
cargo-deny 0.20.2
```

The pinned cargo-deny manifest is schema 1, version `0.20.2`, target
`x86_64-pc-windows-msvc`. Its archive SHA-256 is
`975a22143262fd27476d19ee00c7af67978426e40e1dee94eed6bbade1cf87dc`.
The reviewed executable SHA-256 and the freshly measured authenticated cache
binary SHA-256 are both
`f7292fab58c706638c999e64c4ba82e5128ae628130ba55e3266a768ee431fbf`.

The testing bootstrap produced and checked
`src-tauri/binaries/gemini-browser-sidecar-x86_64-pc-windows-msvc.exe` at
45,200,478 bytes.

## Four Task Commits and Combined Review

The implementation range after plan commit `76e7db86` is:

| Boundary | Commit | Result |
| --- | --- | --- |
| Task 1: unified pinned runner | `3eba5189ddc01219f2ffd413afd79c41eed65a57` — `build: consolidate pinned cargo-deny runner` | One production runner; authenticated transactional cache; absolute locked policy execution. |
| Task 2: dependency and duplicate policy | `42d8447fa8cf08a146d6eb82557f49e1fffda90b` — `test: harden Rust dependency and duplicate policy` | Live Cargo/npm snapshot; strict current equality; optional explicit-base growth check. |
| Task 3: workflow contracts | `42a3e46dfced92b522df17339407627d39958a09` — `ci: isolate advisory writes with structural workflow contracts` | YAML structural tests; scanner/writer split; least privilege and bundle dependency. |
| Adjacent generated fact correction | `a0cb4de7a87ba10eda85dd2303766fd2ada041b2` — `fix: refresh Rust dependency policy for yaml` | Added only the generated `yaml` direct-dependency fact. |
| Task 4: bounded findings | `cc6799ecef141d5c7b3137faa33076fc1dd7f596` — `fix: close bounded Rust Wave 0 findings` | Locked aliases, strict license policy, independent sidecar wire literals, clone-free owned secret transfer, factual docs. |
| Combined-review correction | `563783cce8c8495c22fb9469d4b7eb3705160155` — `fix: close final-hardening review findings` | Closed every combined-review finding. |

The first combined review of `76e7db86..cc6799ec` found no Critical issues,
six Important defects, and two Minor defects. They covered optional npm edge
visibility, exact MCP/Tauri authority, exact workspace membership, strict
duplicate/exception schemas, top-level workflow permissions, transactional
Setup rollback, exact historical CLI behavior, and the generated exact-pin
inventory. Commit `563783cc` corrected all eight findings.

The fresh combined re-review covered the exact range
`76e7db867fd7e2c5b6e9e0b1adc2c12b291993ca..563783cce8c8495c22fb9469d4b7eb3705160155`
and reported **0 Critical, 0 Important, and 0 Minor findings**. Its report is
`.superpowers/sdd/final-hardening-combined-rereview.md`.

## Focused JavaScript, PowerShell, and Rust Evidence

The final focused policy aggregate passed 6 files / 70 tests. It covers the
verifier aliases, live dependency policy, current/base duplicate policy,
repository rules and conventions, and the four structural workflow security
properties. The cargo-deny child-process harness passed 9 behavioral cases,
including real `GITHUB_PATH` failure rollback. Current duplicate equality and
the known-base comparison against
`9bcc047b9ec210c1ea34024f6dc782a1e7caec49` both passed; the latter emitted no
historical-skip notice.

The committed Windows duplicate baseline remains exact: 32 duplicated names,
80 version instances, and the committed per-name cardinality mapping. All
three supply-chain exception arrays are empty.

The exact sidecar wire-shape test passed 1/1 with 77 filtered tests, followed by
the package check and 78/78 package tests. The exact LLM access-boundary test
passed 1/1 with 688 filtered tests, followed by the root package check and
689/689 library tests. The LLM correction moves the private owned
`SecretString`; it adds no clone, exposure, log, serialization, or persistence
path.

The deterministic supply-chain lane completed with `bans ok`, `licenses ok`,
and `sources ok`. Its duplicate-tree output is warning-only because strict
duplicate enforcement is owned by the separate exact baseline command.

## Ordered Local Acceptance

Commands were executed in the exact plan order. Commands 1-15 passed. Command
16 required the documented outside-sandbox path and exposed two unrelated
pre-existing/environmental E2E instabilities before a fresh final full pass;
all attempts are retained below rather than collapsed.

| # | Command | Time (UTC, 2026-08-15) | Exit | Evidence |
| --- | --- | --- | --- | --- |
| 1 | `npm.cmd ci` | 05:54:54.991–05:56:01.734 | 0 | 316 packages installed. |
| 2 | focused six-file `npm.cmd run test:unit -- ...` | 05:56:09.922–05:56:31.997 | 0 | 6 files / 70 tests. |
| 3 | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/cargo-deny.test.ps1` | 05:56:38.390–05:56:55.186 | 0 | 9 behavioral cases. |
| 4 | exact sidecar Rust test | 05:57:05.276–05:57:07.036 | 0 | 1 passed, 77 filtered. |
| 5 | exact LLM Rust test | 05:57:13.080–05:57:15.105 | 0 | 1 passed, 688 filtered. |
| 6 | sidecar `cargo check --all-targets --locked` | 05:57:21.631–05:57:23.567 | 0 | PASS. |
| 7 | sidecar `cargo test --all-targets --locked` | 05:57:29.713–05:57:31.132 | 0 | 78/78. |
| 8 | root `cargo check --all-targets --locked` | 05:57:38.453–05:58:09.210 | 0 | PASS. |
| 9 | root `cargo test --all-targets --locked` | 05:58:17.544–05:59:17.577 | 0 | Library 689/689; binary 0 tests. |
| 10 | `npm.cmd run test:unit` | 05:59:24.862–06:01:18.758 | 0 | 114 files / 873 tests. |
| 11 | `npm.cmd run check` | 06:01:25.746–06:02:01.020 | 0 | 0 errors / 0 warnings. |
| 12 | `npm.cmd run bootstrap:testing` | 06:02:08.893–06:02:29.199 | 0 | Sidecar built and checked at 45,200,478 bytes. |
| 13 | `npm.cmd run check:rust:duplicates` | 06:02:34.680–06:02:36.906 | 0 | Strict current equality passed. |
| 14 | `npm.cmd run check:rust:duplicates -- --base 9bcc047b9ec210c1ea34024f6dc782a1e7caec49` | 06:02:45.441–06:02:48.410 | 0 | Base comparison passed without skip. |
| 15 | `npm.cmd run check:rust:fast` | 06:02:57.483–06:04:24.708 | 0 | rustfmt, Clippy, two feature-off checks, deterministic cargo-deny. |
| 16a | `npm.cmd run verify` (sandbox) | 06:04:33.229–06:10:45.010 | 1 | Unit 873/873, component 281/281, architecture 2/2; OS integration 38/43 with five sandbox-only `termination_unconfirmed` failures. |
| 16b | unchanged outside-sandbox fallback | 06:11:02.743–06:17:49.809 | 1 | OS integration recovered to 43/43; adapter E2E 78/79 because unchanged `slow-pauses` returned stale partial text. |
| 16c | one unchanged stability retry outside sandbox | 06:18:51.300–06:25:25.465 | 1 | Prior case passed; adapter E2E 78/79 due transient Windows `net::ERR_NO_BUFFER_SPACE` before adapter logic. |
| 16d | fresh outside-sandbox full verify after read-only diagnosis and idle recovery | exact UTC not retained | 1 | Unit 873/873, component 281/281, architecture 2/2, OS 43/43, adapter 79/79; app E2E 2/3 after the first responsive-shell test timed out for 20 seconds waiting for `<main>` on a blank cold-start page. The next two app tests passed. |
| diagnostic | `npm.cmd run test:app:e2e -- e2e/app-shell-responsive.spec.ts` outside sandbox | after 16d; exact UTC not retained | 0 | The formerly failing test passed 1/1 in 5.7 seconds (13.8 seconds total), with no tracked fix. |
| 16e | final fresh outside-sandbox `npm.cmd run verify` | local start 09:40:31; approximately 06:40–06:53 UTC | 0 | Full authoritative PASS; `All verification checks passed.` |
| 17 | `npm.cmd run check:rust:advisories` | after 16e; about 21 seconds | 1 | Expected moving RustSec result; exact blockers are recorded below. |
| 18 | `git diff --check` | after command 17 | 0 | No output. |

The final successful command 16e freshly passed:

- unit: 114 files / 873 tests;
- component: 62 files / 281 tests;
- architecture: 1 file / 2 tests;
- Windows OS integration: 4 files / 43 tests;
- Gemini adapter Playwright: 79/79;
- application Playwright: 3/3;
- Svelte diagnostics: 0 errors / 0 warnings;
- rustfmt;
- locked workspace/all-target Cargo tests: `extractum` 689,
  `extractum-analysis` 112, `extractum-core` 22,
  `extractum-gemini-browser` 78, `extractum-llm` 38,
  `extractum-prompt-packs` 253, and `extractum-telegram` 74; binary targets
  selected 0 tests; and
- the verifier's final `git diff HEAD --check`.

The read-only diagnosis is retained at
`.superpowers/sdd/final-hardening-verify-diagnosis.md`. It found that the
final-hardening range changes no adapter, app E2E, Vite, or Playwright source;
the relevant JavaScript manifest delta is only `package.json`. The partial
slow-pause result is explained by an existing read-text/read-controls race; the
network and blank cold-start failures recovered without repository changes.

## Advisory Result

The scheduled/release advisory lane remains red with exit 1. It reported
exactly three RustSec blockers:

- `RUSTSEC-2026-0190`: `anyhow 1.0.102` (fixed in `>=1.0.103`);
- `RUSTSEC-2026-0221`: `event-listener 5.4.1` (fixed in `>=5.4.2`);
- `RUSTSEC-2026-0097`: `rand 0.7.3` (fixed branches are described by the advisory).

It also reported warning-only yanked crate `spin 0.9.8`. No advisory exception
was added. Release and bundle acceptance remain blocked by this result.

## Authorized-Scope and Dependency Diff

The implementation changes only paths authorized by the approved design and
plan: the cargo-deny tool/action/runner and docs; the dependency/duplicate
policy modules, artifacts, and tests; the three Rust workflows and their YAML
contract test; the approved package files; strict `deny.toml`; the two bounded
Rust source files; and factual registry/operator/previous Wave 0 documents.

No Cargo manifest changed in `76e7db86..563783cc`, and
`git diff --exit-code 76e7db86..563783cc -- src-tauri/Cargo.lock` is empty.
The only npm dependency delta is root devDependency `yaml` at `^2.9.0` plus the
`node_modules/yaml` lock entry. No existing npm dependency entry or version
moved. The generated dependency-policy artifact records that factual edge; it
does not infer a new approval.

The tested implementation worktree was tracked-clean before this evidence file
was created. Final local `git diff --check` exited 0.

## Remote Evidence Pending

This branch has not been pushed by this verification work. No pull request or
remote Rust Fast/Rust Full run URL exists to record, and no workflow was
manually dispatched. Local acceptance therefore does not establish remote CI
completion.

The advisory gate is red, so no release `windows-bundle` job was run. No
release executable, MSI, NSIS installer, artifact hash, packaged-application
smoke, or sidecar smoke exists or is claimed. Merge readiness still requires
green remote Rust Fast and Rust Full runs for this reviewed head. Release
readiness additionally requires a successful advisory scan and gated bundle
evidence.
