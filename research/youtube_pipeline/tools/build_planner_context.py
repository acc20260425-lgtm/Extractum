import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import build_planner_context


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build bounded planner context from map artifacts.")
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--max-tokens", default=24000, type=int)
    parser.add_argument("--language", default="ru")
    args = parser.parse_args(argv)

    build_planner_context(args.run_dir, max_tokens=args.max_tokens, language=args.language)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
