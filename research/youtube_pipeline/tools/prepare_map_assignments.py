import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import prepare_map_assignments


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Prepare map extractor assignment files.")
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--output-language", default="ru")
    parser.add_argument("--target-summary-words", default=250, type=int)
    parser.add_argument("--max-fact-count", default=20, type=int)
    args = parser.parse_args(argv)

    prepare_map_assignments(
        args.run_dir,
        output_language=args.output_language,
        target_summary_words=args.target_summary_words,
        max_fact_count=args.max_fact_count,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
