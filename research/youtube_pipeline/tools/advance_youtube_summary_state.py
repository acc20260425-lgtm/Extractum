import argparse
from pathlib import Path

from research.youtube_pipeline.youtube_summary_workflow import ADVANCE_TRANSITIONS, advance_workflow_state


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Advance a YouTube summary workflow_state.json after a known step.")
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--after", required=True, choices=sorted(ADVANCE_TRANSITIONS))
    args = parser.parse_args(argv)

    state = advance_workflow_state(args.run_dir, after=args.after)
    print(f"run_dir={state['run_dir']}")
    print(f"current_stage={state['current_stage']}")
    print(f"next_action={state['next_action']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
