import json
import unittest

from research.youtube_pipeline.llm_client import ChatMessage, OpenAICompatibleClient


class FakeTransport:
    def __init__(self):
        self.calls = []

    def post_json(self, url, headers, payload):
        self.calls.append((url, headers, payload))
        return {
            "choices": [{"message": {"content": "{\"summary_text\":\"ok\"}"}}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5},
        }


class LlmClientTests(unittest.TestCase):
    def test_chat_completion_sends_openai_compatible_payload(self):
        transport = FakeTransport()
        client = OpenAICompatibleClient(
            base_url="https://example.test/v1",
            api_key="secret",
            model="test-model",
            transport=transport,
        )

        response = client.complete([ChatMessage(role="user", content="Hello")], max_tokens=100)

        self.assertEqual(response.text, "{\"summary_text\":\"ok\"}")
        self.assertEqual(response.input_tokens, 10)
        self.assertEqual(response.output_tokens, 5)
        sent_payload = transport.calls[0][2]
        self.assertEqual(sent_payload["model"], "test-model")
        self.assertEqual(sent_payload["max_tokens"], 100)
        self.assertEqual(sent_payload["messages"], [{"role": "user", "content": "Hello"}])
        self.assertEqual(json.loads(json.dumps(sent_payload))["model"], "test-model")


if __name__ == "__main__":
    unittest.main()
