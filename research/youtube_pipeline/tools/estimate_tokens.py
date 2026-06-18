import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import estimate_tokens


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Estimate output tokens for a text file.")
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--language", default="ru")
    args = parser.parse_args(argv)

    print(estimate_tokens(args.input.read_text(encoding="utf-8"), language=args.language))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
