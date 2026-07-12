# Ignore Claude Local Settings Design

**Date:** 2026-07-12
**Status:** Approved for specification review

## Goal

Keep the machine-local `.claude/settings.local.json` file out of Git status and
future commits without ignoring other project-owned `.claude` resources.

## Design

Add the repository-root rule `/.claude/settings.local.json` to `.gitignore`
beside the existing `/.claude/launch.json` rule. Do not modify or delete the
local settings file. Do not ignore the entire `.claude` directory.

## Verification

- `git check-ignore -v .claude/settings.local.json` identifies the new rule.
- `git status --short --untracked-files=all` no longer lists the local file.
- `.gitignore` is the only implementation file changed.

No application behavior, persisted value, API, or documentation contract is
changed; `docs/value-registry.md` does not require an update.
