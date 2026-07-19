# Crate Extraction Shell-Cap Revision Implementation Plan

> **Status: COMPLETED HISTORICAL PLAN; DO NOT RE-EXECUTE.** Its policy was
> superseded on 2026-07-19 by the advisory timing rules in
> `docs/superpowers/specs/2026-07-17-crate-roadmap.md`. The planned Phase 3
> follow-on was canceled before completion and never retained; see its
> [cancellation disposition](../verification/2026-07-19-extractum-process-reapplication-cancellation.md).
> The body below records the policy implementation as originally approved; it
> is not current execution authority.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the approved 2,000 ms / 20% per-slice shell cap, 15,000 ms cumulative ceiling, stability rule, and v2/v3 disposition the enforced repository policy for future crate extractions.

**Architecture:** Keep numeric policy in the focused-loop specification, roadmap state in the crate roadmap, and the obsolete anomaly track in its preserved v2 design. Add one focused Vitest source contract that reads those Markdown authorities as raw text and protects only decision-significant tokens; do not modify product code, Cargo manifests, historical verification records, or the diagnostic harness.

**Tech Stack:** Markdown policy documents, TypeScript, Vitest 4 raw-text imports, Git.

## Global Constraints

- A shell result passes only when its median regression is both `<= 2,000 ms` and `<= 20%`; equality passes.
- The canonical cumulative anchor is the pre-Phase 3 median `9,135 ms`. The valid post-slice shell median must also be `<= 15,000 ms`; crossing that cumulative ceiling requires a new owner-approved policy revision.
- Baseline and candidate series each contain five recorded samples and are valid only when at least four samples are within `<= 300 ms` of their own median.
- An unstable series invalidates the measurement session; it is not a performance failure.
- The old 800 ms / 8% marginal-performance repeat is removed. Infrastructure and measurement-validity retries remain distinct and allowed after their preconditions pass.
- The focused-domain retention gate remains both `>= 25%` and `>= 2.0 seconds` improvement.
- The v2/v3 anomaly track is `moot` for the current crate roadmap. Its technical design stays preserved, and the reviewed v1 harness remains non-production-ready.
- Historical plans, frozen protocols, raw artifacts, verification documents, and `scripts/process-shell-diagnostic/` remain unchanged.
- This plan does not reapply `b364756c`, change Rust, run a performance experiment, or unblock Phase 4. A separate Phase 3 plan owns those actions.
- Execute from a clean isolated worktree created through `superpowers:using-git-worktrees`; do not reuse any preserved v1 measurement worktree.
- Use `npm.cmd`, preserve unrelated worktree changes, and stage only the files named by each task.

---

## File Map

- `src/lib/crate-extraction-shell-cap-contract.test.ts` — new machine-significant contract for the numeric policy and roadmap/v2 disposition.
- `docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md` — current sampling, validity, per-slice, cumulative, and failure-classification authority.
- `docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md` — active Phase 3/4 architecture with a header callout superseding only its obsolete performance clauses.
- `docs/superpowers/specs/2026-07-17-crate-roadmap.md` — cumulative ledger plus Phase 3, Phase 4, and v2/v3 roadmap disposition.
- `docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md` — preserved technical design with a short current-roadmap `moot` header.
- `docs/value-registry.md` — registers the document-only `moot` roadmap disposition and its non-product impact.
- `docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md` — approved source requirement; Task 2 marks it implemented and current after every authority is aligned.

## Rust Verification Loops

This policy slice changes no Rust source, Cargo manifest, package boundary, or
workspace member. It therefore has no Rust RED/GREEN loop and must not warm or
mutate `src-tauri/target`. The later Phase 3 reapplication plan must define the
affected packages, exact RED/GREEN tests, focused checks, package checkpoints,
and all end-of-slice workspace gates required by `AGENTS.md`.

### Task 1: Enforce the revised focused-loop measurement policy

**Files:**

- Create: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md:1-270`
- Modify: `docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md:1-12`
- Read only: `docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md`

**Interfaces:**

- Consumes: the approved values `2,000 ms`, `20%`, `15,000 ms`, five samples, `4/5` within `300 ms`, and no marginal-performance repeat.
- Produces: a focused-loop Markdown authority and one Vitest contract that Task 2 extends with roadmap assertions.

- [ ] **Step 1: Write the focused-policy RED contract**

Create `src/lib/crate-extraction-shell-cap-contract.test.ts` with exactly this initial content:

```ts
import { describe, expect, it } from "vitest";

import focusedLoopDesignRaw from "../../docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md?raw";
import processBoundaryDesignRaw from "../../docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md?raw";

const normalize = (value: string) => value.replace(/\r\n/g, "\n");
const sectionBetween = (value: string, start: string, end: string) => {
  const startIndex = value.indexOf(start);
  const endIndex = value.indexOf(end, startIndex + start.length);
  if (startIndex < 0 || endIndex < 0) {
    throw new Error(`Missing policy section: ${start} -> ${end}`);
  }
  return value.slice(startIndex, endIndex);
};
const focusedLoopDesign = normalize(focusedLoopDesignRaw);
const processBoundaryDesign = normalize(processBoundaryDesignRaw);
const samplingPolicy = sectionBetween(
  focusedLoopDesign,
  "### Sampling",
  "### Retention gates",
);
const retentionPolicy = sectionBetween(
  focusedLoopDesign,
  "### Retention gates",
  "## Failure Classification",
);
const failurePolicy = sectionBetween(
  focusedLoopDesign,
  "## Failure Classification",
  "## Repository Enforcement",
);

describe("crate extraction shell-cap repository policy", () => {
  it("pins the revised focused-loop thresholds and validity rule", () => {
    expect(focusedLoopDesign).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(retentionPolicy).toContain("no more than both");
    expect(retentionPolicy).toContain("- 20%; and");
    expect(retentionPolicy).toContain("- 2,000 ms in absolute median wall time");
    expect(retentionPolicy).toContain("Values exactly at 2,000 ms / 20% pass");
    expect(retentionPolicy).toContain("9,135 ms");
    expect(retentionPolicy).toContain("15,000 ms");
    expect(retentionPolicy).toContain("no marginal-performance repeat");
    expect(retentionPolicy).toContain("exact Phase 3 reapplication");
    expect(retentionPolicy).toContain("cannot produce a performance no-go");
    expect(samplingPolicy).toContain("at least four of the five samples");
    expect(samplingPolicy).toContain("within 300 ms of its own median");
    expect(samplingPolicy).toContain("not a performance failure");
    expect(samplingPolicy).toContain("quiet-window preflight");
    expect(failurePolicy).toContain("Measurement invalidation");
    expect(failurePolicy).toContain("15,000 ms cumulative shell ceiling");
    expect(focusedLoopDesign).toContain(
      "2026-07-17-process-and-gemini-browser-crate-boundary-design.md",
    );
    expect(focusedLoopDesign).toContain(
      "shell-cap and marginal-repeat clauses",
    );
    expect(retentionPolicy).toContain("at least 25%");
    expect(retentionPolicy).toContain("at least 2.0 seconds");
    expect(processBoundaryDesign).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(processBoundaryDesign).toContain("architecture and correctness");
    expect(processBoundaryDesign).toContain("requirements remain active");
  });
});
```

- [ ] **Step 2: Run the new contract and verify RED**

Run:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
```

Expected: one test is collected and fails because the focused-loop design does
not yet link the revision or contain the 2,000 ms / 20%, 15,000 ms, and `4/5`
policy.

- [ ] **Step 3: Update the focused-loop authority**

After the date at the top of
`docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md`, insert:

```markdown
**Current extraction-performance authority:**
[`2026-07-18-crate-extraction-shell-cap-revision-design.md`](2026-07-18-crate-extraction-shell-cap-revision-design.md)

The revision applies prospectively and to the approved exact Phase 3
reapplication. Historical measurements and decisions remain governed by the
thresholds frozen for their original sessions.

This authority supersedes only the shell-cap and marginal-repeat clauses in
`2026-07-17-process-and-gemini-browser-crate-boundary-design.md`. That design's
boundary, facade, dependency, correctness, and restoration requirements remain
active for the exact Phase 3 reapplication.
```

In `## Implementation-Plan Contract`, replace the crate-extraction-specific
bullet list with:

```markdown
For a crate-extraction plan, the section must additionally name:

- the pre-extraction command against `-p extractum`;
- the post-extraction command against the new package;
- the matched logical probe source before and after the move;
- the focused improvement gate and 2,000 ms / 20% per-slice shell cap;
- the canonical 9,135 ms anchor and 15,000 ms cumulative shell ceiling;
- the five-sample, four-within-300-ms validity rule;
- the measurement artifact, invalid-session path, and negative-outcome path.
```

In `### Sampling`, replace the seven-item baseline/candidate list and the
paragraph immediately after it with:

```markdown
Baseline and candidate measurements each use:

1. no active `cargo`, `rustc`, `rust-analyzer`, Tauri, or competing build
   process;
2. recorded Rust/Cargo versions, power profile, and Defender state;
3. one discarded warm-up;
4. five recorded samples;
5. the median of the five samples;
6. the shared canonical target directory;
7. byte-for-byte probe restoration plus SHA-256 verification after every
   sample;
8. at least four of the five samples within 300 ms of its own median.

An unstable baseline or candidate series invalidates the complete measurement
session. It is not a performance failure, and none of its medians may be used
for retention. A fresh session may start only after the quiet-window preflight
passes again.

Thresholds are fixed before candidate measurements and are never adjusted in
response to observed results.
```

Replace the application-shell part of `### Retention gates` through the
paragraph before `## Failure Classification` with:

```markdown
The application-shell probe may regress by no more than both:

- 20%; and
- 2,000 ms in absolute median wall time.

The valid post-slice application-shell median must also be no more than
15,000 ms against the canonical pre-Phase 3 anchor of 9,135 ms. Crossing that
cumulative roadmap ceiling blocks automatic retention and requires a new
owner-approved policy revision. Unused cumulative budget does not raise the
per-slice 2,000 ms / 20% cap.

The shell probe uses an inert edit in a source file retained by `extractum`.
It is measured before and after extraction with the full-workspace check,
because it exists to detect a cost imposed on ordinary application work.

Passing only one side of either paired threshold is insufficient. Values
exactly at 2,000 ms / 20% pass. There is no marginal-performance repeat: a
valid result passes or fails directly. Measurement invalidation and corrected
infrastructure retries remain separate from performance classification.

The exact Phase 3 reapplication is the single exception to a new gating shell
decision: its already-valid historical 10,177 ms result was accepted by the
owner revision. Fresh post-reapplication samples are non-gating ledger
diagnostics and cannot produce a performance no-go for that exact candidate.
This exception does not apply when reconstructed bytes differ materially from
`b364756c`; that case is a new candidate with fresh preregistered timing.
```

In `## Failure Classification`, insert this bullet after infrastructure
failure:

```markdown
- **Measurement invalidation:** fewer than four of five values in either shell
  series are within 300 ms of that series median. Discard the complete session,
  classify no performance result, re-establish the quiet window, and start a
  fresh session from its warm-up.
```

Replace the existing performance-no-go bullet with:

```markdown
- **Performance no-go:** except for the exact Phase 3 reapplication described
  above, the correct candidate misses either focused-domain improvement
  threshold, either side of the 2,000 ms / 20% per-slice shell cap, or the
  15,000 ms cumulative shell ceiling. Record the negative result and follow
  the phase's already-approved retain/revert branch. A cumulative ceiling
  crossing requires a separate owner policy revision before retention.
```

In `## Repository Enforcement`, add the new contract to the invariant list:

```markdown
- the crate-extraction shell contract pins the current per-slice cap,
  cumulative ceiling, stability rule, and absence of a marginal-performance
  repeat.
```

In `## Scope`, add
`src/lib/crate-extraction-shell-cap-contract.test.ts` to the allowed policy
files and state explicitly that this revision changes no Rust source, Cargo
manifest, product behavior, or historical verification record.

After the date in
`docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md`,
insert only this callout; do not edit its historical body:

```markdown
**Current extraction-performance policy:**
[`2026-07-18-crate-extraction-shell-cap-revision-design.md`](2026-07-18-crate-extraction-shell-cap-revision-design.md)

The revision supersedes this document's shell-cap, cumulative/validity, and
marginal-repeat clauses. This document's architecture and correctness
requirements remain active for the exact Phase 3 reapplication and Phase 4
boundary.
```

- [ ] **Step 4: Run focused GREEN and TypeScript/Svelte checking**

Run:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Focused shell-cap contract failed.' }
npm.cmd run check
if ($LASTEXITCODE -ne 0) { throw 'TypeScript/Svelte check failed.' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 1 whitespace check failed.' }
```

Expected: the contract reports one passing test, `npm.cmd run check` exits
zero, and `git diff --check` prints no errors.

- [ ] **Step 5: Commit the focused policy and its contract**

Run:

```powershell
git status --short
git add -- src/lib/crate-extraction-shell-cap-contract.test.ts docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md
if ($LASTEXITCODE -ne 0) { throw 'Could not stage Task 1 policy files.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Task 1 staged whitespace check failed.' }
git diff --cached --name-only
git commit -m "docs: enforce revised crate shell cap"
if ($LASTEXITCODE -ne 0) { throw 'Task 1 commit failed.' }
```

Expected staged inventory before the commit:

```text
docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md
docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md
src/lib/crate-extraction-shell-cap-contract.test.ts
```

### Task 2: Record the roadmap and anomaly-track disposition

**Files:**

- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md:24-360`
- Modify: `docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md:1-30`
- Modify: `docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md:1-5`
- Modify: `docs/value-registry.md:926-974`

**Interfaces:**

- Consumes: Task 1's normalized raw-Markdown contract and focused-loop policy.
- Produces: the active cumulative ledger, exact Phase 3 reapplication state,
  Phase 4 block condition, formal `moot` v2/v3 disposition, and registered
  document-only value.

- [ ] **Step 1: Extend the contract with roadmap RED assertions**

Add these imports below the existing focused-loop import:

```ts
import crateRoadmapRaw from "../../docs/superpowers/specs/2026-07-17-crate-roadmap.md?raw";
import shellCapRevisionRaw from "../../docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md?raw";
import anomalyV2DesignRaw from "../../docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md?raw";
import valueRegistryRaw from "../../docs/value-registry.md?raw";
```

Add these normalized constants below `focusedLoopDesign`:

```ts
const crateRoadmap = normalize(crateRoadmapRaw);
const shellCapRevision = normalize(shellCapRevisionRaw);
const anomalyV2Design = normalize(anomalyV2DesignRaw);
const valueRegistry = normalize(valueRegistryRaw);
```

Append this test inside the existing `describe` block:

```ts
  it("records the cumulative roadmap and moot anomaly disposition", () => {
    expect(crateRoadmap).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(crateRoadmap).toContain("15,000 ms");
    expect(crateRoadmap).toContain("5,865 ms");
    expect(crateRoadmap).toContain(
      "| Reapplied Phase 3 | pending valid post-reapplication median |",
    );
    expect(crateRoadmap).toContain(
      "Phase 3 — `extractum-process` (approved for exact-candidate reapplication)",
    );
    expect(crateRoadmap).toContain("non-gating before/after");
    expect(crateRoadmap).toContain("shell samples and validity counts");
    expect(crateRoadmap).toContain(
      "If reconstruction differs materially from `b364756c`",
    );
    expect(crateRoadmap).toContain(
      "requires a separately approved plan",
    );
    expect(crateRoadmap).toContain("fresh preregistered timing under the revised");
    expect(crateRoadmap).toContain(
      "Phase 4 remains blocked until the exact Phase 3 candidate is integrated",
    );
    expect(anomalyV2Design).toContain(
      "**Status:** `moot` for the current crate roadmap",
    );
    expect(anomalyV2Design).toContain(
      "2026-07-18-crate-extraction-shell-cap-revision-design.md",
    );
    expect(anomalyV2Design).toContain("current v1 harness is not");
    expect(anomalyV2Design).toContain("production-ready infrastructure");
    expect(valueRegistry).toContain("| `moot` | roadmap disposition |");
    expect(valueRegistry).toContain("`moot` is documentation-only");
    expect(shellCapRevision).toContain(
      "**Status:** Implemented; current shell-cap authority",
    );
  });
```

- [ ] **Step 2: Run the expanded contract and verify the new test is RED**

Run:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
```

Expected: two tests are collected; the Task 1 focused-policy test passes and
the new roadmap/disposition test fails on the first missing roadmap assertion.

- [ ] **Step 3: Mark v2/v3 moot without rewriting its technical design**

In `docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md`,
replace the status line with:

```markdown
**Status:** `moot` for the current crate roadmap; preserved historical design
```

After the date, insert:

```markdown
## Current Roadmap Disposition

The owner decision in
[`2026-07-18-crate-extraction-shell-cap-revision-design.md`](2026-07-18-crate-extraction-shell-cap-revision-design.md)
accepts the observed Phase 3 shell cost. V2 and a possible v3 no longer control
Phase 3 retention or Phase 4 entry and must not run as roadmap prerequisites.

This technical design remains preserved for a future owner-approved task that
genuinely requires sub-second precision or causal attribution. Its reviewed v1
harness remediation is deferred with it; the current v1 harness is not
production-ready infrastructure for another protocol.
```

Do not edit any section below this new disposition block.

- [ ] **Step 4: Register the document-only `moot` value**

In the representative-source list for
`## Process-shell diagnostic artifact classifications`, add:

```markdown
- `docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md`
```

Add this row to that section's table:

```markdown
| `moot` | roadmap disposition | Moot | The approved anomaly protocol no longer controls the crate roadmap after an explicit owner policy revision; its design remains preserved for a separately approved precision/causality task. | shell-cap revision / crate roadmap | terminal | none | n/a | yes | v2 design, crate roadmap |
```

Immediately after the table, add:

```markdown
`moot` is documentation-only. It is not persisted in SQLite, exposed through a
product API, rendered in the UI, or used by product fixtures. The diagnostic
harness continues to own only its historical artifact classifications.
```

In
`docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md`,
replace the status line with:

```markdown
**Status:** Implemented; current shell-cap authority
```

- [ ] **Step 5: Update the crate roadmap authority and cumulative ledger**

In the list introduced by `Completed and in-flight slices governed by their
own documents:`, insert this item immediately after the focused-loop design:

```markdown
- [`2026-07-18-crate-extraction-shell-cap-revision-design.md`](2026-07-18-crate-extraction-shell-cap-revision-design.md)
  — current 2,000 ms / 20% per-slice cap, 15,000 ms cumulative ceiling,
  measurement-validity rule, and Phase 3/v2 disposition.
```

In `## Decision Framework`, replace references that make the 2026-07-17
thresholds current with:

```markdown
- For decisions after 2026-07-18, hot-module retention and shell-regression
  gates come from `2026-07-18-crate-extraction-shell-cap-revision-design.md`
  and the updated focused-loop specification. Historical sessions retain their
  originally frozen thresholds.
- Every slice must pass both its 2,000 ms / 20% local shell cap and the
  15,000 ms cumulative shell ceiling. A ceiling crossing requires a separate
  owner-approved policy revision before retention.
```

Before `## Target Crate Map`, add this exact ledger:

```markdown
## Roadmap Shell Budget

The cumulative ceiling is measured on the same workstation/probe family as the
canonical pre-Phase 3 shell median. Only valid series enter this ledger.

| Checkpoint | Valid shell median | Remaining to 15,000 ms | Disposition |
| --- | ---: | ---: | --- |
| Pre-Phase 3 | 9,135 ms | 5,865 ms | canonical anchor |
| Reapplied Phase 3 | pending valid post-reapplication median | pending | non-gating measurement required before Phase 4 timing |
```

Replace the complete Phase 3 section with:

```markdown
### Phase 3 — `extractum-process` (approved for exact-candidate reapplication)

`external_process`, `child_process`, and `process_tree` become shared
OS-process infrastructure below Gemini Browser, YouTube, and diagnostics.
`job_helpers` stays app-side. Candidate `b364756c` improved the focused median
from 9,171 ms to 2,049 ms and regressed the app-shell median from 9,135 ms to
10,177 ms (+1,042 ms / +11.41%). It correctly failed the original 500 ms / 5%
gate and was reverted in `c47372dc`; that historical result remains unchanged.

The approved 2026-07-18 shell-cap revision accepts the same evidence under the
new 2,000 ms / 20% rule. Reapply the exact historical candidate without a new
gating performance experiment, verify its historical tree/blob identity, and
rerun current correctness and completion gates. Record non-gating before/after
shell samples and validity counts; a valid post median seeds the cumulative
ledger. If those diagnostics are invalid, Phase 3 may still retain after all
correctness/completion gates pass, but Phase 4 must establish a valid baseline
before its own performance decision.

The v2/v3 anomaly track is moot for this roadmap. It is not a prerequisite for
Phase 3 or Phase 4, and its reviewed harness remediation remains deferred.

If reconstruction differs materially from `b364756c`, stop the exact-candidate
path. The result is a new candidate and requires a separately approved plan
with fresh preregistered timing under the revised local, cumulative, and
validity rules.
```

Replace the final two sentences of Phase 4 with:

```markdown
Phase 4 remains blocked until the exact Phase 3 candidate is integrated and a
valid shell baseline exists for Phase 4 measurement. No additional v2/v3
diagnostic approval is required.
```

Replace Standing Rule 2 completely with:

```markdown
2. Common measurement mechanics, rename-map inventory comparison, and
   negative-outcome documentation are inherited from the render and focused-
   loop specs. For Phase 4 and later, every shell decision uses five-sample
   baseline/candidate series, requires at least four values within 300 ms of
   each series median, and applies both the 2,000 ms / 20% per-slice cap and
   the 15,000 ms cumulative ceiling. An unstable series invalidates the
   session rather than failing the candidate. The exact historical Phase 3
   reapplication instead records non-gating diagnostics under its explicit
   owner exception.
```

Replace the deferred `Focused-loop metric respec` row with these rows:

```markdown
| Focused-loop metric respec | **Completed 2026-07-17; revised 2026-07-18**: focused commands and the domain gate remain; shell retention now uses 2,000 ms / 20%, the 15,000 ms cumulative ceiling, and the 4/5-within-300-ms validity rule. |
| Process-shell anomaly v2/v3 | **Moot for the current roadmap**: reopen only for a separately approved task requiring sub-second precision or causal attribution; v1 harness remediation remains deferred. |
```

- [ ] **Step 6: Run roadmap GREEN and documentation checks**

Run:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Roadmap shell-cap contract failed.' }
npm.cmd run check
if ($LASTEXITCODE -ne 0) { throw 'TypeScript/Svelte check failed.' }
git diff --check
if ($LASTEXITCODE -ne 0) { throw 'Task 2 whitespace check failed.' }
```

Expected: both shell-cap contract tests pass, `npm.cmd run check` exits zero,
and the whitespace check prints no errors.

- [ ] **Step 7: Commit only the roadmap disposition slice**

Run:

```powershell
git status --short
git add -- src/lib/crate-extraction-shell-cap-contract.test.ts docs/superpowers/specs/2026-07-17-crate-roadmap.md docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md docs/value-registry.md
if ($LASTEXITCODE -ne 0) { throw 'Could not stage Task 2 policy files.' }
git diff --cached --check
if ($LASTEXITCODE -ne 0) { throw 'Task 2 staged whitespace check failed.' }
git diff --cached --name-only
git commit -m "docs: reopen process extraction roadmap"
if ($LASTEXITCODE -ne 0) { throw 'Task 2 commit failed.' }
```

Expected staged inventory before the commit:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md
docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md
docs/value-registry.md
src/lib/crate-extraction-shell-cap-contract.test.ts
```

### Task 3: Run completion gates and protect historical evidence

**Files:**

- Verify only: all Task 1 and Task 2 files
- Must remain unchanged: `scripts/process-shell-diagnostic/`, historical extraction/diagnostic plans, and historical verification records

**Interfaces:**

- Consumes: the two committed policy slices.
- Produces: completion evidence and a review-ready branch for the later Phase 3 planning session.

- [ ] **Step 1: Invoke verification-before-completion and run the focused policy suite**

Run:

```powershell
npm.cmd run test -- src/lib/focused-rust-loop-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Focused policy suite failed.' }
```

Expected: two files and five tests pass; a zero-test result is failure.

- [ ] **Step 2: Run repository TypeScript and full completion gates**

Run:

```powershell
npm.cmd run check
if ($LASTEXITCODE -ne 0) { throw 'TypeScript/Svelte completion check failed.' }
npm.cmd run verify
if ($LASTEXITCODE -ne 0) { throw 'Repository verification failed.' }
```

Expected: both commands exit zero. Do not claim success from the focused test
alone.

- [ ] **Step 3: Verify scope and historical immutability**

Run:

```powershell
git diff --check HEAD~2..HEAD
if ($LASTEXITCODE -ne 0) { throw 'Policy commit range has whitespace errors.' }
git diff --quiet HEAD~2..HEAD -- scripts/process-shell-diagnostic docs/superpowers/plans/2026-07-17-extractum-process-extraction.md docs/superpowers/plans/2026-07-18-process-shell-regression-diagnostic.md docs/superpowers/verification/2026-07-17-extractum-process-extraction.md docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md
if ($LASTEXITCODE -ne 0) { throw 'Historical diagnostic or harness bytes changed.' }
$shellCapExpectedPaths = @(
  'docs/superpowers/specs/2026-07-17-crate-roadmap.md'
  'docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md'
  'docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md'
  'docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md'
  'docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md'
  'docs/value-registry.md'
  'src/lib/crate-extraction-shell-cap-contract.test.ts'
)
$shellCapActualPaths = @(git diff --name-only HEAD~2..HEAD)
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate the policy commit range.' }
$shellCapPathDelta = @(Compare-Object $shellCapExpectedPaths $shellCapActualPaths)
if ($shellCapPathDelta.Count -ne 0) {
  throw "Unexpected policy path inventory: $($shellCapPathDelta | Out-String)"
}
$shellCapStatus = @(git status --short)
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect final worktree status.' }
if ($shellCapStatus.Count -ne 0) {
  throw "Policy worktree is not clean: $($shellCapStatus -join '; ')"
}
$shellCapActualPaths
```

Expected: both diff checks exit zero, status is clean, and the complete changed
path inventory is exactly:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-17-focused-rust-loop-design.md
docs/superpowers/specs/2026-07-17-process-and-gemini-browser-crate-boundary-design.md
docs/superpowers/specs/2026-07-18-crate-extraction-shell-cap-revision-design.md
docs/superpowers/specs/2026-07-18-process-shell-anomaly-v2-design.md
docs/value-registry.md
src/lib/crate-extraction-shell-cap-contract.test.ts
```

- [ ] **Step 4: Request independent code review**

Use `superpowers:requesting-code-review` with the implementation base commit,
current HEAD, this plan, and the approved revision design. Require the reviewer
to check numeric semantics, the cumulative ledger, invalid-session handling,
v2/v3 disposition, value-registry impact, historical immutability, and test
coverage. Fix every valid Important issue before merge.

## Follow-On Boundary

After this plan is implemented and reviewed, write a separate
`docs/superpowers/plans/2026-07-18-extractum-process-reapplication.md`. That plan
must reconstruct `b364756c` from historical Git objects, prove exact tree/blob
identity, collect the approved non-gating shell diagnostics, update the
cumulative ledger, and include a complete `## Rust Verification Loops` section
for `extractum-process`, `extractum`, and the end-of-slice workspace gates.
It must stop and route to a new preregistered candidate plan if reconstructed
bytes differ materially from `b364756c`.
