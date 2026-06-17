import unittest

from research.youtube_pipeline.prompts import build_one_shot_full_json_messages


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


if __name__ == "__main__":
    unittest.main()
