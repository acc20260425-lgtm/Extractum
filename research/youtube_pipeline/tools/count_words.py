import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import word_count


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Count words in a text file.")
    parser.add_argument("--input", required=True, type=Path)
    args = parser.parse_args(argv)

    print(word_count(args.input.read_text(encoding="utf-8")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
