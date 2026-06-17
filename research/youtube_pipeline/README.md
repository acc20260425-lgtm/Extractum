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

## Run One Strategy

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/video_long.txt `
  --video-id video_long `
  --strategy two_pass_summary_structure `
  --output-language ru `
  --max-tokens 8192
```

## Run Tests

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```
