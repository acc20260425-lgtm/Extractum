# Desktop Product Evolution Analysis

> Date: 2026-05-29
> Scope: cross-cutting desktop product maturity for Extractum, informed by
> Telegram Desktop reference review under `reference/tdesktop-dev`.

## Executive Summary

Extractum should not copy Telegram Desktop as a chat client. The useful lesson
is how a mature local desktop application handles operability, settings,
exports, diagnostics, and privacy around a complex local data model.

The next project-level improvements should focus on:

1. sanitized diagnostics and local support bundles;
2. searchable settings and action registries;
3. explicit export profile contracts;
4. resilient settings/data migration;
5. local privacy and data lifecycle controls.

These are not immediate feature rewrites. They are product foundations that
will make future Telegram, YouTube, NotebookLM, media, and provider work easier
to support without turning Extractum into a full Telegram client.

## Reference Areas

Reviewed Telegram Desktop areas:

- `core/crash_reports.*`
- `logs.*`
- `export/export_settings.*`
- `export/export_manager.*`
- `settings/settings_search.*`
- `settings/settings_builder.*`
- `core/shortcuts.*`
- `main/main_session_settings.*`
- `storage/storage_domain.*`
- `core/update_checker.*`
- `lang/lang_instance.*`
- `core/local_url_handlers.*`

These files are useful as product-pattern references, not as architecture to
port. Telegram Desktop is a networked chat client with a large UI and runtime
state model. Extractum is a local archive and analysis workspace.

## Recommended Directions

### 1. Sanitized Diagnostic And Support Bundle

Telegram Desktop keeps structured crash annotations, startup/runtime details,
and rotating logs. Extractum does not need a crash-reporting subsystem yet, but
it would benefit from a local diagnostic bundle command or settings screen.

Recommended scope:

- app version and build metadata;
- database schema version and migration status;
- provider availability summary;
- source/job/run counts and recent terminal states;
- recent sanitized backend/frontend logs;
- runtime checks for secure storage, `yt-dlp`, SQLite, and configured provider
  capabilities;
- explicit redaction of LLM keys, Telegram `api_hash`, session material,
  YouTube cookies, message bodies, transcript bodies, and prompt contents.

This should produce local output only unless a future support flow explicitly
adds user-controlled sharing.

### 2. Structured Local Logs With Redaction

Extractum already has typed errors and user-facing status messages. The missing
piece is a consistent local log policy.

Recommended scope:

- separate categories for `telegram`, `youtube`, `llm`, `analysis`, `storage`,
  `export`, and `ui`;
- bounded retention and size limits;
- startup environment summary that excludes secrets and local content;
- redaction helper shared by diagnostics, logs, and debug status text;
- tests for representative redaction cases before exposing diagnostic export.

This complements the existing stabilization and secret-safety backlog. It
should not introduce remote telemetry by default.

### 3. Searchable Settings Registry

Telegram Desktop treats settings as searchable entries with keywords and recent
usage. Extractum settings are already growing around LLM profiles, YouTube
cookies, provider tests, and future privacy/media controls.

Recommended scope:

- a typed registry entry for each settings destination or action;
- title, section, keywords, route/action id, and optional status metadata;
- settings-page search before the page becomes too long;
- reuse by a future command palette if one is added.

This should stay lightweight. The goal is findability, not a large settings
framework.

### 4. Export Profile Contract

Telegram Desktop export uses a validated settings object that makes scope,
media policy, format, and limits explicit. Extractum should use the same idea
for future NotebookLM and archive exports.

Recommended scope:

- export target: NotebookLM, markdown, source archive, or future formats;
- source scope: single source, source group, saved run, topic, date range;
- content policy: text only, metadata, evidence refs, media metadata, optional
  downloaded media references;
- size and item limits;
- explicit validation before a job starts;
- visible cancel/stop state for long-running exports.

This prevents export behavior from becoming a set of scattered booleans across
the UI and backend.

### 5. Action Registry And Optional Shortcuts

Telegram Desktop has a command/shortcut model. Extractum does not need a full
custom shortcut editor now, but a small action registry would help power-user
workflows and reduce duplicated UI wiring.

Recommended starter actions:

- `analysis.run`
- `analysis.cancel`
- `analysis.openSavedRuns`
- `analysis.focusEvidence`
- `sources.search`
- `sources.sync`
- `sources.cancelJob`
- `export.notebooklm`
- `settings.openProfiles`
- `diagnostics.copySummary`

Each action should define availability, label, and handler ownership. Keyboard
shortcuts and command palette UI can come later.

### 6. Resilient Settings And Data Migration

Telegram Desktop session settings are versioned and defensive about malformed
data. Extractum should apply that discipline to app settings and provider
configuration before settings become more complex.

Recommended scope:

- typed settings records for LLM profiles, provider options, media policy,
  privacy options, and export profiles;
- versioned migration for settings payloads that are stored as JSON or key/value
  records;
- validation plus default repair for malformed records;
- diagnostics that can report repaired or ignored settings without leaking
  values.

This is especially relevant for `app_settings`, LLM profile metadata, YouTube
settings, and future export/media/privacy options.

### 7. Local Privacy And Data Lifecycle Controls

Telegram Desktop protects local account/session data and supports passcode
flows. Extractum already uses OS secure storage and encrypted Telegram session
files, so the next useful layer is data lifecycle visibility.

Recommended scope:

- optional app lock or privacy mode after the core local data model stabilizes;
- "what sensitive data exists locally" summary;
- local archive deletion controls by source, run, or provider;
- diagnostic bundle privacy preview;
- no secret-bearing deep links or logs.

This should remain explicit and user-controlled. Normal source sync and analysis
must not silently change privacy posture.

### 8. Release And Runtime Health

Telegram Desktop has a substantial update pipeline. Extractum does not need to
copy it yet. The useful near-term step is a release and runtime health policy.

Recommended scope:

- release checklist covering `npm run verify`, schema migration state,
  dependency pinning, secret audit, and packaging checks;
- visible runtime health for optional external dependencies such as `yt-dlp`;
- changelog discipline for schema or provider behavior changes;
- signed update strategy only when distribution needs it.

### 9. Internal Deep Links

Telegram Desktop routes many local and remote links. Extractum could eventually
benefit from internal links for support and reproducibility.

Potential future links:

- analysis run details;
- source item or evidence refs;
- settings sections;
- diagnostic bundle summaries.

Do not put secrets, cookies, prompt text, or message content into URLs. Treat
this as a later convenience layer, not a current architecture requirement.

### 10. Localization Readiness

Telegram Desktop has a full localization system. Extractum does not need that
complexity unless multi-language UI becomes a product goal.

If localization becomes real work, use typed message keys and placeholder
validation rather than ad hoc string replacement. Until then, keep UI language
consistent and avoid introducing a localization subsystem prematurely.

## What Not To Copy

- Do not copy Telegram Desktop's account lifecycle, unread state, draft state,
  notification settings, chat-list ordering, or live-client cache model.
- Do not add crash-report upload or telemetry by default.
- Do not add a full custom shortcut editor before there is an action registry
  with real repeated actions.
- Do not copy Telegram Desktop's auto-update complexity before Extractum has a
  distribution requirement that needs it.
- Do not turn settings search into a broad plugin framework.
- Do not introduce app-lock or privacy controls that create a false sense of
  security around unencrypted local database content without a clear threat
  model.

## Suggested Order

1. Define a redaction policy shared by logs, diagnostics, and debug status text.
2. Add a local diagnostic summary/support bundle surface.
3. Introduce a lightweight settings registry and settings search.
4. Define export profile settings before broadening NotebookLM/source-group or
   media export.
5. Add an action registry for repeated workspace commands.
6. Add settings migration/repair helpers as provider and export settings grow.
7. Revisit app lock, internal deep links, and localization only after the
   higher-value foundations are in place.

## Expected Payoff

These changes make Extractum easier to operate and support as local data grows:

- failures become easier to diagnose without leaking private archive content;
- users can find settings as provider and privacy controls grow;
- export behavior remains explicit and testable;
- future media and source-group work has a clear policy surface;
- local privacy work is framed around real data lifecycle controls rather than
  copied chat-client features.
