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


def build_planner_context(
    run_dir: Path,
    *,
    max_tokens: int,
    language: str,
) -> dict[str, object]:
    chunks = read_jsonl(run_dir / "prep" / "chunks.jsonl")
    summaries = read_jsonl(run_dir / "map" / "chunk_summaries.jsonl")
    facts = read_jsonl(run_dir / "map" / "mapped_facts.jsonl")
    summary_by_chunk = {str(row["chunk_id"]): row for row in summaries}
    facts_by_chunk: dict[str, list[dict[str, object]]] = {}
    for fact in facts:
        facts_by_chunk.setdefault(str(fact.get("chunk_id", "")), []).append(fact)

    lines = [
        "# Planner Context",
        "",
        "Use this bounded context to create a Map of Content. Do not invent facts.",
        "",
    ]
    included_chunk_count = 0
    included_fact_count = 0
    truncated = False

    for chunk in chunks:
        chunk_id = str(chunk["chunk_id"])
        summary = summary_by_chunk.get(chunk_id, {})
        candidate_lines = [
            f"## {chunk_id} [{chunk.get('start_timestamp')} - {chunk.get('end_timestamp')}]",
            str(summary.get("chunk_summary", "")),
        ]
        for fact in facts_by_chunk.get(chunk_id, []):
            candidate_lines.append(
                f"- {fact.get('fact_id')}: {fact.get('text')} ({fact.get('timestamp')}, importance {fact.get('importance')})"
            )
        candidate_lines.append("")
        candidate = "\n".join(lines + candidate_lines)
        if estimate_tokens(candidate, language=language) > max_tokens and included_chunk_count > 0:
            truncated = True
            break
        lines.extend(candidate_lines)
        included_chunk_count += 1
        included_fact_count += len(facts_by_chunk.get(chunk_id, []))

    context = "\n".join(lines).rstrip() + "\n"
    context_path = run_dir / "planning" / "planner_context.md"
    context_path.parent.mkdir(parents=True, exist_ok=True)
    context_path.write_text(context, encoding="utf-8", newline="\n")
    metadata = {
        "schema": "agentic-planner-context-metadata-v1",
        "max_tokens": max_tokens,
        "estimated_tokens": estimate_tokens(context, language=language),
        "language": language,
        "included_chunk_count": included_chunk_count,
        "total_chunk_count": len(chunks),
        "included_fact_count": included_fact_count,
        "total_fact_count": len(facts),
        "truncated": truncated or included_chunk_count < len(chunks),
        "chunk_summaries_hash": hash_file(run_dir / "map" / "chunk_summaries.jsonl"),
        "mapped_facts_hash": hash_file(run_dir / "map" / "mapped_facts.jsonl"),
    }
    write_json(run_dir / "planning" / "planner_context_metadata.json", metadata)
    return metadata


def _chunk_ids_from_prep(run_dir: Path) -> list[str]:
    return [str(chunk["chunk_id"]) for chunk in read_jsonl(run_dir / "prep" / "chunks.jsonl")]


def _fallback_moc(run_dir: Path, *, target_words: int, chapter_target_words: int) -> dict[str, object]:
    chunks = read_jsonl(run_dir / "prep" / "chunks.jsonl")
    if not chunks:
        node_count = 1
    else:
        node_count = max(1, round(target_words / chapter_target_words))
        node_count = min(node_count, len(chunks))

    nodes: list[dict[str, object]] = []
    for index in range(node_count):
        start = index * len(chunks) // node_count
        end = (index + 1) * len(chunks) // node_count
        assigned = chunks[start:end] or chunks[:1]
        chunk_ids = [str(chunk["chunk_id"]) for chunk in assigned]
        nodes.append(
            {
                "node_id": f"moc_{index + 1:03d}",
                "title": f"Section {index + 1}",
                "purpose": "Fallback contiguous transcript section.",
                "target_words": max(1, round(target_words / node_count)),
                "time_range": {
                    "start_ms": timestamp_to_ms(str(assigned[0].get("start_timestamp", ""))),
                    "end_ms": timestamp_to_ms(str(assigned[-1].get("end_timestamp", ""))),
                },
                "chunk_ids": chunk_ids,
                "key_questions": [],
                "required_fact_types": ["claim", "example", "quote"],
                "fallback": True,
            }
        )
    return {
        "report_title": "YouTube Transcript Report",
        "source_kind": "youtube_video_transcript",
        "report_thesis": "Fallback MoC generated from contiguous transcript chunks.",
        "target_words": target_words,
        "nodes": nodes,
    }


def validate_moc(
    run_dir: Path,
    *,
    target_words: int,
    chapter_target_words: int = 900,
) -> dict[str, object]:
    raw_path = run_dir / "planning" / "moc.raw.json"
    payload = read_json(raw_path) if raw_path.exists() else {}
    available_chunk_ids = _chunk_ids_from_prep(run_dir)
    errors: list[str] = []
    warnings: list[str] = []

    if not isinstance(payload, dict):
        errors.append("MoC must be a JSON object")
        payload = {}
    nodes = payload.get("nodes")
    if not isinstance(nodes, list) or not nodes:
        errors.append("MoC nodes must be a non-empty list")
        nodes = []

    seen_chunk_ids: dict[str, str] = {}
    covered_chunk_ids: set[str] = set()
    chunk_order = {chunk_id: index for index, chunk_id in enumerate(available_chunk_ids)}
    previous_node_min_index = -1
    for node_index, node in enumerate(nodes, start=1):
        if not isinstance(node, dict):
            errors.append(f"nodes[{node_index}] must be an object")
            continue
        node_id = str(node.get("node_id") or f"moc_{node_index:03d}")
        chunk_ids = node.get("chunk_ids")
        if not isinstance(chunk_ids, list) or not chunk_ids:
            errors.append(f"{node_id} must include chunk_ids")
            continue
        if int(node.get("target_words", 0) or 0) <= 0:
            errors.append(f"{node_id} target_words must be positive")
        node_indexes: list[int] = []
        for chunk_id_value in chunk_ids:
            chunk_id = str(chunk_id_value)
            if chunk_id not in available_chunk_ids:
                errors.append(f"{node_id} references unknown chunk {chunk_id}")
                continue
            node_indexes.append(chunk_order[chunk_id])
            if chunk_id in seen_chunk_ids and not node.get("overlap_reason"):
                errors.append(f"{chunk_id} appears in multiple nodes without overlap_reason")
            seen_chunk_ids[chunk_id] = node_id
            covered_chunk_ids.add(chunk_id)
        if node_indexes:
            node_min_index = min(node_indexes)
            if node_min_index < previous_node_min_index and not node.get("thematic_reason"):
                errors.append(f"{node_id} chunk order is non-ascending without thematic_reason")
            previous_node_min_index = max(previous_node_min_index, node_min_index)

    missing_chunk_ids = [chunk_id for chunk_id in available_chunk_ids if chunk_id not in covered_chunk_ids]
    if missing_chunk_ids:
        errors.append("MoC missing chunk coverage: " + ", ".join(missing_chunk_ids))

    fallback_used = bool(errors)
    if fallback_used:
        moc = _fallback_moc(run_dir, target_words=target_words, chapter_target_words=chapter_target_words)
    else:
        moc = {
            "report_title": str(payload.get("report_title") or "YouTube Transcript Report"),
            "source_kind": str(payload.get("source_kind") or "youtube_video_transcript"),
            "report_thesis": str(payload.get("report_thesis") or ""),
            "target_words": int(payload.get("target_words") or target_words),
            "nodes": nodes,
        }
        if not moc["report_thesis"]:
            warnings.append("report_thesis_empty")

    write_json(run_dir / "planning" / "moc.json", moc)
    validation = {
        "schema": "agentic-moc-validation-v1",
        "valid": not fallback_used,
        "fallback_used": fallback_used,
        "errors": errors,
        "warnings": warnings,
        "available_chunk_ids": available_chunk_ids,
        "covered_chunk_ids": sorted(covered_chunk_ids),
    }
    write_json(run_dir / "planning" / "moc_validation.json", validation)
    return validation


def _dedupe_key(text: str) -> str:
    return " ".join(re.findall(r"\w+", text.lower(), flags=re.UNICODE))


def dedupe_facts(run_dir: Path) -> list[dict[str, object]]:
    facts = read_jsonl(run_dir / "map" / "mapped_facts.jsonl")
    clusters_by_key: dict[str, dict[str, object]] = {}

    for fact in facts:
        text = str(fact.get("text", "")).strip()
        key = _dedupe_key(text) or str(fact.get("fact_id", ""))
        cluster = clusters_by_key.get(key)
        if cluster is None:
            cluster = {
                "fact_id": f"fact_cluster_{len(clusters_by_key) + 1:04d}",
                "claim": text,
                "evidence": text,
                "tags": sorted({str(fact.get("fact_type", ""))}),
                "source_fact_ids": [],
                "source_chunk_ids": [],
                "source_timestamps": [],
            }
            clusters_by_key[key] = cluster
        _append_unique(cluster["source_fact_ids"], str(fact.get("fact_id", "")))
        _append_unique(cluster["source_chunk_ids"], str(fact.get("chunk_id", "")))
        _append_unique(cluster["source_timestamps"], str(fact.get("timestamp", "")))

    deduplicated = list(clusters_by_key.values())
    write_json(run_dir / "alignment" / "deduplicated_facts.json", deduplicated)
    write_json(
        run_dir / "alignment" / "dedupe_report.json",
        {
            "schema": "agentic-dedupe-report-v1",
            "input_fact_count": len(facts),
            "deduplicated_fact_count": len(deduplicated),
        },
    )
    return deduplicated


def _append_unique(values: object, value: str) -> None:
    if not isinstance(values, list) or not value:
        return
    if value not in values:
        values.append(value)


def align_facts(run_dir: Path) -> dict[str, object]:
    moc = read_json(run_dir / "planning" / "moc.json")
    facts = read_json(run_dir / "alignment" / "deduplicated_facts.json")
    if not isinstance(moc, dict) or not isinstance(facts, list):
        raise ValueError("MoC and deduplicated facts must be available before alignment")

    node_alignments: list[dict[str, object]] = []
    aligned_fact_ids: set[str] = set()
    for node in moc.get("nodes", []):
        if not isinstance(node, dict):
            continue
        node_chunk_id_list = [str(chunk_id) for chunk_id in node.get("chunk_ids", [])]
        node_chunk_ids = set(node_chunk_id_list)
        node_fact_ids: list[str] = []
        for fact in facts:
            if not isinstance(fact, dict):
                continue
            source_chunk_ids = {str(chunk_id) for chunk_id in fact.get("source_chunk_ids", [])}
            if node_chunk_ids.intersection(source_chunk_ids):
                fact_id = str(fact["fact_id"])
                node_fact_ids.append(fact_id)
                aligned_fact_ids.add(fact_id)
        node_alignments.append(
            {
                "node_id": str(node.get("node_id", "")),
                "chunk_ids": node_chunk_id_list,
                "aligned_fact_ids": node_fact_ids,
            }
        )

    unaligned = [fact for fact in facts if isinstance(fact, dict) and str(fact.get("fact_id")) not in aligned_fact_ids]
    alignment = {
        "schema": "agentic-fact-alignment-v1",
        "nodes": node_alignments,
        "aligned_fact_count": len(aligned_fact_ids),
        "unaligned_fact_count": len(unaligned),
    }
    write_json(run_dir / "alignment" / "alignment.json", alignment)
    write_json(run_dir / "alignment" / "unaligned_facts.json", unaligned)
    return alignment


def prepare_section_assignments(run_dir: Path) -> list[dict[str, object]]:
    moc = read_json(run_dir / "planning" / "moc.json")
    alignment = read_json(run_dir / "alignment" / "alignment.json")
    facts = read_json(run_dir / "alignment" / "deduplicated_facts.json")
    if not isinstance(moc, dict) or not isinstance(alignment, dict) or not isinstance(facts, list):
        raise ValueError("MoC, alignment, and deduplicated facts must be available")

    facts_by_id = {str(fact["fact_id"]): fact for fact in facts if isinstance(fact, dict)}
    aligned_nodes = {
        str(node["node_id"]): node for node in alignment.get("nodes", []) if isinstance(node, dict)
    }
    fact_usage: dict[str, int] = {}
    for node in aligned_nodes.values():
        for fact_id in node.get("aligned_fact_ids", []):
            fact_usage[str(fact_id)] = fact_usage.get(str(fact_id), 0) + 1

    assignments: list[dict[str, object]] = []
    for index, node in enumerate(moc.get("nodes", []), start=1):
        if not isinstance(node, dict):
            continue
        node_id = str(node.get("node_id", f"moc_{index:03d}"))
        aligned_fact_ids = [str(fact_id) for fact_id in aligned_nodes.get(node_id, {}).get("aligned_fact_ids", [])]
        source_timestamps: list[str] = []
        for fact_id in aligned_fact_ids:
            fact = facts_by_id.get(fact_id, {})
            for timestamp in fact.get("source_timestamps", []):
                _append_unique(source_timestamps, str(timestamp))
        assignment = {
            "node_id": node_id,
            "title": str(node.get("title", f"Section {index}")),
            "section_file": f"sections/{index:03d}-{_slugify(str(node.get('title', f'section-{index}')))}.md",
            "target_words": int(node.get("target_words", 0) or 0),
            "chunk_ids": [str(chunk_id) for chunk_id in node.get("chunk_ids", [])],
            "aligned_fact_ids": aligned_fact_ids,
            "source_timestamps": source_timestamps,
            "overlap_fact_ids": [fact_id for fact_id in aligned_fact_ids if fact_usage.get(fact_id, 0) > 1],
        }
        assignments.append(assignment)

    write_jsonl(run_dir / "alignment" / "section_assignments.jsonl", assignments)
    return assignments


def _slugify(value: str) -> str:
    words = re.findall(r"\w+", value.lower(), flags=re.UNICODE)
    return "-".join(words[:8]) or "section"


def validate_generated_files(
    run_dir: Path,
    *,
    agent_id: str,
    expected_files: Iterable[str],
) -> dict[str, object]:
    expected = [str(path) for path in expected_files]
    missing_files = [path for path in expected if not (run_dir / path).exists()]
    expected_set = set(expected)
    unexpected_files: list[str] = []

    for expected_file in expected:
        parent = (run_dir / expected_file).parent
        if not parent.exists():
            continue
        for path in parent.iterdir():
            if path.is_file():
                relative = path.relative_to(run_dir).as_posix()
                if relative not in expected_set:
                    unexpected_files.append(relative)

    result = {
        "schema": "agentic-generated-files-validation-v1",
        "agent_id": agent_id,
        "valid": not missing_files and not unexpected_files,
        "expected_files": expected,
        "missing_files": sorted(missing_files),
        "unexpected_files": sorted(set(unexpected_files)),
    }
    write_json(run_dir / "review" / "generated_files_validation.json", result)
    return result


def stage_is_reusable(run_dir: Path, manifest_path: Path, current_stage_key: Mapping[str, object]) -> bool:
    if not manifest_path.exists():
        return False
    manifest = read_json(manifest_path)
    if not isinstance(manifest, dict):
        return False
    stored_key = manifest.get("stage_key")
    if not isinstance(stored_key, dict):
        return False
    if stored_key.get("hash") != current_stage_key.get("hash"):
        return False
    output_files = manifest.get("output_files", [])
    if not isinstance(output_files, list):
        return False
    return all((run_dir / str(path)).exists() for path in output_files)


def quality_check(run_dir: Path) -> dict[str, object]:
    assignments_path = run_dir / "alignment" / "section_assignments.jsonl"
    assignments = read_jsonl(assignments_path) if assignments_path.exists() else []
    expected_sections = [str(row.get("section_file", "")) for row in assignments]
    boundary_sections = ["sections/000-overview.md", "sections/999-synthesis.md"]
    expected_files = boundary_sections + expected_sections
    missing_files = [path for path in expected_files if path and not (run_dir / path).exists()]
    section_order_valid = expected_sections == sorted(expected_sections)

    section_word_counts: dict[str, int] = {}
    headings: list[str] = []
    paragraphs: dict[str, list[str]] = {}
    source_label_count = 0
    for relative_path in expected_files:
        path = run_dir / relative_path
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        section_word_counts[relative_path] = word_count(text)
        source_label_count += len(re.findall(r"\b(video summary|youtube transcript|speaker)\b", text.lower()))
        for line in text.splitlines():
            if line.startswith("#"):
                headings.append(line.strip().lower())
        for paragraph in [part.strip() for part in text.split("\n\n") if part.strip()]:
            normalized = " ".join(paragraph.lower().split())
            paragraphs.setdefault(normalized, []).append(relative_path)

    duplicate_headings = sorted({heading for heading in headings if headings.count(heading) > 1})
    duplicate_paragraphs = [
        {"paragraph": paragraph, "files": files}
        for paragraph, files in paragraphs.items()
        if len(files) > 1 and word_count(paragraph) >= 12
    ]
    final_report_path = run_dir / "final" / "report.md"
    final_report = final_report_path.read_text(encoding="utf-8") if final_report_path.exists() else ""
    source_note_present = "summary and analysis of a youtube video transcript" in final_report.lower()
    coverage = {
        "schema": "agentic-coverage-v1",
        "valid": not missing_files,
        "missing_files": missing_files,
        "section_order_valid": section_order_valid,
        "section_word_counts": section_word_counts,
        "total_section_words": sum(section_word_counts.values()),
        "duplicate_headings": duplicate_headings,
        "duplicate_paragraphs": duplicate_paragraphs,
        "source_note_present": source_note_present,
        "source_label_count": source_label_count,
        "source_label_overuse": source_label_count > 2,
    }
    write_json(run_dir / "review" / "coverage.json", coverage)
    coverage_md = [
        "# Coverage",
        "",
        f"- Missing files: {len(missing_files)}",
        f"- Total section words: {coverage['total_section_words']}",
        f"- Duplicate headings: {len(duplicate_headings)}",
        f"- Duplicate paragraphs: {len(duplicate_paragraphs)}",
        f"- Source label count: {source_label_count}",
        "",
    ]
    (run_dir / "review").mkdir(parents=True, exist_ok=True)
    (run_dir / "review" / "coverage.md").write_text("\n".join(coverage_md), encoding="utf-8", newline="\n")
    return coverage


def build_structured_analysis(run_dir: Path) -> dict[str, object]:
    facts = read_json(run_dir / "alignment" / "deduplicated_facts.json")
    alignment = read_json(run_dir / "alignment" / "alignment.json")
    coverage = read_json(run_dir / "review" / "coverage.json") if (run_dir / "review" / "coverage.json").exists() else {}
    if not isinstance(facts, list) or not isinstance(alignment, dict):
        raise ValueError("deduplicated facts and alignment are required")

    lines = ["## Structured Analysis", ""]
    lines.append(f"- Deduplicated facts: {len(facts)}")
    lines.append(f"- Aligned facts: {alignment.get('aligned_fact_count', 0)}")
    if isinstance(coverage, dict):
        lines.append(f"- Missing generated files: {len(coverage.get('missing_files', []))}")
    lines.append("")
    lines.append("### Key Evidence")
    for fact in facts[:20]:
        if not isinstance(fact, dict):
            continue
        timestamps = ", ".join(str(value) for value in fact.get("source_timestamps", []))
        lines.append(f"- {fact.get('claim')} ({timestamps})")
    content = "\n".join(lines).rstrip() + "\n"
    structured_path = run_dir / "review" / "structured_analysis.md"
    structured_path.parent.mkdir(parents=True, exist_ok=True)
    structured_path.write_text(content, encoding="utf-8", newline="\n")
    manifest = {
        "schema": "agentic-structured-analysis-manifest-v1",
        "fact_count": len(facts),
        "structured_analysis_file": "review/structured_analysis.md",
    }
    write_json(run_dir / "review" / "structured_analysis_manifest.json", manifest)
    return manifest


def assemble_report(run_dir: Path) -> dict[str, object]:
    moc = read_json(run_dir / "planning" / "moc.json")
    assignments = read_jsonl(run_dir / "alignment" / "section_assignments.jsonl")
    if not isinstance(moc, dict):
        raise ValueError("MoC must be available before assembly")

    title = str(moc.get("report_title") or "YouTube Transcript Report")
    overview_file = "sections/000-overview.md"
    narrative_section_files = [str(row.get("section_file", "")) for row in assignments]
    synthesis_file = "sections/999-synthesis.md"
    toc_files = [overview_file, *narrative_section_files, "review/structured_analysis.md", synthesis_file]

    report_parts = [
        f"# {title}",
        "",
        "This report is a summary and analysis of a YouTube video transcript.",
        "",
        "## Table of Contents",
    ]
    for section_file in toc_files:
        if not section_file:
            continue
        heading = _first_heading((run_dir / section_file).read_text(encoding="utf-8")) if (run_dir / section_file).exists() else section_file
        report_parts.append(f"- {heading}")
    report_parts.append("")

    for section_file in [overview_file, *narrative_section_files]:
        path = run_dir / section_file
        if path.exists():
            report_parts.append(path.read_text(encoding="utf-8").strip())
            report_parts.append("")

    structured_path = run_dir / "review" / "structured_analysis.md"
    if structured_path.exists():
        report_parts.append(structured_path.read_text(encoding="utf-8").strip())
        report_parts.append("")

    synthesis_path = run_dir / synthesis_file
    if synthesis_path.exists():
        report_parts.append(synthesis_path.read_text(encoding="utf-8").strip())
        report_parts.append("")

    report = "\n".join(report_parts).rstrip() + "\n"
    final_dir = run_dir / "final"
    final_dir.mkdir(parents=True, exist_ok=True)
    (final_dir / "report.md").write_text(report, encoding="utf-8", newline="\n")
    metrics = {
        "strategy": "moc_agentic_writer",
        "entry_point": "agentic_skill",
        "word_count": word_count(report),
        "section_count": len(toc_files),
    }
    write_json(final_dir / "metrics.json", metrics)
    result = {
        "report_file": "final/report.md",
        "metrics_file": "final/metrics.json",
        "word_count": metrics["word_count"],
    }
    write_json(final_dir / "result.json", result)
    return result


def _first_heading(markdown: str) -> str:
    for line in markdown.splitlines():
        if line.startswith("#"):
            return line.lstrip("#").strip()
    return "Untitled section"
