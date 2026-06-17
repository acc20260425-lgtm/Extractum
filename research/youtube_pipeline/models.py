from dataclasses import asdict, dataclass, field
from typing import Any


@dataclass
class TimelineItem:
    start: str = ""
    end: str = ""
    title: str = ""
    summary: str = ""


@dataclass
class Claim:
    text: str = ""
    importance: str = "medium"
    evidence_refs: list[str] = field(default_factory=list)


@dataclass
class Evidence:
    text: str = ""
    timestamp: str = ""
    supports_claims: list[str] = field(default_factory=list)


@dataclass
class ActionItem:
    text: str = ""
    target_audience: str = ""
    priority: str = "medium"


@dataclass
class OpenQuestion:
    text: str = ""
    why_it_matters: str = ""


@dataclass
class NormalizedResult:
    summary_text: str = ""
    timeline: list[TimelineItem] = field(default_factory=list)
    claims: list[Claim] = field(default_factory=list)
    evidence: list[Evidence] = field(default_factory=list)
    action_items: list[ActionItem] = field(default_factory=list)
    open_questions: list[OpenQuestion] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> "NormalizedResult":
        return cls(
            summary_text=str(payload.get("summary_text", "")),
            timeline=[TimelineItem(**item) for item in payload.get("timeline", []) if isinstance(item, dict)],
            claims=[Claim(**item) for item in payload.get("claims", []) if isinstance(item, dict)],
            evidence=[Evidence(**item) for item in payload.get("evidence", []) if isinstance(item, dict)],
            action_items=[
                ActionItem(**item) for item in payload.get("action_items", []) if isinstance(item, dict)
            ],
            open_questions=[
                OpenQuestion(**item) for item in payload.get("open_questions", []) if isinstance(item, dict)
            ],
        )
