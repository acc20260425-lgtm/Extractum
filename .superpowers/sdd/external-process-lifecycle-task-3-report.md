# Task 3 evidence

## RED

`cargo test --manifest-path src-tauri/Cargo.toml youtube::process_runtime -- --nocapture`
was run after registering the new module and before its implementation. It failed
with `E0432`, because `YoutubeProcessRegistry` did not exist.

## Implemented

- Added `YoutubeProcessRegistry` reservation/cancellation ownership and
  `ManagedYtdlpGuard` cleanup.
- Added synchronous `YtdlpLauncher`, `SpawnedYtdlp`, process-tree assignment,
  admission-protected reservation/spawn, and concurrent stdout/stderr draining.
- Routed preview, metadata, comments, captions, and jobs through the managed
  runner without changing public Tauri DTOs/commands.
- Registered the process registry as managed app state.

## GREEN / checks

- `cargo check --manifest-path src-tauri/Cargo.toml`: exit 0.
- Follow-up focused test command exceeded the 120-second command limit while
  compiling the full Rust test binary; no test result was produced. The output
  contained no compile error before timeout.

## Timeout diagnosis (follow-up)

The timeout was not a test hang. A fresh
`cargo test --manifest-path src-tauri/Cargo.toml youtube::process_runtime --no-run`
completed successfully in 163.85 seconds and spent its time compiling the
full `extractum` test-profile binary. After that warm compilation, the focused
test command completed successfully in 1.82 seconds.

## Remaining concerns

The requested injected fake launcher/backpressured-pipe regression, cookie
lifetime cases, and detached stuck-reap fallback are not yet implemented.
Therefore this task is not ready for review or commit.

## Follow-up: injected stuck-reap lifecycle regression

- Added `injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release`
  in `src-tauri/src/youtube/process_runtime.rs`. It invokes the actual managed
  runner through `FakeYtdlpLauncher`, forces the normal timeout followed by the
  one-second reap budget, and verifies the existing network timeout error,
  cookie retention, and retained registry operation while the fake child is
  stuck. Once released, it verifies cookie deletion and an empty registry.
- The requested RED baseline could not be reproduced without reverting the
  already-present `detach_owned_reap` production wiring, which was explicitly
  out of scope for this follow-up. The new test's first executable run was
  green against that pre-existing wiring; the preceding attempt timed out at
  the 120-second command limit during full test-binary compilation, with no
  compiler or test failure output.

## Follow-up verification

- `cargo test --manifest-path src-tauri/Cargo.toml youtube::process_runtime -- --nocapture`:
  exit 0; 8 passed, 0 failed (1.06s test execution).
- `cargo test --manifest-path src-tauri/Cargo.toml youtube -- --nocapture`:
  exit 0; 280 passed, 0 failed.
- `cargo check --manifest-path src-tauri/Cargo.toml`:
  exit 0. Existing unrelated warnings remain in `gemini_browser` and
  `external_process`.
