# Analysis UI Smoke Harness Verification

Date: 2026-05-30

Command:

```powershell
npm.cmd run smoke:analysis
```

Result: passed

Covered surfaces:

- Source Browser: Telegram live source
- Source Browser: YouTube video
- Source Browser: YouTube playlist
- Source Browser: live source group
- Source Browser: run snapshot
- Workspace Parity: single-source setup tools
- Workspace Parity: source-group disabled export
- Workspace Parity: opened single-source run tools
- Workspace Parity: opened source-group run export safety
- Workspace Parity: source mode tools placement

Notes:

- Smoke command remains opt-in.
- `npm.cmd run verify` does not run `smoke:analysis`.
- Fixture cleanup completed and second cleanup summary verified zero fixture rows.
