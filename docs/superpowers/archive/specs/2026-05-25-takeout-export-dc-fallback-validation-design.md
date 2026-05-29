# Takeout Export-DC Fallback Validation Design

## Goal

Close the remaining shifted export-DC fallback validation gap with deterministic
code-backed evidence while keeping natural live evidence caveated.

The slice should prove that Extractum's Takeout export-DC fallback path:

- attempts the shifted export DC;
- falls back only for local transport/session-style failures;
- does not hide Telegram RPC failures;
- records durable fallback provenance through `export_dc_fallback`;
- exposes only sanitized aggregate evidence and warning codes.

## Current State

Takeout import prepares a Telegram Desktop-style shifted DC alias:

```text
export_dc_id = home_dc_id + 4 * 10000
```

`export_dc_invoke` first calls `client.invoke_in_dc(export_dc_id, request)`.
For local transport/session errors it switches to `client.invoke(request)`,
sets `fallback_used = true`, and appends a warning. Telegram RPC errors return
as network errors and must not be masked by fallback.

The production Takeout path wraps this call through
`export_dc_invoke_with_provenance`, which records:

- `telegram_takeout_batches.export_dc_id`;
- `telegram_takeout_batches.used_export_dc = 1`;
- `telegram_takeout_batches.fallback_used = 1` after a fallback transition;
- `ingest_batch_warnings.code = export_dc_fallback`.

The manual validation matrix still marks `Shifted export DC fallback` as
`blocked` because local live runs have not naturally triggered the fallback
branch.

## Chosen Approach

Use deterministic code-backed validation plus a clear live-evidence caveat.

The implementation should add a small testable boundary around export-DC
fallback invocation/provenance. It should simulate fallback-eligible local
errors in Rust tests without calling Telegram and without adding user-facing
force-fallback behavior.

Do not add a production command flag that deliberately forces fallback. The
existing `run_takeout_export_dc_spike(source_id)` remains a live diagnostic for
observed environments, not a simulation interface.

## Behavior To Prove

1. A fallback-eligible local error from the shifted export DC causes exactly one
   fallback transition.
2. After fallback, future calls use the home DC path directly.
3. The warning body may contain low-level transport detail, but durable and UI
   evidence must rely on warning code and sanitized text only.
4. Fallback provenance is inserted before `finalize_ingest_batch`, so batch
   summaries include the warning count and `export_dc_fallback` code.
5. Telegram RPC errors remain terminal errors and do not produce
   `fallback_used = 1` or `export_dc_fallback`.

## Architecture

Keep Telegram integration and SQLite provenance in Rust.

The smallest likely implementation is:

- keep `export_dc_invoke` as the production grammers-backed function;
- introduce an internal testable helper or trait-shaped adapter that represents
  the two invocation operations:
  - shifted DC invoke;
  - home DC invoke;
- use the helper from `export_dc_invoke`, or test the helper directly if doing
  so avoids unnecessary public surface;
- keep `ExportDcAttemptState` as the one-per-batch guard that prevents
  duplicate durable fallback warnings;
- reuse existing provenance functions rather than adding schema or warning
  codes.

This is a narrow backend validation slice. It should not change the Takeout
runtime phases, frontend job DTOs, recovery DTO shape, or normal sync behavior.

## Data Flow

Production flow remains:

```text
prepare_export_dc_alias
  -> export_dc_invoke_with_provenance
    -> record_export_dc_attempt_if_needed
    -> shifted export DC invoke
    -> fallback on eligible local error
    -> home DC invoke
    -> record_export_dc_fallback_if_needed
  -> finalize_ingest_batch
```

The deterministic test flow can replace the grammers client with a fake
invoker that returns:

- fallback-eligible local error for shifted DC;
- success for home DC;
- Telegram RPC error for shifted DC;
- optional repeated calls after fallback.

## Error Handling

Eligible fallback errors remain the existing local/session classes:

- invalid shifted DC;
- I/O;
- transport;
- authentication/session;
- dropped request.

Telegram RPC errors are not fallback-eligible because a server-side result from
the shifted export-DC request should not be silently retried as a local routing
problem.

If durable warning insertion fails, the production Takeout path may still fail.
That is acceptable because provenance durability is part of the validation
contract.

## Tests

Add or strengthen focused Rust tests for:

- `export_dc_invoke` or its internal helper falls back from shifted DC to home
  DC after a fallback-eligible local error;
- once `fallback_used` is true, shifted DC is not attempted again;
- Telegram RPC errors do not fall back;
- `record_export_dc_fallback_if_needed` records `export_dc_fallback` once even
  if the fallback transition is observed more than once;
- validation diagnostics expose `used_export_dc = true`,
  `fallback_used = true`, and warning code `export_dc_fallback` without warning
  bodies.

Existing recovery tests already cover warning-code-only exposure. Keep them,
and extend only if the implementation changes the provenance boundary.

## Documentation Updates

Update the validation matrix row from `blocked` to a code-backed status only
after the deterministic tests pass.

The row should explicitly say that:

- deterministic tests prove the shifted export-DC fallback warning/provenance
  path;
- natural live fallback was not observed in the current environment;
- future live evidence can strengthen the row but is not required to validate
  the local warning/provenance mechanics.

Update `docs/backlog.md` only if the implementation closes the high-priority
export-DC validation outcome. Do not imply that every Telegram environment has
been observed to fall back naturally.

## Privacy Boundary

The slice must not expose:

- Telegram message text;
- source titles or usernames;
- raw Telegram TL payloads;
- raw provider error bodies;
- account, session, auth, header, cookie, or API material.

Safe evidence includes:

- local source id and batch id;
- status and completeness;
- `used_export_dc` and `fallback_used`;
- warning code `export_dc_fallback`;
- aggregate counters and typed/coarse terminal outcomes.

## Acceptance

- Deterministic Rust tests prove shifted export-DC fallback and non-fallback RPC
  behavior.
- Durable provenance records one `export_dc_fallback` warning code for one
  fallback transition.
- Validation diagnostics and recovery-facing state remain sanitized.
- The validation matrix records code-backed fallback coverage and keeps the
  natural-live-fallback caveat visible.
- No production force-fallback command, UI option, schema migration, or
  user-facing behavior change is added.
