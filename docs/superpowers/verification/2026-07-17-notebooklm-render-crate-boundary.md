# NotebookLM Render Crate Boundary Verification

**Date:** 2026-07-17
**Starting commit:** `945bd1b5a16ef4f63294f185c150d12718383a65`
**Decision:** `no_go`

## Outcome

Stage 0 rejected the full-workspace compile-time hypothesis. Editing the
already-extracted `extractum-core` dependency was not faster than editing the
NotebookLM renderer inside the application package under the canonical command:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

The proposed `extractum-notebooklm-render` crate was therefore not created.
No manifest, Rust source, workspace member, compatibility facade, test path, or
source-boundary contract was changed. Conditional Tasks 3-8 were not executed.

## Environment

- OS: Microsoft Windows NT 10.0.26100.0.
- CPU: Intel64 Family 6 Model 158 Stepping 11, GenuineIntel.
- Logical cores: 4.
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`, MSVC host, LLVM 22.1.2.
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`.
- Power profile: Balanced
  (`381b4222-f694-41f0-9685-ff5bb260df2e`).
- Microsoft Defender real-time protection: enabled.
- Canonical target: `G:\Develop\Extractum\src-tauri\target`.
- Active Cargo, rustc, rust-analyzer, Tauri dev/build, Vitest, Vite, or
  Extractum processes: 0.
- Four idle `@hypothesi/tauri-mcp-server` Node processes were recorded but
  excluded from the build/dev-process filter because they do not invoke Cargo
  or own the canonical target.

## Stage 0 Samples

One warm-up per variant was discarded before the recorded series. The first
application warm-up was cold at 55,576 ms; the core warm-up was 10,108 ms.
Neither contributes to a median.

| Variant | Recorded samples (ms) | Median (ms) |
| --- | --- | ---: |
| Application renderer | 9,081; 9,102; 10,153; 9,084; 9,090 | 9,090 |
| Extracted core surrogate | 9,091; 9,112; 9,119; 9,100; 9,083 | 9,100 |
| Focused core diagnostic | 1,018; 1,019; 1,020; 1,028; 1,031 | 1,020 |

The application and core series alternated after their discarded warm-ups.
Every timed probe began with a successful no-op workspace check, appended one
inert comment, invoked Cargo once, and restored the source bytes in `finally`.

## Decision

- Full-workspace percent improvement: `-0.1100%`; required `>= 25%`.
- Full-workspace absolute improvement: `-10 ms`; required `>= 2,000 ms`.
- Percent gate: failed.
- Absolute gate: failed.
- Result: `no_go`.

The surrogate is conservative because `--all-targets` also checks core's own
test targets and `extractum-core` is more broadly shared than the proposed
render crate. The result is not marginal: it misses both gates by orders of
magnitude, so that bias cannot change the decision.

The focused package result is materially faster, but it is diagnostic only.
It supports a different product goal—developers explicitly running
`cargo check -p <domain-crate>`—and does not improve the selected workspace
loop or justify proceeding under this specification.

## Integrity

- `src-tauri/src/notebooklm_export/renderer.rs` was restored to SHA-256
  `1EB52F3D3DE73C319CDFAB1DECDBBF37B8DA4C16DEC3B9FAD7EB0BD6763CB439`.
- `src-tauri/crates/extractum-core/src/media_metadata.rs` was restored to
  SHA-256
  `8DD2ACFE2CAB0B60178DFE522F472892F57105FCD4F2714C5548C796D5656922`.
- All 15 recorded samples have complete metadata, exit code 0, and
  `restored = true`.
- No recorded sample was discarded or replaced.
- The initial runner invocation was blocked before Cargo started by the local
  PowerShell execution policy. It produced no measurement sample. The plan was
  corrected to use process-local `-ExecutionPolicy Bypass`, committed, and a
  new session was started from the commit recorded above.
- Raw logs, source snapshots, metadata, and JSON summaries are outside the
  repository at
  `C:\Users\Dima\AppData\Local\Temp\extractum-notebooklm-render-945bd1b5a16ef4f63294f185c150d12718383a65`.
- The repository was clean immediately after all probe files were restored.

## Follow-Up

Do not extract the seven NotebookLM render modules for the purpose of speeding
up the full workspace check. If focused package development is valuable enough
to pursue, define it as a separately approved workflow with its own commands,
thresholds, and developer guidance rather than reinterpreting this negative
result.
