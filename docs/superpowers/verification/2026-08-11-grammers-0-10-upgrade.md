# Grammers 0.10 Upgrade Verification

## Automated Gates

- extractum-telegram package tests: passed
- extractum-telegram feature-off check: passed
- workspace verify: passed

## Live Sync

- Source `18` / `Язык Zig (канал)` completed live sync with `inserted = 39`,
  `skipped = 4`, and `last_message_id = 562`.
- Terminal state: `completed`.
- The expected Telegram peer and account binding were unchanged after sync.
- Warning codes: `[]`.

## Takeout

- Source `118` / `Test 2` completed Takeout with terminal status `completed`
  and completeness `complete`.
- The Takeout session was loaded, the export-DC alias was present,
  `used_export_dc = true`, and `fallback_used = false`.
- One message batch completed with progress `33 / 33`; the job reported
  `imported = 0` and `skipped = 31`.
- Warning codes: `[]`.

## Sensitive Data

No Telegram session, API hash, credential, access hash, or raw private payload is included.
