import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import quality_check


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run deterministic quality and coverage checks.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    coverage = quality_check(args.run_dir)
    return 0 if coverage["valid"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
