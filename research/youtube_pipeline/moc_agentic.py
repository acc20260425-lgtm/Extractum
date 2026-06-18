import hashlib
import json
import math
import re
from collections.abc import Iterable, Mapping
from pathlib import Path
from typing import Any

from research.youtube_pipeline.moc import (
    chunk_segments_by_approx_tokens,
    format_ms,
    parse_timestamped_transcript,
)


def hash_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def build_stage_key(stage: str, scope: Mapping[str, str]) -> dict[str, object]:
    normalized_scope = {key: str(scope[key]) for key in sorted(scope)}
    key_payload = {
        "stage": stage,
        "scope": normalized_scope,
    }
    return {
        **key_payload,
        "hash": hash_text(json.dumps(key_payload, ensure_ascii=False, sort_keys=True, separators=(",", ":"))),
    }


def write_json(path: Path, data: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def write_jsonl(path: Path, rows: Iterable[Mapping[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        for row in rows:
            handle.write(json.dumps(dict(row), ensure_ascii=False, sort_keys=True) + "\n")


def read_jsonl(path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        payload = json.loads(line)
        if not isinstance(payload, dict):
            raise ValueError(f"JSONL row must be an object: {path}")
        rows.append(payload)
    return rows


def canonical_fact_id(chunk_id: str, local_index: int) -> str:
    if local_index < 1:
        raise ValueError("local_index must be positive")
    normalized_chunk_id = chunk_id.strip()
    if not normalized_chunk_id:
        raise ValueError("chunk_id must be non-empty")
    return f"fact_{normalized_chunk_id}_{local_index:03d}"


def word_count(text: str) -> int:
    return len(re.findall(r"\b\w+(?:[-']\w+)*\b", text, flags=re.UNICODE))


def estimate_tokens(text: str, language: str = "ru") -> int:
    words = word_count(text)
    if words == 0:
        return 0
    ratio = 2.2 if language.lower().startswith("ru") else 1.4
    word_estimate = math.ceil(words * ratio)
    character_floor = math.ceil(len(text) / 4)
    return max(1, word_estimate, character_floor)


def normalize_transcript_text(text: str) -> str:
    lines = [" ".join(line.strip().split()) for line in text.splitlines() if line.strip()]
    if not lines:
        return ""
    return "\n".join(lines) + "\n"


def chunk_transcript_text(
    text: str,
    *,
    target_tokens: int,
    overlap_tokens: int,
    language: str,
) -> list[dict[str, object]]:
    normalized = normalize_transcript_text(text)
    segments, warnings = parse_timestamped_transcript(normalized)
    segment_chunks = chunk_segments_by_approx_tokens(
        segments,
        max_tokens=target_tokens,
        overlap_tokens=overlap_tokens,
    )
    source_hash = hash_text(normalized)
    chunks: list[dict[str, object]] = []
    for chunk in segment_chunks:
        chunks.append(
            {
                "chunk_id": f"chunk_{chunk.chunk_index:03d}",
                "chunk_index": chunk.chunk_index,
                "start_timestamp": format_ms(chunk.start_ms),
                "end_timestamp": format_ms(chunk.end_ms),
                "text": chunk.text,
                "word_count": word_count(chunk.text),
                "estimated_tokens": estimate_tokens(chunk.text, language=language),
                "source_hash": source_hash,
                "warnings": warnings,
            }
        )
    return chunks


def write_prep_artifacts(
    transcript_path: Path,
    output_dir: Path,
    *,
    target_tokens: int,
    overlap_tokens: int,
    language: str,
) -> dict[str, object]:
    raw_text = transcript_path.read_text(encoding="utf-8")
    normalized = normalize_transcript_text(raw_text)
    chunks = chunk_transcript_text(
        normalized,
        target_tokens=target_tokens,
        overlap_tokens=overlap_tokens,
        language=language,
    )
    prep_dir = output_dir / "prep"
    normalized_path = prep_dir / "normalized_transcript.txt"
    chunks_path = prep_dir / "chunks.jsonl"
    manifest_path = prep_dir / "manifest.json"

    normalized_path.parent.mkdir(parents=True, exist_ok=True)
    normalized_path.write_text(normalized, encoding="utf-8", newline="\n")
    write_jsonl(chunks_path, chunks)

    manifest: dict[str, object] = {
        "schema": "agentic-prep-manifest-v1",
        "transcript_path": str(transcript_path),
        "raw_transcript_hash": hash_text(raw_text),
        "normalized_transcript_hash": hash_text(normalized),
        "normalized_transcript_file": "prep/normalized_transcript.txt",
        "chunks_file": "prep/chunks.jsonl",
        "chunk_count": len(chunks),
        "target_tokens": target_tokens,
        "overlap_tokens": overlap_tokens,
        "language": language,
        "warnings": sorted({warning for chunk in chunks for warning in chunk.get("warnings", [])}),
    }
    write_json(manifest_path, manifest)
    return manifest


def timestamp_to_ms(timestamp: str) -> int | None:
    if not timestamp:
        return None
    parts = timestamp.split(":")
    if len(parts) != 3:
        return None
    try:
        hours, minutes, seconds = (int(part) for part in parts)
    except ValueError:
        return None
    return ((hours * 60 + minutes) * 60 + seconds) * 1000


def prepare_map_assignments(
    run_dir: Path,
    *,
    output_language: str,
    target_summary_words: int = 250,
    max_fact_count: int = 20,
) -> dict[str, object]:
    chunks = read_jsonl(run_dir / "prep" / "chunks.jsonl")
    assignments_dir = run_dir / "map" / "assignments"
    expected_files_dir = run_dir / "map" / "expected_files"
    assignments: list[dict[str, object]] = []

    for chunk in chunks:
        chunk_id = str(chunk["chunk_id"])
        output_file = f"map/agent_outputs/{chunk_id}.json"
        assignment = {
            "chunk_id": chunk_id,
            "output_file": output_file,
            "time_range": {
                "start_ms": timestamp_to_ms(str(chunk.get("start_timestamp", ""))),
                "end_ms": timestamp_to_ms(str(chunk.get("end_timestamp", ""))),
            },
            "output_language": output_language,
            "transcript_text": str(chunk.get("text", "")),
            "target_summary_words": target_summary_words,
            "max_fact_count": max_fact_count,
        }
        write_json(assignments_dir / f"{chunk_id}.assignment.json", assignment)
        assignments.append(assignment)

    expected_files = [assignment["output_file"] for assignment in assignments]
    write_json(
        expected_files_dir / "mapper_batch_001.json",
        {"agent_id": "mapper_batch_001", "expected_files": expected_files},
    )
    manifest = {
        "schema": "agentic-map-assignment-manifest-v1",
        "assignment_count": len(assignments),
        "assignments": [f"map/assignments/{assignment['chunk_id']}.assignment.json" for assignment in assignments],
        "expected_files_manifest": "map/expected_files/mapper_batch_001.json",
        "output_language": output_language,
        "chunks_hash": hash_file(run_dir / "prep" / "chunks.jsonl"),
    }
    write_json(run_dir / "map" / "assignment_manifest.json", manifest)
    return manifest


def _load_json_with_light_repair(path: Path) -> tuple[object | None, dict[str, object]]:
    raw = path.read_text(encoding="utf-8")
    try:
        return json.loads(raw), {"attempted": False, "applied": False, "error": ""}
    except json.JSONDecodeError as original_error:
        start = raw.find("{")
        end = raw.rfind("}")
        if start == -1 or end == -1 or end <= start:
            return None, {"attempted": True, "applied": False, "error": str(original_error)}
        candidate = raw[start : end + 1]
        try:
            return json.loads(candidate), {"attempted": True, "applied": True, "error": ""}
        except json.JSONDecodeError as repair_error:
            return None, {"attempted": True, "applied": False, "error": str(repair_error)}


def _validate_map_payload(payload: object, expected_chunk_id: str) -> list[str]:
    errors: list[str] = []
    if not isinstance(payload, dict):
        return ["output must be a JSON object"]
    if payload.get("chunk_id") != expected_chunk_id:
        errors.append("chunk_id does not match assignment")
    if not isinstance(payload.get("chunk_summary"), str) or not str(payload.get("chunk_summary")).strip():
        errors.append("chunk_summary must be a non-empty string")
    facts = payload.get("facts")
    if not isinstance(facts, list):
        errors.append("facts must be a list")
        return errors
    for index, fact in enumerate(facts, start=1):
        if not isinstance(fact, dict):
            errors.append(f"facts[{index}] must be an object")
            continue
        for key in ("local_fact_id", "text", "fact_type", "timestamp", "importance", "chunk_id"):
            if key not in fact:
                errors.append(f"facts[{index}] missing {key}")
    return errors


def validate_map_outputs(run_dir: Path) -> dict[str, object]:
    manifest = read_json(run_dir / "map" / "assignment_manifest.json")
    if not isinstance(manifest, dict):
        raise ValueError("assignment manifest must be an object")

    valid_outputs: list[str] = []
    invalid_outputs: list[dict[str, object]] = []
    repair_attempts: list[dict[str, object]] = []

    for assignment_path_text in manifest.get("assignments", []):
        assignment = read_json(run_dir / str(assignment_path_text))
        if not isinstance(assignment, dict):
            raise ValueError(f"assignment must be an object: {assignment_path_text}")
        output_file = str(assignment["output_file"])
        output_path = run_dir / output_file
        if not output_path.exists():
            invalid_outputs.append({"output_file": output_file, "errors": ["output file missing"]})
            continue

        payload, repair = _load_json_with_light_repair(output_path)
        repair_attempts.append({"output_file": output_file, **repair})
        errors = _validate_map_payload(payload, str(assignment["chunk_id"]))
        if errors:
            invalid_outputs.append({"output_file": output_file, "errors": errors})
        else:
            if repair.get("applied"):
                write_json(output_path, payload)
            valid_outputs.append(output_file)

    validation_manifest = {
        "schema": "agentic-map-validation-manifest-v1",
        "valid_outputs": valid_outputs,
        "invalid_outputs": invalid_outputs,
        "repair_attempts": repair_attempts,
    }
    write_json(run_dir / "map" / "validation_manifest.json", validation_manifest)
    return validation_manifest


def assemble_map_artifacts(run_dir: Path) -> dict[str, object]:
    validation = read_json(run_dir / "map" / "validation_manifest.json")
    if not isinstance(validation, dict):
        raise ValueError("validation manifest must be an object")

    chunk_summaries: list[dict[str, object]] = []
    raw_facts: list[dict[str, object]] = []
    mapped_facts: list[dict[str, object]] = []

    for output_file in validation.get("valid_outputs", []):
        payload = read_json(run_dir / str(output_file))
        if not isinstance(payload, dict):
            raise ValueError(f"map output must be an object: {output_file}")
        chunk_id = str(payload["chunk_id"])
        chunk_summaries.append(
            {
                "chunk_id": chunk_id,
                "chunk_summary": str(payload.get("chunk_summary", "")),
                "claims": payload.get("claims", []),
                "examples": payload.get("examples", []),
                "quotes": payload.get("quotes", []),
                "entities": payload.get("entities", []),
                "open_questions": payload.get("open_questions", []),
            }
        )
        facts = payload.get("facts", [])
        if not isinstance(facts, list):
            continue
        for index, fact in enumerate(facts, start=1):
            if not isinstance(fact, dict):
                continue
            raw_fact = {"chunk_id": chunk_id, **fact}
            mapped_fact = {
                "fact_id": canonical_fact_id(chunk_id, index),
                "chunk_id": chunk_id,
                "local_fact_id": str(fact.get("local_fact_id", "")),
                "text": str(fact.get("text", "")),
                "fact_type": str(fact.get("fact_type", "")),
                "timestamp": str(fact.get("timestamp", "")),
                "importance": int(fact.get("importance", 0)),
            }
            raw_facts.append(raw_fact)
            mapped_facts.append(mapped_fact)

    chunk_summaries.sort(key=lambda row: str(row["chunk_id"]))
    mapped_facts.sort(key=lambda row: str(row["fact_id"]))
    raw_facts.sort(key=lambda row: (str(row["chunk_id"]), str(row.get("local_fact_id", ""))))

    write_jsonl(run_dir / "map" / "chunk_summaries.jsonl", chunk_summaries)
    write_jsonl(run_dir / "map" / "mapped_facts.raw.jsonl", raw_facts)
    write_jsonl(run_dir / "map" / "mapped_facts.jsonl", mapped_facts)
    write_jsonl(run_dir / "map" / "quarantine.jsonl", validation.get("invalid_outputs", []))

    assembly_manifest = {
        "schema": "agentic-map-assembly-manifest-v1",
        "chunk_summary_count": len(chunk_summaries),
        "mapped_fact_count": len(mapped_facts),
        "quarantine_count": len(validation.get("invalid_outputs", [])),
    }
    write_json(run_dir / "map" / "assembly_manifest.json", assembly_manifest)

    map_manifest = {
        "schema": "agentic-map-manifest-v1",
        "assignment_manifest": "map/assignment_manifest.json",
        "validation_manifest": "map/validation_manifest.json",
        "assembly_manifest": "map/assembly_manifest.json",
        **assembly_manifest,
    }
    write_json(run_dir / "map" / "map_manifest.json", map_manifest)
    return map_manifest
