---
description: Launch Extractum (SvelteKit + Tauri) for UI/UX work and interact with it via browser or native window
---

# Running Extractum

## Mode A — Browser only (UI dev, fastest iteration)

```powershell
Start-Process powershell -ArgumentList "-NoProfile -Command `"cd 'G:\Develop\Extractum'; npm run dev`""
```

Wait ~5s for Vite to start, then open: **http://localhost:1420**

Use **Chrome MCP** (`mcp__Claude_in_Chrome__*`) to interact:
- `navigate` → `http://localhost:1420`
- `read_page` → inspect DOM, find elements
- `find` → locate buttons/inputs by text or selector
- `form_input` / `left_click` → interact with UI
- `read_console_messages` → catch JS errors
- `read_network_requests` → inspect API calls

## Mode B — Full Tauri app (native window, release-like)

```powershell
Start-Process powershell -ArgumentList "-NoProfile -Command `"cd 'G:\Develop\Extractum'; npx tauri dev`""
```

Wait ~30-60s for Rust compilation + Vite. Tauri window opens automatically.

Use **Computer Use MCP** (`mcp__computer-use__*`) to interact:
- `screenshot` → see current state
- `left_click` → click elements
- `type` → type text
- `scroll` → scroll content
- `key` → keyboard shortcuts

## UI/UX analysis skills

After launching, apply these skills:
- `/verify` — confirm a specific change works
- `design:design-critique` — structured feedback (hierarchy, consistency, usability)
- `design:accessibility-review` — WCAG 2.1 AA audit
- `design:design-system` — audit Tailwind tokens and component variants

## Key paths

- Svelte components: `src/lib/**/*.svelte`
- Routes: `src/routes/`
- Tailwind config: auto-detected from `tailwindcss` vite plugin
- Components config: `components.json` (shadcn/bits-ui setup)
