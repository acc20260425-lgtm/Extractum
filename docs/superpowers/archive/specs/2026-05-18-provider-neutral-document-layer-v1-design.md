# Provider-Neutral Document Layer v1 - Historical Note

> Status: shipped and archived. Current table details are in
> `docs/database-schema.md`.

## Decision

Analysis uses provider-neutral document rows to represent material that can be
read, cited, browsed, and exported without decoding provider-specific payloads
at the UI boundary.

## Rationale

- Telegram and YouTube need a shared analysis/read model.
- Provider-specific identity still matters, but the analysis surface should not
  depend on raw provider schemas.
- Document refs and kinds give analysis, browsing, and export code a stable
  vocabulary.

## Preserved Contract

- Keep provider-specific raw payloads out of normal runtime UI reads.
- Store document kind and references explicitly.
- Treat provider-neutral rows as a read model, not as the only source of truth
  for provider ingest.
- Add new provider capabilities by extending typed boundaries first.
