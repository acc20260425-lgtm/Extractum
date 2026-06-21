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
names, candidate counts, text lengths, score components, and short hashes, but
not full text.

Candidate observations should include:

- selector;
- DOM order index;
- visible flag;
- text length;
- normalized text hash;
- whether it matched baseline;
- whether it looked like composer/user prompt/navigation text;
- parent/group identifier when grouping is possible.

### Grouping

The extractor should prefer complete assistant-turn containers. For selectors
such as `message-content`, it should look for the closest stable answer ancestor
and aggregate visible descendant text in DOM order. If no stable ancestor can be
identified, it may fall back to a single node, but diagnostics must mark that as
`grouping: "single_node"`.

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
  candidate count/signature stayed unchanged, and no known generation-busy UI was
  visible during the final quiet window.
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
- `completion_notes`.

The sidecar should keep waiting while generation-busy UI is visible, subject to a
hard maximum timeout. If the answer keeps growing, the stable timer resets. If a
new grouped candidate becomes better than the previous selected candidate, the
stable timer also resets.

## Artifacts And Diagnostics

Add a reduced answer extraction artifact for failure and partial cases:

- captured for `timeout`, `failed`, `browser_crashed`, and `timeout_latest`;
- written under the run artifact directory;
- contains no full prompt or answer text;
- contains selector names, candidate counts, lengths, hashes, completion reason,
  and sanitized error stage;
- safe to inspect locally and summarize in copied diagnostics.

Extend `debug_summary` with compact extraction facts:

```ts
interface GeminiBrowserAnswerExtractionDebug {
  raw_candidate_count: number;
  grouped_candidate_count: number;
  selected_candidate_length: number;
  selected_grouping: "assistant_turn" | "single_node" | "unknown";
  selected_score: number | null;
  top_candidate_lengths: number[];
  busy_visible_at_completion: boolean;
  last_growth_elapsed_ms: number | null;
}
```

This can be nested under the existing `debug_summary` to avoid broad result DTO
churn. Rust DTOs should mirror the optional fields and tolerate older run JSON
without them.

The inline run inspector should surface:

- raw/grouped candidate counts;
- selected grouping;
- selected length vs result text length;
- completion reason;
- busy at completion;
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

Add frontend/Rust tests only for DTO and UI display changes:

- Rust run-log round trips optional extraction debug fields.
- TypeScript DTOs accept missing extraction debug fields.
- Run inspector displays extraction facts when present and remains useful when
  absent.

Manual validation should use the known Russian football prompt that previously
made partial extraction visible. A successful manual run should show:

- `result_text_length === debug_final_text_length`;
- completion reason `stable` for a fully rendered answer;
- grouped candidate count greater than or equal to raw candidate count where
  split DOM appears;
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
- Existing managed-mode behavior is not intentionally changed.
- The design of the Gemini page can drift without immediately forcing a blind
  selector tweak; diagnostics should identify what changed.
