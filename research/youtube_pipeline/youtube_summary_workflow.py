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
