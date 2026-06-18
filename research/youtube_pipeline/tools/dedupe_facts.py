import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import dedupe_facts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Deduplicate mapped facts deterministically.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    dedupe_facts(args.run_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
