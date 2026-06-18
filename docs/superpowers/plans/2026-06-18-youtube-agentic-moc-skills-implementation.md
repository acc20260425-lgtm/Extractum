# YouTube Agentic MoC Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a file-backed agentic MoC workflow for long YouTube transcript reports where Python provides deterministic tooling and Codex skills/sub-agents perform all LLM reasoning, with no direct LLM API calls in the v1 agentic path.

**Architecture:** Add a new skill-first workflow beside the existing Python strategies. Python prepares transcript chunks, map assignments, manifests, validation reports, planner context, dedupe/alignment artifacts, structured analysis, report assembly, and quality metrics. Project-local Codex skills orchestrate the workflow and delegate content reasoning to sub-agents through file assignments and file outputs.

**Tech Stack:** Python stdlib, existing `research.youtube_pipeline` helpers, `unittest`, JSON/JSONL/Markdown artifacts, project-local Codex skills under `.agents/skills/youtube-*`, PowerShell-friendly CLI commands.

---

## File Structure

Create:

```text
research/youtube_pipeline/moc_agentic.py
research/youtube_pipeline/tools/__init__.py
research/youtube_pipeline/tools/normalize_transcript.py
research/youtube_pipeline/tools/count_words.py
research/youtube_pipeline/tools/estimate_tokens.py
research/youtube_pipeline/tools/chunk_transcript.py
research/youtube_pipeline/tools/prep_all.py
research/youtube_pipeline/tools/prepare_map_assignments.py
research/youtube_pipeline/tools/validate_map_outputs.py
research/youtube_pipeline/tools/assemble_map_artifacts.py
research/youtube_pipeline/tools/build_planner_context.py
research/youtube_pipeline/tools/validate_moc.py
research/youtube_pipeline/tools/dedupe_facts.py
research/youtube_pipeline/tools/align_facts.py
research/youtube_pipeline/tools/prepare_section_assignments.py
research/youtube_pipeline/tools/validate_generated_files.py
research/youtube_pipeline/tools/build_structured_analysis.py
research/youtube_pipeline/tools/assemble_report.py
research/youtube_pipeline/tools/quality_check.py
research/youtube_pipeline/tests/test_agentic_moc.py
research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt
.agents/skills/youtube-long-report/SKILL.md
.agents/skills/youtube-long-report/examples/map_assignment_sample.json
.agents/skills/youtube-long-report/examples/map_output_sample.json
.agents/skills/youtube-map-extract/SKILL.md
.agents/skills/youtube-map-extract/examples/map_assignment_sample.json
.agents/skills/youtube-map-extract/examples/map_output_sample.json
.agents/skills/youtube-moc-planning/SKILL.md
.agents/skills/youtube-moc-planning/examples/moc_sample.json
.agents/skills/youtube-section-reduce/SKILL.md
.agents/skills/youtube-section-reduce/examples/section_assignment_sample.json
.agents/skills/youtube-section-reduce/examples/alignment_sample.json
.agents/skills/youtube-report-qa/SKILL.md
```

Modify:

```text
.gitignore
research/youtube_pipeline/README.md
```

Do not modify the existing direct-LLM strategy registry for v1. This path is invoked by skills and Python tools, not by `runner.py --strategy`.

---

## Task 1: Track Project-Local YouTube Skills

**Files:**
- `.gitignore`
- `research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt`

**Steps:**

- [x] Update `.gitignore` so only `.agents/skills/youtube-*` skill folders are tracked while the rest of `.agents` remains ignored.

```gitignore
/.agents/
!/.agents/
/.agents/*
!/.agents/skills/
/.agents/skills/*
!/.agents/skills/youtube-*/
!/.agents/skills/youtube-*/**
```

- [x] Add `research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt` with a timestamped 6-8 minute transcript containing:
  - one intro/filler segment;
  - two content-heavy sections;
  - one repeated fact with two timestamps;
  - one closing synthesis.
- [x] Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
git check-ignore --quiet --no-index .agents/skills/youtube-long-report/SKILL.md; if ($LASTEXITCODE -eq 1) { "not ignored" } else { "ignored" }
```

Expected results:
- the unittest command fails until Task 2 creates the test module;
- `git check-ignore` reports `not ignored`.

---

## Task 2: Add Agentic Artifact Models And Workspace Helpers

**Files:**
- `research/youtube_pipeline/moc_agentic.py`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [x] Write failing tests for deterministic IDs, JSONL round trips, stable hashing, and stage cache keys.

```python
from research.youtube_pipeline.moc_agentic import (
    build_stage_key,
    canonical_fact_id,
    hash_file,
    hash_text,
    read_jsonl,
    write_jsonl,
)


def test_hash_text_is_stable():
    assert hash_text("alpha\nbeta") == hash_text("alpha\nbeta")
    assert hash_text("alpha\nbeta") != hash_text("alpha\nbeta\n")


def test_stage_key_includes_declared_scope():
    key = build_stage_key("extract_facts", {"chunks": "abc", "agent": "youtube-map-extract-v1"})
    assert key["stage"] == "extract_facts"
    assert key["scope"]["chunks"] == "abc"
    assert len(key["hash"]) == 64


def test_canonical_fact_id_uses_chunk_and_index():
    assert canonical_fact_id("chunk_003", 4) == "fact_chunk_003_004"
```

- [x] Implement these public helpers in `moc_agentic.py`:

```python
def hash_text(value: str) -> str: ...
def hash_file(path: Path) -> str: ...
def build_stage_key(stage: str, scope: Mapping[str, str]) -> dict[str, object]: ...
def write_json(path: Path, data: object) -> None: ...
def read_json(path: Path) -> object: ...
def write_jsonl(path: Path, rows: Iterable[Mapping[str, object]]) -> None: ...
def read_jsonl(path: Path) -> list[dict[str, object]]: ...
def canonical_fact_id(chunk_id: str, local_index: int) -> str: ...
def word_count(text: str) -> int: ...
def estimate_tokens(text: str, language: str = "ru") -> int: ...
```

- [x] Use UTF-8, sorted JSON keys, and newline-terminated JSON/JSONL files.
- [x] Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

---

## Task 3: Implement Transcript Prep And Chunk Tools

**Files:**
- `research/youtube_pipeline/moc_agentic.py`
- `research/youtube_pipeline/tools/normalize_transcript.py`
- `research/youtube_pipeline/tools/count_words.py`
- `research/youtube_pipeline/tools/estimate_tokens.py`
- `research/youtube_pipeline/tools/chunk_transcript.py`
- `research/youtube_pipeline/tools/prep_all.py`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [x] Add tests for transcript normalization, timestamp preservation, token estimates, chunk IDs, and `prep/manifest.json`.

```python
def test_chunk_transcript_preserves_timestamps(tmp_path):
    transcript = fixture_text("agentic_tiny_transcript.txt")
    chunks = chunk_transcript_text(transcript, target_tokens=160, overlap_tokens=30, language="ru")
    assert chunks[0]["chunk_id"] == "chunk_001"
    assert "start_timestamp" in chunks[0]
    assert "end_timestamp" in chunks[0]
    assert chunks[0]["word_count"] > 0
```

- [x] Implement Python functions:

```python
def normalize_transcript_text(text: str) -> str: ...
def chunk_transcript_text(
    text: str,
    *,
    target_tokens: int,
    overlap_tokens: int,
    language: str,
) -> list[dict[str, object]]: ...
def write_prep_artifacts(
    transcript_path: Path,
    output_dir: Path,
    *,
    target_tokens: int,
    overlap_tokens: int,
    language: str,
) -> dict[str, object]: ...
```

- [x] Each chunk row must include `chunk_id`, `chunk_index`, `start_timestamp`, `end_timestamp`, `text`, `word_count`, `estimated_tokens`, and `source_hash`.
- [x] `prep_all.py` writes:

```text
prep/normalized_transcript.txt
prep/chunks.jsonl
prep/manifest.json
```

- [x] CLI commands must support `--help` and return non-zero on missing input files.
- [x] Run:

```powershell
python -m research.youtube_pipeline.tools.prep_all --transcript research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt --out research/youtube_pipeline/runs/manual/agentic_smoke --language ru --target-tokens 160 --overlap-tokens 30
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

---

## Task 4: Implement Map Assignments, Validation, And Assembly

**Files:**
- `research/youtube_pipeline/moc_agentic.py`
- `research/youtube_pipeline/tools/prepare_map_assignments.py`
- `research/youtube_pipeline/tools/validate_map_outputs.py`
- `research/youtube_pipeline/tools/assemble_map_artifacts.py`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [x] Add tests for map assignment JSON, expected output paths, valid map outputs, invalid map outputs, lightweight JSON repair recording, quarantine reporting, and assembled `map/mapped_facts.jsonl`.
- [x] `prepare_map_assignments.py` reads `prep/chunks.jsonl` and writes:

```text
map/assignments/chunk_001.assignment.json
map/assignments/chunk_002.assignment.json
map/assignment_manifest.json
map/expected_files/mapper_batch_001.json
```

- [x] Each assignment JSON must include:

```json
{
  "chunk_id": "chunk_001",
  "output_file": "map/agent_outputs/chunk_001.json",
  "time_range": {"start_ms": 0, "end_ms": 600000},
  "output_language": "ru",
  "transcript_text": "...",
  "target_summary_words": 250,
  "max_fact_count": 20
}
```

- [x] `validate_map_outputs.py` validates sub-agent outputs, attempts lightweight JSON repair for common formatting defects, records repair attempts, and writes `map/validation_manifest.json`.
- [x] Valid map output schema:

```json
{
  "chunk_id": "chunk_001",
  "time_range": {"start_ms": 0, "end_ms": 600000},
  "chunk_summary": "...",
  "claims": [{"text": "...", "timestamp": "00:10:00", "importance": 4}],
  "examples": [{"text": "...", "timestamp": "00:12:30"}],
  "quotes": [{"text": "...", "timestamp": "00:13:10"}],
  "entities": ["..."],
  "open_questions": ["..."],
  "facts": [
    {
      "local_fact_id": "fact_001",
      "text": "...",
      "fact_type": "claim",
      "timestamp": "00:10:00",
      "importance": 4,
      "chunk_id": "chunk_001"
    }
  ]
}
```

- [x] `assemble_map_artifacts.py` writes:

```text
map/chunk_summaries.jsonl
map/mapped_facts.raw.jsonl
map/mapped_facts.jsonl
map/map_manifest.json
map/assembly_manifest.json
map/quarantine.jsonl
```

- [x] Assembled facts must include deterministic `fact_id`, source `chunk_id`, `timestamp`, `text`, `fact_type`, `importance`, and any normalized supporting fields needed by later alignment.
- [x] Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

---

## Task 5: Implement Planner Context And MoC Validation

**Files:**
- `research/youtube_pipeline/moc_agentic.py`
- `research/youtube_pipeline/tools/build_planner_context.py`
- `research/youtube_pipeline/tools/validate_moc.py`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [x] Add tests for context caps, language-aware token estimates, MoC schema validation, missing chunk coverage, duplicate chunk assignment, and chronological order.
- [x] `build_planner_context.py` reads `prep/chunks.jsonl`, `map/chunk_summaries.jsonl`, and `map/mapped_facts.jsonl`, then writes:

```text
planning/planner_context.md
planning/planner_context_metadata.json
```

- [x] The planner context cap must be configurable:

```powershell
python -m research.youtube_pipeline.tools.build_planner_context --run-dir <run> --max-tokens 24000 --language ru
```

- [x] Include an adaptive default:
  - `24000` tokens for unknown models;
  - lower value accepted from CLI or skill instructions;
  - value recorded in `planning/planner_context_metadata.json`.
- [x] `youtube-moc-planning` writes raw planner output to `planning/moc.raw.json`; `validate_moc.py` validates and normalizes it into `planning/moc.json`.
- [x] `validate_moc.py` writes `planning/moc_validation.json` and owns deterministic fallback planning when planner JSON cannot be corrected.
- [x] Required MoC JSON fields:

```json
{
  "report_title": "string",
  "source_kind": "youtube_video_transcript",
  "report_thesis": "string",
  "target_words": 9000,
  "nodes": [
    {
      "node_id": "moc_001",
      "title": "string",
      "purpose": "string",
      "target_words": 900,
      "time_range": {"start_ms": 0, "end_ms": 600000},
      "chunk_ids": ["chunk_001"],
      "key_questions": ["string"],
      "required_fact_types": ["claim"]
    }
  ]
}
```

- [x] Validation must require chunk coverage, positive target word counts, coherent node order, and no excessive duplicate chunk assignment unless the node explains the thematic overlap.
- [x] Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

---

## Task 6: Implement Fact Dedupe, Alignment, And Section Assignments

**Files:**
- `research/youtube_pipeline/moc_agentic.py`
- `research/youtube_pipeline/tools/dedupe_facts.py`
- `research/youtube_pipeline/tools/align_facts.py`
- `research/youtube_pipeline/tools/prepare_section_assignments.py`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [x] Add tests proving dedupe preserves all original timestamps in `source_timestamps` and all original chunks in `source_chunk_ids`.
- [x] `dedupe_facts.py` reads `map/mapped_facts.jsonl` and writes:

```text
alignment/deduplicated_facts.json
alignment/dedupe_report.json
```

- [x] Deduped fact rows must include:

```json
{
  "fact_id": "fact_cluster_0001",
  "claim": "string",
  "evidence": "string",
  "tags": ["string"],
  "source_fact_ids": ["fact_chunk_001_001"],
  "source_chunk_ids": ["chunk_001"],
  "source_timestamps": ["00:01:20"]
}
```

- [x] `align_facts.py` must align facts to MoC nodes deterministically by `source_chunk_ids` and MoC `chunk_ids`, not by semantic LLM calls.
- [x] `align_facts.py` writes:

```text
alignment/alignment.json
alignment/unaligned_facts.json
```

- [x] `prepare_section_assignments.py` writes one assignment per MoC node:

```text
alignment/section_assignments.jsonl
```

- [x] Each section assignment includes node metadata, MoC `chunk_ids`, aligned fact ids, source timestamps, expected `section_file`, target words, and overlap guidance for facts also aligned to adjacent nodes.
- [x] Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

---

## Task 7: Implement Generated File Ownership And Resume Manifests

**Files:**
- `research/youtube_pipeline/moc_agentic.py`
- `research/youtube_pipeline/tools/validate_generated_files.py`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [x] Add tests for missing section outputs, wrong output filenames, stale stage hashes, and valid resume manifests.
- [x] Define per-stage hash scopes:

```text
prep: transcript file hash + prep options
chunking: normalized transcript hash + chunk size + overlap + tokenizer estimate mode
map_assignments: chunks hash + output language + map schema version
map_outputs: assignment hash + agent model/profile + map schema version + prompt contract version
map_assembly: validated map output hashes + schema version
planner_context: chunk summaries hash + mapped facts hash + target token cap + planner context policy version
moc_planning: planner context hash + target words + output language + planner agent model/profile + MoC prompt contract version
dedupe: mapped facts hash + dedupe policy version
alignment: MoC hash + deduplicated facts hash + alignment policy version
section_assignments: MoC hash + alignment hash + budget policy version
section_writing: section assignment hash + aligned fact ids hash + writer agent model/profile + section prompt contract version
boundary_sections: MoC hash + section file hashes + map manifest hash + boundary prompt contract version
qa: section file hashes + boundary section hashes + QA policy version + QA agent model/profile when used
structured_analysis: deduplicated facts hash + alignment hash + coverage hash + structured analysis policy version
assembly: MoC hash + section file hashes + coverage hash + structured analysis hash + report assembly version
```

- [x] `validate_generated_files.py` checks all expected files listed in manifests and writes:

```text
review/generated_files_validation.json
```

- [x] Reusable stages must be accepted only when the stored stage hash equals the current stage hash and all declared output files exist.
- [x] Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

---

## Task 8: Implement Structured Analysis, Report Assembly, And Quality Check

**Files:**
- `research/youtube_pipeline/moc_agentic.py`
- `research/youtube_pipeline/tools/build_structured_analysis.py`
- `research/youtube_pipeline/tools/assemble_report.py`
- `research/youtube_pipeline/tools/quality_check.py`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [x] Add tests for deterministic structured analysis from fact clusters without LLM calls.
- [x] `build_structured_analysis.py` reads `alignment/deduplicated_facts.json`, `alignment/alignment.json`, and `review/coverage.json`, then writes:

```text
review/structured_analysis.md
review/structured_analysis_manifest.json
```

- [x] Structured analysis sections are built from tags, source timestamps, fact density, and MoC node membership.
- [x] `assemble_report.py` writes:

```text
final/report.md
final/result.json
final/metrics.json
```

- [x] Report assembly order:

```text
# <title>

Source note in 1-2 sentences saying this is a summary of a video/transcript.

## Table of Contents
## Overview
<section files in MoC order>
## Structured Analysis
## Synthesis
```

- [x] `000-overview.md` and `999-synthesis.md` are authored by the orchestrator skill after node sections are available and before QA, using MoC thesis, node titles, section opening paragraphs, section conclusions, and map manifest warnings.
- [x] `quality_check.py` writes:

```text
review/coverage.json
review/coverage.md
```

- [x] Quality checks must include word count, missing files, section order, duplicate heading detection, duplicate high-overlap paragraph detection, source note presence, repeated source-label overuse detection, and cross-section reuse of the same facts as near-identical prose.
- [x] Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

---

## Task 9: Add Project-Local Skills And Examples

**Files:**
- `.agents/skills/youtube-long-report/SKILL.md`
- `.agents/skills/youtube-long-report/examples/map_assignment_sample.json`
- `.agents/skills/youtube-long-report/examples/map_output_sample.json`
- `.agents/skills/youtube-map-extract/SKILL.md`
- `.agents/skills/youtube-map-extract/examples/map_assignment_sample.json`
- `.agents/skills/youtube-map-extract/examples/map_output_sample.json`
- `.agents/skills/youtube-moc-planning/SKILL.md`
- `.agents/skills/youtube-moc-planning/examples/moc_sample.json`
- `.agents/skills/youtube-section-reduce/SKILL.md`
- `.agents/skills/youtube-section-reduce/examples/section_assignment_sample.json`
- `.agents/skills/youtube-section-reduce/examples/alignment_sample.json`
- `.agents/skills/youtube-report-qa/SKILL.md`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [x] Add tests that verify every skill file exists, references only existing Python tools, includes no direct LLM API invocation instructions, and includes required output contracts.
- [x] `youtube-long-report` must be the orchestrator skill. It runs Python prep tools, dispatches map sub-agents, validates outputs, asks the planner skill for `planning/moc.raw.json`, runs MoC validation to produce `planning/moc.json`, prepares section assignments, dispatches section sub-agents, writes overview/synthesis, runs QA, builds structured analysis, and assembles the final report.
- [x] `youtube-map-extract` must accept one assignment JSON and write exactly the declared `output_file`.
- [x] `youtube-moc-planning` must consume `planning/planner_context.md` and write planner JSON to `planning/moc.raw.json`; Python validation produces the normalized `planning/moc.json`.
- [x] `youtube-section-reduce` must accept one assignment object from `alignment/section_assignments.jsonl` and write exactly the declared section file.
- [x] `youtube-report-qa` must inspect generated Markdown and validation files, then write `review/coverage.json`, `review/coverage.md`, and `review/reviewer_notes.md`.
- [x] Add example JSON files that match the schemas from Tasks 4, 5, and 6.
- [x] Each skill must state that Python tools may be invoked, but direct LLM API calls are forbidden in this workflow.
- [x] Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
git check-ignore --quiet --no-index .agents/skills/youtube-long-report/SKILL.md; if ($LASTEXITCODE -eq 1) { "not ignored" } else { "ignored" }
```

Expected `git check-ignore` result: `not ignored` for the YouTube skill files.

---

## Task 10: Add End-To-End Smoke Documentation And Verification

**Files:**
- `research/youtube_pipeline/README.md`
- `research/youtube_pipeline/tests/test_agentic_moc.py`

**Steps:**

- [ ] Add a deterministic smoke test that uses fixture transcript, synthetic map outputs, a synthetic MoC, synthetic section files, and the Python tools to assemble `final/report.md`.
- [ ] The smoke test must not use direct LLM API calls or network access.
- [ ] Update `README.md` with a concise usage section:

```text
Agentic MoC skills workflow:
1. Use the youtube-long-report skill.
2. Provide transcript path and output run directory.
3. The skill runs Python prep tools.
4. Map and section writing are delegated to sub-agents.
5. Python validates, assembles, and writes final/report.md.
```

- [ ] Include the minimal command sequence for deterministic tool-only validation:

```powershell
python -m research.youtube_pipeline.tools.prep_all --transcript research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt --out research/youtube_pipeline/runs/manual/agentic_smoke --language ru --target-tokens 160 --overlap-tokens 30
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

- [ ] Run final verification:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
python -m unittest discover research/youtube_pipeline/tests
git diff --check
git status --short
```

- [ ] Commit the completed implementation with:

```powershell
git add .gitignore .agents/skills/youtube-long-report .agents/skills/youtube-map-extract .agents/skills/youtube-moc-planning .agents/skills/youtube-section-reduce .agents/skills/youtube-report-qa research/youtube_pipeline
git commit -m "feat: add agentic MoC skills workflow"
```

---

## Review Checklist

- [ ] No direct LLM API calls exist in the agentic workflow.
- [ ] Every sub-agent output is file-backed and validated before downstream use.
- [ ] Resume hash scopes are explicit per stage.
- [ ] Fact dedupe preserves `source_timestamps` and `source_chunk_ids`.
- [ ] Fact-to-section alignment is deterministic by chunk ID.
- [ ] Overview and synthesis generation ownership is documented.
- [ ] Structured analysis is deterministic and built from fact clusters.
- [ ] Skill files are tracked despite the broader `.agents` ignore rule.
- [ ] Tests cover deterministic tools without network access.
- [ ] The final report has an unobtrusive source note saying it summarizes a video/transcript.
