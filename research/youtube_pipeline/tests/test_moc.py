import unittest

from research.youtube_pipeline.moc import (
    TranscriptSegment,
    approximate_token_count,
    chunk_segments_by_approx_tokens,
    format_segments_for_prompt,
    parse_timestamp_ms,
    parse_timestamped_transcript,
    word_count,
)


class MocTranscriptTests(unittest.TestCase):
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
            max_tokens=4,
            overlap_tokens=2,
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
