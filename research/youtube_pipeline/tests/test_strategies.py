import unittest

from research.youtube_pipeline.llm_client import LlmResponse
from research.youtube_pipeline.strategies import run_one_shot_full_json


class FakeClient:
    def __init__(self):
        self.calls = []

    def complete(self, messages, max_tokens):
        self.calls.append((messages, max_tokens))
        return LlmResponse(
            text='{"summary_text":"Summary text","timeline":[],"claims":[],"evidence":[],"action_items":[],"open_questions":[]}',
            input_tokens=10,
            output_tokens=20,
        )


class StrategyTests(unittest.TestCase):
    def test_one_shot_full_json_returns_result_and_usage(self):
        client = FakeClient()
        outcome = run_one_shot_full_json(
            client=client,
            transcript="Transcript",
            output_language="ru",
            max_tokens=1000,
        )

        self.assertEqual(outcome.result.summary_text, "Summary text")
        self.assertEqual(outcome.request_count, 1)
        self.assertEqual(outcome.input_tokens, 10)
        self.assertEqual(outcome.output_tokens, 20)
        self.assertTrue(outcome.json_valid)
        self.assertEqual(client.calls[0][1], 1000)


if __name__ == "__main__":
    unittest.main()
