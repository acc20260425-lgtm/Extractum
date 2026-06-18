import argparse
from pathlib import Path

from research.youtube_pipeline.moc_agentic import chunk_transcript_text, write_jsonl


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Chunk a timestamped transcript into JSONL rows.")
    parser.add_argument("--transcript", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--language", default="ru")
    parser.add_argument("--target-tokens", required=True, type=int)
    parser.add_argument("--overlap-tokens", default=0, type=int)
    args = parser.parse_args(argv)

    chunks = chunk_transcript_text(
        args.transcript.read_text(encoding="utf-8"),
        target_tokens=args.target_tokens,
        overlap_tokens=args.overlap_tokens,
        language=args.language,
    )
    write_jsonl(args.out, chunks)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
