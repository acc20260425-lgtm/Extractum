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

### Candidate Collection

Each poll should produce an internal `AnswerExtractionSnapshot`:

```ts
interface AnswerExtractionSnapshot {
  elapsed_ms: number;
  busy_visible: boolean;
  raw_candidates: AnswerCandidateObservation[];
  grouped_candidates: AnswerCandidateSummary[];
  selected_candidate_id: string | null;
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
5. Reject ancestors that contain the current prompt text, visible send/composer
   controls, account/login UI, navigation labels, or previous-turn controls.
6. When multiple valid ancestors remain, prefer the deepest ancestor; break ties
   by later DOM order.

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

## Completion Model

The current completion reasons are `stable`, `timeout_latest`, and `missing`.
Keep those as the public v1 surface, but make their meaning stricter:

- `stable`: selected grouped candidate stayed unchanged for the stable window,
  candidate count/signature stayed unchanged for several consecutive polls after
  the last candidate-count or grouping change, no larger valid candidate appeared
  during that window, and no known generation-busy UI was visible during the
  final quiet window.
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
for the completion window before returning `stable`. This gives protection when
Gemini appends a later block after the stop/busy indicator is absent or
undetected.

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
as a **partial-risk success**:

- write the reduced extraction artifact;
- expose `answer_completion_reason: "timeout_latest"`;
- show the partial-risk state in the run inspector;
- include the partial-risk fact in copied diagnostics;
- do not silently present it as equivalent to `stable`.

Extend `debug_summary` with compact extraction facts:

```ts
interface GeminiBrowserAnswerExtractionDebug {
  raw_candidate_count: number;
  grouped_candidate_count: number;
  selected_candidate_length: number;
  returned_text_length: number;
  selected_grouping: "assistant_turn" | "single_node" | "unknown";
  selected_candidate_rank: number | null;
  selected_score: number | null;
  largest_candidate_length: number;
  larger_candidate_available: boolean;
  top_candidate_lengths: number[];
  busy_visible_at_completion: boolean;
  last_growth_elapsed_ms: number | null;
  candidate_signature_changed_count: number;
  stable_poll_count_after_last_candidate_change: number;
}
```

This can be nested under the existing `debug_summary` to avoid broad result DTO
churn. Rust DTOs should mirror the optional fields and tolerate older run JSON
without them.

The inline run inspector should surface:

- raw/grouped candidate counts;
- selected grouping;
- selected length vs result text length;
- selected rank and largest candidate length;
- completion reason;
- busy at completion;
- candidate signature change count;
- whether an extraction artifact is available.

`Copy diagnostics` should include these fields but still omit full answer text,
full prompt text, raw local paths, URL query/hash data, and account hints.

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
- a broad page section containing composer text is not selected as an answer;
- baseline entries from a previous answer are ignored without dropping the new
  assistant turn;
- copied diagnostics include candidate counts and lengths but not answer text,
  prompt text, paths, email-like hints, or sensitive URLs.
- at least one Playwright DOM fixture page exercises real DOM ancestor grouping
  with nested `message-content`, sibling markdown/list blocks, composer
  controls, and a previous answer. Mocked locator tests are useful for polling
  behavior, but they are not sufficient for validating grouping.

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
- Existing Browser Provider happy path still works in CDP attach mode.
- Managed mode remains compatible with the shared extractor and passes the same
  extraction tests as CDP attach mode.
- The design of the Gemini page can drift without immediately forcing a blind
  selector tweak; diagnostics should identify what changed.
