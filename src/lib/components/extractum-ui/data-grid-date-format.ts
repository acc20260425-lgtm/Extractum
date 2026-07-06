import type { IColumnConfig, ISizeConfig } from "@svar-ui/svelte-grid";

export type ExtractumDateTimeFormat = "date" | "datetime" | "time";

export type ExtractumDataGridColumn = IColumnConfig & {
  dateTimeFormat?: ExtractumDateTimeFormat | false;
};

export type ExtractumDataGridResponsive = Record<
  string,
  {
    sizes?: ISizeConfig;
    columns?: ExtractumDataGridColumn[];
  }
>;

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

export function enhanceDateTimeResponsiveColumns(
  responsive: ExtractumDataGridResponsive | undefined,
  locale?: string | string[],
  timeZone?: string,
): Record<string, { sizes?: ISizeConfig; columns?: IColumnConfig[] }> | undefined {
  if (!responsive) return undefined;

  return Object.fromEntries(
    Object.entries(responsive).map(([breakpoint, config]) => [
      breakpoint,
      {
        ...config,
        columns: config.columns
          ? enhanceDateTimeColumns(config.columns, locale, timeZone)
          : undefined,
      },
    ]),
  );
}
