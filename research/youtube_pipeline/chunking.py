import re


TIMESTAMP_RE = re.compile(r"^\[(?:(\d{1,2}):)?(\d{2}):(\d{2})\]")


def parse_timestamp_seconds(line: str) -> int | None:
    match = TIMESTAMP_RE.match(line.strip())
    if not match:
        return None
    hours = int(match.group(1) or 0)
    minutes = int(match.group(2))
    seconds = int(match.group(3))
    return hours * 3600 + minutes * 60 + seconds


def chunk_by_approx_tokens(transcript: str, max_tokens: int) -> list[str]:
    words = transcript.split()
    if max_tokens <= 0:
        raise ValueError("max_tokens must be positive")
    return [" ".join(words[index : index + max_tokens]) for index in range(0, len(words), max_tokens)]
