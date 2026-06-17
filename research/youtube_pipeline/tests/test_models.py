import unittest

from research.youtube_pipeline.models import (
    ActionItem,
    Claim,
    Evidence,
    NormalizedResult,
    OpenQuestion,
    TimelineItem,
)


class NormalizedResultTests(unittest.TestCase):
    def test_to_dict_uses_research_output_contract(self):
        result = NormalizedResult(
            summary_text="Detailed summary",
            timeline=[TimelineItem(start="00:00:00", end="00:05:00", title="Intro", summary="Setup")],
            claims=[Claim(text="Main claim", importance="high", evidence_refs=["e1"])],
            evidence=[Evidence(text="Quoted support", timestamp="00:01:00", supports_claims=["c1"])],
            action_items=[ActionItem(text="Try the approach", target_audience="developers", priority="medium")],
            open_questions=[OpenQuestion(text="What remains unclear?", why_it_matters="It affects adoption")],
        )

        self.assertEqual(
            result.to_dict(),
            {
                "summary_text": "Detailed summary",
                "timeline": [
                    {"start": "00:00:00", "end": "00:05:00", "title": "Intro", "summary": "Setup"}
                ],
                "claims": [
                    {"text": "Main claim", "importance": "high", "evidence_refs": ["e1"]}
                ],
                "evidence": [
                    {"text": "Quoted support", "timestamp": "00:01:00", "supports_claims": ["c1"]}
                ],
                "action_items": [
                    {"text": "Try the approach", "target_audience": "developers", "priority": "medium"}
                ],
                "open_questions": [
                    {"text": "What remains unclear?", "why_it_matters": "It affects adoption"}
                ],
            },
        )


if __name__ == "__main__":
    unittest.main()
