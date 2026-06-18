import unittest

from research.youtube_pipeline.moc import (
    TranscriptSegment,
    approximate_token_count,
    build_temporal_projection,
    chunk_segments_by_approx_tokens,
    compute_moc_budget,
    fallback_moc_plan,
    format_segments_for_prompt,
    parse_timestamp_ms,
    parse_timestamped_transcript,
    word_count,
)
from research.youtube_pipeline.strategies import StrategyOptions


class MocTranscriptTests(unittest.TestCase):
    def test_compute_moc_budget_uses_tucker_scale_defaults(self):
        budget = compute_moc_budget(
            transcript_words=41384,
            options=StrategyOptions(),
        )

        self.assertEqual(budget.report_min_words, 7000)
        self.assertEqual(budget.report_max_words, 10000)
        self.assertEqual(budget.target_report_words, 8500)
        self.assertEqual(budget.expected_node_min, 8)
        self.assertEqual(budget.expected_node_max, 12)

    def test_compute_moc_budget_applies_independent_min_override(self):
        budget = compute_moc_budget(
            transcript_words=41384,
            options=StrategyOptions(min_report_words=8001),
        )

        self.assertEqual(budget.report_min_words, 8001)
        self.assertEqual(budget.report_max_words, 10000)
        self.assertEqual(budget.target_report_words, 9000)

    def test_compute_moc_budget_applies_independent_max_override(self):
        budget = compute_moc_budget(
            transcript_words=41384,
            options=StrategyOptions(max_report_words=9001),
        )

        self.assertEqual(budget.report_min_words, 7000)
        self.assertEqual(budget.report_max_words, 9001)
        self.assertEqual(budget.target_report_words, 8000)

    def test_compute_moc_budget_clamps_overrides_to_max_report_words(self):
        budget = compute_moc_budget(
            transcript_words=41384,
            options=StrategyOptions(min_report_words=19000, max_report_words=25000),
        )

        self.assertEqual(budget.report_min_words, 19000)
        self.assertEqual(budget.report_max_words, 20000)
        self.assertEqual(budget.target_report_words, 19500)

    def test_compute_moc_budget_depth_scales_words_not_node_expectations(self):
        budget = compute_moc_budget(
            transcript_words=41384,
            options=StrategyOptions(target_depth="book"),
        )

        self.assertEqual(budget.report_min_words, 14000)
        self.assertEqual(budget.report_max_words, 20000)
        self.assertEqual(budget.target_report_words, 17000)
        self.assertEqual(budget.expected_node_min, 8)
        self.assertEqual(budget.expected_node_max, 12)

    def test_compute_moc_budget_rejects_final_min_greater_than_max(self):
        with self.assertRaisesRegex(
            ValueError,
            "min_report_words cannot be greater than max_report_words",
        ):
            compute_moc_budget(
                transcript_words=41384,
                options=StrategyOptions(min_report_words=11000, max_report_words=10000),
            )

    def test_build_temporal_projection_groups_timestamped_segments(self):
        segments = [
            TranscriptSegment(
                f"seg_{index + 1:06d}",
                index * 60000,
                (index + 1) * 60000,
                None,
                f"line {index} words",
            )
            for index in range(12)
        ]

        projection = build_temporal_projection(
            segments,
            source_word_count=240,
            window_ms=300000,
        )
        windows = projection["windows"]

        self.assertEqual(projection["projection_kind"], "temporal_skeleton")
        self.assertEqual(projection["source_segment_count"], 12)
        self.assertNotIn("segment_count", projection)
        self.assertEqual(len(windows), 3)
        self.assertEqual(windows[0]["start_ms"], 0)
        self.assertIn("sampled_timestamped_lines", windows[0])
        self.assertNotIn("sample_lines", windows[0])
        self.assertIn("line 0", windows[0]["first_words"])
        self.assertIn("line 4", windows[0]["last_words"])

    def test_build_temporal_projection_preserves_80_word_edges(self):
        text = " ".join(f"word{index}" for index in range(100))
        segments = [TranscriptSegment("seg_000001", 0, 1000, None, text)]

        projection = build_temporal_projection(
            segments,
            source_word_count=100,
            window_ms=300000,
        )
        window = projection["windows"][0]

        self.assertEqual(len(window["first_words"].split()), 80)
        self.assertEqual(len(window["last_words"].split()), 80)
        self.assertEqual(window["first_words"].split()[0], "word0")
        self.assertEqual(window["first_words"].split()[-1], "word79")
        self.assertEqual(window["last_words"].split()[0], "word20")
        self.assertEqual(window["last_words"].split()[-1], "word99")

    def test_fallback_moc_plan_builds_contiguous_budgeted_nodes(self):
        segments = [
            TranscriptSegment(
                f"seg_{index + 1:06d}",
                index * 1000,
                (index + 1) * 1000,
                None,
                f"segment {index}",
            )
            for index in range(10)
        ]
        budget = compute_moc_budget(
            transcript_words=1200,
            options=StrategyOptions(chapter_target_words=500),
        )

        plan = fallback_moc_plan("video1", segments, budget)
        nodes = plan["nodes"]

        self.assertEqual(plan["video_id"], "video1")
        self.assertEqual(nodes[0]["time_span"]["start_ms"], 0)
        self.assertEqual(nodes[-1]["time_span"]["end_ms"], 10000)
        self.assertEqual(
            sum(node["target_word_count"] for node in nodes),
            budget.target_report_words,
        )
        for previous, current in zip(nodes, nodes[1:]):
            self.assertEqual(
                previous["time_span"]["end_ms"],
                current["time_span"]["start_ms"],
            )

    def test_fallback_moc_plan_sizes_nodes_by_900_target_words(self):
        segments = [
            TranscriptSegment(
                f"seg_{index + 1:06d}",
                index * 1000,
                (index + 1) * 1000,
                None,
                f"segment {index}",
            )
            for index in range(10)
        ]
        budget = compute_moc_budget(
            transcript_words=1200,
            options=StrategyOptions(min_report_words=1400, max_report_words=1400),
        )

        plan = fallback_moc_plan("video1", segments, budget)

        self.assertEqual(len(plan["nodes"]), 2)

    def test_parse_timestamped_transcript_accepts_supported_formats(self):
        transcript = "\n".join(
            [
                "[00:01:02] first line",
                "[01:02:03] second line",
                "02:03 third line",
                "01:02:03 fourth line",
                "untimed line",
            ]
        )

        segments, warnings = parse_timestamped_transcript(transcript)

        self.assertEqual(
            [segment.start_ms for segment in segments],
            [62000, 3723000, 123000, 3723000, None],
        )
        self.assertEqual(segments[0].segment_id, "seg_000001")
        self.assertEqual(segments[-1].text, "untimed line")
        self.assertEqual(warnings, ["missing_timestamps"])

    def test_chunk_segments_by_approx_tokens_preserves_overlap_and_time_span(self):
        segments = [
            TranscriptSegment("seg_000001", 0, 1000, None, "one two"),
            TranscriptSegment("seg_000002", 1000, 2000, None, "three four"),
            TranscriptSegment("seg_000003", 2000, 3000, None, "five six"),
            TranscriptSegment("seg_000004", 3000, 4000, None, "seven eight"),
        ]

        chunks = chunk_segments_by_approx_tokens(
            segments,
            max_tokens=26,
            overlap_tokens=13,
        )

        self.assertEqual(
            [[segment.segment_id for segment in chunk.segments] for chunk in chunks],
            [
                ["seg_000001", "seg_000002"],
                ["seg_000002", "seg_000003"],
                ["seg_000003", "seg_000004"],
            ],
        )
        self.assertEqual(chunks[0].start_ms, 0)
        self.assertEqual(chunks[-1].end_ms, 4000)

    def test_chunk_segments_by_approx_tokens_uses_one_based_chunk_indexes(self):
        segments = [
            TranscriptSegment("seg_000001", 0, 1000, None, "one two"),
            TranscriptSegment("seg_000002", 1000, 2000, None, "three four"),
            TranscriptSegment("seg_000003", 2000, 3000, None, "five six"),
        ]

        chunks = chunk_segments_by_approx_tokens(
            segments,
            max_tokens=2,
            overlap_tokens=0,
        )

        self.assertEqual([chunk.chunk_index for chunk in chunks], [1, 2, 3])

    def test_chunk_segments_by_approx_tokens_budgets_emitted_text(self):
        segments = [
            TranscriptSegment("seg_000001", 0, 1000, None, "alpha"),
            TranscriptSegment("seg_000002", 1000, 2000, None, "bravo"),
            TranscriptSegment("seg_000003", 2000, 3000, None, "charlie"),
        ]

        chunks = chunk_segments_by_approx_tokens(
            segments,
            max_tokens=10,
            overlap_tokens=0,
        )

        for chunk in chunks:
            chunk_tokens = approximate_token_count(chunk.text)
            if len(chunk.segments) == 1:
                segment_tokens = approximate_token_count(
                    format_segments_for_prompt(chunk.segments)
                )
                self.assertTrue(chunk_tokens <= 10 or segment_tokens > 10)
            else:
                self.assertLessEqual(chunk_tokens, 10)

    def test_long_ascii_tokens_are_counted_by_length_and_chunked_alone(self):
        long_text = "a" * 80
        segments = [
            TranscriptSegment("seg_000001", 0, 1000, None, long_text),
            TranscriptSegment("seg_000002", 1000, 2000, None, "tail"),
        ]

        chunks = chunk_segments_by_approx_tokens(
            segments,
            max_tokens=25,
            overlap_tokens=0,
        )

        self.assertGreaterEqual(approximate_token_count(long_text), 20)
        self.assertEqual(
            [[segment.segment_id for segment in chunk.segments] for chunk in chunks],
            [["seg_000001"], ["seg_000002"]],
        )

    def test_parse_timestamp_ms_accepts_fractional_and_range_timestamps(self):
        cases = [
            ("[00:01:02.500] text", 62500, None, "text"),
            ("00:01:02,500 text", 62500, None, "text"),
            ("00:01:02.500 --> 00:01:04.000 text", 62500, 64000, "text"),
        ]

        for line, expected_start_ms, expected_end_ms, expected_text in cases:
            with self.subTest(line=line):
                start_ms, end_ms, text = parse_timestamp_ms(line)
                self.assertEqual(start_ms, expected_start_ms)
                self.assertEqual(end_ms, expected_end_ms)
                self.assertEqual(text, expected_text)

    def test_parse_timestamped_transcript_prefers_explicit_range_end(self):
        transcript = "\n".join(
            [
                "00:00:01.000 --> 00:00:03.000 first",
                "00:00:05.000 second",
            ]
        )

        segments, warnings = parse_timestamped_transcript(transcript)

        self.assertEqual(warnings, [])
        self.assertEqual(segments[0].start_ms, 1000)
        self.assertEqual(segments[0].end_ms, 3000)
        self.assertEqual(segments[1].start_ms, 5000)
        self.assertEqual(segments[1].end_ms, 5000)

    def test_format_segments_for_prompt_keeps_timestamps(self):
        segment = TranscriptSegment("seg_000001", 62000, 62000, None, "hello world")

        self.assertEqual(format_segments_for_prompt([segment]), "[00:01:02] hello world")

    def test_approximate_token_count_is_not_plain_word_count(self):
        text = "hello, world! \u041f\u0440\u0438\u0432\u0435\u0442"

        self.assertGreater(approximate_token_count(text), word_count(text))

    def test_approximate_token_count_bounds_cyrillic_heavy_text(self):
        text = (
            "\u042d\u0442\u043e \u0440\u0435\u0430\u043b\u0438\u0441\u0442\u0438\u0447\u043d\u043e\u0435 "
            "\u043f\u0440\u0435\u0434\u043b\u043e\u0436\u0435\u043d\u0438\u0435 "
            "\u0434\u043b\u044f \u043f\u0440\u043e\u0432\u0435\u0440\u043a\u0438 "
            "\u043e\u0446\u0435\u043d\u043a\u0438 \u0442\u043e\u043a\u0435\u043d\u043e\u0432"
        )

        token_count = approximate_token_count(text)

        self.assertGreater(token_count, word_count(text))
        self.assertLessEqual(token_count, word_count(text) * 4)


if __name__ == "__main__":
    unittest.main()
