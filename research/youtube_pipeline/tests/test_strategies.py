import unittest

from research.youtube_pipeline.llm_client import LlmResponse
from research.youtube_pipeline.strategies import run_chunk_map_reduce, run_one_shot_full_json


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


class SequenceClient:
    def __init__(self, responses):
        self.responses = list(responses)
        self.calls = []

    def complete(self, messages, max_tokens):
        self.calls.append((messages, max_tokens))
        text = self.responses.pop(0)
        return LlmResponse(text=text, input_tokens=10, output_tokens=20)


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

    def test_all_research_strategies_are_registered(self):
        from research.youtube_pipeline.strategies import STRATEGIES

        self.assertEqual(
            sorted(STRATEGIES),
            [
                "chunk_map_reduce",
                "one_shot_full_json",
                "one_shot_markdown_plus_json",
                "timeline_segment_reduce",
                "two_pass_summary_structure",
            ],
        )

    def test_chunk_map_reduce_runs_each_chunk_then_merges(self):
        client = SequenceClient(
            [
                '{"summary_text":"Chunk one","timeline":[],"claims":[],"evidence":[],"action_items":[],"open_questions":[]}',
                '{"summary_text":"Chunk two","timeline":[],"claims":[],"evidence":[],"action_items":[],"open_questions":[]}',
                '{"summary_text":"Merged result","timeline":[],"claims":[],"evidence":[],"action_items":[],"open_questions":[]}',
            ]
        )

        outcome = run_chunk_map_reduce(
            client=client,
            transcript="one two three four five six",
            output_language="ru",
            max_tokens=1000,
            chunk_token_limit=3,
        )

        self.assertEqual(outcome.result.summary_text, "Merged result")
        self.assertEqual(outcome.request_count, 3)
        self.assertEqual(outcome.input_tokens, 30)
        self.assertEqual(outcome.output_tokens, 60)
        self.assertEqual(len(client.calls), 3)
        self.assertIn("Chunk 1 of 2", client.calls[0][0][1].content)
        self.assertIn("Chunk 2 of 2", client.calls[1][0][1].content)
        self.assertIn("Chunk one", client.calls[2][0][1].content)
        self.assertIn("Chunk two", client.calls[2][0][1].content)


if __name__ == "__main__":
    unittest.main()
