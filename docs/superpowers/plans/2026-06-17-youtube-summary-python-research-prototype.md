# YouTube Summary Python Research Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a small Python research runner that compares YouTube summary LLM pipeline strategies on local transcript files.

**Architecture:** The prototype lives under `research/youtube_pipeline/` and uses only Python standard library modules. It separates normalized result models, chunking, metrics, provider calls, prompts, strategies, and CLI orchestration so each part can be tested without making live LLM calls.

**Tech Stack:** Python 3 standard library, `unittest`, OpenAI-compatible HTTP via `urllib.request`, JSONL run artifacts.

---

## File Structure

- Create: `research/youtube_pipeline/__init__.py`
  - Marks the research package.
- Create: `research/youtube_pipeline/models.py`
  - Dataclasses for normalized result, timeline items, claims, evidence, action items, open questions, and run metrics.
- Create: `research/youtube_pipeline/metrics.py`
  - Computes counts, word counts, latency totals, token totals, and writes metrics JSON.
- Create: `research/youtube_pipeline/chunking.py`
  - Splits transcripts by timestamp windows or approximate token windows.
- Create: `research/youtube_pipeline/llm_client.py`
  - Minimal OpenAI-compatible chat-completions client using env vars.
- Create: `research/youtube_pipeline/prompts.py`
  - Prompt builders for each strategy step.
- Create: `research/youtube_pipeline/strategies.py`
  - Implements `one_shot_full_json`, `one_shot_markdown_plus_json`, `two_pass_summary_structure`, `chunk_map_reduce`, and `timeline_segment_reduce`.
- Create: `research/youtube_pipeline/runner.py`
  - CLI entry point that reads transcript files, runs selected strategies, and writes artifacts.
- Create: `research/youtube_pipeline/tests/`
  - Unit tests using `unittest`.
- Create: `research/youtube_pipeline/inputs/.gitkeep`
  - Keeps the input directory in git without committing transcripts.
- Create: `research/youtube_pipeline/runs/.gitkeep`
  - Keeps the output directory in git without committing generated runs.
- Create: `research/youtube_pipeline/README.md`
  - Usage instructions and required environment variables.
- Modify: `.gitignore`
  - Ignore generated research run artifacts and local transcript inputs while keeping `.gitkeep` files.

---

### Task 1: Scaffold Package And Git Ignore Rules

**Files:**
- Create: `research/youtube_pipeline/__init__.py`
- Create: `research/youtube_pipeline/inputs/.gitkeep`
- Create: `research/youtube_pipeline/runs/.gitkeep`
- Create: `research/youtube_pipeline/tests/__init__.py`
- Modify: `.gitignore`

- [ ] **Step 1: Inspect current ignore rules**

Run:

```powershell
Get-Content .gitignore
```

Expected: read existing ignore patterns before editing.

- [ ] **Step 2: Add package marker files**

Create empty files:

```text
research/youtube_pipeline/__init__.py
research/youtube_pipeline/inputs/.gitkeep
research/youtube_pipeline/runs/.gitkeep
research/youtube_pipeline/tests/__init__.py
```

- [ ] **Step 3: Add ignore rules**

Append these rules to `.gitignore`:

```gitignore
# YouTube summary Python research local artifacts
research/youtube_pipeline/inputs/*
!research/youtube_pipeline/inputs/.gitkeep
research/youtube_pipeline/runs/*
!research/youtube_pipeline/runs/.gitkeep
```

- [ ] **Step 4: Verify status**

Run:

```powershell
git status --short
```

Expected: only the new scaffold files and `.gitignore` are changed, plus any pre-existing unrelated files.

- [ ] **Step 5: Commit**

Run:

```powershell
git add .gitignore research/youtube_pipeline/__init__.py research/youtube_pipeline/inputs/.gitkeep research/youtube_pipeline/runs/.gitkeep research/youtube_pipeline/tests/__init__.py
git commit -m "chore: scaffold youtube pipeline research package"
```

Expected: commit succeeds.

---

### Task 2: Normalized Models

**Files:**
- Create: `research/youtube_pipeline/models.py`
- Create: `research/youtube_pipeline/tests/test_models.py`

- [ ] **Step 1: Write failing model tests**

Create `research/youtube_pipeline/tests/test_models.py`:

```python
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
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_models -v
```

Expected: FAIL because `research.youtube_pipeline.models` does not exist.

- [ ] **Step 3: Implement models**

Create `research/youtube_pipeline/models.py`:

```python
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
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_models -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/models.py research/youtube_pipeline/tests/test_models.py
git commit -m "feat: add youtube research result models"
```

Expected: commit succeeds.

---

### Task 3: Metrics Computation

**Files:**
- Create: `research/youtube_pipeline/metrics.py`
- Create: `research/youtube_pipeline/tests/test_metrics.py`

- [ ] **Step 1: Write failing metrics tests**

Create `research/youtube_pipeline/tests/test_metrics.py`:

```python
import unittest

from research.youtube_pipeline.metrics import build_metrics
from research.youtube_pipeline.models import Claim, Evidence, NormalizedResult, TimelineItem


class MetricsTests(unittest.TestCase):
    def test_build_metrics_counts_result_fields_and_usage(self):
        result = NormalizedResult(
            summary_text="one two three four",
            timeline=[TimelineItem(title="A"), TimelineItem(title="B")],
            claims=[Claim(text="claim")],
            evidence=[Evidence(text="e1"), Evidence(text="e2")],
        )

        metrics = build_metrics(
            strategy="two_pass_summary_structure",
            video_id="video_long",
            result=result,
            request_count=2,
            input_tokens=100,
            output_tokens=50,
            latency_seconds=3.5,
            json_valid=True,
        )

        self.assertEqual(metrics["strategy"], "two_pass_summary_structure")
        self.assertEqual(metrics["video_id"], "video_long")
        self.assertEqual(metrics["summary_words"], 4)
        self.assertEqual(metrics["timeline_segments_count"], 2)
        self.assertEqual(metrics["claims_count"], 1)
        self.assertEqual(metrics["evidence_count"], 2)
        self.assertEqual(metrics["action_items_count"], 0)
        self.assertEqual(metrics["open_questions_count"], 0)
        self.assertEqual(metrics["request_count"], 2)
        self.assertTrue(metrics["json_valid"])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_metrics -v
```

Expected: FAIL because `build_metrics` does not exist.

- [ ] **Step 3: Implement metrics**

Create `research/youtube_pipeline/metrics.py`:

```python
from typing import Any

from research.youtube_pipeline.models import NormalizedResult


def count_words(text: str) -> int:
    return len([part for part in text.split() if part.strip()])


def build_metrics(
    *,
    strategy: str,
    video_id: str,
    result: NormalizedResult,
    request_count: int,
    input_tokens: int,
    output_tokens: int,
    latency_seconds: float,
    json_valid: bool,
    notes: str = "",
) -> dict[str, Any]:
    return {
        "strategy": strategy,
        "video_id": video_id,
        "request_count": request_count,
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "latency_seconds": latency_seconds,
        "summary_words": count_words(result.summary_text),
        "timeline_segments_count": len(result.timeline),
        "claims_count": len(result.claims),
        "evidence_count": len(result.evidence),
        "action_items_count": len(result.action_items),
        "open_questions_count": len(result.open_questions),
        "json_valid": json_valid,
        "notes": notes,
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_metrics -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/metrics.py research/youtube_pipeline/tests/test_metrics.py
git commit -m "feat: add youtube research metrics"
```

Expected: commit succeeds.

---

### Task 4: Transcript Chunking

**Files:**
- Create: `research/youtube_pipeline/chunking.py`
- Create: `research/youtube_pipeline/tests/test_chunking.py`

- [ ] **Step 1: Write failing chunking tests**

Create `research/youtube_pipeline/tests/test_chunking.py`:

```python
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
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_chunking -v
```

Expected: FAIL because `chunking.py` does not exist.

- [ ] **Step 3: Implement chunking helpers**

Create `research/youtube_pipeline/chunking.py`:

```python
import re


TIMESTAMP_RE = re.compile(r"^\[(?:(\d{1,2}):)?(\d{2}):(\d{2})\]")


def parse_timestamp_seconds(line: str) -> int | None:
    match = TIMESTAMP_RE.match(line.strip())
    if not match:
        return None
    hours = int(match.group(1) or 0)
    minutes = int(match.group(2))
    seconds = int(match.group(3))
    return hours * 3600 + minutes * 60 + seconds


def chunk_by_approx_tokens(transcript: str, max_tokens: int) -> list[str]:
    words = transcript.split()
    if max_tokens <= 0:
        raise ValueError("max_tokens must be positive")
    return [" ".join(words[index : index + max_tokens]) for index in range(0, len(words), max_tokens)]
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_chunking -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/chunking.py research/youtube_pipeline/tests/test_chunking.py
git commit -m "feat: add youtube transcript chunking helpers"
```

Expected: commit succeeds.

---

### Task 5: LLM Client With Fakeable Transport

**Files:**
- Create: `research/youtube_pipeline/llm_client.py`
- Create: `research/youtube_pipeline/tests/test_llm_client.py`

- [ ] **Step 1: Write failing LLM client tests**

Create `research/youtube_pipeline/tests/test_llm_client.py`:

```python
import json
import unittest

from research.youtube_pipeline.llm_client import ChatMessage, OpenAICompatibleClient


class FakeTransport:
    def __init__(self):
        self.calls = []

    def post_json(self, url, headers, payload):
        self.calls.append((url, headers, payload))
        return {
            "choices": [{"message": {"content": "{\"summary_text\":\"ok\"}"}}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5},
        }


class LlmClientTests(unittest.TestCase):
    def test_chat_completion_sends_openai_compatible_payload(self):
        transport = FakeTransport()
        client = OpenAICompatibleClient(
            base_url="https://example.test/v1",
            api_key="secret",
            model="test-model",
            transport=transport,
        )

        response = client.complete([ChatMessage(role="user", content="Hello")], max_tokens=100)

        self.assertEqual(response.text, "{\"summary_text\":\"ok\"}")
        self.assertEqual(response.input_tokens, 10)
        self.assertEqual(response.output_tokens, 5)
        sent_payload = transport.calls[0][2]
        self.assertEqual(sent_payload["model"], "test-model")
        self.assertEqual(sent_payload["max_tokens"], 100)
        self.assertEqual(sent_payload["messages"], [{"role": "user", "content": "Hello"}])
        self.assertEqual(json.loads(json.dumps(sent_payload))["model"], "test-model")


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_llm_client -v
```

Expected: FAIL because `llm_client.py` does not exist.

- [ ] **Step 3: Implement LLM client**

Create `research/youtube_pipeline/llm_client.py`:

```python
from dataclasses import dataclass
import json
from typing import Any, Protocol
from urllib import request


@dataclass
class ChatMessage:
    role: str
    content: str


@dataclass
class LlmResponse:
    text: str
    input_tokens: int
    output_tokens: int


class JsonTransport(Protocol):
    def post_json(self, url: str, headers: dict[str, str], payload: dict[str, Any]) -> dict[str, Any]:
        ...


class UrllibJsonTransport:
    def post_json(self, url: str, headers: dict[str, str], payload: dict[str, Any]) -> dict[str, Any]:
        body = json.dumps(payload).encode("utf-8")
        req = request.Request(url, data=body, headers=headers, method="POST")
        with request.urlopen(req, timeout=120) as response:
            return json.loads(response.read().decode("utf-8"))


class OpenAICompatibleClient:
    def __init__(
        self,
        *,
        base_url: str,
        api_key: str,
        model: str,
        transport: JsonTransport | None = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.model = model
        self.transport = transport or UrllibJsonTransport()

    def complete(self, messages: list[ChatMessage], max_tokens: int) -> LlmResponse:
        payload = {
            "model": self.model,
            "messages": [{"role": message.role, "content": message.content} for message in messages],
            "max_tokens": max_tokens,
            "temperature": 0.2,
        }
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }
        data = self.transport.post_json(f"{self.base_url}/chat/completions", headers, payload)
        text = data["choices"][0]["message"]["content"]
        usage = data.get("usage") or {}
        return LlmResponse(
            text=text,
            input_tokens=int(usage.get("prompt_tokens") or 0),
            output_tokens=int(usage.get("completion_tokens") or 0),
        )
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_llm_client -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/llm_client.py research/youtube_pipeline/tests/test_llm_client.py
git commit -m "feat: add openai compatible research llm client"
```

Expected: commit succeeds.

---

### Task 6: Prompt Builders

**Files:**
- Create: `research/youtube_pipeline/prompts.py`
- Create: `research/youtube_pipeline/tests/test_prompts.py`

- [ ] **Step 1: Write failing prompt tests**

Create `research/youtube_pipeline/tests/test_prompts.py`:

```python
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
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_prompts -v
```

Expected: FAIL because `prompts.py` does not exist.

- [ ] **Step 3: Implement initial prompt builder**

Create `research/youtube_pipeline/prompts.py`:

```python
from research.youtube_pipeline.llm_client import ChatMessage


RESULT_CONTRACT = """Return JSON with this shape:
{
  "summary_text": "detailed readable summary",
  "timeline": [{"start": "00:00:00", "end": "00:05:00", "title": "", "summary": ""}],
  "claims": [{"text": "", "importance": "high", "evidence_refs": []}],
  "evidence": [{"text": "", "timestamp": "00:00:00", "supports_claims": []}],
  "action_items": [{"text": "", "target_audience": "", "priority": "medium"}],
  "open_questions": [{"text": "", "why_it_matters": ""}]
}
"""


def build_one_shot_full_json_messages(transcript: str, *, output_language: str) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You analyze YouTube transcripts for research. Use only the transcript. "
                "Return one JSON object and no Markdown wrapper."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Create a detailed summary and fill timeline, claims, evidence, action_items, "
                "and open_questions. If action items are absent, return an empty action_items array.\n\n"
                f"{RESULT_CONTRACT}\n\nTranscript:\n{transcript}"
            ),
        ),
    ]
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_prompts -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/prompts.py research/youtube_pipeline/tests/test_prompts.py
git commit -m "feat: add youtube research prompt builders"
```

Expected: commit succeeds.

---

### Task 7: One-Shot Strategy

**Files:**
- Create: `research/youtube_pipeline/strategies.py`
- Create: `research/youtube_pipeline/tests/test_strategies.py`

- [ ] **Step 1: Write failing strategy test**

Create `research/youtube_pipeline/tests/test_strategies.py`:

```python
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
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: FAIL because `strategies.py` does not exist.

- [ ] **Step 3: Implement strategy outcome and one-shot strategy**

Create `research/youtube_pipeline/strategies.py`:

```python
from dataclasses import dataclass
import json
import time
from typing import Protocol

from research.youtube_pipeline.llm_client import ChatMessage, LlmResponse
from research.youtube_pipeline.models import NormalizedResult
from research.youtube_pipeline.prompts import build_one_shot_full_json_messages


class LlmClient(Protocol):
    def complete(self, messages: list[ChatMessage], max_tokens: int) -> LlmResponse:
        ...


@dataclass
class StrategyOutcome:
    result: NormalizedResult
    request_count: int
    input_tokens: int
    output_tokens: int
    latency_seconds: float
    json_valid: bool
    raw_requests: list[dict[str, object]]
    raw_responses: list[dict[str, object]]


def parse_result_json(text: str) -> tuple[NormalizedResult, bool]:
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        return NormalizedResult(summary_text=text), False
    return NormalizedResult.from_dict(payload), True


def run_one_shot_full_json(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    messages = build_one_shot_full_json_messages(transcript, output_language=output_language)
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=max_tokens)
    latency = time.perf_counter() - started
    result, json_valid = parse_result_json(response.text)
    return StrategyOutcome(
        result=result,
        request_count=1,
        input_tokens=response.input_tokens,
        output_tokens=response.output_tokens,
        latency_seconds=latency,
        json_valid=json_valid,
        raw_requests=[{"messages": [message.__dict__ for message in messages], "max_tokens": max_tokens}],
        raw_responses=[{"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}],
    )
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/strategies.py research/youtube_pipeline/tests/test_strategies.py
git commit -m "feat: add one shot youtube research strategy"
```

Expected: commit succeeds.

---

### Task 8: Remaining Strategy Skeletons

**Files:**
- Modify: `research/youtube_pipeline/prompts.py`
- Modify: `research/youtube_pipeline/strategies.py`
- Modify: `research/youtube_pipeline/tests/test_strategies.py`

- [ ] **Step 1: Add failing tests for strategy names**

Append to `StrategyTests` in `test_strategies.py`:

```python
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
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: FAIL because `STRATEGIES` does not exist.

- [ ] **Step 3: Implement strategy registry with working research wrappers**

Append to `strategies.py`:

```python
def run_one_shot_markdown_plus_json(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    return run_one_shot_full_json(
        client=client,
        transcript=transcript,
        output_language=output_language,
        max_tokens=max_tokens,
    )


def run_two_pass_summary_structure(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    first = run_one_shot_full_json(
        client=client,
        transcript=transcript,
        output_language=output_language,
        max_tokens=max_tokens,
    )
    second_input = f"Summary:\n{first.result.summary_text}\n\nTranscript:\n{transcript}"
    second = run_one_shot_full_json(
        client=client,
        transcript=second_input,
        output_language=output_language,
        max_tokens=max_tokens,
    )
    second.request_count = first.request_count + second.request_count
    second.input_tokens += first.input_tokens
    second.output_tokens += first.output_tokens
    second.latency_seconds += first.latency_seconds
    second.raw_requests = first.raw_requests + second.raw_requests
    second.raw_responses = first.raw_responses + second.raw_responses
    return second


def run_chunk_map_reduce(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    return run_two_pass_summary_structure(
        client=client,
        transcript=transcript,
        output_language=output_language,
        max_tokens=max_tokens,
    )


def run_timeline_segment_reduce(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    return run_two_pass_summary_structure(
        client=client,
        transcript=transcript,
        output_language=output_language,
        max_tokens=max_tokens,
    )


STRATEGIES = {
    "one_shot_full_json": run_one_shot_full_json,
    "one_shot_markdown_plus_json": run_one_shot_markdown_plus_json,
    "two_pass_summary_structure": run_two_pass_summary_structure,
    "chunk_map_reduce": run_chunk_map_reduce,
    "timeline_segment_reduce": run_timeline_segment_reduce,
}
```

This creates callable strategy entries. Later research iterations can make each
strategy prompt more distinct after the runner exists.

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/prompts.py research/youtube_pipeline/strategies.py research/youtube_pipeline/tests/test_strategies.py
git commit -m "feat: register youtube research strategies"
```

Expected: commit succeeds.

---

### Task 9: CLI Runner And Artifact Writing

**Files:**
- Create: `research/youtube_pipeline/runner.py`
- Create: `research/youtube_pipeline/tests/test_runner.py`

- [ ] **Step 1: Write failing runner artifact test**

Create `research/youtube_pipeline/tests/test_runner.py`:

```python
import json
from pathlib import Path
import tempfile
import unittest

from research.youtube_pipeline.models import NormalizedResult
from research.youtube_pipeline.runner import write_run_artifacts
from research.youtube_pipeline.strategies import StrategyOutcome


class RunnerTests(unittest.TestCase):
    def test_write_run_artifacts_creates_expected_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[{"messages": []}],
                raw_responses=[{"text": "{}"}],
            )

            output_dir = write_run_artifacts(
                root=Path(tmp),
                strategy="one_shot_full_json",
                video_id="video_short",
                outcome=outcome,
            )

            self.assertTrue((output_dir / "result.json").exists())
            self.assertTrue((output_dir / "result.md").exists())
            self.assertTrue((output_dir / "metrics.json").exists())
            self.assertTrue((output_dir / "raw_requests.jsonl").exists())
            self.assertTrue((output_dir / "raw_responses.jsonl").exists())
            metrics = json.loads((output_dir / "metrics.json").read_text(encoding="utf-8"))
            self.assertEqual(metrics["summary_words"], 2)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_runner -v
```

Expected: FAIL because `runner.py` does not exist.

- [ ] **Step 3: Implement artifact writing and CLI**

Create `research/youtube_pipeline/runner.py`:

```python
import argparse
import json
import os
from pathlib import Path
from typing import Any

from research.youtube_pipeline.llm_client import OpenAICompatibleClient
from research.youtube_pipeline.metrics import build_metrics
from research.youtube_pipeline.strategies import STRATEGIES, StrategyOutcome


def write_json(path: Path, payload: Any) -> None:
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")


def write_jsonl(path: Path, rows: list[dict[str, object]]) -> None:
    path.write_text(
        "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows),
        encoding="utf-8",
    )


def write_run_artifacts(
    *,
    root: Path,
    strategy: str,
    video_id: str,
    outcome: StrategyOutcome,
) -> Path:
    output_dir = root / strategy / video_id
    output_dir.mkdir(parents=True, exist_ok=True)
    result_payload = outcome.result.to_dict()
    write_json(output_dir / "result.json", result_payload)
    (output_dir / "result.md").write_text(outcome.result.summary_text, encoding="utf-8")
    write_json(
        output_dir / "metrics.json",
        build_metrics(
            strategy=strategy,
            video_id=video_id,
            result=outcome.result,
            request_count=outcome.request_count,
            input_tokens=outcome.input_tokens,
            output_tokens=outcome.output_tokens,
            latency_seconds=outcome.latency_seconds,
            json_valid=outcome.json_valid,
        ),
    )
    write_jsonl(output_dir / "raw_requests.jsonl", outcome.raw_requests)
    write_jsonl(output_dir / "raw_responses.jsonl", outcome.raw_responses)
    return output_dir


def build_client_from_env() -> OpenAICompatibleClient:
    return OpenAICompatibleClient(
        base_url=os.environ["YOUTUBE_RESEARCH_LLM_BASE_URL"],
        api_key=os.environ["YOUTUBE_RESEARCH_LLM_API_KEY"],
        model=os.environ["YOUTUBE_RESEARCH_LLM_MODEL"],
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Run YouTube summary pipeline research strategies.")
    parser.add_argument("--input", required=True, help="Path to transcript text file")
    parser.add_argument("--strategy", required=True, choices=sorted(STRATEGIES))
    parser.add_argument("--video-id", required=True)
    parser.add_argument("--output-root", default="research/youtube_pipeline/runs/manual")
    parser.add_argument("--output-language", default="ru")
    parser.add_argument("--max-tokens", type=int, default=8192)
    args = parser.parse_args()

    transcript = Path(args.input).read_text(encoding="utf-8")
    client = build_client_from_env()
    outcome = STRATEGIES[args.strategy](
        client=client,
        transcript=transcript,
        output_language=args.output_language,
        max_tokens=args.max_tokens,
    )
    output_dir = write_run_artifacts(
        root=Path(args.output_root),
        strategy=args.strategy,
        video_id=args.video_id,
        outcome=outcome,
    )
    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_runner -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/runner.py research/youtube_pipeline/tests/test_runner.py
git commit -m "feat: add youtube research runner artifacts"
```

Expected: commit succeeds.

---

### Task 10: README And Full Verification

**Files:**
- Create: `research/youtube_pipeline/README.md`

- [ ] **Step 1: Add README**

Create `research/youtube_pipeline/README.md`:

```markdown
# YouTube Pipeline Research

This directory contains a local Python research prototype for comparing
YouTube summary LLM pipeline strategies.

It reads local transcript files and writes run artifacts under
`research/youtube_pipeline/runs/`.

## Environment

Set these variables for an OpenAI-compatible chat completions endpoint:

```powershell
$env:YOUTUBE_RESEARCH_LLM_BASE_URL = "https://api.openai.com/v1"
$env:YOUTUBE_RESEARCH_LLM_API_KEY = "..."
$env:YOUTUBE_RESEARCH_LLM_MODEL = "..."
```

## Run One Strategy

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/video_long.txt `
  --video-id video_long `
  --strategy two_pass_summary_structure `
  --output-language ru `
  --max-tokens 8192
```

## Run Tests

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```
```

- [ ] **Step 2: Run all Python tests**

Run:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: all tests pass.

- [ ] **Step 3: Check git status**

Run:

```powershell
git status --short
```

Expected: README is modified or added; generated run artifacts are not listed.

- [ ] **Step 4: Commit**

Run:

```powershell
git add research/youtube_pipeline/README.md
git commit -m "docs: add youtube research runner usage"
```

Expected: commit succeeds.

---

## Self-Review

Spec coverage:

- Local transcript inputs are covered by Task 1 and Task 9.
- Normalized output contract is covered by Task 2.
- Metrics are covered by Task 3 and Task 9.
- One-shot baseline is covered by Task 7.
- Five named strategies are covered by Task 8.
- Generated artifacts are covered by Task 9.
- Usage documentation is covered by Task 10.

Intentional first-implementation limitation:

- Task 8 registers all five strategies, but several wrappers initially reuse the
  one-shot/two-pass plumbing. That is acceptable for the first runnable
  prototype because it creates a tested CLI and artifact format. The next
  research iteration should make `chunk_map_reduce` and
  `timeline_segment_reduce` use distinct chunking and segment prompts.

Verification command:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: all tests pass before handing the prototype to manual LLM runs.
