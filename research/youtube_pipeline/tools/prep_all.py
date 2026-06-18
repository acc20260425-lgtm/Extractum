import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import write_prep_artifacts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Prepare transcript artifacts for the agentic MoC workflow.")
    parser.add_argument("--transcript", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--language", default="ru")
    parser.add_argument("--target-tokens", required=True, type=int)
    parser.add_argument("--overlap-tokens", default=0, type=int)
    args = parser.parse_args(argv)

    write_prep_artifacts(
        args.transcript,
        args.out,
        target_tokens=args.target_tokens,
        overlap_tokens=args.overlap_tokens,
        language=args.language,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
