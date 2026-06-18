import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import validate_moc


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate and normalize MoC planner output.")
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--target-words", required=True, type=int)
    parser.add_argument("--chapter-target-words", default=900, type=int)
    args = parser.parse_args(argv)

    validation = validate_moc(
        args.run_dir,
        target_words=args.target_words,
        chapter_target_words=args.chapter_target_words,
    )
    return 1 if validation["fallback_used"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
