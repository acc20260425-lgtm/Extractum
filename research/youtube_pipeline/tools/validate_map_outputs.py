import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import validate_map_outputs


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate map extractor output files.")
    parser.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    manifest = validate_map_outputs(args.run_dir)
    missing_outputs = [
        invalid
        for invalid in manifest["invalid_outputs"]
        if "output file missing" in [str(error) for error in invalid.get("errors", [])]
    ]
    print(
        "validate_map_outputs: "
        f"valid_outputs={len(manifest['valid_outputs'])} "
        f"invalid_outputs={len(manifest['invalid_outputs'])} "
        f"missing_outputs={len(missing_outputs)}"
    )
    for invalid in manifest["invalid_outputs"][:5]:
        output_file = invalid.get("output_file", "<unknown>")
        errors = invalid.get("errors", [])
        if isinstance(errors, list) and errors:
            print(f"- {output_file}: {'; '.join(str(error) for error in errors[:5])}")
        else:
            print(f"- {output_file}: invalid schema")
    print("manifest=map/validation_manifest.json")
    return 1 if manifest["invalid_outputs"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
