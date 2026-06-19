# Runner and Agentic Workflow

This note explains the boundary between the legacy strategy runner in
`research/youtube_pipeline/runner.py` and the newer file-backed agentic YouTube
summary workflow.

## Short Version

There are two different execution paths in this research package:

- `runner.py` runs direct LLM research strategies through an
  OpenAI-compatible chat completions endpoint.
- The agentic workflow is orchestrated by Codex skills and deterministic Python
  tools. Python does not call LLM APIs for reasoning in this path.

Use `runner.py` for legacy strategy experiments. Use the `youtube-summary` skill
for the long-report agentic workflow.

## `runner.py`

`research/youtube_pipeline/runner.py` is a command-line entry point for local
strategy experiments. It reads one transcript file, builds an
`OpenAICompatibleClient` from environment variables, runs one strategy from
`research.youtube_pipeline.strategies.STRATEGIES`, and writes run artifacts.

Typical command:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/example.txt `
  --video-id example_video `
  --strategy adaptive_book_report `
  --output-language ru `
  --max-tokens 8192
```

Required environment variables:

```powershell
$env:YOUTUBE_RESEARCH_LLM_BASE_URL = "http://localhost:20128/v1"
$env:YOUTUBE_RESEARCH_LLM_API_KEY = "..."
$env:YOUTUBE_RESEARCH_LLM_MODEL = "gemini/gemini-3.1-flash-lite-preview"
```

The runner writes artifacts under:

```text
<output-root>/<strategy>/<video-id>/
```

Default output root:

```text
research/youtube_pipeline/runs/manual
```

Core artifacts:

```text
result.json
result.md
metrics.json
raw_requests.jsonl
raw_responses.jsonl
```

Strategies may also emit extra artifacts. Extra artifact filenames must be
simple relative names and cannot collide with the reserved core artifact names,
including case variants such as `RESULT.JSON`.

## Runner Responsibilities

`runner.py` owns only the thin CLI and persistence wrapper:

- parse CLI flags;
- validate compatible report word bounds;
- read the transcript as UTF-8;
- create the OpenAI-compatible client from environment variables;
- build `StrategyOptions`;
- call the selected strategy function;
- write result, metrics, raw request/response logs, and extra artifacts.

The runner does not implement summarization logic itself. Summarization behavior
lives in strategy functions, prompt builders, model classes, and helper modules.

## When To Use `runner.py`

Use `runner.py` when you want to compare or reproduce direct LLM strategy
experiments, for example:

- `one_shot_full_json`
- `two_pass_summary_structure`
- `chunk_map_reduce`
- `adaptive_book_report`
- `moc_guided_map_reduce`

This path is useful for research runs where each strategy owns its LLM calls and
returns a `StrategyOutcome`.

Do not use `runner.py` as the entry point for the agentic workflow. The agentic
workflow deliberately does not add a normal `runner.py --strategy` mode in v1.

## Agentic Workflow

The newer agentic workflow is skill-first and file-backed. Its public entry
point is the project skill:

```text
youtube-summary
```

The workflow exists to produce long, detailed reports from YouTube transcripts
without relying on one huge final LLM call. It breaks the work into files and
stage gates:

```text
transcript
  -> prep/chunks
  -> map/assignments
  -> map extractor sub-agents write map/agent_outputs
  -> validate map outputs
  -> assemble map artifacts
  -> build planner context
  -> MoC planning skill writes planning/moc.raw.json
  -> validate MoC
  -> dedupe and align facts
  -> prepare section assignments
  -> section writer skills write section files
  -> validate generated files
  -> QA
  -> structured analysis
  -> final report assembly
```

Python owns deterministic mechanics in this path. Codex skills and sub-agents
own reasoning work.

## Agentic No-LLM-API Rule

In the agentic path, Python must not call OpenAI, Anthropic, Gemini, Omniroute,
or other LLM APIs for reasoning tasks.

Reasoning tasks include:

- map extraction;
- MoC planning;
- section writing;
- QA judgment;
- final report rewriting.

Those tasks are performed by Codex skills and sub-agents that read assignment
files and write output files. Python may prepare inputs, validate schemas,
repair lightweight JSON wrapping, dedupe facts, align facts, compute metrics,
and assemble already-written sections.

## Main Agentic Files

Public wrapper skill:

```text
.agents/skills/youtube-summary/SKILL.md
```

Lower-level/manual orchestration contract retained for older research notes and
child-skill documentation:

```text
.agents/skills/youtube-long-report/SKILL.md
```

Child skills:

```text
.agents/skills/youtube-map-extract/SKILL.md
.agents/skills/youtube-moc-planning/SKILL.md
.agents/skills/youtube-section-reduce/SKILL.md
.agents/skills/youtube-report-qa/SKILL.md
```

State helpers:

```text
research/youtube_pipeline/youtube_summary_workflow.py
```

Deterministic artifact helpers:

```text
research/youtube_pipeline/moc_agentic.py
```

Primary tool modules:

```text
research/youtube_pipeline/tools/start_youtube_summary.py
research/youtube_pipeline/tools/advance_youtube_summary_state.py
research/youtube_pipeline/tools/validate_map_outputs.py
research/youtube_pipeline/tools/assemble_map_artifacts.py
research/youtube_pipeline/tools/build_planner_context.py
research/youtube_pipeline/tools/validate_moc.py
research/youtube_pipeline/tools/dedupe_facts.py
research/youtube_pipeline/tools/align_facts.py
research/youtube_pipeline/tools/prepare_section_assignments.py
research/youtube_pipeline/tools/validate_generated_files.py
research/youtube_pipeline/tools/quality_check.py
research/youtube_pipeline/tools/build_structured_analysis.py
research/youtube_pipeline/tools/assemble_report.py
```

## Agentic State

Each agentic run has:

```text
<run-dir>/workflow_state.json
```

Default run root:

```text
research/youtube_pipeline/runs/manual/youtube_summary_agentic
```

Run index:

```text
research/youtube_pipeline/runs/manual/youtube_summary_agentic/run_index.json
```

Accepted workflow stages:

```text
map_assignments_ready
map_outputs_ready
map_assembled
planner_context_ready
moc_ready
alignment_ready
sections_ready
qa_ready
final_ready
```

Important: the accepted stage after map artifact assembly is
`map_assembled`, not `map_artifacts_ready`.

## Agentic Commands

Bootstrap or resume through the public skill during normal use. The underlying
bootstrap command is:

```powershell
python -m research.youtube_pipeline.tools.start_youtube_summary `
  --transcript <path> `
  --language ru `
  --target-words 10000
```

Use `--run-dir <path>` to resume an explicit run. Use `--force` only when a
fresh run is desired instead of reusing a matching run.

After deterministic gates, prefer the state advance command:

```powershell
python -m research.youtube_pipeline.tools.advance_youtube_summary_state `
  --run-dir <run-dir> `
  --after <step>
```

Supported `--after` values:

```text
validate_map_outputs
assemble_map_artifacts
build_planner_context
validate_moc
prepare_section_assignments
validate_generated_files
quality_check
assemble_report
```

Manual state updates are available through
`update_youtube_summary_state.py`, but `advance_youtube_summary_state.py` is the
preferred command because it updates stage, next action, artifacts, counts, and
warnings from the gate output.

## Output Differences

`runner.py` output is strategy-oriented:

```text
runs/manual/<strategy>/<video-id>/result.md
runs/manual/<strategy>/<video-id>/metrics.json
runs/manual/<strategy>/<video-id>/raw_requests.jsonl
runs/manual/<strategy>/<video-id>/raw_responses.jsonl
```

Agentic output is run-state-oriented:

```text
<run-dir>/workflow_state.json
<run-dir>/prep/
<run-dir>/map/
<run-dir>/planning/
<run-dir>/alignment/
<run-dir>/sections/
<run-dir>/review/
<run-dir>/final/report.md
<run-dir>/final/metrics.json
<run-dir>/final/result.json
```

The agentic final report is assembled from section files. Avoid a final
whole-report LLM rewrite because it tends to compress details and discard source
coverage.

## Choosing The Right Path

Use `runner.py` when:

- the experiment is one of the direct strategies in `STRATEGIES`;
- raw LLM request/response logs are expected;
- the strategy itself owns all LLM calls;
- a compact reproducible strategy run is enough.

Use `youtube-summary` when:

- the user wants a long, detailed report from a YouTube transcript;
- output-token limits make one-shot or direct map-reduce quality collapse;
- the report should be assembled from validated files;
- sub-agents can perform map extraction and section writing through file
  contracts;
- Python must stay deterministic and avoid direct LLM API reasoning calls.

Use `youtube-long-report` only when working directly with the older manual
orchestrator contract. It is not the preferred public entry point because it
does not own the resume-oriented `workflow_state.json` wrapper flow that
`youtube-summary` provides.

## Testing

Runner-focused tests:

```powershell
python -m unittest research.youtube_pipeline.tests.test_runner
```

Agentic workflow tests:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

Full YouTube pipeline tests:

```powershell
python -m unittest discover research/youtube_pipeline/tests
```
