import json
import tempfile
import unittest
from pathlib import Path

from research.youtube_pipeline.moc_agentic import (
    assemble_map_artifacts,
    build_stage_key,
    canonical_fact_id,
    chunk_transcript_text,
    estimate_tokens,
    hash_file,
    hash_text,
    normalize_transcript_text,
    prepare_map_assignments,
    read_json,
    read_jsonl,
    validate_map_outputs,
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

    def test_prepare_map_assignments_writes_assignment_and_expected_files_manifest(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_single_chunk_prep(Path(temp_dir))

            manifest = prepare_map_assignments(run_dir, output_language="ru")

            self.assertEqual(manifest["assignment_count"], 1)
            assignment_path = run_dir / "map" / "assignments" / "chunk_001.assignment.json"
            expected_files_path = run_dir / "map" / "expected_files" / "mapper_batch_001.json"
            assignment = read_json(assignment_path)
            expected_files = read_json(expected_files_path)

            self.assertEqual(assignment["chunk_id"], "chunk_001")
            self.assertEqual(assignment["output_file"], "map/agent_outputs/chunk_001.json")
            self.assertEqual(expected_files["expected_files"], ["map/agent_outputs/chunk_001.json"])

    def test_validate_map_outputs_repairs_wrapped_json(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_single_chunk_prep(Path(temp_dir))
            prepare_map_assignments(run_dir, output_language="ru")
            self._write_map_output(run_dir, wrapped=True)

            validation = validate_map_outputs(run_dir)

            self.assertEqual(validation["valid_outputs"], ["map/agent_outputs/chunk_001.json"])
            self.assertEqual(validation["invalid_outputs"], [])
            self.assertTrue(validation["repair_attempts"][0]["applied"])

    def test_validate_map_outputs_reports_invalid_schema(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_single_chunk_prep(Path(temp_dir))
            prepare_map_assignments(run_dir, output_language="ru")
            output_path = run_dir / "map" / "agent_outputs" / "chunk_001.json"
            write_json(output_path, {"chunk_id": "chunk_001", "facts": []})

            validation = validate_map_outputs(run_dir)

            self.assertEqual(validation["valid_outputs"], [])
            self.assertEqual(validation["invalid_outputs"][0]["output_file"], "map/agent_outputs/chunk_001.json")

    def test_assemble_map_artifacts_writes_canonical_facts(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_single_chunk_prep(Path(temp_dir))
            prepare_map_assignments(run_dir, output_language="ru")
            self._write_map_output(run_dir)
            validate_map_outputs(run_dir)

            map_manifest = assemble_map_artifacts(run_dir)
            mapped_facts = read_jsonl(run_dir / "map" / "mapped_facts.jsonl")
            chunk_summaries = read_jsonl(run_dir / "map" / "chunk_summaries.jsonl")

            self.assertEqual(map_manifest["mapped_fact_count"], 1)
            self.assertEqual(mapped_facts[0]["fact_id"], "fact_chunk_001_001")
            self.assertEqual(mapped_facts[0]["text"], "Evidence should be stored with timestamps.")
            self.assertEqual(chunk_summaries[0]["chunk_id"], "chunk_001")

    def _write_single_chunk_prep(self, temp_dir: Path) -> Path:
        run_dir = temp_dir / "run"
        write_prep_artifacts(
            FIXTURES_DIR / "agentic_tiny_transcript.txt",
            run_dir,
            target_tokens=10000,
            overlap_tokens=0,
            language="ru",
        )
        return run_dir

    def _write_map_output(self, run_dir: Path, *, wrapped: bool = False) -> None:
        output = {
            "chunk_id": "chunk_001",
            "time_range": {"start_ms": 0, "end_ms": 405000},
            "chunk_summary": "The lecture explains file-backed long-report generation.",
            "claims": [{"text": "Reports need evidence.", "timestamp": "00:02:10", "importance": 4}],
            "examples": [{"text": "A map stage extracts facts.", "timestamp": "00:02:10"}],
            "quotes": [],
            "entities": ["Map of Content"],
            "open_questions": [],
            "facts": [
                {
                    "local_fact_id": "fact_001",
                    "text": "Evidence should be stored with timestamps.",
                    "fact_type": "claim",
                    "timestamp": "00:02:10",
                    "importance": 4,
                    "chunk_id": "chunk_001",
                }
            ],
        }
        output_path = run_dir / "map" / "agent_outputs" / "chunk_001.json"
        output_path.parent.mkdir(parents=True, exist_ok=True)
        payload = json.dumps(output, ensure_ascii=False)
        if wrapped:
            payload = f"assistant note\n{payload}\nend note"
        output_path.write_text(payload + "\n", encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
