import hashlib
import json
import math
import re
from collections.abc import Iterable, Mapping
from pathlib import Path
from typing import Any


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
