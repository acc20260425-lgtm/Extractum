# YouTube Pipeline Research

This directory contains a local Python research prototype for comparing
YouTube summary LLM pipeline strategies.

It reads local transcript files and writes run artifacts under
`research/youtube_pipeline/runs/`.

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

AI monk:

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
one_shot_full_json
one_shot_markdown_plus_json
two_pass_summary_structure
chunk_map_reduce
timeline_segment_reduce
```

## Run Tests

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```
