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
