# NotebookLM Source Group Export - Historical Note

> Status: shipped and archived. Current NotebookLM export behavior is described
> in root docs and implemented in `src-tauri/src/notebooklm_export/`.

## Decision

Telegram source groups can be exported to NotebookLM from local SQLite state.
The export is group-scoped, uses source-group membership, and does not make live
Telegram requests.

YouTube source-group NotebookLM export remains intentionally unsupported until
YouTube-specific export enrichment is designed.

## Rationale

- Group export should preserve the same local-only safety boundary as
  single-source export.
- The archive/read model is the shared source for message text, reply/thread
  context, reactions, media references, and source attribution.
- A generated package should make group membership visible through a manifest
  and per-source metadata, not by hiding all context in one flat text stream.

## Preserved Contract

- Accept `source_id` or `source_group_id` according to the selected export mode.
- Include a `sources/` section for group members.
- Preserve sanitized warnings and unavailable-source notes.
- Fail clearly for unsupported YouTube group export instead of producing a
  partial or misleading archive.

## Current Pointers

- Backend renderer and model: `src-tauri/src/notebooklm_export/`
- Frontend dialog/API: `src/lib/components/analysis/notebooklm-export-dialog.svelte`
  and `src/lib/api/notebooklm-export.ts`
