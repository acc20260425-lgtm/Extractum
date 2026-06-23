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
| `open_ai_compatible` | provider kind | OpenAI compatible | OpenAI-compatible provider. | backend LLM config | taxonomy | yes | provider config |

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
| Prompt-pack validation severity/code | severity/code | Currently typed as `string`; needs inventory from persisted data and backend. | `src/lib/types/prompt-packs.ts` |
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
