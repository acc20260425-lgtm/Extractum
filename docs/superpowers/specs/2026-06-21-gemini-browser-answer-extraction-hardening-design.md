# Gemini Browser Answer Extraction Hardening Design

## Context

The Gemini Browser Provider can now run through a user-controlled Chrome CDP
session, record file-backed runs, and show an inline run inspector in Settings.
The remaining reliability issue is answer extraction: Gemini may visibly produce
a long answer in the browser while Extractum receives only a prefix or a shorter
fragment.

The current sidecar flow in `sidecars/gemini-browser/src/adapter.ts` is:

1. Capture a baseline answer state before clicking Send.
2. Poll answer selectors after Send.
3. Build `AnswerState` from `allTextContents()` for each selector.
4. Drop entries that match the prompt or baseline.
5. Select the longest new text entry.
6. Return it after an 8 second stable window, or return the latest entry at
   timeout.

The inline run inspector already exposes `answer_selector`,
`answer_completion_reason`, `waited_for_answer_ms`, and `final_text_length`.
That helped show whether the sidecar and UI lengths match, but it does not yet
explain why a particular DOM node was selected or whether competing candidate
nodes contained more text.

## Problem

When Gemini changes its page structure, streams slowly, splits answers across
multiple DOM nodes, or updates the answer container after an apparent stable
window, the current extraction algorithm can accept an incomplete text entry.
The UI then shows a partial answer even though the full answer is visible in the
browser.

Likely failure modes:

- The selected selector points at one answer fragment instead of the complete
  assistant turn.
- Gemini renders markdown sections or list items as separate nodes; choosing
  the longest single node loses sibling content.
- A candidate becomes text-stable before Gemini appends later blocks elsewhere
  in the same answer.
- The baseline filter removes or ignores the wrong entry after DOM reuse.
- The answer wait ends at timeout with `timeout_latest`, but the UI does not
  clearly surface that this may be partial.
- A future DOM redesign keeps `message-content` present but changes where the
  full answer text lives.

## Goals

- Prefer a complete assistant response container over a single text fragment.
- Make the extraction decision observable in run diagnostics.
- Distinguish a truly stable full answer from latest-at-timeout partial text.
- Preserve privacy: diagnostics should not copy prompt text, answer text,
  account hints, sensitive URLs, or local artifact paths.
- Keep the Gemini Browser Provider usable without requiring network interception
  or unsupported Gemini APIs.
- Add deterministic tests for long answers, split answers, slow growth, and DOM
  drift.

## Non-Goals

- Do not automate Google login, consent, CAPTCHA, account picker, or phone
  verification.
- Do not weaken the CDP loopback-only security boundary.
- Do not add remote Gemini API calls as a fallback.
- Do not store full Gemini answers in copied diagnostics.
- Do not turn Browser Provider diagnostics into a general raw DOM dump in the
  UI.

## Extraction Model

Introduce an answer extraction module that separates three concepts:

- **Raw node observation:** collect visible candidate nodes from the known answer
  selector contract.
- **Response grouping:** aggregate related nodes into a candidate assistant turn
  when Gemini splits one answer across descendants or sibling blocks.
- **Selection and completion:** score grouped candidates, pick the best answer,
  and decide whether generation is stable or still partial.

The extractor should continue to use DOM selectors first. It may read local DOM
structure, accessible names, text contents, visibility, and element order. It
must not depend on Gemini private backend endpoints.

Default timing constants for v1:

- `ANSWER_POLL_INTERVAL_MS = 500`;
- `ANSWER_STABLE_MS = 8_000`;
- `MIN_STABLE_POLLS_AFTER_SIGNATURE_CHANGE = 3`;
- `MAX_ANSWER_TIMEOUT_MS = 120_000`.

These constants may be configurable in tests, but production defaults should be
explicit so implementation and tests assert the same behavior.
`MAX_ANSWER_TIMEOUT_MS` is intentionally longer than the current 60 seconds
because Gemini can stream slowly through the browser UI. If the implementation
keeps the existing `ANSWER_TIMEOUT_MS` name, it must use this exact hard-maximum
value.

In the normal quiet case, the earliest stable return is:

```text
last_candidate_change_at
+ max(
    ANSWER_STABLE_MS,
    MIN_STABLE_POLLS_AFTER_SIGNATURE_CHANGE * ANSWER_POLL_INTERVAL_MS
  )
```

With the defaults above, a short answer that stops changing immediately should
usually return after roughly 8 seconds plus polling granularity, not after the
120 second hard maximum.

### Candidate Collection

Each poll should produce an internal `AnswerExtractionSnapshot`:

```ts
interface AnswerExtractionSnapshot {
  elapsed_ms: number;
  busy_visible: boolean;
  raw_candidates: AnswerCandidateObservation[];
  grouped_candidates: AnswerCandidateSummary[];
  selected_candidate_id: string | null;
  selected_candidate_signature: string | null;
  selection_reason: string | null;
}
```

The persisted or copied diagnostic form must be safe. It may include selector
names, candidate counts, text lengths, score components, and DOM-derived
candidate ids, but not full text. Do not include deterministic hashes of prompt
or answer text in copied diagnostics. If implementation needs a correlation id
inside a local artifact, prefer a DOM/group id based on selector and element
order. Text-derived hashes should be avoided; if they become necessary for a
local-only artifact, they must be keyed per run, omitted from copied
diagnostics, and documented as non-contractual debug data.

Candidate observations should include:

- selector;
- DOM order index;
- visible flag;
- text length;
- whether it matched baseline;
- whether it looked like composer/user prompt/navigation text;
- parent/group identifier when grouping is possible.

### Structural Baseline

The baseline captured before Send must be structural, not only text-based.
Repeated or similar answers are valid Gemini outputs, so text equality alone
must not cause a new assistant turn to be dropped.

Baseline data should include:

- raw candidate selector and DOM order;
- accepted group id and group DOM order;
- selected ancestor identity when available, such as response index, role/name
  pattern, or stable attribute;
- candidate/group text lengths and descendant block lengths;
- optional text equality markers only as a secondary signal.

After Send, a candidate is considered new when its group id/order/structural
signature did not exist in the baseline, or when it appears after the latest
baseline group in DOM order. Text equality with a baseline answer is not enough
to reject it.

Generated DOM-order ids are per-poll diagnostic ids only. They should not be the
primary structural baseline key because Gemini can insert nodes and shift
absolute DOM paths while streaming. Baseline matching should prefer stable
attributes or response indices, then relative position after the latest baseline
group, and only use generated DOM-order paths as a last-resort diagnostic
fallback.

If no stable attribute or response index exists, only groups after the highest
pre-submit group order should be considered new. Never drop a post-submit
candidate solely because its structural signature or text shape resembles a
baseline candidate.

### Grouping

The extractor should prefer complete assistant-turn containers. For selectors
such as `message-content`, it should find the closest deterministic answer
ancestor and aggregate visible descendant text in DOM order. If no answer
ancestor can be identified, it may fall back to a single node, but diagnostics
must mark that as `grouping: "single_node"`.

Ancestor search must be deterministic:

1. Start at the matched raw answer node.
2. Climb at most six ancestors.
3. Stop before page/conversation/composer containers such as `body`, `main`,
   `form`, `[role="main"]`, `[role="textbox"]`, `textarea`,
   `[contenteditable="true"]`, and known composer containers.
4. Accept the smallest ancestor that looks like an assistant turn, using an
   allowlist of stable patterns such as a response index attribute, an
   assistant/model message role attribute, an article/list item containing
   answer content but no composer textbox, or a direct parent that owns multiple
   `message-content` descendants for the same assistant turn.
5. Reject ancestors with visible send/composer controls, account/login UI,
   navigation labels, or previous-turn controls. Prompt text alone is not enough
   to reject an ancestor because Gemini may quote or restate the user question.
   Reject prompt-containing ancestors only when the prompt text is outside answer
   descendants, or when user/composer role signals indicate the ancestor is a
   prompt container rather than an assistant answer.
6. When multiple valid ancestors remain, prefer the deepest ancestor; break ties
   by later DOM order.

Accepted groups must satisfy a one-turn invariant. A grouped candidate may
represent exactly one post-baseline assistant turn. If an accepted ancestor
contains multiple response indices, multiple assistant/message role groups, or
multiple post-baseline assistant candidate groups, the extractor must split the
children by response index/message role before scoring. If it cannot split them
deterministically, reject that ancestor and continue with a narrower grouping
or `single_node` fallback.

The grouped candidate text is then built from visible answer descendants inside
that ancestor, ordered by DOM position. It must not blindly use the full
ancestor `innerText` when that would include buttons, citations controls,
composer text, or unrelated previous turns.

Grouping rules should avoid broad page-level fallbacks such as `main` or
`section` unless they are explicitly constrained to a post-submit assistant
turn. Broad fallbacks are too likely to capture composer controls, prompt text,
or unrelated page content.

### Scoring

The selected answer should be the best grouped candidate, not necessarily the
longest raw text node. Scoring should consider:

- selector priority from the DOM contract;
- post-submit newness relative to baseline;
- assistant-turn grouping confidence;
- DOM order, preferring the latest matching assistant response;
- text length after normalization;
- exclusion of the prompt/composer/account/navigation text;
- visibility and non-empty rendered text;
- growth history over polling.

Diagnostics should expose enough score facts to explain why a candidate won:
for example `selected_candidate_length`, `candidate_count`,
`selected_selector`, `selected_grouping`, and a compact list of top candidate
lengths and selectors.

Candidate signatures are internal completion inputs. A grouped candidate
signature should be deterministic for a poll and should not require storing full
text. It should include:

- selector and group id;
- group DOM order;
- grouping mode;
- selected ancestor pattern;
- descendant answer block count;
- normalized descendant block lengths in DOM order;
- total normalized text length.

The signature may use the current text length and block-length vector, but it
must not be copied into diagnostics as a text hash. If an implementation uses a
text-derived hash internally to detect same-length text mutation, it must stay
memory-only or be keyed per run and omitted from persisted/copied diagnostics.
`selected_candidate_signature` is an internal polling field. Do not expose the
raw signature string in `debug_summary`, copied diagnostics, or the reduced
artifact when it contains text-derived material. If the artifact needs signature
evidence, store separate structural fields such as selector, group id, DOM
order, block count, and block lengths instead of a raw signature string.

## Completion Model

The current completion reasons are `stable`, `timeout_latest`, and `missing`.
Keep those as the public v1 surface, but make their meaning stricter:

- `stable`: selected grouped candidate text length and block structure stayed
  unchanged for at least `ANSWER_STABLE_MS`, candidate count/signature/rank and
  largest valid candidate length stayed unchanged for at least
  `MIN_STABLE_POLLS_AFTER_SIGNATURE_CHANGE` consecutive polls after the last
  candidate-count, grouping, signature, rank, or largest-length change, no larger
  valid candidate appeared during those polls, and no known generation-busy UI
  was visible during the final quiet window.
- `timeout_latest`: some candidate text was visible, but the extractor could not
  prove stability before timeout.
- `missing`: no valid post-submit answer candidate appeared.

If implementation needs more detail, add internal diagnostic fields rather than
expanding the public result status first. Good examples:

- `busy_visible_at_completion`;
- `candidate_count_at_completion`;
- `selected_candidate_changed_count`;
- `last_growth_elapsed_ms`;
- `stable_window_satisfied`;
- `stable_poll_count_after_last_candidate_change`;
- `candidate_signature_changed_count`;
- `completion_notes`.

The sidecar should keep waiting while generation-busy UI is visible, subject to a
hard maximum timeout. If the answer keeps growing, the stable timer resets. If a
new grouped candidate becomes better than the previous selected candidate, the
stable timer also resets. Busy UI is only an additional signal; it is not
trusted as the only completion gate. The grouped candidate signature, selected
candidate rank, candidate count, and largest candidate length must remain quiet
for both `ANSWER_STABLE_MS` and
`MIN_STABLE_POLLS_AFTER_SIGNATURE_CHANGE` post-change polls before returning
`stable`. This gives protection when Gemini appends a later block after the
stop/busy indicator is absent or undetected.

## Artifacts And Diagnostics

Add a reduced answer extraction artifact for failure and partial cases:

- captured for `timeout`, `failed`, `browser_crashed`, and any run whose
  completion reason is `timeout_latest`;
- written under the run artifact directory;
- contains no full prompt or answer text;
- contains selector names, candidate counts, lengths, score facts, completion
  reason, and sanitized error stage;
- safe to inspect locally and summarize in copied diagnostics.

`timeout_latest` is not a run result status. If visible text exists at timeout,
the result may still be `status: "ok"` with returned text, but it must be treated
as a **partial-risk success** for Settings/test UI:

- write the reduced extraction artifact;
- expose `answer_completion_reason: "timeout_latest"`;
- show the partial-risk state in the run inspector;
- include the partial-risk fact in copied diagnostics;
- do not silently present it as equivalent to `stable`.

Automation consumers must not treat `status: "ok"` with
`answer_completion_reason: "timeout_latest"` as a normal prompt completion. The
prompt-pack Browser Provider stage should either reject it as a typed partial
result or mark it explicitly for retry/manual review. Acceptance tests must cover
that prompt-pack/browser-stage mapping so partial browser text is not silently
fed into downstream analysis.

Extend `debug_summary` with compact extraction facts:

```ts
type GeminiBrowserCandidateRejectReason =
  | "baseline"
  | "composer"
  | "prompt_container"
  | "navigation"
  | "account_or_login"
  | "controls"
  | "multi_turn"
  | "not_visible"
  | "empty"
  | "lower_score";

interface GeminiBrowserAnswerExtractionDebug {
  raw_candidate_count: number;
  grouped_candidate_count: number;
  selected_candidate_length: number;
  returned_text_length: number;
  selected_grouping: "assistant_turn" | "single_node" | "unknown";
  selected_candidate_rank: number | null;
  selected_score: number | null;
  largest_candidate_length: number;
  larger_valid_candidate_available: boolean;
  larger_rejected_candidate_count: number;
  larger_rejected_reasons: GeminiBrowserCandidateRejectReason[];
  top_candidate_lengths: number[];
  busy_visible_at_completion: boolean;
  last_growth_elapsed_ms: number | null;
  candidate_signature_changed_count: number;
  stable_poll_count_after_last_candidate_change: number;
}
```

This is nested under the existing `debug_summary` to avoid broad result DTO
churn:

```ts
interface GeminiBrowserRunDebugSummary {
  // existing fields...
  extraction?: GeminiBrowserAnswerExtractionDebug | null;
}
```

The `extraction` field itself is optional and nullable for older run records and
unexpected legacy sidecar responses. Once `extraction` is present, its fields
are required for new sidecar results so the run inspector can rely on a complete
compact diagnostic summary. Rust DTOs should mirror this optional nested shape
and tolerate older run JSON without it.

The inline run inspector should surface:

- raw/grouped candidate counts;
- selected grouping;
- selected length vs result text length;
- selected rank and largest candidate length;
- larger valid candidate availability and larger rejected candidate count;
- completion reason;
- busy at completion;
- candidate signature change count;
- whether an extraction artifact is available.

`Copy diagnostics` should include these fields but still omit full answer text,
full prompt text, raw local paths, URL query/hash data, and account hints.

The result artifact DTO gains an optional `answer_extraction` field:

```ts
interface GeminiBrowserRunArtifacts {
  run_dir: string | null;
  html: string | null;
  screenshot: string | null;
  telemetry: string | null;
  answer_extraction?: string | null;
  artifact_write_error: string | null;
}
```

Rust and TypeScript DTOs must tolerate missing `answer_extraction` for older run
records. The path should only be a local artifact path in persisted run JSON; it
must not be copied into diagnostics.

Failure to write `artifacts.answer_extraction` must not hide the original run
result. Artifact write errors should populate `artifacts.artifact_write_error`
and leave the run status/completion reason intact. In particular,
`ok + timeout_latest` must remain an `ok` partial-risk run even if the reduced
extraction artifact cannot be written.

The extraction artifact is local-only debug material. It should be safe from
full prompt/answer text, but selector ids, DOM order, lengths, and score facts
can still reveal answer shape. Treat it like other local Browser Provider
artifacts: useful for local debugging, not safe for external sharing without
operator review.

## Testing Strategy

Add deterministic sidecar tests around the extractor before changing production
behavior:

- one answer split across multiple `message-content` descendants is returned as
  one complete text;
- a long markdown/list answer grows in chunks and only returns `stable` after
  the final grouped text stabilizes;
- a single fragment stabilizes while sibling content appears later, and the
  extractor waits for the grouped candidate;
- timeout with visible partial text returns `timeout_latest` and includes partial
  diagnostics;
- prompt-pack/browser-stage automation does not silently treat
  `ok + timeout_latest` as a normal completion;
- a broad page section containing composer text is not selected as an answer;
- baseline entries from a previous answer are ignored without dropping the new
  assistant turn;
- copied diagnostics include candidate counts and lengths but not answer text,
  prompt text, paths, email-like hints, or sensitive URLs;
- at least one Playwright DOM fixture page exercises real DOM ancestor grouping
  with nested `message-content`, sibling markdown/list blocks, composer
  controls, and a previous answer. Mocked locator tests are useful for polling
  behavior, but they are not sufficient for validating grouping.
- the DOM fixture should be a static local HTML page loaded by headless
  Playwright with no network dependency. Timers should be deterministic through
  test-controlled DOM mutation rather than real Gemini streaming.

Add frontend/Rust tests only for DTO and UI display changes:

- Rust run-log round trips optional extraction debug fields.
- TypeScript DTOs accept missing extraction debug fields.
- Run inspector displays extraction facts when present and remains useful when
  absent.

Manual validation should use the known Russian football prompt that previously
made partial extraction visible. A successful manual run should show:

- `result_text_length === debug_final_text_length`;
- completion reason `stable` for a fully rendered answer;
- raw candidate count may be greater than grouped candidate count where split
  DOM appears;
- selected grouped candidate length is greater than each individual fragment
  length in split-answer cases;
- selected grouping not `unknown`;
- copied diagnostics do not include answer text.

Manual or semi-manual validation should also include a deliberately slow or long
answer. The acceptable outcomes are either:

- `stable` with full text and matching result/debug lengths; or
- `timeout_latest` clearly marked as partial-risk in the run inspector and copied
  diagnostics, with downstream automation guarded from consuming it as normal
  completion.

## Rollout

Implement this as a narrow Browser Provider slice:

1. Add extractor tests and internal extraction data structures.
2. Replace `bestNewAnswerText()` with grouped candidate selection.
3. Tighten completion logic around grouped candidate stability and busy UI.
4. Extend `debug_summary` and run inspector display.
5. Add reduced extraction artifact for partial/failure cases.
6. Update Browser Provider troubleshooting docs with the new debugging flow.

The implementation should avoid changing CDP launch, sidecar packaging, or the
test prompt UI unless needed for diagnostics display.

## Acceptance Criteria

- The sidecar returns complete grouped answer text for split/long DOM fixtures.
- Partial/timeout cases are visibly distinguishable in run diagnostics.
- Run inspector shows why a candidate was selected.
- Copied diagnostics remain privacy-safe.
- Prompt-pack/browser-stage automation does not silently consume
  `ok + timeout_latest` as a normal completion.
- Existing Browser Provider happy path still works in CDP attach mode.
- Managed mode remains compatible with the shared extractor and passes the same
  extraction tests as CDP attach mode.
- The design of the Gemini page can drift without immediately forcing a blind
  selector tweak; diagnostics should identify what changed.
