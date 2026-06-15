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

## Follow-up: Run #9 Provider Output

Run `#9` failed in `youtube_summary/transcript_analysis` with
`malformed JSON braces`.

Evidence from stage `#66`:

- `raw_output #2` was a JSON artifact shaped as `{ "text": "..." }`.
- The provider text began as a valid transcript-analysis JSON object.
- The text ended immediately after `"evidence_fragment_candidates":`, without
  a value or closing braces.
- `error #99` contained `{ "error": "malformed JSON braces" }`.

Root cause:

- The stage parser correctly rejected an incomplete JSON object.
- The LLM request layer did not expose a per-request output-token budget, so
  the provider request relied on provider defaults. For this stage, that can
  produce a truncated JSON response that the parser reports as malformed braces.

Fix:

- Added `max_output_tokens` to the shared LLM chat request model.
- Passed it to OpenAI-compatible requests as `max_tokens`.
- Passed it to Gemini requests as `generationConfig.maxOutputTokens`.
- Set YouTube Summary transcript-analysis stage requests to a `4096` output
  token stage budget.
- The stage budget is clamped to the selected provider model's
  `output_token_limit` when that metadata is available.

Verification:

- `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-json --lib llm::`: PASS, 45 tests.
- `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-json --lib prompt_pack_run`: PASS, 8 tests.
- `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-json --lib transcript_analysis`: PASS, 6 tests.

Residual risk:

- Closed by live provider run `#11` after explicit approval.

## Follow-up: Run #11 Live Provider Recheck

After the output-token-budget fix, a new live YouTube Summary provider run was
started through MCP Bridge.

Preflight:

- source id `402`
- `includedVideos.length = 1`
- `blockingFailures.length = 0`
- `estimatedInputTokens = 3272`
- `selectedModelInputLimit = 32000`

Backend freshness check:

- `src-tauri\target\debug\extractum.exe` last write time:
  `15.06.2026 11:52:01`
- running `extractum.exe` process start time: `15.06.2026 11:52:07`
- This indicates the live app was running the rebuilt backend after the Rust
  change.

Run result:

- client request id: `codex-live-output-budget-1781513960231`
- run id: `#11`
- final run status: `complete`
- final result status: `complete`
- transcript-analysis stage id: `#82`
- transcript-analysis stage status: `succeeded`

Artifacts from stage `#82`:

- `prompt_input #1`: present
- `raw_output #2`: present
- `parsed_output #3`: present
- `metrics #4`: present

Raw/parsed output checks:

- `raw_output #2` inner provider text parsed as valid JSON.
- Raw JSON keys:
  `claim_candidates`, `evidence_fragment_candidates`, `schema_version`,
  `stage`, `stage_io_version`, `video_candidate`, `warning_candidates`.
- The old truncation tail after `"evidence_fragment_candidates":` was not
  present.
- `parsed_output #3` contained `evidence_fragment_candidates`.
- Validation findings for the run: `[]`.

Metrics:

- `input_tokens = 4852`
- `output_tokens = 708`
- `latency_ms = 4216`
- `validation_error_count = 0`

Conclusion:

- The new live provider run did not reproduce the malformed JSON braces failure
  seen in run `#9`.
- The transcript-analysis stage succeeded with raw JSON, parsed output, metrics,
  and final canonical result persisted.

Post-run hardening:

- Added provider model output-limit lookup after run `#11`.
- Moved the transcript-analysis `4096` stage output budget from Rust code into
  bundled runtime configuration:
  `src-tauri/prompt-packs/youtube_summary/1.0.0/runtime/transcript_analysis.json`.
- Effective transcript-analysis request limit is now:
  `min(4096, model.output_token_limit)` when provider metadata is available,
  otherwise `4096`.
- This clamp is covered by unit tests; no extra live provider run was started
  for the clamp-only hardening.

## Synthesis Stage

- Date: 2026-06-15
- Scope: YouTube Summary transcript-analysis plus run-scoped synthesis stage.
- Automated verification:
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib`: PASS, 763 tests.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_budget_comes_from_stage_runtime_config`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_synthesis_stage_input_collects_successful_transcript_outputs`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_synthesis_stage_input_uses_latest_parsed_output_wrappers`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator`: PASS, 9 tests.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_is_written_to_quarantine_artifacts`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_with_unknown_source_ref_is_quarantined`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_surfaces_quarantine_write_failure`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib execute_synthesis_stage_rejects_invalid_output_without_success_artifacts`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_run_executes_synthesis_after_transcript_stages`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_single_video_run_skips_synthesis`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_run_marks_partial_when_synthesis_fails`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib result_builder`: PASS, 6 tests.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib persist_final_result_projects_youtube_synthesis_items`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib repair_rebuilds_missing_youtube_synthesis_projection_rows`: PASS.
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib migrations::tests::build_migrations_starts_at_current_schema_baseline`: PASS.
  - `git diff --check`: PASS.
- MCP Bridge smoke:
  - Status: PASS for non-provider command/UI smoke.
  - Connected to `org.ai.extractum` through Tauri MCP Bridge at `localhost:9223`.
  - Backend state returned app `extractum` version `0.2.0`, Tauri `2.10.3`, window `main`.
  - `window.__TAURI__.core.invoke("list_active_prompt_pack_runs", {})` returned `[]`.
  - `window.__TAURI__.core.invoke("list_prompt_pack_runs", {})` returned recent `youtube_summary@1.0.0` runs.
  - Opened `/projects/runs` in the webview and verified the Prompt Pack runs grid rendered recent `youtube_summary@1.0.0` rows.
  - Verified the report workspace rendered a `Synthesis` section and stage list through the accessibility tree.
  - Scope: non-provider Tauri Bridge command/UI smoke only; no new provider run was started.
- Live provider verification:
  - Status: PASS with one follow-up canonicalization defect found and fixed.
  - Source ids: `404`, `402`.
  - Project id: `6`.
  - Preflight:
    - `includedVideos.length = 2`
    - `skippedVideos.length = 0`
    - `blockingFailures.length = 0`
    - `estimatedInputTokens = 6546`
    - `selectedModelInputLimit = 32000`
  - Client request id: `codex-live-synthesis-1781530480448`.
  - Run ID: `#14`.
  - Final run status: `complete`.
  - Final result status: `complete`.
  - Final progress: `3 / 3`.
  - Transcript stages:
    - stage `#106`: `succeeded`
    - stage `#107`: `succeeded`
  - Synthesis stage:
    - stage `#111`: `succeeded`
    - artifacts present: `prompt_input #1`, `raw_output #2`,
      `parsed_output #3`, `metrics #4`
    - metrics: `input_tokens = 4024`, `output_tokens = 406`,
      `latency_ms = 16984`, `validation_error_count = 0`
  - Canonical synthesis:
    - `synthesis` was non-null.
    - `cross_video_themes.length = 2`
    - `common_claims.length = 1`
    - `contradictions_across_videos.length = 0`
    - `source_refs.length = 2`
    - validation findings: `[]`
    - storage warning: `null`
  - Projection rows:
    - `prompt_pack_youtube_videos = 2`
    - `prompt_pack_result_source_refs = 2`
    - `prompt_pack_youtube_synthesis_items = 2`
    - synthesis projection rows:
      - `theme_1`: `Enhanced Functionality and Integration of NotebookLM`
      - `theme_2`: `AI-Powered Content Generation and Application Development`
  - Follow-up defect found during live verification:
    - The provider returned `synthesis_candidate.common_claims[0].text`.
    - The canonical builder only read `summary_text`, so canonical
      `common_claims[0].summary_text` became empty for run `#14`.
    - Fixed result-builder canonicalization to preserve `text` as a fallback
      when `summary_text` is absent.
    - Added regression test:
      `build_canonical_result_preserves_synthesis_common_claim_text`.
    - Red/green evidence:
      - Before fix, the regression test failed with left `String("")`.
      - After fix, the regression test passed.
    - Focused verification:
      - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-review-youtube-summary-synthesis --lib build_canonical_result_preserves_synthesis_common_claim_text`: PASS.
      - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-review-youtube-summary-synthesis --lib result_builder`: PASS, 7 tests.
      - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-review-youtube-summary-synthesis --lib prompt_packs`: PASS, 67 tests.
- Notes:
  - Full lib verification initially exposed that the migration baseline test still expected versions `[1, 2, 3, 4, 5, 6, 7]` while the repository now registers `0008_prompt_pack_run_labels.sql`. The expected version list was corrected to `[1, 2, 3, 4, 5, 6, 7, 8]`, then the focused migration test and full lib suite passed.
  - The automated synthesis tests cover runtime budget loading, synthesis input assembly, validator/quarantine behavior, stage artifact persistence, run-level synthesis lifecycle, canonical result building, and projection repair.
