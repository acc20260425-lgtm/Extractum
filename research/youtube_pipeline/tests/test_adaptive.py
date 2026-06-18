import math
import unittest

from research.youtube_pipeline.adaptive import (
    assemble_adaptive_markdown_report,
    build_outline_chunk_descriptors,
    compute_budget_plan,
    compute_chapter_word_target,
    compute_substance_multiplier,
    extract_previous_chapter_bridge,
    normalize_substance_score,
    partition_weighted_chunks,
    response_token_budget,
)
from research.youtube_pipeline.strategies import StrategyOptions


class AdaptiveHelperTests(unittest.TestCase):
    def test_normalize_substance_score_defaults_and_clamps(self):
        self.assertEqual(normalize_substance_score(None), 3)
        self.assertEqual(normalize_substance_score("bad"), 3)
        self.assertEqual(normalize_substance_score(0), 1)
        self.assertEqual(normalize_substance_score(6), 5)
        self.assertEqual(normalize_substance_score("4"), 4)

    def test_compute_substance_multiplier_uses_narrow_range(self):
        self.assertAlmostEqual(compute_substance_multiplier([1, 1]), 0.7)
        self.assertAlmostEqual(compute_substance_multiplier([3, 3]), 1.0)
        self.assertAlmostEqual(compute_substance_multiplier([5, 5]), 1.3)
        self.assertAlmostEqual(compute_substance_multiplier([]), 1.0)

    def test_compute_budget_plan_records_range_and_midpoint_target(self):
        options = StrategyOptions(output_language="ru", target_depth="auto", chapter_target_words=900)
        plan = compute_budget_plan(transcript_words=41000, substance_scores=[3, 3], options=options)

        self.assertEqual(plan.report_min_words, 7000)
        self.assertEqual(plan.report_max_words, 10000)
        self.assertEqual(plan.target_report_words, 8500)
        self.assertEqual(plan.chapter_count, 9)
        self.assertEqual(plan.chapter_word_target, 944)

    def test_compute_budget_plan_applies_overrides_and_caps(self):
        options = StrategyOptions(
            output_language="ru",
            target_depth="book",
            min_report_words=12000,
            max_report_words=50000,
            chapter_target_words=900,
        )
        plan = compute_budget_plan(transcript_words=80000, substance_scores=[5], options=options)

        self.assertEqual(plan.report_min_words, 12000)
        self.assertEqual(plan.report_max_words, 20000)
        self.assertEqual(plan.target_report_words, 16000)
        self.assertLessEqual(plan.chapter_count, 20)

    def test_compute_budget_plan_rejects_invalid_overrides(self):
        options = StrategyOptions(min_report_words=9000, max_report_words=8000)

        with self.assertRaisesRegex(ValueError, "min_report_words cannot be greater than max_report_words"):
            compute_budget_plan(transcript_words=41000, substance_scores=[3], options=options)

    def test_compute_chapter_word_target_rounds_from_total_and_count(self):
        self.assertEqual(compute_chapter_word_target(8500, 9), 944)

    def test_partition_weighted_chunks_uses_contiguous_dp_groups(self):
        groups = partition_weighted_chunks([1, 1, 10, 1, 1], chapter_count=3)

        self.assertEqual(groups, [(0, 2), (2, 3), (3, 5)])

    def test_response_token_budget_is_language_aware(self):
        self.assertEqual(response_token_budget(900, "ru", max_tokens=8192), math.ceil(900 * 2.8 * 1.15))
        self.assertEqual(response_token_budget(900, "en", max_tokens=8192), math.ceil(900 * 1.8 * 1.15))
        self.assertEqual(response_token_budget(900, "ja", max_tokens=8192), math.ceil(900 * 3.0 * 1.15))
        self.assertEqual(response_token_budget(900, "ru", max_tokens=1000), 1000)

    def test_build_outline_chunk_descriptors_caps_summary_words(self):
        summary = " ".join(f"word{i}" for i in range(120))
        descriptors = build_outline_chunk_descriptors(
            [
                {
                    "chunk_index": 1,
                    "substance_score": 4,
                    "result": {
                        "summary_text": summary,
                        "timeline": [{"title": "Timeline A"}, {"title": "Timeline B"}],
                        "claims": [{"text": "Claim A"}, {"text": "Claim B"}],
                    },
                }
            ]
        )

        self.assertEqual(descriptors[0]["chunk_index"], 1)
        self.assertEqual(descriptors[0]["substance_score"], 4)
        self.assertEqual(len(descriptors[0]["summary_preview"].split()), 100)
        self.assertEqual(descriptors[0]["snippets"], ["Timeline A", "Timeline B", "Claim A"])

    def test_extract_previous_chapter_bridge_uses_tail_and_caps_words(self):
        chapter = "Intro paragraph.\n\n" + " ".join(f"tail{i}" for i in range(250))

        bridge = extract_previous_chapter_bridge(chapter)

        self.assertEqual(len(bridge.split()), 200)
        self.assertTrue(bridge.startswith("tail50"))

    def test_assemble_report_mentions_video_summary_once_without_section(self):
        report = assemble_adaptive_markdown_report(
            overview="Overview",
            chapters=["## Chapter 1\n\nChapter text"],
            chapter_titles=["Chapter title"],
            timeline_markdown="Timeline",
            claims_markdown="Claims",
            action_items_markdown="Actions",
            open_questions_markdown="Questions",
            conclusion="Conclusion",
        )

        self.assertNotIn("## Source Context", report)
        self.assertNotIn("- Source Context", report)
        self.assertIn("Source note: this is a summary and analysis of a YouTube video transcript.", report)
        self.assertLess(report.index("Source note:"), report.index("## Executive Overview"))


if __name__ == "__main__":
    unittest.main()
