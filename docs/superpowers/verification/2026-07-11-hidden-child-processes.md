# Release GUI verification: hidden Windows child processes

## Build evidence

Command:

```powershell
npm.cmd run tauri build -- --no-bundle --features csp-verification
```

Result: completed successfully (exit code 0) and produced
`src-tauri/target/release/extractum.exe`. The build emitted 11 Rust warnings.

Release automation was unavailable because the automation bridge is gated by
`#[cfg(dev)]`; the GUI evidence below is user-observed.

## Manual release-GUI observations

The release executable was launched from
`src-tauri/target/release/extractum.exe`. The user manually exercised each path
and reported `окна нет` ("there is no window") for all of them. This is user-observed
desktop evidence, not automation evidence.

| Path | User observation |
| --- | --- |
| Navigate to **Analysis** (version probe) | No console window flashed or remained visible. |
| Navigate to **Diagnostics** (version probe) | No console window flashed or remained visible. |
| Start a YouTube metadata/preview operation (real `yt-dlp` invocation) | No console window appeared or remained visible. |

The user also confirmed that normal results and errors behaved as usual for all
three exercised paths.

## Why this uses a release GUI build

This check must use a release GUI executable. A dev build has a parent terminal,
which can make a child-process console visibility check falsely appear to pass.
