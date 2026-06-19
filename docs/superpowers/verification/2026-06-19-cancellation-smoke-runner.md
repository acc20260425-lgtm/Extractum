# Cancellation Smoke Runner

Use this local smoke after changes to cancellation, job lifecycle, scheduler
request tracking, Prompt Pack runtime, Analysis runs, YouTube source jobs, or
Takeout import state.

Prerequisites:

- run a debug Tauri app build;
- keep the MCP Bridge plugin connected;
- on Windows, prefer `npm.cmd` because PowerShell may block `npm.ps1`.

Command:

```powershell
npm.cmd run smoke:cancellation
```

The runner connects to the already running app and exercises the normal cancel
commands for:

- Analysis report runs;
- Prompt Pack runs;
- YouTube source jobs;
- Takeout imports.

Expected result:

- each scenario prints `PASS`;
- each seeded job or run reaches `cancelled`;
- fixture cleanup runs before the script exits;
- the script exits with code `0` and prints `Cancellation smoke passed.`

If one scenario fails, check the printed step name first. The script still
attempts fixture cleanup for the scenario it touched.
