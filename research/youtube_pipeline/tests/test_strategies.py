import unittest

from research.youtube_pipeline.llm_client import LlmResponse
from research.youtube_pipeline.strategies import (
    run_chunk_map_reduce,
    run_one_shot_full_json,
    run_antigravity_chunk_map_reduce,
)


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
                "antigravity_chunk_map_reduce",
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
                "# Final report\n\nLong markdown report",
            ]
        )

        outcome = run_chunk_map_reduce(
            client=client,
            transcript="one two three four five six",
            output_language="ru",
            max_tokens=1000,
            chunk_token_limit=3,
        )

        self.assertEqual(outcome.result.summary_text, "# Final report\n\nLong markdown report")
        self.assertEqual(outcome.request_count, 4)
        self.assertEqual(outcome.input_tokens, 40)
        self.assertEqual(outcome.output_tokens, 80)
        self.assertEqual(len(client.calls), 4)
        self.assertIn("Chunk 1 of 2", client.calls[0][0][1].content)
        self.assertIn("Chunk 2 of 2", client.calls[1][0][1].content)
        self.assertIn("Chunk one", client.calls[2][0][1].content)
        self.assertIn("Chunk two", client.calls[2][0][1].content)
        self.assertIn("write the final report", client.calls[3][0][1].content)
        self.assertIn("Merged result", client.calls[3][0][1].content)

    def test_antigravity_chunk_map_reduce_runs_each_chunk_then_separates_reduce_then_merges(self):
        client = SequenceClient(
            [
                '{"summary_text":"Chunk one","timeline":[],"claims":[],"evidence":[],"action_items":[],"open_questions":[]}',
                '{"summary_text":"Chunk two","timeline":[],"claims":[],"evidence":[],"action_items":[],"open_questions":[]}',
                "Chapter 1 summary narrative",
                "Chapter 2 summary narrative",
                '{"timeline":[{"start":"00:00:00","end":"00:05:00","title":"T1","summary":"S1"}]}',
                '{"claims":[{"text":"C1","importance":"high","evidence_refs":[]}],"evidence":[]}',
                '{"action_items":[],"open_questions":[]}',
                "# Final report\n\nLong markdown report from antigravity",
            ]
        )

        outcome = run_antigravity_chunk_map_reduce(
            client=client,
            transcript="one two three four five six",
            output_language="ru",
            max_tokens=1000,
            chunk_token_limit=3,
        )

        self.assertEqual(outcome.result.summary_text, "# Final report\n\nLong markdown report from antigravity")
        self.assertEqual(outcome.result.timeline[0].title, "T1")
        self.assertEqual(outcome.result.claims[0].text, "C1")
        self.assertEqual(outcome.request_count, 8)
        self.assertEqual(outcome.input_tokens, 80)
        self.assertEqual(outcome.output_tokens, 160)
        self.assertEqual(len(client.calls), 8)
        self.assertIn("Chunk 1 of 2", client.calls[0][0][1].content)
        self.assertIn("Chunk 2 of 2", client.calls[1][0][1].content)
        self.assertIn("detailed narrative report", client.calls[2][0][0].content)
        self.assertIn("detailed narrative report", client.calls[3][0][0].content)
        self.assertIn("chronologically sorted timeline", client.calls[4][0][0].content)
        self.assertIn("claims and evidence", client.calls[5][0][0].content)
        self.assertIn("action items and open questions", client.calls[6][0][0].content)
        self.assertIn("research report", client.calls[7][0][1].content)
        self.assertIn("Chapter 1 summary narrative", client.calls[7][0][1].content)
        self.assertIn("Chapter 2 summary narrative", client.calls[7][0][1].content)


if __name__ == "__main__":
    unittest.main()
