# App Sidebar Behavior - Historical Note

> Status: shipped and archived. This note keeps the durable behavior contract;
> current implementation details live in the app shell.

## Decision

The app sidebar supports persistent desktop collapse state and a separate mobile
drawer behavior. The two modes should not leak state into each other.

## Rationale

- Desktop users benefit from a remembered compact navigation state.
- Mobile users need a temporary drawer that closes after navigation and does not
  depend on desktop width preferences.
- Sidebar behavior affects every route, so the contract should stay predictable
  and accessible.

## Preserved Contract

- Store desktop collapsed state in local storage.
- Keep mobile drawer visibility ephemeral.
- Ensure navigation items remain keyboard reachable and screen-reader labeled.
- Avoid treating the sidebar spec as the current source of truth for every app
  shell detail; use current shell code and root docs for that.
