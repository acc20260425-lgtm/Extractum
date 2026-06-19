# YouTube Pipeline Research

This directory contains a local Python research prototype for comparing
YouTube summary LLM pipeline strategies.

It reads local transcript files and writes run artifacts under
`research/youtube_pipeline/runs/`.

For the boundary between the legacy direct-LLM runner and the newer
file-backed agentic workflow, see
`research/youtube_pipeline/RUNNER_AND_AGENTIC_WORKFLOW.md`.

## Environment

Set these variables for an OpenAI-compatible chat completions endpoint:

```powershell
$env:YOUTUBE_RESEARCH_LLM_BASE_URL = "https://api.openai.com/v1"
$env:YOUTUBE_RESEARCH_LLM_API_KEY = "..."
$env:YOUTUBE_RESEARCH_LLM_MODEL = "..."
```

For the current local Omniroute profile used by Extractum:

```powershell
$env:YOUTUBE_RESEARCH_LLM_BASE_URL = "http://localhost:20128/v1"
$env:YOUTUBE_RESEARCH_LLM_MODEL = "gemini/gemini-3.1-flash-lite-preview"
$env:YOUTUBE_RESEARCH_LLM_API_KEY = "..."
```

`YOUTUBE_RESEARCH_LLM_API_KEY` is required even for the local endpoint. Use the
API key accepted by that endpoint; do not commit it or write it into run
artifacts.

## Run One Strategy

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/a9_k-meLQaYP5Y_en_orig.txt `
  --video-id a9_k-meLQaYP5Y `
  --strategy two_pass_summary_structure `
  --output-language ru `
  --max-tokens 8192
```

## Available Local Transcripts

```text
research/youtube_pipeline/inputs/a9_k-meLQaYP5Y_en_orig.txt
research/youtube_pipeline/inputs/tucker_carlson_f_lRdkH_QoY_en.txt
research/youtube_pipeline/inputs/ai_monk_A8_nNYLTXEQ_en_orig.txt
```

## Ready-To-Run Examples

A9:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/a9_k-meLQaYP5Y_en_orig.txt `
  --video-id a9_k-meLQaYP5Y `
  --strategy two_pass_summary_structure `
  --output-language ru `
  --max-tokens 8192
```

Tucker Carlson:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/tucker_carlson_f_lRdkH_QoY_en.txt `
  --video-id tucker_carlson_f_lRdkH_QoY `
  --strategy two_pass_summary_structure `
  --output-language ru `
  --max-tokens 8192
```

Adaptive book report for a very long transcript:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/tucker_carlson_f_lRdkH_QoY_en.txt `
  --video-id f_lRdkH_QoY `
  --strategy adaptive_book_report `
  --output-language ru `
  --target-depth auto `
  --chunk-token-limit 3000 `
  --chapter-target-words 900 `
  --max-tokens 8192
```

MoC-guided map reduce for a very long transcript:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/tucker_carlson_f_lRdkH_QoY_en.txt `
  --video-id f_lRdkH_QoY `
  --strategy moc_guided_map_reduce `
  --output-language ru `
  --max-tokens 8192 `
  --chunk-token-limit 3000 `
  --chunk-overlap-tokens 700 `
  --planner-context-token-limit 120000 `
  --max-slice-tokens 8000
```

## Agentic MoC Skills Workflow

The agentic workflow is skill-first and file-backed. It does not add a normal
`runner.py --strategy` entry point in v1. For normal use, start with the public
`youtube-summary` skill.

### Public Wrapper Skill

For normal use, ask Codex:

```text
Use skill youtube-summary.
Transcript file: research/youtube_pipeline/inputs/example.txt
Write a long Russian summary, about 10000 words.
```

The `youtube-summary` skill creates or resumes a run, prepares transcript
chunks, creates map assignments, dispatches child skills, validates artifacts,
updates `workflow_state.json`, and returns `final/report.md`. The user should
not manually run the deterministic Python commands during normal use.

Map extraction still requires sub-agents. If sub-agents are unavailable before
map extraction, the skill pauses instead of replacing extractor work with direct
LLM API calls or hidden main-agent reasoning.

`youtube-long-report` remains as a lower-level/manual orchestrator contract for
older research notes and child-skill documentation. Prefer `youtube-summary`
for user-facing runs because it owns run creation/resume state and the
`workflow_state.json` stage chain.

The underlying bootstrap command is:

```powershell
python -m research.youtube_pipeline.tools.start_youtube_summary `
  --transcript <path> `
  --language ru `
  --target-words 10000
```

After a deterministic gate succeeds, advance run state with:

```powershell
python -m research.youtube_pipeline.tools.advance_youtube_summary_state `
  --run-dir <run-dir> `
  --after <step>
```

The accepted stage after map artifact assembly is `map_assembled`, not
`map_artifacts_ready`.

Minimal deterministic validation:

```powershell
python -m research.youtube_pipeline.tools.prep_all `
  --transcript research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt `
  --out research/youtube_pipeline/runs/manual/agentic_smoke `
  --language ru `
  --target-tokens 160 `
  --overlap-tokens 30

python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

### Additional Direct-LLM Runner Example

AI monk with the legacy runner:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/ai_monk_A8_nNYLTXEQ_en_orig.txt `
  --video-id ai_monk_A8_nNYLTXEQ `
  --strategy two_pass_summary_structure `
  --output-language ru `
  --max-tokens 8192
```

Available strategies:

```text
adaptive_book_report
antigravity_chunk_map_reduce
moc_guided_map_reduce
one_shot_full_json
one_shot_markdown_plus_json
two_pass_summary_structure
chunk_map_reduce
```

## Adaptive Book Report Flags

- `--target-depth auto|brief|standard|deep|book`: controls the report budget multiplier.
- `--min-report-words`: optional lower override for report budget.
- `--max-report-words`: optional upper override for report budget.
- `--chapter-target-words`: target words used to derive chapter count; default is `900`.
- `--chunk-token-limit`: approximate chunk size used by chunked strategies; default is `3000`.

For Russian output, the strategy uses a larger output-token budget for chapter
generation and expansion because Cyrillic text usually takes more tokens per
word than English.

## Run Tests

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```
