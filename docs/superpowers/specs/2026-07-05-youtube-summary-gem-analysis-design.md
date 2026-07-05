# YouTube Summary Gem Analysis Mode Design

Status: revised draft for user review
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
- `run_transcript_analysis_stage_request` returns one `LlmCompletion`; the existing JSON repair in `execution.rs` repairs final transcript-analysis JSON, not internal `{ part, markdown }` JSON.
- Gemini Browser requests currently derive `browser_run_id` from `run_id + stage_run_id` with only repair-attempt variation, so Gem parts need an explicit request/run discriminator.
- The transcript-analysis output schema allows empty candidate arrays, but the implementation must test that normalization, intermediate entities, result builder, and canonical validation accept the final Gem output.

## Scope

In scope:

- Add `Gem analysis` to the existing `Summary mode` select.
- Add `gem_analysis` to the prompt-pack `control_preset` registry.
- Enforce `gem_analysis` for exactly one included YouTube video.
- Keep part 1 and part 3 transcript-only, but make the transcript timestamped for Gem runs.
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

## Data Contract

Each part analyzes its own source material independently.

Part 1 receives:

- Timestamped transcript text only.
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

- Timestamped transcript text only.
- No comments.
- No description.
- No source metadata.
- No part 1 or part 2 output.

Because part 1 and part 3 require time navigation, `gem_analysis` needs timestamped transcript material. Preserve the user-facing "transcript-only" rule by formatting transcript segments as transcript text with timestamps, for example:

```text
[00:00] Segment text
[00:17] Next segment text
```

Implementation decision for the first version: when `control_preset == "gem_analysis"`, snapshot transcript material with `[MM:SS]` prefixes derived from `youtube_transcript_segments.start_ms`. This requires adding a timestamped transcript source helper and passing `control_preset` into material snapshot creation. After the snapshot is created, Gem part prompts read only the frozen transcript material snapshot and must not read live transcript rows.

## Prompt Adjustments For Missing Data

The original user prompts mention metadata and fact-checking fields that are not present in transcript-only input. The implementation must keep the requested report structure but add explicit availability rules:

- Do not invent title, URL, author, subscriber count, duration, publication date, or view count.
- If those fields are absent from the transcript text, write `Недоступно во входных данных`.
- In the fact-check section, list only mentions found in the transcript.
- If the runtime has no external verification capability, write that independent fact-checking is unavailable instead of fabricating sources.
- If an API or browser model does external lookup on its own, source names and links may be included only when the model can provide concrete working references.

This keeps the prompt useful without forcing hallucinated metadata.

## Architecture

`gem_analysis` is handled in the YouTube Summary execution/orchestration layer, preferably in a focused helper module called from `execute_youtube_summary_run_with_stage_executor`.

For each transcript stage:

1. Build the normal `TranscriptAnalysisStageInput`.
2. Read `control_preset`.
3. If it is not `gem_analysis`, use the current transcript-analysis flow unchanged.
4. If it is `gem_analysis`, run `execute_gem_analysis_transcript_stage`.

`execute_gem_analysis_transcript_stage` performs:

1. Defense-in-depth guard that the run has exactly one included source snapshot.
2. Load timestamped transcript input for parts 1 and 3.
3. Load comments-only material from frozen material snapshots for part 2.
4. Execute part 1 (`passport`) through the selected runtime.
5. Execute part 2 (`comments`) only when non-empty comment material exists.
6. Execute part 3 (`deep_recap`) through the selected runtime.
7. Assemble one final transcript-analysis JSON object.
8. Persist it through the existing `execute_transcript_analysis_stage_with_completion` path so the current parsed output, intermediate entities, result builder, and canonical output remain the outer contract.

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

Repair request IDs append `-repair-<attempt>`.

Gemini Browser run IDs and sources must also include the part discriminator. This requires extending the browser helper so it can include an optional request discriminator, for example:

```text
prompt-pack-<run_id>-stage-<stage_run_id>-gem-passport
prompt_pack:youtube_summary:youtube_summary/transcript_analysis:gem-passport:run:<run_id>:stage:<stage_run_id>
```

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

Use only the input material provided below for this part. Do not use outputs from other Gem analysis parts. Do not invent timestamps, metadata, source titles, subscriber counts, metrics, or links. If a requested item is unavailable in the provided material, write `Недоступно во входных данных`. For fact-checking, do not fabricate sources or URLs; if external verification is unavailable in the current runtime, explicitly state that limitation.

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

The semantic structure remains:

- metadata/context;
- essence;
- optional how-to;
- adaptive module;
- mentions and available fact-checking;
- professional Russian style without filler phrases.

### Part 2 Body: Comments Analysis

Use the user's "ЧАСТЬ 2. Анализ комментариев к видео" prompt unchanged in structure. The only runtime wrapper constraint is that the input contains comments only and comments must be summarized, not quoted verbatim.

### Part 3 Body: Deep Interactive Recap

Use the user's "ЧАСТЬ 3. Глубокий интерактивный пересказ" prompt with one concrete timestamp constraint:

- Every required `[ММ:СС]` timestamp must come from the timestamped transcript input.
- If a point cannot be tied to an input timestamp, omit the timestamp for that point rather than inventing one, and keep this rare.

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

`Include comments` remains the user control for comments. In `gem_analysis`, it controls whether comment material is snapshotted for part 2. If disabled, absent, or empty, part 2 is skipped.

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
  "schema_id": "stage-io/youtube_summary_transcript_analysis_output",
  "attempt_number": 1,
  "gem_analysis": {
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

- Gem transcript input includes real `[MM:SS]` timestamps from frozen transcript data.
- Part 1 prompt receives timestamped transcript only.
- Part 3 prompt receives timestamped transcript only.
- Part 2 prompt receives comments only from frozen comment material snapshots.
- Part 2 skips when comments are disabled, absent, or empty.

Runtime:

- API request IDs are unique per Gem part.
- Gemini Browser run IDs are unique per Gem part.
- Browser prompt conversion still supports Gem part messages.
- Part parser accepts valid `{ part, markdown }`.
- Part parser rejects wrong part, missing markdown, empty markdown, and Markdown-fenced non-JSON.
- Part repair is attempted once on invalid part output.
- Required part failure fails the stage.
- Optional comments failure assembles a successful report with the part 2 failure note.

Final output:

- Assembled report starts with `# Gem-анализ`.
- Final parsed transcript-analysis output has `video_candidate.summary_text`.
- Final output includes all three part headings.
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
- Part 1 and part 3 remain transcript-only by using timestamped transcript text.
- Part 2 is comments-only and loaded from frozen comment material snapshots.
- Source metadata is not passed to parts in the first version.
- Per-part JSON repair is new narrow code, not reuse of final transcript-analysis repair.
- Browser runtime is supported with unique per-part IDs.
- No new artifact kinds are introduced in the first version.
- The final user-visible output is one assembled Russian Markdown report in `summary_text`.
