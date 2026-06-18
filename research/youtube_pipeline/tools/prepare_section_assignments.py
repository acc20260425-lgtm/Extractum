import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import prepare_section_assignments


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Prepare section writer assignments from MoC alignment.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    prepare_section_assignments(args.run_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
