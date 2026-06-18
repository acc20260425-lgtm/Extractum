import unittest

from research.youtube_pipeline.moc import (
    TranscriptSegment,
    approximate_token_count,
    chunk_segments_by_approx_tokens,
    format_segments_for_prompt,
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

    def test_format_segments_for_prompt_keeps_timestamps(self):
        segment = TranscriptSegment("seg_000001", 62000, 62000, None, "hello world")

        self.assertEqual(format_segments_for_prompt([segment]), "[00:01:02] hello world")

    def test_approximate_token_count_is_not_plain_word_count(self):
        text = "hello, world! \u041f\u0440\u0438\u0432\u0435\u0442"

        self.assertGreater(approximate_token_count(text), word_count(text))


if __name__ == "__main__":
    unittest.main()
