import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import assemble_map_artifacts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Assemble validated map outputs into canonical artifacts.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    assemble_map_artifacts(args.run_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
