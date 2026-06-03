# Source Browser Legacy Wrapper Cleanup - Historical Note

> Status: shipped and archived.

## Decision

Legacy Source Browser wrapper components were removed after the shared
`SourceBrowserShell` contract became the single browsing surface.

## Rationale

- Wrappers duplicated state decisions and made source, group, and snapshot modes
  drift apart.
- Removing wrappers made route ownership and shell ownership clearer.
- The proof target for this slice was absence of old wrapper imports in `src`
  plus passing Source Browser tests.

## Preserved Invariant

New source-browsing work should extend the explicit shell/data contract rather
than reintroducing wrapper components that hide subject or data ownership.
