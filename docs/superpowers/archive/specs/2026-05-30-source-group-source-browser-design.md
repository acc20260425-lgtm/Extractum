# Source Group Source Browser - Historical Note

> Status: shipped and archived. Current behavior lives in the shared Source
> Browser shell and analysis route data loaders.

## Decision

Source groups open in `SourceBrowserShell` with an explicit group subject. The
route owns group data loading and passes prepared browser data into the shell.

## Rationale

- Group browsing should reuse the same source browser interaction model as
  single-source and snapshot browsing.
- The shell should not infer group state from unrelated props.
- A dedicated `Sources` tab makes group membership visible before users inspect
  individual items.

## Preserved Contract

- The shell receives an explicit subject for groups.
- Route-level data preparation owns group membership and aggregate activity.
- Group browsing keeps live actions constrained to what the current group
  context actually supports.
