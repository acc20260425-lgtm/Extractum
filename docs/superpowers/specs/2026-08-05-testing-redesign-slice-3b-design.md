# Testing Redesign Slice 3B Design

## Purpose

Close the next two largest frozen source-contract cohorts through rendered
component behavior. No new mechanism: no rule, no RepositoryIndex extension,
no scanner and no browser harness.

## Scope

| Legacy file | Rows | Ledger dispositions |
| --- | ---: | --- |
| `src/lib/research-projects-route-contract.test.ts` | SC-000517--SC-000542 (26) | 20 behavior, 3 mixed, 3 delete after the corrections below |
| `src/lib/analysis-report-canvas.test.ts` | SC-000138--SC-000157 (20) | 20 behavior |

Per-row replacement identities and deletion reasons are already frozen in the
ledger. Apart from the four documented row corrections and component-owner
path suffixes below, this slice implements them without renegotiation.

## Ledger Corrections

`SC-000528` names `rule:telegram-phase-8b-authority-integrity` as its
replacement, which is an unrelated Slice 3A authority. It is reassigned to the
`behavior` disposition and the exact replacement declaration
`wires project source Library delete through the main projects route`. The
neighboring SC-000529 remains bound to its distinct `list projects route`
declaration. Stable IDs, hashes, lineage, assertion counts, invariants, and
original test titles remain unchanged.

`SC-000536` becomes a mixed row. Assertion ordinal 1 is deleted because it
reads a global CSS selector from `+layout.svelte`, which jsdom cannot expose
truthfully as applied behavior. Ordinals 2--6 retain the declared behavior
replacement and prove destructive variants and accessible action names
through rendered controls. No other row in either cohort requires the layout
source.

`SC-000537` becomes a mixed row. Assertion ordinals 1--2 retain the declared
behavior replacement and prove the selected marker and project-row class
through rendered navigation. Ordinals 3--4 are deleted because jsdom cannot
truthfully expose the external stylesheet's `:hover` state or applied
box-shadow. This follows the accepted SC-000238 scoped-style precedent.

`SC-000539` becomes a mixed row. Assertion ordinals 1--9 retain the declared
behavior replacement and prove host accessible names, directly imported
column facts, and ordinary labelled sections. Ordinal 10 is deleted because
the empty/loading overlay is rendered inside the SVAR `Grid`, whose internal
output is not a truthful jsdom boundary.

For all three mixed rows, top-level `disposition`, `replacementIds`, and
`deletionReason` are absent. Each subgroup owns its non-empty `invariant` and
resolution fields; each delete subgroup also owns its `deletionReason`. The
row-level invariant remains, while the subgroup invariants are new. The two
subgroups form an exact, non-overlapping partition of every assertion ordinal.

## Component Project Ownership

Both replacements are rendered Svelte suites and must land in the `component`
Vitest project, whose only include pattern is `src/**/*.component.test.ts`.
They therefore use the Slice 3A `*.behavior.component.test.ts` suffix, and the
ledger `replacementIds` paths are updated to match.

Replacement resolution compares the complete declaration title exactly. The
23 required Projects replacement declarations contain only each `it` title,
so they remain top-level and must not gain a `describe` prefix. Additional
top-level smoke declarations with distinct titles are permitted. The
report-canvas IDs include `report canvas component contract >`; that suite
therefore retains `describe("report canvas component contract")` around its 20
replacement declarations.

## Renderability and SVAR Boundary

`DataGrid.svelte` places `role="region"` and `aria-label` on its outer host,
outside the SVAR `Grid`. Slice 3B proves host accessible names with DOM queries
such as `getByRole("region", { name: "Project sources" })`; pure grid-column
and eligibility facts use direct TypeScript imports. It does not assert SVAR
internals. The existing `SourcesTab.component.test.ts` proves only that the
parent mounts and exposes its bulk-bar controls, not that the grid host itself
was asserted.

The cohort spans both live Projects hierarchies: the current Projects routes
and the `/projects/next` route. Of the 12 research-projects components imported
by the legacy contract, only `SourcesTab` has a direct component test.
`ProjectsShell`, `ProjectRail`, `ProjectInspector`, `ProjectRunsTab`,
`ProjectRunsScreen`, `ProjectRunDialog`, `ConnectFromLibrary`,
`ProjectSourceSummary`, `TopCommandBar`, `ProjectWorkspace`, and
`YoutubeSummaryRunsPanel` do not. The Projects cohort starts with minimal
render smokes for those 11 hosts before adding the full replacement
assertions. Tests of similarly named `/projects/next` primitives are not
treated as evidence for these distinct components.

The analysis component directory has no existing component test. The
report-canvas cohort therefore starts with one minimal successful render of
`report-canvas.svelte` before implementing its 20 replacement declarations.

## Route Ownership

The cohorts also depend on four route modules with no existing component-test
precedent: `src/routes/projects/+page.svelte`,
`src/routes/projects/list/+page.svelte`,
`src/routes/projects/next/+page.svelte`, and
`src/routes/analysis/+page.svelte`. Each receives a minimal route render before
its replacement scenarios are implemented. The fixtures mock the route's
`$lib/api/*` calls and event listeners, return explicit unlisten functions,
and use spies to prove invoked APIs and callback outcomes.

None of the four routes imports `$app/*` directly. The main and list Projects
routes, plus the `ProjectsShell` host smoke, transitively import `browser` from
`$app/environment` through `projects-shared.svelte.ts`. The shared
`sveltekit()` Vitest plugin resolves that module, so no manual mock is planned;
the smallest fixture-local `$app/environment` mock is added only if the first
render smoke disproves that assumption. The next Projects and analysis routes
do not traverse this edge. `IconRail.svelte` uses `$app/state`, but no
production Svelte component imports it, so it is outside this cohort's graph.
The layout route is not rendered because SC-000536 ordinal 1 is explicitly
deleted and no retained assertion needs it.

SC-000517 and SC-000528 require route-level API evidence from the rendered main
Projects page. SC-000527 and SC-000529--SC-000532 use route renders for the
route-owned callback wiring, with receiving-component fixtures only for the
child-owned half of the interaction. SC-000533 uses the rendered main and next
Projects routes to prove selected-source synchronization reaches the YouTube
source-job command. SC-000534 uses the rendered main Projects route to prove a
source-job completion refreshes Workspace content. SC-000151--SC-000154
likewise use the rendered analysis route to prove NotebookLM request/open
behavior and prop delivery into `ReportCanvas`; rendering only the accepting
canvas is not sufficient route evidence.

## Constraints

Inherits the Non-Goals and Timing Principles of the program design and the
Global Constraints of Slice 3A. The source-reader guardrail is enforced by
`scripts/testing/test-conventions.test.ts`, not by this document.

Neither new behavior suite may use `?raw`, `readFileSync`, or another direct
production-source reader, and this slice adds no `sourceReaderExceptions`
entry. If a frozen behavior row cannot be demonstrated through rendered DOM,
callbacks, or a direct module import, implementation stops for a design
revision instead of recording nominal evidence.

The expected implementation is two focused component suites totaling roughly
1,000--1,800 lines for 43 behavior-bearing rows, including component smokes and
the four route-mock fixtures; this is a planning estimate, not a metric or
acceptance threshold.

## Verification

Per cohort: `npm.cmd run test:component` plus
`node scripts/validate-testing-transition.mjs`. The `.component.test.ts`
suffix gives each declaration a census owner whose `test:component` script is
present in `scripts/verify.mjs`, satisfying both halves of replacement
resolution. End of slice: `npm.cmd run verify`.
