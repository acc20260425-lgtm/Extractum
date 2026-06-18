import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import build_structured_analysis


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build deterministic structured analysis from fact clusters.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    build_structured_analysis(args.run_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
