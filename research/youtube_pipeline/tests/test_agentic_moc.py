import json
import tempfile
import unittest
from pathlib import Path

from research.youtube_pipeline.moc_agentic import (
    build_stage_key,
    canonical_fact_id,
    chunk_transcript_text,
    estimate_tokens,
    hash_file,
    hash_text,
    normalize_transcript_text,
    read_json,
    read_jsonl,
    word_count,
    write_prep_artifacts,
    write_json,
    write_jsonl,
)


FIXTURES_DIR = Path(__file__).parent / "fixtures"


def fixture_text(name: str) -> str:
    return (FIXTURES_DIR / name).read_text(encoding="utf-8")


class AgenticArtifactHelperTests(unittest.TestCase):
    def test_hash_text_is_stable(self):
        self.assertEqual(hash_text("alpha\nbeta"), hash_text("alpha\nbeta"))
        self.assertNotEqual(hash_text("alpha\nbeta"), hash_text("alpha\nbeta\n"))

    def test_hash_file_matches_text_hash(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "sample.txt"
            path.write_bytes(b"alpha\nbeta")

            self.assertEqual(hash_file(path), hash_text("alpha\nbeta"))

    def test_stage_key_includes_declared_scope(self):
        key = build_stage_key("extract_facts", {"chunks": "abc", "agent": "youtube-map-extract-v1"})

        self.assertEqual(key["stage"], "extract_facts")
        self.assertEqual(key["scope"], {"agent": "youtube-map-extract-v1", "chunks": "abc"})
        self.assertEqual(len(key["hash"]), 64)

    def test_canonical_fact_id_uses_chunk_and_index(self):
        self.assertEqual(canonical_fact_id("chunk_003", 4), "fact_chunk_003_004")

    def test_canonical_fact_id_rejects_empty_chunk_id(self):
        with self.assertRaises(ValueError):
            canonical_fact_id(" ", 1)

    def test_json_round_trip_uses_sorted_newline_terminated_json(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "nested" / "data.json"
            write_json(path, {"b": 2, "a": 1})

            self.assertEqual(read_json(path), {"a": 1, "b": 2})
            self.assertTrue(path.read_text(encoding="utf-8").endswith("\n"))
            self.assertLess(path.read_text(encoding="utf-8").index('"a"'), path.read_text(encoding="utf-8").index('"b"'))

    def test_jsonl_round_trip_uses_one_object_per_line(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "rows.jsonl"
            write_jsonl(path, [{"b": 2, "a": 1}, {"id": "two"}])

            self.assertEqual(read_jsonl(path), [{"a": 1, "b": 2}, {"id": "two"}])
            self.assertEqual(len(path.read_text(encoding="utf-8").splitlines()), 2)

    def test_read_jsonl_rejects_non_object_rows(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "bad.jsonl"
            path.write_text(json.dumps(["not", "object"]) + "\n", encoding="utf-8")

            with self.assertRaises(ValueError):
                read_jsonl(path)

    def test_word_count_and_language_aware_token_estimate(self):
        text = "One compact sentence with eight plain words."

        self.assertEqual(word_count(text), 7)
        self.assertGreater(estimate_tokens(text, language="ru"), estimate_tokens(text, language="en"))

    def test_normalize_transcript_text_trims_and_keeps_line_boundaries(self):
        raw = "  [00:00:00] hello   world  \n\n  [00:00:10] second line  "

        self.assertEqual(
            normalize_transcript_text(raw),
            "[00:00:00] hello world\n[00:00:10] second line\n",
        )

    def test_chunk_transcript_preserves_timestamps_and_chunk_ids(self):
        transcript = fixture_text("agentic_tiny_transcript.txt")
        chunks = chunk_transcript_text(transcript, target_tokens=160, overlap_tokens=30, language="ru")

        self.assertEqual(chunks[0]["chunk_id"], "chunk_001")
        self.assertEqual(chunks[0]["chunk_index"], 1)
        self.assertEqual(chunks[0]["start_timestamp"], "00:00:00")
        self.assertIn("end_timestamp", chunks[0])
        self.assertGreater(chunks[0]["word_count"], 0)
        self.assertGreater(chunks[0]["estimated_tokens"], 0)
        self.assertEqual(chunks[0]["warnings"], [])

    def test_write_prep_artifacts_writes_manifest_and_chunks(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            output_dir = Path(temp_dir) / "run"
            transcript_path = FIXTURES_DIR / "agentic_tiny_transcript.txt"

            manifest = write_prep_artifacts(
                transcript_path,
                output_dir,
                target_tokens=160,
                overlap_tokens=30,
                language="ru",
            )

            normalized_path = output_dir / "prep" / "normalized_transcript.txt"
            chunks_path = output_dir / "prep" / "chunks.jsonl"
            manifest_path = output_dir / "prep" / "manifest.json"

            self.assertTrue(normalized_path.exists())
            self.assertTrue(chunks_path.exists())
            self.assertTrue(manifest_path.exists())
            self.assertTrue(normalized_path.read_text(encoding="utf-8").endswith("\n"))
            self.assertEqual(read_json(manifest_path), manifest)
            self.assertEqual(manifest["chunk_count"], len(read_jsonl(chunks_path)))
            self.assertEqual(manifest["language"], "ru")


if __name__ == "__main__":
    unittest.main()
