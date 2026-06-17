import { describe, expect, it } from "vitest";
import { parseSafeMarkdown } from "./safe-markdown";

describe("safe markdown parser", () => {
  it("parses safe block formatting without preserving raw html", () => {
    const blocks = parseSafeMarkdown([
      "## Heading <script>alert(1)</script>",
      "",
      "> quoted **point**",
      "",
      "- first",
      "- second",
      "",
      "| A | B |",
      "| --- | --- |",
      "| **one** | `two` |",
      "",
      "Paragraph with **bold** and `code`.",
    ].join("\n"));

    expect(blocks).toMatchObject([
      {
        kind: "heading",
        level: 2,
        parts: [{ kind: "text", text: "Heading <script>alert(1)</script>" }],
      },
      { kind: "blockquote", parts: [{ kind: "text", text: "quoted " }, { kind: "strong", text: "point" }] },
      { kind: "list", ordered: false, items: [[{ kind: "text", text: "first" }], [{ kind: "text", text: "second" }]] },
      {
        kind: "table",
        headers: [[{ kind: "text", text: "A" }], [{ kind: "text", text: "B" }]],
        rows: [[[{ kind: "strong", text: "one" }], [{ kind: "code", text: "two" }]]],
      },
      {
        kind: "paragraph",
        parts: [
          { kind: "text", text: "Paragraph with " },
          { kind: "strong", text: "bold" },
          { kind: "text", text: " and " },
          { kind: "code", text: "code" },
          { kind: "text", text: "." },
        ],
      },
    ]);
  });
});
