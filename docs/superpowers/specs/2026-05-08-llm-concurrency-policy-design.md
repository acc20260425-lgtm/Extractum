# LLM Concurrency Policy And Analysis Preflight Design

## Goal

Make LLM report runs predictable under large local corpora by adding backend preflight limits before chunk workers and queued LLM requests are created.

This design covers both parts of the remaining Phase 4.3 work:

- document the scheduler policy that already exists;
- add hard backend caps and a preflight summary for analysis report runs.

## Current State

The LLM scheduler already isolates work by `(provider, profile)` and allows two running requests per scheduler key. Interactive requests jump ahead of background requests in the same queue. Cancellation and request snapshots already exist.

The risky part is the analysis report pipeline. `start_analysis_report` creates a run, then `run_report_pipeline` loads all eligible corpus messages with `fetch_all`, chunks the full in-memory corpus, and spawns one map task per chunk. For large Telegram archives this can create memory pressure, large local stalls, high LLM cost, and long queues before the user sees the true size of the run.

## Policy

The scheduler policy remains:

- at most `2` running LLM requests per `(provider, profile)`;
- interactive requests have priority over background requests inside a scheduler key;
- requests with different provider/profile keys may run independently;
- cancellation remains request-scoped or run-scoped.

Analysis report runs get a separate preflight policy before map tasks are spawned:

- `max_messages_per_run = 10_000`
- `max_chunks_per_run = 80`
- `max_estimated_input_chars_per_run = 1_500_000`
- `max_background_requests_per_run = 80`

`max_background_requests_per_run` intentionally matches `max_chunks_per_run` in this design because each map chunk creates one background LLM request. It stays separate in naming so a future retry or multi-background-phase design can change request budgeting without changing the user-facing concept.

## Preflight Behavior

Before inserting or starting an analysis run, the backend should compute an `AnalysisRunPreflight` for the selected scope and date range.

The preflight should report:

- selected source ids;
- eligible text message count;
- estimated input character count;
- estimated chunk count;
- configured limits;
- zero or more limit violations.

The estimated input character count should use the same rough accounting as the chunker:

```text
message.content.len()
+ message.ref.len()
+ message.author.len_or_zero()
+ 64
```

The estimated chunk count should use `ANALYSIS_CHUNK_TARGET_CHARS` and the same boundary behavior as `chunk_messages`, but without cloning or retaining the full corpus longer than necessary.

The first implementation still scans eligible rows and decompresses message text to estimate input characters. This is a known limitation: it avoids creating a run row and spawning chunk workers for oversized work, but it does not yet avoid all corpus-read cost. A future optimization can store text length metadata on `items` so preflight can use `COUNT` and `SUM` queries without decompressing every eligible message.

If no eligible text messages are found, the existing user-facing validation should remain:

```text
No synced source documents were found for the selected analysis scope and period
```

If one or more limits are exceeded, `start_analysis_report` should return a validation error before inserting an `analysis_runs` row:

```text
Analysis scope is too large: 73102 documents, 381 estimated chunks, 6200000 estimated input characters. Narrow the period or choose a smaller source scope.
```

The exact numbers should come from the preflight result. The message should mention all relevant scale dimensions but should not expose prompt contents or message text.

If the preflight passes, the run starts normally and emits an early progress event:

```text
Preflight passed: 2430 documents, 18 estimated chunks, 310000 estimated input characters.
```

## Implementation Shape

Add focused preflight code near corpus loading rather than inside the scheduler. The scheduler controls request execution; preflight controls whether an analysis run may create work.

Preferred backend boundaries:

- `analysis::corpus` owns source/message counting and estimated chunk calculation;
- `analysis::report` calls preflight before duplicate-run insertion and run creation;
- `analysis::models` does not need new public Tauri-facing types in the first implementation because preflight remains backend-owned;
- UI work is not required in the first implementation beyond surfacing the existing typed validation error and receiving the early progress event.

The first implementation should not add settings UI. Limits should be constants in backend code, documented in project docs and backlog. Configurable limits require a separate design after the defaults have real-use feedback.

## Data Flow

1. `start_analysis_report` validates basic inputs.
2. It resolves scope to source ids.
3. It runs `preflight_analysis_run(pool, source_ids, period_from, period_to)`.
4. If preflight finds zero eligible messages, return the existing no-documents validation.
5. If preflight exceeds hard caps, return a validation error and do not insert a run row.
6. If preflight passes, perform duplicate-run handling, insert the run row, and spawn `run_report_pipeline`.
7. `run_report_pipeline` emits the preflight summary before loading the full corpus.
8. Existing chunk map/reduce behavior continues under scheduler policy.

## Error Handling

Preflight database errors should map to existing internal/database error behavior. Limit violations are user-correctable and should map to `AppError::validation`.

If preflight returns a database or decompression error, `start_analysis_report` should fail before inserting an `analysis_runs` row. The user should not see a queued run that can never start.

Cancellation behavior does not change. If a run is cancelled after preflight passes, run-scoped cancellation still cancels queued or running LLM requests and marks the run cancelled.

## Testing

Backend tests should cover:

- preflight counts eligible text messages for one source;
- preflight counts across a source group;
- estimated chunks match `chunk_messages` behavior for boundary cases;
- zero eligible messages preserves the current no-documents error;
- exceeding message limit rejects before inserting an `analysis_runs` row;
- exceeding chunk limit rejects before inserting an `analysis_runs` row;
- exceeding estimated input character limit rejects before inserting an `analysis_runs` row;
- a passing preflight allows run insertion;
- scheduler tests still confirm two concurrent requests per `(provider, profile)` and interactive priority.

Frontend tests are optional for the first slice because existing error plumbing already displays typed backend errors. If copy or UI state changes are added, update `analysis-run-workflow` tests.

## Documentation

Update:

- `docs/backlog.md`
- `docs/project.md`
- `docs/design-document.md`
- `docs/architecture-deep-dive.md`

The docs should state that LLM scheduler concurrency is intentionally `2` per `(provider, profile)`, while analysis report runs are additionally capped by preflight limits before chunk workers are spawned.

## Out Of Scope

- Settings UI for custom limits.
- Provider-specific limit presets.
- Full corpus streaming, paged chunk construction, or preflight scanning without decompression.
- Tokenizer-accurate token counts.
- Confirmation modal for large but allowed runs.
- Retry-aware background request budgeting.
