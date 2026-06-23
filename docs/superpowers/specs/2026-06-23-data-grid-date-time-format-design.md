# Data Grid Date/Time Formatting Design

## Goal

All Extractum SVAR data grid wrappers should display explicitly marked date/time columns using the user's browser locale. New tables should get this behavior by using the shared wrapper contract instead of writing one-off `template` functions.

## Scope

This applies to product-facing grid wrappers:

- `ExtractumDataGrid`
- `ExtractumTreeDataGrid`, if it accepts externally supplied columns in the future

The current `Runs` grid is the first target. Its `Created` and `Completed` columns currently render raw ISO strings such as `2026-06-22T21:24:51Z`.

## Column Contract

Columns opt in with an Extractum-specific property:

```ts
dateTimeFormat?: "date" | "datetime" | "time" | false;
```

Examples:

```ts
{ id: "createdAt", header: "Created", dateTimeFormat: "datetime" }
{ id: "publishedDate", header: "Published", dateTimeFormat: "date" }
{ id: "rawCreatedAt", header: "Raw", dateTimeFormat: false }
```

No automatic detection is used. If `dateTimeFormat` is missing, the column is left unchanged.

## Formatting Rules

- `date` displays only the localized date.
- `datetime` displays localized date and time.
- `time` displays only localized time.
- Formatting uses `Intl.DateTimeFormat(undefined, options)` so the browser/user locale is used.
- Supported input values:
  - ISO date/time strings
  - Unix seconds
  - Unix milliseconds
  - `Date` instances
- Numeric timestamps support both seconds and milliseconds by magnitude.
- Empty, nullish, or invalid values render as their original value.

## SVAR Integration

SVAR recommends display formatting through a column `template` or custom `cell`, and `editor: "datepicker"` only for editing. Extractum should follow that model by transforming marked columns before passing them into SVAR:

- If a column has `dateTimeFormat` and no custom `template`, the wrapper adds a `template`.
- If a column already has a custom `template`, the wrapper preserves it.
- `dateTimeFormat: false` disables wrapper date/time formatting.

This keeps the behavior compatible with SVAR and avoids pretending that `type: "datetime"` is a native SVAR column API.

## Implementation Shape

Add a small helper near the grid wrapper, for example:

```text
src/lib/components/extractum-ui/data-grid-date-format.ts
```

The helper should:

- expose the `dateTimeFormat` column type extension;
- parse supported date/time values;
- format via `Intl.DateTimeFormat`;
- return enhanced columns with injected `template` functions where appropriate.

`ExtractumDataGrid` should pass enhanced columns to SVAR. Existing columns without `dateTimeFormat` should be unchanged.

## Testing

Add focused tests for the helper:

- ISO datetime formats through `datetime`.
- Unix seconds and milliseconds both parse correctly.
- `date` omits time.
- `time` omits date.
- invalid values return the original value.
- missing `dateTimeFormat` leaves columns unchanged.
- existing `template` is preserved.

Add or update a contract test so the `Runs` grid columns include `dateTimeFormat: "datetime"` for `Created` and `Completed`.
