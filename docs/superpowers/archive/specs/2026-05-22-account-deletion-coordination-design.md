# Account Deletion Coordination - Historical Note

> Status: shipped and archived. This remains useful because root docs do not
> describe the coordination detail deeply.

## Decision

Account deletion coordinates with source/import state before destructive
cleanup. It should preflight conflicts and avoid deleting data involved in an
active or unsafe workflow.

## Rationale

- Accounts own Telegram session and source relationships that can have active
  imports, recovery state, or provenance.
- Deletion should fail clearly when another workflow still needs the account.
- Preflight blockers are safer than partial deletion followed by repair.

## Preserved Contract

- Check active jobs and source relationships before deletion.
- Report blockers explicitly.
- Keep deletion ordering deterministic so provenance and source cleanup do not
  leave ambiguous state.
- Do not make account deletion responsible for unrelated data repair.
