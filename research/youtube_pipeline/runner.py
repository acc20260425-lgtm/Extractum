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
