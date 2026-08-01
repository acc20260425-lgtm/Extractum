# Testing Infrastructure Redesign Program Index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver the approved testing-infrastructure redesign as evidence-gated slices without making later plans depend on guessed inventories or guessed Rust performance.

**Architecture:** Keep the approved specification as the single behavioral contract. Write and execute one detailed slice plan at a time; each slice commits its implementation and verification evidence before the next plan fixes file-level work. Slice 1 supplies the Rust fast-versus-slow decision, while Slice 2A supplies the census and source-contract ledger that make the migration plans concrete.

**Tech Stack:** Node.js ESM, TypeScript, Vitest 4.1.5, SvelteKit 2, Playwright 1.61, Cargo/Rust 2021, Windows PowerShell, dependency-cruiser, Knip, V8 coverage.

## Global Constraints

- Implement `docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md` at commit `50ce5781` or a descendant.
- Do not add GitHub Actions, branch protection, CI-only thresholds, or hosted-runner assumptions.
- Keep timing observational: no ratification state, fingerprints, rolling counters, eligibility benchmark, modeled critical path, or resource claims.
- Use `npm.cmd`, not plain `npm`, for repository scripts on Windows.
- Use only canonical `src-tauri/target`; never create a slice-specific Cargo target.
- Preserve full `verify` coverage until the replacement scheduler proves equivalent coverage.
- Treat the committed census, ledger, manifests, registries, baselines, and quarantine files as reviewable source; keep raw timings and generated diagnostics under gitignored `artifacts/`.
- End every implementation slice with a clean checkpoint and successful `npm.cmd run verify`.
- Do not push unless the user separately requests it.

---

## Slice Plan Sequence

| Slice | Detailed plan is written when | Required checkpoint output |
| --- | --- | --- |
| 1. Measurement and Rust feasibility | Now | Five-field timing writer, current inventory/duration evidence, Rust compile/check floor, combined test-build time, delta, direct-binary time, Cargo end-to-end time, and an explicit fast-boundary-versus-slow decision |
| 2A. Migration preflight | After Slice 1 is committed | Transition validator integrated into current `verify`, bidirectional runner census, frozen source-contract ledger |
| 2B. Project and browser ownership | After the 2A census and ledger are committed | Disjoint non-empty Vitest projects, Chromium extraction moved to Playwright, cleanup ownership, explicit 20-run lifecycle audit |
| 3. Test migration | After 2B establishes final project owners | Source-dependent tests replaced or deleted with ledger lineage; legacy-contract project and configuration removed |
| 4. Manifest and selector | After Slice 1's Rust decision and Slice 3's final test paths are committed | Validated path-to-owner manifest, fast/slow classification, every unique fast-owner smoke-tested through real feedback selection |
| 5. Process ownership and watch mode | After selector owner commands are stable | 15-second one-shot deadline, descendant cleanup, 5-second watch warning, slow handoff behavior |
| 6. Verification scheduler and performance command | After complete gate inventory is stable | Fixed scheduler constraints, complete non-fail-fast gate execution, 90-second warning, five-run report and optional explicit p95 audit |
| 7. Static analysis, coverage, stability, and quarantine | After final source/test layout exists | dependency-cruiser and Knip gates, expiring baselines, V8 critical-module ratchet, explicit stability/quarantine commands |
| 8. Documentation and final acceptance | After all implementation slices pass | Updated contributor workflow, fresh evidence, acceptance-criteria audit, removal of transitional artifacts |

## Plan Authoring Gate

Before writing each next detailed plan:

1. Read the approved specification and every committed verification record from earlier slices.
2. Re-enumerate the repository paths touched by that slice; do not copy stale filenames from this index.
3. Name exact RED/GREEN tests, focused commands, package checkpoints, and the final full gate.
4. Include no `TODO`, `TBD`, placeholder metric, or speculative file move.
5. Record any approved deviation in the specification before implementation.

## Current Detailed Plan

- `docs/superpowers/plans/2026-08-02-testing-redesign-slice-1-measurement.md`

The next detailed plan is Slice 2A. It is authored only after Slice 1 commits its evidence, because the Rust decision changes which future owner commands and manifest entries are valid.
