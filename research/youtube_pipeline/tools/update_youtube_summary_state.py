import argparse
from pathlib import Path

from research.youtube_pipeline.youtube_summary_workflow import update_workflow_state


def parse_key_value(values: list[str]) -> dict[str, object]:
    parsed: dict[str, object] = {}
    for value in values:
        if "=" not in value:
            raise ValueError(f"Expected key=value, got: {value}")
        key, raw = value.split("=", 1)
        parsed[key] = raw
    return parsed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Update a YouTube summary workflow_state.json file.")
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--stage", required=True)
    parser.add_argument("--next-action", required=True)
    parser.add_argument("--artifact", action="append", default=[])
    parser.add_argument("--count", action="append", default=[])
    parser.add_argument("--warning", action="append", default=[])
    args = parser.parse_args(argv)

    counts: dict[str, object] = {}
    for key, value in parse_key_value(args.count).items():
        try:
            counts[key] = int(str(value))
        except ValueError:
            counts[key] = value

    state = update_workflow_state(
        args.run_dir,
        current_stage=args.stage,
        next_action=args.next_action,
        artifacts=parse_key_value(args.artifact),
        counts=counts,
        validation_warnings=args.warning,
    )
    print(f"run_dir={state['run_dir']}")
    print(f"current_stage={state['current_stage']}")
    print(f"next_action={state['next_action']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
