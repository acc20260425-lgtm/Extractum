import unittest

from research.youtube_pipeline.llm_client import LlmResponse
from research.youtube_pipeline.strategies import (
    StrategyOptions,
    run_adaptive_book_report,
    run_antigravity_chunk_map_reduce,
    run_chunk_map_reduce,
    run_moc_guided_map_reduce,
    run_one_shot_full_json,
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


def normalized_chunk(summary, score=3):
    return (
        '{"substance_score":'
        + str(score)
        + ',"summary_text":"'
        + summary
        + '","timeline":[],"claims":[],"evidence":[],"action_items":[],"open_questions":[]}'
    )


class StrategyTests(unittest.TestCase):
    def test_one_shot_full_json_returns_result_and_usage(self):
        client = FakeClient()
        outcome = run_one_shot_full_json(
            client=client,
            transcript="Transcript",
            options=StrategyOptions(output_language="ru", max_tokens=1000),
        )

        self.assertEqual(outcome.result.summary_text, "Summary text")
        self.assertEqual(outcome.request_count, 1)
        self.assertEqual(outcome.input_tokens, 10)
        self.assertEqual(outcome.output_tokens, 20)
        self.assertTrue(outcome.json_valid)
        self.assertEqual(outcome.extra_metrics, {})
        self.assertEqual(client.calls[0][1], 1000)

    def test_all_research_strategies_are_registered(self):
        from research.youtube_pipeline.strategies import STRATEGIES

        self.assertEqual(
            sorted(STRATEGIES),
            [
                "adaptive_book_report",
                "antigravity_chunk_map_reduce",
                "chunk_map_reduce",
                "moc_guided_map_reduce",
                "one_shot_full_json",
                "one_shot_markdown_plus_json",
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
            options=StrategyOptions(output_language="ru", max_tokens=1000, chunk_token_limit=3),
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
            options=StrategyOptions(output_language="ru", max_tokens=1000, chunk_token_limit=3),
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

    def test_adaptive_book_report_generates_chapters_expands_and_assembles(self):
        transcript = " ".join(f"word{i}" for i in range(1200))
        long_chapter = " ".join(f"chapter2_{i}" for i in range(700))
        expanded_chapter = " ".join(f"expanded1_{i}" for i in range(700))
        client = SequenceClient(
            [
                normalized_chunk("Chunk one dense notes", 3),
                normalized_chunk("Chunk two dense notes", 3),
                normalized_chunk("Chunk three dense notes", 3),
                normalized_chunk("Chunk four dense notes", 3),
                (
                    '{"report_thesis":"Main thesis","key_terms":["Term"],'
                    '"chapters":['
                    '{"chapter_index":1,"title":"First arc","one_liner":"Covers first half","assigned_chunk_indexes":[1,2]},'
                    '{"chapter_index":2,"title":"Second arc","one_liner":"Covers second half","assigned_chunk_indexes":[3,4]}'
                    ']}'
                ),
                "Too short",
                expanded_chapter,
                long_chapter,
                '{"timeline":[{"start":"00:00:00","end":"00:05:00","title":"T1","summary":"S1"}]}',
                '{"claims":[{"text":"C1","importance":"high","evidence_refs":[]}],"evidence":[]}',
                '{"action_items":[{"text":"A1","target_audience":"Audience","priority":"medium"}],"open_questions":[]}',
                "Executive overview text",
                "Final synthesis text",
            ]
        )

        outcome = run_adaptive_book_report(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                max_tokens=5000,
                chunk_token_limit=300,
                chapter_target_words=900,
            ),
        )

        self.assertIn("Generated via `adaptive_book_report`", outcome.result.summary_text)
        self.assertIn("First arc", outcome.result.summary_text)
        self.assertIn("Second arc", outcome.result.summary_text)
        self.assertIn("Executive overview text", outcome.result.summary_text)
        self.assertIn("Final synthesis text", outcome.result.summary_text)
        self.assertIn("expanded1_0", outcome.result.summary_text)
        self.assertEqual(outcome.result.timeline[0].title, "T1")
        self.assertEqual(outcome.result.claims[0].text, "C1")
        self.assertEqual(outcome.result.action_items[0].text, "A1")
        self.assertEqual(outcome.request_count, 13)
        self.assertTrue(outcome.json_valid)
        self.assertEqual(outcome.extra_metrics["strategy_variant"], "adaptive_book_report")
        self.assertEqual(outcome.extra_metrics["chapter_count"], 2)
        self.assertEqual(outcome.extra_metrics["expansion_call_count"], 1)
        self.assertEqual(outcome.extra_metrics["target_report_words"], 1400)
        self.assertFalse(outcome.extra_metrics["outline_fallback_used"])
        self.assertFalse(outcome.extra_metrics["chapter_expansion_shortfall"])
        self.assertTrue(outcome.extra_metrics["substance_score_calibration_warning"])
        chapter_generation_call = client.calls[5]
        self.assertEqual(chapter_generation_call[1], 2254)

    def test_adaptive_book_report_records_outline_fallback_and_expansion_shortfall(self):
        transcript = " ".join(f"word{i}" for i in range(1200))
        still_short = " ".join(f"short_{i}" for i in range(100))
        client = SequenceClient(
            [
                normalized_chunk("Chunk one dense notes", 1),
                normalized_chunk("Chunk two dense notes", 2),
                normalized_chunk("Chunk three dense notes", 3),
                normalized_chunk("Chunk four dense notes", 4),
                "{not valid json",
                "Tiny chapter",
                still_short,
                "Second chapter has enough words " * 140,
                '{"timeline":[]}',
                '{"claims":[],"evidence":[]}',
                '{"action_items":[],"open_questions":[]}',
                "Overview",
                "Conclusion",
            ]
        )

        outcome = run_adaptive_book_report(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                max_tokens=5000,
                chunk_token_limit=300,
                chapter_target_words=900,
            ),
        )

        self.assertTrue(outcome.extra_metrics["outline_fallback_used"])
        self.assertTrue(outcome.extra_metrics["chapter_expansion_shortfall"])
        self.assertFalse(outcome.extra_metrics["substance_score_calibration_warning"])

    def test_adaptive_book_report_rejects_empty_transcript(self):
        client = FakeClient()

        with self.assertRaisesRegex(ValueError, "transcript is empty"):
            run_adaptive_book_report(
                client=client,
                transcript="   ",
                options=StrategyOptions(),
            )

        self.assertEqual(client.calls, [])

    def test_adaptive_book_report_uses_one_shot_for_short_transcript(self):
        client = FakeClient()

        outcome = run_adaptive_book_report(
            client=client,
            transcript="short transcript",
            options=StrategyOptions(output_language="ru", max_tokens=1000),
        )

        self.assertEqual(outcome.result.summary_text, "Summary text")
        self.assertEqual(outcome.request_count, 1)
        self.assertEqual(outcome.extra_metrics["strategy_variant"], "adaptive_book_report_short_fallback")
        self.assertEqual(outcome.extra_metrics["transcript_words"], 2)

    def test_moc_guided_map_reduce_maps_aligns_generates_and_assembles(self):
        transcript = "\n".join(
            [
                "[00:00:00] Media power opening",
                "[00:00:10] Media serves state power",
                "[00:00:20] Family and technology closing",
                "[00:00:30] Family matters",
            ]
        )
        client = SequenceClient(
            [
                (
                    '{"video_id":"video1","report_thesis":"Thesis","global_key_terms":["media","family"],'
                    '"nodes":['
                    '{"node_id":"node_001","title":"Media","time_span":{"start_ms":0,"end_ms":20000},"importance":"high","target_word_count":20,"description_outline":"Media topic","essential_key_terms":["media"],"required_questions":[],"expected_fact_types":["claims"]},'
                    '{"node_id":"node_002","title":"Family","time_span":{"start_ms":20000,"end_ms":40000},"importance":"medium","target_word_count":20,"description_outline":"Family topic","essential_key_terms":["family"],"required_questions":[],"expected_fact_types":["claims"]}'
                    ']}'
                ),
                (
                    '{"chunk_index":1,"chunk_time_span":{"start_ms":0,"end_ms":40000},'
                    '"facts":['
                    '{"fact_id":"f1","kind":"claim","text":"Media serves state power","importance":"high","time_span":{"start_ms":10000,"end_ms":11000},"verbatim_quote":"Media serves state power","speaker":null,"entities":["media"],"topic_tags":["state"],"moc_node_hint":null},'
                    '{"fact_id":"f2","kind":"claim","text":"Family matters","importance":"medium","time_span":{"start_ms":30000,"end_ms":31000},"verbatim_quote":"Family matters","speaker":null,"entities":["family"],"topic_tags":["family"],"moc_node_hint":null}'
                    '],"action_items":[],"open_questions":[]}'
                ),
                "## Section 1: Media\n\nMedia section has enough words with [00:00:10] timestamp " * 3,
                "## Section 2: Family\n\nFamily section has enough words with [00:00:30] timestamp " * 3,
                "Executive overview",
                "Final conclusion",
            ]
        )

        outcome = run_moc_guided_map_reduce(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                video_id="video1",
                max_tokens=2000,
                chunk_token_limit=100,
                chunk_overlap_tokens=10,
                min_report_words=40,
                max_report_words=40,
                chapter_target_words=20,
            ),
        )

        self.assertIn("Generated via `moc_guided_map_reduce`", outcome.result.summary_text)
        self.assertIn("Media", outcome.result.summary_text)
        self.assertIn("Family", outcome.result.summary_text)
        self.assertEqual(outcome.request_count, 6)
        self.assertEqual(outcome.result.timeline[0].title, "Media")
        self.assertEqual(outcome.result.claims[0].text, "Media serves state power")
        self.assertIn("moc.json", outcome.extra_artifacts)
        self.assertIn("mapped_facts.jsonl", outcome.extra_artifacts)
        self.assertEqual(outcome.extra_metrics["moc_node_count"], 2)
        self.assertEqual(outcome.extra_metrics["deduplicated_fact_count"], 2)
        self.assertGreater(outcome.extra_metrics["estimated_transcript_tokens"], 0)
        self.assertEqual(outcome.extra_metrics["parallelism_enabled"], False)
        self.assertEqual(outcome.extra_metrics["parallelizable_map_call_count"], 1)
        self.assertEqual(outcome.extra_metrics["parallelizable_node_call_count"], 2)

    def test_moc_guided_map_reduce_records_invalid_json_when_moc_fallback_is_used(self):
        transcript = "\n".join(
            [
                "[00:00:00] Alpha topic",
                "[00:00:10] Beta topic",
            ]
        )
        client = SequenceClient(
            [
                "not json",
                '{"chunk_index":1,"facts":[],"action_items":[],"open_questions":[]}',
                "Fallback section text with enough words [00:00:00] " * 5,
                "Executive overview",
                "Final conclusion",
            ]
        )

        outcome = run_moc_guided_map_reduce(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                video_id="video1",
                max_tokens=2000,
                chunk_token_limit=100,
                chunk_overlap_tokens=10,
                min_report_words=20,
                max_report_words=20,
                chapter_target_words=20,
            ),
        )

        self.assertFalse(outcome.json_valid)
        self.assertTrue(outcome.extra_metrics["moc_fallback_used"])

    def test_moc_guided_map_reduce_marks_invalid_plan_shape_as_invalid_json(self):
        transcript = "\n".join(
            [
                "[00:00:00] Alpha topic",
                "[00:00:10] Beta topic",
            ]
        )
        client = SequenceClient(
            [
                '{"nodes":[]}',
                '{"chunk_index":1,"facts":[],"action_items":[],"open_questions":[]}',
                "Fallback section text with enough words [00:00:00] " * 5,
                "Executive overview",
                "Final conclusion",
            ]
        )

        outcome = run_moc_guided_map_reduce(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                video_id="video1",
                max_tokens=2000,
                chunk_token_limit=100,
                chunk_overlap_tokens=10,
                min_report_words=20,
                max_report_words=20,
                chapter_target_words=20,
            ),
        )

        self.assertTrue(outcome.extra_metrics["moc_fallback_used"])
        self.assertFalse(outcome.json_valid)

    def test_moc_guided_map_reduce_rejects_unusable_plan_nodes(self):
        transcript = "\n".join(
            [
                "[00:00:00] Alpha topic",
                "[00:00:10] Beta topic",
            ]
        )
        client = SequenceClient(
            [
                (
                    '{"video_id":"video1","report_thesis":"Bad plan",'
                    '"global_key_terms":[],"nodes":[{"node_id":"node_001"}]}'
                ),
                '{"chunk_index":1,"facts":[],"action_items":[],"open_questions":[]}',
                "Fallback section text with enough words [00:00:00] " * 5,
                "Executive overview",
                "Final conclusion",
            ]
        )

        outcome = run_moc_guided_map_reduce(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                video_id="video1",
                max_tokens=2000,
                chunk_token_limit=100,
                chunk_overlap_tokens=10,
                min_report_words=20,
                max_report_words=20,
                chapter_target_words=20,
            ),
        )

        self.assertTrue(outcome.extra_metrics["moc_fallback_used"])
        self.assertFalse(outcome.json_valid)

    def test_moc_guided_map_reduce_keeps_json_valid_when_map_retry_succeeds(self):
        transcript = "\n".join(
            [
                "[00:00:00] Media power opening",
                "[00:00:10] Media serves state power",
            ]
        )
        client = SequenceClient(
            [
                (
                    '{"video_id":"video1","report_thesis":"Thesis","global_key_terms":["media"],'
                    '"nodes":['
                    '{"node_id":"node_001","title":"Media","time_span":{"start_ms":0,"end_ms":20000},"importance":"high","target_word_count":20,"description_outline":"Media topic","essential_key_terms":["media"],"required_questions":[],"expected_fact_types":["claims"]}'
                    ']}'
                ),
                "not json",
                (
                    '{"chunk_index":1,"chunk_time_span":{"start_ms":0,"end_ms":20000},'
                    '"facts":[{"fact_id":"f1","kind":"claim","text":"Media serves state power","importance":"high","time_span":{"start_ms":10000,"end_ms":11000},"verbatim_quote":"Media serves state power","speaker":null,"entities":["media"],"topic_tags":["state"],"moc_node_hint":null}],'
                    '"action_items":[],"open_questions":[]}'
                ),
                "## Section 1: Media\n\nMedia section has enough words with [00:00:10] timestamp " * 3,
                "Executive overview",
                "Final conclusion",
            ]
        )

        outcome = run_moc_guided_map_reduce(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                video_id="video1",
                max_tokens=2000,
                chunk_token_limit=100,
                chunk_overlap_tokens=10,
                min_report_words=20,
                max_report_words=20,
                chapter_target_words=20,
            ),
        )

        self.assertTrue(outcome.json_valid)
        self.assertEqual(outcome.extra_metrics["map_json_warning_count"], 1)
        self.assertEqual(outcome.extra_artifacts["quality_checks.json"]["map_json_warning_count"], 1)


if __name__ == "__main__":
    unittest.main()
