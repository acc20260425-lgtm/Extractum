from dataclasses import dataclass
import json
from typing import Any, Protocol
from urllib import error, request


@dataclass
class ChatMessage:
    role: str
    content: str


@dataclass
class LlmResponse:
    text: str
    input_tokens: int
    output_tokens: int


class JsonTransport(Protocol):
    def post_json(self, url: str, headers: dict[str, str], payload: dict[str, Any]) -> dict[str, Any]:
        ...


class UrllibJsonTransport:
    def post_json(self, url: str, headers: dict[str, str], payload: dict[str, Any]) -> dict[str, Any]:
        body = json.dumps(payload).encode("utf-8")
        req = request.Request(url, data=body, headers=headers, method="POST")
        try:
            with request.urlopen(req, timeout=120) as response:
                raw_body = response.read().decode("utf-8", errors="replace")
                status = getattr(response, "status", "unknown")
        except error.HTTPError as exc:
            raw_body = exc.read().decode("utf-8", errors="replace")
            preview = raw_body.strip()[:500]
            raise RuntimeError(f"LLM endpoint returned HTTP {exc.code} for {url}: {preview}") from exc

        stripped = raw_body.strip()
        if not stripped:
            raise RuntimeError(f"LLM endpoint returned empty response for {url} with HTTP status {status}")
        if stripped.startswith("data:"):
            return parse_sse_chat_completion(stripped, url)
        try:
            return json.loads(stripped)
        except json.JSONDecodeError as exc:
            preview = stripped[:500]
            raise RuntimeError(f"LLM endpoint returned non-JSON response for {url}: {preview}") from exc


def parse_sse_chat_completion(body: str, url: str) -> dict[str, Any]:
    content_parts: list[str] = []
    usage: dict[str, Any] = {}
    for line in body.splitlines():
        stripped = line.strip()
        if not stripped.startswith("data:"):
            continue
        event_data = stripped.removeprefix("data:").strip()
        if not event_data or event_data == "[DONE]":
            continue
        try:
            chunk = json.loads(event_data)
        except json.JSONDecodeError as exc:
            raise RuntimeError(f"LLM endpoint returned invalid SSE JSON chunk for {url}: {event_data[:500]}") from exc
        if isinstance(chunk.get("usage"), dict):
            usage = chunk["usage"]
        for choice in chunk.get("choices", []):
            delta = choice.get("delta") or {}
            if "content" in delta:
                content_parts.append(str(delta["content"]))
    return {
        "choices": [{"message": {"content": "".join(content_parts)}}],
        "usage": usage,
    }


class OpenAICompatibleClient:
    def __init__(
        self,
        *,
        base_url: str,
        api_key: str,
        model: str,
        transport: JsonTransport | None = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.model = model
        self.transport = transport or UrllibJsonTransport()

    def complete(self, messages: list[ChatMessage], max_tokens: int) -> LlmResponse:
        payload = {
            "model": self.model,
            "messages": [{"role": message.role, "content": message.content} for message in messages],
            "max_tokens": max_tokens,
            "temperature": 0.2,
        }
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }
        data = self.transport.post_json(f"{self.base_url}/chat/completions", headers, payload)
        text = data["choices"][0]["message"]["content"]
        usage = data.get("usage") or {}
        return LlmResponse(
            text=text,
            input_tokens=int(usage.get("prompt_tokens") or 0),
            output_tokens=int(usage.get("completion_tokens") or 0),
        )
