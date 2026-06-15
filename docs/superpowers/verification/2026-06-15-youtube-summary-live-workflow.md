# YouTube Summary Live Workflow Verification

Date: 2026-06-15

## Scope

Focused hardening smoke for the shipped YouTube Summary / Prompt Pack MVP.
This check used the running Tauri app through MCP Bridge. The first pass avoided
provider work; after explicit approval, the check started real YouTube Summary
runs and exercised the real cancel command path.

## Live Environment

- Tauri MCP Bridge: connected to `org.ai.extractum` at `localhost:9223`.
- App route base: `http://localhost:1420`.
- Real local Library data was present; no dev-only smoke fixture was needed for
  launch/preflight/result browsing.

## Manual Smoke

- Opened `/projects/library` with a real navigation/reload, not only
  `history.pushState`: PASS.
- Confirmed Library loaded real sources: 29 total, 8 YouTube, 6 videos,
  2 playlists: PASS.
- Selected synced YouTube video source:
  - source id `402`
  - title `This NotebookLM Trick Changes Everything!`
  - external id `MeUemwjlGSk`
  - 14 local items
  - status `active`
- Opened `YouTube Summary` launch dialog from Library Inspector: PASS.
- Preflight for source `402`: PASS.
  - `includedVideos.length = 1`
  - `skippedVideos.length = 0`
  - `blockingFailures.length = 0`
  - `estimatedInputTokens = 3272`
  - `selectedModelInputLimit = 32000`
- Checked live active Prompt Pack runs through `window.__TAURI__.core.invoke`:
  PASS, `list_active_prompt_pack_runs` initially returned `[]`.
- After explicit approval to run provider work, clicked `Start` from the
  YouTube Summary dialog for source `402`: PASS.
  - created run `#8`
  - run completed before the cancel UI could be exercised
- Started another live run directly through Tauri IPC to get the run id quickly:
  PASS.
  - created run `#9`
  - run was visible as active/queued immediately after start
  - by the time `/projects/runs` loaded, the run had already failed with
    `malformed JSON braces`, so UI cancel was no longer available
- Started and immediately cancelled a third live run directly through Tauri IPC:
  PASS.
  - created run `#10`
  - `cancel_prompt_pack_run({ runId: 10 })` returned successfully
  - `list_active_prompt_pack_runs` returned `[]`
  - `/projects/runs` showed run `#10` as `cancelled / none`
- Opened `/projects/runs` with a real navigation/reload: PASS.
- Confirmed terminal Prompt Pack run grid rendered recent `youtube_summary`
  runs: PASS.
  - visible terminal runs included `#7`, `#6`, and `#5`
  - selected run `#7` rendered `complete / complete`
- Confirmed report workspace rendered after reload: PASS.
  - Result metrics: Sources 1, Videos 1, Claims 3, Evidence 2, Findings 0
  - Videos, Claims, Evidence, Stages and Artifacts, Audit Events, and
    Canonical JSON sections were visible.
- Opened stage artifact `raw_output #2`: PASS.
  - `Selected Artifact` rendered JSON with `claim_candidates`,
    `evidence_fragment_candidates`, `schema_version`, and
    `stage = youtube_summary/transcript_analysis`.
- Opened terminal run delete confirmation for run `#7`: PASS.
  - Modal title: `Delete project run?`
  - Message: `Project run #7 will be removed from the local database.`
  - Buttons: `Cancel`, `Delete`
- Clicked `Cancel` in the delete confirmation and rechecked recent runs:
  PASS, run `#7` was still present.
- Reloaded `/projects/runs` after run `#10`: PASS.
  - Found a UI defect where the cancelled/no-result report panel rendered
    `[object Object]`.
  - Fixed the report panel to use shared app-error formatting.
  - Rechecked the live screen and confirmed the panel now renders
    `Error loading project run report: Database error: no rows returned by a query that expected to return at least one row`
    instead of `[object Object]`.

## Automated Verification

Commands run after this note was created:

```powershell
npm.cmd run verify:project-runs
git diff --check
```

Results:

- `test:project-runs`: PASS, 2 files / 12 tests.
- `npm run check`: PASS, `svelte-check found 0 errors and 0 warnings`.
- `test:rust:prompt-pack-runs`: PASS, 8 tests.
- `git diff --check`: PASS.

Notes:

- The Rust command compiled in the sandbox target for about one minute before
  running tests. The tests themselves completed in `0.19s`.
- The only warning was the existing Rust warning about unused
  `PreflightYoutubeSummaryRunRequest` fields.

## Residual Risk

- Real backend cancellation was exercised against run `#10`.
- The UI cancel button/confirmation for an active run was still not fully
  exercised, because live runs `#8` and `#9` reached terminal states before the
  UI could cancel them.
- Run `#9` surfaced a provider-output robustness issue:
  `malformed JSON braces`. That is separate from the project-run management UI
  and should be investigated as prompt/output parsing hardening.
