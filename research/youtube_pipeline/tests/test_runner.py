import json
from pathlib import Path
import tempfile
import unittest

from research.youtube_pipeline.models import NormalizedResult
from research.youtube_pipeline.runner import write_run_artifacts
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


if __name__ == "__main__":
    unittest.main()
