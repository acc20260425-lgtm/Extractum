# Universal Source Item Browser - Historical Note

> Status: shipped and archived. This is the original rationale behind the
> provider-neutral Source Browser surface.

## Decision

Sources from different providers use a shared browser shell with provider-aware
tabs and item renderers. The shell provides navigation, activity, metadata, and
context patterns; provider code supplies typed data.

## Rationale

- Telegram, YouTube video, YouTube playlist, source group, and saved-run
  snapshot browsing should feel like one product area.
- Provider-specific data still needs typed boundaries so the UI does not parse
  raw payloads.
- Activity and metadata are first-class inspection surfaces, not hidden debug
  details.

## Preserved Contract

- Use shared browser presentation for common browsing behavior.
- Keep provider-specific item shape in typed data models.
- Keep Activity and Metadata available where data exists.
- Prefer explicit unsupported/unavailable states over silent empty tabs.
