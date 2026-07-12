# Ignore Claude Local Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ignore the machine-local `.claude/settings.local.json` file without hiding other `.claude` project resources.

**Architecture:** Add one repository-root rule to the existing Claude-local section of `.gitignore`. Verify behavior through Git's ignore engine and worktree status.

**Tech Stack:** Git, `.gitignore`, PowerShell.

## Global Constraints

- Do not modify or delete `.claude/settings.local.json`.
- Do not ignore the entire `.claude` directory.
- Keep existing tracked `.claude` resources eligible for version control.
- Do not edit `docs/value-registry.md`; no application value changes.

---

### Task 1: Ignore the Local Claude Settings File

**Files:**
- Modify: `.gitignore:32-34`

**Interfaces:**
- Consumes: repository-root Git ignore matching.
- Produces: the exact rule `/.claude/settings.local.json`.

- [ ] **Step 1: Verify RED**

Run:

```powershell
git check-ignore -v .claude/settings.local.json
```

Expected: exit code 1 and no matching rule because the file is currently untracked rather than ignored.

- [ ] **Step 2: Add the exact ignore rule**

Insert beside the existing launch-config rule:

```gitignore
# Local Claude configuration (skills under .claude/ stay tracked).
/.claude/launch.json
/.claude/settings.local.json
```

Retain the existing skill-specific rules immediately below it.

- [ ] **Step 3: Verify GREEN**

Run:

```powershell
git check-ignore -v .claude/settings.local.json
git status --short --untracked-files=all
git diff --check
```

Expected: `git check-ignore` reports the new `.gitignore` rule; status lists only `.gitignore` as modified and no longer lists `.claude/settings.local.json`; diff check exits 0.

- [ ] **Step 4: Commit**

```powershell
git add .gitignore
git commit -m "chore: ignore local Claude settings"
```
