# Extractum value registry

This document is the human-readable registry for controlled values used across
Extractum: statuses, states, phases, provider kinds, source kinds, event kinds,
filters, UI tones, and reason codes.

It is intentionally a documentation registry first. Runtime code should not
import this file. After values are reviewed and stabilized, selected domains can
be promoted into a code registry such as `src/lib/status-registry.ts`.

## Registry rules

- `Value` is the machine value. Treat backend, database, event, and persisted
  values as stable contracts.
- `Name` is the short human-readable label we want to show or document.
- `Meaning` describes the actual product semantics, not only the literal word.
- `Source of truth` identifies where the value is produced.
- `Lifecycle` uses a shared classification: `idle`, `active`, `transitional`,
  `terminal`, `derived`, `filter`, `presentation`, or `taxonomy`.
- `User action` describes the default UI affordance: `none`, `wait`, `retry`,
  `login`, `configure`, `cancel`, `inspect_error`, or `choose`.
- `Stable?` means whether renaming the value would affect stored data, API
  contracts, events, or persisted UI state.
- `Current usage` lists representative files, not every reference.

## Review checklist

Use this checklist when adding or changing values:

- Is this value a real machine value, or only user-facing message text?
- Is it produced by backend/database code, frontend derived state, or UI only?
- Is it persisted in the database, local storage, URLs, or event payloads?
- Does the value describe lifecycle, taxonomy, capability, filter, or
  presentation?
- Is there an existing value with the same meaning under another name?
- Does the value need a terminal/active/transitional classification?
- Does the UI need a default tone, icon, action, or disabled reason for it?
- Can the value be renamed safely, or does it require a migration?

## Normalization notes

- Prefer `completed` for event and process terminal states unless a backend
  contract already uses `complete` or `succeeded`.
- Prefer `failed` for process terminal states and `error` for UI/catalog
  derived states that summarize one or more failures.
- Prefer `unavailable` when a resource cannot be used, and `unknown` when the
  application cannot classify the condition.
- Prefer `cancel_requested` for transitional cancellation and `cancelled` for
  terminal cancellation.
- Keep provider/source taxonomy values lowercase snake_case.
- Do not merge UI tone values with domain status values. A status maps to a
  tone; it is not itself a tone.

## Value type taxonomy

| Type | Meaning | Examples |
| --- | --- | --- |
| `status` | Stored or exchanged state of a process or record. | `queued`, `running`, `completed` |
| `state` | Derived state in a frontend model or backend helper. | `available`, `checking`, `inconsistent` |
| `phase` | Step inside an active process. | `chunking`, `map`, `writing` |
| `kind` | Discriminator for event, source, object, or error families. | `telegram`, `progress`, `validation` |
| `mode` | Runtime or UI operation mode. | `managed`, `cdp_attach`, `report` |
| `filter` | Persisted or UI filter value. | `all`, `queued_running` |
| `tone` | Presentation value for visual treatment. | `success`, `danger` |
| `message` | User-facing text only. Not a controlled status unless promoted. | `Project deleted.` |

## Lifecycle taxonomy

| Lifecycle | Meaning |
| --- | --- |
| `idle` | Nothing has started or no work is required. |
| `active` | Work is currently happening. |
| `transitional` | Temporary value between active and terminal states. |
| `terminal` | Process will not progress without a new action. |
| `derived` | Computed from other data and not stored as a primary fact. |
| `filter` | UI or API filter, not an entity state. |
| `presentation` | Visual-only value. |
| `taxonomy` | Domain classification rather than lifecycle. |

## Analysis run statuses

Representative sources:

- `src/lib/types/analysis.ts`
- `src/lib/analysis-utils.ts`
- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/report.rs`

Analysis run status values are stored in `analysis_runs.status` and currently
arrive in TypeScript as `AnalysisRunSummary.status: string`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `queued` | status | Queued | Run is accepted but not executing yet. | backend/db | active | wait/cancel | info | yes | analysis runs, run events |
| `running` | status | Running | Run is executing. | backend/db | active | wait/cancel | info | yes | report canvas, runs list |
| `completed` | status | Completed | Run finished and report output is available. | backend/db | terminal | none | success | yes | report viewer, history |
| `failed` | status | Failed | Run ended with an error. | backend/db | terminal | inspect_error/retry | danger | yes | report viewer, history |
| `cancelled` | status | Cancelled | Run was cancelled before normal completion. | backend/db | terminal | inspect_error/retry | neutral | yes | history, run filters |

Allowed transitions:

```text
queued -> running
queued -> cancelled
running -> completed
running -> failed
running -> cancelled
```

## Analysis run filters and phases

Representative sources:

- `src/lib/types/analysis.ts`
- `src/lib/analysis-state.ts`
- `src/lib/analysis-utils.ts`
- `src/lib/analysis-run-companion-state.ts`

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `all` | filter | All | Show all runs. | frontend/API query | filter | choose | neutral | yes, persisted filter | run filters |
| `queued_running` | filter | Queued or running | Show active runs. | frontend/API query | filter | choose | info | yes, persisted filter | run filters |
| `completed` | filter | Completed | Show completed runs. | frontend/API query | filter | choose | success | yes, persisted filter | run filters |
| `failed` | filter | Failed | Show failed runs. | frontend/API query | filter | choose | danger | yes, persisted filter | run filters |
| `cancelled` | filter | Cancelled | Show cancelled runs. | frontend/API query | filter | choose | neutral | yes, persisted filter | run filters |
| `idle` | phase | Idle | No active report phase. | frontend derived | derived | none | neutral | no | phase labels |
| `queued` | phase | Queued | Active run is waiting. | backend event/frontend derived | active | wait | info | yes when event phase | phase labels |
| `load_items` | phase | Loading items | Source material is being loaded. | backend event | active | wait | info | yes when event phase | phase labels |
| `chunking` | phase | Chunking corpus | Material is split into model-sized chunks. | backend event | active | wait | info | yes when event phase | phase labels |
| `map` | phase | Analyzing chunks | Chunk analysis is running. | backend event | active | wait | info | yes when event phase | phase labels |
| `reduce` | phase | Writing report | Final report synthesis is running. | backend event | active | wait | info | yes when event phase | phase labels |
| `persist` | phase | Saving run | Results are being saved. | backend event | active | wait | info | yes when event phase | phase labels |
| `completed` | phase | Completed | Active phase ended successfully. | backend event | terminal | none | success | yes when event phase | phase labels |
| `failed` | phase | Failed | Active phase ended with an error. | backend event | terminal | inspect_error | danger | yes when event phase | phase labels |
| `cancelled` | phase | Cancelled | Active phase was cancelled. | backend event | terminal | inspect_error | neutral | yes when event phase | phase labels |

## Analysis events and chat events

Representative source: `src/lib/types/analysis.ts`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `queued` | event kind | Queued event | Work was queued. | backend event | active | wait | info | yes | `AnalysisRunEvent`, `AnalysisChatEvent` |
| `started` | event kind | Started event | Work started executing. | backend event | active | wait | info | yes | run/chat events |
| `progress` | event kind | Progress event | Progress counters or message changed. | backend event | active | wait | info | yes | run events |
| `delta` | event kind | Delta event | Streaming text delta arrived. | backend event | active | wait | info | yes | run/chat events |
| `completed` | event kind | Completed event | Work completed successfully. | backend event | terminal | none | success | yes | run/chat events |
| `failed` | event kind | Failed event | Work failed. | backend event | terminal | inspect_error | danger | yes | run/chat events |
| `cancelled` | event kind | Cancelled event | Work was cancelled. | backend event | terminal | inspect_error | neutral | yes | run/chat events |

## Snapshot states and availability

Representative sources:

- `src/lib/types/analysis.ts`
- `src/lib/analysis-run-snapshot-affordance.ts`

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `captured` | state | Captured | A frozen source snapshot was recorded. | backend/db | terminal | none | success | yes | run metadata |
| `capture_failed` | state | Capture failed | Snapshot capture failed. | backend/db | terminal | inspect_error | danger | yes | run metadata |
| `available` | state | Available | Snapshot rows can be loaded. | frontend probe | derived | none | none | no | snapshot affordance |
| `unavailable` | state | Unavailable | Snapshot rows are not available. | frontend probe | derived | inspect_error | warning | no | snapshot affordance |
| `loading` | state | Loading | Snapshot availability probe is in progress. | frontend probe | active | wait | info | no | snapshot affordance |
| `error` | state | Error | Snapshot availability probe failed. | frontend probe | terminal | inspect_error/retry | danger | no | snapshot affordance |
| `unknown` | state | Unknown | Snapshot availability cannot be classified yet. | frontend probe | derived | wait | neutral | no | snapshot affordance |
| `capture_failed_with_error` | state | Capture failed with error | Capture failed and a sanitized error is available. | frontend derived | derived | inspect_error | danger | no | snapshot affordance |
| `not_captured_before_terminal` | state | Not captured before terminal | Run ended before snapshot rows were captured. | frontend derived | derived | inspect_error | warning | no | snapshot affordance |
| `capture_failed_without_error_unknown` | state | Capture failed without error | Capture failed but no clear error is available. | frontend derived | derived | inspect_error | warning | no | snapshot affordance |
| `inconsistent` | state | Inconsistent | Metadata says captured, but rows cannot be loaded. | frontend derived | derived | inspect_error | danger | no | snapshot affordance |
| `verification_failed` | state | Verification failed | Snapshot verification failed. | frontend derived | derived | retry/inspect_error | danger | no | snapshot affordance |
| `checking` | state | Checking | UI is checking saved snapshot availability. | frontend derived | active | wait | info | no | snapshot affordance |
| `pending` | state | Pending | Snapshot is not ready for an active run. | frontend derived | active | wait | info | no | snapshot affordance |

## Source jobs

Representative source: `src/lib/types/sources.ts`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `queued` | status | Queued | Source job is waiting. | backend/db/event | active | wait/cancel | info | yes | YouTube/source jobs |
| `running` | status | Running | Source job is running. | backend/db/event | active | wait/cancel | info | yes | YouTube/source jobs |
| `succeeded` | status | Succeeded | Source job completed successfully. | backend/db/event | terminal | none | success | yes | YouTube/source jobs |
| `failed` | status | Failed | Source job failed. | backend/db/event | terminal | inspect_error/retry | danger | yes | YouTube/source jobs |
| `cancel_requested` | status | Cancel requested | Cancellation was requested but is not terminal yet. | backend/db/event | transitional | wait | warning | yes | YouTube/source jobs |
| `cancelled` | status | Cancelled | Source job was cancelled. | backend/db/event | terminal | retry | neutral | yes | YouTube/source jobs |

Allowed transitions:

```text
queued -> running
queued -> cancelled
running -> succeeded
running -> failed
running -> cancel_requested
cancel_requested -> cancelled
cancel_requested -> failed
```

Source job types:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `youtube_video_metadata_sync` | job type | YouTube video metadata sync | Refresh one video's metadata. | backend/frontend contract | taxonomy | yes | source jobs |
| `youtube_video_transcript_sync` | job type | YouTube video transcript sync | Fetch one video's transcript. | backend/frontend contract | taxonomy | yes | source jobs |
| `youtube_video_comments_sync` | job type | YouTube video comments sync | Fetch one video's comments. | backend/frontend contract | taxonomy | yes | source jobs |
| `youtube_video_full_sync` | job type | YouTube video full sync | Sync metadata, transcript, and comments for one video. | backend/frontend contract | taxonomy | yes | source jobs |
| `youtube_playlist_metadata_sync` | job type | YouTube playlist metadata sync | Refresh playlist metadata. | backend/frontend contract | taxonomy | yes | source jobs |
| `youtube_playlist_full_sync` | job type | YouTube playlist full sync | Sync playlist and videos. | backend/frontend contract | taxonomy | yes | source jobs |
| `youtube_playlist_video_sync` | job type | YouTube playlist video sync | Sync one video from a playlist context. | backend/frontend contract | taxonomy | yes | source jobs |

## Takeout imports

Representative source: `src/lib/types/sources.ts`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `queued` | status/phase | Queued | Import waits to start. | backend/db/event | active | wait/cancel | info | yes |
| `running` | status | Running | Import is running. | backend/db/event | active | wait/cancel | info | yes |
| `cancel_requested` | status | Cancel requested | Cancellation requested. | backend/db/event | transitional | wait | warning | yes |
| `completed` | status/phase | Completed | Import completed. | backend/db/event | terminal | none | success | yes |
| `failed` | status/phase | Failed | Import failed. | backend/db/event | terminal | inspect_error/retry | danger | yes |
| `cancelled` | status/phase | Cancelled | Import was cancelled. | backend/db/event | terminal | retry | neutral | yes |
| `resolving_source` | phase | Resolving source | Import is matching archive data to a source. | backend event | active | wait | info | yes |
| `starting_takeout` | phase | Starting Takeout | Import worker is starting Takeout processing. | backend event | active | wait | info | yes |
| `validating_peer` | phase | Validating peer | Import is validating Telegram peer identity. | backend event | active | wait | info | yes |
| `loading_splits` | phase | Loading splits | Import is loading archive split files. | backend event | active | wait | info | yes |
| `counting` | phase | Counting | Import is counting archive rows. | backend event | active | wait | info | yes |
| `importing_history` | phase | Importing history | Import is writing history rows. | backend event | active | wait | info | yes |
| `finishing_takeout` | phase | Finishing Takeout | Import is completing cleanup/finalization. | backend event | active | wait | info | yes |

Recovery values:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? |
| --- | --- | --- | --- | --- | --- | --- |
| `interrupted` | recovery kind | Interrupted | Import stopped without a normal terminal result. | backend derived | derived | yes |
| `failed` | recovery kind/status | Failed | Import failed and may need recovery. | backend derived | terminal | yes |
| `cancelled` | recovery kind/status | Cancelled | Import was cancelled and may need cleanup. | backend derived | terminal | yes |
| `partial_completed` | recovery kind | Partial completed | Import completed with partial data. | backend derived | terminal | yes |
| `complete` | completeness | Complete | Observed import appears complete. | backend derived | derived | yes |
| `partial` | completeness | Partial | Observed import is incomplete. | backend derived | derived | yes |
| `unknown` | completeness | Unknown | Completeness cannot be classified. | backend derived | derived | yes |

## NotebookLM export

Representative source: `src/lib/types/sources.ts`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `started` | event kind | Started | Export began. | backend event | active | wait | info | yes |
| `progress` | event kind | Progress | Export progress changed. | backend event | active | wait | info | yes |
| `completed` | event kind/phase | Completed | Export completed. | backend event | terminal | none | success | yes |
| `failed` | event kind/phase | Failed | Export failed. | backend event | terminal | inspect_error/retry | danger | yes |
| `loading` | phase | Loading | Export is loading source data. | backend event | active | wait | info | yes |
| `filtering` | phase | Filtering | Export is filtering source data. | backend event | active | wait | info | yes |
| `chunking` | phase | Chunking | Export is splitting output files. | backend event | active | wait | info | yes |
| `preparing_output` | phase | Preparing output | Export is preparing destination files. | backend event | active | wait | info | yes |
| `writing` | phase | Writing | Export is writing files. | backend event | active | wait | info | yes |
| `manifest` | phase | Manifest | Export is writing manifest data. | backend event | active | wait | info | yes |

## Prompt pack runs

Representative source: `src/lib/types/prompt-packs.ts`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `queued` | status/event kind | Queued | Prompt-pack run waits to start. | backend/db/event | active | wait/cancel | info | yes |
| `running` | status | Running | Prompt-pack run is running. | backend/db/event | active | wait/cancel | info | yes |
| `complete` | status | Complete | All required prompt-pack stages succeeded. | backend/db/event | terminal | none | success | yes |
| `partial` | status/event kind | Partial | Run produced a partial result. | backend/db/event | terminal | inspect_error/retry | warning | yes |
| `failed` | status/event kind | Failed | Run failed. | backend/db/event | terminal | inspect_error/retry | danger | yes |
| `cancelled` | status/event kind | Cancelled | Run was cancelled. | backend/db/event | terminal | retry | neutral | yes |
| `interrupted` | status/event kind | Interrupted | Run stopped outside normal completion. | backend/db/event | terminal | inspect_error/retry | warning | yes |
| `started` | event kind | Started | Prompt-pack run started. | backend event | active | wait | info | yes |
| `progress` | event kind | Progress | Prompt-pack run emitted progress. | backend event | active | wait | info | yes |
| `stage_started` | event kind | Stage started | A stage began. | backend event | active | wait | info | yes |
| `stage_completed` | event kind | Stage completed | A stage completed. | backend event | active | wait | success | yes |
| `stage_failed` | event kind | Stage failed | A stage failed. | backend event | active/terminal | inspect_error | danger | yes |
| `completed` | event kind | Completed | Event spelling for terminal completion. | backend event | terminal | none | success | yes |
| `preflight` | phase | Preflight | Validate inputs before running. | backend event | active | wait | info | yes |
| `snapshot` | phase | Snapshot | Capture source snapshot. | backend event | active | wait | info | yes |
| `stage` | phase | Stage | Execute a prompt-pack stage. | backend event | active | wait | info | yes |
| `gem_passport` | phase | Gem passport | Execute Gem analysis part 1, the analytical passport. | backend event | active | wait | info | yes |
| `gem_comments` | phase | Gem comments | Execute optional Gem analysis part 2 over the selected comment sample. | backend event | active | wait | info | yes |
| `gem_deep_recap` | phase | Gem deep recap | Execute Gem analysis part 3, the deep recap. | backend event | active | wait | info | yes |
| `gem_part_repair` | phase | Gem part repair | Repair one Gem analysis part JSON response. | backend event | active | wait | info | yes |
| `validation` | phase | Validation | Validate stage or final result. | backend event | active | wait | info | yes |
| `projection` | phase | Projection | Project result into canonical output. | backend event | active | wait | info | yes |
| `persist` | phase | Persist | Save artifacts and results. | backend event | active | wait | info | yes |
| `terminal` | phase | Terminal | Emit final state. | backend event | terminal | none | neutral | yes |
| `api` | runtime provider | API | Run prompt pack through API model providers. | frontend/backend contract | taxonomy | choose | neutral | yes |
| `gemini_browser` | runtime provider | Gemini Browser | Run prompt pack through browser automation. | frontend/backend contract | taxonomy | choose | neutral | yes |

Naming note: `PromptPackRunStatus` uses `complete`, while events use
`completed`. Keep this visible until the backend contract is intentionally
normalized.

Prompt-pack database-only values:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `pending` | stage status | Pending | Stage exists but has not started. | backend/db | idle | wait/cancel | neutral | yes | `prompt_pack_stage_runs.stage_status` |
| `running` | stage status | Running | Stage is executing. | backend/db | active | wait/cancel | info | yes | `prompt_pack_stage_runs.stage_status` |
| `succeeded` | stage status | Succeeded | Stage completed successfully. | backend/db | terminal | none | success | yes | `prompt_pack_stage_runs.stage_status` |
| `failed` | stage status | Failed | Stage failed. | backend/db | terminal | inspect_error/retry | danger | yes | `prompt_pack_stage_runs.stage_status` |
| `cancelled` | stage status | Cancelled | Stage was cancelled. | backend/db | terminal | retry | neutral | yes | `prompt_pack_stage_runs.stage_status` |
| `skipped` | stage status | Skipped | Stage was intentionally skipped. | backend/db | terminal | none | neutral | yes | `prompt_pack_stage_runs.stage_status` |
| `not_implemented` | stage status | Not implemented | Stage is declared but currently not implemented. | backend/db | terminal | inspect_error | neutral | yes | `prompt_pack_stage_runs.stage_status` |
| `none` | result status | No result | Run has no projected result yet. | backend/db | idle | wait | neutral | yes | `prompt_pack_runs.result_status` |
| `complete` | result status | Complete | Projected result is complete. | backend/db | terminal | none | success | yes | `prompt_pack_results.result_status` |
| `partial` | result status | Partial | Projected result is usable but incomplete. | backend/db | terminal | inspect_error | warning | yes | `prompt_pack_results.result_status` |
| `failed` | result status | Failed | Result projection or validation failed. | backend/db | terminal | inspect_error/retry | danger | yes | `prompt_pack_runs.result_status` |
| `active` | lifecycle status | Active | Prompt-pack version is currently selectable. | backend/db | taxonomy | choose | success | yes | `prompt_pack_versions.lifecycle_status` |
| `included` | inclusion status | Included | Source origin has a snapshot and is included. | backend/db | taxonomy | none | success | yes | `prompt_pack_run_source_origins.inclusion_status` |
| `skipped` | inclusion status | Skipped | Source origin was not included in the run snapshot. | backend/db | taxonomy | inspect_error | warning | yes | `prompt_pack_run_source_origins.inclusion_status` |
| `user` | origin kind | User | Prompt-pack version comes from user data. | backend/db | taxonomy | none | neutral | yes | `prompt_pack_versions.origin_kind` |

Open prompt-pack inventory note: bundled prompt-pack seed data also supplies
`origin_kind` and `lifecycle_status`; `active` and `user` are observed in code,
but additional seed-only values should be verified from prompt-pack assets
before being treated as complete.

Known naming collisions:

| Values | Meaning overlap | Current recommendation |
| --- | --- | --- |
| `complete` / `completed` | Both mean successful terminal completion. | Keep existing contracts; prefer `completed` for new event/process values. |
| `succeeded` / `completed` | Source jobs use `succeeded`, other processes use `completed`. | Keep source job contract; prefer `completed` for new general process values. |
| `failed` / `error` | Both communicate failure. | Use `failed` for terminal process status; use `error` for derived catalog/UI summaries. |
| `ready` / `available` | Both communicate usability. | Use `ready` for runtime/setup readiness; use `available` for resource availability. |
| `unknown` / `unavailable` | Both can appear when data cannot be used. | Use `unknown` for classification uncertainty; use `unavailable` for known lack of availability. |

## Gemini Browser provider and runs

Representative sources:

- `src/lib/types/gemini-browser.ts`
- `src-tauri/src/gemini_browser/types.rs`

Provider status:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `not_started` | status | Not started | Browser provider has not started. | backend state | idle | configure/start | neutral | yes |
| `ready` | status/run status | Ready | Provider or run is ready for use. | backend state | idle/terminal | none | success | yes |
| `needs_login` | status/run status | Needs login | User must log in to Gemini. | backend/browser detector | terminal | login | warning | yes |
| `needs_manual_action` | status/run status | Needs manual action | Browser requires user action. | backend/browser detector | terminal | inspect_error | warning | yes |
| `running` | status/run status | Running | Browser provider or run is active. | backend state | active | wait/cancel | info | yes |
| `stopped` | status | Stopped | Browser provider is stopped. | backend state | terminal | start | neutral | yes |
| `failed` | status/run status | Failed | Provider or run failed. | backend state | terminal | inspect_error/retry | danger | yes |

Run status:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `queued` | status | Queued | Browser run is queued. | backend run log | active | wait/cancel | info | yes |
| `running` | status | Running | Browser run is executing. | backend run log | active | wait/cancel | info | yes |
| `ok` | status | OK | Browser run completed with an answer. | backend run log | terminal | none | success | yes |
| `ready` | status | Ready | Browser became ready. | backend run log | terminal | none | success | yes |
| `blocked` | status | Blocked | Browser run cannot continue. | backend run log | terminal | inspect_error | warning | yes |
| `timeout` | status | Timeout | Answer did not complete before timeout. | backend run log | terminal | retry/inspect_error | warning | yes |
| `browser_crashed` | status | Browser crashed | Browser process or page crashed. | backend run log | terminal | retry/inspect_error | danger | yes |
| `cancelled` | status | Cancelled | Browser run was cancelled. | backend run log | terminal | retry | neutral | yes |

Manual action values:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `login` | manual action | Login | User should log in. | browser detector | terminal | login | yes |
| `account_picker` | manual action | Account picker | User should choose an account. | browser detector | terminal | choose | yes |
| `consent` | manual action | Consent | User should accept a consent dialog. | browser detector | terminal | choose | yes |
| `captcha` | manual action | Captcha | User should complete CAPTCHA. | browser detector | terminal | choose | yes |
| `unknown_modal` | manual action | Unknown modal | Browser shows an unclassified modal. | browser detector | terminal | inspect_error | yes |
| `start_chrome_cdp` | manual action | Start Chrome CDP | User should start/connect Chrome CDP. | provider setup | terminal | configure | yes |

Other Gemini Browser classifications:

| Value set | Type | Values | Source |
| --- | --- | --- | --- |
| Provider mode | mode | `managed`, `cdp_attach` | `src/lib/types/gemini-browser.ts` |
| Debug error stage | kind | `setup`, `composer`, `send`, `answer`, `artifacts`, `transport` | `src/lib/types/gemini-browser.ts` |
| Answer completion reason | reason | `stable`, `timeout_latest`, `missing` | `src/lib/types/gemini-browser.ts` |
| Candidate reject reason | reason | `baseline`, `composer`, `prompt_container`, `navigation`, `account_or_login`, `controls`, `multi_turn`, `not_visible`, `empty`, `lower_score` | `src/lib/types/gemini-browser.ts` |
| Answer grouping | kind | `assistant_turn`, `single_node`, `unknown` | `src/lib/types/gemini-browser.ts` |

## Gemini Browser setup checks

Representative source: `src/lib/gemini-browser-setup-status.ts`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `ready` | state | Ready | Check passed. | frontend derived | derived | none | success | no |
| `action_needed` | state | Action needed | User must fix or configure something. | frontend derived | derived | configure | warning | no |
| `warning` | state | Warning | Check is degraded but not blocking. | frontend derived | derived | inspect_error | warning | no |
| `failed` | state | Failed | Check failed. | frontend derived | derived | inspect_error/retry | danger | no |
| `running` | state | Running | Check is in progress. | frontend derived | active | wait | info | no |
| `unknown` | state | Unknown | Check cannot classify result yet. | frontend derived | derived | wait | neutral | no |
| `not_applicable` | state | Not applicable | Check does not apply in current mode. | frontend derived | derived | none | neutral | no |

## Accounts and runtime status

Representative source: `src/lib/types/accounts.ts`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `not_initialized` | state | Not initialized | Account runtime has not been initialized. | backend runtime | idle | configure | neutral | yes |
| `restoring` | state | Restoring | Account session restore is in progress. | backend runtime | active | wait | info | yes |
| `ready` | state | Ready | Account is ready to use. | backend runtime | idle | none | success | yes |
| `reauth_required` | state | Reauth required | User must authenticate again. | backend runtime | terminal | login | warning | yes |
| `restore_failed` | state | Restore failed | Session restore failed. | backend runtime | terminal | login/inspect_error | danger | yes |

## Library, projects, and catalog view statuses

Representative sources:

- `src/lib/types/library-sources.ts`
- `src/lib/ui/library-catalog-model.ts`
- `src/lib/ui/research-projects-model.ts`

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `active` | status | Active | Source is available and not currently syncing or errored. | backend/frontend derived | idle | none | success/neutral | yes for API |
| `syncing` | status | Syncing | Source has an active sync job. | backend/frontend derived | active | wait | info | yes for API |
| `error` | status | Error | Latest source job or catalog check failed. | backend/frontend derived | terminal | inspect_error/retry | danger | yes for API |
| `unavailable` | status | Unavailable | Source cannot be used in current context. | backend/frontend derived | terminal | configure/inspect_error | warning | yes for API |
| `needs_account` | status | Needs account | Source requires an account that is not ready. | frontend derived | terminal | login | warning | no |
| `ready` | project status | Ready | Project has sources and is not running. | frontend derived | idle | none | success | no |
| `running` | project status | Running | Project has queued/running analysis. | frontend derived | active | wait | info | no |
| `needs_attention` | project status | Needs attention | Project requires user attention. | frontend derived | derived | inspect_error | warning | no |
| `empty` | project status | Empty | Project has no connected sources. | frontend derived | idle | configure | neutral | no |
| `connected` | connection status | Connected | Source is connected to project. | frontend derived | idle | none | success | no |

## Providers and source taxonomy

Representative sources:

- `src/lib/types/library-sources.ts`
- `src/lib/types/sources.ts`
- `src/lib/types/analysis.ts`

Provider and source type values:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `telegram` | provider/source type | Telegram | Telegram source/provider. | backend/API | taxonomy | yes | sources, library, analysis |
| `youtube` | provider/source type | YouTube | YouTube source/provider. | backend/API | taxonomy | yes | sources, library, analysis |
| `rss` | provider/source type | RSS | RSS source/provider. | backend/API | taxonomy | yes | sources, library, analysis |
| `forum` | provider/source type | Forum | Forum source/provider. | backend/API | taxonomy | yes | sources, library, analysis |
| `web` | provider | Web | Web source/provider. | library API | taxonomy | yes | library catalog |
| `other` | provider | Other | Fallback provider bucket. | library API | taxonomy | yes | library catalog |

Source subtype values:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? |
| --- | --- | --- | --- | --- | --- | --- |
| `channel` | source subtype | Channel | Telegram or YouTube channel. | backend/API | taxonomy | yes |
| `supergroup` | source subtype | Supergroup | Telegram supergroup. | backend/API | taxonomy | yes |
| `group` | source subtype | Group | Telegram group. | backend/API | taxonomy | yes |
| `video` | source subtype | Video | YouTube video. | backend/API | taxonomy | yes |
| `playlist` | source subtype | Playlist | YouTube playlist. | backend/API | taxonomy | yes |
| `feed` | source subtype | Feed | RSS feed. | backend/API | taxonomy | yes |
| `thread` | source subtype | Thread | Forum thread. | backend/API | taxonomy | yes |
| `board` | source subtype | Board | Forum board. | backend/API | taxonomy | yes |
| `site` | source subtype | Site | Web site. | backend/API | taxonomy | yes |

Other source taxonomy:

| Value set | Type | Values | Source |
| --- | --- | --- | --- |
| Telegram history scope | mode | `current`, `migrated`, `merged` | `src/lib/types/sources.ts` |
| Telegram item history scope | mode | `current`, `migrated` | `src/lib/types/sources.ts` |
| Initial sync mode | mode | `recent_messages`, `recent_days` | `src/lib/types/sources.ts` |
| Source content label | taxonomy | `messages`, `videos`, `posts`, `items` | `src/lib/types/sources.ts` |
| Forum topic filter kind | filter | `topic`, `uncategorized` | `src/lib/types/sources.ts` |

## YouTube availability and content sync

Representative sources:

- `src/lib/types/sources.ts`
- `src/lib/types/youtube.ts`

Availability values:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `available` | availability | Available | YouTube content is available. | backend/API | terminal | none | success/neutral | yes |
| `upcoming` | availability | Upcoming | Video is scheduled for the future. | backend/API | terminal | wait | info | yes |
| `live_now` | availability | Live now | Video is currently live. | backend/API | active | wait | info | yes |
| `live_ended_transcript_pending` | availability | Live ended, transcript pending | Live stream ended but transcript is not ready. | backend/API | transitional | wait/retry | warning | yes |
| `no_captions` | availability | No captions | Captions/transcript are not available. | backend/API | terminal | inspect_error | warning | yes |
| `private_or_auth_required` | availability | Private or auth required | Auth or permissions are required. | backend/API | terminal | login | warning | yes |
| `members_only` | availability | Members only | Members-only content. | backend/API | terminal | login | warning | yes |
| `age_restricted` | availability | Age restricted | Age-gated content. | backend/API | terminal | login | warning | yes |
| `geo_blocked` | availability | Geo blocked | Content is blocked by region. | backend/API | terminal | inspect_error | warning | yes |
| `deleted` | availability | Deleted | Content was deleted. | backend/API | terminal | none | neutral | yes |
| `removed_from_playlist` | availability | Removed from playlist | Playlist item was removed. | backend/API | terminal | none | neutral | yes |
| `unavailable_unknown` | availability | Unavailable, unknown reason | Content is unavailable for an unknown reason. | backend/API | terminal | inspect_error/retry | warning | yes |

Content sync states:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `not_synced` | state | Not synced | Content has not been synced. | backend/API | idle | retry | neutral | yes |
| `synced` | state | Synced | Content is synced locally. | backend/API | terminal | none | success | yes |
| `unavailable` | state | Unavailable | Content cannot be synced. | backend/API | terminal | inspect_error | warning | yes |
| `failed` | state | Failed | Last sync failed. | backend/API | terminal | retry/inspect_error | danger | yes |
| `unknown` | state | Unknown | Sync state cannot be classified. | backend/API | derived | retry | neutral | yes |

## Topic resolution and migrated history

Representative source: `src/lib/types/sources.ts`.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `never_run` | status | Never run | Topic resolver has not run. | backend/db | idle | wait/retry | neutral | yes |
| `ready` | status | Ready | Topic resolver output is current. | backend/db | terminal | none | success | yes |
| `dirty` | status | Dirty | Topic resolver output needs rebuild. | backend/db | transitional | wait/retry | warning | yes |
| `rebuilding` | status | Rebuilding | Topic resolver is rebuilding output. | backend/db | active | wait | info | yes |
| `failed` | status | Failed | Topic resolver failed. | backend/db | terminal | retry/inspect_error | danger | yes |
| `none` | migrated history status | None | No migrated Telegram history detected. | backend/API | terminal | none | neutral | yes |
| `available` | migrated history status | Available | Migrated Telegram history is available. | backend/API | terminal | none/import | success | yes |
| `unavailable` | migrated history status | Unavailable | Migrated history is not available. | backend/API | terminal | none | neutral | yes |

## Archive/readiness and ingest batches

Representative backend sources:

- `src-tauri/src/archive_read_model.rs`
- `src-tauri/src/readiness.rs`
- `src-tauri/src/ingest_provenance.rs`
- `src-tauri/src/sources/items.rs`

These values are visible in backend diagnostics and export selection logic. A
dedicated frontend type does not yet exist for all of them.

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | UI tone | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `never_built` | status | Never built | Read model has never been built. | backend/db | idle | wait/retry | neutral | yes |
| `building` | status | Building | Read model is being built. | backend/db | active | wait | info | yes |
| `ready` | status | Ready | Read model is current and usable. | backend/db | terminal | none | success | yes |
| `stale` | status | Stale | Read model exists but is out of date. | backend/db | transitional | wait/retry | warning | yes |
| `failed` | status | Failed | Read model build failed. | backend/db | terminal | retry/inspect_error | danger | yes |
| `completed` | status | Completed | Ingest batch completed. | backend/db | terminal | none | success | yes |
| `failed` | status | Failed | Ingest batch failed. | backend/db | terminal | retry/inspect_error | danger | yes |
| `cancelled` | status | Cancelled | Ingest batch was cancelled. | backend/db | terminal | retry | neutral | yes |
| `complete` | completeness | Complete | Ingest batch observed complete data. | backend/db | derived | none | success | yes |
| `partial` | completeness | Partial | Ingest batch observed partial data. | backend/db | derived | inspect_error | warning | yes |
| `unknown` | completeness | Unknown | Ingest completeness is not classified. | backend/db | derived | inspect_error | neutral | yes |
| `inserted` | observation outcome | Inserted | Ingest observation inserted a new item. | backend/db | taxonomy | none | success | yes |
| `duplicate_observed` | observation outcome | Duplicate observed | Ingest observation matched an already-known item. | backend/db | taxonomy | none | neutral | yes |
| `skipped` | observation outcome | Skipped | Ingest observation skipped an item. | backend/db | taxonomy | inspect_error | warning | yes |
| `empty_payload` | reason code | Empty payload | Item was skipped because payload had no usable content. | backend/db | taxonomy | inspect_error | warning | yes |
| `conflict_without_item_id` | reason code | Conflict without item id | Insert conflict occurred but no existing item id could be resolved. | backend/db | taxonomy | inspect_error | warning | yes |

Archive/readiness fallback reasons:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `MissingState` | fallback reason | Missing state | Archive read-model state row is absent. | backend internal | derived | no | NotebookLM export fallback |
| `NeverBuilt` | fallback reason | Never built | Archive read model was never built. | backend internal | derived | no | NotebookLM export fallback |
| `Building` | fallback reason | Building | Archive read model is currently building. | backend internal | active | no | NotebookLM export fallback |
| `Stale` | fallback reason | Stale | Archive read model is stale. | backend internal | derived | no | NotebookLM export fallback |
| `Failed` | fallback reason | Failed | Archive read model build failed. | backend internal | terminal | no | NotebookLM export fallback |
| `OldModelVersion` | fallback reason | Old model version | Archive read model was built with an older model version. | backend internal | derived | no | NotebookLM export fallback |

## Backend-only scheduler and maintenance values

Representative backend sources:

- `src-tauri/src/llm/scheduler.rs`
- `src-tauri/src/llm/mod.rs`
- `src-tauri/src/migrations/baseline_reset.rs`
- `src-tauri/src/sources/legacy_metadata_cleanup.rs`
- `src-tauri/src/sources/items.rs`

LLM scheduler values:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `provider_test` | request kind | Provider test | LLM request tests a provider/profile. | backend scheduler | taxonomy | yes, serialized snapshot | diagnostics/scheduler |
| `analysis_chat` | request kind | Analysis chat | LLM request serves follow-up chat. | backend scheduler | taxonomy | yes, serialized snapshot | diagnostics/scheduler |
| `analysis_report_map` | request kind | Analysis report map | LLM request analyzes one report chunk. | backend scheduler | taxonomy | yes, serialized snapshot | diagnostics/scheduler |
| `analysis_report_reduce` | request kind | Analysis report reduce | LLM request writes final report. | backend scheduler | taxonomy | yes, serialized snapshot | diagnostics/scheduler |
| `prompt_pack_stage` | request kind | Prompt-pack stage | LLM request executes a prompt-pack stage. | backend scheduler | taxonomy | yes, serialized snapshot | diagnostics/scheduler |
| `queued` | scheduler state | Queued | Request is waiting in scheduler queue. | backend scheduler | active | yes, serialized snapshot | diagnostics/scheduler |
| `running` | scheduler state | Running | Request is currently executing. | backend scheduler | active | yes, serialized snapshot | diagnostics/scheduler |
| `gemini` | provider kind | Gemini | Native Gemini provider. | backend LLM config | taxonomy | yes | provider config |
| `openai_compatible` | provider kind | OpenAI compatible | OpenAI-compatible provider. | backend LLM config | taxonomy | yes | provider config |

Internal maintenance values:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `BaselineReady` | migration state | Baseline ready | Database already matches baseline migration history. | backend internal | terminal | no | migration startup |
| `OldHistoryReadyForCutover` | migration state | Old history ready for cutover | Old migration history can be collapsed to baseline. | backend internal | transitional | no | migration startup |
| `Audit` | maintenance mode | Audit | Inspect legacy Telegram metadata without clearing. | backend internal | taxonomy | no | legacy cleanup |
| `Clear` | maintenance mode | Clear | Clear legacy Telegram metadata candidates. | backend internal | taxonomy | no | legacy cleanup |
| `MaintainSingleWrite` | maintenance mode | Maintain single write | Update archive read model as part of one item write. | backend internal | taxonomy | no | source item insert |
| `MarkSourceStaleOnly` | maintenance mode | Mark source stale only | Mark archive read model stale without rebuilding immediately. | backend internal | taxonomy | no | source item insert |
| `Skip` | maintenance mode | Skip | Skip archive read-model maintenance for this write. | backend internal | taxonomy | no | source item insert |

## UI modes, tabs, and filters

Representative sources:

- `src/lib/analysis-workspace-state.ts`
- `src/lib/analysis-run-workflow.ts`
- `src/lib/analysis-run-companion-state.ts`
- `src/lib/source-browser-model.ts`
- `src/lib/gemini-browser-run-inspector.ts`

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? | Current usage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `report` | mode | Report canvas | Analysis workspace shows report canvas. | frontend persisted UI | filter/presentation | yes, persisted | workspace canvas |
| `source` | mode | Source canvas | Analysis workspace shows source browser. | frontend persisted UI | filter/presentation | yes, persisted | workspace canvas |
| `active` | inspector mode | Active | Inspect active analysis run. | frontend UI | filter | no | run inspector |
| `history` | inspector mode | History | Inspect saved run history. | frontend UI | filter | no | run inspector |
| `trace` | inspector mode | Trace | Inspect trace refs. | frontend UI | filter | no | run inspector |
| `chunks` | inspector mode | Chunks | Inspect chunk summaries. | frontend UI | filter | no | run inspector |
| `runs` | tab | Runs | Run companion shows run list. | frontend persisted UI | filter/presentation | yes, persisted | run companion |
| `evidence` | tab | Evidence | Run companion shows evidence. | frontend persisted UI | filter/presentation | yes, persisted | run companion |
| `chat` | tab | Chat | Run companion shows chat. | frontend persisted UI | filter/presentation | yes, persisted | run companion |
| `current` | filter | Current scope | Filter runs to current source/project scope. | frontend persisted UI | filter | yes, persisted | run companion |
| `all` | filter | All scopes | Filter across all scopes. | frontend persisted UI | filter | yes, persisted | run companion |
| `light` | mode | Light refresh | Gemini Browser light refresh. | frontend scheduler | taxonomy | no | Gemini refresh scheduler |
| `full` | mode | Full refresh | Gemini Browser full refresh. | frontend scheduler | taxonomy | no | Gemini refresh scheduler |

## UI presentation values

Representative sources:

- `src/lib/components/ui/types.ts`
- `src/lib/components/ui/StatusMessage.svelte`
- `src/lib/components/ui/Button.svelte`
- `src/lib/components/ui/badge/badge.svelte`

Badge variants:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? |
| --- | --- | --- | --- | --- | --- | --- |
| `default` | tone | Default | Default badge appearance. | design system | presentation | no |
| `warning` | tone | Warning | Caution or degraded state. | design system | presentation | no |
| `member` | tone | Member | Membership-specific badge. | design system | presentation | no |
| `info` | tone | Info | Informational or active state. | design system | presentation | no |
| `success` | tone | Success | Positive terminal state. | design system | presentation | no |
| `danger` | tone | Danger | Error or destructive state. | design system | presentation | no |
| `neutral` | tone | Neutral | Low-emphasis state. | design system | presentation | no |

Status message tones:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | Stable? |
| --- | --- | --- | --- | --- | --- | --- |
| `default` | tone | Default | Normal status text. | design system | presentation | no |
| `error` | tone | Error | Error status text. | design system | presentation | no |
| `info` | tone | Info | Informational status text. | design system | presentation | no |
| `muted` | tone | Muted | Low-emphasis status text. | design system | presentation | no |

## Error and reason codes

Representative sources:

- `src/lib/app-error.ts`
- `src/lib/types/gemini-browser.ts`
- `src/lib/types/prompt-packs.ts`
- `src/lib/types/sources.ts`

Application error kinds:

| Value | Type | Name | Meaning | Source of truth | Lifecycle | User action | Stable? |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `validation` | error kind | Validation | User input or request validation failed. | backend/frontend error contract | terminal | configure | yes |
| `not_found` | error kind | Not found | Requested entity was not found. | backend/frontend error contract | terminal | inspect_error | yes |
| `auth` | error kind | Auth | Authentication or authorization failed. | backend/frontend error contract | terminal | login | yes |
| `network` | error kind | Network | Network or upstream request failed. | backend/frontend error contract | terminal | retry | yes |
| `conflict` | error kind | Conflict | Request conflicts with current state. | backend/frontend error contract | terminal | inspect_error | yes |
| `internal` | error kind | Internal | Unexpected internal error. | backend/frontend error contract | terminal | inspect_error | yes |

Known reason-code families:

| Value set | Type | Values | Source |
| --- | --- | --- | --- |
| Gemini manual actions | reason/action | `login`, `account_picker`, `consent`, `captcha`, `unknown_modal`, `start_chrome_cdp` | `src/lib/types/gemini-browser.ts` |
| Gemini candidate rejection | reason | `baseline`, `composer`, `prompt_container`, `navigation`, `account_or_login`, `controls`, `multi_turn`, `not_visible`, `empty`, `lower_score` | `src/lib/types/gemini-browser.ts` |
| YouTube availability reasons | availability/reason | See YouTube availability section. | `src/lib/types/sources.ts` |
| Prompt-pack validation severity/code | severity/code | Severity is constrained to `info`, `warning`, `error`; `code` remains a string and should not be treated as an enum without a separate inventory. | `src/lib/types/prompt-packs.ts` |
| Takeout warning codes | warning code | Currently typed as `string[]`; needs backend inventory. | `src/lib/types/sources.ts` |

## Open follow-up inventory

These areas are intentionally listed as follow-up work rather than guessed:

- Database-only strings in migrations and SQL constraints.
- UI-only status text stored in local `$state("")` variables.
- Prompt-pack seed-only `origin_kind` and `lifecycle_status` values beyond the
  observed `user` and `active`.
- Diagnostic count dimensions such as `errorKind`, `warningState`,
  `completeness`, and `buildMode`.
- Button variants and Extractum-specific UI variants outside the shared UI
  badge/status-message vocabulary.

## Promotion path to code registry

Recommended order for turning this document into runtime code:

1. Promote presentation-safe values first: badge variants, status tones, labels.
2. Promote frontend-derived view statuses: project, library source, snapshot
   affordance.
3. Promote persisted filters and modes only after checking local storage and URL
   compatibility.
4. Promote backend/db statuses last, with tests around wire values and migration
   compatibility.

Do not rename machine values during promotion unless there is a migration plan.

## Frontend/UI value inventory

Scope: frontend-local state, filters, tabs, badges, and view-model values. Most rows are `internal frontend` unless the note says they are persisted or mirror backend/API values.

### Shared UI and diagnostics

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| UI badges | `BadgeVariant` | `default`, `warning`, `member`, `info`, `success`, `danger`, `neutral` | `src/lib/components/ui/types.ts` | Shared presentation variants. |
| Toasts | `ToastKind` | `error`, `success`, `info` | `src/lib/toasts.ts` | User notification tone. |
| App errors | `AppErrorKind` | `validation`, `not_found`, `auth`, `network`, `conflict`, `internal` | `src/lib/app-error.ts` | Frontend error contract used by diagnostics formatting. |
| YouTube thumbnail resolver result | `kind` | `success`, `terminal_error`, `transient_error` | `src-tauri/src/youtube/thumbnail.rs` | Non-persistent IPC result. `terminal_error` is safe to memoize for the process; `transient_error` retries on a later component mount. |
| Diagnostics tone classifier | success bucket | `available`, `current`, `synced`, `ready`, `succeeded`, `completed`, `complete`, `none` | `src/lib/diagnostics-view-model.ts` | Classifier-only values, not a canonical backend enum. |
| Diagnostics tone classifier | info bucket | `pending`, `queued`, `running`, `cancel_requested`, `partial`, `present` | `src/lib/diagnostics-view-model.ts` | Classifier-only values. |
| Diagnostics tone classifier | warning bucket | `never_synced`, `missing_key`, `not_configured`, `unavailable`, `not_found`, `timed_out`, `cancelled` | `src/lib/diagnostics-view-model.ts` | Classifier-only values. |
| Diagnostics tone classifier | danger bucket | `failed`, `check_failed`, `error`, `internal`, `network`, `auth`, `validation` | `src/lib/diagnostics-view-model.ts` | Classifier-only values. |
| Diagnostics build mode tone | build mode values | `release`, `debug` | `src/lib/diagnostics-view-model.ts` | Recognized display values for build mode. |

### Analysis workspace and report UI

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| Workspace selection | `WorkspaceSelection.kind` | `source`, `source_group`, `none` | `src/lib/analysis-workspace-state.ts` | Frontend selection state. |
| Open run | `OpenRunState.kind` | `none`, `active`, `saved` | `src/lib/analysis-workspace-state.ts` | Frontend open-run state. |
| Canvas | `CanvasMode` | `report`, `source` | `src/lib/analysis-workspace-state.ts` | Main workspace surface. |
| Source basis | `SourceViewBasis` | `live_source`, `run_snapshot` | `src/lib/analysis-workspace-state.ts` | May be persisted in workspace UI state. |
| Companion panel | `CompanionTab` | `evidence`, `chat`, `chunks`, `runs` | `src/lib/analysis-workspace-state.ts` | Report companion tabs. |
| Legacy scope | `LegacyAnalysisScope` | `single_source`, `source_group` | `src/lib/analysis-workspace-state.ts` | Compatibility state. |
| Report launch scope | `AnalysisReportStartState.analysisScope` | `single_source`, `source_group` | `src/lib/analysis-state.ts` | Launch form state. |
| Run list filter | `AnalysisRunFilter` | `all`, `completed`, `failed` | `src/lib/analysis-state.ts` | Saved/active run filter. |
| Trace ref origin | `AnalysisTraceRefOrigin` | `saved`, `resolved`, `unknown` | `src/lib/analysis-state.ts` | Evidence reference provenance in UI. |
| NotebookLM export range | `NotebookLmExportFormState.range` | `entire_history`, `analysis_period` | `src/lib/analysis-state.ts` | Export form state. |
| NotebookLM export scope | `NotebookLmExportRequestScope.kind` | `source`, `source_group` | `src/lib/analysis-state.ts` | Request construction state. |
| Analysis source type | `AnalysisGroupSourceType` | `telegram`, `youtube` | `src/lib/types/analysis.ts` | Analysis group source type. |
| Analysis source option type | `AnalysisSourceOptionType` | `telegram`, `youtube`, `rss`, `forum` | `src/lib/types/analysis.ts` | Analysis source selector type. |
| Prompt template kind | `AnalysisPromptTemplateKind` | `report`, `chat` | `src/lib/types/analysis.ts` | Prompt template category. |
| Snapshot state | `AnalysisSnapshotState` | `captured`, `capture_failed` | `src/lib/types/analysis.ts` | Mirrors run snapshot state in frontend types. |
| YouTube corpus mode | `YoutubeCorpusMode` | `transcript_only`, `transcript_description`, `transcript_description_comments` | `src/lib/types/analysis.ts` | Report input mode. |

### Companion, evidence, and source browsing

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| Chat availability | `ChatAvailabilityReason` | `enabled`, `no_run`, `pending_completion`, `terminal_run`, `checking_snapshot`, `missing_snapshot`, `missing_report`, `capture_failed_with_error`, `not_captured_before_terminal`, `capture_failed_without_error_unknown`, `inconsistent`, `verification_failed`, `unknown_snapshot` | `src/lib/analysis-run-companion-state.ts` | User-facing chat gate reason. |
| Evidence source action | `EvidenceSourceActionDecision.kind` | `run_snapshot`, `live_source`, `unavailable` | `src/lib/analysis-run-companion-state.ts` | Action decision for opening evidence in source view. |
| Companion run status filter | `CompanionRunStatusFilter` | `all`, `completed`, `failed`, `cancelled`, `queued_running` | `src/lib/analysis-run-companion-state.ts` | Run filter UI. |
| Companion run scope filter | `CompanionRunsFilterState.scope` | `all`, `current` | `src/lib/analysis-run-companion-state.ts` | Run filter UI. |
| Companion run entry | `CompanionRunEntry.kind` | `active`, `saved` | `src/lib/analysis-run-companion-state.ts` | Combines active and saved runs. |
| Evidence view basis | `EvidenceSourceViewBasis` | `run_snapshot`, `live_source` | `src/lib/analysis-evidence-source-navigation.ts` | Evidence navigation basis. |
| Evidence scope | `EvidenceSourceScope.kind` | `source`, `group_member` | `src/lib/analysis-evidence-source-navigation.ts` | Evidence source scope. |
| Source return context | `SourceReturnContext.kind` | `evidence` | `src/lib/analysis-evidence-source-navigation.ts` | Return-to-evidence marker. |
| Focused live target | `FocusedLiveSourceTarget.kind` | `source_item`, `youtube_transcript`, `unsupported` | `src/lib/analysis-evidence-source-navigation.ts` | Live source focus target. |
| Loaded evidence data | `LoadedEvidenceSourceData.kind` | `snapshot`, `source_items`, `youtube_transcript` | `src/lib/analysis-evidence-source-navigation.ts` | Loaded data shape for evidence navigation. |
| Source browser tabs | `SourceBrowserTabId` | `timeline`, `transcript`, `comments`, `videos`, `sources`, `items`, `metadata`, `activity` | `src/lib/source-browser-model.ts` | Tab ids. |
| Source browser subject | `SourceBrowserSubject.kind` | `source`, `source_group`, `run_snapshot` | `src/lib/source-browser-model.ts` | Browser subject shape. |
| Run snapshot browser kind | `RunSnapshotBrowserKind` | `source_group`, `telegram_timeline`, `youtube_transcript`, `generic_items` | `src/lib/source-browser-model.ts` | Snapshot browser specialization. |
| Loaded item sort | `LoadedSourceItemSort` | `newest`, `oldest` | `src/lib/source-browser-model.ts` | Source item sort. |
| Loaded YouTube comment sort | `LoadedYoutubeCommentSort` | `newest`, `oldest`, `most_liked` | `src/lib/source-browser-model.ts` | Comment sort. |
| Comments coverage | `CommentsCoverageState` | `unknown`, `not_synced`, `syncing`, `failed`, `synced_empty`, `synced_with_rows` | `src/lib/source-browser-model.ts` | Derived YouTube comments coverage. |
| Telegram history scope UI | history scope option values | `current`, `migrated`, `merged` | `src/lib/components/analysis/source-browser-shell.svelte` | UI selector values; check backend support before reusing. |
| Source reader basis | `SourceReaderBasis` | `live_source`, `run_snapshot` | `src/lib/source-reader-model.ts` | Reader source basis. |
| Source reader kind | `SourceReaderKind` | `telegram_message`, `youtube_transcript`, `youtube_comment`, `youtube_description`, `generic_item` | `src/lib/source-reader-model.ts` | Reader item kind. |
| Source reader history scope | `SourceReaderItem.historyScope` | `current`, `migrated` | `src/lib/source-reader-model.ts` | Reader item history scope. |
| Universal item kind sentinel | `ALL_KINDS` | `__all_source_item_kinds__` | `src/lib/components/analysis/universal-items-view.svelte` | Local filter sentinel. |
| Known source item kinds | `KNOWN_ITEM_KINDS` | `telegram_message`, `youtube_transcript`, `youtube_comment`, `youtube_description` | `src/lib/components/analysis/universal-items-view.svelte` | Known UI labels; unknown values are allowed and displayed as unknown. |

### Snapshot affordance and canvas state

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| Snapshot affordance surface | `SnapshotAffordanceSurface` | `runs-row`, `opened-header`, `run-details`, `source-tab`, `evidence-tab`, `chat-tab` | `src/lib/analysis-run-snapshot-affordance.ts` | Caller surface for snapshot messaging. |
| Snapshot affordance state | `SnapshotAffordanceState` | `available`, `capture_failed_with_error`, `not_captured_before_terminal`, `capture_failed_without_error_unknown`, `inconsistent`, `verification_failed`, `checking`, `pending`, `unknown` | `src/lib/analysis-run-snapshot-affordance.ts` | Derived snapshot UI state. |
| Snapshot affordance severity | `SnapshotAffordanceSeverity` | `none`, `info`, `warning`, `error` | `src/lib/analysis-run-snapshot-affordance.ts` | UI severity. |
| Snapshot probe state | `SnapshotProbeState` | `available`, `unavailable`, `error`, `loading`, `unknown` | `src/lib/analysis-run-snapshot-affordance.ts` | Probe/load state. |
| Snapshot badge variant | `SnapshotBadgeVariant` | `neutral`, `info`, `warning`, `danger` | `src/lib/analysis-run-snapshot-affordance.ts` | Local badge subset. |
| Snapshot availability signal | `SnapshotAvailabilitySignal` | `unknown`, `capturing`, `available`, `unavailable` | `src/lib/analysis-run-snapshot-affordance.ts` | Local alias aligned with canvas availability. |
| Run snapshot availability | `RunSnapshotAvailability` | `unknown`, `capturing`, `available`, `unavailable` | `src/lib/analysis-report-canvas-state.ts` | Canvas-level snapshot state. |
| Source canvas surface | `SourceCanvasSurface` | `live_source`, `run_snapshot_unknown`, `run_snapshot_pending`, `run_snapshot_available`, `run_snapshot_unavailable` | `src/lib/analysis-report-canvas-state.ts` | Derived canvas surface. |

### Library, projects, and source import UI

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| Library provider | `LibrarySourceProvider` | `telegram`, `youtube`, `rss`, `forum`, `web`, `other` | `src/lib/types/library-sources.ts` | Library catalog provider values. |
| Library source subtype | `LibrarySourceSubtype` | `video`, `playlist`, `channel`, `supergroup`, `group`, `feed`, `thread`, `board`, `site`, `null` | `src/lib/types/library-sources.ts` | Library catalog subtype values. |
| Library catalog status | `LibraryCatalogStatus` | `active`, `syncing`, `error`, `unavailable` | `src-tauri/src/library_sources/models.rs`, `src/lib/types/library-sources.ts` | Catalog record status shared by backend API and frontend types. |
| Library catalog source status | `LibraryCatalogSourceStatus` | `active`, `syncing`, `error`, `unavailable` | `src/lib/ui/library-catalog-model.ts` | View-model status. |
| Library catalog filter id | `LibraryCatalogFilterId` | `all`, `provider:<provider>`, `provider:<provider>/subtype:<subtype>` | `src/lib/ui/library-catalog-model.ts` | Structured string id. |
| Research project status | `ProjectStatus` | `ready`, `running`, `needs_attention`, `empty` | `src-tauri/src/projects/read_model.rs`, `src/lib/types/projects.ts`, `src/lib/ui/research-projects-model.ts` | Backend-derived API status for `ProjectSummary`; UI view-model mirrors the same values until UI integration imports the backend type. |
| Research project backing | `ResearchProjectBacking.kind` | `project`, `source_group` | `src/lib/ui/research-projects-model.ts` | View-model backing kind. |
| Project YouTube video Library delete outcome | `DeleteProjectYoutubeVideoSourceOutcome.status` | `deleted`, `blocked_by_other_projects` | `src-tauri/src/projects/mod.rs`, `src/lib/types/projects.ts` | Tauri API response for project-scoped Library deletion. `deleted` is terminal success; `blocked_by_other_projects` is an expected non-error result when other projects still reference the source. |
| Library source status | `LibrarySourceStatus` | `active`, `needs_account`, `syncing`, `error`, `unavailable` | `src/lib/ui/research-projects-model.ts` | Project settings/library attach status. |
| Project source link status | `ProjectSourceLinkView.connectionStatus` | `connected` | `src/lib/ui/research-projects-model.ts` | Current UI has only connected links. |
| Library source filter | `LibraryFilterState.providers` | `telegram`, `youtube`, `rss`, `forum`, `web`, `other` | `src/lib/ui/research-projects-model.ts` | Uses `LibrarySourceProvider`. |
| YouTube smart import provider | `YoutubeSmartImportProvider` | `youtube`, `telegram`, `unknown` | `src/lib/ui/library-add-source-model.ts` | URL classifier provider. |
| YouTube smart import kind | `YoutubeSmartImportKind` | `video`, `playlist`, `channel`, `unsupported` | `src/lib/ui/library-add-source-model.ts` | URL classifier kind. |
| Playlist import item result | `PlaylistImportItemResult.status` | `added`, `skipped`, `failed` | `src/lib/ui/library-add-source-model.ts` | Per-item import result. |

### Gemini Browser frontend UI

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| Gemini Browser provider status | `GeminiBrowserProviderStatusKind` | `not_started`, `ready`, `needs_login`, `needs_manual_action`, `running`, `stopped`, `failed` | `src/lib/types/gemini-browser.ts` | Frontend provider status type. |
| Gemini Browser provider mode | `GeminiBrowserProviderMode` | `managed`, `cdp_attach` | `src/lib/types/gemini-browser.ts` | `cdp_attach` can be persisted in localStorage. |
| Gemini Browser run status | `GeminiBrowserRunStatus` | `queued`, `running`, `ok`, `ready`, `needs_login`, `needs_manual_action`, `blocked`, `timeout`, `browser_crashed`, `failed`, `cancelled` | `src/lib/types/gemini-browser.ts` | Run/result status type. |
| Gemini Browser refresh mode | `GeminiBrowserRefreshMode` | `light`, `full` | `src/lib/gemini-browser-refresh-scheduler.ts` | Polling/refresh mode. |
| Gemini Browser setup check state | `GeminiBrowserSetupCheckState` | `ready`, `action_needed`, `running`, `warning`, `failed`, `unknown`, `not_applicable` | `src/lib/gemini-browser-setup-status.ts` | Setup checklist state. |
| Gemini Browser run history filter | `GeminiBrowserRunHistoryFilter` | `all`, `problems`, `partial_risk`, `manual_action`, `failed` | `src/lib/gemini-browser-run-inspector.ts` | Run history filter. |
| Gemini Browser run history badge | `GeminiBrowserRunHistoryBadge` | `ok`, `stable`, `partial`, `manual`, `failed`, `running`, `queued` | `src/lib/gemini-browser-run-inspector.ts` | Derived row badge. |

### LLM stream frontend values

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| LLM stream event | `LlmStreamEventKind` | `queued`, `started`, `delta`, `completed`, `failed`, `cancelled` | `src/lib/types/llm.ts` | Streaming UI/API event kind. |
## Source-of-truth map for repeated values

This section clarifies ownership for values that intentionally appear in more than one layer. Use the canonical owner when adding or changing values; treat mirrors as consumers that must be updated after the owner changes.

| Value family | Canonical owner | Mirrors and derived users | Rule |
| --- | --- | --- | --- |
| Source provider/type values: `telegram`, `youtube`, `rss`, `forum` | Backend source model and API payloads | `SourceType`, `AnalysisSourceOptionType`, library/project view-model filters | Add new source providers in backend/API first, then mirror in TS types and UI filters. |
| Library-only provider values: `web`, `other` | Library catalog API contract | `LibrarySourceProvider`, catalog filters, project settings library filters | Do not back-port to analysis/source types unless the backend supports them as real analysis sources. |
| Source subtype values: `video`, `playlist`, `channel`, `supergroup`, `group`, `feed`, `thread`, `board`, `site` | Backend source/library records | `SourceSubtype`, `LibrarySourceSubtype`, catalog filter ids, source reader/browser labels | Keep subtype spelling stable because ids such as `provider:<provider>/subtype:<subtype>` depend on it. |
| Analysis run statuses: `queued`, `running`, `completed`, `failed`, `cancelled` | Backend analysis run lifecycle and persisted run records | `AnalysisRunFilter`, companion filters, workspace active/saved derivation, snapshot affordance gates | Frontend may group values, but must not introduce new canonical run statuses. |
| Generic job statuses: `queued`, `running`, `succeeded`, `failed`, `cancel_requested`, `cancelled` | Backend job table/API payloads | Source activity UI, diagnostics tone classifier | UI can classify status tone, but backend owns the lifecycle values. |
| Takeout import statuses and recovery kinds | Backend takeout import/recovery records | Source activity UI and recovery explanations | Add backend value, then add user-facing title/body/severity in frontend if the value is visible. |
| Prompt-pack run statuses, event kinds, and phases | Backend prompt-pack runtime/API payloads | Frontend prompt-pack types and run display | Frontend types are mirrors. Update both TS types and backend event emission together. |
| Gemini Browser provider/run statuses | Gemini Browser backend bridge and run log payloads | `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, setup checklist, run inspector filters/badges | Backend/bridge owns status values; frontend owns derived filters and badges. |
| Gemini Browser provider mode: `managed`, `cdp_attach` | Frontend provider config plus backend bridge request shape | LocalStorage setting, settings panel, bridge API request | Treat as a shared UI/API contract because persisted localStorage values must remain readable. |
| LLM provider kind: `gemini`, `openai_compatible` | Backend `ProviderKind::as_str()` / `parse()` | Frontend LLM profile API types and settings UI | `openai_compatible` is canonical; `omniroute` is a legacy accepted alias only. |
| LLM stream event kinds | LLM streaming API payloads | Frontend stream handling | Add stream events only when both backend emitter and frontend consumer understand them. |
| Snapshot state: `captured`, `capture_failed` | Backend analysis run snapshot fields | `AnalysisSnapshotState`, snapshot affordance derivation | Persisted snapshot state stays small; UI-specific states belong in affordance values. |
| Snapshot affordance/probe/canvas values | Frontend snapshot UI model | Chat availability, evidence source action, source canvas labels | These are derived UI states. Do not persist them unless a backend contract is explicitly added. |
| Badge/status tone values | Frontend shared UI components | Diagnostics, snapshot affordance, status messages | Presentation-only. Do not treat as backend/API status values. |
| `active`, `syncing`, `error`, `unavailable` library statuses | Backend library catalog status payloads | Catalog/project view-model statuses, diagnostics classifier | Backend owns catalog record status. Project-level status values are separate. |
| Project status: `ready`, `running`, `needs_attention`, `empty` | Frontend project view-model | Project cards/settings UI | Derived UI status, not a persisted project lifecycle. |
| Import/classifier sentinels such as `all`, `__all_source_item_kinds__`, `__all_topics__` | Frontend component/model that declares the sentinel | Select filters and local state | Keep local unless the sentinel crosses process/API boundaries. |

When in doubt, prefer these categories:

| Category | Examples | Change policy |
| --- | --- | --- |
| Persisted database/API contract | run status, job status, source type, provider kind | Requires backend, frontend mirror, migration/seed/test fixture review. |
| Shared UI/API request contract | Gemini Browser provider mode, report launch scope | Requires frontend and backend request parsing review. |
| Frontend derived state | project status, snapshot affordance state, canvas surface | Can change in frontend, but update this registry and affected tests/docs. |
| Presentation-only value | badge variant, toast kind, diagnostics tone bucket | Local UI change, but avoid reusing as business status. |
| Local sentinel/filter value | `all`, `__all_topics__`, `provider:<provider>/subtype:<subtype>` | Safe only inside its declaring component/model unless documented otherwise. |
## Migration/seed/fixture value inventory

Scope: values found in SQL migrations, bundled prompt-pack assets, seed code, and fixtures. These values often bypass obvious TypeScript/Rust union definitions, so treat this section as a cross-check layer for persisted data and generated fixtures.

### Prompt-pack persisted and bundled values

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| Prompt-pack version source | `origin_kind` | `bundled`, `user` | `src-tauri/migrations/0006_prompt_pack_mvp.sql`, `src-tauri/prompt-packs/youtube_summary/1.0.0/pack.json` | Persisted. `bundled` is the built-in pack source; `user` blocks seed overwrite collisions. |
| Prompt-pack lifecycle | `lifecycle_status` | `draft`, `active`, `archived` | `src-tauri/migrations/0006_prompt_pack_mvp.sql`, `src-tauri/src/prompt_packs/seed.rs` | Persisted version lifecycle. |
| Prompt-pack schema assets | `schema_kind` | `canonical_result`, `stage_input`, `stage_output`, `pack_data_schema` | `src-tauri/migrations/0006_prompt_pack_mvp.sql`, `src-tauri/src/prompt_packs/seed.rs` | `pack_data_schema` is SQL-allowed even if current bundle seeds only the other three. |
| Prompt-pack source scope | `scope_kind` | `explicit_video`, `playlist` | `src-tauri/migrations/0006_prompt_pack_mvp.sql`, `src-tauri/src/prompt_packs/youtube_summary/snapshots.rs` | Source snapshot scope. |
| Prompt-pack source inclusion | `inclusion_status` | `included`, `skipped`, `blocking` | `src-tauri/migrations/0006_prompt_pack_mvp.sql` | Persisted source-origin inclusion outcome. |
| Prompt-pack material | `material_kind` | `transcript`, `description`, `comment` | `src-tauri/migrations/0006_prompt_pack_mvp.sql` | Material refs captured into prompt-pack snapshots. |
| Prompt-pack stage artifacts | `artifact_kind` | `prompt_input`, `raw_output`, `parsed_output`, `metrics`, `error`, `repair_input`, `intermediate_entities` | `src-tauri/migrations/0006_prompt_pack_mvp.sql`, `src-tauri/migrations/0009_prompt_pack_intermediate_entities_artifacts.sql` | `intermediate_entities` was added after the MVP migration. |
| Prompt-pack artifact redaction | `redaction_state` | `none` | `src-tauri/migrations/0006_prompt_pack_mvp.sql`, `src-tauri/migrations/0009_prompt_pack_intermediate_entities_artifacts.sql` | Default persisted value; no closed CHECK list beyond current default. |
| Prompt-pack validation severity | `severity` | `info`, `warning`, `error` | `src-tauri/migrations/0006_prompt_pack_mvp.sql` | `code` is still free-form text. |
| Prompt-pack provider family | `provider_family` | `generic_chat` | `src-tauri/prompt-packs/youtube_summary/1.0.0/stages/transcript_analysis.json`, runtime assets | Bundled asset value; not SQL-constrained. |
| Prompt-pack validator mode | `validator_mode` | `stage_output` | `src-tauri/prompt-packs/youtube_summary/1.0.0/stages/transcript_analysis.json` | Bundled stage template value; not SQL-constrained. |
| Prompt-pack control preset | `control_preset` | `standard`, `detailed_report`, `gem_analysis` | `src-tauri/prompt-packs/youtube_summary/1.0.0/pack.json`, `src-tauri/src/prompt_packs/runtime.rs`, `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte` | `standard` is default; `detailed_report` is a UI/runtime mode; `gem_analysis` is a single-video sequential multi-request mode. |
| Prompt-pack evidence mode | `evidence_mode` | `standard`, `narrative_only` | `src-tauri/prompt-packs/youtube_summary/1.0.0/pack.json`, `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs` | `narrative_only` changes validation expectations for empty videos. |

### Telegram history and source metadata values

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| Analysis Telegram history scope | `telegram_history_scope` | `current`, `current_plus_migrated` | `src-tauri/migrations/0003_analysis_telegram_history_scope.sql` | Analysis run persisted option. Distinct from source reader UI `current`/`migrated`/`merged`. |
| Telegram migrated-history domain | `migration_domain` | `migrated_from_chat` | `src-tauri/migrations/0002_migrated_history_opt_in_schema.sql` | Optional provenance domain for migrated rows. |
| Telegram history peer kind | `history_peer_kind` | `channel`, `chat`, `user` | `src-tauri/migrations/0001_current_schema_baseline.sql`, `src-tauri/migrations/0002_migrated_history_opt_in_schema.sql` | Telegram peer taxonomy from archive/takeout metadata. |
| Telegram reply peer kind | `reply_to_peer_kind` | `channel`, `chat`, `user` | `src-tauri/migrations/0001_current_schema_baseline.sql`, `src-tauri/migrations/0002_migrated_history_opt_in_schema.sql` | Optional reply peer taxonomy. |
| Telegram resolved peer kind | `resolved_peer_kind` | `channel`, `chat` | `src-tauri/migrations/0002_migrated_history_opt_in_schema.sql` | Resolved peer identity kind. |
| Telegram source peer kind | `peer_kind` | `channel`, `chat` | `src-tauri/migrations/0001_current_schema_baseline.sql` | Telegram source identity peer kind. |
| Telegram takeout history scope | `history_scope` | `unknown`, `current_history`, `current_history_with_migrated_deferred`, `partial_private_history`, `mixed_partial`, `migrated_small_group_history` | `src-tauri/migrations/0001_current_schema_baseline.sql`, `src-tauri/migrations/0002_migrated_history_opt_in_schema.sql` | Takeout/export provenance scope. `migrated_small_group_history` appears in the opt-in migration. |
| Telegram source resolution strategy | `resolution_strategy` | `username`, `dialog`, `legacy_metadata`, `unknown` | `src-tauri/migrations/0001_current_schema_baseline.sql` | Telegram source identity resolution strategy. |
| Topic membership match kind | `match_kind` | `reply_to_top_id`, `typed_root_top_message_id`, `legacy_root_external_id`, `reply_to_msg_id`, `general_fallback` | `src-tauri/migrations/0001_current_schema_baseline.sql` | Topic resolver match provenance. |

### Ingest and typed media metadata values

| Area | Name | Values | Source | Notes |
| --- | --- | --- | --- | --- |
| Ingest batch kind | `ingest_kind` | `takeout`, `sync`, `youtube_metadata`, `youtube_transcript`, `youtube_comments`, `youtube_playlist` | `src-tauri/migrations/0001_current_schema_baseline.sql` | Persisted ingest batch classifier. |
| Ingest observation item kind | `provider_item_kind` | `telegram_message` | `src-tauri/migrations/0001_current_schema_baseline.sql` | Current observation table only constrains Telegram messages. |
| Ingest observation outcome | `outcome` | `inserted`, `duplicate_observed`, `skipped`, `failed` | `src-tauri/migrations/0001_current_schema_baseline.sql` | Observation write outcome. `duplicate_observed` is also listed in the user-facing registry. |
| YouTube video form | `video_form` | `regular`, `short`, `live` | `src-tauri/migrations/0001_current_schema_baseline.sql` | Typed YouTube video metadata form. |
| Analysis document kind | `document_kind` | `telegram_message`, `youtube_transcript`, `youtube_comment`, `youtube_description` | `src-tauri/migrations/0001_current_schema_baseline.sql` | Analysis document material kind. |
## Checklist for adding or changing values

Use this checklist whenever a change introduces or renames a `status`, `state`, `kind`, `mode`, `phase`, `type`, `provider`, `subtype`, `scope`, `severity`, `reason`, or similar string value.

| Check | Question | Expected action |
| --- | --- | --- |
| Owner | Which layer owns the value? | Add or update the row in the registry and mark whether the owner is backend/db, API, shared UI/API, frontend derived, presentation-only, or local sentinel. |
| Persistence | Is the value stored in SQLite, localStorage, files, or fixtures? | Update SQL `CHECK` constraints, migrations, seed data, cleanup scripts, fixtures, and backward-compatible aliases if needed. |
| API mirror | Does the value cross the Tauri/API boundary? | Update Rust DTOs/models, TypeScript types, mapping functions, and command/request/response tests together. |
| Frontend display | Is the value visible to users? | Add labels, badges, empty/error states, filters, disabled reasons, and sorting/grouping behavior. |
| Derived values | Is this only a UI-derived grouping? | Keep it out of backend contracts unless persistence/API support is intentionally added. |
| Legacy aliases | Is an old spelling still accepted? | Document the alias as accepted legacy input, but keep one canonical output value. |
| Fixtures | Do smoke/demo/test fixtures use the value? | Update fixture rows and contract tests so seeded data continues to represent real values. |
| Docs | Does this registry already contain the family? | Extend the existing row/group instead of creating a competing duplicate. |

Recommended review phrase: `Value registry checked: owner, persistence, API mirror, UI display, fixtures.`
