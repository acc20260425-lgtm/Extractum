import json
import tempfile
import unittest
from pathlib import Path

from research.youtube_pipeline.moc_agentic import (
    build_stage_key,
    canonical_fact_id,
    estimate_tokens,
    hash_file,
    hash_text,
    read_json,
    read_jsonl,
    word_count,
    write_json,
    write_jsonl,
)


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


if __name__ == "__main__":
    unittest.main()
