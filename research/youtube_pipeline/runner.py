import argparse
import json
import os
from pathlib import Path
from typing import Any

from research.youtube_pipeline.llm_client import OpenAICompatibleClient
from research.youtube_pipeline.metrics import build_metrics
from research.youtube_pipeline.strategies import STRATEGIES, StrategyOptions, StrategyOutcome


def write_json(path: Path, payload: Any) -> None:
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")


def write_jsonl(path: Path, rows: list[dict[str, object]]) -> None:
    path.write_text(
        "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows),
        encoding="utf-8",
    )


def write_extra_artifact(path: Path, payload: Any) -> None:
    if path.name != str(path) or path.name in {"", ".", ".."}:
        raise ValueError(f"extra artifact filename must be a simple relative name: {path}")
    if isinstance(payload, str):
        path.write_text(payload, encoding="utf-8")
    else:
        write_json(path, payload)


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
    metrics = build_metrics(
        strategy=strategy,
        video_id=video_id,
        result=outcome.result,
        request_count=outcome.request_count,
        input_tokens=outcome.input_tokens,
        output_tokens=outcome.output_tokens,
        latency_seconds=outcome.latency_seconds,
        json_valid=outcome.json_valid,
    )
    metrics.update(outcome.extra_metrics)
    write_json(output_dir / "metrics.json", metrics)
    write_jsonl(output_dir / "raw_requests.jsonl", outcome.raw_requests)
    write_jsonl(output_dir / "raw_responses.jsonl", outcome.raw_responses)
    for filename, payload in outcome.extra_artifacts.items():
        artifact_name = Path(filename)
        if artifact_name.name != str(artifact_name) or artifact_name.name in {"", ".", ".."}:
            raise ValueError(f"extra artifact filename must be a simple relative name: {artifact_name}")
        output_path = output_dir / artifact_name
        if isinstance(payload, str):
            output_path.write_text(payload, encoding="utf-8")
        else:
            write_json(output_path, payload)
    return output_dir


def build_client_from_env() -> OpenAICompatibleClient:
    return OpenAICompatibleClient(
        base_url=os.environ["YOUTUBE_RESEARCH_LLM_BASE_URL"],
        api_key=os.environ["YOUTUBE_RESEARCH_LLM_API_KEY"],
        model=os.environ["YOUTUBE_RESEARCH_LLM_MODEL"],
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run YouTube summary pipeline research strategies.")
    parser.add_argument("--input", required=True, help="Path to transcript text file")
    parser.add_argument("--strategy", required=True, choices=sorted(STRATEGIES))
    parser.add_argument("--video-id", required=True)
    parser.add_argument("--output-root", default="research/youtube_pipeline/runs/manual")
    parser.add_argument("--output-language", default="ru")
    parser.add_argument("--max-tokens", type=int, default=8192)
    parser.add_argument("--chunk-token-limit", type=int, default=3000)
    parser.add_argument("--chunk-overlap-tokens", type=int, default=700)
    parser.add_argument("--target-depth", choices=["auto", "brief", "standard", "deep", "book"], default="auto")
    parser.add_argument("--min-report-words", type=int, default=None)
    parser.add_argument("--max-report-words", type=int, default=None)
    parser.add_argument("--chapter-target-words", type=int, default=900)
    parser.add_argument("--planner-context-token-limit", type=int, default=120000)
    parser.add_argument("--max-slice-tokens", type=int, default=8000)
    return parser


def build_strategy_options(args: argparse.Namespace) -> StrategyOptions:
    if args.min_report_words is not None and args.max_report_words is not None:
        if args.min_report_words > args.max_report_words:
            raise ValueError("min-report-words cannot be greater than max-report-words")
    return StrategyOptions(
        output_language=args.output_language,
        video_id=args.video_id,
        max_tokens=args.max_tokens,
        chunk_token_limit=args.chunk_token_limit,
        chunk_overlap_tokens=args.chunk_overlap_tokens,
        target_depth=args.target_depth,
        min_report_words=args.min_report_words,
        max_report_words=args.max_report_words,
        chapter_target_words=args.chapter_target_words,
        planner_context_token_limit=args.planner_context_token_limit,
        max_slice_tokens=args.max_slice_tokens,
    )


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    transcript = Path(args.input).read_text(encoding="utf-8")
    client = build_client_from_env()
    options = build_strategy_options(args)
    outcome = STRATEGIES[args.strategy](
        client=client,
        transcript=transcript,
        options=options,
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
