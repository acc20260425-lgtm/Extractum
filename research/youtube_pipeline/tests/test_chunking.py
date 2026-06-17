import unittest

from research.youtube_pipeline.chunking import chunk_by_approx_tokens, parse_timestamp_seconds


class ChunkingTests(unittest.TestCase):
    def test_parse_timestamp_seconds_supports_hh_mm_ss(self):
        self.assertEqual(parse_timestamp_seconds("[01:02:03] Speaker text"), 3723)

    def test_parse_timestamp_seconds_returns_none_without_timestamp(self):
        self.assertIsNone(parse_timestamp_seconds("Speaker text without timestamp"))

    def test_chunk_by_approx_tokens_keeps_all_text(self):
        transcript = " ".join(f"word{i}" for i in range(25))
        chunks = chunk_by_approx_tokens(transcript, max_tokens=10)

        self.assertEqual(len(chunks), 3)
        self.assertEqual(" ".join(chunks).split(), transcript.split())


if __name__ == "__main__":
    unittest.main()
