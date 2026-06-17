import unittest

from research.youtube_pipeline.prompts import (
    build_chunk_analysis_messages,
    build_chunk_reduce_messages,
    build_one_shot_full_json_messages,
)


class PromptTests(unittest.TestCase):
    def test_one_shot_prompt_requests_all_research_fields(self):
        messages = build_one_shot_full_json_messages("Transcript text", output_language="ru")
        joined = "\n".join(message.content for message in messages)

        self.assertIn("summary_text", joined)
        self.assertIn("timeline", joined)
        self.assertIn("claims", joined)
        self.assertIn("evidence", joined)
        self.assertIn("action_items", joined)
        self.assertIn("open_questions", joined)
        self.assertIn("Transcript text", joined)
        self.assertIn("ru", joined)

    def test_chunk_analysis_prompt_requests_dense_chunk_notes(self):
        messages = build_chunk_analysis_messages(
            "Chunk text",
            chunk_index=1,
            total_chunks=3,
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("300-600 words", joined)
        self.assertIn("dense chunk notes", joined)
        self.assertIn("do not compress", joined)
        self.assertIn("Chunk 1 of 3", joined)

    def test_chunk_reduce_prompt_requests_long_form_report(self):
        messages = build_chunk_reduce_messages('{"chunks":[]}', output_language="ru")
        joined = "\n".join(message.content for message in messages)

        self.assertIn("1200-2500 words", joined)
        self.assertIn("long-form report", joined)
        self.assertIn("do not summarize the summaries", joined)
        self.assertIn("Detailed narrative", joined)


if __name__ == "__main__":
    unittest.main()
