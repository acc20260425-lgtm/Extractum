import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import read_json, validate_generated_files


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate agent-owned generated files.")
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--agent-id", required=True)
    parser.add_argument("--expected-file", action="append", default=[])
    parser.add_argument("--expected-files-manifest", type=Path)
    args = parser.parse_args(argv)

    expected_files = list(args.expected_file)
    if args.expected_files_manifest:
        manifest = read_json(args.expected_files_manifest)
        if not isinstance(manifest, dict) or not isinstance(manifest.get("expected_files"), list):
            parser.error("expected-files-manifest must contain an expected_files list")
        expected_files.extend(str(path) for path in manifest["expected_files"])
    if not expected_files:
        parser.error("provide --expected-file or --expected-files-manifest")

    result = validate_generated_files(args.run_dir, agent_id=args.agent_id, expected_files=expected_files)
    return 0 if result["valid"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
