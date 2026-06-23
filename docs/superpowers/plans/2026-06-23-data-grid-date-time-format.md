# Data Grid Date/Time Formatting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add shared locale-aware date/time formatting for explicitly marked Extractum DataGrid columns.

**Architecture:** Add a focused TypeScript helper that extends SVAR column configs with an Extractum-owned `dateTimeFormat` property and injects SVAR `template` functions only for opted-in columns. `ExtractumDataGrid` will pass enhanced columns to SVAR, while existing custom templates and unmarked columns remain unchanged. The Runs grid will opt in for `Created` and `Completed`.

**Tech Stack:** Svelte 5, SVAR Svelte Grid, TypeScript, Vitest, browser `Intl.DateTimeFormat`.

---

## File Structure

- Create `src/lib/components/extractum-ui/data-grid-date-format.ts`
  - Owns `dateTimeFormat` types, date parsing, locale formatting, and column enhancement.
- Create `src/lib/components/extractum-ui/data-grid-date-format.test.ts`
  - Unit-tests parsing, formatting, invalid fallback, template preservation, and missing opt-in behavior.
- Modify `src/lib/components/extractum-ui/DataGrid.svelte`
  - Uses enhanced columns before passing them into SVAR.
  - Exports the DataGrid column type from a module script so feature components can use the wrapper contract.
- Modify `src/lib/components/research-projects/ProjectRunsScreen.svelte`
  - Marks `createdAt` and `completedAt` columns with `dateTimeFormat: "datetime"`.
- Modify `src/lib/project-runs-screen-contract.test.ts`
  - Adds contract assertions for the Runs date/time columns.
- Modify `src/lib/research-projects-import-boundary.test.ts`
  - Adds a contract assertion that `ExtractumDataGrid` uses the date/time column enhancer.

---

### Task 1: Date/Time Formatting Helper

**Files:**
- Create: `src/lib/components/extractum-ui/data-grid-date-format.ts`
- Test: `src/lib/components/extractum-ui/data-grid-date-format.test.ts`

- [ ] **Step 1: Write the failing helper tests**

Create `src/lib/components/extractum-ui/data-grid-date-format.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  enhanceDateTimeColumns,
  formatDataGridDateTimeValue,
  parseDataGridDateTimeValue,
  type ExtractumDataGridColumn,
} from "./data-grid-date-format";

describe("data grid date/time formatting", () => {
  it("formats ISO datetime values with localized date and time", () => {
    const formatted = formatDataGridDateTimeValue(
      "2026-06-22T21:24:51Z",
      "datetime",
      "en-US",
      "UTC",
    );

    expect(formatted).toBe("Jun 22, 2026, 21:24");
  });

  it("formats Unix seconds and milliseconds to the same instant", () => {
    const seconds = formatDataGridDateTimeValue(1_719_792_000, "datetime", "en-US", "UTC");
    const milliseconds = formatDataGridDateTimeValue(1_719_792_000_000, "datetime", "en-US", "UTC");

    expect(seconds).toBe("Jun 30, 2024, 16:00");
    expect(milliseconds).toBe(seconds);
  });

  it("formats date values without time", () => {
    const formatted = formatDataGridDateTimeValue("2026-06-22T21:24:51Z", "date", "en-US", "UTC");

    expect(formatted).toBe("Jun 22, 2026");
  });

  it("formats time values without date", () => {
    const formatted = formatDataGridDateTimeValue("2026-06-22T21:24:51Z", "time", "en-US", "UTC");

    expect(formatted).toBe("21:24");
  });

  it("returns invalid values unchanged", () => {
    expect(formatDataGridDateTimeValue("not-a-date", "datetime", "en-US", "UTC")).toBe("not-a-date");
    expect(formatDataGridDateTimeValue("", "datetime", "en-US", "UTC")).toBe("");
    expect(formatDataGridDateTimeValue(null, "datetime", "en-US", "UTC")).toBe(null);
  });

  it("parses Date instances, ISO strings, seconds, and milliseconds", () => {
    expect(parseDataGridDateTimeValue(new Date("2026-06-22T21:24:51Z"))?.toISOString()).toBe(
      "2026-06-22T21:24:51.000Z",
    );
    expect(parseDataGridDateTimeValue("2026-06-22T21:24:51Z")?.toISOString()).toBe(
      "2026-06-22T21:24:51.000Z",
    );
    expect(parseDataGridDateTimeValue(1_719_792_000)?.toISOString()).toBe("2024-06-30T16:00:00.000Z");
    expect(parseDataGridDateTimeValue(1_719_792_000_000)?.toISOString()).toBe("2024-06-30T16:00:00.000Z");
  });

  it("injects templates only for opted-in columns without existing templates", () => {
    const existingTemplate = (value: unknown) => `raw:${String(value)}`;
    const columns: ExtractumDataGridColumn[] = [
      { id: "name", header: "Name" },
      { id: "createdAt", header: "Created", dateTimeFormat: "datetime" },
      { id: "rawCreatedAt", header: "Raw Created", dateTimeFormat: false },
      { id: "publishedAt", header: "Published", dateTimeFormat: "date", template: existingTemplate },
    ];

    const enhanced = enhanceDateTimeColumns(columns, "en-US", "UTC");

    expect(enhanced[0]).toBe(columns[0]);
    expect(enhanced[1]).not.toBe(columns[1]);
    expect(enhanced[1].template?.("2026-06-22T21:24:51Z", {}, enhanced[1])).toBe("Jun 22, 2026, 21:24");
    expect(enhanced[2]).toBe(columns[2]);
    expect(enhanced[3].template).toBe(existingTemplate);
  });
});
```

- [ ] **Step 2: Run the helper tests and verify they fail**

Run:

```bash
npm run test -- src/lib/components/extractum-ui/data-grid-date-format.test.ts
```

Expected: FAIL because `src/lib/components/extractum-ui/data-grid-date-format.ts` does not exist.

- [ ] **Step 3: Implement the helper**

Create `src/lib/components/extractum-ui/data-grid-date-format.ts`:

```ts
import type { IColumnConfig } from "@svar-ui/svelte-grid";

export type ExtractumDateTimeFormat = "date" | "datetime" | "time";

export type ExtractumDataGridColumn = IColumnConfig & {
  dateTimeFormat?: ExtractumDateTimeFormat | false;
};

const UNIX_MILLISECONDS_THRESHOLD = 100_000_000_000;

export function parseDataGridDateTimeValue(value: unknown): Date | null {
  if (value instanceof Date) {
    return Number.isNaN(value.getTime()) ? null : value;
  }

  if (typeof value === "number") {
    if (!Number.isFinite(value)) return null;
    const milliseconds = Math.abs(value) >= UNIX_MILLISECONDS_THRESHOLD ? value : value * 1000;
    const date = new Date(milliseconds);
    return Number.isNaN(date.getTime()) ? null : date;
  }

  if (typeof value === "string") {
    if (!value.trim()) return null;
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? null : date;
  }

  return null;
}

export function dateTimeFormatOptions(kind: ExtractumDateTimeFormat): Intl.DateTimeFormatOptions {
  if (kind === "date") {
    return {
      year: "numeric",
      month: "short",
      day: "numeric",
    };
  }

  if (kind === "time") {
    return {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    };
  }

  return {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  };
}

export function formatDataGridDateTimeValue(
  value: unknown,
  kind: ExtractumDateTimeFormat,
  locale?: string | string[],
  timeZone?: string,
): unknown {
  const date = parseDataGridDateTimeValue(value);
  if (!date) return value;

  const options = dateTimeFormatOptions(kind);
  if (timeZone) {
    options.timeZone = timeZone;
  }

  return new Intl.DateTimeFormat(locale, options).format(date);
}

export function enhanceDateTimeColumns(
  columns: ExtractumDataGridColumn[],
  locale?: string | string[],
  timeZone?: string,
): IColumnConfig[] {
  return columns.map((column) => {
    if (!column.dateTimeFormat || column.template) {
      return column;
    }

    const { dateTimeFormat, ...svarColumn } = column;

    return {
      ...svarColumn,
      template: (value: unknown) => String(formatDataGridDateTimeValue(value, dateTimeFormat, locale, timeZone) ?? ""),
    };
  });
}
```

- [ ] **Step 4: Run helper tests and verify they pass**

Run:

```bash
npm run test -- src/lib/components/extractum-ui/data-grid-date-format.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit helper and tests**

Run:

```bash
git add src/lib/components/extractum-ui/data-grid-date-format.ts src/lib/components/extractum-ui/data-grid-date-format.test.ts
git commit -m "feat: add data grid date formatter"
```

---

### Task 2: Wire Formatter Into ExtractumDataGrid

**Files:**
- Modify: `src/lib/components/extractum-ui/DataGrid.svelte`
- Modify: `src/lib/research-projects-import-boundary.test.ts`

- [ ] **Step 1: Write the failing wrapper contract test**

In `src/lib/research-projects-import-boundary.test.ts`, inside the `routes SVAR Grid through Extractum grid wrappers only` test, after the `visibleOverlay` assertion, add:

```ts
    expect(dataGridSource).toContain("enhanceDateTimeColumns");
    expect(dataGridSource).toContain("enhancedColumns");
```

- [ ] **Step 2: Run the boundary test and verify it fails**

Run:

```bash
npm run test -- src/lib/research-projects-import-boundary.test.ts
```

Expected: FAIL because `DataGrid.svelte` does not yet use `enhanceDateTimeColumns` or `enhancedColumns`.

- [ ] **Step 3: Update DataGrid.svelte**

In `src/lib/components/extractum-ui/DataGrid.svelte`, add this module script before the existing instance script:

```svelte
<script module lang="ts">
  import type { ExtractumDataGridColumn } from "./data-grid-date-format";

  export type { ExtractumDataGridColumn };
</script>
```

In the existing instance script, add the helper import:

```ts
  import { enhanceDateTimeColumns } from "./data-grid-date-format";
```

Change the props type for `columns` from:

```ts
    columns: IColumnConfig[];
```

to:

```ts
    columns: ExtractumDataGridColumn[];
```

Add this derived value after `visibleOverlay`:

```ts
  let enhancedColumns = $derived(enhanceDateTimeColumns(columns));
```

Change the SVAR Grid prop from:

```svelte
        {columns}
```

to:

```svelte
        columns={enhancedColumns}
```

- [ ] **Step 4: Run helper and boundary tests**

Run:

```bash
npm run test -- src/lib/components/extractum-ui/data-grid-date-format.test.ts src/lib/research-projects-import-boundary.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run Svelte check**

Run:

```bash
npm run check
```

Expected: PASS.

- [ ] **Step 6: Commit DataGrid wrapper wiring**

Run:

```bash
git add src/lib/components/extractum-ui/DataGrid.svelte src/lib/research-projects-import-boundary.test.ts
git commit -m "feat: format typed data grid dates"
```

---

### Task 3: Mark Runs Grid Date/Time Columns

**Files:**
- Modify: `src/lib/components/research-projects/ProjectRunsScreen.svelte`
- Modify: `src/lib/project-runs-screen-contract.test.ts`

- [ ] **Step 1: Write the failing Runs grid contract test**

In `src/lib/project-runs-screen-contract.test.ts`, add this test after `uses the Extractum SVAR grid for prompt-pack project runs with update and delete actions`:

```ts
  it("marks run date columns for locale-aware datetime formatting", () => {
    const screenSource = readProjectFile("src/lib/components/research-projects/ProjectRunsScreen.svelte");

    expect(screenSource).toContain('{ id: "createdAt", header: "Created", width: 170, sort: true, dateTimeFormat: "datetime" }');
    expect(screenSource).toContain('{ id: "completedAt", header: "Completed", width: 170, sort: true, dateTimeFormat: "datetime" }');
  });
```

- [ ] **Step 2: Run the Runs screen contract test and verify it fails**

Run:

```bash
npm run test -- src/lib/project-runs-screen-contract.test.ts
```

Expected: FAIL because the Runs grid columns do not yet include `dateTimeFormat: "datetime"`.

- [ ] **Step 3: Mark Created and Completed as datetime columns**

In `src/lib/components/research-projects/ProjectRunsScreen.svelte`, change:

```ts
    { id: "createdAt", header: "Created", width: 170, sort: true },
    { id: "completedAt", header: "Completed", width: 170, sort: true },
```

to:

```ts
    { id: "createdAt", header: "Created", width: 170, sort: true, dateTimeFormat: "datetime" },
    { id: "completedAt", header: "Completed", width: 170, sort: true, dateTimeFormat: "datetime" },
```

- [ ] **Step 4: Run the Runs screen contract test**

Run:

```bash
npm run test -- src/lib/project-runs-screen-contract.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run focused frontend checks**

Run:

```bash
npm run test -- src/lib/components/extractum-ui/data-grid-date-format.test.ts src/lib/research-projects-import-boundary.test.ts src/lib/project-runs-screen-contract.test.ts
npm run check
```

Expected: PASS.

- [ ] **Step 6: Verify the open Runs panel manually**

With the Tauri app running on `/projects/runs`, use Tauri MCP or the browser to inspect the grid. Expected: `Created` and `Completed` no longer render raw ISO strings like `2026-06-22T21:24:51Z`; they render localized date/time strings according to the user's locale.

- [ ] **Step 7: Commit Runs grid opt-in**

Run:

```bash
git add src/lib/components/research-projects/ProjectRunsScreen.svelte src/lib/project-runs-screen-contract.test.ts
git commit -m "feat: localize run grid timestamps"
```

---

## Self-Review

- Spec coverage: The plan implements the explicit `dateTimeFormat` contract, no auto-detection, ISO strings, Unix seconds, Unix milliseconds, `Date` instances, invalid fallback, custom template preservation, and the Runs grid opt-in.
- Placeholder scan: The plan contains no TBD/TODO placeholders and each code-changing step includes concrete code.
- Type consistency: The plan consistently uses `ExtractumDateTimeFormat`, `ExtractumDataGridColumn`, `dateTimeFormat`, `formatDataGridDateTimeValue`, `parseDataGridDateTimeValue`, and `enhanceDateTimeColumns`.
