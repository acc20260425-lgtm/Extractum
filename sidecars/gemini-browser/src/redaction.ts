const SECRET_QUERY_KEYS = new Set(["authuser", "token", "key", "password", "prompt"]);

export function redactUrl(rawUrl: string): string {
  try {
    const url = new URL(rawUrl);
    for (const key of [...url.searchParams.keys()]) {
      if (SECRET_QUERY_KEYS.has(key.toLowerCase())) {
        url.searchParams.set(key, "[redacted]");
      }
    }
    return url.toString();
  } catch {
    return "[invalid-url]";
  }
}

export function redactText(value: string, prompt: string): string {
  const trimmedPrompt = prompt.trim();
  if (!trimmedPrompt) return value;
  return value.split(trimmedPrompt).join("[prompt]");
}
