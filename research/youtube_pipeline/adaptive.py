from dataclasses import dataclass
import math
from typing import Any


MAX_REPORT_WORDS = 20000
MAX_CHAPTERS = 20


@dataclass
class BudgetPlan:
    transcript_words: int
    report_min_words: int
    report_max_words: int
    target_report_words: int
    chapter_count: int
    chapter_word_target: int
    substance_multiplier: float
    average_substance_score: float


def normalize_substance_score(value: object) -> int:
    try:
        score = int(value)
    except (TypeError, ValueError):
        return 3
    return max(1, min(5, score))


def compute_substance_multiplier(scores: list[int]) -> float:
    if not scores:
        return 1.0
    average = sum(scores) / len(scores)
    return 0.7 + 0.6 * ((average - 1) / 4)


def select_base_word_range(transcript_words: int) -> tuple[int, int]:
    if transcript_words < 5000:
        return 1000, 1800
    if transcript_words < 15000:
        return 2000, 3500
    if transcript_words < 35000:
        return 4000, 6500
    if transcript_words < 70000:
        return 7000, 10000
    return 10000, 14000


def depth_multiplier(target_depth: str) -> float:
    return {
        "auto": 1.0,
        "brief": 0.5,
        "standard": 1.0,
        "deep": 1.5,
        "book": 2.0,
    }.get(target_depth, 1.0)


def compute_chapter_word_target(target_report_words: int, chapter_count: int) -> int:
    return max(1, round(target_report_words / max(1, chapter_count)))


def compute_budget_plan(
    *,
    transcript_words: int,
    substance_scores: list[int],
    options: Any,
    chunk_count: int | None = None,
) -> BudgetPlan:
    if options.min_report_words is not None and options.max_report_words is not None:
        if options.min_report_words > options.max_report_words:
            raise ValueError("min_report_words cannot be greater than max_report_words")
    base_min, base_max = select_base_word_range(transcript_words)
    average_score = sum(substance_scores) / len(substance_scores) if substance_scores else 3.0
    substance_multiplier = compute_substance_multiplier(substance_scores)
    multiplier = depth_multiplier(options.target_depth) * substance_multiplier
    scaled_min = round(base_min * multiplier)
    scaled_max = round(base_max * multiplier)
    report_min_words = options.min_report_words if options.min_report_words is not None else scaled_min
    report_max_words = options.max_report_words if options.max_report_words is not None else scaled_max
    report_min_words = min(report_min_words, MAX_REPORT_WORDS)
    report_max_words = min(report_max_words, MAX_REPORT_WORDS)
    if report_min_words > report_max_words:
        raise ValueError("min_report_words cannot be greater than max_report_words")
    target_report_words = round((report_min_words + report_max_words) / 2)
    chapter_count = max(1, round(target_report_words / max(1, options.chapter_target_words)))
    if chunk_count is not None:
        chapter_count = min(chapter_count, max(1, chunk_count))
    chapter_count = min(chapter_count, MAX_CHAPTERS)
    return BudgetPlan(
        transcript_words=transcript_words,
        report_min_words=report_min_words,
        report_max_words=report_max_words,
        target_report_words=target_report_words,
        chapter_count=chapter_count,
        chapter_word_target=compute_chapter_word_target(target_report_words, chapter_count),
        substance_multiplier=substance_multiplier,
        average_substance_score=average_score,
    )


def partition_weighted_chunks(weights: list[int | float], chapter_count: int) -> list[tuple[int, int]]:
    n = len(weights)
    if n == 0:
        return []
    k = max(1, min(chapter_count, n))
    prefix = [0.0]
    for weight in weights:
        prefix.append(prefix[-1] + float(weight))
    target = prefix[-1] / k
    dp = [[float("inf")] * (k + 1) for _ in range(n + 1)]
    cut = [[0] * (k + 1) for _ in range(n + 1)]
    dp[0][0] = 0.0
    for end in range(1, n + 1):
        for groups in range(1, min(k, end) + 1):
            for start in range(groups - 1, end):
                chapter_weight = prefix[end] - prefix[start]
                cost = dp[start][groups - 1] + (chapter_weight - target) ** 2
                if cost < dp[end][groups]:
                    dp[end][groups] = cost
                    cut[end][groups] = start
    groups_out: list[tuple[int, int]] = []
    end = n
    groups = k
    while groups > 0:
        start = cut[end][groups]
        groups_out.append((start, end))
        end = start
        groups -= 1
    groups_out.reverse()
    return groups_out


def first_words(text: str, limit: int) -> str:
    return " ".join(text.split()[:limit])


def build_outline_chunk_descriptors(chunk_results: list[dict[str, Any]]) -> list[dict[str, Any]]:
    descriptors: list[dict[str, Any]] = []
    for row in chunk_results:
        result = row.get("result") if isinstance(row.get("result"), dict) else {}
        timeline = result.get("timeline") if isinstance(result.get("timeline"), list) else []
        claims = result.get("claims") if isinstance(result.get("claims"), list) else []
        snippets: list[str] = []
        for item in timeline:
            if isinstance(item, dict) and item.get("title"):
                snippets.append(str(item["title"]))
            if len(snippets) >= 3:
                break
        for item in claims:
            if len(snippets) >= 3:
                break
            if isinstance(item, dict) and item.get("text"):
                snippets.append(str(item["text"]))
        descriptors.append(
            {
                "chunk_index": row.get("chunk_index"),
                "substance_score": row.get("substance_score", 3),
                "summary_preview": first_words(str(result.get("summary_text", "")), 100),
                "snippets": snippets[:3],
            }
        )
    return descriptors


def extract_previous_chapter_bridge(chapter_text: str, max_words: int = 200) -> str:
    stripped = chapter_text.strip()
    if not stripped:
        return ""
    paragraphs = [part.strip() for part in stripped.split("\n\n") if part.strip()]
    candidate = paragraphs[-1] if paragraphs else stripped
    words = candidate.split()
    if len(words) < 30 or candidate.lstrip().startswith(("-", "*", "1.")):
        words = stripped.split()[-max_words:]
    else:
        words = words[-max_words:]
    return " ".join(words)


def language_token_multiplier(output_language: str) -> float:
    normalized = output_language.lower()
    if normalized.startswith("en"):
        return 1.8
    if normalized.startswith("ru"):
        return 2.8
    return 3.0


def response_token_budget(target_words: int, output_language: str, max_tokens: int) -> int:
    estimate = math.ceil(target_words * language_token_multiplier(output_language) * 1.15)
    return min(max_tokens, estimate)


def build_table_of_contents(chapter_titles: list[str]) -> str:
    lines = ["## Table of Contents", "", "- Executive Overview", "- Part I: Detailed Narrative"]
    for index, title in enumerate(chapter_titles, start=1):
        lines.append(f"  - Chapter {index}: {title}")
    lines.extend(
        [
            "- Part II: Structured Analysis",
            "  - Timeline and Development of Ideas",
            "  - Major Claims and Evidence",
            "  - Actionable Takeaways",
            "  - Open Questions",
            "- Conclusion and Synthesis",
        ]
    )
    return "\n".join(lines)


def assemble_adaptive_markdown_report(
    *,
    overview: str,
    chapters: list[str],
    chapter_titles: list[str],
    timeline_markdown: str,
    claims_markdown: str,
    action_items_markdown: str,
    open_questions_markdown: str,
    conclusion: str,
) -> str:
    parts = [
        "# YouTube Research Report",
        "",
        "Generated via `adaptive_book_report`.",
        "",
        build_table_of_contents(chapter_titles),
        "",
        "## Executive Overview",
        "",
        overview.strip(),
        "",
        "# Part I: Detailed Narrative",
        "",
        "\n\n".join(chapter.strip() for chapter in chapters if chapter.strip()),
        "",
        "# Part II: Structured Analysis",
        "",
        "## Timeline and Development of Ideas",
        "",
        timeline_markdown.strip(),
        "",
        "## Major Claims and Evidence",
        "",
        claims_markdown.strip(),
        "",
        "## Actionable Takeaways",
        "",
        action_items_markdown.strip(),
        "",
        "## Open Questions",
        "",
        open_questions_markdown.strip(),
        "",
        "## Conclusion and Synthesis",
        "",
        conclusion.strip(),
    ]
    return "\n".join(parts).strip() + "\n"
