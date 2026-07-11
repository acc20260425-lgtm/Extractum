# Security hardening verification — 2026-07-11

## Automated

- `npm.cmd run check` — passed (0 errors, 0 warnings).
- `npm.cmd run build` — passed.
- `cargo check --manifest-path src-tauri/Cargo.toml` — passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — passed.
- `git diff --check` — passed.
- `npm.cmd run test` — 1174/1176 passed. Pre-existing unrelated contract failures: `library-prototype-contract.test.ts` and `research-projects-import-boundary.test.ts` expect in-component grid columns that were moved to shared grid helpers.

## Dev MCP and smoke

- `npm.cmd run tauri dev` discovered MCP Bridge on `127.0.0.1:9223`; listener inspection confirmed the Extractum PID was loopback-only.
- `npm.cmd run smoke:cancellation` — passed all four cancellation scenarios.
- After the dev app was stopped and port 9223 was free, `npm.cmd run smoke:analysis` passed all bridge and analysis UI scenarios. Its dev child later reported the existing bundled prompt-pack hash conflict during shutdown.

## Build boundaries

- `npm.cmd run tauri build -- --debug --no-bundle` — passed.
- `npm.cmd run tauri build -- --no-bundle` — passed.
- `npm.cmd run tauri build -- --no-bundle --features csp-verification` — passed.

The build logs contain known Gemini/Apalis/fixture and third-party SVAR warnings. No new security-hardening warning was introduced.
