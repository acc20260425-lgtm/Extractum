export type SafeMarkdownInlinePart =
  | { kind: "text"; text: string }
  | { kind: "strong"; text: string }
  | { kind: "code"; text: string };

export type SafeMarkdownBlock =
  | { kind: "heading"; level: 2 | 3 | 4; parts: SafeMarkdownInlinePart[] }
  | { kind: "paragraph"; parts: SafeMarkdownInlinePart[] }
  | { kind: "blockquote"; parts: SafeMarkdownInlinePart[] }
  | { kind: "list"; ordered: boolean; items: SafeMarkdownInlinePart[][] }
  | { kind: "code"; text: string }
  | { kind: "table"; headers: SafeMarkdownInlinePart[][]; rows: SafeMarkdownInlinePart[][][] }
  | { kind: "divider" };

export function parseSafeMarkdown(markdown: string): SafeMarkdownBlock[] {
  const lines = markdown.replace(/\r\n/g, "\n").split("\n");
  const blocks: SafeMarkdownBlock[] = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index] ?? "";
    const trimmed = line.trim();

    if (!trimmed) {
      index += 1;
      continue;
    }

    if (/^-{3,}$/.test(trimmed)) {
      blocks.push({ kind: "divider" });
      index += 1;
      continue;
    }

    if (trimmed.startsWith("```")) {
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !(lines[index] ?? "").trim().startsWith("```")) {
        codeLines.push(lines[index] ?? "");
        index += 1;
      }
      if (index < lines.length) index += 1;
      blocks.push({ kind: "code", text: codeLines.join("\n") });
      continue;
    }

    if (isTableStart(lines, index)) {
      const headers = parseTableRow(lines[index] ?? "").map(parseInlineMarkdown);
      index += 2;
      const rows: SafeMarkdownInlinePart[][][] = [];
      while (index < lines.length && isTableRow(lines[index] ?? "")) {
        rows.push(parseTableRow(lines[index] ?? "").map(parseInlineMarkdown));
        index += 1;
      }
      blocks.push({ kind: "table", headers, rows });
      continue;
    }

    const heading = /^(#{1,4})\s+(.+)$/.exec(trimmed);
    if (heading) {
      const level = Math.min(Math.max(heading[1].length, 2), 4) as 2 | 3 | 4;
      blocks.push({ kind: "heading", level, parts: parseInlineMarkdown(heading[2]) });
      index += 1;
      continue;
    }

    if (trimmed.startsWith(">")) {
      const quoteLines: string[] = [];
      while (index < lines.length && (lines[index] ?? "").trim().startsWith(">")) {
        quoteLines.push((lines[index] ?? "").trim().replace(/^>\s?/, ""));
        index += 1;
      }
      blocks.push({ kind: "blockquote", parts: parseInlineMarkdown(quoteLines.join(" ")) });
      continue;
    }

    const listMatch = /^((?:[-*])|\d+\.)\s+(.+)$/.exec(trimmed);
    if (listMatch) {
      const ordered = /\d+\./.test(listMatch[1]);
      const items: SafeMarkdownInlinePart[][] = [];
      while (index < lines.length) {
        const itemMatch = /^((?:[-*])|\d+\.)\s+(.+)$/.exec((lines[index] ?? "").trim());
        if (!itemMatch || /\d+\./.test(itemMatch[1]) !== ordered) break;
        items.push(parseInlineMarkdown(itemMatch[2]));
        index += 1;
      }
      blocks.push({ kind: "list", ordered, items });
      continue;
    }

    const paragraphLines: string[] = [];
    while (index < lines.length && shouldContinueParagraph(lines, index)) {
      paragraphLines.push((lines[index] ?? "").trim());
      index += 1;
    }
    blocks.push({ kind: "paragraph", parts: parseInlineMarkdown(paragraphLines.join(" ")) });
  }

  return blocks;
}

function shouldContinueParagraph(lines: string[], index: number) {
  const trimmed = (lines[index] ?? "").trim();
  if (!trimmed) return false;
  if (/^-{3,}$/.test(trimmed)) return false;
  if (trimmed.startsWith("```")) return false;
  if (isTableStart(lines, index)) return false;
  if (/^(#{1,4})\s+/.test(trimmed)) return false;
  if (trimmed.startsWith(">")) return false;
  if (/^((?:[-*])|\d+\.)\s+/.test(trimmed)) return false;
  return true;
}

function parseInlineMarkdown(text: string): SafeMarkdownInlinePart[] {
  const parts: SafeMarkdownInlinePart[] = [];
  const pattern = /(`([^`]+)`)|(\*\*([^*]+)\*\*)/g;
  let cursor = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > cursor) {
      parts.push({ kind: "text", text: text.slice(cursor, match.index) });
    }
    if (match[2] !== undefined) {
      parts.push({ kind: "code", text: match[2] });
    } else if (match[4] !== undefined) {
      parts.push({ kind: "strong", text: match[4] });
    }
    cursor = match.index + match[0].length;
  }

  if (cursor < text.length) {
    parts.push({ kind: "text", text: text.slice(cursor) });
  }
  return parts.length > 0 ? parts : [{ kind: "text", text }];
}

function isTableStart(lines: string[], index: number) {
  const header = lines[index] ?? "";
  const separator = lines[index + 1] ?? "";
  return isTableRow(header) && parseTableRow(separator).every((cell) => /^:?-{3,}:?$/.test(cell));
}

function isTableRow(line: string) {
  return line.trim().startsWith("|") && line.trim().endsWith("|");
}

function parseTableRow(line: string) {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((cell) => cell.trim());
}
