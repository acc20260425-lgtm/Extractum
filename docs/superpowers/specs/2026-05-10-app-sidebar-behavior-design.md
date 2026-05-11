# App Sidebar Behavior Design

## Implementation Status

Status: implemented on `main`. The active shell uses
`src/lib/components/app-sidebar.svelte`, persists the desktop collapsed state,
keeps the theme toggle in the topbar, and uses an off-canvas mobile drawer.

The old implementation plan was removed from active docs because it was an
execution checklist. This design remains the source of truth for intended shell
behavior.

## Context

Before this change, Extractum kept the primary application navigation directly in `src/routes/+layout.svelte` as a fixed-width left sidebar. The sidebar worked on desktop and became a top horizontal navigation on narrow screens. The implemented change makes Extractum's left application panel behave like OmniRoute's shell in the ways that matter for this app: desktop collapse/expand and mobile off-canvas navigation.

## Goals

- Move the primary sidebar into a dedicated `AppSidebar` component.
- Keep the Extractum visual language while adopting OmniRoute-style behavior.
- On desktop, allow the sidebar to collapse into a narrow icon rail.
- Persist the desktop collapsed state in `localStorage`.
- On mobile, replace the current horizontal navigation with an off-canvas drawer.
- Move the theme toggle from the sidebar footer into the topbar so it remains available in collapsed and mobile states.

## Non-Goals

- Do not copy OmniRoute's full Material-style visual system.
- Do not introduce a global layout store for this change.
- Do not add full end-to-end tests to the repository unless the project already has that pattern.
- Do not alter page-level layouts beyond the shell changes needed for the sidebar behavior.

## Component Architecture

Create `src/lib/components/app-sidebar.svelte` for the primary navigation. It owns the sidebar markup and styling for:

- the Extractum brand link;
- the primary navigation links;
- desktop collapse/expand control;
- the sidebar footer shown only in expanded desktop mode;
- mobile drawer rendering.

`src/routes/+layout.svelte` remains the owner of shell state and passes the current route and callbacks into `AppSidebar`.
The `AppSidebar` root drawer/sidebar element should use the stable id `app-sidebar`; the mobile menu button should point to that id with `aria-controls`.

`+layout.svelte` should keep:

- `theme`;
- `sidebarCollapsed`;
- `mobileSidebarOpen`;
- topbar rendering;
- mobile menu button;
- `localStorage` read/write for theme and desktop collapsed state.

The navigation items may stay in `+layout.svelte` and be passed into `AppSidebar`, or move into `AppSidebar` if that keeps the component easier to read. The implementation should avoid duplicating the navigation data between desktop and mobile render paths.

## Desktop Behavior

On first run, the sidebar is expanded. After the user toggles it, store the preference under a stable key:

```text
extractum.sidebar.collapsed
```

Expanded desktop sidebar:

- uses approximately the current sidebar width;
- shows the `E` brand mark, `Extractum`, and the brand subtitle;
- shows nav icons, labels, and captions;
- shows the sidebar footer copy;
- includes a clear collapse button.

The desktop collapse/expand transition should be a lightweight CSS transition on sidebar width/flex-basis. No JavaScript animation is needed.

Collapsed desktop sidebar:

- becomes a narrow icon rail;
- shows the collapse/expand button, the `E` brand mark, and nav icons;
- hides brand text, nav labels, nav captions, and footer;
- preserves active route styling in icon form;
- provides `aria-label` and `title` text for icon-only nav links.

Changing desktop collapsed state must not open or close the mobile drawer.

## Mobile Behavior

At the mobile breakpoint, the sidebar is removed from the normal layout flow and becomes a fixed off-canvas drawer.

Mobile closed state:

- topbar shows a menu button on the left;
- content uses the full viewport width;
- the sidebar is positioned off-screen.

Mobile open state:

- the drawer slides in from the left at expanded sidebar width;
- nav content is readable and expanded, regardless of saved desktop collapsed state;
- an overlay covers the content area;
- clicking the overlay closes the drawer;
- pressing `Escape` closes the drawer;
- clicking a navigation link closes the drawer after route selection, matching OmniRoute.

The mobile drawer should not persist open/closed state in `localStorage`.

## Topbar And Theme

Move the theme toggle into the workspace topbar. It should be available in all shell modes:

- desktop expanded;
- desktop collapsed;
- mobile drawer closed;
- mobile drawer open.

The topbar can keep the current route title and badges. On mobile, the menu button should sit at the left side of the topbar and not crowd the route title or theme button.
After moving the theme toggle, the sidebar footer should keep only the workspace-mode copy. It should be shown in expanded desktop and mobile-open drawer states, and hidden in collapsed desktop mode.

## Responsive Rules

Use the existing narrow-screen breakpoint unless implementation proves it too tight. The expected behavior is:

- above the breakpoint: flex shell with sidebar and workspace side by side;
- at or below the breakpoint: workspace becomes full-width and sidebar becomes off-canvas.

The mobile drawer always renders in expanded form. It must not inherit the desktop collapsed width.

## Accessibility

- Sidebar toggle must be a button with an accessible label that reflects the next action.
- Mobile menu button must expose expanded state and point to the drawer with `aria-controls`.
- The drawer should have a useful `aria-label`.
- Icon-only nav items must keep accessible names.
- Escape key closes the mobile drawer.
- Overlay click closes the mobile drawer without trapping users.
- Opening the mobile drawer should move focus to the `app-sidebar` drawer container so the next Tab reaches drawer content. Users can close it with Escape, close it via the overlay, or close it by selecting a nav link. No focus trap is required for this lightweight navigation drawer unless testing shows keyboard users can tab into hidden content in an incoherent order.

## Testing And Verification

Add or update focused Vitest source tests to lock the intended shell behavior:

- `AppSidebar` exists and is used by `+layout.svelte`.
- `AppSidebar` exposes the stable `app-sidebar` id used by the mobile menu button's `aria-controls`.
- The collapsed preference key is present.
- The layout includes desktop collapsed state and mobile drawer state.
- Mobile nav selection has a close callback path.
- Theme toggle is in the topbar rather than only in the sidebar footer.
- The mobile menu button remains hidden on desktop even though `Button.svelte`
  applies `.ui-button { display: inline-flex; }`.

Run:

```text
npm run check
```

Then run the app locally and verify with Playwright:

- desktop expanded by default with no stored preference;
- desktop collapsed after toggling;
- collapsed state persists after reload;
- mobile drawer opens from the topbar menu button;
- mobile nav click closes the drawer;
- theme toggle remains accessible in topbar.

### Playwright Session Notes

These details are intentionally recorded so the next verification session can
start cleanly:

- On this Windows/PowerShell setup, prefer `npm.cmd` for scripts. Plain `npm`
  may hit PowerShell execution policy for `npm.ps1`.
- Do not assume Vite uses port `5173`. Use the actual URL Vite prints. In this
  session it used `http://127.0.0.1:1420/`.
- If a sandboxed background `npm run dev` process exits immediately or produces
  empty logs, start Vite outside the sandbox and keep the process alive, for
  example:

  ```powershell
  $cmd = 'Set-Location -LiteralPath ''G:\Develop\Extractum''; node.exe node_modules/vite/bin/vite.js --host 127.0.0.1'
  Start-Process -FilePath 'powershell.exe' -ArgumentList @('-NoLogo','-NoExit','-Command',$cmd) -PassThru -WindowStyle Hidden
  ```

- Stop the dev server by identifying the listening PID for the actual port:

  ```powershell
  netstat -ano | findstr ":1420"
  Stop-Process -Id <LISTENING_PID> -Force
  ```

- `.playwright-mcp/` is a generated local artifact and should stay ignored and
  unstaged.
- Tauri IPC errors in the browser console are expected when opening the Svelte
  app directly in a browser rather than inside Tauri. They did not block sidebar
  verification.

## Implementation Notes

Keep the change scoped to the shell. Prefer existing colors, CSS variables, button styling, and route title behavior. Avoid introducing a new global store unless the implementation becomes awkward without one.

## Implementation Learnings

- `Button.svelte` emits the `.ui-button` class and its local CSS sets
  `display: inline-flex`. Layout CSS that needs to hide/show a `Button` by
  class should use a selector at least as specific as
  `:global(.mobile-menu-button.ui-button)`.
- Avoid putting `tabindex={mobileOpen ? -1 : undefined}` directly on the
  noninteractive `aside`. Svelte autofixer reports
  `a11y_no_noninteractive_tabindex`. The accepted implementation keeps
  `role="navigation"` on `#app-sidebar` and sets/removes `tabindex="-1"`
  programmatically immediately before focusing the drawer on mobile open.
