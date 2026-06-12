# New UI Research Projects Verification

Date: 2026-06-12

## Commands

- `npm.cmd run test`: PASS, 78 files / 699 tests
- `npm.cmd run check`: PASS, 0 errors / 0 warnings
- `npm.cmd run build`: PASS

## Visual QA

- `/projects` at 2560x1440: PASS, screenshot `artifacts/new-ui-projects-ultrahd.png`
- `/projects` at 1366x768: PASS, screenshot `artifacts/new-ui-projects-1366.png`
- `Connect from Library` wide sheet: PASS
- Old `/analysis` reachable: PASS by route contract

## Notes

- Browser-only Vite QA renders the shell without Tauri SQL data, so project rows are empty in screenshots.
- Unsupported RSS/forum providers remain visible but disabled through the transition model.
- Telegram/YouTube source-group-backed projects are the only persistable first-slice connect targets.
- SVAR grid build uses `@svar-ui/grid-locales` English grid strings plus `@svar-ui/core-locales` Russian core strings because `grid-locales@2.7.0` does not export `ru`.
