import unittest

from research.youtube_pipeline.metrics import build_metrics
from research.youtube_pipeline.models import Claim, Evidence, NormalizedResult, TimelineItem


class MetricsTests(unittest.TestCase):
    def test_build_metrics_counts_result_fields_and_usage(self):
        result = NormalizedResult(
            summary_text="one two three four",
            timeline=[TimelineItem(title="A"), TimelineItem(title="B")],
            claims=[Claim(text="claim")],
            evidence=[Evidence(text="e1"), Evidence(text="e2")],
        )

        metrics = build_metrics(
            strategy="two_pass_summary_structure",
            video_id="video_long",
            result=result,
            request_count=2,
            input_tokens=100,
            output_tokens=50,
            latency_seconds=3.5,
            json_valid=True,
        )

        self.assertEqual(metrics["strategy"], "two_pass_summary_structure")
        self.assertEqual(metrics["video_id"], "video_long")
        self.assertEqual(metrics["summary_words"], 4)
        self.assertEqual(metrics["timeline_segments_count"], 2)
        self.assertEqual(metrics["claims_count"], 1)
        self.assertEqual(metrics["evidence_count"], 2)
        self.assertEqual(metrics["action_items_count"], 0)
        self.assertEqual(metrics["open_questions_count"], 0)
        self.assertEqual(metrics["request_count"], 2)
        self.assertTrue(metrics["json_valid"])


if __name__ == "__main__":
    unittest.main()
