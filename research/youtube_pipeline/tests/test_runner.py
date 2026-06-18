import json
from pathlib import Path
import tempfile
import unittest

from research.youtube_pipeline.models import NormalizedResult
from research.youtube_pipeline.runner import build_parser, build_strategy_options, write_run_artifacts
from research.youtube_pipeline.strategies import StrategyOutcome


class RunnerTests(unittest.TestCase):
    def test_write_run_artifacts_creates_expected_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[{"messages": []}],
                raw_responses=[{"text": "{}"}],
                extra_metrics={"chapter_count": 3, "target_report_words": 2700},
            )

            output_dir = write_run_artifacts(
                root=Path(tmp),
                strategy="one_shot_full_json",
                video_id="video_short",
                outcome=outcome,
            )

            self.assertTrue((output_dir / "result.json").exists())
            self.assertTrue((output_dir / "result.md").exists())
            self.assertTrue((output_dir / "metrics.json").exists())
            self.assertTrue((output_dir / "raw_requests.jsonl").exists())
            self.assertTrue((output_dir / "raw_responses.jsonl").exists())
            metrics = json.loads((output_dir / "metrics.json").read_text(encoding="utf-8"))
            self.assertEqual(metrics["summary_words"], 2)
            self.assertEqual(metrics["chapter_count"], 3)
            self.assertEqual(metrics["target_report_words"], 2700)

    def test_write_run_artifacts_rejects_unsafe_video_ids_before_writing(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[],
                raw_responses=[],
            )

            unsafe_video_ids = [
                "nested/video1",
                "..\\outside",
                str(Path(tmp) / "absolute-video"),
            ]
            for video_id in unsafe_video_ids:
                with self.subTest(video_id=video_id):
                    with self.assertRaisesRegex(ValueError, "video[- ]id"):
                        write_run_artifacts(
                            root=Path(tmp),
                            strategy="one_shot_full_json",
                            video_id=video_id,
                            outcome=outcome,
                        )

            self.assertFalse((Path(tmp) / "one_shot_full_json").exists())

    def test_build_strategy_options_reads_adaptive_cli_flags(self):
        parser = build_parser()
        args = parser.parse_args(
            [
                "--input",
                "input.txt",
                "--video-id",
                "video1",
                "--strategy",
                "chunk_map_reduce",
                "--output-language",
                "ru",
                "--max-tokens",
                "9000",
                "--chunk-token-limit",
                "2500",
                "--target-depth",
                "deep",
                "--min-report-words",
                "5000",
                "--max-report-words",
                "8000",
                "--chapter-target-words",
                "1000",
            ]
        )

        options = build_strategy_options(args)

        self.assertEqual(options.output_language, "ru")
        self.assertEqual(options.max_tokens, 9000)
        self.assertEqual(options.chunk_token_limit, 2500)
        self.assertEqual(options.target_depth, "deep")
        self.assertEqual(options.min_report_words, 5000)
        self.assertEqual(options.max_report_words, 8000)
        self.assertEqual(options.chapter_target_words, 1000)

    def test_build_strategy_options_reads_moc_cli_flags(self):
        parser = build_parser()
        args = parser.parse_args(
            [
                "--input",
                "input.txt",
                "--video-id",
                "video1",
                "--strategy",
                "chunk_map_reduce",
                "--chunk-overlap-tokens",
                "700",
                "--planner-context-token-limit",
                "120000",
                "--max-slice-tokens",
                "8000",
            ]
        )

        options = build_strategy_options(args)

        self.assertEqual(options.chunk_overlap_tokens, 700)
        self.assertEqual(options.planner_context_token_limit, 120000)
        self.assertEqual(options.max_slice_tokens, 8000)
        self.assertEqual(options.video_id, "video1")

    def test_write_run_artifacts_writes_extra_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[{"messages": []}],
                raw_responses=[{"text": "{}"}],
                extra_artifacts={
                    "moc.json": {"nodes": []},
                    "node_sections.jsonl": '{"node_id":"node_001"}\n',
                },
            )

            output_dir = write_run_artifacts(
                root=Path(tmp),
                strategy="chunk_map_reduce",
                video_id="video1",
                outcome=outcome,
            )

            moc_payload = json.loads((output_dir / "moc.json").read_text(encoding="utf-8"))
            node_sections_text = (output_dir / "node_sections.jsonl").read_text(encoding="utf-8")
            self.assertEqual(moc_payload, {"nodes": []})
            self.assertEqual(node_sections_text, '{"node_id":"node_001"}\n')

    def test_write_run_artifacts_rejects_reserved_extra_artifact_names(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[{"messages": []}],
                raw_responses=[{"text": "{}"}],
                extra_artifacts={"metrics.json": {}},
            )

            with self.assertRaisesRegex(ValueError, "extra artifact filename is reserved: metrics.json"):
                write_run_artifacts(
                    root=Path(tmp),
                    strategy="chunk_map_reduce",
                    video_id="video1",
                    outcome=outcome,
                )

    def test_write_run_artifacts_rejects_case_variant_reserved_extra_artifact_names(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[{"messages": []}],
                raw_responses=[{"text": "{}"}],
                extra_artifacts={"RESULT.JSON": {}},
            )

            with self.assertRaisesRegex(ValueError, "extra artifact filename is reserved: RESULT.JSON"):
                write_run_artifacts(
                    root=Path(tmp),
                    strategy="chunk_map_reduce",
                    video_id="video1",
                    outcome=outcome,
                )

    def test_write_run_artifacts_rejects_nested_extra_artifact_name(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[],
                raw_responses=[],
                extra_artifacts={"nested/file.json": {}},
            )

            with self.assertRaisesRegex(ValueError, "extra artifact filename"):
                write_run_artifacts(
                    root=Path(tmp),
                    strategy="moc_guided_map_reduce",
                    video_id="video1",
                    outcome=outcome,
                )

    def test_build_strategy_options_rejects_min_greater_than_max(self):
        parser = build_parser()
        args = parser.parse_args(
            [
                "--input",
                "input.txt",
                "--video-id",
                "video1",
                "--strategy",
                "chunk_map_reduce",
                "--min-report-words",
                "9000",
                "--max-report-words",
                "8000",
            ]
        )

        with self.assertRaisesRegex(ValueError, "min-report-words cannot be greater than max-report-words"):
            build_strategy_options(args)


if __name__ == "__main__":
    unittest.main()
