import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import validate_map_outputs


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate map extractor output files.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    manifest = validate_map_outputs(args.run_dir)
    return 1 if manifest["invalid_outputs"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
