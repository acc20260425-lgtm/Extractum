# YouTube Summary Gem Analysis Mode Design

Status: approved for implementation planning
Date: 2026-07-05

## Objective

Add a third `Summary mode` to the existing YouTube Summary prompt pack:

- UI label: `Gem analysis`
- Internal `control_preset`: `gem_analysis`

`Gem analysis` is a single-video report mode that runs up to three independent LLM requests and assembles one Markdown report into the existing YouTube Summary result path.

## Reviewed Code Reality

The first draft assumed too much about existing stage inputs and retry behavior. The revised design is based on these current code facts:

- `TranscriptAnalysisStageInput` currently exposes only `transcript_segment_registry`, `allowed_material_refs`, `comment_selection_policy`, `control_preset`, `evidence_mode`, and `output_language`.
- `transcript_segment_registry` entries currently contain only `material_ref_id` and `text`; transcript timestamps from `youtube_transcript_segments.start_ms/end_ms` are not included.
- `build_transcript_analysis_stage_input` currently reads all material snapshots but only puts `material_kind == "transcript"` rows into `transcript_segment_registry`.
- Comment rows can be snapshotted when `include_comments` is true, but part 2 needs a new loader from `prompt_pack_run_material_snapshots`; comments are not already available in the stage input.
- The current comment snapshot policy uses `test_comment_policy()` in the production snapshot path: up to 50 comments, token-capped at 4000, ordered by earliest `published_at`. This is a bounded sample, not a representative audience corpus.
- `load_comment_text` can produce an empty string while `insert_material` still creates a `material_kind == "comment"` row, so part 2 must be gated by trimmed concatenated comment text, not by row existence.
- `run_transcript_analysis_stage_request` returns one `LlmCompletion`; the existing JSON repair in `execution.rs` repairs final transcript-analysis JSON, not internal `{ part, markdown }` JSON.
- Gemini Browser requests currently derive `browser_run_id` from `run_id + stage_run_id` with only repair-attempt variation, so Gem parts need an explicit request/run discriminator.
- Transcript material currently uses `transcript_text_for_source`, which joins segment text without timestamps. Gem timestamp formatting must use the same source selection and truncation policy as the normal transcript path, then add timing only at prompt construction time.
- Database migrations define `youtube_transcript_segments.start_ms` as `NOT NULL`, but older snapshots or future ingest paths may still lack structured timing in the frozen run input. The design needs an explicit degradation path for missing timestamp metadata.
- The transcript-analysis output schema allows empty candidate arrays, but the implementation must test that normalization, intermediate entities, result builder, and canonical validation accept the final Gem output.

## Scope

In scope:

- Add `Gem analysis` to the existing `Summary mode` select.
- Add `gem_analysis` to the prompt-pack `control_preset` registry.
- Enforce `gem_analysis` for exactly one included YouTube video.
- Keep part 1 and part 3 transcript-only, formatting timestamps from frozen structured timing metadata when available.
- Keep part 2 comments-only, loaded from frozen comment material snapshots.
- Add an internal Gem mini-pipeline in the execution/orchestration layer, not hidden inside the current one-completion transcript runtime function.
- Add new per-part JSON parsing and one-repair-attempt handling for `{ part, markdown }`.
- Support both API and Gemini Browser runtimes with unique request IDs and browser run IDs per Gem part.
- Assemble a standard `youtube_summary/transcript_analysis` parsed output with `video_candidate.summary_text`.

Out of scope for the first version:

- A separate prompt pack.
- New top-level persisted prompt-pack stages.
- Multi-video/playlist `gem_analysis` synthesis.
- A dedicated web-search/fact-check stage owned by Extractum.
- Passing description, source metadata, URL, subscribers, view counts, or prior part outputs into Gem parts.
- A result-view redesign.
- Changing the existing comment snapshot sample policy to top-liked, newest, or stratified comments.

## Data Contract

Each part analyzes its own source material independently.

Part 1 receives:

- Transcript text only, timestamped from frozen timing metadata when available.
- No comments.
- No description.
- No source snapshot metadata.
- No part 2 or part 3 output.

Part 2 receives:

- Comment text only, loaded from `prompt_pack_run_material_snapshots` rows where `material_kind == "comment"`.
- No transcript.
- No description.
- No source metadata.
- No part 1 or part 3 output.

Part 3 receives:

- Transcript text only, timestamped from frozen timing metadata when available.
- No comments.
- No description.
- No source metadata.
- No part 1 or part 2 output.

Because part 1 and part 3 require time navigation, `gem_analysis` needs timestamped transcript input. Preserve the user-facing "transcript-only" rule by formatting transcript segments as transcript text with timestamps, for example:

```text
[00:00] Segment text
[00:17] Next segment text
```

Implementation decision for the first version:

- Use bounded ordered transcript segments as the single source of truth for frozen transcript material. Build this segment list once during material snapshotting from the same source rows and ordering that `transcript_text_for_source` currently uses.
- Apply transcript freezing limits to the segment list before rendering any text. Truncate only on segment boundaries. If no complete segment can fit under the configured limit, fail the snapshot/input preparation with a clear overflow error rather than cutting a segment in the middle.
- Store the bounded segment list structurally in the transcript material snapshot for every prompt-pack run, preferably in existing `metadata_json_zstd`, as entries with at least `{ start_ms, end_ms, text }`. This is not `gem_analysis`-specific metadata.
- Render consumer-neutral `text_zstd` from the same bounded segment list by joining segment `text` values. Do not run a second, independent character truncation pass over the joined string.
- Build Gem part 1 and part 3 prompt material from the same bounded segment list by formatting each segment as `[MM:SS] text`.
- Existing and future non-Gem consumers may continue reading plain `text_zstd`; Gem reads the structured segment metadata from the frozen snapshot. Both views represent the same bounded segment list.
- If structured timing is missing for a run, degrade explicitly: part 1 and part 3 receive plain frozen transcript text, the prompt says timestamps are unavailable in the input, and the model is instructed not to invent timestamps. The report remains valid but loses interactive time navigation for that run.

This keeps material freezing neutral to downstream consumers while still giving Gem prompts timestamped transcript input when the run snapshot contains timing.

## Prompt Adjustments For Missing Data

The original user prompts mention metadata and fact-checking fields that are not present in transcript-only input. The implementation must keep the requested report structure but add explicit availability rules:

- Do not invent title, URL, author, subscriber count, duration, publication date, or view count.
- If those fields are absent from the transcript text, write `Недоступно во входных данных`.
- In the fact-check section, list only mentions found in the transcript.
- If the runtime has no external verification capability, write that independent fact-checking is unavailable instead of fabricating sources.
- If an API or browser model does external lookup on its own, source names and links may be included only when the model can provide concrete working references.

This keeps the prompt useful without forcing hallucinated metadata.

## Input Budget And Overflow

Gem analysis intentionally sends transcript material twice: once to part 1 and once to part 3. This is more expensive than `standard` or `detailed_report`, which send the transcript once. The implementation must make that cost explicit in validation instead of relying only on output token budgets.

Before starting Gem part requests, estimate input tokens for each part:

- part 1: shared wrapper + part 1 prompt + timestamped transcript input;
- part 2: shared wrapper + part 2 prompt + comments-only input, when comments are present;
- part 3: shared wrapper + part 3 prompt + timestamped transcript input.

The input cap for each part is the lower of:

- the selected API model input/context limit, when known from the provider/model registry;
- the configured prompt-pack runtime `max_prompt_tokens` value, enforced by new Gem input-budget logic;
- a Gem-specific app-side safety cap if the runtime cannot expose a model-specific context window.

Current transcript runtime code already carries `max_prompt_tokens` in configuration, but the normal transcript request path does not enforce it as an input guard. Gem analysis is the first feature in this area that must enforce an input-budget check before provider calls. Reserve an implementation constant for wrapper overhead and estimator error. The exact value can be tuned in code, but tests must prove that a prompt close to the cap is rejected before a provider call rather than failing after an expensive partial run.

First-version overflow policy:

- Do not truncate transcript input for Gem analysis. Truncation would make part 1 and part 3 disagree with the requested full-video report and could remove timestamps needed by the prompts.
- If part 1 or part 3 estimated input exceeds the cap, block the Gem transcript-analysis stage before any provider call with a clear error such as `Gem analysis transcript is too long for the selected model.`
- If part 2 estimated input exceeds the cap, reduce only the comment sample using the existing comment token cap policy before the prompt is built. If the trimmed comment text is still empty, skip part 2.
- For Gemini Browser, where the browser provider does not expose a reliable enforceable context window, apply the same app-side `max_prompt_tokens` guard and report overflow before opening the browser request.

The UI does not need a full token estimator in this slice, but the execution layer must enforce the guard because API keys and browser sessions can be used outside the normal UI preflight path.

## Comment Sampling And Sentiment Wording

For the first version, part 2 uses the comment material that the existing snapshot policy freezes:

- up to 50 comments;
- bounded by the current comment token cap;
- ordered by earliest publication time in the current production query.

This sample is useful for a bounded comments-only analysis, but it must not be described as representative of all comments or the whole audience. The part 2 prompt must scope every sentiment claim to the selected comment sample.

Required wording changes for part 2:

- Replace "percentage ratio of audience sentiment" with "qualitative or approximate distribution within the selected comment sample".
- Mention that the analysis is based only on the provided comment sample when describing general sentiment.
- Do not ask for exact percentages in v1. Use qualitative labels such as predominantly positive, mixed, skeptical, or neutral, with an explicit sample-level caveat.

Changing the snapshot policy to top-liked, newest, or stratified comments is out of scope for this feature slice. If that policy changes later, update this design and the prompt wording together.

## Architecture

`gem_analysis` is handled in the YouTube Summary execution/orchestration layer, preferably in a focused helper module called from `execute_youtube_summary_run_with_stage_executor`.

For each transcript stage:

1. Build the normal `TranscriptAnalysisStageInput`.
2. Read `control_preset`.
3. If it is not `gem_analysis`, use the current transcript-analysis flow unchanged.
4. If it is `gem_analysis`, run `execute_gem_analysis_transcript_stage`.

`execute_gem_analysis_transcript_stage` performs:

1. Defense-in-depth guard that the run has exactly one included source snapshot.
2. Load transcript input for parts 1 and 3, timestamped when frozen timing metadata is available.
3. Load comments-only material from frozen material snapshots for part 2 and compute trimmed concatenated comment text.
4. Run input-budget checks before the first provider call.
5. Check cancellation before starting part 1.
6. Execute part 1 (`passport`) through the selected runtime.
7. Check cancellation before optional part 2.
8. Execute part 2 (`comments`) only when the trimmed comment material is non-empty.
9. Check cancellation before part 3.
10. Execute part 3 (`deep_recap`) through the selected runtime.
11. Check cancellation before final assembly/persistence.
12. Assemble one final transcript-analysis JSON object.
13. Persist it through the existing `execute_transcript_analysis_stage_with_completion` path so the current parsed output, intermediate entities, result builder, and canonical output remain the outer contract.

The final assembled transcript-analysis JSON has this shape:

```json
{
  "stage_io_version": "1.0",
  "schema_version": "1.0",
  "stage": "youtube_summary/transcript_analysis",
  "video_candidate": {
    "summary_text": "# Gem-анализ\n\n..."
  },
  "claim_candidates": [],
  "evidence_fragment_candidates": [],
  "warning_candidates": []
}
```

`video_candidate.segment_candidates`, `key_point_candidates`, `quote_candidates`, `action_item_candidates`, and `open_question_candidates` may be omitted because the intermediate-entity builder treats those nested arrays as empty when missing. `claim_candidates` and `evidence_fragment_candidates` must be present as arrays.

## Cancellation And Retry Semantics

Gem analysis has multiple provider calls inside one persisted transcript-analysis stage, so it needs internal cancellation checkpoints. The implementation must check the same run cancellation source used by the outer execution loop:

- before part 1 starts;
- after part 1 succeeds and before part 2 starts or is skipped;
- after part 2 succeeds/fails/skips and before part 3 starts;
- after part 3 succeeds and before assembled output is persisted.

If cancellation is detected at a checkpoint, the stage must stop without starting later parts and should follow the existing run/stage cancellation semantics instead of producing a partial report.

First-version retry policy:

- There is no partial credit or per-part result cache.
- If required part 1 or part 3 fails after its repair attempt, the transcript-analysis stage fails.
- If part 3 fails after successful part 1 and optional part 2, a retry reruns all Gem parts from scratch.
- Optional part 2 may fail without failing the stage, but only after its own one repair attempt and with the user-facing failure note in the assembled report.

This accepts higher retry cost in v1 to keep persistence and result loading compatible with the existing single-stage output model.

## Runtime Request Model

The runtime enum needs explicit Gem part request support rather than reusing `TranscriptAnalysisStageExecutionRequest` with hidden prompt changes.

Add internal request types conceptually equivalent to:

```rust
GemAnalysisPartStageExecutionRequest {
    run_id,
    stage_run_id,
    source_snapshot_id,
    source_ref_id,
    part: GemAnalysisPart,
    prompt_input_json,
}

GemAnalysisPartRepairRequest {
    run_id,
    stage_run_id,
    source_snapshot_id,
    source_ref_id,
    part: GemAnalysisPart,
    attempt_number,
    prompt_input_json,
    raw_output,
    error_message,
}
```

Allowed parts:

- `passport`
- `comments`
- `deep_recap`

Part request IDs must be unique:

```text
prompt-pack-run-<run_id>-stage-<stage_run_id>-gem-passport
prompt-pack-run-<run_id>-stage-<stage_run_id>-gem-comments
prompt-pack-run-<run_id>-stage-<stage_run_id>-gem-deep-recap
```

Request ID suffixes are built exactly once:

- normal part request suffix: `gem-<part-slug>`;
- repair request suffix: `gem-<part-slug>-repair-<attempt>`.

Gemini Browser run IDs and sources must also include the part discriminator. This requires extending the browser helper so it can include an optional request discriminator, for example:

```text
prompt-pack-<run_id>-stage-<stage_run_id>-gem-passport
prompt_pack:youtube_summary:youtube_summary/transcript_analysis:gem-passport:run:<run_id>:stage:<stage_run_id>
```

The discriminator parameter must be optional. Existing transcript, synthesis, and repair callers pass `None` and must keep their current request IDs unchanged. Gem part and Gem repair callers pass the final request suffix, such as `gem-passport` or `gem-deep-recap-repair-1`. The browser helper must not append an additional repair suffix internally.

Browser runtime ignores `max_output_tokens`, so Gem prompts must include concise length targets and anti-bloat instructions. API runtime should still set max output token budgets.

## Part Output And Repair

Each part returns strict JSON:

```json
{
  "part": "passport",
  "markdown": "## ..."
}
```

Validation rules:

- response must parse as one JSON object;
- `part` must match the expected part;
- `markdown` must be a non-empty string after trimming;
- no Markdown fence around the JSON.

Existing transcript-analysis repair is not reused for part outputs. Add a narrow Gem part repair prompt that receives:

- expected part;
- parser/validation error;
- original part prompt input;
- invalid raw output.

Repair returns the same `{ part, markdown }` shape. Each part gets at most one repair attempt in the first version.

Required parts:

- Part 1 `passport`
- Part 3 `deep_recap`

If either required part fails after repair, the stage fails.

Optional part:

- Part 2 `comments`

If comments are absent, part 2 is skipped. If comments are present but part 2 fails after repair, the stage may still succeed and the final report includes a concise failure note for part 2.

## Final Markdown Assembly

The assembled report title is Russian to match the strict Russian-language requirement:

```markdown
# Gem-анализ

## Часть 1. Аналитический паспорт видео

<part 1 markdown>

---

## Часть 2. Анализ комментариев к видео

<part 2 markdown, skipped note, or failure note>

---

## Часть 3. Глубокий интерактивный пересказ

<part 3 markdown>
```

No-comments note:

```markdown
Пропущено: содержательные комментарии отсутствуют.
```

Part 2 failure note:

```markdown
Не выполнено: анализ комментариев завершился ошибкой после повторной попытки.
```

Detailed technical errors stay in logs/metrics, not in the user-facing report body.

## Part Prompt Bodies

The implementation should preserve the user's requested structures, with runtime wrappers for JSON output and the data-availability rules above.

### Shared System Message

```markdown
Return strict JSON for one Gem analysis part. Do not include Markdown fences, prose outside JSON, comments, or backend-owned IDs. Put the complete Russian Markdown report in the `markdown` field.
```

### Shared User Preamble

```markdown
Return exactly one strict JSON object:

{
  "part": "<passport|comments|deep_recap>",
  "markdown": "<full Russian Markdown report>"
}

Use only the input material provided below for this part. Do not use outputs from other Gem analysis parts. Do not invent timestamps, metadata, source titles, subscriber counts, metrics, or links. If a requested item is unavailable in the provided material, write `Недоступно во входных данных`. If transcript input has no `[MM:SS]` timestamps, state that timestamps are unavailable in the input and do not create approximate timestamps. For fact-checking, do not fabricate sources or URLs; if external verification is unavailable in the current runtime, explicitly state that limitation. Do not start `markdown` with `#` or `##`; the backend assembler owns the top-level report title and part headings. Start internal headings at `###`, use `####` for nested headings, and avoid leading/trailing horizontal rules.

Input material:
<part-specific material>

Task:
<part-specific prompt body>
```

### Part 1 Body: Analytical Passport

Use the user's "ЧАСТЬ 1. Аналитический паспорт видео" prompt with these concrete edits:

- The "Инфо-карта" section must say that title, original URL, author, subscriber count, duration, publication date, and view count are filled only if they are present in the transcript text; otherwise each missing field is `Недоступно во входных данных`.
- The "Таймлайн" and How-to timecodes must use only `[MM:SS]` timestamps present in the timestamped transcript input.
- The fact-check section must be renamed to `Внешний контекст и Ресурсы (упоминания и доступный фактчекинг)` and must not require fabricated hyperlinks.
- If external verification is unavailable, the fact-check subsection must say: `Независимый фактчекинг недоступен в текущем runtime; ниже перечислены только упоминания из транскрипта.`
- Downshift any requested part-internal headings so the returned part Markdown nests under the assembler's `## Часть 1...` heading. Use `###` for top internal sections and avoid extra top/bottom `---`.

The semantic structure remains:

- metadata/context;
- essence;
- optional how-to;
- adaptive module;
- mentions and available fact-checking;
- professional Russian style without filler phrases.

### Part 2 Body: Comments Analysis

Use the user's "ЧАСТЬ 2. Анализ комментариев к видео" prompt with sampling-aware edits:

- The input contains comments only and comments must be summarized, not quoted verbatim.
- The report must say that sentiment is based on the provided selected comment sample.
- The "Общий сентимент" item must ask for a qualitative or approximate distribution inside the selected sample, not a percentage for the whole audience.
- Exact percentages are out of scope for v1; the model should use qualitative labels and sample-level caveats instead.

### Part 3 Body: Deep Interactive Recap

Use the user's "ЧАСТЬ 3. Глубокий интерактивный пересказ" prompt with one concrete timestamp constraint:

- Every required `[ММ:СС]` timestamp must come from the timestamped transcript input.
- If a point cannot be tied to an input timestamp, omit the timestamp for that point rather than inventing one, and keep this rare.
- Downshift the requested `##`/`###` chapter structure by one level so the returned part Markdown nests under the assembler's `## Часть 3...` heading. Avoid extra top/bottom `---`; use separators only inside the part when they materially improve readability.

The semantic structure remains:

- 800-1000+ word dense recap when the transcript has enough substance;
- logical chapters;
- timestamped navigation;
- tables when comparing concepts;
- LaTeX/code blocks when the transcript contains formulas or code;
- strict Russian analytical style without filler phrases.

## UX

The `Summary mode` select adds:

```svelte
<option value="gem_analysis">Gem analysis</option>
```

The default remains `detailed_report`.

When `gem_analysis` is selected, preflight/start block unless exactly one video is included:

```text
Gem analysis supports exactly one YouTube video.
```

The execution layer repeats this guard so bypassing preflight cannot produce N independent Gem mini-pipelines plus synthesis.

`Include comments` remains the user control for comments. In `gem_analysis`, it controls whether comment material is snapshotted for part 2. If disabled, absent, or empty after trimming concatenated comment text, part 2 is skipped.

## Progress And Events

The outer run still has one persisted transcript-analysis stage. Internal part calls should emit unique request IDs and phases so the UI/event log is understandable.

Suggested phases:

- `gem_passport`
- `gem_comments`
- `gem_deep_recap`
- `gem_part_repair`

Suggested messages:

- `Gem analysis: building analytical passport`
- `Gem analysis: analyzing comments`
- `Gem analysis: writing deep recap`
- `Gem analysis: repairing part JSON`

If new phase strings are introduced, update `docs/value-registry.md`.

The run progress counter can remain one transcript stage for the first version; detailed per-part status lives in events and metrics.

## Artifacts And Observability

Avoid storing per-part outputs as normal `parsed_output` artifacts in the first version because result loading chooses the latest parsed output for the stage.

First-version artifact plan:

- Keep the final stage `prompt_input`, `raw_output`, `parsed_output`, `metrics`, and `intermediate_entities` artifacts as they work today.
- Treat the final `raw_output` as the backend-assembled transcript-analysis JSON, not a single provider response.
- Store per-part observability inside the final `metrics` artifact under a `gem_analysis` object. The current `execute_transcript_analysis_stage_with_completion` builds metrics internally, so implementation must either extend that helper to accept an optional metrics extension or add a sibling persistence helper for assembled Gem output that writes the same outer artifacts plus Gem metrics.

```json
{
  "metrics_kind": "youtube_summary_transcript_analysis",
  "metrics_version": "1.0",
  "attempt_number": 1,
  "gem_analysis": {
    "input_budget": {
      "status": "passed",
      "cap_tokens": 24000,
      "part_estimates": {
        "passport": 12000,
        "comments": 2500,
        "deep_recap": 12000
      }
    },
    "parts": [
      {
        "part": "passport",
        "status": "succeeded",
        "request_id": "...",
        "input_tokens": null,
        "output_tokens": null,
        "latency_ms": 1234,
        "repair_attempted": false
      }
    ],
    "comments_part": {
      "status": "skipped_no_comments"
    }
  }
}
```

Do not add new `artifact_kind` values in this slice. A later migration can add first-class per-part artifact kinds if detailed raw-output inspection becomes necessary.

## Output Token Budgets

For API runtime:

- Part 1: at least `8192`, capped by model output limit.
- Part 2: at least `4096`, capped by model output limit.
- Part 3: at least `8192`, capped by model output limit.

For Gemini Browser:

- No token cap is enforceable through current browser provider.
- The prompts should include length guidance and direct "avoid filler" instructions.

The implementation should not promise exact length compliance for Browser runtime.

## Validation And Registry

Update `docs/value-registry.md`:

- `Prompt-pack control preset`: add `gem_analysis`.
- Add any new event `phase` values introduced for Gem part progress.

Add or update tests to prove:

- final transcript-analysis schema accepts the assembled Gem output;
- `execute_transcript_analysis_stage_with_completion` accepts the assembled output;
- intermediate-entities artifact generation accepts empty candidate arrays;
- canonical result builder and result validation accept a single-video Gem result with empty claims/evidence if that is the chosen final shape.

If any of those checks fail, implementation must either:

- add minimal, grounded candidates derived from part markdown and transcript material refs, or
- update validation/normalization deliberately with tests.

## Tests

Frontend:

- Contract test verifies the `Gem analysis` option and `gem_analysis` value exist.
- Contract test preserves default `detailed_report`.

Preflight/start:

- `gem_analysis` blocks when included video count is not exactly one.
- Execution helper repeats the single-video guard.

Stage input/data:

- Transcript material snapshot uses bounded ordered transcript segments as the single source of truth.
- Transcript material snapshot writes structured timing metadata for all prompt-pack runs, not only `gem_analysis`.
- Consumer-neutral `text_zstd` is rendered from the same bounded segment list stored in metadata.
- Transcript truncation happens on segment boundaries, before rendering `text_zstd` or Gem timestamped input.
- Plain `text_zstd` and Gem timestamped input contain the same segment texts in the same order.
- Gem transcript input includes real `[MM:SS]` timestamps from frozen transcript timing metadata when metadata is present.
- Gem transcript input degrades to plain transcript text and timestamp-unavailable instructions when timing metadata is missing.
- Gem timestamped input uses the same source selection and truncation policy as the normal transcript snapshot helper.
- Part 1 prompt receives transcript only, timestamped when timing metadata is available.
- Part 3 prompt receives transcript only, timestamped when timing metadata is available.
- Part 2 prompt receives comments only from frozen comment material snapshots.
- Part 2 skips when comments are disabled, absent, or empty.
- Part 2 skips when comment material rows exist but trimmed concatenated comment text is empty.
- Part 2 prompt scopes sentiment to the selected comment sample and does not ask for exact percentages.
- Long transcript input that exceeds the per-part input cap blocks before any provider call and is not silently truncated.
- Part 2 comment input obeys the existing comment token cap before prompt construction.

Runtime:

- API request IDs are unique per Gem part.
- Gemini Browser run IDs are unique per Gem part.
- Browser request discriminator is optional; existing transcript, synthesis, and repair callers passing `None` keep unchanged IDs.
- Gem repair request IDs and browser discriminators include `-repair-<attempt>` exactly once.
- Browser prompt conversion still supports Gem part messages.
- Part parser accepts valid `{ part, markdown }`.
- Part parser rejects wrong part, missing markdown, empty markdown, and Markdown-fenced non-JSON.
- Part repair is attempted once on invalid part output.
- Required part failure fails the stage.
- Optional comments failure assembles a successful report with the part 2 failure note.
- Cancellation before part 1 prevents all provider calls.
- Cancellation after part 1 prevents part 2/part 3 calls and does not persist a partial report.
- Cancellation after part 2 prevents part 3 and does not persist a partial report.
- Required part 3 failure after successful earlier parts fails the stage; retry reruns all parts because v1 has no partial result cache.

Final output:

- Assembled report starts with `# Gem-анализ`.
- Final parsed transcript-analysis output has `video_candidate.summary_text`.
- Final output includes all three part headings.
- Part markdown is nested under assembler-owned `## Часть N` headings and does not introduce duplicate top-level `#` or part-level `##` headings.
- No-comments run includes the skipped note.
- Result builder renders the summary in the existing viewer path.

Verification:

- Run the focused frontend contract test.
- Run focused Rust tests for prompt-pack runtime/execution/data helpers.
- Run `cargo check` after Rust backend changes.
- Run `npm.cmd run check` if Svelte/TypeScript changes are broad enough to affect type checking.

## Implementation Decisions Closed By This Revision

- `gem_analysis` remains a `Summary mode`, not a new prompt pack.
- It is single-video only, enforced in preflight/start and execution.
- It uses independent part calls.
- Part 1 and part 3 remain transcript-only; timestamps are formatted from structured frozen timing metadata at prompt-build time when available.
- Bounded ordered transcript segments are the single source of truth for transcript material snapshots.
- Transcript `text_zstd` is rendered from those bounded segments and is not truncated independently.
- Transcript timing metadata is written for all prompt-pack transcript snapshots, not only Gem runs.
- Frozen transcript text remains consumer-neutral and does not vary by `control_preset`.
- If timestamp metadata is missing, Gem degrades to plain transcript input with explicit no-timestamp instructions.
- Part 1 and part 3 input overflow blocks the stage before provider calls; v1 does not truncate transcript input.
- Part 2 is comments-only and loaded from frozen comment material snapshots.
- Part 2 runs only when trimmed concatenated comment text is non-empty.
- Part 2 sentiment language is scoped to the selected comment sample, not the whole audience.
- Internal cancellation checkpoints are required before/between Gem part calls.
- There is no partial per-part result cache or partial credit in v1; retries rerun all parts.
- Source metadata is not passed to parts in the first version.
- Per-part JSON repair is new narrow code, not reuse of final transcript-analysis repair.
- Browser runtime is supported with unique per-part IDs.
- Browser helper discriminator is optional and existing callers pass `None`.
- Gem repair suffixes are appended exactly once by the Gem request-suffix builder.
- Gem part Markdown is nested under assembler-owned `## Часть N` headings.
- No new artifact kinds are introduced in the first version.
- The final user-visible output is one assembled Russian Markdown report in `summary_text`.
