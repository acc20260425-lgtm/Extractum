# YouTube Summary Wrapper Skill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a user-facing `youtube-summary` skill so a user can provide a transcript path once and let Codex orchestrate the existing file-backed agentic MoC workflow without manual Python command bookkeeping.

**Architecture:** Add a thin deterministic workflow-state layer beside the existing `moc_agentic.py` helpers. Python creates and resumes runs, writes `workflow_state.json`, updates `run_index.json`, and exposes bootstrap/state-update CLI commands; Codex skills and sub-agents still perform all LLM reasoning. Add a public `.agents/skills/youtube-summary/SKILL.md` wrapper that delegates to existing child skills and pauses before map extraction when sub-agents are unavailable.

**Tech Stack:** Python stdlib, `unittest`, existing `research.youtube_pipeline.moc_agentic` helpers, project-local Codex skills in `.agents/skills`.

---

## File Structure

- Create: `research/youtube_pipeline/youtube_summary_workflow.py`
  - Owns workflow-state constants, option hashing, run-index lookup, bootstrap run creation/resume, and deterministic state updates.
- Create: `research/youtube_pipeline/tools/start_youtube_summary.py`
  - CLI wrapper around `start_youtube_summary_run(...)`.
- Create: `research/youtube_pipeline/tools/update_youtube_summary_state.py`
  - CLI wrapper around `update_workflow_state(...)`.
- Modify: `research/youtube_pipeline/tests/test_agentic_moc.py`
  - Adds focused tests for workflow-state creation, resume lookup, force-new-run behavior, state transitions, and skill contract.
- Create: `.agents/skills/youtube-summary/SKILL.md`
  - Public skill used by the user; wraps `youtube-long-report` mechanics and references existing child skills.
- Modify: `.gitignore`
  - Add exception for `.agents/skills/youtube-summary/**` if the current ignore rules require explicit skill allowlisting.
- Modify: `research/youtube_pipeline/README.md`
  - Document the one-request user workflow.

## Task 1: Add Workflow Option Hashing And Run Index Primitives

**Files:**
- Create: `research/youtube_pipeline/youtube_summary_workflow.py`
- Modify: `research/youtube_pipeline/tests/test_agentic_moc.py`

- [x] **Step 1: Write failing tests for option hashing and run index rebuild**

Add these imports near the existing imports in `research/youtube_pipeline/tests/test_agentic_moc.py`:

```python
from research.youtube_pipeline.youtube_summary_workflow import (
    WORKFLOW_STATE_SCHEMA,
    compute_options_hash,
    normalize_workflow_options,
    read_run_index,
    rebuild_run_index,
    write_run_index,
)
```

Add these tests to `AgenticArtifactHelperTests`:

```python
    def test_youtube_summary_options_hash_ignores_volatile_paths(self):
        options = normalize_workflow_options(
            output_language="ru",
            target_words=10000,
            target_tokens=1600,
            overlap_tokens=200,
            planner_context_tokens=24000,
        )
        same_options = {
            **options,
            "run_dir": "research/youtube_pipeline/runs/manual/one",
            "created_at": "2026-06-18T16:00:00",
        }

        self.assertEqual(options["schema"], WORKFLOW_STATE_SCHEMA)
        self.assertEqual(compute_options_hash(options), compute_options_hash(same_options))

    def test_youtube_summary_options_hash_changes_for_workflow_fields(self):
        base = normalize_workflow_options(
            output_language="ru",
            target_words=10000,
            target_tokens=1600,
            overlap_tokens=200,
            planner_context_tokens=24000,
        )
        changed = normalize_workflow_options(
            output_language="ru",
            target_words=12000,
            target_tokens=1600,
            overlap_tokens=200,
            planner_context_tokens=24000,
        )

        self.assertNotEqual(compute_options_hash(base), compute_options_hash(changed))

    def test_run_index_round_trip_and_rebuild_from_state_files(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_root = Path(temp_dir) / "runs"
            run_dir = run_root / "sample" / "20260618-160000"
            state = {
                "schema": WORKFLOW_STATE_SCHEMA,
                "run_root": str(run_root),
                "run_dir": str(run_dir),
                "transcript_path": "input.txt",
                "transcript_sha256": "abc",
                "output_language": "ru",
                "target_words": 10000,
                "options_hash": "def",
                "current_stage": "map_assignments_ready",
                "next_action": "dispatch_map_extractors",
                "artifacts": {},
                "counts": {},
                "commands": {},
                "created_at": "2026-06-18T16:00:00",
                "updated_at": "2026-06-18T16:00:00",
                "validation_warnings": [],
            }
            write_json(run_dir / "workflow_state.json", state)

            index = rebuild_run_index(run_root)

            self.assertEqual(index["schema"], "youtube-summary-run-index-v1")
            self.assertEqual(index["runs"][0]["run_dir"], str(run_dir))
            self.assertEqual(index["runs"][0]["transcript_sha256"], "abc")

            write_run_index(run_root, index)
            self.assertEqual(read_json(run_root / "run_index.json"), index)

    def test_read_run_index_rebuilds_broken_index_file(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_root = Path(temp_dir) / "runs"
            run_dir = run_root / "sample" / "20260618-160000"
            state = {
                "schema": WORKFLOW_STATE_SCHEMA,
                "run_root": str(run_root),
                "run_dir": str(run_dir),
                "transcript_path": "input.txt",
                "transcript_sha256": "abc",
                "output_language": "ru",
                "target_words": 10000,
                "options_hash": "def",
                "current_stage": "map_assignments_ready",
                "next_action": "dispatch_map_extractors",
                "artifacts": {},
                "counts": {},
                "commands": {},
                "created_at": "2026-06-18T16:00:00",
                "updated_at": "2026-06-18T16:00:00",
                "validation_warnings": [],
            }
            write_json(run_dir / "workflow_state.json", state)
            (run_root / "run_index.json").write_text("{broken", encoding="utf-8")

            index = read_run_index(run_root)

            self.assertEqual(index["runs"][0]["run_dir"], str(run_dir))
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_summary_options_hash_ignores_volatile_paths research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_summary_options_hash_changes_for_workflow_fields research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_run_index_round_trip_and_rebuild_from_state_files research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_read_run_index_rebuilds_broken_index_file
```

Expected: fail with `ModuleNotFoundError: No module named 'research.youtube_pipeline.youtube_summary_workflow'`.

- [x] **Step 3: Add workflow primitive implementation**

Create `research/youtube_pipeline/youtube_summary_workflow.py`:

```python
from __future__ import annotations

import hashlib
import json
import os
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Mapping

from research.youtube_pipeline.moc_agentic import read_json, write_json

WORKFLOW_STATE_SCHEMA = "youtube-summary-workflow-state-v1"
RUN_INDEX_SCHEMA = "youtube-summary-run-index-v1"
WORKFLOW_SKILL_VERSION = "youtube-summary-v1"
DEFAULT_RUN_ROOT = Path("research/youtube_pipeline/runs/manual/youtube_summary_agentic")
WORKFLOW_STAGES = [
    "created",
    "map_assignments_ready",
    "map_outputs_ready",
    "map_assembled",
    "planner_context_ready",
    "moc_ready",
    "alignment_ready",
    "sections_ready",
    "qa_ready",
    "final_ready",
]
WORKFLOW_OPTION_FIELDS = [
    "schema",
    "output_language",
    "target_words",
    "target_tokens",
    "overlap_tokens",
    "planner_context_tokens",
    "workflow_skill_version",
]


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def transcript_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalize_workflow_options(
    *,
    output_language: str,
    target_words: int,
    target_tokens: int,
    overlap_tokens: int,
    planner_context_tokens: int,
    workflow_skill_version: str = WORKFLOW_SKILL_VERSION,
) -> dict[str, object]:
    return {
        "schema": WORKFLOW_STATE_SCHEMA,
        "output_language": output_language,
        "target_words": target_words,
        "target_tokens": target_tokens,
        "overlap_tokens": overlap_tokens,
        "planner_context_tokens": planner_context_tokens,
        "workflow_skill_version": workflow_skill_version,
    }


def compute_options_hash(options: Mapping[str, object]) -> str:
    stable = {field: options[field] for field in WORKFLOW_OPTION_FIELDS if field in options}
    payload = json.dumps(stable, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def slug_from_transcript_path(path: Path) -> str:
    stem = path.stem.lower()
    chars = [char if char.isalnum() else "_" for char in stem]
    slug = "".join(chars).strip("_")
    return slug[:48] or "transcript"


def atomic_write_json(path: Path, data: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        "w",
        encoding="utf-8",
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".tmp",
        delete=False,
    ) as handle:
        handle.write(json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
        temp_path = Path(handle.name)
    os.replace(temp_path, path)


def read_run_index(run_root: Path) -> dict[str, object]:
    index_path = run_root / "run_index.json"
    if not index_path.exists():
        return {"schema": RUN_INDEX_SCHEMA, "runs": []}
    try:
        value = read_json(index_path)
    except (OSError, ValueError, json.JSONDecodeError):
        return rebuild_run_index(run_root)
    if not isinstance(value, dict) or value.get("schema") != RUN_INDEX_SCHEMA:
        return rebuild_run_index(run_root)
    runs = value.get("runs")
    if not isinstance(runs, list):
        return rebuild_run_index(run_root)
    return value


def write_run_index(run_root: Path, index: Mapping[str, object]) -> None:
    atomic_write_json(run_root / "run_index.json", dict(index))


def rebuild_run_index(run_root: Path) -> dict[str, object]:
    runs: list[dict[str, object]] = []
    if run_root.exists():
        for state_path in sorted(run_root.glob("*/*/workflow_state.json")):
            try:
                state = read_json(state_path)
            except (OSError, ValueError, json.JSONDecodeError):
                continue
            if not isinstance(state, dict) or state.get("schema") != WORKFLOW_STATE_SCHEMA:
                continue
            runs.append(
                {
                    "run_root": str(run_root),
                    "run_dir": str(state_path.parent),
                    "transcript_path": str(state.get("transcript_path", "")),
                    "transcript_sha256": str(state.get("transcript_sha256", "")),
                    "options_hash": str(state.get("options_hash", "")),
                    "current_stage": str(state.get("current_stage", "")),
                    "created_at": str(state.get("created_at", "")),
                    "updated_at": str(state.get("updated_at", "")),
                }
            )
    runs.sort(key=lambda row: str(row.get("updated_at", "")), reverse=True)
    return {"schema": RUN_INDEX_SCHEMA, "runs": runs}
```

- [x] **Step 4: Run tests to verify they pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_summary_options_hash_ignores_volatile_paths research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_summary_options_hash_changes_for_workflow_fields research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_run_index_round_trip_and_rebuild_from_state_files research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_read_run_index_rebuilds_broken_index_file
```

Expected: `Ran 4 tests` and `OK`.

- [x] **Step 5: Commit Task 1**

Run:

```powershell
git add research/youtube_pipeline/youtube_summary_workflow.py research/youtube_pipeline/tests/test_agentic_moc.py
git commit -m "feat: add youtube summary workflow state primitives"
```

## Task 2: Add Bootstrap Run Creation, Resume Lookup, And State Updates

**Files:**
- Modify: `research/youtube_pipeline/youtube_summary_workflow.py`
- Modify: `research/youtube_pipeline/tests/test_agentic_moc.py`

- [x] **Step 1: Write failing tests for bootstrap/resume/update behavior**

Extend the workflow import in `test_agentic_moc.py`:

```python
from research.youtube_pipeline.youtube_summary_workflow import (
    WORKFLOW_STATE_SCHEMA,
    compute_options_hash,
    find_latest_matching_run,
    normalize_workflow_options,
    rebuild_run_index,
    start_youtube_summary_run,
    update_workflow_state,
    write_run_index,
)
```

Add these tests:

```python
    def test_start_youtube_summary_run_creates_state_and_assignments(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_root = Path(temp_dir) / "summary_runs"

            state = start_youtube_summary_run(
                FIXTURES_DIR / "agentic_tiny_transcript.txt",
                run_root=run_root,
                output_language="ru",
                target_words=10000,
                target_tokens=160,
                overlap_tokens=30,
                planner_context_tokens=3000,
            )

            run_dir = Path(str(state["run_dir"]))
            self.assertEqual(state["schema"], WORKFLOW_STATE_SCHEMA)
            self.assertEqual(state["current_stage"], "map_assignments_ready")
            self.assertEqual(state["next_action"], "dispatch_map_extractors")
            self.assertTrue((run_dir / "prep" / "chunks.jsonl").exists())
            self.assertTrue((run_dir / "map" / "assignment_manifest.json").exists())
            self.assertTrue((run_dir / "workflow_state.json").exists())
            self.assertTrue((run_root / "run_index.json").exists())

    def test_start_youtube_summary_run_resumes_latest_matching_run(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_root = Path(temp_dir) / "summary_runs"
            first = start_youtube_summary_run(
                FIXTURES_DIR / "agentic_tiny_transcript.txt",
                run_root=run_root,
                output_language="ru",
                target_words=10000,
                target_tokens=160,
                overlap_tokens=30,
                planner_context_tokens=3000,
            )
            second = start_youtube_summary_run(
                FIXTURES_DIR / "agentic_tiny_transcript.txt",
                run_root=run_root,
                output_language="ru",
                target_words=10000,
                target_tokens=160,
                overlap_tokens=30,
                planner_context_tokens=3000,
            )

            self.assertEqual(second["run_dir"], first["run_dir"])
            self.assertTrue(second["resumed"])

    def test_start_youtube_summary_run_force_creates_new_run(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_root = Path(temp_dir) / "summary_runs"
            first = start_youtube_summary_run(
                FIXTURES_DIR / "agentic_tiny_transcript.txt",
                run_root=run_root,
                output_language="ru",
                target_words=10000,
                target_tokens=160,
                overlap_tokens=30,
                planner_context_tokens=3000,
            )
            second = start_youtube_summary_run(
                FIXTURES_DIR / "agentic_tiny_transcript.txt",
                run_root=run_root,
                output_language="ru",
                target_words=10000,
                target_tokens=160,
                overlap_tokens=30,
                planner_context_tokens=3000,
                force=True,
            )

            self.assertNotEqual(second["run_dir"], first["run_dir"])
            self.assertFalse(second["resumed"])

    def test_start_youtube_summary_run_rejects_explicit_run_dir_without_state(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            explicit_run_dir = Path(temp_dir) / "explicit_run"

            with self.assertRaises(FileNotFoundError):
                start_youtube_summary_run(
                    FIXTURES_DIR / "agentic_tiny_transcript.txt",
                    run_root=Path(temp_dir) / "summary_runs",
                    run_dir=explicit_run_dir,
                    output_language="ru",
                    target_words=10000,
                    target_tokens=160,
                    overlap_tokens=30,
                    planner_context_tokens=3000,
                )

    def test_start_youtube_summary_run_rejects_explicit_run_dir_with_invalid_state(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            explicit_run_dir = Path(temp_dir) / "explicit_run"
            write_json(explicit_run_dir / "workflow_state.json", {"schema": "wrong"})

            with self.assertRaises(ValueError):
                start_youtube_summary_run(
                    FIXTURES_DIR / "agentic_tiny_transcript.txt",
                    run_root=Path(temp_dir) / "summary_runs",
                    run_dir=explicit_run_dir,
                    output_language="ru",
                    target_words=10000,
                    target_tokens=160,
                    overlap_tokens=30,
                    planner_context_tokens=3000,
                )

    def test_update_workflow_state_advances_stage_and_preserves_commands(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            state = start_youtube_summary_run(
                FIXTURES_DIR / "agentic_tiny_transcript.txt",
                run_root=Path(temp_dir) / "summary_runs",
                output_language="ru",
                target_words=10000,
                target_tokens=160,
                overlap_tokens=30,
                planner_context_tokens=3000,
            )

            updated = update_workflow_state(
                Path(str(state["run_dir"])),
                current_stage="map_outputs_ready",
                next_action="assemble_map_artifacts",
                artifacts={"validation_manifest": "map/validation_manifest.json"},
                counts={"valid_map_output_count": 1},
                validation_warnings=["example warning"],
            )

            self.assertEqual(updated["current_stage"], "map_outputs_ready")
            self.assertEqual(updated["artifacts"]["validation_manifest"], "map/validation_manifest.json")
            self.assertEqual(updated["counts"]["valid_map_output_count"], 1)
            self.assertIn("validate_map_outputs", updated["commands"])
            self.assertEqual(updated["validation_warnings"], ["example warning"])
            index = read_json(Path(str(state["run_root"])) / "run_index.json")
            self.assertEqual(index["runs"][0]["current_stage"], "map_outputs_ready")
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_creates_state_and_assignments research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_resumes_latest_matching_run research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_force_creates_new_run research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_rejects_explicit_run_dir_without_state research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_rejects_explicit_run_dir_with_invalid_state research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_update_workflow_state_advances_stage_and_preserves_commands
```

Expected: fail with import errors for the new workflow functions.

- [x] **Step 3: Implement bootstrap/resume/update functions**

Append this code to `research/youtube_pipeline/youtube_summary_workflow.py`:

```python
from research.youtube_pipeline.moc_agentic import prepare_map_assignments, write_prep_artifacts


def default_commands(run_dir: Path, *, output_language: str, target_words: int, planner_context_tokens: int) -> dict[str, str]:
    run = str(run_dir)
    return {
        "validate_map_outputs": f"python -m research.youtube_pipeline.tools.validate_map_outputs --run-dir {run}",
        "assemble_map_artifacts": f"python -m research.youtube_pipeline.tools.assemble_map_artifacts --run-dir {run}",
        "build_planner_context": (
            f"python -m research.youtube_pipeline.tools.build_planner_context --run-dir {run} "
            f"--max-tokens {planner_context_tokens} --language {output_language}"
        ),
        "validate_moc": f"python -m research.youtube_pipeline.tools.validate_moc --run-dir {run} --target-words {target_words}",
        "dedupe_facts": f"python -m research.youtube_pipeline.tools.dedupe_facts --run-dir {run}",
        "align_facts": f"python -m research.youtube_pipeline.tools.align_facts --run-dir {run}",
        "prepare_section_assignments": f"python -m research.youtube_pipeline.tools.prepare_section_assignments --run-dir {run}",
        "quality_check": f"python -m research.youtube_pipeline.tools.quality_check --run-dir {run}",
        "build_structured_analysis": f"python -m research.youtube_pipeline.tools.build_structured_analysis --run-dir {run}",
        "assemble_report": f"python -m research.youtube_pipeline.tools.assemble_report --run-dir {run}",
    }


def find_latest_matching_run(run_root: Path, *, transcript_hash: str, options_hash: str) -> dict[str, object] | None:
    index = read_run_index(run_root)
    matches = [
        run
        for run in index.get("runs", [])
        if isinstance(run, dict)
        and run.get("transcript_sha256") == transcript_hash
        and run.get("options_hash") == options_hash
    ]
    if not matches:
        rebuilt = rebuild_run_index(run_root)
        write_run_index(run_root, rebuilt)
        matches = [
            run
            for run in rebuilt.get("runs", [])
            if isinstance(run, dict)
            and run.get("transcript_sha256") == transcript_hash
            and run.get("options_hash") == options_hash
        ]
    if not matches:
        return None
    matches.sort(key=lambda row: str(row.get("updated_at", "")), reverse=True)
    return matches[0]


def update_index_with_state(run_root: Path, state: Mapping[str, object]) -> None:
    index = read_run_index(run_root)
    runs = [run for run in index.get("runs", []) if isinstance(run, dict)]
    run_dir = str(state["run_dir"])
    runs = [run for run in runs if run.get("run_dir") != run_dir]
    runs.insert(
        0,
        {
            "run_dir": run_dir,
            "transcript_path": str(state.get("transcript_path", "")),
            "transcript_sha256": str(state.get("transcript_sha256", "")),
            "options_hash": str(state.get("options_hash", "")),
            "current_stage": str(state.get("current_stage", "")),
            "created_at": str(state.get("created_at", "")),
            "updated_at": str(state.get("updated_at", "")),
        },
    )
    write_run_index(run_root, {"schema": RUN_INDEX_SCHEMA, "runs": runs})


def next_run_dir(run_root: Path, transcript_path: Path, now: str | None = None) -> Path:
    timestamp = (now or utc_now()).replace("-", "").replace(":", "").replace("Z", "").replace("T", "-")
    base_dir = run_root / slug_from_transcript_path(transcript_path)
    candidate = base_dir / timestamp
    suffix = 2
    while candidate.exists():
        candidate = base_dir / f"{timestamp}-{suffix:02d}"
        suffix += 1
    return candidate


def start_youtube_summary_run(
    transcript_path: Path,
    *,
    run_root: Path = DEFAULT_RUN_ROOT,
    run_dir: Path | None = None,
    output_language: str = "ru",
    target_words: int = 10000,
    target_tokens: int = 1600,
    overlap_tokens: int = 200,
    planner_context_tokens: int = 24000,
    force: bool = False,
) -> dict[str, object]:
    transcript_path = transcript_path.resolve()
    if not transcript_path.exists():
        raise FileNotFoundError(f"Transcript file does not exist: {transcript_path}")

    options = normalize_workflow_options(
        output_language=output_language,
        target_words=target_words,
        target_tokens=target_tokens,
        overlap_tokens=overlap_tokens,
        planner_context_tokens=planner_context_tokens,
    )
    transcript_hash = transcript_sha256(transcript_path)
    options_digest = compute_options_hash(options)

    if run_dir is not None:
        state_path = run_dir / "workflow_state.json"
        if not state_path.exists():
            raise FileNotFoundError(f"workflow_state.json does not exist for explicit run: {state_path}")
        state = read_json(state_path)
        if not isinstance(state, dict) or state.get("schema") != WORKFLOW_STATE_SCHEMA:
            raise ValueError(f"Invalid workflow_state.json for explicit run: {state_path}")
        state["resumed"] = True
        return state
    elif not force:
        match = find_latest_matching_run(run_root, transcript_hash=transcript_hash, options_hash=options_digest)
        if match:
            state = read_json(Path(str(match["run_dir"])) / "workflow_state.json")
            if isinstance(state, dict):
                state["resumed"] = True
                return state
            raise ValueError(f"Matching workflow_state.json is invalid for run: {match['run_dir']}")
        selected_run_dir = next_run_dir(run_root, transcript_path)
    else:
        selected_run_dir = next_run_dir(run_root, transcript_path)

    prep_manifest = write_prep_artifacts(
        transcript_path,
        selected_run_dir,
        target_tokens=target_tokens,
        overlap_tokens=overlap_tokens,
        language=output_language,
    )
    assignment_manifest = prepare_map_assignments(selected_run_dir, output_language=output_language)
    now = utc_now()
    state = {
        **options,
        "run_root": str(run_root),
        "run_dir": str(selected_run_dir),
        "transcript_path": str(transcript_path),
        "transcript_sha256": transcript_hash,
        "options_hash": options_digest,
        "current_stage": "map_assignments_ready",
        "next_action": "dispatch_map_extractors",
        "artifacts": {
            "normalized_transcript": "prep/normalized_transcript.txt",
            "chunks": "prep/chunks.jsonl",
            "prep_manifest": "prep/manifest.json",
            "assignment_manifest": "map/assignment_manifest.json",
        },
        "counts": {
            "chunk_count": int(prep_manifest["chunk_count"]),
            "map_assignment_count": int(assignment_manifest["assignment_count"]),
        },
        "commands": default_commands(
            selected_run_dir,
            output_language=output_language,
            target_words=target_words,
            planner_context_tokens=planner_context_tokens,
        ),
        "created_at": now,
        "updated_at": now,
        "validation_warnings": [],
        "resumed": False,
    }
    write_json(selected_run_dir / "workflow_state.json", state)
    update_index_with_state(run_root, state)
    return state


def update_workflow_state(
    run_dir: Path,
    *,
    current_stage: str,
    next_action: str,
    artifacts: Mapping[str, object] | None = None,
    counts: Mapping[str, object] | None = None,
    validation_warnings: list[str] | None = None,
) -> dict[str, object]:
    if current_stage not in WORKFLOW_STAGES:
        raise ValueError(f"Unknown workflow stage: {current_stage}")
    state_path = run_dir / "workflow_state.json"
    state = read_json(state_path)
    if not isinstance(state, dict) or state.get("schema") != WORKFLOW_STATE_SCHEMA:
        raise ValueError(f"Invalid workflow state: {state_path}")
    state["current_stage"] = current_stage
    state["next_action"] = next_action
    state["updated_at"] = utc_now()
    state.setdefault("artifacts", {})
    state.setdefault("counts", {})
    if artifacts:
        state["artifacts"].update(dict(artifacts))
    if counts:
        state["counts"].update(dict(counts))
    if validation_warnings is not None:
        state["validation_warnings"] = validation_warnings
    write_json(state_path, state)
    run_root = state.get("run_root")
    if isinstance(run_root, str) and run_root:
        update_index_with_state(Path(run_root), state)
    return state
```

- [x] **Step 4: Run tests to verify they pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_creates_state_and_assignments research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_resumes_latest_matching_run research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_force_creates_new_run research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_rejects_explicit_run_dir_without_state research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_start_youtube_summary_run_rejects_explicit_run_dir_with_invalid_state research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_update_workflow_state_advances_stage_and_preserves_commands
```

Expected: `Ran 6 tests` and `OK`.

- [x] **Step 5: Commit Task 2**

Run:

```powershell
git add research/youtube_pipeline/youtube_summary_workflow.py research/youtube_pipeline/tests/test_agentic_moc.py
git commit -m "feat: add youtube summary run bootstrap"
```

## Task 3: Add Bootstrap And State Update CLI Tools

**Files:**
- Create: `research/youtube_pipeline/tools/start_youtube_summary.py`
- Create: `research/youtube_pipeline/tools/update_youtube_summary_state.py`
- Modify: `research/youtube_pipeline/tests/test_agentic_moc.py`

- [x] **Step 1: Write failing CLI contract tests**

Add this test:

```python
    def test_youtube_summary_cli_modules_are_importable(self):
        module_names = [
            "research.youtube_pipeline.tools.start_youtube_summary",
            "research.youtube_pipeline.tools.update_youtube_summary_state",
        ]

        for module_name in module_names:
            self.assertIsNotNone(importlib.util.find_spec(module_name))
```

- [x] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_summary_cli_modules_are_importable
```

Expected: fail because the new modules do not exist.

- [x] **Step 3: Add `start_youtube_summary.py`**

Create `research/youtube_pipeline/tools/start_youtube_summary.py`:

```python
import argparse
from pathlib import Path

from research.youtube_pipeline.youtube_summary_workflow import DEFAULT_RUN_ROOT, start_youtube_summary_run


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Start or resume a user-facing YouTube summary workflow.")
    parser.add_argument("--transcript", required=True, type=Path)
    parser.add_argument("--run-root", default=DEFAULT_RUN_ROOT, type=Path)
    parser.add_argument("--run-dir", type=Path)
    parser.add_argument("--language", default="ru")
    parser.add_argument("--target-words", default=10000, type=int)
    parser.add_argument("--target-tokens", default=1600, type=int)
    parser.add_argument("--overlap-tokens", default=200, type=int)
    parser.add_argument("--planner-context-tokens", default=24000, type=int)
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args(argv)

    state = start_youtube_summary_run(
        args.transcript,
        run_root=args.run_root,
        run_dir=args.run_dir,
        output_language=args.language,
        target_words=args.target_words,
        target_tokens=args.target_tokens,
        overlap_tokens=args.overlap_tokens,
        planner_context_tokens=args.planner_context_tokens,
        force=args.force,
    )
    print(f"run_dir={state['run_dir']}")
    print(f"current_stage={state['current_stage']}")
    print(f"next_action={state['next_action']}")
    print(f"resumed={str(bool(state.get('resumed'))).lower()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [x] **Step 4: Add `update_youtube_summary_state.py`**

Create `research/youtube_pipeline/tools/update_youtube_summary_state.py`:

```python
import argparse
from pathlib import Path

from research.youtube_pipeline.youtube_summary_workflow import update_workflow_state


def parse_key_value(values: list[str]) -> dict[str, object]:
    parsed: dict[str, object] = {}
    for value in values:
        if "=" not in value:
            raise ValueError(f"Expected key=value, got: {value}")
        key, raw = value.split("=", 1)
        parsed[key] = raw
    return parsed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Update a YouTube summary workflow_state.json file.")
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--stage", required=True)
    parser.add_argument("--next-action", required=True)
    parser.add_argument("--artifact", action="append", default=[])
    parser.add_argument("--count", action="append", default=[])
    parser.add_argument("--warning", action="append", default=[])
    args = parser.parse_args(argv)

    counts: dict[str, object] = {}
    for key, value in parse_key_value(args.count).items():
        try:
            counts[key] = int(str(value))
        except ValueError:
            counts[key] = value

    state = update_workflow_state(
        args.run_dir,
        current_stage=args.stage,
        next_action=args.next_action,
        artifacts=parse_key_value(args.artifact),
        counts=counts,
        validation_warnings=args.warning,
    )
    print(f"run_dir={state['run_dir']}")
    print(f"current_stage={state['current_stage']}")
    print(f"next_action={state['next_action']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [x] **Step 5: Run CLI smoke commands**

Run:

```powershell
$runRoot = "research/youtube_pipeline/runs/manual/youtube_summary_cli_smoke"
$startOutput = python -m research.youtube_pipeline.tools.start_youtube_summary --transcript research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt --run-root $runRoot --language ru --target-words 10000 --target-tokens 160 --overlap-tokens 30 --planner-context-tokens 3000 --force
$startOutput
$runDir = ($startOutput | Where-Object { $_ -like "run_dir=*" }) -replace "^run_dir=", ""
```

Expected output contains:

```text
current_stage=map_assignments_ready
next_action=dispatch_map_extractors
resumed=false
```

Then run with the captured run dir:

```powershell
python -m research.youtube_pipeline.tools.update_youtube_summary_state --run-dir $runDir --stage map_outputs_ready --next-action assemble_map_artifacts --artifact validation_manifest=map/validation_manifest.json --count valid_map_output_count=1 --warning "manual smoke warning"
```

Expected output contains:

```text
current_stage=map_outputs_ready
next_action=assemble_map_artifacts
```

- [x] **Step 6: Run unit tests**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_summary_cli_modules_are_importable
```

Expected: `Ran 1 test` and `OK`.

- [x] **Step 7: Commit Task 3**

Run:

```powershell
git add research/youtube_pipeline/tools/start_youtube_summary.py research/youtube_pipeline/tools/update_youtube_summary_state.py research/youtube_pipeline/tests/test_agentic_moc.py
git commit -m "feat: add youtube summary workflow cli"
```

## Task 4: Add User-Facing `youtube-summary` Skill

**Files:**
- Create: `.agents/skills/youtube-summary/SKILL.md`
- Modify: `.gitignore`
- Modify: `research/youtube_pipeline/tests/test_agentic_moc.py`

- [x] **Step 1: Write failing skill contract test**

Modify `test_youtube_skill_files_exist_and_reference_existing_tools` in `test_agentic_moc.py` by adding the new skill file to `skill_files`:

```python
            REPO_ROOT / ".agents" / "skills" / "youtube-summary" / "SKILL.md",
```

Add this test:

```python
    def test_youtube_summary_skill_mentions_state_and_subagent_boundary(self):
        skill_file = REPO_ROOT / ".agents" / "skills" / "youtube-summary" / "SKILL.md"
        text = skill_file.read_text(encoding="utf-8")

        self.assertIn("start_youtube_summary", text)
        self.assertIn("workflow_state.json", text)
        self.assertIn("update_youtube_summary_state", text)
        self.assertIn("youtube-map-extract", text)
        self.assertIn("youtube-section-reduce", text)
        self.assertIn("pause before map extraction", text.lower())
        self.assertIn("Direct LLM API calls are forbidden", text)
        self.assertNotIn("requests.post", text.lower())
        self.assertNotIn("chat.completions", text.lower())
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_skill_files_exist_and_reference_existing_tools research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_summary_skill_mentions_state_and_subagent_boundary
```

Expected: fail because `.agents/skills/youtube-summary/SKILL.md` does not exist.

- [x] **Step 3: Add `.gitignore` allowlist if needed**

Inspect `.gitignore` for `.agents/skills` rules. If it already allows `.agents/skills/youtube-*/**`, do not change it. If each skill is explicitly allowlisted, add:

```gitignore
!.agents/skills/youtube-summary/
!.agents/skills/youtube-summary/**
```

- [x] **Step 4: Create `youtube-summary` skill**

Create `.agents/skills/youtube-summary/SKILL.md`:

````markdown
---
name: youtube-summary
description: Use when the user wants a long report from a YouTube transcript with one public skill request.
---

# YouTube Summary

## Overview

Use this as the public wrapper for long file-backed YouTube transcript reports.
The user provides a transcript path and target language/length once; this skill
creates or resumes the run, runs deterministic Python tools, and delegates
reasoning work to the existing YouTube child skills.

Direct LLM API calls are forbidden. Do not use Python, HTTP clients, or provider
SDKs for map extraction, MoC planning, section writing, or QA judgment.

## Inputs

- transcript path;
- output language, default `ru`;
- target report words, default `10000`;
- optional existing run directory;
- optional chunk settings.

## Bootstrap

Run:

```powershell
python -m research.youtube_pipeline.tools.start_youtube_summary --transcript <path> --language <language> --target-words <words>
```

Use `--run-dir <path>` when the user gives an explicit run. Use `--force` only
when the user asks for a fresh run instead of resuming a matching run.

Read `<run-dir>/workflow_state.json` after bootstrap. Use its `next_action`,
`artifacts`, `counts`, and `commands` fields to continue.

## Workflow

1. Bootstrap or resume the run with `start_youtube_summary`.
2. If `next_action` is `dispatch_map_extractors`, dispatch `youtube-map-extract`
   sub-agents for files in `map/assignments`.
3. If sub-agents are unavailable before map extraction, pause before map
   extraction and explain that the workflow needs map extractor sub-agents.
4. Validate map outputs:
   `python -m research.youtube_pipeline.tools.validate_map_outputs --run-dir <run-dir>`
5. On valid map outputs, update state:
   `python -m research.youtube_pipeline.tools.update_youtube_summary_state --run-dir <run-dir> --stage map_outputs_ready --next-action assemble_map_artifacts`
6. Assemble map artifacts, build planner context, and update state after each
   deterministic gate.
7. Use `youtube-moc-planning` to write `planning/moc.raw.json`.
8. Validate MoC, dedupe facts, align facts, prepare section assignments, and
   update state after each successful gate.
9. Use `youtube-section-reduce` for section files. If section-writer sub-agents
   are unavailable after valid map outputs exist, the main agent may write
   sections sequentially using the same skill contract.
10. The orchestrator writes `sections/000-overview.md` and
    `sections/999-synthesis.md` after section validation. Use only the validated
    MoC thesis, node titles, section opening paragraphs, section conclusions,
    and repeated high-importance facts.
11. Use `youtube-report-qa` for review notes.
12. Run structured analysis and final assembly:
    `python -m research.youtube_pipeline.tools.build_structured_analysis --run-dir <run-dir>`
    `python -m research.youtube_pipeline.tools.assemble_report --run-dir <run-dir>`

## Error Handling

- Do not continue past invalid artifacts.
- Missing map output: dispatch the relevant `youtube-map-extract` assignment.
- Invalid map schema: ask the extractor to rewrite the exact output file.
- Invalid MoC: request one corrected `planning/moc.raw.json`.
- Missing section file: rerun that exact section assignment.
- Source framing overuse: request targeted section rewrite, not a whole-report
  rewrite.

## Output Contract

Return:

- `final/report.md`;
- `final/metrics.json`;
- validation warnings from `workflow_state.json`;
- any files that still require user or sub-agent action.
````

- [x] **Step 5: Run skill contract tests**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_skill_files_exist_and_reference_existing_tools research.youtube_pipeline.tests.test_agentic_moc.AgenticArtifactHelperTests.test_youtube_summary_skill_mentions_state_and_subagent_boundary
```

Expected: `Ran 2 tests` and `OK`.

- [x] **Step 6: Commit Task 4**

Run:

```powershell
git add .agents/skills/youtube-summary/SKILL.md .gitignore research/youtube_pipeline/tests/test_agentic_moc.py
git commit -m "feat: add public youtube summary skill"
```

## Task 5: Document One-Request Workflow

**Files:**
- Modify: `research/youtube_pipeline/README.md`

- [x] **Step 1: Add README section**

Insert this after the opening paragraph of `## Agentic MoC Skills Workflow`:

````markdown
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
````

- [x] **Step 2: Fix existing README wording about map extraction**

In `research/youtube_pipeline/README.md`, replace:

```markdown
Map extraction, MoC planning, and section writing are performed by the
main agent or sub-agents through file contracts.
```

with:

```markdown
Map extraction is performed by sub-agents through file contracts. MoC planning
and section writing are performed by skills and, after valid map outputs exist,
section writing may fall back to sequential main-agent execution using the same
section contract.
```

- [x] **Step 3: Run README/contract grep**

Run:

```powershell
rg -n "youtube-summary|workflow_state|Map extraction is performed by sub-agents|main agent or sub-agents" research/youtube_pipeline/README.md
```

Expected:

- lines for `youtube-summary`;
- lines for `workflow_state`;
- line for `Map extraction is performed by sub-agents`;
- no line containing the old phrase `main agent or sub-agents`.

- [x] **Step 4: Commit Task 5**

Run:

```powershell
git add research/youtube_pipeline/README.md
git commit -m "docs: document youtube summary wrapper skill"
```

## Task 6: Full Verification And Cleanup

**Files:**
- Verify all files touched by Tasks 1-5.

- [ ] **Step 1: Run full agentic test module**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_agentic_moc
```

Expected: all tests pass.

- [ ] **Step 2: Run full YouTube pipeline test suite**

Run:

```powershell
python -m unittest discover research/youtube_pipeline/tests
```

Expected: all tests pass.

- [ ] **Step 3: Run whitespace and red-flag checks**

Run:

```powershell
git diff --check
```

Expected: no output and exit code `0`.

Run:

```powershell
$patterns = @("TO" + "DO", "TB" + "D", "implement " + "later", "fill in " + "details", "Similar " + "to")
rg -n ($patterns -join "|") research/youtube_pipeline/youtube_summary_workflow.py research/youtube_pipeline/tools/start_youtube_summary.py research/youtube_pipeline/tools/update_youtube_summary_state.py .agents/skills/youtube-summary/SKILL.md research/youtube_pipeline/README.md
```

Expected: no matches.

- [ ] **Step 4: Run deterministic CLI smoke**

Run:

```powershell
$runRoot = "research/youtube_pipeline/runs/manual/youtube_summary_smoke"
python -m research.youtube_pipeline.tools.start_youtube_summary --transcript research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt --run-root $runRoot --language ru --target-words 10000 --target-tokens 160 --overlap-tokens 30 --planner-context-tokens 3000 --force
```

Expected:

```text
current_stage=map_assignments_ready
next_action=dispatch_map_extractors
resumed=false
```

Run the same command without `--force`:

```powershell
python -m research.youtube_pipeline.tools.start_youtube_summary --transcript research/youtube_pipeline/tests/fixtures/agentic_tiny_transcript.txt --run-root $runRoot --language ru --target-words 10000 --target-tokens 160 --overlap-tokens 30 --planner-context-tokens 3000
```

Expected:

```text
resumed=true
```

- [ ] **Step 5: Check git status**

Run:

```powershell
git status --short
```

Expected: clean working tree after the task commits.

## Self-Review Checklist

- [ ] Spec requirement: one public `youtube-summary` skill starts the workflow.
- [ ] Spec requirement: Python performs deterministic bootstrap only and never calls LLM APIs.
- [ ] Spec requirement: resume uses `transcript_sha256`, `options_hash`, and `run_index.json`.
- [ ] Spec requirement: `run_index.json` updates are atomic.
- [ ] Spec requirement: `workflow_state.json` updates after each validated stage transition.
- [ ] Spec requirement: map extraction pauses when sub-agents are unavailable.
- [ ] Spec requirement: section writing may fall back to main-agent execution only after valid map outputs exist.
- [ ] Spec requirement: overview and synthesis are orchestrator-owned and are not whole-report rewrites.
- [ ] Spec requirement: final response exposes `final/report.md`, `final/metrics.json`, and validation warnings.
- [ ] No direct LLM API markers in Python or the new skill file.
- [ ] Full test suite passes.
