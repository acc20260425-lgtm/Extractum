# Source Browser Manual Smoke

> Date: 2026-05-30
> Branch: `main`
> Scope: canonical Source Browser surfaces after explicit subject contract cleanup.

## Setup

The Tauri dev app was started from `main` with:

```bash
npm.cmd run tauri dev
```

Fixtures were reset through the MCP bridge:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

Fixture summary:

```text
accounts: 1
chatMessages: 2
llmProfiles: 1
promptTemplates: 1
runs: 6
snapshotMessages: 4
sourceGroups: 1
sources: 4
youtubePlaylistItems: 2
youtubeTranscriptSegments: 3
```

## Results

| Surface | Result | Evidence |
| --- | --- | --- |
| Telegram live source | PASS | `Timeline | Items | Metadata | Activity`; `Timeline` selected by default. |
| YouTube video live source | PASS | `Transcript | Comments | Items | Metadata | Activity`; `Transcript` selected by default. |
| YouTube playlist live source | PASS | `Videos | Items | Metadata | Activity`; `Videos` selected by default. |
| Live source group | PASS | `Sources | Items | Metadata | Activity`; `Sources` selected by default. |
| Run snapshot | PASS | Header showed `Run snapshot` and `View live source`; tabs were `Sources | Items | Metadata`; no `Activity` tab. |

## Additional Checks

- The run snapshot `View live source` action transitioned back to the live group Source Browser.
- Webview console logs contained only MCP bridge info lines.
- The Tauri dev process was stopped after the smoke.
- Final working tree was clean on `main`.

## Notes

The expected YouTube video live-source tab order is:

```text
Transcript | Comments | Items | Metadata | Activity
```
