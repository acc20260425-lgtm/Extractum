# Source Browser Data Prop Consolidation - Historical Note

> Status: shipped and archived.

## Decision

Source Browser modes use explicit prepared data props instead of many
mode-specific ad hoc props.

## Rationale

- `sourceBrowserData`, `groupBrowserData`, and `snapshotBrowserData` make the
  route-to-shell boundary visible.
- Prepared data objects are easier to test than shell-side inference.
- The shell should render the selected subject and data, not fetch or reshape
  unrelated route state.

## Preserved Contract

- Routes own loading and data preparation.
- The shell owns common presentation and interactions.
- New browser modes should add explicit data contracts rather than widening
  optional prop soup.
