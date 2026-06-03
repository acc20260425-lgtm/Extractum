# YouTube Typed Source Metadata - Historical Note

> Status: shipped and archived. Current table details live in
> `docs/database-schema.md`.

## Decision

YouTube runtime behavior uses typed source metadata tables instead of parsing
raw provider payloads in normal UI/read paths.

## Rationale

- Videos and playlists need provider-specific fields, but those fields should
  be queryable and validated.
- Raw provider payloads are useful as import/debug artifacts, not as routine
  source identity or browser state.
- Typed tables make Source Browser, analysis, and future export work safer.

## Preserved Contract

- Keep typed YouTube ownership explicit.
- Treat raw payloads as non-runtime data.
- Add new YouTube behavior through typed table/read-model changes.
