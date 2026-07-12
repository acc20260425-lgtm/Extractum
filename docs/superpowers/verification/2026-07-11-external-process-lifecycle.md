# External process lifecycle verification

Date: 2026-07-12

- `cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture`: 10 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml gemini_browser::cdp_chrome -- --nocapture`: 8 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml gemini_browser -- --nocapture`: 96 passed.
- `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`: 5 passed.
- `cargo check --manifest-path src-tauri/Cargo.toml` and `git diff --check`: passed.
- `npm.cmd run tauri build -- --no-bundle --features csp-verification`: passed; built `src-tauri/target/release/extractum.exe`.

Manual Windows verification on the release executable:

- Extractum-started CDP Chrome and `yt-dlp` disappeared from Task Manager after cleanup.
- Gemini sidecar cleanup and CDP Chrome isolation were checked: stopping Extractum-owned Chrome did not stop an unrelated user Chrome instance.

Live Gemini authentication in managed mode remains subject to Google's browser-security policy. Windows crash-containment of a deliberately force-terminated application was not manually exercised.
