from __future__ import annotations

import hashlib
import json
import os
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Mapping

from research.youtube_pipeline.moc_agentic import prepare_map_assignments, read_json, write_json, write_prep_artifacts

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
            "run_root": str(run_root),
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
    if not force:
        match = find_latest_matching_run(run_root, transcript_hash=transcript_hash, options_hash=options_digest)
        if match:
            state = read_json(Path(str(match["run_dir"])) / "workflow_state.json")
            if isinstance(state, dict):
                state["resumed"] = True
                return state
            raise ValueError(f"Matching workflow_state.json is invalid for run: {match['run_dir']}")
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
