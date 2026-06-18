import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import normalize_transcript_text


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Normalize a timestamped transcript.")
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args(argv)

    normalized = normalize_transcript_text(args.input.read_text(encoding="utf-8"))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(normalized, encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
