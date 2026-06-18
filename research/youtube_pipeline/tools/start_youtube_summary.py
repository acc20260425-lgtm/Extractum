import argparse
from pathlib import Path

from research.youtube_pipeline.youtube_summary_workflow import DEFAULT_RUN_ROOT, start_youtube_summary_run


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Start or resume a user-facing YouTube summary workflow.")
    parser.add_argument("--transcript", required=True, type=Path)
    parser.add_argument("--run-root", default=DEFAULT_RUN_ROOT, type=Path)
    parser.add_argument("--run-dir", type=Path)
    parser.add_argument("--language", default="ru")
    parser.add_argument("--target-words", default=10000, type=int)
    parser.add_argument("--target-tokens", default=1600, type=int)
    parser.add_argument("--overlap-tokens", default=200, type=int)
    parser.add_argument("--planner-context-tokens", default=24000, type=int)
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args(argv)

    state = start_youtube_summary_run(
        args.transcript,
        run_root=args.run_root,
        run_dir=args.run_dir,
        output_language=args.language,
        target_words=args.target_words,
        target_tokens=args.target_tokens,
        overlap_tokens=args.overlap_tokens,
        planner_context_tokens=args.planner_context_tokens,
        force=args.force,
    )
    print(f"run_dir={state['run_dir']}")
    print(f"current_stage={state['current_stage']}")
    print(f"next_action={state['next_action']}")
    print(f"resumed={str(bool(state.get('resumed'))).lower()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
