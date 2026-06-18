import importlib.util
import json
import re
import tempfile
import unittest
from pathlib import Path

from research.youtube_pipeline.moc_agentic import (
    align_facts,
    assemble_report,
    assemble_map_artifacts,
    build_structured_analysis,
    build_stage_key,
    build_planner_context,
    canonical_fact_id,
    chunk_transcript_text,
    estimate_tokens,
    hash_file,
    hash_text,
    dedupe_facts,
    normalize_transcript_text,
    prepare_map_assignments,
    prepare_section_assignments,
    read_json,
    read_jsonl,
    stage_is_reusable,
    quality_check,
    validate_map_outputs,
    validate_moc,
    validate_generated_files,
    word_count,
    write_prep_artifacts,
    write_json,
    write_jsonl,
)
from research.youtube_pipeline.youtube_summary_workflow import (
    WORKFLOW_STATE_SCHEMA,
    compute_options_hash,
    find_latest_matching_run,
    normalize_workflow_options,
    read_run_index,
    rebuild_run_index,
    start_youtube_summary_run,
    update_workflow_state,
    write_run_index,
)


FIXTURES_DIR = Path(__file__).parent / "fixtures"
REPO_ROOT = Path(__file__).parents[3]


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

    def test_build_planner_context_writes_bounded_context_and_metadata(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_single_chunk_prep(Path(temp_dir))
            prepare_map_assignments(run_dir, output_language="ru")
            self._write_map_output(run_dir)
            validate_map_outputs(run_dir)
            assemble_map_artifacts(run_dir)

            metadata = build_planner_context(run_dir, max_tokens=3000, language="ru")

            self.assertTrue((run_dir / "planning" / "planner_context.md").exists())
            self.assertEqual(metadata["included_chunk_count"], 1)
            self.assertEqual(metadata["total_fact_count"], 1)
            self.assertLessEqual(metadata["estimated_tokens"], 3000)

    def test_validate_moc_accepts_complete_plan(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_single_chunk_prep(Path(temp_dir))
            self._write_moc_raw(run_dir, [["chunk_001"]])

            validation = validate_moc(run_dir, target_words=900, chapter_target_words=900)
            moc = read_json(run_dir / "planning" / "moc.json")

            self.assertTrue(validation["valid"])
            self.assertFalse(validation["fallback_used"])
            self.assertEqual(moc["nodes"][0]["chunk_ids"], ["chunk_001"])

    def test_validate_moc_falls_back_when_chunks_are_missing(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_multi_chunk_prep(Path(temp_dir))
            chunks = read_jsonl(run_dir / "prep" / "chunks.jsonl")
            self.assertGreater(len(chunks), 1)
            self._write_moc_raw(run_dir, [[chunks[0]["chunk_id"]]])

            validation = validate_moc(run_dir, target_words=1800, chapter_target_words=900)
            moc = read_json(run_dir / "planning" / "moc.json")
            covered = [chunk_id for node in moc["nodes"] for chunk_id in node["chunk_ids"]]

            self.assertFalse(validation["valid"])
            self.assertTrue(validation["fallback_used"])
            self.assertEqual(sorted(covered), sorted(chunk["chunk_id"] for chunk in chunks))

    def test_validate_moc_falls_back_on_duplicate_chunk_without_reason(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_single_chunk_prep(Path(temp_dir))
            self._write_moc_raw(run_dir, [["chunk_001"], ["chunk_001"]])

            validation = validate_moc(run_dir, target_words=900, chapter_target_words=900)

            self.assertFalse(validation["valid"])
            self.assertTrue(validation["fallback_used"])
            self.assertIn("appears in multiple nodes", validation["errors"][0])

    def test_validate_moc_falls_back_on_non_ascending_chunk_order(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_multi_chunk_prep(Path(temp_dir))
            chunks = read_jsonl(run_dir / "prep" / "chunks.jsonl")
            first_chunk_id = str(chunks[0]["chunk_id"])
            last_chunk_id = str(chunks[-1]["chunk_id"])
            self._write_moc_raw(run_dir, [[last_chunk_id], [first_chunk_id]])

            validation = validate_moc(run_dir, target_words=1800, chapter_target_words=900)

            self.assertFalse(validation["valid"])
            self.assertTrue(validation["fallback_used"])
            self.assertTrue(any("non-ascending" in error for error in validation["errors"]))

    def test_dedupe_facts_preserves_original_timestamps_and_chunks(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir, chunk_ids = self._write_two_chunk_mapped_facts(Path(temp_dir))

            deduplicated = dedupe_facts(run_dir)

            self.assertEqual(len(deduplicated), 1)
            self.assertEqual(deduplicated[0]["source_chunk_ids"], chunk_ids)
            self.assertEqual(deduplicated[0]["source_timestamps"], ["00:02:10", "00:04:35"])

    def test_align_facts_uses_moc_chunk_ids(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir, chunk_ids = self._write_two_chunk_mapped_facts(Path(temp_dir))
            dedupe_facts(run_dir)
            self._write_moc_raw(run_dir, [[chunk_ids[0]], [chunk_ids[1]]])
            validate_moc(run_dir, target_words=1800, chapter_target_words=900)

            alignment = align_facts(run_dir)

            self.assertEqual(alignment["aligned_fact_count"], 1)
            self.assertEqual(alignment["nodes"][0]["aligned_fact_ids"], ["fact_cluster_0001"])
            self.assertEqual(alignment["nodes"][1]["aligned_fact_ids"], ["fact_cluster_0001"])

    def test_prepare_section_assignments_writes_jsonl(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir, chunk_ids = self._write_two_chunk_mapped_facts(Path(temp_dir))
            dedupe_facts(run_dir)
            self._write_moc_raw(run_dir, [[chunk_ids[0]], [chunk_ids[1]]])
            validate_moc(run_dir, target_words=1800, chapter_target_words=900)
            align_facts(run_dir)

            assignments = prepare_section_assignments(run_dir)
            rows = read_jsonl(run_dir / "alignment" / "section_assignments.jsonl")

            self.assertEqual(assignments, rows)
            self.assertEqual(rows[0]["section_file"], "sections/001-node-1.md")
            self.assertEqual(rows[0]["aligned_fact_ids"], ["fact_cluster_0001"])
            self.assertEqual(rows[0]["overlap_fact_ids"], ["fact_cluster_0001"])

    def test_validate_generated_files_accepts_expected_file(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = Path(temp_dir) / "run"
            section_path = run_dir / "sections" / "001-node.md"
            section_path.parent.mkdir(parents=True, exist_ok=True)
            section_path.write_text("section", encoding="utf-8")

            result = validate_generated_files(
                run_dir,
                agent_id="section_writer_moc_001",
                expected_files=["sections/001-node.md"],
            )

            self.assertTrue(result["valid"])
            self.assertEqual(result["missing_files"], [])
            self.assertEqual(result["unexpected_files"], [])

    def test_validate_generated_files_reports_missing_and_wrong_sibling_output(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = Path(temp_dir) / "run"
            wrong_path = run_dir / "sections" / "002-wrong.md"
            wrong_path.parent.mkdir(parents=True, exist_ok=True)
            wrong_path.write_text("wrong", encoding="utf-8")

            result = validate_generated_files(
                run_dir,
                agent_id="section_writer_moc_001",
                expected_files=["sections/001-node.md"],
            )

            self.assertFalse(result["valid"])
            self.assertEqual(result["missing_files"], ["sections/001-node.md"])
            self.assertEqual(result["unexpected_files"], ["sections/002-wrong.md"])

    def test_stage_is_reusable_requires_matching_hash_and_outputs(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = Path(temp_dir) / "run"
            output_path = run_dir / "map" / "mapped_facts.jsonl"
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text("{}\n", encoding="utf-8")
            stage_key = build_stage_key("map_assembly", {"validated_outputs": "abc"})
            manifest_path = run_dir / "map" / "assembly_manifest.json"
            write_json(manifest_path, {"stage_key": stage_key, "output_files": ["map/mapped_facts.jsonl"]})

            self.assertTrue(stage_is_reusable(run_dir, manifest_path, stage_key))
            self.assertFalse(
                stage_is_reusable(
                    run_dir,
                    manifest_path,
                    build_stage_key("map_assembly", {"validated_outputs": "changed"}),
                )
            )

    def test_quality_check_writes_coverage(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_aligned_section_workspace(Path(temp_dir))

            coverage = quality_check(run_dir)

            self.assertTrue(coverage["valid"])
            self.assertEqual(coverage["missing_files"], [])
            self.assertTrue(coverage["section_order_valid"])
            self.assertFalse(coverage["source_note_present"])
            self.assertGreater(coverage["total_section_words"], 0)
            self.assertTrue((run_dir / "review" / "coverage.md").exists())

    def test_build_structured_analysis_uses_fact_clusters(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_aligned_section_workspace(Path(temp_dir))
            quality_check(run_dir)

            manifest = build_structured_analysis(run_dir)
            structured = (run_dir / "review" / "structured_analysis.md").read_text(encoding="utf-8")

            self.assertEqual(manifest["fact_count"], 1)
            self.assertIn("Evidence should be stored with timestamps.", structured)
            self.assertIn("00:02:10, 00:04:35", structured)

    def test_assemble_report_writes_final_report_and_metrics(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = self._write_aligned_section_workspace(Path(temp_dir))
            quality_check(run_dir)
            build_structured_analysis(run_dir)

            result = assemble_report(run_dir)
            report = (run_dir / "final" / "report.md").read_text(encoding="utf-8")
            metrics = read_json(run_dir / "final" / "metrics.json")

            self.assertEqual(result["report_file"], "final/report.md")
            self.assertIn("summary and analysis of a YouTube video transcript", report)
            self.assertIn("## Table of Contents", report)
            self.assertIn("## Structured Analysis", report)
            self.assertEqual(metrics["strategy"], "moc_agentic_writer")

            coverage_after_assembly = quality_check(run_dir)
            self.assertTrue(coverage_after_assembly["source_note_present"])

    def test_agentic_tool_only_smoke_assembles_final_report_from_fixture(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = Path(temp_dir) / "run"
            write_prep_artifacts(
                FIXTURES_DIR / "agentic_tiny_transcript.txt",
                run_dir,
                target_tokens=10000,
                overlap_tokens=0,
                language="ru",
            )
            prepare_map_assignments(run_dir, output_language="ru")
            self._write_map_output(run_dir)
            validate_map_outputs(run_dir)
            assemble_map_artifacts(run_dir)
            build_planner_context(run_dir, max_tokens=3000, language="ru")
            self._write_moc_raw(run_dir, [["chunk_001"]])
            validate_moc(run_dir, target_words=900, chapter_target_words=900)
            dedupe_facts(run_dir)
            align_facts(run_dir)
            prepare_section_assignments(run_dir)
            self._write_smoke_sections(run_dir)
            quality_check(run_dir)
            build_structured_analysis(run_dir)

            result = assemble_report(run_dir)

            self.assertEqual(result["report_file"], "final/report.md")
            self.assertTrue((run_dir / "final" / "report.md").exists())
            self.assertTrue((run_dir / "final" / "metrics.json").exists())

    def test_youtube_skill_files_exist_and_reference_existing_tools(self):
        skill_files = [
            REPO_ROOT / ".agents" / "skills" / "youtube-long-report" / "SKILL.md",
            REPO_ROOT / ".agents" / "skills" / "youtube-map-extract" / "SKILL.md",
            REPO_ROOT / ".agents" / "skills" / "youtube-moc-planning" / "SKILL.md",
            REPO_ROOT / ".agents" / "skills" / "youtube-section-reduce" / "SKILL.md",
            REPO_ROOT / ".agents" / "skills" / "youtube-report-qa" / "SKILL.md",
        ]
        forbidden_api_markers = ["openai.", "anthropic.", "requests.post", "chat.completions"]

        for skill_file in skill_files:
            text = skill_file.read_text(encoding="utf-8")
            self.assertTrue(text.startswith("---\nname: youtube-"))
            self.assertIn("## Output Contract", text)
            self.assertIn("Direct LLM API calls are forbidden", text)
            for marker in forbidden_api_markers:
                self.assertNotIn(marker, text.lower())
            for tool_name in re.findall(r"research\.youtube_pipeline\.tools\.([a-z_]+)", text):
                self.assertIsNotNone(importlib.util.find_spec(f"research.youtube_pipeline.tools.{tool_name}"))

    def test_youtube_skill_examples_are_valid_json(self):
        example_files = [
            REPO_ROOT / ".agents" / "skills" / "youtube-long-report" / "examples" / "map_assignment_sample.json",
            REPO_ROOT / ".agents" / "skills" / "youtube-long-report" / "examples" / "map_output_sample.json",
            REPO_ROOT / ".agents" / "skills" / "youtube-map-extract" / "examples" / "map_assignment_sample.json",
            REPO_ROOT / ".agents" / "skills" / "youtube-map-extract" / "examples" / "map_output_sample.json",
            REPO_ROOT / ".agents" / "skills" / "youtube-moc-planning" / "examples" / "moc_sample.json",
            REPO_ROOT / ".agents" / "skills" / "youtube-section-reduce" / "examples" / "section_assignment_sample.json",
            REPO_ROOT / ".agents" / "skills" / "youtube-section-reduce" / "examples" / "alignment_sample.json",
        ]

        for example_file in example_files:
            payload = json.loads(example_file.read_text(encoding="utf-8"))
            self.assertIsInstance(payload, dict)

    def test_youtube_summary_cli_modules_are_importable(self):
        module_names = [
            "research.youtube_pipeline.tools.start_youtube_summary",
            "research.youtube_pipeline.tools.update_youtube_summary_state",
        ]

        for module_name in module_names:
            self.assertIsNotNone(importlib.util.find_spec(module_name))

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

    def _write_multi_chunk_prep(self, temp_dir: Path) -> Path:
        run_dir = temp_dir / "run"
        write_prep_artifacts(
            FIXTURES_DIR / "agentic_tiny_transcript.txt",
            run_dir,
            target_tokens=120,
            overlap_tokens=0,
            language="ru",
        )
        return run_dir

    def _write_map_output(
        self,
        run_dir: Path,
        *,
        chunk_id: str = "chunk_001",
        output_file: str = "map/agent_outputs/chunk_001.json",
        fact_text: str = "Evidence should be stored with timestamps.",
        fact_timestamp: str = "00:02:10",
        wrapped: bool = False,
    ) -> None:
        output = {
            "chunk_id": chunk_id,
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
                    "text": fact_text,
                    "fact_type": "claim",
                    "timestamp": fact_timestamp,
                    "importance": 4,
                    "chunk_id": chunk_id,
                }
            ],
        }
        output_path = run_dir / output_file
        output_path.parent.mkdir(parents=True, exist_ok=True)
        payload = json.dumps(output, ensure_ascii=False)
        if wrapped:
            payload = f"assistant note\n{payload}\nend note"
        output_path.write_text(payload + "\n", encoding="utf-8")

    def _write_two_chunk_mapped_facts(self, temp_dir: Path) -> tuple[Path, list[str]]:
        run_dir = temp_dir / "run"
        chunks = [
            {
                "chunk_id": "chunk_001",
                "chunk_index": 1,
                "start_timestamp": "00:00:00",
                "end_timestamp": "00:03:00",
                "text": "[00:00:00] first chunk",
                "word_count": 3,
                "estimated_tokens": 10,
                "source_hash": hash_text("two chunk fixture"),
            },
            {
                "chunk_id": "chunk_002",
                "chunk_index": 2,
                "start_timestamp": "00:03:00",
                "end_timestamp": "00:06:00",
                "text": "[00:03:00] second chunk",
                "word_count": 3,
                "estimated_tokens": 10,
                "source_hash": hash_text("two chunk fixture"),
            },
        ]
        write_jsonl(run_dir / "prep" / "chunks.jsonl", chunks)
        prepare_map_assignments(run_dir, output_language="ru")
        assignment_manifest = read_json(run_dir / "map" / "assignment_manifest.json")
        chunk_ids: list[str] = []
        timestamps = ["00:02:10", "00:04:35"]
        for index, assignment_path in enumerate(assignment_manifest["assignments"][:2]):
            assignment = read_json(run_dir / str(assignment_path))
            chunk_id = str(assignment["chunk_id"])
            chunk_ids.append(chunk_id)
            self._write_map_output(
                run_dir,
                chunk_id=chunk_id,
                output_file=str(assignment["output_file"]),
                fact_text="Evidence should be stored with timestamps.",
                fact_timestamp=timestamps[index],
            )
        validation = validate_map_outputs(run_dir)
        self.assertEqual(validation["invalid_outputs"], [])
        assemble_map_artifacts(run_dir)
        return run_dir, chunk_ids

    def _write_aligned_section_workspace(self, temp_dir: Path) -> Path:
        run_dir, chunk_ids = self._write_two_chunk_mapped_facts(temp_dir)
        dedupe_facts(run_dir)
        self._write_moc_raw(run_dir, [[chunk_ids[0]], [chunk_ids[1]]])
        validate_moc(run_dir, target_words=1800, chapter_target_words=900)
        align_facts(run_dir)
        prepare_section_assignments(run_dir)

        sections = {
            "sections/000-overview.md": "## Overview\n\nThis video summary orients the reader once.",
            "sections/001-node-1.md": "## Node 1\n\nThe first section uses the timestamped evidence in context.",
            "sections/002-node-2.md": "## Node 2\n\nThe second section uses the same fact in a different local argument.",
            "sections/999-synthesis.md": "## Synthesis\n\nThe workflow closes by preserving evidence and structure.",
        }
        for relative_path, content in sections.items():
            path = run_dir / relative_path
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content + "\n", encoding="utf-8")
        return run_dir

    def _write_smoke_sections(self, run_dir: Path) -> None:
        sections = {
            "sections/000-overview.md": "## Overview\n\nThis video summary frames the transcript once.",
            "sections/001-node-1.md": "## Node 1\n\nThe section explains the timestamped evidence workflow.",
            "sections/999-synthesis.md": "## Synthesis\n\nThe report closes without a whole-report rewrite.",
        }
        for relative_path, content in sections.items():
            path = run_dir / relative_path
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content + "\n", encoding="utf-8")

    def _write_moc_raw(self, run_dir: Path, chunk_groups: list[list[str]]) -> None:
        nodes = []
        for index, chunk_ids in enumerate(chunk_groups, start=1):
            nodes.append(
                {
                    "node_id": f"moc_{index:03d}",
                    "title": f"Node {index}",
                    "purpose": "Test node",
                    "target_words": 900,
                    "time_range": {"start_ms": 0, "end_ms": 600000},
                    "chunk_ids": chunk_ids,
                    "key_questions": ["What is the main point?"],
                    "required_fact_types": ["claim"],
                }
            )
        write_json(
            run_dir / "planning" / "moc.raw.json",
            {
                "report_title": "Test Report",
                "source_kind": "youtube_video_transcript",
                "report_thesis": "A test thesis.",
                "target_words": 900,
                "nodes": nodes,
            },
        )


if __name__ == "__main__":
    unittest.main()
