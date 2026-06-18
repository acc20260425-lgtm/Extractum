import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import align_facts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Align deduplicated facts to MoC nodes by chunk id.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    align_facts(args.run_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
