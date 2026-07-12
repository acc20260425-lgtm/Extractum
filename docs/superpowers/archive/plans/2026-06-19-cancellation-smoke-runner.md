# Cancellation Smoke Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add one opt-in local command that runs the debug cancellation smoke fixtures for Analysis, Prompt Pack, YouTube source jobs, and Takeout imports.

**Architecture:** Use the existing MCP Bridge WebSocket helpers from `scripts/analysis-smoke-helpers.mjs`. The runner connects to an already running debug Tauri app, invokes existing debug fixture commands, cancels through the normal public Tauri commands, polls terminal state, and always runs cleanup for touched fixtures.

**Tech Stack:** Node ESM, Tauri MCP Bridge, existing Tauri commands, npm scripts.

---

### Task 1: Add Cancellation Smoke Runner

**Files:**
- Create: `scripts/cancellation-smoke.mjs`
- Modify: `package.json`

- [ ] **Step 1: Create `scripts/cancellation-smoke.mjs`**

Implement a Node script that imports `discoverBridge`, `executeJs`, `SmokeAssertionError`, and `expectedAppIdentifier` from `scripts/analysis-smoke-helpers.mjs`.

The script must:
- connect to an already running debug app through MCP Bridge;
- run four scenarios: Analysis, Prompt Pack, YouTube source job, and Takeout import;
- seed each fixture, cancel through the normal command, poll for cancelled state, and clear the fixture in `finally`;
- print `PASS <scenario>` or `FAIL <scenario>`;
- exit with code `1` when any scenario fails.

- [ ] **Step 2: Add npm command**

Add:

```json
"smoke:cancellation": "node scripts/cancellation-smoke.mjs"
```

to `package.json`.

- [ ] **Step 3: Verify script syntax**

Run:

```bash
node --check scripts/cancellation-smoke.mjs
```

Expected: exit code `0`.

- [ ] **Step 4: Run live smoke**

With the debug Tauri app already running and MCP Bridge connected, run:

```bash
npm run smoke:cancellation
```

Expected: all four scenarios print `PASS`, followed by `Cancellation smoke passed.`

- [ ] **Step 5: Commit**

Run:

```bash
git add package.json scripts/cancellation-smoke.mjs docs/superpowers/plans/2026-06-19-cancellation-smoke-runner.md
git commit -m "Add cancellation smoke runner"
```
