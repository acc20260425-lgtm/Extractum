import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import assemble_report


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Assemble final agentic MoC report.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    assemble_report(args.run_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
