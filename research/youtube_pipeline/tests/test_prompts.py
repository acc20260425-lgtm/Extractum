import unittest

from research.youtube_pipeline.prompts import (
    build_adaptive_chapter_expansion_messages,
    build_adaptive_chapter_generation_messages,
    build_adaptive_chapter_outline_messages,
    build_adaptive_chunk_analysis_messages,
    build_adaptive_conclusion_messages,
    build_adaptive_overview_messages,
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

    def test_adaptive_chunk_analysis_prompt_includes_substance_rubric(self):
        messages = build_adaptive_chunk_analysis_messages(
            "Chunk text",
            chunk_index=1,
            total_chunks=2,
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("substance_score", joined)
        self.assertIn("greetings, ads, sponsor reads", joined)
        self.assertIn("Use 1 and 2 when appropriate", joined)
        self.assertIn("600-1000 words", joined)
        self.assertIn("Chunk 1 of 2", joined)

    def test_adaptive_outline_prompt_uses_compact_descriptors(self):
        messages = build_adaptive_chapter_outline_messages(
            chunk_descriptors_json='[{"chunk_index":1,"summary_preview":"Short"}]',
            chapter_groups_json='[{"chapter_index":1,"assigned_chunk_indexes":[1]}]',
            report_min_words=7000,
            report_max_words=10000,
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("report_thesis", joined)
        self.assertIn("key_terms", joined)
        self.assertIn("one_liner", joined)
        self.assertIn("Do not write chapter prose", joined)
        self.assertIn("7000-10000", joined)

    def test_adaptive_chapter_generation_prompt_includes_ledger_and_target(self):
        messages = build_adaptive_chapter_generation_messages(
            chapter_index=1,
            total_chapters=2,
            chapter_word_target=900,
            assigned_notes_json='[{"chunk_index":1}]',
            outline_json='{"report_thesis":"Thesis","key_terms":["Term"],"chapters":[]}',
            previous_bridge="Previous ending",
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("900 words", joined)
        self.assertIn("Thesis", joined)
        self.assertIn("Previous ending", joined)
        self.assertIn("Markdown prose only", joined)
        self.assertNotIn("Refer to speakers", joined)
        self.assertNotIn("source framing", joined)

    def test_adaptive_expansion_prompt_requests_source_grounded_expansion(self):
        messages = build_adaptive_chapter_expansion_messages(
            chapter_index=1,
            chapter_word_target=900,
            current_word_count=300,
            chapter_draft="Short draft",
            assigned_notes_json='[{"substance_score":5}]',
            outline_entry_json='{"title":"Chapter title"}',
            report_thesis="Thesis",
            key_terms=["Term"],
            previous_bridge="Bridge",
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("source-grounded detail", joined)
        self.assertIn("claims, examples, evidence, timeline moments", joined)
        self.assertIn("avoid generic filler", joined)
        self.assertIn("300 words", joined)

    def test_adaptive_overview_and_conclusion_prompts_do_not_rewrite_report(self):
        overview = build_adaptive_overview_messages(
            outline_json='{"report_thesis":"Thesis"}',
            structured_result_json='{"timeline":[]}',
            output_language="ru",
        )
        conclusion = build_adaptive_conclusion_messages(
            outline_json='{"report_thesis":"Thesis"}',
            structured_result_json='{"claims":[]}',
            output_language="ru",
        )
        joined = "\n".join(message.content for message in overview + conclusion)

        self.assertIn("Do not rewrite the chapters", joined)
        self.assertIn("mention once that this is a summary of a YouTube video", joined)
        self.assertIn("Do not repeat this framing", joined)
        self.assertIn("executive overview", joined)
        self.assertIn("final synthesis", joined)


if __name__ == "__main__":
    unittest.main()
